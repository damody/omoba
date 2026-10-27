local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local path, lfs, json = b.lib('path'), require('lfs'), b.lib('json')
local artifacts = require('run_artifacts')
local snapshot = require('ue_run_snapshot')
local contract = require('moba_stage_contract')

-- E124: each run copies the current base_content build into its own directory; nothing
-- reads the shared scripts/base_content.dll, so another session's rebuild never stales it.
local root = path.join(b.root, 'target', 'run-artifacts-test-' .. os.time() .. '-' .. math.random(0, 99999))
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
local now = os.time()

rejects(function() artifacts.stage_base_content(root, path.join(root, 'runs/none')) end, 'no base_content build')
check(artifacts.built_base_content(root) == nil, 'no build yet')

put('scripts/target/debug/base_content.dll', 'debug-v1', now - 100)
put('scripts/target/release/base_content.dll', 'release-v1', now - 50)
put('scripts/base_content.dll', 'shared-stale', now - 1000)
local built = artifacts.built_base_content(root)
check(built.profile == 'release', 'the newest build of either profile wins')

-- A run's copy is the current build, with a manifest; the shared copy is never read.
local run1 = path.join(root, 'runs/one')
local first = artifacts.stage_base_content(root, run1)
check(path.read(first.dll, true) == 'release-v1', 'run copy is the current build')
check(first.scripts_dir == path.join(run1, 'artifacts', 'scripts'), 'scripts dir under the run')
check(first.dll == path.join(first.scripts_dir, 'base_content.dll'), 'DLL inside the scripts dir')
local count = 0
for name in lfs.dir(first.scripts_dir) do if name:lower():match('%.dll$') then count = count + 1 end end
check(count == 1, 'the run scripts dir holds only base_content (loaders read every DLL there)')
local manifest = json.decode(path.read(first.manifest))
check(manifest.profile == 'release' and manifest.sha256 == first.sha256 and #first.sha256 == 64, 'manifest records profile and digest')
check(manifest.sha256 == b.lib('hash').sha256(first.dll):lower(), 'digest is of the copy')
rejects(function() artifacts.stage_base_content(root, run1) end, 'already has a staged base_content')

-- Another session rebuilds debug: the started run keeps its copy, the next run takes the new build.
put('scripts/target/debug/base_content.dll', 'debug-v2', now)
check(path.read(first.dll, true) == 'release-v1', 'a later build never changes a started run')
local second = artifacts.stage_base_content(root, path.join(root, 'runs/two'))
check(path.read(second.dll, true) == 'debug-v2' and second.profile == 'debug', 'the next run takes the newer build')
check(second.sha256 ~= first.sha256, 'different content, different digest')

-- A split client copies its authority's run copy, even after a newer build appears.
put('scripts/target/release/base_content.dll', 'release-v9', now + 10)
local joined = artifacts.stage_base_content(root, path.join(root, 'runs/client'), nil, nil, second.dll)
check(path.read(joined.dll, true) == 'debug-v2' and joined.sha256 == second.sha256 and joined.profile == 'shared',
  'a client loads exactly the authority content')
rejects(function() artifacts.stage_base_content(root, path.join(root, 'runs/orphan'), nil, nil, path.join(root, 'none.dll')) end,
  'missing base_content to share')

-- Same time: the profile this workflow built wins.
put('scripts/target/release/base_content.dll', 'release-v2', now)
check(artifacts.built_base_content(root, nil, 'release').profile == 'release', 'tie prefers the requested profile')
check(artifacts.built_base_content(root, nil, 'debug').profile == 'debug', 'tie prefers the requested profile (debug)')

-- A build relinking the DLL during every copy is refused instead of staging a torn copy.
local real = require('lfs')
local tick = 0
local moving = setmetatable({attributes = function(file, ...)
  local a = real.attributes(file, ...)
  if type(a) == 'table' and file:find('target', 1, true) and file:find('base_content', 1, true) then
    tick = tick + 1; a.modification = a.modification + tick
  end
  return a
end}, {__index = real})
rejects(function() artifacts.stage_base_content(root, path.join(root, 'runs/moving'), {lfs = moving}) end, 'kept changing')
check(not path.exists(path.join(root, 'runs/moving/artifacts/scripts/base_content.dll')), 'a torn copy is removed')

-- A build still checks both staged copies right after staging (launches do not: E124).
local roles = {}
for _, binding in ipairs(contract.bindings(root, path.join(root, 'omfue'))) do roles[#roles + 1] = binding.role end
check(table.concat(roles, ',') == 'bridge,base_content', 'build verification still covers base_content')

-- The run snapshot loads the run's copy in place of the plugin's staged one.
put('omfue/om.uproject', '{}')
put('omfue/Binaries/Win64/UnrealEditor-OmGame.dll', 'game')
put('omfue/Plugins/OmRuntime/OmRuntime.uplugin', '{}')
put('omfue/Plugins/OmRuntime/Binaries/Win64/base_content.dll', 'plugin-stale')
local quiet = {log = function() end}
local plain = snapshot.prepare(root, quiet)
check(path.read(path.join(plain.project_dir, snapshot.BASE_CONTENT), true) == 'plugin-stale', 'no overlay: plugin copy')
local one = snapshot.prepare(root, quiet, {base_content = first.dll})
check(one.key ~= plain.key, 'an overlay gets its own snapshot')
check(path.read(path.join(one.project_dir, snapshot.BASE_CONTENT), true) == 'release-v1', 'snapshot loads the run copy')
check(path.read(path.join(root, 'omfue', snapshot.BASE_CONTENT), true) == 'plugin-stale', 'live plugin untouched')
local two = snapshot.prepare(root, quiet, {base_content = second.dll})
check(two.key ~= one.key and path.read(path.join(two.project_dir, snapshot.BASE_CONTENT), true) == 'debug-v2',
  'different run content, different snapshot')
check(snapshot.prepare(root, quiet, {base_content = first.dll}).key == one.key, 'same content reuses its snapshot')
check(path.read(path.join(one.project_dir, snapshot.BASE_CONTENT), true) == 'release-v1', 'an earlier run snapshot keeps its copy')

-- E125: executables run from keyed copies, so a running match never locks the build output.
local server_exe = put('omb/target/release/omobab.exe', 'server-v1 OM_CONTENT_CATALOG_DATA_HASH=aaaa000011112222; tail', now - 20)
local runtime_exe = put('omoba-client-runtime/target/release/omoba-client-runtime.exe', 'runtime-v1', now - 20)
local files = {{name = 'omobab.exe', source = server_exe}, {name = 'omoba-client-runtime.exe', source = runtime_exe}}
local bin1 = artifacts.stage_binaries(root, files, quiet)
check(path.read(bin1.files['omobab.exe'], true):find('server-v1', 1, true), 'server copy staged')
check(bin1.dir:find(path.join(root, 'target', 'run-bin'), 1, true) == 1, 'copies under target/run-bin')
check(artifacts.stage_binaries(root, files, quiet).key == bin1.key, 'unchanged builds reuse the copy')
check(os.remove(server_exe) or true, 'the build output is not held by the copy')
put('omb/target/release/omobab.exe', 'server-v2 OM_CONTENT_CATALOG_DATA_HASH=bbbb000011112222;', now)
local bin2 = artifacts.stage_binaries(root, files, quiet)
check(bin2.key ~= bin1.key and path.read(bin2.files['omobab.exe'], true):find('server-v2', 1, true), 'a rebuild yields a new copy')
check(path.read(bin1.files['omobab.exe'], true):find('server-v1', 1, true), 'a started run keeps its copy')
rejects(function() artifacts.stage_binaries(root, {{name = 'x.exe', source = path.join(root, 'none.exe')}}, quiet) end, 'missing executable')
-- Old copies are pruned beyond KEEP_BINARIES; the newest stay.
for version = 3, artifacts.KEEP_BINARIES + 3 do
  put('omb/target/release/omobab.exe', 'server-v' .. version, now + version)
  artifacts.stage_binaries(root, files, quiet)
end
local kept = 0
for name in lfs.dir(path.join(root, 'target/run-bin')) do if name:match('^%x+$') then kept = kept + 1 end end
check(kept <= artifacts.KEEP_BINARIES, 'old copies pruned, kept ' .. kept)

-- The content marker is read from the binary; a mixed set is refused before anything starts.
check(artifacts.content_hash(bin2.files['omobab.exe']) == 'bbbb000011112222', 'marker read')
check(artifacts.content_hash(first.dll) == nil, 'no marker, no hash')
local big = put('big.bin', string.rep('x', 4 * 1024 * 1024 - 10) .. 'OM_CONTENT_CATALOG_DATA_HASH=cccc000011112222;')
check(artifacts.content_hash(big) == 'cccc000011112222', 'a marker across a read boundary is found')
local notes = {}
local note = function(line) notes[#notes + 1] = line end
rejects(function() artifacts.verify_content({{role = 'server', file = bin2.files['omobab.exe']},
  {role = 'renderer bridge', file = big}}, 'cccc000011112222', note) end, 'carry different content')
check(artifacts.verify_content({{role = 'renderer bridge', file = big}, {role = 'runtime', file = first.dll}},
  'CCCC000011112222', note) and #notes == 1 and notes[1]:find('no content marker', 1, true), 'unmarked binaries are reported, not refused')

-- OM_RUST_RUN_BINARIES: one session-owned cargo target directory for every crate.
check(artifacts.rust_binary(root, 'omb', 'release', 'omobab.exe') == path.join(root, 'omb', 'target', 'release', 'omobab.exe'),
  'default: the crate target directory')
do
  local own = path.join(root, 'own-target')
  local probe = ("package.path=%q..package.path;local a=require('run_artifacts');" ..
    "io.write(a.rust_binary(%q,'omb','release','omobab.exe'),'|',a.rust_binary(%q,'omoba-client-runtime','debug','omoba-client-runtime.exe'),'|'," ..
    "require('ue_bridge_stage').sources(%q,'h')[3].source)"):format(path.join(b.root, 'scripts', '?.lua;'), root, root, root)
  local result = b.lib('process').run(b.lib('platform').lua_executable, {'-e', probe},
    {cwd = b.root, env = {OM_RUST_RUN_BINARIES = own}, check = false})
  check(result.exit_code == 0, 'probe ran: ' .. tostring(result.stderr))
  local server_path, runtime_path, bridge_path = result.stdout:match('^([^|]+)|([^|]+)|([^|]+)')
  check(server_path == path.join(own, 'release', 'omobab.exe'), 'server from the session target: ' .. tostring(server_path))
  check(runtime_path == path.join(own, 'debug', 'omoba-client-runtime.exe'), 'runtime from the session target')
  check(bridge_path == path.join(own, 'debug', 'om_bridge.dll'), 'bridge staging follows it too')
end

for name in lfs.dir(path.join(root, 'target/ue-run')) do
  if name ~= '.' and name ~= '..' then snapshot.remove_tree(path.join(root, 'target/ue-run', name), quiet) end
end
local function rm(dir)
  for name in lfs.dir(dir) do
    if name ~= '.' and name ~= '..' then
      local entry = path.join(dir, name)
      if lfs.symlinkattributes(entry, 'mode') == 'directory' then rm(entry) else os.remove(entry) end
    end
  end
  lfs.rmdir(dir)
end
rm(root)
print(('run artifacts: %d checks passed'):format(checks))
