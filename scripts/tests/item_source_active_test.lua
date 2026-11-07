local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local active = require('item_source.active')

-- E160: source datadriven actives translate only when every part maps to the engine.
local checks = 0
local function ok(cond, msg) checks = checks + 1; if not cond then error(msg, 2) end end
local function kv(key, children) return { key = key, children = children } end
local function v(key, value) return { key = key, value = value } end
-- Script analyses as item_source/catalog.lua attaches them (item_source/script.lua, E169).
local function analysis(calls, cooldown_items)
  return { found = true, presentation_only = #calls == 0, calls = calls, cooldown_items = cooldown_items or {} }
end
local SCRIPTS = {
  { file = 'items/Addon_Items/item_fire_scroll.lua', ['function'] = 'Shock', analysis = analysis({}, { 'fire_scroll', 'flame_book' }) },
  { file = 'items/Addon_Items/item_rations.lua', ['function'] = 'Shock', analysis = analysis({ 'GetMana', 'SetMana' }) },
  { file = 'items/Addon_Items/item_buff.lua', ['function'] = 'Shock', analysis = analysis({ 'ApplyDataDrivenModifier' }) },
  { file = 'x.lua', ['function'] = 'Shock', analysis = analysis({}) },
}
local function item(on_spell_start, modifiers, description, extra)
  local it = { key = 'fire_scroll', has_active = true, behavior = 'DOTA_ABILITY_BEHAVIOR_IMMEDIATE | DOTA_ABILITY_BEHAVIOR_NO_TARGET',
    cooldown = '26', mana_cost = '50', description = description or '【使用】周圍友軍增加20% 攻擊傷害，持續8秒。',
    scripts = SCRIPTS, active_kv = { on_spell_start = on_spell_start, modifiers = modifiers } }
  for k, x in pairs(extra or {}) do it[k] = x end
  return it
end
local function area(radius, types, names)
  local actions = {}
  for _, n in ipairs(names) do actions[#actions + 1] = kv('ApplyModifier', { v('Target', 'TARGET'), v('ModifierName', n) }) end
  return kv('ActOnTargets', { kv('Target', { v('Types', types), v('Teams', 'DOTA_UNIT_TARGET_TEAM_FRIENDLY'), v('Center', 'CASTER'),
    v('Radius', tostring(radius)) }), kv('Action', actions) })
end
local fire = kv('modifier_fire_attack', { kv('Properties', { v('MODIFIER_PROPERTY_BASEDAMAGEOUTGOING_PERCENTAGE', '20') }),
  v('Passive', '0'), v('Duration', '8'), v('IsHidden', '0'), v('EffectName', 'particles/x.vpcf') })
local presentation = kv('RunScript', { v('ScriptFile', 'items/Addon_Items/item_fire_scroll.lua'), v('Function', 'Shock') })

local r = active.translate(item({ area(700, 'DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC', { 'modifier_fire_attack', 'modifier_undeclared' }), presentation }, { fire }))
ok(r and r.active.radius == 700 and r.active.include_basic and #r.active.modifiers == 1, 'area modifier translates; undeclared names apply nothing')
ok(r.active.modifiers[1].area and r.active.modifiers[1].duration == 8 and math.abs(r.active.modifiers[1].stats.basedamageoutgoing_percentage - 0.2) < 1e-12, 'percentages become fractions')
ok(r.applied.modifier_fire_attack and r.cooldown == 26 and r.mana_cost == 50, 'applied modifiers, cooldown and mana cost')
ok(r.shares_cooldown_with and #r.shares_cooldown_with == 1 and r.shares_cooldown_with[1] == 'flame_book',
  'the script StartCooldown list (minus the item itself) becomes shares_cooldown_with')
-- A KV AbilitySharedCooldown group joins the list.
r = active.translate(item({ area(700, 'DOTA_UNIT_TARGET_HERO', { 'modifier_fire_attack' }) }, { fire }, nil, { shared_cooldown = 'share_fire' }),
  { share_fire = { 'fire_scroll', 'red_book' } })
ok(r and r.shares_cooldown_with[1] == 'red_book' and #r.shares_cooldown_with == 1, 'AbilitySharedCooldown group members share')

r = active.translate(item({ kv('ApplyModifier', { v('Target', 'CASTER'), v('ModifierName', 'modifier_fire_attack'), v('Duration', '5') }) }, { fire },
  '【使用】增加20% 攻擊傷害，持續5秒。'))
ok(r and not r.active.radius and not r.active.modifiers[1].area and r.active.modifiers[1].duration == 5, 'caster modifier; action duration overrides')

r = active.translate(item({ kv('Heal', { v('Target', 'CASTER'), v('HealAmount', '400') }),
  kv('RunScript', { v('ScriptFile', 'items/Addon_Items/item_rations.lua'), v('Function', 'Shock'), v('Mana', '400') }) }, {},
  '【使用】回復400生命和法力。', { cooldown = '45', mana_cost = '0' }))
ok(r and r.active.heal == 400 and r.active.mana == 400 and not r.active.modifiers, 'caster heal and the Mana parameter')

local function refuses(it, pattern)
  local result, why = active.translate(it)
  ok(not result and tostring(why):find(pattern, 1, true), 'expected "' .. pattern .. '", got ' .. tostring(why))
end
refuses(item({ presentation }, {}), 'only in a script absent from the source')
-- E176: the code decides; a described number it does not explain is reported, not refused.
r = active.translate(item({ area(700, 'DOTA_UNIT_TARGET_HERO', { 'modifier_fire_attack' }), presentation }, { fire },
  '【使用】周圍友軍增加20% 攻擊傷害，持續8秒，隱形2.5秒。'))
ok(r and r.description_unexplained and r.description_unexplained[1] == 2.5, 'an unexplained described number is reported')
refuses(item({ area(700, 'DOTA_UNIT_TARGET_HERO', { 'modifier_fire_attack' }) }, { fire }, nil, { behavior = 'DOTA_ABILITY_BEHAVIOR_UNIT_TARGET' }), 'targeted active')
refuses(item({ kv('RunScript', { v('ScriptFile', 'x.lua'), v('Function', 'Shock'), v('SHP1', '3') }) }, {}), 'RunScript parameter SHP1')
-- Scripts are read, not assumed: a game effect in the function, or a missing function, refuses.
refuses(item({ area(700, 'DOTA_UNIT_TARGET_HERO', { 'modifier_fire_attack' }),
  kv('RunScript', { v('ScriptFile', 'items/Addon_Items/item_buff.lua'), v('Function', 'Shock') }) }, { fire }), 'calls ApplyDataDrivenModifier')
refuses(item({ area(700, 'DOTA_UNIT_TARGET_HERO', { 'modifier_fire_attack' }),
  kv('RunScript', { v('ScriptFile', 'items/Addon_Items/missing.lua'), v('Function', 'Shock') }) }, { fire }), 'is not in the source')
refuses(item({ kv('RunScript', { v('ScriptFile', 'items/Addon_Items/item_buff.lua'), v('Function', 'Shock'), v('Mana', '100') }) }, {},
  '【使用】回復100法力。'), 'does more than restore Mana')
local stun = kv('modifier_stun', { kv('States', { v('MODIFIER_STATE_STUNNED', 'MODIFIER_STATE_VALUE_ENABLED') }), v('Duration', '8'),
  kv('Properties', { v('MODIFIER_PROPERTY_BASEDAMAGEOUTGOING_PERCENTAGE', '20') }) })
refuses(item({ kv('ApplyModifier', { v('Target', 'CASTER'), v('ModifierName', 'modifier_stun') }) }, { stun }), 'modifier_stun uses States')
local crit = kv('modifier_crit', { kv('Properties', { v('MODIFIER_PROPERTY_PREATTACK_CRITICALSTRIKE', '20') }), v('Duration', '8') })
refuses(item({ kv('ApplyModifier', { v('Target', 'CASTER'), v('ModifierName', 'modifier_crit') }) }, { crit }), 'PREATTACK_CRITICALSTRIKE is not supported')
local enemies = area(700, 'DOTA_UNIT_TARGET_HERO', { 'modifier_fire_attack' })
enemies.children[1].children[2] = v('Teams', 'DOTA_UNIT_TARGET_TEAM_ENEMY')
refuses(item({ enemies }, { fire }), 'ActOnTargets other than friendly heroes')
-- Passive aura (E173): friendly heroes and units, holder included, stat properties only.
local function aura_item(teams, apply, prop)
  return { key = 'book', specials = { { name = 'pct', value = '7' } }, active_kv = { modifiers = {
    kv('Passive_book', { v('Passive', '1'), v('Aura', 'modifier_book'), v('Aura_Radius', '700'), v('Aura_Teams', teams),
      v('Aura_Types', 'DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC'), v('Aura_ApplyToCaster', apply) }),
    kv('modifier_book', { kv('Properties', { v(prop or 'MODIFIER_PROPERTY_BASEDAMAGEOUTGOING_PERCENTAGE', '%pct') }) }),
  } } }
end
local a = active.aura(aura_item('DOTA_UNIT_TARGET_TEAM_FRIENDLY', '1'))
ok(a and a.aura.name == 'modifier_book' and a.aura.radius == 700 and math.abs(a.aura.stats.basedamageoutgoing_percentage - 0.07) < 1e-12,
  'aura from a passive modifier, %special resolved')
ok(a.modifiers.Passive_book and a.modifiers.modifier_book, 'both modifiers belong to the aura')
local _, why = active.aura(aura_item('DOTA_UNIT_TARGET_TEAM_ENEMY', '1'))
ok(why and why:find('aura on', 1, true), 'enemy auras wait')
_, why = active.aura(aura_item('DOTA_UNIT_TARGET_TEAM_FRIENDLY', '0'))
ok(why and why:find('excludes its holder', 1, true), 'an aura without its holder waits')
_, why = active.aura(aura_item('DOTA_UNIT_TARGET_TEAM_FRIENDLY', '1', 'MODIFIER_PROPERTY_PREATTACK_CRITICALSTRIKE'))
ok(why and why:find('not supported', 1, true), 'unsupported aura stats wait')
ok(active.aura({ active_kv = {} }) == nil, 'no aura is not an error')
-- A modifier without a Duration lasts until removed (0).
r = active.translate(item({ kv('ApplyModifier', { v('Target', 'CASTER'), v('ModifierName', 'modifier_still') }) },
  { kv('modifier_still', { kv('Properties', { v('MODIFIER_PROPERTY_BASEDAMAGEOUTGOING_PERCENTAGE', '7') }) }) }, '【使用】增加7% 攻擊傷害。'))
ok(r and r.active.modifiers[1].duration == 0, 'no Duration becomes a lasting modifier')

-- Enemy unit target (E176): KV/script damage of one type plus one control from the applied modifier.
local function enemy(actions, hit, modifiers, extra)
  local it = item(actions, modifiers, '【使用】暈眩目標並造成50魔法傷害。', { behavior = 'DOTA_ABILITY_BEHAVIOR_UNIT_TARGET',
    target_team = 'DOTA_UNIT_TARGET_TEAM_ENEMY', target_types = 'DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC', cast_range = '700',
    scripts = { { file = 'items/Addon_Items/blow.lua', ['function'] = 'Shock', analysis = { found = true, presentation_only = false, calls = {},
      cooldown_items = {}, effects = { { kind = 'apply_modifier', modifier = 'modifier_blow', non_building = true } } } },
      { file = 'items/Addon_Items/star.lua', ['function'] = 'Shock', analysis = { found = true, presentation_only = false, calls = {},
      cooldown_items = {}, effects = { { kind = 'damage', amount = 150, damage_type = 'magical' },
        { kind = 'apply_modifier', modifier = 'modifier_stunned', duration = 0.05 } } } } } })
  it.active_kv.on_projectile_hit = hit
  for k, x in pairs(extra or {}) do it[k] = x end
  return it
end
local stun = kv('modifier_blow', { kv('States', { v('MODIFIER_STATE_STUNNED', 'MODIFIER_STATE_VALUE_ENABLED') }), v('Duration', '1'), v('IsDebuff', '1') })
local projectile = kv('TrackingProjectile', { v('Target', 'TARGET'), v('Dodgeable', '1'), v('MoveSpeed', '2500'),
  v('ProvidesVision', '1'), v('VisionRadius', '225') })
r = active.translate(enemy({ projectile }, { kv('RunScript', { v('ScriptFile', 'items/Addon_Items/blow.lua'), v('Function', 'Shock') }),
  kv('Damage', { v('Target', 'TARGET'), v('Type', 'DAMAGE_TYPE_MAGICAL'), v('Damage', '50') }) }, { stun }))
ok(r and r.active.kind == 'unit_hex' and r.active.damage == 50 and r.active.damage_type == 'magical'
  and r.active.control.kind == 'stun' and r.active.control.duration == 1 and r.target.team == 'enemy' and r.target.range == 700,
  'projectile hit: KV damage plus the stun of the script-applied modifier')
ok(r.active.projectile and r.active.projectile.speed == 2500 and r.active.projectile.dodgeable and r.active.projectile.vision == 225,
  'E184: the effects ride the dodgeable tracking projectile and land on arrival')
refuses(enemy({ kv('TrackingProjectile', { v('Target', 'TARGET') }) }, { kv('Damage', { v('Target', 'TARGET'),
  v('Type', 'DAMAGE_TYPE_MAGICAL'), v('Damage', '50') }) }, {}), 'without one positive MoveSpeed')
r = active.translate(enemy({ kv('RunScript', { v('ScriptFile', 'items/Addon_Items/star.lua'), v('Function', 'Shock') }) }, nil, {}))
ok(r and r.active.damage == 150 and r.active.control.kind == 'stun' and r.active.control.duration == 0.05, "script damage and Dota's modifier_stunned")
refuses(enemy({ kv('Damage', { v('Target', 'TARGET'), v('Type', 'DAMAGE_TYPE_MAGICAL'), v('Damage', '50') }),
  kv('Damage', { v('Target', 'TARGET'), v('Type', 'DAMAGE_TYPE_PHYSICAL'), v('Damage', '50') }) }, nil, {}), 'damage of two types')
local slow = kv('modifier_blow', { kv('Properties', { v('MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE', '-30') }), v('Duration', '4') })
refuses(enemy({ kv('Damage', { v('Target', 'TARGET'), v('Type', 'DAMAGE_TYPE_MAGICAL'), v('Damage', '50') }),
  kv('RunScript', { v('ScriptFile', 'items/Addon_Items/blow.lua'), v('Function', 'Shock') }) }, nil, { slow }), 'changes MOVESPEED_BONUS_PERCENTAGE')
refuses(enemy({ kv('ApplyModifier', { v('Target', 'TARGET'), v('ModifierName', 'modifier_blow') }) }, nil, { stun }), 'without damage')
-- E188: KV Damage has an inflictor (spell amplification applies), script AMHC:Damage has none.
ok(r and r.active.spell == nil, 'script AMHC damage is not a spell')
r = active.translate(enemy({ kv('Damage', { v('Target', 'TARGET'), v('Type', 'DAMAGE_TYPE_MAGICAL'), v('Damage', '400') }) }, nil, {}))
ok(r and r.active.spell == true, 'KV Damage is a spell')
refuses(enemy({ kv('Damage', { v('Target', 'TARGET'), v('Type', 'DAMAGE_TYPE_MAGICAL'), v('Damage', '50') }),
  kv('RunScript', { v('ScriptFile', 'items/Addon_Items/star.lua'), v('Function', 'Shock') }) }, nil, {}), 'two types or inflictors')
-- E188: a modifier that is only dummy.lua Slow/ReturnSpeed is a named slow on the target.
local SLOW = 'scripts/vscripts/library/common/dummy.lua'
local named = kv('modifier_blow', { v('Duration', '4'), v('IsDebuff', '1'),
  kv('OnCreated', { kv('RunScript', { v('ScriptFile', SLOW), v('Function', 'Slow'), v('name', 'blow'), v('ms_slow', '-30') }) }),
  kv('OnDestroy', { kv('RunScript', { v('ScriptFile', SLOW), v('Function', 'ReturnSpeed'), v('name', 'blow') }) }) })
r = active.translate(enemy({ kv('Damage', { v('Target', 'TARGET'), v('Type', 'DAMAGE_TYPE_MAGICAL'), v('Damage', '50') }),
  kv('RunScript', { v('ScriptFile', 'items/Addon_Items/blow.lua'), v('Function', 'Shock') }) }, nil, { named }))
ok(r and r.active.slow and math.abs(r.active.slow.move_speed - 0.3) < 1e-9 and r.active.slow.duration == 4 and not r.active.control,
  'dummy.lua Slow: 30% for the modifier Duration')
named.children[3] = v('ThinkInterval', '1')
refuses(enemy({ kv('Damage', { v('Target', 'TARGET'), v('Type', 'DAMAGE_TYPE_MAGICAL'), v('Damage', '50') }),
  kv('RunScript', { v('ScriptFile', 'items/Addon_Items/blow.lua'), v('Function', 'Shock') }) }, nil, { named }), 'blow uses')

-- Point area (E178): one script area, the item's cast range, hostile targets (neutrals included).
local bomb = item({ kv('RunScript', { v('ScriptFile', 'items/Addon_Items/bomb.lua'), v('Function', 'Shock'), v('Target', 'POINT') }),
  kv('DelayedAction', { v('Delay', '0.1'), kv('Action', { kv('SpendCharge', {}) }) }) }, {}, '【使用】對範圍內敵人造成300的傷害。',
  { behavior = 'DOTA_ABILITY_BEHAVIOR_POINT | DOTA_ABILITY_BEHAVIOR_AOE', target_team = 'DOTA_UNIT_TARGET_TEAM_ENEMY', cast_range = '600',
    initial_charges = '1', cooldown = '10', mana_cost = nil,
    scripts = { { file = 'items/Addon_Items/bomb.lua', ['function'] = 'Shock', analysis = { found = true, presentation_only = false, calls = {},
      cooldown_items = {}, effects = {}, area = { radius = 300, damage = 300, damage_type = 'magical', skip_buildings = true } } } } })
r = active.translate(bomb)
ok(r and r.active.kind == 'point_nova' and r.active.damage == 300 and r.active.radius == 300 and r.active.damage_type == 'magical'
  and r.active.skip_buildings and r.consumable and r.target.shape == 'point' and r.target.team == 'hostile' and r.target.range == 600,
  'point area from the script, single use')
-- E188: a reviewed area with building damage, a magic-immune policy, sight and a burn modifier.
bomb.scripts[1].analysis.area = nil
bomb.scripts[1].analysis.reviewed_effect = { kind = 'point_area', area = { radius = 800, damage = 550, damage_type = 'magical',
  building_damage = 200, excludes_magic_immune = true, vision = { radius = 400, duration = 3 },
  modifier = { name = 'modifier_burn', duration = 5 } } }
bomb.active_kv.modifiers = { kv('modifier_burn', { v('Duration', '5'), v('ThinkInterval', '1.0'),
  kv('OnIntervalThink', { kv('Damage', { v('Target', 'TARGET'), v('Type', 'DAMAGE_TYPE_MAGICAL'), v('Damage', '80') }) }) }) }
r = active.translate(bomb)
ok(r and r.active.damage == 550 and r.active.building_damage == 200 and r.active.excludes_magic_immune and not r.active.spell
  and r.active.dot and r.active.dot.damage == 80 and r.active.dot.interval == 1 and r.active.dot.duration == 5 and r.active.dot.spell
  and r.active.vision.radius == 400, 'reviewed area: AMHC hit, building damage, KV burn (a spell), sight')
bomb.active_kv.modifiers[1].children[3].children[1].children[2] = v('Type', 'DAMAGE_TYPE_PHYSICAL')
refuses(bomb, 'damage type differs')
bomb.scripts[1].analysis.reviewed_effect = nil
bomb.scripts[1].analysis.reviewed_mismatch = 'local SEARCH_RADIUS = 800'
refuses(bomb, 'no longer has its reviewed fragment')
bomb.scripts[1].analysis.reviewed_mismatch = nil
refuses(bomb, 'not an area the engine reads')

-- Magic immunity (E179): a toggle item acts on OnToggleOn; the reviewed script reads its Time.
local wine = item(nil, {}, '【使用】在3.5秒內獲得魔法免疫。', { behavior = 'DOTA_ABILITY_BEHAVIOR_IMMEDIATE | DOTA_ABILITY_BEHAVIOR_TOGGLE',
  cooldown = '45', mana_cost = nil, initial_charges = '1',
  scripts = { { file = 'items/Addon_Items/item_perceive_wine.lua', ['function'] = 'Shock', analysis = { found = true,
    presentation_only = false, calls = { 'ApplyDataDrivenModifier' }, cooldown_items = {}, effects = {},
    reviewed_effect = { kind = 'magic_immunity', seconds = 'Time' } } } } })
wine.active_kv.on_spell_start = nil
wine.active_kv.on_toggle_on = { kv('RunScript', { v('ScriptFile', 'items/Addon_Items/item_perceive_wine.lua'), v('Function', 'Shock'), v('Time', '3.5') }),
  kv('DelayedAction', { v('Delay', '0.1'), kv('Action', { kv('SpendCharge', {}) }) }) }
r = active.translate(wine)
ok(r and r.active.magic_immunity == 3.5 and r.consumable, 'toggle magic immunity from the reviewed script, single use')

-- Single use: a SpendCharge (alone in a DelayedAction) on a one-charge item.
local spend = kv('DelayedAction', { v('Delay', '0.1'), kv('Action', { kv('SpendCharge', {}) }) })
r = active.translate(item({ kv('Heal', { v('Target', 'CASTER'), v('HealAmount', '420') }), spend }, {}, '回復420生命值。',
  { cooldown = '30', mana_cost = nil, initial_charges = '1' }))
ok(r and r.consumable and r.active.heal == 420, 'one-charge SpendCharge is a consumable; text without 【使用】 is checked whole')
refuses(item({ kv('Heal', { v('Target', 'CASTER'), v('HealAmount', '420') }), spend }, {}, '回復420生命值。',
  { cooldown = '30', initial_charges = '3' }), 'charges other than one')
-- Ally unit target: Heal on the target, cast range and team from the item.
local ally = { behavior = 'DOTA_ABILITY_BEHAVIOR_UNIT_TARGET', target_team = 'DOTA_UNIT_TARGET_TEAM_FRIENDLY',
  target_types = 'DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC', cast_range = '700', cooldown = '45', mana_cost = '60' }
r = active.translate(item({ kv('Heal', { v('Target', 'TARGET'), v('HealAmount', '420') }), kv('FireSound', { v('Target', 'TARGET') }) }, {},
  '【使用】使目標友軍回復420生命值。', ally))
ok(r and r.active.kind == 'unit_heal' and r.active.amount == 420 and r.target.team == 'ally' and r.target.range == 700, 'ally unit heal')
local enemy = {}
for k, x in pairs(ally) do enemy[k] = x end
enemy.target_team = 'DOTA_UNIT_TARGET_TEAM_ENEMY'
refuses(item({ kv('Damage', { v('Target', 'TARGET'), v('Damage', '50') }) }, {}, '【使用】造成50傷害。', enemy), 'Damage other than the target')
refuses(item({ kv('RunScript', { v('ScriptFile', 'x.lua'), v('Function', 'Shock') }) }, {}, '【使用】使目標友軍回復420生命值。', ally),
  'only in a script absent from the source')
print(('item source active: %d checks passed'):format(checks))
