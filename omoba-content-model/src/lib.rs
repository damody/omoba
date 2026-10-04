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

/// Executable subset of Lua-authored effects. `amount_key` resolves a
/// per-level value from `extras`; special skills still use a Rust handler.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AbilityEffect {
    Damage {
        amount_key: String,
        damage_kind: EffectDamageKind,
    },
    HealSelf {
        amount_key: String,
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
        if !(1..=25).contains(&level.required_hero_level) || level.required_hero_level < previous {
            return Err(format!("ability '{}': rank {} required_hero_level must be monotonic in 1..=25", ability.id, rank + 1));
        }
        previous = level.required_hero_level;
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
