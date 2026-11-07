//! Authority-only fixed-point accuracy policies shared by native attacks.
//! Rolls are keyed by `rules::chance::roll_key` (a hero by its player, other units by entity
//! id), so a team replica that renumbers its heroes rolls the same values.
use omoba_sim::Fixed64;

// Keep operation2 stable: legacy native dodge0/critical1 and projectile
// accuracy20/stun21 are separate deterministic streams.
const OP_SOURCE_MISS:u32=2;
pub(crate) fn source_misses(seed:u64,tick:u32,source:u64,chance:Fixed64)->bool {
    roll(seed,tick,source,chance,OP_SOURCE_MISS)
}
pub(crate) fn target_evades(seed:u64,tick:u32,target:u64,chance:Fixed64)->bool {
    roll(seed,tick,target,chance,0)
}
/// Preserve the legacy native critical operation1; independent of miss2/evasion0.
pub(crate) fn source_crits(seed:u64,tick:u32,source:u64,chance:Fixed64)->bool {
    roll(seed,tick,source,chance,1)
}
fn roll(seed:u64,tick:u32,key:u64,chance:Fixed64,operation:u32)->bool {
    let chance=chance.clamp(Fixed64::ZERO,Fixed64::ONE);
    if chance<=Fixed64::ZERO {return false;}
    let roll=Fixed64::from_raw((crate::runtime::tick_random_u64(seed,u64::from(tick),
        (key<<16)|u64::from(operation))%omoba_sim::fixed::SCALE as u64)as i64);
    roll<chance
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World,WorldExt,Builder};
    #[test]
    fn critical_uses_legacy_operation1_and_checked_endpoints() {
        let mut w=World::new();let source=w.create_entity().build();let mut outcomes=Vec::new();
        for tick in 1..129 {
            assert!(!source_crits(77,tick,u64::from(source.id()),Fixed64::from_raw(-1)));
            assert!(source_crits(77,tick,u64::from(source.id()),Fixed64::from_i32(2)));
            let expected=crate::runtime::tick_random_u64(77,u64::from(tick),(u64::from(source.id())<<16)|1)%1024<512;
            let actual=source_crits(77,tick,u64::from(source.id()),Fixed64::from_raw(512));assert_eq!(actual,expected);outcomes.push(actual);
        }
        assert!(outcomes.contains(&true)&&outcomes.contains(&false));
    }
    #[test]
    fn shared_target_evasion_preserves_operation0_and_checked_endpoints() {
        let mut w=World::new();let _other=w.create_entity().build();let target=w.create_entity().build();let mut results=Vec::new();
        for tick in 1..129 {
            for chance in [Fixed64::from_raw(-1),Fixed64::ZERO] {assert!(!target_evades(91,tick,u64::from(target.id()),chance));}
            for chance in [Fixed64::ONE,Fixed64::from_i32(2)] {assert!(target_evades(91,tick,u64::from(target.id()),chance));}
            let expected=crate::runtime::tick_random_u64(91,u64::from(tick),u64::from(target.id())<<16)%1024<512;
            let actual=target_evades(91,tick,u64::from(target.id()),Fixed64::from_raw(512));assert_eq!(actual,expected);
            assert_eq!(actual,target_evades(91,tick,u64::from(target.id()),Fixed64::from_raw(512)));results.push(actual);
        }
        assert!(results.contains(&true)&&results.contains(&false));
    }
    #[test]
    fn shared_source_miss_preserves_operation2_and_checked_endpoints() {
        let mut w=World::new();let source=w.create_entity().build();
        let mut outcomes=Vec::new();
        for tick in 1..129 {
            for chance in [Fixed64::from_raw(-1),Fixed64::ZERO] {assert!(!source_misses(77,tick,u64::from(source.id()),chance));}
            for chance in [Fixed64::ONE,Fixed64::from_i32(2)] {assert!(source_misses(77,tick,u64::from(source.id()),chance));}
            let expected=crate::runtime::tick_random_u64(77,u64::from(tick),(u64::from(source.id())<<16)|2)%1024<512;
            let actual=source_misses(77,tick,u64::from(source.id()),Fixed64::from_raw(512));assert_eq!(actual,expected);
            assert_eq!(actual,source_misses(77,tick,u64::from(source.id()),Fixed64::from_raw(512)));outcomes.push(actual);
        }
        assert!(outcomes.contains(&true)&&outcomes.contains(&false));
    }
}
