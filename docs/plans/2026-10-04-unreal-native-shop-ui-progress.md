# 原生商店 UI 輸入與 DPI 回歸

## 本輪計畫及決定

沿用 OpenSpec `build-unreal-rust-moba-framework` 的6.2部分，維持60Hz範圍，不擴到120Hz或把局部完成當整體框架完成。

1. 檢查真正Slate按鈕路徑與控制器命中區，不以WorldBridge API smoke替代按鈕驗證。
2. 修正缺legacy HUD、DPI、空catalog首次建立、越界ability slot，新增Editor回歸。
3. 完整建置，同Editor兩輪automation後才串行PIE；錯誤記於E096。

問題與決定：

- `CreateOmHud` 原本在沒有legacy Blueprint class或已有root時return，連native bar也跳過。改為分開建立兩者、統一bind；不新增Blueprint graph，不修改角色C++。
- 商店height280為Slate layout units，舊命中區卻固定280像素。使用共用height與實際viewport scale；native／legacy存在條件分開，避免無legacy仍占用其右上命中區。
- 將按鈕輸入建構抽為共用`BuildShopInput`，先重驗能力／商店範圍、catalog ID與非空六格slot，拒絕時清除舊event。此僅UI防呆；不驗算價格／材料／餘額，Gold0仍可提交已知catalog，權威Rust決定拒絕。
- 事件明確填入配置的local player。底層converter原本就會覆寫為配置玩家，故不宣稱已修正錯隊交易；測試確認player2不被上游事件預設1誤導。
- 首次空catalog仍建立六格售出列；相同snapshot不重建、不重複增加按鈕。SetAbilitySlot加上0..3邊界。
- PIE preflight拒絕正在執行的Editor automation；cleanup先查PIE狀態、不對已停止session再stop，cleanup error不覆蓋原始失敗。

## 驗證

- 完整build第一次session32238 exit0；修正測試fixture後session42013 exit0。Lua→UE生成11檔，內容身分hash `de9c7fcfc98d6479` 不變。
- staged bridge SHA-256 `3924164b43a99b463eb2cdb06537a698605385adaf60c6240a0535bd4fdde720`，驗證一致；沒有更改Rust交易規則／C ABI6。
- Editor53176第一輪8/9失敗：獨立Slate tree尚無frame/prepass，cached enabled未更新。依UE實際API補UpdateAllAttributes後再建置；save_all_dirty_assets／QUIT_EDITOR並確認舊Editor退出，沒有force stop。
- 新Editor80648同一session兩輪各9/9通過，新`Om.Runtime.NativeShopInput`涵蓋generic buy/sell、player2、非法ID／slot、capability／range／available、離線button停用、無鍵盤focus、六格首次建立／catalog增減／重複snapshot、0.5／1／2倍DPI邊界與越界ability slot。
- automation尚在執行時誤啟動PIE一次失敗，已如實記錄E096。串行PIE session79747通過；補preflight與cleanup後session61417再通過，PIE已停止。報告保存在`omfue/Saved/McpAutomation/NativeVisual/report.json`與`omfue/Saved/McpAutomation/pie-smoke-report.json`。
- diff --check通過（只有既有LF→CRLF提示），staged-only再次核對一致。Editor保留開啟供後续工作，沒有新增常駐server／runtime。

## 重現

```bat
tools\lua\lua.exe scripts\build_ue_moba.lua --full
tools\lua\lua.exe scripts\ue_native_visual_smoke.lua
tools\lua\lua.exe scripts\ue_pie_smoke.lua
tools\lua\lua.exe scripts\build_ue_moba.lua --verify-staged-only
```

必須等待上一個指令結束，不能同時在同Editor執行automation與PIE。

## 邊界

原生Slate widget建立／輸入建構／停用及DPI幾何已驗證，沒有將fixture當成真人滑鼠或實際hit-test grid的完整端到端驗收；缺legacy時的控制器建立分支已修正，但未另外移除既有資產作PIE故障注入。先前60Hz release雙UE交易成功證據維持有效，不宣稱這轮重新進行長跑。完整選角／小地圖／計分板／硬即時效能基線仍待完成，OpenSpec保持17/30。
