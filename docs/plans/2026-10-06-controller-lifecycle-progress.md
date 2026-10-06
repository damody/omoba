# Unreal Controller 自有 UI 生命週期

## 計畫與決策

1. Controller 建立 own HUD root／command bar／selection widget，並綁定 world bridge listener，但沒有 EndPlay 清理；不能只仰賴選角 widget 的 NativeDestruct／GC，亦不能因換畫面停止後端對局。
2. EndPlay 先標記 ending，阻擋所有共用 HUD input guard、Tick／CreateOmHud／rebind，再只解除本 controller listener、停止自有 selection service、移除及清空自有 widget。
3. 不用 RemoveAllWidgets、不停止全域 runtime／server、不按 PID 名稱清程序，不改角色專屬 C++／Blueprint、內容 hash 或 ABI。
4. Grok Build `run-muwg4oay-dtia3c`／thread `99e0452d-f241-41dc-8b68-17a259d92e48` 委派 controller header／cpp 和一個 native test；primary 擁有整合／diff review／限定模組 compile。
5. 新 HEAD `02757d51f201a03bcea46e1ca1d622594db28dde` 是使用者 damody 保存上一批變更，保留其 commit，不把已提交上一批變更當遺失或重新實作。omfue 既有 slnx 與 Content/OmAutomation 未追蹤檔保留。

## 當前確認

從 omfue 工作目錄執行 restart status 回 success、matching_editors=0、UE root D:/UE5.8。Grok 委派 3m59s 未產生差異，primary 取消；follower cancelled、三個 tracked PID 為 null 且原 agent／bridge 不存在後，primary 才接手三個 Unreal 檔案。stop 的 exit128（bridge 已不存在）不是唯一退出證據。成本／API時間／停滯根因未知，不算 Grok 成功交付。

Primary 實作 EndPlay 自有 listener／selection service／三個 widget 清理、ending input guard、Tick／CreateOmHud／rebind 防重建；另一 controller 和共享 runtime 不受影響。新原生 `Om.Runtime.ControllerOwnedHudLifetime` 斷言亦加入既有固定 Lua scoped／full runner 清單，不另建驗收入口。

以固定 Lua host 呼叫 bundled DotNet／UBT，`OmGameEditor Win64 Development -Module=OmRuntime+OmGenerated+OmEditor -NoEngineChanges` 11 actions 成功，UBT 13.88 秒；log：`omfue/Saved/Logs/controller-lifecycle-modules-20261006.log`。實際 diff review／whitespace check 完成。BpGeneratorUltimate 的既有 plugin dependency warnings 仍存在，本次不是其修復。

測試已編譯但未執行：EditorPreview fixture 明確呼叫 production EndPlay，驗 own widget parent/reference 清除、另一 controller 保留、late input 與 HUD 重建封鎖；不是實際 viewport／Actor Destroy／完整選角到結算證明。没有真對局、完整PIE、兩實機LAN或完整效能驗收；OpenSpec 25/31 保持。

後續當日 widget-input-lifetime 批次已將本測試與WidgetInputLifetime實際在Editor單輪2/2確認；上述「未執行」是本controller編譯批次當時狀態。原Editor78512退出另有獨立original-token成功確認；仍只explicit EndPlay／parent，不宣稱真viewport／Actor Destroy。詳widget-input-lifetime-progress／E330。
