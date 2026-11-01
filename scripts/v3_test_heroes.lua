-- v3 test heroes: a hero whose package so far is only its schema v3 document (v3.lua) enters the
-- content as a test_only hero, so headless tests and line-conformance can spawn it and run its
-- generated Rust (scripts/heroes/<package>). Writes scripts/lua_data/templates/v3_test_heroes.lua.
--
--   tools\lua\lua.exe scripts\v3_test_heroes.lua [--check]
--
-- Every number comes from the v3 document (levels, extras) or the source roster (unit stats,
-- through `source_stats`, templates/hero_source_stats.lua). A hero leaves this list when
-- hero_author builds its full package into authored_heroes.lua.
--
-- `playable = true`: every ability of the document is generated Rust and line-conformance shows
-- 0 fails, so the hero is in the game. Its row names the roster hero by `source_ref` (the unit
-- stats come through the same roster) instead of being test_only, and scripts/hero_dlls.lua lists
-- its crate so every launch builds and stages its DLL. The build refuses a playable hero that has
-- an ability the generator does not produce, or a crate hero_dlls does not list.
package.path = "scripts/?.lua;tools/lua/?.lua;" .. package.path
local v3 = require("hero_author.v3")
local cli = require("hero_author.cli")
local serialize = require("hero_source.serialize")

local OUT = "scripts/lua_data/templates/v3_test_heroes.lua"
-- package: docs/character_pipeline/packages/<package>/v3.lua and scripts/heroes/<package>
local HEROES = {
  { package = "hattori_hanzo", playable = true },
  -- M6 batch 2 (line-conformance REPORT_ONLY)
  { package = "maeda_keiji", playable = true },
  { package = "tachibana_ginchiyo", playable = true },
  { package = "inaba_ittetsu", playable = true },
  { package = "kakizaki_kageie", playable = true },
  { package = "torii_mototada", playable = true },
  { package = "sakuma_morimasa", playable = true },
  { package = "akechi_mitsuhide", playable = true },
  { package = "ukita_hideie", playable = true },
  { package = "shima_sakon", playable = true },
  -- M6 batch 1 (line-conformance REPORT_ONLY), plus A05
  { package = "tokugawa_ieyasu", playable = true },
  { package = "yamagata_masakage", playable = true },
  { package = "okuma_tomohide", playable = true },
  { package = "sasaki_kojiro", playable = true },
  { package = "otani_yoshitsugu", playable = true },
  { package = "maeda_toshiie", playable = true },
  { package = "nohime", playable = true },
  { package = "kojima_sadaoki", playable = true },
  { package = "shimazu_yoshihiro", playable = true },
  { package = "sakai_tadatsugu", playable = true },
  -- M6 batches 2-9, the rest (line-conformance REPORT_ONLY)
  { package = "kuroda_yoshitaka", playable = true },
  { package = "niwa_nagahide", playable = true },
  { package = "baba_nobufusa", playable = true },
  { package = "saito_yoshitatsu", playable = true },
  { package = "yagyu_muneyoshi", playable = true },
  { package = "sengoku_hidehisa", playable = true },
  { package = "takeda_shingen", playable = true },
  { package = "izumo_no_okuni", playable = true },
  { package = "chosokabe_motochika", playable = true },
  { package = "chacha", playable = true },
  { package = "ii_naomasa", playable = true },
  { package = "maeno_nagayasu", playable = true },
  { package = "takeda_katsuyori", playable = true },
  { package = "fuma_kotaro", playable = true },
  { package = "kirigakure_saizo" },
  { package = "akechi_hidemitsu", playable = true },
  { package = "kuki_yoshitaka", playable = true },
  { package = "katakura_kagetsuna", playable = true },
  { package = "sassa_narimasa", playable = true },
  { package = "shibata_katsuie", playable = true },
  { package = "honda_masanobu", playable = true },
  { package = "saito_tomonobu", playable = true },
  { package = "oda_nobunaga", playable = true },
  { package = "koihime", playable = true },
  { package = "mori_motonari", playable = true },
  { package = "usami_sadamitsu", playable = true },
  { package = "ryuzoji_takanobu", playable = true },
  { package = "hirate_masahide", playable = true },
  { package = "takigawa_kazumasu", playable = true },
  { package = "ishida_mitsunari", playable = true },
  { package = "miyoshi_nagayoshi", playable = true },
  { package = "kashin_koji", playable = true },
  { package = "kosokabe_chikayasu", playable = true },
  { package = "tachibana_muneshige", playable = true },
  { package = "kamiizumi_nobutsuna", playable = true },
  { package = "honganji_kennyo", playable = true },
  { package = "ishikawa_kazumasa", playable = true },
  { package = "otomo_sorin", playable = true },
  { package = "naoe_kanetsugu", playable = true },
  { package = "nene", playable = true },
  { package = "mori_ranmaru", playable = true },
  { package = "uesugi_kenshin", playable = true },
  { package = "kato_kiyomasa", playable = true },
  { package = "sakakibara_yasumasa", playable = true },
  { package = "miyamoto_musashi", playable = true },
  { package = "yamamoto_kansuke", playable = true },
  { package = "matsuhime", playable = true },
  { package = "takenaka_shigeharu", playable = true },
  { package = "momochi_sandayu", playable = true },
  { package = "hachisuka_masakatsu", playable = true },
  { package = "hashiba_hideyoshi", playable = true },
  { package = "sanada_masayuki", playable = true },
  { package = "kikuhime", playable = true },
  { package = "ise_hime", playable = true },
  { package = "matsunaga_hisahide", playable = true },
  { package = "mogami_yoshihime", playable = true },
  { package = "taigen_sessai", playable = true },
  { package = "tachibana_dosetsu", playable = true },
  { package = "saito_dosan", playable = true },
  { package = "zuiryuin_nisshu", playable = true },
  { package = "imagawa_yoshimoto", playable = true },
  { package = "ashikaga_yoshiaki" },
  { package = "mochizuki_chiyome", playable = true },
  { package = "kato_danzo", playable = true },
  { package = "kobayakawa_takakage", playable = true },
  { package = "akiyama_nobutomo" },
  { package = "ishikawa_goemon", playable = true },
}

-- v3 hero slots in the MOBA loadout order (Q W E R, then the D slot)
local SLOT_INDEX = { Q = 1, W = 2, E = 3, R = 4, D = 5, F = 6 }
-- omoba-core AbilityLevelData fields; the levels carry them, not extras (hero_author/v3.lua RESERVED)
local RESERVED = v3.RESERVED

local function load_doc(package)
  local p = "docs/character_pipeline/packages/" .. package .. "/v3.lua"
  local doc = assert(loadfile(p, "t", cli.safe_env()))
  doc = doc()
  if type(doc) == "function" then doc = doc({}) end
  return doc
end

local function has(list, v)
  for _, x in ipairs(list or {}) do if x == v then return true end end
  return false
end

local function ability_row(package, ab, compiled)
  return v3.ability_row(package, ab, compiled, { ue = { native_only = true } })
end

-- The roster (templates/hero_sources.lua): `source_ref` of a playable hero.
local function roster()
  local chunk = assert(loadfile("scripts/lua_data/templates/hero_sources.lua", "t", {}))
  return chunk()({})
end

-- A playable hero's every ability is generated (v3_rust.lua --hero, the build.rs generator) and
-- its crate is a hero DLL (scripts/hero_dlls.lua).
local function check_playable(entry, doc)
  V3_RUST_AS_MODULE = true
  local rust = dofile("docs/character_pipeline/tools/v3_rust.lua")
  V3_RUST_AS_MODULE = nil
  local picked
  for _, h in ipairs(rust.heroes(".", "all")) do if h.package == entry.package then picked = h end end
  assert(picked, entry.package .. ": no v3 document for v3_rust.lua")
  local reports, problems = rust.run(".", "scripts", { picked }, { write = false })
  assert(#problems == 0 and reports[1], entry.package .. ": v3_rust.lua: " .. tostring(problems[1]))
  local blockers = {}
  for _, e in ipairs(reports[1].entries) do blockers[e.id] = e.blockers end
  for _, ab in ipairs(doc.abilities) do
    local b = blockers[ab.id]
    assert(b and #b == 0, entry.package .. ": playable, but " .. ab.id .. " is not generated Rust"
      .. (b and (" (" .. table.concat(b, ", ") .. ")") or ""))
  end
  local listed = false
  for _, row in ipairs(dofile("scripts/hero_dlls.lua")) do
    if row.hero == entry.package and row.dir == "heroes/" .. entry.package then listed = true end
  end
  assert(listed, entry.package .. ": playable, but scripts/hero_dlls.lua does not list heroes/" .. entry.package)
end

local function build(entry)
  local doc = load_doc(entry.package)
  if entry.playable then check_playable(entry, doc) end
  local compiled = v3.compile(".", doc)
  local abilities, slots, ranks = {}, {}, {}
  for _, ab in ipairs(doc.abilities) do
    local row = ability_row(entry.package, ab, compiled)
    abilities[#abilities + 1] = row
    local idx = SLOT_INDEX[ab.slot]
    if idx then
      assert(not slots[idx], entry.package .. ": two abilities in slot " .. ab.slot)
      slots[idx] = ab.id
      -- birth rank: auto-learned 1, ultimates 0 (learned at their hero level), basics 1; 0 when
      -- rank 1 needs a higher hero level (B15D at 8), as omoba-content-model validates (E345)
      local later = (row.levels[1].required_hero_level or 1) > 1
      ranks[idx] = not later and (has(doc.hero.auto_learn, ab.id) or not has(ab.tags, "ultimate")) and 1 or 0
    end
  end
  -- an item the document defines (not a shop item: C06 item_c06e) has no catalog row of its own,
  -- so the ability that gives it (give_item) carries its statuses in its v3 block: the host's
  -- dispel classes, tags and immunities come from there (omoba-core dispel.rs)
  local function gives(t, id)
    if type(t) ~= "table" then return false end
    if t.op == "give_item" and t.item == id then return true end
    for _, v in pairs(t) do if gives(v, id) then return true end end
    return false
  end
  for _, it in ipairs(doc.items or {}) do
    local block = (compiled.items or {})[it.id]
    for i, ab in ipairs(doc.abilities) do
      if block and block.statuses and gives(ab, it.id) then
        local v3b = abilities[i].v3
        local have = {}
        for _, st in ipairs(v3b.statuses or {}) do have[st.id] = true end
        v3b.statuses = v3b.statuses or {}
        for _, st in ipairs(block.statuses) do if not have[st.id] then v3b.statuses[#v3b.statuses + 1] = st end end
      end
    end
  end
  -- the host's slots 1..6 (Q W E R D F, line B f04f942d): an empty slot below the highest filled
  -- one is "" with rank 0 (an F without a D, line B cddcb1af); the row never ends in one
  local top = 0
  for i = 1, 6 do if slots[i] then top = i end end
  local list, loadout = {}, {}
  for i = 1, top do
    list[i] = slots[i] or ""
    loadout[i] = slots[i] and ranks[i] or 0
  end
  local name = doc.hero.text.zh_TW.name
  -- a playable hero's select title and background come from its document (hero.text.zh_TW)
  local title, background = "v3 測試英雄", name .. "（schema v3 產生的 Rust；測試用）"
  if entry.playable then title, background = doc.hero.text.zh_TW.title, doc.hero.text.zh_TW.background end
  local hero = {
    id = entry.package,
    -- Not selectable or in a match: tests and conformance only, until it is playable.
    test_only = not entry.playable or nil,
    source_stats = not entry.playable and doc.hero.source_id or nil,
    source_ref = entry.playable and (SOURCES.name .. "@" .. SOURCES.commit .. ":" .. doc.hero.source_id) or nil,
    authored_package = entry.package,
    -- its abilities register through scripts/heroes/<package> (gen_hero_registry.lua skips it)
    v3_crate = entry.package,
    ue = { native_only = true },
    display_name = name,
    title = title,
    background = background,
    abilities = list,
    turn_speed = 720.0,
    moba_loadout = { ranks = loadout, skill_points = 0 },
  }
  -- the items the document defines (C06 item_c06e): they reach the item catalog as rows of their
  -- own (templates/moba_items.lua), so the hero's generated item entry registers and a give_item
  -- finds them; never sold
  local items = {}
  for _, it in ipairs(doc.items or {}) do
    local cost = it.values and it.values.cost and it.values.cost[1] or 0
    local zh = it.text and it.text.zh_TW
    items[#items + 1] = { id = it.id, name = zh and zh.name or it.id, cost = cost, package = entry.package }
  end
  return hero, abilities, items
end

SOURCES = roster()

local function render()
  local data = { heroes = {}, abilities = {}, hero_items = {} }
  for _, entry in ipairs(HEROES) do
    local hero, abilities, items = build(entry)
    data.heroes[#data.heroes + 1] = hero
    for _, a in ipairs(abilities) do data.abilities[#data.abilities + 1] = a end
    for _, it in ipairs(items) do data.hero_items[#data.hero_items + 1] = it end
  end
  table.sort(data.hero_items, function(a, b) return a.id < b.id end)
  local header = "-- GENERATED by scripts/v3_test_heroes.lua from docs/character_pipeline/packages/*/v3.lua; do not edit.\n" ..
    "-- test_only heroes for headless tests and conformance (their generated Rust is scripts/heroes/<package>).\n"
  local body = serialize.serialize(data, header)
  return (body:gsub("^(.-)return ", "%1local data = ", 1)) .. "return function(ctx) return data end\n"
end

local check = arg[1] == "--check"
local text = render()
local f = io.open(OUT, "rb")
local old = f and f:read("a"); if f then f:close() end
-- keep the file's line endings (land_to_live may write CRLF); compare without them
if old and old:find("\r\n", 1, true) then text = text:gsub("\r?\n", "\r\n") end
if check then
  if old ~= text then io.stderr:write(OUT .. " is out of date: run tools\\lua\\lua.exe scripts\\v3_test_heroes.lua\n"); os.exit(1) end
  print("ok " .. OUT)
elseif old ~= text then
  f = assert(io.open(OUT, "wb")); f:write(text); f:close(); print("wrote " .. OUT)
else
  print("unchanged " .. OUT)
end
