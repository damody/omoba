-- 由 Lua 英雄／技能宣告產生 base_content 的 Rust FFI 註冊語句。
local content_root = assert(arg[1], 'content root is required')
local output_mode = arg[2] or 'registry'
assert(output_mode == 'registry' or output_mode == 'ids', 'unknown output mode')
local context, loading, depth = {}, {}, 0
function context.include(relative)
  assert(type(relative)=='string' and relative~='' and not relative:find(':',1,true)
    and not relative:match('^[/\\]'),'include requires a content-relative path')
  local segments={}
  for segment in relative:gmatch('[^/\\]+') do
    assert(segment~='..','include rejects parent-directory escape')
    if segment~='.' then segments[#segments+1]=segment end
  end
  local canonical=table.concat(segments,'/')
  -- Fixed Windows Lua runtime: aliases differing only in case are one file.
  local identity=canonical:lower()
  assert(canonical~='' and not loading[identity],'include cycle: '..canonical)
  assert(depth<64,'include nesting exceeds 64')
  loading[identity]=true;depth=depth+1
  local ok,entries=xpcall(function()
    local builder=assert(loadfile(content_root..'/'..canonical))()
    assert(type(builder)=='function',canonical..' must return a builder function')
    return builder(context)
  end,debug.traceback)
  loading[identity]=nil;depth=depth-1
  assert(ok,entries)
  assert(type(entries)=='table',canonical..' builder must return a table')
  return entries
end
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
  local key = identifier(effect.amount_key, ability.id .. ' effect amount_key')
  local levels = ability.extras and ability.extras[key]
  assert(type(levels) == 'table' and #levels == ability.max_level,
    ability.id .. ' effect amount_key ' .. key .. ' needs one extras value per level')
  for _, value in ipairs(levels) do
    assert(scalar(value),
      ability.id .. ' effect amount_key ' .. key .. ' must be zero or finite in [1/1024,1000000]')
  end
  if effect.kind == 'damage' or effect.kind == 'area_damage' then
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
  assert(effect.kind == 'heal_self', ability.id .. ' effect has unknown kind: ' .. tostring(effect.kind))
  return string.format('crate::generic_effects::EffectOp::HealSelf { amount_key: "%s" }', key)
end

local entries = {}
for _, hero in ipairs(heroes) do
  if not hero.tombstone then
    local hero_id = identifier(hero.id, 'hero id')
    local rust_module = hero.rust_module and identifier(hero.rust_module, hero_id .. ' rust_module')
    local seen = {}
    for _, ability_id in ipairs(hero.abilities or {}) do
      identifier(ability_id, hero_id .. ' ability id')
      local ability = active_abilities[ability_id]
      assert(ability and not ability.tombstone, hero_id .. ' references missing or tombstoned ability ' .. ability_id)
      assert(not seen[ability_id], hero_id .. ' lists ability twice: ' .. ability_id)
      seen[ability_id] = true
      if rust_module then
        entries[#entries + 1] = {module = rust_module, ability = ability_id}
      else
        assert(type(ability.effects) == 'table' and #ability.effects > 0,
          hero_id .. ' ability ' .. ability_id .. ' needs effects or rust_module')
        assert(ability.ability_type == 'active' or ability.ability_type == 'ultimate',
          ability_id .. ' declarative effects require active or ultimate ability_type')
        assert(ability.cast_type == 'instant',
          ability_id .. ' declarative effects require instant cast_type')
        assert(#ability.effects<=32,ability_id..' exceeds 32 effects')
        local effects = {}
        for _, effect in ipairs(ability.effects) do
          assert((effect.kind == 'damage' and ability.target_type == 'unit') or
            (effect.kind == 'slow_enemy' and ability.target_type == 'unit') or
            (effect.kind == 'area_damage' and ability.target_type == 'point') or
            (effect.kind == 'dash_to_point' and ability.target_type == 'point') or
            (effect.kind == 'heal_self' and ability.target_type == 'none'),
            ability_id .. ' effect kind does not match target_type')
          effects[#effects + 1] = effect_code(ability, effect)
        end
        entries[#entries + 1] = {ability = ability_id, effects = effects}
      end
    end
  end
end
if output_mode == 'ids' then
  print('&[')
  for _, entry in ipairs(entries) do
    print(string.format('    "%s",', entry.ability))
  end
  print(']')
else
  print('{')
  for _, entry in ipairs(entries) do
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
