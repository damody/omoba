//! 驗證 templates.lua heroes[].abilities[] 透過 build.rs codegen 生成的
//! `HERO_*_ABILITIES` const 與 `hero_abilities()` lookup 對應正確 ability ids。

use omoba_template_ids::*;

#[test]
fn saika_magoichi_has_4_abilities() {
    let abs = hero_abilities(HERO_SAIKA_MAGOICHI);
    assert_eq!(abs.len(), 4);
    assert_eq!(abs[0], ABILITY_SNIPER_MODE);
    assert_eq!(abs[1], ABILITY_SAIKA_REINFORCEMENTS);
    assert_eq!(abs[2], ABILITY_RAIN_IRON_CANNON);
    assert_eq!(abs[3], ABILITY_THREE_STAGE_TECHNIQUE);
}

#[test]
fn date_masamune_has_4_abilities() {
    let abs = hero_abilities(HERO_DATE_MASAMUNE);
    assert_eq!(abs.len(), 4);
    assert_eq!(abs[0], ABILITY_FLAME_BLADE);
    assert_eq!(abs[1], ABILITY_FIRE_DASH);
    assert_eq!(abs[2], ABILITY_FLAME_ASSAULT);
    assert_eq!(abs[3], ABILITY_MATCHLOCK_GUN);
}

#[test]
fn unknown_hero_has_no_abilities() {
    let abs = hero_abilities(HeroId::UNSPECIFIED);
    assert_eq!(abs.len(), 0);
}

// ===== Phase B: hero / creep / summon stats lookup =====

#[test]
fn saika_magoichi_stats_match_json() {
    let s = hero_stats(HERO_SAIKA_MAGOICHI).expect("saika has stats");
    assert_eq!(s.strength, 18);
    assert_eq!(s.agility, 28);
    assert_eq!(s.intelligence, 16);
    assert_eq!(s.primary_attribute, 1); // 1 = agility
    assert_eq!(s.base_hp, 580);
    assert_eq!(s.base_damage, 52);
    assert_eq!(s.attack_range, Fixed64::from_i32(900));
    assert_eq!(s.level_growth.hp_per_level, Fixed64::from_i32(58));
}

#[test]
fn training_mage_creep_stats() {
    let s = creep_stats(CREEP_TRAINING_MAGE).expect("training_mage has stats");
    assert_eq!(s.hp, Fixed64::from_i32(320));
    assert_eq!(s.damage, Fixed64::from_i32(45));
    assert_eq!(s.enemy_type, 0); // 0 = caster
    assert_eq!(s.ai_type, 0);    // 0 = defensive
    assert_eq!(s.exp_reward, 80);
    assert_eq!(s.gold_reward, 45);
}

#[test]
fn saika_gunner_summon_stats() {
    let s = summon_stats(SUMMON_SAIKA_GUNNER).expect("saika_gunner has stats");
    assert_eq!(s.hp, Fixed64::from_i32(400));
    assert_eq!(s.damage, Fixed64::from_i32(45));
    assert_eq!(s.duration, Fixed64::from_i32(60));
    assert_eq!(s.move_speed, Fixed64::from_i32(320));
}

#[test]
fn unknown_unit_returns_none() {
    assert!(hero_stats(HeroId::UNSPECIFIED).is_none());
    assert!(creep_stats(CreepId::UNSPECIFIED).is_none());
    assert!(summon_stats(SummonId::UNSPECIFIED).is_none());
}
