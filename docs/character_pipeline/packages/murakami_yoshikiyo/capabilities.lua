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
    },
    kit_status = "supported",
    catalog_ready = false,
  }
end
