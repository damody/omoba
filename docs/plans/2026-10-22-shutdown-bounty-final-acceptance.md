# 2026-10-22 連殺終結賞金最終驗收

OpenSpec `complete-moba-shutdown-bounty` **5/5完成**，接續全遊戲17系統盤點，優先單台電腦可驗證內容。

## 實際實作

Lua economy新增kill_streak_bounty門檻／首額／每殺增額／上限。正式訓練政策3連殺起150金，每殺加100，最多700；是目前玩法數值，不宣稱複製LOL特定版本。缺欄舊內容關閉，型別／門檻／金額有界，canonical hash與compiled／hot reload政策要求全部peer重建；正式server明確選用generated政策，legacy config保持關閉。

權威player roster保存連殺，成功敵對英雄致死且獎勵入帳才增加；第一次致死取受害者額外金並清除連殺，匿名／環境／強制死亡亦清除。沿既有召喚／每發投射物來源歸因與已接納死亡錢包，重複致死不重付，助攻金／XP不變；正式MOBA通用DeathTick不再重發Unit預設附近金／XP，死亡事件及legacy獎勵保留，狀態與政策納入digest。

公開score metric發布可領額外賞金，不從總擊殺／私有經濟／可見entity猜測。protobuf additive欄位同步checked-in生成碼，舊snapshot無欄為零，client拒絕重複／錯誤audience／非法金額；C／Rust ABI32、24-byte scoreboard row到Unreal payload，實際計分板顯示「終結賞金 +… 金」，零值清理、非法值閉合。

## 最後集中驗收

RustV4 **1721通過／10既有ignored／0失敗**：template62、core864、codegen57、base256、backend190/1、bridge128/1、client149/8、ABI15；另generation成功。9工作exit0／退休。

新增內容型別／界限／缺欄相容／hash／重建保護，實際傷害連續擊殺與一次終結、匿名／強制死亡、pause／phase／digest、退休召喚物投射物支付死亡錢包並重生保留、正式60Hz generated技能及真實PostStep公開metric；client跨隊伍與protobuf codec、bridge typed HUD跨到ABI與大小驗證。Unreal檢查新額／零值／非法額及實際計分板渲染。

V1核心兩個新fixture失敗：延後Death未drain完就手動重生，及8連殺應650卻誤當700；正式60Hz技能測試另誤要求同step退休entity，改驗當下致死／公開賞金零值及下一step退休。修正測試生命期／改9連殺驗cap，不更動正式規則。等待V1及watcher退出後重跑整套V2，保留V1所有原始log。WorldV1的PNG只有外框未建資料列，目視判為失敗，修正Slate fixture於TakeWidget後設資料並檢查真實row；build-v2／WorldV2及新來源基準包含最後測試修正。保留舊PNG／report／建置與被撤回的來源檢查，不把中間automation成功當視覺接受。正式規則未因fixture失敗而放寬。V2正式技能測出下一step通用death_tick又多發10金，真正修正為正式MobaMatch跳過通用Unit附近金／XP但保留Death，legacy模式保留舊獎勵；新增system回歸，build-v3／RustV3／WorldV3及第三份來源基準含此正式修正。V2失敗及舊建置／畫面／來源檢查全部保留。

UE5.8 **build-v3成功（UBT 3.94秒）**，後端建置與Lua三契約成功。WorldV3 **36成功**（35 Success＋1 SuccessWithWarnings／4個既有preview-world清理警告，0 Failed／NotRun）。兩map各74影格／6片段；兩map輸出與新增原生賞金計分板共三張已目視。屬本機公開場景／UI fixture，不冒稱新真對局或網路ACK接受。

**3350來源／資產**集合、SHA、size、mtime在驗收前後一致；最後比對等待最終Rust與capture worker／watcher退休。current build／staging／DLL及import library另驗。**137份原始證據**逐byte／SHA封存獨立複核，三分鐘監測 **18次／最大181秒**，另讀log確認；21 workflow worker＋21 watcher及全程monitor退出。

V3正式賞金回歸已通過，但既有三路最後seed在400秒game-time預算尚未結束；移除重複XP後成長變慢，塔數仍下降且每step雙隊hash一致。authored-map fixture改明確20分鐘／依fps換算的有限預算，保留所有原勝負／死亡重生／無repair／hash／game.end／凍結斷言。最終整套RustV4與第四份來源基準包含此測試修正，正式程式與build-v3／WorldV3不變；V3失敗亦保留。最後seed於tick24224結束、tick24238完成凍結接受，實際約404秒。

## 仍待完善

連敗金額折價、首殺、全隊經濟差補償及更多終結公告。其他缺口：持續引導／角色施法動畫／特殊幾何、完整召喚複本／競爭落點整組回滾／退休callback、objective永久stack／傳承／小兵光環／分裂、塔皮與美術。真實連線賞金畫面／ACK與其他MVP外部矩陣仍待驗，原MVP維持33/44／11。

錯誤B01–B08、全遊戲盤點與handoff已更新。未新增網路對局／重設配額／commit／push，不維護omfx；未重跑209 native／458 IPC／69 Blueprint／五PCM，不混算。
