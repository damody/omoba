-- C05 maeda_keiji: blue lightning; cyan wind for Matsukaze, red for the warning enemies see.
local BLUE, CYAN, WARN = { 0.08, 0.3, 1.0 }, { 0.05, 0.7, 1.0 }, { 1.0, 0.25, 0.2 }
return {
  recipes = {
    -- c05_matsukaze: a whirlwind on cast, cyan motes while it lasts
    c05w_vo = { system = "burst", sprite = "vortex", size = 320, life = 0.45, tint = CYAN,
      ground = "vortex", ground_size = 600, ground_seconds = 0.6, grow = 0.3 },
    c05w_armlet = { system = "stream", sprite = "spark_star", size = 85, life = 0.4, loop = true, tint = CYAN },
    -- c05_thunder_dance: lightning erupts around him, a bolt to each enemy, a burst on each hit
    c05e_cast_sound = { system = "burst", sprite = "lightning", size = 560, life = 0.4, tint = BLUE,
      ground = "shock_ring", ground_size = 1700, ground_seconds = 0.5, grow = 0.15 },
    c05e_bolt = { system = "stream", sprite = "lightning", size = 110, life = 0.22, loop = true, face = true, tint = BLUE,
      ground = "chain_bolt", ground_size = 230, ground_seconds = 20, height = 90 },
    c05e_impact = { system = "burst", sprite = "lightning", size = 270, life = 0.3, tint = BLUE,
      ground = "blink_flash", ground_size = 240, ground_seconds = 0.25, height = 20, grow = 0.4 },
    -- c05_thunder_spear: a bolt along the thrust and a flash on the spear, then the target crackles
    c05r_chain_lightning = { system = "pop", sprite = "lightning", size = 320, life = 0.22, tint = BLUE,
      ground = "chain_bolt", ground_size = 150, ground_seconds = 0.3, line = true },
    c05r_tempest = { system = "stream", sprite = "lightning", size = 110, life = 0.22, loop = true, tint = BLUE },
    -- c05_flash_thunder: charge under him, the landing mark, the strike
    c05t_vo = { system = "burst", sprite = "lightning", size = 380, life = 0.35, tint = BLUE,
      ground = "vortex", ground_size = 460, ground_seconds = 1.4, grow = 0.3 },
    c05t_aoe_hint = { ground = "arcane_circle", ground_seconds = 1.5, tint = BLUE },
    c05t_aoe_hint_enemy = { ground = "arcane_circle", ground_seconds = 0.8, tint = WARN },
    c05t_blink_start = { system = "pop", sprite = "lightning", size = 640, life = 0.3, tint = BLUE,
      ground = "blink_flash", ground_size = 420, ground_seconds = 0.35 },
    c05t_eclipse_area = { system = "pop", sprite = "lightning", size = 950, life = 0.5, tint = BLUE,
      ground = "shock_ring", ground_seconds = 0.6, grow = 0.15 },
    c05t_blink_end = { system = "burst", sprite = "lightning", size = 520, life = 0.45, tint = BLUE,
      ground = "dust_crack", ground_size = 700, ground_seconds = 0.6, grow = 0.3 },
  },
  notes = {
    c05_matsukaze = "c05w_vo whirlwind burst + c05w_armlet stream for the buff.",
    c05_thunder_dance = "c05e_cast_sound burst, c05e_bolt flight, c05e_impact burst. The cast cue has no radius arg: ground_size 1700 is an art fallback; adding radius to the package cue would make it exact.",
    c05_thunder_spear = "Passive: c05r_chain_lightning on each landed attack (pop at the attacker, line to the target) + c05r_tempest stream on the slowed unit.",
    c05_flash_thunder = "c05t_vo channel, c05t_blink_start flash, c05t_eclipse_area giant bolt (ground uses the package radius), c05t_blink_end burst. If the channel is interrupted the c05t_vo ground still runs its time. c05t_stun has no cue.",
  },
}
