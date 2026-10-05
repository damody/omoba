-- Authoring coordinates are integer world units; Rust generates fixed-point routes.
-- Prototype rule: the base opens only after all three lane towers are destroyed.
return function(ctx)
local maps = {
  {
    id = "three_lane_training",
    lane_length = 2400,
    tower_offset = 700,
    base_unlock = "all_lane_towers",
    -- Keep authored lane corridors and entire camp leashes clear until NPC
    -- obstacle detours are integrated. Hero movement already sweeps these walls.
    terrain = {
      { id = "north_island", min = {1150, 1200}, max = {1250, 1250} },
      { id = "south_island", min = {1150, -1250}, max = {1250, -1200} },
    },
    jungle_camps = {
      { id = "north_guardian", position = {800, 700}, hp = 450, damage = 30,
        move_speed = 260, attack_range = 130, leash_radius = 450,
        attack_interval_seconds = 1, respawn_seconds = 15, gold = 60, xp = 90 },
      { id = "south_guardian", position = {1600, -700}, hp = 450, damage = 30,
        move_speed = 260, attack_range = 130, leash_radius = 450,
        attack_interval_seconds = 1, respawn_seconds = 15, gold = 60, xp = 90 },
    },
    lanes = {
      { id = "top", waypoints = { {0, 0}, {400, 1400}, {2000, 1400}, {2400, 0} } },
      { id = "mid", waypoints = { {0, 0}, {2400, 0} } },
      { id = "bottom", waypoints = { {0, 0}, {400, -1400}, {2000, -1400}, {2400, 0} } },
    },
  },
}
local layered = {}
for key, value in pairs(maps[1]) do layered[key] = value end
layered.id = "three_lane_layered_training"
layered.tower_layers = {1000, 700, 400}
maps[#maps + 1] = layered
return maps
end
