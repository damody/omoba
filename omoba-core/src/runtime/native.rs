//! Native implementation of the shared deterministic runtime.
//!
//! Concrete ECS runtime modules are migrated here from `omb` in the following
//! tasks. Keeping this file present establishes the mandatory module boundary
//! before moving the larger gameplay code.

pub mod ability_runtime;
pub mod ability_channel;
pub mod ally_shield;
pub mod attack_orb;
pub mod attack_stun;
pub mod summon_owner_cap;
pub mod cooldown_refund;
pub mod cooldown_cap;
pub mod hit_stack;
pub mod tether;
pub mod forced_motion;
pub mod reveal_zone;
pub mod tree_cut;
pub mod spell_amplify;
pub mod rules;
pub mod delayed_effect;
pub mod unit_var;
pub mod scheduled_effect;
pub mod status_event;
pub mod hero_command;
pub mod ability_projectile;
pub mod ally_area_buff;
pub mod damage_immunity;
pub mod spell_guard;
pub mod ally_area_restore;
pub mod item_modifiers;
pub mod aura_emitter;
pub mod magic_immunity;
pub mod dispel;
pub mod blink_landing;
pub mod ally_cleanse;
pub mod mana_drain;
pub mod script_gold;
pub mod mana_burn;
pub mod damage_trace;
pub mod taunt;
pub mod command_restriction;
pub(crate) mod targeted_effect;
pub(crate) mod attack_accuracy;
pub(crate) mod cast_windup;
pub(crate) mod cast_approach;
pub(crate) mod tick_start_hp;
pub(crate) mod recovery_channel;
pub mod owner_control;
pub use cast_windup::{CAST_SLOT,CAST_STATE,CAST_REMAINING,CAST_TOTAL,CAST_INSTANCE};
pub mod combat_acquisition;
pub mod command_outcome;
pub mod comp;
pub mod events;
pub mod economy_projection;
pub mod game_processor;
pub mod attributes;
pub mod retaliation;
pub mod gameplay_phases;
pub mod geometry;
pub mod initialization;
pub mod input;
pub mod item;
pub mod shop;
pub mod moba_match;
pub mod summoner_world;
pub(crate) mod summon_navigation;
pub(crate) mod summon_orders;
pub mod ward_world;
pub mod moba_assist;
pub mod scene;
pub mod scripting;
#[cfg(feature = "kcp")]
pub mod simulation_driver;
pub mod snapshot;
pub mod spatial;
pub mod system_dispatcher;
#[cfg(feature = "kcp")]
pub mod td_autoplay;
pub mod td_economy;
pub mod td_layer;
pub mod team_replica_rng;
pub mod tick;

pub use ability_runtime::{armor_to_mult, AbilityRegistry, BuffEntry, BuffStore, UnitStats};
pub use comp::*;
pub use events::{
    RuntimeBroadcast, RuntimeEvent, RuntimeEventSink, RuntimeEventVecSink, RuntimeEvents,
};
pub use summoner_world::{
    drain_pending_summoner_casts, project_team_summoners, summoner_projection_metadata,
    validate_server_loadouts,
};
pub use ward_world::{
    apply_configured_trinket_loadouts, copied_ward_roster, drain_pending_ward_placements,
    install_ward_admission_epochs, project_private_owner_ward, retire_owner_wards,
    route_admitted_ward_placement, set_authoritative_trinket_loadout, settle_ward_placement,
    trusted_actor_generation, validate_trinket_loadouts, WardPlacementIdentity, WardPlacementReceipt,
    WardPlacementRequest, WardReject, WardRosterState, WARD_LOCAL_SLOT,
};
#[cfg(any(test, feature = "test-support"))]
pub use ward_world::override_retained_ward_combat_policy;
pub use game_processor::{
    drain_pending_ability_casts, drain_pending_ability_upgrades, drain_pending_hero_command_clears,
    bind_queued_item_lives, canonical_item_owner, clear_untrusted_item_life, drain_pending_item_uses,
    drain_pending_moves, overwrite_item_owner_canonical, pending_item_use_from_command,
    seal_item_use_owner, write_local_item_actor, drain_pending_tower_ability_casts,
    drain_pending_tower_sells, drain_pending_tower_spawns, drain_pending_tower_target_priorities,
    drain_pending_tower_upgrades, handle_ability_cast_from_input,
    handle_ability_upgrade_from_input, handle_item_use_from_input,
    handle_tower_ability_cast_from_input, handle_tower_sell_from_input,
    handle_tower_spawn_from_input, handle_tower_target_priority_from_input,
    handle_tower_upgrade_from_input, hero_knowledge_category_for_unit_id,
    interrupt_attack_for_accepted_command, process_outcomes, spawn_td_tower,
};
pub use gameplay_phases::*;
pub use initialization::{
    create_world_for_scene, create_world_for_scene_with_content, create_world_from_loaded_content,
    populate_ability_registry, populate_tower_template_registry, populate_tower_upgrade_registry,
    OwnedEcsPool, StateInitializer,
};
pub use input::*;
pub use moba_match::*;
pub use item::{
    apply_item_stats, sell_price, ActiveEffect, ItemBonus, ItemConfig, ItemRegistry,
    ItemStatAggregate, ItemStatApplication,
};
pub use scene::*;
pub use scripting::{
    drain_pending_tower_ability_callbacks, run_script_dispatch, ScriptEvent, ScriptEventQueue,
    ScriptRegistry, ScriptUnitTag, ScriptVisualEvent, ScriptVisualEventKind,
    ScriptVisualEventQueue, SkillTarget,
};
#[cfg(feature = "kcp")]
pub use simulation_driver::{SimulationDriver, SimulationTickProfile, SimulationTickResult};
pub use snapshot::*;
pub use spatial::{Bounds, Entry, SpatialIndex, SpatialIndexParams};
pub use system_dispatcher::{build_phase3_dispatcher, SystemDispatcher};
#[cfg(feature = "kcp")]
pub use td_autoplay::*;
pub use td_economy::*;
pub use td_layer::*;
pub use team_replica_rng::*;
pub use tick::tick_tower_abilities;
