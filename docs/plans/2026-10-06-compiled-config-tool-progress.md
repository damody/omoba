# 已編譯設定工具入口

## 問題與決定

角色工作流建置所選debug／release，prepare卻用不帶profile的cargo run重新編譯debug moba-config。這讓release、no-build與純Rust主機仍隱含依賴編譯工具鏈，和可部署的原生框架不一致。

決定共用 `scripts/moba_config_command.lua`：host／remote／dedicated與prepare-only、互動選角都直接執行指定profile的moba-config.exe。缺檔在建立輸出前拒絕，提示精確檔案及顯式Cargo建置指令，不自動建置、切profile或走Unreal。建置仍由既有明確build流程負責。

process結果必須exit_code==0才解析JSON；保留stdout／stderr失敗診斷，不把失敗stdout當有效鎖定。傳給process的參數是獨立副本，compiled-only環境固定關閉Lua內容／hot reload。互動選角與prepare使用相同政策，不為某角色特製處理。

`--prepare-only` 現在也需要指定profile的已建置工具。開發者可明確執行：

```text
cargo build --manifest-path omb/Cargo.toml -p omobab --bin moba-config --features compiled-content-only --release
```

debug則省略--release。這是顯式開發建置指引，正式啟動不會呼叫Cargo。沒有新增根目錄wrapper／PowerShell／Python fallback。

## 本批局部確認

固定Lua：config-command helper5、integration3、單人選角13、共享選角22、角色8、network7、dedicated7，共65組通過。integration真正執行現有debug／release moba-config的lock-plan與設定预檢；另以注入missing-tool確認沒有輸出mkdir或process呼叫。其他生命週期／renderer測試是注入fixture，不是對局或Unreal操作。

未執行Cargo建置、場次模擬、遊戲、Unreal或LAN完整驗收。HEAD02757保持，原dirty與使用者資產保留；未維護omfx、未commit／push。完整計畫仍25/31，六项整體驗收未冒勾。

## Grok 與錯誤

依Grok委派技能將新module／unit test交給job `run-muwhfuzt-dx1moz`、thread `84646d8f-5f9f-4e80-8c8f-95633f5c75b4`，primary只接現有入口，避免寫入重疊。Grok1分59秒無code後取消；follower terminal、tracked三PID null与原20828／60604不存在確認後primary接手。成本、API時間與卡住根因未知，主agent修改不歸功於Grok。

E332記錄隱含Cargo／profile問題、猜路徑及nested fixture bootstrap錯誤。OpenSpec流程將新契約寫入match spec及tasks；局部結果不取代兩台LAN、完整UI終局與固定12項效能驗收。
