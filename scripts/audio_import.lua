-- Project-owned original PCM validation. No editor or asset mutation here.
local path = require('tools.lua.lib.path')
local json = require('tools.lua.lib.json')
local hash = require('tools.lua.lib.hash')
local M = {}
local function whole(value, lo, hi)
  return type(value)=='number' and value==math.floor(value) and value>=lo and value<=hi
end
function M.pcm(data)
  assert(type(data)=='string' and #data>=48 and #data<=32*1024*1024,'PCM file size invalid')
  assert(data:sub(1,4)=='RIFF' and data:sub(9,16)=='WAVEfmt ' and data:sub(37,40)=='data','PCM container invalid')
  local riff=string.unpack('<I4',data,5)
  local fmt,kind,channels,rate,byte_rate,align,bits=string.unpack('<I4I2I2I4I4I2I2',data,17)
  local bytes=string.unpack('<I4',data,41)
  assert(riff+8==#data and fmt==16 and kind==1 and (channels==1 or channels==2),'PCM format invalid')
  assert(rate>=8000 and rate<=96000 and bits==16 and align==channels*2 and byte_rate==rate*align,'PCM rate/layout invalid')
  assert(bytes>0 and bytes%align==0 and #data==44+bytes,'PCM data size invalid')
  local frames=bytes//align
  assert(frames/rate<=30,'PCM duration exceeds bounded original asset policy')
  local peak=0
  for offset=45,#data,2 do peak=math.max(peak,math.abs(string.unpack('<i2',data,offset))) end
  assert(peak>0,'PCM source is silent')
  return {channels=channels,rate=rate,frames=frames,seconds=frames/rate,bytes=#data}
end
function M.validate(root, manifest_path)
  local source_dir=path.join(root,'omfue/Content/Omoba/Audio/OriginalV1')
  manifest_path=manifest_path or path.join(source_dir,'provenance-v1.json')
  local manifest=json.read(manifest_path)
  assert(manifest.schema=='om.original-audio-source' and manifest.version==1,'original audio provenance schema invalid')
  assert(type(manifest.assets)=='table' and #manifest.assets>=1 and #manifest.assets<=64,'original audio asset count invalid')
  local out={schema='omoba.audio-import-plan.v1',manifest=path.absolute(manifest_path),manifest_sha256=hash.sha256(manifest_path),assets={}}
  local seen={}
  for _,entry in ipairs(manifest.assets) do
    assert(type(entry.id)=='string' and #entry.id<=64 and entry.id:match('^[a-z][a-z0-9_]*$'),'original audio ID invalid')
    assert(not seen[entry.id],'duplicate original audio asset');seen[entry.id]=true
    assert(entry.file==entry.id..'.wav','original audio filename mismatch')
    assert(entry.category=='effects' or entry.category=='music' or entry.category=='voice','original audio category invalid')
    assert(type(entry.sha256)=='string' and #entry.sha256==64 and entry.sha256:match('^[0-9a-f]+$'),'original audio SHA256 invalid')
    local source=path.join(source_dir,entry.file)
    local pcm=M.pcm(assert(path.read(source,true)))
    assert(hash.sha256(source)==entry.sha256,'original audio source hash mismatch')
    assert(whole(entry.bytes,48,32*1024*1024) and pcm.bytes==entry.bytes,'original audio source size mismatch')
    assert(entry.sample_frames==pcm.frames and entry.seconds==pcm.seconds,'original audio duration mismatch')
    assert(manifest.channels==pcm.channels and manifest.sample_rate==pcm.rate and manifest.pcm_bits==16,'original audio provenance layout mismatch')
    out.assets[#out.assets+1]={id=entry.id,category=entry.category,source=path.absolute(source),sha256=entry.sha256,
      bytes=pcm.bytes,channels=pcm.channels,rate=pcm.rate,frames=pcm.frames,seconds=pcm.seconds,
      destination='/Game/Omoba/Audio/OriginalV1',object_path='/Game/Omoba/Audio/OriginalV1/'..entry.id..'.'..entry.id}
  end
  return out
end
function M.verify_result(plan, result)
  assert(type(result)=='table' and result.schema=='omoba.audio-import-result.v1','audio result schema invalid')
  for _,key in ipairs({'imported','verified','saved','failed'}) do
    assert(whole(result[key],0,64),'audio result count invalid: '..key)
  end
  assert(result.failed==0 and result.error=='' and result.imported+result.verified==#plan.assets and result.saved==result.imported,'audio resources incomplete or save count invalid')
  assert(type(result.assets)=='table' and #result.assets==#plan.assets,'audio resource rows missing')
  local expected={};for _,asset in ipairs(plan.assets) do expected[asset.id]=asset end
  local imported,verified=0,0
  for _,row in ipairs(result.assets) do
    local asset=assert(expected[row.id],'unknown or duplicate audio resource row');expected[row.id]=nil
    assert(row.error=='' and row.object_path==asset.object_path and row.source_sha256==asset.sha256 and row.category==asset.category,'audio resource identity mismatch')
    assert(row.loaded_class=='SoundWave' and row.loaded_channels==asset.channels and row.loaded_rate==asset.rate
      and type(row.loaded_seconds)=='number' and math.abs(row.loaded_seconds-asset.seconds)<=0.002,'actual SoundWave resource differs from PCM')
    assert(row.channels==asset.channels and row.rate==asset.rate and row.frames==asset.frames and row.seconds==asset.seconds,'audio source report differs from PCM')
    if row.action=='imported' then imported=imported+1
    elseif row.action=='verified' then verified=verified+1
    else error('audio resource action invalid') end
  end
  assert(not next(expected) and imported==result.imported and verified==result.verified,'audio resource count mismatch')
  return true
end
return M
