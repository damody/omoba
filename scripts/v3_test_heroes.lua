-- v3 test heroes: a hero whose package so far is only its schema v3 document (v3.lua) enters the
-- content as a test_only hero, so headless tests and line-conformance can spawn it and run its
-- generated Rust (scripts/heroes/<package>). Writes scripts/lua_data/templates/v3_test_heroes.lua.
--
--   tools\lua\lua.exe scripts\v3_test_heroes.lua [--check]
--
-- Every number comes from the v3 document (levels, extras) or the source roster (unit stats,
-- through `source_stats`, templates/hero_source_stats.lua). A hero leaves this list when
-- hero_author builds its full package into authored_heroes.lua.
package.path = "scripts/?.lua;tools/lua/?.lua;" .. package.path
local v3 = require("hero_author.v3")
local cli = require("hero_author.cli")
local serialize = require("hero_source.serialize")

local OUT = "scripts/lua_data/templates/v3_test_heroes.lua"
-- package: docs/character_pipeline/packages/<package>/v3.lua and scripts/heroes/<package>
local HEROES = {
  { package = "hattori_hanzo" },
  -- M6 batch 2 (line-conformance REPORT_ONLY)
  { package = "maeda_keiji" },
  { package = "tachibana_ginchiyo" },
  { package = "inaba_ittetsu" },
  { package = "kakizaki_kageie" },
  { package = "torii_mototada" },
  { package = "sakuma_morimasa" },
  { package = "akechi_mitsuhide" },
  { package = "ukita_hideie" },
  { package = "shima_sakon" },
  -- M6 batch 1 (line-conformance REPORT_ONLY), plus A05
  { package = "tokugawa_ieyasu" },
  { package = "yamagata_masakage" },
  { package = "okuma_tomohide" },
  { package = "sasaki_kojiro" },
  { package = "otani_yoshitsugu" },
  { package = "maeda_toshiie" },
  { package = "nohime" },
  { package = "kojima_sadaoki" },
  { package = "shimazu_yoshihiro" },
  { package = "sakai_tadatsugu" },
  -- M6 batches 2-9, the rest (line-conformance REPORT_ONLY)
  { package = "kuroda_yoshitaka" },
  { package = "niwa_nagahide" },
  { package = "baba_nobufusa" },
  { package = "saito_yoshitatsu" },
  { package = "yagyu_muneyoshi" },
  { package = "sengoku_hidehisa" },
  { package = "takeda_shingen" },
  { package = "izumo_no_okuni" },
  { package = "chosokabe_motochika" },
  { package = "chacha" },
  { package = "ii_naomasa" },
  { package = "maeno_nagayasu" },
  { package = "takeda_katsuyori" },
  { package = "fuma_kotaro" },
  { package = "kirigakure_saizo" },
  { package = "akechi_hidemitsu" },
  { package = "kuki_yoshitaka" },
  { package = "katakura_kagetsuna" },
  { package = "sassa_narimasa" },
  { package = "shibata_katsuie" },
  { package = "honda_masanobu" },
  { package = "saito_tomonobu" },
  { package = "oda_nobunaga" },
  { package = "koihime" },
  { package = "mori_motonari" },
  { package = "usami_sadamitsu" },
  { package = "ryuzoji_takanobu" },
  { package = "hirate_masahide" },
  { package = "takigawa_kazumasu" },
  { package = "ishida_mitsunari" },
  { package = "miyoshi_nagayoshi" },
  { package = "kashin_koji" },
  { package = "kosokabe_chikayasu" },
  { package = "tachibana_muneshige" },
  { package = "kamiizumi_nobutsuna" },
  { package = "honganji_kennyo" },
  { package = "ishikawa_kazumasa" },
  { package = "otomo_sorin" },
  { package = "naoe_kanetsugu" },
  { package = "nene" },
  { package = "mori_ranmaru" },
  { package = "uesugi_kenshin" },
  { package = "kato_kiyomasa" },
  { package = "sakakibara_yasumasa" },
  { package = "miyamoto_musashi" },
  { package = "yamamoto_kansuke" },
  { package = "matsuhime" },
  { package = "takenaka_shigeharu" },
  { package = "momochi_sandayu" },
  { package = "hachisuka_masakatsu" },
  { package = "hashiba_hideyoshi" },
  { package = "sanada_masayuki" },
  { package = "kikuhime" },
  { package = "ise_hime" },
  { package = "matsunaga_hisahide" },
  { package = "mogami_yoshihime" },
  { package = "tachibana_dosetsu" },
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

local function build(entry)
  local doc = load_doc(entry.package)
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
  -- the host's slots 1..6 (Q W E R D F, line B f04f942d) hold no gap: a slot after an empty one
  -- (an F without a D) has no seat yet and is listed in the file header, not dropped unseen
  local list, loadout, unseated = {}, {}, {}
  for i = 1, 6 do
    if slots[i] then
      if i == #list + 1 then list[i] = slots[i]; loadout[i] = ranks[i]
      else unseated[#unseated + 1] = slots[i] end
    end
  end
  local name = doc.hero.text.zh_TW.name
  local hero = {
    id = entry.package,
    -- Not selectable or in a match: tests and conformance only, until hero_author builds its package.
    test_only = true,
    source_stats = doc.hero.source_id,
    authored_package = entry.package,
    -- its abilities register through scripts/heroes/<package> (gen_hero_registry.lua skips it)
    v3_crate = entry.package,
    ue = { native_only = true },
    display_name = name,
    title = "v3 測試英雄",
    background = name .. "（schema v3 產生的 Rust；測試用）",
    abilities = list,
    turn_speed = 720.0,
    moba_loadout = { ranks = loadout, skill_points = 0 },
  }
  return hero, abilities, unseated
end

local function render()
  local data, unseated = { heroes = {}, abilities = {} }, {}
  for _, entry in ipairs(HEROES) do
    local hero, abilities, off = build(entry)
    data.heroes[#data.heroes + 1] = hero
    for _, a in ipairs(abilities) do data.abilities[#data.abilities + 1] = a end
    for _, id in ipairs(off) do unseated[#unseated + 1] = entry.package .. ":" .. id end
  end
  local header = "-- GENERATED by scripts/v3_test_heroes.lua from docs/character_pipeline/packages/*/v3.lua; do not edit.\n" ..
    "-- test_only heroes for headless tests and conformance (their generated Rust is scripts/heroes/<package>).\n" ..
    (#unseated > 0 and ("-- Not seated (a slot after an empty one has no host form yet): " .. table.concat(unseated, ", ") .. "\n") or "")
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
