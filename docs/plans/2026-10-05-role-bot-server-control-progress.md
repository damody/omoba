# 正式 server Bot 控制權與輸入接線（2026-10-05）

## 計畫與決策

1. 分開完整對局名冊、server-owned Bot 控制權與外部真人認證；不得把 Bot 加進認證表以補足投影。
2. Bot 使用既有 `role_bot_inputs` 的隊伍 committed disclosure 與 owner gate，輸出正式 PlayerInput。接入現有 server accumulated／MobaMatch input gate／canonical accepted projection／PendingPlayerInputs／phase dispatcher，不直接改 ECS。
3. Lua 是完整配方來源。server 新增可省略的 `MATCH_ROLE_PLAN_JSON`；缺值時保留既有 Story／單路／三路啟動行為。配方模式必須 ThreeLane＋SecureV2Required，最多64 KiB；地圖與其他英雄選擇設定衝突拒絕，不暗中覆蓋。
4. `AUTHENTICATED_TEAM_BINDINGS` 必須與配方 `bot=false` 玩家及隊伍完全一致，允許只有一隊有真人、另一隊全部 Bot。完整 roster 仍由共用 plan compiler 保證兩隊／最多十人／合法 hero 與 named lane。範例十 Bot 配方不要求虛構真人登入。
5. main 保留既有 KCP authorize_player_team 真人表，建立 match 後、tick 前安裝獨立不可重裝的 ServerRoleBotControllers。安裝時再次驗真人與 Bot 對 roster 的完整互斥分割；開局後或第二次安裝拒絕。
6. 每 tick 從外部通道取得輸入後，先剔除 Bot／未知 controller ID，再加入 server 產生的 Bot 輸入。穩定依 player ID 排序，保留同真人既有輸入順序；內部 Bot 不借用 renderer request ID，correlation=0，team projector 仍按既有機制產生隊伍內輸入序號。
7. canonical accepted projection 從驗證過的控制名冊取 team，真人與 Bot 一起沿既有 sanitized actor／target 映射；無 Bot 模式保留舊 AUTH lookup。未更改投影 audience、對方輸入隔離、phase 次序、披露延遲或可靠出站發送。
8. plan compiler 增加 `compile_with_tick_rate`，server 直接使用自己的 STEP_FPS 計算思考間隔，不假設 headless profile。原 compile(profile) 委派至同一實作；60Hz功能確認，90Hz僅確認設定間隔18，不宣稱90Hz完整對局驗收。

## Lua 設定入口

新增 `scripts/lua_data/moba_single_player.lua`：重用共用配方，僅在內容層宣告 player1 為真人，形成一真人＋九 Bot；不是 Rust／Unreal 的特例。舊 `moba_role_match.lua` 仍十 Bot。兩者共用16個技能學習步驟與既有施法政策。

固定 Lua 匯出對應 server 欄位：

```text
tools/lua/lua.exe scripts/export_moba_role_plan.lua scripts/lua_data/moba_single_player.lua omb/target/moba-headless/single-player-server-fields.toml --server-fragment
```

輸出是要替換至 `[server]` 的欄位片段，包含 mode、secure mode、map、JSON 配方、僅真人認證與空 hero overrides；不是整份 game.toml，也不能盲目附加造成重複 key。既有 JSON 匯出 CLI 仍相容。來源 Lua 不被覆寫。

目前完成正式 server bootstrap／tick 接線与可機器驗證的配方欄位入口；尚未將新片段自動合併進正式一真人 Unreal launcher，也未改預設 game.toml。現有 Lua TOML lib 只有簡化 scalar decoder、不具完整 parser／encoder，因此沒有用它擅自重寫含 inline table 的設定。

## 當前功能確認

- `cargo test --manifest-path omb/Cargo.toml -p omobab role_server --lib -- --nocapture`：最後3/3 passed。
  - 一真人／九 Bot 的真實 server setting 編譯為完整十人 roster。真人 secure registration 成功；其餘九 Bot 逐一 register_secure_player 明確拒絕無認證 binding。錯隊、加Bot、缺真人、Story、非Required、英雄來源混用、地圖衝突、無效／過大JSON均拒絕。
  - 固定Lua產生 TOML 片段後，使用正式 `toml::from_str` 讀取並通過設定驗證。最後擴充此指定測試再跑1/1 passed，涵蓋十Bot與一真人九Bot兩份配方，認證表分別為空與唯一player1/team1，技能學習16步。
  - 真正 `State::tick`、共用Specs dispatcher、正式 Wave B delay1與team projector：先兩tick建立披露，合併2個真人命令＋7個初始Bot命令，兩Support合法hold。真人命令順序77→78保留；冒用Bot2與未知99、correlation999均移除。所有accepted team符合roster；下一tick實際安全frames team1僅[1,1,2,3,5]、team2僅[6,7,8,10]，不發布對方輸入。後續60tick真人及九Bot全部有實際位移、未執行極遠spoof目標。
- `cargo check --manifest-path omb/Cargo.toml -p omobab --bin omobab`：exit0，main bootstrap可編譯。最後production程式碼不再變更；後續僅新增Lua recipe／相關fixture。
- 根repo與omb正常換行設定的 diff --check exit0。既有編譯 warnings保留。
- 本批正式State::tick fixture是in-memory、不載入DLL且無技能政策，用於確認controller／movement／projection流程；學習／施法的正式ECS確認是前批證據。不是DLL／KCP socket／external filtered runtime／Unreal整合或100場驗收。

## 錯誤與後續

E163保留三次State fixture失敗、TOML Value轉換失敗及工具讀取／換行檢查問題；均修正後再確認，不取消安全gate。

下一步：使用完整TOML處理能力製作共用Lua launcher設定合併、接外部runtime與Unreal單真人流程；再完善其他技能類型、三種完整英雄原型與Bot戰術。完整filtered／KCP／Unreal／LAN／100場與效能驗收集中最後。5.5仍未全項完成，20/30不變。
