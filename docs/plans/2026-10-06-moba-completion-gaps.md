# MOBA 剩餘 10 項契約缺口（2026-10-06）

> 主agent後續決定：下文保留稽核當下狀態。因既有單場report路徑會覆蓋，已新增通用60Hz批次工具保留逐場證據，不用手動重跑100次；run_moba_runtime_smoke改compiled-only，run_2player_ue商店30Hz已修正為選定tick profile，局部Lua6項通過。詳見compiled-smoke-batch-progress。第3批Grok量測正在實作，未宣稱實测或完整驗收。

本文件只核對 OpenSpec `build-unreal-rust-moba-framework` 尚未勾選的 2.2b、4.1、4.3、4.4、5.4、5.5、6.1、6.2、6.4、6.5。進度註記與 `2026-10-06-moba-functional-readiness.md` 只當索引，完成與否以目前原始碼與 spec 場景為準。本批沒有改程式、沒有改 OpenSpec checkbox、沒有跑 Cargo／UBT／PIE／生成／部署。

核對時 HEAD：root `cb0a5600812d11e7bd9ece319f8bb9f666ff4b76`，`omfue` `20987769ae5fc8e4bdabaee3ea26664450ce5306`。既有 dirty 與 untracked 保留，未回復。沒有遇到需要登記的新工具或建置錯誤，因此沒有新增 E300。E298–E299 的 owned HUD／失效輸入已由前批實作並經 Codex 審查，本文件不把它再列成缺口。

涉及目錄沒有另外的 `AGENTS.md`；維護邊界以根目錄 `AGENTS.md` 為準。唯一前端是 `omfue`。正式對局 60Hz。100 場必須是正式 headless。兩個本機程序不是兩台 LAN。只編譯的 native 斷言不算執行通過。

## 計數

| 分類 | 項數 | 任務 |
| --- | --- | --- |
| 真缺交付 | 1 | 6.5 |
| 實作已有，待最後驗收 | 8 | 2.2b、4.1、4.3、4.4、5.4、5.5、6.1、6.2 |
| 外部環境限制 | 1 | 6.4 |

4.4 的區網那一半與 6.4 共用同一外部限制，不另算成缺程式。6.5 的實測也要一場正式對局，但缺的是正式路徑上沒有留下可彙總的四個量測，不是缺玩法。

## 2.2b 通用英雄事件 — 待驗收

完成條件：英雄 C++ 事件走通用模板；既有英雄無功能回退。spec「英雄模板生成」要求保留的 typed API 轉接通用資料，保留名稱不得恢復自動派發，也不得把作者 metadata 當成渲染 fallback。

已有行為：

- 通用模板在 `omfue/codegen/src/lib.rs` 的 `generated_classes_header`／`generated_classes_source`／`class_surface_signature`。同檔測試要求一般英雄輸出不含 `HandleSaika`、`OmMakeSaika`。
- 相容層只服務內容 ID `saika_magoichi`：`omfue/codegen/src/legacy_hero_compat.rs` 的 `summary`、`class_source`、`GetSaikaMagoichiMetadata`。後者讀 `GetGeneratedHeroMetadata()` 再填舊結構。`HandleSaikaSniperModeChanged_Implementation` 等五個實作是空殼，不是第二套技能邏輯。
- 生成物 `OmContentClasses.h`／`OmContentIds.cpp`／`OmVisualRegistry.cpp` 裡的 Saika 名稱來自 Lua 內容 ID，不是手寫分支。
- 投影命名已是 `OmAbilityProjection`；C ABI 16 拒絕舊版。見 presentation spec「通用技能 C ABI 投影」。

不是缺口：不要刪 `GetSaika*` 或空的 `HandleSaika*`。那是 spec 要求留下的顯式相容呼叫。不要新增角色 C++。

驗收（指令來自原始碼，不是推測）：

```bat
cargo run --manifest-path omfue/codegen/Cargo.toml -- --content-root scripts/lua_data --out omfue/Plugins/OmRuntime/Source/OmGenerated --check
```

`--check` 的定義在 `omfue/codegen/src/main.rs`。Editor 側既有名稱是 `scripts/ue_native_visual_smoke.lua` 的 `--test Om.Generated.SaikaEventDispatch` 與 `--test Om.Generated.BlueprintSurface`。該腳本規定 scoped 確認必須另給 `--out-dir`，且 `--runs` 只能是 1 或 2。這些 native 斷言目前只編譯過，執行前不能算通過。

## 4.1 IPC 輸入與安全投影 — 待驗收

完成條件：版本握手、輸入結果、視野隔離。不是再加一種輸入。

已有行為：

- `omoba-client-runtime/src/presentation_bridge.rs` 的 `serve_renderer_retained` 只接受 `RendererReady`，逾時或非 Ready 拒絕；`validate_renderer_ready` 核對 player／team；第二個 renderer 會因 `renderer already connected` 失敗。`read_envelope` 在 magic 或 `protocol_version != PRESENTATION_PROTOCOL_VERSION` 時回 `presentation protocol mismatch`。
- 輸入結果型別是 `CriticalInputResult`。同檔測試 `covered lifecycle must be dropped, but pending input results must survive reconnect`。KCP 側 `omoba-client-runtime/src/main.rs` 把 `LockstepInbound::SecureTargetInputResult` 記成 `SERVER_ACCEPTED` 或 `SERVER_INVALID_TARGET`。
- 商店 pending／admission floor 在 `omoba-client-runtime/src/input_bridge.rs`（`resumed_allocator_never_wraps_or_reuses_the_server_floor`）與 `shop_presentation.rs`。這是已有恢復，不是未寫的持久化功能。
- 跨隊隱藏由 `omoba-core` 的 team projector／filtered baseline 負責；presentation 在握手通過前不送 snapshot。

驗收：本機雙 renderer 用既有 `scripts/run_2player_ue.lua --single-lane`。未帶 `--single-lane`／`--three-lane` 時，`OMOBA_UE_STEP_FPS` 預設是 120，不能拿來當正式 60Hz。正式 60Hz 要帶 `--single-lane` 且不要把 `OMOBA_UE_STEP_FPS` 設成 90 或 120。

注意：同一檔在 `OMOBA_UE_SHOP_SMOKE=1` 時把 `--presentation-hz` 設成 `30`，不是 60。商店 60Hz 不要用這個 smoke 旗標充數。正式 60Hz 呈現率在 `scripts/moba_role_launch.lua` 的 runtime args：`--presentation-hz 60`，入口是 `scripts/run_moba_role_ue.lua`。商店結果的事後檢查器是 `scripts/verify_ue_shop_run.lua`，且它只接受 `interactive-ue-<digits>` run ID、並要求報告 `tick_rate_hz == 60` 與 `single_lane`。

這條本機雙程序只證明 localhost IPC 與兩隊投影。它不是 6.4 的兩台 LAN。

## 4.3 Hide／Forget／ResetView、cue 去重、renderer 重連 — 待驗收

完成條件：重連不重啟對局、不重播已消費的一次性效果。spec「畫面重連」涵蓋生命週期清理、退役後同 ID 不再准入、PIE 重連不重播。

已有行為：

- `presentation_bridge.rs` 的 `reset_view_envelope` 送 `RenderLifecycleResetView`；Hide／Forget 呼叫 `CueRetention::invalidate`，ResetView 呼叫 `CueRetention::reset`。
- `omoba-client-runtime/src/cue_retention.rs` 的 `CueRetention::capture` 以 view epoch、tick floor、`retired_through` 與 `seen` 拒絕舊 ID。容量 `MAX_CUES`。
- 認得出的一次性 payload 在 `omoba-core/src/runtime/presentation_cue.rs` 的 `PresentationCue::from_effect`：`DMG1`、`ABY1`／`ABY2`／`ABY3`、`ABS2`／`ABS3`、`ARC1`、`HIT1`／`IMP1`。認不出的 effect 在 `capture` 被 `continue` 掉，不會變成另一種 cue。
- 攻擊時間狀態已接上，舊註記「CommittedAttack 只有 counter／phase」已過時。`omoba-core` 的 `AttackVisualState` 含 `elapsed_raw`／`duration_raw`／`paused`；`omfue/bridge/src/projection.rs` 的 `project_animation_state` 把它轉成 `phase_duration_secs` 與 `phase_progress`。

不是缺口：不要為了「所有 cue」再發明 audio 或新效果 ID。沒有進入 `from_effect` 的效果本來就不是這條一次性契約。

驗收：

- 啟動：`scripts/run_2player_ue.lua --single-lane`，環境 `OMOBA_UE_RECONNECT_SMOKE=1`、`OMOBA_UE_SMOKE_SECONDS` 為正數、`OMOBA_UE_STEP_FPS` 為 60 或省略。原始碼禁止這個 smoke 同時開 ability／match／shop／minimap。
- 事後：`scripts/tests/ue_renderer_reconnect_acceptance.lua RUN_DIRECTORY`。它只讀既有 run，要求六個程序角色、舊 UE PID 與新 PID 不同、server／兩個 runtime／對側 UE 不變。

兩台實體機器上的重連屬於 6.4，不能用這條本機指令代替。

## 4.4 正式 MOBA 不再持有第二份玩家模擬 — 待驗收

完成條件：單機與區網都只有一份伺服器權威模擬；每個玩家保留一份獨立 Rust filtered runtime／replica，Unreal bridge只呈現與送出輸入，不再另開舊TD gameplay world。不能把「無額外Unreal端模擬」誤寫成「玩家端完全沒有replica」。

已有行為：`omfue/bridge/src/driver.rs` 的 `validated_driver_mode` 在建立 worker 前解析一次。`presentation_only` 且位址空白會回 `legacy gameplay fallback is forbidden`；player 或 team 為 0 拒絕。`DriverMode::Presentation` 只啟動 `run_presentation_client`，`sim_thread` 是 `None`，`lockstep_step_fps` 寫 0。`LocalTd`／`NetworkTd` 仍是 TD 路徑，不是正式 MOBA。

啟動契約在 `scripts/moba_role_launch.lua`：Unreal args 含 `-om-presentation-only`，環境 `OM_RUNTIME_MODE=presentation-ipc`。單機配方 `scripts/lua_data/moba_single_player.lua` 只把 player 1 的 `bot` 設為 false，其餘沿 `moba_role_match.lua` 的十人角色。

驗收：

```bat
tools\lua\lua.exe scripts\run_moba_role_ue.lua --recipe scripts/lua_data/moba_single_player.lua --profile release
```

完成時確認 UE 行程帶 `-om-presentation-only`，且沒有第二個 gameplay world。`--prepare-only` 只準備、不開局，不能算這項通過。區網證明見 6.4，本機再加一個 UE 程序不算。

## 5.4 三路、野區、地形、導航、建築解鎖 — 待驗收

完成條件：固定種子的正式 headless 對局能結束，不卡住。spec 要求同格薄牆可繞、搜尋失敗不得用最近格假裝成功、基地在該側所有塔層都退役後才解鎖。不要求全域 navmesh。

已有行為：

- 地圖 ID 在 `scripts/lua_data/templates/moba_maps.lua`：`three_lane_training`、`three_lane_layered_training`。
- 解鎖在 `omoba-core/src/runtime/native/moba_match.rs` 的 `MobaMatch::base_unlocked` 與 `structure_unlocked`。塔死亡會改 `lane_tower_layers`，第一個仍存活的層寫回 `lane_towers`。
- 有界路徑在既有導航規劃器；Bot 旅行／追擊／巡邏已改走完整路徑，失敗就 Hold 或換下一個候選。這不是還缺的功能。

驗收工具是 `scripts/run_moba_headless.lua`，它把參數轉給 `omb/src/bin/moba_headless.rs`。該二進位只認得 `--scripts-dir`、`--report`、`--role-plan`、`--plan-only`、`--seed`、`--map`、`--defender`、`--profile`。沒有 `--matches`。一次一場，上限是 `ticks_per_game_second * 600`，沒結束就回 `match timeout`。`--role-plan` 不能和 `--map`／`--defender` 並用。`--defender withdraw` 是原始碼註解寫明的非競賽 fixture，正式批次禁止使用。`--plan-only true` 不模擬。

正式批次就是換種子重跑，不要新寫批次器：

```bat
tools\lua\lua.exe scripts\run_moba_headless.lua --map three_lane_training --profile 60 --seed 1
tools\lua\lua.exe scripts\run_moba_headless.lua --map three_lane_layered_training --profile 60 --seed 1
```

種子可換，但每次都要 `--profile 60`，且不要加 `--defender`。報告裡 `success=true`、`profile_hz=60`、對應 `map_id`、`replay_verified_ticks` 等於結束 tick，才算這一場沒卡住。`scope` 欄位自己寫明這不是 Unreal／filtered／LAN。UE 三路畫面不是這項的完成條件；`run_2player_ue.lua --three-lane` 在原始碼裡只允許 60Hz 基本移動與地圖觀察。

## 5.5 五位置與三種原型的 100 場 — 待驗收

完成條件：100 場正式 60Hz headless，無越權輸入、死局或非法目標。不是再加 Bot 政策。

已有行為：

- 五位置與路線在 `scripts/lua_data/moba_role_match.lua`：`top`／`mid`／`carry`／`support`／`jungle`，`map_id=three_lane_training`，每隊五人，預設全是 Bot。
- 三種原型在 `scripts/lua_data/moba_archetype_match.lua`：`training_vanguard`、`training_luminary`、`training_ranger`（support 用 `training_support`，jungle 再用 vanguard）。它載入 role match 後只換英雄與技能政策。
- Bot 只經 `role_bot_inputs` 產生正式 `PlayerInput`。追擊、撤退、兵線、護送、野區旅行的完整路徑閘門已在 `omoba-core` 的 role bot 規劃裡。

驗收仍是上面的 headless 二進位，沒有 100 場開關。一場的指令：

```bat
tools\lua\lua.exe scripts\run_moba_headless.lua --role-plan-lua scripts/lua_data/moba_archetype_match.lua --profile 60 --seed 1
```

100 場就是 100 個不同 `--seed`，每次 `--profile 60`。不要 `--plan-only`，不要 `--map`，不要 `--defender`。Lua 會把配方匯出成暫存 JSON 再傳 `--role-plan`。

可觀察的失敗只有：`match timeout`（600 遊戲秒，當作死局）、replay hash 分歧、`end_events != 1`、`combat_facts == 0`。有 role plan 時 `skill_coverage_checked` 為 false，四槽全 0 不會失敗。沒有單獨的「越權輸入／非法目標」計數器。那些拒絕已經在 Bot 正式輸入與權威 `SERVER_INVALID_TARGET` 路徑。不要為了 100 場新做計數器或批次程式；100 份 `success=true` 且 `profile_hz=60`、`bot_mode=committed_role_plan` 的報告就是這項證據。局部 native 測試只編譯，不能拿來充 100 場。

## 6.1 動畫、cue、視野、地圖與美術 fallback — 待驗收

完成條件：缺非必要美術時 fallback 可用，且畫面契約在更新前拒絕壞 frame。玩法與 cue 來源已接上，缺的是 native 斷言真的跑過，以及一場 PIE 畫面。

已有符號：

- 動畫：`omfue/Plugins/OmRuntime/Source/OmRuntime` 的 `AOmHeroActor::OnAttackPhase_Implementation`、`AOmUnitActor::OnAttackPhase`；bridge `project_animation_state`。
- 霧：`OmWorldBridgeActor::RebuildFogMask` 與 `FogMaskTexture`。幾何鍵與 route／polygon 範圍是既有 frame gate，不是新系統。
- 地形：C ABI terrain rects 進共用 ISMC；小地圖地形是另一條唯讀投影。
- fallback：Lua `fallback policy` 已生成進 native selector。未知內容不回退成 Saika 或 practice dummy，這是已勾過的行為，不要再改。

驗收名稱都在 `scripts/ue_native_visual_smoke.lua` 的 `report.tests`，至少：

- `Om.Generated.FogGeometryKey`
- `Om.Generated.FrameGeometryRanges`
- `Om.Generated.FrameTextContract`
- `Om.Generated.NativeHeroPresentation`
- `Om.Generated.AnimationStateSmoke`
- `Om.Generated.AbilityCastCue`
- `Om.Generated.ProjectileCueHistory`
- `Om.Generated.CollisionTerrainPresentation`
- `Om.Runtime.MinimapFogGrid`

指令形狀：`tools\lua\lua.exe scripts\ue_native_visual_smoke.lua --test <上面名稱> --runs 1 --out-dir <新目錄>`。不帶 `--test` 會跑整份清單並寫進預設 `omfue/Saved/McpAutomation/NativeVisual`。執行通過之前，這些項目維持「只編譯」。

## 6.2 選角到結算 UI — 待驗收

完成條件：PIE 從選角玩到勝負。各塊 UI 已經有原生入口。前批補上的 `InvalidateOwnedHud`／`NotifyOwnedGameplayUnavailable` 不要重做。

已有行為：

- 選角：`OmHeroSelectionWidget` 與 `scripts/moba_shared_selection.lua`。`run_moba_role_ue.lua --interactive-selection` 在開局前走選角；`--prepare-only` 與 `--interactive-selection` 互斥。
- HUD／商店／小地圖／計分板／勝負：smoke 清單裡的 `Om.Runtime.NativeShopInput`、`Om.Runtime.NativeMinimap`、`Om.Runtime.NativeScoreboard`、`Om.Runtime.NativeMatchResult`、`Om.Runtime.MatchSmokeObjective`。
- 單機十人配方就是 `moba_single_player.lua`（1 名真人 + 9 Bot）。

一次走完的既有入口：

```bat
tools\lua\lua.exe scripts\run_moba_role_ue.lua --interactive-selection --recipe scripts/lua_data/moba_single_player.lua --profile release
```

這會建置並開 Editor／遊戲視窗，本批沒有跑。選角取消必須在開局前停下。結算畫面若要對舊的雙 UE 證據複查，用 `scripts/tests/ue_match_lifecycle_acceptance.lua RUN_DIRECTORY`；它只接受已有的 single-lane 雙隊報告，要求終局後再有 120 tick 的三方 hash。那個檢查器不是選角流程本身。

`scripts/ue_pie_smoke.lua` 的檔頭寫明它是 Editor native hero smoke，gameplay 對局驗收另算。不要把 PIE smoke 通過說成 6.2 完成。

## 6.4 兩台 LAN、視野、重連、版本錯配 — 外部限制

完成條件：兩台實體機器同一場、跨隊看不到對方隱藏狀態、renderer 重連、版本錯配被拒絕，並留下紀錄。

程式已經有，不能在這一台機器上假造：

- 遠端啟動契約在 `scripts/moba_network_launch.lua` 的 `validate`，說明印在 `scripts/run_moba_role_ue.lua` 開頭。主機：`--server-bind <unicast IPv4>`，可重複 `--local-player`。遠端：`--connect <主機 IPv4> --recipe <主機最終 JSON> --local-player <ID>`。遠端不能改 `--server-bind`、不能 `--interactive-selection`、不能再選英雄。IPC 維持 loopback。
- 錯配：presentation `protocol_version` 不符即停；Unreal catalog hash／ABI／surface 不符時停本地 bridge、不 ACK、不重啟後端。這是已有 gate。
- 視野與重連符號同 4.1／4.3。

禁止：`run_2player_ue.lua`、同一台機器開兩個 `run_moba_role_ue.lua`、或任何兩個本機程序，都不得寫成兩台 LAN 通過。沒有第二台機器與獨立網卡位址之前，6.4 維持未完成。這不是下一個實作包。

## 6.5 建置與效能基線 — 真缺交付

完成條件（設計 `docs/superpowers/specs/2026-09-25-unreal-rust-moba-framework-design.md`）：十名英雄、雙方兵線與野怪的正式對局上，量到 server tick、client step、IPC 頻寬、UE frame time；先有基線數字，再固定門檻。tasks.md 的 6.5 下面沒有任何實作註記。目前沒有這份報告，也沒有門檻常數。不要先寫死毫秒數。

現有量測為什麼不夠：

| 量 | 符號 | 正式 60Hz 路徑上的實際行為 |
| --- | --- | --- |
| server tick | `omoba-core/src/comp/tick_profile.rs` 的 `TickProfile::finish_tick_and_maybe_log`，`WINDOW=60` | 只由 `omb/src/state/core.rs` 每 tick 呼叫，而且 `emit_log` 是 `log::debug!`。role launch 把 `RUST_LOG` 設成 `info`，這行不會出現。註解仍寫 30 TPS，不能當門檻。`moba-headless` 的 `SimulationDriver` 不呼叫它。 |
| client step | `omoba-client-runtime/src/main.rs` 的 `STEP_US_` | 只在 `evidence` 為 `Some` 時寫入。`evidence.rs` 的 `EvidenceRecorder::create` 在 `test_mode == false` 時直接 `Ok(None)`。`moba_role_launch.lua` 的正式 runtime args 沒有 `--test-mode`。 |
| IPC 頻寬 | `omoba-core/src/kcp/client.rs` 的 `LockstepInbound` 已帶 `wire_bytes` | `main.rs` 用 `SecureTargetInputResult { msg, .. }` 把位元組丟掉。client-runtime 內沒有 `wire_bytes` 累加。localhost 呈現的 `presentation_bridge.rs::write_envelope` 知道 `bytes.len()`，但只寫出，不計數。 |
| UE frame time | 無 | `omfue` 的 `FPlatformTime` 只用在選角逾時與測試截止。role launch 的 `-ExecCmds=t.MaxFPS 60` 只是上限。`omfx` 的 frame profiler 不在維護範圍，禁止移植。 |

`scripts/build_ue_moba.lua` 只有 `--build-only`、`--full`、`--verify-staged-only`、`--ue-root`。它是建置入口，不是效能報告。

### 最小實作包

只補四條正式路徑的視窗摘要，不改模擬、不改 Bot、不改 ABI、不新增根目錄 bat、不引入 omfx。門檻在第一份實測報告出來之後才寫，本包不要先放數字。

1. `omb/src/state/core.rs`：沿用現有 `TickProfile` 總和，在正式 MOBA（已有 `MobaMatch`）以 info 留下與 debug 相同的 window 平均。不要新做 profiler，不要把註解裡的 30 TPS 寫進門檻。
2. `omoba-client-runtime/src/main.rs`：正式非 test-mode 的 step 也記錄與 `STEP_US_` 相同的耗時；處理 `LockstepInbound` 時保留 `wire_bytes` 並做 window 加總。test-mode 證據路徑維持原樣。
3. `omoba-client-runtime/src/presentation_bridge.rs` 的 `write_envelope`：累計已寫出的 frame 位元組（長度前綴 + payload），與 KCP `wire_bytes` 分開記，避免兩段 IPC 混成一個數。
4. `omfue/Plugins/OmRuntime/Source/OmGenerated/Private/OmWorldBridgeActor.cpp`（以及它的 header，若計數要跨 tick 保存）：在既有呈現 tick 用現成的幀間隔記一筆 `OM_FRAME_MS`，不新增角色分支、不改 HUD 玩法。此檔目前已有未提交修改，下一包必須在那份 diff 上加，不能還原。

完成條件：一場 `run_moba_role_ue.lua --recipe scripts/lua_data/moba_single_player.lua --profile release` 的 info 日誌能讀出這四個 window 值，再把數字寫進報告並才固定門檻。headless 100 場沒有 UE frame，也不能單獨關閉 6.5。

## 下一實作包與最終驗收順序

下一個實作包只有 6.5 的四筆量測。除此之外不要新增防禦性檢查、Bot 政策、navmesh、cue 種類或 UI。

量測落地之後，一次性驗收按這個順序。前一項沒有正式 60Hz 證據就不要用下一項掩蓋：

1. `om_codegen --check`（2.2b 生成契約）。
2. headless `--map three_lane_training` 與 `--map three_lane_layered_training`，`--profile 60`，多個種子，無 `--defender`（5.4）。
3. headless `--role-plan-lua scripts/lua_data/moba_archetype_match.lua --profile 60`，100 個種子（5.5）。
4. `run_moba_role_ue.lua --recipe scripts/lua_data/moba_single_player.lua --profile release`，確認 presentation-only、單一份權威模擬（4.4 的單機半邊），並收集 6.5 四個量。
5. 同一配方改 `--interactive-selection`，從選角玩到結算（6.2）。native smoke 用獨立 `--out-dir` 跑 6.1 清單。
6. 本機 `run_2player_ue.lua --single-lane` 的 60Hz 握手、輸入結果、視野，以及單獨一輪 `OMOBA_UE_RECONNECT_SMOKE=1`（4.1、4.3）。商店不要用會把 presentation 降到 30Hz 的 `OMOBA_UE_SHOP_SMOKE`。
7. 兩台機器才跑 6.4 的 `--server-bind`／`--connect`。沒有第二台就停，並在報告寫明外部限制。

局部 native 測試、`--plan-only`、`--prepare-only`、`ue_pie_smoke.lua`、以及只編譯成功的 automation，都不能把對應任務勾完。
