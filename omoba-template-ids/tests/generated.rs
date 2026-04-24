//! Validate the build-time generated template id tables.
//!
//! These tests pin the wire contract: const values are part of the protocol —
//! changing them silently shifts ids on the wire and breaks client↔server.

use omoba_template_ids::*;

#[test]
fn tower_consts_sequential_from_one() {
    assert_eq!(TOWER_DART.0, 1);
    assert_eq!(TOWER_TACK.0, 2);
    assert_eq!(TOWER_BOMB.0, 3);
    assert_eq!(TOWER_ICE.0, 4);
}

#[test]
fn hero_consts_independent_namespace() {
    // Hero id 1 != Tower id 1 — separate u16 spaces per category.
    assert_eq!(HERO_SAIKA_MAGOICHI.0, 1);
    assert_eq!(HERO_DATE_MASAMUNE.0, 2);
}

#[test]
fn ability_buff_summon_creep_projectile_allocate() {
    assert_eq!(ABILITY_SNIPER_MODE.0, 1);
    assert_eq!(BUFF_STUN.0, 1);
    assert_eq!(BUFF_SLOW.0, 2);
    assert_eq!(SUMMON_SAIKA_GUNNER.0, 1);
    assert_eq!(CREEP_TRAINING_MAGE.0, 1);
    assert_eq!(PROJECTILE_TACK.0, 3);
}

#[test]
fn forward_lookup_by_name() {
    assert_eq!(tower_by_name("tower_tack"), Some(TOWER_TACK));
    assert_eq!(tower_by_name("nonexistent"), None);
    assert_eq!(hero_by_name("saika_magoichi"), Some(HERO_SAIKA_MAGOICHI));
    assert_eq!(ability_by_name("sniper_mode"), Some(ABILITY_SNIPER_MODE));
    assert_eq!(buff_by_name("stun"), Some(BUFF_STUN));
    assert_eq!(projectile_by_name("saika_shot"), Some(PROJECTILE_SAIKA_SHOT));
}

#[test]
fn reverse_id_str_roundtrip() {
    for s in ["tower_dart", "tower_tack", "tower_bomb", "tower_ice"] {
        let id = tower_by_name(s).expect("known tower");
        assert_eq!(tower_id_str(id), s, "roundtrip fail: {}", s);
    }
    assert_eq!(tower_id_str(TowerId(0)), "");
}

#[test]
fn display_name_lookup() {
    assert_eq!(creep_display(CREEP_TRAINING_MAGE), "訓練法師");
    assert_eq!(hero_display(HERO_SAIKA_MAGOICHI), "雜賀孫市");
    assert_eq!(hero_title(HERO_SAIKA_MAGOICHI), "千里狙擊手");
    assert_eq!(tower_display(TOWER_TACK), "Tack Shooter");
}

#[test]
fn unspecified_id_zero() {
    assert_eq!(TowerId::UNSPECIFIED.0, 0);
    assert_eq!(HeroId::UNSPECIFIED.0, 0);
    assert_eq!(BuffId::UNSPECIFIED.0, 0);
    assert_eq!(ProjectileKindId::UNSPECIFIED.0, 0);
    assert_eq!(tower_display(TowerId::UNSPECIFIED), "");
    assert_eq!(creep_display(CreepId::UNSPECIFIED), "");
}

#[test]
fn projectile_kinds_no_display() {
    // Projectile kinds are visual kinds — no display_name fn generated, only id_str.
    assert_eq!(projectile_id_str(PROJECTILE_TACK), "tack");
    assert_eq!(projectile_id_str(PROJECTILE_BOMB_FRAG), "bomb_frag");
}
