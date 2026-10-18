# 原生動畫事件生命期實作計畫

角色組裝、材質與idle／move／attack基礎驗證後，接父項6.4；本文件不代表已實作或接受。

1. 盤點實際frame／animation snapshot／ability cue／damage cue入口與已揭露身份。原生hero與既有Blueprint override共存，只有原生呈現啟用的actor接新動畫。
2. 以公開HP frame建立死亡／重生狀態。首次frame只seed生命狀態，不重播重生；嚴格tick／sequence與entity generation，死亡pose不被下一張idle覆蓋，重生完成回到最新locomotion。
3. 由真正已接受的新ability／damage cue驅動cast／hit。分類由內容metadata或既有generic cue契約決定，不硬寫hero ID、不將被拒絕的本機指令視為施法。去重cache有界、同source清理、hidden不播放。
4. 定義明確優先序與可恢復snapshot：death、respawn、cast、attack、hit、locomotion。短暫clip依實際播放cursor結束後恢復最新狀態，不用HUD倒數或本機成功假設。pause、focus／fog、pool retirement及重連不得重播舊事件。
5. Recall只消費正式權威公開狀態／事件。現有ABI的recall_remaining_seconds僅在owner HUD，不能據此推斷所有英雄回城；若沒有公開生命期接線，先補共用Rust／ABI／Unreal projection，再接loop／cancel／complete，保留gameplay拒絕與權威座標。
6. 增加原生真actor事件驗證與實際clip／CPU skin／frame captures，覆蓋重複、過期、死亡途中施法、重生、pause、hidden、pool、來源切換與既有Blueprint fallback；完成當前功能後再做完整跨入口驗收。

正式熱路徑不逐frame載入資產、建材質或掃描全世界。每個actor只保留有界狀態；GPU／streaming阻塞僅在獨立驗收capture fixture中，不搬進遊戲tick。

## 2026-10-12 實際接線與下一範圍

已修改尚待建置驗證：frame／ability payload 傳遞 view epoch；ability event 的 `kind == AbilityCast` 明確分類，不由 art cue 名稱猜測。hero 保留最新 snapshot，死亡與重生優先，cast 高於 attack／hit。cast／damage 共用64筆紀錄，淘汰時保留每類 tick 退休下限，避免另一類事件塞滿後舊 cue 再次被播放。新真 actor 測試及六角色四種生命期 capture 已加入。這些尚不代表6.4完成。

回城一般解將使用可見 entity 的獨立安全 component，不把 team HUD 擴散給敵方。現有 `visibility.rs` 建立每 entity disclosed component；`secure_replica_contract.rs`、`team_projector.rs`、`filtered_specs.rs` 要同時維護白名單、嚴格 codec 與 malformed 拒絕，再由 `EntityRenderData`／bridge animation projection 傳入 Unreal。公開內容只需已開始 channel 的動畫狀態，不包含原點、目的地、私有指令、HUD 倒數或未揭露 entity；不從玩家 ID 推測另一個 actor 正在回城。權威 `MobaMatch::is_recalling(entity)` 已存在，可在正式 snapshot 階段產生狀態。

回城 component 的清除是權威 cancel／complete 結果，Unreal 應回到最新 locomotion；完成座標由既有 frame 決定。pause、hide、epoch、generation／pool 必須清除舊呈現。新增 schema 時同步協定版本與相容拒絕測試，保留正式 Rust／Unreal 消費端，歷史 omfx 不補接線。這部分尚未修改共享協定，須待當前 cast／hit／death／respawn 範圍驗證後接續。

## 公開回城目前實作中，尚未接受

`channel-visual-rust-v5` 已實際通過8項定向Rust測試與五正式all-target checks。真 Recall／Stop 命令測試額外走過正式 `build_wave_b_read_view`，驗證其 disclosed baseline 的 Recall→None；不是只測getter或手刻component。v4測試編譯失敗保留於 E857。`native-recall-header-v1` 已實際通過 ABI26 MSVC header smoke。普通 Unreal 建置與當次原生／原圖審查仍進行中，尚未接受。

普通 `native-recall-build-v1` 的 IO 觀察：UHT完成後65 actions，前三個 unity 的 cl CPU很低但working set持續增長；當次效能counter取樣 D 槽約5,497,964 read bytes/sec、179 reads/sec、queue3，cl page faults/sec 約1002／129／541。使用的實際 shared PCH 長2,574,581,760 bytes。這是建置期間的觀察，不是編譯失敗或遊戲效能證據；沒有因此修改／刪除 Engine、PCH、cache、BuildId 或中斷仍在前進的編譯。既有三項 ABI gate `native-recall-abi-gates-v1` 實際全部通過。

生命期第一範圍已接受後，新增 `ChannelVisualState` schema `0x464f4713`／固定五byte `CVS1`＋None或Recall，FactKind34；只攜帶已揭露 entity 的當前動畫狀態，不含來源／目的地／私有倒數。visibility 每snapshot先建立有界hero狀態表，不逐entity掃名冊；projector／filtered replica／allowlist同時維護，Selective升8並拒絕7。`channel-visual-rust-v1` 實際四項channel測試與一項真recall／Stop命令測試通過。

下一段已修改待驗：Rust bridge 只解碼當前hero的合法 component，活體Recall清掉舊attack／target／phase資料，以ABI26的新animation flag傳入Unreal；owner HUD仍是原本隊伍私有狀態。原生hero channel loop取代cast／hit／move，死亡／respawn優先，權威clear恢復最新movement，hide／pool不恢復舊channel。既有native真actor與ABI端到端測試擴充，gallery新增六張recall，當次預期46PNG。尚未建置／native／畫面接受，不宣稱6.4完成。


2026-10-12 原生動畫生命期第一範圍接受：`.ai-collab\native-lifecycle-accepted-v1\summary.json`。ordinary NoEngineChanges／NoUBA／nocache成功，fresh十項原生全部Success／零warnings；真ABI cast分類與view epoch、公開HP死亡／重生、cast／attack／hit仲裁、最新snapshot恢復、64筆去重退休下限、pause／hidden／pool／generation隔離已驗。六角色idle／attack及cast／hit／death／respawn共36張，加四來源診斷，共40張actual PNG逐張審查；沿用已驗RetargetV4／RigidPartsV1／MaterialsV1 packages並再驗SHA與mtime。E851–E855原失敗保留，首次8／10及第二次被診斷球遮住的畫面未當接受。attack／cast／hit診斷線框須明確開啟，正式事件與動畫保留。public recall／完整VFX／Saika擴充生命期／服裝武器及世界畫面仍pending，6.4與父項12／44不勾完；無新權威對局／ACK。


2026-10-12 公開回城當前範圍接受：`.ai-collab\native-recall-accepted-v2\summary.json`。Selective8／ABI26，真Recall與Stop經正式visibility producer、projector／replica／bridge，8項Rust、三ABI gate與五正式all-target checks；ordinary NoEngineChanges／NoUBA／nocache與十native Success／零測試warnings，46原始PNG逐項核對（36份與已審原圖byte相同，10份新／變動圖另審，詳review）。loop／取消／death／respawn／pause／hidden及既有fallback已驗，無新權威對局／ACK；完整VFX／Saika擴充／服裝武器／世界與完整驗收仍pending，父項12／44及6.4不勾完。
