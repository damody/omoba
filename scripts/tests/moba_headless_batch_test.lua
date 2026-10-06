local source=debug.getinfo(1,'S').source:sub(2)
local scripts_dir=assert(source:match('^(.*)[/\\]')):gsub('[/\\]tests$','')
package.path=scripts_dir..'/?.lua;'..package.path
local b=require('_bootstrap')
local batch,compiled=require('moba_headless_batch'),require('moba_compiled_content')
local args,json=b.lib('args'),b.lib('json')
local tests=0
local function test(name,fn) fn();tests=tests+1;print('PASS '..name) end
local function rejects(fn) assert(not pcall(fn),'expected rejection') end
local function fixture(seed)
  local players={};for i=1,10 do players[i]={bot=true,team_id=i<=5 and 1 or 2} end
  return {success=true,profile_hz=60,seed=seed,bot_mode='committed_role_plan',defender_policy='guard',
    finish_tick=900,replay_verified_ticks=900,end_events=1,committed_combat_facts=5,match_played=true,
    scope='authoritative headless ECS; not Unreal, filtered replica, LAN, or full game acceptance',
    max_game_seconds=600,stall_game_seconds=300,max_ticks=36000,stall_ticks=18000,
    budget_scope=batch.BUDGET_SCOPE,stall_scope=batch.STALL_SCOPE,
    role_plan={players=players},winner_team=1,winner_side=0,map_id='three_lane_training'}
end
test('defaults and bounded unique seeds',function()
  local o=batch.options({},args)
  assert(o.matches==10 and o.seed==1 and o.max_game_seconds==600 and o.stall_game_seconds==300)
  assert(batch.options({'--matches','10'},args).matches==10)
  local bounds=batch.options({'--max-game-seconds','60','--stall-game-seconds','60'},args)
  assert(bounds.max_game_seconds==60 and bounds.stall_game_seconds==60)
  local upper=batch.options({'--max-game-seconds','3600','--stall-game-seconds','3600'},args)
  assert(upper.max_game_seconds==3600 and upper.stall_game_seconds==3600)
  for _,v in ipairs({{'--matches','0'},{'--matches','11'},{'--matches','100'},{'--matches','1.5'},{'--seed','4294967295','--matches','2'},
      {'--seed','-1'},{'--seed','1','--seed','2'},{'--recipe'},{'--profile','15'},
      {'--max-game-seconds','59'},{'--max-game-seconds','3601'},{'--max-game-seconds','60.0'},
      {'--max-game-seconds','18446744073709551616'},{'--stall-game-seconds','999999999999999999999'},
      {'--max-game-seconds','100'},{'--max-game-seconds','600','--max-game-seconds','700'},
      {'--stall-game-seconds','400','--max-game-seconds','300'},{'--max-game-seconds'}}) do
    rejects(function() batch.options(v,args) end)
  end
end)

test('programmatic execution cannot bypass ten-match cap or start work',function()
  local calls=0
  local function touched() calls=calls+1;error('must reject before callbacks') end
  for _,count in ipairs({0,11,100,1.5}) do
    rejects(function() batch.execute({matches=count,seed=1},{prepare=touched,run=touched,save=touched}) end)
  end
  assert(calls==0)
end)
test('formal evidence rejects weakened and incomplete matches',function()
  batch.verify(fixture(1),1)
  for key,value in pairs({success=false,profile_hz=15,seed=2,bot_mode='legacy_fixture',
      defender_policy='withdraw',finish_tick=0,replay_verified_ticks=899,end_events=2,committed_combat_facts=0,
      match_played=false,max_game_seconds=1800,stall_game_seconds=60,budget_scope='game rule',
      stall_scope='waves',max_ticks=1,stall_ticks=1}) do
    local f=fixture(1);f[key]=value;rejects(function() batch.verify(f,1) end)
  end
  local f=fixture(1);f.role_plan.players[1].bot=false;rejects(function() batch.verify(f,1) end)
end)
test('build once and retain independently named evidence',function()
  local prepared,runs,saved=0,{},{}
  local r=batch.execute({matches=3,seed=10},{prepare=function() prepared=prepared+1 end,
    run=function(seed) runs[#runs+1]=seed;return fixture(seed) end,
    report_path=function(i) return 'match-'..i..'.json' end,
    save=function(value) saved[#saved+1]=json.decode(json.encode(value)) end})
  assert(prepared==1 and table.concat(runs,',')=='10,11,12' and r.success and r.completed_matches==3)
  assert(r.matches[1].report~=r.matches[2].report and #saved==4 and saved[1].success==false)
  assert(r.max_game_seconds==600 and r.stall_game_seconds==300 and saved[1].budget_scope==batch.BUDGET_SCOPE)
end)
test('failed match stops and preserves completed reports',function()
  local calls=0
  local r=batch.execute({matches=3,seed=1},{prepare=function() end,
    run=function(seed) calls=calls+1;if seed==2 then error('fixture timeout') end;return fixture(seed) end,
    report_path=function(i) return 'match-'..i end,save=function() end})
  assert(not r.success and r.completed_matches==1 and calls==2 and r.error:find('fixture timeout',1,true))
end)
test('compiled contract removes inherited runtime Lua flags without mutating source',function()
  local source={OMB_LUA_CONTENT='1',OMB_LUA_HOT_RELOAD='1',other='retained'}
  local env=compiled.environment(source)
  assert(source.OMB_LUA_CONTENT=='1' and env.OMB_LUA_CONTENT=='0' and env.OMB_LUA_HOT_RELOAD=='0')
  assert(env.OMB_LUA_CONTENT_ROOT=='' and env.OMB_STORY_DATA_DIR=='' and env.other=='retained')
  local text='[server]\nSTEP_FPS=60\n[content]\nLUA_CONTENT=true\nLUA_HOT_RELOAD=true\nSCRIPTS_DIR="scripts"\n'
  local fixed=compiled.configuration(text,b.lib('toml'))
  local decoded=b.lib('toml').decode(fixed)
  assert(decoded.content.LUA_CONTENT==false and decoded.content.LUA_HOT_RELOAD==false)
  assert(decoded.content.SCRIPTS_DIR=='scripts' and decoded.server.STEP_FPS==60)
end)
test('smoke uses the shared compiled policy and tools parse without execution',function()
  local path=b.lib('path')
  for _,name in ipairs({'run_moba_runtime_smoke.lua','run_moba_headless_batch.lua','run_2player_ue.lua'}) do
    assert(loadfile(path.join(b.root,'scripts',name)))
  end
  local smoke=path.read(path.join(b.root,'scripts','run_moba_runtime_smoke.lua'))
  assert(smoke:find('compiled.configuration(config',1,true))
  assert(smoke:find('compiled.environment({',1,true))
  assert(smoke:find("args[#args + 1] = compiled.feature",1,true))
  assert(not smoke:find('runtime-lua-content',1,true))
  local ue=path.read(path.join(b.root,'scripts','run_2player_ue.lua'))
  assert(ue:find('compiled.configuration(game',1,true))
  assert(ue:find('"--presentation-hz", tostring(tick_rate)',1,true))
end)
test('verify rejects a longer budget and plan-only configuration',function()
  local budget={max_game_seconds=600,stall_game_seconds=300}
  batch.verify(fixture(1),1,budget)
  local longer=fixture(1);longer.max_game_seconds=1800;longer.max_ticks=1800*60
  rejects(function() batch.verify(longer,1,budget) end)
  local late=fixture(1);late.finish_tick=600*60+1;late.replay_verified_ticks=late.finish_tick
  rejects(function() batch.verify(late,1,budget) end)
  local plan=fixture(1);plan.scope='configuration only; no simulation or match acceptance';plan.match_played=false
  rejects(function() batch.verify(plan,1,budget) end)
  local lied=fixture(1);lied.max_ticks=1800*60
  rejects(function() batch.verify(lied,1,budget) end)
end)
test('requested budget is recorded and a longer run cannot pass verify',function()
  local requested={matches=1,seed=4,max_game_seconds=120,stall_game_seconds=60}
  local ok=batch.execute(requested,{prepare=function() end,run=function(seed)
    local row=fixture(seed);row.max_game_seconds=120;row.stall_game_seconds=60
    row.max_ticks=120*60;row.stall_ticks=60*60;row.finish_tick=100;row.replay_verified_ticks=100
    return row
  end,report_path=function(i) return 'match-'..i end,save=function() end})
  assert(ok.success and ok.max_game_seconds==120 and ok.stall_game_seconds==60)
  local fake=batch.execute(requested,{prepare=function() end,run=function(seed) return fixture(seed) end,
    report_path=function() return 'match' end,save=function() end})
  assert(not fake.success and fake.completed_matches==0 and fake.error:find('execution budget mismatch',1,true))
end)
test('winner team must be a planned positive team and a real draw stays absent',function()
  batch.verify(fixture(1),1)
  local other=fixture(1);other.winner_team=2;other.winner_side=1;batch.verify(other,1)
  local draw=fixture(1);draw.winner_team=nil;draw.winner_side=nil;batch.verify(draw,1)
  local draw_null=fixture(1);draw_null.winner_team=json.null;draw_null.winner_side=json.null
  batch.verify(draw_null,1)
  local zero=fixture(1);zero.winner_team=0;rejects(function() batch.verify(zero,1) end)
  local claimed=fixture(1);claimed.role_plan.players[1].team_id=0;claimed.winner_team=0
  rejects(function() batch.verify(claimed,1) end)
  local outsider=fixture(1);outsider.winner_team=7;rejects(function() batch.verify(outsider,1) end)
  local leftover=fixture(1);leftover.winner_team=nil;leftover.winner_side=0
  rejects(function() batch.verify(leftover,1) end)
  local side_only=fixture(1);side_only.winner_team=json.null;side_only.winner_side=1
  rejects(function() batch.verify(side_only,1) end)
  local missing=fixture(1);missing.role_plan.players[3].team_id=nil
  rejects(function() batch.verify(missing,1) end)
end)
test('single report is fresh unless an explicit path is repeated or occupied',function()
  local args_out,report=batch.with_single_report({'--seed','1'},function() return 'omb/target/moba-headless-runs/1-1' end)
  assert(report=='omb/target/moba-headless-runs/1-1/result.json' and args_out[#args_out]==report)
  assert(batch.paired_diagnostic_path(report)=='omb/target/moba-headless-runs/1-1/result.failure-samples.json')
  local same,explicit=batch.with_single_report({'--report','custom.json','--seed','1'},function() error('reserved') end)
  assert(explicit=='custom.json' and same[1]=='--report' and batch.paired_diagnostic_path(explicit)=='custom.failure-samples.json')
  local windows=batch.paired_diagnostic_path('D:\\code\\a\\match-0001.json')
  assert(windows=='D:\\code\\a\\match-0001.failure-samples.json')
  rejects(function() batch.with_single_report({'--report','a','--report','b'},function() end) end)
  rejects(function() batch.with_single_report({'--report'},function() end) end)
  batch.assert_fresh_output('a.json','a.failure-samples.json',function() return false end)
  rejects(function() batch.assert_fresh_output('a.json','a.failure-samples.json',function(path) return path=='a.json' end) end)
  rejects(function() batch.assert_fresh_output('a.json','a.failure-samples.json',function(path) return path~='a.json' end) end)
end)
print(('batch/compiled policy: %d tests passed; no game processes launched'):format(tests))
