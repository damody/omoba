//! Owner cast cooldown refund (G-4 `cdr_on_cast_hook`). Content declares
//! `cast_cooldown_refund` on an ability; while the hero has it learned, every
//! committed cast (cooldown start or charge use) of one of the hero's other
//! non-ultimate abilities shortens the remaining cooldown of the hero's
//! remaining non-ultimate abilities by the refund ability's rank seconds.
//! Pure function of the hero component and content, applied at serial
//! settlement, so team replicas recompute it from the same cast; opponents
//! receive the settled remaining through `CommittedCooldown`.
use omoba_sim::Fixed64;
use omoba_template_ids::{AbilityTypeC, ability_by_name, active_ability_const};

use super::comp::Hero;

fn basic(id: &str) -> bool {
    ability_by_name(id).and_then(active_ability_const).is_some_and(|a| a.ability_type != AbilityTypeC::Ultimate)
}

/// Total seconds refunded by one cast of `cast`; zero when nothing applies.
pub fn refund_seconds(hero: &Hero, cast: &str) -> Fixed64 {
    if !basic(cast) || !hero.abilities.iter().any(|id| id == cast) { return Fixed64::ZERO; }
    let mut total = 0i64;
    for id in &hero.abilities {
        let rank = hero.get_ability_level(id);
        let Ok(rank) = u8::try_from(rank) else { continue; };
        if rank == 0 { continue; }
        if let Some(seconds) = ability_by_name(id).and_then(active_ability_const).and_then(|a| a.cast_cooldown_refund_at(rank)) {
            total = total.saturating_add(seconds.raw().max(0));
        }
    }
    Fixed64::from_raw(total)
}

/// Loadout slots whose running cooldown this cast shortens, in slot order.
pub fn refunded_slots(hero: &Hero, cast: &str) -> Vec<(usize, String)> {
    if refund_seconds(hero, cast) <= Fixed64::ZERO { return Vec::new(); }
    hero.abilities.iter().enumerate()
        .filter(|(_, id)| id.as_str() != cast && basic(id) && hero.get_cooldown(id) > Fixed64::ZERO)
        .map(|(slot, id)| (slot, id.clone())).collect()
}

/// Applies the refund for one committed cast of `cast`.
pub fn apply(hero: &mut Hero, cast: &str) {
    let by = refund_seconds(hero, cast);
    if by <= Fixed64::ZERO { return; }
    for (_, id) in refunded_slots(hero, cast) { hero.reduce_cooldown(&id, by); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce_and_commit_keep_the_admitted_total() {
        let mut authority = Hero::default();
        authority.start_cooldown("a", Fixed64::from_i32(10));
        authority.tick_cooldowns(Fixed64::from_i32(1));
        assert!(authority.reduce_cooldown("a", Fixed64::from_i32(2)));
        assert_eq!(authority.get_cooldown("a"), Fixed64::from_i32(7));
        assert_eq!(authority.cooldown_total("a", Fixed64::from_i32(10)), Some(Fixed64::from_i32(10)));
        // An opponent replica saw the start, ticked the same second, then the settled value.
        let mut replica = Hero::default();
        replica.start_cooldown("a", Fixed64::from_i32(10));
        replica.tick_cooldowns(Fixed64::from_i32(1));
        replica.commit_cooldown("a", Fixed64::from_i32(7));
        assert_eq!(serde_json::to_value(&replica).unwrap(), serde_json::to_value(&authority).unwrap());
        // Reaching zero ends the cooldown on both sides.
        assert!(authority.reduce_cooldown("a", Fixed64::from_i32(9)));
        replica.commit_cooldown("a", Fixed64::ZERO);
        assert!(authority.ability_cooldowns.is_empty() && authority.ability_cooldown_totals.is_empty());
        assert_eq!(serde_json::to_value(&replica).unwrap(), serde_json::to_value(&authority).unwrap());
        // A fresh start after it ended is not mistaken for a reduction.
        replica.commit_cooldown("a", Fixed64::from_i32(12));
        assert_eq!(replica.cooldown_total("a", Fixed64::from_i32(12)), Some(Fixed64::from_i32(12)));
        assert!(!authority.reduce_cooldown("a", Fixed64::ONE), "nothing running");
    }

    #[test]
    fn heroes_without_a_learned_refund_are_untouched() {
        let mut hero = Hero::default();
        hero.abilities = vec!["not_a_real_ability".into()];
        hero.start_cooldown("not_a_real_ability", Fixed64::from_i32(5));
        assert_eq!(refund_seconds(&hero, "not_a_real_ability"), Fixed64::ZERO);
        apply(&mut hero, "not_a_real_ability");
        assert_eq!(hero.get_cooldown("not_a_real_ability"), Fixed64::from_i32(5));
    }
}
