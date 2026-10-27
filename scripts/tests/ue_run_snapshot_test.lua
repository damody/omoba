local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local path, lfs = b.lib('path'), require('lfs')
local snapshot = require('ue_run_snapshot')

-- A fake repo: omfue with project binaries, one plugin, and a sibling scripts directory.
local root = path.join(b.root, 'target', 'ue-run-snapshot-test-' .. os.time() .. '-' .. math.random(0, 99999))
local function put(relative, text)
  local file = path.join(root, relative)
  path.mkdir_p(path.parent(file))
  path.write(file, text, true)
end
put('omfue/om.uproject', '{}')
put('omfue/Content/Map.umap', 'content')
put('omfue/Saved/Logs/old.log', 'log')
put('omfue/Intermediate/junk.obj', 'junk')
put('omfue/Intermediate/TargetInfo.json', '{"Targets":[]}')
put('omfue/Binaries/Win64/UnrealEditor-OmGame.dll', 'game-v1')
put('omfue/Binaries/Win64/UnrealEditor-OmGame.pdb', 'symbols')
put('omfue/Binaries/Win64/D3D12/D3D12Core.dll', 'd3d')
put('omfue/Plugins/OmRuntime/OmRuntime.uplugin', '{}')
put('omfue/Plugins/OmRuntime/Source/A.cpp', 'source')
put('omfue/Plugins/OmRuntime/Binaries/Win64/UnrealEditor-OmRuntime.dll', 'runtime')
put('scripts/lua_data/story.lua', 'story')
local logs = {}
local deps = {log = function(line) logs[#logs + 1] = line end}

local first = snapshot.prepare(root, deps)
local p = first.project_dir
assert(path.read(path.join(p, 'om.uproject')) == '{}', 'uproject copied')
assert(path.read(path.join(p, 'Binaries/Win64/UnrealEditor-OmGame.dll')) == 'game-v1', 'project DLL copied')
assert(path.read(path.join(p, 'Binaries/Win64/D3D12/D3D12Core.dll')) == 'd3d', 'binary subdirectories copied')
assert(not path.exists(path.join(p, 'Binaries/Win64/UnrealEditor-OmGame.pdb')), 'PDBs are not copied')
assert(lfs.symlinkattributes(path.join(p, 'Intermediate'), 'mode') == 'directory', 'Intermediate is a real directory, not linked')
assert(not path.exists(path.join(p, 'Intermediate/junk.obj')), 'live Intermediate is not copied')
-- TargetInfo.json is seeded so the editor never runs UBT QueryTargets (and waits on its mutex).
assert(path.read(path.join(p, 'Intermediate/TargetInfo.json')) == '{"Targets":[]}', 'TargetInfo seeded')
os.remove(path.join(p, 'Intermediate/TargetInfo.json'))
snapshot.prepare(root, deps)
assert(path.is_file(path.join(p, 'Intermediate/TargetInfo.json')), 'a reused snapshot is seeded too')
assert(path.read(path.join(p, 'Plugins/OmRuntime/Binaries/Win64/UnrealEditor-OmRuntime.dll')) == 'runtime', 'plugin DLL copied')
assert(lfs.symlinkattributes(path.join(p, 'Content'), 'mode') == 'link', 'Content is a junction')
assert(path.read(path.join(p, 'Content/Map.umap')) == 'content', 'Content resolves to live')
assert(path.read(path.join(p, 'Plugins/OmRuntime/Source/A.cpp')) == 'source', 'plugin Source resolves to live')
-- ProjectDir()/.. reaches the repo's other entries (lua_data, omb, ...).
assert(path.read(path.join(first.root, 'scripts/lua_data/story.lua')) == 'story', 'repo siblings linked')
assert(not path.exists(path.join(first.root, 'target')), 'target is never linked (cycle)')
-- Writing through Saved lands in the live tree.
path.write(path.join(p, 'Saved/Logs/new.log'), 'evidence', true)
assert(path.read(path.join(root, 'omfue/Saved/Logs/new.log')) == 'evidence', 'Saved is shared with live')
assert(snapshot.prepare(root, deps).key == first.key, 'unchanged binaries reuse the snapshot')

-- A relink changes the key; the old snapshot stays until it falls out of KEEP.
local keys = {first.key}
for version = 2, snapshot.KEEP + 2 do
  put('omfue/Binaries/Win64/UnrealEditor-OmGame.dll', 'game-v' .. version)
  lfs.touch(path.join(root, 'omfue/Binaries/Win64/UnrealEditor-OmGame.dll'), os.time() + version, os.time() + version)
  local next_snapshot = snapshot.prepare(root, deps)
  assert(next_snapshot.key ~= keys[#keys], 'relinked binaries yield a new snapshot')
  assert(path.read(path.join(next_snapshot.project_dir, 'Binaries/Win64/UnrealEditor-OmGame.dll')) == 'game-v' .. version)
  keys[#keys + 1] = next_snapshot.key
end
local kept = 0
for name in lfs.dir(path.join(root, 'target/ue-run')) do
  if name:match('^%x+$') then kept = kept + 1 end
end
assert(kept == snapshot.KEEP, ('prune keeps %d snapshots, found %d'):format(snapshot.KEEP, kept))
assert(not path.exists(path.join(root, 'target/ue-run', first.key)), 'oldest snapshot pruned')
-- Pruning removed junctions only: the live tree is intact.
assert(path.read(path.join(root, 'omfue/Content/Map.umap')) == 'content', 'prune never deletes live Content')
assert(path.read(path.join(root, 'scripts/lua_data/story.lua')) == 'story', 'prune never deletes live scripts')
assert(path.read(path.join(root, 'omfue/Plugins/OmRuntime/Source/A.cpp')) == 'source', 'prune never deletes live plugin source')

-- A session's shadow build supplies the binaries; Content and Saved stay live.
put('shadow/omfue/Binaries/Win64/UnrealEditor-OmGame.dll', 'game-shadow')
put('shadow/omfue/Plugins/OmRuntime/Binaries/Win64/UnrealEditor-OmRuntime.dll', 'runtime-shadow')
local shadow = snapshot.prepare(root, deps, {binaries = path.join(root, 'shadow/omfue')})
assert(path.read(path.join(shadow.project_dir, 'Binaries/Win64/UnrealEditor-OmGame.dll')) == 'game-shadow', 'shadow project DLL copied')
assert(path.read(path.join(shadow.project_dir, 'Plugins/OmRuntime/Binaries/Win64/UnrealEditor-OmRuntime.dll')) == 'runtime-shadow', 'shadow plugin DLL copied')
assert(path.read(path.join(shadow.project_dir, 'Content/Map.umap')) == 'content', 'shadow snapshot still uses live Content')

-- E142: a snapshot pruned under a running renderer comes back as a bare omfue/Saved tree without
-- ready.txt; the next prepare clears it instead of failing every rename.
snapshot.remove_tree(shadow.root, deps)
put('target/ue-run/' .. shadow.key .. '/omfue/Saved/Config/leftover.ini', 'left by a renderer')
local recovered = snapshot.prepare(root, deps, {binaries = path.join(root, 'shadow/omfue')})
assert(recovered.key == shadow.key and path.is_file(path.join(recovered.root, 'ready.txt')), 'an incomplete snapshot is rebuilt')
assert(path.read(path.join(recovered.project_dir, 'Binaries/Win64/UnrealEditor-OmGame.dll')) == 'game-shadow', 'rebuilt snapshot has its binaries')
assert(path.read(path.join(root, 'omfue/Content/Map.umap')) == 'content', 'clearing never touches live Content')

-- Clean up: snapshots through remove_tree, then the fake live tree.
for name in lfs.dir(path.join(root, 'target/ue-run')) do
  if name ~= '.' and name ~= '..' then snapshot.remove_tree(path.join(root, 'target/ue-run', name), deps) end
end
local function rm(d)
  for name in lfs.dir(d) do
    if name ~= '.' and name ~= '..' then
      local e = path.join(d, name)
      if lfs.symlinkattributes(e, 'mode') == 'directory' then rm(e) else assert(os.remove(e)) end
    end
  end
  assert(lfs.rmdir(d))
end
rm(root)

-- E137: a file Unreal reports from inside a snapshot maps back to the live file behind the
-- junction; copied binaries and paths outside a snapshot stay as they are.
local repo = 'D:/repo'
local function live(file) return snapshot.live_path(repo, file):gsub('\\', '/') end
assert(live('D:/repo/target/ue-run/0d7646eb996e1827/omfue/Saved/Automation/OriginalAudio/a/routed-mix.wav')
  == 'D:/repo/omfue/Saved/Automation/OriginalAudio/a/routed-mix.wav', 'Saved maps to live')
assert(live('d:\\REPO\\target\\ue-run\\abc\\scripts\\lua_data\\x.lua') == 'D:/repo/scripts/lua_data/x.lua', 'case and separators')
assert(live('D:/repo/target/ue-run/abc/omfue/Binaries/Win64/x.dll') == 'D:/repo/target/ue-run/abc/omfue/Binaries/Win64/x.dll',
  'copied binaries have no live counterpart')
assert(live('D:/repo/target/ue-run/abc/omfue/Plugins/OmRuntime/Binaries/Win64/om_bridge.dll')
  == 'D:/repo/target/ue-run/abc/omfue/Plugins/OmRuntime/Binaries/Win64/om_bridge.dll', 'plugin binaries are copies')
assert(live('D:/repo/omfue/Saved/x.wav') == 'D:/repo/omfue/Saved/x.wav', 'live paths are unchanged')
print('ue run snapshot: 35 checks passed')
