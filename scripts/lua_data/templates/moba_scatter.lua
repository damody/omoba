-- Append-only optional loadout. Existing ranger and volley remain unchanged.
return function(ctx)
  return {
    ability={id='ranger_scatter',ue={native_only=true},display_name='散射箭',
      description='朝指定方向射出扇形箭矢，對自身前方扇形內的存活敵人造成一次物理傷害。原地無法指定方向；指向點須在射程內，箭矢延伸至完整射程。',
      ability_type='active',target_type='point',cast_type='instant',max_level=4,
      levels={{range=700,cooldown=11,mana_cost=45},{range=700,cooldown=10,mana_cost=50},
        {range=700,cooldown=9,mana_cost=55},{range=700,cooldown=8,mana_cost=60}},
      extras={damage={110,160,210,260},half_angle={math.pi/6,math.pi/6,math.pi/6,math.pi/6}},
      effects={{kind='cone_damage',amount_key='damage',half_angle_key='half_angle',damage_kind='physical'}}},
    hero={id='training_scatter_ranger',test_only=true,ue={native_only=true},display_name='逐風散射手',title='方向扇形配裝',
      background='逐風遊俠的方向散射配裝，使用共用資料效果與原生呈現。',
      abilities={'ranger_shot','ranger_patch','ranger_scatter','ranger_finisher'},
      strength=16,agility=30,intelligence=16,primary_attribute='agility',
      attack_range=650,base_damage=62,base_armor=1,base_hp=520,base_mana=280,move_speed=325,turn_speed=540,
      level_growth={strength_per_level=1.6,agility_per_level=3.4,intelligence_per_level=1.5,
        damage_per_level=3.8,hp_per_level=48,mana_per_level=24}},
  }
end
