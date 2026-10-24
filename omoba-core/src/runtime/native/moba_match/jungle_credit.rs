//! Authority ownership resolution at the first lethal hit, never at input time.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct JungleCredit {
    pub side:usize,
    pub player:Option<u32>,
}

pub(super) fn living_target(world:&World,teams:&[u32;2],heroes:&[MobaHeroSlot],entity:Entity)->bool {
    world.entities().is_alive(entity)
        && world.read_storage::<CProperty>().get(entity).is_some_and(|p|p.hp>Fixed64::ZERO)
        && world.read_storage::<Faction>().get(entity).is_some_and(|f|teams.iter()
            .any(|team|i64::from(*team)==i64::from(f.team_id)))
        && !heroes.iter().any(|slot|slot.entity==Some(entity) && slot.lethal_pending)
}

pub(super) fn resolve(world:&World,state:&MobaMatch,source:Option<Entity>)->Option<JungleCredit> {
    let mut entity=source?;
    if let Some(origin)=world.try_fetch::<ProjectileImpactOrigin>() {
        if origin.0.source==entity {
            let team=origin.0.credit_team?;
            let side=state.config.teams.iter().position(|id|*id==team)?;
            let player=origin.0.credit_player;
            if player.is_some_and(|player|!state.heroes.iter().any(|slot|slot.player_id==player && slot.side==side)) {return None;}
            return Some(JungleCredit {side,player});
        }
    }
    let factions=world.read_storage::<Faction>();let owners=world.read_storage::<PlayerOwner>();
    let summons=world.read_storage::<SummonedUnit>();
    let team=factions.get(entity)?.team_id;
    let side=state.config.teams.iter().position(|id|i64::from(*id)==i64::from(team))?;
    let mut visited=[None;9];
    // Eight generation-aware summoner links, with no allocations or unbounded traversal.
    for depth in 0..=8 {
        if !world.entities().is_alive(entity) || visited.contains(&Some(entity))
            || factions.get(entity).is_none_or(|f|f.team_id!=team) {return None;}
        visited[depth]=Some(entity);
        if let Some(owner)=owners.get(entity) {
            return state.heroes.iter().any(|slot|slot.player_id==owner.player_id && slot.side==side)
                .then_some(JungleCredit {side,player:Some(owner.player_id)});
        }
        match summons.get(entity) {
            Some(summon) if depth<8=>entity=summon.summoner,
            Some(_)=>return None,
            None=>return Some(JungleCredit {side,player:None}),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{process_outcomes,RuntimeEventVecSink,TowerTemplateRegistry,BuffStore};

    fn fixture()->(World,Entity,Entity) {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        world.insert(TowerTemplateRegistry::default());let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        setup_single_lane_match(&mut world,SingleLaneConfig {map_id:Some(map.id.into()),lane_length:Fixed64::from_i32(map.lane_length),
            additional_players:vec![SingleLanePlayerConfig {player_id:3,team_id:1,hero:"training_luminary".into()}],..Default::default()}).unwrap();
        let (hero,ally)={let mut state=world.write_resource::<MobaMatch>();state.phase=MobaMatchPhase::Playing;
            (state.heroes[0].entity.unwrap(),state.heroes[2].entity.unwrap())};
        for entity in [hero,ally] {world.write_storage::<Gold>().get_mut(entity).unwrap().0=0;}
        (world,hero,ally)
    }
    fn npc(world:&mut World,player:Option<u32>,parent:Option<Entity>)->Entity {
        let entity=world.create_entity().with(Unit::new("credit_fixture".into(),"credit".into(),UnitType::Creep))
            .with(Faction::new(FactionType::Ally,1)).with(property(Fixed64::from_i32(100),Fixed64::ZERO))
            .with(Pos(Vec2::ZERO)).build();
        if let Some(player)=player {world.write_storage::<PlayerOwner>().insert(entity,PlayerOwner::new(player)).unwrap();}
        if let Some(parent)=parent {world.write_storage::<SummonedUnit>().insert(entity,SummonedUnit::new(parent,None,0.0)).unwrap();}
        entity
    }
    fn objective(world:&World)->Entity {
        world.read_resource::<MobaMatch>().jungle_camps.iter().find(|c|c.definition.objective_buff
            .is_some_and(|b|!b.lost_on_death)).unwrap().entity.unwrap()
    }
    fn damage(source:Entity,target:Entity,amount:i32)->Outcome {
        Outcome::Damage {source,target,pos:Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,
            real:Fixed64::from_i32(amount),damage_profile:0,predeclared:false}
    }
    fn flush(world:&mut World,events:Vec<Outcome>) {
        world.write_resource::<Vec<Outcome>>().extend(events);
        process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();
        process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();
    }
    #[test]
    fn owned_chain_is_bounded_generation_aware_and_rejects_cycles_and_mismatched_owners() {
        let (mut world,hero,_)=fixture();let direct=npc(&mut world,Some(1),None);
        assert_eq!(resolve(&world,&world.read_resource::<MobaMatch>(),Some(direct)),Some(JungleCredit {side:0,player:Some(1)}));
        let mut chain=hero;
        for _ in 0..8 {chain=npc(&mut world,None,Some(chain));}
        assert_eq!(resolve(&world,&world.read_resource::<MobaMatch>(),Some(chain)),Some(JungleCredit {side:0,player:Some(1)}));
        let too_deep=npc(&mut world,None,Some(chain));assert_eq!(resolve(&world,&world.read_resource::<MobaMatch>(),Some(too_deep)),None);
        let cycle=npc(&mut world,None,None);world.write_storage::<SummonedUnit>().insert(cycle,SummonedUnit::new(cycle,None,0.0)).unwrap();
        assert_eq!(resolve(&world,&world.read_resource::<MobaMatch>(),Some(cycle)),None);
        for bad in [2,999] {let source=npc(&mut world,Some(bad),None);assert_eq!(resolve(&world,&world.read_resource::<MobaMatch>(),Some(source)),None);}
        let parent=npc(&mut world,Some(1),None);let child=npc(&mut world,None,Some(parent));world.delete_entity(parent).unwrap();world.maintain();
        let _replacement=npc(&mut world,Some(1),None);assert_eq!(resolve(&world,&world.read_resource::<MobaMatch>(),Some(child)),None);
    }
    #[test]
    fn actual_owned_npc_kill_pays_dead_owner_and_restores_persistent_objective_on_respawn() {
        let (mut world,hero,ally)=fixture();let source=npc(&mut world,Some(1),None);let target=objective(&world);
        record_moba_death(&mut world,hero);flush(&mut world,vec![damage(source,target,100_000)]);
        assert_eq!(world.read_resource::<MobaMatch>().heroes[0].gold.0,150);
        assert_eq!((world.read_resource::<MobaMatch>().heroes[0].hero.level,world.read_resource::<MobaMatch>().heroes[0].hero.experience),(3,80));
        assert!(world.read_resource::<BuffStore>().has(ally,"objective_guard"));
        let mut state=world.remove::<MobaMatch>().unwrap();state.elapsed=Fixed64::from_i32(30);state.spawn_hero(&mut world,0);
        let next=state.heroes[0].entity.unwrap();objective_buffs::synchronize(&world,&mut state,Fixed64::ZERO);world.insert(state);
        assert_eq!(world.read_storage::<Gold>().get(next).unwrap().0,150);
        assert_eq!((world.read_storage::<Hero>().get(next).unwrap().level,world.read_storage::<Hero>().get(next).unwrap().experience),(3,80));
        assert!(world.read_resource::<BuffStore>().has(next,"objective_guard"));
    }
    #[test]
    fn source_retirement_after_first_lethal_preserves_credit_and_duplicate_death_is_harmless() {
        let (mut world,hero,ally)=fixture();let source=npc(&mut world,None,Some(hero));let target=objective(&world);
        world.write_resource::<Vec<Outcome>>().push(damage(source,target,100_000));
        process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
        world.delete_entity(source).unwrap();world.maintain();
        flush(&mut world,Vec::new());assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0,150);
        assert!(world.read_resource::<BuffStore>().has(ally,"objective_guard"));
        flush(&mut world,vec![Outcome::Death {ent:target,pos:Vec2::ZERO}]);
        assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0,150);
    }
    #[test]
    fn unowned_team_units_grant_only_team_effect_and_invalid_first_lethal_cannot_be_reclaimed() {
        let (mut world,hero,ally)=fixture();let source=world.read_resource::<MobaMatch>().towers[0].unwrap();let target=objective(&world);
        flush(&mut world,vec![damage(source,target,100_000)]);
        assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0,0);
        assert!(world.read_resource::<BuffStore>().has(ally,"objective_guard"));
        for mode in 0..3 {
            let (mut world,hero,ally)=fixture();let source=npc(&mut world,Some(if mode==0 {999}else{1}),None);
            if mode==1 {world.write_storage::<Faction>().get_mut(source).unwrap().team_id=0;}
            let target=objective(&world);let first=if mode==2 {Outcome::ScriptDirectDamage {target,amount:Fixed64::from_i32(100_000)}}else{damage(source,target,100_000)};
            flush(&mut world,vec![first,Outcome::Heal {target,pos:Vec2::ZERO,amount:Fixed64::from_i32(100)},damage(hero,target,100_000)]);
            assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0,0);assert!(!world.read_resource::<BuffStore>().has(ally,"objective_guard"));
        }
    }
    #[test]
    fn actual_camp_aggro_attacks_living_nonhero_and_returns_after_target_retirement() {
        let (mut world,_,_)=fixture();let source=npc(&mut world,None,None);let target=objective(&world);
        let home=world.read_storage::<Pos>().get(target).unwrap().0;world.write_storage::<Pos>().get_mut(source).unwrap().0=home;
        flush(&mut world,vec![damage(source,target,1)]);
        let mut state=world.remove::<MobaMatch>().unwrap();world.write_resource::<Vec<DamageInstance>>().clear();
        jungle::tick(&mut world,&mut state,Fixed64::ONE);
        assert!(world.read_resource::<Vec<DamageInstance>>().iter().any(|d|d.source.source_entity==target && d.target==source));
        world.delete_entity(source).unwrap();world.maintain();world.write_resource::<Vec<DamageInstance>>().clear();
        jungle::tick(&mut world,&mut state,Fixed64::ONE);
        assert!(world.read_resource::<Vec<DamageInstance>>().iter().all(|d|d.target!=source));
        assert!(state.jungle_camps.iter().find(|c|c.entity==Some(target)).unwrap().returning);
    }
}
