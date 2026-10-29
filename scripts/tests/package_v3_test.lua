-- Package schema v3 validator tests (docs/character_pipeline/schema/v3.md).
-- tools\lua\lua.exe scripts\tests\package_v3_test.lua
package.path = 'scripts/?.lua;' .. package.path
local b = require('_bootstrap')
local path = b.lib('path')
local lfs = require('lfs')

local tools = path.join(b.root, 'docs', 'character_pipeline', 'tools')
local sources = dofile(path.join(tools, 'v3_sources.lua'))
PACKAGE_V3_AS_MODULE = true
local v3 = dofile(path.join(tools, 'package_v3.lua'))
VALIDATE_PACKAGE_AS_MODULE = true
local validator = dofile(path.join(tools, 'validate_package.lua'))
VALIDATE_PACKAGE_AS_MODULE, PACKAGE_V3_AS_MODULE = nil, nil

local count = 0
local function test(name, fn) fn(); count = count + 1; print('PASS ' .. name) end

local function deep_copy(t)
  if type(t) ~= 'table' then return t end
  local c = {}
  for k, v in pairs(t) do c[k] = deep_copy(v) end
  return c
end
local base_doc = assert(v3.load_file(assert(sources.hero_path(b.root, 'C01'), 'no C01 document')))
local function doc() return deep_copy(base_doc) end
local function ability(d, id)
  for _, a in ipairs(d.abilities) do if a.id == id then return a end end
  error('no ability ' .. id)
end
local function errs(d) return table.concat((v3.validate_doc(d)), '\n') end
local function expect(d, fragment)
  local all = errs(d)
  assert(all:find(fragment, 1, true), 'expected error containing "' .. fragment .. '", got:\n' .. all)
end
local function clean(d)
  local all = errs(d)
  assert(all == '', 'expected no errors, got:\n' .. all)
end

-- every v3 document at its authoring location validates (v3_sources.lua)
test('v3_documents_validate', function()
  local n = 0
  for _, f in ipairs(sources.all(b.root)) do
    -- includes the cross-document registry checks (v3-shared-registry.lua)
    local e = v3.validate_file(f)
    assert(#e == 0, f .. ':\n' .. table.concat(e, '\n'))
    n = n + 1
  end
  assert(n >= 1)
end)

-- one document per hero and per item: packages/<pkg>/v3.lua and items/*.lua, nothing else
test('v3_sources_one_copy', function()
  local heroes, seen = sources.heroes(b.root), {}
  for _, h in ipairs(heroes) do
    assert(not seen[h.source_id], 'two documents for ' .. h.source_id .. ': ' .. tostring(seen[h.source_id]) .. ', ' .. h.path)
    seen[h.source_id] = h.path
    assert(not path.is_file(path.join(b.root, sources.PACKAGES, h.package, 'kit.lua')), h.package .. ' still has kit.lua')
  end
  assert(#heroes >= 97, 'every history_nobu hero has a document')
  local items = {}
  for _, f in ipairs(sources.items(b.root)) do
    for _, it in ipairs(assert(v3.load_file(f)).items or {}) do
      assert(not items[it.source_id], it.source_id .. ' in ' .. tostring(items[it.source_id]) .. ' and ' .. f)
      items[it.source_id] = f
    end
  end
  local census = path.join(b.root, 'docs', 'plans', '2026-10-28-history-nobu-full-port', 'census-v3')
  for _, sub in ipairs({ 'heroes', 'items' }) do
    local dir = path.join(census, sub)
    if path.is_directory(dir) then
      for f in lfs.dir(dir) do assert(not f:match('%.lua$'), 'census-v3/' .. sub .. '/' .. f .. ': documents live at their authoring locations') end
    end
  end
end)

test('validate_package_dispatches_v3_files', function()
  local e = validator.validate(assert(sources.hero_path(b.root, 'C01')))
  assert(#e == 0, table.concat(e, '\n'))
  local e2 = validator.validate(path.join(b.root, sources.PACKAGES, 'saika_magoichi'))
  assert(#e2 == 0, table.concat(e2, '\n'))
end)

test('base_document_is_clean', function() clean(doc()) end)

-- structure and types
test('unknown_field_rejected', function()
  local d = doc(); ability(d, 'c01_blood_blade').colddown = '$cooldown'
  expect(d, '不認得的欄位 `colddown`')
end)
test('schema_version_required', function()
  local d = doc(); d.schema_version = 2
  expect(d, 'schema_version: 必須是 3')
end)
test('duplicate_id_rejected', function()
  local d = doc(); ability(d, 'c01_ground_flame').statuses[1].id = 'c01w_frenzy'
  expect(d, '`c01w_frenzy` 重複')
end)
test('duplicate_slot_rejected', function()
  local d = doc(); ability(d, 'c01_ground_flame').slot = 'Q'
  expect(d, 'Q 已被 c01_blood_blade 使用')
end)
test('unknown_action_rejected', function()
  local d = doc(); table.insert(ability(d, 'c01_blood_blade').events.on_cast, { op = 'explode' })
  expect(d, '不認得的 action `explode`')
end)
test('unknown_action_param_rejected', function()
  local d = doc(); ability(d, 'c01_blood_blade').events.on_cast[1].radius = '$radius'
  expect(d, '`apply_status` 沒有參數 `radius`')
end)
test('required_action_param', function()
  local d = doc(); ability(d, 'c01_blood_blade').events.on_cast[1].to = nil
  expect(d, '.to: 必填')
end)
test('unknown_event_rejected', function()
  local d = doc(); ability(d, 'c01_blood_blade').events.on_spell = {}
  expect(d, '不認得的事件 `on_spell`')
end)
test('unknown_status_reference', function()
  local d = doc(); ability(d, 'c01_blood_blade').events.on_cast[1].status = 'c01w_nope'
  expect(d, '狀態 `c01w_nope` 沒有宣告')
end)
test('unknown_stat_key', function()
  local d = doc(); ability(d, 'c01_blood_blade').statuses[1].stats.speed = '$move_speed_pct'
  expect(d, '不認得的 StatKey `speed`')
end)
test('reveal_policy_required', function()
  local d = doc(); ability(d, 'c01_blood_blade').statuses[1].reveal = nil
  expect(d, 'statuses[1](c01w_frenzy).reveal: 必填')
end)

-- expressions
test('expression_syntax_error', function()
  local d = doc(); ability(d, 'c01_blood_blade').cost.mana = '$mana_cost +'
  expect(d, '運算式錯誤')
end)
test('unknown_name_in_expression', function()
  local d = doc(); ability(d, 'c01_blood_blade').cost.mana = 'mana_cost'
  expect(d, '名稱 `mana_cost` 在這裡不存在')
end)
test('unknown_value_reference', function()
  local d = doc(); ability(d, 'c01_blood_blade').cost.mana = '$mana'
  expect(d, '數值 `$mana` 不存在')
end)
test('unknown_unit_property', function()
  local d = doc(); ability(d, 'c01_blood_blade').statuses[1].triggers.on_attack_hit[1].actions[1].amount = '$damage * it.mr'
  expect(d, 'unit 沒有屬性 `mr`')
end)
test('function_arity', function()
  local d = doc(); ability(d, 'c01_blood_blade').events.on_cast[2].cond = 'stacks(caster) > 0'
  expect(d, '`stacks` 需要 2 個參數')
end)
test('function_status_argument_resolved', function()
  local d = doc(); ability(d, 'c01_blood_blade').events.on_cast[2].cond = 'stacks(caster, c01r_nope) > 0'
  expect(d, '狀態 `c01r_nope` 沒有宣告')
end)
test('condition_type_checked', function()
  local d = doc(); ability(d, 'c01_blood_blade').events.on_cast[2].cond = '$damage + 1'
  expect(d, '需要 bool')
end)
test('scope_binding_is_local', function()
  -- `it` exists only inside the for_each
  local d = doc(); table.insert(ability(d, 'c01_blood_blade').events.on_cast, { op = 'damage', to = 'it', type = 'pure', amount = '$damage' })
  expect(d, '名稱 `it` 在這裡不存在')
end)
test('event_parameters_resolved', function()
  local d = doc(); ability(d, 'c01_blood_flame').passive.triggers.on_attack_windup[1].actions[2].at = 'event.victim'
  expect(d, '這個事件沒有參數 `event.victim`')
end)
test('rules_path_resolved', function()
  local d = doc(); ability(d, 'c01_blood_blade').values.radius = { expr = 'rules.hero.no_such', unit = 'dist' }
  expect(d, '規則層沒有 `rules.hero.no_such`')
  local d2 = doc(); ability(d2, 'c01_blood_blade').values.radius = { expr = 'rules.hero.acquisition_range', unit = 'dist' }
  clean(d2)
end)
test('derived_value_cycle', function()
  local d = doc(); local v = ability(d, 'c01_honnoji').values
  v.kv_radius = { expr = '$area_radius', unit = 'dist', hidden = true }
  expect(d, '衍生值循環引用')
end)
test('capture_must_exist', function()
  local d = doc(); ability(d, 'c01_ground_flame').events.on_cast[2].capture = { 'nowhere' }
  expect(d, '`nowhere` 不是目前可用的名稱')
end)
test('routine_args_checked', function()
  local d = doc(); ability(d, 'c01_ground_flame').events.on_cast[2].actions[1].args.stun = nil
  expect(d, 'routine `c01e_wave` 需要參數 stun')
end)

-- unnamed numbers
test('lua_number_in_behavior_rejected', function()
  local d = doc(); ability(d, 'c01_blood_blade').cost.cooldown = 16
  expect(d, '未命名的數字 16')
end)
test('number_in_expression_rejected', function()
  local d = doc(); ability(d, 'c01_blood_blade').statuses[1].triggers.on_attack_hit[1].actions[1].amount = '$damage * 0.7'
  expect(d, '未命名的數字 `0.7`')
end)
test('structural_numbers_allowed', function()
  local d = doc(); ability(d, 'c01_blood_blade').statuses[1].triggers.on_attack_hit[1].actions[1].amount = '$damage * (1 - it.base_magic_resist) + 0'
  clean(d)
end)

-- values
test('value_needs_source', function()
  local d = doc(); ability(d, 'c01_blood_blade').values.cooldown.source = nil
  expect(d, 'values.cooldown.source: 必須寫出處')
end)
test('value_source_prefix', function()
  local d = doc(); ability(d, 'c01_blood_blade').values.cooldown.source = 'AbilityCooldown'
  expect(d, '前綴必須是')
end)
test('value_rank_count', function()
  local d = doc(); ability(d, 'c01_blood_blade').values.cooldown = { 16, 15, unit = 's', source = 'kv:AbilityCooldown' }
  expect(d, '數值個數必須是 1 或 4')
end)
test('value_unit_required', function()
  local d = doc(); ability(d, 'c01_blood_blade').values.cooldown.unit = nil
  expect(d, 'values.cooldown.unit: 必填')
end)
test('count_values_are_integers', function()
  local d = doc(); ability(d, 'c01_ground_flame').values.wave_count[1] = 2.5
  expect(d, 'count／integer 必須是整數')
end)
test('kv_conflict_needs_reason', function()
  local d = doc(); ability(d, 'c01_ground_flame').values.stun.source_differs = nil
  expect(d, '寫了 kv')
end)

-- single source (value-single-source §5)
test('text_has_no_digits', function()
  local d = doc(); local t = ability(d, 'c01_blood_blade').text.zh_TW
  t.body = t.body .. '最多 3 次。'
  expect(d, '說明文字不能有數字')
end)
test('text_has_no_fullwidth_digits', function()
  local d = doc(); local t = ability(d, 'c01_blood_blade').text.zh_TW
  t.body = t.body .. '最多３次。'
  expect(d, '說明文字不能有數字')
end)
test('placeholder_must_exist', function()
  local d = doc(); local t = ability(d, 'c01_blood_blade').text
  t.zh_TW.body = t.zh_TW.body .. '{bonus}'; t.en.body = t.en.body .. '{bonus}'
  expect(d, '佔位符 {bonus} 沒有對應的 value')
end)
test('placeholder_sets_match_between_languages', function()
  local d = doc(); local t = ability(d, 'c01_blood_blade').text.en
  t.body = t.body:gsub('{radius}', 'a radius')
  expect(d, 'text.en: 缺少 zh_TW 有的佔位符 {radius}')
end)
test('placeholder_format_checked', function()
  local d = doc(); local t = ability(d, 'c01_blood_blade').text
  t.zh_TW.body = t.zh_TW.body:gsub('{radius}', '{radius:wide}')
  expect(d, '不認得的格式 `:wide`')
end)
test('name_reference_checked', function()
  local d = doc(); local t = ability(d, 'c01_blood_blade').text
  t.zh_TW.body = t.zh_TW.body .. '{@c01_nothing}'; t.en.body = t.en.body .. '{@c01_nothing}'
  expect(d, '{@c01_nothing} 指向不存在的 id')
end)
test('both_languages_required', function()
  local d = doc(); ability(d, 'c01_blood_blade').text.en = nil
  expect(d, 'text.en: 必填')
end)
test('unused_value_rejected', function()
  local d = doc(); ability(d, 'c01_blood_blade').values.spare = { 5, unit = 's', source = 'kv:Spare' }
  expect(d, 'values.spare: 沒有人用')
end)
test('visible_value_must_reach_player', function()
  local d = doc(); local a = ability(d, 'c01_ground_flame')
  a.values.reveal_duration.hidden = nil
  expect(d, 'values.reveal_duration: 玩家看不到')
end)
test('hidden_value_not_in_text', function()
  local d = doc(); local a = ability(d, 'c01_blood_blade')
  a.values.radius.hidden = true
  expect(d, '{radius} 標了 hidden 卻出現在說明裡')
end)
test('info_line_counts_as_visible', function()
  -- cooldown and mana_cost are never in the body: the info line shows them.
  local d = doc(); local t = ability(d, 'c01_blood_blade').text.zh_TW.body
  assert(not t:find('{cooldown}', 1, true))
  clean(d)
end)

-- hooks and coverage
local function with_hook(d, def)
  d.hooks = { c01_probe = def or { why = '這個測試需要一個無法用資料表達的效果', test = 'c01_probe_conformance' } }
  table.insert(ability(d, 'c01_blood_blade').events.on_cast, { op = 'hook', name = 'c01_probe' })
  return d
end
test('hook_use_needs_needs_hook_coverage', function()
  expect(with_hook(doc()), 'coverage: 用了 hook，coverage 必須是 needs_hook')
end)
test('hook_with_needs_hook_is_clean', function()
  local d = with_hook(doc()); ability(d, 'c01_blood_blade').coverage = 'needs_hook'
  clean(d)
end)
test('hook_needs_reason', function()
  local d = with_hook(doc(), { why = '太難', test = 'x' }); ability(d, 'c01_blood_blade').coverage = 'needs_hook'
  expect(d, '每個 hook 都要寫明為什麼資料表達不了')
end)
test('hook_needs_test', function()
  local d = with_hook(doc(), { why = '這個測試需要一個無法用資料表達的效果' }); ability(d, 'c01_blood_blade').coverage = 'needs_hook'
  expect(d, 'hooks.c01_probe.test: 必須是非空字串')
end)
test('unused_hook_rejected', function()
  local d = doc(); d.hooks = { c01_idle = { why = '宣告了卻沒有被任何技能使用的 hook', test = 't' } }
  expect(d, '宣告了但沒有任何 hook action 使用')
end)
test('needs_hook_without_hook_rejected', function()
  local d = doc(); ability(d, 'c01_blood_blade').coverage = 'needs_hook'
  expect(d, 'needs_hook 但行為裡沒有 hook action')
end)
test('pending_needs_items', function()
  local d = doc(); ability(d, 'c01_blood_blade').coverage = 'pending'
  expect(d, 'coverage = pending 必須列出待確認項目')
  ability(d, 'c01_blood_blade').pending = { { what = '移速倍增', why = '原作說明與腳本衝突，等使用者決定' } }
  clean(d)
end)
test('quirk_policy_required', function()
  local d = doc(); ability(d, 'c01_blood_blade').quirks[1].policy = nil
  expect(d, 'quirks[1].policy: 必填')
end)
test('provenance_required', function()
  local d = doc(); ability(d, 'c01_blood_blade').provenance = {}
  expect(d, '必須列出原作出處')
end)

-- checks added while freezing v3.0 (v3.md §12)
local function push(d, action) table.insert(ability(d, 'c01_blood_blade').events.on_cast, action); return d end
test('illusion_damage_keys_checked', function()
  expect(push(doc(), { op = 'create_illusion', of = 'caster', duration = '$duration', outgoing = { attack_pct = '$damage' } }),
    '不認得的鍵 `attack_pct`')
  clean(push(doc(), { op = 'create_illusion', of = 'caster', duration = '$duration', outgoing = { physical = '-1' },
    copy = { level = true, items = 'fresh', except_abilities = { 'c01_blood_blade' } } }))
end)
test('illusion_copy_fields_checked', function()
  expect(push(doc(), { op = 'create_illusion', of = 'caster', duration = '$duration', copy = { items = 'all' } }),
    '`all` 不是允許的值')
end)
test('unit_overrides_keys_checked', function()
  local d = doc()
  d.unit_templates = { { id = 'c01_dummy', text = { zh_TW = { name = '假人' }, en = { name = 'Dummy' } } } }
  push(d, { op = 'create_unit', template = 'c01_dummy', at = 'caster', overrides = { damage = '$damage' } })
  expect(d, '不認得的鍵 `damage`')
end)
test('remove_status_by_tags_alone', function()
  clean(push(doc(), { op = 'remove_status', from = 'caster', tags = { 'slow' } }))
  expect(push(doc(), { op = 'remove_status', from = 'caster' }), '要指定 status、dispel、tags 或 states')
end)
test('state_init_is_an_expression_and_counts_as_use', function()
  local d = doc()
  ability(d, 'c01_blood_blade').values.spare = { 5, unit = 'count', hidden = true, source = 'design:test' }
  ability(d, 'c01_blood_blade').unit_state = { left = { type = 'int', init = '$spare', reveal = 'team' } }
  clean(d)
  ability(d, 'c01_blood_blade').unit_state.left.init = '$nope'
  expect(d, '數值 `$nope` 不存在')
end)
test('name_reference_needs_status_text', function()
  local d = doc(); local t = ability(d, 'c01_blood_blade').text
  -- a hidden status may go without text; naming it in a template still needs one
  for _, a in ipairs(d.abilities) do
    for _, s in ipairs(a.statuses or {}) do if s.id == 'c01e_stun' then s.visibility = 'hidden'; s.text = nil end end
  end
  t.zh_TW.body = t.zh_TW.body .. '{@c01e_stun}'; t.en.body = t.en.body .. '{@c01e_stun}'
  expect(d, '{@c01e_stun} 指向的狀態沒有 text')
end)
test('function_name_arguments_checked', function()
  local d = doc(); ability(d, 'c01_blood_blade').events.on_cast[2].cond = 'has_state(caster, sleepy)'
  expect(d, '不認得的狀態旗標 `sleepy`')
  d = doc(); ability(d, 'c01_blood_blade').events.on_cast[2].cond = 'has_item(caster, brave_ring)'
  expect(d, '不是道具 id')
  d = doc(); ability(d, 'c01_blood_blade').events.on_cast[2].cond = 'is_item(event_ability, item_tpscroll)'
  expect(d, '名稱 `event_ability` 在這裡不存在')
end)
test('chance_counter_var_must_be_declared', function()
  local d = doc()
  ability(d, 'c01_blood_flame').passive.triggers.on_attack_windup[1].counter = nil
  ability(d, 'c01_blood_flame').passive.triggers.on_attack_windup[1].counter_var = 'c01r_misses'
  expect(d, '保底計數要用已宣告的 unit_state')
  d.unit_state = { c01r_misses = { type = 'int', init = '0', reveal = 'team', scope = 'hero' } }
  clean(d)
end)
test('projectile_where_binds_it', function()
  clean(push(doc(), { op = 'projectile', kind = 'linear', from = 'caster', direction = 'caster.forward', speed = '$radius',
    distance = '$radius', where = 'it.is_hero', on_hit = { { op = 'damage', to = 'it', type = 'pure', amount = '$damage' } } }))
end)

-- compiler (M1): catalog blocks, raw Fixed64 from decimal text, rendered texts
local compiler = dofile(path.join(tools, 'v3_compile.lua'))
test('compiler_raw_values_come_from_decimal_text', function()
  assert(compiler.raw_of('0.53') == 543 and compiler.raw_of('-0.53') == -543 and compiler.raw_of('4.2') == 4301)
  assert(compiler.raw_of('0.15', 'truncate') == 153 and compiler.raw_of('0.15') == 154)
  assert(compiler.raw_of('1900/101') == 19263 and compiler.raw_of('12') == 12288)
  -- more digits than an int64 holds (A13 rad_to_deg): exact digit arithmetic, like v3.rs i128
  assert(compiler.raw_of('57.295779513082320876') == 58671 and compiler.raw_of('-57.295779513082320876') == -58671)
  assert(compiler.raw_of('57.295779513082320876', 'truncate') == 58670)
end)
test('compiler_renders_templates_and_extras', function()
  local out = compiler.compile(doc())
  local flame = out.description.c01_blood_flame
  assert(flame:find('18.81% 機率', 1, true) and flame:find('180/220/260/300%', 1, true), flame)
  assert(not out.description.c01_ground_flame:find('{', 1, true))
  local block = out.abilities.c01_honnoji
  local area
  for _, v in ipairs(block.values) do if v.name == 'area_radius' then area = v end end
  assert(area.derived and area.ranks[3] == '700' and area.raw[3] == 716800)
  assert(out.extras.c01_honnoji.damage[2] == 94)
  assert(out.levels.c01_honnoji[3].cooldown == 113 and out.levels.c01_honnoji[2].required_hero_level == 12)
  local pieces = block.text.zh_TW.pieces
  for _, p in ipairs(pieces) do assert(not (p.text and p.text:find('%d') and not p.name_of and not p.rules)) end
  for _, k in ipairs({ 'values', 'info' }) do assert(block[k] == nil or #block[k] > 0) end
end)
test('handler_reads_count_as_uses', function()
  local d = doc(); local a = ability(d, 'c01_blood_blade')
  a.values.spare = { 5, unit = 's', hidden = true, source = 'design:test' }
  expect(d, 'values.spare: 沒有人用')
  a.coverage = 'rust_handler'; a.handler = { module = 'No1_blood_blade', reads = { '$spare' } }
  clean(d)
  a.handler.reads = { '$nope' }
  expect(d, '`$nope` 不是存在的 $value')
end)

-- provenance comparison
V3_PROVENANCE_AS_MODULE = true
local provenance = dofile(path.join(tools, 'v3_provenance.lua'))
V3_PROVENANCE_AS_MODULE = nil
test('provenance_compares_ranks_kv_and_percent_fractions', function()
  assert(provenance.compare({ 1, 2, unit = 'count' }, { 1, 2 }, 2) == 'ok')
  assert(provenance.compare({ 5, unit = 's' }, { 5 }, 4) == 'ok')
  assert(provenance.compare({ 1, 3, unit = 'count' }, { 1, 2 }, 2) == 'mismatch')
  assert(provenance.compare({ 15, unit = 'pct' }, { 0.15 }, 1) == 'ok')
  assert(provenance.compare({ 9, kv = { 7 }, source_differs = 'x', unit = 's' }, { 7 }, 1) == 'ok')
  assert(provenance.compare({ 9, source_differs = 'x', unit = 's' }, { 7 }, 1) == 'skipped')
end)

-- cross-document registry (v3.2)
test('registry_checks_tags_states_and_cooldown_groups', function()
  local d = doc(); ability(d, 'c01_blood_blade').events.on_cast[2].cond = 'has_status_tag(caster, nobody_declares_this)'
  expect(d, '狀態 tag `nobody_declares_this` 沒有在這份文件宣告')
  d = doc(); ability(d, 'c01_blood_blade').events.on_cast[2].cond = 'has_status_tag(caster, slow)'
  clean(d)
  d = doc(); ability(d, 'c01_blood_blade').unit_state = { shield_left = { type = 'fixed', init = '0', reveal = 'team', scope = 'unit' } }
  expect(d, '同名就跨文件共用')
  d = doc(); ability(d, 'c01_blood_blade').unit_state = { c01w_shield_left = { type = 'fixed', init = '0', reveal = 'team', scope = 'unit' } }
  clean(d)
  d = doc(); ability(d, 'c01_blood_blade').unit_state = { aphotic_shield_remaining = { type = 'int', init = '0', reveal = 'team', scope = 'unit' } }
  expect(d, '登錄表的 `aphotic_shield_remaining` 是 fixed')
  d = doc(); ability(d, 'c01_blood_blade').cost.cooldown_groups = { { group = 'flame_series' } }
  expect(d, '共用冷卻組 `flame_series` 要登錄')
  d = doc(); ability(d, 'c01_blood_blade').cost.cooldown_groups = { { group = 'fire_family' } }
  clean(d)
end)
test('chain_binds_it_and_take_while', function()
  clean(push(doc(), { op = 'chain', from = 'caster', first = 'caster', jumps = '$radius', radius = '$radius', team = 'enemy',
    take_while = 'not it.is_invisible', actions = { { op = 'damage', to = 'it', type = 'pure', amount = '$damage' } } }))
end)

-- behavior IR (M3, v3_ir.lua)
local IR = dofile(path.join(tools, 'v3_ir.lua'))
-- reference evaluator for postfix expressions (numbers as raw Fixed64, mock lookups)
local function eval_ir(ir, index, env)
  local code, stack, pc = ir.exprs[index], {}, 1
  while pc <= #code do
    local t = code[pc]
    local k = t[1]
    if k == 'raw' then stack[#stack + 1] = t[2]
    elseif k == 'bool' then stack[#stack + 1] = t[2]
    elseif k == 'slot' then stack[#stack + 1] = env.slot(ir.slots[t[2]])
    elseif k == 'local' then stack[#stack + 1] = env.locals[t[2]]
    elseif k == 'name' then stack[#stack + 1] = env.names[t[2]]
    elseif k == 'attr' then local o = table.remove(stack); stack[#stack + 1] = o[t[2]]
    elseif k == 'sym' then stack[#stack + 1] = { sym = t[2] }
    elseif k == 'call' then
      local args = {}
      for i = t[3], 1, -1 do args[i] = table.remove(stack) end
      stack[#stack + 1] = env.call(t[2], args)
    elseif k == 'jump_if_false' then if not table.remove(stack) then pc = pc + t[2] end
    elseif k == 'jump' then pc = pc + t[2]
    elseif k == 'op' then
      local op = t[2]
      if op == 'neg' then stack[#stack] = -stack[#stack]
      elseif op == 'not' then stack[#stack] = not stack[#stack]
      else
        local r, l = table.remove(stack), table.remove(stack)
        local v
        if op == '+' then v = l + r elseif op == '-' then v = l - r elseif op == '*' then v = l * r // 1024
        elseif op == '/' then v = l * 1024 // r elseif op == '>' then v = l > r elseif op == '<' then v = l < r
        elseif op == '==' then v = l == r elseif op == '>=' then v = l >= r elseif op == '<=' then v = l <= r end
        stack[#stack + 1] = v
      end
    end
    pc = pc + 1
  end
  assert(#stack == 1, 'expression leaves ' .. #stack .. ' values')
  return stack[1]
end
local function find_instr(ir, op)
  for pi, prog in ipairs(ir.programs or {}) do
    for _, i in ipairs(prog) do if i.op == op then return i, pi end end
  end
end
test('ir_lowers_c01_and_values_become_slots', function()
  local out, problems = IR.compile_doc(doc())
  assert(#problems == 0, table.concat(problems, '\n'))
  local ir = out.c01_blood_blade
  assert(ir.version == 1 and ir.entries.on_cast and ir.statuses[1].id == 'c01w_frenzy')
  -- on_attack_hit: damage amount = $damage * (1 - it.base_magic_resist)
  local dmg = find_instr(ir, 'damage')
  local hit_local
  for _, prog in ipairs(ir.programs) do for _, i in ipairs(prog) do if i.op == 'for_each' then hit_local = i.actions_bind.it end end end
  local value = eval_ir(ir, dmg.amount, {
    slot = function(s) assert(s.name == 'damage'); return 120 * 1024 end,
    locals = { [hit_local] = { base_magic_resist = 300 } }, -- 0.29 as raw
    names = {}, call = function() end })
  assert(value == 120 * 1024 * (1024 - 300) // 1024, tostring(value))
  -- pct slots are marked for the loader
  local scaled = false
  for _, s in ipairs(ir.slots) do if s.name == 'move_speed_pct' then scaled = s.scale == 'pct' end end
  assert(scaled)
  -- the routine with its parameters is compiled into the block that calls it
  local flame = out.c01_ground_flame
  assert(flame.routines.c01e_wave and #flame.routines.c01e_wave.params == 3)
end)
test('ir_short_circuits_and_or_select', function()
  local d = doc()
  ability(d, 'c01_blood_blade').events.on_cast[2].cond = 'caster.is_hero and (caster.alive or caster.is_building)'
  ability(d, 'c01_blood_blade').values.spare = { 2, unit = 'number', hidden = true, source = 'design:test' }
  ability(d, 'c01_blood_blade').events.on_cast[1].duration = 'select(caster.is_hero, $spare, $duration)'
  local out = IR.compile_doc(d)
  local ir = out.c01_blood_blade
  local cond = find_instr(ir, 'if').cond
  local apply = find_instr(ir, 'apply_status')
  local function run(index, hero, alive, building)
    return eval_ir(ir, index, { slot = function(s) return s.name == 'spare' and 2048 or 12288 end, locals = {},
      names = { caster = { is_hero = hero, alive = alive, is_building = building } }, call = function() end })
  end
  assert(run(cond, true, true, false) == true and run(cond, true, false, true) == true)
  assert(run(cond, true, false, false) == false and run(cond, false, true, true) == false)
  assert(run(apply.duration, true) == 2048 and run(apply.duration, false) == 12288)
end)
test('ir_tables_are_pure_arrays_or_maps', function()
  local function check(t, where)
    if type(t) ~= 'table' then return end
    local n, keys = 0, 0
    for k in pairs(t) do keys = keys + 1; if math.type(k) == 'integer' then n = n + 1 end end
    assert(n == 0 or n == keys, 'mixed table at ' .. where)
    assert(keys > 0, 'empty table at ' .. where .. ' (no JSON shape)')
    for k, v in pairs(t) do check(v, where .. '.' .. tostring(k)) end
  end
  for _, f in ipairs(sources.all(b.root)) do
    local out, problems = IR.compile_doc(assert(v3.load_file(f)))
    assert(#problems == 0, f .. ': ' .. table.concat(problems, '\n'))
    for id, ir in pairs(out) do check(ir, f .. ':' .. id) end
  end
end)

-- test scenarios (v3_scenarios.lua): every case refers to values by name, never by number
V3_SCENARIOS_AS_MODULE = true
local scenarios = dofile(path.join(tools, 'v3_scenarios.lua'))
V3_SCENARIOS_AS_MODULE = nil
test('scenarios_cover_c17_and_b01', function()
  for _, id in ipairs({ 'C17', 'B01' }) do
    local d = assert(v3.load_file(assert(sources.hero_path(b.root, id))))
    local r = scenarios.build(d)
    local p = scenarios.check(d, r)
    assert(#p == 0, id .. ': ' .. table.concat(p, '\n'))
    local by = {}
    for _, s in ipairs(r.scenarios) do by[s.id] = s end
    for _, a in ipairs(d.abilities) do
      for rank = 1, a.max_level do
        if a.kind ~= 'passive' and a.cost then
          local prefix = a.id .. '/r' .. rank .. '/'
          assert(by[prefix .. 'normal'] or by[prefix .. 'normal/enemy'] or by[prefix .. 'toggle_on'], 'normal case for ' .. prefix)
          if a.cost.mana then assert(by[prefix .. 'insufficient_mana'], 'mana case for ' .. prefix) end
          -- with stored charges the next cast spends a charge: no_charges covers it (E207)
          if a.cost.cooldown and not a.cost.charges then assert(by[prefix .. 'cooldown'], 'cooldown case for ' .. prefix) end
        end
      end
      for i in ipairs(a.quirks or {}) do assert(by[a.id .. '/r1/quirk/' .. i], a.id .. ' quirk ' .. i) end
    end
  end
  local d = assert(v3.load_file(sources.hero_path(b.root, 'C17')))
  local by = {}
  for _, s in ipairs(scenarios.build(d).scenarios) do by[s.id] = s end
  local n = by['c17_light_chain/r2/normal/enemy']
  assert(n and table.concat(n.refs, ','):find('damage', 1, true), 'expectations name the values')
  assert(by['c17_light_chain/r1/dispel/c17w_root/basic'].expect[1].kind == 'status_removed', 'basic dispel removes the root')
  assert(by['c17_light_chain/r1/trigger/c17w_root/on_created'], 'status triggers get a case')
end)
test('scenarios_for_every_document', function()
  for _, f in ipairs(sources.all(b.root)) do
    local d = assert(v3.load_file(f))
    local p = scenarios.check(d, scenarios.build(d))
    assert(#p == 0, f .. ': ' .. table.concat(p, '\n'))
  end
end)

-- usage statistics and executor build order (v3_usage.lua)
V3_USAGE_AS_MODULE = true
local usage = dofile(path.join(tools, 'v3_usage.lua'))
V3_USAGE_AS_MODULE = nil
test('usage_order_unlocks_every_unblocked_hero', function()
  local r = usage.build(b.root)
  assert(r.components['action:damage'] and r.components['action:damage'].heroes > 0, 'damage is counted')
  local first = {}
  for _, o in ipairs(r.order) do for _, id in ipairs(o.new_heroes) do first[id] = first[id] or o end end
  assert(first.B01 and first.C17, 'the M3 first batch unlocks B01 and C17')
  local blocked = 0
  for _, u in ipairs(r.blocked) do if u.kind == 'hero' then blocked = blocked + 1 end end
  assert(r.order[#r.order].heroes + blocked == r.heroes, 'every hero without pending entries is unlocked at the end')
  for i = 2, #r.order do assert(r.order[i].heroes >= r.order[i - 1].heroes, 'cumulative counts never drop') end
  assert(usage.render(r):find('| 1 | `', 1, true), 'renders the build order table')
end)

-- presentation cue catalog (v3_cues.lua): original KV and script parsing
V3_CUES_AS_MODULE = true
local cues = dofile(path.join(tools, 'v3_cues.lua'))
V3_CUES_AS_MODULE = nil
test('cue_catalog_reads_original_effects', function()
  local kv = cues.kv_assets(table.concat({
    '"X01W" {', '  "precache" { "particle" "particles/x01w/x01w.vpcf" }',
    '  "OnSpellStart" { "FireSound" { "EffectName" "X01W.vo1" } }',
    '  "Modifiers" { "modifier_X01W" { "EffectName" "particles/x01w2/x01w2.vpcf" } // comment "a.vpcf"', '  }', '}' }, '\n'))
  assert(#kv == 3, 'three KV assets, got ' .. #kv)
  assert(kv[1].precache and kv[1].kind == 'particle', 'precache particle')
  assert(kv[2].kind == 'sound' and kv[2].path == 'X01W.vo1', 'FireSound effect is a sound')
  assert(kv[3].modifier == 'modifier_X01W' and kv[3].line == 4, 'modifier effect with its line')
  local lua = cues.lua_assets(table.concat({ 'function X01W(keys)',
    '  local p = ParticleManager:CreateParticle("particles/x01w3/x01w3.vpcf", PATTACH_POINT, keys.caster)',
    '  EmitSoundOn("Hero_X.Cast", keys.caster)', '  -- local q = ParticleManager:CreateParticle("particles/old.vpcf", 0, nil)', 'end' }, '\n'))
  assert(#lua == 3 and lua[1].context == 'X01W' and lua[2].kind == 'sound' and lua[3].commented, 'script particles and sounds')
  -- cue kinds for UE's generic placeholders
  assert(cues.use_kind({ status = {} }, 'x', 'particle') == 'aura', 'a status cue is an aura')
  assert(cues.use_kind({ field_op = 'projectile' }, 'x', 'particle') == 'projectile', 'a projectile cue')
  assert(cues.use_kind({ field_op = 'zone' }, 'x', 'particle') == 'area', 'a zone cue')
  assert(cues.use_kind({ list = 'on_hit' }, 'x', 'particle') == 'impact', 'an on_hit cue')
  assert(cues.use_kind({ event = 'on_channel_tick' }, 'x', 'particle') == 'channel', 'a channel cue')
  assert(cues.use_kind({}, 'a05d_die_gesture', 'particle') == 'gesture', 'a gesture cue')
  assert(cues.use_kind({}, 'x_vo', 'sound') == 'sound' and cues.use_kind({}, 'x', 'particle') == 'effect', 'sound or effect')
end)

-- the IR carries the cast validation block (target and ability flags)
test('ir_target_block', function()
  local d = assert(v3.load_file(assert(sources.hero_path(b.root, 'C17'))))
  local out, problems = IR.compile_doc(d)
  assert(#problems == 0, table.concat(problems, '\n'))
  local t = assert(out.c17_light_chain.target, 'c17_light_chain has a target block')
  assert(t.type == 'unit' and t.team == 'both' and t.types[1] == 'hero', 'type, team and types come from v3')
  assert(t.ability_flags[1] == 'no_target_self', 'ability flags come along')
  assert(type(t.range) == 'number' and out.c17_light_chain.exprs[t.range], 'range is an expression index')
end)

-- unit_state declarations in the IR (m3-ir-design.md §7.2): on by default, every variable the block uses
test('ir_unit_state', function()
  local d = assert(v3.load_file(assert(sources.hero_path(b.root, 'B30'))))
  local off = IR.compile_doc(d, { unit_state = false })
  assert(off.b30_purgatory_slash.unit_state == nil, 'opts.unit_state = false leaves it out')
  local out, problems = IR.compile_doc(d)
  assert(#problems == 0, table.concat(problems, '\n'))
  local ir = out.b30_purgatory_slash
  local row
  for _, r in ipairs(ir.unit_state or {}) do if r.name == 'tsum' then row = r end end
  assert(row, 'b30_purgatory_slash declares tsum')
  assert(row.type == 'fixed' and row.scope == 'match' and row.reveal == 'public', 'type, scope and reveal come from the declaration')
  assert(type(row.init) == 'number' and ir.exprs[row.init], 'init is an expression index')
end)

-- v3.3 host: values a host component reads count as uses; only on data entries; known components
test('host_reads', function()
  local d = doc()
  local a = d.abilities[1]
  a.values.host_cap = { 3, unit = "count", hidden = true, source = "design:host cap" }
  expect(d, '沒有人用')
  a.host = { component = 'ability_projectile', reads = { '$host_cap' } }
  clean(d)
  a.host = { { component = 'ability_projectile', reads = { '$host_cap' } } }
  clean(d)
  a.host = { component = 'no_such_component', reads = { '$host_cap' } }
  expect(d, 'host.component')
  a.host = { component = 'attack_orb', reads = { '$missing' } }
  expect(d, '不是存在的 $value')
  local r = doc()
  for _, x in ipairs(r.abilities) do
    if x.coverage ~= 'rust_handler' then
      x.host = { component = 'attack_orb', reads = {} }
      expect(r, 'host 要列出')
      break
    end
  end
end)

-- v3.3 status runtime_id: validated, carried into the catalog row and the IR
test('status_runtime_id', function()
  local function first_status(d)
    for _, a in ipairs(d.abilities) do if a.statuses and a.statuses[1] then return a, a.statuses[1] end end
  end
  local d = doc()
  local a, s = first_status(d)
  s.runtime_id = 'generic_slow:' .. a.id .. ':{caster}'
  clean(d)
  local c = compiler.compile(d, { ir = false })
  local row
  for _, r in ipairs(c.abilities[a.id].statuses or {}) do if r.id == s.id then row = r end end
  assert(row and row.runtime_id == s.runtime_id, 'catalog row carries runtime_id')
  local out = IR.compile_doc(d)
  for _, st in ipairs(out[a.id] and out[a.id].statuses or {}) do
    if st.id == s.id then assert(st.runtime_id == s.runtime_id, 'IR status carries runtime_id') end
  end
  local bad = doc()
  local _, s2 = first_status(bad)
  s2.runtime_id = 'bad id {target}'
  expect(bad, 'runtime_id')
end)

-- v3.3 (E204): a visible status needs a name and a tooltip body in both languages
test('visible_status_needs_text', function()
  local d = doc()
  local target
  for _, a in ipairs(d.abilities) do
    for _, s in ipairs(a.statuses or {}) do if not target and (s.visibility == 'buff' or s.visibility == 'debuff') then target = s end end
  end
  assert(target, 'the base document has a visible status')
  target.text.en.body = nil
  expect(d, '必須寫 name 與 body')
  local h = doc()
  for _, a in ipairs(h.abilities) do
    for _, s in ipairs(a.statuses or {}) do if s.visibility == 'hidden' then s.text = nil end end
  end
  clean(h)
end)

-- v3.3 bot cast policy: derived from targeting, authored overrides validated, exported for the bot
test('bot_policy', function()
  local d = assert(v3.load_file(assert(sources.hero_path(b.root, 'C17'))))
  local c = compiler.compile(d, { ir = false })
  assert(c.abilities.c17_light_chain.bot.kind == 'enemy_hero', 'a hero-only unit target derives enemy_hero')
  assert(c.abilities.c17_divine_strike.bot.kind == 'enemy_point' and c.abilities.c17_divine_strike.bot.radius_key, 'a point area derives enemy_point with its radius value')
  assert(c.abilities.c17_victory_blessing.bot.source == 'authored', 'the authored policy wins')
  local x = doc()
  ability(x, 'c01_blood_blade').bot = { kind = 'enemy_point', radius_key = '$nope', min_targets = 1 }
  expect(x, '值 `nope` 不存在')
  local y = doc()
  ability(y, 'c01_blood_blade').bot = { kind = 'fly_away' }
  expect(y, 'bot.kind `fly_away`')
  local z = doc()
  ability(z, 'c01_blood_blade').bot = false
  clean(z)
end)

-- vocabulary rendering
test('vocab_markdown_renders', function()
  local md = v3.vocab_md()
  assert(md:find('| `damage` |', 1, true) and md:find('StatKey', 1, true))
end)

-- English text follows presentation/en-glossary.md: every document is lint-clean, and the lint catches
-- a banned variant, second person in a hero document and placeholder drift
test('en_glossary_lint', function()
  local lint = dofile(path.join(tools, 'v3_en_lint.lua'))
  local files = {}
  for _, h in ipairs(sources.heroes(b.root)) do files[#files + 1] = { path = h.path, label = h.package, hero = true } end
  for _, it in ipairs(sources.items(b.root)) do
    local p = type(it) == 'table' and it.path or it
    files[#files + 1] = { path = p, label = p, hero = false }
  end
  local r = lint.run(files)
  assert(r.errors == 0, table.concat(r.lines, '\n'))
  local found = {}
  lint.lint_rows({
    { path = 'x', key = 'body', zh = '{a}', en = 'Deals {a} magical damage.' },
    { path = 'y', key = 'body', zh = '{a}', en = 'You gain {a} armor.' },
    { path = 'z', key = 'body', zh = '{a} {b}', en = 'Gains {a} armor.' },
    { path = 'd', key = 'body', zh = '沒有效果（原作腳本錯誤）。', en = 'Has no effect.' },
    { path = 'e', key = 'body', zh = '沒有效果。', en = 'Has no effect (the original script errors out).' },
    { path = 'm', key = 'name', zh = '濃姬', en = 'Nōhime' },
  }, true, function(level, where) if level == 'error' then found[where] = true end end)
  assert(found['x body'] and found['y body'] and found['z body'], 'banned variant, second person and a missing placeholder are errors')
  assert(found['d body'] and found['e body'] and found['m name'], 'developer wording in either language and long-vowel marks are errors')
end)

print(count .. ' tests passed')
