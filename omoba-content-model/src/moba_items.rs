use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct MobaItemEntry {
    pub catalog_id: u16,
    pub id: String,
    pub name: String,
    pub cost: i32,
    #[serde(default)]
    pub atk: f32,
    #[serde(default)]
    pub hp: f32,
    #[serde(default)]
    pub ms: f32,
    #[serde(default)]
    pub armor: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<MobaItemActiveEntry>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub cooldown: f32,
    #[serde(default)]
    pub recipe: Vec<String>,
    /// Omitted on legacy self items so their catalog JSON stays stable.
    #[serde(default, skip_serializing_if = "is_legacy_self_target")]
    pub target: MobaItemTargetEntry,
}

pub fn validate_moba_items(items: &[MobaItemEntry]) -> Result<(), String> {
    let by_id: BTreeMap<_, _> = items.iter().map(|item| (item.id.as_str(), item)).collect();
    let numeric: BTreeSet<_> = items.iter().map(|item| item.catalog_id).collect();
    if by_id.len() != items.len() || numeric.len() != items.len() || numeric.contains(&0) {
        return Err("MOBA items contain duplicate ids".into());
    }
    for item in items {
        if !item.cooldown.is_finite() || !(0.0..=3600.0).contains(&item.cooldown)
            || (item.cooldown > 0.0 && item.cooldown < 1.0 / 1024.0)
            || (item.active.is_none() && item.cooldown != 0.0) {
            return Err(format!("invalid MOBA item '{}' cooldown", item.id));
        }
        if let Some(active) = &item.active {
            active.validate().map_err(|error| format!("MOBA item '{}': {error}", item.id))?;
        }
        item.target.validate(item.active.as_ref())
            .map_err(|error| format!("MOBA item '{}': {error}", item.id))?;
        if item.id.is_empty() || item.id.len() > 64
            || !item.id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
            || item.name.is_empty() || item.name.len() > 128 || item.cost <= 0
            || item.recipe.len() > 6
            || [item.atk, item.hp, item.ms, item.armor].iter()
                .any(|v| !v.is_finite() || *v < 0.0 || *v > 100_000.0
                    || (*v > 0.0 && (*v * 1024.0).round() == 0.0))
        {
            return Err(format!("invalid MOBA item '{}'", item.id));
        }
        let mut materials = 0i32;
        for id in &item.recipe {
            let component = by_id.get(id.as_str())
                .ok_or_else(|| format!("MOBA item '{}' has unknown component '{id}'", item.id))?;
            // Strictly increasing full price also prevents recipe cycles.
            if component.cost <= 0 || component.cost >= item.cost {
                return Err(format!("MOBA item '{}' has non-increasing recipe price", item.id));
            }
            materials = materials.checked_add(component.cost)
                .ok_or_else(|| format!("MOBA item '{}' recipe cost overflow", item.id))?;
        }
        if materials > item.cost {
            return Err(format!("MOBA item '{}' recipe exceeds full price", item.id));
        }
    }
    Ok(())
}

fn is_zero(value: &f32) -> bool { *value == 0.0 }

fn is_legacy_self_target(target: &MobaItemTargetEntry) -> bool {
    target.is_legacy_self()
}

/// Catalog target contract. Self is the legacy default and is not a missing point.
/// Targeted forms reject when out of range. They do not queue or approach.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct MobaItemTargetEntry {
    pub shape: MobaItemTargetShape,
    pub range: f32,
    pub team: MobaItemTeamFilter,
    pub requires_visible: bool,
    pub out_of_range: MobaItemOutOfRange,
}

impl Default for MobaItemTargetEntry {
    fn default() -> Self { Self::legacy_self() }
}

impl MobaItemTargetEntry {
    pub fn legacy_self() -> Self {
        Self {
            shape: MobaItemTargetShape::Self_,
            range: 0.0,
            team: MobaItemTeamFilter::Any,
            requires_visible: false,
            out_of_range: MobaItemOutOfRange::Reject,
        }
    }

    pub fn is_legacy_self(&self) -> bool {
        *self == Self::legacy_self()
    }

    fn validate(&self, active: Option<&MobaItemActiveEntry>) -> Result<(), String> {
        let positive = |value: f32, maximum: f32| value.is_finite() && (1.0 / 1024.0..=maximum).contains(&value);
        let legacy = self.is_legacy_self();
        let shape_ok = match self.shape {
            MobaItemTargetShape::Self_ => legacy && matches!(active,
                None | Some(MobaItemActiveEntry::Shield { .. } | MobaItemActiveEntry::SprintBuff { .. }
                    | MobaItemActiveEntry::RestoreMana { .. } | MobaItemActiveEntry::DamageReduce { .. }
                    | MobaItemActiveEntry::HeadshotNext { .. })),
            MobaItemTargetShape::Point => !legacy && positive(self.range, 10_000.0)
                && self.team == MobaItemTeamFilter::Enemy && self.requires_visible
                && self.out_of_range == MobaItemOutOfRange::Reject
                && matches!(active, Some(MobaItemActiveEntry::PointNova { .. })),
            MobaItemTargetShape::Unit => !legacy && positive(self.range, 10_000.0)
                && self.team == MobaItemTeamFilter::Enemy && self.requires_visible
                && self.out_of_range == MobaItemOutOfRange::Reject
                && matches!(active, Some(MobaItemActiveEntry::UnitHex { .. })),
        };
        if shape_ok { Ok(()) } else { Err("target contract does not match the active effect".into()) }
    }

    pub fn rust_literal(&self) -> String {
        let fixed = |v: f32| format!("Fixed64::from_raw({})", (f64::from(v) * 1024.0).round() as i64);
        let shape = match self.shape {
            MobaItemTargetShape::Self_ => "Self_",
            MobaItemTargetShape::Point => "Point",
            MobaItemTargetShape::Unit => "Unit",
        };
        let team = match self.team {
            MobaItemTeamFilter::Any => "Any",
            MobaItemTeamFilter::Enemy => "Enemy",
        };
        format!(
            "MobaItemTargetConst {{ shape: MobaItemTargetShapeConst::{shape}, range: {}, team: MobaItemTeamConst::{team}, requires_visible: {}, out_of_range: MobaItemOutOfRangeConst::Reject }}",
            fixed(self.range), self.requires_visible,
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MobaItemTargetShape {
    #[default]
    #[serde(rename = "self")]
    Self_,
    Point,
    Unit,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MobaItemTeamFilter {
    #[default]
    Any,
    Enemy,
}

/// The only authored out-of-range policy. Approach and queue are not item contracts.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MobaItemOutOfRange {
    #[default]
    Reject,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MobaItemActiveEntry {
    Shield { amount: f32, duration: f32 },
    SprintBuff { ms_bonus: f32, duration: f32 },
    RestoreMana { amount: f32 },
    DamageReduce { percent: f32, duration: f32 },
    HeadshotNext { bonus_damage: f32 },
    /// Flat damage to visible enemies standing in the radius of an explicit point.
    PointNova { damage: f32, radius: f32 },
    /// Flat damage to one disclosed living enemy unit.
    UnitHex { damage: f32 },
}

impl MobaItemActiveEntry {
    fn validate(&self) -> Result<(), String> {
        let positive = |value: f32, maximum: f32| value.is_finite() && (1.0 / 1024.0..=maximum).contains(&value);
        let valid = match self {
            Self::Shield { amount, duration } => positive(*amount, 1_000_000.0) && positive(*duration, 60.0),
            Self::SprintBuff { ms_bonus, duration } => positive(*ms_bonus, 10_000.0) && positive(*duration, 60.0),
            Self::RestoreMana { amount } => positive(*amount, 1_000_000.0),
            Self::DamageReduce { percent, duration } => positive(*percent, 1.0) && positive(*duration, 60.0),
            Self::HeadshotNext { bonus_damage } => positive(*bonus_damage, 1_000_000.0),
            Self::PointNova { damage, radius } => positive(*damage, 1_000_000.0) && positive(*radius, 2_000.0),
            Self::UnitHex { damage } => positive(*damage, 1_000_000.0),
        };
        if valid { Ok(()) } else { Err("invalid active effect amount or duration".into()) }
    }

    pub fn rust_literal(&self) -> String {
        let fixed = |v: f32| format!("Fixed64::from_raw({})", (f64::from(v) * 1024.0).round() as i64);
        match self {
            Self::Shield { amount, duration } => format!("MobaItemActiveConst::Shield {{ amount: {}, duration: {} }}", fixed(*amount), fixed(*duration)),
            Self::SprintBuff { ms_bonus, duration } => format!("MobaItemActiveConst::SprintBuff {{ ms_bonus: {}, duration: {} }}", fixed(*ms_bonus), fixed(*duration)),
            Self::RestoreMana { amount } => format!("MobaItemActiveConst::RestoreMana {{ amount: {} }}", fixed(*amount)),
            Self::DamageReduce { percent, duration } => format!("MobaItemActiveConst::DamageReduce {{ percent: {}, duration: {} }}", fixed(*percent), fixed(*duration)),
            Self::HeadshotNext { bonus_damage } => format!("MobaItemActiveConst::HeadshotNext {{ bonus_damage: {} }}", fixed(*bonus_damage)),
            Self::PointNova { damage, radius } => format!("MobaItemActiveConst::PointNova {{ damage: {}, radius: {} }}", fixed(*damage), fixed(*radius)),
            Self::UnitHex { damage } => format!("MobaItemActiveConst::UnitHex {{ damage: {} }}", fixed(*damage)),
        }
    }
}

