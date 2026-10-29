-- Site generator (scripts/build_site.lua): pages against the catalog, no stray numbers,
-- patch notes, content hash and the catalog JSON round trip.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local json = b.lib('json')
local path = b.lib('path')
local B = require('build_site')
local A = require('site.census_adapter')
local CL = require('site.changelog')
local CHECK = require('site.check')
local P = require('site.pages')

local checks = 0
local function ok(cond, msg) checks = checks + 1; if not cond then error(msg, 2) end end
local function find(list, id) for _, x in ipairs(list) do if x.id == id then return x end end end
local function deep_copy(t)
  if type(t) ~= 'table' then return t end
  local c = {}
  for k, v in pairs(t) do c[k] = deep_copy(v) end
  return setmetatable(c, getmetatable(t))
end

-- 1. the twelve v3 samples (M0: ten heroes, two item sets) build without errors; every page
-- passes the page checks. Documents are found at their authoring locations (v3_sources.lua).
local sources = dofile(path.join(b.root, 'docs', 'character_pipeline', 'tools', 'v3_sources.lua'))
local SAMPLES = {}
for _, h in ipairs({ 'A11', 'A13', 'A21', 'B01', 'B11', 'B22', 'C01', 'C06', 'C07', 'C17' }) do
  SAMPLES[#SAMPLES + 1] = assert(sources.hero_path(b.root, h), 'no document for ' .. h)
end
SAMPLES[#SAMPLES + 1] = path.join(b.root, sources.ITEMS, 'pilot_a.lua')
SAMPLES[#SAMPLES + 1] = path.join(b.root, sources.ITEMS, 'pilot_b.lua')
-- Schema validation is line A's test (package_v3_test.lua); it keeps tightening while the samples
-- catch up, so the site test renders them without it. build_site itself always validates.
local r = B.build({ docs = SAMPLES, validate = false })
ok(#r.errors == 0, 'build errors:\n' .. table.concat(r.errors, '\n'))
local cat = r.catalog
ok(#cat.heroes == 10, 'the ten sample heroes')
ok(#cat.items >= 25, 'the 25 pilot items (C06 adds its own item)')
for _, id in ipairs({ 'A11', 'A13', 'A21', 'B01', 'B11', 'B22', 'C01', 'C06', 'C07', 'C17' }) do
  ok(find(cat.heroes, id), 'pilot hero ' .. id .. ' is on the site')
end
local per_lang = {}
for _, p in ipairs(r.pages) do per_lang[p.lang] = (per_lang[p.lang] or 0) + 1 end
ok(per_lang.zh_TW == per_lang.en and per_lang.en == 3 + #cat.heroes + #cat.items, 'one page set per language')
local spans = 0
for _, p in ipairs(r.pages) do
  local errs, counts = CHECK.page(p.html, r.context, nil, p.hero)
  ok(#errs == 0, p.path .. ': ' .. table.concat(errs, '; '))
  spans = spans + counts.values
end
ok(spans > 1000, 'pages show their numbers through value spans')

-- 2. the checker catches a number that does not come from a value, and a wrong value
local c17 = assert((function() for _, p in ipairs(r.pages) do if p.path == 'zh-TW/heroes/C17.html' then return p end end end)())
local stray = c17.html:gsub('</main>', '<p>多出 12 傷害</p></main>')
ok(#CHECK.page(stray, r.context, nil, c17.hero) == 1, 'stray digit is reported')
local wrong, n = c17.html:gsub('(data%-v="c17_light_chain/cooldown"[^>]*>)13<', '%114<')
ok(n > 0 and #CHECK.page(wrong, r.context, nil, c17.hero) == n, 'every changed number is reported')
local fullwidth = c17.html:gsub('</main>', '<p>１</p></main>')
ok(#CHECK.page(fullwidth, r.context, nil, c17.hero) == 1, 'full-width digit is reported')

-- 3. descriptions: literal text of every template has no digits (only placeholders produce numbers)
for _, kind in ipairs({ 'abilities', 'items', 'statuses', 'unit_templates' }) do
  for _, e in ipairs(cat[kind]) do
    for lang, fields in pairs(e.text or {}) do
      for field, pieces in pairs(fields) do
        for _, p in ipairs(pieces) do
          if p.lit then ok(not p.lit:find('%d') and not p.lit:find('\239\188[\144-\153]'), e.id .. '.' .. lang .. '.' .. field .. ' has a digit') end
        end
      end
    end
  end
end

-- 4. values compile exactly: ranks, pct as ratio, Fixed64 raw
local V = r.context.values
local dmg = V.c17_light_chain.damage
ok(table.concat(dmg.exact, ',') == '350,520,690,860' and dmg.raw[1] == 350 * 1024, 'C17 damage ranks and raw')
ok(V.c17_light_chain.guard.exact[1] == '-30' and V.c17_light_chain.guard.raw[1] == -30 * 1024, 'pct is the displayed percent number')
ok(V.c17_light_chain.cast_point.raw[1] == 102, '0.1 s rounds to raw 102')
for owner, list in pairs(cat.values) do
  for _, v in ipairs(list) do
    if v.exact then
      ok(#v.exact == #v.raw, owner .. '.' .. v.name .. ' raw per rank')
    end
  end
end
-- the page shows the value the rank selects
ok(c17.html:find('data%-v="c17_light_chain/damage" data%-f="all" data%-r="2">350/<b>520</b>/690/860<'), 'rank 2 panel bolds the second rank')

-- 5. English pages carry the English names and the same numbers
local c17en = assert((function() for _, p in ipairs(r.pages) do if p.path == 'en/heroes/C17.html' then return p end end end)())
ok(c17en.html:find('<h1>Oichi</h1>', 1, true), 'English hero name')
ok(c17en.html:find('data-v="c17_light_chain/damage" data-f="all" data-r="">350/520/690/860<', 1, true), 'same numbers in English')
ok(c17en.html:find('lang="en"', 1, true), 'html lang')

-- 6. content hash: deterministic, covers values and text
local h1 = B.hash(cat)
ok(h1 == cat.content_hash and #h1 == 64, 'hash is stable')
local changed = deep_copy(cat)
local cv = (function() for _, v in ipairs(changed.values.c17_light_chain) do if v.name == 'cooldown' then return v end end end)()
cv.exact = { '12' }; cv.raw = { 12 * 1024 }
ok(B.hash(changed) ~= h1, 'a value change changes the hash')
local changed_text = deep_copy(cat)
find(changed_text.abilities, 'c17_light_chain').text.en.name = { { lit = 'Renamed' } }
ok(B.hash(changed_text) ~= h1, 'a text change changes the hash')
for _, p in ipairs(r.pages) do
  ok(p.html:find(h1, 1, true), p.path .. ' shows the content hash')
end

-- 7. patch notes from two catalogs
local entries = CL.diff(cat, changed)
ok(#entries == 1 and entries[1].owner == 'c17_light_chain' and entries[1].changes[1].name == 'cooldown', 'diff finds the cooldown change')
ok(#CL.diff(cat, deep_copy(cat)) == 0, 'no changes between equal catalogs')
local r2 = B.build({ catalog = changed, previous_catalog = nil })
ok(#r2.errors == 0, 'changed catalog builds')
local Cold, Cnew = P.context(cat), P.context(changed)
Cnew.catalog.content_hash = B.hash(changed)
local notes = CL.page(Cold, Cnew, 'zh_TW', 'zh-TW')
local html = P.layout(Cnew, notes)
ok(html:find('data%-v="c17_light_chain/cooldown" data%-f="all" data%-r="" data%-c="old">13<'), 'old value shown')
ok(html:find('data%-v="c17_light_chain/cooldown" data%-f="all" data%-r="" data%-c="new">12<'), 'new value shown')
local errs = CHECK.page(html, Cnew, Cold, nil)
ok(#errs == 0, 'patch notes page checks: ' .. table.concat(errs, '; '))

-- 8. catalog.json round trip: --catalog builds the same site
local tmp = path.join(b.root, 'target', 'site-test')
-- auto_previous = false: the round trip compares one build with itself, not with the last test run
local built = B.build({ docs = SAMPLES, validate = false, out = tmp, icons = false, auto_previous = false })
ok(#built.errors == 0, 'build to disk')
ok(path.is_file(tmp .. '/catalog.json') and path.is_file(tmp .. '/zh-TW/heroes/C17.html') and path.is_file(tmp .. '/en/items/index.html'), 'files written')
local again = B.build({ catalog_file = tmp .. '/catalog.json' })
ok(#again.errors == 0, 'catalog.json builds: ' .. table.concat(again.errors, '; '))
ok(again.catalog.content_hash == built.catalog.content_hash, 'catalog.json keeps the content hash')
local same = #again.pages == #built.pages
for i, p in ipairs(again.pages) do if p.html ~= built.pages[i].html then same = false end end
ok(same, 'catalog.json renders the same pages')
local notes_built = B.build({ catalog_file = tmp .. '/catalog.json', previous_file = tmp .. '/catalog.json' })
ok(#notes_built.errors == 0, 'patch notes against itself build')

-- 9. the compiled catalog's v3 export (M1 format) builds the same kind of site, with its own hash
local m1 = B.build({ catalog_file = path.join(b.root, 'scripts/tests/fixtures/v3-content-catalog-sample.json'), icons = false })
ok(#m1.errors == 0, 'v3 export builds: ' .. table.concat(m1.errors, '; '))
ok(m1.catalog.content_hash == 'd6221e8c4427cc2a', 'site shows the content catalog hash')
local oichi = (function() for _, p in ipairs(m1.pages) do if p.path == 'zh-TW/heroes/oichi.html' then return p end end end)()
ok(oichi and oichi.html:find('data-v="c17_light_chain/guard" data-f="abs" data-r="">30/40/50/60%<', 1, true), 'pct from the catalog shows one % at the end')
ok(oichi.html:find('d6221e8c4427cc2a', 1, true), 'footer hash')

-- 10. M8 full site: the source catalog (every original hero and item) with the game catalog's values
-- for shipped heroes; unshipped heroes are marked, unported items listed, both hashes in the footer
local function fixture(name) return json.read(path.join(b.root, 'scripts/tests/fixtures', name)) end
local Reader = require('site.catalog_reader')
local full = B.build({ catalog = Reader.read_source(fixture('source-catalog-sample.json'), fixture('v3-content-catalog-sample.json'), nil), icons = false })
ok(#full.errors == 0, 'source catalog builds: ' .. table.concat(full.errors, '; '))
local function page(p) for _, x in ipairs(full.pages) do if x.path == p then return x end end end
local c17, a24 = page('zh-TW/heroes/C17.html'), page('zh-TW/heroes/A24.html')
ok(c17 and a24 and page('en/heroes/B01.html'), 'one page per original hero, shipped or not')
ok(c17.html:find('數值為遊戲現行版本', 1, true) and not c17.html:find('未上架', 1, true), 'a shipped hero shows the game values')
ok(a24.html:find('badge warn">未上架', 1, true), 'an unshipped hero is marked 未上架')
local home = page('zh-TW/index.html')
ok(home.html:find('<h2>已上架</h2>', 1, true) and home.html:find('<h2>未上架</h2>', 1, true), 'home groups shipped and unshipped heroes')
local shop = page('zh-TW/items/index.html')
ok(shop.html:find('原作有、尚未移植的道具', 1, true) and shop.html:find('not_in_source', 1, true), 'unported items are listed with their reason')
ok(not page('zh-TW/items/item_ancient_doll.html'), 'an unported item has no item page')
ok(c17.html:find('fixture0000000000', 1, true) and c17.html:find(fixture('v3-content-catalog-sample.json').content_data_hash, 1, true),
  'footer shows the game content hash and the source catalog hash')
ok(not full.missing, 'every value the text refers to is in the catalog (document values come from owners)')
ok(page('zh-TW/items/item_dark_boots.html').html:find('class="tree"', 1, true), 'recipes from the source catalog build a tree')
local gap_src = fixture('source-catalog-sample.json')
gap_src.owners = nil
local gap = B.build({ catalog = Reader.read_source(gap_src, nil, nil), icons = false })
ok(gap.missing and gap.missing['A24.shared/hp_gate'], 'a value the text needs but the catalog lacks is reported')
local gap_a24 = (function() for _, x in ipairs(gap.pages) do if x.path == 'zh-TW/heroes/A24.html' then return x end end end)()
ok(gap_a24.html:find('class="missing-value"', 1, true), '... and shown as a gap, never a number')
ok(#CHECK.page(gap_a24.html, gap.context, nil, gap_a24.hero) == 0, '... which the page check accepts')
for _, p in ipairs(full.pages) do
  local errs = CHECK.page(p.html, full.context, nil, p.hero)
  ok(#errs == 0, p.path .. ': ' .. table.concat(errs, '; '))
end

-- game items are listed by game id; the source catalog matches them through v3.source_id
local game = fixture('v3-content-catalog-sample.json')
local src = fixture('source-catalog-sample.json')
for _, it in ipairs(src.items) do
  if it.id == 'item_brave_ring' then
    game.items = { { id = 'brave_ring', catalog_id = 1, v3 = it.v3 } }
    game.items[1].v3.source_id = 'item_brave_ring'
  end
end
local shelf = B.build({ catalog = Reader.read_source(src, game, nil), icons = false })
local function shelf_page(p) for _, x in ipairs(shelf.pages) do if x.path == p then return x end end end
ok(not shelf_page('zh-TW/items/item_brave_ring.html').html:find('未上架', 1, true), 'a shelf item matched by v3.source_id is in the game')
ok(shelf_page('zh-TW/items/item_ghost_mask.html').html:find('badge warn">未上架', 1, true), 'an item the game lacks is 未上架')

-- 11. adapter rejects a document that does not validate
local bad = path.join(tmp, 'bad.lua')
path.write(bad, 'return function(ctx) return { schema_version = 3, kind = "hero", id = "X" } end\n', true)
local _, berrs = A.build({ bad })
ok(#berrs > 0, 'invalid document is an error')

print(string.format('site_build_test: %d pages, %d value spans, %d checks passed', #r.pages, spans, checks))
