# omfue 盤點：證據、缺口與本月範圍

盤點日期：2026-10-24。方法是實際閱讀主要入口、C++ 公開介面與實作、Lua generator／模板、Rust script ABI、近期驗收紀錄，以及指定來源倉庫。本次未啟動 Editor／對局、未重新建置或跑測試，也未逐一人工閱讀 97 套英雄全部技能。

主 repo 當時 HEAD：`334419b`；omfue HEAD：`aa609c2`。兩者都有工作樹增量，因此 HEAD 不能單獨代表本次看到的所有內容。執行時在 schedule.md A1 重新記錄來源狀態與 baseline。

狀態分級：**已見實作**、**部分／整合缺證**、**本輪未證實支援**、**本月新增目標**。找不到完整接受證據不等於程式完全不存在；舊文件的「待辦」也可能已被後續增量補完。

## 1. 與 LOL 類可玩產品的逐系統差距

| 系統 | omfue／正式共用層的證據入口 | 判斷與尚需完成 | 優先 |
|---|---|---|---|
| 啟動／模式 | `scripts/unreal_entry.lua` → `scripts/run_moba_session_ue.lua` → `scripts/run_moba_role_ue.lua`、`scripts/run_ue.lua`、`omfue/Config/DefaultGame.ini` | MOBA 正式入口已有；裸 Editor 仍預設 TD_1，需要清楚的 MOBA Play 設定與一致手冊。 | P0 |
| 選角／進場 | `OmHeroSelectionWidget`、`scripts/run_moba_pie_ue.lua`、selection workflows | 已有選角及同世界 PIE 路徑；須驗新增英雄自動出現、載入失敗、取消／重試與完整進場。 | P0 |
| 結算／再來一局 | `OmMatchSessionSubsystem`、`OmManagedMatchControl`、session coordinator | 已有能力與流程；不能假定所有啟動模式都支援。逐入口驗自然勝負、退出、下一局與清理。 | P0 |
| 移動／普攻／A／S／H | `OmPlayerActionOwnerComponent`、`OmOwnerControlInput`、`OmPlayerController` | 功能大量存在；原 checklist 仍缺真權威 ACK 配對接受，須驗攻擊前後搖、走打與連線回饋。 | P0 |
| 鏡頭／小地圖 | `OmRtsCameraPawn`、`SOmMinimap`、`OmMinimapModel` | 已有投影、改鍵、拖曳、縮放；要在自然對局驗邊界、focus、雙 DPI 與戰爭迷霧一致。 | P0 |
| 技能操作 | `OmCombatAimModel`、各 preview model、`OmCommandBarWidget` | 已有多種幾何／施法模式；新技能需生成對應預覽、rank／資源／狀態。瞬發直線不是有飛行時間的技能。 | P0 |
| 技能動作／特效 | `OmAnimatedUnitActor`、`OmCombatVisualSubsystem`、`OmAbilityCueStyle`、`OmProjectileCueStyle` | 已有動作與有界呈現；目前公開 cue style 偏固定形狀／顏色，需驗設計者能替換為專屬素材。 | P0 |
| HUD／名牌／可讀性 | `OmCommandBarWidget`、`OmUnitNameplateWidget`、HUD asset registry | 已有多次布局與層級修正；近期紀錄仍提及裁切與重疊。用真團戰找問題，不照舊截圖重做所有 UI。 | P0 |
| 商店／裝備 | `OmShopCatalogDecode`、shared shop、Lua moba_items | 近期盤點記載 19 項、配方與 undo 等；產品差距主要是內容深度、角色推薦及真交易回覆，不是從零做商店。 | P0 整合／P1 內容 |
| 成長／經濟 | ability runtime、moba_economy、match／shop 投影 | 等級、技能點、金錢、獎勵與賞金已有；缺完整平衡與不同來源 kit 對接。符文／天賦另列後續。 | P0 kit／P2 符文 |
| 地圖與推進 | `moba_maps.lua`、`moba_match`、`OmMapSurfaceActor`、`OmWorldPresentationActor` | 三路、塔、兵營、主堡與野區已有；要接受可辨識的路線、推進、勝負與鏡頭，不必本月重做地圖編輯器。 | P0 |
| 小兵／塔／野怪 | native moba_match 與近期 whole-game audit | 成長、塔皮、首塔、野怪回頭／分裂／獎勵已有增量；驗 UE 呈現與流程，細緻競技數值延後。 | P0 回歸／P1 平衡 |
| 視野／草叢／眼 | visibility、brush_vision、ward_world、UE filtered frame | 有視野／遮蔽與偵測基礎；需要新英雄投射物／召喚／隱形在真多 client 下不洩漏。任意多邊形 authoring 延後。 | P0 |
| 召喚師技能 | moba_summoners、summoner_world、HUD | 已有五種召喚師法術及 ward 生命期（根目錄 README）；來源 D/F 的英雄技能不得與這些 slot 混為一談。 | P0 slot／P1 內容 |
| bot | `omoba-core/src/runtime/native/moba_match/bots`、line-bot 近期接受 | 已有十席配裝及部分技能決策；只讀已披露狀態，送正式 `MoveTo`／`CastAbility`／`UpgradeAbility`／`ItemBuy`，含友方治療／淨化與道具策略。是否能使用新 kit、購物與完成整局須驗。 | P0 |
| 召喚控制 | summon-priority 最新接受、script world adapter | 有 spawn／歸因／移動基礎；記錄明示玩家控制 UI 仍 pending。 | P1，選定 kit 必需則 P0 |
| 美術工作流 | `AOmAnimatedUnitActor` EditAnywhere 欄位、asset recipe、import／retarget commandlet | 具備可重用基礎；人工覆寫與生成預設的優先序、全 kit editor-only 操作、cook 保存需要完整契約。 | P0 |
| 音效／字幕 | `OmAudioPresentationComponent`、既有 PCM／voice 接受 | 具備共用機制；要增加技能槽位與混音／優先級，人工聽感不能只看 PCM 檔存在。 | P1 |
| 聊天／ping／mute | communication models 與原 checklist | 有本機與部分 transport 工作；要驗真實雙人來回與隱藏目標規則。語音通話服務不列本月。 | P1 |
| 多人／重連 | runtime bridge、client runtime、role/session workflow | 有權威／replica 基礎，但整體接受留真 LAN／重連缺口；不是已完成線上產品。 | P0 |
| 封裝／可分發 | Config packaging 欄位、已有 staging workflows | 部分 stage/cook 設定存在；本次未找到足以宣稱乾淨 packaged MVP 的接受，需明確測試。 | P0 |
| 效能／穩定性 | Lua performance／fixed-step report、現有有界 HUD／VFX | 測試工具存在；不能把分段 synthetic 測試當整局效能。固定硬體與長 session 尚需接受。 | P0 |
| 帳號／配對／排位 | 本輪未作服務端產品完整審計 | 與本機／LAN 可玩 MVP 不同；完整帳號、配對、排名、戰績服務不在月內交付。 | P2 |
| 觀戰／重播產品 | deterministic／paired replay 的測試基礎 | 測試 replay 不等於使用者可操作的觀戰／回放 UI；另列後續。 | P2 |
| 新手教學／練習場 | training 英雄／地圖與現有操作 UI | 本月做短操作說明與技能預覽場；完整互動教學、任意訓練工具後續擴充。 | P1 |
| 無障礙／本地化 | 改鍵、字體、色盤、字幕與合成 gamepad | 中文介面與主要解析度要可用；實體控制器／更多語系不冒稱已接受，非本月核心可延後。 | P1/P2 |

## 2. 最重要的具體發現

### A. Editor 預設與 MOBA 路徑分離

`omfue/Config/DefaultGame.ini` 的 `RuntimeMode=SinglePlayer`、`StoryId=TD_1`；`scripts/unreal_entry.lua` 則明確支援 `run_ue.bat --moba`，無進階參數時帶入 interactive selection。需要的是把既有 MOBA 路徑做成使用者看得懂、Editor 裡找得到的入口，而不是宣稱 MOBA 啟動完全不存在。

`scripts/run_moba_pie_ue.lua` 已有選角到 native result 的 owner 與 timeout 規則；主動重用它，避免再造一個不相容的測試啟動器。先確認它適用的實際條件，不把 smoke selection 當真人選角接受。

### B. 美術屬性已存在，但製作體驗尚未封口

`OmAnimatedUnitActor.h` 已公開 NativeMeshAsset、材質、附掛、NativeAnimations、state slots、fallback policy 等欄位。其 `.cpp` 在初始化時處理已有 skeletal mesh，且目前存在同步載入；需用真實多英雄進場觀察載入停頓與覆寫行為，不能直接推論必須重寫資產系統。

`scripts/ue_apply_asset_recipe.lua` 有預檢、ledger、idempotence 與 Editor 操作；`scripts/ue_asset_recipe_plan.lua` 規劃 mesh／texture／portrait／animation。可在此基礎上補齊完整英雄外觀 profile 與人工設定保護。

### C. 正式 UE 生成工具仍依賴歷史目錄

`omfue/codegen/src/lib.rs` 的 asset recipe 使用 `recipe_source("omfx", &hero.portrait)`。根目錄 README 也說明英雄 `portrait` 寫成 `data/hero_portraits/...`、實際從 `omfx/data/` 讀取，技能 fallback 為 `data/ability_icons/ability_default_placeholder.png`；`scripts/tests/ue_art_swap_editor_test.lua` 的替換圖來自 `omfx/data/hero_portraits/hero_default_portrait.png`。

本月只修改正式消費端與素材來源：改為 omfue 或共用 authoring 擁有的路徑，選擇／複製必要受控素材並驗 hash／引用；**不維護、不編譯、不刪除 omfx**。增加 maintained-source 依賴檢查，排除純歷史文件，確保 omfx 未 checkout 時也能走正式生成／驗收。

### D. 英雄模板量不等於獨立英雄量

`scripts/base_content/src/heroes/mod.rs` 顯式包含雜賀孫市與伊達政宗；其他訓練英雄可沿 generic effects，不應因沒有專用 `.rs` 就判定無技能。

`scripts/lua_data/templates/heroes.lua` 有 apprentice、support、scatter、line 等訓練／配裝變體；近期盤點的「九個英雄配裝」不能當成九個完整來源英雄。正式報表要分來源英雄、kit 版本、角色配裝、可玩接受狀態。

### E. 來源資料不能無腦複製

指定 repo 有 97 個主英雄 KV 檔，另有 variant／舊版；來源 B02 與本地 B02 模組名稱不一致。網站的顯示處理還會回填 `_old` 文字，匯入器要保留「實際原文」與「回填來源」差別。

`AbilityScript` 具 execute／on_tick／on_learn／on_attack_hit；`UnitScript` 有多種事件 hook。這是好基礎，但 ABI 有方法不代表所有正式 host／replica 路徑都完整派發，也不代表複合技能狀態、通用 projectile 及召喚 UI 已完成。

### F. 舊文件不能當當前真相

`omfue/README.md` 前段仍大量記載 2026-05 的 TD／Blueprint 基線；2026-10 的玩法與呈現已大幅增加。本月應更新正式入口文件，將歷史移到可查閱位置，而非讓 Grok 按五月待辦重做。

`docs/plans/2026-10-21-whole-game-completeness-audit.md` 有最新增量與舊「pending」混合；例如不能單憑早期段落就斷言引導／zones 未做。必須交叉查最新來源與該功能 final-acceptance。

## 3. 原有 11 項整合缺口必須追蹤

來源：`openspec/changes/complete-moba-controls-and-presentation/tasks.md` 的未勾選主項。部分功能已有後續子項接受，以下不是要求全部重寫，而是補足各主項真正缺的證據／接線。

| 原 ID | 未關閉範圍 | 對應工作（schedule.md） |
|---|---|---|
| 3.1 | world move／attack、節流與權威 ACK | C1、B1–B2、M1、J1–J2 |
| 3.3 | AttackMoveClick／MoveOnly／AttackOnly 的真事件與權威行為 | C1、B2、M1、J1–J2 |
| 3.4 | Stop／Hold 區別、repeat 與 queue | C1、B2、M1 |
| 3.5 | 只選英雄、auto attack、hidden target | C1、B2、M1、J1–J2 |
| 3.9 | ward／trinket 的完整 action、fog 與生命期 | C4、B3、J1–J2 |
| 4.3 | 真 move／attack／cast 成敗、視覺 feedback | C1、C4、M1、J1–J2 |
| 4.4 | 商店權威交易與 undo receipt | C4、B3、M1、J1–J2 |
| 4.7 | HUD／minimap／字體／色盤與真 DPI | C3、J4 |
| 7.1 | actual input→camera／command ACK→UI，native 與 BP 入口 | M1、C6、J1–J2 |
| 7.3 | resolution／DPI／window mode、focus、PIE、重連 | J4、J1、M4 |
| 7.4 | 真對局／LAN／固定效能（舊預算限制已由使用者 2026-10-25 解除） | M1–M4、J2、J5 接受 |

每項單獨填實際 runtime 證據。不以 L1/L2 的成功畫面去代替 L3/L4，不用計畫日期或文件數量宣稱原 change 已完成。

## 4. 本輪沒有做的驗證

- 沒有觀看當前真實遊戲畫面，因此 HUD／美術品質判斷引用已存在的紀錄與程式結構，不能當最新目視接受。
- 沒有執行 fresh cargo／Unreal build，沒有重新確認現有 dirty work 能否通過。
- 沒有完成全來源技能語意覆蓋統計；97 是檔案索引統計。全量粗分類與候選逐招能力矩陣在 schedule.md D2–D3。
- 沒有測真雙機 LAN、DPI、長 session 或打包。
- 本輪 `openspec` 指令未在目前 shell 中找到；仍可讀取現有 artifacts。執行者需確認既有工具安裝方式，不能以 CLI 不可用為由忽略現有 specs，也不在此輪擅自重建工作流。

這些缺證都放入排程。文件盤點完成不等於遊戲 MVP 已完成。
