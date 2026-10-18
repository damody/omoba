-- Local session control, not gameplay state or an authority acknowledgement.
-- Files/processes belong to the adapter; this policy never accepts executable paths or PIDs.
local M={}
M.manifest_schema='omoba-managed-match-manifest/1'
M.request_schema='omoba-managed-match-request/1'
M.resolution_schema='omoba-managed-match-resolution/1'
M.capability_schema='omoba-managed-match-capability/1'
M.retired_schema='omoba-managed-match-retired/1'
local U32=4294967295
local function integer(value,minimum,maximum)
 return math.type(value)=='integer' and value>=minimum and value<=maximum
end
local function hex(value,length)
 return type(value)=='string' and #value==length and value:match('^[0-9a-f]+$')~=nil
end
local function object(value,fields)
 assert(type(value)=='table' and getmetatable(value)==nil,'expected plain contract object')
 for key in pairs(value)do assert(fields[key],'unknown contract field')end
 for key in pairs(fields)do assert(value[key]~=nil,'missing contract field')end
end
local scope_fields={schema=true,session_id=true,generation=true,catalog_identity_hash=true,content_hash=true,map_catalog_hash=true}
local function fields(extra)
 local result={};for key in pairs(scope_fields)do result[key]=true end
 for _,key in ipairs(extra)do result[key]=true end;return result
end
local manifest_fields=fields({'players'})
local request_fields=fields({'player_id','action','hud_source_epoch','match_phase'})
local retired_fields=fields({'player_id','hud_source_epoch','inputs_retired','hud_retired','runtime_stopped'})
function M.catalog_scope(value)
 object(value,{schema_version=true,scope=true,catalog_identity_hash=true,content_hash=true,map_catalog_hash=true})
 assert(value.schema_version==1 and value.scope=='compiled-moba-catalog','unknown compiled catalog report')
 local result={}
 for _,key in ipairs({'catalog_identity_hash','content_hash','map_catalog_hash'})do assert(hex(value[key],16),'invalid compiled catalog hash');result[key]=value[key]end
 return result
end
local function copy_players(players)
 local result={};for i,id in ipairs(players)do result[i]=id end;return result
end
local function scope(value,schema)
 assert(value.schema==schema,'unknown contract schema')
 assert(hex(value.session_id,64),'invalid session identity')
 assert(integer(value.generation,1,U32),'invalid generation')
 for _,key in ipairs({'catalog_identity_hash','content_hash','map_catalog_hash'})do assert(hex(value[key],16),'invalid catalog scope')end
 return {schema=schema,session_id=value.session_id,generation=value.generation,
  catalog_identity_hash=value.catalog_identity_hash,content_hash=value.content_hash,map_catalog_hash=value.map_catalog_hash}
end
function M.manifest(value)
 object(value,manifest_fields);local result=scope(value,M.manifest_schema)
 assert(type(value.players)=='table' and getmetatable(value.players)==nil,'invalid players')
 local count=0;for index in pairs(value.players)do assert(integer(index,1,10),'invalid player index');count=count+1 end
 assert(count>=1 and count<=10 and #value.players==count,'players must be a dense 1..10 array')
 local seen={};result.players={}
 for index=1,count do local id=value.players[index];assert(integer(id,1,U32) and not seen[id],'invalid or duplicate player');seen[id]=true;result.players[index]=id end
 table.sort(result.players);return result
end
function M.request(value)
 object(value,request_fields);local result=scope(value,M.request_schema)
 assert(integer(value.player_id,1,U32),'invalid request player')
 assert(value.action=='ready' or value.action=='cancel','invalid session action')
 assert(integer(value.hud_source_epoch,1,U32) and value.match_phase==2,'request requires a disclosed completed source')
 result.player_id=value.player_id;result.action=value.action;result.hud_source_epoch=value.hud_source_epoch;result.match_phase=2
 return result
end
function M.retired(value)
 object(value,retired_fields);local result=scope(value,M.retired_schema)
 assert(integer(value.player_id,1,U32) and integer(value.hud_source_epoch,1,U32),'invalid retired source')
 for _,key in ipairs({'inputs_retired','hud_retired','runtime_stopped'})do assert(value[key]==true,'retirement is incomplete');result[key]=true end
 result.player_id=value.player_id;result.hud_source_epoch=value.hud_source_epoch;return result
end
function M.capability(value,player_id,workflow_job)
 local manifest=M.manifest(value);local found=false
 for _,id in ipairs(manifest.players)do if id==player_id then found=true end end
 assert(found,'capability player is not a member')
 local prefix='Local\\omoba-workflow-'
 assert(type(workflow_job)=='string' and #workflow_job<=160 and workflow_job:sub(1,#prefix)==prefix
  and not workflow_job:find('\0',1,true),'invalid private workflow binding')
 local result=scope(manifest,M.manifest_schema);result.schema=M.capability_schema;result.player_id=player_id
 result.workflow_job=workflow_job
 return result -- All control files are fixed siblings; no executable, PID or writable path is granted.
end
function M.new(value)
 local manifest=M.manifest(value)
 local members={};for _,id in ipairs(manifest.players)do members[id]=true end
 local phase,reason='awaiting_ready',''
 local ready,exits,confirmed={},{},{}
 local api={}
 local function all_ready()
  for _,id in ipairs(manifest.players)do if not ready[id]then return false end end;return true
 end
 local function cancel(code)
  phase='cancelled';reason=code;return {kind='cancel_round',reason=code}
 end
 local function same_scope(request)
  for _,key in ipairs({'session_id','generation','catalog_identity_hash','content_hash','map_catalog_hash'})do
   if request[key]~=manifest[key]then return false end
  end
  return members[request.player_id]==true
 end
 function api.accept(value)
  local ok,request=pcall(M.request,value)
  if not ok then return false,'invalid_request' end
  if not same_scope(request)then return false,'foreign_scope' end
  if phase=='committed' or phase=='cancelled'then return false,'terminal_round' end
  if request.action=='cancel'then return true,cancel('player_cancelled')end
  if phase~='awaiting_ready'then return false,'round_closing' end
  if ready[request.player_id]then return false,'duplicate_ready' end
  ready[request.player_id]=request.hud_source_epoch
  if all_ready()then phase='closing';return true,{kind='close_round',players=copy_players(manifest.players)}end
  return true,{kind='wait_for_players'}
 end
 function api.confirm_retirement(value)
  local ok,notice=pcall(M.retired,value)
  if not ok then return false,'invalid_retirement' end
  if not same_scope(notice)then return false,'foreign_scope' end
  if phase~='closing'then return false,'round_not_closing' end
  if ready[notice.player_id]~=notice.hud_source_epoch then return false,'foreign_source_epoch' end
  if confirmed[notice.player_id]then return false,'duplicate_retirement' end
  confirmed[notice.player_id]=true;return true,{kind='await_process_exit'}
 end
 function api.player_exited(id,exit_code)
  if not members[id] or not integer(exit_code,0,U32)then return false,'invalid_exit' end
  if phase=='committed' or phase=='cancelled'then return false,'terminal_round' end
  if exits[id]~=nil then return false,'duplicate_exit' end
  exits[id]=exit_code
  if phase~='closing' or exit_code~=0 or not confirmed[id] then return true,cancel(exit_code==0 and 'player_left' or 'player_failed')end
  return true,{kind='await_retirement'}
 end
 function api.retire_round(workflow_succeeded,cleanup_succeeded)
  if phase~='closing'then return false,'round_not_closing' end
  local complete=workflow_succeeded==true and cleanup_succeeded==true
  for _,id in ipairs(manifest.players)do complete=complete and exits[id]==0 end
  if not complete then return false,cancel('cleanup_unverified')end
  phase='retired';return true,{kind='check_cancellation'}
 end
 function api.commit_next_generation()
  if phase~='retired'then return false,'round_not_retired' end
  if manifest.generation==U32 then return false,cancel('generation_exhausted')end
  phase='committed'
  return true,{kind='start_selection',session_id=manifest.session_id,generation=manifest.generation+1,players=copy_players(manifest.players)}
 end
 function api.abort()
  if phase=='committed' or phase=='cancelled'then return false,'terminal_round' end
  return true,cancel('coordinator_failed')
 end
 function api.resolution(id)
  if not members[id]then return nil,'foreign_player' end
  if phase~='closing' and phase~='cancelled'then return nil,'no_resolution' end
  local result=scope(manifest,M.manifest_schema);result.schema=M.resolution_schema
  result.player_id=id;result.action=phase=='closing' and 'leave_completed_match' or 'cancel_next_match';result.reason=reason
  result.hud_source_epoch=ready[id] or 0
  return result
 end
 function api.snapshot()
  local prepared,retired,notices={},{},{}
  for _,id in ipairs(manifest.players)do if ready[id]then prepared[#prepared+1]=id end;if exits[id]~=nil then retired[#retired+1]=id end;if confirmed[id]then notices[#notices+1]=id end end
  return {phase=phase,reason=reason,generation=manifest.generation,players=copy_players(manifest.players),ready=prepared,exited=retired,retirement_confirmed=notices}
 end
 return api
end
return M
