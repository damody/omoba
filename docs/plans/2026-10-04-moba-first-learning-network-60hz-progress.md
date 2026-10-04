# 純 Lua 英雄首次學習：真實 KCP 60Hz 增量

## 結果與範圍

完成 `training_apprentice` 的純 Lua 宣告、server-owned 開局英雄配置，以及雙 runtime 真實 KCP 首次學技能驗收。沒有直接注入 rank／SP／傷害，也沒有為新英雄編寫專屬 Rust module、C++ 或 Blueprint graph。

這不是整個 MOBA 框架完成，也不是雙 Unreal rank0 按鍵／像素或穩定 60FPS 驗收。OpenSpec 仍為 19/30；5.3、6.2 等完整項維持未勾選。BpGeneratorUltimate 保留作 Editor 資產／Blueprint 整合工具，本輪 native-only 英雄不需生成新的 Blueprint，也未把 build-only 冒充 MCP／PIE。

## 計畫、問題與決策

1. 檢查原架構：單路 server 原本固定 `training_luminary`，無法正常使用另一個 Lua 出生配置。新增 `AUTHENTICATED_HERO_BINDINGS`，只接受 authenticated roster 玩家及 active compiled hero；未指定仍用舊英雄。不是信任 client 任意 wire 選角。
2. 新增 append-only `training_apprentice`，出生四槽 rank0／一點技能點，四個獨立 `apprentice_*` 技能。舊 hero／skill ID 和預設 loadout 不改，`apprentice_lance` 第一級要求英雄 level6；原 `lumen_lance` 第一級仍 level1。既有註冊依英雄產生，複用 skill ID 會重複 FFI handler，因此採獨立追加 ID。
3. 以明確 `OMOBA_FIRST_LEARN_SMOKE=1` 加 test-mode 的一般 renderer intent，先未學施法／等級不足升級，再首次學習、同 request 重送及學後施法；所有命令走正式 runtime／KCP／authority 路徑。Unreal launcher 明確關閉此內部旗標，避免把 runtime injection 誤稱 Unreal 操作。
4. 真實對局與只讀 raw verifier 分開。逐 snapshot 對原始 committed rank／progression；transport acceptance 不是 gameplay 成功。補可重用 Lua verifier，記錄 SHA256 且驗證期間原始證據不得改動。
5. 記錄失敗再修正：u64 request、String array、hardcoded registry 數量、server WARN stream、退出前 checkpoint 追加等詳見錯誤紀錄 E123；未降低正式資料驗證或 gameplay 門檻。

## 實作入口

- Lua 來源：`scripts/lua_data/templates/heroes.lua`、`abilities.lua`。
- Server 配置：`omb/src/config/server_config.rs`，新增可選 `AUTHENTICATED_HERO_BINDINGS`。
- 測試輸入：`omoba-client-runtime/src/presentation_bridge.rs`、`main.rs`。
- 對局入口：`scripts/run_moba_runtime_smoke.lua`；只讀驗證：`scripts/verify_moba_first_learn_run.lua`、runtime `evidence.rs`。
- 測試：`single_lane_match_tests.rs` 的真實 generated Lua 出生／level gate 回歸，及 registry 精確清單／唯一性／effect metadata。

在既有合法 `server` 配置下，可另加：

```toml
[server.AUTHENTICATED_HERO_BINDINGS]
"1" = "training_apprentice"
"2" = "training_apprentice"
```

配置僅在 single_lane 使用，玩家必須已在 authenticated team roster，所有 peers 須由相同 Lua 重建；不是動態選角或 hot reload。

## 真實 60Hz 證據

Run：`target/interactive-runs/moba-runtime-1791102528`。

- launcher `success=true`、`cleanup_verified=true`、`tick_rate_hz=60`，route=`runtime-renderer-intent-injection`、`injected_gameplay_state=false`。
- p1 首次學後成功施法 tick208；p2 tick375。初始未學施法均被 authority 拒絕，level1 的 slot2 升級因 required level6 被拒絕，rank／SP／CD 不變。
- 每位玩家正式 upgrade input3 精確接受一次，同 renderer request 雙送未分配第二個 input 或扣第二點；rank0→1、SP1→0，學後 slot3 有正冷卻。
- raw verifier：p1 1,057／p2 525，合計 **1,582 live snapshots**。每筆 IPC rank／level／SP 對 exact authority tick（IPC tick−1）的原始 wire facts；沒有跨隊 private input 洩漏。
- launcher 當時三方 unique checkpoints：p1 10／末次1200／post-learning9；p2 8／末次1080／post-learning6。
- server 退出後保存 verifier 重新計算：p1 10／末次1200／post-learning9；p2 9／末次1200／post-learning7。退出前追加的合法 checkpoint 已逐筆核對，不要求等於較早 launcher 計數；原末次 tick 必须存在、原統計全覆蓋、零 repair／frame hash 一致。
- 正常後續 Move 抵達：p1 `(921600,716800)`、p2 `(-921600,-716800)`，均為固定點 raw。
- 三個 owned PID：server91740、runtime64228／80532，已另外 inspect 確認退出；stop 後有界 wait，非只憑 stop 回傳成功。
- 保存 verifier `first-learning-verification-report.json` 最終 success=true，九個原始證據 SHA256 前後一致；只寫自己的驗證報告，不改原 capture。

只讀重驗命令：

```text
tools/lua/lua.exe scripts/verify_moba_first_learn_run.lua moba-runtime-1791102528
```

重新對局需明確設定 `OMOBA_FIRST_LEARN_SMOKE=1` 後執行固定 Lua 入口；不能與 shop／recall／roster／combat／upgrade／reconnect smoke 混用，只驗60Hz。

## 建置與回歸

- base_content：98 passed，含實際 Lua apprentice 出生／level gate。
- omoba-core：334 passed。
- omobab `--lib`：155 passed、1 ignored。
- omoba-client-runtime `--lib`：61 passed、8 opt-in ignored；本輪 first-learning opt-in 另跑通過。
- omoba-template-ids 完整 `cargo test`：7 library＋23 generated＋8 hero ability，共38 passed。
- omfue/codegen：25 passed；shipped `--check` 通過，11 files／15 Lua inputs，content hash `3b296ff5bdbcd7d9`。
- omfue/bridge：52 passed、1 opt-in ignored。
- 固定入口 `tools/lua/lua.exe scripts/build_ue_moba.lua --build-only`：OmGameEditor 完整編譯 Succeeded，C ABI9不變；bridge staged SHA256 `0301fb7cb561f3797fe4c5568ddce123609462bed478026d826e08c82999212b`。
- 相關檔案／omb 配置 `git diff --check` 通過；既有 dead_code 與 Git 行尾提示保留，非測試失敗。

## 下一步

將同一個 apprentice server 配置接進明確 opt-in 的雙 Unreal 真實出生／Ctrl 首次學習觀測，禁止內部 runtime injection，核對原 request、正式 input、raw rank／SP／CD 與學後三方 parity。既有 E121 字形呈現裁切是另外的未封關 UI 問題，不重複已排除的猜測，不用單純 geometry／automation 通過宣稱像素正常。

之後仍有完整三路／野區、Bot 三英雄與100場、選角、視野／cue、兩台 LAN 與效能基線等大項；本輪成功只封關首次學習網路增量。
