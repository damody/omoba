-- Summon templates owned by H01 (appended to templates/summons.lua by build).
-- The legacy saika_gunner (formation summons that follow the owner) stays
-- unchanged; source B01E summons a fixed emplacement instead.
return function(ctx)
  return {
    schema_version = 2,
    summons = {
      {
        id = "b01_saika_corps_gunner",
        display_name = "雜賀眾",
        native_visual_profile = "training_ranger",
        -- assumption gunner_stats_from_local: stats follow the local saika_gunner.
        hp = 400, damage = 45, duration = 60,
        move_speed = 0, attack_range = 450, attack_interval = 1.25,
        armor = 2, magic_resistance = 0, collision_radius = 20, vision_radius = 800,
        follow_start_distance = 0, follow_stop_distance = 0,
        target_priority = "nearest",
        immobile = true,
        -- Source: 「會依雜賀的等級上升攻擊 10 點」.
        damage_per_owner_level = 10,
        owner_death = "persist",
      },
    },
  }
end
