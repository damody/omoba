return function(ctx)
  return {
    schema_version = 2,
    placeholder = { body = "capsule_melee", team_tint = true, weapon_socket = "hand_r" },
    animation_states = { idle = "idle", move = "move", attack = "attack", cast = "cast", channel = "cast", death = "death", recall = "idle" },
    abilities = {
      b08_fang_slash = { preview = "swept_path", path_width = 175, animation = "cast",
        cues = { cast = "fang_slash_cast", projectile = "b08_fang_slash", impact = "fang_slash_hit" } },
      b08_flash_cut = { preview = "unit_target", animation = "cast",
        cues = { cast = "flash_cut_blink", impact = "flash_cut_slash" } },
      b08_living_soul = { preview = "none", animation = "cast", cues = { loop = "living_soul_stack" } },
      b08_sky_fang = { preview = "swept_path", path_width = 500, animation = "cast",
        cues = { cast = "sky_fang_cast", projectile = "b08_sky_fang", impact = "sky_fang_hit" } },
      b08_endless_fang = { preview = "unit_target", animation = "cast",
        cues = { cast = "endless_fang_cast", impact = "endless_fang_hit" } },
    },
  }
end
