//! Opt-in single-lane MOBA rules over the production ECS and outcome pipeline.
//! No renderer, wall clock, network, or script-DLL dependency is needed here.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use failure::{err_msg, Error};
use omoba_sim::{Fixed64, Vec2};
use specs::{Builder, Entity, World, WorldExt};

use crate::comp::*;
mod jungle;
mod jungle_credit;
#[cfg(test)]
mod projectile_credit_tests;
mod base_structures;
mod tower_focus;
mod building_rewards;
mod tower_plating;
pub use tower_plating::TowerPlatingRules;
mod first_tower;
mod first_kill;
mod objective_notice;
pub use objective_notice::{OBJECTIVE_TEAM_METRIC_ID,OBJECTIVE_KIND_METRIC_ID,OBJECTIVE_SEQUENCE_METRIC_ID,OBJECTIVE_REMAINING_METRIC_ID};
pub use first_kill::{FIRST_KILL_KILLER_METRIC_ID,FIRST_KILL_VICTIM_METRIC_ID,FIRST_KILL_GOLD_METRIC_ID};
mod kill_bounty;
mod objective_comeback;
pub use omoba_content_model::objective_comeback::ObjectiveComebackRules;
mod death_streak;
pub use death_streak::DeathStreakRules;
pub use kill_bounty::KillBountyRules;
mod minion_growth;
mod objective_buffs;
mod minion_aura;
mod personal_buffs;
pub use minion_growth::MinionGrowthRules;
pub use building_rewards::{BuildingBounty, BuildingRewardRules};
pub(crate) use building_rewards::record_moba_building_damage;
pub use tower_focus::TowerFocusRules;
#[cfg(test)]
mod brush_attack_tests;
#[cfg(feature = "kcp")]
pub mod bots;
#[cfg(feature = "kcp")]
pub mod selection;
pub use jungle::MobaJungleCamp;
pub(crate) use jungle::record_moba_jungle_damage;

/// Recall is a stationary channel: immobilization and silence both interrupt it.
/// Owner-side AI and authority settlement must use the same control policy.
pub(super) fn recall_control_blocked(buffs:&crate::runtime::ability_runtime::BuffStore,entity:Entity)->bool {
    buffs.is_rooted(entity) || buffs.is_silenced(entity)
}

/// Global public HUD metric, not an entity reference or private match resource.
pub const SINGLE_LANE_DELTA_METRIC_ID: u64 = 0x4d4f424144543031;
pub const SINGLE_LANE_ELAPSED_METRIC_ID: u64 = 0x4d4f4241544d3031;
pub const SINGLE_LANE_PHASE_METRIC_ID: u64 = 0x4d4f424150483031;
pub const SINGLE_LANE_WINNER_METRIC_ID: u64 = 0x4d4f4241574e3031;
pub const SINGLE_LANE_RESPAWN_METRIC_ID: u64 = 0x4d52535000000000;
pub const SINGLE_LANE_LENGTH_METRIC_ID: u64 = 0x4d4f42414c4e3031;
pub const SINGLE_LANE_RECALL_ACTIVE_METRIC_ID: u64 = 0x4d52414300000000;
pub const SINGLE_LANE_RECALL_REMAINING_METRIC_ID: u64 = 0x4d524d4e00000000;
pub const SINGLE_LANE_SUMMONER_SLOT0_SPELL: u64 = 0x4d53533000000000;
pub const SINGLE_LANE_SUMMONER_SLOT1_SPELL: u64 = 0x4d53533100000000;
pub const SINGLE_LANE_SUMMONER_SLOT0_COOLDOWN: u64 = 0x4d53534300000000;
pub const SINGLE_LANE_SUMMONER_SLOT1_COOLDOWN: u64 = 0x4d53534400000000;
pub const SINGLE_LANE_KILLS_METRIC_ID: u64 = 0x4d4b494c00000000;
pub const SINGLE_LANE_DEATHS_METRIC_ID: u64 = 0x4d44454100000000;
pub const SINGLE_LANE_ASSISTS_METRIC_ID: u64 = 0x4d41535300000000;
pub const MANA_COST_TICK: u64 = 0x4d4d435400000000;
pub const MANA_COST_RAW: [u64;4] = [0x4d4d433000000000,0x4d4d433100000000,0x4d4d433200000000,0x4d4d433300000000];
pub const MANA_COST_BINDING: [u64;4] = [0x4d4d523000000000,0x4d4d523100000000,0x4d4d523200000000,0x4d4d523300000000];

fn emit_owner_mana_costs(world:&World,state:&MobaMatch,phase:crate::runtime::FactPhase) {
    let heroes=world.read_storage::<Hero>();let owners=world.read_storage::<PlayerOwner>();
    let properties=world.read_storage::<CProperty>();
    let registry=world.try_fetch::<crate::runtime::ability_runtime::AbilityRegistry>();
    let buffs=world.try_fetch::<crate::runtime::ability_runtime::BuffStore>();
    let tick=world.read_resource::<Tick>().0;
    for (index,slot) in state.heroes.iter().enumerate() {
        let team=state.config.teams[slot.side];
        let emit=|offset,namespace,value| {
            world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(crate::runtime::OrderedFact {
                key:crate::runtime::FactOrderingKey {tick,phase,canonical_source_order:0,local_ordinal:10000+index as u32*9+offset,fact_kind:crate::runtime::FactKind::Hud},
                audience:crate::runtime::FactAudience::Team(team),
                fact:crate::runtime::ObservableFact::Hud {team,metric_id:single_lane_player_metric(namespace,slot.player_id),value},
            }).expect("valid team-scoped current ability cost");
        };
        emit(0,MANA_COST_TICK,i64::try_from(tick).unwrap_or(-1));
        let source=slot.entity.filter(|e|state.phase==MobaMatchPhase::Playing && world.entities().is_alive(*e)
            && owners.get(*e).is_some_and(|o|o.player_id==slot.player_id)
            && properties.get(*e).is_some_and(|p|p.hp>Fixed64::ZERO))
            .and_then(|e|heroes.get(e).filter(|h|h.mana_pool.is_some()).map(|h|(e,h)));
        for ability_slot in 0..4 {
            let binding=source.and_then(|(_,h)|h.abilities.get(ability_slot).map(|id|(h,id)))
                .and_then(|(h,id)|Some((omoba_template_ids::ability_by_name(id)?,h.get_ability_level(id))))
                .filter(|(_,rank)|(1..=255).contains(rank)).map_or(0,|(id,rank)|(i64::from(id.raw())<<8)|i64::from(rank));
            let cost=source.and_then(|(e,h)| {
                let id=h.abilities.get(ability_slot)?;let rank=u8::try_from(h.get_ability_level(id)).ok()?;
                let definition=registry.as_ref()?.get(id)?.get_level_data(rank)?;
                let stats=crate::runtime::ability_runtime::UnitStats::from_refs(buffs.as_ref()?,false);
                Some(stats.checked_mana_cost_mult(e).and_then(|m|
                    crate::runtime::ability_runtime::checked_mana_cost(definition.mana_cost,m).ok()).map_or(-2,|v|v.raw()))
            }).unwrap_or(-1);
            emit(1+ability_slot as u32*2,MANA_COST_BINDING[ability_slot],binding);
            emit(2+ability_slot as u32*2,MANA_COST_RAW[ability_slot],cost);
        }
    }
}
// Explicit public-score namespaces; never reuse private owner HUD metrics.
pub const SINGLE_LANE_SCORE_BOUNTY_METRIC_ID: u64 = 0x4d53425900000000;
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
            (SINGLE_LANE_SCORE_BOUNTY_METRIC_ID, state.shutdown_bounty(slot.player_id)),
        ].into_iter().enumerate() {
            emit_public(101 + index as u32 * 5 + ordinal as u32,
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
    first_kill::emit(world,state,phase);
    objective_notice::emit(world,state,phase);
    emit_owner_mana_costs(world,state,phase);
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
    let availability: Vec<_> = state
        .heroes
        .iter()
        .map(|slot| super::shop::owner_shop_availability(world, state, slot.player_id))
        .collect();
    let inventories = world.read_storage::<Inventory>();
    let balances = world.read_storage::<Gold>();
    let effects = world.read_storage::<ItemEffects>();
    let properties = world.read_storage::<CProperty>();
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
        let (shop_available, shop_reason) = availability[side];
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
                        shop_reason,
                        shield_remaining_raw: slot.entity
                            .filter(|entity| properties.get(*entity).is_some_and(|p| p.hp > Fixed64::ZERO))
                            .and_then(|entity| world.try_fetch::<crate::runtime::native::ability_runtime::buff_store::BuffStore>()
                                .map(|store| store.shield_remaining(entity).raw()))
                            .unwrap_or(0),
                    },
                },
            })
            .expect("valid persistent owner economy HUD");
        let projection = crate::runtime::shop_undo_wire::owner_shop_undo_projection(
            world,
            state,
            slot.player_id,
            crate::runtime::shop_undo_wire::phase_byte(phase),
        );
        world
            .read_resource::<crate::runtime::ObservableFactBuffer>()
            .emit(crate::runtime::OrderedFact {
                key: crate::runtime::FactOrderingKey {
                    tick: world.read_resource::<Tick>().0,
                    phase,
                    canonical_source_order: 0,
                    local_ordinal: side as u32 * 2
                        + u32::from(phase == crate::runtime::FactPhase::PostStep),
                    fact_kind: crate::runtime::FactKind::OwnerShopUndo,
                },
                audience: crate::runtime::FactAudience::Team(team),
                fact: crate::runtime::ObservableFact::OwnerShopUndo { team, state: projection },
            })
            .expect("valid owner undo availability");
    }
}

/// Server lockstep domain for moba worlds. Absent `MobaMatch` is not this
/// function's input; the caller keeps the legacy hash in that case. Gold,
/// catalog ids, cooldown bits, source identity, and closed records are included.
/// There is no save schema here. A render snapshot cannot rebuild this ledger.
pub fn lockstep_economy_v2_material(world: &World) -> Vec<u8> {
    let Some(state) = world.try_fetch::<MobaMatch>() else { return Vec::new() };
    let mut heroes: Vec<_> = state.heroes.iter().collect();
    heroes.sort_by_key(|slot| slot.player_id);
    let mut bytes = Vec::new();
    bytes.extend((heroes.len() as u32).to_le_bytes());
    for slot in heroes {
        let (gold, catalogs, cooldowns) = super::shop::projected_owner_wallet_in(world, &state, slot.player_id);
        bytes.extend(slot.player_id.to_le_bytes());
        bytes.extend(gold.to_le_bytes());
        for value in catalogs { bytes.extend(value.to_le_bytes()); }
        for value in cooldowns { bytes.extend(value.to_le_bytes()); }
        let projection = crate::runtime::shop_undo_wire::owner_shop_undo_projection(
            world, &state, slot.player_id, 0,
        );
        let encoded = projection.encode();
        bytes.extend((encoded.len() as u32).to_le_bytes());
        bytes.extend(encoded);
        let ledger = super::shop::owner_ledger_preimage(world, slot.player_id);
        bytes.extend((ledger.len() as u32).to_le_bytes());
        bytes.extend(ledger);
    }
    bytes
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
    pub objective_comeback_rules: ObjectiveComebackRules,
    pub tower_plating_rules: Option<TowerPlatingRules>,
    pub kill_bounty_rules: KillBountyRules,
    pub death_streak_rules: DeathStreakRules,
    pub minion_growth_rules: MinionGrowthRules,
    pub building_reward_rules: BuildingRewardRules,
    /// Shared lane/nexus tower pressure; zero max bonus disables the ramp.
    pub tower_focus_rules: TowerFocusRules,
    /// Authority input-mode lock and bounded held-direction lease; never renderer clocks.
    pub input_scheme_rules: crate::runtime::input_scheme::InputSchemeRules,
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
    /// Formal map plans use the shared authored composition. Isolated legacy
    /// fixtures may still request a simple fixed-size wave explicitly.
    pub authored_minion_waves: bool,
    pub lane_length: Fixed64,
    pub tower_hp: Fixed64,
    pub base_hp: Fixed64,
    pub passive_gold_per_second: u32,
    pub hero_kill_gold: u32,
    pub first_kill_bonus_gold: u32,
    pub hero_assist_gold: u32,
    pub hero_kill_xp: u32,
    pub hero_assist_xp: u32,
    pub lane_creep_xp: u32,
    /// Last hit only; nearby XP recipients do not share this reward.
    pub lane_creep_gold: u32,
    pub lane_xp_radius: u32,
    pub assist_window_seconds: u32,
    /// Explicit opt-in; legacy TD/straight-lane fixtures remain unchanged.
    pub base_recovery_enabled: bool,
    /// Explicit opt-in; never inferred from legacy GameMode::Moba stories.
    pub mana_enabled: bool,
    /// Explicit authority loadouts; absent players have no summoner spells.
    /// Selection is validated before any entities are spawned.
    pub summoner_loadouts: BTreeMap<u32, [Option<String>; 2]>,
    /// Slot 0 catalog ids. Empty leaves every hero unequipped. `None` is an
    /// explicit clear. The authored training file is applied by the server,
    /// not by this default.
    pub trinket_loadouts: BTreeMap<u32, Option<u16>>,
}

impl Default for SingleLaneConfig {
    fn default() -> Self {
        Self {
            tower_focus_rules: TowerFocusRules::default(),
            objective_comeback_rules: ObjectiveComebackRules::default(),
            tower_plating_rules: None,
            building_reward_rules: BuildingRewardRules::default(),
            kill_bounty_rules: KillBountyRules::default(),
            death_streak_rules: DeathStreakRules::default(),
            minion_growth_rules: MinionGrowthRules::default(),
            input_scheme_rules: crate::runtime::input_scheme::InputSchemeRules::default(),
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
            authored_minion_waves: false,
            lane_length: Fixed64::from_i32(2400),
            tower_hp: Fixed64::from_i32(1200),
            base_hp: Fixed64::from_i32(1600),
            passive_gold_per_second: omoba_template_ids::MOBA_PASSIVE_GOLD_PER_SECOND,
            hero_kill_gold: omoba_template_ids::MOBA_HERO_KILL_GOLD,
            first_kill_bonus_gold: 0,
            hero_assist_gold: omoba_template_ids::MOBA_HERO_ASSIST_GOLD,
            hero_kill_xp: omoba_template_ids::MOBA_HERO_KILL_XP,
            hero_assist_xp: omoba_template_ids::MOBA_HERO_ASSIST_XP,
            lane_creep_xp: omoba_template_ids::MOBA_LANE_CREEP_XP,
            lane_creep_gold: omoba_template_ids::MOBA_LANE_CREEP_GOLD,
            lane_xp_radius: omoba_template_ids::MOBA_LANE_XP_RADIUS,
            assist_window_seconds: omoba_template_ids::MOBA_ASSIST_WINDOW_SECONDS,
            base_recovery_enabled: false,
            mana_enabled: false,
            summoner_loadouts: BTreeMap::new(),
            trinket_loadouts: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaneRole {
    Hero,
    Creep,
    Tower,
    Inhibitor,
    NexusTower,
    Base,
}

impl LaneRole {
    fn is_tower(self) -> bool { matches!(self, Self::Tower | Self::NexusTower) }
}

#[derive(Clone, Debug)]
struct LaneUnit {
    entity: Entity,
    team: u8,
    role: LaneRole,
    attack_remaining: Fixed64,
    lane: usize,
    next_waypoint: usize,
    last_hit_retired: bool,
    target: Option<Entity>,
    target_protected: bool,
    attack_sequence: u32,
    tower_focus: tower_focus::TowerFocusState,
}

#[derive(Clone, Debug)]
struct ChampionAggression {
    attacker: Entity,
    victim: Entity,
    expires: Fixed64,
    minion_help: bool,
    minion_expires: Fixed64,
    defender_team: u8,
    victim_position: Vec2,
}

#[derive(Clone, Debug)]
struct BrushAttackReveal {
    attacker: Entity,
    expires: Fixed64,
}

#[derive(Clone, Debug)]
pub struct MobaHeroSlot {
    pub player_id: u32,
    /// Persistent authority policy. Hero replacement never resets this roster value.
    combat_preferences: crate::runtime::combat_controls::OwnerCombatPreferences,
    input_scheme: crate::runtime::input_scheme::OwnerInputScheme,
    /// Lane side (0/1), independent of this slot's index in the player roster.
    pub side: usize,
    pub entity: Option<Entity>,
    pub deaths: u32,
    pub kills: u32,
    /// Consecutive credited hero kills since this player last died.
    pub kill_streak: u32,
    /// Credited enemy deaths since the last credited kill; persists across lives.
    pub unanswered_deaths: u32,
    pub assists: u32,
    lethal_pending: bool,
    recall: Option<(Vec2, Fixed64)>,
    pub respawns: u32,
    pub respawn_at: Option<Fixed64>,
    hero: Hero,
    inventory: Inventory,
    gold: Gold,
    pub(crate) summoners: crate::runtime::summoner_runtime::SummonerLoadout,
    /// Slot 0 trinket. Independent of inventory, abilities, and summoners.
    /// Omission stays unequipped until an authoritative loadout is installed.
    pub(crate) trinket: super::ward_world::WardRosterState,
}

impl MobaHeroSlot {
    pub fn input_scheme(&self) -> crate::runtime::input_scheme::OwnerInputScheme { self.input_scheme }
    /// Stable roster content identity, including while no hero life is present.
    pub fn hero_content_id(&self) -> &str { &self.hero.id }
    pub fn combat_preferences(&self) -> crate::runtime::combat_controls::OwnerCombatPreferences {
        self.combat_preferences
    }
    /// Roster wallet after [`record_moba_death`] and before the replacement spawn.
    /// A cleared entity without an admitted respawn is not this wallet.
    pub(crate) fn admitted_dead_wallet(&mut self) -> Option<(&mut Inventory, &mut Gold)> {
        if self.entity.is_none() && self.respawn_at.is_some() && self.deaths > 0 {
            Some((&mut self.inventory, &mut self.gold))
        } else {
            None
        }
    }

    pub(crate) fn admitted_dead_wallet_ref(&self) -> Option<(&Inventory, Gold)> {
        if self.entity.is_none() && self.respawn_at.is_some() && self.deaths > 0 {
            Some((&self.inventory, self.gold))
        } else {
            None
        }
    }

    /// Lethal damage has been recorded and the replacement spawn has not happened.
    pub(crate) fn death_is_pending(&self) -> bool {
        self.lethal_pending
    }

    /// Test-only poison. Communication HUD facts must ignore this cache.
    #[cfg(test)]
    pub(crate) fn poison_cached_progression_for_test(&mut self) {
        self.hero.abilities.clear();
        self.hero.abilities.push("stale_cached_ability".to_string());
        self.hero.start_cooldown("stale_cached_ability", Fixed64::from_i32(99));
        self.inventory.slots[0] = Some(ItemInstance {
            item_id: omoba_template_ids::MOBA_ITEM_CATALOG[0].id.to_string(),
            cooldown_remaining: 50.0,
        });
    }

    /// Installs the same wallet predicate `record_moba_death` leaves behind.
    #[cfg(test)]
    pub(crate) fn install_admitted_dead_wallet_for_test(&mut self, inventory: Inventory) {
        self.entity = None;
        self.deaths = self.deaths.max(1);
        self.respawn_at = Some(Fixed64::from_i32(1));
        self.inventory = inventory;
    }
}

/// Combat policy copied from the catalog row at placement. A missing key is an
/// ordinary body, not a ward.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WardCombatPolicy {
    pub direct_attack: omoba_template_ids::TrinketDirectAttackConst,
    pub automatic_acquisition: omoba_template_ids::TrinketAutomaticAcquisitionConst,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatPreferenceAdmissionError { WrongOwner, InvalidPhase, InvalidAcquisition }

#[cfg(test)]
#[path = "moba_match/combat_preference_tests.rs"]
mod combat_preference_tests;
#[cfg(test)]
#[path = "moba_match/input_scheme_tests.rs"]
mod input_scheme_tests;

#[derive(Clone, Debug)]
pub struct MobaMatch {
    base_structures: Vec<base_structures::RespawningStructure>,
    pub config: SingleLaneConfig,
    pub phase: MobaMatchPhase,
    pub elapsed: Fixed64,
    pub(crate) input_step: Option<crate::runtime::input_scheme::InputSchemeStepSnapshot>,
    input_reset_sources: BTreeSet<Entity>,
    input_attack_reset_sources: BTreeSet<Entity>,
    earned_gold: [u64;2],
    comeback: objective_comeback::ComebackState,
    income_remainder_raw: i64,
    pending_income_delta_raw: i64,
    pub waves: u32,
    pub heroes: Vec<MobaHeroSlot>,
    pub towers: [Option<Entity>; 2],
    /// One tower per side per lane. `towers` remains the first surviving tower
    /// for legacy HUD/bot callers, never the base-unlock authority.
    pub lane_towers: Vec<[Option<Entity>; 2]>,
    /// Per-lane outer-to-inner authoritative retirement state.
    pub lane_tower_layers: Vec<Vec<[Option<Entity>; 2]>>,
    routes: Vec<[Vec<Vec2>; 2]>,
    pub bases: [Option<Entity>; 2],
    pub jungle_camps: Vec<MobaJungleCamp>,
    assist_ledger: super::moba_assist::AssistLedger,
    first_tower:first_tower::FirstTowerState,
    first_kill:first_kill::FirstKillState,
    objective_notices:objective_notice::ObjectiveNotices,
    personal_buffs: BTreeMap<(u32,&'static str),personal_buffs::PersonalBuffLease>,
    objective_buffs: BTreeMap<(usize,&'static str),objective_buffs::TeamObjectiveLease>,
    tower_plates:BTreeMap<u64,tower_plating::PlateLedger>,
    minion_aura_targets: BTreeMap<(Entity,&'static str),(i32,i32)>,
    champion_aggression: BTreeMap<(u64, u64), ChampionAggression>,
    brush_attack_reveals: BTreeMap<(u64, u32), BrushAttackReveal>,
    next_wave_at: Fixed64,
    units: BTreeMap<u64, LaneUnit>,
    pub(crate) summoner_catalog: Option<crate::runtime::summoner_runtime::SummonerCatalog>,
    /// Canonical placed-ward id to the policy copied at spawn.
    pub(crate) ward_policies: BTreeMap<u64, WardCombatPolicy>,
}

impl MobaMatch {
    fn protects_champion(&self, unit: &LaneUnit, source_pos: Vec2, other: &LaneUnit, sieging: bool) -> bool {
        if sieging {return false;}
        let attacker=entity_key(other.entity);
        self.champion_aggression.range((attacker,0)..=(attacker,u64::MAX)).any(|(_,a)|
            (unit.role.is_tower() || (a.minion_help && a.minion_expires>self.elapsed)) && a.defender_team == unit.team
                && (a.victim_position-source_pos).length_squared() <= Fixed64::from_i32(500)*Fixed64::from_i32(500))
    }
    fn input_scheme_owner_index(&self, authenticated_owner: u32, requested_owner: u32)
        -> Result<usize, crate::runtime::input_scheme::InputSchemeError> {
        use crate::runtime::input_scheme::InputSchemeError;
        if self.elapsed<Fixed64::ZERO { return Err(InputSchemeError::InvalidClock); }
        if authenticated_owner==0 || authenticated_owner!=requested_owner { return Err(InputSchemeError::WrongOwner); }
        if matches!(self.phase,MobaMatchPhase::Finished{..}) { return Err(InputSchemeError::InvalidPhase); }
        let mut matches=self.heroes.iter().enumerate().filter(|(_,slot)| slot.player_id==authenticated_owner);
        let index=matches.next().map(|(index,_)|index).ok_or(InputSchemeError::WrongOwner)?;
        if matches.next().is_some() { return Err(InputSchemeError::WrongOwner); }
        Ok(index)
    }
    pub fn input_scheme_elapsed(&self) -> Fixed64 { if self.elapsed<self.config.warmup {Fixed64::ZERO} else {self.elapsed-self.config.warmup} }
    pub fn admit_input_scheme(&mut self, authenticated_owner: u32, requested_owner: u32, raw: u32)
        -> Result<(), crate::runtime::input_scheme::InputSchemeError> {
        let index=self.input_scheme_owner_index(authenticated_owner,requested_owner)?;
        let elapsed=self.input_scheme_elapsed();
        self.heroes[index].input_scheme.select(raw,elapsed,self.config.input_scheme_rules)
    }
    pub fn admit_direction(&mut self, authenticated_owner: u32, source: Entity,
        gesture: crate::runtime::input_scheme::DirectionGesture, x: i32, y: i32)
        -> Result<crate::runtime::input_scheme::DirectionAdmission, crate::runtime::input_scheme::InputSchemeError> {
        use crate::runtime::input_scheme::InputSchemeError;
        let index=self.input_scheme_owner_index(authenticated_owner,authenticated_owner)?;
        if self.phase!=MobaMatchPhase::Playing { return Err(InputSchemeError::InvalidPhase); }
        let slot=&self.heroes[index];
        if slot.entity!=Some(source) || slot.lethal_pending || source.gen().id()<=0 { return Err(InputSchemeError::InvalidSource); }
        let key=crate::runtime::selective::CanonicalEntityKey{id:source.id(),generation:source.gen().id() as u32};
        let elapsed=self.input_scheme_elapsed();
        self.heroes[index].input_scheme.admit_direction(key,gesture,x,y,elapsed,self.config.input_scheme_rules)
    }
    pub fn neutralize_direction(&mut self, owner: u32, source: Entity) {
        if let Ok(index)=self.input_scheme_owner_index(owner,owner) {
            if self.heroes[index].entity==Some(source) {self.heroes[index].input_scheme.neutralize();}
        }
    }
    pub fn direction_for_source(&self, owner: u32, source: Entity) -> Vec2 {
        let Ok(index)=self.input_scheme_owner_index(owner,owner) else { return Vec2::ZERO; };
        let slot=&self.heroes[index];
        if self.phase!=MobaMatchPhase::Playing || slot.entity!=Some(source) || slot.lethal_pending || source.gen().id()<=0 { return Vec2::ZERO; }
        slot.input_scheme.direction(crate::runtime::selective::CanonicalEntityKey{id:source.id(),generation:source.gen().id() as u32},self.input_scheme_elapsed())
    }
    /// Trusted owner admission seam; wire authentication must supply `authenticated_owner`.
    /// This does not cancel commands, attacks, recalls or cooldowns.
    pub fn admit_combat_preferences(&mut self, authenticated_owner: u32, requested_owner: u32,
        acquisition_raw: u8, idle_auto_attack: bool) -> Result<(), CombatPreferenceAdmissionError> {
        use crate::runtime::combat_controls::{CombatPreferenceUpdate, OwnerCombatPreferences};
        if authenticated_owner == 0 || authenticated_owner != requested_owner {
            return Err(CombatPreferenceAdmissionError::WrongOwner);
        }
        if self.phase != MobaMatchPhase::Playing { return Err(CombatPreferenceAdmissionError::InvalidPhase); }
        let mut matches = self.heroes.iter().enumerate().filter(|(_, slot)| slot.player_id == authenticated_owner);
        let index = matches.next().map(|(index, _)| index).ok_or(CombatPreferenceAdmissionError::WrongOwner)?;
        if matches.next().is_some() || self.heroes[index].combat_preferences.owner_id() != authenticated_owner {
            return Err(CombatPreferenceAdmissionError::WrongOwner);
        }
        let next = OwnerCombatPreferences::try_from_raw(CombatPreferenceUpdate {
            owner_id: authenticated_owner, acquisition_raw, idle_auto_attack, champion_only_direct: false,
        }).map_err(|_| CombatPreferenceAdmissionError::InvalidAcquisition)?;
        self.heroes[index].combat_preferences = next;
        Ok(())
    }

    pub(crate) fn combat_preferences_for_source(&self, owner: u32, source: Entity)
        -> Option<crate::runtime::combat_controls::OwnerCombatPreferences> {
        let mut slots = self.heroes.iter().filter(|slot| slot.player_id == owner);
        let slot = slots.next()?;
        if owner == 0 || slots.next().is_some() || slot.entity != Some(source)
            || slot.combat_preferences.owner_id() != owner { return None; }
        Some(slot.combat_preferences)
    }
    /// Aggregate admission only, not prerequisite identities or hidden HP.
    pub(crate) fn disclosed_structure_states(&self)->BTreeMap<u64,[u8;2]> {
        self.units.values().filter_map(|unit| {
            let role=match unit.role {LaneRole::Tower|LaneRole::NexusTower|LaneRole::Inhibitor=>1,LaneRole::Base=>2,_=>return None};
            Some((crate::runtime::canonical_entity_id(unit.entity),
                [role,u8::from(self.phase==MobaMatchPhase::Playing && self.structure_unlocked(unit))]))
        }).collect()
    }
    pub fn base_unlocked(&self, side: usize) -> bool {
        if side>=2 {return false;}
        if !self.base_structures.is_empty() {
            return self.inhibitor_down(side, None) && self.base_structures.iter()
                .filter(|node|node.team==side && node.role==LaneRole::NexusTower)
                .all(|node|node.entity.is_none());
        }
        let cleared=|lane:&Vec<[Option<Entity>;2]>| !lane.is_empty() && lane.iter().all(|layer|layer[side].is_none());
        match self.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name)
            .map(|map|map.base_unlock).unwrap_or(omoba_template_ids::MobaBaseUnlockConst::AllLaneTowers) {
            omoba_template_ids::MobaBaseUnlockConst::AllLaneTowers=>self.lane_tower_layers.iter().all(cleared),
            omoba_template_ids::MobaBaseUnlockConst::AnyLaneTowers=>self.lane_tower_layers.iter().any(cleared),
        }
    }

    fn structure_unlocked(&self, unit: &LaneUnit) -> bool {
        match unit.role {
            LaneRole::Base => self.base_unlocked(unit.team as usize),
            LaneRole::Inhibitor => self.lane_tower_layers[unit.lane].iter().all(|layer|layer[unit.team as usize].is_none()),
            LaneRole::NexusTower => self.inhibitor_down(unit.team as usize, None),
            LaneRole::Tower => self.lane_towers.get(unit.lane)
                .is_some_and(|lane| lane[unit.team as usize] == Some(unit.entity)),
            _ => true,
        }
    }

    /// Current-match target policy shared by publication, acquisition, and damage.
    /// Alive, hostility, range, disclosure, and owner preference stay with the caller.
    /// A non-playing phase closes every body, including ordinary units.
    #[cfg(test)]
    pub(crate) fn note_ward_policy(
        &mut self,
        entity: Entity,
        direct_attack: omoba_template_ids::TrinketDirectAttackConst,
        automatic_acquisition: omoba_template_ids::TrinketAutomaticAcquisitionConst,
    ) {
        self.ward_policies.insert(
            crate::runtime::canonical_entity_id(entity),
            WardCombatPolicy {
                direct_attack,
                automatic_acquisition,
            },
        );
    }

    /// Missing policy means an ordinary body. Direct attack and automatic
    /// acquisition are separate authored tokens.
    pub(crate) fn ward_direct_attack_allowed(&self, entity: Entity) -> bool {
        match self.ward_policies.get(&crate::runtime::canonical_entity_id(entity)) {
            None => true,
            Some(policy) => match policy.direct_attack {
                omoba_template_ids::TrinketDirectAttackConst::DisclosedEnemy => true,
                omoba_template_ids::TrinketDirectAttackConst::NotDirectlyAttackable => false,
            },
        }
    }

    pub(crate) fn ward_automatic_acquisition_allowed(&self, entity: Entity) -> bool {
        match self.ward_policies.get(&crate::runtime::canonical_entity_id(entity)) {
            None => true,
            Some(policy) => match policy.automatic_acquisition {
                omoba_template_ids::TrinketAutomaticAcquisitionConst::Enabled => true,
                omoba_template_ids::TrinketAutomaticAcquisitionConst::Disabled => false,
            },
        }
    }

    pub fn current_match_target_attackable(&self, target: Entity) -> bool {
        self.phase == MobaMatchPhase::Playing
            && !self.jungle_camps.iter().any(|camp| camp.contains(target) && camp.invulnerable())
            && self.units.get(&entity_key(target)).is_none_or(|unit| self.structure_unlocked(unit))
    }
    pub fn is_recalling(&self, entity: Entity) -> bool {
        self.heroes.iter().any(|s| s.entity == Some(entity) && s.recall.is_some())
    }
    /// Build once per public snapshot, rather than scanning camps per entity.
    pub fn disclosed_unit_attack_states(&self, paused: bool) -> BTreeMap<u64, crate::runtime::attack_visual_state::AttackVisualState> {
        let mut states:BTreeMap<_,_> = self.jungle_camps.iter().flat_map(|camp|std::iter::once(camp).chain(camp.members.iter())).filter_map(|body| body.entity.map(|entity| {
            let state = if self.phase == MobaMatchPhase::Playing { body.attack_visual_state(paused) }
                else { crate::runtime::attack_visual_state::AttackVisualState::default() };
            (crate::runtime::canonical_entity_id(entity), state)
        })).collect();
        for unit in self.units.values().filter(|u|matches!(u.role,LaneRole::Creep | LaneRole::Tower | LaneRole::NexusTower)) {
            let state=if self.phase==MobaMatchPhase::Playing && unit.attack_sequence>0 && unit.attack_remaining>Fixed64::ZERO {
                crate::runtime::attack_visual_state::AttackVisualState {sequence:unit.attack_sequence,phase:2,paused,
                    elapsed_raw:(Fixed64::ONE-unit.attack_remaining).max(Fixed64::ZERO).raw(),duration_raw:Fixed64::ONE.raw()}
            } else {Default::default()};
            states.insert(crate::runtime::canonical_entity_id(unit.entity),state);
        }
        states
    }
    /// Build once per visibility snapshot; no per-entity roster scan or private
    /// recall origin/deadline leaves the authority through this projection.
    pub fn disclosed_channel_states(&self) -> BTreeMap<u64, crate::runtime::channel_visual_state::ChannelVisualState> {
        use crate::runtime::channel_visual_state::ChannelVisualState;
        self.heroes.iter().filter_map(|slot| slot.entity.map(|entity| {
            let state = if self.phase == MobaMatchPhase::Playing && slot.recall.is_some() {
                ChannelVisualState::Recall
            } else { ChannelVisualState::None };
            (crate::runtime::canonical_entity_id(entity), state)
        })).collect()
    }
    #[cfg(feature = "kcp")]
    pub fn allows_player_input(&self, player_id: u32, input: &crate::runtime::PlayerInput) -> bool {
        // Transport independently enforces negotiated, authenticated capability.
        self.heroes.iter().any(|slot| slot.player_id == player_id) && single_lane_input_action_allowed(input)
    }

    pub fn owns_npc_combat(&self, entity: Entity) -> bool {
        self.jungle_camps.iter().any(|camp| camp.contains(entity)) || self.units
            .get(&entity_key(entity))
            .is_some_and(|unit| matches!(unit.role, LaneRole::Creep | LaneRole::Tower | LaneRole::NexusTower))
    }

    /// Current roster in slot order. `entity == None` keeps membership.
    /// Duplicate or zero player ids, and a side with no `config.teams` entry, fail closed.
    pub fn communication_roster(&self) -> Result<Vec<CommunicationMember>, CommunicationRosterError> {
        let mut seen = HashSet::new();
        let mut members = Vec::with_capacity(self.heroes.len());
        for slot in &self.heroes {
            if slot.player_id == 0 || !seen.insert(slot.player_id) {
                return Err(CommunicationRosterError::DuplicateOrZero);
            }
            let Some(team_id) = self.config.teams.get(slot.side).copied() else {
                return Err(CommunicationRosterError::UnresolvedTeam);
            };
            members.push(CommunicationMember {
                player_id: slot.player_id,
                side: slot.side,
                team_id,
            });
        }
        Ok(members)
    }

    fn communication_slot(&self, player_id: u32) -> Option<&MobaHeroSlot> {
        let mut found = None;
        for slot in &self.heroes {
            if slot.player_id == player_id {
                if found.is_some() {
                    return None;
                }
                found = Some(slot);
            }
        }
        found
    }

    /// Inclusive AABB of the compiled map's lanes, camps, and terrain, in `Vec2` raw units.
    /// The point set matches fog geometry sources. A legacy match with no `map_id` is the
    /// straight lane segment `x in [0, lane_length]`, `y == 0`.
    pub fn communication_point_allowed(&self, point: Vec2) -> bool {
        let Some((min_x, min_y, max_x, max_y)) = communication_bounds_raw(self) else {
            return false;
        };
        let x = point.x.raw();
        let y = point.y.raw();
        x >= min_x && x <= max_x && y >= min_y && y <= max_y
    }

    /// Authored camp home. Live entity position, respawn, and returning state are not read.
    /// Two camps with the same id fail closed.
    pub fn communication_objective_home(&self, objective_id: &str) -> Option<Vec2> {
        let mut home = None;
        for camp in &self.jungle_camps {
            if camp.definition.id == objective_id {
                if home.is_some() {
                    return None;
                }
                home = Some(Vec2::new(
                    Fixed64::from_i32(camp.definition.position.0),
                    Fixed64::from_i32(camp.definition.position.1),
                ));
            }
        }
        home
    }

    /// Public kills, deaths, and assists for a current roster player. No display name.
    pub fn communication_public_score(&self, player_id: u32) -> Option<(u32, u32, u32)> {
        let slot = self.communication_slot(player_id)?;
        Some((slot.kills, slot.deaths, slot.assists))
    }
}

/// One current roster member. Team id is `config.teams[side]`, not a cached copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommunicationMember {
    pub player_id: u32,
    pub side: usize,
    pub team_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommunicationRosterError {
    DuplicateOrZero,
    UnresolvedTeam,
}

fn communication_bounds_raw(state: &MobaMatch) -> Option<(i64, i64, i64, i64)> {
    if let Some(id) = state.config.map_id.as_deref() {
        let map = omoba_template_ids::moba_map_by_name(id)?;
        let mut bounds: Option<(i64, i64, i64, i64)> = None;
        let points = map
            .lanes
            .iter()
            .flat_map(|lane| lane.waypoints.iter().copied())
            .chain(map.jungle_camps.iter().flat_map(|camp|camp.home_positions()))
            .chain(map.terrain.iter().flat_map(|region| [region.min, region.max]));
        for (x, y) in points {
            let x = Fixed64::from_i32(x).raw();
            let y = Fixed64::from_i32(y).raw();
            bounds = Some(match bounds {
                None => (x, y, x, y),
                Some((min_x, min_y, max_x, max_y)) => {
                    (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
                }
            });
        }
        return bounds;
    }
    let max_x = state.config.lane_length.raw();
    if max_x < 0 {
        return None;
    }
    Some((0, 0, max_x, 0))
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
                | I::HoldPosition(_)
                | I::Stop(_)
                | I::CastAbility(_)
                | I::UpgradeAbility(_)
                | I::ItemUse(_)
                | I::ItemBuy(_)
                | I::ItemSell(_)
                | I::ShopUndo(_)
                | I::Recall(_)
                | I::SummonerCast(_)
                | I::WardPlacement(_)
                | I::CombatPreferences(_)
                | I::InputScheme(_)
                | I::Direction(_)
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
    if let Some(map) = config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name) {
        let lane = map.lanes.iter().find(|lane| lane.id == "mid").unwrap_or(&map.lanes[0]);
        let index = if team == 0 { 0 } else { lane.waypoints.len()-1 };
        let next = if team == 0 { 1 } else { index-1 };
        let point = |(x,y)| Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
        let origin = point(lane.waypoints[index]);
        let direction = point(lane.waypoints[next])-origin;
        return origin + direction*(offset.min(direction.length())/direction.length());
    }
    Vec2::new(
        if team == 0 {
            offset
        } else {
            config.lane_length - offset
        },
        Fixed64::ZERO,
    )
}

fn finalize_new_life_item_stats(world: &mut World, entity: Entity) {
    use crate::runtime::native::item::{apply_item_stats, ItemStatAggregate, ItemStatApplication};
    let inventory = world.read_storage::<Inventory>().get(entity).cloned().unwrap_or_default();
    let totals = ItemStatAggregate::from_inventory(&inventory, &world.read_resource::<crate::runtime::ItemRegistry>());
    let applied = world.read_storage::<ItemEffects>().get(entity).copied().unwrap_or_default();
    let mut properties = world.write_storage::<CProperty>();
    let mut attacks = world.write_storage::<TAttack>();
    let next = apply_item_stats(
        totals,
        &applied,
        properties.get_mut(entity),
        attacks.get_mut(entity),
        ItemStatApplication::NewLife,
    );
    drop(properties);
    drop(attacks);
    crate::runtime::native::item::sync_item_passives(entity,totals,&mut world.write_resource::<crate::runtime::BuffStore>());
    if let Some(effects) = world.write_storage::<ItemEffects>().get_mut(entity) {
        *effects = next;
    }
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
    if config.tower_plating_rules.is_some_and(|r|!r.is_valid()) {return Err(err_msg("invalid tower plating rules"));}
    if !config.tower_focus_rules.is_valid() {return Err(err_msg("invalid tower focus rules"));}
    if !config.death_streak_rules.is_valid() {return Err(err_msg("invalid death streak rules"));}
    if !config.kill_bounty_rules.is_valid() {return Err(err_msg("invalid kill bounty rules"));}
    if !config.building_reward_rules.is_valid() {return Err(err_msg("invalid building reward rules"));}
    if !config.minion_growth_rules.is_valid() {return Err(err_msg("invalid minion growth rules"));}
    if config.authored_minion_waves && config.map_id.as_deref()
        .and_then(omoba_template_ids::moba_map_by_name).is_none_or(|map|map.minion_wave.is_empty()) {
        return Err(err_msg("authored minion waves require a compiled map composition"));
    }
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
        || !config.objective_comeback_rules.is_valid()
        || config.first_kill_bonus_gold > 1_000_000
        || config.hero_kill_gold > 1_000_000
        || config.hero_assist_gold > 1_000_000
        || config.hero_kill_xp > 1_000_000
        || config.hero_assist_xp > 1_000_000
        || config.lane_creep_xp > 1_000_000
        || config.lane_creep_gold > 1_000_000
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
    let (summoner_catalog, summoner_loadouts) = super::summoner_world::validated_loadouts(
        roster.iter().map(|(id, _, _)| *id), &config.summoner_loadouts,
    ).map_err(|error| err_msg(format!("invalid summoner loadout: {error:?}")))?;
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
        if config.mana_enabled {
            let mut future = hero.clone();
            for level in 1..=25 {
                future.level = level;
                future.moba_mana_capacity().map_err(|error|
                    err_msg(format!("invalid MOBA mana capacity: {id} level {level}: {error:?}")))?;
            }
        }
        definitions.push(hero);
    }
    let needs_directional_plan = world.try_fetch::<crate::runtime::input_scheme::DirectionalStepPlan>().is_none();
    if needs_directional_plan {
        world.insert(crate::runtime::input_scheme::DirectionalStepPlan::default());
    }
    let heroes = roster.iter().enumerate().map(|(index, (player_id, side, _))| MobaHeroSlot {
        player_id: *player_id,
        combat_preferences: crate::runtime::combat_controls::OwnerCombatPreferences::legacy(*player_id).expect("validated MOBA roster owner"),
        input_scheme: crate::runtime::input_scheme::OwnerInputScheme::default(),
        side: *side,
        entity: None,
        deaths: 0,
        kills: 0,
        kill_streak: 0,
        unanswered_deaths: 0,
        assists: 0,
        lethal_pending: false,
        recall: None,
        respawns: 0,
        respawn_at: None,
        hero: definitions[index].clone(),
        inventory: Inventory::default(),
        gold: Gold(0),
        summoners: summoner_loadouts.get(player_id).cloned().unwrap_or_default(),
        trinket: super::ward_world::WardRosterState::unavailable(),
    }).collect();
    let mut state = MobaMatch {
        base_structures: Vec::new(),
        config,
        phase: MobaMatchPhase::Warmup,
        elapsed: Fixed64::ZERO,
        input_step: None,
        input_reset_sources: BTreeSet::new(),
        input_attack_reset_sources: BTreeSet::new(),
        earned_gold: [0;2],
        comeback: objective_comeback::ComebackState::default(),
        income_remainder_raw: 0,
        pending_income_delta_raw: 0,
        waves: 0,
        heroes,
        towers: [None; 2],
        lane_towers: vec![[None;2]; routes.len()],
        lane_tower_layers: vec![Vec::new(); routes.len()],
        routes,
        bases: [None; 2],
        jungle_camps: Vec::new(),
        assist_ledger: super::moba_assist::AssistLedger::default(),
        first_tower:first_tower::FirstTowerState::default(),
        first_kill:first_kill::FirstKillState::default(),
        objective_notices:objective_notice::ObjectiveNotices::default(),
        objective_buffs:BTreeMap::new(),
        tower_plates:BTreeMap::new(),
        minion_aura_targets:BTreeMap::new(),
        personal_buffs:BTreeMap::new(),
        champion_aggression: BTreeMap::new(),
        brush_attack_reveals: BTreeMap::new(),
        next_wave_at: Fixed64::ZERO,
        units: BTreeMap::new(),
        summoner_catalog,
        ward_policies: BTreeMap::new(),
    };
    world.insert(state.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name)
        .map(crate::runtime::brush_vision::BrushVisionRegions::from_map).unwrap_or_default());
    if let Some(map) = state.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name) {
        *world.write_resource::<BlockedRegions>() =
            crate::runtime::moba_map_layout::compiled_blocked_regions(map);
        // A compiled map owns its geometry. Do not merge unrelated campaign
        // occluders; legacy campaign worlds keep their descriptor resource.
        *world.write_resource::<VisionOccluderSet>() =
            crate::runtime::moba_map_layout::compiled_vision_occluders(map);
    }
    world.write_resource::<crate::runtime::TeamVisibilityRuntime>().fog_geometry =
        state.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name)
            .map(crate::runtime::fog_grid::FogGridGeometry::from_map)
            .transpose().map_err(err_msg)?;
    let tower_offsets = state.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name)
        .map_or_else(|| vec![700], |map| map.tower_layers.to_vec());
    for lane in &mut state.lane_tower_layers { *lane = vec![[None; 2]; tower_offsets.len()]; }
    world.insert(crate::runtime::ItemRegistry::generated_moba());
    for team in 0..2 {
        state.spawn_hero(world, team);
        for lane in 0..state.routes.len() {
            for (layer, offset) in tower_offsets.iter().enumerate() {
                let tower = state.spawn_structure(world, team, LaneRole::Tower, lane, Fixed64::from_i32(*offset));
                state.lane_tower_layers[lane][layer][team] = Some(tower);
            }
            state.lane_towers[lane][team] = state.lane_tower_layers[lane][0][team];
        }
        state.towers[team] = state.lane_towers[0][team];
        state.bases[team] = Some(state.spawn_structure(world, team, LaneRole::Base, 0, Fixed64::ZERO));
    }
    for index in 2..state.heroes.len() { state.spawn_hero(world, index); }
    state.setup_base_structures(world);
    jungle::setup(world, &mut state);
    super::ward_world::apply_configured_trinket_loadouts(&mut state)
        .map_err(|error| err_msg(format!("invalid trinket loadout: {error:?}")))?;
    *world.write_resource::<GameMode>() = GameMode::Moba;
    world.write_resource::<MasterSeed>().0 = state.config.seed;
    world.write_resource::<CurrentCreepWave>().is_running = false;
    world.insert(state);
    world.insert(crate::runtime::shop_undo_wire::LockstepHashContract {
        version: crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION,
    });
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
                last_hit_retired: false,
                target: None,
                target_protected: false,
                tower_focus: Default::default(),
                attack_sequence: 0,
            },
        );
    }

    fn spawn_hero(&mut self, world: &mut World, index: usize) {
        let slot = &self.heroes[index];
        let mut hero = slot.hero.clone();
        if self.config.mana_enabled {
            // A new life starts full; do not reuse the saved death balance.
            hero.mana_pool = Some(crate::runtime::ability_runtime::ManaPool::full(
                hero.moba_mana_capacity().expect("validated MOBA mana capacity"))
                .expect("validated MOBA mana pool"));
        }
        let team = slot.side;
        let rank = self.heroes[..index].iter().filter(|s| s.side == team).count();
        let mut spawn = position(&self.config, team, Fixed64::from_i32(120 +
            if self.config.map_id.is_some() { rank as i32 * 60 } else { 0 }));
        if self.config.map_id.is_none() { spawn.y += Fixed64::from_i32(rank as i32 * 60); }
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
            .with(hero)
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
        finalize_new_life_item_stats(world, entity);
        let player_id = self.heroes[index].player_id;
        super::shop::bind_roster_shop_undo(world, player_id, entity);
        self.heroes[index].entity = Some(entity);
        self.heroes[index].lethal_pending = false;
        self.heroes[index].recall = None;
        self.heroes[index].respawn_at = None;
        self.track(entity, team, LaneRole::Hero);
    }

    fn spawn_structure(&mut self, world: &mut World, team: usize, role: LaneRole, lane: usize, offset: Fixed64) -> Entity {
        let (id, hp) = match role {
            LaneRole::Tower => (
                "single_lane_tower",
                self.config.tower_hp,
            ),
            LaneRole::Base => ("single_lane_base", self.config.base_hp),
            LaneRole::Inhibitor => ("moba_inhibitor", Fixed64::from_i32(self.base_structure_rules().expect("authored structure").inhibitor_hp)),
            LaneRole::NexusTower => ("moba_nexus_tower", Fixed64::from_i32(self.base_structure_rules().expect("authored structure").nexus_tower_hp)),
            _ => unreachable!(),
        };
        let mut unit = Unit::new(id.into(), id.into(), UnitType::Neutral);
        unit.max_hp = (hp.raw() / 1024) as i32;
        unit.current_hp = unit.max_hp;
        unit.base_damage = if role.is_tower() { 90 } else { 0 };
        unit.attack_range = Fixed64::from_i32(if role.is_tower() { 650 } else { 0 });
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
        let composition:Vec<_>=if self.config.authored_minion_waves {
            omoba_template_ids::moba_map_by_name(self.config.map_id.as_deref().expect("validated map"))
                .expect("validated map").minion_wave.iter().filter(|entry|self.waves%entry.every==0)
                .flat_map(|entry|std::iter::repeat_n(*entry,entry.count as usize)).collect()
        } else {
            vec![omoba_template_ids::MobaMinionConst {id:"single_lane_creep",count:1,every:1,
                hp:180,damage:24,attack_range:140,move_speed:240};self.config.creeps_per_wave as usize]
        };
        for team in 0..2 {
          for lane in 0..self.routes.len() {
            let lane_composition=self.reinforced_wave(team,lane,&composition);
            for (index,definition) in lane_composition.iter().enumerate() {
                let route = &self.routes[lane][team];
                let mut cursor = 1;
                let spawn = omoba_sim::navigation::advance_route(route,&mut cursor,route[0],
                    Fixed64::from_i32(180 + index as i32 * 35));
                let mut unit = Unit::new(
                    definition.id.into(),
                    "Lane creep".into(),
                    UnitType::Creep,
                );
                let (hp,damage)=self.config.minion_growth_rules.stats(definition.hp,definition.damage,self.elapsed);
                unit.max_hp = hp;
                unit.current_hp = hp;
                unit.base_damage = damage;
                unit.attack_range = Fixed64::from_i32(definition.attack_range);
                unit.move_speed = Fixed64::from_i32(definition.move_speed);
                let entity = world
                    .create_entity()
                    .with(unit)
                    .with(Creep {
                        name: definition.id.into(),
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
                    .with(property(Fixed64::from_i32(hp), Fixed64::from_i32(definition.move_speed)))
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

#[cfg(feature = "kcp")]
fn gameplay_command_interrupts_recall(action: &Option<crate::runtime::PlayerInputEnum>) -> bool {
    use crate::runtime::PlayerInputEnum as I;
    matches!(
        action,
        Some(
            I::MoveTo(_)
                | I::AttackMove(_)
                | I::AttackTarget(_)
                | I::HoldPosition(_)
                | I::Stop(_)
                | I::CastAbility(_)
                | I::ItemUse(_)
        )
    )
}

/// Gameplay commands win over Recall within the same input batch, independent
/// of ordering. Stop is included. Recall itself is consumed here.
#[cfg(feature = "kcp")]
fn apply_recall_commands(world: &mut World, state: &mut MobaMatch, previous_elapsed: Fixed64, direction_begins: &BTreeSet<u32>) {
    use crate::runtime::PlayerInputEnum as I;
    let mut interrupted: std::collections::BTreeSet<u32> = world
        .read_resource::<PendingPlayerInputs>()
        .inputs
        .iter()
        .filter_map(|(player, input)| gameplay_command_interrupts_recall(&input.action).then_some(*player))
        .collect();
    interrupted.extend(direction_begins.iter().copied());
    world.write_resource::<PendingPlayerInputs>().inputs.retain(|(player, input)| {
        let side = state.heroes.iter().position(|slot| slot.player_id == *player).expect("roster filtered");
        let team = state.heroes[side].side;
        if matches!(input.action, Some(I::Recall(_))) {
            if interrupted.contains(player) {
                state.heroes[side].recall = None;
                return false;
            }
            if let Some(entity) = state.heroes[side].entity {
                if world.read_storage::<CProperty>().get(entity).is_some_and(|property| property.hp > Fixed64::ZERO)
                    && !recall_control_blocked(&world.read_resource::<crate::runtime::ability_runtime::BuffStore>(), entity)
                    && state.bases[team].is_some()
                    && state.heroes[side].recall.is_none()
                {
                    let origin = world.read_storage::<Pos>().get(entity).expect("hero position").0;
                    state.heroes[side].recall = Some((
                        origin,
                        previous_elapsed.max(state.config.warmup)
                            + Fixed64::from_i32(omoba_template_ids::MOBA_RECALL_CHANNEL_SECONDS as i32),
                    ));
                    state.heroes[side].input_scheme.neutralize();
                    if let Some(queue) = world.write_storage::<HeroCommandQueue>().get_mut(entity) {
                        queue.clear_all();
                    }
                    world.write_storage::<MoveTarget>().remove(entity);
                }
            }
            return false;
        }
        if gameplay_command_interrupts_recall(&input.action) {
            state.heroes[side].recall = None;
        }
        true
    });
}

/// Apply a previously authenticated directional event to its exact live hero.
/// Presses retire point commands and recall once, preserving attack commitment. Updates
/// only renew the same gesture, including after a packet-loss lease timeout.
pub fn apply_direction_input(world: &mut World, owner: u32, source: Entity,
    gesture: crate::runtime::input_scheme::DirectionGesture, x: i32, y: i32)
    -> Result<crate::runtime::input_scheme::DirectionAdmission, crate::runtime::input_scheme::InputSchemeError> {
    let mut state=world.remove::<MobaMatch>().ok_or(crate::runtime::input_scheme::InputSchemeError::InvalidPhase)?;
    let allow_movement=!world.try_fetch::<GamePause>().is_some_and(|pause|pause.is_paused)
        && world.read_resource::<DeltaTime>().0>Fixed64::ZERO;
    let result=apply_direction_with_state(world,&mut state,owner,source,gesture,x,y,allow_movement);
    world.insert(state);result
}

fn apply_direction_with_state(world: &mut World, state: &mut MobaMatch, owner: u32, source: Entity,
    gesture: crate::runtime::input_scheme::DirectionGesture, x: i32, y: i32, allow_movement: bool)
    -> Result<crate::runtime::input_scheme::DirectionAdmission, crate::runtime::input_scheme::InputSchemeError> {
    use crate::runtime::input_scheme::{DirectionAdmission, DirectionGesture, InputSchemeError};
    if !allow_movement
        && !matches!(gesture, DirectionGesture::End(_)) { return Err(InputSchemeError::InvalidPhase); }
    if !world.entities().is_alive(source)
        || world.read_storage::<Hero>().get(source).is_none()
        || !world.read_storage::<PlayerOwner>().get(source).is_some_and(|value| value.player_id==owner)
        || !world.read_storage::<CProperty>().get(source).is_some_and(|value| value.hp>Fixed64::ZERO)
    { return Err(InputSchemeError::InvalidSource); }
    let index=state.input_scheme_owner_index(owner,owner)?;
    let life=crate::runtime::selective::CanonicalEntityKey {id:source.id(),generation:source.gen().id() as u32};
    let was_held=state.heroes[index].input_scheme.active_press(life)!=0;
    let admission = {
        let admission=state.admit_direction(owner,source,gesture,x,y)?;
        if admission==DirectionAdmission::Began {
            let index=state.input_scheme_owner_index(owner,owner)?;
            state.heroes[index].recall=None;
            state.input_reset_sources.insert(source);
        }
        admission
    };
    if admission==DirectionAdmission::Began {
        super::game_processor::clear_hero_command_queue(world,source);
        // A new keyboard press replaces navigation, not an attack in progress.
    } else if admission==DirectionAdmission::Ended && was_held {
        // The next command tick keeps an explicit attack, but no stale movement
        // waypoint may outlive focus loss or a matching key release.
        world.write_storage::<MoveTarget>().remove(source);
    }
    Ok(admission)
}

fn apply_input_scheme_with_state(world: &mut World,state:&mut MobaMatch,owner:u32,raw:u32)
    -> Result<bool,crate::runtime::input_scheme::InputSchemeError> {
    let index=state.input_scheme_owner_index(owner,owner)?;
    let previous=state.heroes[index].input_scheme.scheme();
    state.admit_input_scheme(owner,owner,raw)?;
    if state.heroes[index].input_scheme.scheme()==previous {return Ok(false);}
    state.heroes[index].recall=None;
    if let Some(source)=state.heroes[index].entity.filter(|entity|world.entities().is_alive(*entity)
        && world.read_storage::<Hero>().get(*entity).is_some()
        && world.read_storage::<PlayerOwner>().get(*entity).is_some_and(|value|value.player_id==owner)) {
        state.input_reset_sources.insert(source);
        state.input_attack_reset_sources.insert(source);
        super::game_processor::clear_hero_command_queue(world,source);
        super::game_processor::interrupt_attack_for_accepted_command(world,source);
    }
    Ok(true)
}

/// Roster-only setting: works while dead and during warmup; a genuine mode
/// change retires held input and commands once. Idempotent retries are inert.
pub fn apply_input_scheme_update(world:&mut World,owner:u32,raw:u32)
    -> Result<bool,crate::runtime::input_scheme::InputSchemeError> {
    let mut state=world.remove::<MobaMatch>().ok_or(crate::runtime::input_scheme::InputSchemeError::InvalidPhase)?;
    let result=apply_input_scheme_with_state(world,&mut state,owner,raw);
    world.insert(state);result
}

#[cfg(feature="kcp")]
fn apply_owner_scheme_inputs(world:&mut World,state:&mut MobaMatch,allow_movement:bool) -> BTreeSet<u32> {
    let inputs=std::mem::take(&mut world.write_resource::<PendingPlayerInputs>().inputs);
    let mut retained=Vec::with_capacity(inputs.len());
    let mut direction_begins=BTreeSet::new();
    for (owner,input) in inputs {
        if let Some(crate::runtime::PlayerInputEnum::InputScheme(value))=input.action.as_ref() {
            if crate::runtime::input_scheme_transport::validate_update(value).is_err() {
                log::warn!("input scheme rejected for owner {}: invalid shape",owner);
            } else if let Err(reason)=apply_input_scheme_with_state(world,state,owner,value.scheme) {
                log::warn!("input scheme rejected for owner {}: {:?}",owner,reason);
            }
        } else if let Some(crate::runtime::PlayerInputEnum::Direction(value))=input.action.as_ref() {
            let result=crate::runtime::direction_transport::validate_sealed(value).map_err(|_|crate::runtime::input_scheme::InputSchemeError::InvalidGesture)
                .and_then(|gesture| {
                    let source=Entity::new(value.sealed_source as u32,specs::world::Generation::new((value.sealed_source>>32) as i32));
                    apply_direction_with_state(world,state,owner,source,gesture,value.x,value.y,allow_movement)
                });
            match result {
                Ok(crate::runtime::input_scheme::DirectionAdmission::Began)=>{
                    if let Some(entity)=state.heroes.iter().find(|h|h.player_id==owner).and_then(|h|h.entity) {
                        super::recovery_channel::cancel(world,entity);
                    }
                    direction_begins.insert(owner);
                },
                Ok(_)=>{},Err(reason)=>log::warn!("direction rejected for owner {}: {:?}",owner,reason),
            }
        } else {retained.push((owner,input));}
    }
    world.write_resource::<PendingPlayerInputs>().inputs=retained;
    direction_begins
}

/// Pre-dispatch hook. Timers use accumulated scaled fixed delta, never wall time.
/// Returns false outside active play so callers can freeze the whole gameplay
/// pipeline, not merely the lane AI. A paused world is inactive even when
/// `DeltaTime` is positive: match elapsed does not advance, so absolute ward
/// deadlines cannot absorb the paused span on resume. Existing worlds without
/// this resource pass.
pub fn begin_moba_match_tick(world: &mut World) -> bool {
    if world.try_fetch::<MobaMatch>().is_some_and(|state|state.phase!=MobaMatchPhase::Playing) {
        super::recovery_channel::clear(world);
    }
    // Results are never replay state and must not survive an inactive tick.
    {
        let mut queue = world.write_resource::<PendingItemUseQueue>();
        queue.settlements.clear();
        queue.undo_settlements.clear();
    }
    let Some(mut state) = world.remove::<MobaMatch>() else {
        return true;
    };
    // An inactive tick must overwrite the previous active controls too.
    state.input_step = Some(crate::runtime::input_scheme::InputSchemeStepSnapshot {
        tick: world.read_resource::<Tick>().0, rows: BTreeMap::new(),
    });
    state.pending_income_delta_raw = 0;
    #[cfg(feature = "kcp")]
    world
        .write_resource::<PendingPlayerInputs>()
        .inputs
        .retain(|(player, input)| state.heroes.iter().any(|s| s.player_id == *player) && single_lane_input_action_allowed(input));
    if matches!(state.phase, MobaMatchPhase::Finished { .. }) {
        objective_comeback::begin(world,&mut state);
        objective_notice::settle(world,&mut state);
        tower_plating::synchronize(world,&state,Fixed64::ZERO);
        objective_buffs::synchronize(world,&mut state,Fixed64::ZERO);
        personal_buffs::synchronize(world,&mut state,Fixed64::ZERO);
        #[cfg(feature="kcp")]
        apply_owner_scheme_inputs(world,&mut state,false);
        let records = super::ward_world::take_ward_records(&mut state);
        super::ward_world::destroy_ward_records(world, &records);
        reject_inactive_shop_inputs(world, &state);
        emit_replica_delta(world, &state, false);
        world.insert(state);
        return false;
    }
    let dt = world.read_resource::<DeltaTime>().0;
    if world
        .try_fetch::<GamePause>()
        .is_some_and(|pause| pause.is_paused)
    {
        #[cfg(feature="kcp")]
        apply_owner_scheme_inputs(world,&mut state,false);
        reject_inactive_shop_inputs(world, &state);
        emit_replica_delta(world, &state, false);
        world.insert(state);
        return false;
    }
    let previous_elapsed = state.elapsed;
    state.elapsed += dt;
    let active=state.elapsed>=state.config.warmup && dt>Fixed64::ZERO;
    if active && state.phase == MobaMatchPhase::Warmup {
        state.phase = MobaMatchPhase::Playing;
        state.next_wave_at = state.elapsed;
    }
    #[cfg(feature="kcp")]
    let direction_begins=apply_owner_scheme_inputs(world,&mut state,active);
    if !active {
        reject_inactive_shop_inputs(world, &state);
        emit_replica_delta(world, &state, false);
        world.insert(state);
        return false;
    }
    // Count only the playing portion of the warmup-crossing tick.
    state.pending_income_delta_raw = (state.elapsed - previous_elapsed.max(state.config.warmup))
        .raw()
        .max(0);
    let summoner_delta = Fixed64::from_raw(state.pending_income_delta_raw);
    let paused = world.read_resource::<GamePause>().is_paused;
    super::summoner_world::tick_match_loadouts(&mut state, summoner_delta, paused);
    super::ward_world::tick_match_wards(world, &mut state, summoner_delta, paused);
    #[cfg(feature = "kcp")]
    apply_recall_commands(world, &mut state, previous_elapsed, &direction_begins);
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
    objective_comeback::begin(world,&mut state);
    state.tick_base_structures(world);
    tower_plating::synchronize(world,&state,dt);
    personal_buffs::synchronize(world,&mut state,dt);
    while state.elapsed >= state.next_wave_at {
        state.spawn_wave(world);
        state.next_wave_at += state.config.wave_interval;
    }
    jungle::tick(world, &mut state, dt);
    // Include newly spawned waves before their first combat decision.
    objective_buffs::synchronize(world,&mut state,dt);
    // Units and decisions are always reduced in (entity ID, generation) order.
    let positions = world.read_storage::<Pos>();
    let properties = world.read_storage::<CProperty>();
    let regions = world.read_resource::<BlockedRegions>();
    let bodies = world.read_storage::<Unit>();
    let visibility = world.try_fetch::<crate::runtime::TeamVisibilityRuntime>();
    let mut movement = Vec::new();
    let mut damage = Vec::new();
    let mut brush_attacks = Vec::new();
    let units: Vec<_> = state.units.values().cloned().collect();
    let buffs=world.read_resource::<crate::runtime::BuffStore>();
    let npc_stats=crate::runtime::UnitStats::from_refs(&buffs,false);
    state.champion_aggression.retain(|_, aggression| aggression.expires > state.elapsed
        && properties.get(aggression.attacker).is_some_and(|p| p.hp > Fixed64::ZERO));
    state.brush_attack_reveals.retain(|_, reveal| reveal.expires > state.elapsed
        && world.entities().is_alive(reveal.attacker)
        && properties.get(reveal.attacker).is_some_and(|p| p.hp > Fixed64::ZERO));
    for unit in &units {
        if !matches!(unit.role, LaneRole::Creep | LaneRole::Tower | LaneRole::NexusTower)
            || properties
                .get(unit.entity)
                .is_none_or(|p| p.hp <= Fixed64::ZERO)
        {
            continue;
        }
        let source_pos = positions.get(unit.entity).expect("lane unit position").0;
        let range = bodies.get(unit.entity).map(|body|body.attack_range).unwrap_or_else(||Fixed64::from_i32(if unit.role.is_tower() {
            650
        } else {
            140
        }));
        let aggro = if unit.role.is_tower() {range} else {range.max(Fixed64::from_i32(700))};
        let sieging=unit.role==LaneRole::Creep && unit.target.and_then(|target|state.units.get(&entity_key(target)))
            .is_some_and(|target| target.role.is_tower() && properties.get(target.entity).is_some_and(|p|p.hp>Fixed64::ZERO) && positions.get(target.entity)
                .is_some_and(|pos|(pos.0-source_pos).length_squared()<=range*range));
        let target = units
            .iter()
            .filter(|other| other.team != unit.team)
            .filter(|other| !unit.role.is_tower() || matches!(other.role,LaneRole::Hero | LaneRole::Creep))
            .filter(|other| visibility.as_ref().and_then(|v|v.teams.get(&state.config.teams[unit.team as usize]))
                .is_none_or(|team|team.index.current.contains(&crate::runtime::canonical_entity_id(other.entity))))
            .filter(|other| {
                properties
                    .get(other.entity)
                    .is_some_and(|p| p.hp > Fixed64::ZERO)
            })
            .filter(|other| {
                state.structure_unlocked(other)
            })
            .filter(|other| matches!(other.role,LaneRole::Hero | LaneRole::Base | LaneRole::NexusTower) || other.lane == unit.lane)
            .filter_map(|other| {
                positions
                    .get(other.entity)
                    .map(|p| (other, p.0, (p.0 - source_pos).length_squared()))
            })
            .filter(|(_, _, distance)| *distance <= aggro * aggro)
            .min_by_key(|(other, _, distance)| {
                let protects = state.protects_champion(unit, source_pos, other, sieging);
                (
                    !protects,
                    unit.target != Some(other.entity) || (unit.target_protected && !protects),
                    match other.role {LaneRole::Creep=>0,LaneRole::Hero=>1,LaneRole::Tower|LaneRole::Inhibitor|LaneRole::NexusTower=>2,LaneRole::Base=>3},
                    distance.raw(),
                    entity_key(other.entity),
                )
            });
        let target_protected = target.is_some_and(|(other,_,_)|state.protects_champion(unit,source_pos,other,sieging));
        let tracked = state
            .units
            .get_mut(&entity_key(unit.entity))
            .expect("tracked unit");
        tracked.attack_remaining = (tracked.attack_remaining - dt).max(Fixed64::ZERO);
        tracked.target = target.map(|(target,_,_)|target.entity);
        tracked.target_protected = target_protected;
        if unit.role.is_tower() {
            tracked.tower_focus.reconcile(target.filter(|(target,_,_)|target.role==LaneRole::Hero)
                .map(|(target,_,_)|target.entity),state.elapsed,state.config.tower_focus_rules);
        }
        if let Some((target, _, distance)) = target {
            if distance <= range * range {
                if tracked.attack_remaining <= Fixed64::ZERO {
                    let base=Fixed64::from_i32(bodies.get(unit.entity).map_or(
                        if unit.role.is_tower() {90} else {24},|body|body.base_damage));
                    let base=if unit.role==LaneRole::Creep {npc_stats.normal_attack_physical(base,unit.entity)} else {base};
                    let amount=if unit.role.is_tower() && target.role==LaneRole::Hero {
                        tracked.tower_focus.commit(target.entity,state.elapsed,base,state.config.tower_focus_rules)
                    } else {base};
                    damage.push(DamageInstance::new_attack(
                        unit.entity,
                        target.entity,
                        amount,
                    ));
                    tracked.attack_remaining = Fixed64::ONE;
                    brush_attacks.push((unit.entity,target.entity));
                    tracked.attack_sequence=tracked.attack_sequence.wrapping_add(1).max(1);
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
    drop(bodies);
    drop(visibility);
    drop(buffs);
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
    capture_input_scheme_step(world, &mut state);
    world.insert(state);
    for (attacker,victim) in brush_attacks {record_brush_attack_reveal(world,attacker,victim);}
    true
}

fn capture_input_scheme_step(world: &World, state: &mut MobaMatch) {
    use crate::runtime::input_scheme::{InputSchemeStepSnapshot, OwnerDirectionalStep};
    let entities = world.entities();
    let heroes = world.read_storage::<Hero>();
    let owners = world.read_storage::<PlayerOwner>();
    let health = world.read_storage::<CProperty>();
    let mut rows = BTreeMap::new();
    for slot in &state.heroes {
        let Some(source) = slot.entity else { continue; };
        if state.input_scheme_owner_index(slot.player_id, slot.player_id).is_err()
            || slot.lethal_pending || !entities.is_alive(source) || heroes.get(source).is_none()
            || !owners.get(source).is_some_and(|owner| owner.player_id == slot.player_id)
            || !health.get(source).is_some_and(|property| property.hp > Fixed64::ZERO) { continue; }
        let key = crate::runtime::selective::CanonicalEntityKey { id: source.id(), generation: source.gen().id() as u32 };
        rows.insert(slot.player_id, OwnerDirectionalStep {
            source, scheme: slot.input_scheme().scheme(),
            direction: if slot.recall.is_some() { Vec2::ZERO } else { state.direction_for_source(slot.player_id, source) },
            press_id: slot.input_scheme().active_press(key),
            navigation_reset: state.input_reset_sources.contains(&source),
            attack_reset: state.input_attack_reset_sources.contains(&source),
        });
    }
    state.input_step = Some(InputSchemeStepSnapshot { tick: world.read_resource::<Tick>().0, rows });
    state.input_reset_sources.clear();
    state.input_attack_reset_sources.clear();
}

fn reject_inactive_shop_inputs(world: &mut World, state: &MobaMatch) {
    #[cfg(feature = "kcp")]
    {
        use crate::runtime::{
            shop::{ShopCommand, ShopError, ShopSettlement},
            PlayerInputEnum,
        };
        let mut results = Vec::new();
        let mut undo_results = Vec::new();
        for (player, input) in &world.read_resource::<PendingPlayerInputs>().inputs {
            match &input.action {
                Some(PlayerInputEnum::ItemBuy(buy)) => results.push(ShopSettlement {
                    player_id: *player,
                    command: ShopCommand::Buy(buy.item_id.clone()),
                    result: Err(ShopError::MatchUnavailable),
                }),
                Some(PlayerInputEnum::ItemSell(sell)) => results.push(ShopSettlement {
                    player_id: *player,
                    command: ShopCommand::Sell(sell.item_slot as usize),
                    result: Err(ShopError::MatchUnavailable),
                }),
                Some(PlayerInputEnum::ShopUndo(undo)) => {
                    let request = if undo.mode == crate::runtime::shop_undo_wire::SHOP_UNDO_MODE_SEQUENCE {
                        crate::runtime::shop::ShopUndoRequest::Sequence(undo.sequence)
                    } else {
                        crate::runtime::shop::ShopUndoRequest::Sequence(0)
                    };
                    let (gold, catalogs, cooldowns) = crate::runtime::shop::projected_owner_wallet_in(world, state, *player);
                    undo_results.push(crate::runtime::shop::ShopUndoSettlement {
                        player_id: *player,
                        request,
                        result: Err(ShopError::MatchUnavailable),
                        gold,
                        catalogs,
                        cooldowns,
                    });
                }
                _ => {}
            }
        }
        let mut queue = world.write_resource::<PendingItemUseQueue>();
        queue.settlements.extend(results);
        queue.undo_settlements.extend(undo_results);
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

/// Credit the authenticated player's current life or admitted respawn wallet.
/// The projectile source itself can already have retired or changed generation.
fn credit_moba_player(world:&World,state:&mut MobaMatch,index:usize,gold:u32,xp:u32)->bool {
    let slot=&mut state.heroes[index];
    let paid;
    let amount=gold.min(i32::MAX as u32) as i32;
    if let Some(entity)=slot.entity {
        if !world.entities().is_alive(entity) {return false;}
        {
            let mut balances=world.write_storage::<Gold>();
            let Some(balance)=balances.get_mut(entity) else {return false;};
            let before=balance.0;
            balance.0=balance.0.saturating_add(amount);
            paid=(i64::from(balance.0)-i64::from(before)) as u64;
        }
        if xp>0 {award_live_moba_xp(world,entity,xp);}
    } else if let Some((_,balance))=slot.admitted_dead_wallet() {
        let before=balance.0;
        balance.0=balance.0.saturating_add(amount);
        paid=(i64::from(balance.0)-i64::from(before)) as u64;
        if xp>0 {slot.hero.add_moba_experience(xp);}
    } else {return false;}
    state.earned_gold[slot.side]=state.earned_gold[slot.side].saturating_add(paid);
    true
}

/// Positive generic reward outcomes count too; shop refunds and sales never use this path.
pub(crate) fn record_moba_gold_gain(world:&World,target:Entity,paid:u64) {
    if paid==0 || world.read_resource::<GamePause>().is_paused {return;}
    let Some(mut state)=world.try_fetch_mut::<MobaMatch>() else {return;};
    if state.phase!=MobaMatchPhase::Playing {return;}
    let Some(side)=state.heroes.iter().find(|s|s.entity==Some(target)).map(|s|s.side) else {return;};
    state.earned_gold[side]=state.earned_gold[side].saturating_add(paid);
}

/// All positive damage paths retire a creep's first lethal hit, even when it
/// has no credited hero. Forced Death and a later Heal cannot manufacture gold.
pub(crate) fn record_moba_lane_damage(world:&World,source:Option<Entity>,target:Entity,lethal:bool) {
    let Some(mut state)=world.try_fetch_mut::<MobaMatch>() else {return;};
    if !lethal || state.phase!=MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused {return;}
    let victim_side={
        let Some(unit)=state.units.get_mut(&entity_key(target)) else {return;};
        if unit.role!=LaneRole::Creep || unit.last_hit_retired {return;}
        unit.last_hit_retired=true;
        usize::from(unit.team)
    };
    let Some(credit)=jungle_credit::resolve(world,&state,source).filter(|c|c.side!=victim_side) else {return;};
    let Some(index)=state.heroes.iter().position(|slot|Some(slot.player_id)==credit.player && slot.side==credit.side) else {return;};
    let gold=state.config.lane_creep_gold;
    credit_moba_player(world,&mut state,index,gold,0);
}

/// Called after actual positive damage to a live target, not after an input ACK.
/// Anonymous script damage can retire a life but cannot claim participation.
pub(crate) fn record_moba_hero_damage(world: &World, source: Option<Entity>, target: Entity, lethal: bool) {
    use super::moba_assist::AssistParticipant;
    if let Some(source) = source { record_moba_champion_aggression(world, source, target, false); }
    let Some(mut state) = world.try_fetch_mut::<MobaMatch>() else { return; };
    if state.phase != MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused {
        return;
    }
    let Some(victim) = state.heroes.iter().position(|slot| slot.entity == Some(target)) else { return; };
    if state.heroes[victim].lethal_pending { return; }
    let roster: Vec<_> = state.heroes.iter().map(|slot| AssistParticipant {
        player_id: slot.player_id, team_id: state.config.teams[slot.side],
    }).collect();
    let killer = jungle_credit::resolve(world,&state,source)
        .and_then(|credit|state.heroes.iter().position(|slot|Some(slot.player_id)==credit.player && slot.side==credit.side))
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
    let bonus=state.config.kill_bounty_rules.bonus(state.heroes[victim].kill_streak);
    state.heroes[victim].kill_streak=0;
    let Some(killer) = killer else { return; };
    let base_gold=state.config.death_streak_rules.gold(state.config.hero_kill_gold,state.heroes[victim].unanswered_deaths);
    let (kill_gold,kill_xp)=(base_gold.saturating_add(bonus),state.config.hero_kill_xp);
    if !credit_moba_player(world,&mut state,killer,kill_gold,kill_xp) {return;}
    personal_buffs::transfer(world,&mut state,killer,victim);
    state.heroes[victim].unanswered_deaths=state.heroes[victim].unanswered_deaths.saturating_add(1);
    state.heroes[killer].unanswered_deaths=0;
    state.heroes[killer].kills = state.heroes[killer].kills.saturating_add(1);
    state.heroes[killer].kill_streak = state.heroes[killer].kill_streak.saturating_add(1);
    let (winner,loser,bonus)=(state.heroes[killer].player_id,state.heroes[victim].player_id,state.config.first_kill_bonus_gold);
    state.first_kill.observe(world.read_resource::<Tick>().0,key,winner,loser,bonus);
    let assist_gold = state.config.hero_assist_gold;
    let assist_xp = state.config.hero_assist_xp;
    for player in assistants {
        let index=state.heroes.iter().position(|slot|slot.player_id==player).expect("validated roster");
        if credit_moba_player(world,&mut state,index,assist_gold,assist_xp) {
            state.heroes[index].assists=state.heroes[index].assists.saturating_add(1);
        }
    }
}

/// Brush exposure is not true sight and never creates an observing source.
/// Called only by admitted basic-attack paths, never by a submitted request.
pub(crate) fn record_brush_attack_reveal(world: &World, attacker: Entity, victim: Entity) {
    let Some(mut state)=world.try_fetch_mut::<MobaMatch>() else {return};
    record_brush_attack_reveal_in(world,&mut state,attacker,victim);
}

const BRUSH_ATTACK_REVEAL_DURATION:Fixed64=Fixed64::from_raw(2048);

fn record_brush_attack_reveal_in(world:&World,state:&mut MobaMatch,attacker:Entity,victim:Entity) {
    let Some(regions)=world.try_fetch::<crate::runtime::brush_vision::BrushVisionRegions>() else {return};
    if regions.regions().is_empty() {return;}
    let positions=world.read_storage::<Pos>();
    let Some(position)=positions.get(attacker) else {return};
    if !regions.regions().iter().any(|region|region.contains(position.0)) {return;}
    let factions=world.read_storage::<Faction>();
    let (Some(source),Some(target))=(factions.get(attacker),factions.get(victim)) else {return};
    if target.team_id<=0 || source.team_id==target.team_id {return;}
    let properties=world.read_storage::<CProperty>();
    if [attacker,victim].iter().any(|entity| !world.entities().is_alive(*entity)
        || properties.get(*entity).is_none_or(|p|p.hp<=Fixed64::ZERO)) {return;}
    if state.phase!=MobaMatchPhase::Playing || world.try_fetch::<GamePause>().is_some_and(|pause|pause.is_paused) {return;}
    let expires=state.elapsed+BRUSH_ATTACK_REVEAL_DURATION;
    state.brush_attack_reveals.insert((entity_key(attacker),target.team_id as u32),BrushAttackReveal {attacker,expires});
}

impl MobaMatch {
    pub(crate) fn brush_reveal_teams(&self, attacker:Entity) -> impl Iterator<Item=u32> + '_ {
        let key=entity_key(attacker);
        self.brush_attack_reveals.range((key,0)..=(key,u32::MAX))
            .filter(|(_,reveal)|reveal.expires>self.elapsed)
            .map(|((_,team),_)|*team)
    }
}

/// Authority-only protection signal. Basic attack windup is deliberate aggression;
/// damage-only signals never make area abilities trigger minion assistance.
pub(crate) fn record_moba_champion_aggression(world: &World, attacker: Entity, victim: Entity, minion_help: bool) {
    let Some(mut state) = world.try_fetch_mut::<MobaMatch>() else { return; };
    if state.phase != MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused { return; }
    let Some(source) = state.heroes.iter().find(|s| s.entity == Some(attacker) && !s.lethal_pending) else { return; };
    let Some(target) = state.heroes.iter().find(|s| s.entity == Some(victim) && !s.lethal_pending) else { return; };
    if source.side == target.side { return; }
    let defender_team = target.side as u8;
    let victim_player=u64::from(target.player_id);
    let Some(victim_position) = world.read_storage::<Pos>().get(victim).map(|p|p.0) else { return; };
    let properties = world.read_storage::<CProperty>();
    if [attacker, victim].iter().any(|e| properties.get(*e).is_none_or(|p| p.hp <= Fixed64::ZERO)) { return; }
    let key = (entity_key(attacker),victim_player);
    let expires = state.elapsed + Fixed64::from_raw(2560);
    // Damage from the same attack cannot erase its already admitted minion help.
    let minion_expires=if minion_help {expires} else {state.champion_aggression.get(&key)
        .filter(|a|a.victim==victim).map_or(Fixed64::ZERO,|a|a.minion_expires)};
    let minion_help=minion_expires>state.elapsed;
    state.champion_aggression.insert(key, ChampionAggression { attacker, victim, expires, minion_help, minion_expires, defender_team, victim_position });
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
    state.brush_attack_reveals.retain(|(source,_),_|*source!=entity_key(entity));
    for unit in state.units.values_mut() {unit.tower_focus.retire_target(entity);}
    jungle::death(world, &mut state, entity);
    tower_plating::retire(world,&state,entity);
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
                slot.input_scheme.neutralize();
                slot.recall = None;
                slot.deaths = slot.deaths.saturating_add(1);
                slot.kill_streak = 0;
                let retired_player = slot.player_id;
                super::shop::retire_owner_shop_undo(world, retired_player);
                slot.respawn_at = Some(state.elapsed + state.config.respawn_delay);
                objective_buffs::retire_hero(&mut state,retired_player);
                personal_buffs::retire_hero(&mut state,retired_player);
                world
                    .write_resource::<crate::runtime::BuffStore>()
                    .remove_all_for(entity);
                minion_aura::synchronize(world,&mut state);
            }
            LaneRole::Inhibitor | LaneRole::NexusTower => state.retire_base_structure(entity),
            LaneRole::Tower => {
                if !unit.last_hit_retired && state.phase==MobaMatchPhase::Playing
                    && !world.read_resource::<GamePause>().is_paused {
                    state.first_tower.observe(world.read_resource::<Tick>().0,entity_key(entity),None,BTreeSet::new());
                }
                for layer in &mut state.lane_tower_layers[unit.lane] {
                    if layer[team] == Some(entity) { layer[team] = None; }
                }
                state.lane_towers[unit.lane][team] = state.lane_tower_layers[unit.lane].iter().find_map(|layer| layer[team]);
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
pub(crate) fn capture_projectile_origin(world:&World,source:Entity)->Option<ProjectileOrigin> {
    if !world.entities().is_alive(source) {return None;}
    let state=world.try_fetch::<MobaMatch>();
    let credit=state.as_deref().and_then(|state|jungle_credit::resolve(world,state,Some(source)));
    Some(ProjectileOrigin {source,script_attack:false,attack_missed:false,normal_attack:false,attack_critical:false,attack_lifesteal:Fixed64::ZERO,launch_position:world.read_storage::<Pos>().get(source).map(|p|p.0),
        credit_team:credit.and_then(|c|state.as_deref().map(|state|state.config.teams[c.side])),
        credit_player:credit.and_then(|c|c.player), script:ProjectileScriptSource::capture(world,source)})
}

pub fn moba_damage_from_allowed(world:&World,source:Option<Entity>,target:Entity)->bool {
    moba_damage_allowed(world,target) && jungle::damage_allowed(world,source,target)
}

#[test]
fn compiled_base_policy_and_legacy_all_lane_policy_are_distinct() {
    let (_world,hero)=stop_command_world();
    let mut state=stop_command_match(hero,MobaMatchPhase::Playing);
    state.lane_tower_layers=vec![vec![[Some(hero),None]],vec![[Some(hero),Some(hero)]]];
    state.config.map_id=Some("three_lane_training".into());
    assert!(state.base_unlocked(1));assert!(!state.base_unlocked(0));assert!(!state.base_unlocked(2));
    state.config.map_id=None;
    assert!(!state.base_unlocked(1));
    state.lane_tower_layers[1][0][1]=None;
    assert!(state.base_unlocked(1));assert!(!state.base_unlocked(0));
}

pub fn moba_damage_allowed(world: &World, target: Entity) -> bool {
    if let Some(state) = world.try_fetch::<MobaMatch>() {
        // The live match wins over a disclosed gate. Filtered worlds without a
        // match continue into the gate below; legacy worlds with neither stay open.
        return state.current_match_target_attackable(target);
    }
    if let Some(gate) = world.try_fetch::<crate::runtime::filtered_specs::DisclosedAttackabilityGate>() {
        return gate.allows(target);
    }
    true
}

/// Synchronize settled buffs before script admission, including their expiry.
/// Both server and headless drivers use the shared script dispatcher. Never
/// invent a managed pool, apply to unrelated heroes, or mutate inactive matches.
pub fn refresh_moba_mana_capacities(world:&mut World) {
    let Some(state)=world.try_fetch::<MobaMatch>() else {return;};
    if !state.config.mana_enabled || state.phase!=MobaMatchPhase::Playing
        || state.pending_income_delta_raw<=0 || world.read_resource::<GamePause>().is_paused {return;}
    let properties=world.read_storage::<CProperty>();
    let buffs=world.read_resource::<crate::runtime::BuffStore>();
    let mut heroes=world.write_storage::<Hero>();
    for slot in &state.heroes {
        let Some(entity)=slot.entity.filter(|_|!slot.lethal_pending) else {continue;};
        if !properties.get(entity).is_some_and(|p|p.hp>Fixed64::ZERO) {continue;}
        if let Some(hero)=heroes.get_mut(entity) {refresh_managed_moba_capacity(hero,&buffs,entity);}
    }
}

fn refresh_managed_moba_capacity(hero:&mut Hero,buffs:&crate::runtime::BuffStore,entity:Entity) {
    let base=hero.moba_mana_capacity().expect("validated MOBA mana capacity");
    let maximum=crate::runtime::UnitStats::from_refs(buffs,false)
        .checked_mana_capacity(base,entity).unwrap_or(base);
    if let Some(pool)=&mut hero.mana_pool {
        pool.set_maximum(maximum).expect("checked MOBA mana capacity");
    }
}

/// Commit hook. Both destroyed bases in the same tick produce a draw.
pub fn finish_moba_match_tick(world: &mut World) {
    super::recovery_channel::finish_tick(world);
    let tick = world.read_resource::<Tick>().0;
    if world.try_fetch::<MobaMatch>().is_some() {
        super::cast_windup::synchronize_preparation(world);
        super::summoner_world::emit_committed_summoner_facts(world);
        emit_incoming_damage_observations(world, tick);
    }
    let Some(mut state) = world.try_fetch_mut::<MobaMatch>() else {
        return;
    };
    if state.phase != MobaMatchPhase::Playing {
        drop(state);
        jungle::synchronize_returning(world);
        emit_structure_observations(world,tick);
        return;
    }
    if !world.read_resource::<GamePause>().is_paused {
        for side in 0..state.heroes.len() {
            let team = state.heroes[side].side;
            let Some((origin, deadline)) = state.heroes[side].recall else { continue; };
            let valid = state.heroes[side].entity.filter(|entity| {
                world.read_storage::<CProperty>().get(*entity).is_some_and(|p| p.hp > Fixed64::ZERO)
                    && world.read_storage::<Pos>().get(*entity).is_some_and(|p| p.0 == origin)
                    // Check after outcomes, before testing the completion deadline.
                    // Damage-free control arriving on the final tick must still cancel.
                    && !recall_control_blocked(&world.read_resource::<crate::runtime::ability_runtime::BuffStore>(),*entity)
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
    first_tower::settle(world,&mut state);
    first_kill::settle(world,&mut state);
    objective_notice::settle(world,&mut state);
    objective_comeback::settle(world,&mut state);
    let delta = std::mem::take(&mut state.pending_income_delta_raw);
    // Public home geometry and living authority state; shared by HP and mana.
    // Compute once after combat/recall, never inside the owner-side planner.
    let mut home_recovery = HashSet::new();
    if state.config.base_recovery_enabled && delta > 0 {
        let positions=world.read_storage::<Pos>();
        let properties=world.read_storage::<CProperty>();
        let radius=Fixed64::from_i32(omoba_template_ids::MOBA_BASE_RECOVERY_RADIUS as i32);
        for slot in &state.heroes {
            let Some(entity)=slot.entity.filter(|_|!slot.lethal_pending) else {continue;};
            let Some(base)=state.bases[slot.side] else {continue;};
            if !properties.get(base).is_some_and(|p|p.hp>Fixed64::ZERO)
                || !properties.get(entity).is_some_and(|p|p.hp>Fixed64::ZERO) {continue;}
            let (Some(own),Some(home))=(positions.get(entity),positions.get(base)) else {continue;};
            if (own.0-home.0).length_squared()<=radius*radius {home_recovery.insert(entity);}
        }
    }
    if state.config.mana_enabled && delta > 0 {
        let properties = world.read_storage::<CProperty>();
        let buffs = world.read_resource::<crate::runtime::BuffStore>();
        let mut heroes = world.write_storage::<Hero>();
        let rate = Fixed64::from_i32(omoba_template_ids::MOBA_MANA_REGEN_PER_SECOND as i32);
        for slot in &state.heroes {
            let Some(entity) = slot.entity.filter(|_| !slot.lethal_pending) else { continue; };
            if !properties.get(entity).is_some_and(|p| p.hp > Fixed64::ZERO) { continue; }
            let Some(hero) = heroes.get_mut(entity) else { continue; };
            refresh_managed_moba_capacity(hero,&buffs,entity);
            if let Some(pool) = &mut hero.mana_pool {
                let base_rate=if home_recovery.contains(&entity) {
                    Fixed64::from_i32(omoba_template_ids::MOBA_BASE_RECOVERY_MANA_PER_SECOND as i32)
                } else {Fixed64::ZERO};
                // Combine rates before applying dt so one remainder carries
                // both contributions and a full pool cannot retain stale credit.
                let natural=crate::runtime::UnitStats::from_refs(&buffs,false).mana_regen(rate,entity);
                let combined=i128::from(natural.raw())+i128::from(base_rate.raw());
                // Buff overflow cannot panic or wrap into a giant refill.
                let combined=i64::try_from(combined).map(Fixed64::from_raw).unwrap_or(base_rate);
                pool.regenerate(combined, Fixed64::from_raw(delta)).expect("nonnegative active time and rate");
            }
        }
    }
    // Authoritative post-combat settlement, never executed by a Bot or replica.
    // Use the same active-time delta as income; warmup/pause cannot heal.
    if state.config.base_recovery_enabled && delta>0 {
        let mut properties=world.write_storage::<CProperty>();
        let amount=i128::from(delta)*i128::from(omoba_template_ids::MOBA_BASE_RECOVERY_HP_PER_SECOND);
        for slot in &state.heroes {
            let Some(entity)=slot.entity.filter(|entity|home_recovery.contains(entity)) else {continue;};
            let Some(health)=properties.get_mut(entity).filter(|p|p.hp>Fixed64::ZERO && p.mhp>Fixed64::ZERO) else {continue;};
            health.hp=Fixed64::from_raw((i128::from(health.hp.raw())+amount)
                .min(i128::from(health.mhp.raw())) as i64);
        }
    }
    let accrued = i128::from(state.income_remainder_raw) + i128::from(delta);
    let seconds = accrued / i128::from(omoba_sim::fixed::SCALE);
    state.income_remainder_raw = (accrued % i128::from(omoba_sim::fixed::SCALE)) as i64;
    let award = (seconds * i128::from(state.config.passive_gold_per_second))
        .min(i128::from(i32::MAX)) as i32;
    if award > 0 {
        for index in 0..state.heroes.len() {
            credit_moba_player(world,&mut state,index,award as u32,0);
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
                // Legacy matches must not receive unknown committed fact 26.
                if state.config.mana_enabled || progression.mana_pool.is_some() {
                    world.read_resource::<crate::runtime::ObservableFactBuffer>()
                        .emit(crate::runtime::OrderedFact {
                            key: crate::runtime::FactOrderingKey { tick,
                                phase: crate::runtime::FactPhase::PostStep, canonical_source_order: source,
                                local_ordinal: 0, fact_kind: crate::runtime::FactKind::CommittedMana },
                            audience: crate::runtime::FactAudience::VisibilityPolicy(
                                omb_script_abi::types::projection_policy_ids::HERO_ABILITY.to_owned()),
                            fact: crate::runtime::ObservableFact::CommittedMana { source,
                                state: crate::runtime::ability_runtime::CommittedManaState(progression.mana_pool.clone()) },
                        }).expect("valid disclosed mana state");
                }
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
        super::cast_windup::clear_preparation(world);
        objective_comeback::begin(world,&mut state);
        objective_notice::settle(world,&mut state);
        tower_plating::synchronize(world,&state,Fixed64::ZERO);
        objective_buffs::synchronize(world,&mut state,Fixed64::ZERO);
        personal_buffs::synchronize(world,&mut state,Fixed64::ZERO);
    }
    emit_owner_economy(world, &state, crate::runtime::FactPhase::PostStep);
    emit_owner_score(world, &state, crate::runtime::FactPhase::PostStep);
    emit_recall_state(world, &state, crate::runtime::FactPhase::PostStep);
    super::cast_windup::emit_progress(world,&state);
    if let Some(winner) = winner {
        state.phase = MobaMatchPhase::Finished { winner, tick };
        let winner_team = winner.map(|side| state.config.teams[side as usize]);
        let records = super::ward_world::take_ward_records(&mut state);
        drop(state);
        super::ward_world::destroy_ward_records(world, &records);
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
    } else {
        drop(state);
    }
    jungle::synchronize_returning(world);
    emit_structure_observations(world,tick);
}

/// Absolute, change-only public aggregate. Capture after gameplay, before the
/// fact barrier; visibility policy owns disclosure and hide/reveal handling.
#[derive(Default)]
struct IncomingDamageObservationCache(BTreeMap<u64, i64>);

#[derive(Default)]
struct StructureObservationCache(BTreeMap<u64,[u8;2]>);

fn emit_structure_observations(world:&mut World,tick:u64) {
    use crate::runtime::{FactAudience, FactKind, FactOrderingKey, FactPhase, ObservableFact, OrderedFact};
    if world.try_fetch::<StructureObservationCache>().is_none() {
        world.insert(StructureObservationCache::default());
    }
    let next=world.read_resource::<MobaMatch>().disclosed_structure_states();
    let mut cache=world.write_resource::<StructureObservationCache>();
    for (&source,&state) in &next {
        if cache.0.get(&source)==Some(&state) {continue;}
        world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(OrderedFact {
            key:FactOrderingKey {tick,phase:FactPhase::PostStep,canonical_source_order:source,
                local_ordinal:0,fact_kind:FactKind::CommittedStructure},
            audience:FactAudience::VisibilityPolicy(omb_script_abi::types::projection_policy_ids::HERO_ABILITY.to_owned()),
            fact:ObservableFact::CommittedStructure {source,state},
        }).expect("valid structure observation");
    }
    cache.0=next;
}

fn emit_incoming_damage_observations(world: &mut World, tick: u64) {
    use specs::Join;
    use crate::runtime::{FactAudience, FactKind, FactOrderingKey, FactPhase, ObservableFact, OrderedFact};
    if world.try_fetch::<IncomingDamageObservationCache>().is_none() {
        world.insert(IncomingDamageObservationCache::default());
    }
    let next: BTreeMap<_,_> = {
        let entities = world.entities();
        let properties = world.read_storage::<CProperty>();
        let positions = world.read_storage::<Pos>();
        let buffs = world.read_resource::<crate::runtime::BuffStore>();
        (&entities,&properties,&positions).join().map(|(entity,_,_)| {
            (crate::runtime::canonical_entity_id(entity),
                buffs.sum_add(entity,omb_script_abi::stat_keys::StatKey::DamageTakenBonus).raw())
        }).collect()
    };
    let mut cache = world.write_resource::<IncomingDamageObservationCache>();
    for (&source,&bonus_raw) in &next {
        if cache.0.get(&source)==Some(&bonus_raw) {continue;}
        world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(OrderedFact {
            key:FactOrderingKey {tick,phase:FactPhase::PostStep,canonical_source_order:source,
                local_ordinal:0,fact_kind:FactKind::CommittedIncomingDamage},
            audience:FactAudience::VisibilityPolicy(omb_script_abi::types::projection_policy_ids::HERO_ABILITY.to_owned()),
            fact:ObservableFact::CommittedIncomingDamage {source,bonus_raw},
        }).expect("valid incoming damage observation");
    }
    cache.0=next;
}

/// Canonical trace digest for the lane rules and their live ECS combat state.
/// This is a replay diagnostic, not the selective-replica wire hash.
pub fn single_lane_replay_digest(world: &World) -> String {
    use sha2::{Digest, Sha256};
    let state = world.read_resource::<MobaMatch>();
    let mut hash = Sha256::new();
    if let Some(channel)=super::recovery_channel::diagnostic(world) {hash.update(format!("|recovery-channel:{channel}"));}
    if let Some(windups)=world.try_fetch::<super::cast_windup::AbilityWindups>() {
        hash.update(format!("|cast-windups:{:?}|cast-view:{:?}",windups.0,windups.1));
    }
    hash.update(format!("|jungle:{:?}", state.jungle_camps));
    hash.update(format!("|objective-buffs:{:?}|personal-buffs:{:?}",state.objective_buffs,state.personal_buffs));
    hash.update(format!("|minion-auras:{:?}",state.minion_aura_targets));
    hash.update(format!("|tower-plating:{:?}:{:?}",state.config.tower_plating_rules,state.tower_plates));
    hash.update(format!("|base-structures:{:?}",state.base_structures));
    // The step envelope must keep the current wire tick even while no gameplay
    // can execute. It is not a running gameplay clock during pause/terminal.
    // Preserve the entire active snapshot (including controls and tick) in the
    // diagnostic; inactive envelope churn must not defeat the freeze check.
    if state.phase==MobaMatchPhase::Playing
        && !world.try_fetch::<GamePause>().is_some_and(|pause|pause.is_paused) {
        hash.update(format!("|input-step:{:?}", state.input_step));
    } else {
        hash.update(b"|input-step:inactive");
    }
    hash.update(format!("|input-resets:{:?}", state.input_reset_sources));
    hash.update(format!("|input-attack-resets:{:?}", state.input_attack_reset_sources));
    for camp in &state.jungle_camps {
        for entity in camp.entities() {
            hash_json(&mut hash, &serde_json::to_value((world.read_storage::<Pos>().get(entity),
                world.read_storage::<CProperty>().get(entity))).expect("jungle replay JSON"));
        }
    }
    hash.update(format!("|map:{:?}|routes:{:?}|towers:{:?}|layers:{:?}",state.config.map_id,state.routes,state.lane_towers,state.lane_tower_layers));
    hash.update(format!("|authored-minions:{}",state.config.authored_minion_waves));
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
        hash.update(format!("|assists:{}|kill-streak:{}|unanswered-deaths:{}", slot.assists,slot.kill_streak,slot.unanswered_deaths));
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
        // Roster identity and committed clocks. This diagnostic is not the
        // selective-replica wire hash and is not a hostile projection.
        hash.update(format!(
            "|summoner-owner:{}|summoner-revision:{}|summoner-slot0:{:?}|summoner-cd0:{}|summoner-slot1:{:?}|summoner-cd1:{}",
            slot.summoners.owner_player_id(),
            slot.summoners.revision(),
            slot.summoners.spell_id(0),
            slot.summoners.cooldown_remaining(0).map(|clock| clock.raw()).unwrap_or(0),
            slot.summoners.spell_id(1),
            slot.summoners.cooldown_remaining(1).map(|clock| clock.raw()).unwrap_or(0),
        ));
        super::ward_world::hash_roster_wards(&mut hash, world, slot);
        hash_json(&mut hash, &serde_json::to_value(slot.combat_preferences).expect("combat preferences JSON"));
        let mut input_material=Vec::new();slot.input_scheme.append_material(&mut input_material);hash.update(input_material);
    }
    hash.update(format!("|assist-rules:{}:{}|ledger:{:?}", state.config.hero_assist_gold,
        state.config.assist_window_seconds, state.assist_ledger));
    hash.update(format!("|champion-aggression:{:?}", state.champion_aggression));
    hash.update(format!("|brush-attack-reveals:{:?}", state.brush_attack_reveals));
    hash.update(format!("|tower-focus-rules:{:?}",state.config.tower_focus_rules));
    hash.update(format!("|building-reward-rules:{:?}|kill-bounty-rules:{:?}",state.config.building_reward_rules,state.config.kill_bounty_rules));
    hash.update(format!("|death-streak-rules:{:?}",state.config.death_streak_rules));
    hash.update(format!("|objective-notices:{:?}",state.objective_notices));
    hash.update(format!("|earned-gold:{:?}|comeback:{:?}|comeback-rules:{:?}",state.earned_gold,state.comeback,state.config.objective_comeback_rules));
    hash.update(format!("|first-tower:{:?}|first-kill:{:?}|first-kill-bonus:{}",state.first_tower,state.first_kill,state.config.first_kill_bonus_gold));
    hash.update(format!("|minion-growth-rules:{:?}",state.config.minion_growth_rules));
    hash.update(format!("|input-scheme-rules:{}:{}",state.config.input_scheme_rules.lock_after().raw(),state.config.input_scheme_rules.direction_lease().raw()));
    hash.update(format!("|xp-rules:{}:{}", state.config.hero_kill_xp, state.config.hero_assist_xp));
    hash.update(format!("|lane-xp-rules:{}:{}", state.config.lane_creep_xp, state.config.lane_xp_radius));
    hash.update(format!("|lane-gold-rules:{}", state.config.lane_creep_gold));
    let positions = world.read_storage::<Pos>();
    let properties = world.read_storage::<CProperty>();
    let heroes = world.read_storage::<Hero>();
    let commands = world.read_storage::<HeroCommandQueue>();
    let attacks = world.read_storage::<TAttack>();
    for (key, unit) in &state.units {
        hash.update(format!("|lane:{}|waypoint:{}",unit.lane,unit.next_waypoint));
        hash.update(format!("|last-hit-retired:{}",unit.last_hit_retired));
        hash.update(format!("|target:{:?}:{}",unit.target,unit.target_protected));
        hash.update(format!("|tower-focus:{:?}",unit.tower_focus));
        hash.update(format!("|attack-sequence:{}",unit.attack_sequence));
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
            hash.update(format!("|commit:{:?}", attack.committed_target.map(entity_key)));
        }
        if let Some(command) = commands.get(unit.entity) {
            hash.update(format!("|{:?}", command));
        }
    }
    // Summons are independent of the authored lane-unit registry. Their lifetime,
    // ownership, position and attack state all influence future gameplay.
    {
        use specs::Join;
        let summons=world.read_storage::<SummonedUnit>();
        let factions=world.read_storage::<Faction>();
        let owners=world.read_storage::<PlayerOwner>();
        for (entity,summon) in (&world.entities(),&summons).join() {
            hash.update(format!("|summon:{}",entity_key(entity)));
            hash_json(&mut hash,&serde_json::to_value((summon,positions.get(entity),properties.get(entity),
                attacks.get(entity),factions.get(entity),owners.get(entity),
                world.read_storage::<Unit>().get(entity),world.read_storage::<Facing>().get(entity),
                world.read_storage::<crate::runtime::scripting::ScriptUnitTag>().get(entity),
                world.read_storage::<MoveTarget>().get(entity))).expect("summon JSON"));
        }
    }
    // In-flight projectiles and queued deaths are future gameplay state too.
    // Excluding them could hide a divergence until the projectile hits.
    use specs::Join;
    let projectiles = world.read_storage::<Projectile>();
    for (entity, projectile, pos) in (&world.entities(), &projectiles, &positions).join() {
        hash.update(entity_key(entity).to_le_bytes());
        if let Some(origins)=world.try_fetch::<ProjectileOrigins>() {
            if let Some(origin)=origins.0.get(&entity) {
                hash_json(&mut hash,&serde_json::to_value(origin).expect("projectile origin JSON"));
            }
        }
        hash_json(
            &mut hash,
            &serde_json::to_value((projectile, pos)).expect("projectile JSON"),
        );
    }
    hash_json(
        &mut hash,
        &serde_json::to_value(&*world.read_resource::<Vec<Outcome>>()).expect("outcome JSON"),
    );
    // Absent and empty ledgers hash the same, so worlds that never buy stay stable.
    if let Some(ledger) = world.try_fetch::<super::shop::ShopUndoLedger>() {
        if !ledger.materially_empty() {
            hash.update(b"|shop-undo:");
            hash_json(&mut hash, &serde_json::to_value(&*ledger).expect("shop undo JSON"));
        }
    }
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
            .filter(|u| state.structure_unlocked(u))
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
                let destination=objective.and_then(|entity| positions.get(entity)).map_or(
                    position(&state.config,1 - team,Fixed64::ZERO), |p| p.0);
                if let Some(tracked)=objective.and_then(|entity|state.units.get(&entity_key(entity))) {
                    let route=&state.routes[tracked.lane][team];
                    let segment=route.windows(2).enumerate().min_by_key(|(_,points)| {
                        let delta=points[1]-points[0];let length=delta.length_squared();
                        let fraction=if length>Fixed64::ZERO {((destination-points[0]).dot(delta)/length)
                            .max(Fixed64::ZERO).min(Fixed64::ONE)} else {Fixed64::ZERO};
                        (destination-(points[0]+delta*fraction)).length_squared().raw()
                    }).map_or(0,|(index,_)|index);
                    let mut prefix=route[..=segment].to_vec();prefix.push(destination);
                    let active=commands.get(entity).and_then(|queue|match queue.active {
                        Some(HeroCommand::AttackMove {pos})=>Some(pos),_=>None});
                    bots::route_destination(&prefix,source,active).unwrap_or(destination)
                } else {destination}
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

#[cfg(test)]
pub(crate) fn stop_command_match(hero: Entity, phase: MobaMatchPhase) -> MobaMatch {
    MobaMatch {
        config: SingleLaneConfig::default(),
        phase,
        elapsed: Fixed64::ZERO,
        input_step: None,
        input_reset_sources: BTreeSet::new(),
        input_attack_reset_sources: BTreeSet::new(),
        earned_gold: [0;2],
        comeback: objective_comeback::ComebackState::default(),
        income_remainder_raw: 0,
        pending_income_delta_raw: 0,
        waves: 0,
        heroes: vec![MobaHeroSlot {
            player_id: 7,
            combat_preferences: crate::runtime::combat_controls::OwnerCombatPreferences::legacy(7).expect("fixture owner"),
            input_scheme: crate::runtime::input_scheme::OwnerInputScheme::default(),
            side: 0,
            entity: Some(hero),
            deaths: 0,
            kills: 0,
        kill_streak: 0,
        unanswered_deaths: 0,
            assists: 0,
            lethal_pending: false,
            recall: None,
            respawns: 0,
            respawn_at: None,
            hero: Hero::default(),
            inventory: Inventory::default(),
            gold: Gold(0),
            summoners: crate::runtime::summoner_runtime::SummonerLoadout::default(),
            trinket: super::ward_world::WardRosterState::unavailable(),
        }],
        towers: [None, None],
        lane_towers: Vec::new(),
        lane_tower_layers: Vec::new(),
        base_structures: Vec::new(),
        routes: Vec::new(),
        bases: [None, None],
        jungle_camps: Vec::new(),
        assist_ledger: super::moba_assist::AssistLedger::default(),
        first_tower:first_tower::FirstTowerState::default(),
        first_kill:first_kill::FirstKillState::default(),
        objective_notices:objective_notice::ObjectiveNotices::default(),
        objective_buffs:BTreeMap::new(),
        tower_plates:BTreeMap::new(),
        minion_aura_targets:BTreeMap::new(),
        personal_buffs:BTreeMap::new(),
        champion_aggression: BTreeMap::new(),
        brush_attack_reveals: BTreeMap::new(),
        next_wave_at: Fixed64::ZERO,
        units: BTreeMap::new(),
        summoner_catalog: None,
        ward_policies: BTreeMap::new(),
    }
}

#[cfg(test)]
pub(crate) fn communication_roster_match(
    heroes: Vec<MobaHeroSlot>,
    phase: MobaMatchPhase,
    elapsed: Fixed64,
    map_id: Option<&str>,
) -> MobaMatch {
    let mut config = SingleLaneConfig::default();
    config.map_id = map_id.map(str::to_string);
    let mut jungle_camps = Vec::new();
    if let Some(id) = map_id {
        if let Some(map) = omoba_template_ids::moba_map_by_name(id) {
            config.lane_length = Fixed64::from_i32(map.lane_length);
            jungle_camps = map.jungle_camps.iter().copied().map(MobaJungleCamp::new).collect();
        }
    }
    MobaMatch {
        config,
        phase,
        elapsed,
        input_step: None,
        input_reset_sources: BTreeSet::new(),
        input_attack_reset_sources: BTreeSet::new(),
        earned_gold: [0;2],
        comeback: objective_comeback::ComebackState::default(),
        income_remainder_raw: 0,
        pending_income_delta_raw: 0,
        waves: 0,
        heroes,
        towers: [None, None],
        lane_towers: Vec::new(),
        lane_tower_layers: Vec::new(),
        base_structures: Vec::new(),
        routes: Vec::new(),
        bases: [None, None],
        jungle_camps,
        assist_ledger: super::moba_assist::AssistLedger::default(),
        first_tower:first_tower::FirstTowerState::default(),
        first_kill:first_kill::FirstKillState::default(),
        objective_notices:objective_notice::ObjectiveNotices::default(),
        objective_buffs:BTreeMap::new(),
        tower_plates:BTreeMap::new(),
        minion_aura_targets:BTreeMap::new(),
        personal_buffs:BTreeMap::new(),
        champion_aggression: BTreeMap::new(),
        brush_attack_reveals: BTreeMap::new(),
        next_wave_at: Fixed64::ZERO,
        units: BTreeMap::new(),
        summoner_catalog: None,
        ward_policies: BTreeMap::new(),
    }
}

#[cfg(test)]
pub(crate) fn shop_fixture_slot(player_id: u32, side: usize, entity: Option<Entity>) -> MobaHeroSlot {
    MobaHeroSlot {
        player_id,
        combat_preferences: crate::runtime::combat_controls::OwnerCombatPreferences::legacy(player_id).expect("fixture roster owner"),
        input_scheme: crate::runtime::input_scheme::OwnerInputScheme::default(),
        side,
        entity,
        deaths: 0,
        kills: 0,
        kill_streak: 0,
        unanswered_deaths: 0,
        assists: 0,
        lethal_pending: false,
        recall: None,
        respawns: 0,
        respawn_at: None,
        hero: Hero::default(),
        inventory: Inventory::default(),
        gold: Gold(0),
        summoners: crate::runtime::summoner_runtime::SummonerLoadout::default(),
        trinket: super::ward_world::WardRosterState::unavailable(),
    }
}

#[cfg(test)]
impl MobaMatch {
    pub(crate) fn track_lane_unit_for_attackability_test(
        &mut self,
        entity: Entity,
        team: u8,
        role: LaneRole,
        lane: usize,
    ) {
        self.units.insert(entity_key(entity), LaneUnit {
            entity,
            team,
            role,
            attack_remaining: Fixed64::ZERO,
            lane,
            next_waypoint: 0,
            last_hit_retired: false,
            target: None,
            target_protected: false,
            tower_focus: Default::default(),
            attack_sequence: 0,
        });
    }

    pub(crate) fn track_jungle_for_attackability_test(&mut self, entity: Entity, returning: bool) {
        let mut camp = MobaJungleCamp::new(omoba_template_ids::MobaJungleConst {
                objective_buff:None,
                personal_buff:None,
                additional_objective_buffs:&[],
                death_children:&[],members:&[],patience_seconds:0,soft_reset_seconds:0,
                id: "current_match_attackability",
                position: (0, 0),
                hp: 1,
                damage: 0,
                move_speed: 0,
                attack_range: 0,
                leash_radius: 0,
                attack_interval_seconds: 1,
                respawn_seconds: 1,
                gold: 0,
                xp: 0,
        });
        camp.entity = Some(entity);
        camp.returning = returning;
        self.jungle_camps.push(camp);
    }
}

#[cfg(test)]
pub(crate) fn stop_command_world() -> (World, Entity) {
    let mut world = World::new();
    world.register::<Hero>();
    world.register::<PlayerOwner>();
    world.register::<CProperty>();
    world.register::<MoveTarget>();
    world.register::<HeroCommandQueue>();
    world.register::<TAttack>();
    world.register::<Pos>();
    world.insert(PendingMoveQueue::default());
    world.insert(Tick(0));
    world.insert(GamePause { is_paused: false });
    let hero = world.create_entity()
        .with(Hero::default())
        .with(PlayerOwner::new(7))
        .with(CProperty {
            hp: Fixed64::from_i32(100),
            mhp: Fixed64::from_i32(100),
            msd: Fixed64::ONE,
            def_physic: Fixed64::ZERO,
            def_magic: Fixed64::ZERO,
        })
        .with(Pos(Vec2::new(Fixed64::from_i32(3), Fixed64::from_i32(4))))
        .with(MoveTarget(Vec2::new(Fixed64::from_i32(9), Fixed64::ZERO)))
        .with(HeroCommandQueue {
            active: Some(HeroCommand::HoldPosition),
            queued: vec![HeroCommand::MoveTo { pos: Vec2::new(Fixed64::ONE, Fixed64::ZERO) }],
        })
        .build();
    (world, hero)
}

#[cfg(test)]
fn stop_command_still_holding(world: &World, hero: Entity) -> bool {
    world.read_storage::<HeroCommandQueue>().get(hero).is_some_and(|queue| {
        queue.active == Some(HeroCommand::HoldPosition) && queue.queued.len() == 1
    }) && world.read_storage::<MoveTarget>().get(hero).is_some()
}

#[cfg(test)]
fn push_stop(world: &mut World) {
    world.write_resource::<PendingMoveQueue>().requests.push(PendingHeroCommand {
        owner_pid: 7,
        queued: false,
        kind: PendingHeroCommandKind::Stop,
    });
}

#[cfg(test)]
fn stop_command_active(world: &World, hero: Entity) -> Option<HeroCommand> {
    world.read_storage::<HeroCommandQueue>().get(hero).and_then(|queue| queue.active)
}

#[cfg(test)]
#[test]
fn stop_command_rejects_inactive_phase_pause_and_foreign_hero() {
    let (mut world, hero) = stop_command_world();
    world.insert(stop_command_match(hero, MobaMatchPhase::Finished { winner: None, tick: 4 }));
    push_stop(&mut world);
    super::game_processor::drain_pending_moves(&mut world);
    assert!(stop_command_still_holding(&world, hero));
    world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Warmup;
    push_stop(&mut world);
    super::game_processor::drain_pending_moves(&mut world);
    assert!(stop_command_still_holding(&world, hero));
    world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
    world.write_resource::<GamePause>().is_paused = true;
    world.write_resource::<PendingMoveQueue>().requests.push(PendingHeroCommand {
        owner_pid: 7,
        queued: false,
        kind: PendingHeroCommandKind::HoldPosition,
    });
    super::game_processor::drain_pending_moves(&mut world);
    assert_eq!(stop_command_active(&world, hero), Some(HeroCommand::HoldPosition));
    assert!(world.read_storage::<HeroCommandQueue>().get(hero).is_some_and(|queue| queue.queued.is_empty()));
    assert!(world.read_storage::<MoveTarget>().get(hero).is_none());
    assert!(crate::runtime::command_claims_movement_priority(false, stop_command_active(&world, hero)));
    push_stop(&mut world);
    super::game_processor::drain_pending_moves(&mut world);
    assert_eq!(stop_command_active(&world, hero), Some(HeroCommand::HoldPosition));
    world.write_resource::<GamePause>().is_paused = false;
    let decoy = world.create_entity().build();
    {
        let mut state = world.write_resource::<MobaMatch>();
        state.heroes[0].entity = Some(decoy);
    }
    push_stop(&mut world);
    super::game_processor::drain_pending_moves(&mut world);
    assert_eq!(stop_command_active(&world, hero), Some(HeroCommand::HoldPosition));
    {
        let mut state = world.write_resource::<MobaMatch>();
        state.heroes[0].entity = Some(hero);
    }
    push_stop(&mut world);
    super::game_processor::drain_pending_moves(&mut world);
    assert_eq!(stop_command_active(&world, hero), None);
    assert!(world.read_storage::<MoveTarget>().get(hero).is_none());
    assert!(!crate::runtime::command_claims_movement_priority(false, None));
}

#[cfg(all(test, feature = "kcp"))]
#[test]
fn stop_command_interrupts_recall_and_blocks_same_batch_start() {
    use crate::runtime::{PlayerInput, PlayerInputEnum};
    let stop = || (7, PlayerInput { action: Some(PlayerInputEnum::Stop(crate::game_proto::Stop {})) });
    let recall = || (7, PlayerInput { action: Some(PlayerInputEnum::Recall(crate::game_proto::Recall {})) });
    let prepare = || {
        let (mut world, hero) = stop_command_world();
        world.insert(PendingPlayerInputs::default());
        world.insert(crate::runtime::ability_runtime::BuffStore::default());
        let mut state = stop_command_match(hero, MobaMatchPhase::Playing);
        state.bases[0] = Some(hero);
        assert!(state.allows_player_input(7, &stop().1));
        assert!(!state.allows_player_input(8, &stop().1));
        assert!(single_lane_input_action_allowed(&stop().1));
        (world, state, hero)
    };
    {
        let (mut world, mut state, hero) = prepare();
        world.write_resource::<PendingPlayerInputs>().inputs = vec![recall()];
        apply_recall_commands(&mut world, &mut state, Fixed64::ZERO, &BTreeSet::new());
        assert!(state.heroes[0].recall.is_some());
        assert_eq!(state.disclosed_channel_states()[&crate::runtime::canonical_entity_id(hero)],crate::runtime::channel_visual_state::ChannelVisualState::Recall);
        assert!(world.read_resource::<PendingPlayerInputs>().inputs.is_empty());
        assert!(world.read_storage::<HeroCommandQueue>().get(hero).is_some_and(|queue| queue.active.is_none()));
        assert!(world.read_storage::<MoveTarget>().get(hero).is_none());
        // Exercise the authority producer, not just a handcrafted replica baseline.
        world.register::<Faction>();
        world.register::<crate::runtime::ReplicationScope>();
        world.register::<crate::runtime::StealthProfile>();
        world.register::<crate::runtime::VisibilityOverride>();
        world.register::<crate::runtime::RememberPolicy>();
        world.register::<crate::runtime::VisionSource>();
        world.register::<Unit>();
        world.register::<crate::runtime::DemoPatrol>();
        world.register::<Facing>();
        world.register::<TurnSpeed>();
        world.register::<CollisionRadius>();
        world.register::<Inventory>();
        world.register::<Gold>();
        world.register::<ItemEffects>();
        world.register::<Tower>();
        world.register::<crate::scripting::ScriptUnitTag>();
        world.register::<crate::runtime::PlacedWard>();
        world.insert(crate::runtime::VisionOccluderSet::default());
        world.insert(state);
        let disclosed = |world: &World| {
            let view = crate::runtime::build_wave_b_read_view(world, 0);
            let row = view.entities.iter().find(|row| row.canonical_id == crate::runtime::canonical_entity_id(hero)).unwrap();
            let components = crate::runtime::decode_disclosed_components(&row.disclosed_baseline).unwrap();
            crate::runtime::channel_visual_state::ChannelVisualState::decode(
                components[&crate::runtime::channel_visual_state::SCHEMA_ID],
            ).unwrap()
        };
        assert_eq!(disclosed(&world), crate::runtime::channel_visual_state::ChannelVisualState::Recall);
        let mut state = world.remove::<MobaMatch>().unwrap();
        world.write_resource::<PendingPlayerInputs>().inputs = vec![stop()];
        apply_recall_commands(&mut world, &mut state, Fixed64::ZERO, &BTreeSet::new());
        world.insert(state);
        assert_eq!(disclosed(&world), crate::runtime::channel_visual_state::ChannelVisualState::None);
    }
    {
        let (mut world, mut state, hero) = prepare();
        state.heroes[0].recall = Some((Vec2::new(Fixed64::from_i32(3), Fixed64::from_i32(4)), Fixed64::from_i32(50)));
        world.write_resource::<PendingPlayerInputs>().inputs = vec![stop()];
        apply_recall_commands(&mut world, &mut state, Fixed64::ZERO, &BTreeSet::new());
        assert!(state.heroes[0].recall.is_none());
        assert_eq!(state.disclosed_channel_states()[&crate::runtime::canonical_entity_id(hero)],crate::runtime::channel_visual_state::ChannelVisualState::None);
        assert!(matches!(
            world.read_resource::<PendingPlayerInputs>().inputs[0].1.action,
            Some(PlayerInputEnum::Stop(_))
        ));
        assert!(stop_command_still_holding(&world, hero));
        push_stop(&mut world);
        world.insert(state);
        super::game_processor::drain_pending_moves(&mut world);
        assert!(world.read_storage::<HeroCommandQueue>().get(hero).is_some_and(|queue| {
            queue.active.is_none() && queue.queued.is_empty()
        }));
    }
    {
        let (mut world, mut state, hero) = prepare();
        world.write_resource::<PendingPlayerInputs>().inputs = vec![recall(), stop()];
        apply_recall_commands(&mut world, &mut state, Fixed64::ZERO, &BTreeSet::new());
        assert!(state.heroes[0].recall.is_none());
        let inputs = &world.read_resource::<PendingPlayerInputs>().inputs;
        assert_eq!(inputs.len(), 1);
        assert!(matches!(inputs[0].1.action, Some(PlayerInputEnum::Stop(_))));
        assert!(stop_command_still_holding(&world, hero));
    }
}

#[cfg(test)]
#[test]
fn disclosed_attackability_gate_never_overrides_authority_match() {
    use crate::runtime::filtered_specs::{DisclosedAttackabilityGate, DisclosedAttackabilityRecord};
    use specs::{Builder, WorldExt};
    let mut world = World::new();
    let target = world.create_entity().build();
    assert!(moba_damage_allowed(&world, target), "no match and no gate stays open");
    let mut gate = DisclosedAttackabilityGate::default();
    gate.record(target, DisclosedAttackabilityRecord::Governed { admitted: false });
    world.insert(gate);
    assert!(!moba_damage_allowed(&world, target));
    world.insert(stop_command_match(target, MobaMatchPhase::Playing));
    assert!(moba_damage_allowed(&world, target), "a playing authority match ignores the gate");
}
