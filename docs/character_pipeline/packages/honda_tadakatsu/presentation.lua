return function(ctx)
  return {
    schema_version = 2,
    placeholder = { body = "capsule_melee", team_tint = true, weapon_socket = "hand_r" },
    animation_states = { idle = "idle", move = "move", attack = "attack", cast = "cast", channel = "cast", death = "death", recall = "idle" },
    abilities = {
      a07_unscathed = { preview = "none", animation = "cast",
        cues = { cast = "unscathed_cast", loop = "unscathed_guard", ["end"] = "unscathed_end" } },
      a07_red_thunder = { preview = "self_circle", animation = "attack",
        cues = { cast = "red_thunder_cast", impact = "red_thunder_shockwave" } },
      a07_shattering_spear = { preview = "none", animation = "attack", cues = { impact = "shattering_spear_crit" } },
      a07_strongest = { preview = "none", animation = "cast",
        cues = { cast = "strongest_cast", loop = "strongest_lightning", impact = "strongest_cleave", ["end"] = "strongest_end" } },
      a07_rouse = { preview = "none", animation = "attack",
        cues = { cast = "rouse_cast", loop = "rouse_haste" } },
    },
  }
end
