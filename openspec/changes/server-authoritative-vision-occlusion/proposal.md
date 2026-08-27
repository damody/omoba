## Why

目前 server 權威的隊伍視野只依距離判定，樹木與不規則地形無法阻擋視線，因此兩隊收到的單位資訊仍可能穿透場景障礙。需要在既有 selective lockstep 的 Wave B 投影階段加入可重現的遮蔽判定，確保送出佇列與同 process replica team 觀戰驗算都使用相同規則。

## What Changes

- 新增固定點、確定性的線段對圓形樹木與簡單多邊形地形遮蔽判定。
- 擴充地圖資料，明確描述有穩定 ID 的視野樹木與不規則遮蔽物。
- 在 Rust `specs` 的 Wave B 中，將距離、朝向扇形、遮蔽、各隊投影與 replica team 預期結果平行計算後一次提交。
- 維持 server authoritative：前端只渲染 server 已揭露的單位，不自行推導或補回被遮蔽資訊。
- 擴充雙玩家 demo，加入足量樹木與不規則地形，讓兩隊可移動英雄並觀察單位進出扇形視野及遮蔽後立即隱藏。
- 將完整測試、效能量測、雙 process 人工驗收集中在實作末段執行。

## Capabilities

### New Capabilities

- `deterministic-vision-occlusion`: 定義 server 權威、固定點、可由 replica team 重播驗算的樹木與不規則地形視野遮蔽。
- `vision-occlusion-demo`: 定義雙玩家 demo 的遮蔽物配置、可移動英雄、隊伍隔離顯示與人工驗收行為。

### Modified Capabilities

- `sim-snapshot-rendering`: 前端 snapshot 渲染必須依 server 揭露結果移除離開視野或遭遮蔽的敵方單位，並呈現視野遮蔽物與扇形邊界。

## Impact

- `omoba-core`：地圖 schema、native components、固定點幾何、Wave B visibility projection 與測試。
- `omb`：地圖載入、server 送出投影、同 process replica team 觀戰驗算與指標。
- `omfx`：遮蔽物渲染、server snapshot 消失語意、雙玩家 demo 操作與視覺提示。
- demo/launcher 與 OpenSpec evidence：新增遮蔽場景、最終測試、效能與雙視窗驗收證據。
- 不新增第三方 runtime dependency，也不改變 server 權威與 client input-only 的安全邊界。
