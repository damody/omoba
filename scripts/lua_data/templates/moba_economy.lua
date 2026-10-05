-- 正式玩法規則；調整後需重建所有 host / script / renderer content。
return function(ctx)
  return { passive_gold_per_second = 2, hero_kill_gold = 300, recall_channel_seconds = 8,
    hero_assist_gold = 100, assist_window_seconds = 10,
    hero_kill_xp = 100, hero_assist_xp = 50,
    lane_creep_xp = 25, lane_xp_radius = 1200,
    base_recovery_hp_per_second = 120, base_recovery_radius = 300,
    mana_regen_per_second = 5, base_recovery_mana_per_second = 60 }
end
