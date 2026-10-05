-- Data-only prototypes, evaluated by both Rust and Unreal content generators.
-- No module registration or character-specific native implementation required.
return function(ctx)
  local definitions = {
    {
      id='training_vanguard', name='磐岩先鋒', title='近戰前排',
      strength=30, agility=14, intelligence=16, primary='strength',
      hp=900, mana=240, damage=58, armor=4.0, range=180.0, speed=300.0,
      growth={strength_per_level=3.2,agility_per_level=1.4,intelligence_per_level=1.5,
        damage_per_level=3.0,hp_per_level=80.0,mana_per_level=22.0},
      skills={
        {id='vanguard_strike',name='破岩擊',kind='damage',amounts={65,100,135,170},
          cooldowns={6,5.5,5,4.5},range=300,damage_kind='physical'},
        {id='vanguard_recover',name='整備',kind='heal_self',amounts={110,165,220,275},
          cooldowns={14,13,12,11},mana_buff={stat='mana_bonus',values={60,90,120,150},durations={6,7,8,9}}},
        {id='vanguard_crush',name='磐岩重擊',kind='damage',amounts={155,240,325,410},
          cooldowns={65,60,55,50},range=330,damage_kind='physical',ultimate=true},
        {id='vanguard_resolve',name='磐岩突進',kind='dash_to_point',range=450,
          cooldowns={28,26,24,22}},
      },
    },
    {
      id='training_ranger', name='逐風遊俠', title='遠程物理輸出',
      strength=16, agility=30, intelligence=16, primary='agility',
      hp=520, mana=280, damage=62, armor=1.0, range=650.0, speed=325.0,
      growth={strength_per_level=1.6,agility_per_level=3.4,intelligence_per_level=1.5,
        damage_per_level=3.8,hp_per_level=48.0,mana_per_level=24.0},
      skills={
        {id='ranger_shot',name='穿風箭',kind='damage',amounts={90,135,180,225},
          slows={0.25,0.30,0.35,0.40},slow_durations={2,2.25,2.5,2.75},
          cooldowns={5,4.5,4,3.5},range=750,damage_kind='physical'},
        {id='ranger_patch',name='野外包紮',kind='heal_self',amounts={55,85,115,145},
          cooldowns={20,18,16,14},mana_buff={stat='mana_regen_constant',values={2,3,4,5},durations={6,7,8,9}}},
        {id='ranger_volley',name='箭雨',kind='area_damage',amounts={110,160,210,260},
          radii={220,240,260,280},cooldowns={11,10,9,8},range=700,damage_kind='physical'},
        {id='ranger_finisher',name='逐風終箭',kind='damage',amounts={210,310,410,510},
          cooldowns={75,65,55,45},range=850,damage_kind='physical',ultimate=true},
      },
    },
  }
  local result={heroes={},abilities={},ability_policies={},ability_learning={}}
  for _,entry in ipairs(definitions) do
    local hero={id=entry.id,ue={native_only=true},display_name=entry.name,title=entry.title,
      background='使用共用宣告式效果與生成呈現的原創 MOBA 原型。',abilities={},
      strength=entry.strength,agility=entry.agility,intelligence=entry.intelligence,
      primary_attribute=entry.primary,base_hp=entry.hp,base_mana=entry.mana,
      base_damage=entry.damage,base_armor=entry.armor,attack_range=entry.range,
      move_speed=entry.speed,turn_speed=540.0,level_growth=entry.growth}
    local healing,attacks={},{}
    for _,skill in ipairs(entry.skills) do
      local healing_skill=skill.kind=='heal_self'
      local area_skill=skill.kind=='area_damage'
      local dash_skill=skill.kind=='dash_to_point'
      local amount_key=healing_skill and 'heal' or 'damage'
      local ability={id=skill.id,ue={native_only=true},display_name=skill.name,
        description=dash_skill and '沿可通行直線立即移動到指定位置；受阻或超距不施放。' or (healing_skill and '恢復自身生命值。' or (area_skill and '對指定位置附近的敵人造成物理傷害。' or '對單一指定敵人造成物理傷害。')),
        ability_type=skill.ultimate and 'ultimate' or 'active',target_type=healing_skill and 'none' or ((area_skill or dash_skill) and 'point' or 'unit'),
        cast_type='instant',max_level=4,levels={},extras={[amount_key]=skill.amounts},
        effects={{kind=skill.kind,amount_key=amount_key,damage_kind=skill.damage_kind}}}
      if dash_skill then ability.extras={};ability.effects={{kind='dash_to_point'}} end
      if skill.mana_buff then
        ability.extras.mana_buff_value=skill.mana_buff.values
        ability.extras.mana_buff_duration=skill.mana_buff.durations
        ability.effects[#ability.effects+1]={kind='mana_buff_self',stat=skill.mana_buff.stat,
          value_key='mana_buff_value',duration_key='mana_buff_duration'}
        ability.description=ability.description..(skill.mana_buff.stat=='mana_bonus'
          and ' 暫時提高魔力容量，不會立即補魔。' or ' 短時間提高自然回魔速率。')
      end
      if skill.slows then
        ability.extras.slow_reduction=skill.slows;ability.extras.slow_duration=skill.slow_durations
        ability.effects[#ability.effects+1]={kind='slow_enemy',reduction_key='slow_reduction',duration_key='slow_duration'}
        ability.description=ability.description..' 命中附加短暫減速。'
      end
      if area_skill then ability.extras.radius=skill.radii;ability.effects[1].radius_key='radius' end
      for rank=1,4 do
        ability.levels[rank]={cooldown=skill.cooldowns[rank],mana_cost=40+rank*5,
          range=skill.range,required_hero_level=skill.ultimate and ({1,6,11,16})[rank] or 1}
      end
      hero.abilities[#hero.abilities+1]=skill.id
      result.abilities[#result.abilities+1]=ability
      local policies=healing_skill and healing or attacks
      policies[#policies+1]={ability=skill.id,intent=healing_skill
        and {kind='self_heal',below_hp_per_mille=600} or (dash_skill and {kind='approach_enemy_point',min_distance=300} or (area_skill and {kind='enemy_point',radius_key='radius',min_targets=1} or {kind='enemy_unit'}))}
    end
    -- Explicit per-content priority; the planner resolves actual loadout slots.
    for _,policies in ipairs({healing,attacks}) do
      for _,policy in ipairs(policies) do result.ability_policies[#result.ability_policies+1]=policy end
    end
    for rank=1,4 do
      for _,ability in ipairs(hero.abilities) do
        result.ability_learning[#result.ability_learning+1]={ability=ability,rank=rank}
      end
    end
    result.heroes[#result.heroes+1]=hero
  end
  return result
end
