-- Only source heroes (history_nobu) may be in the game. The content build enforces the
-- source_ref / test_only markers (template-ids validate_hero_sources); this test enforces the
-- rest of the contract from the Lua side:
--   * templates/hero_sources.lua is the roster generated from the pinned source-catalog;
--   * every hero is a roster hero (source_ref) or explicitly test_only, never both or neither;
--   * every game match recipe (scripts/lua_data/*.lua) seats only source heroes;
--   * recipes that seat test-only heroes live under scripts/test_fixtures/recipes;
--   * the hero_author generator writes source_ref from a package's provenance.
package.path = 'scripts/?.lua;' .. package.path
local b = require('_bootstrap')
local path = b.lib('path')
local lfs = require('lfs')
local roster = require('hero_source.roster')
local generate = require('hero_author.generate')

local count = 0
local function test(name, fn) fn(); count = count + 1; print('PASS ' .. name) end

local content_root = path.join(b.root, 'scripts', 'lua_data')
local builder = require('content_builder').new(content_root)
local sources = builder.include('templates/hero_sources.lua')
local heroes = builder.include('templates/heroes.lua')

test('roster is generated from the pinned source catalog', function()
  local catalog = assert(loadfile(path.join(b.root, 'docs', 'plans', '2026-10-24-omfue-month-mvp', 'source-index', 'source-catalog.lua'), 't', {}))()
  local expected = roster.render(catalog)
  local actual = path.read(path.join(content_root, 'templates', 'hero_sources.lua'), true):gsub('\r\n', '\n')
  assert(actual == expected, 'templates/hero_sources.lua is stale; run tools\\lua\\lua.exe scripts\\hero_source_index.lua roster')
  assert(sources.name == roster.NAME and sources.commit == catalog.source.commit)
end)

local by_source, playable = {}, {}
for _, entry in ipairs(sources.heroes) do by_source[entry.id] = entry end

test('every hero is a roster hero or explicitly test-only', function()
  local claimed = {}
  for _, hero in ipairs(heroes) do
    if not hero.tombstone then
      assert((hero.source_ref ~= nil) ~= (hero.test_only == true), hero.id .. ': exactly one of source_ref / test_only')
      if hero.source_ref then
        local site, commit, id = hero.source_ref:match('^([%w_]+)@(%x+):([ABC]%d%d)$')
        assert(site == sources.name and commit == sources.commit, hero.id .. ': ' .. hero.source_ref)
        assert(by_source[id], hero.id .. ': ' .. id .. ' is not a source hero')
        assert(not claimed[id], id .. ' claimed twice')
        claimed[id] = true
        playable[hero.id] = true
      end
    end
  end
  for _, id in ipairs({ 'saika_magoichi', 'date_masamune', 'omatsu', 'oichi', 'inahime', 'asai_nagamasa', 'murakami_yoshikiyo', 'tamako' }) do
    assert(playable[id], id .. ' must be playable')
  end
  for _, hero in ipairs(heroes) do
    if hero.id:match('^training_') then assert(hero.test_only and not playable[hero.id], hero.id .. ' must be test-only') end
  end
end)

local function load_recipe(file)
  local ok, plan = pcall(function() return assert(loadfile(file))() end)
  assert(ok, file .. ': ' .. tostring(plan))
  return plan
end

test('every game recipe seats only source heroes', function()
  local checked = 0
  for name in lfs.dir(content_root) do
    if name:match('^moba_.*%.lua$') then
      local plan = load_recipe(path.join(content_root, name))
      for _, player in ipairs(plan.players or {}) do
        assert(playable[player.hero], name .. ': player ' .. tostring(player.player_id) .. ' is ' .. tostring(player.hero) .. ', not a source hero')
      end
      checked = checked + 1
    end
  end
  assert(checked >= 10, 'expected the game recipes under scripts/lua_data, found ' .. checked)
end)

test('test-only recipes are fixtures, never game recipes', function()
  local fixtures = path.join(b.root, 'scripts', 'test_fixtures', 'recipes')
  local any_test_only = false
  for name in lfs.dir(fixtures) do
    if name:match('%.lua$') then
      local text = path.read(path.join(fixtures, name), true)
      assert(text:find('TEST FIXTURE', 1, true), name .. ' must say it is a test fixture')
      for _, player in ipairs(load_recipe(path.join(fixtures, name)).players or {}) do
        if not playable[player.hero] then any_test_only = true end
      end
    end
  end
  assert(any_test_only, 'fixtures exist to exercise test-only kits')
end)

test('hero_author writes source_ref from package provenance', function()
  local provenance = { repository = 'https://github.com/damody/history_nobu', commit = sources.commit, source_hero_id = 'C15' }
  assert(roster.source_ref(provenance) == 'history_nobu@' .. sources.commit .. ':C15')
  for _, bad in ipairs({ { repository = 'https://github.com/x/other', commit = sources.commit, source_hero_id = 'C15' },
    { repository = provenance.repository, commit = 'abc', source_hero_id = 'C15' },
    { repository = provenance.repository, commit = sources.commit, source_hero_id = 'tamako' } }) do
    assert(not pcall(roster.source_ref, bad))
  end
  local authored = generate.load(path.read(path.join(content_root, 'templates', 'authored_heroes.lua'), true))
  for _, hero in ipairs(authored.heroes) do assert(hero.source_ref, hero.id .. ' lacks source_ref') end
  for pkg, update in pairs(authored.hero_updates) do assert(update.source_ref, pkg .. ' update lacks source_ref') end
end)

print(count .. ' hero source roster tests passed')
