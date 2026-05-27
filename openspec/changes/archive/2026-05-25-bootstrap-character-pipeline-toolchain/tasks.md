## 1. Bootstrap Prepare 流程

- [x] 1.1 擴充 `docs/character_pipeline/tools/bootstrap.py`，新增 prepare/verify 所需參數與 cache config。
- [x] 1.2 建立 user-local Linux venv，路徑不在 shared AI root 內。
- [x] 1.3 確認 prepare 流程不修改 Windows `venv/Scripts` 或 `python_embeded`。

## 2. PyTorch CUDA

- [x] 2.1 在 Linux venv 內安裝 PyTorch CUDA wheel。
- [x] 2.2 使用 venv Python 驗證 `import torch` 與 `torch.cuda.is_available()`。
- [x] 2.3 將 venv Python / torch 狀態納入 bootstrap 診斷輸出。

## 3. ComfyUI Linux Smoke

- [x] 3.1 使用 Linux venv 安裝或驗證 ComfyUI requirements。
- [x] 3.2 對 shared ComfyUI checkout 執行非長時間 smoke。
- [x] 3.3 確認 smoke 不使用或修改 Windows `python_embeded`。

## 4. Blender

- [x] 4.1 找出 PATH 或 user-local Blender executable。
- [x] 4.2 若 PATH 不存在，安裝或配置 user-local Blender。
- [x] 4.3 執行 Blender background Python smoke。

## 5. First Package Smoke

- [x] 5.1 新增一個非 `hero_example` 的 smoke package。
- [x] 5.2 使用 validator 驗證 smoke package 成功。
- [x] 5.3 執行 `openspec validate bootstrap-character-pipeline-toolchain --strict`。
