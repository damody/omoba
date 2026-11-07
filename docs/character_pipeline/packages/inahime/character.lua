-- H05 稻姬: source A02 (current variant), a new hero appended after existing ones.
return function(ctx)
  return {
    schema_version = 2,
    hero = {
      id = "inahime",
      display_name = "稻姬",
      title = "風中的蠍毒弓",
      role = "marksman",
      combat_read = "遠程 700 射手：切換蠍毒箭每擊扣魔並毒傷減速，風之祈回血並大幅提升攻速，被動減法傷加總傷害，大招長時間隱形且攻擊不現形。",
    },
    identity = {
      game_hero_id = "inahime",
      integration = "append",
      rust_module = "A02_inahime",
      source_hero_id = "A02",
    },
    attack = { kind = "ranged", range = 700, projectile_speed = 2000, attack_point = 0.17, base_attack_time = 1.8 },
    base_stats = {
      -- assumption base_stats_from_training_marksman
      base_hp = 550, base_mana = 300, base_damage = 50, base_armor = 3.0, move_speed = 350, turn_speed = 540,
    },
    gameplay = {
      lane = "bot",
      strengths = { "持續輸出", "攻速爆發", "隱形切入" },
      weaknesses = { "法球耗魔", "沒有硬控" },
    },
  }
end
