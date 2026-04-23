//! Tack Shooter — 近戰放射針塔，MVP 支援 12 升級 flag / stat。
//!
//! 支援:
//! - Path1: needles_12 / needles_16 / needles_32, blade_shooter (hit_radius 110, dmg ≥ 20)
//! - Path2: ring_of_fire (每射一次塔周 200 radius / 20 dmg magical),
//!   inferno_ring (同半徑 / 50 dmg)
//! - Stat: damage_bonus, range_bonus (透過 get_final_*)
//!
//! TODO:
//! - burn_tier1 / burn_tier2: 需 DoT 系統（Task 15）才能真正掛 burn buff。
//!   這裡先暫不處理命中後加 burn。

use omb_script_abi::prelude::*;

pub struct TackTower;

const ATK: f32 = 8.0;
const ASD_INTERVAL: f32 = 1.2;
const RANGE: f32 = 380.0;
const BULLET_SPEED: f32 = 1400.0;
const HIT_RADIUS: f32 = 80.0; // 與 host 端 comp::TACK_NEEDLE_HIT_RADIUS 同步

impl UnitScript for TackTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str("tower_tack")
    }

    fn on_spawn(&self, e: EntityHandle, w: &mut GameWorldDyn<'_>) {
        w.set_tower_atk(e, ATK);
        w.set_tower_range(e, RANGE);
        w.set_asd_interval(e, ASD_INTERVAL);
    }

    fn tower_metadata(&self) -> ROption<TowerMetadata> {
        RSome(TowerMetadata {
            atk: ATK,
            asd_interval: ASD_INTERVAL,
            range: RANGE,
            bullet_speed: BULLET_SPEED,
            splash_radius: 0.0,
            hit_radius: HIT_RADIUS,
            slow_factor: 0.0,
            slow_duration: 0.0,
            cost: 400,
            footprint: 40.0,
            hp: 1.0,
            turn_speed_deg: 3600.0,
            label: RString::from("Tack Shooter"),
        })
    }

    fn on_tick(&self, e: EntityHandle, dt: f32, w: &mut GameWorldDyn<'_>) {
        let asd_interval = w.get_asd_interval(e);
        if asd_interval <= 0.0 {
            return;
        }
        let mut asd_count = w.get_asd_count(e);
        if asd_count < asd_interval {
            asd_count += dt;
            w.set_asd_count(e, asd_count);
        }
        if asd_count < asd_interval {
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

        w.set_asd_count(e, asd_count - asd_interval);

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
            (110.0_f32, atk.max(20.0))
        } else {
            (HIT_RADIUS, atk)
        };

        w.log_info(RStr::from_str("[tower_tack] fire needles!"));

        let step = core::f32::consts::TAU / (needle_count as f32);
        for i in 0..needle_count {
            let angle = step * (i as f32);
            let end = Vec2f::new(
                pos.x + angle.cos() * range,
                pos.y + angle.sin() * range,
            );
            w.spawn_projectile_ex(ProjectileSpec {
                from: pos,
                owner: e,
                path: PathSpec::Straight { end_pos: end },
                speed: BULLET_SPEED,
                damage,
                hit_radius,
                splash_radius: 0.0,
                slow_factor: 0.0,
                slow_duration: 0.0,
                stun_duration: 0.0,
                kind_tag: RString::from(if blade { "tack_blade" } else { "tack" }),
            });
        }

        // Ring of Fire / Inferno Ring：每次發射時塔周 AoE
        let inferno = w.has_tower_flag(e, RStr::from_str("inferno_ring"));
        let ring = inferno || w.has_tower_flag(e, RStr::from_str("ring_of_fire"));
        if ring {
            let (r, dmg) = if inferno {
                (200.0_f32, 50.0_f32)
            } else {
                (200.0_f32, 20.0_f32)
            };
            w.deal_damage_splash(pos, r, dmg, DamageKind::Magical, RSome(e));
            w.play_vfx(RStr::from_str("vfx_ring_of_fire"), pos);
        }

        // TODO burn_tier1 / burn_tier2: 需 DoT 系統 (Task 15) 才能對命中 target 掛 burn buff。
    }
}
