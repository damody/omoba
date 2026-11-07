-- H06 淺井長政: source B08 (current variant), a new hero appended after existing ones.
return function(ctx)
  return {
    schema_version = 2,
    hero = {
      id = "asai_nagamasa",
      display_name = "淺井長政",
      title = "一刀流的近江之主",
      role = "fighter",
      combat_read = "近戰刀客：直線刀牙清線與消耗、瞬移斬擊打斷並燒魔，被動攻速與施法疊層，大招長距離巨型刀牙。",
    },
    identity = {
      game_hero_id = "asai_nagamasa",
      integration = "append",
      rust_module = "B08_asai_nagamasa",
      source_hero_id = "B08",
      unit_script = true,
    },
    attack = { kind = "melee", range = 150, attack_point = 0.25, base_attack_time = 1.8 },
    base_stats = {
      -- assumption base_stats_from_local_melee
      base_hp = 680, base_mana = 240, base_damage = 60, base_armor = 5.0, move_speed = 350, turn_speed = 540,
    },
    gameplay = {
      lane = "top",
      strengths = { "長距離直線傷害", "瞬移切入", "燒魔" },
      weaknesses = { "控制很短", "被動疊層需要連續施法" },
    },
  }
end
