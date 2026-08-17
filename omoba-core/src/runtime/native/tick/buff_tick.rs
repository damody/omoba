//! Buff tick：每 tick 呼叫 `BuffStore::tick(dt)` 倒數、移除過期項。
//!
//! 取代舊 `slow_buff_tick`；所有 buff 統一走 `ability_runtime::BuffStore`。
//! 過期 buff 若 payload 含 `move_speed_bonus` 且 target 還活著且是 Creep →
//! 廣播 `creep/S { id, move_speed }` 讓 client 重算 lerp（buff_id 不再限定
//! "slow"，但 slow buff 採單一 instance 設計：buff_id = "slow"，由 payload
//! 內的 `slow_factor` 欄位驅動「強蓋弱」比較，多次命中只 refresh duration）。
//!
//! **DoT (Task 15)**：payload 含 `dot_damage` 的 buff 每秒對 target 扣 HP。
//! 以 1 秒累計槽 (`dot_accum: f32`) 控制頻率，累積到 1s 時觸發一次整批 dot。

use omb_script_abi::stat_keys::StatKey;
use specs::world::Generation;
use specs::{shred, Entity, Read, ReadStorage, SystemData, Write};

use crate::comp::*;
use crate::scripting::{ScriptEvent, ScriptEventQueue};
use omoba_core::runtime::ability_runtime::{BuffStore, UnitStats};

/// 位移類 payload key — 任一存在於過期 buff 的 payload 就要重算 creep 移速並廣播 `creep/S`。
/// 對應 Dota MOVESPEED_BONUS_* / MOVESPEED_ABSOLUTE / MIN / MAX / LIMIT。
const MOVESPEED_PAYLOAD_KEYS: &[StatKey] = &[
    StatKey::MoveSpeedBonus,
    StatKey::MoveSpeedBonusEquipment,
    StatKey::MoveSpeedBonusBuff,
    StatKey::MoveSpeedBaseOverride,
    StatKey::MoveSpeedBonusPercentage,
    StatKey::MoveSpeedBonusPercentageUnique,
    StatKey::MoveSpeedBonusPercentageUnique2,
    StatKey::MoveSpeedAbsolute,
    StatKey::MoveSpeedAbsoluteMin,
    StatKey::MoveSpeedLimit,
    StatKey::MoveSpeedMax,
];

#[derive(SystemData)]
pub struct BuffTickData<'a> {
    dt: Read<'a, DeltaTime>,
    buffs: Write<'a, BuffStore>,
    creeps: ReadStorage<'a, Creep>,
    cpropertys: ReadStorage<'a, CProperty>,
    positions: ReadStorage<'a, Pos>,
    is_buildings: ReadStorage<'a, IsBuilding>,
    script_events: Write<'a, ScriptEventQueue>,
    outcomes: Write<'a, Vec<Outcome>>,
}

#[derive(Default)]
pub struct Sys;

impl<'a> System<'a> for Sys {
    type SystemData = BuffTickData<'a>;

    const NAME: &'static str = "buff";

    fn run(_job: &mut Job<Self>, mut data: Self::SystemData) {
        // 階段 1c.3：BuffStore::tick 現在直接採用 Fix64。
        let dt = data.dt.0;
        let expired = data.buffs.tick(dt);

        // DoT (Task 15)：連續扣血，每 tick dot_damage * dt，達 dot/s 持續傷害
        // 累積到單次廣播避免每 tick 刷 creep/H。
        // 用 entities_by_key 反向索引取候選，避免對全表 entity 都呼 sum_add。
        let dot_targets: Vec<specs::Entity> = data
            .buffs
            .entities_with_key(StatKey::DotDamage.as_str())
            .collect();
        for entity in dot_targets {
            for (buff_id, entry) in data.buffs.iter_for(entity) {
                let Some(dot_raw) = entry.payload.get("dot_damage").and_then(|v| v.as_i64()) else {
                    continue;
                };
                let profile = entry
                    .payload
                    .get("damage_profile")
                    .and_then(|value| value.as_u64())
                    .map(|value| value as u32);
                let source_id = entry
                    .payload
                    .get("source_entity_id")
                    .and_then(|value| value.as_u64())
                    .map(|value| value as u32);
                let source_gen = entry
                    .payload
                    .get("source_entity_gen")
                    .and_then(|value| value.as_u64())
                    .map(|value| value as i32);
                let (Some(profile), Some(source_id), Some(source_gen)) =
                    (profile, source_id, source_gen)
                else {
                    log::error!(
                        "reject DoT buff={} target={} without explicit profile/source",
                        buff_id,
                        entity.id()
                    );
                    continue;
                };
                if omb_script_abi::types::DamageProfile::from_bits(profile).is_none() {
                    log::error!(
                        "reject DoT buff={} target={} source={} unknown mask={:#x}",
                        buff_id,
                        entity.id(),
                        source_id,
                        profile
                    );
                    continue;
                }
                let damage = omoba_sim::Fixed64::from_raw(dot_raw) * dt;
                if damage <= omoba_sim::Fixed64::ZERO {
                    continue;
                }
                data.outcomes.push(Outcome::Damage {
                    pos: data
                        .positions
                        .get(entity)
                        .map(|position| position.0)
                        .unwrap_or_default(),
                    phys: omoba_sim::Fixed64::ZERO,
                    magi: damage,
                    real: omoba_sim::Fixed64::ZERO,
                    source: Entity::new(source_id, Generation::new(source_gen)),
                    target: entity,
                    damage_profile: profile,
                    predeclared: false,
                });
            }
        }

        for (entity, _buff_id, payload) in expired {
            // 每條過期 buff push ModifierRemoved 事件，讓腳本的 on_modifier_removed
            // 能 hook 到（例：某 stacking debuff 過期時補一個 refresh buff）。
            data.script_events.push(ScriptEvent::ModifierRemoved {
                e: entity,
                modifier_id: _buff_id.clone(),
            });

            // 若 payload 任一 key 屬於位移類 → 對 creep 重算 effective 並廣播 creep/S。
            // 用 UnitStats 套完整 Dota 公式（而非舊的 clamp 0.01-1.0）。
            let touches_movespeed = MOVESPEED_PAYLOAD_KEYS
                .iter()
                .any(|k| payload.get(k.as_str()).is_some());
            if touches_movespeed {
                let is_creep = data.creeps.get(entity).is_some();
                if is_creep {
                    if let Some(cp) = data.cpropertys.get(entity) {
                        let stats = UnitStats::from_refs(
                            &*data.buffs,
                            data.is_buildings.get(entity).is_some(),
                        );
                        let _ = stats.final_move_speed(cp.msd, entity);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comp::{run_now, SysMetrics, TickProfile};
    use crate::runtime::{BuffStore, SimulationTickProfile};
    use omoba_sim::Fixed64;
    use serde_json::json;
    use specs::{Builder, World, WorldExt};

    fn dot_total_for_profile(profile: SimulationTickProfile) -> (i64, Vec<i64>) {
        let mut world = World::new();
        world.register::<Creep>();
        world.register::<CProperty>();
        world.register::<Pos>();
        world.register::<IsBuilding>();
        world.insert(DeltaTime(Fixed64::ZERO));
        world.insert(BuffStore::default());
        world.insert(ScriptEventQueue::default());
        world.insert(Vec::<Outcome>::new());
        world.insert(SysMetrics::default());
        world.insert(TickProfile::default());
        let source = world.create_entity().build();
        let target = world
            .create_entity()
            .with(Creep {
                name: "dot-target".to_string(),
                label: None,
                path: String::new(),
                pidx: 0,
                path_remaining_distance: Fixed64::ZERO,
                block_tower: None,
                status: CreepStatus::Walk,
                td_layer: None,
            })
            .with(CProperty {
                hp: Fixed64::from_i32(100),
                mhp: Fixed64::from_i32(100),
                msd: Fixed64::ZERO,
                def_physic: Fixed64::ZERO,
                def_magic: Fixed64::ZERO,
            })
            .with(Pos(omoba_sim::Vec2::ZERO))
            .build();
        world.write_resource::<BuffStore>().add(
            target,
            "rate-dot",
            Fixed64::from_i32(2),
            json!({
                "dot_damage": Fixed64::from_i32(12).raw(),
                "damage_profile": omb_script_abi::types::DamageProfile::FIRE.bits(),
                "source_entity_id": source.id(),
                "source_entity_gen": source.gen().id(),
            }),
        );

        let mut per_tick = Vec::new();
        for tick in 1..=u64::from(profile.ticks_per_game_second()) {
            world.write_resource::<DeltaTime>().0 =
                Fixed64::from_raw(profile.fixed_raw_for_tick(tick));
            run_now::<Sys>(&world);
            let batch = std::mem::take(&mut *world.write_resource::<Vec<Outcome>>());
            let tick_damage = batch
                .into_iter()
                .filter_map(|outcome| match outcome {
                    Outcome::Damage {
                        magi, target: hit, ..
                    } if hit == target => Some(magi.raw()),
                    _ => None,
                })
                .sum::<i64>();
            per_tick.push(tick_damage);
        }
        (per_tick.iter().sum(), per_tick)
    }

    #[test]
    fn dot_integrates_same_damage_at_fifteen_and_one_twenty_hz() {
        let (coarse_total, coarse_order) = dot_total_for_profile(SimulationTickProfile::Coarse15Hz);
        let (production_total, production_order) =
            dot_total_for_profile(SimulationTickProfile::Production120Hz);
        assert_eq!(coarse_total, Fixed64::from_i32(12).raw());
        assert_eq!(production_total, coarse_total);
        assert!(coarse_order.iter().all(|damage| *damage > 0));
        assert!(production_order.iter().all(|damage| *damage > 0));
    }
}
