return function(ctx)
  return {
    schema_version = 2,
    capabilities = {
      { id = "aoe_circle", status = "supported" },
      { id = "unit_damage", status = "supported" },
      { id = "stun", status = "supported" },
      { id = "leap", status = "supported" },
      { id = "slow_debuff", status = "supported" },
      { id = "permanent_stat_bonus", status = "supported" },
      { id = "random_target_det", status = "supported" },
      { id = "channel", status = "supported" },
      { id = "aura", status = "supported" },
      { id = "ally_area_buff_query", status = "supported" },
      -- W 猛擊 0.2 秒後、R 每擊 0.45 秒後結算：omoba-core delayed_effect
      -- （caster buff 到期 → AbilityScript::on_delayed_effect）。
      { id = "delayed_effect", status = "supported", g_package = "G-2",
        override_reason = "omoba-core runtime/native/delayed_effect.rs：施放／產生後 N 秒（≤10）以 AbilityScript::on_delayed_effect 結算；施法者死亡時取消" },
      -- R 施法點己方視野：reveal_zone 的 plain sight（detect=false，不顯形）。
      { id = "fow_viewer", status = "supported", g_package = "G-4" },
    },
    kit_status = "supported",
    catalog_ready = false,
  }
end
