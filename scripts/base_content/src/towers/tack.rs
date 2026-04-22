//! Tack Shooter — 近戰放射針塔。
//!
//! 由 `on_tick` 驅動：射程內有敵人就發射；8 根黑針以 45° 等分角度向外直線飛，
//! 途中第一個碰到的敵人被針扣血即消失。
//! 升級版本可把 `NEEDLE_COUNT` 改為 16。

use omb_script_abi::prelude::*;

pub struct TackTower;

const ATK: f32 = 8.0;
const ASD_INTERVAL: f32 = 1.2;
const RANGE: f32 = 380.0;
const BULLET_SPEED: f32 = 1400.0;
const NEEDLE_COUNT: u32 = 8;
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
        let range = w.get_tower_range(e);
        // Tack 不鎖定單一目標，只要射程內有敵就開火
        if matches!(w.query_nearest_enemy(pos, range, e), RNone) {
            return;
        }

        w.set_asd_count(e, asd_count - asd_interval);

        let atk = w.get_tower_atk(e);
        w.log_info(RStr::from_str("[tower_tack] fire 8 needles!"));

        let step = core::f32::consts::TAU / (NEEDLE_COUNT as f32);
        for i in 0..NEEDLE_COUNT {
            let angle = step * (i as f32);
            let dx = angle.cos();
            let dy = angle.sin();
            let end = Vec2f::new(pos.x + dx * range, pos.y + dy * range);
            w.spawn_projectile_ex(ProjectileSpec {
                from: pos,
                owner: e,
                path: PathSpec::Straight { end_pos: end },
                speed: BULLET_SPEED,
                damage: atk,
                hit_radius: HIT_RADIUS,
                splash_radius: 0.0,
                slow_factor: 0.0,
                slow_duration: 0.0,
                kind_tag: RString::from("tack"),
            });
        }
    }
}
