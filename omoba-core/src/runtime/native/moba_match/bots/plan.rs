//! Server-owned pre-match configuration, never accepted as gameplay input.
use super::*;
use crate::runtime::SimulationTickProfile;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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
    #[serde(default)]
    pub mana_enabled: bool,
    pub players: Vec<RoleBotPlayerPlan>,
    #[serde(default)]
    pub ability_policies: Vec<BotAbilityPolicy>,
    #[serde(default)]
    pub ability_learning: Vec<BotAbilityLearningStep>,
    #[serde(default)]
    pub sustain: Option<BotSustainPolicy>,
    #[serde(default)]
    pub item_builds: Vec<BotItemBuild>,
    /// JSON objects of player ID to a two-slot array. Null is an unequipped slot.
    /// An omitted or empty map stays unavailable; nothing is equipped by default.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub summoner_loadouts: BTreeMap<u32, [Option<String>; 2]>,
    /// Spotlight recording stage. Held seats must be bots and get no role decisions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub showcase: Option<ShowcaseConfig>,
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
        if abilities::requires_mana(&self.ability_policies) && !self.mana_enabled {
            return Err("bot recovery intent requires explicit mana_enabled".into());
        }
        abilities::validate_learning(&self.ability_learning).map_err(str::to_owned)?;
        items::validate_feasible(&self.item_builds).map_err(str::to_owned)?;
        if let Some(policy)=self.sustain {policy.validate().map_err(str::to_owned)?;}
        if self.sustain.is_some_and(|policy|policy.mana.is_some()) && !self.mana_enabled {
            return Err("bot mana sustain requires explicit mana_enabled".into());
        }
        if self.think_hz == 0 || self.think_hz > fps {
            return Err("bot think_hz must be between 1 and simulation Hz".into());
        }
        crate::runtime::native::summoner_world::validated_loadouts(
            self.players.iter().map(|player| player.player_id),
            &self.summoner_loadouts,
        ).map_err(|error| format!("role plan summoner loadout rejected: {error:?}"))?;
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
            let held = self.showcase.as_ref().is_some_and(|showcase| showcase.hold_players.contains(&player.player_id));
            if held && !player.bot { return Err("a showcase hold seat must be a bot".into()); }
            if player.bot && !held {
                let escort_player_id = (player.role == BotRole::Support).then(|| self.players.iter()
                    .find(|p|p.team_id == player.team_id && p.role == BotRole::Carry).map(|p|p.player_id)).flatten();
                assignments.push(BotAssignment { player_id:player.player_id,role:player.role,lane,escort_player_id });
            }
            let entry = SingleLanePlayerConfig { player_id:player.player_id,team_id:player.team_id,hero:player.hero };
            if first[side].is_none() { first[side] = Some(entry); } else { additional.push(entry); }
        }
        let [Some(left),Some(right)] = first else { return Err("bot plan requires both teams".into()); };
        Ok((SingleLaneConfig {
            building_reward_rules:BuildingRewardRules::training(),
            tower_plating_rules:Some(TowerPlatingRules::training()),
            minion_growth_rules:MinionGrowthRules::training(),
            seed,map_id:Some(map.id.into()),lane_length:Fixed64::from_i32(map.lane_length),
            tower_hp:Fixed64::from_i32(map.tower_hp),base_hp:Fixed64::from_i32(map.base_hp),
            hero_xp_scale_permille:map.hero_xp_scale_permille(),
            authored_minion_waves:true,wave_interval:Fixed64::from_i32(map.wave_interval_seconds as i32),
            players:[left.player_id,right.player_id],heroes:[left.hero,right.hero],additional_players:additional,
            base_recovery_enabled:self.sustain.is_some(),
            mana_enabled:self.mana_enabled,
            summoner_loadouts:self.summoner_loadouts.clone(),
            showcase:self.showcase.clone(),
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
        RoleBotMatchPlan { schema_version:1,map_id:"three_lane_training".into(),think_hz:5,mana_enabled:false,players,ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),summoner_loadouts:BTreeMap::new(),showcase:None }
    }
    #[test]
    fn role_bot_plan_takes_structure_hit_points_from_the_compiled_map() {
        let shipped = plan().compile(1,SimulationTickProfile::Production60Hz).unwrap().0;
        assert_eq!((shipped.tower_hp,shipped.base_hp),(Fixed64::from_i32(1200),Fixed64::from_i32(1600)));
        let mut quick = plan();
        quick.map_id = "three_lane_quick_training".into();
        let map = omoba_template_ids::moba_map_by_name("three_lane_quick_training").unwrap();
        let config = quick.compile(1,SimulationTickProfile::Production60Hz).unwrap().0;
        assert_eq!((config.tower_hp,config.base_hp),(Fixed64::from_i32(map.tower_hp),Fixed64::from_i32(map.base_hp)));
        assert!(map.tower_hp < 1200 && map.base_hp < 1600);
        // E096: the map's hero XP multiplier reaches the role-plan entry too (E055: every entry).
        assert_eq!(shipped.hero_xp_scale_permille,1000);
        assert_eq!(config.hero_xp_scale_permille,map.hero_xp_scale_permille());
        assert!(map.hero_xp_scale_permille() > 1000);
    }
    #[test]
    fn role_bot_plan_compiles_human_and_nine_bots_by_named_lane() {
        let mut p = plan();
        p.ability_learning=vec![BotAbilityLearningStep {ability:"lumen_bolt".into(),rank:1}];
        let (config,bots) = p.compile(42,SimulationTickProfile::Production60Hz).unwrap();
        assert_eq!(bots.ability_learning[0].ability,"lumen_bolt");
        assert_eq!(bots.ability_learning[0].rank,1);
        assert_eq!(config.players,[10,20]); assert_eq!(config.additional_players.len(),8);
        assert_eq!(config.building_reward_rules,BuildingRewardRules::training());
        assert_eq!(config.tower_plating_rules,Some(TowerPlatingRules::training()));
        assert_eq!(config.minion_growth_rules,MinionGrowthRules::training());
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
        p.sustain=Some(BotSustainPolicy {recall_below_hp_per_mille:350,leave_base_at_hp_per_mille:850,threat_radius:1000,mana:None});
        let (config,bots)=p.compile(1,SimulationTickProfile::Production60Hz).unwrap();
        assert!(config.base_recovery_enabled && bots.sustain.is_some());
        p.sustain.as_mut().unwrap().leave_base_at_hp_per_mille=1001;
        assert!(p.compile(1,SimulationTickProfile::Production60Hz).is_err());
    }

    #[test]
    fn mana_sustain_plan_requires_explicit_rules_and_preserves_legacy() {
        let mut p=plan();
        assert!(!p.compile(1,SimulationTickProfile::Production60Hz).unwrap().0.mana_enabled);
        p.sustain=Some(BotSustainPolicy {recall_below_hp_per_mille:350,leave_base_at_hp_per_mille:850,
            threat_radius:1000,mana:Some(BotManaSustainPolicy {recall_below_per_mille:200,leave_base_at_per_mille:850})});
        assert!(p.compile(1,SimulationTickProfile::Production60Hz).is_err());
        p.mana_enabled=true;
        let (config,_)=p.compile(1,SimulationTickProfile::Production60Hz).unwrap();
        assert!(config.mana_enabled && config.base_recovery_enabled);
        let mut json=serde_json::to_value(plan()).unwrap();
        json.as_object_mut().unwrap().remove("mana_enabled");
        assert!(!serde_json::from_value::<RoleBotMatchPlan>(json).unwrap().mana_enabled);
    }

    #[test]
    fn summoner_loadouts_compile_into_match_config_without_a_default_spell() {
        let legacy = plan();
        assert!(legacy.compile(1, SimulationTickProfile::Production60Hz).unwrap().0.summoner_loadouts.is_empty());
        assert!(!serde_json::to_string(&legacy).unwrap().contains("summoner_loadouts"));
        let mut equipped = legacy.clone();
        equipped.summoner_loadouts.insert(10, [Some("training_point_blink_v1".into()), None]);
        equipped.summoner_loadouts.insert(20, [None, Some("training_self_heal_v1".into())]);
        let config = equipped.compile(1, SimulationTickProfile::Production60Hz).unwrap().0;
        assert_eq!(config.summoner_loadouts, equipped.summoner_loadouts);
        equipped.summoner_loadouts.insert(99, [None, None]);
        assert!(equipped.compile(1, SimulationTickProfile::Production60Hz).unwrap_err().contains("WrongOwner"));
        equipped.summoner_loadouts.remove(&99);
        equipped.summoner_loadouts.insert(10, [Some("no-such-spell".into()), None]);
        assert!(equipped.compile(1, SimulationTickProfile::Production60Hz).unwrap_err().contains("UnknownSpell"));
        equipped.summoner_loadouts.insert(10, [Some("training_self_heal_v1".into()), Some("training_self_heal_v1".into())]);
        assert!(equipped.compile(1, SimulationTickProfile::Production60Hz).unwrap_err().contains("DuplicateSpell"));
    }

    #[test]
    fn mana_recovery_plan_requires_explicit_rules() {
        let mut p=plan();
        p.ability_policies=vec![BotAbilityPolicy {ability:"lumen_touch".into(),
            intent:BotAbilityIntent::SelfRecovery {below_hp_per_mille:600,
                below_mana_per_mille:500,restore_key:"mana_restore".into()}}];
        assert!(p.compile(1,SimulationTickProfile::Production60Hz).unwrap_err().contains("explicit mana_enabled"));
        p.mana_enabled=true;
        assert!(p.compile(1,SimulationTickProfile::Production60Hz).is_ok());
        p.ability_policies[0]=BotAbilityPolicy {ability:"ranger_patch".into(),
            intent:BotAbilityIntent::SelfManaRegeneration {below_hp_per_mille:600,below_mana_per_mille:500,
                rate_key:"mana_buff_value".into(),duration_key:"mana_buff_duration".into()}};
        assert!(p.compile(1,SimulationTickProfile::Production60Hz).is_ok());
        p.mana_enabled=false;
        assert!(p.compile(1,SimulationTickProfile::Production60Hz).is_err());
    }

    #[test]
    fn role_bot_items_plan_defaults_and_compiled_validation() {
        let mut p=plan();
        let mut json=serde_json::to_value(&p).unwrap();json.as_object_mut().unwrap().remove("item_builds");
        let legacy:RoleBotMatchPlan=serde_json::from_value(json).unwrap();
        assert!(legacy.compile(1,SimulationTickProfile::Production60Hz).unwrap().1.item_builds.is_empty());
        p.item_builds=vec![BotItemBuild {role:BotRole::Carry,items:vec!["moba_greatsword".into()],return_to_shop:None,active_use:None}];
        assert_eq!(p.compile(1,SimulationTickProfile::Production60Hz).unwrap().1.item_builds[0].items,["moba_greatsword"]);
        p.item_builds[0].items[0]="unknown".into();
        assert!(p.compile(1,SimulationTickProfile::Production60Hz).is_err());
    }
    /// Spotlight stage: one human and two held bot seats. The authority rules
    /// are unchanged; only the starting level, first-life placement, held
    /// seats and minion waves come from the recipe.
    fn showcase_plan() -> RoleBotMatchPlan {
        let player = |player_id:u32,team_id:u32,hero:&str,role:BotRole,bot:bool| RoleBotPlayerPlan {
            player_id,team_id,hero:hero.into(),role,lane:"mid".into(),bot };
        let mut p = plan();
        p.map_id = "three_lane_quick_training".into();
        p.players = vec![player(1,1,"saika_magoichi",BotRole::Mid,false),player(2,1,"murakami_yoshikiyo",BotRole::Support,true),
            player(6,2,"murakami_yoshikiyo",BotRole::Mid,true)];
        p.showcase = Some(ShowcaseConfig { start_level:6, minion_waves:false,
            placements:[(1,[2300,2400]),(2,[2050,2400]),(6,[2750,2400])].into(),
            hold_players:[2,6].into(), hold_hp_permille:5000 });
        p
    }
    #[test]
    fn showcase_stage_levels_places_and_holds_seats_without_role_decisions() {
        let (config,bots) = showcase_plan().compile(3,SimulationTickProfile::Production60Hz).unwrap();
        assert!(bots.assignments.is_empty(), "held seats get no role-bot decisions");
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        setup_single_lane_match(&mut world, config).unwrap();
        let state = world.read_resource::<MobaMatch>();
        let positions = world.read_storage::<Pos>();
        let props = world.read_storage::<CProperty>();
        let commands = world.read_storage::<HeroCommandQueue>();
        let mut hp = BTreeMap::new();
        for slot in &state.heroes {
            let entity = slot.entity.expect("spawned");
            let hero = world.read_storage::<Hero>().get(entity).unwrap().clone();
            assert_eq!(hero.level, 6, "player {}", slot.player_id);
            assert!(hero.skill_points >= 5, "a skill point per level gained");
            let at = match slot.player_id { 1 => [2300,2400], 2 => [2050,2400], _ => [2750,2400] };
            assert_eq!(positions.get(entity).unwrap().0, Vec2::new(Fixed64::from_i32(at[0]), Fixed64::from_i32(at[1])));
            let held = slot.player_id != 1;
            assert_eq!(commands.get(entity).and_then(|queue| queue.active), held.then_some(HeroCommand::HoldPosition));
            hp.insert(slot.player_id, props.get(entity).unwrap().mhp);
        }
        let stats = omoba_template_ids::active_hero_stats(omoba_template_ids::hero_by_name("murakami_yoshikiyo").unwrap()).unwrap();
        assert!(hp[&2] > Fixed64::from_i32(stats.base_hp * 4), "held seat hit points scaled by 5000 per mille: {:?}", hp[&2]);
    }
    #[test]
    fn showcase_stage_rejects_human_holds_and_out_of_roster_or_range_values() {
        let reject = |p:RoleBotMatchPlan| assert!(p.compile(1,SimulationTickProfile::Production60Hz)
            .ok().and_then(|(config,_)| { let pool = crate::runtime::StateInitializer::create_thread_pool();
                let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
                setup_single_lane_match(&mut world, config).ok() }).is_none());
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().hold_players.insert(1); reject(p);
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().hold_players.insert(9); reject(p);
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().placements.insert(9,[1,1]); reject(p);
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().start_level = 0; reject(p);
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().start_level = 26; reject(p);
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().hold_hp_permille = 999; reject(p);
        let json = serde_json::to_value(showcase_plan()).unwrap();
        assert_eq!(serde_json::from_value::<RoleBotMatchPlan>(json.clone()).unwrap().showcase, showcase_plan().showcase);
        let mut unknown = json; unknown["showcase"]["camera"] = serde_json::json!(1);
        assert!(serde_json::from_value::<RoleBotMatchPlan>(unknown).is_err(), "unknown stage fields fail closed");
        assert!(serde_json::to_value(plan()).unwrap().get("showcase").is_none(), "real plans never carry a stage");
    }
}
