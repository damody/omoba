# 將軍知識：節點說明文字 + buff 套用實作

**日期**：2026-07-20  
**狀態**：已核准，待實作

---

## 問題

將軍知識面板有兩個缺陷：

1. **UI**：節點卡片顯示 raw id（如 `dart_pierce_1`），沒有中文名稱或效果說明，玩家看不懂。
2. **功能**：`KnowledgeBonusResource` 在遊戲初始化時被填入加成資料，但 `spawn_td_tower_with_owner` 放塔時從未讀取它，解鎖的知識節點對遊戲實際上沒有任何效果。

---

## 設計

### 1. `knowledge_tree.json` — 新增 `label` 與 `description` 欄位

每個節點加兩個欄位：

- `label`：短中文名稱（顯示於卡片主標題）
- `description`：一行說明，包含效果描述與數值

全部 17 個節點：

| id | label | description |
|---|---|---|
| `dart_pierce_1` | 射程強化 I | 擴大飛鏢塔攻擊範圍 +50 |
| `dart_atk_1` | 攻擊強化 I | 提升飛鏢塔攻擊傷害 +5 |
| `dart_speed_1` | 攻速強化 I | 飛鏢塔攻擊間隔縮短 5% |
| `bomb_splash_1` | 爆炸強化 I | 提升炸彈塔爆炸傷害 +10 |
| `bomb_range_1` | 爆炸範圍 I | 擴大炸彈塔攻擊範圍 +30 |
| `bomb_atk_1` | 爆炸強化 II | 進一步提升炸彈塔爆炸傷害 +15 |
| `ice_slow_1` | 冰域強化 I | 擴大冰晶塔冰凍範圍 +20 |
| `ice_range_1` | 冰域強化 II | 大幅擴大冰晶塔冰凍範圍 +40 |
| `tack_atk_1` | 釘刺強化 I | 提升圖釘塔攻擊傷害 +3 |
| `tack_range_1` | 釘刺範圍 I | 擴大圖釘塔攻擊範圍 +20 |
| `boomerang_atk_1` | 迴旋強化 I | 提升迴旋鏢塔攻擊傷害 +8 |
| `boomerang_range_1` | 迴旋範圍 I | 擴大迴旋鏢塔攻擊範圍 +30 |
| `arty_atk_1` | 砲擊強化 I | 提升砲兵塔攻擊傷害 +20 |
| `arty_range_1` | 砲擊範圍 I | 擴大砲兵塔攻擊範圍 +50 |
| `hero_start_atk` | 英雄出擊 | 英雄初始攻擊傷害提升 +10 |
| `global_tower_atk_1` | 全域攻擊 I | 所有塔攻擊傷害提升 +2 |
| `global_tower_range_1` | 全域射程 I | 所有塔攻擊範圍擴大 +15 |

**向下相容**：`label` 與 `description` 均為 `#[serde(default)]`，舊格式 JSON 不會 panic。

---

### 2. `KnowledgeNode` struct（`omb/src/knowledge/loader.rs`）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: String,
    pub category: String,
    pub kp_cost: u32,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub bonuses: Vec<KnowledgeBonus>,
    #[serde(default)]
    pub label: String,        // 新增
    #[serde(default)]
    pub description: String,  // 新增
}
```

---

### 3. buff 套用（`omoba-core/src/runtime/native/game_processor.rs`）

在 `spawn_td_tower_with_owner` entity build 完成後，讀取 `KnowledgeBonusResource` 並套用對應 category + global 的 buff：

```rust
fn unit_id_to_gk_category(unit_id: &str) -> &str {
    match unit_id {
        "tower_dart"      => "tower_dart",
        "tower_bomb"      => "tower_bomb",
        "tower_ice"       => "tower_ice",
        "tower_tack"      => "tower_tack",
        "tower_boomerang" => "tower_boomerang",
        "tower_arty"      => "tower_arty",
        _                 => "",
    }
}
```

套用邏輯（spawn 後立即執行）：

```rust
let category = unit_id_to_gk_category(unit_id);
let gk = world.read_resource::<KnowledgeBonusResource>();
if gk.enabled && !category.is_empty() {
    let buffs: Vec<(String, String)> = gk
        .bonuses_for(category)
        .iter()
        .chain(gk.global_bonuses().iter())
        .cloned()
        .collect();
    drop(gk);
    let mut buff_store = world.write_resource::<BuffStore>();
    for (buff_id, payload) in &buffs {
        buff_store.apply_permanent(entity, buff_id, payload);
    }
}
```

`apply_permanent`：無持續時間的永久 buff，UnitStats 聚合時自動計入 `sum_add` / `product_mult`。

**注意**：`hero_start_atk` 的 category 是 `"hero"`，英雄不走 `spawn_td_tower`，需另外確認英雄 spawn 路徑是否需要獨立處理（本次 scope 以塔為主，英雄留 TODO）。

---

### 4. UI 卡片更新（`omfx/game/src/native.rs`）

**版面改動：**

```
CARD_H: 88.0 → 110.0
```

**卡片佈局（110px 高）：**

```
┌─────────────────────────────────────────────────────┐
│  射程強化 I                    需 3 KP               │  y+4，高 40px
│  擴大飛鏢塔攻擊範圍 +50         [ 解鎖 3KP ]         │  y+48，高 28px
└─────────────────────────────────────────────────────┘
  左 55%：上=label（font 20px），下=description（font 14px）
  右 40%：上=kp_info，下=unlock 按鈕
```

**struct 改動：**

- `GkPanelUi` 加 `node_labels: Vec<String>`、`node_descs: Vec<String>`
- `node_cards` tuple 由 5 項改為 6 項，加 `desc_t: Handle<Text>`

**JSON 讀取：**

載入時同時讀 `label` 與 `description`；若欄位不存在則 fallback 到 `node_id`（向下相容）。

---

## 變更範圍

| 檔案 | 變更類型 |
|---|---|
| `scripts/lua_data/knowledge_tree.json` | 加 `label` / `description` 到全部 17 個節點 |
| `omb/src/knowledge/loader.rs` | `KnowledgeNode` 加兩個 optional 欄位 |
| `omoba-core/src/runtime/native/game_processor.rs` | `spawn_td_tower_with_owner` 套用 GK buff |
| `omfx/game/src/native.rs` | 卡片 UI 顯示 label + description，CARD_H 調整 |

---

## 不在本次 scope

- 英雄 spawn 路徑的知識加成（`hero_start_atk` category）
- 知識節點 buff 在塔升級後的重新套用
- 知識樹的視覺化依賴關係圖
