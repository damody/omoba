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
}, "\n"))
ok(seen.cue.h01_strike.effect == 2 and seen.cue.h01_strike.placeholder == 1 and seen.cue.h01_strike.cues.h01q_spark == 1, "cue plays counted by art and id")
ok(seen.cue.h01_ghost.unresolved == 1, "an unknown cue id is counted as unresolved")
ok(seen.target.h01_strike == 1, "target cues counted by cast")
ok(seen.status.h01w_guard.played == 2 and seen.status.h01w_guard.art == 1, "status cues counted by status id")
ok(seen.arcs == 1 and seen.peaks[1] == 180, "forced arcs counted")

local source_id, rows = M.expected("hero_one",
  { abilities = {
    { id = "h01_strike", hero = "H01" },
    { id = "h01_guard", hero = "H01", v3 = { statuses = { { id = "h01w_guard" }, { id = "h01w_secret", visibility = "hidden" } } } },
    { id = "h01_ghost", hero = "H01" },
    { id = "h01_idle", hero = "H01" },
    { id = "h02_other", hero = "H02" },
  } },
  { entries = { { kind = "hero", id = "hero_one", source_id = "H01" } } },
  { abilities = { h01_strike = { { cue = "h01q_slash", at = "caster" }, { cue = "h01q_hit", at = "target" } } } },
  { statuses = { h01w_guard = "h01w_ring" } })
ok(source_id == "H01" and #rows == 4, "only the hero's abilities, in catalog order")
ok(#rows[1].cast_cues == 1 and #rows[1].target_cues == 1, "expected cues split by place")
ok(#rows[2].statuses == 1 and rows[2].statuses[1].cue == "h01w_ring", "hidden statuses are left out")
local md, defects = M.render("hero_one", source_id, rows, seen)
ok(md:find("`h01_strike` | effect 2、placeholder 1 | 1（預期 1 個）", 1, true), "strike row")
ok(md:find("h01w_guard 2", 1, true), "status row")
ok(#defects == 2 and defects[1]:find("h01_ghost", 1, true) and defects[2]:find("h01_idle", 1, true), "unresolved cue and silent ability are defects")
ok(md:find("擊退拋物線（全局）：1 次，最高 180 cm", 1, true), "arc summary")
ok(not pcall(M.expected, "nobody", { abilities = {} }, { entries = {} }, {}, { statuses = {} }), "unknown hero refused")
print("ue_presentation_report_test: " .. checks .. " checks passed")
