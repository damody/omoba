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

/// Which heroes a plan may seat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeroAdmission {
    /// Every real match (selection, server role plans, headless batches): source heroes only.
    Playable,
    /// Automated tests and fixtures only: test-only heroes too. No match a player sees uses it.
    Fixture,
}

impl RoleBotMatchPlan {
    /// One hero per team, as in a LoL normal game; the two teams may share heroes. Humans keep
    /// their pick; a bot whose hero a human (or a lower player ID bot) of its team already holds
    /// gets the first playable hero, in catalog order, that its team does not use. Deterministic,
    /// so every host that loads the same plan seats the same heroes. Human-human clashes stay for
    /// the compile check to reject.
    pub fn resolve_bot_hero_conflicts(&mut self) {
        let mut order: Vec<usize> = (0..self.players.len()).collect();
        order.sort_by_key(|&index| (self.players[index].bot, self.players[index].player_id));
        let mut taken: BTreeSet<(u32, String)> = BTreeSet::new();
        for index in order {
            let team = self.players[index].team_id;
            if !self.players[index].bot || taken.insert((team, self.players[index].hero.clone())) {
                taken.insert((team, self.players[index].hero.clone()));
                continue;
            }
            if let Some(free) = omoba_template_ids::HERO_PLAYABLE_IDS.iter()
                .map(|id| id.as_str()).find(|hero| !taken.contains(&(team, (*hero).to_owned()))) {
                self.players[index].hero = free.to_owned();
                taken.insert((team, free.to_owned()));
            }
        }
    }

    /// Build atomically before allocating a World; changing map declaration
    /// order cannot silently reassign a role to a different named lane.
    pub fn compile(&self, seed: u64, profile: SimulationTickProfile)
        -> Result<(SingleLaneConfig, RoleBotConfig), String>
    {
        self.compile_with_tick_rate(seed,profile.ticks_per_game_second())
    }

    /// The server's configured clock is authoritative, including 90 Hz. Source heroes only.
    pub fn compile_with_tick_rate(&self, seed:u64, fps:u32)
        -> Result<(SingleLaneConfig, RoleBotConfig), String>
    {
        self.compile_with_admission(seed,fps,HeroAdmission::Playable)
    }

    /// Tests and fixtures that exercise a test-only hero's kit. Never a server or recipe path.
    pub fn compile_fixture(&self, seed: u64, profile: SimulationTickProfile)
        -> Result<(SingleLaneConfig, RoleBotConfig), String>
    {
        self.compile_with_admission(seed,profile.ticks_per_game_second(),HeroAdmission::Fixture)
    }

    pub fn compile_with_admission(&self, seed:u64, fps:u32, admission:HeroAdmission)
        -> Result<(SingleLaneConfig, RoleBotConfig), String>
    {
        if self.schema_version != 1 { return Err("unsupported role bot plan schema".into()); }
        abilities::validate_policies(&self.ability_policies).map_err(str::to_owned)?;
        if abilities::requires_mana(&self.ability_policies) && !self.mana_enabled {
            return Err("bot recovery intent requires explicit mana_enabled".into());
        }
        abilities::validate_learning(&self.ability_learning).map_err(str::to_owned)?;
        abilities::validate_learning(&self.expanded_learning()).map_err(str::to_owned)?;
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
        // A real match seats each hero at most once per team; fixtures may repeat a test kit.
        let mut team_heroes = BTreeSet::new();
        if admission == HeroAdmission::Playable {
            if let Some(player) = self.players.iter().find(|p| !team_heroes.insert((p.team_id, p.hero.as_str()))) {
                return Err(format!("hero '{}' is seated twice on team {}: one hero per team", player.hero, player.team_id));
            }
        }
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
            let Some(hero_id) = omoba_template_ids::hero_by_name(&player.hero)
                .filter(|id| omoba_template_ids::hero_stats(*id).is_some()) else {
                return Err(format!("unknown active compiled bot hero '{}'",player.hero));
            };
            if admission == HeroAdmission::Playable && !omoba_template_ids::hero_playable(hero_id) {
                return Err(format!("hero '{}' is test-only: a match seats only source heroes",player.hero));
            }
            let lane = map.lane_index(&player.lane)
                .ok_or_else(|| format!("unknown bot lane '{}' in '{}'",player.lane,map.id))?;
            if player.role == BotRole::Jungle && map.jungle_camps.is_empty() {
                return Err("jungle role requires compiled camps".into());
            }
            counts[side] += 1;
            // A held showcase seat is still this seat's bot controller (the server partitions
            // the roster into humans and controllers); the match config makes it issue nothing.
            let held = self.showcase.as_ref().is_some_and(|showcase| showcase.hold_players.contains(&player.player_id));
            if held && !player.bot { return Err("a showcase hold seat must be a bot".into()); }
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
            building_reward_rules:BuildingRewardRules::training(),
            tower_plating_rules:Some(TowerPlatingRules::training()),
            minion_growth_rules:MinionGrowthRules::for_map(map),
            seed,map_id:Some(map.id.into()),lane_length:Fixed64::from_i32(map.lane_length),
            tower_hp:Fixed64::from_i32(map.tower_hp),base_hp:Fixed64::from_i32(map.base_hp),
            hero_xp_scale_permille:map.hero_xp_scale_permille(),
            authored_minion_waves:true,wave_interval:Fixed64::from_i32(map.wave_interval_seconds as i32),
            first_wave_delay:Fixed64::from_i32(map.first_wave_seconds() as i32),
            players:[left.player_id,right.player_id],heroes:[left.hero,right.hero],additional_players:additional,
            base_recovery_enabled:self.sustain.is_some(),
            mana_enabled:self.mana_enabled,
            summoner_loadouts:self.summoner_loadouts.clone(),
            showcase:self.showcase.clone(),
            // A formal match plays by the original game's rules (templates/game_rules.lua), whose
            // reward model replaces the training building bounty and plating above (E187).
            game_rules:omoba_template_ids::GAME_RULES,
            ..Default::default()
        },RoleBotConfig { assignments,think_interval_ticks:u64::from(fps.div_ceil(self.think_hz)),
            ability_policies:self.ability_policies.clone(),ability_learning:self.expanded_learning(),sustain:self.sustain,
            item_builds:self.item_builds.clone() }))
    }

    /// Authored steps in order; a `{ability:"*",rank:0}` step becomes each seated hero's kit
    /// (heroes in player order, abilities the plan already authors skipped).
    fn expanded_learning(&self) -> Vec<abilities::BotAbilityLearningStep> {
        let authored:BTreeSet<String>=self.ability_learning.iter()
            .filter(|step|step.ability!=abilities::KIT_LEARNING).map(|step|step.ability.clone()).collect();
        let mut out=Vec::new();
        for step in &self.ability_learning {
            if step.ability!=abilities::KIT_LEARNING {out.push(step.clone());continue;}
            let mut seen=authored.clone();
            let mut roster=self.players.clone();
            roster.sort_by_key(|player|player.player_id);
            for player in roster {
                let Some(hero)=omoba_template_ids::hero_by_name(&player.hero) else {continue;};
                let kit=abilities::kit_learning(omoba_template_ids::hero_abilities(hero),&seen);
                for step in &kit {seen.insert(step.ability.clone());}
                out.extend(kit);
            }
        }
        out
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
                    hero:["murakami_yoshikiyo","date_masamune","saika_magoichi","oichi","asai_nagamasa"][index].into(),role,lane:lane.into(),bot:team != 1 || index != 0 });
            }
        }
        RoleBotMatchPlan { schema_version:1,map_id:"three_lane_training".into(),think_hz:5,mana_enabled:false,players,ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),summoner_loadouts:BTreeMap::new(),showcase:None }
    }
    #[test]
    fn one_hero_per_team_is_compiled_and_bot_clashes_resolve_deterministically() {
        let mut clash = plan();
        clash.players[1].hero = clash.players[0].hero.clone();
        let error = clash.compile(1,SimulationTickProfile::Production60Hz).unwrap_err();
        assert!(error.contains("seated twice on team 1"), "{error}");
        // Fixtures may repeat a test kit on one team.
        assert!(clash.compile_fixture(1,SimulationTickProfile::Production60Hz).is_ok());
        // The human (player 10) keeps its pick; the clashing bot moves to a free playable hero.
        let mut resolved = clash.clone();
        resolved.resolve_bot_hero_conflicts();
        assert_eq!(resolved.players[0].hero, clash.players[0].hero);
        assert_ne!(resolved.players[1].hero, clash.players[0].hero);
        assert!(resolved.compile(1,SimulationTickProfile::Production60Hz).is_ok());
        let mut again = clash.clone();
        again.resolve_bot_hero_conflicts();
        assert_eq!(again.players.iter().map(|p|p.hero.clone()).collect::<Vec<_>>(),
            resolved.players.iter().map(|p|p.hero.clone()).collect::<Vec<_>>(), "deterministic");
        // Cross-team repeats are allowed (the shipped mirror lineup).
        assert!(plan().players.iter().filter(|p|p.hero=="murakami_yoshikiyo").count() == 2);
        assert!(plan().compile(1,SimulationTickProfile::Production60Hz).is_ok());
    }

    #[test]
    fn role_bot_plan_takes_structure_hit_points_from_the_compiled_map() {
        let shipped = plan().compile(1,SimulationTickProfile::Production60Hz).unwrap().0;
        let standard = omoba_template_ids::moba_map_by_name(shipped.map_id.as_deref().unwrap()).unwrap();
        assert_eq!((shipped.tower_hp,shipped.base_hp),(Fixed64::from_i32(standard.tower_hp),Fixed64::from_i32(standard.base_hp)));
        let mut quick = plan();
        quick.map_id = "three_lane_quick_training".into();
        let map = omoba_template_ids::moba_map_by_name("three_lane_quick_training").unwrap();
        let config = quick.compile(1,SimulationTickProfile::Production60Hz).unwrap().0;
        assert_eq!((config.tower_hp,config.base_hp),(Fixed64::from_i32(map.tower_hp),Fixed64::from_i32(map.base_hp)));
        // The quick map has its own structure values, so the equality above proves they come from the map.
        assert!((map.tower_hp, map.base_hp) != (standard.tower_hp, standard.base_hp));
        // E096: the map's hero XP multiplier reaches the role-plan entry too (E055: every entry).
        assert_eq!(shipped.hero_xp_scale_permille,1000);
        assert_eq!(config.hero_xp_scale_permille,map.hero_xp_scale_permille());
        // E187: the map's first-wave delay reaches the match config (fixtures without a plan keep 0).
        assert_eq!(config.first_wave_delay,Fixed64::from_i32(map.first_wave_seconds() as i32));
        assert!(config.first_wave_delay>Fixed64::ZERO);
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
        assert_eq!(config.minion_growth_rules,MinionGrowthRules::for_map(
            omoba_template_ids::moba_map_by_name(config.map_id.as_deref().unwrap()).unwrap()));
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
        // "*" learns each seated kit: rank by rank, the ultimate first, nothing authored twice.
        let mut p=valid.clone(); p.players[1].hero="omatsu".into();
        p.ability_learning=vec![BotAbilityLearningStep {ability:"a34_frost_bolt".into(),rank:1},BotAbilityLearningStep {ability:"*".into(),rank:0}];
        let learning=p.compile(1,SimulationTickProfile::Production60Hz).unwrap().1.ability_learning;
        assert_eq!(learning[0].ability,"a34_frost_bolt");
        assert!(!learning[1..].iter().any(|step|step.ability=="a34_frost_bolt"),"authored abilities are not expanded again");
        let position=|ability:&str,rank:u8|learning.iter().position(|s|s.ability==ability&&s.rank==rank).unwrap();
        assert!(position("a34_rapid_cooling",1)<position("a34_iceberg_strike",1),"ultimate first within a rank");
        assert!(position("a34_iceberg_strike",1)<position("a34_rapid_cooling",2));
        assert!(learning.iter().any(|s|s.ability=="b29_ground_slam"),"every seated hero's kit");
        // A test-only hero never enters a real match; only fixture compiles admit it.
        let mut p=valid.clone(); p.players[0].hero="training_luminary".into();
        assert!(p.compile(1,SimulationTickProfile::Production60Hz).unwrap_err().contains("test-only"));
        assert!(p.compile_fixture(1,SimulationTickProfile::Production60Hz).is_ok());
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
    fn a_bot_hero_winding_up_its_own_cast_gets_no_new_order() {
        // E187: any new order cancels the owner's windup, so a 0.2 s windup re-ordered on every
        // 0.2 s think never released and never started its cooldown (the cast was chosen again).
        use super::super::super::super::cast_windup::{AbilityWindups,Windup};
        let decisions = |winding:bool| {
            let (config,bots) = plan().compile(3,SimulationTickProfile::Production60Hz).unwrap();
            let pool = crate::runtime::StateInitializer::create_thread_pool();
            let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
            setup_single_lane_match(&mut world, config).unwrap();
            world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
            let player = bots.assignments[0].player_id;
            if winding {
                let entity = world.read_resource::<MobaMatch>().heroes.iter()
                    .find(|slot|slot.player_id==player).unwrap().entity.unwrap();
                let mut windups = AbilityWindups::default();
                windups.0.insert(entity, Windup { skill_id:"murakami_any".into(),
                    target:crate::runtime::native::scripting::SkillTarget::None, origin:Vec2::ZERO,
                    remaining:Fixed64::ONE, rank:1, interrupt_on_damage:false, last_tick:0, anchored:true });
                world.insert(windups);
            }
            let mut players = BTreeSet::new();
            for tick in 1..=bots.think_interval_ticks * 3 {
                world.write_resource::<crate::comp::Tick>().0 = tick;
                crate::runtime::run_committed_visibility_wave_b(&mut world, tick, 0);
                players.extend(super::role_bot_inputs(&world, &bots).unwrap().into_iter().map(|(player,_)|player));
            }
            (player,players)
        };
        let (player,free) = decisions(false);
        assert!(free.contains(&player), "control: the seat orders when it is not casting");
        let (_,winding) = decisions(true);
        assert!(!winding.contains(&player), "no order while its own windup runs");
        assert!(!winding.is_empty(), "the other seats still decide");
    }
    #[test]
    fn showcase_stage_levels_places_and_holds_seats_without_role_decisions() {
        let (config,bots) = showcase_plan().compile(3,SimulationTickProfile::Production60Hz).unwrap();
        // Held seats keep their bot controller (roster partition: humans plus controllers)...
        assert_eq!(bots.assignments.iter().map(|a|a.player_id).collect::<Vec<_>>(), vec![2,6]);
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        setup_single_lane_match(&mut world, config).unwrap();
        // ...but make no role decision on any think tick, while the same seats unheld do decide
        // (published visibility, so an empty result is the hold, not missing perception).
        let decisions = |plan:RoleBotMatchPlan| {
            let (config,bots) = plan.compile(3,SimulationTickProfile::Production60Hz).unwrap();
            let pool = crate::runtime::StateInitializer::create_thread_pool();
            let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
            setup_single_lane_match(&mut world, config).unwrap();
            world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
            let mut players = BTreeSet::new();
            for tick in 1..=bots.think_interval_ticks * 3 {
                world.write_resource::<crate::comp::Tick>().0 = tick;
                crate::runtime::run_committed_visibility_wave_b(&mut world, tick, 0);
                players.extend(super::role_bot_inputs(&world, &bots).unwrap().into_iter().map(|(player,_)|player));
            }
            players
        };
        let mut unheld = showcase_plan();
        unheld.showcase.as_mut().unwrap().hold_players.clear();
        assert!(!decisions(unheld).is_empty(), "control: the same bot seats decide when not held");
        assert!(decisions(showcase_plan()).is_empty(), "held seats issue no input");
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
