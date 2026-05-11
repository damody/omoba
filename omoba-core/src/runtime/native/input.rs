//! Shared lockstep input protocol types used by runtime, backend, and native frontend.

pub use crate::game_proto::{
    AngleI, AttackTarget, CastAbility, FixedI, InputForPlayer, InputSubmit, ItemUse, MoveTo, NoOp,
    PlayerInput, StartRound, TickBatch, TowerPlace, TowerSell, TowerUpgradeInput, UpgradeAbility,
    Vec2I,
};

pub use crate::game_proto::player_input::Action as PlayerInputEnum;
