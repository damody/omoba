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
-- the contact lands at 0.6 s (impact in moba_tripo_hero_visuals). The fourth value tilts the right wrist (degrees, pinky side
-- toward the fingers): a rifle stab's fist would otherwise hold a spear upright. The fifth is the kept frame after which the
-- clip returns to its first pose: a clip that holds the blade behind the body would otherwise show it pointing away until the
-- next swing. The sixth is the contact or release in seconds (contact in moba_tripo_hero_visuals), where the body is held closest to the
-- unit's front (mixamo_to_library.py keeps it there). The seventh narrows how far the chest may turn away from that moment,
-- in degrees: a cast or throw has no swing to keep. The eighth turns the arms by itself (degrees from the source clip): a gun stays on target.
local CLIPS={
  Mx_Sword_Slash={'Great Sword Slash',6,42,0,0,0.64,25},
  Mx_Sword_Inward={'Stable Sword Inward Slash',18,54,0,22,0.59},
  Mx_Sword_Outward={'Stable Sword Outward Slash',8,44,0,0,0.87},
  Mx_Sword_Draw={'Withdrawing Sword',nil,nil,0,0,0.9,25},
  Mx_Axe_Down={'Standing Melee Attack Downward',19,43,45,18,0.28},
  Mx_Axe_Smash={'Great Sword Downward Slash',nil,nil,0,0,0.67,25},
  Mx_Sweep={'Standing Melee Attack Horizontal',11,47,0,0,0.58,25},
  Mx_Spear_Thrust={'Bayonet Stab',10,46,50,0,0.74},
  Mx_Spear_Upward={'Upward Thrust',nil,nil,0,0,0.73,25},
  Mx_Rifle_Fire={'Firing Rifle',nil,nil,0,0,0.07,25,-20},
  Mx_Battlecry={'Standing Taunt Battlecry',0,65,0,0,0.5,25},
  Mx_Charge={'Charge',0,100,0,0,0.33,25},
  Mx_Kick={'Standing Melee Attack Kick Ver. 1',nil,nil,0,0,0.67,25},
  Mx_Throw={'Throw',nil,nil,0,0,0.9,25},
  Mx_Throw_Short={'Throw',8,44,0,0,0.6,25},
  Mx_Cast_Two={'Two Hand Spell Casting',0,75,0,0,0.6,25},
  Mx_Channel={'Standing 2H Magic Attack 03',0,90,0,0,1.0,25},
  Mx_Death_Back='Standing React Death Backward',
  -- No Mixamo file: built on the library's own pistol aim.
  -- Both hands on the axe (two_hand_clip.py), built on clips above or the library's: bake those first.
  Mx_Axe2_Swing={generated='scripts/two_hand_clip.py',source='Mx_Axe_Smash',mode='swing'},
  Mx_Axe2_Slash={generated='scripts/two_hand_clip.py',source='Mx_Sword_Slash',mode='swing'},
  Mx_Axe2_Idle={generated='scripts/two_hand_clip.py',source='library:Idle_Loop',mode='hold'},
  Mx_Axe2_Jog={generated='scripts/two_hand_clip.py',source='library:Jog_Fwd_Loop',mode='hold'},
  Mx_Axe2_Hit={generated='scripts/two_hand_clip.py',source='library:Hit_Chest',mode='hold'},
  Mx_Axe2_Battlecry={generated='scripts/two_hand_clip.py',source='Mx_Battlecry',mode='hold'},
  Mx_Axe2_Channel={generated='scripts/two_hand_clip.py',source='Mx_Channel',mode='hold'},
  Mx_Axe2_Charge={generated='scripts/two_hand_clip.py',source='Mx_Charge',mode='hold'},
  Mx_Pistol_Fire_R={generated='scripts/pistols_clip.py',mode='fire_r'},
  Mx_Pistol_Fire_L={generated='scripts/pistols_clip.py',mode='fire_l'},
  Mx_Pistols_Volley={generated='scripts/pistols_clip.py',mode='volley'},
  Mx_Pistols_Idle={generated='scripts/pistols_clip.py',source='library:Idle_Loop',mode='ready'},
  Mx_Pistols_Jog={generated='scripts/pistols_clip.py',source='library:Jog_Fwd_Loop',mode='ready'},
  Mx_Pistols_Hit={generated='scripts/pistols_clip.py',source='library:Hit_Chest',mode='ready'},
  -- Plays half as fast again so the whole kick fits a cast's first second; speed and spin (start, seconds, degrees) in mixamo_to_library.py.
  Mx_Spin_Kick={'Standing Melee Attack Kick Ver. 1',0,45,0,0,0.42,25,speed=1.5,spin='0.1,0.7,360'},
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
  local source=entry[1] and p.join(input,entry[1]..'.fbx');assert(entry.generated or p.is_file(source),'missing '..tostring(source))
  local out=p.join(baked,name..'.fbx')
  if not(options['--reuse-baked'] and p.is_file(out)) then
  local generated={'-b','--python-exit-code','1','-P',p.join(b.root,entry.generated or''),'--',library}
  if entry.source then generated[#generated+1]=entry.source end
  generated[#generated+1]=out;generated[#generated+1]=entry.mode
  local result=entry.generated and r.run(blender,generated,{cwd=b.root,check=false})
    or r.run(blender,{'-b','--python-exit-code','1','-P',p.join(b.root,'scripts/mixamo_to_library.py'),'--',library,p.join(input,'T-Pose.fbx'),source,out,tostring(entry[2]or''),tostring(entry[3]or''),tostring(entry[4]or 0),tostring(entry[5]or 0),tostring(entry[6]or -1),tostring(entry[7]or 0),tostring(entry[8]or''),tostring(entry.speed or 1),entry.spin or''},{cwd=b.root,check=false})
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
