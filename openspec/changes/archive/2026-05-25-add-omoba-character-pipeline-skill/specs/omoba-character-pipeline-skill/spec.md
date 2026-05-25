## ADDED Requirements

### Requirement: Repo-local character pipeline skill exists

系統 SHALL 提供 repo-local Codex skill `.codex/skills/omoba-character-pipeline/`，用來引導 omoba 角色生成流程。skill SHALL 描述 `bootstrap`、`new`、`draft-package`、`review`、`revise` 階段，並 SHALL 明確宣告第一版不直接修改 `omb/`、`omfx/`、`scripts/base_content/` runtime/gameplay。

#### Scenario: Skill metadata is discoverable

- **WHEN** Codex 掃描 repo-local skills
- **THEN** `.codex/skills/omoba-character-pipeline/SKILL.md` 存在
- **AND** skill 說明它用於從角色描述與技能文本產生 omoba 角色 package
- **AND** skill 說明實作第一步是 bootstrap/verify Ubuntu toolchain

#### Scenario: Skill keeps first version isolated from runtime import

- **WHEN** 使用者要求 draft 第一版角色 package
- **THEN** skill 指引 SHALL 寫入 `docs/character_pipeline/packages/<hero_id>/`
- **AND** skill SHALL NOT 指引直接修改 `omb/`、`omfx/` 或 `scripts/base_content/`

### Requirement: Bootstrap protects shared Windows and Linux AI roots

系統 SHALL 將 `/media/damody/新增磁碟區/AI_Pic` 視為 Windows/Linux shared asset root。bootstrap SHALL 可讀取 shared root 中的 ComfyUI、stable-diffusion-webui、models、workflow 與 outputs，但 SHALL NOT 覆蓋或刪除 Windows portable Python、`venv/Scripts`、`python_embeded` 或既有模型輸出。

Linux 專用 venv SHALL 建立在 shared tool checkout 外，預設路徑 SHALL 是 `~/.cache/omoba-character-pipeline/venvs/`。

#### Scenario: Windows venv is detected but not modified

- **WHEN** bootstrap 檢查 `/media/damody/新增磁碟區/AI_Pic/stable-diffusion-webui/venv/Scripts/python.exe`
- **THEN** bootstrap SHALL 將它診斷為 Windows-shaped venv
- **AND** bootstrap SHALL NOT 刪除、重建或覆蓋該 `venv`
- **AND** bootstrap SHALL 使用 Linux 專用 venv 路徑

#### Scenario: Shared root is probed safely

- **WHEN** bootstrap 驗證 shared asset root
- **THEN** bootstrap SHALL 確認 root 可讀
- **AND** bootstrap SHALL 使用暫存 probe file 驗證可寫
- **AND** bootstrap SHALL 刪除 probe file
- **AND** bootstrap SHALL NOT 修改 models、outputs、workflow 或 Windows portable runtime

### Requirement: Bootstrap verifies Ubuntu AI toolchain before package generation

系統 SHALL 在 package generation 前執行 toolchain bootstrap/verify。bootstrap SHALL 檢查 GPU、Python、PyTorch CUDA、ComfyUI 或 stable-diffusion-webui provider、Blender CLI、Lua interpreter 與 shared root。

#### Scenario: GPU and torch are verified

- **WHEN** bootstrap 執行於 Ubuntu
- **THEN** bootstrap SHALL 嘗試讀取 `nvidia-smi`
- **AND** bootstrap SHALL 在 Linux venv 內 import `torch`
- **AND** bootstrap SHALL 驗證 `torch.cuda.is_available()` 為 true
- **AND** 若任一步失敗，bootstrap SHALL 回報 `ToolchainError`

#### Scenario: Provider smoke test is required

- **WHEN** bootstrap 檢查 2D provider
- **THEN** bootstrap SHALL 優先 smoke-test ComfyUI
- **AND** 若 ComfyUI 不可用，bootstrap SHALL 嘗試 stable-diffusion-webui API fallback
- **AND** 若兩者皆不可用，bootstrap SHALL 回報 `ToolchainError`

#### Scenario: Blender and Lua are verified

- **WHEN** bootstrap 執行環境檢查
- **THEN** bootstrap SHALL 驗證 Blender 可以執行 background Python expression
- **AND** bootstrap SHALL 驗證 `lua5.4` 或相容 Lua interpreter 可載入 package Lua files

### Requirement: Character package uses Lua source files

系統 SHALL 以 Lua 作為角色 package source of truth。每個 package SHALL 位於 `docs/character_pipeline/packages/<hero_id>/`，且 SHALL 包含 `character.lua`、`prompts.lua`、`manifest.lua`、`toolchain.lua`、`omoba_stub.lua` 與 `review.md`。

每個 Lua source file SHALL 使用 `return function(ctx) return { schema_version = 1, ... } end` 形態。

#### Scenario: Draft package creates required files

- **WHEN** skill draft 一個新 hero package
- **THEN** `docs/character_pipeline/packages/<hero_id>/character.lua` 存在
- **AND** `prompts.lua` 存在
- **AND** `manifest.lua` 存在
- **AND** `toolchain.lua` 存在
- **AND** `omoba_stub.lua` 存在
- **AND** `review.md` 存在

#### Scenario: Lua package fields are valid

- **WHEN** validator 載入 package Lua files
- **THEN** 每個 Lua file SHALL 回傳 table
- **AND** 每個 table SHALL 包含 `schema_version = 1`
- **AND** `character.lua` SHALL 包含 hero id、display name、title、role、art direction、abilities 與 automation 設定

### Requirement: Package validation enforces omoba-safe identifiers and paths

系統 SHALL 提供 validator，驗證 hero id、ability ids、ability slots、provider references 與 manifest paths。hero id SHALL 使用 ASCII snake_case。ability ids SHALL 唯一。Q/W/E/R slots SHALL 不重複。manifest paths SHALL 使用平台中立相對路徑或明確 shared root reference，且 SHALL NOT 包含 unsafe `..` traversal。

#### Scenario: Invalid hero id is rejected

- **WHEN** `character.lua` 的 hero id 包含空白、非 ASCII 字元或大寫字母
- **THEN** validator SHALL 回報 `SchemaError`
- **AND** error message SHALL 指出 invalid hero id

#### Scenario: Duplicate ability slot is rejected

- **WHEN** `character.lua` 定義兩個 ability 使用相同 slot
- **THEN** validator SHALL 回報 `SchemaError`
- **AND** error message SHALL 指出 duplicate slot

#### Scenario: Unsafe manifest path is rejected

- **WHEN** `manifest.lua` 包含 `..` traversal 或 Windows-only backslash path
- **THEN** validator SHALL 回報 `SchemaError`
- **AND** error message SHALL 指出 unsafe path

### Requirement: Batch review summarizes AI assumptions and game readiness

系統 SHALL 使用批次 review gate。AI MAY 在 `auto_decide` 模式補齊缺失角色設定、prompt、seed、工具 provider 與 omoba stub，但 `review.md` SHALL 列出 AI-made assumptions、角色/技能一致性、MOBA/TD 視角可讀性、icon 小尺寸可讀性、model/rig 風險、animation coverage 與 omoba import readiness。

#### Scenario: Auto decisions are visible in review

- **WHEN** skill 使用 `auto_decide` 產生 package
- **THEN** `review.md` SHALL 列出 AI 自行決定的重要欄位
- **AND** 使用者 SHALL 能一次審核整個 package

#### Scenario: Game contract warning does not block draft

- **WHEN** package valid 但 `omoba_stub.lua` 包含未來 import 風險
- **THEN** validator SHALL 將其分類為 `GameContractWarning`
- **AND** `review.md` SHALL 顯示 warning
- **AND** draft package SHALL 仍可完成

### Requirement: omoba stub captures future import metadata without applying it

`omoba_stub.lua` SHALL 記錄未來匯入 omoba 所需 metadata，包括 hero id、display name、title、ability ids、Q/W/E/R slots、portrait/icon path、model slot、animation clips、script hints 與 draft template stats。第一版 SHALL NOT 自動套用這些資料到 templates、scripts 或 omfx assets。

#### Scenario: Stub contains expected asset slots

- **WHEN** skill draft 一個 hero package
- **THEN** `omoba_stub.lua` SHALL 包含 portrait/icon 預期路徑
- **AND** SHALL 包含 model slot，例如 `omfx/data/heroes/<hero_id>/<hero_id>.glb`
- **AND** SHALL 包含 `idle`、`run`、`attack`、`cast_q`、`cast_w`、`cast_e`、`cast_r`、`death` animation clips

#### Scenario: Stub is not applied automatically

- **WHEN** package draft 或 review 完成
- **THEN** templates、scripts 與 omfx runtime files SHALL 不會因本流程自動修改
- **AND** `omoba_stub.lua` SHALL 保持為未來 import 草案
