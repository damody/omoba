## 1. Skill 骨架與文件

- [x] 1.1 新增 `.codex/skills/omoba-character-pipeline/SKILL.md`，描述 skill 目的、觸發時機、階段與第一版邊界。
- [x] 1.2 在 skill 文件中定義 `bootstrap`、`new`、`draft-package`、`review`、`revise` 的操作流程與狀態機。
- [x] 1.3 在 skill 文件中明確記錄第一版不得直接修改 `omb/`、`omfx/`、`scripts/base_content/` runtime/gameplay。

## 2. Lua Package Schema 與 Fixture

- [x] 2.1 新增 `docs/character_pipeline/schema/` 文件，說明 `character.lua`、`prompts.lua`、`manifest.lua`、`toolchain.lua`、`omoba_stub.lua` 與 `review.md` 欄位。
- [x] 2.2 新增 fixture package `docs/character_pipeline/packages/hero_example/`，包含完整 Lua files 與 `review.md`。
- [x] 2.3 確保 fixture 使用 ASCII snake_case hero id、唯一 ability ids、Q/W/E/R slot 與平台中立相對路徑。

## 3. Bootstrap 診斷工具

- [x] 3.1 新增 bootstrap script，檢查 `nvidia-smi`、Python、Linux venv root、shared AI root、ComfyUI path、stable-diffusion-webui path、Blender 與 Lua interpreter。
- [x] 3.2 bootstrap script 偵測 Windows-shaped venv/portable Python 時只回報診斷，不刪除、不覆蓋、不重建。
- [x] 3.3 bootstrap script 驗證 shared root read/write probe，probe 後刪除暫存檔。
- [x] 3.4 bootstrap script 輸出結構化診斷結果，區分 `ToolchainError`、`SharedRootError` 與可接受 warning。

## 4. Lua Package Validator

- [x] 4.1 新增 validator script，可用 `lua5.4` 載入 package Lua files 並確認每個 file 回傳 table 與 `schema_version = 1`。
- [x] 4.2 validator 檢查 hero id ASCII snake_case、ability id 唯一、Q/W/E/R slot 不重複。
- [x] 4.3 validator 檢查 `prompts.lua` provider references 存在於 `toolchain.lua`。
- [x] 4.4 validator 檢查 `manifest.lua` paths 不含 unsafe `..` traversal 或 Windows-only backslash。
- [x] 4.5 validator 將阻擋錯誤分類為 `SchemaError`，將未來 import 風險分類為 `GameContractWarning`。

## 5. Package Draft 與 Review 指引

- [x] 5.1 在 skill 文件中加入從角色描述與技能文本產生 Lua package 的具體填寫規則。
- [x] 5.2 在 skill 文件中加入 `auto_decide` 預設行為，要求 AI 自動補齊缺欄位並記錄 assumptions。
- [x] 5.3 在 skill 文件中加入 `review.md` 產生規則，涵蓋角色/技能一致性、視角可讀性、icon 小尺寸可讀性、model/rig 風險、animation coverage 與 omoba import readiness。
- [x] 5.4 在 `omoba_stub.lua` fixture 中示範 portrait/icon path、model slot、animation clips、script hints 與 draft stats。

## 6. 驗證

- [x] 6.1 執行 bootstrap script 的安全診斷模式，確認它不修改 shared root 的 Windows venv/portable Python。
- [x] 6.2 執行 validator 驗證 fixture package 成功。
- [x] 6.3 建立一個臨時 invalid fixture 或測試輸入，確認 invalid hero id、duplicate slot、unsafe path 會失敗。
- [x] 6.4 執行 `openspec validate add-omoba-character-pipeline-skill --strict`，確認 change artifacts 通過驗證。
