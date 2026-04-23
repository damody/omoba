//! 英雄技能腳本 — 從 `omb/ability-system/src/heroes/` 搬來，
//! 改為 `AbilityScript` sabi_trait 實作（透過 `GameWorldDyn` 直接套用效果）。
//!
//! 數值採「buff 表驅動」策略：
//! - Handler 呼叫 `world.add_buff(target, buff_id, duration)` 套用具名 buff
//! - 每個 buff_id 對應的屬性加成由 host 端 `BuffDefRegistry` 登記
//!   （來源：`AbilityDef.effects_preview` 的 `EffectSpec::Buff`）
//! - Toggle 技能第二次施放時 `world.has_buff` + `remove_buff` 處理切換

pub mod B01_saika_magoichi;
pub mod B02_date_masamune;
