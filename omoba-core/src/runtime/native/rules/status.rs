//! 狀態抗性（原作 `vscripts/events.lua:385-412` ModifierGainedFilter）。
//!
//! - 原作：目標是英雄、施加者是敵方時，敵方 modifier 的持續時間 ×(1 − 抗性 − 減抗)；抗性各來源取
//!   最大（`item_states_resistance.lua:9-16`）。`modifier_truesight` 與 22 個大招 modifier
//!   （`events.lua:246-269` `ultimate_modifiers`）不受影響。
//! - 這裡：抗性的計算在 `damage_immunity::resist`（`BuffStore::add` 的收斂點，控制、技能緩速、
//!   技能 DoT，取最強）。個別狀態在 payload 宣告 [`IGNORE_STATUS_RESISTANCE`] 時，持續時間不打折；
//!   緩速抗性（原作 `ms_unslow`，施加當下折算）不屬於狀態抗性，照常套用。
//! - 目前宣告 ignore 的狀態：B29T 大地震撼的暈眩（`modifier_B29T`）、C15T 百箭齊發的暈眩
//!   （`modifier_C15T_2`），兩者都在原作的排除清單。
use serde_json::Value;

/// v3（v3-vocab ABILITY_FLAGS `does_not_break_invisibility`）：施放會破除施法者的隱形。回傳要移除的狀態：
/// 施法者身上所有帶隱形等級的狀態；技能宣告 `does_not_break_invisibility` 時不破隱（catalog v3 區塊的
/// `ability_flags`）。
pub fn invisibility_broken_by_cast(buffs: &crate::runtime::BuffStore, caster: specs::Entity, ability: &str) -> Vec<String> {
    if has_ability_flag(ability, "does_not_break_invisibility") { return Vec::new(); }
    let key = omb_script_abi::stat_keys::StatKey::InvisibilityLevel.as_str();
    let mut ids: Vec<String> = buffs.iter_for(caster)
        .filter(|(_, entry)| entry.payload.get(key).and_then(serde_json::Value::as_i64).is_some_and(|level| level > 0))
        .map(|(id, _)| id.to_owned()).collect();
    ids.sort();
    ids
}

/// 技能的 v3 區塊（`omoba_template_ids::V3_CATALOG_JSON`）有沒有宣告這個 ability flag。
pub fn has_ability_flag(ability: &str, flag: &str) -> bool {
    use std::collections::BTreeMap;
    static FLAGS: std::sync::OnceLock<BTreeMap<String, Vec<String>>> = std::sync::OnceLock::new();
    FLAGS.get_or_init(|| {
        let Ok(doc) = serde_json::from_str::<Value>(omoba_template_ids::V3_CATALOG_JSON) else { return BTreeMap::new() };
        doc["abilities"].as_array().into_iter().flatten().filter_map(|row| {
            let flags: Vec<String> = row["v3"]["ability_flags"].as_array()?.iter().filter_map(|f| f.as_str().map(str::to_owned)).collect();
            Some((row["id"].as_str()?.to_owned(), flags))
        }).collect()
    }).get(ability).is_some_and(|flags| flags.iter().any(|f| f == flag))
}

/// 狀態 payload 的鍵：值為 true 時，這個狀態不受目標的狀態抗性影響。
pub const IGNORE_STATUS_RESISTANCE: &str = "ignore_status_resistance";

/// 原作大招的控制（`ultimate_modifiers`）：穿透魔免，也不受狀態抗性影響。英雄腳本傳給
/// `add_stat_buff` 的 JSON 文字（`magic_immunity::PIERCE_PAYLOAD_JSON` 加上本模組的鍵）。
pub const PIERCING_ULTIMATE_PAYLOAD_JSON: &str = r#"{"__pierces_magic_immunity":1,"ignore_status_resistance":true}"#;

/// payload 是否宣告不受狀態抗性影響。
pub fn ignores_status_resistance(payload: &Value) -> bool {
    payload.get(IGNORE_STATUS_RESISTANCE).and_then(Value::as_bool) == Some(true)
}

/// 在既有的狀態 payload 加上「不受狀態抗性影響」；`null` 視為空物件。
pub fn ignoring_status_resistance(mut payload: Value) -> Value {
    if payload.is_null() { payload = Value::Object(Default::default()); }
    if let Some(object) = payload.as_object_mut() { object.insert(IGNORE_STATUS_RESISTANCE.into(), Value::Bool(true)); }
    payload
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_mark_is_added_to_any_payload_and_read_back() {
        assert!(!ignores_status_resistance(&Value::Null));
        assert!(ignores_status_resistance(&ignoring_status_resistance(Value::Null)));
        let pierce = ignoring_status_resistance(json!({"pierce": true}));
        assert_eq!(pierce["pierce"], true);
        assert!(ignores_status_resistance(&pierce));
        assert!(!ignores_status_resistance(&json!({IGNORE_STATUS_RESISTANCE: 1})), "only an explicit true");
    }

    #[test]
    fn the_ultimate_payload_pierces_and_ignores() {
        let payload: Value = serde_json::from_str(PIERCING_ULTIMATE_PAYLOAD_JSON).unwrap();
        assert_eq!(payload, ignoring_status_resistance(crate::runtime::native::magic_immunity::pierce_payload()));
    }
}
