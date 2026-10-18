local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap');local p,j,r=b.lib('path'),b.lib('json'),b.lib('process')
assert(#arg==1,'requires a new evidence directory');r.supervise_workflow(source,arg)
local policy,control=require('moba_managed_session'),require('moba_managed_control')
local output=p.absolute(arg[1],b.root);assert(not p.exists(output));p.mkdir_p(output)
local compiled=policy.catalog_scope(j.decode(require('moba_config_command').run({profile='debug'},{'--catalog-scope'},r).stdout))
local assertions=0
local function check(value,message)assert(value,message);assertions=assertions+1 end
local function clone(value)if type(value)~='table'then return value end;local copy={};for key,item in pairs(value)do copy[key]=clone(item)end;return copy end
local function context(name)
 local round=p.mkdir_p(p.join(output,name));local game=p.mkdir_p(p.join(round,'match'))
 local bootstrap=p.join(round,'bootstrap.json')
 j.write(bootstrap,{schema=control.bootstrap_schema,session_id=string.rep('a',64),generation=1},false)
 local plan={mode='host',output=game,human_count=2,local_human_count=2,catalog_identity_hash=compiled.catalog_identity_hash,
  content_hash=compiled.content_hash,map_catalog_hash=compiled.map_catalog_hash,clients={{player_id=17,unreal={args={}}},{player_id=99,unreal={args={}}}}}
 local ctx=control.install(plan,bootstrap)
 for index,slot in ipairs(ctx.players)do
  local capability=j.read(slot.capability_file)
  check(capability.player_id==slot.player_id and capability.workflow_job==os.getenv('_OMOBA_PRIVATE_WORKFLOW_JOB'),'actual private workflow and individual slot bound')
  check(not capability.players and not capability.exe and not capability.ready_file,'capability exposes no other seats or writable paths')
  local locator=plan.clients[index].unreal.args[1]
  check(locator==control.locator_prefix..control.encode_locator(slot.capability_file),'opaque locator preserves native argv without path quoting')
 end
 return ctx,plan
end
local function request(ctx,id,action)
 local value=clone(ctx.manifest);value.players=nil;value.schema=policy.request_schema;value.player_id=id
 value.action=action or 'ready';value.hud_source_epoch=4;value.match_phase=2;return value
end
local function retired(ctx,id)
 local value=request(ctx,id);value.schema=policy.retired_schema;value.action=nil;value.match_phase=nil
 value.inputs_retired=true;value.hud_retired=true;value.runtime_stopped=true;return value
end
local function all_ready(ctx)
 for _,slot in ipairs(ctx.players)do j.write(slot.ready_file,request(ctx,slot.player_id),false)end
 control.poll(ctx,0);check(ctx.model.snapshot().phase=='closing','all players can begin coordinated close')
 for _,slot in ipairs(ctx.players)do local reply=j.read(slot.resolution_file);check(reply.action=='leave_completed_match' and reply.hud_source_epoch==4,'close notice matches ready epoch')end
end
local ok,err=xpcall(function()
 local ctx,plan=context('complete')
 j.write(ctx.players[1].ready_file,request(ctx,17),false);control.poll(ctx,0)
 check(ctx.model.snapshot().phase=='awaiting_ready' and not p.exists(ctx.players[1].resolution_file),'one player cannot force the next round')
 j.write(ctx.players[2].ready_file,request(ctx,99),false);control.poll(ctx,10)
 for _,slot in ipairs(ctx.players)do j.write(slot.retired_file,retired(ctx,slot.player_id),false)end
 control.record_exit(ctx,17,0,20);control.record_exit(ctx,99,0,21)
 local next_round=control.finalize(ctx,true,true,22)
 check(next_round and next_round.kind=='start_selection' and next_round.generation==2,'new selection only after exact completed scope')
 check(not next_round.exe and not next_round.pid and not next_round.receipt,'no command/PID/old receipt in next effect')
 check(not pcall(control.install,plan,p.join(p.parent(plan.output),'bootstrap.json')),'same control scope cannot be installed twice')
 local cancelled=context('cancel-first')
 for _,slot in ipairs(cancelled.players)do j.write(slot.ready_file,request(cancelled,slot.player_id),false)end
 j.write(cancelled.players[1].cancel_file,request(cancelled,17,'cancel'),false)
 control.poll(cancelled,0)
 check(cancelled.model.snapshot().phase=='cancelled' and not p.exists(cancelled.players[1].resolution_file),'cancel wins an entire readiness polling batch')
 check(p.is_file(cancelled.players[2].abort_file),'other participant receives append-only abort')
 local late=context('cancel-late');all_ready(late)
 local prior=p.read(late.players[1].resolution_file,true)
 j.write(late.players[1].cancel_file,request(late,17,'cancel'),false);control.poll(late,100)
 check(p.read(late.players[1].resolution_file,true)==prior and p.is_file(late.players[1].abort_file),'late cancellation does not overwrite an issued close')
 check(not control.finalize(late,true,true,101),'late cancellation cannot allocate a new generation')
 local manual=context('manual-close');all_ready(manual);control.record_exit(manual,17,0,1)
 check(manual.model.snapshot().reason=='player_left' and not control.finalize(manual,true,true,2),'manual exit0 without a retirement notice cancels')
 local partial=context('partial-write');local file=partial.players[1].ready_file
 local full=j.encode(request(partial,17));local writer=assert(io.open(file,'wb'));assert(writer:write(full:sub(1,10)));assert(writer:flush())
 control.poll(partial,0);check(#partial.model.snapshot().ready==0,'partial creation is not consent')
 assert(writer:write(full:sub(11)));assert(writer:close());control.poll(partial,100)
 check(#partial.model.snapshot().ready==1,'complete initial write is accepted without replacement')
 local timeout=context('incomplete-write');p.write(timeout.players[1].ready_file,'{"schema":',false)
 control.poll(timeout,0);check(not pcall(control.poll,timeout,1001),'incomplete message has a bounded deadline')
 local misplaced=context('foreign-slot');j.write(misplaced.players[1].ready_file,request(misplaced,99),false)
 check(not pcall(control.poll,misplaced,0) and #misplaced.model.snapshot().ready==0,'request cannot prepare another slot')
 local early=context('preemptive-retirement')
 for _,slot in ipairs(early.players)do j.write(slot.ready_file,request(early,slot.player_id),false)end
 j.write(early.players[1].retired_file,retired(early,17),false)
 check(not pcall(control.poll,early,0) and not p.exists(early.players[1].resolution_file),'retirement before issued close is rejected before any notice')
 local wrong_epoch=context('wrong-epoch');all_ready(wrong_epoch)
 local bad=retired(wrong_epoch,17);bad.hud_source_epoch=5;j.write(wrong_epoch.players[1].retired_file,bad,false)
 check(not pcall(control.poll,wrong_epoch,1) and #wrong_epoch.model.snapshot().retirement_confirmed==0,'retirement must match the original source epoch')
 local no_cleanup=context('cleanup-failed');all_ready(no_cleanup)
 for _,slot in ipairs(no_cleanup.players)do j.write(slot.retired_file,retired(no_cleanup,slot.player_id),false)end
 control.record_exit(no_cleanup,17,0,1);control.record_exit(no_cleanup,99,0,2)
 check(not control.finalize(no_cleanup,true,false,3),'missing owned process cleanup prevents allocation')
end,debug.traceback)
j.write(p.join(output,'summary.json'),{success=ok,assertions=assertions,authority_games=0,authority_ack=false,
 scope='actual private files and kernel job; synthetic participant requests/retirement, no GUI or gameplay',error=ok and '' or err},false)
if not ok then p.write(p.join(output,'errors.md'),'# Managed control failed\n\n'..tostring(err)..'\n\nDecision: keep exact scope files; never allocate a new match from partial or unverified retirement.\n',false);error(err,0)end
print('PASS '..assertions..' actual managed-control file assertions; participant outcomes synthetic, no game/GUI')
