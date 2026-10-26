return function(ctx)
  return {
    schema_version = 2,
    placeholder = { body = "capsule_ranged", team_tint = true, weapon_socket = "hand_r" },
    animation_states = { idle = "idle", move = "move", attack = "attack", cast = "cast", channel = "cast", death = "death", recall = "idle" },
    abilities = {
      c17_light_chain = { preview = "unit_target", animation = "cast",
        cues = { cast = "light_chain_cast", loop = "light_chain_tether", impact = "light_chain_break" } },
      c17_victory_blessing = { preview = "unit_target", animation = "cast",
        cues = { cast = "victory_blessing_cast", loop = "victory_blessing_aura" } },
      c17_divine_strike = { preview = "ground_circle", animation = "cast",
        cues = { cast = "divine_strike_cast", impact = "divine_strike_flash" } },
      c17_aurora_guard = { preview = "ground_circle", animation = "cast",
        cues = { cast = "aurora_guard_charge", impact = "aurora_guard_burst" } },
      c17_devotion = { preview = "none", animation = "cast",
        cues = { impact = "devotion_restore" } },
    },
  }
end
