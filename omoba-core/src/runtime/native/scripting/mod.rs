//! Shared script runtime data used by deterministic gameplay systems.

pub mod event;
pub mod tag;

pub use event::{ScriptEvent, ScriptEventQueue, SkillTarget};
pub use tag::ScriptUnitTag;
