# LAN 選角席位與邀請加入

2026-10-06後續更新：Rust現在向已准入玩家分發同一immutable final plan；遠端選角會以compiled工具再驗後保存OUTPUT/match-plan.json，不再需要下述第二次主機配方傳檔。兩個明確選角／遊戲步驟仍保留；詳見2026-10-06-selection-final-plan-distribution-progress.md。此檔其餘內容保留初批54組歷史證據，不能當新增配方分發的實機驗收。

## 實際問題與架構決定

原始Rust `moba-config --selection-host`已支援具體bind、每席位private invitation與同一SelectionRoom，但Lua shared入口只接受loopback，且忽略local-player為所有真人開本機Unreal。正式遊戲connect已有，不能把重做connect當推進。

新增通用 `moba_selection_placement.lua`，host只建立本機requested人類席位的renderer；空requested維持全部本機語意。unknown／duplicate／非法人類ID及沒有本機席位拒絕；完整roster仍由同一Rust房間管理。這個interactive host至少一個本機真人，dedicated server-only不在此模式中。

新增 `--selection-bind IPv4`，須interactive-selection，不能remote gameplay connect；單一真人加此旗標也走共享房間。預設仍loopback，明確bind才使用Rust的ephemeral TCP port，ready地址必須同interface及canonical port，與正式遊戲的server-bind分開。

新增 `moba_selection_join.lua`／`run_moba_selection_join.lua`：遠端使用自己的private invitation啟動原生選角。只有一個原始owned renderer，不建立Rust gameplay host／client runtime；compiled-profile moba-config proxy及既有native UI繼續處理token、owner／catalog與shared-room protocol。join驗證terminal receipt，之後仍使用host發布的final match-plan加入正式遊戲，不从follower回條造gameplay配方或替他人鎖定。

## 使用流程

主機的配方需宣告所有真人席位。以主repo為工作目錄：

```text
D:\code\omoba\tools\lua\lua.exe scripts\run_moba_role_ue.lua --interactive-selection --recipe <配方JSON> --selection-bind <主機IPv4> --server-bind <主機IPv4> --local-player 1 --output <新的主機輸出目錄>
```

服務ready後入口列出`<主機輸出目錄>-selection/host`。只將`player-6.json`交給玩家6，不能分享整個folder或其他玩家檔案。遠端已建置相同profile的moba-config／Unreal後：

```text
D:\code\omoba\tools\lua\lua.exe scripts\run_moba_selection_join.lua --invite <玩家6邀請JSON> --player 6 --output <新的遠端選角輸出目錄>
```

所有真人完成選角、Rust finalized artifact及本機terminal receipt核對後，主機沿原workflow自動開始正式60Hz對局。把主機正式`match-plan.json`交給遠端；遠端再使用既有入口：

```text
D:\code\omoba\tools\lua\lua.exe scripts\run_moba_role_ue.lua --connect <主機IPv4> --recipe <主機match-plan.json> --local-player 6 --output <新的遠端對局輸出目錄>
```

目前遠端選角到gameplay是兩個明確步驟，不宣稱全自動遠端交接或discovery lobby。取消／逾時不自動lock、不停止remote host；host仍需全部人类鎖定，不把metadata预檢當真人同意。選角與遊戲需要各自TCP／KCP可達，沒有修改防火牆。邀請是bearer credential，既有Rust TCP未加TLS；僅作受信任LAN工作流，不宣稱Internet安全服務。

## 局部確認與界線

固定Lua placement7、join9、shared25、single13共54組通過：defaultalllocal、subset只一視窗完整十人roster、explicit LAN bind參數／wrongready拒絕、私人metadata／parser錯誤不洩token、完成／取消／timeout／wrongowner／badprotocol、PID reused有／無terminal receipt、其餘shared原回歸。

只有純table／injected renderer測試，未啟動真TCP host、遠端網路、Unreal、game或simulation，未跑Cargo／UBT。沒有變更Rust／C++／Blueprint／Lua runtime；已存在的native UI／Rust proxy仍需最後兩台實機串接驗收。65組上一批結果與本批54組不可混稱新的整套驗收。

Grok job `run-muwhr9w2-0c8cqj`／thread `dbcb543e-12bb-442a-b32f-8b78f4eb5d8b` 使用bridge支持的per-child GROK_BINARY_ARGS_JSON限制工具，完整context要求回補丁，4分9秒无結果取消。terminal／three tracked PID null／原84364与13104不存在確認後primary完成；不能聲稱Grok產碼，也不能定因filetool。成本/API時間未知。依Grok委派技能保留有界工作、original retirement與獨立審閱；依OpenSpec保留完整驗收checkbox。

E333記fixture錯誤及上述決策。HEAD02757保持、原dirty與userassets保留、未commit／push／omfx維護，整體25/31，完整4.1／4.3／4.4／6.2／6.4／6.5仍留最後。
