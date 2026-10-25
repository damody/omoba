//! Current resolved phase only: no target, modifier source, or inferred critical.
use crate::runtime::{AttackSequencePhase, TAttack};
use specs::{Component, Join, WorldExt};
use omoba_sim::Fixed64;
pub const SCHEMA_ID: u32 = 0x464f4712;
pub const RANGE_SCHEMA_ID: u32 = 0x464f4717;
const MAX_RANGE_RAW: i64 = 1_000_000 * 1024;
/// Read-only replica presentation, never an attack capability.
#[derive(Clone,Copy,Debug,Component)]
#[storage(specs::VecStorage)]
pub struct DisclosedAttackRange(pub Fixed64);
pub fn checked_range(range:Fixed64)->Option<Fixed64>{(0..=MAX_RANGE_RAW).contains(&range.raw()).then_some(range)}
pub fn encode_range(range:Fixed64)->Option<Vec<u8>> {
    checked_range(range)?;
    let mut bytes=vec![1];bytes.extend(range.raw().to_be_bytes());Some(bytes)
}
pub fn decode_range(bytes:&[u8])->Option<Fixed64> {
    if bytes.len()!=9||bytes[0]!=1{return None;}
    let raw=i64::from_be_bytes(bytes[1..].try_into().ok()?);
    (0..=MAX_RANGE_RAW).contains(&raw).then(||Fixed64::from_raw(raw))
}
/// One collection per snapshot/WaveB; no private modifier source is exported.
pub fn resolved_attack_ranges(world:&specs::World)->std::collections::BTreeMap<u64,Fixed64> {
    let entities=world.entities();let attacks=world.has_value::<specs::storage::MaskedStorage<TAttack>>().then(||world.read_storage::<TAttack>());
    let buildings=world.has_value::<specs::storage::MaskedStorage<crate::runtime::IsBuilding>>().then(||world.read_storage::<crate::runtime::IsBuilding>());let buffs=world.try_fetch::<crate::runtime::BuffStore>();
    let mut out=std::collections::BTreeMap::new();
    if let Some(attacks)=attacks {for (e,a) in (&entities,&attacks).join(){let range=buffs.as_ref().map_or(a.range.v,|b|
        crate::runtime::UnitStats::from_refs(b,buildings.as_ref().is_some_and(|s|s.get(e).is_some())).final_attack_range(a.range.v,e));
        out.insert(crate::runtime::canonical_entity_id(e),checked_range(range).unwrap_or(Fixed64::ZERO));}}
    if let Some(state)=world.try_fetch::<crate::runtime::MobaMatch>(){out.extend(state.disclosed_npc_attack_ranges(world));}
    if world.has_value::<specs::storage::MaskedStorage<DisclosedAttackRange>>(){
        for (e,range) in (&entities,&world.read_storage::<DisclosedAttackRange>()).join(){out.insert(crate::runtime::canonical_entity_id(e),checked_range(range.0).unwrap_or(Fixed64::ZERO));}
    }
    out
}
const MAX_DURATION_RAW: i64 = 86_400 * 1024;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AttackVisualState {
    pub sequence: u32,
    /// 0 idle, 1 windup, 2 recovery; no fabricated impact event.
    pub phase: u8,
    pub paused: bool,
    pub elapsed_raw: i64,
    pub duration_raw: i64,
}
impl AttackVisualState {
    pub fn from_attack(attack: &TAttack) -> Self {
        let Some(timing) = attack.animation_timing else { return Self::default(); };
        let (windup, backswing) = (timing.windup, timing.backswing);
        let (phase, duration, elapsed) = match attack.attack_phase {
            AttackSequencePhase::Idle => return Self::default(),
            AttackSequencePhase::Windup => (1, windup.raw(), i128::from(windup.raw()) + i128::from(attack.asd_count.raw())),
            AttackSequencePhase::Backswing => (2, backswing.raw(), i128::from(attack.asd_count.raw()) - i128::from(windup.raw())),
        };
        if !(1..=MAX_DURATION_RAW).contains(&duration) { return Self::default(); }
        Self { sequence: attack.attack_seq, phase, paused: timing.paused, elapsed_raw: elapsed.clamp(0, i128::from(duration)) as i64, duration_raw: duration }
    }
    pub fn is_valid(self) -> bool {
        self.phase <= 2 && if self.phase == 0 { self == Self::default() }
        else { (1..=MAX_DURATION_RAW).contains(&self.duration_raw) && (0..=self.duration_raw).contains(&self.elapsed_raw) }
    }
    pub fn encode(self) -> Option<Vec<u8>> {
        if !self.is_valid() { return None; }
        let mut bytes = b"AVS2".to_vec();
        bytes.extend(self.sequence.to_le_bytes()); bytes.push(self.phase); bytes.push(u8::from(self.paused));
        bytes.extend(self.elapsed_raw.to_le_bytes()); bytes.extend(self.duration_raw.to_le_bytes());
        Some(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 26 || &bytes[..4] != b"AVS2" || bytes[9] > 1 { return None; }
        let state = Self { sequence:u32::from_le_bytes(bytes[4..8].try_into().ok()?),phase:bytes[8],
            paused: bytes[9] == 1, elapsed_raw:i64::from_le_bytes(bytes[10..18].try_into().ok()?),duration_raw:i64::from_le_bytes(bytes[18..26].try_into().ok()?) };
        state.is_valid().then_some(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_sim::Fixed64;
    #[test]
    fn public_range_closed_codec_read_only_component_and_lifecycle() {
        use specs::Builder;
        for raw in [0,1,256001,MAX_RANGE_RAW]{let r=Fixed64::from_raw(raw);let bytes=encode_range(r).unwrap();assert_eq!(bytes.len(),9);assert_eq!(decode_range(&bytes),Some(r));}
        for raw in [-1,MAX_RANGE_RAW+1,i64::MAX]{assert!(encode_range(Fixed64::from_raw(raw)).is_none());let mut bytes=vec![1];bytes.extend(raw.to_be_bytes());assert!(decode_range(&bytes).is_none());}
        for bytes in [vec![],vec![0;9],vec![1;8],vec![1;10]]{assert!(decode_range(&bytes).is_none());}
        let mut w=specs::World::new();w.register::<DisclosedAttackRange>();
        let e=w.create_entity().with(DisclosedAttackRange(Fixed64::from_i32(250))).build();assert!(!w.has_value::<specs::storage::MaskedStorage<TAttack>>());assert_eq!(resolved_attack_ranges(&w)[&crate::runtime::canonical_entity_id(e)],Fixed64::from_i32(250));
        w.write_storage::<DisclosedAttackRange>().get_mut(e).unwrap().0=Fixed64::from_raw(-1);assert_eq!(resolved_attack_ranges(&w)[&crate::runtime::canonical_entity_id(e)],Fixed64::ZERO);
        w.write_storage::<DisclosedAttackRange>().remove(e);assert!(resolved_attack_ranges(&w).is_empty());w.delete_entity(e).unwrap();assert!(resolved_attack_ranges(&w).is_empty());
    }
    #[test]
    fn attack_visual_state_resolved_timing_codec_and_clear() {
        let mut attack = TAttack::new(Fixed64::ONE, Fixed64::from_i32(8), Fixed64::ONE, Fixed64::ONE);
        attack.begin_attack_windup();
        assert_eq!(AttackVisualState::from_attack(&attack), AttackVisualState::default());
        attack.animation_timing = Some(crate::runtime::AttackAnimationTiming::new((Fixed64::from_raw(100), Fixed64::from_raw(200)), false));
        attack.asd_count = Fixed64::from_raw(-40);
        let windup = AttackVisualState::from_attack(&attack);
        assert_eq!((windup.phase,windup.elapsed_raw,windup.duration_raw),(1,60,100));
        assert_eq!(AttackVisualState::decode(&windup.encode().unwrap()),Some(windup));
        attack.animation_timing.as_mut().unwrap().paused = true;
        let paused = AttackVisualState::from_attack(&attack);
        assert!(paused.paused);
        assert_eq!(AttackVisualState::decode(&paused.encode().unwrap()), Some(paused));
        let mut malformed = paused.encode().unwrap(); malformed[9] = 2;
        assert!(AttackVisualState::decode(&malformed).is_none());
        let mut old = windup.encode().unwrap(); old[..4].copy_from_slice(b"AVS1");
        assert!(AttackVisualState::decode(&old).is_none());
        assert!(!serde_json::to_string(&attack).unwrap().contains("animation_timing"));
        attack.mark_attack_impact(); attack.asd_count = Fixed64::from_raw(150);
        let recovery = AttackVisualState::from_attack(&attack);
        assert_eq!((recovery.phase,recovery.elapsed_raw,recovery.duration_raw),(2,50,200));
        for state in [AttackVisualState {phase:3,..recovery},AttackVisualState {duration_raw:0,..recovery},
            AttackVisualState {elapsed_raw:201,..recovery},AttackVisualState {elapsed_raw:-1,..recovery},
            AttackVisualState {sequence:1,..Default::default()}] { assert!(state.encode().is_none()); }
        for bytes in [vec![], vec![0;25], windup.encode().unwrap()[..24].to_vec()] {assert!(AttackVisualState::decode(&bytes).is_none());}
        attack.clear_attack_sequence(); assert_eq!(AttackVisualState::from_attack(&attack),AttackVisualState::default());
    }
}
