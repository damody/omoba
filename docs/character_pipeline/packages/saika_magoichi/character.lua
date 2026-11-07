-- H01 雜賀孫市: re-specified from source B01 (current variant). Schema v2 gameplay package.
return function(ctx)
  return {
    schema_version = 2,
    hero = {
      id = "saika_magoichi",
      display_name = "雜賀孫市",
      title = "千里狙擊手",
      role = "marksman",
      combat_read = "遠程鐵炮手：短時間拉長射程的狙擊、定點雜賀眾砲台、依目標類別加成的普攻、大招增傷。",
    },
    identity = {
      game_hero_id = "saika_magoichi",
      -- Existing local hero entry: the package updates its ability list in place
      -- (hero ID unchanged); new abilities are appended after existing ones.
      integration = "update_existing",
      rust_module = "B01_saika_magoichi",
      source_hero_id = "B01",
      depends_on = { summons = { "saika_gunner" } },
    },
    attack = {
      kind = "ranged",
      range = 700,
      projectile_speed = 3000,
      attack_point = 0.17,
      base_attack_time = 1.8,
    },
    base_stats = {
      -- assumption base_stats_from_local: source HP/mana come from Dota attributes.
      base_hp = 580,
      base_mana = 300,
      base_damage = 52,
      base_armor = 1.5,
      move_speed = 350,
    },
    gameplay = {
      lane = "bottom_or_mid",
      strengths = { "長射程消耗", "定點召喚守線", "大招爆發" },
      weaknesses = { "沒有位移", "大招前法力需求高" },
    },
  }
end
