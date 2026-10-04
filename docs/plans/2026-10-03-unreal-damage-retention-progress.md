# Unreal runtime 傷害事件保留進度

## 本輪計畫與決定

1. 修正每 step 覆寫 external effects 與低頻發布造成的事件遺失：每 applied step 只 drain DirectCombat，保留其它 effect 的既有狀態語意，不複製整個 world。
2. runtime ledger 在 latest snapshot 覆蓋前保留事件，最多 1024 筆、4096 ticks；超量有警告，過期不重播。投影僅允許同 view epoch、可見目標與相同 disclosure epoch，使用集合避免每筆事件掃描所有 entity。
3. successful socket send 記錄 first sequence／connection，ACK 只清理該世代實際送出的事件。prepare／publish／release 均不是消費。送出前剔除已 ACK／視野失效／基準前的事件，解決先建立後送出的舊 envelope。
4. renderer reconnect 的首個完整 view 是狀態基準；不追播基準之前的歷史傷害。Hide／Forget 精確退休同世代事件，ResetView／view epoch 切換清空。
5. 先完成 Rust 回歸，再 full build／stage SHA 驗證，最後 native Editor 兩輪、PIE 與兩隊真實程序 smoke。

## 已驗證

- core 296 passed，新增 drain 只移出 DirectCombat／不移出其它 effects／二次 drain 為空。
- runtime 38 library＋2 binary passed，新增未發布跨 tick 保留、同 ID 冪等、真正送出之前 ACK 無效、first-send cursor、不相符 connection／較早 ACK 不清除、baseline／epoch／Hide／Reset／TTL／容量測試。
- 真實 TCP 使用正式 shared retention 路徑：同一執行緒連續覆蓋 latest 2→3 後收到兩個不同 tick 的提示，ACK 3 退休；預先建立的 envelope 4 不再包含已消費提示。
- bridge 39 unit＋2 TD integration passed，1 外部 KCP fixture ignored（另以真實雙隊 smoke 驗證，不把 ignored 計成 passed）。全部 bridge tests 在 stage 前完成。
- full build session 71343 exit 0，OmGameEditor 3 actions 成功；Editor PID 32284、MCP HTTP 30000。built／staged bridge SHA-256：`67ac3d7135587c4021dd62a8bbf021befcad66836ad4e8f51c393be4f270e81e`，verify-staged-only 再驗一致。
- native session 90590：同一 Editor 兩輪各 7/7，failed／skipped／not_run 均 0；既有 transient world DestroyActor warning 仍存在，不宣稱零警告。
- PIE session 6346 exit 0：native_mesh_rendered=true、remembered_ghost_rendered=true、counts=[1,0]，測試自行啟動的 PIE 已停止。
- Lua module tests、stage gate 3 cases、observation 6 cases、codegen --check（11 files／13 inputs、de9c7fcfc98d6479）、兩個 repo diff whitespace 檢查通過。
- 真實雙隊五程序 smoke session 26854／run `interactive-ue-1791018681` exit 0：兩隊 own-only=true、UE／replica 位移=true、ACK 981／1491、frame samples 19／15，safe ticks 6049／6039。保存 live observation 再驗 PASS，active session 已移除，五個登記 PID 全部不再存在。
- team 1 replica raw position (-1351680,-1126400)→(-409600,-409600)，tick 6048；team 2 (1351680,1126400)→(1296234,1119067)，tick 6039。team 2 只有小幅移動，仍符合本輪既有 smoke 門檻，不冒充長距離導航驗收。

建置重遇 E012：本專案舊 Editor PID 98896 的 graceful child shutdown 不成功，既有 restart 等 10 秒後僅 force 終止已驗明該專案的 PID／children，確認退出再 stage。沒有停止其他專案、修改共享引擎來源或刪除使用者資產。

## 未完成邊界

OpenSpec 仍 15/30，4.3 不勾選。這只是 sanitized DirectCombat 一次性提示的保留契約；不是所有技能、投射物、音效的統一事件模型，也不是完整 MOBA 對局。真實雙隊 smoke 驗證移動／安全 view／ACK，傷害以 core、socket、bridge、UE synthetic 分段驗證，尚無外部傷害實戰 end-to-end 驗收。

## 2.2b 後續遷移決定

- 實際檢查 `codegen/src/lib.rs`：Saika 專屬 payload structs、metadata、class source 與四技能派發分支仍存在；通用 OnAbilityCue／OnAnimationState／OnAttackPhase 及新英雄 native 模板已存在。
- Editor `SaikaEventDispatch`／BlueprintSurface 仍依賴 HandleSaika* 與 FSaika*，不能直接刪除或宣稱已完成。bridge C ABI v3 的 `saika` 欄位也必須保留 layout 相容，不能只改名稱／結構而不驗證消費端。
- 決定：先將 typed legacy 入口隔離為相容 adapter，正式 hero event 模板只走通用 payload；保留舊 Blueprint 接口直到已存資產圖與 test fixtures 明確遷移，不刪除既有資產。本輪只完成檢查與遷移範圍記錄，未冒充實作完成，2.2b 仍不勾選。

## 最末修正與複驗

- 空 ledger 直接返回，不掃描 entity 集合（E056）。中間 full build 19628 成功、Editor 3864，stage SHA `0493fc7761beb4d1d958aef12d7488cca10dd2bae85f1f53c08ff0189aa8fceb`；native 89143 兩輪均成功。此為中間版本，不當作最終版本的證据。
- 另外修正長時間 renderer 離線、saved latest tick 已舊的基準（E057）：floor=max(saved view tick, applied high_tick)。不追播離線期間歷史，不篡改 state replica_tick。真實 shared-ledger TCP 測試改以 saved tick 1／current tick 2 作 bootstrap，tick 2 清除、後續 tick 3／4 仍跨 overwrite 保留並被 ACK 退休。
- 最終 runtime 38＋2 與 bridge 39＋2 回歸通過（1 external KCP ignored），core 296 通過。上述第一輪／中間報告保留為歷史，不能沿用為最終版本結果。
- 最終 full build session 46020 exit 0，Editor PID 36276／MCP HTTP 30000，built／staged SHA-256：`c7288cd34861753bab3656c0146eec0e4f8331d197908b34d59a7c1d23ca34f0`，verify-staged-only 再驗一致。
- 最終 native 14423：同一 Editor 兩輪各 7/7，failed／skipped／not_run=0；最終 PIE 26431 exit 0，native_mesh_rendered=true、remembered_ghost_rendered=true、counts=[1,0]，已 stop。
- 最終雙隊 smoke session 67749／run `interactive-ue-1791019365` exit 0：team 1／2 own-only、UE 位移、replica 位移皆 true，frame samples 20／15，ACK 1105／1533；safe ticks 6609／6365，replica ticks 6608／6364。team 1 raw (-1351680,-1126400)→(-409600,-409600)，team 2 (1351680,1126400)→(852201,851993)。UE team 2 保存的 frame samples 只觀察到 (1320,1100)→(1313,1098)，不能冒充已在 UE 取樣到最終 replica 位置。
- 最終保存 observation 複驗 PASS，六個 observation fixtures 通過；active-ue-session.json 不存在，server 65820、runtime 55944／98112、UE 27024／67696 全部 StillRunning=False。最後 verify-staged-only 的 SHA 仍為上述 c7288cd3...，全部測試後沒有再執行 bridge Cargo test/build。

錯誤與防重犯決定見 `unreal-moba-error-register.md` E053–E057。
