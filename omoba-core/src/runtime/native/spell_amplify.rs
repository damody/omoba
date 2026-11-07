//! Spell amplification (E188, Dota MODIFIER_PROPERTY_SPELL_AMPLIFY_PERCENTAGE, e.g. the
//! history_nobu spell books and 仕手能靴 "增加 x% 技能及道具傷害"). The source rule is the
//! engine's: only damage with an inflictor ability is amplified.
//! * A packet is amplified when its profile carries `DamageProfile::SPELL`. The source's
//!   datadriven `Damage` actions (OnSpellStart, modifier OnIntervalThink) and
//!   `ApplyDamage{ability=...}` have an inflictor; `AMHC:Damage` (amhc.lua:535-583 calls
//!   ApplyDamage without `ability`), normal attacks and PerformAttack do not, so their packets
//!   never set the bit and are not amplified.
//! * Any damage type (physical, magical, pure) is scaled.
//! * The sources' `spell_amplify_percentage` (fractions) add up, as Dota sums its sources, and
//!   the packet is scaled by 1 + sum before the victim's immunity and modifiers (all
//!   multiplicative, so the order only matters for rounding).
//! It is a BuffStore stat like any other modifier, so item passives, auras and skills can grant
//! it, and team replicas compute the same packet from the same disclosed buffs.
use omb_script_abi::stat_keys::StatKey;
use omb_script_abi::types::DamageProfile;
use omoba_sim::Fixed64;
use specs::Entity;

use super::ability_runtime::BuffStore;

/// The summed amplification of `source` (a fraction; never below -1).
pub fn amplification(buffs: &BuffStore, source: Entity) -> Fixed64 {
    buffs.sum_add(source, StatKey::SpellAmplifyPercentage).max(-Fixed64::ONE)
}

/// The profile without the SPELL mark, as TD layers and presentation know it.
pub const fn type_bits(profile: u32) -> u32 {
    if profile & DamageProfile::TYPE_BITS != 0 { profile & !DamageProfile::SPELL.bits() } else { profile }
}

/// The packet scaled by the source's amplification when the profile is a spell's.
pub fn apply(buffs: &BuffStore, source: Entity, profile: u32, physical: Fixed64, magical: Fixed64, pure: Fixed64)
    -> (Fixed64, Fixed64, Fixed64) {
    if profile & DamageProfile::SPELL.bits() == 0 { return (physical, magical, pure); }
    let amp = amplification(buffs, source);
    if amp == Fixed64::ZERO { return (physical, magical, pure); }
    let scale = Fixed64::ONE + amp;
    (physical * scale, magical * scale, pure * scale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn sources_add_up_and_only_spell_packets_are_amplified() {
        let mut world = World::new();
        let hero = world.create_entity().build();
        let mut buffs = BuffStore::new();
        let spell = DamageProfile::ENERGY.union(DamageProfile::SPELL).bits();
        let hundred = Fixed64::from_i32(100);
        assert_eq!(apply(&buffs, hero, spell, hundred, hundred, hundred), (hundred, hundred, hundred), "no amplification");
        // Two books: 4.5% + 6% = 10.5%.
        buffs.add(hero, "item_passive_a", Fixed64::from_i32(9999), serde_json::json!({"spell_amplify_percentage": 0.045}));
        buffs.add(hero, "item_passive_b", Fixed64::from_i32(9999), serde_json::json!({"spell_amplify_percentage": 0.06}));
        let amp = amplification(&buffs, hero);
        assert!((amp.to_f32_for_render() - 0.105).abs() < 0.002, "{amp:?}");
        let (p, m, r) = apply(&buffs, hero, spell, hundred, Fixed64::from_i32(200), Fixed64::ZERO);
        assert!(p > hundred && m > Fixed64::from_i32(200) && r == Fixed64::ZERO, "every type scales: {p:?} {m:?}");
        assert!((m.to_f32_for_render() - 221.0).abs() < 0.5, "{m:?}");
        // Attacks and inflictor-less script damage (AMHC:Damage) carry no SPELL mark.
        let attack = DamageProfile::NORMAL.bits();
        assert_eq!(apply(&buffs, hero, attack, hundred, hundred, hundred), (hundred, hundred, hundred));
        assert_eq!(type_bits(spell), DamageProfile::ENERGY.bits());
        assert_eq!(type_bits(attack), attack);
    }
}
