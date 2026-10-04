-- Read-only verification of saved dual Unreal upgrade evidence.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json,process,hash=b.lib('path'),b.lib('json'),b.lib('process'),b.lib('hash')
local observe=require('ue_two_team_observation')
local id=assert(arg[1],'Usage: tools/lua/lua.exe scripts/verify_ue_upgrade_run.lua RUN_ID')
assert(#arg==1 and id:match('^interactive%-ue%-%d+$'),'invalid run ID')
local root=path.join(b.root,'target/interactive-runs',id)
local report={success=false,run_id=id,teams={},evidence_hashes={}}
local function read(relative)
 local file=path.join(root,relative);report.evidence_hashes[relative]=hash.sha256(file);return path.read(file,true)
end
local ok,failure=xpcall(function()
 local launcher=json.decode(read('unreal-ipc-smoke-report.json'))
 assert(launcher.success and launcher.cleanup_verified and launcher.upgrade_smoke and launcher.tick_rate_hz==60)
 local cast_mode=launcher.first_learning_cast~=nil
 local learn_slot=cast_mode and 3 or 0
 assert(launcher.upgrade_input_route==(cast_mode and 'unreal-bound-CtrlR-delegate' or 'unreal-bound-CtrlQ-delegate'))
 local first_learning=launcher.first_learning~=nil
 if first_learning then
  assert(launcher.first_learning.contract==1 and launcher.first_learning.hero=='training_apprentice')
  assert(launcher.first_learning.initial_rank==0 and launcher.first_learning.slot==learn_slot)
  assert(launcher.first_learning.input_route==launcher.upgrade_input_route and launcher.first_learning.injected_gameplay_state==false)
  report.first_learning=launcher.first_learning
 end
 if cast_mode then
  assert(first_learning and launcher.first_learning_cast.contract==1 and launcher.first_learning_cast.slot==3
    and launcher.first_learning_cast.input_route=='unreal-bound-R-delegate' and launcher.first_learning_cast.injected_gameplay_state==false)
  report.first_learning_cast=launcher.first_learning_cast
 end
 if launcher.graphics_rhi then
  assert(launcher.graphics_rhi=='d3d11' or launcher.graphics_rhi=='d3d12','invalid recorded RHI')
  report.graphics_rhi=launcher.graphics_rhi
 end
 assert(read('server-game.toml'):match('STEP_FPS%s*=%s*60'))
 local diagnostic=read('server/three-way-checkpoints.jsonl')
 report.parity=observe.observe_parity(diagnostic,json.decode)
 local checkpoints={{},{}}
 for line in diagnostic:gmatch('([^\n]+)\n') do
  local row=json.decode(line)
  assert(row.verdict~='FAIL','failed checkpoint')
  if type(row.expected)=='string' and type(row.external_runtime_pre_repair)=='string' then
   assert(row.team_id==1 or row.team_id==2)
   assert(row.verdict=='PASS' and row.pre_repair_parity==true and row.post_repair_parity==true)
   assert(row.external_runtime_frame_hash==row.observer_frame_hash)
   checkpoints[row.team_id][row.replica_tick]=true
  end
 end
 report.unique_three_way={}
 report.processes={}
 for _,role in ipairs({'server','runtime-p1','runtime-p2','ue-p1','ue-p2'}) do
  local pid=assert(math.tointeger(tonumber(read(role..'.pid'))))
  report.processes[#report.processes+1]=require('ue_saved_process_identity').check(process.inspect(pid),pid,role,launcher.processes)
 end
 report.cleanup_verified=true
 for team=1,2 do
  local unreal_log=read('logs/ue-p'..team..'.stdout.log')
  if report.graphics_rhi then assert(unreal_log:find('Using Forced RHI: '..report.graphics_rhi:upper(),1,true),'requested RHI was not used') end
  if unreal_log:find('OM_UI_BACKBUFFER',1,true) then
   local captures={}
   for saved,width,height,filename in unreal_log:gmatch('OM_UI_BACKBUFFER saved=(%d+) width=(%d+) height=(%d+) path=([^\r\n]+)') do
    assert(saved=='1' and tonumber(width)==1280 and tonumber(height)==720,'backbuffer capture failed or changed extent')
    for _,stage in ipairs({'before','after'}) do
     if path.absolute(filename)==path.absolute(path.join(root,'upgrade-ui','team-'..team..'-'..stage..'.png')) then
      assert(not captures[stage],'duplicate backbuffer capture');captures[stage]=true
     end
    end
   end
   assert(captures.before and captures.after,'missing presented UI capture')
   report.capture_route='presented-backbuffer'
  end
  local upgrade=observe.observe_upgrade(unreal_log,team,first_learning,learn_slot)
  assert(upgrade.complete)
  local cast=cast_mode and observe.observe_first_learn_cast(unreal_log,team) or nil
  local settled_tick=upgrade.after.tick
  if cast then
   assert(cast.complete and cast.before.tick>=upgrade.after.tick and cast.before.input_id~=upgrade.before.input_id)
   settled_tick=cast.after.tick
   report.teams[team]={upgrade=upgrade,cast=cast}
  end
  local layouts={before={},after={}}
  local layout_count=0
  for stage,player,slot,name,width,height,desired_width,desired_height in unreal_log:gmatch(
   'OM_UPGRADE_LAYOUT (%a+) player=(%d+) slot=(%d+) name=(%S+) width=([%d%.]+) height=([%d%.]+) desired_width=([%d%.]+) desired_height=([%d%.]+)') do
   if tonumber(player)==team then
    assert(layouts[stage] and tonumber(slot)<4,'invalid layout diagnostic')
    assert(not layouts[stage][slot],'duplicate layout diagnostic')
    assert(tonumber(width)>0 and tonumber(height)>0 and tonumber(desired_width)>0 and tonumber(desired_height)>0)
    assert(tonumber(width)+0.1>=tonumber(desired_width) and tonumber(height)+0.1>=tonumber(desired_height),'undersized ability name layout')
    layouts[stage][slot]={name=name,width=tonumber(width),height=tonumber(height)}
    layout_count=layout_count+1
   end
  end
  if unreal_log:find('OM_UPGRADE_LAYOUT',1,true) then
   assert(layout_count==8,'expected all four slots before and after')
   for slot=0,3 do assert(layouts.before[tostring(slot)].name==layouts.after[tostring(slot)].name,'upgrade changed ability name') end
   upgrade.layout_verified=true
   upgrade.layouts=layouts
  end
  local runtime=read('logs/runtime-p'..team..'.stderr.log')
  assert(not runtime:find('upgrade smoke',1,true),'runtime injected upgrade')
  assert(not runtime:find('first learn smoke',1,true),'runtime injected first learning')
  local count=0
  for line in runtime:gmatch('[^\r\n]+') do
   if line:find('input forwarded player='..team..' input_id='..upgrade.before.input_id..' ',1,true) then count=count+1 end
  end
  assert(count==1,'original upgrade forwarded exactly once')
  if cast then
   local forwarded=0
   for line in runtime:gmatch('[^\r\n]+') do
    if line:find('input forwarded player='..team..' input_id='..cast.before.input_id..' ',1,true) then forwarded=forwarded+1 end
   end
   assert(forwarded==1,'original R cast forwarded exactly once')
  end
  assert(report.parity[team].passed>=6 and report.parity[team].last_tick>=upgrade.after.tick+120)
  local count,last_tick,after=0,0,0
  for tick in pairs(checkpoints[team]) do
   count=count+1;last_tick=math.max(last_tick,tick)
   if tick>settled_tick then after=after+1 end
  end
  assert(count>=6 and after>=2 and last_tick>=settled_tick+120,'insufficient unique post-input three-way checkpoints')
  report.unique_three_way[team]={count=count,last_tick=last_tick,post_upgrade=after}
  if cast then report.unique_three_way[team].post_cast=after end
  for _,name in ipairs({'team-frame.capture','presentation.capture'}) do
   local relative='team-'..team..'-runtime/'..name
   report.evidence_hashes[relative]=hash.sha256(path.join(root,relative))
  end
  for _,stage in ipairs({'before','after'}) do assert(read('upgrade-ui/team-'..team..'-'..stage..'.png'):sub(1,8)=='\137PNG\r\n\26\n') end
  if not cast then report.teams[team]=upgrade end
 end
 local result=process.run('cargo',{'test','--manifest-path',path.join(b.root,'omoba-client-runtime/Cargo.toml'),
  'real_unreal_ability_upgrade_capture','--','--ignored','--nocapture'},
  {cwd=b.root,env={OMOBA_UE_UPGRADE_CAPTURE_ROOT=root},check=false})
 assert(result.exit_code==0,'raw capture verification failed: '..result.stdout..result.stderr)
 report.capture_validation={}
 for team,count,input in result.stdout:gmatch('real Unreal upgrade team=(%d+) snapshots=(%d+) input=(%d+)') do
  report.capture_validation[#report.capture_validation+1]={team_id=tonumber(team),snapshots=tonumber(count),input_id=tonumber(input)}
 end
 assert(#report.capture_validation==2,'expected two nonempty raw capture checks')
 if cast_mode then
  report.cast_validation={}
  for team,input,tick,before,after,heal in result.stdout:gmatch('real Unreal learned R team=(%d+) cast_input=(%d+) cast_tick=(%d+) hp_before_raw=(%d+) hp_after_raw=(%d+) lua_heal_raw=(%d+)') do
   report.cast_validation[#report.cast_validation+1]={team_id=tonumber(team),input_id=tonumber(input),tick=tonumber(tick),
     hp_before_raw=tonumber(before),hp_after_raw=tonumber(after),lua_heal_raw=tonumber(heal)}
  end
  assert(#report.cast_validation==2,'expected two exact Lua heal settlements')
 end
 for relative,expected in pairs(report.evidence_hashes) do
  assert(hash.sha256(path.join(root,relative))==expected,'evidence changed during verification: '..relative)
 end
 report.success=true
end,debug.traceback)
if not ok then report.error=tostring(failure) end
local out=path.join(root,'unreal-upgrade-verification-report.json')
json.write(out,report,true)
print('[ue-upgrade-verification] success='..tostring(report.success)..'; report: '..out)
assert(ok,failure)
