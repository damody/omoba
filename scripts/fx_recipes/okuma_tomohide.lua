-- B26 大熊朝秀: fire orange; blood red for the fury. Slows in blue.
-- Status cues (SyncStatusCue) draw EITHER the system (centred at the feet) OR the ground image: rows here carry the system.
local FIRE, BLOOD, BLUE = { 1.0, 0.2, 0.015 }, { 0.8, 0.02, 0.01 }, { 0.12, 0.5, 1.0 }
return {
  recipes = {
    -- Q 蓄力波: a swirl at the cast, sparks as the charge starts, a thin fire ring every second, explosions on enemies
    b26w_voice = { system = "pop", sprite = "ramp_pops", size = 170, life = 0.4, tint = FIRE,
      ground = "ramp_band", ground_size = 260, ground_seconds = 0.45, grow = 0.5 },
    b26w_charge_fire = { system = "sparks", tint = FIRE },
    b26w_charge_pulse = { system = "pop", sprite = "ramp_hit", size = 110, life = 0.25, tint = FIRE,
      ground = "ramp_thin_ring", ground_size = 620, ground_fixed = true, ground_seconds = 0.5, grow = 0.35 },
    b26w_fire_explode = { system = "sparks", tint = FIRE },
    b26w_slow_overhead = { system = "stream", sprite = "slow_chains", size = 80, life = 0.3, loop = true, alpha = 0.35, tint = BLUE },
    -- The closing shockwave, whichever way the charge ends
    b26d_thunder_clap = { system = "pop", sprite = "ramp_hit", size = 150, life = 0.25, tint = FIRE,
      ground = "ramp_thin_ring", ground_size = 1000, ground_seconds = 0.45, grow = 0.2 },
    b26w_shockwave = { system = "sparks", tint = FIRE,
      ground = "ramp_blast", ground_size = 330, ground_fixed = true, ground_seconds = 0.4, grow = 0.45, tint = FIRE },
    -- D 釋放蓄力波: a flash, then the two cues above
    b26d_voice = { system = "pop", sprite = "ramp_star", size = 170, life = 0.22, tint = FIRE },
    -- W 血性狂怒: blood-red flames and a ring, a flash as he grows, red motes while it lasts.
    -- b26e_model_scale_up has an 8 s duration, which a ground image would take: no ground on it.
    b26e_voice = { system = "pop", sprite = "ramp_star", size = 130, life = 0.25, tint = BLOOD,
      mesh = "oni_mask", ground_size = 60, ground_seconds = 0.9, height = 65, face_viewer = true },
    b26e_model_scale_up = { system = "pop", sprite = "ramp_rays", size = 210, life = 0.3, tint = BLOOD },
    b26e_status_effect = { system = "stream", sprite = "ramp_pops", size = 80, life = 0.35, loop = true, tint = BLOOD },
    -- R 當者披靡: a roar of fire, huge wings flaring, then fire at the feet for the duration
    b26t_voice_roar = { system = "pop", sprite = "ramp_blast", size = 420, area = true, life = 0.45, tint = FIRE,
      ground = "ramp_thin_ring", ground_size = 900, ground_seconds = 0.55, grow = 0.2 },
    b26t_burst = { system = "pop", sprite = "ramp_rays", size = 460, area = true, life = 0.5, tint = FIRE,
      mesh = "fire_wings", ground_size = 200, ground_seconds = 1.1, height = 40, grow = 0.5, face_viewer = true },
    b26t_wings = { system = "stream", sprite = "flame_sheet", size = 60, life = 0.3, loop = true, alpha = 0.35 },
  },
  notes = {
    b26_charging_wave = "body on b26w_voice, b26w_charge_pulse, b26w_fire_explode, b26w_shockwave, b26d_thunder_clap; b26w_charge_fire and b26w_slow_overhead are the unit bodies",
    b26_blood_fury = "body on b26e_voice and b26e_model_scale_up; b26e_status_effect on the unit; b26e_immune has no cue; it blocks enemy debuffs, which is correct behaviour",
    b26_kozuke_valor = "no cue: the passive plays nothing, so it cannot show a visual without a package change",
    b26_unstoppable = "body on b26t_voice_roar and b26t_burst; b26t_wings shows fire, not wings, for the duration because a status cue with a system drops its ground image; check fire_wings sprite orientation (rot) in video",
    b26_release_wave = "body on b26d_voice, then b26d_thunder_clap and b26w_shockwave",
  },
}
