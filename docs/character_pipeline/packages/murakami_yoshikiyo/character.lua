-- 村上義清: source B29 (current variant), a new hero appended after existing ones.
return function(ctx)
  return {
    schema_version = 2,
    hero = {
      id = "murakami_yoshikiyo",
      display_name = "村上義清",
      title = "震地的信濃猛將",
      role = "fighter",
      combat_read = "近戰力量型戰士：原地猛擊範圍暈眩、跳躍猛擊後以破軍靈光削減周圍敵軍護甲，被動提升身旁友軍攻擊並增加自身力量，大招在指定範圍內連續落下震波反覆暈眩。",
    },
    identity = {
      game_hero_id = "murakami_yoshikiyo",
      integration = "append",
      rust_module = "B29_murakami_yoshikiyo",
      source_hero_id = "B29",
      -- hero UnitScript: on_tick drives the 破軍靈光 enemy aura of b29_quake_axe.
      unit_script = true,
    },
    attack = { kind = "melee", range = 128, attack_point = 0.3, base_attack_time = 1.8 },
    base_stats = {
      -- assumption base_stats_from_local_melee
      base_hp = 680, base_mana = 240, base_damage = 60, base_armor = 5.0, move_speed = 350, turn_speed = 540,
    },
    gameplay = {
      lane = "top",
      strengths = { "範圍暈眩", "團戰削甲", "長時間引導控場" },
      weaknesses = { "位移只有 300", "大招引導可被打斷", "技能耗魔高" },
    },
  }
end
