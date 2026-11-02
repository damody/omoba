-- C03 shimazu_yoshihiro: oni red and fire; steel blue for iron, purple for his thunder.
local RED, STEEL, PURPLE, SLOW = { 1.0, 0.2, 0.1 }, { 0.12, 0.5, 1.0 }, { 0.5, 0.1, 1.0 }, { 0.08, 0.3, 1.0 }
return {
  recipes = {
    -- c03_iron_shirt: a huge iron shield flares, a shield stays underfoot, steel motes while it lasts
    c03w_vo = { system = "pop", sprite = "iron_shield", size = 460, life = 0.45, tint = STEEL,
      ground = "shock_ring", ground_size = 600, ground_seconds = 0.45, grow = 0.2 },
    c03w_grow = { system = "burst", sprite = "spark_star", size = 260, life = 0.4, tint = STEEL,
      ground = "iron_shield", ground_size = 240 },
    c03w_iron_glow = { system = "stream", sprite = "spark_star", size = 85, life = 0.4, loop = true, tint = STEEL },
    -- c03_quake_axe: the axe swing, then dust and a cracked ground
    c03e_vo = { system = "pop", sprite = "crescent_slash", size = 430, life = 0.24, tint = RED },
    c03e_thunder_clap = { system = "burst", sprite = "smoke_sheet", size = 430, life = 0.55,
      ground = "dust_crack", ground_size = 760, ground_seconds = 0.7, grow = 0.3 },
    c03e_move_slow = { ground = "slow_chains", ground_size = 170, ground_seconds = 1.2, tint = SLOW },
    -- c03_thunder_wrath: a purple bolt strikes the target
    c03r_thunder = { system = "pop", sprite = "lightning", size = 480, life = 0.3, tint = PURPLE,
      ground = "blink_flash", ground_size = 300, ground_seconds = 0.3, height = 20, grow = 0.4 },
    -- c03_iron_hammer: a hammer flares at the start and marches; fire and cracks at every tick
    c03t_hammer = { mesh = "war_hammer", ground_size = 260, height = 90, fly = true, spin = 720 },
    c03t_hammer_hit = { system = "burst", sprite = "flame_sheet", size = 420, life = 0.5,
      ground = "dust_crack", ground_size = 900, ground_seconds = 0.45, grow = 0.3 },
  },
  notes = {
    c03_iron_shirt = "c03w_vo shield flare + c03w_grow sparks and follow-ground; c03w_iron_glow stream for the buff. Blocking enemy debuffs is correct behaviour.",
    c03_quake_axe = "c03e_vo slash + c03e_thunder_clap dust burst. The cue has no radius arg: ground_size 760 is an art fallback; adding radius to the package cue would make it exact.",
    c03_thunder_wrath = "Passive: c03r_thunder bolt on the charged hit. c03r_stun has no cue, so the stun itself shows nothing after the bolt.",
    c03_iron_hammer = "c03t_hammer flying image + start flare; c03t_hammer_hit burst each tick (no radius arg, ground_size 900 is an art fallback). The system stays at the start, only the flat image travels. c03t_stun has no cue.",
  },
}
