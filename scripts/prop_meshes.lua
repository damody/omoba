-- Small world props built from the engine's basic shapes instead of a generated 3D model (main session,
-- 2026-10-30: a tiny prop does not spend Tripo credits). Each prop is a list of parts: an engine shape
-- (/Engine/BasicShapes Sphere, Cylinder, Cone, Cube), its transform in cm (rotation pitch, yaw, roll in
-- degrees) and a material slot; each slot is one opaque colour. OmPropMeshCommandlet merges the parts
-- into /Game/Omoba/Props/<id>/SM_<id>. The engine shapes are 100 cm across and centred on their pivot.
--
--   tools\lua\lua.exe scripts\prop_meshes.lua --spec-out FILE.json
--   UnrealEditor-Cmd <project> -run=OmPropMesh -spec=FILE.json
local M = {}

M.MATERIALS = {
  { slot = "cloth", color = { 0.30, 0.17, 0.07 }, roughness = 0.95 },
  { slot = "rope", color = { 0.10, 0.07, 0.04 }, roughness = 0.9 },
  { slot = "gold", color = { 1.00, 0.70, 0.12 }, metallic = 1.0, roughness = 0.3 },
}

M.PROPS = {
  -- C06T 豪奪's ground gold (presentation identity item_money, 原作 models/props_gameplay/gold_bag):
  -- a plump burlap sack tied at the neck, coins at its open mouth and two beside it, as in the Grok
  -- reference omfue/SourceArt/Structures/TripoStaticV1/gold_bag/reference/gold_bag_front.jpg.
  gold_bag = {
    { shape = "Sphere", slot = "cloth", location = { 0, 0, 38 }, scale = { 0.85, 0.85, 0.76 } },
    { shape = "Cylinder", slot = "cloth", location = { 0, 0, 76 }, scale = { 0.34, 0.34, 0.16 } },
    { shape = "Cone", slot = "cloth", location = { 0, 0, 90 }, scale = { 0.42, 0.42, 0.18 }, rotation = { 180, 0, 0 } },
    { shape = "Cylinder", slot = "rope", location = { 0, 0, 72 }, scale = { 0.37, 0.37, 0.05 } },
    { shape = "Cylinder", slot = "gold", location = { 0, 0, 97 }, scale = { 0.3, 0.3, 0.05 } },
    { shape = "Cylinder", slot = "gold", location = { 48, 26, 2.5 }, scale = { 0.26, 0.26, 0.05 } },
    { shape = "Cylinder", slot = "gold", location = { 40, -18, 7 }, scale = { 0.26, 0.26, 0.05 }, rotation = { 0, 0, 20 } },
  },
}

M.SHAPES = { Sphere = true, Cylinder = true, Cone = true, Cube = true }

-- The commandlet spec: props in id order, every part checked.
function M.spec()
  local slots = {}
  for _, m in ipairs(M.MATERIALS) do slots[m.slot] = true end
  local ids = {}
  for id in pairs(M.PROPS) do ids[#ids + 1] = id end
  table.sort(ids)
  local props = {}
  for _, id in ipairs(ids) do
    assert(id:match("^[a-z][a-z0-9_]*$"), "prop id: " .. id)
    for i, part in ipairs(M.PROPS[id]) do
      assert(M.SHAPES[part.shape], id .. " part " .. i .. ": unknown shape " .. tostring(part.shape))
      assert(slots[part.slot], id .. " part " .. i .. ": unknown slot " .. tostring(part.slot))
      for _, v in ipairs(part.scale) do assert(v > 0, id .. " part " .. i .. ": scale must be positive") end
    end
    props[#props + 1] = { id = id, parts = M.PROPS[id] }
  end
  return { schema = "omoba.prop-mesh-spec.v1", materials = M.MATERIALS, props = props }
end

-- The asset path of a prop's mesh (world_units.lua uses it).
function M.asset(id) return "/Game/Omoba/Props/" .. id .. "/SM_" .. id .. ".SM_" .. id end

function M.main(args)
  local out
  local i = 1
  while i <= #args do
    if args[i] == "--spec-out" then i = i + 1; out = args[i] else error("unknown argument: " .. tostring(args[i])) end
    i = i + 1
  end
  assert(out, "usage: prop_meshes.lua --spec-out FILE.json")
  package.path = "scripts/?.lua;" .. package.path
  local b = require("_bootstrap")
  b.lib("json").write(out, M.spec(), true)
  print("prop spec: " .. out)
end

if not PROP_MESHES_AS_MODULE and arg and arg[0] and arg[0]:match("prop_meshes%.lua$") then M.main(arg) end
return M
