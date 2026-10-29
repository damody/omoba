//! The generated-code host hooks end to end through the simulation driver: a cast schedules
//! (scheduled_effect) and the schedule comes back through `on_scheduled` with its captures; a
//! marked status (status_event) reports its holder's damage and its end through
//! `on_status_event`; a state variable (unit_var) written by a cast is read back by the next.
use super::*;
use crate::comp::Hero;
use crate::runtime::BuffStore;
use abi_stable::std_types::{RResult, RStr, RString, RVec, ROk};
use omb_script_abi::types::{DamageEdit, EntityHandle, ScheduleArg, Scheduled, StatusEvent, Target};
use omb_script_abi::world::GameWorldDyn;
use specs::{Entity, WorldExt};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq)]
enum Call {
    Scheduled { owner: u32, step: u16, level: u8, args: Vec<ScheduleArg> },
    Status { holder: u32, status: String, kind: u8, other: u32, amount: Fixed64 },
    ReadVar(Option<String>),
    DamagePre { holder: u32, kind: u8, amount: Fixed64 },
}

#[derive(Clone)]
struct Probe { calls: Arc<Mutex<Vec<Call>>>, id: String }

impl omb_script_abi::ability::AbilityScript for Probe {
    fn ability_id(&self) -> RStr<'_> { RStr::from_str(&self.id) }
    /// A cast: reads the state variable, writes it, schedules step 7 with a number and the caster,
    /// and marks itself with a status listening to its end and its damage.
    fn execute(&self, caster: EntityHandle, _: Target, level: u8, _: RStr<'_>, world: &mut GameWorldDyn<'_>) -> RResult<(), RString> {
        let var = crate::runtime::native::unit_var::Spec { key: "probe:casts".into(), value: crate::runtime::native::unit_var::VarValue::Num(Fixed64::from_i32(1)), hero: false };
        let read = world.get_buff_payload(caster, RStr::from_str(&var.buff_id())).into_option().map(|s| s.to_string());
        self.calls.lock().unwrap().push(Call::ReadVar(read));
        world.add_stat_buff(caster, RStr::from_str(&var.buff_id()), crate::runtime::native::ability_runtime::buff_store::PERMANENT_BUFF_DURATION,
            RStr::from_str(&var.payload().to_string()));
        let args = vec![
            ScheduleArg { kind: crate::runtime::native::scheduled_effect::ARG_NUM, num: Fixed64::from_i32(9), unit: EntityHandle::INVALID, point: omoba_sim::Vec2::ZERO },
            ScheduleArg { kind: crate::runtime::native::scheduled_effect::ARG_UNIT, num: Fixed64::ZERO, unit: caster, point: omoba_sim::Vec2::ZERO },
        ];
        let spec = crate::runtime::native::scheduled_effect::Spec { ability: self.id.clone(), level, step: 7, instance: 1, args };
        world.add_stat_buff(caster, RStr::from_str(&spec.buff_id()), crate::runtime::native::scheduled_effect::Spec::lifetime(Fixed64::from_raw(512)),
            RStr::from_str(&spec.payload().to_string()));
        let marker = crate::runtime::native::status_event::Spec { ability: self.id.clone(), level, status: "probe_mark".into(), source: caster,
            mask: crate::runtime::native::status_event::ON_END | crate::runtime::native::status_event::ON_TAKE_DAMAGE
                | crate::runtime::native::status_event::ON_TAKE_DAMAGE_PRE };
        world.add_stat_buff(caster, RStr::from_str("probe_mark"), Fixed64::from_i32(1),
            RStr::from_str(&serde_json::json!({ (crate::runtime::native::status_event::KEY): marker.row() }).to_string()));
        ROk(())
    }
    fn on_scheduled(&self, owner: EntityHandle, at: Scheduled, args: RVec<ScheduleArg>, _: &mut GameWorldDyn<'_>) {
        self.calls.lock().unwrap().push(Call::Scheduled { owner: owner.id, step: at.step, level: at.level, args: args.to_vec() });
    }
    /// Halves what the marked holder takes.
    fn on_damage_pre(&self, holder: EntityHandle, _: RStr<'_>, event: StatusEvent, _: &mut GameWorldDyn<'_>) -> DamageEdit {
        self.calls.lock().unwrap().push(Call::DamagePre { holder: holder.id, kind: event.kind, amount: event.amount });
        DamageEdit { mode: DamageEdit::MULTIPLY, value: Fixed64::from_raw(512) }
    }
    fn on_status_event(&self, holder: EntityHandle, status: RStr<'_>, event: StatusEvent, _: &mut GameWorldDyn<'_>) {
        self.calls.lock().unwrap().push(Call::Status { holder: holder.id, status: status.to_string(), kind: event.kind,
            other: event.other.id, amount: event.amount });
    }
}

fn driven(calls: Arc<Mutex<Vec<Call>>>) -> (specs::World, crate::runtime::SimulationDriver, Entity, String) {
    use crate::runtime::{SingleLaneConfig, StateInitializer};
    let pool = StateInitializer::create_thread_pool();
    let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
    crate::runtime::setup_single_lane_match(&mut world, SingleLaneConfig {
        warmup: Fixed64::ZERO, wave_interval: Fixed64::from_i32(1_000_000), ..SingleLaneConfig::default()
    }).unwrap();
    let hero = world.read_resource::<crate::runtime::MobaMatch>().heroes.iter().find(|slot| slot.player_id == 1)
        .and_then(|slot| slot.entity).unwrap();
    let id = world.read_storage::<Hero>().get(hero).unwrap().abilities[0].clone();
    let level = serde_json::json!({"cooldown": 0.1, "mana_cost": 0});
    let def: crate::ability_meta::AbilityDef = serde_json::from_value(serde_json::json!({
        "id": id, "name": id, "ability_type": "active", "target_type": "none", "max_level": 4,
        "levels": {"1": level, "2": level, "3": level, "4": level},
    })).unwrap();
    world.insert(crate::runtime::TowerTemplateRegistry::default());
    let mut abilities = crate::runtime::ability_runtime::AbilityRegistry::new();
    abilities.register(def.clone());
    world.insert(abilities);
    let mut scripts = crate::runtime::ScriptRegistry::new();
    scripts.insert_ability_for_test(def, omb_script_abi::ability::AbilityScript_TO::from_value(
        Probe { calls, id: id.clone() }, abi_stable::sabi_trait::prelude::TD_Opaque));
    world.insert(scripts);
    let driver = crate::runtime::SimulationDriver::from_world(&mut world, crate::runtime::SimulationTickProfile::Production60Hz).unwrap();
    (world, driver, hero, id)
}

fn cast(world: &mut specs::World, driver: &mut crate::runtime::SimulationDriver) {
    use crate::game_proto::{player_input::Action, CastAbility, PlayerInput};
    driver.step(world, [(1, PlayerInput { action: Some(Action::CastAbility(CastAbility { ability_index: 0, ..Default::default() })) })]).unwrap();
}

#[test]
fn a_cast_schedule_comes_back_once_with_its_captures_and_a_marked_status_reports_its_end() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let (mut world, mut driver, hero, _) = driven(calls.clone());
    driver.step(&mut world, []).unwrap();
    cast(&mut world, &mut driver);
    let scheduled = |calls: &Arc<Mutex<Vec<Call>>>| calls.lock().unwrap().iter().filter(|c| matches!(c, Call::Scheduled { .. })).cloned().collect::<Vec<_>>();
    assert!(world.read_resource::<BuffStore>().iter_for(hero).any(|(id, _)| id.starts_with(crate::runtime::native::scheduled_effect::PREFIX)),
        "the cast's schedule row is on the caster");
    for _ in 0..20 { driver.step(&mut world, []).unwrap(); }
    assert!(scheduled(&calls).is_empty(), "not before 0.5 s");
    for _ in 0..30 { driver.step(&mut world, []).unwrap(); }
    let got = scheduled(&calls);
    assert_eq!(got.len(), 1, "once");
    let Call::Scheduled { owner, step, level, args } = &got[0] else { unreachable!() };
    assert_eq!((*owner, *step, *level), (hero.id(), 7, 1));
    assert_eq!((args[0].num, args[1].unit.id), (Fixed64::from_i32(9), hero.id()));
    assert!(!world.read_resource::<BuffStore>().iter_for(hero).any(|(id, _)| id.starts_with(crate::runtime::native::scheduled_effect::PREFIX)), "consumed");
    // the marked status expired after 1 s: one end event of kind EXPIRED on the hero
    for _ in 0..40 { driver.step(&mut world, []).unwrap(); }
    let ends: Vec<_> = calls.lock().unwrap().iter().filter_map(|c| match c {
        Call::Status { holder, status, kind, .. } => Some((*holder, status.clone(), *kind)), _ => None }).collect();
    assert_eq!(ends, vec![(hero.id(), "probe_mark".to_owned(), crate::runtime::native::status_event::EXPIRED)]);
}

#[test]
fn state_variables_written_by_one_cast_are_read_by_the_next() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let (mut world, mut driver, _, _) = driven(calls.clone());
    driver.step(&mut world, []).unwrap();
    cast(&mut world, &mut driver);
    for _ in 0..30 { driver.step(&mut world, []).unwrap(); }
    cast(&mut world, &mut driver);
    let reads: Vec<_> = calls.lock().unwrap().iter().filter_map(|c| match c { Call::ReadVar(r) => Some(r.clone()), _ => None }).collect();
    assert_eq!(reads.len(), 2);
    assert_eq!(reads[0], None, "nothing before the first cast");
    let second: serde_json::Value = serde_json::from_str(reads[1].as_deref().expect("written")).unwrap();
    assert_eq!(crate::runtime::native::unit_var::Spec::from_buff("unit_var:probe:casts", &second).map(|s| s.value),
        Some(crate::runtime::native::unit_var::VarValue::Num(Fixed64::from_i32(1))));
}

#[test]
fn a_marked_status_hears_its_holders_damage() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let (mut world, mut driver, hero, _) = driven(calls.clone());
    driver.step(&mut world, []).unwrap();
    cast(&mut world, &mut driver);
    driver.step(&mut world, []).unwrap();
    world.write_resource::<crate::runtime::native::scripting::ScriptEventQueue>().push(crate::runtime::native::scripting::ScriptEvent::Damage {
        attacker: None, victim: hero, amount: Fixed64::from_i32(40), kind: omb_script_abi::types::DamageKind::Magical });
    driver.step(&mut world, []).unwrap();
    let damage: Vec<_> = calls.lock().unwrap().iter().filter_map(|c| match c {
        Call::Status { kind, amount, .. } if *kind == crate::runtime::native::status_event::TAKE_DAMAGE => Some(*amount), _ => None }).collect();
    assert_eq!(damage, vec![Fixed64::from_i32(40)]);
}

#[test]
fn a_marked_status_halves_the_damage_before_it_lands() {
    use crate::runtime::native::comp::CProperty;
    let calls = Arc::new(Mutex::new(Vec::new()));
    let (mut world, mut driver, hero, _) = driven(calls.clone());
    driver.step(&mut world, []).unwrap();
    cast(&mut world, &mut driver);
    driver.step(&mut world, []).unwrap();
    let hp = |world: &specs::World| world.read_storage::<CProperty>().get(hero).unwrap().hp;
    let before = hp(&world);
    let pure = omb_script_abi::types::DamageProfile::TRUE.0;
    world.write_resource::<Vec<crate::comp::Outcome>>().push(crate::comp::Outcome::RawDamage {
        pos: omoba_sim::Vec2::ZERO, phys: Fixed64::ZERO, magi: Fixed64::ZERO, real: Fixed64::from_i32(100),
        source: hero, target: hero, damage_profile: pure, origin: None });
    driver.step(&mut world, []).unwrap();
    let lost = before - hp(&world);
    let pre: Vec<_> = calls.lock().unwrap().iter().filter_map(|c| match c {
        Call::DamagePre { holder, kind, amount } => Some((*holder, *kind, *amount)), _ => None }).collect();
    assert_eq!(pre, vec![(hero.id(), crate::runtime::native::status_event::TAKE_DAMAGE_PRE, Fixed64::from_i32(100))]);
    assert!(lost >= Fixed64::from_i32(49) && lost <= Fixed64::from_i32(51), "half lands (regen aside): lost {lost:?}");
}

#[test]
fn hero_commands_settle_cooldowns_levels_and_health_at_once() {
    use crate::runtime::native::comp::CProperty;
    use crate::runtime::native::hero_command::{Command, BUFF};
    let calls = Arc::new(Mutex::new(Vec::new()));
    let (mut world, mut driver, hero, id) = driven(calls.clone());
    driver.step(&mut world, []).unwrap();
    let send = |world: &mut specs::World, cmd: Command| world.write_resource::<Vec<crate::comp::Outcome>>().push(
        crate::comp::Outcome::AddBuff { target: hero, buff_id: BUFF.into(), duration: Fixed64::ONE, payload: cmd.payload() });
    send(&mut world, Command::CooldownStart { ability: id.clone(), seconds: Fixed64::from_i32(5) });
    driver.step(&mut world, []).unwrap();
    let cd = |world: &specs::World| world.read_storage::<Hero>().get(hero).unwrap().get_cooldown(&id);
    assert!(cd(&world) > Fixed64::from_i32(4) && cd(&world) <= Fixed64::from_i32(5), "started: {:?}", cd(&world));
    assert!(!world.read_resource::<BuffStore>().has(hero, BUFF), "never stored as a buff");
    send(&mut world, Command::CooldownReduce { ability: id.clone(), seconds: Fixed64::from_i32(2) });
    driver.step(&mut world, []).unwrap();
    assert!(cd(&world) > Fixed64::from_i32(2) && cd(&world) < Fixed64::from_i32(3), "reduced: {:?}", cd(&world));
    send(&mut world, Command::CooldownEnd { ability: id.clone() });
    driver.step(&mut world, []).unwrap();
    assert_eq!(cd(&world), Fixed64::ZERO);
    send(&mut world, Command::Level { ability: id.clone(), level: 2 });
    driver.step(&mut world, []).unwrap();
    assert_eq!(world.read_storage::<Hero>().get(hero).unwrap().get_ability_level(&id), 2);
    send(&mut world, Command::Health { value: Fixed64::from_i32(123) });
    driver.step(&mut world, []).unwrap();
    let hp = world.read_storage::<CProperty>().get(hero).unwrap().hp;
    assert!(hp >= Fixed64::from_i32(123) && hp < Fixed64::from_i32(126), "set, then a tick of regen: {hp:?}");
}
