//! Persistent runtime facts the presentation draws beyond units and status icons (work line C):
//! the ground zones an entity holds, the tethers on it, and whether it is an illusion.
//!
//! The authority writes the raw state into the entity's disclosed baseline ([`SCHEMA_ID`],
//! tether sources as canonical ids). The team projector never forwards that component to a
//! replica: each frame it re-sends the state as one team event ([`EVENT_KIND`]) per disclosed
//! entity, with tether sources mapped to that team's replica ids ([`RuntimeFactState::for_team`]).
//! So a team sees a zone only while its caster is disclosed to it, a tether only while both ends
//! are, and the illusion flag only on its own team's units. Keys are salted per team, so they
//! never correlate across teams. Nothing here feeds gameplay or the replica checkpoint.
use omoba_template_ids::{buff_by_name, status_buff_by_key, try_buff_id_str, BuffId};

pub const SCHEMA_ID: u32 = 0x464f4730;
/// Team presentation event (never applied to the replica world); subject = the entity.
pub const EVENT_KIND: u32 = 0x5246_0001;
pub const MAX_ZONES: usize = 16;
pub const MAX_LINKS: usize = 8;
const MAGIC: &[u8; 4] = b"RFS1";

/// A ground zone held by the entity: the disc at `center`, or the capsule to `end`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ZoneFact {
    pub key: u64,
    /// Status (or `ground_damage_zone`) the zone presents as; its name is the cue id.
    pub buff: u16,
    pub center: (i64, i64),
    pub end: (i64, i64),
    pub radius_raw: i64,
}

/// A tether on the entity (the target end). `source` is a canonical id in the authority form
/// and a replica id in the team form, where `source_epoch` is that replica's disclosure epoch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LinkFact {
    pub key: u64,
    pub source: u64,
    pub source_epoch: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeFactState {
    pub zones: Vec<ZoneFact>,
    pub links: Vec<LinkFact>,
    pub illusion: bool,
}

fn fnv(parts: &[&[u8]]) -> u64 {
    parts.iter().flat_map(|part| part.iter()).fold(0xcbf29ce484222325, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3))
}

/// The status a zone buff presents as: its own v3 status, else the generic ground zone.
fn zone_buff(name: &str) -> Option<u16> {
    status_buff_by_key(name).or_else(|| buff_by_name(name)).or_else(|| buff_by_name("ground_damage_zone")).map(|id| id.raw())
}

impl RuntimeFactState {
    /// Authority form from the live buff store; None when the entity has nothing to show.
    pub fn from_store(store: &crate::runtime::BuffStore, entity: specs::Entity) -> Option<Self> {
        use crate::runtime::ability_runtime::damage_zone::{DamageZoneSpec, KEY as ZONE};
        use crate::runtime::native::tether::{Spec as Tether, KEY as TETHER};
        let mut state = Self { illusion: store.has(entity, crate::runtime::native::rules::vocab::state::ILLUSION), ..Self::default() };
        for (name, entry) in store.iter_for(entity) {
            if entry.remaining <= omoba_sim::Fixed64::ZERO { continue; }
            if entry.payload.get(ZONE).is_some() {
                let (Some(spec), Some(buff)) = (DamageZoneSpec::from_payload(&entry.payload), zone_buff(name)) else { continue };
                let center = (spec.center.x.raw(), spec.center.y.raw());
                let end = spec.end.map_or(center, |end| (end.x.raw(), end.y.raw()));
                state.zones.push(ZoneFact { key: fnv(&[b"zone", name.as_bytes()]), buff, center, end, radius_raw: spec.radius.raw() });
            } else if entry.payload.get(TETHER).is_some() {
                let Some(spec) = Tether::from_payload(&entry.payload) else { continue };
                let source = crate::runtime::canonical_entity_id(spec.source());
                state.links.push(LinkFact { key: fnv(&[b"link", name.as_bytes(), &source.to_le_bytes()]), source, source_epoch: 0 });
            }
        }
        state.normalize();
        (!state.is_empty()).then_some(state)
    }

    pub fn is_empty(&self) -> bool { self.zones.is_empty() && self.links.is_empty() && !self.illusion }

    fn normalize(&mut self) {
        self.zones.sort_by_key(|zone| zone.key);
        self.zones.dedup_by_key(|zone| zone.key);
        self.zones.truncate(MAX_ZONES);
        self.links.sort_by_key(|link| link.key);
        self.links.dedup_by_key(|link| link.key);
        self.links.truncate(MAX_LINKS);
    }

    /// What team `team` may see of this entity: `own` when the entity is on that team, and
    /// `source_of` maps a tether source's canonical id to the team's (replica id, epoch) only while
    /// that source is disclosed to it. None when nothing is left.
    pub fn for_team(&self, team: u32, own: bool, source_of: impl Fn(u64) -> Option<(u64, u64)>) -> Option<Self> {
        let salt = team.to_le_bytes();
        let mut state = Self {
            zones: self.zones.iter().map(|zone| ZoneFact { key: fnv(&[&salt, &zone.key.to_le_bytes()]), ..*zone }).collect(),
            links: self.links.iter().filter_map(|link| {
                let (source, source_epoch) = source_of(link.source)?;
                Some(LinkFact { key: fnv(&[&salt, &link.key.to_le_bytes()]), source, source_epoch })
            }).collect(),
            illusion: own && self.illusion,
        };
        state.normalize();
        (!state.is_empty()).then_some(state)
    }

    pub fn encode(&self) -> Option<Vec<u8>> {
        if self.zones.len() > MAX_ZONES || self.links.len() > MAX_LINKS { return None; }
        let mut bytes = MAGIC.to_vec();
        bytes.extend([u8::from(self.illusion), self.zones.len() as u8, self.links.len() as u8]);
        for zone in &self.zones {
            bytes.extend(zone.key.to_le_bytes());
            bytes.extend(zone.buff.to_le_bytes());
            for value in [zone.center.0, zone.center.1, zone.end.0, zone.end.1, zone.radius_raw] { bytes.extend(value.to_le_bytes()); }
        }
        for link in &self.links {
            for value in [link.key, link.source, link.source_epoch] { bytes.extend(value.to_le_bytes()); }
        }
        Some(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        const ZONE: usize = 8 + 2 + 5 * 8;
        const LINK: usize = 3 * 8;
        if bytes.len() < 7 || &bytes[..4] != MAGIC || bytes[4] > 1 { return None; }
        let (zones, links) = (bytes[5] as usize, bytes[6] as usize);
        if zones > MAX_ZONES || links > MAX_LINKS || bytes.len() != 7 + zones * ZONE + links * LINK { return None; }
        let u64_at = |at: usize| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
        let i64_at = |at: usize| i64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
        let mut state = Self { illusion: bytes[4] == 1, ..Self::default() };
        for index in 0..zones {
            let at = 7 + index * ZONE;
            let buff = u16::from_le_bytes([bytes[at + 8], bytes[at + 9]]);
            try_buff_id_str(BuffId(buff))?;
            let v = at + 10;
            let zone = ZoneFact { key: u64_at(at), buff, center: (i64_at(v), i64_at(v + 8)), end: (i64_at(v + 16), i64_at(v + 24)), radius_raw: i64_at(v + 32) };
            if zone.radius_raw <= 0 { return None; }
            state.zones.push(zone);
        }
        for index in 0..links {
            let at = 7 + zones * ZONE + index * LINK;
            state.links.push(LinkFact { key: u64_at(at), source: u64_at(at + 8), source_epoch: u64_at(at + 16) });
        }
        // Canonical bytes only: sorted, unique keys.
        let mut sorted = state.clone();
        sorted.normalize();
        (sorted == state && !state.is_empty()).then_some(state)
    }

    pub fn zone_cue(zone: &ZoneFact) -> &'static str { try_buff_id_str(BuffId(zone.buff)).unwrap_or("ground_damage_zone") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::ability_runtime::damage_zone::DamageZoneSpec;
    use omb_script_abi::types::{DamageKind, DamageProfile};
    use omoba_sim::{Fixed64, Vec2};
    use specs::{Builder, World, WorldExt};

    fn zone(x: i32) -> DamageZoneSpec {
        DamageZoneSpec { end: None, grow: None, linger: None, independent: false, center: Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(5)),
            radius: Fixed64::from_i32(250), amount: Fixed64::ONE, interval: Fixed64::from_raw(512), kind: DamageKind::Magical,
            profile: DamageProfile::FIRE.bits() }
    }

    #[test]
    fn authority_state_reads_zones_tethers_and_illusion_from_buffs() {
        let mut world = World::new();
        let (caster, target, other) = (world.create_entity().build(), world.create_entity().build(), world.create_entity().build());
        let mut store = crate::runtime::BuffStore::new();
        assert_eq!(RuntimeFactState::from_store(&store, caster), None, "nothing to show");
        store.add(caster, "ground_damage_zone", Fixed64::from_i32(3), zone(100).payload());
        let tether = crate::runtime::native::tether::Spec::new(caster.id(), caster.gen().id() as u32, Fixed64::ONE, DamageKind::Magical,
            Fixed64::ONE, Fixed64::from_i32(600));
        store.add(target, "leash", Fixed64::from_i32(4), tether.payload());
        store.add(other, crate::runtime::native::rules::vocab::state::ILLUSION, Fixed64::from_i32(20), serde_json::Value::Null);

        let zones = RuntimeFactState::from_store(&store, caster).unwrap();
        assert_eq!(zones.zones.len(), 1);
        assert_eq!(zones.zones[0].center, (Fixed64::from_i32(100).raw(), Fixed64::from_i32(5).raw()));
        assert_eq!(zones.zones[0].end, zones.zones[0].center, "a disc ends where it starts");
        assert_eq!(RuntimeFactState::zone_cue(&zones.zones[0]), "ground_damage_zone");
        let link = RuntimeFactState::from_store(&store, target).unwrap();
        assert_eq!(link.links[0].source, crate::runtime::canonical_entity_id(caster));
        assert!(RuntimeFactState::from_store(&store, other).unwrap().illusion);
        for state in [&zones, &link] {
            assert_eq!(RuntimeFactState::decode(&state.encode().unwrap()).as_ref(), Some(state));
        }
    }

    #[test]
    fn team_form_salts_keys_maps_sources_and_hides_foreign_illusions() {
        let state = RuntimeFactState {
            zones: vec![ZoneFact { key: 1, buff: buff_by_name("ground_damage_zone").unwrap().raw(), center: (0, 0), end: (0, 0), radius_raw: 1024 }],
            links: vec![LinkFact { key: 2, source: 77, source_epoch: 0 }],
            illusion: true,
        };
        let disclosed = |canonical: u64| (canonical == 77).then_some((9, 3));
        let own = state.for_team(1, true, disclosed).unwrap();
        assert!(own.illusion);
        assert_eq!((own.links[0].source, own.links[0].source_epoch), (9, 3), "the source as this team's replica");
        let other = state.for_team(2, false, disclosed).unwrap();
        assert!(!other.illusion, "only the owner team learns which unit is an illusion");
        assert_ne!(own.zones[0].key, other.zones[0].key, "keys never correlate across teams");
        assert_ne!(own.links[0].key, other.links[0].key);
        let hidden = state.for_team(2, false, |_| None).unwrap();
        assert!(hidden.links.is_empty(), "a tether whose source is hidden stays hidden");
        let only_illusion = RuntimeFactState { illusion: true, ..RuntimeFactState::default() };
        assert_eq!(only_illusion.for_team(2, false, disclosed), None, "nothing left for another team");
    }

    #[test]
    fn decode_rejects_malformed_bytes() {
        let state = RuntimeFactState { links: vec![LinkFact { key: 5, source: 1, source_epoch: 1 }, LinkFact { key: 3, source: 2, source_epoch: 1 }], ..Default::default() };
        let mut unsorted = MAGIC.to_vec();
        unsorted.extend([0, 0, 2]);
        for link in &state.links { for value in [link.key, link.source, link.source_epoch] { unsorted.extend(value.to_le_bytes()); } }
        assert_eq!(RuntimeFactState::decode(&unsorted), None, "rows must be sorted");
        let mut sorted = state.clone();
        sorted.normalize();
        let bytes = sorted.encode().unwrap();
        assert_eq!(RuntimeFactState::decode(&bytes), Some(sorted));
        assert_eq!(RuntimeFactState::decode(&bytes[..bytes.len() - 1]), None);
        assert_eq!(RuntimeFactState::decode(b"RFS1\x00\x00\x00"), None, "an empty state is never sent");
        assert_eq!(RuntimeFactState::decode(b"RFS1\x02\x00\x00"), None);
    }
}
