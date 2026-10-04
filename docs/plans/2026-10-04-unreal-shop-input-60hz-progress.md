# Unreal 商店輸入：60Hz 垂直流程

## 範圍與決定

- 使用者要求先讓 60Hz 成功，120Hz 網路效能驗收延後。沿用 OpenSpec 4.1／6.2；本輪不把商店局部完成當成整個 MOBA 框架完成。
- Lua compiled item catalog 提供穩定 numeric ID、名稱與總價；共用 Rust bridge 生成 frame-owned catalog／字串。原生 Slate 產生通用買入按鈕及六格售出操作，不新增角色 C++、Blueprint graph 或 MCP 執行期相依。
- C ABI 升 6：OmInputCommand 新增 item_catalog_id、buy=12／sell=13；OwnerEconomy 分開 authority shop_available 與 negotiated shop_protocol_enabled；新 catalog pointer 隨 frame lease 到期。header 由既有 cbindgen 建置生成，不手改。
- shop protocol capability 預設 false，只從已協商的正式 bootstrap 寫入 IPC owner economy；未協商／離線／無經濟資料／離開權威商店範圍按鈕禁用。未知 catalog／越界 slot 拒絕。直接 API 仍由 runtime／server 重新驗證，不以 UI gate 代替權威驗證。
- 使用既有 om_submit_input／FOmGameplayInputEvent，legacy gameplay driver 不允許商店提交。WorldBridge 提供共用 reflected SubmitItemBuy／SubmitItemSell；native button 使用同一 subsystem 輸入路徑，僅顯示 queued，實際金錢／装備只由權威 snapshot 更新。
- 成交成功必須是 SHOP_SETTLED；一般 APPLIED、FORWARDED、query-only 不清 shop pending。transport uncertain 保留原 request，busy-ring 仍可重試原結果，不重新交易。跨 runtime 程序 pending 持久化尚未完成。
- 原生商店只在 catalog 變更時重建 Slate rows，按鈕不搶 Q/W/E/R 焦點；調整 HUD 命中區，避免按鈕點擊透傳到場景。

## 驗收方式

- opt-in OMOBA_UE_SHOP_SMOKE=1，正式 single_lane STEP_FPS=60、presentation=30，從預設 Gold 0／每 active 秒 2 開始。
- Unreal 共用 API 自行送不足金錢買入；真實拒絕後等 Gold≥350，再送買入及售出。收到正向 receipt 與空六格／Gold≥175 後才允許既有移動驗收。正常執行沒有自動交易。
- gate 要求三次 distinct queued request、拒絕／買入／出售的原始 receipt ID／時間順序及出售後三方完整 hash。原始 protobuf capture 逐 snapshot 核對 Gold=active income−350+175、六格、owner、immutable receipts，不注入 Gold。
- 這是正式 Unreal API 與資料呈現整合測試，不冒稱已驗證人手滑鼠點擊、完整選角 UI、LAN 或 120Hz。

## 已取得的證據（60Hz release 垂直流程通過）

- core 322 tests PASS；runtime 48 lib＋3 main PASS、3 opt-in ignored；Fyrox cargo check --tests PASS。
- 最終 bridge 46 tests PASS、1 opt-in ignored，包含 shop terminal pending／busy-ring／uncertain／原 ID 不重付回歸。Lua shop observer 的有序交易／錯隊／缺結果／拒絕／餘額不足／重用ID／錯時序／漏裝備／pending 未清共12項通過。
- 修正前 full build session 38470 exit 0，staged SHA-256 8e13349a2824651887e342c5ec4d36aafc5145fc23dfd711b0fba128f4b231c0；Editor 95340。同一 Editor automation 兩輪各 8/8，PIE PASS 並停止。
- Editor 28056／95340 均先 MCP save_all_dirty_assets，再 QUIT_EDITOR，CIM 核對正常退出；此敘述只涵蓋 Editor 正常關閉，不把遊戲測試程序清理冒稱為正常關閉。
- 第一個雙 UE run interactive-ue-1791053032 使用 pending 修正前 DLL，失敗：team1 完成買賣；team2 runtime 已賣出但 renderer 最後 tick11365 未看到11381 receipt，debug inbound backlog602。五 PID 已清理，不增加期限。此 run 不能作完成證據。
- 最終 full build session84743 exit0，staged SHA-256 3924164b43a99b463eb2cdb06537a698605385adaf60c6240a0535bd4fdde720；Editor61016 同 session 兩輪8/8、PIE PASS並停止。latest release runtime／base_content／server 均建置通過。
- 正式 release run `interactive-ue-1791053894`：network 60Hz／presentation 30Hz／UE 上限60FPS。兩隊從 Gold0 自然收入，各原始 request1／2／3 對應拒絕6、買入0、售出0。team1 買入tick10624／出售10629；team2 買入10623／出售10628。出售時皆 Gold175、空六格、bridge pending0，再以正式輸入完成位移與 own-only 呈現。
- 出售後三方 pre/post 完整 hash：team1 170 PASS、team2 162 PASS，最後核對tick10800，沒有 FAIL／repair。原始 protobuf capture 逐 snapshot 驗證 owner、Gold、裝備及三筆 immutable receipts，兩隊5457／5443 snapshots 通過；最後Gold183是出售後繼續累積的正式收入，不是多退款。
- launcher 最末 JSON 保存因 sparse numeric stage keys 失敗，原執行 exit1，不能聲稱 launcher 全程 exit0。修正為字串 keys 並新增 JSON round-trip 回歸；新 `scripts/verify_ue_shop_run.lua` 只讀原始日誌／capture／hash／PID，重新核對全部 gate、保存證據 SHA-256，不重新交易。独立 verifier exit0，報告 `target/interactive-runs/interactive-ue-1791053894/unreal-shop-verification-report.json` 的 success／cleanup_verified／reconstructed_from_original_evidence 均 true。
- 五個 owned PID60852／59488／91904／54528／59420 均已停止，CIM 查無。Editor61016 亦已保存並正常退出。本輪沒有殘留測試程序。

## 重現與未完成範圍

既有保存證據可重新核對（不啟動遊戲、不提交交易）：

```bat
D:\code\omoba\tools\lua\lua.exe scripts\verify_ue_shop_run.lua interactive-ue-1791053894
D:\code\omoba\tools\lua\lua.exe scripts\tests\ue_shop_observation_test.lua
```

60Hz 正式輸入／權威交易／Unreal 呈現串接已通過；這不是硬即時每tick deadline／長時間效能基線的證明。debug backlog 根因、物理滑鼠點擊、完整選角／小地圖／計分板 UI、跨 runtime pending 持久化仍未封關。120Hz 依使用者要求延後，OpenSpec 整體保持17/30，不勾選完整4.1／6.2。
