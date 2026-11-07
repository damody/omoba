return function(ctx)
  return {
    schema_version = 2,
    placeholder = { body = "capsule_ranged", team_tint = true, weapon_socket = "hand_r" },
    animation_states = { idle = "idle", move = "move", attack = "attack", cast = "cast", channel = "cast", death = "death", recall = "idle" },
    abilities = {
      a34_frost_bolt = { preview = "unit_target", animation = "cast",
        cues = { cast = "frost_bolt_cast", projectile = "a34_frost_bolt", impact = "frost_bolt_freeze" } },
      a34_iceberg_strike = { preview = "unit_target_circle", animation = "cast",
        cues = { cast = "iceberg_cast", impact = "iceberg_shatter" } },
      a34_frost_breath = { preview = "swept_path", path_width = 325, animation = "cast",
        cues = { cast = "frost_breath_cast", projectile = "a34_frost_breath", impact = "frost_breath_hit", loop = "frost_breath_dot" } },
      a34_rapid_cooling = { preview = "none", animation = "cast",
        cues = { cast = "rapid_cooling_cast", loop = "rapid_cooling_aura", ["end"] = "rapid_cooling_end" } },
    },
  }
end
