pub use crate::runtime::comp::*;
pub use crate::runtime::comp::{
    blocked_region, bounty, building, check_point, circular_vision, collision_index, creep,
    creep_move_broadcast, damage, facing, fx_queues, game_mode, gold, heightmap, hero, inventory,
    is_base, item_effects, last, lockstep_resources, phys, projectile, resources, tower,
    tower_registry, tower_upgrade_registry, tower_upgrade_rules, unit,
};

pub mod base {
    #[cfg(not(feature = "tracy"))]
    macro_rules! span {
        ($guard_name:tt, $level:ident, $name:expr, $($fields:tt)*) => {
            let span = tracing::span!(tracing::Level::$level, $name, $($fields)*);
            let $guard_name = span.enter();
        };
        ($guard_name:tt, $level:ident, $name:expr) => {
            let span = tracing::span!(tracing::Level::$level, $name);
            let $guard_name = span.enter();
        };
        ($guard_name:tt, $name:expr) => {
            let span = tracing::span!(tracing::Level::TRACE, $name);
            let $guard_name = span.enter();
        };
        ($guard_name:tt, $no_tracy_name:expr, $tracy_name:expr) => {
            $crate::span!($guard_name, $no_tracy_name);
        };
    }
    pub(crate) use span;

    #[cfg(not(feature = "tracy"))]
    macro_rules! prof_span {
        ($guard_name:tt, $name:expr) => {
            let $guard_name = $crate::comp::base::ProfSpan;
        };
        ($name:expr) => {};
    }
    pub(crate) use prof_span;

    pub struct ProfSpan;
}

pub use base::ProfSpan;
pub(crate) use base::{prof_span, span};

#[path = "../../../omb/src/comp/ecs.rs"]
pub mod ecs;
#[path = "../../../omb/src/comp/tick_profile.rs"]
pub mod tick_profile;
#[path = "../../../omb/src/comp/enemy.rs"]
pub mod enemy;
#[path = "../../../omb/src/comp/campaign.rs"]
pub mod campaign;
#[path = "../../../omb/src/comp/player.rs"]
pub mod player;

pub mod outcome {
    pub use crate::runtime::comp::outcome::*;
    pub use crate::runtime::comp::{
        AttackCancelFx, AttackCancelFxQueue, AttackCancelPhase, AttackPhaseFx, AttackPhaseFxQueue,
        DisIndex, ExplosionFx, ExplosionFxQueue, RemovedEntitiesQueue, Searcher, TowerFireFx,
        TowerFireFxQueue,
    };

    pub fn searcher_from_config() -> Searcher {
        Searcher::default()
    }
}

pub mod tower_template {
    pub use crate::runtime::game_processor::spawn_td_tower;
}

pub use campaign::*;
pub use ecs::*;
pub use enemy::*;
pub use outcome::*;
pub use player::*;
pub use tick_profile::{Phase as TickPhase, TickProfile};
