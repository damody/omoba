-- H04 阿市: source C17 (current variant), a new hero appended after existing ones.
return function(ctx)
  return {
    schema_version = 2,
    hero = {
      id = "oichi",
      display_name = "阿市",
      title = "戰國的光之祈願",
      role = "support",
      combat_read = "遠程支援：光鍊依距離懲罰敵人或替友軍減傷、全圖單體祈勝增益、點地眩光打擊，大招延遲後同時傷敵與大量治療友軍。",
    },
    identity = {
      game_hero_id = "oichi",
      integration = "append",
      rust_module = "C17_oichi",
      source_hero_id = "C17",
      unit_script = true,
    },
    attack = { kind = "ranged", range = 600, projectile_speed = 1300, attack_point = 0.2, base_attack_time = 1.8 },
    base_stats = {
      -- assumption base_stats_from_training_caster
      base_hp = 550, base_mana = 400, base_damage = 45, base_armor = 3.0, move_speed = 350, turn_speed = 540,
    },
    gameplay = {
      lane = "support",
      strengths = { "全圖增益", "團戰治療", "距離控制" },
      weaknesses = { "沒有位移", "大招有 1 秒延遲" },
    },
  }
end
