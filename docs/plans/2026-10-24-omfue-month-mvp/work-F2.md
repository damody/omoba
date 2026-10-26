# F2 工作紀錄：解除 omfx 資產依賴

```text
日期：2026-10-25
目前工作 ID／狀態：F2／done（非 UE 部分）；UE Editor 端驗證未執行，交回主 session
玩家或作者新增能做到的事：作者改英雄頭像時只動 omfue/SourceArt/ContentData/data/hero_portraits/；正式 generate、asset recipe、art-swap 驗收都不讀 omfx。缺圖時，工具會指出英雄、recipe 欄位與路徑。
repo HEAD／本次修改：root b1f00b9、omfue d311ad4（皆含其他 session 的 dirty 變更）；本次未 commit。修改檔見下方 §4。
來源英雄與 variant：不適用
已執行命令、exit code、build identity：見 §5
L1／L2／L3／L4 證據（fresh／沿用／未執行）：L1 fresh（codegen 單元測試、Lua 測試、靜態依賴檢查）；L2–L4 未執行（不能動 UE）
gate 與原 11 主項的變化：F2 完成條件「正式 build、generate、art acceptance 都不讀 omfx」在原始碼層成立；Editor import/PIE 的實際確認未執行
失敗、原因、尚未修的風險：見 §6
下一個工作 ID：主 session 決定；F2 的 UE 驗證見 §7
人工／硬體需求與是否已取得：無
owned process／資源清理結果：未啟動 UE 或對局；只跑 cargo test、codegen --check 與 Lua。暫存 fixture 目錄已刪除。
```

## 1. 正式路徑對 omfx 的引用清單（修正前）

掃描範圍：根目錄 `*.bat`、`scripts/`（含 `lua_data`、`tests`）、`proto/`、`omoba-*`、`omb/src`、`omb-mcp`、`omfue/codegen`、`omfue/bridge`、`omfue/restart/src`、`omfue/Source`、`omfue/Plugins/OmRuntime/Source`、`omfue/Config`。排除 `omfx/` 本身、`docs/`、各 `target`／`Intermediate`／`Saved`。

**實際讀檔或路徑依賴（本次已解除）**

| 位置 | 內容 |
|---|---|
| `omfue/codegen/src/lib.rs:3079` | `portrait_source: recipe_source("omfx", &hero.portrait)` |
| `omfue/codegen/src/lib.rs:3715` | 測試期望 `omfx/data/portrait.png` |
| `omfue/Plugins/OmRuntime/Source/OmGenerated/om_asset_recipe.json:13,68` | 生成的 `portrait_source` 指向 `omfx/data/hero_portraits/...`（saika_magoichi、date_masamune） |
| `scripts/tests/ue_art_swap_editor_test.lua:15` | 替換圖 `omfx/data/hero_portraits/hero_default_portrait.png` |

**仍存在、列入允許清單（不屬正式 UE 流程）**

| 位置 | 理由 |
|---|---|
| `scripts/run.lua`、`run_2player.lua`、`run_2player_interactive.lua`、`run_fog_lifecycle.lua`、`dev_run_freshness.lua` | 歷史 Fyrox launcher，接在 `run.bat`／`run_2player.bat` 後面，會建置或啟動 `omfx/executor`。正式 UE 入口（`run_ue.bat` → `unreal_entry.lua`）不經過這些檔。 |
| `omoba-core/src/runtime/production_guards.rs:4,98,124,134` | `#[cfg(test)]` 測試用 `include_str!("../../../omfx/game/src/...")`。**omfx 不存在時 `cargo test -p omoba-core` 會編譯失敗**；正式 build 不受影響。不在本次檔案範圍，列為後續（§6）。 |

**只在文字或註解提到 omfx（不是依賴，不列入檢查）**：`omoba-core/Cargo.toml`、`omoba-sim/Cargo.toml`、`omoba-core/src/**`、`omoba-sim/src/**`、`omb/src/**`、`proto/game.proto` 的註解，`scripts/base_content/assets/*/README.md`，`tools/lua/migration-manifest.md`。

**技能 icon**：`scripts/lua_data/templates/abilities.lua` 的 `icon`（`data/hero1_*.png`、`data/ability_icons/...`）只被 codegen 寫成 `FOmGeneratedAbilityMetadata::IconPath` 字串。UE 原始碼沒有任何地方用它讀檔（HUD 用的是 `omfue/Content/Omoba/UI` 的 atlas），所以不是檔案依賴，也不需要複製。`ability_default_placeholder.png` 只有 omfx 使用。

## 2. 新素材位置與理由

**`omfue/SourceArt/ContentData/`**。內容資料裡的 `data/...` 路徑（例如英雄 `portrait`）以這裡為根解析，因此實際檔案是 `omfue/SourceArt/ContentData/data/hero_portraits/<name>.png`。

- `omfue/SourceArt/` 已經是 omfue 擁有、受版控的來源美術目錄，在 `Content/` 外面，不會被 cook。`Characters/MaterialsV1` 已經用 `manifest.json` 加 sha256 記錄來源，本次照這個做法。
- 根目錄下保留 Lua 的 `data/...` 命名空間，codegen 只要換根目錄，不需要改寫路徑；`scripts/lua_data` 裡的 `portrait` 字串完全沒動，content hash 不變（`codegen --check` 的 content_hash 仍是 `8f951268d0dc9f6a`）。
- 沒選 `scripts/base_content/assets/`：那裡是 script DLL 擁有的 TD 圖，而且要做 `data/` 前綴改寫。沒選 `scripts/lua_data/`：依指示不能動。
- 代價：檔案在 omfue submodule 內，正式提交時要先在 omfue commit／push，再 bump 主 repo 的 gitlink。

## 3. 複製的檔案與 hash

來源 omfx commit `ec9bb4a836fe8ea7ee5823a3a45c47e1fd67f55f`（`data/hero_portraits` 最後修改；omfx HEAD `05640d6`）。只複製，omfx 的檔案沒有搬移或刪除。複製後 hash 與來源一致。記錄在 `omfue/SourceArt/ContentData/manifest.json`。

| 新路徑（`omfue/SourceArt/ContentData/data/hero_portraits/`） | 來源 | SHA-256 | 尺寸 | 用途 |
|---|---|---|---|---|
| `hero_saika_magoichi_portrait.png` | `omfx/data/hero_portraits/hero_saika_magoichi_portrait.png` | `841893c9f87e66c5087eec8f676dde7d793aa90cab4cf82675bad61f446fb0a9` | 160×160 | saika_magoichi `portrait_source` |
| `hero_date_masamune_portrait.png` | `omfx/data/hero_portraits/hero_date_masamune_portrait.png` | `55314a79ba6ddfaafa4b95f6dd265f74995b75912ac27db2a3f6e7f9ee67f38f` | 160×160 | date_masamune `portrait_source` |
| `hero_default_portrait.png` | `omfx/data/hero_portraits/hero_default_portrait.png` | `39b902dbf424b603049e97fc9f8e742cddca912a1322f40b0f1e5edd52245e97` | 160×160 | art-swap 驗收的替換圖 |

## 4. 修改檔

| 檔案 | 變更 |
|---|---|
| `omfue/codegen/src/lib.rs` | 新增 `CONTENT_DATA_SOURCE_ROOT = "omfue/SourceArt/ContentData"`，`portrait_source` 改用它；非法 portrait 路徑的錯誤會帶 `hero '<id>' field 'portrait'`。測試改為期望新根目錄、確認 recipe 不含 `omfx`，並涵蓋 `../omfx/...` 的診斷。`GENERATOR_VERSION` 不變。 |
| `omfue/Plugins/OmRuntime/Source/OmGenerated/om_asset_recipe.json` | 兩行 `portrait_source` 改到新根目錄。以 `codegen --check` 確認與 generator 輸出一致，其他生成檔不變。 |
| `omfue/SourceArt/ContentData/`（新） | 三張 PNG 加 `manifest.json`。 |
| `scripts/ue_asset_recipe_plan.lua` | 每個 job 帶 `hero`、`field`（`model_source`、`texture_source`、`portrait_source`、`animation_sources.<slot>`）。拒絕 `omfx/` 開頭的 source。新增 `missing_sources(plan, is_file)` 與 `describe_missing(missing)`。 |
| `scripts/ue_apply_asset_recipe.lua` | preflight 先一次列出所有缺檔，格式是 `hero <id> field <field>: missing <path>`，任何 import 前就失敗。job signature 欄位不變。 |
| `scripts/tests/ue_art_swap_editor_test.lua` | 替換圖改為 `omfue/SourceArt/ContentData/data/hero_portraits/hero_default_portrait.png`；缺檔時在動 Editor 前就失敗。 |
| `scripts/tests/ue_asset_recipe_test.lua` | 檢查真實 recipe 的所有 source 都存在、portrait 在新根目錄、planner 拒絕 omfx source，以及缺檔診斷包含英雄、欄位與路徑。 |
| `scripts/maintained_source_deps.lua`（新） | maintained-source 依賴檢查（可 `require`，也可直接執行）：掃描 §1 的正式根目錄，找 `omfx/`、`omfx\`、`"omfx"`、`'omfx'` 這類路徑引用（文字提到 omfx 不算）。允許清單逐檔寫理由，部分只允許特定行；不再命中的允許項目會報 stale。另外檢查 recipe 每個 source 都存在且不在 omfx，以及 ContentData manifest 的檔案與 sha256。 |
| `scripts/test_lua_tooling.lua` | 加上 fixture（路徑引用會被抓、文字不會、stale 允許項目會報），並對真實 repo 執行 `assert_clean`。 |

## 5. 命令與 exit code

| 命令 | exit | 結果 |
|---|---|---|
| `cargo test --manifest-path omfue/codegen/Cargo.toml` | 0 | 86 passed／0 failed |
| `cargo run --manifest-path codegen/Cargo.toml -- --content-root ../scripts/lua_data --out Plugins/OmRuntime/Source/OmGenerated --check`（cwd `omfue`，唯讀） | 0 | `checked 21 files from 24 Lua inputs, content_hash=8f951268d0dc9f6a` |
| `tools\lua\lua.exe scripts\tests\ue_asset_recipe_test.lua` | 0 | passed |
| `tools\lua\lua.exe scripts\maintained_source_deps.lua` | 0 | passed |
| `tools\lua\lua.exe scripts\ue_apply_asset_recipe.lua --dry-run` | 0 | dry-run 不連 Editor；兩張 portrait 從新根目錄解析，hash 與表 3 相同；寫出 `omfue/Saved/McpAutomation/AssetRecipe/dry-run.json` |
| 從 scratchpad 跑 `test_lua_tooling.lua` 新增段落的同等 fixture | 0 | fixture 與 `assert_clean` 通過（第一次跑時抓到 fixture 本身的字串，已加入允許清單） |
| `lua -e "assert(loadfile(...))"`：`test_lua_tooling.lua`、`ue_art_swap_editor_test.lua`、`ue_apply_asset_recipe.lua` | 0 | 語法 OK |

**沒有跑完整的 `scripts/test_lua_tooling.lua`**：它會重寫 `scripts/lua_data/TD_STRESS/map.lua`（內容相同，但 mtime 會變），還會跑 memory dump fixture。依限制不碰 `scripts/lua_data`，所以只單獨驗證新增段落。

## 6. 風險與後續

- **`omoba-core` 測試依賴 omfx 原始碼**（`production_guards.rs` 的 `include_str!`）：2026-10-25 由後續 session 修掉，guard 不再讀 omfx，允許清單的這一項也已移除（未 commit）。`maintained_source_deps.lua` 重跑 exit 0。據該 session 回報，omoba-core 的 1095 個 lib 測試通過；`cargo test` 不加篩選時另有兩項失敗，與這次修改無關。§1 表格保留修正前的狀態。
- **下次真的 apply recipe 時，portrait 會重新 import 一次**：job signature 含 `source` 路徑，路徑變了，ledger 會判定為已變更，`/Game/OmGenerated/Heroes/{saika_magoichi,date_masamune}/RecipeV1/Portrait/*` 會用相同 bytes 再 import 一次。這是 owned asset（ledger 有紀錄），不會被當成 unowned 拒絕。
- `run.bat`／`run_2player.bat` 仍是 Fyrox 入口；它們依賴 omfx 是設計使然，不屬於 UE 正式流程。
- 新檔在 omfue submodule 內，提交時照 AGENTS.md 的 submodule 規則：先在 omfue commit／push，再 bump 主 repo 的 gitlink。本次未 commit。

## 7. 未執行的 UE 驗證（交回主 session）

- `tools\lua\lua.exe scripts\ue_apply_asset_recipe.lua`（非 dry-run，需要 Editor MCP）：確認兩張 portrait 從新路徑 re-import，結果是 `imported-verified`，而且只重新 import portrait。
- `tools\lua\lua.exe scripts\tests\ue_art_swap_editor_test.lua`：用新的替換圖跑 baseline／replacement／restored 三個階段。
- 完整 `scripts/build_ue_bridge.lua`／`build_ue_moba.lua`：codegen 以寫入模式重跑，預期 `om_asset_recipe.json` 不變（已用 `--check` 確認）。
- 在沒有 omfx checkout 的乾淨目錄跑正式 generate 與驗收：本次只做了靜態證明（依賴檢查加上 recipe source 存在性），沒有實際移走 omfx。可以併入 J6 的乾淨打包。
- cook／soft reference 診斷（schedule F2 有提到）：portrait 目前只走 recipe import，沒有 soft reference 進 cook，所以本次沒有對應的 cook 檢查。日後 UI 改用 import 後的 portrait asset 時再補。
