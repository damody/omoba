//! Debug module
//!
//! Provides debug visualization overlays and entity inspection tools
//! for the OMOBA Fyrox debug frontend.

pub mod inspector;
pub mod overlays;

pub use inspector::{EntityInspector, EntityProperty, InspectedEntity, PropertyValue};
pub use overlays::{CollisionBox, DebugOverlays, EntityLabel, MovementPath, OverlayType, VisionRange};
