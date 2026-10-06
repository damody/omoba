# 選角共用取消訊號（2026-10-06）

## 計畫與決定

1. 檢查既有程序生命週期，區分合作式取消與強制終止後的監督。
2. 將明確取消訊號放入共用 selection budget，不為每個入口各寫一套清理流程。
3. 接入單機／共享選角、無畫面主機與遠端加入，保留原有真人 consent、原始程序身分與完成配方檢查。
4. 只驗證受影響功能，最後完整遊戲驗收仍另行處理；錯誤集中記 E338。

## 已實作

互動選角的 `run_moba_role_ue.lua`、`run_moba_selection_host.lua` 與 `run_moba_selection_join.lua` 接受 `--selection-cancel-file PATH`。相對路徑以專案根目錄解析；為每次選角選擇新的訊號路徑，在選角尚未交付結果前建立該路徑即可取消。工具只檢查存在性，不讀取、建立或刪除訊號；已存在的訊號會在啟動子程序前拒絕。

共用 `moba_selection_deadline.lua` 同時處理可選期限與取消。未指定期限仍允許真人無時限選角，沒有取消參數時保留既有 numeric/nil budget 行為。檢查包含等待中、原始程序退役後以及最終結果交付前；取消不授予 lock/finalize 權限，不自動開局，不從不完整 seats 補造配方。

共享／無畫面主機取消會清理本次擁有的原始主機與本機 renderer；遠端加入只清理本機原始 renderer，不停止遠端主機。訊號保留供操作者追查。選角交付後不把此訊號當成全域遊戲終止開關。

## 當前驗證

- 共用 budget assertions：無期限取消不取時鐘、啟動前訊號、等待／晚到訊號、期限邊界、三入口 option guards 通過。
- 無畫面主機 12 組通過：含既有訊號不 spawn、等待中取消、已發布但尚未交付的 final artifact 仍拒絕；真 release native config，主機生命週期注入。
- 遠端加入 19 組通過：含既有訊號與等待中取消；部分驗證使用真 debug compiled config，renderer／網路注入。
- 既有單機 13 組與共享 25 組通過；合計 69 組，另加共用 budget assertions。早先同檔 17 組執行不重複計數。
- `git diff --check` 通過；HEAD 維持 `02757d51f201a03bcea46e1ca1d622594db28dde`。

未執行 Rust／Unreal 建置、實際遊戲、場次模擬、雙實機 LAN 或完整驗收；OpenSpec 整體仍 25/31，不能用 fixture 通過代替遊戲完成。

## 邊界與下一步

native spawn 的 Child handle 用來取得原始身分後未持續監督父程序，helper 也是逐次呼叫；程式碼檢查不能證明 Lua 被強制終止後會自動清理。此次是獨立且通用的合作式取消功能，不是該缺口的替代修補。既有迴圈通常每 250ms 檢查，但 native wait／驗證期間不是即時可中斷，仍保留原有有界等待。

強制終止／crash supervision 需要原生生命週期管理與專門故障驗證，尚未實作或宣稱完成。實際雙機 LAN、完整 UI 對局流程與既定效能基準亦尚未完成；不新增 Lua gameplay runtime、不維護 omfx、不以十場以上模擬掩蓋缺口。

## Grok Build 委派

遵循 delegate-to-grok 與 grok-build skill，送出含完整共用模組的自足、無工具、patch-only 有界請求，未開啟寫入模式。

- Job：`run-muwk0gcn-9bt5vq`
- Thread：`594ec77e-0668-4f6e-aff1-4de345e01cf6`
- 201 秒停留 starting，無文字／程式碼／可接受結果，由 primary 取消；不是人類要求取消。
- follower 確認 cancelled，三 tracked PID 均 null，OS 原 bridge 92460／agent 85644 均不存在；stop exit1／taskkill128 不單獨視為退役證明。
- 成本、API 時間、根因未知。無檔案工具的請求也未完成，不能據此歸因檔案工具；未變更全域 MCP／auth／Grok 設定。

實作與整合由 primary 接手，沒有冒稱接受 Grok 補丁。
