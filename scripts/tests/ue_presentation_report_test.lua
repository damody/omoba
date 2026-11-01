UE_PRESENTATION_REPORT_AS_MODULE = true
package.path = "scripts/?.lua;" .. package.path
local M = require("ue_presentation_report")
local checks = 0
local function ok(cond, name) assert(cond, name); checks = checks + 1 end

local seen = M.parse(table.concat({
  "[2026.10.30-10.00.00:000][  0]LogTemp: Display: OM_CUE ability=h01_strike cue=h01q_slash unit=12 art=effect",
  "LogTemp: Display: OM_CUE ability=h01_strike cue=h01q_slash unit=12 art=effect",
  "LogTemp: Display: OM_CUE ability=h01_strike cue=h01q_spark unit=12 art=placeholder",
  "LogTemp: Display: OM_CUE ability=h01_ghost cue=h01e_missing unit=12 art=unresolved",
  "LogTemp: Display: OM_CAST_TARGET_CUES cast=h01_strike anchor=40 played=1 of 1",
  "LogTemp: Display: OM_STATUS_CUE buff=h01w_guard cue=h01w_ring unit=12 art=1",
  "LogTemp: Display: OM_STATUS_CUE buff=h01w_guard cue=h01w_ring unit=12 art=0",
  "LogTemp: Display: OM_FORCED_ARC unit=40 peak_cm=180",
  "LogTemp: Display: OM_BUFF buff=h01w_guard unit=12 route=status_cue",
  "LogTemp: Display: OM_BUFF buff=h01w_guard unit=12 route=aura",
  "LogTemp: Display: OM_CUE ability=h01_guard cue=cast unit=12 art=style",
  "LogTemp: Display: OM_BUFF buff=stun unit=40 route=ring",
}, "\n"))
ok(seen.cue.h01_strike.effect == 2 and seen.cue.h01_strike.placeholder == 1 and seen.cue.h01_strike.cues.h01q_spark == 1, "cue plays counted by art and id")
ok(seen.cue.h01_ghost.unresolved == 1, "an unknown cue id is counted as unresolved")
ok(seen.target.h01_strike == 1, "target cues counted by cast")
ok(seen.status.h01w_guard.played == 2 and seen.status.h01w_guard.art == 1, "status cues counted by status id")
ok(seen.arcs == 1 and seen.peaks[1] == 180, "forced arcs counted")
ok(seen.buff.h01w_guard.status_cue == 1 and seen.buff.h01w_guard.aura == 1, "status instances counted by route")
ok(seen.cue.h01_guard.style == 1, "an event kind drawn by the ability style counts as shown")

local source_id, rows = M.expected("hero_one",
  { abilities = {
    { id = "h01_strike", hero = "H01" },
    { id = "h01_guard", hero = "H01", v3 = { statuses = { { id = "h01w_guard" }, { id = "h01w_secret", visibility = "hidden" } } } },
    { id = "h01_ghost", hero = "H01", v3 = { statuses = { { id = "h01e_stun", runtime_id = "stun" }, { id = "h01e_mark", runtime_id = "h01_ghost:{caster}" } } } },
    { id = "h01_idle", hero = "H01" },
    { id = "h01_passive", hero = "H01", v3 = { bot = { kind = "none", reason = "passive" } } },
    { id = "h02_other", hero = "H02" },
  } },
  { entries = { { kind = "hero", id = "hero_one", source_id = "H01" } } },
  { abilities = { h01_strike = { { cue = "h01q_slash", at = "caster" }, { cue = "h01q_hit", at = "target" } } } },
  { statuses = { h01w_guard = "h01w_ring" } })
ok(source_id == "H01" and #rows == 5, "only the hero's abilities, in catalog order")
ok(#rows[1].cast_cues == 1 and #rows[1].target_cues == 1, "expected cues split by place")
ok(#rows[2].statuses == 1 and rows[2].statuses[1].cue == "h01w_ring", "hidden statuses are left out")
local md, defects = M.render("hero_one", source_id, rows, seen)
ok(md:find("`h01_strike` | effect 2、placeholder 1 | 1（預期 1 個）", 1, true), "strike row")
ok(md:find("h01w_guard：aura 1、status_cue 1", 1, true), "status row lists routes")
ok(md:find("h01e_stun（stun）：ring 1", 1, true), "a shared runtime status is matched by its shared id")
ok(md:find("h01e_mark：0", 1, true), "a per-caster runtime id is not a shared id")
ok(#defects == 2 and defects[1]:find("h01_ghost", 1, true) and defects[2]:find("h01_idle", 1, true), "unresolved cue and silent ability are defects")
ok(md:find("擊退拋物線（全局）：1 次，最高 180 cm", 1, true), "arc summary")
ok(md:find("`h01_passive` | 0 | 0（預期 0 個） | — | 被動，無 cue／狀態可檢查", 1, true), "a passive with nothing to show is not a defect")
local items = M.expected_items({ "item_bomb" },
  { items = { { id = "item_bomb", v3 = { statuses = { { id = "bomb_burn" }, { id = "bomb_secret", visibility = "hidden" } } } } } },
  { abilities = {} }, { statuses = { bomb_burn = "burn_ring" } })
ok(#items == 1 and items[1].aliases[1] == "bomb" and items[1].aliases[2] == "item:bomb" and #items[1].statuses == 1, "an item row with its authority id and public statuses")
local item_md = M.render("道具", "item_bomb", items, M.parse("OM_CUE ability=bomb cue=cast unit=3 art=style\nOM_BUFF buff=bomb_burn unit=9 route=status_cue"))
ok(item_md:find("`item_bomb` | style 1", 1, true) and item_md:find("bomb_burn：status_cue 1", 1, true), "item cues found under the authority id")
ok(not pcall(M.expected_items, { "item_none" }, { items = {} }, {}, { statuses = {} }), "unknown item refused")
ok(not pcall(M.expected, "nobody", { abilities = {} }, { entries = {} }, {}, { statuses = {} }), "unknown hero refused")
-- --server-log tells a silent ability apart: never requested, cancelled before its script ran, or ran undrawn.
local server = M.parse_server("[INFO] - AbilityCast ok pid=1 slot=0 ability='h01_idle'   (x)\n[INFO] - AbilityCast ok pid=1 slot=1 ability='h01_ghost'   (x)\n"..
  "[INFO] - [scripting] cast ok ability='h01_ghost' caster=0 rank=1   (x)\n")
ok(M.cast_outcome(server, "h01_none") == "沒施放" and M.cast_outcome(server, "h01_idle") == "被取消"
  and M.cast_outcome(server, "h01_ghost") == "施放了但沒畫" and M.cast_outcome(nil, "h01_idle") == nil, "cast outcomes")
local md2, defects2 = M.render("hero_one", source_id, rows, seen, server)
ok(md2:find("本局沒有呈現（被取消）", 1, true) and defects2[2]:find("(被取消)", 1, true), "the verdict names the outcome")
print("ue_presentation_report_test: " .. checks .. " checks passed")
