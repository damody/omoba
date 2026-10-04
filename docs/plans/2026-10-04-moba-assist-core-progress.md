# 助攻核心與 60Hz 回歸

## 問題與決定

目前 `SingleLaneConfig.players[2]`、`MobaMatch.heroes[2]` 且兩隊不同，真實單路只能1v1。不能為了測助攻把敵方改成同隊，或把純資料測試稱作多人對局。先建立可重用的權威參與帳本與結算 hooks；接著要擴充同隊 roster 與 owner-private HUD／回城投影，才有真正多人助攻。

- 原創首版規則：Lua `hero_assist_gold=100`、`assist_window_seconds=10`，每名合格助攻固定100 Gold，不從擊殺300 Gold扣款，不實作治療／護盾助攻。
- 只有實際正傷害才記錄。帳本以 generation-packed 受害者生命 ID 與貢獻玩家 ID 儲存最新權威 active-time；同玩家多次命中只更新時間、不累加筆數。
- 同隊有效英雄擊殺時，窗口內參與者排除killer／self／friendly；恰好10秒有效，超過無效。當前隊伍／roster須一致。沒有有效英雄killer的NPC或匿名死亡不發助攻。
- 一次致死消耗整份受害者記錄；Death再次清除、重生新generation不繼承。正傷害為零、暫停／暖場／終局、過期或退休source entity不記錄。
- 活著的助攻者更新Gold storage；死亡者的設計路徑更新persistent slot Gold與assists，供重生保存。**目前1v1不會產生合格助攻者，此獎勵分支尚無真實多人ECS／KCP驗收。**
- 帳本依BTreeMap穩定順序結算，最多10名受害者／每名10筆參與，重複更新在滿額時仍可用。帳本留在authority，不進script ABI／renderer／filtered world；authority replay digest納入規則、assists與帳本。
- Lua→generated Rust常數納入完整內容雜湊；改金額或窗口禁止單邊hot reload／compiled mismatch。整個MOBA section省略的舊TD維持0助攻Gold／10秒預設，有宣告的section需完整欄位。

## 額外修正

`ScriptDirectDamage`沒有source欄位，不能推導攻擊者。它也不經普通Damage的kill hook；匿名致死後Heal再被hero致死原本可能搶獎勵。共同damage hook現在對匿名正傷害／致死只處理生命退休與lethal_pending，不發kill／assist。新增正式ECS outcome回歸；既有TD語意保留。

## 已驗證

- core329 passed，其中5個助攻核心測試涵蓋多人／重複／排序／窗口端點／過期／未來時間／最新hit／錯隊roster／匿名死亡／generation退休／容量。
- base_content81 passed，含新增匿名致死→治療→hero致死不搶獎勵、重複正傷害僅付killer不虛構助攻、相同序列replay；既有60Hz完整雙隊filtered生命周期／逐tick零repair hash與kill/recall回歸通過。
- template63 passed（31 unit＋23 generated＋8 hero＋1 catalog），assist欄位型別／內容hash／compiled mismatch／hot reload拒絕通過；新增無效Gold1000001／窗口0或61的setup原子拒絕通過。
- server153 passed／1 opt-in ignored；runtime54 passed／4 opt-in ignored與main3 passed；bridge50 passed／1 opt-in ignored；Fyrox `cargo check -p omfx --tests`通過。
- 真實KCP `target/interactive-runs/moba-runtime-1791075055`：60Hz、雙隊各9 checkpoints至1080、安全tick1082、實際移動，success與cleanup_verified=true。server73080、runtime37032／90732於保存報告後再次inspect無殘留。**此為2人既有流程回歸，不是多人助攻證據。**
- 錯誤與預防措施見 `unreal-moba-error-register.md` E109。保留原始證據與使用者變更，不commit／push。
- Unreal build-only60465成功；ABI7與C++ surface不變，codegen仍11 files／15 inputs／class hash de9c7fcfc98d6479。助攻規則改的是完整gameplay資料hash，不要求無關角色class metadata改變。bridge staged SHA cfc40ab9b1092487b8bedeb69638b9d53d13be8488756e525f61950a34eab9da一致；本輪未啟動Editor或聲稱助攻UI通過。

## 下一步

1. 將固定每隊一人的slot與配置擴充為明確player→team roster，保持lane side與player index分離。
2. 將同隊多名英雄的private HUD／recall metrics改成owner-scoped，不能讓一名英雄讀條鎖住整隊。
3. 三名以上真實ECS人物走正式Damage／死亡／Gold／assists／重生，再逐tick驗雙隊filtered與fresh bootstrap；之後才補KCP／UE計分板與多人助攻。

5.3仍未完整完成；本輪不新增英雄專屬C++、Blueprint graph或尚無契約的KDA UI。
