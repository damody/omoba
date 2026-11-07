-- Default (generated-side) presentation. Manual Editor appearance settings
-- override these; regenerating never writes over them (README §6).
return function(ctx)
  return {
    schema_version = 2,
    placeholder = { body = "capsule_ranged", team_tint = true, weapon_socket = "Weapon Ref" },
    animation_states = { idle = "idle", move = "move", attack = "attack", cast = "cast", channel = "cast", death = "death", recall = "idle" },
    abilities = {
      b01_sniper_mode = { preview = "none", animation = "cast",
        cues = { cast = "sniper_mode_on", loop = "sniper_mode_aura", ["end"] = "sniper_mode_off" },
        range_ring = "attack_range" },
      b01_saika_corps = { preview = "point_marker", animation = "cast",
        cues = { cast = "saika_corps_cast", spawn = "saika_corps_spawn" }, charges_counter = true },
      b01_rain_iron_cannon = { preview = "none", animation = "attack",
        cues = { impact = "rain_iron_cannon_bonus" } },
      b01_three_stage = { preview = "none", animation = "cast",
        cues = { cast = "three_stage_cast", loop = "three_stage_aura", ["end"] = "three_stage_off" },
        attack_visual = "three_shot_visual_only" },
    },
  }
end
