## Why

`omoba-character-pipeline` 的 package/validator 已完成，但 Ubuntu toolchain 目前仍缺 `torch` 與 Blender，ComfyUI 也尚未有 Linux 專用 venv smoke。需要先把本機 AI 工具鏈 bootstrap 到可執行狀態，才有辦法產生第一個真實角色 package 與後續資產。

## What Changes

- 建立 Linux 專用 Python venv，不使用或覆蓋 shared AI root 裡的 Windows `venv/Scripts` 或 `python_embeded`。
- 安裝/驗證 PyTorch CUDA，使 `torch.cuda.is_available()` 在 RTX 5090 上通過。
- 讓 ComfyUI 能從 Linux venv 啟動或至少完成 import/API smoke，模型與 workflow 仍指向 shared asset root。
- 安裝或配置 Blender Linux executable，讓 background Python smoke test 通過。
- 更新 bootstrap 診斷工具與文件，讓 toolchain 狀態從「只診斷缺工具」推進到「可準備與驗證」。
- 建立第一個非 fixture 角色 package smoke，證明 skill 能從描述產生 Lua package 並通過 validator。

## Capabilities

### New Capabilities

- `character-pipeline-toolchain-bootstrap`: 定義角色生成 pipeline 的 Ubuntu toolchain bootstrap、PyTorch/ComfyUI/Blender 驗證與 first package smoke 需求。

### Modified Capabilities

- 無。

## Impact

- 會新增或修改 `docs/character_pipeline/tools/` 的 bootstrap/setup scripts。
- 會新增 Linux venv 與下載/安裝內容到 user-local cache，例如 `~/.cache/omoba-character-pipeline/`。
- 會讀取 `/media/damody/新增磁碟區/AI_Pic` 下的 ComfyUI、stable-diffusion-webui 與 models，但不得修改 Windows portable runtimes。
- 可能新增一個 smoke package 到 `docs/character_pipeline/packages/<hero_id>/`。
- 不修改 `omb/`、`omfx/`、`scripts/base_content/` runtime/gameplay。
