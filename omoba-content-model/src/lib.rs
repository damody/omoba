use std::collections::BTreeSet;
use serde::{Deserialize, Serialize};

/// Shared gameplay and presentation-facing ability data parsed from Lua.
/// Editor-only `ue` metadata stays with the Unreal generator.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AbilityDefinition {
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub tombstone: bool,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub ability_type: String,
    #[serde(default)]
    pub cast_type: String,
    #[serde(default)]
    pub target_type: String,
    #[serde(default = "default_ability_max_level")]
    pub max_level: u8,
    #[serde(default)]
    pub levels: Vec<AbilityLevel>,
    #[serde(default)]
    pub extras: std::collections::BTreeMap<String, Vec<f32>>,
    #[serde(default)]
    pub effects: Vec<AbilityEffect>,
}

/// Executable subset of Lua-authored effects. Amount-bearing effects resolve
/// `amount_key` from per-level `extras`; special skills still use a Rust handler.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AbilityEffect {
    SlowEnemy { reduction_key:String, duration_key:String },
    /// Instant relocation with the normal swept-terrain collision contract.
    /// Deliberately exclusive until ordered movement/effect semantics exist.
    DashToPoint,
    Damage {
        amount_key: String,
        damage_kind: EffectDamageKind,
    },
    HealSelf {
        amount_key: String,
    },
    AreaDamage {
        amount_key: String,
        radius_key: String,
        damage_kind: EffectDamageKind,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum EffectDamageKind {
    Physical,
    Magical,
    Pure,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AbilityLevel {
    #[serde(default = "default_required_hero_level")]
    pub required_hero_level: u8,
    #[serde(default)]
    pub cooldown: f32,
    #[serde(default)]
    pub mana_cost: f32,
    #[serde(default)]
    pub cast_time: f32,
    #[serde(default)]
    pub range: f32,
}

fn default_required_hero_level() -> u8 { 1 }

/// Authored numeric envelope, not a balance rule. Positive values must survive
/// Q10 quantization; the upper bound leaves headroom for composed effects.
/// Keep the fixed Lua FFI generator's envelope in sync with these constants.
pub const MIN_ABILITY_SCALAR: f32 = 1.0 / 1024.0;
pub const MAX_ABILITY_SCALAR: f32 = 1_000_000.0;

fn valid_ability_scalar(value: f32) -> bool {
    value.is_finite() && (value == 0.0 || (MIN_ABILITY_SCALAR..=MAX_ABILITY_SCALAR).contains(&value))
}

impl Default for AbilityLevel {
    fn default() -> Self {
        Self { required_hero_level: 1, cooldown: 0.0, mana_cost: 0.0, cast_time: 0.0, range: 0.0 }
    }
}

pub fn validate_ability_progression(ability: &AbilityDefinition) -> Result<(), String> {
    if ability.tombstone { return Ok(()); }
    if ability.max_level == 0 || ability.levels.len() != usize::from(ability.max_level) {
        return Err(format!("ability '{}': levels must match nonzero max_level", ability.id));
    }
    let mut previous = 1;
    for (rank, level) in ability.levels.iter().enumerate() {
        for (field, value) in [("cooldown", level.cooldown), ("mana_cost", level.mana_cost),
            ("cast_time", level.cast_time), ("range", level.range)] {
            if !valid_ability_scalar(value) {
                return Err(format!("ability '{}': rank {} {field} must be zero or finite in [1/1024,1000000]", ability.id, rank + 1));
            }
        }
        if !(1..=25).contains(&level.required_hero_level) || level.required_hero_level < previous {
            return Err(format!("ability '{}': rank {} required_hero_level must be monotonic in 1..=25", ability.id, rank + 1));
        }
        previous = level.required_hero_level;
    }
    validate_ability_effects(ability)
}

/// Shared by template IDs, runtime content and Unreal codegen.
pub fn validate_ability_effects(ability: &AbilityDefinition) -> Result<(), String> {
    if ability.tombstone || ability.effects.is_empty() {return Ok(());}
    if ability.effects.len()>32 || !matches!(ability.ability_type.as_str(),"active"|"ultimate")
        || ability.cast_type!="instant" {return Err(format!("ability '{}': effects require bounded instant active/ultimate",ability.id));}
    if ability.effects.iter().any(|effect|matches!(effect,AbilityEffect::DashToPoint)) {
        if ability.effects.len()!=1 || ability.target_type!="point" {
            return Err(format!("ability '{}': dash_to_point requires one exclusive point effect",ability.id));
        }
        if ability.levels.len()!=usize::from(ability.max_level) || ability.levels.iter()
            .any(|l|!l.range.is_finite() || l.range<MIN_ABILITY_SCALAR || l.range>10_000.0) {
            return Err(format!("ability '{}': dash_to_point requires positive bounded per-rank range",ability.id));
        }
        return Ok(());
    }
    let values=|key:&str,radius:bool| -> Result<(),String> {
        let entries=ability.extras.get(key).ok_or_else(||format!("ability '{}': missing effect extras '{key}'",ability.id))?;
        if key.is_empty() || entries.len()!=usize::from(ability.max_level)
            || entries.iter().any(|v|!valid_ability_scalar(*v) || (radius && (*v<1.0/1024.0 || *v>10_000.0))) {
            return Err(format!("ability '{}': invalid per-rank effect extras '{key}'",ability.id));
        }
        Ok(())
    };
    for effect in &ability.effects {
        let (amount,target)=match effect {
            AbilityEffect::DashToPoint=>unreachable!("exclusive movement checked above"),
            AbilityEffect::Damage {amount_key,..}=>(amount_key,"unit"),
            AbilityEffect::SlowEnemy {reduction_key,duration_key}=>{
                values(reduction_key,false)?;values(duration_key,false)?;
                if ability.extras[reduction_key].iter().any(|v|*v<MIN_ABILITY_SCALAR || *v>1.0)
                    || ability.extras[duration_key].iter().any(|v|*v<MIN_ABILITY_SCALAR || *v>60.0) {
                    return Err(format!("ability '{}': slow_enemy requires reduction in [1/1024,1] and duration in [1/1024,60]",ability.id));
                }
                (reduction_key,"unit")
            },
            AbilityEffect::HealSelf {amount_key}=>(amount_key,"none"),
            AbilityEffect::AreaDamage {amount_key,radius_key,..}=>{
                values(radius_key,true)?;
                (amount_key,"point")
            }
        };
        if ability.target_type!=target {return Err(format!("ability '{}': effect does not match target_type",ability.id));}
        if target!="none" && ability.levels.iter().any(|l|!l.range.is_finite() || l.range<1.0/1024.0 || l.range>10_000.0) {
            return Err(format!("ability '{}': targeted effects require positive bounded cast range",ability.id));
        }
        values(amount,false)?;
    }
    Ok(())
}

fn default_ability_max_level() -> u8 {
    4
}

/// Shared hero identity and gameplay values. Keep authoritative whole-number
/// attributes as integers; Unreal converts them only at its rendering boundary.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HeroDefinition {
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub portrait: String,
    #[serde(default)]
    pub background: String,
    #[serde(default)]
    pub tombstone: bool,
    #[serde(default)]
    pub abilities: Vec<String>,
    /// MOBA-only birth loadout; legacy Story and TD initialization is unchanged.
    #[serde(default)]
    pub moba_loadout: MobaLoadout,
    #[serde(default)]
    pub strength: i32,
    #[serde(default)]
    pub agility: i32,
    #[serde(default)]
    pub intelligence: i32,
    #[serde(default)]
    pub primary_attribute: String,
    #[serde(default)]
    pub attack_range: f32,
    #[serde(default)]
    pub base_damage: i32,
    #[serde(default)]
    pub base_armor: f32,
    #[serde(default)]
    pub base_hp: i32,
    #[serde(default)]
    pub base_mana: i32,
    #[serde(default)]
    pub move_speed: f32,
    #[serde(default)]
    pub turn_speed: f32,
    #[serde(default)]
    pub render: Option<HeroRender>,
    #[serde(default)]
    pub level_growth: HeroLevelGrowth,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct MobaLoadout {
    pub ranks: [u8; 4],
    pub skill_points: u8,
}

impl Default for MobaLoadout {
    fn default() -> Self { Self { ranks: [1; 4], skill_points: 0 } }
}

/// Both Rust and Unreal generators validate the same MOBA-only birth rules.
pub fn validate_moba_loadout<'a>(hero: &HeroDefinition, abilities: impl IntoIterator<Item = &'a AbilityDefinition>) -> Result<(), String> {
    if hero.tombstone { return Ok(()); }
    if hero.abilities.len() > 4 { return Err(format!("hero '{}': MOBA loadout exceeds four slots",hero.id)); }
    let definitions: std::collections::BTreeMap<_,_> = abilities.into_iter().map(|a| (a.id.as_str(),a)).collect();
    for (slot,id) in hero.abilities.iter().enumerate() {
        let rank = hero.moba_loadout.ranks[slot];
        let ability = definitions.get(id.as_str()).filter(|a| !a.tombstone)
            .ok_or_else(||format!("hero '{}': unknown MOBA ability '{id}'",hero.id))?;
        validate_ability_progression(ability)?;
        if rank > ability.max_level || (rank > 0 && ability.levels[usize::from(rank-1)].required_hero_level > 1) {
            return Err(format!("hero '{}': invalid MOBA birth rank {rank} for '{id}'",hero.id));
        }
    }
    if hero.moba_loadout.ranks[hero.abilities.len()..].iter().any(|rank| *rank > 1) {
        return Err(format!("hero '{}': MOBA loadout assigns rank to an empty slot",hero.id));
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct HeroLevelGrowth {
    #[serde(default)]
    pub strength_per_level: f32,
    #[serde(default)]
    pub agility_per_level: f32,
    #[serde(default)]
    pub intelligence_per_level: f32,
    #[serde(default)]
    pub damage_per_level: f32,
    #[serde(default)]
    pub hp_per_level: f32,
    #[serde(default)]
    pub mana_per_level: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct HeroRender {
    #[serde(default)]
    pub render_mode: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub texture: String,
    #[serde(default)]
    pub scale: f32,
    #[serde(default)]
    pub pitch_offset_deg: f32,
    #[serde(default)]
    pub roll_offset_deg: f32,
    #[serde(default)]
    pub yaw_offset_deg: f32,
    #[serde(default)]
    pub z_offset: f32,
    #[serde(default)]
    pub muzzle_bone: String,
    #[serde(default)]
    pub animation_sources: std::collections::BTreeMap<String, HeroAnimationSource>,
    #[serde(default)]
    pub animations: std::collections::BTreeMap<String, HeroAnimationBinding>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct HeroAnimationSource {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub animation: String,
    #[serde(default)]
    pub duration_ticks: f32,
    #[serde(default)]
    pub ticks_per_second: f32,
    #[serde(default)]
    pub timeline_offset_ticks: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct HeroAnimationBinding {
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub start_tick: f32,
    #[serde(default)]
    pub repeat_start_tick: f32,
    #[serde(default)]
    pub impact_tick: Option<f32>,
    #[serde(default)]
    pub end_tick: f32,
    #[serde(default, rename = "loop")]
    pub loop_animation: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ContentId {
    pub id: String,
    pub tombstone: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HeroId {
    pub id: String,
    pub tombstone: bool,
    pub abilities: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ContentCatalog {
    pub heroes: Vec<HeroId>,
    pub abilities: Vec<ContentId>,
}

impl ContentCatalog {
    pub fn validate(&self) -> Result<(), String> {
        let mut hero_ids = BTreeSet::new();
        for hero in &self.heroes {
            validate_id("hero", &hero.id)?;
            if !hero_ids.insert(hero.id.as_str()) {
                return Err(format!("duplicate hero id '{}'", hero.id));
            }
        }

        let mut ability_ids = BTreeSet::new();
        let mut active_abilities = BTreeSet::new();
        for ability in &self.abilities {
            validate_id("ability", &ability.id)?;
            if !ability_ids.insert(ability.id.as_str()) {
                return Err(format!("duplicate ability id '{}'", ability.id));
            }
            if !ability.tombstone {
                active_abilities.insert(ability.id.as_str());
            }
        }

        for hero in self.heroes.iter().filter(|hero| !hero.tombstone) {
            let mut seen = BTreeSet::new();
            for ability in &hero.abilities {
                if !active_abilities.contains(ability.as_str()) {
                    return Err(format!(
                        "hero '{}' references missing or tombstoned ability '{}'",
                        hero.id, ability
                    ));
                }
                if !seen.insert(ability.as_str()) {
                    return Err(format!(
                        "hero '{}' references ability '{}' more than once",
                        hero.id, ability
                    ));
                }
            }
        }
        Ok(())
    }

    /// Stable across platforms. Declaration order matters because numeric IDs use it.
    pub fn identity_hash(&self) -> Result<String, String> {
        self.validate()?;
        let mut hash = 0xcbf29ce484222325_u64;
        hash_field(&mut hash, "content-catalog-v1");
        for hero in &self.heroes {
            hash_field(&mut hash, "hero");
            hash_field(&mut hash, &hero.id);
            hash_field(&mut hash, if hero.tombstone { "1" } else { "0" });
            for ability in &hero.abilities {
                hash_field(&mut hash, ability);
            }
            hash_field(&mut hash, "end-hero");
        }
        for ability in &self.abilities {
            hash_field(&mut hash, "ability");
            hash_field(&mut hash, &ability.id);
            hash_field(&mut hash, if ability.tombstone { "1" } else { "0" });
        }
        Ok(format!("{hash:016x}"))
    }
}

fn validate_id(kind: &str, id: &str) -> Result<(), String> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(format!(
            "invalid {kind} id '{id}': use lowercase ASCII, digits, and '_'"
        ));
    }
    Ok(())
}

fn hash_field(hash: &mut u64, field: &str) {
    for byte in (field.len() as u64)
        .to_le_bytes()
        .into_iter()
        .chain(field.bytes())
    {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

/// Hash the complete evaluated `templates.lua` value, including gameplay and
/// presentation fields. `serde_json::Value` sorts object keys deterministically.
pub fn canonical_template_hash(value: &serde_json::Value) -> Result<String, String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| format!("serialize template catalog: {error}"))?;
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in b"omoba-template-v1".iter().chain(bytes.iter()) {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    Ok(format!("{hash:016x}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> ContentCatalog {
        ContentCatalog {
            heroes: vec![HeroId {
                id: "first_hero".into(),
                tombstone: false,
                abilities: vec!["first_skill".into()],
            }],
            abilities: vec![ContentId {
                id: "first_skill".into(),
                tombstone: false,
            }],
        }
    }

    #[test]
    fn rejects_missing_or_tombstoned_ability() {
        let mut c = catalog();
        c.abilities[0].tombstone = true;
        assert!(c.validate().unwrap_err().contains("first_hero"));
        c.abilities[0].tombstone = false;
        c.heroes[0].abilities[0] = "missing".into();
        assert!(c.validate().unwrap_err().contains("missing"));
    }

    #[test]
    fn rejects_duplicate_ids_and_hero_slots() {
        let mut c = catalog();
        c.abilities.push(c.abilities[0].clone());
        assert!(c.validate().unwrap_err().contains("duplicate ability"));
        c.abilities.pop();
        c.heroes[0].abilities.push("first_skill".into());
        assert!(c.validate().unwrap_err().contains("more than once"));
    }

    #[test]
    fn moba_loadout_allows_late_first_learning_but_not_prelearned_locked_rank() {
        let mut hero: HeroDefinition = serde_json::from_value(serde_json::json!({"id":"mage","abilities":["ultimate"],
            "moba_loadout":{"ranks":[0,0,0,0],"skill_points":1}})).unwrap();
        let ability: AbilityDefinition = serde_json::from_value(serde_json::json!({"id":"ultimate","max_level":1,
            "levels":[{"required_hero_level":6}]})).unwrap();
        validate_moba_loadout(&hero,[&ability]).unwrap();
        hero.moba_loadout.ranks[0]=1;
        assert!(validate_moba_loadout(&hero,[&ability]).is_err());
        hero.moba_loadout.ranks[0]=2;
        assert!(validate_moba_loadout(&hero,[&ability]).is_err());
        assert!(serde_json::from_value::<MobaLoadout>(serde_json::json!({"rank":[0,0,0,0]})).is_err());
    }

    #[test]
    fn moba_birth_loadout_is_explicit_and_legacy_defaults_are_preserved() {
        let legacy: HeroDefinition = serde_json::from_value(serde_json::json!({"id":"mage"})).unwrap();
        assert_eq!(legacy.moba_loadout, MobaLoadout::default());
        let unlearned: HeroDefinition = serde_json::from_value(serde_json::json!({"id":"mage",
            "moba_loadout":{"ranks":[0,0,0,0],"skill_points":1}})).unwrap();
        assert_eq!(unlearned.moba_loadout.ranks, [0;4]);
        assert_eq!(unlearned.moba_loadout.skill_points, 1);
        for ranks in [serde_json::json!([0,0,0]),serde_json::json!([0,0,0,256]),serde_json::json!([0,0,0,-1])] {
            assert!(serde_json::from_value::<MobaLoadout>(serde_json::json!({"ranks":ranks})).is_err());
        }
    }

    #[test]
    fn declaration_order_affects_identity_hash() {
        let mut c = catalog();
        let original = c.identity_hash().unwrap();
        c.abilities.push(ContentId {
            id: "second_skill".into(),
            tombstone: false,
        });
        assert_ne!(original, c.identity_hash().unwrap());
        c.abilities.swap(0, 1);
        assert_ne!(original, c.identity_hash().unwrap());
    }

    #[test]
    fn template_hash_is_stable_for_object_key_order_but_changes_with_values() {
        let a = serde_json::json!({"hero": {"hp": 500, "name": "Mage"}, "abilities": ["spark"]});
        let b = serde_json::json!({"abilities": ["spark"], "hero": {"name": "Mage", "hp": 500}});
        let c = serde_json::json!({"abilities": ["spark"], "hero": {"name": "Mage", "hp": 501}});
        assert_eq!(canonical_template_hash(&a), canonical_template_hash(&b));
        assert_ne!(canonical_template_hash(&a), canonical_template_hash(&c));
    }

    #[test]
    fn shared_ability_schema_preserves_levels_extras_and_default_max_level() {
        let input = serde_json::json!({
            "id": "arc_bolt",
            "display_name": "Arc Bolt",
            "ability_type": "active",
            "cast_type": "instant",
            "target_type": "unit",
            "levels": [{"cooldown": 8.0, "mana_cost": 25.0, "range": 600.0}],
            "extras": {"damage": [80.0, 120.0]}
        });
        let ability: AbilityDefinition = serde_json::from_value(input).unwrap();
        assert_eq!(ability.max_level, 4);
        assert_eq!(ability.levels[0].range, 600.0);
        assert_eq!(ability.levels[0].required_hero_level, 1);
        assert_eq!(ability.extras["damage"], vec![80.0, 120.0]);
        let roundtrip = serde_json::to_value(&ability).unwrap();
        assert_eq!(roundtrip["target_type"], "unit");
        assert_eq!(roundtrip["levels"][0]["mana_cost"], 25.0);
    }

    #[test]
    fn ability_progression_requires_bounded_monotonic_complete_ranks() {
        let make = |requirements: &[u8]| -> AbilityDefinition {
            serde_json::from_value(serde_json::json!({"id":"progression", "max_level":requirements.len(),
                "levels":requirements.iter().map(|v| serde_json::json!({"required_hero_level":v})).collect::<Vec<_>>() })).unwrap()
        };
        assert!(validate_ability_progression(&make(&[1,6,11,16])).is_ok());
        for values in [vec![],vec![0],vec![26],vec![6,5]] { assert!(validate_ability_progression(&make(&values)).is_err()); }
        let mut incomplete = make(&[1,2]); incomplete.levels.pop();
        assert!(validate_ability_progression(&incomplete).is_err());
    }

    #[test]
    fn authored_numeric_envelope_checks_every_rank_and_field() {
        let valid: AbilityDefinition = serde_json::from_value(serde_json::json!({
            "id":"numeric", "max_level":2, "levels":[{},{}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        for field in ["cooldown", "mana_cost", "cast_time", "range"] {
            for value in [-1.0, 0.0001, 1_000_001.0, f32::INFINITY, f32::NEG_INFINITY, f32::NAN] {
                let mut bad=valid.clone();
                let level=&mut bad.levels[1];
                match field {"cooldown"=>level.cooldown=value,"mana_cost"=>level.mana_cost=value,
                    "cast_time"=>level.cast_time=value,_=>level.range=value}
                let error=validate_ability_progression(&bad).unwrap_err();
                assert!(error.contains("numeric") && error.contains("rank 2") && error.contains(field),"{error}");
            }
        }
        let mut boundaries=valid.clone();
        boundaries.levels[0].cooldown=MIN_ABILITY_SCALAR;
        boundaries.levels[1].mana_cost=MAX_ABILITY_SCALAR;
        assert!(validate_ability_progression(&boundaries).is_ok());
        let rounded: AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"rounded", "max_level":1,
            "levels":[{"cooldown":0.00097656249,"mana_cost":1000000.01,"range":10000.0001}]
        })).unwrap();
        assert!(validate_ability_progression(&rounded).is_ok());
        // Deleted content does not need valid runtime data.
        boundaries.tombstone=true;boundaries.levels[1].range=f32::NAN;
        assert!(validate_ability_progression(&boundaries).is_ok());
    }

    #[test]
    fn dash_effect_requires_exclusive_point_and_all_rank_ranges() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"dash", "max_level":2,"ability_type":"active","cast_type":"instant",
            "target_type":"point","levels":[{"range":450},{"range":600}],
            "effects":[{"kind":"dash_to_point"}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        for range in [0.0,0.0001,10_001.0,f32::INFINITY,f32::NAN] {
            let mut bad=valid.clone();bad.levels[1].range=range;
            assert!(validate_ability_progression(&bad).is_err());
        }
        let mut bad=valid.clone();bad.target_type="none".into();
        assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.effects.push(AbilityEffect::DashToPoint);
        assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid;bad.effects.push(AbilityEffect::AreaDamage {
            amount_key:"damage".into(),radius_key:"radius".into(),damage_kind:EffectDamageKind::Physical});
        assert!(validate_ability_progression(&bad).unwrap_err().contains("exclusive"));
    }

    #[test]
    fn slow_effect_validates_rank_reduction_duration_range_and_target() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"slow", "max_level":2,"ability_type":"active","cast_type":"instant",
            "target_type":"unit","levels":[{"range":450},{"range":600}],
            "extras":{"reduction":[0.25,0.5],"duration":[2,3]},
            "effects":[{"kind":"slow_enemy","reduction_key":"reduction","duration_key":"duration"}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        for (key,values) in [("reduction",vec![0.0,1.01,f32::NAN]),("duration",vec![0.0,60.1,f32::INFINITY])] {
            for value in values {
                let mut bad=valid.clone();bad.extras.get_mut(key).unwrap()[1]=value;
                assert!(validate_ability_progression(&bad).is_err());
            }
        }
        let mut bad=valid.clone();bad.levels[1].range=0.0;
        assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.extras.remove("duration");
        assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid;bad.target_type="point".into();
        assert!(validate_ability_progression(&bad).is_err());
    }

    #[test]
    fn authored_numeric_envelope_bounds_only_executable_effect_extras() {
        let mut ability: AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"numeric_heal","max_level":1,"ability_type":"active","cast_type":"instant",
            "target_type":"none","levels":[{}],"extras":{"heal":[0],"unused":[-5]},
            "effects":[{"kind":"heal_self","amount_key":"heal"}]
        })).unwrap();
        assert!(validate_ability_progression(&ability).is_ok());
        for value in [MIN_ABILITY_SCALAR,MAX_ABILITY_SCALAR] {
            ability.extras.get_mut("heal").unwrap()[0]=value;
            assert!(validate_ability_progression(&ability).is_ok());
        }
        for value in [0.0001,1_000_001.0,f32::INFINITY,f32::NAN] {
            ability.extras.get_mut("heal").unwrap()[0]=value;
            assert!(validate_ability_progression(&ability).unwrap_err().contains("heal"));
        }
    }

    #[test]
    fn area_effects_validate_shared_target_rank_data_and_limits() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"area_fixture","ability_type":"active","cast_type":"instant","target_type":"point","max_level":1,
            "levels":[{"range":700}],"extras":{"damage":[110],"radius":[220]},
            "effects":[{"kind":"area_damage","amount_key":"damage","radius_key":"radius","damage_kind":"physical"}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        for radius in [0.0,-1.0,0.0001,10_001.0,f32::NAN] {
            let mut bad=valid.clone();bad.extras.insert("radius".into(),vec![radius]);
            assert!(validate_ability_progression(&bad).is_err());
        }
        let mut bad=valid.clone();bad.target_type="unit".into();assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.extras.remove("damage");assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.levels[0].range=0.0;assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.effects=vec![bad.effects[0].clone();33];assert!(validate_ability_progression(&bad).is_err());
    }

    #[test]
    fn cast_preflight_unit_effects_require_representable_per_rank_range() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"range_fixture","ability_type":"active","cast_type":"instant","target_type":"unit","max_level":2,
            "levels":[{"range":600},{"range":700}],"extras":{"damage":[80,125]},
            "effects":[{"kind":"damage","amount_key":"damage","damage_kind":"magical"}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        for range in [0.0,-1.0,0.0001,10_001.0,f32::INFINITY,f32::NAN] {
            let mut bad=valid.clone();bad.levels[1].range=range;
            assert!(validate_ability_progression(&bad).is_err());
        }
        let mut heal=valid;heal.target_type="none".into();
        heal.effects=vec![AbilityEffect::HealSelf {amount_key:"damage".into()}];
        for rank in &mut heal.levels {rank.range=0.0;}
        assert!(validate_ability_progression(&heal).is_ok());
    }

    #[test]
    fn shared_hero_schema_parses_authoritative_stats_and_animation_bindings() {
        let hero: HeroDefinition = serde_json::from_value(serde_json::json!({
            "id": "arc_mage", "strength": 17, "base_hp": 560,
            "base_damage": 42, "abilities": ["arc_bolt"]
        })).unwrap();
        assert_eq!(hero.base_hp, 560);
        assert_eq!(hero.abilities, ["arc_bolt"]);
        assert!(serde_json::from_value::<HeroDefinition>(serde_json::json!({
            "id": "arc_mage", "base_hp": 560.5
        })).is_err());

        let render: HeroRender = serde_json::from_value(serde_json::json!({
            "render_mode": "model_3d",
            "animation_sources": {"idle": {"model": "idle.fbx", "animation": "Take 001"}},
            "animations": {"idle": {"source": "idle", "loop": true}}
        })).unwrap();
        assert_eq!(render.animation_sources["idle"].model, "idle.fbx");
        assert!(render.animations["idle"].loop_animation);
    }
}
