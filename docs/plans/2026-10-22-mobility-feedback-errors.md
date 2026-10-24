# 位移施放提示錯誤與決策

初始實作階段尚未執行驗收；後續實際失敗與結果依序追加，不覆寫舊版本證據。

設計決策：root與stun/silence分開。普通技能不可因root就被誤報無法施放；只對由typed SweptPath／executable DashToPoint宣告位移需求的技能套用root提示。完整公開buff清單不受UI64 cap，讀取限制仍4096；精確owner／generation／存活與source relation守門保留。非法root只使root channel未知，不誤傷普通cast channel；未知metadata無法判定分類，保守撤銷兩者。

沿用前批教訓：TArray閉合先複製首點再Add；新增owner policy同時清ClearHeroSnapshot／ResetRuntimeInputScope；native reflection fixture用scoped FEditorScriptExecutionGuard，不放寬正式派送；圖片fixture不猜world-unit比例；只使用已發現路徑。UE維持-NoUBTMakefiles，既有警告須分類，不能默認當成功無問題。
P01（執行前自查）：複製驗收script時全域替換PathPreview批次名，誤把既有測試ID SweptPathPreview變成SweptMobilityFeedback；已在任何World執行前讀回抓到並恢復正式ID。批次名與既有測試名不能一起全域替換；新sealer／World清單須逐項比對。未漏跑或放寬測試。

P02（WorldV1真失敗）：52成功／1失敗，MobilityCastFeedback唯一錯誤為tooltip需求斷言。fixture未填MaxLevel（0），Level1被正式SetAbilityVisualState的MaxLevel>=Level校驗拒絕；正式校驗正確，不可放寬。改從生成metadata取MaxLevel及同rank真名稱／描述／冷卻／魔力／range／width，比例從RuntimeSettings取得。保留V1 engine／report／PNG／receipt與來源快照，source-v1精確停止，World／capture退休後才修復。重新UE build-v2與World／source-v2，未變更Rust／正式行為。未把此失敗當成引擎環境問題。

最終WorldV2：53成功（52 Success＋1 SuccessWithWarnings）、0 Failed／0 NotRun。4個warnings皆既有RuntimePresentationReset的Untitled World DestroyActor無context清理：2 route、2 fallback；controller重播非額外事件。MobilityCastFeedback與既有CastControlFeedback／SweptPathPreview皆成功。Rust1940通過／10既有ignored，四Lua契約與backend-v1／UE build-v2通過。V1失敗／取消／來源證據保留。
目視12圖：新實際SOmHudToolTip定身剩2.5秒、火焰衝擊、rank1 range900／width150／mana80／CD30、位移root與非原地需求可讀；既有沿路tooltip／地圖／名牌／召喚／選取卡作回歸。first-kill僅公告fixture，HUD部分在外；objective為條幅裁切；不當作完整HUD或真網路／真capsule對局接受。

P01補充：封存script自查也發現同一全域替換留下舊scope字串，已在封存前改為content-derived-mobility-root-and-zero-displacement-feedback並補召喚師／道具提示remaining。批次模板須分開處理路徑、測試ID與scope，不能以replace鏈猜最後內容。
