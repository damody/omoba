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
  local players={};for i=1,10 do players[i]={bot=true} end
  return {success=true,profile_hz=60,seed=seed,bot_mode='committed_role_plan',defender_policy='guard',
    finish_tick=900,replay_verified_ticks=900,end_events=1,committed_combat_facts=5,
    role_plan={players=players},winner_team=1,map_id='three_lane_training'}
end
test('defaults and bounded unique seeds',function()
  local o=batch.options({},args);assert(o.matches==100 and o.seed==1)
  for _,v in ipairs({{'--matches','0'},{'--matches','1.5'},{'--seed','4294967295','--matches','2'},
      {'--seed','-1'},{'--seed','1','--seed','2'},{'--recipe'},{'--profile','15'}}) do
    rejects(function() batch.options(v,args) end)
  end
end)
test('formal evidence rejects weakened and incomplete matches',function()
  batch.verify(fixture(1),1)
  for key,value in pairs({success=false,profile_hz=15,seed=2,bot_mode='legacy_fixture',
      defender_policy='withdraw',finish_tick=0,replay_verified_ticks=899,end_events=2,committed_combat_facts=0}) do
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
print(('batch/compiled policy: %d tests passed; no game processes launched'):format(tests))
