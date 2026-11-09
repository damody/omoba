//! Spawn-time growth from the deterministic, pause-aware match clock.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MinionGrowthRules {
    pub interval_seconds:u32,
    pub hp_step_percent:u16,
    pub damage_step_percent:u16,
    pub max_hp_bonus_percent:u16,
    pub max_damage_bonus_percent:u16,
    /// Flat armor every interval (E187: the original creeps gain +1 armor every 300 s).
    pub armor_step:u16,
}

impl Default for MinionGrowthRules {
    fn default()->Self {
        Self {interval_seconds:180,hp_step_percent:0,damage_step_percent:0,
            max_hp_bonus_percent:0,max_damage_bonus_percent:0,armor_step:0}
    }
}

impl MinionGrowthRules {
    pub fn training()->Self {
        Self {hp_step_percent:10,damage_step_percent:5,max_hp_bonus_percent:200,
            max_damage_bonus_percent:100,..Self::default()}
    }
    /// The compiled map's time growth (moba_maps.lua minion_time_growth; the training growth by default).
    pub fn for_map(map:&omoba_template_ids::MobaMapConst)->Self {
        let g=map.minion_time_growth();
        Self {interval_seconds:g.interval_seconds,hp_step_percent:g.hp_step_percent,damage_step_percent:g.damage_step_percent,
            max_hp_bonus_percent:g.max_hp_bonus_percent,max_damage_bonus_percent:g.max_damage_bonus_percent,armor_step:g.armor_step}
    }
    pub fn is_valid(self)->bool {
        (1..=3600).contains(&self.interval_seconds) && self.hp_step_percent<=100
            && self.damage_step_percent<=100 && self.max_hp_bonus_percent<=1000
            && self.max_damage_bonus_percent<=1000 && self.armor_step<=100
    }
    fn scale(self,base:i32,elapsed:Fixed64,step:u16,cap:u16)->i32 {
        let stages=elapsed.raw().max(0)/(i64::from(self.interval_seconds)*1024);
        let bonus=stages.saturating_mul(i64::from(step)).min(i64::from(cap));
        (i64::from(base.max(0))*(100+bonus)/100).min(i64::from(i32::MAX)) as i32
    }
    /// The armor a creep spawned at `elapsed` gains over its authored armor (game_attr CREEP_ARMOR_BONUS).
    pub(crate) fn armor_bonus(self,elapsed:Fixed64)->i32 {self.armor(0,elapsed)}
    /// Spawn armor: the authored armor plus one step per full interval of play.
    pub(super) fn armor(self,base:i32,elapsed:Fixed64)->i32 {
        if !self.is_valid() {return base;}
        let stages=elapsed.raw().max(0)/(i64::from(self.interval_seconds)*1024);
        (i64::from(base)+stages.saturating_mul(i64::from(self.armor_step))).clamp(0,i64::from(i32::MAX)) as i32
    }
    pub(super) fn stats(self,hp:i32,damage:i32,elapsed:Fixed64)->(i32,i32) {
        if !self.is_valid() {return (hp,damage);}
        (self.scale(hp,elapsed,self.hp_step_percent,self.max_hp_bonus_percent),
            self.scale(damage,elapsed,self.damage_step_percent,self.max_damage_bonus_percent))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Join;
    #[test]
    fn growth_closed_boundary_caps_floor_overflow_and_disabled() {
        let rules=MinionGrowthRules::training();
        assert_eq!(rules.stats(181,25,Fixed64::from_i32(180)-Fixed64::from_raw(1)),(181,25));
        assert_eq!(rules.stats(181,25,Fixed64::from_i32(180)),(199,26));
        assert_eq!(rules.stats(181,25,Fixed64::from_i32(100_000)),(543,50));
        assert_eq!(rules.stats(i32::MAX,i32::MAX,Fixed64::from_raw(i64::MAX)),(i32::MAX,i32::MAX));
        assert_eq!(rules.stats(181,25,Fixed64::from_i32(-1)),(181,25));
        assert_eq!(MinionGrowthRules::default().stats(181,25,Fixed64::from_i32(100_000)),(181,25));
    }
    /// game_attr CREEP_ARMOR_BONUS: one armor step per full interval, nothing with invalid rules.
    #[test]
    fn armor_bonus_counts_full_intervals() {
        let rules=MinionGrowthRules {interval_seconds:300,armor_step:1,..MinionGrowthRules::default()};
        let at=|s:i32|rules.armor_bonus(Fixed64::from_i32(s));
        assert_eq!((at(0),at(299),at(300),at(900)),(0,0,1,3));
        assert_eq!(MinionGrowthRules {interval_seconds:0,armor_step:1,..MinionGrowthRules::default()}.armor_bonus(Fixed64::from_i32(900)),0);
    }
    /// game_attr CREEP_WAVE / CREEP_ARMOR_BONUS read the match: the first wave is 0 (原作 _G.A_count).
    #[test]
    fn scripts_read_the_creep_wave_and_armor_bonus() {
        use omb_script_abi::ext::{game_attr, op, ExtArg as A, ExtValue as V};
        use omb_script_abi::world::GameWorld;
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        setup_single_lane_match(&mut world,SingleLaneConfig {map_id:Some(map.id.into()),
            lane_length:Fixed64::from_i32(map.lane_length),authored_minion_waves:true,
            minion_growth_rules:MinionGrowthRules {interval_seconds:300,armor_step:1,..MinionGrowthRules::default()},..Default::default()}).unwrap();
        let read=|world:&World|{
            let cache=crate::runtime::native::scripting::ParallelAdapterCache::new(world,1);
            let me=world.entities().join().next().unwrap();
            let mut adapter=crate::runtime::native::scripting::ParallelWorldAdapter::new(&cache,me);
            let mut game=|attr:&str|adapter.ext(op::GAME_ATTR,abi_stable::std_types::RSlice::from_slice(&[A::Str(attr.into())])).into_result();
            (game(game_attr::CREEP_WAVE),game(game_attr::CREEP_ARMOR_BONUS))
        };
        let n=|v:i32|Ok(V::Num(Fixed64::from_i32(v)));
        assert_eq!(read(&world),(n(-1),n(0)),"before the first wave");
        let mut state=world.remove::<MobaMatch>().unwrap();state.phase=MobaMatchPhase::Playing;
        state.elapsed=Fixed64::from_i32(650);state.spawn_wave(&mut world);state.spawn_wave(&mut world);
        world.insert(state);
        assert_eq!(read(&world),(n(1),n(2)),"two waves spawned at 650 s: index 1, two armor steps");
    }
    #[test]
    fn actual_authored_waves_apply_growth_once_and_preserve_old_creeps_and_pause() {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        setup_single_lane_match(&mut world,SingleLaneConfig {map_id:Some(map.id.into()),
            lane_length:Fixed64::from_i32(map.lane_length),authored_minion_waves:true,
            minion_growth_rules:MinionGrowthRules::training(),..Default::default()}).unwrap();
        let mut state=world.remove::<MobaMatch>().unwrap();state.phase=MobaMatchPhase::Playing;
        state.elapsed=Fixed64::from_i32(179);state.spawn_wave(&mut world);
        let old:BTreeMap<_,_>=(&world.entities(),&world.read_storage::<Unit>(),&world.read_storage::<CProperty>()).join()
            .filter(|(e,_,_)|state.units.get(&entity_key(*e)).is_some_and(|u|u.role==LaneRole::Creep))
            .map(|(e,u,p)|(e,(u.max_hp,u.base_damage,p.hp))).collect();
        state.elapsed=Fixed64::from_i32(180);state.spawn_wave(&mut world);
        for (e,u,p) in (&world.entities(),&world.read_storage::<Unit>(),&world.read_storage::<CProperty>()).join() {
            if !state.units.get(&entity_key(e)).is_some_and(|u|u.role==LaneRole::Creep) {continue;}
            if let Some(prior)=old.get(&e) {assert_eq!((u.max_hp,u.base_damage,p.hp),*prior);}
            else {let definition=map.minion_wave.iter().find(|m|m.id==u.id).unwrap();
                let (hp,damage)=definition.grown(state.waves);
                let (hp,damage)=state.config.minion_growth_rules.stats(hp,damage,state.elapsed);
                assert_eq!((u.max_hp,u.current_hp,u.base_damage),(hp,hp,damage));
                assert_eq!((p.hp,p.mhp),(Fixed64::from_i32(hp),Fixed64::from_i32(hp)));}
        }
        let waves=state.waves;world.insert(state);world.write_resource::<GamePause>().is_paused=true;
        world.write_resource::<DeltaTime>().0=Fixed64::from_i32(500);
        assert!(!begin_moba_match_tick(&mut world));
        assert_eq!(world.read_resource::<MobaMatch>().elapsed,Fixed64::from_i32(180));
        assert_eq!(world.read_resource::<MobaMatch>().waves,waves);
    }
    #[test]
    fn invalid_growth_setup_is_atomic_and_rules_enter_digest() {
        let invalid=MinionGrowthRules {interval_seconds:0,..Default::default()};
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        assert!(setup_single_lane_match(&mut world,SingleLaneConfig {minion_growth_rules:invalid,..Default::default()}).is_err());
        assert_eq!((&world.entities(),&world.read_storage::<Unit>()).join().count(),0);
        setup_single_lane_match(&mut world,SingleLaneConfig::default()).unwrap();
        let before=single_lane_replay_digest(&world);
        world.write_resource::<MobaMatch>().config.minion_growth_rules=MinionGrowthRules::training();
        assert_ne!(single_lane_replay_digest(&world),before);
    }
}
