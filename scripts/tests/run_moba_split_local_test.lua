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
-- No role rebuilds: the run builds every binary once under the lock (E286), the client runtime included.
local built=split.options({})
assert(table.concat(split.role_args(built,'server',nil,nil,2,'S'),' '):find('--no-build',1,true))
assert(table.concat(split.role_args(built,'client','PLAN',1,2,'C'),' '):find('--no-build',1,true))
do local all={}
  for _,c in ipairs(split.build_commands()) do all[#all+1]=c.exe..' '..table.concat(c.args,' ') end
  all=table.concat(all,'\n')
  assert(all:find('omobab',1,true) and all:find('omoba-client-runtime',1,true) and all:find('--release',1,true),
    'the split build covers the authority and the client runtime')
  -- E330: the UE frontend is built too, before the Rust binaries, unless a shadow or packaged game runs.
  local first=split.build_commands({ue_root='D:/UE5.8'})[1]
  assert(table.concat(first.args,' '):find('build_ue_moba.lua --build-only --ue-root D:/UE5.8',1,true),'the frontend is built first')
  assert(not table.concat(split.build_commands({},'D:/shadow')[1].args,' '):find('build_ue_moba',1,true),'a shadow project is staged elsewhere')
  assert(not table.concat(split.build_commands({packaged_game='G'})[1].args,' '):find('build_ue_moba',1,true),'a packaged game has no frontend build') end
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
local h=split.options({'--hero','1=date_masamune','--hero','6=omatsu'})
local hs=table.concat(split.role_args(h,'server',nil,nil,2,'S'),' ')
assert(hs:find('--hero 1=date_masamune',1,true) and hs:find('--hero 6=omatsu',1,true))
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
-- E325: the whole lock hold, shutdown included, fits inside --max-seconds.
do
  local retire,hard=split.box_deadlines(1000,600)
  assert(hard==601000 and retire==hard-split.SHUTDOWN_RESERVE_MS,'play ends one shutdown reserve before the hard deadline')
  assert(split.SHUTDOWN_RESERVE_MS>=15000,'the reserve covers one bounded child wait')
  local short_retire,short_hard=split.box_deadlines(0,20)
  assert(short_hard==20000 and short_retire==10000,'a short box keeps half of itself for play')
  assert(split.box_deadlines(0,nil)==nil,'no box, no deadline')
  assert(split.shutdown_wait_ms(590000,601000,15000)==11000,'a child wait never runs past the hard deadline')
  assert(split.shutdown_wait_ms(605000,601000,15000)==0,'past the deadline a child is terminated without waiting')
  assert(split.shutdown_wait_ms(0,nil,15000)==15000,'without a box the usual bounded wait applies')
end
-- E343: a measured match takes the performance lock only after the build and the live lock,
-- releases it with the live lock, and is always time boxed.
do
  local o=split.options({'--performance-lock','--max-seconds','600','--no-build'})
  assert(o.performance_lock and o.max_seconds==600)
  assert(not pcall(split.options,{'--performance-lock'}),'a measurement needs --max-seconds')
  local f=assert(io.open('scripts/run_moba_split_local.lua','rb'));local text=f:read('a');f:close()
  local run=assert(text:find('function M.run(options)',1,true))
  local build=assert(text:find('M.build_all(options)',run,true))
  local live=assert(text:find("machine_lock').acquire",run,true))
  local perf=assert(text:find('M.acquire_performance(root,options)',run,true))
  assert(build<live and live<perf,'performance lock after the build and the live lock')
  assert(split.PERFORMANCE_WAIT_S<=300,'the wait for another measurement stays well inside the box')
end
-- E337: the build runs before the live lock, so the frontend build never uses the time box.
do
  local f=assert(io.open('scripts/run_moba_split_local.lua','rb'));local text=f:read('a');f:close()
  local run=assert(text:find('function M.run(options)',1,true))
  local build=assert(text:find('M.build_all(options)',run,true),'the run builds')
  local lock=assert(text:find("machine_lock').acquire",run,true),'the run takes the live lock')
  assert(build<lock,'the build happens before the live lock is taken')
  assert(not text:find('while building',run,true),'no build step inside the time box')
end
-- E533 (replaces the E399 rebuild): nothing is built after the live lock is taken; the roles
-- launch the snapshot's set (E516). The set is read afresh each time.
do
  local f=assert(io.open('scripts/run_moba_split_local.lua','rb'));local text=f:read('a');f:close()
  local run=assert(text:find('function M.run(options)',1,true))
  local lock=assert(text:find("machine_lock').acquire",run,true))
  local build=assert(text:find('M.build_all(options)',run,true))
  assert(build<lock and not text:find('M.build_all(options)',lock,true),'the only build is before the live lock')
  local set=split.script_set()
  assert(set:find('base_content.dll',1,true) and set:find('items.dll',1,true),'the set names the script DLLs')
  assert(split.script_set()==set,'reading it again gives the same set')
end
-- E487: the repo build lock covers the frontend build and the source snapshot only; cargo reads
-- the snapshot after the lock is released, its outputs in the launch's target directories.
do
  local events,held={},{}
  local function guard(name)
    return setmetatable({release=function() if held[name] then held[name]=nil;events[#events+1]='release '..name end end,
      env=function() return {OM_BUILD_LOCK_TOKEN='T'} end},{__close=function(self) self.release() end})
  end
  -- E522: the snapshot is a real tree here, the cargo commands are made from its hero_dlls.lua.
  local snap=(os.getenv('TEMP'):gsub('\\','/'))..'/om_e522_'..os.time()
  local lfs=require('lfs')
  local deps={snapshot=snap,jobs=function() return 3 end,log=function() end,clock=function() return 0 end,
    target_of=function(crate) return 'D:/targets/'..crate end,
    lock={acquire=function(_,label,o) local name=o.name or 'build';held[name]=true;events[#events+1]='acquire '..name;return guard(name) end},
    mirror=function(root,to)
      assert(held.build,'the snapshot is taken under the build lock')
      lfs.mkdir(to);lfs.mkdir(to..'/scripts')
      local out=assert(io.open(to..'/scripts/hero_dlls.lua','wb'))
      out:write("return {{hero='oichi',package='hero_c17',dir='heroes/oichi'}}\n");out:close()
      events[#events+1]='mirror';return 0
    end,
    run=function(command,env)
      local text=table.concat(command.args,' '):gsub('\\','/')
      if command.locked then assert(held.build and env.OM_BUILD_LOCK_TOKEN=='T','the frontend builds live, under the lock')
      elseif command.exe=='cargo' then
        assert(not held.build,'cargo runs after the build lock is released')
        assert(held[split.snapshot_lock_name(snap)],'cargo runs under the snapshot lock')
        assert(text:find(snap,1,true),'cargo reads the snapshot: '..text)
        if command.crate=='scripts' then
          assert(text:find('-p hero_c17',1,true) and not text:find('-p hero_a02',1,true),'cargo builds the snapshot set: '..text)
        end
        assert(env.CARGO_TARGET_DIR=='D:/targets/'..command.crate and env.CARGO_BUILD_JOBS=='3')
      end
      events[#events+1]=command.locked and 'frontend' or command.exe; return {exit_code=0}
    end}
  split.build_all({ue_root='D:/UE5.8'},deps)
  local order=table.concat(events,',')
  local name=split.snapshot_lock_name(snap):gsub('%p','%%%0')
  assert(order:find('^acquire '..name..',acquire build,frontend,mirror,release build,cargo'),order)
  assert(order:find('release '..name..'$'),'the snapshot lock is released last: '..order)
  os.remove(snap..'/scripts/hero_dlls.lua');lfs.rmdir(snap..'/scripts');lfs.rmdir(snap)
  assert(split.snapshot_root('D:/omoba',function() return 'D:\\omoba\\target\\lineC-bins' end):gsub('\\','/'):find('lineC%-bins/split%-src$'))
  assert(split.snapshot_root('D:/omoba',function() return nil end):gsub('\\','/')=='D:/omoba/target/split-src')
end
-- E507: a build from the source snapshot stages and compares the snapshot's script DLL set, so a
-- hero made playable while cargo ran from the snapshot is not staged before it is built.
do
  local tree=(os.getenv('TEMP'):gsub('\\','/'))..'/om_e504_'..os.time()
  require('lfs').mkdir(tree);require('lfs').mkdir(tree..'/scripts')
  local f=assert(io.open(tree..'/scripts/hero_dlls.lua','wb'))
  f:write("return {{hero='inahime',package='hero_a02',dir='heroes/inahime'}}\n");f:close()
  local set=split.script_set(tree)
  assert(set=='base_content.dll,hero_a02.dll,items.dll','the snapshot set is read from its hero_dlls.lua: '..set)
  assert(split.script_set()~=set,'the repo set is read without a root')
  local stage
  for _,command in ipairs(require('moba_server_build').commands({profile='release',root=tree})) do
    if table.concat(command.args,' '):find('stage-dll',1,true) then stage=table.concat(command.args,' ') end
  end
  assert(stage and stage:find('--set-root '..tree,1,true),'stage-dll stages the set of the tree it built: '..tostring(stage))
  for _,command in ipairs(require('moba_server_build').commands({profile='release'})) do
    assert(not table.concat(command.args,' '):find('--set-root',1,true),'a repo build stages the repo set')
  end
  local text=assert(io.open('scripts/run_moba_split_local.lua','rb')):read('a')
  -- E516: the roles read the built set through the environment, a hero landed while they start
  -- does not change what they launch.
  local spawn=assert(text:find('child_env.OMOBA_SOURCES_AS_OF',1,true))
  assert(text:find("child_env[require('script_dlls').SET_ROOT_ENV]=M.snapshot_root(b.root)",spawn,true),'the roles get the snapshot set')
  local dlls=assert(io.open('scripts/script_dlls.lua','rb')):read('a')
  assert(dlls:find('os.getenv(M.SET_ROOT_ENV)',1,true),'script_dlls.modules reads the set root from the environment')
  os.remove(tree..'/scripts/hero_dlls.lua');require('lfs').rmdir(tree..'/scripts');require('lfs').rmdir(tree)
end
-- E540: a role that ended on a failure (errors.md in its run directory) fails the run with the
-- reason; a role without one (a finished match, a closed window) does not.
do
  local lfs=require('lfs')
  local root=(os.getenv('TEMP'):gsub('\\','/'))..'/om_e540_'..os.time()
  lfs.mkdir(root);lfs.mkdir(root..'/p1')
  assert(split.role_failure(root,'p1')==nil,'no errors.md, no failure')
  local out=assert(io.open(root..'/p1/errors.md','wb'))
  out:write('# Role launch error\n\nD:/omoba/scripts/moba_role_launch.lua:910: runtime-p1 exited during match\nstack traceback:\n');out:close()
  assert(split.role_failure(root,'p1'):find('runtime-p1 exited during match',1,true),'the first line after the title is the reason')
  assert(split.role_failure(root,'netem')==nil,'a role without a directory has no failure')
  local text=assert(io.open('scripts/run_moba_split_local.lua','rb')):read('a')
  local ended=assert(text:find("' ended; retiring the rest of this split run'",1,true))
  assert(text:find("if failure then error(child.label..' ended on a failure: '..failure,0) end",ended,true),'an ended role with a failure fails the run')
  os.remove(root..'/p1/errors.md');lfs.rmdir(root..'/p1');lfs.rmdir(root)
end
print('split local topology: options, seats and per-role arguments passed')
