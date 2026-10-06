local source=debug.getinfo(1,'S').source:sub(2)
local tests=assert(source:match('^(.*)[/\\]'))
package.path=assert(tests:match('^(.*)[/\\]tests$'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json,process=b.lib('path'),b.lib('json'),b.lib('process')
local host=require('moba_selection_host')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target/headless-selection-tests'))
local root
for i=1,1000 do local value=path.join(parent,os.time()..'-'..i);if lfs.mkdir(value) then root=value;break end end
assert(root)
local total=0
local function test(name,fn) fn();total=total+1;print('PASS '..name) end
local function rejects(fn) local ok,err=pcall(fn);assert(not ok,'expected rejection');return tostring(err) end
test('headless flags are explicit and cannot start gameplay or renderer',function()
  local value=host.options({'--selection-bind','192.0.2.10','--output','new'})
  assert(value.profile=='release' and value.interactive_selection)
  for _,args in ipairs({{}, {'--output','new'}, {'--selection-bind','0.0.0.0','--output','new'},
    {'--selection-bind','192.0.2.10','--output','new','--local-player','1'},
    {'--selection-bind','192.0.2.10','--output','new','--server-only'},
    {'--selection-bind','192.0.2.10','--output','new','--selection-bind','127.0.0.1'}}) do
    rejects(function() host.options(args) end)
  end
end)
local recipe=require('moba_role_launch').load_recipe(require('moba_role_launch').options({}))
recipe.players[2].bot=false
local recipe_path=path.join(root,'source.json')
path.write(recipe_path,json.encode(recipe))
local original=path.read(recipe_path)
local function execute(mode)
  local output=path.join(root,mode)
  local options=host.options({'--recipe',recipe_path,'--selection-bind','192.0.2.10',
    '--output',output,'--selection-timeout-seconds','1'})
  if mode=='cancel' or mode=='cancel-final' or mode=='cancel-at-start' then
    options.selection_cancel_file=path.join(root,mode..'.signal')
    if mode=='cancel-at-start' then path.write(options.selection_cancel_file,'private signal not to read') end
  end
  local now,alive,spawns,plan,hash,host_dir=0,false,0,nil,nil,nil
  local old={}
  old.run=function(exe,args,run_options)
    -- Real compiled validation, no gameplay or Lua VM in the executable.
    local result=process.run(exe,args,run_options)
    local record=json.decode(result.stdout)
    plan=record.plan;hash=record.selection.catalog_data_hash
    assert(record.scope=='host-prepared-selection' and #plan.players==10)
    return result
  end
  old.spawn=function(exe,args)
    spawns=spawns+1
    assert(exe==require('moba_config_command').resolve(options))
    assert(args[1]=='--selection-host' and args[3]=='192.0.2.10:0' and #args==4)
    if mode=='spawn-failure' then error('injected host spawn failure') end
    host_dir=args[4];path.mkdir_p(host_dir)
    path.write(path.join(host_dir,'ready.json'),json.encode({schema_version=1,
      scope='shared-selection-host-ready',tick_rate_hz=60,
      address=mode=='bad-ready' and '127.0.0.1:12345' or '192.0.2.10:12345',catalog_data_hash=hash}))
    alive=true;return 401
  end
  old.inspect=function() return alive end
  old.stop=function(pid) assert(pid==401);alive=false end
  old.wait=function(pid) assert(pid==401);return not alive end
  local api,fixture=require('tests/selection_owned_fixture').wrap(old)
  local clock={monotonic_ms=function() return now end,sleep_ms=function(ms)
    now=now+ms
    if mode=='cancel' then path.write(options.selection_cancel_file,'');return end
    if mode=='timeout' then return end
    if mode=='host-exit' then alive=false;return end
    local final=require('moba_role_launch').select_heroes(plan,{[1]='training_ranger',[2]='training_luminary'})
    if mode=='tamper' then final.players[3].team_id=99 end
    local seats={};for _,player in ipairs(final.players) do seats[#seats+1]={player=player,locked=true} end
    path.write(path.join(host_dir,'finalized-plan.json'),json.encode({schema_version=1,
      scope='shared-selection-host-finalized',tick_rate_hz=60,plan=final,
      selection={schema_version=1,catalog_data_hash=hash,revision=5,ready=true,finalized=true,seats=seats}}))
    if mode~='retire-failure' then alive=false end
    if mode=='cancel-final' then path.write(options.selection_cancel_file,'') end
  end}
  local ok,result=pcall(host.run,options,api,clock)
  assert(path.read(recipe_path)==original and spawns==(mode=='cancel-at-start' and 0 or 1),
    'source changed or extra child spawned')
  if mode=='success' then
    assert(ok,tostring(result))
    assert(result==path.join(output,'match-plan.json'))
    local final=json.read(result)
    assert(final.players[1].hero=='training_ranger' and final.players[2].bot==false and #final.players==10)
    local report=json.read(path.join(output,'result.json'))
    assert(report.scope=='selection-host-only' and report.human_count==2 and report.recipe_json==result)
    assert(#fixture.stopped==0 and #fixture.waited==1)
    assert(not path.exists(path.join(output,'player-1')) and not path.exists(path.join(output,'player-2')))
  else
    assert(not ok,'failure bypassed')
    assert(not path.exists(path.join(output,'match-plan.json')) and not path.exists(path.join(output,'result.json')))
    assert(path.is_file(path.join(output,'errors.md')))
    if mode=='timeout' or mode=='bad-ready' or mode=='retire-failure' or mode=='cancel' then assert(#fixture.stopped==1) end
    if mode=='cancel' or mode=='cancel-final' or mode=='cancel-at-start' then
      assert(path.is_file(options.selection_cancel_file),'cancel signal removed')
      assert(path.read(path.join(output,'errors.md')):find('selection cancelled',1,true))
    end
  end
  assert(not alive,'original host remained')
end
for _,mode in ipairs({'success','timeout','host-exit','bad-ready','tamper','retire-failure','spawn-failure',
  'cancel','cancel-final','cancel-at-start'}) do
  test('headless host '..mode,function() execute(mode) end)
end
test('existing output is preserved before native validation or spawn',function()
  local options=host.options({'--selection-bind','127.0.0.1','--output',root})
  rejects(function() host.run(options,{run=function() error('must not run') end}, {}) end)
  assert(path.read(recipe_path)==original and not path.exists(path.join(root,'errors.md')))
end)
print(('headless selection: %d groups passed; real native config, injected owned host; no game/Unreal/LAN'):format(total))
