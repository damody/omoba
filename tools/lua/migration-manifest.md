# Script migration manifest

固定 runtime：`D:\code\omoba\tools\lua\lua.exe`（Lua 5.4.8）。下表是 root repository
本次變更的完整腳本契約清冊；`—` 表示沒有該類輸入或副作用。所有 replacement 成功回傳
`0`，參數、驗證、外部命令或必要產物失敗皆回傳非零；背景程序一律記錄 PID 與 executable
identity，並以相反依賴順序清理，不使用全域同名終止。

| 舊入口 → Lua replacement | CLI | 環境變數 | 產物與輸出 | 子程序／清理 |
|---|---|---|---|---|
| `run.bat` → `scripts/run.lua` | `--autoplay-100`, `--trace`, `--build-only` | `OMOBA_BUILD_PROFILE`, trace 設定；寫入既有 OMB/OMFX runtime env | Cargo artifact、staged DLL；fresh/stale/build 訊息 | Cargo 同步；非 build-only 時前景 executor |
| `run_10000.bat` → `scripts/run_10000.lua` | 原高負載參數 | OMB/OMFX 高負載與 release env | release artifacts、暫換 `game.toml` | Cargo/executor；任何結束均還原設定 |
| `run_2player.bat` → `scripts/run_2player.lua` | `headless\|visual` | `OMOBA_*` port、netem、run ID、lifecycle、test flags | team 分流 evidence、PID、dump、screenshot、verdict | server→proxy→兩 runtime→兩 renderer；反向 PID-safe cleanup |
| `run_ue.bat` → `scripts/run_ue.lua` | build/editor/game/headless/network flags | Unreal frontend/backend env | Unreal build/staging 與 smoke 輸出 | Cargo/Unreal/backend；session-scoped cleanup |
| `run_lives1.bat` → `scripts/run_lives1.lua` | 原 positional args | lives1 dev env | 原 stdout/stderr | 前景 executor |
| `run_sandbox.bat` → `scripts/run_sandbox.lua` | 原 positional args | sandbox env | 原 stdout/stderr | 前景 executor |
| `scripts/test_td_1_to_100.bat` → `.lua` | — | Cargo/TD test env | Cargo test output | 同步 Cargo，直接傳回 exit code |
| `capture_fog_screenshots.ps1` → `.lua` | evidence dir、兩 renderer PID | — | 兩 PNG 與 hash | 驗證 PID identity 後觸發；不終止 renderer |
| `compare_fog_evidence.ps1` → `.lua` | evidence dir | lifecycle gate flag | checkpoint comparison、`verdict.json`、JSON stdout | — |
| `dev_run_freshness.ps1` → `.lua` | action、artifact、profile、fixture paths | — | fresh/stale 訊息或 staged DLL | 必要時檔案 copy；不啟動程序 |
| `dump_process_memory.ps1`／`dump_process_memory_linux.sh` → `.lua` | PID、expected exe、output、role | — | dump 與 JSON metadata | Rust helper PID-safe dump；不終止目標 |
| `run_fog_lifecycle.ps1` → `.lua` | server/runtime/renderer PID、route、presentation、evidence | — | `lifecycle.json`、restart PID/evidence | Team 1 renderer/runtime 斷線重連；保持 server/Team 2；最後清理重啟程序 |
| `start_backend.ps1` → `.lua` | exe、cwd、pid/evidence/ready args | OMB server env | PID、redirected stdout/stderr | 啟動 server；ready timeout 時只清理該 PID |
| `start_client_runtime.ps1` → `.lua` | identity、team、route、presentation、evidence | fault probe | PID、runtime logs/evidence | 啟動單一 team runtime；ready timeout PID-safe cleanup |
| `start_fog_demo_frontend.ps1` → `.lua` | identity、team、window、presentation | renderer-only OMB/OMFX env | PID、renderer logs | 啟動單一 renderer；ready timeout PID-safe cleanup |
| `test_run_session_launcher.ps1` → `.lua` | — | — | assertion stdout | helper child；測試後清理 |
| `validate_td_map_bounds.ps1` → `.lua` | map path／fixture | — | 原 validator 訊息 | — |
| `write_fog_run_manifest.ps1` → `.lua` | process、route、mode、profile metadata | — | `manifest.json`（拒絕覆寫） | — |
| `gen_stress_map.py` → `.lua` | output/grid/unit options | — | deterministic Lua map | — |
| `docs/tools/gen_stat_keys.py` → `.lua` | ABI input、output | — | deterministic stat-key catalog | — |
| `docs/tools/migrate_sk_callers.py` → `.lua` | stat keys、source root、dry-run | — | sorted migration report／source edits | — |
| `docs/character_pipeline/tools/bootstrap.py` → `.lua` | diagnostic/prepare/package options | local cache、shared AI/tool paths | diagnostic JSON、small smoke package | 明確 venv Python/ComfyUI/Blender；shared root 唯讀；不清理第三方安裝 |
| `tools/selective_lockstep/common.py` → `.lua` module | module API | — | 共用 deterministic record schema | — |
| `network_fault_injection.py` → `.lua` | fixture、output、seed | — | fault evidence JSONL | — |
| `observer_slowdown.py` → `.lua` | fixture、output、delay | — | slowdown evidence JSONL | — |
| `packet_capture_scan.py` → `.lua` | capture、sentinel、output | — | fail-closed scan verdict | — |
| `paired_world_fixture.py` → `.lua` | output、seed | — | deterministic paired-world fixture | — |
| `redaction_scan.py` → `.lua` | evidence、sentinel | — | fail-closed redaction verdict | — |
| `stress_report.py` → `.lua` | fixture/evidence、output | — | deterministic stress report | — |
| `start_netem_proxy.ps1` → `.lua` | exe、binds、profiles、seed、evidence | — | PID、proxy evidence/logs | 啟動 proxy；ready timeout PID-safe cleanup |
| `stop_netem_proxy.ps1` → `.lua` | PID、expected exe、control addr | — | shutdown 訊息 | UDP graceful shutdown，timeout 後只強停已驗證 PID |
| `send_netem_control.ps1` → `.lua` | address、team、profile/20 bins、tick | — | control result | Rust helper UDP send |
| `run_client_delay_scenario.ps1` → `.lua` | mode、profiles、duration、run ID | `OMOBA_NETEM_*` | scenario evidence/verdict | 呼叫雙玩家 launcher；反向 cleanup |
| `run_client_delay_matrix.ps1` → `.lua` | modes/profiles/runs | `OMOBA_NETEM_*` | matrix runs/summary | 逐 scenario 執行，傳回首個失敗 |

四個保留的 root `.bat` 僅定位 root、呼叫固定 Lua、原樣轉送 `%*` 並回傳
`%ERRORLEVEL%`。其餘舊入口已移除。archive 設計文件保留歷史指令；現行 source、automation、
operator docs 與非 archive spec 必須只引用 Lua 入口。

`omb` 與 `omfx` 是獨立 Git submodule，不是 root repository 的受版控檔案。其各自版本化的
`omb/build.sh`、`omb/install_linkers.sh`、`omb/examples/mqtt_test_client.py` 與
`omfx/game/data/sfx/gen_sfx.py` 已在此揭露，但不屬於本 change 的 root workflow 刪除範圍。
