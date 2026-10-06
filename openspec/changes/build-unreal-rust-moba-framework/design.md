## Context

2026-10-06 Bot追擊導航：共用普通攻擊決策保持原角色排序與focus，最多8候選、out-of-range查本人半徑／公開完整路徑，失敗僅決策局部排除ID、不移除披露警戒；可改選其他候選，全失敗去重Hold，不新增Jungle單挑政策。射程內不加LOS，技能政策不受導航限制。core2＋正式60Hz base4通過，見bot-combat-navigation-progress／E297，完整5.5留最後。

2026-10-06 護送導航准入：Support的最終Escort只用current披露Carry點查一次共用完整路徑，失敗正式Hold去重，地形恢復重選，不猜隊友未披露位置。ApproachStructure保留候選階段已完成的bounded查詢，不在提交分支再查第9次；相鄰協防／護送與新正式60Hz共3通過，見bot-escort-navigation-progress／E296，完整5.5不勾。

2026-10-06 兵線Bot導航：兵線Advance與Jungle巡邏共用有界候選選擇器，跳過抵達點、每think最多8個完整路徑查詢；兵線preferred後只前進不環狀回頭，Jungle維持公開ring。全失敗正式Hold且去重，恢復地形重新選、不永久黑名單；戰鬥／塔前等待／護送保持既有優先。局部結果見bot-lane-navigation-progress／E295，完整5.5仍留最後。

2026-10-06 五位置撤退導航：sustain Retreat在共用有界導航與本人碰撞半徑下檢查公開基地路徑；未找到完整路徑仍可用合法自保技能，否則正式HoldPosition去重取消舊攻擊，地形恢復後重新走MoveTo，不永久黑名單、不讀hidden敵方資料，root既有等待保持。局部60Hz結果見bot-sustain-navigation-progress／E294，完整5.5仍留最後。

2026-10-06 frame文字准入：OmFrameStringRefFits驗寬整數offset＋len與MAX_int32 converter容量，len0保留absent offset-unused；共用frame gate覆蓋全部文字ref／nested projection／inventory與FX text view storage／length，非法不apply／ACK，不靠空字串fallback。讀取器補縮窄防禦，不宣稱UTF-8內容驗證。UE限定9actions、Lua syntax成功，新native逐欄位矩陣僅編譯未執行，未PIE／stage，完整6.4／21/31保持；見frame-text-contract-progress／E293。

2026-10-06 frame幾何range統一：OmFrameRangeFits以u64 start＋count查capacity，route與polygon都在共用frame gate整批准入，避免route局部跳過後仍ACK／部分更新；合法尾端空range允許、不設任意point cap，模型語意保持。UE限定9actions、Lua syntax／whitespace成功，新native矩陣僅編譯未執行，未PIE／stage，完整6.1／6.4與21/31保持；見frame-geometry-ranges-progress／E292。

2026-10-06 霧快取收斂：完整vision circles／tree／polygon spans與points／scale逐欄位精確陣列鍵，不依frame sequence或數量，不讀padding／可碰撞hash；finite與int32 origin邊界檢查，共用frame gate以u64拒絕polygon span越界。無viewer／非法／reset清鍵、重建前隱藏舊plane，成功顯示才commit。UE限定9actions、Lua syntax通過，native鍵矩陣僅編譯未執行、未PIE／stage／效能驗收，完整6.1與21/31保持；見fog-geometry-key-progress／E291。

2026-10-06 斷線HUD一致：InvalidateOwnedHud同源清輸入與Hero／四技能／六物品／Economy UI，dirty latch／publishing guard防洪泛與重入；Tick斷線suppression擋retained owned HUD，但不停止world／cue／frame消費，Stop強制同源清。fixture首輪C3668因誤覆寫ImplementableEvent，改dynamic delegates後限定5actions成功，native斷言未執行、不PIE／stage，21/31保持。見disconnected-hud-progress／E290，完整6.2／6.4留最後。

2026-10-06 owned技能准入：共用finite／非負／remaining<=total且恰零的冷卻檢查，施法／升級／回城共同started＋Connected gate，失敗／Tick斷線前後／Stop清四槽baseline與ready；只有完整owner HUD、Playing／finite正HP建ready。嘗試先清舊InputId，回城明確configured owner，Rust仍最終驗證。首輪C4456回城Bridge遮蔽修正沿用已有變數；native矩陣僅編譯，完整6.2／6.4及21/31保持。見ability-input-policy-progress／E289。

2026-10-06 Unreal compiled-only接線收斂：移除runtime Lua watcher／遞迴scanner／debounce與私有狀態、reload config flag；舊設定反射名稱保留deprecated且忽略，兩個公開RequestRuntimeLuaReload入口回Disabled／生成編譯指引而不呼叫DLL或清呈現cache。UE限定14actions、Lua syntax与whitespace通過，native政策案例僅編譯未執行；未PIE／stage，版本hash與21/31保持。見unreal-compiled-reload-policy-progress／E288，6.0既有完成項補強。

2026-10-06 runtime呈現重設：ResetRuntimePresentation同源供Stop／EndPlay／新handle建立前，清owned ability／recall ready、四槽ID／cooldown與空HUD、選塔、runtime／HUD快取與tree／polygon／fog幾何cache，沿既有actor／route／ghost／terrain清理。同handle重複Start不清frame消費／物品baseline，不重啟後端。UE限定8actions、Lua syntax通過；native停止／重複停止／同baseline恢复案例僅編譯未執行，不重試E285 Cmd、未PIE／stage，完整6.4不勾；見runtime-presentation-reset-progress／E287。

2026-10-06 frame消費分離：FOmFrameConsumption共用shape／lease sequence gate，apply成功即標已套用，ACK失敗同frame只重試ACK、不重播；成功duplicate不重ACK，新frame可進行、Stop重設，stats保持已套用游標。正式Tick總release、不持有pending lease／無界queue。UE限定8actions與Lua syntax成功，新native矩陣僅編譯未執行，遵守E285不重試Cmd，完整6.4不勾；見unreal-frame-consumption-progress／E286。

2026-10-06 frame准入：新增共用size／ABI16／snapshot0或1／全部21個count-pointer與nested fog storage gate；ProcessFrame任何呈現變更前拒絕並回false，Tick成功才ACK／推進且總會release，錯header不先讀sequence。保留合法畫面，下個合法frame恢復，診斷2秒節流；不設stress上限，不把nonnull等同allocation證明。fixtures填正式header，UE限定8actions成功；scoped Cmd啟動exit1且無log，native斷言未執行，不重試或宣稱通過。見unreal-frame-contract-progress／E285，完整6.4留最後。

2026-10-06 Unreal compiled catalog准入：修空surface放行／hash差異warning／不相容仍處理frame，共用header／generation／有界16hex／精確compiled hash與surface檢查；Start拒絕並停止本地bridge，Tick錯配release後Stop且不取frame／ACK。相容預設false，catalog／frame游標重設，Acquire暫時失敗跳tick，不重啟後端。UE限定8actions成功、native矩陣僅編譯未執行，Lua runner語法確認；完整6.4留最後，見unreal-compiled-catalog-gate-progress／E284。

2026-10-06 公開介面簽章更正：舊name-list-only漏metadata／event宣告，改surface-v2簽全部8個真正生成header、穩定檔名排序與length framing，保留kind／ID／class；不簽數值initializer／角色summary。generator2／bridge原子拒絕1、17檔check與UE限定3actions成功，presentation=d5553bbb31c459a4／surface=1e2dbe976a9ee6d7，identity／data與ABI16／wire6／IPC5保持。最後一致部署，不開runtime Lua、不勾完整2.2b；見generated-public-surface-signature-progress／E283。

2026-10-06 技能投影C ABI收斂：既有通用Buff／area同一projection正式改OmAbilityProjection／event.projection，同步Rust、cbindgen與Unreal dispatch，欄位順序及事件／披露／換算語意保持。C ABI16 strict拒絕舊版，wire6／IPC5與hash不改，最後統一重建部署；13局部測試與UE限定10actions成功，header smoke過期11已改16但未單獨執行。其他typed與saved引用仍待，见generic-ability-projection-abi-progress／E282，不勾完整2.2b。

2026-10-06 英雄metadata同源：FOmGeneratedHeroMetadata擴充作者完整數值／render來源／旋轉與動畫清單，舊GetSaikaMagoichiMetadata只轉接23欄位；BaseHp與AbilitySlots分別讀BaseHealth及AbilityIds。不把native_visual換算或渲染fallback寫回metadata，省略render scale保持共用0。15局部測試／17生成check／UE限定9actions成功，歷史summary與其他typed引用仍待；見generic-hero-metadata-adapter-progress／E281，完整2.2b不勾。

2026-10-06 技能metadata同源：通用FOmGeneratedAbilityMetadata追加建置生成ExtrasJson；GetSaikaAbilityMetadata僅呼叫通用查詢、轉接FName／Icon與等長Levels，不再生成技能ID或數值分支。保留reflected名稱與歷史summary，英雄完整typed metadata及其他payload／資產引用仍待；不恢復legacy自動派發。局部14測試、17檔生成check與UE限定9actions成功，內容hash未變，完整2.2b不勾；見generic-ability-metadata-adapter-progress／E280。

2026-10-06 Jungle巡邏導航：只在最終Advance從preferred起沿公開camp ring選共用導航第一完整可達點，每selector8次、抵達50內跳過；無結果持續Hold且去重、地形恢復重考慮，定身不查路／combat與siege優先。兵線waypoint順序不變，不建黑名單／計時器／傳送。core3與正式60Hz1最終通過，首輪fixture錯把acceptance當同tick movement已修；見jungle-patrol-navigation-progress／E279，完整5.4／5.5留最後。

2026-10-06 Bot位移地形准入：ApproachEnemyPoint候選用共用path_hits_regions查本人半徑與公開BlockedRegions，缺值沿script adapter預設30而非普通命令20；受阻候選跳過，全受阻繼續作者政策，普通EnemyPoint不套遮擋。唯讀、不黑名單、不讀hidden dynamic，正式handler仍最終驗證。core2／base4（正式60Hz3）通過，完整5.5未勾選，見bot-relocation-terrain-progress／E278。

2026-10-06 Jungle旅行導航：current合法建築候選依距離／canonical ID排序，最多8個沿共用static_next_waypoint查本人CollisionRadius／公开BlockedRegions，第一完整路徑才ApproachStructure；無結果回原camp巡邏。局部focus／Hold優先且定身不查路，無私有cache／永久黑名單／傳送，地形恢復重新考慮。core3與正式60Hz3通過，不宣稱全域可達／完整5.4／5.5，見jungle-siege-navigation-progress／E277。

2026-10-06 Jungle攻城旅行：保留協防／附近野怪／局部攻城與無兵近塔Hold，之後從current披露選合法建築，塔需當前活同隊兵650內，最近距離／canonical ID稳定排序。ApproachStructure沿共用MoveTo不自動搶兵，抵達550再AttackTarget，相同旅行去重；無機會回公開營地。不讀hidden cache或私有unlock／camp，不宣稱全途安全或全域輪轉。core2與正式60Hz1通過，5.5仍待最後；見jungle-siege-travel-progress／E276。

2026-10-06 Jungle局部攻城：保留協防／野怪focus優先，無focus時不再直接巡邏，而是共用塔wave gate／建築披露／距離／Hold與AttackTarget分支；不新增solo hero或lane creep目標政策。缺當前可攻擊建築仍回camp巡邏，不查hidden state。core3與正式60Hz1確認通過，非全域lane輪轉或完整終局；5.5仍待最後，見jungle-siege-progress／E275。

2026-10-06 共用導航完整路徑契約：有界BFS只在到goal或合法精確terminal edge時重建第一步，移除nearest reachable cell當partial成功的fallback，occupied target統一前置拒絕。None沿既有英雄佇列advance／NPC停留，不代表全域不可達、不做teleport或額外Botcache。四項局部確認與完整5.4界線見navigation-complete-path-progress／E274；最後Rust各執行元件需一致重建。

2026-10-06 共用導航同格繞行：不另設Bot卡住計時器；修static_next_waypoint之same-cell early reject，clear直線快路徑保持，受阻但合法target作獨立terminal edge接既有有界BFS，所有邊維持swept檢查。MoveTo／AttackMove於60Hz實際繞行抵達、逐步無穿牆與重播一致；三項局部確認通過，不勾完整5.4。導航執行語意改變，最後部署需Rust權威／replica／bridge一致重建，不能混用舊binary。見same-cell-navigation-progress／E273。

2026-10-06 Bot物品命令交接：ItemUse替換既有命令，故將E267的回魔先行改為閉集效果優先：護盾／減傷／逃生→回魔→下一擊準備，再以槽位同類穩定排序。例行回魔與增傷在前搖延後，緊急防禦可打斷；非前搖回魔正常恢復，權威／玩家命令取消規則不變。core2與正式60Hz1指定確認通過，完整5.5待最後，見bot-item-order-priority-progress／E272。

2026-10-06 Carry 已準備增傷：UnitStats::normal_attack_physical 共用 outgoing modifier 後再加一次性增傷的正常packet公式，Bot與實際launch同源；觀測不消耗，不讀其他owner或hidden target，保持accuracy gate與披露incoming倍率。正式60Hz短測證明一般輸入至真實launch才消耗，完整5.5仍留最後。詳見carry-armed-damage-progress／E271。

2026-10-06 driver 模式邊界：啟動前單次解析 Presentation／LocalTd／NetworkTd，分支與初始速率共用結果；內部 IPC 入口也拒絕零玩家／隊伍，IPC 無本機 story／DLL 仍合法且不建立模擬執行緒。SinglePlayer 保持拓樸語意，不強制與 IPC 互斥，不依 story 名稱猜模式。局部確認見driver-mode-boundary-progress／E270，完整單機／LAN 的4.4仍待最後驗收。

2026-10-06 主動物品狀態邊界：host-local ItemTimedModifier閉集使BuffStore writer／reader共用kind→family／stat／方向／上界，Bot不解析私有source prefix，也不從混合後淨stat猜效果消失。numeric reader與tick移除同原聚合，保持刷新與strongest-family，不新增公開payload、快取或計時器。局部結果見typed-item-modifier-progress／E269，不勾完整5.5。

2026-10-06 原生守住命令：H與Shift+H chord各自傳false／true到同一命令建構與提交入口，不使用實體Shift取樣。事件重設／非法owner清零／configured ID／HUD與Connected gate，NoTarget正式HoldPosition沿既有Rust權威管線。編譯與局部確認界線見native-hold-chord-progress／E268；不是新gameplay或已執行Editor驗收。

2026-10-06 Bot主動物品決定：BotItemBuild active_use為可選閉合型別政策，0資源門檻停用，半徑／千分比有界。只讀本人資源與current披露威脅，由共用kind選NoTarget ItemUse，回魔／護盾／減傷／衝刺／增傷及槽位穩定排序；既有效果不重複，定身不衝刺、前搖不準備增傷，購買／回城與控制gate保留。Lua開局配方配置，正式Rust執行，不為hero或道具寫Unreal分支。當前結果與E267見bot-active-item-progress，不勾完整5.5。

2026-10-06 正式訓練主動商品：moba_items來源追加穩定ID5–9，Shield100／3秒、Sprint60／3秒、RestoreMana100、DamageReduce25%／2秒、HeadshotNext60；價格500／500／400／600／600，CD12／12／12／15／10秒。原四被動內容保持，新初版平衡可由Lua調整；不綁hero／C++專用分支。正式生成registry與60Hz買用1測試／5情境通過，回魔核對物品精確權威事件而非漏算自然回復的池總量，錯誤記E265。九件C++metadata生成與check／相關UE模組編譯成功，未PIE或統一stage。data=2df5b6b1e02d95d7、presentation=3634f807282b0515，identity保持；最後建置需同步所有元件，ABI／wire／IPC不變。護盾安全投影與21/31未完成大項保持，見active-shop-content-progress。

2026-10-06 原生六格輸入：1–6經OmRuntime唯一binding與反射owned入口送到既有Rust權威管線，五已支援主動效果均NoTarget。純builder核對完整六格／owner／metadata／slot身分／ready／有限零冷卻，不要求商店能力，拒絕清PlayerId=0避免legacy預設1；connected與runtime就緒另由WorldBridge核對。完整owner baseline僅當前自己的快照，Start／Stop／缺owner／斷線無frame清cache，control-only保留。queued不是效果成功，不預扣CD。相關模組編譯14+3 actions成功，C++新增斷言未執行，PIE／新正式主動商品／護盾安全投影與最後部署驗收未完成，hash／協定與21/31不變，見item-native-input-progress／E264。

2026-10-06 主動物品原生 metadata：Rust／UE 生成共用 content-model::moba_items 作者型別與驗證；生成 OmItemCatalog 查詢穩定 ID／效果／量化參數／總冷卻。原生 widget 結合靜態查詢與既有 owner-local 六格剩餘冷卻，不新增 ABI／runtime Lua 或英雄分支，明確 passive／ready／cooldown／unavailable，未知與非法資料不標 ready；完整缺 owner／StopRuntime 清空，control-only保留。生成器2／原作者2與17檔check通過，相關 UE 模組編譯成功，新增 HUD 斷言未執行。盾量須另做安全 owner 投影，不能讀任意 Buff payload；新商品與原生使用操作尚未完成。presentation=e8bdc0929625fdd2，identity／data不變，未重建部署 bridge 或跑 PIE／全驗收，21/31保持，見 item-native-metadata-progress／E263。

2026-10-05 主動物品作者契約：現有 moba_items 支援可選 active kind 閉集與 cooldown，deny_unknown_fields 與核心同界有限值生成前拒絕；生成 Fixed64 MobaItemActiveConst，generated_moba 映射至既有 f32 native ItemConfig 執行器，不建另一份 JSON／runtime Lua。被動 active=None／CD0省略欄位序列化，既有四商品與資料保持，不在本輪新增平衡內容。作者模型局部2/2通過，opt-in runtime feature僅供解析器測試，正式編譯仍 compiled-content-only。UE C++ active metadata／HUD與部署全驗收未完成，21/31保持，見 item-active-authoring-progress／E262。

2026-10-05 舊物品入口收斂：非 MOBA resource_management::use_item 移除五種效果與冷卻副本，精確唯一名字／非零唯一 owner／存活英雄、六格無號槽位預檢後委派 core；未知名字不退回第一英雄，大整數不截斷，失敗不回 completed。名字不是授權，正式 MobaMatch 在公開與私有入口均拒絕舊文字輸入。五效果與冷卻、非法槽位／模糊身分、正式模式雙入口 3/3 通過。其他旧技能／買賣 fallback 未改；generated active 作者與 UE 呈現、最後全驗收待辦，21/31 保持，見 legacy-item-adapter-progress／E261。

2026-10-05 真正護盾：Shield不再以瞬回HP替代，有限正量與1/1024–60秒先預檢，BuffStore共用shield grant／remaining／absorb單一最強餘量、refresh不累加。一般非TD-layer Damage在modifier結算後吸收，ScriptDirectDamage亦吸收，溢出才扣HP；完全吸收有效hit仍interrupt recall，擊殺／HPdamage attribution依實際扣HP。正式60Hz買用／減傷順序／direct／到期／回城中斷1、mixed買用賣遷移1及拒絕11情境1通過；零duration舊fixture改1秒、不再保留假護盾回血。發現omb resource_management舊use_item仍重複MVP，後續需收斂；不宣稱所有入口已完成。无Lua／hash／協定／UE／部署，完整active作者與21/31保持，見item-shield-progress／E260。

2026-10-05 一次性普攻增傷：HeadshotNext改共用BuffStore arm／peek／consume API，私有固定pending key不綁item／hero，有限正bonus1/1024–1000000、TAttack／store先預檢，重複arm採最強。正常handle_projectile解完source／target／attack資料後才消耗，增傷在命中率之前加到physical packet，miss消耗且damage為零，visual副本零傷害；失敗launch不消耗，不在input／windup／技能投射物或cue退役消耗。復用remove_all死亡清除，不永久改base atk、不公開未知Buff payload。正式60Hz買用1、核心launch1與相鄰拒絕1通過；首輪core漏kcp建置錯誤已修正記E259。真正護盾／generated active authoring／完整UE操作待後續，無Lua／hash／協定／部署，21/31保持，見item-next-attack-bonus-progress。

2026-10-05 物品限時減傷：DamageReduce從E257未支援改共用BuffStore之負DamageTakenBonus，item ID穩定source／item_damage_reduce最強family，不永久改防禦或增加獨立傷害流程。比例有限1/1024–1、duration有限1/1024–60秒與原CD／stats／store先預檢；ratio用fixed raw round，所有packet依既有sum-before-multiply結算，含pure，不冒稱護甲。60Hz正式買用兩item，160混合傷害80→120→160、冷卻拒絕不刷新／合法刷新不疊加、11非法情境與相鄰拒絕通過。無Lua內容／UE／hash／協定／部署，真正護盾／HeadshotNext／generated active authoring仍待後續，21/31保持，見item-damage-reduction-progress／E258。

2026-10-05 物品回魔：RestoreMana從E256未支援改為共用managed ManaPool執行。有限正量1/1024–1000000用既有checked fixed換算；先clone pool、在正式mana模式用當前hero base＋BuffStore checked容量再restore，缺pool／非法容量／必要event queue拒絕，整份預檢後提交pool／實際ManaGained與CD／命令。滿魔仍可合法使用但不發布零gain，不替legacy hero創建pool。60Hz正式買用owner／含Buff容量／實際gain，新拒絕8情境與相鄰2案例通過；正式生成物品仍被動、DamageReduce／HeadshotNext仍拒絕，不新增Lua／UE／hash／協定／部署，完整active authoring與21/31保持，見item-mana-restore-progress／E257。

2026-10-05 物品主動效果准入：legacy SprintBuff不再永久加CProperty.msd，改item ID穩定source與item_sprint aggregation family的限時MoveSpeedBonusBuff，復用BuffStore刷新／到期／死亡清除；冷卻與有限有界bonus／duration先驗證。原來只log的RestoreMana／DamageReduce／HeadshotNext明確拒絕，不扣CD／清命令；Shield瞬回HP的明示舊相容保留，不冒稱真正護盾。正式compiled catalog仍被動、不新增runtime Lua或active作者schema；60Hz衝刺與拒絕案例、舊混合買用賣通過，不部署／全驗收。完整主動物品作者模型與效果仍未完成，21/31保持，詳見item-active-admission-progress／E256。

2026-10-05 原生攻擊移動：通用 controller 的 A 直接提交游標位置 AttackMove，Shift+A 保留 queued；不新增 A＋左鍵模式或角色 graph。反射沿 OmRuntime→OmGenerated 單向依賴，WorldBridge 從 configured owner 取得玩家 ID，HUD／選角攔截、視窗外／非有限游標與無效投影不送輸入；復用現有 Rust 權威准入與佇列，無第二份玩法 World。限定模組13 actions成功，新增原生 binding／拒絕案例只編譯未執行Editor／按鍵，不改Lua／hash／協定，完整6.2與21/31保持，見unreal-attack-move-key-progress／E255。

2026-10-05 自由／自己英雄鏡頭：WorldBridge首次fit後不再每snapshot覆寫自由位置，原生camera保存owner安全focus、Space暫時跟隨與Y持續鎖定，顯式追蹤時不edge pan。僅configured owner的存活Hero可供目標；缺席／死亡／非法位置清target不借其他英雄、不跳原點，entity key精確退休舊生命，Stop／ReleaseAll清除且新場次可首次fit。锁定偏好可在自己重新披露後恢復，不猜hidden actor或新增gameplayWorld。限定10＋最終9actions成功，新斷言只編譯未執行Editor／按鍵，21/31與完整6.2保持，見owned-hero-camera-progress／E254。

2026-10-05 Bot受阻續航：role_bot_inputs共用owner rooted禁止新Retreat／Advance／非原地Escort，不清現命令；只有Recall／Retreat被控制阻擋才進本次decision-local recovery_wait，正常rank／成本／CD與作者順序選恢復policy，暫緩進攻，無恢復可用不轉新戰鬥／旅行。原學習／合法旅行與基地Hold優先保留，root不全面禁止普通施法／普攻，stun沿原全等待。到期讀當前committed perception恢復，不新增跨tick游標或英雄分支。新正式60Hz2與相鄰3成功，不改Lua／hash／協定／UE，不部署／全驗收，完整5.5與21/31保持，見rooted-bot-recovery-progress／E253。

2026-10-05 cue退役准入：CueRetention把pending與admitted ID歷史分離，同view epoch的有效ACK／Hide／不可見退役不重新開放原ID。歷史最多1024且淘汰最小ID推進retired floor，pending不因歷史淘汰被刪；排序前綴清過期避免新增全量每step掃描。重連baseline保存最高tick阻止晚到歷史，新epoch／Reset開新domain，原4096tick窗口與同connection送出／ACK保持。這是可丟棄過舊呈現的有界政策，不承諾永久可靠事件。retention6/6與真實loopback TCP1/1成功，不改Lua／hash／協定／UE，不部署／全驗收，完整4.3與21/31保持，見retired-cue-admission-progress／E252。

2026-10-05 主動位移控制整合：ParallelWorldAdapter.advance_with_collision在標準rooted（含stun）時回傳當前位置，overlay優先；既有完整效果預檢拒絕無法抵達目標的位移，不扣費／啟冷卻。不封鎖直接set_pos權威／強制重新定位入口；Bot僅跳ApproachEnemyPoint，普通傷害／恢復不受root全面禁止。正式60Hz新1／adapter新1／相鄰位移1通過，不改內容hash／協定／UE，不部署或全驗收，21/31保持，見rooted-voluntary-relocation-progress／E251。

2026-10-05 回城控制整合：MobaMatch以單一recall_control_blocked判斷標準rooted／silenced（含stun）；begin禁止開始、Outcome結算後在deadline判斷前取消純控制channel，不靠虛構傷害／位置改變。Bot續航及經濟回城同規則；解除需新普通Recall，不重啟舊channel或恢復deadline，不改root允許技能／silence允許普攻的語意。新正式60Hz2項與相鄰2項成功，不改內容hash／協定／UE，不部署或全驗收，21/31保持，見recall-control-interruption-progress／E250。

2026-10-05 通用控制效果：ControlEffectKind為stun／root／silence閉集，Lua control_enemy與相容stun_enemy共用resolved plan／標準BuffId sink、每級1/1024–60秒與enemy unit／range全plan預檢。定身只阻移動、沉默只阻施法、不凍結攻擊時鐘，max TTL沿既有控制契約，沒有獨立來源驅散／韌性免疫。先鋒／遊俠Lua資料附加效果與native-only root／silence公開Buff，不增加角色handler／C++／BP graph；生成階段若省native_only會帶默认BP路徑，必須明確宣告並核對manifest。局部Lua／Rust正式60Hz及scoped native編譯成功，data26f34a124129cc46／呈現ca42ac58715f2f69更新，協定不變、未部署／全驗收，21/31與完整5.5保持，見declarative-control-effects-progress／E249。

2026-10-05 Bot控制決策：role_bot_inputs讀自己authorized owner標準控制狀態，不讀敵方Buff；暈眩本次完全等待、不清或改原命令，沉默只跳施法仍允許正常普攻／技能學習。解除直接用當前committed disclosure／正式輸入恢復，不建立私人等待計時或英雄特例；保留權威handler的獨立拒絕。新正式60Hz與相鄰案例成功，沒有內容hash／協定／UE改動，root策略／韌性免疫／驅散未擴充，完整5.5與21/31保持，見role-bot-control-state-progress／E248。

2026-10-05 宣告式暈眩：共用 schema 與 Rust registry 接受 stun_enemy duration_key，嚴格完整每級1/1024–60秒與敵方unit／射程契約；傷害等整份效果預檢與資源准入後才提交標準 BuffId::Stun。既有移動／普攻／施法阻擋、公開Buff視覺与權威動畫暫停同源，短控制刷新不得縮短長控制，不另造角色C++／Blueprint或runtime Lua。先鋒破岩擊資料接入；不擴充多來源驅散／韌性／免疫。局部生成與正式60Hz確認通過，完整data hash49f5b104fdd4c75c更新，identity／呈現hash／協定保持；未部署／Unreal畫面驗收，完整5.5與21/31維持，見declarative-stun-progress／E247。

2026-10-05 LAN程序所有權：server監聽位址與runtime目的地分開配置，host預設127.0.0.1且可顯式指定unicast IPv4；--connect建立remote-client plan，禁止包含server。--local-player僅篩選配方真人，不改完整authority roster／Bot／AUTH；每台本機runtime與Unreal仍用loopback IPC和獨立埠。remote讀host最終JSON且不接受選角／hero override，沿正式Rust preflight與原生版本協商，不把檔案當認證。健康與清理只處理本轮owned PID；既有本機流程保持。當前native設定與mock程序7/7、相鄰7/7成功，未做兩台LAN或UI驗收，詳見network-role-launch-progress／E246。

2026-10-05 共用選角launcher：固定Lua僅做本機程序協調，不執行玩法；配方1真人保留舊入口、2..10真人由唯一Rust host持有房間，各renderer使用自己的invitation與獨立輸出目錄。Unreal保存合法終局reply收據（followers可為stale error／null plan），Lua檢查身分／hash／shared_room／finalized及host snapshot完全一致；開局僅讀host canonical配方，整份規則／Bot／隊伍／角色不可變。無收據退出是取消，有收據才允許有界發布等待；全員與host退役後交給既有prepare／launch。每程序不可覆寫ownership記錄與獨立cleanup錯誤處理，不用全名kill／假鎖定／自動finalize。13/13模擬確認與native scoped build成功，不代表UE執行或LAN完整流程，詳見shared-selection-launcher-progress／E245。

2026-10-05 共用選角transport：moba-config的獨立host使用唯一SelectionRoom及每真人隨機invitation，先驗schema／hash／token才bind fixed-player service；localhost動態port為預設，明確LAN TCP仍是未加密bearer transport，不冒充TLS／完整帳號准入。worker10／handshake16KiB與5秒、write10秒、disconnect退役lease；明確blocking避免listener polling模式影響worker讀取。Gateway從invitation連host再沿原生pipe回覆，不新增角色socket或runtime Lua；shared_room標記啟用0.5秒read，stale reply有finalized authority snapshot仍退出followers、不寫假配方。ready／invite／host finalized plan同目錄完整pending檔sync後hard link原子發布不覆寫，owned errors.md有界且無秘密。當前4項局部確認與scoped native成功，Editor未執行；Lua多renderer lifecycle／LAN實跑仍待，完整6.2／6.4保持，詳見shared-selection-network-progress／E244。

2026-10-05 多人選角共用核心：SelectionRoom 以 Arc／Mutex 保存唯一 HeroSelectionSession，host 用已准入真人 bind 固定身分的 SelectionService，舊單真人 new 也沿此路徑。驗證／mutation／回覆同一 lock，並發相同 revision 僅一方提交；poison fail closed，不恢復可能部分狀態。成功 finalize 配方保留供 host take_finalized_plan 一次領取，不依賴最後玩家連線，不由 read／reconnect／EOF 補交接；這是記憶體內 host handoff，不宣稱 crash durability。command／JSON-line protocol1 不變，不新增玩家自報身分或假共享大廳。LAN transport／已准入連線接線／UE 遠端選角仍待實作，完整6.2／6.4保持，詳見 shared-selection-room-progress／E243。

2026-10-05 權威動畫暫停：host-local 具名 AttackAnimationTiming 由 hero_tick 在 stun／零dt 標記 paused，不用快照比較推測私人控制來源。AVS2 26bytes 只公開序號／階段／paused／elapsed／duration，idle canonical、strict bool，既有安全狀態路徑傳遞；bridge flags bit2 與 global pause 接入 native animation／phase payload，游標按权威進度定位後凍結、解除後正常恢復。selective wire5拒絕4、C ABI14拒絕13、IPC4與既有layout不變；最後部署同步更新，不能靜默混用。core4／bridge3／真實60Hzsource2共9項及native22actions通過；未做Editor／畫面驗收，21/31／6.1保持，見 authoritative-animation-pause-progress／E241。

2026-10-05 原生動畫重設：Reset 與 fallback 共用清除 asset／slot／action／游標的路徑；重複快照僅恢復未完成非循環或合法循環片段，已完成片段保持停止。soft pointer 優先 Get，失敗素材快取保留；首次播放檢查實際素材長度／裁切區間，縮短後空區間走 fallback，不先接受再於下一次停止。scoped native13＋最終3 actions通過，6個新斷言僅編譯。生成內容與版本不變；完整4.3／6.1保持，詳見 native-animation-reset-playback-progress／E240。

2026-10-05 原生預設倍率：Lua default_play_rate 生成 NativeDefaultPlayRate，普通／legacy 動畫使用共用 helper 合併有限正值預設與當前倍率，以 double 中間值及 0.01–10 邊界避免溢位。合法權威 attack phase 直接以 clip span／duration 同步，沒有美術／state 二次倍率，Impact rate 0 保持。codegen 指定局部測試／生成check通過，新增native斷言只編譯；generator_version5，生成hash b348872bbe367985，data／identity不變。完整6.1保持，見 native-animation-default-rate-progress／E239。

2026-10-05 原生 fallback policy：Lua 三種 policy 生成為通用 enum，初始化／狀態更新共用播放路徑；UseGenericState 可使用通用 attack／idle，UseReferencePose／HideVisual 在基本片段缺失時不替換 action。無片段時清除舊 asset／action／loop，參考姿勢或隱藏模型，下個有效片段恢復顯示；不改 actor 視野控制／gameplay。codegen 1 項與生成／check 通過，native scoped 16 actions 成功，新原生斷言只已編譯。generator_version 4，content_hash 79bf52bd0ba256e2，catalog data／identity 不變；6.1 未完成，詳見 native-animation-fallback-policy-progress／E238。

2026-10-05 通用動畫宣告：移除生成器由任意 sniper 片段推斷 Saika 狀態的分支，特殊狀態只由 Lua 顯式配置。Rust Lua 建置與 Unreal codegen 共用 animation_metadata 驗證六種映射、idle list、f32 可表示的正值播放倍率及 fallback policy，輸出前拒絕非法型別，不靜默使用預設。4 項局部測試及正式 --check 15 檔通過，生成 bytes 不變；完整 2.2b 保持未完成，詳見 generic-animation-metadata-progress／E237。

2026-10-05 正式攻擊動畫來源：hero_tick 保存實際 effective_interval 解析的前搖／後搖，host-local metadata 由 serde 跳過。AVS1 固定 25 bytes 僅公開 sequence／phase／elapsed／duration，與可見英雄 baseline、逐步 AttackVisual absolute event 同源；不披露 target／Buff 來源或合成暴擊／命中。filtered 只更新既有 component，bridge 使用 Q10 時間直接投影既有 C ABI 13，相位 consumer 不增加角色特殊程式；明確 idle 阻止舊 FX 重播。selective wire 4 拒絕 3；IPC／C ABI 不變。當前來源／codec／投影局部驗證見 authoritative-attack-visual-state-progress，完整 6.1 保持未完成。

2026-10-05 正式可見移動動畫：filtered converter原is_moving固定false且不披露private命令；presentation-only bridge從同live ID／disclosure generation的兩個有限正HP公開位置觀測移動，第一次／再披露不猜測，重複tick保留、較舊tick重新建基準；缺席prune、retirement／ResetView／stop與restart清除，control-only不觀測。embedded／TD來源維持。新4項局部測試通過，含IPC敵方位置→animation及coalescing／control／reset；不改ABI、IPC、wire或Lua runtime。調查確認CommittedAttack僅13bytes counter／sequence／phase，actual interval受Buff倍率影響，不能猜正式攻擊相位時間；E233／E234為consumer，不代表formal來源已接通。21/31／6.1保持，詳見disclosed-motion-animation-progress／E235。

2026-10-05 攻擊動畫相位恢復：共用 HeroAnimationBinding.clip_timing 檢查有限片段／來源時脈、嚴格內部且非循環的 impact_tick，兩建置生成器同用；Unreal生成 ImpactSeconds，未知保留 -1。原生 selector 成功後依安全 phase/progress/duration 定位前搖→命中→後搖；同 actor 重建／新action可直接恢復後搖，命中frame暫停，兩種回呼共用播放路徑與PlayRate。游標不觸發animation notify／gameplay，只做呈現；缺metadata保留舊播放、非法值不推測。model1／codegen5與scoped native22 actions首次成功，新native斷言僅已編譯（E224），不stage或全驗收。21/31及6.1保持，詳見 native-attack-phase-cursor-progress／E234。

2026-10-05 原生動畫覆蓋接線：持續 Buff 選出的 overlay／locomotion variant 接通用 NativeAnimationStateSlots；Lua ue.native_visual.state_slots 宣告 state→既有片段，生成階段驗證有界名稱、FName 大小寫重複與目標存在。攻擊優先、走路不退回站姿、移除恢復待機；失敗資產快取剔除並同 frame 有界回退。動畫 tick 時間使用 driver 已協商 LockstepTiming，未知不猜 120Hz，過期／未到攻擊不保持 action。局部生成器1／bridge4通過，native13 actions＋最終資料3 actions成功；native斷言未執行（E224）。21/31與完整6.1保持，詳見 native-animation-state-progress／E233。

2026-10-05 宣告式 Buff 來源：Lua buff.ue.buff_visual.sources→兩生成器共用 effect/reference validator→編譯 Rust lookup；canonical 私有 source key 僅 host-local 解析，公開 ID／剩餘時間合併，不發 caster／stat／payload。slow／回魔／容量資料接通，不改 gameplay／ABI／wire。model1／ABI1／core7／正常generated正式60Hz2、生成check15檔與native12 actions成功；bridge最終結果见 declarative-buff-visual-source-progress／E232。不 stage／畫面驗收，不勾6.1與21/31保持。

2026-10-05 Buff HUD 完整清單：frame 中配置玩家的唯一 hero＋精確 generation→合法 catalog active state 的有界 batch，原生 controller／Slate 顯示整份列表而非最後一筆。完整空快照與 Stop／Reset 清除、control-only 保留；舊單筆 API 不再自動綁定原生 HUD。不顯示 payload、不新增披露／角色 graph／ABI。scoped 14 actions 首次成功；新 native 斷言未執行（E224），21/31 與完整 6.2 保持。詳見 owned-buff-list-hud-progress／E231。

2026-10-05 隊友 Buff 安全來源增量：正式 Rust committed BuffStore→BVS1，僅同隊有玩家 owner 的 Disclosed 英雄、已註冊 ID／剩餘時間，不公開 payload／來源／動態 Mana；每 tick OwnerBuffVisual 更新與空集合清除，baseline／白名單／filtered replica／bridge 接通。不重播歷史 Added／toggle。Selective wire 3 改共用唯一常數，模式 V2 名称保留；局部結果見 owner-team-buff-source-progress／E230，21/31 與完整呈現待完成保持。

2026-10-05 持續 buff 視覺增量：既有 active_buffs→live actor 的獨立 OnBuffState，不重播 Added／Refreshed；visual key 重用與完整 baseline 退役、control-only保留。強化actor rebinding native斷言，首次C2248改automation-only accessor；編譯結果與限制見 persistent-buff-visual-state-progress／E229。正式MOBA尚缺typed visual-only buff來源契約，不擴大披露、不勾6.1，21/31維持。

2026-10-05 Actor 內容綁定增量：replica 內容變更才解析類別，先取得新 actor 再回收，未變綁定走快路徑；同 replica 保留 cue／attack 去重。通用 virtual reset 清插值／tracked effects／nameplate，Hero 清原生動畫 instance；Remove／Reset 同步清綁定。15 actions scoped native 編譯成功，新 automation 未執行（E224）；持續 buff baseline／無通知 asset hot reload 不算完成，21/31維持。詳見 actor-content-rebinding-progress／E228。

2026-10-05 未知內容通用 fallback：safe render 不再把 missing／malformed／empty ID 補成 Saika／dummy，catalog 未註冊小兵不再重映射到 dummy，ID0 使用既有 generic native actor；明確 ID／已披露 Hero.id 維持。移除 Saika-only 插值診斷分支，通用 VeryVerbose 降低預設熱路徑 log。局部3/3與native3 actions成功，無部署／完整原英雄驗收，21/31維持；詳見 unknown-content-fallback-progress／E227。

2026-10-05 通用投射物命中增量：真正 ProjectileHit→target-only ProjectileImpact29／HIT1，投影要求 visible＋Disclosed；IMP1 共用 cue retention／ACK／bridge lease，C ABI13 的 FxImpact2 與 damage1 分開，native 受擊者位置 fallback 不猜碰撞點或隱藏種類。局部確認與限制見 projectile-impact-cue-progress／E226，不部署或重試 E224 啟動條件，21/31維持。

2026-10-05 通用範圍效果：executor整體preflight／資源與效果提交後emit_explosion，正式cast adapter收集有界area結果、成功gate後發布真正caster／skill／rank／team的AbilityArea，同隊且Disclosed caster gate；不再同時發匿名Explosion。APC1／ARC1獨立種類共用retention／ACK／catalog，native明確center／cm半徑／duration球形fallback，不套舊倍率、不新增script ABI。局部2/2＋來源1/1、core8/8、client7/7、bridge5/5與12 native actions成功；native斷言未執行、不重試E224基線、不部署／全鏈。跨隊效果／impact另待契約，21/31不變，詳見ability-area-cue／E225。

2026-10-05 通用caster位移結果：成功invocation的ScriptSetPos overlay提供實際提交終點，fact明確team與raw Fixed64，safe projector只向同隊公開終點，可見對手維持identity／rank。ABS3／ABY3共用既有cue保留／ACK／epoch；bridge沿C ABI13 point＋flag，native明確presence使世界零點不被當缺值，不新增角色分支或script ABI。局部來源1/1、core7/7、client7/7、bridge3/3及20 actions project modules成功；native啟動遇engine／project BuildId錯配，沒有執行automation，不手改ID。範圍中心／impact另待契約，21/31不變；詳見caster-relocation-cue／E224。

2026-10-05 共用施法等級解析：無Mana handler先前以as u8溢位，可能實際execute1卻公布rank257。dispatcher在任何handler／Mana變更前checked解析一次，拒絕超u8／declared max；正式／managed另拒未習得與缺level data。保留legacy無Mana的execute1／呈現0未知，成功gate不變。新36組矩陣1/1＋相鄰3/3、2/2成功。execute API未提供actual effect point，不把請求點冒充結果；位置契約待後續，不部署／不勾整項，詳見checked-cast-rank／E223。

2026-10-05 權威技能等級：共用dispatcher從同一serial Hero ledger／cache在handler前捕捉rank，成功gate後進Ability fact；safe projector僅對可見Disclosed caster公開ABS2，client共用ABY2保留／ACK，bridge沿既有AbilityCast.level。舊8bytes／ABY1維持0未知，不猜target／point／toggle，精確格式與int32上限，C ABI不變。局部權威3/3、core6/6、client7/7（三種格式IPC）、bridge3/3；未stage／native重跑／完整同局。21/31與6.1待完成保持，詳見authoritative-cast-rank及E222。

2026-10-05 權威施法來源：成功SkillCast visual先前仍未發布fact；共用dispatcher成功gate後經既有converter送ObservableFactBuffer，explicit HERO_ABILITY policy、跨同tick drains不重複ordinal。基本ECS同層初始化policy registry／fact buffer／cast ordering，不做臨時fallback。真實generated handler的60Hz headless來源＋顯式共用safe projector／Mana／拒絕指定3/3，既有cast visual相鄰2/2成功。headless結果不是自動server team frame，未部署或真實全鏈；6.1／4.3、21/31保持，詳見authoritative-cast-fact與E221。

2026-10-05 施法Unreal消費：bridge共用PresentationCue處理ABY1／DMG1的safe admission、busy lease、coalescing、lifecycle與published未消費保留；共用既有FNV-1a hash→compiled catalog lookup，未知／非法身分不派發。生成caster-only AbilityCast到自有frame字串，不猜目標／rank／toggle；原生cue history去重、cast fallback不誤判disabled toggle。局部Rust及限定三project modules編譯通過，完整restart仍engine change阻擋，未放寬NoEngineChanges；native synthetic初輪warning已補preview context，最後證據見ability-cue-unreal-consumer與E220。未部署／完整同局，6.1／4.3及21/31保持。

2026-10-05 施法可靠 IPC：共用 PresentationCue／CueRetention 管理 DMG1、ABY1 的 live dependency、總容量1024、Hide／Forget／Reset／同connection成功傳送後ACK及重連基線。正式 client 每成功frame在呈現降頻前擷取安全施法；權威排序後跨external／public配置序號，避免tick＋producer-local ordinal碰撞。client指定7/7（含兩種cue實際localhost IPC）、core4/4、正式binary check成功。bridge／Unreal仍待ABY1 admission、catalog lookup及原生消費，不勾完整4.3／6.1，21/31保持；詳見ability-cue-ipc-retention與E219。

2026-10-05 安全施法 cue 契約：Ability 只在施法者可見且有 Disclosed replica mapping 時發布，禁止可見目標使隱藏來源的技能事件洩漏或 subject fallback 冒充施法者。共用 ABY1 固定 36 bytes 只含 tick／caster replica／epoch／stable ability ID，嚴格 event ID／版本／長度／live epoch；不猜目標、位置或 ACK 成功。此批只完成投影及格式，可靠 IPC 保留／重連去重／catalog lookup／UE 消費仍待接線，6.1／4.3與21/31保持；詳見 safe-ability-cue-contract 與 E218。

2026-10-05 成功施法事件：script dispatcher原先在gate前加入SkillCast visual，legacy無Mana拒絕可能仍發布；共用adapter.cast_succeeded gate對每次cast回復自己的visual checkpoint，保留早先成功事件與其他hooks。generated handler／正式input adapter／真實dispatcher指定2/2確認有無Mana拒絕、同批成功後失敗、cooldown與缺handler。不是全鏈IPC／UE／reconnect驗收，不改audience或legacy Outcome語義；詳見successful-cast-cue與E217，6.1／4.3及21/31保持。

2026-10-05 技能fallback：Lua ue.ability_cue生成sphere／toggle／formation／cone／fan通用原生registry，Actor刪四技能ID派發。cm與payload_distance_scale明確換算、不按值域猜單位，角度在draw邊界轉radians，event數量／距離有界。保留Blueprint與C ABI，不改玩法／視野／ACK。生成器1/1、12項project build與NullRHI原生1/1成功；full catalog hash23c4614509fdbf2d，完整6.1／2.2b仍待最後，詳見declarative-ability-style與E216，不維護omfx。

2026-10-05 投射物 fallback：Lua 可選 ue.projectile_cue 的 RGB／有界正數線寬與半徑生成原生 registry，OmGenerated 啟動安裝／卸載清空，Actor／UObject／塔色共用 Runtime lookup。未知 ID 使用通用預設，不再依投射物名称分支；純呈現、不改傷害／視野／cue 去重。完整 catalog hash40641b573c2890e9，既有 presentation hash 不包含此設定，仍要求 full catalog handshake。生成器指定1/1與15項project native build成功，單項結果見 declarative-projectile-style 與E215；完整6.1／2.2b／對局仍待最後，不維護omfx。

2026-10-05 開局順序：共用 workflow 先建 selection CLI／server／client runtime／前端，先檢查部署再開選角，選角完成後再 guard 部署才開局。verify-staged-only 改共用 bridge built→plugin、base_content built→server與plugin 的 SHA 契約；no-build 不跳過、prepare-only 不啟動程序。report 明確僅為 artifact-copy-consistency，不當作 ABI／feature／内容或遊戲驗收。局部 fixture／mock 8/8 成功，不勾完整6.2，詳見 moba-launch-stage-contract 與 E214。

2026-10-05 原生選角確認：保留 -NoEngineChanges 的本批 project build 已通過，E177 不再阻擋；真實 C4458 Player 遮蔽已改 SelectionPlayerId。選角不 LoadBridge，PreciseTap 使用真正已布局按鈕；run1791181633-1 三次 pointer 回呼→Rust select／lock／finalize→最終配方成功，没有啟動 gameplay。這不是完整選角到結算、LAN 或效能驗收，6.2 保留未完成；詳見 unreal-selection-native-confirmation 與 E213，舊 C++ 未編譯說明為歷史狀態。

2026-10-05 使用者最新範圍決定：唯一維護前端為 omfue。omfx／Fyrox 保留歷史檔案，不再實作、修復相容、建置驗證或列入最終驗收；共用 Rust schema／API 改動只維護正式後端與 Unreal 消費端。歷史 Fyrox 修復紀錄保留作已發生事實，不能據此重新擴大維護範圍，不刪除其 submodule。

2026-10-05 原生選角入口：通用 UOmHeroSelectionWidget 經匿名 pipe 與 Rust selection service 交換 compiled catalog／席位／select／lock／finalize，decimal revision／request tokens 避免 double 截斷。獨立 om-hero-selection 模式封鎖 gameplay bridge；明確完成才寫不可覆寫回覆並退出 renderer。Lua opt-in --interactive-selection 等待退出後驗證 hash／身分／配方只改真人英雄，再沿既有正式開局流程。取消不開局，僅限一真人＋Bot，不冒充共享 LAN 大廳。Rust binary 與 Lua 交接局部已確認，新 C++ 尚未編譯，E177 限制保留，不勾選 6.2；詳見 unreal-hero-selection-entry 進度與 E212。

2026-10-05 持續選角入口：共用 Rust SelectionService 以 host-bound 真人席位處理嚴格 JSON-line read／select／lock／finalize，moba-config --selection-session 暴露 stdin/stdout 入口。每則驗證 protocol1／完整 catalog hash／revision，初始回覆 compiled hero catalog；16 KiB framing、立即 flush、EOF 不自動鎖定或開局，超長 frame 終止防止尾端被誤解。不是連線認證、Unreal UI 或多真人共享大廳；其他真人未鎖定仍拒絕 finalize。當前局部確認見 hero-selection-service 與 E211，6.2 維持待完成。

2026-10-05 共用選角鎖定：Rust HeroSelectionSession 以完整 compiled 配方驗證建立，不持有 World／Lua VM；真人 own seat Select／Lock、Bot 已鎖定、revision 拒絕 stale、拒絕原子性、all-human ready 才 finalize，finalize 後凍結。請求沒有 player ID，host 從 admitted identity 傳入，kernel 不是認證。建置 HERO_CATALOG_IDS 導出與准入同源的選角 metadata，moba-config --lock-plan 供可信本機 host 準備並由正式 launcher 保存 candidate／lock／catalog／final JSON；不是多人同意或 UE 畫面，6.2 仍待完整串接。局部結果與錯誤見 hero-selection-lock 及 E210。

2026-10-05 架構更正（優先於歷史 runtime Lua 描述）：Lua 是作者／建置生成工具，不是正式遊戲腳本 VM。compiled-content-only 經 server／client runtime／base_content／bridge 傳至 template-ids，與 runtime-lua-content 編譯互斥；bridge 不再預設啟用 Lua，legacy DEV 功能保留為明確 opt-in，不能混入正式建置。om_codegen authoring 只在 build dependency 使用，bridge 對 target AUTHORING_ENABLED 做 const assertion；build.rs 生成嵌入 catalog，遊戲不求值 content_root。生成開局設定與子程序環境明確停用 Lua，Lua recipe 僅在啟動工具準備 JSON／TOML 時求值，保存經驗證 match-plan.json 並支援 JSON 再選角。後續選角 UI 必須提交普通資料給權威規則，不執行 Lua。局部已成功，繼續 6.2 選角與剩餘 MOBA；最後集中完整驗收，詳見 compiled-content-only 進度與 E209。

2026-10-05 開局選角：共用 Lua select_heroes 複製 server-owned 配方，僅允許指定既有真人玩家的英雄；CLI --hero PLAYER_ID=HERO 可重複不同玩家，拒絕重複 ID／Bot／未知玩家。英雄目錄仍由正式 Rust moba-config 檢查，生成設定保存真正權威 MATCH_ROLE_PLAN_JSON，launch-plan 保存 human_heroes；不新增 renderer 選角權限或英雄專屬程式。這是 6.2 選角資料入口，不是完整 UI／大廳，詳見 pre-match-hero-selection 與 E208。

2026-10-05 Jungle清野一致性：純jungle_farm_target在當隊current披露選kind3／team0／正HP／550內nearest stable canonical目標，decide及role focus共用；助戰仍優先，EnemyPoint沿既有focus排序將near camp優先於far cluster，但最低覆蓋／射程／成本CD均保留。單體早已distance優先、neutral早已支援，不重寫既有語義；沒有hidden timer／營地私有AI或hero特例。局部結果與fixture錯誤見jungle-farm-focus、E207，完整5.5仍待最後驗收。

2026-10-05 折線推進：非Jungle路線使用本人formal AttackMove目的地維持旅行，50單位到達才前進；中斷無有效route目標時以Fixed64最近線段投影重定位，不再最近頂點＋1提前跳角。public路線／committed本人位置／owner commands，不用hidden敵人或新增cursor；末點不環回，重疊／空單點安全，反向路自然適用。局部core／正式60Hz結果與fixture E0616修正見lane-segment-progression、E206，5.4／5.5仍待最後整體驗收。

2026-10-05 Jungle巡邏：public camp點＋committed本人位置＋本人正式AttackMove destination共用純arrival planner，50單位內才依宣告順序循環到下一個未到達點，重疊點跳過／空單點安全；途中不受tick時計改道，combat中斷後最近點穩定重選。保留技能／回城／恢復優先，不讀camp hidden狀態，不新增第二World或私有游標，不稱完整navigation／安全gank。局部結果見jungle-arrival-patrol與E205，整體20/30及5.5仍待最後驗收。

2026-10-05 cue准入：DMG1 payload與effect ID／tick契約抽core from_effect，runtime ledger及bridge共用1024常數。bridge先take限制解碼輸入，再filter；無效／未知也占budget，超量尾端明確warn並丟棄。一次live披露tuple索引替代逐cue全實體掃描，同批有效ID去重；不改hidden target gate、跨frameACK、reconnect基準或C ABI13／IPC4。這是damage契約，不當成所有cue／完整4.3已完成；詳見damage-cue-admission與E204。

2026-10-05 新命令能力：Hold採獨立command protocol1＋CONTENT_CATALOG_DATA_HASH，不借shop或Recall開關。JoinRequest16／17與TeamGameStart29／30 append-only；legacy0／空值關閉，混合或不匹配拒絕。server需selective安全binding及MOBA mode、owner與nonzero input ID；fresh／cached bootstrap回報一致，client在配置ID前檢查。accepted input追加action_kind20／無target，internal Bot仍走正常權威命令，不依human socket協商。IPC4／C ABI13／KCP2不改；最後完整驗收另做，見E203與command-capability進度。

2026-10-05 Bot持續等待：基地HP／Mana恢復與近距離Escort不再MoveTo自己／idle，與推塔共用正式hold_input；active Hold＋empty queue去重，stale queued正常replacement清除，recovery_move只處理真Move。恢復門檻與committed隊友遠離後正常release，無ECS直接改動／新前端分支。core新1與base相關既有4更新後成功；工具context截斷patch failure／artifact lock等待見E202與role-bot-persistent-wait，完整5.5待最後驗收。

2026-10-05 輸入契約收斂：omoba-core renderer_protocol純資料API持唯一PlayerInput→renderer intent與IPC4／magic／framing上限，Unreal／Fyrox只forward、client re-export；修Fyrox仍固定VERSION3。exhaustive action match、queued／target／slot／catalog完全保留，unsupported／缺point拒絕；encoder不取代owner／epoch／安全目標准入。新core3／client1、既有bridge1與omfx lib check成功，不改C ABI13／內容hash，完整4.1仍待，詳見shared-renderer-input-codec進度與E201。

點位排隊相容補充：Fyrox native InputActionKind少前批Hold分支令check E0004，已補明確分類與converter；omfx lib check成功。不是恢復Fyrox為主要前端，僅維持共用schema依賴者可編譯，不吞Hold為NoOp。詳見E200。

2026-10-05 點位排隊修復：MoveTo／AttackMove在IPC原缺queued，InputBridge固定false；兩intent附加bool3、Unreal與Fyrox轉換／client admission保留。Presentation IPC4嚴格拒絕3，不變selective2／HUD schema2／C ABI13／Lua hash；同步機械生成game.rs。新client2／bridge1／正式60Hz base1成功，立即Move位移fixture首次時序錯誤修正見E200；完整4.1／6.2與UE按鍵待最後。前批Hold共用Runtime action／H與Shift+H程式已接ABI13，UE實際執行仍待E177解除，不能稱已部署。

2026-10-05 通用推塔等待：MoveTo自己抵達後解除不能當Hold，因此新增PlayerInput／RendererInput HoldPosition20，正式持續command／queue replace／attack interruption、安全MovementPriority replay，owner/epoch與Recall中斷沿用。role Bot觀察current活同隊兵距塔650內才取塔，無援軍1100區持續Hold、不重送，未提交／hidden兵不提供時機；不保證塔aggro或免傷。新core1／client1／正常60Hz base2及12雙隊filtered steps hash一致零repair；舊攻城fixture跨地圖／存活錯誤見E198與siege-wave-hold。UE Hold按鍵仍未接入，完整20/30保持。

2026-10-05 通用攻城披露：塔基地原有Unit令render.kind=2，不改renderer而新增0x464f4710 v1 role／admission及CommittedStructure28 changed absolute fact，原視野audience、權威同一unlock與phase規則，finish勝負後擷取。Bot內部區分tower／base／locked，普通戰鬥優先，Carry不套兵尾刀，無hidden prerequisite或HP查詢。core新2＋既有1與正式60Hz base1／8雙隊filtered steps hash一致零repair；fixture錯誤与剩餘見disclosed-structure-siege進度、E197；20/30及完整5.5維持。

2026-10-05 Carry普攻／技能協調：安全尾刀候選在技能前只算一次，若合法既有Windup目標仍可擊殺則保留。Idle就緒或同目標負count前搖時跳過進攻intent，恢復類作者順序與sustain／回城保留，Backswing不阻擋施法；其他role不改。不是任意Cast取消普攻的根因宣告，沒有修改gameplay取消規則或預測未來impact。core1／正常generated正式60Hz base1成功，詳見carry-attack-priority與E196；完整5.5仍未完成。

2026-10-05 通用入傷觀測：0x464f470f v1 聚合 bonus baseline＋CommittedIncomingDamage27 changed absolute fact，在正式 finish／fact barrier 前擷取，沿既有視野 policy 發布，不公開 buff 身分來源。filtered baseline／apply／hash保留，Hide變更不洩漏、Reveal帶新值，沒有每tick ComponentRepair。Carry普攻只依 current披露敵兵與本人final_atk／range／accuracy，用正式 packet共用公式優先目前可擊殺者，不承諾未來投射物尾刀。core2、正式60Hz base2與12雙隊filtered steps零repair／hash一致；限制、錯誤與決策見incoming-damage-observation進度與E195，完整5.5／20/30不變。

2026-10-05 通用兵線尾刀收益：Lua lane_creep_gold20、省略預設0、u32≤1000000→compiled constant／SingleLaneConfig；RuntimeContent compiled agreement／ContentShape不允許規則熱更新／full hash同步。兩條實際正傷害路徑首次致死即退休LaneUnit生命，只有目前roster敵hero source領Gold；NPC／匿名退休但不付、Heal後不可補領，forced Death不付，不共享附近XP recipients或增hero KDA。正常Gold飽和、死亡保存與owner安全投影不變，無角色前端分支。base正式60Hz新2／template正確feature新1／codegen生成check成功；data hash502a59ee5aa2a677，未部署／完整驗收，詳見lane-last-hit-gold進度與E193。

2026-10-05 Support護衛：既有escort_player_id選current披露同隊存活Carry，距本人550且敵hero距Carry350內，以距Carry／本人／canonical ID排序；不讀敵方command／aggro、hidden cache不供escort身分，不稱為實際攻擊者辨識。role_combat_focus每位Bot一次統一Jungle支援與Support護衛，正式AttackTarget及進攻unit／point／AoE共用focus優先，range／rank／Mana／CD／min_targets與作者heal-first不變。core2與正式60Hz結果、fixture容量失敗見support-guard-focus進度與E192；完整5.5仍未完成。

2026-10-05 友軍玩家輸入：既有通用 unit 游標選取與 bridge／IPC／client secure reference 不限定敵隊，保留權威效果同隊判斷，不另造角色 C++／BP 或 ABI。生成期 shared validator 与 fixed Lua 對同一目標混 heal_ally／damage／slow_enemy 同步拒絕 conflicting unit target allegiance，避免生成永遠失敗的技能；runtime全preflight仍保留。單項確認／限制見friendly-cast-contract進度與E191，完整UE及整體5.5／6.2保持待驗。

2026-10-05 通用隊友治療：Lua heal_ally→shared model／fixed registry→whole-plan preflight後deferred heal，既有faction_of補opaque combat-team identity、不改FFI方法表。Bot current披露maxHP／相對傷勢精確排序與range／成本CD；追加Support術士配裝variant不改舊hero／skill ID。多hero共享技能生成只一份implementation、衝突拒絕。core1／正式60Hz base2／舊generic8／Lua7／codegen生成check／九Botprepare成功；真實編譯錯誤與路徑重犯見E190。玩家端友軍選取與完整UE仍待，20/30不勾5.5／6.2，詳見support-ally-heal進度。

2026-10-05 Jungle局部支援：共用assist focus只看已提交的存活hero披露，550範圍內有另一位同隊hero且可見人數不劣勢才優先支援；本人以owner ID排除、HP／距離／canonical ID穩定決勝，hero子集避免反覆掃creep。普攻與unit／point／area意圖共用focus合法候選與原range／cost／CD，不讀hidden state、無hero専屬分支。core新2／正式60Hz base新1證明傷害65與CD／AttackTarget、未提交ally pose與hidden cache不影響；舊role6成功。不是跨路策略gank／安全保證，100場／完整5.5未勾選，20/30不變。詳見jungle-local-assist進度與E189。

2026-10-05 Bot持續回魔：self_mana_regeneration作者rate／duration意圖、共用UnitStats唯讀flat bonus預測、比較扣權威cost後capped期末餘額，不重算baseline regen收益。generic_mana_buff_id在script-abi與正常Lua生成handler／Bot共用含生命身分；同source存在只擋Mana分支，HP治療仍可刷新。只接opt-in ranger_patch，不改平衡或把容量／百分比當flat；目前modifier預測不是未來收益保證。core新2／base正式60Hz新1／九Botprepare成功，整體20/30與完整5.5保持待驗，詳見bot-mana-regeneration進度與E188。

2026-10-05 Bot即時Mana意圖：self_recovery以HP OR本人低魔判斷，作者restore extras／strict門檻／權威折扣後cost正收益才選Mana分支，原HP療傷與回城安全優先不變；plan／runtime都要求explicit mana_enabled，僅opt-in配方換共享回春政策，無英雄runtime分支。restore_key是即時恢復作者提示，不代替額外spend／多段Buff完整預演。core2／正常generated handler正式60Hz base1與固定Lua單人九Botprepare成功；持續Buff策略／100場／UE仍待，20/30保持。詳見bot-mana-recovery進度與E187。

2026-10-05 三原型Mana內容：Lua先鋒整備capacity60–150／6–9秒、遊俠包紮自然regen2–5／6–9秒、共享術士回春restore20–35，原cost／HP／ID維持、無角色runtime分支。正常generated manifest正式60Hz驗三者cost／HP／pool／Buff，Bot預算仍正式input並按新regen7結算。case1（三subcases）／Bot1、codegen生成＋check、mana單人九Botprepare成功；full data hash14ef1dea84790afb，不混舊DLL宣稱部署；100場與mana-only Bot意圖仍待，20/30保持，詳見mana-archetype-content進度與E186。

2026-10-05 Lua持續Mana Buff：shared ManaBuffStat白名單與bounds、mana_buff_self(value_key,duration_key)，每技能unique stat；fixed Lua生成typed EffectOp，generic全preflight／資源准入後deferred add_stat_buff，Buff ID固定ability／stat／entity／generation，max-duration最新payload刷新、不增同來源層，跨技能可疊加。mana-enabled正式60Hz證明cost45／current45／capacity335不補满、重施一筆與失敗留旧Buff；新Rust3／Lua34／codegen --check成功。UI preview／十人100場仍待，20/30不變，詳見lua-mana-buff-declarations進度與E185。

2026-10-05 Mana容量Buff：UnitStats checked原始MOBA Lua容量＋ManaBonus／ExtraManaBonus、family語義重用；負值截0、invalid／overflow／超envelope退base，ManaPool set_maximum增容量不補滿、縮小截限。shared script dispatch建立快取前同步，Outcome後finish再同步，修正到期仍能cast消耗超額餘額；同hook新Buff保持deferred、None／inactive／legacy不改。新core1／正式60Hz base1／舊lifecycle3通過，不勾選整項，Lua持續Buff仍待。詳見mana-capacity-buffs進度與E184。

2026-10-05 Mana Buff回魔：MOBA自然rate經既有UnitStats keys聚合，基地恢復獨立保護且合併一次Q10 remainder；checked_sum_add用i128加總／unsigned_abs family最強，checked_mana_regen對flat與每factor截0、溢位／非法payload使自然0，不全面改其他stat。正式60Hz確認pause／expiry／基地與invalid恢復，core2／base新1／舊sustain2成功；容量Buff與Lua持續Buff仍待，20/30不變。詳見mana-buff-recovery進度與E183。

2026-10-05 Lua魔力effects：新增restore_mana_self／spend_mana_self，instant active／ultimate none-target沿每級extras與共用scalar envelope；固定Lua生成EffectOp、GenericEffectHandler全plan preflight後依作者順序使用真實pool准入資源，再emit heal等效果，RErr由正式host回滾。額外成本不是metadata成本且不重套其倍率；legacy不建立池、Mana沒有EffectSpec preview不冒充HP heal。model1／正式60Hz host1／generic7共9個不同Rust測試、Lua13與codegen --check成功，不新增角色C++或Blueprint graph、不勾選整項。詳見declarative-mana-effects進度與E182。

2026-10-05 腳本資源：沿既有 GameWorld ABI 實作 checked spend／capped restore，metadata由host預留一次、handler讀post-cost餘額；每hook transaction成功才commit絕對pool與下一dispatch通知。serial queued events與固定順序managed tick共享ledger，無managed的TD仍平行；legacy caster修改managed recipient失敗同樣rollback，移除舊cast view。新正式60Hz交易3／既有cast2與core Mana篩選19通過；不改英雄unit掛載、不新增角色C++，宣告式Mana effect／Buff與完整UI／100場仍待，20/30維持。詳見script-mana-transactions進度與E181。

2026-10-05 Bot補魔：sustain可選mana門檻／None及零容量不等待，配方明確mana_enabled與server MATCH_MANA_ENABLED一致才啟用；正常Recall／MoveTo與本人資源、披露威脅。Lua基地Mana速率60進full rules hash／agreement／no hot reload，authority post-combat共享活本人基地eligibility，與自然速率5合併一次Q10餘數結算。新core2／正式60Hz base2／template1／server1與舊HP1成功，新opt-in一真人九Bot60Hz Lua prepare、生成與check成功；一般預設不變、未stage／UE／100場，20/30不變。詳見bot-mana-sustain進度與E180。

2026-10-05 Bot Mana預算：shared checked_mana_cost原始f32→Q10與倍率進位由AI／authority共用；本人Hero／Buff與初始化同腳本metadata的AbilityRegistry，不讀已由SimulationDriver接管的ScriptRegistry。按作者priority跳過不夠／非法／缺資料成本，None保留legacy，AI不預扣且只發正式CastAbility。core2／短60Hz正式base1（rank4成本60跳過→rank1成本45、免耗Buff與精確扣費再生）／既有managed cast base2成功；補魔策略、script操作、UE／100場未完成，20/30不變。詳見bot-mana-budget進度與E179。

2026-10-05 Mana規則協商：JoinRequest14／15、TeamGameStart27／28追加獨立version1與full data hash；server MATCH_MANA_ENABLED預設false且限secure MOBA，一般與role-plan轉入mana_enabled。註冊前拒絕不支援／錯hash／非selective player，initial／rejoin bootstrap回覆實際啟用協商；legacy request保持zero。修正停用對局不發fact26。core1／server2／短60Hz base1成功；未做真實網路或UE驗收、不啟用實際launcher，20/30不變。詳見mana-agreement進度與E178。

2026-10-05 通用Mana HUD：本人disclosed pool→schema2 Q10 raw→bridge範圍與身分檢查→C ABI12→共用UE payload／native MP文字與藍色法力條；legacy schema1只允許unsupported canonical zero。runtime新2與bridge3成功；UHT成功，但共享D:/UE5.8已有Skeletal引擎修改觸發NoEngineChanges，native C++未完成且不繞過版本／引擎閘門。一般規則協商與完整UI仍待，20/30不變。詳見mana-hud進度與E177。

2026-10-05 Mana生命週期：明確 `SingleLaneConfig::mana_enabled` 預設停用；正式上限使用 Lua base_mana＋每級成長，不沿用舊智力公式，開局檢查1–25級。出生／新身分復活滿池、升級只增容量，Lua mana_regen_per_second編譯常數與full data hash／compiled agreement／no hot reload接線；有效時間post-combat結算排除暖機／暫停／死亡／結束，沿CommittedMana安全投影。base3／core1／template規則測試與短60Hz雙隊24steps逐tickhash零repair；一般網路協商／Buff及script資源操作／Mana HUD仍待，不改20/30。詳見mana-lifecycle進度與E176。

2026-10-05 Mana投影：append CommittedMana26／binary v1 disabled2與enabled20bytes，current／maximum／再生餘數均以checked pool invariant驗證，沿可見Hero post-step facts、team projector與filtered絕對結算；既有Hero baseline／fresh bootstrap與digest涵蓋所有欄位。core4、短正式60Hz12ticks雙隊24steps每tickhash零repair成功；fixture漏owner accepted input導致CD缺失已正常接線而非override。正常協商／規則啟用／生命週期／再生／script API／HUD仍待，20/30不變，見mana-projection進度與E175。

2026-10-05 managed施法Mana：serial SkillCast ledger按事件順序驗當级成本／倍率、餘額與CD；handler成功才提交ScriptSetMana，失敗丟棄本次adapter deferred effects／overlay／visual，已捕捉pre-hook panic也禁止提交。managed current_mana讀真實本批餘額，explicit script spend拒絕避免假成功或雙扣；其餘script資源變動仍待。正式60Hz輸入與同batch超支／重複CD base2、adapter1通過；正常網路池未啟用，規則／再生／生命週期／安全投影／HUD仍缺，20/30不變，詳見mana-cast進度與E174。

2026-10-05 法力池第一階段：shared Rust ManaPool 私有Q10餘額／容量與i128再生餘數、checked serde、合法扣除與有界恢復；Hero optional且明確冪等初始化，增加容量不補魔、舊JSON仍None。核心5／Hero2成功；adapter假實作尚未取代，正式規則啟用／ordered cast commit／Mana安全facts與HUD仍待接線，正常對局不先扣費，避免同tick cache超支或replica hash錯配。20/30不變，詳見mana-pool進度與E173，不把資料核心稱完整Mana。

2026-10-05 通用減速：Lua slow_enemy per-rank比例／時間→shared model／FFI→Rust全部敵對／存活／距離 preflight後提交；既有BuffStore整數Q10 move_speed_bonus、來源ability/caster/generation key與generic_ability_slow strongest-family，正式UnitStats／導航移動／TTL使用相同統計。ranger_shot資料附加減速，不加英雄C++或BP；model1／base2含正式60Hz／十二招1／Lua14、生成與--check通過。不是全legacy減速統一、cue／Mana／OmGame或100場驗收，20/30不變，見generic-slow進度與E172。

2026-10-05 即時位移：Lua dash_to_point exclusive point/rank range→共用model／FFI→Rust正HP caster、原地／距離驗證／既有Fixed64 swept-terrain query→精確合法終點才set_pos；拒絕不啟CD，不新增ABI／角色C++。vanguard_resolve保留ID改磐岩突進；Bot explicit approach_enemy_point只用披露living位置、minimum/compiled range與正常技能輸入，不寫Pos。model1／base2含60Hz薄牆／core1／Lua7與三原型十二招1成功、生成與一真人九Bot60Hz預檢通過；不代表持續衝刺／動態推擠／Mana／filtered或UE驗收，20/30不變，見generic-dash與E171。

2026-10-05 技能數值契約：shared model對非tombstone全部rank cooldown／mana_cost／cast_time／range與宣告式amount要求零或有限[1/1024,1000000]；target range／radius保留[1/1024,10000]。Lua FFI同步驗完整rank／門檻與特殊handler，先轉f32避免double邊界單邊拒絕，不clamp或改正式內容。model2／Lua34／原include5／24項正式註冊、base2含60Hz與codegen--check成功；Mana adapter仍假實作、faction_of未實作，本批不是Mana／盟友功能，20/30不變，詳見ability-numeric-contract與E170。

2026-10-05 宣告式施法驗證：單體 Damage 原缺權威距離檢查，新增 shared model／固定Lua per-rank bounded range，共用執行器有效／正HP caster與單體兩端位置／精確距離、HealSelf強制None。整個序列preflight後才輸出，原dispatch僅成功啟CD；AoE只限中心，展開victim不套單體range。新base2／model1、更新AoE正式60Hz1、原generic3與Lua5、UE生成--check成功；無新角色C++／ABI或OmGame-stage，不將局部确认當完整驗收，20/30不變，見generic-cast-preflight與E169。

2026-10-05 通用範圍傷害：shared AbilityEffect／Lua FFI 新增 area_damage 的 point、amount_key／radius_key與rank資料驗證；Rust host query展開、(id,generation)去重、HP0排除、caster range與全序列原子preflight，最多128 resolved effects，不改script ABI。Bot explicit enemy_point只用同隊披露，512單位／32中心预算，正常CastAbility target_pos；ranger_volley保留ID改為箭雨，metadata與全內容hash更新。model1／base2／core1／原三原型十二招1、codegen與一真人九Bot設定預檢成功；未stage／OmGame／100場／最後整合，20/30不變，詳見generic-area-damage與E168。

2026-10-05 Bot通用購裝：Lua可選角色item_builds宣告最多六件終局目標，owner背包multiset先保留全部完成目標、遞迴推缺材料，不使用生命期游標；read-only preview重用正式buy_item kernel，正常ItemBuy再經authority admission與settlement。共用shop半徑、基地購裝優先於低血等待，非法ID／角色／無法在六格依宣告順序組裝的配方在compile拒絕；不查敵方背包或直接改經濟。可選return_to_shop金錢／披露威脅門檻，低血撤退優先，正常Recall完整channel後購裝；完成目標不再經濟回城，不保證隱藏敵人無法攻擊。core4與正式60Hz ECS2成功，Lua混合配方預檢成功；100場／最後UE整合仍待完成，20/30不變，見role-bot-item-builds與E167。

2026-10-05 Bot回城續航：Recall原只傳送，新增Lua compiled基地rate／radius＋SingleLaneConfig明確opt-in，authority post-combat active delta／己方活體基地／正HP非lethal生命／range／maxHP結算，真人Bot共用；最終HP走既有filtered投影，不再由Bot或replica補血。可選sustain精確低血／離開／披露威脅距離，正常MoveTo撤退或Recall、讀條不打斷／基地等待。core2、正式60Hz ECS及短雙隊filtered3、compiled/hot-reload1、新混合配方預檢皆成功；尚未stage／完整對局／100場，20/30不變，見bot-sustain-base-recovery與E166。

2026-10-05 三原型內容：共用Lua moba_archetypes builder附加training_vanguard／training_ranger及八招，舊ID不變，與luminary組成近戰前排／遠程物理／法術型；明確只用單體Damage／HealSelf，不冒充控場或mana。模板、FFI與混合role配方共用資料，FFI補相對include／循環與路徑檢查、templates建置監看。ID指定1/1、三英雄十二招正式60Hz输入／精確HP-CD指定1/1、include5/5、新配方一真人九Bot預檢與Unreal生成成功；尚未OmGame編譯／新DLL stage／完整對局與100場，20/30不變，見moba-archetypes進度與E165。

2026-10-05 配方Unreal launcher：固定Lua＋完整TOML section-field replacement＋正式Rust moba-config預檢，獨立run_moba_role_ue從配方真人集合產生server／逐真人runtime／presentation-only UE；Bot只留server，IPC按真人順序分埠，runtime exact player/team ready才啟UE。保留來源、絕對content路徑、明確profile不fallback，owned PID逐次保存／反向stop與wait／errors.md。host2/2、Lua當前功能5/5（真實設定預檢＋mock生命周期）與一真人九Bot60Hz prepare成功；normal Unreal流程尚未執行，三原型／100場／完整驗收仍未完成，見role-unreal-launch進度與E164，20/30不變。

2026-10-05 正式server Bot控制權：Lua匯出server欄位，MATCH_ROLE_PLAN_JSON嚴格驗完整配方，真人AUTH必須等於bot=false集合；Bot不取得KCP授權。main在開局前安裝獨立controllers，外部Bot／未知ID剔除，內部Bot沿既有MobaMatch gate／canonical accepted projection／PendingPlayerInputs／phase dispatcher，team取match controller roster、audience不放寬。正式State::tick60Hz＋delay1／安全兩隊frame與九Bot位移測試、真人Join成功與九BotJoin拒絕、Lua十Bot及一真人九BotTOML設定皆通過，main check成功。新一真人Unreal launcher與完整filtered／socket／技能／100場驗收尚待完成，見role-bot-server-control及E163，5.5仍未勾選。

2026-10-05 Bot技能學習：Lua獨立ability_learning有序ID／rank步驟，最多64、逐技能從1連續且受compiled上限限制；用owner當前loadout／點數／rank／下一級required_hero_level選一個正式UpgradeAbility，不改ECS。blocked步驟跳過並在門檻達成後回補，省略保持舊配方。core learning2＋plan2、正式60Hz rank0學習→重排slot3施法80傷害ECS1與Lua配置入口通過；server Bot控制權需與真人AUTH分離且同正式projection／dispatch接線，未實作、不勾選5.5，詳見role-bot-learning進度與E162。

2026-10-05 Bot技能意圖：effects_preview不是執行語義，採Lua對局配方explicit EnemyUnit／SelfHeal與宣告priority，驗compiled ID／Active-Ultimate Instant／target型別；按當前owner loadout／rank／CD與Fixed64 range只選disclosed living target，走正式CastAbility與權威handler，不改HP／CD或猜slot。core2／正式60Hz ECS1與Lua配置入口成功；Mana／其他技能／網路接線／三原型／100場未完成，5.5仍未勾選，見ability-intents進度與E161。

2026-10-05 Bot角色策略：公開HP／render由同committed baseline完整framing驗證後取用，死亡／非法資料不選取，每隊每think共用一次。Carry優先低HP兵，Support从完整role roster配對同隊Carry（含真人），只依disclosed living位置用正式MoveTo跟隨；停止等待正常Moves→下一Dispatcher完成，近距離在途命令去重避免新pose振盪，不改共用phase。功能core8與ECS3通過，非精準補刀／完整保護／技能或100場驗收，5.5保持未勾選，詳見lane-strategies及E160。

2026-10-05 Bot 配方：共用嚴格 schema1 對局計畫由固定Lua匯出，以compiled map／hero／named lane驗證，排序後生成真人與Bot共用roster／控制指派及profile相依思考間隔。headless opt-in使用正式driver與每步Wave B，保留舊fixture且禁止混用；plan-only不建World／載DLL，不能冒充完整對局。KCP控制權／admission與技能策略尚待接線，5.5不勾選，詳見role-bot-match-plan與E159。

2026-10-05 Bot 增量：新增獨立五位置 opt-in planner 與正式輸入 adapter，保留舊 Push/Guard fixture。只使用同隊 committed disclosure 白名單／公開compiled routes及camp位置，缺／stale snapshot fail closed；local entity_key 不可混用 generation-high canonical ID。10人60Hz正式driver功能確認成功，非production九Bot launcher或100場驗收；策略與技能仍待實作，5.5保持未勾選，詳見role-bot-perception進度及E158。

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

2026-10-05：map可選tower_layers以沿路己方起點距離外→內遞減，未提供保持舊tower_offset單層；最多8、相鄰200與基地距離邊界生成時驗證，規則進map catalog hash。lane_tower_layers保存真實退役，摘要只指第一存活；共用structure_unlocked約束AI與Damage，HP0不能提早解鎖。新增opt-in layered map不改舊預設、不新增前端玩法；完整對局驗收留最後，詳見layered-lane-buildings與E156。

2026-10-05：renderer 的 explicit PresentationIpc intent 優先於 legacy flags，不由 endpoint 是否存在推測 gameplay 模式；缺位址在 create／driver 拒絕，Unreal 不代入 KCP endpoint。新 enum append 保留旧值、CLI／env／settings 共用位址設定；正式launcher宣告presentation-only，runtime simulation 留外部。保留TD與既有有效IPC位址相容，完整單機／LAN驗收留最後，詳見 explicit-presentation-mode 進度檔及E155。

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
2026-10-06 護盾安全投影決定：沿既有team audience的持續owner economy，只增加存活英雄剩餘吸收量Q10 i64，不公開私人Buff資料或從生成初始量推測。定長94bytes／有界decode、IPC owner schema2與configured player gate、C ABI15與原生共用HUD；selective wire6／IPC5明確拒絕舊元件。死亡／到期／耗盡零值、完整缺owner與Stop清除，control-only保留。限定驗證見owner-shield-projection-progress／E266，不代表PIE與統一部署。
