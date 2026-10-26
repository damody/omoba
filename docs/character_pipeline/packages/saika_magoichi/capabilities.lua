-- Capabilities the H01 kit needs. Statuses mirror the D3 registry
-- (docs/plans/2026-10-24-omfue-month-mvp/source-index/candidate-requirements.lua);
-- the validator cross-checks them with --requirements.
return function(ctx)
  return {
    schema_version = 2,
    capabilities = {
      { id = "timed_stat_buff", status = "supported" },
      { id = "on_attack_hook", status = "supported" },
      { id = "target_class_multiplier", status = "supported" },
      { id = "summon_timed", status = "supported" },
      -- G-2 charges and G-3 immobile summons landed (work-G2.md).
      { id = "charges", status = "supported" },
      { id = "summon_immobile", status = "supported" },
      { id = "vision_radius_bonus", status = "supported",
        override_reason = "D3 登錄為 deferred；本 kit 中夜晚視野不適用（無日夜循環），記為 assumption night_vision_not_applicable" },
    },
    kit_status = "supported",
    -- Only a fully supported and accepted kit may enter the formal selection catalog.
    catalog_ready = false,
  }
end
