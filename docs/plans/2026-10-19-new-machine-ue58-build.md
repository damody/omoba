# 新機 UE5.8 正式建置與接續實作

沿 `handoff.md` 接續 `complete-moba-controls-and-presentation`，既有進度27/44；不使用子代理或Grok，不重設10/10對局額度。使用者要求問題自行判斷、採通用修正、每3分鐘檢查卡住情況，完整驗收留到當前工作完成後。

## 最終結果

正式UE5.8.4 bootstrap、普通NoEngineChanges與stage成功；本輪生命數字及相關通用修正完成。完整198 native／69 Blueprint／458 viewport與五PCM、全部正式Rust consumer scope及五checks已通過，12既有native warning與Rust既有ignored明示保留。3325份最後來源／資產核對成功，沒有新權威對局。詳細接受、不同命令／來源階段的綁定與可攜證據見 `2026-10-19-ue58-final-acceptance.md`；以下過程狀態與失敗為歷史紀錄。

## 計畫

1. 完成正式工具鏈安裝，核對實際compiler、SDK與VS完整狀態。
2. 從保留checkpoint完成ShaderCompileWorker與OmGameEditor bootstrap，接普通NoEngineChanges及staged驗證。
3. 依既有來源核對世界／動畫／HUD缺口；不因父項未勾就重建已有架構。build／Rust／native／asset consumer執行時不改production或test來源。
4. 功能確認後集中執行新機完整驗收；來源／資產與診斷保留，缺證不宣稱通過。

## 已完成的環境與通用修正

- 固定Rust1.95.0、Lua host與cbindgen0.29.4已實際建置／安裝。
- VS實例67e6b32f正式更新至17.14.37806.7；isComplete=true、isRebootRequired=false。MSVC family14.44.35207，實際compiler19.44.35229；SDK10.0.22621.0。
- `ue_build_prerequisites.lua`在任何正式生成／編譯／stage前檢查cbindgen；缺失／非法輸出立即顯示固定已驗安裝指令。九項Lua檢查通過，verify-only沒有增加此依賴。
- Lua host透過Windows ShellExecuteExW/runas提出原生UAC請求，使用結構化參數；不繞過同意、不使用PowerShell／shell fallback。host全26項測試通過。
- `--compiler-version`以公開UE參數一致傳給三個UE build階段，支援Latest／Preview或明確版本；預設行為不變。Lua admission／command tests通過。
- 個人UBT XML過時`PCHMemoryAllocationFactor=1000`造成D8000，經獨立最小重現與二分選項定位。已備份完整原始byte至同目錄`BuildConfiguration.xml.before-ue58-20261019.bak`，只移除該覆寫恢復Engine預設0；其他個人設定不變。真UE BuildSettings保持其他選項與來源相同、只省去該參數即可編譯成功；輸出在Saved，沒有手改Engine responsefile／共享PCH／UBT來源。UBA探索未解決問題，已從正式來源移除該探索選項。

## 當前執行與接受界線

正式入口：

```bat
tools\lua\lua.exe scripts\build_ue_moba.lua --ue-root D:/UE5.8 --build-executor local --disable-action-cache --max-parallel-actions 20 --prefetch-shared-pch --bootstrap-engine --compiler-version Latest
```

本輪輸出：初次`omfue/Saved/BuildTemp/new-machine-bootstrap-v7.log`，續編`new-machine-bootstrap-v8.log`。ShaderCompileWorker已真實Succeeded，114.12秒；OmGameEditor／Engine首次完整圖為4451 actions，尚未宣稱完整build或staged接受。checkpoint僅由成功流程退休，不手刪。

09:15健康檢查確認並行8仍有約41GiB可用RAM；核對本次workflow_supervise操作與精確PID／exe／creation token後，停止其監督程序讓私有Job回收全部後代，確認cl／rustc／cargo／本次UBT均已退出，再以原來源與並行20續編。沒有修改／刪除Engine build cache；UBT沿用已完成產物，剩餘圖4258 actions。20個compiler存活時仍約37GiB可用RAM，屬資源調整，不把計畫性中斷v7算成產品編譯失敗，也不把部分成功當完整接受。

過程失敗v4／v5／v6、MSVC獨立probe與安裝結果保存在Saved/BuildTemp，詳細根因見錯誤register E1300–E1312。安裝器launcher exit0不是背景更新完成；以VS完整狀態、實際檔案及build結果共同判斷。監控核對程序、log時間／action進度、CPU／I/O與記憶體，不能只因程序存在就判斷健康。

先前fresh script-abi為15成功、host26成功；base_content為224成功／17失敗，compiled-content-only單項重現亦失敗，仍待處理，沒有沿用舊機1168作本機成功。未執行新權威對局／ACK／LAN或效能驗收。

## 等待建置時的唯讀定位

v10已真正成功exit0：ShaderCompileWorker、Engine／OmGameEditor bootstrap、普通NoEngineChanges（UBT2.62秒）、資產確認與bridge／base_content stage SHA全部完成；marker由正式流程退休。OmRuntime／OmEditor明確獨立TU，fresh checkout的Unity helper衝突已解決。生命文字已實作並編入，正在HealthTextFocusedV1做原生／D3D12確認，尚未接受其畫面。完整60Hz filtered lifecycle專項v2成功；其餘Rust、完整native／Blueprint／viewport仍待最後執行。

公開combat relation目前由visibility baseline產生，含alive、warmup／structure／jungle等attackable條件；team_projector的逐step公開事件只更新channel／attack／buff視覺，未同步該relation，filtered replica因此保留warmup時的false。後續以同一公開subject／元件capability驗證的絕對狀態事件修正，不能移除hash元件或放寬比較。需驗hidden不洩漏、非法payload／subject拒絕與目前warmup／死亡／重生／structure／ward回歸。

自我治療失敗不是缺少HealSelf guard。正式dispatch::normalize_authoritative_no_target刻意把內容定義為None的技能正規化，既有headless_hero_cast_tests也明確驗游標點／enemy附帶時仍自用；single_lane_match_tests的「mistargeted heal拒絕」舊期待與這個契約衝突。修正時保留無目標內容的權威正規化，將過時案例改為驗實際自用／正確資源與cooldown，以及遠距單位施法仍拒絕；不為了舊斷言去除正式行為。仍待真正實作及重跑，不在此聲稱修復成功。
