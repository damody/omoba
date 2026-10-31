-- Same-machine split topology (test-topology.md mode B+C+C): one dedicated
-- authority plus one --connect client per human seat, each a separate
-- run_moba_role_ue.lua invocation exactly as on separate LAN machines. Supplies
-- the shared host totals, per-seat player IDs, optional client autopilot and a
-- staggered start so only the first renderer may warm shaders. Stopping this
-- workflow retires every child invocation and, through their own supervision,
-- all of their processes.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,time,json,platform,udp=b.lib('path'),b.lib('process'),b.lib('time'),b.lib('json'),b.lib('platform'),b.lib('udp')
local lfs=require('lfs')
local M={}

function M.options(args)
  local options={recipe='scripts/lua_data/moba_two_player.lua',ue_root='D:/UE5.8',no_build=false,admit_timeout_ms=3*3600*1000,heroes={},netem_outages={}}
  local i=1
  while i<=#args do
    local key=args[i]
    if key=='--no-build' then options.no_build=true
    -- The authority records its accepted inputs by default (E086 offline replay).
    elseif key=='--no-accepted-input-log' then options.no_accepted_inputs=true
    -- M3' ② dev hot reload (scripts/hero_hot_reload.lua --run <authority run dir>).
    elseif key=='--script-hot-reload' then options.script_hot_reload=true
    -- A measured match (E343): the performance lock is held only while this match holds the
    -- machine (after the build, inside --max-seconds), never while building or queueing.
    elseif key=='--performance-lock' then options.performance_lock=true
    elseif key=='--hero' then
      i=i+1
      local value=assert(args[i],'--hero requires PLAYER_ID=HERO')
      assert(value:match('^%d+=[%w_]+$'),'--hero requires PLAYER_ID=HERO')
      options.heroes[#options.heroes+1]=value
    elseif key=='--netem-outage' then
      -- TEAM:AT_S:DURATION_S, measured from the moment every human is admitted.
      i=i+1
      local team,at,duration=tostring(args[i] or ''):match('^([12]):(%d+):(%d+)$')
      assert(team,'--netem-outage requires TEAM:AT_SECONDS:DURATION_SECONDS with team 1 or 2')
      assert(tonumber(duration)>0,'--netem-outage duration must be positive')
      options.netem_outages[#options.netem_outages+1]={team=tonumber(team),at_s=tonumber(at),duration_s=tonumber(duration)}
    elseif key=='--recipe' or key=='--ue-root' or key=='--autopilot' or key=='--output' or key=='--port'
      or key=='--netem' or key=='--netem-seed' or key=='--packaged-game' or key=='--max-seconds' or key=='--resolution'
      or key=='--spotlight-script' or key=='--autopilot-shot-interval' or key=='--frame-cap' or key=='--wait-live-lock' then
      i=i+1;options[key:sub(3):gsub('%-','_')]=assert(args[i],key..' requires a value')
    else error('unknown argument: '..key) end
    i=i+1
  end
  if options.autopilot then assert(options.autopilot:match('^[%w_%-]+$'),'--autopilot requires a profile name') end
  assert(not options.spotlight_script or options.autopilot=='spotlight','--spotlight-script requires --autopilot spotlight')
  if options.netem then assert(options.netem:match('^[%w_%-]+$'),'--netem requires a proxy profile name') end
  if options.max_seconds then
    options.max_seconds=assert(math.tointeger(tonumber(options.max_seconds)),'--max-seconds must be an integer')
    assert(options.max_seconds>0,'--max-seconds must be positive')
  end
  assert(#options.netem_outages==0 or options.netem,'--netem-outage requires --netem PROFILE')
  assert(not options.performance_lock or options.max_seconds,'--performance-lock requires --max-seconds (a measurement is time boxed)')
  -- One live match per machine (E197): how long to wait for another one to end.
  options.wait_live_lock=require('machine_lock').wait_seconds(options.wait_live_lock)
  if options.netem_seed then
    options.netem_seed=assert(math.tointeger(tonumber(options.netem_seed)),'--netem-seed must be an integer')
  end
  return options
end

-- Every autopilot trace under ROOT/p<id> recorded match_finished (or, for a
-- spotlight recording, spotlight_finished: the demonstration ends the run).
function M.all_finished(root,humans)
  for _,player_id in ipairs(humans) do
    local trace=path.join(root,'p'..player_id,'autopilot-p'..player_id..'.jsonl')
    -- The renderer appends to its trace while this polls: a sharing violation on
    -- Windows is a transient "not finished yet", never a reason to abort the run.
    local file=path.is_file(trace) and io.open(trace,'rb')
    local text=file and file:read('a') or ''
    if file then file:close() end
    if not text:find('"event":"match_finished"',1,true) and not text:find('"event":"spotlight_finished"',1,true) then return false end
  end
  return #humans>0
end

function M.humans(plan)
  local ids={}
  for _,player in ipairs(assert(plan.players,'match plan has no players')) do
    if player.bot==false then ids[#ids+1]=player.player_id end
  end
  table.sort(ids)
  assert(#ids>=1 and #ids<=10,'split topology needs 1..10 human seats')
  return ids
end

-- Identical totals for every invocation: one authority + one runtime per human; one renderer per human.
-- Team client binds on the proxy; distinct ranges so presentation ports (bind + player ID) never collide.
M.NETEM_TEAM_PORT={[1]=57261,[2]=57361}
M.NETEM_CONTROL='127.0.0.1:63200'
M.NETEM_UPSTREAM={[1]='127.0.0.1:63101',[2]='127.0.0.1:63102'}

function M.netem_args(options,authority_port,evidence)
  local profile=options.netem
  return {'--server','127.0.0.1:'..authority_port,'--control-bind',M.NETEM_CONTROL,
    '--team1-client-bind','127.0.0.1:'..M.NETEM_TEAM_PORT[1],'--team2-client-bind','127.0.0.1:'..M.NETEM_TEAM_PORT[2],
    '--team1-upstream-bind',M.NETEM_UPSTREAM[1],'--team2-upstream-bind',M.NETEM_UPSTREAM[2],
    '--team1-profile',profile,'--team2-profile',profile,'--mode','ordered-delay',
    '--seed',tostring(options.netem_seed or 88442211),'--evidence-dir',evidence,
    table.unpack(M.outage_args(options))}
end

function M.outage_args(options)
  local args={}
  for _,o in ipairs(options.netem_outages or {}) do
    args[#args+1]='--outage'
    args[#args+1]=('team=%d,at_ms=%d,duration_ms=%d'):format(o.team,o.at_s*1000,o.duration_s*1000)
  end
  return args
end

-- Graceful stop so the proxy writes its route/outage summary and evidence.
function M.stop_netem(record)
  udp.send(M.NETEM_CONTROL,json.encode({version=1,action='shutdown'}))
  process.wait_owned(record,7000)
end

function M.role_args(options,role,plan_file,player_id,humans,output,team_id)
  local args={path.join(b.root,'scripts','run_moba_role_ue.lua')}
  local function add(...) for _,value in ipairs({...}) do args[#args+1]=value end end
  -- Local dev topology: the authority answers omb-mcp debug queries (never a LAN default).
  if role=='server' then
    add('--server-only','--debug-query','--recipe',options.recipe)
    -- Hero choices lock into the authority's plan; clients join that plan as-is.
    for _,hero in ipairs(options.heroes or {}) do add('--hero',hero) end
    if not options.no_accepted_inputs then add('--accepted-inputs',path.join(output,'accepted-inputs.jsonl')) end
    if options.script_hot_reload then add('--script-hot-reload') end
  else
    add('--connect','127.0.0.1','--recipe',plan_file,'--local-player',tostring(player_id),'--ue-root',options.ue_root)
    -- Same machine, same match: clients load the authority's own base_content copy (E124).
    add('--base-content',path.join(path.parent(plan_file),'artifacts','scripts','base_content.dll'))
  end
  -- No role builds: this run built every binary once, under the lock (M.build_commands).
  add('--no-build')
  if role~='server' and options.netem then
    add('--port',tostring(assert(M.NETEM_TEAM_PORT[team_id],'netem needs team 1 or 2')))
  elseif options.port then add('--port',options.port) end
  add('--host-simulations',tostring(1+humans),'--host-renderers',tostring(humans),'--output',output)
  if role~='server' and options.autopilot then add('--autopilot',options.autopilot) end
  if role~='server' and options.spotlight_script then add('--spotlight-script',options.spotlight_script) end
  if role~='server' and options.autopilot_shot_interval then add('--autopilot-shot-interval',options.autopilot_shot_interval) end
  if role~='server' and options.packaged_game then add('--packaged-game',options.packaged_game) end
  if role~='server' and options.resolution then add('--resolution',options.resolution) end
  if role~='server' and options.frame_cap then add('--frame-cap',options.frame_cap) end
  return args
end

-- Every Rust binary a split run starts, built once by the run itself before any role
-- launches (E286): the authority, its script DLLs and the client runtime. Building in the authority
-- role alone left the client runtime as it was, and other sessions change shared sources between a
-- build made beforehand and the launch, so a client refused its stale binary. The sources are
-- checked as of this build (E202), so changes landed while the run waits for the machine do not
-- refuse it; the build itself happens before the live lock and its time box (E337).
-- The UE frontend (codegen, bridge, UBT) is built first, like run_moba_role_ue does (E330): with every
-- role on --no-build nothing rebuilt it, so content landed since the last frontend build gave the
-- renderer bridge another content hash than the authority and every launch was refused. A shadow
-- project (OM_UE_RUN_BINARIES, staged by stage_ue_shadow) or a packaged game is not built here.
-- SOURCE_ROOT (default the repo) is where the cargo manifests are read (the source snapshot, E487);
-- the frontend build (`locked`) always builds live, under the repo build lock.
function M.build_commands(options,ue_shadow,source_root)
  options=options or {}
  local commands={}
  if not options.packaged_game and not ue_shadow then
    commands[1]={exe=platform.lua_executable,locked=true,args={path.join(b.root,'scripts','build_ue_moba.lua'),'--build-only','--ue-root',options.ue_root or 'D:/UE5.8'}}
  end
  for _,command in ipairs(require('moba_server_build').commands({profile='release',root=source_root})) do commands[#commands+1]=command end
  commands[#commands+1]={exe='cargo',crate='omoba-client-runtime',args={'build','--manifest-path',path.join(source_root or b.root,'omoba-client-runtime','Cargo.toml'),
    '--features','compiled-content-only','--release'}}
  return commands
end

-- The part of the time box kept for retiring the children: the run stops starting or
-- playing at the retire deadline, and the lock is released by the hard deadline (E325).
M.SHUTDOWN_RESERVE_MS=30000
-- lock acquisition time (ms), --max-seconds -> retire deadline, hard deadline (nil without a box)
function M.box_deadlines(start_ms,max_seconds)
  if not max_seconds then return nil,nil end
  local hard=start_ms+max_seconds*1000
  return hard-math.min(M.SHUTDOWN_RESERVE_MS,max_seconds*500),hard
end
-- Milliseconds one child may still take to exit before the hard deadline (at least 0).
function M.shutdown_wait_ms(now_ms,hard_deadline,cap_ms)
  if not hard_deadline then return cap_ms end
  return math.max(0,math.min(cap_ms,hard_deadline-now_ms))
end

local function wait_for(record,file,needle,timeout_ms,label,box_deadline)
  local deadline=time.monotonic_ms()+timeout_ms
  while true do
    assert(not box_deadline or time.monotonic_ms()<box_deadline,
      label..': time box reached before "'..needle..'" (startup counts toward --max-seconds)')
    local text=path.is_file(file) and path.read(file) or ''
    if text:find(needle,1,true) then return end
    assert(not text:find('stack traceback',1,true),label..' failed; see '..file)
    assert(process.owned_alive(record),label..' exited before "'..needle..'"; see '..file)
    assert(time.monotonic_ms()<deadline,label..' timed out waiting for "'..needle..'"')
    time.sleep_ms(1000)
  end
end

-- Builds every binary of the match once, under one repo build lock (E301: the bridge and the
-- authority are built from the same content; build_ue_moba joins it through the token in the
-- environment). Runs before the live lock is taken (E337): the frontend build (codegen, UBT) can
-- take minutes, and inside the lock it used up the whole --max-seconds before the match started.
-- Returns the moment the sources were built as of, for OMOBA_SOURCES_AS_OF (E202).
-- E487: the repo build lock is held only for the frontend build and a mirror of the Rust/Lua
-- sources (land_to_live.TREE) into a snapshot beside the run's target directory; the cargo builds
-- (every hero DLL, omobab, the client runtime: 17 of a split's 22 minutes after a shared crate
-- changed) then read the snapshot with the lock released, so lands go on meanwhile. The snapshot
-- is taken under the same lock as the frontend build, so the binaries still carry one content
-- hash, and the dep-info of each binary names the snapshot's files (E190 checks those). One
-- snapshot lock per snapshot directory keeps two splits from rewriting it under each other.
M.BUILD_WAIT_S=1800
M.CARGO_JOB_COMMIT_BYTES=5*1024*1024*1024//2
M.SNAPSHOT='split-src'
-- The snapshot directory: inside OM_RUST_RUN_BINARIES when the run has its own target, else target/.
function M.snapshot_root(root,getenv)
  local own=(getenv or os.getenv)('OM_RUST_RUN_BINARIES')
  return path.join(own and own~='' and path.absolute(own,root) or path.join(root,'target'),M.SNAPSHOT)
end
function M.snapshot_lock_name(snapshot)
  return 'split_src-'..(snapshot:lower():gsub('[^%w]+','_'))
end
function M.mirror_sources(root,snapshot)
  local land,shadow=require('land_to_live'),require('stage_ue_shadow')
  local copied=0
  for _,entry in ipairs(land.TREE) do
    local from,to=path.join(root,entry),path.join(snapshot,entry)
    if path.is_directory(from) then copied=copied+shadow.mirror(from,to,{root=root},land.SKIP)
    elseif path.is_file(from) then
      local bytes=path.read(from,true)
      if not path.is_file(to) or path.read(to,true)~=bytes then path.write(to,bytes,true,true);copied=copied+1 end
    end
  end
  return copied
end
function M.build_all(options,deps)
  deps=deps or {}
  local locks=deps.lock or require('build_lock')
  local run=deps.run or function(command,env) return process.run(command.exe,command.args,{cwd=b.root,check=false,env=env}) end
  local mirror=deps.mirror or M.mirror_sources
  local jobs=deps.jobs or function() return require('memory_admission').parallel_jobs(M.CARGO_JOB_COMMIT_BYTES,24) end
  local log=deps.log or print
  local clock=deps.clock or time.monotonic_ms
  local target_of=deps.target_of or function(crate) return require('run_artifacts').rust_target(b.root,crate) end
  local snapshot=deps.snapshot or M.snapshot_root(b.root)
  local function step(command,env)
    local started=clock()
    -- Cargo jobs by the commit left when each build starts (E443).
    env.CARGO_BUILD_JOBS=tostring(jobs())
    local result=run(command,env)
    log(('[split-build] %s %s: %d s'):format(command.exe:match('[^/\\]+$') or command.exe,
      command.crate or (command.args[1] or ''):match('[^/\\]+$') or '',(clock()-started)//1000))
    if result.exit_code~=0 then
      io.stderr:write(result.stderr or '')
      error('split build failed: '..command.exe..' '..table.concat(command.args,' '),0)
    end
  end
  -- The wait for another build is bounded like the wait for the machine (E346): a build that
  -- holds the lock longer fails this run with the holder named instead of queueing for hours.
  local wait={wait_ms=(options.wait_live_lock or M.BUILD_WAIT_S)*1000}
  local snapshot_lock <close> = locks.acquire(b.root,'run_moba_split_local cargo from '..snapshot,
    {wait_ms=wait.wait_ms,name=M.snapshot_lock_name(snapshot)})
  local build_lock <close> = locks.acquire(b.root,'run_moba_split_local build phase',wait)
  local as_of=os.time()
  -- Only the frontend (locked) commands are taken from this list; the rest are made after the snapshot.
  local commands=M.build_commands(options,os.getenv('OM_UE_RUN_BINARIES'))
  for _,command in ipairs(commands) do if command.locked then step(command,build_lock:env()) end end
  local started=clock()
  local copied=mirror(b.root,snapshot)
  log(('[split-build] source snapshot %s: %d file(s) in %d s; build lock released'):format(snapshot,copied,(clock()-started)//1000))
  if build_lock.release then build_lock:release() end
  -- E522: the cargo commands are made from the snapshot just taken, so the hero packages they
  -- build are the snapshot's set; one made before the build lock was waited for missed a hero
  -- landed meanwhile, and stage-dll (the snapshot's set) found its DLL missing.
  commands=M.build_commands(options,os.getenv('OM_UE_RUN_BINARIES'),snapshot)
  for _,command in ipairs(commands) do
    if not command.locked then
      -- Outputs go where the launch reads them (run_artifacts.rust_target), not under the snapshot.
      step(command,command.crate and {CARGO_TARGET_DIR=target_of(command.crate)} or {})
    end
  end
  return as_of
end

-- The performance lock of a measured match (E343): holds as long as the match's processes run,
-- at most the box (grace before the first process is attached).
M.PERFORMANCE_WAIT_S=120
function M.acquire_performance(root,options)
  return require('machine_lock').acquire(b.root,'performance',{run=root,label='run_moba_split_local measured match',
    wait_seconds=math.min(M.PERFORMANCE_WAIT_S,options.max_seconds//2),grace_seconds=options.max_seconds,
    wait_flag='(another measurement holds it; rerun later)'})
end

-- The script DLL set as the sources list it now (base_content, items, every playable hero's DLL),
-- read afresh: hero_dlls.lua changes when a hero is made playable (E399). ROOT (optional) reads
-- a source snapshot's set: a build reads the set of the snapshot it built from (E507).
function M.script_set(root)
  package.loaded['hero_dlls']=nil
  local files={}
  for _,module in ipairs(require('script_dlls').modules(root)) do files[#files+1]=module.file end
  table.sort(files)
  return table.concat(files,',')
end

function M.run(options)
  -- Built before waiting for the machine (E337); without a build, the sources as of the match start.
  local sources_as_of=not options.no_build and M.build_all(options) or nil
  -- E507: the set built is the snapshot's (E487); a hero made playable while cargo ran from the
  -- snapshot is in the repo's set only, and the comparison below builds once more.
  local built_set=sources_as_of and M.script_set(M.snapshot_root(b.root))
  local root=options.output and path.absolute(options.output,b.root) or nil
  if not root then
    local parent=path.mkdir_p(path.join(b.root,'target','split-runs'))
    for suffix=1,1000 do
      local candidate=path.join(parent,os.time()..'-'..suffix)
      if lfs.mkdir(candidate) then root=candidate;break end
    end
  else
    assert(not path.exists(root),'output must be a new directory: '..root);path.mkdir_p(root)
  end
  assert(root,'cannot reserve split run directory')
  -- One live match per machine (E197). Every child joins this lock through its token and
  -- attaches the processes it starts; the lock is released when this run returns or fails.
  local acquired,lock=pcall(require('machine_lock').acquire,b.root,'live_match',{run=root,label='run_moba_split_local',
    wait_seconds=options.wait_live_lock or 0})
  if not acquired then lfs.rmdir(root);error(lock,0) end -- the refused run leaves no directory behind
  local held <close> = lock
  -- E399: a hero made playable while this run waited for the machine adds a script DLL its build
  -- does not have, and the launch refuses it. Built once more, before the time box starts.
  if built_set and M.script_set()~=built_set then
    io.stderr:write('[split] the script DLL set changed while waiting for the machine; building again\n')
    sources_as_of=M.build_all(options)
  end
  -- E343: taken after the live lock and released with it, so the measurement holds it only for
  -- this match; a short wait for another measurement to end counts toward the box.
  local measured <close> = options.performance_lock and M.acquire_performance(root,options) or nil
  -- The time box covers the whole lock hold, startup and shutdown included (E269, E325): play ends
  -- at the retire deadline so --max-seconds 600 frees the machine within ten minutes. The build
  -- happened before the lock (E337).
  local box_deadline,hard_deadline=M.box_deadlines(time.monotonic_ms(),options.max_seconds)
  -- Every role of this match is checked against the sources as they were now (E202).
  local child_env=lock:env()
  child_env.OMOBA_SOURCES_AS_OF=tostring(sources_as_of or os.time())
  -- E516: the roles launch the script DLL set this run built (the snapshot's), not the repo's
  -- as it is when each role starts.
  if sources_as_of then child_env[require('script_dlls').SET_ROOT_ENV]=M.snapshot_root(b.root) end
  local children={}
  local function spawn(label,args)
    local log=path.join(root,label..'.log')
    local _,record=process.spawn_owned(platform.lua_executable,args,{cwd=b.root,stdout=log,stderr=log,env=child_env})
    record=process.validate_owned(record);record.label=label;record.log=log
    lock:attach(record,label)
    if measured then measured:attach(record,label) end
    children[#children+1]=record
    path.write(path.join(root,'children.json'),json.encode(children),true)
    return record,log
  end
  local ok,err=xpcall(function()
    -- The authority prepares and locks the plan that every client then joins.
    local probe_humans=M.humans(require('moba_role_launch').load_recipe({recipe=options.recipe,hero_selections={}}))
    local server_out=path.join(root,'server')
    local server,server_log=spawn('server',M.role_args(options,'server',nil,nil,#probe_humans,server_out))
    wait_for(server,server_log,'Dedicated Rust authority started',options.admit_timeout_ms,'authority',box_deadline)
    local plan_file=path.join(server_out,'match-plan.json')
    local plan=json.read(plan_file)
    local humans=M.humans(plan)
    local teams={}
    for _,player in ipairs(plan.players) do teams[player.player_id]=player.team_id end
    if options.netem then
      -- One human per team: the proxy owns exactly one client endpoint per team route.
      local per_team={}
      for _,id in ipairs(humans) do per_team[teams[id] ]=(per_team[teams[id] ] or 0)+1 end
      assert((per_team[1] or 0)<=1 and (per_team[2] or 0)<=1,'netem split needs at most one human per team')
      local exe=path.join(b.root,'omoba-netem-proxy','target','release','omoba-netem-proxy.exe')
      assert(path.is_file(exe),'missing '..exe..'; run scripts/consistent_build.lua first')
      local evidence=path.mkdir_p(path.join(root,'netem'))
      local _,record=process.spawn_owned(exe,M.netem_args(options,tonumber(options.port or 57061),evidence),
        {cwd=path.parent(exe),stdout=path.join(evidence,'proxy.stdout.log'),stderr=path.join(evidence,'proxy.stderr.log')})
      record=process.validate_owned(record);record.label='netem';record.log=path.join(evidence,'proxy.stdout.log')
      lock:attach(record,'netem')
      children[#children+1]=record
      path.write(path.join(root,'children.json'),json.encode(children),true)
      wait_for(record,record.log,'netem-proxy ready',30000,'netem proxy',box_deadline)
    end
    assert(#humans==#probe_humans,'locked plan changed the human seat count')
    for _,player_id in ipairs(humans) do
      local record,log=spawn('p'..player_id,M.role_args(options,'client',plan_file,player_id,#humans,path.join(root,'p'..player_id),teams[player_id]))
      -- Stagger: the next renderer starts only after this one is admitted (and any shader warmup ended).
      wait_for(record,log,'Humans admitted',options.admit_timeout_ms,'client p'..player_id,box_deadline)
    end
    print(('Split topology running: %d client(s) + dedicated authority; evidence under %s'):format(#humans,root))
    if options.netem then
      -- Outage windows count from here: every client is admitted and playing.
      udp.send(M.NETEM_CONTROL,json.encode({version=1,action='anchor'}))
      for _,o in ipairs(options.netem_outages) do
        print(('netem outage scheduled: team %d at +%d s for %d s'):format(o.team,o.at_s,o.duration_s))
      end
    end
    -- A live check answers within minutes; a time box retires the run deterministically.
    local deadline=box_deadline
    local finished_at
    while true do
      -- With autopilots, a finished match ends the run once every renderer has
      -- recorded the result (its result screenshot follows within seconds).
      if options.autopilot and not finished_at and M.all_finished(root,humans) then
        finished_at=time.monotonic_ms()
        print('match finished on every renderer; retiring after the result captures')
      end
      if finished_at and time.monotonic_ms()-finished_at>=15000 then return end
      if deadline and time.monotonic_ms()>=deadline then
        print(('time box of %d s: play ends %d s early so shutdown and the lock release fit inside it; retiring this split run'):format(options.max_seconds,(hard_deadline-box_deadline)//1000))
        return
      end
      for _,child in ipairs(children) do
        if not process.owned_alive(child) then
          print(child.label..' ended; retiring the rest of this split run')
          return
        end
      end
      time.sleep_ms(1000)
    end
  end,debug.traceback)
  for index=#children,1,-1 do
    local child=children[index]
    if child.label=='netem' and process.owned_alive(child) then M.stop_netem(child) end
    -- Termination is immediate; the wait is bounded by what is left before the hard deadline.
    if process.owned_alive(child) then
      process.stop_owned(child)
      process.wait_owned(child,M.shutdown_wait_ms(time.monotonic_ms(),hard_deadline,15000))
    end
  end
  path.write(path.join(root,'lifecycle.json'),json.encode({schema_version=1,ok=ok,error=ok and nil or tostring(err),children=children}),true)
  assert(ok,err)
  return root
end

if arg and arg[0] and arg[0]:match('run_moba_split_local%.lua$') then
  if arg[1]=='--help' then
    print('Usage: tools/lua/lua.exe scripts/run_moba_split_local.lua [--recipe FILE] [--hero PLAYER_ID=HERO (repeatable)] [--autopilot PROFILE] [--netem PROFILE [--netem-seed N] [--netem-outage TEAM:AT_S:DURATION_S (repeatable)]] [--no-build] [--ue-root PATH] [--port 57061] [--output NEW_DIRECTORY] [--wait-live-lock SECONDS] [--performance-lock]')
    print('--netem routes each team client through omoba-netem-proxy (e.g. wan-80-20-loss1 = 80 ms +/-20 ms RTT, 1% loss); one human per team.')
    print('--netem-outage drops all traffic of one team both ways for DURATION_S seconds, AT_S seconds after every client is admitted (e.g. 1:120:10).')
    print('Runs the same-machine B+C+C topology: one --server-only authority and one --connect client per human seat, with shared host totals and staggered renderer start.')
    return
  end
  process.supervise_workflow(source,arg)
  M.run(M.options(arg))
end
return M
