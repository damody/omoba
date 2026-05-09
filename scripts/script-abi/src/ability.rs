//! `AbilityScript` — 每個 DLL 實作的 sabi_trait 以提供可轉換的
//! 能力（主動技能、大招、切換、塔攻擊）。
//!
//! `UnitScript` 的伴侶：
//! - `UnitScript` 對單元生命週期事件（生成、死亡、命中）做出反應。
//! - 施展特定能力時會呼叫「AbilityScript」。
//!
//! ## 元資料與邏輯
//! 穩定的 ABI 將“AbilityDef”作為 JSON 編碼的“RString”
//! （`AbilityDefFFI::def_json`）。這避免了“StableAbi”的派生
//! `HashMap<String, serde_json::Value>` 並保持 ABI 簡單。主持人
//! 一次將`omoba_core::ability_meta::AbilityDef`反序列化為`omoba_core::ability_meta::AbilityDef`
//! 元資料查詢的 DLL 載入時間（例如客戶端工具提示）。
//!
//! 傳遞給 `execute` 的 `level_data_json` 是 JSON 編碼的
//! 施法者目前等級的 `AbilityLevelData` — 也避免了
//! StableAbi 在 `extra: HashMap<String, Value>` 欄位上。

use crate::types::{EntityHandle, Fixed64, Target};
use crate::world::GameWorldDyn;
use abi_stable::{
    sabi_trait,
    std_types::{RBox, RResult, RStr, RString},
    StableAbi,
};

#[sabi_trait]
pub trait AbilityScript: Send + Sync {
    /// 能力標識符（必須與同伴中的“id”字段匹配
    /// `能力定義`）。由主機用來調度。
    fn ability_id(&self) -> RStr<'_>;

    /// 執行能力。處理程序透過“world”方法應用效果
    /// (`deal_damage`, `add_buff`, `spawn_projectile`, ...) 直接而非
    /// 而不是傳回效果清單 — 反映了「UnitScript」模式。
    ///
    /// `level_data_json` 是 `omoba_core::ability_meta::AbilityLevelData`
    /// 序列化為 JSON。處理程序在進入時反序列化以讀取
    /// `冷卻時間`、`法力消耗`、`範圍`、`額外[...]`等。
    ///
    /// 失敗時返回「RErr(msg)」（呼叫者日誌）；樓主還在
    /// 只有當處理程序選擇時，才會扣除「RErr」的冷卻時間/費用。
    #[sabi(last_prefix_field)]
    fn execute(
        &self,
        caster: EntityHandle,
        target: Target,
        level: u8,
        level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString>;

    /// 在至少產生一個活動效果時呼叫每個主機滴答聲
    /// 靠這個能力才活著。 `elapsed` = 自該能力生效以來的秒數
    /// was cast.預設為無操作（大多數能力都是「即發即棄」）。
    fn on_tick(
        &self,
        _caster: EntityHandle,
        _target: Target,
        _elapsed: Fixed64,
        _world: &mut GameWorldDyn<'_>,
    ) {
    }

    /// 技能被學會（或升等）時觸發。
    /// Passive 技用這個把永久 buff 套上；Active / Toggle / Ultimate 預設忽略。
    fn on_learn(&self, _caster: EntityHandle, _new_level: u8, _world: &mut GameWorldDyn<'_>) {}

    /// 攻擊者命中目標時觸發 — host 會對 attacker 已學的每個 passive ability 輪詢此 hook。
    /// Passive 技能（如雜賀雨鐵砲）用此實作「普攻附帶的額外效果」。
    /// 預設 no-op。
    fn on_attack_hit(
        &self,
        _owner: EntityHandle,
        _attacker: EntityHandle,
        _victim: EntityHandle,
        _level: u8,
        _world: &mut GameWorldDyn<'_>,
    ) {
    }
}

/// `AbilityDef` + 實作它的腳本 — 每種能力一個條目
/// 在 DLL 中。主機註冊表在載入時建立一個「HashMap<id，AbilityDefFFI>」。
///
/// `def_json` 是 `omoba_core::ability_meta::AbilityDef` 序列化的
/// serde_json。將其保留為字串可以避免拖曳 serde-json /
/// 以 `abi_stable::StableAbi` 進行 HashMap。
#[repr(C)]
#[derive(StableAbi)]
pub struct AbilityDefFFI {
    pub def_json: RString,
    pub script: AbilityScript_TO<'static, RBox<()>>,
}
