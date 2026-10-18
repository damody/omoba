# 玩家道具與計分資訊

承接可玩MVP P0.3，僅改正式Unreal呈現；沒有新增權威對局。

- [x] 將計分板標頭與隊伍／玩家欄位集中格式化成中文，保留原公開資料驗證及u32範圍，字級依當前顯示設定更新。
- [x] 道具效果說明涵蓋所有既有ActiveKind；未知／非法資料不顯示NaN或內部ID。空槽、被動、可用、冷卻、不可用採一致短中文。
- [x] 背包tooltip補上既有公開Tooltip效果說明，保留主次與self改鍵；目前只顯示名稱／冷卻而漏效果，不能靠商店另開頁彌補。
- [x] 被動裝備加成：正式作者資料已有atk／hp／armor／ms，但原UE catalog未投影，只顯示被動裝備。由既有validated codegen產生四項數值及明確可用旗標，UE-only payload與同一商店／背包formatter共用；不寫死Sword等名稱／ID，不改Rust遊戲規則或ABI，不猜缺資料為零。
- [x] ordinary build、當前native模型與真widget／小視窗／text2驗證、來源SHA及必要操作回歸；保存證據及錯誤。此為下列資料／呈現範圍，真hover字級另有後續計畫。整體完整驗收仍最後進行。

前三項範圍接受：.ai-collab/playable-mvp-player-item-text-accepted-v1/summary.json。ordinary player-item-text-build-v5（NoEngineChanges／local／nocache，UBT13.39秒）成功；native-v5完整exact9全Success／0warning／exit0，涵蓋真compiled十一道具與七active kinds、道具輸入／shop輸入與交易／退休／fallback／responsive／scoreboard。92來源SHA一致；native-v4同來源十二原圖逐張直接檢視，四組尺寸／文字scale皆可從頭捲到玩家10，不以縮字擠下十列。只有模型與native synthetic HUD／公開state，沒有新對局／ACK或完整MVP驗收。E1156–E1160原失敗保留。被動加成數值仍待下一項，未將「被動裝備」短標籤當成完整道具效果。

被動數值增量範圍接受：.ai-collab/playable-mvp-passive-item-values-accepted-v1/summary.json。Rust1.95 codegen56tests／all-target check、正式generate與read-only check成功，content_hash仍8c832af990400213；ordinary build-v1成功280.39秒，native-v1 exact6全Success／0warning／exit0，99來源SHA一致（含generator與生成metadata／manifest）。四項加成由同一validated source按1/1024生成，新增明確availability旗標，無資料與真零值分開；同一formatter支援自訂fractional／hybrid，非法值不部分揭露，七種active數字亦保留最多三小數而不把0.125印成0.1。真背包tooltip模型保留數值／改鍵，正式catalog輸入與shop搜尋／fallback／退休／responsive通過，沒有Rust ABI或遊戲規則變更，無新game／ACK。

後續：E1163已確認預設tooltip字型8pt及wrap1000的問題；不得把本批GetToolTip文字讀取當作真hover可讀性。接2026-10-18-readable-hud-tooltips.md，再接2026-10-18-player-shop-text.md，P0.3及整體MVP父項仍pending。
