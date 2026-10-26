return function(ctx)
  return {
    schema_version = 2,
    capabilities = {
      { id = "projectile_linear", status = "supported" },
      { id = "unit_damage", status = "supported" },
      { id = "blink_to_target", status = "supported" },
      { id = "stun", status = "supported" },
      { id = "mana_burn", status = "supported" },
      { id = "permanent_stat_bonus", status = "supported" },
      { id = "stacking_buff", status = "supported" },
      { id = "on_cast_trigger", status = "supported" },
      { id = "extra_action_slot", status = "supported" },
    },
    kit_status = "supported",
    catalog_ready = false,
  }
end
