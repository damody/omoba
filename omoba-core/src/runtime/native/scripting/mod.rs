//! Shared script runtime data used by deterministic gameplay systems.

pub mod dispatch;
pub mod event;
pub mod parallel_world_adapter;
pub mod registry;
pub mod tag;

pub use dispatch::{drain_pending_tower_ability_callbacks, run_script_dispatch};
pub use event::{
    script_visual_event_to_observable_fact, ScriptEvent, ScriptEventQueue, ScriptVisualEvent,
    ScriptVisualEventKind, ScriptVisualEventQueue, SkillTarget,
};
pub use parallel_world_adapter::{ParallelAdapterCache, ParallelWorldAdapter};
pub use registry::ScriptRegistry;
pub use tag::ScriptUnitTag;
