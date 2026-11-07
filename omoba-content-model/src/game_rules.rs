//! Game mode rules of the original game (templates/game_rules.lua): hero magic resistance and
//! move speed caps, acquisition ranges, gold, experience and respawn, and the reward model (lane
//! minion, hero kill, structure and jungle comeback rewards, E187). The Rust build reads the
//! table; the runtime never hard-codes these numbers.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GameRules {
    pub hero: HeroRules,
    pub gold: GoldRules,
    pub experience: ExperienceRules,
    pub respawn: RespawnRules,
    pub acquisition: AcquisitionRules,
    pub lane_minions: Vec<LaneMinionReward>,
    pub lane_experience: LaneExperienceRules,
    pub hero_kill: HeroKillRules,
    pub buildings: BuildingRules,
    pub jungle: JungleRules,
    /// Team surrender vote (vscripts/computer_system/surrender.lua, E187). Absent: no surrender.
    #[serde(default)]
    pub surrender: Option<SurrenderRules>,
    /// Home defense aura (GameRulesStateChange.lua:439-447 home_aura and speed_up, E187). Absent: none.
    #[serde(default)]
    pub home_aura: Option<HomeAuraRules>,
    /// Structure defense (dummy.lua modifier_tower_armor / modifier_unit_armor / debuff_tower and
    /// war_magic.lua check_general_live, E187). Absent: none.
    #[serde(default)]
    pub structure_defense: Option<StructureDefenseRules>,
}

/// Percent added to 100 (MODIFIER_PROPERTY_INCOMING_DAMAGE_PERCENTAGE) by the attacker's kind.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AttackerPercents {
    pub hero: i32,
    /// Lane soldiers (npc_dota_creep_lane).
    pub lane_creep: i32,
    pub lane_tower: i32,
    pub general_tower: i32,
    /// A hero's summoned unit.
    pub summon: i32,
    /// Jungle camps and bosses (npc_dota_creep_neutral).
    pub neutral: i32,
    /// Any other source.
    pub other: i32,
}

impl AttackerPercents {
    pub fn all(&self) -> [i32; 7] {
        [self.hero, self.lane_creep, self.lane_tower, self.general_tower, self.summon, self.neutral, self.other]
    }
}

/// debuff_tower: enough enemies around a structure make it take more damage from heroes.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StructurePressureRules {
    pub radius: u32,
    /// Enemy units without an owner (not heroes, not summons) needed within `radius`.
    pub min_unowned_units: u32,
    /// Enemy heroes needed within `radius`.
    pub min_heroes: u32,
    /// Percent added to the damage the structure takes from heroes.
    pub hero_bonus_percent: i32,
}

/// The original game's structure defense (E187).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StructureDefenseRules {
    /// modifier_tower_armor on lane and general towers.
    pub tower_armor: AttackerPercents,
    /// modifier_unit_armor on lane and general towers.
    pub unit_armor: AttackerPercents,
    /// debuff_tower on every structure.
    pub pressure: StructurePressureRules,
    /// The main base cannot be damaged while a friendly general tower stands within this distance.
    pub base_general_radius: u32,
}

impl StructureDefenseRules {
    pub fn validate(&self) -> Result<(), String> {
        let p = &self.pressure;
        if self.tower_armor.all().iter().chain(self.unit_armor.all().iter()).chain([&p.hero_bonus_percent])
            .any(|v| !(-100..=1_000).contains(v))
            || !(1..=10_000).contains(&p.radius) || p.min_unowned_units > 1_000 || p.min_heroes > 1_000
            || !(1..=10_000).contains(&self.base_general_radius) {
            return Err("game_rules.structure_defense needs percents -100..=1000, radii 1..=10000 and unit \
                counts <= 1000".into());
        }
        Ok(())
    }
}

/// The original game's home defense (E187): from `start_seconds` of play, friendly heroes near
/// their own main base regenerate, gain armor and magic resistance, and move at a fixed speed.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HomeAuraRules {
    /// Playing time (after warmup) when both bases start the aura.
    pub start_seconds: u32,
    /// home_aura: heroes within this distance of their main base.
    pub radius: u32,
    /// Percent of max HP regenerated per second (MODIFIER_PROPERTY_HEALTH_REGEN_PERCENTAGE).
    pub hp_regen_max_percent: u32,
    pub armor: u32,
    /// Percent points, multiplied like MODIFIER_PROPERTY_MAGICAL_RESISTANCE_BONUS.
    pub magic_resistance: u32,
    /// speed_up: heroes within this distance move at `speed_up_move_speed` (MOVESPEED_ABSOLUTE).
    pub speed_up_radius: u32,
    pub speed_up_move_speed: u32,
    /// speed_up lasts this long after its last refresh (modifier_speed_up Duration).
    pub speed_up_seconds: u32,
    /// speed_up is refreshed on heroes inside its radius this often (ThinkInterval).
    pub speed_up_refresh_ms: u32,
}

impl HomeAuraRules {
    pub fn validate(&self) -> Result<(), String> {
        if self.start_seconds > 7_200 || ![self.radius, self.speed_up_radius].iter().all(|r| (1..=10_000).contains(r))
            || self.hp_regen_max_percent > 100 || self.armor > 1_000 || self.magic_resistance > 100
            || !(1..=10_000).contains(&self.speed_up_move_speed)
            || !(1..=600).contains(&self.speed_up_seconds) || !(1..=60_000).contains(&self.speed_up_refresh_ms) {
            return Err("game_rules.home_aura needs start_seconds <= 7200, radii 1..=10000, regeneration and magic \
                resistance <= 100 percent, armor <= 1000, a 1..=10000 move speed, a 1..=600 s speed duration and a \
                1..=60000 ms refresh".into());
        }
        Ok(())
    }
}

/// One lane minion's reward, keyed by its moba_maps.lua minion_wave id.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LaneMinionReward {
    pub id: String,
    /// Shared with every enemy hero in `lane_experience.radius` (scaled by their count).
    pub xp: u32,
    /// The last hitter's gold.
    pub gold: u32,
    /// Extra XP to the killing hero.
    pub killer_xp: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LaneExperienceRules {
    pub radius: u32,
    /// Percent of the minion's XP each hero gets when 1, 2, ... heroes are in range; more get none.
    pub percent_by_heroes: Vec<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HeroKillRules {
    pub xp_radius: u32,
    pub xp_per_victim_level: u32,
    /// Percent by (victim level - recipient level) 0, 1, 2, ...; the last entry covers larger gaps.
    pub xp_percent_by_level_gap: Vec<u32>,
    pub killer_lower_level_xp_per_victim_level: u32,
    pub kill_gold: u32,
    pub extra_gold: u32,
    pub streak_threshold: u32,
    pub streak_gold: u32,
    pub max_streak_gold: u32,
    pub death_gold: u32,
    pub max_death_gold: u32,
    pub multi_kill_seconds: u32,
    pub multi_kill_gold: u32,
    pub hoard_threshold: u32,
    pub hoard_step: u32,
    pub hoard_base_steps: u32,
    pub hoard_gold_per_step: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BuildingRules {
    pub tower: u32,
    pub inhibitor: u32,
    pub base: u32,
    pub nexus_tower: u32,
    pub npc_kill_radius: u32,
    pub nexus_tower_ally_radius: u32,
    pub nexus_tower_ally_gold: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct JungleRules {
    pub comeback_xp_percent_per_level: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HeroRules {
    /// Percent.
    pub base_magic_resistance: u32,
    /// Percent.
    pub max_magic_resistance: u32,
    pub max_move_speed: u32,
    pub acquisition_range: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GoldRules {
    pub starting: u32,
    pub passive_start_seconds: u32,
    pub passive_interval_seconds: u32,
    pub passive_amount: u32,
    pub periodic_interval_seconds: u32,
    pub periodic_amount: u32,
    pub sell_refund_percent: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExperienceRules {
    pub max_level: u32,
    /// Total experience to reach level `index + 1`; the first entry is 0.
    pub total_to_reach: Vec<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RespawnRules {
    /// Seconds by hero level (index 0 is level 1).
    pub hero_seconds_by_level: Vec<u32>,
    pub beyond_table_seconds: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AcquisitionRules {
    pub tower: u32,
    pub general_tower: u32,
    pub ashigaru: u32,
    pub archer: u32,
    pub gunner: u32,
    pub cavalry: u32,
}

/// Team surrender vote (vscripts/computer_system/surrender.lua, E187).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SurrenderRules {
    /// Playing time (after warmup, paused time excluded) before the first vote is accepted.
    pub available_after_seconds: u32,
    pub vote_threshold: SurrenderThreshold,
}

/// Votes needed out of `n` counted players: ceil((n * player_numerator + bias_numerator) / denominator),
/// which writes the source's ceil(n * 0.5 + 0.1) exactly in integers.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SurrenderThreshold {
    pub player_numerator: u32,
    pub bias_numerator: u32,
    pub denominator: u32,
}

/// Largest team the surrender threshold is validated for.
pub const MAX_SURRENDER_TEAM_SIZE: u32 = 16;

impl SurrenderThreshold {
    pub fn votes_needed(&self, players: u32) -> u32 {
        let numerator = u64::from(players) * u64::from(self.player_numerator) + u64::from(self.bias_numerator);
        numerator.div_ceil(u64::from(self.denominator.max(1))) as u32
    }
}

impl SurrenderRules {
    pub fn validate(&self) -> Result<(), String> {
        let t = &self.vote_threshold;
        if self.available_after_seconds > 7_200 || !(1..=1_000).contains(&t.denominator)
            || t.player_numerator == 0 || t.player_numerator > t.denominator || t.bias_numerator >= t.denominator
            || (1..=MAX_SURRENDER_TEAM_SIZE).any(|n| !(1..=n).contains(&t.votes_needed(n))) {
            return Err(format!("game_rules.surrender needs available_after_seconds <= 7200 and a threshold of 1..=n votes \
                for every team of 1..={MAX_SURRENDER_TEAM_SIZE} players"));
        }
        Ok(())
    }
}

/// Highest hero level the runtime supports.
pub const MAX_HERO_LEVEL: u32 = 25;

impl GameRules {
    pub fn validate(&self) -> Result<(), String> {
        let hero = &self.hero;
        if hero.base_magic_resistance > hero.max_magic_resistance || hero.max_magic_resistance > 100 {
            return Err("game_rules.hero magic resistance must be base <= max <= 100 percent".into());
        }
        if !(100..=10_000).contains(&hero.max_move_speed) || !(1..=10_000).contains(&hero.acquisition_range) {
            return Err("game_rules.hero max_move_speed and acquisition_range are out of range".into());
        }
        let gold = &self.gold;
        if gold.passive_interval_seconds == 0 || gold.periodic_interval_seconds == 0 || gold.sell_refund_percent > 100
            || [gold.starting, gold.passive_amount, gold.periodic_amount].iter().any(|v| *v > 1_000_000) {
            return Err("game_rules.gold needs positive intervals, amounts <= 1000000 and a refund <= 100 percent".into());
        }
        let xp = &self.experience;
        if xp.max_level == 0 || xp.max_level > MAX_HERO_LEVEL || xp.total_to_reach.len() != xp.max_level as usize
            || xp.total_to_reach.first() != Some(&0) || xp.total_to_reach.windows(2).any(|w| w[1] <= w[0]) {
            return Err(format!("game_rules.experience needs max_level <= {MAX_HERO_LEVEL} increasing totals from 0, one per level"));
        }
        let respawn = &self.respawn;
        if respawn.hero_seconds_by_level.is_empty() || respawn.hero_seconds_by_level.len() > MAX_HERO_LEVEL as usize
            || respawn.hero_seconds_by_level.iter().chain([&respawn.beyond_table_seconds]).any(|s| !(1..=600).contains(s)) {
            return Err("game_rules.respawn needs 1..=600 seconds for at most 25 levels".into());
        }
        let a = &self.acquisition;
        if [a.tower, a.general_tower, a.ashigaru, a.archer, a.gunner, a.cavalry].iter().any(|r| !(1..=10_000).contains(r)) {
            return Err("game_rules.acquisition ranges must be 1..=10000".into());
        }
        let ids: std::collections::BTreeSet<_> = self.lane_minions.iter().map(|m| m.id.as_str()).collect();
        if ids.len() != self.lane_minions.len() || self.lane_minions.iter().any(|m| m.id.is_empty()
            || [m.xp, m.gold, m.killer_xp].iter().any(|v| *v > 100_000)) {
            return Err("game_rules.lane_minions needs distinct ids and rewards <= 100000".into());
        }
        let lane = &self.lane_experience;
        if !(1..=10_000).contains(&lane.radius) || lane.percent_by_heroes.is_empty() || lane.percent_by_heroes.len() > 10
            || lane.percent_by_heroes.iter().any(|p| *p > 1000) {
            return Err("game_rules.lane_experience needs a 1..=10000 radius and 1..=10 percents <= 1000".into());
        }
        let kill = &self.hero_kill;
        if !(1..=10_000).contains(&kill.xp_radius) || kill.xp_percent_by_level_gap.is_empty()
            || kill.xp_percent_by_level_gap.len() > MAX_HERO_LEVEL as usize || kill.xp_percent_by_level_gap.iter().any(|p| *p > 10_000)
            || !(1..=3600).contains(&kill.multi_kill_seconds) || kill.hoard_step == 0 || kill.max_death_gold > kill.kill_gold + kill.extra_gold
            || [kill.xp_per_victim_level, kill.killer_lower_level_xp_per_victim_level, kill.kill_gold, kill.extra_gold, kill.streak_threshold,
                kill.streak_gold, kill.max_streak_gold, kill.death_gold, kill.max_death_gold, kill.multi_kill_gold, kill.hoard_threshold,
                kill.hoard_step, kill.hoard_base_steps, kill.hoard_gold_per_step].iter().any(|v| *v > 1_000_000) {
            return Err("game_rules.hero_kill needs radii 1..=10000, a 1..=3600 s multi-kill window, a positive hoard step, \
                a death reduction no larger than the kill gold and values <= 1000000".into());
        }
        let b = &self.buildings;
        if [b.npc_kill_radius, b.nexus_tower_ally_radius].iter().any(|r| !(1..=10_000).contains(r))
            || [b.tower, b.inhibitor, b.base, b.nexus_tower, b.nexus_tower_ally_gold].iter().any(|v| *v > 1_000_000) {
            return Err("game_rules.buildings needs radii 1..=10000 and bounties <= 1000000".into());
        }
        if self.jungle.comeback_xp_percent_per_level > 1000 {
            return Err("game_rules.jungle comeback percent must be <= 1000".into());
        }
        if let Some(surrender) = &self.surrender { surrender.validate()?; }
        if let Some(home_aura) = &self.home_aura { home_aura.validate()?; }
        if let Some(defense) = &self.structure_defense { defense.validate()?; }
        Ok(())
    }

    /// Experience needed to go from `level` to `level + 1`; None at the cap.
    pub fn experience_to_next(&self, level: u32) -> Option<u32> {
        let totals = &self.experience.total_to_reach;
        let index = usize::try_from(level).ok()?;
        totals.get(index)?.checked_sub(*totals.get(index.checked_sub(1)?)?)
    }

    /// Respawn seconds of a hero of `level`.
    pub fn respawn_seconds(&self, level: u32) -> u32 {
        level.checked_sub(1).and_then(|i| self.respawn.hero_seconds_by_level.get(i as usize).copied())
            .unwrap_or(self.respawn.beyond_table_seconds)
    }
}

#[cfg(test)]
#[test]
fn experience_and_respawn_follow_the_tables() {
    let rules: GameRules = serde_json::from_value(serde_json::json!({
        "hero": {"base_magic_resistance": 30, "max_magic_resistance": 70, "max_move_speed": 700, "acquisition_range": 800},
        "gold": {"starting": 2000, "passive_start_seconds": 45, "passive_interval_seconds": 2, "passive_amount": 30,
            "periodic_interval_seconds": 60, "periodic_amount": 150, "sell_refund_percent": 75},
        "experience": {"max_level": 3, "total_to_reach": [0, 220, 485]},
        "respawn": {"hero_seconds_by_level": [1, 2], "beyond_table_seconds": 140},
        "acquisition": {"tower": 900, "general_tower": 1100, "ashigaru": 1200, "archer": 600, "gunner": 700, "cavalry": 600},
        "lane_minions": [{"id": "lane_melee", "xp": 30, "gold": 25, "killer_xp": 0}],
        "lane_experience": {"radius": 1400, "percent_by_heroes": [100, 70, 60, 50, 40]},
        "hero_kill": {"xp_radius": 2000, "xp_per_victim_level": 30, "xp_percent_by_level_gap": [50, 100, 200, 300, 400],
            "killer_lower_level_xp_per_victim_level": 20, "kill_gold": 100, "extra_gold": 300, "streak_threshold": 2,
            "streak_gold": 100, "max_streak_gold": 400, "death_gold": 50, "max_death_gold": 200, "multi_kill_seconds": 15,
            "multi_kill_gold": 50, "hoard_threshold": 3000, "hoard_step": 1000, "hoard_base_steps": 2, "hoard_gold_per_step": 50},
        "buildings": {"tower": 150, "inhibitor": 300, "base": 50, "nexus_tower": 1300, "npc_kill_radius": 1500,
            "nexus_tower_ally_radius": 2000, "nexus_tower_ally_gold": 300},
        "jungle": {"comeback_xp_percent_per_level": 15},
    })).unwrap();
    rules.validate().unwrap();
    assert_eq!((rules.experience_to_next(1), rules.experience_to_next(2), rules.experience_to_next(3)), (Some(220), Some(265), None));
    assert_eq!((rules.respawn_seconds(1), rules.respawn_seconds(2), rules.respawn_seconds(3)), (1, 2, 140));
    let mut bad = rules.clone();
    bad.experience.total_to_reach = vec![0, 485, 220];
    assert!(bad.validate().is_err());
    let mut bad = rules.clone();
    bad.hero.base_magic_resistance = 80;
    assert!(bad.validate().is_err());
    // E187 reward model bounds.
    let mut bad = rules.clone();
    bad.lane_minions.push(bad.lane_minions[0].clone());
    assert!(bad.validate().is_err(), "duplicate minion id");
    let mut bad = rules.clone();
    bad.lane_experience.percent_by_heroes.clear();
    assert!(bad.validate().is_err());
    let mut bad = rules.clone();
    bad.hero_kill.hoard_step = 0;
    assert!(bad.validate().is_err());
    let mut bad = rules.clone();
    bad.hero_kill.max_death_gold = 401;
    assert!(bad.validate().is_err(), "a death reduction cannot take more than the kill gold");
    let mut bad = rules.clone();
    bad.buildings.npc_kill_radius = 0;
    assert!(bad.validate().is_err());
}

#[cfg(test)]
#[test]
fn surrender_threshold_is_the_sources_ceil_half_plus_a_tenth() {
    // vscripts/computer_system/surrender.lua:91 math.ceil(connected_players*0.5 + 0.1).
    let rules = SurrenderRules {
        available_after_seconds: 900,
        vote_threshold: SurrenderThreshold { player_numerator: 5, bias_numerator: 1, denominator: 10 },
    };
    rules.validate().unwrap();
    let needed: Vec<u32> = (1..=5).map(|n| rules.vote_threshold.votes_needed(n)).collect();
    assert_eq!(needed, [1, 2, 2, 3, 3]);
    for n in 1..=MAX_SURRENDER_TEAM_SIZE {
        assert_eq!(rules.vote_threshold.votes_needed(n), (f64::from(n) * 0.5 + 0.1).ceil() as u32);
    }
    let mut bad = rules.clone();
    bad.vote_threshold.player_numerator = 10; // ceil(n + 0.1) = n + 1 can never be reached.
    assert!(bad.validate().is_err());
    let mut bad = rules.clone();
    bad.vote_threshold.denominator = 0;
    assert!(bad.validate().is_err());
}

#[cfg(test)]
#[test]
fn home_aura_rules_are_optional_and_bounded() {
    // E187: GameRulesStateChange.lua:439-447 (600 s), library/common/dummy.lua:738-767 (1200 / 1300),
    // npc_abilities_custom_load.txt speed_up / home_aura (600, 10 %, 30, 30).
    let rules = HomeAuraRules { start_seconds: 600, radius: 1300, hp_regen_max_percent: 10, armor: 30,
        magic_resistance: 30, speed_up_radius: 1200, speed_up_move_speed: 600, speed_up_seconds: 8,
        speed_up_refresh_ms: 250 };
    rules.validate().unwrap();
    for case in 0..8 {
        let mut bad = rules.clone();
        match case {
            0 => bad.start_seconds = 7_201, 1 => bad.radius = 0, 2 => bad.speed_up_radius = 10_001,
            3 => bad.hp_regen_max_percent = 101, 4 => bad.magic_resistance = 101, 5 => bad.speed_up_seconds = 0,
            6 => bad.speed_up_refresh_ms = 0, _ => bad.speed_up_move_speed = 0,
        }
        assert!(bad.validate().is_err(), "case {case}");
    }
    let parsed: HomeAuraRules = serde_json::from_value(serde_json::json!({"start_seconds": 600, "radius": 1300,
        "hp_regen_max_percent": 10, "armor": 30, "magic_resistance": 30, "speed_up_radius": 1200,
        "speed_up_move_speed": 600, "speed_up_seconds": 8, "speed_up_refresh_ms": 250})).unwrap();
    assert_eq!(parsed, rules);
    assert!(serde_json::from_value::<HomeAuraRules>(serde_json::json!({"start_seconds": 600})).is_err());
}

#[cfg(test)]
#[test]
fn structure_defense_rules_are_optional_and_bounded() {
    // E187: dummy.lua:545-568 modifier_unit_armor, :604-639 modifier_tower_armor, :510-534 debuff_tower,
    // war_magic.lua:838-861 check_general_live.
    let percents = |hero, lane_creep, lane_tower, general_tower, summon, neutral, other|
        AttackerPercents { hero, lane_creep, lane_tower, general_tower, summon, neutral, other };
    let rules = StructureDefenseRules {
        tower_armor: percents(0, -60, 0, 0, -50, -60, -60),
        unit_armor: percents(0, 30, -75, 50, 0, 0, 0),
        pressure: StructurePressureRules { radius: 1100, min_unowned_units: 4, min_heroes: 1, hero_bonus_percent: 40 },
        base_general_radius: 500,
    };
    rules.validate().unwrap();
    for case in 0..5 {
        let mut bad = rules.clone();
        match case {
            0 => bad.tower_armor.summon = -101, 1 => bad.unit_armor.general_tower = 1_001, 2 => bad.pressure.radius = 0,
            3 => bad.pressure.hero_bonus_percent = -101, _ => bad.base_general_radius = 0,
        }
        assert!(bad.validate().is_err(), "case {case}");
    }
    let parsed: StructureDefenseRules = serde_json::from_value(serde_json::json!({
        "tower_armor": {"hero": 0, "lane_creep": -60, "lane_tower": 0, "general_tower": 0, "summon": -50, "neutral": -60, "other": -60},
        "unit_armor": {"hero": 0, "lane_creep": 30, "lane_tower": -75, "general_tower": 50, "summon": 0, "neutral": 0, "other": 0},
        "pressure": {"radius": 1100, "min_unowned_units": 4, "min_heroes": 1, "hero_bonus_percent": 40},
        "base_general_radius": 500})).unwrap();
    assert_eq!(parsed, rules);
    assert!(serde_json::from_value::<StructureDefenseRules>(serde_json::json!({"base_general_radius": 500})).is_err());
}
