//! Cake Splash Tower：沒有 barrel 的 animated-area placeholder tower。

use omb_script_abi::prelude::*;

pub struct CakeSplashTower;

const STATS: &TowerStats = &TOWER_CAKE_SPLASH_STATS;

const CAKE_PARTY_ABILITY_ID: &str = "cake_dessert_party";
const CAKE_PARTY_PULSES: u16 = 10;
const CAKE_PARTY_DAMAGE_FACTOR: Fixed64 = Fixed64::from_raw(512);
const CAKE_PARTY_HASTE_DURATION: Fixed64 = Fixed64::from_raw(614);
const CAKE_PARTY_HASTE_MULTIPLIER: Fixed64 = Fixed64::from_raw(1280);
const SECONDARY_PULSE_DAMAGE_FACTOR: Fixed64 = Fixed64::from_raw(512);
const BURN_DURATION: Fixed64 = Fixed64::from_i32(3);
const BURN_20_FACTOR: Fixed64 = Fixed64::from_raw(205);
const BURN_40_FACTOR: Fixed64 = Fixed64::from_raw(410);
const FROST_DURATION: Fixed64 = Fixed64::from_i32(2);
const FROST_20: Fixed64 = Fixed64::from_raw(-205);
const FROST_35: Fixed64 = Fixed64::from_raw(-358);
const FROST_50: Fixed64 = Fixed64::from_raw(-512);
const VULNERABILITY_15: Fixed64 = Fixed64::from_raw(154);
const VULNERABILITY_25: Fixed64 = Fixed64::from_raw(256);

fn stronger_frosting(left: Fixed64, right: Fixed64) -> Fixed64 {
    if right.raw().abs() > left.raw().abs() {
        right
    } else {
        left
    }
}

fn source_buff_id(prefix: &str, source: EntityHandle) -> String {
    format!("{prefix}:{}:{}", source.id, source.gen)
}

fn apply_hit_effects(
    source: EntityHandle,
    victims: &[EntityHandle],
    triggering_damage: Fixed64,
    w: &mut GameWorldDyn<'_>,
) {
    let burn_factor = if w.has_tower_flag(source, RStr::from_str("cake_burn_40")) {
        BURN_40_FACTOR
    } else if w.has_tower_flag(source, RStr::from_str("cake_burn_20")) {
        BURN_20_FACTOR
    } else {
        Fixed64::ZERO
    };
    let frost = if w.has_tower_flag(source, RStr::from_str("cake_frost_50_vulnerability_25")) {
        Some((FROST_50, VULNERABILITY_25))
    } else if w.has_tower_flag(source, RStr::from_str("cake_frost_vulnerability_15")) {
        Some((FROST_35, VULNERABILITY_15))
    } else if w.has_tower_flag(source, RStr::from_str("cake_frost_35")) {
        Some((FROST_35, Fixed64::ZERO))
    } else if w.has_tower_flag(source, RStr::from_str("cake_frost_20")) {
        Some((FROST_20, Fixed64::ZERO))
    } else {
        None
    };

    let burn_id = source_buff_id("cake_burn", source);
    let burn_payload = serde_json::json!({
        "dot_damage": (triggering_damage * burn_factor).raw(),
    })
    .to_string();
    let frost_id = source_buff_id("cake_frosting", source);
    let frost_payload = frost.map(|(slow, vulnerability)| {
        serde_json::json!({
            "__aggregation_family": "cake_frosting",
            "movespeed_bonus_percentage": slow.raw(),
            "incoming_damage_percentage": vulnerability.raw(),
        })
        .to_string()
    });

    for victim in victims.iter().copied() {
        if burn_factor > Fixed64::ZERO {
            w.add_stat_buff(
                victim,
                RStr::from_str(&burn_id),
                BURN_DURATION,
                RStr::from_str(&burn_payload),
            );
        }
        if let Some(payload) = frost_payload.as_deref() {
            w.add_stat_buff(
                victim,
                RStr::from_str(&frost_id),
                FROST_DURATION,
                RStr::from_str(payload),
            );
        }
    }
}

impl UnitScript for CakeSplashTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str(TOWER_CAKE_SPLASH.as_str())
    }

    fn on_spawn(&self, e: EntityHandle, w: &mut GameWorldDyn<'_>) {
        let stats = super::tower_stats(TOWER_CAKE_SPLASH, STATS);
        w.set_tower_atk(e, stats.atk);
        w.set_tower_range(e, stats.range);
        w.set_asd_interval(e, stats.asd_interval);
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
        let timing = super::tower_attack_timing(TOWER_CAKE_SPLASH, TOWER_CAKE_SPLASH_ATTACK_TIMING);
        let phase = super::advance_attack_phase(e, dt, asd_interval, timing, w);
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
            super::start_attack_windup(e, asd_interval, timing, Target::None, w);
            return;
        }

        let damage = w.get_final_atk(e);
        let radius = range;
        let victims: Vec<EntityHandle> = w.query_enemies_in_range(pos, radius, e).into();
        w.log_info(RStr::from_str("[tower_cake_splash] splash!"));
        w.deal_damage_splash(pos, radius, damage, DamageKind::Magical, RSome(e));
        let secondary_pulses = if w.has_tower_flag(e, RStr::from_str("cake_secondary_pulse_2")) {
            2
        } else if w.has_tower_flag(e, RStr::from_str("cake_secondary_pulse_1")) {
            1
        } else {
            0
        };
        let secondary_damage = damage * SECONDARY_PULSE_DAMAGE_FACTOR;
        for _ in 0..secondary_pulses {
            w.deal_damage_splash(pos, radius, secondary_damage, DamageKind::Magical, RSome(e));
        }
        apply_hit_effects(e, &victims, damage, w);
        w.emit_explosion(pos, radius, Fixed64::from_raw(512));
    }

    fn on_tower_ability_pulse_with_access(
        &self,
        tower: EntityHandle,
        ability_id: RStr<'_>,
        pulse_index: u16,
        access: &TowerActiveAbilityAccessDyn<'_>,
        w: &mut GameWorldDyn<'_>,
    ) -> bool {
        if ability_id.as_str() != CAKE_PARTY_ABILITY_ID || pulse_index >= CAKE_PARTY_PULSES {
            return false;
        }
        let pos = match w.get_pos(tower) {
            RSome(pos) => pos,
            RNone => return false,
        };
        let range = w.get_final_attack_range(tower);
        let damage = w.get_final_atk(tower) * CAKE_PARTY_DAMAGE_FACTOR;

        // An area pulse is an emitted opportunity even when it catches no enemies.
        w.deal_damage_splash(pos, range, damage, DamageKind::Magical, RSome(tower));

        let haste_id = source_buff_id("cake_party_haste", tower);
        let haste_payload = serde_json::json!({
            "__aggregation_family": "cake_party_haste",
            "attack_speed_multiplier": CAKE_PARTY_HASTE_MULTIPLIER.raw(),
        })
        .to_string();
        for friendly in access
            .query_friendly_towers_in_range(pos, range, tower)
            .iter()
            .copied()
        {
            w.add_stat_buff(
                friendly,
                RStr::from_str(&haste_id),
                CAKE_PARTY_HASTE_DURATION,
                RStr::from_str(&haste_payload),
            );
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::towers::projectile_test_support::{fixture, invoke_tick};
    use abi_stable::{sabi_trait::prelude::TD_Opaque, RMut, RRef};
    use omb_script_abi::world::{GameWorld_TO, TowerActiveAbilityAccess_TO};
    use omoba_core::scripting::parallel_world_adapter::{
        ParallelAdapterCache, ParallelTowerActiveAbilityAccess, ParallelWorldAdapter,
    };
    use omoba_core::{Faction, FactionType, Outcome, PlayerOwner, Pos, TAttack, Tower};
    use specs::{Builder, WorldExt};

    fn ready_normal_attack(fixture: &specs::World, tower: specs::Entity) {
        fixture
            .write_storage::<TAttack>()
            .get_mut(tower)
            .unwrap()
            .asd_count = -Fixed64::from_raw(1);
    }

    fn invoke_party_pulse(fixture: &specs::World, tower: specs::Entity) -> (bool, Vec<Outcome>) {
        let cache = ParallelAdapterCache::new(fixture, 1);
        let mut adapter = ParallelWorldAdapter::new(&cache, tower);
        let access_adapter = ParallelTowerActiveAbilityAccess::new(&cache);
        let mut world_dyn = GameWorld_TO::from_ptr(RMut::new(&mut adapter), TD_Opaque);
        let access_dyn =
            TowerActiveAbilityAccess_TO::from_ptr(RRef::new(&access_adapter), TD_Opaque);
        let emitted = CakeSplashTower.on_tower_ability_pulse_with_access(
            EntityHandle {
                id: tower.id(),
                gen: tower.gen().id() as u32,
            },
            RStr::from_str(CAKE_PARTY_ABILITY_ID),
            0,
            &access_dyn,
            &mut world_dyn,
        );
        drop(access_dyn);
        drop(world_dyn);
        (emitted, adapter.finish())
    }

    #[test]
    fn cake_party_emits_ten_half_damage_pulses() {
        assert_eq!(CAKE_PARTY_PULSES, 10);
        assert_eq!(CAKE_PARTY_DAMAGE_FACTOR, Fixed64::from_raw(512));
    }

    #[test]
    fn strongest_frosting_wins() {
        assert_eq!(stronger_frosting(FROST_20, FROST_50), FROST_50);
    }

    #[test]
    fn two_secondary_pulses_are_immediate_half_damage_splashes() {
        let fixture = fixture(
            &["cake_secondary_pulse_1", "cake_secondary_pulse_2"],
            &[Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)],
        );
        ready_normal_attack(&fixture.world, fixture.tower);

        let outcomes = invoke_tick(
            &fixture.world,
            &CakeSplashTower,
            fixture.tower,
            Fixed64::from_raw(1),
        );
        let damage: Vec<Fixed64> = outcomes
            .iter()
            .filter_map(|outcome| match outcome {
                Outcome::ScriptDirectDamage { amount, .. } => Some(*amount),
                _ => None,
            })
            .collect();

        assert_eq!(
            damage,
            vec![
                Fixed64::from_i32(10),
                Fixed64::from_i32(5),
                Fixed64::from_i32(5)
            ]
        );
    }

    #[test]
    fn burn_and_frosting_use_source_stable_ids_and_fixed_payloads() {
        let fixture = fixture(
            &["cake_burn_40", "cake_frost_50_vulnerability_25"],
            &[Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)],
        );
        ready_normal_attack(&fixture.world, fixture.tower);

        let outcomes = invoke_tick(
            &fixture.world,
            &CakeSplashTower,
            fixture.tower,
            Fixed64::from_raw(1),
        );
        let generation = fixture.tower.gen().id() as u32;
        let expected_burn_id = format!("cake_burn:{}:{generation}", fixture.tower.id());
        let expected_frost_id = format!("cake_frosting:{}:{generation}", fixture.tower.id());

        assert!(outcomes.iter().any(|outcome| matches!(outcome,
            Outcome::AddBuff { buff_id, duration, payload, .. }
                if buff_id == &expected_burn_id
                    && *duration == Fixed64::from_i32(3)
                    && payload.get("dot_damage").and_then(|v| v.as_i64()) == Some(4100)
        )));
        assert!(outcomes.iter().any(|outcome| matches!(outcome,
            Outcome::AddBuff { buff_id, duration, payload, .. }
                if buff_id == &expected_frost_id
                    && *duration == Fixed64::from_i32(2)
                    && payload.get("__aggregation_family").and_then(|v| v.as_str()) == Some("cake_frosting")
                    && payload.get("movespeed_bonus_percentage").and_then(|v| v.as_i64()) == Some(-512)
                    && payload.get("incoming_damage_percentage").and_then(|v| v.as_i64()) == Some(256)
        )));
    }

    #[test]
    fn party_no_enemy_still_emits_and_refreshes_only_same_owner_friendly_towers() {
        let mut fixture = fixture(&["cake_dessert_party"], &[]);
        fixture
            .world
            .write_storage::<PlayerOwner>()
            .insert(fixture.tower, PlayerOwner::new(7))
            .unwrap();
        let same_owner = fixture
            .world
            .create_entity()
            .with(Pos(Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)))
            .with(Faction::new(FactionType::Player, 0))
            .with(PlayerOwner::new(7))
            .with(Tower::new())
            .with(TAttack::new(
                Fixed64::ONE,
                Fixed64::ONE,
                Fixed64::from_i32(500),
                Fixed64::from_i32(1000),
            ))
            .build();
        let other_owner = fixture
            .world
            .create_entity()
            .with(Pos(Vec2::new(Fixed64::from_i32(20), Fixed64::ZERO)))
            .with(Faction::new(FactionType::Player, 0))
            .with(PlayerOwner::new(8))
            .with(Tower::new())
            .build();

        let (emitted, outcomes) = invoke_party_pulse(&fixture.world, fixture.tower);
        let expected_id = format!(
            "cake_party_haste:{}:{}",
            fixture.tower.id(),
            fixture.tower.gen().id()
        );

        assert!(
            emitted,
            "the area pulse opportunity consumes its scheduled charge"
        );
        assert!(outcomes.iter().any(|outcome| matches!(outcome,
            Outcome::ScriptTowerFireFx { entity, .. } if *entity == fixture.tower
        )));
        assert!(outcomes.iter().any(|outcome| matches!(outcome,
            Outcome::AddBuff { target, buff_id, duration, payload }
                if *target == same_owner
                    && buff_id == &expected_id
                    && *duration == Fixed64::from_raw(614)
                    && payload.get("attack_speed_multiplier").and_then(|v| v.as_i64()) == Some(1280)
        )));
        assert!(!outcomes.iter().any(|outcome| matches!(outcome,
            Outcome::AddBuff { target, .. } if *target == fixture.tower || *target == other_owner
        )));
    }
}
