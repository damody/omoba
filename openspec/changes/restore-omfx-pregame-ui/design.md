## Context

`omfx` 的 pregame UI 曾在 `9f88f55`、`8fe49fd`、`06baa16` 一帶完成主要流程：主畫面、選地圖、選難度、session loading 與 backend session lifecycle。後續合併與功能修復保留了部分 `pregame.rs` / `backend_session.rs` 結構，但玩家可見的 pregame UI 版面與互動體驗被覆蓋或退化。

目前 `omfx` 最新 `master` 已包含重要修正，例如 lockstep client 30s stall timeout、10s join timeout、前端資源註冊、TD sidebar/tooltip/擊破數顯示，以及 session-scoped backend 啟動流程。這次不能用整檔 checkout 舊 commit 的方式「還原」，因為那會把這些新功能一起倒回去。正確方向是把舊 UI 的有效片段移植到目前架構上，並用測試確認新舊行為相容。

## Goals / Non-Goals

**Goals:**

- 讓 `omfx` 啟動後重新呈現先前設計的 pregame UI：主畫面、開始、選地圖、選難度、loading/error、返回。
- 保留目前最新的 `backend_session` lifecycle、external backend mode、lockstep stall/join timeout 與 gameplay HUD/input 行為。
- 繼續以 scripts-owned pregame catalog 作為 UI 內容來源，避免把地圖、難度、文案與 action 硬寫回 Rust table。
- 用小範圍、可審查的 diff 移植 UI layout/render/hit-test，不整檔回退 `native.rs`、`pregame.rs` 或 `backend_session.rs`。
- 補 regression 驗證，確保 menu idle 不啟動 backend，選圖/選難度後才建立 session，進入 gameplay 後既有功能仍可用。

**Non-Goals:**

- 不重新設計 backend protocol、lockstep cadence 或 authoritative simulation。
- 不改 script ABI；pregame catalog 維持資料檔/asset catalog 層級。
- 不新增商店、帳號、英雄養成、多人 lobby 或地圖解鎖系統。
- 不把目前已修好的 TD sidebar、tooltip、擊破數 UI 改回舊版。
- 不以 `git checkout <old-commit> -- game/src/native.rs` 這種整檔回退作為實作方式。

## Decisions

### Decision: 以目前 `omfx/master` 為唯一實作基底

Implementation SHALL 先在目前 `omfx/master` 上工作，再從舊 commit 擷取需要的 UI 片段。這確保 lockstep timeout、session launcher、資源 registry 與近期 UI 修正都保留。

替代方案是直接回退到 `9f88f55` 或 `8fe49fd` 的檔案。這雖然快，但會覆蓋後續合併修正，尤其 `native.rs` 巨大且近期變動很多，風險太高。

### Decision: 把 pregame UI 視為 `pregame` state/render 層修復，不改 gameplay core

改動應集中在 pregame catalog parsing、`PregameRuntime` state、UI node 建立、layout 更新、hit region 與 action dispatch。`sim_runner`、gameplay HUD、tower/ability input routing 只應在需要阻擋 pregame input 或進入/離開 session 的邊界被觸碰。

替代方案是把 pregame 與 gameplay 初始化重新拆一輪。這會重做已存在的 session boundary，容易引入 lifecycle bug。

### Decision: 舊 UI 行為以 cherry-pick 思維移植，不做 commit-level cherry-pick

實作時先檢查舊 commit 的相關區塊：

- `9f88f55`：完整 pregame/session flow 初版。
- `8fe49fd`：pregame button layout 改善。
- `06baa16`：合併後保留的 pregame module 與資源 meta。

只移植仍與現有 API 相容的 layout、hit-test、文字/卡片呈現邏輯；若舊 code 與現有 session launcher 或 lockstep client 衝突，保留現有實作並重寫 UI adapter。

### Decision: catalog schema 只做相容修復

若目前 scripts catalog 缺少先前 UI 需要的 display/image/action 欄位，可以補 optional 欄位或 fallback，但不得改成 breaking schema。缺圖或 unknown action 必須 log 並安全降級，不得 panic 或阻止 menu 基本流程。

### Decision: 驗證以「保留現有功能」和「恢復 UI」雙軸進行

除了 build/test，也要明確比對：

- `omfx` source 沒有新增 `omobab::` dependency。
- menu idle 不啟動 backend、不建立 lockstep/sim_runner。
- `Start -> Map -> Difficulty -> Gameplay` path 成功。
- `Ctrl+Escape` 或現有 return-to-menu 行為仍能清 session。
- TD sidebar/tooltip/擊破數相關近期改動沒有被覆蓋。

## Risks / Trade-offs

- [Risk] 舊 UI code 與最新 `native.rs` 發生語意衝突 → Mitigation: 不整檔回退，先抽取小函式/小區塊，逐段 build。
- [Risk] 恢復 layout 時破壞 gameplay input routing → Mitigation: pregame state 下明確 consume mouse/key event，`InGame` 才交給 tower/ability handlers。
- [Risk] backend lifecycle 被舊版邏輯倒回 launcher-owned 或 autostart → Mitigation: specs 明確要求保留 session-scoped launcher 與 menu idle 行為。
- [Risk] scripts catalog 與 UI layout 欄位不同步 → Mitigation: 加 catalog validation test，缺 optional asset 只 fallback，缺 required action/config 則 disable entry。
- [Risk] 手動 smoke 依賴本機 backend build 狀態 → Mitigation: automated test 先覆蓋 state/action/launcher ownership，手動 smoke 作為最後確認。

## Migration Plan

1. 在 `omfx` 目前 `master` 上建立工作分支或直接工作，確認 worktree 乾淨。
2. 使用 `git show` 比對 `9f88f55`、`8fe49fd`、`06baa16` 的 pregame UI 區塊，列出可移植片段。
3. 先補/修 `pregame.rs` 的 state、catalog/action/layout model，使 current API 可承載舊 UI。
4. 在 `native.rs` 局部恢復 UI node 建立、layout 更新、hit region 與 render/update path。
5. 保留目前 `backend_session.rs` 與 lockstep client timeout；必要時只做 adapter 層呼叫調整。
6. 更新 scripts/base_content pregame catalog 或 asset path，使 UI 資料完整。
7. 執行 `cargo build --manifest-path omfx/Cargo.toml -p executor` 與相關 unit tests。
8. 手動 smoke pregame flow，確認 menu、選圖、選難度、進 gameplay、返回 menu。

Rollback 策略：若 UI restore 造成 session 問題，保留 `OMFX_LEGACY_AUTOSTART` 或 external backend mode 作為 dev fallback；提交層面應讓 UI restore 與 root submodule pointer 分開，方便 revert。

## Open Questions

- 「之前做的 pregame UI」是否以 `8fe49fd` 的按鈕排版為準，還是還包含更早本機未提交的視覺調整？
- 是否需要同時恢復特定圖片/背景素材，或先恢復 layout/flow，素材沿用目前 catalog？
- 返回 menu 的玩家操作要維持目前 `Ctrl+Escape`，還是 UI 上要補一顆明顯返回按鈕？
