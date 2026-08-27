## 1. 盤點與資料邊界

- [x] 1.1 在 `omoba-core` 找出目前 map descriptor 的 Rust 型別與 Lua 轉換入口，將檔案路徑記入 implementation notes
- [x] 1.2 找出 `BlockedRegions` 從 Lua 到 runtime resource 的載入流程，僅作為接線參考
- [x] 1.3 找出 `FOG_2TEAM_DEMO` 的 Lua 地圖檔與 100 個普通單位、兩位英雄的建立位置
- [x] 1.4 找出 Wave B 建立 immutable committed view 的函式與呼叫端
- [x] 1.5 找出每隊 visibility projection 寫入實際送出佇列的函式
- [x] 1.6 找出 replica team observer 消費送出佇列的 thread 入口
- [x] 1.7 找出 omfx 處理 team snapshot、Forget 與移除 scene node 的函式
- [x] 1.8 建立 `implementation-notes.md`，列出 1.1 至 1.7 的實際檔案與 symbol

## 2. 地圖 ABI 與資料型別

- [x] 2.1 在最接近現有 map descriptor 的 crate 新增 `VisionTreeDescriptor` 型別
- [x] 2.2 為 `VisionTreeDescriptor` 加入 `stable_id` 欄位
- [x] 2.3 為 `VisionTreeDescriptor` 加入固定點轉換前的 `x`、`y` 與 `radius` 欄位
- [x] 2.4 新增 `VisionOccluderPolygonDescriptor` 型別
- [x] 2.5 為 polygon descriptor 加入 `stable_id` 與 `name`
- [x] 2.6 為 polygon descriptor 加入有序頂點集合
- [x] 2.7 在 map descriptor 新增預設為空的 `vision_trees`
- [x] 2.8 在 map descriptor 新增預設為空的 `vision_occluder_polygons`
- [x] 2.9 更新 Lua/map FFI 邊界，使舊地圖省略新欄位時仍可載入
- [x] 2.10 更新 Lua/map FFI 邊界，使 `VisionTrees` 能轉成 Rust descriptor
- [x] 2.11 更新 Lua/map FFI 邊界，使 `VisionOccluderPolygons` 能保留頂點順序

## 3. 地圖遮蔽物驗證

- [x] 3.1 新增共用的 `StableId != 0` 驗證函式
- [x] 3.2 新增同地圖遮蔽物 `StableId` 不得重複的驗證
- [x] 3.3 驗證樹木 `radius > 0`
- [x] 3.4 驗證樹木座標與半徑可安全轉為 simulation fixed-point
- [x] 3.5 驗證多邊形至少包含三個頂點
- [x] 3.6 驗證多邊形沒有重複相鄰頂點
- [x] 3.7 驗證多邊形第一點與最後一點不以重複點閉合
- [x] 3.8 以固定點 signed area 驗證多邊形面積不為零
- [x] 3.9 以非相鄰邊相交檢查拒絕自交多邊形
- [x] 3.10 讓 map load error 包含遮蔽物種類、`StableId` 與失敗原因
- [x] 3.11 確認驗證不會自動修改 winding 或刪除非法頂點

## 4. Runtime 遮蔽物資源

- [x] 4.1 新增固定點 `VisionTreeCircle` runtime 型別
- [x] 4.2 新增固定點 `VisionTerrainPolygon` runtime 型別
- [x] 4.3 新增固定點 `VisionAabb` 型別
- [x] 4.4 為樹木建立包含半徑的 AABB
- [x] 4.5 為多邊形由頂點建立 AABB
- [x] 4.6 新增 `VisionOccluderSet` resource
- [x] 4.7 將合法 descriptor 轉成 `VisionOccluderSet`
- [x] 4.8 依 `(kind, stable_id)` 對 runtime occluders 穩定排序
- [x] 4.9 在沒有新地圖欄位時插入空的 `VisionOccluderSet`
- [x] 4.10 確認 vision occluders 不建立 ECS entity 或 replica identity

## 5. 固定點幾何小工具

- [x] 5.1 新增只處理整數/fixed raw value 的 2D cross-product helper
- [x] 5.2 新增 checked dot-product helper
- [x] 5.3 新增 point-on-segment 判定
- [x] 5.4 新增包含端點與共線重疊的 segment intersection 判定
- [x] 5.5 新增 point-in-simple-polygon 判定
- [x] 5.6 新增 segment AABB 建立函式
- [x] 5.7 新增兩個 AABB 是否相交的判定
- [x] 5.8 新增 source-target 線段對 tree circle 的最近點判定
- [x] 5.9 在 tree circle 判定加入切線等於遮擋的規則
- [x] 5.10 在 tree circle 判定忽略 source 位於樹內的情況
- [x] 5.11 在 tree circle 判定加入 target 位於樹內即遮擋
- [x] 5.12 確認 target 後方的樹不會遮擋
- [x] 5.13 新增 source-target 線段對 polygon edges 的遮擋判定
- [x] 5.14 在 polygon 判定忽略 source 位於內部或邊界的情況
- [x] 5.15 在 polygon 判定加入 target 位於內部或邊界即遮擋
- [x] 5.16 將 checked arithmetic 失敗統一轉成 `Blocked` 結果
- [x] 5.17 新增可限流的 geometry overflow diagnostic 計數器或 log 路徑

## 6. Committed view 接線

- [x] 6.1 新增 `CommittedVisionOccluder::TreeCircle` variant
- [x] 6.2 新增 `CommittedVisionOccluder::TerrainPolygon` variant
- [x] 6.3 將 tree center、radius、AABB 複製到 Wave B immutable view
- [x] 6.4 將 polygon vertices 與 AABB 複製或共享到 Wave B immutable view
- [x] 6.5 保證 committed occluder 順序不受 ECS 或 hash map iteration 影響
- [x] 6.6 將 occluder view 放入 Wave B 的唯讀輸入，不加入任何 team-local writer

## 7. 每隊 LOS 與可見集合

- [x] 7.1 保留 `ServerOnly` 與 `ForceHide` 的立即不可見優先序
- [x] 7.2 保留 `Public`、`ForceShow` 與 owner-team 的立即可見優先序
- [x] 7.3 在一般候選流程先套用既有 detection level
- [x] 7.4 在一般候選流程套用既有視野半徑
- [x] 7.5 若目前已有扇形/朝向條件，讓它在 LOS 前執行且保持既有邊界語意
- [x] 7.6 對通過前置條件的 source-target 建立 segment AABB
- [x] 7.7 只對 AABB 相交的 tree occluders 執行 narrow phase
- [x] 7.8 只對 AABB 相交的 polygon occluders 執行 narrow phase
- [x] 7.9 任一 occluder 擋住 LOS 時將該 source 判為不可見
- [x] 7.10 任一合法 source LOS 暢通時將 target 加入 team visible set
- [x] 7.11 所有合法 source 失敗時讓既有 transition logic 產生 Forget
- [x] 7.12 依 canonical entity key 排序 Reveal、Update 與 Forget
- [x] 7.13 確認 team frame bytes 不包含未揭露 target 的 ID、位置或狀態
- [x] 7.14 確認 occluder runtime identity 不會進入 team protocol bytes

## 8. Rayon 與 Replica team 驗算

- [x] 8.1 將 occluder view 以 immutable reference/共享值提供給所有 team workers
- [x] 8.2 確認每個 team worker 只寫自己的 visible set 與 transitions
- [x] 8.3 保留 team workers 可平行執行的 Wave B 排程
- [x] 8.4 將遮蔽後的 transitions 送入既有實際送出佇列
- [x] 8.5 讓另一 thread 的 team 1 observer 消費 team 1 實際佇列
- [x] 8.6 讓另一 thread 的 team 2 observer 消費 team 2 實際佇列
- [x] 8.7 讓 observer 對遮蔽後 replica 計算既有 team hash/bytes
- [x] 8.8 確認 observer 驗算不直接讀 canonical world entity state
- [x] 8.9 在 mismatch diagnostic 加入 team、tick 與首個不同 entity/transition

## 9. Demo 地圖內容

- [x] 9.1 保留 `FOG_2TEAM_DEMO` 的 10×10、100 個普通單位
- [x] 9.2 確認兩位英雄不計入 100 個普通單位
- [x] 9.3 為 team 1 英雄附近配置可直接看到的普通單位
- [x] 9.4 為 team 2 英雄附近配置不同的可直接看到普通單位
- [x] 9.5 新增至少 64 棵具唯一 `StableId` 的樹木
- [x] 9.6 使用至少兩種樹木半徑
- [x] 9.7 在 team 1 起點附近安排樹後觀察目標
- [x] 9.8 在 team 2 起點附近安排不同的樹後觀察目標
- [x] 9.9 新增一個合法凸多邊形視野地形
- [x] 9.10 新增第一個合法凹多邊形視野地形
- [x] 9.11 新增第二個形狀不同的合法凹多邊形視野地形
- [x] 9.12 安排英雄可用右鍵繞過每類遮蔽物的通道
- [x] 9.13 確認新增 vision occluders 不會意外阻止英雄移動，除非另有 `BlockedRegions`

## 10. omfx 呈現與操作

- [x] 10.1 確認每個 process 只綁定自己的 team 與 hero control identity
- [x] 10.2 確認右鍵地圖位置會送出自身英雄的 move input
- [x] 10.3 確認 UI 命中區不會吞掉一般地圖右鍵
- [x] 10.4 將 public tree descriptors 轉成 demo render data
- [x] 10.5 為樹木建立清楚可辨識的 presentation node 或 debug outline
- [x] 10.6 將 public polygon descriptors 轉成 demo render data
- [x] 10.7 為凸與凹多邊形建立 outline/fill presentation
- [x] 10.8 顯示本隊視野半徑與方向邊界，且不參與 disclosure
- [x] 10.9 在 team snapshot 缺席敵方 entity 時移除 render mirror
- [x] 10.10 在 Forget 時移除 scene node
- [x] 10.11 在 Forget 時移除名稱、血條與其他 entity UI
- [x] 10.12 在 Forget 時清除 selection、target 與 hit-test entry
- [x] 10.13 在 Forget 時移除 replica identity，不保留 remembered ghost
- [x] 10.14 後續 Reveal 時以新的 authoritative baseline 重建 entity
- [x] 10.15 client presentation 推估與 server 衝突時只採用 server 結果
- [x] 10.16 在 debug HUD 顯示 team、visible count、remembered count 與 replica status

## 11. 幾何與驗證測試（全部實作後集中執行）

- [x] 11.1 加入 tree 前方 target 不受遮擋的單元測試
- [x] 11.2 加入 tree 後方 target 受遮擋的單元測試
- [x] 11.3 加入 tree 切線受遮擋的單元測試
- [x] 11.4 加入 source 位於 tree 內會忽略該 tree 的單元測試
- [x] 11.5 加入 target 位於 tree 內受遮擋的單元測試
- [x] 11.6 加入 tree 位於 target 後方不遮擋的單元測試
- [x] 11.7 加入凸多邊形穿邊、碰頂點與沿邊測試
- [x] 11.8 加入凹多邊形穿越與凹口未相交測試
- [x] 11.9 加入多邊形順逆時針結果相同測試
- [x] 11.10 加入 source/target 位於多邊形內或邊界的測試
- [x] 11.11 加入重複 StableId map load 失敗測試
- [x] 11.12 加入非正樹木半徑 map load 失敗測試
- [x] 11.13 加入少於三點、重複點、零面積與自交 polygon 失敗測試
- [x] 11.14 加入 fixed-point 極值或 overflow 會 fail closed 的測試

## 12. Visibility、同步與確定性測試（全部實作後集中執行）

- [x] 12.1 加入半徑內但被樹遮擋不揭露的測試
- [x] 12.2 加入繞過樹後 Reveal 當前 authoritative baseline 的測試
- [x] 12.3 加入再次進入陰影後只產生一次 Forget 的測試
- [x] 12.4 加入兩個 source 只有一條 LOS 暢通仍揭露的測試
- [x] 12.5 加入 owner hero 不因遮蔽消失的測試
- [x] 12.6 加入 detection level 與 visibility override 優先序回歸測試
- [x] 12.7 加入 team 1 與 team 2 得到不同 disclosed set 的測試
- [x] 12.8 加入 hidden ID、位置與狀態不出現在 team bytes 的測試
- [x] 12.9 加入不同 occluder 輸入順序產生相同 bytes 的測試
- [x] 12.10 加入不同 Rayon completion order 產生相同 transitions/hash/bytes 的測試
- [x] 12.11 加入 team 1 observer 遮蔽 Reveal/Forget 驗算測試
- [x] 12.12 加入 team 2 observer 遮蔽 Reveal/Forget 驗算測試
- [x] 12.13 加入 omfx Forget 後 render/UI/identity 全移除測試
- [x] 12.14 加入 omfx Reveal 後以新 baseline 重建測試

## 13. 最終完整檢查與證據

- [x] 13.1 執行 OpenSpec strict validation，修正所有 artifact 錯誤
- [x] 13.2 執行 formatter 與靜態檢查，僅修正本 change 造成的問題
- [x] 13.3 執行 `cargo test --manifest-path omoba-core/Cargo.toml`
- [x] 13.4 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab`
- [x] 13.5 執行 `cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi`
- [x] 13.6 執行 `cargo test --manifest-path scripts/Cargo.toml -p base_content`
- [x] 13.7 執行 `cargo test --manifest-path omfx/Cargo.toml` 或該 workspace 的等效完整測試
- [x] 13.8 執行既有 selective lockstep determinism 與 security suites
- [x] 13.9 建立無遮蔽 baseline 的 Wave B p99 量測紀錄
- [x] 13.10 建立 64 trees + 3 polygons 場景的 Wave B p99 與 server tick 量測紀錄
- [x] 13.11 若效能 gate 失敗，加入不改結果的固定 cell spatial index 後重跑 13.8 至 13.10
- [x] 13.12 使用 `run_2player.bat` 或等效 launcher 啟動一個 server 與兩個獨立 omfx process
- [x] 13.13 人工確認兩隊各自看見不同單位且都看得到自己的英雄
- [x] 13.14 人工確認右鍵可移動兩位英雄
- [x] 13.15 人工確認樹後形成扇形陰影並可繞行 Reveal
- [x] 13.16 人工確認凸與凹多邊形阻擋 LOS
- [x] 13.17 人工確認回到陰影後單位立即消失且 remembered count 為零
- [x] 13.18 保存 team 1 與 team 2 的 before/reveal/forget 畫面
- [x] 13.19 保存測試、效能與 replica observer summary 到 change evidence
- [x] 13.20 檢查 git diff，確認沒有提交 build artifacts、log、DLL、EXE、PDB 或 cache
