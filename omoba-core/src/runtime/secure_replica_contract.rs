//! Production contract shared by every selective-lockstep producer and consumer.
//!
//! Add newly disclosed component or resource schema IDs here only.  Projectors,
//! server observers, and external client runtimes must not maintain local copies.

use std::collections::BTreeSet;

/// Component schemas that are safe to materialize in a team-filtered replica.
pub fn secure_replica_component_allowlist() -> BTreeSet<u32> {
    BTreeSet::from([
        crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID,
        crate::runtime::DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,
        crate::runtime::DISCLOSED_DEMO_PATROL_COMPONENT_SCHEMA_ID,
        crate::runtime::DISCLOSED_HERO_COMPONENT_SCHEMA_ID,
        crate::runtime::DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID,
        crate::runtime::DISCLOSED_FACING_COMPONENT_SCHEMA_ID,
        crate::runtime::DISCLOSED_TURN_SPEED_COMPONENT_SCHEMA_ID,
        crate::runtime::DISCLOSED_COLLISION_RADIUS_COMPONENT_SCHEMA_ID,
        crate::runtime::DISCLOSED_INVENTORY_COMPONENT_SCHEMA_ID,
        crate::runtime::DISCLOSED_TOWER_COMPONENT_SCHEMA_ID,
        crate::runtime::DISCLOSED_SCRIPT_UNIT_TAG_COMPONENT_SCHEMA_ID,
    ])
}

/// Resource schemas that are safe to materialize in a team-filtered replica.
///
/// The current protocol exposes no resource records. Keeping this API explicit
/// prevents consumers from silently inventing a different resource boundary.
pub fn secure_replica_resource_allowlist() -> BTreeSet<u32> {
    BTreeSet::new()
}
