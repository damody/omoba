-- 真田幸村: source B06 (current variant), a new hero appended after existing ones.
return function(ctx)
  return {
    schema_version = 2,
    hero = {
      id = "sanada_yukimura",
      display_name = "真田幸村",
      title = "日本第一兵",
      role = "fighter",
      combat_read = "近戰槍兵：振奮周圍友軍攻擊與回復、彈跳炎彈連續暈眩、被動閃避與受指定技時的魔抗護身，大招雙倍攻速、大量生命與全軍加速。",
    },
    identity = {
      game_hero_id = "sanada_yukimura",
      integration = "append",
      rust_module = "B06_sanada_yukimura",
      source_hero_id = "B06",
      -- hero UnitScript: on_tick refreshes the 日本第一武士 team-wide move-speed aura.
      unit_script = true,
    },
    attack = { kind = "melee", range = 150, attack_point = 0.25, base_attack_time = 1.8 },
    base_stats = {
      -- assumption base_stats_from_source: the roster supplies the real values (E157).
      base_hp = 550, base_mana = 100, base_damage = 111, base_armor = 5.0, move_speed = 350, turn_speed = 540,
    },
    gameplay = {
      lane = "mid",
      strengths = { "多目標連續暈眩", "友軍攻擊與回復增益", "大招長時間強化與全軍加速" },
      weaknesses = { "炎突施法距離只有 250", "大招冷卻長、耗魔高" },
    },
  }
end
