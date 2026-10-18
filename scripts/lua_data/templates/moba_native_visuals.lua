-- Unreal-only presentation. Gameplay IDs, stats and authoritative behavior remain in hero templates.
return function(ctx)
  local root='/Game/Omoba/Characters/'
  local function material(id)return root..'MaterialsV1/materials/M_'..id..'.M_'..id end
  local function clip(rig,name)
    local object=rig..'_animation_library_Anim_Armature_'..name
    return root..'RetargetV4/'..rig..'/'..object..'.'..object
  end
  local definitions={
    date_masamune={rig='male_ranger',head='male',idle='Sword_Idle',attack='Sword_Attack',critical='Sword_Attack',impact=0.65,
      materials={MI_Ranger='ranger',MI_Regular_Male='regular_male'},
      -- Source meters already import as centimeters: keep the original 44.8cm short sword.
      -- Palm center is halfway from the wrist to the four finger roots (~10.7cm).
      -- The source grip center is (0,0,-5cm), hence the +5cm pivot correction.
      equipment={id='sword',mesh='weapon_sword',bone='hand_r',scale=1.0,translation_cm={-0.4,-5.4,5}}},
    training_luminary={rig='female_peasant',head='female',idle='Spell_Simple_Idle_Loop',attack='Spell_Simple_Shoot',critical='Spell_Simple_Shoot',impact=0.3,
      materials={MI_Peasant='peasant'}},
    training_apprentice={rig='female_peasant',head='female',idle='Idle_Talking_Loop',attack='Spell_Simple_Shoot',critical='Spell_Simple_Shoot',impact=0.3,
      materials={MI_Peasant='peasant'}},
    training_vanguard={rig='male_peasant',head='male',idle='Idle_Loop',attack='Punch_Jab',critical='Punch_Cross',impact=0.35,
      materials={MI_Peasant='peasant',MI_Regular_Male='regular_male'}},
    training_ranger={rig='male_ranger',head='male',idle='Idle_Loop',attack='Spell_Simple_Shoot',critical='Spell_Simple_Shoot',impact=0.3,
      materials={MI_Ranger='ranger',MI_Regular_Male='regular_male'}},
    training_support={rig='female_ranger',head='female',idle='Idle_Torch_Loop',attack='Spell_Simple_Shoot',critical='Spell_Simple_Shoot',impact=0.3,
      materials={MI_Ranger='ranger',MI_Regular_Female='regular_female'}},
  }
  local result={}
  for hero,definition in pairs(definitions)do
    local rig=definition.rig
    local profile={mesh_asset=root..'QuaterniusV2/'..rig..'/'..rig..'.'..rig,scale=1.0,
      material_slots={},animation_assets={},attachments={},state_slots={idle_2='idle_2',idle_3='idle_3'}}
    for slot,id in pairs(definition.materials)do profile.material_slots[slot]=material(id)end
    local animations={idle={definition.idle,true},idle_2={'Idle_Talking_Loop',true},idle_3={'Spell_Simple_Idle_Loop',true},
      move={'Jog_Fwd_Loop',true},attack={definition.attack,false,definition.impact},critical={definition.critical,false,definition.impact},
      cast={'Spell_Simple_Shoot',false},hit={'Hit_Chest',false},death={'Death01',false},
      recall={'Spell_Simple_Idle_Loop',true},respawn={'Spell_Simple_Exit',false},emote={'Dance_Loop',true}}
    for slot,binding in pairs(animations)do profile.animation_assets[slot]={asset=clip(rig,binding[1]),loop_animation=binding[2],impact_seconds=binding[3]}end
    if definition.head then
      local gender=definition.head;local name=gender..'_head_neck'
      profile.attachments[1]={id='head',mesh_asset=root..'RigidPartsV1/'..name..'.'..name,bone='Head',normalize_reference_bone_scale=true,
        material_slots={MI_Eyes=material('eyes')}}
      profile.attachments[1].material_slots[gender=='male'and'MI_Hair_1'or'MI_Hair_2']=material(gender=='male'and'hair_1'or'hair_2')
      profile.attachments[1].material_slots[gender=='male'and'MI_Superhero_Male'or'MI_Superhero_Female']=material('superhero_'..gender)
    end
    if definition.equipment then
      local item=definition.equipment
      profile.attachments[#profile.attachments+1]={id=item.id,bone=item.bone,
        mesh_asset='/Game/Omoba/WorldModelsV1/SM_'..item.mesh..'.SM_'..item.mesh,
        scale=item.scale,translation_cm=item.translation_cm,normalize_reference_bone_scale=true}
    end
    result[hero]=profile
  end
  return result
end
