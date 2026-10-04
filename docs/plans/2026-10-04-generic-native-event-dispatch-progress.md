# 通用 native 事件派發

## 計畫與決策

1. 延續 2.2b，先確認保存的 Blueprint 是否仍實作舊回呼，再移除自動相容派發；不新增每英雄旗標或特例。
2. MCP `find_blueprint_by_parent` 在 /Game 回報唯一 Saika 子 Blueprint 為 BP_SaikaMagoichi；其 skeleton 為 25 nodes，只有 OnAnimationState／OnAttackPhase。raw 報告保存於 `omfue/Saved/McpAutomation/saika-parent-audit.json` 與 `saika-generic-dispatch-audit.json`。這不是所有資產函式引用的完整掃描。
3. 全部 Hero／Tower／Summon／Creep 的 OnAnimationState、OnAttackPhase、OnAbilityCue 只沿共同 Super 路徑。刪除 legacy Hook／event_dispatch，以及未再使用的 OmMakeSaika、extras fallback、CueKind 轉換器生成程式；不把專屬邏輯搬到另一個自動 adapter。
4. 保存的 reflected typed payload、metadata 與五個 HandleSaika API 暫留，明確呼叫仍可用，但正常 generic event 不會觸發它們。既有 C ABI11 不變；此相容名稱不應擴充至新英雄。
5. Editor probe 同時確認通用 payload 欄位／instance、舊回呼零次，以及直接呼叫相容 API 一次；原測試名稱 SaikaEventDispatch 保留給現有工具，語意已更新為通用派發與顯式相容性。

## 當前功能確認

- codegen 的 exact legacy API／全英雄通用 hook 測試與防角色分支回填測試各 1/1 通過（其他測試 filtered，不重跑整套）。
- 已結束本專案 Editor 101752，restart 回報 project scope／matching_editors=1／force_terminated=true，另查原 PID 已不存在；沒有停止其他專案。
- C++／生成檔增量建置與指定 Editor 測試結果待本次執行完成後補記。

## 剩餘範圍

- 尚保留 typed reflected API 與 C ABI 相容欄位，不能宣稱完全刪除所有 Saika 型別或所有保存資產引用。
- 不執行全套 MCP、PIE、雙 UE、LAN 或效能驗收。完整 2.2b 無功能回退驗收留最後，OpenSpec 仍 20/30。
