## Context

2026-10-04 fog rebase：manifest v2以新hash domain綁typed grid，v1只允許無grid並保留舊hash契約；恢復保留同隊同場探索，projector與後續frame／bootstrap同步epoch。runtime先驗grid與完整chunk／baseline，再採用恢復retention；主／catch-up共用critical reset→恢復snapshot與latest保留，避免等下一個gameplay frame。四個指定core測試及runtime check成功，尚未做Specs完整對局／KCP／雙UE恢復整合驗收，不勾選4.3／6.1全項。詳見authority-fog-rebase與E143，20/30不變。

2026-10-04 Unreal三態小地圖：共用MinimapModel驗證ABI11／綁定team／epoch／tick／checked Q10 geometry與4096三態後複製lease資料；control保留、full reset清空。row-run overlay覆蓋公開背景與grid外／padding，live／memory仍按安全披露畫在上層，不改target gate。正常build-only與本功能Editor一次1/1通過；未做PIE像素／雙UE／完整rebase／效能整合，不勾選6.1／6.2全項。詳見unreal-minimap-fog-grid與E142，20/30不變。

2026-10-04 正式 fog typed IPC／bridge：protobuf optional FogGridPresentation schema1 傳 team／epoch／sample tick／Q10 geometry／row-major 三態，core 共用驗證後轉換，不擴 gameplay hash 或 hidden 身分。runtime 鎖定 formal session；bridge 正式欄位存在時不回退 legacy overlay。C ABI11 的 optional OmFogGrid／cells 由 frame lease 自有，busy retry／reset／舊 lease 測試通過。typed IPC 是增量，Unreal 三態绘製與完整 rebase 尚未完成；詳見 authority-fog-ipc-bridge 與 E140，20/30 不變。

2026-10-04 權威 fog 發布：compiled Lua map 的公開路線／camp／terrain 產生最多 4096 格 Q10 包圍盒，Wave B 保存 immutable view，team projector 預設每 6 ticks 取樣。專用非 gameplay event 在 padding 前加入，私人 bootstrap 保存最新 FG01；runtime 嚴格 audience／tick／geometry／重複與衝突驗證後保留，缺更新不猜視野，verified rebase 清空等待新 epoch。這是正常更新與 TeamGameStart 的接軌，不宣稱 rebase manifest／typed IPC／ABI／UE 已完成。詳見 authority-fog-publication 進度與 E139，20/30 不變。

2026-10-04 權威fog核心：新AuthorityFogGrid從immutable WaveBReadView正式本隊source／共用LOS取樣，geometry Q10負座標／4096容量／checked spans，三態exploration與advancing epoch reset。FG01傳team／epoch／tick／geometry／row-major cells、不傳source或hidden entities；先驗wire再配置。同tick冪等／倒退拒絕，不替代entity stealth／delay或target gate，不塞每60Hz hot path。網路發布與UE尚未串接，詳見authority-fog-grid與E138。

2026-10-04 正式fog邊界：既有IPC fog來自DemoFogCache，不是正式grid／explored契約。production envelope以safe phase marker識別正式MOBA並停用整組demo投影，session保留界線避免缺HUD／reset回落。小地圖VISION N/A不冒充已探索或全可見，authority披露不變；真正fog須後續geometry／provenance／explored契約。详見formal-fog-boundary與E137，不勾選6.1／6.2。

2026-10-04 小地圖隊伍辨識：owner為player ID，不是team。已披露live marker以公開scoreboard roster原子驗證後映射，configured player須一致才能用local team；缺值／非法／未知顯示灰色，不猜敵我或新增位置。自己／隊友／敵方共用native顏色，frozen memory仍不帶team。high-bit owner保留既有u32位元模式，-1哨兵歧義不擅自改ABI；詳見minimap-teams進度與E136。

2026-10-04 小地圖安全記憶：既有ABI10 frozen ghost獨立複製為UI記憶型別，不含live entity reference／owner，不猜team。render ID去重／live優先、epoch與finite bounds gate；公共route／terrain定範圍，記憶不能擴張範圍。Slate空心暗色／不同圖層，完整快照取代、control保留、Stop清空，不新增前端TTL或hidden motion。只做本功能單輪確認，完整驗收留最後；詳見minimap-memory進度與E135。

2026-10-04 小地圖公開地形層：沿ABI10 terrain_rects複製至UI自有矩形，與route共同建立等比例示意範圍；Slate地形／路線／可見單位分層，不查actor或vision polygon，不自行判定可走性。terrain-only view也可呈現，非法batch全清／control保留／full reset清空。依使用者新指示，完整驗收集中最後；新增指定已知功能單輪與獨立報告入口避免反覆全套驗收。詳見minimap-terrain進度與E134，6.1／6.2仍未全項完成。

2026-10-04 通用地形呈現：validated compiled public map保留到bridge PresentationExtras，C ABI10新增frame-owned terrain_rects與ID；不混入vision polygons。共用UE ISMC依mesh bounds生成footprint／center，visual height可配置，關掉collision／overlap／nav；相同geometry不重建，control保留／full empty與Stop清除／fresh view重建。新內容只改Lua地圖／美術，不新增map專屬graph或C++；最後驗收見unreal-collision-terrain進度與E133，完整5.4／6.1仍未完成。

2026-10-04 共用地圖契約：compiled_blocked_regions統一權威初始化與初始bootstrap的碰撞轉換；runtime bootstrap與RuntimeReady要求map id／hash伴隨唯一schema1且canonical bytes一致的地形，缺失／重複／異動拒絕，不fallback空碰撞。legacy無compiled identity不改；公開地形decoder依剩餘bytes檢查count再配置。這是地形呈現前提，不是視野遮蔽物或完整5.4，詳見compiled-map-contract進度與E132。

2026-10-04 Unreal三路layout決策：既有MapRouteActor／Slate只因bridge依lane_length產生單路而缺三路。public bootstrap map/moba-layout schema1指定compiled map id／catalog hash，runtime bootstrap與bridge RuntimeReady additive7／8嚴格驗同catalog，恢復Lua完整lane waypoints；full與lifecycle共用、缺HUD清route、None legacy保持。沿既有C ABI route列表／共用C++，不新增角色graph或第二份AI，不把terrain當vision occluder。新增--three-lane60Hz基礎map／minimap與Move驗收，不沿用商店／升級等singlelane劇本。營地地形美術／完整建築仍未完成，最後結果見Unreal three-lane layout進度與E131。

2026-10-04 NPC地形決策：英雄既有bounded static_next_waypoint抽為共用，三路creep追擊／回兵線與jungle追擊／回位接直線優先static_step_toward與完整waypoint檢查static_advance_route。每次位移皆swept-circle，20-unit NPC envelope；只查public BlockedRegions，不傳私有aggro／route cursor，不改None單路／TD。短步檢查曾導致繞障後走回牆邊，改查完整waypoint並要求抵達／回位／Heal證據。Lua兵線與camp leash生成限制不放寬；新900tick三seed fixture共5400雙隊steps每tickhash零repair通過，不是任意地形navmesh或UE layout完成。詳見NPC terrain進度與E130。

2026-10-04 Lua地形接入：moba_maps.terrain 使用最多32個整數矩形 id／min／max，共用驗證產生MobaTerrainConst並進入map catalog hash／compiled agreement／hot reload拒絕。compiled map在所有配置／roster驗證通過後安裝公開BlockedRegions，None不改TD／既有單路。沿server既有public bootstrap metadata與filtered builder，不傳私有AI狀態；不假設碰撞同時遮蔽視野。NPC通用detour尚未完成，先禁止擋住100-unit兵線corridor、五人出生點和野怪leash＋100；矩形stack buffer與fixed broad phase減少熱路徑成本，不宣稱60FPS。真實Lua地形三seed各1200tick共7200雙隊steps逐tick零repair／hash／正式輸入detour抵達通過；完整回歸／KCP／stage以Lua terrain進度與E129最終結果為準。5.4仍未完成。

2026-10-04 野區增量決策：Lua moba_maps.jungle_camps 提供整數座標、HP、攻擊、移速、攻擊範圍／間隔、leash、重生秒數、Gold／XP；build-time 驗證並生成 MobaJungleConst，沿既有 compiled map catalog hash／hot reload 拒絕。MobaMatch 另持營地狀態，不偽造第三支玩家隊伍、不混入 lane tower index。新增 HostileNeutral，保留 legacy Neutral 的不可交戰行為；安全 render kind=3 將中立可交戰性帶入 filtered ECS，bridge 通用 Creep fallback，不新增角色 C++／藍圖 graph。仇恨受正傷害的 roster 英雄觸發、超距／死亡清仇恨回位、回位免傷與到家正式 Heal、first-lethal 記帳一次獎勵與新 generation 重生。所有 AI／timer 留權威 resource，沒有 VisionSource；可見 motion／vitals 沿現有 ordered facts，filtered 不重演私有 AI。這是每營地單隻的原型，非大型野怪 Buff、完整野区美術或整項 5.4；詳見 docs/plans/2026-10-04-moba-jungle-60hz-progress.md 與 E127。

2026-10-04 三路第一段決策：SingleLaneConfig保留None預設，map_id選compiled Lua three_lane_training；地圖宣告三路整數waypoints與tower_offset，Rust validates／codegen constants、shared catalog hash與hot reload拒絕。MobaMatch持有每路每隊tower和route cursor，legacy towers欄位只是第一存活塔的HUD／Bot相容摘要，base unlock只信lane_towers全部retired。AI跨路creep／tower不搶線，英雄仍可被附近NPC攻擊；第三座塔正式Death retire才解除傷害入口gate。這是折線導航與單層塔原型，不是地形navmesh或完整5.4，public map layout／UE小地圖與野區後續繼續。詳見docs/plans/2026-10-04-three-lane-navigation-60hz-progress.md與E126。

2026-10-04 數值規則收斂：首次學R後普通R正式delegate與原input3、Lua140HP權威exact治療、5,883 raw/IPC snapshots及post-cast三方hash完成，control-only輸入結果需先於HUD gate讀取；hero每tickfinal HP來自kind22 EquipmentStats，不假設稀疏kind17 Vitals每tick存在。saved-run PID可被Windows重用，只讀核對原role／pid／exe，異exe證明非原程序、同exe或無identity保守拒絕、不stop無關程序。重新跑core334／base98／server155、舊kill-assist XP／lane XP／UE商店／B回城保存verifier後，以5.3合法／非法數值規則條件封關20/30；三路／LAN／完整UI／mana／全部cue／效能仍獨立未完成，下一優先5.4。不再因歷史進度文件的當時缺項永久不勾選；證據矩陣見docs/plans/2026-10-04-moba-rules-60hz-acceptance.md。

2026-10-04 Unreal首次學習決策：獨立UE_FIRST_LEARN opt-in與旧rank升級模式互斥，server-owned apprentice四槽0／出生SP1不改Lua預設；沿共用Ctrl+Q唯一正式delegate與正常IPC／KCP，runtime internal injection全部關閉。允許renderer晚連線前正常XP升級，但學前SP必須等於level、學後含level delta精確扣一；四槽CD0／原input一次／UI exact tick及raw authority逐筆核對。修正共用WorldBridge的cast／learning ID混用：rank0 cast仍禁用，learning獨立保留catalog。真實60Hz雙UE1791104682、5932 snapshots、每隊25 unique三方checkpoint及五PID退出／保存SHA通過；新長名adaptive width修正確定layout不足，但E121局部像素截字仍存在。不是OS鍵盤注入、學後傷害施法或完整UI；完整5.3／6.2維持未勾選。執行與E124見docs/plans/2026-10-04-unreal-first-learning-60hz-progress.md。

2026-10-04 首次學習網路增量：純Lua training_apprentice四槽rank0／一點，append-only獨立技能避免逐英雄FFI重註冊；server-owned AUTHENTICATED_HERO_BINDINGS限定合法roster與compiled active hero，缺值保持舊英雄，不引入client任意選角。真實KCP60Hz正常renderer intent injection→正式input：未學施法／level6拒絕不改rank-SP-CD、同request學習只接受一次、rank0→1後成功施法；1,582 raw/IPC snapshots、雙隊最終10／9 unique checkpoint與post-learning9／7通過、三PID獨立退出。只讀Lua保存verifier核對所有三方紀錄與SHA，容許launcher取樣後退出前合法追加，但不得丟原末次tick或忽略FAIL。UE完整生成／編譯與stage通過，不等同本輪Unreal rank0 Ctrl操作／像素或60FPS，完整5.3／6.2仍未完成。詳見docs/plans/2026-10-04-moba-first-learning-network-60hz-progress.md與E123。

2026-10-04 技能門檻增量：共用 per-rank required_hero_level（缺值1、1..25、單調不下降、rank數等於max_level）同時生成Rust與Unreal registry，MOBA扣SP前權威驗證，compiled agreement與hot-reload gate阻止单邊改資料。lumen_lance明確[1,6,11,16]，不是依槽位猜ultimate；共用native UI讀同資料顯示可升級與原因。現有四招初始rank1；rank1高於出生level明確拒絕開局，rank0／延後首次學習仍待做。60Hz正式升級傷害80→125與6.5秒CD、3902筆真實KCP snapshots及兩輪17/17 UE automation通過，不冒充實體Ctrl鍵、升級HUD像素或完整5.3／6.2。詳見docs/plans/2026-10-04-moba-upgrade-requirements-progress.md與E119。

完整架構與階段驗收見 `docs/superpowers/specs/2026-09-25-unreal-rust-moba-framework-design.md`。目前 `omoba-client-runtime` 已有 filtered replica 與 localhost protobuf IPC，但呈現包含示範用資料；`omfue/bridge` 仍有自己的玩法 world 與直連路徑。`omfue/codegen` 可從 Lua 產生 C++，但包含 Saika 專屬分支。`BpGeneratorUltimate` 已在專案中提供 Editor MCP 通道與資產、Blueprint、PIE 工具。工作樹目前另有未提交變更，實作須逐檔保留並整合。

## Goals / Non-Goals

2026-10-04 兵線 XP 決策：Lua pool25／radius1200，權威 tracked Creep 真實死亡一次性移除後，固定點範圍內存活敵隊 roster 平均整數 floor，不要求 last hit、不分配餘數。正式單路停用 legacy Bounty 避免雙付，舊 MOBA／TD 不改；沿既有 progression／equipment 安全投影與通用 HUD，無角色 C++／BP。規則與全內容 hash／compiled agreement一致，不能 hot reload。此為經驗增量，不代表兵線金錢／技能升級或5.3全項；生命週期测试以正式守方撤離輸入避免把 Bot平衡混入 XP驗收，候選 Bot變更已撤回。詳見docs/plans/2026-10-04-moba-lane-xp-60hz-progress.md與E117。

多人 roster 增量（2026-10-04）：保留預設1v1並增加explicit additional_players／每隊最多五人，英雄 slot Vec保存side，spawn／death／respawn／shop／Recall基地／助攻與owner economy不再把player index當team side。private respawn與Recall兩metric以namespace＋完整u32 player ID識別，filtered核對(team,player)清命令，runtime只讀configured owner，Recall協商升version2。實際雙助手（含死亡）ECS／60Hz replay與三英雄640tick双隊零repair完整hash通過，真實KCP1791076242三個外部runtime／三玩家Move與獨立第三玩家parity通過；不冒充網路助攻、5v5 UE或完整5.3。詳見roster-assist60Hz進度與E110。

助攻核心增量（2026-10-04）：Lua100 Gold／10 active seconds生成規則、正傷害依generation-packed victim與player去重，合法同隊hero killer結算窗口內每名參與者一次，NPC／匿名死亡不猜歸屬。authority-private帳本與assists進replay digest，不傳敵方參與或wire correlation；匿名致死→Heal也標生命退休。固定1v1 roster不可能產生真實同隊助攻，先以純核心多人fixtures與2人正式ECS／60Hz KCP回歸驗規則，不把它稱作完整助攻。下一步擴roster時必須連同owner-scoped HUD／recall安全投影，不能以team index充player index。詳見assist-core進度與E109，5.3不勾選。

回城核心增量（2026-10-04）：獨立Recall tag19，Lua8秒生成權威Fixed64 deadline，重送不延長／pause不計時；開始清命令並停自動攻擊，行動意圖、位置改變、正傷害／ScriptDirectDamage、死亡／基地消失取消，post-step傷害後才傳送。MovementPriority與committed pose沿既有安全投影，filtered不持有MobaMatch；尚無channel capability／IPC／UE B鍵／讀條HUD，secure admission與canonical acceptance保持明確拒絕，不用headless冒充網路回城。TD省略MOBA section維持相容預設。詳見recall-60hz-progress與E106，5.3仍未全項完成。

英雄直接擊殺增量（2026-10-04）：Lua hero_kill_gold 編譯為共用 Rust 規則，禁止單邊hot reload／compiled mismatch；獎勵只接權威 Damage 首次致死，不接ACK／Death通知。每英雄生命的lethal_pending在首個致死時標記、spawn才清除，防同batch治療後再致死重付；只認當前敵方hero entity、Playing且未pause。Gold／kills飽和、死亡重生保留，owner economy沿既有安全投影，不改角色C++／BP graph。先新增deterministic60Hz與完整filtered回歸，runtime smoke所有模式預設60；這不是UE frame-time保證，助攻／回城／召喚物或退休entity歸屬／擊殺UI仍未完成，5.3保持未勾選。詳見hero-kill-60hz-progress與E105。

本段小地圖決策（2026-10-04）：通用原生 Slate 只呈現安全完整快照的 entities 與公開 route points；不讀 actor 世界補位置、不把 frozen memory 當 live target，不需要角色 C++／Blueprint graph。formal IPC 原先無 paths，加入權威 SingleLaneConfig 的公開 lane-length HUD metric 與 additive protobuf 欄位（Q10、0表示未知）；runtime 過濾長度，bridge 僅將有效公開長度轉成單路端點。Unreal 依 route extent 產生 square schematic、10% padding／aspect preserving／Y inversion；不是正式可行走 map bounds。控制frame保留、完整empty/reset及Stop清空。尚無 fog tile繪製、記憶標記、點擊命令、五人隊伍色或三路地形；6.2仍未完成。錯誤與驗收詳見 `docs/plans/unreal-moba-error-register.md` E098 與 minimap progress。

小地圖互動增量（2026-10-04）：同一Slate投影的逆轉只接受有效範圍右鍵Move／Shift排隊；留白／無map／非finite／非法scale／離線不送命令，所有HUD點擊Handled避免穿透，GameplayInputEvent.bConsumed仍false。只用configured owner與通用Point target，不選取entity或查actor，沿既有SubmitGameplayInputEvent→Rust權威輸入路徑，不新增C++玩法或Rust測試入口。非Shipping opt-in以真實cached geometry呼叫原生pointer handler；必須exact一次回呼、實際逆轉點與預期誤差≤0.05、原input ID status0及移動後三方hash，失敗不能直呼API繞過。這不是OS／完整hit-test或導航bounds驗收；完整6.2仍未完成。詳見E099與minimap-input進度檔。

終局UI增量（2026-10-04）：通用native結算面板只呈現正式HUD的Finished／winner／elapsed與runtime成功Start時保存的明確team設定；沿用IPC身分檢查，不以player ID推導team、不新增ABI。未知team或無效資料隱藏，TD／新非終局／Stop／EndPlay清空，死亡玩家仍顯示結果。面板只顯示Victory／Defeat／Draw與權威勝者／時間，沒有尚無契約的重新開局或自算計分按鈕。opt-in驗收要求實際viewport layout、每隊終局截圖、死亡重生後一致結果與UI後120 ticks三方hash；仍不代表完整選角到結算或真人操作，6.2維持未完成。詳見E100與終局UI60Hz進度檔。

60Hz診斷metadata（2026-10-04）：RuntimeReadyPresentation additive tag6直接傳bootstrap tick_rate_hz，renderer驗明確bound player/team後才顯示支援rate。presentation snapshot cadence獨立，不可猜權威rate；缺欄位／不支援值為未知，不能預設120Hz換算時間。頂端Observed僅表snapshot接收觀測，不宣稱client gameplay step；刪除未量測的lag=0t，不以單張FPS60截圖宣稱穩定60FPS。詳見E101。

**Goals:**
- 一個權威 Rust 對局核心，同時支援單機與 LAN。
- Unreal 只負責輸入與呈現，英雄內容由 Lua/Rust 與生成器驅動。
- 資產與 PIE 操作能透過 BpGeneratorUltimate 重跑與驗證。

**Non-Goals:**
- 首版不加入帳號、排位、配對與商業營運系統。
- 不把 Editor MCP 當成遊戲執行期依賴。

## Decisions

- 2026-10-04 Unreal升級綁定：普通QWER與Ctrl+QWER分開註冊FInputChord，Ctrl升級走通用反射／正式IPC／KCP，不以測試旗標覆寫Ctrl狀態。非Shipping opt-in只執行唯一正式delegate一次，等兵線XP取得SP再操作；queued與ACK不足以封關，需原始rank／SP／XP、實際native文字／PNG與操作後120tick三方hash。技能格固定128×128、內寬116；ASCII content ID／rank-CD／Ctrl提示拆三個SimpleTextMode單行block，不使用AutoWrap與shaping cache。此前固定wrap與三frame等待未完整解決，保留PNG反例，不推定引擎根因；未來localized display name不能直接套用ASCII布局限制。此共用框架變更不新增英雄C++／Blueprint graph。完整驗收與限制見docs/plans/2026-10-04-unreal-ability-upgrade-key-60hz-progress.md及E120。

- 2026-10-04 技能升級：沿用正式UpgradeAbility、以Lua max_level及權威SP／phase／pause／alive／roster拒絕非法輸入。敵方input仍私有，visible-only CommittedAbilityRanks(kind25)發布四槽絕對rank，filtered原子驗compiled上限再更新，不重扣SP或清CD。RendererInput19新增upgrade，same-runtime有界request去重；Unreal共用Ctrl+QWER不新增每英雄graph。真實60Hz雙送去重run1791091387及3950筆wire／IPC核對通過，但Lua rank英雄等級門檻／實際Ctrl操作／升級效果仍待驗收，不標完整5.3。

- 2026-10-04 XP增量：Lua hero_kill_xp100／hero_assist_xp50由compiled constants與full rules hash鎖定，權威現有正傷害／lethal_pending／assist ledger結算一次；活體ECS與死亡slot持久進度分開處理。MOBA以整數floor(100×6^(level-1)/5^(level-1))門檻多級升級至25、每級一SP，level delta保留裝備且不補HP，既有TD方法不改。沿已存在的CommittedProgression／EquipmentStats投影，不傳敵方ledger或前端重付XP。兵線Bounty舊路徑／正式技能升級輸入與UI仍待統一，5.3不勾選。HP0 pending-deletion不能以entity存在冒充存活，通用effect preflight必須同時檢查HP；詳見XP60Hz進度檔／E116。

- 計分板畫面驗收增量（2026-10-04）：非Shipping明確flag只在正式Playing／viewport／原生panel geometry就緒時截圖，不注入計分。值相同仍須重試layout；雙隊1v1固定fixture驗PNG、UI資料與wire／IPC exact tick，要求UI後120 ticks三方hash。初始畫面不冒充擊殺更新或十人驗收，完整6.2保持未完成。

- 2026-10-04 公開計分板：持久 roster／KDA 由權威獨立 AllPlayers metrics 發布，原 Gold／物品／owner KDA audience 不變；不依 entity 可見性推算。runtime 原子驗證最多兩隊十人完整列、bridge 核對 owner，ABI9 使用 frame-owned 固定十列；Unreal 只渲染共用唯讀 Slate。完整 presentation reset 缺少 HUD 時清除、control-only 保留，不以此宣稱完整 UI 或多人實戰畫面驗收。

網路助攻驗收增量（2026-10-04）：以test-mode且明確COMBAT opt-in的三runtime只讀披露資料提交普通MoveTo／AttackTarget，不以注入傷害或ACK算assist。hero_tick近零方向保留當前facing仍正常windup／impact，不能因位置重合抑制合法攻擊；非零轉向與既有range／faction／HP gate不變。1791078192真實60Hz一擊殺一助攻，3829筆IPC score／Gold對原始wire及Lua獎勵精確核對、三人各2post-kill parity通過。此為runtime intent injection，不稱Unreal／TCP renderer操作或完整5.3。

美術替換驗收（2026-10-04）：以已owned RecipeV1的原生英雄來源atlas與Lua native_visual配置作代表性驗證，模板actor玩法不改，生成／MCP／actual PIE負責套用。測試來源二進位備份、不同PNG尺寸獨立讀回、實際渲染与第二輪套件SHA冪等皆必須通過；所有測試來源及Lua override最後還原並重建原版。共用驗收不能把縮放1.2寫死；Editor讀生成CDO、PIE讀同一Lua配置，但仍保留值與render斷言。這可封關6.3的內容替換流程，不代表Skeleton retarget或全英雄動畫品質已完成，詳見美術替換進度檔／E102。

1. 採用 `omb` → `omoba-client-runtime` → localhost IPC → `omfue`。替代方案是在 `omfue/bridge` 延續獨立 world；這會複製視野、同步與重連邏輯，因此只作過渡相容路徑。
2. Lua 是靜態資料與可組合 effect 的來源；特殊行為由 `base_content.dll` 中的 Rust handler 執行。替代方案是任意 Lua 轉譯 Rust；現有架構沒有此能力，也難維持確定性。
3. 擴充現有 `omfue/codegen` 生成薄 Unreal 類別、registry 與資產配方；移除角色專屬生成分支。替代方案是逐英雄 Blueprint/C++，無法符合內容製作目標。
4. BpGeneratorUltimate 只在 Editor 啟動時套用配方與驗證 PIE；正式遊戲由 IPC 驅動。替代方案是把 MCP 放進遊戲執行流程，會造成 Editor 耦合。
5. 先修復建置與內容版本檢查，再打通 IPC，之後實作 MOBA 規則與呈現。每階段都以可執行證據封關。

## 執行優先順序修正（2026-10-03）

已有生成／Editor／IPC 基線後，停止把 Saika 相容層與 renderer 補強當作完整遊戲的完成標準。先封關真正可結束的單路 headless 對局（5.1／5.2），再將同一玩法接入正式 server／filtered runtime／Unreal，使玩家可以操作四招、死亡重生、推塔到勝負；最後擴展經濟、三路與完整 UI。2.2b／4.3 僅在阻擋此垂直流程時優先修正，不移除仍被資產參照的相容介面。

單路 headless 以 `create_single_lane_world` 建立空的正式 ECS、載入 generated manifest、安裝 opt-in MobaMatch；SimulationDriver 與 server tick 皆有相同 begin／commit hooks。既有 TD 不安裝此資源。小兵不使用 TD leak 路徑，兵塔將 DamageInstance 交給既有戰鬥管線；英雄沿用 hero_tick／正式輸入／script dispatch。四招在 level 1 學會，商店、mana、三路與選角 UI 仍由後續任务驗收。詳細決定與實際 DLL／120Hz 證據見 `docs/plans/2026-10-03-single-lane-moba-progress.md`。

## Risks / Trade-offs

- 2026-10-04：60Hz同步與文字像素缺陷分離验收。Opt-in呈現backbuffer取證不重畫Slate、不改玩法；Shipping／一般遊戲不做GPU readback，module shutdown移除callback。RHI只接受d3d11／d3d12並核對實際日誌，保留D3D11 default；切後端未解決截字。close-window timeout後stop必須等待退出，不把提出停止當清理完成。實際19/19兩輪／PIE及两輪raw／三方hash／PID退出通過；完整6.2與GPU字形缺陷未封關，詳見unreal-text-paint-60hz-progress與E121。

- 現有未提交的 `omfue` 與啟動工具改動可能與新路徑重疊 → 先讀取 diff，避免覆蓋；優先實作獨立生成器與測試，再逐點整合。
- IPC 目前部分欄位是示範投影 → 先明確標記並加入正式 MOBA schema 與視野測試。
- 腳本 DLL 與 Rust host 需相同 rustc → 建置入口固定 toolchain，先建 DLL 再 stage，啟動時檢查版本。
- Editor MCP 可能未啟動 → 保留資產配方與待處理報告，不能將未驗證資產標成成功。

## Migration Plan

2026-10-04：保存 Saika 子 BP 已只實作 generic 動畫／攻擊事件後，移除三個 hook 的 legacy 自動轉派及專屬 payload maker／fallback；normal dispatch 不再看四技能 ID。reflected typed API／metadata 保留供明確相容呼叫，不新增 legacy enable 旗標、不擴充新英雄特例。probe 分開確認 generic 欄位、零舊回呼與 explicit compatibility；完整品質驗收留最後，詳見 generic-native-event-dispatch 進度檔。

2026-10-04：既有 Blueprint 事件遷移採 declarative isolated-component planner，要求已存在通用替代事件與 exact field→sink 綁定；shared／unknown side effect／缺內容拒絕。MCP 備份後 CAS 刪普通節點，入口另以精確 ID 刪除前再次核對隔離性；前後保存的其他節點內容與接線精確不變。Saika BP 舊重複 action 分支已遷移且重跑冪等，typed API 尚保留、2.2b未封關，詳見 Blueprint generic event migration 進度檔。

2026-10-04：動畫 overlay 以共用 parser 讀 Lua priority／locomotion 綁定，bridge 只從實際 buff 選最高優先序、同分最小穩定 catalog ID；生成 native 名稱常數供 Unreal 共用 model 使用，無每幀 JSON 與角色名稱分支。衍生 sniper flag 不覆蓋 buff 清單；未知 hero 不偽裝 Saika。C ABI11 與 saved Blueprint 相容 API 保留，完整 2.2b 尚未封關，見 generic-animation-overlay 進度檔。

2026-10-04：buff lifecycle → ability 呈現事件來源改由 Lua `ue.buff_visual.ability_binding` 宣告，生成 manifest 與 bridge 共用型別，numeric buff 索引選擇 toggle／transform；未知引用與非法模式拒絕。事件投影不再比較角色／技能 ID。既有 C ABI typed 欄位與 saved Blueprint 仍保留相容；動畫 overlay 與 graph 遷移完成前不封關 2.2b，詳見 `docs/plans/2026-10-04-generic-buff-ability-binding-progress.md`。

英雄事件模板先隔離 legacy_hero_compat adapter：通用 C++ 生成迴圈不直接判斷角色／技能 ID，既有 Saika typed API 只在相容模組中產生。保持生成 bytes、manifest shape 與 reflected 名稱，避免破壞已保存 Blueprint；新英雄不得在該模組擴充。待資產 graph 與 bridge typed projection 明確遷移後，才能真正移除相容介面並封關 2.2b。實作與證據見 `docs/plans/2026-10-03-unreal-generic-event-adapter-progress.md`。

2026-10-03 的實作決定、驗證與剩餘問題補充於 `docs/plans/2026-10-03-unreal-moba-ipc-editor-progress.md`；Editor MCP 最小驗收詳見本 change 的 `evidence/editor-mcp-smoke-2026-10-03.md`。

通用 native skeletal 模板與 Lua 美術設定的實作／驗收補充於 `docs/plans/2026-10-03-unreal-native-visual-progress.md`；每次錯誤的原因、修正與防重犯規則集中於 `docs/plans/unreal-moba-error-register.md`。native CDO 只保存完整 soft object path，BeginPlay 載入 owned 配方資產；已有 Blueprint skeletal 呈現保持相容，不把新 native 基礎誤算成整項 6.1 已完成。

4.3 的安全記憶與重連補強詳見 `docs/plans/2026-10-03-unreal-remembered-ipc-progress.md`：最新安全 snapshot 包含凍結的 server-sanitized ghosts，與 live entities 分開；Forget 可清除相同 epoch 記憶，Reveal／verified rebase 清除舊記憶。尚未驗收 Unreal ghost 畫面與一次性 cue，因此 4.3 不勾選完成。

投射物 retained cue 已在實際 WorldBridge dispatch 以事件 tick／instance／source generation 去重，有界歷史拒絕過期 replay 並提供超量診斷；完整建置及兩輪 6 項 Editor 測試通過，詳見 `docs/plans/2026-10-03-unreal-projectile-cue-progress.md`。此為同一 WorldBridge 的投射物契約，不代表跨新 renderer instance 或所有技能 cue 已完成。

Unreal memory 呈現補強詳見 `docs/plans/2026-10-03-unreal-remembered-ghost-progress.md`：C ABI v2 增加獨立 frozen ghost 與完整視野標記，原生無碰撞 ISMC 不進入 live actor／輸入查找；控制 frame 不覆蓋視野，ghost-only reset 與忙碌 frame ring 可重試。局部生命周期只修改最後已核准的 safe snapshot。新增真實 TCP 重連、原生 Editor automation、PIE fixture 渲染與清除驗收；跨新 renderer instance cue 消費仍未完成。

雙陣營 IPC adapter 驗收詳見 `docs/plans/2026-10-03-unreal-two-team-ipc-progress.md`：C ABI v3 新增 explicit team_id（不推導自 player_id），Rust 真實 TCP 以 player 7/team 2 驗證握手；localhost network protocol v2 與 presentation IPC v3 不變。真實 server + 2 runtimes + 2 UE -game client 已各自顯示 filtered world、由 UE 發送移動並在 replica 與 UE frame 觀察位移。demo fog exact-center/team cache 避免靜止投影反覆 raycast，無動態 occluder 快取假設。這只封關 4.2，不代表正式 MOBA schema、LAN、完整 match 或 renderer reconnect 完成。

Renderer 恢復基準補強詳見 `docs/plans/2026-10-03-unreal-renderer-baseline-progress.md`：每條連線首個完整 snapshot 清除歷史一次性 effects/audio，但保留 persistent safe view；首個 snapshot 晚於 RuntimeReady 的 latest/critical 路徑亦使用同一規則。共享 snapshot 不被修改，後續 live cue 保留。Consumed 僅接受本連線已送出的 snapshot 界線，不能據此宣稱所有 Unreal cue 已有消費契約。4.3 保持未完成。建置流程增加 DLL SHA-256 stage gate，tests 必須在 stage 前跑，dual launcher 即使 skip UE build 也拒絕已知錯配。

UE 消費回報補強詳見 `docs/plans/2026-10-03-unreal-renderer-consumed-progress.md`：C ABI v3 結構不變，新增必要的 om_mark_frame_consumed export；WorldBridge ProcessFrame 完成後明確 mark，release 不等於消費。Rust 內部保存原始 IPC sequence 與不重用 connection ID，busy retry／coalescing 保留身分，control/lifecycle 不 ACK，舊連線 token 不污染新 socket。完整 build、兩輪 Editor tests、PIE 與實際 dual run 1791016176 通過，runtime 日誌確認兩隊 ACK。此為 CPU 呈現處理完成契約，不是 GPU fence，也未用 cursor 刪除尚未建立完整 ID 的 effects。

外部直接傷害提示補強詳見 `docs/plans/2026-10-03-unreal-sanitized-damage-cue-progress.md`：team projector 先對 safe facts 配置隊伍內唯一 ordinal，避免 hidden producers 同 local ordinal 撞號；DMG1 僅含事件 tick、可見 target／epoch 與傷害值，不推導隱藏來源／軌跡。runtime 映射到穩定 effect ID，重連之後所有 live frame 也剔除事件 tick 不晚於恢復基準的 DMG1。bridge 保留 batch／busy／已發佈但未處理的提示，到 mark 或視野失效為止；UE 通用 OnDamageCue 以獨立有界 history 派發原生 fallback。這不代表 Buff／持續效果、音效、可見來源技能、所有 cue 或正式對局戰鬥已封關。

runtime 節流補強詳見 `docs/plans/2026-10-03-unreal-damage-retention-progress.md`：每 applied step 先 drain DirectCombat，避免下一個 step 覆寫尚未發布的事件；有界 ledger 跨 latest overwrite 保留，真正成功送出後記 first snapshot sequence／connection，再以該世代有效 Consumed 清理。prepare／publish 不等於消費。Hide／Forget 精確匹配 target epoch，ResetView／view epoch 清空，reconnect baseline 不追播歷史；送出前剔除已退休 prepared envelope。最多 1024 筆、4096 ticks，超量／過期明確丟棄，不宣稱可靠事件永不丟失或 4.3 全部完成。

最末邊界修正：空 damage ledger 不掃描 entity 集合；renderer 長時間離線使 saved latest view 過舊時，歷史排除 floor 使用 max(saved view tick, applied high_tick)，不將舊 snapshot 的 state tick 偽造為最新，也不播放離線期間累積的歷史提示。晚到的首個完整 snapshot 同樣套用。

1. 建立可重跑的生成器檢查與完整建置流程，保留原 TD 啟動。
2. 為 Unreal 新增 IPC 呈現路徑並用雙隊視野測試驗證；待功能相同後隔離舊玩法 driver。
3. 將英雄與資產生成改為共用內容模型，分角色遷移，最後移除 Saika 特例。
4. 逐步交付 headless MOBA 規則、Bot、三路地圖與 Unreal UI，通過各階段驗收後作為預設 MOBA 啟動模式。

回退方式為保留既有 TD 模式與其啟動參數；新 MOBA 模式在版本或資產驗證失敗時不啟動對局。

## 實作修正（2026-09-25）

- 2026-10-04 公開地形導航前置：既有 grid planner 原只驗格點、movement只驗終點，薄牆／斜角可能穿透；改共用 omoba-sim i128 fixed-point swept-circle polygon，每條 edge、fallback與位移一致。沿既有公開BlockedRegions metadata，不披露隱藏動態障礙／MobaMatch。legacy float只在靜態資料邊界量化、超界fail closed；三seed60Hz共3600雙隊step每tickhash零repair與實際detour抵達通過。不是完整Lua地形／NPC導航／UE map或60FPS驗收，5.4保持未完成；詳見公開地形sweep進度與E128。

- 2026-10-04 rank0出生：英雄Lua可選moba_loadout四槽rank＋初始點數，預設保留舊四招rank1／SP0；shared validator供雙生成器使用，Rust輕量常數與runtime compiled agreement／hotreload gate同步。權威先驗證再套用，respawn不重發點；首次學習沿用正式UpgradeAbility，queued ScriptCast在MobaMatch禁止未學rank的legacy fallback。不以GameMode::Moba當新規則opt-in（舊Story也預設Moba）。60Hz權威／雙隊filtered／owner HUD回歸通過，尚無真實KCP／雙UE rank0出生驗收，詳見rank-zero進度與E122。

- 2026-10-04 Blueprint工具封關：full建置待MCP ready後驗generated recipe英雄與既有RustBP/UI相容WBP；缺失英雄BP以MCP與生成native parent建立，已有parent不同拒絕、不重寫graph。每個compile保留raw輸出，exit0／isError／ok／error_count／health／明確asset身分共同gate，失敗保留pending診斷且停止。真實隔離Actor→Character.Jump編譯失敗／原asset修復、Widget與generated parent建立／save、11個既有資產與Editor／PIE皆通過，3.2完成。UMG是工具能力與舊UI相容，不把空fixture當MOBA完整UI，仍以native Slate為預設。詳見Blueprint validation進度與E104。

- 2026-10-04 真實renderer重連以 opt-in 有界single_lane60Hz驗證；只正常退出並重啟team1 Unreal，server／雙runtime／對側Unreal不重啟。原與新stdout／UserDir隔離，fresh HUD取實際bridge tick rate、新Consumed、不同Point Move原ID ACK與runtime allocator新ID／raw目標、雙隊post-input不同checkpoint共同封關。run1791068322通過，完整4.3的所有一次性cue、LAN及frame-time效能仍未完成；詳見renderer reconnect60Hz進度與E103。Editor QUIT dispatch不等於退出，必須等待工具session exit0並確認PID不存在再啟動隔離測試。

- 商店按鈕驗收：非Shipping opt-in合成Slate pointer，在已建立／enabled／實際scroll範圍內的SButton使用真實cached geometry觸發既有OnClicked，不建測試專用交易入口、不fallback至API。layout未ready等待，dispatch失敗不換ID重送；exact三個回呼request與權威receipt／金錢／裝備／pending一一核對。60Hz release雙UE run1791055726通過，詳見商店按鈕60Hz進度檔；此非實體滑鼠／完整hit-test grid，不勾選完整6.2。

- 原生商店UI修正：native command bar不依賴legacy Blueprint root存在；共用280 layout-unit高度／viewport DPI命中區，UI防呆只核對capability／catalog／實際非空slot，不自行算價格或錢。Editor regression核對真實Slate tree與generic input建構；同Editor automation與PIE串行，不把API／fixture測試稱為真人滑鼠驗收。詳見 `docs/plans/2026-10-04-unreal-native-shop-ui-progress.md`，6.2仍未全項完成。

- 2026-10-04 Unreal 商店輸入：共用 C ABI 6／generic buy-sell event、frame-owned Lua compiled catalog、獨立 negotiated capability，native Slate 生買入與六格售出按鈕。只有正式 IPC runtime 支援買賣，legacy driver 拒絕；SHOP_SETTLED 才清 pending，transport uncertain 保留原 request。single_lane 預設 network60Hz，presentation30Hz shop驗收、UE60FPS上限；debug雙UE長跑未通過，不提高期限，改既有release production profile。完整結果／明確限制見 Unreal shop input 60Hz 進度檔，4.1／6.2 仍未全項完成。
  - release run1791053894 真實雙UE三筆交易／pending0／移動通過；出售後三方hash170／162與5457／5443原始snapshots通過、五程序清理。launcher最末JSON失敗以字串stage keys＋round-trip修正，獨立只讀保存證據verifier成功，不重新交易；60Hz串接封關不等於debug效能／物理滑鼠／硬即時deadline封關，仍不勾選完整4.1／6.2。

- 2026-10-04 商店正式入口：以獨立shop protocol1／完整rules hash／catalog agreement與bound secure player啟用single_lane買賣，Story／舊版不開。server query每秒8次、runtime pending64筆每秒4次只讀查原ID；renderer shop request同ID同payload只查結果，衝突與history1024滿額拒絕。write error視為不確定，不自動換ID再買；主／catch-up保留原tick並恢復原request terminal結果。正式60Hz雙隊真實買賣、各5次重送、terminal query、89個三方hash與逐snapshot經濟驗收通過，UE操作／跨runtime pending恢复未完成；依使用者本輪指示延後120Hz，详見60Hz商店網路進度檔。

- 2026-10-04 被動收入前提：Lua moba_economy 編譯成整數 shared rule，MobaMatch 只累積 playing Fixed64 時間、post-step 一次結算，死亡持有 persistent 金錢；暖場跨界精確裁切，暫停／終局凍結，金額飽和。客戶端使用 owner committed economy、不自行發錢。完整 catalog data hash 覆蓋收入且禁止單邊熱更新；headless 合法購買與真實雙 runtime IPC 金錢驗證通過，但不等於 KCP 買賣／UE income HUD 已驗收，secure gate 不開。詳見收入進度檔。

- 2026-10-04 商店結果恢復：journal 只從正式 authority receipt finalize，匹配原命令／玩家／effective tick，原 terminal 結果不可變。獨立 read-only KCP 查詢從已加入 V2 session 推導 owner，區分 unknown／pending／terminal／expired／admission-late；query 不重新提交交易。runtime 兩個 inbound pump 都驗證身分並只保留原 tick 的結果呈現，不改 Gold／Inventory／hash。真實 unknown query 與正常重連通過，compiled catalog kernel 的重送單元驗收不重扣；正式網路 terminal 買賣、自動 pending 恢復、query capability／節流與 UE 商店未完成，gate 保持關閉。详見 `docs/plans/2026-10-04-shop-result-recovery-progress.md`。

- 2026-10-04 runtime 正常重啟：server InputBuffer 保存每玩家 match-lifetime 的 admission ID high-water mark，TeamGameStart 以獨立 allocator version 1 回傳；runtime 必須相容且從 floor+1 分配，u32::MAX 耗盡拒絕，不繞回。界線不進 gameplay／script ABI、不是交易完成帳本；同玩家多活動 session 的既有拒絕保持。真實 KCP 正常關閉後重連、正式移動與重連後三方 hash 通過；crash／斷網／原始 terminal receipt replay 仍未驗收，商店 secure gate 仍關閉。詳見 `docs/plans/2026-10-04-runtime-input-resume-progress.md`。

- 2026-10-04 商店 transport 前提：Join／bootstrap 協商獨立 shop catalog version/hash，僅檢查配置相容而非交易 capability 或身分認證；新版 selective client 不接受缺少 agreement 的 server。InputBuffer 保存 player-scoped、match-lifetime 的有界 shop admission journal，完全相同原始命令只排一次；逐出 ID 留 watermark 防止舊交易重新執行，原始晚到拒絕亦不因重送而變成功。wire ID 不進 gameplay／script ABI。原始 terminal receipt 重送與 runtime allocator 重連恢復仍待補，不能以 DuplicateShop 冒充購買成功，因此 secure 買賣入口仍關閉；4.1／5.3 維持未完成。詳見 `docs/plans/2026-10-04-shop-admission-catalog-progress.md`。

- 2026-10-04 Unreal 經濟呈現：C ABI 升 5，Gold／六格／receipt 在 frame lease 中自有儲存，configured player ID 與型別／容量驗證後才投影。物品名稱／價格來自 compiled Lua catalog，由共用 WorldBridge／native Slate 一次轉接，不新增角色專屬 C++ 或 Blueprint graph。shop_available 只表示權威場景條件，secure 購買能力未開放，HUD 明確唯讀。真實雙隊／四技能／自己的 Gold 0 六格／shop gate 與三方 hash 通過；非空物品／receipt UI、交易 transport／收入／回城仍待後續，5.3／6.2 不勾選。詳見 `docs/plans/2026-10-04-unreal-economy-hud-progress.md`。

- 2026-10-04 商店回覆／IPC：玩法只輸出 tick-local ordered ShopSettlement；投影層精確核對原始命令與 acceptance correlation，發布 kind 23 真實結算結果，不把 wire ID 寫進 gameplay。inactive／dead 明確拒絕，不把 APPLIED 當交易成功。kind 24 OwnerEconomy 與 live hero 分離，死亡使用權威保存 Gold／Inventory，owner-team audience 後再由 IPC 限定 configured player。每 Applied tick 收集最近 64 筆結果，跨節流／snapshot 覆蓋／TCP renderer 重連保留，新 view epoch 清空；並非永久帳本或 server exactly-once。C ABI／UE 新欄位尚未接上，secure shop 不開放，5.3／6.2 維持未完成；完整證據與後續順序見 `docs/plans/2026-10-04-moba-shop-receipt-ipc-progress.md`。

- 2026-10-03 經濟結算：Gold／Inventory／ItemEffects 只進 owner-team baseline 與 81-byte typed CommittedEconomy（kind 21），Reveal／expected view／fresh bootstrap 都按隊伍過濾；不重演 buy／sell，不因 accepted input 直接扣第二次。可見 EquipmentStats（kind 22）含最終 HP／max HP／speed／armor／attack，但不帶敵方物品／餘額。Lua catalog 明確 numeric id，不依宣告順序。逐 tick 商店／合成／非法拒絕／出售／裝備死亡重生雙隊完整 hash 通過、沒有 ComponentRepair；Gold 的死亡 HUD 保留、交易 receipt／IPC／UE 及正式網路商店開放仍待驗收，5.3 維持未完成。

- 2026-10-03 商店權威核心：共用 Rust 原子 buy／sell，不改舊 JSON／TD 語意。cost 是成品總價，備齊全部材料後扣差價；重複材料佔不同格，六格滿仍可合成，整數 floor 50% 退款。以已認證 player_id 查自己 hero／基地，Playing／未暫停／存活／距基地 ≤300；拒絕不改錢／物品。重生 dirty ItemEffects 沿用 item_tick，不寫角色 C++。ItemBuy／ItemSell tag 17／18 正式權威 tick 與 ItemUse 共用 ordered queue；Lua passive catalog 由現有 build.rs 編譯，納入 data hash，物品變更需重建 peers、不允許 dev hot reload。secure V2 shop 仍拒絕，owner economy 投影／receipt／IPC／UE、擊殺助攻／回城待接，不把此核心算完整 5.3。

- 2026-10-03 完整 filtered 生命周期修正：單路所有可見 target 的 Damage 由 authority vitals／external effect 結算，避免重演不公開的 opponent target／projectile flight；filtered 不持有 MobaMatch，TD／非單路不改。owner 正式 input／script／cooldown 仍執行，禁止 owner CommittedCooldown override。下述歷史「owner clock 不 override」被完整 lifecycle 回歸推翻：可見 CommittedAttack 的 13-byte phase／clock／sequence 也結算 owner，沒有 target；NPC lifecycle pre-step 退休使其不能純本地預測。新增 CommittedProgression（kind 20、16-byte level／XP／next／SP），僅可見來源，範圍驗證後原子套用，不覆寫技能。1618 ticks 全對局＋15 終局 ticks 的双隊完整 canonical hash 回歸通過，沒有 ComponentRepair；這不等於五位置 Bot 或完整 UE/LAN 驗收。

- 2026-10-03 通用四槽輸入：原生 Q/W/E/R 使用 persistent owner HUD 的 generated ability ID／target type；控制器透過 reflected 通用函式跨單向 module 邊界，沒有英雄專屬 graph。opt-in API smoke 必須核對每槽獨立 input ID 的 queued／status=0 applied／正值 cooldown，並等最後施法之後的雙隊 pre/post 三方 hash；不冒充物理按鍵或完整終局。
- secure target 的 canonical 身分是 generation-packed u64；完成既有 epoch／visibility／team 安全驗證後才在 authority 內部取低 32-bit ECS index，不能將不合法的 client opaque ID 直接用作索引。可見敵方不重演私有輸入，需 opt-in typed CommittedAttack（13-byte clock／sequence／phase）補充 post-gameplay 普攻結算；隱藏 actor 不發布，owner 自身 clock 不被 override，不用 ComponentRepair 掩蓋錯誤。施法後 120 tick 每 tick 雙隊 canonical hash 回歸已加入。
- 追趕呈現先決定 Latest／Critical／不準備，不為明知丟棄的 intermediate latest snapshot 重算大型霧區；lifecycle 與 applied-input state 的既有 Critical FIFO 不變，damage 仍每 step 留存。修正後真實雙 UE 四槽兩次全部通過，保留失敗 run，不提高 timeout。完整流程／限制見 `docs/plans/2026-10-03-unreal-four-ability-progress.md`。
- 2026-10-03 owner HUD：維持唯一權威 World 與單一 filtered runtime，以 optional typed persistent HUD 提供配置玩家身分、HP、四槽冷卻、phase／時間／重生／勝負。C ABI 明確升至 4 並同步 staging；未實作 current mana 前標示 unsupported，不用 Unreal 自算或虛構滿魔。只修改共用框架 adapter／原生 UI，不新增角色 C++ 或 Blueprint graph。
- 四技能 filtered 測試發現對手未取得已結算 HP／cooldown；opt-in MobaMatch 使用 typed numeric committed facts，只為可見 actor 發布，不分享對手 input，不傳完整世界，不以 ComponentRepair 掩蓋；owner cooldown 仍走 accepted input 的 deterministic gameplay。這是 outcome projection，不是第二份敵方腳本模擬。完整 UE 技能到終局與混版 LAN 尚待验收。
- smoke 自動移動必須以明確 command-line flag 啟用；正常互動啟動不替玩家提交測試命令。所有錯誤與重跑證據保留在 error register／single-lane HUD progress，首次啟動逾時未確認根因，不能宣稱根治。

- 共用內容模型先固定英雄／技能身分與 tombstone 驗證，再對完整 Lua 模板求值結果取得跨生成器 canonical hash；完整數值與視覺型別 schema 延後到 2.1c。理由是兩個既有 Lua parser 的數值型別和 UE 擴充欄位不同，直接合併容易改變既有內容語意。
- 先為全部英雄及其技能加上通用 Blueprint 可讀 metadata，暫留 Saika 專用事件以維持現有資產相容；待通用 cue/事件 API 覆蓋後再移除。此過渡已由 codegen 測試、OmGame 編譯與 PIE smoke 驗證。
- Editor 停止與 bridge staging 僅針對本專案判定。現有 batch 原本以程序名稱拒絕任何 Unreal Editor，會阻擋同機其他專案；由 `om_restart` 檢查本專案程序後向 batch 傳遞已檢查狀態，直接獨立執行 batch 時仍保留保守的全域檢查。
- Lua 英雄定義記錄 `rust_module`，建置時由固定 Lua 5.4 工具生成 Rust FFI 註冊語句；原有 handler 仍負責特殊技能邏輯，宣告式 effect 執行器獨立開發。這避免把任意 Lua 原始碼轉譯成 Rust 的不穩定路徑。
- Unreal 資產配方先記錄來源與穩定目標路徑，不在產生階段宣稱已匯入。MCP 首次動畫匯入雖建立正確 Skeleton 關聯，卻額外建立 SkeletalMesh/PhysicsAsset；已修正工具匯入類型判定，但 Editor 重啟後 HTTP MCP 連接埠雖開卻無回應，故修正仍待 Editor 端復驗。其他不依賴 MCP 的工作繼續進行。
- `BpGeneratorUltimate` 在此工作樹是未初始化 git submodule，直接改其 C++ 不會進入專案版本；建立 `scripts/ensure_bpgu_animation_import.lua` 以可重跑、未知版本即失敗的方式在 Unreal 建置前確保修正存在，保留插件內當前實際修補供本機編譯。
- 2.1c 已將技能等級、施法欄位、extras、英雄基礎數值、成長數值與模型／動畫來源的共用型別放到 `omoba-content-model`；Rust template-id 與 Unreal codegen 使用相同核心欄位型別。權威英雄整數數值在 Unreal 生成 C++ 邊界轉為顯示用浮點數；Unreal 專用 `ue` metadata 與後端專用攻擊時序仍由各自的薄擴充結構持有。動畫 JSON 型別化後補齊預設欄位，使生成輸出 hash 改變，但原始 Lua canonical hash 未變；已重新生成並編譯 OmGame。現有 Saika Blueprint 參照專屬事件，2.2b 遷移前先保留相容入口。
- 宣告式技能首批只支援即時的指定敵人傷害與自身治療；Lua `effects` 以 `amount_key` 引用每級 `extras`，建置時生成通用 Rust handler 的註冊語句。位移、持續區域、召喚、被動與切換技能仍需 Rust handler，避免假裝現有 ABI 已能安全表達。`faction_of` 在目前權威 adapter 未實作，因此通用傷害在施放前透過權威 `query_enemies_in_range` 驗證指定目標為敵人；整個效果序列先驗證再套用，拒絕部分生效。四技能測試英雄已通過無畫面效果測試，但尚未在完整 headless 對局的輸入／結算路徑施放，故 2.3b 暫不勾選。
- 新英雄 `training_luminary` 不宣告 `rust_module`，四技能使用上述通用執行器。`ue.native_only = true` 明確要求 Unreal 使用生成的 native 類別而不產生空 Blueprint graph；現有 Saika、Date 的 Blueprint 路徑與驗證保持不變。生成器、Rust DLL、bridge 與 OmGame 均通過，2.4b 完成。無模型／貼圖時仍需在 6.1 驗證畫面 fallback。
- 2.3b 的 headless 驗收使用實際 `StateInitializer` 權威 ECS、Lua 生成的 manifest、`ScriptEvent::SkillCast`、`run_script_dispatch` 與 `process_outcomes`；四招的傷害／治療皆以 `CProperty.hp` 結算驗證，且同隊目標在清除冷卻後確實被 effect preflight 拒絕。這涵蓋伺服器施放與結算，不宣稱已完成網路玩家輸入驗證、施法距離或完整對局生命週期；那些仍在 4.x／5.x。
- 4.1 的第一段 IPC 硬化將 renderer 封包版本升至 3，要求第一個封包是附帶 player/team ID 的 `RendererReady`；runtime 在發出任何最新安全投影前核對身分，錯隊或錯玩家立即中斷。這可防止本機多實例接錯端點，但 loopback 身分是自我宣告，不能當作對惡意本機程序的認證。既有 `RendererInput`、filtered snapshot 與輸入結果尚需 MOBA 專用 schema 和端到端視野測試，故 4.1 暫不勾選。
- 2026-10-04 回城接軌：獨立Recall protocol1／fullruleshash，不借shop能力；僅bound selective SingleLane player可用。filtered清舊命令依owner-team PreStep active事實，不依accepted Recall猜測成立；PostStep remaining在傷害／死亡／勝負後結算，完成只依權威基地傳送而非ACK。通用B鍵／HUD經C ABI7提供，真實60Hz雙runtime及雙UE B綁定delegate對局通過（非OS鍵盤注入）。UE1791073998原始Recall2→Move3取消→Recall4完成，原生HUD四張PNG及protobuf倒數／三方66 PASS／5程序cleanup另驗；詳見unreal-recall-key-60hz-progress與E108。4.1／5.3／6.2仍不勾選全項。
- 2026-10-04 owner score 接軌：完整 player ID 的 team-safe K／D／A metric僅在active PostStep結算後發布一次；inactive PreStep維持持久值，filtered不持有MobaMatch或assist ledger。optional IPC缺值不當零，bridge核對owner，C ABI8拒絕7；Unreal共用int64欄位完整容納u32、死亡仍顯示，不新增角色graph。真實三runtime3324筆capture對照來源frame精確通過；snapshot tick是apply後world tick（frame+1），verifier明確減1，不近似取樣。此owner HUD並非公開十人計分板或UE多人實戰驗收。
