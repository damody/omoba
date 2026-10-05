//! Build-time animation declarations shared by Rust and Unreal generators.
use serde_json::Value;

pub fn validate_animation_metadata(id: &str, animation: Option<&Value>) -> Result<(), String> {
    let Some(animation) = animation else { return Ok(()); };
    let object = animation.as_object().ok_or_else(|| format!("{id} ue.animation must be an object"))?;
    for field in ["locomotion_variants", "anim_bp_variables", "state_mapping", "montage_mapping", "attack_phase_mapping", "critical_attack"] {
        if let Some(value) = object.get(field) {
            let map = value.as_object().ok_or_else(|| format!("{id} ue.animation.{field} must be an object"))?;
            if map.len() > 64 { return Err(format!("{id} ue.animation.{field} exceeds 64 entries")); }
            for (key, value) in map {
                if key.trim().is_empty() || value.as_str().is_none_or(|value| value.trim().is_empty()) {
                    return Err(format!("{id} ue.animation.{field}.{key} must map non-empty keys to non-empty strings"));
                }
            }
        }
    }
    if let Some(value) = object.get("idle_variants") {
        let values = value.as_array().ok_or_else(|| format!("{id} ue.animation.idle_variants must be an array"))?;
        if values.len() > 64 || values.iter().any(|value| value.as_str().is_none_or(|text| text.trim().is_empty())) {
            return Err(format!("{id} ue.animation.idle_variants must contain at most 64 non-empty strings"));
        }
    }
    if let Some(value) = object.get("default_play_rate") {
        let rate = value.as_f64().ok_or_else(|| format!("{id} ue.animation.default_play_rate must be a number"))?;
        if !rate.is_finite() || rate <= 0.0 || !(rate as f32).is_finite() || rate as f32 <= 0.0 {
            return Err(format!("{id} ue.animation.default_play_rate must be finite, positive and representable as f32"));
        }
    }
    if let Some(value) = object.get("fallback_policy") {
        if !matches!(value.as_str(), Some("UseGenericState" | "UseReferencePose" | "HideVisual")) {
            return Err(format!("{id} ue.animation.fallback_policy must be UseGenericState, UseReferencePose or HideVisual"));
        }
    }
    Ok(())
}

/// Validate raw authoring metadata before either generator emits any output.
/// Tombstones do not contribute runtime presentation declarations.
pub fn validate_animation_catalog(catalog: &Value) -> Result<(), String> {
    for group in ["heroes", "abilities", "buffs", "summons", "creeps", "towers"] {
        if let Some(entries) = catalog.get(group).and_then(Value::as_array) {
            for entry in entries.iter().filter(|entry| entry.get("tombstone").and_then(Value::as_bool) != Some(true)) {
                let id = entry.get("id").and_then(Value::as_str).unwrap_or(group);
                validate_animation_metadata(id, entry.get("ue").and_then(|ue| ue.get("animation")))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generic_animation_metadata_checks_all_maps_and_numeric_representation() {
        for field in ["locomotion_variants", "anim_bp_variables", "state_mapping", "montage_mapping", "attack_phase_mapping", "critical_attack"] {
            for value in [serde_json::json!([]), serde_json::json!(false), serde_json::json!({"walk":1}), serde_json::json!({"":"clip"}), serde_json::json!({"walk":" "})] {
                let invalid = serde_json::json!({field:value});
                let error = validate_animation_metadata("custom_hero", Some(&invalid)).unwrap_err();
                assert!(error.contains("custom_hero") && error.contains(field), "{error}");
            }
            let valid = serde_json::json!({field:{"custom_state":"custom_clip"}});
            validate_animation_metadata("custom_hero", Some(&valid)).unwrap();
        }
        for value in [serde_json::json!(false), serde_json::json!(null), serde_json::json!(0), serde_json::json!(-1), serde_json::json!(1e100), serde_json::json!(1e-100)] {
            assert!(validate_animation_metadata("custom", Some(&serde_json::json!({"default_play_rate":value}))).is_err());
        }
        for value in [serde_json::json!(false), serde_json::json!(null), serde_json::json!("Unknown")] {
            assert!(validate_animation_metadata("custom", Some(&serde_json::json!({"fallback_policy":value}))).is_err());
        }
        let too_many = (0..65).map(|i| (format!("state_{i}"), serde_json::json!("clip"))).collect::<serde_json::Map<_,_>>();
        assert!(validate_animation_metadata("custom", Some(&serde_json::json!({"state_mapping":too_many}))).is_err());
        for policy in ["UseGenericState", "UseReferencePose", "HideVisual"] {
            validate_animation_metadata("custom", Some(&serde_json::json!({"fallback_policy":policy,"default_play_rate":1.25,"idle_variants":["rest"]}))).unwrap();
        }
    }
    #[test]
    fn generic_animation_metadata_catalog_checks_every_kind_and_skips_tombstones() {
        for group in ["heroes", "abilities", "buffs", "summons", "creeps", "towers"] {
            let mut catalog = serde_json::json!({group:[{"id":"arbitrary","ue":{"animation":{"fallback_policy":false}}}]});
            assert!(validate_animation_catalog(&catalog).unwrap_err().contains("arbitrary"));
            catalog[group][0]["tombstone"] = serde_json::json!(true);
            validate_animation_catalog(&catalog).unwrap();
        }
    }
}
