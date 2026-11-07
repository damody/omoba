-- Source snapshots for headless builds (E347). A headless batch (verify_m6_batch,
-- run_moba_headless_batch) builds its script DLLs and moba-headless from its own copy of the
-- sources into its own cargo target, so it never queues on the repo build lock (target/locks/
-- build_ue_moba): only builds that stage into Unreal need that lock. The E332 guarantee holds
-- without it: both are built from one snapshot, and their embedded content hashes must match.
--
--   local snapshot = require('source_snapshot')
--   local built = snapshot.build_headless(root, 'verify_m6_batch', {into = OUTPUT .. '/bin'})
--   -- built.scripts_dir (the copied script DLL set), built.exe (its moba-headless), built.content_hash
--
-- The snapshot is target/snapshots/<name>/tree, kept between runs so cargo builds incrementally
-- (target/snapshots/<name>/cargo-target). It mirrors land_to_live's source set. No lock is taken:
-- the live tree is listed (path, size, modification) before and after mirroring, and the mirror
-- is repeated until nothing changed while it ran, so the copy is one state of live, never half a
-- landing. Copies keep the source modification time and a file is copied again whenever its size
-- or time differs, so an edit in the same second as an earlier copy is not missed.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local M = {}
M.SKIP = { target = true, ['.git'] = true, log = true }
M.MAX_PASSES = 10

local function deps_of(deps)
  deps = deps or {}
  local b = require('_bootstrap')
  return { path = deps.path or b.lib('path'), lfs = deps.lfs or require('lfs'),
    process = deps.process or b.lib('process'), log = deps.log or print,
    entries = deps.entries or require('land_to_live').TREE }
end

local function names(lfs, dir)
  local list = {}
  for name in lfs.dir(dir) do if name ~= '.' and name ~= '..' then list[#list + 1] = name end end
  table.sort(list)
  return list
end

-- Every file under the source set: relative path -> 'size:modification'.
function M.listing(root, d)
  local out = {}
  local function walk(dir, relative)
    for _, name in ipairs(names(d.lfs, dir)) do
      if not M.SKIP[name] then
        local full, rel = d.path.join(dir, name), relative .. '/' .. name
        local a = d.lfs.attributes(full)
        if a and a.mode == 'directory' then walk(full, rel)
        elseif a and a.mode == 'file' then out[rel] = a.size .. ':' .. a.modification end
      end
    end
  end
  for _, entry in ipairs(d.entries) do
    local full = d.path.join(root, entry)
    local a = d.lfs.attributes(full)
    if a and a.mode == 'directory' then walk(full, entry)
    elseif a and a.mode == 'file' then out[entry] = a.size .. ':' .. a.modification end
  end
  return out
end

local function same(a, b)
  for key, value in pairs(a) do if b[key] ~= value then return false end end
  for key in pairs(b) do if a[key] == nil then return false end end
  return true
end

local function copy(d, from, to, modification)
  d.path.mkdir_p(d.path.parent(to))
  local input = assert(io.open(from, 'rb'))
  local output = assert(io.open(to, 'wb'))
  while true do
    local chunk = input:read(4 * 1024 * 1024)
    if not chunk then break end
    output:write(chunk)
  end
  input:close(); output:close()
  assert(d.lfs.touch(to, modification, modification))
end

local function remove_tree(d, target)
  local a = d.lfs.symlinkattributes(target)
  if not a then return end
  if a.mode == 'directory' then
    for _, name in ipairs(names(d.lfs, target)) do remove_tree(d, d.path.join(target, name)) end
    d.lfs.rmdir(target)
  else os.remove(target) end
end

-- Makes TREE equal to LISTING (taken from ROOT); returns how many files it copied or removed.
local function mirror(root, tree, listing, d)
  local changed = 0
  local wanted = {}
  for rel, stamp in pairs(listing) do
    wanted[rel] = true
    local target = d.path.join(tree, rel)
    local t = d.lfs.attributes(target)
    if not t or t.mode ~= 'file' or (t.size .. ':' .. t.modification) ~= stamp then
      if t and t.mode == 'directory' then remove_tree(d, target) end
      copy(d, d.path.join(root, rel), target, tonumber(stamp:match(':(%d+)$')))
      changed = changed + 1
    end
  end
  for rel in pairs(M.listing(tree, d)) do
    if not wanted[rel] then os.remove(d.path.join(tree, rel)); changed = changed + 1 end
  end
  return changed
end

-- Brings target/snapshots/<NAME>/tree to one consistent state of live; returns the tree path.
function M.capture(root, name, deps)
  local d = deps_of(deps)
  assert(type(name) == 'string' and name:match('^[%w_%-]+$'), 'snapshot name must be a word')
  local tree = d.path.join(root, 'target', 'snapshots', name, 'tree')
  d.path.mkdir_p(tree)
  for pass = 1, M.MAX_PASSES do
    local before = M.listing(root, d)
    local changed = mirror(root, tree, before, d)
    local after = M.listing(root, d)
    if same(before, after) then
      d.log(('[snapshot] %s: %d file(s) updated, live unchanged during pass %d'):format(name, changed, pass))
      return tree
    end
    d.log(('[snapshot] %s: live changed during pass %d; mirroring again'):format(name, pass))
  end
  error(('live kept changing for %d snapshot passes; is a landing loop running?'):format(M.MAX_PASSES), 0)
end


-- One user of a snapshot at a time: two batches with the same snapshot name would overwrite each
-- other's sources and binaries. A directory lock (mkdir is atomic) held from capture to copy.
-- E434: owner.txt names the holding Lua process (pid and creation token, the lua-host
-- caller_owned identity, as build_lock E346). A waiter takes over at once when that process is
-- gone: a batch stopped mid-build left the lock for the whole stale window and the next batch
-- waited hours. A lock with no owner.txt (written right after mkdir) older than
-- LOCK_UNOWNED_SECONDS was left by a holder that died in between, or by the earlier version; a
-- lock older than LOCK_STALE_SECONDS is taken over in any case.
M.LOCK_WAIT_SECONDS = 3 * 3600
M.LOCK_STALE_SECONDS = 3 * 3600
M.LOCK_UNOWNED_SECONDS = 60
function M.lock(d, base)
  local dir = d.path.join(base, 'lock')
  local owner = d.path.join(dir, 'owner.txt')
  local now = d.now or os.time
  local started = now()
  local announced = false
  while not d.lfs.mkdir(dir) do
    local since = d.lfs.attributes(dir, 'modification')
    local text = d.path.is_file(owner) and d.path.read(owner) or nil
    local pid, token = (text or ''):match('^(%d+)\n(%d+)\n')
    local gone
    if pid then
      if not d.process.inspect_owned(math.tointeger(tonumber(pid)), token) then gone = 'its process ' .. pid .. ' has exited' end
    elseif since and now() - since > M.LOCK_UNOWNED_SECONDS then
      gone = 'it names no holder'
    end
    if not gone and since and now() - since > M.LOCK_STALE_SECONDS then gone = 'it is older than the stale limit' end
    if gone then
      d.log('[snapshot] taking over the lock of ' .. base .. ': ' .. gone)
      os.remove(owner)
      d.lfs.rmdir(dir)
    else
      assert(now() - started < M.LOCK_WAIT_SECONDS, 'timed out waiting for snapshot lock ' .. dir)
      if not announced then d.log('[snapshot] another batch uses ' .. base .. ' (process ' .. tostring(pid) .. '); waiting'); announced = true end
      d.sleep_ms(5000)
    end
  end
  local holder = d.caller()
  d.path.write(owner, ('%d\n%s\n%s\n'):format(holder.pid, tostring(holder.creation_token),
    tostring(holder.path or holder.executable or '')), true)
  return setmetatable({}, { __close = function() os.remove(owner); d.lfs.rmdir(dir) end })
end

-- Captures NAME and builds the script DLL set and moba-headless (release, compiled content) from
-- it into its own cargo target, then copies them into OPTIONS.into (a new directory), so the
-- batch runs binaries no later build changes (E125). OPTIONS: env (extra cargo environment;
-- compiled.environment is added), into, no_build (copy the snapshot's last build as it is),
-- under_lock (called with the tree before the lock is released: anything read from live after
-- the capture can be newer than the binaries, and the next batch may mirror the tree, E665).
-- Returns { tree, scripts_dir = into, exe = into/moba-headless.exe, content_hash }.
function M.build_headless(root, name, options, deps)
  local d = deps_of(deps)
  d.sleep_ms = (deps and deps.sleep_ms) or require('_bootstrap').lib('time').sleep_ms
  options = options or {}
  assert(type(options.into) == 'string', 'build_headless needs options.into')
  local compiled = require('moba_compiled_content')
  local script_dlls = require('script_dlls')
  local base = d.path.join(root, 'target', 'snapshots', name)
  d.path.mkdir_p(base)
  d.caller = (deps and deps.caller) or function() return require('tools.lua.lib.host').call('caller_owned', {}) end
  local held <close> = M.lock(d, base)
  local tree = d.path.join(base, 'tree')
  local target = d.path.join(base, 'cargo-target')
  if not options.no_build then
    tree = M.capture(root, name, deps)
    local environment = compiled.environment(options.env)
    environment.CARGO_TARGET_DIR = target
    -- The build starts only while its measured peak commit is free (E441, memory_admission.lua).
    local admission = require('memory_admission')
    admission.new(admission.BUILD_COMMIT_BYTES, { margin = admission.BUILD_MARGIN, status = deps and deps.memory_status,
      log = d.log }):wait(d.sleep_ms)
    for _, args in ipairs({
      script_dlls.cargo_args({ 'build', '--manifest-path', d.path.join(tree, 'scripts', 'Cargo.toml') }, compiled.feature, { '--release' }),
      { 'build', '--manifest-path', d.path.join(tree, 'omb', 'Cargo.toml'), '-p', 'omobab', '--bin', 'moba-headless',
        '--features', compiled.feature, '--release' },
    }) do
      local result = d.process.run('cargo', args, { cwd = tree, env = environment, check = false })
      assert(result.exit_code == 0, 'snapshot build failed: ' .. tostring(result.stderr or ''):sub(-3000))
    end
  end
  local release = d.path.join(target, 'release')
  local files = { 'moba-headless.exe' }
  for _, module in ipairs(script_dlls.modules()) do files[#files + 1] = module.file end
  d.path.mkdir_p(options.into)
  local artifacts = require('run_artifacts')
  local content, seen = nil, {}
  for _, file in ipairs(files) do
    local from = d.path.join(release, file)
    assert(d.path.is_file(from), 'snapshot build has no ' .. file .. ' (' .. from .. ')')
    local to = d.path.join(options.into, file)
    d.path.write(to, d.path.read(from, true), false, true)
    -- One snapshot, one content (E332): every copy that embeds the catalog hash embeds the same.
    local hash = artifacts.content_hash(to)
    if hash then
      seen[#seen + 1] = file .. '=' .. hash
      assert(content == nil or content == hash, 'snapshot binaries carry different content: ' .. table.concat(seen, ', '))
      content = hash
    end
  end
  assert(content, 'no snapshot binary carries a content hash marker')
  d.log(('[snapshot] %s: %d binaries, content %s'):format(name, #files, content))
  if options.under_lock then options.under_lock(tree) end
  return { tree = tree, scripts_dir = options.into, exe = d.path.join(options.into, 'moba-headless.exe'), content_hash = content }
end

return M
