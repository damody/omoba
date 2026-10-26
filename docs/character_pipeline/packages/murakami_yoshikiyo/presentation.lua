return function(ctx)
  return {
    schema_version = 2,
    placeholder = { body = "capsule_melee", team_tint = true, weapon_socket = "hand_r" },
    animation_states = { idle = "idle", move = "move", attack = "attack", cast = "cast", channel = "cast", death = "death", recall = "idle" },
    abilities = {
      b29_ground_slam = { preview = "self_circle", animation = "cast",
        cues = { cast = "ground_slam_cast", impact = "ground_slam_shockwave" } },
      b29_quake_axe = { preview = "ground_circle", animation = "cast",
        cues = { cast = "quake_axe_leap", impact = "quake_axe_thunderclap", loop = "quake_axe_breaker_aura", ["end"] = "quake_axe_aura_end" } },
      b29_morale_boost = { preview = "none", animation = "idle",
        cues = { loop = "morale_boost_aura" } },
      b29_earth_tremor = { preview = "ground_circle", animation = "channel",
        cues = { cast = "earth_tremor_cast", impact = "earth_tremor_wave", loop = "earth_tremor_channel", ["end"] = "earth_tremor_end" } },
    },
  }
end
