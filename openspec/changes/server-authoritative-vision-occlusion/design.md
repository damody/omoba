## Context

既有 selective lockstep 已由 server 在 Rust `specs` 的 Wave B 依隊伍建立不同的 disclosure projection，並把實際送出佇列交給同 process、另一 thread 的 replica team observer 驗算。目前 visibility 只有半徑與 detection level，無法表達樹木或不規則地形造成的方向性陰影。舊 `omb/src/vision` 使用 f32 且不在正式 Wave B 資料流中，不能直接作為跨平台 lockstep 判定。

詳細幾何與架構基準見 `docs/superpowers/specs/2026-08-27-server-authoritative-vision-occlusion-design.md`。

## Goals / Non-Goals

**Goals:**

- 以固定點 LOS 判定圓形樹木及凸、凹簡單多邊形的遮蔽。
- 在 Wave B 的 immutable committed view 中，將距離、扇形方向、遮蔽、各隊投影與 replica 預期值平行計算。
- 維持 server 權威與 fail-closed，讓每隊只收到自身能看到的即時狀態。
- 讓雙玩家 demo 可直接觀察 Reveal、Forget、樹後扇形陰影及不規則地形遮蔽。
- 保持既有地圖相容，空遮蔽物集合必須退化為目前行為。

**Non-Goals:**

- 不實作高度、坡度、半透明、樹木破壞或移動遮蔽物。
- 不讓單位身體阻擋視野。
- 不由 client viewport、GPU 或本地碰撞決定 disclosure。
- 不保留敵方 LastKnown presentation。

## Decisions

### 使用固定點精確 LOS

樹木使用線段最近點對圓判定；多邊形使用 orientation/cross-product 線段相交與 point-in-polygon。所有中間運算使用足以容納乘積的擴寬整數或 checked arithmetic。相較固定角度 ray fan，這不會因取樣角度產生漏光；相較 tile grid，也不會把既有連續座標量化成格子。

切線、碰頂點、沿邊重疊及 target 位於遮蔽物內都算遮擋。source 位於遮蔽物內時忽略該遮蔽物，避免出生點永久失明。運算無法安全完成時回傳不可見並送出限流 diagnostic。

### 遮蔽資料與移動碰撞分離

地圖新增 `VisionTrees` 與 `VisionOccluderPolygons`，每筆都有同地圖唯一且非零的 `StableId`。它們不自動沿用 `BlockedRegions`；需要同時阻擋移動與視野時，由 content 明確宣告兩份資料。載入時拒絕非正半徑、重複相鄰頂點、自交或零面積多邊形。

runtime 使用穩定排序的 `VisionOccluderSet`，並在 Wave B committed view 投影成附 AABB 的 `CommittedVisionOccluder`。遮蔽物本身不是玩家 replica entity，也不提供可被 client targeting 的 gameplay identity。

### 將遮蔽納入既有 Wave B team projection

Wave A 完成位置與 outcomes commit 後，Wave B 一次讀取 immutable entity、vision source 與 occluder view。每個 team 可由 Rayon 平行處理，只寫自己的 current set、transitions、hash 與 bytes。候選 entity 只要被任一合法己方 source 以無遮擋 LOS 看見即揭露；全部 LOS 受阻才 Forget。

所有 source、target、occluder 與 transition 依 canonical key 排序，因此 worker 完成順序不得影響 bytes。Replica team observer 繼續只讀實際送出佇列，不得直接讀 canonical world 當答案。

### 前端只呈現 server disclosure

omfx 對 `Forget` 或 snapshot 缺席的敵方 entity 移除 scene node、identity 與 UI，不建立 remembered ghost。樹木、多邊形輪廓與視野扇形只作為 demo presentation，不參與權威判定，也不能補畫 server 未揭露的單位。

### 先用 AABB broad phase，再以效能 gate 決定 spatial index

第一版逐 LOS 以線段 AABB 篩除遮蔽物，再做精確 narrow phase，減少資料結構風險。最終以 100 個普通單位、兩位英雄、至少 64 棵樹與三個多邊形、120 Hz 量測；Wave B p99 不得高於無遮蔽 baseline 兩倍，server tick 不得持續超過 8.33 ms。未通過才加入固定 cell spatial index，且不得改變判定結果。

## Risks / Trade-offs

- [大量 source-target-occluder 組合增加 Wave B 成本] → 先用靜態 AABB broad phase；以固定壓力場景量測，必要時加入確定性 spatial index。
- [幾何邊界與 overflow 造成跨平台差異] → 全程固定點與擴寬整數，建立邊界、極值及重排 determinism tests；異常時 fail closed。
- [地圖作者誤以為碰撞區自然遮蔽視野] → schema、驗證錯誤與文件明確區分 `BlockedRegions` 和 vision occluders。
- [前端保留 stale entity 洩漏資訊] → 對 team snapshot 缺席與 `Forget` 建立 entity/node/identity 全清除測試，人工驗收 remembered count 必須為零。
- [凹多邊形或自交資料語意不清] → 支援任意 winding 的簡單凹多邊形，自交與零面積在載入期拒絕。

## Migration Plan

1. 加入可預設為空的地圖 schema 與驗證，確保舊地圖輸出不變。
2. 加入獨立固定點幾何 API 與單元測試資料，但暫不接入 projection。
3. 將 validated occluders 放入 committed view，再接入 Wave B team projection 與 replica observer。
4. 擴充 snapshot presentation 與 `FOG_2TEAM_DEMO` 遮蔽物配置。
5. 完成所有程式變更後，集中執行完整測試、determinism、效能與雙 process 人工驗收。

回復時可先移除地圖的兩個 vision 欄位；空集合會回到純半徑/扇形判定。若程式需回退，新增欄位保持 optional/default empty，不要求 wire schema 降版。

## Open Questions

無。實作細節遇到未涵蓋邊界時，依 server authoritative、固定點確定性、資訊最小揭露與 fail-closed 的優先序決定。
