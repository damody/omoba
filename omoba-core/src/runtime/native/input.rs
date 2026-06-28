//! Shared lockstep input protocol types used by runtime, backend, and native frontend.

pub use crate::game_proto::{
    AngleI, AttackMove, AttackTarget, CastAbility, FixedI, InputForPlayer, InputSubmit, ItemUse,
    MoveTo, NoOp, PlayerInput, SetTowerTargetPriority, StartRound, TargetPriority, TickBatch,
    ToggleGameSpeed, TogglePause, TowerPlace, TowerSell, TowerUpgradeInput, UpgradeAbility, Vec2I,
};

pub use crate::game_proto::player_input::Action as PlayerInputEnum;
