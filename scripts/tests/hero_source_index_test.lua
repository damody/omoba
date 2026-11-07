package.path = 'scripts/?.lua;' .. package.path
local b = require('_bootstrap')
local path = b.lib('path')
local hash = b.lib('hash')
local lfs = require('lfs')
local kv = require('hero_source.kv')
local sha256 = require('hero_source.sha256')
local classify = require('hero_source.classify')
local catalog_mod = require('hero_source.catalog')
local serialize = require('hero_source.serialize')

local parent = path.mkdir_p(path.join(b.root, 'target', 'hero-source-tests'))
local root
for i = 1, 1000 do local p = path.join(parent, os.time() .. '-' .. i); if lfs.mkdir(p) then root = p; break end end
assert(root)
local count = 0
local function test(name, fn) fn(); count = count + 1; print('PASS ' .. name) end
local function kinds(doc)
  local out = {}
  for _, d in ipairs(doc.diagnostics) do out[#out + 1] = d.kind end
  return table.concat(out, ',')
end

test('BOM is skipped but offsets stay on original bytes', function()
  local text = kv.BOM .. '"lang"\r\n{\r\n\t"Tokens" { "k" "v" }\r\n}\r\n'
  local doc = kv.parse(text)
  assert(doc.bom and #doc.diagnostics == 0, kinds(doc))
  local lang = doc.nodes[1]
  assert(lang.key == 'lang' and lang.start == 4 and text:sub(lang.start, lang.start + 5) == '"lang"')
  local k = kv.find(kv.find(lang.children, 'Tokens').children, 'k')
  assert(k.value == 'v' and text:sub(k.start, k.stop) == '"k" "v"')
  assert(text:sub(lang.start, lang.stop):sub(-1) == '}')
end)

test('line, block and trailing comments are kept, not parsed as tokens', function()
  local doc = kv.parse('// head\n"a"\n{\n\t// lead\n\t"x" "1" // tail\n\t/* "y" "2" */\n\t"z" "3"\n}\n')
  assert(#doc.diagnostics == 0, kinds(doc))
  local a = doc.nodes[1]
  assert(a.comments[1] == ' head')
  local x, z = a.children[1], a.children[2]
  assert(#a.children == 2 and x.key == 'x' and z.key == 'z')
  assert(x.comments[1] == ' lead' and x.trailing_comment == ' tail')
  assert(z.comments[1]:find('"y"', 1, true))
end)

test('nested blocks and duplicate keys keep order', function()
  local doc = kv.parse('"r" { "OnSpellStart" { "RunScript" { "Function" "a" } "RunScript" { "Function" "b" } } }')
  local events = kv.find(doc.nodes[1].children, 'onspellstart')
  local scripts = kv.find_all(events.children, 'RunScript')
  assert(#scripts == 2 and kv.value(scripts[1].children, 'Function') == 'a' and kv.value(scripts[2].children, 'Function') == 'b')
  local depth = 0
  kv.walk(doc.nodes, function(_, d) if d > depth then depth = d end end)
  assert(depth == 3)
end)

test('escapes decode while raw token is preserved', function()
  local text = '"k" "say \\"hi\\"\\n\\\\path\\q"'
  local n = kv.parse(text).nodes[1]
  assert(n.value == 'say "hi"\n\\path\\q', n.value)
  assert(n.value_token == '"say \\"hi\\"\\n\\\\path\\q"')
end)

test('繁中 strings, unquoted tokens and conditionals', function()
  local doc = kv.parse('"DOTA_Tooltip_Ability_B01W" "狙擊模式"\nkey value [$WIN32]\n')
  assert(doc.nodes[1].value == '狙擊模式' and not doc.invalid_utf8)
  local n = doc.nodes[2]
  assert(n.key == 'key' and n.value == 'value' and n.condition == '[$WIN32]' and not n.key_quoted)
  assert(kv.parse('"k" "\255"').invalid_utf8)
end)

test('rank arrays, placeholders and non-numeric values', function()
  local r = kv.rank_values('900 1050 1200 1400')
  assert(#r == 4 and r[1] == 900 and r[4] == 1400)
  local f = kv.rank_values('0.5 -1 .75')
  assert(f[1] == 0.5 and f[2] == -1 and f[3] == 0.75)
  assert(kv.rank_values('DOTA_UNIT_TARGET_HERO') == nil and kv.rank_values('') == nil and kv.rank_values(nil) == nil)
  assert(kv.rank_values('1 nan') == nil and kv.rank_values('1 inf') == nil)
  assert(kv.placeholder('%bonus_atk') == 'bonus_atk' and kv.placeholder('50%') == nil)
end)

test('missing values, orphan tokens and broken input give diagnostics', function()
  local doc = kv.parse('"a" { "x" }')
  assert(kinds(doc) == 'missing_value' and doc.nodes[1].children[1].missing_value)
  -- A lone token on its own line must not shift the following pairs.
  doc = kv.parse('"h"\n{\n\t\t"0"\n\t"MagicalResistance" "0"\n\t"AttackRange" "700"\n}\n')
  assert(kinds(doc) == 'orphan_token', kinds(doc))
  assert(kv.value(doc.nodes[1].children, 'AttackRange') == '700')
  assert(kv.value(doc.nodes[1].children, 'MagicalResistance') == '0')
  assert(kinds(kv.parse('"a" { "b" "c"')) == 'unclosed_block')
  assert(kinds(kv.parse('"a" "b" }')) == 'unexpected_close')
  assert(kinds(kv.parse('"a" "b')) == 'unterminated_string')
  assert(kinds(kv.parse('"a" { /* x')):find('unterminated_block_comment', 1, true))
end)

test('sha256 matches known vectors and lua-host', function()
  assert(sha256('abc') == 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad')
  assert(sha256(string.rep('a', 1000)) == '41edece42d63e8d9bf515a9ba6932e1c20cbc9f5a5d134645adb5db1b9737ea3')
  local file = path.join(root, 'hash.txt')
  local text = kv.BOM .. '繁中\r\n' .. string.rep('x', 70)
  path.write(file, text, true, true)
  assert(sha256(text) == hash.sha256(file))
end)

test('ability id grammar keeps variants apart', function()
  local p = catalog_mod.parse_ability_id
  assert(p('B01W').kind == 'skill' and p('B01W').slot == 'W' and p('B01W').variant == 'current')
  assert(p('B01W_old').variant == 'old' and p('B01W_old').slot == 'W')
  assert(p('A07W_20').variant == '_20' and p('A28_20TW').kind == 'sub_ability')
  assert(p('A13W2').kind == 'sub_ability' and p('B01_precache').kind == 'precache')
  assert(p('B16MMD').kind == 'auxiliary' and p('B07W_old_soldiercamp').kind == 'auxiliary')
end)

test('classification records evidence and refuses to guess', function()
  local c = classify.classify({ behavior_flags = { 'NO_TARGET' }, text = '能讓攻擊傷害加成的技法' })
  assert(c.status == 'unclassified' and #c.families == 0)
  c = classify.classify({ behavior_flags = { 'POINT', 'AOE' }, block_keys = { LinearProjectile = true },
    modifier_keys = { MODIFIER_STATE_STUNNED = true }, text = '召喚鳳凰，並將敵人暈眩' })
  local tags = {}
  for _, t in ipairs(c.tags) do tags[t.tag] = t end
  assert(tags.circle and tags.projectile_linear and tags.stun and tags.summon)
  assert(tags.summon.evidence[1].source == 'tooltip_text' and tags.projectile_linear.evidence[1].source == 'kv_block')
  assert(c.basis == 'kv+text')
  -- Ordered pairs must stay within one clause.
  assert(not classify.classify({ text = '回復法力。造成生命傷害' }).tags[1])
end)

test('serializer is deterministic, quotes keywords and round-trips bytes', function()
  local value = { b = { 2, 1 }, a = 'x\r\n\t"\\繁', ['function'] = 'f', [1.5] = true, n = 0.1 }
  local s1, s2 = serialize.serialize(value), serialize.serialize(value)
  assert(s1 == s2)
  local back = assert(load(s1, 'ser', 't', {}))()
  assert(back.a == value.a and back['function'] == 'f' and back.b[2] == 1 and back.n == 0.1 and back[1.5])
  -- Tables with holes keep every entry (a sequence check on #t would drop [3] and [4]).
  local holed = assert(load(serialize.serialize({ [1] = 'q', [3] = 'e', [4] = 'r' }), 'h', 't', {}))()
  assert(holed[1] == 'q' and holed[2] == nil and holed[3] == 'e' and holed[4] == 'r')
end)

test('local mapping never pairs by number alone', function()
  local heroes = { { source_hero_id = 'B01', name_zhtw = '雜賀孫市' }, { source_hero_id = 'B02', name_zhtw = '風魔小太郎' },
    { source_hero_id = 'B04', name_zhtw = '伊達政宗' } }
  local locals = { { number = 'B01', game_hero_id = 'saika_magoichi', name_zhtw = '雜賀孫市', dir = 'd1' },
    { number = 'B02', game_hero_id = 'date_masamune', name_zhtw = '伊達政宗', dir = 'd2' } }
  local m = catalog_mod.map_local(heroes, locals).entries
  assert(m[1].status == 'name_and_number_match' and m[1].game_hero_id == 'saika_magoichi')
  assert(m[2].status == 'conflict_number_only' and m[2].game_hero_id == nil and m[2].number_conflict.game_hero_id == 'date_masamune')
  assert(m[3].status == 'name_match_number_differs' and m[3].game_hero_id == 'date_masamune')
end)

test('fixture source builds, verifies and detects tampering', function()
  local src = path.join(root, 'src')
  local commit = string.rep('a', 40)
  path.write(path.join(src, '.git', 'HEAD'), commit .. '\n', true, true)
  -- history_nobu layout (E169): addon under game/dota_addons/nobu, UTF-16LE tooltips, names in
  -- addon_game_mode.lua, selection by herolist.txt, unit defaults in npc_heroes_override.txt.
  local addon = path.join(src, 'game', 'dota_addons', 'nobu')
  local npc = path.join(addon, 'scripts', 'npc')
  path.write(path.join(addon, 'scripts', 'vscripts', 'addon_game_mode.lua'),
    '_G.hero_name_zh = {\n  B01 = "雜賀孫市",\n  B02 = "風魔小太郎",\n}\n', true, true)
  path.write(path.join(addon, 'resource', 'addon_tchinese.txt'), require('hero_source.text').utf8_to_utf16le('"lang"\r\n{\r\n\t"Tokens"\r\n\t{\r\n' ..
    '\t\t"DOTA_Tooltip_Ability_B01W"\t"狙擊模式"\r\n' ..
    '\t\t"DOTA_Tooltip_Ability_B01W_Description"\t"增加<font color=\'#5cb85c\'>(1/2)</font>攻擊距離"\r\n\t}\r\n}\r\n'), true, true)
  for _, faction in ipairs({ 'A_Oda', 'B_Unified', 'C_Neutral' }) do path.mkdir_p(path.join(npc, faction)) end
  path.write(path.join(npc, 'herolist.txt'), '"CustomHeroList"\r\n{\r\n\t"npc_dota_hero_nevermore" "1" //雜賀孫市\r\n}\r\n', true, true)
  path.write(path.join(npc, 'npc_heroes_override.txt'), '"DOTAHeroes"\r\n{\r\n\t"npc_dota_hero_base"\r\n\t{\r\n' ..
    '\t\t"AttackAcquisitionRange"\t\t"800"\r\n\t}\r\n}\r\n', true, true)
  path.write(path.join(npc, 'B_Unified', 'B01.txt'), '//雜賀孫市\r\n"DOTAAbilities"\r\n{\r\n' ..
    '\t"B01_precache" { "BaseClass" "ability_datadriven" }\r\n' ..
    '\t"B01W"\r\n\t{\r\n\t\t"AbilityBehavior"\t"DOTA_ABILITY_BEHAVIOR_TOGGLE"\r\n\t\t"AbilityCastRange"\t"%range"\r\n' ..
    '\t\t"AbilitySpecial" { "01" { "var_type" "FIELD_INTEGER" "range" "900 1050" } }\r\n\t}\r\n' ..
    '\t"B01W_old" { "AbilityBehavior" "DOTA_ABILITY_BEHAVIOR_PASSIVE" "AbilityCooldown" }\r\n}\r\n', true, true)
  path.write(path.join(npc, 'B_Unified', 'B01_2.txt'), kv.BOM .. '"DOTAHeroes"\r\n{\r\n\t"B01"\r\n\t{\r\n' ..
    '\t\t"override_hero"\t"npc_dota_hero_nevermore"\r\n' ..
    '\t\t"Ability1"\t"B01W"\r\n\t\t"AttackCapabilities"\t"DOTA_UNIT_CAP_RANGED_ATTACK"\r\n\t}\r\n}\r\n', true, true)
  local fake_repo = path.join(root, 'repo')
  path.mkdir_p(path.join(fake_repo, 'scripts', 'base_content', 'src', 'heroes', 'B02_date_masamune'))
  path.write(path.join(fake_repo, 'scripts', 'base_content', 'src', 'heroes', 'B02_date_masamune', 'mod.rs'),
    '//! 伊達政宗（Date Masamune）— B02\n', true, true)
  local function build(c)
    return catalog_mod.build({ source = src:gsub('\\', '/'), repository = 'fixture', commit = c,
      repo_root = fake_repo:gsub('\\', '/') })
  end
  local ok, err = pcall(build, string.rep('b', 40))
  assert(not ok and tostring(err):find('expected', 1, true), 'wrong commit must be rejected')
  local catalog = build(commit)
  assert(serialize.serialize(build(commit)) == serialize.serialize(catalog), 'catalog output must be byte-stable')
  assert(#catalog.heroes == 1 and catalog.heroes[1].name_zhtw == '雜賀孫市' and catalog.heroes[1].listed_on_site)
  local by = {}
  for _, a in ipairs(catalog.abilities) do by[a.id] = a end
  local w = by.B01W
  assert(w.variant == 'current' and by.B01W_old.variant == 'old' and w.in_unit_ability_list)
  assert(w.specials[1].ranks[2] == 1050 and w.placeholders[1] == 'range' and not w.unresolved_placeholders)
  assert(w.tooltip.description.value:find('<font', 1, true) and w.description_plain == '增加(1/2)攻擊距離')
  assert(w.classification.status == 'classified')
  assert(by.B01W_old.fields[2].missing_value)
  assert(catalog.hero_unit_defaults[1].key == 'AttackAcquisitionRange', 'hero base defaults indexed')
  assert(require('hero_source.roster').stats(catalog.heroes[1].unit, catalog.hero_unit_defaults).acquisition_range == 800,
    'a unit without AttackAcquisitionRange inherits npc_dota_hero_base')
  local text = path.read(path.join(npc, 'B_Unified', 'B01.txt'), true)
  assert(text:sub(w.start, w.stop) == w.raw and w.key_token == '"B01W"' and w.raw_sha256 == sha256(w.raw))
  assert(catalog.id_mapping.entries[1].number_conflict == nil)
  local problems = catalog_mod.verify(catalog, src)
  assert(#problems == 0, table.concat(problems, '\n'))
  local reloaded = assert(load(serialize.serialize(catalog), 'cat', 't', {}))()
  assert(#catalog_mod.verify(reloaded, src) == 0)
  -- The raw source pack carries source facts only: provenance, KV and tooltip
  -- text in slot order, never the classification.
  local pack = require('hero_source.export').render(catalog, 'B01')
  assert(pack:find(w.raw_sha256, 1, true) and pack:find('增加(1/2)攻擊距離', 1, true) and pack:find('"range" "900 1050"', 1, true))
  assert(pack:find('### B01W：狙擊模式（單位技能欄 1）', 1, true) and pack:find('### B01W_old', 1, true))
  assert(not pack:find('classified', 1, true), 'no analysis in a raw pack')
  path.write(path.join(npc, 'B_Unified', 'B01.txt'), text:gsub('900 1050', '900 1051'), true, true)
  problems = catalog_mod.verify(reloaded, src)
  assert(#problems > 0, 'tampered source must fail verification')
end)

test('requirements derive worst status and reject stale or unmapped reviews', function()
  local requirements = require('hero_source.requirements')
  local catalog = {
    source = { commit = 'c' },
    heroes = { { source_hero_id = 'B01', ability_list = { { ability = 'B01W' }, { ability = 'B01E' } } } },
    abilities = {
      { id = 'B01W', source_hero_id = 'B01', variant = 'current', kind = 'skill', raw_sha256 = 'h1' },
      { id = 'B01E', source_hero_id = 'B01', variant = 'current', kind = 'skill', raw_sha256 = 'h2' },
    },
  }
  local function req()
    return { schema = requirements.SCHEMA, catalog_commit = 'c',
      capabilities = { { id = 'buff', status = 'supported', evidence = 'x' }, { id = 'charges', status = 'needs_framework', evidence = 'y', g_package = 'G-2' } },
      heroes = { { source_hero_id = 'B01', variant = 'current', roster = 'delivery',
        slots = { { ability = 'B01W', action = 'Q' }, { ability = 'B01E', action = 'W' } },
        abilities = {
          { id = 'B01W', reviewed_sha256 = 'h1', read = true, bot_usage = { target = 'self', when = 'w' }, mechanics = { { what = 'w', capability = 'buff' } } },
          { id = 'B01E', reviewed_sha256 = 'h2', read = true, bot_usage = { target = 'point', when = 'e' }, mechanics = { { what = 'e', capability = 'buff' }, { what = 'c', capability = 'charges' } } },
        } } } }
  end
  local derived, problems = requirements.validate(req(), catalog)
  assert(#problems == 0, table.concat(problems, '\n'))
  assert(derived.heroes.B01.abilities.B01W == 'supported' and derived.heroes.B01.status == 'needs_framework')
  assert(derived.heroes.B01.g_packages[1].g == 'G-2' and derived.heroes.B01.g_packages[1].capability == 'charges')
  local function rejects(mutate, fragment)
    local r = req(); mutate(r)
    local _, p = requirements.validate(r, catalog)
    local all = table.concat(p, '\n')
    assert(all:find(fragment, 1, true), 'expected "' .. fragment .. '" in: ' .. all)
  end
  rejects(function(r) r.heroes[1].abilities[1].reviewed_sha256 = 'old' end, 'stale')
  rejects(function(r) r.heroes[1].abilities[1].read = false end, 'must be read')
  rejects(function(r) table.remove(r.heroes[1].slots, 2) end, 'B01E is not mapped')
  rejects(function(r) r.heroes[1].abilities[2].status = 'supported' end, 'mechanics derive needs_framework')
  rejects(function(r) r.heroes[1].abilities[1].mechanics[1].capability = 'nope' end, 'unknown capability')
  rejects(function(r) r.heroes[1].abilities[1].mechanics[1].status_override = 'supported' r.heroes[1].abilities[2].mechanics[2].status_override = 'supported' end, 'needs a reason')
  rejects(function(r) r.heroes[1].slots[2].action = 'Q' end, 'used twice')
  rejects(function(r) r.catalog_commit = 'other' end, 'reviewed against')
  rejects(function(r) r.capabilities[2].g_package = nil end, 'needs g_package')
  rejects(function(r) r.heroes[1].abilities[1].bot_usage = nil end, 'needs bot_usage')
  rejects(function(r) r.heroes[1].roster = 'blind' end, 'do not author bot_usage')
  rejects(function(r) r.heroes[1].slots[2].action = 'extra1' end, 'without an extra_action_slot mechanic')
  local md = requirements.render(req(), catalog, derived)
  assert(md:find('`needs_framework`', 1, true))
end)

-- E170: kit coverage reports per-ability source parity from each package's source.lua.
test('coverage reports reviewed, stale and unchecked source parity', function()
  local coverage = require('hero_source.coverage')
  assert(coverage.parity_status(nil) == coverage.PARITY_NONE)
  assert(coverage.parity_status({ source_ability_id = 'X' }) == coverage.PARITY_NONE)
  assert(coverage.parity_status({ parity = { card = 'parity.md' } }) == coverage.PARITY_REVIEWED)
  assert(coverage.parity_status({ parity = { card = 'parity.md' }, stale_review = { reason = 'r' } }) == coverage.PARITY_STALE)
  local md = coverage.render({
    { plan = 'H01', source = 'B01', name = 'N', package = 'p', kit_status = 'supported', acceptance = 'pending',
      parity = coverage.PARITY_REVIEWED, abilities = {
        { id = 'a', source_ability_id = 'B01W', action = 'Q', status = 'supported', capabilities = {}, parity = coverage.PARITY_REVIEWED },
        { id = 'b', source_ability_id = 'B01T', action = 'R', status = 'supported', capabilities = {}, parity = coverage.PARITY_STALE } } },
  })
  assert(md:find('| 欄位 | 技能 | 來源 | 狀態 | 原作對照 | 能力 |', 1, true), md)
  assert(md:find('| Q | `a` | B01W | supported | 已對照原作 |  |', 1, true), md)
  assert(md:find('| R | `b` | B01T | supported | 待重審 |  |', 1, true), md)
  assert(md:find('| H01 | B01 | N | `p` | supported | pending | 已對照原作 | 2 |', 1, true), md)
end)

local function remove_tree(target)
  assert(target:find('hero%-source%-tests'), 'unsafe fixture cleanup target')
  if path.is_directory(target) then
    for name in lfs.dir(target) do
      if name ~= '.' and name ~= '..' then remove_tree(path.join(target, name)) end
    end
    assert(lfs.rmdir(target))
  elseif path.exists(target) then
    assert(os.remove(target))
  end
end
remove_tree(root)
print(count .. ' hero source index tests passed')
