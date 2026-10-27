local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local path, lfs, json = b.lib('path'), require('lfs'), b.lib('json')
local stage = require('ue_bridge_stage')
local shadow = require('stage_ue_shadow')

-- E124: shadow projects are mirrored and staged by one entry, and a launch checks a project's
-- bridge against what was staged with it, not against whatever the shared target holds now.
local root = path.join(b.root, 'target', 'ue-shadow-stage-test-' .. os.time() .. '-' .. math.random(0, 99999))
local checks = 0
local function check(value, message) assert(value, message); checks = checks + 1 end
local function put(relative, text, time)
  local file = path.join(root, relative)
  path.mkdir_p(path.parent(file))
  path.write(file, text, true, true)
  if time then lfs.touch(file, time, time) end
  return file
end
local function rejects(fn, pattern)
  local ok, err = pcall(fn)
  check(not ok and tostring(err):find(pattern, 1, true), 'expected rejection with ' .. pattern .. ', got ' .. tostring(err))
end
local logs = {}
local quiet = {log = function(line) logs[#logs + 1] = line end}

-- Live project and current builds.
put('omfue/om.uproject', '{"v":1}')
put('omfue/Source/OmGame/Game.cpp', 'game')
put('omfue/Config/DefaultGame.ini', 'ini')
put('omfue/Binaries/Win64/UnrealEditor-OmGame.dll', 'live-binary')
put('omfue/Intermediate/Build/Win64/state.bin', 'ubt-state')
put('omfue/Intermediate/Other/junk.obj', 'junk')
put('omfue/Plugins/OmRuntime/OmRuntime.uplugin', '{}')
put('omfue/Plugins/OmRuntime/Source/OmRuntime/A.cpp', 'a')
put('omfue/Plugins/OmRuntime/Binaries/Win64/UnrealEditor-OmRuntime.dll', 'live-runtime')
put('omfue/Plugins/OmRuntime/Intermediate/x.obj', 'x')
put('omfue/bridge/target/debug/om_bridge.dll', 'bridge-v1')
put('omfue/bridge/target/debug/om_bridge.dll.lib', 'lib-v1')
put('scripts/target/debug/base_content.dll', 'content-v1')
local header = put('build/om_bridge.h', 'header-v1')

-- Sync: a new shadow gets sources plus a one-time build-state seed.
local project = path.join(root, 'shadow')
local deps = {root = root, log = quiet.log}
shadow.sync(project, deps)
check(path.read(path.join(project, 'om.uproject')) == '{"v":1}', 'uproject copied')
check(path.read(path.join(project, 'Source/OmGame/Game.cpp')) == 'game', 'Source mirrored')
check(path.read(path.join(project, 'Plugins/OmRuntime/Source/OmRuntime/A.cpp')) == 'a', 'plugin source mirrored')
check(path.read(path.join(project, 'Binaries/Win64/UnrealEditor-OmGame.dll')) == 'live-binary', 'new shadow seeded with Binaries')
check(path.read(path.join(project, 'Intermediate/Build/Win64/state.bin')) == 'ubt-state', 'new shadow seeded with UBT state')
check(not path.exists(path.join(project, 'Intermediate/Other')), 'only Intermediate/Build is seeded')
check(not path.exists(path.join(project, 'Plugins/OmRuntime/Intermediate')), 'plugin Intermediate never mirrored')
-- A later sync mirrors sources only: the shadow's own binaries stay; removed sources go.
put('shadow/Binaries/Win64/UnrealEditor-OmGame.dll', 'shadow-binary')
put('shadow/Plugins/OmRuntime/Binaries/Win64/UnrealEditor-OmRuntime.dll', 'shadow-runtime')
put('omfue/Source/OmGame/Game.cpp', 'game-v2', os.time() + 5)
os.remove(path.join(root, 'omfue/Config/DefaultGame.ini'))
put('omfue/Config/DefaultEngine.ini', 'engine')
check(shadow.sync(project, deps) == 2, 'only changed files are copied')
check(path.read(path.join(project, 'Source/OmGame/Game.cpp')) == 'game-v2', 'changed source updated')
check(not path.exists(path.join(project, 'Config/DefaultGame.ini')), 'removed source removed')
check(path.read(path.join(project, 'Binaries/Win64/UnrealEditor-OmGame.dll')) == 'shadow-binary', 'shadow Binaries kept')
check(path.read(path.join(project, 'Plugins/OmRuntime/Binaries/Win64/UnrealEditor-OmRuntime.dll')) == 'shadow-runtime',
  'shadow plugin Binaries kept')
check(shadow.sync(project, deps) == 0, 'an unchanged tree copies nothing')
-- E136: live changed before the shadow's last build (another session landed it earlier): the copy is
-- still stamped newer than any build output, so UBT recompiles it.
local before = os.time()
put('omfue/Plugins/OmRuntime/Source/OmRuntime/A.cpp', 'a-v2', before - 3600)
check(shadow.sync(project, deps) == 1, 'a source with an older time is still copied')
local copy = path.join(project, 'Plugins/OmRuntime/Source/OmRuntime/A.cpp')
check(path.read(copy) == 'a-v2', 'older-time source updated')
check(lfs.attributes(copy, 'modification') >= before, 'copy stamped now, never in the past')
check(shadow.sync(project, deps) == 0, 'a fresh copy is not copied again')
rejects(function() shadow.sync(path.join(root, 'omfue'), deps) end, 'cannot be live omfue')

-- Stage: header, lib, bridge and base_content land in the shadow with a record; live untouched.
local record = stage.stage(project, stage.sources(root, header), quiet)
check(path.read(path.join(project, stage.DLL), true) == 'bridge-v1', 'bridge staged')
check(path.read(path.join(project, stage.HEADER), true) == 'header-v1', 'header staged')
check(path.read(path.join(project, stage.LIB), true) == 'lib-v1', 'import library staged')
check(path.read(path.join(project, stage.BASE_CONTENT), true) == 'content-v1', 'base_content staged')
check(record.artifacts.bridge and record.artifacts.header and not record.artifacts.base_content,
  'the record covers the bridge set; base_content is run-owned')
check(json.decode(path.read(path.join(project, stage.RECORD))).artifacts.bridge.sha256 == record.artifacts.bridge.sha256,
  'record written next to the bridge')
check(not path.exists(path.join(root, 'omfue', stage.DLL)), 'live plugin never written by a shadow stage')

-- Launch check: passes with the record even after another session rebuilds the bridge.
check(stage.verify_launch(root, project, quiet).artifacts[1].role == 'bridge', 'staged project passes')
put('omfue/bridge/target/debug/om_bridge.dll', 'bridge-v2')
logs = {}
local report = stage.verify_launch(root, project, {log = function(line) logs[#logs + 1] = line end})
check(report.artifacts[1].sha256 == record.artifacts.bridge.sha256, 'a newer bridge build elsewhere does not block the launch')
check(#logs == 1 and logs[1]:find('not the newest bridge build', 1, true), 'but the launch says it is not the newest')
-- A copied-in DLL or header that was not staged together is refused.
put('shadow/' .. stage.DLL, 'bridge-v2')
rejects(function() stage.verify_launch(root, project, quiet) end, 'differs from its stage record')
put('shadow/' .. stage.DLL, 'bridge-v1')
put('shadow/' .. stage.HEADER, 'header-v2')
rejects(function() stage.verify_launch(root, project, quiet) end, 'header differs from its stage record')
-- A project staged before records existed keeps the old rule: equal to the current build.
put('legacy/' .. stage.DLL, 'bridge-v2')
check(stage.verify_launch(root, path.join(root, 'legacy'), quiet), 'legacy project equal to the build passes')
put('legacy/' .. stage.DLL, 'bridge-v1')
rejects(function() stage.verify_launch(root, path.join(root, 'legacy'), quiet) end, 'differs from current build')
rejects(function() stage.verify_launch(root, path.join(root, 'missing'), quiet) end, 'missing staged bridge')

-- Stage commands: codegen into the shadow, both Rust builds, cbindgen into the shadow.
local runs = {}
put('omfue/bridge/target/debug/om_bridge.dll', 'bridge-v3')
local fake = {root = root, log = quiet.log, process = {run = function(exe, args, options)
  runs[#runs + 1] = exe .. ' ' .. table.concat(args, ' ')
  if exe == 'cbindgen' then path.write(args[#args - 1], 'header-v3', true, true) end
  return {exit_code = 0}
end}}
shadow.stage(project, fake)
local text = table.concat(runs, '\n')
check(text:find(path.join(project, 'Plugins', 'OmRuntime', 'Source', 'OmGenerated'), 1, true), 'codegen writes the shadow')
check(not text:find(path.join(root, 'omfue', 'Plugins'), 1, true), 'no stage command writes live omfue')
check(text:find('compiled-content-only', 1, true) and text:find('bridge', 1, true), 'bridge and base_content built')
check(path.read(path.join(project, stage.HEADER), true) == 'header-v3' and path.read(path.join(project, stage.DLL), true) == 'bridge-v3',
  'restage replaces header and bridge together')
check(stage.verify_launch(root, project, quiet), 'restaged project passes')
fake.process.run = function() return {exit_code = 1} end
rejects(function() shadow.stage(project, fake) end, 'failed with exit 1')

local function rm(target)
  for name in lfs.dir(target) do
    if name ~= '.' and name ~= '..' then
      local entry = path.join(target, name)
      if lfs.symlinkattributes(entry, 'mode') == 'directory' then rm(entry) else os.remove(entry) end
    end
  end
  lfs.rmdir(target)
end
rm(root)
print(('ue shadow stage: %d checks passed'):format(checks))
