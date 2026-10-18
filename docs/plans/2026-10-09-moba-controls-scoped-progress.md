# 對局操作目前成果與接續順序

本頁整理已執行的範圍證據，避免將來源完成、單項通過與最終驗收混淆。完整需求仍以 `2026-10-07-moba-controls-presentation-plan.md`、`2026-10-08-moba-control-event-matrix.md` 與 OpenSpec `complete-moba-controls-and-presentation/tasks.md` 為準。父任務目前8／44；完整驗收最後執行。

2026-10-10 模式HUD已範圍接受：mode-chrome-accepted-v1；ABI21真正snapshot epoch／無fog-grid／mode0與cached roles／量測reservation／完整generation已審查。24distinct Rust、兩項checks、cbindgen/header-layout／Lua31有實跑證據；最後owned-only修正下明示沿用native-v2八Success／兩筆warning。ordinary build-v4 nocache／refresh=false真正成功36.31s，fresh viewport-v2真正290／119／101到mode-retire，原274／119／94前綴與清理／listener全過，六張當次原圖逐張目視，786來源fingerprint未變。E590–E602與原失敗保留。黑色合成場景與MOBA截圖catalog unavailable不是完整世界／商店驗收。接續fresh Grok4.7非fast道具圖示import／registry／六卡／商店來源實作中，圖示尚未接受，最新writer以handoff為準。

| 功能 | 目前範圍證據 | 剩餘界線 |
|---|---|---|
| 小地圖與鏡頭 | minimap-preferences-accepted-v1：左右位置、真實hit／drag、有效camera、座標與來源回收；121／58／36 owned案例 | 戰術ping、完整OS DPI／PIE矩陣 |
| A點地板與按住命令 | held-world-cadence-accepted-v1：真實確認／取消、UI不確認、去重、有界repeat；31／21／10，後續前綴保留 | 原command的真正authority ACK另在最終矩陣 |
| 主／次快捷鍵 | dual-bindings-accepted-v1：獨立release、單owner時鐘、設定commit／unbind；47／30／10，後續前綴保留 | 全平台／PIE／重連入口完整矩陣 |
| QWER施法 | owned-ability-accepted-v1：normal／quick／indicator、修飾鍵、升級、self／point／unit與失效；112／54／35 | 真對局結果與完整入口矩陣 |
| D／F召喚師技能 | owned-summoner-accepted-v1：獨立槽、模式、冷卻、死亡／換來源拒絕；158／69／51 | 未配裝／不支援保持不可用；真對局結果另驗 |
| 六格道具鍵盤與回城 | owned-items-keyboard-accepted-v1：self／point／unit、Alt、B／S／H與恢復；253／115／80 | ward獨立玩法、完整整合矩陣 |
| 六格道具卡 | owned-item-cards-accepted-v1：真實公開點擊、確認／取消、文字輸入guard、disabled／退休；274／119／94 | 圖示尚未import／接線，未建立charges與ward玩法 |
| 設定文字與響應式排版 | settings-physical-accepted-v1：ordinary build-v5、兩項native零警告、11倍率、同寬spin／改鍵／取消／關閉／還原、fresh274／119／94與七張目視圖 | 完整OS DPI／PIE矩陣、其他顯示契約 |
| 模式介面 | mode-chrome-accepted-v1：ABI21真正epoch／無fog-grid／mode0、cached roles／量測reservation／完整generation；普通build-v4、native8保留兩警告、fresh290／119／101及原前綴／cleanup／六張目視通過 | 目前支持的合成視窗與操作證據；完整世界畫面與OS DPI／PIE矩陣仍待做 |
| 道具美術 | E603–E611來源修復審查；ordinary build-v3真正23.57s；正式Content11原PNG／manifest與UAsset，bootstrap-v2真11import11save＋重開11verify0save；repeat兩程序皆11verify0save，11package hash／timestamps不變。native-v2資源平台1254已Success，真實viewport800六卡圖已完整 | native-v2真正6Success／2Fail／一保留warning：400filled與既有400text1.25 fit失敗，E612固定六份文字寬撐高；同Grok4.7非fast正修一般compact量測／呈現；尚未四張完整capture／freshowned／整批接受 |
| 商店／ward／溝通 | 已準備authority undo、獨立trinket catalog、team通訊有界封包 | 各自資料、權限、生命周期與輸入／呈現完整接線 |
| 世界與音效 | 地圖geometry／原創音效import封包已準備；13原創WAV來源已驗 | 正式地圖材質／模型／lighting／動畫與cue、實際音訊資源及播放 |

上表owned數字依序為cases／intents／control stages。所有摘要位於 `.ai-collab/<accepted-id>/summary.json`；後續範圍保留原ordered prefix，明示沿用未變測試，不能把舊結果冒充重新執行。renderer fixture證明本地操作與命令admission，不製造authority ACK。既有權威對局10／10不重設，缺少真對局／LAN／效能條件的證據仍待完成。

接續順序：模式資料基礎→模式HUD組合→道具圖示正式資源／呈現，再依依賴完成商店undo、ward／team溝通、替代控制與世界／音效。每批先獨立審查，再確認當前功能；所有功能整合後執行最終矩陣。實際錯誤持續記錄 `unreal-moba-error-register.md`，原失敗資料保留。
