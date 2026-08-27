return function(ctx)
  return {
    campaign = {
      id = "FOG_2TEAM_DEMO",
      name = "雙隊戰爭迷霧展示",
      hero_id = "saika_magoichi",
      description = "100 個方格單位與兩位玩家英雄的 selective lockstep 展示",
      difficulty = "normal",
      unlock_requirements = {},
    },
    stages = {
      {
        id = "FOG_DEMO",
        name = "雙隊視野驗收",
        stage_type = "training",
        objectives = ctx.array({}),
        optional_objectives = ctx.array({}),
        scoring = { max_stars = 0, star_thresholds = ctx.array({}), scoring_factors = {} },
        environment = { time_of_day = "day", visibility = 1.0 },
        ui_settings = {
          show_minimap = false,
          show_hero_stats = true,
          show_ability_cooldowns = false,
          show_damage_numbers = false,
          enable_pause = false,
          camera_mode = "free",
        },
      },
    },
  }
end
