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
      { id = "projectile_vs_building", status = "supported" },
      { id = "projectile_on_hit_dot", status = "supported" },
      { id = "cdr_on_cast_hook", status = "supported" },
      -- hero-port-8 w1 generic capabilities (omoba-core).
      { id = "projectile_dodge", status = "supported" },
      { id = "projectile_widening", status = "supported" },
      { id = "discrete_dot", status = "supported" },
      { id = "cooldown_cap_buff", status = "supported" },
      { id = "mana_on_cast", status = "supported" },
      { id = "on_hit_stacking_debuff", status = "supported" },
    },
    kit_status = "supported",
    catalog_ready = false,
  }
end
