//! Cake Splash Tower：沒有 barrel 的 animated-area placeholder tower。

use omb_script_abi::prelude::*;

pub struct CakeSplashTower;

const STATS: &TowerStats = &TOWER_CAKE_SPLASH_STATS;

impl UnitScript for CakeSplashTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str(TOWER_CAKE_SPLASH.as_str())
    }

    fn on_spawn(&self, e: EntityHandle, w: &mut GameWorldDyn<'_>) {
        w.set_tower_atk(e, STATS.atk);
        w.set_tower_range(e, STATS.range);
        w.set_asd_interval(e, STATS.asd_interval);
    }

    fn tower_metadata(&self) -> ROption<TowerMetadata> {
        RSome(super::tower_metadata_from_consts(
            TOWER_CAKE_SPLASH,
            STATS,
            &TOWER_CAKE_SPLASH_RENDER,
            TOWER_CAKE_SPLASH_ATTACK_TIMING,
        ))
    }

    fn on_tick(&self, e: EntityHandle, dt: Fixed64, w: &mut GameWorldDyn<'_>) {
        let asd_interval = w.get_asd_interval(e);
        if asd_interval <= Fixed64::ZERO {
            return;
        }
        let phase =
            super::advance_attack_phase(e, dt, asd_interval, TOWER_CAKE_SPLASH_ATTACK_TIMING, w);
        if matches!(phase, super::AttackPhaseStep::Charging) {
            return;
        }

        let pos = match w.get_pos(e) {
            RSome(p) => p,
            RNone => return,
        };
        let range = w.get_final_attack_range(e);
        if matches!(w.query_nearest_enemy(pos, range, e), RNone) {
            return;
        }
        if matches!(phase, super::AttackPhaseStep::Ready) {
            super::start_attack_windup(
                e,
                asd_interval,
                TOWER_CAKE_SPLASH_ATTACK_TIMING,
                Target::None,
                w,
            );
            return;
        }

        let damage = w.get_final_atk(e);
        let radius = if STATS.splash_radius > Fixed64::ZERO {
            STATS.splash_radius
        } else {
            range
        };
        w.log_info(RStr::from_str("[tower_cake_splash] splash!"));
        w.deal_damage_splash(pos, radius, damage, DamageKind::Magical, RSome(e));
        w.emit_explosion(pos, radius, Fixed64::from_raw(512));
    }
}
