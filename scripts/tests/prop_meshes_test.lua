PROP_MESHES_AS_MODULE = true
package.path = "scripts/?.lua;" .. package.path
local M = require("prop_meshes")
local checks = 0
local function ok(cond, name) assert(cond, name); checks = checks + 1 end

local spec = M.spec()
ok(spec.schema == "omoba.prop-mesh-spec.v1", "schema")
ok(#spec.props >= 1 and spec.props[1].id == "gold_bag", "the gold bag is a prop")
local slots = {}
for _, part in ipairs(spec.props[1].parts) do slots[part.slot] = true end
ok(slots.cloth and slots.gold, "the bag is cloth with gold coins")
ok(M.asset("gold_bag") == "/Game/Omoba/Props/gold_bag/SM_gold_bag.SM_gold_bag", "asset path")
M.PROPS.bad = { { shape = "Torus", slot = "cloth", scale = { 1, 1, 1 } } }
ok(not pcall(M.spec), "an unknown engine shape is refused")
M.PROPS.bad = { { shape = "Cube", slot = "paint", scale = { 1, 1, 1 } } }
ok(not pcall(M.spec), "an unknown slot is refused")
M.PROPS.bad = { { shape = "Cube", slot = "cloth", scale = { 1, 0, 1 } } }
ok(not pcall(M.spec), "a flat part is refused")
M.PROPS.bad = nil
print("prop_meshes_test: " .. checks .. " checks passed")
