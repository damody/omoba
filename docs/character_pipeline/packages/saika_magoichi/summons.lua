-- Summon templates owned by H01 (appended to templates/summons.lua by build).
-- The legacy saika_gunner (formation summons that follow the owner) stays
-- unchanged; source B01E summons B01W_UNIT (雜賀眾) instead.
return function(ctx)
  return {
    schema_version = 2,
    summons = {
      {
        id = "b01_saika_corps_gunner",
        display_name = "雜賀眾",
        native_visual_profile = "training_ranger",
        -- Source B01W_UNIT (npc_units_custom_load.txt:4720-4798) and B01.lua:B01E_CHECK:
        -- hp 500 (:4764), armor 1 (:4737), magic resistance 0 (:4738), AttackRate 1.40 (:4745),
        -- range 725 (:4748), day vision 1400 (:4770; no day/night cycle), lifetime 180 (B01.lua:106).
        -- Attack 50 + 5 x hero level (B01.lua:142-143) = 55 + 5 x (level - 1).
        hp = 500, damage = 55, duration = 180,
        -- Source moves at 280 and is player-controllable (B01.lua:108); player control of
        -- summons is needs_framework (G-3), so the closest existing behaviour is a fixed
        -- emplacement that stands where it was placed and shoots enemies in range.
        move_speed = 0, attack_range = 725, attack_interval = 1.4,
        -- for_no_collision (npc_abilities_custom_load.txt:5259): no unit collision.
        armor = 1, magic_resistance = 0, collision_radius = 0, vision_radius = 1400,
        follow_start_distance = 0, follow_stop_distance = 0,
        target_priority = "nearest",
        immobile = true,
        damage_per_owner_level = 5,
        owner_death = "persist",
      },
    },
  }
end
