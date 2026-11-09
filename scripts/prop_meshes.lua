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
  { slot = "frog", color = { 0.20, 0.55, 0.12 }, roughness = 0.7 },
  { slot = "eye", color = { 0.95, 0.95, 0.90 }, roughness = 0.5 },
  { slot = "pupil", color = { 0.03, 0.03, 0.03 }, roughness = 0.5 },
  { slot = "horse", color = { 0.20, 0.08, 0.03 }, roughness = 0.8 },
  { slot = "mane", color = { 0.07, 0.05, 0.04 }, roughness = 0.9 },
  { slot = "hoof", color = { 0.12, 0.10, 0.08 }, roughness = 0.7 },
  { slot = "saddle_cloth", color = { 0.03, 0.05, 0.28 }, roughness = 0.9 },
  { slot = "leather", color = { 0.07, 0.03, 0.01 }, roughness = 0.6 },
}

-- Parts written in cm at life size, then scaled about the origin (the game sizes a prop by its height).
local function scaled(k, parts)
  for _, part in ipairs(parts) do
    for i = 1, 3 do
      if part.location then part.location[i] = part.location[i] * k end
      part.scale[i] = part.scale[i] * k
    end
  end
  return parts
end

M.PROPS = {
  -- The courier (原作 npc_dota_courier2 「馬」, courier-spec-2026-11-09 §1): a chestnut horse facing +X with
  -- an indigo saddle cloth and two burlap saddlebags, so it reads as the item carrier from the game camera.
  -- Cylinders run along Z; a negative pitch leans their top toward +X. Half size so the review capture frames it.
  courier_horse = scaled(0.5, {
    { shape = "Sphere", slot = "horse", location = { 0, 0, 82 }, scale = { 1.4, 0.6, 0.55 } },
    { shape = "Sphere", slot = "horse", location = { 48, 0, 88 }, scale = { 0.65, 0.56, 0.6 } },
    { shape = "Sphere", slot = "horse", location = { -50, 0, 90 }, scale = { 0.65, 0.58, 0.6 } },
    { shape = "Cylinder", slot = "horse", location = { 80, 0, 118 }, scale = { 0.36, 0.3, 0.55 }, rotation = { -50, 0, 0 } },
    { shape = "Sphere", slot = "horse", location = { 112, 0, 132 }, scale = { 0.6, 0.3, 0.32 }, rotation = { -40, 0, 0 } },
    { shape = "Sphere", slot = "mane", location = { 132, 0, 118 }, scale = { 0.28, 0.24, 0.22 } },
    { shape = "Cone", slot = "horse", location = { 104, 8, 150 }, scale = { 0.09, 0.07, 0.16 } },
    { shape = "Cone", slot = "horse", location = { 104, -8, 150 }, scale = { 0.09, 0.07, 0.16 } },
    { shape = "Sphere", slot = "pupil", location = { 118, 14, 140 }, scale = { 0.06, 0.06, 0.06 } },
    { shape = "Sphere", slot = "pupil", location = { 118, -14, 140 }, scale = { 0.06, 0.06, 0.06 } },
    { shape = "Cube", slot = "mane", location = { 72, 0, 128 }, scale = { 0.6, 0.06, 0.1 }, rotation = { 40, 0, 0 } },
    { shape = "Cylinder", slot = "mane", location = { -88, 0, 78 }, scale = { 0.1, 0.1, 0.42 }, rotation = { -23, 0, 0 } },
    { shape = "Cylinder", slot = "horse", location = { 46, 18, 33 }, scale = { 0.15, 0.15, 0.5 } },
    { shape = "Cylinder", slot = "horse", location = { 46, -18, 33 }, scale = { 0.15, 0.15, 0.5 } },
    { shape = "Cylinder", slot = "horse", location = { -46, 18, 33 }, scale = { 0.15, 0.15, 0.5 } },
    { shape = "Cylinder", slot = "horse", location = { -46, -18, 33 }, scale = { 0.15, 0.15, 0.5 } },
    { shape = "Cylinder", slot = "hoof", location = { 46, 18, 4 }, scale = { 0.18, 0.18, 0.08 } },
    { shape = "Cylinder", slot = "hoof", location = { 46, -18, 4 }, scale = { 0.18, 0.18, 0.08 } },
    { shape = "Cylinder", slot = "hoof", location = { -46, 18, 4 }, scale = { 0.18, 0.18, 0.08 } },
    { shape = "Cylinder", slot = "hoof", location = { -46, -18, 4 }, scale = { 0.18, 0.18, 0.08 } },
    { shape = "Sphere", slot = "saddle_cloth", location = { -2, 0, 104 }, scale = { 0.55, 0.64, 0.14 } },
    { shape = "Sphere", slot = "leather", location = { -2, 0, 111 }, scale = { 0.36, 0.32, 0.1 } },
    { shape = "Cylinder", slot = "leather", location = { 14, 0, 116 }, scale = { 0.05, 0.05, 0.1 } },
    { shape = "Sphere", slot = "cloth", location = { -16, 32, 90 }, scale = { 0.24, 0.12, 0.22 } },
    { shape = "Sphere", slot = "cloth", location = { -16, -32, 90 }, scale = { 0.24, 0.12, 0.22 } },
  }),
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
  -- The hex form (v3 state hex, OmStatusLook::ShowsHexForm): any hexed unit is shown as this squat frog,
  -- facing +X like the units, with big eyes that read from the game camera.
  hex_critter = {
    { shape = "Sphere", slot = "frog", location = { 0, 0, 22 }, scale = { 0.75, 0.62, 0.44 } },
    { shape = "Sphere", slot = "frog", location = { 22, 0, 36 }, scale = { 0.48, 0.5, 0.34 } },
    { shape = "Sphere", slot = "eye", location = { 30, 13, 50 }, scale = { 0.17, 0.17, 0.17 } },
    { shape = "Sphere", slot = "eye", location = { 30, -13, 50 }, scale = { 0.17, 0.17, 0.17 } },
    { shape = "Sphere", slot = "pupil", location = { 37, 13, 51 }, scale = { 0.07, 0.07, 0.07 } },
    { shape = "Sphere", slot = "pupil", location = { 37, -13, 51 }, scale = { 0.07, 0.07, 0.07 } },
    { shape = "Sphere", slot = "frog", location = { -18, 26, 8 }, scale = { 0.34, 0.2, 0.16 } },
    { shape = "Sphere", slot = "frog", location = { -18, -26, 8 }, scale = { 0.34, 0.2, 0.16 } },
    { shape = "Sphere", slot = "frog", location = { 26, 20, 5 }, scale = { 0.16, 0.12, 0.1 } },
    { shape = "Sphere", slot = "frog", location = { 26, -20, 5 }, scale = { 0.16, 0.12, 0.1 } },
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
