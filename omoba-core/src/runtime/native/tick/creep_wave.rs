use crate::comp::*;
use omoba_core::runtime::{
    RuntimeBroadcast, RuntimeEvent, RuntimeEvents, TdEconomyCategory, TdEconomyLedger,
    TdEconomyRules,
};
use omoba_sim::Fixed64;
use serde_json::json;
use specs::{shred, Join, Read, ReadStorage, SystemData, Write};
use std::collections::BTreeMap;

#[derive(SystemData)]
pub struct CreepWaveRead<'a> {
    time: Read<'a, Time>,
    dt: Read<'a, DeltaTime>,
    creep_emiters: Read<'a, BTreeMap<String, CreepEmiter>>,
    paths: Read<'a, BTreeMap<String, Path>>,
    check_points: Read<'a, BTreeMap<String, CheckPoint>>,
    creeps: ReadStorage<'a, Creep>,
    game_mode: Read<'a, GameMode>,
    tick: Read<'a, Tick>,
    economy_rules: Read<'a, TdEconomyRules>,
}

#[derive(SystemData)]
pub struct CreepWaveWrite<'a> {
    outcomes: Write<'a, Vec<Outcome>>,
    cur_creep_wave: Write<'a, CurrentCreepWave>,
    creep_waves: Write<'a, Vec<CreepWave>>,
    runtime_events: Write<'a, RuntimeEvents>,
    debug_spawns: Write<'a, PendingDebugCreepSpawnQueue>,
    player_economy: Write<'a, PlayerEconomy>,
    economy_ledger: Write<'a, TdEconomyLedger>,
}

#[derive(Default)]
pub struct Sys;

impl<'a> System<'a> for Sys {
    type SystemData = (CreepWaveRead<'a>, CreepWaveWrite<'a>);

    const NAME: &'static str = "creep_wave";

    fn run(_job: &mut Job<Self>, (tr, mut tw): Self::SystemData) {
        let totaltime = tr.time.0;
        let is_td = tr.game_mode.is_td();

        // 沙箱/測試生怪：drain PendingDebugCreepSpawnQueue（Ctrl+數字熱鍵）。
        // 必須在 wave 早退判斷之前跑，全部波打完後仍可測試。
        if !tw.debug_spawns.requests.is_empty() {
            let requests: Vec<_> = tw.debug_spawns.requests.drain(..).collect();
            for req in requests {
                let Some((emitter_name, cp)) =
                    tr.creep_emiters.iter().nth(req.emitter_index as usize)
                else {
                    log::warn!(
                        "debug_spawn_creep: emitter_index={} 超出範圍（共 {} 種）",
                        req.emitter_index,
                        tr.creep_emiters.len()
                    );
                    continue;
                };
                let Some((path_name, path)) = tr.paths.iter().next() else {
                    log::warn!("debug_spawn_creep: 地圖沒有任何 path，無法生怪");
                    continue;
                };
                let Some(pos) = path.check_points_sim.first().cloned() else {
                    continue;
                };
                for _ in 0..req.count {
                    let mut cpp = cp.root.clone();
                    cpp.path = path_name.clone();
                    let cp0 = CreepData {
                        pos,
                        creep: cpp,
                        cdata: cp.property.clone(),
                        faction_name: cp.faction_name.clone(),
                        turn_speed_deg: Fixed64::from_raw(
                            (cp.turn_speed_deg * omoba_sim::fixed::SCALE as f32) as i64,
                        ),
                        collision_radius: Fixed64::from_raw(
                            (cp.collision_radius * omoba_sim::fixed::SCALE as f32) as i64,
                        ),
                    };
                    tw.outcomes.push(Outcome::Creep { cd: cp0 });
                }
                log::info!(
                    "debug_spawn_creep: 生成 {} x{}（path={}）",
                    emitter_name,
                    req.count,
                    path_name
                );
            }
        }

        let mut cw = tw.cur_creep_wave;
        if cw.wave >= tw.creep_waves.len() {
            return;
        }
        let Some(w) = tw.creep_waves.get(cw.wave) else {
            return;
        };

        // TD 模式：只有按 StartRound 後 is_running=true 才出怪；
        // 波的參考開始時間改用 `cw.wave_start_time`（按下時記錄的 totaltime）。
        // 非 TD：沿用原時間觸發（`w.time` 絕對開始時間）。
        let ref_time = if is_td { cw.wave_start_time } else { w.time };
        let can_run = if is_td {
            cw.is_running
        } else {
            w.time < totaltime as f32
        };
        if !can_run {
            return;
        }

        if cw.path.is_empty() {
            cw.path.resize(w.path_creeps.len(), 0);
        }

        let mut is_end = true;
        let mut spawned_this_tick = false;
        for (i, pc) in w.path_creeps.iter().enumerate() {
            // Drain every occurrence whose authored time is due. This is
            // essential for coarse 66.667 ms ticks and is bounded by the
            // content-derived number of entries in this path, so malformed
            // content cannot create an unbounded runtime loop.
            let mut drained = 0usize;
            while cw.path[i] < pc.creeps.len()
                && pc.creeps[cw.path[i]].time + ref_time < totaltime as f32
            {
                let cur_path_idx = cw.path[i];
                let cp = tr.creep_emiters.get(&pc.creeps[cur_path_idx].name);
                let path = tr.paths.get(&pc.path_name);
                if let (Some(cp), Some(path)) = (cp, path) {
                    if let Some(pos) = path.check_points_sim.first().cloned() {
                        let mut cpp = cp.root.clone();
                        cpp.path = pc.path_name.clone();
                        if let Some(state) = cpp.td_layer.as_mut() {
                            state.spawn_lineage = pc.creeps[cur_path_idx].spawn_lineage;
                        }
                        // Path 初始化時已把 CheckPoint f32 座標轉成 fixed，spawn hot path
                        // 直接重用，避免每隻 creep 重複橋接。
                        let cp0 = CreepData {
                            pos,
                            creep: cpp.clone(),
                            cdata: cp.property.clone(),
                            faction_name: cp.faction_name.clone(),
                            turn_speed_deg: Fixed64::from_raw(
                                (cp.turn_speed_deg * omoba_sim::fixed::SCALE as f32) as i64,
                            ),
                            collision_radius: Fixed64::from_raw(
                                (cp.collision_radius * omoba_sim::fixed::SCALE as f32) as i64,
                            ),
                        };
                        tw.outcomes.push(Outcome::Creep { cd: cp0 });
                        spawned_this_tick = true;
                    }
                }
                cw.path[i] += 1;
                drained += 1;
                if drained >= pc.creeps.len() {
                    log::error!(
                        "creep_wave occurrence guard reached: path={} tick={} dt_raw={} count={}",
                        pc.path_name,
                        tr.tick.0,
                        tr.dt.0.raw(),
                        drained,
                    );
                    break;
                }
            }
            if cw.path[i] < pc.creeps.len() {
                is_end = false;
            }
        }

        if is_end {
            // 所有本波小兵都已派出；TD 模式還要等地圖上沒有活著的 creep 才算結束。
            let any_alive = (&tr.creeps).join().next().is_some();
            if is_td && (any_alive || spawned_this_tick) {
                return;
            }
            if is_td {
                // TD 模式：推進到下一波、進入 idle，等玩家按 StartRound
                cw.wave += 1;
                cw.path.clear();
                cw.is_running = false;
                let finished = cw.wave; // 已完成的波數（從 1 開始給前端看）
                let total = tw.creep_waves.len();
                let amount = tr.economy_rules.round_bonus(finished);
                let player_ids: Vec<u32> = tw.player_economy.balances().keys().copied().collect();
                for player_id in player_ids {
                    if let Err(error) = tw.economy_ledger.apply(
                        &mut tw.player_economy,
                        tr.tick.0,
                        Some(player_id),
                        TdEconomyCategory::RoundBonus,
                        amount,
                        format!("round:{finished}"),
                    ) {
                        log::error!(
                            "TD round bonus rejected round={} player={} amount={}: {}",
                            finished,
                            player_id,
                            amount,
                            error
                        );
                    }
                }
                let payload = json!({
                    "round": finished,
                    "total": total,
                    "is_running": false,
                });
                tw.runtime_events.push(
                    RuntimeEvent::new("td/all/res", "game", "round", payload)
                        .with_broadcast(RuntimeBroadcast::All),
                );
                log::info!(
                    "✅ TD 第 {} 波結束，等待 StartRound（已完成 {}/{}）",
                    finished,
                    finished,
                    total
                );
                // 所有波都打完 → 勝利
                if finished >= total {
                    let end_payload =
                        json!({ "result": "victory", "reason": "all_rounds_cleared" });
                    tw.runtime_events.push(
                        RuntimeEvent::new("td/all/res", "game", "end", end_payload)
                            .with_broadcast(RuntimeBroadcast::All),
                    );
                    log::info!("🏆 TD 勝利：全部 {} 波已清空", total);
                }
            } else {
                cw.wave += 1;
                cw.path.clear();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt};

    fn spawn_order_for_profile(
        profile: crate::runtime::SimulationTickProfile,
    ) -> Vec<(String, u64)> {
        let mut world = World::new();
        world.register::<Creep>();
        world.insert(Time(0.0));
        world.insert(DeltaTime(Fixed64::ZERO));
        world.insert(BTreeMap::from([(
            "red".to_string(),
            CreepEmiter {
                root: Creep {
                    name: "red".to_string(),
                    label: None,
                    path: String::new(),
                    pidx: 0,
                    path_remaining_distance: Fixed64::from_i32(1_000),
                    block_tower: None,
                    status: CreepStatus::Walk,
                    td_layer: Some(TdLayerState {
                        base_archetype: "red".to_string(),
                        current_layer: "red".to_string(),
                        properties: 0,
                        regrow_ceiling: "red".to_string(),
                        regrow_elapsed: Fixed64::ZERO,
                        remaining_leak_value: 1,
                        spawn_lineage: 0,
                    }),
                },
                property: CProperty {
                    hp: Fixed64::ONE,
                    mhp: Fixed64::ONE,
                    msd: Fixed64::ONE,
                    def_physic: Fixed64::ZERO,
                    def_magic: Fixed64::ZERO,
                },
                faction_name: "enemy".to_string(),
                turn_speed_deg: 90.0,
                collision_radius: 20.0,
            },
        )]));
        world.insert(BTreeMap::from([(
            "td_main".to_string(),
            Path {
                check_points: Vec::new(),
                check_points_sim: vec![omoba_sim::Vec2::ZERO],
            },
        )]));
        world.insert(BTreeMap::<String, CheckPoint>::new());
        world.insert(GameMode::TowerDefense);
        world.insert(Tick(0));
        world.insert(TdEconomyRules::default());
        world.insert(TdEconomyLedger::default());
        world.insert(Vec::<Outcome>::new());
        world.insert(RuntimeEvents::default());
        world.insert(PendingDebugCreepSpawnQueue::default());
        world.insert(SysMetrics::default());
        world.insert(TickProfile::default());
        let mut economy = PlayerEconomy::default();
        economy.initialize(1, 650);
        world.insert(economy);
        world.insert(CurrentCreepWave {
            wave: 0,
            path: Vec::new(),
            is_running: true,
            wave_start_time: 0.0,
        });
        world.insert(vec![CreepWave {
            time: 0.0,
            path_creeps: vec![PathCreeps {
                creeps: [0.01f32, 0.02, 0.03, 0.07, 0.071]
                    .into_iter()
                    .enumerate()
                    .map(|(index, time)| CreepEmit {
                        time,
                        name: "red".to_string(),
                        spawn_lineage: index as u64 + 1,
                    })
                    .collect(),
                path_name: "td_main".to_string(),
            }],
        }]);

        let mut order = Vec::new();
        for tick in 1..=60u64 {
            world.write_resource::<Tick>().0 = tick;
            world.write_resource::<Time>().0 += profile.seconds_per_tick();
            world.write_resource::<DeltaTime>().0 =
                Fixed64::from_raw(profile.fixed_raw_for_tick(tick));
            crate::comp::run_now::<Sys>(&world);
            let batch = std::mem::take(&mut *world.write_resource::<Vec<Outcome>>());
            for outcome in batch {
                if let Outcome::Creep { cd } = outcome {
                    order.push((
                        cd.creep.name,
                        cd.creep.td_layer.map_or(0, |v| v.spawn_lineage),
                    ));
                }
            }
            if order.len() == 5 {
                break;
            }
        }
        order
    }

    #[test]
    fn spawn_drain_preserves_authored_order_at_fifteen_and_one_twenty_hz() {
        let coarse = spawn_order_for_profile(crate::runtime::SimulationTickProfile::Coarse15Hz);
        let production =
            spawn_order_for_profile(crate::runtime::SimulationTickProfile::Production120Hz);
        let expected = (1..=5u64)
            .map(|lineage| ("red".to_string(), lineage))
            .collect::<Vec<_>>();
        assert_eq!(coarse, expected);
        assert_eq!(production, expected);
    }

    #[test]
    fn td_wave_clear_awards_separate_rule_bonus_without_heroes() {
        let mut world = World::new();
        world.register::<Creep>();

        world.insert(Time(1.0));
        world.insert(DeltaTime(omoba_sim::Fixed64::ZERO));
        world.insert(BTreeMap::<String, CreepEmiter>::new());
        world.insert(BTreeMap::<String, Path>::new());
        world.insert(BTreeMap::<String, CheckPoint>::new());
        world.insert(GameMode::TowerDefense);
        world.insert(Tick(9));
        world.insert(TdEconomyRules::default());
        world.insert(TdEconomyLedger::default());
        world.insert(Vec::<Outcome>::new());
        world.insert(RuntimeEvents::default());
        world.insert(crate::comp::PendingDebugCreepSpawnQueue::default());
        world.insert(crate::comp::SysMetrics::default());
        world.insert(crate::comp::TickProfile::default());
        let mut economy = PlayerEconomy::default();
        economy.initialize(1, 650);
        economy.initialize(2, 650);
        world.insert(economy);
        world.insert(CurrentCreepWave {
            wave: 0,
            path: Vec::new(),
            is_running: true,
            wave_start_time: 0.0,
        });
        world.insert(vec![CreepWave {
            time: 0.0,
            path_creeps: vec![PathCreeps {
                creeps: Vec::new(),
                path_name: "td_main".to_string(),
            }],
        }]);

        crate::comp::run_now::<Sys>(&world);
        world.maintain();

        let economy = world.read_resource::<PlayerEconomy>();
        assert_eq!(economy.balance(1), Some(751));
        assert_eq!(economy.balance(2), Some(751));
        assert!(world.read_resource::<Vec<Outcome>>().is_empty());
    }

    #[test]
    fn td_wave_does_not_finish_on_the_tick_its_last_creep_is_queued() {
        let mut world = World::new();
        world.register::<Creep>();
        world.insert(Time(1.0));
        world.insert(DeltaTime(Fixed64::from_raw(68)));
        world.insert(BTreeMap::from([(
            "red".to_string(),
            CreepEmiter {
                root: Creep {
                    name: "red".to_string(),
                    label: None,
                    path: String::new(),
                    pidx: 0,
                    path_remaining_distance: Fixed64::from_i32(1_000_000),
                    block_tower: None,
                    status: CreepStatus::Walk,
                    td_layer: None,
                },
                property: CProperty {
                    hp: Fixed64::ONE,
                    mhp: Fixed64::ONE,
                    msd: Fixed64::ONE,
                    def_physic: Fixed64::ZERO,
                    def_magic: Fixed64::ZERO,
                },
                faction_name: String::new(),
                turn_speed_deg: 90.0,
                collision_radius: 20.0,
            },
        )]));
        world.insert(BTreeMap::from([(
            "td_main".to_string(),
            Path {
                check_points: Vec::new(),
                check_points_sim: vec![omoba_sim::Vec2::ZERO],
            },
        )]));
        world.insert(BTreeMap::<String, CheckPoint>::new());
        world.insert(GameMode::TowerDefense);
        world.insert(Tick(1));
        world.insert(TdEconomyRules::default());
        world.insert(TdEconomyLedger::default());
        world.insert(Vec::<Outcome>::new());
        world.insert(RuntimeEvents::default());
        world.insert(PendingDebugCreepSpawnQueue::default());
        world.insert(SysMetrics::default());
        world.insert(TickProfile::default());
        let mut economy = PlayerEconomy::default();
        economy.initialize(1, 650);
        world.insert(economy);
        world.insert(CurrentCreepWave {
            wave: 0,
            path: Vec::new(),
            is_running: true,
            wave_start_time: 0.0,
        });
        world.insert(vec![CreepWave {
            time: 0.0,
            path_creeps: vec![PathCreeps {
                creeps: vec![CreepEmit {
                    time: 0.0,
                    name: "red".to_string(),
                    spawn_lineage: 1,
                }],
                path_name: "td_main".to_string(),
            }],
        }]);

        crate::comp::run_now::<Sys>(&world);

        let current = world.read_resource::<CurrentCreepWave>();
        assert_eq!(current.wave, 0);
        assert!(current.is_running);
        assert_eq!(world.read_resource::<Vec<Outcome>>().len(), 1);
        assert_eq!(world.read_resource::<PlayerEconomy>().balance(1), Some(650));
    }
}
