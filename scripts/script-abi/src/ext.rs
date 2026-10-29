//! `GameWorld::ext`: one general extension point (M3', work line B). `GameWorld` is a
//! `#[sabi_trait]` and holds at most 64 methods (E235); new host reads and requests go through
//! `ext(op, args)` instead of new trait methods. The host dispatches on `op`; an unknown op or
//! wrong arguments return `RErr`, so a DLL built against a newer op set fails loudly, never
//! silently. Ops only read the world or emit closed outcomes; none keeps state in the DLL.
use crate::types::{EntityHandle, Fixed64, Vec2};
use abi_stable::{std_types::RString, StableAbi};

#[repr(C)]
#[derive(StableAbi, Clone, Debug, PartialEq)]
pub enum ExtArg {
    Unit(EntityHandle),
    Num(Fixed64),
    Point(Vec2),
    Str(RString),
    Bool(bool),
}

#[repr(C)]
#[derive(StableAbi, Clone, Debug, PartialEq)]
pub enum ExtValue {
    None,
    Unit(EntityHandle),
    Num(Fixed64),
    Point(Vec2),
    Str(RString),
    Bool(bool),
}

/// Operation ids. Never renumber; append.
pub mod op {
    /// `() -> Bool`: whether the world records a damage trace (test worlds with `DamageTrace`).
    pub const DAMAGE_TRACE_ENABLED: u32 = 1;
    /// `(Str ability, Str path) -> None`: tags the next damage this script deals (trace only).
    pub const DAMAGE_TAG: u32 = 2;
    /// `(Unit, Str ability) -> Num`: the unit's rank of that ability (0 when not learned).
    pub const ABILITY_LEVEL: u32 = 3;
}
