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
    Status { holder: u32, status: String, kind: u8, other: u32, amount: Fixed64, damage_kind: u8 },
    ReadVar(Option<String>),
    DamagePre { holder: u32, kind: u8, amount: Fixed64, damage_kind: u8 },
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
        self.calls.lock().unwrap().push(Call::DamagePre { holder: holder.id, kind: event.kind, amount: event.amount, damage_kind: event.damage_kind });
        DamageEdit { mode: DamageEdit::MULTIPLY, value: Fixed64::from_raw(512) }
    }
    fn on_status_event(&self, holder: EntityHandle, status: RStr<'_>, event: StatusEvent, _: &mut GameWorldDyn<'_>) {
        self.calls.lock().unwrap().push(Call::Status { holder: holder.id, status: status.to_string(), kind: event.kind,
            other: event.other.id, amount: event.amount, damage_kind: event.damage_kind });
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
    // Real packets through handle_damage (the pre hook halves each): a plain pure hit, then a reflected one.
    let pure = omb_script_abi::types::DamageProfile::TRUE;
    for profile in [pure, pure.union(omb_script_abi::types::DamageProfile::REFLECTION)] {
        world.write_resource::<Vec<crate::comp::Outcome>>().push(crate::comp::Outcome::RawDamage {
            pos: omoba_sim::Vec2::ZERO, phys: Fixed64::ZERO, magi: Fixed64::ZERO, real: Fixed64::from_i32(80),
            source: hero, target: hero, damage_profile: profile.bits(), origin: None });
        driver.step(&mut world, []).unwrap();
    }
    driver.step(&mut world, []).unwrap();
    let damage: Vec<_> = calls.lock().unwrap().iter().filter_map(|c| match c {
        Call::Status { kind, amount, damage_kind, .. } if *kind == crate::runtime::native::status_event::TAKE_DAMAGE => Some((*amount, *damage_kind)),
        _ => None }).collect();
    assert_eq!(damage, vec![(Fixed64::from_i32(40), 2), (Fixed64::from_i32(40), 2 | StatusEvent::REFLECTION)],
        "the hp each hit took, the second marked as a reflection");
    // E475: the packet right after an instant attack, from that attacker to that target, is an
    // attack's damage; the same packet without it is not.
    let heard = calls.lock().unwrap().len();
    for attacked in [true, false] {
        let mut batch = world.write_resource::<Vec<crate::comp::Outcome>>();
        if attacked {
            batch.push(crate::comp::Outcome::InstantAttack { attacker: hero, target: hero, critical: None, missed: false, evaded: false });
        }
        batch.push(crate::comp::Outcome::Damage { pos: omoba_sim::Vec2::ZERO, phys: Fixed64::ZERO, magi: Fixed64::ZERO,
            real: Fixed64::from_i32(80), source: hero, target: hero, damage_profile: pure.bits(), predeclared: false });
        drop(batch);
        driver.step(&mut world, []).unwrap();
    }
    driver.step(&mut world, []).unwrap();
    let attacks: Vec<_> = calls.lock().unwrap()[heard..].iter().filter_map(|c| match c {
        Call::Status { kind, damage_kind, .. } if *kind == crate::runtime::native::status_event::TAKE_DAMAGE => Some(*damage_kind & StatusEvent::ATTACK != 0),
        _ => None }).collect();
    assert_eq!(attacks, vec![true, false], "only the attack's packet is marked");
    // E479: the pre-mitigation hooks see the same flag.
    let pre: Vec<_> = calls.lock().unwrap()[heard..].iter().filter_map(|c| match c {
        Call::DamagePre { damage_kind, .. } => Some(*damage_kind & StatusEvent::ATTACK != 0), _ => None }).collect();
    assert_eq!(pre, vec![true, false], "on_take_damage_pre reads is_attack too");
    // E452: a hit that kills is marked lethal; the hp it took is what was left.
    world.write_storage::<crate::runtime::native::comp::CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(30);
    world.write_resource::<Vec<crate::comp::Outcome>>().push(crate::comp::Outcome::RawDamage {
        pos: omoba_sim::Vec2::ZERO, phys: Fixed64::ZERO, magi: Fixed64::ZERO, real: Fixed64::from_i32(80),
        source: hero, target: hero, damage_profile: pure.bits(), origin: None });
    for _ in 0..2 { driver.step(&mut world, []).unwrap(); }
    let last = calls.lock().unwrap().iter().rev().find_map(|c| match c {
        Call::Status { kind, amount, damage_kind, .. } if *kind == crate::runtime::native::status_event::TAKE_DAMAGE => Some((*amount, *damage_kind)),
        _ => None });
    assert_eq!(last, Some((Fixed64::from_i32(30), 2 | StatusEvent::LETHAL)));
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
        Call::DamagePre { holder, kind, amount, .. } => Some((*holder, *kind, *amount)), _ => None }).collect();
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

/// A unit-target cast with a windup: its range is judged when the windup begins, and the release
/// follows that judgment even if the target has moved away (a team replica's view of the target
/// may differ from the authority's by then; e187).
#[test]
fn a_released_windup_follows_the_range_judged_when_it_began() {
    use crate::runtime::{SingleLaneConfig, StateInitializer};
    use crate::runtime::native::comp::Pos;
    let calls = Arc::new(Mutex::new(Vec::new()));
    let pool = StateInitializer::create_thread_pool();
    let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
    crate::runtime::setup_single_lane_match(&mut world, SingleLaneConfig {
        warmup: Fixed64::ZERO, wave_interval: Fixed64::from_i32(1_000_000), ..SingleLaneConfig::default()
    }).unwrap();
    let slot = |world: &specs::World, player: u32| world.read_resource::<crate::runtime::MobaMatch>().heroes.iter()
        .find(|slot| slot.player_id == player).and_then(|slot| slot.entity).unwrap();
    let (hero, enemy) = (slot(&world, 1), slot(&world, 2));
    let id = world.read_storage::<Hero>().get(hero).unwrap().abilities[0].clone();
    let level = serde_json::json!({"cooldown": 1, "mana_cost": 0, "cast_time": 0.25, "range": 300});
    let def: crate::ability_meta::AbilityDef = serde_json::from_value(serde_json::json!({
        "id": id, "name": id, "ability_type": "active", "target_type": "unit", "max_level": 4,
        "levels": {"1": level, "2": level, "3": level, "4": level},
    })).unwrap();
    world.insert(crate::runtime::TowerTemplateRegistry::default());
    let mut abilities = crate::runtime::ability_runtime::AbilityRegistry::new();
    abilities.register(def.clone());
    world.insert(abilities);
    let mut scripts = crate::runtime::ScriptRegistry::new();
    scripts.insert_ability_for_test(def, omb_script_abi::ability::AbilityScript_TO::from_value(
        Probe { calls: calls.clone(), id: id.clone() }, abi_stable::sabi_trait::prelude::TD_Opaque));
    world.insert(scripts);
    let mut driver = crate::runtime::SimulationDriver::from_world(&mut world, crate::runtime::SimulationTickProfile::Production60Hz).unwrap();
    driver.step(&mut world, []).unwrap();
    let at = world.read_storage::<Pos>().get(hero).unwrap().0;
    world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = at + omoba_sim::Vec2::new(Fixed64::from_i32(200), Fixed64::ZERO);
    {
        use crate::game_proto::{player_input::Action, CastAbility, PlayerInput};
        driver.step(&mut world, [(1, PlayerInput { action: Some(Action::CastAbility(CastAbility {
            ability_index: 0, target_entity: Some(enemy.id()), ..Default::default() })) })]).unwrap();
    }
    let executed = |calls: &Arc<Mutex<Vec<Call>>>| calls.lock().unwrap().iter().filter(|c| matches!(c, Call::ReadVar(_))).count();
    assert_eq!(executed(&calls), 0, "the windup runs first");
    // The target walks far out of range during the windup.
    world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = at + omoba_sim::Vec2::new(Fixed64::from_i32(2_000), Fixed64::ZERO);
    for _ in 0..30 { driver.step(&mut world, []).unwrap(); }
    assert_eq!(executed(&calls), 1, "the release follows the range judged when the windup began");
}

/// E449: a status listening to on_mana_spent hears its holder's spend, with the amount.
#[test]
fn a_marked_status_hears_its_holders_mana_spend() {
    use crate::runtime::native::status_event as se;
    let calls = Arc::new(Mutex::new(Vec::new()));
    let (mut world, mut driver, hero, id) = driven(calls.clone());
    driver.step(&mut world, []).unwrap();
    let marker = se::Spec { ability: id.clone(), level: 1, status: "probe_drain".into(), source: EntityHandle { id: hero.id(), gen: hero.gen().id() as u32 }, mask: se::ON_MANA_SPENT };
    world.write_resource::<BuffStore>().add(hero, "probe_drain", Fixed64::from_i32(5), serde_json::json!({ (se::KEY): marker.row() }));
    world.write_resource::<Vec<crate::comp::Outcome>>().push(crate::comp::Outcome::ScriptManaSpent {
        entity: hero, amount: Fixed64::from_i32(75), ability_id: id.clone() });
    for _ in 0..2 { driver.step(&mut world, []).unwrap(); }
    let heard: Vec<_> = calls.lock().unwrap().iter().filter_map(|c| match c {
        Call::Status { holder, status, kind, amount, .. } if *kind == se::MANA_SPENT => Some((*holder, status.clone(), *amount)), _ => None }).collect();
    assert_eq!(heard, vec![(hero.id(), "probe_drain".to_owned(), Fixed64::from_i32(75))]);
}

/// E496: a status listening to on_attack_miss hears its holder's missed attacks (0: missed at
/// launch, 1: evaded), not its hits.
#[test]
fn a_marked_status_hears_its_holders_attack_miss() {
    use crate::runtime::native::status_event as se;
    let calls = Arc::new(Mutex::new(Vec::new()));
    let (mut world, mut driver, hero, id) = driven(calls.clone());
    driver.step(&mut world, []).unwrap();
    let marker = se::Spec { ability: id.clone(), level: 1, status: "probe_fury".into(), source: EntityHandle { id: hero.id(), gen: hero.gen().id() as u32 }, mask: se::ON_ATTACK_MISS };
    world.write_resource::<BuffStore>().add(hero, "probe_fury", Fixed64::from_i32(5), serde_json::json!({ (se::KEY): marker.row() }));
    for (missed, evaded) in [(true, false), (false, true), (false, false)] {
        world.write_resource::<Vec<crate::comp::Outcome>>().push(crate::comp::Outcome::InstantAttack { attacker: hero, target: hero, critical: None, missed, evaded });
        driver.step(&mut world, []).unwrap();
    }
    driver.step(&mut world, []).unwrap();
    let heard: Vec<_> = calls.lock().unwrap().iter().filter_map(|c| match c {
        Call::Status { holder, kind, damage_kind, .. } if *kind == se::ATTACK_MISS => Some((*holder, *damage_kind)), _ => None }).collect();
    assert_eq!(heard, vec![(hero.id(), 0), (hero.id(), 1)], "the two misses, not the hit");
}

/// E465: a status listening to on_order_targeted hears an accepted order naming its holder, with
/// the unit ordered as `other` and the order kind (0 attack, 1 cast) as `damage_kind`.
#[test]
fn a_marked_status_hears_the_order_that_targets_its_holder() {
    use crate::runtime::native::status_event as se;
    use specs::Builder;
    let calls = Arc::new(Mutex::new(Vec::new()));
    let (mut world, mut driver, hero, id) = driven(calls.clone());
    driver.step(&mut world, []).unwrap();
    let marker = se::Spec { ability: id.clone(), level: 1, status: "probe_eye".into(), source: EntityHandle { id: hero.id(), gen: hero.gen().id() as u32 }, mask: se::ON_ORDER_TARGETED };
    world.write_resource::<BuffStore>().add(hero, "probe_eye", Fixed64::from_i32(5), serde_json::json!({ (se::KEY): marker.row() }));
    let enemy = world.create_entity().build();
    crate::runtime::native::order_targets::record(&mut world, enemy, hero, crate::runtime::native::order_targets::OrderKind::Cast, Some(&id));
    crate::runtime::native::order_targets::record(&mut world, enemy, hero, crate::runtime::native::order_targets::OrderKind::Attack, None);
    driver.step(&mut world, []).unwrap();
    let heard: Vec<_> = calls.lock().unwrap().iter().filter_map(|c| match c {
        Call::Status { holder, status, kind, other, damage_kind, .. } if *kind == se::ORDER_TARGETED => Some((*holder, status.clone(), *other, *damage_kind)), _ => None }).collect();
    assert_eq!(heard, vec![(hero.id(), "probe_eye".to_owned(), enemy.id(), 1), (hero.id(), "probe_eye".to_owned(), enemy.id(), 0)]);
}

/// E478: a status listening to on_heal_received hears its holder's heal, with the health gained
/// (capped at the maximum); a heal at full health gains nothing and is not heard.
#[test]
fn a_marked_status_hears_its_holders_heal() {
    use crate::runtime::native::status_event as se;
    let calls = Arc::new(Mutex::new(Vec::new()));
    let (mut world, mut driver, hero, id) = driven(calls.clone());
    driver.step(&mut world, []).unwrap();
    let marker = se::Spec { ability: id.clone(), level: 1, status: "probe_mend".into(), source: EntityHandle { id: hero.id(), gen: hero.gen().id() as u32 }, mask: se::ON_HEAL_RECEIVED };
    world.write_resource::<BuffStore>().add(hero, "probe_mend", Fixed64::from_i32(5), serde_json::json!({ (se::KEY): marker.row() }));
    let mhp = world.read_storage::<crate::runtime::native::comp::CProperty>().get(hero).unwrap().mhp;
    world.write_storage::<crate::runtime::native::comp::CProperty>().get_mut(hero).unwrap().hp = mhp - Fixed64::from_i32(25);
    for _ in 0..2 {
        world.write_resource::<Vec<crate::comp::Outcome>>().push(crate::comp::Outcome::Heal { pos: omoba_sim::Vec2::ZERO, target: hero, amount: Fixed64::from_i32(40) });
        driver.step(&mut world, []).unwrap();
    }
    driver.step(&mut world, []).unwrap();
    let heard: Vec<_> = calls.lock().unwrap().iter().filter_map(|c| match c {
        Call::Status { holder, kind, amount, .. } if *kind == se::HEAL_RECEIVED => Some((*holder, *amount)), _ => None }).collect();
    assert_eq!(heard.len(), 1, "the heal at full health gains nothing: {heard:?}");
    assert_eq!(heard[0].0, hero.id());
    assert!(heard[0].1 > Fixed64::ZERO && heard[0].1 <= Fixed64::from_i32(25), "at most the missing health: {heard:?}");
}

/// A status script that hears the order filters on its holder, spends 10 of the holder's mana
/// (an effect only the holder's team holds) and refuses the order when `cancel`.
#[derive(Clone)]
struct OrderGuard { heard: Arc<Mutex<Vec<(u32, u32)>>>, id: String, cancel: bool, extra: Extra }

/// What an order-filter world adds to the plain probe and guard: a ward (the guard shields its
/// holder, the cast is a zap), a purge (the guard puts `purge_statuses` on its holder, the cast
/// purges them by duration and source) or a death (the cast is a zap, the guard's holder has
/// little life and the guard also hears its death).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Extra { Plain, Ward, Purge, Death }

/// The hero variable (kept across death) the guard writes when its holder dies.
const DEATH_VAR: &str = "test:deaths_heard";

/// Three statuses of the compiled content, by their BuffStore keys (no runtime id, no tags): two
/// of plain abilities and one of an ultimate.
fn purge_statuses() -> (String, String, String) {
    let doc: serde_json::Value = serde_json::from_str(omoba_template_ids::V3_CATALOG_JSON).unwrap();
    let plain = |ultimate: bool| doc["abilities"].as_array().unwrap().iter()
        .filter(move |entry| (entry["ability_type"].as_str() == Some("ultimate")) == ultimate)
        .flat_map(|entry| entry["v3"]["statuses"].as_array().cloned().unwrap_or_default())
        .filter(|status| status.get("runtime_id").is_none() && status.get("tags").is_none())
        .filter_map(|status| status["id"].as_str().map(str::to_owned));
    let mut ordinary = plain(false);
    (ordinary.next().unwrap(), ordinary.next().unwrap(), plain(true).next().unwrap())
}

/// 冰爆之書's purge as its generated code writes it (ability_kit remove_statuses_by_source):
/// statuses the target's own side put on it for longer than half a second, but not an
/// ultimate's, one with no source ability or one whose applier is gone.
#[derive(Clone)]
struct Purger { id: String }

impl omb_script_abi::ability::AbilityScript for Purger {
    fn ability_id(&self) -> RStr<'_> { RStr::from_str(&self.id) }
    fn execute(&self, _: EntityHandle, target: Target, _: u8, _: RStr<'_>, world: &mut GameWorldDyn<'_>) -> RResult<(), RString> {
        use crate::runtime::native::dispel::{PurgeMode, PurgeSpec, SourceIs, SourceRule, SourceTeam, PURGE_BUFF};
        if let Target::Entity(target) = target {
            let spec = PurgeSpec { mode: PurgeMode::All, side: None, team: SourceTeam::Ally, except: Vec::new(), states: Vec::new(),
                limit: None, tags: Vec::new(), exclude_tags: Vec::new(), max_elapsed: None,
                source: SourceRule { min_duration: Some(Fixed64::from_raw(512)), exclude_source_tags: vec!["ultimate".into()],
                    source_is: vec![SourceIs::HasAbility, SourceIs::Alive] } };
            world.add_stat_buff(target, RStr::from_str(PURGE_BUFF), Fixed64::ONE, RStr::from_str(&spec.payload().to_string()));
        }
        ROk(())
    }
}

/// The ward an order guard in `ward` mode puts on its holder for each order: a new independent
/// instance (`test_ward#<n>`) absorbing 300 magical damage before mitigation (absorb.rs), as the
/// generated code of 御魔護符's magic_talisman_shield writes it (ability_kit `absorb_with`).
const WARD: &str = "test_ward";

/// A unit-target cast dealing 200 magical spell damage to its target.
#[derive(Clone)]
struct Zap { id: String }

impl omb_script_abi::ability::AbilityScript for Zap {
    fn ability_id(&self) -> RStr<'_> { RStr::from_str(&self.id) }
    fn execute(&self, caster: EntityHandle, target: Target, _: u8, _: RStr<'_>, world: &mut GameWorldDyn<'_>) -> RResult<(), RString> {
        if let Target::Entity(target) = target {
            world.deal_damage(target, Fixed64::from_i32(200), omb_script_abi::types::DamageKind::Magical,
                omb_script_abi::types::DamageProfile::NORMAL.union(omb_script_abi::types::DamageProfile::SPELL),
                abi_stable::std_types::ROption::RSome(caster));
        }
        ROk(())
    }
}

impl omb_script_abi::ability::AbilityScript for OrderGuard {
    fn ability_id(&self) -> RStr<'_> { RStr::from_str(&self.id) }
    fn execute(&self, _: EntityHandle, _: Target, _: u8, _: RStr<'_>, _: &mut GameWorldDyn<'_>) -> RResult<(), RString> { ROk(()) }
    fn on_status_event(&self, holder: EntityHandle, _: RStr<'_>, event: StatusEvent, world: &mut GameWorldDyn<'_>) {
        use crate::runtime::native::unit_var;
        if event.kind == crate::runtime::native::status_event::OWNER_DEATH {
            // Heard while the holder is whole: it still reads its own variables and life.
            self.heard.lock().unwrap().push((holder.id, u32::MAX));
            let before = world.get_buff_payload(holder, RStr::from_str(&format!("{}{DEATH_VAR}", unit_var::PREFIX))).into_option()
                .and_then(|row| serde_json::from_str::<serde_json::Value>(row.as_str()).ok())
                .and_then(|row| unit_var::Spec::from_buff(&format!("{}{DEATH_VAR}", unit_var::PREFIX), &row))
                .map_or(Fixed64::ZERO, |spec| match spec.value { unit_var::VarValue::Num(n) => n, _ => Fixed64::ZERO });
            let spec = unit_var::Spec { key: DEATH_VAR.into(), value: unit_var::VarValue::Num(before + Fixed64::ONE), hero: true };
            world.add_stat_buff(holder, RStr::from_str(&spec.buff_id()), Fixed64::from_raw(i64::MAX), RStr::from_str(&spec.payload().to_string()));
            return;
        }
        if event.kind != crate::runtime::native::status_event::ORDER_FILTER { return; }
        self.heard.lock().unwrap().push((holder.id, event.other.id));
        world.spend_mana(holder, Fixed64::from_i32(10), RStr::from_str(&self.id));
        if self.extra == Extra::Purge {
            // 8 s, 0.45 s and an ultimate's 8 s, each put there by the holder itself.
            let (long, short, ultimate) = purge_statuses();
            for (status, seconds) in [(long, Fixed64::from_i32(8)), (short, Fixed64::from_raw(461)), (ultimate, Fixed64::from_i32(8))] {
                let row = serde_json::json!({ (crate::runtime::native::illusion::DURATION_KEY): seconds.raw(),
                    "source_entity_id": holder.id, "source_entity_gen": holder.gen });
                world.add_stat_buff(holder, RStr::from_str(&status), seconds, RStr::from_str(&row.to_string()));
            }
        }
        if self.extra == Extra::Ward {
            use crate::runtime::native::absorb::{Spec, Types, BUFF};
            let instance = (1..).find(|n| !world.has_buff(holder, RStr::from_str(&format!("{WARD}#{n}")))).unwrap();
            let status = format!("{WARD}#{instance}");
            let row = serde_json::json!({ (crate::runtime::native::illusion::DURATION_KEY): Fixed64::from_i32(8).raw(),
                "source_entity_id": holder.id, "source_entity_gen": holder.gen });
            world.add_stat_buff(holder, RStr::from_str(&status), Fixed64::from_i32(8), RStr::from_str(&row.to_string()));
            let spec = Spec { status, amount: Fixed64::from_i32(300), types: Types::from_names(&["magical"]).unwrap(), pre_mitigation: true };
            world.add_stat_buff(holder, RStr::from_str(BUFF), Fixed64::from_i32(8), RStr::from_str(&spec.payload().to_string()));
        }
        if self.cancel {
            assert!(world.ext(omb_script_abi::ext::op::CANCEL_ORDER, abi_stable::std_types::RSlice::from_slice(&[])).is_ok());
        }
    }
}

const ORDER_GUARD: &str = "test_order_guard";

fn order_filter_def(id: &str, target: &str) -> crate::ability_meta::AbilityDef {
    let level = serde_json::json!({"cooldown": 1, "mana_cost": 0, "cast_time": 0.25, "range": 300});
    serde_json::from_value(serde_json::json!({
        "id": id, "name": id, "ability_type": "active", "target_type": target, "max_level": 4,
        "levels": {"1": level, "2": level, "3": level, "4": level},
    })).unwrap()
}

/// The probe `id` (a unit-target cast; a zap with a ward, a purger with a purge) and the order guard.
fn order_filter_scripts(calls: Arc<Mutex<Vec<Call>>>, heard: Arc<Mutex<Vec<(u32, u32)>>>, cancel: bool, extra: Extra, id: &str) -> crate::runtime::ScriptRegistry {
    use abi_stable::sabi_trait::prelude::TD_Opaque;
    use omb_script_abi::ability::AbilityScript_TO as To;
    let mut scripts = crate::runtime::ScriptRegistry::new();
    let cast = match extra {
        Extra::Ward | Extra::Death => To::from_value(Zap { id: id.to_owned() }, TD_Opaque),
        Extra::Purge => To::from_value(Purger { id: id.to_owned() }, TD_Opaque),
        Extra::Plain => To::from_value(Probe { calls, id: id.to_owned() }, TD_Opaque),
    };
    scripts.insert_ability_for_test(order_filter_def(id, "unit"), cast);
    scripts.insert_ability_for_test(order_filter_def(ORDER_GUARD, "none"), omb_script_abi::ability::AbilityScript_TO::from_value(
        OrderGuard { heard, id: ORDER_GUARD.to_owned(), cancel, extra }, abi_stable::sabi_trait::prelude::TD_Opaque));
    scripts
}

/// Player 1's hero, player 2's hero, a unit-target probe on the first and an order guard status
/// on the second (refusing when `cancel`). `replica` marks the world as a team replica.
fn order_filter_world(calls: Arc<Mutex<Vec<Call>>>, heard: Arc<Mutex<Vec<(u32, u32)>>>, cancel: bool, extra: Extra, replica: bool)
    -> (specs::World, crate::runtime::SimulationDriver, Entity, Entity, String) {
    use crate::runtime::{SingleLaneConfig, StateInitializer};
    let pool = StateInitializer::create_thread_pool();
    let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
    crate::runtime::setup_single_lane_match(&mut world, SingleLaneConfig {
        warmup: Fixed64::ZERO, wave_interval: Fixed64::from_i32(1_000_000), respawn_delay: Fixed64::from_i32(3), ..SingleLaneConfig::default()
    }).unwrap();
    let slot = |world: &specs::World, player: u32| world.read_resource::<crate::runtime::MobaMatch>().heroes.iter()
        .find(|slot| slot.player_id == player).and_then(|slot| slot.entity).unwrap();
    let (hero, enemy) = (slot(&world, 1), slot(&world, 2));
    let id = world.read_storage::<Hero>().get(hero).unwrap().abilities[0].clone();
    let guard = ORDER_GUARD.to_owned();
    world.insert(crate::runtime::TowerTemplateRegistry::default());
    let mut abilities = crate::runtime::ability_runtime::AbilityRegistry::new();
    abilities.register(order_filter_def(&id, "unit"));
    abilities.register(order_filter_def(&guard, "none"));
    world.insert(abilities);
    world.insert(order_filter_scripts(calls, heard, cancel, extra, &id));
    if replica { world.insert(crate::runtime::filtered_specs::DisclosedMovementPriority::default()); }
    let mut driver = crate::runtime::SimulationDriver::from_world(&mut world, crate::runtime::SimulationTickProfile::Production60Hz).unwrap();
    driver.step(&mut world, []).unwrap();
    use crate::runtime::native::status_event as se;
    let marker = se::Spec { ability: guard.clone(), level: 1, status: "guard".into(), source: EntityHandle { id: enemy.id(), gen: enemy.gen().id() as u32 },
        mask: if extra == Extra::Death { se::ON_ORDER_FILTER | se::ON_OWNER_DEATH } else { se::ON_ORDER_FILTER } };
    // A v3 status the holder put on itself, as the generated code writes it (owner_script_rows carries it).
    world.write_resource::<BuffStore>().add(enemy, "guard", Fixed64::from_i32(30), serde_json::json!({ (se::KEY): marker.row(),
        (crate::runtime::native::illusion::DURATION_KEY): Fixed64::from_i32(30).raw(),
        "source_entity_id": enemy.id(), "source_entity_gen": enemy.gen().id() as u32 }));
    // Halfway between the two spawns, out of the structures' reach, 200 apart.
    let mut pos = world.write_storage::<crate::runtime::native::comp::Pos>();
    let (a, b) = (pos.get(hero).unwrap().0, pos.get(enemy).unwrap().0);
    let mid = omoba_sim::Vec2::new((a.x + b.x) / Fixed64::from_i32(2), (a.y + b.y) / Fixed64::from_i32(2));
    pos.get_mut(hero).unwrap().0 = mid;
    pos.get_mut(enemy).unwrap().0 = mid + omoba_sim::Vec2::new(Fixed64::from_i32(200), Fixed64::ZERO);
    drop(pos);
    // Neither dies of the other's attacks during a soak.
    for unit in [hero, enemy] {
        let mut props = world.write_storage::<crate::runtime::native::comp::CProperty>();
        let prop = props.get_mut(unit).unwrap();
        (prop.mhp, prop.hp) = (Fixed64::from_i32(1_000_000), Fixed64::from_i32(1_000_000));
    }
    // A death world: the guarded hero falls to a few zaps.
    if extra == Extra::Death {
        let mut props = world.write_storage::<crate::runtime::native::comp::CProperty>();
        let prop = props.get_mut(enemy).unwrap();
        (prop.mhp, prop.hp) = (Fixed64::from_i32(450), Fixed64::from_i32(450));
    }
    (world, driver, hero, enemy, id)
}

fn cast_at(world: &mut specs::World, driver: &mut crate::runtime::SimulationDriver, target: Entity) {
    use crate::game_proto::{player_input::Action, CastAbility, PlayerInput};
    driver.step(world, [(1, PlayerInput { action: Some(Action::CastAbility(CastAbility {
        ability_index: 0, target_entity: Some(target.id()), ..Default::default() })) })]).unwrap();
}

fn executions(calls: &Arc<Mutex<Vec<Call>>>) -> usize { calls.lock().unwrap().iter().filter(|c| matches!(c, Call::ReadVar(_))).count() }

/// An order filter (status_event ORDER_FILTER) on the target hears the enemy's cast before its
/// windup, with the caster as `other`; refusing it means the cast never starts, and the authority
/// publishes the refusal for the team replicas.
#[test]
fn an_order_filter_on_the_target_refuses_the_cast_before_its_windup() {
    for cancel in [false, true] {
        let (calls, heard) = (Arc::new(Mutex::new(Vec::new())), Arc::new(Mutex::new(Vec::new())));
        let (mut world, mut driver, hero, enemy, _) = order_filter_world(calls.clone(), heard.clone(), cancel, Extra::Plain, false);
        cast_at(&mut world, &mut driver, enemy);
        for _ in 0..30 { driver.step(&mut world, []).unwrap(); }
        assert_eq!(*heard.lock().unwrap(), vec![(enemy.id(), hero.id())], "heard once, the caster as other (cancel {cancel})");
        assert_eq!(executions(&calls), usize::from(!cancel), "cancel {cancel}");
    }
}

/// A team replica never decides an order filter itself: the caster's team refuses only the casts
/// the authority refused, and the holder's team runs the guard for each refusal it is told of.
#[test]
fn a_replica_follows_the_authoritys_order_filter_refusals() {
    use crate::runtime::native::order_filter::OrderFilterInjections;
    // The guard would refuse, but the replica was told nothing: the cast goes ahead, unheard.
    let (calls, heard) = (Arc::new(Mutex::new(Vec::new())), Arc::new(Mutex::new(Vec::new())));
    let (mut world, mut driver, _, enemy, _) = order_filter_world(calls.clone(), heard.clone(), true, Extra::Plain, true);
    cast_at(&mut world, &mut driver, enemy);
    for _ in 0..30 { driver.step(&mut world, []).unwrap(); }
    assert!(heard.lock().unwrap().is_empty());
    assert_eq!(executions(&calls), 1);
    // Told of the refusal: the cast is refused and the guard runs once on its holder.
    let (calls, heard) = (Arc::new(Mutex::new(Vec::new())), Arc::new(Mutex::new(Vec::new())));
    let (mut world, mut driver, hero, enemy, id) = order_filter_world(calls.clone(), heard.clone(), false, Extra::Plain, true);
    world.insert(OrderFilterInjections { refused: vec![(hero, id.clone())], holders: vec![(enemy, Some(hero), id)] });
    cast_at(&mut world, &mut driver, enemy);
    for _ in 0..30 { driver.step(&mut world, []).unwrap(); }
    assert_eq!(*heard.lock().unwrap(), vec![(enemy.id(), hero.id())]);
    assert_eq!(executions(&calls), 0);
}

/// Replica soak for order filters: the authority and every team's replica (in process, through
/// the server's projection) step a match where player 1 casts at player 2's guarded hero, and
/// every replica's team hash matches the authority's view each tick, refused or not. The caster's
/// team cannot see the guard and the holder's team cannot see the order, so only the published
/// refusal keeps them together.
#[test]
fn order_filter_refusals_keep_every_team_replica_with_the_authority() {
    for cancel in [true, false] {
        let (_, _, calls, heard) = order_filter_soak(cancel, Extra::Plain);
        let executed = executions(&calls);
        if cancel { assert_eq!(executed, 0, "refused everywhere"); } else { assert!(executed >= 10, "executed {executed}"); }
        // The authority and the holder's team replica (from the published fact) both run the guard,
        // for every order, refused or not.
        assert!(heard.lock().unwrap().len() >= 20, "the guard heard each order twice: {:?}", heard.lock().unwrap());
    }
}

/// Replica soak for pre-mitigation shields on independent instances (御魔護符's magic shield):
/// each order puts a new 300 magical pre-mitigation ward instance on the target, then the zap's
/// 200 magical damage drains them; every team replica matches the authority's view each tick.
#[test]
fn pre_mitigation_ward_instances_keep_every_team_replica_with_the_authority() {
    let (world, enemy, _, heard) = order_filter_soak(false, Extra::Ward);
    assert!(heard.lock().unwrap().len() >= 20, "the authority and the holder's replica heard each order: {:?}", heard.lock().unwrap());
    let wards: Vec<i64> = world.read_resource::<BuffStore>().iter_for(enemy)
        .filter(|(id, _)| id.starts_with(&format!("__absorb:{WARD}#")))
        .filter_map(|(_, row)| row.payload.get("remaining_raw")?.as_i64()).collect();
    assert!(wards.iter().any(|left| *left < Fixed64::from_i32(300).raw()), "the last zap drained a ward: {wards:?}");
}

/// Replica soak for a purge by duration and source (冰爆之書／吉光骨食／冰風暴之書): each order
/// has the target's guard put an 8 s, a 0.45 s and an ultimate's 8 s status on it, then the cast
/// purges what it put there for longer than half a second unless an ultimate put it; every team
/// replica matches the authority's view each tick.
#[test]
fn a_purge_by_duration_and_source_keeps_every_team_replica_with_the_authority() {
    let (world, enemy, _, heard) = order_filter_soak(false, Extra::Purge);
    assert!(heard.lock().unwrap().len() >= 20, "the authority and the holder's replica heard each order: {:?}", heard.lock().unwrap());
    let (long, short, ultimate) = purge_statuses();
    let buffs = world.read_resource::<BuffStore>();
    assert!(!buffs.has(enemy, &long), "the last purge took the long status");
    assert!(buffs.has(enemy, &short) && buffs.has(enemy, &ultimate), "the short and the ultimate's stay");
}

/// v3 on_owner_death / an item passive's on_death (scripting::owner_death): the guarded hero
/// dies to the zaps; its guard hears the death once, before it settles, and the hero variable
/// it writes then is on the hero's next life. Every team replica matches the authority's view
/// each tick, through the death and the respawn.
#[test]
fn an_owner_death_hook_runs_before_the_death_settles_and_keeps_every_team_replica_with_the_authority() {
    let (world, enemy, _, heard) = order_filter_soak(false, Extra::Death);
    let deaths: Vec<(u32, u32)> = heard.lock().unwrap().iter().copied().filter(|(_, other)| *other == u32::MAX).collect();
    assert!(!deaths.is_empty() && deaths.iter().all(|(holder, _)| *holder == enemy.id()), "the first life heard its death: {deaths:?}");
    let state = world.read_resource::<crate::runtime::MobaMatch>();
    let slot = state.heroes.iter().find(|slot| slot.player_id == 2).unwrap();
    assert!(slot.deaths >= 1, "the guarded hero died");
    let life = slot.entity.expect("the hero respawned");
    assert_ne!(life, enemy);
    let row = world.read_resource::<BuffStore>().get(life, &format!("{}{DEATH_VAR}", crate::runtime::native::unit_var::PREFIX))
        .map(|row| row.payload.clone());
    let kept = row.as_ref().and_then(|row| crate::runtime::native::unit_var::Spec::from_buff(
        &format!("{}{DEATH_VAR}", crate::runtime::native::unit_var::PREFIX), row));
    assert_eq!(kept.map(|spec| spec.value), Some(crate::runtime::native::unit_var::VarValue::Num(Fixed64::ONE)),
        "the variable the hook wrote while the hero died is on its next life: {row:?}");
}

/// The authority and every team's replica (in process, through the server's projection) step
/// 1440 ticks of `order_filter_world`, player 1 casting at player 2's hero 20 ticks before each
/// checkpoint; fails at the first tick a replica's team hash differs from the authority's view.
fn order_filter_soak(cancel: bool, extra: Extra) -> (specs::World, Entity, Arc<Mutex<Vec<Call>>>, Arc<Mutex<Vec<(u32, u32)>>>) {
    use crate::runtime::inprocess_observer::{accepted_inputs, project_tick, InProcessObserver};
    use crate::game_proto::{player_input::Action, CastAbility, PlayerInput};
    {
        let (calls, heard) = (Arc::new(Mutex::new(Vec::new())), Arc::new(Mutex::new(Vec::new())));
        let (mut world, mut driver, _, enemy, id) = order_filter_world(calls.clone(), heard.clone(), cancel, extra, false);
        let step = |world: &mut specs::World, driver: &mut crate::runtime::SimulationDriver, inputs: Vec<(u32, PlayerInput)>| {
            let accepted = accepted_inputs(world, driver.tick() + 1, &inputs);
            let result = driver.step(world, inputs).unwrap();
            world.write_resource::<crate::runtime::TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted);
            project_tick(world, &result).unwrap();
            result.tick
        };
        step(&mut world, &mut driver, Vec::new());
        let seed = world.try_fetch::<crate::runtime::MasterSeed>().map_or(0, |seed| seed.0);
        let mut observer = InProcessObserver::start(&mut world, driver.tick() + 1, seed, 1, 60).unwrap();
        observer.replace_scripts(|| order_filter_scripts(calls.clone(), heard.clone(), cancel, extra, &id));
        // The checkpoint is every 120 ticks: each order 20 ticks before one, so a refused cast, a
        // windup, a cooldown or the guard's own effects are all live on a compared tick.
        for at in 0..1440 {
            let inputs = if (driver.tick() + 1) % 120 == 100 {
                vec![(1, PlayerInput { action: Some(Action::CastAbility(CastAbility {
                    ability_index: 0, target_entity: Some(enemy.id()), ..Default::default() })) })]
            } else { Vec::new() };
            let tick = step(&mut world, &mut driver, inputs);
            assert_eq!(observer.after_projection(&mut world, tick).unwrap(), None, "cancel {cancel}, {extra:?}, tick {tick}");
        }
        assert!(observer.checked_ticks >= 11, "compared {} ticks", observer.checked_ticks);
        (world, enemy, calls, heard)
    }
}
