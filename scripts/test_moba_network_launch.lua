local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json,process,platform=b.lib('path'),b.lib('json'),b.lib('process'),b.lib('platform')
local launch,network=require('moba_role_launch'),require('moba_network_launch')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','network-launch-tests'))
local root
for i=1,1000 do local p=path.join(parent,os.time()..'-'..i);if lfs.mkdir(p) then root=p;break end end
assert(root)
local count=0
local function test(name,fn) fn();count=count+1;print('PASS '..name) end
local function rejects(fn,reason)
  local ok,err=pcall(fn);assert(not ok,'expected rejection')
  if reason then assert(tostring(err):find(reason,1,true),tostring(err)) end
end
local function contains(args,value) for _,arg in ipairs(args) do if arg==value then return true end end end
test('strict endpoint/remote ownership options',function()
  assert(launch.options({}).server_bind=='127.0.0.1')
  for _,address in ipairs({'0.0.0.0','224.0.0.1','255.255.255.255','256.1.1.1','192.168.01.1',
    'host.local','127.0.0.1:57061','127.0.0.1 & command','::1'}) do
    rejects(function() launch.options({'--server-bind',address}) end)
  end
  for _,args in ipairs({{'--connect','192.0.2.10'},
    {'--connect','192.0.2.10','--recipe','host.lua','--local-player','1'},
    {'--connect','192.0.2.10','--recipe','host.json','--local-player','1','--hero','1=training_ranger'},
    {'--connect','192.0.2.10','--recipe','host.json','--local-player','1','--interactive-selection'},
    {'--connect','192.0.2.10','--recipe','host.json','--local-player','1','--server-bind','192.0.2.20'},
    {'--local-player','1','--local-player','01'},{'--local-player','0'},{'--local-player','4294967296'}}) do
    rejects(function() launch.options(args) end)
  end
end)
test('local seats are explicit admitted humans, sorted without roster mutation',function()
  local humans={{player_id=6,team_id=2},{player_id=1,team_id=1}}
  local selected=network.local_humans(humans,{[6]=true})
  assert(#selected==1 and selected[1].team_id==2 and #humans==2)
  assert(network.local_humans(humans,{})[1].player_id==1 and humans[1].player_id==6)
  rejects(function() network.local_humans(humans,{[3]=true}) end,'not an admitted human')
end)
local host,remote
test('native Rust preflight creates one host and a server-free remote plan',function()
  local recipe=launch.load_recipe(launch.options({}))
  recipe.players[6].bot=false
  local file=path.join(root,'recipe.json');path.write(file,json.encode(recipe))
  local original=path.read(file)
  host=launch.prepare(launch.options({'--prepare-only','--recipe',file,
    '--server-bind','192.0.2.10','--local-player','1','--output',path.join(root,'host')}),process)
  remote=launch.prepare(launch.options({'--prepare-only','--recipe',host.recipe_json,
    '--connect','192.0.2.10','--local-player','6','--output',path.join(root,'remote')}),process)
  assert(host.mode=='host' and host.server and #host.clients==1 and host.clients[1].player_id==1)
  assert(host.human_count==2 and host.bot_count==8 and host.local_human_count==1)
  assert(path.read(host.config):match('SERVER_IP%s*=%s*"([^"]+)"')=='192.0.2.10')
  assert(remote.mode=='remote-client' and remote.server==nil and #remote.clients==1)
  assert(remote.clients[1].player_id==6 and remote.clients[1].team_id==2)
  assert(remote.server_address=='192.0.2.10:57061' and remote.local_human_count==1)
  assert(remote.clients[1].presentation=='127.0.0.1:57062')
  assert(contains(remote.clients[1].runtime.args,'192.0.2.10:57061'))
  assert(contains(remote.clients[1].unreal.args,'-om-presentation-only'))
  assert(json.read(path.join(remote.output,'launch-plan.json')).server==nil)
  assert(path.read(file)==original)
  assert(json.encode(json.read(remote.recipe_json))==json.encode(json.read(host.recipe_json)))
end)
local function lifecycle(mode)
  local plan=json.decode(json.encode(remote))
  plan.output=path.mkdir_p(path.join(root,'lifecycle-'..mode))
  plan.clients[1].runtime.exe=platform.lua_executable
  local spawned,states,stopped,waited={},{},{},{}
  local fake={}
  fake.spawn=function(exe,args,options)
    local pid=100+#spawned+1;spawned[#spawned+1]=pid
    assert(exe==platform.lua_executable and not contains(args,'--selection-host'))
    states[pid]=true
    path.write(options.stdout,'client-runtime ready player_id=6 team_id=2 replica_tick=1')
    if pid==102 and mode~='runtime-exit' then states[pid]=false end
    return pid
  end
  fake.inspect=function(pid) assert(pid~=nil and states[pid]~=nil,'inspected unowned server');return states[pid] end
  fake.poll_ready=function(_,_,predicate)
    assert(predicate())
    if mode=='admission-failure' then error('injected admission failure') end
  end
  fake.stop=function(pid,exe)
    assert(states[pid]~=nil and exe==platform.lua_executable)
    stopped[#stopped+1]=pid
    if mode=='cleanup-failure' and pid==102 then error('injected renderer cleanup failure') end
    states[pid]=false
  end
  fake.wait=function(pid) waited[#waited+1]=pid;return not states[pid] end
  if mode=='runtime-exit' then
    local inspect=fake.inspect
    fake.inspect=function(pid) if pid==101 and #spawned==2 then return false end;return inspect(pid) end
  end
  local ok,err=pcall(launch.launch,plan,platform.lua_executable,fake,
    {sleep_ms=function() error('unexpected sleep') end})
  assert(#spawned==(mode=='admission-failure' and 1 or 2),'remote launcher spawned a server')
  assert(ok==(mode=='success'),tostring(err))
  if mode=='admission-failure' then
    assert(tostring(err):find('injected admission failure',1,true) and table.concat(stopped,',')=='101')
  else
    assert(table.concat(stopped,',')=='102,101')
    assert(table.concat(waited,',')==(mode=='cleanup-failure' and '101' or '102,101'))
  end
  if mode=='runtime-exit' then assert(tostring(err):find('runtime-p6 exited during match',1,true)) end
  if mode=='cleanup-failure' then
    assert(tostring(err):find('injected renderer cleanup failure',1,true))
    assert(path.read(path.join(plan.output,'errors.md')):find('injected renderer cleanup failure',1,true))
  end
  local result=json.read(path.join(plan.output,'lifecycle.json'))
  assert(result.cleanup_succeeded==(mode~='cleanup-failure'))
end
for _,mode in ipairs({'success','admission-failure','runtime-exit','cleanup-failure'}) do
  test('remote owned lifecycle '..mode,function() lifecycle(mode) end)
end
print(('network launch functionality: %d/%d passed; native config, mocked processes, no LAN/Unreal acceptance'):format(count,count))
