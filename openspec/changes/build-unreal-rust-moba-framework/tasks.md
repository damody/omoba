## 1. 建置基線與工具入口

- [x] 1.1 記錄現有工作樹與 `omfue`/Rust/Unreal 建置基線，確認已知錯誤是否仍存在，並保存可重現指令與結果。
- [x] 1.2 為 `omfue/codegen` 加入只讀 `--check` 模式與內容變更檢查；修改生成檔後指令應失敗，恢復後應通過。
- [x] 1.3 建立固定順序的 Lua 建置入口，先建 `base_content.dll` 再執行 Unreal codegen、bridge 與 OmGame，驗證每個階段失敗會立即停止。
- [x] 1.4 把 `omfue/restart` 與 BpGeneratorUltimate MCP 健康檢查接入啟動流程，驗證能查詢 Editor 並讀取最小 PIE 狀態。

## 2. 內容生成

- [ ] 2.1 建立共用型別化內容模型與驗證，確認 Rust 與 UE 生成器對同一 Lua 來源得到相同 ID 與內容雜湊。
- [ ] 2.2 將英雄 Unreal C++ 輸出改為通用模板，移除 Saika 專屬分支，驗證既有英雄可編譯並無功能回退。
- [ ] 2.3 建立技能 effect 宣告與 Rust handler 自動註冊，驗證四技能測試英雄可在 headless 對局施放。
- [ ] 2.4 產生穩定 Unreal 資產配方並實作新英雄範本，驗證新英雄不需手寫角色 C++ 或 Blueprint graph。

## 3. Unreal Editor 自動化

- [ ] 3.1 以 BpGeneratorUltimate MCP 套用資產配方、匯入模型/動畫/材質並驗證引用；同一配方重跑無非預期變更。
- [ ] 3.2 以 MCP 建立必要 Blueprint/UMG、保存並編譯，驗證錯誤能回報資產路徑與診斷。
- [ ] 3.3 以 MCP 執行 PIE 測試操作、收集狀態/日誌/畫面，輸出可檢查驗收報告。

## 4. Rust client runtime 與 Unreal 接軌

- [ ] 4.1 擴充 localhost IPC 為 MOBA 輸入與安全投影，驗證版本握手、輸入結果與視野隔離。
- [ ] 4.2 在 `omfue/bridge` 與 `OmRuntime` 實作 IPC adapter，驗證 Unreal 可顯示兩隊各自的 filtered world 並送出移動。
- [ ] 4.3 實作 Hide/Forget/ResetView、cue 去重與 renderer reconnect，驗證重連不重啟對局或重播一次性效果。
- [ ] 4.4 隔離正式 MOBA 模式的舊 `RuntimeDriver` gameplay 路徑，驗證單機與 LAN 都只持有一份玩家端模擬。

## 5. MOBA 對局規則

- [ ] 5.1 以單路測試圖實作對局階段、隊伍、出生、死亡與重生，驗證固定種子重播雜湊一致。
- [ ] 5.2 實作兵線、防禦塔與基地勝負，驗證一場 headless Bot 對局能正常結束。
- [ ] 5.3 實作經驗、擊殺/助攻、金錢、商店、六格裝備與回城，驗證合法/非法輸入及數值結算。
- [ ] 5.4 建立三路、野區、地形、導航與建築解鎖規則，驗證固定種子批次對局無卡住。
- [ ] 5.5 實作五位置 Bot 與三種完整英雄原型，驗證 100 場 headless 對局無越權輸入、死局或非法目標。

## 6. Unreal 完整對局與驗收

- [ ] 6.1 完成通用動畫、技能 cue、視野與地圖呈現，驗證缺少非必要美術時 fallback 可用。
- [ ] 6.2 完成選角、HUD、商店、小地圖、計分板與勝負 UI，驗證 PIE 可從選角玩到結算。
- [ ] 6.3 驗證美術替換流程：只變更資產與 Lua 配置後重跑生成/MCP，無角色專屬 C++ 或 Blueprint graph 修改。
- [ ] 6.4 驗證兩台 LAN 玩家同局、跨隊視野隔離、renderer 重連與版本錯配處理，保存測試紀錄。
- [ ] 6.5 建立完整建置與效能基線報告，驗證 server tick、client step、IPC 與 UE frame time 符合基線後固定的門檻。
