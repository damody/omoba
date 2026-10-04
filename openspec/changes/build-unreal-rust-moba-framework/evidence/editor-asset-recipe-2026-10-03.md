# Editor 資產配方驗收（2026-10-03）

## 結果

`scripts/ue_apply_asset_recipe.lua` 已透過 BpGeneratorUltimate 2.0.6 的本機 MCP 套用生成配方。10 個唯一工作完成並驗證；Saika 的 7 個動畫槽共用 5 支 AnimSequence，兩個材質槽均引用生成的 BaseColor 材質。Date 與 training hero 無模型時保留 fallback，不為它們虛構模型或動畫。

配方目標使用 `/Game/OmGenerated/Heroes/<id>/RecipeV1`，未改寫既有 `/Game/RustBP` Blueprint。Saika mesh、Skeleton、材質、貼圖、頭像與動畫，以及 Date 頭像，共有 22 個保存套件（包含匯入器額外的 mesh／PhysicsAsset 與原始材質）。額外產物沒有被隱藏或刪除。

## 可重現指令

在 `D:\code\omoba` 執行；Editor 開啟、MCP 可用且 PIE 停止：

```text
tools\lua\lua.exe scripts\tests\ue_asset_recipe_test.lua
tools\lua\lua.exe scripts\ue_apply_asset_recipe.lua --dry-run
tools\lua\lua.exe scripts\ue_apply_asset_recipe.lua
tools\lua\lua.exe scripts\tests\ue_asset_recipe_editor_test.lua
```

最後 Editor 驗收輸出：

```text
Unreal asset recipe Editor acceptance passed: 22 unchanged packages, 10 verified jobs
```

Planner 測試也通過，覆蓋同來源去重、穩定輸出、fallback、版本錯配、越界來源、重複 ID／slot、目標碰撞與動畫缺模型。所有來源先驗證存在並取得 SHA-256，之後才允許 Editor 寫入。

## 型別、引用與重跑檢查

- SkeletalMesh／Texture2D 以 Editor 真實資產摘要驗證；模型依賴生成 Skeleton。
- AnimSequence 以真實型別查詢、正時長／frame 數及指定 Skeleton 依賴驗證，不採信匯入器錯誤的 `asset_type` 標籤。
- 明確指定貼圖 `type:BaseColor`，避免檔名自動猜測失敗；`validate_material` 回報 `is_valid:true`、`issue_count:0`，並驗證材質引用正確 Texture2D。
- 模型每個材質槽讀回生成材質的完整 object path，保存後驗證 Asset Registry 依賴。
- Ledger 保存來源 hash、操作參數及每一步的 importing／verified 狀態。首次匯入拒絕已存在但未被 ledger 擁有的目標；失敗保留紀錄供重試。已驗證資產遭外部改動時停止並報告，不靜默覆寫。
- 實際重跑 `imported:0`、`skipped:10`、`material_bindings_changed:0`，22 個 `.uasset` 的 SHA-256 全數不變。不是只檢查工具的成功字樣。

## 本機報告

- `omfue/Saved/McpAutomation/AssetRecipe/ledger.json`
- `omfue/Saved/McpAutomation/AssetRecipe/report.json`
- `omfue/Saved/McpAutomation/AssetRecipe/idempotence-report.json`：包含完整 package SHA-256 與兩次執行報告。
- 同目錄逐次 MCP 回應，可追查具體資產與工具診斷。

Saved 報告是本機驗收產物，不提交；本紀錄保存可重現流程與結果。

## 邊界

本驗收完成任務 3.1 的資產匯入、引用與重跑要求，不代表 native actor 已播放這批動畫。通用 skeletal 呈現元件、既有 Blueprint 回歸、HUD 與完整 MOBA 對局仍分別屬於 2.2b／3.2／6.x，尚未驗收。
