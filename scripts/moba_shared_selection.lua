-- Local/remote seat placement only. Rust owns admission, selection and the final plan.
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local M={}

function M.result(record,baseline,hash)
  assert(type(record)=='table' and record.schema_version==1
    and record.scope=='shared-selection-host-finalized' and record.tick_rate_hz==60,
    'invalid shared selection host result')
  local state=assert(record.selection,'shared selection snapshot missing')
  assert(state.schema_version==1 and state.ready==true and state.finalized==true
    and state.catalog_data_hash==hash,'shared selection was not finalized with matching content')
  local final=assert(record.plan,'shared selection final plan missing')
  assert(type(final.players)=='table' and #final.players==#baseline.players,'selection roster size changed')
  local known,seen,selections={},{},{}
  for _,p in ipairs(baseline.players) do known[p.player_id]=p end
  for _,p in ipairs(final.players) do
    local original=assert(known[p.player_id],'selection added a player')
    assert(not seen[p.player_id],'selection duplicated a player')
    seen[p.player_id]=true
    if original.bot==false then
      assert(type(p.hero)=='string' and p.hero:match('^[%w_]+$'),'invalid selected hero')
      selections[p.player_id]=p.hero
    end
  end
  local expected=require('moba_role_launch').select_heroes(baseline,selections)
  local actual=json.decode(json.encode(final))
  table.sort(expected.players,function(a,c) return a.player_id<c.player_id end)
  table.sort(actual.players,function(a,c) return a.player_id<c.player_id end)
  assert(json.encode(actual)==json.encode(expected),'selection changed host-owned rules, Bot or roster')
  assert(type(state.seats)=='table' and #state.seats==#actual.players,'final selection seats mismatch')
  local seats={}
  for _,seat in ipairs(state.seats) do
    assert(seat.locked==true and type(seat.player)=='table','selection has an unlocked or malformed seat')
    seats[#seats+1]=seat.player
  end
  table.sort(seats,function(a,c) return a.player_id<c.player_id end)
  assert(json.encode(seats)==json.encode(actual.players),'locked roster differs from final plan')
  return final
end

function M.receipt(reply,player,hash)
  assert(type(reply)=='table' and reply.protocol_version==1 and reply.shared_room==true
    and reply.admitted_player_id==player,'renderer lacks bound shared-room receipt')
  local state=assert(reply.selection,'renderer receipt lacks selection state')
  assert(state.schema_version==1 and state.ready==true and state.finalized==true
    and state.catalog_data_hash==hash,'renderer receipt is not terminal')
  -- A stale read may carry error text; the validated finalized snapshot is authoritative.
  return state
end

local function run_room(options,editor,process,time,output,candidate,exe,local_players,preflight)
  local placement=require('moba_selection_placement')
  local owned,renderers={},{}
  local hash=assert(preflight.selection.catalog_data_hash)
  local host_dir=path.join(output,'host')
  local function spawn(executable,args,directory,label)
    local pid,record=process.spawn_owned(executable,args,{cwd=directory,
      env={OMB_LUA_CONTENT='0',OMB_LUA_HOT_RELOAD='0'},
      stdout=path.join(output,label..'.stdout.log'),stderr=path.join(output,label..'.stderr.log')})
    record=process.validate_owned(record)
    assert(record.pid==pid,'shared selection spawn identity mismatch')
    record.label=label
    owned[#owned+1]=record
    -- Immutable per-process records preserve prior ownership evidence on partial failure.
    path.write(path.join(output,'owned-process-'..#owned..'.json'),json.encode(owned[#owned]))
    return record
  end
  local function cleanup()
    local errors={}
    for i=#owned,1,-1 do
      local entry=owned[i]
      local ok,err=pcall(function()
        if process.owned_alive(entry) then process.stop_owned(entry) end
        assert(process.wait_owned(entry,5000),'owned selection process remained: '..entry.label)
      end)
      if not ok then errors[#errors+1]=entry.label..': '..tostring(err) end
    end
    return errors
  end
  local ok,err=xpcall(function()
    local budget=require('moba_selection_deadline')
    local completion_deadline=budget.start(options,time)
    local host=spawn(exe,placement.host_args(options,candidate,host_dir),b.root,'host')
    local deadline=time.monotonic_ms()+45000
    local ready_path=path.join(host_dir,'ready.json')
    while not path.is_file(ready_path) do
      budget.check(completion_deadline,time)
      assert(process.owned_alive(host),'shared selection host exited before readiness')
      assert(time.monotonic_ms()<deadline,'shared selection host readiness timeout')
      time.sleep_ms(250)
    end
    local ready=json.read(ready_path)
    assert(ready.schema_version==1 and ready.scope=='shared-selection-host-ready'
      and ready.tick_rate_hz==60 and ready.catalog_data_hash==hash,'shared host readiness mismatch')
    placement.ready_address(options,ready.address)
    print('Selection invitations available in '..host_dir..'; share only each remote seat\'s invitation file. Never share the whole host folder.')
    for _,id in ipairs(local_players) do
      budget.check(completion_deadline,time)
      local directory=path.join(output,'player-'..id)
      assert(not path.exists(directory),'renderer output already exists')
      path.mkdir_p(directory)
      local invite=path.join(host_dir,'player-'..id..'.json')
      assert(path.is_file(invite),'host invitation missing for admitted player') -- Never read/log token.
      local receipt=path.join(directory,'finalized-reply.json')
      local log=path.join(directory,'ue.log')
      local args={path.join(b.root,'omfue','om.uproject'),'/Game/Map/Main','-game',
        '-om-hero-selection','-om-presentation-only','-om-selection-exe='..exe,
        '-om-selection-plan='..invite,'-om-selection-player='..id,'-om-selection-result='..receipt,
        '-'..options.graphics,'-noraytracing','-windowed','-ResX=1280','-ResY=720',
        '-nosplash','-nop4','-NoLiveCoding','-AllowMultipleInstances',
        '-UserDir='..path.join(directory,'ue-user'),'-abslog='..log}
      if options.selection_smoke_hero then args[#args+1]='-om-selection-smoke-hero='..options.selection_smoke_hero end
      renderers[#renderers+1]={id=id,owned=spawn(editor,args,path.join(b.root,'omfue'),'player-'..id),
        receipt=receipt,log=log,deadline=time.monotonic_ms()+45000,
        readiness=require('moba_selection_readiness').reader(id,true)}
    end
    local function read_ready(renderer,final)
      if renderer.ready then return true end
      local caught_up
      repeat
        budget.check(completion_deadline,time)
        assert(time.monotonic_ms()<renderer.deadline,'shared renderer handshake timeout')
        renderer.ready,caught_up=renderer.readiness:poll(renderer.log,final)
        budget.check(completion_deadline,time)
        assert(time.monotonic_ms()<renderer.deadline,'shared renderer handshake timeout')
      until not final or renderer.ready or caught_up
      return renderer.ready
    end
    local final_path=path.join(host_dir,'finalized-plan.json')
    local publication_deadline
    while not path.is_file(final_path) do
      budget.check(completion_deadline,time)
      for _,renderer in ipairs(renderers) do
        local alive=process.owned_alive(renderer.owned)
        read_ready(renderer,not alive)
        assert(renderer.ready or time.monotonic_ms()<renderer.deadline,'shared renderer handshake timeout')
        if not alive then
          assert(renderer.ready,'renderer exited without shared-room handshake')
          assert(path.is_file(renderer.receipt),'selection cancelled; no match started')
          M.receipt(json.read(renderer.receipt),renderer.id,hash)
          publication_deadline=publication_deadline or time.monotonic_ms()+5000
        end
      end
      assert(path.is_file(final_path) or process.owned_alive(host),'shared host exited without a final artifact')
      assert(not publication_deadline or time.monotonic_ms()<publication_deadline,'host final artifact publication timeout')
      time.sleep_ms(250)
    end
    local record=json.read(final_path)
    budget.check(completion_deadline,time)
    local final=M.result(record,preflight.plan,hash)
    -- Do not start gameplay while any selection renderer/host still owns the session.
    for _,renderer in ipairs(renderers) do
      assert(process.wait_owned(renderer.owned,budget.wait_ms(completion_deadline,time,15000)),
        'shared renderer did not exit after finalization')
      assert(read_ready(renderer,true),
        'renderer lacks shared-room handshake')
      assert(path.is_file(renderer.receipt),'renderer exited without terminal receipt; selection cancelled')
      local state=M.receipt(json.read(renderer.receipt),renderer.id,hash)
      assert(json.encode(state)==json.encode(record.selection),'renderer terminal state differs from host')
    end
    assert(process.wait_owned(host,budget.wait_ms(completion_deadline,time,5000)),'selection host did not retire')
    budget.check(completion_deadline,time)
    local selected=path.join(output,'match-plan.json')
    assert(not path.exists(selected),'selected match plan already exists')
    path.write(selected,json.encode(final))
    options.recipe=selected
    options.hero_selections={}
  end,debug.traceback)
  if not ok then
    local errors=cleanup()
    local message=tostring(err)..(#errors>0 and '\n\nCleanup errors:\n'..table.concat(errors,'\n') or '')
    path.write(path.join(output,'errors.md'),'# Shared selection handoff error\n\n'..message..
      '\n\nDecision: no gameplay launch or automatic locking; retain only this session artifacts.\n')
    error(message,0)
  end
end
function M.run(options,editor,process,time,output,candidate,exe,humans,preflight)
  local local_players=require('moba_selection_placement').local_players(options,humans)
  return run_room(options,editor,process,time,output,candidate,exe,local_players,preflight)
end
function M.run_headless(options,process,time,output,candidate,exe,humans,preflight)
  require('moba_network_launch').ipv4(assert(options.selection_bind,'headless selection requires explicit bind'))
  assert(#humans>=1 and #humans<=10,'headless selection requires 1..10 human seats')
  assert(not options.selection_smoke_hero and next(options.local_players or {})==nil,
    'headless selection cannot spawn a local renderer')
  return run_room(options,nil,process,time,output,candidate,exe,{},preflight)
end
return M
