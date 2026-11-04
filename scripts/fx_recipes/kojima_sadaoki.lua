-- B27 kojima_sadaoki: gold axe and cracked earth; red for the horsemanship surge.
local GOLD, RED, SLOW = { 1.0, 0.55, 0.08 }, { 1.0, 0.2, 0.1 }, { 0.08, 0.3, 1.0 }
return {
  recipes = {
    -- b27_horsemanship: dust kicked up as he surges, then red motes while it lasts
    b27w_vo = { system = "burst", sprite = "smoke_sheet", size = 340, life = 0.5,
      ground = "shock_ring", ground_size = 420, ground_fixed = true, ground_seconds = 0.4, grow = 0.3, tint = { 0.62, 0.46, 0.3 }, alpha = 0.55 },
    b27w_armlet_glow = { system = "stream", sprite = "spark_star", size = 90, life = 0.4, loop = true, tint = RED },
    -- b27_mighty_strike: throw swing, spinning axe in flight, chop on the target
    b27e_vo = { system = "pop", sprite = "crescent_slash", size = 340, life = 0.26, tint = GOLD },
    b27e_axe = { face = true, mesh = "throw_axe", ground_size = 110, ground_seconds = 20, height = 110, spin = 1080 },
    b27e_hit_sound = { system = "burst", sprite = "crescent_slash", size = 340, life = 0.4, tint = GOLD,
      ground = "dust_crack", ground_size = 400, ground_seconds = 0.45, grow = 0.4, also = { "fx_layer_stone_shards" } },
    -- b27_fearless_charge: fire at his feet; a shield thrust out before him; gold motes for the duration
    b27r_vo = { ground = "banner_rally", ground_size = 240, ground_fixed = true, ground_seconds = 0.8, grow = 0.4 },
    b27r_model_grow = { mesh = "tower_shield", ground_size = 160, ground_seconds = 0.9, grow = 0.6, lead = 55, launch = 85, face_blend = 0.5 },
    b27r_b15w3_aura = { system = "stream", sprite = "spark_star", size = 85, life = 0.4, loop = true, tint = GOLD },
    -- b27_rending_blade: a giant blade comes down, the earth breaks across his whole reach
    b27t_vo = { system = "pop", sprite = "crescent_slash", size = 480, life = 0.3, tint = GOLD,
      ground = "crack_gold", ground_size = 1500, area = true, ground_seconds = 1.3, grow = 0.25 },
    b27t_slam = { system = "burst", sprite = "rock", size = 120, fixed_size = true, life = 0.5,
      mesh = "fissure_blade", ground_size = 300, ground_seconds = 1.1, grow = 0.2, face_viewer = true },
    b27t_screen_shake = { system = "burst", sprite = "spark_star", size = 220, life = 0.35, tint = GOLD,
      ground = "shock_ring", ground_size = 300, ground_seconds = 0.4, grow = 0.3 },
    b27t_move_slow = { ground = "slow_chains", ground_size = 170, ground_seconds = 1.2, tint = SLOW },
  },
  notes = {
    b27_horsemanship = "b27w_vo cast burst (dust) + b27w_armlet_glow red stream for the buff.",
    b27_mighty_strike = "b27e_vo swing, b27e_axe flight, b27e_hit_sound burst. b27e_stun has no cue, so the stun itself shows nothing after the hit.",
    b27_fearless_charge = "b27r_vo fire burst + b27r_model_grow shield; aura stream on b27r_b15w3_aura. The grow ground follows the cue duration, so it stays if the status is dispelled. Blocking enemy debuffs is correct behaviour.",
    b27_rending_blade = "b27t_vo giant blade + b27t_slam dust burst. b27t_slam has no radius arg: ground_size 1800 is an art fallback; adding radius to the package cue would make it exact. b27t_title has no at, no row. b27t_guard has no cue.",
  },
}
