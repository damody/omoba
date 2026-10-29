//! Authority damage settlement, bounded by the tracked structure's lifetime.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BuildingBounty {
    /// Credited independently to every authenticated player on the attacking side.
    pub team_gold: u32,
    /// Equal floor shares for living nearby heroes; any remainder is discarded.
    pub local_gold_pool: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildingRewardRules {
    pub tower: BuildingBounty,
    /// Outer-to-inner overrides; absent depths use `tower`.
    pub tower_layers: [Option<BuildingBounty>;8],
    /// One additional bounty for the first lane tower across the whole match.
    pub first_tower: BuildingBounty,
    pub inhibitor: BuildingBounty,
    pub nexus_tower: BuildingBounty,
    pub base: BuildingBounty,
    pub local_radius: u32,
}

impl Default for BuildingRewardRules {
    fn default() -> Self {
        Self {tower:BuildingBounty::default(),tower_layers:[None;8],first_tower:BuildingBounty::default(),inhibitor:BuildingBounty::default(),
            nexus_tower:BuildingBounty::default(),base:BuildingBounty::default(),local_radius:1200}
    }
}

impl BuildingRewardRules {
    /// Project balance for formal MOBA. Legacy fixtures remain explicitly disabled.
    /// These values are not a claim about a particular League patch.
    pub fn training() -> Self {
        Self {tower:BuildingBounty {team_gold:100,local_gold_pool:250},
            tower_layers:[None,Some(BuildingBounty {team_gold:125,local_gold_pool:300}),
                Some(BuildingBounty {team_gold:150,local_gold_pool:350}),None,None,None,None,None],
            first_tower:BuildingBounty {team_gold:0,local_gold_pool:150},
            inhibitor:BuildingBounty {team_gold:25,local_gold_pool:100},
            nexus_tower:BuildingBounty {team_gold:50,local_gold_pool:150},..Self::default()}
    }

    pub fn is_valid(self) -> bool {
        (1..=10_000).contains(&self.local_radius)
            && [self.tower,self.inhibitor,self.nexus_tower,self.base,self.first_tower].iter()
                .chain(self.tower_layers.iter().flatten())
                .all(|b|b.team_gold<=1_000_000 && b.local_gold_pool<=1_000_000)
    }

    fn bounty(self,role:LaneRole,layer:Option<usize>)->Option<BuildingBounty> {
        match role {LaneRole::Tower=>Some(layer.and_then(|depth|self.tower_layers.get(depth).copied().flatten()).unwrap_or(self.tower)),LaneRole::Inhibitor=>Some(self.inhibitor),
            LaneRole::NexusTower=>Some(self.nexus_tower),LaneRole::Base=>Some(self.base),_=>None}
    }
}

/// Called only after positive committed damage by both native and script paths.
/// Retire before attribution, so anonymous damage, healing, or repeated outcomes
/// cannot manufacture a second payout. Forced retirement alone never pays.
pub(crate) fn record_moba_building_damage(world:&World,source:Option<Entity>,target:Entity,lethal:bool) {
    let Some(mut state)=world.try_fetch_mut::<MobaMatch>() else {return};
    if state.phase!=MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused {return;}
    tower_plating::record(world,&mut state,source,target);
    if !lethal {return;}
    let Some(unit)=state.units.get(&entity_key(target)) else {return};
    let is_tower=unit.role==LaneRole::Tower;
    let notice_kind=match unit.role {LaneRole::Tower=>1,LaneRole::Inhibitor=>2,LaneRole::NexusTower=>3,_=>0};
    let layer=state.tower_layer_of(unit.lane,usize::from(unit.team),target);
    let Some(bounty)=state.config.building_reward_rules.bounty(unit.role,layer) else {return};
    if unit.last_hit_retired {return;}
    let role=unit.role;
    let victim_side=usize::from(unit.team);
    state.units.get_mut(&entity_key(target)).expect("tracked structure").last_hit_retired=true;
    let credited=jungle_credit::resolve(world,&state,source).filter(|credit|credit.side!=victim_side);
    let side=credited.map(|credit|credit.side);
    let Some(side)=side else {
        if is_tower {state.first_tower.observe(world.read_resource::<Tick>().0,entity_key(target),None,BTreeSet::new());}
        return;
    };
    let positions=world.read_storage::<Pos>();
    let properties=world.read_storage::<CProperty>();
    let origin=positions.get(target).map(|p|p.0);
    let radius=Fixed64::from_i32(state.config.building_reward_rules.local_radius as i32);
    let local: BTreeSet<_>=state.heroes.iter().filter(|slot|slot.side==side && !slot.lethal_pending)
        .filter_map(|slot|slot.entity.filter(|entity|world.entities().is_alive(*entity)
            && properties.get(*entity).is_some_and(|p|p.hp>Fixed64::ZERO)
            && origin.zip(positions.get(*entity)).is_some_and(|(origin,p)|(p.0-origin).length_squared()<=radius*radius))
            .map(|_|slot.player_id)).collect();
    drop(positions);drop(properties);
    if is_tower {state.first_tower.observe(world.read_resource::<Tick>().0,entity_key(target),Some(side),local.clone());}
    if is_tower {objective_comeback::observe(world,&mut state,target,side,false);}
    objective_notice::observe(world,&mut state,target,side,notice_kind);
    // Original rules (E187): the structure's own bounty (game_rules.lua buildings), not the
    // training team gold and local pool.
    if let Some(rules)=state.config.game_rules {
        original_rewards::structure_bounty(world,&mut state,&rules,side,credited.and_then(|credit|credit.player),target,role);
        return;
    }
    credit(world,&mut state,side,&local,bounty);
}

pub(super) fn credit(world:&World,state:&mut MobaMatch,side:usize,local:&BTreeSet<u32>,bounty:BuildingBounty) {
    let share=if local.is_empty(){0}else{bounty.local_gold_pool/local.len() as u32};
    for index in 0..state.heroes.len() {
        let slot=&state.heroes[index];
        if slot.side!=side {continue;}
        let amount=bounty.team_gold+if local.contains(&slot.player_id){share}else{0};
        credit_moba_player(world,state,index,amount,0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{process_outcomes,RuntimeEventVecSink,TowerTemplateRegistry};
    use specs::Join;

    fn fixture()->(World,Entity,Entity,Entity) {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        world.insert(TowerTemplateRegistry::default());
        setup_single_lane_match(&mut world,SingleLaneConfig {warmup:Fixed64::ZERO,
            building_reward_rules:BuildingRewardRules::training(),
            additional_players:vec![SingleLanePlayerConfig {player_id:3,team_id:1,hero:"training_luminary".into()},
                SingleLanePlayerConfig {player_id:5,team_id:1,hero:"training_luminary".into()}],..Default::default()}).unwrap();
        let (target,source,helper)={let mut state=world.write_resource::<MobaMatch>();
            state.phase=MobaMatchPhase::Playing;
            (state.towers[1].unwrap(),state.heroes[0].entity.unwrap(),state.heroes[2].entity.unwrap())};
        for gold in (&mut world.write_storage::<Gold>()).join() {gold.0=0;}
        for pos in (&mut world.write_storage::<Pos>()).join() {pos.0=Vec2::new(Fixed64::from_i32(50_000),Fixed64::ZERO);}
        world.write_storage::<Pos>().get_mut(target).unwrap().0=Vec2::ZERO;
        world.write_storage::<Pos>().get_mut(source).unwrap().0=Vec2::ZERO;
        world.write_storage::<Pos>().get_mut(helper).unwrap().0=Vec2::new(Fixed64::from_i32(1200),Fixed64::ZERO);
        (world,target,source,helper)
    }
    fn lethal(source:Entity,target:Entity)->Outcome {
        Outcome::Damage {pos:Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,
            real:Fixed64::from_i32(100_000),source,target,damage_profile:0,predeclared:false}
    }
    fn flush(world:&mut World,outcomes:impl IntoIterator<Item=Outcome>) {
        world.write_resource::<Vec<Outcome>>().extend(outcomes);
        process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();
    }
    fn balance(world:&World,player:u32)->i32 {
        let state=world.read_resource::<MobaMatch>();let slot=state.heroes.iter().find(|s|s.player_id==player).unwrap();
        slot.entity.map_or(slot.gold.0,|e|world.read_storage::<Gold>().get(e).unwrap().0)
    }
    #[test]
    fn committed_tower_damage_splits_at_boundary_pays_dead_wallet_and_never_double_pays() {
        let (mut world,target,source,helper)=fixture();
        let dead=world.read_resource::<MobaMatch>().heroes[3].entity.unwrap();
        record_moba_death(&mut world,dead);
        flush(&mut world,[lethal(source,target),Outcome::Heal {pos:Vec2::ZERO,target,amount:Fixed64::from_i32(500)},lethal(source,target)]);
        assert_eq!(balance(&world,1),225);assert_eq!(balance(&world,3),225);
        assert_eq!(balance(&world,5),100,"dead hero receives only global reward");
        assert_eq!(balance(&world,2),0,"defender receives none");
        assert_eq!(world.read_storage::<Gold>().get(helper).unwrap().0,225);
        assert!(world.read_resource::<MobaMatch>().heroes.iter().all(|s|s.kills==0 && s.assists==0));
        world.maintain();
        let mut state=world.remove::<MobaMatch>().unwrap();state.elapsed=Fixed64::from_i32(100);state.spawn_hero(&mut world,3);world.insert(state);
        assert_eq!(balance(&world,5),100,"reward survives hero replacement");
    }
    #[test]
    fn anonymous_friendly_neutral_and_forced_death_cannot_claim_later() {
        for mode in 0..4 {
            let (mut world,target,source,_)=fixture();
            let first=match mode {
                0=>Outcome::ScriptDirectDamage {target,amount:Fixed64::from_i32(100_000)},
                1=>lethal(world.read_resource::<MobaMatch>().heroes[1].entity.unwrap(),target),
                2=>{world.write_storage::<Faction>().get_mut(source).unwrap().team_id=0;lethal(source,target)},
                _=>Outcome::Death {ent:target,pos:Vec2::ZERO},
            };
            flush(&mut world,[first]);
            world.write_storage::<Faction>().get_mut(source).unwrap().team_id=1;
            flush(&mut world,[Outcome::Heal {pos:Vec2::ZERO,target,amount:Fixed64::from_i32(1000)},lethal(source,target)]);
            assert_eq!(balance(&world,1),0,"mode {mode}");
            assert_eq!(balance(&world,3),0);assert_eq!(balance(&world,5),0);
        }
    }
    #[test]
    fn npc_credit_distant_roster_floor_shares_and_saturation_are_deterministic() {
        let (mut world,target,source,helper)=fixture();
        let npc=world.read_resource::<MobaMatch>().towers[0].unwrap();
        world.write_resource::<MobaMatch>().config.building_reward_rules.tower.local_gold_pool=251;
        world.write_storage::<Gold>().get_mut(source).unwrap().0=i32::MAX-1;
        flush(&mut world,[lethal(npc,target)]);
        assert_eq!(balance(&world,1),i32::MAX);
        assert_eq!(balance(&world,3),225,"floor share, no roster-order remainder");
        assert_eq!(balance(&world,5),100,"outside radius still gets team gold");
        assert_eq!(world.read_storage::<Gold>().get(helper).unwrap().0,225);
    }
    #[test]
    fn phase_pause_nonlethal_and_all_roles_respect_policy() {
        for role in [LaneRole::Tower,LaneRole::Inhibitor,LaneRole::NexusTower,LaneRole::Base,LaneRole::Creep] {
            let (world,target,source,_)=fixture();
            world.write_resource::<MobaMatch>().units.get_mut(&entity_key(target)).unwrap().role=role;
            world.write_resource::<MobaMatch>().config.building_reward_rules.base=BuildingBounty {team_gold:10,local_gold_pool:8};
            world.write_resource::<GamePause>().is_paused=true;
            record_moba_building_damage(&world,Some(source),target,true);assert_eq!(balance(&world,1),0);
            world.write_resource::<GamePause>().is_paused=false;
            world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Finished {winner:Some(0),tick:1};
            record_moba_building_damage(&world,Some(source),target,true);assert_eq!(balance(&world,1),0);
            world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
            record_moba_building_damage(&world,Some(source),target,false);assert_eq!(balance(&world,1),0);
            record_moba_building_damage(&world,Some(source),target,true);
            assert_eq!(balance(&world,1),match role {LaneRole::Tower=>225,LaneRole::Inhibitor=>75,LaneRole::NexusTower=>125,LaneRole::Base=>14,_=>0});
            let once=balance(&world,1);record_moba_building_damage(&world,Some(source),target,true);assert_eq!(balance(&world,1),once);
        }
    }

    #[test]
    fn local_pool_excludes_lethal_pending_and_outside_radius_and_disabled_legacy_stays_zero() {
        let (world,target,source,helper)=fixture();
        world.write_resource::<MobaMatch>().heroes[0].lethal_pending=true;
        world.write_storage::<Pos>().get_mut(helper).unwrap().0=Vec2::new(Fixed64::from_i32(1200)+Fixed64::from_raw(1),Fixed64::ZERO);
        record_moba_building_damage(&world,Some(source),target,true);
        assert_eq!(balance(&world,1),100);assert_eq!(balance(&world,3),100);assert_eq!(balance(&world,5),100);
        let (world,target,source,_)=fixture();
        world.write_resource::<MobaMatch>().config.building_reward_rules=BuildingRewardRules::default();
        record_moba_building_damage(&world,Some(source),target,true);
        assert_eq!(balance(&world,1),0);assert_eq!(balance(&world,3),0);
    }
    #[test]
    fn invalid_policy_is_atomic_and_policy_and_life_ledger_change_digest() {
        for rules in [BuildingRewardRules {local_radius:0,..Default::default()},
            BuildingRewardRules {tower:BuildingBounty {team_gold:1_000_001,local_gold_pool:0},..Default::default()},
            BuildingRewardRules {base:BuildingBounty {team_gold:0,local_gold_pool:1_000_001},..Default::default()}] {
            let pool=crate::runtime::StateInitializer::create_thread_pool();
            let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
            assert!(setup_single_lane_match(&mut world,SingleLaneConfig {building_reward_rules:rules,..Default::default()}).is_err());
            assert!(world.try_fetch::<MobaMatch>().is_none());assert_eq!((&world.entities(),&world.read_storage::<Unit>()).join().count(),0);
        }
        let (world,target,_,_)=fixture();let before=single_lane_replay_digest(&world);
        world.write_resource::<MobaMatch>().config.building_reward_rules.tower.team_gold+=1;
        assert_ne!(single_lane_replay_digest(&world),before);
        world.write_resource::<MobaMatch>().config.building_reward_rules.tower.team_gold-=1;
        assert_eq!(single_lane_replay_digest(&world),before);
        world.write_resource::<MobaMatch>().units.get_mut(&entity_key(target)).unwrap().last_hit_retired=true;
        assert_ne!(single_lane_replay_digest(&world),before);
    }

    #[test]
    fn authored_tower_layer_overrides_follow_depth_after_outer_retirement_and_validate_bounds() {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        setup_single_lane_match(&mut world,SingleLaneConfig {map_id:Some(map.id.into()),lane_length:Fixed64::from_i32(map.lane_length),
            building_reward_rules:BuildingRewardRules::training(),..Default::default()}).unwrap();
        world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
        let hero=world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();world.write_storage::<Gold>().get_mut(hero).unwrap().0=0;
        let targets:Vec<_>=world.read_resource::<MobaMatch>().lane_tower_layers[0].iter().map(|layer|layer[1].unwrap()).collect();
        let mut total=0;
        for (depth,target) in targets.into_iter().enumerate() {
            let origin=world.read_storage::<Pos>().get(target).unwrap().0;world.write_storage::<Pos>().get_mut(hero).unwrap().0=origin;
            record_moba_building_damage(&world,Some(hero),target,true);total+=[350,425,500][depth];
            assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0,total);
            record_moba_death(&mut world,target);
        }
        let mut rules=BuildingRewardRules::default();rules.first_tower.team_gold=1_000_001;assert!(!rules.is_valid());
        rules=BuildingRewardRules::default();rules.tower_layers[7]=Some(BuildingBounty {team_gold:0,local_gold_pool:1_000_001});assert!(!rules.is_valid());
    }

    #[test]
    fn actual_inhibitor_respawn_starts_new_reward_life_without_historical_growth() {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        world.insert(TowerTemplateRegistry::default());
        let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        setup_single_lane_match(&mut world,SingleLaneConfig {map_id:Some(map.id.into()),
            lane_length:Fixed64::from_i32(map.lane_length),building_reward_rules:BuildingRewardRules::training(),..Default::default()}).unwrap();
        world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
        let source=world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
        world.write_storage::<Gold>().get_mut(source).unwrap().0=0;
        let towers:Vec<_>=world.read_resource::<MobaMatch>().lane_tower_layers[0].iter().filter_map(|layer|layer[1]).collect();
        for tower in towers {record_moba_death(&mut world,tower);}
        let mut previous=None;
        for life in 1..=3 {
            let target=world.read_resource::<MobaMatch>().base_structures.iter()
                .find(|n|n.team==1 && n.lane==0 && n.role==LaneRole::Inhibitor).unwrap().entity.unwrap();
            assert_ne!(Some(target),previous);
            let origin=world.read_storage::<Pos>().get(target).unwrap().0;
            world.write_storage::<Pos>().get_mut(source).unwrap().0=origin;
            flush(&mut world,[lethal(source,target)]);
            assert_eq!(balance(&world,1),125*life,"once for each actual structure life");
            // Death is committed by the production outcome path before respawn.
            flush(&mut world,[]);world.maintain();
            let mut state=world.remove::<MobaMatch>().unwrap();
            state.schedule_base_structure_respawns(Fixed64::from_i32(300*life));
            state.elapsed=Fixed64::from_i32(300*life);state.tick_base_structures(&mut world);world.insert(state);
            previous=Some(target);
        }
    }
}
