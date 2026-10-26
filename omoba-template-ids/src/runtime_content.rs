use crate::lua_content::{
    load_content, validate_active_ability_quantization, AbilityEntry, AttackTimingEntry,
    CreepEntry, HeroEntry, HeroRenderEntry, Manifest, StoryBundle, SummonEntry, TdLayerEntry,
    TowerEntry, UpgradeEffectEntry, MobaTrinketEntry,
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
    base_recovery_hp_per_second: u32,
    base_recovery_radius: u32,
    mana_regen_per_second: u32,
    base_recovery_mana_per_second: u32,
    hero_kill_xp: u32,
    hero_assist_xp: u32,
    lane_creep_xp: u32,
    lane_creep_gold: u32,
    lane_xp_radius: u32,
    passive_gold_per_second: u32,
    hero_kill_gold: u32,
    first_kill_bonus_gold: u32,
    kill_streak_bounty_json: String,
    objective_comeback_json: String,
    death_streak_basis_points: Vec<u32>,
    recall_channel_seconds: u32,
    hero_assist_gold: u32,
    assist_window_seconds: u32,
    moba_item_catalog_json: String,
    moba_summoner_catalog_json: String,
    moba_trinket_catalog_json: String,
    moba_trinkets: Vec<MobaTrinketEntry>,
    communication_content_json: String,
    moba_map_catalog_json: String,
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

/// Equipment is compiled/shared, unlike dev hot-reloaded tower stats.
pub fn validate_compiled_moba_catalog() -> Result<(), String> {
    if let Some(content) = ensure_loaded()? {
        content.validate_moba_catalog()?;
    }
    Ok(())
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
    fn validate_moba_catalog(&self) -> Result<(), String> {
        for (index, runtime) in self.hero_stats.iter().enumerate() {
            if let Some(runtime) = runtime {
                let compiled = crate::hero_stats(HeroId(index as u16)).ok_or("missing compiled hero loadout")?;
                if runtime.moba_loadout != compiled.moba_loadout {
                    return Err("MOBA hero loadout differs from compiled content; rebuild all peers".into());
                }
            }
        }
        for (index, runtime) in self.ability_const.iter().enumerate() {
            if let Some(runtime) = runtime {
                let compiled = crate::ability_const(AbilityId(index as u16)).ok_or("missing compiled ability progression")?;
                if runtime.cleanses_allies!=compiled.cleanses_allies {
                    return Err("MOBA ally cleanse execution differs from compiled content; rebuild all peers".into());
                }
                if runtime.recovery_channel!=compiled.recovery_channel
                    || (runtime.recovery_channel.is_some() && (runtime.cast_type!=compiled.cast_type || runtime.target_type!=compiled.target_type)) {
                    return Err("MOBA recovery channel execution differs from compiled content; rebuild all peers".into());
                }
                if runtime.facing_formation_rows!=compiled.facing_formation_rows {
                    return Err("MOBA facing formation differs from compiled content; rebuild all peers".into());
                }
                if runtime.aim_preview_signature!=compiled.aim_preview_signature {
                    return Err("MOBA aim preview differs from compiled content; rebuild all peers".into());
                }
                if runtime.cone_half_angle_ticks!=compiled.cone_half_angle_ticks {
                    return Err("MOBA executable cone geometry differs from compiled content; rebuild all peers".into());
                }
                if runtime.levels.iter().map(|v| (v.required_hero_level,v.cast_interrupt_on_damage)).collect::<Vec<_>>()
                    != compiled.levels.iter().map(|v| (v.required_hero_level,v.cast_interrupt_on_damage)).collect::<Vec<_>>() {
                    return Err("MOBA ability progression differs from compiled content; rebuild all peers".into());
                }
            }
        }
        if self.lane_creep_xp != crate::MOBA_LANE_CREEP_XP || self.lane_xp_radius != crate::MOBA_LANE_XP_RADIUS {
            return Err("MOBA lane XP rules differ from compiled content; rebuild all peers".into());
        }
        if self.lane_creep_gold != crate::MOBA_LANE_CREEP_GOLD {
            return Err("MOBA lane gold rules differ from compiled content; rebuild all peers".into());
        }
        if self.hero_kill_xp != crate::MOBA_HERO_KILL_XP || self.hero_assist_xp != crate::MOBA_HERO_ASSIST_XP {
            return Err("MOBA XP rules differ from compiled content; rebuild all peers".into());
        }
        if self.hero_assist_gold != crate::MOBA_HERO_ASSIST_GOLD || self.assist_window_seconds != crate::MOBA_ASSIST_WINDOW_SECONDS {
            return Err("MOBA assist rules differ from compiled content; rebuild all peers".into());
        }
        if self.base_recovery_hp_per_second != crate::MOBA_BASE_RECOVERY_HP_PER_SECOND
            || self.base_recovery_mana_per_second != crate::MOBA_BASE_RECOVERY_MANA_PER_SECOND
            || self.base_recovery_radius != crate::MOBA_BASE_RECOVERY_RADIUS {
            return Err("MOBA base recovery rules differ from compiled content; rebuild all peers".into());
        }
        if self.recall_channel_seconds != crate::MOBA_RECALL_CHANNEL_SECONDS {
            return Err("MOBA recall rules differ from compiled content; rebuild all peers".into());
        }
        if self.mana_regen_per_second != crate::MOBA_MANA_REGEN_PER_SECOND {
            return Err("MOBA mana regeneration rules differ from compiled content; rebuild all peers".into());
        }
        if serde_json::to_string(&self.death_streak_basis_points).unwrap() != crate::MOBA_DEATH_STREAK_BASIS_POINTS_JSON {
            return Err("MOBA death streak curve differs from compiled content; rebuild all peers".into());
        }
        if self.objective_comeback_json != crate::MOBA_OBJECTIVE_COMEBACK_JSON {
            return Err("MOBA objective comeback differs from compiled content; rebuild all peers".into());
        }
        if self.kill_streak_bounty_json != crate::MOBA_KILL_STREAK_BOUNTY_JSON {
            return Err("MOBA kill streak bounty differs from compiled content; rebuild all peers".into());
        }
        if self.first_kill_bonus_gold != crate::MOBA_FIRST_KILL_BONUS_GOLD {
            return Err("MOBA first kill bonus differs from compiled content; rebuild all peers".into());
        }
        if self.hero_kill_gold != crate::MOBA_HERO_KILL_GOLD {
            return Err("MOBA kill rules differ from compiled content; rebuild all peers".into());
        }
        if self.passive_gold_per_second != crate::MOBA_PASSIVE_GOLD_PER_SECOND {
            return Err("MOBA income rules differ from compiled content; rebuild all peers".into());
        }
        if self.moba_item_catalog_json != crate::MOBA_ITEM_CATALOG_JSON {
            return Err("MOBA item catalog differs from compiled content; rebuild all peers".into());
        }
        if !self.trinkets_match_compiled() {
            return Err("MOBA trinket catalog differs from compiled content; rebuild all peers".into());
        }
        if self.moba_summoner_catalog_json != crate::MOBA_SUMMONER_CATALOG_JSON {
            return Err("MOBA summoner catalog differs from compiled content; rebuild all peers".into());
        }
        if self.communication_content_json != crate::COMMUNICATION_CONTENT_JSON {
            return Err("communication catalog differs from compiled content; rebuild all peers".into());
        }
        if self.moba_map_catalog_json != crate::MOBA_MAP_CATALOG_JSON {
            return Err("MOBA map catalog differs from compiled content; rebuild all peers".into());
        }
        Ok(())
    }

    fn trinkets_match_compiled(&self) -> bool {
        self.moba_trinket_catalog_json == crate::MOBA_TRINKET_CATALOG_JSON
            && self.moba_trinkets.len() == crate::TRINKET_CATALOG.len()
            && self.moba_trinkets.iter().zip(crate::TRINKET_CATALOG).all(|(runtime, compiled)| {
                let direct_attack = match runtime.direct_attack {
                    omoba_content_model::moba_trinkets::MobaTrinketDirectAttack::DisclosedEnemy => {
                        crate::TrinketDirectAttackConst::DisclosedEnemy
                    }
                    omoba_content_model::moba_trinkets::MobaTrinketDirectAttack::NotDirectlyAttackable => {
                        crate::TrinketDirectAttackConst::NotDirectlyAttackable
                    }
                };
                let automatic_acquisition = match runtime.automatic_acquisition {
                    omoba_content_model::moba_trinkets::MobaTrinketAutomaticAcquisition::Disabled => {
                        crate::TrinketAutomaticAcquisitionConst::Disabled
                    }
                    omoba_content_model::moba_trinkets::MobaTrinketAutomaticAcquisition::Enabled => {
                        crate::TrinketAutomaticAcquisitionConst::Enabled
                    }
                };
                runtime.catalog_id == compiled.catalog_id
                    && runtime.id == compiled.id
                    && runtime.display_name == compiled.display_name
                    && runtime.description == compiled.description
                    && fixed64(runtime.placement_range) == compiled.placement_range
                    && runtime.initial_charges == compiled.initial_charges
                    && runtime.max_charges == compiled.max_charges
                    && fixed64(runtime.recharge_seconds) == compiled.recharge_seconds
                    && fixed64(runtime.lifetime_seconds) == compiled.lifetime_seconds
                    && fixed64(runtime.sight_radius) == compiled.sight_radius
                    && runtime.detection_level == compiled.detection_level
                    && runtime.stealth_level == compiled.stealth_level
                    && runtime.hit_points == compiled.hit_points
                    && runtime.maximum_owned_placements == compiled.maximum_owned_placements
                    && direct_attack == compiled.direct_attack
                    && automatic_acquisition == compiled.automatic_acquisition
            })
    }

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
                attack_range: fixed64(entry.attack_range),
                attack_interval: fixed64(entry.attack_interval),
                armor: fixed64(entry.armor),
                magic_resistance: fixed64(entry.magic_resistance),
                collision_radius: fixed64(entry.collision_radius),
                vision_radius: fixed64(entry.vision_radius),
                follow_start_distance: fixed64(entry.follow_start_distance),
                follow_stop_distance: fixed64(entry.follow_stop_distance),
                target_priority: match entry.target_priority {
                    crate::lua_content::SummonTargetPriorityEntry::Nearest=>crate::SummonTargetPriority::Nearest,
                    crate::lua_content::SummonTargetPriorityEntry::OwnerFocus=>crate::SummonTargetPriority::OwnerFocus,
                },
                immobile: entry.immobile,
                damage_per_owner_level: fixed64(entry.damage_per_owner_level),
                owner_death: match entry.owner_death {
                    crate::lua_content::SummonOwnerDeathEntry::Persist=>crate::SummonOwnerDeath::Persist,
                    crate::lua_content::SummonOwnerDeathEntry::Expire=>crate::SummonOwnerDeath::Expire,
                },
                invulnerable: entry.invulnerable,
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
            moba_map_catalog_json: serde_json::to_string(&manifest.moba_maps)
                .map_err(|error| format!("serialize MOBA maps: {error}"))?,
            moba_item_catalog_json: serde_json::to_string(&manifest.moba_items)
                .map_err(|error| format!("serialize MOBA items: {error}"))?,
            moba_summoner_catalog_json: serde_json::to_string(&manifest.moba_summoners)
                .map_err(|error| format!("serialize MOBA summoners: {error}"))?,
            moba_trinket_catalog_json: serde_json::to_string(&manifest.moba_trinkets)
                .map_err(|error| format!("serialize MOBA trinkets: {error}"))?,
            moba_trinkets: manifest.moba_trinkets.clone(),
            communication_content_json: serde_json::to_string(&manifest.moba_communication)
                .map_err(|error| format!("serialize communication catalog: {error}"))?,
            passive_gold_per_second: manifest.moba_economy.passive_gold_per_second,
            base_recovery_hp_per_second: manifest.moba_economy.base_recovery_hp_per_second,
            base_recovery_radius: manifest.moba_economy.base_recovery_radius,
            mana_regen_per_second: manifest.moba_economy.mana_regen_per_second,
            base_recovery_mana_per_second: manifest.moba_economy.base_recovery_mana_per_second,
            hero_kill_xp: manifest.moba_economy.hero_kill_xp,
            hero_assist_xp: manifest.moba_economy.hero_assist_xp,
            lane_creep_xp: manifest.moba_economy.lane_creep_xp,
            lane_creep_gold: manifest.moba_economy.lane_creep_gold,
            lane_xp_radius: manifest.moba_economy.lane_xp_radius,
            hero_kill_gold: manifest.moba_economy.hero_kill_gold,
            first_kill_bonus_gold: manifest.moba_economy.first_kill_bonus_gold,
            kill_streak_bounty_json: serde_json::to_string(&manifest.moba_economy.kill_streak_bounty).unwrap(),
            objective_comeback_json: serde_json::to_string(&manifest.moba_economy.objective_comeback).unwrap(),
            death_streak_basis_points: manifest.moba_economy.death_streak_basis_points.clone(),
            recall_channel_seconds: manifest.moba_economy.recall_channel_seconds,
            hero_assist_gold: manifest.moba_economy.hero_assist_gold,
            assist_window_seconds: manifest.moba_economy.assist_window_seconds,
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
    moba_summoner_catalog_json: String,
    communication_content_json: String,
    base_recovery_hp_per_second: u32,
    base_recovery_radius: u32,
    mana_regen_per_second: u32,
    base_recovery_mana_per_second: u32,
    moba_map_catalog_json: String,
    hero_moba_loadouts: Vec<(String, [u8; 4], u8)>,
    ability_rank_requirements: Vec<(String, u16, Vec<(u8,bool)>)>,
    ability_aim_previews:Vec<(String,String)>,
    ability_recovery_channels:Vec<(String,(String,String,String))>,
    ability_ally_cleanses:Vec<String>,
    hero_kill_xp: u32,
    hero_assist_xp: u32,
    lane_creep_xp: u32,
    lane_creep_gold: u32,
    lane_xp_radius: u32,
    passive_gold_per_second: u32,
    hero_kill_gold: u32,
    first_kill_bonus_gold: u32,
    kill_streak_bounty_json: String,
    objective_comeback_json: String,
    death_streak_basis_points: Vec<u32>,
    recall_channel_seconds: u32,
    hero_assist_gold: u32,
    assist_window_seconds: u32,
    moba_item_catalog_json: String,
    moba_trinket_catalog_json: String,
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
            moba_summoner_catalog_json: serde_json::to_string(&manifest.moba_summoners)
                .expect("validated MOBA summoner catalog"),
            communication_content_json: serde_json::to_string(&manifest.moba_communication)
                .expect("validated communication catalog"),
            moba_map_catalog_json: serde_json::to_string(&manifest.moba_maps)
                .expect("validated MOBA map catalog"),
            hero_moba_loadouts: manifest.heroes.iter().filter(|entry| !entry.tombstone)
                .map(|entry| (entry.id.clone(), entry.moba_loadout.ranks, entry.moba_loadout.skill_points)).collect(),
            ability_rank_requirements: manifest.abilities.iter().filter(|entry| !entry.tombstone)
                .map(|entry| (entry.id.clone(), omoba_content_model::facing_formation_rows(entry),entry.levels.iter().map(|level| (level.required_hero_level,level.cast_interrupt_on_damage)).collect())).collect(),
            ability_aim_previews:manifest.abilities.iter().filter(|entry|!entry.tombstone)
                .map(|entry|(entry.id.clone(),omoba_content_model::ability_aim_preview_signature(entry).expect("validated aim preview"))).collect(),
            ability_recovery_channels:manifest.abilities.iter().filter(|entry|!entry.tombstone).filter_map(|entry|
                match entry.effects.as_slice() {
                    [omoba_content_model::AbilityEffect::RecoveryChannelSelf {amount_key,duration_key,interval_key}]=>
                        Some((entry.id.clone(),(amount_key.clone(),duration_key.clone(),interval_key.clone()))),
                    _=>None,
                }).collect(),
            ability_ally_cleanses:manifest.abilities.iter().filter(|entry|!entry.tombstone && entry.effects.iter().any(|effect|matches!(effect,omoba_content_model::AbilityEffect::CleanseAlly {})))
                .map(|entry|entry.id.clone()).collect(),
            moba_item_catalog_json: serde_json::to_string(&manifest.moba_items)
                .expect("validated MOBA item catalog"),
            moba_trinket_catalog_json: serde_json::to_string(&manifest.moba_trinkets)
                .expect("validated MOBA trinket catalog"),
            passive_gold_per_second: manifest.moba_economy.passive_gold_per_second,
            base_recovery_hp_per_second: manifest.moba_economy.base_recovery_hp_per_second,
            base_recovery_radius: manifest.moba_economy.base_recovery_radius,
            mana_regen_per_second: manifest.moba_economy.mana_regen_per_second,
            base_recovery_mana_per_second: manifest.moba_economy.base_recovery_mana_per_second,
            hero_kill_xp: manifest.moba_economy.hero_kill_xp,
            hero_assist_xp: manifest.moba_economy.hero_assist_xp,
            lane_creep_xp: manifest.moba_economy.lane_creep_xp,
            lane_creep_gold: manifest.moba_economy.lane_creep_gold,
            lane_xp_radius: manifest.moba_economy.lane_xp_radius,
            hero_kill_gold: manifest.moba_economy.hero_kill_gold,
            first_kill_bonus_gold: manifest.moba_economy.first_kill_bonus_gold,
            kill_streak_bounty_json: serde_json::to_string(&manifest.moba_economy.kill_streak_bounty).unwrap(),
            objective_comeback_json: serde_json::to_string(&manifest.moba_economy.objective_comeback).unwrap(),
            death_streak_basis_points: manifest.moba_economy.death_streak_basis_points.clone(),
            recall_channel_seconds: manifest.moba_economy.recall_channel_seconds,
            hero_assist_gold: manifest.moba_economy.hero_assist_gold,
            assist_window_seconds: manifest.moba_economy.assist_window_seconds,
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
        if self.ability_ally_cleanses!=previous.ability_ally_cleanses {
            return Err("MOBA ally cleanse execution changed; rebuild all peers, not hot reload".into());
        }
        if self.ability_recovery_channels!=previous.ability_recovery_channels {
            return Err("MOBA recovery channel execution changed; rebuild all peers, not hot reload".into());
        }
        if self.mana_regen_per_second != previous.mana_regen_per_second {
            return Err("MOBA mana regeneration rules changed; rebuild all peers, not hot reload".into());
        }
        if self.base_recovery_hp_per_second != previous.base_recovery_hp_per_second
            || self.base_recovery_mana_per_second != previous.base_recovery_mana_per_second
            || self.base_recovery_radius != previous.base_recovery_radius {
            return Err("MOBA base recovery rules changed; rebuild all peers, not hot reload".into());
        }
        if self.hero_moba_loadouts != previous.hero_moba_loadouts {
            return Err("MOBA hero loadout changed; rebuild all peers".into());
        }
        if self.ability_aim_previews!=previous.ability_aim_previews {
            return Err("MOBA aim preview changed; rebuild all peers, not hot reload".into());
        }
        if self.ability_rank_requirements != previous.ability_rank_requirements {
            return Err("MOBA ability progression changed; rebuild all peers, not hot reload".into());
        }
        if self.lane_creep_xp != previous.lane_creep_xp || self.lane_xp_radius != previous.lane_xp_radius {
            return Err("MOBA lane XP rules changed; rebuild all peers, not hot reload".into());
        }
        if self.lane_creep_gold != previous.lane_creep_gold {
            return Err("MOBA lane gold rules changed; rebuild all peers, not hot reload".into());
        }
        if self.hero_kill_xp != previous.hero_kill_xp || self.hero_assist_xp != previous.hero_assist_xp {
            return Err("MOBA XP rules changed; rebuild all peers, not hot reload".into());
        }
        if self.recall_channel_seconds != previous.recall_channel_seconds {
            return Err("MOBA recall rules changed; rebuild all peers, not hot reload".into());
        }
        if self.hero_assist_gold != previous.hero_assist_gold || self.assist_window_seconds != previous.assist_window_seconds {
            return Err("MOBA assist rules changed; rebuild all peers, not hot reload".into());
        }
        if self.death_streak_basis_points != previous.death_streak_basis_points {
            return Err("MOBA death streak curve changed; rebuild all peers, not hot reload".into());
        }
        if self.objective_comeback_json != previous.objective_comeback_json {
            return Err("MOBA objective comeback changed; rebuild all peers, not hot reload".into());
        }
        if self.kill_streak_bounty_json != previous.kill_streak_bounty_json {
            return Err("MOBA kill streak bounty changed; rebuild all peers, not hot reload".into());
        }
        if self.first_kill_bonus_gold != previous.first_kill_bonus_gold {
            return Err("MOBA first kill bonus changed; rebuild all peers, not hot reload".into());
        }
        if self.hero_kill_gold != previous.hero_kill_gold {
            return Err("MOBA kill rules changed; rebuild all peers, not hot reload".into());
        }
        if self.passive_gold_per_second != previous.passive_gold_per_second {
            return Err("MOBA income rules changed; rebuild all peers, not hot reload".into());
        }
        if self.moba_item_catalog_json != previous.moba_item_catalog_json {
            return Err("MOBA item catalog changed; rebuild all peers, not hot reload".into());
        }
        if self.moba_trinket_catalog_json != previous.moba_trinket_catalog_json {
            return Err("MOBA trinket catalog changed; rebuild all peers, not hot reload".into());
        }
        if self.moba_summoner_catalog_json != previous.moba_summoner_catalog_json {
            return Err("MOBA summoner catalog changed; rebuild all peers, not hot reload".into());
        }
        if self.communication_content_json != previous.communication_content_json {
            return Err("communication catalog changed; rebuild all peers, not hot reload".into());
        }
        if self.moba_map_catalog_json != previous.moba_map_catalog_json {
            return Err("MOBA map catalog changed; rebuild all peers, not hot reload".into());
        }
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
        moba_loadout: crate::MobaLoadoutConst { ranks: entry.moba_loadout.ranks, skill_points: entry.moba_loadout.skill_points },
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
    omoba_content_model::validate_ability_progression(entry)?;
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
                required_hero_level: level.required_hero_level,
                cooldown: fixed64(level.cooldown),
                mana_cost: fixed64(level.mana_cost),
                cast_time: fixed64(level.cast_time),
                cast_interrupt_on_damage: level.cast_interrupt_on_damage,
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
        cleanses_allies: entry.effects.iter().any(|effect|matches!(effect,omoba_content_model::AbilityEffect::CleanseAlly {})),
        auto_learn: entry.auto_learn,
        ability_type: ability_type(&entry.ability_type)?,
        cast_type: cast_type(&entry.cast_type)?,
        target_type: target_type(&entry.target_type)?,
        max_level: entry.max_level,
        zone_independent: entry.zone_lifetime==Some(omoba_content_model::GroundZoneLifetime::Independent),
        facing_formation_rows: omoba_content_model::facing_formation_rows(entry),
        cone_half_angle_ticks: leak_slice(omoba_content_model::ability_aim_preview(entry)?.ranks.iter().map(|rank|rank.cone_half_angle_ticks).collect()),
        line_widths: leak_slice(omoba_content_model::ability_aim_preview(entry)?.ranks.iter().map(|rank|if entry.effects.iter().any(|effect|matches!(effect,omoba_content_model::AbilityEffect::LineDamage{..})){fixed64(rank.path_width)}else{Fixed64::ZERO}).collect()),
        recovery_channel: match entry.effects.as_slice() {
            [omoba_content_model::AbilityEffect::RecoveryChannelSelf {amount_key,duration_key,interval_key}]=>
                Some(crate::RecoveryChannelConst {amount_key:leak_str(amount_key.clone()),duration_key:leak_str(duration_key.clone()),interval_key:leak_str(interval_key.clone())}),
            _=>None,
        },
        pulse_channel: entry.channel.as_ref().map(|channel|crate::PulseChannelConst {
            anchor:match channel.anchor {omoba_content_model::ChannelAnchor::SelfOrigin=>0,omoba_content_model::ChannelAnchor::Point=>1,omoba_content_model::ChannelAnchor::Unit=>2},
            interval_key:leak_str(channel.interval_key.clone()),pulses_key:leak_str(channel.pulses_key.clone()),
            radius_key:channel.radius_key.clone().map(leak_str)}),
        charges: entry.charges.as_ref().map(|charges|crate::ChargesConst {
            max_key:leak_str(charges.max_key.clone()),recharge_key:leak_str(charges.recharge_key.clone())}),
        attack_orb: entry.attack_orb.as_ref().map(|orb|crate::AttackOrbConst {
            mana_per_attack_key:leak_str(orb.mana_per_attack_key.clone()),amount_key:leak_str(orb.amount_key.clone()),
            damage_kind:match orb.damage_kind {omoba_content_model::EffectDamageKind::Physical=>0,omoba_content_model::EffectDamageKind::Magical=>1,omoba_content_model::EffectDamageKind::Pure=>2},
            move_slow_key:orb.move_slow_key.clone().map(leak_str),attack_slow_key:orb.attack_slow_key.clone().map(leak_str),
            slow_duration_key:orb.slow_duration_key.clone().map(leak_str)}),
        cast_cooldown_refund_key: entry.cast_cooldown_refund.as_ref().map(|refund|leak_str(refund.seconds_key.clone())),
        aim_preview_signature: leak_str(omoba_content_model::ability_aim_preview_signature(entry)?),
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
    #[test]
    fn bot_cleanse_runtime_and_compiled_capabilities_follow_effects() {
        let entry:super::AbilityEntry=serde_json::from_value(serde_json::json!({"id":"arbitrary_cleanse","ability_type":"active","cast_type":"instant","target_type":"unit","max_level":1,"levels":[{"range":600}],"effects":[{"kind":"cleanse_ally"}]})).unwrap();
        assert!(super::ability_const_from_entry(&entry).unwrap().cleanses_allies);
        let mut no_cleanse=entry.clone();no_cleanse.effects.clear();assert!(!super::ability_const_from_entry(&no_cleanse).unwrap().cleanses_allies);
        let aid=crate::ability_by_name("lumen_aid").and_then(crate::active_ability_const).unwrap();assert!(aid.cleanses_allies);
        let bolt=crate::ability_by_name("lumen_bolt").and_then(crate::active_ability_const).unwrap();assert!(!bolt.cleanses_allies);
        let repo=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();let mut content=super::load_content(repo.join("scripts/lua_data")).unwrap();
        let before=super::ContentShape::from_manifest(&content.manifest,&content.stories);
        let aid=content.manifest.abilities.iter_mut().find(|a|a.id=="lumen_aid").unwrap();aid.effects.retain(|effect|!matches!(effect,omoba_content_model::AbilityEffect::CleanseAlly {}));
        assert!(super::ContentShape::from_manifest(&content.manifest,&content.stories).ensure_compatible_with(&before).unwrap_err().contains("ally cleanse execution"));
        let runtime=super::RuntimeContent::from_manifest(content.manifest,content.stories).unwrap();assert!(runtime.validate_moba_catalog().unwrap_err().contains("ally cleanse execution"));
    }
    use super::*;
    use crate::{active_ability_const,active_creep_stats, creep_id_str, CreepId, Fixed64};
    use std::fs;
    use std::path::Path;
    use std::sync::{Mutex, MutexGuard};
    #[test]
    fn recovery_execution_projection_matches_compiled_and_blocks_semantic_reload() {
        let repo=Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut content=load_content(repo.join("scripts/lua_data")).unwrap();
        let before=ContentShape::from_manifest(&content.manifest,&content.stories);
        let entry=content.manifest.abilities.iter_mut().find(|a|a.id=="apprentice_mend").unwrap();
        let keys=ability_const_from_entry(entry).unwrap().recovery_channel.unwrap();
        assert_eq!((keys.amount_key,keys.duration_key,keys.interval_key),("pulse_heal","channel_duration","pulse_interval"));
        assert_eq!(crate::ability_const(crate::ability_by_name("apprentice_mend").unwrap()).unwrap().recovery_channel,Some(keys));
        assert!(crate::ability_const(crate::ability_by_name("lumen_mend").unwrap()).unwrap().recovery_channel.is_none());
        let mut arbitrary=entry.clone();arbitrary.id="any_authored_channel".into();
        assert_eq!(ability_const_from_entry(&arbitrary).unwrap().recovery_channel,Some(keys));
        let values=entry.extras["pulse_heal"].clone();entry.extras.insert("alternate_pulse".into(),values);
        let [omoba_content_model::AbilityEffect::RecoveryChannelSelf {amount_key,..}]=entry.effects.as_mut_slice()else{panic!("exclusive authored effect")};
        *amount_key="alternate_pulse".into();
        assert_eq!(ability_const_from_entry(entry).unwrap().recovery_channel.unwrap().amount_key,"alternate_pulse");
        assert!(ContentShape::from_manifest(&content.manifest,&content.stories).ensure_compatible_with(&before).unwrap_err().contains("recovery channel execution"));
        let runtime=RuntimeContent::from_manifest(content.manifest,content.stories).unwrap();
        assert!(runtime.validate_moba_catalog().unwrap_err().contains("recovery channel execution"));
    }
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
        assert_eq!(runtime.moba_item_catalog_json, crate::MOBA_ITEM_CATALOG_JSON);
        assert_eq!(runtime.moba_summoner_catalog_json, crate::MOBA_SUMMONER_CATALOG_JSON);
        assert_eq!(runtime.moba_trinket_catalog_json, crate::MOBA_TRINKET_CATALOG_JSON);
        assert!(runtime.trinkets_match_compiled());
        runtime.validate_moba_catalog().expect("compiled MOBA catalog agrees with Lua");
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

    #[test]
    fn moba_catalog_changes_require_rebuild_not_dev_hot_reload() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content = load_content(root).unwrap();
        let before = ContentShape::from_manifest(&content.manifest, &content.stories);
        let original_hash = omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        content.manifest_value["moba_items"][0]["cost"] = serde_json::json!(351);
        assert_ne!(original_hash, omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_items[0].cost += 1;
        let after = ContentShape::from_manifest(&content.manifest, &content.stories);
        assert!(after.ensure_compatible_with(&before).unwrap_err().contains("MOBA item catalog changed"));
        let runtime = RuntimeContent::from_manifest(content.manifest, content.stories).unwrap();
        assert!(runtime.validate_moba_catalog().unwrap_err().contains("rebuild all peers"));
    }

    #[test]
    fn communication_catalog_changes_require_rebuild_not_dev_hot_reload() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content = load_content(root).unwrap();
        let before = ContentShape::from_manifest(&content.manifest, &content.stories);
        let runtime = RuntimeContent::from_manifest(content.manifest, content.stories).unwrap();
        assert_eq!(runtime.communication_content_json, crate::COMMUNICATION_CONTENT_JSON);
        runtime.validate_moba_catalog().unwrap();
        content = load_content(PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data")).unwrap();
        content.manifest.moba_communication[0].subtitle.push('！');
        let after = ContentShape::from_manifest(&content.manifest, &content.stories);
        assert!(after.ensure_compatible_with(&before).unwrap_err().contains("communication catalog changed"));
        let runtime = RuntimeContent::from_manifest(content.manifest, content.stories).unwrap();
        assert!(runtime.validate_moba_catalog().unwrap_err().contains("communication catalog differs"));
    }

    #[test]
    fn summoner_catalog_field_and_membership_drift_requires_peer_rebuild() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let cases = [
            (0, "catalog_id", serde_json::json!(101)),
            (0, "id", serde_json::json!("different_identity")),
            (0, "display_name", serde_json::json!("Different label")),
            (0, "description", serde_json::json!("Different description")),
            (0, "cooldown", serde_json::json!(301)),
            (0, "operation", serde_json::json!({"kind":"point_blink","range":401,"out_of_range":"clamp_to_range"})),
            (0, "operation", serde_json::json!({"kind":"point_blink","range":400,"out_of_range":"reject"})),
            (1, "operation", serde_json::json!({"kind":"self_heal","amount":201})),
            (6, "operation", serde_json::json!({"kind":"self_restore_mana","amount":201})),
            (1, "operation", serde_json::json!({"kind":"self_shield","amount":200,"duration":2})),
            (2, "operation", serde_json::json!({"kind":"self_shield","amount":161,"duration":2})),
            (2, "operation", serde_json::json!({"kind":"self_shield","amount":160,"duration":3})),
        ];
        for (index, field, value) in cases {
            let mut content = load_content(root.clone()).unwrap();
            let before = ContentShape::from_manifest(&content.manifest, &content.stories);
            let mut rows = serde_json::to_value(&content.manifest.moba_summoners).unwrap();
            rows[index][field] = value;
            content.manifest.moba_summoners = serde_json::from_value(rows).unwrap();
            crate::lua_content::validate_moba_summoners(&content.manifest.moba_summoners).unwrap();
            let after = ContentShape::from_manifest(&content.manifest, &content.stories);
            assert!(after.ensure_compatible_with(&before).unwrap_err().contains("summoner catalog changed"), "{index}:{field}");
            let runtime = RuntimeContent::from_manifest(content.manifest, content.stories).unwrap();
            assert!(runtime.validate_moba_catalog().unwrap_err().contains("summoner catalog differs"), "{index}:{field}");
        }
        for mutation in 0..4 {
            let mut content = load_content(root.clone()).unwrap();
            let before = ContentShape::from_manifest(&content.manifest, &content.stories);
            match mutation {
                0 => { content.manifest.moba_summoners.pop(); }
                1 => { let mut added = content.manifest.moba_summoners[0].clone(); added.id.push_str("_new"); added.catalog_id = 101; content.manifest.moba_summoners.push(added); }
                2 => content.manifest.moba_summoners.reverse(),
                _ => { let spell = &mut content.manifest.moba_summoners[1]; spell.target = omoba_content_model::moba_summoners::MobaSummonerTarget::Point; spell.operation = omoba_content_model::moba_summoners::MobaSummonerOperation::PointBlink { range: 400., out_of_range: omoba_content_model::moba_summoners::MobaSummonerOutOfRange::Reject }; }
            }
            crate::lua_content::validate_moba_summoners(&content.manifest.moba_summoners).unwrap();
            assert!(ContentShape::from_manifest(&content.manifest, &content.stories).ensure_compatible_with(&before).unwrap_err().contains("summoner catalog changed"));
            assert!(RuntimeContent::from_manifest(content.manifest, content.stories).unwrap().validate_moba_catalog().unwrap_err().contains("summoner catalog differs"));
        }
    }

    #[test]
    fn rejected_real_summoner_reload_preserves_snapshot_and_supported_tuning() {
        fn copy_tree(source: &Path, dest: &Path) {
            fs::create_dir_all(dest).unwrap();
            for entry in fs::read_dir(source).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() { copy_tree(&entry.path(), &dest.join(entry.file_name())); }
                else { fs::copy(entry.path(), dest.join(entry.file_name())).unwrap(); }
            }
        }
        let _guard = test_lock();
        let root = temp_root("summoner_catalog_reload");
        copy_tree(&Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data"), &root);
        let entry = root.join("templates.lua");
        let original = fs::read_to_string(&entry).unwrap();
        let wrapper = |mutation: &str| format!("local original=(function()\n{}\nend)()\nreturn function(ctx) local data=original(ctx) {} return data end\n", original, mutation);
        enable_runtime_content(&root); reset_store();
        validate_compiled_moba_catalog().unwrap();
        let before = runtime_lua_content_info().unwrap().unwrap();
        let active = ensure_loaded().unwrap().unwrap() as *const RuntimeContent;
        for mutation in [
            "data.moba_summoners[1].operation.range=401",
            "data.moba_summoners[2].operation.amount=201",
            "data.moba_summoners[3].operation.duration=3",
            "data.moba_summoners[3].description='changed description'",
            "data.moba_summoners[3].id='changed_identity'",
            "table.remove(data.moba_summoners)",
        ] {
            write(&entry, &wrapper(mutation));
            assert!(CONTENT.get().unwrap().validate_dev().unwrap_err().contains("summoner catalog changed"));
            assert!(reload_runtime_lua_content_dev(None).unwrap_err().contains("summoner catalog changed"));
            assert_eq!(runtime_lua_content_info().unwrap().unwrap(), before);
            assert_eq!(ensure_loaded().unwrap().unwrap() as *const RuntimeContent, active);
            validate_compiled_moba_catalog().unwrap();
        }
        // A fresh startup with changed summoners fails the public peer gate too.
        reset_store();
        assert!(validate_compiled_moba_catalog().unwrap_err().contains("summoner catalog differs"));
        write(&entry, &original); reset_store(); validate_compiled_moba_catalog().unwrap();
        let before = runtime_lua_content_info().unwrap().unwrap();
        let damage = active_ability_const(crate::ABILITY_FLAME_BLADE).unwrap().extras.iter().find(|(key,_)| *key == "damage").unwrap().1[0];
        write(&entry, &wrapper("for _,a in ipairs(data.abilities) do if a.id=='flame_blade' then a.extras.damage[1]=a.extras.damage[1]+1 end end"));
        let accepted = reload_runtime_lua_content_dev(None).unwrap().unwrap();
        assert_eq!(accepted.generation, before.generation + 1); assert_ne!(accepted.hash, before.hash);
        assert_eq!(active_ability_const(crate::ABILITY_FLAME_BLADE).unwrap().extras.iter().find(|(key,_)| *key == "damage").unwrap().1[0], damage + Fixed64::ONE);
        validate_compiled_moba_catalog().unwrap();
        cleanup(root);
    }

    #[test]
    fn moba_loadout_changes_require_rebuild_not_dev_hot_reload() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content = load_content(root).unwrap();
        let before = ContentShape::from_manifest(&content.manifest, &content.stories);
        content.manifest.heroes.iter_mut().find(|hero| hero.id == "training_luminary").unwrap()
            .common.moba_loadout = omoba_content_model::MobaLoadout { ranks: [0;5], skill_points: 1 };
        let after = ContentShape::from_manifest(&content.manifest, &content.stories);
        assert!(after.ensure_compatible_with(&before).unwrap_err().contains("MOBA hero loadout changed"));
        let runtime = RuntimeContent::from_manifest(content.manifest, content.stories).unwrap();
        let id = crate::hero_by_name("training_luminary").unwrap();
        let loadout = runtime.hero_stats[usize::from(id.0)].unwrap().moba_loadout;
        assert_eq!((loadout.ranks,loadout.skill_points),([0;5],1));
        assert!(runtime.validate_moba_catalog().unwrap_err().contains("hero loadout differs"));
    }

    #[test]
    fn ability_progression_changes_require_rebuild_not_dev_hot_reload() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content = load_content(root).unwrap();
        let before = ContentShape::from_manifest(&content.manifest, &content.stories);
        let ability = content.manifest.abilities.iter_mut().find(|entry| entry.id == "lumen_lance").unwrap();
        ability.levels[1].required_hero_level = 7;
        let after = ContentShape::from_manifest(&content.manifest, &content.stories);
        assert!(after.ensure_compatible_with(&before).unwrap_err().contains("ability progression changed"));
        let runtime = RuntimeContent::from_manifest(content.manifest, content.stories).unwrap();
        assert!(runtime.validate_moba_catalog().unwrap_err().contains("ability progression differs"));
    }

    #[test]
    fn facing_formation_rows_reach_compiled_runtime_and_reload_guard() {
        let repo=Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut content=load_content(repo.join("scripts/lua_data")).unwrap();
        let before=ContentShape::from_manifest(&content.manifest,&content.stories);
        let entry=content.manifest.abilities.iter_mut().find(|a|a.id=="saika_reinforcements").unwrap();
        assert_eq!(ability_const_from_entry(entry).unwrap().facing_formation_rows,2);
        assert_eq!(crate::ability_const(crate::ABILITY_SAIKA_REINFORCEMENTS).unwrap().facing_formation_rows,2);
        entry.aim_geometry=Some(omoba_content_model::AbilityAimGeometry::FacingFormation{rows:4});
        entry.extras.insert("summon_count".into(),vec![4.,4.,8.,8.]);
        assert_eq!(ability_const_from_entry(entry).unwrap().facing_formation_rows,4);
        assert!(ContentShape::from_manifest(&content.manifest,&content.stories).ensure_compatible_with(&before).unwrap_err().contains("rebuild all peers"));
        let runtime=RuntimeContent::from_manifest(content.manifest,content.stories).unwrap();
        assert!(runtime.validate_moba_catalog().unwrap_err().contains("facing formation"));
    }

    #[test]
    fn executable_cone_ticks_match_compiled_runtime_projection_and_reload_guard() {
        let repo=Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut content=load_content(repo.join("scripts/lua_data")).unwrap();
        let before=ContentShape::from_manifest(&content.manifest,&content.stories);
        let entry=content.manifest.abilities.iter_mut().find(|a|a.id=="ranger_scatter").unwrap();
        assert_eq!(ability_const_from_entry(entry).unwrap().cone_half_angle_ticks,&[341;4]);
        assert_eq!(crate::ability_const(crate::ABILITY_RANGER_SCATTER).unwrap().cone_half_angle_ticks,&[341;4]);
        assert_eq!(crate::ability_const(crate::ABILITY_RANGER_VOLLEY).unwrap().cone_half_angle_ticks,&[0;4]);
        entry.extras.get_mut("half_angle").unwrap()[1]=std::f32::consts::FRAC_PI_2;
        assert_eq!(ability_const_from_entry(entry).unwrap().cone_half_angle_ticks,&[341,1024,341,341]);
        assert!(ContentShape::from_manifest(&content.manifest,&content.stories).ensure_compatible_with(&before).is_err());
        let runtime=RuntimeContent::from_manifest(content.manifest,content.stories).unwrap();
        assert!(runtime.validate_moba_catalog().is_err());
    }

    #[test]
    fn aim_geometry_numeric_and_semantic_changes_require_rebuild_at_both_boundaries() {
        let repo=Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        for (id,field) in [("saika_reinforcements","summon_count"),("saika_reinforcements","row_spacing"),
            ("saika_reinforcements","col_spacing"),("saika_reinforcements","front_row_distance"),
            ("saika_reinforcements","range"),("fire_dash","dash_width"),("flame_blade","swipe_radius"),("flame_assault","radius")] {
            let mut content=load_content(repo.join("scripts/lua_data")).unwrap();let before=ContentShape::from_manifest(&content.manifest,&content.stories);
            let ability=content.manifest.abilities.iter_mut().find(|a|a.id==id).unwrap();
            if field=="range" {ability.levels[0].range+=1.;}else{ability.extras.get_mut(field).unwrap()[0]+=if field=="summon_count"{2.}else{1.};}
            assert!(ContentShape::from_manifest(&content.manifest,&content.stories).ensure_compatible_with(&before).unwrap_err().contains("aim preview"),"{id} {field}");
            let runtime=RuntimeContent::from_manifest(content.manifest,content.stories).unwrap();
            assert!(runtime.validate_moba_catalog().unwrap_err().contains("aim preview"),"{id} {field}");
        }
    }

    #[test]
    fn rejected_real_geometry_reload_preserves_snapshot_then_nonspatial_reload_succeeds() {
        fn copy_tree(source:&Path,dest:&Path) {
            fs::create_dir_all(dest).unwrap();for entry in fs::read_dir(source).unwrap(){let entry=entry.unwrap();
                if entry.file_type().unwrap().is_dir(){copy_tree(&entry.path(),&dest.join(entry.file_name()));}
                else{fs::copy(entry.path(),dest.join(entry.file_name())).unwrap();}}
        }
        let _guard=test_lock();let root=temp_root("aim_preview_reload");
        copy_tree(&Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data"),&root);
        let entry=root.join("templates.lua");let original=fs::read_to_string(&entry).unwrap();
        enable_runtime_content(&root);reset_store();ensure_loaded().unwrap().unwrap();
        let before=runtime_lua_content_info().unwrap().unwrap();
        let id=crate::ABILITY_FLAME_BLADE;
        let damage=active_ability_const(id).unwrap().extras.iter().find(|(key,_)|*key=="damage").unwrap().1[0];
        let wrapper=|mutation:&str|format!("local original=(function()\n{}\nend)()\nreturn function(ctx) local data=original(ctx) {} return data end\n",original,mutation);
        write(&entry,&wrapper("for _,a in ipairs(data.abilities) do if a.id=='saika_reinforcements' then a.extras.col_spacing[1]=a.extras.col_spacing[1]+1 end end"));
        assert!(reload_runtime_lua_content_dev(None).unwrap_err().contains("aim preview"));
        let after=runtime_lua_content_info().unwrap().unwrap();assert_eq!(after.generation,before.generation);assert_eq!(after.hash,before.hash);
        assert_eq!(active_ability_const(crate::ABILITY_SAIKA_REINFORCEMENTS).unwrap().extras.iter().find(|(key,_)|*key=="col_spacing").unwrap().1[0],Fixed64::from_i32(60));
        write(&entry,&wrapper("for _,a in ipairs(data.abilities) do if a.id=='flame_blade' then a.extras.damage[1]=a.extras.damage[1]+1 end end"));
        let accepted=reload_runtime_lua_content_dev(None).unwrap().unwrap();assert_eq!(accepted.generation,before.generation+1);assert_ne!(accepted.hash,before.hash);
        assert_eq!(active_ability_const(id).unwrap().extras.iter().find(|(key,_)|*key=="damage").unwrap().1[0],damage+Fixed64::ONE);
        cleanup(root);
    }

    #[test]
    fn permanent_objective_rewards_are_hashed_and_require_peer_rebuild() {
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content=load_content(root).unwrap();let before=ContentShape::from_manifest(&content.manifest,&content.stories);
        let hash=omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        let camps=content.manifest_value["moba_maps"][0]["jungle_camps"].as_array_mut().unwrap();
        camps.iter_mut().find(|c|c["additional_objective_buffs"][0].is_object()).unwrap()["additional_objective_buffs"][0]["attack_bonus"]=serde_json::json!(6);
        assert_ne!(hash,omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_maps[0].jungle_camps.iter_mut().find(|c|!c.additional_objective_buffs.is_empty()).unwrap().additional_objective_buffs[0].attack_bonus=6;
        assert!(ContentShape::from_manifest(&content.manifest,&content.stories).ensure_compatible_with(&before).unwrap_err().contains("MOBA map catalog changed"));
        assert!(RuntimeContent::from_manifest(content.manifest,content.stories).unwrap().validate_moba_catalog().unwrap_err().contains("MOBA map catalog differs"));
    }
    #[test]
    fn personal_buff_transfer_policy_is_hashed_and_requires_peer_rebuild() {
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content=load_content(root).unwrap();let before=ContentShape::from_manifest(&content.manifest,&content.stories);
        let hash=omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        let camps=content.manifest_value["moba_maps"][0]["jungle_camps"].as_array_mut().unwrap();
        camps.iter_mut().find(|c|c["personal_buff"].is_object()).unwrap()["personal_buff"]["transfer_on_kill"]=serde_json::json!(false);
        assert_ne!(hash,omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_maps[0].jungle_camps.iter_mut().find_map(|c|c.personal_buff.as_mut()).unwrap().transfer_on_kill=false;
        assert!(ContentShape::from_manifest(&content.manifest,&content.stories).ensure_compatible_with(&before).unwrap_err().contains("MOBA map catalog changed"));
        assert!(RuntimeContent::from_manifest(content.manifest,content.stories).unwrap().validate_moba_catalog().unwrap_err().contains("MOBA map catalog differs"));
    }
    #[test]
    fn objective_stack_policy_is_hashed_and_requires_peer_rebuild() {
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content=load_content(root).unwrap();
        let before=ContentShape::from_manifest(&content.manifest,&content.stories);
        let hash=omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        let camps=content.manifest_value["moba_maps"][0]["jungle_camps"].as_array_mut().unwrap();
        let rule=camps.iter_mut().find(|c|c["objective_buff"].is_object()).unwrap();
        rule["objective_buff"]["max_stacks"]=serde_json::json!(2);
        assert_ne!(hash,omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_maps[0].jungle_camps.iter_mut().find_map(|c|c.objective_buff.as_mut()).unwrap().max_stacks=2;
        assert!(ContentShape::from_manifest(&content.manifest,&content.stories).ensure_compatible_with(&before)
            .unwrap_err().contains("MOBA map catalog changed"));
        assert!(RuntimeContent::from_manifest(content.manifest,content.stories).unwrap().validate_moba_catalog()
            .unwrap_err().contains("MOBA map catalog differs"));
    }
    #[test]
    fn moba_map_changes_require_rebuild_not_hot_reload() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content = load_content(root).unwrap();
        let before = ContentShape::from_manifest(&content.manifest,&content.stories);
        let hash = omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        content.manifest_value["moba_maps"][0]["lanes"][0]["waypoints"][1][1] = serde_json::json!(1500);
        assert_ne!(hash,omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_maps[0].lanes[0].waypoints[1][1] = 1500;
        let after = ContentShape::from_manifest(&content.manifest,&content.stories);
        assert!(after.ensure_compatible_with(&before).unwrap_err().contains("MOBA map catalog changed"));
        let runtime = RuntimeContent::from_manifest(content.manifest,content.stories).unwrap();
        assert!(runtime.validate_moba_catalog().unwrap_err().contains("MOBA map catalog differs"));
    }

    #[test]
    fn moba_income_changes_require_rebuild_not_dev_hot_reload() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content = load_content(root).unwrap();
        let before = ContentShape::from_manifest(&content.manifest, &content.stories);
        let hash = omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        content.manifest_value["moba_economy"]["passive_gold_per_second"] = serde_json::json!(3);
        assert_ne!(hash, omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_economy.passive_gold_per_second += 1;
        let after = ContentShape::from_manifest(&content.manifest, &content.stories);
        assert!(after.ensure_compatible_with(&before).unwrap_err().contains("MOBA income rules changed"));
        let runtime = RuntimeContent::from_manifest(content.manifest, content.stories).unwrap();
        assert!(runtime.validate_moba_catalog().unwrap_err().contains("MOBA income rules differ"));
        for invalid in [serde_json::json!(-1), serde_json::json!(0.5), serde_json::json!("2")] {
            assert!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(
                serde_json::json!({"passive_gold_per_second": invalid})).is_err());
        }
    }

    #[test]
    fn death_streak_curve_is_validated_hashed_and_requires_peer_rebuild() {
        use crate::lua_content::validate_death_streak;
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content=load_content(root).unwrap();
        let before=ContentShape::from_manifest(&content.manifest,&content.stories);
        let hash=omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        content.manifest_value["moba_economy"]["death_streak_basis_points"][2]=serde_json::json!(9166);
        assert_ne!(hash,omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_economy.death_streak_basis_points[2]-=1;
        assert!(ContentShape::from_manifest(&content.manifest,&content.stories).ensure_compatible_with(&before)
            .unwrap_err().contains("death streak curve changed"));
        assert!(RuntimeContent::from_manifest(content.manifest,content.stories).unwrap().validate_moba_catalog()
            .unwrap_err().contains("death streak curve differs"));
        assert!(validate_death_streak(&[]).is_ok());assert!(validate_death_streak(&[10000,0]).is_ok());
        for invalid in [vec![9999],vec![10000,10001],vec![10000,0,1],vec![10000;102]] {
            assert!(validate_death_streak(&invalid).is_err());
        }
        let mut legacy=serde_json::to_value(crate::lua_content::MobaEconomyRules::default()).unwrap();
        legacy.as_object_mut().unwrap().remove("death_streak_basis_points");
        assert!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(legacy.clone()).unwrap().death_streak_basis_points.is_empty());
        for invalid in [serde_json::json!([-1]),serde_json::json!([0.5]),serde_json::json!(["10000"])] {
            legacy["death_streak_basis_points"]=invalid;
            assert!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(legacy.clone()).is_err());
        }
    }
    #[test]
    fn first_kill_bonus_is_hashed_requires_rebuild_and_legacy_defaults_disabled() {
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content=load_content(root).unwrap();let before=ContentShape::from_manifest(&content.manifest,&content.stories);
        let hash=omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        content.manifest_value["moba_economy"]["first_kill_bonus_gold"]=serde_json::json!(101);
        assert_ne!(hash,omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_economy.first_kill_bonus_gold+=1;
        assert!(ContentShape::from_manifest(&content.manifest,&content.stories).ensure_compatible_with(&before).unwrap_err().contains("first kill bonus changed"));
        assert!(RuntimeContent::from_manifest(content.manifest,content.stories).unwrap().validate_moba_catalog().unwrap_err().contains("first kill bonus differs"));
        let mut rules=serde_json::to_value(crate::lua_content::MobaEconomyRules::default()).unwrap();
        rules.as_object_mut().unwrap().remove("first_kill_bonus_gold");
        assert_eq!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(rules.clone()).unwrap().first_kill_bonus_gold,0);
        for invalid in [serde_json::json!(-1),serde_json::json!(0.5),serde_json::json!("100")]{
            rules["first_kill_bonus_gold"]=invalid;assert!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(rules.clone()).is_err());
        }
    }
    #[test]
    fn objective_comeback_content_bounds_hash_and_rebuild_gates_are_shared() {
        use omoba_content_model::objective_comeback::ObjectiveComebackRules;
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content=load_content(root).unwrap();let good=content.manifest.moba_economy.objective_comeback;
        assert!(good.is_valid());assert!(ObjectiveComebackRules::default().is_valid());
        for bad in [ObjectiveComebackRules {starts_seconds:0,..good},ObjectiveComebackRules {starts_seconds:3601,..good},
            ObjectiveComebackRules {activate_gap:0,..good},ObjectiveComebackRules {release_gap:good.activate_gap,..good},
            ObjectiveComebackRules {activation_pool:0,..good},ObjectiveComebackRules {activation_pool:1_000_001,..good},
            ObjectiveComebackRules {tower_gold_per_player:1_000_001,..good},
            ObjectiveComebackRules {epic_gold_per_player:0,tower_gold_per_player:0,..good}] {assert!(!bad.is_valid());}
        for invalid in [serde_json::json!(-1),serde_json::json!(0.5),serde_json::json!("500")] {
            let mut row=serde_json::to_value(good).unwrap();row["activate_gap"]=invalid;
            assert!(serde_json::from_value::<ObjectiveComebackRules>(row).is_err());
        }
        let mut row=serde_json::to_value(good).unwrap();row["unknown"]=serde_json::json!(1);
        assert!(serde_json::from_value::<ObjectiveComebackRules>(row).is_err());
        let mut legacy=serde_json::to_value(crate::lua_content::MobaEconomyRules::default()).unwrap();
        legacy.as_object_mut().unwrap().remove("objective_comeback");
        assert_eq!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(legacy).unwrap().objective_comeback,Default::default());
        let before=ContentShape::from_manifest(&content.manifest,&content.stories);
        let hash=omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        content.manifest_value["moba_economy"]["objective_comeback"]["activation_pool"]=serde_json::json!(1001);
        assert_ne!(hash,omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_economy.objective_comeback.activation_pool+=1;
        assert!(ContentShape::from_manifest(&content.manifest,&content.stories).ensure_compatible_with(&before).unwrap_err().contains("objective comeback changed"));
        assert!(RuntimeContent::from_manifest(content.manifest,content.stories).unwrap().validate_moba_catalog().unwrap_err().contains("objective comeback differs"));
    }
    #[test]
    fn kill_streak_bounty_is_validated_hashed_and_requires_all_peer_rebuild() {
        use crate::lua_content::{KillStreakBountyRules,validate_kill_streak_bounty};
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content=load_content(root).unwrap();
        let before=ContentShape::from_manifest(&content.manifest,&content.stories);
        let hash=omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        content.manifest_value["moba_economy"]["kill_streak_bounty"]["max_gold"]=serde_json::json!(701);
        assert_ne!(hash,omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_economy.kill_streak_bounty.max_gold+=1;
        let after=ContentShape::from_manifest(&content.manifest,&content.stories);
        assert!(after.ensure_compatible_with(&before).unwrap_err().contains("kill streak bounty changed"));
        let runtime=RuntimeContent::from_manifest(content.manifest,content.stories).unwrap();
        assert!(runtime.validate_moba_catalog().unwrap_err().contains("kill streak bounty differs"));
        let good=KillStreakBountyRules {threshold:3,initial_gold:150,per_kill_gold:100,max_gold:700};
        assert!(validate_kill_streak_bounty(good).is_ok());assert!(validate_kill_streak_bounty(Default::default()).is_ok());
        for bad in [KillStreakBountyRules {threshold:0,..good},KillStreakBountyRules {threshold:101,..good},
            KillStreakBountyRules {initial_gold:0,..good},KillStreakBountyRules {initial_gold:701,..good},
            KillStreakBountyRules {per_kill_gold:1_000_001,..good},KillStreakBountyRules {max_gold:1_000_001,..good}]{
            assert!(validate_kill_streak_bounty(bad).is_err());
        }
        for invalid in [serde_json::json!(-1),serde_json::json!(0.5),serde_json::json!("3")] {
            let mut rules=serde_json::to_value(good).unwrap();rules["threshold"]=invalid;
            assert!(serde_json::from_value::<KillStreakBountyRules>(rules).is_err());
        }
        let mut legacy=serde_json::to_value(crate::lua_content::MobaEconomyRules::default()).unwrap();
        legacy.as_object_mut().unwrap().remove("kill_streak_bounty");
        assert_eq!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(legacy).unwrap().kill_streak_bounty,Default::default());
    }

    #[test]
    fn moba_kill_gold_changes_require_rebuild_not_dev_hot_reload() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content = load_content(root).unwrap();
        let before = ContentShape::from_manifest(&content.manifest, &content.stories);
        let hash = omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        content.manifest_value["moba_economy"]["hero_kill_gold"] = serde_json::json!(301);
        assert_ne!(hash, omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_economy.hero_kill_gold += 1;
        let after = ContentShape::from_manifest(&content.manifest, &content.stories);
        assert!(after.ensure_compatible_with(&before).unwrap_err().contains("MOBA kill rules changed"));
        let runtime = RuntimeContent::from_manifest(content.manifest, content.stories).unwrap();
        assert!(runtime.validate_moba_catalog().unwrap_err().contains("MOBA kill rules differ"));
        for invalid in [serde_json::json!(-1), serde_json::json!(0.5), serde_json::json!("300")] {
            assert!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(
                serde_json::json!({"passive_gold_per_second": 2, "hero_kill_gold": invalid})).is_err());
        }
    }

    #[test]
    fn moba_recall_changes_require_rebuild_not_dev_hot_reload() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content = load_content(root).unwrap();
        let before = ContentShape::from_manifest(&content.manifest, &content.stories);
        let hash = omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        content.manifest_value["moba_economy"]["recall_channel_seconds"] = serde_json::json!(9);
        assert_ne!(hash, omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_economy.recall_channel_seconds += 1;
        let after = ContentShape::from_manifest(&content.manifest, &content.stories);
        assert!(after.ensure_compatible_with(&before).unwrap_err().contains("MOBA recall rules changed"));
        let runtime = RuntimeContent::from_manifest(content.manifest, content.stories).unwrap();
        assert!(runtime.validate_moba_catalog().unwrap_err().contains("MOBA recall rules differ"));
    }

    #[test]
    fn moba_base_recovery_requires_rebuild_not_hot_reload() {
        for rate in [true,false] {
            let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
            let mut content=load_content(root).unwrap();
            let before=ContentShape::from_manifest(&content.manifest,&content.stories);
            let hash=omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
            let field=if rate {"base_recovery_hp_per_second"} else {"base_recovery_radius"};
            content.manifest_value["moba_economy"][field]=serde_json::json!(121);
            assert_ne!(hash,omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
            if rate {content.manifest.moba_economy.base_recovery_hp_per_second+=1;}
            else {content.manifest.moba_economy.base_recovery_radius+=1;}
            let after=ContentShape::from_manifest(&content.manifest,&content.stories);
            assert!(after.ensure_compatible_with(&before).unwrap_err().contains("base recovery rules changed"));
            let runtime=RuntimeContent::from_manifest(content.manifest,content.stories).unwrap();
            assert!(runtime.validate_moba_catalog().unwrap_err().contains("base recovery rules differ"));
        }
    }

    #[test]
    fn mana_sustain_base_rate_requires_rebuild_and_checked_legacy_type() {
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content=load_content(root).unwrap();
        let before=ContentShape::from_manifest(&content.manifest,&content.stories);
        let hash=omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        content.manifest_value["moba_economy"]["base_recovery_mana_per_second"]=serde_json::json!(61);
        assert_ne!(hash,omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_economy.base_recovery_mana_per_second+=1;
        assert!(ContentShape::from_manifest(&content.manifest,&content.stories)
            .ensure_compatible_with(&before).unwrap_err().contains("base recovery rules changed"));
        assert!(RuntimeContent::from_manifest(content.manifest,content.stories).unwrap()
            .validate_moba_catalog().unwrap_err().contains("base recovery rules differ"));
        let mut rules=serde_json::to_value(crate::lua_content::MobaEconomyRules::default()).unwrap();
        rules.as_object_mut().unwrap().remove("base_recovery_mana_per_second");
        assert_eq!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(rules.clone()).unwrap().base_recovery_mana_per_second,0);
        for invalid in [serde_json::json!(-1),serde_json::json!(0.5),serde_json::json!("60")] {
            rules["base_recovery_mana_per_second"]=invalid;
            assert!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(rules.clone()).is_err());
        }
    }

    #[test]
    fn mana_lifecycle_regeneration_requires_rebuild_not_hot_reload() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content = load_content(root).unwrap();
        let before = ContentShape::from_manifest(&content.manifest, &content.stories);
        let hash = omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        content.manifest_value["moba_economy"]["mana_regen_per_second"] = serde_json::json!(6);
        assert_ne!(hash, omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_economy.mana_regen_per_second += 1;
        let after = ContentShape::from_manifest(&content.manifest, &content.stories);
        assert!(after.ensure_compatible_with(&before).unwrap_err().contains("mana regeneration rules changed"));
        let runtime = RuntimeContent::from_manifest(content.manifest, content.stories).unwrap();
        assert!(runtime.validate_moba_catalog().unwrap_err().contains("mana regeneration rules differ"));
    }

    #[test]
    fn mana_lifecycle_regeneration_types_and_legacy_omission_are_checked() {
        let mut rules = serde_json::to_value(crate::lua_content::MobaEconomyRules::default()).unwrap();
        rules.as_object_mut().unwrap().remove("mana_regen_per_second");
        let legacy: crate::lua_content::MobaEconomyRules = serde_json::from_value(rules.clone()).unwrap();
        assert_eq!(legacy.mana_regen_per_second, 0);
        for invalid in [serde_json::json!(-1), serde_json::json!(0.5), serde_json::json!("5")] {
            rules["mana_regen_per_second"] = invalid;
            assert!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(rules.clone()).is_err());
        }
    }

    #[test]
    fn moba_assist_rules_require_rebuild_and_validate_types() {
        for window in [false, true] {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
            let mut content = load_content(root).unwrap();
            let before = ContentShape::from_manifest(&content.manifest, &content.stories);
            let hash = omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
            let field = if window { "assist_window_seconds" } else { "hero_assist_gold" };
            content.manifest_value["moba_economy"][field] = serde_json::json!(if window { 11 } else { 101 });
            assert_ne!(hash, omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
            if window { content.manifest.moba_economy.assist_window_seconds += 1; }
            else { content.manifest.moba_economy.hero_assist_gold += 1; }
            let after = ContentShape::from_manifest(&content.manifest, &content.stories);
            assert!(after.ensure_compatible_with(&before).unwrap_err().contains("MOBA assist rules changed"));
            let runtime = RuntimeContent::from_manifest(content.manifest, content.stories).unwrap();
            assert!(runtime.validate_moba_catalog().unwrap_err().contains("MOBA assist rules differ"));
            for invalid in [serde_json::json!(-1), serde_json::json!(0.5), serde_json::json!("10")] {
                let mut rules = serde_json::json!({"passive_gold_per_second":2,"hero_kill_gold":300,
                    "recall_channel_seconds":8,"hero_assist_gold":100,"assist_window_seconds":10});
                rules[field] = invalid;
                assert!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(rules).is_err());
            }
        }
    }

    #[test]
    fn moba_xp_changes_require_rebuild_and_validate_complete_rules() {
        for assist in [false, true] {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
            let mut content = load_content(root).unwrap();
            let before = ContentShape::from_manifest(&content.manifest, &content.stories);
            let hash = omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
            let field = if assist {"hero_assist_xp"} else {"hero_kill_xp"};
            content.manifest_value["moba_economy"][field] = serde_json::json!(101);
            assert_ne!(hash, omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
            if assist { content.manifest.moba_economy.hero_assist_xp += 1; }
            else { content.manifest.moba_economy.hero_kill_xp += 1; }
            let after = ContentShape::from_manifest(&content.manifest, &content.stories);
            assert!(after.ensure_compatible_with(&before).unwrap_err().contains("MOBA XP rules changed"));
            let runtime = RuntimeContent::from_manifest(content.manifest, content.stories).unwrap();
            assert!(runtime.validate_moba_catalog().unwrap_err().contains("MOBA XP rules differ"));
            for invalid in [serde_json::json!(-1),serde_json::json!(0.5),serde_json::json!("100")] {
                let mut rules = serde_json::to_value(crate::lua_content::MobaEconomyRules::default()).unwrap();
                rules[field] = invalid;
                assert!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(rules).is_err());
            }
        }
    }

    #[test]
    fn moba_lane_xp_rules_change_hash_reject_hot_reload_and_invalid_types() {
        for field in ["lane_creep_xp","lane_xp_radius"] {
            let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
            let mut content=load_content(root).unwrap();
            let before=ContentShape::from_manifest(&content.manifest,&content.stories);
            let hash=omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
            content.manifest_value["moba_economy"][field]=serde_json::json!(101);
            assert_ne!(hash,omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
            if field=="lane_creep_xp" {content.manifest.moba_economy.lane_creep_xp+=1;}
            else {content.manifest.moba_economy.lane_xp_radius+=1;}
            let after=ContentShape::from_manifest(&content.manifest,&content.stories);
            assert!(after.ensure_compatible_with(&before).unwrap_err().contains("MOBA lane XP rules changed"));
            let runtime=RuntimeContent::from_manifest(content.manifest,content.stories).unwrap();
            assert!(runtime.validate_moba_catalog().unwrap_err().contains("MOBA lane XP rules differ"));
            for invalid in [serde_json::json!(-1),serde_json::json!(0.5),serde_json::json!("25")] {
                let mut rules=serde_json::to_value(crate::lua_content::MobaEconomyRules::default()).unwrap();
                rules[field]=invalid;
                assert!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(rules).is_err());
            }
            let mut missing=serde_json::to_value(crate::lua_content::MobaEconomyRules::default()).unwrap();
            missing.as_object_mut().unwrap().remove(field);
            assert!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(missing).is_err());
        }
    }

    #[test]
    fn moba_lane_gold_rules_require_rebuild_and_preserve_legacy_zero() {
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut content=load_content(root).unwrap();
        assert_eq!(content.manifest.moba_economy.lane_creep_gold,crate::MOBA_LANE_CREEP_GOLD);
        let before=ContentShape::from_manifest(&content.manifest,&content.stories);
        let hash=omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
        content.manifest_value["moba_economy"]["lane_creep_gold"]=serde_json::json!(21);
        assert_ne!(hash,omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap());
        content.manifest.moba_economy.lane_creep_gold+=1;
        let after=ContentShape::from_manifest(&content.manifest,&content.stories);
        assert!(after.ensure_compatible_with(&before).unwrap_err().contains("MOBA lane gold rules changed"));
        let runtime=RuntimeContent::from_manifest(content.manifest,content.stories).unwrap();
        assert!(runtime.validate_moba_catalog().unwrap_err().contains("MOBA lane gold rules differ"));
        for invalid in [serde_json::json!(-1),serde_json::json!(0.5),serde_json::json!("20")] {
            let mut rules=serde_json::to_value(crate::lua_content::MobaEconomyRules::default()).unwrap();
            rules["lane_creep_gold"]=invalid;
            assert!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(rules).is_err());
        }
        let mut legacy=serde_json::to_value(crate::lua_content::MobaEconomyRules::default()).unwrap();
        legacy.as_object_mut().unwrap().remove("lane_creep_gold");
        assert_eq!(serde_json::from_value::<crate::lua_content::MobaEconomyRules>(legacy).unwrap().lane_creep_gold,0);
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
                "return function(ctx) return {{ creeps = {{ {{ id = '{}', hp = {}, move_speed = 100 }} }}, {} }} end\n",
                creep_id, hp, crate::lua_content::FIXTURE_REQUIRED_MOBA_CATALOGS
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
                "return function(ctx) return {{ creeps = {{ {{ id = '{}' }}, {{ id = '{}' }} }}, {} }} end\n",
                creep_id, second_creep, crate::lua_content::FIXTURE_REQUIRED_MOBA_CATALOGS
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

    #[test]
    fn trinket_runtime_range_charges_and_policy_changes_require_peer_rebuild() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut cases: Vec<(&str, Box<dyn Fn(&mut MobaTrinketEntry)>)> = Vec::new();
        cases.push(("placement_range", Box::new(|entry: &mut MobaTrinketEntry| entry.placement_range = 601.0)));
        cases.push(("initial_charges", Box::new(|entry: &mut MobaTrinketEntry| entry.initial_charges = 2)));
        cases.push(("automatic_acquisition", Box::new(|entry: &mut MobaTrinketEntry| {
            entry.automatic_acquisition = omoba_content_model::moba_trinkets::MobaTrinketAutomaticAcquisition::Enabled;
        })));
        cases.push(("direct_attack", Box::new(|entry: &mut MobaTrinketEntry| {
            entry.direct_attack = omoba_content_model::moba_trinkets::MobaTrinketDirectAttack::NotDirectlyAttackable;
        })));
        for (field, mutate_entry) in cases {
            let mut content = load_content(root.clone()).unwrap();
            let before = ContentShape::from_manifest(&content.manifest, &content.stories);
            let hash = omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap();
            mutate_entry(&mut content.manifest.moba_trinkets[0]);
            content.manifest_value["moba_trinkets"][0] = serde_json::to_value(&content.manifest.moba_trinkets[0]).unwrap();
            assert_ne!(hash, omoba_content_model::canonical_template_hash(&content.manifest_value).unwrap(), "{field}");
            let after = ContentShape::from_manifest(&content.manifest, &content.stories);
            let shape_error = after.ensure_compatible_with(&before).unwrap_err();
            assert!(shape_error.contains("MOBA trinket catalog changed"), "{field}: {shape_error}");
            assert!(shape_error.contains("rebuild all peers"), "{field}: {shape_error}");
            let runtime = RuntimeContent::from_manifest(content.manifest, content.stories).unwrap();
            let catalog_error = runtime.validate_moba_catalog().unwrap_err();
            assert!(catalog_error.contains("rebuild all peers"), "{field}: {catalog_error}");
            assert!(!runtime.trinkets_match_compiled(), "{field}");
        }
    }
    #[test]
    fn damage_interrupt_policy_is_strict_ranked_hashed_and_requires_rebuild() {
        let make=|level:serde_json::Value|serde_json::json!({"id":"fixture","max_level":1,"levels":[level]});
        let omitted:omoba_content_model::AbilityDefinition=serde_json::from_value(make(serde_json::json!({}))).unwrap();
        assert!(!omitted.levels[0].cast_interrupt_on_damage);
        let fixture=temp_root("damage_interrupt_policy");
        let repo=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_owned();
        write(&fixture.join("templates/heroes.lua"),"return function(ctx) return {} end");
        for (lua,json,valid) in [
            ("cast_time=0.5,cast_interrupt_on_damage=true",serde_json::json!({"cast_time":0.5,"cast_interrupt_on_damage":true}),true),
            ("cast_interrupt_on_damage=false",serde_json::json!({"cast_interrupt_on_damage":false}),true),
            ("cast_time=0.5,cast_interrupt_on_damage=1",serde_json::json!({"cast_time":0.5,"cast_interrupt_on_damage":1}),false),
            ("cast_time=0.5,cast_interrupt_on_damage='true'",serde_json::json!({"cast_time":0.5,"cast_interrupt_on_damage":"true"}),false),
            ("cast_time=0,cast_interrupt_on_damage=true",serde_json::json!({"cast_time":0,"cast_interrupt_on_damage":true}),false),
            ("cast_time=61,cast_interrupt_on_damage=true",serde_json::json!({"cast_time":61,"cast_interrupt_on_damage":true}),false),
        ] {
            let result=serde_json::from_value::<omoba_content_model::AbilityDefinition>(make(json))
                .map_err(|e|e.to_string()).and_then(|a|omoba_content_model::validate_ability_progression(&a));
            assert_eq!(result.is_ok(),valid,"{lua}");
            write(&fixture.join("templates/abilities.lua"),&["return function(ctx) return {{id='fixture',max_level=1,levels={{",lua,"}}}} end"].concat());
            let output=std::process::Command::new(repo.join("tools/lua/lua.exe")).arg(repo.join("scripts/gen_hero_registry.lua"))
                .arg(&fixture).output().unwrap();assert_eq!(output.status.success(),valid,"{lua}: {}",String::from_utf8_lossy(&output.stderr));
        }
        let mut content=load_content(repo.join("scripts/lua_data")).unwrap();let before=ContentShape::from_manifest(&content.manifest,&content.stories);
        let hash=content_hash(&content.manifest,&content.stories).unwrap();
        let ability=content.manifest.abilities.iter_mut().find(|v|v.id=="saika_reinforcements").unwrap();
        assert!(ability.levels.iter().all(|v|v.cast_interrupt_on_damage));ability.levels[1].cast_interrupt_on_damage=false;
        assert_ne!(hash,content_hash(&content.manifest,&content.stories).unwrap());
        assert!(ContentShape::from_manifest(&content.manifest,&content.stories).ensure_compatible_with(&before).unwrap_err().contains("rebuild all peers"));
        let runtime=RuntimeContent::from_manifest(content.manifest,content.stories).unwrap();assert!(runtime.validate_moba_catalog().unwrap_err().contains("ability progression differs"));
    }
}
