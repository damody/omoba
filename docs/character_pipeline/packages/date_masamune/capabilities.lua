-- Capabilities the H02 kit needs; statuses mirror the D3 registry (G-1 done).
return function(ctx)
  return {
    schema_version = 2,
    capabilities = {
      { id = "dash_to_point", status = "supported" },
      { id = "random_target_det", status = "supported" },
      { id = "projectile_tracking", status = "supported" },
      { id = "aoe_circle", status = "supported" },
      { id = "stun", status = "supported" },
      { id = "permanent_stat_bonus", status = "supported" },
      { id = "timed_stat_buff", status = "supported" },
      { id = "proc_chance", status = "supported" },
      { id = "attack_range_switch", status = "supported" },
    },
    kit_status = "supported",
    -- Becomes true only after acceptance.md records a real match acceptance.
    catalog_ready = false,
  }
end
