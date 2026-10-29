local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local path, lfs = b.lib('path'), require('lfs')
local real_process = b.lib('process')
local land = require('land_to_live')

-- E143: live inputs land only after codegen/cargo/UBT pass on "live + the change", together.
local root = path.join(b.root, 'target', 'land-test-' .. os.time() .. '-' .. math.random(0, 99999))
local checks = 0
local function check(value, message) assert(value, message); checks = checks + 1 end
local function put(relative, text)
  local file = path.join(root, relative); path.mkdir_p(path.parent(file)); path.write(file, text, true, true); return file
end
local function get(relative) return path.read(path.join(root, relative), true) end
local function rejects(fn, pattern)
  local ok, err = pcall(fn)
  check(not ok and tostring(err):find(pattern, 1, true), 'expected ' .. pattern .. ', got ' .. tostring(err))
end

put('scripts/lua_data/templates/heroes.lua', 'heroes-v1\n')
put('scripts/Cargo.toml', '[workspace]\n')
put('omfue/codegen/Cargo.toml', '[package]\n')
put('omfue/Plugins/OmRuntime/Source/OmRuntime/A.cpp', 'line1\nline2\nline3\n')
put('omoba-core/src/lib.rs', 'core\n')

-- Injected collaborators: every check is recorded; FAIL makes one fail.
local calls, fail, locked, synced, built = {}, {}, 0, 0, 0
local process = {run = function(exe, args, options)
  if exe == 'git' then return real_process.run(exe, args, options) end
  local text = exe .. ' ' .. table.concat(args, ' ')
  calls[#calls + 1] = text
  for pattern in pairs(fail) do if text:find(pattern, 1, true) then return {exit_code = 1, stdout = 'boom ' .. pattern, stderr = ''} end end
  return {exit_code = 0, stdout = '', stderr = ''}
end}
local shadow = {
  mirror = function(from, to) synced = synced + 0; local stage = require('stage_ue_shadow'); return stage.mirror(from, to, {root = root}, {target = true}) end,
  sync = function(project) synced = synced + 1; path.mkdir_p(project) end,
  build = function() built = built + 1; if fail.UBT then error('UBT failed', 0) end end,
}
local lock = {acquire = function() locked = locked + 1; return setmetatable({}, {__close = function() locked = locked - 1 end}) end}
local deps = {root = root, process = process, shadow = shadow, lock = lock, log = function() end}
local function reset() calls, fail, synced, built = {}, {}, 0, 0 end
local function ran(pattern) for _, c in ipairs(calls) do if c:find(pattern, 1, true) then return true end end return false end

-- Path classification decides the checks.
local need = land.needs({'scripts/lua_data/templates/heroes.lua'})
check(need.codegen and need.cargo and need.ubt and not need.bridge, 'lua_data needs codegen, cargo and UBT')
need = land.needs({'omfue/Plugins/OmRuntime/Source/OmRuntime/A.cpp'})
check(need.ubt and not need.cargo, 'C++ needs UBT, not cargo')
need = land.needs({'omfue/bridge/src/lib.rs'})
check(need.bridge and need.cargo and not need.ubt, 'bridge needs its cargo check')
need = land.needs({'omb/src/transport/kcp_transport.rs'})
check(need.server and need.cargo and not need.ubt, 'authority code needs its cargo check')
need = land.needs({'omoba-core/src/lib.rs'})
check(need.server and need.cargo, 'shared crates also check the authority')
need = land.needs({'docs/readme.md'})
check(need.codegen and not need.cargo and not need.ubt, 'any landing still proves codegen')
rejects(function() land.relative(root, path.join(root, 'target', 'x'), {path = path}) end, 'not a live input')
rejects(function() land.relative(root, 'C:/elsewhere/x.lua', {path = path}) end, 'outside the repository')

-- A failing check writes nothing.
local new = put('work/heroes.lua', 'heroes-v2\n')
fail['cargo check base_content'] = true; fail['check --manifest-path'] = true
local report = land.run({changes = {{source = new, live = 'scripts/lua_data/templates/heroes.lua'}}}, deps)
check(not report.passed and not report.landed and get('scripts/lua_data/templates/heroes.lua') == 'heroes-v1\n', 'failed check: live untouched')
check(report.error:find('boom', 1, true), 'the failure reason is reported')
check(locked == 0, 'the build lock is released')
-- The candidate got the change; live did not.
check(path.read(path.join(root, 'target/land/tree/scripts/lua_data/templates/heroes.lua'), true) == 'heroes-v2\n', 'candidate tree has the change')

-- All checks pass: lands, with codegen run over the candidate lua_data, cargo and UBT.
reset()
report = land.run({changes = {{source = new, live = 'scripts/lua_data/templates/heroes.lua'}}}, deps)
check(report.passed and report.landed and get('scripts/lua_data/templates/heroes.lua') == 'heroes-v2\n', 'passing landing writes live')
check(ran('--content-root ' .. path.join(root, 'target', 'land', 'tree', 'scripts', 'lua_data')), 'codegen reads the candidate, not live')
-- E159: the Lua build steps' loader (content_builder) reads the same candidate templates.
check(ran('lua_content_check.lua ' .. path.join(root, 'target', 'land', 'tree', 'scripts', 'lua_data')), 'Lua content loaders read the candidate too')
check(ran('cargo check') and built == 1 and synced == 1, 'cargo and UBT ran on the candidate')
check(not ran(path.join(root, 'omfue', 'Plugins', 'OmRuntime', 'Source', 'OmGenerated')), 'live OmGenerated is never written by a check')

-- 3-way: a change made on an older base merges onto the newer live file.
reset()
local base = put('work/A.base.cpp', 'line1\nline2\nline3\n')
put('omfue/Plugins/OmRuntime/Source/OmRuntime/A.cpp', 'line1-live\nline2\nline3\n')       -- someone else's later edit
local mine = put('work/A.mine.cpp', 'line1\nline2\nline3-mine\n')
report = land.run({changes = {{source = mine, live = 'omfue/Plugins/OmRuntime/Source/OmRuntime/A.cpp', base = base}}}, deps)
check(report.landed and get('omfue/Plugins/OmRuntime/Source/OmRuntime/A.cpp') == 'line1-live\nline2\nline3-mine\n', 'both edits kept')
check(path.read(path.join(root, 'target/land/shadow/Plugins/OmRuntime/Source/OmRuntime/A.cpp'), true) == 'line1-live\nline2\nline3-mine\n',
  'the shadow compiled the merged file')
check(not ran('cargo check'), 'C++-only change skips cargo')
-- A conflicting change refuses everything.
reset()
local conflicting = put('work/A.conflict.cpp', 'line1-other\nline2\nline3\n')
rejects(function() land.run({changes = {{source = conflicting, live = 'omfue/Plugins/OmRuntime/Source/OmRuntime/A.cpp', base = base}}}, deps) end, 'conflict')
check(get('omfue/Plugins/OmRuntime/Source/OmRuntime/A.cpp') == 'line1-live\nline2\nline3-mine\n', 'conflict: live untouched')
check(locked == 0, 'lock released after a refused plan')

-- Several files land together or not at all; a UBT failure keeps all of them out.
reset(); fail.UBT = true
local two = put('work/heroes3.lua', 'heroes-v3\n')
local core = put('work/lib.rs', 'core-v2\n')
report = land.run({changes = {{source = two, live = 'scripts/lua_data/templates/heroes.lua'}, {source = core, live = 'omoba-core/src/lib.rs'}}}, deps)
check(not report.landed and get('scripts/lua_data/templates/heroes.lua') == 'heroes-v2\n' and get('omoba-core/src/lib.rs') == 'core\n', 'all or nothing')

-- Live changing during the checks refuses the landing.
reset()
local racing = {run = function(exe, args, options)
  local result = process.run(exe, args, options)
  if exe == 'cargo' and table.concat(args, ' '):find('check', 1, true) then put('omoba-core/src/lib.rs', 'core-raced\n') end
  return result
end}
report = land.run({changes = {{source = core, live = 'omoba-core/src/lib.rs'}}}, {root = root, process = racing, shadow = shadow, lock = lock, log = function() end})
check(not report.landed and report.error:find('changed during the checks', 1, true) and get('omoba-core/src/lib.rs') == 'core-raced\n', 'race refused')

-- Dry run and the manifest form.
reset()
put('work/manifest.json', ('[{"source":%q,"live":"scripts/lua_data/templates/heroes.lua"}]'):format(two:gsub('\\', '/')))
report = land.run({manifest = 'work/manifest.json', dry_run = true}, deps)
check(report.passed and not report.landed and get('scripts/lua_data/templates/heroes.lua') == 'heroes-v2\n', 'dry run writes nothing')
local options = land.parse({'--change', 'a.lua=scripts/x.lua@base.lua', '--checks', 'codegen,ubt', '--dry-run'})
check(options.changes[1].source == 'a.lua' and options.changes[1].live == 'scripts/x.lua' and options.changes[1].base == 'base.lua'
  and options.checks == 'codegen,ubt' and options.dry_run, 'argument parsing')

-- Removal: SRC '-' removes LIVE with the batch, only while live still equals BASE.
reset()
put('scripts/lua_data/retired.lua', 'retired-v1\n')
local retired_base = put('work/retired.base.lua', 'retired-v1\n')
local moved = put('work/moved.lua', 'moved-v1\n')
report = land.run({changes = {{source = '-', live = 'scripts/lua_data/retired.lua', base = retired_base},
  {source = moved, live = 'scripts/test_fixtures/moved.lua'}}}, deps)
check(report.landed and not path.exists(path.join(root, 'scripts/lua_data/retired.lua'))
  and get('scripts/test_fixtures/moved.lua') == 'moved-v1\n', 'a removal lands with the batch')
check(not path.exists(path.join(root, 'target/land/tree/scripts/lua_data/retired.lua')), 'the checks ran without the removed file')
put('scripts/lua_data/retired.lua', 'retired-v2\n')
rejects(function() land.run({changes = {{source = '-', live = 'scripts/lua_data/retired.lua', base = retired_base}}}, deps) end,
  'changed since its base')
check(get('scripts/lua_data/retired.lua') == 'retired-v2\n', 'a changed file is never removed')
rejects(function() land.run({changes = {{source = '-', live = 'scripts/lua_data/retired.lua'}}}, deps) end, 'needs its BASE')
rejects(function() land.run({changes = {{source = '-', live = 'scripts/lua_data/absent.lua', base = retired_base}}}, deps) end,
  'does not exist')
options = land.parse({'--change', '-=scripts/x.lua@base.lua'})
check(options.changes[1].source == '-' and options.changes[1].live == 'scripts/x.lua', 'removal argument parsing')

-- check_live_consistent: the same checks with no changes, never writing live.
reset()
local consistent = require('check_live_consistent').main({}, deps)
check(consistent.passed and #consistent.files == 0 and consistent.dry_run, 'consistency check passes with no changes')
check(ran('--content-root') and ran('check --manifest-path') and built == 0, 'codegen + cargo, no UBT unless --ubt')

local function rm(target)
  for name in lfs.dir(target) do
    if name ~= '.' and name ~= '..' then
      local entry = path.join(target, name)
      if lfs.symlinkattributes(entry, 'mode') == 'directory' then rm(entry) else os.remove(entry) end
    end
  end
  lfs.rmdir(target)
end
-- E171: line endings. Only an end-of-line change between base and live is no conflict, and the
-- result keeps live's line endings (this test root is not tracked by git; a tracked file gets what
-- a checkout writes).
check(land.eol_of('a\r\nb\r\n') == 'crlf' and land.eol_of('a\nb\n') == 'lf' and land.eol_of('ab') == nil, 'eol detection')
check(land.with_eol('a\r\nb\n', 'crlf') == 'a\r\nb\r\n' and land.with_eol('a\r\nb\n', 'lf') == 'a\nb\n', 'eol conversion')
put('omoba-core/src/eol.rs', 'one\ntwo\nthree\nfour\n')
local eol_base = put('work/eol-base.rs', 'one\r\ntwo\r\nthree\r\n')
local eol_change = put('work/eol-change.rs', 'ONE\r\ntwo\r\nthree\r\n')
reset()
report = land.run({changes = {{source = eol_change, live = 'omoba-core/src/eol.rs', base = eol_base}}}, deps)
check(report.landed, 'a base/live line-ending difference is not a merge conflict: ' .. tostring(report.error))
check(get('omoba-core/src/eol.rs') == 'ONE\ntwo\nthree\nfour\n', 'merged, with live\'s line endings')
-- E187: writing live is all or nothing too. A file another process holds made io.open fail with
-- EINVAL halfway through a batch and left live with half of it; the batch only compiled as a whole.
local function norm(file) return (path.absolute(file):gsub('\\', '/'):lower()) end
local function blocking(relative, times)
  local target, left = norm(path.join(root, relative)), times
  return setmetatable({write = function(file, ...)
    if left > 0 and norm(file) == target then left = left - 1; error('Invalid argument', 0) end
    return path.write(file, ...)
  end}, {__index = path})
end
local function pair_run(file_path)
  return land.run({changes = {{source = put('work/a.rs', 'a-new\n'), live = 'omoba-core/src/a.rs'},
    {source = put('work/b.rs', 'b-new\n'), live = 'omoba-core/src/b.rs'}}},
    {root = root, process = process, shadow = shadow, lock = lock, log = function() end, path = file_path})
end
reset(); put('omoba-core/src/a.rs', 'a-live\n'); put('omoba-core/src/b.rs', 'b-live\n')
report = pair_run(blocking('omoba-core/src/b.rs', math.huge))
check(not report.landed and tostring(report.error):find('nothing landed (1 written file(s) restored)', 1, true),
  'a failing live write refuses the batch: ' .. tostring(report.error))
check(get('omoba-core/src/a.rs') == 'a-live\n' and get('omoba-core/src/b.rs') == 'b-live\n', 'the written file is restored')
check(locked == 0, 'lock released after a failed write')
reset()
report = pair_run(blocking('omoba-core/src/b.rs', 1))
check(report.landed and get('omoba-core/src/a.rs') == 'a-new\n' and get('omoba-core/src/b.rs') == 'b-new\n',
  'a write blocked once is retried: ' .. tostring(report.error))
rm(root)
print(('land to live: %d checks passed'):format(checks))
