//! Private deterministic focus for already committed tower attacks.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TowerFocusRules {
    pub bonus_per_shot_percent: u16,
    pub max_bonus_percent: u16,
    pub reset_after: Fixed64,
}

impl Default for TowerFocusRules {
    fn default() -> Self {
        Self {bonus_per_shot_percent:25,max_bonus_percent:100,reset_after:Fixed64::from_i32(3)}
    }
}

impl TowerFocusRules {
    pub fn is_valid(self) -> bool {
        self.bonus_per_shot_percent<=1000 && self.max_bonus_percent<=1000
            && self.reset_after>Fixed64::ZERO && self.reset_after<=Fixed64::from_i32(60)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct TowerFocusState {
    target: Option<Entity>,
    last_attack_at: Fixed64,
    bonus_percent: u16,
}

impl TowerFocusState {
    pub(super) fn retire_target(&mut self,target:Entity) {
        if self.target==Some(target) {*self=Self::default();}
    }
    fn active(&self,target:Option<Entity>,now:Fixed64,rules:TowerFocusRules) -> bool {
        self.target.is_some() && self.target==target && self.last_attack_at>=Fixed64::ZERO
            && now>=self.last_attack_at && now-self.last_attack_at<rules.reset_after
    }

    pub(super) fn reconcile(&mut self,target:Option<Entity>,now:Fixed64,rules:TowerFocusRules) {
        if !self.active(target,now,rules) { *self=Self::default(); }
    }

    pub(super) fn commit(&mut self,target:Entity,now:Fixed64,base:Fixed64,rules:TowerFocusRules) -> Fixed64 {
        if !rules.is_valid() || now<Fixed64::ZERO { *self=Self::default();return base; }
        let bonus=if self.active(Some(target),now,rules) {
            self.bonus_percent.saturating_add(rules.bonus_per_shot_percent).min(rules.max_bonus_percent)
        } else {0};
        self.target=Some(target);self.last_attack_at=now;self.bonus_percent=bonus;
        base*Fixed64::from_i32(100+i32::from(bonus))/Fixed64::from_i32(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Join;
    fn entities()->(World,Entity,Entity) {
        let mut world=World::new();let a=world.create_entity().build();let b=world.create_entity().build();(world,a,b)
    }
    #[test]
    fn ramp_caps_and_restarts_on_target_loss_change_and_timeout() {
        let (_world,a,b)=entities();let rules=TowerFocusRules::default();let mut focus=TowerFocusState::default();
        for (tick,damage) in [100,125,150,175,200,200,200].into_iter().enumerate() {
            assert_eq!(focus.commit(a,Fixed64::from_i32(tick as i32),Fixed64::from_i32(100),rules),Fixed64::from_i32(damage));
        }
        assert_eq!(focus.commit(b,Fixed64::from_i32(7),Fixed64::from_i32(100),rules),Fixed64::from_i32(100));
        focus.reconcile(None,Fixed64::from_i32(7),rules);
        assert_eq!(focus.commit(b,Fixed64::from_i32(8),Fixed64::from_i32(100),rules),Fixed64::from_i32(100));
        assert_eq!(focus.commit(b,Fixed64::from_i32(11),Fixed64::from_i32(100),rules),Fixed64::from_i32(100),"deadline is closed");
    }
    #[test]
    fn rules_are_bounded_disable_cleanly_and_invalid_clocks_fail_closed() {
        let (_world,a,_)=entities();let mut rules=TowerFocusRules::default();let mut focus=TowerFocusState::default();
        rules.max_bonus_percent=0;
        for now in [0,1,2] {assert_eq!(focus.commit(a,Fixed64::from_i32(now),Fixed64::ONE,rules),Fixed64::ONE);}
        rules.max_bonus_percent=1001;assert!(!rules.is_valid());
        assert_eq!(focus.commit(a,Fixed64::from_i32(3),Fixed64::ONE,rules),Fixed64::ONE);
        assert_eq!(focus,TowerFocusState::default());
        rules=TowerFocusRules::default();rules.reset_after=Fixed64::ZERO;assert!(!rules.is_valid());
        rules.reset_after=Fixed64::from_i32(61);assert!(!rules.is_valid());
        rules=TowerFocusRules::default();focus.commit(a,Fixed64::ONE,Fixed64::ONE,rules);
        assert_eq!(focus.commit(a,Fixed64::ZERO,Fixed64::ONE,rules),Fixed64::ONE,"clock regression resets");
    }

    fn fixture()->(World,Entity,Entity) {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        setup_single_lane_match(&mut world,SingleLaneConfig {warmup:Fixed64::ZERO,..Default::default()}).unwrap();
        world.remove::<crate::runtime::TeamVisibilityRuntime>(); // isolate pressure; visibility has its own integration suite
        let (tower,hero)={let mut state=world.write_resource::<MobaMatch>();state.phase=MobaMatchPhase::Playing;
            state.next_wave_at=Fixed64::from_i32(100_000);(state.towers[0].unwrap(),state.heroes[1].entity.unwrap())};
        for pos in (&mut world.write_storage::<Pos>()).join() {pos.0=Vec2::new(Fixed64::from_i32(100_000),Fixed64::from_i32(100_000));}
        world.write_storage::<Pos>().get_mut(tower).unwrap().0=Vec2::new(Fixed64::ZERO,Fixed64::ZERO);
        world.write_storage::<Pos>().get_mut(hero).unwrap().0=Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO);
        (world,tower,hero)
    }
    fn shot(world:&mut World,tower:Entity)->Option<(Entity,Fixed64)> {
        world.write_resource::<Vec<DamageInstance>>().clear();world.write_resource::<DeltaTime>().0=Fixed64::ONE;
        assert!(begin_moba_match_tick(world));
        world.read_resource::<Vec<DamageInstance>>().iter().find(|d|d.source.source_entity==tower)
            .map(|d|(d.target,d.damage_types.physical))
    }
    #[test]
    fn actual_tower_attacks_ramp_pause_reset_and_hash_private_focus() {
        for role in [LaneRole::Tower,LaneRole::NexusTower] {
        let (mut world,tower,hero)=fixture();
        world.write_resource::<MobaMatch>().units.get_mut(&entity_key(tower)).unwrap().role=role;
        let base=shot(&mut world,tower).unwrap().1;
        assert_eq!(shot(&mut world,tower),Some((hero,base*Fixed64::from_i32(125)/Fixed64::from_i32(100))));
        let before=world.read_resource::<MobaMatch>().units[&entity_key(tower)].tower_focus.clone();
        let elapsed=world.read_resource::<MobaMatch>().elapsed;
        world.write_resource::<GamePause>().is_paused=true;
        world.write_resource::<Vec<DamageInstance>>().clear();world.write_resource::<DeltaTime>().0=Fixed64::from_i32(20);
        assert!(!begin_moba_match_tick(&mut world),"paused tick intentionally does not advance simulation");
        assert_eq!(world.read_resource::<MobaMatch>().elapsed,elapsed);
        assert_eq!(world.read_resource::<MobaMatch>().units[&entity_key(tower)].tower_focus,before);
        assert!(world.read_resource::<Vec<DamageInstance>>().is_empty());
        world.write_resource::<GamePause>().is_paused=false;
        assert_eq!(shot(&mut world,tower).unwrap().1,base*Fixed64::from_i32(150)/Fixed64::from_i32(100));
        world.write_storage::<Pos>().get_mut(hero).unwrap().0=Vec2::new(Fixed64::from_i32(5000),Fixed64::ZERO);
        assert!(shot(&mut world,tower).is_none());
        world.write_storage::<Pos>().get_mut(hero).unwrap().0=Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO);
        assert_eq!(shot(&mut world,tower),Some((hero,base)));
        let changed=world.read_resource::<MobaMatch>().units[&entity_key(tower)].tower_focus.clone();
        world.write_resource::<MobaMatch>().units.get_mut(&entity_key(tower)).unwrap().tower_focus=Default::default();
        assert_ne!(changed,TowerFocusState::default());
        let reset_digest=single_lane_replay_digest(&world);
        world.write_resource::<MobaMatch>().units.get_mut(&entity_key(tower)).unwrap().tower_focus=changed;
        assert_ne!(single_lane_replay_digest(&world),reset_digest,"private focus changes replay material");
        record_moba_death(&mut world,hero);
        assert_eq!(world.read_resource::<MobaMatch>().units[&entity_key(tower)].tower_focus,TowerFocusState::default());
        }
    }
    #[test]
    fn actual_creep_target_resets_pressure_and_stays_at_base_damage() {
        let (mut world,tower,hero)=fixture();let base=shot(&mut world,tower).unwrap().1;
        assert!(shot(&mut world,tower).unwrap().1>base);
        let property=world.read_storage::<CProperty>().get(hero).unwrap().clone();
        let creep=world.create_entity().with(property)
            .with(Pos(Vec2::new(Fixed64::from_i32(50),Fixed64::ZERO))).build();
        world.write_resource::<MobaMatch>().track_lane_unit_for_attackability_test(creep,1,LaneRole::Creep,0);
        world.write_storage::<Pos>().get_mut(hero).unwrap().0=Vec2::new(Fixed64::from_i32(5000),Fixed64::ZERO);
        for _ in 0..3 {assert_eq!(shot(&mut world,tower),Some((creep,base)));}
        world.write_storage::<Pos>().get_mut(creep).unwrap().0=Vec2::new(Fixed64::from_i32(5000),Fixed64::ZERO);
        world.write_storage::<Pos>().get_mut(hero).unwrap().0=Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO);
        assert_eq!(shot(&mut world,tower),Some((hero,base)));
    }
    #[test]
    fn invalid_policy_setup_is_atomic_and_separate_towers_do_not_share_focus() {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        let mut config=SingleLaneConfig::default();config.tower_focus_rules.max_bonus_percent=1001;
        let before=world.entities().join().count();assert!(setup_single_lane_match(&mut world,config).is_err());
        assert_eq!(world.entities().join().count(),before);assert!(world.try_fetch::<MobaMatch>().is_none());
        let (_world,a,b)=entities();let rules=TowerFocusRules::default();let mut first=TowerFocusState::default();let mut second=TowerFocusState::default();
        first.commit(a,Fixed64::ZERO,Fixed64::ONE,rules);first.commit(a,Fixed64::ONE,Fixed64::ONE,rules);
        assert_eq!(second.commit(a,Fixed64::ONE,Fixed64::ONE,rules),Fixed64::ONE);
        first.retire_target(a);assert_eq!(first,TowerFocusState::default());
        assert_eq!(second.commit(b,Fixed64::from_i32(2),Fixed64::ONE,rules),Fixed64::ONE);
    }
}
