-- 由 Lua 英雄／技能宣告產生 base_content 的 Rust FFI 註冊語句。
local content_root = assert(arg[1], 'content root is required')
local output_mode = arg[2] or 'registry'
assert(output_mode == 'registry' or output_mode == 'ids' or output_mode == 'units', 'unknown output mode')
-- arg[3]: a hero of hero_dlls.lua (its own DLL registers only that hero); absent, base_content
-- registers every hero except those.
local only_hero = arg[3] ~= '' and arg[3] or nil
local source = debug.getinfo(1,'S').source:sub(2)
package.path = (source:match('^(.*)[/\\]') or '.') .. '/?.lua;' .. package.path
local own_dll = {}
for _, row in ipairs(require('hero_dlls')) do own_dll[row.hero] = row end
assert(not only_hero or own_dll[only_hero], 'not a hero of hero_dlls.lua: ' .. tostring(only_hero))
-- Whether this output registers HERO's abilities and unit script.
local function selected(hero_id)
  if only_hero then return hero_id == only_hero end
  return not own_dll[hero_id]
end
-- A v3 test hero (scripts/v3_test_heroes.lua) registers through its generated crate
-- (scripts/heroes/<v3_crate>), which tests link next to base_content; no output here registers it.
local function v3_crate_hero(hero) return hero.v3_crate ~= nil end
local context = require('content_builder').new(content_root)
local function load_builder(name) return context.include('templates/'..name..'.lua') end

local function identifier(value, label)
  assert(type(value) == 'string' and value:match('^[A-Za-z_][A-Za-z0-9_]*$'),
    label .. ' must be a Rust identifier')
  return value
end

local heroes = load_builder('heroes')
local abilities = load_builder('abilities')
local active_abilities = {}
-- Same authored numeric envelope as omoba-content-model. Reject positive
-- values that become zero in Q10, rather than silently making casts free.
local function authored_number(value)
  if type(value)~='number' then return nil end
  -- The shared Rust content schema stores f32. Validate that same value,
  -- including boundary rounding, rather than Lua's wider double.
  return string.unpack('f',string.pack('f',value))
end
local function scalar(value)
  value=authored_number(value)
  return value~=nil and (value==0 or (value>=1/1024 and value<=1000000))
end
local function validate_levels(ability)
  if ability.tombstone then return end
  if ability.max_level==nil then ability.max_level=4 end
  assert(type(ability.max_level)=='number' and ability.max_level>=1 and ability.max_level<=255
    and ability.max_level%1==0 and type(ability.levels)=='table' and #ability.levels==ability.max_level,
    ability.id..': levels must match nonzero max_level')
  local previous=1
  for rank,level in ipairs(ability.levels) do
    assert(type(level)=='table',ability.id..': invalid rank '..rank)
    local required=level.required_hero_level
    if required==nil then required=1 end
    assert(type(required)=='number' and required%1==0 and required>=previous and required<=25,
      ability.id..': rank '..rank..' required_hero_level must be monotonic in 1..=25')
    previous=required
    assert(level.cast_interrupt_on_damage==nil or type(level.cast_interrupt_on_damage)=='boolean',
      ability.id..': cast_interrupt_on_damage must be boolean')
    assert(not level.cast_interrupt_on_damage or (type(level.cast_time)=='number' and authored_number(level.cast_time)>=1/1024 and authored_number(level.cast_time)<=60),
      ability.id..': cast_interrupt_on_damage requires cast_time in [1/1024,60]')
    for _,field in ipairs({'cooldown','mana_cost','cast_time','range'}) do
      local value=level[field]
      if value==nil then value=0 end
      assert(scalar(value),ability.id..': rank '..rank..' '..field..' must be zero or finite in [1/1024,1000000]')
    end
  end
end
for _, ability in ipairs(abilities) do
  local id = identifier(ability.id, 'ability id')
  assert(active_abilities[id] == nil, 'duplicate ability id: ' .. id)
  validate_levels(ability)
  active_abilities[id] = ability
end

local function effect_code(ability, effect)
  if effect.kind=='recovery_channel_self' then
    local amount=identifier(effect.amount_key,ability.id..' channel amount_key')
    local duration=identifier(effect.duration_key,ability.id..' channel duration_key')
    local interval=identifier(effect.interval_key,ability.id..' channel interval_key')
    assert(#ability.effects==1 and ability.cast_type=='channeled' and ability.target_type=='none',ability.id..' channel must be exclusive channeled self')
    local raw={}
    for _,spec in ipairs({{amount,1/1024,1000000},{duration,1/1024,60},{interval,0.25,10}})do
      local rows=ability.extras and ability.extras[spec[1]]
      assert(type(rows)=='table' and #rows==ability.max_level,ability.id..' missing channel rank values')
      local values={};for _,value in ipairs(rows)do
        value=authored_number(value);assert(value and value>=spec[2] and value<=spec[3] and value*1024%1==0,ability.id..' invalid channel scalar')
        values[#values+1]=value*1024
      end;raw[spec[1]]=values
    end
    for rank,level in ipairs(ability.levels)do
      assert((level.cast_time or 0)==0 and raw[duration][rank]>=raw[interval][rank] and raw[duration][rank]%raw[interval][rank]==0,ability.id..' channel requires whole pulses and no preparation')
    end
    return string.format('crate::generic_effects::EffectOp::RecoveryChannelSelf { amount_key: "%s", duration_key: "%s", interval_key: "%s" }',amount,duration,interval)
  end
  if effect.kind=='mana_buff_self' then
    local stats={base_mana_regen={'BaseManaRegen',0,1000000},mana_regen_constant={'ManaRegenConstant',-1000000,1000000},
      mana_regen_constant_unique={'ManaRegenConstantUnique',-1000000,1000000},mana_regen_percentage={'ManaRegenPercentage',-1,16},
      mana_regen_total_percentage={'ManaRegenTotalPercentage',-1,16},mana_bonus={'ManaBonus',-1000000,1000000},extra_mana_bonus={'ExtraManaBonus',-1000000,1000000}}
    local stat=assert(stats[effect.stat],ability.id..' invalid mana buff stat')
    local value=identifier(effect.value_key,ability.id..' mana buff value_key')
    local duration=identifier(effect.duration_key,ability.id..' mana buff duration_key')
    for _,spec in ipairs({{value,stat[2],stat[3],false},{duration,1/1024,60,true}}) do
      local entries=ability.extras and ability.extras[spec[1]]
      assert(type(entries)=='table' and #entries==ability.max_level,ability.id..' mana buff requires per-rank '..spec[1])
      for _,number in ipairs(entries) do
        number=authored_number(number)
        assert(number and number>=spec[2] and number<=spec[3] and (number==0 or math.abs(number)>=1/1024),
          ability.id..' invalid mana buff '..spec[1])
      end
    end
    return string.format('crate::generic_effects::EffectOp::ManaBuffSelf { stat: omoba_content_model::ManaBuffStat::%s, value_key: "%s", duration_key: "%s" }',stat[1],value,duration)
  end
  if effect.kind=='stun_enemy' or effect.kind=='control_enemy' then
    local control=effect.kind=='stun_enemy' and 'stun' or effect.control
    local controls={stun='Stun',root='Root',silence='Silence'}
    assert(type(control)=='string' and controls[control],ability.id..' invalid control_enemy control')
    local duration=identifier(effect.duration_key,ability.id..' duration_key')
    local entries=ability.extras and ability.extras[duration]
    assert(type(entries)=='table' and #entries==ability.max_level,ability.id..' '..effect.kind..' requires per-rank duration')
    for _,value in ipairs(entries) do
      value=authored_number(value)
      assert(value~=nil and value>=1/1024 and value<=60,ability.id..' invalid '..effect.kind..' duration')
    end
    for _,rank in ipairs(ability.levels) do
      local range=authored_number(rank.range)
      assert(range~=nil and range>=1/1024 and range<=10000,ability.id..' '..effect.kind..' requires bounded range')
    end
    if effect.kind=='stun_enemy' then
      return string.format('crate::generic_effects::EffectOp::StunEnemy { duration_key: "%s" }',duration)
    end
    return string.format('crate::generic_effects::EffectOp::ControlEnemy { control: omoba_content_model::ControlEffectKind::%s, duration_key: "%s" }',controls[control],duration)
  end
  if effect.kind=='forced_move_enemy' then
    -- G-4 forced displacement; host bounds are canonical in omoba-content-model.
    local motions={knockback='Knockback',pull='Pull'}
    assert(type(effect.motion)=='string' and motions[effect.motion],ability.id..' forced_move_enemy motion must be knockback or pull')
    local distance=identifier(effect.distance_key,ability.id..' distance_key')
    local duration=identifier(effect.duration_key,ability.id..' duration_key')
    for _,spec in ipairs({{distance,3000},{duration,5}}) do
      local entries=ability.extras and ability.extras[spec[1]]
      assert(type(entries)=='table' and #entries==ability.max_level,ability.id..' forced_move_enemy requires per-rank '..spec[1])
      for _,value in ipairs(entries) do
        value=authored_number(value)
        assert(value~=nil and value>=1/1024 and value<=spec[2],ability.id..' invalid forced_move_enemy '..spec[1])
      end
    end
    return string.format('crate::generic_effects::EffectOp::ForcedMoveEnemy { motion: omoba_content_model::ForcedMotionKind::%s, distance_key: "%s", duration_key: "%s" }',motions[effect.motion],distance,duration)
  end
  if effect.kind=='reveal_area' then
    -- G-4 true-sight area at the point target; bounds mirror omoba-content-model.
    local radius=identifier(effect.radius_key,ability.id..' radius_key')
    local duration=identifier(effect.duration_key,ability.id..' duration_key')
    for _,spec in ipairs({{radius,2000},{duration,30}}) do
      local entries=ability.extras and ability.extras[spec[1]]
      assert(type(entries)=='table' and #entries==ability.max_level,ability.id..' reveal_area requires per-rank '..spec[1])
      for _,value in ipairs(entries) do
        value=authored_number(value)
        assert(value~=nil and value>=1/1024 and value<=spec[2],ability.id..' invalid reveal_area '..spec[1])
      end
    end
    for _,rank in ipairs(ability.levels) do
      local range=authored_number(rank.range)
      assert(range~=nil and range>=1/1024 and range<=10000,ability.id..' reveal_area requires bounded range')
    end
    return string.format('crate::generic_effects::EffectOp::RevealArea { radius_key: "%s", duration_key: "%s" }',radius,duration)
  end
  if effect.kind=='slow_enemy' then
    local reduction=identifier(effect.reduction_key,ability.id..' reduction_key')
    local duration=identifier(effect.duration_key,ability.id..' duration_key')
    for _,spec in ipairs({{reduction,1},{duration,60}}) do
      local key,maximum=spec[1],spec[2]
      local entries=ability.extras and ability.extras[key]
      assert(type(entries)=='table' and #entries==ability.max_level,ability.id..' slow_enemy requires per-rank '..key)
      for _,value in ipairs(entries) do
        value=authored_number(value)
        assert(value~=nil and value>=1/1024 and value<=maximum,ability.id..' invalid slow_enemy '..key)
      end
    end
    for _,rank in ipairs(ability.levels) do
      local range=authored_number(rank.range)
      assert(range~=nil and range>=1/1024 and range<=10000,ability.id..' slow_enemy requires bounded range')
    end
    return string.format('crate::generic_effects::EffectOp::SlowEnemy { reduction_key: "%s", duration_key: "%s" }',reduction,duration)
  end
  if effect.kind=='dash_to_point' then
    assert(#ability.effects==1 and ability.target_type=='point',
      ability.id..' dash_to_point requires one exclusive point effect')
    for _,rank in ipairs(ability.levels) do
      local range=authored_number(rank.range)
      assert(range~=nil and range>=1/1024 and range<=10000,
        ability.id..' dash_to_point requires positive bounded per-rank range')
    end
    return 'crate::generic_effects::EffectOp::DashToPoint'
  end
  if effect.kind=='cleanse_ally' then
    for key in pairs(effect)do assert(key=='kind',ability.id..' ally cleanse has no configurable fields')end
    for _,rank in ipairs(ability.levels)do local range=authored_number(rank.range)
      assert(range and range>=1/1024 and range<=10000,ability.id..' ally cleanse requires bounded range')end
    return 'crate::generic_effects::EffectOp::CleanseAlly'
  end
  local key = identifier(effect.amount_key, ability.id .. ' effect amount_key')
  local levels = ability.extras and ability.extras[key]
  assert(type(levels) == 'table' and #levels == ability.max_level,
    ability.id .. ' effect amount_key ' .. key .. ' needs one extras value per level')
  for _, value in ipairs(levels) do
    assert(scalar(value),
      ability.id .. ' effect amount_key ' .. key .. ' must be zero or finite in [1/1024,1000000]')
  end
  if effect.kind == 'damage' or effect.kind == 'area_damage' or effect.kind=='cone_damage' or effect.kind=='line_damage' or effect.kind=='damage_over_time_enemy' then
    local kinds = {physical = 'Physical', magical = 'Magical', pure = 'Pure'}
    local kind = kinds[effect.damage_kind]
    assert(kind, ability.id .. ' effect has invalid damage_kind')
    assert(type(ability.levels)=='table' and #ability.levels==ability.max_level,
      ability.id..' requires per-rank cast range')
    for _,rank in ipairs(ability.levels) do
      local range=authored_number(rank.range)
      assert(range~=nil and range>=1/1024 and range<=10000,
        ability.id..' targeted cast range must be in [1/1024,10000]')
    end
    if effect.kind=='line_damage' then
      assert(#ability.effects==1 and ability.target_type=='point' and ability.cast_type=='instant'
        and (ability.ability_type=='active' or ability.ability_type=='ultimate') and not ability.aim_geometry,
        ability.id..' line_damage requires one exclusive instant point effect')
      local width_key=identifier(effect.width_key,ability.id..' line width_key')
      local widths=ability.extras and ability.extras[width_key]
      assert(type(widths)=='table' and #widths==ability.max_level,ability.id..' requires per-rank line width')
      for _,width in ipairs(widths)do width=authored_number(width);assert(width and width>=1/1024 and width<=5000,ability.id..' invalid line width')end
      return string.format('crate::generic_effects::EffectOp::LineDamage { amount_key: "%s", width_key: "%s", kind: omb_script_abi::types::DamageKind::%s }',key,width_key,kind)
    end
    if effect.kind=='cone_damage' then
      assert(#ability.effects==1 and ability.target_type=='point' and ability.cast_type=='instant'
        and (ability.ability_type=='active' or ability.ability_type=='ultimate') and not ability.aim_geometry,
        ability.id..' cone_damage requires one exclusive instant point effect')
      local angle_key=identifier(effect.half_angle_key,ability.id..' cone half_angle_key')
      local angles=ability.extras and ability.extras[angle_key]
      assert(type(angles)=='table' and #angles==ability.max_level,ability.id..' requires per-rank cone angle')
      for _,angle in ipairs(angles) do angle=authored_number(angle)
        assert(angle and angle>=1/1024 and angle<=authored_number(math.pi/2),ability.id..' invalid cone angle') end
      return string.format('crate::generic_effects::EffectOp::ConeDamage { amount_key: "%s", half_angle_key: "%s", kind: omb_script_abi::types::DamageKind::%s }',key,angle_key,kind)
    end
    if effect.kind=='damage_over_time_enemy' then
      local duration_key=identifier(effect.duration_key,ability.id..' DoT duration_key')
      local durations=ability.extras and ability.extras[duration_key]
      assert(type(durations)=='table' and #durations==ability.max_level,ability.id..' requires per-rank DoT duration')
      for _,duration in ipairs(durations) do
        duration=authored_number(duration)
        assert(duration and duration>=1/1024 and duration<=60,ability.id..' invalid DoT duration')
      end
      return string.format('crate::generic_effects::EffectOp::DamageOverTimeEnemy { amount_key: "%s", duration_key: "%s", kind: omb_script_abi::types::DamageKind::%s }',key,duration_key,kind)
    end
    if effect.kind == 'area_damage' then
      local radius_key=identifier(effect.radius_key,ability.id..' effect radius_key')
      local radii=ability.extras and ability.extras[radius_key]
      assert(type(radii)=='table' and #radii==ability.max_level,ability.id..' requires per-rank radius')
      for rank,radius in ipairs(radii) do
        radius=authored_number(radius)
        assert(radius~=nil and radius>=1/1024 and radius<=10000,ability.id..' radius must be in [1/1024,10000]')
      end
      return string.format('crate::generic_effects::EffectOp::AreaDamage { amount_key: "%s", radius_key: "%s", kind: omb_script_abi::types::DamageKind::%s }',key,radius_key,kind)
    end
    return string.format('crate::generic_effects::EffectOp::Damage { amount_key: "%s", kind: omb_script_abi::types::DamageKind::%s }', key, kind)
  end
  if effect.kind=='drain_mana_enemy' then
    for rank,value in ipairs(levels)do
      local range=authored_number(ability.levels[rank].range)
      assert(authored_number(value)>=1/1024,ability.id..' mana drain requires positive amount')
      assert(range and range>=1/1024 and range<=10000,ability.id..' mana drain requires bounded range')
    end
    return string.format('crate::generic_effects::EffectOp::DrainManaEnemy { amount_key: "%s" }',key)
  end
  if effect.kind=='shield_ally' then
    local duration_key=identifier(effect.duration_key,ability.id..' shield duration_key')
    local durations=ability.extras and ability.extras[duration_key]
    assert(type(durations)=='table' and #durations==ability.max_level,ability.id..' shield requires per-rank duration')
    for rank,value in ipairs(levels)do
      local duration=authored_number(durations[rank]);local range=authored_number(ability.levels[rank].range)
      assert(authored_number(value)>0,ability.id..' shield amount must be positive')
      assert(duration and duration>=1/1024 and duration<=60,ability.id..' invalid shield duration')
      assert(range and range>=1/1024 and range<=10000,ability.id..' shield requires bounded range')
    end
    return string.format('crate::generic_effects::EffectOp::ShieldAlly { amount_key: "%s", duration_key: "%s" }',key,duration_key)
  end
  if effect.kind=='heal_ally' then
    for _,rank in ipairs(ability.levels) do
      local range=authored_number(rank.range)
      assert(range and range>=1/1024 and range<=10000,ability.id..' heal_ally requires bounded range')
    end
    return string.format('crate::generic_effects::EffectOp::HealAlly { amount_key: "%s" }',key)
  end
  local self_ops={heal_self='HealSelf',restore_mana_self='RestoreManaSelf',spend_mana_self='SpendManaSelf'}
  local op=self_ops[effect.kind]
  assert(op, ability.id .. ' effect has unknown kind: ' .. tostring(effect.kind))
  return string.format('crate::generic_effects::EffectOp::%s { amount_key: "%s" }', op, key)
end

-- G-1 projectile delivery: one leading op; numeric bounds are canonical in
-- omoba-content-model (validate_projectile_delivery), shapes are checked here.
local channel_payload={damage=true,slow_enemy=true,stun_enemy=true,control_enemy=true}
local projectile_payload={damage=true,slow_enemy=true,stun_enemy=true,control_enemy=true,forced_move_enemy=true}
local function projectile_code(ability,delivery)
  assert(type(delivery)=='table',ability.id..' projectile must be a table')
  local fields={motion=true,speed_key=true,radius_key=true,lifetime_key=true,max_hits=true,wall=true,target_lost=true,caster_death=true}
  for key in pairs(delivery)do assert(fields[key],ability.id..' unknown projectile field '..tostring(key))end
  local motion=({linear='Linear',homing='Homing'})[delivery.motion]
  assert(motion,ability.id..' projectile motion must be linear or homing')
  assert(ability.target_type==(motion=='Linear' and 'point' or 'unit'),ability.id..' projectile motion does not match target_type')
  local function pick(value,map,default,label)
    if value==nil then return default end
    return assert(map[value],ability.id..' invalid projectile '..label)
  end
  local wall=pick(delivery.wall,{pass='Pass',stop='Stop'},'Pass','wall')
  local lost=pick(delivery.target_lost,{expire='Expire',last_position='LastPosition'},'Expire','target_lost')
  local death=pick(delivery.caster_death,{continue='Continue',expire='Expire'},'Continue','caster_death')
  local hits=delivery.max_hits or 1
  assert(type(hits)=='number' and hits%1==0 and hits>=1 and hits<=64 and (motion=='Linear' or hits==1),
    ability.id..' projectile max_hits must be 1..64 (homing 1)')
  assert(motion=='Linear' or wall=='Pass',ability.id..' homing projectile cannot stop at walls')
  local speed=identifier(delivery.speed_key,ability.id..' projectile speed_key')
  local radius=identifier(delivery.radius_key,ability.id..' projectile radius_key')
  local lifetime=delivery.lifetime_key and identifier(delivery.lifetime_key,ability.id..' projectile lifetime_key')
  assert(motion=='Linear' or lifetime,ability.id..' homing projectile requires lifetime_key')
  for _,key in ipairs({speed,radius,lifetime})do
    assert(type(ability.extras)=='table' and type(ability.extras[key])=='table' and #ability.extras[key]==ability.max_level,
      ability.id..' projectile needs per-rank extras '..key)
  end
  return string.format('crate::generic_effects::EffectOp::Projectile { motion: omoba_content_model::ProjectileMotion::%s, speed_key: "%s", radius_key: "%s", lifetime_key: %s, max_hits: %d, wall: omoba_content_model::ProjectileWall::%s, target_lost: omoba_content_model::ProjectileTargetLost::%s, caster_death: omoba_content_model::ProjectileCasterDeath::%s }',
    motion,speed,radius,lifetime and ('Some("'..lifetime..'")') or 'None',hits,wall,lost,death)
end

-- G-2 pulse channel: one leading op; the payload ops apply once per pulse.
-- Numeric bounds are canonical in omoba-content-model (validate_channel_delivery).
local function channel_code(ability,channel)
  assert(type(channel)=='table',ability.id..' channel must be a table')
  local fields={anchor=true,interval_key=true,pulses_key=true,radius_key=true}
  for key in pairs(channel)do assert(fields[key],ability.id..' unknown channel field '..tostring(key))end
  local anchor=({self='SelfOrigin',point='Point',unit='Unit'})[channel.anchor]
  assert(anchor,ability.id..' channel anchor must be self, point or unit')
  assert(ability.target_type==({SelfOrigin='none',Point='point',Unit='unit'})[anchor],ability.id..' channel anchor does not match target_type')
  local interval=identifier(channel.interval_key,ability.id..' channel interval_key')
  local pulses=identifier(channel.pulses_key,ability.id..' channel pulses_key')
  local radius=channel.radius_key and identifier(channel.radius_key,ability.id..' channel radius_key')
  assert(radius or anchor=='Unit',ability.id..' self and point channels need radius_key')
  for _,key in ipairs({interval,pulses,radius})do
    assert(type(ability.extras)=='table' and type(ability.extras[key])=='table' and #ability.extras[key]==ability.max_level,
      ability.id..' channel needs per-rank extras '..key)
  end
  return string.format('crate::generic_effects::EffectOp::Channel { anchor: omoba_content_model::ChannelAnchor::%s, interval_key: "%s", pulses_key: "%s", radius_key: %s }',
    anchor,interval,pulses,radius and ('Some("'..radius..'")') or 'None')
end

local entries = {}
local registered = {}
for _, hero in ipairs(heroes) do
  if not hero.tombstone then
    local hero_id = identifier(hero.id, 'hero id')
    local rust_module = hero.rust_module and identifier(hero.rust_module, hero_id .. ' rust_module')
    local seen = {}
    -- Loadout abilities, then abilities an authored update moved out of the
    -- loadout (hero.retained_abilities): still registered, never in a slot.
    local registered_ids = {}
    for _, id in ipairs(hero.abilities or {}) do registered_ids[#registered_ids + 1] = id end
    for _, id in ipairs(hero.retained_abilities or {}) do registered_ids[#registered_ids + 1] = id end
    for _, ability_id in ipairs(registered_ids) do
      identifier(ability_id, hero_id .. ' ability id')
      local ability = active_abilities[ability_id]
      assert(ability and not ability.tombstone, hero_id .. ' references missing or tombstoned ability ' .. ability_id)
      assert(not seen[ability_id], hero_id .. ' lists ability twice: ' .. ability_id)
      seen[ability_id] = true
      local implementation=rust_module or '@generic'
      if registered[ability_id] then
        assert(registered[ability_id]==implementation,ability_id..' has conflicting hero handler implementations')
        -- One DLL registers each ability: a hero with its own DLL shares none.
        assert(not own_dll[hero_id] and not own_dll[registered[ability_id .. '@hero']],
          ability_id .. ' is shared with a hero that has its own DLL')
      else
      registered[ability_id]=implementation
      registered[ability_id .. '@hero']=hero_id
      if not selected(hero_id) or v3_crate_hero(hero) then
      elseif rust_module then
        entries[#entries + 1] = {module = rust_module, ability = ability_id}
      else
        assert((type(ability.effects) == 'table' and #ability.effects > 0) or type(ability.summon)=='table' or type(ability.attack_orb)=='table',
          hero_id .. ' ability ' .. ability_id .. ' needs effects, summon, attack_orb or rust_module')
        ability.effects=ability.effects or {}
        assert(ability.ability_type == 'active' or ability.ability_type == 'ultimate'
          or (ability.ability_type == 'toggle' and type(ability.attack_orb)=='table'),
          ability_id .. ' declarative effects require active or ultimate ability_type (toggle only for attack_orb)')
        assert(ability.cast_type == 'instant' or (ability.cast_type=='channeled' and (type(ability.channel)=='table'
            or (#ability.effects==1 and ability.effects[1].kind=='recovery_channel_self'))),
          ability_id .. ' declarative effects require instant, a pulse channel or an exclusive recovery channel')
        assert(#ability.effects<=32,ability_id..' exceeds 32 effects')
        local effects = {}
        -- G-3 summon delivery: exclusive point summon; canonical checks live in
        -- omoba-content-model and the template loader (unit must exist).
        -- G-2 toggle orb: the handler only flips the caster's marker buff; the
        -- host resolves cost/damage/slows from the current rank at attack launch.
        if ability.attack_orb then
          assert(#ability.effects==0 and not ability.summon and not ability.projectile and ability.target_type=='none'
            and ability.cast_type=='instant' and ability.ability_type=='toggle',ability_id..' attack_orb is an exclusive instant toggle')
          effects[1]='crate::generic_effects::EffectOp::ToggleAttackOrb'
        end
        if ability.summon then
          local summon=ability.summon
          for key in pairs(summon)do assert(key=='unit' or key=='duration_key',ability_id..' unknown summon field '..tostring(key))end
          assert(#ability.effects==0 and not ability.projectile and ability.target_type=='point' and ability.cast_type=='instant',
            ability_id..' summon delivery is an exclusive instant point ability')
          assert(type(summon.unit)=='string' and summon.unit:match('^[a-z0-9_]+$'),ability_id..' summon unit must be a template id')
          local duration=summon.duration_key and identifier(summon.duration_key,ability_id..' summon duration_key')
          effects[1]=string.format('crate::generic_effects::EffectOp::SummonAtPoint { unit: "%s", duration_key: %s }',
            summon.unit,duration and ('Some("'..duration..'")') or 'None')
        end
        local delivery=ability.projectile
        if delivery then
          assert(ability.cast_type=='instant' and #ability.effects<=3,ability_id..' projectile delivery requires instant cast and at most 3 payload effects')
          effects[1]=projectile_code(ability,delivery)
        end
        local channel=ability.channel
        if channel then
          assert(not delivery and not ability.summon and not ability.attack_orb and ability.cast_type=='channeled'
            and #ability.effects>=1 and #ability.effects<=3,ability_id..' pulse channel is exclusive with 1..3 payload effects')
          effects[1]=channel_code(ability,channel)
        end
        local mana_stats={}
        local dot_seen=false
        local unit_allegiance
        local consumes_target=false
        local target_kinds=ability.target_type=='unit'
          and {damage=true,drain_mana_enemy=true,heal_ally=true,shield_ally=true,cleanse_ally=true,damage_over_time_enemy=true,slow_enemy=true,stun_enemy=true,control_enemy=true,forced_move_enemy=true}
          or (ability.target_type=='point' and {area_damage=true,line_damage=true,cone_damage=true,reveal_area=true} or {})
        for _,effect in ipairs(ability.effects)do
          if target_kinds[effect.kind]then consumes_target=true end
        end
        for _, effect in ipairs(ability.effects) do
          if effect.kind=='damage_over_time_enemy' then assert(not dot_seen,ability_id..' duplicate DoT effect');dot_seen=true end
          local allegiance=(effect.kind=='heal_ally' or effect.kind=='shield_ally' or effect.kind=='cleanse_ally') and 'ally'
            or ((effect.kind=='damage' or effect.kind=='drain_mana_enemy' or effect.kind=='damage_over_time_enemy' or effect.kind=='slow_enemy' or effect.kind=='stun_enemy' or effect.kind=='control_enemy' or effect.kind=='forced_move_enemy') and 'enemy')
          if allegiance then
            assert(not unit_allegiance or unit_allegiance==allegiance,
              ability_id..' conflicting unit target allegiance')
            unit_allegiance=allegiance
          end
          if effect.kind=='mana_buff_self' then
            assert(type(effect.stat)=='string' and not mana_stats[effect.stat],ability_id..' duplicate/invalid mana buff stat')
            mana_stats[effect.stat]=true
          end
          assert((delivery and projectile_payload[effect.kind]) or (channel and channel_payload[effect.kind]) or
            (not delivery and effect.kind == 'damage' and ability.target_type == 'unit') or
            (effect.kind == 'damage_over_time_enemy' and ability.target_type == 'unit') or
            (effect.kind == 'heal_ally' and ability.target_type == 'unit') or
            (effect.kind == 'shield_ally' and ability.target_type == 'unit') or
            (effect.kind == 'cleanse_ally' and ability.target_type == 'unit') or
            (effect.kind == 'drain_mana_enemy' and ability.target_type == 'unit') or
            (effect.kind == 'slow_enemy' and ability.target_type == 'unit') or
            (effect.kind == 'stun_enemy' and ability.target_type == 'unit') or
            (effect.kind == 'control_enemy' and ability.target_type == 'unit') or
            (effect.kind == 'forced_move_enemy' and ability.target_type == 'unit') or
            (effect.kind == 'area_damage' and ability.target_type == 'point') or
            (effect.kind == 'cone_damage' and ability.target_type == 'point') or
            (effect.kind == 'line_damage' and ability.target_type == 'point') or
            (effect.kind == 'dash_to_point' and ability.target_type == 'point') or
            (effect.kind == 'reveal_area' and ability.target_type == 'point') or
            (effect.kind == 'restore_mana_self' and (ability.target_type=='none' or consumes_target)) or
            ((effect.kind == 'heal_self' or effect.kind=='recovery_channel_self' or effect.kind == 'spend_mana_self' or effect.kind=='mana_buff_self')
              and ability.target_type == 'none'),
            ability_id .. ' effect kind does not match target_type')
          effects[#effects + 1] = effect_code(ability, effect)
        end
        entries[#entries + 1] = {ability = ability_id, effects = effects}
      end
      end
    end
  end
end
-- Hero UnitScripts (owner hooks such as on_skill_cast / on_damage_taken):
-- a hero with `unit_script = true` registers `heroes::<rust_module>::hero_unit_script()`
-- under its own content id, which is the unit id the host binds heroes to.
local hero_units = {}
for _, hero in ipairs(heroes) do
  if not hero.tombstone and hero.unit_script ~= nil and selected(hero.id) and not v3_crate_hero(hero) then
    assert(hero.unit_script == true, hero.id .. ' unit_script must be true when present')
    assert(hero.rust_module, hero.id .. ' unit_script needs rust_module')
    hero_units[#hero_units + 1] = { id = identifier(hero.id, 'hero id'), module = identifier(hero.rust_module, hero.id .. ' rust_module') }
  end
end
if output_mode == 'units' then
  print('{')
  for _, unit in ipairs(hero_units) do
    print(string.format('    v.push(UnitDef { unit_id: "%s".into(), script: UnitScript_TO::from_value(heroes::%s::hero_unit_script(), TD_Opaque) });',
      unit.id, unit.module))
  end
  print('}')
elseif output_mode == 'ids' then
  print('&[')
  for _, entry in ipairs(entries) do
    print(string.format('    "%s",', entry.ability))
  end
  print(']')
else
  print('{')
  for _, entry in ipairs(entries) do
    -- generic_effects lives in base_content; a hero DLL registers only its rust_module handlers.
    assert(entry.module or not only_hero, entry.ability .. ': a hero with its own DLL needs rust_module handlers')
    if entry.module then
      print(string.format('    v.push(heroes::%s::%s_ffi());', entry.module, entry.ability))
    else
      print(string.format('    v.push(crate::generic_effects::generic_effect_ffi(omoba_template_ids::ability_by_name("%s").expect("generated ability id"), &[', entry.ability))
      for _, effect in ipairs(entry.effects) do
        print('        ' .. effect .. ',')
      end
      print('    ]));')
    end
  end
  print('}')
end
