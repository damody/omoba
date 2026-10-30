ITEM_CUES_AS_MODULE = true
package.path = "scripts/?.lua;" .. package.path
local M = require("item_cues")
local checks = 0
local function ok(cond, name) assert(cond, name); checks = checks + 1 end

local t = M.table_of({
  { id = "item_fire_scroll", active = { events = { on_cast = {
    { op = "cue", id = "fire_scroll_cast", at = "caster" },
    { op = "for_each", actions = { { op = "cue", id = "per_unit", at = "it" } } },
    { op = "if", actions = { { op = "cue", id = "nested", at = "target" } } },
    { op = "cue", id = "ground", at = "point" },
    { op = "cue", id = "loose", at = "event.target" },
  } } } },
  { id = "item_plain" },
  { id = "item_passive_only", passive = { triggers = { on_attack_hit = { { op = "cue", id = "x", at = "caster" } } } } },
})
local rows = t.fire_scroll
ok(rows and #rows == 3, "caster, nested target and point cues; a loop's and an event's are left out")
ok(rows[1].cue == "fire_scroll_cast" and rows[1].at == "caster", "in document order, by place")
ok(rows[2].cue == "nested" and rows[2].at == "target", "a cue inside an if counts")
ok(rows[3].cue == "ground" and rows[3].at == "target", "a point cue plays at the target")
ok(t.plain == nil and t.passive_only == nil, "no active, no row")
print("item_cues_test: " .. checks .. " checks passed")
