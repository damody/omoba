-- Cues the packages play at a landing point, a cast or a passive's trigger. Radius comes from the package.
local GOLD, RED, ARCANE, GREEN = { 1.0, 0.55, 0.08 }, { 1.0, 0.2, 0.1 }, { 0.5, 0.1, 1.0 }, { 0.1, 0.9, 0.25 }
return { recipes = {
  b20w_rock_land = { system = "burst", sprite = "smoke_sheet", size = 150, life = 0.45, ground = "dust_crack", ground_size = 300, ground_seconds = 0.5, grow = 0.3 },
  a01e_net_land = { system = "pop", sprite = "net", size = 190, life = 0.4, tint = GOLD, ground = "net", ground_size = 300, ground_seconds = 0.8, grow = 0.3 },
  a14w_spear_cast = { system = "pop", sprite = "chain_bolt", size = 180, life = 0.25, tint = GOLD },
  a12t_cast = { system = "pop", sprite = "arcane_circle", size = 190, life = 0.45, tint = ARCANE },
  a01d_restore = { system = "pop", sprite = "heal_ring", size = 120, life = 0.3, tint = GREEN },
  b20r_restore = { system = "burst", sprite = "heal_ring", size = 190, life = 0.4, tint = RED },
  b26r_restore = { system = "pop", sprite = "heal_ring", size = 130, life = 0.35, tint = RED },
  a12d_ready = { system = "pop", sprite = "arcane_circle", size = 150, life = 0.4, tint = ARCANE },
  a14r_stack_hit = { system = "pop", sprite = "spark_star", size = 120, life = 0.2, tint = GOLD },
} }
