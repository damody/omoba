# IPC／renderer 重連契約封關

## 範圍與決定

4.1 的完成條件是 MOBA 輸入、安全投影、版本握手、輸入結果、視野隔離；4.3 是 Hide／Forget／ResetView、一次性 cue 去重與 renderer 重連不重啟對局。效能、兩台 LAN、從選角到結果的全 UI 分屬6.5／6.4／6.2，不能一直加入這兩項的完成條件。

## 當前確認

`cargo test --manifest-path omoba-client-runtime/Cargo.toml --features compiled-content-only presentation_bridge::tests -- --test-threads=1`：27 passed，0 failed，0 ignored，40.81秒建置、0.17秒測試；不是模擬場次。

涵蓋 wrong-version／oversized framing、player/team handshake 拒絕且不披露、MOBA 不用 demo fog、權威 fog 身分、無 canonical identity 的安全 schema、live caster／無 hidden source cue、完整持續狀態與 terminal input results、owner-only economy／receipt、fragmented input、exclusive reconnect、Hide／Forget disclosure epoch、凍結記憶獨立於 live、retained Ready 與 snapshot、六類 cue 真 TCP 兩段連線後不重播 baseline／離線／ACK 歷史並接受 fresh cue。全部沿正式 compiled-content-only 程式，不是新建替代 adapter。

## 已有實際 Unreal 證據（本批不重跑）

- `omfue/Saved/McpAutomation/FinalNative-20261006-fixtures-fixed/report.json`：JSON完整解析 complete結果，42 passed／0 failed／0 skipped／0 not_run，success=true、repetitions=2。這是已保存原生報告，不宣稱本批執行。
- `openspec/changes/build-unreal-rust-moba-framework/evidence/unreal-renderer-reconnect/final-perf-validation-20261006-v1.json`：60Hz renderer正常關閉並新開；原authority／兩個runtime／對側renderer保持，fresh Playing/HUD tick702→4218，新minimap input在5899獲原生與權威ACK，兩隊post-input不同tick hash各2、總102／101 PASS、cleanup_verified=true。不能由檔名推定效能通過，該run完整效能仍失敗且門檻不修改。
- 既有雙隊四技能、shop接受／拒絕／buy／sell、輸入與視野 capture 的實際 UE 證據保留於tasks4.1歷史記錄；不再因之後的不同UI測試重做已驗證的IPC通道。

依上述互補證據，4.1／4.3可封關。這不是6.2同一選角流程至結算、6.4真LAN、6.5十二項固定效能或workflow crash清理的通過聲明。沒有新增cue類型、角色C++、Blueprint graph、runtime Lua或omfx維護。
