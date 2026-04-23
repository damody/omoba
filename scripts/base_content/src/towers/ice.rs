//! Ice Monkey — 減速塔：AoE 輕傷 + 減速 debuff。
//!
//! 由 `on_tick` 驅動：找最近敵人 → 發 homing + 小 splash + slow buff。
//! 命中時 `projectile_tick` 自動處理 splash 傷害與 `AddBuff` outcome（`slow_{owner}` buff_id），
//! GameProcessor 會把 SlowBuff component 加在每個被掃到的 creep 上。

use omb_script_abi::prelude::*;

pub struct IceTower;

const ATK: f32 = 3.0;
const ASD_INTERVAL: f32 = 1.5;
const RANGE: f32 = 180.0;
const BULLET_SPEED: f32 = 600.0;
const SPLASH_RADIUS: f32 = 90.0;
const SLOW_FACTOR: f32 = 0.5; // 減速至 50%
const SLOW_DURATION: f32 = 2.0;

impl UnitScript for IceTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str("tower_ice")
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
            slow_factor: SLOW_FACTOR,
            slow_duration: SLOW_DURATION,
            cost: 400,
            footprint: 40.0,
            hp: 1.0,
            turn_speed_deg: 360.0,
            label: RString::from("Ice Monkey"),
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
        w.log_info(RStr::from_str("[tower_ice] fire!"));
        w.spawn_projectile_ex(ProjectileSpec {
            from: pos,
            owner: e,
            path: PathSpec::Homing { target },
            speed: BULLET_SPEED,
            damage: atk,
            hit_radius: 0.0,
            splash_radius: SPLASH_RADIUS,
            slow_factor: SLOW_FACTOR,
            slow_duration: SLOW_DURATION,
            stun_duration: 0.0,
            kind_tag: RString::from("ice"),
        });
    }
}
