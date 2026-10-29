use crate::comp::*;
use crate::runtime::combat_controls::{within_raw_range, AcquisitionIntent};
use crate::runtime::native::combat_acquisition::{
    acquire, AcquisitionRequest, CombatAcquisitionView, DisclosureContext,
};
use crate::tick::attack_phase::{
    advance_attack_phase, attack_phase_durations, fixed_secs_to_ms, start_attack_windup, AttackPhaseStep,
};
use omoba_sim::Fixed64;
use specs::prelude::ParallelIterator;
use specs::{shred, Entities, Join, ParJoin, Read, ReadStorage, SystemData, Write, WriteStorage};

/// MOBA 鏡頭下肉眼無感的 facing 變化量（~15°）。舊值 0.05 (~3°) 造成過多 F event。
const FACING_BROADCAST_THRESHOLD_RAD: f32 = 0.26;

/// Hero_tick 的每實體 SimRng op_kind。階段 1de.2：取代 fastrand
/// 無目標攻擊冷卻時間抖動。重新排序或重複使用該常數
/// 跨系統將使重播決定論無效。
const OP_HERO_NO_TARGET_JITTER: u32 = 10;

#[derive(SystemData)]
pub struct HeroRead<'a> {
    entities: Entities<'a>,
    time: Read<'a, Time>,
    dt: Read<'a, DeltaTime>,
    master_seed: Read<'a, MasterSeed>,
    tick: Read<'a, Tick>,
    pos: ReadStorage<'a, Pos>,
    searcher: Read<'a, Searcher>,
    factions: ReadStorage<'a, Faction>,
    owners: ReadStorage<'a, PlayerOwner>,
    disclosed_priority: Option<Read<'a, crate::runtime::filtered_specs::DisclosedMovementPriority>>,
    moba: Option<Read<'a, crate::runtime::MobaMatch>>,
    disclosed_recalls: Option<Read<'a, super::super::cast_windup::DisclosedRecallingOwners>>,
    casts: Option<Read<'a, super::super::cast_windup::AbilityWindups>>,
    admitted_directions: Option<Read<'a, crate::runtime::input_scheme::AdmittedDirectionalSteps>>,
    facts: Read<'a, crate::runtime::ObservableFactBuffer>,
    propertys: ReadStorage<'a, CProperty>,
    turn_speeds: ReadStorage<'a, TurnSpeed>,
    move_targets: ReadStorage<'a, MoveTarget>,
    command_queues: ReadStorage<'a, HeroCommandQueue>,
    buff_store: Read<'a, omoba_core::runtime::ability_runtime::BuffStore>,
    is_buildings: ReadStorage<'a, IsBuilding>,
    visibility: Option<Read<'a, crate::runtime::TeamVisibilityRuntime>>,
    admitted: Option<Read<'a, crate::runtime::native::combat_acquisition::AdmittedCombatPreferences>>,
    attack_gate: Option<Read<'a, crate::runtime::filtered_specs::DisclosedAttackabilityGate>>,
    disclosed_wards: Option<Read<'a, crate::runtime::filtered_specs::DisclosedWardCandidates>>,
    retaliation: Option<Read<'a, super::super::retaliation::HeroRetaliation>>,
    replica_identities: ReadStorage<'a, crate::runtime::ReplicaIdentity>,
    ward_bodies: ReadStorage<'a, crate::runtime::DisclosedWardBody>,
}

#[derive(SystemData)]
pub struct HeroWrite<'a> {
    outcomes: Write<'a, Vec<Outcome>>,
    heroes: WriteStorage<'a, Hero>,
    tatks: WriteStorage<'a, TAttack>,
    facings: WriteStorage<'a, Facing>,
    facing_bcs: WriteStorage<'a, FacingBroadcast>,
}

#[derive(Default)]
pub struct Sys;

impl<'a> System<'a> for Sys {
    type SystemData = (HeroRead<'a>, HeroWrite<'a>);

    const NAME: &'static str = "hero";

    fn run(_job: &mut Job<Self>, (tr, mut tw): Self::SystemData) {
        // 階段 1c.4：dt 現在在整個戰鬥週期中固定為 64。
        let dt: Fixed64 = tr.dt.0;
        // 有損投影僅保留用於搜尋者邊界+面向弧度數學。
        // 注意：搜尋器內部使用 f32 來實作 instant_distance lib 相容性；面對弧度僅是對數。
        let dt_f = dt.to_f32_for_render();
        // 階段 1de.2：SimRng 種子輸入從 par_join 閉包中提升出來
        // （讀<'_，_>在人造絲上不是同步安全的，但裸露的u64/u32是複製的）。
        let master_seed: u64 = tr.master_seed.0;
        let tick: u32 = tr.tick.0 as u32;
        if tr.moba.is_some() {
            for (entity, _) in (&tr.entities, &tw.heroes).join() {
                let source = crate::runtime::canonical_entity_id(entity);
                tr.facts
                    .emit(crate::runtime::OrderedFact {
                        key: crate::runtime::FactOrderingKey {
                            tick: tr.tick.0,
                            phase: crate::runtime::FactPhase::Step,
                            canonical_source_order: source,
                            local_ordinal: 0,
                            fact_kind: crate::runtime::FactKind::MovementPriority,
                        },
                        audience: crate::runtime::FactAudience::VisibilityPolicy(
                            omb_script_abi::types::projection_policy_ids::MOVEMENT.to_owned(),
                        ),
                        fact: crate::runtime::ObservableFact::MovementPriority {
                            source,
                            active: command_claims_movement_priority(
                                tr.move_targets.get(entity).is_some(),
                                tr.command_queues.get(entity).and_then(|queue| queue.active),
                            ) || tr.casts.as_ref().is_some_and(|casts|casts.0.contains_key(&entity))
                                || tr.moba.as_ref().is_some_and(|m| m.is_recalling(entity)),
                        },
                    })
                    .expect("valid visible movement priority");
            }
        }

        // 獲取英雄名稱映射表
        let hero_name_map: std::collections::HashMap<specs::Entity, String> =
            (&tr.entities, &tw.heroes)
                .join()
                .map(|(e, hero)| (e, hero.name.clone()))
                .collect();
        let hero_ids: std::collections::BTreeSet<specs::Entity> =
            hero_name_map.keys().copied().collect();
        let disclosure = DisclosureContext::resolve(
            tr.disclosed_priority.is_some(),
            tr.moba.is_some(),
            tr.visibility.as_deref(),
        );
        let admitted = tr.admitted.as_deref();
        let attack_gate = tr.attack_gate.as_deref();
        let match_state = tr.moba.as_deref();

        // 技能冷卻倒數 — sequential 迴圈一次刷所有 hero 的 ability_cooldowns，
        // 在 par_join 攻擊迭代之前處理，避免 borrow 衝突。
        for (_, hero) in (&tr.entities, &mut tw.heroes).join() {
            // Rank changes (learn, loadout) take effect before this tick's recharge.
            hero.sync_authored_charges();
            hero.tick_cooldowns(dt);
        }

        let mut decisions = (
            &tr.entities,
            &mut tw.heroes,
            &tr.propertys,
            &mut tw.tatks,
            &tr.pos,
            &mut tw.facings,
            &mut tw.facing_bcs,
        )
            .par_join()
            .map_init(
                || {
                    prof_span!(guard, "hero update rayon job");
                    guard
                },
                |_guard, (e, _hero, _pty, atk, pos, facing, facing_bc)| {
                    let mut outcomes: Vec<Outcome> = Vec::new();
                    let keyboard = tr.admitted_directions.as_ref().filter(|steps| steps.tick == tr.tick.0)
                        .and_then(|steps| steps.rows.get(&e))
                        .map(|step| step.scheme == crate::runtime::input_scheme::InputScheme::Keyboard)
                        .unwrap_or_else(|| tr.owners.get(e).is_some_and(|owner| {
                        match_state.is_some_and(|state| {
                            let mut slots = state.heroes.iter().filter(|slot| slot.player_id == owner.player_id);
                            slots.next().is_some_and(|slot| slot.entity == Some(e)
                                && slot.input_scheme().scheme() == crate::runtime::input_scheme::InputScheme::Keyboard)
                                && slots.next().is_none()
                        })
                    }));

                    // 注意：搜尋器內部使用 f32 來實作 instant_distance lib 相容性；呼叫者的最終距離檢查是固定64。
                    let (pos_x_f, pos_y_f) = pos.xy_f32();
                    let pos_vek = vek::Vec2::new(pos_x_f, pos_y_f);

                    // Stun 狀態：暈眩中不攻擊、不累積冷卻（asd_count 凍結）
                    if tr.buff_store.is_disarmed(e) || tr.buff_store.has(e,super::super::recovery_channel::STATUS)
                        || tr.buff_store.has(e,super::super::ability_channel::STATUS) {
                        if let Some(timing) = atk.animation_timing.as_mut() { timing.paused = true; }
                        return (e.id(), outcomes);
                    }
                    if let Some(timing) = atk.animation_timing.as_mut() { timing.paused = dt <= Fixed64::ZERO; }

                    // 用 UnitStats 聚合攻速（Dota ATTACKSPEED_BONUS_CONSTANT 100 → 1 + 100/100 = 2× AS）
                    let stats = omoba_core::runtime::ability_runtime::UnitStats::from_refs(
                        &*tr.buff_store,
                        tr.is_buildings.get(e).is_some(),
                    );
                    let effective_interval = stats.final_attack_interval(atk.asd.v, e);
                    if atk.attack_phase == AttackSequencePhase::Backswing && atk.animation_timing.is_some() {
                        atk.animation_timing = Some(AttackAnimationTiming::new(attack_phase_durations(effective_interval), dt <= Fixed64::ZERO));
                    }

                    let attack_phase =
                        advance_attack_phase(&mut atk.asd_count, dt, effective_interval);
                    if matches!(attack_phase, AttackPhaseStep::Ready) {
                        atk.clear_attack_sequence();
                    }

                    // 移動優先於自動攻擊：有 MoveTarget 時不自動攻擊
                    // （否則 hero 會一直想轉向敵人，與移動轉向互相拉扯卡住）
                    let moving = tr.disclosed_priority.as_ref()
                        .and_then(|priority| priority.0.get(&e).copied())
                        .unwrap_or_else(|| command_claims_movement_priority(
                            tr.move_targets.get(e).is_some(),
                            tr.command_queues.get(e).and_then(|queue| queue.active),
                        ));
                    if moving || tr.casts.as_ref().is_some_and(|casts|casts.0.contains_key(&e))
                        || super::super::cast_windup::hero_recalling(tr.moba.as_deref(),tr.disclosed_recalls.as_deref(),&tr.owners,e) {
                        return (e.id(), outcomes);
                    }

                    // 當攻擊前搖完成或冷卻就緒時，嘗試攻擊。
                    if !matches!(attack_phase, AttackPhaseStep::Charging) {
                        // Wall time is never a deterministic gameplay gate.
                        if dt > Fixed64::ZERO {
                            // 攻擊範圍：UnitStats 聚合（Dota ATTACK_RANGE_BONUS + ATTACK_RANGE_BONUS_UNIQUE，MAX_ATTACK_RANGE clamp）
                            let attack_range: Fixed64 = stats.final_attack_range(atk.range.v, e);
                            let range_bonus: Fixed64 = attack_range - atk.range.v;
                            let hero_name = hero_name_map.get(&e)
                                .cloned()
                                .unwrap_or_else(|| format!("英雄 {}", e.id()));
                            let active = tr.command_queues.get(e).and_then(|queue| queue.active);
                            let committed_impact = keyboard && matches!(attack_phase, AttackPhaseStep::Impact);
                            let (intent, command_point, explicit_target) = if committed_impact {
                                (AcquisitionIntent::AttackTarget, None, atk.committed_target)
                            } else { match active {
                                Some(HeroCommand::AttackTarget { target, .. }) => {
                                    (AcquisitionIntent::AttackTarget, None, Some(target))
                                }
                                Some(HeroCommand::AttackMove { pos }) => {
                                    (AcquisitionIntent::AttackMove, Some(pos), None)
                                }
                                // Idle: answer a recent attacker in attack range first (authority,
                                // retaliation.rs), otherwise the nearest enemy in range.
                                _ => match match_state.filter(|state| state.config.game_rules.is_some()).zip(tr.retaliation.as_deref())
                                    .and_then(|(state, hits)| super::super::retaliation::attacker(hits, e, state.elapsed))
                                    .filter(|attacker| tr.pos.get(*attacker)
                                        .is_some_and(|at| within_raw_range(pos.0, at.0, attack_range))) {
                                    Some(attacker) => (AcquisitionIntent::AttackTarget, None, Some(attacker)),
                                    None => (AcquisitionIntent::Idle, None, None),
                                },
                            }};
                            let view = CombatAcquisitionView {
                                entities: &tr.entities,
                                positions: &tr.pos,
                                health: &tr.propertys,
                                factions: &tr.factions,
                                owners: &tr.owners,
                                heroes: &hero_ids,
                                searcher: &*tr.searcher,
                                disclosure,
                                admitted,
                                attack_gate,
                                match_state,
                                disclosed_wards: tr.disclosed_wards.as_deref(),
                                replica_identities: &tr.replica_identities,
                                ward_bodies: &tr.ward_bodies,
                                buffs: Some(&*tr.buff_store),
                            };
                            let selected = if committed_impact && explicit_target.is_none() { None } else { acquire(
                                &view,
                                AcquisitionRequest {
                                    source: e,
                                    intent,
                                    command_point,
                                    range: attack_range,
                                    explicit_target,
                                },
                            ) };
                            if let Some(target) = selected {
                                log::trace!("{} 在位置 ({:.0}, {:.0}) 取得攻擊目標，攻擊範圍: {:.1} (基礎 {:.1} + buff {:.1})",
                                    hero_name, pos_x_f, pos_y_f,
                                    attack_range.to_f32_for_render(),
                                    atk.range.v.to_f32_for_render(),
                                    range_bonus.to_f32_for_render());
                                let target_pos = tr.pos.get(target)
                                    .map(|p| { let (x, y) = p.xy_f32(); vek::Vec2::new(x, y) })
                                    .unwrap_or(pos_vek);
                                let diff = target_pos - pos_vek;
                                {
                                    // Co-located live targets remain legal attacks.
                                    // Keep facing when there is no meaningful direction,
                                    // rather than suppressing windup and impact forever.
                                    let desired = attack_direction(diff, facing.rad_f32());
                                    let turn = tr.turn_speeds.get(e).map(|t| t.0.to_f32_for_render())
                                        .unwrap_or(std::f32::consts::FRAC_PI_2);
                                    let cur_rad = facing.rad_f32();
                                    let new_rad = rotate_toward(cur_rad, desired, turn * dt_f);
                                    *facing = Facing::from_rad_f32(new_rad);

                                    // 廣播 facing 變化：和「上次廣播」差 > 15° 才送。
                                    let needs_emit = match facing_bc.0 {
                                        None => true,
                                        Some(last) => (new_rad - last).abs() > FACING_BROADCAST_THRESHOLD_RAD,
                                    };
                                    if needs_emit {
                                        facing_bc.0 = Some(new_rad);
                                    }

                                    let angle_diff = normalize_angle(desired - new_rad).abs();
                                    if committed_impact || angle_diff < MOVE_ANGLE_THRESHOLD {
                                        if matches!(attack_phase, AttackPhaseStep::Ready) {
                                            let (windup, backswing) = start_attack_windup(
                                                &mut atk.asd_count,
                                                effective_interval,
                                            );
                                            let attack_seq = atk.begin_attack_windup();
                                            if keyboard { atk.committed_target = Some(target); }
                                            atk.animation_timing = Some(AttackAnimationTiming::new((windup, backswing), false));
                                            outcomes.push(Outcome::AttackPhaseCue {
                                                entity: e,
                                                attack_seq,
                                                is_critical: false,
                                                target: Some(target),
                                                target_pos: tr.pos.get(target).map(|p| p.0),
                                                windup_ms: fixed_secs_to_ms(windup),
                                                backswing_ms: fixed_secs_to_ms(backswing),
                                                dir_rad: desired,
                                            });
                                        } else {
                                            atk.mark_attack_impact();
                                            atk.animation_timing = Some(AttackAnimationTiming::new(attack_phase_durations(effective_interval), false));
                                            outcomes.push(Outcome::ProjectileLine2 {
                                                pos: pos.0,
                                                source: Some(e.clone()),
                                                target: Some(target)
                                            });
                                            let actual_distance = diff.magnitude();
                                            log::debug!("⚔️ {} 發射彈道攻擊，距離: {:.0}，攻擊力: {:.1}",
                                                hero_name, actual_distance,
                                                atk.atk_physic.v.to_f32_for_render());
                                        }
                                    }
                                    // 角度太大 → 繼續轉，本 tick 不開火
                                }
                            } else {
                                log::trace!("{} 沒有找到目標", hero_name);
                            }
                            // A hidden, dead, recycled or out-of-range committed life
                            // fizzles; it must never retarget or reset the ready clock.
                            if committed_impact {
                                atk.mark_attack_impact();
                                atk.animation_timing = Some(AttackAnimationTiming::new(attack_phase_durations(effective_interval), false));
                            }
                            if selected.is_none() && matches!(attack_phase, AttackPhaseStep::Ready) {
                                // 沒有有效目標時，減少一些攻擊冷卻時間避免過度檢查
                                // 0.3 ≈ 307/1024 原始；原始抖動 ε [0, 256) ≈ 0..0.25。
                                // 階段 1de.2：透過 SimRng 確定性每（英雄、刻度）抖動。
                                // Owned heroes are remapped into team-local ECS
                                // IDs. Use the disclosed player identity, never
                                // an authority-only entity index, for idle RNG.
                                let rng_actor = tr.owners.get(e).map_or(e.id(), |owner| owner.player_id);
                                let ordinal = (u64::from(rng_actor) << 16)
                                    | u64::from(OP_HERO_NO_TARGET_JITTER);
                                let jitter = Fixed64::from_raw(
                                    (crate::runtime::tick_random_u64(
                                        master_seed,
                                        u64::from(tick),
                                        ordinal,
                                    ) % 256) as i64,
                                );
                                atk.asd_count = effective_interval - Fixed64::from_raw(307) - jitter;
                                log::trace!("{} 沒有找到有效目標，減少攻擊冷卻時間: {:.3}",
                                    hero_name, atk.asd_count.to_f32_for_render());
                            }
                        }
                    }

                    (e.id(), outcomes)
                },
            )
            .fold(
                || Vec::new(),
                |mut all_outcomes, decision| {
                    all_outcomes.push(decision);
                    all_outcomes
                },
            )
            .reduce(
                || Vec::new(),
                |mut outcomes_a, mut outcomes_b| {
                    outcomes_a.append(&mut outcomes_b);
                    outcomes_a
                },
            );

        decisions.sort_by_key(|(entity_id, _)| *entity_id);
        for (_, mut outcomes) in decisions {
            tw.outcomes.append(&mut outcomes);
        }
    }
}

fn attack_direction(delta: vek::Vec2<f32>, current: f32) -> f32 {
    if delta.magnitude_squared() > 0.01 { delta.y.atan2(delta.x) } else { current }
}

#[cfg(test)]
mod explicit_target_tests {
    use super::attack_direction;
    use crate::comp::HeroCommand;
    use crate::runtime::native::combat_acquisition::test_support::{CombatFixture, IndexedBody};

    #[test]
    fn overlapping_attack_target_preserves_facing_without_blocking_attack() {
        for delta in [vek::Vec2::zero(), vek::Vec2::new(0.01, -0.01)] {
            assert_eq!(attack_direction(delta, 1.25), 1.25);
        }
        assert_eq!(attack_direction(vek::Vec2::new(0.0, 5.0), 1.25), std::f32::consts::FRAC_PI_2);
    }

    #[test]
    fn explicit_target_is_not_limited_by_ten_closer_auto_attack_candidates() {
        let mut fix = CombatFixture::legacy();
        for index in 1..=12 {
            fix.spawn_units(IndexedBody::Creep, index, 2, 100);
        }
        let explicit = fix.spawn_units(IndexedBody::Creep, 30, 2, 100);
        fix.command(HeroCommand::AttackTarget {
            target: explicit,
            chase_origin: None,
        });
        assert_eq!(fix.attack_targets(), vec![explicit]);
    }
}
