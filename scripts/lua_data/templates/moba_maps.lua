-- Authoring coordinates are integer world units; Rust generates fixed-point routes.
-- Prototype rule: the base opens only after all three lane towers are destroyed.
return function(ctx)
return {
  {
    id = "three_lane_training",
    lane_length = 2400,
    tower_offset = 700,
    base_unlock = "all_lane_towers",
    lanes = {
      { id = "top", waypoints = { {0, 0}, {400, 1400}, {2000, 1400}, {2400, 0} } },
      { id = "mid", waypoints = { {0, 0}, {2400, 0} } },
      { id = "bottom", waypoints = { {0, 0}, {400, -1400}, {2000, -1400}, {2400, 0} } },
    },
  },
}
end
