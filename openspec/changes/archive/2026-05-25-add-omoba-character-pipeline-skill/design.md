## Context

omoba 目前已有 Lua-driven gameplay content、script-owned assets、omfx render metadata 與英雄技能 runtime，但角色生成本身還沒有標準化流程。新的角色可能需要從文字描述一路產生立繪、技能圖、三視圖、3D 模型、骨架、動畫 prompt 與未來遊戲匯入資料；如果沒有 package contract，AI 工具輸出會變成不可重跑的散檔。

使用者已有共享 AI 工具根目錄 `/media/damody/新增磁碟區/AI_Pic`，同時給 Windows 與 Ubuntu 使用。實際檢查顯示 GPU 可用，但現有 stable-diffusion-webui 與 ComfyUI 目錄偏 Windows/portable 形態，Python venv 不可跨 OS 共用。因此此 change 必須把共享大檔資產與平台專屬執行環境分開。

## Goals / Non-Goals

**Goals:**

- 建立 repo-local Codex skill：`.codex/skills/omoba-character-pipeline/`。
- 讓 skill 第一階段執行 Ubuntu toolchain bootstrap/verify，缺工具先處理，不等到計畫寫完才安裝。
- 定義 Lua 角色包 schema，輸出在 `docs/character_pipeline/packages/<hero_id>/`。
- 讓 AI 預設可全自動補齊角色設定、prompt、seed、manifest 與 omoba stub，最後再批次審核。
- 保護 Windows/Linux 共用資料夾：共享 models/workflows/outputs，但不共享 venv 或覆蓋 portable Python。
- 提供 validator/smoke tooling，確認 Lua package 可載入、欄位完整、路徑安全、provider 設定一致。

**Non-Goals:**

- 第一版不直接修改 `omb/`、`omfx/`、`scripts/base_content/` runtime/gameplay。
- 第一版不跑長時間 GPU 生圖、3D 生成或動畫生成作為必跑測試。
- 第一版不要求 NVIDIA Kimodo、Omniverse 或任一商業/研究工具為必要依賴。
- 第一版不把 package 自動 import 到正式遊戲 assets。

## Decisions

### Decision: 使用 repo-local skill

`.codex/skills/omoba-character-pipeline/` 直接放在 omoba repo 內，因為第一版會依賴本專案的 hero id、Lua data、omfx asset slot 與 OpenSpec 文件慣例。

替代方案是放在 `~/.codex/skills` 做 global skill，但這會讓 omoba-specific 路徑與 schema 變成隱性假設。

### Decision: bootstrap 是第一個可執行階段

skill SHALL 先檢查 GPU、Python、PyTorch、ComfyUI、stable-diffusion-webui、Blender、Lua interpreter 與 shared root。缺工具時，implementation task 必須先準備工具或回報明確 blocker。

替代方案是先只寫 package，再等 run-assets 階段才安裝工具；這會延後暴露 Linux/Windows venv 不相容、torch 缺失、Blender 缺失等問題。

### Decision: Lua 是 package source of truth

角色包使用 `character.lua`、`prompts.lua`、`manifest.lua`、`toolchain.lua`、`omoba_stub.lua`。這貼近既有 `scripts/lua_data/*` 風格，也讓未來轉成 templates/story patch 比 YAML 更自然。

工具若需要 JSON，應由 runner 從 Lua table 轉出，不要求使用者維護 JSON/YAML。

### Decision: 共享資產、隔離執行環境

`/media/damody/新增磁碟區/AI_Pic` 是 shared asset root。Linux venv 預設放在 `~/.cache/omoba-character-pipeline/venvs/`，不得覆蓋 `venv/Scripts`、`python_embeded` 或 Windows portable trees。

這比直接在 shared checkout 裡重建 `venv` 安全，因為 NTFS 共用資料夾同時被 Windows 與 Linux 使用。

### Decision: ComfyUI 為 primary 2D provider

ComfyUI 優先，stable-diffusion-webui 作 fallback。ComfyUI workflow JSON、固定 seed 與 HTTP API 比 UI-driven generation 更適合自動化。stable-diffusion-webui 保留是因為既有模型較多。

### Decision: 第一版先產生可審核 package，不直接 import

`omoba_stub.lua` 只產生未來匯入 metadata：hero id、ability ids、asset slots、animation clips、script hints 與 draft stats。正式寫入 templates/scripts/omfx assets 由後續 change 處理。

這讓第一版可以先穩定角色生成 contract，不把 gameplay/runtime 改動混進工具 bootstrap。

## Risks / Trade-offs

- [Risk] Linux 建立 venv 或安裝 PyTorch 需要大量磁碟與網路時間。→ Mitigation：bootstrap 先診斷，必要時分步安裝；不把長時間 GPU job 當必跑測試。
- [Risk] Windows/Linux 共用 NTFS 造成 exec bit、symlink、大小寫或行尾問題。→ Mitigation：不在 shared root 放 Linux venv；package 路徑使用 ASCII hero id 與平台中立相對路徑。
- [Risk] ComfyUI portable layout 不是 Linux 原生安裝。→ Mitigation：用 Linux venv 執行 shared checkout 的 `main.py`，models/workflows 指向 shared root。
- [Risk] Lua schema 過早鎖死 3D/animation provider。→ Mitigation：manifest 與 prompts 保留 provider slot，但第一版只要求 package/validator，不要求實際 3D 生成成功。
- [Risk] AI 全自動補欄位產生不可接受的美術或 gameplay 假設。→ Mitigation：`review.md` 必須列出 AI assumptions，批次 gate 讓使用者整包審核。

## Migration Plan

1. 新增 repo-local skill 與 bootstrap/package/review 操作說明。
2. 新增 Lua package schema 文件、fixture package 與 validator。
3. 新增 bootstrap 診斷 script，先只做安全檢查與可選環境建立，不改 Windows portable/venv。
4. 以 fixture 驗證 validator 與 skill 指引可運作。
5. 後續 change 才加入實際 asset generation 與 omoba import。
