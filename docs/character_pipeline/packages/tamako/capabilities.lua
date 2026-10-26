-- Capabilities the 玉子 kit needs. Statuses mirror the D3 registry
-- (capability-registry.md); the validator cross-checks them.
return function(ctx)
  return {
    schema_version = 2,
    capabilities = {
      { id = "projectile_tracking", status = "supported" },
      { id = "aoe_circle", status = "supported" },
      { id = "unit_damage", status = "supported" },
      { id = "stun", status = "supported" },
      { id = "permanent_stat_bonus", status = "supported" },
      { id = "projectile_linear", status = "supported" },
      { id = "ground_area_dot", status = "supported" },
      { id = "summon_timed", status = "supported" },
      { id = "every_nth_attack", status = "supported" },
      -- W: DamageZoneSpec end (capsule) + grow (laid along the arrow) + linger (one shared burn).
      { id = "ground_area_line", status = "supported", g_package = "G-4" },
      -- R: homing projectiles retarget (TargetLostPolicy::Retarget); the R arrows are
      -- summons, so what still blocks R is the two summon gaps below.
      { id = "retarget_on_death", status = "supported", g_package = "G-1" },
      -- R: summons only pick nearest / owner_focus targets and can be attacked;
      -- source arrows are invulnerable and locked on the cast target.
      { id = "summon_forced_target", status = "supported", g_package = "G-3" },
      { id = "summon_invulnerable", status = "supported", g_package = "G-3" },
    },
    kit_status = "supported",
    catalog_ready = false,
  }
end
