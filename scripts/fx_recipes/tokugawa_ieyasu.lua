-- A01 德川家康: gold rally and net, healing green, thunder-blade blue.
-- Status cues (SyncStatusCue) draw EITHER the system (when there is one, centred at the feet) OR the ground image,
-- so status rows here carry only the one that should show.
local GOLD, GREEN, BLUE = { 1.0, 0.55, 0.08 }, { 0.1, 0.9, 0.25 }, { 0.12, 0.5, 1.0 }
return {
  recipes = {
    -- Q 東照權現: healing rings thrown out over the whole area, sparkles on each ally
    a01w_holy_bolt = { system = "burst", sprite = "heal_ring", size = 340, life = 0.55, tint = GREEN,
      ground = "heal_ring", ground_size = 1100, ground_seconds = 0.7, grow = 0.25 },
    a01w_healing_ring = { system = "burst", sprite = "spark_star", size = 110, life = 0.45, tint = GREEN },
    -- W 天羅地網: the net flung at the cast, a net clinging to each rooted enemy while the root lasts
    a01e_vo = { system = "pop", sprite = "net", size = 460, life = 0.35, tint = GOLD },
    a01e_net = { system = "stream", sprite = "net", size = 170, life = 0.4, loop = true, tint = GOLD },
    -- E 雷霆刃: a lightning discharge at the cast and on a proc, a bolt on everyone struck
    a01r_vo = { system = "burst", sprite = "lightning", size = 620, life = 0.45, tint = BLUE,
      ground = "shock_ring", ground_size = 1100, ground_seconds = 0.5, grow = 0.15 },
    a01r_thunder_sound = { system = "pop", sprite = "lightning", size = 380, life = 0.28, tint = BLUE,
      ground = "shock_ring", ground_size = 620, ground_seconds = 0.35, grow = 0.2 },
    a01r_lightning = { system = "pop", sprite = "lightning", size = 460, life = 0.3, tint = BLUE,
      mesh = "thunder_sword", ground_size = 280, ground_seconds = 0.6, height = 140, grow = 0.4, face_viewer = true },
    -- R 三河之勇: banners thrown up at the cast, a large banner on the ground while it lasts
    a01t_vo = { system = "burst", sprite = "banner_rally", size = 520, life = 0.6, tint = GOLD,
      ground = "shock_ring", ground_size = 1400, ground_seconds = 0.7, grow = 0.15 },
    a01t_banner = { mesh = "war_banner", ground_size = 280, ground_seconds = 1.2, height = 140, face_viewer = true },
    -- Worn by every ally in the passive aura, creeps included and for as long as they stay: ground only, on purpose.
    a01t_ring = { ground = "shock_ring", ground_size = 110, ground_seconds = 1.2, tint = GOLD },
  },
  notes = {
    a01_realm_momentum = "no cue: the passive plays nothing, so it cannot show a visual without a package change",
    a01_tosho_gongen = "body on a01w_holy_bolt (area) and a01w_healing_ring (one-shot per ally)",
    a01_heaven_net = "body on a01e_vo (at the caster) and a01e_net (each rooted enemy); no cue at the target point, so an empty cast shows nothing there",
    a01_thunder_blade = "body on a01r_vo, a01r_lightning, a01r_thunder_sound; status a01r_thunder has no cue, so the 5 s charged blade itself shows nothing",
    a01_mikawa_valor = "air body only at the cast (a01t_vo); a01t_banner is ground for the duration; a01t_title has no position and spell_hint_self is shared, both left alone",
  },
}
