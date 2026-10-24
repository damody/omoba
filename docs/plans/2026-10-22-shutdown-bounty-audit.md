# 本輪完整度盤點：連殺與終結賞金

接續全遊戲17系統盤點，補經濟系統仍固定擊殺金的缺口。沿正式後端／共用Rust／內容生成／唯一Unreal前端，沒有新增網路對局。

## 本輪

- Lua `moba_economy.kill_streak_bounty`四欄政策：threshold／initial_gold／per_kill_gold／max_gold；正式內容3殺起150、每殺加100、上限700。這是目前訓練玩法值，不宣稱對應LOL特定版本。舊內容缺省關閉，欄位型別、金額與門檻有界；hash／compiled catalog／hot reload變更均要求所有peer重建。
- 正式server明確選用generated政策，孤立legacy config預設關閉。連殺存在player roster，成功敵對英雄擊殺且錢包入帳才增加；第一次致死取受害者賞金並清除，匿名／環境／強制死亡也清除，重複致死不重付。召喚／投射物沿既有完整來源歸因，死亡玩家可收入已接納的重生錢包；助攻金與XP不變。
- 規則與連殺納入authority digest。額外金可領取時，公開score metric供所有隊伍使用；不從總擊殺、私有經濟或可見entity推測。舊snapshot缺賞金欄位為零，重複／錯誤audience／非法金額拒絕。
- 正式60Hz驗證找到通用death_tick在下一step再次發Unit預設金／XP；正式MobaMatch存在時停用通用附近獎勵，仍發Death，由權威擊殺／助攻／兵線／野區路徑唯一結算。legacy模式保留原規則，另有直接system對照回歸。
- typed protobuf新增optional語意的scalar，checked-in fallback同步；bridge C／Rust ABI32及24-byte scoreboard row，Unreal payload與實際計分板顯示額外賞金，零值移除標籤，非法值閉合。未維護omfx。

## 驗證範圍

集中執行內容規則／重建保護、核心真實傷害累積與一次終結、匿名／強制死亡、phase／pause、digest、退休召喚投射物與死亡錢包、正式60Hz generated技能及真實PostStep公開metric、codec／client隔離／bridge接線、Unreal原生計分板與渲染輸出；兩張map作回歸。結果依最終驗收文件與封存index，不以接線文字冒稱通過。

新增賞金圖為獨立Slate fixture內實際UOmCommandBarWidget的公開計分板，已看到+700金資料列；無真連線／map，底部其他HUD未作此圖的版面接受。第一次只有外框的PNG保留為撤回證據；修正重複獎勵後最終採build-v3／RustV3／WorldV3與新來源基準。不能將fixture可見布林或PNG非空取代目視。

## 後續

連敗金額折價、首殺、全隊經濟差補償與更多終結公告尚未完成。其他缺口仍有持續引導／技能動畫／特殊幾何、完整召喚複本／競爭落點整組回滾／退休callback、objective永久stack／傳承／小兵光環／分裂、塔皮與美術。真實連線賞金畫面與網路ACK不屬本輪單機接受，原MVP仍33/44／11外部待驗。
