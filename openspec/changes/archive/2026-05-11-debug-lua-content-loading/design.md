## Context

`scripts/lua_data` 是 shipped content 的 canonical source，但目前 debug `run.bat` 仍透過 `omoba-template-ids/build.rs` 把 Lua builders 轉成 generated Rust data。這讓塔數值、英雄數值、波次或 story Lua 只要有變更，就可能讓 `omoba-template-ids`、`omb`、`omfx` 或 `base_content.dll` 被 freshness helper 視為 stale，內容調參需要等待 Rust rebuild。

現有 runtime path 有兩個重要限制：`omb` server 透過 `CampaignData::load_generated` 讀 generated story，`omfx` 的 `sim_runner` 也呼叫 `create_world_for_scene`，因此兩邊必須使用同一份 content snapshot；`run_stress.bat` 則是 release 壓測入口，必須維持 build-time generated Rust data，不可預設在 runtime hot path 載入 Lua。Runtime Lua loader 仍需要能被 release build 以明確 feature/env opt-in 編進去，方便用 optimized binary 驗證 content。

## Goals / Non-Goals

**Goals:**

- 讓 debug launchers（`run.bat`、`run_smoke.bat`、`run_smoke_long.bat`）預設編譯/啟用 runtime Lua content loading，直接讀取 `scripts/lua_data` 最新值。
- 讓 release build 可透過明確 feature/env opt-in 使用同一套 runtime Lua loader；未 opt-in 的 default release/stress path 仍使用 generated Rust data。
- 讓 `omb` host 與 `omfx` `sim_runner` 在 runtime Lua content mode 使用同一套 Lua-loaded content snapshot 與相同 validation，避免 lockstep/snapshot 分歧。
- 讓 debug freshness 在 Lua content-only 變更時不重編 Rust artifacts，但 Rust source、manifest、ABI、protocol 或 script source 變更仍照常 rebuild。
- 保留 release/stress/default production path 的 generated Rust data；`run_stress.bat` 不啟用 runtime Lua loader。
- 保留現有 Lua builder contract、deterministic ordering、template/story lookup semantics 與 fail-fast validation。

**Non-Goals:**

- 不變更 gameplay schema、network protocol、script ABI、FFI layout 或 runtime-visible strings。
- 不讓 default release/stress gameplay 隱性讀 Lua，也不把 Lua loader 放進未 opt-in 的 release hot path。
- 不重新設計 Lua builder 格式，不導入新的 content authoring schema。
- 不處理 live reload；debug launcher 啟動時讀一次 content snapshot 即可。

## Decisions

### Decision: 抽出共用 Lua content loader，而不是複製 `build.rs` 邏輯

將 `omoba-template-ids/build.rs` 內的 `LuaContentLoader`、story loading、template manifest conversion 與 validation 抽成可重用模組或 crate API，由 build-time codegen 與 runtime Lua loader 共用。Build-time 仍輸出現有 generated Rust API；runtime Lua loader 則把同一套 Lua-loaded model 轉成 runtime 可用的 content snapshot。

Alternatives considered：在 `omb` 另寫一套 Lua parser 較快，但容易讓 include/path validation、ordering、story validation 與 codegen 分裂；讓 runtime 直接執行 `build.rs` 再讀 OUT_DIR 會回到 rebuild 問題，且不適合 `omfx` 共享。

### Decision: Runtime Lua mode 由 feature/env 明確啟用，debug launchers 預設使用

新增 runtime Lua content feature 與 env switch，例如 Cargo feature `runtime-lua-content` 搭配 `OMB_LUA_CONTENT=1`，並讓 debug launchers 預設用該 feature build 且設定 content root（例如 `OMB_LUA_CONTENT_ROOT=D:/omoba/scripts/lua_data`）。Release build 可用相同 feature/env 明確 opt-in；沒有 feature 或 env 時，`omb` 與 `omfx` 繼續使用 generated Rust data。

Alternatives considered：只用 `cfg(debug_assertions)` 最簡單，但 release build 無法用 optimized binary 測 Lua content；自動偵測 debug profile 會讓 IDE、測試或手動執行有隱性行為；直接永遠讀 Lua 會違反 release/stress contract。明確 feature/env 較容易測試，也能確保 `run_stress.bat` 不啟用。

### Decision: Runtime snapshot 在 process 內載入一次，host 與 sim_runner 使用相同 source contract

`omb` 在 runtime Lua env 啟用時，從 Lua-loaded content 建立與 generated path 等價的 template/story snapshot，並提供 `CampaignData` 初始化 path 使用。`omfx` `sim_runner` 在相同 env 下用同一個 content root 與 story id 初始化 replica world；未啟用時維持 `CampaignData::load_generated`。

Alternatives considered：由 backend 把 content snapshot 傳給 frontend 可保證完全同一份 bytes，但會牽涉 protocol 或 IPC，超出本 change；讓 `sim_runner` 繼續讀 generated data 則會在 runtime Lua content 修改後與 host 分歧。

### Decision: Debug freshness 移除 Lua content 作為 Rust rebuild input，release/stress 保留

`scripts/dev_run_freshness.ps1` 對 debug profile 的 artifact inputs 不再把 `scripts/lua_data` content-only 變更視為 Rust artifact stale；release profile 仍保留 Lua content 為 `omoba-template-ids` codegen input，供 `run_stress.bat` 與 default release generated-data path 使用。Debug launchers 在啟用 Lua loader 時，若 Lua content missing 或 invalid，應在 launch/initialization fail-fast，而不是用 rebuild 掩蓋。若未來有 release Lua opt-in launcher，應沿用同一個 explicit feature/env contract，而不是改變 `run_stress.bat`。

Alternatives considered：完全忽略所有 template-id inputs 會漏掉 `omoba-template-ids/src` 或 public API 變更；只忽略 Lua data 才符合「改 content 不重編，改 Rust API 仍重編」。

## Risks / Trade-offs

- Runtime Lua mode 增加 `mlua` 與 loader dependency → 用 Cargo feature 與 env gate 限制；debug launchers 預設啟用，release build 必須明確 opt-in，default release/stress 不編或不啟用該 path。
- Host 與 `sim_runner` 分別載入 Lua，若啟動過程中檔案被修改可能取得不同 snapshot → runtime Lua content mode 文件標示啟動期間不要修改；未來若需要可再做 backend-to-frontend snapshot handoff。
- 抽出 `build.rs` 邏輯可能觸及 generated public API → 以最小搬移方式保留現有 `omoba-template-ids` generated API 與 tests。
- Debug freshness 不因 Lua 變更 rebuild，可能讓依賴 generated const 的 script DLL 仍使用舊 const → runtime Lua mode 必須讓 runtime tower/template metadata 走 Lua-loaded snapshot；若某些 script behavior 仍編譯期引用 generated const，需列入實作檢查與測試。
- Invalid Lua 從 build-time error 移到 runtime initialization error → launcher 應 non-zero/fail-fast，log 需包含 rejected path、missing story 或 validation error。

## Migration Plan

- 先抽出共用 loader 並讓 `omoba-template-ids` build-time output 與現有 tests 維持不變。
- 加入 feature/env-gated runtime loader，讓 `CampaignData` 與 template lookups 可依 content source 選擇 generated 或 Lua snapshot；debug launchers 預設啟用，release build 可明確 opt-in。
- 更新 `omb` initialization 與 `omfx` `sim_runner`，使兩者在 runtime Lua env 啟用時使用相同 content root/story id。
- 更新 `run.bat`、`run_smoke.bat`、`run_smoke_long.bat` 設定 runtime Lua env 並使用對應 feature build；確認 `run_stress.bat` 不設定且 release freshness 不變。
- 調整 `scripts/dev_run_freshness.ps1`，只在 debug profile 排除 Lua content-only rebuild input。
- 補上 tests/smoke checks：debug Lua override 生效、release opt-in 可編譯/啟用、release generated path 預設不讀 Lua、freshness skip rebuild、missing/invalid Lua fail-fast。

Rollback strategy：移除 runtime Lua env 設定或不啟用 feature 即可回到 generated Rust data；default release/stress path 不變，若 loader 有問題可先停用 launcher env 而不影響 release 壓測。

## Open Questions

- Runtime Lua loader 的 Cargo feature 名稱需在實作時依 crate dependency graph 決定，例如 `runtime-lua-content`。
- 若 script DLL 仍有無法改成 runtime snapshot 的 generated const 使用點，是否接受 debug Lua mode 先限制在 story/map/template metadata，或需要一起替換 script-side數值來源。
