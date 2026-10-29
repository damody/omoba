//! behavior IR（M3）：v3 文件的行為由建置期的 `docs/character_pipeline/tools/v3_ir.lua` 編成這裡的結構，
//! 放在 V3Block 的 `ir`。格式見 docs/character_pipeline/schema/v3-content-model-proposal.md §4.1，
//! 執行器見 docs/plans/2026-10-28-history-nobu-full-port/m3-ir-design.md。
//!
//! - 頂層欄位固定（不認得的頂層欄位拒絕）；所有索引從 1 起算（Lua 陣列）。
//! - **運算式**：後序 token 陣列，每個 token 是 `["slot", 1]`、`["name", "caster"]`、`["op", "+"]`……
//! - **程式**：指令陣列；指令、狀態、模板、selector 是開放的 JSON map，欄位照 v3 action 參數。
//!   omoba-core 的直譯器載入時逐一解析，不認得的 op 或欄位讓那一招載入失敗（不會默默退回 handler）。
//!
//! 這裡只做結構檢查：版本、token 形狀、運算式與程式索引在範圍內。
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// 目前的 IR 版本；直譯器只接受這個版本（舊版 fail closed）。
pub const IR_VERSION: u32 = 1;

/// 一個後序 token：第一個元素是種類，其餘是參數。
pub type IrToken = Vec<Value>;
/// 一個指令（`op` 加上參數）。
pub type IrInstr = Map<String, Value>;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IrBlock {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slots: Vec<IrSlot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exprs: Vec<Vec<IrToken>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub programs: Vec<Vec<IrInstr>>,
    /// 區域變數的數量（`let`、`for_each` 的 `it`、回呼綁定、routine 參數）。
    #[serde(default)]
    pub locals: u32,
    /// 技能事件 → `{ program, ... }`。
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub entries: BTreeMap<String, Map<String, Value>>,
    /// 被動：`stats`、`states`、`triggers`（事件 → `{ program, filter, ... }`）、`aura`、`reveal`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passive: Option<Map<String, Value>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<Map<String, Value>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selectors: Vec<Map<String, Value>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub routines: BTreeMap<String, Map<String, Value>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub templates: BTreeMap<String, Map<String, Value>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hooks: Vec<String>,
    /// 技能的目標規格（`type`、`team`、`types`、`flags`、`ability_flags`、`range`、`aoe_radius`……，運算式欄位是索引）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<Map<String, Value>>,
    /// 施放條件（v3 `requires.condition`）的運算式索引。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<u64>,
    /// 區塊讀寫的 unit_state 宣告（m3-ir-design.md §7.2），依名稱排序。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unit_state: Vec<IrUnitState>,
}

/// 一個 unit_state 變數的宣告。跨文件同名就是同一個變數（登錄表保證宣告一致）。
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IrUnitState {
    pub name: String,
    /// v3 的 STATE_TYPES（fixed、int、bool、ability、unit……）。
    #[serde(rename = "type")]
    pub kind: String,
    /// 見 [`UNIT_STATE_SCOPES`]；`match` 是全場一份。
    pub scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reveal: Option<String>,
    /// 初值的運算式索引；省略時是型別的零值（0、false、none）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub init: Option<u64>,
}

/// unit_state 的 scope。
pub const UNIT_STATE_SCOPES: &[&str] = &["hero", "unit", "match", "ability", "item_instance", "item_name", "permanent"];

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IrSlot {
    pub name: String,
    /// 數值屬於另一個擁有者（狀態、模板、其他技能）時的 id。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// `pct`：運算式讀到的是比例，載入時除以 100。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<String>,
    /// `truncate`：除以 100 時截斷（照數值定義的 rounding）；省略時四捨五入。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rounding: Option<String>,
}

/// 指令或 map 裡指向運算式的 1 起算索引欄位。
pub const EXPR_FIELDS: &[&str] = &["to", "amount", "at", "duration", "count", "lifetime", "limit_per_owner", "cond", "value",
    "filter", "radius", "from", "follow", "interval", "first", "where", "limit", "prefer", "post_filter", "order_by",
    "range", "aoe_radius", "min_range", "vector_length"];
/// 指向程式的 1 起算索引欄位。
pub const PROGRAM_FIELDS: &[&str] = &["program", "actions", "otherwise", "on_expire", "on_break", "on_hit", "on_end",
    "on_enter", "on_leave", "on_pickup", "on_destroyed"];

impl IrBlock {
    /// 結構檢查：版本、token 形狀、運算式與程式索引都在範圍內、跳躍不越界。
    pub fn validate(&self) -> Result<(), String> {
        if self.version != IR_VERSION { return Err(format!("unsupported IR version {}", self.version)); }
        for (index, code) in self.exprs.iter().enumerate() {
            if code.is_empty() { return Err(format!("expr {} is empty", index + 1)); }
            for (at, token) in code.iter().enumerate() {
                let kind = token.first().and_then(Value::as_str).ok_or_else(|| format!("expr {}: token without a kind", index + 1))?;
                let int = |i: usize| token.get(i).and_then(Value::as_u64);
                match kind {
                    "slot" => if !int(1).is_some_and(|i| i >= 1 && i as usize <= self.slots.len()) {
                        return Err(format!("expr {}: slot out of range", index + 1));
                    },
                    "local" => if !int(1).is_some_and(|i| i >= 1 && i <= u64::from(self.locals)) {
                        return Err(format!("expr {}: local out of range", index + 1));
                    },
                    "jump" | "jump_if_false" => if !int(1).is_some_and(|d| at + 1 + d as usize <= code.len()) {
                        return Err(format!("expr {}: jump past the end", index + 1));
                    },
                    "raw" => if token.get(1).and_then(Value::as_i64).is_none() { return Err(format!("expr {}: raw needs an integer", index + 1)); },
                    "bool" | "name" | "attr" | "call" | "op" | "sym" | "state" | "var" => {}
                    other => return Err(format!("expr {}: unknown token `{other}`", index + 1)),
                }
            }
        }
        let (exprs, programs) = (self.exprs.len() as u64, self.programs.len() as u64);
        let check = |map: &Map<String, Value>, what: &str| -> Result<(), String> {
            for (key, value) in map {
                let Some(i) = value.as_u64() else { continue };
                if EXPR_FIELDS.contains(&key.as_str()) && (i == 0 || i > exprs) { return Err(format!("{what}: `{key}` expr {i} out of range")); }
                if PROGRAM_FIELDS.contains(&key.as_str()) && (i == 0 || i > programs) { return Err(format!("{what}: `{key}` program {i} out of range")); }
            }
            Ok(())
        };
        for (index, program) in self.programs.iter().enumerate() {
            for instr in program {
                if instr.get("op").and_then(Value::as_str).is_none() { return Err(format!("program {}: instruction without op", index + 1)); }
                check(instr, &format!("program {}", index + 1))?;
            }
        }
        for (event, entry) in &self.entries { check(entry, event)?; }
        for status in &self.statuses {
            if status.get("id").and_then(Value::as_str).is_none() { return Err("status without id".into()); }
            check(status, "status")?;
        }
        for selector in &self.selectors { check(selector, "selector")?; }
        if let Some(target) = &self.target { check(target, "target")?; }
        if self.requires.is_some_and(|e| e == 0 || e > exprs) { return Err("requires: expr out of range".into()); }
        for (index, state) in self.unit_state.iter().enumerate() {
            if index > 0 && self.unit_state[index - 1].name >= state.name {
                return Err(format!("unit_state: `{}` is not sorted or repeats", state.name));
            }
            if !UNIT_STATE_SCOPES.contains(&state.scope.as_str()) {
                return Err(format!("unit_state `{}`: unknown scope `{}`", state.name, state.scope));
            }
            if state.init.is_some_and(|e| e == 0 || e > exprs) { return Err(format!("unit_state `{}`: init out of range", state.name)); }
        }
        // 有宣告時，每個讀到的變數都要有宣告。
        if !self.unit_state.is_empty() {
            let declared = |name: &str| self.unit_state.binary_search_by(|s| s.name.as_str().cmp(name)).is_ok();
            for (index, code) in self.exprs.iter().enumerate() {
                for token in code {
                    if token.first().and_then(Value::as_str) != Some("state") { continue; }
                    let name = token.get(1).and_then(Value::as_str).unwrap_or_default();
                    if !declared(name) { return Err(format!("expr {}: unit_state `{name}` is not declared", index + 1)); }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// v3_ir.lua's output for B01R (2026-10-29).
    fn rain_iron_cannon() -> serde_json::Value {
        serde_json::json!({"exprs":[[["name","self"],["attr","attack"],["name","event"],["attr","target"],["attr","is_building"],
            ["jump_if_false",2],["slot",1],["jump",7],["name","event"],["attr","target"],["attr","is_hero"],["jump_if_false",2],
            ["slot",2],["jump",1],["slot",3],["op","*"]],[["name","event"],["attr","target"]],[["name","event"],["attr","target"]],
            [["name","event"],["attr","target"],["call","alive",1]]],"locals":0,
            "passive":{"reveal":"public","triggers":{"on_attack_hit":{"filter":4,"program":1}}},
            "programs":[[{"amount":1,"op":"damage","to":2,"type":"physical"},{"at":3,"id":"b01r_hit","op":"cue"}]],
            "slots":[{"name":"building_pct","scale":"pct"},{"name":"hero_pct","scale":"pct"},{"name":"creep_pct","scale":"pct"}],"version":1})
    }

    #[test]
    fn the_compiler_output_parses_and_validates() {
        let block: IrBlock = serde_json::from_value(rain_iron_cannon()).unwrap();
        block.validate().unwrap();
        let mut bad = block.clone();
        bad.exprs[0].push(vec!["slot".into(), 9.into()]);
        assert!(bad.validate().is_err(), "slot out of range");
        let mut bad = block.clone();
        bad.programs[0][0].insert("amount".into(), 7.into());
        assert!(bad.validate().is_err(), "expr index out of range");
        let mut bad = block;
        bad.version = 2;
        assert!(bad.validate().is_err());
        assert!(serde_json::from_value::<IrBlock>(serde_json::json!({"version": 1, "extra": 1})).is_err(), "unknown top-level field");
    }

    /// B30T 的 tsum（v3_ir.lua 打開 EMIT_UNIT_STATE 時的形狀）。
    #[test]
    fn unit_state_declarations_validate() {
        let json = serde_json::json!({"version": 1, "exprs": [[["raw", 0]], [["state", "tsum"]]],
            "unit_state": [{"name": "tsum", "type": "fixed", "scope": "match", "reveal": "public", "init": 1}]});
        let block: IrBlock = serde_json::from_value(json.clone()).unwrap();
        block.validate().unwrap();
        assert_eq!(block.unit_state[0].scope, "match");
        let mut bad = block.clone();
        bad.unit_state[0].init = Some(3);
        assert!(bad.validate().is_err(), "init out of range");
        let mut bad = block.clone();
        bad.unit_state[0].scope = "global".into();
        assert!(bad.validate().is_err(), "unknown scope");
        let mut bad = block.clone();
        bad.unit_state[0].name = "other".into();
        assert!(bad.validate().is_err(), "undeclared state read");
        let mut bad = block.clone();
        bad.unit_state.push(bad.unit_state[0].clone());
        assert!(bad.validate().is_err(), "repeated name");
        let mut row = json;
        row["unit_state"][0]["extra"] = 1.into();
        assert!(serde_json::from_value::<IrBlock>(row).is_err(), "unknown unit_state field");
    }
}
