local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/../?.lua;' .. package.path
require('_bootstrap')
local path = require('tools.lua.lib.path')
local json = require('tools.lua.lib.json')
local inspect = require('ue_binary_preflight').inspect
local engine = 'D:/fixture-engine'
local project = 'D:/fixture-project/om.uproject'
local engine_manifest = path.join(engine, 'Engine/Binaries/Win64/UnrealEditor.modules')
local project_manifest = path.join(path.parent(project), 'Binaries/Win64/UnrealEditor.modules')
local plugins = path.join(path.parent(project), 'Plugins')
local descriptor = path.join(plugins, 'Example/Example.uplugin')
local plugin_manifest = path.join(plugins, 'Example/Binaries/Win64/UnrealEditor.modules')
local files
local function store(file, value) files[path.join(file)] = type(value) == 'table' and json.encode(value) or value end
local function reset()
  files = {}
  store(engine_manifest, {BuildId='same', Modules={}})
  store(project, {Modules={{Name='OmGame'}}, Plugins={{Name='Example',Enabled=true}}})
  store(project_manifest, {BuildId='same', Modules={OmGame='UnrealEditor-OmGame.dll'}})
  store(path.join(path.parent(project_manifest), 'UnrealEditor-OmGame.dll'), 'binary')
  store(descriptor, {Modules={{Name='Example'}}})
  store(plugin_manifest, {BuildId='same', Modules={Example='UnrealEditor-Example.dll'}})
  store(path.join(path.parent(plugin_manifest), 'UnrealEditor-Example.dll'), 'binary')
end
local api = {
  read=function(file) return assert(files[path.join(file)]) end,
  is_file=function(file) return files[path.join(file)] ~= nil end,
  plugin_dirs=function() return {'Example'} end,
}
local count = 0
local function check(expected, mutation, pattern)
  reset(); if mutation then mutation() end
  local before = json.encode(files)
  local report = inspect(engine, project, api)
  assert(report.ready == expected, json.encode(report))
  if pattern then assert(table.concat(report.errors, '\n'):find(pattern,1,true), json.encode(report)) end
  assert(json.encode(files) == before, 'preflight mutated its input')
  count = count + 1
end
check(true)
check(false,function() store(project_manifest,{BuildId='old',Modules={OmGame='UnrealEditor-OmGame.dll'}}) end,'project: BuildId mismatch')
check(false,function() store(plugin_manifest,{BuildId='old',Modules={Example='UnrealEditor-Example.dll'}}) end,'plugin Example: BuildId mismatch')
check(false,function() files[engine_manifest]=nil end,'missing file')
check(false,function() store(engine_manifest,{BuildId='',Modules={}}) end,'engine BuildId unavailable')
check(false,function() store(plugin_manifest,'not JSON') end,'invalid JSON object')
check(false,function() store(plugin_manifest,{BuildId='same',Modules={}}) end,'declared module absent')
check(false,function() files[path.join(path.parent(plugin_manifest),'UnrealEditor-Example.dll')]=nil end,'missing module binary')
check(false,function() store(plugin_manifest,{BuildId='same',Modules={Example='../outside.dll'}}) end,'unsafe module filename')
check(true,function() store(project,{Modules={{Name='OmGame'}},Plugins={{Name='Example',Enabled=false}}}); store(descriptor,'not JSON') end)
check(false,function() store(project,{Modules={{Name='OmGame'}}}); store(descriptor,{EnabledByDefault=true,Modules={{Name='Example'}}}); files[plugin_manifest]=nil end,'missing file')
check(true,function() store(project,{Modules={{Name='OmGame'}}}); store(descriptor,{EnabledByDefault=false,Modules={{Name='Example'}}}); files[plugin_manifest]=nil end)
check(false,function() store(project_manifest,{BuildId='same',Modules=json.null}) end,'missing Modules object')
check(false,function() store(project,'null') end,'invalid JSON object')
check(true,function() store(descriptor,'\239\187\191' .. files[descriptor]) end)
print('Unreal binary preflight: ' .. count .. ' read-only scenarios passed')
