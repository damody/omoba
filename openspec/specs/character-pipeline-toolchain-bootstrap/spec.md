# character-pipeline-toolchain-bootstrap Specification

## Purpose
TBD - created by archiving change bootstrap-character-pipeline-toolchain. Update Purpose after archive.
## Requirements
### Requirement: Linux toolchain preparation uses user-local cache

系統 SHALL 將角色生成 pipeline 的 Linux venv、下載工具與設定狀態放在 `~/.cache/omoba-character-pipeline/` 或使用者指定的等效 cache root。系統 SHALL NOT 在 shared AI root 中建立 Linux venv，且 SHALL NOT 覆蓋 Windows portable runtime。

#### Scenario: Prepare creates Linux venv outside shared root

- **WHEN** 使用者執行 toolchain prepare
- **THEN** Linux venv SHALL 建立在 user-local cache
- **AND** `/media/damody/新增磁碟區/AI_Pic/stable-diffusion-webui/venv/Scripts` SHALL 不被修改
- **AND** `/media/damody/新增磁碟區/AI_Pic/ComfyUI/python_embeded` SHALL 不被修改

### Requirement: PyTorch CUDA is installed and verified in Linux venv

系統 SHALL 在 Linux venv 中安裝 PyTorch，並 SHALL 驗證 `torch.cuda.is_available()`。驗證 SHALL 使用 venv Python，不使用系統 Python 作為權威。

#### Scenario: Torch CUDA verification passes

- **WHEN** toolchain prepare 成功
- **THEN** venv Python 執行 `import torch`
- **AND** `torch.cuda.is_available()` SHALL 回傳 true
- **AND** 診斷報告 SHALL 顯示 torch version 與 CUDA 狀態

### Requirement: ComfyUI Linux smoke is available

系統 SHALL 能使用 Linux venv 對 shared ComfyUI checkout 做 basic smoke。smoke MAY 是 dependency import、`main.py --help`、短暫 HTTP startup probe 或等效非長時間生圖檢查。

#### Scenario: ComfyUI smoke does not modify Windows runtime

- **WHEN** ComfyUI smoke 執行
- **THEN** 使用 Linux venv Python
- **AND** shared ComfyUI source path MAY 被讀取
- **AND** Windows `python_embeded` SHALL 不被執行或修改

### Requirement: Blender executable is available for background smoke

系統 SHALL 提供 Blender executable path。若 PATH 上沒有 Blender，系統 SHALL 使用 user-local cache 的 Blender binary。驗證 SHALL 執行 background Python expression。

#### Scenario: Blender background smoke passes

- **WHEN** bootstrap verify 檢查 Blender
- **THEN** Blender SHALL 能執行 `--background --python-expr`
- **AND** 診斷報告 SHALL 記錄使用的 executable path

### Requirement: First real package smoke passes validator

系統 SHALL 建立一個非 `hero_example` 的 smoke package，使用 Lua package schema，並 SHALL 通過 validator。該 package SHALL 不產生大型 PNG/GLB；manifest assets MAY 保持 `planned`。

#### Scenario: Smoke package validates

- **WHEN** smoke package 建立完成
- **THEN** package path SHALL 位於 `docs/character_pipeline/packages/<hero_id>/`
- **AND** hero id SHALL 為 ASCII snake_case
- **AND** validator SHALL 回報 package valid

