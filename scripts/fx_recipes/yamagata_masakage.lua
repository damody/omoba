-- B20 山縣昌景: rocks and shattered ground, earth orange; the red regiment ultimate in red.
local EARTH, RED = { 1.0, 0.45, 0.1 }, { 1.0, 0.2, 0.1 }
return {
  recipes = {
    -- Q 岩衝: rocks torn up at the cast, then ten rocks flying. A cue with at/to moves only its ground image,
    -- the system stays at the caster, so the flight body is the flying rock and the system just the throw.
    b20w_pick_sound = { system = "burst", sprite = "rock", size = 260, life = 0.45, tint = EARTH,
      ground = "dust_crack", ground_size = 480, ground_seconds = 0.5, grow = 0.4 },
    b20w_rock = { system = "sparks", tint = EARTH, mesh = "rock_chunk", ground_size = 130, ground_seconds = 0.7, height = 80, fly = true, spin = 540 },
    -- W 壞毀波: a dust column and cracked ground, rocks and a shockwave over the area
    b20e_burrow = { system = "burst", sprite = "smoke_sheet", size = 380, life = 0.55, area = true,
      ground = "dust_crack", ground_size = 420, ground_seconds = 0.7, grow = 0.3 },
    b20e_thunderclap = { system = "burst", sprite = "rock", size = 460, life = 0.5, area = true, tint = { 1.0, 0.5, 0.15 },
      mesh = "earth_crown", ground_size = 420, ground_seconds = 0.9, grow = 0.3 },
    -- R 赤備猛攻: every half second rocks burst outward (leaving the model visible) and red cracks fill the radius
    b20t_epicenter = { system = "pop", sprite = "rock", size = 110, life = 0.3, tint = { 1.0, 0.2, 0.1 } },
  },
  notes = {
    b20_rock_rush = "body on b20w_pick_sound (throw) and b20w_rock (flight); no landing cue, so rocks hit without a burst and the point shows nothing; b20w_stun has no cue either",
    b20_ruin_wave = "body on b20e_burrow and b20e_thunderclap; b20e_stun has no cue",
    b20_tenacity = "no cue: the passive plays nothing, so it cannot show a visual without a package change",
    b20_red_charge = "body on b20t_epicenter; b20t_onslaught has no cue of its own; shared move_slow left alone (ground only)",
  },
}
