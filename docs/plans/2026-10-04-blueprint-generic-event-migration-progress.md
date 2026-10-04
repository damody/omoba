# Blueprint 通用動畫事件遷移

## 本批計畫與決策

1. 透過既有 BpGeneratorUltimate MCP 讀實際 skeleton、graph intent 與含 hash 的完整 snapshot，不依歷史節點名稱直接修改。
2. 既有 OnAnimationState 已呼叫同一 ApplySaikaAnimSnapshot，完整傳入 locomotion／overlay／action／phase，且額外保留 IdleVariant、ActionInstanceId、PlayRate。舊 HandleSaikaActionEvent 是重複寫入，其中 instance 固定 0、idle 固定 stand_1。決定刪掉該獨立舊鏈路，保留通用動畫、攻擊與 tracer。
3. 實作可重用的 Lua 規則／工具，不把節點 ID 寫死在程式。JSON 配方指定舊事件、通用事件、payload、sink 與必要欄位；planner 從 wire 建立 component，只允許獨立分支、已知 op／call、bounded 刪除。若共用其他 event／replacement 節點、缺欄位、未知副作用或不完整 snapshot，拒絕修改。
4. 每個必要欄位須連到指定 sink pin，ActionInstanceId 只允許已宣告的純 Conv_Int64ToInt；已遷移但仍殘留舊 payload／call 時拒絕，不把缺少 event 當全數完成。
5. 原資產先編譯、只保存該 package，再備份 binary／SHA。CAS preview 與套用都逐項驗證結果，普通節點用 CAS 刪除；事件入口不支援 CAS delete，故先確認入口完全斷線及其他節點不變，再以 exact node ID 的 documented delete_nodes 刪除入口。這不是整批原子交易；錯誤保留 raw 操作與 backup，不自動重送破壞性批次。

## 工具入口

```bat
tools\lua\lua.exe scripts\ue_migrate_event_branch.lua scripts\migrations\saika-generic-animation.json
tools\lua\lua.exe scripts\ue_migrate_event_branch.lua scripts\migrations\saika-generic-animation.json --apply
tools\lua\lua.exe scripts\tests\ue_event_migration_test.lua
```

第一條只預覽。其他 Blueprint 可以使用自己的 JSON 配方；只支援已存在通用替代鏈路的獨立重複分支，不是自動推斷任意 graph 語意的轉換器。新英雄繼續由 Lua／Rust 與 native 模板產生，不需要此一次性的舊資產遷移。

## 實際修改與確認

- 修改 package：`omfue/Content/RustBP/Heroes/BP_SaikaMagoichi.uasset`。parent／components 不變；EventGraph 31→25 nodes。
- 移除 exact nodes：applySnapshot、breakSaikaAction、castAnim、evtSaikaAction、getAnimInstance、getSaikaMesh。沒有刪除 asset、動畫 Blueprint、通用 handler 或 tracer。
- 前後 compile 均 error_count=0／warning_count=0，保存成功。MCP skeleton 只剩 OnAnimationState／OnAttackPhase。
- 原始備份：`target/event-migration-runs/migration-1791124011/before.uasset`，SHA `f8362f8ef52d7296aff2a478cec0d39fe1713aac4553cbf808331ae6910725e1`。
- 修改後 SHA `5e12eefc6a6bb5c61d933d4d1b41552b638ed7baf63dfa9b8d6caea34c4ccf12`。重跑 1791124065／1791124183 都 already_migrated=true、零新修改，前後 SHA 相同。
- 13 個 planner／結果 gate 直接測試通過；另讀真實 apply 前的 7-get_graph_snapshot 與保存後 14-get_graph_snapshot，保留 25 個節點的完整內容／接線與 hash 精確一致。
- 動畫事件直接派發 `Om.Generated.AnimationStateSmoke` 一輪 1/1 通過；報告 `omfue/Saved/McpAutomation/GenericBlueprintMigration-20261004/report.json`。此為 native probe 的事件派發確認，不能當 saved Blueprint 實際動畫像素或完整 PIE 的證據。
- 本批沒有改 C++／ABI／DLL，不重跑完整建置、完整 MCP gate、PIE、雙 UE 或效能驗收。Editor 101752 保留開啟。

## 回復與剩餘範圍

- 需要回復時，先確認本專案 Editor 已關閉，再將該 binary backup 還原到上列 exact package，重新開啟後編譯；不要在 Editor 已載入資產時直接覆寫 disk，也不要清理 target 中仍需的備份。
- Saika typed reflected API／C ABI 相容名稱仍保留；其他保存資產的引用盤點與安全移除尚未完成。因此 2.2b 不勾選，整體 20/30 不變。
- ApplySaikaAnimSnapshot 的資產內名稱及既有動畫狀態機仍是美術相容配置；本批將事件來源通用化，沒有重寫其動畫 graph。完整品質與對局驗收留最後。
