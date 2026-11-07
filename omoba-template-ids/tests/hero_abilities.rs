//! 驗證 templates.lua heroes[].abilities[] 透過 build.rs codegen 生成的
//! `HERO_*_ABILITIES` const 與 `hero_abilities()` lookup 對應正確 ability ids。

use omoba_template_ids::*;

#[test]
fn saika_magoichi_has_4_abilities() {
    // H01: the source B01 current kit fills the slots (the hand-written kit stays as retained_abilities).
    let abs = hero_abilities(HERO_SAIKA_MAGOICHI);
    assert_eq!(abs, [ABILITY_B01_SNIPER_MODE, ABILITY_B01_SAIKA_CORPS, ABILITY_B01_RAIN_IRON_CANNON, ABILITY_B01_THREE_STAGE]);
}

#[test]
fn date_masamune_has_4_abilities() {
    // H02: the source B04 current kit fills the slots.
    let abs = hero_abilities(HERO_DATE_MASAMUNE);
    assert_eq!(abs, [ABILITY_B04_FLAME_TRAIL_DASH, ABILITY_B04_FIRE_BLAST, ABILITY_B04_FLAME_SWIFTNESS, ABILITY_B04_TANEGASHIMA]);
}

#[test]
fn lua_only_training_luminary_has_4_generated_abilities() {
    let abilities = hero_abilities(HERO_TRAINING_LUMINARY);
    assert_eq!(abilities, [ABILITY_LUMEN_BOLT, ABILITY_LUMEN_TOUCH, ABILITY_LUMEN_LANCE, ABILITY_LUMEN_MEND]);
    let stats = hero_stats(HERO_TRAINING_LUMINARY).expect("training hero stats");
    assert_eq!(stats.base_hp, 550);
    assert_eq!(stats.intelligence, 25);
}

#[test]
fn unknown_hero_has_no_abilities() {
    let abs = hero_abilities(HeroId::UNSPECIFIED);
    assert_eq!(abs.len(), 0);
}

#[test]
fn moba_archetypes_append_ids_and_generate_distinct_four_skill_loadouts() {
    assert_eq!(HERO_TRAINING_APPRENTICE.raw(), 4);
    assert_eq!(HERO_TRAINING_VANGUARD.raw(), 5);
    assert_eq!(HERO_TRAINING_RANGER.raw(), 6);
    let vanguard=hero_stats(HERO_TRAINING_VANGUARD).unwrap();
    let ranger=hero_stats(HERO_TRAINING_RANGER).unwrap();
    let mage=hero_stats(HERO_TRAINING_LUMINARY).unwrap();
    assert!(vanguard.base_hp>mage.base_hp && vanguard.base_hp>ranger.base_hp);
    assert!(vanguard.attack_range< mage.attack_range && mage.attack_range<ranger.attack_range);
    assert_eq!((vanguard.primary_attribute,ranger.primary_attribute,mage.primary_attribute),(0,1,2));
    assert_eq!(hero_abilities(HERO_TRAINING_VANGUARD),
        [ABILITY_VANGUARD_STRIKE,ABILITY_VANGUARD_RECOVER,ABILITY_VANGUARD_CRUSH,ABILITY_VANGUARD_RESOLVE]);
    assert_eq!(hero_abilities(HERO_TRAINING_RANGER),
        [ABILITY_RANGER_SHOT,ABILITY_RANGER_PATCH,ABILITY_RANGER_VOLLEY,ABILITY_RANGER_FINISHER]);
    for hero in [HERO_TRAINING_VANGUARD,HERO_TRAINING_RANGER,HERO_TRAINING_LUMINARY] {
        for &id in hero_abilities(hero) {
            let ability=active_ability_const(id).unwrap();
            assert_eq!(ability.max_level,4);
            assert_eq!(ability.levels.len(),4);
            assert!(ability.levels.iter().all(|rank|rank.cooldown>omoba_sim::Fixed64::ZERO));
        }
    }
}

// ===== Phase B: hero / creep / summon stats lookup =====

#[test]
fn saika_magoichi_stats_match_json() {
    // Playable heroes take their unit stats from the source (B01_2.txt), not from authored values.
    let s = hero_stats(HERO_SAIKA_MAGOICHI).expect("saika has stats");
    assert_eq!(s.strength, 34);
    assert_eq!(s.agility, 28);
    assert_eq!(s.intelligence, 21);
    assert_eq!(s.primary_attribute, 1); // 1 = agility
    assert_eq!(s.base_hp, 550);
    assert_eq!(s.base_damage, 98);
    assert_eq!(s.attack_range, Fixed64::from_i32(700));
    assert_eq!(s.level_growth.hp_per_level, Fixed64::ZERO, "HP grows through strength, not flat");
}

#[test]
fn training_mage_creep_stats() {
    let s = creep_stats(CREEP_TRAINING_MAGE).expect("training_mage has stats");
    assert_eq!(s.hp, Fixed64::from_i32(320));
    assert_eq!(s.damage, Fixed64::from_i32(45));
    assert_eq!(s.enemy_type, 0); // 0 = caster
    assert_eq!(s.ai_type, 0); // 0 = defensive
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
