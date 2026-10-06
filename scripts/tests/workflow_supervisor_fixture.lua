-- Harmless fixed-Lua descendants; no UE, game, network or external mutation.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]tests[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local process,path,json,time,platform=b.lib('process'),b.lib('path'),b.lib('json'),b.lib('time'),b.lib('platform')
local mode,output=assert(arg[1]),assert(arg[2])
local function spawn(role)
  local _,owned=process.spawn_owned(platform.lua_executable,{source,role,output},{cwd=b.root,
    stdout=path.join(output,role..'.stdout.log'),stderr=path.join(output,role..'.stderr.log')})
  json.write(path.join(output,role..'.json'),owned)
end
if mode=='leaf' then
  path.write(path.join(output,'leaf-ready'),'ready')
elseif mode=='child' then
  spawn('leaf')
else
  process.supervise_workflow(source,arg)
  local value=b.lib('host').call('workflow_member',{job_name=assert(os.getenv('_OMOBA_PRIVATE_WORKFLOW_JOB'))})
  assert(value.supervised)
  -- Empty, quotes and trailing slashes survive without a shell.
  assert(arg[3]=='' and arg[4]=='a "quoted" b' and arg[5]=='trailing\\')
  spawn('child')
  assert(time.poll(10000,20,function() return path.exists(path.join(output,'leaf-ready')) end),'leaf did not start')
  path.write(path.join(output,'tree-ready'),'ready')
  if mode=='normal' then os.exit(7,true) end
end
while true do time.sleep_ms(50) end
