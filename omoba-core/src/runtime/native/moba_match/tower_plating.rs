//! Per-life outer-tower threshold ledger; healing and attribution never rewind it.
use super::*;
use crate::runtime::BuffStore;

const BUFF:&str="tower_plating";
const STATUS:&str="tower_plating_status";

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct TowerPlatingRules {
    pub plates:u32,
    pub expires_seconds:u32,
    pub gold_per_plate:u32,
    pub local_radius:u32,
    pub armor_per_plate:u32,
}
impl TowerPlatingRules {
    /// Original project balance, independently configurable from destruction bounties.
    pub fn training()->Self {Self {plates:5,expires_seconds:840,gold_per_plate:125,local_radius:1200,armor_per_plate:20}}
    pub fn is_valid(self)->bool {(1..=16).contains(&self.plates) && (1..=3600).contains(&self.expires_seconds)
        && self.gold_per_plate<=1_000_000 && (1..=10_000).contains(&self.local_radius) && self.armor_per_plate<=1000}
}

#[derive(Clone,Debug)]
pub(super) struct PlateLedger {entity:Entity,max_hp_raw:i64,lost:u32}

pub(super) fn record(world:&World,state:&mut MobaMatch,source:Option<Entity>,target:Entity) {
    let Some(rule)=state.config.tower_plating_rules else {return};
    if state.phase!=MobaMatchPhase::Playing || !rule.is_valid() || state.elapsed<Fixed64::ZERO
        || state.elapsed>=Fixed64::from_i32(rule.expires_seconds as i32) || world.read_resource::<GamePause>().is_paused {return;}
    let Some(unit)=state.units.get(&entity_key(target)).filter(|u|u.role==LaneRole::Tower && !u.last_hit_retired) else {return};
    let victim_side=usize::from(unit.team);
    if state.lane_tower_layers.get(unit.lane).and_then(|layers|layers.first()).and_then(|layer|layer[victim_side])!=Some(target) {return;}
    let Some((hp,max_hp))=world.read_storage::<CProperty>().get(target).map(|p|(p.hp.raw(),p.mhp.raw())).filter(|(_,max)|*max>0) else {return};
    let key=entity_key(target);
    let ledger=state.tower_plates.entry(key).or_insert(PlateLedger {entity:target,max_hp_raw:max_hp,lost:0});
    let lost=((i128::from(ledger.max_hp_raw)-i128::from(hp.max(0))).max(0)*i128::from(rule.plates)
        /i128::from(ledger.max_hp_raw)).min(i128::from(rule.plates)) as u32;
    let newly_lost=lost.saturating_sub(ledger.lost);ledger.lost=ledger.lost.max(lost);
    // Retire thresholds even if the source is anonymous, allied, or has no nearby recipient.
    if newly_lost>0 {
        let credit=jungle_credit::resolve(world,state,source).filter(|c|c.side!=victim_side);
        if let Some(credit)=credit {
            let positions=world.read_storage::<Pos>();let health=world.read_storage::<CProperty>();let origin=positions.get(target).map(|p|p.0);
            let radius=Fixed64::from_i32(rule.local_radius as i32);
            let local:BTreeSet<u32>=state.heroes.iter().filter(|s|s.side==credit.side && !s.lethal_pending)
                .filter_map(|s|s.entity.filter(|e|world.entities().is_alive(*e) && health.get(*e).is_some_and(|p|p.hp>Fixed64::ZERO)
                    && origin.zip(positions.get(*e)).is_some_and(|(a,b)|(a-b.0).length_squared()<=radius*radius)).map(|_|s.player_id)).collect();
            drop(positions);drop(health);
            let count=local.len() as u32;
            let pool=if count==0 {0}else {(rule.gold_per_plate/count)*count*newly_lost};
            building_rewards::credit(world,state,credit.side,&local,BuildingBounty {team_gold:0,local_gold_pool:pool});
        }
    }
    synchronize(world,state,Fixed64::ZERO);
}

pub(super) fn synchronize(world:&World,state:&MobaMatch,pending_tick:Fixed64) {
    let active=state.config.tower_plating_rules.filter(|r|r.is_valid() && state.phase==MobaMatchPhase::Playing
        && state.elapsed>=Fixed64::ZERO && state.elapsed<Fixed64::from_i32(r.expires_seconds as i32));
    let health=world.read_storage::<CProperty>();let mut buffs=world.write_resource::<BuffStore>();
    for ledger in state.tower_plates.values() {
        let armor=active.filter(|_|state.units.get(&entity_key(ledger.entity)).is_some_and(|u|!u.last_hit_retired)
            && world.entities().is_alive(ledger.entity) && health.get(ledger.entity).is_some_and(|p|p.hp>Fixed64::ZERO))
            .map_or(0,|r|r.armor_per_plate*ledger.lost);
        if armor==0 {buffs.remove(ledger.entity,BUFF);continue;}
        let old=buffs.iter_for(ledger.entity).find(|(id,_)|*id==BUFF).and_then(|(_,b)|b.payload["physical_armor_bonus"].as_i64());
        let raw=i64::from(armor)*1024;
        let remaining=active.map_or(Fixed64::ZERO,|r|Fixed64::from_i32(r.expires_seconds as i32)-state.elapsed+pending_tick);
        if old!=Some(raw) {buffs.remove(ledger.entity,BUFF);buffs.add(ledger.entity,BUFF,remaining,
            serde_json::json!({"physical_armor_bonus":raw,"stacks":ledger.lost,"__aggregation_family":BUFF}));}
    }
    // Public state exists before any plate is broken and has no combat modifiers.
    let outer:BTreeSet<Entity>=state.lane_tower_layers.iter().filter_map(|layers|layers.first()).flat_map(|layer|layer.iter().flatten().copied()).collect();
    for entity in outer.iter().copied().chain(state.tower_plates.values().map(|p|p.entity)) {
        let rule=active.filter(|_|outer.contains(&entity) && world.entities().is_alive(entity)
            && state.units.get(&entity_key(entity)).is_some_and(|u|!u.last_hit_retired)
            && health.get(entity).is_some_and(|p|p.hp>Fixed64::ZERO));
        let Some(rule)=rule else {buffs.remove(entity,STATUS);continue;};
        let remaining=rule.plates-state.tower_plates.get(&entity_key(entity)).map_or(0,|p|p.lost.min(rule.plates));
        let correct=buffs.iter_for(entity).find(|(id,_)|*id==STATUS).is_some_and(|(_,b)|
            b.payload["health_segments"]["total"].as_u64()==Some(u64::from(rule.plates))
            && b.payload["health_segments"]["remaining"].as_u64()==Some(u64::from(remaining)));
        if !correct {buffs.remove(entity,STATUS);buffs.add(entity,STATUS,
            Fixed64::from_i32(rule.expires_seconds as i32)-state.elapsed+pending_tick,
            serde_json::json!({"health_segments":{"total":rule.plates,"remaining":remaining}}));}
    }
}

pub(super) fn retire(world:&World,state:&MobaMatch,entity:Entity) {
    world.write_resource::<BuffStore>().remove(entity,STATUS);
    if state.tower_plates.contains_key(&entity_key(entity)) {world.write_resource::<BuffStore>().remove(entity,BUFF);}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{process_outcomes,RuntimeEventVecSink,TowerTemplateRegistry,UnitStats};
    use specs::Join;
    fn fixture()->(World,Entity,Entity,Entity) {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);world.insert(TowerTemplateRegistry::default());
        setup_single_lane_match(&mut world,SingleLaneConfig {tower_hp:Fixed64::from_i32(1000),tower_plating_rules:Some(TowerPlatingRules::training()),
            additional_players:vec![SingleLanePlayerConfig {player_id:3,team_id:1,hero:"training_luminary".into()}],..Default::default()}).unwrap();
        let (target,hero,ally)={let mut state=world.write_resource::<MobaMatch>();state.phase=MobaMatchPhase::Playing;
            (state.towers[1].unwrap(),state.heroes[0].entity.unwrap(),state.heroes[2].entity.unwrap())};
        for p in (&mut world.write_storage::<Pos>()).join(){p.0=Vec2::new(Fixed64::from_i32(50_000),Fixed64::ZERO);}
        for e in [target,hero]{world.write_storage::<Pos>().get_mut(e).unwrap().0=Vec2::ZERO;}
        world.write_storage::<Pos>().get_mut(ally).unwrap().0=Vec2::new(Fixed64::from_i32(1200),Fixed64::ZERO);
        for gold in (&mut world.write_storage::<Gold>()).join(){gold.0=0;}
        (world,target,hero,ally)
    }
    fn hit(world:&mut World,target:Entity,source:Option<Entity>,amount:i32) {
        let outcome=source.map_or(Outcome::ScriptDirectDamage {target,amount:Fixed64::from_i32(amount)},|source|
            Outcome::Damage {source,target,pos:Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,
                real:Fixed64::from_i32(amount),damage_profile:0,predeclared:false});
        world.write_resource::<Vec<Outcome>>().push(outcome);
        process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();
    }
    fn gold(world:&World,e:Entity)->i32 {world.read_storage::<Gold>().get(e).unwrap().0}
    #[test]
    fn public_plate_state_exists_unhit_survives_healing_and_retires_without_combat_effects() {
        let (mut w,target,hero,_)=fixture();synchronize(&w,&w.read_resource::<MobaMatch>(),Fixed64::ZERO);
        let count=|world:&World|world.read_resource::<BuffStore>().iter_for(target).find(|(id,_)|*id==STATUS)
            .map(|(_,b)|b.payload["health_segments"]["remaining"].as_u64().unwrap());
        assert_eq!(count(&w),Some(5));assert!(!w.read_resource::<BuffStore>().has(target,BUFF));
        hit(&mut w,target,Some(hero),400);assert_eq!(count(&w),Some(3));
        w.write_storage::<CProperty>().get_mut(target).unwrap().hp=Fixed64::from_i32(1000);
        synchronize(&w,&w.read_resource::<MobaMatch>(),Fixed64::ZERO);assert_eq!(count(&w),Some(3));
        let armor=UnitStats::from_refs(&w.read_resource::<BuffStore>(),false).final_armor(Fixed64::ZERO,target);
        assert_eq!(armor,Fixed64::from_i32(40));
        w.write_resource::<MobaMatch>().elapsed=Fixed64::from_i32(840);synchronize(&w,&w.read_resource::<MobaMatch>(),Fixed64::ZERO);
        assert_eq!(count(&w),None);
        let (mut w,target,hero,_)=fixture();hit(&mut w,target,Some(hero),200);record_moba_death(&mut w,target);
        assert!(!w.read_resource::<BuffStore>().has(target,STATUS));
    }
    #[test]
    fn committed_thresholds_healing_and_lethal_duplicates_never_reclaim_plates() {
        let (mut w,target,hero,ally)=fixture();hit(&mut w,target,Some(hero),200);
        assert_eq!(gold(&w,hero),62);assert_eq!(gold(&w,ally),62);
        w.write_storage::<CProperty>().get_mut(target).unwrap().hp=Fixed64::from_i32(900);
        hit(&mut w,target,Some(hero),100);assert_eq!(gold(&w,hero),62);
        hit(&mut w,target,Some(hero),400);assert_eq!(gold(&w,hero),186);
        hit(&mut w,target,Some(hero),100_000);assert_eq!(gold(&w,hero),310);assert_eq!(gold(&w,ally),310);
        hit(&mut w,target,Some(hero),100_000);assert_eq!(gold(&w,hero),310);
        assert_eq!(w.read_resource::<MobaMatch>().tower_plates[&entity_key(target)].lost,5);
        assert!(!w.read_resource::<BuffStore>().has(target,BUFF));
    }
    #[test]
    fn attribution_scope_and_independent_rounding_are_packet_grouping_invariant() {
        let (mut a,target,hero,_)=fixture();hit(&mut a,target,Some(hero),600);let merged=gold(&a,hero);
        let (mut w,target,hero,_)=fixture();for _ in 0..3{hit(&mut w,target,Some(hero),200);}
        assert_eq!(gold(&w,hero),merged);assert_eq!(merged,186);
        let (mut w,target,hero,ally2)=fixture();hit(&mut w,target,None,200);assert_eq!(gold(&w,hero),0);
        hit(&mut w,target,Some(hero),200);assert_eq!(gold(&w,hero),62);
        let defender=w.read_resource::<MobaMatch>().heroes[1].entity.unwrap();hit(&mut w,target,Some(defender),200);
        assert_eq!(gold(&w,hero),62);hit(&mut w,target,Some(hero),200);assert_eq!(gold(&w,ally2),124);
        w.write_storage::<Pos>().get_mut(ally2).unwrap().0=Vec2::new(Fixed64::from_i32(1201),Fixed64::ZERO);
        hit(&mut w,target,Some(hero),1000);assert_eq!(gold(&w,hero),249);assert_eq!(gold(&w,ally2),124);
        let (mut w,target,hero,_)=fixture();w.write_resource::<MobaMatch>().lane_tower_layers[0]=vec![[None,None],[None,Some(target)]];
        hit(&mut w,target,Some(hero),200);assert_eq!(gold(&w,hero),0);assert!(w.read_resource::<MobaMatch>().tower_plates.is_empty());
    }
    #[test]
    fn cutoff_pause_defense_and_forced_retirement_respect_authority_lifecycle() {
        let (mut w,target,hero,_)=fixture();hit(&mut w,target,Some(hero),400);
        assert_eq!(UnitStats::from_refs(&w.read_resource::<BuffStore>(),false).final_armor(Fixed64::from_i32(50),target),Fixed64::from_i32(90));
        w.write_resource::<GamePause>().is_paused=true;
        let mut state=w.remove::<MobaMatch>().unwrap();let before=state.tower_plates[&entity_key(target)].lost;
        record(&w,&mut state,Some(hero),target);assert_eq!(state.tower_plates[&entity_key(target)].lost,before);w.insert(state);
        w.write_resource::<GamePause>().is_paused=false;w.write_resource::<MobaMatch>().elapsed=Fixed64::from_i32(840);
        synchronize(&w,&w.read_resource::<MobaMatch>(),Fixed64::ZERO);assert!(!w.read_resource::<BuffStore>().has(target,BUFF));
        let balance=gold(&w,hero);hit(&mut w,target,Some(hero),200);assert_eq!(gold(&w,hero),balance);
        let (mut w,target,hero,_)=fixture();hit(&mut w,target,Some(hero),200);record_moba_death(&mut w,target);
        synchronize(&w,&w.read_resource::<MobaMatch>(),Fixed64::ZERO);assert!(!w.read_resource::<BuffStore>().has(target,BUFF));
        let (mut w,target,hero,_)=fixture();hit(&mut w,target,Some(hero),200);
        w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Finished {winner:Some(0),tick:2};
        synchronize(&w,&w.read_resource::<MobaMatch>(),Fixed64::ZERO);assert!(!w.read_resource::<BuffStore>().has(target,BUFF));
    }
    #[test]
    fn invalid_rules_are_atomic_and_ledger_configuration_change_the_digest() {
        assert!(SingleLaneConfig::default().tower_plating_rules.is_none());
        for case in 0..8 {
            let mut rule=TowerPlatingRules::training();match case {0=>rule.plates=0,1=>rule.plates=17,2=>rule.expires_seconds=0,
                3=>rule.expires_seconds=3601,4=>rule.gold_per_plate=1_000_001,5=>rule.local_radius=0,6=>rule.local_radius=10_001,_=>rule.armor_per_plate=1001}
            let pool=crate::runtime::StateInitializer::create_thread_pool();let mut w=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
            assert!(setup_single_lane_match(&mut w,SingleLaneConfig {tower_plating_rules:Some(rule),..Default::default()}).is_err());
            assert!(w.try_fetch::<MobaMatch>().is_none());assert_eq!(w.entities().join().count(),0);
        }
        let (mut w,target,hero,_)=fixture();let initial=single_lane_replay_digest(&w);hit(&mut w,target,Some(hero),200);
        assert_ne!(initial,single_lane_replay_digest(&w));let before=single_lane_replay_digest(&w);
        w.write_resource::<MobaMatch>().config.tower_plating_rules.as_mut().unwrap().gold_per_plate+=1;
        assert_ne!(before,single_lane_replay_digest(&w));
    }
}
