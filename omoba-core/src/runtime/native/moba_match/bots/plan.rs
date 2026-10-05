//! Server-owned pre-match configuration, never accepted as gameplay input.
use super::*;
use crate::runtime::SimulationTickProfile;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleBotPlayerPlan {
    pub player_id: u32,
    pub team_id: u32,
    pub hero: String,
    pub role: BotRole,
    /// Exact compiled lane ID, not its declaration-order index.
    pub lane: String,
    pub bot: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleBotMatchPlan {
    pub schema_version: u32,
    pub map_id: String,
    pub think_hz: u32,
    pub players: Vec<RoleBotPlayerPlan>,
    #[serde(default)]
    pub ability_policies: Vec<BotAbilityPolicy>,
    #[serde(default)]
    pub ability_learning: Vec<BotAbilityLearningStep>,
    #[serde(default)]
    pub sustain: Option<BotSustainPolicy>,
    #[serde(default)]
    pub item_builds: Vec<BotItemBuild>,
}

impl RoleBotMatchPlan {
    /// Build atomically before allocating a World; changing map declaration
    /// order cannot silently reassign a role to a different named lane.
    pub fn compile(&self, seed: u64, profile: SimulationTickProfile)
        -> Result<(SingleLaneConfig, RoleBotConfig), String>
    {
        self.compile_with_tick_rate(seed,profile.ticks_per_game_second())
    }

    /// The server's configured clock is authoritative, including 90 Hz.
    pub fn compile_with_tick_rate(&self, seed:u64, fps:u32)
        -> Result<(SingleLaneConfig, RoleBotConfig), String>
    {
        if self.schema_version != 1 { return Err("unsupported role bot plan schema".into()); }
        abilities::validate_policies(&self.ability_policies).map_err(str::to_owned)?;
        abilities::validate_learning(&self.ability_learning).map_err(str::to_owned)?;
        items::validate_feasible(&self.item_builds).map_err(str::to_owned)?;
        if let Some(policy)=self.sustain {policy.validate().map_err(str::to_owned)?;}
        if self.think_hz == 0 || self.think_hz > fps {
            return Err("bot think_hz must be between 1 and simulation Hz".into());
        }
        let map = omoba_template_ids::moba_map_by_name(&self.map_id)
            .ok_or_else(|| format!("unknown compiled bot map '{}'", self.map_id))?;
        if self.players.is_empty() || self.players.len() > 10 {
            return Err("role bot plan requires 2..10 players with both teams".into());
        }
        let mut ids = BTreeSet::new();
        let mut roles = BTreeSet::new();
        let mut roster = self.players.clone();
        roster.sort_by_key(|p| p.player_id);
        let mut first = [None, None];
        let mut counts = [0; 2];
        let mut additional = Vec::new();
        let mut assignments = Vec::new();
        for player in roster {
            let side = match player.team_id { 1 => 0, 2 => 1, _ => return Err("bot plan team must be 1 or 2".into()) };
            if player.player_id == 0 || !ids.insert(player.player_id) {
                return Err("bot plan requires unique nonzero player IDs".into());
            }
            if !roles.insert((side,player.role)) || counts[side] >= 5 {
                return Err("bot plan requires unique roles and at most five players per team".into());
            }
            if omoba_template_ids::hero_by_name(&player.hero).and_then(omoba_template_ids::hero_stats).is_none() {
                return Err(format!("unknown active compiled bot hero '{}'",player.hero));
            }
            let lane = map.lanes.iter().position(|lane| lane.id == player.lane)
                .ok_or_else(|| format!("unknown bot lane '{}' in '{}'",player.lane,map.id))?;
            if player.role == BotRole::Jungle && map.jungle_camps.is_empty() {
                return Err("jungle role requires compiled camps".into());
            }
            counts[side] += 1;
            if player.bot {
                let escort_player_id = (player.role == BotRole::Support).then(|| self.players.iter()
                    .find(|p|p.team_id == player.team_id && p.role == BotRole::Carry).map(|p|p.player_id)).flatten();
                assignments.push(BotAssignment { player_id:player.player_id,role:player.role,lane,escort_player_id });
            }
            let entry = SingleLanePlayerConfig { player_id:player.player_id,team_id:player.team_id,hero:player.hero };
            if first[side].is_none() { first[side] = Some(entry); } else { additional.push(entry); }
        }
        let [Some(left),Some(right)] = first else { return Err("bot plan requires both teams".into()); };
        Ok((SingleLaneConfig {
            seed,map_id:Some(map.id.into()),lane_length:Fixed64::from_i32(map.lane_length),
            players:[left.player_id,right.player_id],heroes:[left.hero,right.hero],additional_players:additional,
            base_recovery_enabled:self.sustain.is_some(),
            ..Default::default()
        },RoleBotConfig { assignments,think_interval_ticks:u64::from(fps.div_ceil(self.think_hz)),
            ability_policies:self.ability_policies.clone(),ability_learning:self.ability_learning.clone(),sustain:self.sustain,
            item_builds:self.item_builds.clone() }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn plan() -> RoleBotMatchPlan {
        let mut players = Vec::new();
        for team in 1..=2 {
            for (index,(role,lane)) in [(BotRole::Top,"top"),(BotRole::Mid,"mid"),(BotRole::Carry,"bottom"),
                (BotRole::Support,"bottom"),(BotRole::Jungle,"mid")].into_iter().enumerate() {
                players.push(RoleBotPlayerPlan { player_id:team*10+index as u32,team_id:team,
                    hero:"training_luminary".into(),role,lane:lane.into(),bot:team != 1 || index != 0 });
            }
        }
        RoleBotMatchPlan { schema_version:1,map_id:"three_lane_training".into(),think_hz:5,players,ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new() }
    }
    #[test]
    fn role_bot_plan_compiles_human_and_nine_bots_by_named_lane() {
        let mut p = plan();
        p.ability_learning=vec![BotAbilityLearningStep {ability:"lumen_bolt".into(),rank:1}];
        let (config,bots) = p.compile(42,SimulationTickProfile::Production60Hz).unwrap();
        assert_eq!(bots.ability_learning[0].ability,"lumen_bolt");
        assert_eq!(bots.ability_learning[0].rank,1);
        assert_eq!(config.players,[10,20]); assert_eq!(config.additional_players.len(),8);
        assert_eq!(bots.assignments.len(),9); assert_eq!(bots.think_interval_ticks,12);
        assert!(!bots.assignments.iter().any(|a|a.player_id == 10));
        assert_eq!(bots.assignments.iter().find(|a|a.player_id == 12).unwrap().lane,2);
        assert_eq!(bots.assignments.iter().find(|a|a.player_id == 13).unwrap().escort_player_id,Some(12));
        p.players.reverse();
        let (other,other_bots) = p.compile(42,SimulationTickProfile::Production60Hz).unwrap();
        assert_eq!(other.players,config.players);
        assert_eq!(other_bots.assignments.iter().map(|a|(a.player_id,a.lane)).collect::<Vec<_>>(),
            bots.assignments.iter().map(|a|(a.player_id,a.lane)).collect::<Vec<_>>());
    }
    #[test]
    fn role_bot_plan_rejects_invalid_catalog_identity_and_control_configuration() {
        let valid = plan();
        let reject = |p:RoleBotMatchPlan| assert!(p.compile(1,SimulationTickProfile::Production60Hz).is_err());
        let mut p=valid.clone(); p.schema_version=2; reject(p);
        let mut p=valid.clone(); p.ability_learning=vec![BotAbilityLearningStep {ability:"lumen_bolt".into(),rank:2}]; reject(p);
        let mut p=valid.clone(); p.think_hz=0; reject(p);
        let mut p=valid.clone(); p.think_hz=61; reject(p);
        let mut p=valid.clone(); p.map_id="missing".into(); reject(p);
        let mut p=valid.clone(); p.players[0].hero="missing".into(); reject(p);
        let mut p=valid.clone(); p.players[0].lane="missing".into(); reject(p);
        let mut p=valid.clone(); p.players[0].team_id=3; reject(p);
        let mut p=valid.clone(); p.players[0].player_id=0; reject(p);
        let mut p=valid.clone(); p.players[1].player_id=p.players[0].player_id; reject(p);
        let mut p=valid.clone(); p.players[1].role=p.players[0].role; reject(p);
        let mut p=valid.clone(); p.players.retain(|p|p.team_id==1); reject(p);
        assert!(serde_json::from_str::<RoleBotMatchPlan>(r#"{"schema_version":1,"map_id":"three_lane_training","think_hz":5,"players":[],"unknown":true}"#).is_err());
    }

    #[test]
    fn role_bot_sustain_plan_is_explicit_and_enables_match_recovery() {
        let mut p=plan();
        let (legacy,bots)=p.compile(1,SimulationTickProfile::Production60Hz).unwrap();
        assert!(!legacy.base_recovery_enabled && bots.sustain.is_none());
        p.sustain=Some(BotSustainPolicy {recall_below_hp_per_mille:350,leave_base_at_hp_per_mille:850,threat_radius:1000});
        let (config,bots)=p.compile(1,SimulationTickProfile::Production60Hz).unwrap();
        assert!(config.base_recovery_enabled && bots.sustain.is_some());
        p.sustain.as_mut().unwrap().leave_base_at_hp_per_mille=1001;
        assert!(p.compile(1,SimulationTickProfile::Production60Hz).is_err());
    }

    #[test]
    fn role_bot_items_plan_defaults_and_compiled_validation() {
        let mut p=plan();
        let mut json=serde_json::to_value(&p).unwrap();json.as_object_mut().unwrap().remove("item_builds");
        let legacy:RoleBotMatchPlan=serde_json::from_value(json).unwrap();
        assert!(legacy.compile(1,SimulationTickProfile::Production60Hz).unwrap().1.item_builds.is_empty());
        p.item_builds=vec![BotItemBuild {role:BotRole::Carry,items:vec!["moba_greatsword".into()],return_to_shop:None}];
        assert_eq!(p.compile(1,SimulationTickProfile::Production60Hz).unwrap().1.item_builds[0].items,["moba_greatsword"]);
        p.item_builds[0].items[0]="unknown".into();
        assert!(p.compile(1,SimulationTickProfile::Production60Hz).is_err());
    }
}
