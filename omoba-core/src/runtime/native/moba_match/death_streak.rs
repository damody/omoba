//! Integer-only, content-authored reduction of ordinary hero kill gold.
#[cfg(test)]
use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeathStreakRules {
    /// Empty disables discount. Index is unanswered deaths; last entry clamps.
    pub basis_points: Vec<u32>,
}
impl DeathStreakRules {
    pub fn training() -> Self {
        Self { basis_points: serde_json::from_str(omoba_template_ids::MOBA_DEATH_STREAK_BASIS_POINTS_JSON)
            .expect("validated compiled death streak curve") }
    }
    pub fn is_valid(&self) -> bool {
        let c=&self.basis_points;
        c.len()<=101 && (c.is_empty() || c[0]==10000)
            && c.iter().all(|n|*n<=10000) && c.windows(2).all(|w|w[1]<=w[0])
    }
    pub fn gold(&self, base: u32, deaths: u32) -> u32 {
        if !self.is_valid() || self.basis_points.is_empty() { return base; }
        let index=(deaths as u64).min((self.basis_points.len()-1) as u64) as usize;
        (u64::from(base)*u64::from(self.basis_points[index])/10000) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{process_outcomes,RuntimeEventVecSink,TowerTemplateRegistry};
    fn fixture()->World {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut w=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        w.insert(TowerTemplateRegistry::default());
        setup_single_lane_match(&mut w,SingleLaneConfig {death_streak_rules:DeathStreakRules::training(),
            kill_bounty_rules:KillBountyRules::training(),..Default::default()}).unwrap();
        w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;w
    }
    fn hero(w:&World,side:usize)->Entity {w.read_resource::<MobaMatch>().heroes[side].entity.unwrap()}
    fn flush(w:&mut World) {
        process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();
        process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();w.maintain();
    }
    fn hit(w:&mut World,source:Option<Entity>,target:Entity) {
        w.write_resource::<Vec<Outcome>>().push(source.map_or(Outcome::ScriptDirectDamage {
            target,amount:Fixed64::from_i32(100_000)},|source|Outcome::Damage {source,target,pos:Vec2::ZERO,
            phys:Fixed64::ZERO,magi:Fixed64::ZERO,real:Fixed64::from_i32(100_000),damage_profile:0,predeclared:false}));
        flush(w);
    }
    fn respawn(w:&mut World,side:usize) {let mut s=w.remove::<MobaMatch>().unwrap();s.spawn_hero(w,side);w.insert(s);}
    #[test]
    fn curve_is_bounded_monotone_integer_only_and_legacy_disabled() {
        let rules=DeathStreakRules::training();assert!(rules.is_valid());
        assert_eq!((0..8).map(|n|rules.gold(300,n)).collect::<Vec<_>>(),[300,300,275,225,187,150,125,100]);
        assert_eq!(rules.gold(300,u32::MAX),100);
        assert_eq!(DeathStreakRules::default().gold(u32::MAX,u32::MAX),u32::MAX);
        assert_eq!(rules.gold(u32::MAX,0),u32::MAX);
        for c in [vec![9999],vec![10000,10001],vec![10000,0,1],vec![10000;102]] {
            let bad=DeathStreakRules {basis_points:c};assert!(!bad.is_valid());assert_eq!(bad.gold(300,3),300);
            let pool=crate::runtime::StateInitializer::create_thread_pool();
            let mut w=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
            assert!(setup_single_lane_match(&mut w,SingleLaneConfig {death_streak_rules:bad,..Default::default()}).is_err());
        }
    }
    #[test]
    fn repeated_actual_kills_discount_across_respawns_and_kill_resets() {
        let mut w=fixture();let killer=hero(&w,0);let start=w.read_storage::<Gold>().get(killer).unwrap().0;
        let mut earned=0;
        for (n,reward) in [300,300,275,225,187,150,125,100,100].into_iter().enumerate() {
            let victim=hero(&w,1);hit(&mut w,Some(killer),victim);earned+=reward;
            record_moba_hero_damage(&w,Some(killer),victim,true);
            assert_eq!(w.read_storage::<Gold>().get(killer).unwrap().0,start+earned);
            assert_eq!(w.read_resource::<MobaMatch>().heroes[1].unanswered_deaths,n as u32+1);
            respawn(&mut w,1);
            assert_eq!(w.read_resource::<MobaMatch>().heroes[1].unanswered_deaths,n as u32+1);
        }
        let enemy=hero(&w,1);let before=w.read_storage::<Gold>().get(enemy).unwrap().0;
        hit(&mut w,Some(enemy),killer);
        assert_eq!(w.read_storage::<Gold>().get(enemy).unwrap().0,before+1000,"300 base plus 700 shutdown unaffected");
        assert_eq!(w.read_resource::<MobaMatch>().heroes[1].unanswered_deaths,0);
        assert_eq!(w.read_resource::<MobaMatch>().heroes[0].unanswered_deaths,1);
    }
    #[test]
    fn anonymous_forced_pause_and_invalid_phase_do_not_farm_discount_and_digest_covers_state() {
        let mut w=fixture();let a=hero(&w,0);let b=hero(&w,1);
        let baseline=single_lane_replay_digest(&w);w.write_resource::<MobaMatch>().heroes[1].unanswered_deaths=3;
        assert_ne!(single_lane_replay_digest(&w),baseline);
        let before=single_lane_replay_digest(&w);w.write_resource::<MobaMatch>().config.death_streak_rules.basis_points[2]-=1;
        assert_ne!(single_lane_replay_digest(&w),before);
        w.write_resource::<GamePause>().is_paused=true;record_moba_hero_damage(&w,Some(a),b,true);
        w.write_resource::<GamePause>().is_paused=false;w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Warmup;
        record_moba_hero_damage(&w,Some(a),b,true);
        assert_eq!(w.read_resource::<MobaMatch>().heroes[1].unanswered_deaths,3);
        w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;hit(&mut w,None,b);
        assert_eq!(w.read_resource::<MobaMatch>().heroes[1].unanswered_deaths,3);
        respawn(&mut w,1);let next=hero(&w,1);record_moba_death(&mut w,next);
        assert_eq!(w.read_resource::<MobaMatch>().heroes[1].unanswered_deaths,3);
        assert_eq!(w.read_resource::<MobaMatch>().heroes[0].kills,0);
    }
}
