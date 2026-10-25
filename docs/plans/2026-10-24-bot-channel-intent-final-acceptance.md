# Bot authored channel recovery intent — final acceptance

新增明確 SelfChannelRecovery bot 策略，讓既有引導治療技能可以由 bot 自動開始，再沿用上一批的共用引導等待／恢復。可選 scripts/lua_data/moba_channel_match.lua 使用既有學徒中路，生命低於65%且公開1000範圍內沒有威脅時施放 R；原有其他配裝仍可選用。回城、購物、學習等既有優先序保留；本批正式 rank 測試會隔離這些優先序。

AbilityConst 增加由唯一 RecoveryChannelSelf 效果投影的具名執行 keys；編譯期、runtime載入、startup peer guard 與 ContentShape dev reload guard 都維護此語意。一般技能仍無此投影；其他 bot intent 保持 instant-only，沒有全域放寬未知 channel。規劃時必須有 managed mana，並由實際 host AbilityDef/當前等級的 typed channel spec 與精確費用確認；失效／缺少宣告、錯誤類型、無預算不提交該技能。

安全啟動只讀當前已提交的 team disclosures，對範圍內的存活敵方英雄、小兵、召喚物、野怪與結構採保守威脅判斷，包括中立野怪與受保護結構。邊界包含半徑本身；半徑與生命閾值有界，超過512個觀察或敵方幾何不合法時拒絕開始。隱藏的快取和私有 ECS 位置不構成可知敵情。這是公開情報下的啟動規則，沒有聲稱可以預知埋伏。定身允許站定施法，暫停／沉默與既有 owner channel gate 保留，MP/CD／脈衝仍由正式 authority 結算。

## 集中驗收

- Metadata1：compiled/runtime投影相符，任意作者ID也可投影，instant技能為None，執行key改變同時被 startup 與 reload guard 拒絕。
- Decision2：封閉類型／閾值／半徑／JSON unknown-fields／mana要求；公開威脅的包含邊界、單raw越界、順序、中立／結構、root、資源不足與512／幾何fail-closed。
- Formal1：固定clone-local Lua載入真實十席配裝，5Hz bot→正式60Hz施放；四rank分別驗證 exact MP90/100/110/120、四次35/55/75/95治療、root、pause、不中斷等待及不可繞過CD。實際host metadata缺少／型別錯誤、無預算、公開敵情、私有位置漂移與隱藏快取都覆蓋。每個rank做paired replay。角色等級／rank、敵人位置與學習／購物／sustain隔離是fixture設定，不宣稱整局錄影。
- 八個維護 Rust 套件 fresh1986 passed、0 failed、9個既有ignored；上述 focused4 不重複加入總數。
- 正式後端／moba-config、UE5.8建置、DLL/import library staging SHA、可選配裝離線plan-lock與權威↔generated catalog data hash通過，沒有建立真網路對局。
- 3480個正式來源／資產全量建置後fingerprint一致；相對sealed bot-channel-wait，3473 unchanged、6 existing changed、1新recipe，無刪除。全部原始worker、失敗版、固定輸入、health、owner/waiter、exit receipt、retirement、cleanup及原始API reap保存，sealer完成後另做獨立rehash。
- 本批沒有前端畫面／作者技能資料改動，沒有宣稱新跑native畫面。沿父index保留既有引導UI證據，當前自動施放由新formal測試驗收。

失敗V1原始證據保留：新配裝繼承了被替換角色的學習目標，超過既有64上限。V2從共用authoring graph取得實際選用英雄的技能集合，再篩選學習／策略；沒有截斷清單、提高限制或用舊英雄名稱特判。另補受保護結構的保守威脅分類。兩個probe間只變更這三個明列檔案，其餘投影／執行接線不變。錯誤與預防見 docs/plans/2026-10-24-bot-channel-intent-errors.md。

證據：docs/acceptance/2026-10-24-bot-channel-intent/index.json。下一批來源基準 omfue/Saved/Automation/BotChannelIntentV1/source-before.json，固定 inputs.json，parent bot-channel-intent-parent-v1.json。

下一個具體玩法／操作缺口優先盤點玩家召喚物選取與命令UI：後端已有owner目標與MoveTarget優先機制，需把單機可驗證操作接完整。移動投射物／multistage、更多內容／美術／平衡及11個外部接受仍pending；MVP33/44不變。

沒有新增真網路對局、重置quota、commit/push、維護omfx或終止無關程序。無人值守期限2026-10-26 23:59:59 Asia/Taipei。

獨立封存檔案數 139。
