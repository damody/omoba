-- 正式玩法規則；調整後需重建所有 host / script / renderer content。
return function(ctx)
  return { passive_gold_per_second = 2, hero_kill_gold = 300, recall_channel_seconds = 8,
    first_kill_bonus_gold = 100,
    -- Original training balance: total earned rewards, never unspent wallets.
    objective_comeback = {starts_seconds=180,activate_gap=1500,release_gap=750,
      tower_gold_per_player=100,epic_gold_per_player=125,activation_pool=1000},
    -- Index zero deaths first; last entry is the persistent floor.
    death_streak_basis_points = {10000, 10000, 9167, 7500, 6250, 5000, 4167, 3334},
    kill_streak_bounty = { threshold = 3, initial_gold = 150, per_kill_gold = 100, max_gold = 700 },
    hero_assist_gold = 100, assist_window_seconds = 10,
    hero_kill_xp = 100, hero_assist_xp = 50,
    lane_creep_xp = 25, lane_xp_radius = 1200, lane_creep_gold = 20,
    base_recovery_hp_per_second = 120, base_recovery_radius = 300,
    mana_regen_per_second = 5, base_recovery_mana_per_second = 60 }
end
