# 通用 HoldPosition 與已披露兵線推塔協調

## 計畫與決策

1. 檢查現有等待：MoveTo 抵達清命令，NoOp 不清 AttackMove，均無法保證持續不自動攻擊。
2. 先實作通用正式 HoldPosition，再讓推塔等待使用它；Bot 不直接寫自己 command／HP，也不讀敌方仇恨。
3. 只確認當前功能，更新OpenSpec與防錯；完整驗收最後執行。

## 實作

- proto PlayerInput／RendererInput 新增20 HoldPosition（queued）。client InputBridge 正常 owner／epoch 驗證、轉為正式輸入，無target reference；沒有新增公開除錯繞過。
- HeroCommand／PendingHeroCommand 增加持續Hold，既有正式queue admission、immediate replace與attack interruption沿用；queued flag維持原語意，後續immediate移動／攻擊可取代。
- command tick 不前進Hold、不生成MoveTarget；hero tick 不自動攻擊，timer正常運行。已射出的投射物不回收，Hold不免傷。
- 原MovementPriority公開安全的攻擊抑制旗標，filtered非owner不需知道command／目的地；正式accepted input僅送owner。MOBA allowlist與Recall同批中斷包含Hold。
- Bot先照原一般戰鬥優先順序；選敵塔時要求650內有目前披露的存活同隊兵。1100內觀測到無援軍可攻擊敵塔且無其他戰鬥目標，使用Hold，不fallback AttackMove。
- 同一Hold且queued空不重送，援軍重新披露後正常AttackTarget取代；hidden／dead／enemy／hero／building不能當友軍兵。Support不因escort覆蓋Hold；Jungle保留原camp／支援。
- 650／1100是共用MOBA推塔觀察政策，不是讀取隱藏tower射程／aggro；有兵只代表進攻時機，不保證塔不打英雄，也不是完整塔傷害風險評估。

## 單項確認

- core `--lib siege`：2 passed，新wave gate1＋既有分類1。
- client-runtime `--lib hold_position`：1 passed，protobuf roundtrip／queued保留／wrong owner與stale epoch拒絕。
- base `--lib siege_wave_hold`：1 passed，正常Production60Hz持續65 ticks，無MoveTarget／位移／autoattack、不重送；未提交wave位置不釋放，最新披露後正式AttackTarget接受。
- base `--lib hold_position_filtered`：1 passed，Hold保持再MoveTo取代，兩隊各6 steps／12total canonical hash與權威重新bootstrap一致，零ComponentRepair；foreign accepted input不外洩。
- 相鄰舊 `disclosed_structure_state`：fixture更新後1 passed，仍涵蓋8双隊unlock／Hide／Reveal／Finished steps。兩次fixture失敗與package lock／路徑錯誤詳見E198，沒有改產品規則迎合fixture。

## 剩餘與界線

沒有新增Unreal專用C++／Blueprint graph；renderer API可提交Hold，但UE實際快捷鍵／按鈕尚未接入或驗證。沒有完整攻城戰略、撤退／塔傷害預測、100場、UE／LAN或效能驗收；未build/stage DLL或混用舊二進位。OpenSpec 20/30保持，5.5／4.1完整項不勾選；E177保持。生成內容／Lua hash未變。
