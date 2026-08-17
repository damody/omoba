use crate::lua_content::{
    load_content, validate_active_ability_quantization, AbilityEntry, AttackTimingEntry,
    CreepEntry, HeroEntry, HeroRenderEntry, Manifest, StoryBundle, SummonEntry, TdLayerEntry,
    TowerEntry, UpgradeEffectEntry,
};
use crate::{
    ability_by_name, ability_id_str, creep_id_str, hero_id_str, summon_id_str, tower_id_str,
    AbilityConst, AbilityId, AbilityLevelDataConst, AbilityTypeC, ActiveAbilityConst,
    AttackTimingConst, CastTypeC, CreepId, CreepStats, Fixed64, GeneratedStory,
    HeroAnimationBindingConst, HeroAnimationSourceConst, HeroId, HeroRenderMetadataConst,
    HeroRenderModeC, HeroStats, LevelGrowth, StatOpC, StoryValue, SummonId, SummonStats,
    TargetTypeC, TdLayerMetadataConst, TowerBarrelLayoutC, TowerBarrelVariantConst, TowerId,
    TowerRecoilConst, TowerRecoilModeC, TowerRenderAnimationConst, TowerRenderMetadataConst,
    TowerRenderModeC, TowerRenderPointConst, TowerRotationModeC, TowerStats, UpgradeDefConst,
    UpgradeEffectConst, UpgradeEffectKindC,
};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

static CONTENT: OnceLock<RuntimeContentStore> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeContentInfo {
    pub generation: u64,
    pub hash: String,
    pub root: PathBuf,
}

struct RuntimeContentStore {
    state: RwLock<RuntimeContentState>,
}

enum RuntimeContentState {
    Uninitialized,
    Disabled,
    Loaded(RuntimeSnapshot),
    Failed(String),
}

#[derive(Clone)]
struct RuntimeSnapshot {
    content: &'static RuntimeContent,
    shape: ContentShape,
    info: RuntimeContentInfo,
}

pub struct RuntimeContent {
    tower_stats: Vec<Option<&'static TowerStats>>,
    tower_display: Vec<Option<&'static str>>,
    tower_render: Vec<Option<&'static TowerRenderMetadataConst>>,
    tower_attack_timing: Vec<Option<AttackTimingConst>>,
    tower_upgrades: Vec<Option<&'static [&'static [UpgradeDefConst]]>>,
    hero_stats: Vec<Option<&'static HeroStats>>,
    hero_display: Vec<Option<&'static str>>,
    hero_title: Vec<Option<&'static str>>,
    hero_portrait: Vec<Option<&'static str>>,
    hero_render: Vec<Option<&'static HeroRenderMetadataConst>>,
    hero_abilities: Vec<Option<&'static [AbilityId]>>,
    hero_attack_timing: Vec<Option<AttackTimingConst>>,
    creep_stats: Vec<Option<&'static CreepStats>>,
    creep_display: Vec<Option<&'static str>>,
    creep_attack_timing: Vec<Option<AttackTimingConst>>,
    summon_stats: Vec<Option<&'static SummonStats>>,
    summon_display: Vec<Option<&'static str>>,
    summon_attack_timing: Vec<Option<AttackTimingConst>>,
    ability_const: Vec<Option<&'static AbilityConst>>,
    ability_display: Vec<Option<&'static str>>,
    ability_description: Vec<Option<&'static str>>,
    td_layer_catalog: &'static [TdLayerMetadataConst],
    td_layers: HashMap<&'static str, &'static TdLayerMetadataConst>,
    td_layer_digest: u64,
    story_ids: &'static [&'static str],
    stories: HashMap<&'static str, &'static GeneratedStory>,
}

pub fn ensure_loaded() -> Result<Option<&'static RuntimeContent>, String> {
    CONTENT
        .get_or_init(RuntimeContentStore::new)
        .ensure_loaded()
}

fn active_content() -> Option<&'static RuntimeContent> {
    match ensure_loaded() {
        Ok(content) => content,
        Err(err) => panic!("runtime Lua content load failed: {}", err),
    }
}

pub fn reload_runtime_lua_content_dev(
    expected_hash: Option<&str>,
) -> Result<Option<RuntimeContentInfo>, String> {
    CONTENT
        .get_or_init(RuntimeContentStore::new)
        .reload_dev(expected_hash)
}

pub fn validate_runtime_lua_content_dev() -> Result<Option<RuntimeContentInfo>, String> {
    CONTENT.get_or_init(RuntimeContentStore::new).validate_dev()
}

pub fn runtime_lua_content_info() -> Result<Option<RuntimeContentInfo>, String> {
    CONTENT.get_or_init(RuntimeContentStore::new).info()
}

pub fn runtime_lua_content_generation() -> Result<Option<u64>, String> {
    runtime_lua_content_info().map(|info| info.map(|info| info.generation))
}

pub fn runtime_lua_content_hash() -> Result<Option<String>, String> {
    runtime_lua_content_info().map(|info| info.map(|info| info.hash))
}

pub fn lua_hot_reload_enabled() -> bool {
    env_truthy("OMB_LUA_CONTENT") && env_truthy("OMB_LUA_HOT_RELOAD")
}

impl RuntimeContentStore {
    fn new() -> Self {
        Self {
            state: RwLock::new(RuntimeContentState::Uninitialized),
        }
    }

    fn ensure_loaded(&self) -> Result<Option<&'static RuntimeContent>, String> {
        {
            let state = self.state.read().map_err(lock_err)?;
            if !matches!(*state, RuntimeContentState::Uninitialized) {
                return content_from_state(&state);
            }
        }

        let mut state = self.state.write().map_err(lock_err)?;
        if matches!(*state, RuntimeContentState::Uninitialized) {
            *state = load_initial_state();
        }
        content_from_state(&state)
    }

    fn info(&self) -> Result<Option<RuntimeContentInfo>, String> {
        let _ = self.ensure_loaded()?;
        let state = self.state.read().map_err(lock_err)?;
        match &*state {
            RuntimeContentState::Loaded(snapshot) => Ok(Some(snapshot.info.clone())),
            RuntimeContentState::Disabled => Ok(None),
            RuntimeContentState::Failed(err) => Err(err.clone()),
            RuntimeContentState::Uninitialized => Ok(None),
        }
    }

    fn reload_dev(
        &self,
        expected_hash: Option<&str>,
    ) -> Result<Option<RuntimeContentInfo>, String> {
        let _ = self.ensure_loaded()?;

        let mut state = self.state.write().map_err(lock_err)?;
        let RuntimeContentState::Loaded(current) = &*state else {
            return match &*state {
                RuntimeContentState::Disabled => Ok(None),
                RuntimeContentState::Failed(err) => Err(err.clone()),
                RuntimeContentState::Uninitialized => Ok(None),
                RuntimeContentState::Loaded(_) => unreachable!(),
            };
        };

        let root = content_root();
        let next_generation = current.info.generation.saturating_add(1);
        let next = load_runtime_snapshot(root, next_generation)?;
        next.shape.ensure_compatible_with(&current.shape)?;
        if let Some(expected_hash) = expected_hash {
            if next.info.hash != expected_hash {
                return Err(format!(
                    "runtime Lua content hash mismatch: expected {}, got {}",
                    expected_hash, next.info.hash
                ));
            }
        }

        let info = next.info.clone();
        *state = RuntimeContentState::Loaded(next);
        Ok(Some(info))
    }

    fn validate_dev(&self) -> Result<Option<RuntimeContentInfo>, String> {
        let _ = self.ensure_loaded()?;

        let state = self.state.read().map_err(lock_err)?;
        let RuntimeContentState::Loaded(current) = &*state else {
            return match &*state {
                RuntimeContentState::Disabled => Ok(None),
                RuntimeContentState::Failed(err) => Err(err.clone()),
                RuntimeContentState::Uninitialized => Ok(None),
                RuntimeContentState::Loaded(_) => unreachable!(),
            };
        };

        let root = content_root();
        let next_generation = current.info.generation.saturating_add(1);
        let next = load_runtime_snapshot(root, next_generation)?;
        next.shape.ensure_compatible_with(&current.shape)?;
        Ok(Some(next.info))
    }

    #[cfg(test)]
    fn reset_for_tests(&self) {
        *self.state.write().unwrap() = RuntimeContentState::Uninitialized;
    }
}

fn content_from_state(
    state: &RuntimeContentState,
) -> Result<Option<&'static RuntimeContent>, String> {
    match state {
        RuntimeContentState::Loaded(snapshot) => Ok(Some(snapshot.content)),
        RuntimeContentState::Disabled => Ok(None),
        RuntimeContentState::Failed(err) => Err(err.clone()),
        RuntimeContentState::Uninitialized => Ok(None),
    }
}

fn load_initial_state() -> RuntimeContentState {
    if !env_truthy("OMB_LUA_CONTENT") {
        return RuntimeContentState::Disabled;
    }

    let root = content_root();
    match load_runtime_snapshot(root, 1) {
        Ok(snapshot) => RuntimeContentState::Loaded(snapshot),
        Err(err) => RuntimeContentState::Failed(err),
    }
}

fn load_runtime_snapshot(root: PathBuf, generation: u64) -> Result<RuntimeSnapshot, String> {
    let content = load_content(root.clone())?;
    let hash = content_hash(&content.manifest, &content.stories)?;
    let shape = ContentShape::from_manifest(&content.manifest, &content.stories);
    let runtime = RuntimeContent::from_manifest(content.manifest, content.stories)
        .map_err(|err| format!("{} (root={})", err, root.display()))?;
    Ok(RuntimeSnapshot {
        content: leak(runtime),
        shape,
        info: RuntimeContentInfo {
            generation,
            hash,
            root,
        },
    })
}

fn content_hash(manifest: &Manifest, stories: &[StoryBundle]) -> Result<String, String> {
    let bytes = serde_json::to_vec(&(manifest, stories))
        .map_err(|err| format!("serialize runtime Lua content for hash: {}", err))?;
    Ok(format!("{:016x}", fnv1a64(&bytes)))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn lock_err<T>(err: std::sync::PoisonError<T>) -> String {
    format!("runtime Lua content store lock poisoned: {}", err)
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn content_root() -> PathBuf {
    std::env::var("OMB_LUA_CONTENT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("scripts/lua_data")
        })
}

impl RuntimeContent {
    fn from_manifest(manifest: Manifest, stories: Vec<StoryBundle>) -> Result<Self, String> {
        validate_active_abilities(&manifest.towers)?;
        let tower_stats = build_indexed(&manifest.towers, "tower", |raw, entry| {
            ensure_id("tower", raw, &entry.id, tower_id_str(TowerId(raw)))?;
            Ok(Some(leak(TowerStats {
                atk: fixed64(entry.atk),
                asd_interval: fixed64(entry.asd_interval),
                range: fixed64(entry.range),
                bullet_speed: fixed64(entry.bullet_speed),
                splash_radius: fixed64(entry.splash_radius),
                hit_radius: fixed64(entry.hit_radius),
                slow_factor: fixed64(entry.slow_factor),
                slow_duration: fixed64(entry.slow_duration),
                cost: entry.cost,
                footprint: fixed64(entry.footprint),
                placement_radius: fixed64(entry.placement_radius),
                hp: fixed64(entry.hp),
                turn_speed_deg: fixed64(entry.turn_speed_deg),
            })))
        })?;
        let tower_display = build_indexed(&manifest.towers, "tower", |raw, entry| {
            ensure_id("tower", raw, &entry.id, tower_id_str(TowerId(raw)))?;
            Ok(Some(leak_str(display_name(&entry.display_name, &entry.id))))
        })?;
        let tower_render = build_indexed(&manifest.towers, "tower", |raw, entry| {
            ensure_id("tower", raw, &entry.id, tower_id_str(TowerId(raw)))?;
            Ok(Some(leak(build_tower_render_metadata(entry)?)))
        })?;
        let tower_attack_timing = build_indexed(&manifest.towers, "tower", |raw, entry| {
            ensure_id("tower", raw, &entry.id, tower_id_str(TowerId(raw)))?;
            Ok(Some(attack_timing(entry.attack_timing)?))
        })?;
        let tower_upgrades = build_indexed(&manifest.towers, "tower", |raw, entry| {
            ensure_id("tower", raw, &entry.id, tower_id_str(TowerId(raw)))?;
            if entry.upgrades.is_empty() {
                return Ok(None);
            }
            Ok(Some(build_tower_upgrades(entry)?))
        })?;

        let hero_stats = build_indexed(&manifest.heroes, "hero", |raw, entry| {
            ensure_id("hero", raw, &entry.id, hero_id_str(HeroId(raw)))?;
            Ok(Some(leak(build_hero_stats(entry)?)))
        })?;
        let hero_display = build_indexed(&manifest.heroes, "hero", |raw, entry| {
            ensure_id("hero", raw, &entry.id, hero_id_str(HeroId(raw)))?;
            Ok(Some(leak_str(display_name(&entry.display_name, &entry.id))))
        })?;
        let hero_title = build_indexed(&manifest.heroes, "hero", |raw, entry| {
            ensure_id("hero", raw, &entry.id, hero_id_str(HeroId(raw)))?;
            Ok(Some(leak_str(entry.title.clone())))
        })?;
        let hero_portrait = build_indexed(&manifest.heroes, "hero", |raw, entry| {
            ensure_id("hero", raw, &entry.id, hero_id_str(HeroId(raw)))?;
            Ok(Some(leak_str(entry.portrait.clone())))
        })?;
        let hero_render = build_indexed(&manifest.heroes, "hero", |raw, entry| {
            ensure_id("hero", raw, &entry.id, hero_id_str(HeroId(raw)))?;
            entry
                .render
                .as_ref()
                .filter(|render| !render.render_mode.trim().is_empty())
                .map(|render| build_hero_render_metadata(&entry.id, render).map(leak))
                .transpose()
        })?;
        let hero_abilities = build_indexed(&manifest.heroes, "hero", |raw, entry| {
            ensure_id("hero", raw, &entry.id, hero_id_str(HeroId(raw)))?;
            let mut abilities = Vec::new();
            for ability in &entry.abilities {
                let id = ability_by_name(ability).ok_or_else(|| {
                    format!(
                        "hero '{}' references unknown ability '{}'",
                        entry.id, ability
                    )
                })?;
                abilities.push(id);
            }
            Ok(Some(leak_slice(abilities)))
        })?;
        let hero_attack_timing = build_indexed(&manifest.heroes, "hero", |raw, entry| {
            ensure_id("hero", raw, &entry.id, hero_id_str(HeroId(raw)))?;
            Ok(Some(attack_timing(entry.attack_timing)?))
        })?;

        let creep_stats = build_indexed(&manifest.creeps, "creep", |raw, entry| {
            ensure_id("creep", raw, &entry.id, creep_id_str(CreepId(raw)))?;
            Ok(Some(leak(build_creep_stats(entry)?)))
        })?;
        let creep_display = build_indexed(&manifest.creeps, "creep", |raw, entry| {
            ensure_id("creep", raw, &entry.id, creep_id_str(CreepId(raw)))?;
            Ok(Some(leak_str(display_name(&entry.display_name, &entry.id))))
        })?;
        let creep_attack_timing = build_indexed(&manifest.creeps, "creep", |raw, entry| {
            ensure_id("creep", raw, &entry.id, creep_id_str(CreepId(raw)))?;
            Ok(Some(attack_timing(entry.attack_timing)?))
        })?;

        let summon_stats = build_indexed(&manifest.summons, "summon", |raw, entry| {
            ensure_id("summon", raw, &entry.id, summon_id_str(SummonId(raw)))?;
            Ok(Some(leak(SummonStats {
                hp: fixed64(entry.hp),
                damage: fixed64(entry.damage),
                duration: fixed64(entry.duration),
                move_speed: fixed64(entry.move_speed),
            })))
        })?;
        let summon_display = build_indexed(&manifest.summons, "summon", |raw, entry| {
            ensure_id("summon", raw, &entry.id, summon_id_str(SummonId(raw)))?;
            Ok(Some(leak_str(display_name(&entry.display_name, &entry.id))))
        })?;
        let summon_attack_timing = build_indexed(&manifest.summons, "summon", |raw, entry| {
            ensure_id("summon", raw, &entry.id, summon_id_str(SummonId(raw)))?;
            Ok(Some(attack_timing(entry.attack_timing)?))
        })?;

        let ability_const = build_indexed(&manifest.abilities, "ability", |raw, entry| {
            ensure_id("ability", raw, &entry.id, ability_id_str(AbilityId(raw)))?;
            Ok(Some(leak(ability_const_from_entry(entry)?)))
        })?;
        let ability_display = build_indexed(&manifest.abilities, "ability", |raw, entry| {
            ensure_id("ability", raw, &entry.id, ability_id_str(AbilityId(raw)))?;
            Ok(Some(leak_str(display_name(&entry.display_name, &entry.id))))
        })?;
        let ability_description = build_indexed(&manifest.abilities, "ability", |raw, entry| {
            ensure_id("ability", raw, &entry.id, ability_id_str(AbilityId(raw)))?;
            Ok(Some(leak_str(entry.description.clone())))
        })?;

        let td_layer_digest = fnv1a64(
            &serde_json::to_vec(&manifest.td_layers)
                .map_err(|error| format!("serialize runtime TD layer catalog: {error}"))?,
        );
        let td_layer_catalog = leak_slice(
            manifest
                .td_layers
                .iter()
                .map(build_td_layer)
                .collect::<Vec<_>>(),
        );
        let td_layers = td_layer_catalog
            .iter()
            .map(|entry| (entry.id, entry))
            .collect();

        let mut story_map = HashMap::new();
        let mut story_id_vec = Vec::new();
        for story in stories {
            let leaked_id = leak_str(story.id.clone());
            story_id_vec.push(leaked_id);
            let generated = leak(GeneratedStory {
                id: leaked_id,
                entity: story_value(story.entity),
                ability: story_value(story.ability),
                mission: story_value(story.mission),
                map: story_value(story.map),
            });
            story_map.insert(leaked_id, generated);
        }

        Ok(Self {
            tower_stats,
            tower_display,
            tower_render,
            tower_attack_timing,
            tower_upgrades,
            hero_stats,
            hero_display,
            hero_title,
            hero_portrait,
            hero_render,
            hero_abilities,
            hero_attack_timing,
            creep_stats,
            creep_display,
            creep_attack_timing,
            summon_stats,
            summon_display,
            summon_attack_timing,
            ability_const,
            ability_display,
            ability_description,
            td_layer_catalog,
            td_layers,
            td_layer_digest,
            story_ids: leak_slice(story_id_vec),
            stories: story_map,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ContentShape {
    towers: Vec<Option<String>>,
    heroes: Vec<Option<String>>,
    abilities: Vec<Option<String>>,
    buffs: Vec<Option<String>>,
    summons: Vec<Option<String>>,
    creeps: Vec<Option<String>>,
    projectile_kinds: Vec<Option<String>>,
    td_layers: Vec<String>,
    stories: Vec<StoryShape>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoryShape {
    id: String,
    entity: JsonShape,
    ability: JsonShape,
    mission: JsonShape,
    map: JsonShape,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum JsonShape {
    Null,
    Bool,
    Number,
    String,
    Array(Vec<JsonShape>),
    Object(Vec<(String, JsonShape)>),
}

impl ContentShape {
    fn from_manifest(manifest: &Manifest, stories: &[StoryBundle]) -> Self {
        Self {
            towers: entry_shape(&manifest.towers),
            heroes: entry_shape(&manifest.heroes),
            abilities: entry_shape(&manifest.abilities),
            buffs: entry_shape(&manifest.buffs),
            summons: entry_shape(&manifest.summons),
            creeps: entry_shape(&manifest.creeps),
            projectile_kinds: entry_shape(&manifest.projectile_kinds),
            td_layers: manifest
                .td_layers
                .iter()
                .map(|entry| entry.id.clone())
                .collect(),
            stories: stories
                .iter()
                .map(|story| StoryShape {
                    id: story.id.clone(),
                    entity: JsonShape::from_value(&story.entity),
                    ability: JsonShape::from_value(&story.ability),
                    mission: JsonShape::from_value(&story.mission),
                    map: JsonShape::from_value(&story.map),
                })
                .collect(),
        }
    }

    fn ensure_compatible_with(&self, previous: &Self) -> Result<(), String> {
        compare_shape("tower ids", &self.towers, &previous.towers)?;
        compare_shape("hero ids", &self.heroes, &previous.heroes)?;
        compare_shape("ability ids", &self.abilities, &previous.abilities)?;
        compare_shape("buff ids", &self.buffs, &previous.buffs)?;
        compare_shape("summon ids", &self.summons, &previous.summons)?;
        compare_shape("creep ids", &self.creeps, &previous.creeps)?;
        compare_shape(
            "projectile kind ids",
            &self.projectile_kinds,
            &previous.projectile_kinds,
        )?;
        if self.td_layers != previous.td_layers {
            return Err("runtime Lua content TD layer ids changed; restart gameplay".into());
        }
        if self.stories != previous.stories {
            return Err(
                "runtime Lua content story topology changed; restart gameplay to apply structural changes"
                    .into(),
            );
        }
        Ok(())
    }
}

fn build_td_layer(entry: &TdLayerEntry) -> TdLayerMetadataConst {
    TdLayerMetadataConst {
        id: leak_str(entry.id.clone()),
        label: leak_str(entry.label.clone()),
        hp: entry.hp,
        move_speed: entry.move_speed,
        children: leak_slice(entry.children.iter().cloned().map(leak_str).collect()),
        cash: entry.cash,
        leak_value: entry.leak_value,
        properties: entry.properties,
        accepted_damage: entry.accepted_damage,
        regrow_eligible: entry.regrow_eligible,
        fortified_eligible: entry.fortified_eligible,
    }
}

impl JsonShape {
    fn from_value(value: &JsonValue) -> Self {
        match value {
            JsonValue::Null => Self::Null,
            JsonValue::Bool(_) => Self::Bool,
            JsonValue::Number(_) => Self::Number,
            JsonValue::String(_) => Self::String,
            JsonValue::Array(values) => Self::Array(values.iter().map(Self::from_value).collect()),
            JsonValue::Object(values) => {
                let mut pairs: Vec<_> = values
                    .iter()
                    .map(|(key, value)| (key.clone(), Self::from_value(value)))
                    .collect();
                pairs.sort_by(|left, right| left.0.cmp(&right.0));
                Self::Object(pairs)
            }
        }
    }
}

fn entry_shape<T: RuntimeEntry>(entries: &[T]) -> Vec<Option<String>> {
    entries
        .iter()
        .map(|entry| {
            if entry.tombstone() {
                None
            } else {
                Some(entry.id().to_string())
            }
        })
        .collect()
}

fn compare_shape<T: PartialEq + std::fmt::Debug>(
    label: &str,
    next: &T,
    previous: &T,
) -> Result<(), String> {
    if next != previous {
        return Err(format!(
            "runtime Lua content {} changed; restart gameplay to apply structural changes",
            label
        ));
    }
    Ok(())
}

trait RuntimeEntry {
    fn id(&self) -> &str;
    fn tombstone(&self) -> bool;
}

impl RuntimeEntry for TowerEntry {
    fn id(&self) -> &str {
        &self.id
    }

    fn tombstone(&self) -> bool {
        self.tombstone
    }
}

impl RuntimeEntry for HeroEntry {
    fn id(&self) -> &str {
        &self.id
    }

    fn tombstone(&self) -> bool {
        self.tombstone
    }
}

impl RuntimeEntry for CreepEntry {
    fn id(&self) -> &str {
        &self.id
    }

    fn tombstone(&self) -> bool {
        self.tombstone
    }
}

impl RuntimeEntry for SummonEntry {
    fn id(&self) -> &str {
        &self.id
    }

    fn tombstone(&self) -> bool {
        self.tombstone
    }
}

impl RuntimeEntry for AbilityEntry {
    fn id(&self) -> &str {
        &self.id
    }

    fn tombstone(&self) -> bool {
        self.tombstone
    }
}

impl RuntimeEntry for crate::lua_content::Entry {
    fn id(&self) -> &str {
        &self.id
    }

    fn tombstone(&self) -> bool {
        self.tombstone
    }
}

impl RuntimeEntry for crate::lua_content::ProjKind {
    fn id(&self) -> &str {
        &self.id
    }

    fn tombstone(&self) -> bool {
        self.tombstone
    }
}

fn build_indexed<T, U, F>(
    entries: &[T],
    kind: &str,
    mut convert: F,
) -> Result<Vec<Option<U>>, String>
where
    T: RuntimeEntry,
    F: FnMut(u16, &T) -> Result<Option<U>, String>,
{
    let mut out = vec![None];
    let mut raw = 1u16;
    for entry in entries {
        if entry.tombstone() {
            out.push(None);
        } else {
            out.push(convert(raw, entry).map_err(|err| format!("{} id {}: {}", kind, raw, err))?);
        }
        raw = raw
            .checked_add(1)
            .ok_or_else(|| format!("{}Id exhausted u16 space", kind))?;
    }
    Ok(out)
}

fn ensure_id(kind: &str, raw: u16, expected: &str, actual: &str) -> Result<(), String> {
    if actual != expected {
        return Err(format!(
            "{} id order changed: generated raw {} is '{}', Lua content has '{}'; rebuild generated ids",
            kind, raw, actual, expected
        ));
    }
    Ok(())
}

fn display_name(display: &str, id: &str) -> String {
    if display.is_empty() {
        id.to_string()
    } else {
        display.to_string()
    }
}

fn fixed64(value: f32) -> Fixed64 {
    Fixed64::from_raw((value * 1024.0).round() as i64)
}

fn leak<T>(value: T) -> &'static T {
    Box::leak(Box::new(value))
}

fn leak_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn leak_slice<T>(values: Vec<T>) -> &'static [T] {
    Box::leak(values.into_boxed_slice())
}

fn attack_timing(entry: AttackTimingEntry) -> Result<AttackTimingConst, String> {
    let total = entry.windup as u32 + entry.backswing as u32;
    if total != 1000 {
        return Err(format!(
            "attack_timing must have windup + backswing == 1000, got {} + {} = {}",
            entry.windup, entry.backswing, total
        ));
    }
    Ok(AttackTimingConst {
        windup: entry.windup,
        backswing: entry.backswing,
    })
}

fn build_tower_render_metadata(entry: &TowerEntry) -> Result<TowerRenderMetadataConst, String> {
    let mut render = entry.render.clone();
    if render.render_mode.is_empty() {
        render.render_mode = "base_barrel".into();
    }
    if render.base.is_empty() {
        render.base = if render.render_mode == "animated_area" {
            format!("assets/towers/{}_frame_01.png", entry.id)
        } else {
            format!("assets/towers/{}_base.png", entry.id)
        };
    }
    if render.render_mode != "animated_area" && render.barrel.is_empty() {
        render.barrel = format!("assets/towers/{}_barrel.png", entry.id);
    }
    if render.visual_size <= 0.0 {
        return Err(format!(
            "tower '{}' render.visual_size must be > 0 and declared in scripts Lua metadata",
            entry.id
        ));
    }
    if render.rotation_mode.is_empty() {
        render.rotation_mode = "targeted".into();
    }
    if render.barrel_layout.is_empty() {
        render.barrel_layout = "single".into();
    }
    if render.barrel_pivot.x == 0.0 && render.barrel_pivot.y == 0.0 {
        render.barrel_pivot.x = 0.5;
        render.barrel_pivot.y = 0.65;
    }

    let body_frames = if render.render_mode == "animated_area" {
        render.animation.frames.clone()
    } else {
        Vec::new()
    };
    Ok(TowerRenderMetadataConst {
        render_mode: tower_render_mode(&render.render_mode)?,
        base: leak_str(render.base),
        barrel: leak_str(render.barrel),
        visual_size: fixed64(render.visual_size),
        barrel_frames: leak_string_slice(render.barrel_frames),
        body_frames: leak_string_slice(body_frames),
        barrel_animation: tower_animation(render.barrel_animation),
        body_animation: tower_animation(render.animation),
        rotation_mode: tower_rotation_mode(&render.rotation_mode)?,
        barrel_layout: tower_barrel_layout(&render.barrel_layout)?,
        barrel_variants: tower_barrel_variants(render.barrel_variants),
        barrel_offset: tower_point(render.barrel_offset),
        barrel_pivot: tower_point(render.barrel_pivot),
        muzzle_offset: tower_point(render.muzzle_offset),
        default_angle_deg: fixed64(render.default_angle_deg),
        recoil: TowerRecoilConst {
            mode: tower_recoil_mode(&render.recoil.mode)?,
            distance: fixed64(render.recoil.distance),
            scale: fixed64(render.recoil.scale),
            duration_ms: render.recoil.duration_ms,
            return_ms: render.recoil.return_ms,
        },
    })
}

fn leak_string_slice(values: Vec<String>) -> &'static [&'static str] {
    leak_slice(values.into_iter().map(leak_str).collect())
}

fn tower_animation(src: crate::lua_content::TowerAnimationEntry) -> TowerRenderAnimationConst {
    TowerRenderAnimationConst {
        fps: fixed64(src.fps),
        loop_animation: src.loop_animation,
        fire_fps: fixed64(src.fire_fps),
        fire_once: src.fire_once,
    }
}

fn tower_point(src: crate::lua_content::TowerPointEntry) -> TowerRenderPointConst {
    TowerRenderPointConst {
        x: fixed64(src.x),
        y: fixed64(src.y),
    }
}

fn tower_barrel_variants(
    values: Vec<crate::lua_content::TowerBarrelVariantEntry>,
) -> &'static [TowerBarrelVariantConst] {
    leak_slice(
        values
            .into_iter()
            .map(|value| TowerBarrelVariantConst {
                min_path: value.min_path,
                min_level: value.min_level,
                count: value.count,
                image: leak_str(value.image),
                frames: leak_string_slice(value.frames),
            })
            .collect(),
    )
}

fn tower_render_mode(value: &str) -> Result<TowerRenderModeC, String> {
    match value {
        "" | "base_barrel" => Ok(TowerRenderModeC::BaseBarrel),
        "animated_area" => Ok(TowerRenderModeC::AnimatedArea),
        other => Err(format!(
            "unknown tower render_mode '{}', expected base_barrel|animated_area",
            other
        )),
    }
}

fn tower_rotation_mode(value: &str) -> Result<TowerRotationModeC, String> {
    match value {
        "" | "targeted" | "target-facing" | "target_facing" => Ok(TowerRotationModeC::Targeted),
        "fixed" => Ok(TowerRotationModeC::Fixed),
        other => Err(format!(
            "unknown tower rotation_mode '{}', expected targeted|fixed",
            other
        )),
    }
}

fn tower_barrel_layout(value: &str) -> Result<TowerBarrelLayoutC, String> {
    match value {
        "" | "single" => Ok(TowerBarrelLayoutC::Single),
        "radial_count_variants" => Ok(TowerBarrelLayoutC::RadialCountVariants),
        other => Err(format!(
            "unknown tower barrel_layout '{}', expected single|radial_count_variants",
            other
        )),
    }
}

fn tower_recoil_mode(value: &str) -> Result<TowerRecoilModeC, String> {
    match value {
        "" | "directional" => Ok(TowerRecoilModeC::Directional),
        "scale_pulse" => Ok(TowerRecoilModeC::ScalePulse),
        other => Err(format!(
            "unknown tower recoil.mode '{}', expected directional|scale_pulse",
            other
        )),
    }
}

fn build_hero_stats(entry: &HeroEntry) -> Result<HeroStats, String> {
    Ok(HeroStats {
        strength: entry.strength,
        agility: entry.agility,
        intelligence: entry.intelligence,
        primary_attribute: primary_attribute(&entry.primary_attribute)?,
        attack_range: fixed64(entry.attack_range),
        base_damage: entry.base_damage,
        base_armor: fixed64(entry.base_armor),
        base_hp: entry.base_hp,
        base_mana: entry.base_mana,
        move_speed: fixed64(entry.move_speed),
        turn_speed: fixed64(entry.turn_speed),
        level_growth: LevelGrowth {
            strength_per_level: fixed64(entry.level_growth.strength_per_level),
            agility_per_level: fixed64(entry.level_growth.agility_per_level),
            intelligence_per_level: fixed64(entry.level_growth.intelligence_per_level),
            damage_per_level: fixed64(entry.level_growth.damage_per_level),
            hp_per_level: fixed64(entry.level_growth.hp_per_level),
            mana_per_level: fixed64(entry.level_growth.mana_per_level),
        },
    })
}

fn build_hero_render_metadata(
    hero_id: &str,
    render: &HeroRenderEntry,
) -> Result<HeroRenderMetadataConst, String> {
    if render.render_mode != "model_3d" {
        return Err(format!(
            "hero '{}' render_mode '{}' expected model_3d",
            hero_id, render.render_mode
        ));
    }
    if render.scale <= 0.0 {
        return Err(format!("hero '{}' render scale must be > 0", hero_id));
    }
    if render.animation_sources.is_empty() {
        return Err(format!(
            "hero '{}' render animation_sources must not be empty",
            hero_id
        ));
    }
    let sources = leak_slice(
        render
            .animation_sources
            .iter()
            .map(|(key, source)| HeroAnimationSourceConst {
                key: leak_str(key.clone()),
                model: leak_str(source.model.clone()),
                animation: leak_str(source.animation.clone()),
                duration_ticks: fixed64(source.duration_ticks),
                ticks_per_second: fixed64(source.ticks_per_second),
                timeline_offset_ticks: fixed64(source.timeline_offset_ticks),
            })
            .collect(),
    );
    let bindings = leak_slice(
        render
            .animations
            .iter()
            .map(|(action, binding)| HeroAnimationBindingConst {
                action: leak_str(action.clone()),
                source: leak_str(binding.source.clone()),
                start_tick: fixed64(binding.start_tick),
                end_tick: fixed64(binding.end_tick),
                repeat_start_tick: fixed64(binding.repeat_start_tick),
                has_impact_tick: binding.impact_tick.is_some(),
                impact_tick: fixed64(binding.impact_tick.unwrap_or(0.0)),
                loop_animation: binding.loop_animation,
            })
            .collect(),
    );
    Ok(HeroRenderMetadataConst {
        render_mode: HeroRenderModeC::Model3d,
        model: leak_str(render.model.clone()),
        texture: leak_str(render.texture.clone()),
        scale: fixed64(render.scale),
        pitch_offset_deg: fixed64(render.pitch_offset_deg),
        roll_offset_deg: fixed64(render.roll_offset_deg),
        yaw_offset_deg: fixed64(render.yaw_offset_deg),
        z_offset: fixed64(render.z_offset),
        muzzle_bone: leak_str(render.muzzle_bone.clone()),
        animation_sources: sources,
        animations: bindings,
    })
}

fn primary_attribute(value: &str) -> Result<u8, String> {
    match value {
        "" | "strength" => Ok(0),
        "agility" => Ok(1),
        "intelligence" => Ok(2),
        other => Err(format!("unknown primary_attribute '{}'", other)),
    }
}

fn enemy_type(value: &str) -> Result<u8, String> {
    match value {
        "" | "caster" => Ok(0),
        "melee" => Ok(1),
        "ranged" => Ok(2),
        "boss" => Ok(3),
        other => Err(format!("unknown enemy_type '{}'", other)),
    }
}

fn ai_type(value: &str) -> Result<u8, String> {
    match value {
        "" | "defensive" => Ok(0),
        "aggressive" => Ok(1),
        "patrol" => Ok(2),
        "guard" => Ok(3),
        "passive" => Ok(4),
        "berserker" => Ok(5),
        other => Err(format!("unknown ai_type '{}'", other)),
    }
}

fn build_creep_stats(entry: &CreepEntry) -> Result<CreepStats, String> {
    Ok(CreepStats {
        hp: fixed64(entry.hp),
        armor: fixed64(entry.armor),
        magic_resistance: fixed64(entry.magic_resistance),
        damage: fixed64(entry.damage),
        attack_range: fixed64(entry.attack_range),
        move_speed: fixed64(entry.move_speed),
        enemy_type: enemy_type(&entry.enemy_type)?,
        ai_type: ai_type(&entry.ai_type)?,
        exp_reward: entry.exp_reward,
        gold_reward: entry.gold_reward,
    })
}

fn ability_const_from_entry(entry: &AbilityEntry) -> Result<AbilityConst, String> {
    if entry.levels.len() != entry.max_level as usize {
        return Err(format!(
            "ability '{}': levels.len()={} but max_level={}",
            entry.id,
            entry.levels.len(),
            entry.max_level
        ));
    }
    let levels = leak_slice(
        entry
            .levels
            .iter()
            .map(|level| AbilityLevelDataConst {
                cooldown: fixed64(level.cooldown),
                mana_cost: fixed64(level.mana_cost),
                cast_time: fixed64(level.cast_time),
                range: fixed64(level.range),
            })
            .collect(),
    );
    let mut extras = Vec::new();
    for (key, values) in &entry.extras {
        if values.len() != entry.max_level as usize {
            return Err(format!(
                "ability '{}': extras['{}'].len()={} but max_level={}",
                entry.id,
                key,
                values.len(),
                entry.max_level
            ));
        }
        extras.push((
            leak_str(key.clone()),
            leak_slice(values.iter().copied().map(fixed64).collect()),
        ));
    }
    Ok(AbilityConst {
        ability_type: ability_type(&entry.ability_type)?,
        cast_type: cast_type(&entry.cast_type)?,
        target_type: target_type(&entry.target_type)?,
        max_level: entry.max_level,
        icon: leak_str(entry.icon.clone()),
        description: leak_str(entry.description.clone()),
        levels,
        extras: leak_slice(extras),
    })
}

fn ability_type(value: &str) -> Result<AbilityTypeC, String> {
    match value {
        "" | "active" => Ok(AbilityTypeC::Active),
        "toggle" => Ok(AbilityTypeC::Toggle),
        "ultimate" => Ok(AbilityTypeC::Ultimate),
        "passive" => Ok(AbilityTypeC::Passive),
        other => Err(format!("unknown ability_type '{}'", other)),
    }
}

fn cast_type(value: &str) -> Result<CastTypeC, String> {
    match value {
        "" | "instant" => Ok(CastTypeC::Instant),
        "channeled" => Ok(CastTypeC::Channeled),
        other => Err(format!("unknown cast_type '{}'", other)),
    }
}

fn target_type(value: &str) -> Result<TargetTypeC, String> {
    match value {
        "" | "none" => Ok(TargetTypeC::None),
        "point" => Ok(TargetTypeC::Point),
        "unit" => Ok(TargetTypeC::Unit),
        other => Err(format!("unknown target_type '{}'", other)),
    }
}

fn build_tower_upgrades(
    entry: &TowerEntry,
) -> Result<&'static [&'static [UpgradeDefConst]], String> {
    if entry.upgrades.len() != 3 {
        return Err(format!(
            "tower '{}' upgrades must have 3 paths, got {}",
            entry.id,
            entry.upgrades.len()
        ));
    }
    let mut paths = Vec::new();
    for (path_idx, path) in entry.upgrades.iter().enumerate() {
        if path.len() != 4 {
            return Err(format!(
                "tower '{}' path {} must have 4 levels, got {}",
                entry.id,
                path_idx,
                path.len()
            ));
        }
        let mut defs = Vec::new();
        for (level_idx, upgrade) in path.iter().enumerate() {
            let level = (level_idx + 1) as u8;
            let expected = upgrade_cost(entry.cost, level);
            if upgrade.cost != expected {
                return Err(format!(
                    "tower '{}' path {} L{} cost mismatch: declared {} but base {} x multiplier = {}",
                    entry.id, path_idx, level, upgrade.cost, entry.cost, expected
                ));
            }
            let effects = leak_slice(
                upgrade
                    .effects
                    .iter()
                    .map(|effect| match effect {
                        UpgradeEffectEntry::StatMod { key, value, op } => Ok(UpgradeEffectConst {
                            kind: UpgradeEffectKindC::StatMod,
                            key: leak_str(key.clone()),
                            value: fixed64(*value),
                            op: stat_op(op)?,
                        }),
                        UpgradeEffectEntry::BehaviorFlag { flag } => Ok(UpgradeEffectConst {
                            kind: UpgradeEffectKindC::BehaviorFlag,
                            key: leak_str(flag.clone()),
                            value: Fixed64::from_raw(0),
                            op: StatOpC::Add,
                        }),
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            );
            defs.push(UpgradeDefConst {
                name: leak_str(upgrade.name.clone()),
                description: leak_str(upgrade.description.clone()),
                cost: upgrade.cost,
                effects,
                active_ability: upgrade.active_ability.as_ref().map(|ability| {
                    let raw = validate_active_ability_quantization(ability)
                        .expect("active ability was validated before runtime conversion");
                    ActiveAbilityConst {
                        ability_id: leak_str(ability.ability_id.clone()),
                        display_name: leak_str(ability.display_name.clone()),
                        description: leak_str(ability.description.clone()),
                        icon: leak_str(ability.icon.clone()),
                        cooldown: Fixed64::from_raw(raw.cooldown),
                        duration: Fixed64::from_raw(raw.duration),
                        pulse_interval: Fixed64::from_raw(raw.pulse_interval),
                        pulse_count: ability.pulse_count,
                    }
                }),
            });
        }
        paths.push(leak_slice(defs));
    }
    Ok(leak_slice(paths))
}

fn validate_active_abilities(entries: &[TowerEntry]) -> Result<(), String> {
    let mut seen = HashSet::new();
    for entry in entries {
        for (path_idx, path) in entry.upgrades.iter().enumerate() {
            for (level_idx, upgrade) in path.iter().enumerate() {
                let Some(ability) = &upgrade.active_ability else {
                    continue;
                };
                if level_idx != 3 {
                    return Err(format!(
                        "tower '{}' path {} L{} active ability must be on level 4",
                        entry.id,
                        path_idx,
                        level_idx + 1
                    ));
                }
                if ability.ability_id.trim().is_empty() {
                    return Err(format!(
                        "tower '{}' active ability id must be non-empty",
                        entry.id
                    ));
                }
                if !seen.insert(ability.ability_id.as_str()) {
                    return Err(format!(
                        "duplicate active ability id: {}",
                        ability.ability_id
                    ));
                }
                validate_active_ability_quantization(ability)?;
            }
        }
    }
    Ok(())
}

fn upgrade_cost(base_cost: i32, level: u8) -> i32 {
    let mul: f32 = match level {
        1 => 0.25,
        2 => 0.50,
        3 => 1.00,
        4 => 2.50,
        _ => return 0,
    };
    (base_cost as f32 * mul) as i32
}

fn stat_op(value: &str) -> Result<StatOpC, String> {
    match value {
        "" | "add" => Ok(StatOpC::Add),
        "mul" => Ok(StatOpC::Mul),
        other => Err(format!("unknown stat_op '{}', expected add|mul", other)),
    }
}

fn story_value(value: JsonValue) -> StoryValue {
    match value {
        JsonValue::Null => StoryValue::Null,
        JsonValue::Bool(value) => StoryValue::Bool(value),
        JsonValue::Number(value) => StoryValue::Number(value.as_f64().unwrap_or(0.0)),
        JsonValue::String(value) => StoryValue::String(leak_str(value)),
        JsonValue::Array(values) => {
            StoryValue::Array(leak_slice(values.into_iter().map(story_value).collect()))
        }
        JsonValue::Object(values) => StoryValue::Object(leak_slice(
            values
                .into_iter()
                .map(|(key, value)| (leak_str(key), story_value(value)))
                .collect(),
        )),
    }
}

fn get_index<T: Copy>(values: &[Option<T>], raw: u16) -> Option<T> {
    values.get(raw as usize).copied().flatten()
}

pub fn tower_stats(id: TowerId) -> Option<&'static TowerStats> {
    active_content().and_then(|content| get_index(&content.tower_stats, id.0))
}

pub fn tower_display(id: TowerId) -> Option<&'static str> {
    active_content().and_then(|content| get_index(&content.tower_display, id.0))
}

pub fn tower_render_metadata(id: TowerId) -> Option<&'static TowerRenderMetadataConst> {
    active_content().and_then(|content| get_index(&content.tower_render, id.0))
}

pub fn tower_attack_timing(id: TowerId) -> Option<AttackTimingConst> {
    active_content().and_then(|content| get_index(&content.tower_attack_timing, id.0))
}

pub fn tower_upgrades(id: TowerId) -> Option<&'static [&'static [UpgradeDefConst]]> {
    active_content().and_then(|content| get_index(&content.tower_upgrades, id.0))
}

pub fn hero_stats(id: HeroId) -> Option<&'static HeroStats> {
    active_content().and_then(|content| get_index(&content.hero_stats, id.0))
}

pub fn hero_display(id: HeroId) -> Option<&'static str> {
    active_content().and_then(|content| get_index(&content.hero_display, id.0))
}

pub fn hero_title(id: HeroId) -> Option<&'static str> {
    active_content().and_then(|content| get_index(&content.hero_title, id.0))
}

pub fn hero_portrait(id: HeroId) -> Option<&'static str> {
    active_content().and_then(|content| get_index(&content.hero_portrait, id.0))
}

pub fn hero_render_metadata(id: HeroId) -> Option<&'static HeroRenderMetadataConst> {
    active_content().and_then(|content| get_index(&content.hero_render, id.0))
}

pub fn hero_abilities(id: HeroId) -> Option<&'static [AbilityId]> {
    active_content().and_then(|content| get_index(&content.hero_abilities, id.0))
}

pub fn hero_attack_timing(id: HeroId) -> Option<AttackTimingConst> {
    active_content().and_then(|content| get_index(&content.hero_attack_timing, id.0))
}

pub fn creep_stats(id: CreepId) -> Option<&'static CreepStats> {
    active_content().and_then(|content| get_index(&content.creep_stats, id.0))
}

pub fn creep_display(id: CreepId) -> Option<&'static str> {
    active_content().and_then(|content| get_index(&content.creep_display, id.0))
}

pub fn creep_attack_timing(id: CreepId) -> Option<AttackTimingConst> {
    active_content().and_then(|content| get_index(&content.creep_attack_timing, id.0))
}

pub fn summon_stats(id: SummonId) -> Option<&'static SummonStats> {
    active_content().and_then(|content| get_index(&content.summon_stats, id.0))
}

pub fn summon_display(id: SummonId) -> Option<&'static str> {
    active_content().and_then(|content| get_index(&content.summon_display, id.0))
}

pub fn summon_attack_timing(id: SummonId) -> Option<AttackTimingConst> {
    active_content().and_then(|content| get_index(&content.summon_attack_timing, id.0))
}

pub fn ability_const(id: AbilityId) -> Option<&'static AbilityConst> {
    active_content().and_then(|content| get_index(&content.ability_const, id.0))
}

pub fn ability_display(id: AbilityId) -> Option<&'static str> {
    active_content().and_then(|content| get_index(&content.ability_display, id.0))
}

pub fn ability_description(id: AbilityId) -> Option<&'static str> {
    active_content().and_then(|content| get_index(&content.ability_description, id.0))
}

pub fn td_layer_catalog() -> Option<&'static [TdLayerMetadataConst]> {
    active_content().map(|content| content.td_layer_catalog)
}

pub fn td_layer_by_name(id: &str) -> Option<&'static TdLayerMetadataConst> {
    active_content().and_then(|content| content.td_layers.get(id).copied())
}

pub fn td_layer_digest() -> Option<u64> {
    active_content().map(|content| content.td_layer_digest)
}

pub fn story_by_name(name: &str) -> Option<&'static GeneratedStory> {
    active_content().and_then(|content| content.stories.get(name).copied())
}

pub fn story_ids() -> Option<&'static [&'static str]> {
    active_content().map(|content| content.story_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{active_creep_stats, creep_id_str, CreepId, Fixed64};
    use std::fs;
    use std::path::Path;
    use std::sync::{Mutex, MutexGuard};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    fn temp_root(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("omoba_runtime_content_{name}_{stamp}"))
    }

    #[test]
    fn shipped_runtime_lua_td_layers_match_generated_catalog_and_digest() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("scripts/lua_data");
        let content = load_content(root).expect("load shipped Lua content");
        let runtime = RuntimeContent::from_manifest(content.manifest, content.stories)
            .expect("build runtime content");

        assert_eq!(runtime.td_layer_digest, crate::TD_LAYER_CATALOG_DIGEST);
        assert_eq!(runtime.td_layer_catalog, crate::td_layer_catalog());
        for generated in crate::td_layer_catalog() {
            assert_eq!(
                runtime.td_layers.get(generated.id).copied(),
                Some(generated),
                "{}",
                generated.id
            );
        }
    }

    fn write(path: &Path, text: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, text).unwrap();
    }

    fn write_templates(root: &Path, creep_id: &str, hp: f32) {
        write(
            &root.join("templates.lua"),
            &format!(
                "return function(ctx) return {{ creeps = {{ {{ id = '{}', hp = {}, move_speed = 100 }} }} }} end\n",
                creep_id, hp
            ),
        );
    }

    fn minimal_story(root: &Path, story: &str, creep: &str) {
        let dir = root.join(story);
        write(
            &dir.join("entity.lua"),
            "return function(ctx) return {} end\n",
        );
        write(
            &dir.join("ability.lua"),
            "return function(ctx) return {} end\n",
        );
        write(
            &dir.join("mission.lua"),
            "return function(ctx) return {} end\n",
        );
        write(
            &dir.join("map.lua"),
            &format!(
                "return function(ctx) return {{ Creep = {{ {{ Name = '{}' }} }} }} end\n",
                creep
            ),
        );
    }

    fn reset_store() {
        CONTENT
            .get_or_init(RuntimeContentStore::new)
            .reset_for_tests();
    }

    fn enable_runtime_content(root: &Path) {
        std::env::set_var("OMB_LUA_CONTENT", "1");
        std::env::set_var("OMB_LUA_CONTENT_ROOT", root);
        std::env::set_var("OMB_LUA_HOT_RELOAD", "1");
    }

    fn cleanup(root: PathBuf) {
        std::env::remove_var("OMB_LUA_CONTENT");
        std::env::remove_var("OMB_LUA_CONTENT_ROOT");
        std::env::remove_var("OMB_LUA_HOT_RELOAD");
        reset_store();
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn reload_updates_generation_hash_and_active_values() {
        let _guard = test_lock();
        let root = temp_root("reload_values");
        let creep_id = creep_id_str(CreepId(1));
        write_templates(&root, creep_id, 100.0);
        minimal_story(&root, "S", creep_id);
        enable_runtime_content(&root);
        reset_store();

        ensure_loaded().unwrap().unwrap();
        let initial = runtime_lua_content_info().unwrap().unwrap();
        assert_eq!(initial.generation, 1);
        assert_eq!(
            active_creep_stats(CreepId(1)).unwrap().hp,
            Fixed64::from_raw(100 * 1024)
        );

        write_templates(&root, creep_id, 150.0);
        let reloaded = reload_runtime_lua_content_dev(None).unwrap().unwrap();
        assert_eq!(reloaded.generation, 2);
        assert_ne!(reloaded.hash, initial.hash);
        assert_eq!(
            active_creep_stats(CreepId(1)).unwrap().hp,
            Fixed64::from_raw(150 * 1024)
        );

        cleanup(root);
    }

    #[test]
    fn invalid_reload_keeps_previous_generation_active() {
        let _guard = test_lock();
        let root = temp_root("invalid_reload");
        let creep_id = creep_id_str(CreepId(1));
        write_templates(&root, creep_id, 100.0);
        minimal_story(&root, "S", creep_id);
        enable_runtime_content(&root);
        reset_store();
        ensure_loaded().unwrap().unwrap();

        write(
            &root.join("templates.lua"),
            "return function(ctx) return { creeps =\n",
        );
        let err = reload_runtime_lua_content_dev(None).unwrap_err();
        assert!(err.contains("load Lua builder templates.lua"), "{err}");
        let info = runtime_lua_content_info().unwrap().unwrap();
        assert_eq!(info.generation, 1);
        assert_eq!(
            active_creep_stats(CreepId(1)).unwrap().hp,
            Fixed64::from_raw(100 * 1024)
        );

        cleanup(root);
    }

    #[test]
    fn structural_reload_is_rejected() {
        let _guard = test_lock();
        let root = temp_root("structural_reload");
        let creep_id = creep_id_str(CreepId(1));
        write_templates(&root, creep_id, 100.0);
        minimal_story(&root, "S", creep_id);
        enable_runtime_content(&root);
        reset_store();
        ensure_loaded().unwrap().unwrap();

        let second_creep = creep_id_str(CreepId(2));
        write(
            &root.join("templates.lua"),
            &format!(
                "return function(ctx) return {{ creeps = {{ {{ id = '{}' }}, {{ id = '{}' }} }} }} end\n",
                creep_id, second_creep
            ),
        );
        let err = reload_runtime_lua_content_dev(None).unwrap_err();
        assert!(err.contains("creep ids changed"), "{err}");
        assert_eq!(runtime_lua_content_generation().unwrap(), Some(1));

        cleanup(root);
    }

    #[test]
    fn content_hash_is_deterministic() {
        let _guard = test_lock();
        let root = temp_root("hash_deterministic");
        let creep_id = creep_id_str(CreepId(1));
        write_templates(&root, creep_id, 100.0);
        minimal_story(&root, "S", creep_id);

        let first = load_content(root.clone()).unwrap();
        let first_hash = content_hash(&first.manifest, &first.stories).unwrap();
        let second = load_content(root.clone()).unwrap();
        let second_hash = content_hash(&second.manifest, &second.stories).unwrap();
        assert_eq!(first_hash, second_hash);

        fs::remove_dir_all(root).ok();
    }
}
