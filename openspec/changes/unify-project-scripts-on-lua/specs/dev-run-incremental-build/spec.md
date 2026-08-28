## MODIFIED Requirements

### Requirement: `run*.bat` skips fresh build steps

四個根目錄 `.bat` SHALL 只呼叫固定 `D:\code\omoba\tools\lua\lua.exe` 與對應 Lua launcher、轉送全部參數並回傳 exit code。Lua launcher SHALL 在對應 output artifact 存在，且沒有 configured relevant input 比該 artifact 更新時，skip script DLL、backend 與 frontend Cargo build steps。Debug launcher SHALL 使用 debug artifacts；高負載 launcher SHALL 使用 release artifacts。

Skipped frontend build SHALL 仍會從 repo root launch 已 build 的 `omfx/target/<profile>/executor.exe`。

#### Scenario: source 未變時第二次 launch 會 skip builds
- **WHEN** Lua launcher 成功執行一次，且沒有 relevant input files 變更
- **THEN** 下一次 invocation 時，script DLL、backend 與 frontend build artifacts 會回報 up-to-date
- **AND** launch frontend 前不會對這些 fresh artifacts invoke Cargo build work

#### Scenario: artifact missing 時會 rebuild
- **WHEN** 任一 required matching-profile output artifact missing
- **THEN** Lua launcher 將該 artifact 視為 stale
- **AND** 繼續前會 invoke matching Cargo build step

#### Scenario: 高負載 launcher 使用 release artifacts
- **WHEN** `run_10000.bat` 以 fresh release script DLL、backend 與 frontend 執行
- **THEN** 它回報這些 release artifacts up-to-date
- **AND** build pipeline 不使用 debug Cargo build steps

#### Scenario: Batch wrapper 只轉送到 Lua
- **WHEN** 靜態檢查四個根目錄 `.bat`
- **THEN** 每個檔案只定位 root、呼叫固定 Lua runtime、轉送 `%*` 與回傳 exit code
- **AND** 不包含 build、copy、process、network 或 evidence 分支

### Requirement: launcher-specific runtime behavior 保持不變

Lua incremental freshness checks SHALL NOT 改變 launcher-specific runtime setup。一般開發、高負載、本機雙玩家與 Unreal frontend 四種入口 SHALL 保留各自環境、artifact profile、process topology、auto-start／auto-exit、game configuration swap 與失敗清理行為。

#### Scenario: 一般開發 launcher 保留 Lua content mode
- **WHEN** `run.bat` 使用 fresh debug artifacts
- **THEN** Lua launcher skip 不必要建置
- **AND** 仍啟用 runtime Lua content 與 hot reload 的既有環境

#### Scenario: 高負載 launcher 失敗仍還原設定
- **WHEN** `run_10000.bat` 在 swap game configuration 後發生建置或執行失敗
- **THEN** Lua launcher 還原原始設定
- **AND** 回傳原始非零 exit code

#### Scenario: 雙玩家 launcher 保留安全拓撲
- **WHEN** `run_2player.bat` 啟動 headless 或 visual 模式
- **THEN** Lua launcher 建立 authoritative server、兩個獨立 team runtime，以及選配 proxy 與 renderer
- **AND** 停止時依相反依賴順序清理該 session 程序

