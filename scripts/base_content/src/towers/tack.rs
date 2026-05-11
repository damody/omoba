//! Tack Shooter — 近戰放射針塔，MVP 支援 12 升級 flag / stat。
//!
//! 支援:
//! - 路徑1：needles_12/needles_16/needles_32，blade_shooter（命中半徑110，傷害≥20）
//! - Path2: ring_of_fire (每射一次塔周 200 radius / 20 dmg magical),
//!   inferno_ring (同半徑 / 50 dmg)
//! - Stat: damage_bonus, range_bonus (透過 get_final_*)
//!
//! 待辦事項：
//! - burn_tier1 / burn_tier2: 需 DoT 系統（Task 15）才能真正掛 burn buff。
//!   這裡先暫不處理命中後加 burn。

use omb_script_abi::prelude::*;

pub struct TackTower;

// 數值唯一來源：scripts/lua_data/templates.lua → omoba_template_ids 編譯期生成
// `TOWER_TACK_STATS`。hit_radius 80 須與 host 端 `comp::TACK_NEEDLE_HIT_RADIUS` 同步。
const STATS: &TowerStats = &TOWER_TACK_STATS;

impl UnitScript for TackTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str(TOWER_TACK.as_str())
    }

    fn on_spawn(&self, e: EntityHandle, w: &mut GameWorldDyn<'_>) {
        let stats = super::tower_stats(TOWER_TACK, STATS);
        w.set_tower_atk(e, stats.atk);
        w.set_tower_range(e, stats.range);
        w.set_asd_interval(e, stats.asd_interval);
    }

    fn tower_metadata(&self) -> ROption<TowerMetadata> {
        RSome(super::tower_metadata_from_consts(
            TOWER_TACK,
            STATS,
            &TOWER_TACK_RENDER,
            TOWER_TACK_ATTACK_TIMING,
        ))
    }

    fn on_tick(&self, e: EntityHandle, dt: Fixed64, w: &mut GameWorldDyn<'_>) {
        let asd_interval = w.get_asd_interval(e);
        if asd_interval <= Fixed64::ZERO {
            return;
        }
        let stats = super::tower_stats(TOWER_TACK, STATS);
        let timing = super::tower_attack_timing(TOWER_TACK, TOWER_TACK_ATTACK_TIMING);
        let phase = super::advance_attack_phase(e, dt, asd_interval, timing, w);
        if matches!(phase, super::AttackPhaseStep::Charging) {
            return;
        }

        let pos = match w.get_pos(e) {
            RSome(p) => p,
            RNone => return,
        };
        let range = w.get_final_attack_range(e);
        // Tack 不鎖定單一目標，只要射程內有敵就開火
        if matches!(w.query_nearest_enemy(pos, range, e), RNone) {
            return;
        }
        if matches!(phase, super::AttackPhaseStep::Ready) {
            super::start_attack_windup(e, asd_interval, timing, Target::None, w);
            return;
        }

        let atk = w.get_final_atk(e);

        // 針數 + blade
        let blade = w.has_tower_flag(e, RStr::from_str("blade_shooter"));
        let needle_count: u32 = if w.has_tower_flag(e, RStr::from_str("needles_32")) {
            32
        } else if w.has_tower_flag(e, RStr::from_str("needles_16")) {
            16
        } else if w.has_tower_flag(e, RStr::from_str("needles_12")) {
            12
        } else {
            8
        };

        let (hit_radius, damage) = if blade {
            let twenty = Fixed64::from_i32(20);
            let dmg = if atk > twenty { atk } else { twenty };
            (Fixed64::from_i32(110), dmg)
        } else {
            (stats.hit_radius, atk)
        };

        w.log_info(RStr::from_str("[tower_tack] fire needles!"));

        let step_deg: i32 = 360 / (needle_count as i32);
        for i in 0..needle_count {
            let angle = omoba_sim::trig::Angle::from_degrees_i32(step_deg * (i as i32));
            let end = Vec2 {
                x: pos.x + omoba_sim::trig::cos(angle) * range,
                y: pos.y + omoba_sim::trig::sin(angle) * range,
            };
            w.spawn_projectile_ex(ProjectileSpec {
                from: pos,
                owner: e,
                path: PathSpec::Straight { end_pos: end },
                speed: stats.bullet_speed,
                damage,
                hit_radius,
                splash_radius: Fixed64::ZERO,
                slow_factor: Fixed64::ZERO,
                slow_duration: Fixed64::ZERO,
                stun_duration: Fixed64::ZERO,
                kind_id: if blade {
                    PROJECTILE_TACK_BLADE.0
                } else {
                    PROJECTILE_TACK.0
                },
            });
        }

        // Ring of Fire / Inferno Ring：每次發射時塔周 AoE
        let inferno = w.has_tower_flag(e, RStr::from_str("inferno_ring"));
        let ring = inferno || w.has_tower_flag(e, RStr::from_str("ring_of_fire"));
        if ring {
            let (r, dmg) = if inferno {
                (Fixed64::from_i32(200), Fixed64::from_i32(50))
            } else {
                (Fixed64::from_i32(200), Fixed64::from_i32(20))
            };
            w.deal_damage_splash(pos, r, dmg, DamageKind::Magical, RSome(e));
            w.play_vfx(RStr::from_str("vfx_ring_of_fire"), pos);
        }

        // TODO burn_tier1 / burn_tier2: 需 DoT 系統 (Task 15) 才能對命中 target 掛 burn buff。
    }
}
