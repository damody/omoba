-- Generated-side presentation defaults; Editor appearance settings override them.
return function(ctx)
  return {
    schema_version = 2,
    placeholder = { body = "capsule_melee", team_tint = true, weapon_socket = "weapon_r" },
    animation_states = { idle = "idle", move = "move", attack = "attack", cast = "cast", channel = "cast", death = "death", recall = "idle" },
    abilities = {
      b04_flame_trail_dash = { preview = "swept_path", path_width = 600, animation = "cast",
        cues = { cast = "flame_trail_dash_start", path = "flame_trail_dash_path", projectile = "b04_flame_trail_dash", impact = "flame_trail_hit" } },
      b04_fire_blast = { preview = "point_circle", animation = "cast",
        cues = { cast = "fire_blast_cast", impact = "fire_blast_explode" } },
      b04_flame_swiftness = { preview = "none", animation = "idle", cues = { learn = "flame_swiftness_learn" } },
      b04_tanegashima = { preview = "none", animation = "cast",
        cues = { cast = "tanegashima_equip", loop = "tanegashima_aura", ["end"] = "tanegashima_unequip" },
        attack_visual = "musket_projectile" },
    },
  }
end
