# work-reject-reason：玩家看得到操作為什麼被拒

```text
日期：2026-10-25
目前工作 ID／狀態：MVP 待辦「拒絕原因可見」／bridge 已著陸並通過測試；UE 原始碼已著陸，尚未編譯（需主 session 跑 UE 建置與 Editor 測試）
玩家或作者新增能做到的事：施法或升級被權威規則拒絕時，左上角的指令回饋行會顯示「施法指令遭拒絕：冷卻中」這類原因，不再只寫「遭拒絕」。
repo HEAD／本次修改：未 commit；著陸時間 2026-10-25 16:16（split13 已在 15:14 起跑）
已執行命令、exit code、build identity：見「測試」
L1／L2／L3／L4 證據：L1 fresh（bridge）；UE Editor 測試已撰寫但未執行；L3 未執行
下一個工作 ID：主 session 跑 build_ue_bridge／UE 建置與 Editor 測試，之後實機確認
owned process／資源清理結果：未啟動任何 process；cargo 只寫 target/session-ward 與 target/session-ward/shadow-target
```

## 資料流

runtime／replica 的 `command_outcome` 拒絕碼（`CriticalInputResult.result_code`，見 work-ack-result.md）
→ bridge `publish_presentation_input_result`：`OmAppliedInput.reason_code = rejection_code_number(result_code)`（只在 `accepted=false` 時有值；accepted、商店、undo、ward 的結果都是 0；未知字串也是 0，不會猜）
→ UE `AOmWorldBridgeActor` 把 `Input.reason_code` 傳進 `UOmCommandBarWidget::NotePlayerActionResult(..., Reason)`
→ `FOmActionFeedbackTracker::Result(..., Reason)`：Reason 為 0 時維持原本的「X指令遭拒絕」；已知碼顯示「X指令遭拒絕：<原因>」；其他非零值顯示通用原因「無法執行」。
→ 顯示在既有的 `OmActionFeedback` 文字列（畫面左上的指令回饋行）。

replica 不能判定的 host-only 碼照 E046 的規則處理：非權威 world 只會回報 owner 自己英雄狀態判定得出的碼，所以 UE 不會顯示假的「超出射程」或「無法施放」。

## CABI

- `OM_ABI_VERSION` 36 → 37；新常數 `OM_REJECTION_REASON_COUNT = 18`（cbindgen 匯出）。
- `OmAppliedInput` 在 `status` 之後加 `uint32_t reason_code`：大小 40 → 48，`reason_code` 的 offset 是 40（Rust 測試與 header smoke 都有斷言）。
- 已更新的 pin：bridge 單元測試 8 處（`OM_ABI_VERSION,37`）、`summon.rs`（36 現在也會被拒絕）、`smoke/om_bridge_header_smoke.cpp`（原本停在 35，見 E048）、UE `OmSummonInputTests`（Current ABI37）。
- staged header `Plugins/OmRuntime/Source/ThirdParty/OmBridge/include/om_bridge.h` 已換成 cbindgen 由新原始碼產生的版本，和舊 header 的差異只有本次新增的內容。

## 原因碼對照（`OmRejectReason.cpp`，順序與 `command_outcome::REJECTION_CODES` 相同，只能往後加）

| 碼 | runtime | UE 顯示 |
|---|---|---|
| 1 | INVALID_SLOT | 無效的技能欄位 |
| 2 | NO_HERO | 沒有可操作的英雄 |
| 3 | ABILITY_NOT_BOUND | 此欄位沒有技能 |
| 4 | NOT_LEARNED | 尚未學習 |
| 5 | ON_COOLDOWN | 冷卻中 |
| 6 | NOT_ENOUGH_MANA | 法力不足 |
| 7 | SILENCED | 受到沉默或控制 |
| 8 | OUT_OF_RANGE | 超出射程 |
| 9 | INVALID_TARGET | 目標無效或不可見 |
| 10 | ALREADY_CASTING | 施法中 |
| 11 | GAME_NOT_ACTIVE | 對局尚未進行 |
| 12 | HERO_INACTIVE | 英雄無法行動 |
| 13 | CAST_REJECTED | 無法施放 |
| 14 | SKILL_POINTS | 沒有技能點 |
| 15 | MAX_LEVEL | 已達最高等級 |
| 16 | LEVEL_REQUIRED | 英雄等級不足 |
| 17 | UPGRADE_REJECTED | 無法升級 |
| 18 | NO_CHARGE | 沒有可用次數 |
| 其他 | — | 無法執行（通用） |

文字使用 `LOCTEXT`（namespace `OmRejectReason`），可以進在地化流程。

新增碼時要同時改三個地方：`REJECTION_CODES` 往後 append、bridge 的 `OM_REJECTION_REASON_COUNT`（bridge 測試會比對兩者長度）、`OmRejectReason.cpp` 的 case（UE 測試會檢查 1..COUNT 每個碼都有專用文字）。

## autopilot

`OmAutoPilot.cpp` 的 authority trace 多了 `"reason_code":N`，放在 `status` 之後。修改前已讀過現況，只改了這一行 Printf 和它的參數。

## 修改檔案

- bridge：`src/lib.rs`、`src/driver.rs`、`src/projection.rs`、`src/summon.rs`、`cbindgen.toml`、`smoke/om_bridge_header_smoke.cpp`
- UE：新增 `OmRuntime/Public/OmRejectReason.h`、`OmRuntime/Private/OmRejectReason.cpp`；修改 `OmActionFeedbackTracker.h/.cpp`、`OmCommandBarWidget.h/.cpp`、`OmGenerated/Private/OmWorldBridgeActor.cpp`、`OmRuntime/Private/OmAutoPilot.cpp`、`OmEditor/Private/OmActionFeedbackTests.cpp`（新測試 `Om.Runtime.RejectReasonText`）、`OmEditor/Private/OmSummonInputTests.cpp`、`ThirdParty/OmBridge/include/om_bridge.h`

## 測試

| 命令 | exit | 結果 |
|---|---|---|
| shadow：`cargo test --manifest-path omfue/bridge/Cargo.toml --lib` | 0 | 146 passed、1 ignored |
| 著陸後的實際樹：`cargo test --manifest-path omfue/bridge/Cargo.toml --lib` | 0 | 146 passed、1 ignored。新增 `rejection_reason_numbers_match_the_runtime_codes`（數量與 runtime 一致、編號逐一對應、ABI 37、結構大小 48）；`ipc_input_waits_for_terminal_result...` 擴充：INVALID_TARGET 對到 9、ON_COOLDOWN 對到 5、未知碼對到 0 |
| shadow：`cargo test --manifest-path omfue/bridge/Cargo.toml`（含 integration test） | 101 | `tests/runtime_driver_smoke.rs` 本來就編不過：ABI36 新增的 `summon_command` 沒有補進這個檔案，與本修改無關（E048） |
| UE Editor：`Om.Runtime.RejectReasonText`、`Om.Runtime.ActionFeedbackCorrelation`、summon ABI37 | 未執行 | 依指示不跑 UE；需要主 session 建置後執行 |

## 主 session 需要做的

- 重跑 `build_ue_bridge.lua`：會重新產生 header（應該和已著陸的版本一致）並 stage DLL；UE 與 bridge 必須同時是 ABI37。
- UE 建置，並執行上面三個 Editor 測試。
- 實機確認：在冷卻中按施法，左上角應出現「施法指令遭拒絕：冷卻中」；autopilot 的 trace 會帶 `reason_code`。

## split15 追加（2026-10-25 17:08 著陸）

實機證據（`target/mvp-a3/split15`）判讀：

- p6 的 `cast_r_target` 到了 server 是 `CastAbility ability_index=2`（tick 5234、7542、9008、11112、11143、31269、34056……），也就是 `lumen_lance`，不是 3（`lumen_tremor`）。`cast_w_target` 到了 server 是 index 0（`lumen_bolt`）。所以 lumen_tremor 從來沒被施放，server log 沒有 tremor 的 AbilityCast 是正確的。這是 UE 端「按鍵 → 技能欄位」對應的問題（p6，training_channeler），不在 runtime 範圍。
- tick 9182 的拒絕原因 5（冷卻中）對應的是實際送出的 lumen_bolt，那時它確實還在冷卻（6.67 秒），所以原因碼沒有錯。另外說明：引導期間施放其他技能，規則是打斷引導（等同新指令），不會回「施法中」。ALREADY_CASTING 只適用於 windup。
- 引導技在施放成立的 transaction 就進入 CD，所以 autopilot 用 cooldown>0 判斷生效對引導技也成立。
- 真正的缺口是 lance：這是 unit 技能，送出時沒帶目標，handler 回「requires an entity target」，renderer 卻收到 accepted，因為 replica 不能判定 handler 失敗。

修正：

- 新碼 `TARGET_REQUIRED`（第 19 號）：admission 時如果 unit 技能沒有 `target_entity`，或 point 技能沒有 `target_pos`，就在 in-flight 上記下這個問題。施法仍照常排入（維持「admission 照收、settlement 決定」的既有契約）；之後若真的失敗，就回報 TARGET_REQUIRED。這只看輸入本身，所以 replica 也可以回報。有帶目標但目標無效的情況，replica 仍然不判定。
- bridge `OM_REJECTION_REASON_COUNT` 改為 19（ABI 仍是 37，結構沒變）；UE 對照表新增 19「需要指定目標」；smoke 與 Editor 測試的數量改成 19。
- 引導 log（info 等級）：`AbilityChannel start pid= caster= ability= level= pulses= interval_raw= anchor= tick=`、每一波 `AbilityChannel pulse ... pulse=i/N tick=`、結束 `AbilityChannel end ... pulses=i/N reason=`。reason 可能是 completed、stunned、silenced、moved、dead、recall、inactive、anchor_lost、cancelled（新指令或施放其他技能）、reset（比賽重設）。

測試（shadow，以最新的 live omoba-core 為基準）：omoba-core lib 1139 passed（新增 `a_missing_unit_target_is_reported_by_input_shape_in_every_world`、`end_reasons_name_the_interruption_for_live_logs`）、client 161、bridge 146（含 integration 2）。著陸後 live 樹上 base_content 的 channel／headless cast／mana 子集 9 passed。

需要重建：omobab、omoba-client-runtime、bridge（header 只有 COUNT 改變）、UE（OmRejectReason.cpp、Editor 測試）。base_content.dll 不需要。
