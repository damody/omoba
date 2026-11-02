-- Mixamo motions (files from target/mixamo-input, never committed) through to the Tripo rigs:
-- Blender moves each onto the CC0 library's skeleton (scripts/mixamo_to_library.py), then the retarget commandlet
-- sends it through each hero's existing retargeter into /Game/Omoba/Characters/AddedClipsV1/<hero>/<hero>_<Clip>.
--
--   tools\lua\lua.exe scripts\mixamo_clips.lua --ue-root D:\UE5.8 --heroes sasaki_kojiro [--clips Mx_A,Mx_B] [--bake-only]
local script=debug.getinfo(1,'S').source:sub(2);local dir=assert(script:match('^(.*)[/\\]'));package.path=dir..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,r=b.lib('path'),b.lib('json'),b.lib('process')
local model=require('character_retarget')

-- Clip name in moba_tripo_hero_visuals -> the Mixamo file's name.
local CLIPS={
  Mx_Sword_Slash='Great Sword Slash',
  Mx_Sword_Inward='Stable Sword Inward Slash',
  Mx_Sword_Outward='Stable Sword Outward Slash',
  Mx_Sword_Draw='Withdrawing Sword',
}

local options={};local i=1
while i<=#arg do
  local key=arg[i]
  if key=='--bake-only' then options[key]=true;i=i+1 else options[key]=assert(arg[i+1],key);i=i+2 end
end
local function list(text)local out={};for v in (text or''):gmatch('[^,]+')do out[#out+1]=v end;return out end
local names=list(options['--clips']);if #names==0 then for name in pairs(CLIPS)do names[#names+1]=name end;table.sort(names)end
local input,baked=p.join(b.root,'target/mixamo-input'),p.join(b.root,'target/mixamo-baked');p.mkdir_p(baked)
local blender=p.join(b.root,'target/tools/blender/blender-4.5.9-windows-x64/blender.exe')
local library=p.join(b.root,'omfue/SourceArt/Characters/QuaterniusV1/animation_library.fbx')

local added={}
for _,name in ipairs(names)do
  local source=p.join(input,assert(CLIPS[name],'unknown clip '..name)..'.fbx');assert(p.is_file(source),'missing '..source)
  local out=p.join(baked,name..'.fbx')
  local result=r.run(blender,{'-b','--python-exit-code','1','-P',p.join(b.root,'scripts/mixamo_to_library.py'),'--',library,p.join(input,'T-Pose.fbx'),source,out},{cwd=b.root,check=false})
  assert(result.exit_code==0 and p.is_file(out),'bake failed: '..name..'\n'..tostring(result.stdout):sub(-1500))
  added[#added+1]={name=name,fbx=out}
  print('baked '..name)
end
if options['--bake-only'] then return end

local engine=p.absolute(assert(options['--ue-root'],'--ue-root required'),b.root)
local heroes=list(assert(options['--heroes'],'--heroes required'))
local plan=model.validate(b.root,heroes,'tripo_v1');plan.added_clips=added
local evidence=p.join(b.root,'target/mixamo-baked/evidence');p.mkdir_p(evidence)
local plan_file,output=p.join(evidence,'plan.json'),p.join(evidence,'result.json');j.write(plan_file,plan,false)
local snapshot=require('ue_run_snapshot').prepare(b.root)
local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{snapshot.uproject,'-run=OmCharacterRetarget','-OmRetargetPlan='..plan_file,'-OmRetargetResult='..output,
  '-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding','-abslog='..p.join(evidence,'engine.log')},{cwd=snapshot.project_dir,check=false})
local value=p.is_file(output)and j.read(output)or{}
assert(result.exit_code==0 and value.success,'retarget failed: '..tostring(value.error))
print(('added %d clip(s) for %d hero(es), saved %d'):format(#added,#heroes,value.saved))
