# MOBA 實作／驗收分界清單（2026-10-06）

這份清單用來收斂既有10項未勾選任務，不取代OpenSpec完成條件。以下是主agent目前程式核對，仍須Grok實作回報及整合審查；不宣稱全功能完成或完整驗收成功。

| 未勾項 | 已有實作入口／依據 | 收斂條件 |
| --- | --- | --- |
| 2.2b 通用事件 | codegen通用模板、legacy metadata沿同一通用資料轉接；E280–283、C ABI16 | 確認保留顯式舊API不復活自動角色分支，最後生成／native回歸 |
| 4.1 IPC／安全投影 | client-runtime presentation_bridge RendererReady身分驗證及單renderer lease、protobuf IPC | 完整版本握手／正式輸入結果／跨隊隔離验收 |
| 4.3 重連／cue | presentation_bridge retained Ready、baseline、consume sequence及cue_retention | 最後renderer斷開／恢復、一次性效果不可重播 |
| 4.4 單一玩家模擬 | bridge validated_driver_mode於worker建立前解析Presentation／LocalTd／NetworkTd，禁止空endpoint fallback | 單機與LAN正式模式整合確認，不能藉測試啟動舊前端 |
| 5.4 地圖／導航／解鎖 | compiled三路map／camp、moba_match lane_tower_layers／base_unlocked、共用完整路徑導航 | 固定種子完整lifecycle與deadlock檢查 |
| 5.5 五位置／原型 | RoleBotMatchPlan compile、mixed moba_archetype_match公開配方、core共用bots；E294–297補齊旅行／追擊 | 100場正式60Hz headless，不能把局部fixtures算100場 |
| 6.1 畫面層 | native動畫／cue／fallback、frame准入、fog／terrain／route；E285–293 | 最後native assertions實際執行＋PIE畫面，而非僅編譯 |
| 6.2 對局UI | native選角、command bar商店／四技能／六物品／小地圖／scoreboard／result | Grok第1批owned HUD與入口失效實作整合後，最後PIE選角至結算 |
| 6.4 網路驗收 | strict catalog／frame ABI、client-runtime team/player驗證與重連路徑 | 兩台LAN真人同局與錯配／重連；本機多程序不冒充兩台 |
| 6.5 基線報告 | Lua build_ue_moba／headless／runtime smoke與既有instrumentation | 統一release建置／部署、固定門檻與實測server／client／IPC／UE時序報告 |

## 下一批的選擇方式

Grok與主agent只針對目前程式對上述契約的具體缺失提出下一個實作包，附精確檔案、缺失行為、完成條件；純粹尚未跑完整驗收不再被分類成缺程式。不得無限新增未要求的遊戲特色、AI策略或防禦性細節。

所有root與submodule dirty變更保留，不commit／push／deploy前述未整合版本。Grok協作state保留實際job與session，費用若未報不得當免費。
