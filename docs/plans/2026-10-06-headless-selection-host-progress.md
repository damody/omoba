# 無畫面選角主機

## 問題與通用決定

專用遊戲主機已不依賴 Unreal，但原共享選角工作流至少需要一個本機真人 renderer，無法只提供遠端選角房間。新增獨立的 run_moba_selection_host.lua，而不放寬既有 server-only 與 interactive-selection 互斥規則。

新入口只建 pre-match Rust 房間，所有真人保留為遠端席位；私人 Invitation／准入／lock／finalize 沿既有 moba-config 與 SelectionRoom，完成只發布 match-plan.json，不開 gameplay、不替玩家鎖定、不讀取或印出私人 token。Unreal／client runtime／腳本 DLL 不是這個選角入口的依賴。

共用 moba_selection_prepare 提供原互動選角與無畫面入口一致的作者配方載入、真人清單及指定 profile 原生 validator。Lua 僅在開局工具求值作者配方；Rust preflight --lock-plan 驗證的是副本，不是遠端房間中的玩家同意。

## 使用方式

需先明確建置 compiled-content-only 的 moba-config（預設 release）。例如：

```text
cargo build --manifest-path omb/Cargo.toml -p omobab --bin moba-config --features compiled-content-only --release
tools/lua/lua.exe scripts/run_moba_selection_host.lua --selection-bind <主機IPv4> --output <新的選角輸出目錄> --recipe <真人席位Lua或JSON配方>
```

選角工具會列出 OUTPUT/host，僅傳送每位真人自己的 player-ID.json，不分享整個目錄。玩家用既有 run_moba_selection_join.lua 加入；所有真人明確鎖定並 finalize 後，主機取得 OUTPUT/match-plan.json。可選 selection-timeout-seconds 1..7200，逾時只退役本次原始選角程序並記 errors.md，不自動同意或啟動遊戲。

接著使用既有專用主機流程明確開局：

```text
tools/lua/lua.exe scripts/run_moba_role_ue.lua --server-only --server-bind <主機IPv4> --recipe <選角輸出>/match-plan.json --output <新的遊戲輸出目錄>
```

遠端以自己的完成配方沿 --connect／--local-player 進入遊戲。選角房間與 gameplay 是兩個明確階段，不發明新的連線協定或自動服務發現。

## 局部確認

固定 Lua 執行下列局部指令成功：

```text
tools/lua/lua.exe scripts/tests/moba_selection_host_test.lua
tools/lua/lua.exe scripts/test_moba_hero_selection.lua
tools/lua/lua.exe scripts/test_moba_shared_selection.lua
tools/lua/lua.exe scripts/run_moba_selection_host.lua --help
```

新入口9組、原單人13組、共享25組，共47組。新入口使用真正的 compiled-content-only release moba-config --lock-plan 預檢，唯一 selection host process 與時間注入；確認成功只出一個host、不開renderer、不改作者配方，兩真人十席位完整保留，產生 selection-host-only result。timeout／host exit／wrong bind／roster tamper／retirement failure／spawn failure不發布 final recipe／result並留errors.md；existing output不修改。

入口與共享 room 的實作完成，沒有新增正式協定／Rust玩法或重建 Rust／UE。這不是完整 socket host/proxy／Unreal／雙實機LAN驗收；本輪沒有場次模擬。完整六項保持待驗收25/31，不以本機替身當實機證據。

## Grok 委派与獨立審查

依 Grok Build 委派技能提供單檔共享 room 抽取任務，primary 同時負責新入口／共同preflight／局部fixtures。job run-muwjqv3n-r3f1lo、thread6d137eeb-495c-41f3-b13c-ef8debade675在讀取階段246秒無修改，primary取消並確認follower terminal cancelled、三tracked PIDnull、OS原85836／100776不存在後完成run_room抽取。沒有接受Grok補丁或宣稱其產碼，成本／API時間／原因未知；沒有變更全域MCP／auth／Grok設定。primary檢查實際差異及所有新增來源，局部確認後接受此功能；HEAD02757保持，不commit／push，不操作omfx。防錯E337。
