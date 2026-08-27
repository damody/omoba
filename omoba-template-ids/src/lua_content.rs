#![allow(dead_code)]

use mlua::{Lua, LuaSerdeExt, Value as LuaValue};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct LuaContent {
    pub(crate) manifest: Manifest,
    pub(crate) stories: Vec<StoryBundle>,
    pub(crate) read_files: Vec<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Manifest {
    #[serde(default)]
    pub(crate) towers: Vec<TowerEntry>,
    #[serde(default)]
    pub(crate) heroes: Vec<HeroEntry>,
    #[serde(default)]
    pub(crate) abilities: Vec<AbilityEntry>,
    #[serde(default)]
    pub(crate) buffs: Vec<Entry>,
    #[serde(default)]
    pub(crate) summons: Vec<SummonEntry>,
    #[serde(default)]
    pub(crate) creeps: Vec<CreepEntry>,
    #[serde(default)]
    pub(crate) projectile_kinds: Vec<ProjKind>,
    #[serde(default)]
    pub(crate) td_layers: Vec<TdLayerEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct TdLayerEntry {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) label: String,
    pub(crate) hp: u32,
    pub(crate) move_speed: u32,
    #[serde(default)]
    pub(crate) children: Vec<String>,
    pub(crate) cash: u32,
    pub(crate) leak_value: u32,
    #[serde(default)]
    pub(crate) properties: u32,
    pub(crate) accepted_damage: u32,
    #[serde(default = "default_true")]
    pub(crate) regrow_eligible: bool,
    #[serde(default = "default_true")]
    pub(crate) fortified_eligible: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Entry {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) tombstone: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct TowerEntry {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) tombstone: bool,
    #[serde(default)]
    pub(crate) atk: f32,
    #[serde(default)]
    pub(crate) asd_interval: f32,
    #[serde(default)]
    pub(crate) range: f32,
    #[serde(default)]
    pub(crate) bullet_speed: f32,
    #[serde(default)]
    pub(crate) splash_radius: f32,
    #[serde(default)]
    pub(crate) hit_radius: f32,
    #[serde(default)]
    pub(crate) slow_factor: f32,
    #[serde(default)]
    pub(crate) slow_duration: f32,
    #[serde(default)]
    pub(crate) cost: i32,
    #[serde(default)]
    pub(crate) footprint: f32,
    #[serde(default)]
    pub(crate) placement_radius: f32,
    #[serde(default)]
    pub(crate) hp: f32,
    #[serde(default)]
    pub(crate) turn_speed_deg: f32,
    #[serde(default)]
    pub(crate) render: TowerRenderEntry,
    #[serde(default = "default_attack_timing")]
    pub(crate) attack_timing: AttackTimingEntry,
    #[serde(default)]
    pub(crate) upgrades: Vec<Vec<UpgradeEntry>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub(crate) struct TowerRenderEntry {
    #[serde(default)]
    pub(crate) render_mode: String,
    #[serde(default)]
    pub(crate) base: String,
    #[serde(default)]
    pub(crate) barrel: String,
    #[serde(default)]
    pub(crate) visual_size: f32,
    #[serde(default)]
    pub(crate) barrel_frames: Vec<String>,
    #[serde(default)]
    pub(crate) animation: TowerAnimationEntry,
    #[serde(default)]
    pub(crate) barrel_animation: TowerAnimationEntry,
    #[serde(default)]
    pub(crate) rotation_mode: String,
    #[serde(default)]
    pub(crate) barrel_layout: String,
    #[serde(default)]
    pub(crate) barrel_variants: Vec<TowerBarrelVariantEntry>,
    #[serde(default)]
    pub(crate) barrel_offset: TowerPointEntry,
    #[serde(default)]
    pub(crate) barrel_pivot: TowerPointEntry,
    #[serde(default)]
    pub(crate) muzzle_offset: TowerPointEntry,
    #[serde(default)]
    pub(crate) default_angle_deg: f32,
    #[serde(default)]
    pub(crate) recoil: TowerRecoilEntry,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct TowerAnimationEntry {
    #[serde(default)]
    pub(crate) frames: Vec<String>,
    #[serde(default)]
    pub(crate) fps: f32,
    #[serde(default, rename = "loop")]
    pub(crate) loop_animation: bool,
    #[serde(default)]
    pub(crate) fire_fps: f32,
    #[serde(default)]
    pub(crate) fire_once: bool,
}

impl Default for TowerAnimationEntry {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            fps: 10.0,
            loop_animation: true,
            fire_fps: 18.0,
            fire_once: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct TowerPointEntry {
    #[serde(default)]
    pub(crate) x: f32,
    #[serde(default)]
    pub(crate) y: f32,
}

impl Default for TowerPointEntry {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct TowerRecoilEntry {
    #[serde(default)]
    pub(crate) mode: String,
    #[serde(default)]
    pub(crate) distance: f32,
    #[serde(default)]
    pub(crate) scale: f32,
    #[serde(default)]
    pub(crate) duration_ms: u32,
    #[serde(default)]
    pub(crate) return_ms: u32,
}

impl Default for TowerRecoilEntry {
    fn default() -> Self {
        Self {
            mode: "directional".into(),
            distance: 7.0,
            scale: 0.94,
            duration_ms: 70,
            return_ms: 110,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct TowerBarrelVariantEntry {
    #[serde(default)]
    pub(crate) min_path: u8,
    #[serde(default)]
    pub(crate) min_level: u8,
    #[serde(default)]
    pub(crate) count: u16,
    #[serde(default)]
    pub(crate) image: String,
    #[serde(default)]
    pub(crate) frames: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub(crate) struct AttackTimingEntry {
    #[serde(default = "default_attack_windup")]
    pub(crate) windup: u16,
    #[serde(default = "default_attack_backswing")]
    pub(crate) backswing: u16,
}

pub(crate) fn default_attack_windup() -> u16 {
    350
}

pub(crate) fn default_attack_backswing() -> u16 {
    650
}

pub(crate) fn default_attack_timing() -> AttackTimingEntry {
    AttackTimingEntry {
        windup: default_attack_windup(),
        backswing: default_attack_backswing(),
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct UpgradeEntry {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) cost: i32,
    #[serde(default)]
    pub(crate) effects: Vec<UpgradeEffectEntry>,
    #[serde(default)]
    pub(crate) active_ability: Option<ActiveAbilityEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct ActiveAbilityEntry {
    #[serde(default)]
    pub(crate) ability_id: String,
    #[serde(default)]
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) icon: String,
    #[serde(default)]
    pub(crate) cooldown: f32,
    #[serde(default)]
    pub(crate) duration: f32,
    #[serde(default)]
    pub(crate) pulse_interval: f32,
    #[serde(default)]
    pub(crate) pulse_count: u16,
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct ActiveAbilityFixedRaw {
    pub(crate) cooldown: i64,
    pub(crate) duration: i64,
    pub(crate) pulse_interval: i64,
}

pub(crate) fn validate_active_ability_quantization(
    ability: &ActiveAbilityEntry,
) -> Result<ActiveAbilityFixedRaw, String> {
    let quantize = |field: &str, value: f32| {
        if !value.is_finite() {
            return Err(format!(
                "active ability '{}' {} must be finite",
                ability.ability_id, field
            ));
        }
        Ok((value * 1024.0).round() as i64)
    };
    let raw = ActiveAbilityFixedRaw {
        cooldown: quantize("cooldown", ability.cooldown)?,
        duration: quantize("duration", ability.duration)?,
        pulse_interval: quantize("pulse_interval", ability.pulse_interval)?,
    };
    if raw.cooldown <= 0 || raw.duration < 0 {
        return Err(format!(
            "active ability '{}' cooldown must quantize positive and duration must not be negative",
            ability.ability_id
        ));
    }
    let pulses_zero = raw.pulse_interval == 0 && ability.pulse_count == 0;
    let pulses_positive = raw.pulse_interval > 0 && ability.pulse_count > 0;
    if !pulses_zero && !pulses_positive {
        return Err(format!(
            "active ability '{}' pulse_interval and pulse_count must both quantize positive or both be zero",
            ability.ability_id
        ));
    }
    if pulses_positive
        && raw.duration
            < raw
                .pulse_interval
                .saturating_mul(ability.pulse_count as i64)
    {
        return Err(format!(
            "active ability '{}' duration must cover every authored pulse",
            ability.ability_id
        ));
    }
    Ok(raw)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum UpgradeEffectEntry {
    StatMod {
        key: String,
        value: f32,
        #[serde(default = "default_stat_op")]
        op: String,
    },
    BehaviorFlag {
        flag: String,
    },
}

fn default_stat_op() -> String {
    "add".into()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct HeroEntry {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) portrait: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) background: String,
    #[serde(default)]
    pub(crate) tombstone: bool,
    #[serde(default)]
    pub(crate) abilities: Vec<String>,
    #[serde(default)]
    pub(crate) strength: i32,
    #[serde(default)]
    pub(crate) agility: i32,
    #[serde(default)]
    pub(crate) intelligence: i32,
    #[serde(default)]
    pub(crate) primary_attribute: String,
    #[serde(default)]
    pub(crate) attack_range: f32,
    #[serde(default)]
    pub(crate) base_damage: i32,
    #[serde(default)]
    pub(crate) base_armor: f32,
    #[serde(default)]
    pub(crate) base_hp: i32,
    #[serde(default)]
    pub(crate) base_mana: i32,
    #[serde(default)]
    pub(crate) move_speed: f32,
    #[serde(default)]
    pub(crate) turn_speed: f32,
    #[serde(default = "default_attack_timing")]
    pub(crate) attack_timing: AttackTimingEntry,
    #[serde(default)]
    pub(crate) render: Option<HeroRenderEntry>,
    #[serde(default)]
    pub(crate) level_growth: HeroLevelGrowthEntry,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub(crate) struct HeroRenderEntry {
    #[serde(default)]
    pub(crate) render_mode: String,
    #[serde(default)]
    pub(crate) model: String,
    #[serde(default)]
    pub(crate) texture: String,
    #[serde(default)]
    pub(crate) scale: f32,
    #[serde(default)]
    pub(crate) pitch_offset_deg: f32,
    #[serde(default)]
    pub(crate) roll_offset_deg: f32,
    #[serde(default)]
    pub(crate) yaw_offset_deg: f32,
    #[serde(default)]
    pub(crate) z_offset: f32,
    #[serde(default)]
    pub(crate) muzzle_bone: String,
    #[serde(default)]
    pub(crate) animation_sources: BTreeMap<String, HeroAnimationSourceEntry>,
    #[serde(default)]
    pub(crate) animations: BTreeMap<String, HeroAnimationBindingEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub(crate) struct HeroAnimationSourceEntry {
    #[serde(default)]
    pub(crate) model: String,
    #[serde(default)]
    pub(crate) animation: String,
    #[serde(default)]
    pub(crate) duration_ticks: f32,
    #[serde(default)]
    pub(crate) ticks_per_second: f32,
    #[serde(default)]
    pub(crate) timeline_offset_ticks: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub(crate) struct HeroAnimationBindingEntry {
    #[serde(default)]
    pub(crate) source: String,
    #[serde(default)]
    pub(crate) start_tick: f32,
    #[serde(default)]
    pub(crate) repeat_start_tick: f32,
    #[serde(default)]
    pub(crate) impact_tick: Option<f32>,
    #[serde(default)]
    pub(crate) end_tick: f32,
    #[serde(default, rename = "loop")]
    pub(crate) loop_animation: bool,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub(crate) struct HeroLevelGrowthEntry {
    #[serde(default)]
    pub(crate) strength_per_level: f32,
    #[serde(default)]
    pub(crate) agility_per_level: f32,
    #[serde(default)]
    pub(crate) intelligence_per_level: f32,
    #[serde(default)]
    pub(crate) damage_per_level: f32,
    #[serde(default)]
    pub(crate) hp_per_level: f32,
    #[serde(default)]
    pub(crate) mana_per_level: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct CreepEntry {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) tombstone: bool,
    #[serde(default)]
    pub(crate) hp: f32,
    #[serde(default)]
    pub(crate) armor: f32,
    #[serde(default)]
    pub(crate) magic_resistance: f32,
    #[serde(default)]
    pub(crate) damage: f32,
    #[serde(default)]
    pub(crate) attack_range: f32,
    #[serde(default)]
    pub(crate) move_speed: f32,
    #[serde(default)]
    pub(crate) enemy_type: String,
    #[serde(default)]
    pub(crate) ai_type: String,
    #[serde(default = "default_attack_timing")]
    pub(crate) attack_timing: AttackTimingEntry,
    #[serde(default)]
    pub(crate) exp_reward: i32,
    #[serde(default)]
    pub(crate) gold_reward: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct SummonEntry {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) tombstone: bool,
    #[serde(default)]
    pub(crate) hp: f32,
    #[serde(default)]
    pub(crate) damage: f32,
    #[serde(default)]
    pub(crate) duration: f32,
    #[serde(default)]
    pub(crate) move_speed: f32,
    #[serde(default = "default_attack_timing")]
    pub(crate) attack_timing: AttackTimingEntry,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct AbilityEntry {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) tombstone: bool,
    #[serde(default)]
    pub(crate) icon: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) ability_type: String,
    #[serde(default)]
    pub(crate) cast_type: String,
    #[serde(default)]
    pub(crate) target_type: String,
    #[serde(default = "default_max_level")]
    pub(crate) max_level: u8,
    #[serde(default)]
    pub(crate) levels: Vec<AbilityLevelEntry>,
    #[serde(default)]
    pub(crate) extras: BTreeMap<String, Vec<f32>>,
}

fn default_max_level() -> u8 {
    4
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub(crate) struct AbilityLevelEntry {
    #[serde(default)]
    pub(crate) cooldown: f32,
    #[serde(default)]
    pub(crate) mana_cost: f32,
    #[serde(default)]
    pub(crate) cast_time: f32,
    #[serde(default)]
    pub(crate) range: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct ProjKind {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) tombstone: bool,
}

#[derive(Clone)]
pub(crate) struct LuaContentLoader {
    root: PathBuf,
    root_canonical: PathBuf,
    state: Rc<RefCell<LuaLoaderState>>,
}

#[derive(Default)]
struct LuaLoaderState {
    read_files: BTreeSet<PathBuf>,
    include_stack: Vec<PathBuf>,
}

impl LuaContentLoader {
    pub(crate) fn new(root: PathBuf) -> Result<Self, String> {
        let root_canonical = root
            .canonicalize()
            .map_err(|e| format!("canonicalize content root {}: {}", root.display(), e))?;
        Ok(Self {
            root,
            root_canonical,
            state: Rc::new(RefCell::new(LuaLoaderState::default())),
        })
    }

    pub(crate) fn load<T>(&self, lua: &Lua, rel_path: &str) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let value = self
            .load_value(lua, rel_path)
            .map_err(|e| format!("load Lua builder {}: {}", rel_path, e))?;
        lua.from_value(value)
            .map_err(|e| format!("convert Lua builder {} output: {}", rel_path, e))
    }

    pub(crate) fn load_json_value(
        &self,
        lua: &Lua,
        rel_path: &str,
    ) -> Result<serde_json::Value, String> {
        self.load(lua, rel_path)
    }

    pub(crate) fn load_map_json_value(
        &self,
        lua: &Lua,
        rel_path: &str,
        story_id: &str,
    ) -> Result<serde_json::Value, String> {
        let value = self
            .load_value(lua, rel_path)
            .map_err(|e| format!("load Lua builder {rel_path}: {e}"))?;
        let LuaValue::Table(map) = value else {
            return Err(format!("map builder {rel_path} must return a table"));
        };

        let selector_value: LuaValue = map
            .get("SelectSpawnPath")
            .map_err(|e| format!("read {story_id}.SelectSpawnPath: {e}"))?;
        map.set("SelectSpawnPath", LuaValue::Nil)
            .map_err(|e| format!("remove {story_id}.SelectSpawnPath before serialization: {e}"))?;

        if let LuaValue::Function(selector) = selector_value {
            let paths: mlua::Table = map
                .get("Path")
                .map_err(|e| format!("read {story_id}.Path for SelectSpawnPath: {e}"))?;
            let path_count = paths.raw_len();
            if path_count == 0 {
                return Err(format!(
                    "story {story_id} defines SelectSpawnPath but has no paths"
                ));
            }

            let rounds = lua
                .create_table()
                .map_err(|e| format!("create {story_id} spawn path selection table: {e}"))?;
            for round_index in 0..crate::td_rounds::round_count() {
                let round = lua.create_table().map_err(|e| {
                    format!(
                        "create {story_id} round {} selection table: {e}",
                        round_index + 1
                    )
                })?;
                for (balloon_index, balloon) in
                    crate::td_rounds::round(round_index).into_iter().enumerate()
                {
                    let params = lua.create_table().map_err(|e| {
                        format!(
                            "create {story_id} round {} balloon {} params: {e}",
                            round_index + 1,
                            balloon_index + 1
                        )
                    })?;
                    params.set("id", balloon.id).map_err(|e| e.to_string())?;
                    params
                        .set("label", balloon.label)
                        .map_err(|e| e.to_string())?;
                    params
                        .set("base", balloon.base)
                        .map_err(|e| e.to_string())?;
                    params
                        .set("hp", balloon.selector_hp)
                        .map_err(|e| e.to_string())?;
                    params
                        .set("camo", balloon.camo)
                        .map_err(|e| e.to_string())?;
                    params
                        .set("regrow", balloon.regrow)
                        .map_err(|e| e.to_string())?;
                    params
                        .set("fortified", balloon.fortified)
                        .map_err(|e| e.to_string())?;

                    let context = || {
                        format!(
                            "story {story_id} SelectSpawnPath round {} balloon {}",
                            round_index + 1,
                            balloon_index + 1
                        )
                    };
                    let selected: LuaValue = selector
                        .call((round_index + 1, balloon_index + 1, params))
                        .map_err(|e| format!("{} failed: {e}", context()))?;
                    let selected = valid_path_index(selected, path_count)
                        .map_err(|reason| format!("{} {reason}", context()))?;
                    round
                        .set(balloon_index + 1, selected)
                        .map_err(|e| format!("store {} result: {e}", context()))?;
                }
                rounds.set(round_index + 1, round).map_err(|e| {
                    format!("store {story_id} round {} selections: {e}", round_index + 1)
                })?;
            }
            map.set("SpawnPathSelections", rounds)
                .map_err(|e| format!("store {story_id}.SpawnPathSelections: {e}"))?;
        } else if !matches!(selector_value, LuaValue::Nil) {
            return Err(format!(
                "story {story_id} SelectSpawnPath must be a function, got {}",
                selector_value.type_name()
            ));
        }

        lua.from_value(LuaValue::Table(map))
            .map_err(|e| format!("convert Lua builder {rel_path} output: {e}"))
    }

    fn load_value(&self, lua: &Lua, rel_path: &str) -> mlua::Result<LuaValue> {
        let full_path = self.resolve_existing(rel_path)?;
        {
            let state = self.state.borrow();
            if let Some(first) = state.include_stack.iter().position(|p| p == &full_path) {
                let mut cycle: Vec<String> = state.include_stack[first..]
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect();
                cycle.push(full_path.display().to_string());
                return Err(mlua::Error::external(format!(
                    "Lua include cycle: {}",
                    cycle.join(" -> ")
                )));
            }
        }

        {
            let mut state = self.state.borrow_mut();
            state.read_files.insert(full_path.clone());
            state.include_stack.push(full_path.clone());
        }

        let result = (|| {
            let source = fs::read_to_string(&full_path).map_err(|e| {
                mlua::Error::external(format!("read {}: {}", full_path.display(), e))
            })?;
            let builder: mlua::Function = lua.load(&source).set_name(rel_path).eval()?;
            let ctx = self.create_context(lua)?;
            builder.call(ctx)
        })();

        self.state.borrow_mut().include_stack.pop();
        result
    }

    fn create_context(&self, lua: &Lua) -> mlua::Result<mlua::Table> {
        let ctx = lua.create_table()?;

        // Lua 的空 table 預設會被 serde 視為 map。Content descriptor 經常需要
        // 明確的空 sequence，因此提供 ctx.array({})，避免用假資料佔位。
        ctx.set(
            "array",
            lua.create_function(|lua, table: mlua::Table| {
                table.set_metatable(Some(lua.array_metatable()));
                Ok(table)
            })?,
        )?;

        let include_loader = self.clone();
        ctx.set(
            "include",
            lua.create_function(move |lua, rel_path: String| {
                include_loader.load_value(lua, &rel_path)
            })?,
        )?;

        let read_text_loader = self.clone();
        ctx.set(
            "read_text",
            lua.create_function(move |_lua, rel_path: String| {
                read_text_loader.read_text(&rel_path)
            })?,
        )?;

        let read_toml_loader = self.clone();
        ctx.set(
            "read_toml",
            lua.create_function(move |lua, rel_path: String| {
                let text = read_toml_loader.read_text(&rel_path)?;
                let parsed: toml::Value = toml::from_str(&text).map_err(|e| {
                    mlua::Error::external(format!("parse TOML {}: {}", rel_path, e))
                })?;
                lua.to_value(&parsed)
            })?,
        )?;

        Ok(ctx)
    }

    fn read_text(&self, rel_path: &str) -> mlua::Result<String> {
        let full_path = self.resolve_existing(rel_path)?;
        self.state.borrow_mut().read_files.insert(full_path.clone());
        fs::read_to_string(&full_path)
            .map_err(|e| mlua::Error::external(format!("read {}: {}", full_path.display(), e)))
    }

    fn resolve_existing(&self, rel_path: &str) -> mlua::Result<PathBuf> {
        let rel = Path::new(rel_path);
        if rel.as_os_str().is_empty() || rel.is_absolute() {
            return Err(mlua::Error::external(format!(
                "rejected content path '{}': must be relative to scripts/lua_data",
                rel_path
            )));
        }
        for component in rel.components() {
            match component {
                Component::Normal(_) | Component::CurDir => {}
                Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                    return Err(mlua::Error::external(format!(
                        "rejected content path '{}': parent/absolute paths are not allowed",
                        rel_path
                    )));
                }
            }
        }
        let full_path = self.root.join(rel);
        let canonical = full_path.canonicalize().map_err(|e| {
            mlua::Error::external(format!("resolve {}: {}", full_path.display(), e))
        })?;
        if !canonical.starts_with(&self.root_canonical) {
            return Err(mlua::Error::external(format!(
                "rejected content path '{}': resolved outside scripts/lua_data",
                rel_path
            )));
        }
        Ok(canonical)
    }

    pub(crate) fn read_files(&self) -> Vec<PathBuf> {
        self.state.borrow().read_files.iter().cloned().collect()
    }
}

fn valid_path_index(value: LuaValue, path_count: usize) -> Result<usize, String> {
    let index = match value {
        LuaValue::Integer(index) => usize::try_from(index).ok(),
        LuaValue::Number(index) if index.is_finite() && index.fract() == 0.0 => {
            if index >= 1.0 && index <= usize::MAX as f64 {
                Some(index as usize)
            } else {
                None
            }
        }
        _ => None,
    };
    match index {
        Some(index) if (1..=path_count).contains(&index) => Ok(index),
        Some(index) => Err(format!(
            "returned path index {index}; expected an integer in 1..={path_count}"
        )),
        None => Err(format!(
            "returned {}; expected an integer in 1..={path_count}",
            lua_value_description(&value)
        )),
    }
}

fn lua_value_description(value: &LuaValue) -> String {
    match value {
        LuaValue::Nil => "nil".into(),
        LuaValue::Boolean(value) => value.to_string(),
        LuaValue::Integer(value) => value.to_string(),
        LuaValue::Number(value) => value.to_string(),
        LuaValue::String(value) => format!("string {:?}", value.to_string_lossy()),
        _ => value.type_name().to_string(),
    }
}

#[derive(Debug, Serialize, Clone)]
pub(crate) struct StoryBundle {
    pub(crate) id: String,
    pub(crate) entity: serde_json::Value,
    pub(crate) ability: serde_json::Value,
    pub(crate) mission: serde_json::Value,
    pub(crate) map: serde_json::Value,
}

pub(crate) fn load_content(content_root: PathBuf) -> Result<LuaContent, String> {
    let lua = Lua::new();
    let loader = LuaContentLoader::new(content_root.clone())?;
    let manifest: Manifest = loader.load(&lua, "templates.lua")?;
    validate_td_layers(&manifest.td_layers)?;
    let stories = load_stories(&loader, &lua, &content_root, &manifest)?;
    if stories.iter().any(|story| {
        story
            .map
            .get("GameMode")
            .and_then(serde_json::Value::as_str)
            == Some("TowerDefense")
    }) {
        validate_td_round_references(&manifest.td_layers)?;
    }
    Ok(LuaContent {
        manifest,
        stories,
        read_files: loader.read_files(),
    })
}

pub(crate) fn validate_td_layers(entries: &[TdLayerEntry]) -> Result<(), String> {
    const KNOWN_PROPERTIES: u32 = 0b1111;
    const KNOWN_DAMAGE: u32 = 0xff;
    let by_id: BTreeMap<&str, &TdLayerEntry> = entries
        .iter()
        .map(|entry| (entry.id.as_str(), entry))
        .collect();
    if by_id.len() != entries.len() {
        return Err("TD layer catalog contains duplicate ids".into());
    }
    for entry in entries {
        if entry.id.is_empty() {
            return Err("TD layer id must not be empty".into());
        }
        if entry.hp == 0 {
            return Err(format!("TD layer '{}' hp must be positive", entry.id));
        }
        if entry.move_speed == 0 {
            return Err(format!(
                "TD layer '{}' move_speed must be positive",
                entry.id
            ));
        }
        if entry.accepted_damage == 0 || entry.accepted_damage & !KNOWN_DAMAGE != 0 {
            return Err(format!(
                "TD layer '{}' has invalid accepted_damage mask {:#x}",
                entry.id, entry.accepted_damage
            ));
        }
        if entry.properties & !KNOWN_PROPERTIES != 0 {
            return Err(format!(
                "TD layer '{}' has invalid properties mask {:#x}",
                entry.id, entry.properties
            ));
        }
        if entry.properties & 0b0110 != 0 {
            return Err(format!(
                "TD layer '{}' must author Regrow/Fortified as round modifiers, not base properties",
                entry.id
            ));
        }
        for child in &entry.children {
            if !by_id.contains_key(child.as_str()) {
                return Err(format!(
                    "TD layer '{}' field children references unknown layer '{}'",
                    entry.id, child
                ));
            }
        }
    }

    fn visit<'a>(
        id: &'a str,
        by_id: &BTreeMap<&'a str, &'a TdLayerEntry>,
        visiting: &mut Vec<&'a str>,
        done: &mut BTreeSet<&'a str>,
    ) -> Result<(), String> {
        if let Some(start) = visiting.iter().position(|candidate| *candidate == id) {
            let mut cycle = visiting[start..].to_vec();
            cycle.push(id);
            return Err(format!("TD layer cycle: {}", cycle.join(" -> ")));
        }
        if done.contains(id) {
            return Ok(());
        }
        visiting.push(id);
        let entry = by_id[id];
        for child in &entry.children {
            visit(child, by_id, visiting, done)?;
        }
        visiting.pop();
        done.insert(id);
        Ok(())
    }

    let mut done = BTreeSet::new();
    for entry in entries {
        visit(&entry.id, &by_id, &mut Vec::new(), &mut done)?;
    }
    for entry in entries {
        let child_leak = entry.children.iter().try_fold(0u32, |total, child| {
            total
                .checked_add(by_id[child.as_str()].leak_value)
                .ok_or_else(|| format!("TD layer '{}' child leak_value overflow", entry.id))
        })?;
        let expected_leak = entry
            .hp
            .checked_add(child_leak)
            .ok_or_else(|| format!("TD layer '{}' leak_value overflow", entry.id))?;
        if entry.leak_value != expected_leak {
            return Err(format!(
                "TD layer '{}' leak_value={} does not equal hp+children={}",
                entry.id, entry.leak_value, expected_leak
            ));
        }
    }
    if !entries.is_empty() {
        validate_td_round_references(entries)?;
    }
    Ok(())
}

fn validate_td_round_references(entries: &[TdLayerEntry]) -> Result<(), String> {
    let by_id: BTreeMap<&str, &TdLayerEntry> = entries
        .iter()
        .map(|entry| (entry.id.as_str(), entry))
        .collect();
    for round_index in 0..crate::td_rounds::round_count() {
        for balloon in crate::td_rounds::round(round_index) {
            let entry = by_id.get(balloon.base).ok_or_else(|| {
                format!(
                    "TD round {} references unknown layer '{}'",
                    round_index + 1,
                    balloon.base
                )
            })?;
            if balloon.regrow && !entry.regrow_eligible {
                return Err(format!(
                    "TD round {} layer '{}' is not Regrow eligible",
                    round_index + 1,
                    balloon.base
                ));
            }
            if balloon.fortified && !entry.fortified_eligible {
                return Err(format!(
                    "TD round {} layer '{}' is not Fortified eligible",
                    round_index + 1,
                    balloon.base
                ));
            }
        }
    }
    Ok(())
}

fn load_stories(
    loader: &LuaContentLoader,
    lua: &Lua,
    content_root: &Path,
    manifest: &Manifest,
) -> Result<Vec<StoryBundle>, String> {
    let mut story_ids: Vec<String> = fs::read_dir(content_root)
        .map_err(|e| format!("read content root {}: {}", content_root.display(), e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }
            let id = entry.file_name().to_string_lossy().into_owned();
            let has_story_files = ["entity.lua", "ability.lua", "mission.lua", "map.lua"]
                .iter()
                .all(|name| path.join(name).is_file());
            has_story_files.then_some(id)
        })
        .collect();
    story_ids.sort();

    let active_creeps: HashSet<&str> = manifest
        .creeps
        .iter()
        .filter(|creep| !creep.tombstone)
        .map(|creep| creep.id.as_str())
        .collect();

    story_ids
        .into_iter()
        .map(|id| {
            let entity = loader.load_json_value(lua, &format!("{}/entity.lua", id))?;
            let ability = loader.load_json_value(lua, &format!("{}/ability.lua", id))?;
            let mission = loader.load_json_value(lua, &format!("{}/mission.lua", id))?;
            let map = loader.load_map_json_value(lua, &format!("{}/map.lua", id), &id)?;
            validate_map_creep_references(&id, &map, &active_creeps)?;
            Ok(StoryBundle {
                id,
                entity,
                ability,
                mission,
                map,
            })
        })
        .collect()
}

fn validate_map_creep_references(
    story_id: &str,
    map: &serde_json::Value,
    active_creeps: &HashSet<&str>,
) -> Result<(), String> {
    let forbidden = [
        "Label",
        "HP",
        "DefendPhysic",
        "DefendMagic",
        "MoveSpeed",
        "damage",
        "attack_range",
        "enemy_type",
        "ai_type",
        "exp_reward",
        "gold_reward",
        "coins",
    ];
    let Some(creeps) = map.get("Creep").and_then(serde_json::Value::as_array) else {
        return Ok(());
    };
    for creep in creeps {
        let name = creep
            .get("Name")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("story {} map Creep[] entry missing Name", story_id))?;
        for field in forbidden {
            if creep.get(field).is_some() {
                return Err(format!(
                    "story {} map Creep '{}' has forbidden map-local unit field '{}'",
                    story_id, name, field
                ));
            }
        }
        if !active_creeps.contains(name) {
            return Err(format!(
                "story {} map Creep '{}' does not resolve to a generated creep template",
                story_id, name
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("omoba_lua_content_{name}_{stamp}"))
    }

    fn td_layer(id: &str, children: &[&str]) -> TdLayerEntry {
        TdLayerEntry {
            id: id.into(),
            label: id.into(),
            hp: 1,
            move_speed: 100,
            children: children.iter().map(|child| (*child).into()).collect(),
            cash: 1,
            leak_value: 1,
            properties: 0,
            accepted_damage: 0xff,
            regrow_eligible: true,
            fortified_eligible: true,
        }
    }

    #[test]
    fn td_layer_validation_reports_unknown_child_and_cycle_paths() {
        let unknown = validate_td_layers(&[td_layer("red", &["missing"])]).unwrap_err();
        assert!(unknown.contains("red"), "{unknown}");
        assert!(unknown.contains("missing"), "{unknown}");
        assert!(unknown.contains("children"), "{unknown}");

        let cyclic = validate_td_layers(&[
            td_layer("red", &["blue"]),
            td_layer("blue", &["green"]),
            td_layer("green", &["red"]),
        ])
        .unwrap_err();
        assert!(cyclic.contains("red -> blue -> green -> red"), "{cyclic}");
    }

    #[test]
    fn td_layer_validation_rejects_invalid_masks_and_stats() {
        let mut invalid_mask = td_layer("red", &[]);
        invalid_mask.accepted_damage = 1 << 12;
        let error = validate_td_layers(&[invalid_mask]).unwrap_err();
        assert!(error.contains("red"), "{error}");
        assert!(error.contains("accepted_damage"), "{error}");

        let mut invalid_hp = td_layer("red", &[]);
        invalid_hp.hp = 0;
        let error = validate_td_layers(&[invalid_hp]).unwrap_err();
        assert!(error.contains("red"), "{error}");
        assert!(error.contains("hp"), "{error}");
    }

    fn write(path: &Path, text: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, text).unwrap();
    }

    fn active_ability() -> ActiveAbilityEntry {
        ActiveAbilityEntry {
            ability_id: "test_active".into(),
            display_name: "Test".into(),
            description: "Test ability".into(),
            icon: "test.png".into(),
            cooldown: 10.0,
            duration: 5.0,
            pulse_interval: 0.5,
            pulse_count: 10,
        }
    }

    #[test]
    fn active_ability_rejects_positive_values_that_quantize_to_zero() {
        let mut ability = active_ability();
        ability.cooldown = 0.0001;
        assert!(validate_active_ability_quantization(&ability).is_err());

        ability.cooldown = 10.0;
        ability.pulse_interval = 0.0001;
        assert!(validate_active_ability_quantization(&ability).is_err());
    }

    #[test]
    fn instant_active_ability_allows_zero_duration_without_pulses() {
        let mut ability = active_ability();
        ability.duration = 0.0;
        ability.pulse_interval = 0.0;
        ability.pulse_count = 0;

        let raw = validate_active_ability_quantization(&ability).unwrap();

        assert_eq!(raw.duration, 0);
    }

    #[test]
    fn active_ability_rejects_non_finite_fixed_values() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut ability = active_ability();
            ability.duration = value;
            assert!(validate_active_ability_quantization(&ability).is_err());
        }
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

    #[test]
    fn spawn_path_selector_receives_one_based_indices_and_balloon_fields() {
        let root = temp_root("spawn_path_selector_args");
        write(
            &root.join("templates.lua"),
            "return function(ctx) return { creeps = { { id = 'known_creep' } } } end\n",
        );
        minimal_story(&root, "S", "known_creep");
        write(
            &root.join("S/map.lua"),
            r#"return function(ctx)
  return {
    Path = { { Name = 'a' }, { Name = 'b' }, { Name = 'c' } },
    Creep = { { Name = 'known_creep' } },
    SelectSpawnPath = function(round_index, balloon_index, balloon)
      if round_index < 1 or balloon_index < 1 then error('indices must be one-based') end
      for _, key in ipairs({ 'id', 'label', 'base', 'hp', 'camo', 'regrow', 'fortified' }) do
        if balloon[key] == nil then error('missing balloon field ' .. key) end
      end
      return ((balloon_index - 1) % 3) + 1
    end,
  }
end
"#,
        );

        let content = load_content(root.clone()).expect("valid selector content");
        let first_round = content.stories[0].map["SpawnPathSelections"][0]
            .as_array()
            .expect("round selections");
        assert_eq!(
            &first_round[..6],
            serde_json::json!([1, 2, 3, 1, 2, 3]).as_array().unwrap()
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn spawn_path_selector_rejects_out_of_range_result_with_context() {
        let root = temp_root("spawn_path_selector_range");
        write(
            &root.join("templates.lua"),
            "return function(ctx) return { creeps = { { id = 'known_creep' } } } end\n",
        );
        minimal_story(&root, "S", "known_creep");
        write(
            &root.join("S/map.lua"),
            "return function(ctx) return { Path = { { Name = 'only' } }, Creep = { { Name = 'known_creep' } }, SelectSpawnPath = function() return 2 end } end\n",
        );

        let error = load_content(root.clone()).expect_err("path 2 must be rejected");
        assert!(
            error.contains("story S SelectSpawnPath round 1 balloon 1"),
            "{error}"
        );
        assert!(error.contains("expected an integer in 1..=1"), "{error}");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn include_order_is_preserved() {
        let root = temp_root("include_order");
        write(
            &root.join("templates.lua"),
            "return function(ctx) return { creeps = { ctx.include('a.lua')[1], ctx.include('b.lua')[1] } } end\n",
        );
        write(
            &root.join("a.lua"),
            "return function(ctx) return { { id = 'a' } } end\n",
        );
        write(
            &root.join("b.lua"),
            "return function(ctx) return { { id = 'b' } } end\n",
        );
        minimal_story(&root, "S", "a");

        let content = load_content(root.clone()).unwrap();
        let ids: Vec<&str> = content
            .manifest
            .creeps
            .iter()
            .map(|entry| entry.id.as_str())
            .collect();
        assert_eq!(ids, ["a", "b"]);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn unsafe_paths_are_rejected() {
        let root = temp_root("unsafe_path");
        write(
            &root.join("templates.lua"),
            "return function(ctx) ctx.read_text('../secret.txt') return {} end\n",
        );
        let err = load_content(root.clone()).unwrap_err();
        assert!(
            err.contains("rejected content path '../secret.txt'"),
            "{err}"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn include_cycles_fail_clearly() {
        let root = temp_root("include_cycle");
        write(
            &root.join("templates.lua"),
            "return function(ctx) return ctx.include('a.lua') end\n",
        );
        write(
            &root.join("a.lua"),
            "return function(ctx) return ctx.include('b.lua') end\n",
        );
        write(
            &root.join("b.lua"),
            "return function(ctx) return ctx.include('a.lua') end\n",
        );
        let err = load_content(root.clone()).unwrap_err();
        assert!(err.contains("Lua include cycle"), "{err}");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn stories_are_sorted() {
        let root = temp_root("story_sort");
        write(
            &root.join("templates.lua"),
            "return function(ctx) return { creeps = { { id = 'creep_a' } } } end\n",
        );
        minimal_story(&root, "Z", "creep_a");
        minimal_story(&root, "A", "creep_a");
        let content = load_content(root.clone()).unwrap();
        let ids: Vec<&str> = content
            .stories
            .iter()
            .map(|story| story.id.as_str())
            .collect();
        assert_eq!(ids, ["A", "Z"]);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn missing_creep_template_reference_fails() {
        let root = temp_root("missing_creep_ref");
        write(
            &root.join("templates.lua"),
            "return function(ctx) return { creeps = { { id = 'known_creep' } } } end\n",
        );
        minimal_story(&root, "S", "missing_creep_template");
        let err = load_content(root.clone()).unwrap_err();
        assert!(err.contains("missing_creep_template"), "{err}");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn tower_defense_story_requires_complete_layer_catalog() {
        let root = temp_root("td_requires_layers");
        write(
            &root.join("templates.lua"),
            "return function(ctx) return { creeps = { { id = 'known_creep' } } } end\n",
        );
        minimal_story(&root, "TD", "known_creep");
        write(
            &root.join("TD/map.lua"),
            "return function(ctx) return { GameMode = 'TowerDefense', Creep = { { Name = 'known_creep' } } } end\n",
        );

        let error = load_content(root.clone()).expect_err("TD content needs layer catalog");
        assert!(
            error.contains("TD round 1 references unknown layer 'red'"),
            "{error}"
        );
        fs::remove_dir_all(root).ok();
    }
}
