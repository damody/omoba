pub mod arty;
pub mod bomb;
pub mod boomerang;
pub mod cake_splash;
pub mod dart;
pub mod ice;
pub mod tack;

use omb_script_abi::prelude::*;

pub enum AttackPhaseStep {
    Charging,
    Ready,
    Impact,
}

pub fn attack_phase_durations(interval: Fixed64, timing: AttackTimingConst) -> (Fixed64, Fixed64) {
    let windup = interval * Fixed64::from_i32(timing.windup as i32) / Fixed64::from_i32(1000);
    let backswing = interval - windup;
    (windup, backswing)
}

pub fn advance_attack_phase(
    e: EntityHandle,
    dt: Fixed64,
    interval: Fixed64,
    timing: AttackTimingConst,
    w: &mut GameWorldDyn<'_>,
) -> AttackPhaseStep {
    let (windup, _) = attack_phase_durations(interval, timing);
    let mut asd_count = w.get_asd_count(e);
    if asd_count < Fixed64::ZERO {
        asd_count += dt;
        if asd_count < Fixed64::ZERO {
            w.set_asd_count(e, asd_count);
            return AttackPhaseStep::Charging;
        }
        w.set_asd_count(e, windup + asd_count);
        return AttackPhaseStep::Impact;
    }

    if asd_count < interval {
        asd_count += dt;
        w.set_asd_count(e, asd_count);
    }
    if asd_count >= interval {
        AttackPhaseStep::Ready
    } else {
        AttackPhaseStep::Charging
    }
}

pub fn start_attack_windup(
    e: EntityHandle,
    interval: Fixed64,
    timing: AttackTimingConst,
    target: Target,
    w: &mut GameWorldDyn<'_>,
) {
    let (windup, backswing) = attack_phase_durations(interval, timing);
    let over = {
        let count = w.get_asd_count(e) - interval;
        if count > Fixed64::ZERO {
            count
        } else {
            Fixed64::ZERO
        }
    };
    w.set_asd_count(e, over - windup);
    w.emit_attack_phase_fx(
        e,
        target,
        fixed_secs_to_ms(windup),
        fixed_secs_to_ms(backswing),
    );
}

pub fn fixed_secs_to_ms(value: Fixed64) -> u32 {
    (value.to_f32_for_render() * 1000.0).clamp(0.0, u32::MAX as f32) as u32
}

pub fn tower_stats(id: TowerId, fallback: &'static TowerStats) -> &'static TowerStats {
    omoba_template_ids::active_tower_stats(id).unwrap_or(fallback)
}

pub fn tower_attack_timing(id: TowerId, fallback: AttackTimingConst) -> AttackTimingConst {
    omoba_template_ids::active_tower_attack_timing(id).unwrap_or(fallback)
}

pub fn tower_metadata_from_consts(
    id: TowerId,
    stats: &TowerStats,
    render: &TowerRenderMetadataConst,
    attack_timing: AttackTimingConst,
) -> TowerMetadata {
    let stats = omoba_template_ids::active_tower_stats(id).unwrap_or(stats);
    let render = omoba_template_ids::active_tower_render_metadata(id).unwrap_or(render);
    let attack_timing = omoba_template_ids::active_tower_attack_timing(id).unwrap_or(attack_timing);
    TowerMetadata {
        atk: stats.atk,
        asd_interval: stats.asd_interval,
        range: stats.range,
        bullet_speed: stats.bullet_speed,
        splash_radius: stats.splash_radius,
        hit_radius: stats.hit_radius,
        slow_factor: stats.slow_factor,
        slow_duration: stats.slow_duration,
        cost: stats.cost,
        footprint: stats.footprint,
        placement_radius: stats.placement_radius,
        hp: stats.hp,
        turn_speed_deg: stats.turn_speed_deg,
        label: RString::from(omoba_template_ids::active_tower_display(id)),
        render: render_metadata_from_const(render),
        attack_timing: AttackTimingMetadata {
            windup: attack_timing.windup,
            backswing: attack_timing.backswing,
        },
    }
}

fn render_metadata_from_const(src: &TowerRenderMetadataConst) -> TowerRenderMetadata {
    TowerRenderMetadata {
        render_mode: RString::from(render_mode_name(src.render_mode)),
        base: RString::from(src.base),
        barrel: RString::from(src.barrel),
        visual_size: src.visual_size,
        barrel_frames: rstrings(src.barrel_frames),
        body_frames: rstrings(src.body_frames),
        barrel_animation: render_animation_from_const(src.barrel_animation),
        body_animation: render_animation_from_const(src.body_animation),
        rotation_mode: RString::from(rotation_mode_name(src.rotation_mode)),
        barrel_layout: RString::from(barrel_layout_name(src.barrel_layout)),
        barrel_variants: barrel_variants_from_const(src.barrel_variants),
        barrel_offset: render_point_from_const(src.barrel_offset),
        barrel_pivot: render_point_from_const(src.barrel_pivot),
        muzzle_offset: render_point_from_const(src.muzzle_offset),
        default_angle_deg: src.default_angle_deg,
        recoil: TowerRecoil {
            mode: RString::from(recoil_mode_name(src.recoil.mode)),
            distance: src.recoil.distance,
            scale: src.recoil.scale,
            duration_ms: src.recoil.duration_ms,
            return_ms: src.recoil.return_ms,
        },
    }
}

fn rstrings(values: &'static [&'static str]) -> RVec<RString> {
    let mut out = RVec::new();
    for value in values {
        out.push(RString::from(*value));
    }
    out
}

fn barrel_variants_from_const(
    values: &'static [TowerBarrelVariantConst],
) -> RVec<TowerBarrelVariant> {
    let mut out = RVec::new();
    for value in values {
        out.push(TowerBarrelVariant {
            min_path: value.min_path,
            min_level: value.min_level,
            count: value.count,
            image: RString::from(value.image),
            frames: rstrings(value.frames),
        });
    }
    out
}

fn render_point_from_const(src: TowerRenderPointConst) -> TowerRenderPoint {
    TowerRenderPoint { x: src.x, y: src.y }
}

fn render_animation_from_const(src: TowerRenderAnimationConst) -> TowerRenderAnimation {
    TowerRenderAnimation {
        fps: src.fps,
        loop_animation: src.loop_animation,
        fire_fps: src.fire_fps,
        fire_once: src.fire_once,
    }
}

fn render_mode_name(value: TowerRenderModeC) -> &'static str {
    match value {
        TowerRenderModeC::BaseBarrel => "base_barrel",
        TowerRenderModeC::AnimatedArea => "animated_area",
    }
}

fn rotation_mode_name(value: TowerRotationModeC) -> &'static str {
    match value {
        TowerRotationModeC::Targeted => "targeted",
        TowerRotationModeC::Fixed => "fixed",
    }
}

fn barrel_layout_name(value: TowerBarrelLayoutC) -> &'static str {
    match value {
        TowerBarrelLayoutC::Single => "single",
        TowerBarrelLayoutC::RadialCountVariants => "radial_count_variants",
    }
}

fn recoil_mode_name(value: TowerRecoilModeC) -> &'static str {
    match value {
        TowerRecoilModeC::Directional => "directional",
        TowerRecoilModeC::ScalePulse => "scale_pulse",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attack_phase_durations_preserve_interval_sum() {
        let timing = AttackTimingConst {
            windup: 350,
            backswing: 650,
        };
        let interval = Fixed64::from_i32(1);
        let (windup, backswing) = attack_phase_durations(interval, timing);
        assert_eq!(windup + backswing, interval);
        assert_eq!(windup, Fixed64::from_raw(358));
        assert_eq!(backswing, interval - windup);
    }

    #[test]
    fn faster_attack_speed_shortens_both_phases() {
        let timing = AttackTimingConst {
            windup: 350,
            backswing: 650,
        };
        let full_interval = Fixed64::from_i32(1);
        let half_interval = Fixed64::from_raw(512);
        let (full_windup, full_backswing) = attack_phase_durations(full_interval, timing);
        let (half_windup, half_backswing) = attack_phase_durations(half_interval, timing);
        assert!(half_windup < full_windup);
        assert!(half_backswing < full_backswing);
        assert_eq!(half_windup + half_backswing, half_interval);
    }
}

#[cfg(test)]
pub(crate) mod projectile_test_support {
    use super::*;
    use abi_stable::{sabi_trait::prelude::TD_Opaque, RMut, RRef};
    use omb_script_abi::{
        script::UnitScript,
        types::{EntityHandle, ProjectileHitContext},
        world::{GameWorld_TO, ProjectileQuery_TO, TowerCooldownAccess_TO},
    };
    use omoba_core::runtime::BuffStore;
    use omoba_core::scripting::ScriptUnitTag;
    use omoba_core::{
        scripting::parallel_world_adapter::{
            ParallelAdapterCache, ParallelProjectileQuery, ParallelTowerCooldownAccess,
            ParallelWorldAdapter,
        },
        BlockedRegions, CProperty, CollisionRadius, Creep, Facing, Faction, Hero, IsBuilding,
        Outcome, PlayerOwner, Pos, Searcher, TAttack, Tick, Tower, Unit,
    };
    use specs::{Builder, World, WorldExt};

    pub struct Fixture {
        pub world: World,
        pub tower: specs::Entity,
        pub enemies: Vec<specs::Entity>,
    }

    pub fn fixture(flags: &[&str], enemy_positions: &[Vec2]) -> Fixture {
        let mut world = World::new();
        world.register::<TAttack>();
        world.register::<Pos>();
        world.register::<Facing>();
        world.register::<CProperty>();
        world.register::<Unit>();
        world.register::<Hero>();
        world.register::<Faction>();
        world.register::<Creep>();
        world.register::<Tower>();
        world.register::<IsBuilding>();
        world.register::<CollisionRadius>();
        world.register::<ScriptUnitTag>();
        world.register::<PlayerOwner>();
        world.insert(BuffStore::default());
        world.insert(Searcher::default());
        world.insert(BlockedRegions::default());
        world.insert(Tick(1));

        let mut tower_data = Tower::new();
        tower_data.upgrade_flags = flags.iter().map(|flag| (*flag).to_string()).collect();
        let tower = world
            .create_entity()
            .with(Pos(Vec2::ZERO))
            .with(Facing(Angle::ZERO))
            .with(Faction::new(omoba_core::FactionType::Player, 0))
            .with(tower_data)
            .with(TAttack::new(
                Fixed64::from_i32(10),
                Fixed64::ONE,
                Fixed64::from_i32(500),
                Fixed64::from_i32(1000),
            ))
            .build();
        let enemies: Vec<_> = enemy_positions
            .iter()
            .copied()
            .map(|pos| {
                world
                    .create_entity()
                    .with(Pos(pos))
                    .with(Faction::new(omoba_core::FactionType::Enemy, 1))
                    .with(Creep {
                        name: "test".to_string(),
                        label: None,
                        path: "test".to_string(),
                        pidx: 0,
                        path_remaining_distance: Fixed64::ZERO,
                        block_tower: None,
                        status: omoba_core::CreepStatus::Walk,
                    })
                    .with(CProperty {
                        hp: Fixed64::from_i32(100),
                        mhp: Fixed64::from_i32(100),
                        msd: Fixed64::ZERO,
                        def_physic: Fixed64::ZERO,
                        def_magic: Fixed64::ZERO,
                    })
                    .build()
            })
            .collect();
        let creep_index = enemies
            .iter()
            .zip(enemy_positions.iter())
            .map(|(entity, pos)| {
                (
                    *entity,
                    vek::Vec2::new(pos.x.to_f32_for_render(), pos.y.to_f32_for_render()),
                )
            })
            .collect::<Vec<_>>();
        world
            .write_resource::<Searcher>()
            .creep
            .rebuild_from(creep_index);
        Fixture {
            world,
            tower,
            enemies,
        }
    }

    pub fn invoke(
        fixture: &World,
        script: &impl UnitScript,
        tower: specs::Entity,
        victim: specs::Entity,
        context: ProjectileHitContext,
    ) -> Vec<Outcome> {
        let cache = ParallelAdapterCache::new(fixture, 1);
        let mut adapter = ParallelWorldAdapter::new(&cache, tower);
        adapter.set_projectile_hit_generation(context.generation);
        let query_adapter = ParallelProjectileQuery::new(&cache);
        let query_dyn = ProjectileQuery_TO::from_ptr(RRef::new(&query_adapter), TD_Opaque);
        let mut world_dyn = GameWorld_TO::from_ptr(RMut::new(&mut adapter), TD_Opaque);
        script.on_projectile_hit(
            EntityHandle {
                id: tower.id(),
                gen: tower.gen().id() as u32,
            },
            EntityHandle {
                id: victim.id(),
                gen: victim.gen().id() as u32,
            },
            context,
            &query_dyn,
            &mut world_dyn,
        );
        drop(world_dyn);
        adapter.finish()
    }

    pub fn invoke_tick(
        fixture: &World,
        script: &impl UnitScript,
        tower: specs::Entity,
        dt: Fixed64,
    ) -> Vec<Outcome> {
        let cache = ParallelAdapterCache::new(fixture, 1);
        let mut adapter = ParallelWorldAdapter::new(&cache, tower);
        let mut cooldown_adapter = ParallelTowerCooldownAccess::new(&cache);
        let mut world_dyn = GameWorld_TO::from_ptr(RMut::new(&mut adapter), TD_Opaque);
        let mut cooldown_dyn =
            TowerCooldownAccess_TO::from_ptr(RMut::new(&mut cooldown_adapter), TD_Opaque);
        script.on_tower_tick(
            EntityHandle {
                id: tower.id(),
                gen: tower.gen().id() as u32,
            },
            dt,
            &mut cooldown_dyn,
            &mut world_dyn,
        );
        drop(cooldown_dyn);
        drop(world_dyn);
        let mut outcomes = adapter.finish();
        outcomes.extend(cooldown_adapter.finish());
        outcomes
    }
}
