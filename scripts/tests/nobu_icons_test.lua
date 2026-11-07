-- M6: original nobu icons (scripts/import_nobu_icons.lua) and how the HUD manifest uses them
-- (scripts/ue_hud_portrait_coverage.lua): only nobu's own files, manual assignments win, authored
-- rows are never replaced, nobu rows replace the shared placeholder and are derived like it.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local json = b.lib('json')
NOBU_ICONS_AS_MODULE = true
local nobu = require('import_nobu_icons')
NOBU_ICONS_AS_MODULE = nil
local coverage = require('ue_hud_portrait_coverage')

local checks = 0
local function ok(cond, msg) checks = checks + 1; if not cond then error(msg, 2) end end

-- plan --------------------------------------------------------------------------------------
local textures = {
  A01W = 'custom/A01/a01w', A01E = 'meepo_earthbind', A02W = 'dota2/a32_w',
  item_sword = 'item_custom/sword', item_bad = 'custom/icon_1',
}
local files = {
  ['custom/A01/a01w'] = 'game/x/spellicons/custom/A01/a01w.png',
  ['dota2/a32_w'] = 'game/x/spellicons/dota2/a32_w.png',
  ['item_custom/sword'] = 'game/x/items/custom/sword.png',
}
local supplement = {
  texture = function(sid) return textures[sid] end,
  texture_png = function(tex) return files[tex] end,
  hero_icon = function(sid) return sid == 'A02' and 'content/x/panorama/images/heroes/npc_dota_hero_axe.png' or nil end,
}
local catalog = {
  heroes = { { source_id = 'A01' }, { source_id = 'A02' }, { source_id = 'A03' } },
  abilities = {
    { id = 'a01_w', v3 = { source_id = 'A01W' } },
    { id = 'a01_e', v3 = { source_id = 'A01E' } },
    { id = 'a02_w', v3 = { source_id = 'A02W' } },
    { id = 'a02_e', v3 = { source_id = 'A02E' } },
  },
  items = { { id = 'item_sword', source_id = 'item_sword' }, { id = 'item_bad', source_id = 'item_bad' } },
}
local rows = nobu.plan(catalog, supplement, {
  packages = { A01 = 'oda_a', A02 = 'oda_b', A03 = 'oda_c' },
  site_pics = nil,
  overrides = { ability = { a01_e = 'game/x/spellicons/custom/manual.png' }, item = { sword = false } },
})
local by = {}
for _, r in ipairs(rows) do by[r.kind .. ':' .. r.id] = r end
ok(by['ability:a01_w'].status == 'original' and by['ability:a01_w'].origin == files['custom/A01/a01w'], 'custom texture is original')
ok(by['ability:a01_e'].status == 'original' and by['ability:a01_e'].manual, 'a manual assignment wins over a Dota texture')
ok(by['ability:a02_w'].status == 'placeholder' and by['ability:a02_w'].reason:find('outside nobu', 1, true), 'Dota copies are placeholders')
ok(by['ability:a02_e'].status == 'placeholder' and by['ability:a02_e'].reason:find('no AbilityTextureName', 1, true), 'missing texture')
ok(by['item:sword'].status == 'placeholder' and by['item:sword'].reason == 'override: placeholder', 'manual placeholder')
ok(by['item:bad'].reason:find('Dota built-in', 1, true), 'texture the addon does not ship')
ok(by['hero:oda_b'].status == 'original' and by['hero:oda_a'].status == 'placeholder', 'portrait from override_hero only')
ok(by['hero:oda_c'].reason:find('no nobu portrait', 1, true), 'portrait missing')
ok(rows[1].kind == 'ability' and rows[#rows].kind == 'item', 'rows sorted by kind then id')
ok(nobu.file_name('ability', 'a01_w') == 'nobu-ability-a01_w.png', 'file name')
ok(not pcall(nobu.file_name, 'hero', 'Bad-Id'), 'ids are checked')
local encoded = nobu.encode_index({ schema = nobu.SCHEMA, edge = 128, source = { repository = 'history_nobu', commit = 'c' }, entries = rows })
local back = json.decode(encoded)
ok(back.schema == nobu.SCHEMA and #back.entries == #rows and back.entries[1].id == rows[1].id, 'index round-trips, one entry per line')
ok(nobu.render_report(back):find('`a02_w`', 1, true), 'report lists placeholders')

-- coverage ----------------------------------------------------------------------------------
local index = { schema = 'omoba.nobu-icons.v1', entries = {
  { kind = 'hero', id = 'new_hero', status = 'original', file = 'nobu-hero-new_hero.png' },
  { kind = 'hero', id = 'old_hero', status = 'original', file = 'nobu-hero-old_hero.png' },
  { kind = 'ability', id = 'new_q', status = 'original', file = 'nobu-ability-new_q.png' },
  { kind = 'ability', id = 'new_w', status = 'placeholder' },
} }
local n = coverage.nobu_files(index)
ok(n.portrait.new_hero == 'nobu-hero-new_hero.png' and n.ability.new_q and not n.ability.new_w, 'index -> files')
ok(next(coverage.nobu_files(nil).portrait) == nil, 'no index, no files')
ok(not pcall(coverage.nobu_files, { schema = 'omoba.nobu-icons.v1', entries = { { kind = 'hero', id = 'x', status = 'original', file = 'other.png' } } }), 'a borrowed file is refused')

local heroes = {
  { id = 'old_hero', display_name = 'Old', abilities = { 'old_q' } },
  { id = 'new_hero', display_name = 'New', abilities = { 'new_q', 'new_w' } },
  { id = 'bare_hero', display_name = 'Bare', abilities = {} },
}
local abilities = { { id = 'old_q', display_name = 'OQ' }, { id = 'new_q', display_name = 'NQ' }, { id = 'new_w', display_name = 'NW' } }
local manifest = { schemaVersion = 1, entries = {
  { kind = 'portrait', id = 'old_hero', displayName = 'Old', file = 'old-hero-v1.png', columns = 1, rows = 1, tile = 0 },
  { kind = 'ability', id = 'old_q', displayName = 'OQ', file = 'ability-placeholder-v1.png', columns = 1, rows = 1, tile = 0, placeholder = true },
} }
ok(#coverage.upgrade_placeholders(manifest, n) == 0, 'no nobu art for old_q: stays a placeholder')
local added = coverage.placeholder_entries(heroes, manifest, n)
ok(#added == 2, 'two heroes without portraits')
local rows_by = {}
local function index_rows() rows_by = {}; for _, r in ipairs(manifest.entries) do rows_by[r.kind .. ':' .. r.id] = r end end
index_rows()
ok(rows_by['portrait:old_hero'].file == 'old-hero-v1.png', 'authored portrait kept although nobu art exists')
ok(rows_by['portrait:new_hero'].source == 'nobu' and rows_by['portrait:new_hero'].placeholder == nil, 'nobu portrait for a new hero')
ok(rows_by['portrait:bare_hero'].placeholder == true, 'no nobu art: shared placeholder')
coverage.ability_placeholder_entries(heroes, abilities, manifest, n)
index_rows()
ok(rows_by['ability:new_q'].file == 'nobu-ability-new_q.png' and rows_by['ability:new_w'].placeholder == true, 'ability rows')
n.portrait.bare_hero = 'nobu-hero-bare_hero.png'
local up = coverage.upgrade_placeholders(manifest, n)
ok(#up == 1 and up[1] == 'portrait:bare_hero', 'a placeholder takes new nobu art')
local inspect = function() return { width = 128, height = 128 } end
ok(coverage.validate(heroes, manifest, inspect) == 3, 'portraits validate with nobu rows')
ok(coverage.validate_abilities(heroes, abilities, manifest, inspect) == 3, 'abilities validate with nobu rows')
index_rows()
rows_by['portrait:new_hero'].file = 'nobu-hero-old_hero.png'
ok(not pcall(coverage.validate, heroes, manifest, inspect), 'a nobu row must use its own file')
rows_by['portrait:new_hero'].file = 'nobu-hero-new_hero.png'
local removed = coverage.prune_placeholders({ heroes[1], heroes[3] }, manifest)
ok(#removed == 1 and removed[1] == 'new_hero', 'nobu rows are derived: pruned with their hero')
local removed_abilities = coverage.prune_ability_placeholders({ heroes[1] }, abilities, manifest)
ok(#removed_abilities == 2, 'nobu and placeholder ability rows pruned when no hero equips them')
index_rows()
ok(rows_by['portrait:old_hero'] and rows_by['ability:old_q'], 'authored and still-equipped rows stay')

-- items: shop items without art get raw nobu rows; fixtures and authored rows are left alone
local item_index = { schema = 'omoba.nobu-icons.v1', entries = {
  { kind = 'item', id = 'katana', status = 'original', file = 'nobu-item-katana.png' },
  { kind = 'item', id = 'fixture', status = 'original', file = 'nobu-item-fixture.png' },
  { kind = 'item', id = 'drawn', status = 'original', file = 'nobu-item-drawn.png' },
} }
local ni = coverage.nobu_files(item_index)
local shop = {
  { id = 'katana', name = '刀' }, { id = 'fixture', name = 'F', test_only = true },
  { id = 'drawn', name = 'D' }, { id = 'bare', name = 'B' },
}
local items_manifest = { schemaVersion = 1, entries = {
  { kind = 'item', id = 'drawn', file = 'drawn.png', resource = '/Game/Omoba/UI/Items/drawn.drawn', columns = 1, rows = 1, tile = 0 },
} }
local added_items = coverage.item_entries(shop, items_manifest, ni)
ok(#added_items == 1 and added_items[1] == 'katana', 'only a sold item without art gets nobu art')
ok(items_manifest.entries[2].file == 'nobu-item-katana.png' and items_manifest.entries[2].displayName == '刀'
  and items_manifest.entries[2].resource == nil and items_manifest.entries[2].source == 'nobu', 'raw nobu item row')
ok(#coverage.prune_items(shop, items_manifest, ni) == 0, 'current rows stay')
shop[1].test_only = true
local pruned = coverage.prune_items(shop, items_manifest, ni)
ok(#pruned == 1 and pruned[1] == 'katana' and #items_manifest.entries == 1, 'a nobu row leaves with its item; authored rows stay')

-- the site shows the HUD's art: authored first, then nobu, never an atlas tile
do
  local paths = b.lib('path')
  local tmp = b.root .. '/target/nobu-icons-test'
  local ui = tmp .. '/omfue/Content/Omoba/UI'
  paths.mkdir_p(ui .. '/Abilities/Source')
  local function touch(p) local f = assert(io.open(p, 'wb')); f:write('x'); f:close() end
  touch(ui .. '/oda-v1.png'); touch(ui .. '/nobu-hero-oda.png'); touch(ui .. '/nobu-hero-mori.png')
  touch(ui .. '/Abilities/Source/oda_q.png'); touch(ui .. '/nobu-ability-oda_w.png'); touch(ui .. '/atlas.png')
  local f = assert(io.open(ui .. '/nobu-icons-v1.json', 'wb'))
  f:write(json.encode({ schema = 'omoba.nobu-icons.v1', entries = {
    { kind = 'hero', id = 'oda', source_id = 'A01', status = 'original', file = 'nobu-hero-oda.png' },
    { kind = 'hero', id = 'mori', source_id = 'A02', status = 'original', file = 'nobu-hero-mori.png' },
    { kind = 'ability', id = 'oda_q', source_id = 'A01W', status = 'original', file = 'nobu-ability-oda_q.png' },
    { kind = 'ability', id = 'oda_w', source_id = 'A01E', status = 'original', file = 'nobu-ability-oda_w.png' },
    { kind = 'ability', id = 'oda_e', source_id = 'A01R', status = 'placeholder' },
  } })); f:close()
  f = assert(io.open(ui .. '/hud-assets-v1.json', 'wb'))
  f:write(json.encode({ schemaVersion = 1, entries = {
    { kind = 'portrait', id = 'oda', file = 'oda-v1.png', columns = 1, rows = 1, tile = 0 },
    { kind = 'ability', id = 'oda_q', file = 'oda_q.png', resource = '/Game/Omoba/UI/Abilities/oda_q.oda_q', columns = 1, rows = 1, tile = 0 },
    { kind = 'ability', id = 'oda_w', file = 'atlas.png', columns = 4, rows = 4, tile = 3 },
  } })); f:close()
  local site_icons = require('site.ue_icons')
  local w, wrapped = site_icons.wrap({ available = true }, tmp)
  ok(wrapped, 'index present: wrapped')
  ok(w.hero_icon('A01') == 'ue/hero-oda.png' and w.files['ue/hero-oda.png'] == ui .. '/oda-v1.png', 'authored portrait wins')
  ok(w.files[w.hero_icon('A02')] == ui .. '/nobu-hero-mori.png', 'nobu portrait otherwise')
  ok(w.files[w.ability_icon('A01W')] == ui .. '/Abilities/Source/oda_q.png', 'imported ability source')
  ok(w.files[w.ability_icon('A01E')] == ui .. '/nobu-ability-oda_w.png', 'atlas tile not used; nobu instead')
  ok(w.ability_icon('A01R') == nil and w.item_icon('item_x') == nil, 'placeholder and unknown give nil')
  local _, none = site_icons.wrap({}, tmp .. '/missing')
  ok(not none, 'no index: the original supplement stays')
end

-- the live index, when present, matches its files
local live = b.root .. '/' .. nobu.UI_DIR .. '/' .. nobu.INDEX
if io.open(live, 'rb') then ok(nobu.check(b.root) > 0, 'live nobu icons match the index') end
print(string.format('nobu_icons_test: %d checks passed', checks))
