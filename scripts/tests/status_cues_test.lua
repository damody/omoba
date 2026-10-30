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
      { id = "q_rank", runtime_id = "q_rank_r{rank}:{caster}", cue = "q_rank_fx" },
      { id = "q_stun", runtime_id = "stun", cue = "q_stunned" },
      { id = "q_plain" },
      { id = "q_orb", runtime_id = "attack_orb:q", cue = "orb_fx" },
    } },
  } } },
  { name = "items/x.lua", value = { items = {
    { id = "item_sake", statuses = { { id = "sake_haste", runtime_id = "Passive_sake", cue = "sake_glow" } } },
    { id = "item_other", statuses = { { id = "other_stun", runtime_id = "stun", cue = "other_stunned" } } },
    { id = "item_root", statuses = { { id = "net", runtime_id = "root", cue = "net_fx" } } },
    { id = "item_root2", statuses = { { id = "net2", runtime_id = "root" } } },
  } } },
})
ok(t.q_glow == "q_glow_fx", "a status without runtime_id lives as its id")
ok(t["q_guard:*"] == "q_guard_fx", "a per-source buff is kept as its prefix")
ok(t["q_rank_r*"] == "q_rank_fx", "a rank-filled buff is kept as the prefix before its first placeholder")
ok(t.Passive_sake == "sake_glow", "an item status lives as its runtime id")
ok(t.stun == nil and t.root == nil, "a buff shared by statuses with different cues shows none")
ok(#ambiguous == 2 and ambiguous[1] == "root" and ambiguous[2] == "stun", "shared buffs are listed")
ok(t.q_plain == nil and t["attack_orb:q"] == nil, "no cue, or an attack orb, no row")
print("status_cues_test: " .. checks .. " checks passed")
