-- 本多忠勝: source A07 (current variant), a new hero appended after existing ones.
return function(ctx)
  return {
    schema_version = 2,
    hero = {
      id = "honda_tadakatsu",
      display_name = "本多忠勝",
      title = "戰國最強",
      role = "fighter",
      combat_read = "近戰重槍：短暫物理無敵並把法術傷害化為生命、原地槍擊範圍重減速、被動暴擊，大招 2.25 倍攻速、魔法免疫並對周圍敵人濺射。",
    },
    identity = {
      game_hero_id = "honda_tadakatsu",
      integration = "append",
      rust_module = "A07_honda_tadakatsu",
      source_hero_id = "A07",
      -- hero UnitScript: on_attack_hit applies the 戰國最強 cleave while the ultimate is active.
      unit_script = true,
    },
    attack = { kind = "melee", range = 150, attack_point = 0.3, base_attack_time = 1.8 },
    base_stats = {
      -- assumption base_stats_from_source: the roster supplies the real values (E157).
      base_hp = 550, base_mana = 100, base_damage = 119, base_armor = 5.0, move_speed = 350, turn_speed = 540,
    },
    gameplay = {
      lane = "top",
      strengths = { "短時間物理無敵", "範圍重減速", "大招魔免與濺射" },
      weaknesses = { "沒有位移", "大招前只能靠減速黏人" },
    },
  }
end
