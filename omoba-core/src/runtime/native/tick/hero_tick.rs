use crate::comp::*;
use crate::tick::attack_phase::{
    advance_attack_phase, fixed_secs_to_ms, start_attack_windup, AttackPhaseStep,
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

// Explicit attacks must not lose their target merely because the bounded
// nearest-neighbour auto-attack query contains ten closer creeps. The caller
// supplies only a live target's current position; normal range/faction/HP gates
// below still apply, including in a filtered replica.
fn include_explicit_attack_candidate(
    candidates: &mut Vec<DisIndex>, target: Option<specs::Entity>,
    target_pos: Option<vek::Vec2<f32>>, origin: vek::Vec2<f32>, range: f32,
) {
    let (Some(target), Some(position)) = (target, target_pos) else { return; };
    let distance = position.distance_squared(origin);
    if distance.is_finite() && range.is_finite() && range > 0.0
        && distance <= range * range && !candidates.iter().any(|hit| hit.e == target) {
        candidates.push(DisIndex { e: target, dis: distance });
    }
}

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
    facts: Read<'a, crate::runtime::ObservableFactBuffer>,
    propertys: ReadStorage<'a, CProperty>,
    turn_speeds: ReadStorage<'a, TurnSpeed>,
    move_targets: ReadStorage<'a, MoveTarget>,
    command_queues: ReadStorage<'a, HeroCommandQueue>,
    buff_store: Read<'a, omoba_core::runtime::ability_runtime::BuffStore>,
    is_buildings: ReadStorage<'a, IsBuilding>,
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
                            active: tr.move_targets.get(entity).is_some() || tr.moba.as_ref().is_some_and(|m| m.is_recalling(entity)),
                        },
                    })
                    .expect("valid visible movement priority");
            }
        }

        // 獲取英雄的陣營信息和名稱用於敵友判斷和日誌記錄
        let hero_faction_map: std::collections::HashMap<specs::Entity, Faction> =
            (&tr.entities, &tr.factions, &tw.heroes)
                .join()
                .map(|(e, f, _)| (e, f.clone()))
                .collect();

        // 獲取英雄名稱映射表
        let hero_name_map: std::collections::HashMap<specs::Entity, String> =
            (&tr.entities, &tw.heroes)
                .join()
                .map(|(e, hero)| (e, hero.name.clone()))
                .collect();

        // 技能冷卻倒數 — sequential 迴圈一次刷所有 hero 的 ability_cooldowns，
        // 在 par_join 攻擊迭代之前處理，避免 borrow 衝突。
        for (_, hero) in (&tr.entities, &mut tw.heroes).join() {
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

                    // 注意：搜尋器內部使用 f32 來實作 instant_distance lib 相容性；呼叫者的最終距離檢查是固定64。
                    let (pos_x_f, pos_y_f) = pos.xy_f32();
                    let pos_vek = vek::Vec2::new(pos_x_f, pos_y_f);

                    // Stun 狀態：暈眩中不攻擊、不累積冷卻（asd_count 凍結）
                    if tr.buff_store.is_stunned(e) {
                        return (e.id(), outcomes);
                    }

                    // 用 UnitStats 聚合攻速（Dota ATTACKSPEED_BONUS_CONSTANT 100 → 1 + 100/100 = 2× AS）
                    let stats = omoba_core::runtime::ability_runtime::UnitStats::from_refs(
                        &*tr.buff_store,
                        tr.is_buildings.get(e).is_some(),
                    );
                    // 0.01 ≈ 10/1024 raw — 下限以避免被零除的發散。
                    let asd_mult_raw = stats.final_attack_speed_mult(e);
                    let min_asd_mult = Fixed64::from_raw(10);
                    let asd_mult = if asd_mult_raw < min_asd_mult { min_asd_mult } else { asd_mult_raw };
                    let effective_interval: Fixed64 = atk.asd.v / asd_mult;

                    let attack_phase =
                        advance_attack_phase(&mut atk.asd_count, dt, effective_interval);
                    if matches!(attack_phase, AttackPhaseStep::Ready) {
                        atk.clear_attack_sequence();
                    }

                    // 移動優先於自動攻擊：有 MoveTarget 時不自動攻擊
                    // （否則 hero 會一直想轉向敵人，與移動轉向互相拉扯卡住）
                    let moving = tr.disclosed_priority.as_ref()
                        .and_then(|priority| priority.0.get(&e).copied())
                        .unwrap_or_else(|| tr.move_targets.get(e).is_some());
                    if moving || tr.moba.as_ref().is_some_and(|m| m.is_recalling(e)) {
                        return (e.id(), outcomes);
                    }

                    // 當攻擊前搖完成或冷卻就緒時，嘗試攻擊。
                    if !matches!(attack_phase, AttackPhaseStep::Charging) {
                        // Wall time is never a deterministic gameplay gate.
                        if dt > Fixed64::ZERO {
                            // 搜尋攻擊範圍內的所有單位
                            let search_n = 10; // 搜尋最近的 10 個目標
                            // 攻擊範圍：UnitStats 聚合（Dota ATTACK_RANGE_BONUS + ATTACK_RANGE_BONUS_UNIQUE，MAX_ATTACK_RANGE clamp）
                            let attack_range: Fixed64 = stats.final_attack_range(atk.range.v, e);
                            let range_bonus: Fixed64 = attack_range - atk.range.v;
                            let search_range: Fixed64 = attack_range + Fixed64::from_i32(50); // 稍微擴大搜尋範圍以確保不遺漏邊界目標
                            // 注意：搜尋器內部使用 f32 來實作 instant_distance lib 相容性；呼叫者的最終距離檢查是固定64。
                            let attack_range_f = attack_range.to_f32_for_render();
                            let search_range_f = search_range.to_f32_for_render();
                            let (creep_targets, _) =
                                tr.searcher.creep.search_nn_two_radii(pos_vek, attack_range_f, search_range_f, search_n);
                            let (tower_targets, _) =
                                tr.searcher.tower.search_nn_two_radii(pos_vek, attack_range_f, search_range_f, search_n);
                            // 合併 creep + tower 候選，一起走敵友判斷
                            let mut potential_targets = Vec::with_capacity(creep_targets.len() + tower_targets.len());
                            potential_targets.extend(creep_targets);
                            potential_targets.extend(tower_targets);
                            potential_targets.sort_by(|a, b| a.dis.total_cmp(&b.dis)
                                .then_with(|| a.e.id().cmp(&b.e.id()))
                                .then_with(|| a.e.gen().id().cmp(&b.e.gen().id())));
                            let explicit_target = tr.command_queues.get(e).and_then(|queue| match queue.active {
                                Some(HeroCommand::AttackTarget { target, .. }) if tr.entities.is_alive(target) => Some(target),
                                _ => None,
                            });
                            include_explicit_attack_candidate(&mut potential_targets, explicit_target,
                                explicit_target.and_then(|target| tr.pos.get(target).map(|pos| {
                                    let (x, y) = pos.xy_f32(); vek::Vec2::new(x, y)
                                })), pos_vek, attack_range_f);

                            // 偵錯：顯示搜尋結果
                            // 獲取英雄名稱
                            let hero_name = hero_name_map.get(&e)
                                .cloned()
                                .unwrap_or_else(|| format!("英雄 {}", e.id()));

                            if potential_targets.len() > 0 {
                                log::trace!("{} 在位置 ({:.0}, {:.0}) 搜尋到 {} 個潛在目標，攻擊範圍: {:.1} (基礎 {:.1} + buff {:.1})",
                                    hero_name, pos_x_f, pos_y_f, potential_targets.len(),
                                    attack_range.to_f32_for_render(),
                                    atk.range.v.to_f32_for_render(),
                                    range_bonus.to_f32_for_render());
                            } else {
                                log::trace!("{} 沒有找到目標", hero_name);
                            }

                            // 過濾出可攻擊的敵對目標（必須在攻擊範圍內）
                            // 注意：搜尋器返回 f32 平方距離； f32 中的比較可接受（邊界有損還可以）。
                            let mut valid_targets = Vec::new();
                            let attack_range_squared = attack_range_f * attack_range_f; // 計算攻擊範圍的平方

                            if let Some(hero_faction) = hero_faction_map.get(&e) {
                                for target_info in potential_targets.iter() {
                                    // 首先檢查距離是否在攻擊範圍內
                                    if target_info.dis <= attack_range_squared {
                                        if let Some(target_faction) = tr.factions.get(target_info.e) {
                                            // 嚴格敵友判定：只有 is_hostile_to = true 才算敵對
                                            if hero_faction.is_hostile_to(target_faction)
                                                && tr.propertys.get(target_info.e)
                                                    .is_some_and(|p| p.hp > Fixed64::ZERO) {
                                                valid_targets.push(target_info);
                                            }
                                        }
                                        // 無 Faction → 不攻擊（安全預設，避免誤擊我方單位）
                                    }
                                }
                            } else {
                                log::warn!("{} 沒有陣營信息，無法進行敵友判斷", hero_name);
                            }

                            if valid_targets.len() > 0 {
                                // 攻擊最近的敵人：先轉向，角度 < 30° 才能開火
                                let forced_target = tr.command_queues.get(e).and_then(|queue| {
                                    match queue.active {
                                        Some(HeroCommand::AttackTarget { target, .. }) => Some(target),
                                        _ => None,
                                    }
                                });
                                let selected_target = forced_target
                                    .and_then(|target| {
                                        valid_targets
                                            .iter()
                                            .copied()
                                            .find(|target_info| target_info.e == target)
                                    })
                                    .unwrap_or(valid_targets[0]);
                                let target = selected_target.e;
                                // 注意：轉向日誌使用 f32 邊界 — Fix64 沒有顯示。
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
                                    if angle_diff < MOVE_ANGLE_THRESHOLD {
                                        if matches!(attack_phase, AttackPhaseStep::Ready) {
                                            let (windup, backswing) = start_attack_windup(
                                                &mut atk.asd_count,
                                                effective_interval,
                                            );
                                            let attack_seq = atk.begin_attack_windup();
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
                                            outcomes.push(Outcome::ProjectileLine2 {
                                                pos: pos.0,
                                                source: Some(e.clone()),
                                                target: Some(target)
                                            });
                                            let actual_distance = selected_target.dis.sqrt();
                                            log::debug!("⚔️ {} 發射彈道攻擊，距離: {:.0}，攻擊力: {:.1}",
                                                hero_name, actual_distance,
                                                atk.atk_physic.v.to_f32_for_render());
                                        }
                                    }
                                    // 角度太大 → 繼續轉，本 tick 不開火
                                }
                            } else if matches!(attack_phase, AttackPhaseStep::Ready) {
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
    use super::*;
    use specs::{Builder, World, WorldExt};
    #[test]
    fn overlapping_attack_target_preserves_facing_without_blocking_attack() {
        for delta in [vek::Vec2::zero(), vek::Vec2::new(0.01, -0.01)] {
            assert_eq!(attack_direction(delta, 1.25), 1.25);
        }
        assert_eq!(attack_direction(vek::Vec2::new(0.0, 5.0), 1.25), std::f32::consts::FRAC_PI_2);
    }
    #[test]
    fn explicit_target_is_not_limited_by_ten_closer_auto_attack_candidates() {
        let mut world = World::new();
        let target = world.create_entity().build();
        let mut candidates: Vec<_> = (0..10).map(|i| DisIndex {
            e: world.create_entity().build(), dis: (i + 1) as f32,
        }).collect();
        let origin = vek::Vec2::new(0.0, 0.0);
        include_explicit_attack_candidate(&mut candidates, Some(target), Some(vek::Vec2::new(400.0, 0.0)), origin, 550.0);
        assert_eq!(candidates.len(), 11);
        assert_eq!(candidates[10].e, target);
        include_explicit_attack_candidate(&mut candidates, Some(target), Some(vek::Vec2::new(400.0, 0.0)), origin, 550.0);
        assert_eq!(candidates.len(), 11);
        for (position, range) in [(vek::Vec2::new(600.0, 0.0), 550.0),
            (vek::Vec2::new(f32::NAN, 0.0), 550.0), (origin, 0.0)] {
            let mut empty = Vec::new();
            include_explicit_attack_candidate(&mut empty, Some(target), Some(position), origin, range);
            assert!(empty.is_empty());
        }
        let mut empty = Vec::new();
        include_explicit_attack_candidate(&mut empty, Some(target), None, origin, 550.0);
        include_explicit_attack_candidate(&mut empty, None, Some(origin), origin, 550.0);
        assert!(empty.is_empty());
    }
}
