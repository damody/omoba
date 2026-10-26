# 單機多程序測試拓撲與規則式 Autopilot

建立日期：2026-10-25（Asia/Taipei）。本文件補充 README §8 與 schedule 的 L3 驗收方式。第二台電腦到位前，所有多人驗收先在**一台電腦上跑 2 個 omfue client＋3 個 Rust 程序**；第二台到位後只把其中一個 client 移到另一台，其餘設定不變。

## 1. 拓撲

```text
                    ┌──────────────────────────────┐
                    │ omobab.exe（權威 server）      │  Rust #1
                    │ --server-only, KCP :57061     │
                    └──────────────┬───────────────┘
                 （可選）omoba-netem-proxy，每隊一個 client bind
              ┌────────────────────┴────────────────────┐
┌─────────────▼──────────────┐            ┌─────────────▼──────────────┐
│ omoba-client-runtime p1    │ Rust #2    │ omoba-client-runtime p2    │ Rust #3
│ --server HOST:PORT         │            │ --server HOST:PORT         │
│ --presentation-bind 127…   │            │ --presentation-bind 127…   │
└─────────────┬──────────────┘            └─────────────┬──────────────┘
      presentation IPC（只走 loopback）           presentation IPC
┌─────────────▼──────────────┐            ┌─────────────▼──────────────┐
│ omfue（UnrealEditor -game） │            │ omfue（UnrealEditor -game） │
│ ＋ Autopilot（team 1）      │            │ ＋ Autopilot（team 2）      │
└────────────────────────────┘            └────────────────────────────┘
               ＋ 8 個 server 端 bot（由 recipe 宣告）
```

「3 個 Rust server」= 1 個 `omobab` 權威＋每位人類各 1 個 `omoba-client-runtime`。netem proxy 是第 4 個可選 Rust 程序，只在網路矩陣測試時加入。

## 2. 現有入口（已讀程式碼，尚未在本計畫中執行）

`scripts/run_moba_role_ue.lua`／`scripts/moba_role_launch.lua` 已經支援：

| 模式 | 參數 | 程序 |
|---|---|---|
| A. 單一 host、多個本機人類 | recipe 宣告 2 個人類座位，或 `--local-player 1 --local-player 2` | 一次啟動 omobab＋2 runtime＋2 UE |
| B. 獨立權威 | `--server-only [--server-bind IPv4]` | 只有 omobab，產生 `match-plan.json` 給 client |
| C. 遠端 client | `--connect HOST_IPV4 --recipe HOST_FINAL.json --local-player ID` | 1 runtime＋1 UE，不啟動 server |

每次執行都有 private Windows Job 監管、owned-process journal、獨立輸出目錄與 `lifecycle.json`。工作流必須沿用它，不另寫新的 launcher。

依根目錄 README：預設配方是 `scripts/lua_data/moba_single_player.lua`（release、KCP `57061`、story `FOG_2TEAM_DEMO`）。準備階段寫出新的 `game.toml`，`STEP_FPS=60`，Lua 關閉。準備後的目錄有 `match-plan.json`、`selection-candidate.json`、`selection-lock.json` 與 `hero-catalog.json`，可用 `--recipe FILE.json` 重開。英雄可用 `--hero PLAYER_ID=HERO` 替換。兩位以上真人的互動選角走 `scripts/moba_shared_selection.lua`，它不是 LAN 共享大廳。autopilot 驗收預設以 `--hero` 或已鎖定的 `--recipe` 固定英雄，選角流程另外驗。

**建議**：每天的快速迭代用模式 A；正式 L3a 驗收使用 **B＋C＋C**（三次獨立呼叫），因為這和未來雙機 LAN 的結構完全相同。

## 3. 讀程式碼時發現、要先處理的缺口（schedule.md A2–A3）

以下是程式碼閱讀推論；除已註明「已確認」者外，必須先實際驗證再修：

1. **同機兩個 `--connect` 的 presentation port 衝突（已確認，`moba_role_launch.lua` 第 349–352 行）。** `moba_role_launch.prepare` 用 `127.0.0.1:(port+ordinal)` 當 presentation bind，而 ordinal 在每次呼叫內都從 1 開始。兩個模式 C 在同一台機器上會綁同一個 port。需要新增明確的 presentation port 基底參數，或用 player ID 推導 port；不得因此改變權威 port。
2. **CPU 預算重複計算。** `moba_host_budget.local_budget` 每次呼叫都假設自己獨佔整台機器。B＋C＋C 會各自分配，可能超額。需要一個選項把「同機總程序數」傳進去，或在單機模式下由一個協調入口統一分配。
3. **netem 尚未接到 role launcher。** `start_netem_proxy.lua` 是給舊的 two-team 腳本用的（team1/team2 client bind）。L4 網路矩陣前要讓模式 C 的 `--connect` 能指向 proxy 的 client bind，並在證據裡記錄 profile 與 seed。
4. **本機 UE 數量與 Editor 授權。** 兩個 `UnrealEditor -game -AllowMultipleInstances` 加上 Editor 本身很吃記憶體。在 A3 量測這台機器能否穩定同時跑 2 個 client＋（可選）1 個 Editor，寫入硬體基準。
5. **對局數不限。** 使用者 2026-10-25 授權：本機對局不限局數，舊 `10/10` 對局預算不再適用於本計畫。autopilot 可以每天跑多局；同一時間只跑一組拓撲，效能與 soak 量測時機器上不跑其他對局。

## 4. 規則式 Autopilot（取代「1 真人」的大部分驗收）

### 4.1 為什麼不能用現有 bot

`omoba-core/src/runtime/native/moba_match/bots` 的 bot 留在 server process。根目錄 README 記載它只讀已披露狀態，再送正式 `MoveTo`／`CastAbility`／`UpgradeAbility`／`ItemBuy`。但它不拿 KCP 連線，也完全不經過 omfue 與 client runtime。原 11 項缺口（3.1、3.3、3.4、3.5、4.3、7.1 等）出在 **UE 輸入 → request → 權威 ACK → UI** 這條路上，只有從 client 端驅動才驗得到。

### 4.2 契約

- **位置（已實作 v0）**：`omfue/Plugins/OmRuntime/Source/OmRuntime/Public/OmAutoPilot.h` 的 `FOmAutoPilot`，整個類別包在 `#if !UE_BUILD_SHIPPING` 內，Shipping 不編譯。由 `AOmPlayerController` 在 `-om-autopilot=<profile>` 時建立。launcher 參數是 `run_moba_role_ue.lua --autopilot PROFILE`，trace 寫到 `<run>/autopilot-p<id>.jsonl`，覆蓋統計寫到同名的 `.coverage.json`。
- **觀察**：只讀該玩家合法可見的資料。資料來源是 `UOmRuntimeBridgeSubsystem::OnPresentationFrameProcessed`：WorldBridgeActor 處理完 frame 後，在同一個 lease 內廣播，內容是 team-filtered 實體、owner HUD、公開 lane route 和 applied input 結果。不讀 server、不讀其他 client、不讀霧中資訊。
- **輸出（只有一條路徑）**：在 `FSlateApplication` 合成鍵盤和滑鼠事件，先 mouse move 再按鍵，指標是虛擬座標，不移動 OS 游標。按鍵一律從 control profile 的 `FindChordBinding(OmControlAction::...)` 查，所以玩家改鍵後照樣有效；滑鼠位置由世界座標投影後換成 viewport cached geometry 的 Slate absolute 座標，OS DPI 不是 100% 時也正確。事件會經過 PreGame preprocessor、UI 攔截、InputComponent 綁定，最後才到 bridge submission，和真人輸入同一條路。原本規劃的「input-component 模式」因此不再需要；J4 的 DPI／視窗矩陣也直接用這條路徑。
- **Trace 對應**：`OnGameplayInputTransportAccepted` 抓到合成事件對應的 input_id；frame 的 `applied_inputs` 給出權威結果（status 0 為 accepted，否則 rejected）。UI 吃掉、沒有送出的事件記成 `not_submitted`，不會繞道。
- **可以失敗**：被 UI 擋住、目標失效、沒藍、CD 中，都照實記錄 rejected，不得繞道。
- **行為（有限狀態機）**：開局買裝 → 上線 → 補刀／普攻 → 敵方英雄在射程內且資源足夠就放技能 → 低血撤退或回城 → 回泉水補給、購物 → 升級技能（固定順序）→ 有兵線就推塔 → 死亡等待重生 → 對局結束看結算 → 返回／下一局。
- **技能使用**：讀英雄 package 的 `bot_usage` metadata（目標類型、射程、保留資源）。生成的英雄只要 autopilot 能正確施放，就順便驗證了生成的瞄準與目標 metadata；盲測英雄也照用，不寫專用規則。
- **覆蓋模式**：每局至少觸發一次：右鍵移動、A、Shift+A、MoveOnly、AttackOnly、S、H、自用技能、升級、購買、合成、賣出、undo、插眼、ping、聊天、回城、死亡重生、小地圖點擊、計分板。結束時輸出 `coverage.json`，直接對應 G2 與原 11 項。
- **Trace**：每個動作一筆 `{tick, autopilot_intent, input_route, request_id, authority_result(accepted/rejected+reason), ui_observation}`，綁定 build hash、content hash、recipe、seed。
- **確定性**：給定 seed 時行為可重現；不追求打得好，追求穩定打完、每種操作都用過。

### 4.3 驗收層級改寫

README §8 的 L3 拆成兩種，日報分開記錄：

- **L3a**：真 runtime／authority＋真 omfue 輸入管線，由 autopilot 驅動。G2、G3 的對局部分與原 11 項的大部分證據使用 L3a。
- **L3h**：使用者親自試玩。只在每個里程碑（schedule.md §2 的 M1–M4）各一局，判斷手感、可讀性、HUD 與美術；autopilot 無法驗收這些。

L4 的「真雙機 LAN」仍需第二台電腦。單機 B＋C＋C 標為 **L3a-loopback**，不得記成 LAN 通過。

## 5. 分階段

| 階段 | 對應工作（schedule.md） | 內容 | 完成條件 |
|---|---|---|---|
| T0 | A3 | 用模式 A 跑 2 UE＋3 Rust，量測記憶體／CPU，記錄是否可穩定運行；驗 §3 的 1、2 兩點 | 有可重複的啟動命令與資源基準 |
| T1 | A2、A3 | 修 presentation port 與 CPU 預算；B＋C＋C 在單機跑通到雙方進場 | 三次獨立呼叫、兩個 UE 都看到對局，owned cleanup 乾淨 |
| T2 | B1 | Autopilot v0：右鍵移動、普攻、trace | 一局內產生 trace，可對到權威 ACK |
| T3 | B2、B3 | Autopilot v1／v2：A、Shift+A、S/H、技能；商店、升級、回城、插眼、ping、聊天、覆蓋模式 | `coverage.json` 全部項目有結果（通過或明確 rejected 原因） |
| T4 | M1 | B＋C＋C，2 個 autopilot 對抗＋8 bot，自然打完一局並進下一局 | G2 的 L3a 證據；使用者 L3h 一局 |
| T5 | M2 | 同上，加入 H01／H02 與兩種外觀 | 雙 client 內容 hash 一致、無資訊洩漏 |
| T6 | J1 | netem 接入模式 C；斷線 10 秒重連 | 已定義 profile 下的 L3a-loopback 網路證據 |
| T7 | J2（第二台到位後） | 把一個模式 C 移到第二台，`--server-bind` 改 LAN IPv4 | 真 L4 LAN；同一 build／content hash |
| T8 | J5 | 2 個 autopilot 跑 2 小時 soak 與效能量測 | 固定負載、可重現的 profile |

## 6. 換到雙機時要改的東西

只改這些：

- 權威：`--server-only --server-bind <LAN IPv4>`。
- 第二台：`--connect <主機 LAN IPv4> --recipe <host 的 match-plan.json> --local-player 2`。
- 兩台使用同一 build 與 content hash；第二台需要同版 `omoba-client-runtime.exe`、omfue、base_content。
- 防火牆：launcher 不修改防火牆（help 已載明），由使用者手動開放 KCP port。

其餘：autopilot profile、trace 格式、coverage、證據目錄結構都不變，單機與雙機結果可以直接比較。
