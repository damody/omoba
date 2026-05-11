//! Shared script runtime data used by deterministic gameplay systems.

#[path = "../../../../../omb/src/scripting/dispatch.rs"]
pub mod dispatch;
pub mod event;
#[path = "../../../../../omb/src/scripting/registry.rs"]
pub mod registry;
pub mod tag;
#[path = "../../../../../omb/src/scripting/world_adapter.rs"]
pub mod world_adapter;

pub use dispatch::run_script_dispatch;
pub use event::{ScriptEvent, ScriptEventQueue, SkillTarget};
pub use registry::ScriptRegistry;
pub use tag::ScriptUnitTag;
pub use world_adapter::WorldAdapter;
