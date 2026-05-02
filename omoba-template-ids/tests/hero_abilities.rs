//! 驗證 templates.json heroes[].abilities[] 透過 build.rs codegen 生成的
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
