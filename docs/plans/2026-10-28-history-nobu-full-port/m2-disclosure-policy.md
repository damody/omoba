# M2 spike：狀態與變數的揭露政策

建立日期：2026-10-28。對應 README §3.2 第 1 點。負責：工作線 B。

## 1. 問題

- 執行器在 authority 與每個隊伍的 replica 上都要跑出相同結果。
- replica 看不到敵方的隱藏狀態：敵方技能給的 buff、敵方的計數器、敵方 BuffStore 的大部分內容。
- E168（技能力量造成 attribute_hp mismatch）、E175（replica 前搖被誤取消）都是「一邊讀到另一邊沒有的狀態，做出不同決定」。

要驗證的規則：**會影響對方 replica 決策的判定，只能讀雙方都有的狀態。**

## 2. 現況：replica 實際上做哪些事

讀 `filtered_specs.rs`、`team_projector.rs`、`game_processor.rs` 的結果：

| 項目 | authority | team replica |
|---|---|---|
| 傷害結算（傷害管線第 ③～⑤ 段） | 全部 | MOBA 中，所有已揭露的實體都在 `DisclosedAuthorityCombatTargets`：`rules::damage::admit` 直接拒絕，不在本地扣血。生命、法力由 committed vitals 揭露 |
| 擲骰（失手、閃避、爆擊、攻擊暈眩） | 決定結果 | 也會跑（同一套系統），但 replica 的 ECS id 是隊伍本地的重新編號（`hero_tick.rs:339-342` 的註解），所以擲出的值和 authority 不同；結果只影響本地不結算的傷害 |
| 施法許可（法力、冷卻、目標合法性） | 決定 | 也判定；`process_outcomes` 以「批次開始」的世界判定，避免同 tick 隱藏效果翻轉結果 |
| 前搖取消、指令拒絕、攻擊目標驗證 | 決定 | 也判定；E175 改成讀揭露的快照（`DisclosedRecallingOwners`）與雙方都有的元件 |
| 施加 buff（`BuffStore::add` 的魔免阻擋、狀態抗性、緩速抗性） | 決定 | 只有己方單位的 buff 列（`status_rows`）要與 authority 一致；敵方單位的狀態由揭露覆寫 |

## 3. 三種揭露方式

| 政策 | 意思 | 誰看得到 | 現有例子 |
|---|---|---|---|
| `public` | 狀態本身隨單位揭露 | 看得到這個單位的所有隊伍 | 公開 buff 目錄（`BuffVisualState`：暈眩、緩速、`magic_immune`……）、`TAttack` 的計時（`CommittedAttack`）、回城快照 |
| `team` | 只有己方知道 | 擁有者的隊伍 | 技能給自己的 buff（A07T 的狀態抗性、魔抗池、緩速抗性）、偽隨機計數器、單次攻擊紀錄、技能變數 |
| `derived` | 只揭露結算結果 | 看得到這個單位的所有隊伍 | 生命與法力（committed vitals）、受到傷害加成（`CommittedIncomingDamage`）、三圍總值（`AttributeTotals::disclosed`，E168）、屬性面板 |

宣告的位置（M3 的 IR）：每個狀態與狀態變數在資料裡宣告 `disclose = "public" | "team" | "derived"`；預設 `team`。`derived` 要指定揭露的是哪個結算值。

## 4. 規則與檢查方式

一個判定**會影響對方 replica**，當它的結果會改變對方隊伍可見、而且對方 replica 也會自己判定的東西：
- 施法或攻擊能不能成立（冷卻、法力、指令佇列）；
- 前搖是否取消；
- 對方單位身上的 `public` 狀態（例如暈眩的有無）。

這類判定只能讀：
1. `public` 狀態；
2. `derived` 的已揭露結果（例如揭露的生命，而不是用對方的 BuffStore 重算）；
3. 判定方自己隊伍的 `team` 狀態，**而且**判定結果不會被對方 replica 重算（例如己方技能的冷卻是己方的事）。

不能讀：對方隊伍的 `team` 狀態，以及 tick 中途才改變的私有狀態（E175）。

## 5. 驗證（spike）

### 5.1 M2 新規則逐條檢查

| 規則 | 讀什麼 | 誰判定 | 結論 |
|---|---|---|---|
| 傷害管線（含 `require_source_alive`、法術增幅、魔抗池、魔免） | 來源與目標的 BuffStore（含對方的 `team` 狀態） | 只有 authority 結算（replica 的 `admit` 拒絕已揭露目標） | 可行：結果以 `derived`（vitals）揭露 |
| 狀態抗性 ignore | 施加的 payload（`public` 的狀態）與目標的抗性（`team`） | 施加時；對方單位的狀態由揭露覆寫 | 可行：狀態持續時間以 `public` 揭露，replica 不自己決定敵方狀態長度 |
| 減療取最強 | 受治療者自己的 BuffStore | authority 結算治療 | 可行（vitals 揭露） |
| 爆擊在前搖開始擲骰、偽隨機計數器 | 攻擊者自己的 BuffStore 與計數器 | 擲骰在 authority 有效；replica 擲出的值不同但不結算 | 可行，**但有條件**：見 5.2 |
| 攻擊事件（命中、未命中、取消） | 投射物判定 | 雙方都會產生事件 | 可行，前提是 M3 的事件處理只在 authority 產生會影響對方的效果 |

### 5.2 發現：擲骰用的是 ECS id

- `attack_accuracy::roll` 與 `rules::chance::unit_roll` 用 `(seed, tick, entity.id(), op)`。replica 的 ECS id 是本地編號，所以 replica 擲出的值和 authority 不同。
- 今天這不會造成 mismatch，因為 replica 不結算傷害。
- 但 M3 的資料技能若在 replica 上依擲骰結果做**己方可見的決定**（例如「爆擊時給自己一個 buff」、「觸發時重置自己的冷卻」），己方 replica 會和 authority 分歧。
- **M3 的規則**：
  1. 擲骰一律以揭露的身分為鍵（例如 `canonical_entity_id` 的 authority 值、或英雄的 player id），像 `hero_tick` 的閒置抖動一樣；或者
  2. 依擲骰結果的效果只在 authority 執行，結果以 `public`／`derived` 揭露。
  - 建議 1：成本低，而且讓 replica 的預測正確。
- 偽隨機計數器屬於 `team`；只要擲骰鍵一致，己方 replica 的計數器就會和 authority 相同。

### 5.3 以測試驗證

- `rules::*` 單元測試：只依輸入計算，不讀全域隱藏狀態。
- parity soak（雙方 replica 每 tick 對 authority）：A07（魔抗池、狀態抗性、緩速抗性同時生效）、B29T／C15T（ignore 的暈眩）、port8 全英雄加道具。結果記在 `execution-status.md` 的 M2 紀錄。
- 2026-10-28 結果：5 個英雄 parity soak 與 single_lane 的 `replicas_identical` 共 35 個，只有 gunner 視野邊緣 soak 失敗，而它在 live 基底同樣失敗（既有問題，與規則層無關）。實機局 `hero-port-8/live-8` 的 accepted inputs 重播 45895 tick，mismatch 0。

## 6. 結論

- 規則可行：M2 的所有新規則都符合「影響對方的判定只讀雙方都有的狀態」，因為傷害、治療與敵方狀態都在 authority 結算後以 `public`／`derived` 揭露。
- M3 要做的事：
  1. IR 的狀態與變數宣告 `disclose`，預設 `team`；
  2. 執行器在 replica 上遇到「讀對方 `team` 狀態的判定」時拒絕編譯；
  3. 擲骰改以揭露身分為鍵（5.2）。
