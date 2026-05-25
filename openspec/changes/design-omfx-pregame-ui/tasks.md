## 1. Session 狀態與啟動邊界

- [x] 1.1 在 `omfx/game` 新增 pregame/session state 型別，涵蓋 `MainMenu`、`MapSelect`、`DifficultySelect`、`StartingSession`、`InGame`、`SessionEnded`。
- [x] 1.2 將 `omfx/game/src/native.rs` 目前在 `Plugin::init` 內啟動 `lockstep_client` 與 `sim_runner` 的邏輯拆到 `start_game_session(selection)`。
- [x] 1.3 確保預設啟動只初始化 menu 必要場景/UI，不建立 `lockstep_client`、不 spawn `sim_runner`、不連線 backend。
- [x] 1.4 新增 session config 型別，保存從 scripts catalog 解析出的 `session_id`、`map_id`、story/runtime identifier、difficulty id、network address 與 local player 設定。

## 2. Scripts Pregame Content Catalog

- [x] 2.1 在 `scripts/base_content` 新增 pregame UI catalog/manifest 與預設資料，涵蓋主畫面、地圖選擇、難度選擇。
- [x] 2.2 定義 catalog schema，包含 screen id、widget/card id、顯示文字、圖片 slot/path、enabled/locked 狀態、action id、map id、story/runtime identifier、difficulty id 與 reward/description metadata。
- [x] 2.3 實作 `omfx` catalog loader，優先載入 scripts content mod 資料，frontend fallback 僅作診斷與開發保底。
- [x] 2.4 實作 catalog validation，未知 action、缺少 story id、缺少 difficulty id 或缺圖都要 log，且不得 panic。
- [x] 2.5 文件化 content/mod 作者應修改 scripts catalog 與 scripts assets，不修改 `omfx` Rust hard-coded table。

## 3. Backend Session Launcher

- [x] 3.1 新增 `backend_session` module，透過 env/config 找到已建置好的 `omobab.exe`，不得依賴 `omobab` crate。
- [x] 3.2 實作 backend executable 啟動，傳入 scripts catalog 選出的 `OMB_STORY`、difficulty/session metadata、KCP address 與必要 content path。
- [x] 3.3 實作 idempotent shutdown，支援正常結束、啟動失敗、返回 menu 與 `on_deinit` 清理。
- [x] 3.4 支援 external backend mode，明確停用 session launcher 時只連線外部 backend，且 teardown 不關閉非自己啟動的 process。
- [x] 3.5 加入啟動/關閉 log，包含 `session_id`、`map_id`、story/runtime identifier 與 difficulty id。

## 4. Pregame UI Flow

- [x] 4.1 實作主畫面 layout：資料與圖片 slot 來自 scripts catalog，視覺上是滿版 lobby 背景、底部大型開始按鈕、可保留但不送 gameplay input 的周邊入口。
- [x] 4.2 實作白名單 pregame action dispatch，支援 `Navigate`、`Back`、`SelectMap`、`SelectDifficulty`、`StartSession`、`NoOp`。
- [x] 4.3 實作主畫面點擊 catalog start action 後切換到地圖選擇，且不啟動 backend。
- [x] 4.4 實作地圖選擇卡片 layout，卡片、預覽圖、locked/disabled 狀態與 story/runtime identifier 來自 scripts catalog。
- [x] 4.5 實作地圖選擇返回主畫面、選 enabled map 前往難度畫面、點 locked map 不進入下一步。
- [x] 4.6 實作難度選擇畫面，選項、圖示、獎勵/描述與 difficulty config 來自 scripts catalog，base_content 預設提供 easy/medium/hard。
- [x] 4.7 實作難度選擇後進入 `StartingSession` loading 狀態，禁止重複點擊造成多個 session。

## 5. Gameplay 進入與離開

- [x] 5.1 在 `StartingSession` 成功啟動 backend 後建立 `lockstep_client`，再以相同 story/runtime config spawn `sim_runner`。
- [x] 5.2 進入 `InGame` 後恢復既有 gameplay renderer、HUD、tower/ability input routing 與 lockstep event drain。
- [x] 5.3 在 pregame 狀態遮蔽或停用 gameplay hotkeys、map click、tower placement、ability casting 與 `StartRound` input。
- [x] 5.4 實作 gameplay 返回 menu 或 exit-session action，依序停止 gameplay input、drop lockstep/sim_runner、shutdown backend。
- [x] 5.5 串接權威 game-over/session-end 條件；若目前缺少明確訊號，先以可測的 existing end state 或 TODO guard 實作最小 teardown path。
- [x] 5.6 確保 `Plugin::on_deinit` 對 loading/in-game session 呼叫同一套 teardown 且不 panic。

## 6. Dev Flow 與設定

- [x] 6.1 調整 native dev run 設定，使 menu idle 不預先常駐 backend；必要時新增 env flag 保留 legacy autostart。
- [x] 6.2 確認 `.bat` 修改後維持 CRLF 行尾。
- [x] 6.3 定義 release/dev 下 backend executable path 的查找順序，避免依賴 `omb/` source layout。
- [x] 6.4 文件化 scripts catalog 的 map/difficulty 到 story/runtime config mapping 與 external backend mode 用法。

## 7. 驗證

- [x] 7.1 執行 `cargo build --manifest-path omfx/Cargo.toml -p executor`，確認 native frontend build 通過且不依賴 `omobab` crate。
- [x] 7.2 新增或更新 automated test，驗證 menu-only startup 不建立 lockstep/sim_runner/backend session。
- [x] 7.3 新增或更新 catalog loader/action dispatch test，驗證 scripts catalog 可替換 map/difficulty/UI action 且 unknown action 安全 no-op。
- [x] 7.4 新增或更新 session launcher test，驗證啟動 config、idempotent shutdown、external backend mode ownership。
- [ ] 7.5 手動 smoke：啟動 `omfx` → 從 scripts catalog 顯示主畫面 → 開始 → 選地圖 → 選難度 → 進入既有 gameplay 畫面。
- [ ] 7.6 手動 smoke：替換 scripts catalog 的文案或 map/difficulty entry 後，不改 `omfx` source 即反映在 pregame UI。
- [ ] 7.7 手動 smoke：遊戲中返回 menu 或觸發結束後，確認 session-owned `omobab.exe` 被關閉且可再開下一場。
- [x] 7.8 搜尋 `omfx/game`，確認沒有 `omobab::`、`omobab =`、frontend-owned `cargo run`/`cargo build` backend startup path。
