-- H02 伊達政宗: re-specified from source B04 (current variant). Schema v2 gameplay package.
-- Source B02 is a different hero (風魔小太郎); the local Rust module keeps its
-- historical folder name B02_date_masamune, which is only an identifier.
return function(ctx)
  return {
    schema_version = 2,
    hero = {
      id = "date_masamune",
      display_name = "伊達政宗",
      title = "獨眼龍",
      role = "melee_bruiser",
      combat_read = "近戰突進手：衝刺時對沿途 300 範圍內的敵人放出追蹤火焰、圓形火焰暈眩、被動攻速護甲、大招換上種子島鐵砲遠距離攻擊並有機率停頓目標。",
    },
    identity = {
      game_hero_id = "date_masamune",
      integration = "update_existing",
      rust_module = "B02_date_masamune",
      source_hero_id = "B04",
    },
    attack = { kind = "melee", range = 250, attack_point = 0.17, base_attack_time = 1.8 },
    base_stats = {
      -- assumption base_stats_from_local
      base_hp = 680, base_mana = 240, base_damage = 60, base_armor = 2.5, move_speed = 350,
    },
    gameplay = {
      lane = "top_or_jungle",
      strengths = { "衝刺切入", "範圍暈眩", "大招期間遠距離消耗" },
      weaknesses = { "衝刺後沒有第二段位移", "火焰追蹤彈可被拉開距離閃掉" },
    },
  }
end
