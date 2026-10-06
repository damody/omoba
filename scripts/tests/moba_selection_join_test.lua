local source=debug.getinfo(1,'S').source:sub(2)
local tests=assert(source:match('^(.*)[/\\]'))
package.path=assert(tests:match('^(.*)[/\\]tests$'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json,platform=b.lib('path'),b.lib('json'),b.lib('platform')
local join=require('moba_selection_join')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target/selection-join-tests'))
local root
for i=1,1000 do local value=path.join(parent,os.time()..'-'..i);if lfs.mkdir(value) then root=value;break end end
assert(root)
local total=0
local function test(name,fn) fn();total=total+1;print('PASS '..name) end
local function rejects(fn,reason)
  local ok,err=pcall(fn);assert(not ok,'expected rejection')
  if reason then assert(tostring(err):find(reason,1,true),tostring(err)) end
  return tostring(err)
end
local secret=string.rep('a',64)
local invite=path.join(root,'player-6.json')
local hash='0123456789abcdef'
path.write(invite,json.encode({schema_version=1,scope='selection-room-link',admitted_player_id=6,
  address='192.0.2.10:12345',catalog_data_hash=hash,token=secret}))
local function options(name)
  return join.options({'--invite',invite,'--player','6','--output',path.join(root,name),
    '--profile','debug','--selection-timeout-seconds','2'})
end
test('strict remote selection flags and no gameplay options',function()
  assert(options('flags').player==6)
  for _,args in ipairs({{}, {'--invite',invite,'--player','06','--output','new'},
    {'--invite',invite,'--player','0','--output','new'},
    {'--invite',invite,'--player','6','--output','new','--server-only','true'},
    {'--invite',invite,'--player','6','--output','new','--player','7'}}) do
    rejects(function() join.options(args) end)
  end
  local args={'--interactive-selection','--selection-bind','192.0.2.10','--local-player','1'}
  assert(require('moba_role_launch').options(args).selection_bind=='192.0.2.10')
  rejects(function() require('moba_role_launch').options({'--selection-bind','192.0.2.10'}) end)
end)
test('private invitation metadata without returning token',function()
  local metadata=join.invitation(options('meta'))
  assert(metadata.token==nil and metadata.catalog_data_hash==hash)
  local wrong=options('wrong');wrong.player=1
  local err=rejects(function() join.invitation(wrong) end,'metadata mismatch')
  assert(not err:find(secret,1,true))
  local malformed=path.join(root,'malformed.json');path.write(malformed,'{"token":"'..secret..'"')
  wrong.invite=malformed
  err=rejects(function() join.invitation(wrong) end,'invalid selection invitation JSON')
  assert(not err:find(secret,1,true))
end)
local function lifecycle(mode)
  local opts=options(mode)
  if mode=='signal' or mode=='signal-at-start' then
    opts.selection_cancel_file=path.join(root,mode..'.signal')
    if mode=='signal-at-start' then path.write(opts.selection_cancel_file,'') end
  end
  local alive,now,spawns=true,0,0
  local result_file,log
  local final={schema_version=1,players={{player_id=6,team_id=1,bot=false,hero='training_ranger'}}}
  local receipt_hash=hash
  if mode=='native-validation' then
    local launch=require('moba_role_launch')
    local authored=launch.load_recipe({hero_selections={}})
    local candidate=path.join(root,'native-validation-input.json')
    path.write(candidate,json.encode(authored))
    local checked=require('moba_config_command').run(opts,{'--lock-plan',candidate},b.lib('process'))
    local locked=json.decode(checked.stdout)
    final=locked.plan;receipt_hash=locked.selection.catalog_data_hash
    for _,player in ipairs(final.players) do if player.bot==false then opts.player=player.player_id;break end end
    opts.invite=path.join(root,'native-invitation.json')
    path.write(opts.invite,json.encode({schema_version=1,scope='selection-room-link',admitted_player_id=opts.player,
      address='192.0.2.10:12345',catalog_data_hash=receipt_hash,token=secret}))
  end
  if mode=='missing-owner' then final.players[1].player_id=7 end
  if mode=='bot-owner' then final.players[1].bot=true end
  local old={}
  local validations=0
  old.run=function(exe,args,run_options)
    validations=validations+1
    assert(exe:match('moba%-config%.exe$') and args[1]=='--lock-plan' and #args==2)
    assert(json.encode(json.read(args[2]))==json.encode(final))
    assert(run_options.env.OMB_LUA_CONTENT=='0')
    if mode=='native-validation' then return b.lib('process').run(exe,args,run_options) end
    if mode=='config-failure' then return {exit_code=1,stderr='fixture config failure'} end
    local validated=json.decode(json.encode(final))
    if mode=='changed-plan' then validated.players[1].hero='training_luminary' end
    return {exit_code=0,stdout=json.encode({scope='host-prepared-selection',plan=validated,
      selection={ready=true,finalized=true,catalog_data_hash=hash}})}
  end
  old.spawn=function(exe,args)
    assert(exe==platform.lua_executable)
    spawns=spawns+1
    for _,arg in ipairs(args) do
      assert(not arg:find(secret,1,true) and not arg:find('om-presentation=',1,true))
      result_file=result_file or arg:match('^%-om%-selection%-result=(.*)$')
      log=log or arg:match('^%-abslog=(.*)$')
    end
    assert(result_file and log)
    local protocol=mode=='bad-ready' and '10' or '1'
    path.write(log,'OM_SELECTION_READY player='..opts.player..' protocol='..protocol..' shared_room=1\n')
    return 606
  end
  old.inspect=function() return alive end
  old.stop=function(pid) assert(pid==606);alive=false end
  old.wait=function() return not alive end
  local api,fixture=require('tests.selection_owned_fixture').wrap(old)
  local clock={monotonic_ms=function() return now end,sleep_ms=function(ms)
    now=now+ms
    if mode=='signal' then path.write(opts.selection_cancel_file,'');return end
    if mode=='timeout' then return end
    if mode=='reused' or mode=='reused-cancel' then fixture.reuse(606) end
    local seats={};for _,player in ipairs(final.players) do seats[#seats+1]={player=player,locked=mode~='unlocked-seat'} end
    if mode~='cancel' and mode~='reused-cancel' then path.write(result_file,json.encode({protocol_version=1,shared_room=true,
      admitted_player_id=mode=='wrong-player' and 1 or opts.player,
      plan=mode=='missing-plan' and json.null or final,
      selection={schema_version=1,ready=true,finalized=true,catalog_data_hash=receipt_hash,seats=seats}})) end
    if mode~='reused' and mode~='reused-cancel' then alive=false end
  end}
  local verified=false
  local ok,err=pcall(join.run,opts,api,clock,{
    editor=function() return platform.lua_executable,'fixture-engine' end,
    verify_frontend=function(engine,received_process)
      assert(engine=='fixture-engine' and received_process==api and spawns==0)
      assert(not path.exists(opts.output));verified=true
    end})
  assert(verified)
  assert(spawns==(mode=='signal-at-start' and 0 or 1))
  local success=mode=='success' or mode=='reused' or mode=='native-validation'
  assert(ok==success,tostring(err))
  assert(#fixture.stopped==((mode=='timeout' or mode=='signal') and 1 or 0))
  if success then
    local result=json.read(path.join(opts.output,'result.json'))
    assert(result.scope=='selection-client-only' and result.player_id==opts.player)
    assert(result.recipe_json==path.join(opts.output,'match-plan.json') and validations==1)
    assert(json.encode(json.read(result.recipe_json))==json.encode(final))
  else
    assert(not path.exists(path.join(opts.output,'result.json')))
    assert(not path.exists(path.join(opts.output,'match-plan.json')))
    assert(not path.read(path.join(opts.output,'errors.md')):find(secret,1,true))
    if mode=='signal' or mode=='signal-at-start' then
      assert(path.is_file(opts.selection_cancel_file))
      assert(path.read(path.join(opts.output,'errors.md')):find('selection cancelled',1,true))
    end
  end
end
test('frontend mismatch fails before output or renderer',function()
  local opts=options('frontend-mismatch')
  local calls=0
  rejects(function() join.run(opts,{spawn_owned=function() calls=calls+1 end},{},{
    editor=function() return platform.lua_executable,'fixture-engine' end,
    verify_frontend=function(engine) assert(engine=='fixture-engine');error('fixture stage mismatch') end,
  }) end,'fixture stage mismatch')
  assert(calls==0 and not path.exists(opts.output))
end)
for _,mode in ipairs({'success','cancel','timeout','wrong-player','bad-ready','reused','reused-cancel',
  'missing-plan','unlocked-seat','config-failure','changed-plan','missing-owner','bot-owner','native-validation',
  'signal','signal-at-start'}) do
  test('remote original-owned selection '..mode,function() lifecycle(mode) end)
end
print(('selection join: %d groups passed; injected renderer, no network/game/Unreal acceptance'):format(total))
