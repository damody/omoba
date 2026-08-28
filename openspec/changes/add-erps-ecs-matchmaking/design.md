## Context

ERPS 是 monorepo 內的新服務，但必須是與 `omb` 分離的獨立 process。Client 直接透過 gRPC 排隊；`omb` 以 game server 身分註冊並接收 launch command。現有本地 `specs` fork 已支援 parallel dispatcher，適合保存大量 player、party、ticket、proposal、match 與 server 狀態，但 party 裝箱與跨 bucket 搜尋不適合直接以全域 ECS join 暴力處理。

系統的關鍵限制是：單節點記憶體 authority、100,000 名排隊玩家、deterministic 結果、party 不可拆、ready check 後才配置容量、異質 server 加權成本，以及 Rust／C SDK 的穩定網路契約。已核准的完整產品設計位於 `docs/superpowers/specs/2026-08-28-erps-ecs-matchmaking-design.md`。

## Goals / Non-Goals

**Goals:**

- 以 Specs ECS 建立可平行、可重現且不重複 claim 的配對核心。
- 支援 1v1、5v5 與八人自由混戰的指定 party 規則及 Elo 品質策略。
- 以 ready check、信用分與保留等待時間的恢復規則處理拒絕／逾時。
- 依 region、模式成本、總容量與 instance 上限安全配置 `omb` instances。
- 提供 versioned gRPC、Rust async SDK、C ABI poll SDK 與完整負載驗證。

**Non-Goals:**

- 多 ERPS 節點 active-active、distributed queue 或 queue 重啟恢復。
- Running match migration、自動雲端擴縮或管理 Web UI。
- Glicko／TrueSkill、玩家合作／作弊偵測。
- macOS 或 static C library 發行。

## Decisions

### 單一 ECS authority 加平行候選計算

gRPC task 只寫入 bounded command queue，不直接修改 `World`。Runtime 在短 batch window 後以穩定順序套用命令；依 region、mode、Elo bucket 建立不可變候選 snapshot，再由 Specs dispatcher／Rayon 平行評分。結果回到單一 deterministic claim／commit 階段，重新驗證 ticket、party revision、player 與 proposal state。

替代方案是純 ECS join 或每 shard 一個 `World`。前者無法有效處理 bounded bin-packing，後者會引入跨 world party／容量一致性問題。單一 authority 讓 invariant 與 replay 容易驗證，平行化則集中在最昂貴的只讀搜尋。

### Stable ID 與 deterministic tie-break

RPC 不暴露 Specs `Entity`。所有 domain object 使用 stable opaque ID；owner shard、命令順序、候選排序與 tie-break 都只依 logical time、設定、seed 與 stable ID。不得依賴 hash iteration 或 worker completion order。

### Bounded matching 而非全域最佳化

1v1 使用相容搜尋範圍內的最近 Elo。5v5 對 1～5 人 party 做 bounded 5+5 bin-packing；八人模式對 1～4 人 party 做 bounded sum-to-eight。評分兼顧等待時間、Elo 差／離散、party 結構與 stable tie-break。全域最佳解在 100,000 玩家下成本不可控，因此不採用。

### Proposal 與 placement 分離

候選先建立 proposal，所有玩家於預設 15 秒內個別同意。Ready check 期間只做 soft capacity feasibility，不建立 reservation；全員同意後才原子配置 server。只有 game instance 回報 `Ready` 並提供 endpoint／connection token 後才發布 match。

這避免未確認 proposal 占用容量，也防止 client 收到無法連線的假成功。代價是全員同意後仍可能短暫等待容量，系統以有期限的 placement waiting 與無懲罰回 queue 處理。

### 個人信用責任與 party 完整性

每位 player 個別回覆 proposal。主動拒絕預設扣 2、逾時預設扣 5；基礎設施錯誤不扣分。含失敗成員的 party 進入 `NotReady`，不自動踢人或改 roster；未受影響的完整 ticket 保留原 `enqueued_at` 回 queue。

Rating 與信用分透過 `PlayerProfileProvider` 隔離。Memory provider 供第一版與測試；production 可在不改 ECS／RPC lifecycle 的情況下接外部 profile service。

### 獨立 ERPS process 與雙服務面

Client 直接使用 `MatchmakingService`；建立 session 的 RPC 命名為 `OpenSession`，避免與 tonic generated client 的 `connect()` constructor 衝突，SDK 對外仍可提供 `connect()`。`omb` 使用 `GameServerService`。另有唯讀 admin service。Proto envelope 帶 major／minor，mutation 帶 `request_id`；所有 queues 有界，關鍵事件不可靜默丟棄。正式環境預設 TLS 與 token validator，明文只允許 loopback 或明確 development 設定。

### SDK 共用 contract、不同消費模型

Rust SDK 暴露 async API／event stream。C SDK 內部持有 Rust runtime 與背景網路執行緒，但只寫 bounded event queue；遊戲主迴圈以 `poll` 單 consumer 取得 library-owned opaque event，並使用明確 release API。這避免 callback 重入與跨 allocator lifetime 問題。

### 異質容量採雙重硬限制

每台 server 同時受 capacity units 與 `max_instances`（1～100）限制，各模式有不同 cost。ERPS policy 定義可信上限，server 不能自行提高。Placement 先過 region／mode／健康／容量／instance 硬條件，再依碎片、負載與近期 launch failure 排序。

## Risks / Trade-offs

- [單一 authority 限制水平擴充與容錯] → 第一版明確以 100,000 玩家單節點為驗收；隔離 command、profile 與 gRPC 邊界，未來另案設計 partition／replication。
- [Bounded search 可能不是全域最佳配對] → 固定 deterministic budget，優先等待公平性，再量測 Elo quality 與未匹配率調整 bucket／halo／budget。
- [Ready check 後容量可能消失] → 使用 placement waiting、reservation CAS、launch retry；infra failure 不扣分並保留等待時間。
- [Heartbeat 延遲可能造成暫時容量不一致] → Generation、可信 policy、reservation ledger、reconcile 與 timeout 回收共同防止超配。
- [Unicode 房名存在混淆字元] → NFC normalization 並只允許 Unicode letter／number；名稱只作顯示，加入仍靠不可猜的短效 token。
- [C ABI lifetime／thread misuse] → Opaque handle、單一 release、明確 thread contract、panic containment 與真實 C compiler smoke test。
- [100,000 玩家測試受硬體影響] → 正確性 invariant 為硬門檻；效能報告固定記錄硬體、workers、seed、設定並與同環境 baseline 比較。

## Migration Plan

1. 新增 crates、proto 與 feature-isolated build，不接既有啟動流程。
2. 完成 in-process ECS 核心與 deterministic tests。
3. 完成 gRPC server、Rust SDK、C SDK 及模擬 game server。
4. 讓 `omb` 以明確設定選擇是否註冊 ERPS；未啟用時既有 gameplay 啟動不變。
5. 在 development 環境以 load test 與少量真實 clients 驗證，再啟用 production TLS／profile provider。
6. 回滾時停止 client 導流與 `omb` ERPS 註冊，關閉獨立 ERPS process；既有 gameplay path 不需資料 migration。

## Open Questions

無。產品行為與第一版非目標已由核准設計定案；效能參數保留為設定與基準量測結果，不在此先固定硬體無關門檻。
