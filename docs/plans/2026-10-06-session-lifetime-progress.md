# UE session 精確程序生命週期（2026-10-06）

## 決定

PID + executable 不能證明程序屬於本次啟動。舊 active session 缺精確出生識別時，保留檔案並拒絕啟動／清理；不能自動刪舊紀錄、掃描／猜程序或採 exe-only fallback。

Grok `run-muwamhpi-sxwx3k` 已讀來源但9m35s沒有本批code diff，primary決定取消收回。follower confirmed cancelled，tracked pid/agentPid/bridgePid null；stop工具exit128因PID已不存在，metrics/cost未知。沒有接受該工作。前一個 `run-muwacisn-dq61gc` retention fix已審查接受，不回退。

## Primary 實作

- Native `spawn` 的 owned_identity opt-in，從原始 std::process::Child handle取得 exe + 完整 FILETIME creation_token 字串，失敗只透過原始 Child 清理本次新程序，不重開 PID。舊spawn返回契約維持。
- 新 `inspect_owned`：同一handle讀身份；真正不存在／退出才alive=false，access/query error傳出，不能當程序已退出。
- 新 `stop_owned`：同一query+terminate handle比較canonical exe／精確token，保留handle至TerminateProcess及bounded wait完成。RAII關handle；token拒float、leading zero、非法／overflow。無非Windows shellfallback。
- Lua `spawn_owned`／validate_owned／inspect_owned／assert_owned／owned_alive／stop_owned／close_window_owned；舊API保留但此UE launcher不再以exe-only停程序。失敗仍fail closed。
- `ue_session_identity` preflight整份有界records再reverse stop；任一缺token、重複PID／role、sparse array、access或reuse mismatch，preflight時零stop。停止時native再核對；不宣稱多程序操作是原子交易，後續 race 出錯可能已有先前owned程序被清理，但不得誤停其他lifetime。
- launcher publication含identity_version=1與全部spawn原始身份；cleanup若無法證明owned已退出保留active檔。reconnect更新最新完整identity，不只換PID；同PID新lifetime也能分辨。reconnect continuity固定原四個身份，graceful WM_CLOSE走strict native身份入口／poll owned lifetime，仍不以forced stop冒充graceful成功。
- 其他歷史workflow的legacy process APIs本批未全庫遷移；project-bound MCP舊秒級birth接口保持相容，不冒稱所有外部程序控制均已升級。

## 本功能確認

- Rust `owned_creation_token`：1 passed、7 filtered；helper dev build exit0，无新warning。
- 固定 Lua `ue_session_identity_test.lua`：7/7純mock policy；不是實際TerminateProcess測試。
- opt-in `process_identity_live_test.lua --run-owned-fixture`：一次本次建立短命隱藏Lua fixture，原始Child身份、native錯誤token停止與WM_CLOSE拒絕、正確token無視窗posted0／精確停止、owned已退出通過。仅本次保留身份的fixture；本次空stdout/stderr及空目錄自行清理。加入WM_CLOSE後做當前功能確認；無遊戲／UE／網路。
- reconnect observation既有17＋3 bounded phase pure場景通过；launcher/reconnect loadfile語法通過。未以pure parser結果冒稱真實renderer重連。

## 接續

### 短程採樣實際結果

`replica-stage-20261006-lifetime-v1` launcher exit0，success=true、cleanup_verified=true、release／compiled-content-only／single_lane／60Hz；兩隊原生movement／owner HUD／consumed gate通過。primary再透過strict owned_alive逐五筆原始identity確認均已不存在，active-ue-session.json已退役。真實game path使用本次原始Child身份完成精確cleanup。

來源與新生成摘要留在 `target/interactive-runs/replica-stage-20261006-lifetime-v1/replica-stage-diagnostic.json`（不提交）。兩runtime各59完整窗口、3540 represented successful samples；各未滿窗口保留來源限制，不排除任何已完成窗口尖峰。p1 tick227/seq227 outer351103800ns，fixed_step351065000ns；p2同tick227 outer475328000ns，fixed_step475284400ns。最大時間在fixed_step，不是decode／hash／host_finalize；wall-time不能再推論是CPU忙或scheduler等待。比50ms嚴重超標，沒有聲稱performance成功，不改門檻。

後续縮小到filtered fixed_step內的CPU／排程等待來源，先靜態定位相關dispatcher／pool／系統路徑再實作通用修正；不回去猜report history或JSON成本，不盲目調worker/priority、不重跑Blueprint/PIE/全場batch。

本次是120秒movement gate一次短程診斷，不是完整自然終局或renderer reconnect／全驗收；重建Rust runtime一次但skip其他build。完整框架25/31，client固定50ms及最後全驗收保持待辦。
