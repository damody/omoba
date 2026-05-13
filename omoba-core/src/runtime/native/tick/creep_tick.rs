use crate::comp::phys::MAX_COLLISION_RADIUS;
use crate::comp::*;
use omoba_sim::trig::{angle_rotate_toward, atan2 as sim_atan2, fixed_rad_to_ticks, TAU_TICKS};
use omoba_sim::{Angle, Fixed64, Vec2 as SimVec2};
use specs::prelude::ParallelIterator;
use specs::{shred, Entities, ParJoin, Read, ReadStorage, SystemData, Write, WriteStorage};
use std::collections::BTreeMap;

/// MOBA 鏡頭下肉眼無感的 facing 變化量（~15°）。舊值 0.05 (~3°) 造成過多 F event。
const FACING_BROADCAST_THRESHOLD_RAD: f32 = 0.26;

#[derive(SystemData)]
pub struct CreepRead<'a> {
    entities: Entities<'a>,
    time: Read<'a, Time>,
    dt: Read<'a, DeltaTime>,
    /// P4：伺服器滴答計數器；在客戶端的 cree.M 中用作 `start_tick`
    /// 外推錨。
    tick: Read<'a, Tick>,
    paths: Read<'a, BTreeMap<String, Path>>,
    check_points: Read<'a, BTreeMap<String, CheckPoint>>,
    cpropertys: ReadStorage<'a, CProperty>,
    turn_speeds: ReadStorage<'a, TurnSpeed>,
    radii: ReadStorage<'a, CollisionRadius>,
    searcher: Read<'a, Searcher>,
    buff_store: Read<'a, omoba_core::runtime::ability_runtime::BuffStore>,
    is_buildings: ReadStorage<'a, IsBuilding>,
    creeps: ReadStorage<'a, Creep>,
    pos: ReadStorage<'a, Pos>,
    facings: ReadStorage<'a, Facing>,
    facing_bcs: ReadStorage<'a, FacingBroadcast>,
}

#[derive(SystemData)]
pub struct CreepWrite<'a> {
    /// P4：用於 M 發射選通的每個 Creep 最後廣播快照。
    /// 在第一次發射時延遲插入（對於 Creep 來說組件可能不存在
    /// 在 P4 升級路徑之前就存在）。
    mv_broadcasts: WriteStorage<'a, CreepMoveBroadcast>,
    outcomes: Write<'a, Vec<Outcome>>,
    taken_damages: Write<'a, Vec<TakenDamage>>,
}

#[derive(Default)]
pub struct Sys;

impl<'a> System<'a> for Sys {
    type SystemData = (CreepRead<'a>, CreepWrite<'a>);

    const NAME: &'static str = "creep";

    fn run(_job: &mut Job<Self>, (tr, mut tw): Self::SystemData) {
        let time = tr.time.0;
        let dt = tr.dt.0;
        // CProperty.msd 的 dt 的舊版 f32 視圖（仍然是 f32；第 1c 階段）。
        let dt_f = dt.to_f32_for_render();
        let server_tick = tr.tick.0;
        let _tick_span = tracing::span!(
            tracing::Level::TRACE,
            "omoba_core::runtime::creep_tick.tick",
            perfetto = true,
            tick = server_tick,
        )
        .entered();

        // P4 發出從 par_join 通道收集的候選者，由實體鍵入。
        // 承載電流（目標、速度、起始位置、朝向）- 閘控 +
        // 記錄更新在下面連續發生，因此我們可以觸摸 mv_broadcasts
        // 無需在並行閉包內對抗借用規則。
        // 採用固定 64 有效負載 — 在第 2 階段 KCP 標籤返工中重新設計。
        struct MoveCandidate {
            entity: specs::Entity,
            target: vek::Vec2<f32>,
            velocity: f32,
            start_pos: vek::Vec2<f32>,
            facing: f32,
        }

        let (mut outcomes, move_candidates) = {
            let _par_join_span = tracing::span!(
                tracing::Level::TRACE,
                "omoba_core::runtime::creep_tick.par_join",
                perfetto = true,
                tick = server_tick,
            )
            .entered();

            (
                &tr.entities,
                &tr.creeps,
                &tr.pos,
                &tr.cpropertys,
                &tr.facings,
                &tr.facing_bcs,
            )
                .par_join()
                .filter(|(_e, _creep, _p, _cp, _f, _fb)| true)
                .map_init(
                    || {
                        prof_span!(guard, "creep update rayon job");
                        let span = tracing::span!(
                            tracing::Level::TRACE,
                            "omoba_core::runtime::creep_tick.rayon_job",
                            perfetto = true,
                            tick = server_tick,
                        );
                        let tracing_guard = span.entered();
                        (guard, tracing_guard)
                    },
                    |(_guard, _tracing_guard), (e, creep, pos, cp, facing, facing_bc)| {
                        let mut outcomes: Vec<Outcome> = Vec::new();
                        let mut cands: Vec<MoveCandidate> = Vec::new();

                        #[inline(always)]
                        fn p_to_f(p: SimVec2) -> vek::Vec2<f32> {
                            vek::Vec2::new(p.x.to_f32_for_render(), p.y.to_f32_for_render())
                        }
                        #[inline(always)]
                        fn a_to_rad(a: Angle) -> f32 {
                            (a.ticks() as f32 / TAU_TICKS as f32) * std::f32::consts::TAU
                        }

                        let mut new_pos = pos.0;
                        let mut new_status = creep.status.clone();
                        let mut new_pidx = creep.pidx;
                        let mut new_facing = facing.0;
                        let mut new_facing_bc = facing_bc.0;
                        let mut needs_update = false;

                        if cp.hp <= Fixed64::ZERO {
                            log::info!(
                                "☠️ creep_tick sees hp<=0: name={} hp={:.1} mhp={:.1} ent={}",
                                creep.name,
                                cp.hp.to_f32_for_render(),
                                cp.mhp.to_f32_for_render(),
                                e.id()
                            );
                            outcomes.push(Outcome::Death { pos: pos.0, ent: e });
                            return (outcomes, cands);
                        }

                        let Some(path) = tr.paths.get(&creep.path) else {
                            return (outcomes, cands);
                        };
                        if creep.block_tower.is_some() {
                            return (outcomes, cands);
                        }
                        if creep.pidx >= path.check_points.len() {
                            if !matches!(creep.status, CreepStatus::Leaked) {
                                outcomes.push(Outcome::CreepLeaked { ent: e });
                                new_status = CreepStatus::Leaked;
                                needs_update = true;
                            }
                        } else if let Some(p) = path.check_points.get(creep.pidx) {
                            let target_point_f: vek::Vec2<f32> = p.pos;
                            let target_point = SimVec2::new(
                                Fixed64::from_raw(
                                    (target_point_f.x * omoba_sim::fixed::SCALE as f32) as i64,
                                ),
                                Fixed64::from_raw(
                                    (target_point_f.y * omoba_sim::fixed::SCALE as f32) as i64,
                                ),
                            );
                            let stats = omoba_core::runtime::ability_runtime::UnitStats::from_refs(
                                &*tr.buff_store,
                                tr.is_buildings.get(e).is_some(),
                            );
                            let effective_msd = stats.final_move_speed(cp.msd, e);

                            match creep.status {
                                CreepStatus::PreWalk => {
                                    cands.push(MoveCandidate {
                                        entity: e,
                                        target: target_point_f,
                                        velocity: effective_msd.to_f32_for_render(),
                                        start_pos: p_to_f(pos.0),
                                        facing: a_to_rad(facing.0),
                                    });
                                    new_status = CreepStatus::Walk;
                                    needs_update = true;
                                }
                                CreepStatus::Walk => {
                                    if tr.buff_store.is_rooted(e) {
                                        return (outcomes, cands);
                                    }

                                    let step = effective_msd * dt;
                                    let diff = target_point - pos.0;
                                    let dist_sq = diff.length_squared();
                                    let arrived_eps_sq = Fixed64::from_raw(10);
                                    if dist_sq < arrived_eps_sq {
                                        new_pidx = creep.pidx + 1;
                                        if let Some(t) = path.check_points.get(new_pidx) {
                                            cands.push(MoveCandidate {
                                                entity: e,
                                                target: t.pos,
                                                velocity: effective_msd.to_f32_for_render(),
                                                start_pos: p_to_f(new_pos),
                                                facing: a_to_rad(new_facing),
                                            });
                                        }
                                        needs_update = true;
                                    } else {
                                        let desired_angle = sim_atan2(diff.y, diff.x);
                                        let turn_rate = tr
                                            .turn_speeds
                                            .get(e)
                                            .map(|t| t.0)
                                            .unwrap_or(Fixed64::from_raw(1608));
                                        let max_step_ticks = fixed_rad_to_ticks(turn_rate * dt);
                                        new_facing = angle_rotate_toward(
                                            facing.0,
                                            desired_angle,
                                            max_step_ticks,
                                        );
                                        let new_facing_rad = a_to_rad(new_facing);
                                        let facing_needs_emit = match facing_bc.0 {
                                            None => true,
                                            Some(last) => {
                                                (new_facing_rad - last).abs()
                                                    > FACING_BROADCAST_THRESHOLD_RAD
                                            }
                                        };
                                        if facing_needs_emit {
                                            new_facing_bc = Some(new_facing_rad);
                                        }
                                        needs_update = true;

                                        let diff_ticks = (desired_angle.ticks()
                                            - new_facing.ticks())
                                        .rem_euclid(TAU_TICKS);
                                        let signed_diff_ticks = if diff_ticks > TAU_TICKS / 2 {
                                            diff_ticks - TAU_TICKS
                                        } else {
                                            diff_ticks
                                        };
                                        if signed_diff_ticks.abs() < MOVE_ANGLE_THRESHOLD_TICKS {
                                            let radius = tr
                                                .radii
                                                .get(e)
                                                .map(|r| r.0)
                                                .unwrap_or(Fixed64::from_i32(20));
                                            let self_entity = e;
                                            let radius_f = radius.to_f32_for_render();
                                            let hits = |p_sim: SimVec2| -> bool {
                                                let q_r = radius_f + MAX_COLLISION_RADIUS;
                                                let p_vek = vek::Vec2::new(
                                                    p_sim.x.to_f32_for_render(),
                                                    p_sim.y.to_f32_for_render(),
                                                );
                                                for di in
                                                    tr.searcher.search_collidable(p_vek, q_r, 16)
                                                {
                                                    if di.e == self_entity {
                                                        continue;
                                                    }
                                                    let Some(other_r) =
                                                        tr.radii.get(di.e).map(|cr| cr.0)
                                                    else {
                                                        continue;
                                                    };
                                                    let touch = radius + other_r;
                                                    let touch_f = touch.to_f32_for_render();
                                                    if di.dis < touch_f * touch_f {
                                                        return true;
                                                    }
                                                }
                                                false
                                            };

                                            let mut blocked = false;
                                            if dist_sq > step * step {
                                                let v = diff.normalized() * step;
                                                let full = pos.0 + v;
                                                if !hits(full) {
                                                    new_pos = full;
                                                } else {
                                                    let only_x =
                                                        SimVec2::new(pos.0.x + v.x, pos.0.y);
                                                    let only_y =
                                                        SimVec2::new(pos.0.x, pos.0.y + v.y);
                                                    if !hits(only_x) {
                                                        new_pos = only_x;
                                                    } else if !hits(only_y) {
                                                        new_pos = only_y;
                                                    } else {
                                                        blocked = true;
                                                    }
                                                }
                                            } else if !hits(target_point) {
                                                new_pos = target_point;
                                                new_pidx = creep.pidx + 1;
                                                if let Some(t) = path.check_points.get(new_pidx) {
                                                    cands.push(MoveCandidate {
                                                        entity: e,
                                                        target: t.pos,
                                                        velocity: effective_msd.to_f32_for_render(),
                                                        start_pos: p_to_f(new_pos),
                                                        facing: a_to_rad(new_facing),
                                                    });
                                                }
                                            } else {
                                                blocked = true;
                                            }

                                            if !blocked {
                                                cands.push(MoveCandidate {
                                                    entity: e,
                                                    target: target_point_f,
                                                    velocity: effective_msd.to_f32_for_render(),
                                                    start_pos: p_to_f(new_pos),
                                                    facing: a_to_rad(new_facing),
                                                });
                                            }
                                        }
                                    }
                                }
                                CreepStatus::Stop => {
                                    new_status = CreepStatus::PreWalk;
                                    needs_update = true;
                                }
                                CreepStatus::Leaked => {}
                            }
                        } else {
                            outcomes.push(Outcome::Death { pos: pos.0, ent: e });
                        }

                        if needs_update {
                            outcomes.push(Outcome::CreepUpdate {
                                entity: e,
                                pos: new_pos,
                                status: new_status,
                                pidx: new_pidx,
                                facing: new_facing,
                                facing_broadcast: new_facing_bc,
                            });
                        }
                        (outcomes, cands)
                    },
                )
                .fold(
                    || (Vec::new(), Vec::<MoveCandidate>::new()),
                    |(mut all_outcomes, mut all_cands), (mut outcomes, mut cands)| {
                        all_outcomes.append(&mut outcomes);
                        all_cands.append(&mut cands);
                        (all_outcomes, all_cands)
                    },
                )
                .reduce(
                    || (Vec::new(), Vec::<MoveCandidate>::new()),
                    |(mut outcomes_a, mut cands_a), (mut outcomes_b, mut cands_b)| {
                        outcomes_a.append(&mut outcomes_b);
                        cands_a.append(&mut cands_b);
                        (outcomes_a, cands_a)
                    },
                )
        };

        // P4 串行發射閘通：將每個候選者與
        // 實體的最後廣播快照（CreepMoveBroadcast 元件）。
        // 僅當目標偏離或速度變化 > 5% / 時才發出蠕變。
        // > 1.0 絕對值或實體沒有先前的快照。更新
        // 發出後的組件，因此下一個刻度的比較使用新的基線。
        for cand in move_candidates.into_iter() {
            let need_emit = match tw.mv_broadcasts.get(cand.entity) {
                Some(bcast) => bcast.should_emit(cand.target, cand.velocity),
                None => true, // first-ever candidate for this entity
            };
            if !need_emit {
                continue;
            }

            // 階段 5.2：遺留 0x02 GameEvent 製作人刪減。鎖步刻度批次處理
            // (0x10)攜帶權威pos；客戶端從 sim 渲染。

            // 更新（或插入）廣播快照以便後續刻度
            // 與新基線進行比較。規格::寫入儲存::插入
            // 僅在無效實體上傳回 Err — 可以安全地忽略。
            let mut snap = tw
                .mv_broadcasts
                .get(cand.entity)
                .cloned()
                .unwrap_or_default();
            snap.record(cand.target, cand.velocity, server_tick);
            let _ = tw.mv_broadcasts.insert(cand.entity, snap);
        }

        tw.outcomes.append(&mut outcomes);
        // 傷害計算 - 改為生成 Damage 事件
        for td in tw.taken_damages.iter() {
            if let Some(cp) = tr.cpropertys.get(td.ent) {
                // 記錄攻擊前狀態
                let hp_before = cp.hp;
                let max_hp = cp.mhp;

                // Phase 1c.4: cp.* / td.* / Outcome::Damage.{phys,magi,real} 全 Fixed64。
                let phys_raw = td.phys - cp.def_physic;
                let phys_damage: Fixed64 = if phys_raw < Fixed64::ZERO {
                    Fixed64::ZERO
                } else {
                    phys_raw
                };
                let magi_raw = td.magi - cp.def_magic;
                let magi_damage: Fixed64 = if magi_raw < Fixed64::ZERO {
                    Fixed64::ZERO
                } else {
                    magi_raw
                };
                let total_damage: Fixed64 = phys_damage + magi_damage;

                // 獲取目標名稱用於日誌
                let target_name = if let Some(creep) = tr.creeps.get(td.ent) {
                    creep.name.clone()
                } else {
                    // 暫時使用實體 ID，因為沒有在 Read 結構中包含 Hero
                    format!("Entity({:?})", td.ent.id())
                };

                if total_damage > Fixed64::ZERO {
                    // 階段 1c.4：Outcome::Damage.pos 是 SimVec2（階段 1c.2）。
                    let target_pos = tr.pos.get(td.ent).map(|p| p.0).unwrap_or(SimVec2::ZERO);

                    // 生成傷害事件（日誌將在 state.rs 中統一處理）
                    tw.outcomes.push(Outcome::Damage {
                        pos: target_pos,
                        phys: phys_damage,
                        magi: magi_damage,
                        real: Fixed64::ZERO,
                        source: td.source, // 使用正確的攻擊者
                        target: td.ent,
                        predeclared: false, // melee / on-touch damage — never pre-declared
                    });
                } else if td.phys > Fixed64::ZERO || td.magi > Fixed64::ZERO {
                    // 只有在有原始傷害但被完全防禦時才顯示
                    log::info!(
                        "🛡️ {} | Damage BLOCKED: Phys {:.1} vs Def {:.1}, Magi {:.1} vs Def {:.1}",
                        target_name,
                        td.phys.to_f32_for_render(),
                        cp.def_physic.to_f32_for_render(),
                        td.magi.to_f32_for_render(),
                        cp.def_magic.to_f32_for_render()
                    );
                }
            }
        }
        tw.taken_damages.clear();
    }
}
