return function(ctx)
  return {
    schema_version = 2,
    capabilities = {
      { id = "bounce_chain", status = "supported",
        override_reason = "本批補上的通用能力（E163）：ability_projectile Spec.bounce（命中後在 radius 內決定性隨機挑另一個可見敵方英雄或一般單位，只排除當下目標，總命中上限）" },
      { id = "spell_guard", status = "supported",
        override_reason = "本批新增的通用能力（E163）：host 在敵方指定技能施放成立時，依目標身上的 spell_guard 項目套魔抗並開始冷卻" },
      { id = "ally_area_buff_query", status = "supported" },
      { id = "timed_stat_buff", status = "supported" },
      { id = "projectile_tracking", status = "supported" },
      { id = "unit_damage", status = "supported" },
      { id = "stun", status = "supported" },
      { id = "evasion", status = "supported" },
      { id = "permanent_stat_bonus", status = "supported" },
      { id = "heal", status = "supported" },
      { id = "aura", status = "supported" },
      { id = "hero_unit_hooks", status = "supported" },
    },
    kit_status = "supported",
    catalog_ready = false,
  }
end
