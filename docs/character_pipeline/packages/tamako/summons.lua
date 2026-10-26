-- Summon templates owned by I1 tamako (appended to templates/summons.lua by build).
-- 百箭齊發 lethal arrows: one template per R rank (attack 50/100/150).
-- assumption hundred_arrows_arrow_unit: arrows hover where they are released
-- (immobile, generic emplacement AI) with attack range 1000 instead of
-- chasing; they are invulnerable and lock the assigned target (summon_target).
local function arrow(rank, damage)
  return {
    id = "c15_lethal_arrow_r" .. rank,
    display_name = "致命箭矢",
    native_visual_profile = "training_ranger",
    hp = 1, damage = damage, duration = 10,
    move_speed = 0, attack_range = 1000, attack_interval = 1,
    armor = 0, magic_resistance = 0, collision_radius = 0, vision_radius = 0,
    follow_start_distance = 0, follow_stop_distance = 0,
    target_priority = "nearest",
    immobile = true,
    invulnerable = true,
    damage_per_owner_level = 0,
    owner_death = "persist",
  }
end

return function(ctx)
  return {
    schema_version = 2,
    summons = { arrow(1, 50), arrow(2, 100), arrow(3, 150) },
  }
end
