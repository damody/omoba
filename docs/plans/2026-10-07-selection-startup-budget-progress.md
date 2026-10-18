# 選角 renderer 冷啟動預算共用化

原計畫 OpenSpec `build-unreal-rust-moba-framework` 維持27/31；本次修復三個選角入口的具體啟動缺口，不把純fixture當作LAN或自然終局證據。

## 實作

- Grok4.7Fast提供 `moba_selection_readiness.deadline`：editor-ready timeout嚴格整數1..3600秒、預設1800秒；由原selection completion token截短，取消訊號照舊拒絕。
- Primary接單人interactive、共享renderer與remote join三條路徑，移除各自45秒renderer常數。Remote CLI轉送既有 `--editor-ready-timeout-seconds` 並更新help。
- Native selection host及client-runtime連線仍有原45秒期限；沒有改ready marker、玩家／protocol／room、receipt／compiled catalog、original process identity、取消或自動鎖定規則。
- Primary新增 `ue_selection_startup_validation.lua`，僅驗證實際選角renderer與Rust finalized plan；嚴格選項、compiled/frontend preflight、新fixture目錄原子預留、private workflow supervision、三個native pointer callbacks、原renderer已退出、錯誤MD與JSON report。零authority／client-runtime／新對局。

## 驗證

三路cold-ready injected clock在60秒才輸出合法握手，configured startup90秒、原total120秒。接線前真正negative-v3三個程序exit1，分別報single/shared/remote handshake timeout。接線後真實測試全部exit0：

| 入口 | 結果 |
|---|---|
| `scripts/test_moba_hero_selection.lua` |14/14|
| `scripts/test_moba_shared_selection.lua` |26/26|
| `scripts/tests/moba_selection_join_test.lua` |20 groups|
| `scripts/tests/moba_selection_readiness_test.lua` |14/14|
| `scripts/tests/moba_selection_deadline_test.lua` |deadline/cancellation全部guards通過|

涵蓋原始child退出／PID重用、不完整或錯protocol／room／player、original receipt／compiled native validation、短completion timeout與cancel；helper另驗非法fraction／string／NaN／infinity與有效總期限截短。既有old-renderer case明確設定短startup budget，保持必須拒絕。

實機單人選角v1b約3分鐘後出現合法ready並完成select／lock／finalize，證明新預算允許真實超過45秒的啟動。原始report elapsed242016ms／success=false：Windows自然退出時exact identity query存取被拒，cleanup亦同錯誤。尚未將此完整實機驗證判PASS，正在修復E356的same-handle退休競態。正式whole fixed12／true two-host LAN／same-PIE natural-result未驗收，true-authority batch10/10保持。

## 工作與錯誤

最終實機v2已PASS：程序exit0、elapsed150453ms、選角英雄training_ranger、native select／lock／finalize三個exact callback、original renderer exit與cleanup_verified=true，simulations_executed=0。報告在 `target/role-ue-runs/machine-selection-startup-budget-v2/startup-validation-report.json`；ready時間08:51:40.523UTC，三callback於08:51:41.079/.138/.155，EnginePreExit於.311。同handle退休修正後完整fixture成功；不只把退出前選角成功當PASS。最後再次跑五個受影響Lua入口，single14／shared26／join20groups／readiness14與deadline/cancel全部exit0。

Grok helper-only job `run-muxuid9b-3cfo15`／thread `77d1d838-1fc0-4334-b073-d5039043bd7c` completed40秒、4calls、USD0.08853872。Primary獨立diff/source與tests接受helper；先前較大writer8分鐘無source修改取消，成本未知。不能把Primary的consumer/test接線歸給Grok。

初次回歸先因本機missing debug moba-config被正式前置拒絕；補正式compiled-only debug build125秒exit0。其後fixture空log改寫缺explicit overwrite導致negative-v2失敗，修正後negative-v3才算真正逾時證據。實機新fixture首次因gameplay automation需finish observer而在spawn前拒絕，沿現有selection-only test adapter處理，不放寬production gate。詳E355及ignored raw logs。
