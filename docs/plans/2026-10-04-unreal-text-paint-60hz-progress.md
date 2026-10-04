# Unreal 60Hz：文字繪製診斷與程序退出修正

## 計畫與決定

1. 延續 `build-unreal-rust-moba-framework` 的 6.2 增量；總進度維持 19/30，不把升級資料成功當成完整 UI 完成。
2. 不重試 E120 已排除的寬度、wrap、volatile、自身 clip；以真實 OnPaint、字型條目、公開繪製批次的頂點／索引與 clip 診斷。
3. 名稱獨立 layer 未解決，已撤回；D3D12 比較也未解決，D3D11 default 保持不變。不修改 Unreal 引擎、不增加角色專屬程式或 Blueprint graph。
4. 修正已確認的退出競態，再完整建置／MCP／原生測試／PIE／60Hz 雙 UE 驗證。

## 已實作

- 共用名稱 widget 的 opt-in paint 診斷：正式文字、位置、geometry、cull、clip、layer、frame；字型 Valid／XAdvance／atlas index；merge 後索引引用與實際 batch clip。
- 診斷只由 `om-upgrade-smoke` 的 layout 查詢要求，不影響正常每 entity 熱路徑，不改文字繪製結果；單次 renderer callback 用後移除，widget 銷毀時取消未完成 callback。
- 雙 UE launcher 增加白名單 `OMOBA_UE_RHI=d3d11|d3d12`，預設仍 D3D11；report 記錄選項，保存驗證器核對兩隊實際 `Using Forced RHI` 日誌。非法值在建立程序／證據目錄之前拒絕。
- `process.graceful_stop` 關窗超時後發出 stop，再 bounded wait 實際退出；仍未退出就失敗，不以 stop 已提出當清理完成。
- 無 OS mock 測試覆蓋強制停止後等待、等待超時、PID 已不存在與 executable mismatch。

## 實測證據

| run | 原始 snapshots（隊 1／2） | 三方 PASS（各隊） | 最後 tick | 保存驗證／清理 |
| --- | --- | --- | --- | --- |
| 1791098159 | 2958／2952 | 50／50 | 3000 | 通過 |
| 1791098401 | 2940／2931 | 50／50 | 3000 | 通過 |
| 1791098864 | 2929／2922 | 50／50 | 3000 | 通過 |
| 1791099179 | 3065／3057 | 51／52 | 3120 | 通過 |
| 1791099344 | 3229／3222 | 54／54 | 3240 | 通過 |
| 1791099775 | 2970／2962 | 48／48 | 3000 | 通過（RHI D3D11 另驗） |
| 1791100233 | 3560／3553 | 58／57 | 3600 | 通過（真正 backbuffer） |
| 1791100483 | 2954／2949 | 50／50 | 3000 | 通過（真正 backbuffer、RHI另驗） |

上述均 60Hz，兩隊原始 input 2 各一次，經真正 Ctrl+Q bound delegate、IPC／KCP 與權威升級，不是 OS 實體按鍵注入。

- 字型診斷：每個字元 Valid=1，scale0.6660，合理字距與 texture0。
- 最終繪製資料：每個名稱 rect 內索引 54／60／60／54、right332／432／525／618，兩隊前後相同；batch clip 為 (0,0,1280,720)，stencil0。這些只證明 CPU 提交資料，不證明 GPU 像素。
- D3D12 run1791099494：兩隊 upgrade complete，圖片仍截短；原 report cleanup_verified=false，保存失敗報告不修改。稍後五 PID 均已退出，定位 stop 後未 wait 的競態並修正。此 run 不列入完整成功驗收。
- Full build 通過，staged bridge SHA-256 `6330b7204db53d856d3aff4c78ae7d3d1a71fb65d7856d2bfe10762001cd9ad2`；codegen 11 files／15 Lua inputs、hash `ac6ff592a15c8b5e`。
- `compile-1791099694` Blueprint MCP 通過；Editor35204 同一程序兩輪原生測試各18/18，串行 PIE 通過，之後 PID 已退出。
- Lua upgrade／two-team observation 及 process graceful-stop 回歸通過；OpenSpec strict 通過。
- 最終 helper／診斷／opt-in 回歸版本重新 full build 通過；`compile-1791100638` MCP 通過，Editor31976 同程序兩輪各19/19（增加 `RenderedUiCaptureOptIn`），串行 PIE 通過。兩個 backbuffer run 的五程序均由保存驗證器另驗退出。

## 未完成與下一步

- 新比較：本機 `SlateApplication::TakeScreenshot` 會額外 `PrivateDrawWindows`，而上述診斷是正常繪製，不能把兩者混為一談。通用 opt-in `FOmRenderedUiCapture` 在 `OnBackBufferReadyToPresent` 讀取含 UI 的真正 backbuffer、不重畫 Slate；兩張完成 PNG 及讀取結果由保存驗證器核對。只用於升級 smoke，Shipping 不啟用，停用時沒有 readback；module shutdown 移除 callback 並等待 render thread。兩輪八 PNG 均人工檢視，仍有截字，所以截圖重畫不是唯一原因；取證工具不是 UI 修正。
- GPU input batch-relative 範圍補查：兩隊前後皆372 vertices／558 indices／max371／invalid0。沒有確定 GPU 根因；下一次只能針對字形 UV／atlas 上傳或呈現覆寫等新證據調查，不再重試 E120／E121 的否定假設。
- 優先順序：依使用者「先讓60Hz成功」要求，已通過的60Hz輸入／權威／raw／三方一致性／清理與尚未完成的文字像素缺陷分開，不因顯示問題把同步測試寫成失敗，也不把整個框架寫成完成。

- 隊 1 技能名稱／冷卻文字仍可能截短；隊 2 多張完整。已縮小到提交資料之後的呈現問題，但尚未確認 GPU 根因；不再猜 layout，不以切換 RHI 或單張正常圖片封關。
- 完整 5.3／6.2、rank0 初次學習、實體按鍵、完整選角到結算及跨機 LAN 等仍未完成。此增量不勾選整項。
- 防錯詳見 `unreal-moba-error-register.md` E121。
