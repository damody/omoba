## 1. 權威玩家經濟模型

### 1.1 `PlayerEconomy` resource

**目的：** 建立可獨立於 entity 使用、具 deterministic ordering 與安全交易語意的 TD 錢包。
**輸入：** 核准設計、`omoba-core/src/runtime/native/comp` 既有 resource pattern。
**產出：** `PlayerEconomy` module、re-export、World resource 初始化及單元測試。
**依賴：** 無。
**Owner／Wave：** primary／wave 1。
**Gate／Evidence：** PE-API；`evidence/index.jsonl`。
**完成門檻：** API 測試涵蓋成功、missing account、insufficient funds、負成本、saturating credit 與插入順序。

- [x] 1.1.1 新增 `PlayerEconomy` ordered account model 與 `balance`、`initialize`、`try_debit`、`credit_saturating` API。
- [x] 1.1.2 將 `PlayerEconomy` re-export 並插入 campaign／standard ECS World 的 resource 初始化路徑。
- [x] 1.1.3 新增 focused model tests，並將命令、exit status 與結果記錄到 `evidence/index.jsonl` 的對應 task IDs。

### 1.2 TD 帳戶初始化

**目的：** 在 hero creation 之前以 resolved config 初始化 player 1、player 2。
**輸入：** 1.1、`TdDifficultyConfig`、`init_creep_wave`。
**產出：** environment-independent initializer helper 與零英雄測試。
**依賴：** 1.1。
**Owner／Wave：** primary／wave 1。
**Gate／Evidence：** PE-INIT；`evidence/index.jsonl`。
**完成門檻：** 650 與 10,000 兩種 resolved config 都建立兩個正確帳戶，Hero 數量可為零。

- [x] 1.2.1 在 TD mode 初始化流程呼叫接受 resolved config 的 player-account helper，且不讀取 Hero 或 `OMB_NO_HEROES`。
- [x] 1.2.2 調整 hero-enabled TD 相容路徑，確保 Hero `Gold` 不成為塔交易的 authority。
- [x] 1.2.3 新增預設 650、覆寫 10,000 與零 Hero 初始化測試並記錄 evidence。

## 2. TD 交易與獎勵路由

### 2.1 塔交易

**目的：** 建塔、升級與出售完全使用 requesting player account。
**輸入：** 1.2、既有 tower validation／refund logic。
**產出：** 無 Hero 依賴的 tower transaction handlers 與 rollback-safe tests。
**依賴：** 1.2。
**Owner／Wave：** primary／wave 2。
**Gate／Evidence：** PE-TXN；`evidence/index.jsonl`。
**完成門檻：** 成功交易只改 owner 餘額；missing／insufficient／ownership failure 均不產生部分 mutation。

- [x] 2.1.1 將 tower placement validation 與 debit 從 `player_hero_entity`／`Gold` 切換為 `PlayerEconomy`。
- [x] 2.1.2 將 tower upgrade debit 與 tower sale credit 切換為 `PlayerEconomy`，移除交易對 Hero command queue 的依賴。
- [x] 2.1.3 新增無英雄成功建塔、升級、出售與跨玩家隔離測試並記錄 evidence。
- [x] 2.1.4 新增 missing account、insufficient balance 與 ownership rejection 不異動測試並記錄 evidence。

### 2.2 Bloons 式收入

**目的：** round income 與可歸屬的 pop bounty 在零英雄時仍進入正確玩家帳戶。
**輸入：** 1.2、`creep_wave`、death／damage outcome source metadata。
**產出：** account-based reward paths 與 MOBA regression tests。
**依賴：** 1.2。
**Owner／Wave：** primary／wave 2。
**Gate／Evidence：** PE-REWARD；`evidence/index.jsonl`。
**完成門檻：** round income credit 所有帳戶；owned source 只 credit owner；unknown source 不入帳；MOBA 行為不變。

- [x] 2.2.1 將 TD round income 從 Hero-targeted `GainGold` 改為所有 initialized accounts 的 saturating credit。
- [x] 2.2.2 查證 death outcome 的 source ownership，將 TD creep bounty 路由至確定的 `PlayerOwner` account；若 contract 不成立則依 B 類流程先修 artifacts。
- [x] 2.2.3 新增零英雄 round income、owned tower bounty、unknown source 與 MOBA regression tests並記錄 evidence。

## 3. Snapshot、前端與 deterministic hash

### 3.1 Snapshot contract

**目的：** render snapshot 不靠 entity 即可提供玩家金錢。
**輸入：** 1.2、`SimWorldSnapshot` 與兩條 extraction paths。
**產出：** ordered player cash snapshot field、read-only tests 與更新後的 struct literals。
**依賴：** 1.2。
**Owner／Wave：** primary／wave 3。
**Gate／Evidence：** PE-SNAPSHOT；`evidence/index.jsonl`。
**完成門檻：** 零 Hero snapshot 含正確餘額，重複 extraction 不改 resource，相關 crates 編譯。

- [x] 3.1.1 擴充 `SimWorldSnapshot` 並在 native extraction paths 複製 ordered player balances。
- [x] 3.1.2 更新 snapshot struct literals／fixtures並新增零 Hero 與 extraction read-only tests，記錄 evidence。

### 3.2 omfx 本機玩家金錢

**目的：** HUD 與 affordability 使用 `local_player_id` 帳戶且不建立假 Hero。
**輸入：** 3.1、omfx snapshot consumption 與現有 `hero_state.gold` consumers。
**產出：** player-cash consumption helper 與 frontend tests。
**依賴：** 3.1。
**Owner／Wave：** primary／wave 3。
**Gate／Evidence：** PE-UI；`evidence/index.jsonl`。
**完成門檻：** zero-Hero、multi-player local selection、missing-local-account 三個情境通過。

- [x] 3.2.1 在每次 snapshot consumption 時獨立更新 `local_player_id` 的 `hero_state.gold`，Hero metadata path 保持原語意。
- [x] 3.2.2 新增 zero-Hero、多玩家與 missing-local-account UI state tests並記錄 evidence。

### 3.3 authoritative state hash

**目的：** 玩家金錢變化可被 lockstep desync detection 偵測。
**輸入：** 1.1、`omb/src/lockstep/state_hash_producer.rs`。
**產出：** economy hash contribution 與 deterministic tests。
**依賴：** 1.1。
**Owner／Wave：** primary／wave 3。
**Gate／Evidence：** PE-HASH；`evidence/index.jsonl`。
**完成門檻：** balance difference 改變 hash，相同帳戶以不同插入順序產生相同 hash。

- [x] 3.3.1 將排序後的 player balances 納入 `compute_state_hash` authoritative bytes。
- [x] 3.3.2 新增 balance-change 與 insertion-order hash tests並記錄 evidence。

## 4. 整合驗證與交付

### 4.1 自動測試 gates

**目的：** 證明核心、後端與前端沒有 regression。
**輸入：** phases 1–3 完整實作。
**產出：** focused 與 full-suite evidence records。
**依賴：** 2.1、2.2、3.1、3.2、3.3。
**Owner／Wave：** primary／wave 4。
**Gate／Evidence：** PE-CORE、PE-BACKEND、PE-FRONTEND；`evidence/index.jsonl`。
**完成門檻：** 所列測試命令 exit 0；ignored tests只接受既有標記。

- [x] 4.1.1 執行 focused player economy／tower／reward／snapshot／hash tests並記錄完整結果。
- [x] 4.1.2 執行 `cargo test --manifest-path omoba-core/Cargo.toml --no-fail-fast` 並記錄 evidence。
- [x] 4.1.3 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab --no-fail-fast` 並記錄 evidence。
- [x] 4.1.4 執行 `cargo test --manifest-path omfx/Cargo.toml -p game --no-fail-fast` 或實際 package 名稱的等價 frontend suite並記錄 evidence。

### 4.2 無英雄啟動與最終審查

**目的：** 驗證實際 launcher 有起始金錢且 change 可安全交付。
**輸入：** 4.1、`run.bat`、`run_10000.bat`。
**產出：** bounded smoke、OpenSpec validation、final diff review。
**依賴：** 4.1。
**Owner／Wave：** primary／wave 4。
**Gate／Evidence：** PE-SMOKE、PE-SPEC、PE-DIFF；`evidence/index.jsonl`。
**完成門檻：** 無 Hero、player 1 顯示／可使用正確起始金錢、無殘留程序、strict validation 通過且未改動無關 `omfue` state。

- [x] 4.2.1 執行 bounded hero-free launcher smoke，驗證帳戶初始化 log／snapshot 金錢且清理完整 process tree。
- [x] 4.2.2 執行 `openspec validate hero-independent-td-economy --strict` 與 artifact placeholder／contradiction scan並記錄 evidence。
- [x] 4.2.3 審查 final diff、CRLF launcher bytes、既有 `omfue`／submodule state 與所有 evidence task IDs，修正範圍外變更。
