return function(ctx)
  return {
    schema_version = 2,
    placeholder = { body = "capsule_ranged", team_tint = true, weapon_socket = "hand_l" },
    animation_states = { idle = "idle", move = "move", attack = "attack", cast = "cast", channel = "cast", death = "death", recall = "idle" },
    abilities = {
      a02_venom_arrow = { preview = "none", animation = "cast",
        cues = { cast = "venom_arrow_toggle", loop = "venom_arrow_glow", impact = "venom_arrow_poison" } },
      a02_wind_prayer = { preview = "none", animation = "cast",
        cues = { cast = "wind_prayer_cast", loop = "wind_prayer_focus" } },
      a02_wind_drift = { preview = "none", animation = "cast", cues = {} },
      a02_shadow_meld = { preview = "none", animation = "cast",
        cues = { cast = "shadow_meld_cast", loop = "shadow_meld_fade", ["end"] = "shadow_meld_reveal" } },
    },
  }
end
