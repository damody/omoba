# C 線交接（2026-11-09，依使用者停工指示）

## 已完成的 commit（都沒有 push）

| 內容 | omfue | 主 repo（含 gitlink） |
|---|---|---|
| B 的 UnitOrPoint（omfue 部分，代 commit） | 1b7c25c1 | 4878fd06 |
| EntityOrPoint：點到單位送 Entity、點地面送 Point；codegen 改為 "EntityOrPoint" | 36ad47f3 | bf3ab9fc |
| EntityOrPoint 的 CommandBar 部分：巨龍 hotbar 可用、目標文字 | 940e62a5 | 88591f8e |
| 信使模型 courier_horse（prop_meshes.lua；5 個 DShader 材質 .dsi） | d88e98b4 | 84e8fb33 |
| 信使外觀（summons.lua）、bridge 的 OM_ENTITY_FLAG_COURIER、小地圖馬蹄形標記、UE 測試檔 | 5267c281 | 4511686e |

驗證狀態：
- EntityOrPoint：UE 測試 24 項成功。巨龍兩條分支都在實機確認過，紀錄在 target/lineC-heroes/dragon-ally-6 和 dragon-ground-6：點友方單位會衝刺 600 再吐龍息，點地面直接吐龍息。
- 信使這批：land_to_live 已通過 codegen、cargo、bridge 與 UBT。header 與 cbindgen 產生的一致，bridge Rust 測試通過。

## 未完成

1. **信使 UE 測試 `Om.Runtime.CourierPresentation` 還沒真正跑過。**
   - live 的 UE binary 是 11:10 建的，早於這批 land，裡面沒有這個測試。
   - 影子 `target/ue-shadow-c-eop` 沒有 Content，連既有的 PropMesh 測試也會失敗，不能用來驗資產。
   - 要等 live 用 `build_ue_moba.lua --build-only` 重建之後再跑。我排隊等鎖的那次建置已經取消。
2. **實機看信使。**
   - E 修好「信使只認 Summon」的五處（snapshot.rs 兩處、visibility.rs、summon_orders.rs、filtered_specs.rs）之前，信使不會以 (Summon, "courier") 出現，所以看不到馬，也收不到指令。
   - 修好後，開一局 MATCH_SHOP_RANGE_RULE=true、role plan 的 shop_range_rule=true，確認畫面上有馬、小地圖有馬蹄形標記，選取後有「送貨」「回家」兩個按鈕。
3. **商店儲藏處 6 格**：等 B 的 or21（OwnerStash、StashFull=17）。出售等 S2 的 item_slot 6..11。版面在 lineC-courier-ue-prep-2026-11-09.md §3。
4. **7 個 unit_or_point 技能補錄（尚未開始）**：
   - a06_flame_of_anger（ii_naomasa）
   - a32_asura_blade（takigawa_kazumasu）
   - c10_black_dragon_wave（kosokabe_chikayasu）
   - b08_old_fang_slash、b08_old_sky_fang（asai_nagamasa_old）
   - b04_old_flame_blade（date_masamune_old）
   - a14_old_fire_phoenix（maeda_toshiie_old）

   每個都要錄點敵方單位、點地面兩段。A 說 or22 只改 UE 的目標指示，遊戲結果不變。
5. **巨龍點友方的 WARN**：B 的 or18 land 後，重跑 dragon-ally，確認 server log 沒有 "ItemUse failed … target is not visible"。
6. **千里大劍**上架後，若 autopilot 不支援 ally_anchor_unit（unit 目標），要補上。

## 影子與路徑

- `target/ue-shadow-c-eop`：EntityOrPoint 和巨龍驗證用的影子。最後一次在 06:3x 做了 `--sync --build`。沒有 Content。
- 實機跑的方式：
  - 設 `OM_UE_RUN_BINARIES=D:/omoba/target/ue-shadow-c-eop`。
  - base_content 用影子自己 stage 的 `Plugins/OmRuntime/Binaries/Win64/base_content.dll`。
  - 每次跑之前都要重新 stage，因為 content hash 會變。
- 驅動腳本在 session scratchpad：`uetest/combat_aim_shadow.lua`、`dragon/{ally,ground}/`。

## 已知問題

- **舊影子 `target/ue-shadow-c` 有 20 個 11-02 的未同步編輯**，所以 `stage_ue_shadow --sync` 拒絕同步。我沒有動它，另外開了 ue-shadow-c-eop。這些編輯要先確認是否已經放回 live，才能清掉或重新同步。
- **截圖曝光**：Om.Runtime.PropMesh 的截圖明顯偏亮，金錢袋也一樣，只能比較形狀，顏色要在遊戲裡判斷。
- **prop 重存**：OmPropMesh commandlet 每次都會重存 gold_bag 和 hex_critter 的 SM。產生 courier_horse 之後我用 git checkout 還原了這兩個檔。之後再跑時同樣要還原。
- **courier_horse 是第一版。** 截圖相機在左後上方，頭和脖子被壓短。鞍布是中性的靛藍，沒有隊伍色，因為 prop commandlet 一個槽只會產生一個材質。
- **ue_binary_preflight.require_ready 說 live 已就緒**，但 live 的 OmEditor.dll 其實早於最新的 land。它可能沒有檢查 UE 模組原始碼的時間。
