# omoba 架構海報 SVG — Design

**目標**：產出一張 8K 解析度 / 16:9 SVG 學術海報，呈現 omoba 專案六個 crate（`omb` / `omfx` / `omoba-core` / `map_editor` / `omoba-template-ids` / `scripts`）的完整架構，並以內嵌 SMIL 動畫呈現三條敘事軌：build/codegen pipeline、ECS tick heartbeat、KCP transport packet flow。

**輸出檔**：`docs/diagrams/omoba-architecture-poster.svg`（單檔，純 SVG markup；無檔案大小上限，可放大量裝飾與細節）。

---

## 1. 整體版面

| 屬性 | 值 |
|---|---|
| `viewBox` | `0 0 7680 4320`（8K，16:9） |
| 畫布長寬比 | 16 : 9（橫向，方便螢幕投影 / 海報直印） |
| 字型策略 | system stack — `-apple-system, BlinkMacSystemFont, "PingFang TC", "Microsoft JhengHei UI", sans-serif`；代碼識別符 fallback `JetBrains Mono → Cascadia Code → Consolas → monospace` |
| Web font | 不使用（檔案大、印刷端不一定可用） |

**版面三段式（橫向，y 留 240 標題 / 280 圖例）**：

```
y=0    ┌──────────────────────────────────────────────────┐
       │                  TITLE BAR                       │
y=240  ├────────┬───────────────────────┬─────────────────┤
       │  L     │      M                │      R          │
       │ BUILD  │  omb host (ECS)       │  TRANSPORT      │
       │ amber  │  green tick heartbeat │  cyan packet    │
       │ 0–2280 │  2280–4880            │  4880–7000      │
y=4040 ├────────┴───────────────────────┴─────────────────┤
       │              LEGEND / METRICS                     │
y=4320 └──────────────────────────────────────────────────┘
```

右側 x=7000–7680 區塊作為 sidebar（mini node graph + 浮層說明）。

---

## 2. 配色系統

| 用途 | Hex | 備註 |
|---|---|---|
| 背景 | `#0b1220` | 深海軍藍，比純黑柔和 |
| 卡片底 | `#141d2e` | 比背景亮 8% |
| 主文字 | `#e6edf3` | 暖白 |
| 副文字 | `#94a3b8` | 冷灰 |
| build 軌 | `#f59e0b` | warm amber，`templates.json` → DLL |
| transport 軌 | `#22d3ee` | cyan，`omb` → `omfx` |
| FFI 邊界 | `#a78bfa` | 紫，`abi_stable` cdylib boundary |
| ECS tick | `#4ade80` | 綠，dispatcher 脈動 |
| 警告 | `#fb7185` | 玫瑰紅，rustc 同版約束等 |

四個事件 dot 顏色（packet flow 用）：
- `hero.stats` → `#4ade80`（綠）
- `creep.M` → `#facc15`（黃）
- `projectile.C` → `#f97316`（橘）
- `heartbeat` → `#94a3b8`（灰）

---

## 3. 左半 — Build / Codegen 軌（amber 流光）

5 個卡片從上到下串成 build pipeline，色帶 `#f59e0b` 貫穿主動脈：

| # | 卡片 | 內容 |
|---|---|---|
| ① | `Story/templates.json` | Tower / Creep / Hero entries · single source of truth |
| ② | `omoba-template-ids`（codegen） | sequential u16 newtype · `TowerKindId` / `CreepKindId` · serde + abi_stable safe |
| ③ | `scripts/script-abi`（FFI 邊界） | `UnitScript` · `AbilityScript` · `GameWorld` trait · `StatKey` enum（138 variants）· `BuffId` enum |
| ④ | `scripts/base_content`（cdylib） | `towers/` · `heroes/` · `summons/` · abilities (Q W E R hooks) · → `base_content.dll` |
| ⑤ | DLL hot-load | 複製到 `omb/scripts/`；abi_stable 要求 host + cdylib 同 rustc |

**FFI 邊界視覺化**：③ 卡片左下到右下用紫色虛線 `stroke-dasharray="80 48"` 配 `<animate stroke-dashoffset>` 緩慢流動，明顯標示「跨 FFI 從這裡開始」。

**`map_editor`** 放在左半下方獨立小卡（脫離主動脈，因它是離線 tooling）。

---

## 4. 中央 — omb host ECS（green tick heartbeat）

5 個 layer + 上方 ECS Resources bar，從上到下：

| Layer | 模組 | 重點型別 / 函式 |
|---|---|---|
| Resources | top bar | `GameMode` · `PlayerLives` · `TowerKind` · `DeltaTime` · `ScriptEventQueue` · `BuffStore` |
| A: state/ | resource_management | `upgrade_skill` · `move_player` · `build_hero_stats_payload()` |
| B: scripting/ | dispatch · registry · world_adapter | `ScriptEvent → AbilityScript`，DLL handle cache，`GameWorld` FFI 回呼 |
| C: ability_runtime/ | BuffStore · UnitStats · Dispatcher | `sum_add(StatKey)`、`final_atk` / `final_msd` / `final_attack_range` |
| D: tick/ | parallel-ish systems | `buff_tick → ability → projectile_tick → hero_tick → creep_tick → summon_tick` |
| E: transport/ | OutboundMsg → KCP encoder | feature gate（`mqtt` / `grpc` / `kcp`，default = kcp） |

**動畫（C 軌 ECS tick heartbeat）**：綠色光環從 Layer A 出發，每 3 秒繞一圈：A→B→C→D→E，每個 layer 的左邊界亮起 0.5 秒。光環走到 Layer E 後從右邊界吐出一顆 cyan dot 進右軌（與右半 packet flow 接續）。Layer D 額外有一條紫色細線連回左半 ⑤ DLL（標 `world_adapter` 的 FFI 雙向回呼）。

**FFI 雙向掃光**：紫虛線在中央與左半之間每 ~8 秒做一次「先左→右 host call script，再右→左 script call back GameWorld」。

---

## 5. 右半 — Transport / Runtime 軌（cyan packet flow）

6 個卡片，遵循 packet 實際旅程：

| # | 卡片 | 內容 |
|---|---|---|
| ⑥ | `omb` host ECS broadcast | `state/resource_management.rs` · `build_hero_stats_payload()` · `OutboundMsg::typed`（P2）· tower / creep / hero / projectile 廣播站 |
| ⑦ | `omoba-core` encode 層 | `ability_meta` · `tower_meta` · `grpc/` (tonic) · `kcp/` (prost + tag framing) · `proto/game.proto` (P9 stripped) |
| ⑧ | KCP wire（UDP 50061） | `lz4_flex` high-bit tag (P1) · two-tier batch window (P6) · per-player AOI broadphase (P5) · seq gap detection |
| ⑨ | `omoba-core` kcp client decode | `GameEventData` typed/legacy · `Position16` / `Fixed16` / `Facing8` · client cache merge (`HeroStatic` + `HeroHot`) |
| ⑩ | `omfx` (Fyrox engine) | `network/` EventBuffer (BinaryHeap) · `render/` scene + ECS sync · `ui/` eui (immediate-mode) · creep velocity extrapolation |
| ⑪ | Fyrox executor | spawn `omobab.exe`（hard-coded debug 路徑）· 60 fps render loop |

**動畫（A 軌 packet flow）**：8–12 顆小色點沿 ⑥→⑦→⑧→⑨→⑩ 連續灌流。每顆 dot 帶 1 個事件類型色（見配色），用 `<animateMotion>` 沿一條 `<path>` 跑，每顆起點隨機延遲 0–4 秒讓密度看起來不規則。在 ⑧ KCP wire 段，dot 體積會「壓縮」（scale 0.6→1.0 重新放大）視覺化 lz4。

**P0–P9 優化標記**：每個優化標籤（`P5 AOI` 等）用紫色小圓點掛在對應卡片邊緣。

---

## 6. 周邊（標題、圖例、metrics、sidebar）

**頂部標題列（y=0~240）**：

- 主標 `omoba` 88pt + 副標 36pt 繁中說明
- 漸層分隔線（amber → purple → green → cyan）

**底部圖例列（y=4040~4320）四欄**：

1. Color legend：4 個色票對應四軌
2. Animation legend：build pulse / packet flow / tick heartbeat 的小型示意
3. Tech stack：`Rust 1.91.0 · specs 0.20 · abi_stable 0.11 · Fyrox · prost · tokio_kcp · lz4_flex`
4. Metrics 數字方塊：stress 場景 1000 塔 × 1000 creep · KCP P9 後 traffic 顯著下降 · 60 fps 維持 · `payload_bytes` 監控

**右側 sidebar（x=7000~7680）**：

- 6 crate 互依關係 mini node graph（scripts/script-abi 居中為樞紐）
- 「為什麼分兩 cargo workspace」短引文
- `map_editor` / `omb-mcp` 兩個離線工具的小卡

**左下浮層（x=40~720, y=3500~4030）**：

- 「rustc 同版約束」警示框（玫瑰紅邊）
- 紫色 `⚠ FFI safety` icon + 兩行說明

---

## 7. 動畫實作（純 SMIL，無 JS 依賴）

| 動畫 | 元素 | 實作 |
|---|---|---|
| build pulse | `<rect>` 漸層發光圈沿主動脈 | `<animate attributeName="y" dur="6s" repeatCount="indefinite"/>` 沿 5 卡片 y 軸跳，`keyTimes` 控停留時間 |
| packet flow | `<circle>` × 12 沿 `<path>` | `<animateMotion dur="4s" begin="0s/0.3s/0.7s/...">`，begin 錯開讓密度不規則 |
| lz4 壓縮視覺 | `<animateTransform type="scale">` | KCP 卡片那段路徑上 0.6→1.0 變化 |
| tick heartbeat | layer 邊框 `stroke-opacity` 輪流脈動 | 5 個 `<animate>` 共用 dur=3s，每個 begin offset 0.6s |
| FFI 雙向掃光 | `<path stroke-dashoffset>` | dur=8s，前 4s 正向後 4s 反向（keyTimes 切） |

所有動畫 `repeatCount="indefinite"`。Print fallback：CSS `@media print { animateMotion, animate { display: none; } }` — 印刷時動畫凍結在第一幀，主結構仍完整可讀。

---

## 8. 檔案規模

- 不設大小上限 — 海報優先細節豐富
- 預估渲染效能：50–100 個並發 SMIL 動畫 element 在現代瀏覽器仍流暢
- 瀏覽器相容：Firefox / Safari / Chromium 皆原生支援 SMIL

---

## 9. 實作步驟概要

1. 建立 `<svg viewBox="0 0 7680 4320">` 框架 + 背景 + `<defs>`（漸層、filter、reusable symbols）
2. 標題列 + 漸層分隔線
3. 三軌主結構：左 build 5 卡 / 中 ECS 5 layer / 右 transport 6 卡
4. 主動脈 path（amber + cyan）+ FFI 紫虛線
5. SMIL 動畫：build pulse、packet flow、tick heartbeat、FFI 雙向掃光、lz4 壓縮
6. 圖例列、sidebar mini node graph、左下警示
7. CSS print fallback、`@font-face` 不嵌
8. 自我檢查：閱讀視線是否從左走到右（build → ECS → transport → render）

---

## 10. 不在 scope 內

- **互動式 hover-reveal**：海報優先、不依賴滑鼠（user 選 C，A2 印刷導向）
- **真實 metrics 數據**：暫用占位符（`P9 後 traffic ↓ XX%`），需要 omb-mcp 量測後填回
- **map_editor / omb-mcp 內部結構**：兩個離線工具只占小卡，不展開模組
