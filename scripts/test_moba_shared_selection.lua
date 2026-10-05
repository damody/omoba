local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local launch,shared=require('moba_role_launch'),require('moba_shared_selection')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','shared-selection-launch-tests'))
local root
for i=1,1000 do local p=path.join(parent,os.time()..'-'..i);if lfs.mkdir(p) then root=p;break end end
assert(root)
local hash='0123456789abcdef'
local baseline=launch.load_recipe(launch.options({}))
baseline.players[2].bot=false
local function copy(value) return json.decode(json.encode(value)) end
local function record(plan)
  local final=launch.select_heroes(plan,{[1]='training_ranger',[2]='training_luminary'})
  local seats={}
  for _,p in ipairs(final.players) do seats[#seats+1]={player=copy(p),locked=true} end
  return {schema_version=1,scope='shared-selection-host-finalized',tick_rate_hz=60,plan=final,
    selection={schema_version=1,catalog_data_hash=hash,revision=5,ready=true,finalized=true,seats=seats}}
end
local total=0
local function test(name,fn) fn();total=total+1;print('PASS '..name) end
local function rejects(fn) assert(not pcall(fn),'expected rejection') end

test('host plan permits only all human hero choices',function()
  local r=record(baseline)
  local before=json.encode(baseline)
  assert(shared.result(r,baseline,hash).players[1].hero=='training_ranger')
  assert(json.encode(baseline)==before)
  for _,mutate in ipairs({
    function(x) x.scope='other' end,
    function(x) x.tick_rate_hz=120 end,
    function(x) x.selection.catalog_data_hash='wrong' end,
    function(x) x.selection.ready=false end,
    function(x) x.plan.players[3].hero='training_ranger' end,
    function(x) x.plan.players[1].team_id=2 end,
    function(x) x.plan.players[1].bot=true end,
    function(x) x.plan.players[2].player_id=x.plan.players[1].player_id end,
    function(x) x.selection.seats[1].locked=false end,
    function(x) x.selection.seats[1].player.hero='different' end,
    function(x) x.plan.think_hz=60 end,
  }) do local changed=copy(r);mutate(changed);rejects(function() shared.result(changed,baseline,hash) end) end
end)

test('terminal receipt accepts stale text but not wrong bound identity',function()
  local receipt={protocol_version=1,shared_room=true,admitted_player_id=2,error='stale revision',
    selection=record(baseline).selection,plan=json.null}
  assert(shared.receipt(receipt,2,hash).finalized)
  rejects(function() shared.receipt(receipt,1,hash) end)
  receipt.shared_room=false
  rejects(function() shared.receipt(receipt,2,hash) end)
end)

local function execute(name,mode)
  local directory=path.join(root,name)
  path.mkdir_p(directory)
  local recipe=path.join(directory,'source.json')
  path.write(recipe,json.encode(baseline))
  local original=path.read(recipe)
  local options=launch.options({'--interactive-selection','--profile','debug','--recipe',recipe,
    '--hero','1=training_vanguard','--output',path.join(directory,'match')})
  local now,host_dir,plan=0,nil,nil
  local peers,states,stopped={},{},{}
  local publish
  local fake={}
  fake.run=function(_,args)
    assert(args[1]=='--lock-plan')
    plan=json.read(args[2])
    assert(plan.players[1].hero=='training_vanguard')
    return {stdout=json.encode({scope='host-prepared-selection',plan=plan,
      selection={catalog_data_hash=hash}})}
  end
  fake.spawn=function(exe,args)
    if args[1]=='--selection-host' then
      host_dir=args[3]
      assert(not path.exists(host_dir))
      path.mkdir_p(host_dir)
      path.write(path.join(host_dir,'ready.json'),json.encode({schema_version=1,
        scope='shared-selection-host-ready',tick_rate_hz=60,address='127.0.0.1:12345',
        catalog_data_hash=mode=='bad-ready' and 'wrong' or hash}))
      for _,id in ipairs({1,2}) do
        path.write(path.join(host_dir,'player-'..id..'.json'),'INVITATION_MUST_NOT_BE_READ_BY_LUA')
      end
      states[101]={alive=true,exe=exe}
      return 101
    end
    local id,log,result,invite
    for _,arg in ipairs(args) do
      assert(not arg:find('om-presentation=',1,true),'gameplay started during selection')
      id=id or tonumber(arg:match('^%-om%-selection%-player=(%d+)$'))
      log=log or arg:match('^%-abslog=(.*)$')
      result=result or arg:match('^%-om%-selection%-result=(.*)$')
      invite=invite or arg:match('^%-om%-selection%-plan=(.*)$')
    end
    assert(id and log and result and invite==path.join(host_dir,'player-'..id..'.json'))
    assert(args[4]=='-om-hero-selection' and exe=='mock-editor.exe')
    path.write(log,('OM_SELECTION_READY player=%d protocol=1 shared_room=%d\n'):format(id,
      mode=='old' and id==2 and 0 or 1))
    peers[id]={result=result,pid=200+id}
    states[200+id]={alive=true,exe=exe}
    return 200+id
  end
  fake.inspect=function(pid) return states[pid] and states[pid].alive and {path=states[pid].exe} or nil end
  fake.stop=function(pid,exe)
    assert(states[pid] and states[pid].exe==exe,'unowned process stop')
    stopped[pid]=true
    if mode=='cleanup-error' and pid==201 then error('injected cleanup failure') end
    states[pid].alive=false
  end
  fake.wait=function(pid)
    assert(states[pid],'wait on unowned process')
    return not states[pid].alive
  end
  local function receipts(r,only)
    for id,peer in pairs(peers) do if not only or id==only then
      if not path.exists(peer.result) and not (mode=='missing-receipt' and id==2) then
        path.write(peer.result,json.encode({protocol_version=1,shared_room=true,
          admitted_player_id=mode=='bad-receipt' and id==2 and 1 or id,
          selection=r.selection,plan=id==1 and r.plan or json.null,
          error=id==2 and 'stale revision' or json.null}))
      end
      states[peer.pid].alive=false
    end end
  end
  publish=function()
    local r=record(plan)
    if mode=='tamper' then r.plan.players[3].hero='training_ranger' end
    receipts(r)
    path.write(path.join(host_dir,'finalized-plan.json'),json.encode(r))
    states[101].alive=false
  end
  local clock={monotonic_ms=function() return now end,sleep_ms=function(ms)
    assert(ms==250);now=now+ms
    if mode=='cancel' or mode=='cleanup-error' then states[202].alive=false
    elseif mode=='host-exit' then states[101].alive=false
    elseif mode=='old' then -- No timeout fallback or synthetic consent.
    elseif mode=='publication-timeout' then receipts(record(plan),2)
    elseif mode=='delayed-publication' and now==250 then receipts(record(plan),2)
    else publish() end
  end}
  local ok,err=pcall(launch.interactive_select,options,'mock-editor.exe',fake,clock)
  assert(path.read(recipe)==original,'source recipe was modified')
  assert(not path.exists(options.output),'gameplay configuration was created during selection')
  if mode=='success' or mode=='delayed-publication' then
    assert(ok,tostring(err))
    for ordinal,pid in ipairs({101,201,202}) do
      local entry=json.read(path.join(options.output..'-selection','owned-process-'..ordinal..'.json'))
      assert(entry.pid==pid and entry.exe==states[pid].exe,'ownership evidence missing')
    end
    assert(next(options.hero_selections)==nil and options.recipe~=recipe)
    assert(json.read(options.recipe).players[1].hero=='training_ranger')
    assert(next(stopped)==nil,'successful exited process was stopped')
  else
    assert(not ok,'failure bypassed')
    local reasons={cancel='selection cancelled', ['host-exit']='host exited without a final artifact',
      old='shared renderer handshake timeout', ['bad-ready']='host readiness mismatch',
      tamper='selection changed host-owned rules', ['bad-receipt']='bound shared-room receipt',
      ['missing-receipt']='terminal receipt', ['publication-timeout']='publication timeout',
      ['cleanup-error']='selection cancelled'}
    assert(tostring(err):find(assert(reasons[mode]),1,true),'unexpected failure: '..tostring(err))
    assert(options.recipe==recipe and options.hero_selections[1]=='training_vanguard')
    local errors=path.read(options.output..'-selection/errors.md')
    assert(#errors>0 and not errors:find('INVITATION_MUST_NOT_BE_READ_BY_LUA',1,true))
    if mode=='cleanup-error' then
      assert(stopped[201] and stopped[101],'cleanup stopped after the first failure')
      assert(errors:find('Cleanup errors:',1,true) and errors:find('injected cleanup failure',1,true))
    end
    for pid,state in pairs(states) do
      assert(not state.alive or (mode=='cleanup-error' and pid==201),'owned process was missed')
    end
  end
end
for _,mode in ipairs({'success','delayed-publication','cancel','host-exit','old','bad-ready',
  'tamper','bad-receipt','missing-receipt','publication-timeout','cleanup-error'}) do
  test('shared launch '..mode,function() execute(mode,mode) end)
end
print(('shared selection launcher: %d/%d passed; mocked processes, no gameplay/Unreal acceptance'):format(total,total))
