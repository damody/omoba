-- Cues the packages play at a landing point, a cast or a passive's trigger. Radius comes from the package.
local GOLD, RED, ARCANE, GREEN = { 1.0, 0.55, 0.08 }, { 1.0, 0.2, 0.1 }, { 0.5, 0.1, 1.0 }, { 0.1, 0.9, 0.25 }
return { recipes = {
  b20w_rock_land = { system = "pop", sprite = "rock", size = 60, fixed_size = true, life = 0.3, ground = "crack_earth", ground_size = 230, ground_fixed = true, ground_seconds = 0.4, grow = 0.5 },
  a01e_net_land = { system = "pop", sprite = "net", size = 190, life = 0.4, tint = GOLD, ground = "net", ground_size = 300, ground_seconds = 0.8, grow = 0.3 },
  a14w_spear_cast = { system = "pop", sprite = "chain_bolt", size = 180, life = 0.25, tint = { 1.0, 0.5, 0.04 }, also = { "fx_layer_spear_glow" },
    ground = "ramp_spear", ground_size = 380, ground_seconds = 0.5, height = 60, launch = 500, front = 150, face_camera = 0.85 },
  a12t_cast = { system = "pop", sprite = "soft_glow", size = 110, fixed_size = true, life = 0.25, tint = { 0.3, 0.02, 0.85 },
      ground = "possession_sigil", ground_size = 250, ground_fixed = true, ground_seconds = 0.8, grow = 0.75, also = { "fx_layer_possession_flare", "fx_layer_possession_embers" } },
  a01d_restore = { ground = "ramp_band", ground_size = 240, ground_fixed = true, ground_seconds = 0.5, grow = 0.6, tint = { 0.12, 0.5, 1.0 }, also = { "fx_layer_mana_motes" } },
  b20r_restore = { system = "burst", sprite = "heal_ring", size = 280, life = 0.6, tint = GREEN },
  b26r_restore = { system = "pop", sprite = "ramp_pops", size = 190, life = 0.45, tint = { 0.05, 0.7, 0.12 }, ground = "ramp_band", ground_size = 240, ground_fixed = true, ground_seconds = 0.45, grow = 0.6, no_damage_flash = true },
  -- The stand-ins' own hits: small, so they never cover the hero being shown.
  -- The ring every cast draws: small and quick, so it does not stand between the skills.
  cast = { system = "pop", sprite = "blink_flash", size = 70, fixed_size = true, life = 0.12, tint = { 0.45, 0.8, 0.9 },
    ground = "shock_ring", ground_size = 150, ground_fixed = true, ground_seconds = 0.3, grow = 0.75 },
  b01r_hit = { system = "pop", sprite = "spark_star", size = 70, fixed_size = true, life = 0.18, tint = GOLD },
  b04e_impact = { system = "pop", sprite = "blink_flash", size = 90, fixed_size = true, life = 0.2, tint = ARCANE },
  b04e_blast = { system = "sparks", tint = ARCANE },
  a12d_stack = { mesh = "gunsen_fan", ground_size = 150, ground_seconds = 0.5, lead = 80, launch = 81, height = 50, grow = 0.75, delay = 0.25, face_viewer = true },
  a12d_ready = { system = "pop", sprite = "ramp_star", size = 180, life = 0.4, tint = { 1.0, 0.3, 0.02 },
    ground = "ramp_thin_ring", ground_size = 520, ground_fixed = true, ground_seconds = 0.5, grow = 0.4 },
  a14r_stack_hit = { system = "pop", sprite = "stack_spears", size = 300, life = 0.36, tint = { 1.0, 0.42, 0.02 }, ground = "ramp_band", ground_size = 200, ground_seconds = 0.35, height = 30, grow = 0.5 },
} }
