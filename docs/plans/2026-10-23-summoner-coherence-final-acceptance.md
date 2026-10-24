# 召喚師目錄一致性：最終驗收

完成 `complete-summoner-catalog-coherence`。召喚師的權威操作與 Unreal 作者名稱／說明是編譯期目錄，先前 RuntimeContent／ContentShape 沒有保存這份目錄，Lua 開發重載可能發布新雜湊而實際技能仍執行舊數值。

現在從完整已驗證共享型別產生 MOBA_SUMMONER_CATALOG_JSON，啟動 peer gate 及開發重載相容性均比對整份目錄。身份、文字、冷卻、目標、操作數值／策略及新增／移除／順序變更要求重建所有 peers；沒有名稱特判、ABI 修改或假稱支援召喚師熱重載。

- Rust **1419 成功、零失敗、零 ignored**：content40、template81、codegen68、core930、base_content300。全模擬使用 release，測試181.46秒；不是所有 workspace 全面重跑。
- 15種有效作者變更在 startup／shape 邊界被拒；實際 Lua validate/reload 驗證閃躍、治療、護盾數值及身份／說明／移除均保留原 generation、hash 和內容指標。新啟動經公開 peer gate 拒絕不符資料；還原後，合法技能傷害修改仍可增加generation並更新數值。
- 正式後端與 moba-config 建置成功；同一已接受 script DLL 同步到後端和 Unreal 外掛，完整部署 SHA 一致。compiled-content-only bridge 沒有 runtime Lua 程式路徑；其副本一致且未更動。本批沒有原生／UI／ABI 修改，沒有重跑或冒稱當前 UE65／25PNG；該證據屬前批護障。
- **3420來源／資產** exactset、SHA、size、mtime 前後一致；**55份證據**逐byte及SHA獨立校驗成功。三個 worker 與其 watcher 全部退休並reap，170秒health timer亦退休；最大採樣間隔171秒。最終CIM未見本輪Lua／Cargo／rustc／測試／Unreal程序。
- 無測試／建置失敗；保留既有unused TD flags及protoc/generated fallback warnings。探索路徑錯誤與部署邊界記入錯誤MD及E1508。

證據：[index.json](../acceptance/2026-10-23-summoner-coherence/index.json)。錯誤：[errors](2026-10-23-summoner-coherence-errors.md)。沒有新增真網路對局、quota reset、commit/push或omfx工作。

通用channels／zones、整組召喚資源原子回滾、完整combat replica接受、護障專用圖示及11個外部父項仍pending。MVP33/44不變；下一批應優先具體玩法缺口，避免重複已完成名牌／目錄驗收。48h原期限2026-10-25 00:33:38 Taipei不變。
