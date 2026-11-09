local p = arg[1]
local f = io.open(p, "rb"); local s = f:read("a"); f:close()
local nl = s:find("\r\n", 1, true) and "\r\n" or "\n"
local function rep(old, new)
  old, new = old:gsub("\n", nl), new:gsub("\n", nl)
  local a, b = s:find(old, 1, true); assert(a and not s:find(old, b + 1, true), "exactly once: " .. old:sub(1, 70))
  s = s:sub(1, a - 1) .. new .. s:sub(b + 1)
end
rep([[-- a direction is cast at a point like a vector (the same as generate.lua's TARGET_TYPE); unit_or_point
-- too: v3_rust.lua lowers it as a point cast (`cx.target_point()`, a unit aims at its position, E338)
-- a tree target is a point the host snaps to a standing tree's centre (targets_tree, line B E403)
local V3_TARGET_TYPE = { none = "none", unit = "unit", point = "point", direction = "point", vector = "point", unit_or_point = "point", tree = "point" }]],
[[-- a direction is cast at a point like a vector (the same as generate.lua's TARGET_TYPE); unit_or_point
-- as is (line B c38a574a: a unit clicked is checked as a unit target, anything else as a point;
-- v3_rust.lua reads `cx.target_point()`, a unit's position, E338)
-- a tree target is a point the host snaps to a standing tree's centre (targets_tree, line B E403)
local V3_TARGET_TYPE = { none = "none", unit = "unit", point = "point", direction = "point", vector = "point", unit_or_point = "unit_or_point", tree = "point" }
local TAKES_UNIT = { unit = true, unit_or_point = true }]])
rep([[    unit_target = target.type == "unit" and (target.team == "ally" or target.team == "enemy") and target.team or nil,]],
[[    unit_target = TAKES_UNIT[target.type] and (target.team == "ally" or target.team == "enemy") and target.team or nil,]])
rep([[    targets_heroes_only = target.type == "unit" and type(target.types) == "table" and #target.types == 1]],
[[    targets_heroes_only = TAKES_UNIT[target.type] and type(target.types) == "table" and #target.types == 1]])
f = io.open(p, "wb"); f:write(s); f:close()
