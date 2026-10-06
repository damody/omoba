-- Selection-only remote renderer. Invitation authentication remains in the Rust proxy.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local M={}
function M.options(args)
  local values={['--profile']=true,['--ue-root']=true,['--graphics']=true,
    ['--output']=true,['--selection-timeout-seconds']=true,['--selection-cancel-file']=true}
  local forwarded={'--interactive-selection'}
  local invite,player
  local seen={};local i=1
  while i<=#args do
    local key=args[i];assert(not seen[key],'duplicate selection join option')
    seen[key]=true;i=i+1;local value=assert(args[i],'selection join option requires a value')
    if key=='--invite' then invite=value
    elseif key=='--player' then
      player=value:match('^%d+$') and math.tointeger(tonumber(value))
      assert(player and player>0 and player<=4294967295 and tostring(player)==value,'player must be canonical nonzero u32')
    else
      assert(values[key],'unknown selection join option')
      forwarded[#forwarded+1]=key;forwarded[#forwarded+1]=value
    end
    i=i+1
  end
  assert(invite and player,'selection join requires --invite and --player')
  local options=require('moba_role_launch').options(forwarded)
  assert(options.output,'selection join requires a new --output directory')
  options.invite=path.absolute(invite,b.root);options.player=player
  return options
end
function M.invitation(options)
  assert(path.is_file(options.invite),'selection invitation file missing')
  assert(path.attributes(options.invite).size<=4096,'selection invitation exceeds size limit')
  -- Do not propagate parser diagnostics that could include private invitation bytes.
  local ok,value=pcall(json.read,options.invite)
  assert(ok and type(value)=='table','invalid selection invitation JSON')
  assert(value.schema_version==1 and value.scope=='selection-room-link'
    and value.admitted_player_id==options.player and type(value.catalog_data_hash)=='string'
    and value.catalog_data_hash~='' and type(value.token)=='string' and #value.token==64
    and value.token:match('^[0-9a-f]+$'),'selection invitation metadata mismatch')
  local ip=type(value.address)=='string' and value.address:match('^([^:]+):%d+$')
  assert(ip,'invalid selection invitation address')
  local address_ok=pcall(require('moba_selection_placement').ready_address,{selection_bind=ip},value.address)
  assert(address_ok,'invalid selection invitation address')
  -- Never return token to callers or include it in arguments/logs/artifacts.
  return {catalog_data_hash=value.catalog_data_hash,address=value.address}
end
function M.run(options,process,time,dependencies)
  dependencies=dependencies or {}
  local output=path.absolute(options.output,b.root)
  assert(not path.exists(output),'selection join output must be a new directory')
  local invitation=M.invitation(options)
  local exe=require('moba_config_command').resolve(options)
  local editor,ue_root=(dependencies.editor or require('moba_role_launch').editor)(options)
  local verify_frontend=dependencies.verify_frontend or require('moba_frontend_preflight').verify
  verify_frontend(ue_root,process)
  local budget=require('moba_selection_deadline')
  local completion
  path.mkdir_p(output)
  local receipt,log=path.join(output,'finalized-reply.json'),path.join(output,'ue.log')
  local args={path.join(b.root,'omfue/om.uproject'),'/Game/Map/Main','-game',
    '-om-hero-selection','-om-presentation-only','-om-selection-exe='..exe,
    '-om-selection-plan='..options.invite,'-om-selection-player='..options.player,
    '-om-selection-result='..receipt,'-'..options.graphics,'-noraytracing','-windowed',
    '-ResX=1280','-ResY=720','-nosplash','-nop4','-NoLiveCoding','-AllowMultipleInstances',
    '-UserDir='..path.join(output,'ue-user'),'-abslog='..log}
  local owned
  local ok,err=xpcall(function()
    completion=budget.start(options,time)
    local pid,record=process.spawn_owned(editor,args,{cwd=path.join(b.root,'omfue'),
      env={OMB_LUA_CONTENT='0',OMB_LUA_HOT_RELOAD='0'},
      stdout=path.join(output,'renderer.stdout.log'),stderr=path.join(output,'renderer.stderr.log')})
    owned=process.validate_owned(record)
    assert(owned.pid==pid,'selection join spawn identity mismatch')
    path.write(path.join(output,'owned-process.json'),json.encode(owned))
    local readiness=require('moba_selection_readiness').reader(options.player,true)
    local ready=false;local deadline=time.monotonic_ms()+45000
    while true do
      budget.check(completion,time)
      local alive=process.owned_alive(owned)
      if not ready then
        local caught_up
        repeat
          assert(time.monotonic_ms()<deadline,'remote selection renderer handshake timeout')
          ready,caught_up=readiness:poll(log,not alive)
          budget.check(completion,time)
          assert(time.monotonic_ms()<deadline,'remote selection renderer handshake timeout')
        until alive or ready or caught_up
      end
      if not alive then
        assert(ready,'remote selection renderer exited without shared-room handshake')
        assert(path.is_file(receipt),'remote selection cancelled; no terminal receipt')
        local reply_ok,reply=pcall(json.read,receipt)
        assert(reply_ok,'invalid terminal selection receipt JSON')
        require('moba_shared_selection').receipt(reply,options.player,invitation.catalog_data_hash)
        local final=assert(type(reply.plan)=='table' and reply.plan~=json.null and reply.plan,
          'terminal receipt lacks host final plan; rebuild the selection host and proxy')
        require('moba_shared_selection').result({schema_version=1,scope='shared-selection-host-finalized',
          tick_rate_hz=60,selection=reply.selection,plan=final},final,invitation.catalog_data_hash)
        local own_seat=false
        for _,seat in ipairs(reply.selection.seats) do
          if seat.player.player_id==options.player and seat.player.bot==false then own_seat=true end
        end
        assert(own_seat,'host final plan lacks admitted human seat')
        local candidate=path.join(output,'host-final-candidate.json')
        path.write(candidate,json.encode(final))
        local checked=require('moba_config_command').run(options,{'--lock-plan',candidate},process)
        local validation=json.decode(checked.stdout)
        assert(validation.scope=='host-prepared-selection' and validation.selection
          and validation.selection.finalized==true and validation.selection.ready==true
          and validation.selection.catalog_data_hash==invitation.catalog_data_hash,
          'host final plan failed compiled catalog validation')
        assert(json.encode(validation.plan)==json.encode(final),'compiled validation changed host final plan')
        assert(process.wait_owned(owned,budget.wait_ms(completion,time,5000)),'remote original renderer remained')
        budget.check(completion,time)
        local recipe=path.join(output,'match-plan.json')
        path.write(recipe,json.encode(final))
        path.write(path.join(output,'result.json'),json.encode({schema_version=1,scope='selection-client-only',
          player_id=options.player,receipt=receipt,recipe_json=recipe}))
        return
      end
      time.sleep_ms(250)
    end
  end,debug.traceback)
  local cleaned,cleanup_error=pcall(function()
    if owned then
      if process.owned_alive(owned) then process.stop_owned(owned) end
      assert(process.wait_owned(owned,5000),'remote original renderer remained after cleanup')
    end
  end)
  if not ok or not cleaned then
    local message=(ok and '' or tostring(err))..(cleaned and '' or '\nCleanup error: '..tostring(cleanup_error))
    path.write(path.join(output,'errors.md'),'# Remote selection error\n\n'..message..
      '\n\nDecision: retire only the original local renderer. Never start gameplay, auto-lock, stop the remote host or log invitation tokens.\n')
    error(message,0)
  end
  print('Selection completed for player '..options.player..'; this does not start gameplay. Host final recipe saved: '..path.join(output,'match-plan.json'))
end
return M
