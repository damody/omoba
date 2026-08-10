## Context

TD campaign 初始化目前固定由 `StateInitializer::create_campaign_heroes` 依 mission hero 清單建立 Hero entities；`run.bat` 啟動的 frontend local simulation 與 launcher-owned backend 都走相同初始化。`run_10000.bat` 只先設定金錢再呼叫 `run.bat`。需求是只讓這兩個入口成為無英雄 session，不改 mission source of truth，也不影響 `run_2player.bat`、`run_ue.bat` 或未使用旗標的 runtime caller。

兩個 batch 檔必須維持 CRLF 與 UTF-8 without BOM。Hero entity 目前同時承載玩家 `Gold`、`PlayerOwner` 與塔操作權限，因此移除英雄會使手動建塔、升塔、賣塔不可用；此限制已由使用者接受。

## Goals / Non-Goals

**Goals:**

- 以 `OMB_NO_HEROES=1` 明確控制 campaign hero creation。
- 讓 `run.bat` 與 `run_10000.bat` 的 frontend/backend process 一致繼承旗標。
- 在跳過時不建立任何 Hero entity，也不排入 hero spawn event。
- 未設定或值不為 `1` 時維持既有初始化。
- 以不修改 process-global environment 的方式測試 policy 與 entity 結果。

**Non-Goals:**

- 將玩家經濟或塔操作權限從 Hero entity 拆出。
- 修改 mission Lua、hero template、protocol、ABI 或前端選角 UI。
- 改變其他 root launcher。

## Decisions

### 使用精確值環境旗標

runtime 僅在 `std::env::var("OMB_NO_HEROES").ok().as_deref() == Some("1")` 時停用英雄。這與既有 `OMB_NO_CREEPS=1` 慣例一致，且避免空字串、`0` 或拼錯值意外改變遊戲。

替代方案是清空 `TD_1/mission.lua`，但會污染共用內容並影響所有 launcher；生成後再刪除則可能留下 script event 或其他初始化副作用，因此不採用。

### 在唯一 hero creation boundary 提前返回

環境解析留在薄 wrapper；實際建立邏輯接受可測試的 resolved policy。停用時在讀取 hero source、建立 component 與 enqueue `ScriptEvent::Spawn` 前返回，並輸出一次 `OMB_NO_HEROES=1` 診斷。

這個邊界同時被 backend 與 local simulation 使用，可避免只修其中一側造成 lockstep world 不一致。

### 兩個 launcher 都明確設定旗標

`run.bat` 設定旗標後啟動 session；`run_10000.bat` 在呼叫 `run.bat` 前也設定同一旗標。後者雖然技術上冗餘，但讓 wrapper 的 runtime contract 可直接從檔案判讀，且未來若呼叫鏈調整仍保持意圖。

### 測試不改 process-global environment

單元測試直接呼叫接受 resolved policy 的 helper，分別驗證停用與預設路徑。這避免 Rust tests 平行執行時 `set_var`／`remove_var` 互相污染。batch 驗證另以 byte-level 檢查 CRLF 與 BOM，並以受控 `cmd.exe` parse/smoke 檢查第一階段輸出。

## Risks / Trade-offs

- [Risk] Hero-free session 無玩家 Gold owner，塔操作會被既有 handler 拒絕。→ 此為明確接受限制；log 與 spec 記錄，不在本 change 建立替代 economy。
- [Risk] frontend 與 backend 旗標不同會造成 lockstep 初始 world 分歧。→ 兩者由同一 launcher process tree 繼承環境，且 initialization policy 位於共用 `omoba-core`。
- [Risk] `.bat` 被工具轉成 LF 後再次出現 `'M' is not recognized`。→ 寫入時強制 CRLF、UTF-8 without BOM，驗證所有 LF 都由 CRLF 組成。
- [Risk] 直接 smoke 可能啟動互動式 frontend。→ smoke 僅觀察至 freshness/build boundary，必要時使用 bounded timeout 並確認沒有殘留 launcher-owned process。

## Migration Plan

1. 加入共用初始化 policy 與測試。
2. 在兩個 launcher 設定 `OMB_NO_HEROES=1` 並正規化行尾。
3. 跑 `omoba-core`、`omobab` 測試及 batch byte/parse 驗證。
4. 若需回復，移除兩個 launcher 的環境設定即可恢復預設英雄；runtime flag 支援可保留供其他明確 opt-in caller 使用。

## Open Questions

無。Hero-free session 的塔操作限制已接受，scope 與旗標值已確定。
