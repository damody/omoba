-- Retention for cargo target directories the tooling owns (land_to_live's target/land/cargo-target,
-- source snapshots, shadow builds). Cargo never removes what it no longer uses: every change of
-- features, flags or dependencies leaves the old `<crate>-<hash>` outputs in deps/, and rustc leaves
-- old incremental sessions behind. target/land grew to 122 GB this way (2026-11-08).
--
--   tools\lua\lua.exe scripts\cargo_target_retention.lua DIR [--keep-days N] [--drop-pdb] [--all] [--dry-run]
--
-- Per profile directory (debug, release, ...) of DIR:
--   * deps/: files are grouped by `<crate>-<hash>`. The newest group of every crate stays; an older
--     group stays while it was written within the keep window. The rest goes, with the group's
--     .fingerprint/ and build/ entries, so cargo rebuilds it if it is wanted again.
--   * incremental/: `<crate>-<hash>` directories follow the same rule; inside each kept one only
--     the newest session (s-*) and its lock stay.
--   * --drop-pdb also removes every .pdb (debug info no check needs; land builds without it).
-- --all removes DIR entirely (a finished shadow's target). Directories are removed without
-- descending into junctions (ue_run_snapshot.remove_tree); a junction is removed as a link.
-- Only run it on a target no cargo is building in (land_to_live holds the build lock).
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local M = {}

M.KEEP_DAYS = 2

local function deps_of(deps)
  deps = deps or {}
  return { lfs = deps.lfs or require('lfs'), now = deps.now or os.time,
    remove_tree = deps.remove_tree or function(dir) require('ue_run_snapshot').remove_tree(dir) end }
end

local function entries(lfs, dir)
  local list = {}
  if lfs.attributes(dir, 'mode') ~= 'directory' then return list end
  for name in lfs.dir(dir) do
    if name ~= '.' and name ~= '..' then list[#list + 1] = name end
  end
  table.sort(list)
  return list
end

-- size and newest modification time under PATH (a file or a directory, junctions not followed)
local function measure(lfs, path)
  local mode = lfs.symlinkattributes(path, 'mode')
  if mode == 'file' then
    return lfs.attributes(path, 'size') or 0, lfs.attributes(path, 'modification') or 0
  elseif mode == 'directory' then
    local size, newest = 0, lfs.attributes(path, 'modification') or 0
    for _, name in ipairs(entries(lfs, path)) do
      local s, m = measure(lfs, path .. '/' .. name)
      size, newest = size + s, math.max(newest, m)
    end
    return size, newest
  end
  return 0, lfs.symlinkattributes(path, 'modification') or 0
end

local function remove(d, path, stats, dry_run)
  local size = measure(d.lfs, path)
  stats.bytes, stats.removed = stats.bytes + size, stats.removed + 1
  if dry_run then return end
  if d.lfs.symlinkattributes(path, 'mode') == 'directory' then d.remove_tree(path)
  else assert(os.remove(path)) end
end

-- `name-0123456789abcdef.ext` / `libname-hash.rlib` -> crate name, group key (`name-hash`, the
-- unit's .fingerprint/ and build/ entry); nil for other files
local function deps_group(file)
  local stem, hash = file:match('^(.-)%-(%x%x%x%x%x%x%x%x%x%x%x%x%x%x%x%x)[%.]')
  if not stem then stem, hash = file:match('^(.-)%-(%x%x%x%x%x%x%x%x%x%x%x%x%x%x%x%x)$') end
  if not stem then return nil end
  local crate = stem:gsub('^lib', '')
  return crate, crate .. '-' .. hash
end

-- groups -> the keys to remove: every group but the newest of its crate and those written since CUTOFF
local function stale(groups, cutoff)
  local newest = {}
  for key, g in pairs(groups) do
    if not newest[g.crate] or groups[newest[g.crate]].time < g.time then newest[g.crate] = key end
  end
  local out = {}
  for key, g in pairs(groups) do
    if newest[g.crate] ~= key and g.time < cutoff then out[#out + 1] = key end
  end
  table.sort(out)
  return out
end

local function prune_profile(d, profile, cutoff, options, stats)
  local deps = profile .. '/deps'
  local groups = {}
  for _, file in ipairs(entries(d.lfs, deps)) do
    local path = deps .. '/' .. file
    if options.drop_pdb and file:match('%.pdb$') then remove(d, path, stats, options.dry_run)
    else
      local crate, key = deps_group(file)
      if key then
        local g = groups[key] or { crate = crate, time = 0, files = {}, kinds = {} }
        groups[key] = g
        g.files[#g.files + 1] = path
        g.time = math.max(g.time, d.lfs.attributes(path, 'modification') or 0)
        local ext = file:match('%.(%w+)$')
        if ext and ext ~= 'd' and ext ~= 'o' then g.kinds[ext] = true end
      end
    end
  end
  -- a crate's check (rmeta), build (rlib) and test (exe) outputs are different units in use at once:
  -- the newest is kept per crate and kind of output
  for _, g in pairs(groups) do
    local kinds = {}
    for ext in pairs(g.kinds) do kinds[#kinds + 1] = ext end
    table.sort(kinds)
    g.crate = g.crate .. ' ' .. table.concat(kinds, ',')
  end
  for _, key in ipairs(stale(groups, cutoff)) do
    for _, path in ipairs(groups[key].files) do remove(d, path, stats, options.dry_run) end
    for _, side in ipairs({ '.fingerprint', 'build' }) do
      local path = profile .. '/' .. side .. '/' .. key
      if d.lfs.symlinkattributes(path, 'mode') then remove(d, path, stats, options.dry_run) end
    end
  end
  local incremental = profile .. '/incremental'
  local dirs = {}
  for _, name in ipairs(entries(d.lfs, incremental)) do
    local crate = name:match('^(.+)%-%w+$')
    local path = incremental .. '/' .. name
    if crate and d.lfs.symlinkattributes(path, 'mode') == 'directory' then
      local _, time = measure(d.lfs, path)
      dirs[name] = { crate = crate, time = time }
    end
  end
  local gone = {}
  for _, name in ipairs(stale(dirs, cutoff)) do
    gone[name] = true
    remove(d, incremental .. '/' .. name, stats, options.dry_run)
  end
  for name in pairs(dirs) do if not gone[name] then
    local dir = incremental .. '/' .. name
    local sessions = {}
    for _, entry in ipairs(entries(d.lfs, dir)) do
      local id = entry:match('^(s%-%w+%-%w+)%-%w+$')
      if id and d.lfs.symlinkattributes(dir .. '/' .. entry, 'mode') == 'directory' then
        local _, time = measure(d.lfs, dir .. '/' .. entry)
        sessions[#sessions + 1] = { id = id, path = dir .. '/' .. entry, time = time }
      end
    end
    table.sort(sessions, function(a, b) return a.time > b.time end)
    for index = 2, #sessions do
      remove(d, sessions[index].path, stats, options.dry_run)
      local lock = dir .. '/' .. sessions[index].id .. '.lock'
      if d.lfs.symlinkattributes(lock, 'mode') == 'file' then remove(d, lock, stats, options.dry_run) end
    end
  end end
end

-- Prunes cargo target DIR; options: keep_days, drop_pdb, all, dry_run. Returns { bytes, removed }.
function M.prune(dir, options, deps)
  options = options or {}
  local d = deps_of(deps)
  local stats = { bytes = 0, removed = 0 }
  dir = dir:gsub('\\', '/'):gsub('/$', '')
  if d.lfs.attributes(dir, 'mode') ~= 'directory' then return stats end
  if options.all then remove(d, dir, stats, options.dry_run); return stats end
  local cutoff = d.now() - (options.keep_days or M.KEEP_DAYS) * 86400
  for _, name in ipairs(entries(d.lfs, dir)) do
    local profile = dir .. '/' .. name
    if d.lfs.symlinkattributes(profile, 'mode') == 'directory'
      and (d.lfs.attributes(profile .. '/deps', 'mode') == 'directory' or d.lfs.attributes(profile .. '/incremental', 'mode') == 'directory') then
      prune_profile(d, profile, cutoff, options, stats)
    end
  end
  return stats
end

function M.main(argv)
  local dir, options = nil, {}
  local i = 1
  while i <= #argv do
    local v = argv[i]
    if v == '--keep-days' then options.keep_days = assert(tonumber(argv[i + 1]), '--keep-days needs a number'); i = i + 1
    elseif v == '--drop-pdb' then options.drop_pdb = true
    elseif v == '--all' then options.all = true
    elseif v == '--dry-run' then options.dry_run = true
    elseif not dir and not v:match('^%-') then dir = v
    else error('usage: tools/lua/lua.exe scripts/cargo_target_retention.lua DIR [--keep-days N] [--drop-pdb] [--all] [--dry-run]', 0) end
    i = i + 1
  end
  assert(dir, 'usage: tools/lua/lua.exe scripts/cargo_target_retention.lua DIR [--keep-days N] [--drop-pdb] [--all] [--dry-run]')
  local stats = M.prune(dir, options)
  print(('%s %d entr%s, %.1f GB in %s'):format(options.dry_run and 'would remove' or 'removed', stats.removed,
    stats.removed == 1 and 'y' or 'ies', stats.bytes / 1e9, dir))
  return stats
end

if not pcall(debug.getlocal, 4, 1) then M.main(arg) end
return M
