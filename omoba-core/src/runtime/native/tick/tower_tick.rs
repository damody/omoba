use crate::comp::*;
use crate::tick::attack_phase::{
    advance_attack_phase, fixed_secs_to_ms, start_attack_windup, AttackPhaseStep,
};
use omoba_sim::Fixed64;
use specs::prelude::ParallelIterator;
use specs::Entity;
use specs::{shred, Entities, ParJoin, Read, ReadStorage, SystemData, Write, WriteStorage};
use std::time::Instant;

/// MOBA 鏡頭下肉眼無感的 facing 變化量（~15°）。舊值 0.05 (~3°) 造成過多 F event。
const FACING_BROADCAST_THRESHOLD_RAD: f32 = 0.26;

/// tower_tick 的每實體 SimRng op_kind。階段 1de.2：取代 fastrand
/// 無目標攻擊冷卻時間抖動。重新排序或重複使用該常數
/// 跨系統將使重播決定論無效。
const OP_TOWER_NO_TARGET_JITTER: u32 = 11;

#[derive(SystemData)]
pub struct TowerRead<'a> {
    entities: Entities<'a>,
    time: Read<'a, Time>,
    dt: Read<'a, DeltaTime>,
    master_seed: Read<'a, MasterSeed>,
    tick: Read<'a, Tick>,
    pos: ReadStorage<'a, Pos>,
    searcher: Read<'a, Searcher>,
    factions: ReadStorage<'a, Faction>,
    creeps: ReadStorage<'a, Creep>,
    cpropertys: ReadStorage<'a, CProperty>,
    turn_speeds: ReadStorage<'a, TurnSpeed>,
    // 有 ScriptUnitTag 的塔由腳本 on_tick 自主決策；tower_tick 只幫忙轉向
    script_tags: ReadStorage<'a, crate::scripting::ScriptUnitTag>,
}

#[derive(SystemData)]
pub struct TowerWrite<'a> {
    outcomes: Write<'a, Vec<Outcome>>,
    towers: WriteStorage<'a, Tower>,
    propertys: WriteStorage<'a, TProperty>,
    tatks: WriteStorage<'a, TAttack>,
    facings: WriteStorage<'a, Facing>,
    facing_bcs: WriteStorage<'a, FacingBroadcast>,
}

struct TowerDecisionRead<'a, 'b> {
    dt: Fixed64,
    dt_f: f32,
    master_seed: u64,
    tick: u32,
    time1: Instant,
    pos: &'b ReadStorage<'a, Pos>,
    searcher: &'b Searcher,
    factions: &'b ReadStorage<'a, Faction>,
    creeps: &'b ReadStorage<'a, Creep>,
    cpropertys: &'b ReadStorage<'a, CProperty>,
    turn_speeds: &'b ReadStorage<'a, TurnSpeed>,
    script_tags: &'b ReadStorage<'a, crate::scripting::ScriptUnitTag>,
}

#[derive(Debug)]
pub(crate) struct TowerTickDecision {
    entity: Entity,
    tower: Tower,
    property: TProperty,
    attack: TAttack,
    facing: Facing,
    facing_bc: FacingBroadcast,
    outcomes: Vec<Outcome>,
}

#[derive(Default)]
pub struct Sys;

impl<'a> System<'a> for Sys {
    type SystemData = (TowerRead<'a>, TowerWrite<'a>);

    const NAME: &'static str = "tower";

    fn run(_job: &mut Job<Self>, (tr, mut tw): Self::SystemData) {
        // 階段 1c.4：dt 在整個戰鬥週期中固定為 64。
        let dt: Fixed64 = tr.dt.0;
        let read = TowerDecisionRead {
            dt,
            dt_f: dt.to_f32_for_render(),
            master_seed: tr.master_seed.0,
            tick: tr.tick.0 as u32,
            time1: Instant::now(),
            pos: &tr.pos,
            searcher: &*tr.searcher,
            factions: &tr.factions,
            creeps: &tr.creeps,
            cpropertys: &tr.cpropertys,
            turn_speeds: &tr.turn_speeds,
            script_tags: &tr.script_tags,
        };

        let mut decisions = (
            &tr.entities,
            &tw.towers,
            &tw.propertys,
            &tw.tatks,
            &tr.pos,
            &tw.facings,
            &tw.facing_bcs,
        )
            .par_join()
            .map_init(
                || {
                    prof_span!(guard, "tower update rayon job");
                    guard
                },
                |_guard, (e, tower, pty, atk, pos, facing, facing_bc)| {
                    decide_tower_tick(&read, e, tower, pty, atk, pos, facing, facing_bc)
                },
            )
            .fold(Vec::new, |mut all, decision| {
                all.push(decision);
                all
            })
            .reduce(Vec::new, |mut a, mut b| {
                a.append(&mut b);
                a
            });

        decisions.sort_by_key(|decision| decision.entity.id());
        let mut outcomes = Vec::new();
        for mut decision in decisions {
            if let Some(tower) = tw.towers.get_mut(decision.entity) {
                *tower = decision.tower;
            }
            if let Some(property) = tw.propertys.get_mut(decision.entity) {
                *property = decision.property;
            }
            if let Some(attack) = tw.tatks.get_mut(decision.entity) {
                *attack = decision.attack;
            }
            if let Some(facing) = tw.facings.get_mut(decision.entity) {
                *facing = decision.facing;
            }
            if let Some(facing_bc) = tw.facing_bcs.get_mut(decision.entity) {
                *facing_bc = decision.facing_bc;
            }
            outcomes.append(&mut decision.outcomes);
        }
        tw.outcomes.append(&mut outcomes);
    }
}

#[allow(clippy::too_many_arguments)]
fn decide_tower_tick(
    tr: &TowerDecisionRead<'_, '_>,
    e: Entity,
    tower_src: &Tower,
    pty_src: &TProperty,
    atk_src: &TAttack,
    pos: &Pos,
    facing_src: &Facing,
    facing_bc_src: &FacingBroadcast,
) -> TowerTickDecision {
    let mut tower = tower_src.clone();
    let mut pty = *pty_src;
    let mut atk = *atk_src;
    let mut facing = *facing_src;
    let mut facing_bc = *facing_bc_src;
    let mut outcomes: Vec<Outcome> = Vec::new();

    tower.ultimate_cooldown = (tower.ultimate_cooldown - tr.dt).max(Fixed64::ZERO);

    // 注意：搜尋器內部使用 f32 來實作 instant_distance lib 相容性；呼叫者的最終距離檢查是固定64。
    let (pos_x_f, pos_y_f) = pos.xy_f32();
    let pos_vek = vek::Vec2::new(pos_x_f, pos_y_f);

    // 腳本塔：開火/asd_count 由 on_tick 自管；非腳本塔：host 管全部。
    let is_scripted = tr.script_tags.get(e).is_some();
    let attack_phase = if is_scripted {
        AttackPhaseStep::Charging
    } else {
        advance_attack_phase(&mut atk.asd_count, tr.dt, atk.asd.val())
    };
    if matches!(attack_phase, AttackPhaseStep::Ready) {
        atk.clear_attack_sequence();
    }

    if pty.mblock > 0 {
        let stale_block_creeps: Vec<Entity> = tower
            .block_creeps
            .iter()
            .copied()
            .filter(|blocked| tr.pos.get(*blocked).is_none())
            .collect();
        tower.block_creeps = tower
            .block_creeps
            .iter()
            .copied()
            .filter(|blocked| !stale_block_creeps.contains(blocked))
            .collect();
        pty.block = tower.block_creeps.len() as i32;
    }
    if pty.mblock > pty.block {
        let size_sq: Fixed64 = pty.size * pty.size;
        for nc in tower.nearby_creeps.iter() {
            if tower.block_creeps.contains(&nc.ent) {
                continue;
            }
            if let Some(p) = tr.pos.get(nc.ent) {
                let diff = p.0 - pos.0;
                if diff.length_squared() < size_sq {
                    tower.block_creeps.push(nc.ent);
                    outcomes.push(Outcome::CreepStop {
                        source: e,
                        target: nc.ent,
                    });
                }
            }
        }
    }

    let do_seek = !is_scripted && !matches!(attack_phase, AttackPhaseStep::Charging);
    if do_seek {
        let time2 = Instant::now();
        let elpsed = time2.duration_since(tr.time1);
        if elpsed.as_secs_f32() < 0.05 {
            let search_n = tr.searcher.creep.count().max(1);
            let range_f = atk.range.val().to_f32_for_render();
            let (creeps, near_creeps) =
                tr.searcher
                    .creep
                    .search_nn_two_radii(pos_vek, range_f, range_f + 30., search_n);

            let my_faction = tr.factions.get(e);
            let hostile_creeps: Vec<_> = creeps
                .iter()
                .filter(|ci| {
                    let hostile = match (my_faction, tr.factions.get(ci.e)) {
                        (Some(mf), Some(tf)) => mf.is_hostile_to(tf),
                        (None, _) => true,
                        (_, None) => true,
                    };
                    hostile
                        && tr
                            .creeps
                            .get(ci.e)
                            .map(|creep| tower_can_target_creep(&tower, creep))
                            .unwrap_or(true)
                })
                .collect();

            if !hostile_creeps.is_empty() {
                if pty.mblock > 0 {
                    tower.nearby_creeps.clear();
                    for c in hostile_creeps.iter() {
                        let dis_fx = Fixed64::from_raw((c.dis * 1024.0) as i64);
                        tower.nearby_creeps.push(NearbyEnt {
                            ent: c.e,
                            dis: dis_fx,
                        });
                    }
                }
                let target_entity = select_tower_target(
                    tower.target_priority,
                    &hostile_creeps,
                    tr.creeps,
                    tr.cpropertys,
                )
                .unwrap_or_else(|| hostile_creeps[0].e);
                let target_pos = tr
                    .pos
                    .get(target_entity)
                    .map(|p| {
                        let (x, y) = p.xy_f32();
                        vek::Vec2::new(x, y)
                    })
                    .unwrap_or(pos_vek);
                let diff = target_pos - pos_vek;
                if diff.magnitude_squared() > 0.01 {
                    let desired = diff.y.atan2(diff.x);
                    let turn = tr
                        .turn_speeds
                        .get(e)
                        .map(|t| t.0.to_f32_for_render())
                        .unwrap_or(std::f32::consts::FRAC_PI_2);
                    let cur_rad = facing.rad_f32();
                    let new_rad = rotate_toward(cur_rad, desired, turn * tr.dt_f);
                    facing = Facing::from_rad_f32(new_rad);

                    let needs_emit = match facing_bc.0 {
                        None => true,
                        Some(last) => (new_rad - last).abs() > FACING_BROADCAST_THRESHOLD_RAD,
                    };
                    if needs_emit {
                        facing_bc.0 = Some(new_rad);
                    }

                    if normalize_angle(desired - new_rad).abs() < MOVE_ANGLE_THRESHOLD {
                        if matches!(attack_phase, AttackPhaseStep::Ready) {
                            let (windup, backswing) =
                                start_attack_windup(&mut atk.asd_count, atk.asd.val());
                            let attack_seq = atk.begin_attack_windup();
                            outcomes.push(Outcome::AttackPhaseCue {
                                entity: e,
                                attack_seq,
                                is_critical: false,
                                target: Some(target_entity),
                                target_pos: tr.pos.get(target_entity).map(|p| p.0),
                                windup_ms: fixed_secs_to_ms(windup),
                                backswing_ms: fixed_secs_to_ms(backswing),
                                dir_rad: desired,
                            });
                        } else {
                            atk.mark_attack_impact();
                            outcomes.push(Outcome::ProjectileLine2 {
                                pos: pos.0,
                                source: Some(e),
                                target: Some(target_entity),
                            });
                        }
                    }
                }
            } else if matches!(attack_phase, AttackPhaseStep::Ready) && near_creeps.is_empty() {
                let mut rng = omoba_sim::SimRng::from_master_entity(
                    tr.master_seed,
                    tr.tick,
                    e.id(),
                    OP_TOWER_NO_TARGET_JITTER,
                );
                let jitter = Fixed64::from_raw((rng.next_u32() % 256) as i64);
                atk.asd_count = atk.asd.val() - Fixed64::from_raw(307) - jitter;
            }
        }
    }

    TowerTickDecision {
        entity: e,
        tower,
        property: pty,
        attack: atk,
        facing,
        facing_bc,
        outcomes,
    }
}

pub(crate) fn select_tower_target(
    priority: TowerTargetPriority,
    candidates: &[&DisIndex],
    creeps: &ReadStorage<'_, Creep>,
    cpropertys: &ReadStorage<'_, CProperty>,
) -> Option<Entity> {
    candidates
        .iter()
        .copied()
        .min_by(|a, b| compare_tower_targets(priority, a, b, creeps, cpropertys))
        .map(|candidate| candidate.e)
}

fn compare_tower_targets(
    priority: TowerTargetPriority,
    a: &DisIndex,
    b: &DisIndex,
    creeps: &ReadStorage<'_, Creep>,
    cpropertys: &ReadStorage<'_, CProperty>,
) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let primary = match priority {
        TowerTargetPriority::First => {
            let ar = creeps
                .get(a.e)
                .map(|c| c.path_remaining_distance)
                .unwrap_or_else(|| Fixed64::from_i32(1_000_000));
            let br = creeps
                .get(b.e)
                .map(|c| c.path_remaining_distance)
                .unwrap_or_else(|| Fixed64::from_i32(1_000_000));
            ar.partial_cmp(&br).unwrap_or(Ordering::Equal)
        }
        TowerTargetPriority::Last => {
            let ar = creeps
                .get(a.e)
                .map(|c| c.path_remaining_distance)
                .unwrap_or_else(|| Fixed64::from_i32(1_000_000));
            let br = creeps
                .get(b.e)
                .map(|c| c.path_remaining_distance)
                .unwrap_or_else(|| Fixed64::from_i32(1_000_000));
            br.partial_cmp(&ar).unwrap_or(Ordering::Equal)
        }
        TowerTargetPriority::Nearest => a.dis.partial_cmp(&b.dis).unwrap_or(Ordering::Equal),
        TowerTargetPriority::Farthest => b.dis.partial_cmp(&a.dis).unwrap_or(Ordering::Equal),
        TowerTargetPriority::HighestHealth => {
            let ahp = cpropertys.get(a.e).map(|p| p.hp).unwrap_or(Fixed64::ZERO);
            let bhp = cpropertys.get(b.e).map(|p| p.hp).unwrap_or(Fixed64::ZERO);
            bhp.partial_cmp(&ahp).unwrap_or(Ordering::Equal)
        }
        TowerTargetPriority::LowestHealth => {
            let ahp = cpropertys.get(a.e).map(|p| p.hp).unwrap_or(Fixed64::ZERO);
            let bhp = cpropertys.get(b.e).map(|p| p.hp).unwrap_or(Fixed64::ZERO);
            ahp.partial_cmp(&bhp).unwrap_or(Ordering::Equal)
        }
    };
    primary.then_with(|| a.e.id().cmp(&b.e.id()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn tower_internal_cooldown_ticks_once_with_authoritative_fixed_dt() {
        let mut world = World::new();
        world.register::<Tower>();
        world.register::<TProperty>();
        world.register::<TAttack>();
        world.register::<Pos>();
        world.register::<Facing>();
        world.register::<FacingBroadcast>();
        world.register::<Faction>();
        world.register::<Creep>();
        world.register::<CProperty>();
        world.register::<TurnSpeed>();
        world.register::<crate::scripting::ScriptUnitTag>();
        world.insert(Time::default());
        world.insert(DeltaTime(Fixed64::from_raw(512)));
        world.insert(MasterSeed::default());
        world.insert(Tick::default());
        world.insert(Searcher::default());
        world.insert(Vec::<Outcome>::new());
        world.insert(crate::comp::SysMetrics::default());
        world.insert(crate::comp::TickProfile::default());

        let mut tower = Tower::new();
        tower.ultimate_cooldown = Fixed64::from_i32(2);
        let entity = world
            .create_entity()
            .with(tower)
            .with(TProperty::new(Fixed64::ONE, 0, Fixed64::ONE))
            .with(TAttack::new(
                Fixed64::ONE,
                Fixed64::ONE,
                Fixed64::ONE,
                Fixed64::ONE,
            ))
            .with(Pos(omoba_sim::Vec2::ZERO))
            .with(Facing::default())
            .with(FacingBroadcast::default())
            .build();

        crate::comp::run_now::<Sys>(&world);

        assert_eq!(
            world
                .read_storage::<Tower>()
                .get(entity)
                .unwrap()
                .ultimate_cooldown,
            Fixed64::from_raw(1536)
        );
    }

    fn add_creep(world: &mut World, remaining: i32, hp: i32) -> Entity {
        world
            .create_entity()
            .with(Creep {
                name: "test".to_string(),
                label: None,
                path: "p".to_string(),
                pidx: 0,
                path_remaining_distance: Fixed64::from_i32(remaining),
                block_tower: None,
                status: CreepStatus::Walk,
                td_layer: None,
            })
            .with(CProperty {
                hp: Fixed64::from_i32(hp),
                mhp: Fixed64::from_i32(hp),
                msd: Fixed64::ZERO,
                def_physic: Fixed64::ZERO,
                def_magic: Fixed64::ZERO,
            })
            .build()
    }

    #[test]
    fn tower_priority_selects_by_path_rank_distance_health_and_entity_tie() {
        let mut world = World::new();
        world.register::<Creep>();
        world.register::<CProperty>();
        let first = add_creep(&mut world, 10, 50);
        let last = add_creep(&mut world, 80, 200);
        let low_hp = add_creep(&mut world, 40, 5);
        let candidates = vec![
            DisIndex { e: first, dis: 9.0 },
            DisIndex { e: last, dis: 1.0 },
            DisIndex {
                e: low_hp,
                dis: 4.0,
            },
        ];
        let refs: Vec<_> = candidates.iter().collect();
        let creeps = world.read_storage::<Creep>();
        let props = world.read_storage::<CProperty>();

        assert_eq!(
            select_tower_target(TowerTargetPriority::First, &refs, &creeps, &props),
            Some(first)
        );
        assert_eq!(
            select_tower_target(TowerTargetPriority::Last, &refs, &creeps, &props),
            Some(last)
        );
        assert_eq!(
            select_tower_target(TowerTargetPriority::Nearest, &refs, &creeps, &props),
            Some(last)
        );
        assert_eq!(
            select_tower_target(TowerTargetPriority::Farthest, &refs, &creeps, &props),
            Some(first)
        );
        assert_eq!(
            select_tower_target(TowerTargetPriority::HighestHealth, &refs, &creeps, &props),
            Some(last)
        );
        assert_eq!(
            select_tower_target(TowerTargetPriority::LowestHealth, &refs, &creeps, &props),
            Some(low_hp)
        );
    }
}
