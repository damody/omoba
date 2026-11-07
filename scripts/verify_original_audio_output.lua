-- Actual realtime mixer output acceptance; never substitutes source WAVs for recordings.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process')
local engine,id;local i=1
while i<=#arg do
 if arg[i]=='--ue-root' then i=i+1;engine=assert(arg[i])
 elseif arg[i]=='--run-id' then i=i+1;id=assert(arg[i])
 elseif arg[i]=='--help' then print('tools/lua/lua.exe scripts/verify_original_audio_output.lua --ue-root PATH --run-id FRESH');return
 else error('unknown audio output argument: '..tostring(arg[i]))end;i=i+1
end
engine=p.absolute(assert(engine,'--ue-root required'),b.root)
assert(type(id)=='string' and #id<=80 and id:match('^[%w_-]+$'),'fresh bounded --run-id required')
local project=p.join(b.root,'omfue/om.uproject');require('ue_binary_preflight').require_ready(engine,project)
-- Bridge against its stage record, base_content as this run's own copy, Unreal from a run
-- snapshot rather than live omfue (E116, E124, E127).
require('ue_bridge_stage').verify_launch(b.root)
r.supervise_workflow(source,arg)
local staged=require('run_artifacts').stage_base_content(b.root,p.join(b.root,'target','original-audio-output',id))
local snapshot=require('ue_run_snapshot').prepare(b.root,nil,{base_content=staged.dll})
local evidence=p.join(b.root,'.ai-collab/original-audio-output',id)
local output=p.join(b.root,'omfue/Saved/Automation/OriginalAudio',id)
assert(not p.exists(evidence) and not p.exists(output),'fresh audio output evidence required');p.mkdir_p(evidence)
local report={schema='omoba.original-audio-output-summary.v1',run_id=id,success=false,authority_games=0,no_sound_flag=false,physical_speaker_audibility_verified=false}
local ok,err=xpcall(function()
 local args={snapshot.uproject,'-unattended','-nop4','-nosplash','-d3d12','-AudioMixer','-NoLiveCoding',
  -- This owned realtime recorder must render while its unattended window is unfocused.
  -- Engine command-line config is process-local; preserve user focus and persistent settings.
  '-ini:Engine:[Audio]:UnfocusedVolumeMultiplier=1.0',
  '-ini:EditorPerProjectUserSettings:[/Script/UnrealEd.LevelEditorMiscSettings]:bAllowBackgroundAudio=True',
  '-ini:EditorPerProjectUserSettings:[/Script/UnrealEd.LevelEditorMiscSettings]:EditorVolumeLevel=1.0',
  '-om-original-audio-output='..id,'-ExecCmds=Automation RunTests Om.Runtime.OriginalAudioOutput',
  '-TestExit=Automation Test Queue Empty','-ReportExportPath='..p.join(evidence,'native-report'),'-abslog='..p.join(evidence,'native-engine.log')}
 local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),args,{cwd=snapshot.project_dir,check=false})
 report.process_exit=result.exit_code
 p.write(p.join(evidence,'stdout.log'),result.stdout or '',false,true);p.write(p.join(evidence,'stderr.log'),result.stderr or '',false,true)
 assert(result.exit_code==0,'audio-enabled editor failed: '..tostring(result.exit_code))
 report.native=require('ue_local_acceptance_contract').native_json(assert(p.read(p.join(evidence,'native-report/index.json'),true)),{'Om.Runtime.OriginalAudioOutput'})
 local actual=j.read(p.join(output,'output.json'))
 assert(actual.schema=='omoba.original-audio-output.v1' and actual.realtime_device==true and actual.owned_submix_only==true,'output is not an actual owned realtime mixer recording')
 assert(type(actual.device_name)=='string' and #actual.device_name>0 and actual.physical_speaker_audibility_verified==false,'device evidence or audibility boundary invalid')
 assert(type(actual.component_cap)=='number' and actual.component_cap>=2 and actual.component_cap<=32,'actual voice cap invalid')
 assert(type(actual.recordings)=='table' and #actual.recordings==5,'actual recordings missing')
 local expected={['routed-mix']=true,['master-muted']=false,['ambient-restored']=true,['voice-routed']=true,['voice-muted']=false}
 local captures={}
 for _,row in ipairs(actual.recordings)do
  -- Unreal runs from a snapshot and reports the WAVs through its junctions (E137).
  local file=require('ue_run_snapshot').live_path(b.root,row.file);local name=assert(file:match('([^/\\]+)%.wav$'),'recording WAV name missing')
  assert(expected[name]~=nil and file:gsub('\\','/'):lower()==p.join(output,name..'.wav'):gsub('\\','/'):lower(),'unexpected recording path or duplicate')
  local nonzero=expected[name];expected[name]=nil
  local data=assert(p.read(file,true));assert(#data>=48 and #data<=32*1024*1024,'recorded PCM size invalid')
  assert(data:sub(1,4)=='RIFF' and data:sub(9,16)=='WAVEfmt ' and data:sub(37,40)=='data','recorded PCM layout invalid')
  local riff=string.unpack('<I4',data,5);local fmt,kind,channels,rate,byte_rate,align,bits=string.unpack('<I4I2I2I4I4I2I2',data,17)
  local bytes=string.unpack('<I4',data,41)
  assert(riff+8==#data and fmt==16 and kind==1 and bits==16 and channels>=1 and channels<=8 and rate>=8000 and rate<=192000
    and byte_rate==rate*channels*2 and align==channels*2 and bytes>0 and bytes%align==0 and #data==44+bytes,'recorded PCM header invalid')
  local frames=bytes//align;local peak,sum=0,0
  for at=45,#data,2 do local value=string.unpack('<i2',data,at)/32767;peak=math.max(peak,math.abs(value));sum=sum+value*value end
  local rms=math.sqrt(sum/(bytes/2))
  assert(frames==row.frames and channels==row.channels and rate==row.sample_rate and math.abs(frames/rate-row.seconds)<1/rate,'recorded frame metadata differs from actual PCM')
  assert(row.expected_nonzero==nonzero and type(row.peak)=='number' and row.peak==row.peak and math.abs(peak-row.peak)<0.00007
    and type(row.rms)=='number' and row.rms==row.rms and math.abs(rms-row.rms)<0.00007,'recorded amplitude metadata differs from actual PCM')
  assert(frames/rate>=0.25 and frames/rate<=5,'recorded duration outside scoped test bounds')
  assert(nonzero and peak>=0.0001 or not nonzero and peak<=0.0001,'actual recorded output violates nonzero/muted expectation')
  captures[#captures+1]={file=file,sha256=h.sha256(file),frames=frames,channels=channels,sample_rate=rate,seconds=frames/rate,peak=peak,rms=rms,expected_nonzero=nonzero}
 end
 assert(not next(expected),'recording coverage incomplete')
 report.output_manifest=p.join(output,'output.json');report.device=actual.device_name;report.component_cap=actual.component_cap;report.recordings=captures;report.success=true
end,debug.traceback)
report.error=ok and '' or tostring(err);j.write(p.join(evidence,'summary.json'),report,false)
assert(ok,err);print('Verified five actual realtime mixer PCM: effects/music routed, master muted, ambient restored, voice routed and voice muted; physical speaker audibility not inferred')
