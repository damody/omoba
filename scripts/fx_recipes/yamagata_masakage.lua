-- B20 山縣昌景: rocks and shattered ground, earth orange; the red regiment ultimate in red.
local EARTH, RED = { 1.0, 0.18, 0.015 }, { 0.9, 0.03, 0.01 }
return {
  recipes = {
    -- Q 岩衝: rocks torn up at the cast, then ten rocks flying. A cue with at/to moves only its ground image,
    -- the system stays at the caster, so the flight body is the flying rock and the system just the throw.
    b20w_pick_sound = { system = "pop", sprite = "ramp_hit", size = 130, life = 0.3, tint = EARTH,
      ground = "ramp_crack", ground_size = 280, ground_seconds = 0.6, grow = 0.4 },
    b20w_rock = { system = "sparks", tint = EARTH, mesh = "rock_chunk", ground_size = 42, ground_seconds = 0.7, height = 70, fly = true, spin = 540, scatter = 55, scatter_size = 0.6 },
    -- W 壞毀波: a dust column and cracked ground, rocks and a shockwave over the area
    b20e_burrow = { system = "pop", sprite = "ramp_hit", size = 150, fixed_size = true, life = 0.3, tint = EARTH,
      ground = "ramp_crack", ground_size = 420, ground_seconds = 0.8, grow = 0.4 },
    b20e_thunderclap = { system = "sparks", tint = EARTH,
      mesh = "earth_crown", ground_size = 270, ground_seconds = 0.7, grow = 0.4 },
    -- R 赤備猛攻: every half second rocks burst outward (leaving the model visible) and red cracks fill the radius
    b20t_epicenter = { system = "pop", sprite = "ramp_hit", size = 130, fixed_size = true, life = 0.3, tint = RED,
      ground = "ramp_crack_deep", ground_size = 640, area = true, ground_fixed = true, ground_seconds = 0.6, grow = 0.5 },
  },
  notes = {
    b20_rock_rush = "body on b20w_pick_sound (throw) and b20w_rock (flight); no landing cue, so rocks hit without a burst and the point shows nothing; b20w_stun has no cue either",
    b20_ruin_wave = "body on b20e_burrow and b20e_thunderclap; b20e_stun has no cue",
    b20_tenacity = "no cue: the passive plays nothing, so it cannot show a visual without a package change",
    b20_red_charge = "body on b20t_epicenter; b20t_onslaught has no cue of its own; shared move_slow left alone (ground only)",
  },
}
