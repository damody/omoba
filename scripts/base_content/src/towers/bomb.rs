//! Bomb Shooter — AoE 塔：射出引信彈、命中後以 splash_radius 掃半徑內敵人。
//!
//! 由 `on_tick` 主動驅動：找最近敵人 → 發 homing + splash。
//! 爆炸紅圈特效由 `projectile_tick` 在 `proj.radius > 1.0` 命中時自動 `emit_explosion`。

use omb_script_abi::prelude::*;

pub struct BombTower;

const ATK: f32 = 30.0;
const ASD_INTERVAL: f32 = 1.5;
const RANGE: f32 = 400.0;
const BULLET_SPEED: f32 = 900.0;
const SPLASH_RADIUS: f32 = 200.0;

impl UnitScript for BombTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str("tower_bomb")
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
            splash_radius: SPLASH_RADIUS,
            hit_radius: 0.0,
            slow_factor: 0.0,
            slow_duration: 0.0,
            cost: 650,
            footprint: 50.0,
            hp: 1.0,
            turn_speed_deg: 360.0,
            label: RString::from("Bomb Shooter"),
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
        let target = match w.query_nearest_enemy(pos, range, e) {
            RSome(t) => t,
            RNone => return,
        };

        w.set_asd_count(e, asd_count - asd_interval);

        let atk = w.get_tower_atk(e);
        w.log_info(RStr::from_str("[tower_bomb] fire!"));
        w.spawn_projectile_ex(ProjectileSpec {
            from: pos,
            owner: e,
            path: PathSpec::Homing { target },
            speed: BULLET_SPEED,
            damage: atk,
            hit_radius: 0.0,
            splash_radius: SPLASH_RADIUS,
            slow_factor: 0.0,
            slow_duration: 0.0,
            kind_tag: RString::from("bomb"),
        });
    }
}
