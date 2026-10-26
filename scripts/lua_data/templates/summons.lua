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
    -- Append-only G-3 fixed emplacement: never moves; attack +10 per owner level.
    {
      id = "lumen_sentry_turret",
      ue = {native_only=true,native_visual=visual},
      display_name = "晨光哨塔",
      hp = 350.0,
      damage = 30.0,
      duration = 30.0,
      move_speed = 0.0,
      attack_range = 600.0,
      attack_interval = 1.0,
      armor = 3.0,
      magic_resistance = 0.0,
      collision_radius = 24.0,
      vision_radius = 700.0,
      immobile = true,
      damage_per_owner_level = 10.0,
      owner_death = "expire",
    },
    -- @hero_author:authored-include (scripts/hero_author.lua build): authored summons appended last.
    (function()
      local out = {}
      local visuals = ctx.include('templates/moba_native_visuals.lua')
      for _, summon in ipairs(ctx.include('templates/authored_heroes.lua').summons or {}) do
        local row = {}
        for key, value in pairs(summon) do row[key] = value end
        local profile = row.native_visual_profile
        row.native_visual_profile = nil
        if profile then
          local base = assert(visuals[profile], 'unknown native visual profile ' .. tostring(profile))
          local visual = {}
          for key, value in pairs(base) do visual[key] = value end
          row.ue = { native_only = true, native_visual = visual }
        end
        out[#out + 1] = row
      end
      return table.unpack(out)
    end)(),
  }
end
