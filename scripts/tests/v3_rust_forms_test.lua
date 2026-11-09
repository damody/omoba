-- v3_rust.lua forms on the host APIs (status_event, scheduled_effect, unit_var, damage_pre,
-- hero_command): across every hero document, each form is generated somewhere, in the shape the
-- host expects. tools\lua\lua.exe scripts\tests\v3_rust_forms_test.lua
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
local function has(text, needle) return text:find(needle, 1, true) ~= nil end

-- every generated entry file of every hero document
local texts = {}
for _, h in ipairs(sources.heroes(b.root)) do
  local doc = assert(v3.load_file(h.path))
  local files = R.generate(doc, { root = b.root, rules = v3.rules(), package = h.package or 'x' })
  for rel, text in pairs(files) do
    local id = rel:match('^src/generated/([%w_]+)%.rs$')
    if id and id ~= 'registry' and id ~= 'values' and id ~= 'ids' and id ~= 'routines' and id ~= 'mod' then texts[#texts + 1] = { id = id, text = text } end
  end
end
local function some(pred)
  for _, t in ipairs(texts) do if pred(t.text) then return t end end
end

test('a delay schedules with its captures and comes back through on_scheduled', function()
  local t = assert(some(function(s) return has(s, 'cx.schedule(owner, ID, ') and has(s, 'fn on_scheduled(') end), 'a delay')
  assert(has(t.text, 'args.as_slice()'), t.id)
end)

test('a status trigger marks the status and answers on_status_event', function()
  assert(some(function(s) return has(s, 'fn on_status_event(&self, holder: EntityHandle, status: RStr') end), 'a status event hook')
end)

test('state variables read with their init and write with their scope', function()
  assert(some(function(s) return has(s, 'cx.state_num(') or has(s, 'cx.state_bool(') end), 'a read')
  assert(some(function(s) return has(s, 'cx.set_state_num(') or has(s, 'cx.set_state_bool(') or has(s, 'cx.set_state_unit(') end), 'a write')
end)

test('a damage-pre trigger returns its edit from on_damage_pre', function()
  local t = assert(some(function(s) return has(s, 'fn on_damage_pre(') end), 'a damage-pre hook')
  assert(has(t.text, 'cx.damage_edit'), t.id)
end)

test('chance rolls the host RNG', function()
  assert(some(function(s) return has(s, 'if cx.chance(p) {') end), 'a chance')
end)

test('a passive listening to its holder carries a marker with its events', function()
  local t = some(function(s) return s:find('cx%.passive_marker%(ID, .-, [1-9]%d*%)') ~= nil end)
  if t then assert(has(t.text, 'fn on_status_event(') or has(t.text, 'fn on_damage_pre('), t.id .. ': the marker events come back')
  else print('  (no generated passive listens to its holder yet)') end
end)

test('requires.has_status refuses the cast first', function()
  local t = some(function(s) return has(s, 'v3: requires has_status') end)
  if t then
    local cast = t.text:match("fn on_cast%(cx: &mut Cx<'_, '_>%) %-> Res {\n(.-)\n")
    assert(cast and has(cast, 'cx.has_status(cx.caster'), t.id .. ': the check is the first line of the cast')
  else print('  (no generated cast with requires.has_status yet)') end
end)

test('every damage carries its ability and v3 effect path (line B damage trace)', function()
  local n = 0
  for _, t in ipairs(texts) do
    assert(not has(t.text, 'cx.damage(to,'), t.id .. ': a damage without its path')
    for _, epath in t.text:gmatch('cx%.damage_at%("([%w_]+)", "([^"]*)"') do
      n = n + 1
      assert(epath ~= '?' and epath:sub(-1) == ']', t.id .. ': path ' .. epath)
    end
  end
  assert(n > 0, 'some generated damage')
end)

test('ability_level reads the host rank (ext)', function()
  assert(some(function(s) return has(s, 'cx.ability_level(u, ') end), 'an ability_level read')
end)

test('a taunt status taunts through cx.taunt and its removal ends it', function()
  local h
  for _, x in ipairs(sources.heroes(b.root)) do if x.package == 'akiyama_nobutomo' then h = x end end
  local doc = assert(v3.load_file(h.path))
  local ab
  for _, a in ipairs(doc.abilities) do if a.id == 'b24_taunt' then ab = a end end
  -- only the taunt: drop the parts the generator still blocks (include_illusions, the stop order)
  local each = ab.events.on_cast[1]
  local flags = {}
  for _, fl in ipairs(each.select.flags or {}) do if fl ~= 'include_illusions' then flags[#flags + 1] = fl end end
  each.select.flags = flags
  local branch = each.actions[2].actions[1].actions[1]
  branch.otherwise = nil
  each.actions[3] = nil -- the later stop order
  doc.abilities = { ab }
  local files = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'x' })
  local text = assert(files['src/generated/b24_taunt.rs'], 'b24_taunt generates')
  assert(has(text, 'cx.taunt(to, cx.caster, duration)?;'), 'the apply taunts')
  assert(has(text, 'given by the taunt above'), 'the attack order is the taunt')
  assert(has(text, 'cx.end_taunt(from)?;'), 'removing the status ends the taunt')
end)

test('remove_status max_elapsed removes only the young statuses through the kit', function()
  local h
  for _, x in ipairs(sources.heroes(b.root)) do if x.package == 'imagawa_yoshimoto' then h = x end end
  local doc = assert(v3.load_file(assert(h, 'imagawa_yoshimoto').path))
  local hit, ab
  local function find(list, owner)
    for _, a in ipairs(list or {}) do
      if type(a) == 'table' then
        if a.op == 'remove_status' and a.dispel == 'all' and not hit then hit, ab = a, owner end
        for _, k in ipairs({ 'actions', 'otherwise' }) do find(a[k], owner) end
      end
    end
  end
  for _, x in ipairs(doc.abilities) do for _, list in pairs(x.events or {}) do find(list, x) end end
  assert(hit, 'a dispel')
  hit.tags = { 'slow' }; hit.max_elapsed = '0.1'
  doc.abilities = { ab }
  local files = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'x' })
  local text = assert(files['src/generated/' .. ab.id .. '.rs'], ab.id .. ' generates')
  assert(has(text, 'cx.remove_statuses_younger(from, ability_kit::StatusFilter {'), 'the kit call')
  assert(has(text, '&["slow"], &[], Some(max_elapsed))?;'), 'tags and the age')
end)

test('chain on_end runs once when the bolt cannot jump, with the last unit and its jump', function()
  local h
  for _, x in ipairs(sources.heroes(b.root)) do if x.package == 'matsunaga_hisahide' then h = x end end
  local doc = assert(v3.load_file(assert(h, 'matsunaga_hisahide').path))
  local chain, ab
  local function find(list, owner)
    for _, a in ipairs(list or {}) do
      if type(a) == 'table' then
        if a.op == 'chain' then chain, ab = a, owner end
        for _, k in ipairs({ 'actions', 'otherwise' }) do find(a[k], owner) end
      end
    end
  end
  for _, x in ipairs(doc.abilities) do for _, list in pairs(x.events or {}) do find(list, x) end end
  assert(chain, 'a chain')
  chain.on_end = { { op = 'if', cond = 'jump > 1', actions = {
    { op = 'damage', to = 'it', type = 'magical', amount = '$base_damage * jump' } } } }
  doc.abilities = { ab }
  local files = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'x' })
  local text = assert(files['src/generated/' .. ab.id .. '.rs'], ab.id .. ' generates')
  assert(has(text, ' > jumps + Fixed64::ONE { None } else {'), 'out of jumps ends the bolt')
  assert(has(text, ' - Fixed64::ONE;'), 'the end jump is the last hit')
  assert(has(text, '<= jumps + Fixed64::ONE {'), 'the last hit schedules the end')
  assert(has(text, '.unwrap_or('), 'a unit gone keeps the bolt where it was')
  local _, damages = text:gsub('cx%.damage', '')
  assert(damages >= 3, 'the end body deals its damage (' .. damages .. ')')
end)

test('chain on_end that reads neither it nor jump binds them unused', function()
  local h
  for _, x in ipairs(sources.heroes(b.root)) do if x.package == 'matsunaga_hisahide' then h = x end end
  local doc = assert(v3.load_file(assert(h, 'matsunaga_hisahide').path))
  local chain, ab
  local function find(list, owner)
    for _, a in ipairs(list or {}) do
      if type(a) == 'table' then
        if a.op == 'chain' then chain, ab = a, owner end
        for _, k in ipairs({ 'actions', 'otherwise' }) do find(a[k], owner) end
      end
    end
  end
  for _, x in ipairs(doc.abilities) do for _, list in pairs(x.events or {}) do find(list, x) end end
  assert(chain, 'a chain')
  chain.on_end = { { op = 'damage', to = 'caster', type = 'magical', amount = '$base_damage' } }
  doc.abilities = { ab }
  local files = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'x' })
  local text = assert(files['src/generated/' .. ab.id .. '.rs'], ab.id .. ' generates')
  assert(has(text, 'let _l_it = l_chain_prev_;'), 'the unread end unit')
  assert(has(text, 'let _l_jump = l_jump - Fixed64::ONE;'), 'the unread end jump')
end)

test('ability.cast_range outside the cast falls back to this rank of target.range', function()
  local t = some(function(s) return has(s, 'match cx.cast_range { Some(r) => r, None => super::values::') end)
  assert(t, 'a cast range read with its declared range')
  assert(not some(function(s) return has(s, 'cx.cast_range.unwrap_or(') end), 'no read falls back to zero alone')
end)

local function saika_corps()
  local h
  for _, x in ipairs(sources.heroes(b.root)) do if x.package == 'saika_magoichi' then h = x end end
  local doc = assert(v3.load_file(assert(h, 'saika_magoichi').path))
  local ab, make
  for _, x in ipairs(doc.abilities) do
    for _, a in ipairs((x.events or {}).on_cast or {}) do if a.op == 'create_unit' then ab, make = x, a end end
  end
  assert(ab, 'a create_unit')
  return doc, ab, make
end

test('create_unit count without as makes that many units one by one', function()
  local doc, ab, make = saika_corps()
  make.as = nil
  local files = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'x' })
  local text = assert(files['src/generated/' .. ab.id .. '.rs'], ab.id .. ' generates')
  assert(has(text, '.min(ability_kit::MAX_REPEAT);'), 'the count, bounded')
  assert(has(text, 'for _ in 0..count {'), 'the loop')
  assert(not has(text, 'makes one unit at a time'), 'no runtime refusal')
end)

test('create_unit lifetime over an hour blocks at generation', function()
  local doc, ab, make = saika_corps()
  make.lifetime = 3601
  local _, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'x' })
  local blockers
  for _, e in ipairs(report.entries) do if e.id == ab.id then blockers = table.concat(e.blockers, '; ') end end
  assert(blockers and blockers:find('create_unit.lifetime over 3600 s', 1, true), 'the blocker: ' .. tostring(blockers))
end)

local function enemy_sweep()
  for _, x in ipairs(sources.heroes(b.root)) do
    local doc = v3.load_file(x.path)
    for _, ab in ipairs(doc and doc.abilities or {}) do
      for _, a in ipairs((ab.events or {}).on_cast or {}) do
        if a.op == 'for_each' and a.select and a.select.team == 'enemy' and a.select.radius and not a.select.where then return doc, ab, a.select end
      end
    end
  end
  error('an enemy for_each')
end

test('selector exclude_types building takes every class but buildings', function()
  local doc, ab, sel = enemy_sweep()
  sel.types, sel.exclude_types = { 'all' }, { 'building' }
  local files = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'x' })
  local text = assert(files['src/generated/' .. ab.id .. '.rs'], ab.id .. ' generates')
  assert(has(text, 'types: UnitTypes::NonBuilding'), 'non-buildings')
end)

test('selector types hero+building+tower skips every other class', function()
  local doc, ab, sel = enemy_sweep()
  sel.types, sel.exclude_types = { 'hero', 'building', 'tower' }, nil
  local files = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'x' })
  local text = assert(files['src/generated/' .. ab.id .. '.rs'], ab.id .. ' generates')
  assert(has(text, 'types: UnitTypes::All'), 'every class')
  assert(has(text, '!cx.is_hero(') and has(text, '!cx.is_building('), 'the others skipped')
end)

test('game.creep_wave, game.creep_armor_bonus, is_summon and illusion_damage read the host', function()
  local doc, ab, sel = enemy_sweep()
  sel.where = 'it.is_summon'
  for _, x in ipairs(ab.events.on_cast) do
    if x.select == sel then
      x.actions = { { op = 'damage', to = 'it', type = 'magical', amount = 'game.creep_wave + game.creep_armor_bonus + it.illusion_damage' } }
    end
  end
  local files = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'x' })
  local text = assert(files['src/generated/' .. ab.id .. '.rs'], ab.id .. ' generates')
  assert(has(text, 'cx.game_num(ability_kit::game_attr::CREEP_WAVE)'), 'the creep wave')
  assert(has(text, 'cx.game_num(ability_kit::game_attr::CREEP_ARMOR_BONUS)'), 'the creep armor bonus')
  assert(has(text, 'ability_kit::unit_attr::IS_SUMMON'), 'is_summon')
  assert(has(text, 'ability_kit::unit_attr::ILLUSION_DAMAGE'), 'illusion_damage')
end)

test('a toggle item registers as a toggle, runs its active on_unequip and reads the host ability tags', function()
  local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/pilot_b.lua'))
  local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
  for _, e in ipairs(report.entries) do
    if e.id == 'item_thunderbolt_book' or e.id == 'item_the_over_lightning_art_of_war' then
      assert(#e.blockers == 0, e.id .. ': ' .. table.concat(e.blockers, '; '))
    end
  end
  local book = assert(files['src/generated/item_thunderbolt_book.rs'], 'the book generates')
  assert(has(book, 'ability_kit::item_kind_ffi(ID, "toggle", '), 'registered as a toggle item')
  assert(has(book, 'fn on_unequip_active(') and has(book, 'on_unequip_active(&mut cx)'), 'its active on_unequip')
  assert(has(book, 'cx.event_ability_has_tag("toggle")') and has(book, 'cx.event_ability_has_tag("no_cast_trigger")'), 'the host tags')
  local over = assert(files['src/generated/item_the_over_lightning_art_of_war.rs'], 'the art of war generates')
  assert(has(over, '(StatKey::CooldownPercentage, (Fixed64::ZERO - ('), 'cooldown reduction as a negative cooldown percentage')
end)

test('an on_order_targeted that cancels runs as the host order filter and refuses the order', function()
  local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/batch_I3.lua'))
  local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
  for _, e in ipairs(report.entries) do
    if e.id == 'item_protection_amulet' then assert(#e.blockers == 0, e.id .. ': ' .. table.concat(e.blockers, '; ')) end
  end
  local text = assert(files['src/generated/item_protection_amulet.rs'], 'the amulet generates')
  assert(has(text, 'fn st_protection_amulet_ready_on_order_filter('), 'the order filter trigger')
  assert(has(text, '("protection_amulet_ready", 21) =>'), 'served on status event ORDER_FILTER')
  assert(not has(text, 'on_order_targeted('), 'not the after-windup event')
  assert(has(text, 'cx.cancel_order()?;'), 'cancel_cast refuses the order')
  assert(has(text, 'cx.event_ability_cooldown_start(of)?;'), "the orderer pays the ordered ability's cooldown")
  assert(has(text, 'cx.event_ability_mana_cost()') and has(text, 'cx.burn_mana(from, amount)?;'), 'and its mana, as a burn')
  assert(has(text, 'cx.event_ability_has_tag("ultimate")'), 'ultimates pass')
  assert(has(text, 'cx.item_cooldown_start(of, "item_protection_amulet", amount)?;'), 'the amulet cools down in its slots')
  assert(has(text, 'cx.item_cooldown_remaining(cx.caster, "item_protection_amulet") <= Fixed64::ZERO'), 'its readiness')
  assert(has(text, 'ability_kit::item_kind_ffi(ID, "passive", '), 'registered as a passive item')
end)

test('a pre_mitigation shield on an independent status absorbs as the instance it applied', function()
  local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/pilot_b.lua'))
  local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
  for _, e in ipairs(report.entries) do
    if e.id == 'item_magic_talisman' then assert(#e.blockers == 0, e.id .. ': ' .. table.concat(e.blockers, '; ')) end
  end
  local text = assert(files['src/generated/item_magic_talisman.rs'], 'the talisman generates')
  assert(has(text, 'let n = cx.apply_status_instance(to, &super::ids::status::MAGIC_TALISMAN_SHIELD, duration, stats)?;'), 'the instance applied')
  assert(has(text, 'cx.absorb_with(to, &super::ids::status::MAGIC_TALISMAN_SHIELD, n, absorb, &["magical"], duration, true)?;'), 'its own pre-mitigation shield')
end)

test('a shield a source applies is that source\'s, and a bare item generates as its catalog row', function()
  local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/batch_I0.lua'))
  local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
  for _, e in ipairs(report.entries) do
    if e.id == 'item_devil_supressor_armor' then assert(#e.blockers == 0, e.id .. ': ' .. table.concat(e.blockers, '; ')) end
  end
  local text = assert(files['src/generated/item_devil_supressor_armor.rs'], 'the armor generates')
  assert(has(text, 'cx.apply_status_from(source, to, &super::ids::status::DEVIL_SUPRESSOR_SHIELD'), 'applied by its source')
  assert(has(text, 'cx.caster = source;') and has(text, 'cx.absorb_with(to, &super::ids::status::DEVIL_SUPRESSOR_SHIELD, 0, absorb, &["magical"], duration, true);'), 'the source\'s pre-mitigation shield')
  local orb = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/batch_I1.lua'))
  local _, orb_report = R.generate(orb, { root = b.root, rules = v3.rules(), package = 'items' })
  for _, e in ipairs(orb_report.entries) do
    if e.id == 'item_hama_orb' then assert(#e.blockers == 0, e.id .. ': ' .. table.concat(e.blockers, '; ')) end
  end
end)

test('the shop catalog names a source key with an apostrophe by its v3 id', function()
  local catalog = R.content_catalog(b.root)
  assert(catalog.items.item_samurais_badge, "item_samurai's_badge is item_samurais_badge")
end)

test('remove_status by source and duration, and an item passive hearing its holder\'s casts', function()
  local function gen(file, id)
    local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/' .. file))
    local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
    for _, e in ipairs(report.entries) do
      if e.id == id then assert(#e.blockers == 0, e.id .. ': ' .. table.concat(e.blockers, '; ')) end
    end
    return assert(files['src/generated/' .. id .. '.rs'], id .. ' generates')
  end
  local storm = gen('batch_I1.lua', 'item_ice_storm_book')
  assert(has(storm, 'cx.remove_statuses_by_source(from, '), 'by source')
  assert(has(storm, 'ability_kit::SourceFilter { min_duration, exclude_source_tags: &["ultimate"], source_is: &["has_ability", "alive"] }'), 'its source filter')
  local bizen = gen('batch_I0.lua', 'item_bizennosafunekanemitsu')
  assert(has(bizen, 'cx.passive_marker(ID, &[], 128);'), 'the marker hears ABILITY_EXECUTED')
  assert(has(bizen, '("item_bizennosafunekanemitsu", 9) => {'), 'served on kind 9')
end)

test('an item hears its holder\'s death: active on_owner_death, item_count, pick_item slots all', function()
  local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/batch_I1.lua'))
  local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
  for _, e in ipairs(report.entries) do
    if e.id == 'item_insight_gem_shard' then
      for _, x in ipairs(e.blockers) do assert(x:match('^catalog:'), e.id .. ': ' .. x) end
    end
  end
  local shard = assert(files['src/generated/item_insight_gem_shard.rs'], 'the shard generates')
  assert(has(shard, 'cx.passive_marker(ID, &[], 524288);'), 'the marker hears OWNER_DEATH')
  assert(has(shard, '("item_insight_gem_shard", 22) => {'), 'served on kind 22')
  assert(has(shard, '.filter(|&slot| cx.item_in_slot(u, slot).as_deref() == Some("item_insight_gem_hero")).count()'), 'item_count')
  assert(has(shard, 'cx.remove_picked_item(p)?;'), 'removes what the pick found')
end)

test('an item passive hears its holder\'s death and respawn', function()
  local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/pilot_b.lua'))
  local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
  for _, e in ipairs(report.entries) do
    if e.id == 'item_death_of_scepter' then
      for _, x in ipairs(e.blockers) do assert(x:match('^catalog:'), e.id .. ': ' .. x) end
    end
  end
  local scepter = assert(files['src/generated/item_death_of_scepter.rs'], 'the scepter generates')
  assert(has(scepter, '], 1573376);'), 'the marker hears KILL, OWNER_DEATH and RESPAWN')
  assert(has(scepter, '("item_death_of_scepter", 22) => {'), 'death on kind 22')
  assert(has(scepter, '("item_death_of_scepter", 23) => {'), 'respawn on kind 23')
end)

test('an item instance\'s bool / unit state, read and written in its repeat steps', function()
  local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/batch_I2.lua'))
  local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
  for _, e in ipairs(report.entries) do
    if e.id == 'item_moon_mirror' then
      for _, x in ipairs(e.blockers) do assert(x:match('^catalog:'), e.id .. ': ' .. x) end
    end
  end
  local mirror = assert(files['src/generated/item_moon_mirror.rs'], 'the mirror generates')
  assert(has(mirror, 'if let Some(k) = cx.item_instance_key("batch_i2:moon_mirror_cur") { cx.set_state_unit(on, &k, value, false)?; }'), 'unit write')
  assert(has(mirror, 'match cx.item_instance_key("batch_i2:moon_mirror_live") { Some(k) => cx.state_bool(h, &k, false), None => false }'), 'bool read')
end)

test('an aura over heroes and buildings selects HeroesAndBuildings', function()
  local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/batch_I1.lua'))
  local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
  for _, e in ipairs(report.entries) do
    if e.id == 'item_insight_gem_hero' then
      for _, x in ipairs(e.blockers) do assert(x:match('^catalog:'), e.id .. ': ' .. x) end
    end
  end
  local gem = assert(files['src/generated/item_insight_gem_hero.rs'], 'the gem generates')
  assert(has(gem, 'UnitTypes::HeroesAndBuildings'), 'building+hero')
end)

test('an item registers under its v3 id, not its source key', function()
  local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/batch_I1.lua'))
  local files = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
  local shard = assert(files['src/generated/item_insight_gem_shard.rs'], 'the shard generates')
  assert(has(shard, 'pub const ID: &str = "item_insight_gem_shard";'), 'its v3 id')
end)

test('an item cast refunds its mana and cooldown', function()
  local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/batch_I3.lua'))
  local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
  for _, e in ipairs(report.entries) do
    if e.id == 'item_soul_adder' then
      for _, x in ipairs(e.blockers) do assert(x:match('^catalog:'), e.id .. ': ' .. x) end
    end
  end
  local adder = assert(files['src/generated/item_soul_adder.rs'], 'the adder generates')
  assert(has(adder, 'cx.refund_mana();'), 'mana')
  assert(has(adder, 'cx.refund_cooldown();'), 'cooldown')
end)

test('a projectile with a lifetime launches as projectile_expiring', function()
  local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/pilot_b.lua'))
  local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
  for _, e in ipairs(report.entries) do
    if e.id == 'item_ronin_jitte' then
      for _, x in ipairs(e.blockers) do assert(x:match('^catalog:'), e.id .. ': ' .. x) end
    end
  end
  local jitte = assert(files['src/generated/item_ronin_jitte.rs'], 'the jitte generates')
  assert(has(jitte, 'cx.projectile_expiring(ID, p, '), 'its lifetime')
end)

test('not primary_is, an event-typed damage and an item orb slot released on unequip', function()
  local function gen(file, id)
    local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/' .. file))
    local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
    for _, e in ipairs(report.entries) do
      if e.id == id then
        for _, x in ipairs(e.blockers) do assert(x:match('^catalog:'), e.id .. ': ' .. x) end
      end
    end
    return assert(files['src/generated/' .. id .. '.rs'], id .. ' generates')
  end
  local sword = gen('pilot_b.lua', 'item_the_great_sword_of_invincible')
  assert(has(sword, 'ability_kit::When::NotPrimary(ability_kit::Attr::Agility)'), 'not primary_is')
  local teeth = gen('pilot_a.lua', 'item_the_dark_yellow_teeth_and_zimbabwe')
  assert(has(teeth, 'ability_kit::event_damage_type(cx.event_damage_kind), Dmg {'), 'the event hit type')
  local spear = gen('batch_I5.lua', 'item_the_spear_of_saddle')
  assert(has(spear, 'if cx.orb_slot_free(caster, ID) { let _ = cx.orb_slot_claim(caster, ID); }'), 'the first copy claims')
  assert(has(spear, 'let _ = cx.orb_slot_release(caster, ID); }'), 'the last copy releases')
end)

test('an item with recharging uses generates (the catalog row keeps them)', function()
  local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/pilot_a.lua'))
  local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
  for _, e in ipairs(report.entries) do
    if e.id == 'item_earth_wall_book' then
      for _, x in ipairs(e.blockers) do assert(x:match('^catalog:'), e.id .. ': ' .. x) end
    end
  end
  assert(files['src/generated/item_earth_wall_book.rs'], 'the book generates')
end)

test('an item unit target without heroes and has_tag courier', function()
  local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/pilot_b.lua'))
  local files, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
  for _, e in ipairs(report.entries) do
    if e.id == 'item_the_great_sword_of_teleport' then
      for _, x in ipairs(e.blockers) do assert(x:match('^catalog:'), e.id .. ': ' .. x) end
    end
  end
  local sword = assert(files['src/generated/item_the_great_sword_of_teleport.rs'], 'the sword generates')
  assert(has(sword, 'cx.has_unit_tag(u, "courier")'), 'courier tag')
end)

test('an item active may write its bot policy (no code, no blocker)', function()
  local doc = assert(v3.load_file(b.root .. '/docs/character_pipeline/items/batch_I1.lua'))
  local _, report = R.generate(doc, { root = b.root, rules = v3.rules(), package = 'items' })
  for _, e in ipairs(report.entries) do
    if e.id == 'item_great_dragon' then
      for _, x in ipairs(e.blockers) do assert(x:match('^catalog:'), e.id .. ': ' .. x) end
    end
  end
end)

test('the courier template is a summon row the host spawns as a courier', function()
  package.loaded['v3_summons'] = nil
  local S = require('v3_summons')
  local rows = S.build(b.root, S.existing(b.root))
  local row
  for _, r in ipairs(rows) do if r.id == 'courier' then row = r end end
  assert(row, 'a courier row')
  assert(row.courier == true and row.magic_immune == true and row.no_attack == true, 'courier, magic immune, no attack')
  assert(row.move_speed == 550 and row.turn_rate == 0.5 and row.collision_radius == 0 and row.owner_death == 'persist', 'its numbers')
end)

print(('%d tests passed'):format(count))
