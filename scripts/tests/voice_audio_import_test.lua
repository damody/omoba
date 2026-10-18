local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]tests[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash');local a=require('voice_audio_import')
local count=0
local function check(fn)fn();count=count+1 end
local function reject(fn)check(function()assert(not pcall(fn),'invalid voice accepted')end)end
local function page(flags,granule,seq,payload,serial)
 return 'OggS'..string.pack('<BBI8I4I4I4BB',0,flags,granule,serial or 1,seq,0,1,#payload)..payload
end
local header='\1vorbis'..string.pack('<I4BI4',0,2,44100)..string.rep('\0',14)
local bytes=page(2,0,0,header)..page(4,88,1,'abc')
local samples=string.rep(string.pack('<i2',12),88*2)
local wavebytes='RIFF'..string.pack('<I4',36+#samples)..'WAVEfmt '..string.pack('<I4I2I2I4I4I2I2',16,1,2,44100,44100*4,4,16)..'data'..string.pack('<I4',#samples)..samples
check(function()local x=a.ogg(bytes);assert(x.frames==88 and x.rate==44100 and x.channels==2)end)
for _,bad in ipairs({bytes:sub(2),bytes:sub(1,-2),bytes..'x',page(2,0,0,header)..page(0,88,1,'abc'),
 page(2,0,0,header)..page(4,88,2,'abc'),page(2,0,0,header)..page(4,88,1,'abc',2),
 page(2,0,0,header)..page(4,441001,1,'abc'),page(2,10,0,header)..page(4,8,1,'abc'),
 page(2,0,0,header)..page(4,0,1,'abc'),page(0,0,0,header)..page(4,88,1,'abc'),
 page(2,0,0,'not a vorbis identification packet')..page(4,88,1,'abc')})do reject(function()a.ogg(bad)end)end
local root=p.join(b.root,'.ai-collab/voice-import-test-fixtures',tostring(os.time())..tostring({}):gsub('[^%w]',''))
local dir=p.join(root,'omfue/Content/Omoba/Audio/VoiceV1');p.mkdir_p(dir)
p.write(p.join(dir,'sample.ogg'),bytes,false,true);p.write(p.join(dir,'LICENSE.txt'),'test CC0 fixture',false,true)
local manifest={schema='omoba.cc0-voice-source/1',license='CC0-1.0',included_license_file='LICENSE.txt',included_license_sha256=h.sha256(p.join(dir,'LICENSE.txt')),
 assets={{id='sample',file='sample.ogg',category='voice',sha256=h.sha256(p.join(dir,'sample.ogg')),bytes=#bytes,spoken_text='Ready',caption='準備開始'}}}
local file=p.join(dir,'provenance-v1.json')
local function write(value)j.write(file,value,true)end
write(manifest)
check(function()local plan=a.validate(root);assert(#plan.assets==1 and plan.assets[1].frames==88 and plan.assets[1].destination=='/Game/Omoba/Audio/VoiceV1')end)
for _,mutate in ipairs({function(m)m.license='unknown'end,function(m)m.included_license_file='../LICENSE.txt'end,
 function(m)m.included_license_sha256=string.rep('0',64)end,function(m)m.assets[1].id='../sample'end,
 function(m)m.assets[1].file='../sample.ogg'end,function(m)m.assets[1].category='effects'end,
 function(m)m.assets[1].caption=''end,function(m)m.assets[1].spoken_text=''end,
 function(m)m.assets[1].sha256=string.rep('0',64)end,function(m)m.assets[1].bytes=#bytes+1 end,
 function(m)m.assets[2]=m.assets[1]end,function(m)m.assets={}end})do
 local copy=j.decode(j.encode(manifest));mutate(copy);write(copy);reject(function()a.validate(root)end)
end
write(manifest)
local plan=a.validate(root);local x=plan.assets[1]
local result={schema='omoba.audio-import-result.v1',imported=1,verified=0,saved=1,failed=0,error='',assets={{
 id=x.id,object_path=x.object_path,source_sha256=x.sha256,category='voice',channels=2,rate=44100,frames=88,seconds=88/44100,
 loaded_class='SoundWave',loaded_channels=2,loaded_rate=44100,loaded_seconds=88/44100,action='imported',error=''}}}
check(function()assert(a.verify_result(plan,result))end)
result.assets[1].loaded_rate=48000;reject(function()a.verify_result(plan,result)end)
check(function()assert(a.verify_existing_packages(root)==nil)end)
manifest.assets={}
for _,id in ipairs({'announcer_ready','announcer_game_over','announcer_victory','announcer_defeat'})do
 p.write(p.join(dir,id..'.ogg'),bytes,false,true);p.write(p.join(dir,id..'.uasset'),'synthetic package fixture',false,true)
 p.write(p.join(dir,id..'.wav'),wavebytes,false,true)
 manifest.assets[#manifest.assets+1]={id=id,file=id..'.ogg',category='voice',sha256=h.sha256(p.join(dir,id..'.ogg')),bytes=#bytes,spoken_text='fixture',caption='fixture'}
end
write(manifest)
local producer=p.join(root,a.producer);p.write(producer,'synthetic producer fixture',false,true)
local receipt={schema='omoba.voice-imported-assets/1',manifest_sha256=h.sha256(file),producer_sha256=h.sha256(producer),packages={}}
for _,entry in ipairs(manifest.assets)do local relative='omfue/Content/Omoba/Audio/VoiceV1/'..entry.id..'.uasset'
 local decoded='omfue/Content/Omoba/Audio/VoiceV1/'..entry.id..'.wav'
 receipt.packages[#receipt.packages+1]={id=entry.id,file=relative,sha256=h.sha256(p.join(root,relative)),decoded_file=decoded,decoded_sha256=h.sha256(p.join(root,decoded))}
end
local receiptfile=p.join(root,a.receipt);j.write(receiptfile,receipt,false)
check(function()assert(a.require_ready(root)==4)end)
local asset=p.join(root,receipt.packages[1].file);p.write(asset,'changed package',true,true)
reject(function()a.verify_existing_packages(root)end);reject(function()a.require_ready(root)end)
p.write(asset,'synthetic package fixture',true,true)
local decoded=p.join(root,receipt.packages[1].decoded_file);p.write(decoded,'changed PCM',true,true)
reject(function()a.verify_existing_packages(root)end);p.write(decoded,wavebytes,true,true)
p.write(producer,'changed producer',true,true);reject(function()a.require_ready(root)end)
p.write(producer,'synthetic producer fixture',true,true)
receipt.packages[2]=receipt.packages[1];j.write(receiptfile,receipt,true)
reject(function()a.verify_existing_packages(root)end)
print('voice_audio_import: '..count..' structural/source/result checks passed; synthetic pages do not claim native decoding')
