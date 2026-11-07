//! What one point of a hero attribute is worth (templates/attribute_rules.lua). The Rust and
//! Unreal builds read the same table; the runtime never hard-codes a coefficient.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AttributeRules {
    /// The Dota 2 version the coefficients follow (documentation; e.g. "7.41f").
    pub version: String,
    pub strength: StrengthRules,
    pub agility: AgilityRules,
    pub intelligence: IntelligenceRules,
    /// Attack damage per point of the hero's primary attribute.
    pub primary_damage: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StrengthRules {
    pub health: f32,
    pub health_regen: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgilityRules {
    pub armor: f32,
    pub attack_speed: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IntelligenceRules {
    pub mana: f32,
    pub mana_regen: f32,
    /// Percent magic resistance per point.
    pub magic_resistance: f32,
}

impl AttributeRules {
    /// Coefficients are per point and bounded: a typo cannot make one point worth a whole hero.
    pub fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty() {
            return Err("attribute_rules.version names the Dota 2 version the coefficients follow".into());
        }
        let checks = [
            ("strength.health", self.strength.health, 100.0),
            ("strength.health_regen", self.strength.health_regen, 10.0),
            ("agility.armor", self.agility.armor, 10.0),
            ("agility.attack_speed", self.agility.attack_speed, 10.0),
            ("intelligence.mana", self.intelligence.mana, 100.0),
            ("intelligence.mana_regen", self.intelligence.mana_regen, 10.0),
            ("intelligence.magic_resistance", self.intelligence.magic_resistance, 1.0),
            ("primary_damage", self.primary_damage, 10.0),
        ];
        for (name, value, limit) in checks {
            if !value.is_finite() || !(0.0..=limit).contains(&value) {
                return Err(format!("attribute_rules.{name} {value} must be within 0..={limit}"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn attribute_rules_reject_unknown_fields_and_out_of_range_values() {
    let rules = |health: f64| serde_json::json!({"version": "7.41f", "primary_damage": 1.0,
        "strength": {"health": health, "health_regen": 0.1}, "agility": {"armor": 0.167, "attack_speed": 1.0},
        "intelligence": {"mana": 12.0, "mana_regen": 0.05, "magic_resistance": 0.1}});
    serde_json::from_value::<AttributeRules>(rules(22.0)).unwrap().validate().unwrap();
    assert!(serde_json::from_value::<AttributeRules>(rules(-1.0)).unwrap().validate().is_err());
    assert!(serde_json::from_value::<AttributeRules>(rules(1000.0)).unwrap().validate().is_err());
    let mut extra = rules(22.0);
    extra["strength"]["armor"] = serde_json::json!(1.0);
    assert!(serde_json::from_value::<AttributeRules>(extra).is_err());
}
