//! Shared script runtime data used by deterministic gameplay systems.

pub mod dispatch;
pub mod event;
pub mod registry;
pub mod tag;
pub mod world_adapter;

pub use dispatch::run_script_dispatch;
pub use event::{ScriptEvent, ScriptEventQueue, SkillTarget};
pub use registry::ScriptRegistry;
pub use tag::ScriptUnitTag;
pub use world_adapter::WorldAdapter;
