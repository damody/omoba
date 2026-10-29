# M6 前置：UE 通用機制需要的 runtime fact（工作線 C → B）

工作線 C（UE 通用機制與呈現）提出，2026-10-29。目的是讓 UE 用一套通用程式支援 M6 各批的機制，不必為個別英雄寫 C++ 特判。

- 下表列出 UE 需要、但目前 runtime 沒有提供的 fact，以及建議的欄位。執行器（工作線 B）可以調整名稱與編碼，回頭更新本文件即可。
- UE 端會先依本文件做好顯示與測試。欄位為 0 時代表「沒有」或「未知」，畫面維持現狀。
- 現況盤點：資料路徑是 core `Hero` → client runtime `moba_hud::project` → proto `AbilityHudPresentation` → bridge `OmMobaHud.abilities[5]` → UE `FOmAbilityHudStatePayload`。

## 1. 技能欄（每一格，`AbilityHudPresentation` 與 `OmAbilityHud`）

| 機制 | 建議欄位 | 現況 | 誰做 |
|---|---|---|---|
| 充能 | `max_charges`、`charges`、`charge_remaining`、`charge_total` | core、transport、proto、HUD schema 5 都已有，**bridge 漏接** | C：bridge 與 UE（ABI 40） |
| 開關 | `toggle_state`：0 未知、1 關、2 開 | 只有靜態的 toggle 類型 | B：core 產生；C：bridge 與 UE |
| 法球自動施放 | `autocast_state`：0 未知、1 關、2 開，只給持有者；另需切換自動施放的輸入命令 | 只有 core 的 `attack_orb:<id>` 標記 | B、C |
| 重施放窗口 | `recast_remaining`、`recast_total`、`recasts_left`、`recast_instance` | 沒有 | B、C |
| 限時換技能 | `catalog_id` 每一幀已經會送；另加 `original_catalog_id`、`swap_remaining`、`swap_total` | core 不會在出生後改 `Hero.abilities` | B、C |
| 隱藏或子技能 | `slot_flags` bit0 = 不顯示 | UE 只能隱藏第 5 格 | B、C |

- 時間一律用 Fixed64 raw，和既有的 `cooldown_raw` 相同；bridge 轉成秒。
- 重施放的 `recast_instance` 是單調遞增的編號，UE 用來判斷窗口換新。

## 2. 施法輸入

| 機制 | 建議 | 誰做 |
|---|---|---|
| 向量施法（兩點） | `CastAbility` 加第二個點 `target_pos_end`；技能 metadata 加 `vector_length`（由 v3 的 `target.vector_length` 產生）；bridge 的 `OmInputCommand` 加 `world_point_end`；UE 新增 `EOmGameplayInputTargetKind::Vector`（先點起點、再點方向） | B：協定與執行器；C：bridge、UE 輸入與預覽 |

### 2.1 向量施法協定提案（工作線 C，2026-10-29，待工作線 B 同意）

UE 端的輸入與預覽已完成：瞄準模型的兩段確認、catalog 的 `target.vector_length`、預覽線、取消。目前送出時在本地 fail closed，訊息是「向量施法的協定欄位尚未接上」，不會改成普通的地點施法送出。

| 層 | 欄位 | 說明 |
|---|---|---|
| proto `CastAbility` | `target_pos_end`（Vec2，Fixed64 raw，選填） | 只在技能的 `target.type == "vector"` 時出現；`target_pos` 是起點。執行器以 `normalize(end - start) * vector_length` 為方向與長度，不信任 client 的長度 |
| 驗證 | 起點在 `range` 內；終點與起點不同 | 起點超出範圍：照一般地點施法的規則（走近或拒絕）；沒有方向：拒絕，錯誤碼新增 `vector_direction_missing` |
| bridge C ABI | `OmInputCommand` 加 `world_point_end`（OmVec3）與 `flags` bit `OM_INPUT_HAS_POINT_END` | ABI 升一版；舊版 bridge 不認得這個 bit，UE 檢查 ABI 版本，不足時繼續 fail closed |
| UE 反射介面 | `SubmitOwnedAbilityCommand` 加 `PointEnd`（FVector）與 `bHasPointEnd` | `OmPlayerActionOwnerComponent::SubmitCastIntent` 已經有這兩個值（`FOmCombatAimIntent`） |
| 呈現 | `AbilityHudPresentation` 不變；`vector_length` 由 catalog 提供 | 不需要新的 runtime fact |

### 2.2 路徑型投射物與連鎖彈跳（UE 端）

- 預覽幾何已完成：`FOmPathPreview::Build`（路徑每一段的寬帶）、`ChainRings`（每一跳的搜尋範圍）。測試 `Om.Runtime.VectorCastPreview`。
- 等工作線 B 提供 §3 的 `path` 與 `ProjectileBounce` 後，bridge 解碼並交給這兩個函式畫出來；cue 的外觀依 cue 目錄的 kind（§6）。

## 3. 投射物與 cue

| 機制 | 建議 | 誰做 |
|---|---|---|
| 投射物外觀 | 投射物 entity 帶發射它的技能 `ability_catalog_id` 和 `cue_id`（v3 projectile 的 `cue`），並揭露飛行元件（motion、speed、radius） | B：揭露；C：bridge 解碼、UE 依 cue 呈現 |
| 路徑型投射物 | 揭露路徑點（`path`）或目前段落的終點 | B、C |
| 連鎖彈跳 | `PresentationCue::ProjectileBounce { from, to, hits_left, cue_id }`，bridge 轉成 `OmFxCue` | B、C |
| 持續區域 | 區域紀錄（中心、半徑、剩餘時間、隊伍、`cue_id`），取代只有一次的 APC1 | B、C |
| v3 `cue` 動作 | 執行器執行 `cue` 動作時送出呈現事件，帶字串 `cue_id`（bridge 的 `OmAbilityEvent.cue_id` 已有欄位） | B：送出事件；C：UE 依 cue 目錄的 kind 播佔位特效 |

## 4. 版本

- bridge 結構改變時，`OM_ABI_VERSION` 加一，並更新 bridge 測試裡所有寫死版本號的 assert 與 size 鎖定。
- proto 或 HUD 欄位增加時，依既有規則升 HUD `SCHEMA_VERSION`、`PRESENTATION_PROTOCOL_VERSION`、transport 版本，並在常數旁留一行說明。
- UE 已經會拒絕版本不符（`OmFrameContract.h`、`OmCatalogCompatibility.h`），舊版 fail closed。

## 5. 進度

| 項目 | 狀態 |
|---|---|
| 充能（bridge ABI 40、UE 技能格顯示） | **完成**（2026-10-29，E208）：bridge `OmAbilityHud` 加 4 個欄位；技能格左下角顯示剩餘次數，tooltip 顯示「次數：1/2，下一次 X 秒」 |
| 開關、自動施放、重施放、換技能、隱藏 | **UE 端完成**：`FOmAbilityHudStatePayload` 已有對應欄位（`ToggleState`、`AutocastState`、`Recast*`、`OriginalCatalogId`／`Swap*`、`SlotFlags`）、技能格左上角徽章、tooltip 行與隱藏邏輯，測試 `Om.Runtime.AbilitySlotStateHud`。**等工作線 B 提供 runtime fact**，之後由工作線 C 接 bridge（ABI 41）與 `OmWorldBridgeActor` |
| cue 佔位特效 | **完成**：見 §6 |
| 向量施法、路徑投射物、彈跳、持續區域、cue 事件 | 等工作線 B 確認欄位與排程 |

工作線 B 的排程（2026-10-29 回覆後更正，對照 m6-waves.md；欄位名稱照本文件，不改）：

| runtime fact | 時程 |
|---|---|
| v3 `cue` 動作送 ability event（`cue_id` 放 v3 cue id） | M3 第二批 |
| `toggle_state`、換技能 `original_catalog_id`／`swap_*`、投射物 cue | M5 完成前（基準元件，10 位上架英雄要用） |
| 法球 `autocast_state`／`orb_slot`、連鎖彈跳 | M6 第 9 批 |
| recast、向量施法 `target_pos_end` | 未排：waves 還沒有對應元件，資料端出現時再排 |
| 持續區域 | 未另外指定 |

每一批定下 proto 後，工作線 B 回寫 §1～3 並通知工作線 C，C 再接 bridge（每批升 ABI）與 `OmWorldBridgeActor`。

## 6. cue 目錄（UE 端）

- `scripts/gen_ue_cue_table.lua` 把 `docs/character_pipeline/presentation/cue-catalog.lua` 編成 `OmRuntime/Private/OmCueCatalogData.inl`（1080 個 cue、852 個原作資產）。cue 目錄重新產生後要跑一次；`scripts/tests/ue_cue_table_test.lua` 會檢查是否過期。
- `OmCue::Resolve(cue id)`：kind、原作資產路徑、佔位特效；`OmCue::CueForOriginalAsset(路徑)`：原作路徑 → cue id（不分大小寫與斜線方向）。
- Editor 覆寫：Project Settings > Om Presentation Cues（`UOmCueSettings`，存在 DefaultGame.ini）。
  - `CueOverrides`：cue id → Niagara 特效、音效、kind 或顏色；目錄沒有的 id 也可以新增。
  - `OriginalAssetCues`：原作路徑 → cue id，None 表示取消對應。
- 呈現：`FOmAbilityCuePayload.CueKind` 是目錄裡的 cue id 時（也就是 bridge `OmAbilityEvent.cue_id` 帶 v3 cue id），單位改播該 cue 指定的特效；沒有指定就播 kind 的通用佔位特效（環、圓盤或連線，依 kind 換顏色；sound 沒有畫面）。`cast`、`area` 等既有名稱不在目錄裡，維持原本的樣式。
- 工作線 B 只要讓執行器的 `cue` 動作送出帶 v3 cue id 的 ability event，UE 不必再改。
