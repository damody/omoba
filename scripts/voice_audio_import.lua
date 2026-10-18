-- CC0 voice recordings use a separate fixed namespace; no external decoder fallback.
local p=require('tools.lua.lib.path')
local j=require('tools.lua.lib.json')
local h=require('tools.lua.lib.hash')
local M={}
function M.ogg(data)
 assert(type(data)=='string' and #data>=64 and #data<=2*1024*1024,'voice Ogg size invalid')
 local offset,sequence,serial,last,ended=1,0,nil,0,false
 local channels,rate
 while offset<=#data do
  assert(not ended and offset+26<=#data and data:sub(offset,offset+3)=='OggS' and data:byte(offset+4)==0,'Ogg page invalid')
  local flags=data:byte(offset+5);local granule,page_serial,page_sequence=string.unpack('<I8I4I4',data,offset+6)
  assert(flags<=7 and page_sequence==sequence and sequence<4096,'Ogg sequence invalid')
  assert(not serial or serial==page_serial,'chained voice streams unsupported');serial=page_serial
  local segments=data:byte(offset+26);local header_end=offset+26+segments
  assert(header_end<=#data,'Ogg segment table truncated')
  local bytes=0;for n=offset+27,header_end do bytes=bytes+data:byte(n)end
  assert(header_end+bytes<=#data,'Ogg payload truncated')
  if sequence==0 then
   assert(flags==2 and bytes>=30 and data:sub(header_end+1,header_end+7)=='\1vorbis','voice Vorbis identification missing')
   local version;version,channels,rate=string.unpack('<I4BI4',data,header_end+8)
   assert(version==0 and (channels==1 or channels==2) and rate>=8000 and rate<=96000,'voice Vorbis format invalid')
  end
  if granule~=-1 then assert(granule>=last and granule<=rate*10,'voice duration or granule invalid');last=granule end
  ended=(flags&4)~=0;offset=header_end+bytes+1;sequence=sequence+1
 end
 assert(ended and last>0,'voice EOS or sample count missing')
 return {channels=channels,rate=rate,frames=last,seconds=last/rate,bytes=#data}
end
function M.validate(root,manifest_path)
 local dir=p.join(root,'omfue/Content/Omoba/Audio/VoiceV1')
 manifest_path=manifest_path or p.join(dir,'provenance-v1.json')
 local manifest=j.read(manifest_path)
 assert(manifest.schema=='omoba.cc0-voice-source/1' and manifest.license=='CC0-1.0','voice provenance invalid')
 assert(manifest.included_license_file=='LICENSE.txt' and type(manifest.included_license_sha256)=='string'
  and h.sha256(p.join(dir,'LICENSE.txt'))==manifest.included_license_sha256,'voice included license missing or changed')
 assert(type(manifest.assets)=='table' and #manifest.assets>=1 and #manifest.assets<=64,'voice source count invalid')
 local out={schema='omoba.voice-import-plan.v1',manifest=p.absolute(manifest_path),manifest_sha256=h.sha256(manifest_path),assets={}}
 local seen={}
 for _,entry in ipairs(manifest.assets)do
  assert(type(entry.id)=='string' and #entry.id<=64 and entry.id:match('^[a-z][a-z0-9_]*$') and not seen[entry.id],'voice source ID invalid or duplicate')
  seen[entry.id]=true
  assert(entry.file==entry.id..'.ogg' and entry.category=='voice','voice source namespace invalid')
  assert(type(entry.sha256)=='string' and entry.sha256:match('^[0-9a-f]+$') and #entry.sha256==64,'voice source hash invalid')
  assert(type(entry.caption)=='string' and #entry.caption>0 and #entry.caption<=240 and type(entry.spoken_text)=='string' and #entry.spoken_text>0,'voice transcript missing')
  local file=p.join(dir,entry.file);local size=p.attributes(file).size
  assert(size>=64 and size<=2*1024*1024,'voice source file size invalid')
  assert(h.sha256(file)==entry.sha256,'voice source SHA mismatch')
  local info=M.ogg(assert(p.read(file,true)))
  assert(info.bytes==entry.bytes,'voice source size mismatch')
  out.assets[#out.assets+1]={id=entry.id,category='voice',source=p.absolute(file),sha256=entry.sha256,bytes=info.bytes,
   channels=info.channels,rate=info.rate,frames=info.frames,seconds=info.seconds,
   destination='/Game/Omoba/Audio/VoiceV1',object_path='/Game/Omoba/Audio/VoiceV1/'..entry.id..'.'..entry.id}
 end
 return out
end
M.verify_result=require('audio_import').verify_result
M.producer='omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmOriginalAudioImportCommandlet.cpp'
M.receipt='omfue/Content/Omoba/Audio/VoiceV1/imported-assets-v1.json'
function M.verify_existing_packages(root)
 local file=p.join(root,M.receipt);if not p.exists(file)then return nil end
 local receipt=j.read(file);assert(receipt.schema=='omoba.voice-imported-assets/1' and type(receipt.packages)=='table'
  and #receipt.packages>=1 and #receipt.packages<=64,'voice receipt invalid')
 local seen={}
 for _,row in ipairs(receipt.packages)do
  assert(type(row.id)=='string' and #row.id<=64 and row.id:match('^[a-z][a-z0-9_]*$') and not seen[row.id],'voice receipt ID invalid or duplicated')
  seen[row.id]=true
  assert(row.file=='omfue/Content/Omoba/Audio/VoiceV1/'..row.id..'.uasset'
   and h.sha256(p.join(root,row.file))==row.sha256,'managed voice package changed; restore it before import')
  assert(row.decoded_file=='omfue/Content/Omoba/Audio/VoiceV1/'..row.id..'.wav'
   and h.sha256(p.join(root,row.decoded_file))==row.decoded_sha256,'derived voice PCM changed; restore it before import')
 end
 return receipt
end
function M.require_ready(root)
 local plan=M.validate(root);local receipt=assert(M.verify_existing_packages(root),'voice assets have not been imported')
 local ids={};for _,row in ipairs(plan.assets)do ids[row.id]=true end
 for _,id in ipairs({'announcer_ready','announcer_game_over','announcer_victory','announcer_defeat'})do assert(ids[id],'missing required voice: '..id)end
 assert(receipt.schema=='omoba.voice-imported-assets/1' and receipt.manifest_sha256==plan.manifest_sha256
  and receipt.producer_sha256==h.sha256(p.join(root,M.producer)) and #receipt.packages==#plan.assets,'voice verified imports are stale')
 local expected={};for _,row in ipairs(plan.assets)do expected[row.id]=row end
 for _,row in ipairs(receipt.packages)do
  local source=assert(expected[row.id],'voice imported package duplicated')
  local pcm=require('audio_import').pcm(p.read(p.join(root,row.decoded_file),true))
  assert(pcm.channels==source.channels and pcm.rate==source.rate and pcm.frames==source.frames,'derived voice PCM differs from Ogg metadata')
  expected[row.id]=nil
 end
 assert(not next(expected),'voice imported packages incomplete');return #plan.assets
end
return M
