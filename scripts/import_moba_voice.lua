-- Supervised CC0 voice audio import and independent no-save repeat verification.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local p,j,h,r=b.lib('path'),b.lib('json'),b.lib('hash'),b.lib('process')
local importer=require('voice_audio_import')
local engine,id,validate_only,ensure
local i=1;while i<=#arg do
 if arg[i]=='--ue-root' then i=i+1;engine=assert(arg[i])
 elseif arg[i]=='--run-id' then i=i+1;id=assert(arg[i])
 elseif arg[i]=='--validate-only' then validate_only=true
 elseif arg[i]=='--ensure' then ensure=true
 elseif arg[i]=='--help' then print('tools/lua/lua.exe scripts/import_moba_voice.lua --ue-root PATH --run-id FRESH [--validate-only]');return
 else error('unknown audio import argument: '..tostring(arg[i])) end;i=i+1
end
engine=p.absolute(assert(engine,'--ue-root required'),b.root)
local plan=importer.validate(b.root)
importer.verify_existing_packages(b.root)
if ensure and not id and pcall(importer.require_ready,b.root)then print('CC0 voice imports match source and package SHAs');return end
if ensure and not id then id='ensure-'..tostring(os.time())..'-'..tostring({}):gsub('[^%w]','')end
assert(type(id)=='string' and id:match('^[%w_-]+$'),'fresh --run-id required')
r.supervise_workflow(source,arg)
if validate_only then print('Validated '..#plan.assets..' CC0 voice Ogg sources; no assets imported');return end
local project=p.join(b.root,'omfue/om.uproject')
require('ue_binary_preflight').require_ready(engine,project)
local evidence=p.join(b.root,'.ai-collab/voice-audio-import',id)
assert(not p.exists(evidence),'fresh evidence required');p.mkdir_p(evidence)
local report={schema='omoba.audio-import-summary.v1',run_id=id,success=false,assets=#plan.assets,authority_games=0,playback_verified=false,source_manifest_sha256=plan.manifest_sha256}
local ok,err=xpcall(function()
 local plan_file=p.join(evidence,'plan.json');j.write(plan_file,plan,false)
 local function once(label)
  local output=p.join(evidence,label..'-result.json')
  local args={project,'-run=OmOriginalAudioImport','-OmAudioPlan='..plan_file,'-OmAudioResult='..output,
   '-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding','-abslog='..p.join(evidence,label..'-engine.log')}
  local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),args,{cwd=p.join(b.root,'omfue'),check=false})
  p.write(p.join(evidence,label..'-stdout.log'),result.stdout or '',false,true)
  p.write(p.join(evidence,label..'-stderr.log'),result.stderr or '',false,true)
  assert(result.exit_code==0,label..' commandlet failed: '..tostring(result.exit_code))
  local parsed=j.read(output);importer.verify_result(plan,parsed);return parsed
 end
 report.first=once('first')
 local stamps={}
 for _,asset in ipairs(plan.assets) do
  local file=p.join(b.root,'omfue/Content/Omoba/Audio/VoiceV1',asset.id..'.uasset')
  assert(p.is_file(file),'missing imported SoundWave package')
  stamps[file]={sha256=h.sha256(file),mtime=p.attributes(file).modification}
 end
 report.second=once('second')
 assert(report.second.imported==0 and report.second.saved==0 and report.second.verified==#plan.assets,'repeat did not preserve existing assets')
 for file,stamp in pairs(stamps) do
  assert(h.sha256(file)==stamp.sha256 and p.attributes(file).modification==stamp.mtime,'repeat changed existing audio package: '..file)
 end
 local final=importer.validate(b.root)
 assert(j.encode(final)==j.encode(plan),'audio source changed during import')
 local receipt={schema='omoba.voice-imported-assets/1',manifest_sha256=plan.manifest_sha256,
  producer_sha256=h.sha256(p.join(b.root,importer.producer)),packages={}}
 local decoded={};for _,row in ipairs(report.first.assets)do decoded[row.id]=assert(row.decoded_sha256)end
 for _,asset in ipairs(plan.assets)do local file='omfue/Content/Omoba/Audio/VoiceV1/'..asset.id..'.uasset'
  local wave='omfue/Content/Omoba/Audio/VoiceV1/'..asset.id..'.wav';local sha=h.sha256(p.join(b.root,wave));assert(sha==decoded[asset.id])
  receipt.packages[#receipt.packages+1]={id=asset.id,file=file,sha256=h.sha256(p.join(b.root,file)),decoded_file=wave,decoded_sha256=sha}
 end
 local receiptfile=p.join(b.root,importer.receipt)
 if not p.exists(receiptfile) or j.encode(j.read(receiptfile))~=j.encode(receipt)then j.write(receiptfile,receipt,true)end
 importer.require_ready(b.root)
 report.packages=stamps;report.success=true
end,debug.traceback)
report.error=ok and '' or tostring(err);j.write(p.join(evidence,'summary.json'),report,false)
assert(ok,err);print('Verified '..#plan.assets..' actual SoundWave resources; second import saved zero packages; playback remains pending')
