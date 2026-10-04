# Unreal 非零死亡計分板 60Hz 驗收

## 計畫與決策

1. 只在明確 opt-in 的正式雙UE單路60Hz測試中，等待真實死亡與非零死亡數後保存原生計分板。
2. 沿用正式四技能、死亡／重生與基地勝負流程；不注入傷害、戰績或修改timeout。
3. 核對原始authority wire、IPC快照及UI紀錄，驗證死亡後戰績保留至結算與程序清理。

- `OMOBA_UE_SCOREBOARD_DEATH_SMOKE=1` 必須同時啟用初始計分板及完整match smoke；一般互動啟動無新增測試輸入或截圖。
- 共用Widget只在Playing、owner死亡、有效戰績且自身Deaths>0、viewport面板可見與geometry非零時截圖。初始request立即return，兩者獨立PNG，避免同frame覆寫。沒有英雄專用C++或Blueprint graph。
- Lua觀測器驗固定1v1 fixture名單，死亡gate要求自己非零死亡；正式runtime仍支援兩隊最多十人。
- Rust capture verifier逐snapshot比較公開五欄，snapshot tick明確減1對authority。死亡UI tick還必須HUD無存活hero，Finished必須仍保留自身死亡數。

## 已驗證

- Unreal build-only編譯成功；Lua觀測器9初始＋4死亡＋31既有情境通過，launcher語法通過；runtime lib 59 passed／6外部capture ignored。
- 正式release run `target/interactive-runs/interactive-ue-1791080545`：success／cleanup_verified皆true，tick_rate_hz=60，雙隊4/4技能、正式objective AttackTarget確認套用；死亡6／7次、重生6／6次，Finished10028，winner1，結果UI10029。
- 初始UI tick951／1256；死亡UI tick3307／3929。team1死亡板P1=0/1/0、P2=1/0/0；team2死亡板P1=0/1/0、P2=1/1/0。本次確有非零玩家擊殺列，不以NPC死亡替代kill；未驗非零assist畫面。
- `real_unreal_scoreboard_capture` opt-in另行執行成功：10289＋10273共20,562筆IPC，逐tick與authority board完全一致，兩隊相同tick board一致，初始／死亡UI五欄精確一致，戰績保留至Finished。
- 三方hash169／167 PASS rows至10200，晚於UI／終局120 ticks。不是169個連續ticks或完整效能測試。
- 兩張初始、兩張死亡與兩張結果1280×720 PNG已實際檢視；死亡HUD顯示Respawning與非零KDA，結算Victory／Defeat並同時保留P1=0/6/0、P2=1/7/0。截圖顯示FPS60，但不宣稱整段穩定60FPS。
- report五程序84916／5576／36436／72564／91932再以Lua process.inspect獨立確認退出。

## 限制與後續

截圖是request後的viewport draw；UI log與IPC可精確對tick，PNG不具有GPU fence或同tick原子像素契約。實際畫面已確認非零戰績與死亡／終局狀態，未將此說成逐frame像素同步證明。

OpenSpec維持19/30；6.2選角到結算、十人多人與非零助攻畫面未完成，5.3 XP完整經濟／成長仍待驗證。錯誤與防重犯規則記E115。

## Editor與產物收尾

- full build成功，BpGeneratorUltimate MCP就緒；`compile-1791080873/report.json`的11個Blueprint全部編譯成功，未創建缺失資產。
- 同一Editor原生／legacy測試兩輪各15/15成功；隨後串行PIE smoke成功，沒有與automation重疊。這是共用元件相容性，不冒充多人遊戲實測。
- MCP保存dirty assets成功，正常關閉本輪Editor5380並獨立inspect確認退出；六個本輪owned程序全部退出，無殘留測試。
- 最後build-only與stage核對成功，bridge SHA-256 `d7dd01b823f7874b3383b304526b1615e08cc0140d4b04dab140701f0c35c012`；codegen --check的11檔／15Lua輸入一致，content hash `de9c7fcfc98d6479`。無角色專用C++／BP變更。
- OpenSpec strict validation與主repo／omfue scoped diff whitespace檢查通過；既有dirty變更保留，沒有commit／push／清理Unreal資產。
