## 1. 現況盤點與差異比對

- [x] 1.1 確認 root 與 `omfx` worktree 狀態，避免覆蓋未提交修改。
- [x] 1.2 比對 `omfx` 舊 pregame UI commits：`9f88f55`、`8fe49fd`、`06baa16` 中 `game/src/native.rs`、`game/src/pregame.rs`、`game/src/backend_session.rs` 的相關區塊。
- [x] 1.3 建立可移植清單，標出 layout/render/hit-test/action dispatch 可直接沿用的區塊，以及必須重寫以配合目前 API 的區塊。
- [x] 1.4 確認目前 `omfx` 最新 `master` 的 lockstep timeout、session launcher、TD sidebar、tooltip、擊破數相關 code path，作為不得回退的保護清單。

## 2. Pregame Catalog 與 State 修復

- [x] 2.1 檢查 `scripts/base_content/assets/pregame_ui/catalog.json` 是否仍包含主畫面、地圖選擇、難度選擇所需資料。
- [x] 2.2 修復或補齊 `omfx/game/src/pregame.rs` 的 catalog model、validation、fallback 與 action enum，使其支援舊 UI 版面需要的欄位。
- [x] 2.3 確保 unknown action、missing story id、missing difficulty id、missing optional image asset 都安全降級並輸出 diagnostic log。
- [x] 2.4 保持 catalog 為 canonical source，不新增正常流程使用的 Rust hard-coded map/difficulty table。

## 3. UI Layout / Render / Input 恢復

- [x] 3.1 在目前 `native.rs` 架構上恢復 pregame main menu 的 UI node 建立、layout 更新與 start control。
- [x] 3.2 恢復 map select 畫面：地圖卡片、enabled/locked 狀態、返回、hit region 與選取後 transition。
- [x] 3.3 恢復 difficulty select 畫面：difficulty cards、描述/獎勵文字、返回、選取後 `StartingSession` transition。
- [x] 3.4 恢復 loading/error 狀態顯示，session startup 失敗時可回到 difficulty select 或可重試狀態。
- [x] 3.5 確保 pregame 狀態 consume mouse/key input，不送 tower placement、ability casting、start round 或 gameplay map click。
- [x] 3.6 確保 `InGame` 狀態仍使用目前 gameplay HUD、TD sidebar、tooltip、擊破數與 input routing。

## 4. Session Lifecycle 相容性

- [x] 4.1 保留目前 `backend_session.rs` session-scoped executable launcher，不整檔回退舊版。
- [x] 4.2 確認 map/difficulty selection 產生的 session config 仍包含 story/runtime identifier、difficulty、network address、session id。
- [x] 4.3 確認 menu idle 不啟動 backend、不建立 lockstep client、不 spawn `sim_runner`。
- [x] 4.4 確認 session startup 成功後才啟動 lockstep client 與 local `sim_runner`。
- [x] 4.5 確認返回 menu、game over、startup failure、plugin deinit 都走 idempotent session shutdown。
- [x] 4.6 確認 external backend mode 不關閉非 `omfx` 自己啟動的 backend process。

## 5. 測試與驗證

- [x] 5.1 新增或更新 pregame action dispatch tests，覆蓋 start、back、select map、select difficulty、disabled entry、unknown action。
- [x] 5.2 新增或更新 catalog validation tests，覆蓋 valid catalog、missing required config、missing optional asset fallback。
- [x] 5.3 新增或更新 session lifecycle tests，覆蓋 menu-only startup、session-owned launcher ownership、external backend mode、idempotent shutdown。
- [x] 5.4 執行 `cargo build --manifest-path omfx/Cargo.toml -p executor`。
- [x] 5.5 搜尋 `omfx/game`，確認沒有新增 `omobab =`、`omobab::`、frontend-owned `cargo run` / `cargo build` backend path。
- [ ] 5.6 手動 smoke：啟動 `omfx` → restored main menu → start → map select → difficulty select → loading → gameplay。
- [ ] 5.7 手動 smoke：返回 menu 或 session end 後，確認 session-owned backend process 被關閉，且可再開下一場。
- [ ] 5.8 手動 smoke：確認 TD sidebar、tooltip、擊破數、技能/tower input 在 gameplay 中仍正常。

## 6. 提交與整合

- [x] 6.1 在 `omfx` submodule commit pregame UI restore，commit message 說明保留 current session/runtime fixes。
- [ ] 6.2 在 root repo 更新 `omfx` submodule pointer。
- [x] 6.3 若有修改 `.bat`，確認 CRLF 行尾。
- [x] 6.4 更新 OpenSpec task checkbox 與最終驗證紀錄。
