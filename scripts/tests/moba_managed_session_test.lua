local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local session=require('moba_managed_session')
local assertions=0
local function check(value,message)assert(value,message);assertions=assertions+1 end
local function clone(value)if type(value)~='table'then return value end;local result={};for k,v in pairs(value)do result[k]=clone(v)end;return result end
local function manifest(players,generation)
 return {schema=session.manifest_schema,session_id=string.rep('a',64),generation=generation or 1,
  catalog_identity_hash=string.rep('b',16),content_hash=string.rep('c',16),map_catalog_hash=string.rep('d',16),players=players or {7,99}}
end
local function request(id,action,identity)
 local result=clone(identity or manifest());result.players=nil;result.schema=session.request_schema
 result.player_id=id;result.action=action or 'ready';result.hud_source_epoch=4;result.match_phase=2;return result
end
local function ready_model(players)
 local identity=manifest(players);local model=session.new(identity)
 for _,id in ipairs(players)do check(model.accept(request(id,'ready',identity)),'valid readiness')end
 return model,identity
end
local function notice(id,identity)
 local result=clone(identity or manifest());result.players=nil;result.schema=session.retired_schema
 result.player_id=id;result.hud_source_epoch=4;result.inputs_retired=true;result.hud_retired=true;result.runtime_stopped=true;return result
end
local function retire(model,players)
 local identity=manifest(players,model.snapshot().generation)
 for _,id in ipairs(players)do
  check(model.confirm_retirement(notice(id,identity)),'completed actual presentation retirement required')
  check(model.player_exited(id,0),'expected clean player exit')
 end
 check(model.retire_round(true,true),'all owned processes and workflow retired')
end
local identity=manifest();local model=session.new(identity)
identity.players[1]=123;check(model.snapshot().players[1]==7,'manifest input must not mutate active membership')
local snapshot=model.snapshot();snapshot.players[1]=456;check(model.snapshot().players[1]==7,'snapshot must not mutate active membership')
check(not model.commit_next_generation(),'cannot start selection before readiness')
local ok,effect=model.accept(request(7));check(ok and effect.kind=='wait_for_players','one player cannot decide for all')
check(not model.accept(request(7)),'repeat readiness rejected');check(#model.snapshot().ready==1,'repeat leaves exact count')
ok,effect=model.accept(request(99));check(ok and effect.kind=='close_round','all actual members authorize closing')
check(not model.commit_next_generation(),'closing does not allocate new authority')
local reply=model.resolution(99);check(reply.schema==session.resolution_schema and reply.player_id==99 and reply.action=='leave_completed_match','scoped close notification')
check(not model.resolution(13),'no notification for foreign player')
retire(model,{7,99});ok,effect=model.commit_next_generation();check(ok and effect.kind=='start_selection' and effect.generation==2,'fresh selection follows verified retirement')
check(not effect.recipe and not effect.receipt and not effect.exe and not effect.pid,'effect cannot reuse a receipt or launch arbitrary executable')
check(not model.commit_next_generation(),'new generation emitted once');check(not model.accept(request(7,'cancel')),'committed old round is terminal')
for count=1,10 do
 local players={};for index=1,count do players[index]=index*17 end
 local next_model,next_identity=ready_model(players)
 check(next_model.snapshot().phase=='closing','1..10 humans use the same state machine')
 retire(next_model,players);local accepted,next_effect=next_model.commit_next_generation()
 check(accepted and #next_effect.players==count and next_effect.players[count]==count*17,'all declared identities retained')
 next_identity.generation=2;local new_round=session.new(next_identity)
 check(not new_round.accept(request(players[1],'ready',manifest(players,1))),'previous generation request cannot prepare the new round')
end
for _,at in ipairs({'waiting','closing','retired'})do
 local cancelled=at=='waiting' and session.new(manifest()) or ready_model({7,99})
 if at=='retired'then retire(cancelled,{7,99})end
 check(cancelled.accept(request(7,'cancel')),'cancel is accepted until commit')
 check(cancelled.snapshot().phase=='cancelled' and not cancelled.commit_next_generation(),'cancel wins even after old cleanup')
 check(cancelled.resolution(99).action=='cancel_next_match','other participant receives cancellation')
end
for _,failure in ipairs({'early_exit','crash','cleanup','role_failure','missing_exit'})do
 local failed=failure=='early_exit' and session.new(manifest()) or ready_model({7,99})
 if failure=='early_exit'then failed.player_exited(7,0)
 elseif failure=='crash'then failed.player_exited(7,3)
 else failed.confirm_retirement(notice(7));failed.player_exited(7,0);if failure~='missing_exit'then failed.confirm_retirement(notice(99));failed.player_exited(99,0)end;failed.retire_round(failure~='role_failure',failure~='cleanup')end
 check(failed.snapshot().phase=='cancelled' and not failed.commit_next_generation(),'failure cannot restart: '..failure)
end
local invalid_cases={
 function(r)r.session_id=string.rep('e',64)end,function(r)r.generation=2 end,
 function(r)r.player_id=42 end,function(r)r.catalog_identity_hash=string.rep('e',16)end,
 function(r)r.content_hash=string.rep('e',16)end,function(r)r.map_catalog_hash=string.rep('e',16)end,
 function(r)r.schema='unknown/2'end,function(r)r.exe='untrusted'end,function(r)r.hud_source_epoch=0 end,
 function(r)r.match_phase=1 end,function(r)r.action='restart'end,function(r)r.player_id=-1 end,
 function(r)r.hud_source_epoch=1.5 end,function(r)r.generation=4294967296 end,
}
for _,mutate in ipairs(invalid_cases)do
 local untouched=session.new(manifest());local bad=request(7);mutate(bad)
 check(not untouched.accept(bad),'malformed or foreign request rejected')
 check(untouched.snapshot().phase=='awaiting_ready' and #untouched.snapshot().ready==0,'rejection cannot prepare/close/allocate')
end
for _,players in ipairs({{}, {7,7}, {0}, {-1}, {4294967296}, {1,2,3,4,5,6,7,8,9,10,11}, {[1]=7,[3]=99}})do
 check(not pcall(session.new,manifest(players)),'invalid membership rejected')
end
local saturated_identity=manifest({7},4294967295);local saturated=session.new(saturated_identity)
check(saturated.accept(request(7,'ready',saturated_identity)),'final valid generation can finish normally');retire(saturated,{7})
check(not saturated.commit_next_generation() and saturated.snapshot().reason=='generation_exhausted','generation never wraps')
local exited=ready_model({7,99});check(exited.confirm_retirement(notice(7)),'first retirement accepted');check(not exited.confirm_retirement(notice(7)),'duplicate retirement rejected')
check(exited.player_exited(7,0),'first exit accepted');check(not exited.player_exited(7,0),'duplicate exit cannot count twice')
check(not exited.player_exited(13,0),'foreign process report cannot authorize cleanup')
local aborted=session.new(manifest());check(aborted.abort(),'coordinator error cancels');check(not aborted.accept(request(7)),'error does not silently resume')
local native_crash=ready_model({7,99})
check(native_crash.player_exited(7,0xC0000005),'full Windows NTSTATUS is a valid process result')
check(native_crash.snapshot().phase=='cancelled' and not native_crash.commit_next_generation(),'Windows access violation cannot launch the next match')
local job='Local\\omoba-workflow-policy-fixture'
local capability=session.capability(manifest(),99,job)
check(capability.schema==session.capability_schema and capability.player_id==99 and not capability.players,'capability grants one declared player')
check(not capability.exe and not capability.pid and not capability.ready_file,'capability grants no executable, PID or arbitrary file path')
check(capability.workflow_job==job,'capability is bound to the declared private workflow')
check(not pcall(session.capability,manifest(),13,job),'foreign player gets no capability')
check(not pcall(session.capability,manifest(),7,''),'capability without a private workflow is rejected')
local unverified=ready_model({7,99});unverified.confirm_retirement(notice(7));unverified.confirm_retirement(notice(99));unverified.player_exited(7,0);unverified.player_exited(99,0)
check(not unverified.retire_round(0,true),'workflow completion is an explicit boolean, not an invented process exit code')
local compiled={schema_version=1,scope='compiled-moba-catalog',catalog_identity_hash=string.rep('b',16),content_hash=string.rep('c',16),map_catalog_hash=string.rep('d',16)}
check(session.catalog_scope(compiled).content_hash==compiled.content_hash,'actual compiled catalog scope has no synthetic session identity')
for _,mutate in ipairs({function(v)v.schema_version=2 end,function(v)v.content_hash=string.rep('c',64)end,function(v)v.exe='untrusted'end})do
 local invalid=clone(compiled);mutate(invalid);check(not pcall(session.catalog_scope,invalid),'invalid compiled metadata rejected before player selection')
end
local closed_manually=ready_model({7,99});closed_manually.player_exited(7,0)
check(closed_manually.snapshot().reason=='player_left' and not closed_manually.commit_next_generation(),'all-ready plus a manual exit0 is not coordinated retirement')
for _,mutate in ipairs({function(v)v.hud_source_epoch=5 end,function(v)v.inputs_retired=false end,
 function(v)v.runtime_stopped=false end,function(v)v.generation=2 end,function(v)v.player_id=13 end,
 function(v)v.exe='untrusted'end})do
 local current=ready_model({7,99});local invalid=notice(7);mutate(invalid)
 check(not current.confirm_retirement(invalid),'foreign or incomplete retirement cannot authorize new selection')
 check(#current.snapshot().retirement_confirmed==0,'failed notice leaves no positive cleanup proof')
end
print('PASS '..assertions..' managed-session assertions; no filesystem, process launch, authority game or ACK')
