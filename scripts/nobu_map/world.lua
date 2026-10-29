-- World entities of a Hammer (Source 2) map: walks the CMapWorld node tree of a parsed .vmap,
-- accumulating each node's origin / angles / scales, and expands CMapPrefab nodes by loading the
-- prefab .vmap they point at (relative to the addon content root), recursively. Read-only.
--
--   local world = require("nobu_map.world")
--   local ents, info = world.entities(content_root, "maps/nobu.vmap")
--   -- ents[i] = { classname, origin = {x,y,z}, yaw, props = {...}, source = "maps/...vmap#nodeID",
--   --   via = { { name, path, origin, node } enclosing prefabs, outermost first } }
--   -- A prefab whose file is not in the addon (Dota's stock content) becomes one entity
--   -- `prefab:<path>` at its placement, with `unresolved_prefab` set.
local dmx = require("nobu_map.dmx")
local M = {}

local rad = math.pi / 180

-- 3x3 rotation from Source QAngle (pitch, yaw, roll) in degrees: R = Rz(yaw) Ry(pitch) Rx(roll)
local function rotation(a)
  local p, y, r = (a[1] or 0) * rad, (a[2] or 0) * rad, (a[3] or 0) * rad
  local sp, cp, sy, cy, sr, cr = math.sin(p), math.cos(p), math.sin(y), math.cos(y), math.sin(r), math.cos(r)
  return {
    { cp * cy, sr * sp * cy - cr * sy, cr * sp * cy + sr * sy },
    { cp * sy, sr * sp * sy + cr * cy, cr * sp * sy - sr * cy },
    { -sp, sr * cp, cr * cp },
  }
end
local function mul(a, b)
  local m = {}
  for i = 1, 3 do m[i] = {} for j = 1, 3 do m[i][j] = a[i][1] * b[1][j] + a[i][2] * b[2][j] + a[i][3] * b[3][j] end end
  return m
end
local function apply(m, v) return { m[1][1] * v[1] + m[1][2] * v[2] + m[1][3] * v[3], m[2][1] * v[1] + m[2][2] * v[2] + m[2][3] * v[3], m[3][1] * v[1] + m[3][2] * v[2] + m[3][3] * v[3] } end
local IDENTITY = { origin = { 0, 0, 0 }, rot = { { 1, 0, 0 }, { 0, 1, 0 }, { 0, 0, 1 } }, scale = { 1, 1, 1 }, yaw = 0 }

-- parent transform x local node -> world transform
local function compose(parent, node)
  local o = node.origin or { 0, 0, 0 }
  local s = node.scales or { 1, 1, 1 }
  local scaled = { o[1] * parent.scale[1], o[2] * parent.scale[2], o[3] * parent.scale[3] }
  local wo = apply(parent.rot, scaled)
  return {
    origin = { parent.origin[1] + wo[1], parent.origin[2] + wo[2], parent.origin[3] + wo[3] },
    rot = mul(parent.rot, rotation(node.angles or { 0, 0, 0 })),
    scale = { parent.scale[1] * s[1], parent.scale[2] * s[2], parent.scale[3] * s[3] },
    yaw = parent.yaw + ((node.angles and node.angles[2]) or 0),
  }
end

local cache = {}
local function load(content_root, rel)
  if cache[rel] == nil then
    local f = io.open(content_root .. "/" .. rel, "rb")
    if not f then cache[rel] = false else
      local d = f:read("a"); f:close()
      cache[rel] = dmx.parse(d)
    end
  end
  return cache[rel] or nil
end

local function props_of(el)
  local p = el.attrs.entity_properties
  local out = {}
  if type(p) == "table" and p.attrs then
    for k, v in pairs(p.attrs) do if type(v) ~= "table" then out[k] = v end end
  end
  return out
end

-- returns entities, info = { files = { rel = elements }, missing = { rel }, prefabs = n }
function M.entities(content_root, rel)
  local ents, info = {}, { files = {}, missing = {}, prefabs = 0, meshes = 0 }
  local function walk(doc, rel_path, el, parent, via, depth)
    assert(depth < 16, "prefab nesting too deep at " .. rel_path)
    local t = compose(parent, el.attrs)
    if el.type == "CMapEntity" then
      local props = props_of(el)
      ents[#ents + 1] = { classname = props.classname, origin = t.origin, yaw = t.yaw, props = props, via = via,
        source = rel_path .. "#" .. tostring(el.attrs.nodeID) }
    elseif el.type == "CMapMesh" then
      info.meshes = info.meshes + 1
    elseif el.type == "CMapPrefab" then
      info.prefabs = info.prefabs + 1
      local target = el.attrs.targetMapPath
      local sub = target and load(content_root, target)
      if not sub then
        info.missing[#info.missing + 1] = tostring(target)
        -- the prefab's own placement is still known (e.g. Dota's stock fountain prefabs)
        ents[#ents + 1] = { classname = "prefab:" .. tostring(target), origin = t.origin, yaw = t.yaw, props = {}, via = via,
          source = rel_path .. "#" .. tostring(el.attrs.nodeID), unresolved_prefab = target }
      else
        info.files[target] = #sub.elements
        local nvia = {}
        for i, v in ipairs(via) do nvia[i] = v end
        nvia[#nvia + 1] = { name = (el.attrs.targetName and el.attrs.targetName ~= "" and el.attrs.targetName or nil),
          path = target, origin = t.origin, node = rel_path .. "#" .. tostring(el.attrs.nodeID) }
        local w = sub.root.attrs.world
        if w then walk(sub, target, w, t, nvia, depth + 1) end
      end
    end
    for _, c in ipairs(el.attrs.children or {}) do
      if type(c) == "table" and c.attrs then walk(doc, rel_path, c, t, via, depth) end
    end
  end
  local doc = assert(load(content_root, rel), "cannot read " .. rel)
  info.files[rel] = #doc.elements
  info.root = doc
  walk(doc, rel, doc.root.attrs.world, IDENTITY, {}, 0)
  return ents, info
end

return M
