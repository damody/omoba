-- hero_author v3 documents (scripts/hero_author/v3.lua): extras, levels and descriptions from v3.lua.
-- tools\lua\lua.exe scripts\tests\hero_author_v3_test.lua
package.path = 'scripts/?.lua;' .. package.path
local b = require('_bootstrap')
local v3 = require('hero_author.v3')

local count = 0
local function test(name, fn) fn(); count = count + 1; print('PASS ' .. name) end
local function fails(fn, fragment)
  local ok, err = pcall(fn)
  assert(not ok, 'expected failure containing ' .. fragment)
  assert(tostring(err):find(fragment, 1, true), 'expected "' .. fragment .. '", got: ' .. tostring(err))
end

local function kit()
  return {
    schema_version = 3, kind = 'hero', id = 'X01',
    source = { repository = 'history_nobu', commit = 'f966720', root = 'game/dota_addons/nobu/scripts/' },
    hero = { source_id = 'X01', text = { zh_TW = { name = '測試' }, en = { name = 'Test' } } },
    abilities = { {
      id = 'x01_strike', source_id = 'X01W', slot = 'Q', kind = 'active', max_level = 2,
      values = {
        cooldown = { 10, 8, unit = 's', source = 'kv:AbilityCooldown' },
        mana_cost = { 50, unit = 'mana', source = 'kv:AbilityManaCost' },
        radius = { 200, unit = 'dist', source = 'kv:radius', v2_key = 'shape_radius' },
        damage = { 100, 150, unit = 'dmg', damage_type = 'magical', source = 'kv:damage', v2_key = 'damage_amount' },
      },
      text = {
        zh_TW = { name = '打擊', body = '對 {radius} 內的敵人造成 {damage} 魔法傷害。' },
        en = { name = 'Strike', body = 'Deals {damage} magic damage within {radius}.' },
      },
      target = { type = 'point', aoe_radius = '$radius' },
      cost = { mana = '$mana_cost', cooldown = '$cooldown' },
      coverage = 'rust_handler', handler = { module = 'No1_strike' }, confidence = 'high',
      provenance = { 'npc/X/X01.txt:1-20' },
    } },
  }
end
-- the v2 entry hero_author.generate builds for the same ability
local function entry()
  return {
    id = 'x01_strike', max_level = 2, description = 'old 100/150',
    levels = { { cooldown = 10, mana_cost = 50, cast_time = 0, range = 0 }, { cooldown = 8, mana_cost = 50, cast_time = 0, range = 0 } },
    extras = { damage_amount = { 100, 150 }, shape_radius = { 200, 200 } },
    targeting_shape = { kind = 'circle', radius_key = 'shape_radius' },
  }
end

test('kit_values_become_extras_levels_and_description', function()
  local compiled = v3.compile(b.root, kit())
  local e = entry()
  v3.apply(e, compiled)
  assert(e.extras.damage[2] == 150 and e.extras.radius[1] == 200, 'v3 names')
  assert(e.extras.shape_radius[1] == 200, 'host alias keeps its v2 name')
  assert(e.extras.damage_amount == nil, 'replaced v2 keys are dropped')
  assert(e.extras.cooldown == nil and e.extras.mana_cost == nil, 'level fields stay out of extras (AbilityLevelData flattens them)')
  assert(e.levels[2].cooldown == 8)
  assert(e.description == '對 200 內的敵人造成 100/150 魔法傷害。', e.description)
  assert(e.v3 and e.v3.text.en.rendered:find('100/150', 1, true))
end)

test('host_keys_survive_missing_fields', function()
  -- the first contract field absent must not hide the later ones (ipairs over nils)
  local keys = v3.host_keys({ charges = { max_key = 'charges_max', recharge_key = 'charges_recharge_seconds' } })
  assert(keys.charges_max and keys.charges_recharge_seconds)
end)

test('a v2 number that differs from the kit fails', function()
  local e = entry(); e.extras.damage_amount = { 100, 160 }
  fails(function() v3.apply(e, v3.compile(b.root, kit())) end, 'differs from abilities.lua')
  e = entry(); e.levels[1].cooldown = 11
  fails(function() v3.apply(e, v3.compile(b.root, kit())) end, 'differs from abilities.lua')
end)

test('every v2 number needs a kit value', function()
  local e = entry(); e.extras.projectile_speed = { 900, 900 }
  fails(function() v3.apply(e, v3.compile(b.root, kit())) end, 'projectile_speed')
end)

test('host keys need a kit value', function()
  local k = kit(); k.abilities[1].values.radius.v2_key = nil
  local e = entry(); e.extras.shape_radius = nil
  fails(function() v3.apply(e, v3.compile(b.root, k)) end, 'host key shape_radius')
end)

test('rust handlers may not read replaced keys', function()
  local compiled = v3.compile(b.root, kit())
  local e = entry(); v3.apply(e, compiled)
  v3.check_rust_keys({ ['rust/No1_strike.rs'] = 'let d=extra("damage")?; let r=extra("shape_radius")?;' }, { e })
  fails(function() v3.check_rust_keys({ ['rust/No1_strike.rs'] = 'let d=extra("damage_amount")?;' }, { e }) end, 'damage_amount')
  fails(function() v3.check_rust_keys({ ['rust/No1_strike.rs'] = 'const D:&str="damage_amount";' }, { e }) end, 'replaced v2 key')
end)

print(count .. ' hero_author v3 tests passed')
