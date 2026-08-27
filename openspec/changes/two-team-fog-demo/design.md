## Context

`server-authoritative-selective-lockstep` 已完成 secure V2 team stream、filtered bootstrap、team-scoped replica ID、shared `SelectiveReplicaRuntime` 與每隊 observer validation。現有 `run_2player.bat` 已有一個 server 加兩個 frontend 的骨架，但引用不存在的 `run_2player_client.bat`，無法成為可重複驗收入口。現有 campaign scene 主要建立單一英雄，舊 `CircularVision` 註解也明確屬於 renderer hint，不能直接作為 secure visibility authority。

本 change 建立 opt-in 的 `FOG_2TEAM_DEMO`，同時跨 Lua content、omb scene/bootstrap、selective projection、omfx presentation 與 Windows launcher。Rust toolchain 固定 1.95.0；根目錄不新增 `.bat`，修改的 `.bat` 必須保持 CRLF。

## Goals / Non-Goals

**Goals:**

- 用 deterministic 10×10 方格建立精確 100 個一般單位，另加兩個玩家英雄。
- 一個 omb 與兩個獨立 omfx process 分別模擬 Team 1／Team 2。
- 每隊以英雄為中心、半徑 700 的 team-shared 圓形 visibility 決定 server disclosure。
- 在兩個視窗清楚呈現 fog、vision circle、正常單位、LastKnown ghost 與 filtered count。
- 以 16 個 deterministic patrol units 反覆觸發 reveal/hide。
- 保留 outbound-first、observer-second 的每隊 wire-byte validation。
- 讓 `run_2player.bat` 可重複啟動、左右排列、記錄 per-player log 並精確清理自己的 process。

**Non-Goals:**

- 不加入遮蔽物、視錐、草叢、高度差或新的 fog gameplay rule。
- 不將 102-unit demo 當成效能 benchmark。
- 不修改既有 TD/MVP story、matchmaking、登入 UI 或 active-match team switching。
- 不允許 legacy global snapshot、raw ECS identity、global seed 或跨隊資料進入 player process。

## Decisions

### Decision 1：新增獨立 Lua story，不改既有關卡

建立 `scripts/lua_data/FOG_2TEAM_DEMO/`，使用既有可渲染 template。方格位置由 row/column、固定原點與 220-unit spacing 推導；team assignment 由 stable index 規則產生精確 33／33／34。相較修改 `MVP_1`，獨立 story 可避免回歸污染並讓 fixture 可單獨驗證。

### Decision 2：以 demo scene descriptor 跨 Lua/import boundary

Lua package 宣告 grid geometry、hero spawn、team/owner、vision radius、remember policy 與 patrol descriptors。Import layer先驗證數量、唯一 stable spawn key、合法 team 與 finite coordinate，再建立 ECS。Production selective types 留在 runtime boundary；script ABI 只在確有跨 DLL 資料需求時加入 `abi_stable` 友善 POD，不引入 `specs`。

### Decision 3：102 個單位採固定 stable spawn order

建立順序固定為 100 個 grid units（row-major）後 2 個 heroes。巡邏集合由固定 stable index 清單選出 16 個，每個 path 只有兩個 endpoint、固定速度與 deterministic direction reversal。不得使用 runtime RNG、wall clock 或 entity allocation completion order決定配置。

### Decision 4：新 authority 只接既有 team projection

英雄建立 `VisionSource(team_id, radius=700)` 與 player ownership；一般單位使用 `TeamVision` scope 與 `LastKnown` remember policy。Visibility resolution、3-tick commitment、fresh baseline、replica identity、frame ordering 與 correction 沿用 secure V2 contract。舊 viewport/`CircularVision` renderer hint 不可回到 gameplay authority。

### Decision 5：Neutral 不是 Public

34 個 Neutral 單位仍使用 `TeamVision`，只有進入該 team 的圓形視野才揭露。這使兩個 client 能對同一 neutral unit 呈現 visible、hidden 或雙方 visible 三種狀態，而不建立全域 disclosure 例外。

### Decision 6：fog 與 LastKnown 都是 filtered presentation

omfx 從 `SelectiveReplicaRuntime` 的 filtered snapshot 取得目前 disclosed entities，從獨立 remembered render cache 取得 ghosts。Fog overlay 與 vision circle 可由該玩家已知的自有英雄位置和固定 demo radius繪製；不得要求 full-world mask。HUD 的 100 grid units／2 heroes 是固定 demo metadata，`Currently disclosed` 與 `Remembered ghosts` 則只計算本地 filtered collections。

### Decision 7：launcher 直接管理兩個 client process

修改既有 `run_2player.bat`，移除不存在 helper 的依賴。launcher 使用 PowerShell `Start-Process -PassThru` 或等價的既有安全 helper，為每個 client 設定獨立 player ID、name、lockstep name、team title/log suffix與 window placement。正常清理只使用已取得的 PID，不以 image-wide `taskkill` 作一般 lifecycle。

### Decision 8：完整測試集中最後

實作 wave 僅執行 compile 與 focused smoke。全部 content、backend、frontend、launcher 完成後，才集中執行 map cardinality、determinism、visibility boundary、protocol isolation、client presentation、process topology 與人工雙視窗驗收。

### Decision 9：證據與調整規則

每個完成的 L3 task 在 `evidence/index.jsonl` 建立唯一 record。A-level 可拆 task 或修正命令；B-level 若發現核准範圍內的設計/spec 錯誤，須同步更新 design/spec/tasks 並使相關 evidence stale；C-level 若要改變 100+2 數量、700 半徑、secure boundary、blocking gate、required evidence、根目錄腳本政策或 external/destructive action，仍須使用者核准。任何調整不得降低 gate。

## Data Flow

```text
FOG_2TEAM_DEMO Lua
  -> validated demo descriptors
  -> omb authoritative Specs world (100 grid + 2 heroes)
  -> Wave A deterministic movement/outcome
  -> team visibility resolution + transition commitment
  -> Team 1 / Team 2 projector
  -> encoded team frame
       -> session send queue -> omfx process -> SelectiveReplicaRuntime -> snapshot/cache -> render
       -> non-blocking Arc tap -> same-team observer thread -> hash/coverage diagnostics
```

## Failure Handling

- Content 數量、team distribution、spawn key、coordinate 或 patrol descriptor 不合法時，demo 啟動 fail fast，不建立部分場景。
- V2 negotiation、authenticated team binding 或 duplicate player 失敗時 fail closed，不降級。
- Client 或 server 提前退出時，launcher 回傳非零狀態並只清理本次持有 PID。
- Observer queue overflow 不阻塞 outbound；記 coverage gap、discard stale observer 並 filtered rebootstrap。
- Missing render asset 使用既有安全 fallback，但不得改變 gameplay visibility 或 entity count。

## Observability and Security Gates

- `G-DEMO-CARDINALITY`：100 grid + 2 heroes，team distribution 33／33／34。
- `G-DEMO-DETERMINISM`：位置、spawn order、patrol outcome 重跑一致。
- `G-DEMO-TEAM-ISOLATION`：兩隊 bootstrap/frame 只含各自 disclosed state。
- `G-DEMO-PRESENTATION`：fog、circle、counts、LastKnown 僅依 filtered state。
- `G-DEMO-LAUNCHER`：一 server、兩獨立 frontend、不同 identity/log、PID-scoped cleanup。
- `G-DEMO-OBSERVER`：兩隊 observer 消費實際 wire bytes且不阻塞 outbound。
- `G-DEMO-MANUAL`：雙視窗人工步驟可觀察不同集合、reveal、hide、overlap 與 ghost。

## Risks / Trade-offs

- **[Risk] 現有 Lua schema 無法直接表示 team/demo spawn** → 新增最小、可驗證 descriptor，避免把 demo 特例散落於一般 importer。
- **[Risk] 100 個 sprite 與 fog overlay 降低可讀性** → 固定 spacing、team 色彩與半透明 overlay，camera 初始 framing涵蓋局部而非完整地圖。
- **[Risk] LastKnown cache 尚未接完整 renderer** → 以獨立 render-only cache/marker完成，不把 ghost 混入 deterministic entity list。
- **[Risk] Windows 多 process 環境變數互相污染** → 每個 `Start-Process` 使用獨立 environment/argument block與 log suffix。
- **[Risk] 視窗位置 API 在多螢幕差異** → 以 primary work area 左右排列；無法定位時仍啟動並在 log 警告，不影響 match correctness。

## Migration Plan

1. 新增 opt-in content 與 loader validation，不改預設 story。
2. 接上 authoritative demo scene、team projection 與 focused tests。
3. 加入 omfx demo presentation 與 window identity/placement。
4. 修正 `run_2player.bat` 並保持 CRLF。
5. 集中跑所有 blocking gates，最後啟動雙視窗供人工驗收。

Rollback 只需停止使用 `FOG_2TEAM_DEMO` 並還原 launcher入口；active secure match 仍禁止 runtime downgrade。

## Open Questions

無。未指定細節依 deterministic、fail-closed、最小 public contract 與不擴張 disclosure 原則由實作者決定。

