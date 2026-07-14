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

use omb_script_abi::buff_ids::BuffId;
use omb_script_abi::stat_keys::StatKey;
use omoba_sim::Fixed64;
use serde_json::Value;
use specs::Entity;
use std::collections::{BTreeMap, HashMap};

const AGGREGATION_FAMILY_KEY: &str = "__aggregation_family";

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
fn read_fixed_from_payload(value: &serde_json::Value) -> Fixed64 {
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
}

/// 以 `Entity -> buff_id` 為 key 的 O(1) buff 索引。
/// `entities_by_key` 是 stat key → entity → 引用計數的反向索引，
/// 加速「哪些 entity 受某類 stat 影響」的查詢（regen / DoT 系統用）。
#[derive(Default, Debug)]
pub struct BuffStore {
    buffs: HashMap<Entity, HashMap<String, BuffEntry>>,
    entities_by_key: HashMap<String, HashMap<Entity, u32>>,
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

impl BuffStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// 新增或刷新 buff。若已存在：duration 取 max、payload 依 should_replace
    /// 策略決定是否覆寫——例如 slow 採單一 instance（buff_id = "slow"），
    /// 由 payload 的 `slow_factor` 欄位驅動「強蓋弱」比較（見上方 Reserved
    /// 負載約定）。
    pub fn add(&mut self, entity: Entity, buff_id: &str, duration: Fixed64, payload: Value) {
        let entity_buffs = self.buffs.entry(entity).or_default();
        match entity_buffs.get_mut(buff_id) {
            Some(e) => {
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
                    },
                );
                for k in &new_keys {
                    self.index_inc(entity, k);
                }
            }
        }
    }

    pub fn remove(&mut self, entity: Entity, buff_id: &str) {
        let _ = self.remove_entry(entity, buff_id);
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

    fn sum_add_key(&self, entity: Entity, key: &str) -> Fixed64 {
        let mut total = Fixed64::ZERO;
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
                total += value;
                continue;
            };
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
    }

    pub fn is_silenced(&self, entity: Entity) -> bool {
        self.has(entity, BuffId::Silence.as_str()) || self.has(entity, BuffId::Stun.as_str())
    }

    /// 倒數所有 buff 並回傳過期的 `(Entity, buff_id, payload)` 清單。
    /// 呼叫端可依 payload 內容決定是否廣播（例：payload 含 move_speed_bonus
    /// 表示這是移速影響類 buff，要發 creep/S 還原訊息）。
    /// 階段 1c.3：dt 為固定 64 秒。
    pub fn tick(&mut self, dt: Fixed64) -> Vec<(Entity, String, Value)> {
        let mut expired = Vec::new();
        // 先收集 expired，避免 retain 內動態借 self（index_dec 也要 &mut self）
        let mut to_drop: Vec<(Entity, String)> = Vec::new();
        for (e, entity_buffs) in self.buffs.iter_mut() {
            for (id, v) in entity_buffs.iter_mut() {
                v.remaining -= dt;
                if v.remaining <= Fixed64::ZERO {
                    to_drop.push((*e, id.clone()));
                }
            }
        }
        for (e, id) in to_drop {
            if let Some(entry) = self.remove_entry(e, &id) {
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
