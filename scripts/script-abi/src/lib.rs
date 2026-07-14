//! omb-script-abi — omb 主機和腳本 DLL 之間的穩定 ABI 合約。
//!
//! 這個箱子是主機和所有腳本 cdylib 的**唯一**東西
//! 依賴。它必須僅使用 `abi_stable` 類型；沒有規格，沒有 omb main
//! 板條箱，沒有任何東西可以將引擎內部結構拉過 FFI 邊界。

#![allow(non_camel_case_types, non_local_definitions)]

pub mod ability;
pub mod buff_ids;
pub mod manifest;
pub mod script;
pub mod stat_keys;
pub mod types;
pub mod world;

// 重新匯出範本 ID 新類型，以便腳本可以 `use omb_script_abi::prelude::*;`
// 並引用`TOWER_TACK`，`TPL_HERO_SAIKA_MAGOICHI`等，無需添加
// 每個腳本箱中都有一個單獨的依賴項。
pub use omoba_template_ids::{
    AbilityId, BuffId, CreepId, HeroId, ProjectileKindId, SummonId, TowerId,
};

pub mod prelude {
    pub use crate::ability::{AbilityDefFFI, AbilityScript, AbilityScript_TO};
    pub use crate::script::{UnitScript, UnitScript_TO};
    pub use crate::stat_keys;
    pub use crate::types::*;
    pub use crate::world::ProjectileQuery;
    pub use crate::world::{
        GameWorld, GameWorldDyn, GameWorld_TO, TowerActiveAbilityAccess,
        TowerActiveAbilityAccessDyn, TowerActiveAbilityAccess_TO, TowerCooldownAccess,
        TowerCooldownAccessDyn, TowerCooldownAccess_TO,
    };
    pub use abi_stable::{
        rstr,
        sabi_trait::prelude::*,
        std_types::{RBox, RNone, ROption, RSome, RStr, RString, RVec},
    };

    // 讓腳本可以方便建這些常用型別不用自己 import
    pub use crate::types::{
        Angle, DamageInfo, DamageKind, EntityHandle, Fixed64, PathSpec, ProjectileHitContext,
        ProjectileSpec, Target, TowerMetadata, Vec2,
    };

    // 模板 ids — 由 omoba-template-ids build.rs 產生的 newtype + const。
    // 透過 `use omb_script_abi::prelude::*;` 和引用導入腳本
    // TOWER_TACK / HERO_SAIKA_MAGOICHI / ABILITY_SNIPER_MODE / BUFF_STUN 等
    pub use omoba_template_ids::*;
}
