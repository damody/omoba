local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local launch=require('moba_role_launch')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','selection-handoff-tests'))
local root
for i=1,1000 do local p=path.join(parent,os.time()..'-'..i);if lfs.mkdir(p) then root=p;break end end
assert(root)
local baseline=launch.load_recipe(launch.options({}))
local function reply()
  return {protocol_version=1,admitted_player_id=1,error=json.null,
    selection={catalog_data_hash='testhash',ready=true,finalized=true},
    plan=launch.select_heroes(baseline,{[1]='tamako'})}
end
local function reply_with(plan)
  local value=reply()
  value.plan=plan
  value.selection.summoner_loadouts=plan.summoner_loadouts
  return value
end
local total=0
local function test(name,fn) fn();total=total+1;print('PASS '..name) end
local function rejects(fn) assert(not pcall(fn),'expected rejection') end

test('finalized handoff changes only the bound human hero',function()
  local value=reply()
  local selected=launch.selection_result(value,baseline,1,'testhash')
  assert(selected.players[1].hero=='tamako')
  assert(baseline.players[1].hero~='tamako')
  for _,mutate in ipairs({
    function(r) r.protocol_version=2 end,
    function(r) r.admitted_player_id=2 end,
    function(r) r.error='rejected' end,
    function(r) r.selection.ready=false end,
    function(r) r.selection.finalized=false end,
    function(r) r.selection.catalog_data_hash='wrong' end,
    function(r) r.plan.players[2].hero='tamako' end,
    function(r) r.plan.players[1].team_id=2 end,
    function(r) r.plan.think_hz=60 end,
    function(r) r.plan.summoner_loadouts={['1']={'training_point_blink_v1',json.null}} end,
    function(r) r.selection.summoner_loadouts={['1']={'training_self_heal_v1',json.null}} end,
    function(r) r.plan.summoner_loadouts=json.null end,
    function(r) r.selection.summoner_loadouts=json.null end,
  }) do local r=reply();mutate(r);rejects(function() launch.selection_result(r,baseline,1,'testhash') end) end
  local equipped=launch.select_heroes(baseline,{[1]='tamako'})
  equipped.summoner_loadouts={['1']={'training_point_blink_v1',json.null}}
  local source=launch.select_heroes(baseline,{})
  source.summoner_loadouts={['1']={'training_point_blink_v1',json.null}}
  local selected=launch.selection_result(reply_with(equipped),source,1,'testhash')
  assert(selected.summoner_loadouts['1'][1]=='training_point_blink_v1')
  equipped.summoner_loadouts={['1']={'training_self_shield_v1',json.null}}
  source.summoner_loadouts={['1']={'training_self_shield_v1',json.null}}
  assert(launch.selection_result(reply_with(equipped),source,1,'testhash').summoner_loadouts['1'][1]=='training_self_shield_v1')
  local omitted=launch.select_heroes(baseline,{})
  omitted.summoner_loadouts={}
  assert(launch.selection_result(reply_with(launch.select_heroes(baseline,{[1]='tamako'})),omitted,1,'testhash'))
end)

-- E210: picking a bot's default hero. The service moves that bot (E164); the launch accepts the
-- final roster by its rules, not by which seats changed.
test('a human may take a bot teammate default hero; the moved bot is legal',function()
  local function bot_hero(id) for _,p in ipairs(baseline.players) do if p.player_id==id then return p.hero end end end
  local taken=bot_hero(3)
  assert(taken and taken~=baseline.players[1].hero,'fixture: seat 3 is a bot with its own hero')
  local function moved_reply(replacement)
    local plan=json.decode(json.encode(baseline))
    for _,p in ipairs(plan.players) do
      if p.player_id==1 then p.hero=taken elseif p.player_id==3 then p.hero=replacement end
    end
    local value=reply_with(plan)
    value.selection.seats={}
    for _,p in ipairs(plan.players) do value.selection.seats[#value.selection.seats+1]={locked=true,player=json.decode(json.encode(p))} end
    value.hero_catalog={}
    for _,p in ipairs(baseline.players) do value.hero_catalog[#value.hero_catalog+1]={hero=p.hero} end
    return value
  end
  -- the replacement is a hero team 1 does not use (the human's old one is free now)
  local free=baseline.players[1].hero
  local accepted=launch.selection_result(moved_reply(free),baseline,1,'testhash')
  for _,p in ipairs(accepted.players) do
    if p.player_id==1 then assert(p.hero==taken) end
    if p.player_id==3 then assert(p.hero==free) end
  end
  rejects(function() launch.selection_result(moved_reply(bot_hero(2)),baseline,1,'testhash') end) -- duplicate on team 1
  rejects(function() launch.selection_result(moved_reply('not_offered'),baseline,1,'testhash') end) -- not in the catalog
  local unlocked=moved_reply(free)
  unlocked.selection.seats[1].locked=false
  rejects(function() launch.selection_result(unlocked,baseline,1,'testhash') end)
  -- a bot whose hero nobody took may not change
  local stray=moved_reply(free)
  for _,p in ipairs(stray.plan.players) do if p.player_id==4 then p.hero=bot_hero(9) end end
  for _,s in ipairs(stray.selection.seats) do if s.player.player_id==4 then s.player.hero=bot_hero(9) end end
  rejects(function() launch.selection_result(stray,baseline,1,'testhash') end)
  -- a seat may change only its hero
  local relaned=moved_reply(free)
  relaned.plan.players[3].lane='top'
  rejects(function() launch.selection_result(relaned,baseline,1,'testhash') end)
end)

local function execute(name,mode)
  local options=launch.options({'--interactive-selection','--profile','debug','--hero','1=asai_nagamasa',
    '--output',path.join(root,name)})
  if mode=='completion-timeout' then options.selection_timeout_seconds=1 end
  if mode=='old' then options.editor_ready_timeout_seconds=1 end
  if mode=='cold-ready' then options.editor_ready_timeout_seconds=90;options.selection_timeout_seconds=120 end
  local elapsed=0
  local cold_log
  local slept,spawned,stopped=false,false,false
  local fake={run=function(_,args)
    assert(args[1]=='--lock-plan')
    local candidate=json.read(args[2])
    assert(candidate.players[1].hero=='asai_nagamasa')
    return {exit_code=0,stdout=json.encode({scope='host-prepared-selection',plan=candidate,
      selection={catalog_data_hash='testhash'}})}
  end,spawn=function(_,args)
    spawned=true
    local result
    assert(args[4]=='-om-hero-selection')
    for _,a in ipairs(args) do
      assert(not a:find('om-presentation=',1,true),'gameplay runtime started during selection')
      result=result or a:match('^%-om%-selection%-result=(.*)$')
      local log=a:match('^%-abslog=(.*)$')
      if mode=='cold-ready' then cold_log=log or cold_log end
      if log and mode~='old' then
        local marker='OM_SELECTION_READY player=1 protocol='..(mode=='protocol-prefix' and '10' or '1')
          ..' shared_room='..(mode=='wrong-room' and '1' or '0')
        local prefix=mode=='retired-backlog' and string.rep('startup\n',40000) or ''
        path.write(log,mode=='cold-ready' and '' or prefix..marker..(mode=='retired-tail' and '' or '\n'))
      end
    end
    assert(result)
    if mode~='cancel' and mode~='reused-cancel' and mode~='old' and mode~='completion-timeout' then
      local r=reply()
      if mode=='tamper' then r.plan.players[2].bot=false end
      path.write(result,json.encode(r))
    end
    return 123
  end,inspect=function() if mode=='retired-backlog' or mode=='retired-tail' then return false end
    return mode=='reused-success' or mode=='reused-cancel'
      or ((mode=='old' or mode=='completion-timeout') and not stopped) or not slept end,
    wait=function() assert(slept or mode=='retired-backlog' or mode=='retired-tail');return true end,
    stop=function() assert(mode=='old' or mode=='completion-timeout','must not stop an unrelated or exited process');stopped=true end}
  local fixture
  fake,fixture=require('tests.selection_owned_fixture').wrap(fake)
  local ok,err=pcall(launch.interactive_select,options,'mock-editor.exe',fake,
    {sleep_ms=function(ms) assert(ms==250);slept=true;elapsed=elapsed+ms
      if mode=='cold-ready' then elapsed=60000;path.write(cold_log,'OM_SELECTION_READY player=1 protocol=1 shared_room=0\n',true) end
      if mode=='reused-success' or mode=='reused-cancel' then fixture.reuse(123) end end,
      monotonic_ms=function() return slept and mode=='old' and 50000 or elapsed end})
  assert(spawned and (slept or mode=='retired-backlog' or mode=='retired-tail'))
  if mode=='success' or mode=='reused-success' or mode=='retired-backlog' or mode=='retired-tail' or mode=='cold-ready' then
    assert(ok and next(options.hero_selections)==nil,tostring(err))
    assert(path.is_file(options.recipe))
    assert(json.read(options.recipe).players[1].hero=='tamako')
  else
    assert(not ok)
    assert(path.is_file(options.output..'-selection/errors.md'))
    assert(not path.exists(options.output),'cancelled selection created match output')
    if mode=='completion-timeout' then
      assert(stopped and elapsed==1000)
      assert(path.read(options.output..'-selection/errors.md'):find('selection completion timed out',1,true))
    end
  end
  if mode=='reused-success' or mode=='reused-cancel' then
    assert(not stopped and #fixture.stopped==0,'reused PID was stopped')
    local original=json.read(options.output..'-selection/owned-process.json')
    assert(original.creation_token~=fixture.current[123].creation_token,'ownership evidence changed to new lifetime')
  end
end
test('selection process waits for exit before handoff and clears CLI overrides',function() execute('success','success') end)
test('window cancellation never starts or auto-locks a match',function() execute('cancel','cancel') end)
test('tampered roster fails closed and records an error',function() execute('tamper','tamper') end)
test('old renderer cannot hang forever or bypass selection handshake',function() execute('old','old') end)
test('ready renderer without finalization is bounded and never launches gameplay',function()
  execute('completion-timeout','completion-timeout')
end)
test('prepare-only and interactive mode cannot be combined',function()
  rejects(function() launch.options({'--prepare-only','--interactive-selection'}) end)
end)
test('original selection exits and reused PID does not block valid handoff',function()execute('reused-success','reused-success')end)
test('reused PID cannot satisfy cancellation or receive cleanup stop',function()execute('reused-cancel','reused-cancel')end)
test('protocol prefix cannot satisfy single selection handshake',function()execute('protocol-prefix','protocol-prefix')end)
test('shared-room marker cannot satisfy single selection handshake',function()execute('wrong-room','wrong-room')end)
test('retired producer backlog drains in bounded chunks',function()execute('retired-backlog','retired-backlog')end)
test('unterminated marker is accepted only after producer retirement',function()execute('retired-tail','retired-tail')end)
test('cold single renderer handshake after60s within90s startup budget',function()execute('cold-ready','cold-ready')end)
print(('selection handoff functionality: %d/%d passed; mocked renderer, no match started'):format(total,total))
