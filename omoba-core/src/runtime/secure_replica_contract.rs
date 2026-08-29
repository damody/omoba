//! Production contract shared by every selective-lockstep producer and consumer.
//!
//! Add newly disclosed component or resource schema IDs here only.  Projectors,
//! server observers, and external client runtimes must not maintain local copies.

use std::collections::BTreeSet;

pub const PUBLIC_BLOCKED_REGIONS_NAMESPACE: &str = "map";
pub const PUBLIC_BLOCKED_REGIONS_KEY: &str = "blocked-regions";

pub fn encode_public_blocked_regions(regions: &crate::runtime::BlockedRegions) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&(regions.0.len() as u32).to_be_bytes());
    for region in &regions.0 {
        let name = region.name.as_bytes();
        out.extend_from_slice(&(name.len() as u32).to_be_bytes());
        out.extend_from_slice(name);
        out.extend_from_slice(&(region.points.len() as u32).to_be_bytes());
        for point in &region.points {
            out.extend_from_slice(&point.x.to_bits().to_be_bytes());
            out.extend_from_slice(&point.y.to_bits().to_be_bytes());
        }
    }
    out
}

pub fn decode_public_blocked_regions(bytes: &[u8]) -> Option<crate::runtime::BlockedRegions> {
    fn u32_at(bytes: &[u8], cursor: &mut usize) -> Option<u32> {
        let value = u32::from_be_bytes(bytes.get(*cursor..*cursor + 4)?.try_into().ok()?);
        *cursor += 4;
        Some(value)
    }
    let mut cursor = 0;
    let count = usize::try_from(u32_at(bytes, &mut cursor)?).ok()?;
    let mut regions = Vec::with_capacity(count);
    for _ in 0..count {
        let name_len = usize::try_from(u32_at(bytes, &mut cursor)?).ok()?;
        let name = std::str::from_utf8(bytes.get(cursor..cursor + name_len)?)
            .ok()?
            .to_owned();
        cursor += name_len;
        let point_count = usize::try_from(u32_at(bytes, &mut cursor)?).ok()?;
        let mut points = Vec::with_capacity(point_count);
        for _ in 0..point_count {
            let x = f32::from_bits(u32_at(bytes, &mut cursor)?);
            let y = f32::from_bits(u32_at(bytes, &mut cursor)?);
            if !x.is_finite() || !y.is_finite() {
                return None;
            }
            points.push(vek::Vec2::new(x, y));
        }
        regions.push(crate::runtime::BlockedRegion { name, points });
    }
    (cursor == bytes.len()).then_some(crate::runtime::BlockedRegions(regions))
}

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
