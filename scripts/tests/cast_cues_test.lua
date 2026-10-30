CAST_CUES_AS_MODULE = true
package.path = "scripts/?.lua;" .. package.path
local M = require("cast_cues")
local checks = 0
local function ok(cond, name) assert(cond, name); checks = checks + 1 end

local items = M.items_of({
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
local rows = items.fire_scroll
ok(rows and #rows == 3, "caster, nested target and point cues; a loop's and an event's are left out")
ok(rows[1].cue == "fire_scroll_cast" and rows[1].at == "caster", "in document order, by place")
ok(rows[2].cue == "nested" and rows[2].at == "target", "a cue inside an if counts")
ok(rows[3].cue == "ground" and rows[3].at == "target", "a point cue plays at the target")
ok(items.plain == nil and items.passive_only == nil, "no active, no row")

local abilities = M.abilities_of({
  { abilities = {
    { id = "c17_light_chain", events = { on_cast = { { op = "cue", id = "c17w_cast", at = "caster" }, { op = "cue", id = "c17w_hit", at = "target" } } } },
    { id = "c17_passive", events = { on_attack_hit = { { op = "cue", id = "x", at = "target" } } } },
    { id = "c17_silent" },
  } },
})
ok(#abilities.c17_light_chain == 2 and abilities.c17_light_chain[2].cue == "c17w_hit" and abilities.c17_light_chain[2].at == "target",
  "a hero ability's on_cast cues, caster and target")
ok(abilities.c17_passive == nil and abilities.c17_silent == nil, "only on_cast plays with the cast")
print("cast_cues_test: " .. checks .. " checks passed")
