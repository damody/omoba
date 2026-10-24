# 路徑施放提示錯誤與決策

P01：第一次編輯codegen誤把raw string當普通字串，插入literal \n與escape引號；在任何生成／建置前讀回發現，已改為真換行與原始引號。預防：先確認所在Rust literal種類，局部patch後讀回。

P02：誤猜OmCastPreviewFeedback.cpp存在；實際為header-only。P03：Windows rg把含wildcard的路徑當literal，出現os error123。兩者均為查詢失敗，未修改正式來源；後續以已知目錄搭配-g查詢。

決策：僅畫意圖capsule，不掃描actor推測地形或暗藏單位，不宣稱預覽終點已獲權威確認。既有後端固定點碰撞裁切／root回滾維持。單機圖片不當作外部ACK。
P04（WorldV1真失敗）：SweptPathPreview在Outline.Add(Outline[0])觸發UE TArray alias斷言，native process exit3；不是環境問題。即使Reserve足夠，TArray仍禁止加入來自自身的引用。改為先複製const FVector First再Add(First)，適用所有閉合輪廓。保留V1 engine log／receipt，source-v1等待worker先以精確PID生命期停止，待capture-v1退出後才修改。修復僅C++ header，不重跑未變更的Rust內容；重新UE build-v2／World-v2及source capture/check-v2。

P05（WorldV2真失敗）：51成功／1失敗，唯一失敗為ClearHeroSnapshot後路徑寬度仍1500cm；之前只補ResetRuntimeInputScope，漏掉共用hero snapshot清理。新增欄位須追蹤所有retirement入口，已在ClearHeroSnapshot清除，不放寬測試。WorldV2／source capture-v2保留，精確停止仍等待的source-v2後、capture-v2退休後修復，重新build／World／capture／check-v3。
P06（目視修正fixture）：V2 tooltip圖可讀，但test用10cm/單位，而正式設定為100，導致畫出range120／width15且Level1沿用rank4射程。正式bridge比例原本正確；fixture改從RuntimeSettings與rank1 metadata取range900／width150，補真內容名稱／描述／魔力／冷卻，避免錯誤圖片證明。幾何單元測試保留明確cm輸入。

最終WorldV3：52成功（51 Success＋1 SuccessWithWarnings）、0 Failed／0 NotRun，4個warnings均為既有RuntimePresentationReset的Untitled World DestroyActor無context清理訊息（2 route、2 fallback）；engine log也有controller重播，非8個新事件。SweptPathPreview完整通過。RustV1為1938通過／10既有ignored；四Lua契約、backend-v1、UE build-v3通過。未把V1崩潰或V2退休失敗抹除。
目視：V3新實際SOmHudToolTip圖顯示火焰衝擊、range900／width150／mana80／CD30與三模式指引可讀。其餘10圖作既有功能回歸：地圖斜向／角色／名牌清楚；first-kill是公告區fixture，HUD部分在外；objective是條幅裁切；不拿它們當完整HUD／真網路／新增路徑遊戲錄影接受。新增capsule以真native builder與owner接線測試／正式paint編譯驗證，並未擷取實際對局capsule影像。
