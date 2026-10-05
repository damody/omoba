# Carry／Support 策略與公開健康資訊（2026-10-05）

## 本批計畫與決策

1. Bot perception 改由同一份 committed baseline 解碼位置／team／kind／owner／HP，仍先限定 TeamVisibilityState.index.current。沒有讀取敵方 ECS Pos／CProperty、私有命令、camp aggro 或記憶 ghost。
2. visibility 新增共用 borrowed component framing decoder：先依剩餘 bytes 限制 count，驗整個 frame／長度、schema 重複與 trailing bytes，不使用部分成功資料。健康欄位要求既有40-byte schema、0≤HP≤maxHP且maxHP>0；缺／錯誤健康資料不建立可攻擊 perception。
3. 目標必須 disclosed HP>0。Carry 在550公開距離內優先敵方小兵，再以絕對 HP 小者優先、距離與canonical ID穩定決勝；其他lane角色維持基礎策略，Support優先敵方英雄。這是低HP兵線策略，不是已保證 last-hit 或預測隱藏攻擊／彈道。
4. BotAssignment 增加可選 escort_player_id。match plan 編譯從完整角色 roster 配對同隊 Carry，包含真人，不只在 Bot assignments 找對象；執行前驗非自己、同隊、存在、Support專用。
5. Support 沒有可攻擊目標時跟隨已公開且存活的 Carry，200距離內保持；缺／死／未公開時走自己的公開路線，不查 authority 補座標。跟隨／停追逐全部使用正式 MoveTo(queued=false)，不直接修改 ECS command queue。
6. 每隊 perception 在每次 think 共用一次，不為五名 Bot 重複解碼；空 Bot配置不解碼。這是熱路徑重複工作的修正，未做效能基線驗收。
7. 原有角色與英雄來源不變，無英雄ID／技能slot特例、C++／Blueprint／ABI變更。KCP controller ownership 接線與通用技能依然獨立待辦。

## 時序錯誤與通用修正

- 新增停止追逐的 ECS 測試最初失敗：MoveTo 到 committed 自己位置後，該 tick 仍先沿舊方向移動約5.1 units。正式 Dispatcher 早於 Moves admission，不能假設即時停止。
- 不改共用模擬順序。Support 等待同一近距離在途 MoveTo 完成，不追著每tick新pose替換，避免每tick思考時來回修正；queued命令存在時仍用正式替換命令取消。
- 測試確認原始Stop target精確等於committed位置、當tick正常接入MoveTo、下一tick自然回到目標、之後不重送。失敗與修正保存於E160，不宣稱零延遲停止。

## 當前功能確認

- core role_bot 篩選：8/8 passed（6個perception／策略／identity＋2個plan）。包含hidden baseline改變不影響目標、死亡排除、低HP小兵優先、Support敵我與存活檢查、非法baseline／重複schema／超大count／trailing拒絕、compiled escort配對。
- base_content role_bot 篩選：最後版本3/3 passed，exit0。包含原10人60Hz正式輸入、真人＋九Bot控制權不變，以及新增真人Carry的30ticks跟隨／未commit authority位置不影響目標／停止追逐時序確認；另驗unknown／跨隊escort拒絕。
- 前述首次停止測試1/1 FAILED保留，不以先前未含停止場景的3/3掩蓋。
- diff --check成功。既有td_rounds warnings保留，沒有Unreal build／DLL staging／MCP／100场或完整終局驗收。

## 未完成

Carry精準補刀與安全攻擊傷害／冷卻決策、Support完整保護／技能、Jungle gank／有限安全記憶、通用技能與三種完整英雄；政策距離／優先序的內容調整與KCP controller ownership／admission仍待接線。所有上述完成後才做100場與Unreal／LAN整體驗收。

OpenSpec 5.5 不勾選，整體20/30不變。
