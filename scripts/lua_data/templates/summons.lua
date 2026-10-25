return function(ctx)
  local function copy(value)
    if type(value)~='table' then return value end
    local result={};for key,item in pairs(value)do result[key]=copy(item)end;return result
  end
  local visual=copy(ctx.include("templates/moba_native_visuals.lua").training_ranger)
  visual.height_world_units=65
  return {
    {
      id = "saika_gunner",
      ue = {native_only=true,native_visual=visual},
      display_name = "雜賀鐵炮兵",
      hp = 400.0,
      damage = 45.0,
      duration = 60.0,
      move_speed = 320.0,
      attack_range = 450.0,
      attack_interval = 1.25,
      armor = 2.0,
      magic_resistance = 0.0,
      collision_radius = 20.0,
      vision_radius = 800.0,
      follow_start_distance = 750.0,
      follow_stop_distance = 350.0,
      target_priority = "owner_focus",
    },
  }
end
