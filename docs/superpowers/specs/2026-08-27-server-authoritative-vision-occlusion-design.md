# Server 權威視野遮蔽設計

## 1. 目標

擴充目前的 server-authoritative selective lockstep，使樹木與不規則地形能阻擋圓形視野中的部分方向，形成扇形陰影。玩家只能收到至少被一個己方視野來源以無遮擋視線看到的單位；離開視野或進入遮蔽區的單位必須由 server 送出 `Forget`，前端不得保留殘影。

本次改造保持 server 為唯一權威來源，不讓 viewport、前端視覺裁切或 client 提交的遮蔽資料參與可見性決策。

## 2. 範圍

### 2.1 納入範圍

- 樹木圓形遮蔽物。
- 任意簡單多邊形地形遮蔽物，包含凹多邊形。
- 固定點、跨平台一致的 line-of-sight 判定。
- 多個己方視野來源的「任一路徑可見」合併規則。
- Specs／Rayon Wave B team-parallel 計算。
- Lua 地圖資料、server 載入、demo 呈現與雙玩家驗證。
- 遮蔽造成的 Reveal／Forget transition、team hash 與 replica bytes 驗證。

### 2.2 不納入範圍

- 高度差、坡度、半透明樹冠或部分可見比例。
- 動態破壞樹木、種樹或移動地形。
- 單位身體本身阻擋視野。
- 將既有 `omb/src/vision` f32 陰影模組直接納入 lockstep。
- 用 client viewport 或 GPU visibility buffer 決定 server disclosure。

## 3. 核心決策

採用固定點精確 line-of-sight，而不採固定角度光線取樣或 tile visibility grid。

原因如下：

- 現有 simulation 已使用 `Fixed64`，可避免 f32 平台差異。
- 線段對圓與線段對多邊形可定義精確邊界，沒有角度取樣漏光問題。
- 地圖遮蔽物為靜態資料，可建立穩定排序與 AABB broad phase。
- 目前 demo 規模適合逐候選 LOS；未來可在不改公開契約的情況下替換 spatial index。

## 4. 資料模型

### 4.1 地圖資料

Lua map descriptor 新增：

- `VisionTrees`：每筆包含 `StableId`、`X`、`Y`、`Radius`。
- `VisionOccluderPolygons`：每筆包含 `StableId`、`Name` 與至少三個頂點。

`StableId` 在同一地圖內必須唯一且非零。座標與半徑必須有限，半徑必須大於零。多邊形不得有重複相鄰頂點、自交或零面積。

現有 `BlockedRegions` 不會自動成為視野遮蔽物。若地形需要同時阻擋移動和視野，地圖必須在兩個欄位明確宣告。這可避免既有地圖因碰撞資料而無意改變視野行為。

### 4.2 ECS 與 committed view

Server 初始化時將合法資料存入靜態 `VisionOccluderSet` resource。Wave B immutable read view 新增穩定排序的 `CommittedVisionOccluder`：

- `TreeCircle { stable_id, center, radius, aabb }`
- `TerrainPolygon { stable_id, vertices, aabb }`

所有座標在載入邊界轉換成 `Fixed64`。遮蔽物不需要 ECS entity，也不進入玩家 replica、遊戲 state hash 或可被 client 引用的 identity table；它只影響 server 產生的可見集合。若地圖希望公開樹木外觀，使用獨立的 public presentation descriptor，不暴露未公開 gameplay identity。

## 5. 可見性演算法

對每個 team 與候選 entity，依序執行：

1. `ServerOnly` 或 `ForceHide` 立即不可見。
2. `Public`、`ForceShow` 或本隊 `OwnerTeam` entity 立即可見。
3. 只保留 team 相同且 detection level 足夠的 vision source。
4. 目標中心必須位於 source 半徑內，邊界算可見候選。
5. 使用 source-target 線段 AABB 篩除不相交的遮蔽物。
6. 以固定點 narrow phase 判定線段是否在到達目標前撞到遮蔽物。
7. 任一 source 無遮擋即為可見；所有 source 都被遮擋才不可見。

### 5.1 樹木判定

使用線段最近點對圓心的平方距離，避免平方根。若最近點位於 source 與 target 之間，且距離平方小於或等於半徑平方，即為遮擋。

- 切線接觸算遮擋。
- source 位於該樹圓內時忽略該樹，避免出生點永久失明。
- target 中心位於樹圓內時視為被遮擋。
- 遮蔽物位於 target 後方不影響可見性。

圓形樹冠從 source 看去會自然在其後方形成由兩條切線界定的扇形陰影，不需要用浮點角度建構扇形。

### 5.2 不規則地形判定

對 source-target 線段與多邊形所有邊執行固定點 orientation／cross-product 線段相交判定。

- 穿過、碰到頂點或沿著邊界重疊都算遮擋。
- target 位於多邊形內或邊界上時不可見。
- source 位於多邊形內或邊界上時忽略該多邊形。
- 支援順時針、逆時針與凹多邊形。
- 自交多邊形在載入時拒絕，不能進入 runtime。

## 6. 平行與確定性

Wave A 完成 outcome、fact 與 position commit 後，Wave B 建立單一 immutable view，其中包含 entity、vision source 與遮蔽物。

每個 `TeamVisibilityState` 可由 Rayon 平行處理，但只寫入自己的 current set、candidate transition 與 history。遮蔽物按 `(kind, stable_id)` 排序；entity 與 source 也沿用 stable canonical order。輸出 transition 最後按 canonical ID 排序。

任何 team 的工作完成順序不得影響：

- Reveal／Forget 集合與順序。
- team hash。
- encoded team frame bytes。
- server replica observer 的驗算結果。

Replica team observer 維持在同一 process 的另一個 thread，使用實際送出佇列內容驗算，不存取 canonical world 作為答案捷徑。

## 7. 前端呈現

前端只繪製 filtered replica entity。`Forget` 會移除 deterministic node 與 identity，不建立 remembered presentation。

Demo 額外繪製已公開的樹木輪廓、地形輪廓與視野圓，方便人工確認陰影形狀。這些圖形只解釋 server 結果，不參與 disclosure。前端可保留與 server 相同的 presentation-only 裁切作為 transition-delay 防呆，但不得用它補畫 server 未揭露的 entity。

## 8. Demo 場景

擴充 `FOG_2TEAM_DEMO`：

- 保留 10×10、100 個普通單位與兩位額外英雄。
- 加入多組不同半徑的樹木，確保樹後有單位可供觀察。
- 加入至少一個凸多邊形與兩個凹多邊形地形。
- 兩位英雄起點各自能看到不同遮蔽案例。
- 英雄可用右鍵繞過樹木與地形，使單位 Reveal；回到陰影後立即 Forget。

## 9. 錯誤處理與安全

- 非法樹木或多邊形使 map load 明確失敗，不做靜默修復。
- 計算溢位使用擴寬整數中間值或 checked arithmetic；無法安全計算時 fail closed 為不可見並記錄限流 diagnostic。
- 未知遮蔽物種類 fail closed 並拒絕 content schema。
- 遮蔽物資料不接受 client mutation。
- Secure input 仍只允許已驗證玩家的座標型移動；entity targeting 必須使用 replica reference 與 disclosure epoch。

## 10. 效能策略

第一版使用靜態 AABB broad phase 加精確 narrow phase。每個 source-target pair 只檢查 AABB 與 LOS 線段重疊的遮蔽物。

Blocking gate：100 個普通單位、兩個英雄、至少 64 個樹木與三個多邊形，在 120 Hz demo 下 Wave B 的 p99 不得超過既有無遮蔽 baseline 的 2 倍，且整體 server tick 不得持續超過 8.33 ms。若 gate 未通過，僅可在不改判定結果的前提下加入固定 cell spatial index；不得降低遮蔽物數、tick rate 或測試規模來通過。

## 11. 測試與驗收

### 11.1 幾何單元測試

- 樹前、樹後、樹內、切線、target 後方樹木。
- source 在樹內。
- 凸與凹多邊形。
- 穿邊、碰頂點、沿邊、source 在內、target 在內。
- 順逆時針等價。
- 非法自交、重複點、零面積拒絕。

### 11.2 可見性與同步測試

- 半徑內但被遮擋時 Forget。
- 繞過遮蔽物時 Reveal 並取得即時 baseline。
- 多來源只有一條 LOS 暢通時仍 Reveal。
- detection level 與 override 優先序不變。
- 兩隊得到不同 disclosed set。
- hidden canonical ID、position 與遮蔽物內部資料不進入 team bytes。
- 不同 Rayon completion order 產生相同 transition、hash 與 bytes。
- replica observer 對兩隊各自驗算通過。

### 11.3 最終人工驗收

集中在所有實作完成後執行完整測試。啟動一個 server 與兩個獨立 omfx process：

- 確認兩隊視野不同。
- 確認樹木後方形成扇形不可見區。
- 確認凹多邊形能阻擋穿越其邊界的視線。
- 右鍵移動英雄繞過遮蔽物後單位 Reveal。
- 再次進入陰影後單位立即消失且 remembered count 為零。
- 擷取兩隊 before／after 畫面並保存 evidence。

## 12. 推出與回復

新欄位預設空陣列，因此沒有視野遮蔽物的既有地圖維持目前結果。功能由 content schema 是否包含遮蔽物資料自然啟用，不新增 client-controlled feature flag。

若需要回復，可移除地圖的 `VisionTrees` 與 `VisionOccluderPolygons`；server 演算法在空集合下退化為目前的半徑判定。Wire schema 不需要回退，因為遮蔽物不直接進入 team protocol。

## 13. 已決定事項

- 使用固定點精確 LOS。
- 樹木為圓形，不使用 f32 角度陰影模組。
- 不規則地形為明確的視野多邊形，不自動沿用 `BlockedRegions`。
- 邊界接觸一律視為遮擋。
- 任一己方視野來源無遮擋即可看見。
- 離開 LOS 使用 `Forget`，不保留 LastKnown。
- Server 結果優先，client 只做呈現與延遲防呆。

