return function(ctx)
  return {
    schema_version = 2,
    capabilities = {
      { id = "unit_damage", status = "supported" },
      { id = "root", status = "supported" },
      { id = "timed_stat_buff", status = "supported" },
      { id = "global_ally_target", status = "supported" },
      { id = "aoe_circle", status = "supported" },
      { id = "delayed_effect", status = "supported", override_reason = "延遲 1 秒等於 AbilityCastPoint=1，以施法前搖 cast_time 表達" },
      { id = "max_hp_percent_damage", status = "supported" },
      { id = "heal", status = "supported" },
      { id = "flat_damage_block", status = "supported" },
      { id = "tether_distance_scaled", status = "supported" },
      { id = "ally_area_query", status = "supported" },
      { id = "on_cast_trigger", status = "supported" },
      { id = "extra_action_slot", status = "supported" },
    },
    kit_status = "supported",
    catalog_ready = false,
  }
end
