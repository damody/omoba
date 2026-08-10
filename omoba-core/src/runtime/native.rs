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
pub mod scripting;
pub mod snapshot;
pub mod spatial;
pub mod system_dispatcher;
pub mod tick;

pub use ability_runtime::{armor_to_mult, AbilityRegistry, BuffEntry, BuffStore, UnitStats};
pub use comp::*;
pub use events::{
    RuntimeBroadcast, RuntimeEvent, RuntimeEventSink, RuntimeEventVecSink, RuntimeEvents,
};
pub use game_processor::{
    drain_pending_ability_casts, drain_pending_ability_upgrades, drain_pending_hero_command_clears,
    drain_pending_item_uses, drain_pending_moves, drain_pending_tower_ability_casts,
    drain_pending_tower_sells, drain_pending_tower_spawns, drain_pending_tower_target_priorities,
    drain_pending_tower_upgrades, handle_ability_cast_from_input,
    handle_ability_upgrade_from_input, handle_item_use_from_input,
    handle_tower_ability_cast_from_input, handle_tower_sell_from_input,
    hero_knowledge_category_for_unit_id,
    handle_tower_spawn_from_input, handle_tower_target_priority_from_input,
    handle_tower_upgrade_from_input, interrupt_attack_for_accepted_command, process_outcomes,
    spawn_td_tower,
};
pub use initialization::{
    create_world_for_scene, create_world_for_scene_with_content, create_world_from_loaded_content,
    populate_ability_registry, populate_tower_template_registry, populate_tower_upgrade_registry,
    StateInitializer,
};
pub use input::*;
pub use item::{sell_price, ActiveEffect, ItemBonus, ItemConfig, ItemRegistry};
pub use scene::*;
pub use scripting::{
    drain_pending_tower_ability_callbacks, run_script_dispatch, ScriptEvent, ScriptEventQueue,
    ScriptRegistry, ScriptUnitTag, ScriptVisualEvent, ScriptVisualEventKind,
    ScriptVisualEventQueue, SkillTarget,
};
pub use snapshot::*;
pub use spatial::{Bounds, Entry, SpatialIndex, SpatialIndexParams};
pub use system_dispatcher::{build_phase3_dispatcher, SystemDispatcher};
pub use tick::tick_tower_abilities;
