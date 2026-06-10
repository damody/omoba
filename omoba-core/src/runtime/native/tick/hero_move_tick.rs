use omoba_sim::trig::{angle_rotate_toward, atan2 as sim_atan2, fixed_rad_to_ticks, TAU_TICKS};
use omoba_sim::{Angle, Fixed64, Vec2 as SimVec2};
use specs::prelude::ParallelIterator;
use specs::{shred, Entities, ParJoin, Read, ReadStorage, SystemData, WriteStorage};

use crate::comp::phys::MAX_COLLISION_RADIUS;
use crate::comp::*;
use std::sync::atomic::{AtomicU64, Ordering};

static TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(SystemData)]
pub struct HeroMoveRead<'a> {
    entities: Entities<'a>,
    dt: Read<'a, DeltaTime>,
    heroes: ReadStorage<'a, Hero>,
    propertys: ReadStorage<'a, CProperty>,
    turn_speeds: ReadStorage<'a, TurnSpeed>,
    radii: ReadStorage<'a, CollisionRadius>,
    searcher: Read<'a, Searcher>,
    /// Debug only：驗證 hero 是否進入 polygon 但未被 blocker 擋
    regions: Read<'a, BlockedRegions>,
    buff_store: Read<'a, omoba_core::runtime::ability_runtime::BuffStore>,
    is_buildings: ReadStorage<'a, IsBuilding>,
}

#[derive(SystemData)]
pub struct HeroMoveWrite<'a> {
    pos: WriteStorage<'a, Pos>,
    move_targets: WriteStorage<'a, MoveTarget>,
    facings: WriteStorage<'a, Facing>,
}

#[derive(Default)]
pub struct Sys;

/// 檢查若單位移動到 `new_center` 是否會撞進任何其他有 CollisionRadius 的實體。
/// Region 阻擋透過 blocker entities 一起走 Searcher 查詢，不再需要 polygon 測試。
pub(crate) fn hits_any(
    new_center: SimVec2,
    radius: Fixed64,
    searcher: &Searcher,
    radii: &ReadStorage<CollisionRadius>,
    self_entity: specs::Entity,
    _regions: &BlockedRegions,
) -> bool {
    // 注意：搜尋器內部使用 f32 來實作 instant_distance lib 相容性；呼叫者的最終距離檢查是固定64。
    let radius_f = radius.to_f32_for_render();
    let q_r = radius_f + MAX_COLLISION_RADIUS;
    let center_vek = vek::Vec2::new(
        new_center.x.to_f32_for_render(),
        new_center.y.to_f32_for_render(),
    );
    for di in searcher.search_collidable(center_vek, q_r, 16) {
        if di.e == self_entity {
            continue;
        }
        let Some(other_r) = radii.get(di.e).map(|cr| cr.0) else {
            continue;
        };
        // touch = radius + other_r — 保持固定64 以便與呼叫者保持一致。
        let touch = radius + other_r;
        let touch_f = touch.to_f32_for_render();
        if di.dis < touch_f * touch_f {
            return true;
        }
    }
    false
}

/// 計算避開其他單位的下一步位置：嘗試直接走 → 只走 X → 只走 Y → 停。
/// 回傳 (新位置, 是否抵達目標範圍)。
pub fn advance_with_collision(
    pos: SimVec2,
    target: SimVec2,
    step: Fixed64,
    radius: Fixed64,
    searcher: &Searcher,
    radii: &ReadStorage<CollisionRadius>,
    self_entity: specs::Entity,
    regions: &BlockedRegions,
) -> (SimVec2, bool) {
    let diff = target - pos;
    let distance = diff.length();
    // 0.5 = 固定64::from_raw(512)
    let arrived_eps = Fixed64::from_raw(512);
    if distance < arrived_eps {
        return (target, true);
    }
    // Normalized() 在內部處理零—但我們已經提前確定了距離 < 0.5。
    let direction = diff.normalized();
    // step.max(1.0) → 如果step < 1，則將閾值視為1.0
    let one = Fixed64::ONE;
    let snap_threshold = if step > one { step } else { one };
    if distance <= snap_threshold {
        if !hits_any(target, radius, searcher, radii, self_entity, regions) {
            return (target, true);
        }
        return (pos, false);
    }
    let full = pos + direction * step;
    if !hits_any(full, radius, searcher, radii, self_entity, regions) {
        return (full, false);
    }
    let only_x = SimVec2::new(pos.x + direction.x * step, pos.y);
    if !hits_any(only_x, radius, searcher, radii, self_entity, regions) {
        return (only_x, false);
    }
    let only_y = SimVec2::new(pos.x, pos.y + direction.y * step);
    if !hits_any(only_y, radius, searcher, radii, self_entity, regions) {
        return (only_y, false);
    }
    (pos, false)
}

impl<'a> System<'a> for Sys {
    type SystemData = (HeroMoveRead<'a>, HeroMoveWrite<'a>);

    const NAME: &'static str = "hero_move";

    fn run(_job: &mut Job<Self>, (tr, mut tw): Self::SystemData) {
        let dt = tr.dt.0;
        if dt <= Fixed64::ZERO {
            return;
        }
        // dt_f 僅保留用於舊版 f32 廣播有線格式 + CProperty.msd
        // （仍然是 f32；階段 1c 將遷移）。角度數學現在完全是固定64/角度。
        let dt_f = dt.to_f32_for_render();

        // 每 120 tick (~2s) log 一次 searcher 各 index 大小，確認 region 已載入
        let t = TICK_COUNTER.fetch_add(1, Ordering::Relaxed);
        if t % 120 == 0 {
            log::debug!(
                "🔍 searcher sizes: hero={}, creep={}, tower={}, region={}",
                tr.searcher.hero.count(),
                tr.searcher.creep.count(),
                tr.searcher.tower.count(),
                tr.searcher.region.count()
            );
        }

        // par_join 並行處理所有 hero — 各 hero 的 collision query 是 Searcher 的 read-only
        // 操作，可安全並行；&mut tw.pos / &mut tw.facings 由 specs 保證同 entity 只被一個
        // thread 寫入。collect 結果後再一次性 remove move_targets。
        // 注意：ParJoin 在這裡是確定性安全的——每個英雄只寫入自己的 pos/face 儲存槽
        // （規範強制每個實體隔離），且每個實體的固定64/角度數學是與順序無關的。
        // 收集到的「結果」Vec 排序僅是有線格式（廣播順序）；鎖步狀態不受影響。
        let results: Vec<(Option<specs::Entity>, (u32, f32, f32, f32))> = (
            &tr.entities,
            &tr.heroes,
            &tr.propertys,
            &mut tw.pos,
            &tw.move_targets,
            &mut tw.facings,
        )
            .par_join()
            .map_init(
                || {
                    prof_span!(guard, "hero_move rayon job");
                    guard
                },
                |_guard, (entity, _hero, property, pos, move_target, facing)| {
                    // 廣播值（傳統 f32 有線格式）。
                    let pos_x_f = pos.0.x.to_f32_for_render();
                    let pos_y_f = pos.0.y.to_f32_for_render();
                    let facing_rad_out = angle_to_rad_f32(facing.0);

                    // Root / stun 狀態：完全凍結（不轉向、不位移、不消耗 MoveTarget）
                    if tr.buff_store.is_rooted(entity) {
                        return (None, (entity.id(), pos_x_f, pos_y_f, facing_rad_out));
                    }

                    let target = move_target.0;
                    let diff = target - pos.0;
                    let distance = diff.length();
                    // 用 UnitStats 聚合移速（對應 Dota MOVESPEED_BONUS_* / MOVESPEED_ABSOLUTE /
                    // MOVESPEED_MAX/MIN/LIMIT）；建築物會被 is_buildings 跳過（hero 不會）。
                    // CProperty.msd 仍然是 f32（第 1c 階段遷移）；保留 f32 路徑。
                    let stats = omoba_core::runtime::ability_runtime::UnitStats::from_refs(
                        &*tr.buff_store,
                        tr.is_buildings.get(entity).is_some(),
                    );
                    // 階段 1c.4：CProperty.msd / Final_move_speed 為 Fix64（階段 1c.2 / 1c.3）。
                    // dt 是固定64。 step = effective_msd * dt — 總是保持固定64。
                    let effective_msd: Fixed64 = stats.final_move_speed(property.msd, entity);
                    let step: Fixed64 = effective_msd * dt;

                    let mut arrived_entity: Option<specs::Entity> = None;

                    // 距離 > 0.5 — 固定 64 from_raw(512) = 0.5
                    if distance > Fixed64::from_raw(512) {
                        // 使用確定性的 Fix64 atan2 計算所需的朝向。
                        let desired_angle: Angle = sim_atan2(diff.y, diff.x);
                        let turn_rate = tr
                            .turn_speeds
                            .get(entity)
                            .map(|t| t.0)
                            .unwrap_or(Fixed64::from_raw(1608)); // π/2 rad/s default
                                                                 // 透過確定性助手轉換（rad/s × s）Fixed64 → 角度刻度。
                        let max_step_ticks = fixed_rad_to_ticks(turn_rate * dt);
                        facing.0 = angle_rotate_toward(facing.0, desired_angle, max_step_ticks);

                        // 面向夾角 < 30° 才能前進 — compare in Angle ticks.
                        let diff_ticks =
                            (desired_angle.ticks() - facing.0.ticks()).rem_euclid(TAU_TICKS);
                        let signed_diff_ticks = if diff_ticks > TAU_TICKS / 2 {
                            diff_ticks - TAU_TICKS
                        } else {
                            diff_ticks
                        };
                        if signed_diff_ticks.abs() < MOVE_ANGLE_THRESHOLD_TICKS {
                            let radius = tr
                                .radii
                                .get(entity)
                                .map(|r| r.0)
                                .unwrap_or(Fixed64::from_i32(30));
                            let (new_pos, reached) = advance_with_collision(
                                pos.0,
                                target,
                                step,
                                radius,
                                &tr.searcher,
                                &tr.radii,
                                entity,
                                &tr.regions,
                            );
                            pos.0 = new_pos;
                            if reached {
                                arrived_entity = Some(entity);
                            }
                        }
                        // 角度太大 → 只轉向、不位移（本 tick 不動）
                    } else {
                        arrived_entity = Some(entity);
                    }

                    // 廣播值使用後步位置+後旋轉面向。
                    let out_x = pos.0.x.to_f32_for_render();
                    let out_y = pos.0.y.to_f32_for_render();
                    let out_facing = angle_to_rad_f32(facing.0);
                    (arrived_entity, (entity.id(), out_x, out_y, out_facing))
                },
            )
            .collect();

        // 移除已到達的 MoveTarget（par_join 結束後序列 mutation 才安全）
        for (arrived, _) in &results {
            if let Some(entity) = arrived {
                tw.move_targets.remove(*entity);
            }
        }
    }
}

/// 角度 → f32 弧度用於舊版 Hero.M 線格式。有損邊界；
/// 內部模擬數學現在保留在角度中。
#[inline]
fn angle_to_rad_f32(a: omoba_sim::Angle) -> f32 {
    (a.ticks() as f32 / TAU_TICKS as f32) * std::f32::consts::TAU
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_core::runtime::ability_runtime::BuffStore;
    use specs::{Builder, Join, World, WorldExt};

    fn movement_world() -> (World, specs::Entity) {
        let mut world = World::new();
        world.register::<Faction>();
        world.register::<HeroCommandQueue>();
        world.register::<Hero>();
        world.register::<CProperty>();
        world.register::<CollisionRadius>();
        world.register::<Facing>();
        world.register::<IsBuilding>();
        world.register::<MoveTarget>();
        world.register::<PlayerOwner>();
        world.register::<Pos>();
        world.register::<TAttack>();
        world.register::<TurnSpeed>();
        world.insert(DeltaTime(Fixed64::from_raw(1024 / 30)));
        world.insert(Tick(0));
        world.insert(Time::default());
        world.insert(GamePause::default());
        world.insert(CurrentCreepWave::default());
        world.insert(PendingMoveQueue::default());
        world.insert(PendingTowerSpawnQueue::default());
        world.insert(PendingTowerSellQueue::default());
        world.insert(PendingTowerUpgradeQueue::default());
        world.insert(PendingAbilityUpgradeQueue::default());
        world.insert(PendingAbilityCastQueue::default());
        world.insert(PendingItemUseQueue::default());
        world.insert(PendingHeroCommandClearQueue::default());
        world.insert(PendingTowerTargetPriorityQueue::default());
        world.insert(Searcher::default());
        world.insert(BlockedRegions::default());
        world.insert(BuffStore::default());
        world.insert(crate::comp::SysMetrics::default());
        world.insert(crate::comp::TickProfile::default());

        let hero = world
            .create_entity()
            .with(Hero::default())
            .with(Faction::new(FactionType::Player, 0))
            .with(PlayerOwner::new(1))
            .with(CProperty {
                hp: Fixed64::from_i32(100),
                mhp: Fixed64::from_i32(100),
                msd: Fixed64::from_i32(120),
                def_physic: Fixed64::ZERO,
                def_magic: Fixed64::ZERO,
            })
            .with(CollisionRadius(Fixed64::from_i32(10)))
            .with(Facing(Angle::ZERO))
            .with(TurnSpeed(Fixed64::from_raw(1608)))
            .with(Pos(SimVec2::new(Fixed64::ZERO, Fixed64::ZERO)))
            .build();

        (world, hero)
    }

    #[test]
    fn hero_with_move_target_advances_position() {
        let (mut world, hero) = movement_world();
        let _ = world.write_storage::<MoveTarget>().insert(
            hero,
            MoveTarget(SimVec2::new(Fixed64::from_i32(100), Fixed64::ZERO)),
        );
        let start = world.read_storage::<Pos>().get(hero).unwrap().0;

        crate::comp::run_now::<Sys>(&world);
        world.maintain();

        let end = world.read_storage::<Pos>().get(hero).unwrap().0;
        assert!(
            end.x > start.x,
            "expected hero x to advance, start={:?} end={:?}",
            start,
            end
        );
    }

    #[test]
    fn pending_move_command_advances_hero_after_command_tick() {
        let (mut world, hero) = movement_world();
        let start = world.read_storage::<Pos>().get(hero).unwrap().0;
        {
            let mut q = world.write_resource::<PendingMoveQueue>();
            q.requests.push(PendingHeroCommand {
                owner_pid: 1,
                queued: false,
                kind: PendingHeroCommandKind::MoveTo {
                    pos: SimVec2::new(Fixed64::from_i32(100), Fixed64::ZERO),
                },
            });
        }

        crate::runtime::drain_pending_moves(&mut world);
        world.maintain();
        crate::comp::run_now::<crate::tick::hero_command_tick::Sys>(&world);
        world.maintain();
        crate::comp::run_now::<Sys>(&world);
        world.maintain();

        let end = world.read_storage::<Pos>().get(hero).unwrap().0;
        assert!(
            end.x > start.x,
            "expected pending MoveTo to advance hero, start={:?} end={:?}",
            start,
            end
        );
    }

    #[test]
    fn lockstep_move_input_advances_on_following_tick() {
        let (mut world, hero) = movement_world();
        world.insert(PendingPlayerInputs {
            tick: 1,
            inputs: vec![(
                1,
                crate::runtime::PlayerInput {
                    action: Some(crate::runtime::PlayerInputEnum::MoveTo(
                        crate::runtime::MoveTo {
                            target: Some(crate::runtime::Vec2I {
                                x: 100 * 1024,
                                y: 0,
                            }),
                            queued: false,
                        },
                    )),
                },
            )],
        });
        let start = world.read_storage::<Pos>().get(hero).unwrap().0;

        world.write_resource::<Tick>().0 = 1;
        crate::comp::run_now::<crate::tick::player_input_tick::Sys>(&world);
        world.maintain();
        crate::runtime::drain_pending_moves(&mut world);
        world.maintain();

        world.write_resource::<PendingPlayerInputs>().inputs.clear();
        world.write_resource::<Tick>().0 = 2;
        crate::comp::run_now::<crate::tick::hero_command_tick::Sys>(&world);
        world.maintain();
        crate::comp::run_now::<Sys>(&world);
        world.maintain();

        let end = world.read_storage::<Pos>().get(hero).unwrap().0;
        assert!(
            end.x > start.x,
            "expected lockstep MoveTo to advance next tick, start={:?} end={:?}",
            start,
            end
        );
    }

    #[test]
    fn td1_initialized_hero_moves_after_pending_move() {
        let scene = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repo root")
            .join("scripts/lua_data/TD_1");
        if !scene.exists() {
            eprintln!("skipping TD_1 movement test: {} missing", scene.display());
            return;
        }

        let mut world =
            crate::runtime::create_world_for_scene(&scene).expect("TD_1 world should initialize");
        let hero = {
            let entities = world.entities();
            let heroes = world.read_storage::<Hero>();
            let owners = world.read_storage::<PlayerOwner>();
            (&entities, &heroes, &owners)
                .join()
                .find_map(|(entity, _hero, owner)| (owner.player_id == 1).then_some(entity))
                .expect("player 1 hero")
        };
        let start = world.read_storage::<Pos>().get(hero).unwrap().0;
        {
            let mut q = world.write_resource::<PendingMoveQueue>();
            q.requests.push(PendingHeroCommand {
                owner_pid: 1,
                queued: false,
                kind: PendingHeroCommandKind::MoveTo {
                    pos: start + SimVec2::new(Fixed64::from_i32(120), Fixed64::ZERO),
                },
            });
        }

        crate::runtime::drain_pending_moves(&mut world);
        world.maintain();
        let mut dispatcher = crate::runtime::build_phase3_dispatcher().expect("dispatcher");
        for tick in 1..=20u64 {
            world.write_resource::<Tick>().0 = tick;
            world.write_resource::<DeltaTime>().0 = Fixed64::from_raw(1024 / 30);
            dispatcher.dispatch(&world);
            world.maintain();
        }

        let end = world.read_storage::<Pos>().get(hero).unwrap().0;
        assert!(
            (end - start).length() > Fixed64::from_i32(50),
            "expected TD_1 hero to move at gameplay speed, start={:?} end={:?}",
            start,
            end
        );
    }
}
