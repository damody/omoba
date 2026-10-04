//! Opt-in single-lane MOBA rules over the production ECS and outcome pipeline.
//! No renderer, wall clock, network, or script-DLL dependency is needed here.

use std::collections::BTreeMap;

use failure::{err_msg, Error};
use omoba_sim::{Fixed64, Vec2};
use specs::{Builder, Entity, World, WorldExt};

use crate::comp::*;
mod jungle;
pub use jungle::MobaJungleCamp;
pub(crate) use jungle::record_moba_jungle_damage;

/// Global public HUD metric, not an entity reference or private match resource.
pub const SINGLE_LANE_DELTA_METRIC_ID: u64 = 0x4d4f424144543031;
pub const SINGLE_LANE_ELAPSED_METRIC_ID: u64 = 0x4d4f4241544d3031;
pub const SINGLE_LANE_PHASE_METRIC_ID: u64 = 0x4d4f424150483031;
pub const SINGLE_LANE_WINNER_METRIC_ID: u64 = 0x4d4f4241574e3031;
pub const SINGLE_LANE_RESPAWN_METRIC_ID: u64 = 0x4d52535000000000;
pub const SINGLE_LANE_LENGTH_METRIC_ID: u64 = 0x4d4f42414c4e3031;
pub const SINGLE_LANE_RECALL_ACTIVE_METRIC_ID: u64 = 0x4d52414300000000;
pub const SINGLE_LANE_RECALL_REMAINING_METRIC_ID: u64 = 0x4d524d4e00000000;
pub const SINGLE_LANE_KILLS_METRIC_ID: u64 = 0x4d4b494c00000000;
pub const SINGLE_LANE_DEATHS_METRIC_ID: u64 = 0x4d44454100000000;
pub const SINGLE_LANE_ASSISTS_METRIC_ID: u64 = 0x4d41535300000000;
// Explicit public-score namespaces; never reuse private owner HUD metrics.
pub const SINGLE_LANE_SCORE_COUNT_METRIC_ID: u64 = 0x4d53434f434e5431;
pub const SINGLE_LANE_SCORE_TEAM_METRIC_ID: u64 = 0x4d53544d00000000;
pub const SINGLE_LANE_SCORE_KILLS_METRIC_ID: u64 = 0x4d534b4c00000000;
pub const SINGLE_LANE_SCORE_DEATHS_METRIC_ID: u64 = 0x4d53445400000000;
pub const SINGLE_LANE_SCORE_ASSISTS_METRIC_ID: u64 = 0x4d53415300000000;

fn emit_owner_score(world: &World, state: &MobaMatch, phase: crate::runtime::FactPhase) {
    let emit_public = |ordinal, metric_id, value| {
        world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(crate::runtime::OrderedFact {
            key: crate::runtime::FactOrderingKey { tick: world.read_resource::<Tick>().0, phase,
                canonical_source_order: 0, local_ordinal: ordinal, fact_kind: crate::runtime::FactKind::Hud },
            audience: crate::runtime::FactAudience::AllPlayers,
            fact: crate::runtime::ObservableFact::Hud { team: 0, metric_id, value },
        }).expect("valid public score");
    };
    emit_public(100, SINGLE_LANE_SCORE_COUNT_METRIC_ID, state.heroes.len() as i64);
    for (index, slot) in state.heroes.iter().enumerate() {
        let team = state.config.teams[slot.side];
        for (ordinal, (namespace, value)) in [
            (SINGLE_LANE_SCORE_TEAM_METRIC_ID, team),
            (SINGLE_LANE_SCORE_KILLS_METRIC_ID, slot.kills),
            (SINGLE_LANE_SCORE_DEATHS_METRIC_ID, slot.deaths),
            (SINGLE_LANE_SCORE_ASSISTS_METRIC_ID, slot.assists),
        ].into_iter().enumerate() {
            emit_public(101 + index as u32 * 4 + ordinal as u32,
                single_lane_player_metric(namespace, slot.player_id), i64::from(value));
        }
        for (ordinal, (namespace, value)) in [
            (SINGLE_LANE_KILLS_METRIC_ID, slot.kills),
            (SINGLE_LANE_DEATHS_METRIC_ID, slot.deaths),
            (SINGLE_LANE_ASSISTS_METRIC_ID, slot.assists),
        ].into_iter().enumerate() {
            world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(crate::runtime::OrderedFact {
                key: crate::runtime::FactOrderingKey {
                    tick: world.read_resource::<Tick>().0, phase, canonical_source_order: 0,
                    local_ordinal: 40 + index as u32 * 3 + ordinal as u32,
                    fact_kind: crate::runtime::FactKind::Hud,
                },
                audience: crate::runtime::FactAudience::Team(team),
                fact: crate::runtime::ObservableFact::Hud {
                    team, metric_id: single_lane_player_metric(namespace, slot.player_id), value: i64::from(value),
                },
            }).expect("valid persistent owner score");
        }
    }
}

/// The low 32 bits identify a player, never a team or a transient ECS entity.
pub const fn single_lane_player_metric(namespace: u64, player_id: u32) -> u64 {
    (namespace & 0xffff_ffff_0000_0000) | player_id as u64
}

pub fn single_lane_metric_player(metric_id: u64, namespace: u64) -> Option<u32> {
    let player = metric_id as u32;
    (player != 0 && metric_id & 0xffff_ffff_0000_0000 == namespace).then_some(player)
}

fn emit_recall_state(world: &World, state: &MobaMatch, phase: crate::runtime::FactPhase) {
    for (side, slot) in state.heroes.iter().enumerate() {
        let team = state.config.teams[slot.side];
        let remaining = if state.phase == MobaMatchPhase::Playing {
            slot.recall.map_or(0, |(_, deadline)| (deadline - state.elapsed).max(Fixed64::ZERO).raw())
        } else { 0 };
        let metric_id = if phase == crate::runtime::FactPhase::PreStep {
            SINGLE_LANE_RECALL_ACTIVE_METRIC_ID
        } else { SINGLE_LANE_RECALL_REMAINING_METRIC_ID };
        world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(crate::runtime::OrderedFact {
            key: crate::runtime::FactOrderingKey {
                tick: world.read_resource::<Tick>().0, phase, canonical_source_order: 0,
                local_ordinal: 20 + side as u32, fact_kind: crate::runtime::FactKind::Hud,
            },
            audience: crate::runtime::FactAudience::Team(team),
            fact: crate::runtime::ObservableFact::Hud {
                team, metric_id: single_lane_player_metric(metric_id, slot.player_id),
                value: if phase == crate::runtime::FactPhase::PreStep { i64::from(remaining > 0) } else { remaining },
            },
        }).expect("valid owner-team recall state");
    }
}

fn emit_replica_delta(world: &mut World, state: &MobaMatch, active: bool) {
    // Exactly one score sample per tick. Active ticks publish after damage and
    // death settlement; exposing both phases would make first-match HUD readers
    // select the old score from before the kill.
    if !active { emit_owner_score(world, state, crate::runtime::FactPhase::PreStep); }
    let tick = world.read_resource::<Tick>().0;
    emit_recall_state(world, state, crate::runtime::FactPhase::PreStep);
    if state.phase != MobaMatchPhase::Playing {
        emit_recall_state(world, state, crate::runtime::FactPhase::PostStep);
    }
    emit_owner_economy(world, state, crate::runtime::FactPhase::PreStep);
    let value = if active {
        world.read_resource::<DeltaTime>().0.raw()
    } else {
        0
    };
    world.write_resource::<Time>().0 = state.elapsed.raw() as f64 / omoba_sim::fixed::SCALE as f64;
    let phase = match state.phase {
        MobaMatchPhase::Warmup => 0,
        MobaMatchPhase::Playing => 1,
        MobaMatchPhase::Finished { .. } => 2,
    };
    for (ordinal, (metric_id, value)) in [
        (SINGLE_LANE_DELTA_METRIC_ID, value),
        (SINGLE_LANE_ELAPSED_METRIC_ID, state.elapsed.raw()),
        (SINGLE_LANE_PHASE_METRIC_ID, phase),
        (
            SINGLE_LANE_WINNER_METRIC_ID,
            match state.phase {
                MobaMatchPhase::Finished {
                    winner: Some(side), ..
                } => i64::from(state.config.teams[side as usize]),
                _ => 0,
            },
        ),
        (SINGLE_LANE_LENGTH_METRIC_ID, state.config.lane_length.raw()),
    ]
    .into_iter()
    .enumerate()
    {
        world
            .read_resource::<crate::runtime::ObservableFactBuffer>()
            .emit(crate::runtime::OrderedFact {
                key: crate::runtime::FactOrderingKey {
                    tick,
                    phase: crate::runtime::FactPhase::PreStep,
                    canonical_source_order: 0,
                    local_ordinal: ordinal as u32,
                    fact_kind: crate::runtime::FactKind::Hud,
                },
                audience: crate::runtime::FactAudience::AllPlayers,
                fact: crate::runtime::ObservableFact::Hud {
                    team: 0,
                    metric_id,
                    value,
                },
            })
            .expect("valid public lane delta");
    }
    for (side, hero) in state.heroes.iter().enumerate() {
        let team = state.config.teams[hero.side];
        let remaining = if hero.entity.is_none() {
            hero.respawn_at
                .map_or(0, |at| (at - state.elapsed).max(Fixed64::ZERO).raw())
        } else {
            0
        };
        world
            .read_resource::<crate::runtime::ObservableFactBuffer>()
            .emit(crate::runtime::OrderedFact {
                key: crate::runtime::FactOrderingKey {
                    tick,
                    phase: crate::runtime::FactPhase::PreStep,
                    canonical_source_order: 0,
                    local_ordinal: 5 + side as u32,
                    fact_kind: crate::runtime::FactKind::Hud,
                },
                audience: crate::runtime::FactAudience::Team(team),
                fact: crate::runtime::ObservableFact::Hud {
                    team,
                    metric_id: single_lane_player_metric(SINGLE_LANE_RESPAWN_METRIC_ID, hero.player_id),
                    value: remaining,
                },
            })
            .expect("valid owner-team respawn HUD");
    }
}

fn emit_owner_economy(world: &World, state: &MobaMatch, phase: crate::runtime::FactPhase) {
    use crate::runtime::native::economy_projection::{CommittedEconomyState, OwnerEconomyState};
    let inventories = world.read_storage::<Inventory>();
    let balances = world.read_storage::<Gold>();
    let effects = world.read_storage::<ItemEffects>();
    let positions = world.read_storage::<Pos>();
    let properties = world.read_storage::<CProperty>();
    let active =
        state.phase == MobaMatchPhase::Playing && !world.read_resource::<GamePause>().is_paused;
    for (side, slot) in state.heroes.iter().enumerate() {
        let team = state.config.teams[slot.side];
        let economy = if let Some(entity) = slot.entity {
            CommittedEconomyState::capture(
                *balances.get(entity).expect("live hero gold"),
                inventories.get(entity).expect("live hero inventory"),
                *effects.get(entity).expect("live hero effects"),
            )
        } else {
            CommittedEconomyState::capture(slot.gold, &slot.inventory, ItemEffects::default())
        }
        .expect("valid persistent owner economy");
        let shop_available = active
            && slot.entity.is_some_and(|entity| {
                properties.get(entity).is_some_and(|p| p.hp > Fixed64::ZERO)
                    && positions
                        .get(entity)
                        .zip(state.bases[slot.side].and_then(|base| positions.get(base)))
                        .is_some_and(|(hero, base)| {
                            (hero.0 - base.0).length_squared()
                                <= Fixed64::from_i32(300) * Fixed64::from_i32(300)
                        })
            });
        world
            .read_resource::<crate::runtime::ObservableFactBuffer>()
            .emit(crate::runtime::OrderedFact {
                key: crate::runtime::FactOrderingKey {
                    tick: world.read_resource::<Tick>().0,
                    phase,
                    canonical_source_order: 0,
                    local_ordinal: side as u32 * 2
                        + u32::from(phase == crate::runtime::FactPhase::PostStep),
                    fact_kind: crate::runtime::FactKind::OwnerEconomy,
                },
                audience: crate::runtime::FactAudience::Team(team),
                fact: crate::runtime::ObservableFact::OwnerEconomy {
                    team,
                    state: OwnerEconomyState {
                        player_id: slot.player_id,
                        economy,
                        shop_available,
                    },
                },
            })
            .expect("valid persistent owner economy HUD");
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobaMatchPhase {
    Warmup,
    Playing,
    Finished { winner: Option<u8>, tick: u64 },
}

#[derive(Clone, Debug)]
pub struct SingleLanePlayerConfig {
    pub player_id: u32,
    pub team_id: u32,
    pub hero: String,
}

#[derive(Clone, Debug)]
pub struct SingleLaneConfig {
    /// None preserves the verified straight lane. Some selects a compiled Lua map.
    pub map_id: Option<String>,
    pub seed: u64,
    pub players: [u32; 2],
    /// Authenticated wire team IDs; array indices remain lane sides (0/1).
    pub teams: [u32; 2],
    pub heroes: [String; 2],
    /// Initial players stay first for compatibility; extra players have explicit teams.
    pub additional_players: Vec<SingleLanePlayerConfig>,
    pub warmup: Fixed64,
    pub respawn_delay: Fixed64,
    pub wave_interval: Fixed64,
    pub creeps_per_wave: u32,
    pub lane_length: Fixed64,
    pub tower_hp: Fixed64,
    pub base_hp: Fixed64,
    pub passive_gold_per_second: u32,
    pub hero_kill_gold: u32,
    pub hero_assist_gold: u32,
    pub hero_kill_xp: u32,
    pub hero_assist_xp: u32,
    pub lane_creep_xp: u32,
    pub lane_xp_radius: u32,
    pub assist_window_seconds: u32,
}

impl Default for SingleLaneConfig {
    fn default() -> Self {
        Self {
            map_id: None,
            seed: 1,
            players: [1, 2],
            teams: [1, 2],
            heroes: ["training_luminary".into(), "training_luminary".into()],
            additional_players: Vec::new(),
            warmup: Fixed64::from_i32(2),
            respawn_delay: Fixed64::from_i32(5),
            wave_interval: Fixed64::from_i32(8),
            creeps_per_wave: 3,
            lane_length: Fixed64::from_i32(2400),
            tower_hp: Fixed64::from_i32(1200),
            base_hp: Fixed64::from_i32(1600),
            passive_gold_per_second: omoba_template_ids::MOBA_PASSIVE_GOLD_PER_SECOND,
            hero_kill_gold: omoba_template_ids::MOBA_HERO_KILL_GOLD,
            hero_assist_gold: omoba_template_ids::MOBA_HERO_ASSIST_GOLD,
            hero_kill_xp: omoba_template_ids::MOBA_HERO_KILL_XP,
            hero_assist_xp: omoba_template_ids::MOBA_HERO_ASSIST_XP,
            lane_creep_xp: omoba_template_ids::MOBA_LANE_CREEP_XP,
            lane_xp_radius: omoba_template_ids::MOBA_LANE_XP_RADIUS,
            assist_window_seconds: omoba_template_ids::MOBA_ASSIST_WINDOW_SECONDS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaneRole {
    Hero,
    Creep,
    Tower,
    Base,
}

#[derive(Clone, Debug)]
struct LaneUnit {
    entity: Entity,
    team: u8,
    role: LaneRole,
    attack_remaining: Fixed64,
    lane: usize,
    next_waypoint: usize,
}

#[derive(Clone, Debug)]
pub struct MobaHeroSlot {
    pub player_id: u32,
    /// Lane side (0/1), independent of this slot's index in the player roster.
    pub side: usize,
    pub entity: Option<Entity>,
    pub deaths: u32,
    pub kills: u32,
    pub assists: u32,
    lethal_pending: bool,
    recall: Option<(Vec2, Fixed64)>,
    pub respawns: u32,
    pub respawn_at: Option<Fixed64>,
    hero: Hero,
    inventory: Inventory,
    gold: Gold,
}

#[derive(Clone, Debug)]
pub struct MobaMatch {
    pub config: SingleLaneConfig,
    pub phase: MobaMatchPhase,
    pub elapsed: Fixed64,
    income_remainder_raw: i64,
    pending_income_delta_raw: i64,
    pub waves: u32,
    pub heroes: Vec<MobaHeroSlot>,
    pub towers: [Option<Entity>; 2],
    /// One tower per side per lane. `towers` remains the first surviving tower
    /// for legacy HUD/bot callers, never the base-unlock authority.
    pub lane_towers: Vec<[Option<Entity>; 2]>,
    routes: Vec<[Vec<Vec2>; 2]>,
    pub bases: [Option<Entity>; 2],
    pub jungle_camps: Vec<MobaJungleCamp>,
    assist_ledger: super::moba_assist::AssistLedger,
    next_wave_at: Fixed64,
    units: BTreeMap<u64, LaneUnit>,
}

impl MobaMatch {
    pub fn base_unlocked(&self, side: usize) -> bool {
        self.lane_towers.iter().all(|lane| lane[side].is_none())
    }
    pub fn is_recalling(&self, entity: Entity) -> bool {
        self.heroes.iter().any(|s| s.entity == Some(entity) && s.recall.is_some())
    }
    #[cfg(feature = "kcp")]
    pub fn allows_player_input(&self, player_id: u32, input: &crate::runtime::PlayerInput) -> bool {
        // Transport independently enforces negotiated, authenticated capability.
        self.heroes.iter().any(|slot| slot.player_id == player_id) && single_lane_input_action_allowed(input)
    }

    pub fn owns_npc_combat(&self, entity: Entity) -> bool {
        self.jungle_camps.iter().any(|camp| camp.entity == Some(entity)) || self.units
            .get(&entity_key(entity))
            .is_some_and(|unit| matches!(unit.role, LaneRole::Creep | LaneRole::Tower))
    }
}

#[cfg(feature = "kcp")]
pub fn single_lane_input_action_allowed(input: &crate::runtime::PlayerInput) -> bool {
    use crate::runtime::PlayerInputEnum as I;
    matches!(
        input.action,
        Some(
            I::NoOp(_)
                | I::MoveTo(_)
                | I::AttackMove(_)
                | I::AttackTarget(_)
                | I::CastAbility(_)
                | I::UpgradeAbility(_)
                | I::ItemUse(_)
                | I::ItemBuy(_)
                | I::ItemSell(_)
                | I::Recall(_)
        )
    )
}

fn entity_key(entity: Entity) -> u64 {
    (u64::from(entity.id()) << 32) | u64::from(entity.gen().id() as u32)
}

fn faction(side: u8, team: u32) -> Faction {
    Faction::new(
        if side == 0 {
            FactionType::Player
        } else {
            FactionType::Enemy
        },
        team as i32,
    )
}

fn position(config: &SingleLaneConfig, team: usize, offset: Fixed64) -> Vec2 {
    Vec2::new(
        if team == 0 {
            offset
        } else {
            config.lane_length - offset
        },
        Fixed64::ZERO,
    )
}

fn property(hp: Fixed64, speed: Fixed64) -> CProperty {
    CProperty {
        hp,
        mhp: hp,
        msd: speed,
        def_physic: Fixed64::ZERO,
        def_magic: Fixed64::ZERO,
    }
}

/// Transport-free authoritative bootstrap, shared by the CLI and tests. Script
/// handlers come from the caller's generated manifest (real DLL in the CLI).
pub fn create_single_lane_world(
    config: SingleLaneConfig,
    scripts: crate::runtime::ScriptRegistry,
) -> Result<World, Error> {
    validate_single_lane_scripts(&config, &scripts)?;
    let pool = crate::runtime::StateInitializer::create_thread_pool();
    let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
    crate::runtime::populate_tower_template_registry(&mut world, &scripts);
    crate::runtime::populate_tower_upgrade_registry(&mut world);
    crate::runtime::populate_ability_registry(&mut world, &scripts);
    world.insert(scripts);
    setup_single_lane_match(&mut world, config)?;
    Ok(world)
}

/// Shared fail-closed manifest validation for server and transport-free callers.
pub fn validate_single_lane_scripts(
    config: &SingleLaneConfig,
    scripts: &crate::runtime::ScriptRegistry,
) -> Result<(), Error> {
    for id in config.heroes.iter().chain(config.additional_players.iter().map(|p| &p.hero)) {
        let hero_id = omoba_template_ids::hero_by_name(id)
            .ok_or_else(|| err_msg(format!("unknown single-lane hero: {id}")))?;
        for ability in omoba_template_ids::active_hero_abilities(hero_id) {
            if scripts.get_ability(ability.as_str()).is_none() {
                return Err(err_msg(format!(
                    "script registry missing ability {}",
                    ability.as_str()
                )));
            }
        }
    }
    Ok(())
}

/// Installs a match into an initialized, empty campaign world. Validation is
/// atomic: bad configuration must not leave partially spawned entities behind.
pub fn setup_single_lane_match(world: &mut World, config: SingleLaneConfig) -> Result<(), Error> {
    use specs::Join;
    if world.try_fetch::<MobaMatch>().is_some()
        || (&world.entities(), &world.read_storage::<Unit>())
            .join()
            .next()
            .is_some()
    {
        return Err(err_msg("single-lane match requires an empty world"));
    }
    if config.players[0] == config.players[1]
        || config.players.contains(&0)
        || config.teams[0] == config.teams[1]
        || config
            .teams
            .iter()
            .any(|team| !crate::runtime::SUPPORTED_REPLICA_TEAMS.contains(team))
        || config.warmup < Fixed64::ZERO
        || config.respawn_delay <= Fixed64::ZERO
        || config.wave_interval <= Fixed64::ZERO
        || !(1..=16).contains(&config.creeps_per_wave)
        || config.lane_length < Fixed64::from_i32(2000)
        || config.lane_length > Fixed64::from_i32(1_000_000)
        || config.tower_hp <= Fixed64::ZERO
        || config.base_hp <= Fixed64::ZERO
        || config.passive_gold_per_second > 10_000
        || config.hero_kill_gold > 1_000_000
        || config.hero_assist_gold > 1_000_000
        || config.hero_kill_xp > 1_000_000
        || config.hero_assist_xp > 1_000_000
        || config.lane_creep_xp > 1_000_000
        || !(1..=10_000).contains(&config.lane_xp_radius)
        || !(1..=60).contains(&config.assist_window_seconds)
    {
        return Err(err_msg("invalid single-lane match configuration"));
    }
    let routes = match config.map_id.as_deref() {
        None => vec![[
            vec![position(&config,0,Fixed64::ZERO),position(&config,1,Fixed64::ZERO)],
            vec![position(&config,1,Fixed64::ZERO),position(&config,0,Fixed64::ZERO)],
        ]],
        Some(id) => {
            let map = omoba_template_ids::moba_map_by_name(id)
                .ok_or_else(|| err_msg(format!("unknown compiled MOBA map: {id}")))?;
            if config.lane_length != Fixed64::from_i32(map.lane_length) {
                return Err(err_msg("MOBA map base distance differs from match lane_length"));
            }
            map.lanes.iter().map(|lane| {
                let forward: Vec<_> = lane.waypoints.iter().map(|&(x,y)|
                    Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y))).collect();
                let reverse = forward.iter().copied().rev().collect();
                [forward,reverse]
            }).collect()
        }
    };
    let mut roster: Vec<_> = (0..2).map(|side| (config.players[side], side, config.heroes[side].clone())).collect();
    let mut counts = [1_usize; 2];
    for player in &config.additional_players {
        let side = config.teams.iter().position(|team| *team == player.team_id)
            .ok_or_else(|| err_msg("additional single-lane player has unknown team"))?;
        if player.player_id == 0 || roster.iter().any(|(id, _, _)| *id == player.player_id) || counts[side] >= 5 {
            return Err(err_msg("invalid additional single-lane player roster"));
        }
        counts[side] += 1;
        roster.push((player.player_id, side, player.hero.clone()));
    }
    let mut definitions = Vec::new();
    #[cfg(feature = "runtime-lua-content")]
    omoba_template_ids::runtime_content::validate_compiled_moba_catalog().map_err(err_msg)?;
    for (_, _, id) in &roster {
        let hero_id = omoba_template_ids::hero_by_name(id)
            .ok_or_else(|| err_msg(format!("unknown single-lane hero: {id}")))?;
        if omoba_template_ids::active_hero_stats(hero_id).is_none() {
            return Err(err_msg(format!("missing single-lane hero stats: {id}")));
        }
        let mut hero = Hero::from_campaign_data(&crate::runtime::scene::import_campaign::HeroJD {
            id: id.clone(),
            abilities: Vec::new(),
        });
        let loadout = omoba_template_ids::active_hero_stats(hero_id).unwrap().moba_loadout;
        apply_moba_birth_loadout(&mut hero, loadout)?;
        definitions.push(hero);
    }
    let heroes = roster.iter().enumerate().map(|(index, (player_id, side, _))| MobaHeroSlot {
        player_id: *player_id,
        side: *side,
        entity: None,
        deaths: 0,
        kills: 0,
        assists: 0,
        lethal_pending: false,
        recall: None,
        respawns: 0,
        respawn_at: None,
        hero: definitions[index].clone(),
        inventory: Inventory::default(),
        gold: Gold(0),
    }).collect();
    let mut state = MobaMatch {
        config,
        phase: MobaMatchPhase::Warmup,
        elapsed: Fixed64::ZERO,
        income_remainder_raw: 0,
        pending_income_delta_raw: 0,
        waves: 0,
        heroes,
        towers: [None; 2],
        lane_towers: vec![[None;2]; routes.len()],
        routes,
        bases: [None; 2],
        jungle_camps: Vec::new(),
        assist_ledger: super::moba_assist::AssistLedger::default(),
        next_wave_at: Fixed64::ZERO,
        units: BTreeMap::new(),
    };
    if let Some(map) = state.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name) {
        *world.write_resource::<BlockedRegions>() =
            crate::runtime::moba_map_layout::compiled_blocked_regions(map);
    }
    world.write_resource::<crate::runtime::TeamVisibilityRuntime>().fog_geometry =
        state.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name)
            .map(crate::runtime::fog_grid::FogGridGeometry::from_map)
            .transpose().map_err(err_msg)?;
    for team in 0..2 {
        state.spawn_hero(world, team);
        for lane in 0..state.routes.len() {
            let tower = state.spawn_structure(world, team, LaneRole::Tower, lane);
            state.lane_towers[lane][team] = Some(tower);
        }
        state.towers[team] = state.lane_towers[0][team];
        state.bases[team] = Some(state.spawn_structure(world, team, LaneRole::Base, 0));
    }
    for index in 2..state.heroes.len() { state.spawn_hero(world, index); }
    jungle::setup(world, &mut state);
    *world.write_resource::<GameMode>() = GameMode::Moba;
    world.write_resource::<MasterSeed>().0 = state.config.seed;
    world.write_resource::<CurrentCreepWave>().is_running = false;
    world.insert(crate::runtime::ItemRegistry::generated_moba());
    world.insert(state);
    Ok(())
}

/// Validate the entire Lua birth loadout before mutating a hero. Only match
/// setup calls this; respawn restores persistent ranks and never relearns.
pub fn apply_moba_birth_loadout(hero: &mut Hero, loadout: omoba_template_ids::MobaLoadoutConst) -> Result<(), Error> {
    if hero.abilities.len() > 4 { return Err(err_msg("MOBA loadout exceeds four slots")); }
    for (slot, ability) in hero.abilities.iter().enumerate() {
        let rank = loadout.ranks[slot];
        let definition = omoba_template_ids::ability_by_name(ability)
            .and_then(omoba_template_ids::active_ability_const)
            .ok_or_else(|| err_msg(format!("missing initial ability progression: {ability}")))?;
        if rank > definition.max_level {
            return Err(err_msg(format!("initial MOBA rank exceeds max_level: {ability}")));
        }
        if rank > 0 {
            let required = definition.levels.get(usize::from(rank - 1))
                .ok_or_else(|| err_msg("missing initial MOBA rank requirement"))?.required_hero_level;
            if hero.level < i32::from(required) {
                return Err(err_msg(format!("prelearned single-lane ability '{ability}' requires hero level {required}")));
            }
        }
    }
    // Default rank one is a compatibility sentinel for absent legacy slots.
    if loadout.ranks[hero.abilities.len()..].iter().any(|rank| *rank > 1) {
        return Err(err_msg("MOBA loadout assigns rank to an empty slot"));
    }
    for (slot, ability) in hero.abilities.iter().enumerate() {
        hero.ability_levels.insert(ability.clone(), i32::from(loadout.ranks[slot]));
    }
    hero.skill_points = i32::from(loadout.skill_points);
    Ok(())
}

impl MobaMatch {
    fn track(&mut self, entity: Entity, team: usize, role: LaneRole) {
        self.units.insert(
            entity_key(entity),
            LaneUnit {
                entity,
                team: team as u8,
                role,
                attack_remaining: Fixed64::ZERO,
                lane: 0,
                next_waypoint: 1,
            },
        );
    }

    fn spawn_hero(&mut self, world: &mut World, index: usize) {
        let slot = &self.heroes[index];
        let team = slot.side;
        let rank = self.heroes[..index].iter().filter(|s| s.side == team).count();
        let mut spawn = position(&self.config, team, Fixed64::from_i32(120));
        spawn.y += Fixed64::from_i32(rank as i32 * 60);
        let id = omoba_template_ids::hero_by_name(&slot.hero.id).expect("validated hero");
        let stats = omoba_template_ids::active_hero_stats(id).expect("validated stats");
        let level_bonus = slot.hero.level.saturating_sub(1);
        let hp =
            Fixed64::from_i32(stats.base_hp) + slot.hero.level_growth.hp_per_level * level_bonus;
        let attack = Fixed64::from_i32(stats.base_damage)
            + slot.hero.level_growth.damage_per_level * level_bonus;
        let mut unit = Unit::new(slot.hero.id.clone(), slot.hero.name.clone(), UnitType::Hero);
        unit.max_hp = (hp.raw() / 1024) as i32;
        unit.current_hp = unit.max_hp;
        unit.base_damage = (attack.raw() / 1024) as i32;
        unit.base_armor = stats.base_armor;
        unit.attack_range = stats.attack_range;
        unit.move_speed = stats.move_speed;
        unit.abilities = slot.hero.abilities.clone();
        let mut props = property(hp, stats.move_speed);
        props.def_physic = stats.base_armor;
        let entity = world
            .create_entity()
            .with(slot.hero.clone())
            .with(unit)
            .with(PlayerOwner::new(slot.player_id))
            .with(faction(team as u8, self.config.teams[team]))
            .with(ReplicationScope {
                kind: ReplicationScopeKind::OwnerTeam,
                owner_team: Some(self.config.teams[team]),
            })
            .with(VisionSource {
                team: self.config.teams[team],
                radius: Fixed64::from_i32(1000),
                detection_level: 0,
            })
            .with(Pos(spawn))
            .with(props)
            .with(TAttack::new(
                attack,
                Fixed64::ONE,
                stats.attack_range,
                Fixed64::from_i32(1200),
            ))
            .with(Facing::from_rad_f32(if team == 0 {
                0.0
            } else {
                std::f32::consts::PI
            }))
            .with(FacingBroadcast::default())
            .with(TurnSpeed(stats.turn_speed))
            .with(CollisionRadius(Fixed64::from_i32(20)))
            .with(CircularVision::new(1000.0, 800.0))
            .with(slot.inventory.clone())
            .with(slot.gold)
            .with(ItemEffects {
                dirty: true,
                ..ItemEffects::default()
            })
            .build();
        self.heroes[index].entity = Some(entity);
        self.heroes[index].lethal_pending = false;
        self.heroes[index].recall = None;
        self.heroes[index].respawn_at = None;
        self.track(entity, team, LaneRole::Hero);
    }

    fn spawn_structure(&mut self, world: &mut World, team: usize, role: LaneRole, lane: usize) -> Entity {
        let (id, hp, offset) = match role {
            LaneRole::Tower => (
                "single_lane_tower",
                self.config.tower_hp,
                self.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name)
                    .map_or(Fixed64::from_i32(700), |map| Fixed64::from_i32(map.tower_offset)),
            ),
            LaneRole::Base => ("single_lane_base", self.config.base_hp, Fixed64::ZERO),
            _ => unreachable!(),
        };
        let mut unit = Unit::new(id.into(), id.into(), UnitType::Neutral);
        unit.max_hp = (hp.raw() / 1024) as i32;
        unit.current_hp = unit.max_hp;
        unit.base_damage = if role == LaneRole::Tower { 90 } else { 0 };
        unit.attack_range = Fixed64::from_i32(if role == LaneRole::Tower { 650 } else { 0 });
        unit.move_speed = Fixed64::ZERO;
        let route = &self.routes[lane][team];
        let mut cursor = 1;
        let spawn = omoba_sim::navigation::advance_route(route,&mut cursor,route[0],offset);
        let mut builder = world
            .create_entity()
            .with(unit)
            .with(faction(team as u8, self.config.teams[team]))
            .with(ReplicationScope {
                kind: ReplicationScopeKind::OwnerTeam,
                owner_team: Some(self.config.teams[team]),
            })
            .with(VisionSource {
                team: self.config.teams[team],
                radius: Fixed64::from_i32(1000),
                detection_level: 0,
            })
            .with(Pos(spawn))
            .with(property(hp, Fixed64::ZERO))
            .with(IsBuilding)
            .with(CircularVision::new(1000.0, 800.0));
        if role == LaneRole::Base {
            builder = builder.with(IsBase);
        }
        let entity = builder.build();
        self.track(entity, team, role);
        self.units.get_mut(&entity_key(entity)).expect("tracked structure").lane = lane;
        entity
    }

    fn spawn_wave(&mut self, world: &mut World) {
        self.waves += 1;
        for team in 0..2 {
          for lane in 0..self.routes.len() {
            for index in 0..self.config.creeps_per_wave {
                let route = &self.routes[lane][team];
                let mut cursor = 1;
                let spawn = omoba_sim::navigation::advance_route(route,&mut cursor,route[0],
                    Fixed64::from_i32(180 + index as i32 * 35));
                let mut unit = Unit::new(
                    "single_lane_creep".into(),
                    "Lane creep".into(),
                    UnitType::Creep,
                );
                unit.max_hp = 180;
                unit.current_hp = 180;
                unit.base_damage = 24;
                unit.attack_range = Fixed64::from_i32(140);
                unit.move_speed = Fixed64::from_i32(240);
                let entity = world
                    .create_entity()
                    .with(unit)
                    .with(Creep {
                        name: "single_lane_creep".into(),
                        label: None,
                        // No legacy TD path: lane rules own this unit's movement.
                        path: if self.config.map_id.is_none() { "__single_lane__".into() }
                            else { format!("__moba_lane_{lane}__") },
                        pidx: 0,
                        path_remaining_distance: self.config.lane_length,
                        block_tower: None,
                        status: CreepStatus::Walk,
                        td_layer: None,
                    })
                    .with(faction(team as u8, self.config.teams[team]))
                    .with(ReplicationScope {
                        kind: ReplicationScopeKind::OwnerTeam,
                        owner_team: Some(self.config.teams[team]),
                    })
                    .with(VisionSource {
                        team: self.config.teams[team],
                        radius: Fixed64::from_i32(700),
                        detection_level: 0,
                    })
                    .with(Pos(spawn))
                    .with(property(Fixed64::from_i32(180), Fixed64::from_i32(240)))
                    .with(CircularVision::new(700.0, 500.0))
                    .build();
                self.track(entity, team, LaneRole::Creep);
                let tracked = self.units.get_mut(&entity_key(entity)).expect("tracked creep");
                tracked.lane = lane;
                tracked.next_waypoint = cursor;
            }
          }
        }
    }
}

/// Pre-dispatch hook. Timers use accumulated scaled fixed delta, never wall time.
/// Returns false outside active play so callers can freeze the whole gameplay
/// pipeline, not merely the lane AI. Existing worlds without this resource pass.
pub fn begin_moba_match_tick(world: &mut World) -> bool {
    // Results are never replay state and must not survive an inactive tick.
    world
        .write_resource::<PendingItemUseQueue>()
        .settlements
        .clear();
    let Some(mut state) = world.remove::<MobaMatch>() else {
        return true;
    };
    state.pending_income_delta_raw = 0;
    #[cfg(feature = "kcp")]
    world
        .write_resource::<PendingPlayerInputs>()
        .inputs
        .retain(|(player, input)| state.heroes.iter().any(|s| s.player_id == *player) && single_lane_input_action_allowed(input));
    if matches!(state.phase, MobaMatchPhase::Finished { .. }) {
        reject_inactive_shop_inputs(world);
        emit_replica_delta(world, &state, false);
        world.insert(state);
        return false;
    }
    let dt = world.read_resource::<DeltaTime>().0;
    let previous_elapsed = state.elapsed;
    state.elapsed += dt;
    if state.elapsed < state.config.warmup || dt <= Fixed64::ZERO {
        reject_inactive_shop_inputs(world);
        emit_replica_delta(world, &state, false);
        world.insert(state);
        return false;
    }
    if state.phase == MobaMatchPhase::Warmup {
        state.phase = MobaMatchPhase::Playing;
        state.next_wave_at = state.elapsed;
    }
    // Count only the playing portion of the warmup-crossing tick.
    state.pending_income_delta_raw = (state.elapsed - previous_elapsed.max(state.config.warmup))
        .raw()
        .max(0);
    #[cfg(feature = "kcp")]
    {
        use crate::runtime::PlayerInputEnum as I;
        // Gameplay commands win over Recall within the same input batch,
        // independent of ordering; none may be deferred behind a channel.
        let interrupted: std::collections::BTreeSet<u32> = world.read_resource::<PendingPlayerInputs>().inputs.iter()
            .filter_map(|(player, input)| matches!(input.action, Some(I::MoveTo(_) | I::AttackMove(_) | I::AttackTarget(_) | I::CastAbility(_) | I::ItemUse(_))).then_some(*player)).collect();
        world.write_resource::<PendingPlayerInputs>().inputs.retain(|(player, input)| {
            let side = state.heroes.iter().position(|s| s.player_id == *player).expect("roster filtered");
            let team = state.heroes[side].side;
            if matches!(input.action, Some(I::Recall(_))) {
                if interrupted.contains(player) { state.heroes[side].recall = None; return false; }
                if let Some(entity) = state.heroes[side].entity {
                    if world.read_storage::<CProperty>().get(entity).is_some_and(|p| p.hp > Fixed64::ZERO)
                        && state.bases[team].is_some() && state.heroes[side].recall.is_none() {
                        let origin = world.read_storage::<Pos>().get(entity).expect("hero position").0;
                        state.heroes[side].recall = Some((origin, previous_elapsed.max(state.config.warmup) + Fixed64::from_i32(omoba_template_ids::MOBA_RECALL_CHANNEL_SECONDS as i32)));
                        if let Some(queue) = world.write_storage::<HeroCommandQueue>().get_mut(entity) { queue.clear_all(); }
                        world.write_storage::<MoveTarget>().remove(entity);
                    }
                }
                return false;
            }
            if matches!(input.action, Some(I::MoveTo(_) | I::AttackMove(_) | I::AttackTarget(_) | I::CastAbility(_) | I::ItemUse(_))) {
                state.heroes[side].recall = None;
            }
            true
        });
    }
    emit_replica_delta(world, &state, true);
    for team in 0..state.heroes.len() {
        if state.heroes[team].entity.is_none() {
            state.heroes[team].hero.tick_cooldowns(dt);
        }
        if state.heroes[team]
            .respawn_at
            .is_some_and(|time| state.elapsed >= time)
        {
            state.heroes[team].respawns = state.heroes[team].respawns.saturating_add(1);
            state.spawn_hero(world, team);
        }
    }
    while state.elapsed >= state.next_wave_at {
        state.spawn_wave(world);
        state.next_wave_at += state.config.wave_interval;
    }
    jungle::tick(world, &mut state, dt);
    // Units and decisions are always reduced in (entity ID, generation) order.
    let positions = world.read_storage::<Pos>();
    let properties = world.read_storage::<CProperty>();
    let regions = world.read_resource::<BlockedRegions>();
    let mut movement = Vec::new();
    let mut damage = Vec::new();
    let units: Vec<_> = state.units.values().cloned().collect();
    for unit in &units {
        if !matches!(unit.role, LaneRole::Creep | LaneRole::Tower)
            || properties
                .get(unit.entity)
                .is_none_or(|p| p.hp <= Fixed64::ZERO)
        {
            continue;
        }
        let source_pos = positions.get(unit.entity).expect("lane unit position").0;
        let range = Fixed64::from_i32(if unit.role == LaneRole::Tower {
            650
        } else {
            140
        });
        let aggro = Fixed64::from_i32(if unit.role == LaneRole::Tower {
            650
        } else {
            700
        });
        let target = units
            .iter()
            .filter(|other| other.team != unit.team)
            .filter(|other| {
                properties
                    .get(other.entity)
                    .is_some_and(|p| p.hp > Fixed64::ZERO)
            })
            .filter(|other| {
                other.role != LaneRole::Base || state.base_unlocked(other.team as usize)
            })
            .filter(|other| matches!(other.role,LaneRole::Hero | LaneRole::Base) || other.lane == unit.lane)
            .filter_map(|other| {
                positions
                    .get(other.entity)
                    .map(|p| (other, p.0, (p.0 - source_pos).length_squared()))
            })
            .filter(|(_, _, distance)| *distance <= aggro * aggro)
            // Towers protect the lane by targeting minions before heroes.
            .min_by_key(|(other, _, distance)| {
                (
                    unit.role == LaneRole::Tower && other.role != LaneRole::Creep,
                    distance.raw(),
                    entity_key(other.entity),
                )
            });
        let tracked = state
            .units
            .get_mut(&entity_key(unit.entity))
            .expect("tracked unit");
        tracked.attack_remaining = (tracked.attack_remaining - dt).max(Fixed64::ZERO);
        if let Some((target, _, distance)) = target {
            if distance <= range * range {
                if tracked.attack_remaining <= Fixed64::ZERO {
                    damage.push(DamageInstance::new_attack(
                        unit.entity,
                        target.entity,
                        Fixed64::from_i32(if unit.role == LaneRole::Tower { 90 } else { 24 }),
                    ));
                    tracked.attack_remaining = Fixed64::ONE;
                }
                continue;
            }
        }
        if unit.role == LaneRole::Creep {
            let step = properties.get(unit.entity).expect("lane property").msd * dt;
            let pos = if let Some((_,destination,_)) = target {
                if state.config.map_id.is_none() {
                    let diff = destination - source_pos;
                    let distance = diff.length();
                    if distance > Fixed64::ZERO { source_pos + diff * (step.min(distance) / distance) }
                    else { source_pos }
                } else { crate::tick::hero_command_tick::static_step_toward(source_pos,destination,step,
                    Fixed64::from_i32(20), &regions) }
            } else if state.config.map_id.is_none() {
                // Preserve the previously verified straight-lane integration.
                let destination = position(&state.config,1 - unit.team as usize,Fixed64::ZERO);
                let diff = destination - source_pos;
                let distance = diff.length();
                if distance > Fixed64::ZERO { source_pos + diff * (step.min(distance) / distance) }
                else { source_pos }
            } else {
                crate::tick::hero_command_tick::static_advance_route(&state.routes[unit.lane][unit.team as usize],
                    &mut tracked.next_waypoint,source_pos,step,Fixed64::from_i32(20), &regions)
            };
            if pos != source_pos { movement.push((unit.entity,pos)); }
        }
    }
    drop(properties);
    drop(positions);
    drop(regions);
    for (entity, pos) in movement {
        if let Some(current) = world.write_storage::<Pos>().get_mut(entity) {
            current.0 = pos;
        }
        let source = crate::runtime::canonical_entity_id(entity);
        world
            .read_resource::<crate::runtime::ObservableFactBuffer>()
            .emit(crate::runtime::OrderedFact {
                key: crate::runtime::FactOrderingKey {
                    tick: world.read_resource::<Tick>().0,
                    phase: crate::runtime::FactPhase::PreStep,
                    canonical_source_order: source,
                    local_ordinal: 0,
                    fact_kind: crate::runtime::FactKind::PreStepMovement,
                },
                audience: crate::runtime::FactAudience::VisibilityPolicy(
                    omb_script_abi::types::projection_policy_ids::MOVEMENT.to_owned(),
                ),
                fact: crate::runtime::ObservableFact::Movement {
                    source,
                    x_mm: pos.x.raw(),
                    y_mm: pos.y.raw(),
                    facing_ticks: None,
                },
            })
            .expect("valid lane movement fact");
    }
    world.write_resource::<Vec<DamageInstance>>().extend(damage);
    world.insert(state);
    true
}

fn reject_inactive_shop_inputs(world: &mut World) {
    #[cfg(feature = "kcp")]
    {
        use crate::runtime::{
            shop::{ShopCommand, ShopError, ShopSettlement},
            PlayerInputEnum,
        };
        let results: Vec<_> = world
            .read_resource::<PendingPlayerInputs>()
            .inputs
            .iter()
            .filter_map(|(player, input)| {
                let command = match &input.action {
                    Some(PlayerInputEnum::ItemBuy(buy)) => ShopCommand::Buy(buy.item_id.clone()),
                    Some(PlayerInputEnum::ItemSell(sell)) => {
                        ShopCommand::Sell(sell.item_slot as usize)
                    }
                    _ => return None,
                };
                Some(ShopSettlement {
                    player_id: *player,
                    command,
                    result: Err(ShopError::MatchUnavailable),
                })
            })
            .collect();
        world
            .write_resource::<PendingItemUseQueue>()
            .settlements
            .extend(results);
    }
}

fn award_live_moba_xp(world: &World, entity: Entity, amount: u32) {
    let growth = {
        let mut heroes = world.write_storage::<Hero>();
        let Some(hero) = heroes.get_mut(entity) else { return; };
        let before = hero.level;
        hero.add_moba_experience(amount);
        let levels = hero.level - before;
        (hero.level_growth.hp_per_level * levels, hero.level_growth.damage_per_level * levels)
    };
    // Apply only level deltas; item bonuses remain intact. XP is not a heal and
    // cannot resurrect an entity already queued for death in this outcome batch.
    if let Some(prop) = world.write_storage::<CProperty>().get_mut(entity) { prop.mhp += growth.0; }
    if let Some(attack) = world.write_storage::<TAttack>().get_mut(entity) {
        attack.atk_physic = Vf32::new(attack.atk_physic.clone().val() + growth.1);
    }
}

/// Called after actual positive damage to a live target, not after an input ACK.
/// Anonymous script damage can retire a life but cannot claim participation.
pub(crate) fn record_moba_hero_damage(world: &World, source: Option<Entity>, target: Entity, lethal: bool) {
    use super::moba_assist::AssistParticipant;
    let Some(mut state) = world.try_fetch_mut::<MobaMatch>() else { return; };
    if state.phase != MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused {
        return;
    }
    let Some(victim) = state.heroes.iter().position(|slot| slot.entity == Some(target)) else { return; };
    if state.heroes[victim].lethal_pending { return; }
    let roster: Vec<_> = state.heroes.iter().map(|slot| AssistParticipant {
        player_id: slot.player_id, team_id: state.config.teams[slot.side],
    }).collect();
    let killer = source.and_then(|entity| state.heroes.iter().position(|slot| slot.entity == Some(entity)))
        .filter(|index| *index != victim && roster[*index].team_id != roster[victim].team_id);
    let key = crate::runtime::canonical_entity_id(target);
    let now = state.elapsed.raw();
    if let Some(side) = killer { state.assist_ledger.record(key, roster[victim], roster[side], now, true); }
    if !lethal { return; }
    // Even an uncredited lethal hit retires participation. Heal cannot turn a
    // queued death into a second reward for this life.
    state.heroes[victim].lethal_pending = true;
    let window = i64::from(state.config.assist_window_seconds) * 1024;
    let assistants = state.assist_ledger.settle(key, killer.map(|side| roster[side]), now, window, &roster);
    let Some(killer) = killer else { return; };
    let mut gold = world.write_storage::<Gold>();
    let Some(balance) = gold.get_mut(source.expect("credited source")) else { return; };
    balance.0 = balance.0.saturating_add(state.config.hero_kill_gold as i32);
    state.heroes[killer].kills = state.heroes[killer].kills.saturating_add(1);
    award_live_moba_xp(world, source.expect("credited source"), state.config.hero_kill_xp);
    let assist_gold = state.config.hero_assist_gold as i32;
    let assist_xp = state.config.hero_assist_xp;
    for player in assistants {
        let slot = state.heroes.iter_mut().find(|slot| slot.player_id == player).expect("validated roster");
        if let Some(entity) = slot.entity {
            if let Some(balance) = gold.get_mut(entity) { balance.0 = balance.0.saturating_add(assist_gold); }
            else { continue; }
            award_live_moba_xp(world, entity, assist_xp);
        } else {
            slot.gold.0 = slot.gold.0.saturating_add(assist_gold);
            slot.hero.add_moba_experience(assist_xp);
        }
        slot.assists = slot.assists.saturating_add(1);
    }
}

pub(crate) fn interrupt_moba_recall(world: &World, target: Entity) {
    if let Some(mut state) = world.try_fetch_mut::<MobaMatch>() {
        if let Some(slot) = state.heroes.iter_mut().find(|s| s.entity == Some(target)) { slot.recall = None; }
    }
}

/// Captures progression before the normal death outcome deletes the entity.
/// Duplicate death outcomes cannot schedule multiple respawns.
pub fn record_moba_death(world: &mut World, entity: Entity) {
    let Some(mut state) = world.remove::<MobaMatch>() else {
        return;
    };
    jungle::death(world, &mut state, entity);
    if let Some(unit) = state.units.remove(&entity_key(entity)) {
        state.assist_ledger.retire(crate::runtime::canonical_entity_id(entity));
        let team = unit.team as usize;
        match unit.role {
            LaneRole::Hero => {
                let slot = state.heroes.iter_mut().find(|slot| slot.entity == Some(entity)).expect("tracked hero slot");
                slot.hero = world
                    .read_storage::<Hero>()
                    .get(entity)
                    .expect("dying hero")
                    .clone();
                slot.inventory = world
                    .read_storage::<Inventory>()
                    .get(entity)
                    .cloned()
                    .unwrap_or_default();
                slot.gold = world
                    .read_storage::<Gold>()
                    .get(entity)
                    .copied()
                    .unwrap_or_default();
                slot.entity = None;
                slot.recall = None;
                slot.deaths = slot.deaths.saturating_add(1);
                slot.respawn_at = Some(state.elapsed + state.config.respawn_delay);
                world
                    .write_resource::<crate::runtime::BuffStore>()
                    .remove_all_for(entity);
            }
            LaneRole::Tower => {
                state.lane_towers[unit.lane][team] = None;
                state.towers[team] = state.lane_towers.iter().find_map(|lane| lane[team]);
            }
            LaneRole::Base => state.bases[team] = None,
            LaneRole::Creep => {
                if state.phase == MobaMatchPhase::Playing
                    && !world.read_resource::<GamePause>().is_paused
                    && world.read_storage::<CProperty>().get(entity).is_some_and(|p| p.hp <= Fixed64::ZERO)
                {
                    if let Some(origin) = world.read_storage::<Pos>().get(entity).map(|p| p.0) {
                        let radius = Fixed64::from_i32(state.config.lane_xp_radius as i32);
                        let recipients: Vec<_> = state.heroes.iter().filter(|slot|
                            slot.side != team && !slot.lethal_pending).filter_map(|slot| slot.entity)
                            .filter(|hero| world.entities().is_alive(*hero)
                                && world.read_storage::<CProperty>().get(*hero).is_some_and(|p| p.hp > Fixed64::ZERO)
                                && world.read_storage::<Pos>().get(*hero).is_some_and(|p|
                                    (p.0 - origin).length_squared() <= radius * radius))
                            .collect();
                        if !recipients.is_empty() {
                            // Equal floor shares, no iteration-order remainder bonus.
                            let share = state.config.lane_creep_xp / recipients.len() as u32;
                            for hero in recipients { award_live_moba_xp(world, hero, share); }
                        }
                    }
                }
            }
        }
    }
    world.insert(state);
}

/// Checked at the damage boundary, not only by AI. A client cannot bypass the
/// tower prerequisite by issuing a direct attack/skill on the base.
pub fn moba_damage_allowed(world: &World, target: Entity) -> bool {
    world.try_fetch::<MobaMatch>().is_none_or(|state| {
        state.phase == MobaMatchPhase::Playing
            && !state.jungle_camps.iter().any(|camp| camp.entity == Some(target) && camp.returning)
            && state.units.get(&entity_key(target)).is_none_or(|unit| {
                unit.role != LaneRole::Base || state.base_unlocked(unit.team as usize)
            })
    })
}

/// Commit hook. Both destroyed bases in the same tick produce a draw.
pub fn finish_moba_match_tick(world: &mut World) {
    let tick = world.read_resource::<Tick>().0;
    let Some(mut state) = world.try_fetch_mut::<MobaMatch>() else {
        return;
    };
    if state.phase != MobaMatchPhase::Playing {
        return;
    }
    if !world.read_resource::<GamePause>().is_paused {
        for side in 0..state.heroes.len() {
            let team = state.heroes[side].side;
            let Some((origin, deadline)) = state.heroes[side].recall else { continue; };
            let valid = state.heroes[side].entity.filter(|entity| {
                world.read_storage::<CProperty>().get(*entity).is_some_and(|p| p.hp > Fixed64::ZERO)
                    && world.read_storage::<Pos>().get(*entity).is_some_and(|p| p.0 == origin)
            });
            if valid.is_none() || state.bases[team].is_none() {
                state.heroes[side].recall = None;
            } else if state.elapsed >= deadline {
                let base = state.bases[team].unwrap();
                let destination = world.read_storage::<Pos>().get(base).expect("base position").0;
                world.write_storage::<Pos>().get_mut(valid.unwrap()).expect("hero position").0 = destination;
                state.heroes[side].recall = None;
            }
        }
    }
    // Commit once, after gameplay; replicas receive the settled owner economy
    // rather than independently predicting income or mutating private timers.
    let delta = std::mem::take(&mut state.pending_income_delta_raw);
    let accrued = i128::from(state.income_remainder_raw) + i128::from(delta);
    let seconds = accrued / i128::from(omoba_sim::fixed::SCALE);
    state.income_remainder_raw = (accrued % i128::from(omoba_sim::fixed::SCALE)) as i64;
    let award = (seconds * i128::from(state.config.passive_gold_per_second))
        .min(i128::from(i32::MAX)) as i32;
    if award > 0 {
        let mut gold = world.write_storage::<Gold>();
        for slot in &mut state.heroes {
            let balance = match slot.entity {
                Some(entity) => gold.get_mut(entity).expect("live hero gold"),
                None => &mut slot.gold,
            };
            balance.0 = balance.0.saturating_add(award);
        }
    }
    // Hero movement can finish before the final turn/attack phase. Publish the
    // committed visible pose, not an intermediate system pose or private target.
    let positions = world.read_storage::<Pos>();
    let facings = world.read_storage::<Facing>();
    let attacks = world.read_storage::<TAttack>();
    for hero in &state.heroes {
        if let Some(entity) = hero.entity {
            let source = crate::runtime::canonical_entity_id(entity);
            let economy =
                crate::runtime::native::economy_projection::CommittedEconomyState::capture(
                    *world
                        .read_storage::<Gold>()
                        .get(entity)
                        .expect("MOBA hero gold"),
                    world
                        .read_storage::<Inventory>()
                        .get(entity)
                        .expect("MOBA hero inventory"),
                    *world
                        .read_storage::<ItemEffects>()
                        .get(entity)
                        .expect("MOBA item effects"),
                )
                .expect("valid MOBA economy state");
            let team = state.config.teams[hero.side];
            world
                .read_resource::<crate::runtime::ObservableFactBuffer>()
                .emit(crate::runtime::OrderedFact {
                    key: crate::runtime::FactOrderingKey {
                        tick,
                        phase: crate::runtime::FactPhase::PostStep,
                        canonical_source_order: source,
                        local_ordinal: 0,
                        fact_kind: crate::runtime::FactKind::CommittedEconomy,
                    },
                    audience: crate::runtime::FactAudience::Team(team),
                    fact: crate::runtime::ObservableFact::CommittedEconomy {
                        source,
                        state: economy,
                    },
                })
                .expect("valid owner-team economy settlement");
            if let (Some(property), Some(attack)) = (
                world.read_storage::<CProperty>().get(entity),
                attacks.get(entity),
            ) {
                world
                    .read_resource::<crate::runtime::ObservableFactBuffer>()
                    .emit(crate::runtime::OrderedFact {
                        key: crate::runtime::FactOrderingKey {
                            tick,
                            phase: crate::runtime::FactPhase::PostStep,
                            canonical_source_order: source,
                            local_ordinal: 0,
                            fact_kind: crate::runtime::FactKind::CommittedEquipmentStats,
                        },
                        audience: crate::runtime::FactAudience::VisibilityPolicy(
                            omb_script_abi::types::projection_policy_ids::HERO_ABILITY.to_owned(),
                        ),
                        fact: crate::runtime::ObservableFact::CommittedEquipmentStats {
                            source,
                            hp_raw: property.hp.raw(),
                            max_hp_raw: property.mhp.raw(),
                            speed_raw: property.msd.raw(),
                            armor_raw: property.def_physic.raw(),
                            attack_raw: attack.atk_physic.clone().val().raw(),
                        },
                    })
                    .expect("valid visible equipment stats");
            }
            if let Some(progression) = world.read_storage::<Hero>().get(entity) {
                let source = crate::runtime::canonical_entity_id(entity);
                world
                    .read_resource::<crate::runtime::ObservableFactBuffer>()
                    .emit(crate::runtime::OrderedFact {
                        key: crate::runtime::FactOrderingKey {
                            tick,
                            phase: crate::runtime::FactPhase::PostStep,
                            canonical_source_order: source,
                            local_ordinal: 0,
                            fact_kind: crate::runtime::FactKind::CommittedProgression,
                        },
                        audience: crate::runtime::FactAudience::VisibilityPolicy(
                            omb_script_abi::types::projection_policy_ids::HERO_ABILITY.to_owned(),
                        ),
                        fact: crate::runtime::ObservableFact::CommittedProgression {
                            source,
                            level: progression.level,
                            experience: progression.experience,
                            experience_to_next: progression.experience_to_next,
                            skill_points: progression.skill_points,
                        },
                    })
                    .expect("valid disclosed hero progression");
                let ranks = std::array::from_fn(|slot| progression.abilities.get(slot)
                    .map_or(0, |id| progression.get_ability_level(id)));
                world.read_resource::<crate::runtime::ObservableFactBuffer>()
                    .emit(crate::runtime::OrderedFact {
                        key: crate::runtime::FactOrderingKey {tick,
                            phase:crate::runtime::FactPhase::PostStep,canonical_source_order:source,
                            local_ordinal:0,fact_kind:crate::runtime::FactKind::CommittedAbilityRanks},
                        audience:crate::runtime::FactAudience::VisibilityPolicy(
                            omb_script_abi::types::projection_policy_ids::HERO_ABILITY.to_owned()),
                        fact:crate::runtime::ObservableFact::CommittedAbilityRanks {source,ranks},
                    }).expect("valid disclosed ability ranks");
            }
            if let Some(attack) = attacks.get(entity) {
                let source = crate::runtime::canonical_entity_id(entity);
                world
                    .read_resource::<crate::runtime::ObservableFactBuffer>()
                    .emit(crate::runtime::OrderedFact {
                        key: crate::runtime::FactOrderingKey {
                            tick,
                            phase: crate::runtime::FactPhase::PostStep,
                            canonical_source_order: source,
                            local_ordinal: 0,
                            fact_kind: crate::runtime::FactKind::CommittedAttack,
                        },
                        audience: crate::runtime::FactAudience::VisibilityPolicy(
                            omb_script_abi::types::projection_policy_ids::HERO_ABILITY.to_owned(),
                        ),
                        fact: crate::runtime::ObservableFact::CommittedAttack {
                            source,
                            elapsed_raw: attack.asd_count.raw(),
                            sequence: attack.attack_seq,
                            phase: match attack.attack_phase {
                                AttackSequencePhase::Idle => 0,
                                AttackSequencePhase::Windup => 1,
                                AttackSequencePhase::Backswing => 2,
                            },
                        },
                    })
                    .expect("valid disclosed hero attack clock");
            }
            if let Some(pos) = positions.get(entity) {
                let source = crate::runtime::canonical_entity_id(entity);
                world
                    .read_resource::<crate::runtime::ObservableFactBuffer>()
                    .emit(crate::runtime::OrderedFact {
                        key: crate::runtime::FactOrderingKey {
                            tick,
                            phase: crate::runtime::FactPhase::PostStep,
                            canonical_source_order: source,
                            local_ordinal: 0,
                            fact_kind: crate::runtime::FactKind::Movement,
                        },
                        audience: crate::runtime::FactAudience::VisibilityPolicy(
                            omb_script_abi::types::projection_policy_ids::MOVEMENT.to_owned(),
                        ),
                        fact: crate::runtime::ObservableFact::Movement {
                            source,
                            x_mm: pos.0.x.raw(),
                            y_mm: pos.0.y.raw(),
                            facing_ticks: facings.get(entity).map(|facing| facing.0.ticks()),
                        },
                    })
                    .expect("valid committed visible hero pose");
            }
        }
    }
    drop(positions);
    drop(facings);
    drop(attacks);
    let winner = match state.bases {
        [None, None] => Some(None),
        [None, Some(_)] => Some(Some(1)),
        [Some(_), None] => Some(Some(0)),
        _ => None,
    };
    if let Some(winner) = winner {
        state.phase = MobaMatchPhase::Finished { winner, tick };
    }
    emit_owner_economy(world, &state, crate::runtime::FactPhase::PostStep);
    emit_owner_score(world, &state, crate::runtime::FactPhase::PostStep);
    emit_recall_state(world, &state, crate::runtime::FactPhase::PostStep);
    if let Some(winner) = winner {
        state.phase = MobaMatchPhase::Finished { winner, tick };
        let winner_team = winner.map(|side| state.config.teams[side as usize]);
        drop(state);
        world
            .read_resource::<crate::runtime::ObservableFactBuffer>()
            .emit(crate::runtime::OrderedFact {
                key: crate::runtime::FactOrderingKey {
                    tick,
                    phase: crate::runtime::FactPhase::PostStep,
                    canonical_source_order: 0,
                    local_ordinal: 0,
                    fact_kind: crate::runtime::FactKind::Terminal,
                },
                audience: crate::runtime::FactAudience::AllPlayers,
                fact: crate::runtime::ObservableFact::Terminal {
                    result_code: if winner_team.is_some() { 1 } else { 2 },
                    winning_team: winner_team,
                },
            })
            .expect("valid public match terminal");
        world
            .write_resource::<crate::runtime::RuntimeEvents>()
            .push(
                crate::runtime::RuntimeEvent::new(
                    "game.end",
                    "game",
                    "end",
                    serde_json::json!({"winner_team": winner_team, "tick": tick}),
                )
                .with_broadcast(crate::runtime::RuntimeBroadcast::All),
            );
    }
}

/// Canonical trace digest for the lane rules and their live ECS combat state.
/// This is a replay diagnostic, not the selective-replica wire hash.
pub fn single_lane_replay_digest(world: &World) -> String {
    use sha2::{Digest, Sha256};
    let state = world.read_resource::<MobaMatch>();
    let mut hash = Sha256::new();
    hash.update(format!("|jungle:{:?}", state.jungle_camps));
    for camp in &state.jungle_camps {
        if let Some(entity) = camp.entity {
            hash_json(&mut hash, &serde_json::to_value((world.read_storage::<Pos>().get(entity),
                world.read_storage::<CProperty>().get(entity))).expect("jungle replay JSON"));
        }
    }
    hash.update(format!("|map:{:?}|routes:{:?}|towers:{:?}",state.config.map_id,state.routes,state.lane_towers));
    hash.update(format!(
        "{:?}|{}|{}|{}|{}",
        state.phase,
        state.elapsed.raw(),
        state.next_wave_at.raw(),
        state.waves,
        state.config.seed
    ));
    hash.update(format!(
        "|income:{}|{}|{}|kill-gold:{}",
        state.config.passive_gold_per_second,
        state.income_remainder_raw,
        state.pending_income_delta_raw,
        state.config.hero_kill_gold
    ));
    for slot in &state.heroes {
        hash.update(format!("|side:{}", slot.side));
        hash.update(format!("|assists:{}", slot.assists));
        hash.update(format!(
            "|{}|{:?}|{}|{}|{:?}|{}|{}|kills:{}|lethal:{}|recall:{:?}",
            slot.player_id,
            slot.entity,
            slot.deaths,
            slot.respawns,
            slot.respawn_at,
            slot.hero.level,
            slot.hero.experience,
            slot.kills,
            slot.lethal_pending,
            slot.recall
        ));
        hash_json(
            &mut hash,
            &serde_json::to_value((&slot.hero, &slot.inventory, slot.gold)).expect("slot JSON"),
        );
    }
    hash.update(format!("|assist-rules:{}:{}|ledger:{:?}", state.config.hero_assist_gold,
        state.config.assist_window_seconds, state.assist_ledger));
    hash.update(format!("|xp-rules:{}:{}", state.config.hero_kill_xp, state.config.hero_assist_xp));
    hash.update(format!("|lane-xp-rules:{}:{}", state.config.lane_creep_xp, state.config.lane_xp_radius));
    let positions = world.read_storage::<Pos>();
    let properties = world.read_storage::<CProperty>();
    let heroes = world.read_storage::<Hero>();
    let commands = world.read_storage::<HeroCommandQueue>();
    let attacks = world.read_storage::<TAttack>();
    for (key, unit) in &state.units {
        hash.update(format!("|lane:{}|waypoint:{}",unit.lane,unit.next_waypoint));
        let pos = positions.get(unit.entity).expect("lane position");
        let hp = properties.get(unit.entity).expect("lane hp");
        hash_json(
            &mut hash,
            &serde_json::to_value((
                hp,
                world.read_storage::<Unit>().get(unit.entity),
                world.read_storage::<Inventory>().get(unit.entity),
                world.read_storage::<Gold>().get(unit.entity),
                world.read_storage::<ItemEffects>().get(unit.entity),
                world.read_storage::<Facing>().get(unit.entity),
                world.read_storage::<MoveTarget>().get(unit.entity),
            ))
            .expect("lane components JSON"),
        );
        hash.update(format!(
            "|{key}|{}|{:?}|{}|{}|{}|{}",
            unit.team,
            unit.role,
            unit.attack_remaining.raw(),
            pos.0.x.raw(),
            pos.0.y.raw(),
            hp.hp.raw()
        ));
        if let Some(hero) = heroes.get(unit.entity) {
            hash.update(format!("|{}|{}", hero.level, hero.experience));
            let cooldowns: BTreeMap<_, _> = hero.ability_cooldowns.iter().collect();
            for (id, value) in cooldowns {
                hash.update(format!("|{id}|{}", value.raw()));
            }
            hash_json(&mut hash, &serde_json::to_value(hero).expect("hero JSON"));
        }
        if let Some(attack) = attacks.get(unit.entity) {
            hash.update(format!(
                "|{}|{}|{:?}",
                attack.asd_count.raw(),
                attack.attack_seq,
                attack.attack_phase
            ));
        }
        if let Some(command) = commands.get(unit.entity) {
            hash.update(format!("|{:?}", command));
        }
    }
    // In-flight projectiles and queued deaths are future gameplay state too.
    // Excluding them could hide a divergence until the projectile hits.
    use specs::Join;
    let projectiles = world.read_storage::<Projectile>();
    for (entity, projectile, pos) in (&world.entities(), &projectiles, &positions).join() {
        hash.update(entity_key(entity).to_le_bytes());
        hash_json(
            &mut hash,
            &serde_json::to_value((projectile, pos)).expect("projectile JSON"),
        );
    }
    hash_json(
        &mut hash,
        &serde_json::to_value(&*world.read_resource::<Vec<Outcome>>()).expect("outcome JSON"),
    );
    format!("{:x}", hash.finalize())
}

fn hash_json(hash: &mut sha2::Sha256, value: &serde_json::Value) {
    use sha2::Digest;
    match value {
        serde_json::Value::Object(map) => {
            hash.update(b"{");
            let ordered: BTreeMap<_, _> = map.iter().collect();
            for (key, value) in ordered {
                hash.update(serde_json::to_vec(key).expect("JSON key"));
                hash.update(b":");
                hash_json(hash, value);
                hash.update(b",");
            }
            hash.update(b"}");
        }
        serde_json::Value::Array(array) => {
            hash.update(b"[");
            for value in array {
                hash_json(hash, value);
                hash.update(b",");
            }
            hash.update(b"]");
        }
        value => hash.update(serde_json::to_vec(value).expect("JSON value")),
    }
}

#[cfg(feature = "kcp")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SingleLaneBotPolicy {
    Push,
    Guard,
}

/// Bots submit the exact PlayerInput schema used by network players. They do
/// not mutate HP, position, command queues, or invoke script hooks themselves.
#[cfg(feature = "kcp")]
pub fn single_lane_bot_inputs(
    world: &World,
    policies: [SingleLaneBotPolicy; 2],
) -> Vec<(u32, crate::runtime::PlayerInput)> {
    use crate::runtime::{
        AttackMove, AttackTarget, CastAbility, PlayerInput, PlayerInputEnum, Vec2I,
    };
    let state = world.read_resource::<MobaMatch>();
    if state.phase != MobaMatchPhase::Playing {
        return Vec::new();
    }
    let positions = world.read_storage::<Pos>();
    let properties = world.read_storage::<CProperty>();
    let heroes = world.read_storage::<Hero>();
    let mut inputs = Vec::new();
    let commands = world.read_storage::<HeroCommandQueue>();
    for slot in &state.heroes {
        let team = slot.side;
        let policy = policies[team];
        let Some(entity) = slot.entity else {
            continue;
        };
        let Some(hero) = heroes.get(entity) else {
            continue;
        };
        let source = positions.get(entity).expect("bot position").0;
        let hp = properties.get(entity).expect("bot hp");
        let enemy_team = 1 - team;
        let objective = state.towers[enemy_team].or(state.bases[enemy_team]);
        let target = state
            .units
            .values()
            .filter(|u| u.team as usize != team)
            .filter(|u| u.role != LaneRole::Base || state.base_unlocked(u.team as usize))
            .filter(|u| {
                properties
                    .get(u.entity)
                    .is_some_and(|p| p.hp > Fixed64::ZERO)
            })
            .filter_map(|u| {
                positions
                    .get(u.entity)
                    .map(|p| (u, (p.0 - source).length_squared()))
            })
            .filter(|(_, d)| *d <= Fixed64::from_i32(550) * Fixed64::from_i32(550))
            .min_by_key(|(u, d)| (Some(u.entity) != objective, d.raw(), entity_key(u.entity)));
        let ready = |index: usize| {
            hero.abilities
                .get(index)
                .is_some_and(|id| hero.can_use_ability(id) && !hero.is_on_cooldown(id))
        };
        // The training hero is declarative. This policy's slot choice is data
        // convention for this fixture, not a universal hero-specific engine.
        let heal = hp.hp < hp.mhp * Fixed64::from_raw(614);
        let cast = if heal && ready(3) {
            Some((3, None))
        } else if heal && ready(1) {
            Some((1, None))
        } else if policy == SingleLaneBotPolicy::Push && target.is_some() && ready(2) {
            Some((2, target.map(|(u, _)| u.entity.id())))
        } else if policy == SingleLaneBotPolicy::Push && target.is_some() && ready(0) {
            Some((0, target.map(|(u, _)| u.entity.id())))
        } else {
            None
        };
        let action = if let Some((ability_index, target_entity)) = cast {
            PlayerInputEnum::CastAbility(CastAbility {
                ability_index,
                target_entity,
                target_pos: None,
            })
        } else if policy == SingleLaneBotPolicy::Push && state.base_unlocked(enemy_team) {
            let Some(base) = state.bases[enemy_team] else {
                continue;
            };
            if commands.get(entity).is_some_and(|queue| {
                matches!(queue.active,
                Some(HeroCommand::AttackTarget { target, .. }) if target == base)
            }) {
                continue;
            }
            PlayerInputEnum::AttackTarget(AttackTarget {
                target_id: base.id(),
                queued: false,
            })
        } else {
            let destination = if policy == SingleLaneBotPolicy::Guard {
                if state.config.map_id.is_some() {
                    state.towers[team].and_then(|tower| positions.get(tower)).map_or(
                        position(&state.config,team,Fixed64::ZERO), |p| p.0)
                } else { position(&state.config, team, Fixed64::from_i32(700)) }
            } else if state.config.map_id.is_some() {
                objective.and_then(|entity| positions.get(entity)).map_or(
                    position(&state.config,1 - team,Fixed64::ZERO), |p| p.0)
            } else {
                position(&state.config, 1 - team, Fixed64::ZERO)
            };
            if (destination - source).length_squared() <= Fixed64::ONE
                || commands.get(entity).is_some_and(|queue| {
                    matches!(queue.active,
                    Some(HeroCommand::AttackMove { pos }) if pos == destination)
                })
            {
                continue;
            }
            PlayerInputEnum::AttackMove(AttackMove {
                target: Some(Vec2I {
                    x: destination.x.raw() as i32,
                    y: destination.y.raw() as i32,
                }),
                queued: false,
            })
        };
        inputs.push((
            slot.player_id,
            PlayerInput {
                action: Some(action),
            },
        ));
    }
    inputs
}
