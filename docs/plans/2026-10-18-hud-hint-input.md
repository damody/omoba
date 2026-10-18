# HUD 提示與可操作區域

目前整合世界回歸發現兩個獨立問題：舊 compiled map fixture 為ABI28，正式reader為ABI29，嚴格檢查正確拒絕；短技能提示仍用interactive popup攔截滑鼠，400×234小地圖中心實際命中SBorder而非小地圖。失敗原圖已直接檢視，不以清掉測試tooltip或改點擊座標遮蔽問題。

- [x] 沿既有Rust正式producer重新匯出公開geometry為ABI29，maps2／synthetic visibility／無live units／無authority ACK；不手改舊fixture。world-export-v2單一正式export測試成功，map catalog hash仍b7a9dda88f881c49。
- [x] Tooltip依實際SScrollBox overflow決定是否需要閱讀互動。短內容仍可hover觀看但整個popup不參與hit-test，不搶click／focus；長內容保留scroll、keyboard閱讀、退場與owner守衛。
- [x] 真Slate短內容顯示與穿透、移出來源即關閉；既有800／400文字1／2長內容scroll／焦點／退休回歸保留。
- [x] 普通NoEngineChanges／NoUBA／nocache建置，同次回歸含小地圖、C面板、buff、HUD與兩map整合world。保存當次原圖，逐張檢視必要固定視點；不是新增權威對局或最後完整MVP驗收。

沿用共用FOmHudReadableText字級工具，避免小地圖的字級依賴tooltip widget類別。保持Lua5.4工作流，將共用字級header納入140份相關來源指紋與正式geometry producer證據；不變更Rust／IPC11／ABI29。

接受：playable-mvp-hud-hint-input-accepted-v1。ordinary build-v3 13.27秒成功；native-v2 exact13 Success／0test warning／trueexit0；140來源全程一致，33 Rust／proto／ABI來源與既有已完成gate相同。9張當次hint原圖及兩map各6張game viewport原圖共21張逐張直接檢視；C面板四張既有hero-attributes-accepted-v1原圖明示沿用，其當前功能另在本次13項中fresh驗證。world actors／visibility明示synthetic；視點為固定renderer fixture，不冒充正常權威對局或最後MVP驗收。E1196–E1199原始失敗artifact保留。

E1199診斷：同一window且座標不變，first paint後的visibility尚待visualizer更新，parent仍非passive，hit包含tooltip；完成該window最後paint後，hit確實落在SViewport且不含tooltip。測試依既有長提示流程完成真正paint，沒有移動測試點來避開提示。
