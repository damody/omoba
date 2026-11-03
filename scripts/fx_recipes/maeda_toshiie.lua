-- A14 前田利家: gold spear, blue lightning for the fury. Status cues draw only their system, so they carry no ground.
local GOLD, BLUE = { 1.0, 0.55, 0.08 }, { 0.12, 0.5, 1.0 }
return {
  recipes = {
    -- a14_impact_spear: the landing crash
    a14w_impact = { system = "burst", sprite = "spark_star", size = 420, life = 0.45, tint = GOLD,
      ground = "dust_crack", ground_size = 700, ground_seconds = 0.6, grow = 0.25 },
    a14w_move_slow = { system = "stream", sprite = "slow_chains", size = 90, life = 0.25, loop = true, tint = BLUE },
    -- a14_divine_strike: a bolt from him to the target; the burst is attached, so it goes with him through the blink
    a14e_blink_sparkles = { system = "burst", sprite = "blink_flash", size = 440, life = 0.4, tint = GOLD,
      ground = "lightning", ground_size = 340, ground_seconds = 0.8, line = true },
    -- Wings over the struck target: his blow keeps the star.
    a14e_hit = { system = "pop", sprite = "fire_wings", size = 340, life = 0.5, tint = GOLD,
      ground = "dust_crack", ground_size = 300, ground_seconds = 0.6, grow = 0.3 },
    -- He arrives far away: the landing marks itself.
    a14e_arrive = { system = "burst", sprite = "blink_flash", size = 260, life = 0.35, tint = GOLD,
      ground = "shock_ring", ground_size = 420, ground_seconds = 0.5, grow = 0.2 },
    -- a14_spear_fury: thunder at the cast, then lightning on him while it lasts
    a14t_vo = { system = "burst", sprite = "spark_star", size = 480, life = 0.5, tint = BLUE,
      ground = "shock_ring", ground_size = 900, ground_seconds = 0.55, grow = 0.15 },
    a14t_title = { system = "pop", sprite = "lightning", size = 900, life = 0.45, tint = BLUE,
      ground = "shock_ring", ground_size = 620, ground_seconds = 0.7, grow = 0.3 },
    a14t_lightning = { system = "stream", sprite = "lightning", size = 130, life = 0.22, loop = true, tint = BLUE },
  },
  notes = {
    a14_impact_spear = "no cue at the cast: nothing shows until a14w_impact, up to 1.87 s later where the target lands; needs a thrust cue in the package. missing_image: spear_thrust. a14w_stun has no cue.",
    a14_divine_strike = "a14e_blink_sparkles carries it, a14e_hit marks the struck target; a14e_arrive marks the landing; a14e_stun has no cue.",
    a14_stacked_spear = "plays no cue at all: only his normal fx_swing / fx_strike show.",
    a14_spear_fury = "a14t_vo + a14t_title at the cast, a14t_lightning for the duration.",
  },
}
