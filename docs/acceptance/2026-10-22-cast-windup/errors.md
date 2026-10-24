# 技能前搖錯誤紀錄

- W01：已有 cast_time=0.5 的正式技能仍即時 execute，提示數值與遊戲時序不符；接正式排程及釋放交易，不用延遲視覺冒充前搖。
- W02：讀取時再次猜測 ability_runtime.rs／visual_event.rs 而失敗；正式來源分別是 ability_runtime/ 與 scripting/event.rs，後續以 rg --files 定位。
- W03：OpenSpec CLI 前輪已確認不可用；維護既有格式 artifacts，依使用者授權自行決定實作，不為 CLI 重複中斷工作。

- W04：Windows rg 不接受路徑參數裡的 *.h glob；應使用具體目錄加 -g '*.h'，不要再推測檔名。


- W05：檢查時引用不存在的 OmUnitTargetPreviewTests.cpp；已改由 rg --files 找到既有 OmPointAreaPreviewTests.cpp，新增測試沿用該檔。
- W06：前搖需在回城與對局結束取消；第一批執行期間補生命期規則與核心測試，V1保留為中間結果，最終V2必須等V1 worker／watcher全退休才開始。


- W07：前搖期間 queued command 應等待，不能和立即命令一樣搶先移動；已在 hero command decision 保留佇列／停止導航，方向輸入仍可移動並取消。這些生命期／提示fixture補強在V1編譯後完成，最終使用RustV2與build-v2，不把中間build-v1當最後版本。


- W08：V1 新增資源測試沿用無 map 的 legacy fixture，該 fixture 沒有 managed ManaPool，直接 unwrap 失敗；新增案例明確裝入合法 ManaPool，保留失敗V1 log，不放寬正式交易或把沒有魔力當成功扣款。


驗收保留的非致命警告包含模板 dead_code、unused import／測試fixture mut、protoc 使用 generated fallback，以及UE四筆既有 preview-world 清理警告；沒有把成功宣稱為零警告。

