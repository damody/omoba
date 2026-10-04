# Unreal B 綁定回城：60Hz 雙隊驗收

## 範圍與決定

- 保持 60Hz 權威模擬。回城規則／傷害中斷／基地傳送仍由 Rust 與 Lua 生成的 8 秒規則決定，UE 不寫生命、位置或遊戲計時器。
- 測試直接執行 InputComponent 唯一 B / IE_Pressed 綁定的 delegate，再經正常反射 callback、通用輸入 adapter、TCP renderer IPC、runtime 驗證、KCP authority、team-private HUD 與畫面返回。不是 OS／實體鍵盤注入；也不是 runtime 內部 intent injection。
- 第一次 B 開始讀條，120 ticks 後送正常 MoveTo 取消；等待新位置後第二次 B，驗證權威完成與自己的公開基地端點相符。每次提交只嘗試一次，拒絕／錯 kind／錯 owner／錯 HUD 進永久失敗狀態，不換 ID 重試掩蓋錯誤。
- 使用 opt-in `OMOBA_UE_RECALL_SMOKE=1`，限定 bounded single-lane 60Hz，與其他 smoke 模式互斥。launcher 明確關閉 `OMOBA_RECALL_SMOKE`，並同 profile 建置 scripts DLL、server、runtime。
- 取消與完成分別要求原始 ID ACK；ACK 只代表套用，不代表讀條或完成。原生 Slate 文字必須等於權威 remaining，完成後至少 120 ticks 的三方 hash 才能通過。

## 首輪完整 gameplay 證據

保存於 `target/interactive-runs/interactive-ue-1791073690`：

| 隊伍 | B1 / ACK | 取消 Move / tick | B2 / ACK | 完成 tick / 基地 |
| --- | --- | --- | --- | --- |
| 1 | 2 / 3212 | 3 / 3338 | 4 / 3360 | 3840 / (0, 0) |
| 2 | 2 / 3212 | 3 / 3339 | 4 / 3361 | 3841 / (2400, 0) |

- 每隊恰好兩個 B callback、各一次原始 Recall forwarding；無 runtime recall injection。
- 三方 hash 各 68 PASS，最後 checkpoint 4080，無 FAIL／repair；launcher success、cleanup_verified=true。
- server63268、runtime8008/91128、UE53260/39944 退出，saved checker再次 inspect 驗證無殘留。
- 原始 protobuf capture 解碼通過：隊1 4147 snapshots、取消124 ticks／完成479 ticks；隊2 4138 snapshots、取消125 ticks／完成479 ticks。此 479 是 first positive sample 到 zero 的觀測差，不把它改成 480 幀 gameplay 規則。
- 首輪 PNG 視覺 QA 發現長回城文字被右側裁掉；所以此輪只封關 gameplay，不當成 UI 完成。match 文字改兩行、小字、wrap 後重跑。

## 最終 HUD 與 Lua 動態 DLL 實跑

`target/interactive-runs/interactive-ue-1791073998` 使用修正後 C++，release base_content 明確開啟 runtime-lua-content，scripts/server/runtime 同 profile 重建。launcher 與獨立 saved verifier 均 success=true／cleanup_verified=true。

| 隊伍 | B1 / ACK | 取消 Move / tick | B2 / ACK | 完成 tick / 基地 |
| --- | --- | --- | --- | --- |
| 1 | 2 / 2704 | 3 / 2830 | 4 / 2852 | 3332 / (0, 0) |
| 2 | 2 / 3153 | 3 / 3279 | 4 / 3301 | 3782 / (2400, 0) |

- 每隊恰好兩次 B 綁定回呼與原始 ID2／4 forwarding，各 ACK status0；移動 input3 取消。兩隊啟動時間不同，並不要求同 tick 開始。
- 三方 checkpoint 各66 PASS、最後3960，超過較晚完成3782至少120 ticks；無 FAIL／repair。
- protobuf capture：4085／4063 snapshots，兩隊取消124 ticks、第二段完成479 ticks，倒數每個新 tick 嚴格下降、兩段恰好正值→零、不重設。
- 四張PNG均實際視覺檢查：active8.0s／complete0.0s均在1280×720右側完整可讀；基地回城後商店顯示in range，仍無免費治療或位置寫入。
- server85876、runtime89760／75832、UE72524／74352皆退出；saved verifier再次檢查原始5個PID無殘留。
- 這封關 B 綁定 delegate／正常網路／回城 HUD 增量；不是實體鍵盤事件驗收，也不是完整遊戲 UI 封關。

## 驗證入口

```text
tools/lua/lua.exe scripts/tests/ue_recall_observation_test.lua
tools/lua/lua.exe scripts/build_ue_moba.lua --build-only
tools/lua/lua.exe scripts/run_2player_ue.lua --single-lane
tools/lua/lua.exe scripts/verify_ue_recall_run.lua RUN_ID
```

雙 UE 執行環境：OMOBA_UE_RECALL_SMOKE=1、OMOBA_UE_SMOKE_SECONDS=120、OMOBA_UE_STEP_FPS=60、OMOBA_RELEASE=1、OMOBA_SKIP_UE_BUILD=1、OMOBA_SKIP_BUILD=0。先完成 C++ build 才能 skip UE build。

saved checker從原始 logs、config、PNG、filtered-world與三方checkpoints 重算，再執行 ignored Rust protobuf capture test；輸出獨立 verification JSON，不修改原始 launcher 報告。

## 已完成檢查與限制

- C++ build76907／19509成功，ABI7與staged SHA382dea09e779aacf310f87238b1fd82921b6d6ba3c260b28948ccb99d17d903d一致；codegen11 files／15 inputs／hash de9c7fcfc98d6479。
- runtime lib54 passed／4 opt-in ignored，main3 passed；首輪 raw capture opt-in1 passed。
- Lua 新回城 positive／缺證據／錯原始ID／錯status／錯HUD／重複callback／瞬間完成／JSON roundtrip通過。既有movement、abilities、parity、match、shop、minimap與result observer回歸通過。
- 完整框架仍有助攻、選角、計分板、多線／野區、完整動畫／資產替換、LAN與效能驗收等未完成。OpenSpec 4.1／5.3／6.2維持未勾選，不把此增量當完整 MOBA 完成。
- 錯誤與預防措施集中記錄於 unreal-moba-error-register.md 的 E108；首跑舊 release DLL 失敗證據保留於 interactive-ue-1791073582。

## 最終 Editor 回歸與收尾

- full build1195成功；本輪新增 raw-capture test source 觸發 Rust 依賴重新連結，因此最終 staged bridge SHA 改為057c643b3df2e93abb736498b1a503101d6a5052e89fd1eefa35f5248d536514。verify-staged-only一致；前面的雙UE證據使用19509建置的382dea…，兩者不混寫。
- Editor36528／BpGeneratorUltimate MCP readiness通過；compile-1791074258的11個Blueprint均零編譯錯誤，沒有新增回城Blueprint graph。
- NativeVisual64537兩輪各13/13通過；PIE14819正常開始、測試、截圖、stop後確認未運行。保存於omfue/Saved/McpAutomation/NativeVisual/report.json與pie-smoke-report.json。
- MCP save_all_dirty_assets成功（target/recall-key-save-all.json）；固定Lua host WM_CLOSE本輪明確PID36528，wait10s與inspect确认正常退出，沒有force kill或關閉不明Editor。
- scoped git diff --check與OpenSpec strict通過；未commit/push、未刪除原始失敗／成功證據或使用者變更。
