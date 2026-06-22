pub mod arty;
pub mod bomb;
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
