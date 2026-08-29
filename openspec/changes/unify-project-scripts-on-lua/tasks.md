# Lua 腳本統一實作任務

執行規則：依章節順序完成；每個子項只處理一個小責任，讓 5.6 Terra 可直接理解與修改。第 15 章以前不要跑完整測試，只做必要的檔案存在性與靜態確認；完整測試全部集中在最後。

## 1. 建立遷移清冊與契約

- [x] 1.1 將受版控的 `.bat`、`.ps1`、`.py`、`.sh` 路徑記錄到 migration manifest
- [x] 1.2 為每支舊腳本記錄 CLI 參數
- [x] 1.3 為每支舊腳本記錄讀取與寫入的環境變數
- [x] 1.4 為每支舊腳本記錄輸出檔案與 stdout/stderr 格式
- [x] 1.5 為每支舊腳本記錄成功及失敗 exit code
- [x] 1.6 為每支舊腳本記錄啟動的子程序
- [x] 1.7 為每支舊腳本記錄停止與失敗清理順序
- [x] 1.8 標記四個要保留的根目錄 `.bat`
- [x] 1.9 標記歷史文件與現行可執行文件的引用差異

## 2. 建立 Lua module 基礎

- [x] 2.1 建立 `tools/lua/lib/` 目錄與 module 命名規則
- [x] 2.2 建立 repository root 探測函式
- [x] 2.3 建立固定 Lua executable 路徑常數
- [x] 2.4 建立一致的 `main()` 錯誤包裝器
- [x] 2.5 建立一致的非零 exit code 回傳方式
- [x] 2.6 建立 Lua 測試 harness 入口
- [x] 2.7 建立 Windows 與 Linux 平台辨識函式

## 3. 實作參數與路徑模組

- [x] 3.1 在 `args.lua` 實作 positional argument 解析
- [x] 3.2 在 `args.lua` 實作 `--name value` 解析
- [x] 3.3 在 `args.lua` 實作 boolean flag 解析
- [x] 3.4 在 `args.lua` 實作整數與範圍驗證
- [x] 3.5 在 `args.lua` 實作 allowed-values 驗證
- [x] 3.6 在 `path.lua` 實作 Windows separator 正規化
- [x] 3.7 在 `path.lua` 實作 join 與 absolute path
- [x] 3.8 在 `path.lua` 實作 file／directory exists
- [x] 3.9 在 `path.lua` 實作安全的 parent directory 建立
- [x] 3.10 在 `path.lua` 實作檔案 timestamp 與 size 查詢
- [x] 3.11 在 `path.lua` 實作不覆寫寫入模式
- [x] 3.12 在 `path.lua` 實作 Windows command argument quoting

## 4. 實作 JSON、時間與 hash 模組

- [x] 4.1 在 `json.lua` 實作 null、boolean、number 與 string decode
- [x] 4.2 在 `json.lua` 實作 array 與 object decode
- [x] 4.3 在 `json.lua` 拒絕 trailing garbage 與 malformed escape
- [x] 4.4 在 `json.lua` 實作 deterministic object encode
- [x] 4.5 在 `json.lua` 實作 JSONL append
- [x] 4.6 在 `time.lua` 實作 UTC timestamp
- [x] 4.7 在 `time.lua` 實作 millisecond deadline 與 polling
- [x] 4.8 在 `hash.lua` 實作檔案 SHA-256 呼叫
- [x] 4.9 在 `hash.lua` 驗證 SHA-256 為 64 位 hexadecimal

## 5. 實作 process 與原生 helper

- [x] 5.1 定義 Rust helper 的 versioned JSON request schema
- [x] 5.2 定義 Rust helper 的 success／error response schema
- [x] 5.3 實作同步 command 執行 operation
- [x] 5.4 實作背景 process 啟動 operation
- [x] 5.5 實作 PID 存活查詢 operation
- [x] 5.6 實作 PID executable path 查詢 operation
- [x] 5.7 實作 PID 與預期 executable 雙重驗證
- [x] 5.8 實作單一 PID graceful stop operation
- [x] 5.9 實作 timeout 後的單一 PID forced stop operation
- [x] 5.10 實作 UDP datagram send operation
- [x] 5.11 實作 process memory dump 所需 operation
- [x] 5.12 在 `process.lua` 包裝同步 command
- [x] 5.13 在 `process.lua` 包裝背景 process 與 PID
- [x] 5.14 在 `process.lua` 實作 bounded wait
- [x] 5.15 在 `process.lua` 實作反向順序 cleanup stack
- [x] 5.16 在 `udp.lua` 包裝 helper 的 UDP operation
- [x] 5.17 讓 Rust helper 直接寫 response file，避免背景 child 繼承 shell redirect handle

## 6. 實作 evidence 共用模組

- [x] 6.1 在 `evidence.lua` 實作 run directory 不覆寫建立
- [x] 6.2 在 `evidence.lua` 實作 manifest JSON 寫入
- [x] 6.3 在 `evidence.lua` 實作 timeline JSONL 追加
- [x] 6.4 在 `evidence.lua` 實作 verdict PASS／FAIL／UNVERIFIED
- [x] 6.5 在 `evidence.lua` 實作 PID 與 executable identity 欄位
- [x] 6.6 在 `evidence.lua` 實作檔案 hash 欄位
- [x] 6.7 在 `evidence.lua` 實作 20-bin 權重驗證
- [x] 6.8 在 `evidence.lua` 實作 evidence read helper

## 7. 轉換低依賴 generator 與 validator

- [x] 7.1 將 `docs/tools/gen_stat_keys.py` 移植為 Lua
- [x] 7.2 將 `docs/tools/migrate_sk_callers.py` 移植為 Lua
- [x] 7.3 將 `scripts/gen_stress_map.py` 移植為 Lua
- [x] 7.4 將 `scripts/validate_td_map_bounds.ps1` 移植為 Lua
- [x] 7.5 將 `scripts/dev_run_freshness.ps1` 移植為 Lua
- [x] 7.6 保留 generator 的 deterministic output ordering
- [x] 7.7 保留 validator 的原 exit code 與錯誤文字語意

## 8. 轉換 selective-lockstep 工具

- [x] 8.1 將 `tools/selective_lockstep/common.py` 移植為 Lua module
- [x] 8.2 將 `network_fault_injection.py` 移植為 Lua
- [x] 8.3 將 `observer_slowdown.py` 移植為 Lua
- [x] 8.4 將 `packet_capture_scan.py` 移植為 Lua
- [x] 8.5 將 `paired_world_fixture.py` 移植為 Lua
- [x] 8.6 將 `redaction_scan.py` 移植為 Lua
- [x] 8.7 將 `stress_report.py` 移植為 Lua
- [x] 8.8 保留 selective-lockstep record schema 與 generation 規則
- [x] 8.9 保留 sentinel 與 fail-closed 判定

## 9. 轉換 character pipeline 工具

- [x] 9.1 將 `docs/character_pipeline/tools/bootstrap.py` 移植為 Lua
- [x] 9.2 將其餘 character pipeline 受版控 Python scripts 逐支移植為 Lua
- [x] 9.3 保留 shared AI root 唯讀安全檢查
- [x] 9.4 保留 user-local cache root 行為
- [x] 9.5 保留 venv Python executable 明確選擇
- [x] 9.6 保留 PyTorch CUDA probe
- [x] 9.7 保留 ComfyUI smoke
- [x] 9.8 保留 Blender background smoke
- [x] 9.9 保留 package schema validator
- [x] 9.10 保留 smoke package 不產生大型 asset 的限制

## 10. 轉換 process 與 fog helper

- [x] 10.1 將 `scripts/start_backend.ps1` 移植為 Lua
- [x] 10.2 將 `scripts/start_client_runtime.ps1` 移植為 Lua
- [x] 10.3 將 `scripts/start_fog_demo_frontend.ps1` 移植為 Lua
- [x] 10.4 將 `scripts/dump_process_memory.ps1` 移植為 Lua
- [x] 10.5 將 `scripts/dump_process_memory_linux.sh` 行為合併到 Lua
- [x] 10.6 將 `scripts/capture_fog_screenshots.ps1` 移植為 Lua
- [x] 10.7 將 `scripts/run_fog_lifecycle.ps1` 移植為 Lua
- [x] 10.8 將 `scripts/write_fog_run_manifest.ps1` 移植為 Lua
- [x] 10.9 將 `scripts/compare_fog_evidence.ps1` 移植為 Lua
- [x] 10.10 保留 team evidence 分流與 disclosure monotonic 判定
- [x] 10.11 保留 hidden target rejection 與 rebase recovery 判定
- [x] 10.12 保留 screenshot hash 與 renderer restart isolation 判定

## 11. 轉換 netem helper 與 scenario

- [x] 11.1 將 `scripts/start_netem_proxy.ps1` 移植為 Lua
- [x] 11.2 將 `scripts/stop_netem_proxy.ps1` 移植為 Lua
- [x] 11.3 將 `scripts/send_netem_control.ps1` 移植為 Lua
- [x] 11.4 將 `scripts/run_client_delay_scenario.ps1` 移植為 Lua
- [x] 11.5 將 `scripts/run_client_delay_matrix.ps1` 移植為 Lua
- [x] 11.6 保留 ordered-delay 與 natural-reorder 選項
- [x] 11.7 保留 Team 1／Team 2 獨立 profile
- [x] 11.8 保留 custom 20-bin JSON 載入
- [x] 11.9 保留 soak profile switch 與 authoritative tick
- [x] 11.10 保留 run ID 不覆寫規則
- [x] 11.11 保留 proxy PID 與 graceful shutdown

## 12. 轉換測試與特殊入口

- [x] 12.1 將 `scripts/test_run_session_launcher.ps1` 移植為 Lua
- [x] 12.2 將 `scripts/test_td_1_to_100.bat` 移植為 Lua
- [x] 12.3 將 `run_lives1.bat` 行為移到 `scripts/` Lua 入口
- [x] 12.4 將 `run_sandbox.bat` 行為移到 `scripts/` Lua 入口
- [x] 12.5 保留 TD 1 到 100 的 build、執行與 exit code
- [x] 12.6 保留 session launcher 的 PID scope assertions

## 13. 轉換四個主要 launcher

- [x] 13.1 將 `run.bat` 的實際流程移植到 Lua
- [x] 13.2 將 `run_10000.bat` 的實際流程移植到 Lua
- [x] 13.3 將 `run_2player.bat` 的實際流程移植到 Lua
- [x] 13.4 將 `run_ue.bat` 的實際流程移植到 Lua
- [x] 13.5 在一般 launcher 保留 debug runtime Lua content mode
- [x] 13.6 在一般 launcher 保留 hot reload mode
- [x] 13.7 在高負載 launcher 保留 release artifact profile
- [x] 13.8 在高負載 launcher 保留 game configuration restore
- [x] 13.9 在雙玩家 launcher保留 direct 與 netem 模式
- [x] 13.10 在雙玩家 launcher 保留 headless 與 visual 模式
- [x] 13.11 在雙玩家 launcher保留反向停止順序
- [x] 13.12 在 Unreal launcher 保留既有 frontend 環境與參數
- [x] 13.13 將 `run.bat` 縮減成固定 Lua wrapper
- [x] 13.14 將 `run_10000.bat` 縮減成固定 Lua wrapper
- [x] 13.15 將 `run_2player.bat` 縮減成固定 Lua wrapper
- [x] 13.16 將 `run_ue.bat` 縮減成固定 Lua wrapper
- [x] 13.17 確保四個 `.bat` 原樣轉送 `%*`
- [x] 13.18 確保四個 `.bat` 回傳 Lua exit code
- [x] 13.19 將四個 `.bat` 正規化為 CRLF UTF-8 無 BOM

## 14. 更新呼叫端、文件並移除舊腳本

- [x] 14.1 更新 Rust source 中的舊腳本路徑
- [x] 14.2 更新 Cargo test 或 build metadata 中的舊腳本路徑
- [x] 14.3 更新現行 OpenSpec specs 中的 launcher 名稱與 Lua 指令
- [x] 14.4 更新非 archive 的現行操作文件
- [x] 14.5 更新 `AGENTS.md` 的 Lua runtime 與四個 wrapper 規則
- [x] 14.6 更新 README 類文件的操作範例
- [x] 14.7 更新 CI 或 repository automation 的腳本路徑
- [x] 14.8 移除已替換的 `.ps1`
- [x] 14.9 移除已替換的 `.py`
- [x] 14.10 移除已替換的 `.sh`
- [x] 14.11 移除 `run_lives1.bat` 與 `run_sandbox.bat`
- [x] 14.12 移除 `scripts/test_td_1_to_100.bat`
- [x] 14.13 確認 `.gitignore` 不會忽略新的 Lua source
- [x] 14.14 確認沒有修改與本 change 無關的 user worktree 內容
- [x] 14.15 將 repository 內 character-pipeline skill 的舊 Python 指令改為固定 Lua 指令
- [x] 14.16 讓 bundled `lua.exe`、`lua54.dll` 與 `lfs.dll` 可納入版控並更新 provenance
- [x] 14.17 用 `.gitattributes` 固定四個 Batch wrapper 的 CRLF 行尾
- [x] 14.18 將 dump、PDB 與 trace 類可重建原生產物排除在可提交集合之外
- [x] 14.19 將尚未封存的 TD autoplay OpenSpec 從已移除 Batch 入口更新為固定 Lua 入口

## 15. 最後執行 Lua 靜態與模組測試

- [x] 15.1 對每個受版控 `.lua` 執行 `loadfile` syntax check
- [x] 15.2 執行 args module tests
- [x] 15.3 執行 path 與 Windows quoting tests
- [x] 15.4 執行 JSON valid／invalid fixture tests
- [x] 15.5 執行 process helper protocol tests
- [x] 15.6 執行 PID executable identity tests
- [x] 15.7 執行 UDP loopback tests
- [x] 15.8 執行 SHA-256 pin test
- [x] 15.9 執行 evidence 不覆寫與 JSONL tests
- [x] 15.10 掃描確認只剩四個允許的 `.bat`
- [x] 15.11 掃描確認沒有受版控 `.ps1`、`.py` 或 `.sh`
- [x] 15.12 檢查四個 `.bat` 只有 wrapper 行為
- [x] 15.13 檢查四個 `.bat` CRLF 與 UTF-8 無 BOM
- [x] 15.14 掃描現行 source、skill 與操作文件沒有引用已移除的舊腳本
- [x] 15.15 驗證 bundled Lua runtime 未被 ignore 且三個檔案 SHA-256 符合 provenance
- [x] 15.16 用永久回歸測試逐一檢查四個根目錄 Batch wrapper
- [x] 15.17 用 helper child 驗證錯誤 executable identity 不會終止其他程序
- [x] 15.18 確保 Lua module test 成功或失敗時都會清理自己的暫存目錄
- [x] 15.19 在 PATH 沒有 Lua 時實際驗證根目錄 wrapper 仍呼叫固定 runtime
- [x] 15.20 驗證 spawn、PID guard 與停止流程不留下 host request／response 暫存檔
- [x] 15.21 用永久回歸測試掃描 active OpenSpec 不得引用已移除的工作流腳本

## 16. 最後執行工具與 launcher 測試

- [x] 16.1 執行 stat key generator golden test
- [x] 16.2 執行 SK caller migration fixture test
- [x] 16.3 執行 stress map generator deterministic test
- [x] 16.4 執行 TD map bounds validator tests
- [x] 16.5 執行 selective-lockstep 全部 fixture tests
- [x] 16.6 執行 character pipeline validator smoke
- [x] 16.7 執行 character pipeline toolchain read-only diagnostics
- [x] 16.8 執行 freshness helper fresh／stale／error tests
- [x] 16.9 執行 session launcher PID scope tests
- [x] 16.10 執行 TD 1 到 100 自動測試入口
- [x] 16.11 執行一般 launcher bounded smoke
- [x] 16.12 執行高負載 launcher bounded smoke 並確認設定還原
- [x] 16.13 執行 Unreal launcher argument smoke

## 17. 最後執行 netem、fog 與回歸測試

- [x] 17.1 執行 custom 20-bin profile smoke
- [x] 17.2 執行 ordered-delay smoke
- [x] 17.3 執行 natural-reorder smoke
- [x] 17.4 執行 runtime profile control switch
- [x] 17.5 執行 server + Team 1 + Team 2 三程序測試
- [x] 17.6 驗證兩隊 replica 不含視野外資訊
- [x] 17.7 驗證英雄移動與 server authoritative correction
- [x] 17.8 驗證 Reveal、Hide 與 Forget
- [x] 17.9 驗證 hidden target rejection
- [x] 17.10 驗證斷線、rebase 與 replay recovery
- [x] 17.11 執行兩 renderer visual isolation smoke
- [x] 17.12 執行受影響 Rust helper crate tests
- [x] 17.13 執行 `omoba-netem-proxy` tests
- [x] 17.14 執行 `omoba-client-runtime` tests
- [x] 17.15 執行 `omoba-core` tests
- [x] 17.16 執行 `omb` library tests
- [x] 17.17 執行 `omfx` tests
- [x] 17.18 執行 `openspec validate unify-project-scripts-on-lua --strict`
- [x] 17.19 執行 root、`omb` 與 `omfx` 的 `git diff --check`
- [x] 17.20 清理可重建的測試與 build 暫存產物
- [x] 17.21 確認 tasks 全部完成且沒有未勾選項目
- [x] 17.22 清除本次 `lua-migration-*` 測試中沒有 PASS verdict 的失敗與未完成 evidence run
- [x] 17.23 清除本次 PASS evidence 中可重建且禁止提交的 memory dump
