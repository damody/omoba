return function(ctx)
  return {
    schema_version = 2,
    placeholder = { body = "capsule_melee", team_tint = true, weapon_socket = "hand_r" },
    animation_states = { idle = "idle", move = "move", attack = "attack", cast = "cast", channel = "cast", death = "death", recall = "idle" },
    abilities = {
      b06_brave_warriors = { preview = "self_circle", animation = "cast",
        cues = { cast = "brave_warriors_cast", loop = "brave_warriors_buff" } },
      b06_flame_fangs = { preview = "unit_target", animation = "cast",
        cues = { cast = "flame_fangs_cast", projectile = "b06_flame_fangs", impact = "flame_fangs_hit" } },
      b06_mind_eye = { preview = "none", animation = "idle", cues = { loop = "mind_eye_guard" } },
      b06_first_warrior = { preview = "none", animation = "cast",
        cues = { cast = "first_warrior_cast", loop = "first_warrior_aura", ["end"] = "first_warrior_end" } },
    },
  }
end
