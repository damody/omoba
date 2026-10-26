-- H03 阿松: source A34 (current variant), a new hero appended after existing ones.
return function(ctx)
  return {
    schema_version = 2,
    hero = {
      id = "omatsu",
      display_name = "阿松",
      title = "加賀的冰華",
      role = "caster",
      combat_read = "遠程冰系施法者：追蹤寒霜彈結凍單體、冰山轟擊擴散減速、前方冰霜吐息與持續傷害，大招大幅縮短基礎技能冷卻。",
    },
    identity = {
      game_hero_id = "omatsu",
      integration = "append",
      rust_module = "A34_omatsu",
      source_hero_id = "A34",
    },
    attack = { kind = "ranged", range = 600, projectile_speed = 1300, attack_point = 0.25, base_attack_time = 1.8 },
    base_stats = {
      -- assumption base_stats_from_training_caster
      base_hp = 550, base_mana = 400, base_damage = 45, base_armor = 1.0, move_speed = 350, turn_speed = 540,
    },
    gameplay = {
      lane = "mid",
      strengths = { "單體控制", "範圍消耗", "冷卻循環" },
      weaknesses = { "沒有位移", "技能多為短射程" },
    },
  }
end
