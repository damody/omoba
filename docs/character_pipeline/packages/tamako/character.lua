-- 玉子: source C15 (current variant), a new hero appended after existing ones.
return function(ctx)
  return {
    schema_version = 2,
    hero = {
      id = "tamako",
      display_name = "玉子",
      title = "百箭的星弓",
      role = "marksman",
      combat_read = "遠程 700 射手：對地面範圍內每個敵人各射一支追蹤箭並暈眩，爆裂箭沿直線留下長時間火焰，被動提高攻擊與靈活，大招暈眩目標並放出 10 支致命箭矢。",
    },
    identity = {
      game_hero_id = "tamako",
      integration = "append",
      rust_module = "C15_tamako",
      source_hero_id = "C15",
      -- hero UnitScript: on_attack_hit counts landed attacks for the 百箭齊發 passive.
      unit_script = true,
    },
    attack = { kind = "ranged", range = 700, projectile_speed = 1800, attack_point = 0.2, base_attack_time = 1.8 },
    base_stats = {
      -- assumption base_stats_from_local_marksman
      base_hp = 550, base_mana = 300, base_damage = 50, base_armor = 3.0, move_speed = 350, turn_speed = 540,
    },
    gameplay = {
      lane = "bot",
      strengths = { "範圍多目標暈眩", "直線區域封鎖", "大招單體爆發" },
      weaknesses = { "沒有位移", "技能耗魔高" },
    },
  }
end
