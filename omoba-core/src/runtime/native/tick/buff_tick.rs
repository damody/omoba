//! Buff tick：每 tick 呼叫 `BuffStore::tick(dt)` 倒數、移除過期項。
//!
//! 取代舊 `slow_buff_tick`；所有 buff 統一走 `ability_runtime::BuffStore`。
//! 過期 buff 若 payload 含 `move_speed_bonus` 且 target 還活著且是 Creep →
//! 廣播 `creep/S { id, move_speed }` 讓 client 重算 lerp（buff_id 不再限定
//! "slow"，但 slow buff 採單一 instance 設計：buff_id = "slow"，由 payload
//! 內的 `slow_factor` 欄位驅動「強蓋弱」比較，多次命中只 refresh duration）。
//!
//! **DoT (Task 15)**：payload 含 `dot_damage` 的 buff 每秒對 target 扣 HP。
//! 按有效生命期積分，保留次量子餘數；透過正常 Damage outcome 結算。

use omb_script_abi::stat_keys::StatKey;
use specs::{shred, Read, ReadStorage, SystemData, Write,Entities};

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
    entities: Entities<'a>,
    factions: ReadStorage<'a,Faction>,
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
        // Zone lifetimes belong to this exact source entity generation.
        let cancelled:Vec<_>=data.buffs.entities_with_key(super::super::ability_runtime::damage_zone::KEY)
            .filter(|source|!data.entities.is_alive(*source) || data.factions.get(*source).is_none()
                || data.cpropertys.get(*source).is_none_or(|p|p.hp<=omoba_sim::Fixed64::ZERO)).collect();
        for source in cancelled {data.buffs.cancel_damage_zones(source);}
        for pulse in data.buffs.drain_damage_zones(dt) {
            // Do not capture victims or publish a cue ahead of earlier pulse
            // lethals. The serial processor resolves each host-only envelope.
            data.outcomes.push(match pulse.lease {
                Some(lease)=>Outcome::IndependentDamageZonePulse {source:pulse.source,key:pulse.key,lease},
                None=>Outcome::DamageZonePulse {source:pulse.source,payload:pulse.spec.payload()},
            });
        }
        // Integrate the active interval before expiry. Fractional raw damage belongs
        // to the buff entry, so changing simulation frequency cannot lose damage.
        for hit in data.buffs.drain_periodic_damage(dt) {
            if data.cpropertys.get(hit.target).is_none_or(|p|p.hp<=omoba_sim::Fixed64::ZERO) {continue;}
            let (phys,magi,real)=match hit.spec.kind {
                omb_script_abi::types::DamageKind::Physical=>(hit.amount,omoba_sim::Fixed64::ZERO,omoba_sim::Fixed64::ZERO),
                omb_script_abi::types::DamageKind::Magical=>(omoba_sim::Fixed64::ZERO,hit.amount,omoba_sim::Fixed64::ZERO),
                omb_script_abi::types::DamageKind::Pure=>(omoba_sim::Fixed64::ZERO,omoba_sim::Fixed64::ZERO,hit.amount),
            };
            data.outcomes.push(Outcome::RawDamage {pos:data.positions.get(hit.target).map(|p|p.0).unwrap_or_default(),
                phys,magi,real,source:hit.spec.source,target:hit.target,damage_profile:hit.spec.profile,origin:None});
        }
        let expired = data.buffs.tick(dt);

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
    use specs::{Builder, World, WorldExt,Join};

    fn dot_total_for_profile(profile: SimulationTickProfile) -> (i64, Vec<i64>) {
        let mut world = World::new();
        world.register::<Creep>();
        world.register::<CProperty>();
        world.register::<Pos>();
        world.register::<IsBuilding>();
        world.register::<Faction>();
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
                    Outcome::RawDamage {
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

    fn zone_world()->World {
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
        world.insert(crate::runtime::TowerTemplateRegistry::default());world
    }
    fn zone_body(world:&mut World,x:i32,team:i32)->specs::Entity {
        world.create_entity().with(Pos(omoba_sim::Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO)))
            .with(Faction::new(if team==1 {FactionType::Player}else{FactionType::Enemy},team))
            .with(CProperty {hp:Fixed64::from_i32(1000),mhp:Fixed64::from_i32(1000),msd:Fixed64::ZERO,
                def_physic:Fixed64::ZERO,def_magic:Fixed64::from_i32(25)}).build()
    }
    fn zone_spec()->crate::runtime::ability_runtime::damage_zone::DamageZoneSpec {
        crate::runtime::ability_runtime::damage_zone::DamageZoneSpec {independent:false,center:omoba_sim::Vec2::new(Fixed64::from_i32(500),Fixed64::ZERO),
            radius:Fixed64::from_i32(100),amount:Fixed64::from_i32(40),interval:Fixed64::from_raw(512),
            kind:omb_script_abi::types::DamageKind::Magical,profile:omb_script_abi::types::DamageProfile::FIRE.bits()}
    }
    fn zone_step(world:&mut World,dt:Fixed64)->Vec<specs::Entity> {
        world.write_resource::<DeltaTime>().0=dt;run_now::<Sys>(world);
        let before:Vec<_>=(&world.entities(),&world.read_storage::<CProperty>()).join().map(|(e,p)|(e,p.hp)).collect();
        crate::runtime::process_outcomes(world,&mut crate::runtime::RuntimeEventVecSink::default()).unwrap();
        let props=world.read_storage::<CProperty>();let mut targets:Vec<_>=before.into_iter()
            .filter_map(|(e,hp)|props.get(e).filter(|p|p.hp<hp).map(|_|e)).collect();
        targets.sort_by_key(|e|(e.id(),e.gen().id()));targets
    }
    #[test]
    fn ground_zone_tick_preserves_chronological_pulse_outcome_order() {
        let mut world=zone_world();let low=zone_body(&mut world,0,1);let high=zone_body(&mut world,0,1);
        let target=zone_body(&mut world,500,2);let mut late=zone_spec();late.interval=Fixed64::from_raw(256);
        {let mut buffs=world.write_resource::<BuffStore>();
            buffs.add(low,"late",Fixed64::from_i32(3),late.payload());
            assert!(buffs.drain_damage_zones(Fixed64::from_raw(252)).is_empty());buffs.tick(Fixed64::from_raw(252));
            buffs.add(high,"early",Fixed64::from_i32(3),zone_spec().payload());
            assert_eq!(buffs.drain_damage_zones(Fixed64::from_raw(500)).len(),2);buffs.tick(Fixed64::from_raw(500));}
        world.write_resource::<DeltaTime>().0=Fixed64::from_raw(20);run_now::<Sys>(&world);
        let sources:Vec<_>=world.read_resource::<Vec<Outcome>>().iter().filter_map(|o|match o {
            Outcome::DamageZonePulse {source,..}=>Some(*source),_=>None}).collect();
        assert_eq!(sources,vec![high,low],"higher ID is due at12raw, lower ID at16raw");
        crate::runtime::process_outcomes(&mut world,&mut crate::runtime::RuntimeEventVecSink::default()).unwrap();
        assert_eq!(world.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(940));
        assert!(zone_step(&mut world,Fixed64::ZERO).is_empty());
    }
    #[test]
    fn ground_zone_samples_exact_live_circle_each_pulse_and_settles_defenses_shields_once() {
        let mut world=zone_world();let source=zone_body(&mut world,0,1);
        let inside=zone_body(&mut world,500,2);let tangent=zone_body(&mut world,600,2);
        let outside=zone_body(&mut world,600,2);world.write_storage::<Pos>().get_mut(outside).unwrap().0.x+=Fixed64::from_raw(1);
        let ally=zone_body(&mut world,500,1);let dead=zone_body(&mut world,500,2);world.write_storage::<CProperty>().get_mut(dead).unwrap().hp=Fixed64::ZERO;
        let building=zone_body(&mut world,500,2);world.write_storage::<IsBuilding>().insert(building,IsBuilding).unwrap();
        world.write_resource::<BuffStore>().grant_damage_shield(inside,Fixed64::from_i32(10),Fixed64::from_i32(5));
        world.write_resource::<BuffStore>().add(source,"zone",Fixed64::from_i32(3),zone_spec().payload());
        assert!(zone_step(&mut world,Fixed64::from_raw(256)).is_empty());
        assert_eq!(zone_step(&mut world,Fixed64::from_raw(256)),vec![inside,tangent]);
        assert_eq!(world.read_storage::<CProperty>().get(inside).unwrap().hp,Fixed64::from_i32(980));
        assert_eq!(world.read_resource::<BuffStore>().shield_remaining(inside),Fixed64::ZERO);
        world.write_storage::<Pos>().get_mut(inside).unwrap().0.x=Fixed64::ZERO;
        world.write_storage::<Pos>().get_mut(outside).unwrap().0.x=Fixed64::from_i32(500);
        assert_eq!(zone_step(&mut world,Fixed64::from_raw(512)),vec![tangent,outside]);
        assert_eq!(world.read_storage::<CProperty>().get(tangent).unwrap().hp,Fixed64::from_i32(940));
        assert_eq!(world.read_storage::<CProperty>().get(outside).unwrap().hp,Fixed64::from_i32(970));
        assert_eq!(world.read_storage::<CProperty>().get(inside).unwrap().hp,Fixed64::from_i32(980));
        for target in [source,ally,building] {assert_eq!(world.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(1000));}
    }
    #[test]
    fn ground_zone_death_retirement_and_explicit_removal_never_resume_on_reused_source_id() {
        for retired in [false,true] {
            let mut world=zone_world();let source=zone_body(&mut world,0,1);let target=zone_body(&mut world,500,2);
            world.write_resource::<BuffStore>().add(source,"zone",Fixed64::from_i32(3),zone_spec().payload());
            assert!(zone_step(&mut world,Fixed64::ZERO).is_empty());
            if retired {world.delete_entity(source).unwrap();world.maintain();let _replacement=zone_body(&mut world,0,1);}
            else {world.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::ZERO;}
            assert!(zone_step(&mut world,Fixed64::from_raw(512)).is_empty());assert!(!world.read_resource::<BuffStore>().has(source,"zone"));
            assert_eq!(world.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(1000));
        }
        let mut world=zone_world();let source=zone_body(&mut world,0,1);let _target=zone_body(&mut world,500,2);
        world.write_resource::<BuffStore>().add(source,"zone",Fixed64::from_i32(3),zone_spec().payload());
        world.write_resource::<BuffStore>().remove(source,"zone");assert!(zone_step(&mut world,Fixed64::from_i32(3)).is_empty());
    }
    #[test]
    fn ground_zone_ingress_rejects_invalid_payload_or_lifetime_without_replacing_active_zone() {
        let mut world=zone_world();let source=zone_body(&mut world,0,1);let original=zone_spec().payload();
        world.write_resource::<BuffStore>().add(source,"zone",Fixed64::ONE,original.clone());
        for case in 0..4 {
            let mut payload=original.clone();let duration=match case {0=>Fixed64::from_raw(255),1=>Fixed64::from_i32(31),_=>Fixed64::ONE};
            if case==2 {payload["damage_zone"]["interval"]=json!(0);}if case==3 {payload["damage_taken_bonus"]=json!(-1024);}
            world.write_resource::<Vec<Outcome>>().push(Outcome::AddBuff{target:source,buff_id:"zone".into(),duration,payload});
            crate::runtime::process_outcomes(&mut world,&mut crate::runtime::RuntimeEventVecSink::default()).unwrap();
            assert_eq!(world.read_resource::<BuffStore>().get(source,"zone").unwrap().payload,original);
            assert_eq!(world.read_resource::<BuffStore>().get(source,"zone").unwrap().remaining,Fixed64::ONE);
        }
    }
}
