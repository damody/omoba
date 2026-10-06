# 選角與結算共用有界日誌讀取

## 計畫與決策

- 接續 OpenSpec `build-unreal-rust-moba-framework` 6.2，25/31；不為局部確認勾選完整 UI／PIE 驗收。
- Grok Build `run-muwe88iz-sv5due`／thread `a9570963-3dfd-47ba-85ae-c7673107a5c2` 採只產 patch 的 tool-free 委派；6m43s 無回覆或 patch，primary取消。Follower cancelled／metadata三PIDnull／原bridge和agent不存在確認後，primary接手全部實作、整合與測試。停滯根因／API時間／成本未知；不是Grok交付，不改全域MCP／工具設定／認證。
- 新共用 `moba_bounded_log` 只處理 orchestration IO；128KiB／poll、64KiB／line、LF／CRLF、增量 offset、原 producer 退役後 final EOF。不進遊戲 Lua VM，不增角色專屬 C++／BP。
- IO failure／short read／truncation／disappearance 不能用空字串 fallback，不能讓勝負觀察器沿用舊 complete history。所有 opened handles 在 callback 前關閉；read／seek／close 失敗不提交 offset。
- 選角保留 valid readiness latch；勝負持續讀取並保存每玩家一筆 canonical 結果、驗證矛盾歷史。嚴格 result marker 不接受版本式前綴、額外欄位或 leading zero。
- 結算期限共用原deadline；poll前後與成功報告發布前均檢查，包含screenshot predicate耗時，不能先break再檢查期限。

## 局部結果

固定 `D:/code/omoba/tools/lua/lua.exe` 執行以下受影響測試：

| 腳本 | 實際結果 |
| --- | --- |
| `scripts/tests/moba_bounded_log_test.lua` | 8/8 |
| `scripts/tests/moba_role_finish_observer_test.lua` | 14/14（含success／timeout／late screenshot三個mock launcher情境） |
| `scripts/tests/moba_selection_readiness_test.lua` | 11/11 |
| `scripts/test_moba_hero_selection.lua` | 13/13 |
| `scripts/test_moba_shared_selection.lua` | 22/22 |

共68個局部test groups通過。初次共用reader／result parser fixture失敗已修E328，只有受影響兩組重跑；選角三組未重跑。測試中的Unreal started來自mock launcher print，不是開過UE；沒有Cargo、對局模擬、真程序、Unreal重開或完整效能驗收。保留25/31；完整UI流程、第二實機LAN及既有固定12項門檻驗收未宣稱完成。
