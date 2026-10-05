# 定身期間 Bot 旅行與受阻續航整合（2026-10-05）

## 計畫與實作決定

1. 接續5.5，修正通用Bot控制與續航的整合：定身期間原先仍可能提出新撤退／推線／護衛旅行，且續航決策無法回城時直接continue，會排擠合法恢復技能。
2. 每位Bot只讀authorized owner標準rooted，沿既有stun全等待；定身不提交新的Retreat、Advance、非原地Escort，保留現有命令。正常Hold、普攻與技能能力不被定身一概停用，權威移動控制仍獨立執行，不把AI策略当安全准入。
3. 只有續航Recall被控制阻擋，或Retreat被定身阻擋時，進入本次decision-local recovery_wait。沿原作者policy順序與rank／Mana／CD規則選合法恢復技能，抑制EnemyUnit／EnemyPoint／ApproachEnemyPoint進攻意圖；無技能可用便不發新戰鬥／旅行命令。合法技能學習仍保持原規則。
4. 沒有新增跨tick私人游標／等待計時器；控制到期依當前committed disclosure重新決定正常撤退或回城。已在途的撤退、基地Hold與合法Recall維持原優先序，不用零距離Move或偽造Hold取代受阻等待。

## 當前功能確認

- 新正式60Hz續航1項，分有威脅撤退與無威脅回城兩個案例：原本選MoveTo／Recall，定身後改正式自療，HP100→210、恢復CD成功、較早宣告的進攻技能未進CD；四次冷卻期間零新輸入且位置保持；到期恢復原逃生輸入並經正式dispatcher接納。
- 新正式60Hz旅行1項：以正式Hold建立既有命令，Top／Mid／Carry／Support／Jungle在定身時均零旅行輸入且不改既有命令，控制到期全部恢復MoveTo／AttackMove決策。
- 相鄰回城控制、暈眩／沉默Bot語意、定身位移與自療案例各1/1通過。新2＋相鄰3共5個直接相關確認，沒有全套測試或最後驗收。
- 不改Lua／內容生成／hash／協定／Unreal C++／Blueprint，不部署DLL、不維護omfx、不commit／push。完整框架21/31與完整5.5仍保持待完成，100場／LAN／UE／效能驗收留最後。

## 錯誤與防重犯

- 首輪旅行fixture在HeroCommandQueue unwrap失敗：新生hero未必有命令component。改用正式HoldPosition輸入建立真正既有命令再檢查保留，而非測試直接塞component或放寬production。新續航首輪已成功；修正後兩項與相鄰三項均成功，沒有Rust編譯失敗。
- 長上下文讀取超限時改具體符號／小區段，不把截斷輸出當完整證據；已知td_rounds dead_code警告保留。問題與決定記錄E253。
