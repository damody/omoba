//! Build-time bindings from private declarative effects to public buff identity.
use crate::{AbilityDefinition, AbilityEffect, ManaBuffStat};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BuffVisualFamily {
    SlowEnemy,
    ManaBuffSelf,
    DamageOverTimeEnemy,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuffVisualSource {
    pub ability_id: String,
    pub kind: BuffVisualFamily,
    #[serde(default)]
    pub stat: Option<ManaBuffStat>,
}

#[derive(Clone, Debug)]
pub struct BuffVisualBinding {
    pub buff_id: String,
    pub source: BuffVisualSource,
}

impl BuffVisualSource {
    /// Disambiguate slow and ongoing damage from the same authored ability.
    pub fn effect_stat(&self)->Option<&'static str> {
        match self.kind {BuffVisualFamily::DamageOverTimeEnemy=>Some("dot_damage"),_=>self.stat.map(ManaBuffStat::as_str)}
    }
}

pub fn collect_buff_visual_sources(manifest: &Value) -> Result<Vec<BuffVisualBinding>, String> {
    let abilities: Vec<AbilityDefinition> = serde_json::from_value(
        manifest
            .get("abilities")
            .cloned()
            .unwrap_or_else(|| Value::Array(vec![])),
    )
    .map_err(|err| format!("buff visual sources: invalid abilities: {err}"))?;
    let abilities: BTreeMap<_, _> = abilities
        .iter()
        .filter(|a| !a.tombstone)
        .map(|a| (a.id.as_str(), a))
        .collect();
    let mut bindings = Vec::new();
    let mut seen = BTreeSet::new();
    for buff in manifest
        .get("buffs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if buff
            .get("tombstone")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            continue;
        }
        let Some(sources) = buff.pointer("/ue/buff_visual/sources") else {
            continue;
        };
        let id = buff
            .get("id")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or("buff visual sources: missing buff ID")?;
        let sources: Vec<BuffVisualSource> = serde_json::from_value(sources.clone())
            .map_err(|err| format!("buff '{id}' visual sources: {err}"))?;
        if sources.len() > 32 {
            return Err(format!("buff '{id}': too many visual sources"));
        }
        for source in sources {
            let ability = abilities.get(source.ability_id.as_str()).ok_or_else(|| {
                format!(
                    "buff '{id}': unknown source ability '{}'",
                    source.ability_id
                )
            })?;
            let matched =
                ability
                    .effects
                    .iter()
                    .any(|effect| match (source.kind, source.stat, effect) {
                        (BuffVisualFamily::SlowEnemy, None, AbilityEffect::SlowEnemy { .. }) => {
                            true
                        }
                        (BuffVisualFamily::DamageOverTimeEnemy,None,AbilityEffect::DamageOverTimeEnemy {..})=>true,
                        (
                            BuffVisualFamily::ManaBuffSelf,
                            Some(stat),
                            AbilityEffect::ManaBuffSelf {
                                stat: effect_stat, ..
                            },
                        ) => stat == *effect_stat,
                        _ => false,
                    });
            if !matched {
                return Err(format!(
                    "buff '{id}': source does not match effect of '{}'",
                    source.ability_id
                ));
            }
            let key = (
                source.ability_id.clone(),
                match source.kind {
                    BuffVisualFamily::SlowEnemy => 0,
                    BuffVisualFamily::ManaBuffSelf => 1,
                    BuffVisualFamily::DamageOverTimeEnemy => 2,
                },
                source.stat,
            );
            if !seen.insert(key) {
                return Err(format!("buff '{id}': duplicate/conflicting visual source"));
            }
            bindings.push(BuffVisualBinding {
                buff_id: id.into(),
                source,
            });
        }
    }
    Ok(bindings)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dot_visual_family_matches_only_authored_dot_and_keeps_a_distinct_effect_channel() {
        let valid=serde_json::json!({"abilities":[{"id":"a","effects":[{"kind":"damage_over_time_enemy","amount_key":"r","duration_key":"d","damage_kind":"physical"}]}],
            "buffs":[{"id":"bleed","ue":{"buff_visual":{"sources":[{"ability_id":"a","kind":"damage_over_time_enemy"}]}}}]});
        let bindings=collect_buff_visual_sources(&valid).unwrap();assert_eq!(bindings[0].source.effect_stat(),Some("dot_damage"));
        for kind in ["slow_enemy","mana_buff_self"] {let mut bad=valid.clone();bad["buffs"][0]["ue"]["buff_visual"]["sources"][0]["kind"]=Value::String(kind.into());assert!(collect_buff_visual_sources(&bad).is_err());}
        let mut bad=valid;bad["buffs"][0]["ue"]["buff_visual"]["sources"][0]["stat"]=Value::String("mana_bonus".into());assert!(collect_buff_visual_sources(&bad).is_err());
    }
    #[test]
    fn buff_visual_source_bindings_reject_unknown_mismatched_and_conflicting_sources() {
        let valid = serde_json::json!({"abilities":[{"id":"a","effects":[{"kind":"slow_enemy","reduction_key":"r","duration_key":"d"}]}],
            "buffs":[{"id":"slow","ue":{"buff_visual":{"sources":[{"ability_id":"a","kind":"slow_enemy"}]}}}]});
        assert_eq!(collect_buff_visual_sources(&valid).unwrap().len(), 1);
        for source in [
            serde_json::json!({"ability_id":"missing","kind":"slow_enemy"}),
            serde_json::json!({"ability_id":"a","kind":"mana_buff_self","stat":"mana_bonus"}),
            serde_json::json!({"ability_id":"a","kind":"slow_enemy","stat":"mana_bonus"}),
            serde_json::json!({"ability_id":"a","kind":"slow_enemy","secret":1}),
        ] {
            let mut invalid = valid.clone();
            invalid["buffs"][0]["ue"]["buff_visual"]["sources"][0] = source;
            assert!(collect_buff_visual_sources(&invalid).is_err());
        }
        let mut duplicate = valid.clone();
        let entry = duplicate["buffs"][0].clone();
        duplicate["buffs"].as_array_mut().unwrap().push(entry);
        assert!(collect_buff_visual_sources(&duplicate).is_err());
        let mut retired = valid;
        retired["abilities"][0]["tombstone"] = Value::Bool(true);
        assert!(collect_buff_visual_sources(&retired).is_err());
    }
}
