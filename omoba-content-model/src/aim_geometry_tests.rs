use super::*;

#[test]
fn active_cone_contract_quantizes_rank_geometry_and_rejects_ambiguous_effects() {
 let mut ability=fixture();ability.aim_geometry=None;
 ability.extras.insert("hit".into(),vec![110.,160.]);
 ability.extras.insert("angle".into(),vec![std::f32::consts::FRAC_PI_6,std::f32::consts::FRAC_PI_2]);
 ability.effects=vec![AbilityEffect::ConeDamage{amount_key:"hit".into(),half_angle_key:"angle".into(),damage_kind:EffectDamageKind::Physical}];
 validate_ability_progression(&ability).unwrap();
 let preview=ability_aim_preview(&ability).unwrap();assert!(!preview.requires_movement);
 assert_eq!(preview.ranks[0].cone_half_angle_ticks,341);assert_eq!(preview.ranks[1].cone_half_angle_ticks,1024);
 assert_eq!(preview.ranks[0].point_radius,0.);assert_eq!(preview.ranks[0].path_width,0.);
 let signature=ability_aim_preview_signature(&ability).unwrap();
 let mut changed=ability.clone();changed.extras.get_mut("angle").unwrap()[0]=0.25;
 assert_ne!(ability_aim_preview_signature(&changed).unwrap(),signature);
 for angle in [0.,-1.,0.0001,1.571,f32::NAN,f32::INFINITY] {
  let mut bad=ability.clone();bad.extras.get_mut("angle").unwrap()[1]=angle;assert!(validate_ability_progression(&bad).is_err(),"{angle}");
 }
 let mut bad=ability.clone();bad.effects.push(AbilityEffect::HealSelf{amount_key:"hit".into()});assert!(validate_ability_progression(&bad).is_err());
 let mut bad=ability.clone();bad.aim_geometry=Some(AbilityAimGeometry::PointCircle{radius_key:"hit".into()});assert!(validate_ability_progression(&bad).is_err());
 for target in ["none","unit"] {let mut bad=ability.clone();bad.target_type=target.into();assert!(validate_ability_progression(&bad).is_err());}
 let mut bad=ability.clone();bad.cast_type="channel".into();assert!(validate_ability_progression(&bad).is_err());
 let mut bad=ability;bad.extras.get_mut("angle").unwrap().pop();assert!(validate_ability_progression(&bad).is_err());
}

#[test]
fn shared_preview_projection_tracks_effective_geometry_and_ignores_nonspatial_tuning() {
 let mut ability=fixture();let before=ability_aim_preview_signature(&ability).unwrap();
 ability.extras.insert("damage".into(),vec![100.,200.]);ability.description="new cosmetic wording".into();
 assert_eq!(ability_aim_preview_signature(&ability).unwrap(),before);
 let mut changed=ability.clone();changed.levels[1].range+=1.;assert_ne!(ability_aim_preview_signature(&changed).unwrap(),before);
 changed=ability.clone();changed.extras.get_mut("actual_width").unwrap()[1]+=1.;assert_ne!(ability_aim_preview_signature(&changed).unwrap(),before);
 let projected=ability_aim_preview(&ability).unwrap();assert_eq!(projected.ranks[1].path_width,210.);assert!(projected.requires_movement);
 ability.aim_geometry=None;ability.extras.insert("small".into(),vec![10.,20.]);ability.extras.insert("large".into(),vec![30.,40.]);
 ability.effects=vec![AbilityEffect::AreaDamage{amount_key:"damage".into(),radius_key:"small".into(),damage_kind:EffectDamageKind::Magical},
  AbilityEffect::AreaDamage{amount_key:"damage".into(),radius_key:"large".into(),damage_kind:EffectDamageKind::Magical}];
 let projected=ability_aim_preview(&ability).unwrap();assert_eq!(projected.ranks[1].point_radius,40.);assert_eq!(projected.ranks[1].path_width,0.);
 assert!(!projected.requires_movement);let before=ability_aim_preview_signature(&ability).unwrap();
 ability.extras.get_mut("large").unwrap()[0]+=1.;assert_ne!(ability_aim_preview_signature(&ability).unwrap(),before);
}

#[test]
fn facing_formation_is_explicit_bounded_and_rank_complete() {
 let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({"id":"unrelated_formation","ability_type":"active",
  "cast_type":"instant","target_type":"point","max_level":2,"levels":[{"range":800},{"range":800}],
  "extras":{"summon_count":[2,8],"row_spacing":[60,60],"col_spacing":[60,60],"front_row_distance":[120,120]},
  "aim_geometry":{"kind":"facing_formation","rows":2}})).unwrap();
 validate_ability_progression(&valid).unwrap();assert!(!ability_requires_movement(&valid));
 assert_eq!(facing_formation_rows(&valid),2);
 let mut four=valid.clone();four.aim_geometry=Some(AbilityAimGeometry::FacingFormation{rows:4});
 four.extras.insert("summon_count".into(),vec![4.,8.]);validate_ability_progression(&four).unwrap();assert_eq!(facing_formation_rows(&four),4);
 let roundtrip:AbilityDefinition=serde_json::from_value(serde_json::to_value(&valid).unwrap()).unwrap();validate_ability_progression(&roundtrip).unwrap();
 for (key,value) in [("summon_count",2.5),("summon_count",3.),("summon_count",130.),("row_spacing",0.),
  ("col_spacing",f32::NAN),("front_row_distance",-1.),("front_row_distance",10_000.)] {
  let mut bad=valid.clone();bad.extras.get_mut(key).unwrap()[1]=value;assert!(validate_ability_progression(&bad).is_err(),"{key} {value}");
 }
 for rows in [0,3,129] {let mut bad=valid.clone();bad.aim_geometry=Some(AbilityAimGeometry::FacingFormation{rows});assert!(validate_ability_progression(&bad).is_err());}
 let mut bad=valid.clone();bad.extras.get_mut("col_spacing").unwrap().pop();assert!(validate_ability_progression(&bad).is_err());
 let mut bad=valid.clone();bad.target_type="unit".into();assert!(validate_ability_progression(&bad).is_err());
 let mut bad=valid;bad.effects.push(AbilityEffect::DashToPoint);assert!(validate_ability_progression(&bad).is_err());
}

fn fixture() -> AbilityDefinition {
    serde_json::from_value(serde_json::json!({
        "id":"arbitrary_dash", "ability_type":"active", "cast_type":"instant", "target_type":"point",
        "max_level":2, "levels":[{"range":900},{"range":1200}],
        "extras":{"actual_width":[150,210]},
        "aim_geometry":{"kind":"swept_path","width_key":"actual_width"}
    })).unwrap()
}

#[test]
fn swept_path_contract_validates_rank_width_range_and_target_without_identity_inference() {
    let valid=fixture(); validate_ability_progression(&valid).unwrap();
    for width in [0.0,-1.0,f32::NAN,f32::INFINITY,5001.0,0.0001] {
        let mut bad=valid.clone(); bad.extras.get_mut("actual_width").unwrap()[1]=width;
        assert!(validate_ability_progression(&bad).is_err(),"width {width}");
    }
    for range in [0.0,10_001.0,f32::NAN] {
        let mut bad=valid.clone();bad.levels[1].range=range;assert!(validate_ability_progression(&bad).is_err());
    }
    let mut bad=valid.clone();bad.extras.get_mut("actual_width").unwrap().pop();assert!(validate_ability_progression(&bad).is_err());
    let mut bad=valid.clone();bad.extras.clear();assert!(validate_ability_progression(&bad).is_err());
    let mut bad=valid.clone();bad.target_type="unit".into();assert!(validate_ability_progression(&bad).is_err());
    let mut bad=valid.clone();bad.cast_type="channel".into();assert!(validate_ability_progression(&bad).is_err());
    let mut plain=valid;plain.aim_geometry=None;plain.extras.clear();validate_ability_progression(&plain).unwrap();
}

#[test]
fn swept_path_contract_rejects_unknown_kinds_fields_and_preserves_roundtrip() {
    let value=serde_json::to_value(fixture()).unwrap();
    let roundtrip:AbilityDefinition=serde_json::from_value(value.clone()).unwrap();
    validate_ability_progression(&roundtrip).unwrap();
    let mut bad=value.clone();bad["aim_geometry"]["kind"]="cosmetic_cone".into();
    assert!(serde_json::from_value::<AbilityDefinition>(bad).is_err());
    let mut bad=value;bad["aim_geometry"]["radius"]=999.into();
    assert!(serde_json::from_value::<AbilityDefinition>(bad).is_err());
}

#[test]
fn movement_requirement_follows_authored_displacement_not_names_or_extra_keys() {
    let mut ability=fixture();assert!(ability_requires_movement(&ability));
    ability.aim_geometry=None;assert!(!ability_requires_movement(&ability));
    ability.id="fire_dash".into();ability.extras.insert("dash_width".into(),vec![150.0,150.0]);
    assert!(!ability_requires_movement(&ability));
    ability.effects.push(AbilityEffect::DashToPoint);
    validate_ability_progression(&ability).unwrap();assert!(ability_requires_movement(&ability));
    ability.effects=vec![AbilityEffect::Damage {amount_key:"hit".into(),damage_kind:EffectDamageKind::Magical}];
    assert!(!ability_requires_movement(&ability));
}

#[test]
fn point_circle_contract_validates_rank_dimensions_and_does_not_require_movement() {
    let mut valid=fixture();valid.aim_geometry=Some(AbilityAimGeometry::PointCircle {radius_key:"actual_width".into()});
    validate_ability_progression(&valid).unwrap();assert!(!ability_requires_movement(&valid));
    let roundtrip:AbilityDefinition=serde_json::from_value(serde_json::to_value(&valid).unwrap()).unwrap();
    validate_ability_progression(&roundtrip).unwrap();
    for value in [0.0,-1.0,10_001.0,f32::NAN,f32::INFINITY,0.0007] {
        let mut bad=valid.clone();bad.extras.get_mut("actual_width").unwrap()[1]=value;
        assert!(validate_ability_progression(&bad).is_err());
    }
    for target in ["none","unit"] {let mut bad=valid.clone();bad.target_type=target.into();assert!(validate_ability_progression(&bad).is_err());}
    let mut bad=valid.clone();bad.extras.get_mut("actual_width").unwrap().pop();assert!(validate_ability_progression(&bad).is_err());
    let mut bad=valid.clone();bad.extras.clear();assert!(validate_ability_progression(&bad).is_err());
    let mut bad=valid;bad.levels[1].range=0.0;assert!(validate_ability_progression(&bad).is_err());
}

#[test]
fn point_circle_rejects_ambiguous_executable_geometry_and_empty_dimension_keys() {
    let mut ability=fixture();ability.aim_geometry=Some(AbilityAimGeometry::PointCircle {radius_key:"actual_width".into()});
    ability.effects.push(AbilityEffect::AreaDamage {amount_key:"hit".into(),radius_key:"other".into(),damage_kind:EffectDamageKind::Physical});
    assert!(validate_ability_progression(&ability).is_err());
    ability.effects.clear();ability.extras.insert("".into(),vec![100.,100.]);
    for geometry in [AbilityAimGeometry::PointCircle {radius_key:"".into()},AbilityAimGeometry::SweptPath {width_key:"".into()}] {
        ability.aim_geometry=Some(geometry);assert!(validate_ability_progression(&ability).is_err());
    }
}
