return function(ctx)
  return {
    schema_version = 2,
    capabilities = {
      { id = "projectile_tracking", status = "supported" },
      { id = "projectile_linear", status = "supported" },
      { id = "unit_damage", status = "supported" },
      { id = "stun", status = "supported" },
      { id = "aoe_circle", status = "supported" },
      { id = "slow_debuff", status = "supported" },
      { id = "permanent_stat_bonus", status = "supported" },
      { id = "cooldown_reduction", status = "supported" },
      { id = "projectile_vs_building", status = "supported" },
      { id = "projectile_on_hit_dot", status = "supported" },
      { id = "cdr_on_cast_hook", status = "supported" },
    },
    kit_status = "supported",
    catalog_ready = false,
  }
end
