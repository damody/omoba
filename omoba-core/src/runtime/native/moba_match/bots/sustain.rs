//! Owner resources/public-home recovery; threats use disclosed units only.
use super::*;

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotSustainPolicy {
    pub recall_below_hp_per_mille: u16,
    pub leave_base_at_hp_per_mille: u16,
    pub threat_radius: u32,
    #[serde(default)]
    pub mana: Option<BotManaSustainPolicy>,
}
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotManaSustainPolicy {
    pub recall_below_per_mille: u16,
    pub leave_base_at_per_mille: u16,
}
impl BotSustainPolicy {
    pub fn validate(&self) -> Result<(), &'static str> {
        if let Some(mana)=self.mana {
            if mana.recall_below_per_mille==0 || mana.recall_below_per_mille>=mana.leave_base_at_per_mille
                || mana.leave_base_at_per_mille>1000 {
                return Err("bot mana sustain requires 0 < recall < leave <=1000");
            }
            if omoba_template_ids::MOBA_BASE_RECOVERY_MANA_PER_SECOND==0 {
                return Err("bot mana sustain requires compiled base mana recovery");
            }
        }
        if self.recall_below_hp_per_mille==0
            || self.recall_below_hp_per_mille>=self.leave_base_at_hp_per_mille
            || self.leave_base_at_hp_per_mille>1000 || !(1..=10_000).contains(&self.threat_radius) {
            return Err("bot sustain requires 0 < recall < leave <=1000 and threat radius 1..10000");
        }
        if omoba_template_ids::MOBA_BASE_RECOVERY_HP_PER_SECOND==0
            || omoba_template_ids::MOBA_BASE_RECOVERY_RADIUS==0 {
            return Err("bot sustain requires enabled compiled base recovery");
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq)]
pub(super) enum Recovery { Hold, Recall, Retreat(Vec2) }

#[cfg(test)]
fn decide(policy:&BotSustainPolicy,health:&CProperty,own:Vec2,home:Vec2,
    team:u32,seen:&[SeenUnit]) -> Option<Recovery> {
    decide_with_mana(policy,health,None,own,home,team,seen)
}
pub(super) fn decide_with_mana(policy:&BotSustainPolicy,health:&CProperty,
    mana:Option<&crate::runtime::ability_runtime::ManaPool>,own:Vec2,home:Vec2,
    team:u32,seen:&[SeenUnit]) -> Option<Recovery> {
    if health.hp<=Fixed64::ZERO || health.mhp<=Fixed64::ZERO {return None;}
    let below=|threshold|i128::from(health.hp.raw())*1000
        < i128::from(health.mhp.raw())*i128::from(threshold);
    let radius=Fixed64::from_i32(omoba_template_ids::MOBA_BASE_RECOVERY_RADIUS as i32);
    // No pool and a zero-capacity pool are different states, but neither can
    // be restored by waiting. Never turn either into an infinite base hold.
    let low_mana=|leave|match (policy.mana,mana) {
        (Some(policy),Some(pool)) if pool.maximum()>Fixed64::ZERO=>
            i128::from(pool.current().raw())*1000 < i128::from(pool.maximum().raw())
                * i128::from(if leave {policy.leave_base_at_per_mille} else {policy.recall_below_per_mille}),
        _=>false,
    };
    if (own-home).length_squared()<=radius*radius {
        return (below(policy.leave_base_at_hp_per_mille) || low_mana(true)).then_some(Recovery::Hold);
    }
    if !below(policy.recall_below_hp_per_mille) && !low_mana(false) {return None;}
    if disclosed_threat(own,team,seen,policy.threat_radius) {
        Some(Recovery::Retreat(home))
    } else {Some(Recovery::Recall)}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mana_sustain_thresholds_legacy_zero_capacity_and_disclosed_threats() {
        use crate::runtime::ability_runtime::ManaPool;
        let policy=BotSustainPolicy {recall_below_hp_per_mille:350,leave_base_at_hp_per_mille:850,threat_radius:1000,
            mana:Some(BotManaSustainPolicy {recall_below_per_mille:200,leave_base_at_per_mille:850})};
        assert!(policy.validate().is_ok());
        let hp=property(Fixed64::from_i32(100),Fixed64::ZERO);
        let home=Vec2::ZERO;let own=Vec2::new(Fixed64::from_i32(2000),Fixed64::ZERO);
        let mut pool=ManaPool::new(Fixed64::from_raw(200*1024-1),Fixed64::from_i32(1000)).unwrap();
        assert_eq!(decide_with_mana(&policy,&hp,Some(&pool),own,home,1,&[]),Some(Recovery::Recall));
        let enemy=SeenUnit {canonical_id:1,position:own,team:2,kind:1,owner_player_id:2,hp_raw:1,max_hp_raw:100};
        assert_eq!(decide_with_mana(&policy,&hp,Some(&pool),own,home,1,&[enemy]),Some(Recovery::Retreat(home)));
        pool.restore(Fixed64::from_raw(1)).unwrap();
        assert_eq!(decide_with_mana(&policy,&hp,Some(&pool),own,home,1,&[]),None);
        assert_eq!(decide_with_mana(&policy,&hp,Some(&pool),home,home,1,&[]),Some(Recovery::Hold));
        pool.restore(Fixed64::from_i32(650)).unwrap();
        assert_eq!(decide_with_mana(&policy,&hp,Some(&pool),home,home,1,&[]),None);
        for pool in [None,Some(ManaPool::full(Fixed64::ZERO).unwrap())] {
            assert_eq!(decide_with_mana(&policy,&hp,pool.as_ref(),own,home,1,&[]),None);
            assert_eq!(decide_with_mana(&policy,&hp,pool.as_ref(),home,home,1,&[]),None);
        }
        let legacy:BotSustainPolicy=serde_json::from_str(r#"{"recall_below_hp_per_mille":350,"leave_base_at_hp_per_mille":850,"threat_radius":1000}"#).unwrap();
        assert!(legacy.mana.is_none());
        for (recall,leave) in [(0,850),(850,850),(900,850),(200,1001)] {
            assert!(BotSustainPolicy {mana:Some(BotManaSustainPolicy {recall_below_per_mille:recall,leave_base_at_per_mille:leave}),..policy}.validate().is_err());
        }
    }
    #[test]
    fn role_bot_sustain_thresholds_home_and_disclosed_threats() {
        let policy=BotSustainPolicy {recall_below_hp_per_mille:350,leave_base_at_hp_per_mille:850,threat_radius:1000,mana:None};
        assert!(policy.validate().is_ok());
        let home=Vec2::ZERO;let own=Vec2::new(Fixed64::from_i32(2000),Fixed64::ZERO);
        let mut hp=CProperty {hp:Fixed64::from_i32(349),mhp:Fixed64::from_i32(1000),
            msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO};
        assert_eq!(decide(&policy,&hp,own,home,1,&[]),Some(Recovery::Recall));
        let enemy=SeenUnit {canonical_id:1,position:own,team:2,kind:1,owner_player_id:2,hp_raw:1,max_hp_raw:100};
        assert_eq!(decide(&policy,&hp,own,home,1,&[enemy]),Some(Recovery::Retreat(home)));
        assert_eq!(decide(&policy,&hp,own,home,1,&[SeenUnit {hp_raw:0,..enemy}]),Some(Recovery::Recall));
        assert_eq!(decide(&policy,&hp,own,home,1,&[SeenUnit {team:1,..enemy}]),Some(Recovery::Recall));
        hp.hp=Fixed64::from_i32(350);assert_eq!(decide(&policy,&hp,own,home,1,&[]),None);
        hp.hp=Fixed64::from_i32(849);assert_eq!(decide(&policy,&hp,home,home,1,&[]),Some(Recovery::Hold));
        hp.hp=Fixed64::from_i32(850);assert_eq!(decide(&policy,&hp,home,home,1,&[]),None);
        hp.hp=Fixed64::ZERO;assert_eq!(decide(&policy,&hp,home,home,1,&[]),None);
        for invalid in [BotSustainPolicy {recall_below_hp_per_mille:0,..policy},
            BotSustainPolicy {leave_base_at_hp_per_mille:350,..policy},
            BotSustainPolicy {leave_base_at_hp_per_mille:1001,..policy},
            BotSustainPolicy {threat_radius:0,..policy}] {assert!(invalid.validate().is_err());}
    }
}
