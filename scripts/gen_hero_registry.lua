-- 由 Lua 英雄／技能宣告產生 base_content 的 Rust FFI 註冊語句。
local content_root = assert(arg[1], 'content root is required')
local output_mode = arg[2] or 'registry'
assert(output_mode == 'registry' or output_mode == 'ids', 'unknown output mode')
local function load_builder(name)
  local builder = assert(dofile(content_root .. '/templates/' .. name .. '.lua'))
  assert(type(builder) == 'function', name .. ' must return a builder function')
  local entries = builder({})
  assert(type(entries) == 'table', name .. ' builder must return a table')
  return entries
end

local function identifier(value, label)
  assert(type(value) == 'string' and value:match('^[A-Za-z_][A-Za-z0-9_]*$'),
    label .. ' must be a Rust identifier')
  return value
end

local heroes = load_builder('heroes')
local abilities = load_builder('abilities')
local active_abilities = {}
for _, ability in ipairs(abilities) do
  local id = identifier(ability.id, 'ability id')
  assert(active_abilities[id] == nil, 'duplicate ability id: ' .. id)
  active_abilities[id] = ability
end

local function effect_code(ability, effect)
  local key = identifier(effect.amount_key, ability.id .. ' effect amount_key')
  local levels = ability.extras and ability.extras[key]
  assert(type(levels) == 'table' and #levels == ability.max_level,
    ability.id .. ' effect amount_key ' .. key .. ' needs one extras value per level')
  for _, value in ipairs(levels) do
    assert(type(value) == 'number' and value >= 0 and value < math.huge,
      ability.id .. ' effect amount_key ' .. key .. ' must be finite and nonnegative')
  end
  if effect.kind == 'damage' then
    local kinds = {physical = 'Physical', magical = 'Magical', pure = 'Pure'}
    local kind = kinds[effect.damage_kind]
    assert(kind, ability.id .. ' effect has invalid damage_kind')
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
        local effects = {}
        for _, effect in ipairs(ability.effects) do
          assert((effect.kind == 'damage' and ability.target_type == 'unit') or
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
