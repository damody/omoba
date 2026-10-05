//! One-shot presentation payloads contain replica identity only, never a hidden source.
pub const MAX_DAMAGE_CUES_PER_SNAPSHOT: usize = 1024;
pub const MAX_PRESENTATION_CUES_PER_SNAPSHOT: usize = MAX_DAMAGE_CUES_PER_SNAPSHOT;

/// Stable fact identity shared by native event production and catalog lookup.
/// This intentionally preserves the existing FNV-1a wire identity.
pub fn stable_content_fact_id(text: &str) -> u64 {
    text.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

/// Recognized one-shot payloads share delivery/retention policy. Unknown
/// payloads never acquire reliable one-shot semantics by accident.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationCue {
    Damage(DamagePresentationCue),
    Ability(AbilityPresentationCue),
    AbilityArea(AbilityAreaPresentationCue),
    ProjectileImpact(ProjectileImpactPresentationCue),
}

impl PresentationCue {
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        DamagePresentationCue::decode(bytes).map(Self::Damage)
            .or_else(|| AbilityPresentationCue::decode(bytes).map(Self::Ability))
            .or_else(|| AbilityAreaPresentationCue::decode(bytes).map(Self::AbilityArea))
            .or_else(|| ProjectileImpactPresentationCue::decode(bytes).map(Self::ProjectileImpact))
    }

    pub fn from_effect(id: u64, bytes: &[u8], maximum_tick: u64) -> Option<Self> {
        let cue = Self::decode(bytes)?;
        (id >> 32 == cue.tick() && id as u32 != 0 && cue.tick() <= maximum_tick).then_some(cue)
    }

    pub fn tick(self) -> u64 {
        match self { Self::Damage(cue) => cue.tick, Self::Ability(cue) => cue.tick, Self::AbilityArea(cue) => cue.cast.tick, Self::ProjectileImpact(cue) => cue.tick }
    }

    /// The only live entity dependency of the current safe payload formats.
    pub fn disclosed_entity(self) -> (u64, u64) {
        match self {
            Self::Damage(cue) => (cue.target_id, cue.disclosure_epoch),
            Self::Ability(cue) => (cue.caster_id, cue.disclosure_epoch),
            Self::AbilityArea(cue) => (cue.cast.caster_id, cue.cast.disclosure_epoch),
            Self::ProjectileImpact(cue) => (cue.target_id, cue.disclosure_epoch),
        }
    }

    pub fn encode(self) -> Vec<u8> {
        match self { Self::Damage(cue) => cue.encode(), Self::Ability(cue) => cue.encode(), Self::AbilityArea(cue) => cue.encode(), Self::ProjectileImpact(cue) => cue.encode() }
    }

    pub fn from_public_event(tick:u64, event:&crate::game_proto::TeamPublicEvent,
        live_epoch:impl FnOnce(u64)->Option<u64>) -> Option<(u64,Self)> {
        if event.event_kind == crate::runtime::FactKind::ProjectileImpact as u32 {
            ProjectileImpactPresentationCue::from_public_event(tick,event,live_epoch).map(|(id,cue)|(id,Self::ProjectileImpact(cue)))
        } else if event.sanitized_payload.starts_with(b"APC1") {
            AbilityAreaPresentationCue::from_public_event(tick,event,live_epoch).map(|(id,cue)|(id,Self::AbilityArea(cue)))
        } else { AbilityPresentationCue::from_public_event(tick,event,live_epoch).map(|(id,cue)|(id,Self::Ability(cue))) }
    }
}

/// Target-local contact is deliberately distinct from damage and projectile expiry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectileImpactPresentationCue {
    pub tick: u64,
    pub target_id: u64,
    pub disclosure_epoch: u64,
}

impl ProjectileImpactPresentationCue {
    fn valid(self) -> bool {
        self.target_id != 0 && (1..=u64::from(u32::MAX)).contains(&self.disclosure_epoch)
    }

    pub fn from_public_event(tick: u64, event: &crate::game_proto::TeamPublicEvent,
        live_epoch: impl FnOnce(u64) -> Option<u64>) -> Option<(u64, Self)> {
        if event.event_kind != crate::runtime::FactKind::ProjectileImpact as u32
            || event.sanitized_payload != b"HIT1" { return None; }
        let target_id = event.subject.as_ref()?.value;
        let cue = Self { tick, target_id, disclosure_epoch: live_epoch(target_id)? };
        cue.valid().then_some((presentation_effect_id(tick, event.stable_sub_index)?, cue))
    }

    pub fn encode(self) -> Vec<u8> {
        let mut bytes = b"IMP1".to_vec();
        for value in [self.tick, self.target_id, self.disclosure_epoch] { bytes.extend(value.to_le_bytes()); }
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 28 || &bytes[..4] != b"IMP1" { return None; }
        let cue = Self { tick: u64::from_le_bytes(bytes[4..12].try_into().ok()?),
            target_id: u64::from_le_bytes(bytes[12..20].try_into().ok()?),
            disclosure_epoch: u64::from_le_bytes(bytes[20..28].try_into().ok()?) };
        cue.valid().then_some(cue)
    }
}

/// An admitted area effect, distinct from casting identity or relocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AbilityAreaPresentationCue {
    pub cast: AbilityPresentationCue,
    pub center: (i64,i64),
    pub radius_raw:i64,
    pub duration_raw:i64,
}

impl AbilityAreaPresentationCue {
    fn valid(self)->bool {
        self.cast.valid() && self.cast.caster_relocation.is_none()
            && AbilityPresentationCue::valid_point(self.center.0,self.center.1)
            && (1..=10_000*1024).contains(&self.radius_raw)
            && (1..=60*1024).contains(&self.duration_raw)
    }
    pub fn public_payload(ability_id:u64,rank:u32,x:i64,y:i64,radius:i64,duration:i64)->Vec<u8> {
        let mut bytes=b"APC1".to_vec(); bytes.extend(ability_id.to_le_bytes()); bytes.extend(rank.to_le_bytes());
        for value in [x,y,radius,duration] { bytes.extend(value.to_le_bytes()); } bytes
    }
    pub fn from_public_event(tick:u64,event:&crate::game_proto::TeamPublicEvent,
        live_epoch:impl FnOnce(u64)->Option<u64>)->Option<(u64,Self)> {
        let bytes=&event.sanitized_payload;
        if event.event_kind != crate::runtime::FactKind::Ability as u32 || bytes.len()!=48 || &bytes[..4]!=b"APC1" {return None;}
        let caster_id=event.subject.as_ref()?.value;
        let cue=Self {cast:AbilityPresentationCue {tick,caster_id,disclosure_epoch:live_epoch(caster_id)?,
            ability_id:u64::from_le_bytes(bytes[4..12].try_into().ok()?),rank:u32::from_le_bytes(bytes[12..16].try_into().ok()?),caster_relocation:None},
            center:(i64::from_le_bytes(bytes[16..24].try_into().ok()?),i64::from_le_bytes(bytes[24..32].try_into().ok()?)),
            radius_raw:i64::from_le_bytes(bytes[32..40].try_into().ok()?),duration_raw:i64::from_le_bytes(bytes[40..48].try_into().ok()?)};
        cue.valid().then_some((presentation_effect_id(tick,event.stable_sub_index)?,cue))
    }
    pub fn encode(self)->Vec<u8> {
        let mut bytes=b"ARC1".to_vec();
        for value in [self.cast.tick,self.cast.caster_id,self.cast.disclosure_epoch,self.cast.ability_id] {bytes.extend(value.to_le_bytes());}
        bytes.extend(self.cast.rank.to_le_bytes());
        for value in [self.center.0,self.center.1,self.radius_raw,self.duration_raw] {bytes.extend(value.to_le_bytes());} bytes
    }
    pub fn decode(bytes:&[u8])->Option<Self> {
        if bytes.len()!=72 || &bytes[..4]!=b"ARC1" {return None;}
        let cue=Self {cast:AbilityPresentationCue {tick:u64::from_le_bytes(bytes[4..12].try_into().ok()?),
            caster_id:u64::from_le_bytes(bytes[12..20].try_into().ok()?),disclosure_epoch:u64::from_le_bytes(bytes[20..28].try_into().ok()?),
            ability_id:u64::from_le_bytes(bytes[28..36].try_into().ok()?),rank:u32::from_le_bytes(bytes[36..40].try_into().ok()?),caster_relocation:None},
            center:(i64::from_le_bytes(bytes[40..48].try_into().ok()?),i64::from_le_bytes(bytes[48..56].try_into().ok()?)),
            radius_raw:i64::from_le_bytes(bytes[56..64].try_into().ok()?),duration_raw:i64::from_le_bytes(bytes[64..72].try_into().ok()?)};
        cue.valid().then_some(cue)
    }
}

/// Successful cast identity from a filtered public event. Targets and world
/// coordinates are deliberately absent: the current Ability fact does not
/// disclose those fields. ABY2 additionally carries the successful invocation
/// rank; ABY1 remains readable as rank unknown. This is not an input acknowledgement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AbilityPresentationCue {
    pub tick: u64,
    pub caster_id: u64,
    pub disclosure_epoch: u64,
    /// Stable fact ID, not an Unreal catalog index or an entity ID.
    pub ability_id: u64,
    /// Rank captured at successful invocation; zero is explicitly unknown.
    pub rank: u32,
    /// Team-scoped, actually submitted caster relocation, in raw Fixed64 units.
    /// Never an input point, enemy target location, or projectile impact.
    pub caster_relocation: Option<(i64, i64)>,
}

impl AbilityPresentationCue {
    pub fn from_public_event(
        tick: u64,
        event: &crate::game_proto::TeamPublicEvent,
        live_epoch: impl FnOnce(u64) -> Option<u64>,
    ) -> Option<(u64, Self)> {
        if event.event_kind != crate::runtime::FactKind::Ability as u32 { return None; }
        let payload = event.sanitized_payload.as_slice();
        let (ability_id, rank, caster_relocation) = match payload.len() {
            8 => (u64::from_le_bytes(payload.try_into().ok()?), 0, None),
            16 if &payload[..4] == b"ABS2" => (
                u64::from_le_bytes(payload[4..12].try_into().ok()?),
                u32::from_le_bytes(payload[12..16].try_into().ok()?),
                None,
            ),
            32 if &payload[..4] == b"ABS3" => (
                u64::from_le_bytes(payload[4..12].try_into().ok()?),
                u32::from_le_bytes(payload[12..16].try_into().ok()?),
                Some((i64::from_le_bytes(payload[16..24].try_into().ok()?),
                    i64::from_le_bytes(payload[24..32].try_into().ok()?))),
            ),
            _ => return None,
        };
        if payload.len() == 16 && rank == 0 { return None; }
        let caster_id = event.subject.as_ref()?.value;
        let cue = Self {
            tick, caster_id,
            disclosure_epoch: live_epoch(caster_id)?,
            ability_id, rank, caster_relocation,
        };
        if !cue.valid() { return None; }
        Some((presentation_effect_id(tick, event.stable_sub_index)?, cue))
    }

    fn valid(self) -> bool {
        self.caster_id != 0 && self.disclosure_epoch != 0 && self.ability_id != 0
            && self.rank <= i32::MAX as u32
            && self.caster_relocation.is_none_or(|(x, y)| Self::valid_point(x, y))
    }

    /// Only a visible successful cast may disclose this payload. No target,
    /// position or toggle state is implied by the rank extension.
    pub fn public_payload(ability_id: u64, rank: u32) -> Vec<u8> {
        if rank == 0 { return ability_id.to_le_bytes().to_vec(); }
        let mut bytes = b"ABS2".to_vec();
        bytes.extend(ability_id.to_le_bytes());
        bytes.extend(rank.to_le_bytes());
        bytes
    }

    fn valid_point(x: i64, y: i64) -> bool {
        const LIMIT: i64 = 1_000_000 * 1024;
        (-LIMIT..=LIMIT).contains(&x) && (-LIMIT..=LIMIT).contains(&y)
    }

    pub fn public_relocation_payload(ability_id: u64, rank: u32, x: i64, y: i64) -> Vec<u8> {
        // Preserve cast identity if the optional native position is invalid.
        if !Self::valid_point(x, y) { return Self::public_payload(ability_id, rank); }
        let mut bytes = b"ABS3".to_vec();
        bytes.extend(ability_id.to_le_bytes()); bytes.extend(rank.to_le_bytes());
        bytes.extend(x.to_le_bytes()); bytes.extend(y.to_le_bytes());
        bytes
    }

    pub fn encode(self) -> Vec<u8> {
        let mut bytes = if self.caster_relocation.is_some() { b"ABY3".to_vec() }
            else if self.rank == 0 { b"ABY1".to_vec() } else { b"ABY2".to_vec() };
        for value in [self.tick, self.caster_id, self.disclosure_epoch, self.ability_id] {
            bytes.extend(value.to_le_bytes());
        }
        if self.rank != 0 || self.caster_relocation.is_some() { bytes.extend(self.rank.to_le_bytes()); }
        if let Some((x, y)) = self.caster_relocation { bytes.extend(x.to_le_bytes()); bytes.extend(y.to_le_bytes()); }
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let rank = match bytes.len() {
            36 if &bytes[..4] == b"ABY1" => 0,
            40 if &bytes[..4] == b"ABY2" => {
                let rank = u32::from_le_bytes(bytes[36..40].try_into().ok()?);
                if rank == 0 { return None; }
                rank
            },
            56 if &bytes[..4] == b"ABY3" => u32::from_le_bytes(bytes[36..40].try_into().ok()?),
            _ => return None,
        };
        let cue = Self {
            tick: u64::from_le_bytes(bytes[4..12].try_into().ok()?),
            caster_id: u64::from_le_bytes(bytes[12..20].try_into().ok()?),
            disclosure_epoch: u64::from_le_bytes(bytes[20..28].try_into().ok()?),
            ability_id: u64::from_le_bytes(bytes[28..36].try_into().ok()?),
            rank,
            caster_relocation: if bytes.len() == 56 { Some((
                i64::from_le_bytes(bytes[40..48].try_into().ok()?),
                i64::from_le_bytes(bytes[48..56].try_into().ok()?),
            )) } else { None },
        };
        cue.valid().then_some(cue)
    }

    /// The resolver must contain only currently live disclosed replicas, not
    /// remembered ghosts. Consumers also resolve ability_id in their catalog.
    pub fn from_effect(
        effect_id: u64, bytes: &[u8], maximum_tick: u64,
        live_epoch: impl FnOnce(u64) -> Option<u64>,
    ) -> Option<Self> {
        let cue = Self::decode(bytes)?;
        (effect_id >> 32 == cue.tick && effect_id as u32 != 0
            && cue.tick <= maximum_tick
            && live_epoch(cue.caster_id) == Some(cue.disclosure_epoch)).then_some(cue)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamagePresentationCue {
    pub tick: u64,
    pub target_id: u64,
    pub disclosure_epoch: u64,
    pub amount_milli: i64,
}

impl DamagePresentationCue {
    /// Validate the stream identity as well as the typed payload. Snapshot
    /// sequence is not event identity; callers still enforce target disclosure.
    pub fn from_effect(effect_id: u64, bytes: &[u8], maximum_tick: u64) -> Option<Self> {
        let cue = Self::decode(bytes)?;
        (effect_id >> 32 == cue.tick && effect_id as u32 != 0 && cue.tick <= maximum_tick)
            .then_some(cue)
    }

    pub fn encode(self) -> Vec<u8> {
        let mut bytes = b"DMG1".to_vec();
        bytes.extend(self.tick.to_le_bytes());
        bytes.extend(self.target_id.to_le_bytes());
        bytes.extend(self.disclosure_epoch.to_le_bytes());
        bytes.extend(self.amount_milli.to_le_bytes());
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 36 || &bytes[..4] != b"DMG1" { return None; }
        let cue = Self {
            tick: u64::from_le_bytes(bytes[4..12].try_into().ok()?),
            target_id: u64::from_le_bytes(bytes[12..20].try_into().ok()?),
            disclosure_epoch: u64::from_le_bytes(bytes[20..28].try_into().ok()?),
            amount_milli: i64::from_le_bytes(bytes[28..36].try_into().ok()?),
        };
        (cue.target_id != 0 && cue.disclosure_epoch != 0 && cue.amount_milli > 0).then_some(cue)
    }
}

/// Collision-free within one team/view stream. Reject overflow; never truncate an ID.
pub fn presentation_effect_id(tick: u64, sub_index: u32) -> Option<u64> {
    let tick = u32::try_from(tick).ok()?;
    Some((u64::from(tick) << 32) | u64::from(sub_index.checked_add(1)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projectile_impact_cue_is_strict_target_local_and_not_damage() {
        let mut event = crate::game_proto::TeamPublicEvent {
            event_kind: crate::runtime::FactKind::ProjectileImpact as u32,
            subject: Some(crate::game_proto::ReplicaEntityId { value: 4 }),
            sanitized_payload: b"HIT1".to_vec(), stable_sub_index: 2,
        };
        let (id, cue) = PresentationCue::from_public_event(17, &event, |_| Some(3)).unwrap();
        assert_eq!(cue.disclosed_entity(), (4, 3));
        let bytes = cue.encode();
        assert_eq!(bytes.len(), 28);
        assert_eq!(PresentationCue::from_effect(id, &bytes, 17), Some(cue));
        assert!(DamagePresentationCue::decode(&bytes).is_none());
        assert!(PresentationCue::from_effect(id, &bytes, 16).is_none());
        assert!(PresentationCue::from_effect(id + (1 << 32), &bytes, 18).is_none());
        for length in 0..28 { assert!(PresentationCue::decode(&bytes[..length]).is_none()); }
        let mut extra = bytes.clone(); extra.push(0);
        assert!(PresentationCue::decode(&extra).is_none());
        for (offset, value) in [(12, 0_u64), (20, 0), (20, u64::from(u32::MAX) + 1)] {
            let mut bad = bytes.clone(); bad[offset..offset+8].copy_from_slice(&value.to_le_bytes());
            assert!(PresentationCue::decode(&bad).is_none());
        }
        assert!(PresentationCue::from_public_event(17, &event, |_| None).is_none());
        assert!(PresentationCue::from_public_event(17, &event, |_| Some(0)).is_none());
        assert!(PresentationCue::from_public_event(u64::from(u32::MAX)+1, &event, |_| Some(3)).is_none());
        event.event_kind = crate::runtime::FactKind::Projectile as u32;
        assert!(PresentationCue::from_public_event(17, &event, |_| Some(3)).is_none());
        event.event_kind = crate::runtime::FactKind::ProjectileImpact as u32;
        event.sanitized_payload.push(0);
        assert!(PresentationCue::from_public_event(17, &event, |_| Some(3)).is_none());
    }
    #[test]
    fn ability_cue_area_contract_keeps_kind_units_and_strict_validation() {
        let mut event=crate::game_proto::TeamPublicEvent {
            event_kind:crate::runtime::FactKind::Ability as u32,
            subject:Some(crate::game_proto::ReplicaEntityId {value:4}),stable_sub_index:1,
            sanitized_payload:AbilityAreaPresentationCue::public_payload(123,3,0,0,200*1024,1024),
        };
        let (id,cue)=PresentationCue::from_public_event(17,&event,|_|Some(2)).unwrap();
        let PresentationCue::AbilityArea(area)=cue else {panic!("area cannot become cast or relocation")};
        assert_eq!(area.center,(0,0)); assert_eq!(area.radius_raw,200*1024);
        assert_eq!(PresentationCue::from_effect(id,&cue.encode(),17),Some(cue));
        assert!(AbilityPresentationCue::decode(&cue.encode()).is_none());
        assert!(PresentationCue::from_effect(id,&cue.encode(),16).is_none());
        for length in 0..72 {assert!(PresentationCue::decode(&cue.encode()[..length]).is_none());}
        for (offset,value) in [(40,i64::MAX),(48,i64::MIN),(56,0),(56,10_000*1024+1),(64,0),(64,60*1024+1)] {
            let mut invalid=cue.encode();invalid[offset..offset+8].copy_from_slice(&value.to_le_bytes());
            assert!(PresentationCue::decode(&invalid).is_none());
        }
        event.sanitized_payload.push(0);
        assert!(PresentationCue::from_public_event(17,&event,|_|Some(2)).is_none());
    }
    #[test]
    fn ability_cue_relocation_exact_codec_preserves_zero_and_rejects_invalid_points() {
        let mut event = crate::game_proto::TeamPublicEvent {
            event_kind: crate::runtime::FactKind::Ability as u32,
            subject: Some(crate::game_proto::ReplicaEntityId {value: 4}), stable_sub_index: 0,
            sanitized_payload: AbilityPresentationCue::public_relocation_payload(123, 0, 0, 0),
        };
        let (id, cue) = AbilityPresentationCue::from_public_event(17, &event, |_| Some(2)).unwrap();
        assert_eq!(cue.caster_relocation, Some((0, 0)));
        let bytes = cue.encode();
        assert_eq!((&bytes[..4], bytes.len()), (&b"ABY3"[..], 56));
        assert_eq!(PresentationCue::from_effect(id, &bytes, 17), Some(PresentationCue::Ability(cue)));
        for length in 0..56 { assert!(AbilityPresentationCue::decode(&bytes[..length]).is_none()); }
        let mut invalid = bytes.clone(); invalid.push(0);
        assert!(AbilityPresentationCue::decode(&invalid).is_none());
        for offset in [40, 48] {
            let mut invalid = bytes.clone(); invalid[offset..offset + 8].copy_from_slice(&i64::MAX.to_le_bytes());
            assert!(AbilityPresentationCue::decode(&invalid).is_none());
        }
        event.sanitized_payload[16..24].copy_from_slice(&i64::MIN.to_le_bytes());
        assert!(AbilityPresentationCue::from_public_event(17, &event, |_| Some(2)).is_none());
        assert_eq!(AbilityPresentationCue::public_relocation_payload(123, 3, i64::MAX, 0), AbilityPresentationCue::public_payload(123, 3));
    }
    #[test]
    fn ability_cue_rank_versions_are_exact_and_backward_compatible() {
        let mut event = crate::game_proto::TeamPublicEvent {
            event_kind: crate::runtime::FactKind::Ability as u32,
            subject: Some(crate::game_proto::ReplicaEntityId {value: 4}),
            sanitized_payload: AbilityPresentationCue::public_payload(123, 3),
            stable_sub_index: 0,
        };
        let (id, cue) = AbilityPresentationCue::from_public_event(17, &event, |_| Some(2)).unwrap();
        assert_eq!(cue.rank, 3);
        let bytes = cue.encode();
        assert_eq!((&bytes[..4], bytes.len()), (&b"ABY2"[..], 40));
        assert_eq!(PresentationCue::from_effect(id, &bytes, 17), Some(PresentationCue::Ability(cue)));
        for length in 0..40 { assert!(AbilityPresentationCue::decode(&bytes[..length]).is_none()); }
        let mut invalid = bytes.clone(); invalid.push(0);
        assert!(AbilityPresentationCue::decode(&invalid).is_none());
        for rank in [0, u32::MAX] {
            let mut invalid = bytes.clone(); invalid[36..40].copy_from_slice(&rank.to_le_bytes());
            assert!(AbilityPresentationCue::decode(&invalid).is_none());
            event.sanitized_payload = b"ABS2".to_vec();
            event.sanitized_payload.extend(123_u64.to_le_bytes());
            event.sanitized_payload.extend(rank.to_le_bytes());
            assert!(AbilityPresentationCue::from_public_event(17, &event, |_| Some(2)).is_none());
        }
        event.sanitized_payload = AbilityPresentationCue::public_payload(123, 0);
        let (_, legacy) = AbilityPresentationCue::from_public_event(17, &event, |_| Some(2)).unwrap();
        assert_eq!(legacy.rank, 0);
        assert_eq!((&legacy.encode()[..4], legacy.encode().len()), (&b"ABY1"[..], 36));
        assert_eq!(AbilityPresentationCue::decode(&legacy.encode()), Some(legacy));
        event.sanitized_payload = AbilityPresentationCue::public_payload(123, 3);
        event.sanitized_payload[3] = b'3';
        assert!(AbilityPresentationCue::from_public_event(17, &event, |_| Some(2)).is_none());
    }
    #[test]
    fn ability_cue_stable_hash_preserves_existing_wire_identity() {
        assert_eq!(stable_content_fact_id(""), 0xcbf29ce484222325);
        assert_eq!(stable_content_fact_id("hello"), 0xa430d84680aabd0b);
    }
    #[test]
    fn ability_cue_codec_rejects_malformed_identity_and_stale_disclosure() {
        let cue = AbilityPresentationCue {tick: 17, caster_id: 4, disclosure_epoch: 3, ability_id: 123, rank: 0, caster_relocation: None};
        let id = presentation_effect_id(17, 0).unwrap();
        let bytes = cue.encode();
        assert_eq!(AbilityPresentationCue::from_effect(id, &bytes, 17, |_| Some(3)), Some(cue));
        for length in 0..bytes.len() {
            assert!(AbilityPresentationCue::decode(&bytes[..length]).is_none());
        }
        let mut extended = bytes.clone(); extended.push(0);
        assert!(AbilityPresentationCue::decode(&extended).is_none());
        assert!(DamagePresentationCue::decode(&bytes).is_none());
        for offset in [12, 20, 28] {
            let mut invalid = bytes.clone(); invalid[offset..offset + 8].fill(0);
            assert!(AbilityPresentationCue::decode(&invalid).is_none());
        }
        let mut unknown_version = bytes.clone(); unknown_version[3] = b'2';
        assert!(AbilityPresentationCue::decode(&unknown_version).is_none());
        for (effect, max_tick) in [(0, 17), (17 << 32, 17), (id, 16), (presentation_effect_id(18, 0).unwrap(), 18)] {
            assert!(AbilityPresentationCue::from_effect(effect, &bytes, max_tick, |_| Some(3)).is_none());
        }
        assert!(AbilityPresentationCue::from_effect(id, &bytes, 17, |_| Some(4)).is_none());
        assert!(AbilityPresentationCue::from_effect(id, &bytes, 17, |_| None).is_none());
    }

    #[test]
    fn ability_cue_public_event_requires_exact_payload_and_live_caster() {
        let mut event = crate::game_proto::TeamPublicEvent {
            event_kind: crate::runtime::FactKind::Ability as u32,
            subject: Some(crate::game_proto::ReplicaEntityId { value: 4 }),
            sanitized_payload: 123_u64.to_le_bytes().to_vec(), stable_sub_index: 2,
        };
        let (id, cue) = AbilityPresentationCue::from_public_event(17, &event, |id| (id == 4).then_some(3)).unwrap();
        assert_eq!(id, presentation_effect_id(17, 2).unwrap());
        assert_eq!(cue.ability_id, 123);
        assert!(AbilityPresentationCue::from_public_event(17, &event, |_| None).is_none());
        assert!(AbilityPresentationCue::from_public_event(17, &event, |_| Some(0)).is_none());
        assert!(AbilityPresentationCue::from_public_event(u64::from(u32::MAX) + 1, &event, |_| Some(3)).is_none());
        event.stable_sub_index = u32::MAX;
        assert!(AbilityPresentationCue::from_public_event(17, &event, |_| Some(3)).is_none());
        event.stable_sub_index = 2;
        event.sanitized_payload.push(0);
        assert!(AbilityPresentationCue::from_public_event(17, &event, |_| Some(3)).is_none());
        event.sanitized_payload = vec![0; 8];
        assert!(AbilityPresentationCue::from_public_event(17, &event, |_| Some(3)).is_none());
        event.sanitized_payload = 123_u64.to_le_bytes().to_vec();
        event.event_kind = crate::runtime::FactKind::DirectCombat as u32;
        assert!(AbilityPresentationCue::from_public_event(17, &event, |_| Some(3)).is_none());
        event.event_kind = crate::runtime::FactKind::Ability as u32;
        event.subject = None;
        assert!(AbilityPresentationCue::from_public_event(17, &event, |_| Some(3)).is_none());
    }
    #[test]
    fn damage_effect_identity_requires_matching_tick_nonzero_ordinal_and_safe_payload() {
        let cue = DamagePresentationCue { tick: 17, target_id: 4, disclosure_epoch: 3, amount_milli: 12000 };
        let id = presentation_effect_id(17, 0).unwrap();
        assert_eq!(DamagePresentationCue::from_effect(id, &cue.encode(), 17), Some(cue));
        assert!(DamagePresentationCue::from_effect(id, &cue.encode(), 16).is_none());
        assert!(DamagePresentationCue::from_effect(17 << 32, &cue.encode(), 17).is_none());
        assert!(DamagePresentationCue::from_effect(presentation_effect_id(18, 0).unwrap(), &cue.encode(), 18).is_none());
        assert!(DamagePresentationCue::from_effect(0, &cue.encode(), 17).is_none());
        let mut malformed = cue.encode(); malformed.push(0);
        assert!(DamagePresentationCue::from_effect(id, &malformed, 17).is_none());
    }
    #[test]
    fn typed_damage_payload_and_ids_fail_closed() {
        let cue = DamagePresentationCue {tick: 17, target_id: 4, disclosure_epoch: 3, amount_milli: 12000};
        let mut bytes = cue.encode();
        assert_eq!(DamagePresentationCue::decode(&bytes), Some(cue));
        bytes.push(0);
        assert_eq!(DamagePresentationCue::decode(&bytes), None);
        assert!(presentation_effect_id(u64::from(u32::MAX) + 1, 0).is_none());
        assert!(presentation_effect_id(1, u32::MAX).is_none());
        assert_ne!(presentation_effect_id(17, 0), presentation_effect_id(17, 1));
        assert_ne!(presentation_effect_id(17, 0), presentation_effect_id(18, 0));
    }
}
