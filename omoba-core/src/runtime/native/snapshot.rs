use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use omoba_sim::Fixed64;
use specs::{Join, ReadStorage, World, WorldExt};

use crate::lockstep_timing::LOCKSTEP_ONE_SECOND_TICKS_U32;
use crate::tower_meta::TowerActiveAbilityDef;

use super::ability_runtime::{AbilityRegistry, BuffStore, UnitStats};
use super::comp::hero::AttributeType;
use super::comp::{
    BlockedRegions, CProperty, Creep, CreepWave, CurrentCreepWave, Facing, GamePause, GameSpeed,
    Gold, Hero, HeroCommand, HeroCommandQueue, Inventory, IsBuilding, MoveTarget,
    Path as CreepPath, PlayerLives, PlayerOwner, Pos, Projectile, RemovedEntitiesQueue, TAttack,
    Tower, TowerAbilityCastResult, TowerAbilityCastResults, TowerSpawnOrder, TowerTemplateRegistry,
    TowerUpgradeRegistry,
};
use super::scripting::{ScriptUnitTag, ScriptVisualEventKind, ScriptVisualEventQueue};

pub use super::comp::{AttackCancelFx, AttackPhaseFx, ExplosionFx, TowerFireFx};

const PERMANENT_BUFF_REMAINING_RAW_THRESHOLD: i64 = (i32::MAX as i64) / 2;
const RENDER_FX_RETENTION_TICKS: u32 = LOCKSTEP_ONE_SECOND_TICKS_U32 / 2;

#[derive(Clone, Debug, Default)]
pub struct AppliedInputMeta {
    pub input_id: u32,
    pub server_receive_tick: u32,
    pub server_drain_tick: u32,
    pub server_queue_us: u64,
    pub client_receive_us: u64,
    pub game_forward_us: u64,
    pub extract_data_for_render_us: u64,
}

#[derive(Default, Clone, Debug)]
pub struct SimWorldSnapshot {
    pub tick: u32,
    pub entities: Vec<EntityRenderData>,
    pub paths: Vec<Vec<(f32, f32)>>,
    pub removed_entity_ids: Vec<u32>,
    pub round: u32,
    pub total_rounds: u32,
    pub lives: i32,
    pub round_is_running: bool,
    pub is_paused: bool,
    pub game_speed_multiplier: u32,
    pub blocked_regions: Vec<BlockedRegionSnapshot>,
    pub abilities: Arc<Vec<AbilityDefSnapshot>>,
    pub tower_templates: Arc<Vec<TowerTemplateSnapshot>>,
    pub tower_upgrades: Arc<Vec<TowerUpgradeDefSnapshot>>,
    pub explosions: Vec<ExplosionFx>,
    pub tower_fire_fx: Vec<TowerFireFx>,
    pub attack_phase_fx: Vec<AttackPhaseFx>,
    pub attack_cancel_fx: Vec<AttackCancelFx>,
    pub script_visual_events: Vec<ScriptVisualEventSnapshot>,
    pub applied_input_ids: Vec<u32>,
    pub applied_input_meta: Vec<AppliedInputMeta>,
    pub lua_content_generation: u64,
    pub lua_content_hash: String,
    pub dev_lua_reload_error: Option<String>,
    pub latest_tower_ability_cast_result: Option<TowerAbilityCastResultSnapshot>,
    pub tower_ability_cast_results: Vec<TowerAbilityCastResultSnapshot>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TowerAbilityCastResultSnapshot {
    pub player_id: u32,
    pub tower_entity_id: u32,
    pub ability_id: String,
    pub accepted: bool,
    pub reason: String,
    pub result_serial: u32,
}

impl From<&TowerAbilityCastResult> for TowerAbilityCastResultSnapshot {
    fn from(result: &TowerAbilityCastResult) -> Self {
        Self {
            player_id: result.player_id,
            tower_entity_id: result.tower_entity_id,
            ability_id: result.ability_id.clone(),
            accepted: result.accepted,
            reason: result.reason.clone(),
            result_serial: result.result_serial,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TowerActiveAbilitySnapshot {
    pub ability_id: String,
    pub display_name: String,
    pub description: String,
    pub icon: String,
    pub cooldown_total: f32,
    pub cooldown_remaining: f32,
    pub active_remaining: f32,
    pub activation_serial: u32,
}

#[derive(Clone, Debug, Default)]
pub struct BlockedRegionSnapshot {
    pub points: Vec<(f32, f32)>,
    pub circle: Option<((f32, f32), f32)>,
}

#[derive(Clone, Debug)]
pub struct ScriptVisualEventSnapshot {
    pub kind: ScriptVisualEventKind,
    pub primary_entity_id: u32,
    pub primary_entity_gen: u32,
    pub secondary_entity_id: Option<u32>,
    pub secondary_entity_gen: Option<u32>,
    pub skill_id: Option<String>,
    pub state_id: Option<String>,
    pub modifier_id: Option<String>,
    pub order_id: Option<String>,
    pub amount: f32,
    pub damage: f32,
    pub action_instance_id: u64,
    pub first_tick: u64,
    pub latest_tick: u64,
    pub hook_count: u32,
    pub accumulated_dt_secs: f32,
}

#[derive(Clone, Debug)]
pub struct AbilityDefSnapshot {
    pub ability_id: String,
    pub display_name: String,
    pub max_level: u8,
    pub icon_path: String,
}

#[derive(Clone, Debug)]
pub struct TowerUpgradeDefSnapshot {
    pub tower_kind: String,
    pub path: u8,
    pub level: u8,
    pub name: String,
    pub description: String,
    pub cost: i32,
    pub active_ability: Option<TowerActiveAbilityDef>,
}

#[derive(Clone, Debug, Default)]
pub struct TowerRenderPointSnapshot {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Default)]
pub struct TowerRenderAnimationSnapshot {
    pub fps: f32,
    pub loop_animation: bool,
    pub fire_fps: f32,
    pub fire_once: bool,
}

#[derive(Clone, Debug, Default)]
pub struct TowerBarrelVariantSnapshot {
    pub min_path: u8,
    pub min_level: u8,
    pub count: u16,
    pub image: String,
    pub frames: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct TowerRecoilSnapshot {
    pub mode: String,
    pub distance: f32,
    pub scale: f32,
    pub duration_ms: u32,
    pub return_ms: u32,
}

#[derive(Clone, Debug)]
pub struct TowerTemplateSnapshot {
    pub unit_id: String,
    pub label: String,
    pub cost: i32,
    pub footprint: f32,
    pub placement_radius: f32,
    pub range: f32,
    pub splash_radius: f32,
    pub hit_radius: f32,
    pub slow_factor: f32,
    pub slow_duration: f32,
    pub render_mode: String,
    pub base_image: String,
    pub barrel_image: String,
    pub render_visual_size: f32,
    pub barrel_frames: Vec<String>,
    pub body_frames: Vec<String>,
    pub barrel_animation: TowerRenderAnimationSnapshot,
    pub body_animation: TowerRenderAnimationSnapshot,
    pub rotation_mode: String,
    pub barrel_layout: String,
    pub barrel_variants: Vec<TowerBarrelVariantSnapshot>,
    pub barrel_offset: TowerRenderPointSnapshot,
    pub barrel_pivot: TowerRenderPointSnapshot,
    pub muzzle_offset: TowerRenderPointSnapshot,
    pub default_angle_deg: f32,
    pub recoil: TowerRecoilSnapshot,
    pub attack_windup: u16,
    pub attack_backswing: u16,
}

#[derive(Clone, Debug, Default)]
pub struct HeroAnimationSourceSnapshot {
    pub key: String,
    pub model: String,
    pub animation: String,
    pub duration_ticks: f32,
    pub ticks_per_second: f32,
    pub timeline_offset_ticks: f32,
}

#[derive(Clone, Debug, Default)]
pub struct HeroAnimationBindingSnapshot {
    pub action: String,
    pub source: String,
    pub start_tick: f32,
    pub end_tick: f32,
    pub repeat_start_tick: f32,
    pub impact_tick: Option<f32>,
    pub loop_animation: bool,
}

#[derive(Clone, Debug, Default)]
pub struct HeroRenderSnapshot {
    pub render_mode: String,
    pub model: String,
    pub texture: String,
    pub scale: f32,
    pub pitch_offset_deg: f32,
    pub roll_offset_deg: f32,
    pub yaw_offset_deg: f32,
    pub z_offset: f32,
    pub muzzle_bone: String,
    pub animation_sources: Vec<HeroAnimationSourceSnapshot>,
    pub animations: Vec<HeroAnimationBindingSnapshot>,
    pub is_moving: bool,
    pub sniper_mode: bool,
}

#[derive(Clone, Debug, Default)]
pub struct EntityRenderData {
    pub entity_id: u32,
    pub entity_gen: u32,
    pub spawn_order: u64,
    pub kind: EntityKind,
    pub pos_x: f32,
    pub pos_y: f32,
    pub facing_rad: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub unit_id: String,
    pub hero_name: String,
    pub hero_title: String,
    pub hero_level: i32,
    pub hero_xp: i32,
    pub hero_xp_next: i32,
    pub hero_skill_points: i32,
    pub hero_primary_attribute: String,
    pub hero_strength: i32,
    pub hero_agility: i32,
    pub hero_intelligence: i32,
    pub gold: i32,
    pub hero_ext: Option<Box<HeroStatsExt>>,
    pub hero_render: Option<Box<HeroRenderSnapshot>>,
    pub projectile_owner_entity_id: Option<u32>,
    pub owner_player_id: Option<u32>,
    pub upgrade_levels: Option<[u8; 3]>,
    pub tower_pops: Option<u32>,
    pub tower_atk: Option<f32>,
    pub tower_asd: Option<f32>,
    pub tower_target_priority: String,
    pub tower_active_ability: Option<TowerActiveAbilitySnapshot>,
    pub hero_command: Option<Box<HeroCommandSnapshot>>,
    pub buffs: Vec<BuffSnapshot>,
    pub attack_range: f32,
}

fn tower_active_ability_snapshot(
    tower: Option<&Tower>,
    hp: i32,
    upgrades: &TowerUpgradeRegistry,
) -> Option<TowerActiveAbilitySnapshot> {
    let state = tower?.active_ability.as_ref()?;
    if hp <= 0 {
        return None;
    }
    let def = upgrades.iter_all().find_map(|def| {
        def.active_ability
            .as_ref()
            .filter(|ability| ability.ability_id == state.ability_id)
    })?;
    Some(TowerActiveAbilitySnapshot {
        ability_id: state.ability_id.clone(),
        display_name: def.display_name.clone(),
        description: def.description.clone(),
        icon: def.icon.clone(),
        cooldown_total: def.cooldown.to_f32_for_render(),
        cooldown_remaining: state.cooldown_remaining.to_f32_for_render(),
        active_remaining: state.active_remaining.to_f32_for_render(),
        activation_serial: state.activation_serial,
    })
}

fn latest_tower_ability_cast_result(world: &World) -> Option<TowerAbilityCastResultSnapshot> {
    world
        .try_fetch::<TowerAbilityCastResult>()
        .and_then(|result| {
            (result.result_serial != 0).then(|| TowerAbilityCastResultSnapshot::from(&*result))
        })
}

fn tower_ability_cast_results(world: &World) -> Vec<TowerAbilityCastResultSnapshot> {
    world
        .try_fetch::<TowerAbilityCastResults>()
        .map(|results| {
            results
                .latest_by_player
                .values()
                .map(TowerAbilityCastResultSnapshot::from)
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Clone, Debug, Default)]
pub struct HeroCommandSnapshot {
    pub command_type: String,
    pub target_entity_id: Option<u32>,
    pub destination: Option<(f32, f32)>,
    pub next_waypoint: Option<(f32, f32)>,
    pub queued_count: u8,
    pub queue_limit: u8,
    pub queued_targets: Vec<HeroQueuedCommandSnapshot>,
}

#[derive(Clone, Debug, Default)]
pub struct HeroQueuedCommandSnapshot {
    pub command_type: String,
    pub target_entity_id: Option<u32>,
    pub target: Option<(f32, f32)>,
}

fn hero_command_target_point(
    command: HeroCommand,
    pos_storage: &ReadStorage<'_, Pos>,
) -> Option<(f32, f32)> {
    command
        .destination()
        .map(|pos| (pos.x.to_f32_for_render(), pos.y.to_f32_for_render()))
        .or_else(|| {
            command.target().and_then(|entity| {
                pos_storage
                    .get(entity)
                    .map(|pos| (pos.0.x.to_f32_for_render(), pos.0.y.to_f32_for_render()))
            })
        })
}

fn hero_command_snapshot(
    queue: Option<&HeroCommandQueue>,
    move_target: Option<&MoveTarget>,
    pos_storage: &ReadStorage<'_, Pos>,
) -> Option<Box<HeroCommandSnapshot>> {
    let queue = queue?;
    let active = queue.active?;
    let destination = active
        .destination()
        .map(|pos| (pos.x.to_f32_for_render(), pos.y.to_f32_for_render()));
    let next_waypoint = move_target.map(|target| {
        (
            target.0.x.to_f32_for_render(),
            target.0.y.to_f32_for_render(),
        )
    });
    let queued_targets = queue
        .queued
        .iter()
        .copied()
        .map(|command| HeroQueuedCommandSnapshot {
            command_type: command.command_type().to_string(),
            target_entity_id: command.target().map(|entity| entity.id()),
            target: hero_command_target_point(command, pos_storage),
        })
        .collect();
    Some(Box::new(HeroCommandSnapshot {
        command_type: active.command_type().to_string(),
        target_entity_id: active.target().map(|entity| entity.id()),
        destination,
        next_waypoint,
        queued_count: queue.queued.len().min(u8::MAX as usize) as u8,
        queue_limit: HeroCommandQueue::LIMIT as u8,
        queued_targets,
    }))
}

#[cfg(test)]
mod tests {
    use super::super::comp::{HeroCommand, TowerAbilityCastResults};
    use super::*;
    use omoba_sim::Fixed64;

    #[test]
    fn tower_upgrade_snapshot_exposes_active_ability_metadata() {
        use omoba_template_ids::TOWER_ARTY;

        let registry = TowerUpgradeRegistry::new();
        let snapshots = build_tower_upgrade_def_snapshots(&registry);
        let active = snapshots
            .iter()
            .find(|def| def.tower_kind == TOWER_ARTY.as_str() && def.path == 2 && def.level == 4)
            .and_then(|def| def.active_ability.as_ref())
            .expect("Arty level-four active ability snapshot");

        assert_eq!(active.ability_id, "arty_fire_at_will");
        assert_eq!(active.display_name, "火力全開");
        assert_eq!(active.cooldown, Fixed64::from_i32(10));
        assert_eq!(active.duration, Fixed64::from_i32(3));
        assert_eq!(active.pulse_interval, Fixed64::from_raw(512));
        assert_eq!(active.pulse_count, 6);
    }

    #[test]
    fn tower_ability_cast_result_snapshots_are_sorted_by_player() {
        let mut world = World::new();
        let mut results = TowerAbilityCastResults::default();
        for player_id in [8, 7] {
            results.latest_by_player.insert(
                player_id,
                TowerAbilityCastResult {
                    player_id,
                    tower_entity_id: player_id * 10,
                    ability_id: format!("ability_{player_id}"),
                    accepted: false,
                    reason: "tower_missing".into(),
                    result_serial: player_id,
                },
            );
        }
        world.insert(results);

        let snapshots = tower_ability_cast_results(&world);

        assert_eq!(
            snapshots
                .iter()
                .map(|result| result.player_id)
                .collect::<Vec<_>>(),
            vec![7, 8]
        );
    }

    #[test]
    fn hero_command_snapshot_exposes_active_command_without_mutating_queue() {
        let queue = HeroCommandQueue {
            active: Some(HeroCommand::AttackMove {
                pos: omoba_sim::Vec2::new(Fixed64::from_i32(10), Fixed64::from_i32(20)),
            }),
            queued: vec![HeroCommand::MoveTo {
                pos: omoba_sim::Vec2::new(Fixed64::from_i32(30), Fixed64::from_i32(40)),
            }],
        };
        let before = queue.clone();
        let move_target = MoveTarget(omoba_sim::Vec2::new(
            Fixed64::from_i32(11),
            Fixed64::from_i32(21),
        ));

        let mut world = World::new();
        world.register::<Pos>();
        let pos_storage = world.read_storage::<Pos>();

        let snapshot =
            hero_command_snapshot(Some(&queue), Some(&move_target), &pos_storage).unwrap();

        assert_eq!(queue, before);
        assert_eq!(snapshot.command_type, "attack_move");
        assert_eq!(snapshot.destination, Some((10.0, 20.0)));
        assert_eq!(snapshot.next_waypoint, Some((11.0, 21.0)));
        assert_eq!(snapshot.queued_count, 1);
        assert_eq!(snapshot.queue_limit, HeroCommandQueue::LIMIT as u8);
        assert_eq!(snapshot.queued_targets.len(), 1);
        assert_eq!(snapshot.queued_targets[0].command_type, "move_to");
        assert_eq!(snapshot.queued_targets[0].target, Some((30.0, 40.0)));
    }
}

#[derive(Clone, Debug, Default)]
pub struct BuffSnapshot {
    pub buff_id: String,
    pub remaining_secs: f32,
    pub payload_json: String,
}

#[derive(Clone, Debug, Default)]
pub struct HeroStatsExt {
    pub armor: f32,
    pub magic_resist: f32,
    pub attack_damage: f32,
    pub attack_range: f32,
    pub move_speed: f32,
    pub attack_speed_sec: f32,
    pub bullet_speed: f32,
    pub mana: f32,
    pub max_mana: f32,
    pub buffs: Vec<BuffSnapshot>,
    pub inventory: [Option<String>; 6],
    pub ability_levels: [i32; 4],
    pub ability_ids: [Option<String>; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EntityKind {
    Hero,
    Tower,
    Creep,
    Projectile,
    #[default]
    Other,
}

pub fn buff_remaining_secs_for_snapshot(remaining: Fixed64) -> f32 {
    if remaining.raw() >= PERMANENT_BUFF_REMAINING_RAW_THRESHOLD {
        -1.0
    } else {
        remaining.to_f32_for_render()
    }
}

pub fn hero_render_snapshot_for_unit_id(
    unit_id: &str,
    is_moving: bool,
    sniper_mode: bool,
) -> Option<Box<HeroRenderSnapshot>> {
    let hero_name = unit_id.strip_prefix("hero_")?;
    let hero_id = omoba_template_ids::hero_by_name(hero_name)?;
    let render = omoba_template_ids::active_hero_render_metadata(hero_id)?;
    let render_mode = match render.render_mode {
        omoba_template_ids::HeroRenderModeC::Model3d => "model_3d",
    };

    Some(Box::new(HeroRenderSnapshot {
        render_mode: render_mode.to_string(),
        model: render.model.to_string(),
        texture: render.texture.to_string(),
        scale: render.scale.to_f32_for_render(),
        pitch_offset_deg: render.pitch_offset_deg.to_f32_for_render(),
        roll_offset_deg: render.roll_offset_deg.to_f32_for_render(),
        yaw_offset_deg: render.yaw_offset_deg.to_f32_for_render(),
        z_offset: render.z_offset.to_f32_for_render(),
        muzzle_bone: render.muzzle_bone.to_string(),
        animation_sources: render
            .animation_sources
            .iter()
            .map(|source| HeroAnimationSourceSnapshot {
                key: source.key.to_string(),
                model: source.model.to_string(),
                animation: source.animation.to_string(),
                duration_ticks: source.duration_ticks.to_f32_for_render(),
                ticks_per_second: source.ticks_per_second.to_f32_for_render(),
                timeline_offset_ticks: source.timeline_offset_ticks.to_f32_for_render(),
            })
            .collect(),
        animations: render
            .animations
            .iter()
            .map(|binding| HeroAnimationBindingSnapshot {
                action: binding.action.to_string(),
                source: binding.source.to_string(),
                start_tick: binding.start_tick.to_f32_for_render(),
                end_tick: binding.end_tick.to_f32_for_render(),
                repeat_start_tick: binding.repeat_start_tick.to_f32_for_render(),
                impact_tick: binding
                    .has_impact_tick
                    .then(|| binding.impact_tick.to_f32_for_render()),
                loop_animation: binding.loop_animation,
            })
            .collect(),
        is_moving,
        sniper_mode,
    }))
}

pub fn retain_recent_render_fx<T: Clone>(
    retained: &mut VecDeque<T>,
    current: Vec<T>,
    tick: u32,
    spawn_tick: impl Fn(&T) -> u32,
) -> Vec<T> {
    let fx_span = tracing::trace_span!(
        "omoba_core::runtime::retain_recent_render_fx",
        perfetto = true,
        tick,
        retained_count = retained.len(),
        current_count = current.len(),
    )
    .entered();
    retained.extend(current);
    while retained
        .front()
        .map(|fx| tick.saturating_sub(spawn_tick(fx)) > RENDER_FX_RETENTION_TICKS)
        .unwrap_or(false)
    {
        retained.pop_front();
    }
    let out = retained.iter().cloned().collect();
    drop(fx_span);
    out
}

fn drain_script_visual_events(world: &World) -> Vec<ScriptVisualEventSnapshot> {
    let mut queue = world.write_resource::<ScriptVisualEventQueue>();
    queue
        .drain()
        .into_iter()
        .map(|event| ScriptVisualEventSnapshot {
            kind: event.kind,
            primary_entity_id: event.primary.id(),
            primary_entity_gen: event.primary.gen().id() as u32,
            secondary_entity_id: event.secondary.map(|entity| entity.id()),
            secondary_entity_gen: event.secondary.map(|entity| entity.gen().id() as u32),
            skill_id: event.skill_id,
            state_id: event.state_id,
            modifier_id: event.modifier_id,
            order_id: event.order_id,
            amount: event.amount.to_f32_for_render(),
            damage: event.damage.to_f32_for_render(),
            action_instance_id: event.action_instance_id,
            first_tick: event.first_tick,
            latest_tick: event.latest_tick,
            hook_count: event.hook_count,
            accumulated_dt_secs: event.accumulated_dt.to_f32_for_render(),
        })
        .collect()
}

pub fn build_ability_def_snapshots(reg: &AbilityRegistry) -> Vec<AbilityDefSnapshot> {
    let metadata_span = tracing::trace_span!(
        "omoba_core::runtime::build_ability_def_snapshots",
        perfetto = true,
    )
    .entered();
    let out = reg
        .all()
        .map(|d| AbilityDefSnapshot {
            ability_id: d.id.clone(),
            display_name: d.name.clone(),
            max_level: d.max_level,
            icon_path: d.icon.clone().unwrap_or_default(),
        })
        .collect();
    drop(metadata_span);
    out
}

pub fn build_tower_template_snapshots(reg: &TowerTemplateRegistry) -> Vec<TowerTemplateSnapshot> {
    let metadata_span = tracing::trace_span!(
        "omoba_core::runtime::build_tower_template_snapshots",
        perfetto = true,
    )
    .entered();
    let out = reg
        .iter_ordered()
        .map(|t| TowerTemplateSnapshot {
            unit_id: t.unit_id.clone(),
            label: t.label.clone(),
            cost: t.cost,
            footprint: t.footprint,
            placement_radius: t.placement_radius,
            range: t.range,
            splash_radius: t.splash_radius,
            hit_radius: t.hit_radius,
            slow_factor: t.slow_factor,
            slow_duration: t.slow_duration,
            render_mode: t.render.render_mode.clone(),
            base_image: t.render.base.clone(),
            barrel_image: t.render.barrel.clone(),
            render_visual_size: t.render.visual_size,
            barrel_frames: t.render.barrel_frames.clone(),
            body_frames: t.render.body_frames.clone(),
            barrel_animation: TowerRenderAnimationSnapshot {
                fps: t.render.barrel_animation.fps,
                loop_animation: t.render.barrel_animation.loop_animation,
                fire_fps: t.render.barrel_animation.fire_fps,
                fire_once: t.render.barrel_animation.fire_once,
            },
            body_animation: TowerRenderAnimationSnapshot {
                fps: t.render.body_animation.fps,
                loop_animation: t.render.body_animation.loop_animation,
                fire_fps: t.render.body_animation.fire_fps,
                fire_once: t.render.body_animation.fire_once,
            },
            rotation_mode: t.render.rotation_mode.clone(),
            barrel_layout: t.render.barrel_layout.clone(),
            barrel_variants: t
                .render
                .barrel_variants
                .iter()
                .map(|v| TowerBarrelVariantSnapshot {
                    min_path: v.min_path,
                    min_level: v.min_level,
                    count: v.count,
                    image: v.image.clone(),
                    frames: v.frames.clone(),
                })
                .collect(),
            barrel_offset: TowerRenderPointSnapshot {
                x: t.render.barrel_offset.x,
                y: t.render.barrel_offset.y,
            },
            barrel_pivot: TowerRenderPointSnapshot {
                x: t.render.barrel_pivot.x,
                y: t.render.barrel_pivot.y,
            },
            muzzle_offset: TowerRenderPointSnapshot {
                x: t.render.muzzle_offset.x,
                y: t.render.muzzle_offset.y,
            },
            default_angle_deg: t.render.default_angle_deg,
            recoil: TowerRecoilSnapshot {
                mode: t.render.recoil.mode.clone(),
                distance: t.render.recoil.distance,
                scale: t.render.recoil.scale,
                duration_ms: t.render.recoil.duration_ms,
                return_ms: t.render.recoil.return_ms,
            },
            attack_windup: t.attack_timing.windup,
            attack_backswing: t.attack_timing.backswing,
        })
        .collect();
    drop(metadata_span);
    out
}

pub fn build_tower_upgrade_def_snapshots(
    reg: &TowerUpgradeRegistry,
) -> Vec<TowerUpgradeDefSnapshot> {
    let metadata_span = tracing::trace_span!(
        "omoba_core::runtime::build_tower_upgrade_def_snapshots",
        perfetto = true,
    )
    .entered();
    let mut defs: Vec<TowerUpgradeDefSnapshot> = reg
        .iter_all()
        .map(|d| TowerUpgradeDefSnapshot {
            tower_kind: d.tower_kind.clone(),
            path: d.path,
            level: d.level,
            name: d.name.clone(),
            description: d.description.clone(),
            cost: d.cost,
            active_ability: d.active_ability.clone(),
        })
        .collect();
    defs.sort_by(|a, b| {
        a.tower_kind
            .cmp(&b.tower_kind)
            .then(a.path.cmp(&b.path))
            .then(a.level.cmp(&b.level))
    });
    drop(metadata_span);
    defs
}

pub fn extract_snapshot(
    world: &mut World,
    tick: u32,
    abilities_arc: Arc<Vec<AbilityDefSnapshot>>,
    tower_templates_arc: Arc<Vec<TowerTemplateSnapshot>>,
    tower_upgrades_arc: Arc<Vec<TowerUpgradeDefSnapshot>>,
    applied_input_ids: Vec<u32>,
    applied_input_meta: Vec<AppliedInputMeta>,
) -> SimWorldSnapshot {
    let snapshot_span = tracing::trace_span!(
        "omoba_core::runtime::extract_snapshot",
        perfetto = true,
        tick,
    )
    .entered();
    let storage_span = tracing::trace_span!(
        "omoba_core::runtime::extract_runtime.storage_borrows",
        perfetto = true,
    )
    .entered();
    let entities = world.entities();
    let pos_storage = world.read_storage::<Pos>();
    let facing_storage = world.read_storage::<Facing>();
    let cprop_storage = world.read_storage::<CProperty>();
    let hero_storage = world.read_storage::<Hero>();
    let tower_storage = world.read_storage::<Tower>();
    let tower_spawn_order_storage = world.read_storage::<TowerSpawnOrder>();
    let proj_storage = world.read_storage::<Projectile>();
    let creep_storage = world.read_storage::<Creep>();
    let unit_tag_storage = world.read_storage::<ScriptUnitTag>();
    let move_target_storage = world.read_storage::<MoveTarget>();
    let command_queue_storage = world.read_storage::<HeroCommandQueue>();
    let gold_storage = world.read_storage::<Gold>();
    let tatk_storage = world.read_storage::<TAttack>();
    let owner_storage = world.read_storage::<PlayerOwner>();
    let buff_store = world.read_resource::<BuffStore>();
    let is_building_storage = world.read_storage::<IsBuilding>();
    let inventory_storage = world.read_storage::<Inventory>();
    let tower_upgrade_registry = world.read_resource::<TowerUpgradeRegistry>();
    drop(storage_span);

    let mut out = Vec::new();
    let entities_span = tracing::trace_span!(
        "omoba_core::runtime::extract_runtime.entities",
        perfetto = true,
    )
    .entered();
    for (entity, pos) in (&entities, &pos_storage).join() {
        let entity_span = tracing::trace_span!(
            "omoba_core::runtime::extract_runtime.entity",
            perfetto = true,
            entity = entity.id(),
        )
        .entered();
        let classify_span = tracing::trace_span!(
            "omoba_core::runtime::extract_runtime.entity.classify",
            perfetto = true,
        )
        .entered();
        let kind = if hero_storage.get(entity).is_some() {
            EntityKind::Hero
        } else if tower_storage.get(entity).is_some() {
            EntityKind::Tower
        } else if proj_storage.get(entity).is_some() {
            EntityKind::Projectile
        } else if creep_storage.get(entity).is_some() {
            EntityKind::Creep
        } else {
            EntityKind::Other
        };
        drop(classify_span);

        let scalar_span = tracing::trace_span!(
            "omoba_core::runtime::extract_runtime.entity.scalars",
            perfetto = true,
            kind = ?kind,
        )
        .entered();
        let facing = facing_storage
            .get(entity)
            .map(|f| {
                (f.0.ticks() as f32) / (omoba_sim::trig::TAU_TICKS as f32)
                    * 2.0
                    * std::f32::consts::PI
            })
            .unwrap_or(0.0);

        let (hp, max_hp) = cprop_storage
            .get(entity)
            .map(|c| {
                (
                    c.hp.to_f32_for_render() as i32,
                    c.mhp.to_f32_for_render() as i32,
                )
            })
            .unwrap_or((0, 0));

        let (px, py) = pos.xy_f32();
        drop(scalar_span);

        let identity_span = tracing::trace_span!(
            "omoba_core::runtime::extract_runtime.entity.identity_render",
            perfetto = true,
            kind = ?kind,
        )
        .entered();
        let unit_id = unit_tag_storage
            .get(entity)
            .map(|t| t.unit_id.clone())
            .unwrap_or_default();
        let hero_render = if matches!(kind, EntityKind::Hero) {
            hero_render_snapshot_for_unit_id(
                &unit_id,
                move_target_storage.get(entity).is_some(),
                buff_store.has(entity, "sniper_mode"),
            )
        } else {
            None
        };
        let projectile_owner_entity_id = proj_storage.get(entity).map(|p| p.owner.id());
        let owner_player_id = owner_storage.get(entity).map(|o| o.player_id);
        let gold = gold_storage.get(entity).map(|g| g.0).unwrap_or(0);
        drop(identity_span);

        let stats_span = tracing::trace_span!(
            "omoba_core::runtime::extract_runtime.entity.stats_attack",
            perfetto = true,
            kind = ?kind,
        )
        .entered();
        let stats = UnitStats::from_refs(&*buff_store, is_building_storage.get(entity).is_some());
        let attack_range = tatk_storage
            .get(entity)
            .map(|a| {
                stats
                    .final_attack_range(a.range.v, entity)
                    .to_f32_for_render()
            })
            .unwrap_or(0.0);
        drop(stats_span);

        let hero_fields_span = tracing::trace_span!(
            "omoba_core::runtime::extract_runtime.entity.hero_fields",
            perfetto = true,
            is_hero = matches!(kind, EntityKind::Hero),
        )
        .entered();
        let (
            hero_name,
            hero_title,
            hero_level,
            hero_xp,
            hero_xp_next,
            hero_skill_points,
            hero_primary_attribute,
            hero_strength,
            hero_agility,
            hero_intelligence,
        ) = if let Some(h) = hero_storage.get(entity) {
            let attr = match h.primary_attribute {
                AttributeType::Strength => "力量",
                AttributeType::Agility => "敏捷",
                AttributeType::Intelligence => "智力",
            };
            (
                h.name.clone(),
                h.title.clone(),
                h.level,
                h.experience,
                h.experience_to_next,
                h.skill_points,
                attr.to_string(),
                h.strength,
                h.agility,
                h.intelligence,
            )
        } else {
            (
                String::new(),
                String::new(),
                0,
                0,
                0,
                0,
                String::new(),
                0,
                0,
                0,
            )
        };
        drop(hero_fields_span);

        let hero_ext_span = tracing::trace_span!(
            "omoba_core::runtime::extract_runtime.entity.hero_ext",
            perfetto = true,
            is_hero = matches!(kind, EntityKind::Hero),
        )
        .entered();
        let entity_buffs: Vec<BuffSnapshot> = buff_store
            .iter_for(entity)
            .map(|(id, entry)| BuffSnapshot {
                buff_id: id.to_string(),
                remaining_secs: buff_remaining_secs_for_snapshot(entry.remaining),
                payload_json: serde_json::to_string(&entry.payload).unwrap_or_default(),
            })
            .collect();
        let hero_ext = if matches!(kind, EntityKind::Hero) {
            let prop = cprop_storage.get(entity);
            let atk = tatk_storage.get(entity);

            let armor = prop
                .map(|p| stats.final_armor(p.def_physic, entity).to_f32_for_render())
                .unwrap_or(0.0);
            let magic_resist = prop
                .map(|p| {
                    stats
                        .final_magic_resist(p.def_magic, entity)
                        .to_f32_for_render()
                })
                .unwrap_or(0.0);
            let move_speed = prop
                .map(|p| stats.final_move_speed(p.msd, entity).to_f32_for_render())
                .unwrap_or(0.0);
            let attack_damage = atk
                .map(|a| stats.final_atk(a.atk_physic.v, entity).to_f32_for_render())
                .unwrap_or(0.0);
            let attack_speed_sec = atk
                .map(|a| {
                    let asd_mult = stats.final_attack_speed_mult(entity).to_f32_for_render();
                    let base = a.asd.v.to_f32_for_render();
                    if asd_mult > 0.0 {
                        base / asd_mult
                    } else {
                        base
                    }
                })
                .unwrap_or(0.0);
            let bullet_speed = atk
                .map(|a| a.bullet_speed.to_f32_for_render())
                .unwrap_or(0.0);
            let mana = 0.0_f32;
            let max_mana = 0.0_f32;

            let mut inventory: [Option<String>; 6] = Default::default();
            if let Some(inv) = inventory_storage.get(entity) {
                for (i, slot) in inv.slots.iter().enumerate().take(6) {
                    inventory[i] = slot.as_ref().map(|it| it.item_id.clone());
                }
            }

            let mut ability_ids: [Option<String>; 4] = Default::default();
            let mut ability_levels: [i32; 4] = [0; 4];
            if let Some(h) = hero_storage.get(entity) {
                for i in 0..4 {
                    if let Some(id) = h.abilities.get(i) {
                        let lvl = h.ability_levels.get(id).copied().unwrap_or(0);
                        ability_levels[i] = lvl;
                        ability_ids[i] = Some(id.clone());
                    }
                }
            }

            Some(Box::new(HeroStatsExt {
                armor,
                magic_resist,
                attack_damage,
                attack_range,
                move_speed,
                attack_speed_sec,
                bullet_speed,
                mana,
                max_mana,
                buffs: entity_buffs.clone(),
                inventory,
                ability_levels,
                ability_ids,
            }))
        } else {
            None
        };
        drop(hero_ext_span);

        let push_span = tracing::trace_span!(
            "omoba_core::runtime::extract_runtime.entity.push",
            perfetto = true,
            kind = ?kind,
        )
        .entered();
        let upgrade_levels: Option<[u8; 3]> = if matches!(kind, EntityKind::Tower) {
            tower_storage.get(entity).map(|t| t.upgrade_levels)
        } else {
            None
        };
        let tower_pops: Option<u32> = if matches!(kind, EntityKind::Tower) {
            tower_storage.get(entity).map(|t| t.pops)
        } else {
            None
        };
        let tower_atk: Option<f32> = if matches!(kind, EntityKind::Tower) {
            tatk_storage
                .get(entity)
                .map(|a| a.atk_physic.v.to_f32_for_render())
        } else {
            None
        };
        let tower_asd: Option<f32> = if matches!(kind, EntityKind::Tower) {
            tatk_storage
                .get(entity)
                .map(|a| a.asd.v.to_f32_for_render())
        } else {
            None
        };
        let tower_target_priority = tower_storage
            .get(entity)
            .map(|t| t.target_priority.as_str().to_string())
            .unwrap_or_default();
        let tower_active_ability =
            tower_active_ability_snapshot(tower_storage.get(entity), hp, &tower_upgrade_registry);
        let hero_command = hero_command_snapshot(
            command_queue_storage.get(entity),
            move_target_storage.get(entity),
            &pos_storage,
        );

        out.push(EntityRenderData {
            entity_id: entity.id(),
            entity_gen: entity.gen().id() as u32,
            spawn_order: tower_spawn_order_storage
                .get(entity)
                .map(|order| order.0)
                .unwrap_or(u64::MAX),
            kind,
            pos_x: px,
            pos_y: py,
            facing_rad: facing,
            hp,
            max_hp,
            unit_id,
            hero_name,
            hero_title,
            hero_level,
            hero_xp,
            hero_xp_next,
            hero_skill_points,
            hero_primary_attribute,
            hero_strength,
            hero_agility,
            hero_intelligence,
            gold,
            hero_ext,
            hero_render,
            projectile_owner_entity_id,
            owner_player_id,
            upgrade_levels,
            tower_pops,
            tower_atk,
            tower_asd,
            tower_target_priority,
            tower_active_ability,
            hero_command,
            buffs: entity_buffs,
            attack_range,
        });
        drop(push_span);
        drop(entity_span);
    }
    drop(entities_span);

    let paths: Vec<Vec<(f32, f32)>> = world
        .read_resource::<BTreeMap<String, CreepPath>>()
        .values()
        .map(|p| {
            p.check_points
                .iter()
                .map(|cp| (cp.pos.x, cp.pos.y))
                .collect()
        })
        .collect();

    let removed_entity_ids: Vec<u32> = {
        let mut q = world.write_resource::<RemovedEntitiesQueue>();
        std::mem::take(&mut q.pending)
    };

    let blocked_regions: Vec<BlockedRegionSnapshot> = world
        .read_resource::<BlockedRegions>()
        .0
        .iter()
        .map(|r| BlockedRegionSnapshot {
            points: r.points.iter().map(|p| (p.x, p.y)).collect(),
            circle: None,
        })
        .collect();

    let round: u32;
    let total_rounds: u32;
    let round_is_running: bool;
    {
        let ccw = world.read_resource::<CurrentCreepWave>();
        round = ccw.wave as u32;
        round_is_running = ccw.is_running;
    }
    {
        let waves = world.read_resource::<Vec<CreepWave>>();
        total_rounds = waves.len() as u32;
    }
    let lives = world.read_resource::<PlayerLives>().0;
    let is_paused = world.read_resource::<GamePause>().is_paused;
    let game_speed_multiplier = world
        .try_fetch::<GameSpeed>()
        .map(|speed| speed.multiplier())
        .unwrap_or(1);

    let explosions: Vec<ExplosionFx> = {
        let mut q = world.write_resource::<super::comp::ExplosionFxQueue>();
        std::mem::take(&mut q.pending)
    };
    let tower_fire_fx: Vec<TowerFireFx> = {
        let mut q = world.write_resource::<super::comp::TowerFireFxQueue>();
        std::mem::take(&mut q.pending)
    };
    let attack_phase_fx: Vec<AttackPhaseFx> = {
        let mut q = world.write_resource::<super::comp::AttackPhaseFxQueue>();
        std::mem::take(&mut q.pending)
    };
    let attack_cancel_fx: Vec<AttackCancelFx> = {
        let mut q = world.write_resource::<super::comp::AttackCancelFxQueue>();
        std::mem::take(&mut q.pending)
    };
    let script_visual_events = drain_script_visual_events(world);

    let snapshot = SimWorldSnapshot {
        tick,
        entities: out,
        paths,
        removed_entity_ids,
        round,
        total_rounds,
        lives,
        round_is_running,
        is_paused,
        game_speed_multiplier,
        blocked_regions,
        abilities: abilities_arc,
        tower_templates: tower_templates_arc,
        tower_upgrades: tower_upgrades_arc,
        explosions,
        tower_fire_fx,
        attack_phase_fx,
        attack_cancel_fx,
        script_visual_events,
        applied_input_ids,
        applied_input_meta,
        lua_content_generation: omoba_template_ids::runtime_lua_content_generation()
            .ok()
            .flatten()
            .unwrap_or(0),
        lua_content_hash: omoba_template_ids::runtime_lua_content_hash()
            .ok()
            .flatten()
            .unwrap_or_default(),
        dev_lua_reload_error: None,
        latest_tower_ability_cast_result: latest_tower_ability_cast_result(world),
        tower_ability_cast_results: tower_ability_cast_results(world),
    };
    drop(snapshot_span);
    snapshot
}

/// Extract data for render from the ECS world without rebuilding
/// initialization-only fields such as paths and static metadata registries.
pub fn extract_data_for_render(
    world: &mut World,
    tick: u32,
    applied_input_ids: Vec<u32>,
    applied_input_meta: Vec<AppliedInputMeta>,
) -> SimWorldSnapshot {
    let snapshot_span = tracing::trace_span!(
        "omoba_core::runtime::extract_data_for_render",
        perfetto = true,
        tick,
    )
    .entered();
    let storage_span = tracing::trace_span!(
        "omoba_core::runtime::extract_runtime.storage_borrows",
        perfetto = true,
    )
    .entered();
    let entities = world.entities();
    let pos_storage = world.read_storage::<Pos>();
    let facing_storage = world.read_storage::<Facing>();
    let cprop_storage = world.read_storage::<CProperty>();
    let hero_storage = world.read_storage::<Hero>();
    let tower_storage = world.read_storage::<Tower>();
    let tower_spawn_order_storage = world.read_storage::<TowerSpawnOrder>();
    let proj_storage = world.read_storage::<Projectile>();
    let creep_storage = world.read_storage::<Creep>();
    let unit_tag_storage = world.read_storage::<ScriptUnitTag>();
    let move_target_storage = world.read_storage::<MoveTarget>();
    let command_queue_storage = world.read_storage::<HeroCommandQueue>();
    let gold_storage = world.read_storage::<Gold>();
    let tatk_storage = world.read_storage::<TAttack>();
    let owner_storage = world.read_storage::<PlayerOwner>();
    let buff_store = world.read_resource::<BuffStore>();
    let is_building_storage = world.read_storage::<IsBuilding>();
    let inventory_storage = world.read_storage::<Inventory>();
    let tower_upgrade_registry = world.read_resource::<TowerUpgradeRegistry>();
    drop(storage_span);

    let mut out = Vec::new();
    let entities_span = tracing::trace_span!(
        "omoba_core::runtime::extract_runtime.entities",
        perfetto = true,
    )
    .entered();
    for (entity, pos) in (&entities, &pos_storage).join() {
        let kind = if hero_storage.get(entity).is_some() {
            EntityKind::Hero
        } else if tower_storage.get(entity).is_some() {
            EntityKind::Tower
        } else if proj_storage.get(entity).is_some() {
            EntityKind::Projectile
        } else if creep_storage.get(entity).is_some() {
            EntityKind::Creep
        } else {
            EntityKind::Other
        };

        let facing = facing_storage
            .get(entity)
            .map(|f| {
                (f.0.ticks() as f32) / (omoba_sim::trig::TAU_TICKS as f32)
                    * 2.0
                    * std::f32::consts::PI
            })
            .unwrap_or(0.0);

        let (hp, max_hp) = cprop_storage
            .get(entity)
            .map(|c| {
                (
                    c.hp.to_f32_for_render() as i32,
                    c.mhp.to_f32_for_render() as i32,
                )
            })
            .unwrap_or((0, 0));

        let (px, py) = pos.xy_f32();
        let unit_id = unit_tag_storage
            .get(entity)
            .map(|t| t.unit_id.clone())
            .unwrap_or_default();
        let hero_render = if matches!(kind, EntityKind::Hero) {
            hero_render_snapshot_for_unit_id(
                &unit_id,
                move_target_storage.get(entity).is_some(),
                buff_store.has(entity, "sniper_mode"),
            )
        } else {
            None
        };
        let projectile_owner_entity_id = proj_storage.get(entity).map(|p| p.owner.id());
        let owner_player_id = owner_storage.get(entity).map(|o| o.player_id);
        let gold = gold_storage.get(entity).map(|g| g.0).unwrap_or(0);
        let stats = UnitStats::from_refs(&*buff_store, is_building_storage.get(entity).is_some());
        let attack_range = tatk_storage
            .get(entity)
            .map(|a| {
                stats
                    .final_attack_range(a.range.v, entity)
                    .to_f32_for_render()
            })
            .unwrap_or(0.0);

        let (
            hero_name,
            hero_title,
            hero_level,
            hero_xp,
            hero_xp_next,
            hero_skill_points,
            hero_primary_attribute,
            hero_strength,
            hero_agility,
            hero_intelligence,
        ) = if let Some(h) = hero_storage.get(entity) {
            let attr = match h.primary_attribute {
                AttributeType::Strength => "力量",
                AttributeType::Agility => "敏捷",
                AttributeType::Intelligence => "智力",
            };
            (
                h.name.clone(),
                h.title.clone(),
                h.level,
                h.experience,
                h.experience_to_next,
                h.skill_points,
                attr.to_string(),
                h.strength,
                h.agility,
                h.intelligence,
            )
        } else {
            (
                String::new(),
                String::new(),
                0,
                0,
                0,
                0,
                String::new(),
                0,
                0,
                0,
            )
        };

        let entity_buffs: Vec<BuffSnapshot> = buff_store
            .iter_for(entity)
            .map(|(id, entry)| BuffSnapshot {
                buff_id: id.to_string(),
                remaining_secs: buff_remaining_secs_for_snapshot(entry.remaining),
                payload_json: serde_json::to_string(&entry.payload).unwrap_or_default(),
            })
            .collect();

        let hero_ext = if matches!(kind, EntityKind::Hero) {
            let prop = cprop_storage.get(entity);
            let atk = tatk_storage.get(entity);

            let armor = prop
                .map(|p| stats.final_armor(p.def_physic, entity).to_f32_for_render())
                .unwrap_or(0.0);
            let magic_resist = prop
                .map(|p| {
                    stats
                        .final_magic_resist(p.def_magic, entity)
                        .to_f32_for_render()
                })
                .unwrap_or(0.0);
            let move_speed = prop
                .map(|p| stats.final_move_speed(p.msd, entity).to_f32_for_render())
                .unwrap_or(0.0);
            let attack_damage = atk
                .map(|a| stats.final_atk(a.atk_physic.v, entity).to_f32_for_render())
                .unwrap_or(0.0);
            let attack_speed_sec = atk
                .map(|a| {
                    let asd_mult = stats.final_attack_speed_mult(entity).to_f32_for_render();
                    let base = a.asd.v.to_f32_for_render();
                    if asd_mult > 0.0 {
                        base / asd_mult
                    } else {
                        base
                    }
                })
                .unwrap_or(0.0);
            let bullet_speed = atk
                .map(|a| a.bullet_speed.to_f32_for_render())
                .unwrap_or(0.0);
            let mana = 0.0_f32;
            let max_mana = 0.0_f32;

            let mut inventory: [Option<String>; 6] = Default::default();
            if let Some(inv) = inventory_storage.get(entity) {
                for (i, slot) in inv.slots.iter().enumerate().take(6) {
                    inventory[i] = slot.as_ref().map(|it| it.item_id.clone());
                }
            }

            let mut ability_ids: [Option<String>; 4] = Default::default();
            let mut ability_levels: [i32; 4] = [0; 4];
            if let Some(h) = hero_storage.get(entity) {
                for i in 0..4 {
                    if let Some(id) = h.abilities.get(i) {
                        let lvl = h.ability_levels.get(id).copied().unwrap_or(0);
                        ability_levels[i] = lvl;
                        ability_ids[i] = Some(id.clone());
                    }
                }
            }

            Some(Box::new(HeroStatsExt {
                armor,
                magic_resist,
                attack_damage,
                attack_range,
                move_speed,
                attack_speed_sec,
                bullet_speed,
                mana,
                max_mana,
                buffs: entity_buffs.clone(),
                inventory,
                ability_levels,
                ability_ids,
            }))
        } else {
            None
        };

        let upgrade_levels: Option<[u8; 3]> = if matches!(kind, EntityKind::Tower) {
            tower_storage.get(entity).map(|t| t.upgrade_levels)
        } else {
            None
        };
        let tower_pops: Option<u32> = if matches!(kind, EntityKind::Tower) {
            tower_storage.get(entity).map(|t| t.pops)
        } else {
            None
        };
        let tower_atk: Option<f32> = if matches!(kind, EntityKind::Tower) {
            tatk_storage
                .get(entity)
                .map(|a| a.atk_physic.v.to_f32_for_render())
        } else {
            None
        };
        let tower_asd: Option<f32> = if matches!(kind, EntityKind::Tower) {
            tatk_storage
                .get(entity)
                .map(|a| a.asd.v.to_f32_for_render())
        } else {
            None
        };
        let tower_target_priority = tower_storage
            .get(entity)
            .map(|t| t.target_priority.as_str().to_string())
            .unwrap_or_default();
        let tower_active_ability =
            tower_active_ability_snapshot(tower_storage.get(entity), hp, &tower_upgrade_registry);
        let hero_command = hero_command_snapshot(
            command_queue_storage.get(entity),
            move_target_storage.get(entity),
            &pos_storage,
        );

        out.push(EntityRenderData {
            entity_id: entity.id(),
            entity_gen: entity.gen().id() as u32,
            spawn_order: tower_spawn_order_storage
                .get(entity)
                .map(|order| order.0)
                .unwrap_or(u64::MAX),
            kind,
            pos_x: px,
            pos_y: py,
            facing_rad: facing,
            hp,
            max_hp,
            unit_id,
            hero_name,
            hero_title,
            hero_level,
            hero_xp,
            hero_xp_next,
            hero_skill_points,
            hero_primary_attribute,
            hero_strength,
            hero_agility,
            hero_intelligence,
            gold,
            hero_ext,
            hero_render,
            projectile_owner_entity_id,
            owner_player_id,
            upgrade_levels,
            tower_pops,
            tower_atk,
            tower_asd,
            tower_target_priority,
            tower_active_ability,
            hero_command,
            buffs: entity_buffs,
            attack_range,
        });
    }
    drop(entities_span);

    let queues_span = tracing::trace_span!(
        "omoba_core::runtime::extract_runtime.queues",
        perfetto = true,
        entities = out.len(),
    )
    .entered();
    let removed_entity_ids: Vec<u32> = {
        let mut q = world.write_resource::<RemovedEntitiesQueue>();
        std::mem::take(&mut q.pending)
    };

    let round: u32;
    let total_rounds: u32;
    let round_is_running: bool;
    {
        let ccw = world.read_resource::<CurrentCreepWave>();
        round = ccw.wave as u32;
        round_is_running = ccw.is_running;
    }
    {
        let waves = world.read_resource::<Vec<CreepWave>>();
        total_rounds = waves.len() as u32;
    }
    let lives = world.read_resource::<PlayerLives>().0;
    let is_paused = world.read_resource::<GamePause>().is_paused;
    let game_speed_multiplier = world
        .try_fetch::<GameSpeed>()
        .map(|speed| speed.multiplier())
        .unwrap_or(1);

    let explosions: Vec<ExplosionFx> = {
        let mut q = world.write_resource::<super::comp::ExplosionFxQueue>();
        std::mem::take(&mut q.pending)
    };
    let tower_fire_fx: Vec<TowerFireFx> = {
        let mut q = world.write_resource::<super::comp::TowerFireFxQueue>();
        std::mem::take(&mut q.pending)
    };
    let attack_phase_fx: Vec<AttackPhaseFx> = {
        let mut q = world.write_resource::<super::comp::AttackPhaseFxQueue>();
        std::mem::take(&mut q.pending)
    };
    let attack_cancel_fx: Vec<AttackCancelFx> = {
        let mut q = world.write_resource::<super::comp::AttackCancelFxQueue>();
        std::mem::take(&mut q.pending)
    };
    let script_visual_events = drain_script_visual_events(world);
    drop(queues_span);

    let assemble_span = tracing::trace_span!(
        "omoba_core::runtime::extract_runtime.assemble",
        perfetto = true,
        entities = out.len(),
        removed = removed_entity_ids.len(),
        explosions = explosions.len(),
        tower_fire_fx = tower_fire_fx.len(),
        attack_phase_fx = attack_phase_fx.len(),
        attack_cancel_fx = attack_cancel_fx.len(),
    )
    .entered();
    let snapshot = SimWorldSnapshot {
        tick,
        entities: out,
        paths: Vec::new(),
        removed_entity_ids,
        round,
        total_rounds,
        lives,
        round_is_running,
        is_paused,
        game_speed_multiplier,
        blocked_regions: Vec::new(),
        abilities: Arc::new(Vec::new()),
        tower_templates: Arc::new(Vec::new()),
        tower_upgrades: Arc::new(Vec::new()),
        explosions,
        tower_fire_fx,
        attack_phase_fx,
        attack_cancel_fx,
        script_visual_events,
        applied_input_ids,
        applied_input_meta,
        lua_content_generation: omoba_template_ids::runtime_lua_content_generation()
            .ok()
            .flatten()
            .unwrap_or(0),
        lua_content_hash: omoba_template_ids::runtime_lua_content_hash()
            .ok()
            .flatten()
            .unwrap_or_default(),
        dev_lua_reload_error: None,
        latest_tower_ability_cast_result: latest_tower_ability_cast_result(world),
        tower_ability_cast_results: tower_ability_cast_results(world),
    };
    drop(assemble_span);
    drop(snapshot_span);
    snapshot
}
