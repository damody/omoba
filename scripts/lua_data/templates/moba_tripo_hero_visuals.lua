-- Unreal-only presentation for heroes whose own body is a generated Tripo rig
-- (omfue/SourceArt/Characters/TripoV1/<hero>/, imported by import_character_sources --source-set tripo_v1
-- and animated by retarget_characters --target-set tripo_v1 from the shared CC0 animation library).
-- Never gameplay data. A hero is listed in `imported` only after both steps verified its assets, so a
-- hero without them keeps its previous visual (or the shared fallback).
return function(ctx)
  local root='/Game/Omoba/Characters/'
  -- Clip choices per hero from animation_library (A_TPose ... Walk_Loop). Hero bodies are modelled
  -- empty-handed; a held weapon is a separate static model (weapons below).
  local definitions={
    -- The rifle hangs on the back: an empty-handed two-hand pistol stance reads as a zombie at rest,
    -- so saika idles relaxed and raises the aim only to shoot.
    saika_magoichi={idle='Idle_Loop',attack='Pistol_Shoot',critical='Pistol_Shoot',impact=0.25,cast='Pistol_Shoot',
      extra={sniper={'Pistol_Aim_Neutral',true},sniper_move={'Walk_Loop',true}},state_slots={sniper_mode='sniper',sniper_walk='sniper_move'}},
    date_masamune={idle='Sword_Idle',attack='Sword_Attack',critical='Sword_Attack',impact=0.65},
    asai_nagamasa={idle='Sword_Idle',attack='Sword_Attack',critical='Sword_Attack',impact=0.65},
    murakami_yoshikiyo={idle='Idle_Loop',attack='Sword_Attack',critical='Punch_Cross',impact=0.65},
    -- The spear is strapped across the back: empty-handed sword swings read as spear thrusts at MOBA distance.
    honda_tadakatsu={idle='Idle_Loop',attack='Sword_Attack',critical='Sword_Attack',impact=0.65},
    sanada_yukimura={idle='Idle_Loop',attack='Sword_Attack',critical='Sword_Attack',impact=0.65},
    inahime={idle='Idle_Loop',attack='Spell_Simple_Shoot',critical='Spell_Simple_Shoot',impact=0.3},
    tamako={idle='Idle_Loop',attack='Spell_Simple_Shoot',critical='Spell_Simple_Shoot',impact=0.3},
    -- A raised casting hand at rest leaves her cast/attack indistinguishable from idle (NativeHeroPoseSanity).
    omatsu={idle='Idle_Loop',attack='Spell_Simple_Shoot',critical='Spell_Simple_Shoot',impact=0.3},
    oichi={idle='Idle_Talking_Loop',attack='Spell_Simple_Shoot',critical='Spell_Simple_Shoot',impact=0.3},
    -- Lane minions (templates/world_units.lua): weapons hang on their backs like the heroes'.
    minion_ashigaru={idle='Idle_Loop',attack='Sword_Attack',critical='Sword_Attack',impact=0.5},
    minion_archer={idle='Idle_Loop',attack='Spell_Simple_Shoot',critical='Spell_Simple_Shoot',impact=0.3},
    minion_siege={idle='Idle_Loop',attack='Punch_Cross',critical='Punch_Cross',impact=0.4},
    -- Neutral jungle camps (templates/world_units.lua): kappa members, tengu guardians, oni epic.
    jungle_kappa={idle='Idle_Loop',attack='Punch_Cross',critical='Punch_Cross',impact=0.4},
    jungle_tengu={idle='Idle_Loop',attack='Sword_Attack',critical='Sword_Attack',impact=0.5},
    jungle_oni={idle='Idle_Loop',attack='Sword_Attack',critical='Punch_Cross',impact=0.65},
  }
  -- Held weapons: a separate generated static model (omfue/SourceArt/Structures/TripoStaticV1/<mesh>/,
  -- import_character_sources --source-set tripo_static_v1) attached to a hand bone of the empty-handed body,
  -- so the rig and its clips never deform a weapon. Grip placement is relative to the hand bone
  -- (Om.Runtime.NativeHeroEquipment checks the weapon follows that hand through idle/move/attack/cast).
  -- Every model is a Tripo export laid along X (100 cm long, centred on X=0, the grip end at -X) with its
  -- pivot on the bottom face (center_z = half its height). length scales it to a share of the 100 cm body;
  -- grip is where the fist closes, as a fraction of the length from the centre (-0.5 = the very end).
  -- Blades, hafts and bows run through the fist; a gun lies along the fingers so its barrel points where the arm aims.
  -- material names a build-time colour variant (manifest recolors, scripts/recolor_characters.lua): the dark iron
  -- axe and black-lacquered spear are lightened ('bright') and made a little larger so they read at MOBA distance.
  local weapons={
    saika_magoichi={mesh='weapon_matchlock',hand='hand_r',length=1.0,grip=-0.30,center_z=13.3,along='fingers'},
    date_masamune={mesh='weapon_katana',hand='hand_r',length=0.95,grip=-0.40,center_z=6.0},
    asai_nagamasa={mesh='weapon_katana',hand='hand_r',length=0.95,grip=-0.40,center_z=6.0},
    murakami_yoshikiyo={mesh='weapon_war_axe',hand='hand_r',length=1.1,grip=-0.35,center_z=20.4,material={slot='tripo_mat_a262ff90',variant='bright'}},
    honda_tadakatsu={mesh='weapon_tonbokiri',hand='hand_r',length=2.2,grip=-0.15,center_z=3.3,material={slot='tripo_mat_c3c8b4ec',variant='bright'}},
    sanada_yukimura={mesh='weapon_jumonji',hand='hand_r',length=1.7,grip=-0.15,center_z=10.3},
    inahime={mesh='weapon_yumi',hand='hand_l',length=1.25,grip=-0.05,center_z=9.0},
    tamako={mesh='weapon_hankyu',hand='hand_l',length=0.9,grip=0,center_z=10.1},
  }
  -- Hand frame of the generated UE5 Mannequin rig: fingers along -Y, the fist's bar axis along +Z (hand_r)
  -- or -Z (hand_l). The weapon's X axis runs through the fist (pitch +-90) and its grip point sits in the palm.
  local palm={0,-7,0}
  local function grip_transform(w)
    local s,gx=w.length,w.grip*100
    -- Along the fingers (-Y): yaw -90 maps the weapon's X onto -Y; the grip point lands in the palm.
    if w.along=='fingers'then return {palm[1],palm[2]+s*gx,palm[3]-s*w.center_z},{0,-90,0}end
    if w.hand=='hand_r'then return {palm[1]+s*w.center_z,palm[2],palm[3]-s*gx},{90,0,0}end
    return {palm[1]-s*w.center_z,palm[2],palm[3]+s*gx},{-90,0,0}
  end

  -- Heroes whose Tripo rig and retargeted clips are imported and verified.
  local imported={saika_magoichi=true,date_masamune=true,omatsu=true,oichi=true,inahime=true,asai_nagamasa=true,
    murakami_yoshikiyo=true,tamako=true,honda_tadakatsu=true,sanada_yukimura=true,minion_ashigaru=true,minion_archer=true,minion_siege=true,
    jungle_kappa=true,jungle_tengu=true,jungle_oni=true}
  -- Build-time colour variants (manifest recolor block, scripts/recolor_characters.lua) whose texture and
  -- material are imported: the variant material replaces the body's single section. Oichi's pale pink
  -- read as white beside Omatsu at MOBA distance.
  local recolored={oichi='sakura'}
  -- Bodies with ally/enemy colour variants of their off-white cloth (manifest recolors), chosen at runtime
  -- by each unit's disclosed relation to the local player.
  -- Enabled together with the codegen/runtime support for animated relation materials.
  local team_colored={minion_ashigaru=true,minion_archer=true,minion_siege=true}
  local M={}
  function M.profile(hero_id,previous)
    local definition=definitions[hero_id]
    if not(definition and imported[hero_id])then return nil end
    local function clip(name)
      local object=hero_id..'_animation_library_Anim_Armature_'..name
      return root..'TripoRetargetV1/'..hero_id..'/'..object..'.'..object
    end
    local profile={mesh_asset=root..'TripoV1/'..hero_id..'/'..hero_id..'.'..hero_id,scale=1.0,material_slots={},animation_assets={},
      attachments={},state_slots={idle_2='idle_2',idle_3='idle_3'}}
    local animations={idle={definition.idle,true},idle_2={'Idle_Loop',true},idle_3={'Idle_Talking_Loop',true},
      move={'Jog_Fwd_Loop',true},attack={definition.attack,false,definition.impact},critical={definition.critical,false,definition.impact},
      cast={definition.cast or'Spell_Simple_Shoot',false},hit={'Hit_Chest',false},death={'Death01',false},
      recall={'Spell_Simple_Idle_Loop',true},respawn={'Spell_Simple_Exit',false},emote={'Dance_Loop',true}}
    for slot,binding in pairs(definition.extra or{})do animations[slot]=binding end
    for slot,binding in pairs(animations)do profile.animation_assets[slot]={asset=clip(binding[1]),loop_animation=binding[2],impact_seconds=binding[3]}end
    for state,slot in pairs(definition.state_slots or{})do profile.state_slots[state]=slot end
    if recolored[hero_id]then
      local material=hero_id..'_mat_'..recolored[hero_id]
      profile.material_sections={['0']=root..'TripoVariantsV1/'..hero_id..'/'..material..'.'..material}
    end
    if team_colored[hero_id]then
      local function variant(team)local material=hero_id..'_mat_'..team;return {['0']=root..'TripoVariantsV1/'..hero_id..'/'..material..'.'..material}end
      profile.ally_material_sections=variant('ally');profile.enemy_material_sections=variant('enemy')
    end
    local weapon=weapons[hero_id]
    if weapon then
      local mesh='/Game/Omoba/Structures/TripoStaticV1/'..weapon.mesh..'/'..weapon.mesh..'.'..weapon.mesh
      local translation,rotation=grip_transform(weapon)
      profile.attachments[1]={id='weapon',bone=weapon.hand,mesh_asset=mesh,scale=weapon.length,
        translation_cm=translation,rotation_deg=rotation,normalize_reference_bone_scale=true}
      if weapon.material then
        local name=weapon.mesh..'_mat_'..weapon.material.variant
        profile.attachments[1].material_slots={[weapon.material.slot]='/Game/Omoba/Structures/TripoStaticVariantsV1/'..weapon.mesh..'/'..name..'.'..name}
      end
    end
    -- The hero's authored size survives; the generated mesh is normalized to it at runtime.
    if previous and previous.height_world_units then profile.height_world_units=previous.height_world_units end
    return profile
  end
  M.definitions=definitions
  M.weapons=weapons
  M.imported=imported
  M.recolored=recolored
  M.team_colored=team_colored
  return M
end
