# Unreal 編譯 catalog 嚴格准入（2026-10-06）

## 計畫與實際問題

盤點正式啟動版本檢查後，發現catalog的空介面簽章可被當相容、內容錯配只記警告，而且不相容旗標未阻止Tick處理frame。不能只依建置工具最後可能攔住來維持安全。

## 實作與決定

- 新共用OmCatalogCompatibility.h驗證Catalog指標、struct_size、C ABI、lease／catalog非零generation與兩份16字元lowercase hex。
- string ref先以uint64核對邊界，再讀固定16bytes，不把超大長度交給UTF轉換；內容及surface必須與真正compiled OmRegistry常數完全相符。
- StartRuntime立即驗證；失敗返回false並停止本地bridge，清既有HUD／actor／map呈現。
- Tick在任何frame取得、ProcessFrame或ACK之前驗證；錯配release catalog後停止本地bridge、不消費frame。已有合法catalog而暫時Acquire失敗僅跳过該tick。
- 每次取得都驗證，不讓相同generation掩蓋改變；預設相容false，Start／Stop重設catalog游標、Stop重設frame sequence。
- 不殺後端、不重開對局、不改權威規則／IPC／視野／cue ledger，不增加Lua VM或暫時fallback。
- 嚴格compiled內容契約也不再放行舊DEV內容hash差異；更新內容需生成並重建，不能繼續依舊C++ metadata呈現新數值。

## 本輪確認

- OmGameEditor限定OmRuntime／OmGenerated／OmEditor、保留NoEngineChanges：8actions，Result Succeeded、exit0。
- 新Om.Generated.CompiledCatalogContract案例涵蓋合法、空catalog、零／錯generation、舊ABI、錯size、null table、缺hash、超長／越界ref、非hex、內容／surface錯配與恢復。僅編譯，未執行Editor斷言。
- scripts/ue_native_visual_smoke.lua加入此案例，fixed Lua loadfile語法確認成功；沒有執行整套runner或覆寫完整驗收證據。
- strict OpenSpec與主repo／omfue whitespace檢查通過。
- 日誌：`D:/code/omoba/omfue/Saved/Logs/compiled-catalog-contract.log`。

## 剩餘與版本

仍需最後統一重建部署後，執行原生案例及真實startup mismatch／無frame消費／重連。沒有stage、PIE、LAN或完整驗收，本輪不能稱這些執行結果已成功。

identity `ff3ef5e2957aa89f`／data `2df5b6b1e02d95d7`／presentation `d5553bbb31c459a4`／surface `1e2dbe976a9ee6d7`保持；C ABI16／wire6／IPC5保持。僅維護omfue。

完整6.4仍未勾選，總進度21/31。防錯紀錄E284。
