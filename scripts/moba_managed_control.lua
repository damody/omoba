-- Append-only local control plane. No file supplied by a frontend can name a PID/executable/path.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j=b.lib('path'),b.lib('json')
local policy=require('moba_managed_session')
local M={bootstrap_schema='omoba-managed-match-bootstrap/1',locator_prefix='-om-next-match-control-hex='}
local function hex_path(value)return (value:gsub('.',function(byte)return string.format('%02x',byte:byte())end))end
M.encode_locator=hex_path
local function read_bootstrap(file,plan)
 assert(type(file)=='string' and p.is_absolute(file) and file:match('[^/\\]+$')=='bootstrap.json','invalid managed bootstrap location')
 assert(p.parent(p.absolute(plan.output,b.root)):lower()==p.parent(file):lower(),'managed output and bootstrap must belong to the same new round')
 assert(p.attributes(file).size<=4096,'bootstrap exceeds limit')
 local value=j.read(file);local allowed={schema=true,session_id=true,generation=true}
 for key in pairs(value)do assert(allowed[key],'unknown managed bootstrap field')end
 assert(value.schema==M.bootstrap_schema,'unknown managed bootstrap schema')
 return value
end
function M.install(plan,bootstrap_file)
 assert(plan.mode=='host' and plan.human_count>=1 and plan.human_count<=10
  and plan.local_human_count==plan.human_count and #plan.clients==plan.human_count,'managed provider requires every human to be local')
 local bootstrap=read_bootstrap(bootstrap_file,plan)
 local players={};for _,client in ipairs(plan.clients)do
  players[#players+1]=client.player_id
  assert(client.unreal and type(client.unreal.args)=='table','missing managed Unreal arguments')
  for _,arg in ipairs(client.unreal.args)do assert(type(arg)=='string' and not arg:lower():find('^-om%-next%-match%-control'),'reserved control flag already present')end
 end
 local manifest=policy.manifest({schema=policy.manifest_schema,session_id=bootstrap.session_id,generation=bootstrap.generation,
  catalog_identity_hash=plan.catalog_identity_hash,content_hash=plan.content_hash,map_catalog_hash=plan.map_catalog_hash,players=players})
 local job=assert(os.getenv('_OMOBA_PRIVATE_WORKFLOW_JOB'),'managed control requires a private workflow')
 assert(b.lib('host').call('workflow_member',{job_name=job}).supervised==true,'managed control workflow membership rejected')
 local directory=p.join(p.parent(bootstrap_file),'control')
 assert(not p.exists(directory) and require('lfs').mkdir(directory),'managed control directory must be new')
 local context={manifest=manifest,model=policy.new(manifest),directory=directory,players={},processed={},partial={},published={},exited={}}
 for _,client in ipairs(plan.clients)do
  local id=client.player_id;local folder=p.join(directory,'player-'..id);assert(require('lfs').mkdir(folder))
  local control={player_id=id,directory=folder,capability_file=p.join(folder,'capability.json'),ready_file=p.join(folder,'ready.json'),
   cancel_file=p.join(folder,'cancel.json'),retired_file=p.join(folder,'retired.json'),resolution_file=p.join(folder,'resolution.json'),abort_file=p.join(folder,'abort.json')}
  j.write(control.capability_file,policy.capability(manifest,id,job),false)
  client.unreal.args[#client.unreal.args+1]=M.locator_prefix..hex_path(control.capability_file)
  context.players[#context.players+1]=control
 end
 table.sort(context.players,function(a,c)return a.player_id<c.player_id end)
 j.write(p.join(directory,'manifest.json'),manifest,false)
 plan.managed_session=context
 return context
end
local function message(context,file,now,final)
 if context.processed[file] or not p.is_file(file)then return nil end
 local handle=io.open(file,'rb');local bytes
 if handle then bytes=handle:read(16385);handle:close();assert(not bytes or #bytes<=16384,'managed message exceeds limit')end
 local ok,value=pcall(j.decode,bytes or '')
 if ok then context.partial[file]=nil;return value end
 local first=context.partial[file] or now;context.partial[file]=first
 assert(not final and now>=first and now-first<=1000,'managed message remains incomplete after producer retirement/deadline')
 return nil
end
local function accepted(context,ok,reason)
 if ok then return end
 local phase=context.model.snapshot().phase
 assert((phase=='cancelled' and (reason=='terminal_round' or reason=='round_not_closing'))
  or reason=='duplicate_ready' or reason=='duplicate_retirement','managed message rejected: '..tostring(reason))
end
function M.publish(context)
 local phase=context.model.snapshot().phase
 if phase~='closing' and phase~='cancelled'then return end
 if phase=='closing' and not context.closing_announced then
  -- Nothing legitimately retires in response to a command that was not published yet.
  for _,control in ipairs(context.players)do assert(not p.exists(control.retired_file),'retirement appeared before coordinator close command')end
  context.closing_announced=true
 end
 for _,control in ipairs(context.players)do
  local file=phase=='closing' and control.resolution_file or control.abort_file
  if not context.published[file]then
   assert(not p.exists(file),'managed resolution output already exists')
   j.write(file,assert(context.model.resolution(control.player_id)),false);context.published[file]=true
  end
 end
end
function M.poll(context,now,final_player)
 assert(math.type(now)=='integer' and now>=0,'managed control requires monotonic integer time')
 -- Cancellation dominates readiness, including two writes observed in the same polling batch.
 for _,action in ipairs({'cancel','ready'})do for _,control in ipairs(context.players)do
  local file=action=='cancel' and control.cancel_file or control.ready_file
  local value=message(context,file,now,final_player==true or final_player==control.player_id)
  if value then
   local request=policy.request(value);assert(request.player_id==control.player_id and request.action==action,'managed request disagrees with its slot/file')
   local ok,reason=context.model.accept(request);accepted(context,ok,reason);context.processed[file]=true
  end
 end end
 M.publish(context)
 for _,control in ipairs(context.players)do
  local value=message(context,control.retired_file,now,final_player==true or final_player==control.player_id)
  if value then
   local notice=policy.retired(value);assert(notice.player_id==control.player_id,'retirement disagrees with its slot')
   local ok,reason=context.model.confirm_retirement(notice);accepted(context,ok,reason);context.processed[control.retired_file]=true
  end
 end
 return context.model.snapshot()
end
function M.record_exit(context,player_id,exit_code,now)
 assert(not context.exited[player_id],'duplicate managed process exit')
 -- The caller has already validated this result against its original native Child lease.
 -- Re-read after confirmed OS exit so a just-published retirement notice cannot be missed.
 M.poll(context,now,player_id)
 local ok,reason=context.model.player_exited(player_id,exit_code);accepted(context,ok,reason)
 context.exited[player_id]=true;M.publish(context)
end
function M.finalize(context,workflow_succeeded,cleanup_succeeded,now)
 M.poll(context,now,true)
 if context.model.snapshot().phase=='cancelled'then return nil end
 local ok,reason=context.model.retire_round(workflow_succeeded,cleanup_succeeded)
 if not ok then M.publish(context);return nil,reason end
 M.poll(context,now,true) -- Cancellation still wins after old process cleanup.
 if context.model.snapshot().phase=='cancelled'then return nil end
 local committed,effect=context.model.commit_next_generation()
 if not committed then M.publish(context);return nil,effect end
 return effect
end
return M
