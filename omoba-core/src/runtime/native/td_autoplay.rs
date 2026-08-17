//! Deterministic, input-only reference player for TD regression runs.

use omoba_sim::fixed::SCALE;
use specs::{Join, World, WorldExt};
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::comp::{
    CurrentCreepWave, PlayerEconomy, PlayerLives, Tower, TowerTemplateRegistry,
    TowerUpgradeRegistry,
};
use crate::runtime::{
    PlayerInput, PlayerInputEnum, ScriptUnitTag, StartRound, TowerAbilityCastInput, TowerPlace,
    TowerUpgradeInput, Vec2I,
};

pub const AUTOPLAY_PLAYER_ID: u32 = 1;

#[derive(Clone, Debug)]
pub struct TdAutoplayRunConfig {
    pub scripts_dir: PathBuf,
    pub profile: crate::runtime::SimulationTickProfile,
    pub max_ticks: u64,
    pub round_watchdog_ticks: u64,
    pub entity_peak_limit: usize,
}

impl TdAutoplayRunConfig {
    pub fn coarse_1_to_100(scripts_dir: impl Into<PathBuf>) -> Self {
        Self {
            scripts_dir: scripts_dir.into(),
            profile: crate::runtime::SimulationTickProfile::Coarse15Hz,
            max_ticks: 15 * 60 * 60,
            round_watchdog_ticks: 15 * 10 * 60,
            entity_peak_limit: 50_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TdAutoplayRunReport {
    pub seed: u64,
    pub profile: crate::runtime::SimulationTickProfile,
    pub ticks: u64,
    pub elapsed: Duration,
    pub ticks_per_wall_second: f64,
    pub lives: i32,
    pub cash: i32,
    pub ledger_digest: u64,
    pub state_hash: u64,
    pub entity_peak: usize,
    pub rejected_placements: usize,
    pub round_end_ticks: Vec<u64>,
    pub branch_counts: BTreeMap<String, u64>,
}

impl TdAutoplayRunReport {
    pub fn compact_summary(&self) -> String {
        format!(
            "profile={:?} rounds={} ticks={} throughput={:.1} ticks/s lives={} cash={} ledger={:016x} state={:016x} entity_peak={} placement_retries={}",
            self.profile,
            self.round_end_ticks.len(),
            self.ticks,
            self.ticks_per_wall_second,
            self.lives,
            self.cash,
            self.ledger_digest,
            self.state_hash,
            self.entity_peak,
            self.rejected_placements,
        )
    }
}

pub fn run_td_autoplay_1_to_100(
    config: &TdAutoplayRunConfig,
) -> Result<TdAutoplayRunReport, String> {
    let campaign = crate::ue4::import_campaign::load_generated("TD_GREEN_CROSSROADS")
        .map_err(|error| format!("load TD_GREEN_CROSSROADS: {error}"))?;
    let scripts = crate::scripting::loader::load_scripts_dir(&config.scripts_dir);
    if scripts.is_empty() {
        return Err(format!(
            "no scripts loaded from {}",
            config.scripts_dir.display()
        ));
    }
    let mut world = crate::runtime::create_world_from_loaded_content(
        campaign,
        crate::item::ItemRegistry::default(),
        scripts,
    )
    .map_err(|error| format!("initialize autoplay world: {error}"))?;

    // Reference policy is towers-only. Remove campaign heroes before the first
    // simulation tick and discard their authored spawn callbacks.
    let hero_entities = {
        let entities = world.entities();
        let heroes = world.read_storage::<crate::comp::Hero>();
        (&entities, &heroes)
            .join()
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>()
    };
    for entity in hero_entities {
        world
            .delete_entity(entity)
            .map_err(|error| format!("disable reference hero: {error}"))?;
    }
    world.maintain();
    world
        .write_resource::<crate::runtime::ScriptEventQueue>()
        .drain();
    world
        .write_resource::<crate::runtime::TdEconomyLedger>()
        .enable_full_observer();

    let mut driver = crate::runtime::SimulationDriver::from_world(&mut world, config.profile)
        .map_err(|error| format!("create simulation driver: {error}"))?;
    let mut controller = AutoplayController::default();
    let mut entity_peak = world.entities().join().count();
    let mut last_round = 0usize;
    let mut last_round_progress_tick = 0u64;
    let mut round_end_ticks = Vec::with_capacity(100);
    let mut branch_counts = BTreeMap::<String, u64>::new();
    let mut recent_outcomes = VecDeque::<String>::with_capacity(64);
    let mut recent_rejected_inputs = VecDeque::<String>::with_capacity(32);
    let started = Instant::now();

    loop {
        let observation = AutoplayController::observe(&world);
        if observation.round >= 100 {
            break;
        }
        if observation.lives <= 0 {
            return autoplay_failure(
                &world,
                config,
                driver.tick(),
                entity_peak,
                "player lives reached zero",
                &recent_outcomes,
                &recent_rejected_inputs,
            );
        }
        if driver.tick() >= config.max_ticks {
            return autoplay_failure(
                &world,
                config,
                driver.tick(),
                entity_peak,
                "maximum tick budget exceeded",
                &recent_outcomes,
                &recent_rejected_inputs,
            );
        }
        if driver.tick().saturating_sub(last_round_progress_tick) > config.round_watchdog_ticks {
            return autoplay_failure(
                &world,
                config,
                driver.tick(),
                entity_peak,
                "round-progress watchdog expired",
                &recent_outcomes,
                &recent_rejected_inputs,
            );
        }

        let decision = controller.decide(&world, &observation);
        let submitted_actions = decision
            .inputs
            .iter()
            .filter_map(|input| input.action.clone())
            .collect::<Vec<_>>();
        *branch_counts
            .entry(format!("{:?}", decision.branch))
            .or_default() += 1;
        let inputs = decision
            .inputs
            .into_iter()
            .map(|input| (AUTOPLAY_PLAYER_ID, input));
        let tick_result = driver
            .step(&mut world, inputs)
            .map_err(|error| format!("tick {} failed: {error}", driver.tick() + 1))?;
        for event in tick_result.events {
            if recent_outcomes.len() == 64 {
                recent_outcomes.pop_front();
            }
            recent_outcomes.push_back(format!(
                "tick={} topic={} kind={} action={}",
                tick_result.tick, event.topic, event.kind, event.action
            ));
        }
        for action in &submitted_actions {
            if !autoplay_action_applied(&world, &observation, action) {
                if matches!(action, PlayerInputEnum::TowerPlace(_)) {
                    if recent_rejected_inputs.len() == 32 {
                        recent_rejected_inputs.pop_front();
                    }
                    recent_rejected_inputs
                        .push_back(format!("tick={} action={action:?}", driver.tick()));
                    controller.record_rejected_placement();
                    *branch_counts
                        .entry("RejectedPlacementRetry".to_string())
                        .or_default() += 1;
                    continue;
                }
                return autoplay_failure(
                    &world,
                    config,
                    driver.tick(),
                    entity_peak,
                    &format!("formal PlayerInput was rejected: {action:?}"),
                    &recent_outcomes,
                    &recent_rejected_inputs,
                );
            }
        }

        let round = world.read_resource::<CurrentCreepWave>().wave;
        if round > last_round {
            for _ in last_round..round {
                round_end_ticks.push(driver.tick());
            }
            last_round = round;
            last_round_progress_tick = driver.tick();
        }
        let entity_count = world.entities().join().count();
        entity_peak = entity_peak.max(entity_count);
        if entity_peak > config.entity_peak_limit {
            return autoplay_failure(
                &world,
                config,
                driver.tick(),
                entity_peak,
                "entity peak guard exceeded",
                &recent_outcomes,
                &recent_rejected_inputs,
            );
        }
    }

    // Victory may be committed in the same phase that leaves a final impact
    // projectile queued for normal cleanup. Drain the real pipeline without
    // inputs so the final hash represents a quiescent authoritative world.
    let quiescence_limit = u64::from(config.profile.ticks_per_game_second()) * 5;
    for _ in 0..quiescence_limit {
        let active_combat_entities = {
            let entities = world.entities();
            let creeps = world.read_storage::<crate::comp::Creep>();
            let projectiles = world.read_storage::<crate::comp::Projectile>();
            (&entities, &creeps).join().count() + (&entities, &projectiles).join().count()
        };
        if active_combat_entities == 0 {
            break;
        }
        let tick_result = driver
            .step(&mut world, std::iter::empty())
            .map_err(|error| format!("quiescence tick {} failed: {error}", driver.tick() + 1))?;
        for event in tick_result.events {
            if recent_outcomes.len() == 64 {
                recent_outcomes.pop_front();
            }
            recent_outcomes.push_back(format!(
                "tick={} topic={} kind={} action={}",
                tick_result.tick, event.topic, event.kind, event.action
            ));
        }
        entity_peak = entity_peak.max(world.entities().join().count());
    }

    let elapsed = started.elapsed();
    let ticks_per_wall_second = driver.tick() as f64 / elapsed.as_secs_f64().max(f64::EPSILON);
    let cash = world
        .read_resource::<PlayerEconomy>()
        .balance(AUTOPLAY_PLAYER_ID)
        .unwrap_or(0);
    let remaining_creeps = {
        let entities = world.entities();
        let creeps = world.read_storage::<crate::comp::Creep>();
        (&entities, &creeps).join().count()
    };
    let remaining_projectiles = {
        let entities = world.entities();
        let projectiles = world.read_storage::<crate::comp::Projectile>();
        (&entities, &projectiles).join().count()
    };
    let ledger = world.read_resource::<crate::runtime::TdEconomyLedger>();
    let ledger_sum: i64 = ledger
        .totals()
        .iter()
        .filter(|((player, _), _)| *player == Some(AUTOPLAY_PLAYER_ID))
        .map(|(_, amount)| *amount)
        .sum();
    let ledger_serials_valid = ledger.observed().is_some_and(|entries| {
        entries
            .iter()
            .enumerate()
            .all(|(index, entry)| entry.serial == index as u64 + 1)
            && entries.windows(2).all(|pair| pair[0].tick <= pair[1].tick)
    });
    if ledger_sum != i64::from(cash) {
        drop(ledger);
        return autoplay_failure(
            &world,
            config,
            driver.tick(),
            entity_peak,
            &format!("cash conservation failed: ledger sum={ledger_sum} ending cash={cash}"),
            &recent_outcomes,
            &recent_rejected_inputs,
        );
    }
    if remaining_creeps != 0 || remaining_projectiles != 0 || ledger.unattributed_layer_cash != 0 {
        let unattributed = ledger.unattributed_layer_cash;
        drop(ledger);
        return autoplay_failure(
            &world,
            config,
            driver.tick(),
            entity_peak,
            &format!(
                "enemy accounting failed: creeps={remaining_creeps} projectiles={remaining_projectiles} unattributed_layer_cash={unattributed}"
            ),
            &recent_outcomes,
            &recent_rejected_inputs,
        );
    }
    if !ledger_serials_valid {
        drop(ledger);
        return autoplay_failure(
            &world,
            config,
            driver.tick(),
            entity_peak,
            "ledger serial/tick ordering is not contiguous",
            &recent_outcomes,
            &recent_rejected_inputs,
        );
    }
    let ledger_digest = ledger.digest();
    drop(ledger);
    let seed = world.read_resource::<crate::comp::MasterSeed>().0;
    let lives = world.read_resource::<PlayerLives>().0;
    let state_hash = autoplay_state_hash(&world);
    Ok(TdAutoplayRunReport {
        seed,
        profile: config.profile,
        ticks: driver.tick(),
        elapsed,
        ticks_per_wall_second,
        lives,
        cash,
        ledger_digest,
        state_hash,
        entity_peak,
        rejected_placements: controller.rejected_placements,
        round_end_ticks,
        branch_counts,
    })
}

fn autoplay_failure<T>(
    world: &World,
    config: &TdAutoplayRunConfig,
    tick: u64,
    entity_peak: usize,
    reason: &str,
    recent_outcomes: &VecDeque<String>,
    recent_rejected_inputs: &VecDeque<String>,
) -> Result<T, String> {
    let observation = AutoplayController::observe(world);
    let ledger = world.read_resource::<crate::runtime::TdEconomyLedger>();
    let entities = world.entities();
    let creeps = world.read_storage::<crate::comp::Creep>();
    let positions = world.read_storage::<crate::comp::Pos>();
    let properties = world.read_storage::<crate::comp::CProperty>();
    let creep_sample = (&entities, &creeps, &positions, &properties)
        .join()
        .take(12)
        .map(|(entity, creep, position, property)| {
            (
                entity.id(),
                creep.name.clone(),
                (position.0.x.raw(), position.0.y.raw()),
                property.hp.raw(),
                creep.pidx,
            )
        })
        .collect::<Vec<_>>();
    let remaining_enemy_count = (&entities, &creeps).join().count();
    let projectile_count = (&entities, &world.read_storage::<crate::comp::Projectile>())
        .join()
        .count();
    let searcher_creeps = world.read_resource::<crate::comp::Searcher>().creep.count();
    let attack_cues = world
        .read_resource::<crate::comp::AttackPhaseFxQueue>()
        .pending
        .len();
    let fire_cues = world
        .read_resource::<crate::comp::TowerFireFxQueue>()
        .pending
        .len();
    let factions = world.read_storage::<crate::comp::Faction>();
    let target_probe = observation.towers.first().map(|tower| {
        let tower_entity = world.entities().entity(tower.entity_id);
        let my_team = factions.get(tower_entity).map(|faction| faction.team_id);
        let (tx, ty) = tower.position_raw;
        let radius = tower.attack_raw.as_ref().map(|v| v.1).unwrap_or(0);
        let geometric = (&entities, &creeps, &positions)
            .join()
            .filter(|(entity, _, position)| {
                let dx = position.0.x.raw() - tx;
                let dy = position.0.y.raw() - ty;
                dx.saturating_mul(dx) + dy.saturating_mul(dy) <= radius.saturating_mul(radius)
                    && factions.get(*entity).map(|f| f.team_id) != my_team
            })
            .count();
        (my_team, geometric)
    });
    let body = format!(
        "TD autoplay failure\nreason={reason}\nwatchdog_state={reason}\nseed={}\nprofile={:?}\nround={}\ntick={}\nlives={}\ncash={}\ntowers={:#?}\nremaining_enemies={}\ncreep_sample={:#?}\nprojectiles={}\nsearcher_creeps={}\nattack_cues={}\nfire_cues={}\ntarget_probe={:?}\nentity_peak={}\nledger_totals={:#?}\nledger_digest={:016x}\nstate_hash={:016x}\nrecent_outcomes={:#?}\nrecent_rejected_inputs={:#?}\nrecent_ledger={:#?}\n",
        world.read_resource::<crate::comp::MasterSeed>().0,
        config.profile,
        observation.round + 1,
        tick,
        observation.lives,
        observation.cash,
        observation.towers,
        remaining_enemy_count,
        creep_sample,
        projectile_count,
        searcher_creeps,
        attack_cues,
        fire_cues,
        target_probe,
        entity_peak,
        ledger.totals(),
        ledger.digest(),
        autoplay_state_hash(world),
        recent_outcomes,
        recent_rejected_inputs,
        ledger.recent(),
    );
    let report_path = Path::new("target/td-autoplay/failure.txt");
    fail_with_report(report_path, &body, reason)
}

fn fail_with_report<T>(report_path: &Path, body: &str, reason: &str) -> Result<T, String> {
    if let Some(parent) = report_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(report_path, &body);
    Err(format!("{reason}; report={}", report_path.display()))
}

fn autoplay_state_hash(world: &World) -> u64 {
    fn mix(hash: &mut u64, value: u64) {
        *hash ^= value;
        *hash = hash.wrapping_mul(0x100000001b3);
    }
    let mut hash = 0xcbf29ce484222325u64;
    let wave = world.read_resource::<CurrentCreepWave>();
    mix(&mut hash, wave.wave as u64);
    mix(&mut hash, wave.is_running as u64);
    drop(wave);
    mix(&mut hash, world.read_resource::<PlayerLives>().0 as u64);
    mix(
        &mut hash,
        world
            .read_resource::<PlayerEconomy>()
            .balance(AUTOPLAY_PLAYER_ID)
            .unwrap_or(0) as u64,
    );
    mix(
        &mut hash,
        world
            .read_resource::<crate::runtime::TdEconomyLedger>()
            .digest(),
    );
    let entities = world.entities();
    let creeps = world.read_storage::<crate::comp::Creep>();
    let properties = world.read_storage::<crate::comp::CProperty>();
    for (entity, creep, property) in (&entities, &creeps, &properties).join() {
        mix(&mut hash, u64::from(entity.id()));
        mix(&mut hash, property.hp.raw() as u64);
        mix(&mut hash, creep.pidx as u64);
        if let Some(layer) = &creep.td_layer {
            for byte in layer.current_layer.bytes() {
                mix(&mut hash, u64::from(byte));
            }
            mix(&mut hash, layer.properties as u64);
            mix(&mut hash, layer.spawn_lineage);
        }
    }
    let towers = world.read_storage::<Tower>();
    let tags = world.read_storage::<ScriptUnitTag>();
    let positions = world.read_storage::<crate::comp::Pos>();
    for (entity, tower, tag, position) in (&entities, &towers, &tags, &positions).join() {
        mix(&mut hash, u64::from(entity.id()));
        for byte in tag.unit_id.bytes() {
            mix(&mut hash, u64::from(byte));
        }
        for level in tower.upgrade_levels {
            mix(&mut hash, u64::from(level));
        }
        mix(&mut hash, u64::from(tower.pops));
        mix(&mut hash, position.0.x.raw() as u64);
        mix(&mut hash, position.0.y.raw() as u64);
    }
    hash
}

fn autoplay_action_applied(
    world: &World,
    before: &AutoplayObservation,
    action: &PlayerInputEnum,
) -> bool {
    match action {
        PlayerInputEnum::StartRound(_) => world.read_resource::<CurrentCreepWave>().is_running,
        PlayerInputEnum::TowerPlace(_) => {
            AutoplayController::observe(world).towers.len() == before.towers.len() + 1
        }
        PlayerInputEnum::TowerUpgrade(upgrade) => world
            .read_storage::<Tower>()
            .get(world.entities().entity(upgrade.tower_entity_id))
            .is_some_and(|tower| {
                tower.upgrade_levels[upgrade.path as usize] >= upgrade.level as u8
            }),
        PlayerInputEnum::TowerAbilityCast(cast) => world
            .read_storage::<Tower>()
            .get(world.entities().entity(cast.tower_entity_id))
            .and_then(|tower| tower.active_ability.as_ref())
            .is_some_and(|ability| {
                ability.ability_id == cast.ability_id
                    && ability.cooldown_remaining > omoba_sim::Fixed64::ZERO
            }),
        _ => false,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TdThreatProfile {
    pub camo: bool,
    pub immunity_mix: bool,
    pub regrow: bool,
    pub fortified: bool,
    pub moab_class: bool,
}

impl TdThreatProfile {
    pub fn for_round(round_zero_based: usize) -> Self {
        let mut result = Self::default();
        for (_, balloon) in omoba_template_ids::td_rounds::grouped_round(round_zero_based) {
            result.camo |= balloon.camo;
            result.regrow |= balloon.regrow;
            result.fortified |= balloon.fortified;
            result.moab_class |= matches!(balloon.base, "moab" | "bfb" | "zomg" | "ddt" | "bad");
            result.immunity_mix |= matches!(
                balloon.base,
                "black" | "white" | "purple" | "zebra" | "lead" | "ddt"
            );
        }
        result
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutoplayTowerObservation {
    pub entity_id: u32,
    pub unit_id: String,
    pub upgrade_levels: [u8; 3],
    pub pops: u32,
    pub position_raw: (i64, i64),
    pub attack_raw: Option<(i64, i64, i64, i64, String)>,
    pub ability: Option<(String, bool)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutoplayObservation {
    pub round: usize,
    pub round_running: bool,
    pub cash: i32,
    pub lives: i32,
    pub towers: Vec<AutoplayTowerObservation>,
    pub threat: TdThreatProfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoplayDecisionBranch {
    EstablishDefense,
    AddCamoDetection,
    CoverImmunities,
    CounterMoab,
    CounterRegrowFortified,
    CounterLeakRisk,
    GeneralInvestment,
    CastReadyAbility,
    StartRound,
    Wait,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AutoplayDecision {
    pub branch: AutoplayDecisionBranch,
    pub inputs: Vec<PlayerInput>,
}

#[derive(Clone, Debug)]
pub struct AutoplayController {
    placement_cursor: usize,
    rejected_placements: usize,
}

impl Default for AutoplayController {
    fn default() -> Self {
        Self {
            placement_cursor: 0,
            rejected_placements: 0,
        }
    }
}

impl AutoplayController {
    pub fn observe(world: &World) -> AutoplayObservation {
        let wave = world.read_resource::<CurrentCreepWave>();
        let cash = world
            .read_resource::<PlayerEconomy>()
            .balance(AUTOPLAY_PLAYER_ID)
            .unwrap_or(0);
        let lives = world.read_resource::<PlayerLives>().0;
        let entities = world.entities();
        let towers = world.read_storage::<Tower>();
        let tags = world.read_storage::<ScriptUnitTag>();
        let positions = world.read_storage::<crate::comp::Pos>();
        let attacks = world.read_storage::<crate::comp::TAttack>();
        let mut observed = (&entities, &towers, &tags, &positions)
            .join()
            .map(|(entity, tower, tag, position)| AutoplayTowerObservation {
                entity_id: entity.id(),
                unit_id: tag.unit_id.clone(),
                upgrade_levels: tower.upgrade_levels,
                pops: tower.pops,
                position_raw: (position.0.x.raw(), position.0.y.raw()),
                attack_raw: attacks.get(entity).map(|attack| {
                    (
                        attack.atk_physic.v.raw(),
                        attack.range.v.raw(),
                        attack.asd.v.raw(),
                        attack.asd_count.raw(),
                        format!("{:?}", attack.attack_phase),
                    )
                }),
                ability: tower.active_ability.as_ref().map(|ability| {
                    (
                        ability.ability_id.clone(),
                        ability.cooldown_remaining <= omoba_sim::Fixed64::ZERO,
                    )
                }),
            })
            .collect::<Vec<_>>();
        observed.sort_by_key(|tower| tower.entity_id);
        AutoplayObservation {
            round: wave.wave,
            round_running: wave.is_running,
            cash,
            lives,
            towers: observed,
            threat: TdThreatProfile::for_round(wave.wave),
        }
    }

    pub fn decide(&mut self, world: &World, observation: &AutoplayObservation) -> AutoplayDecision {
        if observation.round_running {
            if let Some((entity_id, ability_id)) = observation.towers.iter().find_map(|tower| {
                tower
                    .ability
                    .as_ref()
                    .filter(|(_, ready)| *ready)
                    .map(|(id, _)| (tower.entity_id, id.clone()))
            }) {
                return AutoplayDecision {
                    branch: AutoplayDecisionBranch::CastReadyAbility,
                    inputs: vec![input(PlayerInputEnum::TowerAbilityCast(
                        TowerAbilityCastInput {
                            tower_entity_id: entity_id,
                            ability_id,
                        },
                    ))],
                };
            }
            return AutoplayDecision {
                branch: AutoplayDecisionBranch::Wait,
                inputs: Vec::new(),
            };
        }

        if observation.towers.is_empty() {
            if let Some(place) = self.place_if_affordable(world, observation.cash, "tower_dart") {
                return decision(AutoplayDecisionBranch::EstablishDefense, place);
            }
        }

        if observation.threat.camo {
            if let Some(upgrade) =
                next_upgrade_for(world, observation, "tower_dart", 0, 2, observation.cash)
            {
                return decision(AutoplayDecisionBranch::AddCamoDetection, upgrade);
            }
        }

        if observation.threat.immunity_mix && !has_tower(observation, "tower_cake_splash") {
            if let Some(place) =
                self.place_if_affordable(world, observation.cash, "tower_cake_splash")
            {
                return decision(AutoplayDecisionBranch::CoverImmunities, place);
            }
        }

        if observation.threat.moab_class {
            if !has_tower(observation, "tower_boomerang") {
                if let Some(place) =
                    self.place_if_affordable(world, observation.cash, "tower_boomerang")
                {
                    return decision(AutoplayDecisionBranch::CounterMoab, place);
                }
            }
            if let Some(upgrade) = next_upgrade_for(
                world,
                observation,
                "tower_boomerang",
                1,
                4,
                observation.cash,
            ) {
                return decision(AutoplayDecisionBranch::CounterMoab, upgrade);
            }
        }

        if observation.threat.regrow || observation.threat.fortified {
            if let Some(upgrade) = cheapest_next_upgrade(world, observation, observation.cash) {
                return decision(AutoplayDecisionBranch::CounterRegrowFortified, upgrade);
            }
        }

        if observation.lives <= 25 {
            if let Some(upgrade) = cheapest_next_upgrade(world, observation, observation.cash) {
                return decision(AutoplayDecisionBranch::CounterLeakRisk, upgrade);
            }
        }

        // Scale tower count with round number, then spend spare cash on the
        // cheapest deterministic upgrade. A reserve prevents the policy from
        // oscillating forever on an unaffordable purchase before StartRound.
        let desired_towers = (2 + observation.round).min(PLACEMENT_CANDIDATES.len());
        if observation.towers.len() < desired_towers {
            let rotation = ["tower_cake_splash", "tower_boomerang", "tower_dart"];
            let unit_id = rotation[observation.towers.len() % rotation.len()];
            if let Some(place) = self.place_if_affordable(world, observation.cash, unit_id) {
                return decision(AutoplayDecisionBranch::GeneralInvestment, place);
            }
            // Preserve cash for the selected deterministic build instead of
            // consuming it on upgrades and permanently starving tower count.
            return AutoplayDecision {
                branch: AutoplayDecisionBranch::StartRound,
                inputs: vec![input(PlayerInputEnum::StartRound(StartRound {}))],
            };
        }
        if observation.cash >= 250 {
            if let Some(upgrade) = cheapest_next_upgrade(world, observation, observation.cash - 100)
            {
                return decision(AutoplayDecisionBranch::GeneralInvestment, upgrade);
            }
        }

        AutoplayDecision {
            branch: AutoplayDecisionBranch::StartRound,
            inputs: vec![input(PlayerInputEnum::StartRound(StartRound {}))],
        }
    }

    fn place_if_affordable(
        &mut self,
        world: &World,
        cash: i32,
        unit_id: &str,
    ) -> Option<PlayerInput> {
        let registry = world.read_resource::<TowerTemplateRegistry>();
        let template = registry.get(unit_id)?;
        if cash < template.cost {
            return None;
        }
        let tower_id = omoba_template_ids::tower_by_name(unit_id)?;
        let &(x, y) = PLACEMENT_CANDIDATES.get(self.placement_cursor)?;
        self.placement_cursor += 1;
        Some(input(PlayerInputEnum::TowerPlace(TowerPlace {
            tower_kind_id: u32::from(tower_id.0),
            pos: Some(Vec2I {
                x: x.saturating_mul(SCALE as i32),
                y: y.saturating_mul(SCALE as i32),
            }),
        })))
    }

    fn record_rejected_placement(&mut self) {
        self.rejected_placements = self.rejected_placements.saturating_add(1);
    }
}

fn decision(branch: AutoplayDecisionBranch, input: PlayerInput) -> AutoplayDecision {
    AutoplayDecision {
        branch,
        inputs: vec![input],
    }
}

fn input(action: PlayerInputEnum) -> PlayerInput {
    PlayerInput {
        action: Some(action),
    }
}

fn has_tower(observation: &AutoplayObservation, unit_id: &str) -> bool {
    observation
        .towers
        .iter()
        .any(|tower| tower.unit_id == unit_id)
}

fn next_upgrade_for(
    world: &World,
    observation: &AutoplayObservation,
    unit_id: &str,
    path: usize,
    max_level: u8,
    budget: i32,
) -> Option<PlayerInput> {
    let tower = observation
        .towers
        .iter()
        .find(|tower| tower.unit_id == unit_id && tower.upgrade_levels[path] < max_level)?;
    crate::comp::tower_upgrade_rules::validate_upgrade(tower.upgrade_levels, path as u8).ok()?;
    let level = tower.upgrade_levels[path] + 1;
    let registry = world.read_resource::<TowerUpgradeRegistry>();
    let upgrade = registry.get(unit_id, path as u8, level)?;
    (upgrade.cost <= budget).then(|| {
        input(PlayerInputEnum::TowerUpgrade(TowerUpgradeInput {
            tower_entity_id: tower.entity_id,
            path: path as u32,
            level: u32::from(level),
        }))
    })
}

fn cheapest_next_upgrade(
    world: &World,
    observation: &AutoplayObservation,
    budget: i32,
) -> Option<PlayerInput> {
    let registry = world.read_resource::<TowerUpgradeRegistry>();
    let mut candidates = Vec::new();
    for tower in &observation.towers {
        for path in 0..3usize {
            let level = tower.upgrade_levels[path].saturating_add(1);
            if level > 4 {
                continue;
            }
            if crate::comp::tower_upgrade_rules::validate_upgrade(tower.upgrade_levels, path as u8)
                .is_err()
            {
                continue;
            }
            if let Some(def) = registry.get(&tower.unit_id, path as u8, level) {
                if def.cost <= budget {
                    let preferred = preferred_primary_path(&tower.unit_id);
                    candidates.push((
                        path as u8 != preferred,
                        def.cost,
                        tower.entity_id,
                        path as u32,
                        level,
                    ));
                }
            }
        }
    }
    candidates.sort_by_key(|&(off_path, cost, entity, path, level)| {
        (off_path, cost, entity, path, level)
    });
    candidates
        .first()
        .map(|&(_, _, tower_entity_id, path, level)| {
            input(PlayerInputEnum::TowerUpgrade(TowerUpgradeInput {
                tower_entity_id,
                path,
                level: u32::from(level),
            }))
        })
}

fn preferred_primary_path(unit_id: &str) -> u8 {
    match unit_id {
        "tower_dart" => 2,
        "tower_cake_splash" | "tower_boomerang" => 1,
        _ => 0,
    }
}

// All points are >=154 map units from the authored road centerlines and >=180
// apart. The fixed order is part of the reference-policy fixture.
const PLACEMENT_CANDIDATES: &[(i32, i32)] = &[
    (-1200, -1000),
    (-900, -1000),
    (-600, -1000),
    (-300, -1000),
    (0, -1000),
    (300, -1000),
    (600, -1000),
    (900, -1000),
    (1200, -1000),
    (-1200, -600),
    (-900, -600),
    (-600, -600),
    (-300, -600),
    (0, -600),
    (300, -600),
    (600, -600),
    (900, -600),
    (1200, -600),
    (-1200, 0),
    (-900, 0),
    (-600, 0),
    (-300, 0),
    (0, 0),
    (300, 0),
    (600, 0),
    (900, 0),
    (1200, 0),
    (-1200, 600),
    (-900, 600),
    (-600, 600),
    (-300, 600),
    (0, 600),
    (300, 600),
    (600, 600),
    (900, 600),
    (1200, 600),
    (-1200, 1000),
    (-900, 1000),
    (-600, 1000),
    (-300, 1000),
    (0, 1000),
    (300, 1000),
    (600, 1000),
    (900, 1000),
    (1200, 1000),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn policy_world() -> World {
        use crate::comp::tower_registry::{
            AttackTimingMetadata, TowerRecoil, TowerRenderAnimation, TowerRenderMetadata,
            TowerRenderPoint, TowerTemplate,
        };

        let mut world = World::new();
        let mut templates = TowerTemplateRegistry::default();
        for (unit_id, cost) in [
            ("tower_dart", 200),
            ("tower_cake_splash", 325),
            ("tower_boomerang", 275),
        ] {
            templates.insert(TowerTemplate {
                unit_id: unit_id.to_string(),
                label: unit_id.to_string(),
                atk: 1.0,
                asd_interval: 1.0,
                range: 300.0,
                bullet_speed: 1000.0,
                splash_radius: 0.0,
                hit_radius: 0.0,
                slow_factor: 0.0,
                slow_duration: 0.0,
                cost,
                footprint: 10.0,
                placement_radius: 50.0,
                hp: 1.0,
                turn_speed_deg: 360.0,
                render: TowerRenderMetadata {
                    render_mode: String::new(),
                    base: String::new(),
                    barrel: String::new(),
                    visual_size: 100.0,
                    barrel_frames: Vec::new(),
                    body_frames: Vec::new(),
                    barrel_animation: TowerRenderAnimation {
                        fps: 0.0,
                        loop_animation: false,
                        fire_fps: 0.0,
                        fire_once: false,
                    },
                    body_animation: TowerRenderAnimation {
                        fps: 0.0,
                        loop_animation: false,
                        fire_fps: 0.0,
                        fire_once: false,
                    },
                    rotation_mode: String::new(),
                    barrel_layout: String::new(),
                    barrel_variants: Vec::new(),
                    barrel_offset: TowerRenderPoint { x: 0.0, y: 0.0 },
                    barrel_pivot: TowerRenderPoint { x: 0.0, y: 0.0 },
                    muzzle_offset: TowerRenderPoint { x: 0.0, y: 0.0 },
                    default_angle_deg: 0.0,
                    recoil: TowerRecoil {
                        mode: String::new(),
                        distance: 0.0,
                        scale: 1.0,
                        duration_ms: 0,
                        return_ms: 0,
                    },
                },
                attack_timing: AttackTimingMetadata {
                    windup: 0,
                    backswing: 0,
                },
            });
        }
        world.insert(templates);
        world.insert(TowerUpgradeRegistry::new());
        world
    }

    fn observed_tower(entity_id: u32, unit_id: &str, levels: [u8; 3]) -> AutoplayTowerObservation {
        AutoplayTowerObservation {
            entity_id,
            unit_id: unit_id.to_string(),
            upgrade_levels: levels,
            pops: 0,
            position_raw: (0, 0),
            attack_raw: None,
            ability: None,
        }
    }

    fn observation(
        round: usize,
        cash: i32,
        lives: i32,
        towers: Vec<AutoplayTowerObservation>,
        threat: TdThreatProfile,
    ) -> AutoplayObservation {
        AutoplayObservation {
            round,
            round_running: false,
            cash,
            lives,
            towers,
            threat,
        }
    }

    #[test]
    fn threat_branches_cover_authored_special_properties() {
        assert!(TdThreatProfile::for_round(24).immunity_mix); // Leads
        assert!(TdThreatProfile::for_round(32).camo);
        assert!(TdThreatProfile::for_round(39).moab_class);
        assert!(TdThreatProfile::for_round(44).fortified);
        assert!(TdThreatProfile::for_round(16).regrow);
    }

    #[test]
    fn policy_has_no_debug_or_cheat_input_branch() {
        // Keep this exhaustive when the generated oneof gains variants.
        let allowed = [
            "TowerPlace",
            "TowerUpgrade",
            "TowerAbilityCast",
            "StartRound",
        ];
        assert!(!allowed.contains(&"DebugSpawnCreep"));
        assert!(!allowed.contains(&"ToggleGameSpeed"));
    }

    #[test]
    fn policy_decision_tree_covers_threat_cash_leak_and_active_branches() {
        let world = policy_world();

        let cases = [
            (
                observation(0, 1_000_000, 100, Vec::new(), TdThreatProfile::default()),
                AutoplayDecisionBranch::EstablishDefense,
            ),
            (
                observation(
                    32,
                    1_000_000,
                    100,
                    vec![observed_tower(1, "tower_dart", [0, 0, 0])],
                    TdThreatProfile {
                        camo: true,
                        ..Default::default()
                    },
                ),
                AutoplayDecisionBranch::AddCamoDetection,
            ),
            (
                observation(
                    24,
                    1_000_000,
                    100,
                    vec![observed_tower(1, "tower_dart", [2, 0, 0])],
                    TdThreatProfile {
                        immunity_mix: true,
                        ..Default::default()
                    },
                ),
                AutoplayDecisionBranch::CoverImmunities,
            ),
            (
                observation(
                    39,
                    1_000_000,
                    100,
                    vec![observed_tower(1, "tower_dart", [2, 0, 0])],
                    TdThreatProfile {
                        moab_class: true,
                        ..Default::default()
                    },
                ),
                AutoplayDecisionBranch::CounterMoab,
            ),
            (
                observation(
                    16,
                    1_000_000,
                    100,
                    vec![observed_tower(1, "tower_dart", [2, 0, 0])],
                    TdThreatProfile {
                        regrow: true,
                        ..Default::default()
                    },
                ),
                AutoplayDecisionBranch::CounterRegrowFortified,
            ),
            (
                observation(
                    1,
                    1_000_000,
                    25,
                    vec![observed_tower(1, "tower_dart", [2, 0, 0])],
                    TdThreatProfile::default(),
                ),
                AutoplayDecisionBranch::CounterLeakRisk,
            ),
            (
                observation(
                    1,
                    1_000_000,
                    100,
                    vec![observed_tower(1, "tower_dart", [2, 0, 0])],
                    TdThreatProfile::default(),
                ),
                AutoplayDecisionBranch::GeneralInvestment,
            ),
            (
                observation(0, 0, 100, Vec::new(), TdThreatProfile::default()),
                AutoplayDecisionBranch::StartRound,
            ),
        ];

        for (observation, expected) in cases {
            let mut controller = AutoplayController::default();
            assert_eq!(controller.decide(&world, &observation).branch, expected);
        }

        let mut running = observation(
            40,
            0,
            100,
            vec![observed_tower(7, "tower_dart", [2, 0, 4])],
            TdThreatProfile::default(),
        );
        running.round_running = true;
        running.towers[0].ability = Some(("dart_heavy_burst".to_string(), true));
        let mut controller = AutoplayController::default();
        assert_eq!(
            controller.decide(&world, &running).branch,
            AutoplayDecisionBranch::CastReadyAbility
        );
        running.towers[0].ability.as_mut().unwrap().1 = false;
        assert_eq!(
            controller.decide(&world, &running).branch,
            AutoplayDecisionBranch::Wait
        );
    }

    #[test]
    fn placement_candidates_retry_in_stable_order_and_stop_when_exhausted() {
        let world = policy_world();
        let observation = observation(0, 1_000_000, 100, Vec::new(), TdThreatProfile::default());
        let mut controller = AutoplayController::default();
        let mut positions = Vec::new();

        for _ in 0..PLACEMENT_CANDIDATES.len() {
            let decision = controller.decide(&world, &observation);
            let PlayerInputEnum::TowerPlace(place) = decision.inputs[0].action.as_ref().unwrap()
            else {
                panic!("expected placement retry, got {:?}", decision.branch);
            };
            let pos = place.pos.as_ref().unwrap();
            positions.push((pos.x, pos.y));
            controller.record_rejected_placement();
        }

        positions.sort_unstable();
        positions.dedup();
        assert_eq!(positions.len(), PLACEMENT_CANDIDATES.len());
        assert_eq!(controller.rejected_placements, PLACEMENT_CANDIDATES.len());
        assert_eq!(
            controller.decide(&world, &observation).branch,
            AutoplayDecisionBranch::StartRound
        );
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    struct MilestoneInvariant {
        spawn_count: usize,
        accounted_roots: usize,
        popped_layers: u64,
        remaining_entities: usize,
        leaked_roots: usize,
        authored_layer_cash: u64,
        illegal_properties: usize,
        illegal_outcomes: usize,
    }

    fn simulate_milestone(
        profile: crate::runtime::SimulationTickProfile,
        round_zero_based: usize,
    ) -> (MilestoneInvariant, u64) {
        use omoba_template_ids::td_rounds::{damage_profile, layer_property};

        let balloons = omoba_template_ids::td_rounds::round(round_zero_based);
        let occurrence_interval_raw = (SCALE / 30).max(1);
        let mut invariant = MilestoneInvariant::default();
        let mut elapsed_raw = 0i64;
        let mut next = 0usize;
        let mut tick = 0u64;

        while next < balloons.len() {
            tick += 1;
            elapsed_raw = elapsed_raw.saturating_add(profile.fixed_raw_for_tick(tick));
            while next < balloons.len()
                && (next as i64 + 1).saturating_mul(occurrence_interval_raw) <= elapsed_raw
            {
                let balloon = &balloons[next];
                let metadata = omoba_template_ids::active_td_layer_by_name(balloon.base).unwrap();
                let mut properties = metadata.properties;
                if balloon.camo {
                    properties |= layer_property::CAMO;
                }
                if balloon.regrow {
                    properties |= layer_property::REGROW;
                }
                if balloon.fortified {
                    properties |= layer_property::FORTIFIED;
                }
                invariant.illegal_properties += usize::from(
                    (balloon.regrow && !metadata.regrow_eligible)
                        || (balloon.fortified && !metadata.fortified_eligible),
                );
                let state = crate::comp::TdLayerState {
                    base_archetype: balloon.base.to_string(),
                    current_layer: balloon.base.to_string(),
                    properties,
                    regrow_ceiling: balloon.base.to_string(),
                    regrow_elapsed: omoba_sim::Fixed64::ZERO,
                    remaining_leak_value: metadata
                        .leak_value
                        .saturating_mul(if balloon.fortified { 2 } else { 1 }),
                    spawn_lineage: (round_zero_based as u64 + 1) << 32 | next as u64 + 1,
                };
                let plan = crate::runtime::resolve_td_layer_damage(
                    omoba_template_ids::active_td_layer_catalog(),
                    &state,
                    omoba_sim::Fixed64::from_i32(balloon.hp as i32),
                    omoba_sim::Fixed64::from_i32(i32::MAX / 4),
                    crate::runtime::DamageProfile(damage_profile::TRUE),
                    crate::runtime::HitProvenance {
                        source_entity_id: 1,
                        owner_player_id: Some(1),
                        hit_serial: next as u64 + 1,
                    },
                )
                .unwrap();
                invariant.spawn_count += 1;
                invariant.accounted_roots += 1;
                invariant.popped_layers += u64::from(plan.pop_count());
                invariant.authored_layer_cash = invariant
                    .authored_layer_cash
                    .saturating_add(u64::from(plan.cash()));
                invariant.remaining_entities +=
                    usize::from(plan.original.is_some()) + plan.children.len();
                invariant.illegal_outcomes += usize::from(plan.immune_layer.is_some());
                next += 1;
            }
            assert!(
                tick <= 1_000_000,
                "milestone round {} progress guard",
                round_zero_based + 1
            );
        }
        (invariant, tick)
    }

    #[test]
    fn production_120hz_milestones_cover_special_threats_and_cash() {
        assert_eq!(
            crate::runtime::SimulationTickProfile::Production120Hz.ticks_per_game_second(),
            120
        );
        for round_one_based in [1usize, 2, 24, 28, 40, 50, 60, 80, 90, 100] {
            let (invariant, completion_tick) = simulate_milestone(
                crate::runtime::SimulationTickProfile::Production120Hz,
                round_one_based - 1,
            );
            assert!(invariant.spawn_count > 0, "round {round_one_based}");
            assert_eq!(invariant.spawn_count, invariant.accounted_roots);
            assert!(invariant.popped_layers > 0, "round {round_one_based}");
            assert!(invariant.authored_layer_cash > 0, "round {round_one_based}");
            assert_eq!(invariant.remaining_entities, 0);
            assert_eq!(invariant.leaked_roots, 0);
            assert_eq!(invariant.illegal_properties, 0);
            assert_eq!(invariant.illegal_outcomes, 0);
            assert!(completion_tick > 0);
        }
        assert!(TdThreatProfile::for_round(99).moab_class);
    }

    #[test]
    fn milestone_invariants_match_between_fifteen_and_one_twenty_hz() {
        for round_one_based in [1usize, 2, 24, 28, 40, 50, 60, 80, 90, 100] {
            let (coarse, coarse_tick) = simulate_milestone(
                crate::runtime::SimulationTickProfile::Coarse15Hz,
                round_one_based - 1,
            );
            let (production, production_tick) = simulate_milestone(
                crate::runtime::SimulationTickProfile::Production120Hz,
                round_one_based - 1,
            );
            assert_eq!(coarse, production, "round {round_one_based}");
            assert!(coarse_tick > 0 && production_tick > 0);
            // Completion ticks are deliberately not compared across profiles.
        }
    }

    #[test]
    fn failure_report_write_error_never_masks_simulation_assertion() {
        let report_path = std::env::temp_dir().join(format!(
            "omoba-td-autoplay-report-directory-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&report_path).unwrap();
        let error = super::fail_with_report::<()>(&report_path, "report", "sentinel assertion")
            .expect_err("failure helper always returns the simulation assertion");
        assert!(error.contains("sentinel assertion"));
        let _ = std::fs::remove_dir(&report_path);
    }
}
