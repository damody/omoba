-- M6: the UE cue table (OmCueCatalogData.inl) is compiled from the cue catalog by
-- scripts/gen_ue_cue_table.lua and must match it; rendering keeps kinds, masks and both mappings.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
CUE_TABLE_AS_MODULE = true
local gen = require('gen_ue_cue_table')
CUE_TABLE_AS_MODULE = nil

local checks = 0
local function ok(cond, msg) checks = checks + 1; if not cond then error(msg, 2) end end

-- a small catalog renders every table
local text = gen.render({
  source = { repository = 'history_nobu', commit = 'abc1234' },
  cues = {
    b_hit = { kind = 'impact', kinds = { 'impact', 'effect' }, assets = { { path = 'particles\\x\\hit.vpcf' } } },
    a_vo = { kind = 'sound', assets = { { path = 'A.vo1' } } },
    c_generic = { kind = 'projectile', placeholder = 'generic' },
  },
  assets = {
    ['A.vo1'] = { kind = 'sound', cues = { 'a_vo' } },
    ['particles\\x\\hit.vpcf'] = { kind = 'particle', cues = { 'b_hit', 'a_vo' } },
    ['B.vo1'] = { kind = 'sound', cues = {} },
  },
})
ok(text:find('TEXT("history_nobu@abc1234")', 1, true), 'source marker')
ok(text:find('{ TEXT("a_vo"), EOmCueKind::Sound, 0x0004, 0, 1, false },', 1, true), 'sound row')
-- impact (9) and effect (1) bits
ok(text:find('{ TEXT("b_hit"), EOmCueKind::Impact, 0x0202, 1, 1, false },', 1, true), 'impact row with both kinds')
ok(text:find('{ TEXT("c_generic"), EOmCueKind::Projectile, 0x0008, 2, 0, true },', 1, true), 'generic row')
ok(text:find('TEXT("particles\\\\x\\\\hit.vpcf")', 1, true), 'backslashes escaped')
-- paths sorted: A.vo1 (cue 0), B.vo1 (none), particles (cues 1, 0)
ok(text:find('{ TEXT("A.vo1"), EOmCueAssetKind::Sound, 0, 1 },', 1, true), 'path row A')
ok(text:find('{ TEXT("B.vo1"), EOmCueAssetKind::Sound, 1, 0 },', 1, true), 'path row without cues')
ok(text:find('EOmCueAssetKind::Particle, 1, 2 },', 1, true), 'path row with two cues')
ok(text:find('\t0, 1, 0,', 1, true), 'path cue indices in catalog order')
ok(not pcall(gen.render, { source = { repository = 'r', commit = 'c' }, cues = { x = { kind = 'nope' } }, assets = {} }), 'unknown kind refused')

-- the live table is up to date with the live catalog
ok(gen.main({ '--check' }) == 0, 'OmCueCatalogData.inl is stale: run tools\\lua\\lua.exe scripts\\gen_ue_cue_table.lua')
print(string.format('ue_cue_table_test: %d checks passed', checks))
