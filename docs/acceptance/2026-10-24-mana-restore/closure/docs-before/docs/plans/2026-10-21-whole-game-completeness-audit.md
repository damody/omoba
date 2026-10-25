最新彈體首碰增量：2026-10-24 projectile-contact；移除中點nearest16截取，以同一空間查詢的所有候選及共用整數Q32接觸時間選最早者，同量化按id/generation穩定排序；缺Pos／Creep的舊索引列拒絕。四種索引、密集離軸目標、入口／相切／raw邊界與翻轉順序通過，無新全世界掃描。FreshRust九套2093／9ignored、focused11、backend／UE5.8／staging／catalog、native cue2／0warnings成功；3487來源一致（3484未變／3變更），151份證據獨立重核。詳見 docs/plans/2026-10-24-projectile-contact-final-acceptance.md。維持creep碰撞政策，沒有新增技能／截圖／真網路對局；MVP33/44、11外部項不變。

最新共用彈體修正：2026-10-24 projectile-flight；移動／掃掠共用剩餘壽命與整數端點，修正最後tick越過終點、過量壽命移動及不足1單位／raw1零速提前命中。既有瞄準端點沿同一整數機制且保留原限制，沒有名稱特判。FreshRust九套2082通過／9ignored、focused5、backend／UE5.8／staging／catalog、native cue2／0warnings通過；3487來源一致（3485未變／2變更），157份證據獨立重核。原始失敗與退休保留；詳見 docs/plans/2026-10-24-projectile-flight-final-acceptance.md。沒有新增技能／截圖／真網路對局；更多飛行技能／MOBA skillshot碰撞與MVP33/44、11外部項仍待。

最新召喚物 UE 操作增量：2026-10-24 summon-ui；exact owned living summon 選取、Alt+右鍵指定移動／Alt+S恢復自主、可保存遷移快捷鍵與實際綁定提示。獨立型別輸入保留主人＋召喚物 opaque life，client／bridge 目前來源守門；presentation12／selective14／ABI36。active UE 橋接建置改走固定Lua。FreshRust1999/0失敗/9ignored、focused4、backend／UE5.8／DLL-import／catalog、native9/0warnings及實際Slate截圖目視通過。3487來源一致（3457未變／25變更／5新增），1269份證據獨立重核。UI01–UI13失敗與12工作原始exit／退休保留。詳見 docs/plans/2026-10-24-summon-ui-final-acceptance.md。未新增真網路對局；MVP33/44與11外部項不變。

最新召喚物傳輸增量：2026-10-24 summon-wire；opaque雙參照經secure session解析、正式精確owner權威再檢查、canonical ACK遮蔽與notification-only replica，client host建構API；selective14、presentation11。Core4/client1/backend2/actual formal1與paired replay通過。FreshRust1996/0failed/9ignored、UE5.8/backend/staging/catalog與3482來源一致，失敗證據封存。UE issuing UI仍pending，未宣稱玩家可操作。詳見 docs/plans/2026-10-24-summon-wire-final-acceptance.md；MVP33/44與11外部項不變。

最新召喚物增量：2026-10-24 owned-summon-authority；獨立精確owner/summon世代MoveTo/Release、32/tick上限、生命／隊伍／控制／地形權威、重用導航與解除後自主恢復，主人真實引導隔離及paired replay通過。FreshRust1990/0failed/9ignored、focused5、backend/UE5.8/staging/catalog與3481來源一致。UE issuing/scoped transport仍pending，入口明確拒絕，未宣稱玩家可操作。詳見 docs/plans/2026-10-24-summon-orders-final-acceptance.md；MVP33/44與11外部項不變。

最新bot施法增量：2026-10-24 bot-channel-intent；封閉SelfChannelRecovery與compiled/runtime執行投影／peer與reload守門，僅依公開威脅、自身生命與精確host資源開始。可選十席學徒中路配裝／共用graph篩選學習；實際5Hz→60Hz四rank施放／脈衝／root／pause／CD／pairedreplay通過。FreshRust1986/0failed/9ignored、focused4、backend/UE5.8/staging/offlineplan/catalog與3480來源一致；失敗原始證據封存。詳見 docs/plans/2026-10-24-bot-channel-intent-final-acceptance.md。接續盤點玩家召喚物選取／命令操作；MVP33/44與11外部項不變。

最新bot行為增量：2026-10-24 bot-channel-wait；共用決策等待自身真正技能／道具引導，完成或中斷後恢復，假狀態與其他owner不影響。實際正式60Hz脈衝／root／pause／Stop／pairedreplay、FreshRust1982/0failed/9ignored、backend/UE5.8/staging與3479來源一致通過。詳見 docs/plans/2026-10-24-bot-channel-wait-final-acceptance.md。下一優先是封閉的自動引導技能intent與安全啟動；本批沒有宣稱自動開始channel政策。MVP33/44與11外部項不變。

最新引導玩法增量：2026-10-24 authored-recovery-skill；既有學徒 R 由資料驅動站定 2 秒／0.5 秒四脈衝，原子 MP/CD、命令／受傷／控制中斷、root/pause/replay 及共用提示已實作。FreshRust1980/0failed/9ignored＋focused12＋Lua37、backend/UE5.8/staging/catalog/native6（0warnings）、3479來源一致與圖像檢視通過；七個原始 workers 含失敗版完整保留。詳見 docs/plans/2026-10-24-recovery-skill-final-acceptance.md。下一優先是 bot 的封閉引導意圖與等待／恢復，尚未宣稱 bot channel 支援；MVP33/44與11外部項不變。

# 正式遊戲完整度盤點與單機優先清單

最新提示增量：2026-10-24 summon-guidance；召喚策略自動產生集火／合法索敵／停止後回退說明，所有技能共用 registry Description→身分綁定HUD payload→提示接線，維持模組分層並保留升級／施法資訊。共用C++轉義支援多行／控制字元。Rust1973來自V4，V6完整來源與精確單行test-only差異證明沿用；Lua19、fresh backend/UE5.8/staging/catalog/native6（0warnings）及三等級Slate圖像檢視通過。詳見 docs/plans/2026-10-24-summon-guidance-final-acceptance.md。六版成功／失敗證據保留；MVP33/44、11外部項不變。

最新玩法增量：2026-10-24 summon-owner-focus；封閉資料策略預設nearest、可選owner_focus，既有鐵炮兵啟用後會協助主人目前明確攻擊的合法目標，失效／隱藏／越界時沿既有索敵回退。真正formal AttackTarget→雙召喚物射擊／90實傷、Stop→最近敵人、paired replay通過。FreshRust1972＋focused3、codegen/backend/UE5.8/DLL-import staging及權威↔generated catalog hash核對成功；3476來源一致、10變更／45候選明列。詳見 docs/plans/2026-10-24-summon-focus-final-acceptance.md。未新增pet指令UI或native畫面重跑；MVP33/44與11外部項不變。

最新召喚物增量：2026-10-24 summon-arrival；共用exact Entity/current Tick host移動歸屬補齊抵達當tick的第二次腳本追敵，目標移除仍停在終點，下一tick恢復追擊。原生私有phase-local紀錄不增加wire／序列化導航欄位。FreshRust1969＋focused3、實際召喚／正式60Hz／paired replay、backend與UE5.8／DLL-import staging通過，3476來源一致。詳見 docs/plans/2026-10-24-summon-arrival-final-acceptance.md。玩家召喚選取／命令UI、更多channels／multistage仍pending；33/44與11外部項不變。

最新召喚物增量：2026-10-24 summon-priority；共用script adapter讓仍有效的host MoveTarget優先，避免命令／跟隨移動後再被追敵橫向加走；清除目標後恢復自主追擊。實際召喚技能→combatant→正式60Hz移動與paired replay、adapter／root／非召喚物回歸通過。FreshRust1968＋focused2、backend與UE5.8／DLL-import staging成功，3476來源一致；未宣稱本批native畫面重跑。詳見 docs/plans/2026-10-24-summon-priority-final-acceptance.md。到達當tick的共用步長及玩家召喚控制UI仍pending。

最新離線玩法接線：2026-10-24 line-bot；可選十席配裝讓雙方中路使用直線Q並啟用魔力，實際Lua→嚴格plan→5Hz bot決策→正式60Hz命中。四rank／視野隱藏與私有位置隔離／禁錮／資源／端帽與寬度／paired replay通過。新鮮Rust1966、focused1、Lua23與離線plan-lock成功，3476來源一致；UE/native93/77影像沿用前批已封存證據，未宣稱本批重建。詳見 docs/plans/2026-10-24-line-bot-final-acceptance.md。MVP33/44與11外部接受不變。

目前最新完整驗收：2026-10-24 linear-skill；共用瞬發直線膠囊、固定小數截短、9英雄直線配裝、公開觀察bot選點、非位移／穿透地形提示及正式60Hz命中／防禦／資源／重播。Rust1965＋sim2／Lua23／backend／UE5.8／native93／3475來源與77張目視證據通過。詳見 docs/plans/2026-10-24-linear-skill-final-acceptance.md。其他玩法／美術及11外部接受仍pending。


最新玩法增量：2026-10-23 periodic-damage補DoT有效生命期積分、每buff次量子餘數、來源／kind/profile隔離及穩定到期順序；Lua通用敵方DoT接遊俠10/15/20/25 DPS、4秒流血與公開canonical狀態，與減速分開且隱藏私有來源。V2 Rust1771passed/9既有ignored、backend／UE5.8及World66成功；165份證據／29PNG、3425輸入一致、health 13 samples／max171秒，獨立校驗通過。詳見2026-10-23-periodic-damage-final-acceptance.md。新發現共用script／DoT raw damage缺少基礎護甲／抗性，而DamageInstance已先計算；NEXT優先raw／settled共用結算與防double-mitigation，然後持續引導／zones。本批不宣稱armor parity或獨立地面區域完成。


最新玩法增量：2026-10-23 cast-transactions補原生施法結算封套與受支援召喚整組預檢／提交：後段拒絕不生成、不扣法力／冷卻、不發成功提示；所有腳本法力回呼（含on_tick）用原始法力池一致性檢查與同批通知，避免失敗前筆的費用被後續絕對寫入夾帶。現有四rank召喚、late-terrain、資源依賴及多owner測試通過；Rust1523／9既有ignored、World66、backend／UE5.8成功，3423輸入一致、153證據／29PNG獨立校驗。詳見2026-10-23-cast-transactions-final-acceptance.md。當前作者召喚＋施法者資源／朝向已整組結算；任意混合地形／生命副作用的召喚仍不支援並整筆拒絕。後續優先持續引導／zones。

最新呈現增量：2026-10-23 objective-banner將已公開目標事件增加為獨立18..36px可讀橫幅，shared typed selector、authority expiry／reset／退休／HUD顯隱立即更新，沿既有header寬度與文字縮放，無localtimer或推測成功。UE5.8 buildV2／World66／1Rust export成功，3421輸入一致、129證據／29PNG審閱與獨立校驗，保留V1visibility失敗原始來源／日誌。詳見2026-10-23-objective-banner-final-acceptance.md。既有首殺下方HUD裁切、部分名牌重疊未修復；接續優先channels/zones或整組召喚玩法交易。

最新一致性修正：2026-10-23 summoner-coherence 以完整shared目錄序列化補啟動peer gate與開發重載守門，拒絕作者／編譯期召喚師漂移並保留舊generation/hash，合法傷害調整仍可重載。Rust1419全部通過、backend建置及DLL部署一致、3420來源一致、55證據獨立校驗。本批Rust-only，未重跑UE。詳見2026-10-23-summoner-coherence-final-acceptance.md。接續優先具體channels/zones或整組召喚交易，而非重複目錄／名牌複核。

最新增量：2026-10-23 summoner-shield新增第三個正式可選護障，catalog3／160護盾／2秒／180CD由Lua提供；common SelfShield plan/ECS sink、獨立D/F冷卻與既有shield生命期，控制可用、禁療不縮減、不取消移動。generated SummonerCatalog與typed作者名稱／說明接實際tooltip。Rust1572／8既有ignored、World65、六Lua套件、backend／UE5.8成功，3420輸入一致。詳見2026-10-23-summoner-shield-final-acceptance.md。専用圖示／美術、channels／zones／wholeformation回滾與11外部接受仍pending。

前批增量：2026-10-23 summon-settlement補每實體的living-owner／世代／隊伍與陣營種類／optional玩家歸屬／生命期結算守門；拒絕不發Spawn hook或成功fact，重複結果不覆寫或刪除已有單位。930 Rust、World64、backend／UE5.8成功，3416輸入一致。詳見2026-10-23-summon-settlement-final-acceptance.md。整組召喚資源原子回滾仍pending，不將per-entity接受當作whole-cast接受。

前批增量：2026-10-23 recall-progress 新增共用回城秒數default／validate／generated getter與typed HUD total，實際header共用read-only Slate進度條只讀權威剩餘／總秒數；半途首次快照、pause不外插、零不冒稱完成、舊tick／sequence、非法數值、死亡／換生命／重設／退役均有驗證。Rust針對本批190、World64、backend／UE5.8成功，3415輸入一致；詳見2026-10-23-recall-progress-final-acceptance.md。一般技能持續引導／zones／settlement及11外部接受仍pending；回城bar不替代那些功能。

前批增量：2026-10-23 item-feedback 新增主動道具共用的當前slot/catalog冷卻、公開中心射程及敵我／可攻擊條件提示；地點道具的正式效果半徑接入既有世界瞄準圈。metadata來自schema，無道具名稱特判；來源／目標世代、隱藏、替換、非法數值及退役清除。保留道具原有規則，不套技能mana／沉默／定身限制。codegen66、World63、UE5.8成功，3413輸入一致；詳見2026-10-23-item-feedback-final-acceptance.md。channels／zones／settlement與11外部接受仍pending。

前批增量：2026-10-23 upgrade-preview 新增所有技能共用的逐rank基礎魔力／冷卻／施法準備／射程資料、typed升級狀態與目前→下一級比較；未學習、等級不足、無技能點、滿級、死亡／未知／重設／退役均有明確處理。實際+按鈕與一般技能tooltip共用，既有權威升級門檻不變。針對本批codegen65通過、World62成功、UE5.8成功，3412輸入一致；不宣稱重跑全部Rust或完成channels／zones／settlement及11外部接受。詳見2026-10-23-upgrade-preview-final-acceptance.md。

前批增量：2026-10-23 aim-coherence 以同一共用投影產生native geometry並封存compiled/runtime signature；射程、有效area、swept width、formation數量／間距／前距與契約變更在啟動／reload拒絕，實際reload拒絕保留原generation/hash，非幾何damage調參可接受。Rust1976／10既有ignored、World60、UE5.8及backend成功，3411輸入一致。詳見2026-10-23-aim-coherence-final-acceptance.md。其他非幾何metadata同步仍依原政策，channels／zones、settlement、replica與11外部接受pending。

前批增量：2026-10-23 formation-preview 新增共用facing_formation作者契約與逐rank預檢／generated metadata；公開owned facing、<=128請求位置、隱藏／替換／退役清除及實際tooltip。Rust1973／10既有ignored、World60、UE5.8及backend成功；3411輸入一致。詳見2026-10-23-formation-preview-final-acceptance.md。預覽不是地形調整後的出生接受；整組settlement、channels／zones、replica與11外部接受仍pending。

前批增量：2026-10-23 summon-formation 補齊共用逐rank陣形預檢、固定點朝向座標、有界數量／間距／生命期及living-owner守門；保留2/4/6/8召喚與45/50/55/60秒，修正自身朝向說明。Rust1970／10既有ignored、World59、UE5.8及backend成功；3409來源／資產一致。詳見2026-10-23-summon-formation-final-acceptance.md。既有managed cast回滾驗證不等於後續settlement整組原子接受；持續引導／特殊幾何、combat replica與11項外部接受仍pending。

前批增量：2026-10-23 hero-reservation 補齊可見英雄名牌的固定64保留足跡，一般非英雄名牌避讓，選取／指向可跨英雄保留區但仍互相避讓；隱藏、退役、世代與移動恢復共用實際Slate量測。Rust1966／10既有ignored、World59成功、backend及UE5.8成功；3407來源／資產前後一致。詳見2026-10-23-hero-reservation-final-acceptance.md。英雄彼此、焦點例外及超過64英雄保留上限仍有限制；持續引導／區域、特殊幾何、召喚整組commit及11項外部接受維持pending。48h無人值守omoba-48至2026-10-25 00:33:38 Taipei。

前批增量：2026-10-23 nameplate-spacing 補齊公開非英雄名牌的固定64候選螢幕避讓、選取／指向優先、目前mesh錨點、真Slate隱藏量測及分離後恢復。Rust1966／10既有ignored、World58成功（4既有warning）、backend及UE5.8成功；3406來源／資產前後一致。詳見2026-10-23-nameplate-spacing-final-acceptance.md及../acceptance/2026-10-23-nameplate-spacing/index.json。英雄名牌交叉碰撞仍待處理；持續引導、特殊幾何、召喚整組commit及11項外部接受維持pending。48h無人值守omoba-48至2026-10-25 00:33:38 Taipei。

前批增量：2026-10-23 transform-buff 補齊火繩銃與三段擊的共用嚴格資料預檢／living-self commit，省略值只取相同 authored rank，錯誤值不再靜默套空 buff。保留七個 rank 的平衡值、Q10／normalized 編碼、refresh／expiry 與視覺連發單次命中。Rust1966通過／10既有ignored、World56成功（4既有warning）、backend及UE5.8成功；3404來源／資產前後一致。詳見2026-10-23-transform-buff-final-acceptance.md及../acceptance/2026-10-23-transform-buff/index.json。持續引導／區域、特殊幾何完整預覽、召喚整組commit與11項外部接受仍未完成。

前批增量：2026-10-23（22日開工批次）passive-cone 補齊真正普攻命中派發被動、受擊點前向扇形、技能／視覺子彈排除、退休英雄來源與逐級資料校驗，並用明確metadata修正被動tooltip。Rust1962通過／10既有ignored、World56成功（4既有warning）、backend與UE5.8成功；3402來源／資產前後一致。完整結果見2026-10-22-passive-cone-final-acceptance.md及../acceptance/2026-10-22-passive-cone/index.json。持續引導、主動特殊幾何、召喚整組commit與11項外部接受仍未完成。


前批增量：2026-10-22 summoner-feedback 補齊 D/F point summoner 的公開控制／原地位移／當前slot冷卻提示與落點規則 tooltip，並在權威模型／真ECS queue拒絕原地閃躍、不扣CD或清命令。Rust1952通過／10既有ignored、World55成功（4既有warning）、backend及UE5.8成功；3398來源／資產前後完全一致。詳見2026-10-22-summoner-feedback-final-acceptance.md及../acceptance/2026-10-22-summoner-feedback/index.json。這是既有PointBlink契約的通用slot／catalog方案；更多summoner operation、道具位移提示、持續引導及召喚整組commit仍待補齊。

更新日期：2026-10-23。範圍為omb、共用Rust／內容生成、scripts與唯一前端omfue。這是功能／接線層盤點，並非逐行程式證明，也不宣稱複製特定LOL版本的全部規則、內容或美術。omfx不維護。既有完整MVP仍33/44；11項缺外部ACK／LAN／實際OS DPI／效能證據，不能因本機UI成功就勾完。

## 各系統現況

| 系統 | 已有接線／證據入口 | 尚需完善或接受 |
|---|---|---|
| 遊戲模式與生命周期 | native/moba_match、SimulationDriver；warmup／playing／finished、死亡／重生、暫停、recall與結算 | 更完整的競技局規則與觀戰／回放產品流程；既有reconnect仍有外部接受缺口 |
| 英雄與成長 | generated hero catalog、ability_runtime、award_live_moba_xp；正式九個英雄配裝（含可選方向散射配裝）、rank／技能點、屬性／魔力／Buff；生成目錄驅動9英雄配裝的主動及終極技能原生動畫、VFX、死亡／重生／視野退休生命週期驗證 | 原創英雄內容量與專用機制、美術；完整天賦／符文另立規格，不以屬性欄存在當已完成 |
| 技能執行 | scripts/base_content與native pipeline；共用傷害／治療／CC／資源交易、前搖／取消／釋放、普攻被動、召喚整組接受與導航；綁定與獨立地面區域、主動扇形、瞬發直線膠囊與共同截短／公開觀察bot選點、死亡與到期守門 | 更多移動／碰撞／追蹤彈體、多段／通用引導內容與召喚控制；完整可見性戰鬥複本生命期與真網路仍需外部接受 |
| 技能操作與提示 | OmCombatAimModel／CommandBar共用三施法模式、改鍵／自用、單位／圓形／路徑走廊／朝向陣形／來源扇形預覽；逐rank法力／冷卻／前搖與既有引導進度、方向缺失／控制／資源未知提示及tooltip；畫面座標與繪製層修正 | 真連線owner ACK／實體輸入接受、尚未實作的技能形狀與更多具體內容；預覽不代表權威地形調整或出生接受 |
| 戰鬥與命令 | 共用命令／護甲／歸因／失手閃避MISS／暴擊傷害與實際扣血後CRIT；普通吸血、技能與正式吸血／暴擊裝備；合成／購買及出售撤銷／重生、目標隱私與有限文字池 | 更廣技能內容、命中／幾何玩法、完整combat複本及真ACK |
| 兵線 | moba_maps.lua／moba_match；兩三路map、authored近戰／遠程／砲車、超級兵、route、索敵、護主、推塔時護主例外；新增生成時HP／攻擊時間成長與cap | 依兵種更細的平衡曲線、局勢差距補償與專屬機制；既有行為保留回歸 |
| 防禦建築 | lane tower layers、base_structures；前置開門、inhibitor／nexus towers、死亡／重生、super wave、同英雄連續攻擊壓力；每生命隊伍／附近摧毀金、新增整場首塔一次額外金與持久depth分層獎勵 | 新增外塔限時分段金／破層護甲；新增通用塔皮分段名牌／倒數；新增權威塔／兵營／基地塔公告、有界佇列與native HUD；建築美術與更細競技平衡仍待補，不把首塔／分層金當成全建築規則 |
| 地圖空間 | templates/moba_maps.lua、moba_map_layout、OmWorldBridgeActor；左下／右上基地、三路、河流、14牆、兩張map、導航／碰撞 | 地圖編輯工具的任意多邊形、跳牆／高低地專用規則與更完整美術地形 |
| 視野與草叢 | visibility、fog_grid、brush_vision、ward_world；range／LOS／stealth／scope、連通capsule草叢、普攻兩秒揭露、探索霧格 | 任意多邊形authoring；更多偵測／掃描類內容與全流程UI；不能把草叢外框或視野未知當可見 |
| 野區戰鬥 | jungle.rs；16營地32初始主體、group aggro、patience、soft／hard回頭、回血、重生、每body獎勵與LOS；資料指定隊伍Buff／死亡政策；新增PlayerOwner／8-link召喚鏈歸因、致死快照、team-only目標效果與非英雄仇恨 | 已接每發來源快照與命中獎勵歸因；新增限時隊伍疊層與內容封頂；新增個人營地Buff及剩餘時間擊殺傳承；新增永久史詩累積／多項營地獎勵；新增內容驅動附近友軍兵線攻擊／護甲光環、去重與生命期；新增通用有界死亡分裂樹、待生子體去重、屍體附近碰撞安全出生、完整清營才重生；兩張map雙方石怪營啟用兩代分裂；其他特殊營地機制仍可擴充 |
| 經濟與獎勵 | moba_economy.lua、moba_assist、shop；被動金、擊殺／助攻、尾刀金／附近XP、角色化建築摧毀、NPC觸發與死亡隊員錢包；新增tick統一提交的首塔獎勵／分層金 | 新增內容驅動連殺終結賞金與公開計分板；已接canonical首殺額外金／公開公告；新增跨重生連敗折價／擊殺重設；新增外塔限時逐層本地金；新增全隊累計實得獎勵ledger、內容驅動目標補償／雙門檻／每次啟用池額、塔與史詩根怪canonical分金及公開狀態；新增權威史詩根怪公告／四秒遊戲時間／pause凍結／canonical多目標排序；仍缺更多目標專屬名稱／音效與完整平衡資料 |
| 商店與道具 | 共用生成19項；引導、定身、淨化、吸血、暴擊、通用唯一被動及升級靴；新增獨立購買群組／1至6件上限／合成後計數、鞋靴限1件、原子拒絕與undo守門、buy專用結果16及共用原生說明；recipe／交易undo、欄位冷卻恢復不回溯HP、死亡錢包與重生均已單機驗收 | 更多被動與裝備內容／build推薦及平衡；undo父項指定真權威接受仍待外部驗收 |
| 召喚師與ward | summoner_world／ward_world、loadout、HUD；blink／heal／self-shield／control-cleanse／self-sprint內容、charges／冷卻、team公開與生命期 | 更多原創spell／trinket／偵測內容及護障專用美術；五種現有spell不代表完整競技選項 |
| 鏡頭與輸入 | control profiles、camera、minimap、accessible pointer；改鍵、三施法模式、gamepad合成軸、focus／source退休 | 實體控制器與實際OS DPI／視窗模式接受，合成事件不能替代硬體矩陣 |
| HUD與溝通 | CommandBar、隊友卡／scoreboard／chat／ping／marker／mute／settings／owned scopes；新增通用Buff層數×N、限時／永久效果、只改層數刷新與有界metadata解碼；新增公開damage_shield、獨立護盾條／數值／權威倒數，部分吸收／耗盡／到期／死亡與可見性退休 | 真聊天／ping權威往返、更多狀態細節與新玩法HUD；產品整合仍有父項外部證據缺口 |
| 動畫、音效與美術 | NativeCharacter、animation／combatVfx、original audio／五PCM、字幕、資源生命期 | 專用角色／場景美術、技能專屬動畫、完整戰鬥可讀性與硬體效能；PCM通過不等於人工聽感接受 |
| 技術與驗收 | Rust deterministic tests、owned phased viewport、Blueprint、來源SHA與staging；固定Lua5.4、Rust1.95／UE5.8 | LAN／新真對局／固定硬體負載／長時間單session接受，不能靠重設配額或分段結果冒稱完成 |

上表以實際正式來源和handoff既有接受為依據；「仍缺」限目前盤點入口未形成完整規則／接受的範圍，不宣稱對所有歷史文件做絕對不存在判斷。

## 本輪與後續單機優先順序

1. 本輪：選取目標資訊卡整合公開護盾／控制／塔皮／返回狀態，共用名牌格式與顯示節流；可見單位暈眩／定身／沉默名牌、主要時間與並存效果、生命期／節流；可見野怪committed返回／不可受傷重置名牌與生命期；QWER當前公開暈眩／沉默原因與剩餘時間、指定單位技能的當前公開陣營／可攻擊條件與source／target中心射程提示、隱藏／退休／generation守門；此前World共用有界召喚落點預約、跨event／on_tick不重疊與取消／提交退休；此前明確召喚物公開類別、typed catalog與原生角色／動畫生命期；此前逐rank HP傷害中斷施法、Rust／固定Lua嚴格bool及內容重建守門、shield例外與正式tooltip；已完成的權威目標公告包含committed塔／兵營／基地塔與史詩根怪、每生命去重、canonical排序／最多7待播、四秒scaled時間、pause／到期／終局，public tuple／protobuf／CABI34／native HUD與正式60Hz雙技能接受。前輪抑療與目標經濟補償保留回歸。
2. 可見角色準備／釋放動畫及逐rank HP傷害中斷政策已接；退休來源on-hit腳本快照／唯讀戰鬥查詢與追加傷害／子彈歸因、世代隔離已接；共用精確capsule與碰撞裁切的瞬間沿路衝刺、四rank fire_dash、單次命中與失敗資源回滾已接；現有作者召喚、競爭落點及施法者資源已整組原子接受／拒絕，法力回呼依賴守門已補。接續優先持續引導／脈衝區域及特殊幾何完整預覽；完整combat複本接受、更多控制與任意混合地形／生命副作用交易仍待擴充。
3. 後續：更多目標專屬名稱／音效與平衡資料、召喚通用spawn ledger、特殊技能幾何／導引metadata與實際落點一致性，避免英雄／營地名稱特判。野怪有界死亡分裂已接；其他特殊營地玩法可另擴充。
4. 後續：更多道具／召喚師內容、建築美術／目標專屬呈現、兵種差異平衡與玩法HUD，均以現有map和離線authority ECS驗證。
5. 外部條件：11個MVP父項所需真ACK／LAN／實際DPI／固定效能，維持pending，不新增連線對局來填數。

本輪完成狀態與最新接受數字以各批 final-acceptance.md 及證據索引為準；若實作發現新缺口，更新錯誤MD與交接，不以盤點表代替測試。前輪塔focus／cooldown、建築／首塔獎勵、兵線成長與史詩Buff證據保留。既有33/44是原MVP文件勾選數，不代表所有英雄技能（包含仍待擴充的召喚控制策略）皆已功能驗證。

## 2026-10-22 沿路施放提示接續

新增有校驗的aim_geometry契約、逐rank路徑寬度、Unreal owner意圖capsule與tooltip。射程截短、地形與權威不確定性明示；不依英雄ID或特效推測。集中結果見2026-10-22-path-preview-final-acceptance.md與../acceptance/2026-10-22-path-preview/index.json。完整特殊幾何、持續引導與11項外部接受仍pending。

## 2026-10-22 位移施放提示接續

補typed SweptPath／executable DashToPoint共用movement policy、公開owner root獨立channel、定身／未知／原地落點提示與技能tooltip。普通技能維持root下可施放；不新增local gate或ACK推測。集中結果見2026-10-22-mobility-feedback-final-acceptance.md與../acceptance/2026-10-22-mobility-feedback/index.json。其他單機與外部缺口維持上述盤點。

## 2026-10-22 正式圓形爆發技能接續

新增共用PointBlast嚴格預檢／精確circle／canonical單次傷害與可選控制；炎刃／炎襲接線，非法cast用RErr資源回滾；typed PointCircle供真rank半徑預覽，保持非位移。集中結果見2026-10-22-point-blast-final-acceptance.md與../acceptance/2026-10-22-point-blast/index.json。其他單機及11項外部缺口不因本批勾完。


## 2026-10-23 原始傷害結算接續

完成直接script／DoT／新彈體共用raw-vs-settled與live護甲／抗性、shield一次、TD／歸因及同批死亡guard。1781passed/0failed/9existingignored；3426正式輸入一致；221份證據／29PNG；health15 samples／max171秒；backend/UE5.8/World66成功。完整證據見2026-10-23-raw-damage-final-acceptance.md。沿用既有防禦balance policy，不宣稱全LoL combat parity。channels/zones、更多內容、11外部父項與MVP33/44仍待完成。


## 2026-10-23 持續地面傷害區域接續

完成可共用的施法者生命綁定圓形脈衝區域，炎襲正式接線六次燃燒，metadata／description資料驅動；動態entry/exit、pause、刷新／到期／remove/death與護盾防禦驗證。1790passed/0failed/9existingignored；3427正式輸入一致；148份證據／29PNG；health10 samples／max170秒，backend/UE5.8/World66成功。詳見2026-10-23-ground-zones-final-acceptance.md。獨立zone生命期、持續引導／專用地面資產與其他內容仍缺；11外部/MVP33/44未勾完。


## 2026-10-23 持續區域公開提示接續

玩家角色資訊新增「持續區域」倒數，checked native payload聚合canonical ID/time，不公開內部key與位置；死亡同批清除，刷新／pause／到期／owner/gen及真Slate接受。1795passed/0failed/9existingignored；3428正式輸入一致；117份證據／30PNG；health7 samples／max171秒，backend/UE5.8/World67成功。詳見2026-10-23-ground-zone-status-final-acceptance.md。常駐專用進度條／地面美術、持續引導與其餘11外部/MVP33/44仍待完成。


## 2026-10-23 偵隱飾品接續

新增權威loadout可配置的原創偵隱燈，偵測等級／圓邊界／地形／敵我／摧毀、真admitted queue／driver／pause／expiry／recharge接受；既有tooltip加入作者說明、距離、視野與生命期，真Slate畫面接受。1799passed/0failed/9existingignored；3429正式輸入一致；246份證據／31PNG；health12 samples／max171秒；World69成功。沒有新增對局內選擇UI或圖示，不宣稱完整LOL控制守衛規則；11外部/MVP33/44與持續引導仍pending。


## 2026-10-23 英雄升等提示接續

收到同owner／entitygeneration更高等級時，真HUD顯示短暫中文升等提示；首次同步／重複／倒退／結束／缺列／scope退休不虛構，localpresentation timer不影響gameplay。World70全執行／0failed／0notrun；3432正式輸入一致；101份證據／32PNG；health7 samples／max171秒；UE5.8接受。Rust/backend未變更並繼承1799+9ignored來源證據。11外部/MVP33/44、持續引導與其他具體玩法缺口仍pending。


## 2026-10-23 施法準備時間屬性接續

CastTimePercentage原有key正式接入通用preparationstart，checked固定點計算／sameinstance總時長鎖定／非法拒絕，既有技能正式60Hz效果與ManaCD單次結算驗證；Unreal顯示effective進度與<0.1s正值提示。1803passed/0failed/9existingignored；World71全執行；3433正式輸入一致；260份證據／34PNG；health16 samples／max171秒；backend/UE5.8接受。持續引導脈衝未完成，不把準備時間modifier視為channel。其餘11外部/MVP33/44仍pending。


## 2026-10-23 技能冷卻屬性接續

既有兩項cooldown percentage接入正式dispatch，ledger與outcome同值、release取樣／running不重縮放；checked非法／overflow／上下限與正式60Hz資源及pause驗證。1808passed/0failed/9existingignored；World71全執行；3433正式輸入一致；121份證據／34PNG；health8 samples／max171秒；backend/UE5.8接受。11外部/MVP33/44、持續引導及其他具體缺口仍pending。


## 2026-10-23 魔力消耗modifier接續

正式施法與bot改共享checked成本乘數；非法/overflow不變免費技能，合法折扣／free／加價與前搖release實際結算驗證。1813passed/0failed/9existingignored；World71全執行；3433正式輸入一致；145份證據／34PNG；health8 samples／max172秒；backend/UE5.8成功。11外部/MVP33/44/持續引導等缺口仍pending。


## 2026-10-23 實際冷卻總長HUD接續

sharedHero記錄當次CDtotal，ownerHUD不再用base作running分母；正式10／40秒modifier、pause／serde／invalid／legacy／native half-fraction驗證。1819passed/0failed/9existingignored；World71全執行；3433正式輸入一致；150份證據／34PNG；health10 samples／max171秒；backend/UE5.8成功。有效Mana cost前端預檢/提示仍以base查詢，列為後續具體操作缺口；11外部/MVP33/44/持續引導仍pending。


## 2026-10-23 有效魔力消耗接續

原base預檢錯擋折扣修復；當前權威cost team-scoped tick/binding/unique檢查，ABI35 typed狀態與價格接提示／precheck。1824passed/0failed/9existingignored；World71全執行；3433正式輸入一致；300份證據／34PNG；health15 samples／max171秒；backend/UE5.8成功。11外部/MVP33/44、channel等仍pending。

## 2026-10-23 消耗異常瞄準提示接續

補上同catalog權威cost state3到共用瞄準Evaluate，顯示「魔力消耗無法計算，目前無法施放」及警示色，保留unlearned/cooldown優先；unknown不誤報，角色／技能資訊退休清除。UE5.8成功、World71全執行／0failed／0notrun、3433正式輸入一致、81份證據／34PNG、health max170秒。Rust1824+9ignored明確繼承前批，3430 unchanged＋exact3native changes有SHA證明；不是重跑結果。詳見2026-10-23-invalid-cost-aim-final-acceptance.md。新提示本身以native狀態／文字／顏色及OnPaint接線驗證，未宣稱專用瞄準截圖。持續引導、完整combat replica、11外部/MVP33/44仍pending。

## 2026-10-23 解控召喚師技能接續

新增第四個可配置召喚師「淨念」(catalog4/210秒/self)，共用嚴格資料操作清除目前canonical暈眩／定身／沉默，保留其他Buff、其他單位及移動；無免疫、無控制亦扣CD，未改預設loadout或新增對局內選擇器。plan/commit失敗不扣冷卻，unsupported sink拒絕，正式60Hz輸入及public catalog4/CD/pause/noimmunity驗證；Unreal生成身分與共用卡片／實際中文tooltip畫面接受。

Rust1829passed/0failed/9既有ignored、backend/UE5.8 buildV2/stage/import成功、World72全執行／0failed／0notrun、3434正式輸入一致、250份證據／35PNG、health max171秒；三輪Rust失敗與一次UE建置失敗及精確修正前來源均保留。測試掃描器只排除確切docs/acceptance歷史封存，未刪改證據。詳見2026-10-23-control-cleanse-final-acceptance.md。這是明確有限的原創解控，不宣稱完整LoL Cleanse；channel/完整combat replica/11外部/MVP33/44仍pending。

## 2026-10-23 持續治療引導道具接續

新增可購買「靜心符」500金／30秒CD：站定引導3秒，每滿1秒治療40，啟動不立即治療。通用嚴格ChannelHeal資料與主機生命期支援暫停、重複tick防重、抑療／HP上限、同tick生命受傷先中斷、新指令／控制／位置／owner／生命／階段失效取消；定身與護盾全吸收不取消，禁止自動普攻。共用self-item分類接商店／六格道具／施放命令；中文完整說明與公開剩餘時間已驗，未改wire/C ABI。

Final acceptance: Rust1839 passed/0failed/9existingignored (ABI15/content44/templates116/codegen70/core975/base326/bridge136/client157); backend and UE5.8 buildV2 succeeded; staged DLL/import-library SHA matched. Native WorldV2 executed73:72success+1existingwarningcase,0failed/0notrun. Source consistency verified3436 inputs. All36 PNGs accepted:7 changed/new actual images viewed,29 prior SHA-identical images inherited. Archive contains292 independently reverified evidence files and36 images; 20 health samples/max171 seconds. Six final workers and four historical workers have creation-token-bound retired receipts; launch sessions and the single170-second health worker were reaped. No owned test/build worker remains.

詳見2026-10-23-recovery-channel-final-acceptance.md及E1524錯誤紀錄。原始RustV1／NativeV1失敗、精確修正前來源與test-only修正證明均保留；專用channel進度條／動畫／VFX、完整combat replica及11外部/MVP33/44仍pending。

## 2026-10-23 共用引導進度呈現接續

補上當前存活玩家角色的公開引導進度條：使用既有canonical總時長／剩餘秒數，無UI自行計時；中途首次快照比例正確，重複／舊快照不回捲或復活取消狀態，死亡／換代／scope／退休清除。共用模型與真實HUD樹接線不依英雄／道具名稱，無Rust／wire／C ABI變更。

UE5.8 buildV3/staged DLL/import SHA passed. WorldV3 executed74:73success+1existingwarningcase,0failed/0notrun. Source guard verified3439 inputs; prior accepted Rust1839passed/0failed/9existingignored and backend result are inherited after SHA proof of3432 unchanged prior inputs +4 exact native changes +3 native additions; no new Rust suite is claimed. 520 evidence files and37 PNGs independently reverified;7 changed/new images actually viewed and30 prior SHA-identical images inherited. Health9 samples/max171 seconds. Four final owned workers and seven historical workers have retired creation-token receipts; every launcher, all three inheritance readers and the single health monitor were reaped; CIM found no owned worker remaining.

真實Slate截圖為獨立共用元件，未宣稱對局完整畫面／動畫／VFX完成。E1525保留新測試TArray夾具崩潰及原始證據；詳見2026-10-23-channel-progress-final-acceptance.md。引導動畫／VFX、完整combat replica與11外部/MVP33/44仍pending。

## 2026-10-23 公開複本引導總時長接續

既有owner-team Buff元件／事件加入嚴格BVS2 typed total，保留BVS1解碼；主機producer、baseline/update檢查、filtered橋接與原生共用進度串通。只公開canonical catalog／remaining／total，無來源／位置／私人clock／任意JSON、無英雄／道具名稱推測。取消回空狀態；legacy無total仍維持未知。

Final RustV2 all8 maintained scopes passed1844/0failed/9existingignored:ABI15/content44/templates116/codegen70/core979/base326/bridge137/client157. Backend and UE5.8 buildV3 passed; staged DLL/import SHA matched. Native WorldV2 executed75 with0failed/0notrun, including actual Rust-filtered/ABI-export-to-native progress integration. Source guard verified3440 inputs:3434 unchanged prior inputs +5 Rust changes +1 native test addition. 497 evidence files and37 PNGs independently reverified; health12 samples/max171 seconds. All6 final workers and6 historical workers (4 failed/2 successful) have creation-token-bound retired receipts; launchers and the single170-second health monitor were reaped and CIM showed no remaining owned work.

原始RustV1夾具失敗、6份修正前來源及test-only修正證明保留。詳見2026-10-23-public-channel-final-acceptance.md及E1526。此批關閉目前維護版filtered總時長缺口，不宣稱舊二進位理解BVS2；完整combat replica、channel動畫／VFX／更多效果、窄施法提示換行、既有worldless裁切／map標籤重疊與11外部/MVP33/44仍pending。

## 2026-10-23 技能施法提示單行呈現

共用技能格顯示精簡施法秒數／取消／完成，保留完整中文狀態於tooltip；依使用者字體大小量測卡片並換列，避免時間逐字拆行或縮小文字。真實GameViewport30組配置涵蓋2解析度／3文字縮放／5狀態，檢查單行字形高度、配置寬度及生命周期。

UE5.8 build-v1/staged DLL/import SHA passed. WorldV2 executed75:74success+1existingwarningcase,0failed/0notrun. Source guard verified3440 inputs; accepted Rust1844passed/0failed/9existingignored and backend results inherited after exact SHA proof of3437 unchanged prior inputs plus3 native deltas; no new full Rust suite is claimed. Archive contains360 independently reverified evidence files and37 PNGs. Health5 samples/max171 seconds. Four successful final workers and two failed historical workers have creation-token-bound retired receipts; all launchers, inheritance reader and the single170-second health monitor were reaped. CIM found no owned worker remaining.

6 changed/new images actually viewed; 31 SHA-identical accepted images inherited. Actual cast crop is40x24, previously38x65; <0.1s remains one line. Actual full1280x720 viewport shows cooldown20, cast progress and hotkey intact. No font shrinking or local cast countdown.

詳見2026-10-23-compact-cast-final-acceptance.md與E1527。原始不完整runner cohort保存，正式來源未因runner修復變動。窄施法提示換行缺口關閉；channel動畫／VFX、完整combat replica、11外部/MVP33/44、worldless裁切／mapfocus標籤重疊仍pending。

## 2026-10-23 共用世界引導特效

公開canonical引導總時長／剩餘時間接原生角色frame，沿用epoch／tick／sequence隔離與既有有界特效池。每角色一個青綠圓環依權威進度改變大小；本機時間只脈動亮度，不預測完成。支援取消／隱藏／死亡／換代／scope／退休／回城優先與滿載後新快照恢復，避免重複generic Buff圓環。沒有引導時不再掃描全部實體。

UE5.8 buildV3/staged DLL/import SHA passed. WorldV2 executed76:75success+1existingwarningcase,0failed/0notrun. Source guard verified3441 inputs; full accepted Rust1844passed/0failed/9existingignored and backend result inherited after SHA proof3431 unchanged prior inputs+9 native changes+1 addition; no full Rust rerun is claimed. Archive contains437 independently reverified evidence files; 41 final accepted PNGs, with historical captures also retained. Health8 samples/max171 seconds. Four final and six historical successful workers have creation-token-bound retired receipts; all launchers, three inheritance readers and the single170-second health monitor were reaped. CIM found no owned build/test/source worker remaining.

11 new/changed images actually viewed; 30 accepted prior SHA-identical images inherited. Four new512x512 images show the actual idle native hero with a turquoise channel ring, isolated midway and larger late ring, and a completely cleared shader image. No channel bone animation is claimed; existing idle pose is used for the character capture.

All initial cohorts are preserved. BuildV1 passed; empty-state review then added a projector fast return and empty snapshot assertion (exact2 deltas,8 unchanged between ten-source inputs-v1/v2). WorldV1 passed76 native tests but actual character+ring PNG was rejected for T-pose: the fixture omitted native idle state after an epoch reset. A fixture-only follow-up sends current idle, asserts the actual animation slot and lights the camera-facing side; all9 production sources unchanged from inputs-v2 to final inputs-v3. Full freshWorldV2 and finalBuildV3 accepted. Original rawWorldV1/report/available images, every source cohort and all6 historical worker receipts remain archived. No failed native result is invented and the visually rejected run is not approved. E1528 records root causes and prevention.

詳見2026-10-23-channel-vfx-final-acceptance.md與E1528。Channel bone animation, further gameplay effects, full combat replica,11 external tasks/MVP33/44 and known worldless first-kill crop/map focus-label overlaps remain pending. This closes common world channel VFX for maintained canonical metadata. No new real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.

## 2026-10-23 共用骨架引導呈現

以共用channel語意映射／專用slot優先，依設定fallback重用現有recall／cast片段。由權威進度取樣骨架姿勢並保持，網路快照延遲不由本機動畫推測完成。取消恢復最近移動，死亡／重生／回城／準備優先，隱藏／換代／重置清除。兩種真骨架與實際姿勢圖接受，未新增專用美術資產。

UE5.8 buildV2/staged DLL/import SHA passed. WorldV2 executed77:76success+1existingwarningcase,0failed/0notrun. Source guard verified3442 inputs; full accepted Rust1844passed/0failed/9existingignored and backend result inherited after SHA proof3438 unchanged prior inputs+3 native changes+1 addition; no full Rust rerun claimed. Archive contains397 independently reverified files; 44 final accepted PNGs, with failed historical captures also retained. Health5 samples/max171 seconds. Four final workers and four historical workers (2successful/2failed) have creation-token-bound retired receipts. All launchers, two inheritance readers and the single170-second health monitor were reaped; CIM found no owned build/test/source worker remaining.

11 new/changed images actually viewed; 33 accepted prior SHA-identical images inherited. Three actual512x512 early/mid/late pose images and the combined turquoise-ring/character image show sampled channel posture. Existing sustained asset movement is subtle; exact normalized cursors are asserted on two real skeletons. No bespoke animation assets or new channel gameplay are claimed.

NativeWorldV1 correctly failed the second-skeleton first-state assertion because the fixture reused epoch101/tick1 after the first skeleton advanced that shared world epoch to tick4. Original raw77-test run/report/all available images, false World/source receipts and4 exact inputs-v1 remain archived. Fixture-only repair reserves100+H*10 epoch ranges and logs initial scope; all3 other native sources unchanged. Final buildV2/fresh fullWorldV2 use inputs-v2. Both real skeletons completed with initial epochs100 and110. E1529 records root cause and prevention; production admission was not weakened.

詳見2026-10-23-channel-animation-final-acceptance.md及E1529。Common canonical channel strip/public duration/world ring/native pose gaps are now closed within the maintained shared mechanism. More authored channel assets/content, further gameplay effects, full combat replica,11 external tasks/MVP33/44 and known worldless first-kill crop/map focus-label overlaps remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 指定敵方控制道具

共用封閉控制種類／固定精度持續時間從Lua一路接到權威結算與原生說明，新增縛行符：30傷害、定身1.5秒、射程450、冷卻18秒、650金。建築在所有寫入前拒絕，存活目標沿用canonical控制；較長既有控制不縮短，死亡不新增控制。介面顯示中文限制／時間，異常資料停用。

Full8-scope Rust1849passed/0failed/9existingignored (ABI15/content45/templates117/codegen71/core979/base328/bridge137/client157). Formal backend and UE5.8 buildV1 passed; staged DLL and bridge import SHA matched. NativeWorldV1 executed78:77success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3443 inputs;3429 unchanged prior inputs+10 exact formal inputs+4 generated deltas. Archive contains157 independently reverified evidence files and45 accepted PNGs:9 actual new/changed images viewed,36 prior SHA-identical images inherited. Health8 samples/max171 seconds. Six final creation-token-bound owned workers retired; all launchers/source readers and the single170-second health monitor reaped; final CIM found no owned build/test/source process remaining.

詳見2026-10-23-targeted-control-final-acceptance.md與E1530。

E1530 records read-only guessed-path errors and prevention. No build/test failure occurred in this batch; existing warnings/ignored cases are retained. Source freeze released only after every reader retired. Further gameplay/content polish, full combat replica,11 external tasks/MVP33/44 and known worldless first-kill crop/map focus-label overlaps remain pending. This closes the shared targeted-control item mechanism, not every LoL feature. No new real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 控制道具建築指向提示

將typed非建築限制接至共用施放預覽；只對當前公開、其餘條件合法的建築顯示拒絕，不洩漏隱藏／過期目標種類。原無控制道具保留建築施放提示，替換／退休清除政策。

UE5.8 BuildV2 and staged DLL/import SHA passed. WorldV1 executed79:78success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3444 inputs; prior full8-scope Rust1849passed/0failed/9existingignored and backend accepted by exact SHA proof3436 unchanged inputs+7 native deltas+1 native addition; no new full Rust suite claimed. Archive contains131 independently reverified evidence files and46 final PNGs. Health6 samples/max171seconds. Four final and one failed historical worker have creation-token-bound retired receipts. All launchers, two inheritance readers and the single170-second health monitor were reaped; final CIM found no owned build/test/source process remaining.

BuildV1 failed C2248 because the new fixture lacked an existing-style WITH_DEV_AUTOMATION_TESTS friend declaration. Original failed build/log/false retired receipt,8inputs-v1 and initial inheritance proof retained. After all readers retired, only one test-guard friend line was added; other7 inputs unchanged. Inputs-v2 and fresh BuildV2 accepted. Production members remain private and admission was not weakened. E1531 records root cause, repeated read-only path mistakes and prevention.

Evidence:docs/acceptance/2026-10-23-control-item-feedback/index.json; final/errors:docs/plans/2026-10-23-control-item-feedback-{final-acceptance,errors}.md. All readers retired; source freeze released. Further gameplay/content polish, full combat replica,11 external tasks/MVP33/44 and known worldless first-kill crop/map focus-label overlaps remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 獨立HUD幾何排版

共用Slate幾何弱回呼沿用既有DPI／尺寸快取與測量重排，沒有遊戲世界Tick也能正確排版。三種真獨立視窗尺寸接受；原首殺HUD右下裁切以實際前後圖關閉。

UE5.8 BuildV2/staged DLL/import SHA passed. FullWorldV2 executed80:79success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3445 inputs; prior full8-scope Rust1849passed/0failed/9existingignored and backend inherited after exact SHA proof3439 unchanged inputs+5 native deltas+1 native addition; no new full Rust rerun claimed. Archive contains419 independently reverified evidence files and49 final PNGs; 12 actual new/changed images viewed, 37 accepted prior SHA-identical images inherited. Health7 samples/max171seconds. Four final and four historical workers (2successful/2failed) have creation-token-bound retired receipts. All launchers, both inheritance readers and the single170-second health monitor reaped; final CIM found no owned build/test/source worker remaining.

BuildV1 passed; rawWorldV1 executed80 but one ObjectiveAnnouncementBanner fixture failed its unchanged36px font assertion. It used a private temporary font override rather than persistent interface preferences. After all readers retired, retained that assertion and changed only the fixture to SetInterfacePreviewForAutomation(TextScale). All5 initial inputs stayed byte-identical; preserved the original sixth fixture bound to the failed3445source baseline. Final6inputs-v2/fullBuildV2/fullWorldV2 accepted. Original failed report/all49images, false World/source receipts and successful historical build/source-capture evidence remain archived. E1532 records cause/prevention and pre-reader edit/pre-execution sealer version mistakes; no production layout check or font threshold was weakened.

Evidence:docs/acceptance/2026-10-23-detached-hud/index.json; final/errors:docs/plans/2026-10-23-detached-hud-{final-acceptance,errors}.md. All readers retired; source freeze released. Known standalone first-kill HUD crop closed. Map focus-label overlaps, further gameplay/content polish, full combat replica and11 external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 自用淨化道具

共用封閉self_control_cleanse效果重用既有淨化機制；新增淨化護符1400金／冷卻45秒，可在受控時解除自身三種支援控制，保留其他效果與既有命令，不提供免疫。正式60Hz、原生自用輸入／中文說明／公開狀態接受。

Full8-scope RustV2 1856passed/0failed/9existingignored (ABI15/content46/templates118/codegen72/core980/base331/bridge137/client157). Formal backend and UE5.8 BuildV2 passed; staged DLL/import SHA matched. FullNativeWorldV2 executed81:80success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3446 inputs;3432 prior inputs unchanged+10 exact formal inputs-v3+4 generated deltas. Archive contains521 independently reverified evidence files and50 accepted PNGs; 10 actual changed/new images viewed,40 accepted prior SHA-identical images inherited. Health15 samples/max171seconds. Six final and five historical creation-token-bound workers retired; all launchers/source readers and the single170-second monitor reaped; final CIM found no owned build/test/source process remaining.

NativeWorldV1 also rejected the new guidance assertion: the legacy unconditional positive-amount tooltip guard hid the parameterless effect. Original81 report/screenshots and false receipts are preserved. After all readers retired, only the shared native tooltip guard and its fixture changed; five malformed effect-metadata cases were added and original guidance assertion retained. Inputs-v3 freezes the native repair with other8 byte-identical inputs. RustV2 remains bound to its exact unchanged Rust/Lua inputs; UE BuildV2/fullWorldV2/sourceV2 reran. Successful historical build/source capture and both failed native/source runs remain archived; five historical workers comprise2successful/3failed including RustV1.

RustV1 exposed an actual legacy self-item admission bypass: Warmup use consumed an item. Preserved its failing assertion, raw six-scope logs/false retired receipt and10 exact inputs-v1. After every reader retired, repaired the common transaction with the existing active-hero context helper, preserving no-match/Stop semantics; both legacy shield and new cleanse now reject pause/non-playing/mismatched roster atomically. Only game_processor and formal test sources changed; other8 exact inputs remained unchanged. Fresh full8-scope RustV2 accepted; no failed run accepted. E1533 records this cause/fix/prevention and pre-reader workflow/path corrections. Evidence:docs/acceptance/2026-10-23-self-cleanse-item/index.json; final/errors:docs/plans/2026-10-23-self-cleanse-item-{final-acceptance,errors}.md. All readers retired/source freeze released. Map focus-label overlaps, further gameplay/content, full combat replica and11 external tasks/MVP33/44 remain pending. Standalone first-kill HUD crop remains closed. No real network matches/quota reset/commit/push/omfx maintenance. Original deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 焦點名牌避免遮住英雄

共用名牌admission移除焦點穿越英雄保留區的例外；選取／指向保留穩定優先順序及既有資訊卡，所有非英雄名牌一致避開英雄血條。實際量測／四種縮放／新畫面恢復／焦點截圖接受。英雄彼此重疊及64英雄保留上限仍待辦，不宣稱所有地圖名牌問題關閉。

UE5.8 BuildV1/staged DLL/import SHA passed. FullWorldV1 executed81:80success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3446 inputs;3443 prior inputs unchanged+3 exact native deltas prove inheritance of prior full8-scope Rust1856passed/0failed/9ignored and formal backend; no new full Rust run claimed. Archive contains105 independently reverified evidence files and50 accepted PNGs;12 actual changed images viewed,38 prior SHA-identical images inherited. Health4 samples/max171seconds. Four owned workers retired; all launchers, exact inheritance reader and single170second health monitor reaped; final CIM found no owned build/test/source worker remaining.

No build/test failure occurred. Errors/prevention include the pre-reader Saved helper substring assumption; completed its remaining visual comparator directly without rerunning partial mutations. Evidence:docs/acceptance/2026-10-23-focus-nameplate/index.json; final/errors:docs/plans/2026-10-23-focus-nameplate-{final-acceptance,errors}.md. Source freeze released after every reader retired. More gameplay/content polish, hero-to-hero overlap, full combat replica and11 external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 自用疾行召喚師

共享封閉self_sprint操作與有界限時移速聚合；第五個召喚師疾行80移速／6秒／180冷卻。與道具疾行只取最高值、各自倒數；保留命令／控制／基礎移速。正式60Hz及原生D/F／中文提示接受。現有五個召喚師是訓練內容，不宣稱完整競技選項或精確LOL平衡。

Full8-scope RustV4 1863passed/0failed/9existingignored (ABI15/content47/templates118/codegen73/core984/base332/bridge137/client157). Formal backend/UE5.8 BuildV2/staged DLL/import SHA passed. FullWorldV2 executed82:81success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3447 inputs;3433 prior inputs unchanged+11 exact formal inputs-v4+3 generated deltas. Archive contains599 independently reverified evidence files and51 accepted PNGs;11 actual changed/new images viewed,40 prior SHA-identical images inherited. Health17 samples/max171seconds. Six final and eight historical owned workers (6successful/2failed) retired; all launchers/readers and single170second health monitor reaped; final CIM found no owned build/test/source worker remaining.

RustV1 failed a legacy fixed4summoner count in the trinket loader fixture; retained original fixture SHA bound to prior accepted baseline and replaced with5 plus actual sprint identity/data assertions. RustV2 passed first5scopes then formal compilation rejected a guessed HeroCommand::Hold test assertion; actual helper uses HoldPosition. Every reader retired/reaped before each repair. Original10inputs-v1/extra fixture,11inputs-v2, both raw failed roots and false retired receipts preserved; final11inputs-v4 accepted after full8scope rerun. Runtime enum addition sits at its physical end to preserve existing serde order. Successful fullRustV3/backendV1/UEBuildV1/fullWorldV1/sourceV1 cohort is also preserved. Final audit found an operation-wide reserved sprint source aliases different authored IDs. After all readers retired, derive the source from validated current loadout identity and add real100/2second+80/6second regression (100 then80 then0). Only native summoner_world and formal pause fixture changed from11inputsv3;9others unchanged. Fresh fullRustV4/core984/backendV2/UEV2/fullWorldV2/sourceV2 accepted. Final8historical workers include6successful/2failed. No successful earlier run substitutes for the repaired final source. E1535 errors/prevention:docs/plans/2026-10-23-summoner-sprint-errors.md. Evidence:docs/acceptance/2026-10-23-summoner-sprint/index.json. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap/64hero reservation limit, full combat replica and11 external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 D/F冷卻可讀性

一般卡片／邏輯文字／tooltip的正值尾端顯示<0.1秒，明確剩餘／總冷卻；非法數值不冒充有效倒數。距離格式與既有compact倒數／readiness保持。

UE5.8 BuildV2/staged DLL/import SHA passed. FullWorldV2 executed82:81success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3447 inputs;3445 prior inputs unchanged+2 exact native inputs-v2 prove inheritance of prior full8scope Rust1863passed/0failed/9ignored and backend; no new full Rust run claimed. Final inheritance is bound to fresh full-source verification, not a second redundant whole-tree hash. Archive contains401 independently reverified files and51 accepted PNGs;15 actual changed images viewed,36 prior SHA-identical images inherited. Health7 samples/max171seconds. Four final and four historical owned workers (2success/2failed) retired; all launchers/initial inheritance reader/source readers and single170second health monitor reaped; final CIM found no owned build/test/source worker remaining.

WorldV1 ran82 but ResponsiveHudLayout failed its retained integer range assertion because SanitizeFloat changed850cm to850.0cm. Original2inputs-v1, full report/all51screenshots, falseWorld/source receipts and successful build/sourcecapture/initialinheritance retained. Every reader retired/reaped before restoring separate distance scalar formatting and adding signedzero normalization/coverage. Fresh2inputs-v2/fullUEV2/fullWorldV2/sourceV2 accepted; no failed run accepted. Preparation class-string/missing-helper failure occurred before readers, was corrected and recorded. Errors:docs/plans/2026-10-23-summoner-time-errors.md; evidence:docs/acceptance/2026-10-23-summoner-time/index.json. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap/64hero reservation cap, full combat replica and11 external tasks/MVP33/44 remain pending. No network match/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 六道具槽冷卻提示

All six item slots now reuse the D/F cooldown scalar in logical pairs, expanded countdown and concise labels. Positive tails below0.1seconds display<0.1 rather than0.0; fractional values retain one decimal and trailing.0 is omitted. Exact signedzero hides the card countdown. Negative/nonfinite/inverted pairs disclose unavailable timing; passive/empty/cleared slots do not retain active countdown. Existing compact ceiling, authority/readiness/input policy remain unchanged; no local timers or identity branches.

Existing ItemCastFeedback fixture now covers all six slots, tiny/fractional positives, signedzero readiness, four malformed pairs, passive suppression and economy scope clearing. Its actual Slate capture includes the live logical positive-tail label alongside existing target/range/visibility guidance. Retained D/F, responsive HUD and action retirement tests remain in the full native suite.

UE5.8 BuildV2/staged DLL/import SHA passed. FullWorldV2 executed82:81success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3447 inputs;3444 prior inputs unchanged+3 exact native inputs-v2 prove inheritance of prior full8scope Rust1863passed/0failed/9ignored and backend; no new fullRust run claimed. Inheritance bound to fresh source verification. Archive contains409 independently reverified files and51 accepted PNGs;12 changed images actually viewed,39 prior SHA-identical images inherited. Health6 samples/max172seconds. Four final and four historical workers (2success/2failed) retired; all launchers/source readers and single170second health monitor reaped; final CIM found no owned build/test/source worker remaining.

Preparation path guesses and atomic patch mismatch were corrected before build/readers and recorded in docs/plans/2026-10-23-item-time-errors.md. WorldV1 failed exact old 2.0-second presentation expectations after intentional removal of trailing.0. Original fixture SHA bound to failed source baseline, original2inputs-v1 and extra fixture, failed full report/all51images and all4retired receipts preserved. Repaired only two expected text literals to2 and2/5; retained all semantic/status/tooltip checks. Fresh3inputs-v2/fullUEV2/fullWorldV2/sourceV2 accepted. No failed runtime accepted. Evidence:docs/acceptance/2026-10-23-item-time/index.json. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap/64hero reservation cap, full combat replica and11 external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 英雄保留區移除64位截斷

Every current projected visible hero with matching actor entity/generation now reserves its exact measured screen footprint in the existing public-target pass. Removed the top64 hero heap and reservation admission cap; inline64 storage can grow. Nonhero shortlist, priority, focus semantics and output remain bounded64. Reservation union is independent of hero input order, with exact per-rectangle overlap rather than a conservative enclosing rectangle. Fresh view state preserves hidden/retired/new-generation/moved behavior; hero rendering and world anchors remain unchanged. Hero-to-hero overlap remains pending.

Existing HeroNameplateReservation fixture verifies a distinct65th hero footprint, reversed input order, available exact gap,10000 valid reservations, retained64 nonhero output, malformed geometry and four UI scales. Actual current hero widget becomes65th reservation for concealment/retirement/reveal/new-generation and two actual ordinary/focused captures. Existing movement recovery retained. The10000 case exercises reservation storage/admission, not a claim of10000 live rendered hero widgets or a full stress-world benchmark. Storage O(H), bounded nonhero overlap work O(H*64); each actual projected visible hero has one measured layout in this pass.

UE5.8 BuildV1/staged DLL/import SHA passed. FullWorldV1 executed82:81success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3447 inputs;3444 prior inputs unchanged+3 exact native inputs-v1 prove inheritance of prior full8scope Rust1863passed/0failed/9ignored and backend; no new fullRust run claimed. Inheritance bound to fresh source verification. Archive contains110 independently reverified files and51 accepted PNGs;13 changed images actually viewed,38 prior SHA-identical images inherited. Health4 samples/max172seconds. Four owned workers retired; all launchers/source readers and single170second health monitor reaped; final CIM found no owned build/test/source worker remaining.

Pre-reader geometry review corrected one gap fixture width to fit both padded neighbors; call-site review preserved actor identity/generation validation when moving work between phases. Errors/prevention:docs/plans/2026-10-23-all-hero-errors.md. No failed runtime accepted. Evidence:docs/acceptance/2026-10-23-all-hero/index.json. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap, full combat replica and11 external tasks/MVP33/44 remain pending. The former64hero reservation truncation is closed. No real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 可點擊回城入口

Added a discoverable native header recall action alongside the existing read-only progress. RequestRecallCard delegates to the existing cursor-free recall reflection/owner/modal/life/admission path, without coordinate or queue state. Current HUD/bridge binding and retirement guards match other cards. Shared pointer shell consumes disabled press/release and handles press-time keyboard blocks; the button does not take focus. No new backend operation, local countdown or inferred completion/ACK.

The action reads the ordered authoritative recall snapshot. Ready requires current valid alive/playing owner/entity, legal authored total and exactzero remaining; active channel/shop/invalid/dead/finished/retired cannot present ready. Unknown protocol/scope/HUD chrome collapses it. Label reads actual primary binding, tooltip lists actual primary/secondary, authored total and interruption guidance. Existing progress predicate shares equivalent snapshot validation; new disclosure/clear transitions notify header layout. Authority retains all final command gates, including conditions not disclosed by this HUD payload.

Existing RecallChannelProgress fixture tests the actual same live SButton during channel/zero/shop, default binding, no-owner safe activation, malformed/dead/ended states, retired visibility, actual disabled shell pointer handling, existing cursor-free missing-surface rejection and no-target/no-queue semantics, owner retirement. Existing actual capture contains ready action plus three current progress rows. Existing responsive and action-owner regressions passed. This is single-machine native UI/input-path verification, not a real authority ACK or match.

UE5.8 BuildV3/staged DLL/import SHA passed. FullWorldV3 executed82:81success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3447 inputs;3442 prior inputs unchanged+5 exact native inputs-v3 prove inheritance of prior full8scope Rust1863passed/0failed/9ignored and backend; no new fullRust run claimed. Inheritance bound to fresh source verification. Archive contains447 independently reverified files and51 accepted PNGs;8 changed images actually viewed,43 prior SHA-identical images inherited. Health8 samples/max171seconds. Four final and five historical workers (3success/2failed) retired; all launchers/source readers and single170second health monitor reaped; final CIM found no owned build/test/source worker remaining.

Errors/prevention:docs/plans/2026-10-23-recall-action-errors.md. Read-only inventory path misses and pre-reader pointer/layout review preserved. Successful BuildV1/stage and original5inputs-v1 preserved; before native/source readers, mode audit found a retained MOBA snapshot could expose the action after TD/waiting switch. After build reader retired/reaped, added explicit presentation guard and actual same-button TD/waiting/recovery assertions. WorldV2 ran82 and failed unmounted Slate cached-attribute assertions in RecallChannelProgress; production ready/disabled capture already rendered correctly. All readers retired before changing only fixture to mount an actual window and pump Slate frames. Hidden children may retain cached enabled state, so hidden-state cases require collapsed root and model admission false; visible states still require actual button disabled. Original5inputs-v2/all failed51images/report and4retired receipts retained. Fresh5inputs-v3/fullUEV3/fullWorldV3/sourceV3 accepted; no failed runtime accepted. Evidence:docs/acceptance/2026-10-23-recall-action/index.json. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap, full combat replica and11 external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 加速與減傷公開生命期

Shared typed sprint/damage-reduction families now disclose canonical movement_boost / damage_reduction IDs with authored Chinese names 移速提升 / 傷害減免. Validation requires the closed family, exact integer stat payload, valid magnitude and positive<=60-second lifetime. Forged reserved names, malformed/extra/float fields and invalid lifetimes do not become public status. Existing BVS1 codec/catalog/owner consumers are reused. No private source key, spell/item identity or magnitude is published; source names are not parsed. Gameplay strongest-family aggregation remains unchanged.

The public timer is the maximum valid source lifetime: at least one effect of that family remains, without claiming that the strongest magnitude lasts that long. Actual current stats continue using existing strongest-source rules. Core verifies stronger-shorter/weaker-longer, independent expiration, refresh/removal, foreign entity and malformed reserved data; existing real summoner ECS test verifies actual cast plus item source status and expiry. Actual Rust store->canonical codec->filtered replica->ABI export supplies new native HUD identity/countdown/foreign owner/old generation/clear/retirement and full unsigned64 key-preservation assertions. Existing shared localized hero-details renderer displays both statuses; no dedicated new icon/VFX or magnitude timer is claimed.

Fresh full8scope RustV1 1867passed/0failed/9existingignored (ABI15/content47/templates118/codegen73/core987/base332/bridge138/client157). Formal backend/UE5.8 BuildV3/staged DLL/import SHA passed. FullWorldV3 executed83:82success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3448 inputs;3433 prior inputs unchanged+7 exact formal inputs-v4+8 explicit generated deltas. Archive contains529 independently reverified files and52 accepted PNGs;8 changed/new images actually viewed,44 prior SHA-identical images inherited. Health13 samples/max171seconds. Seven final and five historical workers (3successful/2failed) retired; all launchers/readers and single170second health monitor reaped; final CIM found no owned build/test/source worker remaining.

Before native builds, audit identified actual movement visual key0x9ca5c927d356ae1a exceeds int64. FullRust and concurrent fullsource snapshot readers retired/reaped before correcting only native fixture to uint64 parsing plus bitwise preservation assertions. Original6inputs-v1 and fullRust source snapshot retained;5Rust/Lua inputs unchanged. Final source comparison proves all later differences from that fullRust snapshot limited to the two native C++ files and explicit generated outputs; no Rust rerun after C++-only change claimed or needed. Post-UEBuildV1 audit found late valid Buff snapshots could repopulate retired UI cache. Successful original build/stage/6inputs-v2 plus original widget SHA bound to fullRust snapshot retained. After reader retirement sanitized the shared retired setter to empty, added late active publication regression, froze7inputs-v3 and rebuilt UEV2/fullWorldV2/sourceV2. FullWorldV2 failed only the new fixture because a C ABI POD row from AddDefaulted was not value-initialized; all original7inputs-v3/report/52PNG and failed dependency receipts preserved. All readers retired before fixing only fixture initialization, freezing7inputs-v4 and rebuilding/fullWorldV3/sourceV3. No failed runtime accepted. Read-only path/count diagnostic errors and native field audit recorded with prevention in docs/plans/2026-10-23-modifier-status-errors.md. Evidence:docs/acceptance/2026-10-23-modifier-status/index.json. Final baseline for future batches is omfue/Saved/BuildTemp/modifier-status-source-before-v3.json, final cohort inputs-v4; rust-source-before-v1 is the earlier Rust proof only. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap, full combat replica and11 external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 自身狀態摘要列

Added a visible read-only owned status summary to the existing header flow. Reuses the exact shared buff validation/localized naming/stack counts/permanent and sub-tenth positive lifetime formatter. At most two24-character sanitized effect names plus explicit remaining-count line; the shared hover tooltip retains the full<=64 accepted rows. Existing diagnostic getters retain compatibility. Cached event-driven formatting only, no added per-entity/frame loop, new wire field, authority command or local timer. Existing header width budget/max360, text scaling and ellipsis bound the row.

The summary and tooltip require current alive/playing MOBA hero context, matching entity and generation, visible HUD and active scope. Empty/expired/malformed/foreign lists clear it; dead/end/TD/waiting/new-generation/retirement hide it and late publication cannot revive retirement. Existing actual Rust filtered ABI fixture confirms real movement_boost/damage_reduction rows reach the visible summary. New mounted Slate fixture checks pre-arriving/no-context state, two-row+overflow, full tooltip/stack/permanence/sub-tenth, private-key exclusion, generation/dead/end/mode/expiry/invalid/long sanitized name and retirement, and captures actual summary plus full tooltip. No real network match or authority ACK claimed.

UE5.8 BuildV1/staged DLL/import SHA passed. FullWorldV1 executed84:83success+1existingwarningcase,0failed/0notrun,warnings4. Full source guard3449 inputs,3445unchanged+4exactnative inputs-v1; prior fresh8scope Rust1867passed/0failed/9ignored and backend inherited by source-bound proof, no freshRust rerun claimed. Archive119 independently verified files and53accepted PNGs;12changed/new actually viewed,41priorSHAidentical inherited. Health5samples/max170seconds. Four owned workers retired; all launchers/source readers and single170second health monitor reaped, final CIM found no owned worker remaining.

Evidence:docs/acceptance/2026-10-23-owned-status/index.json. Errors/prevention:docs/plans/2026-10-23-owned-status-errors.md. Final source baseline omfue/Saved/BuildTemp/owned-status-source-before-v1.json, frozen4inputs-v1. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap, full combat replica and11external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 自用減傷召喚師堅守

Added shared closed SelfDamageReduction author/runtime operation and append-only catalog6 堅守 (25%/3seconds/CD180). Existing loadouts are retained until explicitly equipped. Strict positive quantized reduction<=1, lifetime<=60, no-target shape and existing alive/context/owner/pause/cooldown plan/commit rules apply. Unsupported adapter or absent ECS BuffStore rejects without cooldown. Source-keyed grant uses the existing item_damage_reduce family, strongest value and independent source expiry. No hero/spell-name parsing or native special rules. No healing/shield/cleanse/immunity, original movement/control state retained.

Existing canonical damage_reduction status/details/visible summary receive the effect and represent any valid remaining source lifetime. Author/model/codegen/catalog, actual ECS control/command preservation/public time and atomic rejection tests plus formal60Hz real packet damage160->120->140->160 with weaker longer item, pause and owner-team cooldown disclosure pass. New native actual generated catalog/description/readyD/F/self-shape/cooldown/retirement fixture captures shared guidance. No true match or authority ACK claim.

Fresh full8scope RustV2 1873passed/0failed/9existingignored. Formal backend and UE5.8 BuildV2/staged DLL/import SHA passed. FullWorldV1 executed85:84success+1existingwarningcase,0failed/0notrun,warnings4. Source guard3450inputs;3437unchanged+10exactformal inputs-v3+3explicitgenerated deltas. Archive218independently verified files and54accepted PNGs;12changed/new actually viewed,42priorSHAidentical inherited. Health10samples/max171seconds. Six final workers and two historical failed workers (RustV1/buildV1) retired; every launcher/source reader and single170second monitor reaped, final CIM no owned worker remaining.

RustV1 passed4scopes then failed core compile E0425 in a new test because BuffStore use was scoped to a sibling fixture. All readers retired/reaped before adding only the missing test-local import. Original9inputs-v1/rawfailed logs/retired receipt preserved; fresh9inputs-v2 has8unchanged+1test-only change and full8RustV2 accepted. UEBuildV1 failed new native fixture private tooltip access because its WITH_DEV_AUTOMATION_TESTS friend was missing. After all readers retired added only that friend;10inputs-v3 retains all9RustV2 sources byte-identical, original parent header and failedbuild retained. FreshUEV2/native/source required and passed before final acceptance. Pre-reader field/match/float and saved helper errors with prevention retained in docs/plans/2026-10-23-summoner-guard-errors.md. Evidence:docs/acceptance/2026-10-23-summoner-guard/index.json. Final source baseline omfue/Saved/BuildTemp/summoner-guard-source-before-v1.json and10inputs-v3. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap, full combat replica and11external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 英雄名牌有限螢幕避讓

Current validated projected visible heroes now use stable self/selected/hover/other, center-distance and identity priority for nearby vertical nameplate placement. A fresh conservative occupancy bit grid attempts at most37positions within144logical pixels. Grid memory<=1048576bits and cell visits<=2000000 per view; no64hero cutoff and no quadratic hero rectangle scan. All heroes remain visible; invalid/crowded/exhausted placement retains original anchor. SortingO(HlogH), existing nonhero overlap workO(H*64), plus fixed grid work; this is not a10000live-rendered performance benchmark.

Actor placement uses current measured logical layout and component pivot, matching UE5.8 screen-layer SConstraintCanvas alignment. World actor/mesh/nameplate anchors and target input remain unchanged. Actor validates offset and size; call-site reserves only actual applied footprints for existing64nonhero admission. Concealment/render-collapse/source revocation/retirement/new-life/unprojected state resets pivot. Existing public-frame cadence retained, with no extra entity-per-frame pass or local time.

New native test verifies padded separation, exact moved nonhero footprint, edge/down placement, four geometry scales, stable ordering/recovery, invalid input,10000dense helper work budget and fixed grid storage. Two actual actor/widget/component pivots paint in a real Slate window; measured desktop bounds do not overlap, world anchor is unchanged, and separated actors return to original alignment. Hide/revoke/retire/lateplacement/newgeneration assertions pass. Two actual captures establish placement/recovery. The overall compiled map image still contains dense label overlaps; this batch does not establish full-map or true-network separation. Dense fallback and broader projection/disclosure/integration scenarios remain work to investigate, without weakening existing visibility gates.

UE5.8 BuildV2/staged DLL/import SHA passed. FullWorldV1 executed86:85success+1existingwarningcase,0failed/0notrun,warnings4. Full source guard3452 inputs;3446unchanged+6exactnative inputs-v2 prove inheritance of priorfresh8scope Rust1873passed/0failed/9ignored and backend. No freshRust/backend rerun claimed. Archive140independently verified files and56accepted PNGs;13changed/new actually viewed,43priorSHAidentical inherited. Health5samples/max171seconds. Four final workers and one historical successful build retired, all launchers/source readers and single170second monitor reaped; final CIM no owned worker remaining.

After successful UEBuildV1/stage, pre-native engine review caught default centered Canvas capture heading. All readers retired before correcting only fixture heading alignment; original6inputs-v1/successfulbuild/stage preserved,6inputs-v2 retains5production sources unchanged. No failed runtime accepted. Errors/prevention:docs/plans/2026-10-23-hero-placement-errors.md. Evidence:docs/acceptance/2026-10-23-hero-placement/index.json. Final baseline omfue/Saved/BuildTemp/hero-placement-source-before-v1.json,6inputs-v2. All readers retired/source freeze released.

Further gameplay/content polish, dense hero-label/fallback and whole-map integration, full combat replica and11external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 地圖實際投影名牌整合

OpenSpec complete-projected-map-nameplate-integration4/4.

The compiled-map fixture previously published its initial public frames before the actual local viewport/HUD and final overview camera existed; later window paints alone did not refresh frame-driven screen admission. It now publishes a next formal synthetic public frame after final overview and combat camera fitting, consistently advancing animation/frame sequences before any transient combat events. No per-paint event replay, production disclosure override or assumed player identity was added.

The overview now checks actual cached desktop bounds for every admitted nonhero against all other admitted labels, preserves all six visible hero labels and requires nonempty <=64 nonhero admission. Hero-to-hero finite-space fallback remains permitted by the shared production design. This is integrated synthetic public rendering, not a real match or authority ACK.

UE5.8 BuildV3/staged DLL and import SHA passed. Full native86:85success+1existingwarningcase,0failed/0notrun,warnings4. Fullsource3452 verifies3451 unchanged+1native fixture input; prior fresh8scope Rust1873/0failed/9ignored and backend inherited by source-bound proof. No freshfullRust/backend rerun claimed. Archive360 independently verified files/57PNG, 12changed/additional actually viewed and45SHAidentical inherited. Health7samples/max171seconds. Four final and four historical workers (two successful, one failed native suite, one source pipeline with failed inheritance) retired; all launchers/readers and health monitor reaped, final CIM no owned worker remaining.

Evidence:docs/acceptance/2026-10-23-projected-map/index.json. Errors/prevention:docs/plans/2026-10-23-projected-map-errors.md. Final baseline omfue/Saved/BuildTemp/projected-map-source-before-v2.json, one frozen input inputs-v2. All readers retired/source freeze released. Further concrete gameplay polish, dense hero fallback, full combat replica and11 external tasks/MVP33/44 remain pending. No true network match, quota reset, commit/push or omfx maintenance. Original deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.


## 2026-10-23 驗收程序真實退出結果

OpenSpec fix-owned-acceptance-exit-status4/4.

Shared scripts/owned_acceptance.lua and scripts/run_owned_acceptance.lua now reuse existing moba_session_process/identity-bound owned-run Windows exit receipts. A nonzero actual exit fails even after intermediate exit=0 or a successful summary. Optional summary must be fresh/bounded/successful; exit-only tasks use actual exit. Exclusive outputs preserve identities/spawn/ready/exit/stdout/stderr/receipt/health/errors. Only recorded original lifetimes are cleaned up. Direct runner source SHA is bound before/after; whole-source batch verification remains separate. Built-in waiting samples current explicit logs at<=170seconds during normal waits. Archived legacy helpers unchanged.

Actual regression:37new acceptance assertions plus50existing platform assertions; three real formal CLI cases fail1/missing-summary1/pass0. Covers successful-summary-then-error, zero-without-required-summary, success, exit-only, stale evidence and PID/token/executable/retirement mismatch. Expected negative failures preserve original logs/receipts and are not runtime failures accepted as success. Consolidated CLI itself used the new formal launcher. Three final workers exited0 with identity-bound raw receipts and retired children/waiters; all CLI launcher sessions reaped and final CIM no owned worker.

Fullsource3456 checked:3452 prior inputs byte-identical+4exactadded shared Lua files, frozen inputs-v1. Prior UE5.8/native86(0failed,4existingwarnings), fresh8Rust1873/0failed/9ignored and57viewedPNG inherited by source-bound proof; no engine/native/fullRust rerun or new visual claim. Archive215 independently reverified files. Evidence:docs/acceptance/2026-10-23-owned-acceptance/index.json. Usage:docs/plans/2026-10-23-owned-acceptance-workflow.md. Errors/root/prevention:docs/plans/2026-10-23-owned-acceptance-errors.md.

Future batches MUST use tools/lua/lua.exe scripts/run_owned_acceptance.lua <spec.json>, with fresh exclusive output and optional fresh result.json; never infer final success from intermediate log strings or use the historical Saved wall-guidance-launch.lua for new jobs. Final baseline omfue/Saved/BuildTemp/owned-acceptance-source-before-v1.json; four frozen inputs-v1. All readers retired/source freeze released. complete-nameplate-screen-spacing and projected-map integration already4/4; resume concrete single-PC gameplay/skill/presentation gaps next, not repeated completed verification. Full combat replica/11external tasks/MVP33/44 remain pending. No true network game/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## 2026-10-23 施法控制／回城正倒數

OpenSpec complete-active-countdown-readability4/4.

Shared native FOmDurationText preserves positive sub-tenth authority seconds as<0.1, exact zero as0, invalid/negative as unavailable, normal countdown decimal precision and optional compact integral trimming. Cast-preview stun/silence/root/item-aim cooldown plus recall header/status countdowns now share it; existing item/summoner cooldown helper delegates without changing ordinary precision. Permanent control is handled before generic negative validation and retains no invented timer. No local tick, gameplay/command/admission changes or hero-name rules. Original successful V1 build/full87report/source and original cropped capture retained; after all readers retired only new fixture window height/footer containment changed.5inputs-v2 retains4other inputs unchanged; fresh BuildV2/fullWorldV2/fullsourceV2 plus explicit proofV3 accepted. FullsourceV2 hash passed but parent binding failed from broad version replacement; new launcher correctly returned exit1/success=false and preserved original evidence. After reader retirement, explicit immutable parent path proofV3 reused the successful full hash component and compared every snapshot/current frozen input. No formal source changed; no redundant full hash/native/Rust rerun. Failed inheritance remains historical only.

Existing real public bridge/control fixture now verifies0.01second stun remains active with exact authority time and positive display. New native boundary/normal/invalid/permanent/control/root/item tests mount actual recall Slate0.01second, capture four shared guidance rows plus real recall strip, and verify zero/retirement/late publication.

UE5.8 BuildV2 and staged DLL/import SHA passed. Full native87:86success+1existingwarningcase,0failed/0notrun,warnings4. Fullsource3458 verifies3453 unchanged+5exactnative inputs-v2 (three existing plus two new); priorfresh8Rust1873/0failed/9ignored and backend inherited by bound source proof, no freshfullRust/backend rerun claimed. Archive489independently verified files/58PNG; 13changed/new actually viewed and45SHAidentical inherited. Four final and three historical successful actual-exit CLI workers have exact Windows exit0; one additional historical failed inheritance worker has exact exit1/success=false. All original child/waiter lifetimes retired; health max151531ms; all launcher sessions reaped, final CIM no owned workers.

Evidence:docs/acceptance/2026-10-23-duration-readability/index.json. Errors/prevention:docs/plans/2026-10-23-duration-readability-errors.md. Final baseline omfue/Saved/BuildTemp/duration-readability-source-before-v2.json,5frozen inputs-v2. All readers retired/source freeze released. Continue concrete single-PC gameplay/skill/presentation gaps; full combat replica/11external/MVP33/44 remain. No true network game/quota reset/commit/push/omfx. Shared actual-exit launcher remains mandatory for new jobs. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.


## 2026-10-23 改鍵後的世界瞄準指引

OpenSpec complete-profile-aware-cast-instructions4/4.

World aiming footer now reads current active WorldCombat/Pressed primary/secondary PointerConfirm, EscapePriority and CancelAim chords, including actual modifiers and localized engine key labels. Intrinsic right-click cancel retained. QuickWithIndicator names actual held key, or generic release if token unknown; missing confirmation explicitly unavailable. Inactive keyboard-movement cancellation is not advertised, and current profile changes are reflected without a cached string. Shared helper applies to the common ability/summoner/item/ward overlay, retaining existing wrap/text scale and scope/lifecycle. No command, profile persistence or input gate changes.

Native fixture verifies default/rebound primary+secondary/modifiers/Escape/missing/inactive/held-release/fresh-help behavior and real owner MatchesPointerConfirm acceptance/rejection/retirement. Actual Slate rows show default/custom/indicator/missing/keyboard movement instructions with full bottom containment assertion; original pixels reviewed. Test-only WITH_DEV friend supports actual owner checks.

UE5.8 BuildV1 and staged DLL/import SHA passed. Full native88:87success+1existingwarningcase,0failed/0notrun,warnings4. Fullsource3460:3456 unchanged+4exactnative inputs-v1 (two existing plus two new), inherits priorfresh8Rust1873/0failed/9ignored and backend by bound source proof; no freshfullRust/backend rerun claimed. Archive149independent files/59PNG; 13changed/new actually viewed and46SHAidentical inherited. Four actual-exit CLI workers exited0 with retired original child/waiter receipts; health max151391ms, all sessions reaped, final CIM no owned worker.

Batch uses immutable explicit configuration: parent index-bound baseline preflight before build/readers, frozen config SHA checked by source consumers/sealer, no broad ancestor version rewriting. Evidence:docs/acceptance/2026-10-23-cast-instruction/index.json. Errors/prevention:docs/plans/2026-10-23-cast-instruction-errors.md. Final baseline omfue/Saved/BuildTemp/cast-instruction-source-before-v1.json,4frozen native inputs/config-v1. All readers retired/source freeze released. Continue concrete single-PC gameplay/skill/presentation gaps; full combat replica/11external/MVP33/44 remain. No true network game/quota reset/commit/push/omfx. New actual-exit launcher required. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.


## 2026-10-23 技能工具提示完整按鍵與圖示操作

OpenSpec complete-ability-tooltip-bindings4/4.

Ability QWER tooltip now lists active primary/secondary cast chords with localized modifier labels. Normal shortcut and every icon-click aiming route reuse shared actual WorldCombat confirmation/cancel help. Quick and indicator shortcut behavior remains distinct, while icon clicks correctly describe forced normal aiming. Missing shortcuts retain complete icon confirmation/cancel instructions; no-target casts remain immediate. Existing tooltip lifecycle/profile gates and world helper are reused without new command paths, local ticking, hero-name special cases or gameplay mutation.

Extended native regression validates secondary modifier cast bindings, actual shared normal confirmation, active/inactive/missing bindings, unbound shortcut icon route, immediate no-target mode, and quick/indicator icon routes via validated profile APIs. Shared Slate rows render full shortcut/icon and unbound instructions with actual bottom containment. This captures production guidance strings, not a live-match input/authority ACK.

UE5.8 BuildV1/staged DLL/import SHA passed. Full native88:87success+1existingwarningcase,0failed/0notrun,warnings4. Fullsource3460:3458unchanged+2exactexisting native inputs-v1. Priorfresh8Rust1873/0failed/9ignored and backend inherited by immutable parent/source proof; no freshfullRust/backend rerun claimed. Archive147independent files/59PNG; 19changed images actually viewed and40SHAidentical inherited. Four actual-exit CLI workers exited0 with retired child/waiter receipts; health max152297ms. All launcher sessions reaped; final owned-process check recorded.

Evidence:docs/acceptance/2026-10-23-tooltip-guidance/index.json. Errors/prevention:docs/plans/2026-10-23-tooltip-guidance-errors.md. Final baseline omfue/Saved/BuildTemp/tooltip-guidance-source-before-v1.json,2frozen native inputs/config-v1. All readers retired/source freeze released. Continue concrete single-PC gameplay and presentation gaps; full combat replica/11external/MVP33/44 remain. No true network game/quota reset/commit/push/omfx. New actual-exit launcher required. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.


## 2026-10-23 瞄準與訊號輪盤實際繪製層級

OpenSpec fix-aim-overlay-layer-order4/4.

Common production aiming/communication overlay now propagates the highest actually used paint layer. Communication center text contributes base+5, cast guidance base+2 and actually drawn attack cursor base+1; early returns retain any prior drawing maximum. Missing owner/inactive/tiny communication rendering returns base. SOmAttackMoveAimOverlay returns this actual result, replacing its previous constant base+1. This honors Slate sibling ordering without arbitrary fixed reservations, draw-element scans, new entity loops, command/input/visibility changes or global z-order changes.

New native regression mounts real production overlays through an automation-only factory. Probe measures actual newly emitted text and the real child paint return, accounting for SCompoundWidget child+1. Real ping and emote model gestures both establish actual text at base+5; inactive/retired owner consumes no extra overlay layers. Two ping wheels render side-by-side: left visible, right covered by a later opaque blue sibling. Every interior right-pane pixel is checked blue, and the original PNG was inspected. Fixture origin0 uses existing production edge clamping; this establishes compositing, not real pointer coordinate/input routing or network ACK. Test-only volatility makes direct fixture state changes paint fresh.

V1 full native failed the new emote fixture arm because it supplied a forbidden world point, followed by a dependent paint assertion. Original full89 failure report/PNG, successful build/stage/capture,4inputs-v1 and exact Windows exit1 retained. After all original readers retired/reaped, only fixture point-kind input changed;3production files stayed byte-identical. V2 fresh build/full89native/fullsource passed, but actual visual review rejected its center-only pixel sample: blue panel did not overlap wheel center. Original V2 successful reports, rejected original PNG,4inputs-v2 and all4 actual exit0 workers retained. After all V2 readers retired/reaped, fixture changed to full-pane coverage and visible reference wheel. V3 frozen4inputs-v3 still retains all3production files byte-identical toV1/V2. No production input validation weakened and no rejected pixels accepted. Errors/root/prevention recorded in docs/plans/2026-10-23-aim-layer-errors.md.

UE5.8 BuildV3/staged DLL/import SHA passed. Final FullWorldV3 native89:88success+1existingwarningcase,0failed/0notrun,warnings4. Fullsource3461:3457unchanged+4exactnative inputs-v3 (three existing plus one new). Priorfresh8Rust1873/0failed/9ignored and backend inherited by immutable parent/source proof; no freshfullRust/backend rerun claimed. Archive818independent files/60acceptedPNG; 14changed/new actually viewed and46SHAidentical inherited. Four final actual-exit workers exited0. Seven historical workers include one actual failed native exit1 and six actual exit0; all original child/waiter lifetimes retired, all launcher sessions and sealer reaped. Health max151484ms; final owned-process check recorded.

Evidence:docs/acceptance/2026-10-23-aim-layer/index.json. Final baseline omfue/Saved/BuildTemp/aim-layer-source-before-v3.json,4frozen native inputs/config-v3. All readers retired/source freeze released. Continue concrete single-PC gameplay and presentation gaps; full combat replica/11external/MVP33/44 remain. No true network game/quota reset/commit/push/omfx. Actual-exit launcher required. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.

V3 startup has a separately preserved engine Dataflow Date member-initialization diagnostic (2entries before selected tests). It was absent in4compared prior logs; exact local engine header initializes reflected Date with wall-clock FDateTime::Now(), a source-based explanation for the non-deterministic initialization comparison. Exact header/SHA, raw diagnostic and comparison retained under engine-diagnostic; no engine edits/suppression. Final native89 still has0failed/0notrun/4warnings; no claim of diagnostic-free engine startup.


Next concrete batch: investigate actual communication wheel pointer-to-paint coordinates at nonzero window positions and logical scales, through the real owned viewport route. V2 cached-root-origin fixture mismatch is an observation, not proof of a gameplay defect; V3 origin0/compositing acceptance does not resolve that question. See docs/plans/2026-10-24-communication-coordinate-followup.md. Use final aim-layer source-before-v3.json and index-bound parent proof, do not repeat accepted layer work. No current workers remain.


## 2026-10-24 視窗位移下的瞄準／輪盤座標

OpenSpec fix-desktop-to-paint-coordinates4/4.

Confirmed engine-space cause of prior nonzero-window mismatch: SWidget caches desktop geometry after appending current FPaintArgs window translation, while OnPaint allotted geometry remains window-relative. Shared checked FOmSlatePaintSpace now subtracts the current paint window translation before inverse geometry, rejecting nonfinite/singular inputs and clearing output on failure. Current offset propagates through production overlay to communication origin, attack range/cursor, cast hint anchor, world outlines, marker and source guide. No fixed offset/DPI guess, cached prior window location, command/input/visibility/selection change, new per-entity loop or hero-name rule. Existing actual maximum paint-layer propagation is preserved.

Native geometry round trips cover4layout scales and signed window translations; legacy direct inverse disagrees. Actual production overlay is mounted at2nonzero moved native-window positions and3render scales. FPointerEvent screen origin arms the real gesture model; probe compares actual emitted center text render transform plus actual current paint window offset against that desktop origin within1pixel. Invalid coordinate/singular layout and retirement cases covered; two original window captures inspected. This establishes event coordinate convention and production presentation, not successful live owner-context admission, true network ACK or physical OS DPI/controller acceptance. Existing ownership/input tests are included in full native suite.

UE5.8 BuildV1/staged DLL/import SHA passed. Full native90:89success+1existingwarningcase,0failed/0notrun,warnings4. Fullsource3463:3458unchanged+5exactnative inputs-v1 (three existing plus two new). Priorfresh8Rust1873/0failed/9ignored and backend inherited by immutable parent/source proof; no freshfullRust/backend rerun claimed. Archive158independent files/62acceptedPNG; 15changed/new actually viewed and47SHAidentical inherited. Four final actual-exit workers exited0 with retired original child/waiter receipts; all launcher sessions/sealer reaped. Health max151719ms; final owned-process check recorded.

Evidence:docs/acceptance/2026-10-24-paint-space/index.json. Errors/root/prevention:docs/plans/2026-10-24-paint-space-errors.md. Final baseline omfue/Saved/BuildTemp/paint-space-source-before-v1.json,5frozen native inputs/config-v1. All readers retired/source freeze released. Prior communication coordinate presentation question is resolved by actual draw measurements; full owned admission and hardware/true-network requirements remain distinct. Continue concrete single-PC gameplay/presentation gaps; full combat replica/11external/MVP33/44 remain. No true network game/quota reset/commit/push/omfx. Actual-exit launcher required. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.


## 2026-10-24 主動方向扇形與散射配裝

OpenSpec complete-active-directional-cone4/4.

Added closed data-driven ConeDamage: one exclusive instant active/ultimate point effect with per-rank range/damage/half-angle. Q10 half-angle conversion uses the existing shared4096-turn sector predicate. A distinct in-range aim point supplies direction; sector reaches full rank range. Living enemy candidates are canonical(id,generation) sorted/deduplicated, geometry-filtered, then admitted through bounded atomic damage settlement. Invalid direction/range/data or >128resolved hits rejects before commit. Existing damage attribution/authority remain; no false circular area cue.

Append-only ranger_scatter and optional training_scatter_ranger loadout reuse existing ranger visual data. Original training_ranger and ranger_volley circle stay unchanged. No hero-name execution branch. Common generated rank ticks flow through typed HUD payload to owner cache, sector outline, distinct missing-direction instruction and actual skill tooltip. Unknown/replaced/retired states clear geometry. Native outline is read-only floating-point aiming guidance; authoritative fixed-point predicate decides exact hits. No terrain/projectile blocking promise, live-match owner-admission or network ACK claim.

Coverage: strict shared model and Lua11case boundaries (including correctedf32 pi/2 limit), append-only IDs/loadout, all-rank canonical target/dedup/sector radial-angular boundary/overflow/zero-direction/overrange tests. Formal60Hz actual cast verifies forward hit110, reverse empty cast, zero/overrange rejection, resources/cooldown and pause. Native generated rank geometry/owner state/retirement/tooltip regressions plus actual Slate tooltip capture. Hero lifecycle matrix now resolves every generated native class and typed metadata automatically:8heroes/30active+ultimate casts actually exercise real clips/pooledVFX/dedup plus death/respawn/pause/concealment/generation lifecycle. No second hero/skill-name list.

Fresh8Rust 1877passed/0failed/9ignored; fresh formal backend and UE5.8 build/stagedDLL/import SHA passed. Fullnative 91:90success+1existingwarningcase,0failed/0notrun,warnings4. Fullsource 3468: 3429unchanged+39exactchanged inputs against index-bound paint-space parent. Rust/backend results are fresh, not inherited. After that cohort, only2presentation assets and1native lifecycle test changed; immutable full source proofs bind those repairs. Final baseline derives from the successfully verified asset snapshot plus exactly that one frozen native test edit, then passes the whole3468byte/size/mtime check. Archive 870independent files/63PNG; 14actually viewed and49SHAidentical inherited. All original job child/waiter receipts retain actual Windows exits and retirement; health max151921ms. Original failed generation/offline-retry and Lua boundary cohorts remain archived.

Evidence:docs/acceptance/2026-10-24-active-cone/index.json. Errors/root/prevention:docs/plans/2026-10-24-active-cone-errors.md. Finalbaseline omfue/Saved/BuildTemp/active-cone-source-before-v4.json. No remaining readers after final retirement; source freeze released. Full combat replica/11external/MVP33/44 remain. No true network game/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.


2026-10-24裝備吸血盤點補充：商店現有撤銷只支援購買；出售撤銷尚未實作，本批只驗證出售後無法套用舊購買撤銷，且不恢復裝備／吸血或HP。出售撤銷列入後續可單機驗證的玩法缺口。


2026-10-24出售撤銷後續盤點：本批已實作並完整單機驗收，取代前述裝備吸血批次當時的「只支援購買撤銷」限制；原歷史敘述保留，不代表目前狀態。商店共18項，吸血與暴擊裝備各兩項。無人值守期限依使用者延長至2026-10-26 23:59:59 Asia/Taipei。


2026-10-24唯一被動：本批取代先前「更多唯一被動待擴充」中的基礎共用機制缺口。19項道具含新增疾步長靴；一般屬性與不同群組可疊加，同群組唯一被動不疊加。更多獨立玩法與11外部接受仍pending。


2026-10-24購買群組：基礎資料驅動持有上限與合成／undo守門已完成單機接受，取代前述「購買互斥規則待完成」的基礎機制缺口。唯一被動去重與購買限制是兩個獨立契約；目前19項，鞋靴群組限1件。其他玩法與11外部接受仍pending。


2026-10-24獨立地面區域：共用生命期機制及炎刃四級燃燒已完成單機接受，炎襲維持死亡取消；取代先前基礎獨立地面區域生命期待完成缺口。更多線性／多階段／召喚控制及11外部接受仍pending。


2026-10-24直線技能：新增通用瞬發非位移直線傷害、原生寬度預覽與正確地形／定身提示、公開觀察bot選點及第9個可選配裝。獨立區域生命期已於前批完成。更多移動／追蹤彈體、引導／多段／召喚控制與11外部接受仍pending。
