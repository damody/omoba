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
  { package = "hattori_hanzo" },
  -- M6 batch 2 (line-conformance REPORT_ONLY)
  { package = "maeda_keiji", playable = true },
  { package = "tachibana_ginchiyo" },
  { package = "inaba_ittetsu", playable = true },
  { package = "kakizaki_kageie", playable = true },
  { package = "torii_mototada", playable = true },
  { package = "sakuma_morimasa", playable = true },
  { package = "akechi_mitsuhide", playable = true },
  { package = "ukita_hideie", playable = true },
  { package = "shima_sakon", playable = true },
  -- M6 batch 1 (line-conformance REPORT_ONLY), plus A05
  { package = "tokugawa_ieyasu", playable = true },
  { package = "yamagata_masakage" },
  { package = "okuma_tomohide", playable = true },
  { package = "sasaki_kojiro", playable = true },
  { package = "otani_yoshitsugu", playable = true },
  { package = "maeda_toshiie" },
  { package = "nohime", playable = true },
  { package = "kojima_sadaoki", playable = true },
  { package = "shimazu_yoshihiro" },
  { package = "sakai_tadatsugu", playable = true },
  -- M6 batches 2-9, the rest (line-conformance REPORT_ONLY)
  { package = "kuroda_yoshitaka", playable = true },
  { package = "niwa_nagahide" },
  { package = "baba_nobufusa" },
  { package = "saito_yoshitatsu", playable = true },
  { package = "yagyu_muneyoshi", playable = true },
  { package = "sengoku_hidehisa", playable = true },
  { package = "takeda_shingen" },
  { package = "izumo_no_okuni" },
  { package = "chosokabe_motochika", playable = true },
  { package = "chacha" },
  { package = "ii_naomasa" },
  { package = "maeno_nagayasu" },
  { package = "takeda_katsuyori" },
  { package = "fuma_kotaro" },
  { package = "kirigakure_saizo" },
  { package = "akechi_hidemitsu", playable = true },
  { package = "kuki_yoshitaka" },
  { package = "katakura_kagetsuna", playable = true },
  { package = "sassa_narimasa" },
  { package = "shibata_katsuie" },
  { package = "honda_masanobu" },
  { package = "saito_tomonobu", playable = true },
  { package = "oda_nobunaga", playable = true },
  { package = "koihime", playable = true },
  { package = "mori_motonari" },
  { package = "usami_sadamitsu", playable = true },
  { package = "ryuzoji_takanobu" },
  { package = "hirate_masahide" },
  { package = "takigawa_kazumasu", playable = true },
  { package = "ishida_mitsunari" },
  { package = "miyoshi_nagayoshi", playable = true },
  { package = "kashin_koji" },
  { package = "kosokabe_chikayasu" },
  { package = "tachibana_muneshige", playable = true },
  { package = "kamiizumi_nobutsuna", playable = true },
  { package = "honganji_kennyo" },
  { package = "ishikawa_kazumasa" },
  { package = "otomo_sorin" },
  { package = "naoe_kanetsugu", playable = true },
  { package = "nene", playable = true },
  { package = "mori_ranmaru" },
  { package = "uesugi_kenshin", playable = true },
  { package = "kato_kiyomasa" },
  { package = "sakakibara_yasumasa" },
  { package = "miyamoto_musashi", playable = true },
  { package = "yamamoto_kansuke", playable = true },
  { package = "matsuhime" },
  { package = "takenaka_shigeharu" },
  { package = "momochi_sandayu", playable = true },
  { package = "hachisuka_masakatsu" },
  { package = "hashiba_hideyoshi" },
  { package = "sanada_masayuki" },
  { package = "kikuhime" },
  { package = "ise_hime" },
  { package = "matsunaga_hisahide" },
  { package = "mogami_yoshihime", playable = true },
  { package = "taigen_sessai", playable = true },
  { package = "tachibana_dosetsu", playable = true },
  { package = "saito_dosan" },
  { package = "zuiryuin_nisshu" },
  { package = "imagawa_yoshimoto" },
  { package = "ashikaga_yoshiaki" },
  { package = "mochizuki_chiyome" },
  { package = "kato_danzo" },
  { package = "kobayakawa_takakage" },
  { package = "akiyama_nobutomo" },
  { package = "ishikawa_goemon" },
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
  return hero, abilities
end

SOURCES = roster()

local function render()
  local data = { heroes = {}, abilities = {} }
  for _, entry in ipairs(HEROES) do
    local hero, abilities = build(entry)
    data.heroes[#data.heroes + 1] = hero
    for _, a in ipairs(abilities) do data.abilities[#data.abilities + 1] = a end
  end
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
