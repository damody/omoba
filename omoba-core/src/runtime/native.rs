//! Native implementation of the shared deterministic runtime.
//!
//! Concrete ECS runtime modules are migrated here from `omb` in the following
//! tasks. Keeping this file present establishes the mandatory module boundary
//! before moving the larger gameplay code.

pub mod ability_runtime;
pub mod comp;
pub mod events;
pub mod game_processor;
pub mod geometry;
pub mod initialization;
pub mod input;
pub mod item;
pub mod scene;
pub mod snapshot;
pub mod scripting;
pub mod spatial;
pub mod system_dispatcher;

pub use ability_runtime::{armor_to_mult, AbilityRegistry, BuffEntry, BuffStore, UnitStats};
pub use comp::*;
pub use events::{
    RuntimeBroadcast, RuntimeEvent, RuntimeEventSink, RuntimeEventVecSink, RuntimeEvents,
};
pub use game_processor::{
    drain_pending_ability_casts, drain_pending_ability_upgrades, drain_pending_item_uses,
    drain_pending_moves, drain_pending_tower_sells, drain_pending_tower_spawns,
    drain_pending_tower_upgrades, handle_ability_cast_from_input,
    handle_ability_upgrade_from_input, handle_item_use_from_input, handle_tower_sell_from_input,
    handle_tower_spawn_from_input, handle_tower_upgrade_from_input, spawn_td_tower,
    interrupt_attack_for_accepted_command, process_outcomes,
};
pub use initialization::{
    create_world_for_scene, populate_ability_registry, populate_tower_template_registry,
    populate_tower_upgrade_registry, StateInitializer,
};
pub use input::*;
pub use item::{sell_price, ActiveEffect, ItemBonus, ItemConfig, ItemRegistry};
pub use scene::*;
pub use snapshot::*;
pub use scripting::{
    run_script_dispatch, ScriptEvent, ScriptEventQueue, ScriptRegistry, ScriptUnitTag, SkillTarget,
};
pub use spatial::{Bounds, Entry, SpatialIndex, SpatialIndexParams};
pub use system_dispatcher::{build_phase3_dispatcher, SystemDispatcher};
