-- B26 大熊朝秀: fire orange; blood red for the fury. Slows in blue.
-- Status cues (SyncStatusCue) draw EITHER the system (centred at the feet) OR the ground image: rows here carry the system.
local FIRE, BLOOD, BLUE = { 1.0, 0.55, 0.08 }, { 1.0, 0.2, 0.1 }, { 0.12, 0.5, 1.0 }
return {
  recipes = {
    -- Q 蓄力波: a swirl at the cast, fire at the feet while charging, a fire pulse every second, explosions on enemies
    b26w_voice = { system = "pop", sprite = "vortex", size = 520, life = 0.5, tint = FIRE,
      ground = "vortex", ground_size = 700, ground_seconds = 0.6, grow = 0.3 },
    b26w_charge_fire = { system = "stream", sprite = "flame_sheet", size = 120, life = 0.5, loop = true, bold = true },
    b26w_charge_pulse = { system = "burst", sprite = "flame_sheet", size = 220, life = 0.4, tint = FIRE,
      ground = "shock_ring", ground_size = 1000, ground_seconds = 0.45, grow = 0.15 },
    b26w_fire_explode = { system = "burst", sprite = "flame_sheet", size = 190, life = 0.45, tint = FIRE },
    b26w_slow_overhead = { system = "stream", sprite = "slow_chains", size = 120, life = 0.4, loop = true, tint = BLUE },
    -- The closing shockwave, whichever way the charge ends
    b26d_thunder_clap = { system = "pop", sprite = "shock_ring", size = 600, life = 0.35, tint = FIRE,
      ground = "shock_ring", ground_size = 1000, ground_seconds = 0.5, grow = 0.15 },
    b26w_shockwave = { system = "burst", sprite = "flame_sheet", size = 480, life = 0.6, tint = FIRE,
      mesh = "shock_dome", ground_size = 400, ground_seconds = 0.5, height = 70, grow = 0.3 },
    -- D 釋放蓄力波: a flash, then the two cues above
    b26d_voice = { system = "pop", sprite = "blink_flash", size = 420, life = 0.25, tint = FIRE },
    -- W 血性狂怒: blood-red flames and a ring, a flash as he grows, red motes while it lasts.
    -- b26e_model_scale_up has an 8 s duration, which a ground image would take: no ground on it.
    b26e_voice = { system = "burst", sprite = "flame_sheet", size = 460, life = 0.5, tint = BLOOD,
      ground = "shock_ring", ground_size = 720, ground_seconds = 0.45, grow = 0.2 },
    b26e_model_scale_up = { system = "pop", sprite = "blink_flash", size = 380, life = 0.3, tint = BLOOD },
    b26e_status_effect = { system = "stream", sprite = "spark_star", size = 80, life = 0.35, loop = true, tint = BLOOD },
    -- R 當者披靡: a roar of fire, huge wings flaring, then fire at the feet for the duration
    b26t_voice_roar = { system = "burst", sprite = "flame_sheet", size = 620, life = 0.6, tint = FIRE,
      ground = "shock_ring", ground_size = 1000, ground_seconds = 0.6, grow = 0.15 },
    b26t_burst = { system = "pop", sprite = "fire_wings", size = 900, life = 0.6,
      mesh = "fire_wings", ground_size = 300, ground_seconds = 1.5, height = 170, grow = 0.5, face_viewer = true },
    b26t_wings = { system = "stream", sprite = "flame_sheet", size = 120, life = 0.4, loop = true },
  },
  notes = {
    b26_charging_wave = "body on b26w_voice, b26w_charge_pulse, b26w_fire_explode, b26w_shockwave, b26d_thunder_clap; b26w_charge_fire and b26w_slow_overhead are the unit bodies",
    b26_blood_fury = "body on b26e_voice and b26e_model_scale_up; b26e_status_effect on the unit; b26e_immune has no cue; it blocks enemy debuffs, which is correct behaviour",
    b26_kozuke_valor = "no cue: the passive plays nothing, so it cannot show a visual without a package change",
    b26_unstoppable = "body on b26t_voice_roar and b26t_burst; b26t_wings shows fire, not wings, for the duration because a status cue with a system drops its ground image; check fire_wings sprite orientation (rot) in video",
    b26_release_wave = "body on b26d_voice, then b26d_thunder_clap and b26w_shockwave",
  },
}
