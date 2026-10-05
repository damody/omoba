# 共用多人選角房間與一次性開局交接

## 本輪計畫與決定

1. 沿用 build-unreal-rust-moba-framework 的 6.2／6.4；本機選角到配方交接已有接線，這輪實作多人 host 所需的共用核心，不重跑完整驗收。
2. 新 SelectionRoom 保存唯一 HeroSelectionSession；host 對每個已准入真人呼叫 bind，回傳固定身分的 SelectionService。Arc clone 只共用同一 session；Bot／未知身分拒絕。沒有 gameplay World、runtime Lua、角色特例或第二份選角規則。
3. SelectionService::new 保留原簽名，委派房間 bind，既有 moba-config --selection-session／原生 UI 的單真人 JSON-line 路徑不分岔。host／連線層須先完成准入，不把 recipe 中有該玩家等同連線認證。
4. protocol1／完整 catalog hash／expected_revision 驗證、修改與回覆取得同一 lock；兩個玩家可並發呼叫，但同 revision 不能同時成功。正常規則錯誤維持 SelectionReply.error；poison authority 另回 Result Err，serve 轉 io error 且不輸出不可信快照。
5. 成功 finalize 配方保存在 room，host take_finalized_plan 原子且只能領取一次。最後按鈕玩家即使離線，host 仍能領取同一配方；read／失敗／rebind／EOF 不建立第二份交接，也不自動開局。此為記憶體內交接，不宣稱跨 host process crash 的持久化。

## 通用接線方式

```rust
let room = SelectionRoom::new(HeroSelectionSession::new(plan, seed, 60)?);
let first = room.bind(first_admitted_player_id)?;
let second = room.bind(second_admitted_player_id)?;
// Host routes each admitted connection to its own fixed service.
// Services can run on separate threads; no lock is held during pipe I/O.
// After a successful finalize:
let launch_plan = room.take_finalized_plan()?;
```

取到 Some 才交給既有正式開局 adapter，None 不猜預設配方；這輪沒有新增網路 listener、玩家自報身分 envelope 或各自獨立大廳程序。

## 局部確認

- 新雙真人完整核心案例：共用選擇、stale 回覆帶目前狀態與原 request token、未全員 ready 拒絕、各自選擇／鎖定、成功 finalize、另一方看到 freeze、斷線後 host 一次領取、重連／EOF 不補領。
- 新真正兩 thread 案例：Barrier 同時提交 revision0 lock，恰一方成功；另一方用 revision1 重試，ready=true；再並發 finalize，恰一份成功結果與一次 host 交接。
- 新 poison 案例：故障注入後 bind／handle／serve／take 全部失敗封閉，沒有輸出不可信初始 JSON。
- 首輪新3項與舊單真人服務4項通過。加入 host 一次性交接後，最終 `cargo test --manifest-path omoba-core/Cargo.toml --features compiled-content-only hero_selection_` 共11項全部成功（新3／service舊4／kernel舊4），14.71秒編譯、測試0.01秒；沒有編譯或斷言失敗。
- 最終 `cargo check --manifest-path omb/Cargo.toml -p omobab --bin moba-config --features compiled-content-only` 成功，5.92秒；唯一正式 CLI 使用點維持原 new／serve 介面，沒有遺漏新的 fallible handle 接線。沒有重建或啟動 Unreal，因本轮沒有 C++／native資料格式變更。
- OpenSpec strict validation 與主 repo whitespace check 成功；既有 td_rounds dead_code／protoc 缺少時的版控 schema fallback 警告保留，不擴大安裝／修改工具鏈。

## 明確剩餘範圍

- 尚須將 LAN 已准入連線接到共用 room、同步其他玩家的狀態，以及 Unreal 大廳的遠端 adapter；目前各本機 UI 的選角子程序模式不因此變成共享 LAN 大廳。
- 尚未做雙 Unreal／選角到結算／LAN／效能完整驗收；不 stage／部署／commit／push、不維護 omfx。
- protocol1／Lua catalog hash／C ABI14／IPC4／selective wire5 不變。錯誤與防錯見 E243；不因子項完成勾完整6.2／6.4，整體21/31保持。
