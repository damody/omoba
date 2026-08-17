//! Tack Shooter — 近戰放射針塔，MVP 支援 12 升級 flag / stat。
//!
//! 支援:
//! - 路徑1：needles_12/needles_16/needles_32，blade_shooter（命中半徑110，傷害≥20）
//! - Path2: ring_of_fire (每射一次塔周 200 radius / 20 dmg magical),
//!   inferno_ring (同半徑 / 50 dmg)
//! - Stat: damage_bonus, range_bonus (透過 get_final_*)
//!
//! - burn_tier1 / burn_tier2: projectile hit 套用 5/10 DPS 的 DoT buff。

use omb_script_abi::prelude::*;

pub struct TackTower;

// 數值唯一來源：scripts/lua_data/templates.lua → omoba_template_ids 編譯期生成
// `TOWER_TACK_STATS`。hit_radius 80 須與 host 端 `comp::TACK_NEEDLE_HIT_RADIUS` 同步。
const STATS: &TowerStats = &TOWER_TACK_STATS;
const BLADE_MAELSTROM_ABILITY_ID: &str = "tack_blade_maelstrom";
const BLADE_MAELSTROM_PULSES: u16 = 4;
const BLADE_MAELSTROM_BLADES: u32 = 16;
const BLADE_MAELSTROM_RANGE: Fixed64 = Fixed64::from_i32(600);

fn burn_spec(e: EntityHandle, w: &GameWorldDyn<'_>) -> Option<(Fixed64, Fixed64)> {
    if w.has_tower_flag(e, RStr::from_str("burn_tier2")) {
        Some((Fixed64::from_i32(3), Fixed64::from_i32(10)))
    } else if w.has_tower_flag(e, RStr::from_str("burn_tier1")) {
        Some((Fixed64::from_i32(2), Fixed64::from_i32(5)))
    } else {
        None
    }
}

fn apply_burn(e: EntityHandle, victim: EntityHandle, w: &mut GameWorldDyn<'_>) {
    if let Some((duration, dps)) = burn_spec(e, w) {
        let payload = serde_json::json!({
            "dot_damage": dps.raw(),
            "damage_profile": DamageProfile::FIRE.bits(),
            "source_entity_id": e.id,
            "source_entity_gen": e.gen,
        })
        .to_string();
        w.add_stat_buff(
            victim,
            RStr::from_str("tack_burn"),
            duration,
            RStr::from_str(&payload),
        );
    }
}

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
                damage_profile: DamageProfile::SHARP,
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
            w.deal_damage_splash(
                pos,
                r,
                dmg,
                DamageKind::Magical,
                DamageProfile::FIRE,
                RSome(e),
            );
            if inferno {
                for victim in w.query_enemies_in_range(pos, r, e).iter().copied() {
                    apply_burn(e, victim, w);
                }
            }
            w.play_vfx(RStr::from_str("vfx_ring_of_fire"), pos);
        }
    }

    fn on_tower_ability_pulse(
        &self,
        tower: EntityHandle,
        ability_id: RStr<'_>,
        pulse_index: u16,
        w: &mut GameWorldDyn<'_>,
    ) -> bool {
        if ability_id.as_str() != BLADE_MAELSTROM_ABILITY_ID
            || pulse_index >= BLADE_MAELSTROM_PULSES
        {
            return false;
        }
        let pos = match w.get_pos(tower) {
            RSome(pos) => pos,
            RNone => return false,
        };
        let stats = super::tower_stats(TOWER_TACK, STATS);
        let damage = w.get_final_atk(tower) * Fixed64::from_i32(3);
        let step_ticks = omoba_sim::trig::TAU_TICKS / BLADE_MAELSTROM_BLADES as i32;
        for i in 0..BLADE_MAELSTROM_BLADES {
            let angle = omoba_sim::trig::Angle::from_ticks(step_ticks * i as i32);
            let end = Vec2 {
                x: pos.x + omoba_sim::trig::cos(angle) * BLADE_MAELSTROM_RANGE,
                y: pos.y + omoba_sim::trig::sin(angle) * BLADE_MAELSTROM_RANGE,
            };
            w.spawn_projectile_ex(ProjectileSpec {
                from: pos,
                owner: tower,
                path: PathSpec::Straight { end_pos: end },
                speed: stats.bullet_speed,
                damage,
                damage_profile: DamageProfile::SHARP,
                hit_radius: Fixed64::from_i32(110),
                splash_radius: Fixed64::ZERO,
                slow_factor: Fixed64::ZERO,
                slow_duration: Fixed64::ZERO,
                stun_duration: Fixed64::ZERO,
                kind_id: PROJECTILE_TACK_BLADE.0,
            });
        }
        true
    }

    fn on_projectile_hit(
        &self,
        attacker: EntityHandle,
        victim: EntityHandle,
        context: ProjectileHitContext,
        _query: &omb_script_abi::world::ProjectileQueryDyn<'_>,
        w: &mut GameWorldDyn<'_>,
    ) {
        if context.generation == 0
            && (context.kind_id == PROJECTILE_TACK.0 || context.kind_id == PROJECTILE_TACK_BLADE.0)
        {
            apply_burn(attacker, victim, w);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::towers::projectile_test_support::{fixture, invoke, invoke_pulse, invoke_tick};
    use omoba_core::Outcome;
    use specs::WorldExt;

    #[test]
    fn blade_maelstrom_emits_four_consumable_rings_of_sixteen_blades() {
        let fixture = fixture(
            &["needles_32", "burn_tier2"],
            &[Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)],
        );
        let mut total = 0;
        for pulse_index in 0..4 {
            let (consumed, outcomes) = invoke_pulse(
                &fixture.world,
                &TackTower,
                fixture.tower,
                "tack_blade_maelstrom",
                pulse_index,
            );
            assert!(consumed);
            let blades: Vec<_> = outcomes
                .iter()
                .filter(|outcome| {
                    matches!(outcome,
                        Outcome::ScriptProjectile { kind_id, .. }
                            if *kind_id == PROJECTILE_TACK_BLADE.0
                    )
                })
                .collect();
            assert_eq!(blades.len(), 16);
            assert!(blades.iter().all(|outcome| matches!(outcome,
                Outcome::ScriptProjectile { damage_phys, hit_radius, .. }
                    if *damage_phys == Fixed64::from_i32(30)
                        && *hit_radius == Fixed64::from_i32(110)
            )));
            total += blades.len();
        }
        assert_eq!(total, 64);

        let burn = invoke(
            &fixture.world,
            &TackTower,
            fixture.tower,
            fixture.enemies[0],
            ProjectileHitContext {
                kind_id: PROJECTILE_TACK_BLADE.0,
                generation: 0,
            },
        );
        assert!(burn.iter().any(|outcome| matches!(outcome,
            Outcome::AddBuff { payload, .. }
                if payload["dot_damage"] == Fixed64::from_i32(10).raw()
        )));
    }

    #[test]
    fn blade_maelstrom_rejects_unknown_id_and_out_of_range_pulse() {
        let fixture = fixture(&["needles_32"], &[]);
        for (ability_id, pulse_index) in [("wrong", 0), ("tack_blade_maelstrom", 4)] {
            let (consumed, outcomes) = invoke_pulse(
                &fixture.world,
                &TackTower,
                fixture.tower,
                ability_id,
                pulse_index,
            );
            assert!(!consumed);
            assert!(outcomes.is_empty());
        }
    }

    #[test]
    fn burn_tiers_apply_five_and_ten_dps_without_projectile_slow() {
        for (flag, expected_dps) in [("burn_tier1", 5), ("burn_tier2", 10)] {
            let mut fixture = fixture(&[flag], &[Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)]);
            fixture
                .world
                .write_storage::<omoba_core::TAttack>()
                .get_mut(fixture.tower)
                .unwrap()
                .asd_count = -Fixed64::from_raw(1);
            let fired = invoke_tick(
                &fixture.world,
                &TackTower,
                fixture.tower,
                Fixed64::from_raw(1),
            );
            assert!(fired
                .iter()
                .filter_map(|outcome| match outcome {
                    Outcome::ScriptProjectile {
                        slow_factor,
                        slow_duration,
                        ..
                    } => Some((*slow_factor, *slow_duration)),
                    _ => None,
                })
                .all(|slow| slow == (Fixed64::ZERO, Fixed64::ZERO)));

            let hit = invoke(
                &fixture.world,
                &TackTower,
                fixture.tower,
                fixture.enemies[0],
                ProjectileHitContext {
                    kind_id: PROJECTILE_TACK.0,
                    generation: 0,
                },
            );
            assert!(hit.iter().any(|outcome| matches!(outcome,
                Outcome::AddBuff { payload, .. }
                    if payload["dot_damage"] == Fixed64::from_i32(expected_dps).raw()
            )));
        }
    }

    #[test]
    fn inferno_ring_applies_tier_two_burn_to_every_enemy_in_ring() {
        let mut fixture = fixture(
            &["inferno_ring", "burn_tier2"],
            &[
                Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
                Vec2::new(Fixed64::from_i32(20), Fixed64::ZERO),
            ],
        );
        fixture
            .world
            .write_storage::<omoba_core::TAttack>()
            .get_mut(fixture.tower)
            .unwrap()
            .asd_count = -Fixed64::from_raw(1);
        let outcomes = invoke_tick(
            &fixture.world,
            &TackTower,
            fixture.tower,
            Fixed64::from_raw(1),
        );
        let burns = outcomes
            .iter()
            .filter(|outcome| {
                matches!(outcome,
                    Outcome::AddBuff { payload, .. }
                        if payload["dot_damage"] == Fixed64::from_i32(10).raw()
                )
            })
            .count();
        assert_eq!(burns, 2);
    }
}
