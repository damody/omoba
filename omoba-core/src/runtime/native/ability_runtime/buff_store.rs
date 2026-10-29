//! `BuffStore` — host 端統一的 buff 儲存與倒數系統。
//!
//! 取代原本散在多處的 buff 實作（`SlowBuff` component + `slow_buff_tick`）。
//! 所有來自 DLL 腳本的 `world.add_buff` / `world.add_stat_buff` 最終都寫
//! 到這個 resource；每 tick 由 `tick::buff_tick` 倒數，過期自動移除。
//!
//! 每筆 buff 可攜帶 `payload: serde_json::Value`，讓 host 系統（例如
//! `creep_tick` 的移速計算）從 buff 身上讀出數值（如 slow factor）。
//!
//! ## 保留的有效負載約定
//!
//! - `slow_factor: f64` — 觸發 add() 的「強蓋弱」語意：同 buff_id 已存在
//!   且雙方 payload 都帶 `slow_factor` 時，較小者勝出（refresh duration 不變、
//!   payload 只在新 factor 較強時才覆寫）。任一邊缺欄位或型別不對 → fallback
//!   到舊行為（直接覆寫）。新增其他需要「比較合併」的 stat 時，請在這裡記載。
//! - `__aggregation_family: string` — 同一 family 的不同 source buff 對每個 stat
//!   只採絕對值最強者；乘法 stat 採離 1.0 最遠者。Family 依名稱排序聚合，
//!   保持 fixed-point 計算順序確定。
//!   例外：`generic_ability_slow` 的 `move_speed_bonus` 照原作緩速規則
//!   （dummy.lua:816-902）——同一來源名稱只算一筆，不同來源以 1−Π(1−sᵢ)
//!   乘法疊加（見 [`combine_ability_slows`]）；同 family 的攻速常數減速仍取最強。

use omb_script_abi::buff_ids::BuffId;
use omb_script_abi::stat_keys::StatKey;
use omoba_sim::Fixed64;
use serde_json::Value;
use specs::Entity;
use std::collections::{BTreeMap, HashMap};

const AGGREGATION_FAMILY_KEY: &str = "__aggregation_family";
/// Ability slows (原作 `dummy.lua` `Slow`／`slow_self_passive`).
pub const ABILITY_SLOW_FAMILY: &str = "generic_ability_slow";
const ABILITY_MOVE_SLOW_KEY: &str = "move_speed_bonus";

/// Source name of an ability slow: the ability of a `generic_slow:<ability>:..`
/// key (原作 `target.ms_slow[keys.name]` 以技能名為鍵), else the buff id itself.
fn ability_slow_source(name: &str) -> &str {
    omb_script_abi::buff_ids::declarative_buff_source(name)
        .filter(|(_, stat)| stat.is_none()).map_or(name, |(ability, _)| ability)
}

/// One move-speed slow of the ability-slow family. The same source name keeps
/// one value (原作同名覆蓋; the strongest here, independent of HashMap order).
fn note_ability_slow<'a>(slows: &mut BTreeMap<&'a str, i64>, name: &'a str, raw: i64) {
    // A slow is a reduction in 0..=1; positive or malformed values do not slow.
    let slow = raw.saturating_neg().clamp(0, 1024);
    slows.entry(ability_slow_source(name)).and_modify(|old| *old = (*old).max(slow)).or_insert(slow);
}

/// 原作 `slow_self_passive`（dummy.lua:866-902）：不同來源的移速減速乘法疊加，
/// total = 1 − Π(1 − sᵢ)。Returned as the raw (negative) `move_speed_bonus`.
/// Sources fold in name order with truncating Q10 products, so every host
/// computes the same value.
pub fn combine_ability_slows(slows: &BTreeMap<&str, i64>) -> i64 {
    let remaining = slows.values().fold(1024i128, |left, slow| left * i128::from(1024 - slow) / 1024);
    -(1024 - remaining as i64)
}

/// Host-local item effect identity; consumers do not parse private source keys.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ItemTimedModifier { Sprint, DamageReduction }

impl ItemTimedModifier {
    fn family(self) -> &'static str {
        match self { Self::Sprint => "item_sprint", Self::DamageReduction => "item_damage_reduce" }
    }
    fn stat(self) -> StatKey {
        match self { Self::Sprint => StatKey::MoveSpeedBonusBuff, Self::DamageReduction => StatKey::DamageTakenBonus }
    }
    fn maximum(self) -> Fixed64 {
        match self { Self::Sprint => Fixed64::from_i32(10_000), Self::DamageReduction => Fixed64::ONE }
    }
}

/// 從 JSON 負載中讀取固定 64 的數字統計值。
///
/// 階段 1de.2 線路編碼：原始 `Fixed64::raw()` i32 儲存為 JSON 數字
/// （整數）。刪除 f64 → `* 1024 as i32` 量化，最多遺失
/// 14 位元精確度和有風險的平台發散 IEEE-754 乘法。
///
/// 向後相容：如果 JSON 數字是浮點數（舊腳本有效負載
/// 尚未遷移到 `.raw()` 發射），回退到舊的
/// 量化。一旦每個腳本編寫者都使用“.raw()”，我們就可以刪除它
/// 分支（第 2 階段）。
#[inline]
pub(crate) fn read_fixed_from_payload(value: &serde_json::Value) -> Fixed64 {
    if let Some(i) = value.as_i64() {
        // PHASE 1de.2 鎖步正確形式：原始固定 64 整數。
        Fixed64::from_raw(i as i64)
    } else if let Some(f) = value.as_f64() {
        // 階段 2 遺留：f64 → 固定 64 量化。已棄用；全部刪除一次
        // 腳本有效負載編寫器發出“.raw()”整數。
        Fixed64::from_raw((f * 1024.0) as i64)
    } else {
        Fixed64::ZERO
    }
}

#[derive(Debug, Clone)]
pub struct BuffEntry {
    pub remaining: Fixed64,
    pub payload: Value,
    /// Fraction of one raw damage quantum; owned by this buff lifetime.
    periodic_remainder: i64,
    zone_clock: i64,
}

/// Canonical persistent lifetime used by stat buffs and snapshots. Explicit removal still applies.
pub const PERMANENT_BUFF_DURATION: Fixed64 = Fixed64::from_raw(i64::MAX);

/// 以 `Entity -> buff_id` 為 key 的 O(1) buff 索引。
/// `entities_by_key` 是 stat key → entity → 引用計數的反向索引，
/// 加速「哪些 entity 受某類 stat 影響」的查詢（regen / DoT 系統用）。
#[derive(Default, Debug)]
pub struct BuffStore {
    buffs: HashMap<Entity, HashMap<String, BuffEntry>>,
    entities_by_key: HashMap<String, HashMap<Entity, u32>>,
    independent_zones:BTreeMap<(Entity,String),super::damage_zone::IndependentDamageZone>,
    next_zone_lease:u64,
    /// Rows marked for status events (status_event::KEY) that left their holder, with how
    /// (status_event EXPIRED / REMOVED / CLEARED), in the order they left. Drained by the script
    /// dispatch; transient, not part of the replicated state.
    status_ended: Vec<(Entity, String, Value, u8)>,
}

#[derive(Debug, Clone, Copy)]
pub struct MoveSpeedSums {
    pub absolute: Fixed64,
    pub base_override: Fixed64,
    pub bonus_equipment: Fixed64,
    pub percentage: Fixed64,
    pub bonus_buff: Fixed64,
    pub absolute_min: Fixed64,
    pub max: Fixed64,
    pub limit: Fixed64,
}

impl Default for MoveSpeedSums {
    fn default() -> Self {
        Self {
            absolute: Fixed64::ZERO,
            base_override: Fixed64::ZERO,
            bonus_equipment: Fixed64::ZERO,
            percentage: Fixed64::ZERO,
            bonus_buff: Fixed64::ZERO,
            absolute_min: Fixed64::ZERO,
            max: Fixed64::ZERO,
            limit: Fixed64::ZERO,
        }
    }
}

/// State buff id: the carrier is not an obstacle for other units' movement.
pub const NO_UNIT_COLLISION: &str = "no_unit_collision";

impl BuffStore {
    /// Canonical time-only disclosure for the closed shared modifier families.
    /// Malformed reserved names/families cannot fall back to catalog-name lookup.
    pub(crate) fn timed_modifier_visual(name: &str, entry: &BuffEntry) -> Result<Option<&'static str>, ()> {
        let family = entry.payload.get(AGGREGATION_FAMILY_KEY).and_then(Value::as_str);
        let kind = match family {
            Some("item_sprint") => Some(ItemTimedModifier::Sprint),
            Some("item_damage_reduce") => Some(ItemTimedModifier::DamageReduction),
            _ => None,
        };
        let reserved = matches!(name, "movement_boost" | "damage_reduction");
        let Some(kind) = kind else { return if reserved { Err(()) } else { Ok(None) }; };
        let id = match kind { ItemTimedModifier::Sprint => "movement_boost", ItemTimedModifier::DamageReduction => "damage_reduction" };
        if reserved && name != id { return Err(()); }
        if entry.remaining <= Fixed64::ZERO || entry.remaining > Fixed64::from_i32(60)
            || entry.payload.as_object().is_none_or(|object| object.len() != 2) { return Err(()); }
        let raw = entry.payload.get(kind.stat().as_str()).and_then(Value::as_i64).ok_or(())?;
        let valid = match kind {
            ItemTimedModifier::Sprint => (1..=kind.maximum().raw()).contains(&raw),
            ItemTimedModifier::DamageReduction => (-kind.maximum().raw()..=-1).contains(&raw),
        };
        if valid { Ok(Some(id)) } else { Err(()) }
    }

    /// Validates before mutation and preserves the existing strongest family.
    pub fn grant_item_timed_modifier(&mut self, entity: Entity, item_id: &str,
        kind: ItemTimedModifier, magnitude: Fixed64, duration: Fixed64) -> bool
    {
        self.grant_timed_modifier(entity,item_id,kind,magnitude,duration)
    }

    /// Shared source grant; historical family identities remain stable for snapshots.
    pub fn grant_timed_modifier(&mut self, entity: Entity, source: &str,
        kind: ItemTimedModifier, magnitude: Fixed64, duration: Fixed64) -> bool
    {
        if magnitude <= Fixed64::ZERO || magnitude > kind.maximum()
            || duration <= Fixed64::ZERO || duration > Fixed64::from_i32(60) { return false; }
        let value = match kind { ItemTimedModifier::Sprint => magnitude, ItemTimedModifier::DamageReduction => -magnitude };
        self.add(entity, &format!("{}:{}", kind.family(), source), duration,
            serde_json::json!({kind.stat().as_str(): value.raw(), AGGREGATION_FAMILY_KEY: kind.family()}));
        true
    }

    /// Presence of an effective family member, not its source spelling or net
    /// stat after unrelated effects. Entries remain active until tick removes
    /// them, matching the aggregation lifecycle (including pending zero time).
    pub fn has_item_timed_modifier(&self, entity: Entity, kind: ItemTimedModifier) -> bool {
        self.iter_for(entity).any(|(_, entry)| {
            if entry.payload.get(AGGREGATION_FAMILY_KEY).and_then(Value::as_str) != Some(kind.family()) { return false; }
            let Some(value) = entry.payload.get(kind.stat().as_str()) else { return false; };
            let raw = read_fixed_from_payload(value).raw();
            match kind {
                ItemTimedModifier::Sprint => (1..=kind.maximum().raw()).contains(&raw),
                ItemTimedModifier::DamageReduction => (-kind.maximum().raw()..=-1).contains(&raw),
            }
        })
    }

    pub fn shield_remaining(&self, entity: Entity) -> Fixed64 {
        self.get(entity, "damage_shield")
            .and_then(|entry| entry.payload.get("shield_remaining_raw"))
            .and_then(Value::as_i64).map(Fixed64::from_raw)
            .filter(|amount| *amount > Fixed64::ZERO && *amount <= Fixed64::from_i32(1_000_000))
            .unwrap_or(Fixed64::ZERO)
    }

    pub fn grant_healing_reduction(&mut self, entity:Entity, multiplier:Fixed64, duration:Fixed64)->bool {
        if multiplier<Fixed64::ZERO || multiplier>=Fixed64::ONE
            || duration<=Fixed64::ZERO || duration>Fixed64::from_i32(60) {return false;}
        let current=self.get(entity,"healing_reduction").filter(|b|b.remaining>Fixed64::ZERO && b.remaining<=Fixed64::from_i32(60));
        let factor=current.and_then(|b|b.payload.get("heal_received_multiplier")).and_then(Value::as_i64)
            .filter(|raw|(0..1024).contains(raw)).map_or(multiplier,|raw|multiplier.min(Fixed64::from_raw(raw)));
        let duration=current.map_or(duration,|b|duration.max(b.remaining));
        self.add(entity,"healing_reduction",duration,serde_json::json!({"heal_received_multiplier":factor.raw(),"__aggregation_family":"healing_reduction"}));
        true
    }

    pub fn grant_damage_shield(&mut self, entity: Entity, amount: Fixed64, duration: Fixed64) -> bool {
        if amount <= Fixed64::ZERO || amount > Fixed64::from_i32(1_000_000)
            || duration <= Fixed64::ZERO || duration > Fixed64::from_i32(60) { return false; }
        self.add(entity, "damage_shield", duration,
            serde_json::json!({ "shield_remaining_raw": amount.max(self.shield_remaining(entity)).raw() }));
        true
    }

    /// Consume only positive settled damage; return the unabsorbed remainder.
    pub fn absorb_damage_with_shield(&mut self, entity: Entity, damage: Fixed64) -> Fixed64 {
        if damage <= Fixed64::ZERO { return damage; }
        let shield = self.shield_remaining(entity);
        if shield == Fixed64::ZERO { return damage; }
        let absorbed = damage.min(shield);
        let remaining = shield - absorbed;
        if remaining == Fixed64::ZERO { self.remove(entity, "damage_shield"); }
        else if let Some(entry) = self.buffs.get_mut(&entity).and_then(|entries| entries.get_mut("damage_shield")) {
            entry.payload["shield_remaining_raw"] = serde_json::json!(remaining.raw());
        }
        damage - absorbed
    }

    /// Private, one-shot flat bonus. Not a permanent attack stat or public buff ID.
    /// Re-arming retains the strongest pending bonus, independent of item identity.
    pub fn arm_next_attack_bonus(&mut self, entity: Entity, amount: Fixed64) -> bool {
        if amount <= Fixed64::ZERO || amount > Fixed64::from_i32(1_000_000) { return false; }
        let amount = amount.max(self.next_attack_bonus(entity));
        self.add(entity, "__next_attack_bonus", Fixed64::from_raw(i64::MAX),
            serde_json::json!({ "next_attack_bonus_raw": amount.raw() }));
        true
    }

    pub fn next_attack_bonus(&self, entity: Entity) -> Fixed64 {
        self.get(entity, "__next_attack_bonus")
            .and_then(|entry| entry.payload.get("next_attack_bonus_raw"))
            .and_then(Value::as_i64).map(Fixed64::from_raw)
            .filter(|amount| *amount > Fixed64::ZERO && *amount <= Fixed64::from_i32(1_000_000))
            .unwrap_or(Fixed64::ZERO)
    }

    pub fn consume_next_attack_bonus(&mut self, entity: Entity) {
        self.remove(entity, "__next_attack_bonus");
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// 新增或刷新 buff。若已存在：duration 取 max、payload 依 should_replace
    /// 策略決定是否覆寫——例如 slow 採單一 instance（buff_id = "slow"），
    /// 由 payload 的 `slow_factor` 欄位驅動「強蓋弱」比較（見上方 Reserved
    /// 負載約定）。
    pub fn add(&mut self, entity: Entity, buff_id: &str, duration: Fixed64, payload: Value) {
        // A magic-immune unit keeps hostile debuffs out unless they pierce (magic_immunity, E179).
        if crate::runtime::native::magic_immunity::blocks(self, entity, buff_id, &payload) { return; }
        // Status / slow resistance of the target (damage_immunity::resist).
        let (duration, payload) = crate::runtime::native::damage_immunity::resist(self, entity, buff_id, duration, payload);
        if payload.get(super::damage_zone::KEY).is_some() {
            let Some(spec)=super::damage_zone::DamageZoneSpec::from_payload(&payload)else{return;};
            // An independent lease requires immutable host provenance, not just JSON.
            if spec.independent || duration<spec.interval || duration>Fixed64::from_i32(30){return;}
        }
        self.independent_zones.remove(&(entity,buff_id.to_owned()));
        self.insert_buff(entity,buff_id,duration,payload);
    }

    fn insert_buff(&mut self, entity: Entity, buff_id: &str, duration: Fixed64, payload: Value) {
        let entity_buffs = self.buffs.entry(entity).or_default();
        match entity_buffs.get_mut(buff_id) {
            Some(e) => {
                if payload.get(super::damage_zone::KEY).is_some() {
                    e.remaining=duration;
                    e.zone_clock=0;
                }
                if duration > e.remaining {
                    e.remaining = duration;
                }
                // payload 替換策略：
                //   - 若雙方 payload 都帶 `slow_factor`，較小者（更強）勝出，
                //     僅在新 payload 較強時才覆寫；否則保留原 payload。
                //   - 否則維持原本行為：覆寫。
                let should_replace = match (
                    e.payload.get("slow_factor").and_then(|v| v.as_f64()),
                    payload.get("slow_factor").and_then(|v| v.as_f64()),
                ) {
                    (Some(old), Some(new)) => new < old,
                    // 任一邊缺 slow_factor 或型別非數字 → fallback 到舊行為（覆寫）
                    _ => true,
                };
                if should_replace {
                    // A reused key must never pay an old source/kind's fractional damage.
                    if ["source_entity_id","source_entity_gen","dot_kind","damage_profile"].iter()
                        .any(|key|e.payload.get(*key)!=payload.get(*key)) {e.periodic_remainder=0;}
                    let old_keys: Vec<String> =
                        Self::payload_keys(&e.payload).map(String::from).collect();
                    let new_keys: Vec<String> =
                        Self::payload_keys(&payload).map(String::from).collect();
                    e.payload = payload;
                    for k in &old_keys {
                        if !new_keys.contains(k) {
                            self.index_dec(entity, k);
                        }
                    }
                    for k in &new_keys {
                        if !old_keys.contains(k) {
                            self.index_inc(entity, k);
                        }
                    }
                }
                // should_replace == false: duration 已 refresh, payload 不動, 索引不動
            }
            None => {
                let new_keys: Vec<String> =
                    Self::payload_keys(&payload).map(String::from).collect();
                entity_buffs.insert(
                    buff_id.to_string(),
                    BuffEntry {
                        remaining: duration,
                        payload,
                        periodic_remainder: 0,
                        zone_clock: 0,
                    },
                );
                for k in &new_keys {
                    self.index_inc(entity, k);
                }
            }
        }
    }

    pub fn remove(&mut self, entity: Entity, buff_id: &str) {
        self.independent_zones.remove(&(entity,buff_id.to_owned()));
        if let Some(entry) = self.remove_entry(entity, buff_id) {
            self.record_status_end(entity, buff_id, &entry.payload, crate::runtime::native::status_event::REMOVED);
        }
    }

    fn record_status_end(&mut self, entity: Entity, buff_id: &str, payload: &Value, kind: u8) {
        if payload.get(crate::runtime::native::status_event::KEY).is_some() {
            self.status_ended.push((entity, buff_id.to_owned(), payload.clone(), kind));
        }
    }

    /// The status-event rows that left their holders since the last drain (status_event).
    pub fn drain_status_ended(&mut self) -> Vec<(Entity, String, Value, u8)> {
        std::mem::take(&mut self.status_ended)
    }

    fn remove_entry(&mut self, entity: Entity, buff_id: &str) -> Option<BuffEntry> {
        let mut remove_entity = false;
        let entry = match self.buffs.get_mut(&entity) {
            Some(entity_buffs) => {
                let entry = entity_buffs.remove(buff_id);
                remove_entity = entity_buffs.is_empty();
                entry
            }
            None => None,
        };
        if remove_entity {
            self.buffs.remove(&entity);
        }
        if let Some(entry) = &entry {
            let keys: Vec<String> = Self::payload_keys(&entry.payload)
                .map(String::from)
                .collect();
            for k in &keys {
                self.index_dec(entity, k);
            }
        }
        entry
    }

    pub fn has(&self, entity: Entity, buff_id: &str) -> bool {
        self.buffs
            .get(&entity)
            .is_some_and(|entity_buffs| entity_buffs.contains_key(buff_id))
    }

    pub fn get(&self, entity: Entity, buff_id: &str) -> Option<&BuffEntry> {
        self.buffs
            .get(&entity)
            .and_then(|entity_buffs| entity_buffs.get(buff_id))
    }

    pub fn has_any(&self, entity: Entity) -> bool {
        self.buffs
            .get(&entity)
            .is_some_and(|entity_buffs| !entity_buffs.is_empty())
    }

    /// 清除 entity 的所有 buff（單位死亡時呼叫）。
    pub fn remove_all_for(&mut self, entity: Entity) {
        if let Some(entity_buffs) = self.buffs.remove(&entity) {
            let mut ended: Vec<(&String, &BuffEntry)> = entity_buffs.iter()
                .filter(|(_, entry)| entry.payload.get(crate::runtime::native::status_event::KEY).is_some()).collect();
            ended.sort_by(|a, b| a.0.cmp(b.0));
            let ended: Vec<(Entity, String, Value, u8)> = ended.into_iter()
                .map(|(id, entry)| (entity, id.clone(), entry.payload.clone(), crate::runtime::native::status_event::CLEARED)).collect();
            self.status_ended.extend(ended);
            for entry in entity_buffs.into_values() {
                let keys: Vec<String> = Self::payload_keys(&entry.payload)
                    .map(String::from)
                    .collect();
                for k in &keys {
                    self.index_dec(entity, k);
                }
            }
        }
    }

    /// 迭代某單位身上所有 buff（供 creep_tick 算移速乘數等）。
    pub fn iter_for(&self, entity: Entity) -> impl Iterator<Item = (&str, &BuffEntry)> {
        self.buffs
            .get(&entity)
            .into_iter()
            .flat_map(|entity_buffs| entity_buffs.iter().map(|(id, v)| (id.as_str(), v)))
    }

    /// 從 payload 抽出所有頂層 key（這些就是 stat key 字串）。
    /// payload 不是 Object 時返回空 iterator。
    fn payload_keys(payload: &Value) -> impl Iterator<Item = &str> {
        payload
            .as_object()
            .into_iter()
            .flat_map(|m| m.keys().map(|s| s.as_str()))
    }

    /// Whether `entity` has a buff whose payload carries `key` (the reverse index, O(1)).
    pub fn entity_has_key(&self, entity: Entity, key: &str) -> bool {
        self.entities_by_key.get(key).is_some_and(|holders| holders.contains_key(&entity))
    }

    fn index_inc(&mut self, entity: Entity, key: &str) {
        let inner = self.entities_by_key.entry(key.to_string()).or_default();
        *inner.entry(entity).or_insert(0) += 1;
    }

    fn index_dec(&mut self, entity: Entity, key: &str) {
        if let Some(inner) = self.entities_by_key.get_mut(key) {
            if let Some(cnt) = inner.get_mut(&entity) {
                *cnt = cnt.saturating_sub(1);
                if *cnt == 0 {
                    inner.remove(&entity);
                }
            }
            if inner.is_empty() {
                self.entities_by_key.remove(key);
            }
        }
    }

    /// 反向查詢：哪些 entity 身上有 buff payload 含 `key`。
    /// 配合 `regen_tick` / `buff_tick` 的 DoT 掃描，把「對全表 sum_add」
    /// 變成「只對候選 entity sum_add」。返回 iterator，呼叫端可 collect 或 filter。
    pub fn entities_with_key<'a>(&'a self, key: &str) -> impl Iterator<Item = Entity> + 'a {
        self.entities_by_key
            .get(key)
            .into_iter()
            .flat_map(|m| m.keys().copied())
    }

    /// 加法聚合：對 entity 身上所有 buff，若 `payload[stat]` 是數字則加總。
    /// 慣例：`_bonus` 後綴的 stat 用這個（例 `range_bonus`、`damage_bonus`）。
    /// 階段 1de.2：有效負載偏好原始 i32（鎖步正確）；傳統 f64 仍然
    /// 透過「read_fixed_from_payload」後備接受。
    pub fn sum_add(&self, entity: Entity, stat: StatKey) -> Fixed64 {
        self.sum_add_key(entity, stat.as_str())
    }

    /// 基礎攻擊間隔覆寫（`base_attack_time_constant`，原作 modifier 的 base_attack_time）：
    /// 多個來源取最小的正值（與走訪順序無關）。
    pub fn base_attack_time_override(&self, entity: Entity) -> Option<Fixed64> {
        self.iter_for(entity).filter_map(|(_, entry)| entry.payload.get(StatKey::BaseAttackTimeConstant.as_str()).and_then(Value::as_i64))
            .filter(|raw| *raw > 0).min().map(Fixed64::from_raw)
    }

    /// Checked aggregation for resource accounting. Preserve family strongest
    /// selection, but sum raw values in i128 so HashMap order cannot overflow.
    /// Invalid values or an unrepresentable final sum fail closed.
    pub fn checked_sum_add(&self, entity: Entity, stat: StatKey) -> Option<Fixed64> {
        let mut total=0i128;
        let mut families:BTreeMap<&str,i64>=BTreeMap::new();
        let mut ability_slows:BTreeMap<&str,i64>=BTreeMap::new();
        for (name,entry) in self.iter_for(entity) {
            let Some(value)=entry.payload.get(stat.as_str()) else {continue;};
            let raw=if let Some(raw)=value.as_i64() {raw} else {
                let scaled=value.as_f64()?*1024.0;
                // The upper bound is exclusive: f64 rounds i64::MAX to 2^63.
                if !scaled.is_finite() || scaled< i64::MIN as f64 || scaled>=9223372036854775808.0 {return None;}
                scaled as i64
            };
            let family=entry.payload.get(AGGREGATION_FAMILY_KEY).and_then(Value::as_str);
            if family==Some(ABILITY_SLOW_FAMILY) && stat.as_str()==ABILITY_MOVE_SLOW_KEY {
                note_ability_slow(&mut ability_slows,name,raw);continue;
            }
            if let Some(family)=family {
                families.entry(family).and_modify(|current| {
                    if raw.unsigned_abs()>current.unsigned_abs()
                        || (raw.unsigned_abs()==current.unsigned_abs() && raw>*current) {*current=raw;}
                }).or_insert(raw);
            } else {total=total.checked_add(i128::from(raw))?;}
        }
        for raw in families.into_values() {total=total.checked_add(i128::from(raw))?;}
        total=total.checked_add(i128::from(combine_ability_slows(&ability_slows)))?;
        Some(Fixed64::from_raw(i64::try_from(total).ok()?))
    }

    fn sum_add_key(&self, entity: Entity, key: &str) -> Fixed64 {
        let mut total = Fixed64::ZERO;
        let mut family_values: BTreeMap<&str, Fixed64> = BTreeMap::new();
        let mut ability_slows: BTreeMap<&str, i64> = BTreeMap::new();
        for (name, entry) in self.iter_for(entity) {
            let Some(value) = entry.payload.get(key) else {
                continue;
            };
            let value = read_fixed_from_payload(value);
            let Some(family) = entry
                .payload
                .get(AGGREGATION_FAMILY_KEY)
                .and_then(Value::as_str)
            else {
                total += value;
                continue;
            };
            if family == ABILITY_SLOW_FAMILY && key == ABILITY_MOVE_SLOW_KEY {
                note_ability_slow(&mut ability_slows, name, value.raw());
                continue;
            }
            family_values
                .entry(family)
                .and_modify(|current| {
                    if value.raw().abs() > current.raw().abs()
                        || (value.raw().abs() == current.raw().abs() && value > *current)
                    {
                        *current = value;
                    }
                })
                .or_insert(value);
        }
        family_values
            .into_values()
            .fold(total, |acc, value| acc + value)
            + Fixed64::from_raw(combine_ability_slows(&ability_slows))
    }

    pub fn move_speed_sums(&self, entity: Entity) -> MoveSpeedSums {
        let mut sums = MoveSpeedSums::default();
        for key in MOVE_SPEED_SUM_KEYS {
            let value = self.sum_add_key(entity, key);
            if let Some(slot) = move_speed_sum_slot(&mut sums, key) {
                *slot += value;
            }
        }
        sums
    }

    /// 乘法聚合：對 entity 身上所有 buff，若 `payload[stat]` 是數字則連乘。
    /// 空集合回 1.0 (Fixed64::ONE)。慣例：`_multiplier` 後綴的 stat 用這個
    /// （例 `attack_speed_multiplier`、`move_speed_multiplier`）。
    /// 階段 1de.2：有效負載偏好原始 i32（鎖步正確）；傳統 f64 仍然
    /// 透過「read_fixed_from_payload」後備接受。
    pub fn product_mult(&self, entity: Entity, stat: StatKey) -> Fixed64 {
        let key = stat.as_str();
        let mut product = Fixed64::ONE;
        let mut family_values: BTreeMap<&str, Fixed64> = BTreeMap::new();
        for (_, entry) in self.iter_for(entity) {
            let Some(value) = entry.payload.get(key) else {
                continue;
            };
            let value = read_fixed_from_payload(value);
            let Some(family) = entry
                .payload
                .get(AGGREGATION_FAMILY_KEY)
                .and_then(Value::as_str)
            else {
                product *= value;
                continue;
            };
            family_values
                .entry(family)
                .and_modify(|current| {
                    let distance = (value - Fixed64::ONE).raw().abs();
                    let current_distance = (*current - Fixed64::ONE).raw().abs();
                    if distance > current_distance
                        || (distance == current_distance && value > *current)
                    {
                        *current = value;
                    }
                })
                .or_insert(value);
        }
        family_values
            .into_values()
            .fold(product, |acc, value| acc * value)
    }

    /// 控制類 buff 判定 — 這些 buff_id 出現在單位身上代表其處於特定 CC 狀態。
    /// 約定：`stun` 同時禁攻擊與移動；`silence` 禁技能施放；`root` 只禁移動。
    pub fn is_stunned(&self, entity: Entity) -> bool {
        self.has(entity, BuffId::Stun.as_str())
    }

    pub fn is_rooted(&self, entity: Entity) -> bool {
        self.has(entity, BuffId::Root.as_str()) || self.has(entity, BuffId::Stun.as_str())
            // A forced displacement owns the position even after its stun is cleansed.
            || self.has(entity, crate::runtime::native::forced_motion::BUFF)
    }

    /// Timed stealth granted by `invisibility_level` buffs (whole levels).
    /// Visibility uses the higher of this and the entity's `StealthProfile`;
    /// attacking does not reveal. Zero when no such buff is active.
    pub fn stealth_level(&self, entity: Entity) -> u16 {
        let raw = self.sum_add(entity, StatKey::InvisibilityLevel).raw();
        u16::try_from((raw / 1024).max(0)).unwrap_or(u16::MAX)
    }

    /// Other units' movement does not collide with this one while it carries
    /// the [`NO_UNIT_COLLISION`] state buff. Terrain still blocks it.
    pub fn ignores_unit_collision(&self, entity: Entity) -> bool {
        self.has(entity, NO_UNIT_COLLISION)
    }

    pub fn is_silenced(&self, entity: Entity) -> bool {
        self.has(entity, BuffId::Silence.as_str()) || self.has(entity, BuffId::Stun.as_str())
    }

    /// Closed hostile slow producers only. Do not infer debuffs from a negative
    /// movement stat: voluntary stances and mixed/unknown effects must survive.
    fn cleanseable_slow(name:&str,entry:&BuffEntry)->bool {
        if entry.remaining<=Fixed64::ZERO{return false;}
        let Some(payload)=entry.payload.as_object()else{return false;};
        if payload.len()!=2{return false;}
        if omb_script_abi::buff_ids::declarative_buff_source(name).is_some_and(|(_,stat)|stat.is_none()) {
            return payload.get(AGGREGATION_FAMILY_KEY).and_then(Value::as_str)==Some("generic_ability_slow")
                && payload.get("move_speed_bonus").and_then(Value::as_i64).is_some_and(|v|(-1024..=-1).contains(&v));
        }
        if name!="slow"{return false;}
        // Projectile slow still publishes render floats. Admit only exact Q10
        // values of that producer, not a permissive float-to-gameplay conversion.
        let raw=|key:&str,min:i64,max:i64|->Option<i64>{
            let n=payload.get(key)?.as_f64()?*1024.;
            (n.is_finite()&&n.fract()==0.&&n>=min as f64&&n<=max as f64).then_some(n as i64)
        };
        match (raw("slow_factor",0,1023),raw("move_speed_bonus",-1024,-1)) {
            (Some(factor),Some(bonus))=>bonus==factor-1024,_=>false,
        }
    }

    /// Read-only parity with canonical cleansing. Standard key presence follows
    /// the existing blocking semantics; slows share the exact closed predicate.
    pub fn has_cleanseable_controls(&self, entity:Entity)->bool {
        [BuffId::Stun,BuffId::Root,BuffId::Silence].iter().any(|id|self.has(entity,id.as_str()))
            || self.buffs.get(&entity).is_some_and(|rows|rows.iter().any(|(name,entry)|Self::cleanseable_slow(name,entry)))
    }

    /// Explicit supported controls and recognized hostile slows only; unrelated
    /// buffs retain payload, lifetime and integration clocks. Canonical removal
    /// maintains reverse indexes and never grants future control immunity.
    pub fn cleanse_controls(&mut self, entity: Entity) {
        for id in [BuffId::Stun, BuffId::Root, BuffId::Silence] {
            self.remove(entity, id.as_str());
        }
        self.cleanse_slows(entity);
    }

    /// Removes the entity's cleanseable movement slows only.
    pub fn cleanse_slows(&mut self, entity: Entity) {
        let mut slows:Vec<_>=self.buffs.get(&entity).into_iter().flat_map(|rows|rows.iter())
            .filter(|(name,entry)|Self::cleanseable_slow(name,entry)).map(|(name,_)|name.clone()).collect();
        slows.sort();for id in slows{self.remove(entity,&id);}
    }

    /// 倒數所有 buff 並回傳過期的 `(Entity, buff_id, payload)` 清單。
    /// 呼叫端可依 payload 內容決定是否廣播（例：payload 含 move_speed_bonus
    /// 表示這是移速影響類 buff，要發 creep/S 還原訊息）。
    /// 階段 1c.3：dt 為固定 64 秒。
    /// Integrate BEFORE tick retires entries. Stable ordering is independent of HashMap seed.
    pub fn drain_periodic_damage(&mut self, dt: Fixed64) -> Vec<super::periodic_damage::PeriodicDamageTick> {
        use super::periodic_damage::{PeriodicDamageSpec,PeriodicDamageTick};
        if dt<=Fixed64::ZERO {return Vec::new();}
        let mut entities:Vec<_>=self.entities_with_key("dot_damage").collect();
        entities.sort_by_key(|e|(e.id(),e.gen().id()));
        let mut result=Vec::new();
        for target in entities {
            let Some(entries)=self.buffs.get_mut(&target) else {continue;};
            let mut keys:Vec<_>=entries.keys().cloned().collect();keys.sort();
            for key in keys {
                let entry=entries.get_mut(&key).unwrap();
                let Some(spec)=PeriodicDamageSpec::from_payload(&entry.payload) else {continue;};
                let elapsed=dt.min(entry.remaining).max(Fixed64::ZERO);
                if let (Some(interval),Some(pulse))=(spec.interval,spec.pulse_amount()) {
                    // Discrete ticks: the remainder is this buff's raw clock since its last
                    // pulse; a pulse lands at every full interval, including one that ends
                    // exactly at expiry (integrated before `tick` retires the entry).
                    let clock=i128::from(entry.periodic_remainder)+i128::from(elapsed.raw());
                    let pulses=clock/i128::from(interval.raw());
                    entry.periodic_remainder=(clock%i128::from(interval.raw())) as i64;
                    let Ok(raw)=i64::try_from(i128::from(pulse.raw())*pulses) else {continue;};
                    if raw>0 {result.push(PeriodicDamageTick{target,spec,amount:Fixed64::from_raw(raw)});}
                    continue;
                }
                let numerator=i128::from(spec.rate.raw())*i128::from(elapsed.raw())+i128::from(entry.periodic_remainder);
                let Ok(raw)=i64::try_from(numerator/1024) else {continue;};
                entry.periodic_remainder=(numerator%1024) as i64;
                if raw>0 {result.push(PeriodicDamageTick{target,spec,amount:Fixed64::from_raw(raw)});}
            }
        }
        result
    }

    /// Sample pulses before lifetime expiry; only indexed zone owners are visited.
    pub fn cancel_damage_zones(&mut self,source:Entity) {
        let keys:Vec<_>=self.buffs.get(&source).into_iter().flat_map(|entries|entries.iter())
            .filter(|(_,entry)|entry.payload.get(super::damage_zone::KEY).is_some()).map(|(key,_)|key.clone()).collect();
        // Source retirement clears its public status mirror, never an independent lease.
        for key in keys {self.remove_entry(source,&key);}
    }

    pub fn can_schedule_independent_zones(&self,source:Entity,keys:&[&str])->bool {
        use super::damage_zone::{MAX_INDEPENDENT_ZONES,MAX_INDEPENDENT_ZONES_PER_SOURCE};
        if keys.len()>MAX_INDEPENDENT_ZONES_PER_SOURCE || self.next_zone_lease.checked_add(keys.len() as u64).is_none()
            || keys.iter().any(|key|key.is_empty()||key.len()>128||key.chars().any(char::is_control)){return false;}
        let new:std::collections::BTreeSet<_>=keys.iter().copied()
            .filter(|key|!self.independent_zones.contains_key(&(source,(*key).to_owned()))).collect();
        self.independent_zones.len()+new.len()<=MAX_INDEPENDENT_ZONES
            && self.independent_zones.keys().filter(|(owner,_)|*owner==source).count()+new.len()<=MAX_INDEPENDENT_ZONES_PER_SOURCE
    }

    pub fn schedule_independent_zone(&mut self,source:Entity,key:&str,duration:Fixed64,
        spec:super::damage_zone::DamageZoneSpec,origin:crate::runtime::comp::ProjectileOrigin,faction:crate::runtime::comp::Faction)->bool {
        if !spec.independent || !spec.valid() || duration<spec.interval || duration>Fixed64::from_i32(30)
            || origin.source!=source || origin.normal_attack || origin.script_attack || origin.attack_missed || origin.attack_critical
            || origin.attack_lifesteal!=Fixed64::ZERO || origin.script.is_some()
            || !self.can_schedule_independent_zones(source,&[key]) {return false;}
        let Some(lease)=self.next_zone_lease.checked_add(1)else{return false;};
        self.next_zone_lease=lease;
        self.insert_buff(source,key,duration,spec.payload());
        self.independent_zones.insert((source,key.to_owned()),super::damage_zone::IndependentDamageZone {
            spec,remaining:duration,clock:0,lease,origin,faction,age:Fixed64::ZERO});
        true
    }

    pub fn independent_zone(&self,source:Entity,key:&str,lease:u64)->Option<&super::damage_zone::IndependentDamageZone> {
        self.independent_zones.get(&(source,key.to_owned())).filter(|entry|entry.lease==lease)
    }

    /// Authority-only future state. Never append this book to public actor buffs.
    pub fn damage_zone_diagnostic(&self)->Option<Value> {
        use super::damage_zone::KEY;
        if self.next_zone_lease==0 && self.independent_zones.is_empty() && !self.entities_by_key.contains_key(KEY){return None;}
        let mut bound=Vec::new();
        for source in self.entities_with_key(KEY) {for (key,entry) in self.iter_for(source) {
            if entry.payload.get(KEY).is_some(){bound.push(((source.id(),source.gen().id(),key.to_owned()),
                serde_json::json!([source.id(),source.gen().id(),key,entry.remaining.raw(),entry.zone_clock,entry.payload])));}
        }}
        bound.sort_by(|a,b|a.0.cmp(&b.0));
        let mut independent:Vec<_>=self.independent_zones.iter().map(|((source,key),entry)|
            ((source.id(),source.gen().id(),key.clone()),serde_json::json!([source.id(),source.gen().id(),key,entry.remaining.raw(),entry.clock,
                entry.lease,entry.spec.payload(),entry.origin,entry.faction]))).collect();
        independent.sort_by(|a,b|a.0.cmp(&b.0));
        Some(serde_json::json!({"next_lease":self.next_zone_lease,"bound":bound.into_iter().map(|(_,row)|row).collect::<Vec<_>>(),
            "independent":independent.into_iter().map(|(_,row)|row).collect::<Vec<_>>()}))
    }

    /// Sample pulses before lifetime expiry; only indexed zone owners are visited.
    pub fn drain_damage_zones(&mut self,dt:Fixed64)->Vec<super::damage_zone::DamageZonePulse> {
        use super::damage_zone::{DamageZoneSpec,DamageZonePulse,KEY};
        if dt<=Fixed64::ZERO {return Vec::new();}
        // Last tick's final pulses have settled. Expiry itself must not revoke a
        // pulse due exactly at that endpoint; explicit remove/refresh does revoke it.
        self.independent_zones.retain(|_,entry|entry.remaining>Fixed64::ZERO);
        let mut owners:Vec<_>=self.entities_with_key(KEY).collect();owners.sort_by_key(|e|(e.id(),e.gen().id()));
        let mut pulses=Vec::new();
        for source in owners {
            let Some(entries)=self.buffs.get_mut(&source)else{continue;};
            let mut keys:Vec<_>=entries.keys().cloned().collect();keys.sort();
            for key in keys {
                let entry=entries.get_mut(&key).unwrap();
                let Some(spec)=DamageZoneSpec::from_payload(&entry.payload)else{continue;};
                if spec.independent {continue;}
                if entry.remaining<=Fixed64::ZERO || entry.remaining>Fixed64::from_i32(30) {continue;}
                let active=dt.min(entry.remaining).raw();
                let previous=entry.zone_clock;
                let clock=previous+active;
                entry.zone_clock=clock%spec.interval.raw();
                for ordinal in 1..=clock/spec.interval.raw() {
                    let due=ordinal*spec.interval.raw()-previous;
                    pulses.push((due,source.id(),source.gen().id(),key.clone(),
                        DamageZonePulse {source,spec,key:key.clone(),lease:None,due_offset:Fixed64::from_raw(due)}));
                }
            }
        }
        for ((source,key),entry) in &mut self.independent_zones {
            let active=dt.min(entry.remaining).raw();let previous=entry.clock;let clock=previous+active;
            entry.clock=clock%entry.spec.interval.raw();
            for ordinal in 1..=clock/entry.spec.interval.raw() {
                let due=ordinal*entry.spec.interval.raw()-previous;
                pulses.push((due,source.id(),source.gen().id(),key.clone(),DamageZonePulse {
                    source:*source,spec:entry.spec,key:key.clone(),lease:Some(entry.lease),due_offset:Fixed64::from_raw(due)}));
            }
        }
        // A later low-ID source must not consume shields or take first-lethal
        // credit ahead of an earlier high-ID pulse in the same simulation tick.
        pulses.sort_by(|a,b|(a.0,a.1,a.2,&a.3).cmp(&(b.0,b.1,b.2,&b.3)));
        pulses.into_iter().map(|(_,_,_,_,pulse)|pulse).collect()
    }

    pub fn tick(&mut self, dt: Fixed64) -> Vec<(Entity, String, Value)> {
        if dt<=Fixed64::ZERO {return Vec::new();}
        for entry in self.independent_zones.values_mut(){entry.age+=dt.min(entry.remaining);entry.remaining-=dt.min(entry.remaining);}
        let mut expired = Vec::new();
        // 先收集 expired，避免 retain 內動態借 self（index_dec 也要 &mut self）
        let mut to_drop: Vec<(Entity, String)> = Vec::new();
        for (e, entity_buffs) in self.buffs.iter_mut() {
            for (id, v) in entity_buffs.iter_mut() {
                if v.remaining==PERMANENT_BUFF_DURATION {continue;}
                v.remaining -= dt;
                if v.remaining <= Fixed64::ZERO {
                    to_drop.push((*e, id.clone()));
                }
            }
        }
        to_drop.sort_by(|(a,ak),(b,bk)|(a.id(),a.gen().id(),ak).cmp(&(b.id(),b.gen().id(),bk)));
        for (e, id) in to_drop {
            if let Some(entry) = self.remove_entry(e, &id) {
                self.record_status_end(e, &id, &entry.payload, crate::runtime::native::status_event::EXPIRED);
                expired.push((e, id, entry.payload));
            }
        }
        expired
    }

    pub fn len(&self) -> usize {
        self.buffs.values().map(HashMap::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.buffs.is_empty()
    }
}

fn move_speed_sum_slot<'a>(sums: &'a mut MoveSpeedSums, key: &str) -> Option<&'a mut Fixed64> {
    match key {
        "movespeed_absolute" => Some(&mut sums.absolute),
        "movespeed_base_override" => Some(&mut sums.base_override),
        "movespeed_bonus_equipment" => Some(&mut sums.bonus_equipment),
        "movespeed_bonus_percentage"
        | "movespeed_bonus_percentage_unique"
        | "movespeed_bonus_percentage_unique_2"
        | "move_speed_bonus" => Some(&mut sums.percentage),
        "movespeed_bonus_buff" => Some(&mut sums.bonus_buff),
        "movespeed_absolute_min" => Some(&mut sums.absolute_min),
        "movespeed_max" => Some(&mut sums.max),
        "movespeed_limit" => Some(&mut sums.limit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bot_self_cleanse_eligibility_matches_canonical_removal_without_mutation() {
        let e=ent(1,1);let other=ent(2,1);let mut buffs=BuffStore::default();assert!(!buffs.has_cleanseable_controls(e));
        for name in ["stun","root","silence"]{buffs.add(e,name,fx(3.),serde_json::json!({}));assert!(buffs.has_cleanseable_controls(e));assert!(!buffs.has_cleanseable_controls(other));buffs.cleanse_controls(e);assert!(!buffs.has_cleanseable_controls(e));}
        for payload in [serde_json::json!({"move_speed_bonus":-128}),serde_json::json!({"move_speed_bonus":-0.5,"slow_factor":0.6}),serde_json::json!({"slow_factor":0.5})]{
            buffs.add(e,"slow",fx(3.),payload);assert!(!buffs.has_cleanseable_controls(e));buffs.remove(e,"slow");}
        buffs.add(e,"stance",fx(3.),serde_json::json!({"move_speed_bonus":-128}));assert!(!buffs.has_cleanseable_controls(e));
        let key=omb_script_abi::buff_ids::generic_slow_buff_id("arbitrary_skill",omb_script_abi::types::EntityHandle{id:9,gen:1});
        buffs.add(e,&key,fx(3.),serde_json::json!({"move_speed_bonus":-256,"__aggregation_family":"generic_ability_slow"}));
        let before=buffs.get(e,&key).unwrap().clone();assert!(buffs.has_cleanseable_controls(e));assert_eq!(buffs.get(e,&key).unwrap().remaining,before.remaining);assert_eq!(buffs.get(e,&key).unwrap().payload,before.payload);
        buffs.cleanse_controls(e);assert!(!buffs.has_cleanseable_controls(e));assert!(buffs.has(e,"stance"));
        buffs.add(e,"slow",fx(3.),serde_json::json!({"move_speed_bonus":-0.5,"slow_factor":0.5}));assert!(buffs.has_cleanseable_controls(e));buffs.cleanse_controls(e);assert!(!buffs.has_cleanseable_controls(e));
    }
    #[test]
    fn slow_cleanse_accepts_closed_producers_preserves_other_entries_and_reverse_indexes() {
        let e=ent(1,1);let other=ent(2,1);let mut buffs=BuffStore::default();
        let key=omb_script_abi::buff_ids::generic_slow_buff_id("arbitrary_skill",omb_script_abi::types::EntityHandle{id:9,gen:1});
        let second=omb_script_abi::buff_ids::generic_slow_buff_id("other_skill",omb_script_abi::types::EntityHandle{id:10,gen:2});
        let payload=serde_json::json!({"move_speed_bonus":-256,"__aggregation_family":"generic_ability_slow"});
        for entity in [e,other]{buffs.add(entity,&key,fx(2.),payload.clone());buffs.add(entity,"slow",fx(2.),serde_json::json!({"move_speed_bonus":-0.5,"slow_factor":0.5}));}
        buffs.add(e,&second,fx(3.),serde_json::json!({"move_speed_bonus":-512,"__aggregation_family":"generic_ability_slow"}));
        for id in ["stun","root","silence"]{buffs.add(e,id,fx(2.),serde_json::json!({}));}
        for (id,payload) in [
            ("voluntary_stance",serde_json::json!({"move_speed_bonus":-512})),
            ("retained_dot",serde_json::json!({"dot_damage":100,"__aggregation_family":"different"})),
            ("generic_slow:foreign:9:1",serde_json::json!({"move_speed_bonus":-256,"__aggregation_family":"different"})),
            ("generic_slow:mixed:9:1",serde_json::json!({"move_speed_bonus":-256,"__aggregation_family":"generic_ability_slow","dot_damage":12})),
            ("generic_slow:ambiguous:09:1",serde_json::json!({"move_speed_bonus":-256,"__aggregation_family":"generic_ability_slow"})),
            ("generic_slow:positive:9:1",serde_json::json!({"move_speed_bonus":256,"__aggregation_family":"generic_ability_slow"})),
            ("generic_slow:float:9:1",serde_json::json!({"move_speed_bonus":-0.5,"__aggregation_family":"generic_ability_slow"})),
        ]{buffs.add(e,id,fx(7.),payload);}
        buffs.grant_damage_shield(e,Fixed64::from_i32(30),fx(5.));buffs.grant_healing_reduction(e,Fixed64::from_raw(512),fx(5.));
        {let row=buffs.buffs.get_mut(&e).unwrap().get_mut("retained_dot").unwrap();row.periodic_remainder=17;row.zone_clock=23;}
        let retained:Vec<_>=buffs.buffs[&e].iter().filter(|(name,_)|!matches!(name.as_str(),"slow"|"stun"|"root"|"silence")&&name.as_str()!=key.as_str()&&name.as_str()!=second.as_str()).map(|(name,row)|(name.clone(),row.clone())).collect();
        buffs.cleanse_controls(e);assert!(!buffs.has(e,&key)&&!buffs.has(e,"slow")&&!buffs.is_stunned(e)&&!buffs.is_rooted(e)&&!buffs.is_silenced(e));
        assert!(!buffs.has(e,&second));
        assert!(!buffs.entities_with_key("slow_factor").any(|entity|entity==e));assert!(buffs.entities_with_key("slow_factor").any(|entity|entity==other));
        for (name,before) in retained{let after=buffs.get(e,&name).unwrap();assert_eq!(after.payload,before.payload);assert_eq!(after.remaining,before.remaining);assert_eq!((after.periodic_remainder,after.zone_clock),(before.periodic_remainder,before.zone_clock));}
        buffs.cleanse_controls(other);assert!(!buffs.entities_with_key("slow_factor").any(|_|true));
        buffs.add(e,&key,fx(2.),payload);assert!(buffs.has(e,&key),"no immunity");
        for payload in [serde_json::json!({"move_speed_bonus":-0.5,"slow_factor":0.4}),serde_json::json!({"move_speed_bonus":-0.3333,"slow_factor":0.6667}),
            serde_json::json!({"move_speed_bonus":-512,"slow_factor":512}),serde_json::json!({"move_speed_bonus":-0.5,"slow_factor":0.5,"extra":1}),serde_json::json!({"move_speed_bonus":"-0.5","slow_factor":0.5})]{
            buffs.add(e,"slow",fx(2.),payload);buffs.cleanse_controls(e);assert!(buffs.has(e,"slow"));buffs.remove(e,"slow");}
        for value in [-1024,-1]{let payload=serde_json::json!({"move_speed_bonus":value,"__aggregation_family":"generic_ability_slow"});buffs.add(e,&key,fx(2.),payload);buffs.cleanse_controls(e);assert!(!buffs.has(e,&key));}
        for factor in [0,1,1023]{buffs.add(e,"slow",fx(2.),serde_json::json!({"move_speed_bonus":(factor-1024)as f64/1024.,"slow_factor":factor as f64/1024.}));buffs.cleanse_controls(e);assert!(!buffs.has(e,"slow"));}
    }

    #[test]
    fn independent_zone_nonce_exhaustion_rejects_without_blocking_ordinary_casts() {
        let mut buffs=BuffStore::default();let source=ent(1,1);buffs.next_zone_lease=u64::MAX;
        let before=buffs.damage_zone_diagnostic();assert!(!buffs.can_schedule_independent_zones(source,&["zone"]));
        assert!(buffs.can_schedule_independent_zones(source,&[]));assert_eq!(buffs.damage_zone_diagnostic(),before);
    }
    #[test]
    fn public_damage_shield_metadata_tracks_absorption_regrant_and_expiry() {
        let entity=ent(1,1);let mut buffs=BuffStore::default();
        assert!(omoba_template_ids::buff_by_name("damage_shield").is_some());
        assert!(buffs.grant_damage_shield(entity,Fixed64::from_i32(100),Fixed64::from_i32(3)));
        assert_eq!(buffs.get(entity,"damage_shield").unwrap().payload["shield_remaining_raw"],102400);
        assert_eq!(buffs.absorb_damage_with_shield(entity,Fixed64::from_i32(40)),Fixed64::ZERO);
        assert_eq!(buffs.get(entity,"damage_shield").unwrap().payload["shield_remaining_raw"],61440);
        assert!(buffs.grant_damage_shield(entity,Fixed64::from_i32(20),Fixed64::from_i32(2)));
        assert_eq!(buffs.shield_remaining(entity),Fixed64::from_i32(60));
        assert_eq!(buffs.absorb_damage_with_shield(entity,Fixed64::from_i32(70)),Fixed64::from_i32(10));
        assert!(!buffs.has(entity,"damage_shield"));
        buffs.grant_damage_shield(entity,Fixed64::from_i32(20),Fixed64::from_i32(1));
        buffs.tick(Fixed64::from_i32(1));assert!(!buffs.has(entity,"damage_shield"));
    }

    #[test]
    fn canonical_permanent_lifetime_never_ticks_down_and_explicit_removal_works() {
        let entity=ent(1,1);
        let mut buffs=BuffStore::default();
        buffs.add(entity,"forever",PERMANENT_BUFF_DURATION,serde_json::json!({"physical_armor_bonus":1024}));
        buffs.add(entity,"finite",Fixed64::from_i32(1),serde_json::json!({}));
        assert_eq!(buffs.tick(Fixed64::from_i32(2)).len(),1);
        for _ in 0..10 {assert!(buffs.tick(Fixed64::from_raw(i64::MAX/2)).is_empty());}
        assert_eq!(buffs.iter_for(entity).next().unwrap().1.remaining,PERMANENT_BUFF_DURATION);
        assert_eq!(crate::runtime::buff_remaining_secs_for_snapshot(PERMANENT_BUFF_DURATION),-1.0);
        buffs.remove(entity,"forever");assert!(!buffs.has(entity,"forever"));
    }
    use super::*;
    use serde_json::json;
    use specs::world::Generation;

    fn ent(id: u32, gen: i32) -> Entity {
        Entity::new(id, Generation::new(gen))
    }

    fn fx(seconds: f32) -> Fixed64 {
        Fixed64::from_raw((seconds * 1024.0) as i64)
    }

    #[test]
    fn item_timed_modifier_identity_is_typed_and_independent_of_source_and_net_stat() {
        let e=ent(1,1);let other=ent(2,1);let mut s=BuffStore::new();
        for kind in [ItemTimedModifier::Sprint,ItemTimedModifier::DamageReduction] {
            assert!(!s.has_item_timed_modifier(e,kind));
            for (amount,time) in [(Fixed64::ZERO,fx(1.0)),(kind.maximum()+Fixed64::ONE,fx(1.0)),
                (Fixed64::ONE,Fixed64::ZERO),(Fixed64::ONE,fx(61.0))] {
                assert!(!s.grant_item_timed_modifier(e,"item",kind,amount,time));
                assert!(!s.has_item_timed_modifier(e,kind));
            }
            let amount=if kind==ItemTimedModifier::Sprint {fx(60.0)} else {fx(0.25)};
            assert!(s.grant_item_timed_modifier(e,"item",kind,amount,fx(1.0)));
            assert!(s.has_item_timed_modifier(e,kind));
            assert!(!s.has_item_timed_modifier(other,kind));
            s.add(e,"unrelated_vulnerability",fx(10.0),json!({StatKey::DamageTakenBonus.as_str():1024}));
            if kind==ItemTimedModifier::DamageReduction { assert!(s.sum_add(e,kind.stat())>Fixed64::ZERO); }
            assert!(s.has_item_timed_modifier(e,kind));
            s.tick(fx(1.0));
            assert!(!s.has_item_timed_modifier(e,kind));
            // A source name alone cannot mark an active effect.
            s.add(e,&format!("{}:fake",kind.family()),fx(1.0),json!({}));
            assert!(!s.has_item_timed_modifier(e,kind));
            let value=if kind==ItemTimedModifier::Sprint {amount.raw()} else {-amount.raw()};
            s.add(e,"arbitrary_source",fx(1.0),json!({AGGREGATION_FAMILY_KEY:kind.family(),kind.stat().as_str():value}));
            assert!(s.has_item_timed_modifier(e,kind));
            s.remove_all_for(e);
            // The query uses the same legacy numeric reader as aggregation.
            s.add(e,"legacy_source",fx(1.0),json!({AGGREGATION_FAMILY_KEY:kind.family(),kind.stat().as_str():value as f64 / 1024.0}));
            assert!(s.has_item_timed_modifier(e,kind));
            s.remove_all_for(e);
            assert!(!s.has_item_timed_modifier(e,kind));
        }
    }

    #[test]
    fn entities_with_key_returns_entity_after_add() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(e, "buff_a", fx(5.0), json!({ "move_speed_bonus": -0.5 }));
        let found: Vec<Entity> = s.entities_with_key("move_speed_bonus").collect();
        assert_eq!(found, vec![e]);
    }

    #[test]
    fn remove_clears_index() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(e, "b", fx(5.0), json!({ "x": 1.0 }));
        s.remove(e, "b");
        let found: Vec<Entity> = s.entities_with_key("x").collect();
        assert!(found.is_empty(), "expected empty, got {:?}", found);
    }

    #[test]
    fn tick_expired_clears_index() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(e, "b", fx(1.0), json!({ "x": 1.0 }));
        let expired = s.tick(fx(2.0)); // duration < dt → expire
        assert_eq!(expired.len(), 1);
        let found: Vec<Entity> = s.entities_with_key("x").collect();
        assert!(
            found.is_empty(),
            "expected empty after expire, got {:?}",
            found
        );
    }

    #[test]
    fn remove_all_for_clears_index() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(e, "a", fx(5.0), json!({ "x": 1.0, "y": 2.0 }));
        s.add(e, "b", fx(5.0), json!({ "z": 3.0 }));
        s.remove_all_for(e);
        for k in &["x", "y", "z"] {
            let found: Vec<Entity> = s.entities_with_key(k).collect();
            assert!(found.is_empty(), "key {} not cleared: {:?}", k, found);
        }
    }

    #[test]
    fn refcount_multiple_buffs_same_key() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(e, "buff1", fx(5.0), json!({ "k": 1.0 }));
        s.add(e, "buff2", fx(5.0), json!({ "k": 2.0 }));

        // 兩者都存在 - 實體仍在索引中
        assert_eq!(s.entities_with_key("k").count(), 1);

        s.remove(e, "buff1");
        // 還剩下一個 → 仍然被索引
        let found: Vec<Entity> = s.entities_with_key("k").collect();
        assert_eq!(
            found,
            vec![e],
            "after removing 1 of 2, entity should still be indexed"
        );

        s.remove(e, "buff2");
        // 都消失了 → 未編入索引
        assert!(s.entities_with_key("k").next().is_none());
    }

    #[test]
    fn slow_dedup_stronger_replaces_weaker() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        // 先加弱 slow（factor 越小越強，0.5 比 0.3 弱）
        s.add(
            e,
            "slow",
            fx(5.0),
            json!({ "move_speed_bonus": -0.5, "slow_factor": 0.5 }),
        );
        // 加強 slow
        s.add(
            e,
            "slow",
            fx(5.0),
            json!({ "move_speed_bonus": -0.7, "slow_factor": 0.3 }),
        );
        // 應該保留強 slow（factor=0.3）
        let entry = s.get(e, "slow").expect("slow buff missing");
        let factor = entry
            .payload
            .get("slow_factor")
            .and_then(|v| v.as_f64())
            .unwrap();
        assert!(
            (factor - 0.3).abs() < 1e-6,
            "expected 0.3 (stronger), got {}",
            factor
        );
    }

    #[test]
    fn read_fixed_from_payload_prefers_raw_i64() {
        // 階段 1de.2：整數有效負載 → 原始固定 64（鎖步正確）。
        // 0.5（固定 64 原始資料 = 512）。
        let v = serde_json::json!(512);
        assert_eq!(read_fixed_from_payload(&v), Fixed64::from_raw(512));
    }

    #[test]
    fn read_fixed_from_payload_legacy_f64_fallback() {
        // 向後相容：浮點有效負載仍然透過 *1024 量化進行解析。
        let v = serde_json::json!(0.5);
        assert_eq!(read_fixed_from_payload(&v), Fixed64::from_raw(512));
    }

    #[test]
    fn read_fixed_from_payload_nonnumeric_returns_zero() {
        let v = serde_json::json!("not_a_number");
        assert_eq!(read_fixed_from_payload(&v), Fixed64::ZERO);
        let v_null = serde_json::Value::Null;
        assert_eq!(read_fixed_from_payload(&v_null), Fixed64::ZERO);
    }

    #[test]
    fn sum_add_handles_mixed_raw_and_legacy_encodings() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        // 舊腳本（舊版 f64）：0.3 → 原始 307
        s.add(e, "old_buff", fx(5.0), json!({ "move_speed_bonus": 0.3 }));
        // 新腳本（原始 i32）：0.2 → 原始 204
        s.add(e, "new_buff", fx(5.0), json!({ "move_speed_bonus": 204 }));
        let total = s.sum_add(e, StatKey::MoveSpeedBonus);
        // 307 + 204 = 511
        assert_eq!(total, Fixed64::from_raw(511));
    }

    #[test]
    fn slow_dedup_weaker_does_not_replace_stronger() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(
            e,
            "slow",
            fx(3.0),
            json!({ "move_speed_bonus": -0.7, "slow_factor": 0.3 }),
        );
        s.add(
            e,
            "slow",
            fx(10.0),
            json!({ "move_speed_bonus": -0.5, "slow_factor": 0.5 }),
        );
        let entry = s.get(e, "slow").expect("slow buff missing");
        let factor = entry
            .payload
            .get("slow_factor")
            .and_then(|v| v.as_f64())
            .unwrap();
        assert!(
            (factor - 0.3).abs() < 1e-6,
            "expected 0.3 to be preserved, got {}",
            factor
        );
        // duration 應取 max（既有行為）
        assert!(
            entry.remaining >= fx(9.99),
            "expected duration ≥ 10, got {:?}",
            entry.remaining
        );
    }

    #[test]
    fn aggregation_family_sums_only_strongest_magnitude_per_stat() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(
            e,
            "cake_frosting:1",
            fx(2.0),
            json!({
                "__aggregation_family": "cake_frosting",
                "movespeed_bonus_percentage": -205,
                "incoming_damage_percentage": 256
            }),
        );
        s.add(
            e,
            "cake_frosting:2",
            fx(2.0),
            json!({
                "__aggregation_family": "cake_frosting",
                "movespeed_bonus_percentage": -512,
                "incoming_damage_percentage": 154
            }),
        );

        assert_eq!(
            s.sum_add(e, StatKey::MoveSpeedBonusPercentage),
            Fixed64::from_raw(-512)
        );
        assert_eq!(
            s.sum_add(e, StatKey::IncomingDamagePercentage),
            Fixed64::from_raw(256)
        );
        assert_eq!(s.move_speed_sums(e).percentage, Fixed64::from_raw(-512));
    }

    #[test]
    fn aggregation_family_multiplies_only_factor_farthest_from_one() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(
            e,
            "party:1",
            fx(1.0),
            json!({
                "__aggregation_family": "cake_party_haste",
                "attack_speed_multiplier": 1280
            }),
        );
        s.add(
            e,
            "party:2",
            fx(1.0),
            json!({
                "__aggregation_family": "cake_party_haste",
                "attack_speed_multiplier": 1280
            }),
        );

        assert_eq!(
            s.product_mult(e, StatKey::AttackSpeedMultiplier),
            Fixed64::from_raw(1280)
        );
    }

    /// 原作 dummy.lua:866-902：不同名稱的技能緩速 1−Π(1−sᵢ)；同名只算一筆；
    /// 攻速常數減速只取最強。Insertion order never changes the result.
    #[test]
    fn ability_slows_stack_multiplicatively_per_source_name_like_the_source() {
        let slow = |raw: i64| json!({"move_speed_bonus": raw, "__aggregation_family": ABILITY_SLOW_FAMILY});
        let rows = [
            ("generic_slow:alpha:1:1", slow(-307)),
            ("generic_slow:beta:2:1", slow(-205)),
            // Same source name from another caster: one entry, the strongest.
            ("generic_slow:alpha:3:1", slow(-256)),
            ("generic_slow:gamma:4:1", json!({(StatKey::AttackSpeedBonusConstant.as_str()): -30 * 1024, "__aggregation_family": ABILITY_SLOW_FAMILY})),
            ("generic_slow:delta:5:1", json!({(StatKey::AttackSpeedBonusConstant.as_str()): -50 * 1024, "__aggregation_family": ABILITY_SLOW_FAMILY})),
            ("boots", json!({"move_speed_bonus": 100})),
        ];
        let mut results = Vec::new();
        for reversed in [false, true] {
            let mut s = BuffStore::new();
            let e = ent(1, 1);
            let order: Vec<_> = if reversed { rows.iter().rev().collect() } else { rows.iter().collect() };
            for (id, payload) in order { s.add(e, id, fx(5.0), payload.clone()); }
            // 1024·(1024−307)/1024 = 717; 717·(1024−205)/1024 = 573 → slow 451; boots stay additive.
            assert_eq!(s.sum_add(e, StatKey::MoveSpeedBonus), Fixed64::from_raw(-451 + 100));
            assert_eq!(s.move_speed_sums(e).percentage, Fixed64::from_raw(-451 + 100));
            assert_eq!(s.checked_sum_add(e, StatKey::MoveSpeedBonus), Some(Fixed64::from_raw(-451 + 100)));
            assert_eq!(s.sum_add(e, StatKey::AttackSpeedBonusConstant), Fixed64::from_i32(-50), "attack slow: strongest only");
            results.push(s.sum_add(e, StatKey::MoveSpeedBonus));
            // A full slow stops the unit no matter what else stacks; expiry restores the product.
            s.add(e, "generic_slow:omega:6:1", fx(1.0), slow(-1024));
            assert_eq!(s.sum_add(e, StatKey::MoveSpeedBonus), Fixed64::from_raw(-1024 + 100));
            s.tick(fx(1.0));
            assert_eq!(s.sum_add(e, StatKey::MoveSpeedBonus), Fixed64::from_raw(-451 + 100));
        }
        assert_eq!(results[0], results[1]);
        assert_eq!(combine_ability_slows(&BTreeMap::new()), 0);
        // Positive or out-of-range members cannot speed up or over-slow.
        let mut s = BuffStore::new();
        let e = ent(2, 1);
        s.add(e, "generic_slow:up:1:1", fx(5.0), slow(512));
        s.add(e, "generic_slow:over:1:1", fx(5.0), slow(-4096));
        assert_eq!(s.sum_add(e, StatKey::MoveSpeedBonus), Fixed64::from_raw(-1024));
    }
}

const MOVE_SPEED_SUM_KEYS: &[&str] = &[
    "movespeed_absolute",
    "movespeed_base_override",
    "movespeed_bonus_equipment",
    "movespeed_bonus_percentage",
    "movespeed_bonus_percentage_unique",
    "movespeed_bonus_percentage_unique_2",
    "move_speed_bonus",
    "movespeed_bonus_buff",
    "movespeed_absolute_min",
    "movespeed_max",
    "movespeed_limit",
];
