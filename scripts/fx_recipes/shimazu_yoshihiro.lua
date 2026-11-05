-- C03 shimazu_yoshihiro: oni red and fire; steel blue for iron, purple for his thunder.
local RED, STEEL, PURPLE, SLOW, EARTH = { 1.0, 0.09, 0.015 }, { 0.04, 0.22, 0.95 }, { 0.35, 0.02, 0.95 }, { 0.08, 0.3, 1.0 }, { 0.75, 0.42, 0.18 }
return {
  recipes = {
    -- c03_iron_shirt: a huge iron shield flares, a shield stays underfoot, steel motes while it lasts
    c03w_vo = { system = "pop", sprite = "ramp_star", size = 170, life = 0.3, tint = STEEL,
      mesh = "iron_bell", ground_size = 200, ground_seconds = 0.7, grow = 0.5, mesh_opacity = 0.45 },
    c03w_grow = { system = "pop", sprite = "ramp_pops", size = 170, life = 0.4, tint = STEEL,
      ground = "ramp_ring_marks", ground_size = 240, ground_seconds = 0.6, grow = 0.6 },
    c03w_iron_glow = { system = "stream", sprite = "ramp_pops", size = 85, life = 0.4, loop = true, tint = STEEL },
    -- c03_quake_axe: the axe swing, then dust and a cracked ground
    c03e_vo = { system = "pop", sprite = "ramp_slash", size = 210, life = 0.24, tint = RED },
    c03e_thunder_clap = { system = "burst", sprite = "rock", size = 65, fixed_size = true, life = 0.45,
      ground = "ramp_crack", ground_size = 420, ground_fixed = true, ground_seconds = 0.9, grow = 0.4, tint = RED, also = { "fx_layer_quake_sparks" } },
    c03e_move_slow = { ground = "slow_chains", ground_size = 170, ground_seconds = 1.2, tint = SLOW },
    -- c03_thunder_wrath: a purple bolt strikes the target
    c03r_thunder = { system = "pop", sprite = "lightning", size = 480, life = 0.3, tint = PURPLE,
      ground = "ramp_rays", ground_size = 190, ground_seconds = 0.3, height = 20, grow = 0.5 },
    -- c03_iron_hammer: a hammer flares at the start and marches; fire and cracks at every tick
    -- Each blow lands 300 ahead of where the hammer would start, so it flies that far ahead and meets them.
    -- It faces the viewer and does not tumble: end on, a hammer reads as a barrel.
    -- It comes down on each of the package's blows (interval 0.6 s), at the point the blow lands.
    c03t_hammer = { mesh = "war_hammer", ground_size = 170, fly = true, lead = 300, face_viewer = true, bounce = 130, bounce_seconds = 0.6, pivot = 60 },
    c03t_hammer_hit = {
      ground = "ramp_crack", ground_size = 560, area = true, ground_fixed = true, ground_seconds = 0.5, grow = 0.4, tint = RED },
  },
  notes = {
    c03_iron_shirt = "c03w_vo shield flare + c03w_grow sparks and follow-ground; c03w_iron_glow stream for the buff. Blocking enemy debuffs is correct behaviour.",
    c03_quake_axe = "c03e_vo slash + c03e_thunder_clap dust burst. The cue has no radius arg: ground_size 760 is an art fallback; adding radius to the package cue would make it exact.",
    c03_thunder_wrath = "Passive: c03r_thunder bolt on the charged hit. c03r_stun has no cue, so the stun itself shows nothing after the bolt.",
    c03_iron_hammer = "c03t_hammer flying image + start flare; c03t_hammer_hit burst each tick (no radius arg, ground_size 900 is an art fallback). The system stays at the start, only the flat image travels. c03t_stun has no cue.",
  },
}
