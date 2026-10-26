return function(ctx)
  return {
    schema_version = 2,
    placeholder = { body = "capsule_ranged", team_tint = true, weapon_socket = "hand_l" },
    animation_states = { idle = "idle", move = "move", attack = "attack", cast = "cast", channel = "cast", death = "death", recall = "idle" },
    abilities = {
      c15_scatter_shot = { preview = "ground_circle", animation = "attack",
        cues = { cast = "scatter_shot_cast", projectile = "c15_scatter_shot", impact = "scatter_shot_stun" } },
      c15_burst_arrow = { preview = "swept_path", path_width = 340, animation = "cast",
        cues = { cast = "burst_arrow_cast", projectile = "c15_burst_arrow", loop = "burst_arrow_fire_line", ["end"] = "burst_arrow_fire_end" } },
      c15_precise_light = { preview = "none", animation = "cast", cues = { loop = "precise_light_glow" } },
      c15_hundred_arrows = { preview = "unit_target", animation = "cast",
        cues = { cast = "hundred_arrows_cast", impact = "hundred_arrows_stun", loop = "lethal_arrow_orbit", ["end"] = "lethal_arrow_fade" } },
    },
  }
end
