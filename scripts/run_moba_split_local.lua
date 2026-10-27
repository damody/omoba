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
      or key=='--spotlight-script' or key=='--autopilot-shot-interval' or key=='--frame-cap' then
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
  else
    add('--connect','127.0.0.1','--recipe',plan_file,'--local-player',tostring(player_id),'--ue-root',options.ue_root)
    -- Same machine, same match: clients load the authority's own base_content copy (E124).
    add('--base-content',path.join(path.parent(plan_file),'artifacts','scripts','base_content.dll'))
  end
  -- Only the authority builds; clients reuse the same artifacts (no mid-run rebuild).
  if options.no_build or role~='server' then add('--no-build') end
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

local function wait_for(record,file,needle,timeout_ms,label)
  local deadline=time.monotonic_ms()+timeout_ms
  while true do
    local text=path.is_file(file) and path.read(file) or ''
    if text:find(needle,1,true) then return end
    assert(not text:find('stack traceback',1,true),label..' failed; see '..file)
    assert(process.owned_alive(record),label..' exited before "'..needle..'"; see '..file)
    assert(time.monotonic_ms()<deadline,label..' timed out waiting for "'..needle..'"')
    time.sleep_ms(1000)
  end
end

function M.run(options)
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
  local children={}
  local function spawn(label,args)
    local log=path.join(root,label..'.log')
    local _,record=process.spawn_owned(platform.lua_executable,args,{cwd=b.root,stdout=log,stderr=log})
    record=process.validate_owned(record);record.label=label;record.log=log
    children[#children+1]=record
    path.write(path.join(root,'children.json'),json.encode(children),true)
    return record,log
  end
  local ok,err=xpcall(function()
    -- The authority prepares and locks the plan that every client then joins.
    local probe_humans=M.humans(require('moba_role_launch').load_recipe({recipe=options.recipe,hero_selections={}}))
    local server_out=path.join(root,'server')
    local server,server_log=spawn('server',M.role_args(options,'server',nil,nil,#probe_humans,server_out))
    wait_for(server,server_log,'Dedicated Rust authority started',options.admit_timeout_ms,'authority')
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
      children[#children+1]=record
      path.write(path.join(root,'children.json'),json.encode(children),true)
      wait_for(record,record.log,'netem-proxy ready',30000,'netem proxy')
    end
    assert(#humans==#probe_humans,'locked plan changed the human seat count')
    for _,player_id in ipairs(humans) do
      local record,log=spawn('p'..player_id,M.role_args(options,'client',plan_file,player_id,#humans,path.join(root,'p'..player_id),teams[player_id]))
      -- Stagger: the next renderer starts only after this one is admitted (and any shader warmup ended).
      wait_for(record,log,'Humans admitted',options.admit_timeout_ms,'client p'..player_id)
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
    local deadline=options.max_seconds and time.monotonic_ms()+options.max_seconds*1000
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
        print(('time box of %d s reached; retiring this split run'):format(options.max_seconds))
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
    if process.owned_alive(child) then process.stop_owned(child);process.wait_owned(child,15000) end
  end
  path.write(path.join(root,'lifecycle.json'),json.encode({schema_version=1,ok=ok,error=ok and nil or tostring(err),children=children}),true)
  assert(ok,err)
  return root
end

if arg and arg[0] and arg[0]:match('run_moba_split_local%.lua$') then
  if arg[1]=='--help' then
    print('Usage: tools/lua/lua.exe scripts/run_moba_split_local.lua [--recipe FILE] [--hero PLAYER_ID=HERO (repeatable)] [--autopilot PROFILE] [--netem PROFILE [--netem-seed N] [--netem-outage TEAM:AT_S:DURATION_S (repeatable)]] [--no-build] [--ue-root PATH] [--port 57061] [--output NEW_DIRECTORY]')
    print('--netem routes each team client through omoba-netem-proxy (e.g. wan-80-20-loss1 = 80 ms +/-20 ms RTT, 1% loss); one human per team.')
    print('--netem-outage drops all traffic of one team both ways for DURATION_S seconds, AT_S seconds after every client is admitted (e.g. 1:120:10).')
    print('Runs the same-machine B+C+C topology: one --server-only authority and one --connect client per human seat, with shared host totals and staggered renderer start.')
    return
  end
  process.supervise_workflow(source,arg)
  M.run(M.options(arg))
end
return M
