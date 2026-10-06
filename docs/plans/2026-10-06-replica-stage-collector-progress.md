# Replica 分段診斷收集器（2026-10-06）

## 決定與實作

正式 client max 仍固定 50ms；不靠放寬門檻或反覆全驗收解決。Grok 移除可證明沒有讀者的逐 step report map，primary 審查後接受，report 回傳及正式 checkpoint/evidence 路徑保持。這消除無界記憶體保留，不等於已證明尖峰根因。

primary 新增 `scripts/moba_replica_stage_report.lua` 與固定 Lua CLI `scripts/collect_replica_stage.lua`。讀取既有 OM_REPLICA_STAGE v1，每個窗口必須是 60 個成功 Applied samples；保留同一個 outer 最慢 sample 的全部七段與未知 residual，不將各段獨立最大值拼成不存在的 step。whole-file summary 只保留最後窗口與全域 outer peak，固定大小；wall time 不推論成純 CPU time。

契約拒絕版本／身份錯誤、缺欄位、未知或重複／escaped JSON keys、負數／非整數／非有限值、加總矛盾、重複與倒退 timeline。只允許已知 log4rs／ANSI suffix，其他尾隨資料拒絕。每次讀 8192 bytes，每行最多 65536 bytes；讀取錯誤不能冒充 EOF，檔案 handle 在成功及失敗都關閉。

CLI：

```
D:/code/omoba/tools/lua/lua.exe scripts/collect_replica_stage.lua RUNTIME_LOG PLAYER_ID TEAM_ID
```

stdout JSON 是 completed_windows_only_not_performance_acceptance，不含 pass/fail 效能准入結論。最後不足 60 samples 的窗口不會出現在來源日誌，報告明示此限制。每個 process lifetime／單調 timeline 分開收集；來源 v1 沒有 epoch，倒退時拒絕而不猜 rebase。此入口不啟動遊戲、不改原始日誌、不寫驗收結果。

## 本批確認

- 固定 Lua `scripts/tests/moba_replica_stage_report_test.lua`：11/11，包含實際 tmpfile chunk／CRLF／完整未換行尾列／超長行／截斷 JSON，以及 injected I/O error 關檔。
- Grok retention fix：primary 獨立 runtime replica_stage lib 1 passed／87 filtered；compiled-content-only binary check exit0。既有三個 td_rounds dead-code warnings 保留。測試不是載 DLL 的 report 生成整合證據。
- `build_ue_moba.lua --verify-staged-only` exit0，read-only，bridge SHA `0e5bac851dff2e8a4a3b7acdaeac6b200ab65897f174838bb6fa9be48154fe5d`，base SHA `2cb13f4dcec12e34bd1a24c0e9f0b27f2d27843c79784409088a503396dbe60f`。
- 本批未跑對局、simulation、UE／Blueprint／PIE 或完整驗收；没有新實機 stage 日誌，也沒有宣稱 50ms 尖峰已修。
- primary 後續一次 `cargo build --manifest-path omoba-client-runtime/Cargo.toml --release --bin omoba-client-runtime --features compiled-content-only` exit0，28.65s；仅既有三個 td_rounds warnings。新 executable 已包含分段診斷與 retention 修正，未啟動；没有重新生成／部署DLL。CLI無參數exit1是預期usage拒絕，不算成功診斷。

## 接續順序

後續當前結果：strict lifetime已由primary完成，Rust1/Lua7/真owned fixture確認。一次真60Hz短程run `replica-stage-20261006-lifetime-v1` success／五owned identity cleanup獨立核對通過。collector從真stderr各收59窗口／3540 samples；兩队同tick227尖峰351.1038ms／475.328ms，fixed_step分別351.065ms／475.2844ms。這證明本批可取得可核對的真分段資料，不證明CPU根因或50ms通過。原「沒有實機stage採樣」只指採樣前的增量；下段順序原決策已執行，詳session-lifetime-progress。

launch cleanup 發現 PID/executable-only 可能誤停重用 PID 的同名程序。先由 Grok 限定修 strict lifetime/native same-handle stop，primary 審查與局部功能確認，再考慮一次短程實機分段採樣。全框架仍 25/31，6.5 不勾選；LAN 要第二實機。模擬單次最多10場，不拆批规避。
