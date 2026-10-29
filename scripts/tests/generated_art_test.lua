-- M6: generated art (scripts/generated_art.lua) fills the ids nobu has no original for. Original
-- nobu art wins, hand-assigned rows win over both, generated rows are derived like nobu rows, and
-- the import resamples the reviewed original and records its provenance.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local json = b.lib('json')
local paths = b.lib('path')
GENERATED_ART_AS_MODULE = true
local ga = require('generated_art')
local coverage = require('ue_hud_portrait_coverage')

local checks = 0
local function ok(cond, msg) checks = checks + 1; if not cond then error(msg, 2) end end

-- index and overlay ---------------------------------------------------------------------------
ok(ga.file('ability', 'a01_net') == 'gen-ability-a01_net.png', 'file name')
ok(not pcall(ga.file, 'portrait', 'x'), 'kind is hero, not portrait')
local gen = { schema = ga.SCHEMA, entries = {
  { kind = 'hero', id = 'bare_hero', file = 'gen-hero-bare_hero.png' },
  { kind = 'hero', id = 'nobu_hero', file = 'gen-hero-nobu_hero.png' },
  { kind = 'ability', id = 'new_w', file = 'gen-ability-new_w.png' },
  { kind = 'cue', id = 'a01w_glow', file = 'gen-cue-a01w_glow.png' },
} }
ok(not pcall(ga.rows, { schema = ga.SCHEMA, entries = { { kind = 'hero', id = 'x', file = 'gen-hero-y.png' } } }), 'a borrowed file is refused')
local nobu = { schema = 'omoba.nobu-icons.v1', edge = 128, entries = {
  { kind = 'hero', id = 'nobu_hero', source_id = 'A01', status = 'original', file = 'nobu-hero-nobu_hero.png' },
  { kind = 'hero', id = 'bare_hero', source_id = 'A02', status = 'placeholder', reason = 'dota art' },
  { kind = 'ability', id = 'new_w', source_id = 'A02W', status = 'placeholder', reason = 'dota art' },
  { kind = 'ability', id = 'new_e', source_id = 'A02E', status = 'placeholder', reason = 'dota art' },
} }
local over = ga.overlay(nobu, gen)
ok(over.edge == 128 and #over.entries == 4, 'overlay keeps the index shape')
ok(over.entries[1].status == 'original' and over.entries[1].file == 'nobu-hero-nobu_hero.png', 'original nobu art is never replaced')
ok(over.entries[2].status == 'generated' and over.entries[2].file == 'gen-hero-bare_hero.png' and over.entries[2].reason == nil, 'placeholder -> generated')
ok(over.entries[4].status == 'placeholder', 'no generated art: still a placeholder')
ok(nobu.entries[2].status == 'placeholder', 'overlay copies, never edits the nobu index')

-- HUD coverage --------------------------------------------------------------------------------
local files = coverage.nobu_files(nobu, gen)
ok(files.portrait.nobu_hero == 'nobu-hero-nobu_hero.png' and files.sources.portrait.nobu_hero == 'nobu', 'nobu wins over generated')
ok(files.portrait.bare_hero == 'gen-hero-bare_hero.png' and files.sources.portrait.bare_hero == 'generated', 'generated fills the gap')
ok(files.ability.new_w and not files.ability.new_e, 'ability files')
local heroes = {
  { id = 'authored_hero', display_name = 'A', abilities = {} },
  { id = 'bare_hero', display_name = 'Bare', abilities = { 'new_w', 'new_e' } },
}
local abilities = { { id = 'new_w', display_name = 'NW' }, { id = 'new_e', display_name = 'NE' } }
local manifest = { schemaVersion = 1, entries = {
  { kind = 'portrait', id = 'authored_hero', displayName = 'A', file = 'authored-v1.png', columns = 1, rows = 1, tile = 0 },
  { kind = 'ability', id = 'new_w', displayName = 'NW', file = 'ability-placeholder-v1.png', columns = 1, rows = 1, tile = 0, placeholder = true },
} }
local up = coverage.upgrade_placeholders(manifest, files)
ok(#up == 1 and up[1] == 'ability:new_w' and manifest.entries[2].source == 'generated' and manifest.entries[2].file == 'gen-ability-new_w.png', 'a placeholder takes generated art')
coverage.placeholder_entries(heroes, manifest, files)
coverage.ability_placeholder_entries(heroes, abilities, manifest, files)
local by = {}
for _, r in ipairs(manifest.entries) do by[r.kind .. ':' .. r.id] = r end
ok(by['portrait:bare_hero'].source == 'generated' and by['portrait:bare_hero'].placeholder == nil, 'generated portrait for a new hero')
ok(by['ability:new_e'].placeholder == true, 'no art at all: shared placeholder')
local inspect = function() return { width = 256, height = 256 } end
ok(coverage.validate(heroes, manifest, inspect) == 2, 'portraits validate with generated rows')
ok(coverage.validate_abilities(heroes, abilities, manifest, inspect) == 2, 'abilities validate with generated rows')
by['portrait:bare_hero'].file = 'nobu-hero-bare_hero.png'
ok(not pcall(coverage.validate, heroes, manifest, inspect), 'a generated row must use its own gen- file')
by['portrait:bare_hero'].file = 'gen-hero-bare_hero.png'
ok(#coverage.prune_placeholders({ heroes[1] }, manifest) == 1, 'generated rows are derived: pruned with their hero')
ok(#coverage.prune_ability_placeholders({ heroes[1] }, abilities, manifest) == 2, 'generated ability rows pruned when no hero equips them')

-- import --------------------------------------------------------------------------------------
do
  local tmp = b.root .. '/target/generated-art-test'
  local ui = tmp .. '/' .. ga.UI_DIR
  paths.mkdir_p(ui)
  local src = tmp .. '/original.png'
  -- 2x2 opaque PNG made by the host, so the test needs no fixture file
  local host = b.lib('host')
  local seed = b.root .. '/omfue/Content/Omoba/UI/ability-placeholder-v1.png'
  host.call('png_square_icon', { source = seed, output = src, edge = 64 })
  local stage = { generated = '2026-10-29', assets = {
    { kind = 'ability', id = 'new_w', source = src, prompt = 'p', review = 'viewed', attempts = 2, rejected = { { attempt = 'a1', reason = 'frame' } } },
    { kind = 'cue', id = 'a01w_glow', source = src, prompt = 'p', review = 'viewed' },
  } }
  ok(not pcall(ga.import, tmp, { generated = '2026-10-29', assets = { { kind = 'ability', id = 'x', source = src, prompt = 'p' } } }), 'an unreviewed image is refused')
  local n, total = ga.import(tmp, stage)
  ok(n == 2 and total == 2, 'imported two rows')
  ok(paths.is_file(ui .. '/gen-ability-new_w.png') and paths.is_file(tmp .. '/' .. ga.CUE_DIR .. '/gen-cue-a01w_glow.png'), 'files written where they belong')
  ok(ga.check(tmp) == 2, 'check verifies the recorded hashes')
  local index = ga.read_index(tmp)
  local row = ga.rows(index).ability.new_w
  ok(row.edge == ga.EDGE and row.tool == ga.TOOL and row.attempts == 2 and row.rejected[1].reason == 'frame' and row.original_size[1] == 64, 'provenance recorded')
  stage.assets = { stage.assets[1] }
  ok(select(2, ga.import(tmp, stage)) == 2, 'importing an id again replaces its row')
end

print('generated_art_test: ' .. checks .. ' checks passed')
