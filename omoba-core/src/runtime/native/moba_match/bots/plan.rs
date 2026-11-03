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
        p.ability_policies=vec![BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"lumen_touch".into(),
            intent:BotAbilityIntent::SelfRecovery {below_hp_per_mille:600,
                below_mana_per_mille:500,restore_key:"mana_restore".into()}}];
        assert!(p.compile(1,SimulationTickProfile::Production60Hz).unwrap_err().contains("explicit mana_enabled"));
        p.mana_enabled=true;
        assert!(p.compile(1,SimulationTickProfile::Production60Hz).is_ok());
        p.ability_policies[0]=BotAbilityPolicy {caster_status:None,caster_no_status:None,caster_summon:None,ability:"ranger_patch".into(),
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
            hold_players:[2,6].into(), strike_players:Default::default(), hold_hp_permille:5000, start_items:Default::default(), props:Vec::new(), casts:Vec::new(), patrols:Vec::new(), refill_players:Default::default() });
        p
    }
    /// Player 6 (omatsu) learns frost bolt to rank 2 and casts it at player 1.
    fn casting_plan(first_seconds:u32,every_seconds:u32) -> RoleBotMatchPlan {
        let mut p = showcase_plan();
        p.players[2].hero = "omatsu".into();
        p.showcase.as_mut().unwrap().casts = vec![ShowcaseCast { player:6, ability:"a34_frost_bolt".into(), rank:2,
            target:ShowcaseCastTarget::Player { player:1 }, first_seconds, when_near:None, every_seconds }];
        p
    }
    /// Player 6's inputs for think ticks, its hero set to `rank`, the clock at each of `seconds`.
    fn stand_in_actions(plan:&RoleBotMatchPlan,rank:i32,seconds:&[i32]) -> Vec<Option<PlayerInputEnum>> {
        let (config,bots) = plan.compile(3,SimulationTickProfile::Production60Hz).unwrap();
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        setup_single_lane_match(&mut world, config).unwrap();
        world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
        let entity = world.read_resource::<MobaMatch>().heroes.iter().find(|slot|slot.player_id==6).unwrap().entity.unwrap();
        if rank>0 { world.write_storage::<Hero>().get_mut(entity).unwrap().ability_levels.insert("a34_frost_bolt".into(),rank); }
        seconds.iter().map(|at| {
            world.write_resource::<MobaMatch>().elapsed = Fixed64::from_i32(*at);
            let tick = bots.think_interval_ticks;
            world.write_resource::<crate::comp::Tick>().0 = tick;
            crate::runtime::run_committed_visibility_wave_b(&mut world, tick, 0);
            super::role_bot_inputs(&world,&bots).unwrap().into_iter().find(|(player,_)|*player==6).and_then(|(_,input)|input.action)
        }).collect()
    }
    #[test]
    fn a_stand_in_learns_then_casts_on_its_schedule_only() {
        let every = casting_plan(10,15);
        let learning = stand_in_actions(&every,0,&[1]);
        assert!(matches!(learning[0], Some(PlayerInputEnum::UpgradeAbility(_))), "learns first: {learning:?}");
        let casts = stand_in_actions(&every,2,&[9,10,11,24,25,26,40]);
        let cast_at:Vec<bool> = casts.iter().map(|action|matches!(action,Some(PlayerInputEnum::CastAbility(_)))).collect();
        assert_eq!(cast_at, [false,true,false,false,true,false,true], "{casts:?}");
        // The named hero is the target, and between casts the seat holds as before.
        let target = { let (config,_) = every.compile(3,SimulationTickProfile::Production60Hz).unwrap();
            let pool = crate::runtime::StateInitializer::create_thread_pool();
            let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
            setup_single_lane_match(&mut world, config).unwrap();
            let id = world.read_resource::<MobaMatch>().heroes.iter().find(|slot|slot.player_id==1).unwrap().entity.unwrap().id(); id };
        assert!(matches!(&casts[1], Some(PlayerInputEnum::CastAbility(cast)) if cast.target_entity==Some(target) && cast.target_pos.is_none()));
        assert!(!casts.iter().any(|action|matches!(action,Some(PlayerInputEnum::MoveTo(_)|PlayerInputEnum::AttackTarget(_)))), "it never leaves its place: {casts:?}");
        // every_seconds 0 is one cast.
        let once = stand_in_actions(&casting_plan(10,0),2,&[10,25]);
        assert!(matches!(once[0],Some(PlayerInputEnum::CastAbility(_))) && !matches!(once[1],Some(PlayerInputEnum::CastAbility(_))), "{once:?}");
    }
    #[test]
    fn a_stand_in_whose_cast_is_running_gets_no_order() {
        use super::super::super::super::cast_windup::{AbilityWindups,Windup};
        let (config,bots) = casting_plan(10,15).compile(3,SimulationTickProfile::Production60Hz).unwrap();
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        setup_single_lane_match(&mut world, config).unwrap();
        world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
        let entity = world.read_resource::<MobaMatch>().heroes.iter().find(|slot|slot.player_id==6).unwrap().entity.unwrap();
        let mut windups = AbilityWindups::default();
        windups.0.insert(entity, Windup { skill_id:"a34_frost_bolt".into(), target:crate::runtime::native::scripting::SkillTarget::None,
            origin:Vec2::ZERO, remaining:Fixed64::ONE, rank:1, interrupt_on_damage:false, last_tick:0, anchored:true });
        world.insert(windups);
        world.write_storage::<HeroCommandQueue>().insert(entity, HeroCommandQueue::default()).unwrap();
        let tick = bots.think_interval_ticks;
        world.write_resource::<crate::comp::Tick>().0 = tick;
        crate::runtime::run_committed_visibility_wave_b(&mut world, tick, 0);
        assert!(!super::role_bot_inputs(&world,&bots).unwrap().iter().any(|(player,_)|*player==6));
    }
    #[test]
    fn showcase_casts_are_rejected_off_the_stage_and_absent_from_real_plans() {
        let refused = |p:RoleBotMatchPlan| { let (config,_) = p.compile(1,SimulationTickProfile::Production60Hz).unwrap();
            let pool = crate::runtime::StateInitializer::create_thread_pool();
            let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
            setup_single_lane_match(&mut world, config).is_err() };
        assert!(!refused(casting_plan(10,15)), "control");
        let edit = |change:&dyn Fn(&mut ShowcaseCast)| { let mut p = casting_plan(10,15); change(&mut p.showcase.as_mut().unwrap().casts[0]); p };
        assert!(refused(edit(&|cast| cast.player = 1)), "not a held seat");
        assert!(refused(edit(&|cast| cast.ability = "b20_tenacity".into())), "not in the seat's kit");
        assert!(refused(edit(&|cast| cast.rank = 0)), "rank 0");
        assert!(refused(edit(&|cast| cast.rank = 99)), "above its levels");
        assert!(refused(edit(&|cast| cast.target = ShowcaseCastTarget::Player { player:9 })), "target off the roster");
        assert!(refused(edit(&|cast| cast.target = ShowcaseCastTarget::Point { at:[600_000,0] })), "outside the world");
        assert!(refused(edit(&|cast| cast.first_seconds = 3601)), "past the longest stage");
        assert!(!refused(edit(&|cast| cast.when_near = Some([2300,2400,100]))), "a circle around the target player");
        assert!(refused(edit(&|cast| cast.when_near = Some([2300,2400,0]))), "an empty circle");
        assert!(refused(edit(&|cast| { cast.when_near = Some([2300,2400,100]); cast.target = ShowcaseCastTarget::Point { at:[1,1] }; })), "a circle watches a player");
        let mut unknown = serde_json::to_value(casting_plan(10,15)).unwrap();
        unknown["showcase"]["casts"][0]["damage"] = serde_json::json!(5);
        assert!(serde_json::from_value::<RoleBotMatchPlan>(unknown).is_err(), "unknown cast fields fail closed");
        assert!(serde_json::to_value(showcase_plan()).unwrap()["showcase"].get("casts").is_none(), "a stage without casts stays as it was");
    }
    /// Player 6 walks (2750,2000) to (2750,2800); it stands at the first.
    fn patrol_plan() -> RoleBotMatchPlan {
        let mut p = showcase_plan();
        let stage = p.showcase.as_mut().unwrap();
        stage.placements.insert(6,[2750,2000]);
        stage.patrols = vec![ShowcasePatrol { player:6, a:[2750,2000], b:[2750,2800] }];
        p
    }
    #[test]
    fn a_patrolling_stand_in_turns_at_each_end_and_a_push_or_a_strike_only_delays_its_leg() {
        let think = |plan:&RoleBotMatchPlan, steps:&mut dyn FnMut(&mut specs::World,specs::Entity,specs::Entity,usize)->bool| {
            let (config,bots) = plan.compile(3,SimulationTickProfile::Production60Hz).unwrap();
            let pool = crate::runtime::StateInitializer::create_thread_pool();
            let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
            setup_single_lane_match(&mut world, config).unwrap();
            world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
            let of = |world:&specs::World,id:u32| world.read_resource::<MobaMatch>().heroes.iter().find(|slot|slot.player_id==id).unwrap().entity.unwrap();
            let (seat,enemy) = (of(&world,6),of(&world,1));
            world.write_storage::<HeroCommandQueue>().insert(seat, HeroCommandQueue::default()).unwrap();
            let tick = bots.think_interval_ticks;
            world.write_resource::<crate::comp::Tick>().0 = tick;
            let mut out = Vec::new();
            for step in 0.. {
                if !steps(&mut world,seat,enemy,step) {break;}
                crate::runtime::run_committed_visibility_wave_b(&mut world, tick, 0);
                out.push(super::role_bot_inputs(&world,&bots).unwrap().into_iter().find(|(player,_)|*player==6).and_then(|(_,input)|input.action));
            }
            out
        };
        let at = |x:i32,y:i32| Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
        let move_to = |action:&Option<PlayerInputEnum>,y:i32| matches!(action,Some(PlayerInputEnum::MoveTo(order))
            if order.target==Some(Vec2I {x:Fixed64::from_i32(2750).raw() as i32,y:Fixed64::from_i32(y).raw() as i32}));
        let put = |world:&mut specs::World,seat:specs::Entity,to:Vec2,active:Option<HeroCommand>| {
            world.write_storage::<Pos>().get_mut(seat).unwrap().0 = to;
            world.write_storage::<HeroCommandQueue>().get_mut(seat).unwrap().active = active; };
        let walk = think(&patrol_plan(), &mut |world,seat,_,step| { match step {
            0 => {}
            // On the leg to b: no new order.
            1 => put(world,seat,at(2750,2400),Some(HeroCommand::MoveTo {pos:at(2750,2800)})),
            // Past halfway with the order gone (a cast took over): still b, not the nearer a or its place.
            2 => put(world,seat,at(2750,2700),None),
            // Pushed sideways, more than 60 off the line and off its place: still b.
            3 => put(world,seat,at(2950,2500),Some(HeroCommand::MoveTo {pos:at(2750,2800)})),
            // The move ended at b: turns round; then a, where it turns round again.
            4 => put(world,seat,at(2750,2800),None),
            5 => put(world,seat,at(2750,2000),None),
            // Stopped short of b only by the 60 another body can keep it off: turns too.
            6 => put(world,seat,at(2750,2750),None),
            _ => return false,
        } true });
        assert!(move_to(&walk[0],2800),"first to b: {walk:?}");
        assert!(walk[1].is_none(),"{walk:?}");
        assert!(move_to(&walk[2],2800),"{walk:?}");
        assert!(walk[3].is_none(),"{walk:?}");
        assert!(move_to(&walk[4],2000),"{walk:?}");
        assert!(move_to(&walk[5],2800),"{walk:?}");
        assert!(move_to(&walk[6],2000),"{walk:?}");
        // A striking seat attacks whoever comes within reach; with no one there it walks the same leg.
        let mut striking = patrol_plan();
        striking.showcase.as_mut().unwrap().strike_players.insert(6);
        let struck = think(&striking, &mut |world,seat,enemy,step| { match step {
            0 => put(world,seat,at(2750,2400),None),
            1 => world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = at(2750,2500),
            2 => world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = at(2300,2400),
            _ => return false,
        } true });
        assert!(move_to(&struck[0],2800),"{struck:?}");
        assert!(matches!(struck[1],Some(PlayerInputEnum::AttackTarget(_))),"{struck:?}");
        assert!(move_to(&struck[2],2800),"{struck:?}");
        // Without a patrol the same push is the old way back to its place.
        let mut still = patrol_plan();
        still.showcase.as_mut().unwrap().patrols.clear();
        let back = think(&still, &mut |world,seat,_,step| { match step {
            0 => put(world,seat,at(2750,2400),None),
            _ => return false,
        } true });
        assert!(move_to(&back[0],2000),"{back:?}");
    }
    #[test]
    fn showcase_patrols_are_rejected_off_the_stage_and_absent_from_real_plans() {
        let refused = |p:RoleBotMatchPlan| { let (config,_) = p.compile(1,SimulationTickProfile::Production60Hz).unwrap();
            let pool = crate::runtime::StateInitializer::create_thread_pool();
            let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
            setup_single_lane_match(&mut world, config).is_err() };
        assert!(!refused(patrol_plan()), "control");
        let edit = |change:&dyn Fn(&mut ShowcasePatrol)| { let mut p = patrol_plan(); change(&mut p.showcase.as_mut().unwrap().patrols[0]); p };
        assert!(refused(edit(&|patrol| patrol.player = 1)), "not a held seat");
        assert!(refused(edit(&|patrol| patrol.player = 9)), "off the roster");
        assert!(refused(edit(&|patrol| patrol.b = patrol.a)), "nowhere to walk");
        assert!(refused(edit(&|patrol| patrol.b = [600_000,0])), "outside the world");
        let mut twice = patrol_plan(); let stage = twice.showcase.as_mut().unwrap(); stage.patrols.push(stage.patrols[0].clone());
        assert!(refused(twice), "one walk per seat");
        let mut unknown = serde_json::to_value(patrol_plan()).unwrap();
        unknown["showcase"]["patrols"][0]["speed"] = serde_json::json!(5);
        assert!(serde_json::from_value::<RoleBotMatchPlan>(unknown).is_err(), "unknown patrol fields fail closed");
        assert!(serde_json::to_value(showcase_plan()).unwrap()["showcase"].get("patrols").is_none(), "a stage without patrols stays as it was");
    }
    #[test]
    fn a_refill_seat_is_full_again_only_out_of_a_living_enemy_strikers_reach() {
        let mut staged = showcase_plan();
        { let stage = staged.showcase.as_mut().unwrap();
          stage.strike_players = [6].into(); stage.refill_players = [1].into();
          stage.placements.insert(6,[2750,2400]); stage.placements.insert(2,[2300,2400]); }
        // The seat at (x,2400) on half life (or `life`) after one tick: whether it is full, and its life.
        struct Stage { world:specs::World, tick:u64, seat:specs::Entity, striker:specs::Entity }
        impl Stage {
            fn new(plan:&RoleBotMatchPlan) -> Self {
                let (config,_) = plan.compile(1,SimulationTickProfile::Production60Hz).unwrap();
                let pool = crate::runtime::StateInitializer::create_thread_pool();
                let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
                setup_single_lane_match(&mut world, config).unwrap();
                { let mut state = world.write_resource::<MobaMatch>(); state.phase = MobaMatchPhase::Playing; let warmup = state.config.warmup; state.elapsed = warmup; }
                world.write_resource::<crate::comp::DeltaTime>().0 = Fixed64::from_raw(16);
                let of = |world:&specs::World,id:u32| world.read_resource::<MobaMatch>().heroes.iter().find(|slot|slot.player_id==id).unwrap().entity.unwrap();
                let (seat,striker) = (of(&world,1),of(&world,6));
                { let mut properties = world.write_storage::<CProperty>(); let p = properties.get_mut(seat).unwrap(); p.mhp = Fixed64::from_i32(1000); }
                let tick = world.read_resource::<crate::comp::Tick>().0;
                Stage { tick, world, seat, striker }
            }
            fn at(&mut self,x:i32,life:i32) -> Fixed64 {
                self.world.write_storage::<Pos>().get_mut(self.seat).unwrap().0 = Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(2400));
                self.world.write_storage::<CProperty>().get_mut(self.seat).unwrap().hp = Fixed64::from_i32(life);
                self.tick += 1;
                self.world.write_resource::<crate::comp::Tick>().0 = self.tick;
                begin_moba_match_tick(&mut self.world);
                self.world.read_storage::<CProperty>().get(self.seat).unwrap().hp
            }
        }
        let full = Fixed64::from_i32(1000);
        let mut stage = Stage::new(&staged);
        // Next to the striker (2750): up to and including its reach no refill, one past it a refill.
        assert!(stage.at(2650,500) < full, "within reach");
        assert!(stage.at(2450,500) < full, "at the reach");
        assert_eq!(stage.at(2449,500), full, "one past the reach");
        // The allied seat 2 stands right there; it would not strike this seat, were it a striker.
        { let mut allied = staged.clone(); allied.showcase.as_mut().unwrap().hold_players.insert(2); allied.showcase.as_mut().unwrap().strike_players = [2].into();
          assert_eq!(Stage::new(&allied).at(2300,500), full, "an ally's reach is no danger"); }
        // The dead: nothing brings back a life at zero, and a lethal hit waiting for its death stays.
        assert_eq!(stage.at(2000,0), Fixed64::ZERO, "the dead stay dead");
        { let mut state = stage.world.write_resource::<MobaMatch>(); state.heroes.iter_mut().find(|slot|slot.player_id==1).unwrap().lethal_pending = true; }
        assert!(stage.at(2000,500) < full, "a lethal hit pending");
        { let mut state = stage.world.write_resource::<MobaMatch>(); state.heroes.iter_mut().find(|slot|slot.player_id==1).unwrap().lethal_pending = false; }
        // A dead striker is no danger.
        let striker = stage.striker;
        stage.world.write_storage::<CProperty>().get_mut(striker).unwrap().hp = Fixed64::ZERO;
        assert_eq!(stage.at(2650,500), full, "a dead striker");
        // A striker off its place: danger is by its place, the reach it strikes in, not where it stands.
        let mut stage = Stage::new(&staged);
        let striker = stage.striker;
        let off = |stage:&mut Stage,x:i32| stage.world.write_storage::<Pos>().get_mut(striker).unwrap().0 = Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(2400));
        off(&mut stage,2000);
        assert_eq!(stage.at(2000,500), full, "next to a striker walking back, out of its place's reach");
        assert!(stage.at(2650,500) < full, "in its place's reach, the striker elsewhere");
        // Paused, or off the Playing phase: nothing.
        let mut stage = Stage::new(&staged);
        stage.world.write_resource::<GamePause>().is_paused = true;
        assert!(stage.at(2000,500) < full, "paused");
        stage.world.write_resource::<GamePause>().is_paused = false;
        stage.world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Warmup;
        assert!(stage.at(2000,500) < full, "warmup");
        // A stage that does not name the seat leaves it alone.
        let mut unnamed = staged.clone(); unnamed.showcase.as_mut().unwrap().refill_players.clear();
        assert!(Stage::new(&unnamed).at(2000,500) < full, "not named");
        assert!(Stage::new(&showcase_plan()).at(2000,500) < full, "a stage without it");
    }
    #[test]
    fn a_strike_seat_measures_its_reach_from_its_place_and_lets_go_outside_it() {
        let mut p = showcase_plan();
        { let stage = p.showcase.as_mut().unwrap(); stage.strike_players = [6].into(); stage.placements.insert(6,[2750,2400]); }
        let (config,bots) = p.compile(3,SimulationTickProfile::Production60Hz).unwrap();
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        setup_single_lane_match(&mut world, config).unwrap();
        world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
        let of = |world:&specs::World,id:u32| world.read_resource::<MobaMatch>().heroes.iter().find(|slot|slot.player_id==id).unwrap().entity.unwrap();
        let (seat,hero) = (of(&world,6),of(&world,1));
        world.write_storage::<HeroCommandQueue>().insert(seat, HeroCommandQueue::default()).unwrap();
        let tick = bots.think_interval_ticks;
        world.write_resource::<crate::comp::Tick>().0 = tick;
        let mut order = |seat_x:i32,hero_x:i32| {
            let at = |x:i32| Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(2400));
            world.write_storage::<Pos>().get_mut(seat).unwrap().0 = at(seat_x);
            world.write_storage::<Pos>().get_mut(hero).unwrap().0 = at(hero_x);
            crate::runtime::run_committed_visibility_wave_b(&mut world, tick, 0);
            super::role_bot_inputs(&world,&bots).unwrap().into_iter().find(|(player,_)|*player==6).and_then(|(_,input)|input.action)
        };
        let back = Vec2I {x:Fixed64::from_i32(2750).raw() as i32,y:Fixed64::from_i32(2400).raw() as i32};
        assert!(matches!(order(2750,2650),Some(PlayerInputEnum::AttackTarget(_))),"in reach, from its place");
        // Walked up for its attack, off its place: a hero still in reach of the place is still attacked...
        assert!(matches!(order(2500,2450),Some(PlayerInputEnum::AttackTarget(_))),"at the reach");
        assert!(matches!(order(2000,2650),Some(PlayerInputEnum::AttackTarget(_))),"far from the seat, in reach of the place");
        // ...a hero past it is let go, however close to the seat, and the seat walks back.
        assert!(matches!(order(2500,2449),Some(PlayerInputEnum::MoveTo(MoveTo {target:Some(target),..})) if target==back),"past the reach");
        // At its place it holds rather than attacks a hero out of reach.
        assert!(matches!(order(2750,2449),Some(PlayerInputEnum::HoldPosition(_))|None),"holds");
    }
    #[test]
    fn showcase_refills_are_rejected_off_the_roster_and_absent_from_real_plans() {
        let mut off = showcase_plan(); off.showcase.as_mut().unwrap().refill_players = [9].into();
        let (config,_) = off.compile(1,SimulationTickProfile::Production60Hz).unwrap();
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        assert!(setup_single_lane_match(&mut world, config).is_err(), "off the roster");
        assert!(serde_json::to_value(showcase_plan()).unwrap()["showcase"].get("refill_players").is_none(), "a stage without it stays as it was");
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
        // E390: a stand-in pushed off its place walks back to it, then holds there again.
        {
            let (config,bots) = showcase_plan().compile(3,SimulationTickProfile::Production60Hz).unwrap();
            let pool = crate::runtime::StateInitializer::create_thread_pool();
            let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
            setup_single_lane_match(&mut world, config).unwrap();
            world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
            let entity = world.read_resource::<MobaMatch>().heroes.iter().find(|s| s.player_id == 2).unwrap().entity.unwrap();
            let place = Vec2::new(Fixed64::from_i32(2050), Fixed64::from_i32(2400));
            let inputs_of = |world:&mut World| {
                let mut out = Vec::new();
                for tick in 1..=bots.think_interval_ticks {
                    world.write_resource::<crate::comp::Tick>().0 = tick;
                    crate::runtime::run_committed_visibility_wave_b(world, tick, 0);
                    out.extend(super::role_bot_inputs(world, &bots).unwrap().into_iter()
                        .filter(|(player,_)| *player == 2).filter_map(|(_,input)| input.action));
                }
                out
            };
            world.write_storage::<Pos>().get_mut(entity).unwrap().0 = place + Vec2::new(Fixed64::from_i32(450), Fixed64::ZERO);
            let pushed = inputs_of(&mut world);
            assert!(pushed.iter().any(|action| matches!(action, PlayerInputEnum::MoveTo(m)
                if m.target.as_ref().is_some_and(|t| t.x == place.x.raw() as i32 && t.y == place.y.raw() as i32))),
                "pushed off its place it walks back: {pushed:?}");
            world.write_storage::<Pos>().get_mut(entity).unwrap().0 = place;
            world.write_storage::<HeroCommandQueue>().insert(entity, HeroCommandQueue::default()).unwrap();
            let back = inputs_of(&mut world);
            assert!(back.iter().any(|action| matches!(action, PlayerInputEnum::HoldPosition(h) if !h.queued)),
                "back at its place it holds again: {back:?}");
            assert!(!back.iter().any(|action| matches!(action, PlayerInputEnum::MoveTo(_))), "no move once back");
        }
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
            // Every seat starts on Hold Position, the demonstrator too (it must not auto-attack a
            // stand-in before its renderer starts the demonstration).
            assert_eq!(commands.get(entity).and_then(|queue| queue.active), Some(HeroCommand::HoldPosition), "player {}", slot.player_id);
            hp.insert(slot.player_id, props.get(entity).unwrap().mhp);
        }
        let stats = omoba_template_ids::active_hero_stats(omoba_template_ids::hero_by_name("murakami_yoshikiyo").unwrap()).unwrap();
        assert!(hp[&2] > Fixed64::from_i32(stats.base_hp * 4), "held seat hit points scaled by 5000 per mille: {:?}", hp[&2]);
    }
    #[test]
    fn showcase_stage_starts_a_player_with_its_items_without_spending_gold() {
        let mut p = showcase_plan();
        p.showcase.as_mut().unwrap().start_items.insert(1,vec!["napalm_bomb".into(),"sake".into()]);
        let (config,_) = p.compile(3,SimulationTickProfile::Production60Hz).unwrap();
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        setup_single_lane_match(&mut world, config).unwrap();
        let hero = |id:u32| world.read_resource::<MobaMatch>().heroes.iter().find(|slot|slot.player_id==id).and_then(|slot|slot.entity).unwrap();
        let (one,two) = (hero(1),hero(2));
        let inventories = world.read_storage::<crate::comp::Inventory>();
        let held = |entity| inventories.get(entity).unwrap().slots.iter().flatten().map(|item|item.item_id.clone()).collect::<Vec<_>>();
        assert_eq!(held(one), vec!["napalm_bomb".to_string(),"sake".to_string()]);
        assert!(held(two).is_empty(), "only the named player gets items");
        let gold = world.read_storage::<crate::comp::Gold>();
        assert_eq!(gold.get(one).unwrap().0, gold.get(two).unwrap().0, "no gold spent");
    }
    #[test]
    fn showcase_start_items_may_list_more_purchases_than_slots_when_components_combine() {
        // E490: the item spotlight lists each combined item's components first (m73: 11 purchases).
        let list = ["staff","staff","spell_book","rockfall_book","shuriken","staff","staff","spell_book",
            "spirit_ingot","lesser_shield","the_poketto_of_meiso"];
        let mut p = showcase_plan();
        p.showcase.as_mut().unwrap().start_items.insert(1,list.iter().map(|id| id.to_string()).collect());
        let (config,_) = p.compile(3,SimulationTickProfile::Production60Hz).unwrap();
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        setup_single_lane_match(&mut world, config).unwrap();
        let entity = world.read_resource::<MobaMatch>().heroes.iter().find(|slot|slot.player_id==1).and_then(|slot|slot.entity).unwrap();
        let inventories = world.read_storage::<crate::comp::Inventory>();
        let held = inventories.get(entity).unwrap().slots.iter().flatten().map(|item|item.item_id.clone()).collect::<Vec<_>>();
        for shown in ["rockfall_book","shuriken","spell_book","the_poketto_of_meiso"] {
            assert!(held.iter().any(|id| id==shown), "{shown} in {held:?}");
        }
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
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().placements.insert(1,[-600_000,0]); reject(p);
        // Negative world coordinates are a valid stage (E574: the nobu tree stage stands at (-1400,-600)).
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().placements.insert(1,[-1400,-600]);
        let (config,_) = p.compile(1,SimulationTickProfile::Production60Hz).unwrap();
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        setup_single_lane_match(&mut world, config).unwrap();
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().start_level = 0; reject(p);
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().start_level = 26; reject(p);
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().hold_hp_permille = 999; reject(p);
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().start_items.insert(9,vec!["sake".into()]); reject(p);
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().start_items.insert(1,vec!["no_such_item".into()]); reject(p);
        // Seven separate items do not fit six slots; the grant refuses the stage (E490).
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().start_items.insert(1,vec!["sake".into();7]); reject(p);
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().start_items.insert(1,vec!["sake".into();65]); reject(p);
        // E630: a prop of a seat's team, inside the world, at most 16 of them.
        let prop = |team_of:u32,at:[i32;2]| ShowcaseProp { kind:ShowcasePropKind::Unit, team_of, at };
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().props.push(prop(9,[1,1])); reject(p);
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().props.push(prop(1,[600_000,0])); reject(p);
        let mut p = showcase_plan(); p.showcase.as_mut().unwrap().props = vec![prop(1,[1,1]);17]; reject(p);
        let json = serde_json::to_value(showcase_plan()).unwrap();
        assert_eq!(serde_json::from_value::<RoleBotMatchPlan>(json.clone()).unwrap().showcase, showcase_plan().showcase);
        let mut unknown = json; unknown["showcase"]["camera"] = serde_json::json!(1);
        assert!(serde_json::from_value::<RoleBotMatchPlan>(unknown).is_err(), "unknown stage fields fail closed");
        assert!(serde_json::to_value(plan()).unwrap().get("showcase").is_none(), "real plans never carry a stage");
        let mut unknown = serde_json::to_value(showcase_plan()).unwrap();
        unknown["showcase"]["props"] = serde_json::json!([{"kind":"unit","team_of":1,"at":[1,1],"hp":5}]);
        assert!(serde_json::from_value::<RoleBotMatchPlan>(unknown).is_err(), "unknown prop fields fail closed");
        let mut kind = serde_json::to_value(showcase_plan()).unwrap();
        kind["showcase"]["props"] = serde_json::json!([{"kind":"tree","team_of":1,"at":[1,1]}]);
        assert!(serde_json::from_value::<RoleBotMatchPlan>(kind).is_err(), "unknown prop kinds fail closed");
    }
    #[test]
    fn showcase_props_stand_still_as_allied_creeps_and_bodies_stay_for_the_match() {
        // E630: c19 奇襲 needs an allied non-hero unit at its point, b23 亡靈之契 a body near the caster.
        use crate::runtime::native::scripting::parallel_world_adapter::RetiredPositions;
        let mut p = showcase_plan();
        p.showcase.as_mut().unwrap().props = vec![
            ShowcaseProp { kind:ShowcasePropKind::Unit, team_of:1, at:[1900,2400] },
            ShowcaseProp { kind:ShowcasePropKind::Corpse, team_of:6, at:[2100,2400] },
        ];
        let (config,_) = p.compile(3,SimulationTickProfile::Production60Hz).unwrap();
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        let before = world.read_resource::<crate::comp::Tick>().0;
        setup_single_lane_match(&mut world, config).unwrap();
        let point = |x:i32,y:i32| Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y));
        let teams = world.read_resource::<MobaMatch>().config.teams;
        let prop = {
            use specs::Join;
            let creeps = world.read_storage::<crate::comp::Creep>();
            let positions = world.read_storage::<Pos>();
            let entities = world.entities();
            let found:Vec<_> = (&*entities,&creeps,&positions).join().filter(|(_,_,pos)| pos.0 == point(1900,2400)).map(|(e,_,_)| e).collect();
            assert_eq!(found.len(), 1, "one unit prop");
            found[0]
        };
        assert_eq!(world.read_storage::<crate::comp::Faction>().get(prop).unwrap().team_id, teams[0] as i32, "the demonstrator's team");
        assert!(world.read_resource::<MobaMatch>().lane_role(prop).is_none(), "never a lane unit");
        let body = { let units = world.read_storage::<crate::comp::Unit>(); let unit = units.get(prop).unwrap(); (unit.base_damage, unit.move_speed, unit.attack_range) };
        assert_eq!(body, (0, Fixed64::ZERO, Fixed64::ZERO), "no damage, speed or reach");
        assert_eq!(world.read_storage::<CProperty>().get(prop).unwrap().msd, Fixed64::ZERO);
        let bodies = |world:&World, since:u64| world.read_resource::<RetiredPositions>().units_in_range(point(2100,2400), Fixed64::from_i32(10), since);
        let found = bodies(&world, before);
        assert_eq!(found.len(), 1, "a body from the start");
        assert_eq!(found[0].1, teams[1] as i32, "of player 6's team");
        assert!(!world.entities().is_alive(found[0].0), "a body is a removed unit");
        // Played on for longer than a body is kept: the prop never moved and the body is still found.
        {
            let mut state = world.write_resource::<MobaMatch>();
            state.phase = MobaMatchPhase::Playing;
            let warmup = state.config.warmup;
            state.elapsed = warmup;
        }
        world.write_resource::<crate::comp::DeltaTime>().0 = Fixed64::from_raw(16);
        for tick in 1..=120 {
            world.write_resource::<crate::comp::Tick>().0 = before + tick;
            begin_moba_match_tick(&mut world);
        }
        assert_eq!(world.read_storage::<Pos>().get(prop).unwrap().0, point(1900,2400), "the unit prop stands still");
        // Two body times on: the last rewrite of the loop above is older than the look-back.
        let late = before + RetiredPositions::KEEP_TICKS * 2;
        let late = late - late % ShowcaseCorpses::REFRESH_TICKS;
        world.write_resource::<crate::comp::Tick>().0 = late;
        begin_moba_match_tick(&mut world);
        assert_eq!(bodies(&world, late - RetiredPositions::KEEP_TICKS).len(), 1, "the body is rewritten, never aged out");
        assert!(world.read_storage::<CProperty>().get(prop).unwrap().hp > Fixed64::ZERO, "the unit prop is alive");
    }
}
