//! Package schema v3 content blocks (docs/character_pipeline/schema/v3-content-model-proposal.md §2).
//!
//! Each authored ability row and shelf item row may carry `v3`: the values (decimal text plus
//! Fixed64 raw computed from the text at build time), the info line, and the parsed tooltip
//! templates in zh_TW and en. The Lua compiler (docs/character_pipeline/tools/v3_compile.lua)
//! produces it; this module only checks that the block is self-consistent, so every consumer
//! (UE tooltips, the website, bots) can trust it without re-validating.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Current schema revision; any 3.x block is accepted (minor revisions only add).
pub const SCHEMA: &str = "3.1";
/// Fixed64 scale of `raw` (omoba_sim::fixed::SCALE).
pub const SCALE: i64 = 1024;
pub const LANGS: [&str; 2] = ["zh_TW", "en"];
pub const UNITS: [&str; 20] = ["s", "pct", "dmg", "heal", "mana", "hp", "dist", "speed", "move_speed", "atk_speed",
    "armor", "stat", "count", "deg", "gold", "xp", "level", "per_s", "ratio", "number"];
pub const FORMATS: [&str; 6] = ["all", "current", "signed", "abs", "int", "max"];
pub const INFO_FIELDS: [&str; 13] = ["cast_range", "aoe_radius", "cast_point", "channel_time", "mana_cost", "health_cost",
    "gold_cost", "cooldown", "charges_max", "charge_restore", "required_level", "cost", "recipe_cost"];

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct V3Block {
    pub schema: String,
    pub kind: String,
    pub source_id: String,
    pub max_level: u8,
    #[serde(default)]
    pub coverage: Option<String>,
    /// The Rust handler that still runs the behavior (coverage = rust_handler).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handler: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charges_on_rank_up: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<V3Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub info: Vec<V3Ref>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stat_lines: Vec<V3Ref>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<V3Status>,
    #[serde(default)]
    pub text: BTreeMap<String, V3Text>,
    /// Bot cast policy (v3.3): the omoba-core BotAbilityIntent shape (`kind` tag, `*_key` value
    /// names) plus `source` (authored | derived), `proposed` and `reason`/`note`; see
    /// docs/character_pipeline/tools/v3_bot.lua.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot: Option<serde_json::Value>,
    /// The behavior IR (M3, crate::ir); none while a Rust handler runs the behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ir: Option<crate::ir::IrBlock>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct V3Value {
    pub name: String,
    pub unit: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub hidden: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_type: Option<String>,
    /// `hero_level`: one rank per hero level (25), not per ability rank.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per: Option<String>,
    /// The v2 extras key this value replaced (M1 migration; removed with extras in M5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v2_key: Option<String>,
    /// Author decimal or fraction text per rank; empty for a runtime expression.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ranks: Vec<String>,
    /// Fixed64 raw of each rank.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw: Vec<i64>,
    /// A value only known at run time (reads unit properties).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expr: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub derived: bool,
}

/// An info-line field, an item stat line, or a reference to a value of this or another owner.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct V3Ref {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stat: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expr: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct V3Status {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    /// Dispel class (`none`, `basic`, `strong`, `death_only`): what the host's dispel removes (E214).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dispel: Option<String>,
    /// Who learns the status is on a unit (`public`, `team`, `derived`; v3.md §4 `reveal`). The host
    /// shows a `public` status's icon to every team that sees the unit, others only to its own (E216).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reveal: Option<String>,
    /// The BuffStore id the current runtime uses (v3.3); `{caster}` stands for the caster's "id:gen".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<V3Value>,
    #[serde(default)]
    pub text: BTreeMap<String, V3Text>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct V3Text {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pieces: Vec<V3Piece>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rendered: Option<String>,
}

/// One segment of a parsed template: literal text, a value, another entry's name, or a rules value.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct V3Piece {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
}

impl V3Block {
    pub fn value(&self, name: &str) -> Option<&V3Value> { self.values.iter().find(|v| v.name == name) }
    /// Fixed64 raw of a value at a 1-based rank (the last rank repeats).
    pub fn raw_at(&self, name: &str, rank: u8) -> Option<i64> {
        let v = self.value(name)?;
        let i = usize::from(rank.max(1) - 1).min(v.raw.len().checked_sub(1)?);
        v.raw.get(i).copied()
    }
}

/// Fixed64 raw of a decimal ("12.5", "-0.53") or fraction ("1900/101") text, rounded half away
/// from zero (`truncate`: toward zero). Integer arithmetic only, never through f32.
pub fn raw_of(text: &str, truncate: bool) -> Option<i64> {
    let (negative, body) = match text.strip_prefix('-') { Some(rest) => (true, rest), None => (false, text) };
    let (n, d): (i128, i128) = if let Some((a, b)) = body.split_once('/') {
        (a.parse().ok()?, b.parse().ok()?)
    } else if let Some((int, frac)) = body.split_once('.') {
        if frac.is_empty() || frac.len() > 18 { return None; }
        (format!("{int}{frac}").parse().ok()?, 10i128.checked_pow(frac.len() as u32)?)
    } else {
        (body.parse().ok()?, 1)
    };
    if d <= 0 || !body.bytes().all(|b| b.is_ascii_digit() || b == b'.' || b == b'/') { return None; }
    let scaled = n.checked_mul(i128::from(SCALE))?;
    let mut q = scaled / d;
    let r = scaled % d;
    if !truncate && 2 * r >= d { q += 1; }
    let q = if negative { -q } else { q };
    i64::try_from(q).ok()
}

fn has_digit(text: &str) -> bool { text.chars().any(|c| c.is_ascii_digit() || ('０'..='９').contains(&c)) }

fn check_values(values: &[V3Value], max_level: u8, owner: &str) -> Result<BTreeSet<String>, String> {
    let mut names = BTreeSet::new();
    for v in values {
        if v.name.is_empty() || !v.name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') {
            return Err(format!("{owner}: invalid value name '{}'", v.name));
        }
        if !names.insert(v.name.clone()) { return Err(format!("{owner}: duplicate value '{}'", v.name)); }
        if !UNITS.contains(&v.unit.as_str()) { return Err(format!("{owner}: value '{}' has unknown unit '{}'", v.name, v.unit)); }
        if v.ranks.is_empty() {
            if v.expr.as_deref().is_none_or(str::is_empty) || !v.raw.is_empty() {
                return Err(format!("{owner}: value '{}' needs ranks or a runtime expr", v.name));
            }
            continue;
        }
        let want = if v.per.as_deref() == Some("hero_level") { 25 } else { usize::from(max_level) };
        if v.ranks.len() != want || v.raw.len() != want {
            return Err(format!("{owner}: value '{}' needs {want} ranks and raw values", v.name));
        }
        for (text, raw) in v.ranks.iter().zip(&v.raw) {
            let nearest = raw_of(text, false).ok_or_else(|| format!("{owner}: value '{}' rank '{text}' is not decimal", v.name))?;
            let truncated = raw_of(text, true).unwrap_or(nearest);
            if *raw != nearest && *raw != truncated {
                return Err(format!("{owner}: value '{}' raw {raw} does not match '{text}'", v.name));
            }
        }
    }
    Ok(names)
}

fn check_text(text: &BTreeMap<String, V3Text>, names: &BTreeSet<String>, owner: &str, required: bool) -> Result<(), String> {
    if text.is_empty() && !required { return Ok(()); }
    for lang in LANGS {
        let Some(t) = text.get(lang) else { return Err(format!("{owner}: text.{lang} is required")); };
        if t.name.is_empty() || t.name.len() > 256 { return Err(format!("{owner}: text.{lang}.name is empty or too long")); }
        if t.body.as_deref().is_some_and(|b| b.len() > 4096) { return Err(format!("{owner}: text.{lang}.body is too long")); }
        if t.body.is_some() && t.pieces.is_empty() { return Err(format!("{owner}: text.{lang} has a body but no pieces")); }
        for p in &t.pieces {
            let kinds = [p.value.is_some(), p.name_of.is_some(), p.rules.is_some()].iter().filter(|b| **b).count();
            if kinds > 1 || (kinds == 0 && p.text.is_none()) { return Err(format!("{owner}: text.{lang} piece is ambiguous")); }
            if kinds == 0 && p.text.as_deref().is_some_and(has_digit) {
                return Err(format!("{owner}: text.{lang} literal contains a number (values belong in placeholders)"));
            }
            if let Some(value) = &p.value {
                if p.owner.is_none() && !names.contains(value) {
                    return Err(format!("{owner}: text.{lang} placeholder '{value}' is not a value"));
                }
            }
            if p.format.as_deref().is_some_and(|f| !FORMATS.contains(&f)) {
                return Err(format!("{owner}: text.{lang} has unknown format"));
            }
        }
    }
    if text.keys().any(|k| !LANGS.contains(&k.as_str())) { return Err(format!("{owner}: text has an unknown language")); }
    Ok(())
}

/// Checks a block against its row: kind, schema, rank counts, raw values, references and texts.
pub fn validate_block(block: &V3Block, kind: &str, max_level: u8) -> Result<(), String> {
    let owner = format!("v3 {}", block.source_id);
    if !block.schema.strip_prefix("3.").is_some_and(|m| !m.is_empty() && m.bytes().all(|b| b.is_ascii_digit())) {
        return Err(format!("{owner}: schema must be 3.x"));
    }
    if block.kind != kind { return Err(format!("{owner}: kind must be {kind}")); }
    if block.max_level != max_level || max_level == 0 { return Err(format!("{owner}: max_level must equal the row's {max_level}")); }
    if block.source_id.is_empty() || block.source_id.len() > 128 { return Err(format!("{owner}: source_id is required")); }
    let names = check_values(&block.values, max_level, &owner)?;
    if let Some(ir) = &block.ir { ir.validate().map_err(|e| format!("{owner}: ir: {e}"))?; }
    for r in block.info.iter().chain(&block.stat_lines) {
        if r.field.as_deref().is_some_and(|f| !INFO_FIELDS.contains(&f)) { return Err(format!("{owner}: unknown info field")); }
        match (&r.value, &r.expr) {
            (Some(v), None) => if r.owner.is_none() && !names.contains(v) { return Err(format!("{owner}: info refers to missing value '{v}'")); },
            (None, Some(e)) if !e.is_empty() => {}
            _ => return Err(format!("{owner}: info row needs a value or an expr")),
        }
    }
    check_text(&block.text, &names, &owner, true)?;
    for s in &block.statuses {
        let snames = check_values(&s.values, max_level, &format!("{owner} status {}", s.id))?;
        let mut all = names.clone();
        all.extend(snames);
        check_text(&s.text, &all, &format!("{owner} status {}", s.id), false)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block() -> V3Block {
        serde_json::from_value(serde_json::json!({
            "schema": "3.0", "kind": "ability", "source_id": "C17R", "max_level": 2,
            "values": [
                {"name": "damage", "unit": "dmg", "ranks": ["200", "300"], "raw": [204800, 307200]},
                {"name": "chance", "unit": "pct", "ranks": ["1900/101", "1900/101"], "raw": [19263, 19263]},
                {"name": "scale", "unit": "dmg", "expr": "caster.agi * $damage", "derived": true}
            ],
            "info": [{"field": "cooldown", "value": "damage"}],
            "text": {
                "zh_TW": {"name": "神助一擊", "body": "造成 {damage} 傷害", "pieces": [{"text": "造成 "}, {"value": "damage"}, {"text": " 傷害"}]},
                "en": {"name": "Divine Strike", "body": "Deals {damage}", "pieces": [{"text": "Deals "}, {"value": "damage"}]}
            }
        })).unwrap()
    }

    #[test]
    fn raw_values_come_from_the_decimal_text() {
        assert_eq!(raw_of("0.53", false), Some(543));
        assert_eq!(raw_of("-0.53", false), Some(-543));
        assert_eq!(raw_of("4.2", false), Some(4301));
        assert_eq!(raw_of("0.15", true), Some(153));
        assert_eq!(raw_of("0.15", false), Some(154));
        assert_eq!(raw_of("1900/101", false), Some(19263));
        assert_eq!(raw_of("12", false), Some(12288));
        for bad in ["", "1.", "a", "1/0", "1e3", "--1"] { assert_eq!(raw_of(bad, false), None, "{bad}"); }
    }

    #[test]
    fn a_consistent_block_validates_and_answers_ranks() {
        let b = block();
        validate_block(&b, "ability", 2).unwrap();
        assert_eq!(b.raw_at("damage", 2), Some(307200));
        assert_eq!(b.raw_at("damage", 9), Some(307200));
        assert_eq!(b.raw_at("scale", 1), None);
    }

    #[test]
    fn inconsistent_blocks_are_rejected() {
        let cases: Vec<Box<dyn Fn(&mut V3Block)>> = vec![
            Box::new(|b| b.values[0].raw[1] = 1),
            Box::new(|b| { b.values[0].ranks.pop(); }),
            Box::new(|b| b.values[0].unit = "parsec".into()),
            Box::new(|b| b.info[0].value = Some("missing".into())),
            Box::new(|b| b.text.get_mut("en").unwrap().pieces[0].text = Some("Deals 5 ".into())),
            Box::new(|b| { b.text.remove("en"); }),
            Box::new(|b| b.text.get_mut("zh_TW").unwrap().pieces[1].value = Some("nope".into())),
            Box::new(|b| b.schema = "2".into()),
            Box::new(|b| b.max_level = 3),
        ];
        for (i, mutate) in cases.iter().enumerate() {
            let mut b = block();
            mutate(&mut b);
            assert!(validate_block(&b, "ability", 2).is_err(), "case {i}");
        }
        assert!(validate_block(&block(), "item", 2).is_err());
    }
}
