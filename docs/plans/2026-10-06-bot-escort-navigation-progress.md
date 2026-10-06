# Support 護送導航（2026-10-06）

## 問題與決策

順著5.5共用旅行流程，Escort原本只看距離並直接提交／保留MoveTo，沒有確認本人半徑下是否存在公開完整路徑。

- 最終Escort沿static_next_waypoint共用有界導航，採本人CollisionRadius與公開BlockedRegions。受阻提交正式HoldPosition取消舊追隨，已Hold無queued去重，不轉回AttackMove進攻。
- 目的地只來自current披露Carry，不讀新但未提交披露的權威Pos，不猜隱藏位置或建立替代目標；地形恢復重新普通MoveTo，不永久黑名單。
- 自保、協防與其他戰鬥優先保持；follow radius內保持原Hold語意，root不提交新旅行。
- 原Escort／ApproachStructure曾共用提交分支；新增導航若一起套用，會讓已通過最多8候選查詢的攻城多查第9次。實作時拆分，只對尚未驗證的Escort查路，ApproachStructure不重查；不是放寬導航或增加budget。

## 局部確認

- `cargo test --manifest-path scripts/Cargo.toml -p base_content escort -- --nocapture`：3 tests通過。新正式60Hz1驗受阻Hold、去重、未披露隊友新位置不授權新目標、地形恢復与正常移動；原正式護送與協防2通過。
- 與本輪兵線導航core2＋base3合計8個局部tests，不把filtered out算已跑；既有template三個dead_code warnings保持，無新編譯／測試失敗。
- 不跑100場、LAN、PIE、UE全驗收或release stage；ABI／生成hash及21/31保持，不維護omfx、不開runtime Lua。
- 決策與預防記E296；完整5.5仍留最後。
