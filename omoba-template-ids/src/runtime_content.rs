use crate::lua_content::{
    load_content, AbilityEntry, AttackTimingEntry, CreepEntry, HeroEntry, HeroRenderEntry,
    Manifest, StoryBundle, SummonEntry, TowerEntry, UpgradeEffectEntry,
};
use crate::{
    ability_by_name, ability_id_str, AbilityConst, AbilityId, AbilityLevelDataConst,
    AbilityTypeC, AttackTimingConst, CastTypeC, CreepId, CreepStats, Fixed64, GeneratedStory,
    HeroAnimationBindingConst, HeroAnimationSourceConst, HeroId, HeroRenderMetadataConst,
    HeroRenderModeC, HeroStats, LevelGrowth, StatOpC, StoryValue, SummonId, SummonStats,
    TargetTypeC, TowerBarrelLayoutC, TowerBarrelVariantConst, TowerId,
    TowerRecoilConst, TowerRecoilModeC, TowerRenderAnimationConst, TowerRenderMetadataConst,
    TowerRenderModeC, TowerRenderPointConst, TowerRotationModeC, TowerStats, UpgradeDefConst,
    UpgradeEffectConst, UpgradeEffectKindC, creep_id_str, hero_id_str, summon_id_str,
    tower_id_str,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

static CONTENT: OnceLock<Result<Option<RuntimeContent>, String>> = OnceLock::new();

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
    story_ids: &'static [&'static str],
    stories: HashMap<&'static str, &'static GeneratedStory>,
}

pub fn ensure_loaded() -> Result<Option<&'static RuntimeContent>, String> {
    match CONTENT.get_or_init(load_from_env) {
        Ok(Some(content)) => Ok(Some(content)),
        Ok(None) => Ok(None),
        Err(err) => Err(err.clone()),
    }
}

fn active_content() -> Option<&'static RuntimeContent> {
    match ensure_loaded() {
        Ok(content) => content,
        Err(err) => panic!("runtime Lua content load failed: {}", err),
    }
}

fn load_from_env() -> Result<Option<RuntimeContent>, String> {
    if !env_truthy("OMB_LUA_CONTENT") {
        return Ok(None);
    }
    let root = content_root();
    let content = load_content(root.clone())?;
    RuntimeContent::from_manifest(content.manifest, content.stories)
        .map(Some)
        .map_err(|err| format!("{} (root={})", err, root.display()))
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
                    format!("hero '{}' references unknown ability '{}'", entry.id, ability)
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
            story_ids: leak_slice(story_id_vec),
            stories: story_map,
        })
    }
}

trait RuntimeEntry {
    fn tombstone(&self) -> bool;
}

impl RuntimeEntry for TowerEntry {
    fn tombstone(&self) -> bool {
        self.tombstone
    }
}

impl RuntimeEntry for HeroEntry {
    fn tombstone(&self) -> bool {
        self.tombstone
    }
}

impl RuntimeEntry for CreepEntry {
    fn tombstone(&self) -> bool {
        self.tombstone
    }
}

impl RuntimeEntry for SummonEntry {
    fn tombstone(&self) -> bool {
        self.tombstone
    }
}

impl RuntimeEntry for AbilityEntry {
    fn tombstone(&self) -> bool {
        self.tombstone
    }
}

fn build_indexed<T, U, F>(entries: &[T], kind: &str, mut convert: F) -> Result<Vec<Option<U>>, String>
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

fn build_tower_upgrades(entry: &TowerEntry) -> Result<&'static [&'static [UpgradeDefConst]], String> {
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
            });
        }
        paths.push(leak_slice(defs));
    }
    Ok(leak_slice(paths))
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
        JsonValue::Array(values) => StoryValue::Array(leak_slice(values.into_iter().map(story_value).collect())),
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

pub fn story_by_name(name: &str) -> Option<&'static GeneratedStory> {
    active_content().and_then(|content| content.stories.get(name).copied())
}

pub fn story_ids() -> Option<&'static [&'static str]> {
    active_content().map(|content| content.story_ids)
}
