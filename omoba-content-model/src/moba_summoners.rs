//! Lua-authored summoner spell rows. Gameplay numbers stay in Lua.
//!
//! Parsing may use `f32`. Every accepted balance value is finite, strictly
//! positive, and quantized once with the same expression as
//! `omoba-template-ids/build.rs` `fixed64_lit`: `(value * 1024.0).round() as i32`.
//! An empty list is rejected. Callers must not default a missing catalog to empty.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Explicit `catalog_id` values. Declaration order does not assign identity.
pub const MAX_MOBA_SUMMONER_SPELLS: usize = 64;
pub const MAX_MOBA_SUMMONER_CATALOG_ID: u16 = 1024;
/// Same raw envelope the cast model accepts (`SIM_TERRAIN_COORD_MAX_RAW`).
pub const SUMMONER_FIXED64_MAX_RAW: i32 = 1 << 29;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MobaSummonerEntry {
    pub catalog_id: u16,
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub cooldown: f32,
    pub target: MobaSummonerTarget,
    pub operation: MobaSummonerOperation,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MobaSummonerTarget {
    None,
    Point,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MobaSummonerOutOfRange {
    Reject,
    ClampToRange,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MobaSummonerOperation {
    PointBlink {
        range: f32,
        out_of_range: MobaSummonerOutOfRange,
    },
    SelfHeal { amount: f32 },
}

impl MobaSummonerTarget {
    pub fn rust_literal(self) -> &'static str {
        match self {
            Self::None => "SummonerTargetConst::None",
            Self::Point => "SummonerTargetConst::Point",
        }
    }
}

impl MobaSummonerOutOfRange {
    pub fn rust_literal(self) -> &'static str {
        match self {
            Self::Reject => "SummonerOutOfRangeConst::Reject",
            Self::ClampToRange => "SummonerOutOfRangeConst::ClampToRange",
        }
    }
}

impl MobaSummonerOperation {
    pub fn rust_literal(&self) -> String {
        match self {
            Self::PointBlink { range, out_of_range } => format!(
                "SummonerSpellOperationConst::PointBlink {{ range: {}, out_of_range: {} }}",
                fixed64_rust_lit(*range),
                out_of_range.rust_literal(),
            ),
            Self::SelfHeal { amount } => format!(
                "SummonerSpellOperationConst::SelfHeal {{ amount: {} }}",
                fixed64_rust_lit(*amount),
            ),
        }
    }
}

/// Identical to `fixed64_lit` in `omoba-template-ids/build.rs`.
pub fn fixed64_rust_lit(value: f32) -> String {
    let raw = (value * 1024.0).round() as i32;
    format!("Fixed64::from_raw({raw})")
}

pub fn summoner_authoring_fixed_raw(value: f32) -> Result<i32, String> {
    if !value.is_finite() || value <= 0.0 {
        return Err("summoner balance must be finite and positive".into());
    }
    let scaled = f64::from(value) * 1024.0;
    if !scaled.is_finite() {
        return Err("summoner balance quantization is not finite".into());
    }
    let rounded = scaled.round();
    let max_raw = f64::from(SUMMONER_FIXED64_MAX_RAW);
    if !(1.0..=max_raw).contains(&rounded) {
        return Err("summoner balance is outside the fixed-point envelope".into());
    }
    let lit = (value * 1024.0).round() as i32;
    if f64::from(lit) != rounded {
        return Err("summoner balance does not match fixed64_lit quantization".into());
    }
    Ok(lit)
}

pub fn validate_moba_summoners(spells: &[MobaSummonerEntry]) -> Result<(), String> {
    if spells.is_empty() || spells.len() > MAX_MOBA_SUMMONER_SPELLS {
        return Err("summoner catalog must be a non-empty bounded list".into());
    }
    let mut ids = BTreeSet::new();
    let mut numeric = BTreeSet::new();
    for spell in spells {
        if spell.catalog_id == 0 || spell.catalog_id > MAX_MOBA_SUMMONER_CATALOG_ID {
            return Err(format!("summoner '{}' catalog_id is outside 1..={MAX_MOBA_SUMMONER_CATALOG_ID}", spell.id));
        }
        if !ids.insert(spell.id.as_str()) || !numeric.insert(spell.catalog_id) {
            return Err(format!("duplicate summoner identity '{}'", spell.id));
        }
        validate_summoner_spell(spell)?;
    }
    Ok(())
}

fn validate_summoner_spell(spell: &MobaSummonerEntry) -> Result<(), String> {
    let id_ok = !spell.id.is_empty()
        && spell.id.len() <= 64
        && spell.id.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
    let text_ok = |value: &str, max_chars: usize| {
        let count = value.chars().count();
        (1..=max_chars).contains(&count) && value.chars().all(|ch| !ch.is_control())
    };
    if !id_ok || !text_ok(&spell.display_name, 64) || !text_ok(&spell.description, 512) {
        return Err(format!("invalid summoner spell text '{}'", spell.id));
    }
    summoner_authoring_fixed_raw(spell.cooldown).map_err(|error| format!("summoner '{}' cooldown: {error}", spell.id))?;
    match (&spell.operation, spell.target) {
        (MobaSummonerOperation::PointBlink { range, .. }, MobaSummonerTarget::Point) => {
            summoner_authoring_fixed_raw(*range).map_err(|error| format!("summoner '{}' range: {error}", spell.id))?;
        }
        (MobaSummonerOperation::SelfHeal { amount }, MobaSummonerTarget::None) => {
            summoner_authoring_fixed_raw(*amount).map_err(|error| format!("summoner '{}' amount: {error}", spell.id))?;
        }
        _ => return Err(format!("summoner '{}' operation does not match target shape", spell.id)),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical_template_hash;

    fn blink(catalog_id: u16, id: &str) -> MobaSummonerEntry {
        MobaSummonerEntry {
            catalog_id,
            id: id.into(),
            display_name: "Blink".into(),
            description: "Point blink fixture.".into(),
            cooldown: 12.0,
            target: MobaSummonerTarget::Point,
            operation: MobaSummonerOperation::PointBlink {
                range: 30.0,
                out_of_range: MobaSummonerOutOfRange::ClampToRange,
            },
        }
    }

    fn heal(catalog_id: u16, id: &str) -> MobaSummonerEntry {
        MobaSummonerEntry {
            catalog_id,
            id: id.into(),
            display_name: "Heal".into(),
            description: "Self heal fixture.".into(),
            cooldown: 8.0,
            target: MobaSummonerTarget::None,
            operation: MobaSummonerOperation::SelfHeal { amount: 15.0 },
        }
    }

    #[test]
    fn moba_summoners_catalog_identity_bounds_and_shape_are_strict() {
        let pair = vec![blink(1, "blink_v1"), heal(2, "heal_v1")];
        validate_moba_summoners(&pair).unwrap();
        let mut reversed = pair.clone();
        reversed.reverse();
        validate_moba_summoners(&reversed).unwrap();
        assert_eq!(reversed[0].catalog_id, 2);
        assert_eq!(reversed[0].id, "heal_v1");

        #[derive(Deserialize)]
        struct RequiredCatalog {
            moba_summoners: Vec<MobaSummonerEntry>,
        }
        assert!(serde_json::from_value::<RequiredCatalog>(serde_json::json!({})).is_err());
        let parsed = serde_json::from_value::<RequiredCatalog>(serde_json::json!({"moba_summoners": &pair})).unwrap();
        assert_eq!(parsed.moba_summoners, pair);
        validate_moba_summoners(&parsed.moba_summoners).unwrap();
        assert!(validate_moba_summoners(&[]).is_err());

        let mut too_many = Vec::new();
        for index in 1..=MAX_MOBA_SUMMONER_SPELLS as u16 + 1 {
            too_many.push(heal(index, &format!("heal_{index}")));
        }
        assert!(validate_moba_summoners(&too_many).is_err());

        let mut duplicated = pair.clone();
        duplicated[1].id = duplicated[0].id.clone();
        assert!(validate_moba_summoners(&duplicated).is_err());
        duplicated = pair.clone();
        duplicated[1].catalog_id = duplicated[0].catalog_id;
        assert!(validate_moba_summoners(&duplicated).is_err());
        for catalog_id in [0, MAX_MOBA_SUMMONER_CATALOG_ID + 1] {
            let mut bad = pair.clone();
            bad[0].catalog_id = catalog_id;
            assert!(validate_moba_summoners(&bad).is_err());
        }
        for id in ["", "Blink", "has-dash", &"a".repeat(65)] {
            let mut bad = pair.clone();
            bad[0].id = id.into();
            assert!(validate_moba_summoners(&bad).is_err(), "{id}");
        }
        for (name, description) in [("", "ok"), ("ok", ""), ("line\n", "ok"), ("ok", "bad\t")] {
            let mut bad = pair.clone();
            bad[0].display_name = name.into();
            bad[0].description = description.into();
            assert!(validate_moba_summoners(&bad).is_err());
        }
        for value in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0001, 3_000_000.0] {
            let mut bad = pair.clone();
            bad[0].cooldown = value;
            assert!(validate_moba_summoners(&bad).is_err());
            bad = pair.clone();
            bad[0].operation = MobaSummonerOperation::PointBlink {
                range: value,
                out_of_range: MobaSummonerOutOfRange::Reject,
            };
            assert!(validate_moba_summoners(&bad).is_err());
            bad = pair.clone();
            bad[1].operation = MobaSummonerOperation::SelfHeal { amount: value };
            assert!(validate_moba_summoners(&bad).is_err());
        }
        let mut mismatched = pair.clone();
        mismatched[0].target = MobaSummonerTarget::None;
        assert!(validate_moba_summoners(&mismatched).is_err());
        mismatched = pair.clone();
        mismatched[1].target = MobaSummonerTarget::Point;
        assert!(validate_moba_summoners(&mismatched).is_err());

        let mut raw = serde_json::to_value(&pair[0]).unwrap();
        raw["target"] = serde_json::json!("unit");
        assert!(serde_json::from_value::<MobaSummonerEntry>(raw.clone()).is_err());
        raw = serde_json::to_value(&pair[0]).unwrap();
        raw["target"] = serde_json::json!("entity");
        assert!(serde_json::from_value::<MobaSummonerEntry>(raw).is_err());
        raw = serde_json::to_value(&pair[0]).unwrap();
        raw["operation"]["kind"] = serde_json::json!("dash_to_point");
        assert!(serde_json::from_value::<MobaSummonerEntry>(raw).is_err());
        raw = serde_json::to_value(&pair[0]).unwrap();
        raw["mana"] = serde_json::json!(10);
        assert!(serde_json::from_value::<MobaSummonerEntry>(raw).is_err());

        assert_eq!(summoner_authoring_fixed_raw(300.0).unwrap(), 307_200);
        assert_eq!(summoner_authoring_fixed_raw(400.0).unwrap(), 409_600);
        assert_eq!(summoner_authoring_fixed_raw(240.0).unwrap(), 245_760);
        assert_eq!(summoner_authoring_fixed_raw(200.0).unwrap(), 204_800);
        assert_eq!(summoner_authoring_fixed_raw(524_288.0).unwrap(), SUMMONER_FIXED64_MAX_RAW);
        assert!(summoner_authoring_fixed_raw(524_288.0625).is_err());
        assert_eq!(fixed64_rust_lit(300.0), "Fixed64::from_raw(307200)");
        assert_eq!(
            pair[0].operation.rust_literal(),
            "SummonerSpellOperationConst::PointBlink { range: Fixed64::from_raw(30720), out_of_range: SummonerOutOfRangeConst::ClampToRange }"
        );
        assert_eq!(pair[1].operation.rust_literal(), "SummonerSpellOperationConst::SelfHeal { amount: Fixed64::from_raw(15360) }");
        assert_eq!(pair[0].target.rust_literal(), "SummonerTargetConst::Point");
        assert_eq!(pair[1].target.rust_literal(), "SummonerTargetConst::None");
    }

    #[test]
    fn one_moba_summoner_field_changes_canonical_template_hash() {
        let spells = vec![blink(1, "blink_v1"), heal(2, "heal_v1")];
        let base = serde_json::json!({ "moba_summoners": spells });
        let before = canonical_template_hash(&base).unwrap();
        assert_eq!(before, canonical_template_hash(&base).unwrap());
        let keys = [
            "catalog_id",
            "id",
            "display_name",
            "description",
            "cooldown",
            "target",
            "operation",
        ];
        let serialized = serde_json::to_value(&spells[0]).unwrap();
        for key in keys {
            assert!(serialized.get(key).is_some(), "{key}");
        }
        let operation = serialized["operation"].as_object().unwrap();
        assert!(operation.contains_key("kind"));
        assert!(operation.contains_key("range"));
        assert!(operation.contains_key("out_of_range"));
        assert!(serde_json::to_value(&spells[1]).unwrap()["operation"].as_object().unwrap().contains_key("amount"));

        let mut cases: Vec<(&str, serde_json::Value)> = Vec::new();
        let mut cooldown = base.clone();
        cooldown["moba_summoners"][0]["cooldown"] = serde_json::json!(13.0);
        cases.push(("cooldown", cooldown));
        let mut range = base.clone();
        range["moba_summoners"][0]["operation"]["range"] = serde_json::json!(31.0);
        cases.push(("range", range));
        let mut policy = base.clone();
        policy["moba_summoners"][0]["operation"]["out_of_range"] = serde_json::json!("reject");
        cases.push(("out_of_range", policy));
        let mut kind = base.clone();
        kind["moba_summoners"][0]["operation"] = serde_json::json!({"kind": "self_heal", "amount": 15.0});
        cases.push(("operation", kind));
        let mut target = base.clone();
        target["moba_summoners"][0]["target"] = serde_json::json!("none");
        cases.push(("target", target));
        let mut amount = base.clone();
        amount["moba_summoners"][1]["operation"]["amount"] = serde_json::json!(16.0);
        cases.push(("amount", amount));
        let mut name = base.clone();
        name["moba_summoners"][0]["display_name"] = serde_json::json!("Other");
        cases.push(("display_name", name));
        let mut description = base.clone();
        description["moba_summoners"][0]["description"] = serde_json::json!("Other description.");
        cases.push(("description", description));
        let mut catalog_id = base.clone();
        catalog_id["moba_summoners"][0]["catalog_id"] = serde_json::json!(9);
        cases.push(("catalog_id", catalog_id));
        let mut id = base.clone();
        id["moba_summoners"][0]["id"] = serde_json::json!("blink_v2");
        cases.push(("id", id));
        for (field, changed) in cases {
            assert_ne!(before, canonical_template_hash(&changed).unwrap(), "{field}");
        }
    }
}
