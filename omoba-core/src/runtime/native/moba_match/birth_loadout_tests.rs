//! A hero row with an empty slot ("", an original hero with F but no D) keeps F in its slot:
//! the empty slot is never learned and refuses a birth rank.
use super::*;

#[test]
fn an_empty_slot_is_skipped_by_the_birth_loadout() {
    let abilities: Vec<String> = omoba_template_ids::hero_abilities(omoba_template_ids::hero_by_name("saika_magoichi").unwrap())
        .iter().take(4).map(|id| id.as_str().to_owned()).collect();
    let mut hero = Hero::default();
    hero.abilities = abilities.clone();
    hero.abilities.push(String::new());
    hero.abilities.push(abilities[0].clone());
    hero.abilities[0] = omoba_template_ids::hero_abilities(omoba_template_ids::hero_by_name("saika_magoichi").unwrap())
        .get(4).map_or_else(|| "lumen_bolt".to_owned(), |id| id.as_str().to_owned());
    let mut loadout = omoba_template_ids::MobaLoadoutConst { ranks: [0; HERO_ABILITY_SLOTS], skill_points: 1 };
    apply_moba_birth_loadout(&mut hero, loadout).unwrap();
    assert!(!hero.ability_levels.contains_key(""), "the empty slot is not an ability");
    assert_eq!(hero.ability_levels.len(), 5);
    loadout.ranks[4] = 1;
    assert!(apply_moba_birth_loadout(&mut hero, loadout).unwrap_err().to_string().contains("empty slot 4"));
}
