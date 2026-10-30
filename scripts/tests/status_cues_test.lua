STATUS_CUES_AS_MODULE = true
package.path = "scripts/?.lua;" .. package.path
local M = require("status_cues")
local checks = 0
local function ok(cond, name) assert(cond, name); checks = checks + 1 end

local t, ambiguous = M.table_of({
  { name = "hero", value = { abilities = {
    { id = "q", statuses = {
      { id = "q_glow", cue = "q_glow_fx" },
      { id = "q_guard", runtime_id = "q_guard:{caster}", cue = "q_guard_fx" },
      { id = "q_plain" },
      { id = "q_orb", runtime_id = "attack_orb:q", cue = "orb_fx" },
      { id = "shared", cue = "hero_fx" },
    } },
  } } },
  { name = "items/x.lua", value = { items = {
    { id = "item_sake", statuses = { { id = "sake_haste", runtime_id = "Passive_sake", cue = "sake_glow" } } },
    { id = "item_other", statuses = { { id = "shared", cue = "item_fx" } } },
    { id = "item_root", statuses = { { id = "net", cue = "net_fx" } } },
    { id = "item_root2", statuses = { { id = "net" } } },
  } } },
})
ok(t.q_glow == "q_glow_fx", "a status by its id")
ok(t.q_guard == "q_guard_fx" and t["q_guard:{caster}"] == nil, "a per-source status by its id, not its runtime id")
ok(t.sake_haste == "sake_glow" and t.Passive_sake == nil, "an item status by its id")
ok(t.shared == nil and t.net == nil, "an id with different cues shows none")
ok(#ambiguous == 2 and ambiguous[1] == "net" and ambiguous[2] == "shared", "ambiguous ids are listed")
ok(t.q_plain == nil and t.q_orb == nil, "no cue, or an attack orb, no row")
print("status_cues_test: " .. checks .. " checks passed")
