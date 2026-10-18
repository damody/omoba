local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]tests[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,h=b.lib('path'),b.lib('json'),b.lib('hash')
local a=require('audio_import')
local count=0
local function check(fn)fn();count=count+1 end
local function rejected(fn)check(function()assert(not pcall(fn),'invalid original audio accepted')end)end
local function pcm(channels,rate,samples)
 local data=string.rep(string.pack('<i2',12),samples*channels)
 return 'RIFF'..string.pack('<I4',36+#data)..'WAVEfmt '..string.pack('<I4I2I2I4I4I2I2',16,1,channels,rate,rate*channels*2,channels*2,16)..'data'..string.pack('<I4',#data)..data
end
local bytes=pcm(2,48000,24)
check(function()local x=a.pcm(bytes);assert(x.frames==24 and x.channels==2 and x.rate==48000)end)
check(function()assert(a.pcm(pcm(1,8000,2)).channels==1)end)
for _,offset in ipairs({1,5,9,13,17,21,23,25,29,33,35,37,41}) do
 local bad=bytes:sub(1,offset-1)..string.char(bytes:byte(offset)~255)..bytes:sub(offset+1)
 rejected(function()a.pcm(bad)end)
end
rejected(function()a.pcm(bytes:sub(1,-2))end)
rejected(function()a.pcm(bytes:sub(1,44)..string.rep('\0',#bytes-44))end)
rejected(function()a.pcm(pcm(2,48000,48000*31))end)
local root=p.join(b.root,'.ai-collab/audio-import-test-fixtures',tostring(os.time())..'-'..tostring({}):gsub('[^%w]',''))
local dir=p.join(root,'omfue/Content/Omoba/Audio/OriginalV1');p.mkdir_p(dir)
local file=p.join(dir,'sample.wav');p.write(file,bytes,false,true)
local manifest={schema='om.original-audio-source',version=1,channels=2,sample_rate=48000,pcm_bits=16,assets={{id='sample',file='sample.wav',category='effects',sha256=h.sha256(file),bytes=#bytes,sample_frames=24,seconds=24/48000}}}
local manifest_path=p.join(dir,'provenance-v1.json');j.write(manifest_path,manifest,false)
local plan=a.validate(root)
check(function()assert(#plan.assets==1 and plan.assets[1].object_path=='/Game/Omoba/Audio/OriginalV1/sample.sample')end)
local function bad_manifest(fn)
 local doc=j.decode(j.encode(manifest));fn(doc);j.write(manifest_path,doc,true);rejected(function()a.validate(root)end);j.write(manifest_path,manifest,true)
end
for _,fn in ipairs({
 function(x)x.schema='other'end,function(x)x.version=2 end,function(x)x.assets={}end,
 function(x)x.assets[2]=x.assets[1]end,function(x)x.assets[1].id='../sample'end,
 function(x)x.assets[1].file='../sample.wav'end,function(x)x.assets[1].sha256=string.rep('0',64)end,
 function(x)x.assets[1].bytes=1 end,function(x)x.assets[1].sample_frames=1 end,
 function(x)x.assets[1].seconds=1 end,function(x)x.sample_rate=8000 end,
 function(x)x.channels=1 end,function(x)x.pcm_bits=32 end,function(x)x.assets[1].category='unknown'end,
 function(x)for i=2,65 do x.assets[i]=x.assets[1]end end
})do bad_manifest(fn)end
local item=plan.assets[1]
local row={id=item.id,object_path=item.object_path,source_sha256=item.sha256,category=item.category,channels=item.channels,rate=item.rate,frames=item.frames,seconds=item.seconds,loaded_class='SoundWave',loaded_channels=item.channels,loaded_rate=item.rate,loaded_seconds=item.seconds,error='',action='imported'}
local result={schema='omoba.audio-import-result.v1',imported=1,verified=0,saved=1,failed=0,error='',assets={row}}
check(function()a.verify_result(plan,result)end)
for _,fn in ipairs({
 function(x)x.failed=1 end,function(x)x.saved=0 end,function(x)x.assets={}end,
 function(x)x.assets[1].loaded_class='SoundCue'end,function(x)x.assets[1].loaded_rate=8000 end,
 function(x)x.assets[1].loaded_channels=1 end,function(x)x.assets[1].loaded_seconds=10 end,
 function(x)x.assets[1].source_sha256=string.rep('0',64)end,function(x)x.assets[1].action='verified'end,
 function(x)x.assets[1].object_path='/Game/Other.sample'end,function(x)x.assets[2]=x.assets[1]end,
 function(x)x.assets[1].error='save failed'end,function(x)x.imported=0.5 end
})do local copy=j.decode(j.encode(result));fn(copy);rejected(function()a.verify_result(plan,copy)end)end
check(function()local copy=j.decode(j.encode(result));copy.imported=0;copy.saved=0;copy.verified=1;copy.assets[1].action='verified';a.verify_result(plan,copy)end)
os.remove(manifest_path);os.remove(file)
print('PASS original audio import contracts: '..count..' independent cases')
