# D4 選角名單（定案）

依據：[d3-requirements.md](d3-requirements.md)（由 `candidate-requirements.lua` 生成並驗證）。來源固定 `damody/nobugg_official@ca6ee8602473f6015a397cc121dac6baeb70b2e5`，全部使用 `current` 版。名單由 D3 狀態與換人規則推導，記錄日期 2026-10-25。

## 定案名單

| 計畫槽 | 來源 | 定位 | D3 狀態 | 需要的 G 包能力 |
|---|---|---|---|---|
| H01 | B01 雜賀孫市 | 射手 | needs_framework | G-2 `charges`、G-3 `summon_immobile` |
| H02 | B04 伊達政宗 | 近戰／位移 | needs_framework | G-1 `projectile_tracking`（W 的火焰彈） |
| H03 | A34 阿松 | 施法者／範圍 | needs_framework | G-1 `projectile_tracking`、`projectile_linear`；G-4 `cdr_on_cast_hook` |
| H04 | C17 阿市 | 支援／控制 | needs_framework | G-3 `extra_action_slot`（D 被動是第 5 個技能）；G-4 `tether_distance_scaled`、`on_cast_trigger` |
| H05 | A02 稻姬 | 切換／隱形 | needs_framework | G-2 `toggle_mana_per_attack`；G-4 `stealth_attack_no_reveal`、`no_unit_collision` |
| H06 | **B08 淺井長政**（換人，原 A13） | 近戰特殊（直線刀牙／瞬移／額外大招） | needs_framework | G-1 `projectile_linear`；G-3 `extra_action_slot`；G-4 `on_cast_trigger` |
| 盲測 1 | C15 玉子 | 遠程射手 | needs_framework | G-1 投射物、`retarget_on_death`；G-4 `ground_area_line` |
| 盲測 2 | B29 村上義清 | 近戰／控制 | needs_framework | G-2 `channel`；G-4 `aura` |
| 候補 | B28 松永久秀 | 施法者 | needs_framework | G-1、G-2 `channel`、G-4 `tether_distance_scaled` |
| 候補 | B09 真田昌幸 | 支援／切換 | needs_framework | G-2 `channel`、`delayed_effect`；G-4 `mana_shield`、`enemy_cast_extends_debuff` |
| 候補 | A04 竹中重治 | 召喚／施法者 | needs_framework | G-2 `channel`；G-3 `extra_action_slot`（D 被動） |
| 保留（不交付） | A13 服部半藏 | 特殊 | deferred | backlog `illusion`、`curving_hook`；另需 G-2、G-3、G-4 多項 |

所有候選都是 `needs_framework`，沒有任何一位能只靠現有能力完成；這是預期結果，也是 G 包要做的事。只有 D3 標 `supported` 的機制可以直接生成資料。

## 換人紀錄

1. **H06：A13 服部半藏 → B08 淺井長政**
   - 原因：A13W 的分身（`illusion`）與 A13E 可轉彎的鉤鎖（`curving_hook`）在 D3 判為 `deferred`。README §5 把分身列為「只有被選中才做」的完整機制，schedule §4.4 也規定 M3 之後不新增大架構。不能刪掉這兩招來湊數（規則 3）。
   - 為什麼是 B08：它是 H02／H06 的同定位候補。它需要的 G 包能力（`projectile_linear`、`on_cast_trigger`、`extra_action_slot`）都已被其他交付英雄需要，不會為了 H06 多開框架。
   - A13 保留在候選目錄，標 `deferred`，能力缺口列入 next-month backlog。
2. 其餘五位照草案，不換。

## G 包優先順序（由名單推導）

依「解鎖多少位交付英雄」排序；同時影響盲測或候補的放後面作次要考量。

1. **G-1 投射物**（`projectile_tracking`、`projectile_linear`）：解鎖 H02、H03、H06。必須讓直線彈命中英雄、支援穿透／停止、命中順序穩定，並處理可見性裁切。
2. **G-4 英雄 UnitScript 掛點**（`on_cast_trigger`，及以它為基礎的 `cdr_on_cast_hook`）：解鎖 H03、H04、H06 的被動。這是一個通用掛點：讓英雄註冊 UnitScript 的 `on_skill_cast`／`on_damage_taken`，不為個別英雄特判。
3. **G-2 複合狀態**（`charges`、`toggle_mana_per_attack`）：解鎖 H01、H05。
4. **G-3**（`summon_immobile`、`extra_action_slot`）：解鎖 H01、H04、H06。本遊戲 loadout 上限 4 個技能且被動也佔一格，所以來源有 D 技能（含被動）的 kit 都需要額外槽。額外槽要擴充「MOBA loadout 最多 4 技能」的限制（`moba_match.rs`、`omoba-content-model`），D／F 召喚師技能不受影響。
5. **G-4 其他**（`stealth_attack_no_reveal`、`no_unit_collision`、`tether_distance_scaled`）：解鎖 H04、H05。英雄隱形要接上既有的 `StealthProfile` 與偵測過濾，不另做一套。

最低線 4 套的最短路徑是 H01＋H02＋H03＋H06。它們只需要 G-1、G-2 `charges`、G-3 的兩項，以及 G-4 的英雄 UnitScript 掛點；H04、H05 另外還需要 tether 與隱形。

## 換人規則（定案）

1. 只有下列情況才能換人：
   - D3 判定某招為 `deferred`。
   - 某招判為 `needs_framework`，但該能力在 M3 前無法完成。
   - 某招判為 `ambiguous`，且使用者無法回答。
2. 依同定位候補依序替換：
   - 射手：C15 不可用，它是盲測。
   - 近戰：B08 已用於 H06，近戰沒有候補了。
   - 施法者：B28。
   - 支援：B09。
   - 召喚／切換：A04、B09。
3. 換人要寫進 execution-status.md 與本檔的換人紀錄，保留原英雄與原因。
4. 不刪技能；不把 `_old`／`_20` 變體當成另一位來源英雄；不改名充數。
5. 盲測英雄不得因為「好做」而與交付名單互換。盲測遇到 `needs_framework` 時，如實記為能力缺口。

## 假設政策

- 缺數值：採明確的 MVP 預設，標 `assumed`，寫入 package `assumptions`。例：B01E 召喚物數值沿用本地 `saika_gunner`。
- 缺語意：在缺失的 vscript 中、描述也沒寫清楚時，標 `ambiguous` 並集中詢問；不能用普通傷害替代。
- KV 與描述衝突：兩者都保留在 `kv_text_conflicts`。預設採用 **KV 數值＋描述語意**，並在 package 寫明採用理由。例：A34E 的主傷害取 KV。
- 本遊戲沒有的系統：三圍屬性與日夜視野換算成具體 StatKey，或標為不適用並記錄，不悄悄刪除。
- H01 照使用者決定，以來源 current 版重新規格化。本地 `B01_saika_magoichi` 只當程式起點，所有差異寫進 package `assumptions`。
