-- D13 fixes of original-game bugs (fix-candidates.md, quirk policy `fixed`): every fixed quirk names its
-- FC number and that row is marked done; each data fix holds in the v3 document and reaches the Rust
-- that v3_rust.lua generates from it.
-- tools\lua\lua.exe scripts\tests\v3_fixed_quirks_test.lua
package.path = 'scripts/?.lua;' .. package.path
local b = require('_bootstrap')
local path = b.lib('path')

local tools = path.join(b.root, 'docs', 'character_pipeline', 'tools')
local sources = dofile(path.join(tools, 'v3_sources.lua'))
PACKAGE_V3_AS_MODULE = true
local v3 = dofile(path.join(tools, 'package_v3.lua'))
PACKAGE_V3_AS_MODULE = nil
V3_RUST_AS_MODULE = true
local R = dofile(path.join(tools, 'v3_rust.lua'))
V3_RUST_AS_MODULE = nil

local count = 0
local function test(name, fn) fn(); count = count + 1; print('PASS ' .. name) end

local function load(sid)
  local doc = assert(v3.load_file(assert(sources.hero_path(b.root, sid), 'no ' .. sid .. ' document')))
  local errs = v3.validate_doc(doc)
  assert(#errs == 0, sid .. ': ' .. tostring(errs[1]))
  return doc
end
local function ability(doc, id)
  for _, a in ipairs(doc.abilities) do if a.id == id then return a end end
  error('no ability ' .. id)
end
local function status(a, id)
  for _, s in ipairs(a.statuses or {}) do if s.id == id then return s end end
  error('no status ' .. id)
end
local function ranks(v) local t = {} for i, x in ipairs(v) do t[i] = x end return table.concat(t, '/') end
local function generated(doc, file)
  local files = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'test' })
  -- an entry the generator cannot lower yet has no file (no stub, D7)
  return files['src/generated/' .. file] or ('// ' .. file .. ' is not generated yet')
end
local function has(text, needle) return text:find(needle, 1, true) ~= nil end

test('every fixed quirk names an FC row that fix-candidates.md marks done', function()
  local f = assert(io.open(path.join(b.root, 'docs', 'plans', '2026-10-28-history-nobu-full-port', 'fix-candidates.md'), 'rb'))
  local md = f:read('a'); f:close()
  local seen = 0
  for _, p in ipairs(sources.all(b.root)) do
    local doc = assert(v3.load_file(p))
    local function walk(x)
      if type(x) ~= 'table' then return end
      if x.policy == 'fixed' then
        seen = seen + 1
        local fc = assert(tostring(x.text):match('FC%-%d%d%d'), p .. ': a fixed quirk without its FC number')
        local row = md:match('\n(| ' .. fc:gsub('%-', '%%-') .. ' |[^\n]*)')
        assert(row, fc .. ': no row in fix-candidates.md')
        assert(row:find('✓', 1, true), fc .. ': the row is not marked done')
      end
      for _, v in pairs(x) do walk(v) end
    end
    walk(doc)
  end
  assert(seen >= 100, 'expected the hero and item fixed quirks, saw ' .. seen)
end)

test('FC-054 B20 tenacity grants its magic resistance (B20R_mrBonus)', function()
  local doc = load('B20')
  local a = ability(doc, 'b20_tenacity')
  assert(ranks(a.values.magic_resist) == '8/16/24/32', ranks(a.values.magic_resist))
  assert(a.passive.stats.magic_resist == '$magic_resist')
  -- the passive stats buff of the generated DLL carries the authored percent points
  local rs = generated(doc, 'b20_tenacity.rs')
  if not has(rs, 'is not generated yet') then
    assert(has(rs, 'StatKey::MagicalResistanceBonus, super::values::b20_tenacity::MAGIC_RESIST.at(cx.rank).raw()'), rs)
  end
  local values = generated(doc, 'values.rs')
  assert(has(values, 'pub const MAGIC_RESIST: Ranked<4> = Ranked([8192, 16384, 24576, 32768]);'), 'B20R magic_resist raw')
end)

test('FC-055 B27 horsemanship speeds Sadaoki up instead of feeding the slow pool', function()
  local doc = load('B27')
  local a = ability(doc, 'b27_horsemanship')
  local s = status(a, 'b27w_buff')
  assert(s.stats.slow_pct == nil, 'no counter-slow')
  assert(s.stats.move_speed_pct == '$move_bonus' and s.stats.attack_speed == '$attack_speed')
  assert(ranks(a.values.attack_speed) == '150')
  local rs = generated(doc, 'b27_horsemanship.rs')
  assert(not has(rs, 'is not generated yet'), 'B27W generates')
  assert(has(rs, '(StatKey::AttackSpeedBonusConstant, (super::values::b27_horsemanship::ATTACK_SPEED.at(cx.rank)).raw())'), rs)
  assert(has(rs, '(StatKey::MoveSpeedBonusPercentage, (super::values::b27_horsemanship::MOVE_BONUS_FRAC.at(cx.rank)).raw())'), rs)
end)

test('FC-048 A12 awe slows the enemies it marks', function()
  local doc = load('A12')
  local a = ability(doc, 'a12_awe')
  assert(ranks(a.values.slow) == '15/20/25/30', ranks(a.values.slow))
  local s = status(a, 'a12r_mark')
  assert(s.stats.slow_pct == '$slow' and s.duration == '$mark_duration')
  local tagged = false
  for _, t in ipairs(s.tags or {}) do if t == 'slow' then tagged = true end end
  assert(tagged, 'the mark is a slow (rules-layer tag)')
end)

test('FC-047 A12 flash of insight, empowered, stuns and spends the empowerment', function()
  local doc = load('A12')
  local a = ability(doc, 'a12_insight_flash')
  assert(ranks(a.values.empowered_stun) == '0.5/1/1.5/2', ranks(a.values.empowered_stun))
  local s = status(a, 'a12w_stun')
  assert(s.duration == '$empowered_stun' and s.states[1] == 'stun')
  local branch = a.events.on_cast[2]
  assert(branch.op == 'if' and branch.cond == 'state.a12d_empowered', 'the empowered branch comes first')
  assert(branch.actions[1].op == 'for_each' and branch.actions[1].actions[1].status == 'a12w_stun')
  local spend = branch.actions[2]
  assert(spend.op == 'set_state' and spend.var == 'a12d_empowered' and spend.value == 'false', 'the empowerment is spent')
end)

-- the first action of `list` (depth first) with op `op`
local function find_op(list, op)
  for _, x in ipairs(list or {}) do
    if x.op == op then return x end
    for _, k in ipairs({ 'actions', 'otherwise' }) do
      local hit = find_op(x[k], op)
      if hit then return hit end
    end
  end
end

test('FC-031 B26 kozuke valor restores every non-lethal hit', function()
  local doc = load('B26')
  local a = ability(doc, 'b26_kozuke_valor')
  local gate = find_op(a.passive.triggers.on_take_damage, 'if')
  assert(gate and gate.cond == 'not event.damage.lethal', tostring(gate and gate.cond))
end)

test('FC-070 A23 falling fire calls down at most max_wave (10) waves', function()
  local doc = load('A23')
  local a = ability(doc, 'a23_falling_fire')
  assert(ranks(a.values.max_wave) == '10')
  local rep = find_op(a.events.on_cast, 'repeat')
  assert(rep and rep.count == '$max_wave' and rep['while'] == 'caster.is_channeling', 'the wave loop is capped and still ends with the channel')
end)

test('FC-118 A16 blood aura reflects with the current rank, not the rank it was applied at', function()
  local doc = load('A16')
  local a = ability(doc, 'a16_blood_aura')
  local s = status(a, 'a16r_reflect')
  assert(s.vars == nil, 'no locked level')
  local hit = find_op(s.triggers.on_take_damage.actions or s.triggers.on_take_damage, 'damage')
  assert(hit and hit.amount == 'event.damage.final * ability_level(self, a16_blood_aura) * $reflect_step', tostring(hit and hit.amount))
end)

test('FC-123 A35 flame raid lands on clear ground', function()
  local doc = load('A35')
  local a = ability(doc, 'a35_flame_raid')
  local t = find_op(a.events.on_cast, 'teleport')
  assert(t and t.find_clear_space == true)
end)

-- per-rank numbers of a value as the compiler resolves them (derived values included)
local C = dofile(path.join(tools, 'v3_compile.lua'))
local function numbers(doc, owner, name)
  local out = C.compile(doc, { ir = false })
  local block = out.abilities[owner] or out.items[owner]
  for _, row in ipairs(block.values or {}) do if row.name == name then return table.concat(row.ranks, '/') end end
  error('no value ' .. owner .. '.' .. name)
end
local function item_doc(file)
  local doc = assert(v3.load_file(path.join(b.root, 'docs', 'character_pipeline', 'items', file)))
  local errs = v3.validate_doc(doc)
  assert(#errs == 0, file .. ': ' .. tostring(errs[1]))
  return doc
end
local function item(doc, id)
  for _, it in ipairs(doc.items) do if it.id == id then return it end end
  error('no item ' .. id)
end
local function per_stack_full(doc, owner, per, cap, total)
  -- full stacks give exactly the original fixed bonus, rank by rank
  local p, c, t = numbers(doc, owner, per), numbers(doc, owner, cap), numbers(doc, owner, total)
  local ps, cs, ts = {}, {}, {}
  for x in p:gmatch('[^/]+') do ps[#ps + 1] = tonumber(x) end
  for x in c:gmatch('[^/]+') do cs[#cs + 1] = tonumber(x) end
  for x in t:gmatch('[^/]+') do ts[#ts + 1] = tonumber(x) end
  for i = 1, #ps do
    local cap_i, total_i = cs[i] or cs[#cs], ts[i] or ts[#ts]
    assert(math.abs(ps[i] * cap_i - total_i) < 1e-6, ('%s rank %d: %s x %s ~= %s'):format(owner, i, ps[i], cap_i, total_i))
  end
end

test('FC-038 A14 stacked spear: stacks grant attack, full stacks equal the original +15', function()
  local doc = load('A14')
  local a = ability(doc, 'a14_stacked_spear')
  assert(a.passive.stats.attack_damage == nil, 'no flat bonus')
  local s = status(a, 'a14r_stack')
  assert(s.stacking.value_stacks == true and s.stats.attack_damage.per_stack == true)
  assert(numbers(doc, 'a14_stacked_spear', 'stack_cap') == '5/10/15/20')
  per_stack_full(doc, 'a14_stacked_spear', 'attack_per_stack', 'stack_cap', 'attack_bonus')
end)

test('FC-039 A14 spear fury: attack speed only while cast, per stack, full stacks equal the original', function()
  local doc = load('A14')
  local a = ability(doc, 'a14_spear_fury')
  assert(a.passive.stats == nil or a.passive.stats.attack_speed == nil, 'no permanent attack speed')
  assert(a.passive.triggers.on_attack_hit.filter == 'has_status(self, a14t_rage)', 'stacks only during the rage')
  assert(status(a, 'a14t_aspd_stack').stats.attack_speed.per_stack == true)
  per_stack_full(doc, 'a14_spear_fury', 'aspd_per_stack', 'max_count', 'attack_speed')
  local reset = find_op(status(a, 'a14t_rage').triggers.on_removed, 'set_stacks')
  assert(reset and reset.status == 'a14t_aspd_stack' and reset.count == '0', 'stacks reset when the rage ends')
end)

test('FC-041 B21 undying demon: one stack per 12% health lost, up to 8, full stacks equal the original', function()
  local doc = load('B21')
  local a = ability(doc, 'b21_undying_demon')
  assert(numbers(doc, 'b21_undying_demon', 'stack_max') == '8/8/8')
  per_stack_full(doc, 'b21_undying_demon', 'attack_per_stack', 'stack_max', 'attack')
  per_stack_full(doc, 'b21_undying_demon', 'armor_per_stack', 'stack_max', 'armor')
  local s = status(a, 'b21t_bonus')
  assert(s.stacking.max == '$stack_max' and s.stats.armor.per_stack == true)
end)

test('FC-071 B02 eighteen shuriken throws eighteen', function()
  local doc = load('B02')
  assert(numbers(doc, 'b02_eighteen_shuriken', 'shuriken_count') == '18/18/18/18')
  local rep = find_op(ability(doc, 'b02_eighteen_shuriken').events.on_cast, 'repeat')
  local found = false
  local function walk(list) for _, x in ipairs(list or {}) do if x.op == 'repeat' and x.count == '$shuriken_count' then found = true end walk(x.actions) end end
  walk(ability(doc, 'b02_eighteen_shuriken').events.on_cast)
  assert(rep and found, 'the volley repeats shuriken_count times')
end)

test('FC-044 Ikeda Kijinmaru: one stack per nearby non-hero ally, up to 4, full equals the original', function()
  local doc = item_doc('batch_I4.lua')
  local it = item(doc, 'item_the_great_sword_of_disease')
  assert(numbers(doc, 'item_the_great_sword_of_disease', 'minion_cap') == '4')
  per_stack_full(doc, 'item_the_great_sword_of_disease', 'armor_per_minion', 'minion_cap', 'self_armor')
  per_stack_full(doc, 'item_the_great_sword_of_disease', 'regen_per_minion', 'minion_cap', 'self_regen')
  local s
  for _, x in ipairs(it.statuses) do if x.id == 'disease_with_minion' then s = x end end
  assert(s.stacking.max == '$minion_cap' and s.stats.armor.per_stack == true and s.stats.hp_regen.per_stack == true)
  assert(find_op(it.passive.triggers.periodic.actions, 'set_stacks'), 'the stack count follows the units nearby')
end)

for _, row in ipairs({ { 'FC-113', 'batch_I1.lua', 'item_kousetsusamonnji', 'kousetsusamonnji_count', 'kousetsusamonnji_crit' },
    { 'FC-114', 'batch_I4.lua', 'item_the_great_sword_of_precision', 'the_great_sword_of_precision_count', 'precision_crit' } }) do
  test(row[1] .. ' ' .. row[3] .. ': two copies roll and count their guarantee on their own', function()
    local it = item(item_doc(row[2]), row[3])
    assert(it.unit_state[row[4]].scope == 'item_instance', 'pity counter per copy')
    local crit
    for _, x in ipairs(it.statuses) do if x.id == row[5] then crit = x end end
    assert(crit and crit.scope == 'item_instance', 'crit status per copy')
  end)
end

-- every action of a list, depth first (actions, otherwise, on_absorb ...)
local function all_ops(list, out)
  out = out or {}
  for _, x in ipairs(list or {}) do
    if type(x) == 'table' and x.op then
      out[#out + 1] = x
      for _, k in ipairs({ 'actions', 'otherwise', 'on_hit', 'on_end' }) do all_ops(x[k], out) end
    end
  end
  return out
end
local function routine(doc, name) return assert(doc.routines and doc.routines[name], 'no routine ' .. name) end

test('FC-090 C09 frost thorn adds no extra attack on buildings', function()
  local doc = load('C09')
  local orb = status(ability(doc, 'c09_frost_thorn'), 'c09w_orb')
  for _, x in ipairs(all_ops(orb.triggers.on_attack_windup)) do assert(x.op ~= 'attack', 'no extra PerformAttack') end
end)

test('FC-009 B30 purgatory slash ignites each target on its own clock', function()
  local doc = load('B30')
  assert(doc.unit_state == nil or doc.unit_state.tsum == nil, 'no shared tsum')
  local rep = find_op(routine(doc, 'b30t_ignite').actions, 'repeat')
  assert(rep.count == '$dot_duration / $tsum_step - 1' and rep.as == 'i')
  local apply = find_op(rep.actions, 'apply_status')
  assert(apply.duration == '$dot_duration - (i + 1) * $tsum_step', apply.duration)
end)

test('FC-026 / FC-120 A18 shield absorbs magic damage before it settles', function()
  local doc = load('A18')
  local s = status(ability(doc, 'a18_stubborn_tactics'), 'a18r_shield')
  assert(s.absorb and s.absorb.amount == '$shield' and s.absorb.types[1] == 'magical' and s.absorb.phase == 'post_mitigation')
  assert(s.triggers == nil and s.vars == nil, 'no after-the-fact SetHealth')
end)

test('FC-001 / FC-027 / FC-083 A24 blood shield is private, absorbs, and the charge stun is never negative', function()
  local doc = load('A24')
  assert(doc.unit_state.aphotic_shield_remaining == nil and doc.unit_state.a24_blood_shield, 'private shield variable')
  local s = status(ability(doc, 'a24_blood_shield'), 'a24d_shield')
  assert(s.absorb.amount == 'state.a24_blood_shield', 'absorbs the current shield')
  assert(ability(doc, 'a24_blood_shield').passive.triggers.on_take_damage == nil, 'no SetHealth after damage')
  local pay = find_op(routine(doc, 'a24_blood_price').actions, 'damage')
  assert(pay.flags and pay.flags[1] == 'hp_removal', 'paying health is not absorbed')
  local stun = find_op(ability(doc, 'a24_blood_charge').events.on_cast, 'if')
  local apply = find_op(stun.actions, 'apply_status')
  assert(apply.duration:match('^max%(') and apply.duration:match(', 0%)$'), apply.duration)
end)

-- a status of an ability or item by id
local function status_of(entry, id)
  for _, x in ipairs(entry.statuses or {}) do if x.id == id then return x end end
  error('no status ' .. id)
end

test('shared AphoticShieldRemaining shields (FC-002/032, 003/033, 004/034, 023/115) absorb on their own', function()
  for _, row in ipairs({ { 'B36', 'b36_heart_fence', 'b36e_shield' }, { 'C06', 'c06_lavish_strike', 'c06r_shield' },
      { 'C12', 'c12_space_rift', 'c12e_shield' }, { 'A04', 'a04_frost_shelter', 'a04r_shield' } }) do
    local doc = load(row[1])
    assert(doc.unit_state == nil or doc.unit_state.aphotic_shield_remaining == nil, row[1] .. ': no shared shield variable')
    local st = status(ability(doc, row[2]), row[3])
    assert(st.absorb and st.absorb.amount, row[1] .. ': absorbs')
    assert(st.triggers == nil or st.triggers.on_take_damage == nil, row[1] .. ': no SetHealth after damage')
  end
  -- FC-115: recasting refills (refresh resets the absorb), and the expiry heal reads the tracked rest
  local a04 = status(ability(load('A04'), 'a04_frost_shelter'), 'a04r_shield')
  assert(a04.triggers.on_refreshed[1].var == 'left' and a04.triggers.on_refreshed[1].value == '$shield')
  assert(a04.triggers.on_removed.actions[1].amount == 'vars.left')
end)

test('FC-013 / FC-018 / FC-036 Fubuki shoulder guards: own absorbing shield, independent slows', function()
  local it = item(item_doc('batch_I1.lua'), 'item_fubuki_shoulders')
  assert(it.passive.triggers.on_take_damage == nil, 'no global OnTakeDamage over every holder')
  assert(status_of(it, 'fubuki_ice_shield').absorb.amount == 'state.fubuki_shoulders_shield')
  for _, id in ipairs({ 'fubuki_attack_slow', 'fubuki_nova_slow' }) do assert(status_of(it, id).vars == nil, id .. ': no muting') end
  assert(status_of(it, 'fubuki_reactive_slow').stats.slow_pct == '$reactive_slow')
  assert(numbers(item_doc('batch_I1.lua'), 'item_fubuki_shoulders', 'reactive_slow') == '15')
end)

test('FC-019 / FC-020 / FC-021 lightning items reset only the attacker\'s pity counter', function()
  for _, row in ipairs({ { 'pilot_a.lua', 'item_oochidorijuumonnjiyari' }, { 'batch_I2.lua', 'item_lightning_katana' }, { 'batch_I3.lua', 'item_raikiri' } }) do
    local it = item(item_doc(row[1]), row[2])
    local seen = 0
    local function walk(x)
      if type(x) ~= 'table' then return end
      if x.op == 'set_state' and x.var == 'raikiri' and x.value == '0' then seen = seen + 1; assert(x.on == 'self', row[2] .. ': resets ' .. tostring(x.on)) end
      for _, v in pairs(x) do walk(v) end
    end
    walk(it.passive)
    assert(seen == 1, row[2] .. ': one reset')
  end
end)

test('FC-064 ~ FC-068 The Art of War shares its cooldown with every scroll family both ways', function()
  local it = item(item_doc('pilot_b.lua'), 'item_the_art_of_war')
  local groups = {}
  for _, g in ipairs(it.active.cost.cooldown_groups) do groups[g.group] = g end
  for _, name in ipairs({ 'fire_family', 'wind_family', 'mountain_family', 'forest_family' }) do
    assert(groups[name] and groups[name].role == nil, name .. ': a full member')
  end
end)

test('FC-095 / FC-096 prophet armors block per attack by chance, no stacking charges', function()
  for _, id in ipairs({ 'item_prophet_akazonae_armor', 'item_prophet_armor' }) do
    local it = item(item_doc('batch_I3.lua'), id)
    assert(it.passive.stats.physical_block == '$block' and it.passive.stats.physical_block_chance == '$block_chance', id)
    assert(it.passive.triggers == nil or it.passive.triggers.on_attacked_hit == nil, id .. ': no random walk')
  end
end)

test('FC-097 ~ FC-101 true sight auras and actives keep their own statuses', function()
  for _, row in ipairs({ { 'batch_I1.lua', 'item_insight_gem_shard' }, { 'batch_I3.lua', 'item_prophet_armor' } }) do
    local it = item(item_doc(row[1]), row[2])
    local aura = it.passive.aura.status
    for _, x in ipairs(all_ops(it.active.events.on_cast)) do
      if x.op == 'apply_status' then assert(x.status ~= aura, row[2] .. ': the active uses its own status') end
    end
  end
end)

test('FC-092 / FC-077 soul hyper cuts 50% status resistance; tiger bashes every 5 for ranged', function()
  local doc = item_doc('batch_I4.lua')
  assert(numbers(doc, 'item_the_great_sword_of_soul_hyper', 'active_sr') == '50')
  assert(numbers(doc, 'item_the_great_sword_of_tiger', 'bash_hits_ranged') == '5')
  local found = false
  local function walk(x)
    if type(x) ~= 'table' then return end
    if x.op == 'if' and type(x.cond) == 'string' and x.cond:find('select(self.is_ranged, $bash_hits_ranged, $bash_hits)', 1, true) then found = true end
    for _, v in pairs(x) do walk(v) end
  end
  walk(item(doc, 'item_the_great_sword_of_tiger'))
  assert(found, 'the bash count depends on the attack type')
end)

test('FC-128 moon mirror puts the lantern lockout on the unit it heals', function()
  local it = item(item_doc('batch_I2.lua'), 'item_moon_mirror')
  for _, x in ipairs(all_ops(it.active.events.on_cast)) do
    if x.op == 'apply_status' and x.status == 'moon_mirror_lantern_lock' then assert(x.to ~= 'state.moon_mirror_cur', 'not the previous unit') end
  end
end)

test('FC-112 / FC-043 / FC-127 Tosei Gusoku per copy; Chidori steals stack and decay only after a steal', function()
  local doc = item_doc('batch_I0.lua')
  local armor = item(doc, 'item_contemporary_armor')
  for _, id in ipairs({ 'contemporary_melee_guard', 'contemporary_ranged_guard' }) do
    local st = status_of(armor, id)
    assert(st.scope == 'item_instance' and st.unique_group == 'contemporary_guard', id)
  end
  local chidori = item(doc, 'item_chidori')
  for _, id in ipairs({ 'chidori_int_gain', 'chidori_int_loss' }) do
    local st = status_of(chidori, id)
    assert(st.stacking.value_stacks == true and st.stats.int.per_stack == true, id)
  end
  local top = chidori.passive.triggers.on_attack_hit
  for _, x in ipairs(top) do assert(x.op ~= 'delay', 'the decay is not scheduled for every hit') end
  local branch = top[4]
  assert(branch.op == 'if' and find_op(branch.actions, 'delay'), 'the decay follows a steal')
end)

-- every action anywhere under a table (events, statuses, routines ...)
local function every_op(x, out)
  out = out or {}
  if type(x) ~= 'table' then return out end
  if x.op then out[#out + 1] = x end
  for _, v in pairs(x) do every_op(v, out) end
  return out
end

test('waves 1-4: A26 bombs, A12 field, A11 gaze, C14 slow, B34 howl, A36 brew, C24 self cast, B02 stun, B09 damage, B18 sight, A33 step, B11 illusion', function()
  -- FC-014: each bomb settles as itself
  local a26 = load('A26')
  assert(a26.unit_state.a26t_small_flag == nil, 'no shared small_bomb flag')
  for _, x in ipairs(every_op(ability(a26, 'a26_scorching_bomb'))) do
    if x.op == 'projectile' then
      assert(#x.on_end == 1 and x.on_end[1].op == 'call', 'the bomb settles its own burst')
    end
  end
  -- FC-049: the field negates far damage before it settles, and spends the empowerment
  local a12 = load('A12')
  local field = ability(a12, 'a12_mystic_field')
  assert(status(field, 'a12e_field_guard').triggers.on_take_damage_pre.actions[1].cancel == true)
  local spends = false
  for _, x in ipairs(every_op(field.events.on_cast)) do if x.op == 'set_state' and x.var == 'a12d_empowered' then spends = true end end
  assert(spends, 'the empowerment is spent')
  -- FC-116: the gaze target walks toward Dosan
  local dest = find_op(ability(load('A11'), 'a11_evil_gaze').events.on_cast, 'let')
  assert(dest.value == 'lerp_point(target, caster, $distance_pct)', dest.value)
  -- FC-011: every howl instance slows on its own
  local howl = status(ability(load('B34'), 'b34_tenmoku_howl'), 'b34t_slow_instance')
  assert(howl.stacking == 'independent' and howl.stats.slow_pct == '$slow' and howl.triggers == nil)
  -- FC-029: the floor holds before damage settles
  local brew = status(ability(load('A36'), 'a36_bitter_brew'), 'a36r_brew')
  assert(brew.triggers.on_take_damage == nil and brew.triggers.on_take_damage_pre.actions[1].op == 'modify_damage')
  -- FC-091: a self cast heals and buffs once
  local guarded = 0
  for _, x in ipairs(every_op(ability(load('C24'), 'c24_last_life').events.on_cast)) do
    if x.op == 'if' and x.cond == 'target ~= caster' then guarded = guarded + 1 end
  end
  assert(guarded == 2, 'the target heal and the target buff are skipped on a self cast')
  -- FC-072: the opening stun
  local thunders = ability(load('B02'), 'b02_four_thunders')
  assert(numbers(load('B02'), 'b02_four_thunders', 'first_stun') == '1/1/1')
  local opens = false
  for _, x in ipairs(thunders.events.on_cast) do if x.op == 'apply_status' and x.status == 'b02t_first_stun' then opens = true end end
  assert(opens, 'stuns on cast')
  -- FC-073: KV basicDMG
  assert(numbers(load('B09'), 'b09_backlash_rift', 'fixed_damage') == '20/30/40/50')
  -- FC-053: the claw reveals invisible enemies
  local claw = status(ability(load('B18'), 'b18_earth_claw'), 'b18t_claw_ai')
  assert(claw.aura and claw.aura.status == 'b18t_reveal')
  -- FC-028: ghost step ignores the hit before it settles
  local step = status(ability(load('A33'), 'a33_ghost_step'), 'a33e_ghost_walk')
  assert(step.triggers.on_take_damage == nil and step.triggers.on_take_damage_pre)
  local cancels = false
  for _, x in ipairs(every_op(step.triggers.on_take_damage_pre)) do if x.op == 'modify_damage' and x.cancel then cancels = true end end
  assert(cancels)
  -- FC-052: the illusion gets the caster's remaining time
  local remaining = false
  for _, x in ipairs(every_op(load('B11').routines)) do
    if x.op == 'apply_status' and x.status == 'b11t_enable' and x.duration == 'status_remaining(caster, b11t_enable)' then remaining = true end
  end
  assert(remaining, 'illusion duration is the remaining time')
end)

test('wave 9: C08 steal never loops, counts per Danzo; C07 clouds on their own; B16 hound shares before; B28 serpent cap', function()
  -- FC-058: count the stealable slots first; no loop at all
  local c08 = load('C08')
  local devour = ability(c08, 'c08_devour')
  for _, x in ipairs(every_op(devour)) do assert(x.op ~= 'repeat', 'no reroll loop') end
  local gate
  for _, x in ipairs(every_op(devour)) do if x.op == 'if' and x.cond == 'ok_q + ok_w + ok_e > 0' then gate = x end end
  assert(gate, 'the steal is skipped when no slot is stealable')
  local adds = false
  for _, x in ipairs(every_op(gate.actions)) do if x.op == 'add_ability' then adds = true end end
  assert(adds, 'the steal happens inside the gate')
  -- FC-017: the stack display is the remaining count, the counter is per hero
  assert(devour.unit_state.c08t_use_count.scope == 'hero')
  local shown
  for _, x in ipairs(every_op(devour.passive)) do if x.op == 'set_stacks' then shown = x.count end end
  assert(shown == 'ability.level - state.c08t_use_count', tostring(shown))
  -- FC-016: each cloud reads itself, and only the newest cloud's timer zeroes the move skill
  local wrath = ability(load('C07'), 'c07_raijin_wrath')
  for _, x in ipairs(every_op(status(wrath, 'c07t_storm'))) do
    for _, k in ipairs({ 'cond', 'where' }) do assert(not has(tostring(x[k] or ''), 'state.c07t_storm'), 'a cloud never reads the newest cloud') end
    if x.args then assert(x.args.storm == 'self') end
  end
  local guarded = false
  for _, x in ipairs(every_op(wrath.events.on_cast)) do if x.op == 'if' and x.cond == 'state.c07t_storm == storm' then guarded = true end end
  assert(guarded, 'an old cloud expiring leaves the new cloud movable')
  -- FC-030: the hound takes its share before Chiyome's damage settles
  local union = status(ability(load('B16'), 'b16_beast_union'), 'b16t_union')
  assert(union.triggers.on_take_damage_pre.actions[1].op == 'modify_damage'
    and union.triggers.on_take_damage_pre.actions[1].multiplier == '1 - $share_pct')
  for _, x in ipairs(every_op(union.triggers.on_take_damage)) do assert(x.op ~= 'set_health', 'no after-the-fact refill') end
  -- FC-089: wave_num serpents, at most 4 jumps each
  local b28 = load('B28')
  assert(numbers(b28, 'b28_serpent_fire', 'wave_num') == '4/8/12')
  assert(numbers(b28, 'b28_serpent_fire', 'max_jumps') == '4/4/4')
  local fe = find_op(ability(b28, 'b28_serpent_fire').events.on_cast, 'for_each')
  assert(fe.select.limit == '$wave_num' and fe.select.order == 'nearest')
  assert(find_op(fe.actions, 'chain').jumps == '$max_jumps')
end)

test('items: invincible sword immune, hannya pursuit stacks, saddle 30% cut, haramaki shield, ninjas vanish', function()
  -- FC-037: real damage immunity
  local sword = status_of(item(item_doc('pilot_b.lua'), 'item_the_great_sword_of_invincible'), 'invincible_shield')
  assert(sword.triggers.on_take_damage == nil and sword.triggers.on_take_damage_pre[1].cancel == true)
  -- FC-042: 4 stacks of 7.5%, reset when facing nobody
  local i0 = item_doc('batch_I0.lua')
  local hannya = item(i0, 'item_hannya_onimaru')
  assert(hannya.passive.stats.move_speed_pct == nil, 'no flat always-on speed')
  assert(numbers(i0, 'item_hannya_onimaru', 'move_speed_pct') == '7.5' and numbers(i0, 'item_hannya_onimaru', 'pursuit_max_stacks') == '4')
  local pursuit = status_of(hannya, 'hannya_pursuit')
  assert(pursuit.stacking.value_stacks == true and pursuit.stats.move_speed_pct.per_stack == true)
  assert(status_of(hannya, 'hannya_facing').triggers.on_removed[1].status == 'hannya_pursuit', 'facing nobody resets the stacks')
  -- FC-045: the 30% branch removes its stack count of armor
  local saddle = item(item_doc('batch_I5.lua'), 'item_the_spear_of_saddle')
  local cut = status_of(saddle, 'saddle_pct_cut')
  assert(cut.stacking.value_stacks == true and cut.stats.armor.per_stack == true)
  local sets = find_op(saddle.passive.triggers.on_attack_hit.actions, 'set_stacks')
  assert(sets and sets.count == 'floor(armor_now * $armor_cut_pct)')
  -- FC-062: the active shields allies
  local haramaki = item(i0, 'item_devil_supressor_armor')
  assert(numbers(i0, 'item_devil_supressor_armor', 'shield_duration') == '25')
  assert(find_op(haramaki.active.events.on_cast, 'apply_status').status == 'devil_supressor_shield')
  local shield = status_of(haramaki, 'devil_supressor_shield')
  assert(shield.absorb.amount == '$shield_base + source.level * $shield_per_level' and shield.absorb.types[1] == 'magical')
  -- FC-093 / FC-134: the replaced genin vanishes and gives up its number first
  local ninja1 = item(item_doc('pilot_a.lua'), 'item_ninja1')
  local kills, zeroes = 0, 0
  for _, x in ipairs(every_op(ninja1.active.events.on_cast)) do
    assert(not (x.op == 'remove_status' and x.status == 'ninja1_lifetime'), 'no permanent genin')
    if x.op == 'kill' then kills = kills + 1 end
    if x.op == 'set_state' and x.var == 'item_ninja1_number' and x.value == '0' then zeroes = zeroes + 1 end
  end
  assert(kills == 2 and zeroes == 2)
  -- FC-094: one chunin at a time
  local ninja2 = item(item_doc('batch_I2.lua'), 'item_ninja2')
  local killed = false
  for _, x in ipairs(every_op(ninja2.active.events.on_cast)) do
    assert(not (x.op == 'remove_status' and x.status == 'ninja2_lifetime'), 'no permanent chunin')
    if x.op == 'kill' then killed = true end
  end
  assert(killed)
  assert(status_of(ninja2, 'ninja2_passive').triggers.on_death[1].cond == 'state.ninja2_current', 'only the tracked chunin untracks')
end)

-- true when some table under `node` satisfies `pred`
local function any(node, pred, seen)
  if type(node) ~= 'table' then return false end
  seen = seen or {}
  if seen[node] then return false end
  seen[node] = true
  if pred(node) then return true end
  for _, v in pairs(node) do if any(v, pred, seen) then return true end end
  return false
end
local function op_is(op, fields)
  return function(t)
    if t.op ~= op then return false end
    for k, v in pairs(fields or {}) do if t[k] ~= v then return false end end
    return true
  end
end
local function contains(field, needle)
  return function(t) return type(t[field]) == 'string' and has(t[field], needle) end
end
local function statuses_by_id(doc)
  local out = {}
  any(doc, function(t) if t.id and t.duration and t.stacking then out[t.id] = t end end)
  return out
end

test('waves 5-8: each fix holds in its document', function()
  local cases = {
    { 'FC-010', 'A29', function(d) return statuses_by_id(d).a29r_slow.stacking == 'independent' end, true },
    { 'FC-122', 'A29', op_is('let', { name = 'lost', value = 'min(self.mana, $mana_lost)' }) },
    { 'FC-015', 'B13', function(t) return t.filter == 'state.b13e_wake_armed' and any(t.actions, op_is('remove_status', { from = 'self', status = 'b13e_sleep' })) end },
    { 'FC-024', 'A05', function(t) return t.on_take_damage_pre and any(t.on_take_damage_pre, op_is('modify_damage', { cancel = true })) end },
    { 'FC-080', 'A05', op_is('remove_status', { from = 'caster', status = 'a05t_oath' }) },
    { 'FC-035', 'C23', function(t) return t.on_take_damage_pre and any(t.on_take_damage_pre, op_is('modify_damage', { cancel = true })) end },
    { 'FC-040', 'A31', function(d) local st = statuses_by_id(d).a31w_slow; return st.stacking.value_stacks and st.stats.move_speed_pct.per_stack end, true },
    { 'FC-051', 'A15', op_is('damage', { type = 'physical', amount = 'avg_attack_against(self, it) * $split_ratio' }) },
    { 'FC-117', 'A15', contains('cond', 'stacks(it, a15e_hit) < $a15_moon_strike.max_hit') },
    { 'FC-056', 'B32', op_is('let', { name = 'n', value = 'max(floor(len / ($path_radius * $two)) + 1, $two)' }) },
    { 'FC-057', 'B33', function(t) return t.heal_reduction_pct == '$heal_cut' end },
    { 'FC-059', 'C11', function(t) return t.shape == 'circle' and type(t.radius) == 'string' and has(t.radius, 'jump_range') end },
    { 'FC-060', 'C11', op_is('apply_status', { status = 'c11t_stun' }) },
    { 'FC-061', 'C21', function(d) return numbers(d, 'c21_soaring_quake', 'slash_count') == '8/12/16' end, true },
    { 'FC-074', 'B12', contains('cond', 'not (is_ally(caster, it) and not it.is_hero)') },
    { 'FC-075', 'C12', function(t) return t.remove_on and t.cue == 'c12r_mask' and t.remove_on[1] == 'attack' and t.remove_on[2] == 'cast' end },
    { 'FC-111', 'C12', op_is('remove_status', { from = 'self', status = 'c12d_count' }) },
    { 'FC-076', 'C19', function(t) return t.cue == 'c19t_aura_fx' and t.states and t.states[1] == 'untargetable_by_spells' end },
    { 'FC-079', 'A04', function(t) return t.base_attack_time ~= nil end },
    { 'FC-081', 'A13', op_is('swap_abilities', { cooldown_policy = 'continue' }) },
    { 'FC-082', 'A13', function(d) return not any(d, function(t) return t.on_phase_start and any(t.on_phase_start, op_is('apply_status', { status = 'a13w_invulnerable' })) end) end, true },
    { 'FC-132', 'A13', function(t) return t.type == 'unit' and t.team == 'ally' and t.types and t.types[1] == 'hero' and t.range == nil end },
    { 'FC-085', 'A32', op_is('swap_abilities', { cooldown_policy = 'continue' }) },
    { 'FC-088', 'B24', op_is('swap_abilities', { cooldown_policy = 'continue' }) },
    { 'FC-108', 'B24', op_is('remove_status', { from = 'self', status = 'b24e_taunt' }) },
    { 'FC-109', 'B24', function(d) local r = statuses_by_id(d).b24e_taunt.remove_on; return r and r[1] == 'owner_death' end, true },
    { 'FC-084', 'A28', function(d) return statuses_by_id(d).a28t_lifetime.dispel == 'none' end, true },
    { 'FC-086', 'B17', contains('cond', 'event.target.is_hero and not event.target.is_illusion and not alive(event.target)') },
    { 'FC-087', 'B23', op_is('remove_status', { from = 'caster', status = 'b23r_magic_immune' }) },
    { 'FC-107', 'B23', function(d) return statuses_by_id(d).b23d_damage_reduction.dispel == 'none' end, true },
    { 'FC-103', 'A06', op_is('set_ability_active', { ability = 'a06_flame_of_anger', active = false }) },
    { 'FC-105', 'B21', op_is('call', { routine = 'b21t_update' }) },
    { 'FC-119', 'A17', function(t) return t.on_attacked_hit and t.on_attacked_hit.filter == 'event.attack.is_ranged' end },
    { 'FC-124', 'B07', op_is('set_health', { to = 'u', value = 'u.max_hp' }) },
    { 'FC-125', 'B22', op_is('remove_status', { from = 'target', dispel = 'all', team_of_source = 'enemy' }) },
  }
  for _, c in ipairs(cases) do
    local fc, sid, pred, whole = c[1], c[2], c[3], c[4]
    local doc = load(sid)
    local ok
    if whole then ok = pred(doc) else ok = any(doc, pred) end
    assert(ok, fc .. ' ' .. sid .. ': the fix is not in the document')
  end
  -- toggles switch off at death so a respawn never shows an active toggle without its effect (FC-102/104/110/133)
  for _, sid in ipairs({ 'A03', 'B15', 'C07', 'A32' }) do
    assert(any(load(sid), function(t) return t.on_death and any(t.on_death, op_is('toggle_ability', { on = false })) end), sid .. ': no toggle-off at death')
  end
end)

test('listed heroes: B08 leaves other shields alone, A07 negates before settling and has its pity crit', function()
  -- FC-005
  local b08 = load('B08')
  assert(not any(b08, contains('var', 'aphotic_shield_remaining')), 'B08R never touches the shared shield field')
  -- FC-025
  local guard = statuses_by_id(load('A07')).a07w_convert
  assert(guard.triggers.on_take_damage == nil, 'no after-the-fact refill')
  assert(any(guard.triggers.on_take_damage_pre, op_is('modify_damage', { cancel = true })), 'negated before it settles')
  assert(guard.triggers.on_take_damage_pre.filter == 'is_enemy(event.attacker, self) and not event.damage.is_physical', 'lethal hits too')
  -- FC-046
  local spear = ability(load('A07'), 'a07_shattering_spear')
  assert(spear.unit_state.a07r_noncrit.type == 'int')
  assert(any(spear.passive, contains('cond', 'state.a07r_noncrit >= $misses')), 'the pity crit')
end)

print(('%d tests passed'):format(count))
