//! Shared script runtime data used by deterministic gameplay systems.

pub mod dispatch;
pub mod event;
pub mod parallel_world_adapter;
pub mod registry;
pub mod tag;

pub use dispatch::run_script_dispatch;
pub use event::{
    ScriptEvent, ScriptEventQueue, ScriptVisualEvent, ScriptVisualEventKind,
    ScriptVisualEventQueue, SkillTarget,
};
pub use parallel_world_adapter::{ParallelAdapterCache, ParallelWorldAdapter};
pub use registry::ScriptRegistry;
pub use tag::ScriptUnitTag;
