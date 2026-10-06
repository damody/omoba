# 純 Rust 專用主機角色

## 問題與決定

遠端 client 已能用 `--connect` 加入同一 host，因此不重做網路玩法。缺口是 host 工作流固定解析／建置 Unreal，而空 local-player 集合會選所有真人。

新增明確 `--server-only`，只改部署角色，不改選角、席位、權威規則、生成內容或協定。真人完整 roster 保留；本機 client／renderer 數為零，後端仍固定60Hz與 compiled-content-only，Lua不在遊戲runtime執行。

此角色拒絕 connect、本機玩家、互動選角與原生結算畫面擷取。需要換英雄時仍可用 recipe／既有 `--hero PLAYER_ID=HERO`，沿正式 Rust lock-plan 完成鎖定；不假裝遠端玩家已使用互動大廳同意。

## 實作

- `scripts/moba_server_build.lua`：同一 debug／release profile 建置 base_content、moba-config、omobab，再由固定Lua stage-dll部署。檢查两個authority exe存在，以及內建來源DLL／正式部署DLL SHA-256相符，不要求Unreal／bridge／client-runtime。
- 角色啟動／網路選項／工作流：`dedicated-server` 模式、零本機client、只呼叫server build／verify，prepare-only不啟動任何程序，no-build仍驗部署一致性。
- 程序生命週期：只spawn一個主機，記錄 executable與creation-token；等待原程序退出，不因零renderer立即殺主機。中途錯誤沿既有original-only清理與MD錯誤輸出，PID reuse不停止replacement。
- 舊remote測試fixture遷移至已存在的owned adapter，不減弱正式程序身分檢查。

## 使用

在主repo工作目錄使用固定Lua：

```text
D:\code\omoba\tools\lua\lua.exe scripts\run_moba_role_ue.lua --server-only --server-bind <主機實際IPv4> --profile release
```

預設仍綁loopback，區網需明確填入主機實際unicast IPv4。輸出目錄中的 `match-plan.json` 是供遠端入口使用的final recipe；遠端使用既有 `--connect <主機IPv4> --recipe <該JSON> --local-player <已宣告真人ID>`。未修改防火牆或安裝設定。

## 本批局部確認

固定Lua `test_moba_server_build.lua` 4組、`test_moba_dedicated_server.lua` 7組、`test_moba_network_launch.lua` 7組、`test_moba_role_launch.lua` 8組通過，共26組。包含真正Rust lock-plan／configuration-only預檢；build呼叫、檔案一致性與程序生命週期的測試是注入fixture，沒有執行真正Cargo全建置、對局模擬、client runtime或Unreal。

Grok job `run-muwh76hh-6y910g`、thread `5fef2d53-a4e0-4d7a-950e-f689b4ccf650` 2分18秒無程式產出後已取消；follower終止、三個tracked PID為null且兩個原程序不存在後，主agent接手實作。成本／API時間與卡住根因未知，不能把主agent程式歸功於Grok。依委派技能保留獨立diff審閱與局部確認，依OpenSpec保留完整验收界線。

錯誤與預防見中央紀錄E331。完整計畫仍25/31：不將本批配置／fixture通過冒充兩台實機LAN、完整UI自然終局或12項固定效能上限驗收；完整驗收留最後，未維護omfx、未提交或推送。
