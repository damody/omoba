# General Knowledge — 將軍知識節點一覽

資料來源：`scripts/lua_data/knowledge_tree.json`
Runtime buff key 格式：`gk_<node_id>`（duration = -1，永久 toggle）

---

## tower_dart — 飛鏢塔

| 節點 ID | KP 費用 | 前置節點 | Buff Key | 加成內容 |
|---|---|---|---|---|
| `dart_pierce_1` | 3 | — | `gk_dart_pierce_1` | `range_bonus +50` |
| `dart_atk_1` | 4 | `dart_pierce_1` | `gk_dart_atk_1` | `bonus_damage +5` |
| `dart_speed_1` | 5 | `dart_atk_1` | `gk_dart_speed_1` | `attack_speed_multiplier ×1.05` |

---

## tower_bomb — 炸彈塔

| 節點 ID | KP 費用 | 前置節點 | Buff Key | 加成內容 |
|---|---|---|---|---|
| `bomb_splash_1` | 4 | — | `gk_bomb_splash_1` | `bonus_damage +10` |
| `bomb_range_1` | 3 | `bomb_splash_1` | `gk_bomb_range_1` | `range_bonus +30` |
| `bomb_atk_1` | 5 | `bomb_range_1` | `gk_bomb_atk_1` | `bonus_damage +15` |

---

## tower_ice — 冰晶塔

| 節點 ID | KP 費用 | 前置節點 | Buff Key | 加成內容 |
|---|---|---|---|---|
| `ice_slow_1` | 3 | — | `gk_ice_slow_1` | `range_bonus +20` |
| `ice_range_1` | 3 | `ice_slow_1` | `gk_ice_range_1` | `range_bonus +40` |

---

## tower_tack — 圖釘塔

| 節點 ID | KP 費用 | 前置節點 | Buff Key | 加成內容 |
|---|---|---|---|---|
| `tack_atk_1` | 3 | — | `gk_tack_atk_1` | `bonus_damage +3` |
| `tack_range_1` | 3 | `tack_atk_1` | `gk_tack_range_1` | `range_bonus +20` |

---

## tower_boomerang — 迴旋鏢塔

| 節點 ID | KP 費用 | 前置節點 | Buff Key | 加成內容 |
|---|---|---|---|---|
| `boomerang_atk_1` | 4 | — | `gk_boomerang_atk_1` | `bonus_damage +8` |
| `boomerang_range_1` | 3 | `boomerang_atk_1` | `gk_boomerang_range_1` | `range_bonus +30` |

---

## tower_arty — 砲兵塔

| 節點 ID | KP 費用 | 前置節點 | Buff Key | 加成內容 |
|---|---|---|---|---|
| `arty_atk_1` | 5 | — | `gk_arty_atk_1` | `bonus_damage +20` |
| `arty_range_1` | 4 | `arty_atk_1` | `gk_arty_range_1` | `range_bonus +50` |

---

## hero — 英雄

| 節點 ID | KP 費用 | 前置節點 | Buff Key | 加成內容 |
|---|---|---|---|---|
| `hero_start_atk` | 5 | — | `gk_hero_start_atk` | `bonus_damage +10` |

---

## global — 全域（所有塔 + 英雄）

| 節點 ID | KP 費用 | 前置節點 | Buff Key | 加成內容 |
|---|---|---|---|---|
| `global_tower_atk_1` | 8 | — | `gk_global_tower_atk_1` | `bonus_damage +2` |
| `global_tower_range_1` | 8 | `global_tower_atk_1` | `gk_global_tower_range_1` | `range_bonus +15` |

---

## 加成機制說明

- **`bonus_damage`**（加法）：與其他 `bonus_damage` buff 疊加後，加到基礎攻擊力上。
- **`range_bonus`**（加法）：與其他 `range_bonus` buff 疊加後，加到基礎射程上。
- **`attack_speed_multiplier`**（乘法）：0.05 代表攻擊間隔縮短 5%（`asd * (1 - 0.05)`），需確認 host 端聚合方向。
- **CHIMPS 模式**：所有 `gk_` buff 不套用（`KnowledgeBonusApplySystem` 直接 return），KP 仍正常發放。
- **Buff duration**：`-1`（永久 toggle，不被 `clear_expired_buffs` 清除）。
