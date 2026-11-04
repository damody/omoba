-- A26 濃姬: gunpowder orange. Flame and smoke keep their own colours (no tint). Status cues draw only their system.
local ORANGE = { 1.0, 0.55, 0.08 }
return {
  recipes = {
    -- a26_volley: four shots through Mx_Pistols_Volley's clockwise turn (0.2 s apart, front, right, back, left, right pistol first), then a blast on each enemy
    a26w_flak_sound = { ground = "pistol_shot", ground_size = 190, ground_fixed = true, ground_seconds = 0.22, launch = 480, from_hand = true, delay = 0.2,
      face_camera = 0.85, repeat_count = 4, repeat_seconds = 0.2, repeat_yaw = 90 },
    a26w_hit_explosion = { system = "burst", sprite = "flame_sheet", size = 230, life = 0.4,
      ground = "dust_crack", ground_size = 240, ground_seconds = 0.35, grow = 0.5 },
    -- a26_powder_kick: powder trailing Mx_Spin_Kick's turn (0.7 s, counter-clockwise), then the blast
    a26e_voice = { ground = "powder_swirl", ground_size = 340, ground_fixed = true, ground_seconds = 0.7, height = 45, delay = 0.1 },
    a26e_rocket_sound = { system = "pop", sprite = "smoke_sheet", size = 120, fixed_size = true, life = 0.7,
      ground = "powder_blast", ground_size = 600, area = true, ground_seconds = 0.5, grow = 0.35, also = { "fx_layer_powder_sparks" } },
    a26e_smoke = { system = "sparks", tint = ORANGE },
    -- a26_twin_guns: on a crit one flame leaves the pistol that fired. The cue comes at the windup's start; the shot is 0.2 s on, less with her attack speed. Its image has its own colours.
    a26r_crit_gesture = { ground = "pistol_flare", ground_size = 120, ground_fixed = true, ground_seconds = 0.22, lead = 30, launch = 100, from_hand = true, delay = 0.15, grow = 0.6, face_camera = 0.85 },
    -- a26_scorching_bomb
    a26t_voice = { system = "pop", sprite = "spark_star", size = 280, life = 0.45, tint = ORANGE,
      ground = "arcane_circle", ground_size = 340, ground_seconds = 0.5, grow = 0.4 },
    a26t_bomb_arcana = { system = "stream", sprite = "flame_sheet", size = 60, life = 0.3, loop = true, face = true,
      mesh = "fire_bomb", ground_size = 90, ground_seconds = 20, height = 110, spin = 360 },
    a26t_small_bomb = { system = "stream", sprite = "flame_sheet", size = 35, life = 0.2, loop = true, face = true,
      mesh = "fire_bomb", ground_size = 70, ground_seconds = 20, height = 90, spin = 360 },
    a26t_overhead_bomb = { system = "stream", sprite = "spark_star", size = 50, life = 0.3, loop = true, tint = ORANGE },
    a26t_big_explosion = { system = "burst", sprite = "flame_sheet", size = 700, life = 0.7,
      ground = "dust_crack", ground_size = 800, ground_seconds = 0.8, grow = 0.25 },
    a26t_small_explosion = { system = "burst", sprite = "rock", size = 130, life = 0.35, tint = ORANGE,
      ground = "dust_crack", ground_size = 260, ground_seconds = 0.35, grow = 0.4 },
  },
  notes = {
    a26_volley = "a26w_flak_sound at her always plays; a26w_hit_explosion on each enemy.",
    a26_powder_kick = "a26e_rocket_sound always plays; a26e_smoke draws a line per enemy; a26e_stun has no cue.",
    a26_twin_guns = "only a26r_crit_gesture, on crits; the attack speed itself has no cue.",
    a26_scorching_bomb = "a26t_bomb_arcana flight, a26t_big_explosion on landing; passive a26t_small_bomb + a26t_small_explosion; a26t_overhead_bomb moved from a flat disc to sparks on her. a26t_stun has no cue. spell_hint is shared, left alone. missing_image: bomb (rock stands in).",
  },
}
