//! Deterministic summon lifetime and player-command movement.
use specs::{shred, Entities, Join, Read, ReadStorage, SystemData, Write, WriteStorage};
use crate::comp::*;
use omoba_sim::Fixed64;

/// Phase-local ownership; never serialized or replicated. Entity includes its generation.
#[derive(Default)]
pub(crate) struct HostMovement {
    tick: Option<u64>,
    moved: std::collections::BTreeSet<specs::Entity>,
}
impl HostMovement {
    fn reset(&mut self,tick:u64) {self.tick=Some(tick);self.moved.clear();}
    pub(crate) fn owns(&self,tick:u64,entity:specs::Entity)->bool {
        self.tick==Some(tick) && self.moved.contains(&entity)
    }
}

#[derive(SystemData)]
pub struct SummonTickData<'a> {
    entities: Entities<'a>,
    dt: Read<'a, DeltaTime>,
    pause: Read<'a, GamePause>,
    tick: Read<'a, Tick>,
    host_movement: Write<'a, HostMovement>,
    summoned: WriteStorage<'a, SummonedUnit>,
    property: ReadStorage<'a, CProperty>,
    pos: WriteStorage<'a, Pos>,
    move_targets: WriteStorage<'a, MoveTarget>,
    ordered: WriteStorage<'a, crate::runtime::native::hero_command::OrderedAttack>,
    following: WriteStorage<'a, crate::runtime::native::hero_command::OrderedFollow>,
    attacks: ReadStorage<'a, TAttack>,
    facings: WriteStorage<'a, Facing>,
    collision: ReadStorage<'a, CollisionRadius>,
    blocked: Read<'a, BlockedRegions>,
    buffs: Read<'a, crate::runtime::BuffStore>,
    outcomes: Write<'a, Vec<Outcome>>,
}

#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = SummonTickData<'a>;
    const NAME: &'static str = "summon";
    fn run(_job:&mut Job<Self>,mut data:Self::SystemData) {
        data.host_movement.reset(data.tick.0);
        if data.pause.is_paused || data.dt.0<=Fixed64::ZERO {return;}
        let dt=data.dt.0;
        for (entity,summon) in (&data.entities,&mut data.summoned).join() {
            let was_expired=summon.is_expired();
            if summon.update(dt) && !was_expired {
                data.outcomes.push(Outcome::EntityRemoved {entity});
            }
        }
        let owners:std::collections::BTreeMap<_,_>=(&data.entities,&data.summoned).join()
            .filter_map(|(_,s)|data.entities.is_alive(s.summoner)
                .then(||data.property.get(s.summoner).filter(|p|p.hp>Fixed64::ZERO)
                    .and_then(|_|data.pos.get(s.summoner)).map(|p|(s.summoner,p.0))).flatten()).collect();
        for (entity,summon) in (&data.entities,&mut data.summoned).join() {
            // G-3 owner policy: dead (HP 0) or retired exact owner life ends the summon.
            if summon.expire_with_owner && !owners.contains_key(&summon.summoner) && summon.expire_now() {
                data.outcomes.push(Outcome::EntityRemoved {entity});
            }
        }
        // An ordered attack (hero_command, E412) chases its living target until it is in attack
        // range, then stands for the AI to attack it; a dead or gone target ends the order.
        let mut ended=Vec::new();
        for (entity,summon,ordered) in (&data.entities,&data.summoned,&data.ordered).join() {
            let target=ordered.0;
            let goal=(data.entities.is_alive(target) && data.property.get(target).is_some_and(|p|p.hp>Fixed64::ZERO))
                .then(||data.pos.get(target).map(|p|p.0)).flatten();
            let (Some(goal),Some(at))=(goal,data.pos.get(entity).map(|p|p.0)) else {ended.push(entity);continue;};
            if summon.is_expired() || summon.immobile {ended.push(entity);continue;}
            let base=data.attacks.get(entity).map_or(Fixed64::ZERO,|attack|attack.range.v);
            let range=crate::runtime::UnitStats::from_refs(&data.buffs,false).final_attack_range(base,entity);
            if (goal-at).length()>range {let _=data.move_targets.insert(entity,MoveTarget(goal));}
            else {data.move_targets.remove(entity);}
        }
        for entity in ended {data.ordered.remove(entity);data.move_targets.remove(entity);}
        // An ordered follow (E489) walks to its living leader whenever it is out of reach and
        // keeps the order; a dead or gone leader ends it.
        let mut ended=Vec::new();
        let gap=Fixed64::from_i32(crate::runtime::native::hero_command::FOLLOW_GAP);
        for (entity,summon,following) in (&data.entities,&data.summoned,&data.following).join() {
            let leader=following.0;
            let goal=(data.entities.is_alive(leader) && data.property.get(leader).is_some_and(|p|p.hp>Fixed64::ZERO))
                .then(||data.pos.get(leader).map(|p|p.0)).flatten();
            let (Some(goal),Some(at))=(goal,data.pos.get(entity).map(|p|p.0)) else {ended.push(entity);continue;};
            if summon.is_expired() || summon.immobile {ended.push(entity);continue;}
            let radius=|e|data.collision.get(e).map_or(Fixed64::ZERO,|r|r.0);
            if (goal-at).length()>radius(entity)+radius(leader)+gap {let _=data.move_targets.insert(entity,MoveTarget(goal));}
            else {data.move_targets.remove(entity);}
        }
        for entity in ended {data.following.remove(entity);data.move_targets.remove(entity);}
        for (entity,summon) in (&data.entities,&mut data.summoned).join() {
            if summon.is_expired() || data.property.get(entity).is_none_or(|p|p.hp<=Fixed64::ZERO) {continue;}
            // An ordered attack owns its movement; the follow rules wait.
            if data.ordered.get(entity).is_some() || data.following.get(entity).is_some() {summon.navigation.auto_target=None;continue;}
            // Fixed emplacements never acquire follow or autonomous destinations.
            if summon.immobile {summon.navigation.auto_target=None;continue;}
            let nav=&mut summon.navigation;
            let current=data.move_targets.get(entity).map(|m|m.0);
            // A command replacing the autonomous destination owns movement until
            // it completes. Never overwrite it with a stale follow destination.
            if current.is_some() && current!=nav.auto_target {nav.auto_target=None;continue;}
            let owner=owners.get(&summon.summoner).copied();
            let position=data.pos.get(entity).map(|p|p.0);
            if let Some((owner,position))=owner.zip(position).filter(|_|nav.follow_start_distance>Fixed64::ZERO) {
                let distance=(owner-position).length();
                let threshold=if nav.auto_target.is_some() {nav.follow_stop_distance}else{nav.follow_start_distance};
                if distance>threshold+Fixed64::from_raw(2) {
                    nav.auto_target=Some(owner);
                    let _=data.move_targets.insert(entity,MoveTarget(owner));
                    continue;
                }
            }
            if nav.auto_target.take().is_some() {data.move_targets.remove(entity);}
        }
        let mut arrived=Vec::new();
        for (entity,summon,property,pos,target,facing) in (&data.entities,&mut data.summoned,
            &data.property,&mut data.pos,&data.move_targets,&mut data.facings).join() {
            if summon.is_expired() || property.hp<=Fixed64::ZERO {continue;}
            if summon.immobile || data.buffs.is_rooted(entity) {continue;}
            let delta=target.0-pos.0;
            let distance=delta.length();
            if distance<=Fixed64::from_raw(1) {arrived.push(entity);continue;}
            let speed=crate::runtime::UnitStats::from_refs(&data.buffs,false).final_move_speed(property.msd,entity);
            let mut step=speed.max(Fixed64::ZERO)*dt;
            if summon.navigation.auto_target.is_some() {
                step=step.min((distance-summon.navigation.follow_stop_distance).max(Fixed64::ZERO));
            }
            if step<=Fixed64::ZERO {continue;}
            let radius=data.collision.get(entity).map(|r|r.0).unwrap_or(Fixed64::ZERO);
            let next=super::super::summon_navigation::advance(&mut summon.navigation,pos.0,target.0,
                step.min(distance),dt,radius,
                crate::runtime::native::rules::world::movement_regions(&data.buffs,entity,&data.blocked));
            let moved=next-pos.0;
            // Compare coordinates directly: squared fixed-point length can erase tiny steps.
            if next!=pos.0 {data.host_movement.moved.insert(entity);}
            if moved.length_squared()>Fixed64::ZERO {facing.0=omoba_sim::trig::atan2(moved.y,moved.x);}
            pos.0=next;
            if (target.0-pos.0).length()<=Fixed64::from_raw(1) {arrived.push(entity);}
        }
        for entity in arrived {data.move_targets.remove(entity);}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder,World,WorldExt};
    #[test]
    fn summon_host_frame_ownership_is_exact_generation_tick_and_reset() {
        let mut world=World::new();let old=world.create_entity().build();
        let mut record=HostMovement::default();assert!(!record.owns(0,old));
        record.reset(42);record.moved.insert(old);assert!(record.owns(42,old));assert!(!record.owns(43,old));
        world.delete_entity(old).unwrap();world.maintain();let fresh=world.create_entity().build();
        assert_eq!(old.id(),fresh.id());assert_ne!(old,fresh);assert!(!record.owns(42,fresh));
        record.reset(42);assert!(!record.owns(42,old));
        record.moved.insert(fresh);record.reset(43);assert!(!record.owns(43,fresh));
    }
    fn fixture()->(World,specs::Entity,specs::Entity,specs::Dispatcher<'static,'static>) {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        let property=CProperty {hp:Fixed64::from_i32(100),mhp:Fixed64::from_i32(100),
            msd:Fixed64::from_i32(320),def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO};
        let owner=world.create_entity().with(Pos(omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO)))
            .with(property.clone()).build();
        let mut summoned=SummonedUnit::new_fixed(owner,Some(Fixed64::from_i32(30)));
        summoned.navigation.follow_start_distance=Fixed64::from_i32(100);
        summoned.navigation.follow_stop_distance=Fixed64::from_i32(40);
        let unit=world.create_entity().with(summoned).with(property).with(Pos(omoba_sim::Vec2::ZERO))
            .with(Facing(omoba_sim::Angle::ZERO)).with(CollisionRadius(Fixed64::from_i32(10))).build();
        let mut builder=specs::DispatcherBuilder::new();crate::comp::dispatch::<Sys>(&mut builder,&[]);
        let mut dispatcher=builder.build();dispatcher.setup(&mut world);
        world.write_resource::<DeltaTime>().0=Fixed64::from_raw(17);
        (world,owner,unit,dispatcher)
    }
    #[test]
    fn follow_hysteresis_respects_explicit_commands_root_pause_and_owner_generation() {
        let (mut world,owner,unit,mut dispatcher)=fixture();
        dispatcher.dispatch(&world);
        assert!(world.read_resource::<HostMovement>().owns(world.read_resource::<Tick>().0,unit));
        assert!(world.read_storage::<SummonedUnit>().get(unit).unwrap().navigation.auto_target.is_some());
        let before=world.read_storage::<Pos>().get(unit).unwrap().0;
        world.write_resource::<crate::runtime::BuffStore>().add(unit,"root",Fixed64::from_i32(10),serde_json::json!({}));
        dispatcher.dispatch(&world);assert_eq!(world.read_storage::<Pos>().get(unit).unwrap().0,before);
        assert!(!world.read_resource::<HostMovement>().owns(world.read_resource::<Tick>().0,unit));
        world.write_resource::<crate::runtime::BuffStore>().remove(unit,"root");
        world.write_resource::<GamePause>().is_paused=true;
        let remaining=world.read_storage::<SummonedUnit>().get(unit).unwrap().time_remaining;
        dispatcher.dispatch(&world);assert_eq!(world.read_storage::<SummonedUnit>().get(unit).unwrap().time_remaining,remaining);
        world.write_resource::<GamePause>().is_paused=false;
        let explicit=before+omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(150));
        world.write_storage::<MoveTarget>().insert(unit,MoveTarget(explicit)).unwrap();
        for _ in 0..12 {dispatcher.dispatch(&world);}
        assert_eq!(world.read_storage::<Pos>().get(unit).unwrap().0.x,before.x);
        assert_eq!(world.read_storage::<MoveTarget>().get(unit).unwrap().0,explicit);
        assert!(world.read_storage::<SummonedUnit>().get(unit).unwrap().navigation.auto_target.is_none());
        world.write_storage::<MoveTarget>().remove(unit);
        for _ in 0..90 {dispatcher.dispatch(&world);}
        let owner_pos=world.read_storage::<Pos>().get(owner).unwrap().0;
        assert!((world.read_storage::<Pos>().get(unit).unwrap().0-owner_pos).length()<=Fixed64::from_i32(41));
        assert!(world.read_storage::<MoveTarget>().get(unit).is_none());
        world.write_storage::<Pos>().get_mut(owner).unwrap().0.x=Fixed64::from_i32(500);
        dispatcher.dispatch(&world);assert!(world.read_storage::<MoveTarget>().get(unit).is_some());
        world.delete_entity(owner).unwrap();world.maintain();let replacement=world.create_entity().with(Pos(owner_pos)).build();
        assert_eq!(replacement.id(),owner.id());assert_ne!(replacement.gen(),owner.gen());
        let before=world.read_storage::<Pos>().get(unit).unwrap().0;dispatcher.dispatch(&world);
        assert_eq!(world.read_storage::<Pos>().get(unit).unwrap().0,before);
        assert!(world.read_storage::<MoveTarget>().get(unit).is_none());
    }
    /// E412: `issue_order` reaches a summon: an attack order chases the target into attack range
    /// past the follow rules, a move order moves it, a stop clears both.
    #[test]
    fn a_summon_takes_issued_attack_move_and_stop_orders() {
        use crate::runtime::native::hero_command::{self,Command,OrderKind,OrderedAttack};
        let (mut world,_,unit,mut dispatcher)=fixture();
        world.write_storage::<TAttack>().insert(unit,TAttack::new(Fixed64::ONE,Fixed64::ONE,Fixed64::from_i32(100),Fixed64::ZERO)).unwrap();
        let enemy=world.create_entity().with(Pos(omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(600))))
            .with(CProperty {hp:Fixed64::ONE,mhp:Fixed64::ONE,msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
        let issue=|world:&mut World,command:Command|assert!(hero_command::apply(world,unit,hero_command::BUFF,&command.payload()));
        issue(&mut world,Command::OrderAttack {target:(enemy.id(),enemy.gen().id())});
        assert!(world.read_resource::<crate::runtime::BuffStore>().has(unit,omb_script_abi::buff_ids::SUMMON_TARGET_BUFF));
        for _ in 0..200 {dispatcher.dispatch(&world);}
        let at=world.read_storage::<Pos>().get(unit).unwrap().0;
        let gap=(omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(600))-at).length();
        assert!(gap<=Fixed64::from_i32(100) && gap>=Fixed64::from_i32(90),"stands in attack range: {gap:?}");
        assert!(world.read_storage::<SummonedUnit>().get(unit).unwrap().navigation.auto_target.is_none(),"the follow rules wait");
        world.write_storage::<CProperty>().get_mut(enemy).unwrap().hp=Fixed64::ZERO;
        dispatcher.dispatch(&world);
        assert!(world.read_storage::<OrderedAttack>().get(unit).is_none(),"a dead target ends the order");
        let point=omoba_sim::Vec2::new(Fixed64::from_i32(-300),Fixed64::from_i32(600));
        issue(&mut world,Command::Order {kind:OrderKind::Move,at:(point.x.raw() as i32,point.y.raw() as i32)});
        assert_eq!(world.read_storage::<MoveTarget>().get(unit).map(|m|m.0),Some(point));
        assert!(!world.read_resource::<crate::runtime::BuffStore>().has(unit,omb_script_abi::buff_ids::SUMMON_TARGET_BUFF));
        issue(&mut world,Command::Order {kind:OrderKind::Stop,at:(0,0)});
        assert!(world.read_storage::<MoveTarget>().get(unit).is_none());
        // E489: a follow order walks to the leader, keeps following it as it moves, and a stop ends it.
        let leader=world.create_entity().with(Pos(omoba_sim::Vec2::new(Fixed64::from_i32(800),Fixed64::ZERO)))
            .with(CProperty {hp:Fixed64::ONE,mhp:Fixed64::ONE,msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
        issue(&mut world,Command::OrderFollow {target:(leader.id(),leader.gen().id())});
        let reach=|world:&World|(world.read_storage::<Pos>().get(leader).unwrap().0-world.read_storage::<Pos>().get(unit).unwrap().0).length();
        for _ in 0..400 {dispatcher.dispatch(&world);}
        assert!(reach(&world)<=Fixed64::from_i32(hero_command::FOLLOW_GAP)+world.read_storage::<CollisionRadius>().get(unit).map_or(Fixed64::ZERO,|r|r.0)
            +Fixed64::from_i32(20),"reached the leader: {:?}",reach(&world));
        assert!(world.read_storage::<hero_command::OrderedFollow>().get(unit).is_some(),"the order stays");
        world.write_storage::<Pos>().get_mut(leader).unwrap().0=omoba_sim::Vec2::new(Fixed64::from_i32(800),Fixed64::from_i32(900));
        dispatcher.dispatch(&world);
        assert_eq!(world.read_storage::<MoveTarget>().get(unit).map(|m|m.0),Some(omoba_sim::Vec2::new(Fixed64::from_i32(800),Fixed64::from_i32(900))),
            "it follows the leader as it moves");
        issue(&mut world,Command::Order {kind:OrderKind::Stop,at:(0,0)});
        assert!(world.read_storage::<hero_command::OrderedFollow>().get(unit).is_none() && world.read_storage::<MoveTarget>().get(unit).is_none());
        // E495: a queued attack is an attack order for an idle summon, and ignored by a busy one.
        let queued=Command::OrderAttackQueued {target:(leader.id(),leader.gen().id())};
        issue(&mut world,queued.clone());
        assert_eq!(world.read_storage::<hero_command::OrderedAttack>().get(unit).map(|o|o.0),Some(leader));
        issue(&mut world,Command::Order {kind:OrderKind::Move,at:(0,0)});
        assert!(!hero_command::apply(&mut world,unit,hero_command::BUFF,&queued.payload()),"a moving summon keeps its move");
        assert!(world.read_storage::<hero_command::OrderedAttack>().get(unit).is_none());
        issue(&mut world,Command::Order {kind:OrderKind::Stop,at:(0,0)});
        // An immobile summon takes no move.
        world.write_storage::<SummonedUnit>().get_mut(unit).unwrap().immobile=true;
        assert!(!hero_command::apply(&mut world,unit,hero_command::BUFF,&Command::Order {kind:OrderKind::Move,at:(0,0)}.payload()));
    }

    #[test]
    fn summon_command_navigation_detours_and_expiry_retires_even_while_returning() {
        let (mut world,_,unit,mut dispatcher)=fixture();
        world.write_resource::<BlockedRegions>().0.push(BlockedRegion {name:"follow-wall".into(),
            points:vec![vek::Vec2::new(80.0,-30.0),vek::Vec2::new(81.0,-30.0),
                vek::Vec2::new(81.0,30.0),vek::Vec2::new(80.0,30.0)]});
        let mut detoured=false;
        for _ in 0..180 {
            let before=world.read_storage::<Pos>().get(unit).unwrap().0;dispatcher.dispatch(&world);
            let next=world.read_storage::<Pos>().get(unit).unwrap().0;
            assert!(!super::super::hero_move_tick::path_hits_regions(before,next,Fixed64::from_i32(10),&world.read_resource::<BlockedRegions>()));
            detoured|=next.y!=Fixed64::ZERO;
        }
        assert!(detoured);assert!(world.read_storage::<MoveTarget>().get(unit).is_none());
        // E397: a flyer goes straight through the wall it walked around.
        world.write_storage::<Pos>().get_mut(unit).unwrap().0=omoba_sim::Vec2::ZERO;
        world.write_resource::<crate::runtime::BuffStore>().add(unit,crate::runtime::native::rules::vocab::state::FLYING,
            Fixed64::from_i32(30),serde_json::Value::Null);
        let mut crossed=false;
        for _ in 0..30 {
            let before=world.read_storage::<Pos>().get(unit).unwrap().0;dispatcher.dispatch(&world);
            let next=world.read_storage::<Pos>().get(unit).unwrap().0;
            assert_eq!(next.y,Fixed64::ZERO,"no detour");
            crossed|=super::super::hero_move_tick::path_hits_regions(before,next,Fixed64::from_i32(10),&world.read_resource::<BlockedRegions>());
        }
        assert!(crossed,"through the wall");
        world.write_storage::<SummonedUnit>().get_mut(unit).unwrap().time_remaining=Some(Fixed64::from_raw(1));
        dispatcher.dispatch(&world);
        assert!(world.read_resource::<Vec<Outcome>>().iter().any(|o|matches!(o,Outcome::EntityRemoved {entity} if *entity==unit)));
    }
}
