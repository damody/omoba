-- Run snapshot: automation starts Unreal from a copy of the project's binaries, never from
-- the live omfue tree. A running UnrealEditor(-Cmd) maps every module DLL it loaded, so a
-- test or autopilot run against live omfue made the user's own Visual Studio build fail
-- with LNK1104 (E116). The snapshot copies only Binaries (no PDBs) and links everything
-- else (Content, Config, Saved, plugin sources, and the repo's other top-level entries,
-- which the project reaches through ProjectDir()/..) with directory junctions.
--
-- Layout: target/ue-run/<key>/{omfue/..., scripts -> <repo>/scripts, omb -> <repo>/omb, ...}
-- <key> is derived from the live binaries' names, sizes and times, so a rebuild yields a
-- new snapshot and runs already started keep the one they loaded.
local M = {}

M.KEEP = 3
M.SKIP_BINARY = {pdb = true, exp = true, lib = true, exe = true}
-- Repo entries not linked into a snapshot: the project itself, the snapshot's own parent
-- (a cycle), and VCS/tool metadata.
M.SKIP_REPO = {omfue = true, target = true, ['.git'] = true}
M.SKIP_PROJECT = {Binaries = true, Plugins = true, Intermediate = true}

local function deps_of(deps)
  deps = deps or {}
  local b = require('_bootstrap')
  return {
    path = deps.path or b.lib('path'),
    lfs = deps.lfs or require('lfs'),
    host = deps.host or b.lib('host'),
    sha256 = deps.sha256 or b.lib('hash').sha256,
    log = deps.log or print,
  }
end

local function entries(lfs, dir)
  local list = {}
  for name in lfs.dir(dir) do
    if name ~= '.' and name ~= '..' then list[#list + 1] = name end
  end
  table.sort(list)
  return list
end

local function extension(name) return (name:match('%.([^.]+)$') or ''):lower() end

-- Every binary directory a snapshot copies, as {live, relative} pairs.
local function binary_dirs(d, project)
  local list = {}
  local function add(live, relative)
    if d.path.is_directory(live) then list[#list + 1] = {live = live, relative = relative} end
  end
  add(d.path.join(project, 'Binaries'), 'Binaries')
  for _, plugin in ipairs(entries(d.lfs, d.path.join(project, 'Plugins'))) do
    add(d.path.join(project, 'Plugins', plugin, 'Binaries'), 'Plugins/' .. plugin .. '/Binaries')
  end
  return list
end

local function walk_binaries(d, dir, relative, fn)
  for _, name in ipairs(entries(d.lfs, dir)) do
    local file = d.path.join(dir, name)
    local mode = d.lfs.symlinkattributes(file, 'mode')
    if mode == 'directory' then walk_binaries(d, file, relative .. '/' .. name, fn)
    elseif mode == 'file' and not M.SKIP_BINARY[extension(name)] then fn(file, relative .. '/' .. name) end
  end
end

-- Where a run's own base_content copy replaces the one staged into the plugin (E124).
M.BASE_CONTENT = 'Plugins/OmRuntime/Binaries/Win64/base_content.dll'

-- FNV-1a over "relative|size|mtime" lines: cheap, and changes whenever UBT relinks.
-- OVERLAY_DIGEST (a run's base_content SHA-256) is mixed in, so runs with different
-- content never share a snapshot.
function M.key(project, deps, overlay_digest)
  local d = deps_of(deps)
  local h = 0xcbf29ce484222325
  local function mix(text)
    for byte in text:gmatch('.') do
      h = ((h ~ byte:byte()) * 0x100000001b3) & 0xffffffffffffffff
    end
  end
  for _, dir in ipairs(binary_dirs(d, project)) do
    walk_binaries(d, dir.live, dir.relative, function(file, relative)
      local a = d.lfs.attributes(file)
      mix(relative .. '|' .. a.size .. '|' .. a.modification .. '\n')
    end)
  end
  if overlay_digest then mix(M.BASE_CONTENT .. '|' .. overlay_digest .. '\n') end
  return ('%016x'):format(h)
end

local function copy_file(source, destination, lfs)
  local input = assert(io.open(source, 'rb'))
  local output = assert(io.open(destination, 'wb'))
  while true do
    local chunk = input:read(4 * 1024 * 1024)
    if not chunk then break end
    output:write(chunk)
  end
  input:close(); output:close()
  local modified = lfs.attributes(source, 'modification')
  lfs.touch(destination, modified, modified)
end

local function link(d, link_path, target)
  d.host.call('junction_create', {link = link_path, target = target})
end

-- Deletes a snapshot tree without ever descending into a junction: a junction is removed
-- as a link, so the live Content/Saved/scripts it points to are never touched.
function M.remove_tree(dir, deps)
  local d = deps_of(deps)
  for _, name in ipairs(entries(d.lfs, dir)) do
    local entry = d.path.join(dir, name)
    local mode = d.lfs.symlinkattributes(entry, 'mode')
    if mode == 'link' then
      assert(d.host.call('junction_remove', {link = entry}).removed, 'refusing to delete non-junction link ' .. entry)
    elseif mode == 'directory' then
      M.remove_tree(entry, d)
    else
      assert(os.remove(entry))
    end
  end
  assert(d.lfs.rmdir(dir))
end

local function build(d, root, project, staging, binaries, base_content)
  d.path.mkdir_p(staging)
  for _, name in ipairs(entries(d.lfs, root)) do
    if not M.SKIP_REPO[name] and d.path.is_directory(d.path.join(root, name)) then
      link(d, d.path.join(staging, name), d.path.join(root, name))
    end
  end
  local snapshot_project = d.path.join(staging, 'omfue')
  d.path.mkdir_p(snapshot_project)
  for _, name in ipairs(entries(d.lfs, project)) do
    local live = d.path.join(project, name)
    if d.path.is_directory(live) then
      if not M.SKIP_PROJECT[name] then link(d, d.path.join(snapshot_project, name), live) end
    elseif name:match('%.uproject$') then
      copy_file(live, d.path.join(snapshot_project, name), d.lfs)
    end
  end
  for _, plugin in ipairs(entries(d.lfs, d.path.join(project, 'Plugins'))) do
    local live = d.path.join(project, 'Plugins', plugin)
    if d.path.is_directory(live) then
      local copy = d.path.join(snapshot_project, 'Plugins', plugin)
      d.path.mkdir_p(copy)
      for _, name in ipairs(entries(d.lfs, live)) do
        local item = d.path.join(live, name)
        if d.path.is_directory(item) then
          if name ~= 'Binaries' and name ~= 'Intermediate' then link(d, d.path.join(copy, name), item) end
        else
          copy_file(item, d.path.join(copy, name), d.lfs)
        end
      end
    end
  end
  for _, dir in ipairs(binary_dirs(d, binaries)) do
    walk_binaries(d, dir.live, dir.relative, function(file, relative)
      local destination = d.path.join(snapshot_project, relative)
      d.path.mkdir_p(d.path.parent(destination))
      copy_file(file, destination, d.lfs)
    end)
  end
  if base_content then
    local destination = d.path.join(snapshot_project, M.BASE_CONTENT)
    d.path.mkdir_p(d.path.parent(destination))
    copy_file(base_content, destination, d.lfs)
  end
end

-- Without Intermediate/TargetInfo.json the editor runs UBT -Mode=QueryTargets at startup,
-- which waits on UBT's machine-wide mutex for any running build (a session's shadow build,
-- the user's VS build). Seed it from live omfue, stamped now, so the cache stays valid
-- unless a *.Target.cs changes afterwards. Runs on reuse too: older snapshots lack it.
local function seed_target_info(d, project, snapshot_project)
  local live = d.path.join(project, 'Intermediate', 'TargetInfo.json')
  local seeded = d.path.join(snapshot_project, 'Intermediate', 'TargetInfo.json')
  if d.path.is_file(live) and not d.path.is_file(seeded) then
    d.path.write(seeded, d.path.read(live, true), true, true)
  end
end

-- Old snapshots are renamed aside first: Windows refuses to rename a directory whose DLLs
-- a running Unreal still maps, so a snapshot in use is skipped, never half-deleted.
local function prune(d, base, current)
  local ready = {}
  for _, name in ipairs(entries(d.lfs, base)) do
    local dir = d.path.join(base, name)
    if name ~= current and d.path.is_directory(dir) then
      local marker = d.path.join(dir, 'ready.txt')
      if name:match('^%x+$') and d.path.is_file(marker) then
        ready[#ready + 1] = {dir = dir, time = d.lfs.attributes(marker, 'modification')}
      elseif not name:match('^%x+$') and os.time() - (d.lfs.attributes(dir, 'modification') or 0) > 3600 then
        ready[#ready + 1] = {dir = dir, time = 0, stale = true}
      end
    end
  end
  table.sort(ready, function(a, b) return a.time > b.time end)
  for index, item in ipairs(ready) do
    if item.stale or index >= M.KEEP then
      local trash = item.dir .. '.trash-' .. os.time()
      if item.dir:match('%.trash%-') or os.rename(item.dir, trash) then
        local target = item.dir:match('%.trash%-') and item.dir or trash
        local ok, err = pcall(M.remove_tree, target, d)
        if not ok then d.log('[ue-run-snapshot] could not finish removing ' .. target .. ': ' .. tostring(err)) end
      end
    end
  end
end

-- The live path behind a path Unreal reports from inside a run snapshot (E137). Unreal run
-- from target/ue-run/<key>/omfue reports Saved/... files through the snapshot's junctions;
-- every linked entry keeps its relative place, so the live path is the same path under the
-- repo root. Copied binaries (Binaries, Intermediate, plugin Binaries) have no live
-- counterpart and come back unchanged, as does any path outside a snapshot.
function M.live_path(root, file, deps)
  local d = deps_of(deps)
  local absolute = d.path.absolute(file, root)
  local normal = absolute:gsub('\\', '/')
  local base = d.path.join(root, 'target', 'ue-run'):gsub('\\', '/') .. '/'
  if normal:sub(1, #base):lower() ~= base:lower() then return absolute end
  local relative = normal:sub(#base + 1):match('^[^/]+/(.+)$')
  if not relative then return absolute end
  local lower = relative:lower()
  if lower:match('^omfue/binaries/') or lower:match('^omfue/intermediate/')
    or lower:match('^omfue/plugins/[^/]+/binaries/') then return absolute end
  return d.path.join(root, relative)
end

-- Returns {root, project_dir, uproject, key}. The caller passes these to Unreal instead
-- of the live omfue paths. options.binaries names another project directory (a session's
-- shadow build) whose Binaries are copied instead of live omfue's, so a session can test
-- its own build while the user's editor holds the live DLLs; OM_UE_RUN_BINARIES sets it
-- for every launcher in a workflow. options.base_content is a run's own base_content copy
-- (run_artifacts.lua): it replaces the plugin's staged one in the snapshot, so the renderer
-- loads exactly what that run's server and runtimes load.
function M.prepare(root, deps, options)
  local d = deps_of(deps)
  local base_content = options and options.base_content
  local digest = base_content and d.sha256(base_content):lower()
  local project = d.path.join(root, 'omfue')
  local binaries = (options and options.binaries) or os.getenv('OM_UE_RUN_BINARIES')
  binaries = binaries and binaries ~= '' and d.path.absolute(binaries, root) or project
  assert(d.path.is_directory(d.path.join(binaries, 'Plugins')), 'run snapshot binaries project has no Plugins: ' .. binaries)
  local base = d.path.join(root, 'target', 'ue-run')
  d.path.mkdir_p(base)
  for attempt = 1, 3 do
    local key = M.key(binaries, d, digest)
    local snapshot = d.path.join(base, key)
    local result = {root = snapshot, key = key, project_dir = d.path.join(snapshot, 'omfue'),
      uproject = d.path.join(snapshot, 'omfue', 'om.uproject')}
    if d.path.is_file(d.path.join(snapshot, 'ready.txt')) then
      seed_target_info(d, project, result.project_dir)
      prune(d, base, key)
      return result
    end
    -- A snapshot directory without ready.txt is never usable (ready.txt is written before the
    -- rename): a renderer started from a snapshot that was pruned under it recreates
    -- omfue/Saved and Intermediate there. Left alone it blocks the rename below forever (E142).
    if d.path.is_directory(snapshot) then
      local ok, err = pcall(M.remove_tree, snapshot, d)
      if not ok then d.log('[ue-run-snapshot] could not clear incomplete ' .. snapshot .. ': ' .. tostring(err)) end
    end
    local staging = snapshot .. '.partial-' .. os.time() .. '-' .. math.random(0, 999999)
    build(d, root, project, staging, binaries, base_content)
    seed_target_info(d, project, d.path.join(staging, 'omfue'))
    -- A build that relinked a DLL while we copied makes the copy inconsistent: retry.
    if M.key(binaries, d, digest) == key then
      d.path.write(d.path.join(staging, 'ready.txt'), key .. '\n' .. os.date('!%Y-%m-%dT%H:%M:%SZ') .. '\n', true)
      if os.rename(staging, snapshot) or d.path.is_file(d.path.join(snapshot, 'ready.txt')) then
        if d.path.is_directory(staging) then M.remove_tree(staging, d) end
        d.log('[ue-run-snapshot] ' .. snapshot)
        prune(d, base, key)
        return result
      end
    end
    M.remove_tree(staging, d)
    d.log('[ue-run-snapshot] live binaries changed while copying (attempt ' .. attempt .. '); retrying')
  end
  error('live omfue binaries kept changing while preparing a run snapshot; is a build running?')
end

return M
