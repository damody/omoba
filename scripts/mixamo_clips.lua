-- Mixamo motions (files from target/mixamo-input, never committed) through to the Tripo rigs:
-- Blender moves each onto the CC0 library's skeleton (scripts/mixamo_to_library.py), then the retarget commandlet
-- sends it through each hero's existing retargeter into /Game/Omoba/Characters/AddedClipsV1/<hero>/<hero>_<Clip>.
--
--   tools\lua\lua.exe scripts\mixamo_clips.lua --ue-root D:\UE5.8 --heroes sasaki_kojiro [--clips Mx_A,Mx_B] [--bake-only] [--reuse-baked]
local script=debug.getinfo(1,'S').source:sub(2);local dir=assert(script:match('^(.*)[/\\]'));package.path=dir..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j,r=b.lib('path'),b.lib('json'),b.lib('process')
local model=require('character_retarget')

-- Clip name in moba_tripo_hero_visuals -> the Mixamo file's name, with the frames kept when the motion runs long.
-- Swings keep 36 frames (1.2 s) around the fastest moment of the hand, so a fast attack cycle still reads and
-- the contact lands at 0.6 s (impact in moba_tripo_hero_visuals). The fourth value turns a clip authored side-on
-- (degrees about up), so at the contact the head, chest and hand are all within about 30 degrees of the unit's front:
-- the body faces the target, not only the hand. The sixth is the kept frame after which the clip returns to its first
-- pose: a clip that holds the blade behind the body would otherwise show it pointing away until the next swing. The fifth tilts
-- the right wrist (degrees, pinky side toward the fingers): a rifle stab's fist would otherwise hold a spear upright.
local CLIPS={
  Mx_Sword_Slash={'Great Sword Slash',6,42,-47},
  Mx_Sword_Inward={'Stable Sword Inward Slash',18,54,32,0,22},
  Mx_Sword_Outward={'Stable Sword Outward Slash',8,44,-34},
  Mx_Sword_Draw='Withdrawing Sword',
  Mx_Axe_Down={'Standing Melee Attack Downward',7,43,-20},
  Mx_Axe_Smash='Great Sword Downward Slash',
  Mx_Sweep={'Standing Melee Attack Horizontal',11,47,-35},
  Mx_Spear_Thrust={'Bayonet Stab',10,46,39,50},
  Mx_Spear_Upward='Upward Thrust',
  Mx_Rifle_Fire='Firing Rifle',
  Mx_Battlecry={'Standing Taunt Battlecry',0,65,25},
  Mx_Charge={'Charge',0,100},
  Mx_Kick='Standing Melee Attack Kick Ver. 1',
  Mx_Throw='Throw',
  Mx_Throw_Short={'Throw',8,44},
  Mx_Cast_Two={'Two Hand Spell Casting',0,75},
  Mx_Channel={'Standing 2H Magic Attack 03',0,90},
  Mx_Death_Back='Standing React Death Backward',
}

local options={};local i=1
while i<=#arg do
  local key=arg[i]
  if key=='--bake-only' or key=='--reuse-baked' then options[key]=true;i=i+1 else options[key]=assert(arg[i+1],key);i=i+2 end
end
local function list(text)local out={};for v in (text or''):gmatch('[^,]+')do out[#out+1]=v end;return out end
local names=list(options['--clips']);if #names==0 then for name in pairs(CLIPS)do names[#names+1]=name end;table.sort(names)end
local input,baked=p.join(b.root,'target/mixamo-input'),p.join(b.root,'target/mixamo-baked');p.mkdir_p(baked)
local blender=p.join(b.root,'target/tools/blender/blender-4.5.9-windows-x64/blender.exe')
local library=p.join(b.root,'omfue/SourceArt/Characters/QuaterniusV1/animation_library.fbx')

local added={}
for _,name in ipairs(names)do
  local entry=assert(CLIPS[name],'unknown clip '..name);if type(entry)=='string' then entry={entry} end
  local source=p.join(input,entry[1]..'.fbx');assert(p.is_file(source),'missing '..source)
  local out=p.join(baked,name..'.fbx')
  if not(options['--reuse-baked'] and p.is_file(out)) then
  local result=r.run(blender,{'-b','--python-exit-code','1','-P',p.join(b.root,'scripts/mixamo_to_library.py'),'--',library,p.join(input,'T-Pose.fbx'),source,out,tostring(entry[2]or''),tostring(entry[3]or''),tostring(entry[4]or 0),tostring(entry[5]or 0),tostring(entry[6]or 0)},{cwd=b.root,check=false})
  assert(result.exit_code==0 and p.is_file(out),'bake failed: '..name..'\n'..tostring(result.stdout):sub(-1500))
  end
  added[#added+1]={name=name,fbx=out}
  print('baked '..name)
end
if options['--bake-only'] then return end

local engine=p.absolute(assert(options['--ue-root'],'--ue-root required'),b.root)
local heroes=list(assert(options['--heroes'],'--heroes required'))
local plan=model.validate(b.root,heroes,'tripo_v1');plan.added_clips=added
local evidence=p.join(b.root,'target/mixamo-baked/evidence',heroes[1]..'-'..os.time());p.mkdir_p(evidence)
local plan_file,output=p.join(evidence,'plan.json'),p.join(evidence,'result.json');j.write(plan_file,plan,false)
local snapshot=require('ue_run_snapshot').prepare(b.root)
local result=r.run(p.join(engine,'Engine/Binaries/Win64/UnrealEditor-Cmd.exe'),{snapshot.uproject,'-run=OmCharacterRetarget','-OmRetargetPlan='..plan_file,'-OmRetargetResult='..output,
  '-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding','-abslog='..p.join(evidence,'engine.log')},{cwd=snapshot.project_dir,check=false})
local value=p.is_file(output)and j.read(output)or{}
assert(result.exit_code==0 and value.success,'retarget failed: '..tostring(value.error))
print(('added %d clip(s) for %d hero(es), saved %d'):format(#added,#heroes,value.saved))
