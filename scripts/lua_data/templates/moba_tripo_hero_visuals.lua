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
    inahime={idle='Idle_Loop',attack='Mx_Throw_Short',critical='Mx_Throw_Short',impact=0.6},
    tamako={idle='Idle_Loop',attack='Mx_Throw_Short',critical='Mx_Throw_Short',impact=0.6},
    -- A raised casting hand at rest leaves her cast/attack indistinguishable from idle (NativeHeroPoseSanity).
    omatsu={idle='Idle_Loop',attack='Mx_Throw_Short',critical='Mx_Throw_Short',impact=0.6},
    oichi={idle='Idle_Talking_Loop',attack='Mx_Throw_Short',critical='Mx_Throw_Short',impact=0.6},
    -- Empty-handed H3.1 rig (finger metacarpals; explicit retarget chains in its manifest).
    tokugawa_ieyasu={idle='Idle_Loop',cast='Mx_Throw_Short',attack='Mx_Sword_Inward',critical='Mx_Sword_Outward',impact=0.3,contact=0.59,critical_impact=0.72,critical_contact=0.87,
      extra={death={'Mx_Death_Back',false},['ability_a01_heaven_net']={'Mx_Throw',false},['ability_a01_mikawa_valor']={'Mx_Charge',false},['ability_a01_thunder_blade']={'Mx_Sword_Slash',false},['ability_a01_tosho_gongen']={'Mx_Cast_Two',false}}},
    yamagata_masakage={idle='Idle_Loop',cast='Mx_Throw_Short',attack='Mx_Sword_Inward',critical='Mx_Sword_Slash',impact=0.3,contact=0.59,critical_impact=0.47,critical_contact=0.64,
      extra={['ability_b20_red_charge']={'Mx_Battlecry',false},['ability_b20_rock_rush']={'Mx_Throw',false},['ability_b20_ruin_wave']={'Mx_Axe_Smash',false}}},
    okuma_tomohide={idle='Mx_Axe2_Idle',idle_2='Mx_Axe2_Idle',idle_3='Mx_Axe2_Idle',move='Mx_Axe2_Jog',hit='Mx_Axe2_Hit',cast='Mx_Axe2_Slash',attack='Mx_Axe2_Swing',critical='Mx_Axe2_Swing',impact=0.5,contact=0.67,critical_impact=0.5,critical_contact=0.67,
      extra={['ability_b26_blood_fury']={'Mx_Axe2_Battlecry',false},['ability_b26_charging_wave']={'Mx_Axe2_Channel',false},['ability_b26_release_wave']={'Mx_Axe2_Swing',false},['ability_b26_unstoppable']={'Mx_Axe2_Charge',false}}},
    sasaki_kojiro={idle='Idle_Loop',cast='Mx_Throw_Short',attack='Mx_Sword_Inward',critical='Mx_Sword_Outward',impact=0.3,contact=0.59,critical_impact=0.72,critical_contact=0.87,
      extra={['ability_c22_dream_mist']={'Mx_Sword_Draw',false},['ability_c22_flying_swallow']={'Mx_Sword_Slash',false},['ability_c22_ganryu']={'Mx_Battlecry',false},['ability_c22_swallow_nest']={'Mx_Cast_Two',false}}},
    otani_yoshitsugu={idle='Idle_Loop',cast='Mx_Throw_Short',attack='Mx_Throw_Short',critical='Mx_Throw_Short',impact=0.6,
      extra={death={'Mx_Death_Back',false},['ability_a12_awe']={'Mx_Cast_Two',false},['ability_a12_insight_flash']={'Mx_Cast_Two',false},['ability_a12_mystic_arrow']={'Mx_Throw',false},['ability_a12_mystic_pierce']={'Mx_Battlecry',false},['ability_a12_mystic_possession']={'Mx_Channel',false}}},
    maeda_toshiie={idle='Idle_Loop',cast='Mx_Throw_Short',attack='Mx_Spear_Thrust',critical='Mx_Sweep',impact=0.6,contact=0.74,critical_impact=0.35,critical_contact=0.58,
      extra={['ability_a14_divine_strike']={'Mx_Spear_Upward',false},['ability_a14_impact_spear']={'Mx_Sweep',false},['ability_a14_spear_fury']={'Mx_Battlecry',false}}},
    kojima_sadaoki={idle='Mx_Axe2_Idle',idle_2='Mx_Axe2_Idle',idle_3='Mx_Axe2_Idle',move='Mx_Axe2_Jog',hit='Mx_Axe2_Hit',cast='Mx_Axe2_Slash',attack='Mx_Axe2_Swing',critical='Mx_Axe2_Swing',impact=0.5,contact=0.67,critical_impact=0.5,critical_contact=0.67,
      extra={death={'Mx_Death_Back',false},['ability_b27_fearless_charge']={'Mx_Axe2_Battlecry',false},['ability_b27_horsemanship']={'Mx_Axe2_Charge',false},['ability_b27_mighty_strike']={'Mx_Axe2_Slash',false},['ability_b27_rending_blade']={'Mx_Axe2_Swing',false}}},
    shimazu_yoshihiro={idle='Mx_Axe2_Idle',idle_2='Mx_Axe2_Idle',idle_3='Mx_Axe2_Idle',move='Mx_Axe2_Jog',hit='Mx_Axe2_Hit',cast='Mx_Axe2_Slash',attack='Mx_Axe2_Swing',critical='Mx_Axe2_Swing',impact=0.5,contact=0.67,critical_impact=0.5,critical_contact=0.67,
      extra={['ability_c03_iron_hammer']={'Mx_Axe2_Slash',false},['ability_c03_iron_shirt']={'Mx_Axe2_Battlecry',false},['ability_c03_quake_axe']={'Mx_Axe2_Swing',false}}},
    maeda_keiji={idle='Idle_Loop',cast='Mx_Throw_Short',attack='Mx_Spear_Thrust',critical='Mx_Sweep',impact=0.6,contact=0.74,critical_impact=0.35,critical_contact=0.58,
      extra={death={'Mx_Death_Back',false},['ability_c05_flash_thunder']={'Mx_Channel',false},['ability_c05_matsukaze']={'Mx_Battlecry',false},['ability_c05_thunder_dance']={'Mx_Spear_Upward',false}}},
    nohime={idle='Mx_Pistols_Idle',idle_2='Mx_Pistols_Idle',idle_3='Mx_Pistols_Idle',move='Mx_Pistols_Jog',hit='Mx_Pistols_Hit',cast='Mx_Pistol_Fire_R',attack='Mx_Pistol_Fire_R',critical='Mx_Pistol_Fire_R',impact=0.2,
      extra={death={'Mx_Death_Back',false},attack_left={'Mx_Pistol_Fire_L',false,0.2},critical_left={'Mx_Pistol_Fire_L',false,0.2},['ability_a26_powder_kick']={'Mx_Spin_Kick',false},['ability_a26_scorching_bomb']={'Mx_Throw',false},['ability_a26_volley']={'Mx_Pistols_Volley',false}}},
    -- Lane minions (templates/world_units.lua): weapons hang on their backs like the heroes'.
    minion_ashigaru={idle='Idle_Loop',attack='Sword_Attack',critical='Sword_Attack',impact=0.5},
    minion_archer={idle='Idle_Loop',attack='Mx_Throw_Short',critical='Mx_Throw_Short',impact=0.6},
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
  -- flip puts the tip on the thumb side: the hand bones of some rigs point the other way, and the sword clips need the
  -- blade ahead of the thumb to swing it at the target.
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
    -- The eleventh-of-November heroes share the existing weapon models.
    sasaki_kojiro={mesh='weapon_katana',hand='hand_r',length=1.15,grip=-0.40,center_z=6.0,flip=true},
    tokugawa_ieyasu={mesh='weapon_katana',hand='hand_r',length=0.95,grip=-0.40,center_z=6.0,flip=true},
    yamagata_masakage={mesh='weapon_katana',hand='hand_r',length=0.95,grip=-0.40,center_z=6.0,flip=true},
    -- Held in both hands (Mx_Axe2_*): the right fist is up the haft, the head on its thumb side, with room for the left below it.
    okuma_tomohide={mesh='weapon_war_axe',hand='hand_r',length=1.1,grip=-0.15,center_z=20.4,flip=true},
    maeda_toshiie={mesh='weapon_jumonji',hand='hand_r',length=1.7,grip=-0.15,center_z=10.3,flip=true},
    maeda_keiji={mesh='weapon_tonbokiri',hand='hand_r',length=2.2,grip=-0.15,center_z=3.3,flip=true},
    kojima_sadaoki={mesh='weapon_war_axe',hand='hand_r',length=1.1,grip=-0.15,center_z=20.4,flip=true},
    shimazu_yoshihiro={mesh='weapon_war_axe',hand='hand_r',length=1.25,grip=-0.15,center_z=20.4,flip=true},
    -- A pistol in each hand, fired in turn (attack_left, Mx_Pistol_Fire_R and _L).
    nohime={mesh='weapon_flintlock',hand='hand_r',length=0.32,grip=-0.32,center_z=20,along='fingers',both_hands=true},
  }
  -- Hand frame of the generated UE5 Mannequin rig: fingers along -Y, the fist's bar axis along +Z (hand_r)
  -- or -Z (hand_l). The weapon's X axis runs through the fist (pitch +-90) and its grip point sits in the palm.
  local palm={0,-7,0}
  local function grip_transform(w)
    local s,gx=w.length,w.grip*100
    -- Along the fingers (-Y): yaw -90 maps the weapon's X onto -Y; the grip point lands in the palm.
    if w.along=='fingers'then return {palm[1],palm[2]+s*gx,palm[3]-s*w.center_z},{0,-90,0}end
    if w.hand=='hand_r' and not w.flip then return {palm[1]+s*w.center_z,palm[2],palm[3]-s*gx},{90,0,0}end
    return {palm[1]-s*w.center_z,palm[2],palm[3]+s*gx},{-90,0,0}
  end

  -- First-version bodies: shared library clips only, melee or ranged by the hero's attack range.
  local first_version={
    tachibana_ginchiyo='melee',
    inaba_ittetsu='ranged',
    kakizaki_kageie='melee',
    torii_mototada='ranged',
    sakuma_morimasa='melee',
    akechi_mitsuhide='melee',
    ukita_hideie='ranged',
    shima_sakon='melee',
    saito_yoshitatsu='melee',
    yagyu_muneyoshi='melee',
    kuroda_yoshitaka='ranged',
    saito_dosan='ranged',
    sengoku_hidehisa='melee',
    niwa_nagahide='ranged',
    maeno_nagayasu='ranged',
    takeda_katsuyori='melee',
    fuma_kotaro='ranged',
    akechi_hidemitsu='melee',
    honda_masanobu='ranged',
    saito_tomonobu='melee',
    mori_motonari='ranged',
    kikuhime='melee',
    sanada_masayuki='ranged',
    koihime='ranged',
    takeda_shingen='melee',
    kuki_yoshitaka='melee',
  }
  for hero,kind in pairs(first_version)do
    definitions[hero]=kind=='ranged' and {idle='Idle_Loop',attack='Spell_Simple_Shoot',critical='Spell_Simple_Shoot',impact=0.5}
      or {idle='Idle_Loop',attack='Sword_Attack',critical='Sword_Attack',impact=0.65}
  end
  -- Heroes whose Tripo rig and retargeted clips are imported and verified.
  local imported={saika_magoichi=true,date_masamune=true,omatsu=true,oichi=true,inahime=true,asai_nagamasa=true,
    murakami_yoshikiyo=true,tamako=true,honda_tadakatsu=true,sanada_yukimura=true,tokugawa_ieyasu=true,
    yamagata_masakage=true,okuma_tomohide=true,sasaki_kojiro=true,otani_yoshitsugu=true,maeda_toshiie=true,nohime=true,kojima_sadaoki=true,shimazu_yoshihiro=true,maeda_keiji=true,minion_ashigaru=true,minion_archer=true,minion_siege=true,
    jungle_kappa=true,jungle_tengu=true,jungle_oni=true}
  for hero in pairs(first_version)do imported[hero]=true end
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
      -- Further motions (scripts/mixamo_clips.lua) live beside the library's.
      if name:sub(1,3)=='Mx_' then local added=hero_id..'_'..name;return root..'AddedClipsV1/'..hero_id..'/'..added..'.'..added end
      local object=hero_id..'_animation_library_Anim_Armature_'..name
      return root..'TripoRetargetV1/'..hero_id..'/'..object..'.'..object
    end
    local profile={mesh_asset=root..'TripoV1/'..hero_id..'/'..hero_id..'.'..hero_id,scale=1.0,material_slots={},animation_assets={},
      attachments={},state_slots={idle_2='idle_2',idle_3='idle_3'}}
    local animations={idle={definition.idle,true},idle_2={definition.idle_2 or'Idle_Loop',true},idle_3={definition.idle_3 or'Idle_Talking_Loop',true},
      move={definition.move or'Jog_Fwd_Loop',true},attack={definition.attack,false,definition.impact,definition.contact},critical={definition.critical,false,definition.critical_impact or definition.impact,definition.critical_contact},
      cast={definition.cast or'Spell_Simple_Shoot',false},hit={definition.hit or'Hit_Chest',false},death={'Death01',false},
      recall={'Spell_Simple_Idle_Loop',true},respawn={'Spell_Simple_Exit',false},emote={'Dance_Loop',true}}
    for slot,binding in pairs(definition.extra or{})do animations[slot]=binding end
    for slot,binding in pairs(animations)do profile.animation_assets[slot]={asset=clip(binding[1]),loop_animation=binding[2],impact_seconds=binding[3],contact_seconds=binding[4]}end
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
    -- The left hand's bar axis is the right's reversed: the same gun upright turns over about its length.
    if weapon and weapon.both_hands then
      local translation=grip_transform(weapon)
      profile.attachments[2]={id='weapon_l',bone='hand_l',mesh_asset=profile.attachments[1].mesh_asset,scale=weapon.length,
        translation_cm={translation[1],translation[2],-translation[3]},rotation_deg={0,-90,180},normalize_reference_bone_scale=true}
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
