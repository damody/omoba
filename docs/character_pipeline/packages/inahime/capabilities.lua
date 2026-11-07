return function(ctx)
  return {
    schema_version = 2,
    capabilities = {
      { id = "toggle_mana_per_attack", status = "supported" },
      { id = "on_attack_hook", status = "supported" },
      { id = "slow_debuff", status = "supported" },
      { id = "heal", status = "supported" },
      { id = "timed_stat_buff", status = "supported" },
      { id = "permanent_stat_bonus", status = "supported" },
      { id = "stealth_attack_no_reveal", status = "supported" },
      { id = "no_unit_collision", status = "supported" },
    },
    kit_status = "supported",
    catalog_ready = false,
  }
end
