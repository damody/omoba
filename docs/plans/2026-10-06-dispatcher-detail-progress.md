# Dispatcher 同步驟分段（iteration 11）

## 計畫與決策

OpenSpec `build-unreal-rust-moba-framework` 25/31，6.5 未完成。先定位既有 client 50ms 超標，保留門檻、原始失敗證據與冷啟動；不重跑完整驗收。Grok 僅修改 Nearby 四條收集管線及其進度文件，主 agent 負責審查、接線與整合。

- Nearby 移除每列兩個一元素 Vec，改 worker accumulator 直接 push `(Entity, Pos)`。保留 Rayon、span、Unit+Creep multiset、穩定排序 `(id,generation)`、f32 投影與 tower dirty gate；這是一般性配置改善，不是已證明的尖峰根因。
- opt-in dispatcher 診斷細分 input、首次 build、execute、residual。既有 TickProfile 的 18 個 Job counter 做 bounded 前後差值；未知系統、倒退或重複執行明確 unavailable，不清累計值。不建立另一套玩法執行順序。
- Job 本體可能平行重疊，且不含 fetch／排程／統計；僅並列 body sum 與 slowest job，不從 wall time 相減推算等待。
- `OM_DISPATCHER v1` 是獨立記錄，舊 stage／fixed-step schema 不改。三者連結同一窗口 first strict outer peak；Lua collector 嚴格欄位、身份、accounting、timeline、缺行／重複／未知資料 fail closed，8192 byte chunk／65536 byte line／constant memory。
- 正常未啟用診斷路徑不新增 clock 或 snapshot，資料不進 world／hash／ABI／wire／content。格式化在 outer 計時外。

## 已實際局部確認

- Grok `run-muwclcls-frg1rk` completed，8m19s，thread `12dc1f35-98f5-49b1-b639-42c2288d137f`，reported USD 0.51644504（不是預估或總專案成本）。主 agent 實際差異審查後獨立 `nearby_collection` 3 passed／497 filtered。
- dispatcher_detail 3 passed／497 filtered；tick 1 concrete Specs profile／plain world equality 1 passed／499 filtered。
- replica_stage 10 passed／490 filtered，含舊記錄、whole peak、opt-in hash equality與 stale 清理。
- Lua dispatcher collector 6/6。純窗口 fixture 不是場次模擬。
- staged bridge/content SHA 已核對保持原值；runtime release 27.85s 編譯成功，僅三個既有 td_rounds dead_code warnings。未操作 omfx、UE engine、他人程序、git commit／push。

## 真路徑結果

唯一 `dispatcher-detail-20261006-v1`（release／60Hz／雙 UE／120s）exit0，movement／owner HUD／consumed 成功。主 agent 另外以五筆原始 creation_token 核對全部退出，active session 不存在，三行 linked collector 成功；兩隊各 50 完整窗口／3000 Applied samples，末尾不滿窗口不計。

| 同一 outer peak | p1 tick/seq 501 | p2 tick/seq 446 |
|---|---:|---:|
| outer ms | 30.7792 | 24.8499 |
| fixed_step ms | 30.0451 | 24.8007 |
| dispatcher ms | 8.4811 | 23.9719 |
| input/build/execute ms | .0132 / 0 / 8.4519 | .0081 / 0 / 23.9469 |
| Job count/body sum ms | 18 / .1316 | 18 / .0279 |
| slowest body | nearby .0431ms | nearby .0119ms |
| process CPU ns | 0（合法值） | 0（合法值） |

本次完整窗口 client outer peak 低於固定 50ms；不是十二項完整效能驗收或歷史失敗已永久修復證明，不勾 6.5。p1 同一步 preparation 21.0269ms，p2 execute wall 遠高於 Job 本體；不能把比例當 precise wait 或配置優化的因果證明。保留舊 93/102ms 及 351/475ms 證據。此輪足以確認診斷與 Nearby 功能，停止追加效能採樣，轉往選角／對局原始 child ownership 接線。完整證據 `target/interactive-runs/dispatcher-detail-20261006-v1/linked-dispatcher-diagnostic.json`（建置證據不提交）。
