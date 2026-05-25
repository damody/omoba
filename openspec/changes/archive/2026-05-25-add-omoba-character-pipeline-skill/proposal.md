## Why

目前角色生成流程橫跨角色設定、技能文本、2D 圖像、三視圖、3D 模型、骨架、動畫與 omoba/omfx 匯入資料，但缺少一個可重跑、可審核、能先檢查 Ubuntu 工具鏈的標準工作流。

這個 change 建立 repo-local Codex skill，讓一段角色描述與技能文本可以先產生完整 Lua 角色包與工具鏈規格，後續再逐步接上實際 AI 生成與遊戲匯入。

## What Changes

- 新增 `.codex/skills/omoba-character-pipeline/` repo-local skill。
- 新增角色生成 package 規格，輸出到 `docs/character_pipeline/packages/<hero_id>/`。
- 使用 Lua 作為角色包 source of truth，不使用 YAML。
- 將 Ubuntu toolchain bootstrap/verify 放在流程第一步，先檢查或準備 GPU、Python、PyTorch、ComfyUI、stable-diffusion-webui、Blender 與共享 AI 資產根目錄。
- 支援 Windows/Linux 共用 `/media/damody/新增磁碟區/AI_Pic` 的模型、workflow 與 outputs，但 Linux venv 與可執行環境獨立建立，不覆蓋 Windows portable/venv。
- 新增批次 review gate：AI 可先全權補齊預設值，最後產生 `review.md` 讓使用者一次審整包。
- 產生 `omoba_stub.lua`，先記錄未來對接 omb/omfx/templates/script 的 hero id、ability slots、asset slots、animation clips 與 script hints。

## Capabilities

### New Capabilities

- `omoba-character-pipeline-skill`: 定義 repo-local 角色生成 skill、Ubuntu bootstrap、Lua 角色包 schema、批次審核與未來 omoba 匯入 stub 的需求。

### Modified Capabilities

- 無。

## Impact

- 新增 OpenSpec change artifacts。
- 新增 `.codex/skills/omoba-character-pipeline/` skill 文件與可能的輔助 scripts。
- 新增 `docs/character_pipeline/` 下的 schema/fixture/tooling 文件或測試資料。
- 會讀取既有共享 AI 工具路徑，但不得修改 Windows portable Python 或 Windows venv。
- 第一版不修改 `omb/`、`omfx/`、`scripts/base_content/` 的 runtime/gameplay 程式碼。
