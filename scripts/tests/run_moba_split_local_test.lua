package.path='scripts/?.lua;'..package.path
local split=require('run_moba_split_local')

local o=split.options({'--autopilot','lane_v0','--no-build'})
assert(o.autopilot=='lane_v0' and o.no_build and o.recipe:find('moba_two_player',1,true))
assert(not pcall(split.options,{'--autopilot','bad name'}))
assert(not pcall(split.options,{'--bogus'}))
assert(split.options({'--max-seconds','600'}).max_seconds==600)
do
  local b=require('_bootstrap');local path=b.lib('path')
  local root=path.mkdir_p(path.join(b.root,'target','tests','split-finished'))
  for _,id in ipairs({1,6}) do path.mkdir_p(path.join(root,'p'..id)) end
  path.write(path.join(root,'p1','autopilot-p1.jsonl'),'{"event":"match_finished","tick":9}\n',true)
  path.write(path.join(root,'p6','autopilot-p6.jsonl'),'{"event":"decision"}\n',true)
  assert(not split.all_finished(root,{1,6}),'one renderer still playing')
  path.write(path.join(root,'p6','autopilot-p6.jsonl'),'{"event":"match_finished","tick":9}\n',true)
  assert(split.all_finished(root,{1,6}) and not split.all_finished(root,{}))
end
assert(not pcall(split.options,{'--max-seconds','0'}) and not pcall(split.options,{'--max-seconds','ten'}))

-- Human seats come from the locked plan, sorted; all-Bot or oversize plans are refused.
assert(table.concat(split.humans({players={{player_id=6,bot=false},{player_id=2,bot=true},{player_id=1,bot=false}}}),',')=='1,6')
assert(not pcall(split.humans,{players={{player_id=1,bot=true}}}))

-- Every invocation gets identical host totals; only clients carry identity and autopilot.
local server=table.concat(split.role_args(o,'server',nil,nil,2,'S'),' ')
assert(server:find('--server-only',1,true) and server:find('--host-simulations 3 --host-renderers 2',1,true))
assert(not server:find('--autopilot',1,true) and not server:find('--local-player',1,true))
assert(server:find('--debug-query',1,true) and not table.concat(split.role_args(o,'client','PLAN',6,2,'C'),' '):find('--debug-query',1,true))
local client=table.concat(split.role_args(o,'client','PLAN',6,2,'C'),' ')
-- E124: clients load the authority's run copy of base_content, never a build of their own.
do local shared=table.concat(split.role_args(o,'client','S/match-plan.json',6,2,'C'),' ')
  assert(shared:find('--base-content S'..package.config:sub(1,1),1,true) or shared:find('--base-content S/',1,true))
  assert(shared:find('artifacts',1,true) and not server:find('--base-content',1,true)) end
-- E086: the authority records its accepted inputs by default; clients never do.
assert(server:find('--accepted-inputs S',1,true) and not table.concat(split.role_args(o,'client','PLAN',6,2,'C'),' '):find('--accepted-inputs',1,true))
do local off={} for k,v in pairs(o) do off[k]=v end off.no_accepted_inputs=true
  assert(not table.concat(split.role_args(off,'server',nil,nil,2,'S'),' '):find('--accepted-inputs',1,true)) end
assert(client:find('--connect 127.0.0.1 --recipe PLAN --local-player 6',1,true) and client:find('--no-build',1,true))
assert(client:find('--host-simulations 3 --host-renderers 2',1,true) and client:find('--autopilot lane_v0',1,true))
-- Clients never rebuild mid-run even when the authority is allowed to build.
local built=split.options({})
assert(not table.concat(split.role_args(built,'server',nil,nil,2,'S'),' '):find('--no-build',1,true))
assert(table.concat(split.role_args(built,'client','PLAN',1,2,'C'),' '):find('--no-build',1,true))
assert(#split.humans(require('moba_role_launch').load_recipe({recipe='scripts/lua_data/moba_two_player.lua',hero_selections={}}))==2)
-- Netem: each team client enters through its own proxy bind; the authority is untouched.
local n=split.options({'--netem','wan-80-20-loss1','--netem-seed','7'})
assert(n.netem=='wan-80-20-loss1' and n.netem_seed==7)
assert(not pcall(split.options,{'--netem','bad profile'}))
local t1=table.concat(split.role_args(n,'client','PLAN',1,2,'C',1),' ')
local t2=table.concat(split.role_args(n,'client','PLAN',6,2,'C',2),' ')
assert(t1:find('--port 57261',1,true) and t2:find('--port 57361',1,true))
assert(not table.concat(split.role_args(n,'server',nil,nil,2,'S'),' '):find('--port',1,true))
local proxy=table.concat(split.netem_args(n,57061,'E'),' ')
assert(proxy:find('--server 127.0.0.1:57061',1,true) and proxy:find('--team1-profile wan-80-20-loss1',1,true) and proxy:find('--seed 7',1,true))
-- Scheduled single-team outages: forwarded to the proxy in ms, require --netem.
local out=split.options({'--netem','wan-80-20-loss1','--netem-outage','1:120:10','--netem-outage','2:360:40'})
local outage=table.concat(split.netem_args(out,57061,'E'),' ')
assert(outage:find('--outage team=1,at_ms=120000,duration_ms=10000',1,true) and outage:find('--outage team=2,at_ms=360000,duration_ms=40000',1,true))
assert(not table.concat(split.netem_args(n,57061,'E'),' '):find('--outage',1,true))
for _,bad in ipairs({'3:1:1','1:1:0','1:x:5','1:5'}) do assert(not pcall(split.options,{'--netem','fixed-20','--netem-outage',bad}),bad) end
assert(not pcall(split.options,{'--netem-outage','1:120:10'}),'outage needs a proxy')
local h=split.options({'--hero','1=date_masamune','--hero','6=training_frost_mage'})
local hs=table.concat(split.role_args(h,'server',nil,nil,2,'S'),' ')
assert(hs:find('--hero 1=date_masamune',1,true) and hs:find('--hero 6=training_frost_mage',1,true))
assert(not table.concat(split.role_args(h,'client','PLAN',1,2,'C'),' '):find('--hero',1,true))
assert(not pcall(split.options,{'--hero','date_masamune'}))
-- Hero spotlight: the script goes to renderer seats only, with the spotlight autopilot,
-- and a finished demonstration ends the run like a finished match.
assert(not pcall(split.options,{'--spotlight-script','S.json'}))
assert(not pcall(split.options,{'--autopilot','lane_v0','--spotlight-script','S.json'}))
local sp=split.options({'--autopilot','spotlight','--spotlight-script','S.json'})
assert(table.concat(split.role_args(sp,'client','PLAN',1,1,'C'),' '):find('--autopilot spotlight --spotlight-script S.json',1,true))
assert(not table.concat(split.role_args(sp,'server',nil,nil,1,'S'),' '):find('--spotlight-script',1,true))
do
  local b=require('_bootstrap');local path=b.lib('path')
  local root=path.mkdir_p(path.join(b.root,'target','tests','split-spotlight-finished'))
  path.mkdir_p(path.join(root,'p1'))
  path.write(path.join(root,'p1','autopilot-p1.jsonl'),'{"event":"spotlight_slot","slot":0}\n',true)
  assert(not split.all_finished(root,{1}),'still demonstrating')
  path.write(path.join(root,'p1','autopilot-p1.jsonl'),'{"event":"spotlight_finished","reason":"complete"}\n',true)
  assert(split.all_finished(root,{1}))
end
print('split local topology: options, seats and per-role arguments passed')
