package.path='scripts/?.lua;'..package.path
local warmup=require('ue_shader_warmup')

-- Frames count only after the map load, from the bracketed frame field.
assert(warmup.max_frame_after_load('LogInit: x\n[2026][  0]LogShaderCompilers: y\n')==nil)
local log='[a][  0]LogShaderCompilers: job\n[b][  0]LogLoad: Took 0.4 seconds\n[c][ 99]LogTemp: x\n[d][184]LogTemp: y\n'
assert(warmup.max_frame_after_load(log)==184)
assert(warmup.max_frame_after_load('[b][  0]LogLoad: Took 0.4 seconds\n')==0)

-- Warmup keeps project/map/RHI but drops identity, corelimit and the match endpoint.
local args=warmup.warmup_args({'D:/omoba/omfue/om.uproject','/Game/Map/Main','-game','-om-player=1',
  '-om-presentation=127.0.0.1:57062','-d3d11','-corelimit=6','-om-autopilot=lane_v0',
  '-om-autopilot-trace=x.jsonl','-UserDir=old','-abslog=old.log','-sessionname=omfue-role-p1'},'OUT')
local text=table.concat(args,' ')
assert(args[1]=='D:/omoba/omfue/om.uproject' and args[2]=='/Game/Map/Main')
assert(text:find('-d3d11',1,true) and text:find('-game',1,true))
for _,gone in ipairs({'-corelimit','-om-autopilot','57062','-UserDir=old','-abslog=old.log','omfue-role-p1'}) do
  assert(not text:find(gone,1,true),'warmup kept '..gone)
end
assert(text:find('-om-presentation=127.0.0.1:1',1,true) and text:find('ue-shader-warmup.log',1,true))

-- A stamp names the engine binary identity, so a new engine build warms again.
local stamp=warmup.stamp_path('D:/omoba','tools/lua/lua.exe','D:/omoba/omfue/om.uproject','d3d11')
assert(stamp:find('ue-shader-warmup',1,true) and stamp:find('d3d11-',1,true) and stamp:match('%.json$'))
assert(not pcall(warmup.stamp_path,'D:/omoba','missing/editor.exe','p','d3d11'))
print('shader warmup: frame parsing, argument hygiene and stamp identity passed')
