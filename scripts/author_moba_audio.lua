-- Original deterministic sound design. No samples or third-party compositions.
-- Source WAV preparation only; Unreal import/playback is a separate consumer.
local source=debug.getinfo(1,'S').source:sub(2)
local script_dir=assert(source:match('^(.*)[/\\]'))
package.path=script_dir..'/?.lua;'..package.path
local bootstrap=require('_bootstrap')
local path,json,hash=bootstrap.lib('path'),bootstrap.lib('json'),bootstrap.lib('hash')
local sample_rate=48000
local tau=2*math.pi
local recipes={
 {id='ui_hover',category='effects',seconds=.065,frequency=920,end_frequency=1160,tone=.22,noise=0,seed=11},
 {id='ui_confirm',category='effects',seconds=.16,frequency=620,end_frequency=1240,tone=.38,noise=.015,seed=13},
 {id='ui_cancel',category='effects',seconds=.12,frequency=720,end_frequency=360,tone=.28,noise=.02,seed=17},
 {id='ui_refused',category='effects',seconds=.22,frequency=155,end_frequency=110,tone=.36,noise=.03,seed=19},
 {id='move_admitted',category='effects',seconds=.10,frequency=440,end_frequency=660,tone=.25,noise=.025,seed=23},
 {id='attack_admitted',category='effects',seconds=.22,frequency=260,end_frequency=100,tone=.30,noise=.22,seed=29},
 {id='impact',category='effects',seconds=.16,frequency=110,end_frequency=45,tone=.42,noise=.30,seed=31},
 {id='cast_admitted',category='effects',seconds=.44,frequency=330,end_frequency=990,tone=.30,noise=.12,seed=37},
 {id='healing',category='effects',seconds=.65,frequency=523.25,end_frequency=784.875,tone=.34,noise=.01,seed=41},
 {id='death',category='effects',seconds=.85,frequency=220,end_frequency=55,tone=.36,noise=.10,seed=43},
 {id='respawn',category='effects',seconds=.90,frequency=196,end_frequency=784,tone=.33,noise=.025,seed=47},
 {id='recall_channel',category='effects',seconds=1.60,frequency=261.625,end_frequency=392.4375,tone=.24,noise=.015,seed=53},
 {id='training_ambient',category='music',seconds=8,frequency=98,end_frequency=98,tone=.16,noise=.008,seed=59},
}
local destination=path.join(bootstrap.root,'omfue/Content/Omoba/Audio/OriginalV1')
path.mkdir_p(destination)
local manifest={schema='om.original-audio-source',version=1,authoring_source='scripts/author_moba_audio.lua',
 provenance='Original deterministic oscillator/noise/envelope composition; no external recordings, samples or melodies.',
 sample_rate=sample_rate,channels=2,pcm_bits=16,unreal_imported=false,runtime_playback_verified=false,assets={}}

local function write_identical_or_new(file,data)
 if path.exists(file) then assert(path.read(file,true)==data,'existing source differs; preserve it and version the recipe: '..file)
 else path.write(file,data,false,true) end
end

for _,recipe in ipairs(recipes) do
 assert(recipe.id:match('^[a-z][a-z_]+$') and recipe.seconds>0 and recipe.seconds<=8)
 local count=math.floor(recipe.seconds*sample_rate+.5)
 local frames,phase,low_noise={},0,0
 local rng,peak,sum_squares,sum=recipe.seed,0,0,0
 for n=0,count-1 do
  local time=n/sample_rate
  local progress=n/(count-1)
  rng=(1664525*rng+1013904223)&0xffffffff
  local noise=rng/2147483648-1
  low_noise=low_noise+.13*(noise-low_noise)
  local frequency=recipe.frequency*(recipe.end_frequency/recipe.frequency)^progress
  phase=(phase+tau*frequency/sample_rate)%tau
  local attack=math.min(1,time/.008)
  local release=math.min(1,(recipe.seconds-time)/.035)
  local envelope=(math.sin(math.min(attack,release)*math.pi/2))^2
  if recipe.id~='training_ambient' then envelope=envelope*math.exp(-2.3*progress) end
  local tone=math.sin(phase)+.22*math.sin(2*phase)+.09*math.sin(3*phase)
  if recipe.id=='training_ambient' then
   -- Integer cycles over eight seconds make the stationary harmonic bed repeatable.
   tone=math.sin(tau*98*time)+.35*math.sin(tau*147*time)+.20*math.sin(tau*196*time)
  end
  local value=envelope*(recipe.tone*tone+recipe.noise*low_noise)
  assert(value==value and math.abs(value)<.85,'nonfinite or clipping source')
  local pcm=math.floor(value*32767+.5)
  peak=math.max(peak,math.abs(pcm));sum_squares=sum_squares+pcm*pcm;sum=sum+pcm
  frames[#frames+1]=string.pack('<i2i2',pcm,pcm)
 end
 local data=table.concat(frames)
 local wav='RIFF'..string.pack('<I4',36+#data)..'WAVEfmt '..string.pack('<I4I2I2I4I4I2I2',16,1,2,sample_rate,sample_rate*4,4,16)..'data'..string.pack('<I4',#data)..data
 local file=path.join(destination,recipe.id..'.wav')
 write_identical_or_new(file,wav)
 -- Read the actual persisted artifact and validate the standard PCM envelope.
 local readback=assert(path.read(file,true))
 assert(readback:sub(1,4)=='RIFF' and readback:sub(9,16)=='WAVEfmt ' and readback:sub(37,40)=='data')
 local riff_size=string.unpack('<I4',readback,5)
 local fmt_size,format,channels,rate,byte_rate,align,bits=string.unpack('<I4I2I2I4I4I2I2',readback,17)
 local data_size=string.unpack('<I4',readback,41)
 assert(riff_size+8==#readback and fmt_size==16 and format==1 and channels==2 and rate==sample_rate)
 assert(byte_rate==rate*align and align==4 and bits==16 and data_size==count*align and #readback==44+data_size)
 local rms=math.sqrt(sum_squares/count)/32767
 assert(peak>0 and rms>.005 and math.abs(sum/count/32767)<.03,'silent or DC-biased sound')
 manifest.assets[#manifest.assets+1]={id=recipe.id,category=recipe.category,file=recipe.id..'.wav',
  seconds=count/sample_rate,sample_frames=count,bytes=#readback,sha256=hash.sha256(file),peak=peak/32767,rms=rms,recipe=recipe}
 print(recipe.id..' PCM verified '..count..' frames')
end
local manifest_file=path.join(destination,'provenance-v1.json')
local encoded=json.encode(manifest)
write_identical_or_new(manifest_file,encoded)
print('Prepared '..#recipes..' original PCM sources; import and actual playback remain pending')
