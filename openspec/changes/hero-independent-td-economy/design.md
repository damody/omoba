## Context

目前 TD 的起始金錢與後續消費都依附在 `Hero + PlayerOwner + Gold` 實體組合。`OMB_NO_HEROES=1` 在 hero creation boundary 提前返回後，玩家不再有錢包實體，`player_hero_entity` 也使建塔、升級與出售直接失敗；回合收入同樣只對 Hero 產生 `GainGold`。omfx 又只從 Hero snapshot 更新 `hero_state.gold`，因此畫面與權威模擬都顯示零金錢。

限制包括：TD 維持兩個獨立玩家帳戶、lockstep 結果必須 deterministic、`run_10000.bat` 的 10,000 覆寫必須保留、MOBA 的 Hero 金錢／道具／經驗不得受影響，而且不能用隱形 Hero 或 render entity 規避問題。

## Goals / Non-Goals

**Goals:**

- 讓 TD 金錢生命週期完全獨立於 Hero entity。
- 保留目前 Bloons 對齊的 650 起始金錢、既有 round income table 與 `OMB_TD_STARTING_GOLD` 覆寫。
- 讓建塔、升級、出售、擊破獎勵、回合收入、snapshot、HUD 與 state hash 使用同一份權威資料。
- 失敗命令不得部分扣款或加款，缺少帳戶時須提供可診斷錯誤。

**Non-Goals:**

- 不調整塔價、升級價、退款公式、回合資料或獎勵數值。
- 不加入共享錢包、玩家轉帳、農場或其他收入塔。
- 不重構非 TD 的 Hero 道具、技能、經驗或金錢流程。
- 不新增外部 wire protocol 或持久化 migration。

## Decisions

### 1. 以 `PlayerEconomy` resource 作為 TD 唯一權威錢包

`omoba-core` 新增 `PlayerEconomy`，內部使用 `BTreeMap<u32, i32>`，提供 `balance`、`initialize`、`try_debit` 與 `credit_saturating`。ordered map 讓 snapshot 與 hashing 不受插入順序影響；方法封裝讓負成本、帳戶不存在及餘額不足都在 mutation 前拒絕。

不採用「隱形錢包 entity」，因為它會混入 entity lifecycle、snapshot 分類與場景清理；不採用「隱形 Hero」，因為它違反零 Hero 需求並保留原耦合。

### 2. TD mode 初始化帳戶，Hero 金錢僅維持相容

`init_creep_wave` 解析出 `GameMode::TowerDefense` 與 `TdDifficultyConfig` 時，初始化 player 1、player 2 帳戶。帳戶建立發生在 hero creation 之前且不讀取 `OMB_NO_HEROES`。測試透過接受 resolved config 的 helper 驗證 650 與 10,000，不修改 process-global env。

Hero-enabled TD 可保留 Hero 上的 `Gold` component 供舊的 hero-specific 顯示或程式碼使用，但塔防交易與 TD reward 僅寫 `PlayerEconomy`，避免雙重權威。MOBA 不建立或不使用 TD 帳戶。

### 3. 塔操作以 player ID 直接交易

建塔與升級先完成 entity、owner、規則、位置及模板驗證，再呼叫 `try_debit(owner_pid, cost)`；出售先驗證 tower ownership 與退款數字，再 `credit_saturating`。塔操作不再呼叫 `player_hero_entity`，也不再為交易清除 Hero command queue。Hero command、ability、item 等非塔操作仍保留原 helper。

### 4. TD reward 依帳戶與來源 owner 路由

round clear 直接將 Bloons round income 加到所有已初始化帳戶。TD creep death 若 damage source 有 `PlayerOwner`，將 bounty credit 給該 owner；無 owner 時不猜測玩家。非 TD 仍採現有鄰近 Hero 金錢與經驗流程。

### 5. Snapshot 直接發布玩家金錢

`SimWorldSnapshot` 新增 ordered `player_gold` 欄位，從 `PlayerEconomy` 複製。omfx 每次收到 snapshot 時，若存在 `local_player_id` entry，就更新既有 `hero_state.gold`；Hero metadata、生命、技能與 `entity_id` 仍只由 Hero entity 更新。這讓既有 HUD、shop affordability 與送出建塔前檢查復用同一欄位，不建立假 Hero。

### 6. State hash 納入帳戶資料

`omb::lockstep::compute_state_hash` 將排序後的 `(player_id, balance)` 追加至 hash input。測試固定「餘額改變會改 hash」及「不同插入順序相同 hash」。render-only snapshot extraction 不修改帳戶。

### 7. 調整證據的分類

- A（任務微調）：在不改需求下拆分測試、重排實作順序或改用等價測試指令。
- B（設計／spec 修正）：在既定範圍內若實際 death outcome 無法取得 source owner，須先更新 reward routing 設計、spec、tasks 並重新驗證。
- C（重大變更）：改成共享錢包、改起始金額／收入表、改玩家數、加入 wire protocol 或擴大非 TD 經濟範圍，必須取得使用者核准。

## Risks / Trade-offs

- [Risk] Hero-enabled TD 的 Hero `Gold` 可能與帳戶數字不同 → TD UI 與塔操作明確只讀 `PlayerEconomy`；測試禁止交易改動 Hero Gold。
- [Risk] 部分測試手動建立 World 而未插入 resource → economy 交易回傳明確 missing-resource/account 錯誤；共用 TD fixture 必須插入帳戶。
- [Risk] pop reward 的 source 在某些 damage path 缺失 → 僅有確定 `PlayerOwner` 時入帳，不將獎勵錯配給其他玩家。
- [Risk] snapshot 新欄位使大量 struct literal 編譯失敗 → 使用 `..Default::default()` 或逐一補欄位，並跑 omoba-core、omb、omfx 完整相關測試。
- [Trade-off] 暫時保留 Hero `Gold` component 造成 TD 中存在相容鏡像，但可避免擴大到道具／英雄 UI 重構；它不再是 TD authority。

## Migration Plan

1. 新增並註冊 `PlayerEconomy`，先以單元測試固定 API 與 deterministic ordering。
2. 將 TD initialization 與 tower transactions 切換到 resource，再修改 rewards。
3. 擴充 snapshot、omfx consumption 與 state hash。
4. 跑 focused tests、omoba-core、omb、omfx game tests及無英雄啟動 smoke。
5. 若需 rollback，回退本 change；`Gold` component 與非 TD 行為未移除，沒有資料 migration。

## Open Questions

無。若實作證據顯示 source ownership contract 不成立，依 B 類修正流程更新 artifacts，而不是猜測入帳玩家。
