-- A12 大谷吉繼: arcane purple. Status cues (a12r_vortex, a12t_aura) draw only their system, so they carry no ground.
local ARCANE, GREEN, BLUE = { 0.5, 0.1, 1.0 }, { 0.1, 0.9, 0.25 }, { 0.12, 0.5, 1.0 }
return {
  recipes = {
    -- shared by a12_insight_flash and a12_insight_flash_t: the flare at the caster
    a12w_vo = { system = "pop", sprite = "blink_flash", size = 180, life = 0.25, tint = ARCANE },
    -- a12_insight_flash: a burst of light that throws enemies out
    a12w_glow = { system = "burst", sprite = "blink_flash", size = 620, life = 0.7, tint = ARCANE,
      ground = "shock_ring", ground_size = 720, ground_fixed = true, ground_seconds = 0.7, grow = 0.2 },
    a12w_burst = { system = "pop", sprite = "blink_flash", size = 320, life = 0.45, tint = ARCANE },
    -- a12_insight_flash_t: a vortex pulling inward at the point
    a12w_hide_ring = { system = "burst", sprite = "vortex", size = 480, life = 0.55, tint = ARCANE,
      ground = "vortex", ground_size = 1000, ground_seconds = 0.8 },
    -- shared by a12_mystic_pierce and a12_mystic_field: crackling power and a circle at the field's size
    a12e_vo = { system = "burst", sprite = "spark_star", size = 170, life = 0.45, tint = ARCANE,
      ground = "arcane_circle", ground_size = 800, ground_seconds = 0.9, grow = 0.3 },
    a12e_heal_number = { system = "pop", sprite = "heal_ring", size = 240, life = 0.4, tint = GREEN,
      ground = "heal_ring", ground_size = 220, ground_seconds = 0.45, grow = 0.5 },
    a12e_mana_number = { system = "pop", sprite = "spark_star", size = 220, life = 0.35, tint = BLUE,
      ground = "arcane_circle", ground_size = 220, ground_seconds = 0.45, grow = 0.5 },
    -- shared by a12_awe and a12_awe_t
    a12r_vo = { system = "pop", sprite = "arcane_circle", size = 260, life = 0.35, tint = ARCANE },
    a12r_burst = { system = "burst", sprite = "spark_star", size = 220, life = 0.5, tint = ARCANE,
      ground = "vortex", ground_size = 800, ground_seconds = 0.8, grow = 0.3, alpha = 0.6 },
    a12r_vortex = { system = "stream", sprite = "vortex", size = 90, life = 0.25, loop = true, tint = ARCANE },
    -- a12_mystic_arrow
    a12f_arc_sound = { system = "pop", sprite = "soft_glow", size = 120, life = 0.15, tint = ARCANE },
    a12f_arrow = { system = "stream", sprite = "spark_star", size = 90, life = 0.5, loop = true, face = true, tint = ARCANE,
      mesh = "arcane_arrow", ground_size = 200, ground_seconds = 20, height = 70, linger = 0.7 },
    a12f_streak = { ground = "chain_bolt", ground_size = 170, ground_seconds = 0.7, height = 60, line = true, tint = ARCANE },
    a12f_hit = { system = "pop", sprite = "blink_flash", size = 240, life = 0.3, tint = ARCANE,
      ground = "blink_flash", ground_size = 130, ground_seconds = 0.35, line = true },
    -- a12_mystic_possession: purple lightning crackling on him for the whole form
    a12t_aura = { system = "stream", sprite = "possession_flare", size = 50, life = 0.5, loop = true },
  },
  notes = {
    a12_insight_flash = "a12w_glow (at caster, always plays) carries it; a12w_burst only on visible enemies; the empowered stun a12w_stun has no cue.",
    a12_insight_flash_t = "a12w_hide_ring at the point; a12w_vo flare at the caster; a12w_t_stun has no cue.",
    a12_scheme = "a12d_stack on himself for each stack, a12d_ready when the next cast is strengthened (package_cues).",
    a12_mystic_pierce = "a12e_vo at cast, a12e_heal_number / a12e_mana_number on procs; the 10 s buff itself has no status cue.",
    a12_mystic_field = "only cue is a12e_vo, shared with a12_mystic_pierce: one flash and circle at cast, nothing for its seconds of duration; needs its own field cue in the package.",
    a12_awe = "a12r_burst at the point, a12r_vortex on the slowed.",
    a12_awe_t = "same a12r_burst and a12r_vortex; ground is 800 across while its damage radius is 500, since the cue passes no radius.",
    a12_mystic_arrow = "a12f_arrow flight, a12f_arc_sound muzzle; a12f_streak lights its path, a12f_hit pierces each target along the flight. missing_image: arrow (crescent_slash stands in).",
    a12_mystic_possession = "a12t_aura stream only, weakest of the kit: no cast cue of its own and spell_hint_self is shared with 4 other heroes so left alone; needs a cast cue in the package for a big body.",
  },
}
