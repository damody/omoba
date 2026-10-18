//! Lua-authored trinket rows. Gameplay numbers stay in Lua.
//!
//! This catalog is not an item, ability, or summoner slot. It does not place
//! entities or grant vision. Parsing may use `f32` for continuous fields.
//! Every accepted continuous value is finite, strictly positive, and quantized
//! once with the same expression as `omoba-template-ids/build.rs` `fixed64_lit`:
//! `(value * 1024.0).round() as i32`. An empty list is rejected. Callers must
//! not default a missing catalog to empty.
//!
//! Direct attack and automatic acquisition are typed policies. They are not
//! inferred from an index named creep, and they are not selected by matching a
//! trinket id string.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Explicit `catalog_id` values. Declaration order does not assign identity.
pub const MAX_MOBA_TRINKETS: usize = 64;
pub const MAX_MOBA_TRINKET_CATALOG_ID: u16 = 1024;
/// Same raw envelope the summoner cast model accepts (`SIM_TERRAIN_COORD_MAX_RAW`).
pub const TRINKET_FIXED64_MAX_RAW: i32 = 1 << 29;
pub const MAX_TRINKET_CHARGES: u16 = 16;
pub const MAX_TRINKET_OWNED_PLACEMENTS: u16 = 16;
pub const MAX_TRINKET_HIT_POINTS: u16 = 10_000;
/// Detection and stealth levels. Zero is a real authored level.
pub const MAX_TRINKET_LEVEL: u16 = 16;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MobaTrinketEntry {
    pub catalog_id: u16,
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub placement_range: f32,
    pub initial_charges: u16,
    pub max_charges: u16,
    pub recharge_seconds: f32,
    pub lifetime_seconds: f32,
    pub sight_radius: f32,
    pub detection_level: u16,
    pub stealth_level: u16,
    pub hit_points: u16,
    pub maximum_owned_placements: u16,
    pub direct_attack: MobaTrinketDirectAttack,
    pub automatic_acquisition: MobaTrinketAutomaticAcquisition,
}

/// Who may directly attack a placed trinket. Add a variant for a future
/// trinket; do not branch on its stable id.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MobaTrinketDirectAttack {
    /// A legitimately disclosed enemy may issue a direct attack.
    DisclosedEnemy,
    /// No direct attack is authored. This does not place or hide an entity.
    NotDirectlyAttackable,
}

/// Whether ordinary automatic acquisition may select a placed trinket.
/// Add a variant for a future trinket; do not branch on its stable id.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MobaTrinketAutomaticAcquisition {
    Disabled,
    /// Authored token only. No acquisition index is consulted in this catalog.
    Enabled,
}

impl MobaTrinketDirectAttack {
    pub fn rust_literal(self) -> &'static str {
        match self {
            Self::DisclosedEnemy => "TrinketDirectAttackConst::DisclosedEnemy",
            Self::NotDirectlyAttackable => "TrinketDirectAttackConst::NotDirectlyAttackable",
        }
    }
}

impl MobaTrinketAutomaticAcquisition {
    pub fn rust_literal(self) -> &'static str {
        match self {
            Self::Disabled => "TrinketAutomaticAcquisitionConst::Disabled",
            Self::Enabled => "TrinketAutomaticAcquisitionConst::Enabled",
        }
    }
}

/// Identical to `fixed64_lit` in `omoba-template-ids/build.rs`.
pub fn fixed64_rust_lit(value: f32) -> String {
    let raw = (value * 1024.0).round() as i32;
    format!("Fixed64::from_raw({raw})")
}

pub fn trinket_authoring_fixed_raw(value: f32) -> Result<i32, String> {
    if !value.is_finite() || value <= 0.0 {
        return Err("trinket balance must be finite and positive".into());
    }
    let scaled = f64::from(value) * 1024.0;
    if !scaled.is_finite() {
        return Err("trinket balance quantization is not finite".into());
    }
    let rounded = scaled.round();
    let max_raw = f64::from(TRINKET_FIXED64_MAX_RAW);
    if !(1.0..=max_raw).contains(&rounded) {
        return Err("trinket balance is outside the fixed-point envelope".into());
    }
    let lit = (value * 1024.0).round() as i32;
    if f64::from(lit) != rounded {
        return Err("trinket balance does not match fixed64_lit quantization".into());
    }
    Ok(lit)
}

pub fn validate_moba_trinkets(trinkets: &[MobaTrinketEntry]) -> Result<(), String> {
    if trinkets.is_empty() || trinkets.len() > MAX_MOBA_TRINKETS {
        return Err("trinket catalog must be a non-empty bounded list".into());
    }
    let mut ids = BTreeSet::new();
    let mut numeric = BTreeSet::new();
    for trinket in trinkets {
        if trinket.catalog_id == 0 || trinket.catalog_id > MAX_MOBA_TRINKET_CATALOG_ID {
            return Err(format!(
                "trinket '{}' catalog_id is outside 1..={MAX_MOBA_TRINKET_CATALOG_ID}",
                trinket.id
            ));
        }
        if !ids.insert(trinket.id.as_str()) || !numeric.insert(trinket.catalog_id) {
            return Err(format!("duplicate trinket identity '{}'", trinket.id));
        }
        validate_trinket(trinket)?;
    }
    Ok(())
}

fn validate_trinket(trinket: &MobaTrinketEntry) -> Result<(), String> {
    let id_ok = !trinket.id.is_empty()
        && trinket.id.len() <= 64
        && trinket
            .id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
    let text_ok = |value: &str, max_chars: usize| {
        let count = value.chars().count();
        (1..=max_chars).contains(&count) && value.chars().all(|ch| !ch.is_control())
    };
    if !id_ok || !text_ok(&trinket.display_name, 64) || !text_ok(&trinket.description, 512) {
        return Err(format!("invalid trinket text '{}'", trinket.id));
    }
    for (field, value) in [
        ("placement_range", trinket.placement_range),
        ("recharge_seconds", trinket.recharge_seconds),
        ("lifetime_seconds", trinket.lifetime_seconds),
        ("sight_radius", trinket.sight_radius),
    ] {
        trinket_authoring_fixed_raw(value)
            .map_err(|error| format!("trinket '{}' {field}: {error}", trinket.id))?;
    }
    if trinket.initial_charges == 0
        || trinket.max_charges == 0
        || trinket.initial_charges > trinket.max_charges
        || trinket.max_charges > MAX_TRINKET_CHARGES
    {
        return Err(format!("trinket '{}' charge limits are zero, inverted, or excessive", trinket.id));
    }
    if trinket.maximum_owned_placements == 0
        || trinket.maximum_owned_placements > MAX_TRINKET_OWNED_PLACEMENTS
    {
        return Err(format!(
            "trinket '{}' owned placement limit is zero or excessive",
            trinket.id
        ));
    }
    if trinket.hit_points == 0 || trinket.hit_points > MAX_TRINKET_HIT_POINTS {
        return Err(format!("trinket '{}' hit_points is zero or excessive", trinket.id));
    }
    if trinket.detection_level > MAX_TRINKET_LEVEL || trinket.stealth_level > MAX_TRINKET_LEVEL {
        return Err(format!("trinket '{}' level is outside 0..={MAX_TRINKET_LEVEL}", trinket.id));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical_template_hash;

    fn training(catalog_id: u16, id: &str) -> MobaTrinketEntry {
        MobaTrinketEntry {
            catalog_id,
            id: id.into(),
            display_name: "偵察燈".into(),
            description: "放置後暫時提供友方視野。電荷有限，用盡後需等待回復。".into(),
            placement_range: 600.0,
            initial_charges: 1,
            max_charges: 2,
            recharge_seconds: 120.0,
            lifetime_seconds: 90.0,
            sight_radius: 900.0,
            detection_level: 0,
            stealth_level: 1,
            hit_points: 3,
            maximum_owned_placements: 3,
            direct_attack: MobaTrinketDirectAttack::DisclosedEnemy,
            automatic_acquisition: MobaTrinketAutomaticAcquisition::Disabled,
        }
    }

    #[test]
    fn moba_trinkets_catalog_identity_bounds_quantization_and_policy_are_strict() {
        let pair = vec![training(1, "training_scout_lantern_v1"), training(2, "second_lantern_v1")];
        validate_moba_trinkets(&pair).unwrap();
        let mut reversed = pair.clone();
        reversed.reverse();
        validate_moba_trinkets(&reversed).unwrap();
        assert_eq!(reversed[0].catalog_id, 2);
        assert_eq!(reversed[0].id, "second_lantern_v1");
        assert_eq!(reversed[1].catalog_id, 1);
        assert_eq!(reversed[1].id, "training_scout_lantern_v1");

        #[derive(Deserialize)]
        struct RequiredCatalog {
            moba_trinkets: Vec<MobaTrinketEntry>,
        }
        assert!(serde_json::from_value::<RequiredCatalog>(serde_json::json!({})).is_err());
        let parsed = serde_json::from_value::<RequiredCatalog>(serde_json::json!({"moba_trinkets": &pair})).unwrap();
        assert_eq!(parsed.moba_trinkets, pair);
        validate_moba_trinkets(&parsed.moba_trinkets).unwrap();
        assert!(validate_moba_trinkets(&[]).is_err());

        let mut too_many = Vec::new();
        for index in 1..=MAX_MOBA_TRINKETS as u16 + 1 {
            too_many.push(training(index, &format!("lantern_{index}")));
        }
        assert!(validate_moba_trinkets(&too_many).is_err());

        let mut duplicated = pair.clone();
        duplicated[1].id = duplicated[0].id.clone();
        assert!(validate_moba_trinkets(&duplicated).is_err());
        duplicated = pair.clone();
        duplicated[1].catalog_id = duplicated[0].catalog_id;
        assert!(validate_moba_trinkets(&duplicated).is_err());
        for catalog_id in [0, MAX_MOBA_TRINKET_CATALOG_ID + 1] {
            let mut bad = pair.clone();
            bad[0].catalog_id = catalog_id;
            assert!(validate_moba_trinkets(&bad).is_err());
        }
        for id in ["", "Lantern", "has-dash", &"a".repeat(65)] {
            let mut bad = pair.clone();
            bad[0].id = id.into();
            assert!(validate_moba_trinkets(&bad).is_err(), "{id}");
        }
        for (name, description) in [("", "ok"), ("ok", ""), ("line\n", "ok"), ("ok", "bad\t"), ("ok", "bad\u{0000}")] {
            let mut bad = pair.clone();
            bad[0].display_name = name.into();
            bad[0].description = description.into();
            assert!(validate_moba_trinkets(&bad).is_err());
        }
        let mut long_name = pair.clone();
        long_name[0].display_name = "名".repeat(65);
        assert!(validate_moba_trinkets(&long_name).is_err());
        let mut long_description = pair.clone();
        long_description[0].description = "述".repeat(513);
        assert!(validate_moba_trinkets(&long_description).is_err());

        let setters: [fn(&mut MobaTrinketEntry, f32); 4] = [
            |entry, value| entry.placement_range = value,
            |entry, value| entry.recharge_seconds = value,
            |entry, value| entry.lifetime_seconds = value,
            |entry, value| entry.sight_radius = value,
        ];
        for value in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0001, 3_000_000.0, 524_288.0625] {
            for assign in setters {
                let mut bad = pair.clone();
                assign(&mut bad[0], value);
                assert!(validate_moba_trinkets(&bad).is_err(), "{value}");
            }
        }
        assert_eq!(trinket_authoring_fixed_raw(1.0 / 1024.0).unwrap(), 1);
        assert_eq!(trinket_authoring_fixed_raw(524_288.0).unwrap(), TRINKET_FIXED64_MAX_RAW);
        assert_eq!(trinket_authoring_fixed_raw(600.0).unwrap(), 614_400);
        assert_eq!(trinket_authoring_fixed_raw(120.0).unwrap(), 122_880);
        assert_eq!(trinket_authoring_fixed_raw(90.0).unwrap(), 92_160);
        assert_eq!(trinket_authoring_fixed_raw(900.0).unwrap(), 921_600);
        assert_eq!(fixed64_rust_lit(600.0), "Fixed64::from_raw(614400)");
        assert!(trinket_authoring_fixed_raw(f32::MAX).is_err());

        for (initial, max) in [(0, 2), (1, 0), (0, 0), (3, 2), (1, MAX_TRINKET_CHARGES + 1)] {
            let mut bad = pair.clone();
            bad[0].initial_charges = initial;
            bad[0].max_charges = max;
            assert!(validate_moba_trinkets(&bad).is_err(), "{initial}/{max}");
        }
        let mut equal_charges = pair.clone();
        equal_charges[0].initial_charges = 2;
        equal_charges[0].max_charges = 2;
        validate_moba_trinkets(&equal_charges).unwrap();
        for placements in [0, MAX_TRINKET_OWNED_PLACEMENTS + 1] {
            let mut bad = pair.clone();
            bad[0].maximum_owned_placements = placements;
            assert!(validate_moba_trinkets(&bad).is_err());
        }
        for hit_points in [0, MAX_TRINKET_HIT_POINTS + 1] {
            let mut bad = pair.clone();
            bad[0].hit_points = hit_points;
            assert!(validate_moba_trinkets(&bad).is_err());
        }
        for level in [MAX_TRINKET_LEVEL + 1] {
            let mut bad = pair.clone();
            bad[0].detection_level = level;
            assert!(validate_moba_trinkets(&bad).is_err());
            bad = pair.clone();
            bad[0].stealth_level = level;
            assert!(validate_moba_trinkets(&bad).is_err());
        }
        let mut zero_levels = pair.clone();
        zero_levels[0].detection_level = 0;
        zero_levels[0].stealth_level = 0;
        validate_moba_trinkets(&zero_levels).unwrap();
        let mut max_levels = pair.clone();
        max_levels[0].detection_level = MAX_TRINKET_LEVEL;
        max_levels[0].stealth_level = MAX_TRINKET_LEVEL;
        max_levels[0].hit_points = MAX_TRINKET_HIT_POINTS;
        max_levels[0].maximum_owned_placements = MAX_TRINKET_OWNED_PLACEMENTS;
        max_levels[0].max_charges = MAX_TRINKET_CHARGES;
        validate_moba_trinkets(&max_levels).unwrap();

        let mut raw = serde_json::to_value(&pair[0]).unwrap();
        raw["direct_attack"] = serde_json::json!("creep");
        assert!(serde_json::from_value::<MobaTrinketEntry>(raw).is_err());
        raw = serde_json::to_value(&pair[0]).unwrap();
        raw["direct_attack"] = serde_json::json!("DisclosedEnemy");
        assert!(serde_json::from_value::<MobaTrinketEntry>(raw).is_err());
        raw = serde_json::to_value(&pair[0]).unwrap();
        raw["automatic_acquisition"] = serde_json::json!("creep");
        assert!(serde_json::from_value::<MobaTrinketEntry>(raw).is_err());
        raw = serde_json::to_value(&pair[0]).unwrap();
        raw["automatic_acquisition"] = serde_json::json!("enabled");
        let enabled: MobaTrinketEntry = serde_json::from_value(raw).unwrap();
        validate_moba_trinkets(&[enabled.clone()]).unwrap();
        assert_eq!(enabled.automatic_acquisition, MobaTrinketAutomaticAcquisition::Enabled);
        raw = serde_json::to_value(&pair[0]).unwrap();
        raw["direct_attack"] = serde_json::json!("not_directly_attackable");
        let unattackable: MobaTrinketEntry = serde_json::from_value(raw).unwrap();
        validate_moba_trinkets(&[unattackable]).unwrap();
        raw = serde_json::to_value(&pair[0]).unwrap();
        raw["creep_index"] = serde_json::json!(true);
        assert!(serde_json::from_value::<MobaTrinketEntry>(raw).is_err());
        raw = serde_json::to_value(&pair[0]).unwrap();
        raw["initial_charges"] = serde_json::json!(1.5);
        assert!(serde_json::from_value::<MobaTrinketEntry>(raw).is_err());
        raw = serde_json::to_value(&pair[0]).unwrap();
        raw["hit_points"] = serde_json::json!(-1);
        assert!(serde_json::from_value::<MobaTrinketEntry>(raw).is_err());
        raw = serde_json::to_value(&pair[0]).unwrap();
        raw["detection_level"] = serde_json::json!(0.0);
        assert!(serde_json::from_value::<MobaTrinketEntry>(raw).is_err());
        assert_eq!(pair[0].direct_attack.rust_literal(), "TrinketDirectAttackConst::DisclosedEnemy");
        assert_eq!(pair[0].automatic_acquisition.rust_literal(), "TrinketAutomaticAcquisitionConst::Disabled");
    }

    #[test]
    fn one_moba_trinket_field_changes_canonical_template_hash() {
        let trinkets = vec![training(1, "training_scout_lantern_v1")];
        let base = serde_json::json!({ "moba_trinkets": trinkets });
        let before = canonical_template_hash(&base).unwrap();
        assert_eq!(before, canonical_template_hash(&base).unwrap());
        let serialized = serde_json::to_value(&trinkets[0]).unwrap();
        for key in [
            "catalog_id",
            "id",
            "display_name",
            "description",
            "placement_range",
            "initial_charges",
            "max_charges",
            "recharge_seconds",
            "lifetime_seconds",
            "sight_radius",
            "detection_level",
            "stealth_level",
            "hit_points",
            "maximum_owned_placements",
            "direct_attack",
            "automatic_acquisition",
        ] {
            assert!(serialized.get(key).is_some(), "{key}");
        }

        let mut cases: Vec<(&str, serde_json::Value)> = Vec::new();
        let mut catalog_id = base.clone();
        catalog_id["moba_trinkets"][0]["catalog_id"] = serde_json::json!(9);
        cases.push(("catalog_id", catalog_id));
        let mut id = base.clone();
        id["moba_trinkets"][0]["id"] = serde_json::json!("training_scout_lantern_v2");
        cases.push(("id", id));
        let mut name = base.clone();
        name["moba_trinkets"][0]["display_name"] = serde_json::json!("其他");
        cases.push(("display_name", name));
        let mut description = base.clone();
        description["moba_trinkets"][0]["description"] = serde_json::json!("其他說明。");
        cases.push(("description", description));
        let mut placement_range = base.clone();
        placement_range["moba_trinkets"][0]["placement_range"] = serde_json::json!(601.0);
        cases.push(("placement_range", placement_range));
        let mut initial_charges = base.clone();
        initial_charges["moba_trinkets"][0]["initial_charges"] = serde_json::json!(2);
        cases.push(("initial_charges", initial_charges));
        let mut max_charges = base.clone();
        max_charges["moba_trinkets"][0]["max_charges"] = serde_json::json!(3);
        cases.push(("max_charges", max_charges));
        let mut recharge = base.clone();
        recharge["moba_trinkets"][0]["recharge_seconds"] = serde_json::json!(121.0);
        cases.push(("recharge_seconds", recharge));
        let mut lifetime = base.clone();
        lifetime["moba_trinkets"][0]["lifetime_seconds"] = serde_json::json!(91.0);
        cases.push(("lifetime_seconds", lifetime));
        let mut sight = base.clone();
        sight["moba_trinkets"][0]["sight_radius"] = serde_json::json!(901.0);
        cases.push(("sight_radius", sight));
        let mut detection = base.clone();
        detection["moba_trinkets"][0]["detection_level"] = serde_json::json!(1);
        cases.push(("detection_level", detection));
        let mut stealth = base.clone();
        stealth["moba_trinkets"][0]["stealth_level"] = serde_json::json!(2);
        cases.push(("stealth_level", stealth));
        let mut hit_points = base.clone();
        hit_points["moba_trinkets"][0]["hit_points"] = serde_json::json!(4);
        cases.push(("hit_points", hit_points));
        let mut placements = base.clone();
        placements["moba_trinkets"][0]["maximum_owned_placements"] = serde_json::json!(4);
        cases.push(("maximum_owned_placements", placements));
        let mut direct_attack = base.clone();
        direct_attack["moba_trinkets"][0]["direct_attack"] = serde_json::json!("none");
        cases.push(("direct_attack", direct_attack));
        let mut automatic = base.clone();
        automatic["moba_trinkets"][0]["automatic_acquisition"] = serde_json::json!("enabled");
        cases.push(("automatic_acquisition", automatic));
        for (field, changed) in cases {
            assert_ne!(before, canonical_template_hash(&changed).unwrap(), "{field}");
        }
    }
}
