//! Player-roster streaks and bounded, content-authored shutdown gold.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KillBountyRules {
    pub threshold:u32,
    pub initial_gold:u32,
    pub per_kill_gold:u32,
    pub max_gold:u32,
}
impl KillBountyRules {
    pub fn training()->Self {
        serde_json::from_str(omoba_template_ids::MOBA_KILL_STREAK_BOUNTY_JSON).expect("validated compiled bounty rules")
    }
    pub fn is_valid(self)->bool {
        if self.threshold==0 {return self==Self::default();}
        self.threshold<=100 && self.initial_gold>0 && self.initial_gold<=self.max_gold
            && self.per_kill_gold<=1_000_000 && self.max_gold<=1_000_000
    }
    pub fn bonus(self,streak:u32)->u32 {
        if !self.is_valid() || self.threshold==0 || streak<self.threshold {return 0;}
        self.initial_gold.saturating_add(streak.saturating_sub(self.threshold).saturating_mul(self.per_kill_gold)).min(self.max_gold)
    }
}

impl MobaMatch {
    pub fn shutdown_bounty(&self,player:u32)->u32 {
        self.heroes.iter().find(|s|s.player_id==player && s.entity.is_some() && !s.lethal_pending)
            .map_or(0,|s|self.config.kill_bounty_rules.bonus(s.kill_streak))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{process_outcomes,RuntimeEventVecSink,TowerTemplateRegistry};
    use specs::Join;
    fn fixture()->World {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut w=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        w.insert(TowerTemplateRegistry::default());
        setup_single_lane_match(&mut w,SingleLaneConfig {kill_bounty_rules:KillBountyRules::training(),..Default::default()}).unwrap();
        w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
        for gold in (&mut w.write_storage::<Gold>()).join(){gold.0=0;}
        w
    }
    fn hero(w:&World,side:usize)->Entity {w.read_resource::<MobaMatch>().heroes[side].entity.unwrap()}
    fn flush(w:&mut World){
        process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();
        process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();w.maintain();
    }
    fn hit(w:&mut World,source:Option<Entity>,target:Entity){
        let outcome=source.map_or(Outcome::ScriptDirectDamage {target,amount:Fixed64::from_i32(100_000)},|source|
            Outcome::Damage {source,target,pos:Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,
                real:Fixed64::from_i32(100_000),damage_profile:0,predeclared:false});
        w.write_resource::<Vec<Outcome>>().push(outcome);flush(w);
    }
    fn respawn(w:&mut World,side:usize){let mut s=w.remove::<MobaMatch>().unwrap();s.spawn_hero(w,side);w.insert(s);}
    #[test]
    fn thresholds_cap_disable_and_extreme_streak_are_bounded(){
        let r=KillBountyRules::training();assert!(r.is_valid());
        assert_eq!((r.bonus(2),r.bonus(3),r.bonus(4),r.bonus(u32::MAX)),(0,150,250,700));
        assert_eq!(KillBountyRules::default().bonus(u32::MAX),0);
        for bad in [KillBountyRules {threshold:0,..r},KillBountyRules {threshold:101,..r},
            KillBountyRules {initial_gold:701,..r},KillBountyRules {max_gold:1_000_001,..r}]{assert!(!bad.is_valid());assert_eq!(bad.bonus(9),0);}
    }
    #[test]
    fn actual_kills_accumulate_shutdown_once_and_reset_on_uncredited_death(){
        let mut w=fixture();let first=hero(&w,0);
        for n in 1..=4 {
            let victim=hero(&w,1);hit(&mut w,Some(first),victim);
            assert_eq!(w.read_resource::<MobaMatch>().heroes[0].kill_streak,n);respawn(&mut w,1);
        }
        assert_eq!(w.read_resource::<MobaMatch>().shutdown_bounty(1),250);
        let enemy=hero(&w,1);let before=w.read_storage::<Gold>().get(enemy).unwrap().0;
        hit(&mut w,Some(enemy),first);flush(&mut w);
        assert_eq!(w.read_storage::<Gold>().get(enemy).unwrap().0,before+300+250);
        assert_eq!(w.read_resource::<MobaMatch>().shutdown_bounty(1),0);
        assert_eq!(w.read_resource::<MobaMatch>().heroes[0].kill_streak,0);
        w.write_resource::<Vec<Outcome>>().push(Outcome::Death {ent:first,pos:Vec2::ZERO});flush(&mut w);
        assert_eq!(w.read_storage::<Gold>().get(enemy).unwrap().0,before+550);
        respawn(&mut w,0);let next=hero(&w,0);assert_eq!(w.read_resource::<MobaMatch>().shutdown_bounty(1),0);
        w.write_resource::<MobaMatch>().heroes[0].kill_streak=8;
        hit(&mut w,None,next);assert_eq!(w.read_resource::<MobaMatch>().heroes[0].kill_streak,0);
        assert_eq!(w.read_storage::<Gold>().get(enemy).unwrap().0,before+550);
    }
    #[test]
    fn forced_death_pause_phase_digest_and_configuration_are_explicit(){
        let mut w=fixture();let source=hero(&w,0);let target=hero(&w,1);
        let before=single_lane_replay_digest(&w);w.write_resource::<MobaMatch>().heroes[1].kill_streak=3;
        assert_ne!(single_lane_replay_digest(&w),before);
        for paused in [true,false] {
            w.write_resource::<GamePause>().is_paused=paused;
            if !paused {w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Warmup;}
            record_moba_hero_damage(&w,Some(source),target,true);
            assert_eq!(w.read_resource::<MobaMatch>().heroes[1].kill_streak,3);
        }
        w.write_resource::<GamePause>().is_paused=false;record_moba_death(&mut w,target);
        assert_eq!(w.read_resource::<MobaMatch>().heroes[1].kill_streak,0);
        assert_eq!(w.read_resource::<MobaMatch>().heroes[0].kills,0);
        let mut s=w.remove::<MobaMatch>().unwrap();s.config.kill_bounty_rules.threshold=101;w.insert(s);
        let pool=crate::runtime::StateInitializer::create_thread_pool();let mut other=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        assert!(setup_single_lane_match(&mut other,SingleLaneConfig {kill_bounty_rules:KillBountyRules {threshold:101,..KillBountyRules::training()},..Default::default()}).is_err());
    }
}
