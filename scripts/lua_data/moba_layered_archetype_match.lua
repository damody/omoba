-- The same formal ten-seat roster on the compiled layered three-lane map.
-- Inherit hero/role policies; only author the map choice, not native code.
local source = debug.getinfo(1, 'S').source:sub(2)
local dir = assert(source:match('^(.*)[/\\]'))
local plan = assert(loadfile(dir .. '/moba_archetype_match.lua'))()
plan.map_id = 'three_lane_layered_training'
return plan
