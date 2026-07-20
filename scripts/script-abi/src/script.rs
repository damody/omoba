//! `UnitScript` — 每個單位類型（英雄/塔/小兵）實現的 sabi_trait。
//! 所有鉤子都有預設的無操作實作；腳本僅覆蓋它們需要的內容。
//!
//! Hooks 命名對應 Dota 2 MODIFIER_EVENT_*：`on_attack_start` / `on_attack_landed`
//! / `on_attacked` / `on_health_gained` / `on_mana_gained` / `on_spent_mana`
//! / `on_heal_received` / `on_state_changed` / `on_modifier_added` /
//! `on_modifier_removed`。所有 hook 皆為 no-op default，腳本只覆寫需要的。

use crate::types::{
    DamageInfo, EntityHandle, Fixed64, ProjectileHitContext, Target, TowerMetadata,
};
use crate::world::{
    GameWorldDyn, ProjectileQueryDyn, TowerActiveAbilityAccessDyn, TowerCooldownAccessDyn,
};
use abi_stable::{
    sabi_trait,
    std_types::{RNone, ROption, RStr},
};

#[sabi_trait]
pub trait UnitScript: Send + Sync {
    /// 主機用於調度的單元標識符（必須與“script”欄位匹配
    /// 在設備的配置條目中）。
    fn unit_id(&self) -> RStr<'_>;

    /// 當實體生成時呼叫一次。
    #[sabi(last_prefix_field)]
    fn on_spawn(&self, _e: EntityHandle, _w: &mut GameWorldDyn<'_>) {}

    /// 使用“ScriptUnitTag”呼叫實體的每個刻度。腳本使用這個
    /// 驅動主動行為（例如塔：找到目標→生成彈頭）。
    /// `dt` 是以秒為單位的刻度增量。
    fn on_tick(&self, _e: EntityHandle, _dt: Fixed64, _w: &mut GameWorldDyn<'_>) {}

    /// 塔的靜態 metadata（atk/asd/range/bullet_speed/...）。
    /// host 在 startup 時 iter registry 收集，連同 host 端的 cost/footprint/label
    /// 組成完整 template 廣播給前端（下拉選單成本顯示 + placement 預覽 range）。
    /// 回 `RNone` 表示「這不是 TD 塔」（英雄/敵人 creep 等）。
    fn tower_metadata(&self) -> ROption<TowerMetadata> {
        RNone
    }

    /// 當實體死亡時呼叫。 `killer` = 已知的殺戮實體。
    fn on_death(
        &self,
        _e: EntityHandle,
        _killer: ROption<EntityHandle>,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// 在造成傷害之前呼叫受害者。腳本可能會發生變化
    /// `info.amount` 實現護盾/傷害減少/反射。
    fn on_damage_taken(&self, _e: EntityHandle, _info: &mut DamageInfo, _w: &mut GameWorldDyn<'_>) {
    }

    /// 在“on_damage_taken”解決問題後呼叫攻擊者
    /// 最終金額。對於吸血、擊中效果很有用。
    fn on_damage_dealt(
        &self,
        _attacker: EntityHandle,
        _victim: EntityHandle,
        _final_amount: Fixed64,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// 當技能啟動時對施法者進行召喚。
    fn on_skill_cast(
        &self,
        _caster: EntityHandle,
        _skill_id: RStr<'_>,
        _target: Target,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// 在攻擊連線時呼叫攻擊者。
    /// 塔式腳本通常存在於此（飛濺、刺穿、暴擊）。
    fn on_attack_hit(
        &self,
        _attacker: EntityHandle,
        _victim: EntityHandle,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    // ============================================================
    // Dota 2 MODIFIER_EVENT_* 對應 hooks
    // ============================================================

    /// 對應 `MODIFIER_EVENT_ON_ATTACK_START`：攻擊動作準備發射（pre-cast）。
    /// 腳本可在此選擇 target（orb 技能），修改即將出擊的屬性。
    fn on_attack_start(
        &self,
        _attacker: EntityHandle,
        _target: ROption<EntityHandle>,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// 對應 `MODIFIER_EVENT_ON_ATTACK_LANDED`：攻擊實際命中（在 `on_attack_hit`
    /// 之後由 host 派發，做為一個更通用的 hook 點，含未命中/格擋資訊）。
    fn on_attack_landed(
        &self,
        _attacker: EntityHandle,
        _victim: EntityHandle,
        _damage: Fixed64,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// 對應 `MODIFIER_EVENT_ON_ATTACK_FAIL`：攻擊失誤（evasion / miss）。
    fn on_attack_fail(
        &self,
        _attacker: EntityHandle,
        _victim: EntityHandle,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// 對應 `MODIFIER_EVENT_ON_ATTACKED`：本 unit 被攻擊（命中或未命中皆派發）。
    /// 與 `on_damage_taken` 區別：這裡在解析 pre-damage 前就觸發，
    /// 適合做計數器類行為（被攻擊 N 次 → 觸發護盾）。
    fn on_attacked(
        &self,
        _victim: EntityHandle,
        _attacker: EntityHandle,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// 對應 `MODIFIER_EVENT_ON_HEALTH_GAINED`：HP 淨增加（heal 或 regen）。
    fn on_health_gained(&self, _e: EntityHandle, _amount: Fixed64, _w: &mut GameWorldDyn<'_>) {}

    /// 對應 `MODIFIER_EVENT_ON_MANA_GAINED`：MP 淨增加。
    fn on_mana_gained(&self, _e: EntityHandle, _amount: Fixed64, _w: &mut GameWorldDyn<'_>) {}

    /// 對應 `MODIFIER_EVENT_ON_SPENT_MANA`：腳本釋放技能花費 mana 後。
    fn on_spent_mana(
        &self,
        _caster: EntityHandle,
        _cost: Fixed64,
        _ability_id: RStr<'_>,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// 對應 `MODIFIER_EVENT_ON_HEAL_RECEIVED`：回復量被計算完（含 heal_received_multiplier）。
    fn on_heal_received(
        &self,
        _target: EntityHandle,
        _amount: Fixed64,
        _source: ROption<EntityHandle>,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// 對應 `MODIFIER_EVENT_ON_STATE_CHANGED`：單位狀態改變（stun / silence /
    /// root / invisible / invulnerable 等）。`state_id` 為狀態 id 字串；
    /// `active=true` 代表剛進入，`false` 代表剛離開。
    fn on_state_changed(
        &self,
        _e: EntityHandle,
        _state_id: RStr<'_>,
        _active: bool,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// 對應 `MODIFIER_EVENT_ON_MODIFIER_ADDED`：身上新增 buff/modifier。
    fn on_modifier_added(
        &self,
        _e: EntityHandle,
        _modifier_id: RStr<'_>,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// 對應 `MODIFIER_EVENT_ON_MODIFIER_REMOVED`：身上 buff/modifier 過期或被移除。
    fn on_modifier_removed(
        &self,
        _e: EntityHandle,
        _modifier_id: RStr<'_>,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// 對應 `MODIFIER_EVENT_ON_ORDER`：玩家下達命令（move / attack / cast 等）。
    /// `order_kind` 為命令類型字串（"move" / "attack" / "cast" / "stop" / "hold"）；
    /// `target` 為命令對象。
    fn on_order(
        &self,
        _e: EntityHandle,
        _order_kind: RStr<'_>,
        _target: Target,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// 對應 `MODIFIER_EVENT_ON_RESPAWN`：英雄復活完成。
    fn on_respawn(&self, _e: EntityHandle, _w: &mut GameWorldDyn<'_>) {}

    /// Projectile-only impact hook. Generic melee/non-projectile hits continue to use
    /// `on_attack_hit`; tower projectile chains use this provenance-aware hook.
    fn on_projectile_hit(
        &self,
        _attacker: EntityHandle,
        _victim: EntityHandle,
        _context: ProjectileHitContext,
        _query: &ProjectileQueryDyn<'_>,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// Tower tick extension with deterministic internal-cooldown access.
    /// The default preserves every existing script's `on_tick` implementation.
    fn on_tower_tick(
        &self,
        e: EntityHandle,
        dt: Fixed64,
        _cooldowns: &mut TowerCooldownAccessDyn<'_>,
        w: &mut GameWorldDyn<'_>,
    ) {
        self.on_tick(e, dt, w);
    }

    /// Called once after a tower active cast is accepted.
    fn on_tower_ability_activate(
        &self,
        _tower: EntityHandle,
        _ability_id: RStr<'_>,
        _w: &mut GameWorldDyn<'_>,
    ) {
    }

    /// Called once for each scheduled pulse. Returning true consumes a charge.
    fn on_tower_ability_pulse(
        &self,
        _tower: EntityHandle,
        _ability_id: RStr<'_>,
        _pulse_index: u16,
        _w: &mut GameWorldDyn<'_>,
    ) -> bool {
        true
    }

    /// Extension-aware activation hook. Existing scripts only implementing the
    /// exact public hook above continue to work through this default delegate.
    fn on_tower_ability_activate_with_access(
        &self,
        tower: EntityHandle,
        ability_id: RStr<'_>,
        _access: &TowerActiveAbilityAccessDyn<'_>,
        w: &mut GameWorldDyn<'_>,
    ) {
        self.on_tower_ability_activate(tower, ability_id, w);
    }

    /// Extension-aware pulse hook with the same compatibility delegation.
    fn on_tower_ability_pulse_with_access(
        &self,
        tower: EntityHandle,
        ability_id: RStr<'_>,
        pulse_index: u16,
        _access: &TowerActiveAbilityAccessDyn<'_>,
        w: &mut GameWorldDyn<'_>,
    ) -> bool {
        self.on_tower_ability_pulse(tower, ability_id, pulse_index, w)
    }
}
