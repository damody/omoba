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
    #[serde(default, skip_serializing_if = "is_zero")]
    pub lifesteal: f32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub critical_chance: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique_passive: Option<MobaItemUniquePassive>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purchase_group: Option<MobaItemPurchaseGroup>,
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
    validate_purchase_groups(items.iter().filter_map(|item|item.purchase_group.as_ref()))?;
    let by_id: BTreeMap<_, _> = items.iter().map(|item| (item.id.as_str(), item)).collect();
    let numeric: BTreeSet<_> = items.iter().map(|item| item.catalog_id).collect();
    if by_id.len() != items.len() || numeric.len() != items.len() || numeric.contains(&0) {
        return Err("MOBA items contain duplicate ids".into());
    }
    for item in items {
        if let Some(passive)=&item.unique_passive {passive.validate()?;}
        if !item.lifesteal.is_finite() || !(0.0..=1.0).contains(&item.lifesteal)
            || (item.lifesteal>0.0 && item.lifesteal<1.0/1024.0) {
            return Err(format!("invalid MOBA item '{}' lifesteal",item.id));
        }
        if !item.critical_chance.is_finite() || !(0.0..=1.0).contains(&item.critical_chance)
            || (item.critical_chance>0.0 && item.critical_chance<1.0/1024.0) {
            return Err(format!("invalid MOBA item '{}' critical_chance",item.id));
        }
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MobaItemPurchaseGroup {pub id:String,pub name:String,pub limit:u8}

impl MobaItemPurchaseGroup {
    pub fn validate(&self)->Result<(),String> {
        if self.id.is_empty() || self.id.len()>64
            || !self.id.bytes().all(|b|b.is_ascii_lowercase()||b.is_ascii_digit()||b==b'_')
            || self.name.trim().is_empty() || self.name.len()>128 || self.name.chars().any(char::is_control)
            || !(1..=6).contains(&self.limit) {return Err("invalid MOBA item purchase_group".into());}
        Ok(())
    }
    pub fn rust_literal(&self)->String {
        format!("MobaItemPurchaseGroupConst {{ id: {:?}, name: {:?}, limit: {} }}",self.id,self.name,self.limit)
    }
}

pub fn validate_purchase_groups<'a>(groups:impl IntoIterator<Item=&'a MobaItemPurchaseGroup>)->Result<(),String> {
    let mut definitions=BTreeMap::new();
    for group in groups {
        group.validate()?;
        let value=(group.name.as_str(),group.limit);
        if definitions.insert(group.id.as_str(),value).is_some_and(|old|old!=value) {
            return Err(format!("inconsistent purchase_group '{}'",group.id));
        }
    }
    Ok(())
}

#[cfg(test)]
#[test]
fn unique_passive_rejects_malformed_groups_values_and_unknown_fields() {
    let base=serde_json::json!({"group":"arbitrary_group","name":"Passive","priority":1,"ms":25});
    let parse=|value|serde_json::from_value::<MobaItemUniquePassive>(value);
    parse(base.clone()).unwrap().validate().unwrap();
    for (key,value) in [("group",serde_json::json!("")),("group",serde_json::json!("Bad Group")),
        ("name",serde_json::json!("\n")),("priority",serde_json::json!(0)),("priority",serde_json::json!(65536)),
        ("ms",serde_json::json!(-1)),("ms",serde_json::json!(0)),("ms",serde_json::json!(0.0001)),
        ("lifesteal",serde_json::json!(1.01)),("critical_chance",serde_json::json!("0.5")),("unknown",serde_json::json!(1))] {
        let mut v=base.clone();v[key]=value;assert!(parse(v).map_or(true,|v|v.validate().is_err()),"{key}");
    }
    for bad in [f32::NAN,f32::INFINITY] {let mut v=parse(base.clone()).unwrap();v.hp=bad;assert!(v.validate().is_err());}
    let legacy:MobaItemEntry=serde_json::from_value(serde_json::json!({"catalog_id":1,"id":"legacy","name":"Legacy","cost":1})).unwrap();
    assert!(serde_json::to_value(legacy).unwrap().get("unique_passive").is_none());
}

#[cfg(test)]
#[test]
fn purchase_groups_validate_limits_consistency_and_legacy_omission() {
    let base=serde_json::json!({"id":"arbitrary_group","name":"Equipment","limit":2});
    let parse=|v|serde_json::from_value::<MobaItemPurchaseGroup>(v);
    let valid=parse(base.clone()).unwrap();validate_purchase_groups([&valid,&valid]).unwrap();
    for (key,value) in [("id",serde_json::json!("")),("id",serde_json::json!("Bad Id")),("name",serde_json::json!("\n")),
        ("limit",serde_json::json!(0)),("limit",serde_json::json!(7)),("limit",serde_json::json!(1.5)),("unknown",serde_json::json!(1))] {
        let mut v=base.clone();v[key]=value;assert!(parse(v).map_or(true,|v|v.validate().is_err()),"{key}");
    }
    let mut other=valid.clone();other.limit=3;assert!(validate_purchase_groups([&valid,&other]).is_err());
    other=valid.clone();other.name="Different label".into();assert!(validate_purchase_groups([&other,&valid]).is_err());
    other.id="independent".into();validate_purchase_groups([&valid,&other]).unwrap();
    let row:MobaItemEntry=serde_json::from_value(serde_json::json!({"catalog_id":1,"id":"legacy","name":"Legacy","cost":1})).unwrap();
    assert!(serde_json::to_value(row).unwrap().get("purchase_group").is_none());
}

/// One indivisible passive. Highest priority wins per group; item id breaks ties.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MobaItemUniquePassive {
    pub group: String,
    pub name: String,
    pub priority: u16,
    #[serde(default)] pub atk: f32,
    #[serde(default)] pub hp: f32,
    #[serde(default)] pub ms: f32,
    #[serde(default)] pub armor: f32,
    #[serde(default)] pub lifesteal: f32,
    #[serde(default)] pub critical_chance: f32,
}

impl MobaItemUniquePassive {
    pub fn validate(&self)->Result<(),String> {
        let stats=[self.atk,self.hp,self.ms,self.armor];
        let rates=[self.lifesteal,self.critical_chance];
        if self.group.is_empty() || self.group.len()>64
            || !self.group.bytes().all(|b|b.is_ascii_lowercase()||b.is_ascii_digit()||b==b'_')
            || self.name.trim().is_empty() || self.name.len()>128 || self.name.chars().any(char::is_control)
            || self.priority==0
            || stats.iter().any(|v|!v.is_finite()||*v<0.||*v>100_000.||(*v>0.&&*v<1./1024.))
            || rates.iter().any(|v|!v.is_finite()||*v<0.||*v>1.||(*v>0.&&*v<1./1024.))
            || stats.iter().chain(rates.iter()).all(|v|*v==0.) {
            return Err("invalid MOBA item unique_passive".into());
        }
        Ok(())
    }

    pub fn rust_literal(&self)->String {
        let fixed=|v:f32|format!("Fixed64::from_raw({})",(f64::from(v)*1024.).round() as i64);
        format!("MobaItemUniquePassiveConst {{ group: {:?}, name: {:?}, priority: {}, atk: {}, hp: {}, ms: {}, armor: {}, lifesteal: {}, critical_chance: {} }}",
            self.group,self.name,self.priority,fixed(self.atk),fixed(self.hp),fixed(self.ms),fixed(self.armor),fixed(self.lifesteal),fixed(self.critical_chance))
    }
}

#[cfg(test)]
#[test]
fn item_lifesteal_is_bounded_quantized_and_legacy_zero_is_omitted() {
    let base=serde_json::json!({"catalog_id":491,"id":"arbitrary_passive","name":"Passive","cost":1});
    let legacy:MobaItemEntry=serde_json::from_value(base.clone()).unwrap();
    assert_eq!(legacy.lifesteal,0.);assert!(serde_json::to_value(legacy).unwrap().get("lifesteal").is_none());
    for value in [0.,0.125,0.25,1.] {let mut v=base.clone();v["lifesteal"]=serde_json::json!(value);validate_moba_items(&[serde_json::from_value(v).unwrap()]).unwrap();}
    for value in [-0.1,1.01,0.0001,f32::NAN,f32::INFINITY] {let mut v: MobaItemEntry=serde_json::from_value(base.clone()).unwrap();v.lifesteal=value;assert!(validate_moba_items(&[v]).is_err());}
    for value in [serde_json::json!("0.5"),serde_json::Value::Null] {let mut v=base.clone();v["lifesteal"]=value;assert!(serde_json::from_value::<MobaItemEntry>(v).is_err());}
}

fn is_legacy_self_target(target: &MobaItemTargetEntry) -> bool {
    target.is_legacy_self()
}

#[test]
fn hostile_item_schema_is_explicit_closed_and_visible_for_both_shapes() {
    for (shape,active) in [("unit",serde_json::json!({"kind":"unit_hex","damage":10})),
        ("point",serde_json::json!({"kind":"point_nova","damage":10,"radius":80}))] {
        for team in ["enemy","hostile"] {
            let mut value=serde_json::json!({"catalog_id":491,"id":"arbitrary_offense","name":"Fixture","cost":1,
                "active":active,"target":{"shape":shape,"range":450,"team":team,"requires_visible":true}});
            let row: MobaItemEntry=serde_json::from_value(value.clone()).unwrap();validate_moba_items(&[row.clone()]).unwrap();
            assert!(row.target.rust_literal().contains(if team=="hostile"{"MobaItemTeamConst::Hostile"}else{"MobaItemTeamConst::Enemy"}));
            value["target"]["requires_visible"]=serde_json::json!(false);
            assert!(validate_moba_items(&[serde_json::from_value(value.clone()).unwrap()]).is_err());
            value["target"]["team"]=serde_json::json!("neutral");assert!(serde_json::from_value::<MobaItemEntry>(value).is_err());
        }
    }
    let mut target=MobaItemTargetEntry::legacy_self();target.team=MobaItemTeamFilter::Hostile;
    assert!(target.validate(None).is_err());
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
                    | MobaItemActiveEntry::ChannelHeal { .. } | MobaItemActiveEntry::SelfControlCleanse {}
                    | MobaItemActiveEntry::RestoreMana { .. } | MobaItemActiveEntry::DamageReduce { .. }
                    | MobaItemActiveEntry::HeadshotNext { .. })),
            MobaItemTargetShape::Point => !legacy && positive(self.range, 10_000.0)
                && matches!(self.team, MobaItemTeamFilter::Enemy | MobaItemTeamFilter::Hostile) && self.requires_visible
                && self.out_of_range == MobaItemOutOfRange::Reject
                && matches!(active, Some(MobaItemActiveEntry::PointNova { .. })),
            MobaItemTargetShape::Unit => !legacy && positive(self.range, 10_000.0)
                && matches!(self.team, MobaItemTeamFilter::Enemy | MobaItemTeamFilter::Hostile) && self.requires_visible
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
            MobaItemTeamFilter::Hostile => "Hostile",
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
    /// Source faction hostility, including aggressive neutrals; passive neutrals stay excluded.
    Hostile,
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
    SelfControlCleanse {},
    ChannelHeal { amount:f32, duration:f32, interval:f32 },
    Shield { amount: f32, duration: f32 },
    SprintBuff { ms_bonus: f32, duration: f32 },
    RestoreMana { amount: f32 },
    DamageReduce { percent: f32, duration: f32 },
    HeadshotNext { bonus_damage: f32 },
    /// Flat damage to visible enemies standing in the radius of an explicit point.
    PointNova { damage: f32, radius: f32 },
    /// Flat damage to one disclosed living enemy unit.
    UnitHex { damage: f32, #[serde(default)] healing_reduction:Option<HealingReductionEntry>,
        #[serde(default,skip_serializing_if="Option::is_none")] control:Option<ItemControlEntry> },
}

#[derive(Debug,Deserialize,Serialize,Clone)]
#[serde(deny_unknown_fields)]
pub struct ItemControlEntry {pub kind:crate::ControlEffectKind,pub duration:f32}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct HealingReductionEntry {pub multiplier:f32,pub duration:f32}

impl MobaItemActiveEntry {
    fn validate(&self) -> Result<(), String> {
        let positive = |value: f32, maximum: f32| value.is_finite() && (1.0 / 1024.0..=maximum).contains(&value);
        let valid = match self {
            Self::SelfControlCleanse {} => true,
            Self::ChannelHeal { amount,duration,interval } => positive(*amount,1_000_000.0)
                && positive(*duration,60.0) && interval.is_finite() && (0.25..=10.0).contains(interval)
                && duration>=interval && ((*duration*1024.0).round() as i64)%((*interval*1024.0).round() as i64)==0,
            Self::Shield { amount, duration } => positive(*amount, 1_000_000.0) && positive(*duration, 60.0),
            Self::SprintBuff { ms_bonus, duration } => positive(*ms_bonus, 10_000.0) && positive(*duration, 60.0),
            Self::RestoreMana { amount } => positive(*amount, 1_000_000.0),
            Self::DamageReduce { percent, duration } => positive(*percent, 1.0) && positive(*duration, 60.0),
            Self::HeadshotNext { bonus_damage } => positive(*bonus_damage, 1_000_000.0),
            Self::PointNova { damage, radius } => positive(*damage, 1_000_000.0) && positive(*radius, 2_000.0),
            Self::UnitHex { damage,healing_reduction,control } => positive(*damage, 1_000_000.0)
                && control.as_ref().is_none_or(|c|positive(c.duration,60.0))
                && healing_reduction.as_ref().is_none_or(|r|r.multiplier.is_finite() && (0.0..1.0).contains(&r.multiplier)
                    && (r.multiplier*1024.0).round()<1024.0 && positive(r.duration,60.0)),
        };
        if valid { Ok(()) } else { Err("invalid active effect amount or duration".into()) }
    }

    pub fn rust_literal(&self) -> String {
        let fixed = |v: f32| format!("Fixed64::from_raw({})", (f64::from(v) * 1024.0).round() as i64);
        match self {
            Self::SelfControlCleanse {} => "MobaItemActiveConst::SelfControlCleanse".into(),
            Self::ChannelHeal {amount,duration,interval} => format!("MobaItemActiveConst::ChannelHeal {{ amount: {}, duration: {}, interval: {} }}",fixed(*amount),fixed(*duration),fixed(*interval)),
            Self::Shield { amount, duration } => format!("MobaItemActiveConst::Shield {{ amount: {}, duration: {} }}", fixed(*amount), fixed(*duration)),
            Self::SprintBuff { ms_bonus, duration } => format!("MobaItemActiveConst::SprintBuff {{ ms_bonus: {}, duration: {} }}", fixed(*ms_bonus), fixed(*duration)),
            Self::RestoreMana { amount } => format!("MobaItemActiveConst::RestoreMana {{ amount: {} }}", fixed(*amount)),
            Self::DamageReduce { percent, duration } => format!("MobaItemActiveConst::DamageReduce {{ percent: {}, duration: {} }}", fixed(*percent), fixed(*duration)),
            Self::HeadshotNext { bonus_damage } => format!("MobaItemActiveConst::HeadshotNext {{ bonus_damage: {} }}", fixed(*bonus_damage)),
            Self::PointNova { damage, radius } => format!("MobaItemActiveConst::PointNova {{ damage: {}, radius: {} }}", fixed(*damage), fixed(*radius)),
            Self::UnitHex { damage,healing_reduction,control } => {
                let reduction=healing_reduction.as_ref().map_or("None".to_owned(),|r|format!("Some(MobaHealingReductionConst {{ multiplier:{},duration:{} }})",fixed(r.multiplier),fixed(r.duration)));
                let control=control.as_ref().map_or("None".to_owned(),|c|format!("Some(MobaItemControlConst {{kind:MobaItemControlKindConst::{:?},duration:{} }})",c.kind,fixed(c.duration)));
                format!("MobaItemActiveConst::UnitHex {{ damage: {},healing_reduction:{reduction},control:{control} }}", fixed(*damage))
            },
        }
    }
}

#[cfg(test)]
mod channel_tests {
    use super::*;
    #[test]
    fn control_cleanse_is_strict_self_operation_without_unpublished_immunity() {
        let value=serde_json::json!({"catalog_id":399,"id":"arbitrary_cleanse","name":"Cleanse","cost":1,"cooldown":45,"active":{"kind":"self_control_cleanse"}});
        let row:MobaItemEntry=serde_json::from_value(value.clone()).unwrap();validate_moba_items(&[row.clone()]).unwrap();
        assert_eq!(row.active.unwrap().rust_literal(),"MobaItemActiveConst::SelfControlCleanse");
        for field in ["immunity","duration","target_buff"] {let mut bad=value.clone();bad["active"][field]=true.into();assert!(serde_json::from_value::<MobaItemEntry>(bad).is_err());}
        let mut bad=value;bad["target"]=serde_json::json!({"shape":"unit","range":450,"team":"enemy","requires_visible":true,"out_of_range":"reject"});
        assert!(validate_moba_items(&[serde_json::from_value(bad).unwrap()]).is_err());
    }
    #[test]
    fn targeted_control_is_closed_bounded_and_optional() {
        for kind in ["stun","root","silence"] {
            let value=serde_json::json!({"kind":"unit_hex","damage":30,"control":{"kind":kind,"duration":1.5}});
            let effect:MobaItemActiveEntry=serde_json::from_value(value.clone()).unwrap();effect.validate().unwrap();
            assert!(effect.rust_literal().contains("duration:Fixed64::from_raw(1536)"));
            for duration in [0.,-1.,0.0001,60.001] {
                let mut bad=value.clone();bad["control"]["duration"]=serde_json::json!(duration);
                assert!(serde_json::from_value::<MobaItemActiveEntry>(bad).unwrap().validate().is_err());
            }
            let mut bad=value;bad["control"]["typo"]=true.into();assert!(serde_json::from_value::<MobaItemActiveEntry>(bad).is_err());
        }
        assert!(serde_json::from_value::<MobaItemActiveEntry>(serde_json::json!({"kind":"unit_hex","damage":30,"control":{"kind":"arbitrary_buff","duration":1}})).is_err());
        let old:MobaItemActiveEntry=serde_json::from_value(serde_json::json!({"kind":"unit_hex","damage":55})).unwrap();
        assert!(old.rust_literal().contains("control:None"));
    }
    #[test]
    fn channel_healing_requires_bounded_integral_pulses_and_a_self_contract() {
        let value=serde_json::json!({"catalog_id":31,"id":"arbitrary_channel","name":"Recovery","cost":500,"cooldown":9,
            "active":{"kind":"channel_heal","amount":40,"duration":3,"interval":1}});
        let row: MobaItemEntry=serde_json::from_value(value.clone()).unwrap();validate_moba_items(&[row.clone()]).unwrap();
        assert!(row.active.unwrap().rust_literal().contains("MobaItemActiveConst::ChannelHeal"));
        for (field,bad) in [("amount",0.0),("amount",1_000_001.0),("duration",61.0),("duration",0.5),("duration",2.5),("interval",0.0),("interval",0.249),("interval",11.0)] {
            let mut invalid=value.clone();invalid["active"][field]=serde_json::json!(bad);
            let row=serde_json::from_value::<MobaItemEntry>(invalid).unwrap();assert!(validate_moba_items(&[row]).is_err(),"{field}={bad}");
        }
        let mut invalid=value.clone();invalid["active"]["early_heal"]=true.into();assert!(serde_json::from_value::<MobaItemEntry>(invalid).is_err());
        let mut invalid=value;invalid["target"]=serde_json::json!({"shape":"point","range":500,"team":"enemy","requires_visible":true,"out_of_range":"reject"});
        assert!(validate_moba_items(&[serde_json::from_value(invalid).unwrap()]).is_err());
    }
}


#[cfg(test)]
#[test]
fn item_critical_chance_is_bounded_quantized_and_legacy_zero_is_omitted() {
    let base=serde_json::json!({"catalog_id":491,"id":"arbitrary_passive","name":"Passive","cost":1});
    let legacy:MobaItemEntry=serde_json::from_value(base.clone()).unwrap();
    assert_eq!(legacy.critical_chance,0.);assert!(serde_json::to_value(legacy).unwrap().get("critical_chance").is_none());
    for value in [0.,0.125,0.25,1.] {let mut v=base.clone();v["critical_chance"]=serde_json::json!(value);validate_moba_items(&[serde_json::from_value(v).unwrap()]).unwrap();}
    for value in [-0.1,1.01,0.0001,f32::NAN,f32::INFINITY] {let mut v: MobaItemEntry=serde_json::from_value(base.clone()).unwrap();v.critical_chance=value;assert!(validate_moba_items(&[v]).is_err());}
    for value in [serde_json::json!("0.5"),serde_json::Value::Null] {let mut v=base.clone();v["critical_chance"]=value;assert!(serde_json::from_value::<MobaItemEntry>(v).is_err());}
}
