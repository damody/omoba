## Context

前一個 change 已建立 `omoba-character-pipeline` skill、Lua package schema、fixture、bootstrap 診斷工具與 validator。實測診斷結果顯示 shared AI root、ComfyUI checkout、stable-diffusion-webui checkout、GPU 與 Lua fallback 可用，但目前缺 Python `torch` 與 Blender。ComfyUI 也尚未建立 Linux 專用 venv。

這個 change 的目標是把診斷工具推進到可準備 toolchain，並用 user-local cache 建立 Ubuntu 執行環境。所有大型 shared assets 繼續留在 `/media/damody/新增磁碟區/AI_Pic`，Windows portable runtime 不修改。

## Goals / Non-Goals

**Goals:**

- 建立 Linux 專用 Python venv。
- 安裝並驗證 CUDA PyTorch。
- 讓 ComfyUI 能在 Linux venv 中完成 basic import 或 HTTP smoke。
- 配置 Blender Linux executable，通過 background Python smoke。
- 更新 bootstrap script，支援 `--prepare` 或等效流程。
- 產生一個非 fixture smoke package 並通過 validator。

**Non-Goals:**

- 不要求完成長時間生圖、3D 生成、rigging 或動畫生成。
- 不修改 Windows `venv/Scripts`、`python_embeded`、models、workflow 或 outputs。
- 不需要 sudo；若系統套件缺失，優先使用 user-local cache。
- 不把 smoke package import 到 `omb/`、`omfx/` 或 `scripts/base_content/`。

## Decisions

### Decision: user-local cache only

所有新建執行環境與下載工具放在 `~/.cache/omoba-character-pipeline/`。這避免 sudo，也避免污染 shared NTFS 根目錄。

### Decision: ComfyUI 使用 shared checkout + Linux venv

ComfyUI 原始碼仍使用 `/media/damody/新增磁碟區/AI_Pic/ComfyUI/ComfyUI`，但 Python dependencies 裝在 Linux venv。這讓 Windows portable 版仍可使用同一份 checkout/models。

### Decision: PyTorch 先以 pip wheel 安裝

優先在 Linux venv 內安裝 PyTorch CUDA wheel，驗證 `torch.cuda.is_available()`。不把 PyTorch 裝到系統 Python。

### Decision: Blender 以 PATH 或 user-local binary

先找 PATH 上的 `blender`。若沒有，下載或放置 user-local Blender binary 到 cache，bootstrap 記錄 executable path。

### Decision: smoke package 不產生大型資產

smoke package 證明 skill/package/validator 可運作即可。資產 manifest 仍是 `planned`，避免把長 GPU job 納入本 change。

## Risks / Trade-offs

- [Risk] PyTorch 對 RTX 5090 / driver / Python 3.12 的 wheel 相容性可能失敗。→ Mitigation：安裝在獨立 venv，可重建；失敗時保留診斷輸出。
- [Risk] ComfyUI dependencies 安裝時間長或有 native package 問題。→ Mitigation：先要求 import/API smoke，不要求完整 workflow 生圖。
- [Risk] Blender 下載檔大。→ Mitigation：若 PATH 已有 Blender 就跳過；否則 user-local cache 下載一次。
- [Risk] shared NTFS checkout 上執行 Python 可能遇到權限/路徑問題。→ Mitigation：venv 與 cache 放 Linux home，shared root 只當 source/assets。

## Migration Plan

1. 擴充 bootstrap script 的準備流程與設定輸出。
2. 建立 Linux venv 並安裝 PyTorch/ComfyUI dependencies。
3. 配置 Blender executable。
4. 跑 bootstrap verify，確認剩餘 blocker。
5. 新增 smoke package 並跑 validator。
