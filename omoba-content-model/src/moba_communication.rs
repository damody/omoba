//! Original communication content. Numeric identity is explicit and append-only.
//! Optional media references describe resources; they do not assert UE import success.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_COMMUNICATION_CONTENT: usize = 32;

pub fn validate_optional_catalog(value: &serde_json::Value) -> Result<(), String> {
    if let Some(raw) = value.get("moba_communication") {
        let rows: Vec<CommunicationContentEntry> = serde_json::from_value(raw.clone())
            .map_err(|e| format!("communication catalog: {e}"))?;
        validate_communication_content(&rows)?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationSemantic {
    Joke, Challenge, Dance, Laugh, Salute,
    Acknowledge, Applaud, Thanks, Surprise, Resolve, Patience, Celebrate, Regroup,
}

impl CommunicationSemantic {
    pub fn is_interaction(self) -> bool {
        matches!(self, Self::Joke | Self::Challenge | Self::Dance | Self::Laugh | Self::Salute)
    }
    pub fn rust_literal(self) -> &'static str {
        match self {
            Self::Joke => "CommunicationSemanticConst::Joke",
            Self::Challenge => "CommunicationSemanticConst::Challenge",
            Self::Dance => "CommunicationSemanticConst::Dance",
            Self::Laugh => "CommunicationSemanticConst::Laugh",
            Self::Salute => "CommunicationSemanticConst::Salute",
            Self::Acknowledge => "CommunicationSemanticConst::Acknowledge",
            Self::Applaud => "CommunicationSemanticConst::Applaud",
            Self::Thanks => "CommunicationSemanticConst::Thanks",
            Self::Surprise => "CommunicationSemanticConst::Surprise",
            Self::Resolve => "CommunicationSemanticConst::Resolve",
            Self::Patience => "CommunicationSemanticConst::Patience",
            Self::Celebrate => "CommunicationSemanticConst::Celebrate",
            Self::Regroup => "CommunicationSemanticConst::Regroup",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CommunicationContentEntry {
    pub catalog_id: u16,
    pub id: String,
    pub semantic: CommunicationSemantic,
    /// 1..=5 for character interactions, 0 for wheel emotes.
    pub interaction_slot: u8,
    /// 1..=8 for wheel emotes, 0 for character interactions.
    pub wheel_slot: u8,
    pub label: String,
    pub subtitle: String,
    pub duration_raw: u16,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub audio: Option<String>,
    #[serde(default)]
    pub animation: Option<String>,
}

pub fn validate_communication_content(rows: &[CommunicationContentEntry]) -> Result<(), String> {
    if rows.is_empty() || rows.len() > MAX_COMMUNICATION_CONTENT {
        return Err("communication catalog must be nonempty and bounded".into());
    }
    let (mut ids, mut numeric, mut semantics, mut interactions, mut wheel) =
        (BTreeSet::new(), BTreeSet::new(), BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    let mut resources = BTreeSet::new();
    let text = |s: &str, max: usize| !s.is_empty() && s.chars().count() <= max
        && s.chars().any(|c| !c.is_whitespace())
        && s.chars().all(|c| !c.is_control() && c != '\u{2028}' && c != '\u{2029}');
    for row in rows {
        if row.catalog_id == 0 || row.catalog_id > 1024
            || !numeric.insert(row.catalog_id) || !ids.insert(row.id.as_str())
            || !semantics.insert(row.semantic)
            || row.id.is_empty() || row.id.len() > 64
            || !row.id.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
            || !text(&row.label, 16) || !text(&row.subtitle, 80)
            || !(1024..=8192).contains(&row.duration_raw)
        {
            return Err(format!("invalid or duplicate communication content '{}'", row.id));
        }
        let expected_slot = match row.semantic {
            CommunicationSemantic::Joke => 1, CommunicationSemantic::Challenge => 2,
            CommunicationSemantic::Dance => 3, CommunicationSemantic::Laugh => 4,
            CommunicationSemantic::Salute => 5, _ => 0,
        };
        if row.semantic.is_interaction() {
            if row.interaction_slot != expected_slot || row.wheel_slot != 0
                || !interactions.insert(row.interaction_slot) {
                return Err("invalid character interaction slot".into());
            }
        } else if row.interaction_slot != 0 || !(1..=8).contains(&row.wheel_slot)
            || !wheel.insert(row.wheel_slot) {
            return Err("invalid emote wheel slot".into());
        }
        for (kind, resource) in [&row.icon, &row.audio, &row.animation].into_iter().enumerate() {
            let Some(resource) = resource else { continue };
            if !resource.starts_with("/Game/Omoba/") || resource.len() > 192
                || resource.contains("..") || resource.ends_with('/')
                || !resource.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'/' | b'_' | b'.')) {
                return Err(format!("invalid communication media reference '{}'", row.id));
            }
            if !resources.insert((kind, resource.as_str())) {
                return Err("distinct communication semantics cannot alias the same media resource".into());
            }
        }
    }
    if interactions.len() != 5 || wheel.is_empty() {
        return Err("communication catalog requires five interactions and a nonempty emote wheel".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rows() -> Vec<CommunicationContentEntry> {
        [CommunicationSemantic::Joke, CommunicationSemantic::Challenge, CommunicationSemantic::Dance,
         CommunicationSemantic::Laugh, CommunicationSemantic::Salute, CommunicationSemantic::Thanks]
            .into_iter().enumerate().map(|(i, semantic)| CommunicationContentEntry {
                catalog_id: i as u16 + 1, id: format!("original_{i}"), semantic,
                interaction_slot: if i < 5 { i as u8 + 1 } else { 0 },
                wheel_slot: if i == 5 { 1 } else { 0 }, label: "招呼".into(),
                subtitle: "並肩向前。".into(), duration_raw: 2048,
                icon: None, audio: None, animation: None,
            }).collect()
    }
    #[test]
    fn communication_catalog_rejects_identity_semantic_slot_text_and_media_errors() {
        let good = rows(); validate_communication_content(&good).unwrap();
        let mut reverse = good.clone(); reverse.reverse(); validate_communication_content(&reverse).unwrap();
        assert!(validate_communication_content(&[]).is_err());
        for case in 0..12 {
            let mut bad = good.clone();
            match case {
                0 => bad[1].catalog_id = bad[0].catalog_id,
                1 => bad[1].id = bad[0].id.clone(),
                2 => bad[1].semantic = bad[0].semantic,
                3 => bad[0].interaction_slot = 2,
                4 => bad[5].wheel_slot = 9,
                5 => bad[0].label = "\n".into(),
                6 => bad[0].subtitle = "x".repeat(81),
                7 => bad[0].duration_raw = 0,
                8 => bad[0].icon = Some("/Game/Omoba/../External".into()),
                9 => bad[0].audio = Some("https://example.invalid/audio".into()),
                10 => bad[0].animation = Some(String::new()),
                11 => { bad.remove(0); }, _ => unreachable!(),
            }
            assert!(validate_communication_content(&bad).is_err(), "case {case}");
        }
        let mut raw = serde_json::to_value(&good[0]).unwrap();
        raw["semantic"] = serde_json::json!("unknown");
        assert!(serde_json::from_value::<CommunicationContentEntry>(raw).is_err());
        let mut aliased = good.clone();
        aliased[0].icon = Some("/Game/Omoba/UI/Shared".into());
        aliased[1].icon = aliased[0].icon.clone();
        assert!(validate_communication_content(&aliased).is_err());
    }
    #[test]
    fn communication_catalog_hash_covers_text_slots_semantics_and_optional_resources() {
        let good = rows(); let raw = serde_json::to_value(&good).unwrap();
        let hash = crate::canonical_template_hash(&raw).unwrap();
        for key in ["catalog_id", "id", "semantic", "interaction_slot", "wheel_slot", "label", "subtitle", "duration_raw", "icon", "audio", "animation"] {
            let mut changed = raw.clone(); changed[0][key] = serde_json::json!("changed");
            assert_ne!(hash, crate::canonical_template_hash(&changed).unwrap(), "{key}");
        }
        assert!(good.iter().all(|r| r.icon.is_none() && r.audio.is_none() && r.animation.is_none()));
    }
}
