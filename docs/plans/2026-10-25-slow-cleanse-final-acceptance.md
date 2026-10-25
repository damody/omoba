# 正式緩速淨化：最終單機驗收

OpenSpec `complete-slow-cleansing` 3/3完成。

共用 BuffStore::cleanse_controls 現在清除目前的暈眩、定身、沉默，以及可辨識的正式技能／彈體緩速。既有召喚師淨念（catalog4、210秒）與淨化護符（catalog14、45秒、1400金）沿用同一結算，自身／無目標／可受控使用、存活與owner、冷卻等政策維持。召喚師與道具提示同步更新。

資料技能緩速須通過共用declarative source parser、generic_ability_slow家族、單一[-1024,-1] Q10負移速與精確兩欄負載；標準彈體slow須有精確兩欄、Q10可精確表達且互相一致的slow_factor／move_speed_bonus。只對正剩餘生命期判定，不依英雄、營地或任意名稱子字串猜測。自用姿態減速、正向加速、未知／異常／混合負載與其他效果保留。

收集後排序並走既有canonical remove，維護反向索引；保留效果的payload、剩餘時間、periodic remainder及zone clock不變。多來源緩速同時移除，不留下較弱來源接手；沒有新增免疫或每frame掃描。未擴充ABI／wire／ID或技能內容。

定向7項Rust全部通過。核心驗多來源、強弱邊界、合法projectile Q10格式、異常／混合／不明來源、志願減速與buff／DoT時鐘保留、索引與重新套用。既有召喚師計畫提交／拒絕及真實item／summoner60Hz控制案例保留。新正式Production60Hz世界使用實際generated穿風箭Lv1／Lv4，對手購買道具或裝備召喚師：技能傷害／流血／緩速確實命中；pause凍結緩速時間；淨化在受控時清除正式緩速，恢復有效移速且公開slow消失；流血、抑療、護盾、加速與自用負移速保留。後續正式箭擊重新施加緩速，尚在冷卻的道具／召喚師不能重清；paired replay一致。

FreshRust九套 **2123通過／0失敗／9既有ignored**。正式backend／moba-config、UE5.8、DLL/import staging與離線catalog成功。Native4：SummonerControlCleanse、SelfCleanseItem、AbilityRankEffects、DisclosedAllyCard，0測試warnings；兩張新的原生Slate提示截圖已目視驗收。道具保留原有公開root快照前後回歸；實際slow公開狀態由正式60Hz案例驗證。這是共用tooltip元件接受，不代表完整遊戲viewport或實體OS DPI矩陣。

**3494來源／資產一致**：3484未變／10變更／0新增。集中驗收前後hash、set、size、mtime一致；247份證據獨立重核。原始owned API實際退出、程序退休與健康間隔≤180秒保留，錯誤／根因／預防見同批errors MD。

Catalog data hash `40339cea465f0fa5`，identity `b01c80b5babb0fa3`，class surface `7c09b85b21eea4de`。ABI36／presentation12／selective14維持。

未新增真網路對局／quota reset／commit／push／omfx維護或終止無關程序；MVP33/44與11外部項維持。

![召喚師實際提示](D:/omoba/docs/acceptance/2026-10-25-slow-cleanse/native/capture/summoner/cleanse-tooltip.png)

![道具實際提示](D:/omoba/docs/acceptance/2026-10-25-slow-cleanse/native/capture/item/self-cleanse-item.png)
