## Context

使用者指出小地圖點擊／滾輪不能操作及畫面過於簡陋，並要求包含 A 點地板與修飾鍵的完整 LoL 式對局操作實作。既有 71 native／69 Blueprint 本機驗收不是使用者操作的完整證據。本次操作範圍包括對局、HUD、鏡頭、設定與隊伍溝通；鍵位以本專案預設 profile 明確定義，可重新綁定。

已確認 source：`SOmMinimap::OnMouseButtonDown` 只送右鍵MoveRequested，左鍵被Handled但沒有鏡頭事件；camera pawn有AddZoomInput與named BindAxis，但DefaultInput.ini只有MouseWheelAxis AxisConfig，沒有對應AxisMappings，且預設EnhancedInput。這是高度可信的接線缺口，仍須實際viewport輸入驗證。A目前直接AttackMoveNow，不是A→左鍵確認。

## Goals / Non-Goals

**Goals:**

- 依序完成鏡頭、小地圖、A指示模式、施法與快捷鍵、HUD、隊伍溝通、戰鬥與場景視覺；所有列入完整對局profile的動作有實際行為或明確權威拒絕，不留下假的功能入口。
- 一份具版本的可持久化設定，涵蓋滑鼠／鍵盤、施法模式、鏡頭、UI大小、色彩與音量，錯誤設定有可恢復預設。
- 保留權威模擬、filtered frame與單份玩家replica，camera為local presentation；資訊面板／鏡頭不能讀取敵方隱藏actor補資料。
- 真輸入必須改變有效render camera或取得原gameplay ACK，保存證據；原計畫未完成項目保持獨立。

**Non-Goals:**

- 不複製LoL圖像／音訊、角色或專屬技能；不建立Riot客戶端、排名／社交服務、觀戰平台或商業配對。對局內基本操作都在範圍內。
- 不修改UE Engine或授權、不維護omfx、不以假winner／本機生成ward或spell效果代替權威行為。

## Decisions

### 1. 統一可設定的輸入動作與UI仲裁

用語意action（Move、AttackMoveAim、AttackMoveClick、Stop、Hold、CastSlot、SelfCastSlot、UpgradeSlot、CameraFocus、CameraLock、Ping等）對應版本化profile。現有legacy／Blueprint adapter轉接同一dispatcher，只有一條路徑可提交權威command。正式通用接線採per-local-controller UOmPlayerActionOwnerComponent與owned UInputComponent，使用UE公開InputStack、明確axis key與可回收profile binding；其可與EnhancedPlayerInput共存，不另註冊重複mapping context，不依賴未定義named axis。兼容既有BP_PlayerController僅繼承APlayerController，不改既有uasset父類別。camera亦由exact owner持有獨立component並消費同一profile；退役／pop／owner交接清除held state與event sample。這是本次實作的正式輸入架構。

UI優先序：文字輸入／modal > 可互動HUD > 小地圖專屬gesture > 施法／攻擊aim > 世界選取／移動 > camera edge pan。Mouse capture與按鍵release在focus loss／PIE stop／reconnect時取消；不留下按住狀態。Wheel作用於世界鏡頭，scroll list由UI消費；小地圖hover的wheel依profile明確允許／阻擋，不含糊穿透。

仲裁按action分類，不能因游標停在HUD就禁用所有鍵：S／H／B、自我施法與升級等不需要world point的動作，在非文字／非modal的HUD hover仍可執行；point／target命令必須取得合法世界target或由該HUD明確提供target。Shift／Ctrl／Alt同時按下有固定priority與衝突檢查。

### 2. 鏡頭與小地圖使用同一安全座標轉換

復用OmMinimapModel投影／反投影，DPI／letterbox／有效map bounds一處計算。左鍵按下与拖曳只發local focus request，右鍵發現有MoveTo權威command；無效map點消費事件而無副作用。左鍵drag capture至release，focus loss必須釋放。camera不是gameplay hero actor：從本local controller所持有的有效camera pawn／view target取得，不掃全世界猜pawn。鏡頭viewport輪廓由實際camera rays與平面交點投影，含縮放／map clamp。

模式仲裁：Free／LockedHero／TemporaryFocus／MinimapInspect。Space按住追蹤hero、放開回到先前camera模式與視點；Y切換鎖定。小地圖拖曳在LockedHero暫停跟隨，放開回復鎖定；Free模式保留所選視點。回城／死亡／重生不是任意重設camera的理由，失效hero focus安全退回Free；新owner frame恢復原設定。縮放以有界距離與固定傾角為基礎，不用調大高度模擬正常視角。

### 3. A指示與施法是明確的狀態機

A進入AttackMoveAim：畫出攻擊範圍與游標狀態；左鍵有效地板確認一次AttackMove，右鍵／Esc取消；HUD點擊不確認世界指令。另提供AttackMoveClick（建議Shift+A，可改鍵），不要將Shift佇列偷偷套用給相同快捷鍵。既有queued能力保留獨立可設定action且清楚標示為專案擴充。

技能每slot提供normal／quick／quick-with-indicator：normal按鍵進入aim＋左鍵確認；quick按下直接提交；indicator按住顯示、release提交、Esc或focus loss取消。Ctrl+QWER升級，Alt+QWER自我施法；只攻擊英雄modifier與toggle支持合法目標过滤。S停止與H保持須使用不同可驗證語意；不能兩鍵都發同一hold後宣稱功能齊全。

### 4. 尚缺gameplay動作先定義權威能力

D/F召喚師技能、飾品／ward、ping與目標限定若現有schema缺少能力，先擴充共用命令／內容設定與server驗證，再接runtime／C ABI／UE。按玩家與slot配置自有能力（例如移位／治療）與ward資源／生命周期／視野，不借用QWER、伪造owner frame或只在UE產生效果。維持content／protocol版本一致；沒有mana的英雄不畫假mana bar。

召喚師技能參數必須由內容設定決定，cooldown／charges由權威回覆；ward被visibility pipeline正確揭露與隱藏。不是新遊戲平衡重設：測試內容與現有正式對局配置分開記錄，不能藉此強迫舊v6終局。

### 5. HUD與場景採可讀性的共同基準

底部集中hero portrait／HP資源／QWER／D/F／item／金錢；右下minimap与camera框；左側隊友狀態；上方隊伍比分／時間／公開目標；商店和Tab計分板作獨立overlay。hover tooltip含熱鍵、消耗、cooldown与不可用原因；point click／attack／cast／shop／recall有明確視覺與音訊回饋。所有UI根容器使用適當hit-test可見性，透明空白區不攔截世界輸入。

先建立一張完整可讀三路地圖：地面／可走與阻擋邊界、lane／river／jungle、塔／基地、hero／creep輪廓、光照、地面陰影與team色。裝飾不遮目標，不更動權威collision。skill VFX遵循已有權威cue与生命周期，hidden敵人無音訊／特效洩漏。原創圖像／材質與音效有provenance，缺檔明確記錄，正式模式無debug cube/checker作主要視覺。

### 6. 操作驗收分層，避免helper假成功

各增量先純狀態／座標／權威Rust回歸，再native widget事件與actual camera transform／projection，最後新owned viewport的真按鍵／pointer流程。记录localplayer、view target、camera projection、命令ID與ACK；不能只看DesiredZoomDistance或counter。UI／camera可先在0authority fixture驗收；gameplay新能力用deterministic Rust fixtures与ABI回歸。

既有true-authority批次10/10耗盡不重設；自然整場、實際新能力對局、真LAN和固定效能的成功證據需另有可用執行條件，缺證不勾PASS。純fixture不假裝真人或LAN。1920×1080／2560×1440及100%／150% DPI，windowed／fullscreen／PIE，新模式從選角進入與重連都列入矩陣。

## Risks / Trade-offs

- [Enhanced Input與舊Blueprint重複綁定] → 每action一個owner dispatcher，列出migration清單，驗證一次physical event只產生一個command。
- [跟隨模式把小地圖focus立即拉回] → 明確gesture state與resume規則，測actual camera持續多frame。
- [不同local player／PIE世界的camera誤選] → local controller／viewport限定，不用actor名稱或全域查詢。
- [新增ward／ping洩漏資訊] → team filter、fog gate、資料最小化与remote負驗證。
- [畫面改善造成性能退步] → pool／節流／LOD與既有固定門檻；不提高基準，沒有完整樣本就保持pending。
- [同時新增數十按鍵使UI不可學習] → 主profile、可搜尋hotkey設定、遊戲內快捷鍵提示與衝突提示。

## Migration Plan

先camera/minimap與輸入焦點；再A/施法與權威action；再HUD和visual slice；最後隊伍溝通、可選WASD與完整設定。每批保留既有assets、engine gate與ABI一致性，release/stageSHA後正式本機驗收。新版設定有schema version與舊設定migration；錯誤設定保留檔案並回復預設，rollback停用新mapping context並保留合法既有runtime入口。

## Open Questions

無需使用者確認的事項。實作前盤點DF／ward／S停止／隊伍聊天的既有後端支援，按已定契約補缺。Riot版本可能改動預設鍵位，參照來源僅說明控制分類；本專案profile表是實作與驗收依據。
