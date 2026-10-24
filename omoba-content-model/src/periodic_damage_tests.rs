use super::*;
fn ability()->AbilityDefinition {
    serde_json::from_value(serde_json::json!({"id":"bleed_fixture","max_level":2,"ability_type":"active","cast_type":"instant","target_type":"unit",
        "levels":[{"range":500},{"range":600}],"extras":{"rate":[1.25,2.5],"time":[2.25,4.0]},
        "effects":[{"kind":"damage_over_time_enemy","amount_key":"rate","duration_key":"time","damage_kind":"physical"}]})).unwrap()
}
#[test]
fn dot_declaration_checks_all_ranks_lifetimes_allegiance_and_duplicates() {
    let valid=ability();assert!(validate_ability_progression(&valid).is_ok());
    assert_eq!(unit_target_allegiance(&valid.effects),Ok(UnitTargetAllegiance::Enemy));
    for time in [0.0,-1.0,0.0001,60.01,f32::INFINITY,f32::NAN] {let mut bad=valid.clone();bad.extras.get_mut("time").unwrap()[1]=time;assert!(validate_ability_progression(&bad).is_err());}
    for rate in [-1.0,0.0001,1_000_001.0,f32::NAN] {let mut bad=valid.clone();bad.extras.get_mut("rate").unwrap()[0]=rate;assert!(validate_ability_progression(&bad).is_err());}
    let mut bad=valid.clone();bad.effects.push(bad.effects[0].clone());assert!(validate_ability_progression(&bad).is_err());
    let mut bad=valid.clone();bad.effects.push(AbilityEffect::HealAlly{amount_key:"rate".into()});assert!(validate_ability_progression(&bad).is_err());
    let mut bad=valid.clone();bad.target_type="none".into();assert!(validate_ability_progression(&bad).is_err());
    let mut bad=valid.clone();bad.levels[1].range=0.0;assert!(validate_ability_progression(&bad).is_err());
    let mut bad=valid;bad.extras.remove("time");assert!(validate_ability_progression(&bad).is_err());
}
