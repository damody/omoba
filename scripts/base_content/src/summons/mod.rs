//! 召喚物 UnitScript 實作 — 由英雄技能呼叫 `spawn_summoned_unit` 時
//! 透過 `ScriptUnitTag` 綁定對應 unit_id，dispatch tick 每幀呼叫 on_tick
//! 驅動攻擊/移動邏輯。
//!
//! 目前僅有 saika_gunner（雜賀鐵炮兵）；未來擴充其他召喚物同樣放這裡。

pub mod saika_gunner;

pub use saika_gunner::SaikaGunner;
