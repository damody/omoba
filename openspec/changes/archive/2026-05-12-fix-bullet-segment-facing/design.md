## Context

`omfx` 目前會在前端根據 server snapshot 渲染 projectile。一般 projectile 以 `projectile_spawn_pos` 記錄第一次看到時的發射位置，並在每幀用目前 projectile 位置與 spawn position 算出拖尾矩形的長度與 `rotation`。使用者觀察到的問題是子彈線段有時沒有從發射點正確面向敵人，且飛行中可能旋轉，這代表目前視覺方向仍受到後續位置或目標更新影響。

這次變更限定在前端視覺層，目標是讓線段看起來從發射點朝向初始目標方向飛行；不改變 server projectile、傷害、命中、transport 或 script ABI。

## Goals / Non-Goals

**Goals:**
- 子彈線段建立時，使用發射點到初始目標位置或初始 projectile 位置的向量決定朝向。
- 子彈線段飛行期間維持同一個視覺朝向，不因目標移動、snapshot 插值或追蹤更新而旋轉。
- 保留現有拖尾長度上限、最小可見長度、顏色、z ordering 與清理流程。
- 讓修正可透過局部單元測試或小型 helper 測試驗證，避免只能靠手動觀察。

**Non-Goals:**
- 不修改 projectile 的 gameplay 軌跡、命中時間或傷害計算。
- 不新增 protocol 欄位或要求 server 傳送額外方向資料。
- 不重寫 projectile rendering 架構，也不改動 tower、hero 或 creep 的一般渲染流程。

## Decisions

- 在 client 端鎖定 projectile 視覺方向。
  - 理由：問題是渲染朝向不穩，client 已有發射點與當前 projectile 位置，可在第一次看到 projectile 時取得足夠資訊；不需要修改 server contract。
  - 替代方案：讓 server 傳 initial direction。這會增加 protocol 與 snapshot schema 變更，超出修正視覺 bug 所需範圍。

- 將「方向」與「當前位置」分離。
  - 理由：拖尾仍應跟隨 projectile 的當前位置移動，但 `rotation` 應使用固定方向；這可以修正飛行中旋轉，同時保留子彈移動感。
  - 替代方案：每幀用 `pos - spawn_pos` 重算方向。這是目前會造成旋轉或起始偏差的核心風險。

- 發射點優先使用現有 muzzle render position fallback。
  - 理由：`projectile_initial_spawn_pos` 已會嘗試從 owner muzzle 取得較可信的視覺發射點，失敗時 fallback 到 projectile position；沿用它可避免引入新資料來源。
  - 替代方案：一律用 projectile 第一次出現位置。這會讓剛建立時線段可能從 projectile 中心開始，較不符合「從發射點面向敵人」的需求。

- 用小型純計算 helper 驗證方向與拖尾端點。
  - 理由：Fyrox scene graph 渲染較難直接測試；把線段幾何計算抽出成純函式，可測試「初始 rotation 固定」與「tail/mid 計算正確」。
  - 替代方案：只做手動測試。這較容易回歸，且無法明確保證飛行中不旋轉。

## Risks / Trade-offs

- [Risk] 若 projectile 第一次出現時尚未離開發射點，初始方向向量可能接近零 → Mitigation：在方向長度太短時使用可用的目標位置、上一個穩定方向，或安全 fallback `(1, 0)`，並避免產生 NaN rotation。
- [Risk] 追蹤型 projectile 的視覺線段不再朝向移動中的目標即時轉向 → Mitigation：這是本變更刻意行為；需求要求飛行中不旋轉，命中與位置仍依現有邏輯更新。
- [Risk] X 軸翻轉與 batch quad rotation 慣例可能和 scene line segment 不同 → Mitigation：以既有 `QuadParams.rotation` 的座標系為準，新增測試或手動案例覆蓋水平、垂直與斜向射擊。
