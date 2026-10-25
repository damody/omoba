最新召喚師玩法增量：2026-10-24 mana-restore；共用self_restore_mana與catalog7「澄息」200回魔／180秒CD、D/F既有路徑與原創提示、可選單人魔力配裝。checked ManaPool／plan資源重驗／原子CD，缺池拒絕、滿魔耗CD、禁療獨立，保留HP／控制／移動／引導。FreshRust9套2100／9ignored、focused7、backend／UE5.8／staging／離線plan/catalog、native2／0warnings成功；3488來源一致（3474未變／13變更／1新增），390份證據獨立重核。詳見 docs/plans/2026-10-24-mana-restore-final-acceptance.md。未新增真網路對局／截圖／專用圖示；MVP33/44、11外部項不變。

最新彈體首碰增量：2026-10-24 projectile-contact；移除中點nearest16截取，以同一空間查詢的所有候選及共用整數Q32接觸時間選最早者，同量化按id/generation穩定排序；缺Pos／Creep的舊索引列拒絕。四種索引、密集離軸目標、入口／相切／raw邊界與翻轉順序通過，無新全世界掃描。FreshRust九套2093／9ignored、focused11、backend／UE5.8／staging／catalog、native cue2／0warnings成功；3487來源一致（3484未變／3變更），151份證據獨立重核。詳見 docs/plans/2026-10-24-projectile-contact-final-acceptance.md。維持creep碰撞政策，沒有新增技能／截圖／真網路對局；MVP33/44、11外部項不變。

最新共用彈體修正：2026-10-24 projectile-flight；移動／掃掠共用剩餘壽命與整數端點，修正最後tick越過終點、過量壽命移動及不足1單位／raw1零速提前命中。既有瞄準端點沿同一整數機制且保留原限制，沒有名稱特判。FreshRust九套2082通過／9ignored、focused5、backend／UE5.8／staging／catalog、native cue2／0warnings通過；3487來源一致（3485未變／2變更），157份證據獨立重核。原始失敗與退休保留；詳見 docs/plans/2026-10-24-projectile-flight-final-acceptance.md。沒有新增技能／截圖／真網路對局；更多飛行技能／MOBA skillshot碰撞與MVP33/44、11外部項仍待。

最新召喚物 UE 操作增量：2026-10-24 summon-ui；exact owned living summon 選取、Alt+右鍵指定移動／Alt+S恢復自主、可保存遷移快捷鍵與實際綁定提示。獨立型別輸入保留主人＋召喚物 opaque life，client／bridge 目前來源守門；presentation12／selective14／ABI36。active UE 橋接建置改走固定Lua。FreshRust1999/0失敗/9ignored、focused4、backend／UE5.8／DLL-import／catalog、native9/0warnings及實際Slate截圖目視通過。3487來源一致（3457未變／25變更／5新增），1269份證據獨立重核。UI01–UI13失敗與12工作原始exit／退休保留。詳見 docs/plans/2026-10-24-summon-ui-final-acceptance.md。未新增真網路對局；MVP33/44與11外部項不變。

# 換機交接：MOBA／Unreal 前端

更新日期：2026-10-22（Asia/Taipei）。新電腦請先讀本文件，再讀 `AGENTS.md`。2026-10-18交接時將主倉庫、後端 `omb`、Unreal `omfue` 與其內層 `BpGeneratorUltimate` 提交至各自遠端 `master`；下表是該次基準。本輪新機實作與驗收尚未另行commit／push。

| 倉庫 | 前次交接基準提交 |
|---|---|
| `omb` | `f043858bd0d88d0f79d05d469b2bb224410aea99` |
| `omfue` | `44890d4d7222eea5f90c9a1e656e98a81294e9d0` |
| `omfue/Plugins/BpGeneratorUltimate` | `dfdbd34dabf1b70fd4c221baffeec0bb61942dd4` |

主倉庫版本請執行`git rev-parse HEAD`，避免在文件中寫入自身commit造成循環。

## 目前狀態

完整 MVP **尚未完成**。OpenSpec `complete-moba-controls-and-presentation` 共44項，**33完成／11待驗**；規格 ready 不是功能全數完成。詳細清單在 `openspec/changes/complete-moba-controls-and-presentation/tasks.md`，功能核對在 `docs/plans/2026-10-18-mvp-functional-task-audit.md`。

| 已接線的範圍 | 內容與接受界線 |
|---|---|
| 鏡頭／小地圖 | 左鍵定位與拖曳、滾輪、邊緣平移、Space／Y、隊友focus、視窗框及小地圖偏好；實際owned viewport事件已驗。 |
| 操作設定 | 可保存／遷移／恢復的快捷鍵、精確修飾鍵、A確認瞄準、QWER三施法模式、升級／自用、六格道具、D/F、回城；WASD、游標鍵與搖桿映射沿共用命令。搖桿使用合成軸，非實體控制器接受。 |
| 權威契約 | Rust側Stop／Hold、攻擊偏好、輸入模式、召喚師、ward、商店receipt／undo及隊伍溝通的共享資料／驗證已接線；需要新真對局ACK的父項仍待驗，不能只因按鈕存在就勾完。 |
| HUD／商店 | 七位正式英雄頭像、slot圖示／rank／冷卻／charges／tooltip、隊友卡／計分板／屬性／FPS與RTT、商店搜尋／分類／recipe／receipt、死亡／回城／結算與輸入focus。 |
| 溝通 | team／all chat、八語意ping／輪盤、快捷訊息、表情、標記、mute與公開資料隔離。 |
| 場景／角色 | 兩張公開三路map的地面、lane／river／jungle、阻擋輪廓、塔／基地、角色衣裝與裝備、共享team／selection標記、camera fit與光照生命期。6.2／6.4／6.6已完成本機驗收；真對局／LAN／OS DPI仍待驗。 |
| 動畫／聲音 | 公開cue／HP／回城狀態驅動的原生動畫、池化VFX；原創UI／戰鬥／ambient音效，四段CC0英文播報，獨立master／music／effects／voice設定與字幕。 |
| 工具 | Lua 5.4工作流、擁有權程序清理、正式Engine bootstrap／NoEngineChanges建置、native／Blueprint驗收、實際submix PCM，以及可選的owned視窗MJPEG AVI錄影。 |
| BpGeneratorUltimate | UE5.8相容、MCP／PIE與工具修正；移除啟動時自動歡迎／新聞彈窗，手動入口保留。 |

## 2026-10-22 永久史詩累積與多項獎勵（最新）

OpenSpec complete-moba-permanent-objectives **5/5完成**。Lua額外隊伍Buff獎勵最多4項／缺欄空、單營地重複ID與每scope32政策預算／scope一致性守門；隊伍duration0為永久，死亡保留且不可傳承。南史詩原120秒護甲維持，另給每層5攻擊／最多4層永久傳承。權威expires=None、BuffStore canonical MAX明確不倒數，跨死亡／重生保存，結束／新局清除，digest完整；沿既有snapshot -1／ABI33永久flag／Unreal持續生效。

RustV1 **1754通過／10既有ignored**、無失敗重試；UE5.8成功（UBT **45.23秒**）、後端與Lua三契約、World37成功／4既有清理warnings。正式60Hz三層／永久snapshot、1,000,000秒後保存、匿名死亡＋native DeathTick後重生保留通過。**3353來源一致／44份證據**封存獨立複核，三張PNG已目視；三分鐘監測7次／最大181秒，worker／watcher全退出。前批退休守門保留，未重犯EXE鎖定／HP0與退休邊界。

剩餘小兵光環／野怪分裂、經濟差補償、公告、持續引導／特殊幾何／動畫、召喚複本／回滾、塔皮與美術。MVP33/44／11外部待驗維持。詳[驗收](docs/plans/2026-10-22-permanent-objectives-final-acceptance.md)、[錯誤P01](docs/plans/2026-10-22-permanent-objectives-errors.md)、[盤點](docs/plans/2026-10-22-permanent-objectives-audit.md)。

## 2026-10-22 個人野區Buff與擊殺傳承（前次）

OpenSpec complete-moba-buff-transfer **5/5完成**。Lua personal_buff／transfer_on_kill兼容預設與界限、同map政策／scope一致性、hash與重建守門；四個營地原創120秒攻擊／護甲Buff，個人lease與史詩隊伍lease分開。有效敵對擊殺結算後只將剩餘期限傳給活著的擊殺者，同類取較晚期限、不刷新完整duration。匿名／強制死亡、同生命重複、同隊／pause／非Playing／死去接收者均閉合，死亡／重生／到期／結束與digest完整。既有Buff／屬性到Unreal沿用ABI33。

RustV2 **1747通過／10既有ignored**；UE5.8 V2成功（UBT **3.21秒**）、後端與Lua三契約、World37成功／4既有清理warnings。正式60Hz技能取得、傳承約60秒／modifier10、pause與到期、匿名傷害＋native DeathTick清理成功。**3353來源一致／52份證據**封存獨立複核，三張PNG已目視；三分鐘監測9次／最大181秒，worker／watcher全退出。V1錯誤T01為HP0／退休生命期邊界；V2依檔名推導前批receipt並等待owner／watcher完全退休，未重犯測試EXE重疊鎖定。

剩餘永久stack／光環／分裂、經濟差補償、公告、持續引導／特殊幾何／動畫、召喚複本／回滾、塔皮與美術。MVP33/44／11外部待驗維持。詳[驗收](docs/plans/2026-10-22-buff-transfer-final-acceptance.md)、[錯誤T01](docs/plans/2026-10-22-buff-transfer-errors.md)、[盤點](docs/plans/2026-10-22-buff-transfer-audit.md)。

## 2026-10-22 史詩目標隊伍Buff疊層（前次）

OpenSpec complete-moba-objective-stacks **5/5完成**。Lua max_stacks缺欄1、1..16界限、hash／compiled catalog與hot reload重建守門；南史詩最多4層、每層15護甲／120秒共享刷新，北史詩單層維持。權威lease保存層數、同隊同政策未到期疊加封頂、到期／政策變更重設，單一BuffStore entry套用整數倍效果；跨死亡資格／重生／pause／結束清理及digest完整。既有Buff／戰鬥屬性到Unreal沿用ABI33，層數專用UI及永久stack／傳承另待實作。

RustV3 **1740通過／10既有ignored**；V2因前批測試EXE占用而LNK1104，V3加入receipt與owner／watcher完全退休守門後串行重跑，失敗證據封存；後續同target批次必須沿此守門，不可重疊重建測試EXE。UE5.8 V2成功（UBT **3.19秒**）、後端與Lua三契約成功、World37成功／4既有清理warnings。五次真實營地死亡重生封頂四層、同生命去重與生命期回歸成功；正式60Hz三層45護甲／payload、pause與到期清理通過。**3352來源一致／57份證據**封存獨立複核、三張PNG已目視；三分鐘監測8次／最大181秒，worker／watcher全退休。

剩餘永久stack／傳承／光環／分裂、經濟差補償、更多公告、持續引導／特殊幾何／動畫、召喚複本／回滾／退休callback、塔皮與美術。MVP33/44／11外部待驗維持。詳[驗收](docs/plans/2026-10-22-objective-stacks-final-acceptance.md)、[錯誤S01–S05](docs/plans/2026-10-22-objective-stacks-errors.md)、[盤點](docs/plans/2026-10-22-objective-stacks-audit.md)。

## 2026-10-22 連敗擊殺金折價（前次）

OpenSpec complete-moba-death-streak-rewards **5/5完成**。Lua整數basis-point曲線／缺欄停用／界限與hash及重建保護；正式server採generated政策。權威敵對擊殺入帳後受害者連敗加一、擊殺者重設，跨重生保存，同生命去重；匿名／強制死亡不能刷折價。只折普通基礎金，首殺／終結／助攻／XP維持，沿既有來源歸因／死亡錢包及Unreal owner Gold，ABI33維持；digest涵蓋政策與狀態。

最終RustV2 **1734通過／10既有ignored**；UE5.8 V2建置成功（UBT **3.19秒**）、後端與Lua三契約成功、World37成功／4既有清理warnings。正式60Hz技能100＋150＋100＝350金、不重付與pause成功；九次實際擊殺跨重生折價／擊殺重設通過。**3352來源一致／44份證據**封存獨立複核，三張PNG已目視；三分鐘監測7次／最大181秒，worker／watcher全退休。

後續全隊經濟差補償、更多戰鬥／建築／史詩公告、持續引導／施法動畫／特殊幾何、召喚複本／整組回滾／退休callback、史詩stack／傳承／光環／分裂、塔皮與美術。MVP33/44／11外部待驗維持。詳[驗收](docs/plans/2026-10-22-death-streak-final-acceptance.md)、[錯誤D01–D03](docs/plans/2026-10-22-death-streak-errors.md)、[盤點](docs/plans/2026-10-22-death-streak-audit.md)。

## 2026-10-22 首殺額外獎勵與權威公告（前次）

OpenSpec complete-moba-first-kill **5/5完成**。Lua首殺金100／缺欄0關閉／界限與hash／重建保護；有效敵對英雄擊殺後收集候選，按(tick,完整victim canonical identity)唯一選定並finish支付。既有玩家／召喚／投射物歸因與死亡錢包共用，匿名／強制死亡不消耗。首殺與終結金可同領，pause凍結，公告五秒隨authority elapsed到期，里程碑不重設，digest涵蓋。公開三metric→typed proto→ABI33／12-byte notice→Unreal資訊列，缺欄／非法／非對手資料閉合。

最終RustV1 **1729通過／10既有ignored**（template63／core868／base257／bridge129／client150），含generation九工作exit0／退休。UE5.8建置成功（UBT **60.75秒**）、後端與Lua三契約成功；World37成功，4既有清理警告。正式60Hz技能550金同領、不重付、pause360step與resume到期通過。**3351來源一致／44份證據**逐byte／SHA封存獨立複核，三張原生輸出已目視；三分鐘監測9次／最大181秒，7 workflow worker＋7 watcher及全程monitor退出。

後續連敗折價／全隊經濟差補償、更完整擊殺／建築／史詩公告與聲音、持續引導／施法動畫／特殊幾何、完整召喚複本／整組回滾／退休callback、史詩stack／傳承／光環／分裂、塔皮與美術。原MVP33/44／11外部待驗維持，未新增網路對局／重設配額／commit／push，omfx不維護。詳[最終驗收](docs/plans/2026-10-22-first-kill-final-acceptance.md)、[錯誤F01–F03](docs/plans/2026-10-22-first-kill-errors.md)、[盤點](docs/plans/2026-10-22-first-kill-audit.md)。

## 2026-10-22 連殺終結賞金與公開計分板（前次）

OpenSpec complete-moba-shutdown-bounty **5/5完成**。Lua四欄政策／bounded validation／hash與重建保護，正式server選用generated政策；player roster連殺、實際敵對英雄致死一次獎勵、任何死亡清除、召喚／退休投射物歸因與死亡錢包，digest完整。公開score→typed proto→ABI32／24-byte row→Unreal賞金文字；舊snapshot缺欄為零、非法值閉合，助攻金不變。

最終RustV4 **1721通過／10既有ignored**（template62／core864／base256／bridge128／client149），含generation九工作exit0／退休。V1新測試生命期／cap期望修正後等整批退出才重跑，原log保留；V2另找到通用死亡多發10金，正式修正並重建V3後通過。UE5.8建置成功（UBT **3.94秒**）、後端與Lua三契約成功；World36成功，4個既有清理警告。**3350來源一致／137份證據**逐byte／SHA封存獨立複核，三張原生輸出已目視；監測18次／最大181秒，21 workflow worker＋21 watcher及全程monitor退出。

後續連敗折價／首殺／經濟差補償與公告、持續引導／施法動畫／特殊幾何、完整召喚複本／整組回滾／退休callback、史詩stack／傳承／光環／分裂、塔皮及美術。原MVP33/44／11外部待驗維持，沒有新增網路對局／重設配額／commit／push，omfx不維護。詳[最終驗收](docs/plans/2026-10-22-shutdown-bounty-final-acceptance.md)、[錯誤B01–B08](docs/plans/2026-10-22-shutdown-bounty-errors.md)、[盤點](docs/plans/2026-10-22-shutdown-bounty-audit.md)。

## 2026-10-22 權威施法進度、取消與結果HUD（前次）

OpenSpec complete-moba-cast-progress **5/5完成**。正式MOBA保存Preparing／Canceled／Released、remaining／total／instance，成功由真實交易決定；Team HUD五metric→client typed protobuf→ABI31／40-byte ability→Unreal技能欄進度與短暫文字。拒絕缺欄／重複／非法tuple，同instance防時間倒退、終態覆寫與復活，死亡／空frame清理，UI不自行完成施法。

最終RustV2 **1713通過／10既有ignored**（core859／base255／bridge128／client148），含generation九工作exit0／退休。UE5.8建置成功（UBT **86.25秒**）、後端建置與Lua三契約成功；World **35成功**（34 Success＋1 SuccessWithWarnings／4既有清理警告）。**3349來源一致／66份證據**逐byte／SHA封存獨立複核；三分鐘監測8次／最大181秒，10 workflow worker＋10 watcher及全程monitor退出。

原MVP33/44／11外部待驗維持。後續真實連線動態畫面／角色動畫、持續引導與中斷政策、特殊幾何、完整召喚複本／整組回滾／退休callback、史詩stack／傳承／光環／分裂、塔皮／公告與美術。未新增網路對局／重設配額／commit／push，omfx不維護。詳[最終驗收](docs/plans/2026-10-22-cast-progress-final-acceptance.md)、[錯誤P01–P04](docs/plans/2026-10-22-cast-progress-errors.md)、[盤點](docs/plans/2026-10-22-cast-progress-audit.md)。

## 2026-10-22 正式技能前搖、取消與提示（前次）

OpenSpec complete-moba-cast-windup **5/5完成**。正式MOBA依cast_time固定點準備，來源／目標完整generation、rank／起點與remaining／last_tick；不提早扣款／冷卻／發成功cue，到期再走完整交易。移動／立即命令／死亡／silence／stun／recall／非playing取消；queued等待，pause凍結，自主普攻停，digest涵蓋。UE tooltip接generated前搖秒數與取消規則，instant清除舊提示，動態進度／持續引導仍未完成。

最終RustV2 **1709通過／10既有ignored**（core858／base254），9工作含generationexit0／退休；V1新ManaPool fixture失敗log保留，補合法pool，最終等整批V1／watcher退出後開始。UE5.8 build-v2／後端建置成功（UBT **3.50秒**）；Lua三契約、World **34成功**（33 Success＋1 SuccessWithWarnings／4既有警告），新增CastWindupTooltip通過。**3349來源一致／46份證據**逐byte／SHA封存獨立複核；監測8次／最大181秒，9 workflow worker＋9 watcher及全程monitor退出。

原MVP33/44／11外部待驗維持。後續動態施法進度／中斷政策／持續引導、非法cast意圖與命令規則、特殊幾何、完整召喚複本／競爭落點整組回滾、退休callback、史詩stack／傳承／光環／分裂、塔皮／公告與美術。未新增網路對局／重設配額／commit／push，omfx不維護。詳[最終驗收](docs/plans/2026-10-22-cast-windup-final-acceptance.md)、[錯誤W01–W08](docs/plans/2026-10-22-cast-windup-errors.md)、[盤點](docs/plans/2026-10-22-cast-windup-audit.md)。

## 2026-10-22 召喚落點、跟隨與導航（前次）

OpenSpec complete-moba-summon-navigation **5/5 完成**。Lua／generated／runtime 接跟隨啟動／停止距離，雜賀 750／350；腳本接納與原生生成都用公開地形、友軍體積與同批占位找合法落點。明確 MoveTarget 優先，否則主人離開後沿共用有界導航返回，waypoint 快取 250ms、每步 swept-circle；腳本追擊同用公開地形導航。主人死亡／generation 失效取消自主返回，pause／root／buff 移速／期限與 digest 接線保留。

RustV1 **1706 通過／10 既有 ignored**（template61／core856／base253），9 工作含 generation 均 exit0／退休；正式技能牆內要求陣形、實際合法生成、返回與雙次重播通過。UE5.8／後端建置成功，UBT **3.52 秒**；Lua 三契約與 World33 成功（32 Success＋1 SuccessWithWarnings／4 既有警告）。**3348 來源一致、41 份證據**逐 byte／SHA 封存獨立複核，監測 6 次／最大 181 秒，7 workflow worker＋7 watcher 及全程 monitor 退出。

原 MVP 33/44／11 外部待驗維持。後續完整召喚複本生命期、更多控制／隊形、競爭落點整組原子回滾、退休 callback、施法排程／中斷、特殊幾何、史詩 stack／傳承／光環／分裂、塔皮／公告與美術。未新增網路對局／重設配額／commit／push，omfx 不維護。詳[最終驗收](docs/plans/2026-10-22-summon-nav-final-acceptance.md)、[錯誤 N01–N03](docs/plans/2026-10-22-summon-nav-errors.md)、[盤點](docs/plans/2026-10-22-summon-nav-audit.md)。

## 2026-10-22 投射物來源與跨生命期獎勵（前次）

OpenSpec complete-moba-projectile-source-credit **5/5 完成**。三個正式投射物工廠保存每發來源完整 generation／發射位置／隊伍／玩家，命中傷害 scope 使用後清除；英雄／小兵／野怪／建築共用歸因，來源退休或玩家死亡／重生仍正確領取。舊彈不因來源改歸屬或 id 重用而轉帳；digest 與清理完整接線。TD 退休來源賞金／腳本 on-hit callback 重建仍不在本輪範圍。

最終 RustV1 **1700 通過／10 既有 ignored**（core852／base252），9 工作含 generation 均 exit0／退休；實際技能槍手過期後命中仍有擊殺金。UE5.8／後端建置成功，UBT **3.04 秒**；Lua 三契約與 World **33 成功**（32 Success＋1 SuccessWithWarnings／4 既有警告）。**3347 來源／資產**前後一致，DLL／import library 另驗。**41 份證據**逐 byte／SHA 封存獨立複核；監測 7 次／最大 181 秒，7 workflow worker＋7 watcher 及全程 monitor 退休。

原 MVP 仍 33/44／11 外部待驗。接續通用施法排程／中斷、召喚避障／跟隨／完整複本生命期、退休來源 callback、史詩 stack／傳承／光環／分裂、特殊幾何、塔皮／公告與美術。沒有新增網路對局／重設配額／commit／push，omfx 不維護。詳[最終驗收](docs/plans/2026-10-22-projectile-credit-final-acceptance.md)、[錯誤 P01–P04](docs/plans/2026-10-22-projectile-credit-errors.md)、[盤點](docs/plans/2026-10-22-projectile-credit-audit.md)。

## 2026-10-22 召喚視野、索敵與控制（前次）

OpenSpec complete-moba-summon-combat-visibility **5/5完成**。內容vision_radius接正式VisionSource；召喚nearest／range共用committed disclosure、存活／敵對／LOS、結構／野怪與ward auto政策。暈眩停自主hook，定身保留普攻／禁移動，沉默不阻普攻；追擊／命令移速與攻擊屬性共用UnitStats。援軍普通攻擊接正式phase cue／草叢開火揭露。

最終RustV2 **1695通過／10既有ignored**（core848／base251），9工作含generation均exit0／退休；Lua三契約成功。UE5.8／後端建置成功，UBT **3.24秒**；World **33成功**（32 Success＋1 SuccessWithWarnings／4既有preview-world警告）。**3346來源／資產**接受前後一致，DLL／import library另驗。**46份原始證據**逐byte／SHA封存獨立複核；三分鐘監測8次／最大181秒，9個workflow worker與9個watcher退休，全程monitor退出。

原MVP仍33/44／11外部待驗。接續通用施法排程／中斷、退休投射物來源、召喚落點／跟隨／控制與完整複本生命期、史詩stack／傳承／光環／分裂、特殊幾何及美術。未新增連線對局／重設配額／commit／push，omfx不維護，209／458／69／五PCM未重跑不混算。詳[最終驗收](docs/plans/2026-10-22-summon-combat-final-acceptance.md)、[錯誤C01–C06](docs/plans/2026-10-22-summon-combat-errors.md)、[盤點](docs/plans/2026-10-22-summon-combat-audit.md)。

## 2026-10-22 通用召喚生成與生命期（前次）

OpenSpec complete-moba-summon-spawning **5/5完成**。原INVALID入口已實作：內容數值、真實handle／deferred生成、主人與隊伍、確定性平行hook生成順序與失敗回收；固定點期限／pause／永久、命令碰撞移動與正式退休。索敵英雄／野怪／單位及LOS，實際60Hz技能已生成援軍、發射投射物並造成傷害。

最終RustV3 **1692通過／10既有ignored**（core846／base250），9工作含generation均exit0／退休；Lua三契約成功。UE5.8 build-v2／backend-v2成功，UBT **3.65秒**；WorldV2 **33成功**（32 Success＋1 SuccessWithWarnings／4既有preview-world警告）。**3346來源／資產**接受前後一致；DLL／import library另驗。**77份原始證據**逐byte／SHA封存並獨立複核；三分鐘監測11次、最大181秒，21個workflow worker與21個watcher退休，全程monitor退出。

原MVP仍33/44／11外部待驗。接續cast scheduling／中斷、退休投射物來源、召喚落點／跟隨／控制與複本接受、史詩stack／傳承／光環／分裂、特殊幾何與美術。未新增連線對局／重設配額／commit／push，omfx不維護；209／458／69／五PCM未重跑不混算。詳[最終驗收](docs/plans/2026-10-22-summon-spawn-final-acceptance.md)、[錯誤S01–S12](docs/plans/2026-10-22-summon-spawn-errors.md)、[盤點](docs/plans/2026-10-22-summon-spawn-audit.md)。

## 2026-10-22 野怪非英雄來源歸因與仇恨（前次）

OpenSpec complete-moba-jungle-source-credit **5/5完成**。來源PlayerOwner／有界generation-aware召喚鏈歸因，first lethal保存player／side；個人獎勵與team-only objective分開，死亡錢包／來源退休／重複保護。非英雄可實際吸引camp並被攻擊，退休回頭，LOS原規則保留。

Rust **1687通過／10既有ignored**：template60、core842、codegen57、base249、backend190/1、bridge127/1、client147/8、ABI15，另generation成功；9工作均exit0／退休。Lua三契約成功。UE5.8 build-v1與backend-v1成功，UBT **3.43秒**；World **33成功**（31 Success＋2 SuccessWithWarnings／5 warnings，0 Failed／NotRun），4筆preview-world清理與1筆背景HTTP timeout保留。

**3346來源／資產**的集合、SHA、size、mtime接受前後一致；DLL與import library依current build／接受前staging另驗。**39份原始證據**逐byte與SHA封存、獨立複核。三分鐘監測7次，UTC最大間隔181秒；8個owned worker及8個watcher均退休，全程health monitor已退出。

**優先缺口：spawn_summoned_unit仍是INVALID stub，召喚技能尚未完成生成。** 本輪ownership fixture／credit不代表召喚技能成功；接續生成／ownership／duration／退休pipeline與已退休投射物來源快照。原MVP文件33/44／11外部待驗不是全部技能已驗。未新增連線對局／重設配額／commit／push，omfx不維護；前輪209／458／69／五PCM不混算。詳[最終驗收](docs/plans/2026-10-21-jungle-credit-final-acceptance.md)、[錯誤J01–J07](docs/plans/2026-10-21-jungle-credit-errors.md)、[盤點](docs/plans/2026-10-21-whole-game-completeness-audit.md)。

## 2026-10-21 第一塔與分層建築獎勵（前次）

OpenSpec add-moba-first-tower-reward **5/5完成**。首塔額外金與地圖持久depth獎勵共用正式building wallet；最早tick／同tick完整entity定序，匿名／強制consume，pause保留、死亡資格與重生金保留、一次提交／digest。60Hz生成技能同step OwnerEconomy已含500金。

Rust **1681通過／10既有ignored**：template60、core837、codegen57、base248、backend190/1、bridge127/1、client147/8、ABI15；另generation成功，9工作均exit0／退休。Lua三契約成功。UE5.8 build-v1／backend-v1成功，UBT **3.35秒**。World **33成功**（32 Success＋1 SuccessWithWarnings／4 warnings，0 Failed／NotRun）。

**3345來源／資產**的集合、SHA、size、mtime接受前後一致；DLL與import library依current build／接受前staging另驗。**38份原始證據**逐byte及SHA封存、獨立複核。三分鐘監測5次，UTC最大間隔181秒；7個owned worker與7個watcher均退休，全程health monitor已退出。

本輪33項World回歸，前輪209／458／69／五PCM未重跑不混算。原MVP仍33/44／11外部待驗；沒有新增連線對局／重設配額／commit／push，omfx不維護。後續cast scheduling／interrupt、objective stack／傳承／小兵光環／非英雄野怪歸因、分裂、塔皮／公告與特殊技能幾何。詳[最終驗收](docs/plans/2026-10-21-first-tower-final-acceptance.md)、[錯誤T01–T02](docs/plans/2026-10-21-first-tower-errors.md)、[盤點](docs/plans/2026-10-21-whole-game-completeness-audit.md)。

## 2026-10-21 史詩目標隊伍增益（前次）

OpenSpec add-moba-objective-team-buffs **6/6完成**。營地資料指定Buff政策，主體正式Death歸因後授予擊殺隊伍，兩張map共用。BuffStore／stats／snapshot／原生catalog接線；refresh不疊加，死亡失去或保留／重生原期限、pause／過期／terminal／digest。

Rust **1676通過／10既有ignored**：template60、core832、codegen57、base248、backend190/1、bridge127/1、client147/8、ABI15，另generation成功；9工作均exit0／退休。Lua三契約成功。UE5.8 build-v1、backend-v1成功，UBT **51.42秒**；World **33成功**（32 Success＋1 SuccessWithWarnings／4既有preview-world清理警告，0 Failed／NotRun）。

**3344來源／資產**的集合、SHA、size、mtime接受前後一致；DLL與import library另依接受前staging／current build核對。**39份原始證據**逐byte及SHA封存並獨立複核；三分鐘監測8次，UTC最大間隔181秒；8個owned worker及8個watcher均退休，全程health monitor也已退出。

ECS與60Hz生成技能／snapshot已驗，33項含新增catalog／Buff清單／lifecycle；場景像素不是真對局增益hover／ACK。前輪209／458／69／五PCM未重跑不混算。原MVP仍33/44、11外部待驗；未新增連線對局／重設配額／commit／push，omfx不維護。後續cast scheduling／interrupt、永久stack／傳承／小兵光環、非英雄來源歸因、分裂與特殊技能幾何。詳[最終驗收](docs/plans/2026-10-21-objective-buffs-final-acceptance.md)、[錯誤O01–O04](docs/plans/2026-10-21-objective-buffs-errors.md)、[盤點](docs/plans/2026-10-21-whole-game-completeness-audit.md)。

## 2026-10-21 建築獎勵與兵線成長（前次）

OpenSpec add-moba-building-rewards **6/6完成**。正式人類／role bot plan啟用建築隊伍／附近摧毀獎勵與生成時兵線成長；各自有界、legacy default停用、setup原子驗證及digest。建築每生命首次致死一次，含NPC來源、死亡錢包、local floor share與重生；新兵依pause-aware時間成長，舊兵不變。

Rust **1670通過／10既有ignored**：template59、core828、codegen57、base247、backend190/1、bridge127/1、client147/8、ABI15；另codegen generation成功，9工作均exit0／退休。Lua三契約成功。UE5.8正式build-v1與backend-v1成功，UBT **3.39秒**。World **30項成功**（29 Success＋1 SuccessWithWarnings／4既有preview-world警告，0 Failed／NotRun），兩map各74影格／6片段。

接受前後 **3343來源／資產**的集合、SHA、size與mtime一致；DLL／import library依current build與接受前staging另驗。**41份原始證據檔**逐byte與SHA封存後獨立複核，含全部Rust log、建置、receipt、盤點與錯誤MD。三分鐘監測8次，UTC最大間隔181秒；10個owned worker與10個watcher均退休，另本輪全程health monitor已退出。

本輪未重新跑209 native／458 IPC／69 Blueprint／五PCM，不混用前輪數字；原MVP仍33/44、11外部待驗，沒有新增連線對局／重設配額／commit／push。後續通用cast scheduling／interrupt、epic buff／split、塔皮／第一塔與特殊技能幾何。詳[最終驗收](docs/plans/2026-10-21-building-rewards-final-acceptance.md)、[錯誤B01–B06](docs/plans/2026-10-21-building-rewards-errors.md)、[更新盤點](docs/plans/2026-10-21-whole-game-completeness-audit.md)。

## 2026-10-21 全遊戲盤點、塔持續攻擊與冷卻明細（前次）

OpenSpec add-moba-tower-focus-and-cast-timing **5/5完成**。完成[17範圍盤點](docs/plans/2026-10-21-whole-game-completeness-audit.md)。一般塔／主堡塔focus第一擊base、同英雄後續+25%至+100%，3秒timeout，可配置；換目標／轉小兵／失去目標／死亡／退休重置，pause凍結，policy／state入digest不入wire。Tooltip新增實際冷卻剩餘／總長；cast_time未有通用權威排程，撤回準備顯示並列優先缺口。

Rust **1659通過／10既有ignored**（core818）；World **30項**成功；完整local-v1 **209 native、458案例／222 intents／157 stages／13 cursor、69 Blueprint、五段實際PCM**成功。native為201 Success＋8 SuccessWithWarnings／12 warnings；Blueprint 0 compiler errors／0 warnings／0 failed loads。

**3341來源／資產**接受前後集合、SHA、size、mtime一致，derived DLL及import library另依current build／接受前staging核對。**20份原始JSON＋1份盤點MD**逐byte封存並獨立複核；完整流程8次健康取樣，180秒排程UTC最大181秒，另持續讀最新log；**13個owned工作與13個watcher全退役**。

RustV4：template59、core818、codegen57、base246、backend190/1、bridge127/1、client147/8、ABI15；9工作含生成均exit0並退休。Lua local69／owned140及map ID契約成功。UE5.8 build-v2成功，UBT61.65秒，backend-v1成功。World為28 Success＋2 SuccessWithWarnings／5 warnings，0 Failed／NotRun；兩map各74影格／6片段，背景HTTP timeout警告保留。

已目視實際tooltip的3.5／12秒冷卻，非真對局hover；塔行為以ECS驗證。詳[最終驗收](docs/plans/2026-10-21-tower-focus-final-acceptance.md)、[錯誤F01–F08](docs/plans/2026-10-21-tower-focus-errors.md)、[封存](docs/acceptance/2026-10-21-tower-focus/index.json)。未新增連線對局／重設配額／commit／push，omfx不維護，舊MVP仍33/44／11外部待驗；後續補通用施法排程、建築獎勵、史詩怪效果／分裂與特殊技能幾何。

## 2026-10-21 技能魔力提示與野怪牆體出手（前次）

`add-moba-cast-resource-guidance` **5/5完成**。目前公開rank ManaCosts接HUD與tooltip；owned有效mana瞄準提示不足，零cost／未知分開，未學習／替換／source移除／退休清理，提示不攔截submit。所有camp主體與附屬怪共用正式LOS，被牆擋住不扣攻擊週期、不產生damage／cue／brush reveal，沿原導航追擊，視線允許後恢復出手。

最終RustV2 **1654通過／10既有ignored**（core813）、Lua69＋140／ID契約、build-v2 UBT6.70秒、backend-v1及WorldV1 **30項**成功，兩map各74影格／6片段。完整local-v1成功：**209 native、458案例／222 intents／157 stages／13 cursor、69 Blueprint、五段實際PCM**；native 201 Success＋8 SuccessWithWarnings／12 warnings，Blueprint0 error／0 warning。實際SOmHudToolTip元件截圖已目視魔力37.5、距離／半徑與一般施法文字可讀，非真對局hover。

**3340來源／資產**接受前後一致，current build／接受前staging／import library SHA另驗。**20份原始JSON**逐byte封存並獨立複核；8次完整流程健康取樣，180秒排程UTC最大180秒，另持續讀log；11個owned工作與11個watcher全退役。詳見[最終驗收](docs/plans/2026-10-21-cast-resource-final-acceptance.md)、[錯誤M01–M05](docs/plans/2026-10-21-cast-resource-errors.md)、[封存](docs/acceptance/2026-10-21-cast-resource/index.json)。失敗RustV1／build-v1與舊capture保留。

未新增權威對局／重設配額，未commit／push，omfx不維護。舊MVP仍33/44，11項外部接受保留。後續可做特殊技能形狀／metadata、史詩怪增益／分裂、多邊形草叢與專用美術。

## 2026-10-21 連通草叢與施法操作提示（前次）

`connect-moba-brush-regions` **5/5完成**。精確膠囊連通圖支援直接／傳遞連通、切線與轉彎，不因AABB角落誤接；載入時計算、immutable Arc共用至公開視野、即時targeting與fog，熱路徑不建圖。兩張現有map各六段草叢，新增鏡像分支，地表與隱蔽同源。技能tooltip讀取目前控制profile，顯示三施法模式的按下／確認／放開時機、圖示點擊行為及改鍵／未綁定，保留原目標／距離／半徑。

最終RustV3 **1653通過／10既有ignored**（core812），Lua69＋140／ID契約、build-v2 UBT23.86秒、backend-v1與WorldV1 **30項**成功；兩map各74影格／6片段。完整local-v1成功：**209 native、458案例／222 intents／157 stages／13 cursor、69 Blueprint、五段實際PCM**。native 201 Success＋8 SuccessWithWarnings／12 warnings；Blueprint0 compiler error／failed load。

**3340來源／資產**接受前後一致；derived DLL與import library另依current build／接受前staging SHA核對。**20份原始JSON**逐byte封存並獨立複核，8次完整流程健康取樣，180秒排程UTC最大181秒，另持續讀log；12個owned工作與12個watcher全部退休。已目視連通地表、fog、camp與原始影片既有施法預覽文字；新增tooltip內容以native接受，錄影未取得展開狀態。詳見[最終驗收](docs/plans/2026-10-21-connected-brush-final-acceptance.md)、[錯誤C01–C09](docs/plans/2026-10-21-connected-brush-errors.md)、[封存](docs/acceptance/2026-10-21-connected-brush/index.json)。過期／失敗cohort保留，不冒稱零警告或單session效能。

未新增權威對局／重設配額，未commit／push，omfx不維護。舊MVP仍33/44；11項外部接受保留。後續可做任意多邊形草叢authoring、特殊技能精確形狀／metadata、史詩怪增益／分裂與專用美術。

## 2026-10-21 草叢普攻揭露與技能tooltip（前次）

`add-moba-brush-attack-reveal` **6/6完成**。草叢內已成立的普攻對受擊非中立隊伍揭露攻擊者兩秒模擬時間；刷新延長、暫停凍結，死亡／退休／換map清理，期限納入digest。原生、script、lane AI及中立野怪／附屬怪共用同一規則；不以ForceShow繞過range／LOS／stealth／ServerOnly／ForceHide，其他草叢 occupants保持隱藏。Fog僅亮已合法公開攻擊者所在格，不建立VisionSource。

技能tooltip新增公開目標類型、ally／enemy需求、距離及point落點半徑；依WorldUnitsToCm換算，未知／未學習／非法／替換／退休不留舊數值，保留說明與legacy label。原生Om.Runtime.AbilityTargetTooltip已成功，原始frame850／1000已目視新增地點／距離文字可讀；己方／敵方與半徑以native驗證，不冒稱都有截圖。

RustV5 **1648通過／10既有ignored**（core807），Lua69＋140與ID契約、最新build-v2 UBT3.34秒、backend-v2、WorldV2 **30項**成功。兩map各74影格／6片段。最終local-v2完整成功：**209 native、458案例／222 intents／157 stages／13 cursor、69 Blueprint、五段實際PCM**。native為201 Success＋8 SuccessWithWarnings、12 warnings（含音訊警告）；Blueprint 0 error／0 warning。

接受前後**3340個來源／資產檔一致**，另核對current build／接受前staging及import library SHA，沒有把2個derived outputs當來源。19份原始JSON逐byte封存並獨立複核；完整工作流7次健康取樣，180秒排程的UTC觀測最大間隔181秒，期間另持續讀取最新log確認進度。所有owned工作流與watcher已退休。 詳見[最終驗收](docs/plans/2026-10-21-brush-attack-final-acceptance.md)、[錯誤R01–R06](docs/plans/2026-10-21-brush-attack-errors.md)、[原始JSON封存](docs/acceptance/2026-10-21-brush-attack/index.json)。RustV1／V2／V4失敗保留；V3／WorldV1雖過，仍因漏接野怪入口主動退休local-v1，最終以V5／WorldV2／local-v2取代，不冒稱單session效能。

未新增權威對局／重設配額，未commit／push，omfx不維護。舊MVP仍33/44；11項外部ACK／LAN／實際OS DPI／效能接受保留。後續可做連接多邊形草叢、特殊技能精確形狀／metadata、史詩怪增益／分裂及專用美術。

## 2026-10-21 牆體視野與單位目標提示（前次）

`add-moba-wall-vision-and-target-guidance` **5/5完成**。compiled terrain的Fixed64牆體LOS與碰撞／公開幾何同源，兩map正式boot接入Wave B／fog／即時單位道具，legacy descriptor政策保留。技能提示由生成metadata及公開rank顯示己方／敵方可見單位需求，未知／替換／未學習／退休清除，不把候選當權威合法性確認。原始frame850已目視敵方需求與確認／取消文字可讀；己方字樣以UE native驗證，未取得對應抽樣截圖。

Rust **1640通過／10既有ignored**；UE WorldV2 **29項成功**；完整local-v3 **208 native、458cases／222intents／157stages／13cursor、69Blueprint、五PCM**成功。native為200 Success＋8 SuccessWithWarnings／12 warnings；World為28 Success＋1 SuccessWithWarnings／4 warnings；音訊1 warning已包含於native統計，Blueprint0 error／0 warning。 最新UE build-v2 UBT1.72秒、backend-v2成功。兩map各74影格／6片段。

正式來源／資產 **3339檔**接受前後一致；2項derived outputs依current build、接受前staging SHA與import library另驗。v1→v3只有音訊runner Lua改變，Rust／Unreal來源不變，明確保留RustV2／WorldV2 cohort。19份原始JSON逐byte封存及SHA獨立複核；完整流程8個健康樣本，UTC最大間隔180秒，另有持續讀取最新測試log檢查；owned workflow與watcher全部退休。 V1／V2背景音訊為零的失敗保留；正式runner採Engine＋Editor兩層process-local設定，先五PCM獨立驗證，再完整v3成功，不修改OS音量／持久設定或放寬斷言。

詳見[最終驗收](docs/plans/2026-10-21-wall-guidance-final-acceptance.md)、[錯誤W01–W06](docs/plans/2026-10-21-wall-guidance-errors.md)、[原始JSON封存](docs/acceptance/2026-10-21-wall-guidance/index.json)。未新增權威對局／重設配額，未commit／push；舊MVP仍33/44。後續可做連接多邊形草叢、特殊技能精確形狀／手寫metadata、史詩怪增益／分裂及專用美術，11項外部接受另列。

## 2026-10-21 草叢隱蔽與技能落點提示（前次）

`add-moba-brush-and-point-area-preview` **6/6完成**。兩張三路map各四塊可通行草叢，optional bounded brushes與公開地表同源；權威使用與現有地表一致的膠囊形doubled-raw幾何，外接矩形角落不誤隱蔽。Wave B公開單位、fog格網與單位道具即時可見性共用規則，仍受距離／偵測／既有LOS資源限制，保留scope／owner／override優先序；不擋草叢後方草外視野。切回舊map清除舊resource。

技能生成器直接讀 executable AreaDamage 的 radius_key，依公開rank產生point落點傷害圈（同心圓聯集取最大半徑），未學習／未知／非point／替換／非法值／runtime退休不殘留。箭雨220／240／260／280已接到HUD及共用Slate投影，與較細的施法距離圈分別顯示；不從cosmetic cue猜範圍，不推測單位候選的權威合法性。特殊手寫技能的扇形／路徑尚未全面接線。

最終Rust **1636通過／10既有ignored**（template59／core795／codegen57／base246／backend190／bridge127／client147／ABI15）；Lua69＋140及ID契約通過。UE5.8 build-v2 UBT3.14秒、backend-v2建置／staging成功。WorldV2 **28項成功**、兩map各74幀／6片段；完整local-v2 **207 native、458cases／222intents／157stages／13cursor、69Blueprint、五PCM**通過，native12／World4／音訊1警告保留。原始frame750已檢查共用提示文字可讀，未當成箭雨AoE截圖；實體喇叭聽感、權威ACK與單session效能未驗，沒有新增權威對局或重設配額。

詳見 [最終驗收](docs/plans/2026-10-21-brush-preview-final-acceptance.md)、[錯誤P01–P08](docs/plans/2026-10-21-brush-preview-errors.md)、[13份原始JSON封存](docs/acceptance/2026-10-21-brush-preview/index.json)。8次健康檢查最大間隔180秒，流程全部退休。最終盤點3339來源／資產一致，另外2個正式建置產物（import lib、scripts DLL）已證明於完整接受前更新且符合build／staging SHA；不能聲稱3341檔全部未變。P06發現圓角外觀／矩形權威不一致後，已主動退休local-v1並完整重跑新cohort。尚可直接實作：連接多邊形草叢、特殊技能精確形狀、史詩怪增益／分裂及專用美術；舊MVP仍33/44，未用本機證據冒充11項外部接受。未commit／push。

## 2026-10-21 水晶、基地雙塔與超級兵（前次）

`complete-moba-base-structure-progression` **6/6完成**。新增可選共享base_structures規則：同路線塔全部退休後開水晶，任一水晶死亡後開基地雙塔，雙塔全滅才開主堡；水晶300秒復生、雙塔180秒／40%血量復生，復生會重新保護主堡，暫停／結束凍結且重複死亡不延長期限。所有私有state／deadline納入digest，公開admission沿原有建築投影。敵方同路水晶死亡期間每波超級兵取代砲車，三路全破每路兩隻；水晶復生只恢復後續編成，已有超級兵保留。一般三路無角色計畫入口也已接上authored兵線與波次，保留舊單路模式。

實際畫面發現最初基地重疊，已改兩map共享幾何2倍／4800邊長（lane、terrain、地表與camp中心），射程／速度／模型大小保留；線塔2200/1600/1000、水晶650、雙塔400、基地鏈minimum_spacing250。塔位用有界整數route弧長驗證，沒有水平假設；保留左下／右上基地與反向河流、14terrain、16營地32怪。新水晶Tower32770、雙塔32771、超級兵Creep60000，原ID不漂移。兩隊六水晶四雙塔已接原生呈現，超級兵115高度／骨架／動畫生命週期已驗；仍沿用現有模型，水晶美術尚非專用晶體。

最終Rust **1573通過／10既有ignored**（246／790／58／190／127／147／15）、Lua140＋69與ID契約；三種子5v5各4500tick及雙副本無repair，三種子完整三路16493／16493／19329tick成功結束。UE5.8最後build-v5 UBT10.63秒，backend-v3正式建置／staging成功。完整local-v2 **206 native、458cases／222intents／157stages／13cursor、69Blueprint、五PCM**成功；WorldV4場景27成功，兩map各74幀／6片段。native12警告、場景4、音訊1保留，實體喇叭聽感未驗。新原始JPEG frame450已確認E缺少可見單位／放開與取消文字可讀。

來源證據分開：UE接受cohort source-v4共3339檔前後一致；完成檢查後僅改`omb/src/config/server_config.rs`補一般三路兵線，精確transition證明其餘3338檔未變，該後端檔另跑完整190項及正式建置。最終source-v5的3339檔亦前後一致。封存 **13份byte-identical JSON**：`docs/acceptance/2026-10-21-base-structures/index.json`；完整結果 `docs/plans/2026-10-21-base-structures-final-acceptance.md`，錯誤B01–B15／E1448。local-v1小地圖fixture漏算Slate screen round trip而失敗，保持一raw誤差上限、改為實際螢幕取樣後才以完整local-v2接受；早期重疊場景與所有failed cohort保留。每180秒監測、最終6個樣本與owned程序退休確認。

未新增權威連線對局、重設10/10、維護omfx、commit或push；舊MVP仍33/44、11外部缺證。仍可在不新增對局下推進草叢視野、史詩buff／分裂、所有技能精確AoE與專用結構美術；下一輪應直接實作可完成的新範圍，避免只重跑已有接受。phased不當作單session效能或權威ACK。

## 2026-10-21 群怪、野區牆與技能施放提示（前次）

`extend-moba-camp-and-structure-rules` 6/6完成。共享Lua新增有界members與patience／soft reset；兩map各16營地、32隻怪，群組仇恨／重置但各自HP、攻擊、擊殺獎勵，最後一隻死亡才開始整營重生。軟返回可在營地範圍內重接戰，外部傷害拒絕；硬返回免傷且較快回家並恢復。私有state與全部member位置／HP雜湊；附屬怪40000起獨立保留IDs，追加map不改原IDs。12個鏡射牆群加原2塊共14terrain，驗lane／出生／全部怪本體不阻塞與真NPC繞障。正式基地策略為任一路三層塔全部死亡退休即可突破，保留舊all-lane策略；尚非完整inhibitor／nexus雙塔鏈。

施放預覽共用owner／公開資料：射程圓、方向線／箭頭、落點、射程內外顏色與距離、缺少可見單位／落點、QWER未學習／冷卻、實際施放鍵及確認／取消方式。提示沿既有preview退休，不改提交契約；單位型只表示候選，不猜命中或把地面點當單位中心。真AVI先發現10字級縮放後過小，修正為共用physical文字下限、至少12physical px／大字偏好、量測行高與換行底板，並依玩家視口放在落點附近。最終800×600原始影格已檢視，正常／帶提示快速施法文字可讀。

Rust1568通過／10既有ignored、Lua140＋69及跨map附屬ID專項。三種子5v5各4500tick、雙世界／雙team副本零repair，三種子營地3000tick與完整三路生命週期成功。最後UE5.8建置UBT8.38秒、正式backend建置／staging成功；fresh local-v2完整206 native、458cases／222intents／157stages／13cursor phased coverage、69Blueprint及五PCM成功；27場景專項與兩map各74幀／6片段成功，3338來源／資產前後一致，三分鐘監測8樣本及owned程序退休已確認。native12警告、場景4、PCM1保留，不冒稱零警告；local-v1控制通過但字太小，已由新建置與完整v2取代。

封存 `docs/acceptance/2026-10-21-moba-completeness/index.json`；結果 `docs/plans/2026-10-21-moba-completeness-final-acceptance.md`，細節錯誤 `docs/plans/2026-10-21-completeness-errors.md` C01–C09，總索引E1447。原始AVI未修改，JPEG使用binary且逐byte核對，失敗擷取不接受。沿用既有模型；完整水晶／基地雙塔及其重生、史詩buff／分裂、草叢視野、所有技能精確AoE等仍待後續實作，不宣稱LOL完整複製。未新增權威連線對局、重設10/10、處理omfx、commit或push；歷史MVP33/44及11外部缺證保留，phased不當作單session效能／權威ACK。

## 2026-10-21 左下／右上三路與 NPC 行為（前次）

本輪 `align-diagonal-map-and-npc-behaviors` 7/7完成。兩張公開map改為左下／右上基地、外側上下路與斜向中路、反向河道、18塔與16營地；基地／出生／塔位與Bot推線沿共享route，消除水平假設。正式兵線讀取Lua編成：30秒一波、3近戰＋3遠程、每第三波加砲車，使用真射程／傷害與公開攻擊動畫。護主依攻擊者生命＋受害player保存並雜湊，分開小兵／塔期限，支援成功指定技能、塔對實際英雄傷害反擊、AOE不延長小兵、正在拆塔者不轉火；Idle／AttackMove不自動拉中立，明確攻擊仍可。野怪追擊、繞障、硬leash返回免傷、到家恢復與重生已驗；world_units合併全部map且保留穩定ID。正式投影有界上限64KiB，真KCP framing與超限拒絕已驗。

Rust1565通過／10既有ignored，Lua140＋69及跨map catalog專項；三種子正式5v5各4500tick、雙世界與雙team副本無repair，另有三路完整生命週期。正式後端與UE5.8建置成功（最終UBT14.63秒），fresh local-v2完整206native、458cases／222intents／157stages／13cursor phased coverage、69Blueprint及五PCM成功；26場景專項與兩map全景／實際viewport已檢視，各74幀／6片段，3337來源／資產前後一致，三分鐘監測及所有owned程序退休已確認。native與場景含已記錄警告，不宣稱零警告；local-v1舊catalog==2測試失敗保留，修正為真目標ID與實際Slate／TCP驗證後才完整重跑。

封存 `docs/acceptance/2026-10-21-diagonal-npc/index.json`；完整結果 `docs/plans/2026-10-21-diagonal-map-npc-final-acceptance.md`，錯誤E1433–E1446。呈現沿用現有模型，單營地單怪、硬leash，沒有LOL完整野怪群、patience、史詩增益或水晶機制；基地沿用all_lane_towers解鎖。場景fixture是合成可見性／代表單位位置，不能冒稱新權威對局ACK；NPC邏輯由真Rust模擬驗證。未新增權威連線對局、重設10/10、修改omfx、commit或push。舊MVP仍33/44、11外部缺證保留；若只是要求繼續且外部條件未變，不重跑相同驗收冒充新進度。

## 2026-10-20 不新增對局範圍已收尾（前次）

2026-10-20不新增對局範圍收尾：目前功能已由最近完整本機接受驗證，fresh複驗3335來源／資產與該接受完全一致，38份封存SHA／size及原始458案例／222 wire／157 stages／13 cursor strict union再核對成功；本輪沒有重建或重新宣稱fresh全部測試。206 native、26專項、69 Blueprint、五PCM、140＋69 Lua明示沿用來源相同的最近接受。剩餘11項逐項缺證見`docs/plans/2026-10-20-nonmatch-closeout.md`：8項需原權威ACK，4.7／7.3需實際100%／150% OS DPI配對，7.4需真對局／LAN及單session固定效能；現有四輪PIE只實測150%。33/44不變，沒有新增對局或重設10/10。一般繼續應先查此表；外部條件未變時，不以無直接對應缺證的新局部修正或相同完整測試重跑取代剩餘項目。E1432記錄本輪checker輸出命名疏漏與獨立輸出處理。

## 2026-10-20 移動與生命值音效目前frame／綁定來源（最近功能完整接受）

2026-10-20移動與生命值音效目前frame／綁定來源接受：audio component共用O(1) IsCurrentPresentationFrame驗正epoch及精確epoch／tick，health與locomotion均在基準副作用前使用；移動入口另驗目前綁定元件及同world／bridge owner。真world／source／兩bridge音效元件有效red重現外來元件吃掉合法起步；green驗舊idle不假起步、落後audio frame但高於HP tick及future狀態不改baseline、fresh起步、同entity future／past health不污染clock／HP且保留恢復caption。原entity／epoch、死亡與新timeline斷言保留；NoSound只驗admission／caption且零voice，五PCM另驗真輸出。不新增actor時鐘欄位、timer、掃描、重送或Rust／FFI。普通UE5.8 local無action cache建置22.80秒，26專項、fresh206 native、完整分段458案例／222 wire／157 stages／13 cursor、69 Blueprint與五PCM成功；140＋69 Lua、3335來源一致，527 Rust／ABI及93內容Lua未變，完整Rust明示沿用。兩段錄影、公開map144幀／12片段、八圖、三分鐘監測及清理已驗。38份封存`docs/acceptance/2026-10-20-audio-locomotion/index.json`，詳`docs/plans/2026-10-20-audio-locomotion-final-acceptance.md`，E1431完整記錄。未新增對局、重設10/10、commit或push；33/44、十一父項待權威／真LAN／效能或實際OS DPI證據，phased不冒稱單session效能。

## 2026-10-20 生命值音效角色／呈現來源隔離（上一輪完整接受）

2026-10-20生命值音效角色／呈現來源隔離接受：PresentOriginalAudioFrame在任何基準／owner／scope／HP／tick副作用前核對目前entity id／generation、正且一致的audio epoch與同world／bridge owner。真world／source／cue history有效red精確重現錯世代frame產生目前死亡cue；green驗錯id／generation／舊epoch隔離、外來future tick不污染clock、目前恢復／死亡cue與caption、新epoch低tick先seed且不假重生，NoSound零voice明示，實際音訊另由五PCM驗證。不新增欄位、timer、掃描、網路重試或ABI。普通UE5.8 local無action cache建置6.17秒、最終fixture8.13秒；26專項、fresh206 native、完整分段458案例／222 wire／157 stages／13 cursor、69 Blueprint與五PCM成功，140＋69 Lua、3335來源一致，527 Rust／ABI及93內容Lua輸入未變，完整Rust明示沿用。首輪MinimapPreferences右拖曳命中失敗與diagnostic-v2未重現保留；fixture明確前景／tooltip／cursor前置與還原，未改命中座標或放寬斷言，具體舊遮擋來源未確認；WorldV3及source-v2獨立接受。兩段錄影、公開map144幀／12片段、八圖、三分鐘監測及清理已驗。43份封存`docs/acceptance/2026-10-20-audio-health/index.json`，詳`docs/plans/2026-10-20-audio-health-final-acceptance.md`，E1429–E1430完整記錄。未新增對局、重設10/10、commit或push；33/44、十一父項待權威／真LAN／效能或實際OS DPI證據，phased不冒稱單session效能。

## 2026-10-20 骨架動畫生命值frame時序（上一輪完整接受）

2026-10-20骨架動畫生命值frame時序接受：AOmAnimatedUnitActor在副作用前以既有NativeHealthTick拒絕落後已接受HP frame的動畫快照，補齊僅比較上一筆動畫tick的缺口；保留current/equal tick refinement、隱藏、新epoch及pool reset，不新增欄位、timer、掃描或ABI。七正式hero實際骨架／instance有效red精確重現recall變idle；green驗姿勢與暫停保護、current/equal tick恢復及新時間軸。四個既有fixture將代表目前狀態的default tick 0改成明確目前HP tick，所有原斷言保留；另納入英雄生命週期、小兵與原創動畫。普通UE5.8 local無action cache建置7.84秒、二十五專項、fresh206 native、完整分段458案例／222 wire／157 stages／13 cursor、69 Blueprint與五PCM成功；140＋69 Lua、3335來源一致，527 Rust／ABI及93內容Lua輸入未變，完整Rust明示沿用。兩段錄影、公開map144幀／12片段、八圖、三分鐘監測及清理已驗。38份封存`docs/acceptance/2026-10-20-animation-frame/index.json`，詳`docs/plans/2026-10-20-animation-frame-final-acceptance.md`；E1425查找、E1426產品缺口及E1427 UE報告BOM解析錯誤均記錄。未新增對局、重設10/10、commit或push；33/44、十一父項待權威／真LAN／效能或實際OS DPI證據，phased不冒稱單session效能。

## 2026-10-20 戰鬥特效動畫快照時序（上一輪完整接受）

2026-10-20戰鬥特效動畫快照時序接受：AOmUnitActor以獨立CombatAnimationTick拒絕比目前動畫舊的快照，避免HP frame尚未推進時被舊動畫取消回城或改暫停；foreign entity與負tick先拒絕，同tick refinement保留，concealment保留時序，presentation reset及epoch切換清時序。共用所有角色，零新timer／掃描／網路重試，Rust／FFI不變。真生成hero／world pool有效red精確重現舊動畫取消回城，green驗stale cancel／unpause、實際age凍結、same-tick cancel／resume、舊回城不復活、foreign generation不污染clock、隱藏與新epoch低tick。普通UE5.8 local無action cache建置44.31秒、二十二專項、fresh206 native、完整分段458案例／222 wire／157 stages／13 cursor、69 Blueprint與五PCM成功；140＋69 Lua、3335來源一致，527 Rust／ABI及93內容Lua輸入未變，完整Rust明示沿用。兩段錄影、公開map144幀／12片段、八圖、三分鐘監測及清理已驗。38份封存`docs/acceptance/2026-10-20-combat-ordering/index.json`，詳`docs/plans/2026-10-20-combat-ordering-final-acceptance.md`；E1423記錄查找操作錯誤、E1424記錄實際缺口與修正。未新增對局、重設10/10、commit或push；33/44、十一父項待權威／真LAN／效能或實際OS DPI證據，phased不冒稱單session效能。

## 2026-10-20 目前回城持續特效配置恢復（上一輪完整接受）

2026-10-20目前回城持續特效配置恢復接受：SetRecall回報實際呈現狀態，actor另記bRecallVisualPending；配置被拒絕後，只有後續目前有效動畫且scope／frame／capacity／resources允許時補配，正常已呈現回城不重啟age。O(1) capacity gate使池滿pending快照不重掃效果陣列；取消、隱藏、死亡、epoch／presentation reset清pending，不重播oneshot、不新增timer或網路重試、不提高512上限。真生成hero／pool有效red填滿512格並釋放後重現缺圈框；green驗pending有界、恰好一圈、age保持、取消不復活及fresh回城可用，保留Buff、死亡／隱藏／scope、shader及content rebinding。普通UE5.8 local無action cache建置44.95秒、二十二專項、fresh206 native、完整分段458案例／222 wire／157 stages／13 cursor、69 Blueprint與五PCM成功；140＋69 Lua、3335來源前後一致，527 Rust／ABI與93內容Lua未變，完整Rust明示沿用。兩段錄影、公開map144幀／12片段、八圖、三分鐘監測及清理已驗。38份封存`docs/acceptance/2026-10-20-recall-admission/index.json`，詳`docs/plans/2026-10-20-recall-admission-final-acceptance.md`，E1421記錄缺口與修正。未新增對局、重設10/10、commit或push；33/44、十一父項仍待權威／真LAN／效能或實際OS DPI證據，phased不冒稱單session效能。

## 2026-10-20 已有native Buff更新退休（上一輪完整接受）

2026-10-20已有native Buff更新退休接受：AOmUnitActor private UpdateTrackedNativeBuff供default Refresh／Updated共用，已追蹤native handle遇zero remaining、無pool或SyncBuff拒絕時，沿UntrackBuffEffect同步退休renderer與ownership並回傳handled，避免殘留handle或進legacy。合法有限值與-1無限期保留；不自動重試或在update建立新visual，fresh State沿既有admission恢復。custom／legacy不由helper接管，target／generation guard保留，無Rust／FFI schema或逐entity tick變更。真生成hero／pool有效red先驗renderer0再重現handle1，green驗terminal refresh、NaN refresh／update、hidden update、infinite及custom保護，保留舊世代隔離、512容量、shader與content rebinding。普通UE5.8 local無action cache建置63.09秒、二十二專項、fresh206 native、完整分段458案例／222 wire／157 stages／13 cursor、69 Blueprint與五PCM成功；140＋69 Lua、3335來源前後一致，527 Rust／ABI與93內容Lua未變，完整Rust明示沿用。兩段錄影、公開map144幀／12片段、八圖、三分鐘監測及清理已驗。38份封存`docs/acceptance/2026-10-20-buff-update/index.json`，詳`docs/plans/2026-10-20-buff-update-final-acceptance.md`，E1420記錄缺口與修正。相依建置／測試均確認session退出後才接續。未新增對局、重設10/10、commit或push；33/44、十一父項仍待權威／真LAN／效能或實際OS DPI證據，phased不冒稱單session效能。

## 2026-10-20 Buff事件entity世代隔離（上一輪完整接受）

2026-10-20 Buff事件entity世代隔離接受：OmUnitActor共用IsCurrentBuffTarget在default Added／State／Removed／Refreshed／Updated所有視覺操作前，以O(1)驗目前有效entity及target id／generation一致；native、tracked custom、明確legacy與debug cue共用邊界，避免同visual key的舊事件清目前handle／pool或改目前文字。既有Cleanup／Untrack API不變，合法事件仍能建立／更新／退休；未新增header／ABI或逐entity掃描。真生成hero／pool有效RedV2精確重現stale remove清目前ownership，green驗錯id／世代五類事件、zero refresh隔離、current update／remove、custom與legacy文字保護，保留512容量／回復／shader與content rebinding回歸。RedV1因未確認build session退出就啟動被排除，獨立RedV2才採證，E1418–E1419記錄流程與實際缺口。普通UE5.8 local無action cache建置6.30秒、二十二專項、fresh206 native、完整分段458案例／222 wire／157 stages／13 cursor、69 Blueprint與五PCM成功；140＋69 Lua、3335來源前後一致，527 Rust／ABI與93內容Lua未變，完整Rust明示沿用。兩段錄影、公開map144幀／12片段、八圖、三分鐘監測及清理已驗。39份封存`docs/acceptance/2026-10-20-buff-identity/index.json`，詳`docs/plans/2026-10-20-buff-identity-final-acceptance.md`。payload無Runtime epoch，本輪不冒稱跨Runtime ACK或任意Blueprint override隔離。未新增對局、重設10/10、commit或push；33/44、十一父項仍待權威／真LAN／效能或實際OS DPI證據，phased不冒稱單session效能。

## 2026-10-20 正式Buff特效配置邊界（上一輪完整接受）

2026-10-20正式Buff特效配置邊界接受：AOmUnitActor::OnBuffState讓native路徑完整封閉，錯target／generation先拒絕，zero remaining退休，SyncBuff成功才配置ownership handle；滿載／隱藏／非法狀態拒絕時清過期native／text bookkeeping並返回，不再落入legacy文字元件繞過容量。既有自訂handler與明確legacy模式保留，持續狀態在容量回復後可由fresh snapshot恢復。真world共享池及正式生成hero有效red實際填滿512格後重現額外registered text；green驗連續拒絕不增component／handle、不超上限、容量回復、錯世代／隱藏／NaN拒絕及自訂／legacy相容與清理。首輪green的抽象UObject探針觸發ensure，改用具體USceneComponent後重跑，失敗及source-v1保留，最終source-v3另建；全輪v1另揭露無native pool的EditorPreview依賴舊fallback，改Editor world及有效HP後保留原斷言，納入專項並完整重跑；失敗全輪及source-v2保留。普通UE5.8 local無action cache正式修正6.17秒／最終fixture10.13秒成功，二十二專項、fresh206 native、完整分段458案例／222 wire／157 stages／13 cursor、69 Blueprint與五PCM成功；140＋69 Lua、3335來源前後一致，527 Rust／ABI與93內容Lua未變，完整Rust明示沿用。兩段錄影、公開map144幀／12片段、八張原圖、三分鐘監測及清理已驗。42份封存`docs/acceptance/2026-10-20-buff-admission/index.json`，詳`docs/plans/2026-10-20-buff-admission-final-acceptance.md`；E1413–E1417保留查找失誤、實際缺口與fixture錯誤。未新增對局、重設10/10、commit或push；33/44、十一父項仍待權威／真LAN／效能或實際OS DPI證據，phased不冒稱單session效能。

## 2026-10-20 失效HUD閱讀提示退休（上一輪完整接受）

2026-10-20失效HUD閱讀提示退休接受：SOmHudToolTipHost正常Tick在持有tip但IsEmpty時，透過既有OnClosed歸還仍屬自己閱讀提示的user焦點，再以CloseIfCurrent清舊內容，避免只隱藏卻保留焦點及回原HUD範圍復活舊提示。局部shared pointer保護close callback，正常可用提示定位、字型、scroll與快捷鍵仲裁保留，不呼叫global CloseToolTip或關閉其他host。既有ReadableHudToolTip真GameViewport／Slate hover、wheel、click有效red精確重現焦點殘留；green在global tooltip更新前驗host自己退休及不復活，完整原尺寸／字級／鍵盤／Esc／GC回歸保留。普通UE5.8 local無action cache建置5.57秒、二十一專項、fresh206 native、完整分段458案例／222 wire／157 stages／13 cursor、69 Blueprint與五PCM成功；140＋69 Lua、3335來源前後一致，527 Rust／ABI及93內容Lua未變，完整Rust明示沿用。兩段錄影、公開map144幀／12片段、十七張原圖、三分鐘監測及清理已驗。48份封存`docs/acceptance/2026-10-20-tooltip-retirement/index.json`，詳`docs/plans/2026-10-20-tooltip-retirement-final-acceptance.md`；E1410–E1412保留rg誤用、實際產品缺口與來源基準未完成就比對的流程錯誤。未新增對局、重設10/10、commit或push；33/44、十一父項仍待權威／真LAN／效能或實際OS DPI證據，phased不冒稱單session效能。

## 2026-10-20 小地圖公開視圖切換（上一輪完整接受）

2026-10-20小地圖公開視圖切換接受：SOmMinimap::SetState以O(1)比較本機玩家、payload公開player／team及map bounds，context改變或不可用時退休自己記錄的Slate user／pointer capture、inspect、communication overlay與camera footprint；同視圖更新保留拖曳，其他user／pointer不受影響。擴充既有MinimapPreferences真Runtime及actual Slate fixture，有效red精確重現team切換殘留捕捉，green驗team／player／bounds、舊move失效與fresh press可用。修正ResponsiveHudLayout測試的前一個互動提示污染及paint-to-desktop座標，實際wheel必須命中指定HUD scroll；原tooltip、16次上限及完整文字斷言保留。普通UE5.8最終建置17.82秒、二十專項、fresh206 native、完整分段458案例／222 wire／157 stages／13 cursor、69 Blueprint及五PCM成功；140＋69 Lua、3335來源前後一致，527 Rust／ABI及93內容Lua未變，完整Rust明示沿用。兩段錄影、公開map144幀／12片段、九張圖、三分鐘監測及清理均驗證。45份封存`docs/acceptance/2026-10-20-minimap-view-scope/index.json`，詳`docs/plans/2026-10-20-minimap-view-scope-final-acceptance.md`；E1403–E1409保留實際缺口、四份failed-green、失敗診斷建置與防重犯。未新增對局、重設10/10、commit或push；33/44、十一父項仍待權威／真LAN／效能或實際OS DPI證據，phased不冒稱單session效能。

## 2026-10-20 整組HUD投影Runtime生命週期（上一輪完整接受）

2026-10-20整組HUD投影Runtime生命週期接受：補齊舊結算modal、回城時鐘及其他HUD投影跨Runtime保留的缺口。widget ResetRuntimeInputScope集中透過既有setter同步清除結果model／實際panel、回城payload／文字、英雄／資源／技能／summoner、team／scoreboard／stats／selection／Buff／minimap及inspect、通訊、feedback、商店transactions／undo與提交狀態，再設為等待新投影。HUD owner沿既有ClearPresentedCaches一起退休identity／mode／epoch／composition與rate cache，避免新Runtime同模式被誤認未變；保留widget／owner與控制偏好，沒有新增逐entity／frame工作。真port-zero Runtime及合法bound未選用來源，透過正式typed API放入明示受控投影，red精確重現舊結算未退休；green驗整組清理、fresh hero／clock、冪等Start與重複Stop。失敗建置C4458及兩份failed-green保留；修正SlotIndex、先visual後label，測試精確驗只剩目前快捷鍵而非錯誤要求整段空白。普通UE5.8最終建置6.32秒、十九專項、fresh206 native、完整分段458案例／222 wire／157 stages／13 cursor、69 Blueprint及五PCM成功；140＋69 Lua、3335來源前後一致，527 Rust／ABI及93內容Lua未變，完整Rust明示沿用。兩段錄影、公開map144幀／12片段、八圖、三分鐘監測及清理均驗證。41份封存`docs/acceptance/2026-10-20-hud-projection-runtime-scope/index.json`，詳`docs/plans/2026-10-20-hud-projection-runtime-scope-final-acceptance.md`，E1399–E1402記錄實際缺口與防重犯。受控typed payload不冒稱真權威結果或ACK；沒有新增對局、重設10/10、commit或push，33/44、十一父項仍缺權威／真LAN／效能或實際OS DPI證據，phased不冒稱單session效能。

## 2026-10-20 HUD已揭露身分Runtime生命週期（上一輪完整接受）

2026-10-20 HUD已揭露身分Runtime生命週期接受：補齊頭像／名稱及已揭露身分快取跨Runtime保留的缺口。HUD自己的ResetGameplayInputScope沿既有ClearIdentityCaches清除身分及overlay快取，標示composition重新套用，再走widget scope清理；不依賴world發空投影、下一tick或來源選用順序，保留可重用owner／widget與同Runtime死亡的正常身分保留。既有真port-zero Runtime fixture用契約完整但未選用的來源，經正式ApplyHeroHud放入明示受控公開投影；red-v2精確重現，green驗立即清頭像／名稱、新Runtime死亡不借舊身分、fresh英雄可用、冪等Start與同Runtime死亡保留當前身分、重複Stop及退休widget安全。red-v1普通Actor缺六delegates遭正確拒絕，不算有效red。普通UE5.8建置6.42秒、十九專項、fresh206 native、完整分段458案例／222 wire／157 stages／13 cursor、69 Blueprint及五PCM成功；140＋69 Lua、3335來源前後一致，527 Rust／ABI及93內容Lua未變，完整Rust明示沿用。兩段錄影與公開map144幀／12片段、八圖、三分鐘監測與清理均核對。39份封存`docs/acceptance/2026-10-20-hud-identity-runtime-scope/index.json`，詳`docs/plans/2026-10-20-hud-identity-runtime-scope-final-acceptance.md`，E1396–E1398保留查找／fixture錯誤及實際缺口。受控typed payload不冒稱真網路資料或ACK；沒有新增對局、重設10/10、commit或push，33/44、十一父項仍缺權威／真LAN／效能或實際OS DPI證據，phased不冒稱單session效能。

## 2026-10-20 記分板Runtime生命週期（上一輪完整接受）

2026-10-20記分板Runtime生命週期接受：修正直接Stop／Start仍保留Tab hold的缺口。InputOwner共用ResetRuntimeInputScope透過CloseOwnedScoreboard清全部hold槽位／keys，widget自身reset同步關閉requested，不依賴下一tick、world清rows或訂閱順序。實際PlayerInput按Tab的有效red精確重現；green驗無tick立即退休、舊release、新press、冪等Start保留live hold與重複Stop，保留控制profile及可重用owner。普通UE5.8建置17.18秒，十九專項、fresh206 native、完整分段458案例／222 wire／157 stages／13 cursor、69 Blueprint及五PCM成功；140＋69 Lua、3335來源前後一致，527 Rust／ABI及93內容Lua未變，完整Rust明示沿用。兩段錄影與兩公開map錄影驗證、八圖逐張檢視、三分鐘監測及清理成功。38份封存`docs/acceptance/2026-10-20-scoreboard-runtime-scope/index.json`，詳`docs/plans/2026-10-20-scoreboard-runtime-scope-final-acceptance.md`，E1395記錄重現與通用修正。沒有新增對局、重設10/10、commit或push；33/44、十一父項缺權威／真LAN／固定效能或實際OS DPI證據，phased不冒稱單session效能。

## 2026-10-20 溝通Runtime生命週期（上一輪完整接受）

2026-10-20溝通Runtime生命週期接受：補齊直接Stop／Start仍保留草稿、gesture及通訊UI快取的缺口。widget共用ResetCommunicationScope供Runtime reset／terminal退休／無bridge刷新，清實際edit／rows、history／objectives、markers、音訊去重／roster、scope／room／run、mute／result token與notice；僅草稿持有焦點時歸還自己viewport，保留偏好及可重用widget／owner，正常view更新的有效草稿規則不變。InputOwner同步CancelCommunicationGesture，沿既有queued release退休，不在回呼發訊息。真port-zero Runtime＋actual owned widget/model受控合法view red準確重現；green19專項成功（四筆既有preview-world warning），驗即時退休、新Runtime重用、冪等Start保留live狀態及偏好。普通UE5.8建置35.79秒、fresh206 native／完整分段458案例／222 wire／157 stages／13 cursor／69 Blueprint／五PCM、140＋69 Lua及3335來源前後一致；527 Rust／ABI、93內容Lua未變，完整Rust明示沿用。兩段8fps錄影、兩map144幀／12片段、六HUD與兩map圖、三分鐘監測及所有清理成功。38份封存`docs/acceptance/2026-10-20-communication-runtime-scope/index.json`，詳`docs/plans/2026-10-20-communication-runtime-scope-final-acceptance.md`；E1394保留有效red及共用修正。受控scope初始化不冒稱真聊天事件／權威ACK，完整viewport事件另驗；沒有新增對局、重設10/10、commit或push，33/44、11父項仍缺權威／真LAN／固定效能或實際OS DPI配對，phased不冒稱單session效能。

## 2026-10-20 鏡頭Runtime生命週期（上一輪完整接受）

2026-10-20鏡頭Runtime生命週期接受：補齊Space temporary follow及camera held state跨直接Stop／Start保留的缺口。camera從PostInitializeComponents／owner刷新維護scope訂閱、EndPlay／BeginDestroy解除；共用ReleaseHeldCameraModes在Runtime退休時不恢復舊視點，清pan／drag／scout／minimap inspect、舊hero／隊友位置與pointer樣本，保留profile、input component、zoom及lock preference。真port-zero Runtime＋InputStack red-v2準確重現；green-v2十九專項成功（四筆既有preview-world warning），驗舊Space／release不復活、新press可用、按住方向鍵停止、minimap跨重啟取消、保存鎖定隨fresh快照恢復及冪等Start保留hold。正式build-v1為51.31秒、最終fixture build-v3為6.09秒；fresh 206 native／分段458案例／222 wire／157 stages／13游標／69 Blueprint／五PCM、140＋69 Lua契約成功，3335來源前後一致，527 Rust／ABI與93內容Lua未變，完整Rust明示沿用。兩公開map錄影144幀／12片段及兩段完整8fps錄影嚴格驗證、六HUD與兩map圖逐張檢視、三分鐘監測與所有清理成功。E1390–E1393保留操作失誤、fixture無效red、有效red與越界failed-green；41份封存`docs/acceptance/2026-10-20-camera-runtime-scope/index.json`，詳`docs/plans/2026-10-20-camera-runtime-scope-final-acceptance.md`。沒有新對局、重設10/10、commit或push，33/44及11父項缺證仍待驗；phased清單覆蓋不稱單session效能／權威ACK／真LAN／實際OS DPI配對完成。

## 2026-10-20 世界Runtime與分段完整接受（上一輪完整接受）

2026-10-20世界Runtime與分段完整接受：共用world scope reset及suppressed control-only結果修正完成，當前18專項成功（四筆既有preview-world warning）；ordinary UE5.8建置UBT11.27秒，140 owned＋69 local Lua契約、205 native、69既有Blueprint與五PCM成功。新增正式foundation入口及嚴格聯集：294／120／103加tail164／102／54，合計458案例／222 wire／157 stages與13游標觀察；兩段各保留原860／900秒期限、90 proof frames、8fps錄影及獨立完整清理，逐段重驗raw artifacts、七binary SHA與內容／ABI一致。當前整套phased local acceptance成功，single_session_full_success=false，原三次單次全程逾時與feature-only失敗包仍保留，未宣稱效能修復。兩公開map錄影144幀／12片段成功，3335來源資產前後一致；527 Rust／ABI及93內容Lua未變，完整Rust明示沿用。六HUD及兩map圖片已逐張檢視，world停止／恢復另有實際actor與widget斷言。詳`docs/plans/2026-10-20-world-runtime-scope-phased-acceptance.md`與`docs/acceptance/2026-10-20-world-runtime-scope-phased/index.json`。沒有新增權威對局、重設10/10、commit或push；33/44、11父項待驗不變，實際authority／OS DPI／LAN與固定效能缺證不勾選。

## 2026-10-20 世界Runtime生命週期（前次feature-only；單次全程未通過）

2026-10-20世界Runtime生命週期：已實作共用scope訂閱／解除，直接subsystem Stop／Start也清除世界actor／target／cue／audio與frame游標；抽DispatchOwnedInputResults補suppressed HUD漏control-only結果，完整snapshot維持先收據後結果及未連線owned HUD門檻。red及green-v1失敗保留，green-v2十八專項通過（四筆既有preview-world warning），ordinary UE5.8建置30.64秒、205 native／69 Blueprint／五PCM分項成功，127 Lua與stage SHA成功；3334來源前後一致，527 Rust／ABI及93內容Lua未變，完整Rust明示沿用。完整viewport三輪仍超出原860秒，含不錄影v3；進度147／209、144／200、153／215（control stages／wire），全部清理成功，未宣告本輪fresh完整458事件PASS。兩map錄影144幀／12片段獨立嚴格成功，不能替代完整流程。27份封存`docs/acceptance/2026-10-20-world-runtime-scope/index.json`明確success=false、feature_success=true，詳`docs/plans/2026-10-20-world-runtime-scope-validation.md`。E1377–E1382記錄全部錯誤與診斷界線；沒有降低門檻、新對局、重設10/10、commit或push。33/44及11父項待驗不變，上一輪owner-runtime-scope完整接受僅為歷史參考。

## 2026-10-20 輸入owner Runtime生命週期（上一輪完整接受）

2026-10-20輸入owner Runtime生命週期接受：補齊直接Stop／Start仍保留ward renderer admission及舊操作資格的缺口。共用EnsureRuntimeInputBindings在owner建立即訂閱命令與scope reset，替換／退休同時解除；停止只取消方向／analog／pointer hold／瞄準／cast與舊hero／ward／能力／道具資格，不在停止回呼發送命令，保留profile及可重用owner。red-v1實際997未歸零已重現；green十七專項零warning，普通UE5.8建置36.78秒、完整205 native／458實際操作／222 wire／157 stage／13 cursor／69 Blueprint／五PCM及3334來源一致，127 Lua與三分鐘檢查成功。527 Rust／ABI及93內容Lua未變，完整Rust明示沿用；22份證據`docs/acceptance/2026-10-20-owner-runtime-scope/index.json`，詳`docs/plans/2026-10-20-owner-runtime-scope-final-acceptance.md`。E1376已記錄，沒有新對局／重設10/10／commit／push，父項33/44及11待驗不變。

## 2026-10-20 商店Runtime生命週期（歷史）

2026-10-20商店Runtime生命週期接受：補齊直接Stop／Start仍保留舊undo pending／receipt watch／request id／purchase pin及經濟資格的缺口。widget共用ResetRuntimeInputScope先退休關聯與結算文字，再沿既有空經濟投影退休buy／sell及recipe資格；保留widget等新公開投影恢復，冪等Start保留live pending，不推測退款或重送。red-v1實際987未歸零已重現；green十六專項零warning，普通UE5.8建置37.56秒、完整205 native／458實際操作／222 wire／157 stage／13 cursor／69 Blueprint／五PCM及3334來源一致，127 Lua與三分鐘檢查成功。527 Rust／ABI及93內容Lua未變，完整Rust明示沿用；22份證據`docs/acceptance/2026-10-20-shop-runtime-scope/index.json`，詳`docs/plans/2026-10-20-shop-runtime-scope-final-acceptance.md`。E1373–E1374已記錄；沒有新對局／重設10/10／commit／push，父項33/44及11待驗不變。

## 2026-10-20 Runtime輸入關聯生命週期（歷史）

2026-10-20 Runtime輸入關聯生命週期接受：補齊兩個World tick之間直接Stop／Start的HUD回覆退休；Runtime每次明確Stop及成功建立新實例同步重設一般操作關聯，冪等Start保留pending。先以真port-zero Runtime重現唯一失敗，再驗直接Stop即時清除、舊結果不復活、兩次重啟可各自使用request 1、來源隔離與退休。普通UE5.8建置50.25秒、十六專項零warning、完整205 native／458實際操作／222 wire／157 stage／13 cursor／69 Blueprint／五PCM及3334來源一致；127 Lua通過，527 Rust／ABI與93內容Lua未變，完整Rust明示沿用。三分鐘程序檢查及最終清理成功，E1370保留red／green證據。沒有新對局／重設10/10／commit／push；33/44、11父項待驗不變。詳`docs/plans/2026-10-20-runtime-input-scope-final-acceptance.md`，22份可攜證據見`docs/acceptance/2026-10-20-runtime-input-scope/index.json`。

## 2026-10-20 Runtime共用送出回覆（歷史）

2026-10-20 Runtime共用送出回覆接受：修正Stop／Hold直接Runtime入口漏通知；改在成功提交後單一發布實際wire owner／request id，由live HUD owner依selected source與captured controller／epoch訂閱並在重綁／退休解除，移除世界及小地圖的分散通知。普通build-v3成功10.30秒，十六專項零warning；完整205 native／458實際操作（新增每次Stop／Hold送出到HUD斷言）／222 wire／157 stage／13 cursor／69 Blueprint／五PCM與3334來源一致。527 Rust／ABI、93內容Lua未變，完整Rust明示沿用；127 Lua契約與三分鐘owned監測通過。E1366–E1369記錄缺口、失敗候選與修復，沒有新對局／重設10/10，沒有commit／push；33/44、11父項待驗不變。詳`docs/plans/2026-10-20-runtime-feedback-final-acceptance.md`，可攜證據`docs/acceptance/2026-10-20-runtime-feedback/index.json`。

## 2026-10-20 一般操作回覆接線（歷史）

本輪實際補齊4.3一般操作request／result HUD接線，涵蓋世界／小地圖、Move／AttackMove／AttackTarget／cast／Stop／Hold等；64筆／10秒有界關聯，區分送出、確認、拒絕及不確定，不重送、不預測效果。普通build-v6成功14.59秒，十六專項零warning、完整205 native／458操作／69 Blueprint／五PCM、3334來源驗收前後一致，527 Rust／ABI及93內容Lua未變明示沿用。詳 `docs/plans/2026-10-20-action-feedback-final-acceptance.md`，可攜證據 `docs/acceptance/2026-10-20-action-feedback/index.json`。E1359–E1365保留全部失敗；33/44及11父項待驗不變，沒有新對局／重設10/10，沒有commit／push。

## 2026-10-19 非新增對局收尾（歷史）

接續複核：`docs/plans/2026-10-19-nonmatch-continuation-recheck.md`。fresh Lua契約127通過，3331來源／資產SHA、size、mtime及41份可攜證據SHA／size再次一致，正式程式未變，沿用下述完整接受。沒有找到可補齊剩餘11項的權威錄製；control.ack只是測試控制回覆，不是gameplay ACK。狀態仍33/44，沒有新對局／重設額度；E1358記錄本次唯讀盤點錯誤。

依 `docs/plans/2026-10-19-nonmatch-final-acceptance.md` 完成1.2／1.3／4.1／6.2／6.4／6.6。普通build-v29成功14.31秒，fresh十三專項零warning、203 native全成功（12既有warning）、458操作／222 wire／157 stage／13 cursor、69 Blueprint零診斷、五PCM及全部清理成功；3331來源／資產SHA、size、mtime完整複驗一致。可攜原圖、摘要與指紋在 `docs/acceptance/2026-10-19-nonmatch/index.json`。Rust／ABI及內容Lua未變，明示沿用下方新機完整接受，非fresh Rust。

共用動畫亂序、nameplate root回收、血條遮字與地表／模型遮擋修正已驗；原生及未修改原BP真PIE stop／restart與兩種window mode成功，實測僅150% OS DPI。公開map錄影144幀／12片段及fixed before／after圖明示synthetic，不是新權威對局／ACK。完整v2曾在舊道具卡tick座標與視窗取樣路徑失敗；共用actual paint／hit-grid及完整render-buffer校正後v5整體接受，原失敗不覆寫，詳E1354／E1357。

剩餘11項：3.1／3.3／3.4／3.5／3.9／4.3／4.4／7.1／7.4包含權威ACK、真對局／LAN或固定效能證據；4.7／7.3缺實際100／150 OS DPI配對。10/10額度不重設，不新增對局，不把wire admission當ACK。後續只在具可用條件時驗這些範圍。本輪尚未commit／push。下方新機與更早接受仍保留歷史。

## 2026-10-19 新機完整本機驗收

使用`D:/UE5.8`（實際5.8.4）完成正式bootstrap／普通NoEngineChanges／最後stage SHA；實作公開生命增減文字池、持續公開攻擊資格同步與相關通用修正。完整198 native成功（12既有warning保留）、69 Blueprint零compiler／load診斷、458 viewport／222 wire／157 control／13游標、五實際PCM、3900影格／11 AVI及全部清理成功。Rust／ABI／TD1–100波／各consumer與五checks均成功，既有ignored與重複harness明示保留；3325份最後來源／資產SHA及mtime核對通過。

詳`docs/plans/2026-10-19-ue58-final-acceptance.md`，可攜摘要與原圖在`docs/acceptance/2026-10-19-new-machine/index.json`。影片仍為synthetic filtered-feed黑底世界fixture，非權威ACK／LAN／整場世界美術接受；沒有新權威對局、沒有重設10/10。4.3生命數字子範圍完成，整體仍27/44與17項待辦。

## 前一台電腦的完整驗收（歷史）

可攜摘要：`docs/acceptance/2026-10-18-handoff/index.json`。原始接受報告與來源指紋、八張錄影原始JPEG及兩張已審世界PNG也保存在同目錄。

- ordinary NoEngineChanges／local／NoUBA／nocache建置成功，最後入口20秒。
- **197項完整原生成功**，12個既有warning保留；另明確執行2項editor專項，全部成功／零warning，不加算為198項完整原生。
- **458個實際視窗事件案例／222 wire intents／157 control stages／13個實際游標觀察**。wire admission不是權威ACK。
- **69個既有Blueprint**編譯／載入零error、零warning，資產SHA與mtime未變。
- **五份真實submix PCM**通過；沒有宣稱實體喇叭聽感已驗，也沒有把PCM當成AVI音軌。
- **1197份來源／資產及保護mtime**核對一致。441份已擷取Rust／ABI輸入相同，明示沿用之前真實通過的1168 Rust測試／五checks／九個既有ignored及ABI30；不是此次重新跑Rust。
- 錄影有4811張取樣影格／11個原尺寸片段，約138 MiB；本機原始經過797.450秒，封裝8fps播放約601.375秒，真實時間看JSONL。八張原始JPEG已直接看過。**這是黑底synthetic filtered-feed輸入fixture，不是正式整場世界錄影或美術接受。**

本次交接只整理版本與可攜性，不把既有驗收改稱新電腦重新通過。換機後必須重新建置與驗收。

## 新電腦取得版本

```bat
git clone --recurse-submodules --branch master https://github.com/damody/omoba.git
cd omoba
git submodule sync --recursive
git submodule update --init --recursive
git status --short
git submodule status --recursive
```

已有clone時先保存當地變更，再對主倉庫`git pull --ff-only`並更新submodule。不要各自把submodule拉到任意最新HEAD；主倉庫gitlink才是這次可重建組合。submodule處於detached HEAD是正常狀態。歷史`omfx`不編譯、不修復、不納入驗收，也不刪除。

## 工具鏈與建置

- Windows；Rust固定 **1.95.0**。host與`base_content.dll`必須同rustc，不能只升級其中一邊。
- 本機引擎是 **UE5.8.4**，位於`D:\UE5.8`，沒有隨Git上傳Engine或其build cache。新電腦需自行準備相容引擎。
- 新機成功的C++工具鏈：VS2022 Community17.14.37806.7、MSVC compiler19.44.35229（目錄14.44.35207）、Windows SDK10.0.22621.0。依實際可用工具鏈診斷，不修改Engine manifest／BuildId冒充相容。
- Lua固定用clone內`tools\lua\lua.exe`；不要新增PowerShell／Python／shell fallback。根目錄只維持既有四個薄bat。
- Unreal專案是`omfue\om.uproject`。DLL／EXE、Binaries、Intermediate、Saved、DDC、target與個人設定不搬移，應在新電腦重建。

在**實際clone根目錄**執行（引擎路徑依新機調整）：

```bat
rustup toolchain install 1.95.0
cargo install cbindgen --version 0.29.4 --locked
tools\lua\lua.exe scripts\build_ue_moba.lua --ue-root D:/UE5.8 --build-executor local --disable-action-cache --max-parallel-actions 1 --prefetch-shared-pch
tools\lua\lua.exe scripts\build_ue_moba.lua --ue-root D:/UE5.8 --verify-staged-only
```

第一次遇到正式EngineChange時，走同一Lua入口加`--refresh-engine`，完成後再跑一次不含refresh的普通建置。若存在`omfue/Saved/BuildTemp/source-engine-bootstrap.json`，必須完成正式bootstrap流程，**不手刪marker、不改Engine／UBT／共享PCH**。原機限制並行1是驗過的保守基準；快電腦可在確認RAM／磁碟餘裕後使用工具支援的`--max-parallel-actions 1..64`，不可因此放寬驗收門檻。

`run_ue.bat --td --editor --no-build --dx12`可開既有editor路徑。正式MOBA入口是`run_ue.bat --moba`，但它會啟動真對局，請先遵守下節的對局限制；本次交接未另開對局。

2026-10-19新機補充：VS2022須提供UE接受的MSVC版本，並以實際`cl.exe`與UBT log核對；安裝器launcher的exit0不代表背景套件已完成。若換工具鏈後舊建置圖仍選舊compiler，local入口可加`--compiler-version Latest`或明確MSVC版本，該選擇會一致套用ShaderCompileWorker、bootstrap與普通NoEngineChanges階段。個人UBT XML中的過時`PCHMemoryAllocationFactor=1000`已由最小重現確認會造成D8000；新機已先完整備份後只移除此覆寫，恢復Engine原有自動配置。不要修改Engine／UBT來源或手改共享responsefile；完整本機結果以當輪新log為準。

## 新機驗收與限制

```bat
tools\lua\lua.exe scripts\moba_renderer_fixture.lua build
cargo build --manifest-path omb/Cargo.toml -p omobab --bin moba-config
cargo build --manifest-path scripts/Cargo.toml -p base_content --release
tools\lua\lua.exe scripts\ue_local_acceptance.lua --ue-root D:/UE5.8 --run-id 新的唯一ID --native-rhi d3d12 --record-video
cargo test --manifest-path omb/Cargo.toml -p omobab
cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi
cargo test --manifest-path scripts/Cargo.toml -p base_content
cargo test --manifest-path scripts/Cargo.toml -p omoba-core
```

Lua完整本機驗收不新增權威對局，但需要先有正式建置與本機renderer fixture執行檔。原始log／PCM／AVI會在新機產生，不應從舊機cache補假成功。其餘Rust／ABI驗證細節與五checks請參考保留的Rust報告及功能計畫。

本次交接另以四個倉庫的暫存區建立全新checkout（明確啟用`core.autocrlf=true`），**1472個素材／授權／receipt／匯入producer／固定Lua runtime輸入逐一SHA相同**，四個根目錄bat保持CRLF／無BOM。證據為`docs/acceptance/2026-10-18-handoff/fresh-checkout-sha256.json`；這是可攜性驗證，並非新機完整建置。

**既有權威對局批次已10/10耗盡。** 換電腦不會重設額度；不要另開match、改10/10計數、換入口繞過或用synthetic fixture冒充真對局／ACK／LAN／效能。若沒有合適執行條件，保留待驗。

## 剩餘17項：依原計畫繼續

1. **1.2／1.3**：既有遊戲入口的固定視點與actual controller／pawn／view target／focus／hit-test；native及原Blueprint action owner的PIE／重連矩陣。
2. **3.1／3.3／3.4／3.5**：右鍵、精確點擊／AttackMove、Stop／Hold、英雄限定與autoattack的真權威ACK及行為矩陣；不要重寫已完成接線來冒充新功能。
3. **3.9**：ward完整權威charges／生命期／team／fog與實際對局接受。
4. **4.1／4.3／4.4／4.7**：完整HUD／objective版面、指令成功與拒絕回饋、商店權威流程，以及字體／palette／縮放與多resolution／DPI接受。
5. **6.2／6.4／6.6**：所有正式角色／單位可辨識性、缺失動畫與VFX收尾；正式出生、lane、多人combat、fog、HUD、shop的固定視點before-after PNG與事件影片。現在的黑底輸入錄影不能替代正式世界影片。
6. **7.1／7.3／7.4**：全action跨native／原Blueprint的實際事件→有效camera／原ACK／visible UI；OS DPI／window mode／PIE／重連矩陣；有條件才做真match／LAN／固定效能。

建議先依`docs/plans/2026-10-18-mvp-functional-task-audit.md`與實際來源核對缺口，優先做能直接改善可玩性的世界、動畫與HUD部分；各項已實作但欠證的部分要明確區分，不因父項未勾就再造另一套架構。

## 延續工作規則

- 依使用者最新指示，**自行實作，不使用子代理或Grok**；不接續`.ai-collab`舊active job。先列計畫直接做，遇到問題自行判斷通用解，不反覆要求確認。
- 錯誤必須寫入`docs/plans/unreal-moba-error-register.md`；本次交接前最後編號E1299，新增前先檢查檔頭避免重號。
- build、Rust、native或asset consumer尚活著時，不改production／test來源。工具回傳session代表還在跑，等真正退出並核對來源後才修正。
- 改submodule時先在最內層commit／push，再讓上層更新gitlink；不可只推父repo而漏推子commit。
- 素材／授權與SHA守衛保留原始byte；已補`.gitattributes`避免資產與匯入producer的checkout行尾轉換。原始授權空白不隨意美化；既有UE兩處程式尾端空格、Rust三檔EOF空行及tasks.md尾空格也不為交接重寫已驗收來源，沒有宣稱全量diff-check零診斷。
- `.ai-collab`調查／草稿、log／trace／錄影／cache與`graphify-out`不搬移；已追蹤的歷史AI協作紀錄保留，可攜接受證據以`docs/acceptance/2026-10-18-handoff/`為準。內含原始報告的`D:\omoba`等路徑只代表舊機來源，不是新機執行指令。
- 沒有提交Engine、私有登入設定、API key、建置產物或個人對局profile；不需要Git LFS才能取得本批素材，新增單檔小於GitHub的100MiB限制。


## 2026-10-22 通用Buff層數玩家提示（最新）

# 2026-10-22 通用Buff層數最終驗收

OpenSpec complete-moba-buff-stack-presentation **5/5完成**。延續全遊戲17系統盤點，補上單台電腦可驗證的玩家狀態資訊。

## 實作

權威隊伍Buff新增通用JSON欄位stacks，既有objective_stacks維持。限時／永久規則、數值與封頂由原有lease控制，沒有UI自行推算或依營地／英雄ID特判。沿原有ABI33 payload字串與hash／更新事件傳遞。

Unreal共用有界decoder：空payload／缺欄為單層，stacks只接受1..65535整數；非法JSON、非數字、非整數、超界、超過4096 bytes與越界string ref拒絕。完整owner列表先篩選hero／generation／duration才解析，typed row StackCount在widget再次驗證。複數顯示「名稱 ×N（剩餘時間／持續生效）」，單層文字與diagnostic getter維持相容；只改層數也刷新player快取，空baseline／換代仍清除。

## 集中驗收

RustV1 **1756通過／10既有ignored／0失敗**：template70、core883、codegen57、base261、backend190／1、bridge130／1、client150／8、ABI15；generation等九工作exit0且退休。

新增權威測試經五次實際南史詩死亡／重生，限時／永久共用stacks與舊欄位一致、最多四層。新增ABI回歸驗證payload bytes、permanent flag、hash、同key更新、hero換代舊owner退休與空snapshot清除。

原生OwnedBuffListSnapshot測試新增timed／permanent的×3／×4文字、count-only快取刷新、非法Blueprint輸入、decoder缺欄／最大值／錯誤型別／非整數／超界、ABI string ref越界等案例。這是實際初始化／TakeWidget後的原生widget驗證；本輪沒有新增專用層數截圖。

UE5.8 build-v1成功（UBT **58.24秒**）、正式後端與Lua三契約成功。WorldV1 **37成功**（36 Success＋1 SuccessWithWarnings，0 Failed／NotRun），4既有preview-world清理警告保留。兩張map各74影格／6片段，兩張map及既有首殺資訊列三張PNG已目視，屬場景與既有HUD回歸。

**3355來源／資產**接受前後集合、SHA、size、mtime一致，七項staging與import library另驗。**44份證據**逐byte／SHA封存後獨立複核；三分鐘監測 **5次／最大181秒**，7 worker＋7 watcher與全程monitor退出。

## 界線與剩餘

本輪無建置／測試失敗，兩次路徑查詢錯誤與熱路徑設計注意記錄於2026-10-22-buff-stacks-errors.md。完整MVP仍33/44，11項外部ACK／LAN／OS DPI／效能待驗未改；沒有新增網路對局、配額重設、commit／push或omfx維護。未把前輪native／IPC／Blueprint／PCM數字算成本輪接受。

仍需補小兵光環／分裂、經濟補償、更多擊殺／目標公告、技能引導／幾何與施法動畫、召喚複本生命期、塔皮與美術。盤點是功能入口清單，這批完成不代表整個遊戲已達LOL全部規則與內容。

證據索引：../acceptance/2026-10-22-buff-stacks/index.json。



## 2026-10-22 史詩兵線光環（最新）

# 2026-10-22 史詩兵線光環最終驗收

OpenSpec complete-moba-objective-minion-aura **5/5完成**。接續17系統盤點，補單台電腦可驗證的史詩目標與兵線連動。

## 實作

隊伍獎勵新增optional minion_aura：buff_id、radius1..3000、攻擊／護甲bonus0..1000且非全零。舊內容缺欄None，個人Buff不可帶光環，ID必須註冊、不可與英雄獎勵ID混用，同ID政策一致。Lua manifest、生成與canonical hash沿現有內容管線。北史詩保留180秒／死亡遺失英雄Buff，另給900半徑、+15攻擊／+10護甲的攻城祝福。

權威從現有team lease選有效／存活／eligible英雄，以固定點平方距離影響同side存活LaneRole::Creep，包含邊界、排除英雄／塔／野怪／敵人。同類光環不隨英雄數或獎勵層數疊加。以正常BuffStore單instance提供加成，僅在membership／payload改變或instance缺失時寫入；離開、發射者死亡、資格失去、到期／結束清除，pause凍結。新wave於首次戰鬥決策前同步；state保存自己擁有的target generation／ID／加成並納入digest。沒有emitter時不掃兵線。

同時修正實作檢查發現的實際戰鬥接線：MOBA兵線攻擊發射以既有UnitStats聚合基礎攻擊，DamageInstance的Unit／Hero護甲與魔抗同樣聚合既有BuffStore。原有傷害旗標、暴擊／閃避與固定點減傷公式保留。效果沿公開snapshot／ABI33 buff payload與生成Unreal catalog傳遞，沒有新增專用光環美術或宣稱有新圖示。

## 集中驗收

V1於policy tuple的E0382編譯錯誤失敗，三worker與watcher完全退休後才改成借用Option<&Aura>並跑V2；V1所有原始log與receipt封存。Lua三契約V1成功。

RustV2 **1762通過／10既有ignored／0失敗**：template72、core886、codegen57、base262、backend190／1、bridge130／1、client150／8、ABI15。含generation九工作exit0且退休。

新增core三案例覆蓋含邊界範圍、隊伍／兵種、兩英雄重疊、正常攻擊／護甲聚合、離開／返回、HP0／死亡資格、到期／結束、pause、新wave、移除generation與缺失Buff恢復。新增正式60Hz測試經真實北史詩死亡授獎，驗證實際lane AI DamageInstance攻擊+15、真實100物傷依基礎護甲+10只減傷一次、公開snapshot含siege_aura、敵人不受益、匿名傷害後native DeathTick退休並撤回小兵效果。既有完整map／雙隊digest／終局凍結及其他玩法一起回歸。

UE5.8 build-v2成功（UBT **46.03秒**）、正式後端build-v2與Lua三契約成功。WorldV1 **37成功**（36 Success＋1 SuccessWithWarnings，0 Failed／NotRun），4既有preview-world清理警告保留。兩張map各74影格／6片段，兩張map與既有首殺資訊列三張原生PNG已目視，屬既有場景／HUD回歸，不當成專用光環畫面接受。

**3356來源／資產**接受前後集合、SHA／size／mtime一致，七項staging與import library另驗。**68份證據**逐byte／SHA封存後獨立複核；三分鐘監測 **8次／最大181秒**。7個最終worker／watcher及3個失敗V1 worker／watcher與全程monitor均退休。

## 剩餘與界線

錯誤A01..A04與通用修正收於2026-10-22-minion-aura-errors.md，總登錄E1472。完整MVP仍33/44，11項外部ACK／LAN／OS DPI／效能待驗維持；沒有新增網路對局、配額重設、commit／push或omfx維護。沒有引用前輪native／IPC／Blueprint／PCM數字作本輪成功。

仍需擴充野怪分裂、經濟補償、塔皮、更多目標公告、技能引導／特殊幾何與施法動畫、召喚複本生命期及美術；完整度盤點仍是功能入口清單，不代表整個遊戲已達LOL全部規則與內容。

證據索引：../acceptance/2026-10-22-minion-aura/index.json。


## 2026-10-22 外塔塔皮獎勵與加固（最新）

# 2026-10-22 外塔塔皮最終驗收

OpenSpec complete-moba-tower-plating **5/5完成**。接續17系統完整度盤點，補上單機可驗證的塔皮玩法與防禦單位邊界修正。

## 實作

SingleLaneConfig新增optional TowerPlatingRules，預設None保留舊fixture；正式server與role plan均啟用同一training設定：5層、840秒、每層125附近金、1200半徑、每破層+20護甲。plates1..16、expiry1..3600、gold<=1m、radius1..10000、armor<=1000皆於setup原子驗證。數值為本專案平衡，不宣稱對應特定LOL patch。

權威只接受tracked外層lane塔，依entity generation保存max HP與已破層數。i128固定點門檻可一次跨多層；治療、重複Outcome不會重領。匿名、同隊或無附近英雄的已跨門檻仍消耗，不能晚點追領。沿既有英雄／召喚／NPC隊伍歸因，僅附近存活進攻英雄逐層均分捨餘，不因合併傷害多拿餘數；不發額外KDA或XP，與既有拆塔／首塔金獨立。

加固沿正常BuffStore，使用有限剩餘時間與pending tick補償，不改基礎護甲；下次實際傷害讀到已破層*20護甲。到期、HP0、強制退役、結束清除，ledger與規則加入digest。金額沿既有owner economy同tick送出，Buff沿ABI33 snapshot與生成Unreal catalog。專用塔血條分段與美術仍未新增。

檢查防禦路徑同時修正前輪魔抗單位接線：DamageInstance百分點與UnitStats normalized比例明確往返，保留25%基礎魔抗，20%額外魔抗組合後約40%；100魔傷實際約60。固定點結算與既有公式／旗標保留。

## 集中驗收

RustV1 **1768通過／10既有ignored／0失敗**：template72、core891、codegen57、base263、backend190／1、bridge130／1、client150／8、ABI15。含generation九工作exit0且退休。

新增core四個塔皮案例涵蓋exact HP門檻、逐層／跨多層分金一致、半徑包含／超界、匿名／同隊消耗、內塔排除、治療／重複致死、護甲聚合、pause／expiry、強制退役／結束、原子無entity副作用及digest。另新增魔抗百分點回歸。既有server與role plan測試確認正式啟用設定。

新增正式60Hz案例經generated lumen_bolt實際施放破層，驗證當tick owner economy含正確塔皮金、公開tower_plating payload層數與有限倒數；下一次100物傷實際按當前加固減傷一次，840秒後不再給金且清Buff；無額外擊殺／助攻。既有全map／雙隊digest／死亡重生／結束凍結與其他玩法一併回歸。

UE5.8 build-v1成功（UBT **45.18秒**）、正式後端與Lua三契約成功。WorldV1 **37成功**（36 Success＋1 SuccessWithWarnings，0 Failed／NotRun），4既有preview-world清理警告保留。兩map各74影格／6片段，兩map與既有首殺資訊列三張原生PNG已目視，屬既有場景與HUD回歸，不當成塔皮專用美術接受。

**3357來源／資產**接受前後集合、SHA／size／mtime一致；七項staging與import library另驗。**44份證據**逐byte／SHA封存並獨立複核；三分鐘監測 **8次／最大181秒**。7 worker／watcher與全程monitor均退出。

## 剩餘與界線

本輪沒有建置／測試失敗。P01..P04紀錄分金、退休、路徑查詢與前輪魔抗接線修正，總登錄E1473。完整MVP33/44與11項外部ACK／LAN／OS DPI／效能待驗維持；沒有新增網路對局、重設配額、commit／push或omfx維護。沒有把前輪native／IPC／Blueprint／PCM數字算成本輪接受。

後續仍需塔血條分段與美術、野怪分裂、經濟補償、更多目標公告、技能引導／特殊幾何與動畫、召喚複本生命期。這批完成不代表整個遊戲已達LOL全部規則與內容。

證據索引：../acceptance/2026-10-22-tower-plating/index.json。


## 2026-10-22 塔皮分段名牌（最新）

# 2026-10-22 塔皮分段名牌最終驗收

OpenSpec complete-moba-tower-plating-presentation **5/5完成**。接續17系統盤點，補塔皮的玩家可見層數與期限。

## 實作

獨立tower_plating_status native Buff公開health_segments={total,remaining}，本身沒有戰鬥加成。有效外塔第一次active tick即提供5/5，未受損也可見；剩餘由權威不可倒退ledger決定，治療不恢復已破層數。有限期限、退休／到期／結束清除，既有護甲與金額政策維持。

沿ABI33既有公開Buff payload與catalog，Unreal共用有界decoder驗證total1..16、remaining0..total、有限正期限、payload4096與字串邊界。只接精確公開存活generation，相同target多來源衝突拒絕。每frame只建一次map，原始UTF8 marker先篩選，沒有候選就不做JSON或entity集合，活體集合只保留候選key。完整baseline清除缺項，control-only保留。

typed FOmHealthSegmentState沿FrameState到實際Actor與原生名牌。分段列搭配數字「塔皮3/5」與權威剩餘秒數，與HP條獨立，HP回復不會誤導成塔皮恢復。普通名牌不建立cell，首次有效資料才建立最多16格並重用；Actor／Widget以顯示ceil秒數快取，避免同秒每frame寫Slate。血條偏好、可見性預算與退休清除延用，沒有renderer自行推算deadline。

## 集中驗收

V1 Rust全數通過；UE V1於測試列舉Hostile不存在而編譯失敗。前批Rust／UE／後端／Lua consumer與watcher均退休後才改Enemy，並完成延遲配置／整秒快取修正；V1 Rust與UE失敗原始紀錄封存。

RustV2 **1770通過／10既有ignored／0失敗**：template72、core892、codegen57、base263、backend190／1、bridge131／1、client150／8、ABI15。含generation九工作exit0且退休。新增core驗證未受損5層、無額外加成、破層後治療仍3層、到期／退休；新增ABI驗證初始metadata bytes、有限期限、剩餘變更事件與移除。正式60Hz既有塔皮技能／錢包／減傷案例擴充初始status、公開remaining與到期清除，其他玩法與完整map／雙隊digest／終局凍結一併回歸。

UE5.8 build-v2成功（UBT **81.22秒**）、正式後端V1與Lua三契約成功。WorldV1 **38成功**（37 Success＋1 SuccessWithWarnings，0 Failed／NotRun），4則測試警告記錄保留。

新增HealthSegmentsPresentation原生測試：非法JSON／型別／整數／超界／期限拒絕，公用ABI字串／catalog／generation／衝突守門，實際Bridge→Actor→Widget，control-only保留／完整缺項清除／退休，普通名牌0 cell／首次16格／重用／同顯示秒不重寫、偏好與死亡清除。專用原生Slate PNG已目視確認分段格、3/5與剩120秒；另兩map與既有首殺資訊列三張PNG回歸，兩map各74影格／6片段。專用截圖為實際原生widget fixture，不宣稱新增真對局畫面接受。

**3360來源／資產**接受前後集合、SHA／size／mtime一致，七項staging與import library另驗。**67份證據**逐byte／SHA封存獨立複核；三分鐘監測 **9次／最大181秒**。7個最終worker／watcher、2個V1 worker／watcher與全程monitor均退出。

## 剩餘與界線

錯誤U01..U06與修正收於2026-10-22-plate-ui-errors.md，總登錄E1474。MVP33/44與11項外部ACK／LAN／OS DPI／效能待驗維持；沒有新增網路對局、重設配額、commit／push或omfx維護，沒有引用前輪native／IPC／Blueprint／PCM數字冒作本輪接受。

仍需更完整建築／場景美術、野怪分裂、經濟補償、更多目標公告、技能引導／特殊幾何與動畫、召喚複本生命期。本輪完成不代表整個遊戲已達LOL全部規則與內容。

證據索引：../acceptance/2026-10-22-plate-ui/index.json。


## 2026-10-22 野怪有界死亡分裂（最新）

# 2026-10-22 野怪死亡分裂最終驗收

OpenSpec complete-moba-jungle-splitting **5/5完成**。依17系統完整度盤點，新增單機可驗證的野怪死亡分裂；正式維護範圍維持omb／共用Rust／scripts／omfue。

## 實作

共用Lua catalog的主體與初始成員皆可指定death_children。最多兩代分裂、每body四子體、每營生命期總數32；遞迴驗證HP／傷害／獎勵／ID／offset、累積home與root leash、座標界線及地形碰撞。規則與後代內容均參與catalog hash，舊內容省略欄位維持空子體；無營地名稱判斷的runtime分裂規則。

致死ledger提交後，退役body僅排入一次子體定義與屍體附近的碰撞安全位置，mutable營地tick建立原生Unit／Faction／Pos／CProperty。子體保留原營群體仇恨、patience／soft／hard回頭及攻擊／回血規則；子體自己的HP／攻擊／gold／XP由內容指定，不繼承目標／個人Buff。強制退役與終局不產生子體。

有待生或存活後代時不啟動重生倒數；最後子體退休才開始deadline。整營重生重新建立初始人口，清除舊後代。pending定義與位置進既有deterministic digest。兩張map的blue／red石怪營皆啟用兩代分裂：2初始→4→4，共10個生命期個體，每營總124金／193XP；此為原創訓練平衡。

## 集中驗收

V1因模組helper引用區域valid_id而編譯失敗；整批consumer／watcher退休後提升共用函式。V2 UE與後端成功，Rust新增移動fixture缺ObservableFactBuffer而1失敗；等整批退休後補正式資源，再跑V3。原始失敗與先前成功紀錄皆封存，不加總多批通過數。

最終RustV3 **1774通過／10既有ignored／0失敗**：template73、core894、codegen57、base264、backend190／1、bridge131／1、client150／8、ABI15；包含generation九工作exit0且退休。

新增內容測試驗證樹深／分支／總人口／累積home／非法後代／deep hash／舊欄位預設。新增core測試驗證待生queue／重複死亡／各代數量／最後清空／deadline／初始人口重建、強制與終局退役、屍體附近位置、Buff不繼承、原營仇恨與hard回頭。新增正式60Hz測試對兩map雙方營地實際Damage／Death、重複致死、每body錢包／XP、公開entity snapshot、全清重生與相同seed digest重跑；既有整場雙隊filtered lifecycle等一併回歸。

UE5.8 build-v3成功（UBT **3.21秒**），正式後端v3與Lua三契約成功。WorldV1 **38成功**（37 Success＋1 SuccessWithWarnings，0 Failed／NotRun），4既有測試警告保留。兩map各74影格／6片段，兩map與既有首殺／塔皮名牌四張原生PNG已目視，屬回歸；本輪不宣稱取得分裂專用實際對局畫面。

**3360來源／資產**接受前後集合、SHA／size／mtime一致；七項staging及import library另驗。**93份證據**逐byte／SHA封存並獨立複核；三分鐘監測 **10次／最大181秒**。7個最終worker／watcher、6個先前worker／watcher與全程monitor均退出。

## 剩餘與界線

錯誤與預防記於2026-10-22-jungle-split-errors.md，總登錄E1475。仍需經濟補償、更多目標公告、持續引導／特殊技能幾何與動畫、召喚複本生命期與建築／場景美術。原MVP33/44與11項外部ACK／LAN／OS DPI／效能待驗不變；沒有新增真網路對局、重設配額、commit／push或omfx維護。

證據索引：../acceptance/2026-10-22-jungle-split/index.json。


## 2026-10-22 場上護盾條與權威倒數（最新）

# 2026-10-22 場上護盾名牌最終驗收

OpenSpec complete-moba-shield-nameplates **5/5完成**。沿整體17系統盤點，補護盾在場上單位的可讀性；正式範圍維持omb／共用Rust／scripts／omfue。

## 實作

原內部__damage_shield改為damage_shield，追加native-only公開Buff catalog。儲存仍是一個有界效果，既有護盾上限100萬、期限60秒、強者保留、部分吸收／耗盡／到期／死亡政策維持。既有可見entity Buff snapshot與catalog hash承載shield_remaining_raw，不用HP推算、不變更CABI33。owner economy護盾值仍保留且與公開資料一致。

Unreal共用decoder驗證payload4096、JSON數字整數、raw1..1024000000、明確除1024與有限正期限；Blueprint typed FOmShieldState二次檢查。Bridge在原始UTF8 marker篩選後才解碼，只接registered catalog與公開存活精確generation；字串越界、未知catalog、多來源衝突拒絕。完整baseline缺項清除、control-only保留；重置清map。

typed frame經Actor送原生名牌，獨立淺藍護盾條相對max HP、視覺封頂1；數字保留完整護盾量與權威ceil秒數，HP條不變。沿原有名牌偏好／可見性budget／退休。整秒顯示快取避免同秒Slate重寫；無護盾時連固定高度容器一併收合，不改普通名牌尺寸。

## 集中驗收

RustV2 **1776通過／10既有ignored／0失敗**：template73、core895、codegen57、base264、backend190／1、bridge132／1、client150／8、ABI15；含generation九工作exit0且退休。

新增core驗證公開catalog與payload100→60、較弱重給保留60、超額傷害回傳餘額、耗盡移除及到期；新增ABI測試確認部分吸收bytes／Updated、generation換代OwnerRemoved與到期Removed。既有正式60Hz generated道具購買／施放案例擴充public snapshot100→60→0、到期／死亡，與owner economy相符；既有護盾減傷順序、不治療、整場雙隊filtered lifecycle等一併回歸。

UE5.8 build-v2成功（UBT **3.64秒**），正式後端與Lua三契約成功。WorldV2 **39成功**（38 Success＋1 SuccessWithWarnings，0 Failed／NotRun），4既有測試警告保留。新增ShieldNameplatePresentation驗證解碼型別／界線／期限、公用ABI bounds／catalog／generation／衝突／死亡、實際Bridge→Actor→Widget、control-only／完整缺項／退休，原生數字／偏好／死亡／非法Blueprint／同秒快取；專用原生PNG已目視確認護盾條、300與剩3秒及獨立HP條。

兩map各74影格／6片段，另兩map／既有首殺／塔皮名牌四張PNG回歸；五張原生圖片均目視。專用護盾截圖為原生widget fixture，不宣稱新增真連線畫面接受。

**3363來源／資產**接受前後集合、SHA／size／mtime一致；七項staging及import library另驗。**79份證據**逐byte／SHA封存並獨立複核；三分鐘監測 **8次／最大181秒**。7個最終worker／watcher、6個V1 worker／watcher與全程monitor均退出。

## 剩餘與界線

V1橋接測試因OwnerRemoved列舉名稱錯誤而編譯失敗，世界export因此未執行；全批退休後修為BuffOwnerRemoved，完成V2完整重跑與原始失敗封存。S01..S05記錄公開身分接線、查詢quote修正、單位／期限及無護盾容器收合。錯誤MD為2026-10-22-shield-ui-errors.md，總登錄E1476。仍需經濟補償、更多目標公告、持續引導／特殊技能幾何與動畫、召喚複本生命期與建築／場景美術；原MVP33/44與11項外部ACK／LAN／OS DPI／效能缺證不變。沒有新增真網路對局、重設配額、commit／push或omfx維護。

證據索引：../acceptance/2026-10-22-shield-ui/index.json。


## 2026-10-22 通用抑療與共用治療結算（最新）

# 2026-10-22 抑療與共用治療結算最終驗收

OpenSpec complete-moba-healing-reduction **5/5完成**。接續17系統盤點，補玩家可實際使用的抑療與原有HealReceivedMultiplier接線。

## 實作

UnitStats共用received_healing，native Heal、ScriptHeal、正常正值HP regen採同一受療倍率／DisableHealing政策。raw與legacy因子限0..4，依source／family canonical排序、家族強者聚合、i128中間值與checked轉換；非法／溢位政策不產生治療，總增益封頂4倍。正負治療分明，寬位元HP加總與上限，不允許一般治療把HP0復活；Unit fallback也不再經render float轉換。

召喚師治療將當前權威倍率放入SummonerCastContext與不可變plan identity，在missing-HP cap前套用；提交重驗倍率，變更舊計畫拒絕，不在sink重複套用。完全禁療仍可按正常規則提交0治療與冷卻。泉水與營地完成hard reset為還原，泉水維持原規則，camp reset明確還原HP，避免抑療使重設營地殘血。

UnitHex新增optional healing_reduction={multiplier,duration}，內容檢查倍率0..小於1並拒絕量化為1、期限1/1024..60秒／非法型別／未知欄位，規則進content hash。舊內容省略維持None。native執行前重驗，沿既有敵隊／可見／距離／generation／owner與cooldown交易守門；拒絕不扣冷卻、不寫Buff，傷害後只對仍存活target加抑療。

咒擊符原ID11、價格450、55傷害、450距離／8秒冷卻保留，新增50%受療倍率持續4秒；此為原創訓練平衡。公共healing_reduction native Buff保留最強倍率與最長有效剩餘期限，同類來源不重複相乘；到期／死亡清除，沿既有公開Buff snapshot／ABI33與owned list顯示。generated Unreal metadata加入獨立倍率／期限，原生tooltip依本地化數字格式顯示50%抑療與4秒期限，非法資訊不啟用hotbar。沒有CABI layout變更。

## 集中驗收

V1因新增camp測試缺BuffStore import而core編譯失敗；全批退休後修正，並補發現的召喚師直接HP入口。V2 core897通過、舊assert期望致死後治療復活而1失敗；base新召喚師fixture缺protobuf型別匯入。全批退休後將舊assert明確改為0並保留存活target治療對照，protobuf改完整型別，完成V3。原始失敗與先前成功紀錄全封存，不加總多批數字。V3 Rust全通過後，WorldV1新tooltip斷言誤用整數文字，依既有本地化一位小數格式修正C++測試；Rust來源未再修改，保留V3 Rust／後端，完成UE V4／WorldV2及新來源比對。

最終RustV3 **1782通過／10既有ignored／0失敗**：template74、core898、codegen58、base265、backend190／1、bridge132／1、client150／8、ABI15；含generation九工作exit0且退休。

新增內容測試驗證optional／界線／量化／型別／hash／legacy，codegen驗證獨立typed metadata。新增core涵蓋native／script／legacy Unit、raw倍率／家族／canonical／溢位／regen恰一次／強弱refresh／到期／負治療／滿血／死亡。擴充既有camp reset及召喚師模型／實際ECS案例，驗證倍率在cap前、倍率變更舊plan拒絕、0治療冷卻。

新增正式60Hz案例實際買咒擊符、超距拒絕且CD0、合法ItemUse與generated70治療變35、native與script20各變10、公開Buff raw512／有限期限、pause凍結、實際SummonerCast200先減半再補50缺血到550、到期70治療恢復、死亡清Buff。既有整場雙隊filtered lifecycle、兵線／塔／野怪／技能等一併回歸。

UE5.8 build-v4成功（UBT **5.38秒**），正式後端v3與Lua三契約成功。WorldV2 **40成功**（39 Success＋1 SuccessWithWarnings，0 Failed／NotRun），4既有測試警告保留。新增HealingReductionItemMetadata確認generated ID11道具仍可unit targeting、55傷害、50%／4秒說明與非法metadata拒絕。兩map各74影格／6片段，兩map與既有首殺／塔皮／護盾五張原生PNG均目視，屬回歸，不宣稱新增抑療專用實際連線畫面。

**3364來源／資產**接受前後集合、SHA／size／mtime一致；七項staging及import library另驗。**105份證據**逐byte／SHA封存並獨立複核；三分鐘監測 **17次／最大181秒**。7個最終worker／watcher、10個先前worker／watcher與全程monitor均退出。

## 剩餘與界線

H01..H11與修正收於2026-10-22-healing-cut-errors.md，總登錄E1477。仍需全隊經濟補償、更多目標公告、持續引導／特殊技能幾何與動畫、召喚複本生命期與建築／場景美術。原MVP33/44與11項外部ACK／LAN／OS DPI／效能缺證維持；沒有新增真網路對局、重設配額、commit／push或omfx維護。

證據索引：../acceptance/2026-10-22-healing-cut/index.json。


## 2026-10-22 目標經濟補償（最新）

# 2026-10-22 目標經濟補償最終驗收

OpenSpec complete-moba-objective-comeback **5/5完成**。依整體17系統盤點補上全隊經濟補償，沿正式玩法實作並完成單機接受。

## 實作

共用 ObjectiveComebackRules 由 Lua 提供，包含開始時間、啟用與退出經濟差、塔與史詩每人額外金、每次啟用總池，整數有界／未知欄位拒絕。全零保留舊設定停用；規則進內容hash與compiled catalog／hot reload重建守門，正式server／role match設定接入。core明確依賴既有本機內容模型，compiled-only也可使用同一規則，不更動script ABI。

正式訓練180秒後，累計實得差1500啟用、750退出；敵方lane tower每人100、具目標獎勵的史詩根怪每人125，每次池1000。這是專案訓練平衡，不宣稱特定LOL版本。

隊伍收益使用u64飽和ledger；擊殺／助攻／尾刀／野怪／建築／首塔／首殺／塔皮／被動金統一經credit_moba_player，一般GainGold正向實付也記錄。購買／賣出／退款不改累計，死亡保留，飽和錢包只計實付，XP0避免無效成長工作。已認證死亡錢包可取得補償；清空entity但無合法respawn身分不得領取。

資格於tick開始依上一批提交收益鎖定；兩道門檻避免抖動，池耗盡但差距仍大時不自動補滿。塔／史詩致死路徑只記一次claim，tick結束依tick／完整entity生命身分排序，等分且餘數捨去／無足額平均份額則收池。保守扣除已提交份額，即使錢包飽和也不讓同批後續claim越過總額上限。普通野怪、匿名、友軍與強制退休不製造補償；pause不推進。終局清除資格與待發claim。

公開native Buff objective_comeback追加於目錄末尾，既有ID不位移；正式快照／既有ABI與Unreal owned list顯示「目標補償啟用（持續生效）」，不帶戰鬥屬性，復活重新同步，退出／池耗盡／死亡／終局按生命期移除。完整ledger／規則／claim納入既有replay digest。

## 集中驗收

Rust最終 **1788通過／10既有ignored／0失敗**；template75、core902、codegen58、base266、backend190／1、bridge132／1、client150／8、ABI15，含generation九工作均成功退休。新增內容案例涵蓋界線／型別／未知欄位／legacy停用／hash與重建。核心涵蓋一般GainGold、spending不影響、飽和、死亡／重生、時間／雙門檻／pause／耗盡不重開／終局、逆序多目標／重複claim／死者平分、實際tower與epic致死與普通／匿名／強制退休排除。

正式60Hz使用generated單體技能擊殺英雄形成300實得差，清現金不改資格，遊戲時間後啟用敵隊補償，實際重生與永久public snapshot、generated技能摧塔取得100額外金、pause凍結與終局清Buff。整場雙map／雙隊filtered lifecycle及既有技能／塔／野怪／兵線一併回歸。

UE5.8 build-v1與正式後端build-v1成功，Lua三契約成功。WorldV1 **41成功**，0 Failed／NotRun；新增ObjectiveComebackStatus驗證generated目錄與原生owned list顯示、owner隔離、舊generation拒絕與完整缺項退休。既有警告保留。兩map各74影格／6片段，兩map、首殺、塔皮與護盾五張原生圖片均目視，屬回歸；不宣稱新增補償專用真連線畫面。

V1因core缺直接內容模型依賴而編譯失敗；九工作退出後修正。V2 core全通過、base新fixture誤將英雄TAttack helper套塔而失敗；全批與world均退出後，只修該Rust測試，再完整重跑Rust，正式生產碼、UE與後端產物保留。失敗原始log與receipt全部封存，不加總不同批通過數。

**3367來源／資產**接受前後集合、SHA／size／mtime一致，七項DLL staging與import library另驗。**88份證據**逐byte／SHA封存並獨立複核，三分鐘監測 **11次／最大180秒**；7最終worker／watcher與先前失敗cohort、全程monitor均退出。

## 剩餘

錯誤紀錄：2026-10-22-objective-comeback-errors.md，總登錄E1478。仍有更多目標公告／平衡資料、持續引導／特殊技能幾何與動畫、召喚完整複本生命期、建築／場景美術與外部ACK／LAN／OS DPI／效能接受。原MVP33/44與11外部缺證不變。沒有新增真網路對局、重設配額、commit／push或omfx維護。

證據索引：../acceptance/2026-10-22-objective-comeback/index.json。


## 2026-10-22 權威目標公告（最新）

# 2026-10-22 權威目標公告最終驗收

OpenSpec complete-moba-objective-notices **5/5完成**。接續17系統盤點，補上可單機驗證的建築與史詩目標公告。

## 實作

lane tower／inhibitor／nexus tower／具objective rewards的epic根怪沿committed lethal credit與既有每生命退休守門產生claim；不靠視野中的實體消失猜測。普通野怪、匿名、友軍、重複傷害與強制退休不製造敵方成功。新生命重生可再次公告。

tick結束依tick／完整entity生命身分排序，現播公告不被打斷，最多七個最近待播；每則四秒scaled遊戲時間，pause凍結、到期換下一則、空值與終局清除。狀態與序號納入replay digest，沒有英雄／營地名稱特判。

public team／kind／sequence／remaining_raw四元組跨filtered public events、protobuf optional ObjectiveNoticePresentation、CABI34 OmObjectiveNoticeHud與Unreal typed HUD。checked-in proto fallback由既有OMOBA_UPDATE_PROTO_FALLBACK建置入口更新。protobuf保留schema4與首殺欄位的相容性；CABI明確從33升34，Rust／C++ header smoke驗證24-byte／8-byte alignment與各offset，既有版本守門拒絕舊bridge。

client檢查完整／唯一／public team／種類／正序號與4秒內剩時，包含同id但截短payload的重複拒絕；非法objective notice只清該公告，不清整個HUD。Unreal再次檢查非法Blueprint metadata，實際native match strip顯示藍／紅隊摧毀防禦塔、兵營、基地防禦塔或奪取史詩目標；下一空樣本清除舊文字。使用當前權威狀態，沒有一次性音效重播。

## 集中驗收

RustV2 **1794通過／10既有ignored／0失敗**：template75、core905、codegen58、base267、backend190／1、bridge133／1、client151／8、ABI15，含generation九工作全部成功退休。

新core驗證兩隊合法敵方外塔逆序結算一致、重複致死只一次、現播不中斷、最多七待播、pause／到期換序／耗盡／終局與digest、各建築role、普通野怪／匿名／友軍／強制退休排除與實際epic致死。client涵蓋完整／缺項／重複／截短／錯scope／界線／雙隊與codec；bridge驗證typed資料跨實際frame slot到ABI34，非法／缺項／終局只清公告而保留HUD。

正式60Hz由兩名英雄在同tick用generated單體技能實際摧塔與擊殺epic，公告先藍隊外塔後紅隊史詩，pause保持四元組，四秒後換下一則，再到期與終局清空。整場雙map／雙隊filtered lifecycle、既有塔／兵線／野怪／技能一起回歸。

UE5.8 build-v2與正式後端build-v2成功，Lua三契約成功。WorldV2 **42成功**，0 Failed／NotRun，4既有警告保留；新ObjectiveNotice驗證所有類別、非法／終局拒絕、實際Widget／Slate顯示與下一空樣本清除，專用原生目標公告PNG已目視。兩map各74影格／6片段及五張既有回歸圖片也均目視；六張圖片屬本機原生fixture，非新增真連線接受。

V1 core904通過／1新fixture把外塔改NexusTower而被真實開門保護擋住；bridge新fixture猜測不存在的latest_snapshot欄位，world在map export前置編譯失敗，沒有宣稱V1 UE測試完成。全部consumer／watcher退休後改用兩隊合法外塔及既有frame_slots／published_slot，完成V2整批；失敗原始log／receipt另封存，不加總多批數字。

**3369來源／資產**接受前後集合、SHA／size／mtime一致；七項DLL staging與import library另驗。**87份證據**逐byte／SHA封存並獨立複核；全程監測 **12次**，合併11項工作監測的最大檢查間隔 **181秒**（全程單一監測因排程抖動最大183秒，原始保留）。7最終worker／watcher、4先前worker／watcher及全程monitor均退出。

## 剩餘

N01..N05見2026-10-22-objective-notice-errors.md，總登錄E1479。仍有角色施法動畫／持續引導／特殊技能幾何、召喚完整複本生命期、目標專屬名稱／音效與平衡、建築／場景美術，以及11外部ACK／LAN／OS DPI／效能缺證。原MVP33/44維持；沒有新增真網路對局、quota reset、commit／push或omfx維護。

證據索引：../acceptance/2026-10-22-objective-notice/index.json。


## 2026-10-22 權威施法準備動作（最新）

# 2026-10-22 權威施法準備動作最終驗收

OpenSpec complete-moba-cast-preparation-animation **5/5完成**。接續17系統全遊戲盤點，本批補可見英雄施法前搖動作及中斷生命期。

## 實作

實際AbilityWindups在outcome與buff timer後同步native-only cast_preparation公開Buff，不從本機瞄準、動畫名稱或cooldown推論。每次準備帶有有界Q10 total／remaining及保留int64精度的十進位instance字串；後置檢查取消遲到的控制、移動、rank變更及死亡，終局清理。僅遍歷有效windup與已索引狀態，沒有新增全體entity逐frame掃描。

正式BuffSnapshot沿既有ABI34 active buff傳至Unreal typed FrameState。公開投影及Bridge仍使用完整entity generation與既有視野／來源排序；解碼明確檢查JSON型別、大小、數值界線、十進位overflow及合法剩時，拒絕非英雄、死亡、重複或非法資料。

角色沿既有cast資產對齊權威前搖進度及播放速度。準備優先於walk／hit；死亡、respawn、recall仍較優先。取消恢復最新移動姿態，成功release清準備後啟動獨立瞬時動作，退休instance不可復播。pause停播，snapshot延遲時保持cast末端；隱藏清快取，重新可見須收到新的有效權威frame才可恢復當前進度。epoch／pool generation重設沿既有守門。

不改技能魔力／冷卻／傷害或召喚效果交易，沒有逐英雄特判，沒有新增美術資產。

## 集中驗收

最終RustV2 **1799通過／10既有ignored／0失敗**：template75、core908、codegen58、base268、backend190／1、bridge134／1、client151／8、ABI15；含generation九工作全部成功退休。

新增核心驗證精確第一tick、後續倒數／pause、停止／控制／移動／rank／死亡、未admit view／終局／換代退休。正式Production60Hz generated saika_reinforcements驗證512 raw起始、pause凍結、Stop／stun取消不扣魔力、重施法instance遞增、30tick後僅產生兩個召喚物並進入cooldown。Bridge實際ABI payload保留最大int64 identity，純倒數以active snapshot更新且不重發cue，generation替換與移除退休。

UE5.8 build-v2成功（UBT **5.55秒**），正式後端build-v1成功，Lua三契約成功。WorldV2 **43成功**，0 Failed／NotRun；4既有清理warnings保留。CastPreparationAnimation檢查有界解碼、public ABI、實際skeletal animation資產／25%與50%cursor、移動與hit優先、pause／末端hold、取消／新instance／隱藏／release／死亡／respawn／reset。另回歸兩map的地形、塔、兵線、野怪與既有HUD／技能提示；兩map各74影格／6片段，六張回歸PNG已目視。本批沒有另宣稱施法動作專用截圖或真連線接受。

V1 World42成功／1失敗：Unreal TryGetStringField會將數字轉字串；先檢查EJson::String後讀值修正。RustV1新增bridge fixture將純倒數誤期為BuffUpdated，改驗既有active snapshot節流契約；V1來源檢查因Rust成功守門拒絕。所有consumer／watcher退休後才修來源，V2整批通過。原始失敗證據保留，不加總兩批數字。

**3372來源／資產**接受前後集合、SHA／size／mtime一致，七項DLL與import library另驗。**100份證據**逐byte／SHA封存並獨立複核；全程監測 **13次**、最大間隔 **171秒**，7最終及5先前worker／watcher與全程monitor皆退出。

## 剩餘

持續引導／脈衝、特殊線／扇形／多段／導引技能幾何與提示，召喚完整可見複本生命期及更多控制，目標專屬名稱／音效與平衡，英雄／建築／地圖美術仍有工作。11項外部ACK／LAN／實際OS DPI／固定效能證據維持待驗，MVP33/44不變；未新增真網路對局、quota reset、commit／push或omfx維護。

P01..P04、T01..T02見2026-10-22-cast-preparation-errors.md；總登錄E1480。完整證據索引：../acceptance/2026-10-22-cast-preparation/index.json。


## 2026-10-22 內容驅動施法受傷中斷（最新）

# 2026-10-22 內容驅動施法受傷中斷最終驗收

OpenSpec complete-moba-cast-damage-interruption **5/5完成**。接續全遊戲17系統盤點，本批完成單機可驗證的逐rank受傷中斷施法與正式tooltip。

## 實作

AbilityLevel新增嚴格cast_interrupt_on_damage布林值，缺值false；啟用時要求cast_time在Q10可表示的1/1024..60秒。Rust與固定Lua generator拒絕字串／數字布林及無前搖政策。靜態模板、runtime catalog、script AbilityDef JSON及Unreal generated metadata逐rank保留；content hash包含政策，compiled比較與reload shape要求所有peer重建，未變更ABI34。

admission將政策複製進每次Windup，既有replay digest納入。一般／投射物共用傷害結算與腳本直接傷害在shield之後，只在正HP debit且仍pending／opt-in時沿通用cancel清除，沒有英雄名稱特判。完全護盾吸收、零傷害、治療或未啟用政策不觸發中斷；已release效果不回滾。取消結果沿既有owner進度、公開準備Buff與角色動作退休。

雜賀援軍四rank啟用，其他未設定技能保留預設。Unreal正式Bridge依rank傳typed HUD布林，真Widget tooltip顯示HP傷害取消及完全護盾例外；未知形狀、未學習、零前搖及disabled rank不繼承舊提示。

## 集中驗收

RustV1 **1803通過／10既有ignored／0失敗**：template76、core909、codegen59、base269、backend190／1、bridge134／1、client151／8、ABI15，含generation九工作成功退休。

新增template測試實際呼叫固定Lua，驗證與Rust一致接受／拒絕；省略政策false、逐rankhash、runtime／compiled與reload rebuild守門。core涵蓋預設／opt-in、取消公開marker／結果、release不可回滾與digest。codegen以任意技能驗證true／false rank順序及typed metadata，未加入英雄特判。

正式Production60Hz實際施放generated援軍，比較一般／腳本直接HP傷害、完全shield兩路、部分shield、零傷害及治療七組；中斷不扣魔力或進cooldown，未中斷只產生兩個召喚物並提交cooldown，後續傷害不回滾。既有召喚生成／戰鬥／退休、雙map／雙隊filtered整場生命周期及塔／兵／野怪一起回歸。

UE5.8 build-v2成功（UBT **6.43秒**），正式後端build-v1及Lua三契約成功。WorldV2 **44成功**，0 Failed／NotRun，4既有清理warnings保留；新原生測試驗正式四rank metadata、真Widget提示與未知形狀／未學習／零前搖／disabled負例；既有43項動畫、技能提示、HUD與地圖回歸。兩map各74影格／6片段，六張回歸PNG已目視；未宣稱新增真連線或施法受傷專用截圖。

V1 World43成功／1失敗是新fixture漏設已知Point形狀，受到既有tooltip守門。等待全部consumer／watcher及來源檢查退出後，只修原生測試；V2前後來源基準逐檔比較，唯一變動為OmPointAreaPreviewTests.cpp，保留未受影響的完整RustV1結果。UE／World與來源guard重做V2，原始失敗log／report／receipt封存，不加總兩批數字。

**3372來源／資產**V2接受前後集合、SHA／size／mtime一致；七項DLL與import library另驗。**92份證據**逐byte／SHA封存並獨立複核。全程健康監測 **10次**、最大間隔 **171秒**；7最終及4先前worker／watcher與全程monitor全部退出。

## 剩餘

持續引導／脈衝、特殊線／扇形／多段／導引幾何與提示；召喚完整可見複本生命期、更多控制與競爭落點原子回滾；目標專屬名稱／音效、更多道具／召喚師與平衡資料、英雄／建築／場景美術仍有工作。MVP33/44、11項外部ACK／LAN／實際OS DPI／固定效能證據維持待驗，原生44測試成功不代表44個MVP父項完成。

P01..P03／T01見2026-10-22-cast-interrupt-errors.md，總登錄E1481。零新增真網路對局／quota reset／commit／push，omfx未維護。

證據索引：../acceptance/2026-10-22-cast-interrupt/index.json。


## 2026-10-22 召喚物公開身分與原生呈現（最新）

# 2026-10-22 召喚物公開身分與原生呈現最終驗收

OpenSpec complete-moba-summon-presentation **5/5完成**。接續全遊戲盤點，完成單機可驗證的召喚物種類、typed catalog、公開呈現與原生動畫生命期。

## 實作

full／fast snapshot 明確辨識 UnitType::Summon；公開WaveB用render.kind=4、filtered只重建公開UnitType與內容身分，不加入私有SummonedUnit／AI。Bridge沿既有ABI34 EntitySummon／CatalogSummon，解決同名小兵／召喚物目錄混用，未知內容仍保留召喚物種類並走fallback。25-byte render、protobuf及CABI結構大小不變。無Unit storage的精簡World仍可擷取快照。

召喚物加入通用移動、腳本attack_phase_fx、生命值與動畫投影。saika_gunner使用既有ranger rig／材質／動畫，隔離視覺設定、高度65世界單位，生成原生summon class；保留原ID／400HP／技能與傷害。這是可見原生角色接線，專屬槍械與角色美術仍待製作。

## 集中驗收

RustV5 **1807通過／10既有ignored／0失敗**：template76、core910、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15；含generation九工作全部成功退出。

新增核心測試驗full／fast與WaveB公開身分一致、不洩漏私有腳本／AI、同ID一般單位不誤分類。Bridge真正公開snapshot經通用projection到CABI，驗typed catalog、移動、隱藏／重現、退休／新generation與未知內容。codegen驗實際內容rig、身高、idle／move／attack／death與正確class。正式Production60Hz技能生成兩個owned summon、提交視野後實際開火與公開attack cue、pause及到期退休；其餘塔／兵／野怪／雙map生命周期一起回歸。

UE5.8 build-v2成功（UBT **5.55秒**），正式後端build-v1及Lua三契約成功。WorldV2 **45成功**、0 Failed／NotRun，4既有清理warnings保留。新增測試由真Bridge生成實際summon actor，驗名牌、已載入rig與精確動畫clip、站立／移動／攻擊／pause／隱藏／重現／死亡／移除／新generation。既有44項技能提示、角色、HUD與地圖一起回歸。兩map各74影格／6片段；7張實際PNG已目視，包含新召喚物原生GPU截圖。未新增真網路對局，也未冒稱完整可見戰鬥複本接受。

RustV1漏顯式import，V2精簡World儲存區／fixture資源與公開視野更新，V3新測試誤讀MobaMatch專用動畫欄；WorldV1抽象基底生成造成空指標；V4修復工具防覆寫保護導致未建立runner。皆記錄根因、等待消費者退出再修復，保留失敗receipt與原始logs，不加總失敗與成功數字。最終完整RustV5與WorldV2重新接受。

**3373來源／資產**最終接受前後集合、SHA／size／mtime一致；七項DLL與import library另驗。**159份證據**逐byte／SHA封存並獨立複核。健康監測 **14次**、最大間隔 **171秒**，7最終與7先前worker／watcher、全程monitor全部退出。

## 剩餘

完整combat replica生命周期、退休來源on-hit callback、競爭落點整組原子回滾、更多召喚控制、持續引導／脈衝與特殊技能幾何仍待實作。更多英雄／道具／召喚師、目標專屬名稱音效、建築／場景美術與平衡內容保留。MVP仍33/44，11個外部ACK／LAN／實際OS DPI／固定效能接受不以本機World代替。

錯誤詳見2026-10-22-summon-presentation-errors.md，總登錄E1482。零新增真網路對局／quota reset／commit／push；omfx未維護。

證據索引：../acceptance/2026-10-22-summon-presentation/index.json。


## 2026-10-22 共用召喚落點預約（最新）

# 2026-10-22 共用召喚落點預約最終驗收

OpenSpec complete-moba-summon-reservation-ledger **5/5完成**。接續全遊戲盤點，補上單機權威可驗證的跨event／on_tick召喚位置承諾。

## 實作

原先每個adapter只檢查ECS與自己的pending outcomes，不同技能事件或平行hook canonical重放仍可能向腳本承諾重疊位置，commit才移開。新增World範圍PlacementReservations，以完整Entity含generation記錄team、位置與半徑，Arc／Mutex與BTreeMap保持受控共享與固定排序。只在既有canonical serial allocation建立預約，speculative平行hook不分配；同team共享，敵隊預約不影響己方。每World4096、每invocation128上限。

finish將預約交給outcomes；discard_cast／Drop立即撤銷自己的預約並刪除保留handle。正式summon生成時退休對應預約，沿既有owner／地形／友軍體積檢查提交；新cache清除已退休generation。StateInitializer正式World安裝共享資源，精簡測試World保留cache內fallback。無新增協定／ABI／Lua內容格式變更。

## 集中驗收

RustV1 **1809通過／10既有ignored／0失敗**：template76、core912、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15；含內容generation九工作全部成功退出。

新增跨cache同team位置分離、取消後可重用位置、Drop撤銷、敵隊隔離、退休generation清除及4096容量拒絕。既有8單位canonical平行hook測試新增兩兩至少40世界單位間距，真的process_outcomes後每個實際Pos與腳本取得的位置完全一致，ledger清空，並保留returned handle後續facing命令。正式60Hz召喚生成／戰鬥／退休、雙map生命周期及塔／兵／野怪一起回歸。

UE5.8 build-v1成功（UBT **3.56秒**）、正式後端build-v1與Lua三契約成功。既有World **45成功**、0 Failed／NotRun，4既有清理warnings保留。地圖、召喚物動畫、技能提示、HUD與資源生命期回歸；兩map各74影格／6片段，7張PNG已目視。本輪新增功能的直接證據是權威ECS實際settlement測試，World與PNG是消費端回歸，沒有冒稱新增專用畫面或真網路對局。

**3373來源／資產**接受前後集合、SHA／size／mtime一致；七DLL與import library另驗。**72份證據**逐byte／SHA封存並獨立複核。健康監測 **6次**、最大間隔 **171秒**；7組worker／watcher與全程monitor全部退出。編譯／功能接受沒有失敗重跑；路徑盤點及Saved helper識別字問題已於啟動World前修正並記錄。

## 剩餘

任意中間outcome改變世界／退休owner後的整組施法commit原子回滾仍需額外交易設計，本次共用預約不冒稱該項完成。退休來源callback需完整發射時上下文，不以script ID猜測；完整combat replica、持續引導／特殊技能幾何、更多原創英雄道具召喚師、美術與音效仍待擴充。MVP33/44、11項外部ACK／LAN／實際DPI／固定效能條件維持pending。

錯誤與決策見2026-10-22-summon-ledger-errors.md，總登錄E1483。零新增真網路對局／quota reset／commit／push；omfx未維護。

證據索引：../acceptance/2026-10-22-summon-ledger/index.json。


## 2026-10-22 指定單位技能射程提示（最新）

# 2026-10-22 指定單位技能射程提示最終驗收

OpenSpec complete-moba-unit-cast-range **5/5完成**。接續全遊戲盤點，補上單機可驗證的指定單位技能射程與當前公開資料生命周期。

## 實作

Bridge新增只讀TryGetPublishedCastUnitPositions，以明確player、source hero與source／target完整generation，從同一份目前PublishedDirectTargets讀取存活可見中心。owner不一致、來源死亡／替換、目標隱藏／死亡／退休／新generation、無效座標皆拒絕，輸出歸零。沒有Actor掃描或last-known推測。

Owner僅保存目前marker的Entity，初始化與pointer move走共用SetCastMarkerSample，支援world與disclosed card；普通UI與退休清空。繪製時經嚴格反射契約即時讀公開中心，依2D距離顯示射程內外、數字與顏色，目標marker使用單位中心而非cursor。沒有有效目標時距離未知，仍保留原有施法者射程圈；未知range、未學習／冷卻／魔力提示優先。點地技能既有落點／半徑維持。通用於已有entity預覽的技能、道具與召喚師，不改命令提交或權威admission，不把提示當ACK。

OmRuntime保持不依賴OmGenerated，不建立全場單位座標cache，也不新增每entity每幀操作。Bridge API與新enum只在Unreal內部，未變更CABI34／protobuf。

## 建置完整性

V1 build成功但UBT來源快取未收集新增.cpp，World案例完整性守門發現只有45項而拒絕。正式Lua ordinary／bootstrap與既有restart旗標資料加入UE5.8原始碼明確支援的-NoUBTMakefiles，讓每次建置發現來源集合；保留NoEngineChanges，不刪cache、不改引擎或全域設定。Lua規劃契約與restart測試驗兩種入口一致。

BuildV2真的編譯新測試後，抓到fixture讀protected縮放及錯用seq欄位；等所有消費者退出後，僅修測試CPP為UPROPERTY反射與CABI實際sequence。BuildV5成功，WorldV4實際找到46項。WorldV2／V3進一步定位：direct公開查詢成功，只有未初始化Actor的測試World略過ProcessEvent。沿UE5.8 Actor.cpp與UObject/Script.h，fixture使用FEditorScriptExecutionGuard限定作用域，保持正式查詢守門；WorldV4真反射與完整生命周期成功。來源guard新增restart crate，RustV2所有guarded Rust／內容輸入mtime均早於該owned工作起點；後續僅改原生fixture，因此保留已通過的RustV2，不混加數字。V1不完整report／log與BuildV2錯誤均封存。

## 集中驗收

RustV2 **1818通過／10既有ignored／0失敗**：template76、core912、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15、restart9；含generation十工作成功退出。四Lua契約成功，正式後端build-v1成功，UE5.8 build-v5成功（UBT **5.67秒**）。

WorldV4 **46成功**、0 Failed／NotRun，4既有清理warnings保留。新增實際World公開frame→Bridge→owner反射→preview測試，驗source／target中心與cursor分離、舊本機hero座標不影響、精確射程邊界、移動即時超距、未知range、卡片、錯owner／source generation、目標隱藏／死亡／新generation、source死亡／非hero、目標缺席、UI清空與owner退休，並保留未學習／冷卻／魔力、legacy candidate與NaN range守門。

既有45項地圖、召喚動畫、技能提示、HUD與資源生命周期一起回歸。兩map各74影格／6片段；7張既有PNG已目視，作消費端回歸，沒有冒稱新增射程專用截圖或真連線ACK。

**3378來源／資產**最終接受前後集合、SHA／size／mtime一致；七DLL與import library另驗。**150份證據**逐byte／SHA封存並獨立複核。健康監測 **12次**、最大間隔 **171秒**，7最終及11先前worker／watcher與全程monitor皆退出。

## 剩餘

持續引導／特殊技能幾何、完整combat replica、退休來源callback完整上下文、召喚整組commit原子回滾、更多原創英雄／道具／召喚師、美術與音效仍待擴充。MVP33/44、11項外部ACK／LAN／實際OS DPI／固定效能條件維持pending，本機射程提示不替代權威接受。

錯誤與決策見2026-10-22-unit-cast-range-errors.md，總登錄E1484。零新增真網路對局／quota reset／commit／push，omfx未維護。

證據索引：../acceptance/2026-10-22-unit-cast-range/index.json。


## 2026-10-22 指定單位技能陣營提示（最新）

# 2026-10-22 指定單位技能陣營提示最終驗收

OpenSpec complete-moba-unit-cast-allegiance **5/5完成**。接續全遊戲盤點，補上單機可驗證的QWER指定單位技能即時條件提示。

## 實作

Bridge新增只讀TryEvaluatePublishedCastTarget，沿目前玩家、source hero、source／target完整generation及公開存活中心守門，重用正式命令提交的IsPublishedLivingAlly／IsPublishedLegalHostile。未符合敵友需求顯示「目標陣營不符」，敵方缺少目前可攻擊條件顯示「目標目前不可施放」，條件未知顯示「目標條件尚未確認」。非敵對不直接等於友軍；中立關係以正式現有規則判定。未知metadata不猜測。

Owner經嚴格六參數反射契約讀目前公開表；目標變更陣營、失去可攻擊條件不需移動游標即可更新。目標死亡／隱藏／generation替換、source失效／owner退休即失效。未學習、冷卻、魔力、缺少目標保留優先，已通過條件繼續顯示當前中心射程內外。OmRuntime不依賴OmGenerated，不掃Actor、不建立全場cache、不新增每entity每幀操作。不改命令提交、CABI34或protobuf，也不把預覽當作ACK。未具陣營metadata的道具／召喚師維持既有射程提示，不聲稱本批新增其陣營檢查。

## 集中驗收

RustV1 **1818通過／10既有ignored／0失敗**：template76、core912、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15、restart9；含generation十工作成功。四Lua契約成功，正式後端build-v1成功，UE5.8 build-v1成功（UBT **58.12秒**）。正式-NoUBTMakefiles入口確實納入新增CPP。

WorldV1 **47成功**、0 Failed／NotRun。新增真實World公開frame→Bridge→owner反射→feedback測試：敵友需求、同目標關係即時改變、自用、中立不推測、敵方失去可攻擊條件、未知需求、owner不符、目標死亡／隱藏／新generation、source死亡與owner退休，以及提示優先序與legacy行為。測試World限定FEditorScriptExecutionGuard，不改正式生命周期。

既有46項地圖／召喚／技能／HUD／資源回歸一起通過；4既有清理warnings保留。兩張地圖各74影格／6片段，7張既有PNG已目視，屬消費端回歸，沒有新增功能專用截圖或真連線ACK宣稱。

**3379來源／資產**在接受前後集合、SHA／size／mtime一致；七DLL與import library另驗。**75份證據**逐byte／SHA封存並獨立複核。健康監測 **6次**、最大間隔 **171秒**，7組worker／watcher及全程monitor皆退出。

## 剩餘與錯誤

持續引導／特殊幾何、完整combat replica、退休來源callback上下文、召喚整組commit原子回滾、更多原創英雄／道具／召喚師、美術音效仍待擴充。MVP33/44，11項外部ACK／LAN／實際OS DPI／固定效能條件維持pending。本批沒有宣稱全遊戲已與LOL一致。

錯誤MD：2026-10-22-unit-cast-allegiance-errors.md；總登錄E1485。Saved helper初版遮蔽Lua pairs，於啟動驗收前改名修復；未改正式工作流。零新增真網路對局、quota reset、commit／push，omfx未維護。

證據索引：../acceptance/2026-10-22-unit-cast-allegiance/index.json。


## 2026-10-22 控制效果施法提示（最新）

# 2026-10-22 控制效果施法提示最終驗收

OpenSpec complete-moba-cast-control-feedback **5/5完成**。接續全遊戲盤點，補上目前公開控制效果對QWER預覽的原因與時間提示。

## 實作

Bridge新增TryGetPublishedCastControl，從同一份完整公開frame有界掃描最多4096項Buff，僅保存目前source摘要。依typed catalog識別stun／silence；root僅禁移動，不誤當技能阻擋。暈眩優先沉默，同類取最長公開剩餘時間，永久效果不顯示負數倒數。不使用UI64項截斷清單，不掃Actor、不建立全場cache；查詢O(1)。

查詢沿正式公開source中心的玩家、hero、存活與完整generation守門。替換／清空／revoke／退休即失效；非完整presentation、未知metadata、malformed當前控制資料保持未知。owner嚴格五參數反射讀取，point／unit的QWER預覽顯示「暈眩中，無法施放技能」或「沉默中，無法施放技能」與當前公開剩餘秒數；未知顯示「施法控制狀態尚未確認」。未開啟預覽的instant self不宣稱新增浮層。

未學習／冷卻／魔力保留優先；無控制阻擋時繼續既有目標條件與射程提示。道具／召喚師未硬套技能控制規則。沒有使用牆鐘推估倒數，也沒有改命令提交、後端裁決、CABI34或protobuf；提示不代表ACK或未來釋放必然成功。OmRuntime保持不依賴OmGenerated。

## 集中驗收

RustV1 **1818通過／10既有ignored／0失敗**：template76、core912、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15、restart9；含generation十工作成功。四Lua契約成功，正式後端build-v1成功，UE5.8 build-v1成功（UBT **54.42秒**）；新CPP確實編譯。

WorldV1 **48成功**、0 Failed／NotRun，保留4個清理warnings。新增真World frame→Bridge→owner反射→feedback測試驗空快照、root、silence、stun優先、同類最長時間、永久效果、第66項控制、他generation、無效時間／instance／metadata、快照替換移除、錯owner、死亡／source revoke／退休／空presentation、可讀標籤與原有提示優先序。fixture的FEditorScriptExecutionGuard限定測試scope，沒有放寬正式生命周期。

既有47項技能、地圖、召喚、HUD與資源回歸一起通過。兩張地圖各74影格／6片段，七張既有PNG已目視，僅作消費端回歸，不冒稱本功能專用截圖或網路ACK。

**3380來源／資產**接受前後集合、SHA／size／mtime一致；七DLL與import library另驗。**75份證據**逐byte／SHA封存並獨立複核。健康監測 **5次**、最大間隔 **171秒**，7組worker／watcher及全程monitor皆退出。

## 剩餘

持續引導／特殊幾何、完整combat replica、退休來源callback上下文、召喚整組commit原子回滾，以及更多英雄／道具／召喚師、美術音效仍待擴充。MVP33/44，11項外部ACK／LAN／實際OS DPI／固定效能接受維持pending，沒有宣稱全遊戲已與LOL一致。

錯誤與決策：2026-10-22-cast-control-errors.md，總登錄E1486。P01 Windows路徑glob、P02宣告入口與BuffIdName未知ID fallback、P03 Saved封存helper改名路徑均已修正。零新增真網路對局、quota reset、commit／push；omfx未維護。

證據索引：../acceptance/2026-10-22-cast-control/index.json。


## 2026-10-22 野怪返回名牌（最新）

# 2026-10-22 野怪返回名牌最終驗收

OpenSpec complete-moba-jungle-return-presentation **5/5完成**。接續全遊戲盤點，補可見野怪返回／重置期間的可讀性。

## 實作

權威commit hook於本tick outcomes與勝負裁決後，為存活camp bodies同步jungle_return非戰鬥狀態。mode1取自returning且仍可傷害，mode2取自returning且invulnerable；根怪、members與死亡分裂成員共用根camp正式規則。重接仇恨、回家、死亡／lethal pending、finished清除，pause保持已提交狀態。使用內部PERMANENT_BUFF_DURATION，公開snapshot轉-1 sentinel，不會當作倒數過期；只在狀態改變時替換。

內容catalog追加ID22，既有stun1／silence9／cast_preparation21保留。payload僅含mode，不公開aggro target、patience、reset或respawn timer，不改既有傷害、回血、導航、獎勵或重生規則。既有entity可見性投影與active_buffs承載，CABI34／protobuf不變。

Unreal嚴格有界解碼，驗typed catalog、完整generation與living creep；重複、未知、越界或malformed資料清除。Bridge→frame payload→Actor→名牌顯示「正在返回 · 可重新交戰」／「返回重置 · 無法受傷」，尊重health偏好與文字縮放。沿原有SameDisplay節流，沒有新增每entity每frame Slate寫入。control-only frame維持已公開狀態；完整replacement缺席、死亡、reset/rebind清空；hide時清除返回cache，reveal不可重播，fresh frame才恢復。

## 集中驗收

RustV1 **1820通過／10既有ignored／0失敗**：template76、core914、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15、restart9；含generation十工作成功。新增兩項core測試驗根怪／members、永久duration、無silence/root副作用、pause、實際reaggro、death／finished；實際leash耗盡→soft→hard→home完成。四Lua契約成功，正式後端build-v1成功，UE5.8 build-v1成功（UBT **86.31秒**）。

WorldV1 **49成功**、0 Failed／NotRun，保留4個既有清理warnings。新增嚴格JSON／sentinel、typed公開ABI、重複、錯generation、非creep、死亡、字串bounds、不可見entity、實際Frame→Bridge→Actor→名牌、節流、偏好、control-only／replacement、hide/reveal與退休測試。新返回名牌實際Slate PNG已目視，另保留七張既有消費端PNG。兩張地圖各74影格／6片段；本機synthetic消費端不當作真網路ACK。

**3383來源／資產**接受前後集合、SHA／size／mtime一致；七DLL與import library另驗。**76份證據**逐byte／SHA封存並獨立複核，8張PNG已目視。健康監測 **7次**、最大間隔 **171秒**，7組worker／watcher及全程monitor皆退出。

## 剩餘與錯誤

持續引導／特殊幾何、完整combat replica、退休來源callback上下文、召喚整組commit原子回滾、更多原創英雄／道具／召喚師、美術音效仍待擴充。MVP33/44，11項外部ACK／LAN／實際OS DPI／固定效能接受維持pending，未宣稱全遊戲已與LOL一致。

錯誤與決策：2026-10-22-jungle-return-errors.md；總登錄E1487。P01修正永久Buff內部與公開sentinel的差異；P02誤讀非必要模組路徑改依實際import；P03補hide/reveal舊狀態清理，修改時無原生消費者且generation已退出，Rust輸入未變。所有問題保留MD，沒有臨時關閉生命周期或可見性守門。

零新增真網路對局、quota reset、commit／push，omfx未維護。證據索引：../acceptance/2026-10-22-jungle-return/index.json。


## 2026-10-22 可見單位控制效果名牌（最新）

# 2026-10-22 可見單位控制效果名牌最終驗收

OpenSpec complete-moba-crowd-control-nameplates **5/5完成**。接續全遊戲盤點，補可見單位的暈眩／定身／沉默名牌。

## 實作

新增FOmCrowdControlState，三種控制各自保存公開剩餘時間。同類多來源取最長，永久優先；主效果優先暈眩、定身、沉默，只用該效果的時間，並標示其他並存效果，避免把長沉默誤當暈眩持續。永久不顯示負數倒數；極大有限時間呈現9999+秒，避免整數轉換溢位。沒有wall-clock推算。

Bridge從最多4096項公開Buff與100000 entity rows建摘要，使用typed catalog canonical ID，不依賴動畫、顯示名稱或UI64項截斷清單。非法duration、重複控制instance、重複entity、死亡、缺席／錯generation皆拒絕；未知Buff不憑空產生控制，沒有公開entity時Buff本身不能揭露狀態。未顯示控制不代表保證可操作。

接線為公開frame→Bridge→typed frame payload→Actor→名牌，沿health偏好與文字縮放；只在主要效果顯示秒數、組合或偏好改變時重畫，次要純倒數不引起Slate寫入。死亡、reset/rebind清空，hide清cache、reveal不可重播，fresh authority才恢復；control-only保留，完整replacement缺席清理。不改後端控制規則、內容catalog、CABI34／protobuf或命令裁決，OmRuntime不依賴OmGenerated。

## 集中驗收

RustV1 **1820通過／10既有ignored／0失敗**：template76、core914、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15、restart9；含generation十工作成功。四Lua契約成功，正式後端build-v1成功，UE5.8 build-v1成功（UBT **101.66秒**）。

WorldV1 **50成功**、0 Failed／NotRun，保留4個既有清理warnings。新增測試涵蓋主要時間與並存效果、root／silence區分、永久／極大時間、NaN／無限／非法負值、同類多來源、重複instance／entity、錯generation、死亡／不可見、輸入上限、名牌偏好／節流／顯示秒數邊界，以及實際Frame→Bridge→Actor→Slate、control-only／完整replacement、hide/reveal與退休。新控制名牌實際Slate PNG已目視；既有返回名牌及七張消費端PNG保留。兩張地圖各74影格／6片段，本機synthetic呈現不當作真網路ACK。

**3386來源／資產**接受前後集合、SHA／size／mtime一致；七DLL與import library另驗。**77份證據**逐byte／SHA封存並獨立複核，9張PNG已目視。健康監測 **6次**、最大間隔 **170秒**，7組worker／watcher及全程monitor皆退出。

## 剩餘與決策

持續引導／特殊幾何、完整combat replica、退休來源callback上下文、召喚整組commit原子回滾、更多英雄／道具／召喚師、美術音效仍待擴充。MVP33/44，11項外部ACK／LAN／實際OS DPI／固定效能接受維持pending，未宣稱全遊戲已與LOL一致。

錯誤與決策記錄：2026-10-22-crowd-control-errors.md；總登錄E1488。P01曾過早讀取尚未完成的地圖截圖，改為等World退休後再目視；本批防錯聚焦獨立控制時間、永久sentinel、顯示整數上限、完整公開清單與生命周期清理。驗收過程無新增建置／測試失敗，不重複前輪路徑glob／改名失誤，也沒有放寬守門。

零新增真網路對局、quota reset、commit／push，omfx未維護。證據索引：../acceptance/2026-10-22-crowd-control/index.json。


## 2026-10-22 選取目標戰鬥資訊卡（最新）

# 2026-10-22 選取目標戰鬥資訊卡最終驗收

OpenSpec complete-moba-selected-unit-combat-details **5/5完成**。接續全遊戲盤點，選取目標資訊卡整合目前公開護盾、控制效果、塔皮與野怪返回狀態。

## 實作

FOmSelectedUnitStatePayload新增typed Shield／CrowdControl／HealthSegments／JungleReturnMode，從同一公開frame既有摘要取得。沿既有player、source／target generation、living／visibility與runtime scope守門；不可受傷的公開單位仍可inspect，選取不等同attack admission。不新增私人stats／敵方技能冷卻查詢，不改後端、內容catalog、CABI34或protobuf。

資訊卡與名牌共用OmShieldLabel、OmCrowdControlLabel、OmHealthSegmentsLabel、OmJungleReturnLabel，維持既有文字、bounded標籤／數字與倒數。非法子狀態獨立省略，不污染HP主卡。HP不變但護盾／控制／塔皮／返回或顯示秒數改變會發布HUD；同顯示秒數保留節流，但Bridge selected getter仍保存最新raw公開值。HUD自身的顯示snapshot只需保持正確文字，不宣稱每fractional秒發布。

UOmPlayerHudOwnerComponent維持原有source／epoch／player檢查並傳遞完整typed payload。取消selection、target死亡／隱藏／generation替換、source conflict及runtime／HUD退休仍清理整張卡。不新增全場cache或每entity每frame Slate更新，OmRuntime不依賴OmGenerated。

## 集中驗收

RustV1 **1820通過／10既有ignored／0失敗**：template76、core914、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15、restart9；含generation十工作成功。四Lua契約成功，正式後端build-v1成功，UE5.8 build-v1成功（UBT **68.51秒**）。

WorldV1 **51成功**、0 Failed／NotRun，保留4個既有清理warnings。新增真公開frame→Bridge selection→實際native HUD測試，驗四項資料、名牌格式一致、malformed子狀態、固定HP時獨立更新盾／控制／返回／塔皮、同秒raw快照、完整缺席、取消選取／重選、世代／source conflict／隱藏／死亡與HUD退休。OmSelectedUnitCard實際Slate截圖已目視，四項狀態及HP完整可讀；原有控制／返回名牌與七張消費端PNG保留。兩張地圖各74影格／6片段；本機synthetic呈現不當作真網路ACK。

**3387來源／資產**接受前後集合、SHA／size／mtime一致；七DLL與import library另驗。**78份證據**逐byte／SHA封存並獨立複核，10張PNG已目視。健康監測 **7次**、最大間隔 **171秒**，7組worker／watcher及全程monitor皆退出。

## 剩餘與錯誤

持續引導／特殊幾何、完整combat replica、退休來源callback上下文、召喚整組commit原子回滾、更多英雄／道具／召喚師、美術音效仍待擴充。MVP33/44，11項外部ACK／LAN／實際OS DPI／固定效能接受維持pending，未宣稱全遊戲已與LOL一致。

錯誤與決策：2026-10-22-selected-combat-errors.md，總登錄E1489。P01猜測選取測試檔名造成讀取不存在路徑，改用實際測試ID與rg定位；後續建置／測試無新增失敗。所有圖片等World receipt成功及worker／watcher退休後才目視；未放寬正式守門。

零新增真網路對局、quota reset、commit／push，omfx未維護。證據索引：../acceptance/2026-10-22-selected-combat/index.json。


## 2026-10-22 彈體來源上下文（最新）

# 2026-10-22 彈體來源上下文最終驗收

OpenSpec complete-projectile-source-context 5/5完成。本批增加正式戰鬥完整度：已發射彈體不再因來源entity退休而跳過腳本命中效果；追加傷害與子彈保持原始MOBA來源歸因。

## 實作與邊界

ProjectileOrigin保存不可變、可共享的每發來源快照，包含腳本身分與唯讀戰鬥資料；不複製整個世界或塔的空間快取。dispatch在精確來源生命退休後才回退查詢，存活來源仍用即時數值，is_alive不偽造。影響的是已承諾彈體的命中回呼，不允許死亡來源重新主動施法／行動。

正式tick把命中與來源一併交給串行腳本派送。追加Damage及ScriptProjectile只在精確來源一致時帶入內部封套，拒絕巢狀／來源不一致的封套，再走原有傷害、observable facts及生成流程。子彈保留generation、origin及清理生命期。退休召喚物回呼的目標查詢保留目標可見性／可攻擊性／LOS守門，已承諾彈體不再被來源退休擋掉。

來源快照buff時間只供該發射上下文使用，不重新套用或延長全局buff。未新增公開敵方私有資料、ABI、protobuf或CABI欄位；沒有英雄／塔名稱特判。

## 集中驗收

最終RustV3：**1825通過／10既有ignored／0失敗**，含template76、core919、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15、restart9，以及generation十工作全部成功。五個新增核心案例包含真飛行／退休與ID重用／read-only資料／追加傷害／子彈繼承及清理、存活來源即時修改、非法封套拒絕、MOBA擊殺獎勵歸因與一次性、退休召喚物目標守門。四Lua契約成功。

正式backend-v3與UE5.8 build-v3成功，UBT **3.49秒**。WorldV3 **51成功／0 Failed／0 NotRun**，共5個warnings（4個既有測試World清理警告，1個UE自動連線探測HTTP逾時）。兩張地圖各74影格／6片段；十張現有功能PNG均在World worker／watcher退出後目視。本批沒有新增Unreal UI功能；圖片與World用於回歸，不拿synthetic viewport當網路ACK。

**3388來源／資產**接受前後集合、SHA／size／mtime一致；七DLL及import library另驗。封存**114份證據**並獨立逐byte／SHA校驗，含先前失敗preflight與過渡版本receipt。健康監測**15次**、最大間隔**171秒**，最終7組worker／watcher及所有過渡工作均退休，全程monitor退出。

## 錯誤與剩餘

錯誤見2026-10-22-projectile-context-errors.md，總登錄E1490。路徑猜測、測試pattern欄位錯位、fixture缺資源、preflight缺exit標記均保留紀錄；未覆寫失敗證據。召喚物來源退休邊界在最終接受前修正，不放寬自動AI或目標守門。

持續引導／特殊幾何、完整combat replica接受、召喚整組commit原子回滾、特殊英雄命中機制、更多英雄／道具／召喚師、美術音效仍待完善。MVP33/44、11項外部ACK／LAN／實際OS DPI／固定效能接受維持pending，未宣稱整個遊戲已等同LOL。零新增真網路對局、quota reset、commit／push；omfx未維護。

證據索引：../acceptance/2026-10-22-projectile-context/index.json。


## 2026-10-22 沿路碰撞衝刺（最新）

# 2026-10-22 沿路碰撞衝刺最終驗收

OpenSpec complete-swept-dash-combat 5/5完成。本批修正fire_dash直接傳送、只傷害終點的玩法缺口，增加可重用的固定點沿路衝刺與單次命中機制。

## 實作

omoba-sim提供有界i128精確capsule判定；包含相切、端點、零長度、斜線、大座標及非法值守門。base_content共用SweptDash先校驗內容預算及來源，依rank射程clamp；透過既有advance_with_collision檢查公開地形／root政策，受阻時最多25次查詢找同一直線最遠可行prefix，避免穿薄牆或軸向滑動。只對實際路徑的中心點capsule內活敵人canonical排序去重，各命中一次；所有預檢完成後才提交位置／朝向與傷害。

fire_dash使用range／dash_width／damage_per_tick／dash_duration逐rank內容，範圍900／1000／1100／1200、寬150、傷害250／300／350／400。使用固定點per_tick * duration * 10換算，面板與實傷一致；移除102/1024近似0.1作除數的偏差。非法metadata／rank／target、定身、零位移或完全封閉透過既有RErr施法交易退回，不扣mana／CD，不發布成功施放cue。

metadata改用既有Custom selector表達沿路，Lua與原始碼描述如實說明瞬間、遇地形停止、每目標一次。沒有英雄名稱特判；其他既有純DashToPoint的全段可行政策保留。本批並未實作0.5秒逐tick位移、殘留火焰、持續引導或新的路徑瞄準圖形；不改ABI／protobuf／CABI。

## 集中驗收

最終RustV2 **1904通過／10既有ignored／0失敗**。本輪將sim72全量納入，含新增3個geometry案例；base277含新增7個共用執行器／真handler／正式60Hz案例。其他為template76、core919、codegen60、backend190／1、bridge135／1、client151／8、ABI15、restart9；含generation共11工作全部成功。

正式60Hz案例涵蓋四rank射程clamp與魔力／冷卻、實際中段／終點／側外／遠端／友軍、薄牆裁切與碰撞半徑、root／zero／封閉回滾、後續30 tick無重複傷害。非法資料／極值及沿路metadata另驗。測試以既有mana_regen_percentage=-1024 fixture隔離自然回魔，正式回魔規則未修改。

四Lua契約成功；正式backend-v2與UE5.8 build-v2成功，UBT **7.94秒**。WorldV2 **51成功／0 Failed／0 NotRun**，共4個warnings，已逐項分類於錯誤MD及engine log。兩張地圖各74影格／6片段；10張既有功能PNG已在World worker／watcher退休後目視，作為Unreal消費端回歸，並非新增技能路徑預覽或真網路ACK證據。

**3389來源／資產**前後集合、SHA／size／mtime一致；七DLL及import library另驗。**87份證據**逐byte／SHA封存及獨立校驗，含V1失敗資料。健康監測**14次**、最大間隔**171秒**；最終7組worker／watcher、V1過渡工作及全程monitor均退出。

## 錯誤與剩餘

錯誤記錄2026-10-22-dash-path-errors.md，總登錄E1491。V1為276通過／1失敗，唯一失敗是fixture忽略同tick自然回魔85 raw；修正測試隔離後嚴格檢查扣魔一次，未放寬正式玩法或覆寫失敗證據。API／路徑猜測錯誤亦已記錄。

尚缺持續引導、按時間位移／殘留效果、特殊技能幾何完整預覽、完整combat replica接受、召喚commit整組原子回滾、更多英雄／道具／召喚師、美術音效。原MVP33/44、11項外部ACK／LAN／實際OS DPI／固定效能接受不變；未宣稱全遊戲等同LOL。零新增真網路對局、quota reset、commit／push，omfx未維護。

證據索引：../acceptance/2026-10-22-dash-path/index.json。


## 2026-10-22 沿路施放提示（最新）

# 2026-10-22 沿路施放提示最終驗收

OpenSpec complete-swept-path-preview 5/5完成。本批接續真實沿路衝刺，補上內容驅動的意圖路徑輪廓與技能說明。

## 實作

共用AbilityDefinition新增可選typed aim_geometry::swept_path／width_key契約；兩端內容生成器校驗instant active/ultimate point、完整rank、正值且有界range／width。不依英雄名、技能ID、Custom字串或特效半徑推測。fire_dash顯式宣告既有dash_width，真handler仍消費同一extras。任意ID與不同key的codegen案例確認可重用。

Generated metadata發布逐rank SweptPathWidths，bridge依目前公開rank與世界比例送入既有owner HUD payload。owner在合法point且已學習／已知射程時保留寬度；替換／未知型別／非法值／角色退休清除。無ABI／protobuf／CABI改動。

新共用C++ builder產生51點封閉capsule，寬度為直徑，沿地面依請求距離與射程取較小值，含起終端；零位移與非有限值清除輪廓。owner實際paint使用同一builder及既有投影／split viewport／DPI裁切與pending aim生命期，不掃描actor或做隱藏碰撞查詢。射程圈與point-area預覽共用輪廓投影，既有單位提示保留。

技能tooltip補寬度、超距離截短、遇地形停止與伺服器結果說明；瞄準時超距離衝刺不再要求靠近目標。此輪廓是意圖，不預告已裁切地形終點、不阻擋或發送命令、不代表ACK；真實固定點碰撞、root拒絕、mana／CD回滾維持既有正式規則。root專用預覽文字、定時移動／火焰殘留、更多特殊技能幾何仍待完善。

## 集中驗收

RustV1 **1938通過／10既有ignored／0失敗**，含content33、sim72、template76、core919、codegen61、base277、backend190／1、bridge135／1、client151／8、ABI15、restart9，含generation共12工作全部成功。新增兩個schema測試與一個arbitrary-ID codegen測試；base正式60Hz衝刺、薄牆、rank、root／零位移／封閉回滾亦全量回歸。四Lua契約成功。

正式backend-v1與UE5.8 build-v3成功；最終UBT **8.94秒**（首次build-v1為84.32秒）。七DLL與import library另做SHA驗證。WorldV3 **52成功／0 Failed／0 NotRun**；warnings逐項留在本批errors及engine log。新增SweptPathPreview真native案例涵蓋四rank、輪廓端帽／閉合／寬度、射程clamp、斜向短距離／地面高度、zero／nonfinite、owner／tooltip替換與退休。新PNG擷取實際SOmHudToolTip，供文字可讀性驗收；不將tooltip圖宣稱為真對局路徑錄影。

兩張既有地圖各74影格／6片段；共11張PNG在World worker／watcher退出後目視，包含新技能提示及10張地圖／HUD／角色既有功能回歸。**3393來源／資產**前後集合、SHA／size／mtime一致。**123份證據**封存並獨立逐byte／SHA比對。健康監測**9次**、最大間隔**171秒**；7組最終worker／watcher與8組V1／V2過渡工作全部退休，monitor退出。

## 錯誤與剩餘

WorldV2另發現ClearHeroSnapshot未清路徑寬度；已補共用清理入口並維持嚴格斷言。目視同時修正tooltip fixture的world-unit／rank誤用，正式bridge比例規則未變。WorldV1發現TArray閉合自引用斷言，exit3。已先複製首點再Add，保留失敗engine log、receipt與來源快照；取消仍等待的source-v1 checker後、capture退出後才修復，重建並以WorldV3完整接受，未把崩潰當環境問題或覆寫舊證據。其他查詢／literal編輯錯誤記於2026-10-22-path-preview-errors.md，總登錄E1492。

持續引導、定時移動／殘留、完整combat replica接受、召喚整組commit原子回滾、更多英雄／道具／召喚師、美術音效仍待完善；全遊戲盤點已更新。MVP33/44與11項外部ACK／LAN／實際OS DPI／固定效能接受維持pending。零新增真網路對局、quota reset、commit／push；omfx未維護。

證據索引：../acceptance/2026-10-22-path-preview/index.json。


## 2026-10-22 位移施放提示（最新）

# 2026-10-22 位移施放提示最終驗收

OpenSpec complete-mobility-cast-feedback 5/5完成。新增通用位移技能的定身／狀態未知／原地落點提示，普通技能保持定身下可施放。

## 實作

omoba-content-model的ability_requires_movement由typed SweptPath或executable DashToPoint導出位移需求；不看英雄名、技能ID、動畫或dash_width鍵名。codegen產生bRequiresMovement，bridge只在有效公開rank／point／正值range時送入既有HUD payload，owner在已學習point保留policy，替換／未知形狀／ClearHeroSnapshot／ResetRuntimeInputScope清除。無ABI／protobuf／CABI變更。

bridge對完整公開owner source buff snapshot建立獨立root channel：1 clear、2 rooted、0 unknown，永久時間-1；使用精確entity generation、owner、披露、存活與source relation守門，保留4096列預算，不受UI64展示截斷。root取同類最晚解除時間；非法duration／instance／重複root保持未知，未知buff metadata保守撤銷兩種channel。普通stun／silence channel不因root-only壞資料而宣稱不能施法。snapshot／來源撤銷／退休同步清兩種channel。

owner兩種查詢共用嚴格reflection簽章與回傳校驗；不掃描actors或讀取隱藏資料。預覽保留unlearned／cooldown／mana／stun／silence優先，再呈現定身、永久定身、未知位移控制與零地面距離。原地檢查使用公開位置XY與引擎epsilon，不把Z差當可位移，也不宣稱預測固定點量化邊界。普通point技能可選自身位置。tooltip補位移技能root／不同落點需求。

提示保持唯讀、瞄準可留待控制解除，不自行取消／拒絕／發送命令，不分配input id、不推定ACK或自行倒數。後端既有固定點碰撞、root拒絕及mana／CD回滾維持；位移召喚師／道具尚未因本批自動接入這套ability policy。

## 集中驗收

RustV1 **1940通過／10既有ignored／0失敗**；content34、sim72、template76、core919、codegen62、base277、backend190／1、bridge135／1、client151／8、ABI15、restart9，含generation共12工作全部成功。新增model來源分類與arbitrary-ID generator測試；正式60Hz衝刺、薄牆、root／zero／封閉回滾、原技能與地圖生命期全量回歸。四Lua契約成功。

正式backend-v1與UE5.8 build-v2成功，UBT **7.13秒**；七DLL與import library SHA另驗。WorldV2 **53成功／0 Failed／0 NotRun**，warnings共4個，分類在本批errors及engine log。新增MobilityCastFeedback以真bridge／owner reflection驗證公開root／最晚時間／永久／UI64外／非法值／duplicate／未知metadata／世代／死亡／撤銷／移除／退休、普通cast隔離、feedback優先序／原地XY、生成policy與tooltip替換／退休。

新圖擷取真SOmHudToolTip的定身剩餘時間與實際技能操作需求；其餘11圖作既有地圖／HUD／角色／沿路技能說明回歸。共12張PNG在World worker／watcher退休後目視；不當作真對局capsule錄影或ACK。兩張地圖各74影格／6片段。**3394來源／資產**前後集合、SHA／size／mtime一致；封存**105份證據**並獨立逐byte／SHA校驗。健康採樣**7次**、最大間隔**171秒**；7組最終worker／watcher、4組V1過渡工作及monitor已全部退出。

## 錯誤與剩餘

WorldV1為52成功／1失敗，tooltip fixture缺MaxLevel被正式HUD校驗拒絕；已從真metadata補齊，不放寬校驗，保留engine／report／PNG／receipt與來源快照，重建後WorldV2全量接受。另於執行前自查修正批次名替換誤改舊測試ID，未漏跑或放寬測試；詳見2026-10-22-mobility-feedback-errors.md，總登錄E1493。正式驗收實際失敗／警告皆以保留log為準。

持續引導、定時衝刺／殘留、更多特殊幾何、位移召喚師／道具提示、完整combat replica接受、召喚commit整組原子回滾、更多英雄／道具／召喚師、美術與音效仍待完善。MVP33/44及11項外部ACK／LAN／實際OS DPI／固定效能接受不變。零新增真網路對局、quota reset、commit／push；omfx未維護。

證據索引：../acceptance/2026-10-22-mobility-feedback/index.json。


## 2026-10-22 正式圓形爆發技能（最新）

# 2026-10-22 正式圓形爆發技能最終驗收

OpenSpec complete-point-blast-combat 5/5完成。補齊炎刃／炎襲的實際點選圓形戰鬥規則與對應瞄準半徑。

## 實作

base_content新增共用PointBlast resolver／executor，wrapper明確傳入AbilityId、數值鍵、damage kind／profile及可選closed stun／root／silence，不以英雄名稱特判。嚴格解析rank JSON；現在存在但型別／範圍不合法的extra拒絕，缺漏可用同rank生成常數且checked index不panic。限制damage0..1000000、range／radius1raw..10000、control1raw..60；正值低於固定點quantum拒絕。

來源必須alive且HP正值、存在合法位置，target必須point，source／center在共用有界幾何範圍；只限制落點中心的cast range，圓內敵人可延伸到施法者射程外。重用i128精確circle（零長度capsule），包含相切，排除一raw外、友軍、死亡／HP0與caster；候選canonical排序去重。全部預檢結束才發一次cue與每敵人一次damage／可選control，保留FIRE profile、canonical buff及既有瞬發語義。非法RErr交還既有host cast交易退回mana／CD／deferred outcomes，未宣稱一般process_outcomes整組原子回滾。

flame_blade改為合法point-circle、炎襲共用damage+stun；原四rank damage／range／mana／cooldown／radius／stun數值保持。原本None／Entity錯誤target或壞JSON會回ROk的路徑修正為失敗；炎刃preview改AreaEffect，炎襲preview增加stun StatusModifier。Lua描述如實說明指定地點、圓形、各一次傷害；模組歷史WERT註解改成實際QWER slot，未更動loadout。

共用typed PointCircle／radius_key發布手寫handler的同一radius鍵，兩生成器校驗完整rank、維度／range邊界及point instant active／ultimate；與executable effects互斥，避免雙重幾何。ability_requires_movement明確match SweptPath而非is_some，圓形技能不受root位移限制。Unreal沿既有current rank payload→owner circle／tooltip呈現炎刃100與炎襲500，不讀cosmetic特效半徑。無ABI／protobuf／CABI改動。

## 集中驗收

RustV1 **1950通過／10既有ignored／0失敗**；content36、sim72、template76、core919、codegen63、base284、backend190／1、bridge135／1、client151／8、ABI15、restart9，含generation共12工作成功。新增2個schema、1個arbitrary-ID codegen與7個base案例。

真adapter測試驗證exact circle／tangent／one-raw外／alive／HP0／ally／FIRE profile／canonical順序與三種control、非法source／budget／extreme／range、合法自身point、真兩handler的JSON／rank／extra／target拒絕與fallback、circle／stun metadata。正式Production60Hz涵蓋兩技能×四rank、中心最大射程與圓邊界、實際damage／stun、mana扣一次／CD、十tick不重複傷害、outside／Entity／missing資源回滾與root下非位移仍可施放。共享既有isolated／no-regen／dynamic slot fixture，未複製另一套對局設定。

四Lua契約成功。正式backend-v1與UE5.8 build-v1成功，UBT **48.71秒**；七DLL與import library SHA另驗。WorldV1 **54成功／0 Failed／0 NotRun**，warnings共4個，分類見本批errors及engine log。新增PointBlastMetadata真native案例確認兩技能各四rank半徑、零swept width、非位移policy、owner circle快取、tooltip／未學習清除。

兩張新PNG分別擷取真SOmHudToolTip，避免把雙技能長文塞在超出螢幕的單一視窗；其餘12圖作既有地圖／HUD／角色／沿路與位移提示回歸。共14張PNG在World worker／watcher退休後目視；圖片不當作真對局施法或ACK。兩張地圖各74影格／6片段。**3397來源／資產**前後集合、SHA／size／mtime一致；**86份證據**封存並獨立逐byte／SHA校驗。健康採樣**8次**、最大間隔**171秒**，7組worker／watcher及monitor已全部退出。

## 錯誤與剩餘

盤點路徑猜測與build前private-import自查記於2026-10-22-point-blast-errors.md，總登錄E1494。新幾何分類、真metadata／world-unit／MaxLevel與no-regen fixture沿用前批教訓；所有實際失敗及warning保留在log，不覆寫證據。

仍缺persistent zone／持續引導、定時衝刺／殘留、更多特殊幾何、位移召喚師／道具提示、完整combat replica接受、召喚整組commit原子回滾、更多英雄／道具／召喚師、美術音效。MVP33/44及11項外部ACK／LAN／實際OS DPI／固定效能接受不變。零新增真網路對局、quota reset、commit／push；omfx未維護。

證據索引：../acceptance/2026-10-22-point-blast/index.json。


# 2026-10-22 召喚師施放提示最終驗收

OpenSpec `complete-summoner-cast-feedback`：5/5 完成。本批增加 D／F 地點召喚師技能的控制、原地位移、冷卻與落點提示，並修正權威原地閃躍消耗冷卻。

## 實作

- 真正的 PaintAim 呼叫共用 owner helper，從當前公開 row 校驗 protocol、存活／HP、擁有者、slot／catalog、重複 slot、目標類型、有限正值射程與完整冷卻數值；不合法或退休狀態顯示未知。
- 直接重用現有 bridge reflection 與 CastPreviewFeedback，呈現暈眩、沉默、定身與權威剩餘時間；不另設控制快取、倒數或本地施放守門。D／F 換位及任意 catalog ID 使用同一流程。
- 依目前封閉契約，Point summoner operation 是 PointBlink；未來新增 operation 必須擴充契約。HUD 未公開每個技能的超距離政策，因此提示保留伺服器判定，不假設所有技能必定截短。不預測地形落點，不代表 ACK。
- 真實 tooltip 保留「指定地點」，增加不同地面落點、定身／暈眩／沉默限制、跨牆與可站立落點規則。SelfHeal 不繼承位移限制，既有數值及控制政策保持。
- 權威模型在固定點超距離處理完成後比較 destination／origin；原地回 `NoDisplacement`，真 ECS queue 發 `NO_DISPLACEMENT` SettlementRejected receipt。拒絕發生在 effect／cooldown／命令清除前；一 raw 位移仍合法。

## 集中驗收

Rust V2：**1952 通過／10 既有 ignored／0 失敗**，含 generation 共 12 工作成功。

| 套件 | 通過 | ignored |
|---|---:|---:|
| content | 36 | 0 |
| sim | 72 | 0 |
| template | 76 | 0 |
| core | 921 | 0 |
| codegen | 63 | 0 |
| base_content release | 284 | 0 |
| backend | 190 | 1 |
| bridge | 135 | 1 |
| client | 151 | 8 |
| ABI | 15 | 0 |
| restart | 9 | 0 |

新增模型案例涵蓋雙 slot、Reject／ClampToRange、原地與 clamp 後固定點零位移、合法一 raw。真 ECS queue 案例檢查拒絕 receipt、HP／位置／移動／完整命令 queue／雙冷卻保持，隨後一 raw 合法施放會扣正確 slot 並清移動。

四個固定 Lua 契約成功。正式 backend V1 成功；UE5.8 V1／V2 建置成功，最終 UBT **18.58 秒**。七個 staged DLL 與 import library SHA 分別核對。

World V2：**55 成功／0 Failed／0 NotRun**，其中 54 Success、1 SuccessWithWarnings、4 個既有 RuntimePresentationReset 的 Untitled World DestroyActor warning。新增 SummonerCastFeedback 與既有 ResponsiveHudLayout 均成功；其餘地圖、塔／野怪／小兵與角色呈現、施法、HUD、音效資源等原生矩陣保留回歸。

新原生案例使用真 owner／bridge reflection、公開 root／stun／silence snapshot，驗證控制優先序／時間、D／F 換位、任意 catalog、錯誤 owner／generation、失效／未裝備／重複 slot／NaN／負冷卻／非法 range／protocol、退休與清除；同時擷取真正 SOmHudToolTip。它是合成公開 snapshot 接受，不能代替真網路施放或 ACK。

World worker／watcher 退出後目視 **15 張 PNG**。新提示可讀，使用正式 world scale；兩張地圖各 74 影格／6 片段，authority_ack=false。既有分層地圖標籤擁擠、公告 fixture 裁切等限制詳列 errors.md，未冒稱完整 HUD 或 OS DPI 接受。

**3398 來源／資產**前後集合、SHA／size／mtime 完全一致。**115 份證據**封存並另行逐 byte／SHA 校驗；**7 次健康採樣，最大間隔 170 秒**。7 組最終 worker／watcher，以及保留歷史結果的 5 組 worker／watcher 全部退休，monitor 亦已退出。

## 錯誤、證據與剩餘

Rust V1 的舊獨立冷卻 fixture 使用原地施放；改成實際位移並保留原斷言後，V2 全量通過。World V1 的 tooltip 用語回歸保留原文字後以 V2 全量通過，未放寬版面測試。等待中的 source V1 用已記錄 PID／executable／creation token 精確取消；最終使用修正後 V2 基準。所有失敗、取消原因與證據保留；詳見同目錄 `2026-10-22-summoner-feedback-errors.md`，總登錄 E1495。

證據索引：`../acceptance/2026-10-22-summoner-feedback/index.json`。仍缺持續引導／持續區域、定時衝刺／殘留、更多特殊幾何與召喚師內容、道具位移提示、召喚整組 commit 原子回滾、完整 combat replica 生命期接受及更多美術音效。MVP 33/44 與 11 項外部 ACK／LAN／實際 OS DPI／固定效能接受維持原狀。本批零新增真網路對局、quota reset、commit／push；omfx 未維護。


# 普攻被動與受擊點扇形最終驗收

2026-10-22 批次，於 2026-10-23 臺北時間跨日完成。OpenSpec `complete-passive-cone-combat`：5/5 完成。

## 實際修正

雨鐵砲原本雖描述受擊點扇形，卻從攻擊者位置測量候選角度；原生普攻投射物命中也沒有接上既有的被動 hook。本批修正兩者，讓真正普攻命中後，以受擊點為起點、沿攻擊者至受擊點方向，在 90°／150 半徑內對每個存活敵人結算一次既有比例真實傷害。

- `omoba-sim::sector` 提供可共用的有界扇形判定。半徑／dot／cross 使用 i128；角度沿用引擎 4096 tick、Q10 sin/cos 邊界，弧度依共用 TAU 常數取最近 tick。包含起點、角度／半徑邊界，支援 ray／半平面／大角度／全圓，非法維度失敗關閉。這不是無量化的實數角度宣稱。
- 共用 PassiveCone resolver 校驗 rank、半徑、角度、比例及攻擊／傷害預算，無 unchecked rank indexing。先完成預檢與精確篩選，再 canonical 排序去重並發傷害；排除友軍、死亡／HP0、來源本身與扇形外候選。零方向不發效果。保留原四 rank 百分比／半徑／角度資料與 Pure／TRUE profile。
- 普攻來源以 authority-only `normal_attack` 明確標記；只有真正且正傷害的那發原生普攻具有資格，純視覺連發／miss 沒有。技能子彈即使使用 NORMAL damage profile，或沿用 credit／source snapshot，也會清除此資格，避免被動連鎖重複觸發。
- Hero 可在沒有 UnitScript tag component 時保存最多 64 筆排序過的有效技能 rank；活來源讀當前值，精確舊世代退休後使用發射時不可變來源資料。被動 type／max rank 由 registry 校驗，依 ID 固定順序派發。保留既有 projectile credit 與死亡後命中語義，不復活來源或讀取替代世代。
- Rain 的主動 execute 明確 RErr；on_learn 校驗 rank 並使用 canonical 永久生命期。Unreal 從明確 metadata 發布 `bPublishedPassive`，被動 tooltip 顯示「無需主動施放」，替換為 active state 時恢復一般提示。沒有新增瞄準扇形 overlay 或假造權威落點／ACK。

無 script ABI、public C ABI 或 protobuf 變更；不依英雄名稱或特效半徑判定命中。

## 集中驗收

最終 Rust V2：**1962 通過／10 既有 ignored／0 失敗**，含內容生成共 12 工作成功。

| 套件 | 通過 | ignored |
|---|---:|---:|
| content | 36 | 0 |
| sim | 75 | 0 |
| template | 76 | 0 |
| core | 921 | 0 |
| codegen | 63 | 0 |
| base_content release | 291 | 0 |
| backend | 190 | 1 |
| bridge | 135 | 1 |
| client | 151 | 8 |
| ABI | 15 | 0 |
| restart | 9 | 0 |

新增 3 個幾何、5 個真 adapter／來源與 2 個正式 60Hz 案例。涵蓋起點、前後方、角度／半徑相切、一 raw 外、旋轉與極端輸入；四 rank 資料、非法 rank／budget／零方向、被動主動拒絕及永久 marker；真普攻三發視覺連發只有一發具資格、退休來源與同 ID 新世代隔離、技能子彈不繼承資格。

60Hz 案例經真正原生普攻投射物生成、飛行、命中、被動派發與傷害結算，驗證四 rank 的主要目標額外傷害、前向／邊界命中、後方／角外不命中、只結算一次、pause 凍結飛行與命中、發射後暈眩保留被動；另驗主動施放拒絕不扣資源。這是離線 authority ECS 的真結算，不是新網路對局或正式輸入 ACK。

四個固定 Lua 契約成功。正式 backend V1 與 UE5.8 build V1 成功，UBT **70.67 秒**；七 staged DLL 與 import library SHA 分別驗證。

World V1：**56 成功／0 Failed／0 NotRun**，其中 55 Success、1 SuccessWithWarnings；4 個既有 RuntimePresentationReset／Untitled DestroyActor warning。新增 PassiveConeMetadata 驗證真正 generated passive type／描述、四 rank 零 point-circle／swept path、非位移、實際 tooltip／active 提示退休與替換。既有地圖、NPC、角色、施放及 HUD 矩陣保留回歸。

World worker／watcher 退出後目視 **16 張 PNG**，新被動 tooltip 可讀且沒有主動按鍵指示。兩地圖各 74 影格／6 片段、authority_ack=false；既有地圖標籤擁擠與局部 fixture 裁切限制詳列 errors.md，未當作全 HUD／真連線／實際 OS DPI 接受。

最終 **3402 來源／資產**集合、SHA、size、mtime 前後一致。另比較第一輪建置／World 與最終來源，只有 Rust 測試佇列清理那個檔案不同，其他正式輸入 SHA／size 一致；seal 與 independent verify 都重新檢查此條件。

**133 份證據**封存並另行逐 byte／SHA 校驗，包含完整最終紀錄、第一輪失敗、focused probe、來源等價證明與執行腳本。健康採樣 **13 次、最大間隔 171 秒**；7 組最終、4 組歷史／probe worker／watcher 與 health monitor 全數退休。

## 錯誤與剩餘

Rust V1 唯一失敗是測試假設閒置 Hero 一定有命令佇列，結果 290 通過／1 失敗；改成有佇列才清理，沒有放寬任何傷害／pause／stun 斷言。Focused probe 2/2、完整 Rust V2 均通過。失敗、自然拒絕的 source V1 與所有原始證據完整保留，見 `2026-10-22-passive-cone-errors.md`，總登錄 E1496。

證據索引：`../acceptance/2026-10-22-passive-cone/index.json`。

仍缺持續引導／持續區域、定時衝刺／殘留、更多主動特殊幾何與完整預覽、變身技能資料的共用嚴格校驗、召喚整組 commit 原子回滾、完整 combat replica 生命期接受及更多英雄／道具／召喚師／美術音效。MVP 33/44 與 11 項外部 ACK／LAN／實際 OS DPI／固定效能接受維持原狀。零新增真網路對局、quota reset、commit／push；omfx 未維護。


---

# 自身變身技能共用校驗與最終驗收

完成 `complete-transform-buff-validation`。火繩銃與三段擊改用同一個有界資料解析／living-self 執行器，先檢查等級、JSON、時間、數值、型別與整數發數，再提交唯一 buff。省略 extras 才使用相同 authored rank；顯式錯值不再靜默補零／fallback。一般屬性維持 Q10 整數，attack_stun_* 維持主機實際讀取的小數格式，未變更七個原始 rank 的平衡數值、ABI 或 Unreal C++。

新測試驗證所有 rank、負值／超限／錯型別／null／低於量子／非法目標／死亡或退休來源均無部分效果、recast 單項 refresh 與精確 expiry。正式 60Hz PlayerInput 驗證七個 rank 只扣一次魔力／冷卻、pause 不倒數、冷卻拒絕不刷新、真正普攻吃到增益，三發視覺只有一顆有傷害與 normal-attack 資格，實際飛行命中後總傷害只一次，變身到期恢復原攻擊。

## 最終結果

- Rust V2：**1966 passed／10 既有 ignored／0 failed**，12 工作含內容生成；base_content 295。四項新增測試的 focused probe 4/4 通過。
- 四項 Lua 契約、backend V1、UE5.8 build V1 成功；UBT 保留 -NoUBTMakefiles，增量執行 1.66 秒。7 個 staged DLL 與 import library SHA 均符合建置輸出。
- World V2：**56 成功／0 Failed／0 NotRun**（55 Success、1 SuccessWithWarnings）；4 個既有 RuntimePresentationReset／Untitled DestroyActor warning。
- **3404 來源／資產**集合、SHA、size、mtime 前後一致；與第一輪 native 建置輸入另比較，只有 Rust 測試檔的 lazy-resource fixture 修正不同，其餘正式輸入 SHA／size 一致。
- 封存 **151 份證據**並另行逐 byte／SHA 校驗；World worker／watcher 退出後目視 **16 張 PNG**。既有 layered 地圖標籤擁擠、first-kill 下方 HUD fixture 裁切與 objective 窄 strip 限制保留；本批沒有新增 UI 視覺功能。兩地圖各 74 影格／6 片段、authority_ack=false。
- 健康採樣 **7 次，最大間隔 171 秒**；7 組最終及 5 組歷史／probe worker／watcher 與 health monitor 全數退休。

## 錯誤與範圍

Rust V1 294/1 的唯一失敗為新測試提前讀取尚未由首次投射物建立的 ProjectileOrigins；所有讀取者退出後，僅把 prior set 讀取改為 optional，未放寬傷害或數量斷言。另修正 World runner 複製時誤改的既有可選截圖參數，V2 補齊畫面。失敗及原始證據均保留，詳見 2026-10-23-transform-buff-errors.md，總登錄 E1497。

全遊戲盤點見 2026-10-21-whole-game-completeness-audit.md 與本批 audit.md。持續引導／區域、定時位移／殘留、特殊幾何完整預覽、召喚整組 commit 原子回滾、完整 combat replica 生命期接受與更多內容／美術音效仍待補齊。MVP 33/44 與 11 項外部 ACK／LAN／實際 OS DPI／固定效能接受維持 pending。零新增真網路對局、quota reset、commit／push，omfx 未維護。

證據索引：`../acceptance/2026-10-23-transform-buff/index.json`。

## 進行中：48 小時無人值守／名牌避讓（2026-10-23）

使用者已授權至 2026-10-25 00:33:38 Taipei 的無人值守工作。App heartbeat `omoba-48` 已設為本對話每 3 分鐘接續，詳見 `docs/plans/2026-10-23-unattended-48h.md`；不要重複建立。優先完成 `openspec/changes/complete-nameplate-screen-spacing/tasks.md`。

本批已接線 bounded screen spacing、目前 mesh anchor、collapsed wrapper 之外的真 Slate layout 尺寸量測及新 native tests／capture。工作前綴 `omfue/Saved/BuildTemp/nameplate-spacing-`；Rust／UE5.8 build／backend／contracts 已啟動，正式輸入目前凍結。先檢查各 receipt、creation token、worker／watcher 活性與日誌再接續，避免重複建置或在 reader 存活時改來源。錯誤見 `docs/plans/2026-10-23-nameplate-spacing-errors.md`。最終尚需 World 58、source before/check、圖像目視、證據封存與總登錄 E1498。


---

# 公開非英雄名牌避讓：最終驗收

完成 complete-nameplate-screen-spacing。沿用每視角最多64候選與選取／游標／建築／中央距離的穩定順序，新增實際 Slate 尺寸與目前 mesh 錨點的有界矩形避讓；重疊的低優先名牌暫時隱藏，移開後可恢復。名牌不移離單位、英雄顯示規則與地面關係標記不變。文字縮放／狀態資訊會自然改變量測足跡。最多64次量測與固定容量幾何工作，不對10000實體建立全體兩兩碰撞。

實際地圖的非英雄名牌堆疊減少；英雄彼此及英雄／非英雄交叉仍可能存在，後續優先評估保留英雄血條的共用障礙保留策略。本批不宣稱所有名牌碰撞均已解決。

初次／隱藏量測與正式 UMG 顯示共用純 Slate layout factory，避免短暫 SObjectWidget 釋放根節點。ReleaseSlateResources 清除量測根，下次需求可重建。UI比例查詢留在負責UMG的OmRuntime模組。保留所有視野、陣營、生命與世代檢查。

## 驗收

- Rust **1966通過／10既有ignored／0失敗**，12工作含內容生成；四項Lua契約、後端及UE5.8正式建置成功。
- World **58成功／0 Failed／0 NotRun**，包含新增NameplateScreenSpacing與既有HealthNameplateBudget；4個既有RuntimePresentationReset場景銷毀warning。
- 密集碰撞、padding邊界、縮放、非法資料、固定64上限、穩定優先順序、首次量測、隱藏／資源釋放後恢復及真Slate繪製均驗證；既有合成地圖保留兩隊full-health可見與關閉偏好時隱藏的斷言。
- **3406來源／資產**集合、SHA、size、mtime前後一致，7 staged DLL與import library SHA通過；先前已通過避讓測試的正式輸入另與最終比較，只允許測試fixture差異。
- 封存 **164份證據**並獨立逐byte／SHA校驗；World退出後目視 **18張PNG**，含密集／恢復與兩地圖。圖像是單機原生fixture，非真網路ACK或實際OS DPI接受；既有first-kill局部HUD裁切與objective窄strip限制仍記錄。
- 健康採樣 **10次，最大171秒**；全部最終與歷史worker／watcher及本批health程序均退休。

## 錯誤與接續

第一輪建置跨模組UMG連結失敗；第一次原生測試暴露臨時SObjectWidget生命期；修正後既有地圖fixture仍需在手動插值後刷新視角判定並使用分離的焦點名牌。未放寬正式避讓規則或原斷言，完整失敗、精確來源檢查取消與後續重跑證據保留。詳見2026-10-23-nameplate-spacing-errors.md，總登錄E1498。

48小時無人值守heartbeat **omoba-48** 仍接續本對話，期限 **2026-10-25 00:33:38台北時間**。後續先讀handoff／盤點／錯誤與程序receipt，避免重複工作。持續引導／區域、定時位移與特殊幾何、召喚整組commit、完整combat replica生命期與更多內容／美術音效仍pending；MVP33/44、11項外部ACK／LAN／實際DPI／固定效能接受不變。未新增真網路對局、quota reset、commit/push或omfx維護。

證據索引：`../acceptance/2026-10-23-nameplate-spacing/index.json`。

## 進行中：hero reservation（2026-10-23 01時）

48h heartbeat `omoba-48` 仍為同一既有排程，勿重建或延長期限。`complete-nameplate-screen-spacing` 已完成；目前 `openspec/changes/complete-hero-nameplate-reservation/tasks.md` 處理可見英雄保留區與一般非英雄避讓，焦點目標保留例外。英雄／非英雄各64候選，量測與儲存有界；隱藏／退休英雄不保留空間。

目前前綴 `omfue/Saved/BuildTemp/hero-reservation-`；Rust／UE建置／backend／contracts 正在集中驗收，來源凍結。先讀 receipt、owner creation token、日誌及 tasks，不可重複啟動或在 reader 存活時修改來源。後續預期 World59、source capture/check、20張截圖與證據封存；錯誤／預防見 `docs/plans/2026-10-23-hero-reservation-errors.md`。下一總登錄 E1499。


---

# 英雄名牌保留區：最終驗收

完成 complete-hero-nameplate-reservation。共用固定容量矩形機制保留當前可見英雄的真Slate足跡，讓一般小兵／野怪／建築名牌避開英雄。選取與指向的非英雄可跨英雄保留區，仍服從非英雄間優先避讓。英雄彼此不隱藏、不移位；焦點例外與超過64英雄的保留上限仍明確存在。

英雄候選採自身／選取／指向／中央距離／實體世代的穩定順序，最多64英雄與64非英雄量測；驗證目前渲染可見性、生命、披露與世代，使用目前mesh錨點。每視角重建保留區，隱藏、退役、移動與換世代後無殘留障礙。

## 驗收

- Rust **1966通過／10既有ignored／0失敗**，12組含內容生成；四項Lua契約、正式後端與UE5.8建置成功。
- World **59成功／0 Failed／0 NotRun**；新HeroNameplateReservation與既有NameplateScreenSpacing、HealthNameplateBudget成功。既有RuntimePresentationReset場景銷毀warning保留。
- 幾何邊界、非法資料、獨立64容量、普通／焦點例外、真英雄隱藏／重現／重設／換世代及Slate截圖驗證。
- **3407來源／資產**集合、SHA、size、mtime前後一致；7個staged DLL與import library一致。
- 封存 **108份證據**並獨立逐byte／SHA校驗；程序退出後目視 **20張PNG**，包含普通避讓、焦點例外及兩地圖。單機fixture不替代外部ACK、實際OS DPI或固定效能接受。
- 健康採樣 **6次、最大間距171秒**；7組worker／watcher及health程序均退休。

## 錯誤與接續

本批沒有建置／測試失敗。一次診斷讀取誤猜health.log副檔名，實際為health.jsonl；已記錄原因並要求先查實際檔案名稱，詳見errors.md與總登錄E1499。

48小時無人值守omoba-48繼續至2026-10-25 00:33:38 Taipei。持續引導／區域、定時位移與特殊幾何、召喚整組commit、combat replica生命期與內容美術音效仍pending；MVP33/44、11項外部接受不變。未新增真網路對局、quota reset、commit/push或omfx維護。

證據索引：`../acceptance/2026-10-23-hero-reservation/index.json`。

## 進行中：summon formation（2026-10-23 01時）

目前change為complete-summon-formation-validation，優先項complete-nameplate-screen-spacing與hero-reservation都已完成。共用rank-aware陣形預檢與全座標生成已接現有召喚，內容描述修正自身朝向；三個純／真adapter測試與一個Production60Hz四rank測試已寫。這不宣稱後續整組settlement原子性完成。

目前前綴omfue/Saved/BuildTemp/summon-formation-，config記錄RustV2、build/backendV2、contracts/world/sourceV1；V1三個失敗worker已退休並保留，原因Fixed64沒有abs，改上下界比較。health每170秒，原排程omoba-48期限不變。先讀receipt、health.jsonl、errors及tasks，避免重啟執行中工作或修改reader使用的來源。預期Rust1970／10、World59、source3409，總登錄下一E1500。缺prepare-docs／seal／verify／finalize的實際執行；20張截圖須在本批World程序退休後重新檢查。


---

# 召喚陣形嚴格預檢：最終驗收

完成 complete-summon-formation-validation。共用resolver拒絕錯誤JSON、型別、小數／超額數量、非法rank、生命期／間距與超界陣形；缺省值只取目前rank正式內容，先完成所有固定點朝向座標再發出召喚。保留四rank數量2/4/6/8與45/50/55/60秒、自身朝向兩排與既有合法落點調整，修正內容說明。缺位置／死亡owner拒絕。

## 驗收

- Rust **1970通過／10既有ignored／零失敗**；新增3個resolver／真adapter測試與1個Production60Hz四rank測試。第二排落點失敗時第一排已預約實體經managed discard完全清理，可重試；正式60Hz各rank扣魔一次、啟動CD、pause凍結生命期。
- 四Lua契約、backend及UE5.8建置成功；World59成功、零Failed／NotRun，4既有場景銷毀warning保留。
- **3409份來源／資產**集合、SHA、size、mtime一致；7 staged DLL與import library核對成功。
- **137份證據**獨立逐byte／SHA驗證；程序退休後檢查 **20張PNG**。保留既有first-kill局部HUD裁切、objective窄strip及英雄／焦點交疊的範圍限制。
- 健康採樣5次、最大間隔171秒；最終及失敗worker／watcher與health程序均退休。

## 失敗與範圍

V1編譯發現Fixed64沒有abs方法；改採上下界比較，保留原始失敗，等所有reader退出後才修正。詳見errors.md與總登錄E1500。此批驗證施法預檢及既有managed request回滾，不宣稱後續並行owner變更時整組outcome settlement已具原子性。

48h omoba-48沿用期限2026-10-25 00:33:38 Taipei。下一批優先補齊既有召喚的共用自身朝向陣形瞄準提示：generated ExtrasJson已有逐rank陣形值，而point aiming目前只有range／area／swept-path。應用明確authored preview語意，不能使用英雄名稱特判或宣稱預覽就是地形調整後最終出生點。持續引導／區域、整組settlement、combat replica生命期、內容／美術／音效與11外部接受維持pending，MVP33/44不變。沒有新增真對局、quota reset、commit/push或omfx維護。

證據：`../acceptance/2026-10-23-summon-formation/index.json`。

## 進行中：facing formation preview（2026-10-23 01時）

上批召喚陣形預檢已完整接受；本批complete-facing-formation-preview已實作明確作者契約、generated逐rank metadata、typed UI spec、公開owned facing與<=128請求位置繪製／tooltip。新增shared schema與generator測試，原生FacingFormationPreview測試含lifecycle及正交意圖圖＋實際tooltip；不是3D viewport出生點接受。

前綴omfue/Saved/BuildTemp/formation-preview-，config目前全部V1、history空；Rust已通過content／template／core／codegen並完成generate，base測試、build/backend/contracts正在執行，來源凍結。health每170秒，原排程omoba-48期限不變；不可重複啟動或在reader存活時改來源。後續預期Rust1972／10、World60、source3411與21圖。尚需staged、World/source capture/check、prepare-docs／stop／seal／verify／finalize；下一總登錄E1501。先讀receipt、health.jsonl與errors再接續。

### 同批更新：V2（01:56 Taipei）

V1全成功後發現rows/executor契約風險，等待全部reader退休才補共用AbilityConst.rows、handler消費與熱重載相容守門。新增template測試與row4案例；現在config全部V2、history含V1七組worker（成功退休）。V2 UE5.8/backend/contracts與World60已成功、staged已核對，source-before-v2為3411份；Rust base299通過，ABI/restart收尾，完整預期1973／10。

V2所有21圖已檢查：18張與逐張目視過的V1完全同SHA，另3張V2重新目視；visual-review-v2.json保留每圖review_source及SHA並由seal/verify強制核對。尚待source-v2最終receipt、prepare-docs／stop／seal／verify／finalize。來源仍凍結，勿重跑或改正式來源。下一批具體項為numeric geometry熱重載與build-time native預覽一致性（rows已守門，count／spacing／front等尚須共用政策），不能把本批當全部數值同步已完成。E1501仍待finalize寫入。


---

# 自身朝向陣形施放提示：最終驗收

完成complete-facing-formation-preview。新增明確facing_formation作者契約與標準逐rank數量／行列間距／前距欄位，特殊handler幾何不由英雄名稱、cue形狀或碰巧有extras推斷。generated metadata經bridge縮放為typed UI spec；owned overlay公開目前有效朝向，瞄準繪製最多128個請求位置，tooltip顯示排／列與地形／友軍佔位調整規則。保留原point指令、距離、mana/CD/control與取消規則。

## 驗收

- Rust **1973通過／10既有ignored／零失敗**，新增共享schema與產生器契約測試；四Lua契約、正式backend及UE5.8建置成功。
- World **60成功／零Failed／NotRun**，含新增FacingFormationPreview：四rank、非指定技能不推斷、旋轉、非法資料／未知朝向、隱藏／重現與退役清除、真tooltip及Slate圖像。4既有RuntimePresentationReset warning保留。
- **3411來源／資產**集合、SHA、size、mtime前後一致；7 staged DLL與import library核對成功。
- 封存 **177份證據**，獨立逐byte／SHA校驗；程序退出後檢查 **21張PNG**。新增圖為使用正式幾何函式的正交意圖圖＋真tooltip，不能當作實際3D遊戲viewport或地形調整後出生點的畫面證明。
- 健康採樣12次、最大間隔172秒；最終及歷史worker／watcher與health程序均退休。

V1全部21圖曾逐張目視；V2其中18張與V1逐位元相同，沿用已目視像素，另3張有變化的V2圖重新目視。每圖review_source與SHA證據另驗。V1測試全成功後才由架構複核發現rows傳遞缺口；等待所有reader退休後補AbilityConst／executor／reload guard，再執行V2，保留兩輪證據。當前兩排技能平衡不變，四排與reload拒絕另有測試。

## 範圍與接續

預覽是目前公開朝向的請求陣形，伺服器可因地形／佔位與釋放時狀態調整。既有first-kill局部HUD裁切、objective窄strip與英雄／焦點交疊仍有限制。錯誤根因／修正／預防見errors.md及E1501。

48h omoba-48沿用期限2026-10-25 00:33:38 Taipei。下一個具體單機缺口：排數已釘住，但既有runtime數值調參與build-time native metadata的rank數量／間距／前距一致性仍需共用政策與guard；不宣稱本批已同步全部幾何數值熱重載。整組召喚settlement、持續引導／區域、其他特殊幾何、combat replica生命期、內容／美術／音效及11外部接受仍pending；MVP33/44不變。未新增真網路對局、quota reset、commit/push或omfx維護。

證據：`../acceptance/2026-10-23-formation-preview/index.json`。

## 進行中：aim preview reload coherence（2026-10-23 02時）

上一批formation-preview已完整接受；本批complete-aim-preview-reload-coherence已實作共用semantic projection（geometry kind/key、target/action、rank range、effective area、swept width、formation各值、movement），codegen與compiled/runtime signatures共用，啟動及ContentShape reload守門。新增三測試：共用投影、8類正式geometry變更矩陣、實際reload拒絕保留generation/hash再接受damage調參；新增測試已過，預期完整Rust1976／10、World60、source3411。

目前前綴omfue/Saved/BuildTemp/aim-coherence-，config全部V1/history空；Rust已過content/template/core/codegen/generate，base、UEbuild/backend/contracts執行中，來源凍結。health每170秒，原omoba-48期限不變。尚需staged、World/source capture/check、21圖像素比對/差異目視、prepare-docs／stop／seal／verify／finalize。先讀receipt/owner/health.jsonl/errors，不得重複啟動或在reader存活時修改正式來源。下一登錄E1502。接續優先實際玩法channels/zones或authored內容缺口，不以其他非幾何metadata政策當本批已完成。


---

# 技能預覽與熱重載幾何一致性：最終驗收

完成complete-aim-preview-reload-coherence。共用內容投影同時供Unreal產生器與compiled/runtime簽章使用，涵蓋作者幾何kind/key、target/action語意、每rank射程、有效圓形範圍、衝刺寬度、陣形排數／數量／間距／前距與movement語意。移除產生器重複解讀邏輯。啟動及reload拒絕幾何不一致，要求重建所有端；不把damage／duration／cosmetic等非幾何值一律鎖死。

## 驗收

- Rust **1976通過／10既有ignored／零失敗**；新增共用投影測試、8種正式內容幾何變更矩陣及實際reload交易測試。錯誤reload後generation/hash與active資料不變，隨後damage調參正常接受。
- 四Lua契約、正式backend與UE5.8建置成功；World60成功、零Failed／NotRun，4既有RuntimePresentationReset warning保留。
- **3411來源／資產**集合、SHA、size、mtime一致；7 staged DLL與import library核對。
- **112份證據**獨立逐byte／SHA驗證；21張PNG沿已接受像素比對與差異目視，逐圖review_source/SHA可追溯。原生fixture／意圖圖不替代3D最終出生點、真ACK或實際OS DPI接受。
- 健康採樣7次，最大間隔171秒；全部worker／watcher及health程序退休。

## 範圍與接續

本批關閉幾何數值reload一致性缺口，並非把全部非幾何UI metadata／敘述同步重做。錯誤根因／預防見errors.md與E1502。既有first-kill局部HUD裁切、objective窄strip及英雄／焦點名牌限制仍記錄。

48h omoba-48沿用期限2026-10-25 00:33:38 Taipei。接續優先盤點現有authored技能可補的持續區域／引導與實際玩家可見玩法；整組召喚settlement、combat replica生命期、內容／美術／音效及11外部接受仍pending，MVP33/44不變。沒有新增真網路對局、quota reset、commit/push或omfx維護。

證據：`../acceptance/2026-10-23-aim-coherence/index.json`。


---

# 技能升級前比較：最終驗收

完成complete-ability-upgrade-preview。所有正式技能共用Lua逐rank產生的基礎魔力、冷卻、施法準備與射程。既有+升級按鈕及一般技能tooltip共用比較內容，顯示目前→下一級、所需英雄等級與1技能點；未學習只顯示第一級，缺等級／技能點、滿級與不可用均有中文理由。快捷鍵沿用目前控制設定。資料標示為不含屬性修正的基礎值，不當作effective runtime數值或升級成功回執。

## 驗收

- 本批針對產生器與Unreal變更：**codegen65項成功**、實際產生成功、**World62成功／零Failed／NotRun**（含新AbilityUpgradePreview及原NativeAbilityProgression），4既有RuntimePresentationReset warning保留；四Lua契約及UE5.8建置成功。
- 沒有改動玩法平衡、Rust模擬或script ABI；全Rust前批1976通過／10既有ignored證據保留，本批未重跑或冒稱新增完整Rust接受。
- **3412來源／資產**集合、SHA、size與mtime一致；7 staged DLL與import library一致。
- **88份證據**獨立逐byte／SHA驗證；**22張PNG**，沿已目視像素精確比對及差異重新目視，review_source可追溯。新圖使用實際共享Slate tooltip呈現首學／比較／滿級，並非權威升級交易成功的畫面證明。
- 已驗證現有內容數字、遲學等級6、缺點、滿級、未知ID／越界rank、非有限數值、替換、runtime reset及retirement不殘留資料。既有HUD／技能提示／地圖呈現回歸保留。
- 健康採樣4次，最大間隔171秒；6個worker及watcher、health均退休。

## 範圍與接續

錯誤與預防見errors.md及E1503：先發現路徑再讀，Lua替換escape與執行前語法檢查。原生fixture的局部HUD裁切、objective窄strip及英雄／焦點名牌交疊限制保留。

48h omoba-48沿用2026-10-25 00:33:38 Taipei期限。channels／zones、整組召喚settlement、完整combat replica生命期、更多原創內容／美術／音效及11外部接受仍pending，MVP33/44不變。本批選擇具體玩家操作資訊缺口，不把typed metadata當作新增持續技能。未新增真網路對局、quota reset、commit/push或omfx工作。

證據：`../acceptance/2026-10-23-upgrade-preview/index.json`。


---

# 主動道具施放提示：最終驗收

完成complete-item-cast-feedback。共用catalog從作者target.team產生unit allegiance，hotbar傳遞作用半徑，六格owner cache保存當前identity／cooldown／geometry。真正PaintAim改用當前道具feedback：冷卻顯示秒數、敵我與可攻擊條件使用公開bridge表、距離取公開source/target中心，地點道具使用正式效果半徑畫作用圈。替換／隱藏／世代／未知／非法資料／來源退役不會沿用舊狀態。道具tooltip與overlay共用可見敵人及超距離拒絕／不走近提示。沒有變更平衡、輸入提交或權威端規則，也不套用技能的魔力／沉默／定身限制。

## 驗收

- 本批**codegen66項成功**、實際產生、四Lua契約與UE5.8建置成功；**World63成功／零Failed／NotRun**，4既有RuntimePresentationReset warning保留。原NativeAbilityProgression及前批AbilityUpgradePreview仍成功。
- 真bridge公開表測試：中心距離不受任意cursor影響、移出射程、敵我變化、不可攻擊、隱藏／世代替換、冷卻3.5秒、道具換槽identity、地點半徑80與單次單位換算、超距離、無地面樣本、來源世代、非法數值、輸入／來源退役。
- **3413來源／資產**集合、SHA、size、mtime一致；7 staged DLL及import library一致。
- **101份證據**獨立逐byte／SHA驗證；**23張PNG**沿已目視像素精確比對及差異目視，review_source可追溯。新圖是實際共享Slate tooltip／feedback文字；沒有冒稱為玩家操作下的3D道具作用圈畫面或權威命中回執。作用圈接線以原生metadata/cache檢查、正式PaintAim及建置／整合回歸驗證。
- 健康採樣6次，最大間隔171秒；6個最終worker、2個歷史worker及其watcher、health均退休。V1因測試friend宣告缺漏而建置失敗；等待讀取程序退休後補宣告，以V2建置／來源快照驗收，失敗日誌、標頭與指紋另封存。沒有改動Rust模擬或script ABI；全Rust1976／10沿前批證據，本批未冒稱重跑。

## 範圍與接續

錯誤預防見errors.md及E1504。既有first-kill局部HUD裁切、objective窄strip及英雄／焦點名牌交疊仍有限制。channels／zones、整組召喚settlement、完整combat replica生命期、更多原創內容／美術／音效及11外部接受仍pending，MVP33/44不變。

48h omoba-48沿用2026-10-25 00:33:38 Taipei期限。未新增真網路對局、quota reset、commit/push或omfx工作。接續優先具體玩法缺口；不以本批道具意圖提示冒稱已完成持續技能或對局權威接受。

證據：`../acceptance/2026-10-23-item-feedback/index.json`。


## 進行中：recall channel progress（2026-10-23 02:56 Taipei）

complete-recall-channel-progress已實作shared recall_rules default8／1..60驗證、template共用、generated registry總時長／freshness Root、UI-only total、正式HUD共用read-only Slate strip；不擴C ABI或本機外插。RustV1因serialized Root漏欄位codegen2失敗，已保留failed-codegen-v1.rs及原始logs，所有reader退休後修正；RustV2 content39/template79/codegen67/core-recall5共190已過。正式backend、UE buildV1、contractsV1、stagedV1已過。

目前prefix omfue/Saved/BuildTemp/recall-progress-；config rust2/world1/build1/backend1/contracts1/capture1/check1/history={rust-v1}。World64與source-check執行中、來源凍結，source3415；health每170秒。先看owner／receipt／engine及health，不重複啟動或改正式來源。World/source完成後執行visual-review-v1，目視changed像素及新recall strip，再mark proof、prepare-docs、stop／reap health、seal／verify／finalize。helpers已備，最終應24PNG、7最終worker＋1歷史worker；E1505待finalize。48h omoba-48原期限不變。一般技能channels／zones／召喚settlement等仍pending。


### Recall驗收接續修正（02:59 Taipei）

WorldV1在fixture NewObject後未Initialize就TakeWidget，WidgetTree空指標崩潰（exit3）；舊engine／stdout／stderr、fixture及來源before/verified均保留。讀UE5.8 UserWidget.cpp確認Initialize建立tree；等所有reader退休後只補fixture Initialize與三個capture model初始化。現在config rust2/world2/build2/backend1/contracts1/capture2/check2，history={rust-v1,build-v1,world-v1,source-capture-v1,source-v1}；buildV2和source-captureV2執行中。來源仍凍結，不重複啟動。buildV2後staged-v2，再world-v2／source-v2；visual-review-v1已指向WorldV2，最終190／64／3415／24PNG／7final＋5history。finalize/prepare-docs/stop/seal/verify已接V2與失敗證據，仍待全部綠燈。


---

# 回城引導進度條：最終驗收

完成complete-recall-channel-progress。共用內容模型統一既有省略default8與1..60整數秒驗證；generated registry從同一作者資料提供總秒數，bridge透過UI-only total送入owned HUD。既有compiled/runtime duration與catalog guards保留，不擴C ABI、不從首次倒數猜總長。Header使用實際共用Slate strip顯示權威剩餘時間與1-remaining/total進度，並提示移動／攻擊／施法可中斷。沒有本機計時外插，零倒數只隱藏，不冒稱回城成功。

## 驗收

- 本批Rust **190項通過／零失敗／零ignored**：content39、runtime template79、codegen67與既有core回城5項（其餘916項filtered）。實際產生、四Lua契約、正式backend與UE5.8建置成功。沒有冒稱重跑全部Rust。
- **World64成功／零Failed／NotRun**，含新增RecallChannelProgress與既有技能升級／道具提示回歸；4既有RuntimePresentationReset warning保留。
- 原生驗證半途首次快照即50%、重送pause不前進、舊tick／sequence不倒退、新快照75%、零與重啟、12種非法／未知／死亡／終局／缺protocol情況、生命替換、runtime reset、retirement與晚到資料；實際共用Strip為HitTestInvisible並能收合。
- **3415來源／資產**集合、SHA、size、mtime一致；7 staged DLL及import library一致。
- **203份證據**獨立逐byte／SHA驗證，**24張PNG**沿已目視像素精確比對及差異目視，review_source可追溯。新圖由實際HUD共用strip呈現三個published進度；不是新網路對局或權威回城完成回執。
- 健康採樣10次、最大間隔171秒；7最終worker、9歷史worker及各watcher與health均退休。RustV1因serialized Root漏欄位而2項codegen測試失敗；補Root／initializer後RustV2接受。WorldV1因fixture沒有Initialize WidgetTree就TakeWidget而崩潰。WorldV2雖64項通過，目視發現地圖時間／KDA標籤回歸；根因是新進度條排序防護擋住共用入口的舊標籤更新。所有reader退休後，改為獨立channel快照並新增標籤回歸斷言，以build／World／source V3接受。全部失敗source／log／summary、engine堆疊、V2圖片與舊來源指紋保留。

## 範圍與接續

錯誤根因／預防見errors.md及E1505。保持原回城快捷鍵、命令／傷害／控制中斷及完成移動的authority規則。既有局部HUD裁切、objective窄strip及英雄／焦點名牌交疊仍有限制。

48h omoba-48沿用2026-10-25 00:33:38 Taipei期限。一般技能channels／zones、整組召喚settlement、完整combat replica生命期、更多原創內容／美術／音效及11外部接受仍pending，MVP33/44不變。未新增真網路對局、quota reset、commit/push或omfx工作。

證據：`../acceptance/2026-10-23-recall-progress/index.json`。


## Active checkpoint: summon settlement admission (2026-10-23 03:19 Taipei)

Implemented complete-summon-settlement-admission. Core926+formation4 passed asRustV3; officialbackendV1 andUEbuildV2/stagedV2 passed;WorldV1 passed64/4 existing warnings,24PNG all reviewed. RustV1 compile import failure andRustV2 deferred-deletion test failure preserved; all failed readers retired before repairs. Source3416 capture done; source-check remains active, production inputs frozen. Prefix omfue/Saved/BuildTemp/spawn-admission-. Current health session5794, source session34837; capture session80737 to reap. Once source retires/reaped, run stop.lua, reaphealth5794, seal.lua, verify.lua, finalize.lua; prepared helpers parse. Final6 workers+3history,930/64/3416/24PNG expected. Do not duplicate tests/builds or change production while source reader runs. E1506 pendingfinalize. Per-entity admission does not finish whole-formation resource atomicity.48h omoba-48 original deadline unchanged.


---

# 召喚結算守門：最終驗收

完成complete-summon-settlement-admission。正式serial outcome settlement重新檢查召喚者存活、世代、陣營種類／隊伍、optional PlayerOwner與原有0..3600秒生命期。零秒仍為永久召喚。重複結果不覆寫、回滿血或刪除已落地單位；其他已有位置／生命元件的實體也受保護。拒絕時釋放落點預約，空白保留handle依specs正常frame boundary退休，不觸發Spawn hook或成功fact。成功spawn fact延後至實際元件落地後才送出，其他outcome排序維持原契約。

## 集中驗收

- Rust **930成功／零失敗／零ignored**：全omoba-core926，以及base_content召喚陣形4（另295 filtered）。包含5個新增結算測試、正式60Hz全rank召喚、扣費一次及pause生命周期。沒有冒稱重跑所有workspace Rust。
- 新測試透過真正process_outcomes及ECS資源：死亡／缺生命元件、隊伍／陣營種類改變、玩家歸屬變更／移除／新增、owner世代替換、永久／3600秒界限、負值／超界、未知template、封閉落點、重複結果、既有owner實體保護；成功hook與fact各一次，拒絕均為零。
- 正式backend及UE5.8建置成功；**64項Unreal回歸成功**，零Failed／NotRun，4個既有RuntimePresentationReset warning保留。7 staged DLL與import library一致。
- **3416來源／資產**集合、SHA、size、mtime前後一致；**108份證據／24張已審閱PNG**獨立逐byte／SHA校驗，圖片沿前批已審閱像素精確比對或差異目視。此批是authority結算變更，圖片只代表原生呈現回歸，不代替新玩法權威證明。
- 健康採樣4次，最大間隔170秒；6最終worker、3歷史worker及其watcher均退休。全局170秒timer保留；其clone沿用部分歷史來源路徑，當前進度另由各owned worker監看及模型讀取實際日誌／CPU確認，未用不變的歷史檔案推斷當前程序有進度。
- RustV1因test未匯入BlockedRegion而E0422；V2因誤把specs延遲刪除當成立即is_alive=false而3項新測試失敗（923通過）。全部reader退休後保留來源／日誌，再補明確import及frame boundary maintain，V3全926成功。buildV1及V2均成功，正式backend沿V1；兩者間僅cfg(test)fixture修正，正式結算來源未再改動。

## 邊界與接續

這是每個召喚實體的結算接受，不是整個formation的資源原子回滾。若一組中後段因其他outcome改變場景而拒絕，整組rollback仍pending，不能因本批勾完。未修改ABI、英雄名稱特判、平衡值或現有召喚生命周期政策。一般技能channels／zones、完整combat replica、更多原創內容／美術／音效及11項外部接受仍pending，MVP33/44不變。

錯誤詳見2026-10-23-summon-settlement-errors.md與E1506。48h omoba-48沿原2026-10-25 00:33:38 Taipei期限持續；未新增網路對局、quota reset、commit/push或omfx工作。證據：`../acceptance/2026-10-23-summon-settlement/index.json`。


## Active: authored summoner self shield (2026-10-23 03:36 Taipei)

add-summoner-self-shield adds formalcatalog3 training_self_shield_v1 (160shield/2seconds/180CD, Lua-authored), common operation/validation/generated const/model/ECS sink, shared generated OmSummonerCatalog and typed native DisplayName/Description +actual tooltip. Retains higher shield AND longer lifetime, no stacking; controls allowed/no move cancellation/healing reduction ignored. Defaults1/2 remain. Newformal60Hz shield input test passes; mainRustV2 content40/template79/codegen68/generation/core930 passed; fullbase_content long deterministic replay still progressing, then bridge-summoner filter follows. Client-runtime151/8existingignored passed. SixLua/selection cases passed; backendV1/UEbuildV1/stagedV1 passed. WorldV1 running65 including new native tooltip/capture; sourcecapture341?done (expected3420), source-check waitsRustV2 receipt. Production frozen.

Prefix omfue/Saved/BuildTemp/summoner-shield-. Owned sessions:mainRust35431;World10482;source79516;dynamichealth76256. Allother current build/backend/contracts/selection/client/capture sessions reaped. RustV1 core929/1failed regrant-duration assumption; allreaderretired, failedworld/content/hud sources preserved, descriptions/tests corrected and client-runtime enum consumer fixed. Earlyhealth16474 retired; rawhealth.jsonl preserved, newhealth-v2 dynamically discovers current cohorts and uses done-v2 marker. E1507 pendingfinalize. Need complete Rust/World/source, compare25PNGs against prior24, viewchanged+newshieldtooltip, thenstop/seal/verify/finalize and update inventory. No networkgames/quota/commits/omfx. Dedicated shield icon/art remains pending; text/card and canonical existing shield rendering are implemented. Generalchannels/zones/wholeformation atomicity/11externalACK remainpending.48h originaldeadline unchanged.


### Summoner shield acceptance profile correction (2026-10-23 03:44 Taipei)

World65 success/25PNG allreviewed,backend/UEbuild/stageV1/client151+8ignored/sixLua cases accepted on unchangedfrozen3420source. RustV2 fullbase accidentallyuseddebug; teststillhealthy but superseded to match predecessor --release fullsimulation acceptance. Exactreadyroot44208->cargo39488->test40892 verified and only test40892 stopped by native creationtoken; debug-retirement.json retained. RustV2 and dependent sourceV1 both retired/reaped as incomplete/failure, not passed. No formal source edits since V1 regrant/client-consumer repair.

Now config final={rust-v3,client-v1,build-v1,backend-v1,contracts-v1,selection-v1,world-v1,source-capture-v1,source-v2},history={rust-v1,rust-v2,source-v1}. MainRustsession65512 runs whole scoped cohort (fullbase --release), sourcesession72366 waitsRustV3 thenchecks samebefore-v1 baseline toverified-v2. Dynamichealthsession76256 continues usingdone-v2; early16474 alreadyreaped. Currentproductionfrozen, no duplicatework. Preparedseal/verify/finalize nowarchiveall3Rust versions,debugretirement,3historyand9finalworkers. FinishRust/source, thenstop/reaphealth/seal/verify/finalize. errorsMD and E1507 pendingfinalizer. Newshieldtextcaptureactualreadable; dedicatedicon remainspending.48h deadlineunchanged.


---

# 護障召喚師技能：最終驗收

完成add-summoner-self-shield。正式Lua增加catalog3 `training_self_shield_v1`護障：160護盾、2秒、180秒冷卻，舊blink1／heal2及預設D/F不變。共用作者enum／嚴格驗證、generated fixed-point operation、immutable plan與ECS sink接入現有D/F冷卻交易。None target、正有限可量化amount、duration<=60校驗；拒絕不寫Buff／冷卻，成功一次後重送拒絕。Missing BuffStore拒絕，其他既有spell仍可使用其原路徑。

使用既有召喚師配裝／選角流程，將D或F其中一格設為`training_self_shield_v1`；不自動取代原配裝。此目錄為build-time生成，修改作者數值後須重建；未宣稱新增召喚師runtime hot reload支援。

護盾沿共用damage_shield吸收／公開／到期生命期，不另造局部護盾。不累加，保留較高護盾量與較長剩餘時間；可在控制狀態使用，不取消移動，禁療不縮減護盾。正式Unreal新增共用generated OmSummonerCatalog，依catalog與spell identity相符才投影DisplayName／Description，實際D/F tooltip顯示作者說明，沒有技能名稱特判或C ABI變更。

## 集中驗收

- 本批Rust **1572成功／零失敗／8既有ignored**：content40、template79、codegen68、core930、base_content300、bridge-summoner4、正式client-runtime151。實際generation成功；不是所有workspace全面重跑。
- 新驗證包含作者非法／缺字段／unknown字段與target shape、generated identity與escaping、計畫sink失敗及重送、禁療獨立、真正ECS控制／冷卻／生命／owner／pause／phase／missing resource／wrong target拒絕、較高shield與較長lifetime重授、吸收及到期。
- 正式60Hz PlayerInput測試確認新spell接受、public buff payload163840raw、pause凍結、冷卻重送不刷新、170點直接傷害由160護盾吸收後HP只扣10，以及後續到期。全base_content既有bounded三seed×24000tick三路filtered lifecycle與其他離線回放完成；無新網路對局。
- 六項Lua契約／選角套件成功，包含新spell ID handoff；正式backend及UE5.8建置成功。**World65成功／零Failed／NotRun**，新增SummonerSelfShield實際tooltip與既有召喚師／護盾／技能／地圖回歸通過；4個既有RuntimePresentationReset warning保留。
- **3420來源／資產**集合、SHA、size、mtime一致；7 staged DLL及import library一致。**168份證據／25PNG**獨立逐byte／SHA校驗，20張與前批已審閱像素精確相同，五張差異／新圖已目視。新圖是實際共用tooltip，不冒稱新spell硬體按鍵或真網路接受。
- 9個最終worker、3個歷史worker及watcher退休，兩段health timer均退休；合計採樣10次、跨階段最大間隔171秒。V1 core929成功／1失敗是重授時限誤認2而非既有5；保留原始world／content／client-hud來源和V1全部日誌，reader退休後修測試／作者說明並補client enum consumer。V2全core930及新60Hz測試成功，但全base_content誤用debug；對齊前批release配置前，以ready Cargo與parent chain／精確native creation token只退休本輪debug測試，保留所有不完整日誌，不當作通過。V3使用release重跑全base_content並完成整個Rust範圍；sourceV1因前置cohort未完成而退休，sourceV2在同一凍結3420輸入接受，正式來源未再改動。監看V2動態列出本批各版本實際日誌。

## 範圍與接續

專用護障圖示／美術仍待補；當前使用既有文字／卡片及通用護盾呈現。fixture無player controller的主要／次要未綁定行，不代表實際遊戲預設D鍵缺失。既有first-kill局部下方HUD裁切、objective窄strip、英雄／焦點名牌重疊仍有限制。一般技能channels／zones、整組召喚資源原子回滾、完整combat replica生命期、更多原創內容及11項外部接受仍pending，MVP33/44不變。

錯誤／預防詳見2026-10-23-summoner-shield-errors.md與E1507。48h omoba-48沿原2026-10-25 00:33:38 Taipei期限持續。未新增真網路對局、quota reset、commit/push或omfx工作。證據：`../acceptance/2026-10-23-summoner-shield/index.json`。

## Completed cohort history: summoner catalog coherence (2026-10-23 04:00 Taipei)

complete-summoner-catalog-coherence adds complete serialized generated summoner catalog, startup peer gate and ContentShape reload compatibility. Covers all authored identity/text/target/operation fields and membership/order without spell-name branches. New template matrix15 cases and actual validate/reload snapshot preservation plus fresh startup rejection and later supported damage reload passed. Content40/template81/codegen68/core930 passed; full base_content --release compilation/test running, then official backend build/stage. No native/ABI/UI source changed; UE not rerun for Rust-only compatibility fix. Formal production source frozen.

Prefix omfue/Saved/BuildTemp/summoner-coherence-. Rust session68557, source session9761, direct170second health timer76694; source-capture session84064 successfully retired/reaped. Source guard copied no production files, captured3420; check waits Rust+capture receipts. No duplicate workflows. Prepared stop/seal/verify helpers must run only after all final receipts retire; stop marker then reap health, seal all logs/source/receipts, independently verify byte/SHA, update final docs/tasks/inventory/errors E1508. Wholeformation rollback/generalchannels/zones and11external parents remainpending. Original48h deadline unchanged.

### Summoner coherence final (2026-10-23 04:07 Taipei)

上述cohort已全部完成，無active reader。4/4 OpenSpec tasks checked。完整shared serialized召喚師目錄補startup peer gate及reload守門；15有效變更矩陣與actualLua拒絕保留generation/hash/content pointer，fresh startup拒絕，後續合法傷害reload成功。Rust content40/template81/codegen68/core930/base_content300共1419，零失敗／ignored；全base release181.46秒。正式backend/moba-config成功，accepted script DLL同步scripts及UEplugin，stage SHA一致；未改native/UI/ABI，未重跑UE。

3420正式輸入exactset/SHA/size/mtime一致；docs/acceptance/2026-10-23-summoner-coherence/index.json共55證據獨立byte/SHA成功，3worker+watcher及healthtimer全部退休/reap。健康5次採樣、max171秒；最終CIM無本輪Lua/Cargo/rustc/測試/Unreal。errors與E1508已更新；無新測試失敗，discovery錯誤與backend-only部署範圍已記錄。所有owned sessions68557/84064/9761/76694均結束。

接續優先具體持續引導／zones或wholeformation資源交易玩法，勿重做已完成screen-spacing與summoner目錄。護障專用美術、完整combat replica與11外部父項pending，MVP33/44不變。48h omoba-48原deadline2026-10-25 00:33:38 Taipei不變；未新增真網路對局/quota reset/commit/push/omfx。

## Completed cohort history: objective announcement banner (2026-10-23 04:20 Taipei)

complete-objective-announcement-banner adds a distinct18..36px bold public objective line under measured header, driven only by authority team/kind/sequence/remaining+phase. Shared BuildObjectiveAnnouncement used by summary+banner; HUDowner forwards every accepted hero payload including nonMOBA clear, reset/retire clear, postretirement replay staysempty. No local event timer, ABI, Rust gameplay or balance change. Initial suspected wrap bug disproved: RefreshHeaderBudget already uses explicit WrapTextAt with AutoWrapText(false); scope corrected beforeimplementation, no wrappingfixclaim. Discovery path errors recorded in objective-banner-errors.md.

UEbuildV1 success, stagedDLL/import parityverified; mainbuildsession98712 retired/reaped. WorldV1 native66 running plus fixtureexport; captureV1 and finalsourceV1 running, productionfrozen. Prefix omfue/Saved/BuildTemp/objective-banner-. Sessions World30499,sourcecapture25265,sourcecheck38919,health6632. Newtest actualbanner4cases1280/640/320 andtext2x plus11invalidsamples/allroles/reset/retire. If testfails, firstretireallreaders andsavefailedsource/logs; correctfixtureorproductionthennewcohortwithoutoverwritingV1. Sourceguardexpects3421; fullnative29PNGs=prior25+4new, keepstablecaptureflags. Build uses -NoUBTMakefiles through standardLua enginebootstrap. Sourcecapturestartedafterbuildgeneratedwritersretired.

Preparedvisual helper comparespriorSummonerShieldWorldV1 toObjectiveBannerWorldV1 withsame25subpaths; new4objective-banner images needactualview. CompleteWorld/source, visualreview,stopallreceipts/reaphealth, thenprepareseal/verify/finaldocs/tasks/inventory/E1509. Currentfeatureisreadablepresentation, notgeneralchannels/zones/wholeformationtransactions, externalparentsstillpending.48h originaldeadlineunchanged; no networkgames/quota/commits/omfx.

### Objective banner V2 acceptance in progress (2026-10-23 04:27 Taipei)

WorldV1 65passed/1failed (newbanner eight immediate expiry/reset visibility assertions),buildV1passed. Text/modelclearpassed butcachedSlatelambda visibilitylagged. PreservedV1 widget/header/owner/test andreport/log/4bannerPNG; allV1readers/watchersretired/reaped inclcapture25265/world30499/source38919 andhealth6632. sourceV1fails prerequisite,notaccepted. Afterretirement, storedSBorder andexplicitvisibility updates inauthoritysetter,HUDchromehide/show,reset/retire; addedimmediatehide/show/retirementassertions. No weakenedassertions. V2preparepartiallyclonedhelpersbeforewrongstop-v1path; correctedactualstop.lua andresumedonlybytes-identicalexistinghelpers,errorsMDrecorded.

BuildV2session55456success/retired/reaped; stageV2DLL/importverified. WorldV2session92257 **66success/0failed/notrun,4existingwarnings**,exportRust1passed; retired/reaped. captureV2session53751success/retired/reaped3421expected. finalsourceV2session74006nowchecks frozen3421 againstbeforeV2. healthV2session89969runs170sectimer; earlyV1healthretained, bothphasesunionmaxmust<=180.

29PNGreviewdone:21exactprior;8explicitviews=recall/layeredmap/firstkill/summonpose+4newbanner1280/640/320+1x/2xtext. visual.jsonallviewed withsourcehash; originalfirstkilllower/rightHUDclipandheronameplateoverlapstillpending,noDPI/full320playabilityclaim. Stop-v2afterfinalreceipts thenreaphealth;seal.lua/verify.lua preparedtoarchive4final+4historyworkers,failureoriginals,all29finalPNG,source/stage/health. Needrunseal/verifyandupdatefinaldocs/tasks/inventory/E1509. Allproductionsourcesfrozen,noWorld/buildduplication.48h originaldeadlineunchanged.

### Objective announcement banner final (2026-10-23 04:30 Taipei)

完成complete-objective-announcement-banner，4/4taskschecked，無active reader。正式HUD新增獨立18..36px目標公告，typed team/role/sequence/remaining/phase共用selector；已接受HUDsource更新，expiry/非MOBA/終局/reset/retire即時清文字與visibility，postretirementreplay不能恢復；HUDhide/show同步，HitTestInvisible不攔輸入，沿HeaderBudget/文字縮放。無localtimer／成功推測／ABI或Rust玩法更改。保留舊matchsummary文字相容性。

BuildV2/部署/import成功，WorldV2 **66success/0Failed/NotRun,4existingwarnings**，1Rustcompiled-mapexport成功。3421來源exactset/SHA/size/mtime一致；docs/acceptance/2026-10-23-objective-banner/index.json **129證據/29PNG**獨立byte/SHA成功。21圖exactprior、8差異/新增explicitview；新320+2x公告四行無裁切，非實體DPI/完整320playability。原首殺圖下/右HUD裁切與部分名牌重疊維持限制。

V1原生65success1fail(8expiry/resetvisibility斷言)，原始4source/report/log/4banner圖保留；allreaders退休後修explicitSBorder visibility、加立即HUDhide/show/retirement斷言，V2全66接受。V1source因前置World失敗拒絕；非來源成功。最初wrap推論錯誤在implementation前更正，既有HeaderBudget原已explicitwrap。helper部分clone wrongstop-v1path以正確stop.lua及bytes-idempotentresume處理，錯誤MD/E1509已更新。

4final+4historyworkers及watcher、兩段healthtimer全退休/reap；7healthsamples/max171秒。全部sessions98712/25265/30499/38919/6632/55456/53751/92257/74006/89969及seal32681結束。最終CIM無Lua/Cargo/rustc/cl/link/Unreal。接續優先具體channels/zones或wholeformation資源交易，勿重做screen-spacing、summoner目錄或公告。11外部父項pending，MVP33/44不變；48h原deadline2026-10-25 00:33:38 Taipei不變。無networkgames/quota/commits/omfx。

## Completed cohort history: cast settlement transactions (2026-10-23 04:50 Taipei)

complete-cast-settlement-transactions (2/4tasks implemented, acceptance pending). New native Outcome::ScriptCastTransaction captures caster/skill/rank/faction/optionalplayer/expectedMana, ordered effects and serialized CastSettlementCue DTO. Scriptdispatch wraps all managed successful casts and legacy summoning casts, defers cues/facts/aggression. game_processor uses localVecDeque, compares authoritative life/control/phase/mana/ownership/rank/cooldown, preflights complete boundedformation, commits allspawn bodies on exclusiveWorld using immutable statsrefs, thenresourceeffects and ScriptCastCommitted marker publishesnormalfacts/cues. Reject retiresonlyuncommittedreservations, preservesexistingbodies, no resource/cue. Mixed formation effects currentlysafe-whitelist caster mana/CD/facing+spawns only; unsupported terrain/life sideeffects rejectwholecast. OrdinaryAoE not512limited. Existingper-entityspawn path remains.

Modifiedfiles: omoba-core/src/runtime/native/comp/outcome.rs, game_processor.rs, scripting/dispatch.rs; newcast_settlement.rs andcast_settlement_tests.rs (nestedmodule undergame_processor); scripts/base_content/src/single_lane_match_tests.rs newformalpublicinput+60Hzscriptphase late-terrain test. New4coretests include12rejections,completecommit/duplicate,dependentabsolutemanaandfriendlyplacement; core934 allpassed. Base301 --release V1stillrunninglong3seed lifecycle, compiling2m34 completed; currently3oldtestsFAILED because they assertmanagedsuccesscuesBEFOREprocess_outcomes: cast_visuals_publish_only_success_for_managed_and_legacy_handlers,checked_cast_rank...,successful_cast_facts_keep_unique_order_across_dispatch_drains_in_one_tick. Newlate-terraintest/allrankformation/newphase andotherformaltests passed. Lethealthyboundedtestcomplete; no duplicate or stop.

Workers: prefixomfue/Saved/BuildTemp/cast-transactions-. RustV1session9933 (ownerreceiptwhenends; workersequentialcore/base-release/bridge/client, breakonfail); dynamic170secondhealthsession92525 (health-v1 readsALL current CastTransactions directories/versions, keepwholebatch untilfinaldone-v1.json). V1core934passed; baseexpected298passed/3failed ifnomore, bridge/clientnotrunonfailure. CurrentproductionfrozenwhileRustV1reads. NoUE/backend/sourcefingerprintworkerstarted. PreparedSavedpreserve-v1.lua ran and retained7frozenV1inputfiles includingadapterunder cast-transactions-frozen-v1/; errorsMDrecordsdiscovery/failedapplypatchandfurthermanaaudit.

REQUIRED V2 closure discovered: noncast on_tick resource hooks also use staged ledger, so afterrejectedcast their ScriptSetMana absolute write couldstillcarryfaileddebit even thoughlatercasts CASreject. MUST fixthisbeforefinalacceptance (notacceptedlimitation). Plan add native ScriptManaCommit {entity,expected:ManaPool,pool:ManaPool,notifications:Vec<ManaSettlementNotification>}; adapter adds mana_originals map captured frombegin ledger /cacheonfirststage; commit_mana_transaction gathersmatchingScriptManaSpent/Gained into this rowinstead of separate notifications+rawScriptSetMana. ProcessorCAS+life beforepoolwriteandnotifications, guarding ALL scriptcallbacks, notjustcasts. Forformationadmit whitelist ManaCommit entitycaster and validate sequential expectedpool beforeanyspawn; ordinarycallbacksguardindependent. AddnewrealECStest failedcastthenoncastManaCommitexpectedfailedP1 cannotdebit/notify, freshsnapshotretryworks. KeeplegacyexplicitrawScriptSetMana forcompatibilitytests but productionadapterusesguardedrow. Adaptercurrent test scripted_mana_transaction_checks_amount_life_identity_capacity_and_rollback expects3separateoutcomes; update to1guardedrow with2orderednotifications andfullbase/rawstate assertions. MoveSimVec2 importincast_settlement.rs behindcfg(test) (newproductionunusedimportwarning).

AfterV1readersretire/reap, implementV2+adjustoldfixtures totestnewcontract withoutweakening: assertmanagedcueabsentbeforeprocess, thenprocess_outcomes beforedrainingexpectedsuccess. Unique-ordertest currentlyrunstwo dispatchdrains withoutsettlement; process_outcomes EACHdrain, ensurefixturehasenoughmana (inspectmana_cast_fixture; preferbalance300) andpreservetwo sameTickdifferentordinalchecks. checkedrank testmoveexistingprocess_outcomes beforecues/factsassertions, stillassertalloverflow/rank/CD cases. PreservefailedoriginalalreadycopiedbeforeV1; rawlogskeep.

Need V2fullcore/base-release/bridge/client, backendbuild/stage, UE5.8buildandnativeWorld66 (noCABIchangebutRustbridge/corebehaviorchanged), sourceguardafterbuildgeneratedwritersretire (expected3423 unlessnewsourcefiles), current29PNGcomparepriorObjectiveBannerV2 andreviewchanged. ErrorregisterE1510/finalMD/tasks/inventory/handoffpending. Existing48hdeadlineunchanged. NewOutcome variantsserde DTO do NOT changecrossDLLABI; no omfx/networkgames/quota/commits. UsefixedLua/Rust1.95.0; no subagents.

### Cast transactions V3 in progress (2026-10-23 05:07 Taipei)

V1finished/reaped session9933:core934passed,base298passed/3oldpre-settlementcuefixturesfailed; base181.61sec. Saved7V1sources undercast-transactions-frozen-v1. V2 closureimplemented: Outcome::ScriptManaCommit {changes:Vec<ManaSettlementChange>,notifications:Vec<ManaSettlementNotification>}; eachchangeentity+expected/finalManaPool, notifyGained/Spent withentity. Adaptermana_originals snapshotsbeginledger/firsttouch; commitManaTransaction sortsdirtyowners,gathersallnotificationsinoriginalinvocationorder, emitsoneguardedbatch. MembershipHashSet avoidsquadraticlookups. processOutcomes checksallowners/life/poolandnotificationmembership beforeANYwrite/notify; casterformationwhitelistallowsguardedcasterresources only. New2realECS tests stalecallbackfollowingrejectedcast,multiowneratomicity/notificationorder. Test-onlySimVec2importcfgfixednewwarning. No FFICABIchange.

Old3basefixturesstrengthened: managedsuccessabsentbeforeprocess, thenserialsettlementbeforeoriginalcue/rank/policy/CDchecks. Unique-orderfixtureuses280(maxcapacity280,not300),settleseachdrainandcollectsvisuals; preserves2sameTickuniqueordinal/policy. V2allpassed **core936/base301/bridge135+1existingignored/client151+8existingignored=1523passed/9ignored**, fullbase181.44sec; session52639retired/reaped. prepare-v3session37373retired/reaped; capturespre-fastpath-v2.rs.

V3addsno-summon fastpath inplan_summons beforeworldoccupancyscan (ordinarymanagedcastsavoidO(entitycount)). OnlyeditedafterallV2readersretired. V3Rustsession16844 currentlycore936passed/fullbase301--releasehealthy boundedlonglast3seedreplay; thenbridge/clientfollow. No otherprodreader. Globaldynamichealth92525stillrunningfromV1,170sec andallcurrentCastTransactionsdirs/versions (DO NOTduplicate/restart). Currentproductionfrozen.

Acceptancehelperspreparedbycast-transactions-prepare-acceptance.lua (alreadyran): build-v1/staged-v1/world-v1/source-capture-v1. Native66all29captureflags/subpathsremainstable, rootCastTransactionsWorldV1. Backend-v1/source-v1/visual/stop/seal/verifyalsoSavedprepared. Source-v1gatesrust-v3+backend/build/world/capture final receipts; stopgates6finalworkersrust-v3/backend/build/world/capture/source plus2historyrust-v1(failed),rust-v2(passed). SealexpectsfinalRust1523/9,World66,source3423,29PNG,oneglobalhealthfinishedmax<=180; archivesV1failureinputs/logs,V2passedhistory,V3final. review.lua NOTYETcreated (mustactuallyviewdiffimages thenmarkonlycheckedsha). NoUE/backend/world/sourcecapturelaunchedyet—waitRustV3pass/reap, runbackendownedthenUEbuildowned, stageverify, capture+World+sourcecheck. SourcecaptureAFTERbuildgeneratedwritersretire. Finalsealexpectsallrunnersincludingreview.lua, whichneedcreateafterviews. errorsMD/source/taskshandoffupdatedthroughV2/V3; E1510/finaldocs/inventorypending.

Finalclaimsboundary: currentauthoredformations(+castermana/CD/facing) settlewholegroup orrejectbeforeanyspawn/fee/cue; arbitrarymixedterrain/lifeeffectswithinformationareunsupportedandrejectedentirely. Laterdependentcasts/callbacksconservativelycancelonstaleexpectedpool, freshnextinputcanretry; no temporaryrefundrebasing. OrdinaryAoEisnot512limited; onlyboundedformations<=128spawns/512effects. Existingpureper-entityspawnpathunchanged. MVP33/44,11externalparentsremainpending. No networkgames/quota/commits/omfx.48horiginaldeadlineunchanged.

### Cast settlement transactions final (2026-10-23 05:19 Taipei)

complete-cast-settlement-transactions **4/4taskschecked**，無active reader。整組作者召喚先預檢全落點／owner／資源，再用immutable stats與exclusiveWorld提交；拒絕不生成、不扣費／CD／SpentMana、不發成功cue。普通managedcasts亦採原生封套與預期法力guard；所有scriptcallback/on_tick用ScriptManaCommit多owner原始池檢查與同批有序通知，封閉staleabsoluteManaWrite旁路。無hero/camp名特判／ABI修改。無召喚的managed施法不建立全場占位snapshot。

最終V3 **Rust1523passed/0failed/9existingignored**：core936/base301/bridge135+1ignored/client151+8ignored，base-release180.65sec。6新增ECS測試、1正式input＋60Hzscriptphase late-terrain fixture及全部四rank/完整driver離線回放成功。V1core934/base298+3oldprecommitcuefixturefails原始7sources/logs保留；V2全1523pass/9ignored為prefastpathhistory，全部reader退休/reap後才改fastpath；V3全scope同一最終凍結來源接受，不混用舊revision。3舊fixture加強before-settlement absence後保留原rank/identity/policy/CD與uniqueordinal要求。

正式backend/moba-config及UE5.8build成功，stagedDLL/importSHA一致。World66/zeroFailed/NotRun、4existingwarnings；29PNG=25exactprior+4explicitview(recall/map/firstkill/summon)。圖片僅呈現回歸，新權威行為由ECS/input測試證明。3423formalinputs exactset/SHA/size/mtime不變；docs/acceptance/2026-10-23-cast-transactions/index.json **153證據**獨立byte/SHA通過；13healthsamples/max171秒。6final+2historyworkers/watchers及單一全階段170sec healthtimer均退休/reap，finalCIM無Lua/Cargo/rustc/cl/link/UE/base程序。sessions9933/52639/37373/16844/83705/22361/48071/21713/98723/92525及seal1623均結束。

final-acceptance/errorMD、E1510、盤點/tasks已更新。範圍是現有作者召喚＋castermana/CD/facing完整接受／拒絕；任意混合terrain/life副作用召喚不支援並整筆拒絕，不冒稱任意script回滾。後續依賴staleMana的casts/callbacks保守取消，新輸入可重試。顯式native raw ScriptSetMana保留兼容，production scripttransaction走guardedrow。舊首殺圖下/右HUD裁切與部分hero/focus名牌重疊仍限制。

NEXT優先具體持續引導／脈衝zones或原創技能玩法，勿重做已完成screen-spacing、summoner目錄、公告或currentformation結算。完整combat replica、更多控制／內容及11外部父項pending，MVP33/44不變。48h omoba-48原deadline2026-10-25 00:33:38 Taipei不變；無networkgames/quota resets/commit/push/omfx。

### Periodic damage acceptance in progress (2026-10-23 05:58 Taipei)

本批 complete-periodic-damage-lifetime 已完成DoT有效生命期／分數餘數／穩定順序與逐rank遊俠流血、公開canonical bleed binding。V2全Rust1771passed/9existingignored，backend/UE5.8/buildstage成功；World66/4existingRuntimePresentationReset warnings成功，29PNG=25exactprior+4已明確view，visual.json全已審閱。V1兩個舊瞬間HP fixture失敗原始10sources/logs已保留後修正，新玩法fixture V1/V2均通過。

目前**只有 final source check 尚在跑**，不得重複啟動。prefix omfue/Saved/BuildTemp/periodic-damage-；source-v1 owned launcher execsession48576；單一健康timer health-v2 execsession46339每170秒。source-capture-v1已退休/reap（session40325）；World已退休/reap2172，Rust39005/backend1439/build11848均退休/reap。完成source check後retire/reap、stop.lua停timer並reap46339，再執行seal.lua 2、verify.lua與更新final acceptance/tasks/盤點。sealer與visual review helpers均已備妥，所有來源仍凍結，禁止先改Rust/C++。

重要NEXT新發現：共用legacy script/DoT直接Outcome::Damage未走基礎護甲／抗性，而DamageInstance已先算過；predeclared=false兩者都有，不能全域重算造成double-mitigation。優先建立明確raw-vs-settled共用結算並驗證雙路徑、TD/shields/歸因，再做channels/zones。詳見本批errors MD Follow-up；本批只接受raw DoT生命期與既有shield/packet語意，不冒稱armor parity。


### Periodic damage final (2026-10-23)

前述in-progress已完全結束，無active reader。complete-periodic-damage-lifetime **4/4taskschecked**。V2 Rust1771passed/0failed/9existingignored（ABI15/content42/templates115/codegen68/core941/base304/bridge135+1/client151+8）；base182.71sec。backend／UE5.8／stage/import及World66/4existingwarnings成功；165份證據／29PNG、3425輸入一致、health 13 samples／max171秒，已獨立驗證。29PNG=25exactprior+4explicitview；舊首殺HUD裁切、hero/focus重疊保持限制。6final+2historyworkers及2healthtimers退休、全部sessionreap，finalCIM無本批工作程序。V1兩項舊fixture失敗／10原始sources、監控v1前綴錯誤及template漏feature均保留，最終scope全由V2接受。

NEXT第一優先**共用raw-vs-settled傷害結算**：script/DoT直接Outcome::Damage缺基礎護甲／抗性，DamageInstance路徑已先算過，predeclared兩路都false；需明確raw packet入口與一次防禦結算，涵蓋physical/magical/pure、buff防禦、shield、TD profiles／歸因，避免全域double-mitigation。先讀本批errors.md Follow-up與final-acceptance.md，再列計畫實作。不要重跑已完成生命期或只做複核。之後接channels／zones與其餘單機玩法。DoT本批跨頻率保證是raw scheduled amount，不擴張到任意post-modifier HP rounding。完整combat replica、11外部父項及MVP33/44維持pending。原omoba-48期限2026-10-25 00:33:38 Taipei不變，不新增真網路對局／quota reset／commit/push／omfx。

### Raw damage in progress (2026-10-23)

complete-raw-damage-settlement tasks1/2checked。RawDamage專用入口在serial settlement先套共享固定點防禦，再進原Damage/ProjectileDamage的packet modifier/shield/credit；DamageInstance同用純math helper不重複套。Checked buff armor/resist依Hero CP、Unit、CP-only取base；pure bypass，TD layered保留profile/layer規則。新增serialized Projectile.damage_stage，舊資料defaultSettled，所有新原生homing/directional/script/backendlegacy factory明確Raw。無C ABI/public protocol更動，omfx未碰。

目前 full RustV2正在client收尾，execsession47547；不得重複啟動。prefix omfue/Saved/BuildTemp/raw-damage-；170秒health-v2 timer execsession10119，prefix11/9已調整正確。FocusedV1 core938/9failed（5新fixture少TowerTemplateRegistry、4舊退休source pure flight用無效profile0），原12sources/logs已保存frozen-focused-v1；session97824退休reap後修正registry與TRUE mask，保留原credit/wallet/gen要求。FocusedV2 core948/0fail、session44326退休reap。後加兩個正式60Hz input新test與11個script-output RawDamage斷言遷移，fullV2 core948/base306成功（base181.97sec）、bridge通過、client尚待最終receipt。

下一步：等fullV2所有reader自然退休/reap，執行raw-damage-prepare-v3.lua保存19正式來源／生成V3 runner，然後補RawDamage conversion的remove_uids target守門及一個ECS fixture：Death(target)先於RawDamage，同批不能再扣目標護盾。這個guard尚未實作，勿誤當已完成。再跑全RustV3、正式backend/UE5.8/staged、World66、source-capture/check。raw-damage-seal.lua動態接受version3並保存focus1/2＋fullV2歷史及兩組frozen inputs；visual/review/stop/verify helpers已備妥。不得先改正在被讀的Rust來源。

最後關timer10119後seal/verify，補E1512/final MD/盤點/tasks/handoff。繼續原deadline 2026-10-25 00:33:38 Taipei；完整replica/11外部/MVP33/44仍pending，後續優先具體channels/zones或經證實的玩法缺口，不重做已完成nameplate-spacing。

### Raw damage V3 final acceptance running (2026-10-23 06:42 Taipei)

上段RustV2／未實作guard資訊已過期：V2全1780passed/9ignored成功並退休reap；保存19inputs後新增retirement red test重現0passed/1failed，保存兩原始sources並退休reader後加remove_uids target guard。最終V3全1781passed/0failed/9existingignored（core949/base306，base181.36sec）成功、session4797退休reap。backend44087與UE5.8 build93952亦成功退休reap，staged DLL/import SHA verified。所有正式來源凍結。

目前WorldV1 session81251、source-capture-v1 session67177執行中，不得重複啟動；單一170秒health-v2 session10119仍跨階段監控。完成後退休reap，再visual.lua比對29PNG，實際view changed圖後review.lua傳實際檢視路徑；source-v1需等所有前置receipt成功，完成後退休reap。最後stop.lua／reap10119、final CIM、seal.lua 3、verify.lua；raw-damage-final-docs.lua已備妥但尚未執行，只能獨立verify成功及所有程序退休後跑。sealer包含focus1/2、fullV2、red失敗共4history與3組frozeninputs；最終V3是唯一最終scope依據。不得混用或覆寫失敗證據。

06:44 Taipei更新：World66（65success+1successwithwarnings／4existingwarnings／zeroFailed/NotRun）成功，81251已退休reap；source-capture67177亦成功退休reap。29PNG=25exactprior+4已view並review.sha核對，全viewed。現在只有source-v1 session65172正在做最終一致性check，health10119仍單一全階段監控；不得重複啟動。等source65172成功退休reap後按上述stop/seal3/verify/final-docs順序執行。


### Raw damage final (2026-10-23)

前述in-progress已結束。complete-raw-damage-settlement 4/4taskschecked；同批remove_uids target guard已實作，retirement red原始失敗及兩sources保留。最終V3 1781passed/0failed/9existingignored；3426正式輸入一致；221份證據／29PNG；health15 samples／max171秒。backend/UE5.8/staged/import與World66成功，29PNG已審閱。6final+4historyworkers/watchers及health-v2 timer全部退休reap，無active reader。原V2全1780pass／focus1失敗與focus2成功的原logs/frozen inputs均封存；V3完整scope是唯一最終依據。

共用RawDamage先讀live防禦再走一次packet/shield/credit；DamageInstance共用math保留settled。舊projectile defaultSettled，新native/script/backend factories明確Raw，無CABI/protocol變更。保留已退休source合法flight，拒絕已退休target後續shield扣除。pure/TD/invalid/來源身份／正式60Hz箭流血魔法抗性均驗證。舊首殺裁切與hero/focus重疊保持限制。NEXT具體channels/zones或其他經證實單機玩法缺口；不要重做nameplate-spacing／raw settlement。11外部父項、MVP33/44 pending。omoba-48原期限2026-10-25 00:33:38 Taipei不變；無真networkgames/quota reset/commit/push/omfx。

### Ground damage zones in progress (2026-10-23 07:00 Taipei)

Raw-damage cohort is fully complete: 4/4tasks/E1512, Rust1781/9ignored/backend/UE5.8/World66/staged/import/source3426 accepted; 221evidence/29PNG independently verified,15health/max171sec, allworkers/timer retired/reaped. See raw-damage final acceptance; do not redo.

NEW complete-ground-damage-zones: caster-owned anchored pulse scheduler via checked stat-buff JSON, no GameWorld ABI extension (table full). DamageZoneSpec validates exact scalar/kind/profile/geometry; BuffStore reverse-index and per-entry clock, first pulse after interval, final exact pulse before expiry, refresh replaces anchor/lifetime and resets clock, remove/death cancels exactsourcegeneration. live exact-circle living enemies only, excludesbuildings/allies; emits RawDamage and existing Explosion cue eachpulse. mininterval0.25/maxduration30 (max120catchup), current tick geometry for catchup; not historical sub-tick reconstruction. Shared point_blast::with_zone fully admits before any outputs, fullabilityID/casterID/gen key. Flameassault keeps200/300/400/500 initialmagic+stun, adds25/35/45/55 per0.5s for3s; Lua description derives fromextras; preview addssecondAreaEffect. No hero/camp execution specialcase. Casterdeath cancelszone; not independent post-death zones.

6newcore tests (3scheduler/3ECS),3newbase tests (metadata rejection+2formal60Hz) added. RustV1 stoppedatcorecompile E0599 missing specs::Builder innewtestmodule; ABI/content/templates/codegenpassed, no behavior testresults. session87203/owner/watcher retiredreaped. Preserved10 exactsources underground-zones-frozen-rust-v1/manifest.json beforeimportfix. V2 full8scope RUNNING execsession19226, currentlycore compiling; freezeallproduction. Single170sec health-v1 execsession28312, dynamicprefixuses# notmagicnumbers; do NOTrestarttimer or duplicate tests.

Prefixomfue/Saved/BuildTemp/ground-zones-. Acceptancehelpers prepared backend-v1/build-v1/staged-v1/world-v1/source-capture-v1/source-v1/review/visual/stop/seal/verify. source-v1 now gatesrust-v2. Worldroot GroundZonesWorldV1, native66/29captureflags unchanged, priorRawDamageWorldV1/2026-10-23-raw-damage. V2success→retire/reap→backendowned→UEbuildowned→stageverify→World+sourcecaptureparallel→retire/reap→visualandactuallyviewallchangedimages→review→sourcecheckowned→retire/reap→stophealth/reap28312→finalCIM→seal.lua 2→verify.lua→finaldocs/tasks/盤點/E1513/handoff. SealexpectsRust>=1790 (core955/base309 assumingnoextratests),9existingignored,source>=3427,29reviewedPNG,health<=180; includesallolderRustversionsandtheirfrozenmanifests. Final-docs helper NOT YET written. Source list10files listedsealer; preserve10perfailurebeforeanyfix. Nativeandbase tests notyetfullyrun; do NOTclaimzonesaccepted.

NewerrorsMD2026-10-23-ground-zones-errors.md recordspathguesses,initialzonekeycollisiondesigncorrectedbeforetests,flattenedmetadatafixturecorrectionandV1Buildererror. NoUE/backend/sourcefingerprintworkerstartedyet. No真networkgames/quotareset/commit/push/omfx. Originaldeadline2026-10-25 00:33:38 Taipei unchanged;11external/MVP33/44 pending.

GroundZones07:09Taipei update: RustV2 all1790passed/0failed/9existingignored (core955/base309,base184.66sec) success; session19226 retired/reaped. All new6core/3base tests passed. backend32246 success retired/reaped. UEbuild-v1 exec85285 RUNNING; do not duplicate. Source frozen. Health-v1 singleexec28312 remains170sec acrossallphases. Tasks1/2checked;3/4pending. ground-zones-final-docs.lua nowprepared (syntaxvalid), onlyrunafterseal2+independentverify+allprogramsretired. No moreproductionchangesplanned forcurrentfinalscope. Otheracceptancehelpersprereq rust-v2 confirmed. NEXT waitbuild85285retire/reap→stagedverify→WorldV1/sourcecaptureV1parallel→actualviewchangedimages→review→sourcecheck→stop/reap28312→CIM→seal2/verify/finaldocs. E1513pending. Originaldeadline/11external/MVP33/44unchanged.


### Ground damage zones final (2026-10-23)

前述in-progress已結束。complete-ground-damage-zones 4/4tasks/E1513，1790passed/0failed/9existingignored；3427正式輸入一致；148份證據／29PNG；health10 samples／max170秒。backend/UE5.8/staged/import與World66成功、29PNG已審閱、證據獨立bytes/SHA驗證，所有workers/watchers及單一health-v1 timer退休reap，無active reader。V1缺Builder編譯失敗及10原始sources/logs保留，各後續歷史版本亦封存，最終scope全以最後完整revision接受。

DamageZoneSpec+BuffStore reverse-index/perentry clock，固定落點、live exactcircle enemy/allies/building政策、首次interval後／finalpulse先於expiry、refresh replace+reset、remove/source death+generation retirement取消、pausefreeze、boundedcatchup。RawDamage防禦/shield/credit一次。Shared point_blast::with_zone預檢全spec後輸出；Flameassault瞬間blast/stun不變，加25/35/45/55每0.5秒3秒6pulses，metadata/description同extras。沒有GameWorld ABI擴充或英雄營地名稱特判。這是caster-bound zone，非獨立postdeath lifetime或channel；existingExplosion pulsecue非專用地面美術。

NEXT依具體單機缺口繼續，優先真正持續引導／其他原創內容與操作呈現；不要重做raw settlement／groundzone scheduler／nameplate-spacing。11外部/MVP33/44與完整combatreplica pending。原deadline2026-10-25 00:33:38 Taipei不變；無真networkgames/quota reset/commit/push/omfx。

### Ground zone public status in progress (2026-10-23)

Prior ground-zones cohort fully accepted4/4/E1513: Rust1790/9ignored,backend/UE5.8/World66/source3427,148evidence/29PNG/health10max170,allretired. Do not redo.

NEW complete-ground-zone-status: append-onlyground_damage_zone buff("持續區域"); BuffVisualState onlyadmitscheckedDamageZoneSpec+remaining(0,30], rejectsforgedcanonicalname/malformedpayload; aggregatesmaxremainingwithoutprivatekey/coords/JSON. handle_death immediatelycancel_damage_zones. Existingownedhero-detailsUI renderslocalizedcountdown. Lua zone descriptioncollapsesidenticalrankvaluesandusesgeneric傷害label. NativeOmGroundZoneStatusTests.cpp addsOm.Runtime.GroundZoneStatus realinitializedUObject+Slate tooltipcapture+owner/gen/expiry/scopechecks. New3core/1baseformal60Hz/1bridgeprojectiontests. NoCABI/newwirefields.

CurrentFULLRustV1exec25724: core958passed,ABI/content/templates/codegenpassed,base--releasecompiling. FreezeallformalSources while readeractive. Single170secondhealth-v1exec65845, prefixomfue/Saved/BuildTemp/ground-zone-status-, CamelGroundZoneStatus. DoNOTduplicate/restart. No backend/UE/World/sourceworkerstartedyet. Tasks1checked,2-4pending.

PreparedworkersRustV1/backend-v1/build-v1/staged-v1/world-v1/source-capture-v1/source-v1/visual/review/stop/seal/verify. Source-v1prereqRustV1. NativeWorld67(addedGroundZoneStatus),30PNG(prior29pluszone-status/zone-status.png),priorGroundZonesWorldV1/2026-10-23-ground-zones. Sealeracceptsversion1,expectedRust>=1795 (core958/base310/bridge136+existingignored),source>=3428(newCPP),6finalworkers/nohistoryifV1pass. Sources7listatsealerend. NewCPPisnotyetcompiledoraccepted. EnsurestandardUEbuild-NoUBTMakefilesdiscoversfile. Independentlyviewallchanged29priorimagesplusnewzone-statusPNG,thenreviewactualpaths. Final-docshelperNOTYETWRITTEN;E1514pending. Onanyfailedreaderretire/reapandpreserve7exactsources+logs beforefix;adaptfinalworkernames/version/historygatesratherthanoverwritingevidence.

Next waitV1complete/reap25724→backendowned→UEowned→stagedverify→World67+sourcecaptureparallel→retire/reap→visual/view/review→sourcecheckowned→retire/reap→stop/reap65845→finalCIM→seal1/verify→finaldocs/E1514/tasks/盤點/handoff. SourcefreezeallthewayafterRustfinal. Originaldeadline2026-10-25 00:33:38Taipei/11external/MVP33/44unchanged. No真networkgame/quotareset/commit/push/omfx. ErrorMDrecordsplaceholderremovedbeforecompile,wrongruntime.rspathandfixedreviewrowcountmessage.

GroundZoneStatus07:33Taipei update: RustV1 all1795passed/0failed/9existingignored (core958/base310/bridge136+1/client151+8),base182.74sec;25724retired/reaped. AfterallRustreadersretired,preserve-scope.lua saved2pre-UE C++inputs+manifest underground-zone-status-pre-scope-cache; addedSetBuffListSnapshot(empty)atRetireGameplayInputbeforeidempotentguard, nativeassertcacheemptyandreactivatesameherodoesnotreviveoldrow. NoRustsourcechangesafterV1. FirstUEbuild/World67mustvalidatefinalCpp; notclaimingunrunredtest. Sealerincludespre-scope-cache3filesandfinal8sources(includingOmCommandBarWidget.cpp). backend-v1 RUNNINGexec54999. Healthsingle65845 remains170sec;doNOTduplicate. Otherhelpersandfinal-docs.luaallprepared,Native67/30PNG/source>=3428. Fullsourcefrozenagain. Nextbackendretire/reap→UEbuild→stage→World/sourcecaptureparallel→visualactualviewsinclnewzone-status→review→sourcecheck→stop/reap65845→CIM→seal1/verify/finaldocsE1514. Originaldeadline/11external/MVP33/44unchanged.


### Ground-zone public status final (2026-10-23)

前述in-progress已結束。complete-ground-zone-status4/4tasks/E1514，1795passed/0failed/9existingignored；3428正式輸入一致；117份證據／30PNG；health7 samples／max171秒。backend/UE5.8/staged/import與World67成功；新增實際Slate zone-status PNG，全部30PNG已審閱，證據獨立bytes/SHA通過。所有worker/watchers與單一health-v1 timer退休reap，無active reader。

BuffVisualState從checkedDamageZoneSpec與boundedremaining映射append-onlyground_damage_zone("持續區域")，拒絕forgedname/invalidpayload，多區域maxremaining合併；只publiccatalogID/time，無privatekey/coords/JSON。handle_death即時cancelzones。Hero details既有ownedbufflist顯示倒數；typedsource/reset/generation/gate保持。Lua rank prose合併相同值與generic傷害標示。3core+1base正式60Hz+1bridge+1native真widget／Slate測試；本批未新增常駐進度條或獨立postdeathzone。

NEXT具體持續引導、偵測／trinket內容或其他經證實單機玩法缺口；勿重做已接受nameplate/raw/groundzone與status。11external/MVP33/44與完整combatreplica pending。原deadline2026-10-25 00:33:38Taipei不變；無真networkgames/quotareset/commit/push/omfx。

### Detection trinket in progress (2026-10-23 07:52 Taipei)

NEW complete-detection-trinket: append-only catalog2 training_detection_lantern_v1 偵隱燈, visible stealth0/detection2/range600/sight600/life60/recharge180/onecharge/HP4/oneplacement. Existing authority loadout+admitted ward queue+placement/vision/retirement mechanisms, no wire/CABI extension or name specialcases. Generated native identity includes authored description/range/sight/lifetime; shared BuildWardCatalogDetails feeds existing valid-owner tooltip with dynamic state intact. No in-match selector UI or dedicated icon added. Tests: 1template/1codegen/2base (real queue+Production60Hz step, pause/deadlines/recharge; detection levels/exactradius/LOS/team/destruction) plus Om.Runtime.DetectionTrinket real initialized widget/shared Slate capture.

FULLRustV1 RUNNING exec5241, prefix detection-trinket-, root DetectionTrinketRustV1. Freeze all formal sources while readers active. Single170sec health-v1 exec11171 already running; DO NOT restart/duplicate. Helpers prepared rust/backend/build/staged/world/sourcecapture/sourcecheck/visual/review/stop/seal/verify. NativeWorld68/31PNG, prior GroundZoneStatusWorldV1/2026-10-23-ground-zone-status, new detection-trinket/detection-trinket.png. Expected1799passed/9existingignored/source>=3429; only accept actual results. Eight modified inputs listed sealer. E1515pending, final-docs helper not yet written. If failure retire/reap and freeze exact8sources+rawlogs BEFORE fixes; adapt version/history prerequisites without overwriting failures.

Next Rust→backendowned→UEowned→stage→World68/sourcecaptureparallel→actualviewchangedimages→review→sourcecheckowned→stophealth/reap11171→finalCIM→seal1/verify→finaldocs/tasks/inventory/handoff. External11/MVP33/44 remainpending; no realnetworkgames/quotareset/commit/push/omfx. Original unattended deadline unchanged.

DetectionTrinket update07:57Taipei: V1failed oldLua-loader rowcount; V2failed fixture teamu32→Factioni32 mismatch; V3mistakenconversionu8 alsofailed. Allthree owner/watchers retired/reaped (5241/18513/98601), original9sourcesperversion+manifest/logs preserved. Correctchecked i32conversion afterreadingactualsignature. FULLRustV4 RUNNINGexec33715; freezeformalsources. Singlehealth11171 remainsrunning170sec. source-v1prereq nowrust-v4; sealerincludesall3historicalversions and9finalsources (addedlua_content.rs). Final-docs.lua prepared but mustnotrunbeforeallacceptance+seal4/independentverify. OtherhelpersunchangedWorld68/31PNG/source>=3429/expected1799+9ignored. NoUE/backend/native/sourceworkerstartedyet.

DetectionTrinket08:02Taipei update: FULLRustV4 all8scopes succeeded, base312/0failed/180.70sec; totalexpected1799/9existingignored. Allnewtests passed. Session33715retiredreaped. Backend-v1nowrunning (read current launcher receipt/process; nexttooloutputhasexecid). Tasks1/2checked;3/4pending. Formal sourcefreeze continues, healthsingle11171. Noadditionalproductionchangesplanned. Nextbackend→UE→stage→World/sourcecapture→visual/review→sourcecheck→stop/reap11171→seal4/verify/finaldocsE1515.

DetectionTrinket08:06Taipei: backend14774successretiredreaped. UE V1C1083 RuntimecannotincludeOmGenerated (reverse dependency);3430retired/reaped,14exactpre-fixfiles+manifest/rawlogpreserved. FixedGenerated bridge→existingOwnerWardHudView metadatafields→Runtime sharedformatter; no cycle/newwire/CABI. AddedoldUiTextureImport unknownID65535 fix and includeexistingnative test, Worldnow69/31PNG. Finalmodifiedsources12, Rustunchangedsince1799V4pass. UEbuild-v2 RUNNING (nexttooloutputexecid), source-v1/staged/sealerprereqsadaptedtobuild-v2; sealerincludesfailedbuild-v1receipt/log/frozen14sources and3olderRustversions. Nextbuild2→stage→World69/sourcecapture→visual/review→sourcecheck→stop/reap11171→seal4/verify/finaldocsE1515. Healthsingle11171stillactive170sec.

DetectionTrinket08:12Taipei: UEbuildV2success/reaped4424; stage/importverified. WorldV1prelaunchduplicate DetectionTrinket name fromoverbroad Camelrename; nativecommandvalidatorrejectedbeforeUEstart, originalrunner/exportgeometry/logsretained. WorldV2correct69unique:69executed/68success+1existingwarningcase/4existingwarnings/0failed/notrun;64665reaped. sourcecaptureV1success60951reaped. 31PNG reviewed:26exactprior+5actualview (recall/map/firstkill/summon/newdetector). NewdetectorChineseSlatefullreadable; oldfirstkillbottom/rightcrop andpartialmaphero/focusoverlapremainknownlimitations. Sourcecheck-v1 RUNNING (nexttooloutputexecid), prereqRustV4/backendV1/buildV2/worldV2/captureV1. Healthsingle11171active. Nextsourcecheckretire/reap→stop/reap11171→CIM→seal4/verify/finaldocsE1515. Sealerarchivesfailedworld-v1runner/receipt/exportlogs, failedbuild-v1+14freeze,3failedRust+9freezeeach, final12sources. No formalchangesafterUEV2.


### Detection trinket final (2026-10-23)

前述in-progress結束，complete-detection-trinket4/4tasks/E1515，1799passed/0failed/9existingignored；3429正式輸入一致；246份證據／31PNG；health12 samples／max171秒。backend/UE5.8/stage/import/World69成功；31PNG已審閱，所有final/history reader/watchers與health11171退休reap。失敗原始sources/logs和最終12sources保留，來源／證據一致性獨立驗證。

Catalog2原創visible detector經既有authority loadout／admitted queue／game-time與vision通用機制；偵測1/2可見3不可見、exactradius／LOS／team／destruction及pause／expiry／recharge測試成功。Generated native description/range/sight/lifetime→shared BuildWardCatalogDetails→既有validowner wardtooltip，dynamicstate與authority gate保留。無對局內selector／專用icon／新wire/CABI。

NEXT依盤點繼續具體單機玩法／呈現缺口，優先持續引導或已證實操作缺口；不要重做已接受nameplate/raw/zone/status/detector機制。11external/MVP33/44與完整combat replica pending。原deadline2026-10-25 00:33:38Taipei不變，無真networkgames/quotareset/commit/push/omfx。

### Hero level feedback in progress (2026-10-23 08:22 Taipei)

Prior detectiontrinket4/4/E1515 accepted1799/9ignored,World69,3429inputs,246evidence31PNG,allretired. Nameplate4/4alreadyaccepted. NEW complete-hero-level-feedback: native-only FOmHeroLevelFeedback observes existing authority HUDpayload; firstvalidsamplebaseline, player/exactentitygeneration scoped, highestlevel+elapsedmonotonic replayguards; latestincrease coalesces4secondpresentationnotice, no XP prediction. End/missing/retirement clears; retiredHUD cannotrevive. Realcommandbarheader uses localized text/readable font/wrap/showHUD andlayout invalidation. Source5files: newtrackerh/cpp/newOmHeroLevelFeedbackTests.cpp pluswidgeth/cpp. NoRust/backend/content/wire/CABIchange.

UEbuild-v1 RUNNING (nexttooloutputexecid), freezeformalsources. Single170sechealth-v1 exec61060; DO NOT duplicate. Helpers hero-level- prefix; World70addsHeroLevelFeedback whilepreserving existing69identifiers, capturehero-level/hero-level.png; total32PNG. PriorDetectionTrinketWorldV2/index2026-10-23-detection-trinket. Rust/backend1799acceptedcohort referencedwithunchanged-sourceproof, notclaimednewrerun. Formalcount expected3432(3newfiles). sourcecapture/check useexistingguard. Preparesealer/finaldocs whilebuildrunning; E1516pending. Onfailedreaderretire/reap+preserve5originalsources+logsBEFOREfix. Nextbuild→stage→World70+sourcecaptureparallel→actualviewchangedPNG→review→sourcecheck→stop/reap61060→CIM→seal/independentverify→finaldocs/tasks/inventory/handoff. Originaldeadline/11external/MVP33/44unchanged.

HeroLevel UE update (exact UTC in receipts): UE V1success52.39sec thenretired/reaped35266. Preflightread signedFOmEntityRef andhardennegativegeneration/outsideu32ID beforefirstnativetest;5pre-fixsources+manifestpreserved, notredtest. UE V2success5.69sec/13769retiredreaped, staged/importSHAverified. WorldV1exec54506/sourcecaptureV1exec34757 nowrunning, formal sourcesfrozen. Singlehealth61060active. Sealer.lua accepts2, archivesV1successfulbuild+5freeze/history, validatesWorld70/32PNG/source3432, bidirectionalallRust/Lua/Cargo/proto inputsunchangedvsacceptedDetectionTrinketindex (1799+9ignored inheritednotrerun). final-docs.lua readyE1516 pending. NextWorld/captureretire/reap→visual actualviews/review→sourcecheckowned→retire/reap→stop/reap61060→CIM→seal2/verify/finaldocs. Tasks1/2checked,3/4pending.

HeroLevel native/visual update: WorldV1 all70executed,69success+1existingwarningcase/4warnings/0failed/notrun;54506retiredreaped. CaptureV1success34757retiredreaped. NewHeroLevelFeedbacktestpassedfullboundaries+actualwidgetheaderpixels. All32PNGreviewed:27exactprior+5actualviews (recall/map/firstkill/summon/newhero-level); noticefullreadableChineseGoldline. Oldfirstkillcrop/mapnameplateoverlapunchangedknown. Sourcecheck-v1 RUNNING(nexttooloutputexecid), freezeformalsources. Singlehealth61060active170sec. Nextcheckretire/reap→stop/reap61060→CIM→seal2/verify/finaldocsE1516. Final-docs/sealerprepared, nootherworkrunning.

HeroLevel source-wrapper correction: sourceV1actual3432fingerprintchecksuccess/proofwritten, butlauncher summary"-"requiresexit=0stdoutmarker omittedinnewrunner. Supervisorreceiptfalsepreserved;9854retiredreaped. Noformal/gameplaychanges. Newsource-v2RUNNING(nexttooloutputexecid), printscompletionmarkerandwritesfreshsource-verified-v2.json. Sealerfinalworkeradaptedsource-v2, archivesfailedsupervisionV1runner/receipt/logs/actualsuccessproof; historicalcount2(buildV1+sourceV1)when seal2. Finaldocsneedmentionthiswrappererrorbeforecompletion. Healthsingle61060stillactive;finalizeafterV2successfulretire/reap→stop/reap61060→CIM→seal2/verify/finaldocsE1516.


### Hero level feedback final (2026-10-23)

前述in-progress結束，complete-hero-level-feedback4/4tasks/E1516，World70全執行／0failed／0notrun；3432正式輸入一致；101份證據／32PNG；health7 samples／max171秒。UE5.8/stage/import/World70成功。32PNG全部審閱；newactualheaderhero-level.png。Rust/backend/content未變更，前批1799/9ignored接受以雙向檔案集合/SHA繼承，不宣稱重跑。全部worker/watchers/health61060退休reap；5V1preflight來源及final5sources、接受證據封存獨立核對。

FOmHeroLevelFeedback: firstbaseline、owner/exactentitypositivegen、highestlevel/elapsedmonotonic、newestlevelcoalesce、4sec localpresentation timer、missing/invalid/nonplaying/scope/retirementreset。Realcommandbarheaderreadable/wrapped/showHUD，noXPprediction/newwire/CABI/hero namecase。收到跳級只公告最新值，不捏造漏收中間事件。

NEXT持續引導／其他具體單機玩法呈現缺口；不要重做已接受nameplate/raw/zone/status/detector/levelnotice。OldfirstkillHUDcrop與部分hero/focusoverlap仍known。11external/MVP33/44與完整combatreplica pending；原deadline2026-10-25 00:33:38Taipei不變。無真networkgames/quotareset/commit/push/omfx。

### Cast-time modifiers in progress (2026-10-23)

Priorhero-level4/4/E1516acceptedWorld70/source3432/101evidence32PNG; allretired. Nameplate4/4done. NEWcomplete-cast-time-modifiers: existingCastTimePercentage("casttime_percentage")wasunusedbyproductionwindup. UnitStats::checked_cast_time useschecked_sum_add familyselection, checked1+bonus, existing102rawfloor, i128multiply/1024, positivebase min1raw/max60sec; invalid/overflow/overbudgetrejectstartwithoutresource/effects. Instantstaysinstant. Dispatchsnapshotsdurationonlyatstart; midcastdrift/expirydoesnotretime/reapplyat release. Nohero/campcase/newcontent/wire/CABI. Thisispreparationmodifierfix, NOTsustainedchannelmechanics.

2newcoretests checkedarithmetic/family/float/bounds;2newbaseformal60Hztests haste/slow/floor/pause/snapshot/actualsummonrelease/manaCDonce/invalidreject/instantunaffected. NewOmCastTimeModifierTests.cpp nativeOm.Runtime.CastTimeModifiers checksrealinitializedwidgeteffectivefraction/stableinstance-total/newinstance/positive<0.1slabel/retirement;actualabilityreadouttagcapturecast-time/cast-time.png. Widgetrejectssameinstance-totaldrift andshows<0.1sinsteadofrounded0.0.

FULLRustV1RUNNING(nexttooloutputexecid), freezeallformalsources. Single170sechealth-v1exec14433; DONOTduplicate. Prefixcast-time-, CamelCastTime; helpersfull8scope/backend/build/staged/world/sourcecapture/sourcecheck/visual/review/stop/seal/verifyprepared. Expected1803passed/9ignored(core960/base314), nativeWorld71/33PNG/source3433. PriorHeroLevelWorldV1/index2026-10-23-hero-level. Sealeracceptsversion1and5finalsources:unit_stats.rs/dispatch.rs/single_lane_match_tests.rs/widget.cpp/newNativeCPP. Onfailurefullyretire/reapandfreeze5inputs+logsBEFOREfix. Final-docsnotyetprepared/E1517pending. NextRust→backend→UE→stage→World71+sourcecaptureparallel→actualchangedPNGview/review→sourcecheck→stop/reap14433→CIM→seal1/verify→finaldocs/tasks/inventory/handoff. Originaldeadline/11external/MVP33/44unchanged.

CastTime Rust final update(exactUTCinreceipts): V1all1803passed/0failed/9ignored/core960/base314(base181.83sec),86662retired/reaped. AfterallRustreadersretired,preserved2C++pre-tooltipinputs+manifest, changedtooltipto基礎施法前搖＋actualadmissionattrs note/nativeassertions. NoRustchanges. Backend-v1nowRUNNING(nexttooloutputsession). Tasks1/2checked;3/4pending. FormalfreezeagainthroughUE/World/source. Singlehealth14433remains170sec. Sealerincludespre-tooltip3historyfilesand5finalsources; final-docs.luapreparedE1517pending. Nextbackend→UE→stage→World71/sourcecapture→visual/review→sourcecheck→stop/reap14433→CIM→seal1/verify/finaldocs. Nootherworkrunning.

CastTime UE update: backend-v1success72320retiredreaped;UEbuild-v1success18.64sec/57134retiredreaped;stage/importSHAverified. WorldV1/sourcecaptureV1RUNNING(nexttooloutputsessions);doNOTduplicate. Allformalinputsremainfrozen. Native71/33PNG expected, existing70namespreserved, newCastTimeModifiersreadoutcapture. Healthsingle14433still170sec. NextWorld/captureretire/reap→visualactualviews/review→sourcecheckowned→retire/reap→stop/reap14433→CIM→seal1/verify/finaldocsE1517.

CastTime visual rejection/UI V2: WorldV1all71pass(70success+1existingwarningcase/4warnings)77154retiredreaped;captureV1success65502retiredreaped. ActualnewPNGreadoutcroppedright. InspectionfounddensityPlaceAbilityReadouts.ClearChildren removescastrowsandonlyrestoreshotkey/CD; initialworldlessfixturedoesnotexerciseNativeTickrefits. Preserve6pre-readoutinputs+manifest, visual-v1.json, alloriginalWorldV1logs/reports/PNG. NoRustchanges. AddedpersistentAbilityCastReadoutRows/Texts inwidget.h, reparentretain/characterwrap/readablefonts/fitpolicy/layoutinvalidation;nativeTestnowrealownedGameViewport,800x600/1280x720,textscale1/2,actualrefits/retained2children/widthheightcontainmentbeforecapture. UEbuild-v2RUNNING(nexttooloutputexecid). Singlehealth14433active. Newhelpersstaged-v2/world-v2/capture-v2/source-v2;visualcurrentCastTimeWorldV2/priorHeroLevel. SealeracceptsRustversion1 butfinalbuild2/world2/capture2/source2/stage2 andarchives3successfulV1workers+visualrejection+6preUIinputs. Final6sources/formalcount3433/native71/images33. Afterbuild2→stage2→World2+capture2→actualview/review→source2→stop/reap14433→seal1/verify/finaldocsE1517. Updatefinal-docsproseforreadoutrootcausebeforecompletion.

CastTime capture V3 update: UE2success71.83sec/74195reaped;stage2verified;World2all71pass73418reaped/capture2success81807reaped. Actualfitlogsclause=passallmatrix, butchildPNGshowsmostlybar/blankratherthanfullglyphs;pixelproofrejected, nottestfailure. UE TakeScreenshotCommon usesGeneratePathToWidget arrangedgeometry; inspectpaintgeometryunderResolutionNormalizer. Preserved6pre-captureinputs+manifest andallV2reports/PNG/visual-v2.json (V1immutable;visualoverwritefailurefixedviaV2filename).

OnlyNativeTestCPPchangedforV3capture: fullactualviewportRootPNG + exactrectanglecopiedfromGetPaintSpaceGeometry/framecoordinates, norescale;capture.jsoncoords;assertglyphpixels>4. Runtime/RustunchangedsinceUI2/Rust1803. UEbuild-v3RUNNINGexec12580. Newstage3/world3/capture3/source3helpers;finalvisual-v3.json/currentCastTimeWorldV3/native71/34PNG(includesfullframe)/3433inputs;sealerfinal6workers and6historicalsuccessfulworkers(V1/V2build/world/capture)+allfailedpixelproof/frozen6+6sources. final-docsprosemustupdate34PNG/V3captureandvisualrejectionbeforefinalrun. Healthsingle14433still170sec. Nextbuild3→stage3→World3+capture3→actuallyviewfullframe+crop/allchangedimages→review→source3→stop/reap14433→CIM→seal1/verify/finaldocsE1517. Nootheractivework.

CastTime V3 final readers update: build3success6.84sec/12580reaped,stage3SHAverified;World3all71executed(70success+1existingwarningcase/4warnings)38217reaped;capture3success13899reaped. Newnativeglyphproofpassed. Actualfullviewport1280x720andpaintcrop[left595,top589,right632,bottom651]bothviewed:complete施法/<0.1/swrappedlabel+progressvisible, nocroppedglyphs. Prior32images:28exact+4actualviews(recall/map/firstkill/summon);plus2newactualviews=34reviewed. Source3nowRUNNING(nexttooloutputsession), sourcebefore3/verified3. Singlehealth14433active. FinaldocsupdatesnowmatchV3/fullframe+crop/34PNG, oldV1/V2visualrejections/sources preserved. Nextsource3retire/reap→stop/reap14433→CIM→seal1/verify/finaldocsE1517. Allformalinputsremainfrozen.


### Cast-time modifiers final (2026-10-23)

前述in-progress結束，complete-cast-time-modifiers4/4tasks/E1517，1803passed/0failed/9existingignored；World71全執行；3433正式輸入一致；260份證據／34PNG；health16 samples／max171秒。backend/UE5.8/stage/import/World71成功，全部34PNG審閱，所有worker/watchers/health14433退休reap。正式來源及封存證據獨立核對。

UnitStats::checked_cast_time共享checkedfamilyaggregation／i128duration／102rawmultfloor／1rawpositivefloor／60secmax，dispatch僅startsnapshot；midcastmodifierdrift不改時長，instant不受影響；2core＋2baseformal60Hz證明publictotal/remaining／pause／releaseeffect及ManaCDonce／invalidreject。UI同instance總時長漂移拒絕，positive<0.1s不顯示假0，persistent castrows跨densityrefit／文字量測換行，真ownedviewportfullframe與paintrect原像素PNG、coords／glyphproof。無newwire/CABI/contentID/heronamecase。這是preparationmodifier，不是sustainedchannel。

NEXT持續引導／其他已證實單機缺口；可查cooldown_mult是否同樣尚未接managedcast，但先確認既有handler／settlement避免重複套用。勿重做已接受nameplate/raw/zone/status/detector/level/casttime。11external/MVP33/44/完整combatreplica pending；oldfirstkillHUDcrop/mapfocusoverlapknown。原deadline2026-10-25 00:33:38Taipei不變，無真networkgames/quotareset/commit/push/omfx。


### Cooldown modifiers in progress (2026-10-23)

PriorcasttimeE1517/4tasksaccepted1803+9ignored/World71/source3433, allretired. Newcomplete-cooldown-modifiers: sharedchecked_cooldown(percentage+stacking checkedfamilyaggregation/102rawfloor/positive1raw/max1millionsec), prehandlersresolvedonce usedbymanagedledgerandactualoutcome. Releaseusesthen-currentstatsafterwindup; runningCDunchangedbylaterbuffdrift. Nohero/campbranch/newwire/CABI. 2core+3baseformal60Hznewtests. Formalmodified3sources(unit_stats/dispatch/single_lane_match_tests).

FULLRustV1RUNNINGexec25658;core962passed,basereleasecompilingwithCPUprogress. Sourcefreeze. Single170sechealth-v1exec2581 active, DONOTduplicate. cooldown- prefix/CooldownRustV1/CooldownWorldV1. Allhelperspreparedincludingcleansealer(noCastTimehistoryrequirements), finaldocsE1518pending. Expected1808+9ignored/World71/34PNG/source3433, onlyacceptactualresults. NewnativeUItestnotrequired(noUIchange), existing71regressionretainedandcast-timecaptureflagpreflightfixed. NextRust→backend→UE→stage→World/sourcecaptureparallel→actualchangedPNGview/review→sourcecheck→stop/reap2581→CIM→seal1/verify/finaldocs. Preservefailedexact3sources+originalreportsBEFOREanyfix. Originaldeadline/11external/MVP33/44unchanged.


### Cooldown modifiers final (2026-10-23)

前述in-progress已結束。實際native workroot為CooldownWorldV3；原world-v1 supervisor錯找V1路徑而false，原receipt/runner/logs保留。新owned world-confirm-v1已核對實際report71及原始成功輸出，未重跑native；sealer包含final6workers與historical1failed supervisor。後續以已封存index為準，勿依前述預期V1路徑啟動重複工作。

complete-cooldown-modifiers4/4tasks/E1518，1808passed/0failed/9existingignored；World71全執行；3433正式輸入一致；121份證據／34PNG；health8 samples／max171秒。backend/UE5.8/stage/import成功，34PNG審閱，final6workers/watchers及health2581退休reap，正式來源與證據獨立驗證。

checked_cooldown共享checked兩項percentage+stacking／familyaggregation／102rawmultfloor／positive1raw／1millionsecmax；dispatch prehandlers解析一次，managedledger與emittedCD同值。Windup開始合法性檢查、release當下屬性決定CD；running不隨buff重縮放，pausefreeze。2core＋3formal60Hz測試通過。無newwire/CABI/contentID/heronamecase。

NEXT持續引導或其他已證實單機操作／玩法缺口，不重做已接受nameplate/casttime/cooldown。11external/MVP33/44/完整combatreplica pending；oldfirstkillHUDcrop/mapfocusoverlapknown。原deadline2026-10-25 00:33:38Taipei不變。無真networkgames/quotareset/commit/push/omfx。


### Mana cost modifiers in progress (2026-10-23)

PriorcooldownE1518accepted1808+9ignored/World71/source3433/allretired. Newcomplete-mana-cost-modifiers: UnitStats.checked_mana_cost_mult checked_sum_add/checkedONEplus/validnegativeclampzero, authorityprepare_mana_cast andbotbudgetshared. 4formalmodifiedsources(unit_stats/dispatch/bots/single_lane_match_tests), 2core+3baseformaltests. FULLRustV1RUNNINGexec88323/core964passed/basereleasecompilingwithCPUprogress. Sourcefreeze; one170sechealth74829active DONOTduplicate. Prefixmana-cost-/CamelManaCost, nativeactualrootManaCostWorldV1 validatedagainstlauncher. Existingcast-timecaptureflag/testIDs unchanged. Helpers/sealer/verify/finaldocsE1519prepared. Expect1813+9ignored/World71/34PNG/source3433 onlyacceptactual.

Knownpreflightwhilecompile: newbotfixtureusesManaPool but top-levelruntime::*doesnotreexportit; anticipatecompileerror, DONOTeditsourcereadersactive. WaitRustretire/reap, preserveexact4sources+rawlogs, thenaddlocaluseandrunV2. Noformalchangeuntilretirement. Sourceinspectionalsoincorrectlyguessednative/mod.rs(actualnative.rs);rgfailureonly, discoverexactpathsbeforefuture reads. NextRustcohort→backend→UE→stage→World/sourcecaptureparallel→viewchangedimages/review→sourcecheck→stop/reap74829→CIM→seal(version)/verify/finaldocs. Originaldeadline/external11/MVP33/44unchanged.


ManaCost V2update: RustV1actualE0433missingManaPoolfixtureimport,88323retiredreaped; frozen-rust-v1 hasexact4inputs+manifest, allrawlogs/ready/false receiptpreserved. Addedtest-localexplicitManaPoolimportonly. FULLRustV2RUNNINGexec97910,core964passed/base3newformaltestsallpassed; existinglonglifecyclepending. Singlehealth74829active170sec DONOTduplicate. Finalsealeraccepts2, includeshistoricalV1cohort+freeze; source-v1prereqrust-v2. Sourcefreezecontinues. FinaldocsE1519preparedanddescribesfailure. NextRustV2retire→backend→UE→stage→World71actualManaCostWorldV1/sourcecaptureparallel→actualPNGview/review→sourcecheck→stop/reap74829→CIM→seal2/verify/finaldocs.


### Mana cost modifiers final (2026-10-23)

前述in-progress已結束，complete-mana-cost-modifiers4/4/E1519；1813passed/0failed/9existingignored；World71全執行；3433正式輸入一致；145份證據／34PNG；health8 samples／max172秒。backend/UE5.8/stage/import/World71成功，34PNG審閱；所有worker/watchers與health74829退休reap，證據來源獨立驗證。實際native workroot ManaCostWorldV1；以accepted index為準。

checked_mana_cost_mult共享checked_sum_add/manacost_percentage/family/checkedONEplus/validnegative→zero，authority與bot共用，再走checkedceilproduct/ManaPool。2core＋3formal60Hz驗證實際折扣／加價／free／invalid reject／release時取樣／bot read-only admission。Runtime無hero/campcase/newwire/CABI。

NEXT持續引導或其他已證實單機操作缺口；不要重做accepted nameplate/casttime/cooldown/mana-cost。11external/MVP33/44/完整combatreplica pending；firstkillHUDcrop/mapfocusoverlapknown。原deadline2026-10-25 00:33:38Taipei不變。無真networkgames/quotareset/commit/push/omfx。


### Effective cooldown HUD in progress (2026-10-23)

PriorManaCostE1519accepted1813+9ignored/World71/source3433/allretired. Newcomplete-effective-cooldown-hud: Hero optionalserde ability_cooldown_totals snapshotsactualstartduration, refresh/clear/expiry cleanup, nonpositivedtignored/saturatingsub. cooldown_total validatespresent bounds, legacy missing usesmax(base,remaining) conservativefallback(notclaimedoriginalduration), inactive staleignored/pruned. clientmoba_hud usesactualrunningtotal(existingproto/CABIunchanged), invalidpresent rejectsownerHeroHUD. 3core+1baseformal60Hz+2client newtests, nativeCastTimeModifiers existingtestextends10/40sechalf-fraction. 4modifiedformalsources(hero.rs/moba_hud.rs/single_lane_match_tests.rs/OmCastTimeModifierTests.cpp).

FULLRustV1RUNNINGexec13330/core967passed/basereleasecompileCPUprogress. Sourcefreeze. Single170sechealth71602activeDONOTduplicate. effective-cd-/EffectiveCdRustV1/EffectiveCdWorldV1 exactroot/flagpreflightpassed. Helpers/sealer/finaldocsE1520prepared; expect1819+9ignored/World71/34PNG/source3433onlyacceptactual. NextRust→backend→UE→stage→World/sourcecaptureparallel→actualPNGview/review→sourcecheck→stop/reap71602→CIM→seal1/verify/finaldocs. Preserve4exactinputs+rawlogsBEFOREanyfix.

NEXTconfirmedgap forfuturebatch: Generated OmWorldBridgeActor.cpp abilitymana precheck1918andPayload5834 stillbasecost; authorityalreadycheckedmodifier. Needpubliceffectivecost通用接線避免折扣被前端擋/加價假可用; donotrecomputeprivatebuffsC++. ExistingfirstkillfixturehasworldlessSlatewindowandHUDcropknown, notconfirmedproductionbug. Originaldeadline/external11/MVP33/44unchanged.


EffectiveCd V2update: V1core967/base321success, bridgeclientlibraryE0433(omoba_sim dev-only dependency), no clienttestsrun.13330retiredreaped; frozen-rust-v1exact4sources+manifestandallrawlogs/ready/falsereceiptpreserved. Fixedproductiontypename toexistingnormaldependencyomoba_template_ids::Fixed64(reexportverified), noCargochange. FULLRustV2RUNNINGexec60934; singlehealth71602stillactive170secDONOTduplicate. Sourcefreeze. source-v1prereqrust-v2/sealeraccepts2andarchivesfailedV1+4freeze. FinaldocsE1520preparedwithfailurecause. NextRustV2→backend→UE→stage→World71/sourcecaptureparallel→actualPNGview/review→sourcecheck→stop/reap71602→CIM→seal2/verify/finaldocs. Allotherdeadline/pendingconstraintsunchanged.


EffectiveCd finalreadersupdate: RustV2all1819passed/9ignored/core967/base321/client153,60934retiredreaped. Savedpre-native-proof.cpp+SHAreceipt AFTERallRustreadersretired; strengthenednativeexacttotal10/40tooltipassert andactual40total20remainingHUDcapture (Rustunchanged). Backend83933/UE99689successretiredreaped; stage/importSHAverified. World71allpass(70success+1existingwarningcase/4warnings/0failed/notrun),44299retiredreaped; sourcecapture63942successretiredreaped. 34PNG reviewed28exactprior+6actualviews includingactualW20cooldownandfullcastlabel. Sourcecheck-v1RUNNINGexec57883. Formalfreeze; singlehealth71602active170sec. Nextsourcecheckretire/reap→stop/reap71602→CIM→seal2/verify/finaldocsE1520. SealerincludesV1failedRust+4freeze andpre-native2files,final4sources. Finaldocsprosematchesnativeproof/failure/34PNG. Nootherworkrunning.


### Effective cooldown HUD final (2026-10-23)

前述in-progress已結束，complete-effective-cooldown-hud4/4/E1520；1819passed/0failed/9existingignored；World71全執行；3433正式輸入一致；150份證據／34PNG；health10 samples／max171秒。backend/UE5.8/stage/import/World71成功，34PNG審閱；全部worker/watchers與health71602退休reap，證據來源獨立驗證。actualrootEffectiveCdWorldV1。

Hero optional ability_cooldown_totals capturesstartduration/refresh/clear/expiry; nonpositivedtignored/saturatingsub. cooldown_total validatespresent bounds; missinglegacyfallbackmax(base,remaining), staleinactiveignored/pruned. clientmoba_hud usesactualtotal(existingproto/CABIshapeunchanged), invalidpresent rejectsownerHeroHUD. 3core＋1formal60Hz＋2client＋existingnativehalf-fraction10/40sec accepted. Nohero/campcase.

NEXT具體操作缺口：OmWorldBridgeActor.cpp SubmitOwnedAbilityCastCommand約1918仍用generatedbaseManaCost預檢，Payload.PublishedManaCost約5834亦base；authority已checkedcostmodifier，折扣會被前端誤擋、加價可能顯示可用。需要authority-disclosed effective cost通用接線（別在C++重算私有buff），readactualheaders/schema first。Otherchannels/11external/MVP33/44/完整combatreplica pending；firstkillHUDcrop/mapfocusoverlapknown。原deadline2026-10-25 00:33:38Taipei不變。無真networkgames/quotareset/commit/push/omfx。


### Effective mana HUD in progress (2026-10-23)

PriorEffectiveCdE1520accepted1819+9ignored/World71/source3433/allretired. Newcomplete-effective-mana-hud: authorityemit_owner_mana_costs calledoncewithownerScore(PreStepinactive/PostStepactive), team-scoped currenttick+4bindings/4rawcosts fromcheckedUnitStats+AbilityRegistry. raw-1unknown/-2invalid/0free; bindingtemplateID.raw()<<8|rank; clientvalidatescurrenttick/team/unique/slot/catalog/rank andactualownerpool→state0unknown1affordable2short3invalid. AdditiveprotoAbilityHud fields; ABI35 OmAbilityHud48bytes statusoffset40/price44, Rust/header/smoke/projection/driverassertionssync. Generatedactor usesstatusprecheckandcurrentpricepayload, removesbasecostgate, resetscoststatus onretire. Nativeformattershowscurrentcost/invalid/unknown. NoC++privatebuffmath/newnetworkgame.

Formal15sources listedsealer (includinggeneratedproto). V1coreE0605AbilityIdnewtypenotprimitive;20539retiredreaped; frozen-rust-v1exact15inputs+manifest/rawlogs/ready/falsereceiptpreserved. FixedID.raw() APIs verifiedbuild.rs all3Rustsites+fixtures. FULLRustV2RUNNINGexec6669/core967passed/basereleasecompilingCPUprogress. Single170sechealth12645activeDONOTduplicate. Prefixeffective-mana-/EffectiveManaRustV2/EffectiveManaWorldV1, source-v1prereqrust-v2/sealeraccepts2withfailedV1history. Expected1823+9ignored (3newclient+1base, existingbridge/nativeextend),World71/34PNG/source3433onlyacceptactual.

PRE-FINALCONCERN: bridge currentlymin(3)status andnegativeprice→0 retainsstate1; malformedtypedcostcouldbecome free. AfterALLV2readersretired/reaped, preserve15scope+logs, addsharedtypedcostsanitizer validatingclassificationagainstactualmana andnonnegativeprice; invalid→state3price0, unknownstate0requiresprice0. Fixexistingbridgefixture tosetmana_supported=true/mana30/max100 forvalid22.5price. Addclientinvalidtypedtest; needRustV3 (expected1824+9ignored). DoNOTeditwhileV2active. Finaldocsnotprepared/E1521pending. Nextfinishcostboundary→fullRust→backend→UE→stage→World/sourcecaptureparallel→actualPNGviews/review→sourcecheck→stop/reap12645→CIM→seal(version)/verify/finaldocs. Sourcefreezeuntilretired. Originaldeadline/external11/MVP33/44unchanged.


EffectiveMana V3RustFINAL: V2all1823+9ignoredsuccess/6669retiredreaped; saved18-sourcefrozen-rust-v2+manifest/wholecohort beforetyped-boundary/nativefloatfix. Addedchecked_ability_mana_cost: invalidprice/classification/rank/mana support→state3price0; bridgeusesitandexistingfixturetrueMana30/level1/price22.5. Nativeowner cachesstatus/classificationwithall3retireclears; exactshortsurvivesfloatequality; invalidcostfailurelocalized. Final18sourcessealer; V1failed15freeze/V2successful18freeze archived.

FULLRustV3all1824passed/0failed/9ignored(core967/base322/client157),40989retiredreaped. Backend-v1RUNNING(nexttooloutputsession). Sourcefreeze. Singlehealth12645stillactive170secDONOTduplicate. source-v1prereqrust-v3/sealeraccepts3andhistorical2workers. FinaldocsE1521preparedwithallscope/failure/history;World71/34PNG/source3433expectedonlyactual. Nextbackend→UE→stage→World/sourcecaptureparallel→actualPNGview/review→sourcecheck→stop/reap12645→CIM→seal3/verify/finaldocs. Noformalchangesplanned. Originaldeadline/external11/MVP33/44unchanged.


EffectiveMana nativeV2update: Backend25145/UE1 93660/stage/importsuccessretiredreaped. NativeV1actual71complete/69success+1warningcase+1Fail AbilityTargetTooltip旧broadassertforbadeall魔力消耗includingnewunknownline.33430/sourcecapture5983retiredreaped; preserved18pre-native-v2sources+manifest, rawreport/logs/all34PNG includedsealer. ChangedONLYexistingnativeasserttoexclude numericprefix魔力消耗：andrequireunknownmessage(slot0actualfixture). NoRust/productionlogicchangesafterRustV3accepted1824.

UEbuild2success22963retiredreaped;stage2SHAverified. WorldV2RUNNINGexec88683 actualrootEffectiveManaWorldV2;sourcecaptureV2RUNNING13938. Single170sechealth12645activeDONOTduplicate. Formalfreeze. Finalhelpersworld2/capture2/source2/stage2/visual2; source2prereqRust3/backend1/build2/world2/capture2;sealeraccepts3, final6workers/history5(RustV1false/V2true/build1true/world1false/capture1true), all3frozen scopes15+18+18/final18sources. finaldocsE1521preparedwithallhistory/nativefailure,actualrootV2fixed. NextWorld/captureretire/reap→visualactualviews/review→source2→stop/reap12645→CIM→seal3/verify/finaldocs. Originaldeadline/external11/MVP33/44unchanged.


### Effective mana HUD final (2026-10-23)

前述in-progress已結束，complete-effective-mana-hud4/4/E1521；1824passed/0failed/9existingignored；World71全執行；3433正式輸入一致；300份證據／34PNG；health15 samples／max171秒。backend/UE5.8/stage/import/World71成功，34PNG審閱；所有worker/watchers及health12645退休reap，正式來源／證據獨立驗證。actualrootEffectiveManaWorldV2；以accepted index為準。

authority emits team-scoped tick+4templateID.raw/rankbindings+4checkedrawprices withownerScoreoncepertick;0free/-1unknown/-2invalid. clientunique/team/tick/binding validation + exactownermana→state0unknown1affordable2short3invalid; sharedtypedchecker preventsnegative/contradictoryprice→free. ABI35 OmAbilityHud48bytes/status40/price44 header/smokesync; Generatedactor usescurrentstate/price, nobasegate; nativepreviewkeepsexactauthorityshortclassification andclearsonretire. 1newbase+4newclient, existingbridge/nativeextensions. Final18sources; V1failure15freeze/V2successful18freeze preserved. Noherocampcase/privatebuffpayload/C++statmath.

NEXT其他具體單機玩法／技能操作缺口（持續引導等），勿重做acceptedcost/cooldown/nameplate。firstkillfixture worldless HUDcrop與mapfocusoverlapknown。11external/MVP33/44/完整combatreplica pending；原deadline2026-10-25 00:33:38Taipei不變。無真networkgames/quotareset/commit/push/omfx。

### Invalid cost aim feedback in progress (2026-10-23)

New complete-invalid-cost-aim-feedback: confirmed typed cost state3 was omitted from aim Evaluate despite tooltip/submit already handling it. Appended InvalidManaCost local enum, optional typed state argument, Chinese label/orange warning; OnPaint only supplies matching pending catalog state. Existing PointAreaPreview extends explicit3 vs0/1/2/malformed, precedence, color/text and unlearned/missing-shape/owner reset. Three native sources only; no Rust/proto/ABI/backend change.

UE5.8 build-v1 successful and93286retired/reaped; staged/import SHA verified. World71 RUNNING71461 actualrootInvalidAimWorldV1; sourcecapture10881 and inheritance96789 running (3430 unchanged vs previous accepted3433, exact3native changes expected). SINGLE170sec health50823 active, no duplicate. Formal sourcefreeze. Helpers invalid-aim-*.lua; errorsE1522 MD records guessed module read failure only. Next reap readers→visual compare/view→sourcecheck→done-v1/reaphealth→CIM→seal/verify/finaldocs. No fresh Rust test claim; prior1824+9ignored inherited only after source proof. Originaldeadline/external11/MVP33/44 unchanged.

### Invalid cost aim feedback final (2026-10-23)

前述in-progress已結束，complete-invalid-cost-aim-feedback4/4/E1522。Native-only exact3sources；sharedInvalidManaCost explicitstatus3/matchingcatalog/Chinese/orange；既有PointAreaPreview增加status/precedence/reset斷言。UE5.8/stage/import成功，World71(70success+1existingwarningcase/4warnings/0failed/notrun)；3433正式輸入一致；81份證據／34PNG(28exactprior+6actualviews)；health max170sec。1824Rustpassed+9existingignored是前批繼承，3430unchanged/exact3native delta來源證明，不是新重跑。No dedicated new aim screenshot claim。

Build93286/World71461/sourcecapture10881/inherit96789/sourcecheck3925/health50823/sealer33604全部完成並reaped；四worker/watchersretired，presealCIMclean。docs/acceptance/2026-10-23-invalid-cost-aim/index.json封存後81份bytes/SHA/originalsource獨立校驗成功；final/errorsMD及完整度盤點已更新。

NEXT具體新玩法（優先真正持續引導／通用channel，別把castpreparation或groundzone稱為channel），或單機操作缺口。尚未新增channelchange；既有英雄資料並未找到現成channel宣告，需先設計共用metadata+host生命週期再選合適作者內容，不能私改既有非channel技能語義。勿重做cost/CD/nameplate接受。firstkillworldlessfixtureHUDcrop/mapfocusoverlapknown；11external/MVP33/44/完整combatreplica pending。原deadline2026-10-25 00:33:38Taipei不變；無真networkgames/quotareset/commit/push/omfx。

### Control cleanse in progress (2026-10-23)

New add-summoner-control-cleanse: Lua catalog4 training_control_cleanse_v1「淨念」210s/self; only canonical stun/root/silence removal via indexed BuffStore.cleanse_controls, no immunity/slow/DoT/all-buff clearing, no action interruption or defaultloadoutchange. Strict authored emptystruct SelfControlCleanse{}, generated/runtimeunitoperations and resolvedselfeffect, traitdefaultErr forunsupported sink, EcsSinkremovescontrols thenmodelcommitsCD. Newcontent1/core3/baseformal60Hz1/native1; codegenexistingtestextendsarbitraryID; generatedtemplatecount4. Runtimewire/ABIunchanged. Final11authoredsources include testscannerfix; actualgeneratedCPPs followUEbuild.

V1 actualserdeunit acceptedextraimmunityfield: content42pass1fail;13671retired/reaped, exact10inputs/manifest frozen-rust-v1, rawlogs/false receipt preserved; fixedauthoredvarianttoemptystruct. V2 ABI15/content43success; templateintegration22pass1fail historical docs/acceptance frozen game.rs oldpathscan;63656retired/reaped, exact11frozen-rust-v2+manifest. Fixedsharedscannerexcludeexactworkspace docs/acceptance (retainarchivesandotherdocs/source scans). FULLRustV3RUNNING90216: ABI/content/templates/codegen/coreallpassed (core970), basereleasecompiling. SOURCEFREEZE. SINGLE170sec health8050activeDONOTduplicate. Helperscontrol-cleanse-*, rootsControlCleanseRustV3/WorldV1; World72/newcleanse-tooltipPNG expected35 onlyactualaccepted. source-v1prereqRust3; sealerexpectsV1+V2history/frozen10+11/finalrust-inputs-v3manifest11, no premature completion. ErrorsE1523MD updated. NextRustV3retire/reap→backend→UE→stage→World/captureparallel→actualPNGreview→sourcecheck→done/reaphealth→CIM→seal/verify/finaldocs. Originaldeadline/external11/MVP33/44unchanged.

ControlCleanse V4update: V3core970passed, basecompileE0616 privateMobaHeroSlot.summoners innewcrosscratefixture;90216retired/reaped, exact11frozen-rust-v3/manifest archived. Replacedprivatefieldreadwithactualpublic project_team_summoners/decode andcatalog4/player2/cooldown assertions. No runtimechange. FULLRustV4RUNNING82860, immutable11rust-inputs-v4manifest. SINGLEhealth8050stillactive170secDONOTduplicate. source-v1prereqRust4/sealerfinalRust4+3historicalfalsecohorts/frozen10+11+11, finalWorldV1/35images expectedonlyactual. Formalfreeze. NextRust→backend→UE→stage→World/capture→view→sourcecheck→stophealth→seal. ErrorsMD E1523 containsall3failures/prevention.

ControlCleanse RustV4FINAL: all8scopes1829passed/0failed/9existingignored (ABI15/content43/templates116/codegen69/core970/base323/bridge136/client157).82860retired/reaped. Formal60Hzcleanseinput/controlretirement/catalog4publicCD/pause/noimmunity passed. Backend-v1RUNNING(nexttooloutputsession). SOURCEFREEZE. health8050active170secDONOTduplicate. sealerfinalRust4 with3failedhistories, finalinputs11 andgeneratedCPPdeltasallowedonlyOmGenerated. Native72/35PNG/source3434expectedonlyactual. Nextbackend→UE→stage→World/captureparallel→view→sourcecheck→done/reaphealth→CIM→seal/verify/finaldocs.

ControlCleanse NativeV2update: backend5610passedretired/reaped; UEbuildV1 C2248newnative testmissingBuildSummonerTooltipfriend;15956retired/reaped. Saved12pre-native-v2sources/manifest AFTERallreadersretired; addedonlyfriendFOmSummonerCleanseTestunderWITH_DEV_AUTOMATION_TESTS. NoRust/gameplaychange;11RustV4inputsunchanged. UEbuildV2RUNNING1032. stage-v2/build-v2/source-v1prereqbuild2; WorldV1/visualV1 unchanged. Sealerfinal6workersRust4/backend1/build2/world1/capture1/source1 +4historicalfalse(V1/V2/V3Rust+build1), frozen10+11+11+12. SINGLEhealth8050active170secDONOTduplicate. E1523errors/finaldraftupdated. Nextbuild2→stage2→World/captureparallel→view→sourcecheck→done/reaphealth→CIM→seal/verify/finaldocs. Formalfreeze.

ControlCleanse finalreadersupdate: UEbuild2 1032successretired/reaped;stage2DLL/importSHAverified. World72(71success+1existingwarningcase/4warnings/0failed/notrun)13291retired/reaped;sourcecapture86044successretired/reaped.35PNGreviewed29exactprior+6actualviewsincludingnewcleanse-tooltipfullChinese. Baseline3434inputs,15expectedchanges=12manual+3OmGenerated(catalogcpp/assetrecipe/codegenmanifest), sealerallowedscopevalid. Sourcecheck-v1RUNNING24150;SOURCEFREEZE. SINGLEhealth8050active170secDONOTduplicate. FinalsealerRust4/backend1/build2/world1/capture1/source1/history4false/freeze10+11+11+12;finalRust4inputs11unchanged,headerfriendonlynativefollowup. Nextsourcecheckretire/reap→done-v1→reaphealth8050→CIM→seal/verify/finaldocs. ErrorsE1523/finaldraftupdated;originaldeadline/external11/MVP33/44unchanged.

### Control cleanse final (2026-10-23)

前述in-progress已結束，add-summoner-control-cleanse5/5/E1523。新增Lua第四個可配置召喚師「淨念」catalog4/210s/self，strict SelfControlCleanse{}作者schema、generated/runtime unitop、typedselfeffect、unsupported sinkErr，EcsSink透過indexed BuffStore.cleanse_controls移除canonical stun/root/silence再commitCD。其他Buff/units/HP/MoveTarget不變；無免疫、不清slow/DoT、無control亦消耗CD；未改defaultloadout或新增match內選擇器。無hero/camp特判/newwire/CABI/FFI方法。

RustV4全8範圍1829passed/0failed/9既有ignored：ABI15/content43/templates116/codegen69/core970/base323/bridge136/client157。新增content1/core3/baseformal60Hz1/native1；正式input→publiccatalog4/CD/pause/controlclear/noimmunity接受。backend/UE5.8 buildV2/stage/import成功；World72(71success+1existingwarningcase/4warnings/0failed/notrun)。3434正式輸入一致，15delta(12manual+3generated OmSummonerCatalog.cpp/assetrecipe/codegenmanifest)。250份證據／35PNG(29exactprior+6actualviews inclnewcleanse-tooltip全文可讀)。health max171sec。docs/acceptance/2026-10-23-control-cleanse/index.json封存後250files bytes/SHA/originalsource逐一再驗成功。

所有workers/watchers及health8050均退休reaped；preseal與postsealCIMclean。成功Rust82860/backend5610/build2 1032/World13291/capture86044/source24150/sealer4478；歷史Rust13671/63656/90216與build1 15956失敗原始logs/false receipts全部保留，frozen source scopes10+11+11+12。V1serdeunit漏拒unknownfield改emptystruct；V2掃描誤入歷史archive改exact docs/acceptance排除；V3跨crate private CD改publicteamprojection；UE1缺private tooltip testfriend補測試專用friend，不放寬publicAPI。最終11RustV4inputs未變，friendheader為native-onlyfollowup。

NEXT繼續具體新玩法/操作缺口，優先真正持續引導或特殊技能幾何等；不要重做已accepted cleanse/cost/CD/nameplates。持續引導仍未開始，既有作者技能皆instant；需通用metadata/host生命週期與合適新內容，不可把前搖/zone當channel或任改既有技能語義。firstkillworldlessfixtureHUDcrop/mapfocusoverlapknown；11external/MVP33/44/完整combat replica pending。原deadline2026-10-25 00:33:38Taipei不變。無真networkgames/quotareset/commit/push/omfx。

### Recovery channel in progress (2026-10-23)

New add-channeled-recovery-item: actualhost stationary healing channel, Lua shopitem12「靜心符」cost500/CD30/40HP per1s for3s. StrictChannelHeal author/generated/runtimefixedSpec; commonitemtransaction preflightsowner/life/phase/control/pause/shape thenclearcommands/helddirection/attackwindup andstart, CDonce. HostChannels BTreeMap<Entity> owner/origin/spec/remaining/clock/lasttick; advance in finish_moba_match_tick AFTERalloutcomes BEFOREownerHUD. ExistingHPdamage/acceptedcommand hooks cancel; position/silence/stun/owner/life/phase/recall invalidation, rootallows/shieldonlyallows/pausefreezes; heroTickautoattacksuppressed. Replaydiagnosticincludesprivateclock/spec. Publiccanonical recovery_channel bufftimeonly +strictpayloadtotal validation, noorigin/source; nativeenumappendChannelHeal+intervalmetadata/commonselfkindhelper, shop/hotbar/useinput/Chineseguidance/statusdetails. NoCABI/proto/FFIchange, noexistingheroabilitysemanticschange. Dedicatedchannelbar/animationVFXnotclaimed.

Final18sources bound in recovery-channel-rust-inputs-v1 manifest. FullRustV1RUNNING10516; SOURCEFREEZE. SINGLE170sec health48827activeDONOTduplicate. Helpersrecovery-channel-* preparedfresh(nooldhistory), rootsRecoveryChannelRustV1/WorldV1, expected1839passed+9ignored/World73/36PNG/source3436onlyactualaccepted. Newauthor1/core5/baseformal3/codegen1/native1. Nativeusespublicwidgethelpers(nofriendneeded)andOmRegistry.hactualbufflookupdecl. ErrorsE1524recordsonewrongmodpath/atomicpatchcontextfailure, noactualtestfailureyet. NextRustretire/reap→backend→UE→stage→World/captureparallel→view→sourcecheck→done/reaphealth→CIM→seal/verify/finaldocs. Originaldeadline/external11/MVP33/44unchanged.

RecoveryChannel V2update: V1 ABI15/content44/templates116/codegen70/core975success; basecompileE0004 oldfive-item testmatchmissingChannelHeal.10516retired/reaped, exact18inputs frozen-rust-v1 beforeaddingexplicitChannelHeal dedicatedfixturearm(notwildcard). FULLRustV2RUNNING78465; allnewcore5 andformal3 testsPASSED, baseremainderlong3lane lifecyclepending. Final18sourcesboundrust-inputs-v2; source-v1prereqRust2; sealerfinalRust2+failedV1worker/fullcohort/frozen18 history. SOURCEFREEZE. SINGLE170sechealth48827activeDONOTduplicate. E1524errors/finaldraftupdated. World73/36PNG/source3436expectedonlyactual. NextRust→backend→UE→stage→World/captureparallel→actualviews→sourcecheck→done/reaphealth→CIM→seal/verify/finaldocs. Originaldeadline/external11/MVP33/44unchanged.

RecoveryChannel RustFINAL: V2all8scopes1839passed/0failed/9existingignored (ABI15/content44/templates116/codegen70/core975/base326/bridge136/client157).78465retired/reaped; basefullsuite190.51sec andall3newformalchannelcases passed. Backend-v1RUNNING(nexttooloutputsession). SOURCEFREEZE. SINGLEhealth48827active170secDONOTduplicate. Final18Rustinputsv2 unchanged; sealerexpectsRust2/onefailedhistory/frozen18, World73/36PNG/source3436onlyactual. Nextbackend→UE→stage→World/captureparallel→views→sourcecheck→stop/reaphealth→CIM→seal/verify/finaldocs. Error/finaldraftE1524prepared.

## Recovery channel FINAL — supersedes all earlier in-progress notes

OpenSpec add-channeled-recovery-item6/6 completed. 靜心符 catalog12 cost500/CD30,3-second stationary healing channel40HP per1-second delayed pulse; shared checked ChannelHeal and host state, live received-healing/cap, commands/HPdamage/CC/owner/life/phase/position cancellation, root/shield-only preservation, pause/doubletick protection, autoattack suppression and replay diagnostic. Native generated interval/self-item contract plus actual Chinese description and public2.5s status capture accepted.

Final acceptance: Rust1839 passed/0failed/9existingignored (ABI15/content44/templates116/codegen70/core975/base326/bridge136/client157); backend and UE5.8 buildV2 succeeded; staged DLL/import-library SHA matched. Native WorldV2 executed73:72success+1existingwarningcase,0failed/0notrun. Source consistency verified3436 inputs. All36 PNGs accepted:7 changed/new actual images viewed,29 prior SHA-identical images inherited. Archive contains292 independently reverified evidence files and36 images; 20 health samples/max171 seconds. Six final workers and four historical workers have creation-token-bound retired receipts; launch sessions and the single170-second health worker were reaped. No owned test/build worker remains.

HistoricalRustV1 E0004 and NativeV1 one-decimal assertion failure preserved with exact originals; native follow-up changed only new C++ test, with prior/current SHA proof and17 unchanged Rust inputs. Evidence index:docs/acceptance/2026-10-23-recovery-channel/index.json; final/error MD:docs/plans/2026-10-23-recovery-channel-{final-acceptance,errors}.md. All work stopped/reaped cleanly for batch boundary; no reader active and no source freeze remains. Next concrete gap: dedicated common channel progress presentation/animation/VFX or another audited single-PC gameplay gap; do not rerun accepted tests without new changes. Full combat replica/11 external/MVP33/44 and known fixture crop/overlap/narrow label wrapping remain pending. No real network games/quota resets/commit/push/omfx changes. Keep original deadline2026-10-25 00:33:38 Asia/Taipei; automationomoba-48 continues.

## Channel progress FINAL — supersedes prior recovery-channel next-step note

OpenSpec complete-channel-progress4/4 complete. Common owned read-only channel progress strip now connected beside recall feedback. Native bounded decoder uses existing recovery_channel.total_raw and remaining only; generic model validates local living playing hero/generation, unique channel metadata and ordered snapshots. Midway first snapshot, old/duplicate/cancel/death/replacement/scope/retirement and actual Slate rendering passed. No local timer/new C ABI/wire/Rust/source gameplay change.

UE5.8 buildV3/staged DLL/import SHA passed. WorldV3 executed74:73success+1existingwarningcase,0failed/0notrun. Source guard verified3439 inputs; prior accepted Rust1839passed/0failed/9existingignored and backend result are inherited after SHA proof of3432 unchanged prior inputs +4 exact native changes +3 native additions; no new Rust suite is claimed. 520 evidence files and37 PNGs independently reverified;7 changed/new images actually viewed and30 prior SHA-identical images inherited. Health9 samples/max171 seconds. Four final owned workers and seven historical workers have retired creation-token receipts; every launcher, all three inheritance readers and the single health monitor were reaped; CIM found no owned worker remaining.

NativeV1 crashed in the new duplicate-row fixture because TArray Add cannot take its own element by reference. Original incomplete run logs/maps/available PNGs and7 exact pre-repair inputs are archived. Exact test-only follow-up proof verifies detached copy before Add; after that fixture-only repair all6 production inputs were unchanged. BuildV2/full WorldV2 succeeded after fixture repair. Subsequent real bridge review found opaque FNV1a64 keys were incorrectly limited to positive int64; V3 accepts every nonzero bit pattern and tests a high-bit actual ABI key. Both V1 and successful V2 originals and all three seven-input source cohorts are retained. No failed evidence was overwritten. E1525 contains cause/prevention and read-only path/helper pattern mistakes. Source freeze is released after all readers retired.

Evidence:docs/acceptance/2026-10-23-channel-progress/index.json; final/errors:docs/plans/2026-10-23-channel-progress-{final-acceptance,errors}.md. All readers stopped; no source freeze remains. Next concrete gap: versioned bounded channel-total disclosure in public-score BVS1 replacement/extension (legacy identity/remaining-only snapshots cannot show optional progress). Also narrow cast-time slot wrapping (under0.1s wraps across lines) and channel animation/VFX remain. Do not infer total from remaining or item names. Avoid merely rerunning accepted checks. Full combat replica,11 external/MVP33/44, existing worldless first-kill HUD crop and map focus/nameplate overlaps remain pending. No real network matches/quota resets/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Taipei and automationomoba-48 unchanged.

## Public channel duration IN PROGRESS

OpenSpec complete-public-channel-duration3/4 implemented. BuffVisualSnapshot versioned BVS2 wrapper retains exact BVS1 codec, bounded canonical total/remaining only; visibility producer, filtered baseline/update validation and bridge consume both. Native newPublicChannelDuration test uses actual Rust ABI export via driver::ipc_tests::public_channel_duration_filtered_bridge_exports_actual_abi_and_clears. Six final inputs frozen in public-channel-inputs-v1/manifest.json; SOURCE FREEZE while readers active. Core979 passed incl4new tests, full8-scope Rust running44902; release base compiled2m40s and formal suite advancing. SINGLE170-second health45573 active, do not duplicate. Expected only after actual results:Rust1844/9ignored,World75,source3440/images37. Fresh public-channel-* helpers syntax checked, world requires actual export file (zero-match cannot pass). Next Rust retire/reap→backend→UE/stage→World+source capture→views→source check→stop/reaphealth→CIM→seal/reverify→final docs/tasks/audit. E1526 logs read-only path/wildcard mistakes and corrected unlaunched module-name helper; no actual test failure so far. Deadline/external11/MVP33/44 unchanged.

PublicChannel native acceptance update: RustV1 failed only new bridge fixture using add as countdown;44902retired/reaped, original6 inputs-v1/logs frozen. Exact test-only tick repair→FULLRustV2 actual1844passed/0failed/9existingignored,8580retired/reaped. Backend1 passed/reaped28418. UEbuild1 failed C1083 Misc/LexFromString.h (actualString/LexFromString.h),37024retired/reaped; originalRust6 inputs-v2+failedbuildlog preserved. Native include-only followup→final6inputs-v3 (fiveRustinputsunchanged), build2+stage/importpassed,reaped69834. World1 RUNNING39677 with actualpublic ABI JSON export and75 tests; source-capture1 RUNNING42042 expecting3440. SINGLEhealth45573 active170sec DONOTduplicate. Source-v1 waitsRust2/Backend1/Build2/World1/capture1. Sealer expects final6workers+2failedhistory, exactfinal6 plusoriginalcohorts v1/v2 and include/test-only proofs, fullRust1844/9/native75/images37. SOURCEFREEZE untilallreadersretired. NextWorld/capture→viewactualchanges→sourcecheck→done/reaphealth/CIM→seal/reverify→close-docs/tasks/audit. E1526/draftupdated; deadline/external11/MVP33/44 unchanged.

## Public channel duration FINAL — supersedes all in-progress notes and prior BVS1 gap

OpenSpec complete-public-channel-duration4/4 complete. BuffVisualSnapshot BVS2 on existing owner-team schema/event carries only strict canonical total/remaining; BVS1 codec retained. Authoritative visibility produces wrapper; filtered baseline/update validation and Unreal bridge decode both; reconstructed sanitized JSON reaches existing native typed progress strip. Team-only visibility/hidden/no arbitrary payload/clear/legacy unknown contracts covered. No C ABI/proto/new schema/event ID or gameplay rule change.

Final RustV2 all8 maintained scopes passed1844/0failed/9existingignored:ABI15/content44/templates116/codegen70/core979/base326/bridge137/client157. Backend and UE5.8 buildV3 passed; staged DLL/import SHA matched. Native WorldV2 executed75 with0failed/0notrun, including actual Rust-filtered/ABI-export-to-native progress integration. Source guard verified3440 inputs:3434 unchanged prior inputs +5 Rust changes +1 native test addition. 497 evidence files and37 PNGs independently reverified; health12 samples/max171 seconds. All6 final workers and6 historical workers (4 failed/2 successful) have creation-token-bound retired receipts; launchers and the single170-second health monitor were reaped and CIM showed no remaining owned work.

RustV1 failed only the new bridge countdown fixture: re-adding a smaller duration preserves the longer Buff duration. Original raw7-scope cohort/ready/false receipt and6 exact original sources are archived; client had not run in that failed cohort. A bound test-only follow-up uses the real BuffStore.tick(0.5s), with the other5 inputs unchanged. Full8-scope V2 and actual export fixture passed after repair. E1526 records cause/prevention and earlier read-only path/wildcard/helper-module errors; no failure evidence was overwritten. UEbuildV1 then failed C1083 in the new native test because LexFromString.h was referenced under Misc instead of the actual UE5.8 String directory. Original build log/false receipt and all6 RustV2 inputs are archived. A bound include-only follow-up changed that native test header and no Rust input; buildV2 succeeded under inputs-v3; that native run then exposed the float assertion below. Full RustV2 is retained under exact five-Rust-source SHA proof; no Rust rerun after C++ test-only corrections is claimed. NativeWorldV1 later failed one two-thirds elapsed assertion because UE default float tolerance1e-8 is below the one-ULP difference between1-1/3 and2/3. Original full raw75-test run/ABI JSON/images and6 inputs-v3 are preserved, along with its failed upstream-gated source worker. A bound assertion-only follow-up logs actual/expected and uses1e-6, below one Q10/60second step; buildV3/fullWorldV2 use final inputs-v4. All5 Rust inputs remain SHA-identical to the full passed RustV2 cohort.

Evidence:docs/acceptance/2026-10-23-public-channel/index.json; final/error MD:docs/plans/2026-10-23-public-channel-{final-acceptance,errors}.md. All readers and monitors retired; no source freeze remains. Maintained filtered public channel-total gap is closed. Next concrete single-PC presentation gap: narrow ability cast-time label wraps across lines for under0.1s; inspect common hotbar layout and improve compact display with actual capture, not hero/skill-name cases. Channel animation/VFX/more effects, full combat replica and11 external/MVP33/44 remain; known worldless first-kill crop/map focus-label overlaps retained. No new real match/quota reset/commit/push/omfx maintenance. Original deadline2026-10-25 00:33:38 Taipei and automationomoba-48 unchanged.

## Compact cast readout FINAL — supersedes prior narrow wrapping gap

OpenSpec complete-compact-cast-readout4/4 complete. Compact active time and distinct cancelled/completed labels stay single-line at selected font size; full Chinese cast status remains in ability tooltip. Both retained and initial card layouts measure unwrapped content for existing responsive reflow. Real viewport30 configurations cover2sizes/3textscales/5states with glyph-height, arranged bounds, lifecycle and cooldown checks. No Rust/wire/C ABI/gameplay change.

UE5.8 build-v1/staged DLL/import SHA passed. WorldV2 executed75:74success+1existingwarningcase,0failed/0notrun. Source guard verified3440 inputs; accepted Rust1844passed/0failed/9existingignored and backend results inherited after exact SHA proof of3437 unchanged prior inputs plus3 native deltas; no new full Rust suite is claimed. Archive contains360 independently reverified evidence files and37 PNGs. Health5 samples/max171 seconds. Four successful final workers and two failed historical workers have creation-token-bound retired receipts; all launchers, inheritance reader and the single170-second health monitor were reaped. CIM found no owned worker remaining.

6 changed/new images actually viewed; 31 SHA-identical accepted images inherited. Actual cast crop is40x24, previously38x65; <0.1s remains one line. Actual full1280x720 viewport shows cooldown20, cast progress and hotkey intact. No font shrinking or local cast countdown.

Original incompleteWorldV1 + upstream-gated source failure retained, exact runner-only correction restores existing PublicChannelDuration test and fixture flag. Three frozen native inputs unchanged betweenWorldV1/V2. E1527 records cause/prevention. Evidence:docs/acceptance/2026-10-23-compact-cast/index.json; final/errors:docs/plans/2026-10-23-compact-cast-{final-acceptance,errors}.md. No active reader/source freeze. Next concrete gap: common channel animation/VFX or another audited single-PC gameplay feature; do not repeat accepted nameplate/cast wrap/public duration checks. Full combat replica/11external/MVP33/44/worldless firstkillcrop/mapfocusoverlap remain pending. Originaldeadline2026-10-25 00:33:38Taipei and automationomoba-48 unchanged. No true network games/quota resets/commit/push/omfx.

## Channel world VFX FINAL — supersedes channel VFX pending

OpenSpec complete-channel-world-vfx4/4 complete. Canonical disclosed channel metadata now reaches a typed living-entity frame state and the actor epoch/tick/sequence gate. One indexed effect per actor reuses the original512-slot instanced pool:O(1) lookup/update, bounded capacity drop and fresh snapshot recovery. Authoritative elapsed fraction controls geometry; local time changes only pulse opacity and pause freezes pulse age. Empty channel state skips the full entity scan. Cancel, hide, death, epoch/generation/reset/opt-out and recall priority retire the effect. Reveal alone does not replay old state. Typed buff callbacks do not add a second generic ring; legacy unknown-total and custom handlers retain their existing paths. No hero/camp/item-name special cases, gameplay, private disclosure, C ABI, wire or Rust change.

UE5.8 buildV3/staged DLL/import SHA passed. WorldV2 executed76:75success+1existingwarningcase,0failed/0notrun. Source guard verified3441 inputs; full accepted Rust1844passed/0failed/9existingignored and backend result inherited after SHA proof3431 unchanged prior inputs+9 native changes+1 addition; no full Rust rerun is claimed. Archive contains437 independently reverified evidence files; 41 final accepted PNGs, with historical captures also retained. Health8 samples/max171 seconds. Four final and six historical successful workers have creation-token-bound retired receipts; all launchers, three inheritance readers and the single170-second health monitor were reaped. CIM found no owned build/test/source worker remaining.

11 new/changed images actually viewed; 30 accepted prior SHA-identical images inherited. Four new512x512 images show the actual idle native hero with a turquoise channel ring, isolated midway and larger late ring, and a completely cleared shader image. No channel bone animation is claimed; existing idle pose is used for the character capture.

All initial cohorts are preserved. BuildV1 passed; empty-state review then added a projector fast return and empty snapshot assertion (exact2 deltas,8 unchanged between ten-source inputs-v1/v2). WorldV1 passed76 native tests but actual character+ring PNG was rejected for T-pose: the fixture omitted native idle state after an epoch reset. A fixture-only follow-up sends current idle, asserts the actual animation slot and lights the camera-facing side; all9 production sources unchanged from inputs-v2 to final inputs-v3. Full freshWorldV2 and finalBuildV3 accepted. Original rawWorldV1/report/available images, every source cohort and all6 historical worker receipts remain archived. No failed native result is invented and the visually rejected run is not approved. E1528 records root causes and prevention.

Evidence:docs/acceptance/2026-10-23-channel-vfx/index.json; final/errors:docs/plans/2026-10-23-channel-vfx-{final-acceptance,errors}.md. All readers retired/no source freeze. Next concrete gap: native channel bone animation or another audited single-PC gameplay feature; do not repeat accepted channel strip/public duration/cast wrapping/nameplate checks. Channel bone animation, further gameplay effects, full combat replica,11 external tasks/MVP33/44 and known worldless first-kill crop/map focus-label overlaps remain pending. This closes common world channel VFX for maintained canonical metadata. No new real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.

## Channel native animation FINAL — supersedes native channel pose pending

OpenSpec complete-channel-native-animation4/4 complete. Shared native channel presentation selects explicit semantic mapping, then dedicated channel slot, then existing recall/cast clips only under configured generic fallback. Broken explicit mapping fails closed. Cursor is sampled from the accepted authoritative elapsed fraction and held between snapshots; local mesh time cannot predict completion. Life/identity/world-scope/tick/sequence checks precede reset. Death/respawn, recall and preparation retain priority; channel blocks ordinary locomotion/hit/release pose replacement. Current clear restores latest locomotion; hide/rebind/reset retire cached state and reveal alone cannot replay it. O(1) actor fields, no timers/scans/components, no hero/item-name special cases or Rust/wire/C ABI change.

UE5.8 buildV2/staged DLL/import SHA passed. WorldV2 executed77:76success+1existingwarningcase,0failed/0notrun. Source guard verified3442 inputs; full accepted Rust1844passed/0failed/9existingignored and backend result inherited after SHA proof3438 unchanged prior inputs+3 native changes+1 addition; no full Rust rerun claimed. Archive contains397 independently reverified files; 44 final accepted PNGs, with failed historical captures also retained. Health5 samples/max171 seconds. Four final workers and four historical workers (2successful/2failed) have creation-token-bound retired receipts. All launchers, two inheritance readers and the single170-second health monitor were reaped; CIM found no owned build/test/source worker remaining.

11 new/changed images actually viewed; 33 accepted prior SHA-identical images inherited. Three actual512x512 early/mid/late pose images and the combined turquoise-ring/character image show sampled channel posture. Existing sustained asset movement is subtle; exact normalized cursors are asserted on two real skeletons. No bespoke animation assets or new channel gameplay are claimed.

NativeWorldV1 correctly failed the second-skeleton first-state assertion because the fixture reused epoch101/tick1 after the first skeleton advanced that shared world epoch to tick4. Original raw77-test run/report/all available images, false World/source receipts and4 exact inputs-v1 remain archived. Fixture-only repair reserves100+H*10 epoch ranges and logs initial scope; all3 other native sources unchanged. Final buildV2/fresh fullWorldV2 use inputs-v2. Both real skeletons completed with initial epochs100 and110. E1529 records root cause and prevention; production admission was not weakened.

Evidence:docs/acceptance/2026-10-23-channel-animation/index.json; final/errors:docs/plans/2026-10-23-channel-animation-{final-acceptance,errors}.md. All readers retired/no source freeze. Next batch choose another concrete single-PC gameplay/operation gap from inventory; avoid repeating accepted channel/nameplate/cast-wrapping checks. Common canonical channel strip/public duration/world ring/native pose gaps are now closed within the maintained shared mechanism. More authored channel assets/content, further gameplay effects, full combat replica,11 external tasks/MVP33/44 and known worldless first-kill crop/map focus-label overlaps remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.

## Targeted control item IN PROGRESS

Using OpenSpec add-targeted-control-item (3/4). Ten exact formal inputs frozen in targeted-control-inputs-v1/manifest.json; do not edit maintained sources while readers run. Optional closed stun/root/silence is shared from Lua through content model/generated constants/runtime/native metadata. Item13 binding charm:30 damage/root1.5s/range450/CD18/cost650. Canonical BuffStore only on surviving targets; building rejection before all writes. Native description/hotbar reject malformed control.

Full8-scope Rust launcher targeted-control-rust-v1 currently running; abi/content/templates/codegen/core979/base328/bridge passed, client compiling. Two new actual60Hz tests passed movement/pause/expiry/public snapshot and generic control atomicity/building/refresh/lethal semantics. SINGLE170-second targeted-control-health-v1 active; do not duplicate. Planned sequential backend-v1/build-v1/staged-v1/source-capture-v1/world-v1/source-v1 use prepared fixed Lua helpers. Native expected78 and45 images; expected Rust1849/9ignored is only a plan until receipts. Draft acceptance/errors MD2026-10-23-targeted-control, E1530 records read-only path mistakes. No actual build/test failure yet. Retire/reap all readers before source repairs; preserve failed raw evidence if any. No network match/quota reset/commit/push/omfx; original deadline unchanged.


## Targeted control item COMPLETE

OpenSpec add-targeted-control-item4/4 complete. Previous IN PROGRESS section is historical; no workers/source freeze remain.

Shared optional control schema closes kinds to stun/root/silence with positive fixed-point duration up to60seconds. Generated Rust constants, native runtime configuration and native shop metadata preserve one authored policy. Authority preflights target/visibility/range/life/cooldown and rejects buildings before damage/status/cooldown writes. Canonical BuffStore control applies only to surviving targets and never truncates longer existing control. Existing item11 healing reduction remains unchanged.

Appended item13 moba_binding_charm / 縛行符: cost650, direct damage30, root1.5seconds, range450, cooldown18seconds. Chinese tooltip states control duration, building exclusion and existing visible-enemy/range-reject behavior. Invalid control metadata disables hotbar use. No special case by item/hero/camp name and no C ABI change.

Two actual60Hz formal tests verify real catalog purchase/use, damage, out-of-range atomic rejection, canonical public root, paused lifetime, blocked movement and pending movement resumed after expiry. Generic transaction coverage exercises all3kinds, malformed duration/building rejection without mutation, max-duration refresh and lethal-target exclusion. Native test verifies generated values, Chinese guidance, invalid-data rejection, prior-item compatibility and actual shared public-status countdown/removal; screenshot reviewed.

Full8-scope Rust1849passed/0failed/9existingignored (ABI15/content45/templates117/codegen71/core979/base328/bridge137/client157). Formal backend and UE5.8 buildV1 passed; staged DLL and bridge import SHA matched. NativeWorldV1 executed78:77success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3443 inputs;3429 unchanged prior inputs+10 exact formal inputs+4 generated deltas. Archive contains157 independently reverified evidence files and45 accepted PNGs:9 actual new/changed images viewed,36 prior SHA-identical images inherited. Health8 samples/max171 seconds. Six final creation-token-bound owned workers retired; all launchers/source readers and the single170-second health monitor reaped; final CIM found no owned build/test/source process remaining.

Evidence:docs/acceptance/2026-10-23-targeted-control/index.json; final/errors:docs/plans/2026-10-23-targeted-control-{final-acceptance,errors}.md.

E1530 records read-only guessed-path errors and prevention. No build/test failure occurred in this batch; existing warnings/ignored cases are retained. Source freeze released only after every reader retired. Further gameplay/content polish, full combat replica,11 external tasks/MVP33/44 and known worldless first-kill crop/map focus-label overlaps remain pending. This closes the shared targeted-control item mechanism, not every LoL feature. No new real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## Control item target feedback COMPLETE

OpenSpec complete-control-item-target-feedback4/4.

Validated control-item hotbar policy reaches the owner preview. A read-only bridge query first reuses current source/target identity, visibility, life, allegiance and attackability admission, then uses the current disclosed kind to report an otherwise legal building. The shared renderer shows「此道具無法指定建築」with warning color. Hidden/stale/foreign target kind remains unknown; non-control damaging items keep building admission. Slot replacement, source retirement and scope reset clear policy. No item/hero/camp name rule, actor-class guess, hidden query, gameplay gate, authority ACK claim or C ABI change.

Actual generated catalog13 versus11 exercised source-relative published tower/hero rows, foreign owner, ally/unattackable/hidden target, stale/current generation, current kind refresh, replacement and source retirement. Localized shared refusal screenshot actually viewed. Existing item feedback/native control status and the entire native suite remain accepted.

UE5.8 BuildV2 and staged DLL/import SHA passed. WorldV1 executed79:78success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3444 inputs; prior full8-scope Rust1849passed/0failed/9existingignored and backend accepted by exact SHA proof3436 unchanged inputs+7 native deltas+1 native addition; no new full Rust suite claimed. Archive contains131 independently reverified evidence files and46 final PNGs. Health6 samples/max171seconds. Four final and one failed historical worker have creation-token-bound retired receipts. All launchers, two inheritance readers and the single170-second health monitor were reaped; final CIM found no owned build/test/source process remaining.

BuildV1 failed C2248 because the new fixture lacked an existing-style WITH_DEV_AUTOMATION_TESTS friend declaration. Original failed build/log/false retired receipt,8inputs-v1 and initial inheritance proof retained. After all readers retired, only one test-guard friend line was added; other7 inputs unchanged. Inputs-v2 and fresh BuildV2 accepted. Production members remain private and admission was not weakened. E1531 records root cause, repeated read-only path mistakes and prevention.

Evidence:docs/acceptance/2026-10-23-control-item-feedback/index.json; final/errors:docs/plans/2026-10-23-control-item-feedback-{final-acceptance,errors}.md. All readers retired; source freeze released. Further gameplay/content polish, full combat replica,11 external tasks/MVP33/44 and known worldless first-kill crop/map focus-label overlaps remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.

## Detached HUD layout IN PROGRESS

OpenSpec complete-detached-hud-layout2/4. Five original native inputs frozen; new shared Slate geometry callback/weak UObject layout helper fixes worldless HUD first-kill crop. Real worldless1280/800/1600client resize bounds test passed in fullWorldV1. BuildV1 passed but WorldV1 raw80 rejected by one existing ObjectiveAnnouncementBanner fixture: private ApplyReadableFonts(2) was overwritten by real default profile during geometry update. Kept36px assertion and repaired fixture only to use SetInterfacePreviewForAutomation. Original5 inputs unchanged; preserved original sixth fixture bound to failed3445source-before-v1 baseline. Inputs-v2 contains6 exact inputs. Failed original80 run/all49images and false World/source receipts retained; no failed run accepted. AllV1 build/world/source capture/check and first inheritance reader retired/reaped before repair.

BuildV2/staged-v2 passed. FreshWorldV2, source-capture-v2 and source-v2 currently running via owned launchers; exactinherit-v2 reader running. SINGLE170-second detached-hud-health-v1 remains active (do not duplicate). Finalsource expected3445, unchanged3439+5 native deltas+1addition; Rust1849/9ignored and backend inherited only after SHA proof. Expected80native/49images; inspect actual final output before accepting. Need reap sessions, review images, stop/reap health with existing detached-hud-done.lua, execute corrected seal-v2 and update final docs/tasks/inventory. V2 sealer preserves parent control-item-feedback-source-before-v1 and original singlehealth-v1/done-v1; do not broadly rename inherited/monitor versions. No source mutation while readers remain. E1532 errors/root causes/prevention saved.


## Detached HUD layout COMPLETE

OpenSpec complete-detached-hud-layout4/4; preceding IN PROGRESS section is historical.

Shared resolution normalizer now publishes actual Slate geometry during Slate Tick through an optional delegate. The HUD binds only a weak UObject reference and calls the same memoized geometry/layout helper used by NativeTick. It updates even when no game world exists, retains the existing DPI normalization and measured compact reflow, and performs no layout mutation during arrange/paint or per-entity work. Three actual detached window sizes1280x720/800x600/1600x900client exercise worldless bounds and resize recovery without manual NativeTick. Existing real game-viewport/DPI coverage passed.

The standalone first-kill notice capture now shows complete combat panel/shop/inventory within the screenshot; prior lower/right crop is closed with actual before/after evidence. The no-world fixture still displays unknown gameplay data appropriately; no synthetic authority ACK or real network match is claimed.

UE5.8 BuildV2/staged DLL/import SHA passed. FullWorldV2 executed80:79success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3445 inputs; prior full8-scope Rust1849passed/0failed/9existingignored and backend inherited after exact SHA proof3439 unchanged inputs+5 native deltas+1 native addition; no new full Rust rerun claimed. Archive contains419 independently reverified evidence files and49 final PNGs; 12 actual new/changed images viewed, 37 accepted prior SHA-identical images inherited. Health7 samples/max171seconds. Four final and four historical workers (2successful/2failed) have creation-token-bound retired receipts. All launchers, both inheritance readers and the single170-second health monitor reaped; final CIM found no owned build/test/source worker remaining.

BuildV1 passed; rawWorldV1 executed80 but one ObjectiveAnnouncementBanner fixture failed its unchanged36px font assertion. It used a private temporary font override rather than persistent interface preferences. After all readers retired, retained that assertion and changed only the fixture to SetInterfacePreviewForAutomation(TextScale). All5 initial inputs stayed byte-identical; preserved the original sixth fixture bound to the failed3445source baseline. Final6inputs-v2/fullBuildV2/fullWorldV2 accepted. Original failed report/all49images, false World/source receipts and successful historical build/source-capture evidence remain archived. E1532 records cause/prevention and pre-reader edit/pre-execution sealer version mistakes; no production layout check or font threshold was weakened.

Evidence:docs/acceptance/2026-10-23-detached-hud/index.json; final/errors:docs/plans/2026-10-23-detached-hud-{final-acceptance,errors}.md. All readers retired; source freeze released. Known standalone first-kill HUD crop closed. Map focus-label overlaps, further gameplay/content polish, full combat replica and11 external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.

## Self cleanse item IN PROGRESS

OpenSpec add-self-cleanse-item2/4. Closed self_control_cleanse effect, authored item14淨化護符1400gold/CD45seconds, shared canonical control removal, native self-input/Chinese guidance and formal/native fixtures implemented. RustV1 found legacy self-item Warmup admission bypass; retained actual failing assertion/evidence, retired/reaped Rust launcher before repair. Common transaction now reuses active_hero_context_allows (same Stop/no-match policy) for pause/phase/roster rejection. Added legacy shield/new-cleanse atomic regression. Inputs-v2 freezes10 sources; exactly game_processor/formal tests differ fromV1, other8 unchanged.

Fresh full8-scope RustV2 running via self-cleanse-item-rust-v2; SINGLE170-second self-cleanse-item-health-v1 remains active, do not duplicate. No backend/UE/native reader launched yet. Prepared downstream backend-v1/build-v1/staged-v1/source-capture-v1/world-v1/source-v1; final native expected81/source3446/images50, Rust expected1856/9ignored. Treat expected counts as plans until receipts. Final sealer-v1 uses RustV2/inputsv2 and archives failedRustV1; parent detached-hud source baseline remainsV2. Original health/done marker remainV1. Source freeze active; do not mutate10 inputs while readers run. Need actual visual review, worker retirement/reaping, final seal/hash verification/docs/tasks. E1533 errors MD retained. Deadline/restrictions unchanged.

### Self cleanse recovery checkpoint

RustV2 full8scopes1856passed/0failed/9ignored and backendV1 passed; both launchers reaped. BuildV1/stage passed, source-captureV1 passed3446; nativeWorldV1 ran81 but SelfCleanseItem failed retained tooltip assertion because unconditional amount>0 preflight hides parameterless cleanse. World/sourceV1 false receipts and all images retained; every reader/launcher reaped before repair. Repaired only shared native tooltip and native fixture, kept original assertion and added5malformed metadata cases. Frozen inputs-v3:other8sources byte-identical versusv2. BuildV2 currently running. Fresh staged-v2/source-capture-v2/world-v2/source-v2 ready; final seal-v2 preserves RustV1 plus4historical native workers (5total). FinalRust stillV2, backendV1, parentbaselineDetachedHudV2; singlehealth-v1/done-v1 unchanged. Do not mutate frozen sources or duplicate health. Need actual final81/50PNG review/source3446/seal/retirement/docs acceptance.


## Self cleanse item COMPLETE

OpenSpec add-self-cleanse-item4/4.

Closed self_control_cleanse item effect reuses the same BuffStore::cleanse_controls operation as the maintained summoner cleanse. It removes only canonical stun/root/silence, preserves other buffs/payloads/indexes and foreign entities, grants no immunity/healing/shield, and keeps movement/attack command queues. Existing ordinary item preflight and controlled-use policy remain; all effect/cooldown writes follow owner/life/phase/target-contract/resource validation. The legacy command-clear policy for other self item effects is unchanged; no item/hero/camp name branch.

Appended item14 moba_purification_charm / 淨化護符: cost1400, self/no-target, cooldown45seconds. Strict author schema rejects extra immunity/duration/buff fields and unit-target contracts. Generated native metadata and Chinese guidance describe removals, unrelated debuff retention and absence of immunity. Native self-input uses ItemUse, not spell targeting. Public control removal/reapplication renders through existing canonical status path, no C ABI change.

Actual60Hz catalog purchase/use succeeded while all3controls were present; only own controls removed, unrelated effects/shield/antiheal and foreign controls preserved, HP unchanged, command retained, public status cleared, cooldown spent, reapplication still controls hero and cooldown replay refused. Rejection tests cover invalid slot/owner/phase/death/NaN cooldown without effect/command/cooldown mutation, and empty cleanse with legacy target bytes ignored/self-only execution and active movement order retained. Schema/constant/codegen/runtime/native tests passed.

Full8-scope RustV2 1856passed/0failed/9existingignored (ABI15/content46/templates118/codegen72/core980/base331/bridge137/client157). Formal backend and UE5.8 BuildV2 passed; staged DLL/import SHA matched. FullNativeWorldV2 executed81:80success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3446 inputs;3432 prior inputs unchanged+10 exact formal inputs-v3+4 generated deltas. Archive contains521 independently reverified evidence files and50 accepted PNGs; 10 actual changed/new images viewed,40 accepted prior SHA-identical images inherited. Health15 samples/max171seconds. Six final and five historical creation-token-bound workers retired; all launchers/source readers and the single170-second monitor reaped; final CIM found no owned build/test/source process remaining.

NativeWorldV1 also rejected the new guidance assertion: the legacy unconditional positive-amount tooltip guard hid the parameterless effect. Original81 report/screenshots and false receipts are preserved. After all readers retired, only the shared native tooltip guard and its fixture changed; five malformed effect-metadata cases were added and original guidance assertion retained. Inputs-v3 freezes the native repair with other8 byte-identical inputs. RustV2 remains bound to its exact unchanged Rust/Lua inputs; UE BuildV2/fullWorldV2/sourceV2 reran. Successful historical build/source capture and both failed native/source runs remain archived; five historical workers comprise2successful/3failed including RustV1.

RustV1 exposed an actual legacy self-item admission bypass: Warmup use consumed an item. Preserved its failing assertion, raw six-scope logs/false retired receipt and10 exact inputs-v1. After every reader retired, repaired the common transaction with the existing active-hero context helper, preserving no-match/Stop semantics; both legacy shield and new cleanse now reject pause/non-playing/mismatched roster atomically. Only game_processor and formal test sources changed; other8 exact inputs remained unchanged. Fresh full8-scope RustV2 accepted; no failed run accepted. E1533 records this cause/fix/prevention and pre-reader workflow/path corrections. Evidence:docs/acceptance/2026-10-23-self-cleanse-item/index.json; final/errors:docs/plans/2026-10-23-self-cleanse-item-{final-acceptance,errors}.md. All readers retired/source freeze released. Map focus-label overlaps, further gameplay/content, full combat replica and11 external tasks/MVP33/44 remain pending. Standalone first-kill HUD crop remains closed. No real network matches/quota reset/commit/push/omfx maintenance. Original deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## Focus nameplate collision IN PROGRESS

OpenSpec complete-focus-nameplate-collision2/4; preceding complete-nameplate-screen-spacing remains4/4. Removed focus-only hero reservation bypass from shared Admit API and maintained bridge caller; selected/hovered retain candidate ordering and selected-details HUD, all nonhero plates avoid hero footprints. Heroes stay visible; hero-to-hero overlaps/64reservation limit remain pending. Three exact native inputs frozen in focus-nameplate-inputs-v1. Native fixture adds scaled reservations/rejection atomicity/fresh recovery; actual focused capture caption describes avoidance.

BuildV1/staged SHA passed and launcher reaped. FullWorldV1, source-capture-v1/source-v1 and exact inherit-v1 currently running; SINGLE170second focus-nameplate-health-v1 active, do not duplicate. Expected81native/source3446/50PNG and inherited prior1856Rust/9ignored/backend only if3443unchanged+3native delta proof succeeds. Source freeze active. Need reap all sessions, actual image comparison/review, finalsource check, done marker/health retirement, focus-nameplate-seal-v1.lua independent hash verification and docs/tasks/inventory. No native failure yet. Fixed Lua preparation helper substring error recorded in docs/plans/2026-10-23-focus-nameplate-errors.md; no source repair due to helper. Prior self-cleanse accepted baseline staysself-cleanse-item-source-before-v2.json; do not rename inherited baseline. Deadline/restrictions unchanged.


## Focus nameplate collision COMPLETE

OpenSpec complete-focus-nameplate-collision4/4; preceding IN PROGRESS checkpoint is historical.

All current nonhero healthplates now share the same bounded screen-space collision checks against hero reservations and earlier admitted nonhero rows. Removed the focus bypass from the shared Admit API and maintained bridge caller. Selected/hovered targets keep stable shortlist priority, selection markers and existing selected-unit HUD details; heroes remain visible and labels stay attached to actual world anchors. Hidden/disclosed identity guards, geometry measurement, fresh reservation lifecycle and64slot bounded hot path are unchanged. No hero/unit/camp name special case.

Measured actual hero layout/capture verifies the focused nonhero plate no longer obscures the hero. Rejected candidates do not occupy admission storage. Four UI scales, fresh recovery, existing concealment/retirement/generation/movement/capacity checks passed; consolidated public-selection/selected-details tests passed. Hero-to-hero overlaps and the64hero reservation cap remain explicit independent limits; this closes the focus-to-hero exception only.

UE5.8 BuildV1/staged DLL/import SHA passed. FullWorldV1 executed81:80success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3446 inputs;3443 prior inputs unchanged+3 exact native deltas prove inheritance of prior full8-scope Rust1856passed/0failed/9ignored and formal backend; no new full Rust run claimed. Archive contains105 independently reverified evidence files and50 accepted PNGs;12 actual changed images viewed,38 prior SHA-identical images inherited. Health4 samples/max171seconds. Four owned workers retired; all launchers, exact inheritance reader and single170second health monitor reaped; final CIM found no owned build/test/source worker remaining.

No build/test failure occurred. Errors/prevention include the pre-reader Saved helper substring assumption; completed its remaining visual comparator directly without rerunning partial mutations. Evidence:docs/acceptance/2026-10-23-focus-nameplate/index.json; final/errors:docs/plans/2026-10-23-focus-nameplate-{final-acceptance,errors}.md. Source freeze released after every reader retired. More gameplay/content polish, hero-to-hero overlap, full combat replica and11 external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## Summoner self sprint IN PROGRESS

OpenSpec add-summoner-self-sprint2/4. Added closed SelfSprint operation/schema/generated constants/real ECS sink/default unsupported sink rejection. Shared BuffStore grant_timed_modifier preserves item wrapper/family identity; reserved @summoner source shares strongest sprint aggregation with independent expiry. Appended catalog5疾行:80flat speed/6seconds/CD180; preserves controls/base speed/commands, no collision bypass or immunity. Formal60Hz/model/ECS/schema/codegen/native tooltip tests implemented. No new public effect timer/icon claimed; existing owner effective stats and authored guidance cover disclosure.

RustV1 ABI/content passed, templates83passed/1failed due legacy fixed4 summoner count in trinket loader fixture. Original10inputs-v1 and raw false retired receipts retained; extra original lua_content.rs bound to prior accepted baseline before editing. All readers retired/reaped before repair. Updated loader to5+explicit typed sprint data; moved runtime enum variant to physical end preserving existing serde order. Frozen11inputs-v2 (9initial inputs unchanged). Fresh full8scope RustV2 running, session72502; SINGLE170second summoner-sprint-health-v1 session7047 active, do not duplicate. Backend/build/staged/source-capture/world/source-v1 not started. Prepared helpers; expectedRust1862/9ignored/native82/source3447/images51. Source freeze active; counts remain plans until receipts. Sealer-v1 preserves failedRustV1/originalextra. Need actual counts/native/images/build/source/retirement/fullseal/docs. Prior focus-nameplate baseline isfocus-nameplate-source-before-v1.json. Deadline/restrictions unchanged.


### Summoner sprint V3 recovery checkpoint

RustV2 passed ABI/content/templates/codegen/core983, formal release compile failed guessed HeroCommand::Hold in new test (actual HoldPosition). Preserved raw six-scope logs/summary/false retired receipt and11inputs-v2. Every reader/launcher reaped before one-token fixture correction; frozen11inputs-v3,10 unchanged. Fresh full8scope RustV3 running session54801; single170second health-v1 session7047 remains active, do not duplicate. Backend/UE/native not yet started. Final sealer usesRustV3/inputsv3, preserves both failed histories plus original extra fixture bound prior baseline. Prepared full close/review helpers; expected1862Rust/82native/3447source/51images remain plans until actual receipts. No source edits while readers run.


### Summoner sprint native acceptance checkpoint

FullRustV3 passed1862/0failed/9existingignored; formal60Hz sprint passed, base332/core983. Rust launcher reaped. BackendV1/buildV1/staged DLL/import SHA passed and launchers reaped. FreshWorldV1 session33536, source-capture-v1 session58990 and source-v1 session62466 running. Single170second health-v1 session7047 remains active. Expected82native/source3447/51PNG pending actual receipts; prepared visual/review/seal/close helpers. Source freeze11inputs-v3 remains. Final sealer-v1 usesRustV3 and preserves failedRustV1/V2 plus original eleventh fixture. Need actual visual review, source proof, reader retirement/reaping, done/health retirement, independent seal and docs.


### Summoner sprint generalization audit — do not accept V1 native as final

RustV3/backendV1/UEBuildV1/stage and fullWorldV1 82passed (81success+1existingwarningcase,0failed) succeeded. Actual sprint tooltip viewed/readable. World launcher session33536 reaped. Source-capture-v1 session58990 completed; source-v1 session62466 may still hash. Singlehealth-v1 session7047 remains. No formal mutation yet: wait/reap both source readers first.

Final audit found actual source aliasing in new EcsSink: reserved constant @summoner:self_sprint aliases distinct custom authored sprint IDs allowed by loadout. Both slots may carry different IDs with same SelfSprint operation, so weaker can overwrite stronger and their durations cannot remain independent. Need derive reserved source from validated loadout.spell_id(request.slot) inside the same transaction, store it in EcsSink, and use shared grant with that source; no item/hero name branch. Add actual two-distinct-custom-sprint ECS regression: stronger100/2seconds, weaker80/6seconds; max100 then80 then0. Update formal pause lifetime check to locate magnitude rather than private source spelling. This is a required general solution repair, not a claimed failed existing run. Preserve successful fullRustV3/inputsv3/backend/build/world/sourceV1 as historical evidence before edits.

After every reader retired, freeze inputs-v4 with exactly native/summoner_world.rs and formal single_lane_match_tests.rs changed from11v3. Run fullRustV4 expected1863 (core984)+backend/build/staged/source-capture/world/sourceV2; native remains82/images51/source3447. Sealer must retain2failed Rust histories plus6successful V3/backend/native histories (8historical workers total), all original inputs. Final priorbaselinefocus-nameplate-source-before-v1 remains unchanged; SINGLEhealth/done stayV1. Current seal-v1/close-docs still describes V3 and must be adapted before execution; do not execute acceptance helpers against old native run.


### Summoner sprint V4 source identity repair running

All V1 native/source readers retired/reaped. Repaired sprint source to @summoner:<validated loadout spell ID> in EcsSink; two distinct definitions preserve strongest/independent expiry. Added core real100/2+80/6 regression, formal pause uses actual magnitude rather than private source string. Frozen11inputs-v4,9unchanged. FullRustV4 session88069 running; singlehealth-v1 session7047 stays active. Prepared fresh backend/build/staged/source-capture/world/sourceV2 and visual/reviewV2/sealV2; finalclose updatedV4. Expected1863/core984/native82/source3447/51PNG. FinalsealerV2 preserves all8historical workers (2failedRustV1/V2 +6successfulRustV3/backendV1/buildV1/worldV1/sourcecaptureV1/sourceV1). Prior focus baseline, singlehealth/done remainV1. Do not execute oldsealV1 or mutate frozen sources while readers run.


## Summoner self sprint COMPLETE

OpenSpec add-summoner-self-sprint4/4; preceding recovery checkpoints are historical.

Added closed self_sprint operation with positive Q10 movement bonus<=10000 and duration<=60seconds through author schema/generated constants/authority model/real ECS sink. Unsupported sinks fail before cooldown. The shared timed modifier grant preserves existing item wrapper and historical aggregation identity. Reserved summoner source cannot collide with authored item IDs. Sprint sources use strongest-family magnitude with independent expiration; no base-speed writes, command cancellation, control removal, immunity or collision bypass. Existing owner/life/phase/pause/shape/cooldown transaction gates remain.

Appended catalog5 training_self_sprint_v1 / 疾行:80flat speed for6seconds, cooldown180seconds. Native D/F self card uses actual generated Chinese author description and existing cooldown state, both slots/retirement/replay tested. No new public effect-duration timer or dedicated icon is claimed; current effective owner stats and authored guidance remain the disclosure surface.

Schema/codegen bounds, generated loader/balance identity and unsupported model sink tests passed. Real ECS verifies active movement preserved, root retained, base speed unchanged, cooldown charged only after effect, weaker item sprint restored after summoner expiry and independent item expiry. Atomic context/missing-BuffStore rejection coverage and actual60Hz catalog input verify strongest magnitude/foreign-owner isolation/HoldPosition preserved/owner-team cooldown/pause/duplicate refusal/normal expiry.

Full8-scope RustV4 1863passed/0failed/9existingignored (ABI15/content47/templates118/codegen73/core984/base332/bridge137/client157). Formal backend/UE5.8 BuildV2/staged DLL/import SHA passed. FullWorldV2 executed82:81success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3447 inputs;3433 prior inputs unchanged+11 exact formal inputs-v4+3 generated deltas. Archive contains599 independently reverified evidence files and51 accepted PNGs;11 actual changed/new images viewed,40 prior SHA-identical images inherited. Health17 samples/max171seconds. Six final and eight historical owned workers (6successful/2failed) retired; all launchers/readers and single170second health monitor reaped; final CIM found no owned build/test/source worker remaining.

RustV1 failed a legacy fixed4summoner count in the trinket loader fixture; retained original fixture SHA bound to prior accepted baseline and replaced with5 plus actual sprint identity/data assertions. RustV2 passed first5scopes then formal compilation rejected a guessed HeroCommand::Hold test assertion; actual helper uses HoldPosition. Every reader retired/reaped before each repair. Original10inputs-v1/extra fixture,11inputs-v2, both raw failed roots and false retired receipts preserved; final11inputs-v4 accepted after full8scope rerun. Runtime enum addition sits at its physical end to preserve existing serde order. Successful fullRustV3/backendV1/UEBuildV1/fullWorldV1/sourceV1 cohort is also preserved. Final audit found an operation-wide reserved sprint source aliases different authored IDs. After all readers retired, derive the source from validated current loadout identity and add real100/2second+80/6second regression (100 then80 then0). Only native summoner_world and formal pause fixture changed from11inputsv3;9others unchanged. Fresh fullRustV4/core984/backendV2/UEV2/fullWorldV2/sourceV2 accepted. Final8historical workers include6successful/2failed. No successful earlier run substitutes for the repaired final source. E1535 errors/prevention:docs/plans/2026-10-23-summoner-sprint-errors.md. Evidence:docs/acceptance/2026-10-23-summoner-sprint/index.json. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap/64hero reservation limit, full combat replica and11 external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## Summoner cooldown readability IN PROGRESS

OpenSpec complete-summoner-cooldown-readability2/4. Two native sources frozen: shared widget timer/pair formatting and existing SummonerCastFeedback fixture. Positive<0.1 uses<0.1, exact0 remains0, one-decimal text trims.0; invalid negative/nonfinite/inverted pair unavailable; actual tooltip names remaining/total. Range scalar separate, existing compact ceil countdown unchanged, no local timer or new input admission. Added both-slot tiny/fractional/zero/invalid/scope-clear and existing real capture.

Single170second summoner-time-health-v1 session40160 active. Build-v1 session39487 and exactinherit-v1 session31687 running; no native/source readers yet. Expected82native/3447source/51PNG and inheritance1863Rust/9ignored/backend only after3445unchanged+2native proof. Prepared fixedLua runners/visual/sealer. Need build/stage, source-capture/world/source-v1, actual image review, worker retirement/reaping/done/health stop, finalseal/hash/docs/tasks. Sources frozen; do not duplicate workers. Prep helper stopped in memory on class replacement in done marker, missing health launch started no process; corrected conditional before freeze, error MD saved. Prior baseline summoner-sprint-source-before-v2.json; do not rename parent version. Deadline/restrictions unchanged.


### Summoner time V2 recovery checkpoint

Initial fullWorldV1 ran82 but ResponsiveHudLayout failed retained 範圍850cm assertion due850.0 formatting. Native summoner edge cases passed; sourcecaptureV1/inheritV1/buildV1 passed; dependent sourceV1 false. All readers/launchers retired/reaped, only singlehealth-v1 session40160 active. Repaired both frozen sources with separate range scalar, signedzero normalization/test. Inputs-v2 contains2changed native sources. FreshBuildV2 session11613 running; staged-v2/sourcecapture/world/sourceV2 helpers ready. Finalinherit-v2 is invoked by source-v2 after fresh sourceguard to prove3445unchanged+2native without redundant fullhash. Finalseal-v2 retains4historical workers/fullfailed51images/initialproof. Finalvisual/review/close helpers ready. Need actual fresh82/source3447/51image review and retirement/done/seal/docs; no more formal source changes while readers run. Parent summoner-sprint source baseline remainsV2; singlehealth/done remainV1.


## Summoner cooldown readability COMPLETE

OpenSpec complete-summoner-cooldown-readability4/4; preceding checkpoints historical.

Shared logical/expanded D/F cooldown pair formatting now keeps positive values below0.1seconds visibly positive (<0.1), normalizes signed zero to0 and displays one decimal without trailing.0. Negative/nonfinite/inverted pairs disclose unavailable timing. Actual tooltip names current remaining/total seconds from the same disclosed row. Distance has a separate scalar formatter preserving existing integer/fractional range text; existing compact ceil countdown, readiness, authority and input gates remain unchanged. No local countdown/extrapolation or spell-name branch.

Actual existing SummonerCastFeedback fixture covers D/F tiny positives, fractions, exact/signedzero, malformed pairs and cleared scope. Retained responsive viewport range assertion and full native current-frame/retirement/selection tests passed. Actual shared tooltip capture shows explicit cooldown and positive tail sample. All51 images compared; changed captures actually viewed.

UE5.8 BuildV2/staged DLL/import SHA passed. FullWorldV2 executed82:81success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3447 inputs;3445 prior inputs unchanged+2 exact native inputs-v2 prove inheritance of prior full8scope Rust1863passed/0failed/9ignored and backend; no new full Rust run claimed. Final inheritance is bound to fresh full-source verification, not a second redundant whole-tree hash. Archive contains401 independently reverified files and51 accepted PNGs;15 actual changed images viewed,36 prior SHA-identical images inherited. Health7 samples/max171seconds. Four final and four historical owned workers (2success/2failed) retired; all launchers/initial inheritance reader/source readers and single170second health monitor reaped; final CIM found no owned build/test/source worker remaining.

WorldV1 ran82 but ResponsiveHudLayout failed its retained integer range assertion because SanitizeFloat changed850cm to850.0cm. Original2inputs-v1, full report/all51screenshots, falseWorld/source receipts and successful build/sourcecapture/initialinheritance retained. Every reader retired/reaped before restoring separate distance scalar formatting and adding signedzero normalization/coverage. Fresh2inputs-v2/fullUEV2/fullWorldV2/sourceV2 accepted; no failed run accepted. Preparation class-string/missing-helper failure occurred before readers, was corrected and recorded. Errors:docs/plans/2026-10-23-summoner-time-errors.md; evidence:docs/acceptance/2026-10-23-summoner-time/index.json. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap/64hero reservation cap, full combat replica and11 external tasks/MVP33/44 remain pending. No network match/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## Item cooldown readability IN PROGRESS

OpenSpec complete-item-cooldown-readability2/4. Two native inputs frozen under item-time-inputs-v1. Shared scalar across item logical/expanded/concise text, six-slot positive/zero/malformed/passive/clear tests and existing actual capture updated. Singlehealth session51789 and UEbuild-v1 launcher60964 active. Need stage/sourcecapture/fullWorld82/source3447/inheritance1863/visual51/seal/retirement/docs. Helpers item-time-*-v1 prepared; no formal source edits while readers run. No duplicate jobs. Deadline unchanged.


### Item time V2 recovery checkpoint

WorldV1 executed82; new six-slot tests passed, ResponsiveHudLayout failed exact old2.0秒 /2.0/5.0秒 strings after intended common scalar trim. All4V1 readers retired/reaped; original extra fixture saved and SHA bound to source-before-v1. Changed only expected2秒 /2/5秒; retained semantic/tooltip/font/layout checks. Inputs-v2=3native files; widget and item fixture unchanged fromV1. BuildV2 and staged SHA passed. Current sourcecapture-v2 session59248, world-v2 session87179, source-v2 session72236; singlehealth-v1 session51789 remains. Expected3447source/3444unchanged+3native/82native/51PNG/inherited1863Rust. Helpers version-relative audit corrected before reader launch; seal-v2 retains all4historical readers, failedWorldV1 full51images, original2inputsv1+extra fixture. Need reap final readers, actualvisualreview, done/healthstop/seal/docs. No formal source edits while frozen; no duplicatejobs.


## Item cooldown readability COMPLETE

OpenSpec complete-item-cooldown-readability4/4; preceding checkpoints historical.

All six item slots now reuse the D/F cooldown scalar in logical pairs, expanded countdown and concise labels. Positive tails below0.1seconds display<0.1 rather than0.0; fractional values retain one decimal and trailing.0 is omitted. Exact signedzero hides the card countdown. Negative/nonfinite/inverted pairs disclose unavailable timing; passive/empty/cleared slots do not retain active countdown. Existing compact ceiling, authority/readiness/input policy remain unchanged; no local timers or identity branches.

Existing ItemCastFeedback fixture now covers all six slots, tiny/fractional positives, signedzero readiness, four malformed pairs, passive suppression and economy scope clearing. Its actual Slate capture includes the live logical positive-tail label alongside existing target/range/visibility guidance. Retained D/F, responsive HUD and action retirement tests remain in the full native suite.

UE5.8 BuildV2/staged DLL/import SHA passed. FullWorldV2 executed82:81success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3447 inputs;3444 prior inputs unchanged+3 exact native inputs-v2 prove inheritance of prior full8scope Rust1863passed/0failed/9ignored and backend; no new fullRust run claimed. Inheritance bound to fresh source verification. Archive contains409 independently reverified files and51 accepted PNGs;12 changed images actually viewed,39 prior SHA-identical images inherited. Health6 samples/max172seconds. Four final and four historical workers (2success/2failed) retired; all launchers/source readers and single170second health monitor reaped; final CIM found no owned build/test/source worker remaining.

Preparation path guesses and atomic patch mismatch were corrected before build/readers and recorded in docs/plans/2026-10-23-item-time-errors.md. WorldV1 failed exact old 2.0-second presentation expectations after intentional removal of trailing.0. Original fixture SHA bound to failed source baseline, original2inputs-v1 and extra fixture, failed full report/all51images and all4retired receipts preserved. Repaired only two expected text literals to2 and2/5; retained all semantic/status/tooltip checks. Fresh3inputs-v2/fullUEV2/fullWorldV2/sourceV2 accepted. No failed runtime accepted. Evidence:docs/acceptance/2026-10-23-item-time/index.json. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap/64hero reservation cap, full combat replica and11 external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## All hero nameplate reservations IN PROGRESS

OpenSpec complete-all-hero-nameplate-reservations2/4. Removed hero top64 heap and fixed64 rectangle admission; all projected current visible actor-matching hero footprints reserve in existing pass, nonhero64 budget unchanged. Three sources frozen all-hero-inputs-v1. Existing fixture adds distinct65th/order/exactgap/10000storage/64nonhero and actual65th-reservation lifecycle/capture. Hero-to-hero overlap pending;10000 is helper reservation test, not full rendered stress-world performance. UEbuild/stagedV1 passed. Singlehealth session28755; sourcecapture86137/world78323/source reader active. Need native82/source3447/exact3444unchanged+3native inheritance1863/51PNG review/seal/retire/docs. No source edits or duplicatejobs while frozen. Original deadline unchanged.


## All hero nameplate reservations COMPLETE

OpenSpec complete-all-hero-nameplate-reservations4/4; preceding checkpoints historical.

Every current projected visible hero with matching actor entity/generation now reserves its exact measured screen footprint in the existing public-target pass. Removed the top64 hero heap and reservation admission cap; inline64 storage can grow. Nonhero shortlist, priority, focus semantics and output remain bounded64. Reservation union is independent of hero input order, with exact per-rectangle overlap rather than a conservative enclosing rectangle. Fresh view state preserves hidden/retired/new-generation/moved behavior; hero rendering and world anchors remain unchanged. Hero-to-hero overlap remains pending.

Existing HeroNameplateReservation fixture verifies a distinct65th hero footprint, reversed input order, available exact gap,10000 valid reservations, retained64 nonhero output, malformed geometry and four UI scales. Actual current hero widget becomes65th reservation for concealment/retirement/reveal/new-generation and two actual ordinary/focused captures. Existing movement recovery retained. The10000 case exercises reservation storage/admission, not a claim of10000 live rendered hero widgets or a full stress-world benchmark. Storage O(H), bounded nonhero overlap work O(H*64); each actual projected visible hero has one measured layout in this pass.

UE5.8 BuildV1/staged DLL/import SHA passed. FullWorldV1 executed82:81success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3447 inputs;3444 prior inputs unchanged+3 exact native inputs-v1 prove inheritance of prior full8scope Rust1863passed/0failed/9ignored and backend; no new fullRust run claimed. Inheritance bound to fresh source verification. Archive contains110 independently reverified files and51 accepted PNGs;13 changed images actually viewed,38 prior SHA-identical images inherited. Health4 samples/max172seconds. Four owned workers retired; all launchers/source readers and single170second health monitor reaped; final CIM found no owned build/test/source worker remaining.

Pre-reader geometry review corrected one gap fixture width to fit both padded neighbors; call-site review preserved actor identity/generation validation when moving work between phases. Errors/prevention:docs/plans/2026-10-23-all-hero-errors.md. No failed runtime accepted. Evidence:docs/acceptance/2026-10-23-all-hero/index.json. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap, full combat replica and11 external tasks/MVP33/44 remain pending. The former64hero reservation truncation is closed. No real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## Recall action IN PROGRESS

OpenSpec add-recall-action-button2/4. Added shared RequestRecallCard delegating existing cursor-free recall, validated ordered current state button, actual binding/tooltip, disabled pointer shell/press-time modal, HUD binding/retirement and header layout notifications. Existing RecallChannelProgress fixture extended actual SButton transitions/disabled pointer/rejection/retirement; capture ready+3channels. Five sources frozen recall-action-inputs-v1. Singlehealth session29746;build-v1 launcher89442 active. Need stage/sourcecapture/fullWorld82/source3447/3442unchanged+5native inheritance1863/51PNG actualreview/seal/retire/docs. No source edits while frozen, no duplicatejobs. No real authority ACK claimed. Deadline unchanged.


### Recall action V2 mode guard checkpoint

BuildV1/stage passed; before native/source readers, audit found retained MOBA recall payload across SetAuthoritativePresentation(2/0). Sole build reader retired/reaped before explicit IsMobaPresentation gate and same-live-button TD/waiting/MOBA regression. Original5inputs-v1 and successful BuildV1/stage preserved; fresh5inputs-v2 has2changed/3unchanged. BuildV2 launcher26498 active; singlehealth-v1 session29746 continues. Final helpers sourcecapture/world/source/visual/sealV2 ready, parent all-hero-source-before-v1 remainsV1. Finalsealer preserves1historical successful build, no historicalnative claimed. Need stage/full82/source3447/inheritance1863/51PNG review/retire/seal/docs. Do not mutate frozen sources or duplicatejobs.


### Recall action V3 mounted Slate recovery

WorldV2 ran82, failed only RecallChannelProgress new unmounted registered-attribute checks. Actual captured ready/channel-disabled UI already rendered correctly. AllV2 readers retired/reaped (build/capture true,world/source false). Changed only fixture to mount action+strip in real window, Pump prepass/tick/redraw after state/shop/mode/reset/retirement; hidden cases require collapsed root plus model rejection, visible states retain actual SButton check. Inputs-v3=5,1fixturechanged/4unchanged fromV2. BuildV3 launcher38324 active; singlehealth-v1 session29746 remains. Prepared sourcecapture/world/source/visual/sealV3, parent all-hero-source-before-v1 unchanged. Finalseal retains5historicalworkers:3successful(buildV1,buildV2,captureV2)/2failed(worldV2,sourceV2), original5inputsV1/V2 and fullfailedWorldV2 all51images. Preparation delimiter parse failed before executing, corrected to equals-delimited Lua and recorded. Need freshbuild/stage/full82/source3447/3442+5inherit1863/51PNG actualreview/finalretire/seal/docs. No more source mutations while frozen.


## Recall action button COMPLETE

OpenSpec add-recall-action-button4/4; preceding checkpoints historical.

Added a discoverable native header recall action alongside the existing read-only progress. RequestRecallCard delegates to the existing cursor-free recall reflection/owner/modal/life/admission path, without coordinate or queue state. Current HUD/bridge binding and retirement guards match other cards. Shared pointer shell consumes disabled press/release and handles press-time keyboard blocks; the button does not take focus. No new backend operation, local countdown or inferred completion/ACK.

The action reads the ordered authoritative recall snapshot. Ready requires current valid alive/playing owner/entity, legal authored total and exactzero remaining; active channel/shop/invalid/dead/finished/retired cannot present ready. Unknown protocol/scope/HUD chrome collapses it. Label reads actual primary binding, tooltip lists actual primary/secondary, authored total and interruption guidance. Existing progress predicate shares equivalent snapshot validation; new disclosure/clear transitions notify header layout. Authority retains all final command gates, including conditions not disclosed by this HUD payload.

Existing RecallChannelProgress fixture tests the actual same live SButton during channel/zero/shop, default binding, no-owner safe activation, malformed/dead/ended states, retired visibility, actual disabled shell pointer handling, existing cursor-free missing-surface rejection and no-target/no-queue semantics, owner retirement. Existing actual capture contains ready action plus three current progress rows. Existing responsive and action-owner regressions passed. This is single-machine native UI/input-path verification, not a real authority ACK or match.

UE5.8 BuildV3/staged DLL/import SHA passed. FullWorldV3 executed82:81success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3447 inputs;3442 prior inputs unchanged+5 exact native inputs-v3 prove inheritance of prior full8scope Rust1863passed/0failed/9ignored and backend; no new fullRust run claimed. Inheritance bound to fresh source verification. Archive contains447 independently reverified files and51 accepted PNGs;8 changed images actually viewed,43 prior SHA-identical images inherited. Health8 samples/max171seconds. Four final and five historical workers (3success/2failed) retired; all launchers/source readers and single170second health monitor reaped; final CIM found no owned build/test/source worker remaining.

Errors/prevention:docs/plans/2026-10-23-recall-action-errors.md. Read-only inventory path misses and pre-reader pointer/layout review preserved. Successful BuildV1/stage and original5inputs-v1 preserved; before native/source readers, mode audit found a retained MOBA snapshot could expose the action after TD/waiting switch. After build reader retired/reaped, added explicit presentation guard and actual same-button TD/waiting/recovery assertions. WorldV2 ran82 and failed unmounted Slate cached-attribute assertions in RecallChannelProgress; production ready/disabled capture already rendered correctly. All readers retired before changing only fixture to mount an actual window and pump Slate frames. Hidden children may retain cached enabled state, so hidden-state cases require collapsed root and model admission false; visible states still require actual button disabled. Original5inputs-v2/all failed51images/report and4retired receipts retained. Fresh5inputs-v3/fullUEV3/fullWorldV3/sourceV3 accepted; no failed runtime accepted. Evidence:docs/acceptance/2026-10-23-recall-action/index.json. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap, full combat replica and11 external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## Timed modifier status IN PROGRESS

OpenSpec expose-timed-modifier-status2/4. Shared typed-family canonical movement_boost/damage_reduction projection validates exact integer payload/lifetime and rejects forged reserved names; authored two native-only Chinese IDs. Store/codec/actual summoner ECS + actual filtered Rust ABI export/new native HUD fixture/capture implemented. Original6sources frozen modifier-status-inputs-v1. Full8RustV1 passed1867/0failed/9ignored (15/47/118/73/987/332/138/157), launcher27987 retired/reaped. Singlehealth35592 remains. Fullsource snapshot rust-source-v1 launcher31118 running to bind all Rust inputs before native-only repair: source key audit found movement26 key0x9ca5c927d356ae1a, new C++fixture currently int64 parse mustbeuint64. Prepared native-key-fix.lua, run only after snapshot success/retirement/reaping; it verifies6old hashes/baseline, fixes onlyC++fixture+adds bitwiseassert, freezes6inputs-v2 with5Rust/Lua unchanged. No production Rust changes; no fullRust rerun needed after C++-only fix, finalsealer proves all final deltas from Rust source snapshot limited to nativefixture/exact generated outputs. Backend/build/sourcecapture/world/sourceV1 prepared but not launched. FullWorld83 uses actual new Rust ABI fixture+52PNG. SealerV1 ready for7finalworkers Rust/Rustsource/backend/build/world/sourcecapture/source;6formal+explicit generated whitelist/source3448/fresh1867, old6inputs archived. Need nativefix/backend/build/stage/full83/sourceguard/actualvisual/seal/retire/docs. No duplicatejobs, noformal edits before active snapshot retires. Deadline unchanged.


### Modifier status native-only repair and backend passed

FullRustV1 1867passed/0failed/9ignored verified; fullRust source snapshot rust-source-before-v1 captured3448 and reader31118 retired/reaped. Ran native-key-fix after allreaders retired: only C++fixture uint64 parsing+bitwise row key assertions;6inputs-v2,5Rust/Lua unchanged, original6inputs-v1 retained. native-only-repair.json binds fullRust snapshot and old/new nativeSHA. BackendV1 success/launcher64448 reaped. UEbuildV1 active. Singlehealth35592 continues. Finalsealer now7workers inclRustsource snapshot, checks finalfullsource vsRustsnapshot allowing onlyNativefixture+explicitgenerated files, freshRust1867 remains valid; finalnative83/52PNG andsource3448 planned. Need UE/stage/sourcecapture/world/source/actualvisual/seal/retire/docs. No source edits while frozen.


### Timed modifier V3 final acceptance checkpoint

FullWorldV2 failed only new POD import fixture (uninitialized payload); sourceV2 correctly failed dependency. All readers retired/reaped; only fixture changed to Buff={} before assigning ABI fields. Original7inputs-v3/full52failedPNG/report preserved. Fresh7inputs-v4, proof native-only-repair-v3 binds unchanged5Rust/Lua plus2native to fullRust1867 snapshot. BuildV3/stage passed/reaped59686. FullWorldV3 passed83(82success+1existingwarningcase,4warnings), launcher9145 reaped;52PNG includes8actualchanged/new viewed +44priorSHAidentical, visual-v3 reviewed. CaptureV3 passed3448/reaped19799. Diff15=7formal+8explicitgenerated;3433unchanged. SourceV3 launcher9995 currently running fullhashcheck; single170sec health35592 remains. Finalsealer seal-v3 now exactV3 source/capture paths,7final+5historical(3success/2fail), fullfailedV2 retained, inputs-v4/nativeproofv3. Need source success/reap9995, done marker/reap35592/CIMnone, seal-v3/reap/close-docs-v3/tasks. Finalbaseline source-before-v3, not Rustsourcebeforev1. No more formal edits while frozen; no duplicatejobs. Deadline unchanged.


## Timed modifier public status COMPLETE

OpenSpec expose-timed-modifier-status4/4; preceding checkpoints historical.

Shared typed sprint/damage-reduction families now disclose canonical movement_boost / damage_reduction IDs with authored Chinese names 移速提升 / 傷害減免. Validation requires the closed family, exact integer stat payload, valid magnitude and positive<=60-second lifetime. Forged reserved names, malformed/extra/float fields and invalid lifetimes do not become public status. Existing BVS1 codec/catalog/owner consumers are reused. No private source key, spell/item identity or magnitude is published; source names are not parsed. Gameplay strongest-family aggregation remains unchanged.

The public timer is the maximum valid source lifetime: at least one effect of that family remains, without claiming that the strongest magnitude lasts that long. Actual current stats continue using existing strongest-source rules. Core verifies stronger-shorter/weaker-longer, independent expiration, refresh/removal, foreign entity and malformed reserved data; existing real summoner ECS test verifies actual cast plus item source status and expiry. Actual Rust store->canonical codec->filtered replica->ABI export supplies new native HUD identity/countdown/foreign owner/old generation/clear/retirement and full unsigned64 key-preservation assertions. Existing shared localized hero-details renderer displays both statuses; no dedicated new icon/VFX or magnitude timer is claimed.

Fresh full8scope RustV1 1867passed/0failed/9existingignored (ABI15/content47/templates118/codegen73/core987/base332/bridge138/client157). Formal backend/UE5.8 BuildV3/staged DLL/import SHA passed. FullWorldV3 executed83:82success+1existingwarningcase,0failed/0notrun,warnings4. Source guard verified3448 inputs;3433 prior inputs unchanged+7 exact formal inputs-v4+8 explicit generated deltas. Archive contains529 independently reverified files and52 accepted PNGs;8 changed/new images actually viewed,44 prior SHA-identical images inherited. Health13 samples/max171seconds. Seven final and five historical workers (3successful/2failed) retired; all launchers/readers and single170second health monitor reaped; final CIM found no owned build/test/source worker remaining.

Before native builds, audit identified actual movement visual key0x9ca5c927d356ae1a exceeds int64. FullRust and concurrent fullsource snapshot readers retired/reaped before correcting only native fixture to uint64 parsing plus bitwise preservation assertions. Original6inputs-v1 and fullRust source snapshot retained;5Rust/Lua inputs unchanged. Final source comparison proves all later differences from that fullRust snapshot limited to the two native C++ files and explicit generated outputs; no Rust rerun after C++-only change claimed or needed. Post-UEBuildV1 audit found late valid Buff snapshots could repopulate retired UI cache. Successful original build/stage/6inputs-v2 plus original widget SHA bound to fullRust snapshot retained. After reader retirement sanitized the shared retired setter to empty, added late active publication regression, froze7inputs-v3 and rebuilt UEV2/fullWorldV2/sourceV2. FullWorldV2 failed only the new fixture because a C ABI POD row from AddDefaulted was not value-initialized; all original7inputs-v3/report/52PNG and failed dependency receipts preserved. All readers retired before fixing only fixture initialization, freezing7inputs-v4 and rebuilding/fullWorldV3/sourceV3. No failed runtime accepted. Read-only path/count diagnostic errors and native field audit recorded with prevention in docs/plans/2026-10-23-modifier-status-errors.md. Evidence:docs/acceptance/2026-10-23-modifier-status/index.json. Final baseline for future batches is omfue/Saved/BuildTemp/modifier-status-source-before-v3.json, final cohort inputs-v4; rust-source-before-v1 is the earlier Rust proof only. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap, full combat replica and11 external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## Visible owned status summary IN PROGRESS

OpenSpec add-owned-status-summary2/4. Existing complete-nameplate-screen-spacing4/4 already done. New summary shares validated localized/stack/permanent/sub-tenth buff formatter, max2bounded24char names+morecount, full shared tooltip; currentaliveplayingMOBA identity/gen/chrome/retirement gated. Existing header budget/max360/scale/ellipsis, eventcached.4native frozen inputs-v1 (Widget.h/cpp,newOwnedStatusSummaryTests.cpp,existingTimedModifierStatusTests.cpp actualRustABIassert). Prepared fixedLua build/stage/fullWorld84/sourcecapture3449/source/inheritance3445unchanged+4native/priorfreshRust1867/backend/visual53/seal/close helpers. BuildV1 launcher1866 active, compiler CPUprogress confirmed; single170sec health50318active. Need build retire/reap, stage, sourcecapture/world/source, actual changed/new visual review+boundedreviewhelper, allreadersretire/done/reaphealth/CIMnone/seal/close-docs/tasks. Finalparent modifier-status/index.json + modifier-status-source-before-v3.json. Do not edit frozenformal sources duringreaders or duplicatejobs. Errors docs/plans/2026-10-23-owned-status-errors.md. No realmatches/quotareset/commit/push/omfx; deadlineunchanged.


## Visible owned status summary COMPLETE

OpenSpec add-owned-status-summary4/4.

Added a visible read-only owned status summary to the existing header flow. Reuses the exact shared buff validation/localized naming/stack counts/permanent and sub-tenth positive lifetime formatter. At most two24-character sanitized effect names plus explicit remaining-count line; the shared hover tooltip retains the full<=64 accepted rows. Existing diagnostic getters retain compatibility. Cached event-driven formatting only, no added per-entity/frame loop, new wire field, authority command or local timer. Existing header width budget/max360, text scaling and ellipsis bound the row.

The summary and tooltip require current alive/playing MOBA hero context, matching entity and generation, visible HUD and active scope. Empty/expired/malformed/foreign lists clear it; dead/end/TD/waiting/new-generation/retirement hide it and late publication cannot revive retirement. Existing actual Rust filtered ABI fixture confirms real movement_boost/damage_reduction rows reach the visible summary. New mounted Slate fixture checks pre-arriving/no-context state, two-row+overflow, full tooltip/stack/permanence/sub-tenth, private-key exclusion, generation/dead/end/mode/expiry/invalid/long sanitized name and retirement, and captures actual summary plus full tooltip. No real network match or authority ACK claimed.

UE5.8 BuildV1/staged DLL/import SHA passed. FullWorldV1 executed84:83success+1existingwarningcase,0failed/0notrun,warnings4. Full source guard3449 inputs,3445unchanged+4exactnative inputs-v1; prior fresh8scope Rust1867passed/0failed/9ignored and backend inherited by source-bound proof, no freshRust rerun claimed. Archive119 independently verified files and53accepted PNGs;12changed/new actually viewed,41priorSHAidentical inherited. Health5samples/max170seconds. Four owned workers retired; all launchers/source readers and single170second health monitor reaped, final CIM found no owned worker remaining.

Evidence:docs/acceptance/2026-10-23-owned-status/index.json. Errors/prevention:docs/plans/2026-10-23-owned-status-errors.md. Final source baseline omfue/Saved/BuildTemp/owned-status-source-before-v1.json, frozen4inputs-v1. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap, full combat replica and11external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Original unattended deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## Summoner self guard IN PROGRESS

OpenSpec add-summoner-self-guard2/4. Added typed shared SelfDamageReduction operation to strict author/generated/model/planner/unsupported-defaultsink/ECS grant; catalog6堅守25%3sec/CD180, same item strongest-family independentlifetime/canonicaldamage_reduction/publicsummary, no native name rules.9formal inputs-v1 initially frozen. RustV1 passedABI/content/templates/codegen then corecompileE0425 missingtest-localBuffStoreimport in atomicrejectionfixture. Failed9inputs/rawlogs/receipt preserved;launcher92960 reapedfailed. Afterreadersretired fixed only testimport;9inputs-v2 has8unchanged. FreshRustV2 launcher43737 active:4scopes+corepassed, base_content releasecompiling (lastclock13:33:53UTC); single170sec health31747active. Need full8Rust1873/9ignored (ABI15/content48/templates118/codegen74/core990/base333/bridge138/client157 planned), backendV1/buildV1/stage, capture3450/world85/source,54PNG actualreview, seal6final+1historicalRustfail, retire/done/reaphealth/CIMnone/close/tasks. Saved runners prefix summoner-guard-, prepare/prepare-acceptance/test-repair/seal/close-docs ready; source-v1 waitsRustV2. New UI native OmSummonerGuardTests.cpp sharesactualgeneratedidentity/genericD/F selfcard and tooltip capture. Parentowned-status/index.json/source-before-v1. Sealer explicitgeneratedallowlist includesSummonerCatalog.cpp. No moreformal editswhile readers. ErrorMD docs/plans/2026-10-23-summoner-guard-errors.md. No duplicatejobs/realnetwork/quotareset/commit/push/omfx. Deadlineunchanged.


### Self guard fullRust/backend passed; UE building

FreshRustV2 full8scopes1873passed/0failed/9ignored confirmed15/48/118/74/990/333/138/157, launcher43737 retired/reaped. Formal60Hz realdamage160->120->140->160/pause/owner cooldown passes. BackendV1 success/launcher29686reaped. UEbuildV1 now active (see currentlauncher session tool result); singlehealth31747continues. All9inputs-v2 frozen; prior9inputs-v1/RustV1compilefail preserved. Need UE/stage/sourcecapture/world85/source3450/visual54actualreview/seal6final+1failedhistory/retire/done/reap/close. Saved sealer/close-docs ready; sealer parses freshRust and validates9formal+explicitgenerated against owned-status parent. No further source mutation while readers. Deadline unchanged.


### Self guard native test-access repair

UEBuildV1 failed C2248 newGuardfixture calling private tooltip without matching WITH_DEV_AUTOMATION_TESTS friend; launcher62195 retired/reaped. Only nativeh testfriend added afterallreadersretired.10inputs-v3=all9RustV2 sources unchanged+nativeheader friend; originalheader SHA bound owned-status baseline/rawparentartifact. FreshRust1873 remains valid, backendV1 passed. UEbuildV2 launcher7921active, singlehealth31747continues. FailedRustV1/9inputs-v1 and failedUEV1/9inputs-v2 retained; finalsealer now6final+2failedhistory,10formal+explicitgenerated/source3450/native85/54PNG. Sealer validates parentheader rawSHA and9unchangedRustcohort. Need stage-v2 afterbuild,reap; capture/world/source-v1 waitbuild-v2,rust-v2; actualvisualreview; retirement/done/healthreap/seal/close-docs/tasks. No duplicatejobs or formal edits while frozen. Deadlineunchanged.


## Summoner self guard COMPLETE

OpenSpec add-summoner-self-guard4/4.

Added shared closed SelfDamageReduction author/runtime operation and append-only catalog6 堅守 (25%/3seconds/CD180). Existing loadouts are retained until explicitly equipped. Strict positive quantized reduction<=1, lifetime<=60, no-target shape and existing alive/context/owner/pause/cooldown plan/commit rules apply. Unsupported adapter or absent ECS BuffStore rejects without cooldown. Source-keyed grant uses the existing item_damage_reduce family, strongest value and independent source expiry. No hero/spell-name parsing or native special rules. No healing/shield/cleanse/immunity, original movement/control state retained.

Existing canonical damage_reduction status/details/visible summary receive the effect and represent any valid remaining source lifetime. Author/model/codegen/catalog, actual ECS control/command preservation/public time and atomic rejection tests plus formal60Hz real packet damage160->120->140->160 with weaker longer item, pause and owner-team cooldown disclosure pass. New native actual generated catalog/description/readyD/F/self-shape/cooldown/retirement fixture captures shared guidance. No true match or authority ACK claim.

Fresh full8scope RustV2 1873passed/0failed/9existingignored. Formal backend and UE5.8 BuildV2/staged DLL/import SHA passed. FullWorldV1 executed85:84success+1existingwarningcase,0failed/0notrun,warnings4. Source guard3450inputs;3437unchanged+10exactformal inputs-v3+3explicitgenerated deltas. Archive218independently verified files and54accepted PNGs;12changed/new actually viewed,42priorSHAidentical inherited. Health10samples/max171seconds. Six final workers and two historical failed workers (RustV1/buildV1) retired; every launcher/source reader and single170second monitor reaped, final CIM no owned worker remaining.

RustV1 passed4scopes then failed core compile E0425 in a new test because BuffStore use was scoped to a sibling fixture. All readers retired/reaped before adding only the missing test-local import. Original9inputs-v1/rawfailed logs/retired receipt preserved; fresh9inputs-v2 has8unchanged+1test-only change and full8RustV2 accepted. UEBuildV1 failed new native fixture private tooltip access because its WITH_DEV_AUTOMATION_TESTS friend was missing. After all readers retired added only that friend;10inputs-v3 retains all9RustV2 sources byte-identical, original parent header and failedbuild retained. FreshUEV2/native/source required and passed before final acceptance. Pre-reader field/match/float and saved helper errors with prevention retained in docs/plans/2026-10-23-summoner-guard-errors.md. Evidence:docs/acceptance/2026-10-23-summoner-guard/index.json. Final source baseline omfue/Saved/BuildTemp/summoner-guard-source-before-v1.json and10inputs-v3. All readers retired/source freeze released.

Further gameplay/content polish, hero-to-hero overlap, full combat replica and11external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## Hero nameplate placement ACCEPTANCE IN PROGRESS

OpenSpec improve-hero-nameplate-placement2/4.6native production/test sources implement bounded bitgrid placement within144logicalpixels, fixed1Mbits/2Mcell visits, stable allhero sorting/self-focuspriority, actorpivot/current measuredsize/worldanchorunchanged/lifecycle reset and finalexactnonhero reservations. New native actual2hero component/widget/bounds + helperedge/scales/order/10000budget passes. Successful BuildV1/stage saved, pre-nativecaptioncenter audit changedonlyfixtureheading;6inputs-v2 has5productionunchanged. BuildV2/stagepassed/reaped52403. FullWorldV1 passed86(85success+1existingwarncase,4warnings), launcher42017reaped.56PNG reviewed13changed/new+43SHAidentical, includingactualstacked/recovered. Whole-map capture stilldenseoverlap; nofullmapseparationclaim, preserveprojection/disclosuregates anddensefallback. CaptureV1 passed3452/reaped81848. SourceV1 launcher12653activefullhashcheck/inherit3446unchanged+6native/priorRust1873/backend; single170sec health67140active. Need sourcepass/reap,done/reaphealth/CIMnone/seal-v1/reap/close-docs/tasks. Sealer4final+1historicalsuccessfulbuild, original6inputs-v1+stage retained, final6inputs-v2. Finalbaselineplannedhero-placement-source-before-v1.json,parentSummonerGuardindex/sourcebeforev1. No duplicatejobs or frozenformal edits. Deadlineunchanged.


## Bounded hero nameplate placement COMPLETE

OpenSpec improve-hero-nameplate-placement4/4.

Current validated projected visible heroes now use stable self/selected/hover/other, center-distance and identity priority for nearby vertical nameplate placement. A fresh conservative occupancy bit grid attempts at most37positions within144logical pixels. Grid memory<=1048576bits and cell visits<=2000000 per view; no64hero cutoff and no quadratic hero rectangle scan. All heroes remain visible; invalid/crowded/exhausted placement retains original anchor. SortingO(HlogH), existing nonhero overlap workO(H*64), plus fixed grid work; this is not a10000live-rendered performance benchmark.

Actor placement uses current measured logical layout and component pivot, matching UE5.8 screen-layer SConstraintCanvas alignment. World actor/mesh/nameplate anchors and target input remain unchanged. Actor validates offset and size; call-site reserves only actual applied footprints for existing64nonhero admission. Concealment/render-collapse/source revocation/retirement/new-life/unprojected state resets pivot. Existing public-frame cadence retained, with no extra entity-per-frame pass or local time.

New native test verifies padded separation, exact moved nonhero footprint, edge/down placement, four geometry scales, stable ordering/recovery, invalid input,10000dense helper work budget and fixed grid storage. Two actual actor/widget/component pivots paint in a real Slate window; measured desktop bounds do not overlap, world anchor is unchanged, and separated actors return to original alignment. Hide/revoke/retire/lateplacement/newgeneration assertions pass. Two actual captures establish placement/recovery. The overall compiled map image still contains dense label overlaps; this batch does not establish full-map or true-network separation. Dense fallback and broader projection/disclosure/integration scenarios remain work to investigate, without weakening existing visibility gates.

UE5.8 BuildV2/staged DLL/import SHA passed. FullWorldV1 executed86:85success+1existingwarningcase,0failed/0notrun,warnings4. Full source guard3452 inputs;3446unchanged+6exactnative inputs-v2 prove inheritance of priorfresh8scope Rust1873passed/0failed/9ignored and backend. No freshRust/backend rerun claimed. Archive140independently verified files and56accepted PNGs;13changed/new actually viewed,43priorSHAidentical inherited. Health5samples/max171seconds. Four final workers and one historical successful build retired, all launchers/source readers and single170second monitor reaped; final CIM no owned worker remaining.

After successful UEBuildV1/stage, pre-native engine review caught default centered Canvas capture heading. All readers retired before correcting only fixture heading alignment; original6inputs-v1/successfulbuild/stage preserved,6inputs-v2 retains5production sources unchanged. No failed runtime accepted. Errors/prevention:docs/plans/2026-10-23-hero-placement-errors.md. Evidence:docs/acceptance/2026-10-23-hero-placement/index.json. Final baseline omfue/Saved/BuildTemp/hero-placement-source-before-v1.json,6inputs-v2. All readers retired/source freeze released.

Further gameplay/content polish, dense hero-label/fallback and whole-map integration, full combat replica and11external tasks/MVP33/44 remain pending. No real network matches/quota reset/commit/push/omfx maintenance. Deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## Projected map nameplate integration COMPLETE

OpenSpec complete-projected-map-nameplate-integration4/4.

The compiled-map fixture previously published its initial public frames before the actual local viewport/HUD and final overview camera existed; later window paints alone did not refresh frame-driven screen admission. It now publishes a next formal synthetic public frame after final overview and combat camera fitting, consistently advancing animation/frame sequences before any transient combat events. No per-paint event replay, production disclosure override or assumed player identity was added.

The overview now checks actual cached desktop bounds for every admitted nonhero against all other admitted labels, preserves all six visible hero labels and requires nonempty <=64 nonhero admission. Hero-to-hero finite-space fallback remains permitted by the shared production design. This is integrated synthetic public rendering, not a real match or authority ACK.

UE5.8 BuildV3/staged DLL and import SHA passed. Full native86:85success+1existingwarningcase,0failed/0notrun,warnings4. Fullsource3452 verifies3451 unchanged+1native fixture input; prior fresh8scope Rust1873/0failed/9ignored and backend inherited by source-bound proof. No freshfullRust/backend rerun claimed. Archive360 independently verified files/57PNG, 12changed/additional actually viewed and45SHAidentical inherited. Health7samples/max171seconds. Four final and four historical workers (two successful, one failed native suite, one source pipeline with failed inheritance) retired; all launchers/readers and health monitor reaped, final CIM no owned worker remaining.

Evidence:docs/acceptance/2026-10-23-projected-map/index.json. Errors/prevention:docs/plans/2026-10-23-projected-map-errors.md. Final baseline omfue/Saved/BuildTemp/projected-map-source-before-v2.json, one frozen input inputs-v2. All readers retired/source freeze released. Further concrete gameplay polish, dense hero fallback, full combat replica and11 external tasks/MVP33/44 remain pending. No true network match, quota reset, commit/push or omfx maintenance. Original deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.

Next concrete reliability work: replace legacy Saved wall-guidance-launch.lua stdout substring success detection with a reusable formal fixed-Lua acceptance launcher using existing identity-bound owned-run exit receipts (tools/lua/lib/process.lua run_owned_to_files / poll_owned_exit; scripts/run_owned_session_process.lua and scripts/tests/process_owned_run_test.lua are discovered existing entry points). The observed sourceV2 subprocess actually failed and process.run correctly rejected it, but legacy launcher marked receipt success after seeing an intermediate exit=0. Final SourceV3 used explicit successful inherited-v3 artifact and sealer independently validated all actual source/native/stage proofs plus stderr, so this batch is accepted. Preserve the false-positive historical receipt. Regression must cover early-success output followed by late failure and mismatched process identity, before switching future batch launchers. No running workers remain; do not rerun completed projected-map acceptance or change frozen archived helpers.


## Shared actual-exit acceptance COMPLETE

OpenSpec fix-owned-acceptance-exit-status4/4.

Shared scripts/owned_acceptance.lua and scripts/run_owned_acceptance.lua now reuse existing moba_session_process/identity-bound owned-run Windows exit receipts. A nonzero actual exit fails even after intermediate exit=0 or a successful summary. Optional summary must be fresh/bounded/successful; exit-only tasks use actual exit. Exclusive outputs preserve identities/spawn/ready/exit/stdout/stderr/receipt/health/errors. Only recorded original lifetimes are cleaned up. Direct runner source SHA is bound before/after; whole-source batch verification remains separate. Built-in waiting samples current explicit logs at<=170seconds during normal waits. Archived legacy helpers unchanged.

Actual regression:37new acceptance assertions plus50existing platform assertions; three real formal CLI cases fail1/missing-summary1/pass0. Covers successful-summary-then-error, zero-without-required-summary, success, exit-only, stale evidence and PID/token/executable/retirement mismatch. Expected negative failures preserve original logs/receipts and are not runtime failures accepted as success. Consolidated CLI itself used the new formal launcher. Three final workers exited0 with identity-bound raw receipts and retired children/waiters; all CLI launcher sessions reaped and final CIM no owned worker.

Fullsource3456 checked:3452 prior inputs byte-identical+4exactadded shared Lua files, frozen inputs-v1. Prior UE5.8/native86(0failed,4existingwarnings), fresh8Rust1873/0failed/9ignored and57viewedPNG inherited by source-bound proof; no engine/native/fullRust rerun or new visual claim. Archive215 independently reverified files. Evidence:docs/acceptance/2026-10-23-owned-acceptance/index.json. Usage:docs/plans/2026-10-23-owned-acceptance-workflow.md. Errors/root/prevention:docs/plans/2026-10-23-owned-acceptance-errors.md.

Future batches MUST use tools/lua/lua.exe scripts/run_owned_acceptance.lua <spec.json>, with fresh exclusive output and optional fresh result.json; never infer final success from intermediate log strings or use the historical Saved wall-guidance-launch.lua for new jobs. Final baseline omfue/Saved/BuildTemp/owned-acceptance-source-before-v1.json; four frozen inputs-v1. All readers retired/source freeze released. complete-nameplate-screen-spacing and projected-map integration already4/4; resume concrete single-PC gameplay/skill/presentation gaps next, not repeated completed verification. Full combat replica/11external tasks/MVP33/44 remain pending. No true network game/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei and automationomoba-48 unchanged.


## Active countdown readability COMPLETE

OpenSpec complete-active-countdown-readability4/4.

Shared native FOmDurationText preserves positive sub-tenth authority seconds as<0.1, exact zero as0, invalid/negative as unavailable, normal countdown decimal precision and optional compact integral trimming. Cast-preview stun/silence/root/item-aim cooldown plus recall header/status countdowns now share it; existing item/summoner cooldown helper delegates without changing ordinary precision. Permanent control is handled before generic negative validation and retains no invented timer. No local tick, gameplay/command/admission changes or hero-name rules. Original successful V1 build/full87report/source and original cropped capture retained; after all readers retired only new fixture window height/footer containment changed.5inputs-v2 retains4other inputs unchanged; fresh BuildV2/fullWorldV2/fullsourceV2 plus explicit proofV3 accepted. FullsourceV2 hash passed but parent binding failed from broad version replacement; new launcher correctly returned exit1/success=false and preserved original evidence. After reader retirement, explicit immutable parent path proofV3 reused the successful full hash component and compared every snapshot/current frozen input. No formal source changed; no redundant full hash/native/Rust rerun. Failed inheritance remains historical only.

Existing real public bridge/control fixture now verifies0.01second stun remains active with exact authority time and positive display. New native boundary/normal/invalid/permanent/control/root/item tests mount actual recall Slate0.01second, capture four shared guidance rows plus real recall strip, and verify zero/retirement/late publication.

UE5.8 BuildV2 and staged DLL/import SHA passed. Full native87:86success+1existingwarningcase,0failed/0notrun,warnings4. Fullsource3458 verifies3453 unchanged+5exactnative inputs-v2 (three existing plus two new); priorfresh8Rust1873/0failed/9ignored and backend inherited by bound source proof, no freshfullRust/backend rerun claimed. Archive489independently verified files/58PNG; 13changed/new actually viewed and45SHAidentical inherited. Four final and three historical successful actual-exit CLI workers have exact Windows exit0; one additional historical failed inheritance worker has exact exit1/success=false. All original child/waiter lifetimes retired; health max151531ms; all launcher sessions reaped, final CIM no owned workers.

Evidence:docs/acceptance/2026-10-23-duration-readability/index.json. Errors/prevention:docs/plans/2026-10-23-duration-readability-errors.md. Final baseline omfue/Saved/BuildTemp/duration-readability-source-before-v2.json,5frozen inputs-v2. All readers retired/source freeze released. Continue concrete single-PC gameplay/skill/presentation gaps; full combat replica/11external/MVP33/44 remain. No true network game/quota reset/commit/push/omfx. Shared actual-exit launcher remains mandatory for new jobs. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.


## Profile-aware world cast instructions COMPLETE

OpenSpec complete-profile-aware-cast-instructions4/4.

World aiming footer now reads current active WorldCombat/Pressed primary/secondary PointerConfirm, EscapePriority and CancelAim chords, including actual modifiers and localized engine key labels. Intrinsic right-click cancel retained. QuickWithIndicator names actual held key, or generic release if token unknown; missing confirmation explicitly unavailable. Inactive keyboard-movement cancellation is not advertised, and current profile changes are reflected without a cached string. Shared helper applies to the common ability/summoner/item/ward overlay, retaining existing wrap/text scale and scope/lifecycle. No command, profile persistence or input gate changes.

Native fixture verifies default/rebound primary+secondary/modifiers/Escape/missing/inactive/held-release/fresh-help behavior and real owner MatchesPointerConfirm acceptance/rejection/retirement. Actual Slate rows show default/custom/indicator/missing/keyboard movement instructions with full bottom containment assertion; original pixels reviewed. Test-only WITH_DEV friend supports actual owner checks.

UE5.8 BuildV1 and staged DLL/import SHA passed. Full native88:87success+1existingwarningcase,0failed/0notrun,warnings4. Fullsource3460:3456 unchanged+4exactnative inputs-v1 (two existing plus two new), inherits priorfresh8Rust1873/0failed/9ignored and backend by bound source proof; no freshfullRust/backend rerun claimed. Archive149independent files/59PNG; 13changed/new actually viewed and46SHAidentical inherited. Four actual-exit CLI workers exited0 with retired original child/waiter receipts; health max151391ms, all sessions reaped, final CIM no owned worker.

Batch uses immutable explicit configuration: parent index-bound baseline preflight before build/readers, frozen config SHA checked by source consumers/sealer, no broad ancestor version rewriting. Evidence:docs/acceptance/2026-10-23-cast-instruction/index.json. Errors/prevention:docs/plans/2026-10-23-cast-instruction-errors.md. Final baseline omfue/Saved/BuildTemp/cast-instruction-source-before-v1.json,4frozen native inputs/config-v1. All readers retired/source freeze released. Continue concrete single-PC gameplay/skill/presentation gaps; full combat replica/11external/MVP33/44 remain. No true network game/quota reset/commit/push/omfx. New actual-exit launcher required. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.


## Complete ability tooltip bindings COMPLETE

OpenSpec complete-ability-tooltip-bindings4/4.

Ability QWER tooltip now lists active primary/secondary cast chords with localized modifier labels. Normal shortcut and every icon-click aiming route reuse shared actual WorldCombat confirmation/cancel help. Quick and indicator shortcut behavior remains distinct, while icon clicks correctly describe forced normal aiming. Missing shortcuts retain complete icon confirmation/cancel instructions; no-target casts remain immediate. Existing tooltip lifecycle/profile gates and world helper are reused without new command paths, local ticking, hero-name special cases or gameplay mutation.

Extended native regression validates secondary modifier cast bindings, actual shared normal confirmation, active/inactive/missing bindings, unbound shortcut icon route, immediate no-target mode, and quick/indicator icon routes via validated profile APIs. Shared Slate rows render full shortcut/icon and unbound instructions with actual bottom containment. This captures production guidance strings, not a live-match input/authority ACK.

UE5.8 BuildV1/staged DLL/import SHA passed. Full native88:87success+1existingwarningcase,0failed/0notrun,warnings4. Fullsource3460:3458unchanged+2exactexisting native inputs-v1. Priorfresh8Rust1873/0failed/9ignored and backend inherited by immutable parent/source proof; no freshfullRust/backend rerun claimed. Archive147independent files/59PNG; 19changed images actually viewed and40SHAidentical inherited. Four actual-exit CLI workers exited0 with retired child/waiter receipts; health max152297ms. All launcher sessions reaped; final owned-process check recorded.

Evidence:docs/acceptance/2026-10-23-tooltip-guidance/index.json. Errors/prevention:docs/plans/2026-10-23-tooltip-guidance-errors.md. Final baseline omfue/Saved/BuildTemp/tooltip-guidance-source-before-v1.json,2frozen native inputs/config-v1. All readers retired/source freeze released. Continue concrete single-PC gameplay and presentation gaps; full combat replica/11external/MVP33/44 remain. No true network game/quota reset/commit/push/omfx. New actual-exit launcher required. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.


## Actual aim/communication paint layers COMPLETE

OpenSpec fix-aim-overlay-layer-order4/4.

Common production aiming/communication overlay now propagates the highest actually used paint layer. Communication center text contributes base+5, cast guidance base+2 and actually drawn attack cursor base+1; early returns retain any prior drawing maximum. Missing owner/inactive/tiny communication rendering returns base. SOmAttackMoveAimOverlay returns this actual result, replacing its previous constant base+1. This honors Slate sibling ordering without arbitrary fixed reservations, draw-element scans, new entity loops, command/input/visibility changes or global z-order changes.

New native regression mounts real production overlays through an automation-only factory. Probe measures actual newly emitted text and the real child paint return, accounting for SCompoundWidget child+1. Real ping and emote model gestures both establish actual text at base+5; inactive/retired owner consumes no extra overlay layers. Two ping wheels render side-by-side: left visible, right covered by a later opaque blue sibling. Every interior right-pane pixel is checked blue, and the original PNG was inspected. Fixture origin0 uses existing production edge clamping; this establishes compositing, not real pointer coordinate/input routing or network ACK. Test-only volatility makes direct fixture state changes paint fresh.

V1 full native failed the new emote fixture arm because it supplied a forbidden world point, followed by a dependent paint assertion. Original full89 failure report/PNG, successful build/stage/capture,4inputs-v1 and exact Windows exit1 retained. After all original readers retired/reaped, only fixture point-kind input changed;3production files stayed byte-identical. V2 fresh build/full89native/fullsource passed, but actual visual review rejected its center-only pixel sample: blue panel did not overlap wheel center. Original V2 successful reports, rejected original PNG,4inputs-v2 and all4 actual exit0 workers retained. After all V2 readers retired/reaped, fixture changed to full-pane coverage and visible reference wheel. V3 frozen4inputs-v3 still retains all3production files byte-identical toV1/V2. No production input validation weakened and no rejected pixels accepted. Errors/root/prevention recorded in docs/plans/2026-10-23-aim-layer-errors.md.

UE5.8 BuildV3/staged DLL/import SHA passed. Final FullWorldV3 native89:88success+1existingwarningcase,0failed/0notrun,warnings4. Fullsource3461:3457unchanged+4exactnative inputs-v3 (three existing plus one new). Priorfresh8Rust1873/0failed/9ignored and backend inherited by immutable parent/source proof; no freshfullRust/backend rerun claimed. Archive818independent files/60acceptedPNG; 14changed/new actually viewed and46SHAidentical inherited. Four final actual-exit workers exited0. Seven historical workers include one actual failed native exit1 and six actual exit0; all original child/waiter lifetimes retired, all launcher sessions and sealer reaped. Health max151484ms; final owned-process check recorded.

Evidence:docs/acceptance/2026-10-23-aim-layer/index.json. Final baseline omfue/Saved/BuildTemp/aim-layer-source-before-v3.json,4frozen native inputs/config-v3. All readers retired/source freeze released. Continue concrete single-PC gameplay and presentation gaps; full combat replica/11external/MVP33/44 remain. No true network game/quota reset/commit/push/omfx. Actual-exit launcher required. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.

V3 startup has a separately preserved engine Dataflow Date member-initialization diagnostic (2entries before selected tests). It was absent in4compared prior logs; exact local engine header initializes reflected Date with wall-clock FDateTime::Now(), a source-based explanation for the non-deterministic initialization comparison. Exact header/SHA, raw diagnostic and comparison retained under engine-diagnostic; no engine edits/suppression. Final native89 still has0failed/0notrun/4warnings; no claim of diagnostic-free engine startup.


Next concrete batch: investigate actual communication wheel pointer-to-paint coordinates at nonzero window positions and logical scales, through the real owned viewport route. V2 cached-root-origin fixture mismatch is an observation, not proof of a gameplay defect; V3 origin0/compositing acceptance does not resolve that question. See docs/plans/2026-10-24-communication-coordinate-followup.md. Use final aim-layer source-before-v3.json and index-bound parent proof, do not repeat accepted layer work. No current workers remain.


## Shared desktop-to-paint coordinates COMPLETE

OpenSpec fix-desktop-to-paint-coordinates4/4.

Confirmed engine-space cause of prior nonzero-window mismatch: SWidget caches desktop geometry after appending current FPaintArgs window translation, while OnPaint allotted geometry remains window-relative. Shared checked FOmSlatePaintSpace now subtracts the current paint window translation before inverse geometry, rejecting nonfinite/singular inputs and clearing output on failure. Current offset propagates through production overlay to communication origin, attack range/cursor, cast hint anchor, world outlines, marker and source guide. No fixed offset/DPI guess, cached prior window location, command/input/visibility/selection change, new per-entity loop or hero-name rule. Existing actual maximum paint-layer propagation is preserved.

Native geometry round trips cover4layout scales and signed window translations; legacy direct inverse disagrees. Actual production overlay is mounted at2nonzero moved native-window positions and3render scales. FPointerEvent screen origin arms the real gesture model; probe compares actual emitted center text render transform plus actual current paint window offset against that desktop origin within1pixel. Invalid coordinate/singular layout and retirement cases covered; two original window captures inspected. This establishes event coordinate convention and production presentation, not successful live owner-context admission, true network ACK or physical OS DPI/controller acceptance. Existing ownership/input tests are included in full native suite.

UE5.8 BuildV1/staged DLL/import SHA passed. Full native90:89success+1existingwarningcase,0failed/0notrun,warnings4. Fullsource3463:3458unchanged+5exactnative inputs-v1 (three existing plus two new). Priorfresh8Rust1873/0failed/9ignored and backend inherited by immutable parent/source proof; no freshfullRust/backend rerun claimed. Archive158independent files/62acceptedPNG; 15changed/new actually viewed and47SHAidentical inherited. Four final actual-exit workers exited0 with retired original child/waiter receipts; all launcher sessions/sealer reaped. Health max151719ms; final owned-process check recorded.

Evidence:docs/acceptance/2026-10-24-paint-space/index.json. Errors/root/prevention:docs/plans/2026-10-24-paint-space-errors.md. Final baseline omfue/Saved/BuildTemp/paint-space-source-before-v1.json,5frozen native inputs/config-v1. All readers retired/source freeze released. Prior communication coordinate presentation question is resolved by actual draw measurements; full owned admission and hardware/true-network requirements remain distinct. Continue concrete single-PC gameplay/presentation gaps; full combat replica/11external/MVP33/44 remain. No true network game/quota reset/commit/push/omfx. Actual-exit launcher required. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.


## Active directional cone IN PROGRESS

OpenSpec complete-active-directional-cone2/4. Shared ConeDamage typed/Lua/generic execution and append-only ranger_scatter/training_scatter_ranger; original volley/ranger unchanged. Published shared Q10/4096turn half-angle ticks reach common native source-sector outline/missing-direction feedback/tooltip/retirement. No hero-name branch/false circle cue. FreshRustV2 passed all8scopes1877/0failed/9ignored (15/49/119/74/990/335/138/157); Lua11/11. Actual formal60Hz forward110damage/reverseempty/zero+overrange rejection/manaCD/pause and boundedcanonicaldedup tests pass. FailedGenV1offline/GenV2badpattern/RustV1f32pi2boundary preserved; successfulGenV3. Every original receipt retired and launcher reaped. Errors docs/plans/2026-10-24-active-cone-errors.md.

20authored-v3 frozen; current36inputs-v1/config-v1 against immutablepaint-space parent; whole3467source baseline active-cone-source-before-v2.json,3431same+36changed. Allformal sources remain frozen. BackendV1 active tool session93456 (check receipt/CIM before duplicate); helpers prefix omfue/Saved/BuildTemp/active-cone-. Need reapbackend -> UEbuildV1/stagedSHA -> worldV1 full91native/63PNG and verifyV1 whole-source (canparallel afterbuild), actualview allchangedPNG/review-notes-v1.json/review-v1/seal-v1/reap/ownedretirement/close-docs. Preparedactual-exit specs/runners/config and sealer/close-docs ready, except reviewnotes must follow actualviews. Sealer includes9originalworkers (3fail+6success) and fresh8Rust/backend; do notinheritRust. Run scripts/run_owned_acceptance.lua forbuild/world/verify. No true network game/quotareset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei unchanged.


## Active directional cone COMPLETE

OpenSpec complete-active-directional-cone4/4.

Added closed data-driven ConeDamage: one exclusive instant active/ultimate point effect with per-rank range/damage/half-angle. Q10 half-angle conversion uses the existing shared4096-turn sector predicate. A distinct in-range aim point supplies direction; sector reaches full rank range. Living enemy candidates are canonical(id,generation) sorted/deduplicated, geometry-filtered, then admitted through bounded atomic damage settlement. Invalid direction/range/data or >128resolved hits rejects before commit. Existing damage attribution/authority remain; no false circular area cue.

Append-only ranger_scatter and optional training_scatter_ranger loadout reuse existing ranger visual data. Original training_ranger and ranger_volley circle stay unchanged. No hero-name execution branch. Common generated rank ticks flow through typed HUD payload to owner cache, sector outline, distinct missing-direction instruction and actual skill tooltip. Unknown/replaced/retired states clear geometry. Native outline is read-only floating-point aiming guidance; authoritative fixed-point predicate decides exact hits. No terrain/projectile blocking promise, live-match owner-admission or network ACK claim.

Coverage: strict shared model and Lua11case boundaries (including correctedf32 pi/2 limit), append-only IDs/loadout, all-rank canonical target/dedup/sector radial-angular boundary/overflow/zero-direction/overrange tests. Formal60Hz actual cast verifies forward hit110, reverse empty cast, zero/overrange rejection, resources/cooldown and pause. Native generated rank geometry/owner state/retirement/tooltip regressions plus actual Slate tooltip capture. Hero lifecycle matrix now resolves every generated native class and typed metadata automatically:8heroes/30active+ultimate casts actually exercise real clips/pooledVFX/dedup plus death/respawn/pause/concealment/generation lifecycle. No second hero/skill-name list.

Fresh8Rust 1877passed/0failed/9ignored; fresh formal backend and UE5.8 build/stagedDLL/import SHA passed. Fullnative 91:90success+1existingwarningcase,0failed/0notrun,warnings4. Fullsource 3468: 3429unchanged+39exactchanged inputs against index-bound paint-space parent. Rust/backend results are fresh, not inherited. After that cohort, only2presentation assets and1native lifecycle test changed; immutable full source proofs bind those repairs. Final baseline derives from the successfully verified asset snapshot plus exactly that one frozen native test edit, then passes the whole3468byte/size/mtime check. Archive 870independent files/63PNG; 14actually viewed and49SHAidentical inherited. All original job child/waiter receipts retain actual Windows exits and retirement; health max151921ms. Original failed generation/offline-retry and Lua boundary cohorts remain archived.

Evidence:docs/acceptance/2026-10-24-active-cone/index.json. Errors/root/prevention:docs/plans/2026-10-24-active-cone-errors.md. Finalbaseline omfue/Saved/BuildTemp/active-cone-source-before-v4.json. No remaining readers after final retirement; source freeze released. Full combat replica/11external/MVP33/44 remain. No true network game/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.


## Bot directional cone IN PROGRESS

OpenSpec complete-bot-directional-cone2/4. Shared typed compiled/runtime per-rank cone ticks and strict EnemyCone min_targets; bounded32aims/512disclosed observations/canonicaldedup/128hit preflight. Full source range and common fixed-point sector predicate, role/guardfocus/mana/CD/attack/control gates retained. Optional scripts/lua_data/moba_scatter_match.lua swapscarry loadouts only; no network match launched. Added5coretests+1runtimecatalogtest+1formal60Hz actualbotcast. Current sources FROZEN:7inputs-v1/config-v1, whole3470baseline bot-cone-source-before-v1.json versus immutable active-cone source-before-v4/index (3463same+7changed expected). Owned rust-v1 active, tool session72006; inspect receipt/progress/CIM before any duplicate. ABI/content/templates/codegen/core alreadyexit0, core995; base_content compiling release. Runner prefix omfue/Saved/BuildTemp/bot-cone-, jobs omfue/Saved/Acceptance/BotCone/. Prepared backend/build/staged/world(full91)/verify/recipe specs; run only after required prior exit0 and reaplauncher. All newjobs use scripts/run_owned_acceptance.lua. Need concentratedfresh8Rust/backend/UEstaged/native91/sourceverify/offline10playerrecipe, preserve failures, seal/archiveclosure and finaldocs. No omfx/newnetwork/quota/commit/push; deadlineunchanged.


# Bot directional cone COMPLETE

OpenSpec complete-bot-directional-cone4/4.

Shared compiled/runtime AbilityConst now exposes per-rank quantized cone half-angle ticks from the same validated aim geometry used by authoritative execution and native aiming guidance; runtime catalog reload rejects geometry drift. Closed EnemyCone {min_targets} requires actual executable instant active/ultimate point-cone metadata. No free-form angle/radius-key inference or hero-name rule.

The common planner uses committed disclosed living role-eligible opponents, canonical-ID deduplication, bounded32 enemy-center candidate axes and512 unique in-range observations. It scores the full source-centered sector with the authoritative fixed-point predicate, preserves disclosed guard/farm focus by coverage, then hit count, distance and canonical ID. Zero direction, unsupported coordinates, known >128-hit sectors or observation overflow are skipped. Existing author policy order, role restrictions, mana, rank, cooldown, attack deferral and control gates remain. This is a deterministic bounded enemy-center heuristic, not a claim of globally optimal continuous-angle aiming or prediction.

Optional scripts/lua_data/moba_scatter_match.lua derives from the existing mixed recipe, selects the scatter loadout for carry seats, adds explicit min2 cone policy and consecutive learning ranks. Existing default recipes remain. Fresh compiled moba-config --lock-plan admits all10players/2carry seats at60Hz without creating a World or network match.

Five focused core contracts cover strict schema/catalog matching, all-rank source range, radial/angular edges, dedup/permutation stability, disclosed focus and hidden-focus isolation, bounds/budgets, role/mana/root/attack gates. One runtime catalog test checks compiled/load/reload coherence. Formal60Hz actual bot input hits3stationary neutral targets at100/600/610distance for110each, charges45mana and starts cooldown; private uncommitted positions do not change aim and hidden cached units cannot satisfy min2.

Fresh8Rust 1884passed/0failed/9existing ignored (15/49/120/74/995/336/138/157); shared Lua cone11/11. Fresh backend build/stage and UE5.8 build/stagedDLL/import SHA passed. Fresh full native91: 90success+1existing warning case,0failed/0notrun. Fullsource3470:3463unchanged+7exact frozen inputs versus index-bound active-cone parent. All native presentation and authored executable geometry inputs are unchanged;63previously reviewed PNG are inherited by exact SHA and source proof. Fresh native raw captures are retained separately and are not claimed as newly reviewed visual acceptance.

Archive524 independently verified files; 8actual owned jobs exited0 with original child/waiter retirement, all launchers and closure helper reaped. Maximum health gap151578ms <=180000. No failed acceptance cohort this batch; pre-freeze ambiguous boundary fixture and read-path mistake/root/prevention recorded in docs/plans/2026-10-24-bot-cone-errors.md.

Evidence:docs/acceptance/2026-10-24-bot-cone/index.json. Final baseline omfue/Saved/BuildTemp/bot-cone-source-before-v1.json. All readers retired; source freeze released. Continue concrete single-PC gameplay gaps such as independent zone lifetimes or additional data-driven content; do not repeat accepted nameplate/active-cone/bot-cone work. Full combat replica/11external/MVP33/44 remain. No true network match/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.


## Zone pulse ordering IN PROGRESS

OpenSpec fix-zone-pulse-ordering1/4. Existing zone drain grouped all catch-up pulses by source/key rather than actual within-tick due time, allowing later low-ID hits to take first-lethal credit. Shared DamageZonePulse now carries due_offset; scheduler sorts due offset/source ID/generation/key, preserving current target sampling, death cancellation, lifetime endpoint and ordinary damage settlement. Added2scheduler+1actualECS+1formal60Hz tests (earlier high-ID source vs canonical exacttie; pause/overkill). Four sources FROZEN in zone-order-inputs-v1/config-v1 against final bot-cone source-before-v1/index, whole3470 expected3466same+4changed. All old4bytes preserved before editing. Owned RustV1 ACTIVE tool session36929/jobs omfue/Saved/Acceptance/ZoneOrder/rust-v1; inspect progress/receipt/CIM before duplicate. Current phase initialfullsourcecapture. Helpers prefix omfue/Saved/BuildTemp/zone-order-; backend/build/staged/world91/verify specs ready; prepare-seal-v2 generated seal/closure. PreparationV1 failed literal comma-vs-final-brace match before output; originalsource/tool-output transcription preserved and root/prevention logged in docs/plans/2026-10-24-zone-order-errors.md; nested PowerShell inventory masked outerLuaerror, do not infer success fromthat outerstatus. No formal edits while readers active. Need reapRust ->backend ->UEbuild/staged ->world+sourceverify, seal/reap/closure/finaldocs/tasks/audit. FreshRust expected1888/9ignored (core998/base337) ifallpass. Nativepresentation unchanged; prior63reviewedPNG mayinherit byexactSHA/fullsourceproof, freshcapturesrawonly unlessactuallyviewed. No true networkmatch/quotareset/commit/push/omfx; deadline2026-10-25 00:33:38 Asia/Taipei unchanged.


# Zone pulse ordering COMPLETE

OpenSpec fix-zone-pulse-ordering4/4.

Fixed a concrete combat ordering defect: catch-up zone pulses were grouped by source/key, so a later low-ID hit could precede an earlier high-ID hit in the same tick. Each shared DamageZonePulse now carries its exact positive fixed-point due_offset. The common scheduler derives offsets from the prior remainder and active lifetime, then sorts by due offset, source ID/generation and buff key. BuffTick and ordinary raw-damage settlement retain that chronological order. No hero/camp-name rule or custom kill-credit override.

Existing source-bound death/removal cancellation, rank/admission bounds, clock remainder, inclusive expiry pulse, nonpositive-dt no-op, hostile living target sampling, defenses, shields and first-lethal overkill protection remain. This orders zone pulses relative to other zone pulses. Positions still sample the current simulation state; it does not reconstruct historical movement or impose global chronology on unrelated combat subsystems. Independent zone lifetime policies remain a separate future feature.

New tests prove a high-ID source due at12raw precedes a low-ID source due at16raw and equal-time keys are canonical; one catch-up step equals15/60/120Hz absolute due streams with exact lifetime endpoints. Actual ECS verifies RawDamage source order and normal resistance settlement. Formal60Hz production driver, native admitted zone payloads and real hero death/score paths verify earlier high-ID lethal credit, canonical low-ID exact-tie credit, pause clock preservation and one death despite same-tick overkill. Clock priming is explicit fixture setup with victim outside the area; this is not a newly authored skill-cast or network-match claim.

Fresh8Rust1888passed/0failed/9existing ignored (15/49/120/74/998/337/138/157); unchanged shared Lua cone11/11. Fresh backend build/stage and UE5.8 build/stagedDLL/import SHA passed. Fresh native91: 90success+1existing warning case,0failed/0notrun,warnings4. Fullsource3470:3466unchanged+4exact frozen Rust inputs against index-bound bot-cone parent; all4old byte copies independently match parent hashes. Native presentation and authored executable geometry inputs unchanged;63previously reviewed images inherited by exactSHA/fullsourceproof. Fresh native raw captures retained separately, not claimed newly reviewed.

Archive511 independently verified files; 7actual owned jobs exited0 with original child/waiter retirement, all launchers/closure reaped. Health max151500ms <=180000. Original preparationV1 literal-delimiter failure source and explicitly labeled tool-output transcription retained; correction used distinct prepare-seal-v2, and no frozen gameplay source repair was needed. Error/root/prevention:docs/plans/2026-10-24-zone-order-errors.md.

Evidence:docs/acceptance/2026-10-24-zone-order/index.json. Finalbaseline omfue/Saved/BuildTemp/zone-order-source-before-v1.json. All readers retired/source freeze released. Do not repeat accepted nameplate/active-cone/bot-cone/order work; continue concrete single-PC gameplay such as independent zones or further data-driven content. Fullcombat replica/11external/MVP33/44 remain. No true network match/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.


## Zone pulse retirement IN PROGRESS

OpenSpec fix-zone-pulse-retirement1/4. Previous acceptedzone-order sorted due offsets, but BuffTick still expanded all sources before serial hits; earlier same-tick source death left late ghost damage. Added host-only serialized Outcome::DamageZonePulse{source,payload}; BuffTick queues it in existing scheduler order. game_processor decodes/checks source exactgeneration/aliveHP/faction/removal set at serial resolution, then frontqueues canonical current living hostile nonbuilding RawDamage and ordinary cue before nextpulse. Existing defense/shield/credit path remains. New helper zone_pulse_settlement.rs; explicit managed-cast blacklist prevents script transaction smuggling. Current target sampling at serial time, no history interpolation/new lifetime policy. Updated oldzone ECS test helper for deferred queue;4newcoretests cover serde/currenttargets/defenses, source death/removal/generation/missingdata, invalidpayload no revoke/cue, managed isolation. Formal60Hz added earlier high-ID source killing late source at12raw before16raw, protected target survives; live-source control still kills target. Nativeclock priming is fixture setup, no newskill/networkcast claim.

Six formal sources FROZEN in zone-retire-inputs-v1/config-v1,5originaloldbytes preserved and1newmodule. Parent immutable zone-order source-before-v1/index3470; newwhole3471 expected3465same+6changed. Owned RustV1 ACTIVE tool session46766, jobs omfue/Saved/Acceptance/ZoneRetire/rust-v1, current initialsourcecapture. Inspect receipt/log/CIM before duplicate. Prefix omfue/Saved/BuildTemp/zone-retire-; backend/build/staged/world91/verify/seal/closure specs ready. ExpectedfreshRust1893/9ignored (core1002/base338) ifallpass. Need finish/reapRust,backend,UEbuild/stage,world+verify,seal/reap/closure/finaldocs/tasks/audit. No formal edits until readers retire. Errors/prevention docs/plans/2026-10-24-zone-retire-errors.md (read-only guessed native/mod.rs path; actualnative.rs). No newtrue network/quotareset/commit/push/omfx; deadline2026-10-25 00:33:38 Asia/Taipei unchanged.


## Zone retirement archive recovery IN PROGRESS

Game functionality complete/frozen6inputs; fresh8Rust1893/0fail/9ignored, backend/UE/staged and native91 allpassed originalV1 and launchers reaped. VerifyV1 actualWindows1 correctly rejected parentZoneOrder index lacking ownbaseline row: sealer cloned escaped ^bot%-cone%- filter and archived oldBotConeSaved ratherthanownSaved. Original511fileindex unchanged. Owned amend-v1 passed/retired/reaped; distinct zone-order/index-amended-v2.json (553 independent files) derives entire3470SHAmap from originalsealedBotConeancestor+four sealedSHA deltas, corroborated byoldfrozencopies. PreservedsnapshotSHA newlybound, not retroactivelyoriginallysealed. Amendment report docs/plans/2026-10-24-zone-order-archive-amendment.md.

Current source freeze remains: zone-retire-source-before-v1.json3471, same6gameplay hashes in inputs-v2/config-v2, amendedparent path; no source repair/Rust/native rerun. VerifyV2 ACTIVE toolsession46531 (check receipt/CIM beforeduplicate). Need reap ->seal-v2 actual-exit spec ->reap ->archive-closure-v2 ->reap ->close-docs-v2. Oldseal-v1 neverexecuted and retained; newseal-v2 derivesliteralfilenameTagfromconfig, checksmandatoryownsnapshot/config/manifest/allinputcopies beforeindex. Sealer/closure must retain failedverify-v1 and successfulamend/verify-v2; expected9workers/8exit0+1originalfailedpreflight. Create close-docs-v2 fromoriginalhelper withconfig-v2 and correctsuccess/failurecounts/amendment narrative. All root code is frozen; no newtrue network/quotareset/commit/push/omfx; deadlineunchanged.


# Zone pulse retirement COMPLETE

OpenSpec fix-zone-pulse-retirement4/4.

Fixed same-tick ghost zone damage: BuffTick previously expanded every due pulse into victim damage and cues before serial hits settled, so a source killed by an earlier pulse could still hit later. It now queues a host-only serializable DamageZonePulse envelope in the existing scheduler order. Serial resolution validates the common strict zone payload, exact source generation, living HP/faction and current removal set, then samples current live hostile non-building targets in the anchored circle. Canonical RawDamage children and the ordinary cue settle before the next pulse envelope. Invalid/retired sources emit neither damage nor cue and cancel only their exact-generation source-bound zones. Existing defenses, shields and credit are still owned by normal raw-damage settlement.

Managed script transactions explicitly reject host-only pulse envelopes. Malformed payloads emit nothing and cannot revoke valid live zones. Entity ID reuse cannot revive old pulses or cancel replacement-generation zones. Existing due offsets, inclusive expiry pulse, pause, rank/lifetime bounds and source-bound cancellation remain. This is current serial-state sampling with atomic pulse expansion, not historical movement interpolation, global combat chronology, an independent zone lifetime policy or new skill content.

Four new core tests cover host trace serde/current entering/leaving/new targets and normal resistance settlement, source death/removal/reused generation/missing faction or health, malformed payload no effects/revoke, and managed-cast isolation with an admitted empty-cast control. Existing ECS zone tests now observe deferred pulse order and actual post-settlement HP changes rather than eagerly expanded victim packets. Formal60Hz production-driver test admits native zone payloads and primes clocks without an emitted pulse: high-ID source due12raw kills a lower-ID source due16raw, whose later zone cannot kill the protected ally. A surviving-source control still performs the late kill, proving this is not a blanket late-pulse drop. Clock priming is explicit fixture setup; no newly authored skill cast/network-match claim.

Fresh8Rust1893passed/0failed/9existing ignored (15/49/120/74/1002/338/138/157); shared Lua cone11/11. Fresh backend build/stage and UE5.8 build/stagedDLL/import SHA passed. Fresh native91: 90success+1existing warning case,0failed/0notrun,warnings4. Fullsource3471:3465unchanged+6exact frozen inputs against index-bound zone-order parent;5original old byte copies match parent hashes and1new settlement module. Native presentation and authored executable geometry unchanged;63previously reviewed images inherited by exactSHA/fullsourceproof, fresh native raw captures retained separately and not claimed newly reviewed.

Archive562 independently verified files; 9actual owned jobs retired with original child/waiter identities (8exit0, 1preserved original failure). Health max153250ms <=180000. Sealer actual Windows exit0 and child/waiter retirement verified; its tool-session ID was lost during context compaction, so tool-session reaping is not claimed. Errors/root/prevention:docs/plans/2026-10-24-zone-retire-errors.md and docs/plans/2026-10-24-zone-retire-closure-errors.md.

VerifyV1 original Windows exit1 is preserved: strict baseline binding detected the parent archive selection omission. Original parent index is untouched; a distinct amended parent derives its entire3470byte-SHA map from the originally sealed ancestor+four deltas, preserving snapshot metadata and newly binding its SHA. Successful owned amendment receipt and full-source VerifyV2 bind the same6gameplay inputs; mandatory own-archive coverage prevents recurrence. See docs/plans/2026-10-24-zone-order-archive-amendment.md. No gameplay-source repair or repeated passed gameplay cohort.

Evidence:docs/acceptance/2026-10-24-zone-retire/index.json. Finalbaseline omfue/Saved/BuildTemp/zone-retire-source-before-v1.json. All readers retired/source freeze released. Do not repeat accepted nameplate/cone/order/retirement work; continue concrete single-PC gameplay including independent zone lifetime policies or more data-driven content. Fullcombat replica/11external/MVP33/44 remain. No true network match/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei/automationomoba-48 unchanged.

## Native accuracy IN PROGRESS

OpenSpec fix-native-damage-stat-coherence1/4. Zone-retire complete4/4,index562,source3471,Rust1893/native91; prioritynameplate alreadycomplete. Current new2formalinputs FROZEN: damage_tick.rs and single_lane_match_tests.rs, original bytecopies in native-accuracy-old/manifest. Core reads shared UnitStats evasion/negativeevasion+source miss in native DamageInstance; CProperty-only fallbackdefenses; preserve Unit and Hero baseprecedence/legacycrit; missRNGoperation2 without changingdodge0/crit1. Avoidedactualdamage0/nohealing/normalabilitytrueflagsbypass.2coretests and1formal60Hz actual laneAI test (full evade/miss/negativecounter and removalrestore). No newitem/skill/UI or scriptattackruleclaim. Frozen inputsv1/configv1,expected3471=3469same+2changed vs properlyboundzone-retireparent. ExpectedRust1896/9ignored:15/49/120/74/1004/339/138/157 ifpass.

Owned RustV1 ACTIVE toolsession10326 originalowner22248/jobs omfue/Saved/Acceptance/NativeAccuracy/rust-v1. Current initialsourcecapture; check receipt/progress/CIM before duplicate. Prefix omfue/Saved/BuildTemp/native-accuracy-. Prepared backend/build/staged/world91/verify/seal specs plus archive-closure-v1/close-docs-v1, all syntaxchecked. PreparationV1 failed at redundant textreplacement AFTER manifest/config/sixjobhelpers; originalhelper retained, distinctprepare-v2 onlyfilledremainingsealer/specs. Errorrecord docs/plans/2026-10-24-native-accuracy-errors.md. Need finish/reapRust,backend,UEbuild,staged,world,verify,seal,reap,closure,reap,closedocs. Freezehelperswhenarchived; nevereditsealedhelper; checkedbyte-identicalarchiveretries. No newtrue network/quotareset/commit/push/omfx. omoba-48/deadline2026-10-25 00:33:38Taipei unchanged.

## Native accuracy acceptance checkpoint

Current OpenSpec2/4; all2formalinputs stillfrozen. Fresh8Rust1896/0failed/9ignored (core1004,base339), both newcoretests and actual laneAI60Hz evade/miss/counter/removal test passed. OriginalRustsession10326 reaped0; backend1200/build67100 reaped0, staged direct actual0, world73506 reaped0. Native91=90success+1existingwarningcase,0failed/0notrun,4warnings. All these owned child/waiter identities retired; no extra game/network/quota operation.

VerifyV1 ACTIVE toolsession99099/jobsNativeAccuracy/verify-v1; original strict zone-retire index parentbinding already proper. Need finish/reapverify then seal-v1 spec/reap, execute immutable archive-closure-v1/reap then close-docs-v1. All helper source frozen at seal; do not edit afterward. Generic closure already supports byte-identical existing archive copies. Need update tasks/errors/inventory (close-docs helper ready) and finalprocessaudit. Prefixnative-accuracy-. Rootdeadline andautomationunchanged. Currentmodifiedgamefiles are only damage_tick.rs and single_lane_match_tests.rs; no more edits while readers active.


# Native damage stat coherence COMPLETE

OpenSpec fix-native-damage-stat-coherence4/4.

Native lane/camp DamageInstance settlement now reads common UnitStats evasion/negative-evasion and attacker miss modifiers. Defense lookup includes CProperty-only bodies while preserving existing Unit base armor/magic-resistance and Hero CProperty precedence. No new item/ability, hero-name branches, authored balance or script-projectile rule changes. Legacy critical chance/multiplier and dodge0/crit1 RNG streams remain; new miss uses stable operation2. can_dodge is the existing avoidable-hit gate; normal ability/true flags bypass both. Avoided hits clear actual damage, total and healing and do not emit successful Damage/Heal/DirectCombat outputs. This does not implement every attack system or new miss UI.

Two new core tests exercise actual ECS Sys with property-only/Unit/Hero bases, full evasion, negative-evasion cancellation, full source miss, expiry, lifesteal and ability bypass; deterministic calculators cover zero/one endpoints, repeatable fractional streams, no healing on avoidance and critical-stream independence. Formal production60Hz test uses real spawned opposing lane creeps and native AI: full evasion or miss prevents hits, counter-evasion control hits, and removal restores hits. Other entities are spatially isolated with movement disabled; no native queue fixture is substituted for lane AI.

Fresh8Rust1896passed/0failed/9existing ignored (15/49/120/74/1004/339/138/157), shared Lua cone11/11. Fresh backend/stage and UE5.8/stagedDLL/import SHA passed. Native91:90success+1existing warning case,0failed/0notrun,warnings4. Fullsource3471:3469unchanged+2exact frozen Rust inputs, both original old byte copies independently match index-bound zone-retire parent.63previously reviewed images inherited by exactSHA/fullsourceproof; fresh native raw captures retained separately, not newly reviewed.

Archive505 independently checked files; 7original owned jobs retired with actual Windows exits/owner/waiter identities; max health gap151187ms <=180000. PreparationV1 failed before any job at a redundant replacement; exact original helper remains, distinct prepare-v2 only filled missing artifacts. See docs/plans/2026-10-24-native-accuracy-errors.md and docs/acceptance/2026-10-24-native-accuracy/index.json. Sealed helpers immutable; closure retries require byte-identical copies.

Finalbaseline omfue/Saved/BuildTemp/native-accuracy-source-before-v1.json. All readers retired/source freeze released. Nameplate/cone/zones/ordering/retirement/native accuracy accepted; continue concrete single-PC gameplay. Fullcombat replica/11external/MVP33/44 remain. No true network match/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei/omoba-48 unchanged.

## Attack miss IN PROGRESS

OpenSpec fix-basic-attack-miss-effects1/4. Parentnative-accuracycomplete,index505,Rust1896/native91/source3471; no leftoverworkers at start. Found legacyAccuracyBonus miss zeroedattackdamage but still rolledattachedstun; shared MissPercentage notusedby nativehero launch. New privatecommon attack_accuracy::source_misses retains exactoperation2; nativeDamageInstance refactored tosamefunction; nativehero launch combinescommonmiss withlegacyaccuracy20 andclearsdamage+stunifmissed; dodge0/crit1/stun21 unchanged. No targetevasionimpact/newcontent/scriptchild/summonclaim. Four oldfiles bytepreserved plusnewattack_accuracy.rs: fiveformalinputs FROZEN. Expected3472=3467same+5changed vsstrictlyboundnative-accuracyparent.

Newhelpercoretest exactoperation2/bounds/fractionalrepeat plus formal60Hz actualAttackTarget launches3projectiles, verifies shared/legacy/combinedmiss, damage/stun/provenance, oncebonusconsumption, latebuffchangesand actualimpact/retirement. ExpectedfreshRust1898/9ignored:15/49/120/74/1005/340/138/157 ifpass. OwnedRustV1 ACTIVE toolsession13515; jobsomfue/Saved/Acceptance/AttackMiss/rust-v1; prefixomfue/Saved/BuildTemp/attack-miss-. Checkreceipt/progress/CIM beforeduplicate. Backend/build/staged/world91/verify/seal specs and immutableclosure/closedocs ready. Need finish/reapall, thenfinaldocs/tasks/audit. No formal editswhileactive; helpersfreezeatsealing; newversionsforrecovery. Errors docs/plans/2026-10-24-attack-miss-errors.md. Deadline2026-10-25 00:33:38Taipei/omoba-48 unchanged; no network/quota/commit/push/omfx/unrelatedtermination.

## Attack miss V2 recovery IN PROGRESS

OriginalRustV1 completedWindows1, owner952/waiter48096 retired/cleanuptrue; launcher13515 reaped1. Core1005passed,base339pass+1fail; bridge/clientNOTRUNinV1. Originalrawfailure mode0 atnewtestline23: launched0expected3. Fixtureasd_count100s withinterval1s carriespositiveoverage in start_attack_windup and neverentersnegativewindup/Impact. Onlycorrectedtest to asd_count=asd.v (normalexactready), afterallV1readersretired. Gamechanges andnewsharedmodule unchanged. Originalfiveinputs-v1,source-before-v1,helpers/logs/exitreceipts retained; no failure relabeledpass.

Newfiveinputs-v2/config-v2 FROZEN; source-before-v2/verified-v2; sameparentnative-accuracy,index505,Rust1896,source3471. Expected3472=3467same+5changed (4originaloldbytes+newattack_accuracy.rs), freshRust1898/9ignored ifpass. OwnedRustV2 ACTIVE toolsession79928, outputjobsAttackMiss/rust-v2, currentinitialsourcecapture. Prefixattack-miss-. Need finish/reap thenallNEWv2 backend/build/staged/world91/verify/seal specs, archive-closure-v2/reap, close-docs-v2. Saved V1 build/world/source/seal specs neverran; useV2only. Finalexpected8ownedjobs:7successV2+1retainedfailedRustV1. Sealer-v2 readsRustV2summary/verify-v2 andarchivesbothcohorts; ordinarystrictindex-boundparent; allhelperversionsimmutableatsealing. No formaleditwhileactive, no duplicatejobs. Errorcause/prevention docs/plans/2026-10-24-attack-miss-errors.md. Userdeadline2026-10-25 00:33:38Taipei/omoba-48unchanged; no newtrue network/quotareset/commit/push/omfx/unrelatedtermination.

## Attack miss V2 build/native checkpoint

FreshcompleteRustV2 passed1898/0failed/9ignored (core1005/base340); real newformal5mode3projectile/actualimpacttest passed afterfixturecount=interval fix. OriginalV1 failed339basepass/1fail retainedunchanged, notrelabeled. RustV2session79928, backendV2session24905 andUEbuildV2session70355 allreaped0; stagedV2directactual0. Alloriginalownedchild/waiterreceiptsretired/cleanuptrue. Fiveformalinputs remainfrozen inputs-v2/config-v2/source-before-v2 count3472.

WorldV2 ACTIVE toolsession80752; workomfue/Saved/Automation/AttackMissWorldV2, expectednative91. Checkreceipt/log/CIM beforeduplicate. Needreapworld ->verify-v2 spec/reap ->seal-v2 spec/reap ->immutablearchive-closure-v2/reap ->close-docs-v2 ->finalprocessaudit. source-v2 andseal-v2 were extended BEFORE their first execution with actualfailedRustV1receipt validation plusmanifestdelta proof:4identicalruntimeinputs+onlysingle_lane_match_tests.rschangedfixture; syntaxchecked. Ordinaryparent native-accuracy index505/Rust1896/ownsource-before-v1bound. Current expected3472=3467same+5changed (4originaloldbytes+newmodule), final8ownedjobs7passV2+1failedRustV1. Future cleanbatch must NOTclone recovery-specific gates (seeattack-miss-workflow-notes.md); useordinarysource-v1/genericsealer structure. No formaleditwhilecurrentreadersactive; no helpereditaftersealing. Rootdeadline/omoba-48unchanged. No newtrue network/quotareset/commit/push/omfx/unrelatedtermination.


# Basic attack miss effects COMPLETE

OpenSpec fix-basic-attack-miss-effects4/4.

Native DamageInstance and native hero launch now share the exact existing source-miss operation2 through authority-only attack_accuracy. Legacy dodge0/critical1 and projectile accuracy20/stun21 streams stay stable. Hero launch combines shared MissPercentage with legacy AccuracyBonus; either failure clears physical damage and attached attack stun. Committed timing, next-attack bonus consumption and harmless visual pellets remain. Original RustV1 Windows exit1/base339pass+1fail retained: invalid fixture attack clock100s produced no shot. After retirement/reaping, only the fixture changed to exactly one ready interval; fresh complete RustV2 and subsequent v2 jobs verify the corrected frozen inputs. See original raw logs/inputs, not a relabeled pass. No target-evasion-at-impact, script child/summon reclassification, new item/ability or hero-name rule. Zero base damage without a miss preserves its prior effect policy.

New core test proves exact compatibility with old operation2, checked zero/one/out-of-range endpoints and repeatable fractional results. Previous native ECS/determinism coverage still passes. Formal production60Hz PlayerInput attacks use three actual native projectiles: one real shot plus two visual pellets; shared miss, legacy miss and combined miss carry zero damage/no stun/no normal provenance and consume the one-shot bonus once. Successful controls hit with attached stun. Removing launch miss in flight cannot restore damage; adding late source miss cannot cancel a committed hit. Actual impact HP/control and retirement are verified, not just packets.

Fresh8Rust1898passed/0failed/9existing ignored (15/49/120/74/1005/340/138/157), shared Lua cone11/11. Fresh backend/stage and UE5.8/stagedDLL/import SHA passed. Native91:90success+1existing warning case,0failed/0notrun,warnings4. Fullsource3472:3467unchanged+5exact frozen Rust inputs (four original byte copies match index-bound native-accuracy parent plus one new helper).63previously reviewed images inherited by exactSHA/fullsourceproof; fresh native raw captures retained separately, not newly reviewed.

Archive572 independently checked files; 8original owned jobs retired with actual Windows exits/owner/waiter identities; max health gap151843ms <=180000. Errors/root/prevention:docs/plans/2026-10-24-attack-miss-errors.md. Evidence:docs/acceptance/2026-10-24-attack-miss/index.json. Finalbaseline omfue/Saved/BuildTemp/attack-miss-source-before-v2.json. All readers retired/source freeze released.

Nameplate/cone/zones/native accuracy/hero source misses accepted; continue concrete single-PC gameplay, including target evasion for native hero projectiles with explicit impact policy. Fullcombat replica/11external/MVP33/44 remain. No true network match/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei/omoba-48 unchanged.

## Primary projectile evasion IN PROGRESS

OpenSpec complete-basic-projectile-evasion1/4. Parentattack-misscomplete,index572,Rust1898/native91/source3472; parentownbaseline source-before-v2 bound. Nameplatealreadycomplete,noactiveworkersatstart. Newtarget_evades commonoperation0 exactnativeDodge extractedbesidesunchangedSourceMiss2; nativeDamage usesit. ProjectileRead nowexistingBuffStore/MasterSeed; atsingle-target reachedimpact checkauthorityorigin.normal_attack and exactorigin.source==projectile.owner, samplecurrenttargetevasion/negativeevasion BEFOREdamage/hit/slow/stun generation. AlwayskeepDeath/publicremoval. No sourcealivegate,profile/nameinference,AOE/swept/script/summonclassification/newUI. New2coretests stableoperation0/provenance7modesincludingdeleted/reusedsource andpublicremoval; formal60Hz7target-statecases usesactual3shots/HP/control/retirement, exactreadyinterval andoncebonus, pause/expiry/lateadd/remove/counter.

Fourformalinputs FROZEN inputs-v1/config-v1/old4bytes; expected3472=3468same+4changed vsattack-missparent. ExpectedfreshRust1901/9ignored:15/49/120/74/1007/341/138/157 ifpass. OwnedRustV1 ACTIVE toolsession61575/jobsomfue/Saved/Acceptance/ProjectileEvade/rust-v1, initialsourcecapture. Prefixomfue/Saved/BuildTemp/projectile-evade-. Cleanordinarysource-v1/genericsealer clonedfromnative-accuracy, NOT priorrecovery-specificgates. Backend/build/staged/world91/verify/seal specs+immutableclosure/closedocs ready/syntaxchecked. Needfinish/reapall,seal/closure/docs/tasks/audit/finalprocesscheck. No formaleditwhileactive/no duplicatejobs, helperfreezeatsealing. Errors docs/plans/2026-10-24-projectile-evade-errors.md (read-onlyguessedobservable_facts.rs;actualstable_fact.rs). No true network/quotareset/commit/push/omfx/unrelatedtermination; deadline2026-10-25 00:33:38Taipei/omoba-48unchanged.

## Primary projectile evasion build/native checkpoint

OpenSpec2/4. FreshcompleteRust1901/0failed/9ignored, core1007/base341, bothnewcoretests andactualformal60Hz7case/3shot/HP/control/retirement passed onFIRSTcohort. Rustsession61575/backend9989/UEbuild4255 reaped0; stageddirectactual0. Alloriginalownedreceiptsretired/cleanuptrue. Fourformalinputs stillFROZEN inputs-v1/config-v1/source-before-v1 count3472. Expected3468same+4changed vsparentattack-miss index572/Rust1898/ownsource-before-v2bound. Cleanordinarysource/seal helpers; no priorbatchfailure-recovery gates.

WorldV1 ACTIVE toolsession40201, workomfue/Saved/Automation/ProjectileEvadeWorldV1, expectednative91. Checkreceipt/log/CIM beforeduplicate. Needreapworld ->verify-v1 spec/reap ->seal-v1 spec/reap ->immutablearchive-closure-v1/reap ->close-docs-v1 ->finalprocessaudit. Prefixomfue/Saved/BuildTemp/projectile-evade-. Finalexpected7ownedjobs all0 ifpass. No newfailures exceptread-onlyguessedfactfile recordedbeforecodework; do noteditformalwhileactive orhelpersaftersealing. Allwarmupreadiness usesexactinterval; no needrepeatoldnameplate/attackmisswork. Source-timemiss remainslaunchimmutable, targetevasioncurrentatimpactonlynormalcheckedhomingsingle-target; script/visual/AOE/sweptclassificationunchanged. No network/quota/commit/push/omfx/unrelatedtermination; deadline/omoba-48 unchanged.


# Basic projectile impact evasion COMPLETE

OpenSpec complete-basic-projectile-evasion4/4.

Native DamageInstance and native primary homing single-target attack impacts share exact target RNG operation0 through attack_accuracy; source miss2 and legacy crit1/accuracy20/stun21 remain unchanged. A primary shot checks current exact-generation target evasion/negative-evasion at reached impact only when its authority origin.normal_attack is true and origin.source matches projectile.owner. Evaded impacts skip create_projectile_damage entirely, so no RawDamage/ProjectileHit/slow/stun is queued; ordinary projectile Death/public removal remains. Owner death/reused numeric ID does not disable this policy. No NORMAL-profile inference, hero-name rule, source-alive revocation, AOE/swept/script/summon reclassification, new ABI/item or miss UI. Launch source miss remains immutable; target evasion is current at impact.

Two new core tests prove exact operation0 compatibility/endpoints/fractional repeatability and actual projectile Sys provenance modes: primary, script NORMAL, missing origin, counter-evasion, retired/reused source, no evasion and mismatched origin. Evaded cases contain only projectile Death and still emit public removal, with no damage/hit/slow/stun. Formal production60Hz PlayerInput test uses three actual native shots (one real plus two harmless visual pellets), ordinary exactly-ready attack clock,120damage/one-shot consumption at launch and seven impact-state cases: late evasion addition, removal, retained evasion, counter, no-evasion control, expiry and paused retention. Actual final HP/control/retirement and no bonus refund verified.

Fresh8Rust1901passed/0failed/9existing ignored (15/49/120/74/1007/341/138/157), shared Lua cone11/11. Fresh backend/stage and UE5.8/stagedDLL/import SHA passed. Native91:90success+1existing warning case,0failed/0notrun,warnings4. Fullsource3472:3468unchanged+4exact frozen Rust inputs; four original pre-edit byte copies match strict index-bound attack-miss source-before-v2 parent.63previously reviewed images inherited by exactSHA/fullsourceproof; fresh native raw captures retained separately, not newly reviewed.

Archive508 independently checked files; 7original owned jobs retired with actual Windows exits/owner/waiter identities; max health gap151203ms <=180000. Errors/root/prevention:docs/plans/2026-10-24-projectile-evade-errors.md. Evidence:docs/acceptance/2026-10-24-projectile-evade/index.json. Finalbaseline omfue/Saved/BuildTemp/projectile-evade-source-before-v1.json. All readers retired/source freeze released.

Nameplate/cone/zones/native accuracy/hero launch misses/primary impact evasion accepted. Remaining content/system gaps include script/summon attack classification, more concrete combat content and miss presentation; fullcombat replica/11external/MVP33/44 remain. No true network match/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei/omoba-48 unchanged.


## Script ordinary attack accuracy IN PROGRESS

OpenSpec complete-script-attack-accuracy1/4. Typed PathSpec::AttackHoming via existing full GameWorld ABI (no new virtual slot), summon shooting integration; host rejects child/context/managed-cast, nonzero splash/hit radius/nonpositive speed/negative damage and common target admission. Shared source miss2 at launch clears damage/control, current target evasion0 at impact uses separate authority script_attack flag, never native hero normal_attack eligibility. General script Homing/Straight unchanged. Zero-damage/missed script shots retain existing harmless callback policy. New formal real-summon production60Hz7cases, actual2shots, source expiry beforeimpact/HP/retirement; core provenance test extended3scriptcases.

12formalinputs FROZEN inputs-v1/config-v1; parentprojectile-evade index508/Rust1901/source3472 ownsource-before-v1bound. ExpectedfreshRust1902/core1007/base342/native91; fullsource3472=3460same+12changed. OwnedRustV1 ACTIVE API session94946. Prefixomfue/Saved/BuildTemp/script-accuracy-, jobsomfue/Saved/Acceptance/ScriptAccuracy/. No duplicatejobs or formaledituntilreadersretired. Preparedbackend/build/staged/world/verify/seal/closure/closedocs helpers. EditV1 failed assertion before gameprocessorwrite; V2 applies only original gameprocessorfile and completesdocs, earlierwriteskept, no blindrerun; originals/errorspreserved. Need finish/reapRust then subsequentjobs, seal/closure/docs. No network/quota/commit/push/omfx/unrelatedtermination; deadline/omoba-48unchanged.


## Script attack accuracy fixture recovery

RustV1session94946 reaped actualWindows1; owner10732/waiter43996 retired/cleanuptrue/max151937ms. Core1007pass;base341pass/1newfailure mode3 HP+900raw duefixturezeroUnitarmorbutHeroauthorityCPropertydef_physicstillnonzero. Onlyfixturedef_physic=0 corrected AFTERretirement. All11runtimeinputs byte-identical; frozeninputs-v1 originalretained plusfinalinputs-v2/config-v2/source-before-v2. FreshV2all8suite/backend/build/staged/world91/source/seal needed, helpersprepared/syntaxchecked. source-v2/seal-v2 explicitlyvalidateoriginalfailedreceipt and11same+1fixturedelta. No originalfailure relabel; finalclosedocs-v2 documentsrecovery. Need launchRustV2 and track/reapAPI, thenrest.


## Script attack accuracy build/native checkpoint

OpenSpec2/4. FreshcompleteRustV2 passed1902/0failed/9ignored (core1007/base342); actualnewformal60Hz7case2shot/HP/expiry/retirement passed. RustV2session15260/backend46369/UEbuild96436 reapedactual0; stagedownedacceptance directactual0. OriginalRustV1session94946 actual1/reaped, core1007/base341pass1fixturefail retained; onlyCPropertydef_physic=0 fixturefix,11runtimefilesbyteidentical proof required bysource/seal-v2. Alloriginalreceipts retired/cleanuptrue. Formal12inputs FROZEN inputs-v2/config-v2/source-before-v2; parentprojectile-evade index508/Rust1901/ownsource-before-v1bound. Expected3472=3460same+12changed.

WorldV2 ACTIVE API13753, workomfue/Saved/Automation/ScriptAccuracyWorldV2, expectednative91. Checkreceipt/log/CIM beforeduplicate. Needreapworld ->verify-v2spec/reap ->seal-v2spec/reap ->immutablearchive-closure-v2/reap ->close-docs-v2 ->finalprocessaudit. Prefixomfue/Saved/BuildTemp/script-accuracy-. SealerarchivesfailedV1withactualexitproof+successfulV2, finalexpected8originalworkers7zero+1preservedfailure. close-docs-v2correctlyparentbaselinev1/finalownv2; allhelpersunsealeduntilfirstseal, nevereditafter. No formaleditwhileactive. No network/quota/commit/push/omfx/unrelatedtermination; deadline/omoba-48unchanged.


# Script ordinary attack accuracy COMPLETE

OpenSpec complete-script-attack-accuracy4/4.

Typed PathSpec::AttackHoming explicitly classifies single-target script ordinary attacks through the existing projectile ABI; both host and content DLL rebuilt with Rust1.95.0. Saika summon ordinary shooting consumes it. Shared source-miss operation2 is sampled at launch and clears damage/attached slow/stun, without revoking flight. Shared target-evasion operation0 samples current target evasion/counter at impact only for authority script_attack or native normal_attack provenance, matching exact source generation. Evasion suppresses damage/hit/control and preserves public retirement. Script attack eligibility is separate from native hero hit-passive eligibility. Ordinary Homing/Straight, NORMAL-profile script children and visual shots keep their semantics. No name inference, new GameWorld virtual slot, source-death cancellation or network match. Zero-damage/source-missed script shots retain the existing harmless projectile callback policy; they never acquire native normal-hit provenance.

Formal production60Hz uses the real summoning skill and two actual ordinary shots, with source miss/removal, target evasion, late target evasion/removal, counter-evasion, late source miss and paused flight. Every source expires before impact; actual target HP and exact projectile/provenance retirement are checked. Existing native projectile core test adds explicit script provenance, retired-source and mismatched-origin controls while checking no damage/hit/slow/stun envelopes and preserved public removal.

Fresh8Rust1902passed/0failed/9existing ignored. Fresh backend/stage and UE5.8/stagedDLL/import SHA passed. Native91:90success+1existing warning case,0failed/0notrun,warnings4. Fullsource3472:3460unchanged+12exact frozen Rust ABI/consumer/provenance/test inputs. Original failed RustV1 (341base pass/1fixture failure) preserved with actual Windows1/retirement; finalV2 differs only in authoritative fixture armor,11runtimeinputs byte-identical. CompletefreshV2 passes. Original pre-edit byte copies match strict index-bound projectile-evade source-before-v1 parent.63previously reviewed images inherited by exactSHA/fullsourceproof; fresh native raw captures retained separately, not newly reviewed.

Archive593 independently checked files; 8original owned jobs retired with actual Windows exits/owner/waiter identities; max health gap151937ms <=180000. Errors/root/prevention:docs/plans/2026-10-24-script-accuracy-errors.md. Evidence:docs/acceptance/2026-10-24-script-accuracy/index.json. Finalbaseline omfue/Saved/BuildTemp/script-accuracy-source-before-v2.json. All readers retired/source freeze released.

Remaining: more concrete gameplay/content, explicit missed-shot presentation and broader scripted attack geometry/critical policies; fullcombat replica/11external/MVP33/44 remain. No true network match/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei/omoba-48 unchanged.

Post-seal report writer V2 failed before writes because actual index uses source_count, not source.count. Immutableoriginal remainssealed. Newscript-accuracy-doc-recovery-v3 validates schema+originalhelperSHA then evaluates one-field correction; docs/tasks/audit complete. Recovery records docs/plans/2026-10-24-script-accuracy-closure-errors.md are outside593-file sealedcohort, not retroactively claimed. AllknownAPI sessions reaped:94946 actual1originalfailure;15260/46369/96436/13753/5620/21748/45159 actual0; stageddirectactual0. FinalCIM empty.


## Attack avoidance feedback IN PROGRESS

OpenSpec complete-attack-avoidance-feedback. Priornameplate/scriptaccuracycomplete,index593/Rust1902/native91/source3472. Noactivejobsatstart. NewFactKind36/ObservableFactAttackAvoided appended; targetonlyAVD1 publicsafeprojection requiresvisible/liveidentity, noattacker/reason. TypedPresentationCueAVD1->existingretention/ACK->FxImpactcue3->nativeOnAttackAvoidedCue; noimpactsound/ring, boundedshared64global/4ownerfloatingMISStextwithpause/hide/generationretirement. NativeDamage actualis_dodged (miss/evasion) and native/script primaryprojectiletargetevasionemit; source-missedprojectileflightcontinuespriorpolicy(no fabricatedimpactcue). Coreprivacy+codec2newtests, bridge1new/client1newtest, actualformalnativeAIandsummonsevenmodescheckfacts; nativePublicHealthTextLifecycleaddsactualMISSglyphreadpixels+capture health-text/attack-avoided.png, no newnativecase. ExpectedfreshRust1906/core1009/base342/bridge139/client158/native91.

OriginalV1 API21356 intentionallycancelledbeforecompletedtests afterreadingappend-onlysnapshotcontract; newclienttestmustusefreshafter_ackview. Exactowner3524/creation134372649446714253stop_owned; supervisorownedjobdescendantretirementonly, Windows1actualreceipt/reaped; waiter35052retired/cleanuptrue/max62812, finalCIMnone. Thisisnotcompletedtestfailureorpass. V2onlyclientfixturechanges;13otherinputsbyteidentical, source/seal-v2enforcecancelrecord+actualexitproof+delta. Frozen14formalinputs inputs-v2/config-v2, expected3472=3458same+14changed. Parentownbaseline script-accuracy-source-before-v2bound. Prefixomfue/Saved/BuildTemp/avoid-feedback-, jobsomfue/Saved/Acceptance/AvoidFeedback/, worldAvoidFeedbackWorldV2. NeedlaunchRustV2(track/reapAPI), backend/build/staged/world, viewactualnewMISSimage thencreatevisual-review-v2.json SHA proof, verify/seal/closure/docs. Sealerrequiresonefreshreview plus63historicalimages ->64total. NewreportMUSTuseindex.source_count (priorclosurebug), immutablehelpersafterseal. No formaleditwhileactive. No network/quota/commit/push/omfx/unrelatedtermination; deadline/omoba-48unchanged.


## Avoidance feedback RustV2 checkpoint

ACTIVE API6956, jobomfue/Saved/Acceptance/AvoidFeedback/rust-v2, originalowner45628/waiter6352. Sourcecaptured3472/frozen14inputs-v2; firstABI/content/templates/codegen/corepassed; core1009 includesnewprivacy/codec. Actualbase summon7cases/avoidancefacts passed, fullbase342longreplay stillrunning atcheckpoint, no sourcechanges. ExpectedcompleteRust1906/core1009/base342/bridge139/client158/9ignored/native91. NeedreapRust ->backend/build/staged/world-v2 ->VIEWfresh worldAvoidFeedbackWorldV2/health-text/attack-avoided.png ->runavoid-feedback-review-v2.lua ONLYafterview ->verify-v2 ->seal-v2/reap ->archive-closure-v2/reap ->close-docs-v2 ->finalprocessaudit. Newworldv2adds -om-health-text-capture=.../health-text; newfreshMISSimage plusprior63 =>64reviewedimages. Close-docs-v2precreatedwithassertindex.source_count3472/images64, avoidspriornil-sourcebug. Source/seal-v2enforceintentionalV1cancelactualreceipt+cancelrecord+13same/1clientfixturechanged, do notrelabelfailureorpass. OriginalV1 API21356 actual1/reaped, job3524creation134372649446714253onlyownedstop; retiredcleanuptrue; partiallogs/inputs retained. No formaleditwhileactive/nohelpereditafterseal. Deadline/omoba-48unchanged; no network/quota/commit/push/omfx/unrelatedtermination.


## Avoidance feedback build/native checkpoint

OpenSpec1/4 pendingnativeglyphvalidation. FreshcompleteRustV2 passed1906/0failed/9ignored (core1009/base342/bridge139/client158), actualsummon7casesandnativeDamageInstanceavoidancefacts pass; newclientACK/hideusesfreshsnapshot. Rustsession6956/backend96598/UEbuild33544 allreapedactual0; stagedownedacceptancedirectactual0, DLL/importSHAverified. V1intentionalcancelAPI21356actual1/reaped remainswithpartiallogs/original14inputs, source/seal-v2checkcancelrecord+13same/1clientfixturefix. Formal14inputs FROZEN inputs-v2/config-v2/source-before-v2, expected3472=3458same+14changed vsparentScriptAccuracyindex593/Rust1902/ownbaselinev2bound.

WorldV2 ACTIVE API35967, workomfue/Saved/Automation/AvoidFeedbackWorldV2, expected91. Checkreceipt/engine/CIMbeforeduplicate. Newworld-om-health-text-capture=health-text writespublic-restoration.png andattack-avoided.png. MUSTviewexactfreshattack-avoided.png afterworldpass, thenrunavoid-feedback-review-v2.lua tobindmanualSHA; sealerrequiresfresh1+historical63=64. Needworldreap ->review ->verify-v2/reap ->seal-v2/reap ->archive-closure-v2/reap ->close-docs-v2 ->finalCIM. Allhelpersprecreated; reportassertssource_countcorrecttop-level. No helpereditafterseal/noformaleditwhileactive. No network/quota/commit/push/omfx/unrelatedtermination; deadline/omoba-48unchanged.


## Avoidance native coverage correction

WorldV2session35967 reapedactual0/native91pass, but listomittedPublicHealthTextLifecycle; expectedimageabsent/os3 (not a completed failing native case). OriginalWorldV2unchanged. Needrequired ownedglyph-v3 onecase on SAMEfrozen14inputs/stagedDLLSHA,no sourcechange/rebuild/rerunRust. NewglyphworkworldAvoidFeedbackWorldV2/glyph-v3, captureglyph-v3/health-text/attack-avoided.png. Sealer-v2/review-v2/closedocs-v2 were NEVERexecuted/unsealed and updated torequireglyphfamily/count1/proof/image, aggregate92=91+1distinctnative, image64=63historical+1fresh. AlloriginalWorldV2receipts/logskept, no coverage relabel. Needlaunchglyph-v3/trackreapAPI, viewexactimage/runreview-v2, verify-v2/seal/closure/docs. c.config-v2native_count91stilldescribesprimarysuite; finalindexnativecombinesprimary_suite_count91+focused_glyph_count1. Formalinputs remainfrozen; no helpereditaftersealing.


# Confirmed attack avoidance feedback COMPLETE

OpenSpec complete-attack-avoidance-feedback4/4.

Native DamageInstance actual avoidance and native/script primary homing target-evasion resolutions now emit appended FactKind36/ObservableFact::AttackAvoided. Safe projection requires a currently visible disclosed target, exposes no attacker/modifier reason and uses distinct AVD1 public/typed presentation payloads with current disclosure epoch. Existing common retention/ACK and bridge FxImpact cue_id3 carry the target-local one-shot to Unreal. It is not damage or actual projectile contact and plays no impact sound/ring. Native floating MISS text shares the existing64global/4per-owner pool, dedup, pause, fog/generation/scope retirement and supported Latin glyphs. Never inferred from zero damage, health delta or projectile expiry. Source-missed hero/script projectile flight keeps its existing policy and does not fabricate a confirmed impact-avoidance cue.

New strict codec, visible/disclosed/remembered/forgotten projection, bridge anchor/kind and client ACK/hide cases pass. Existing formal production60Hz actual summon seven-state/2shot test checks avoidance facts and damage/retirement; native lane DamageInstance cases check actual avoidance facts. Existing projectile Sys provenance modes prove exact single target-only avoidance and public removal while preserving no damage/hit/control. The completed91-case WorldV2 suite did not include PublicHealthTextLifecycle; a separate required glyph-v3 one-case run on the same frozen14inputs and verified staged DLLs supplies the92nd distinct native case. Native PublicHealthTextLifecycle verifies actual MISS TextRender component, dedup, bounded reuse and rendered glyph pixels; fresh attack-avoided.png manually inspected before sealing. Prior63 viewed images retain historical coverage; they are not used as substitutes for the new glyph evidence.

Fresh8Rust1906passed/0failed/9existing ignored; backend/stage and UE5.8/stagedDLL/import SHA pass. Native92:91success+1existing warning case,0failed/0notrun,warnings4. Fullsource3472:3458unchanged+14exact frozen authority/presentation/test inputs. Pre-edit originals match strict index-bound script-accuracy source-before-v2 parent. OriginalRustV1 intentionally cancelled before completed tests after append-only snapshot contract preflight; actualWindows1/owner/waiter/job retirement and partial logs/inputs/cancel record preserved. V2 only fixes the new ACK test to use a fresh snapshot;13otherinputs byte-identical, checked by final source/seal tools. It is not a relabeled test failure or pass.

Archive600 independently checked files; 9original owned jobs retired with actual Windows exits/owner/waiter identities; max health gap151922ms <=180000. Errors/root/prevention:docs/plans/2026-10-24-avoid-feedback-errors.md. Evidence:docs/acceptance/2026-10-24-avoid-feedback/index.json. Finalbaseline omfue/Saved/BuildTemp/avoid-feedback-source-before-v2.json. All readers retired/source freeze released.

Remaining: explicit source-missed projectile result policy/presentation, more gameplay/critical/geometry content, fullcombat replica and11external/MVP33/44. No true network match/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei/omoba-48 unchanged.


# Immutable missed ordinary projectiles COMPLETE

OpenSpec complete-missed-projectile-outcomes4/4.

Native hero and typed script ordinary shots now retain their already computed source-side miss in authority-only immutable provenance. The homing impact bypasses damage, hit callbacks and control envelopes, emits the existing target-only MISS cue once when the exact target still exists, then retires normally. Source expiry or ID reuse cannot repair the committed result. Visual pellets and hit-context children do not inherit the flag; zero-damage abilities are never classified as missed by damage amount. Legacy serialized origins default to no source miss. No public attacker/reason or ABI changes.

Fresh Rust 1907 passed/0failed/9existing ignored across eight packages; Lua11/11, backend and UE5.8 builds actual exit0. Native 92 distinct explicitly selected cases (91 Success/1 SuccessWithWarnings, 4 existing warnings), including PublicHealthTextLifecycle with required actual MISS PNG. Formal60Hz hero shared/legacy/both miss sources, real summons after source expiry, late modifier changes, primary/visual separation, exact source provenance, legitimate zero damage and target removal passed.

3472 formal sources frozen/verified;3462 unchanged/10 explicit changed versus strict index-bound avoidance-feedback parent. 525 independently rehashed files; 7 original workers retired with validated actual Windows exits, max health gap151297ms. Original edit failures and original input copies retained. All64 reviewed images are inherited with exact SHA and recorded prior coverage; fresh native raw captures retained separately, not claimed as newly viewed.

Remaining: more shared critical/hit and geometric gameplay, full combat replica, presentation/art and11external acceptance/MVP33/44. Next concrete investigation: compare shared UnitStats::crit modifiers with native DamageInstance and hero ordinary projectile launch; do not assume shared stat getters are already consumed. No true network match, quota reset, commit/push or omfx work. Deadline2026-10-25 00:33:38 Asia/Taipei/omoba-48 unchanged.


# Shared ordinary attack critical damage COMPLETE

OpenSpec complete-shared-attack-critical4/4.

Common UnitStats now resolves caller base chance/default multiplier plus shared chance and positive absolute multiplier (including explicit1x), preserving the existing script getter0/1x. Native DamageInstance property/unit/hero bodies consume shared modifiers, retain the legacy hero base chance/default2x and stable critical operation1, respect can_crit, avoid before critical evaluation, multiply physical damage only before defenses, and derive lifesteal from actual physical damage. Hero and explicit AttackHoming ordinary projectiles retain their prior zero implicit base chance and consume shared critical buffs once at launch; no visual pellet, generic ability or missed-shot multiplication. Late buff changes and source expiry do not rewrite committed amounts. No new critical presentation cue is claimed.

Fresh Rust1911 passed/0failed/9existing ignored across eight packages, Lua11/11, backend and UE5.8 actual exit0. Core tests cover bases/defaults/absolute multiplier/expiry, operation1 and native body/flag/miss/physical-only/lifesteal. Formal60Hz hero main+visual shots check120 versus360 with one-shot bonus consumed once and late buff add/remove; real summons check threefold ordinary damage and late buffs after launch with source retirement. Native92 cases (91 Success/1 SuccessWithWarnings, 4 existing warnings), including actual MISS PNG lifecycle.

3472 sources frozen/verified:3466 unchanged/6 explicit modified versus strict index-bound missed-projectile parent. 570 evidence files independently rehashed; 10 original workers retired with validated actual Windows exits; max health gap151343ms. All64 reviewed images inherited with exact SHA and prior coverage; fresh native raw captures retained separately, not claimed as newly reviewed. Backend V1 actual child Windows1 followed a Lua host empty response/exit1073807364 and interrupted outer monitor. Original outer receipt/API reap are unavailable; separately owned recovery verifies original PID/creation/executable/actual exit and unchanged frozen6 inputs. Clearly marked recovered receipt and measured interrupted health gap are retained without backfilled coverage. Backend V2 is a fresh success on identical inputs; other fresh jobs retain normal<=180000ms monitoring. Seal V1 preflight blocked an incorrect inherited destination before any child spawn or parent mutation. Versioned config/inputs-v2 and fresh verify-v2 bind the correct independent archive while all6 runtime inputs remain identical. Original preflight/config/helpers are retained. Errors/prevention recorded in docs/plans/2026-10-24-shared-critical-errors.md.

Remaining: critical visual/typed public outcome policy, more hit/geometric gameplay, full combat replica/art and11external acceptance/MVP33/44. No true network match/quota reset/commit/push/omfx work. Deadline2026-10-25 00:33:38 Asia/Taipei/omoba-48 unchanged.


# Native leech serial settlement COMPLETE

OpenSpec complete-native-leech-settlement4/4.

Existing native DamageInstance physical lifesteal/magical vamp flags now travel in a host-only single-Damage envelope. Serial admission, packet reduction, shields and actual target HP loss precede healing. Post-base-defense component proportions allocate actual loss; pure earns none; a single checked rational rounding combines physical/magical rates and overflow grants no healing. Exact source life/current HP, pending removals, self hits and TD layered bodies cannot earn leech. Nested effects/envelopes are rejected. Existing received-healing, HP cap/no revival and source committed-vitals publication are reused immediately in serial order. DamageResult.healing is diagnostic only; the speculative independent native Heal was removed. Zero-leech native hits retain plain Damage and no extra box allocation.

Fresh Rust1915/0failed/9existing ignored across eight packages; Lua11/11, backend and UE5.8 actual exit0. Core mixed/shield/overkill/packet reduction/healing cut/source reuse/dead source/stale target/nested rejection/checked overflow/serde cases pass. Formal60Hz checks shielded60HP loss, overkill30, immunity0, received-healing cut30 and evasion0, including actual source CommittedVitals. Natural regen and XP threshold are isolated only in the fixture. Native92 cases (91 Success/1 SuccessWithWarnings, 4 existing warnings), including required actual MISS PNG.

3472 sources frozen/verified:3466 unchanged/6 explicit modified versus strict index-bound shared-critical parent. Independent fresh archive destination checked before any readers. 641 independently rehashed evidence files; 9 original workers retired with validated actual Windows exits; max health gap150625ms. All64 reviewed images inherited with exact hashes/historical coverage; fresh native raw captures separately retained, not newly claimed as viewed. RustV1 intentionally cancelled before completed core tests after final cast-policy inspection; actualWindows1/API43376 reaped and owner/waiter/private descendants retired. V2 keeps4 initial inputs byte-identical, extends only the new raw_damage test and adds explicit host-envelope cast rejection. Original partial logs/helpers/config/cancel receipt retained. RustV2 completed core1016/0fail and base343pass/1new fixture failure:100-second shield was rejected by the existing60-second limit. Original actualWindows1/API9901 reaped and full logs/receipts retained. V3 changes only that fixture to an asserted legal5-second/40-capacity shield;5other inputs byte-identical to V2 and exact HP expectations retained. Errors/prevention:docs/plans/2026-10-24-settled-leech-errors.md.

Finalbaseline omfue/Saved/BuildTemp/settled-leech-source-before-v3.json, config-v3.json and inputs-v3 are index-bound. All readers retired and source freeze released after archive closure. Remaining: authored/shared Lifesteal stat and ordinary projectile leech policy, critical public outcome/visuals, more hit/geometric gameplay, full combat replica/art and11external acceptance/MVP33/44. No new item/network match/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei/omoba-48 unchanged.

## Ordinary attack lifesteal final cohort IN PROGRESS

OpenSpec complete-ordinary-attack-lifesteal2/4. Eight frozen runtime/test inputs, prefix omfue/Saved/BuildTemp/ordinary-leech-, config/inputs/source-before-v4. Shared UnitStats Lifesteal0..1, native Attack max(explicit,shared) once, native/script primary launch snapshot; actual-loss serial helper, no skills/visual/children/callback follow-up inheritance. Source death/ID reuse cannot heal a replacement. Final integration discovered callback provenance needed attack_lifesteal0 before ScriptEvent::ProjectileHit while normal-hit attribution remains; V4 changes only game_processor and existing raw_damage core test, six other files byte-identical to passedV3.

ACTIVE RustV4 API53205; inspect logs/receipt and reap before other readers. Expected fresh1918/core1017/base346/bridge139/client158/9ignored plus Lua11/11; eight modified/full3472 versus strict settled-leech parent index and own source-before-v3. V1 failed core compilation E0433 new test reference, actual1/API43704 reaped/retired; V2 core1017pass/base345pass1fixture failure from direct hero ECS deletion breaking persistent owner economy, actual1/API19252 reaped/retired; V3 fixes only fixture to formal Outcome::Death, completed all1918 actual0/API11825 reaped/retired. Preserve all originals, transitions/cohort/retirement hashes. FinalV4 callback extension uses same test count; no source edits during readers. Errors docs/plans/2026-10-24-ordinary-leech-errors.md.

After RustV4 actual0/reap, sequential owned backend-v4/build-v4/staged-v4/world-v4/verify-v4/seal-v4 specs. World explicitly92 includes PublicHealthTextLifecycle/actual MISS PNG. Fresh destination docs/acceptance/2026-10-24-ordinary-leech preflighted. All helpers precreated; inherited64 viewed images retain prior SHA/coverage, no new native presentation claimed. Reap seal then direct fixedLua archive-closure-v4 (may yield/reap), then sealed close-docs-v4 and final CIM. Source_count3472 top-level. No true network/quota/commit/push/omfx/unrelated termination. Deadline/automation unchanged.


# Shared ordinary attack lifesteal COMPLETE

OpenSpec complete-ordinary-attack-lifesteal4/4.

Existing shared Lifesteal now aggregates through UnitStats, normalized0..1. Explicit native Attack packets use the stronger explicit/shared physical rate once; skills/item/environment/reflect retain explicit flags only. Native hero and typed script primary ordinary projectiles snapshot the shared rate at launch. Late buffs cannot change it. Exact source/provenance/nonmissed classification gates serial settlement; children clear the rate, visual shots retain0, profiles never infer opt-in. Actual post-defense physical HP loss after shields/overkill/immunity earns leech; magical/pure components earn none. Existing checked serial helper, received-healing, cap/no revival, exact source life and source CommittedVitals are reused. Before script hit callbacks, the shared leech entitlement is cleared while normal-hit attribution/credit survives; immediate authored follow-up damage cannot inherit it. No additional public schema/ABI/UI policy.

Fresh Rust1918/0failed/9existing ignored, Lua11/11, backend and UE5.8 actual0, staged DLL/import SHA verified. New native attack five-mode test proves shared opt-in/skill opt-out/explicit nonduplication/stronger explicit rate. Core exact projectile provenance/script/native/mismatch/miss/dead/reused source/full shield cases pass. Production60Hz actual hero attacks check launch snapshot, visual pellets0, late-added/removed rate, shield, healing cut, evasion, source miss/death/removal, overkill and shared rate clamps. Actual typed summon launch provenance is checked in its existing formal source-expiry/accuracy/critical fixture. Native92 (91Success/1SuccessWithWarnings, 4existing warnings), required health-text lifecycle/actual MISS PNG.

Fullsource3472=3464unchanged+8frozen modified versus strict parent settled-leech own source-before-v3 baseline. 721 independent evidence files rehashed; 10 original workers retired with identity-bound actual Windows exits; maximum health gap150672ms.64 reviewed images inherited by exact SHA and historical coverage; fresh raw native captures separately retained, not claimed as newly reviewed. Independent archive destinations preflighted. Errors/prevention:docs/plans/2026-10-24-ordinary-leech-errors.md. Final baseline omfue/Saved/BuildTemp/ordinary-leech-source-before-v4.json. Original RustV1 test-module compile error and RustV2 invalid direct hero deletion fixture failure remain preserved with actualWindows1/API43704 and19252 reaped plus exact retirement. V2 changes only the new test module reference; V3 only the new formal fixture to production Outcome::Death. Each transition keeps seven other inputs byte-identical and all exact HP assertions. Intermediate RustV3 passed1918/actual0/API11825 reaped; final callback policy refinement changes two inputs and keeps six identical to V3, with the same core case extended to assert callback isolation. All sources released only after original readers and sealer retired/API reaped/archive closure.

Remaining: authored item/ability lifesteal content, critical public outcomes/visuals, further hit/geometric gameplay, full combat replica/art and11external acceptance/MVP33/44. No new true network match/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei and omoba-48 unchanged.

Post-closure documentation-only correction: actual parent is settled-leech-source-before-v3, this batch own baseline is ordinary-leech-source-before-v4. Sealed config/proof/index721 already correct; original writer preserved. V5 correction/notes outside721 cohort:docs/plans/2026-10-24-ordinary-leech-closure-errors.md. Known API reaps:43704/19252 original failures actual1;11825 intermediate success0;53205 finalRust0;67099backend0;33345UE0;staged direct0;14152world0;18676verify0;40052seal0;16517archiveclosure0;doc writer direct0. All sources/readers released, final CIM empty; automation/deadline unchanged.

## Ordinary leech current-target policy IN PROGRESS

OpenSpec complete-ordinary-leech-target-policy2/4. Shared UnitStats::ordinary_leech_target requires known hostile Faction relation and no canonical IsBuilding; HostileNeutral wildlife remains eligible. Native Attack gates only shared Lifesteal when creating a packet; explicit native flags keep their existing semantics. Ordinary projectile gates shared rate at current impact classification/allegiance while launch ratio remains frozen. Damage admission and callback/source-life/shield/heal-cut math unchanged. Existing core case extends unknown/neutral/structure/current allegiance controls with actual target HP; native formal11states and actual projectile14states include late classification changes and explicit structure control. Expected Rust1918/core1017/base346/native92/source3472.

ACTIVE RustV2 API81872, prefix omfue/Saved/BuildTemp/leech-target-, jobsLeechTarget/rust-v2. Frozen5 inputs config-v2/inputs-v2/source-before-v2; parent docs/acceptance/2026-10-24-ordinary-leech/index.json721 and its index-bound own source-before-v4, expected3467same/5changed. OriginalV1 ABI/content/templates/codegen passed; core E0433 because raw_damage is private inside game_processor, despite filesystem location. ActualWindows1/API60046 reaped/owner/waiter/private readers retired; original inputs/logs/helpers/summary retained. V2 moves pure classifier to verified UnitStats, changes raw_damage/damage_tick, adds pre-edit-preserved UnitStats original, keeps game_processor and base fixture identical to V1. No source edits while readers active. Errors docs/plans/2026-10-24-leech-target-errors.md.

After RustV2 actual0/reap, sequential owned backend-v2/build-v2/staged-v2/world-v2/verify-v2/seal-v2 specs; explicit92native/actualMISS PNG. Fresh archive docs/acceptance/2026-10-24-leech-target checked before readers. All helpers precreated, clean sealer plus failedV1/transition guards. Reap sealer then direct fixedLua archive-closure-v2 (reap if yielded), sealed close-docs-v2, finalCIM/API ledger. Report renders parent/own baselines from validated config (avoid previous broad version-substitution typo).64 reviewed images inherited exactSHA/historical coverage, fresh native raw captures not newly claimed viewed. No network/quota/commit/push/omfx/unrelated termination; deadline/automation unchanged.


# Current-target ordinary leech policy COMPLETE

OpenSpec complete-ordinary-leech-target-policy4/4.

Shared ordinary Lifesteal now uses one current-target eligibility helper: both factions must exist and be hostile under the existing Faction relation, and the target must not have the canonical IsBuilding marker. HostileNeutral wildlife remains eligible; same-team/Ally/passive Neutral/unknown faction and structures cannot grant shared ordinary leech. Native Attack uses this policy when forming the damage packet; ordinary projectile impact checks current classification and allegiance, independently of its frozen launch rate. No identity/camp/hero special cases. Explicit native physical/spell leech flags preserve their separate authored effect semantics, including explicit structure controls. Damage admission/flight, source-life checks, actual physical HP-loss/shield/overkill arithmetic, received-healing, callback isolation and source CommittedVitals remain unchanged.

Fresh Rust1918/0failed/9existing ignored and Lua11/11. Existing meaningful cases extended: core projectile provenance/life/mixed damage plus structures/current allies/HostileNeutral/passive Neutral/missing source and target factions, with actual target HP proving healing policy does not invent damage admission; native formal60Hz11states includes shared structure rejection and explicit structure-leech control; actual hero projectile14states includes building/allegiance changes after launch and HostileNeutral success. Backend/UE5.8 actual0 and staged DLL/import SHA verified. Native92 (91Success/1SuccessWithWarnings, 4existing warnings), required PublicHealthTextLifecycle/actual MISS PNG.

Fullsource3472=3467unchanged+5explicit frozen inputs against strict parent omfue/Saved/BuildTemp/ordinary-leech-source-before-v4.json, index docs/acceptance/2026-10-24-ordinary-leech/index.json. Own baseline omfue/Saved/BuildTemp/leech-target-source-before-v2.json. 571 independently rehashed evidence files and 8 original workers retired with identity-bound actual Windows exits; maximum health gap150688ms.64 prior reviewed images retain exact SHA/historical coverage; fresh native raw captures separately retained and not newly claimed as reviewed. Fresh archive/jobs/run destinations preflighted. Errors/prevention:docs/plans/2026-10-24-leech-target-errors.md. Original RustV1 core compilation E0433/private module ownership failure is preserved with actualWindows1/API60046 reaped and exact retirement. V2 moves the pure classifier to verified shared UnitStats, changes only raw_damage/damage_tick among the four original inputs, adds the preserved fifth UnitStats input and keeps the two other inputs byte-identical. All exact healing/damage assertions retained. All sources released only after original readers/sealer retired and archive closure.

Remaining: authored item/ability lifesteal content, critical public outcomes/visuals, further geometry/gameplay, full combat replica/art and11external acceptance/MVP33/44. No true network match/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei/omoba-48 unchanged.

Leech-target final API ledger:60046 original compile failure actual1/reaped;81872 finalRust0;17432backend0;83901UE0;staged direct0;22292world0;71325verify0;30533seal0;94167archiveclosure0;closedocs direct0. All original readers/sealer retired, source freeze released, final CIM empty. Report parent is read directly from config/proof (ordinary-leech-source-before-v4), own baseline leech-target-source-before-v2. Next prioritize player-facing authored lifesteal ability/item content through shared rank metadata/effects and truthful tooltip, then critical public outcome/presentation; do not rerun accepted nameplate/leech audits without a new change.


# Authored lifesteal transform COMPLETE

OpenSpec complete-authored-lifesteal-transform4/4.

Matchlock Gun now authors12.5%/18.75%/25% ordinary physical lifesteal for its three ranks in the shared Lua ability data. Existing generic bounded transform fields, effect-preview modifiers and BuffStore feed shared UnitStats.Lifesteal and the settled ordinary-hit path; refresh replaces one lease, expiry removes it. No hero/camp identity special case in damage processing. Actual hostile physical HP loss grants healing; shields, structures and non-hostile/unknown targets cannot supply shared ordinary leech; existing received-healing and source-life rules remain. Description and generated Unreal metadata publish all ranks and conditions from the same content source.

Existing script/formal tests extended for exact Q10 rates128/192/256, refresh/expiry, invalid/out-of-range/subquantum metadata and effect overrides. The production60Hz all-rank transform case now observes actual ordinary projectile healing with passive regeneration disabled and visual children ineligible, then expiry clears the rate. Native ReadableHudToolTip reads generated actual R metadata, asserts rank extras and opens the actual R card through Slate pointer hover; its fresh PNG was directly viewed and SHA-bound.64 historical reviewed images retain their original provenance; total65, not65 newly viewed captures.

Fresh Rust1918/0failed/9existing ignored and Lua11/11. Codegen/backend/UE5.8 actual0, staged DLL/import hashes verified. Native92 (91Success/1SuccessWithWarnings, 4existing warnings), required actual MISS PNG and authored R tooltip PNG.

Fullsource3472=3464unchanged+8explicit frozen inputs against strict parent omfue/Saved/BuildTemp/leech-target-source-before-v2.json, index docs/acceptance/2026-10-24-leech-target/index.json. Own baseline omfue/Saved/BuildTemp/authored-leech-source-before-v2.json. 938 independent evidence files and 12 original workers retired with identity-bound actual Windows exits; maximum health gap150641ms. Generated outputs were preserved before codegen and frozen only after codegen retirement. Errors/prevention:docs/plans/2026-10-24-authored-leech-errors.md. Original readers/sealer retired before source release and archive closure.

WorldV1 failed only the new tooltip fixture hover (actual Windows1/API60059 reaped). V2 publishes the actual ability HUD state and hovers card-border padding rather than a child-owned tooltip. Only the native test CPP changed; all successful V1 Rust/content inputs are SHA-identical. Fresh V2 UE/staged/native/source acceptance follows retirement, retaining the original failure. Recovery-preparer N02 failed before readers and was completed by a new immutable V3 preparer.

Remaining: authored item lifesteal, critical public outcomes/visuals, more geometry/gameplay and combat coverage/art. Full MVP remains33/44 with11external acceptance items. No true network match/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei/omoba-48 unchanged.


# Equipment lifesteal COMPLETE

OpenSpec complete-item-lifesteal4/4.

Shared authored item schema now has normalized0..1lifesteal, legacy zero omission and nonfinite/negative/over-one/subquantum/type guards. Generated Rust catalog/runtime ItemBonus and native Unreal metadata consume the same field. Inventory totals feed one canonical permanent BuffStore lease, replaced on dirty rebuild and production new-life finalization, removed at zero; no item/hero identity condition in processing. Shared ordinary attack/projectile leech settlement, hostile non-building policy, shield/actual physical HP-loss/healing-reduction/source-life checks remain authoritative. Existing-body equipment rebuild never heals.

Real shop offers catalog15汲生短刃(900gold,10attack,12.5%lifesteal, sword component) and16汲生長刀(1800gold,25attack,25%lifesteal, short blade+sword recipe). Existing item prices/recipes unchanged. Shared native item tooltip publishes exact percentage, actual physical HP-loss, exclusions, healing reduction and100%cap. New actual disabled item-card hover PNG directly inspected;65 historical reviewed images retain their source/SHA, total66, not66 fresh views.

New model/codegen contracts and production60Hz generated-shop test cover buy, multi-item aggregation, recipe replacement, independent ability-buff addition, upgrade undo, sale and rejected undo after sale, zero-inventory removal, actual100damage->12.5healing, unchanged current HP across inventory rebuilds and real death/new-life restoration. Fresh Rust1921/0failed/9existing ignored and Lua11/11. Codegen/backend/UE5.8 actual0, staged DLL/import hashes verified. Native92(91Success/1SuccessWithWarnings,4existing warnings), actual MISS/R-tooltip/equipment-tooltip PNGs required.

Fullsource3472=3454unchanged+18explicit frozen inputs against strict parentomfue/Saved/BuildTemp/authored-leech-source-before-v2.json, indexdocs/acceptance/2026-10-24-authored-leech/index.json. Own baselineomfue/Saved/BuildTemp/item-leech-source-before-v3.json. 755 independent rehashed evidence files, 10 original workers retired with identity-bound actual Windows exits; maximum health gap150750ms. All generated candidates were preserved before owned codegen and exact outputs frozen after retirement. Errors/prevention:docs/plans/2026-10-24-item-leech-errors.md. Sources released only after original readers/sealer retirement and archive closure.

RustV1 recipe-fixture failure actual1/API80620 retired/reaped; V2 explicitly purchases components and asserts existing purchase-only undo rejection after sale. V2 changed only the fixture and passed1921tests. Review then found missing custom-catalog admission and purchase net-leech benefit tracking; V3 adds both policies and actual post-heal refund rejection. Fresh V3 focused plus full regressions are final; V1 failure and accepted V2 are independently retained. Sale undo remains a separate gameplay gap.

Remaining: critical public outcomes/visuals, additional geometry/gameplay, wider combat coverage/art and11external acceptance items; full MVP remains33/44. No true network match/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei and omoba-48 unchanged.


# Settled critical feedback COMPLETE

OpenSpec complete-critical-hit-feedback4/4.

Existing critical damage now retains explicit provenance through native damage and primary native/typed ordinary projectiles. Host-only appended DamageWithCritical envelope opens around immediate damage/leech only; cast admission rejects it, nested/non-damage packets are rejected. Default-false attack_critical/source_critical preserve legacy JSON and freeze the existing launch roll; visual pellets, inherited child shots and immediate on-hit callbacks clear entitlement. No magnitude inference or second RNG roll, no change to damage rules. Serial settlement emits CriticalHit only for eligible non-TD-layer target actual positive physical HP loss after admission/global modifiers/shields. Fully shielded, missed/dodged, zero-physical, stale/dead and invalid envelope paths emit none. Late modifier removal/addition and source retirement retain committed primary projectile semantics.

Appended FactKind37 and strict28-byte CRT1 cue disclose current target replica/epoch and one-shot identity only, never attacker/damage amount. Team projection requires currently visible and disclosed target; hidden/remembered/forgotten targets rejected. Existing FxImpact discriminator4 carries the cue without ABI layout change; native kind24 renders golden CRIT using the existing64global/4per-owner bounded text pool and existing dedup/expiry/fog/scope/life cleanup. Critical does not duplicate generic impact sound. Actual native CRIT glyph pixels captured, inspected and SHA-bound;66 prior reviews keep historical source/SHA, total67, not67 new views.

Meaningful coverage: strict cue encoding/identity/epoch/overflow/privacy and actual bridge target anchor/discriminator; formal60Hz14native modes including multiplier1, full/partial shields, physical/magic separation, invalid/nested envelope and leech interaction; actual hero primary/visual launch with late modifier change; typed ordinary summon critical surviving exact source retirement, no late reroll/avoided impact. Three focused formal cases pass before full fresh Rust1925/0failed/9existing ignored, Lua11/11. Backend/UE5.8 actual0 and staged DLL/import SHA verified. Native92(91Success/1SuccessWithWarnings,4existing warnings); actual MISS/R/equipment/CRIT PNGs required.

Fullsource3472=3450unchanged+22explicit frozen inputs against strict parentomfue/Saved/BuildTemp/item-leech-source-before-v3.json, indexdocs/acceptance/2026-10-24-item-leech/index.json; own baselineomfue/Saved/BuildTemp/critical-feedback-source-before-v3.json. 759 independent rehashed evidence files and 9 original workers retired with identity-bound actual Windows exits; maximum health gap150687ms. No authored/generator content changes require standalone generation; normal UE build generation preserves all frozen sources including maintained OmWorldBridgeActor.cpp. Errors/prevention:docs/plans/2026-10-24-critical-feedback-errors.md. Original readers/sealer retired before archive closure/source release.

RustV1 core1018passed/1failed preserved with actualWindows1/API51703 reaped. The existing test assumed outer leech; V2 only adjusts wrapper-aware test traversal and adds explicit critical-presence assertions, retaining exact physical/magic/pure/rate checks. All other runtime source files are byte-identical; damage_tick runtime logic is unchanged. V2 core1019passed but focused release compile failed E0599; V3 corrects only the fixture API name to the actual grant_damage_shield. Fresh V3 full acceptance is final; both original failures and exact exits are retained. Recovery-preparer C02 failed before readers because config references repeat in archive coverage; V3 completed the never-executed helpers without changing runtime inputs.

Remaining: sale undo, additional authored critical equipment/content, further geometry/gameplay and combat coverage/art;11external acceptance items/full MVP33/44 remain. No true network match/quota reset/commit/push/omfx. Deadline2026-10-25 00:33:38 Asia/Taipei/omoba-48 unchanged.


# Authored critical equipment COMPLETE

OpenSpec complete-item-critical-chance4/4.

Shared item critical_chance defaults to zero, omits legacy zero JSON, rejects nonfinite/negative/over-one/subquantum and malformed values at model and native custom-catalog admission. Generated Rust catalog and Unreal metadata use the same value. Existing inventory dirty rebuild and new-life finalization share one permanent item lease containing both lifesteal and preattack_criticalstrike; independent ability modifiers remain separate. Chance sums and caps100%. No item/hero identity branches or new critical RNG/multiplier rules.

New shop catalog17精準手套 costs600gold and grants12.5%critical;18精準長刀 costs1800gold,25attack and25%critical using17+sword. Existing items/prices/recipes unchanged. Net chance is offensive for purchase-undo benefit tracking even when flat attack is zero. Sale undo remains unsupported and is not claimed. Native common tooltip prints exact percentage, additive100%cap, existing default2x physical multiplier and avoided/fully shielded hit feedback exclusions. Actual generated critical item hover image inspected and SHA-bound;67 prior reviews retain historical provenance, total68.

Formal production60Hz validates real generated-shop buy/recipe/upgrade undo/sale, pure-chance combat refund rejection, independent modifiers, additive cap, shared lease with lifesteal, guaranteed200physical damage and25actual healing from100base, one settled CriticalHit, no inventory healing, zero-removal and production death/new-life restoration. Fresh Rust1928/0failed/9existing ignored, Lua11/11; backend/UE5.8 actual0, staged DLL/import SHA verified. Native92(91Success/1SuccessWithWarnings,4existing warnings), fresh MISS/R/lifesteal/CRIT/critical-equipment captures required.

Fullsource3472=3456unchanged+16explicit frozen changes against strict parentomfue/Saved/BuildTemp/critical-feedback-source-before-v3.json/indexdocs/acceptance/2026-10-24-critical-feedback/index.json; own baselineomfue/Saved/BuildTemp/item-critical-source-before-v1.json. All37 generated candidates preserved before owned codegen; exact outputs frozen after original retirement. 597 independent rehashed files/8 original workers, actual Windows exits/identity retirement checked; max health gap150641ms. Readers and sealer retired before archive closure. Errors/prevention:docs/plans/2026-10-24-item-critical-errors.md.

Remaining: sale undo, wider geometry/gameplay/combat coverage/art and11external acceptance items; full MVP33/44 remains. No true network match/quota reset/commit/push/omfx. Fixed unattended deadline2026-10-25 00:33:38 Asia/Taipei, automationomoba-48 unchanged.


# User deadline extension

On2026-10-24 the user explicitly extended unattended work through10/26. Active deadline is now2026-10-26 23:59:59 Asia/Taipei, superseding2026-10-25 00:33:38 and older historical reports. Existing omoba-48 was updated through the app tool, and saved configuration read back confirmed ACTIVE, same thread, every3minutes through the new local deadline. No duplicate schedule or model/notification preference change. At expiry stop new work, preserve state/evidence and summarize.

The running sale-undo acceptance continues with immutable runtime sources. Nameplate screen spacing and item-critical are already complete; continue current OpenSpec, then useful single-machine gameplay/presentation gaps. No true network match/quota reset/commit/push/omfx/unrelated termination.


# Shop sale undo COMPLETE

OpenSpec complete-shop-sale-undo4/4.

Authority history now has explicit Purchase/Sale operations with the existing eight-record bound, sequence pinning/deduplication, owner/source generation, shop/phase admission, dirty stat rebuild and retained-wallet binding. Sale records capture the actual credited amount, original slot and exact cooldown bits. Undo reclaims that captured credit and restores the instance only after all balance/shape/source checks pass. No HP/mana/XP/position rewind or cooldown reset; unsupported/malformed/insufficient/occupied/stale attempts leave wallet, inventory and history unchanged. Operation and result slot are committed under ledger hash domainv2; receipt layouts remain unchanged.

Selling keeps the prior monotone policy: all older records close permanently before the new sale record is added. Later eligible purchases can unwind first, then undo that sale; older benefited purchases never reopen. Combat, movement, active item use and death invalidate sale eligibility through existing authority hooks. Admitted dead-roster sales remain independent from the retired body and can undo there or bind onto the actual new-life identity. No item/hero-specific branches.

Native common button/status/result/accessibility/control-setting text describes transaction undo and explains sale debit, item/cooldown restore and no health rewind. Insufficient credit reclaim uses the existing InsufficientGold result6/balance admission reason, with readable UI feedback. Two actual native800/400 shop captures directly inspected and SHA-bound;68 historical image reviews retain their source/provenance, total70.

Meaningful coverage includes atomic rejection for funds/occupied slot/combat/source/cooldown, original versus changed catalog price, zero-credit sale, exact remaining cooldown, later purchase chain, monotone old-purchase closure, movement and commitment operation/slot separation. Production60Hz formal sell/undo inputs execute twice through real queue settlements/private economy facts, duplicate sequence, no HP rewind, retained wallet sale/undo, actual death/new-life binding and deterministic replay. Existing lifesteal/critical sale tests now verify restored shared passives. Fresh same-batch Rust1932/0failed/9existing ignored (V3 sources reused only after all3471other files matched; V4 changes one native capture-test scroll restore), Lua11/11; backend/UE5.8 actual0, staged DLL/import SHA checked. Native92(91Success/1SuccessWithWarnings,4existing warnings); prior MISS/R/lifesteal/CRIT/critical-item and fresh sale-undo captures required.

Fullsource3472=3466unchanged+6explicit frozen changes against strict parentomfue/Saved/BuildTemp/item-critical-source-before-v1.json/indexdocs/acceptance/2026-10-24-item-critical/index.json; own baselineomfue/Saved/BuildTemp/sale-undo-source-before-v4.json. 1397 independent rehashed evidence files/17 original workers; identity-bound actual exits and retirement checked, max health gap150703ms. Original readers/sealer retired before archive closure. Errors and prevention:docs/plans/2026-10-24-sale-undo-errors.md.

Remaining: further geometry/gameplay/combat coverage/art and11external acceptance items; full MVP33/44 remains. No true network match/quota reset/commit/push/omfx/unrelated termination. Existing deadline2026-10-26 23:59:59 Asia/Taipei and automationomoba-48 unchanged.


# Equipment unique passives COMPLETE

OpenSpec complete-item-unique-passives4/4.

Optional shared unique_passive validates a bounded group/name/priority and quantized atk/hp/ms/armor/lifesteal/critical chance. Legacy omission is stable. Generated Rust and Unreal metadata share the same source. Inventory aggregate selects one entire highest-priority passive per group, with lexicographic item-id tie breaking and deterministic group order. Duplicate copies never stack unique effects; ordinary base bonuses remain additive. No item/hero-name runtime exceptions. Existing dirty inventory/new-life paths apply the aggregate without healing or overwriting independent ability modifiers.

Purchase undo now compares actual before/after aggregates, so a suppressed passive creates no phantom benefit and recipe upgrades track the real positive delta. Sale undo keeps its captured-credit/slot/cooldown/monotone policy. Boots now author movement group tier1/25; new catalog19疾步長靴 costs900 including boots component and supplies tier2/45. Same-group boots cannot stack movement. Native shared tooltip publishes passive name, tier, exact bonus and same-group/duplicate nonstacking. Fresh actual native tooltip directly inspected and SHA-bound;70 historical reviews retain provenance.

Coverage includes arbitrary group/id slot permutations, whole-winner selection instead of per-stat maxima, priority/id ties, duplicates/base stacking, multiple independent groups, winner removal/fallback, malformed model/custom catalog atomic rejection, net undo benefits, production60Hz formal purchase/recipe/undo/sale/undo/death/new life/no HP rewind and deterministic replay. Fresh Rust1937/0failed/9existing ignored; Lua11/11; backend/UE5.8 actual0, staged DLL/import SHA verified; native92(91Success/1SuccessWithWarnings,4existing warnings).

Fullsource3472=3456unchanged+16explicit frozen changes; parentomfue/Saved/BuildTemp/sale-undo-source-before-v4.json/indexdocs/acceptance/2026-10-24-sale-undo/index.json; own baselineomfue/Saved/BuildTemp/unique-passive-source-before-v2.json. 688 independent rehashed files/10 original retired workers; actual identity-bound exits checked, max health gap150765ms. Errors and prevention:docs/plans/2026-10-24-unique-passive-errors.md. Original failed cohorts, if any, remain archived without weakening assertions.

Remaining: more independent ground-zone/linear/channel/summon gameplay, equipment exclusive purchase groups/build recommendations/art/balance and11external acceptance items. MVP33/44 unchanged. No true network match/quota reset/commit/push/omfx/unrelated termination. Deadline2026-10-26 23:59:59 Asia/Taipei and existing automation unchanged.


# Equipment purchase groups COMPLETE

OpenSpec complete-item-purchase-groups4/4.

Optional shared purchase_group has validated id/display name/limit1..6, with consistent definitions across a catalog and stable omission for legacy rows. Generated Rust and Unreal share the authored policy. The common atomic buy kernel consumes actual recipe instances and inserts the candidate into a clone before counting group ownership, then commits gold/inventory only when legal. Group limits are independent from unique-passive selection and ordinary attribute stacking. Preview/bot planning shares this kernel. Undo validates its restored inventory against current catalog ownership rules before refund/reclaim/restoration; conflicts remain the existing unavailable/shape reason and do not consume history or sequence.

Existing light boots and tier2 swift boots share authored footwear/鞋靴 limit1. Duplicate base boots or buying base boots alongside the upgrade now decline, while upgrading actual component boots succeeds for600gold. Sale frees the group and newest-first undo can restore the legal previous item. Normal buy rejection appends PurchaseGroupLimit result16; strict40-byte receipt v1 accepts16 only for buy, keeps14/15 undo-only and rejects unknown codes. Bridge reuses this codec and retains owner/epoch/request guards; undo wire/reason layouts remain unchanged. Native common tooltip publishes maximum ownership and post-recipe counting; shared transaction feedback explains the group rejection. Fresh real hover was directly inspected and SHA-bound;71 historical reviews retain provenance.

Meaningful coverage: arbitrary groups with limit2, independent group limits, duplicate rejection without wallet/inventory mutation, valid post-component recipe/full-bag upgrade, malformed/inconsistent custom definitions, read-only preview, atomic undo conflict and recovery without consuming sequence, strict buy-only result16 codec/bridge checks and owner isolation. Production60Hz formal input tests cover duplicate rejection with unchanged history, owner-only rejection receipts, recipe/sale/rebuy/undo, no HP rewind, retained-wallet rejection, actual death/new-life persistence and deterministic replay. The earlier unique-passive formal case now follows the current one-footwear catalog policy; arbitrary passive stacking and whole-winner tests still independently prove that mechanism.

Fresh eight-package Rust1943/0failed/9existing ignored, plus3/3 affected maintained backend item-adapter tests (separate count); Lua11/11; backend/UE5.8 actual0 and staged DLL/import SHA verified. Native92(91Success/1SuccessWithWarnings,4existing warnings).

Fullsource3472=3452unchanged+20explicit frozen changes; parentomfue/Saved/BuildTemp/unique-passive-source-before-v2.json/indexdocs/acceptance/2026-10-24-unique-passive/index.json; own baselineomfue/Saved/BuildTemp/purchase-group-source-before-v1.json. 634 independent rehashed files/9 original retired workers; actual identity-bound exits checked, max health gap150875ms. Errors/prevention:docs/plans/2026-10-24-purchase-group-errors.md. Original failures, if any, remain preserved.

Remaining: independent zone lifetime, additional linear/channel/summon gameplay, more equipment/passive content/build recommendations/art/balance and11external acceptance items. MVP33/44 unchanged. No true network match/quota reset/commit/push/omfx/unrelated termination. Deadline2026-10-26 23:59:59 Asia/Taipei and automation unchanged.


# Independent ground zones COMPLETE

OpenSpec complete-independent-ground-zones4/4.

Shared typed authored zone lifetime defaults to caster-bound and supports independent point/instant circles with validated complete rank arrays. Flame Assault retains its six pulses and death cancellation. Flame Blade retains instant damage and gains four burn pulses over2seconds at0.5second intervals for10/15/20/25 damage by rank; generated Q tooltip states persistence after caster death and same-life refresh.

The private bounded authority book supports128 global/8 per source, checked monotone leases, canonical due-time ordering and exact-source-generation credit/allegiance captured before initial damage. Public status has no credit/lease/script snapshot. Death removes its public mirror without canceling independent lifetime or binding it to a new life. Explicit cancel/refresh revokes already queued old leases. Final expiry pulse settles before retirement on the next positive drain; pause/nonpositive ticks do not advance gameplay time. Managed casts preflight the complete zone batch before resource/cue/effect commitment. Shared ordinary raw-damage/defense/shield/reward settlement is reused, with all attack/critical/leech roles disabled. Private phase/book/lease identity enters deterministic replay state.

Coverage includes strict lifetime schema and compilation, partition-invariant15/60/120Hz due streams, death/remove-all persistence, generation reuse and changed source allegiance, exact expiry, queued cancel/refresh, host-only tokens and unknown leases, capacity/atomic batch admission, nonce exhaustion and private phase diagnostics. Formal production60Hz tests cover all four Q ranks after caster death, pause and four exact pulses plus deterministic replay. Existing bound-zone tests remain.

Fresh eight-package Rust1955/0failed/9existing ignored; Lua11/11; backend/UE5.8 actual0 and staged DLL/import SHA verified. Native92(91Success/1SuccessWithWarnings,4existing warnings).73 reviewed images comprise72 inherited SHA-bound reviews plus one directly inspected real Q tooltip.

Fullsource3472=3450unchanged+22explicit frozen changes; parentdocs/acceptance/2026-10-24-purchase-group/index.json; own baselineomfue/Saved/BuildTemp/independent-zone-source-before-v3.json. 775 independent rehashed files/10 retired original workers; max health gap150656ms. Original failures and remediation are preserved in the archive and docs/plans/2026-10-24-independent-zone-errors.md.

Remaining: further linear/channel/multistage/summon control, content/art/balance and11external acceptance items; MVP33/44 unchanged. No true network match/quota reset/commit/push/omfx/unrelated termination. Deadline2026-10-26 23:59:59 Asia/Taipei unchanged.


# Linear skillshots COMPLETE

OpenSpec complete-linear-skillshots4/4.

One exclusive typed instant point LineDamage effect uses rank-complete shared amount/width metadata and a common deterministic fixed-point clamped endpoint. Its closed capsule includes source/end caps and tangency, does not move its caster or stop on terrain, and applies one raw magic/physical/pure packet per exact live hostile nonbuilding generation. Canonical deduplication and max128 resolved hits finish before output/resource commitment. Zero/unsupported axes and malformed data reject atomically. Endpoint arithmetic uses raw squared distances and integer ceiling square root, preserving valid quantum-sized requests and avoiding coarse normalized-vector error.

Compiled line widths and runtime-loaded widths derive from the actual effect and enter the existing semantic aiming signature. Shared EnemyLine bot policy reads only committed observations, uses the same endpoint/capsule, deduplicates canonical IDs, bounds512 observations/32 candidates and128 hits, then scores focus coverage/count/distance/id deterministically. Normal cost/control/attack-priority admission remains; root does not require a relocation query.

Append-only lumen_ray/貫光射線 and selectable training_line_mage/晨光貫光者 loadout bring the catalog to9 selectable hero loadouts with one new active skill. Existing skills/IDs/loadouts remain. The new Q deals100/150/200/250magic damage, width120, range900, mana45/50/55/60 and cooldown9/8/7/6. Generated shared capsule preview and ordinary tooltip distinguish terrain-piercing nonmovement from terrain-stopped displacement; zero axis has an explicit missing-direction state. Actual generated Q hover directly inspected and SHA-bound;73 prior reviewed images retain provenance.

Coverage: strict Rust/Lua schemas and complete ranks; nonspatial tuning versus semantic width reload signatures; actual generated native metadata; shared endpoint short/long/diagonal/quantum/bounds checks; canonical closed capsule hits/no misleading circle cue, buildings/life/enemy filtering, crowd and malformed-input rejection; bounded bot geometry/permutation/dedup/focus/public target/mana/root behavior. Formal production60Hz tests cover four ranks, beyond-range clamp, exact caps, terrain, root, shield and live defense once, no movement, resource/cooldown admission, pause, invalid targets/controls and deterministic replay.

RustV2 eight-package basis1965/0failed/9existing ignored, plus2 shared-sim focused cases and Lua23/23; V4 reuses these and backendV2 only after complete source proof:3468 unchanged prior inputs, exactly4 changed presentation files and3 new art/provenance inputs. Fresh UE5.8V4 actual0 and staged DLL/import SHA verified. Native93(91Success/2SuccessWithWarnings,5warnings;4existing world-context fixture warnings plus1 aggregate-run D3D12 virtual-reservation warning, preserved with a clean fresh-process NativeSummonPresentation isolation; physical GPU/performance acceptance stays pending).77 reviewed images:73 inherited and4 new (actual Q hover, actual production portrait, original portrait, original skill icon). Original portrait and icon use built-in imagegen, copied byte-for-byte to omfue/Content/Omoba/UI/training-line-mage-v1.png and lumen-ray-v1.png; full exact prompts/source paths/dimensions/SHA in linear-skill-artwork-provenance-v1.json. Native decode covers9 portraits and the new ability resource; dedicated Lua portrait coverage fixtures and actual source readiness also pass.

Fullsource3475=3440unchanged+35explicit frozen changes; parentdocs/acceptance/2026-10-24-independent-zone/index.json; own baselineomfue/Saved/BuildTemp/linear-skill-source-before-v4.json. 1011 independent rehashed files/19 retired original workers; max health gap150641ms. Errors and original evidence:docs/plans/2026-10-24-linear-skill-errors.md.

Remaining: actual moving/colliding/homing projectiles, further channel/multistage/summon control, content/art/balance and11external acceptance items; MVP33/44 unchanged. This instant line does not claim projectile travel/obstacle collision or a full line-skill product set. No true network match/quota reset/commit/push/omfx/unrelated termination. Deadline2026-10-26 23:59:59 Asia/Taipei unchanged.


Postclosure clarification and next concrete increment: docs/plans/2026-10-24-linear-skill-postclosure-notes.md. The sealed console label freshRust is historical; index/source proof and final report explicitly retain unchanged Rust/backendV2 and fresh UE/nativeV4. Next prefer authored offline EnemyLine configuration and full bot-to-formal-input integration, then channel/multistage/summon controls. All current work retired; no new true network matches.


2026-10-24 line-bot completed: optional scripts/lua_data/moba_line_match.lua; actual authored10-seat plan/mana/5Hz cadence through formal60Hz bot line hit, fourranks/disclosure/root/resources/replay. FreshRust1966/0failed/9ignored +focused1 +Lua23 +offlineplan-lock; source3476=3474same+1testchanged+1newrecipe/no deletions. Evidence docs/acceptance/2026-10-24-line-bot/index.json; report docs/plans/2026-10-24-line-bot-final-acceptance.md; errors docs/plans/2026-10-24-line-bot-errors.md. FailedV1 preserved, originalAPI40347 exit1 andV2API58741 exit0 reaped, bothownedworkers retired. No freshUE/backend build claimed; parentlinear-skill native93/77images remains presentation basis. Next concrete work: sustained channel/multistage or summon controls using common typed mechanisms. MVP33/44 and11externalpending, no true network matches/quotareset/commit/push/omfx. Deadline2026-10-26 23:59:59 Asia/Taipei unchanged.


2026-10-24 summon-priority completed: shared ParallelWorldAdapter active MoveTarget precedence for UnitType::Summon, preserving root/non-summon/forced relocation and resume after clear. Actual authored summons/formal60Hz/6 commanded ticks/private script chase/paired digest accepted. FreshRust1968/0failed/9ignored +focused2 +dedicatedbackend/moba-config +UE5.8 +stagedDLL/import SHA; source3476=3474same+2frozenchanges/noadditions/deletions. Evidence docs/acceptance/2026-10-24-summon-priority/index.json; report docs/plans/2026-10-24-summon-priority-final-acceptance.md; errors docs/plans/2026-10-24-summon-priority-errors.md. API70172 actual0 reaped and ownedworker retired. No new native run; native93/77images remains historical presentation basis. Remaining arrival-frame movement-budget sharing, explicit summon command UI, sustainedchannels/multistage/homing projectiles/content/balance and11externalpending/MVP33of44. No true networkmatch/quotareset/commitpush/omfx. Deadline2026-10-26 23:59:59 Asia/Taipei unchanged.

Next concrete increment before channels: close the arrival-frame movement-budget gap. summon_tick removes MoveTarget on arrival, so the current active-destination guard intentionally no longer applies then. Add a general exact-Entity/current-Tick host-movement ownership record (no new hero-name rule or wire/serialized navigation field), assert a destination one unit away cannot be followed by a second script chase in that same tick, then autonomous chase resumes next tick. Preserve this accepted cohort and avoid rerunning completed nameplate/line-bot reviews.

WIP 2026-10-24 summon-arrival: OpenSpec complete-summon-arrival-movement-budget. New private HostMovement (exact Entity+Tick) in summon_tick/shared adapter; actual arrival/resume fixture expanded. V1 core2 passed but formal failed because shared initializer lacked required resource; V1 frozen/logs/receipt retained, original API38377 reaped actual1. After retirement, preserved initialization.rs against parent and wired shared StateInitializer::initialize_resources. Current V2 OWNED acceptance RUNNING, original exec API session28893 must be reaped; do not start duplicates. Jobs omfue/Saved/Acceptance/SummonArrival/accept-v2; work omfue/Saved/Automation/SummonArrivalV2; helper omfue/Saved/BuildTemp/summon-arrival-accept-v2.lua; parent-v2 records4inputs against sealed summon-priority. Source3476 expected3472same+4changes, freshRust1969 expected +focused3, then backend/UE/staging/full source check. Errors docs/plans/2026-10-24-summon-arrival-errors.md. Prepared but NOT EXECUTED sealer summon-arrival-seal-v2.lua requires actualAPI28893 exit0 ledger; close-docs-v1.lua must be adapted to V2/4changes/failedV1 before use. No current completion claim. Every150sec owned health plus manual process/log checks. No true networkmatches/commitpush/quotareset/omfx. Deadline2026-10-26 23:59:59 Asia/Taipei unchanged.


2026-10-24 summon-arrival completed: private phase-local exact-Entity/current-Tick HostMovement record in summon_tick, consumed by shared adapter alongside active MoveTarget. Host arrival cannot gain another script chase step; next tick resumes, generation reuse/stale ticks/root/reset covered. FreshRust1969/0failed/9ignored +focused3 across2commands +actual authored summons/formal60Hz/paired replay +backend/moba-config +UE5.8 +stagedDLL/import SHA. Source3476=3472same+4frozenchanges/noadditions/deletions. Evidence docs/acceptance/2026-10-24-summon-arrival/index.json; report docs/plans/2026-10-24-summon-arrival-final-acceptance.md; errors docs/plans/2026-10-24-summon-arrival-errors.md. V2 API28893 actual0 reaped and ownedworker/waiter retired; V1 API38377 actual1 reaped and failed worker archived. Next parent baseline omfue/Saved/Automation/SummonArrivalV2/source-before.json, frozen inputs omfue/Saved/Automation/SummonArrivalV2/inputs.json. Prior arrival-budget pending is now superseded; no serialized navigation/wire/ABI/UI change or new native automation run. Next prefer actual shared sustained-channel/multistage skill or explicit summon control; do not merely repeat completed nameplate/line/summon checks. Broader projectiles/content/art/balance and11externalpending/MVP33of44 remain. No true networkmatch/quotareset/commitpush/omfx. Deadline2026-10-26 23:59:59 Asia/Taipei unchanged.

WIP 2026-10-24 summon-owner-focus: OpenSpec complete-summon-owner-focus, optional closed nearest/owner_focus summon metadata; authored gunner opts in. Shared adapter prefers only exact live matching-owner HeroCommandQueue active AttackTarget, keeping existing target admission and nearest fallback. Current primary8files +2 generated catalog JSON changes;45 explicit frozen candidates vs3476 parent inputs, expected3466same+10changes/noadditions. Codegen job codegen-v1 completed actual0/direct API; generated class surface/IDs unchanged, catalog data hash refreshed. V1 template focused pass then core test failed because changing owner to enemy made that owner the true nearest enemy; test assertion corrected only, original API6424 reaped actual1 and frozen/logs/receipt preserved. V2 RUNNING original exec API77985 MUST be reaped; do not start duplicates. Jobs omfue/Saved/Acceptance/SummonFocus/accept-v2; work omfue/Saved/Automation/SummonFocusV2; helper summon-focus-accept-v2.lua. Parent summon-focus-parent-v1.json binds sealed summon-arrival and37generated originals. Metadata/core focused tests pass; formal real projectile test then8Rust/backend/UE/staging/fullsource still in progress. FreshRust1972 expected, focused3. Errors docs/plans/2026-10-24-summon-focus-errors.md. Need post-build moba-config catalog hash vs generated JSON crosscheck, then sealer/archive including codegen/failedV1/successV2 and API retirement, docs/audit/tasks. No fresh native run or gameplay completion yet claimed. No true networkmatch/quotareset/commitpush/omfx; deadline2026-10-26 23:59:59 Asia/Taipei unchanged.


2026-10-24 summon-owner-focus COMPLETED: OpenSpec complete-summon-owner-focus4of4. Closed typed nearest(default)/owner_focus metadata, compiled+runtime parity, authored gunner opt-in; shared adapter uses only exact live matching-owner hero active AttackTarget preference before existing summon target admission. Hidden/range/dead/ownerlife/gen/default/cleared command coverage and actual formal60Hz2gunner shots/90damage on focused hero, Stop→nearest body/90damage, paired replay accepted. FreshRust1972/0failed/9ignored +focused3 +codegen +backend/moba-config +UE5.8 +stagedDLL/import SHA +authority/generated catalog crosscheck. Source3476=3466same+10changes/noadditions;45frozen candidates incl37generated; V1→V2 44same+1core-test-only assertion. Catalog IDs/class surface unchanged; data hash 0dad507a24b6b91f. Evidence docs/acceptance/2026-10-24-summon-focus/index.json (259 independently rehashed files/4 retired original workers); report docs/plans/2026-10-24-summon-focus-final-acceptance.md; errors docs/plans/2026-10-24-summon-focus-errors.md. API6424 failed1 and77985 success0 reaped, codegen/catalog direct0, post-sealer API in index.closure. Next parent baseline omfue/Saved/Automation/SummonFocusV2/source-before.json; frozen inputs omfue/Saved/Automation/SummonFocusV2/inputs.json; parent bindings summon-focus-parent-v1.json. Prior WIP superseded, no workers pending. No new native run; prior native93/77 images remain historical presentation evidence. Next prefer actual shared sustained-channel/multistage skill in an existing loadout or improve skill explanation of owner-focus, not repeated completed checks. Explicit summon command UI, broader projectile/content/art/balance and11externalpending/MVP33of44 remain. No true networkmatch/quotareset/commitpush/omfx. Deadline2026-10-26 23:59:59 Asia/Taipei unchanged.

WIP 2026-10-24 summon-guidance: OpenSpec complete-summon-guidance2of4. Shared Lua policy-derived summon guidance appended to authored summoning skill;19Lua cases include defaults/owner_focus/arbitraryIDs/invalid/actualgraphmutation. V1 actual API24880 exit1 reaped/worker retired after allRust1972+backend passed but UE OtherCompilationError exposed raw newline in common escape_cpp_string. V2 common C++ control escaping + regression passed freshRust1973/backend/UE/catalog/source3478; API71284 actual0 reaped and worker retired. V2 is intermediate, not UI completion: tracing found normal skill HUD never reads generated descriptions and upgrade state suppresses legacy payload text (SG03). V3 adds shared registry Description, common ID-matched HUD description ahead of upgrade/cast guidance, native Om.Runtime.SummonGuidance tests+capture and5 related regressions. V3 RUNNING original exec API44120 MUST be reaped; do not duplicate. Jobs omfue/Saved/Acceptance/SummonGuidance/accept-v3; work omfue/Saved/Automation/SummonGuidanceV3; runner summon-guidance-accept-v3.lua; parent summon-guidance-parent-v3.json binds previous summon-focus sealed index+baseline,41 original candidates+2 additions=43 frozen candidates, all37generated included. Expected source3478 relative parent3476 with2added Lua files. Generator header changes intentionally refresh class surface signature; catalog IDs stable, authority/generated data hash must match. Need finish8Rust/backend/UE/staging/offlinecatalog/native6/sourcecheck, inspect actual screenshot, originalAPIreap, archive V1failure+V2intermediate+V3 final sources/receipts/parentproof, final docs/tasks/audit. Errors docs/plans/2026-10-24-summon-guidance-errors.md. No true networkmatch/quotareset/commitpush/omfx/unrelatedtermination. Deadline2026-10-26 23:59:59 Asia/Taipei unchanged.

WIP superseding summon-guidance V3: V3 API44120 actual1 reaped/retired after Rust1973/backend passed, UE C1083 because OmRuntime cannot include OmGenerated. V4 fixes production via identity-bound Description fields in local HUD payload, produced by maintained OmWorldBridgeActor.cpp. V4 Rust1973/backend/UE/staging/catalog pass;5native cases pass then new fixture crashed uninitialized TakeWidget. API35520 actual1 reaped/retired. V5 Initialize fixed crash; exact3477same+1test-only vsV4 proof/backend/UE/catalog pass, but native SummonGuidance failed blank screenshot because discarded Slate root retired HUD. API83236 actual1 reaped/retired; raw report/blank PNG preserved. V6 Initialize+retained WidgetRoot is test-only; RUNNING originalAPI90909 MUST reap. Jobs omfue/Saved/Acceptance/SummonGuidance/accept-v6; work omfue/Saved/Automation/SummonGuidanceV6; helper summon-guidance-accept-v6.lua.44frozen candidates/parent42 incl37directory originals+2newLua;formal3478. V6 inherits Rust1973 from V4 only after exact replacement/fullsource proof; fresh backend/UE/native6/catalog/sourcecheck must finish. No valid pixels/completion claimed yet. Errors docs/plans/2026-10-24-summon-guidance-errors.md SG01..SG07. Need final sealerV6(not staleV3/V4/V5), archive6cohorts expectedexits1,0,1,1,1,0+Rust inheritance+pixelreview+closure, then accurate finaldocs/tasks/audit/handoff. No true networkmatch/quotareset/commitpush/omfx/unrelatedtermination. Deadline2026-10-26 23:59:59 Asia/Taipei unchanged.


2026-10-24 summon-guidance COMPLETED: OpenSpec complete-summon-guidance4of4. Policy-derived summon guidance and common skill descriptions via generated registry→identity-bound local HUD payload→generic BuildAbilityTooltip; no runtime→generated dependency. Common C++ control escaping supports multiline text. V6 Lua19+fresh backend/UE5.8/staging/catalog+native6/0warnings+reviewed1740x1143 three-rank shared-Slate capture. Rust1973/0failed/9ignored inherited explicitly from fresh V4 all8suites: exact test-only Initialize+retained WidgetRoot replacement,3477same+1testsource vsV4,all production/Rust bytes unchanged. Finalsource3478=3464same+12changed+2added vssealedparent,no deletions;44frozen candidates incl37directory originals. Evidencedocs/acceptance/2026-10-24-summon-guidance/index.json(659files); report docs/plans/2026-10-24-summon-guidance-final-acceptance.md; errors docs/plans/2026-10-24-summon-guidance-errors.md. APIs V1=24880/1,V2=71284/0,V3=44120/1,V4=35520/1,V5=83236/1,V6=90909/0 all reaped;6 original workers/waiters retired,previous WIPs superseded,no jobs pending. Registry class surface9660373696db6cb4 intentionally refreshed;catalog IDs stable/datahash7733ace28538bc8b matchesauthority. Next parent baseline omfue/Saved/Automation/SummonGuidanceV6/source-before.json; frozeninputs omfue/Saved/Automation/SummonGuidanceV6/inputs.json; bindings summon-guidance-parent-v6.json. Next prefer actual shared sustained-channel/multistage skill in an existing loadout; avoid repeating completed nameplate/line/summon/tooltip checks. Pet command UI,projectiles/content/art/balance and11externalpending/MVP33of44 remain. No true networkmatch/quotareset/commitpush/omfx/unrelatedtermination. Deadline2026-10-26 23:59:59 Asia/Taipei unchanged.

WIP 2026-10-24 authored-recovery-skill: OpenSpec complete-authored-recovery-channel; proposal/design/spec/tasks exist. Preserve binding omfue/Saved/BuildTemp/recovery-skill-parent-v1.json:52original candidates incl37OmGenerated directory files+15primary, sealed summon-guidance index and baseline omfue/Saved/Automation/SummonGuidanceV6/source-before.json3478. Existing originals copied recovery-skill-old-v1 before edits, builder separately preserved/added before its edit. Current code adds closed AbilityEffect/EffectOp RecoveryChannelSelf keys; strict exclusive channeled/no-target/exact fixed-value rank validation in sharedRust andfixedLua generator. New serde-defaulted AbilityDef.host_execution:Option<HostAbilityExecution> keeps effects_preview non-executable and GameWorld full ABI table unchanged. Generic direct invocation fails closed. Native dispatch stages ScriptRecoveryChannel internaloutcome, atomic settlement rechecks exactowner/life/phase/spec and allows onlyresource/facing/CD mutations; common begin_recovery_channel shares existing item transition andauthorityclock. cast_windup.cancel now removes queued chan­neled SkillCasts via AbilityRegistry metadata, preserving instant/othercaster events. Apprentice_mend existing ID/art/loadout becomes2sec/0.5sec4pulses35/55/75/95,total140/220/300/380,originalmana/cd retained; lumen_mend unchangedinstant. No runtime name branches/magicbuffstartpayload.3authored formal tests added foractualrank/cost/root/pause/pairedreplay,Cast→Stop/Stop→Cast ordering/nocharge/control,dmg/stopboundary. Core metadata+settlement andschemaunit tests added. Native frontend generated metadata NOT regenerated yet; no UE/backend finalacceptance/nativecoverage yet. Need add fixedLua generatorcontract cases/native actualskillmetadata+HUD/channelpresentation after probes, then codegen/fullsource/final8Rust/backend/UE/stage/catalog/native acceptance andarchivefailures/tasks/audit/handoff.
ProbeV1 source snapshots in RecoverySkillProbeV1/inputs.json52files. Schema1passed,core6passed/1failed: newsettlementfixture lacked Factionstorage; onlytestregisterFaction corrected after original API36558actual1 reaped/retired/cleanup. ProbeV2 freezesbeforecompile, schema1/core7pass, releasecompilefailedoldexhaustiveEffectOpfixturemissingnewchannelvariant. OriginalAPI63840actual1 reaped/retired/cleanup. Corrected fixturearm and addeddirectfailclosed/typedmetadata handler test. ProbeV3 RUNNING originalAPI51935 MUST reap; jobs omfue/Saved/Acceptance/RecoverySkill/probe-v3,work omfue/Saved/Automation/RecoverySkillProbeV3,helper recovery-skill-probe-v3.lua.52frozen sourcecandidates capturedbeforecompile; schema/corepassed,handlerreleasecompiling,formal3notrunyet. Do not edit source while readers live or launchduplicate jobs. Errors docs/plans/2026-10-24-recovery-skill-errors.md RC01..RC04. Same-framecancel must actually pass before completion; nativefixturesmust Initialize+retainTakeWidgetroot; OmRuntimecannotimportGenerated. No true networkmatch/quotareset/commitpush/omfx/unrelatedtermination. Deadline2026-10-26 23:59:59 Asia/Taipei unchanged.

WIP authored-recovery-skill superseding probeV3: probeV3 originalAPI51935 actual1 reaped/retired; schema1/core7/typedhandler1 and2formal tests passed, third expected input-list ordering instead of actual ability-phase-before-move-phase. Corrected only test to expect same-tick Stop domination in both list orders and successful next-tick cast; probeV4 API13904 actual0 reaped/retired, all12focused tests pass (1schema+7core+1handler+3formal). No channel-specific phase reordering. Added generic authoring-derived recovery descriptions from actual pulse/duration/interval (arbitrary IDs too), fixedLua test scripts/test_recovery_skill_contract.lua18passed; lumen_mend unchangedinstant. Preserved original OmRecoveryChannelTests.cpp and added native AuthoredRecoverySkill metadata/actualHUD/synthetic2sec public-status presentation+capture. Parent recovery-skill-parent-v1.json now53original candidates (37dir+16primary), addition1Lua=>54frozen; originalcopies immutable. Only OpenSpec/docs/Saved helpers otherwise added. Full accepted baseline previousSummonGuidanceV6/source-before.json3478; expectedfinal3479.
FINAL ACCEPTANCE RUNNING originalexecAPI73537 MUST reap: helper omfue/Saved/BuildTemp/recovery-skill-accept-v1.lua/jobs omfue/Saved/Acceptance/RecoverySkill/accept-v1/work omfue/Saved/Automation/RecoverySkillV1. Lua18+priorSummon19 passed/codegen+freeze54+source-before3479 complete; fullRustabi/content/templates/codegen passed, corecompiling; base/bridge/client/backend/UE/stage/offlinecatalog/publicABIexport/native6/sourcecheckpending. Do not edit sources/startduplicates while workerreads. Nativehelper recovery-skill-native-v1.lua runs AuthoredRecoverySkill,RecoveryChannel,AbilityUpgradePreview,SummonGuidance,OwnedChannelProgress,PublicChannelDuration with actual exported3sec oldsharedABIfixture for PublicChannelDuration; newskillnativecase explicitly models same2sec payload, actualskill execution testedformalRust. Need review capture recovery-skill.png, originalAPI73537actualexit, archive4probejobs expected1,1,1,0 plusfinalaccepted job, allsource/hash/native/rawfailed/provenance/health/retirement, then finaldocs/tasks/audit/handoff. ExpectedRust1980/9ignored = prior1973+schema1+core2+base4; do not assume until logs counted. Nativeexpected6/0warnings; stableIDs/classsurface9660373696db6cb4 shouldremain, intentionaldatahashchange mustmatchauthority. ErrorMD docs/plans/2026-10-24-recovery-skill-errors.md RC01..RC05. No networkmatch/quotareset/commitpush/omfx/unrelatedtermination; deadline2026-10-26 23:59:59 Asia/Taipei unchanged.

WIP recovery channel: acceptV2 API23438 actual1 reaped; full Rust1980/Lua37/backend/UE/native6 all passed but source guard rejected the build-time HUD manifest label projection. RC07 recorded. V3 expands parent55/frozen56 with original manifest reconstructed exactly and SHA-bound to sealed parent; synchronizes labels before freezing. Full V3 concentrated acceptance starts next; do not duplicate or edit formal sources while it runs.

COMPLETED authored-recovery-skill superseding ALL prior RecoverySkill WIPs: complete-authored-recovery-channel6/6. Existing apprentice_mend R now real2sec/0.5sec4pulses35/55/75/95,total140/220/300/380; originalcost/CD/ID/art/loadout retained. Closed typed host_execution, exclusive strict authoring, atomic settlement and shared recovery lifecycle/cancellation/root/pause/healing; no new stableGameWorld method/name branches. Actual formal60Hz costs/pulses/control/damage/Stop/pairedreplay pass. FreshV3 Rust1980/0failed/9ignored +focused12+Lua37+backend/UE5.8/DLL-import staging/offlinecatalog/native6/0warnings; reviewed realmetadata/HUD synthetic2secpresentation image, shared3secactualABIexport fixture separately checked. Fullsource3479 postbuild exact; 3454same/24changed/1added; 761independent archivedfiles withclosure. Evidence docs/acceptance/2026-10-24-recovery-skill/index.json; finalreport/errors docs/plans/2026-10-24-recovery-skill-{final-acceptance,errors}.md. APIs probes36558/63840/51935 actual1,13904actual0;accept73537/23438actual1,82351actual0 allreaped/retired; sealerAPI inindexclosure. Nextparentbaseline omfue/Saved/Automation/RecoverySkillV3/source-before.json,inputs.json,parent recovery-skill-parent-v3.json. NEXT concreteincrement typed bot channel-recovery intent + safe waiting/resumption for item/skill channels using disclosed observations; bots currently instant-only and recall-onlyskip, no channel-awarebotclaim. MVP33/44 and11externalpending. No true networkmatch/quotareset/commitpush/omfx/unrelatedtermination. Deadline2026-10-26 23:59:59 Asia/Taipei.

RecoverySkill final retirement supplement: sealerV4 originalAPI30611 actual0 reaped; originalclosureV4 API25551 actual0 reaped; supplementarypostclosure API11099 actual0 reaped; finaldocwriter directactual0. Index761files independently rehashed; OpenSpec6/6. Additional readonly retirement and helper-overwrite observations preserved separately in docs/plans/2026-10-24-recovery-skill-postseal-errors.md and archiveclosure; original errorMD immutable. Final ledger omfue/Saved/BuildTemp/recovery-skill-final-ledger-v3.json.

WIP bot-channel-wait: complete-bot-recovery-channel-wait3/4; three source files preserved against RecoverySkillV3/source-before3479 and sealed recoveryindex. Owner-scoped private Channels query guards shared role-bot input entry before purchases/sustain/skill/attack/move; no status-key inference, timer or name branch. Formal actual apprentice skill + real purchased item complete pulses under root/pause, resume after finish/Stop, other owner continues, forged presentation buff does not freeze; pairedreplay. ProbeV1 API55237actual1 retired/reaped (itemfixture wrongly bought outside shop), V2API88039actual0 retired/reaped2passed after test-only real-base purchase correction. Errors docs/plans/2026-10-24-bot-channel-wait-errors.md. Next launch finalaccept omfue/Saved/BuildTemp/bot-channel-wait-accept-v1.json; do not edit source while reader active. ExpectedRust1982/9ignored, backend/UE/stage/fullsource3479; no newfrontendpresentation changes/native rerun claimed. Bot channel initiation policy remains pending separately. No networkmatch/quota/commitpush/omfx. Deadline2026-10-26 23:59:59 Asia/Taipei.

BotChannelWait FINAL ACCEPT RUNNING originalAPI85433 MUST reap; jobs omfue/Saved/Acceptance/BotChannelWait/accept-v1,work omfue/Saved/Automation/BotChannelWaitV1,helper bot-channel-wait-accept-v1.lua. Frozen3inputs/fullsource-before3479, abi/content/templates/codegen/core passed; base_content formal suite active and CPU progressing. Do not edit formal sources or duplicate work. Backend/UE/stage/postsourcecheck pending. Sealer bot-channel-wait-seal-v1.lua/closure-v1/final-docs-v1 prepared but not run; after originalactual0 create bot-channel-wait-workers-v1.json list3 entries (probeV1 API55237exit1/probeV2 API88039exit0/acceptV1 API85433exit0) each jobs/work/api/exit_code. Record finalAPI after actualexit only. Sealer expects1982/9ignored/source3476same+3changes and2focused, validatesallretirement/health/parent bindings; archive no new nativepresentation claim. Reap sealer/closure actualAPIs before finaldocs.

COMPLETED bot-channel-wait superseding its prior WIP: complete-bot-recovery-channel-wait4/4. Exact-owner private Channels gate protects all role_bot_inputs branches for existing item/skill channels; no status-key inference/name branches/extra timer/Stop. Actual authoredskill+realpurchaseditem/formal60Hz pulses/root/pause/completion/Stop/otherowner/fakebuff/pairedreplay pass2focused. FreshRust1982/0failed/9ignored +backend/moba-config +UE5.8 +stagedDLL/importSHA; fullsource3479=3476same/3changed,noadds/deletes. Three retired originalworkers probeV1actual1/probeV2actual0/finalacceptactual0; sealerAPI+closure and 107independentfiles in docs/acceptance/2026-10-24-bot-channel-wait/index.json. Report/errors docs/plans/2026-10-24-bot-channel-wait-{final-acceptance,errors}.md. No fresh nativepresentation/Lua/contentgeneration claim; parent recovery evidence bound by fullsource. Nextparent baseline omfue/Saved/Automation/BotChannelWaitV1/source-before.json,inputs.json,parent bot-channel-wait-parent-v1.json. NEXT typed SelfChannelRecovery bot intent, closedactualexecution validation and disclosedsafe initiation; channel initiation stillpending. MVP33/44,11externalpending. No networkmatch/quotareset/commitpush/omfx/unrelatedtermination. Deadline2026-10-26 23:59:59 Asia/Taipei.

BotChannelWait final original API ledger: probeV1 55237actual1 reaped;probeV2 88039actual0 reaped;acceptV1 85433actual0 reaped;sealer65410actual0 reaped;closure/direct0;finaldocs/direct0. Three owned workers retired/cleanup and health verified;107 archivedfiles independent rehash. Final readonly process inventory found no owned BotChannelWait lua/cargo/UE process. Ledger omfue/Saved/BuildTemp/bot-channel-wait-final-ledger-v1.json; no work remains running for this batch.

WIP bot-channel-intent: complete-bot-channel-recovery-intent artifacts created. Preserved6 originals against sealed BotChannelWaitV1/source-before3479, addition scripts/lua_data/moba_channel_match.lua =>7 frozen candidates/expected3480 formal sources. AbilityConst explicit RecoveryChannelConst keys projected in build.rs/runtime_content; startupcatalog and ContentShape hotreload guards. Bot SelfChannelRecovery threshold/radius validates supported compiled class; requires managedmana and actual typed live host spec in cost provider. Ownerhealth +committed disclosed combatantsinclneutral/protectedstructures, inclusive radius,512cap/checkedgeometry; root allowed, priorwait lifecycle retained. Optional10seat apprentice-mid recipe includes actual selected-loadout filtered policies/learning. Tests: metadata1,core2,formalactualplan/fourranks/5Hz→60Hz/cost/root/pause/disclosure/livehostmetadata/budget/wait/pairedreplay1. ProbeV1 originalAPI26592actual1 reaped/retired:metadata/corepassed;inheritedlearning>64. Fixed common-graph selected-loadout filtering (no truncation/hostlimitweakening), addkind6protectedstructure threat/corecase, explicitplanboundassert afterretirement. ProbeV2 RUNNING originalAPI64261 MUST reap; jobs omfue/Saved/Acceptance/BotChannelIntent/probe-v2/work omfue/Saved/Automation/BotChannelIntentProbeV2/helper bot-channel-intent-probe-v2.lua; metadata/corepassed, releaseformalcompiling. Do not edit formal sources/startduplicate. Fullaccept bot-channel-intent-accept-v1.json PREPARED NOT STARTED; expects8Rust1986/9ignored plusbackend/UE/stage/offlinechannelplan/catalog/fullsource3480; source count/rust count must actuallyverify. ErrorMD docs/plans/2026-10-24-bot-channel-intent-errors.md CI01..CI04. Need sealallprobes/fullaccept originalinputs/health/retirement/API andparentbindings, finaldocs/tasks/audit/handoff. No nativepresentation changes/freshnative claim; no networkmatch/quotareset/commitpush/omfx. Deadline2026-10-26 23:59:59 Asia/Taipei.

BotChannelIntent probeV2 originalAPI64261 actual0 reaped/retired/cleanup:metadata1/core2/formalactual4ranks1=4focusedpassed; optionalrecipe selectedloadout filter valid, actual5Hz bot→formal60Hz R/cost/pulses/root/pause/hidden&staleenemy/livehostdecl/budget/wait/cooldown/pairedreplay accepted. OpenSpec3/4; now starting finalaccept-v1, preserveformalinputs while readers run. ExpectedfreshRust1986/9ignored/source3480 (3473same+6changed+1recipeaddition), verifylogs ratherthanassume. Fullacceptincludesbackend/UE/stage/offlinechannelplan/cataloghash. Sealer/finaldocs notyetprepared.

BotChannelIntent FINAL ACCEPT RUNNING originalAPI12982 MUST reap; jobs omfue/Saved/Acceptance/BotChannelIntent/accept-v1/work omfue/Saved/Automation/BotChannelIntentV1/helper bot-channel-intent-accept-v1.lua. Frozen7/fullsource-before3480; abi/content/templates/codegen/corepassed, fullbase_content CPU progressing (long3laneformal fixture), bridge/client/backend/UE/stage/offlinechannelplan/catalog/postsourcecheck pending. No source mutations or duplicatejobs. Sealer bot-channel-intent-seal-v1.lua expects3workers(probeV1 API26592actual1/probeV2API64261actual0/finalAPI12982actual0), freshRust1986/9ignored,focused4,3473same/6changes/1addition, exact7finalinputs equalprobeV2 and3declaredrepairs betweenprobes. Record finalAPI onlyafteractualreap using bot-channel-intent-record-accept-v1.lua, then seal. Closure-v1/final-docs-v1 prepared but notrun; recordsealeractual0 JSON beforeclosure, reapsealer/closure actualAPI, thenfinaldocs/ledger/CIM. Errors CI01..CI04 immutable aftersealing. Nextaftercompletion: player-owned summon selection/commands UI audit.

COMPLETED bot-channel-intent superseding its prior WIP: complete-bot-channel-recovery-intent4/4. Closed SelfChannelRecovery with compiled/runtime RecoveryChannelConst projection, startup/reload semantics guard, actuallivehostspec+mana admission, disclosed bounded hostile geometryinclneutral/protectedstructures; no unknown-channel relaxation/privateenemy reads/name inference. Optional scripts/lua_data/moba_channel_match.lua ten-seat apprentice-mid/mana/65%/radius1000; sharedauthoring-graph selected-loadout policy/learning filter preserves64cap. Fourrank actual5Hz→formal60Hz R/cost90..120/pulses35..95/root/pause/budget/hostdeclaration/disclosure/wait/CD/pairedreplay passed; fixtureisolateslearning/shop/sustain priorities. FreshRust1986/0failed/9ignored +focused4 +backend/moba-config/UE5.8/DLL-importSHA/offlineplan/cataloghash; fullsource3480=3473same/6changed/1recipeadded,nodeletes. Evidence docs/acceptance/2026-10-24-bot-channel-intent/index.json; 139independentfiles and3retiredworkers inclfailedV1. Report/errors docs/plans/2026-10-24-bot-channel-intent-{final-acceptance,errors}.md. No freshnativepresentation claim; parentUIbasis unchanged. Nextparent baseline omfue/Saved/Automation/BotChannelIntentV1/source-before.json,inputs.json,parent bot-channel-intent-parent-v1.json. NEXT concrete audit/implementation player-owned summon selection and command UI using existing exact-owner backend movement/focus; thenprojectiles/multistage/content/art/balance. MVP33/44,11externalpending. No networkmatch/quotareset/commitpush/omfx/unrelatedtermination. Deadline2026-10-26 23:59:59 Asia/Taipei.

BotChannelIntent final original API ledger: probeV1 26592actual1 reaped;probeV2 64261actual0 reaped;acceptV1 12982actual0 reaped;sealer86535actual0 reaped;closure/direct0;finaldocs/direct0. Three owned workers retired/cleanup and health verified;139 archivedfiles independent rehash. Final readonly process inventory found no owned BotChannelIntent lua/cargo/UE process. Ledger omfue/Saved/BuildTemp/bot-channel-intent-final-ledger-v1.json; no work remains running for this batch. Before next module path lookup use rg --files on verified containing dirs (CI05); do not infer directories from Rust module names.

WIP owned-summon-authority: complete-owned-summon-movement-authority1/4. Parent summon-orders-parent-v2.json binds BotChannelIntentV1 baseline3480 and sealedindex;10existing originals preserved +1newmodule. ProbeV1 API31924 actual1 reaped (exhaustive renderer conversion); frozen/raw evidence retained. V2 freezes11 candidates, runs core shape+renderer denial, secure ingress denial and actual formal summon tests. New action is currently formal-driver-only; renderer/secure ingress/canonical network acceptance explicitly reject until future scoped transport. No UI control availability claim. Need reap original V2 API, resolve fixtures, then full maintained Rust/backend/UE/staging/source acceptance and seal/tasks/audit.

OwnedSummon probeV3 API64000 actual0 reaped/retired: core2/backend1/formal2 pass. V2 API36342 actual1 reaped: negative foreign-summoner fixture accidentally enabled legitimate autonomous follow; V3 isolates only negative fixture navigation, positive follow/channel/replay unchanged. Next finalaccept-v1 freezes11 inputs/fullsource3481 and runs8maintained Rust/backend/UE/stage/catalog/sourcecheck; expected1990/9ignored mustverify. Sealer-v2 expects4workers and1test-onlyV2→V3 sourcechange; finaldocs-v2 prepared notrun. Do not edit source while final reader active.

COMPLETED owned-summon-authority superseding prior WIP: complete-owned-summon-movement-authority4/4. Separate exact owner/summon entity generations and closed MoveTo/Release, bounded32/player/tick, roster/life/faction/control/terrain validation before mutation; existing navigation/attack interrupt/arrival budget/autonomy retained, no hero command/cast/channel cancellation. Actual Mago summons/formal60Hz +mixed real recovery fixture +negative cases/flood/paired replay passed. FreshRust1990/0failed/9ignored,focused5,backend/moba-config/UE5.8/stagedDLL-import/offlineplan/catalog;3481source=3470same/10changed/1newmodule,nodeletes. Index docs/acceptance/2026-10-24-summon-orders/index.json (179 files) seals failedV1/failedV2/successV3/finalaccept/health/retirement/originalAPIs. Errors/report docs/plans/2026-10-24-summon-orders-{errors,final-acceptance}.md. Nextparentbaseline omfue/Saved/Automation/SummonOrdersV1/source-before.json/inputs.json,parent summon-orders-parent-v2.json. NEXT UE owned summon selection/command UI +negotiated scoped actor transport: CURRENT renderer/secure coordinate/canonical network acceptance explicitlyreject newaction; no player-operable summon UI claim or newnativepresentationrun. MVP33/44,11externalpending. No networkmatch/quotareset/commitpush/omfx/unrelatedtermination. Deadline2026-10-26 23:59:59 Asia/Taipei.

OwnedSummon final original API ledger: probeV1 31924actual1 reaped;probeV2 36342actual1 reaped;probeV3 64000actual0 reaped;acceptV1 79193actual0 reaped;sealerV2 37255actual0 reaped;closureV2/direct0;finaldocsV2/direct0. Four workers retired/cleanup/health verified;179 independent archived files rehashed. Final readonly CIM found no matching owned lua/cargo/rustc/UnrealEditor-Cmd worker; UE reusable MSBuild nodemode/nodeReuse servers identified separately and left intact. Ledger omfue/Saved/BuildTemp/summon-orders-final-ledger-v1.json. No current batch job remains running.

WIP summon-wire: complete-owned-summon-secure-transport. Parent summon-wire-parent-v1.json binds sealed summon-orders179files/baseline3481. Preserved12 originals +new shared summon_transport module. Distinct client SummonCommandInput tag30 carries2opaque refs; secure14 edge validates session/player/team/epoch/input-tick then seals formal29, actual authority retains exact owner checks. Canonical ACK29 redacts identities; projector/replicas reject leaks/aliases and Specs treats ACK notification-only, checks owned Hero+owned kind4 recipient. Client host constructor uses current own hero/summon disclosures. No renderer intent/UI exposure; presentation11 unchanged. ProbeV1 freezes13 candidates and runs core4/client1/backend1/formal1; need actual original API reap, full8Rust/backend/UE/stage/source3482(expected1996/9ignored) then seal/docs/tasks. No source mutation while readers active.

SummonWire WIP update: probesV1 API63933/V2 API96756/V3 API42000/V4 API81827/V5 API63042 all actual1 reaped/retired, immutable inputs/raw logs preserved. SW02 explicit lower formal client-action rejection; SW03/SW04 exact advertised snapshot and frame replica ticks; SW05 include materialized public defaults for canonical hash; SW06 external fixture uses public baseline encoder. Runtime acceptance was not weakened. V6 original API40187 RUNNING MUST reap; jobs omfue/Saved/Acceptance/SummonWire/probe-v6,work omfue/Saved/Automation/SummonWireProbeV6,helper summon-wire-probe-v6.lua. Core4/client1/backend1 passed; real formal release compile active. Parent-v2 now13existing +1newmodule=14 frozen candidates; expected3482source/3468same13changed1added. Fullaccept-v1 PREPARED NOT STARTED; expected8Rust1996/9ignored. Sealer-v2 expects7workers and V2→V5 onlynew-module test repairs,V5→V6 onlyclient-test repair; finaldocs notyetprepared. No renderer intent/UI exposure; selective14,presentation11.

SummonWire probeV6 API40187 actual0 reaped/retired: core4/client1/backend1/formal1 pass, actual authored summons move via sealedrefs and reject changedplayer Release; projection Applied/hash/no-second-move and pairedreplay pass. OpenSpec3/4. Starting fullaccept-v1; originalAPI mustreap. Frozen14/fullsource3482; expectedfreshRust1996/9ignored +rawcanonicalnegative1 (focusedtotal8). Sealer-v2 expects7workers, verifies13originalchanges/1moduleaddition and all immutable probes inclV5 clientcompilefailure. No formal source mutations while full reader active.

SummonWire FINAL ACCEPT RUNNING originalAPI50714 MUST reap; jobs omfue/Saved/Acceptance/SummonWire/accept-v1,work omfue/Saved/Automation/SummonWireV1,helper summon-wire-accept-v1.lua. Frozen14/fullsource3482 capture active; expectedfresh8Rust1996/9ignored +rawcanonicalnegative1/focused8/backend/UE/staging/offlineplan/catalog/postsourcecheck. Do not mutate formal source or duplicatejobs. After actual0 call summon-wire-record-accept-v1.lua 50714, then summon-wire-seal-v2.lua (7workers) and reap its originalAPI; write summon-wire-seal-api-v2.json actual0 before closure-v2, then final-docs-v1. Sealerproof validates V1→V2 extra tick boundary only; V2→V5 newmodule tests only; V5→V6 client encoder fixture only. Finalreport/docs prepared notrun. ErrorsMD SW01..SW07 remain unsealed; no nativeUI claim.

COMPLETED summon-wire superseding all prior WIP: complete-owned-summon-secure-transport4/4. Client-only opaque2reference SummonCommandInput tag30→secure session/player/team/view/input-tick sealing→existing exact-owned formal29. Raw canonical ingress denied. Canonical ACK29 carries projected actor/target and fullyredacted payload; projector/replicas reject leaks/aliases/clientform. Specs ACK is notification-only with owned Hero+Summon check, no private summon behavior/second movement. Client host constructs only current own hero/kind4 summon. Selective14 rejects old13;presentation11/no renderer intent. Core4/client1/backend2/actual Mago summon formal60Hz1+pairedreplay passed; fresh8Rust1996/0failed/9ignored/backend/moba-config/UE5.8/DLL-import staging/offlineplan/catalog. Source3482=3468same/13changed/1newmodule,no deletes. Evidence docs/acceptance/2026-10-24-summon-wire/index.json (294files); errors/report docs/plans/2026-10-24-summon-wire-{errors,final-acceptance}.md. Nextparentbaseline omfue/Saved/Automation/SummonWireV1/source-before.json/inputs.json,parent summon-wire-parent-v2.json. NEXT typed UE/bridge summon issuing and selection/control feedback; current converter explicitlyunsupported, UI stillpending/no freshnative claim. MVP33/44,11externalpending. No networkmatch/quotareset/commitpush/omfx/unrelatedtermination. Deadline2026-10-26 23:59:59 Asia/Taipei.

SummonWire final API ledger: probesV1 63933actual1,V2 96756actual1,V3 42000actual1,V4 81827actual1,V5 63042actual1,V6 40187actual0;finalaccept50714actual0;sealer40092actual0;closure66930actual0,all yielded originalAPIs reaped. Finaldocs/direct0. Seven ownedworkers retired/cleanup/health verified. Closure independently rehashed294files and proved shared transport production prefix unchanged across all6probes and client production text unchanged outside the repaired constructor fixture. Final readonly CIM found no matching owned lua/cargo/rustc/UnrealEditor-Cmd worker; reusable UE MSBuild nodeReuse servers left intact. Ledger omfue/Saved/BuildTemp/summon-wire-final-ledger-v1.json. No batch work remains running.

WIP summon-ui: complete-owned-summon-ue-controls. Parent summon-ui-parent-v1.json preserves57 existing candidates incl37generated/ABIheader against sealed SummonWireV1/source3482, plus5 additions (Rust summon module, C++ model h/cpp/native tests, Lua bridge build). New typed ABI36 and presentation12 renderer intent31 reuse opaque SummonCommandInput; selective14 unchanged. Clientgateway exact current refs/capability; bridge current live owner/summon scope guard; UE selected exact-owner commandability/model, configurable Alt+RMB/Alt+S migration-safe optional rows, HUD actual binding guidance, no hero cancel. New active bridge build Lua replaces historical batch calls; do not execute old batch/PowerShell path. Native3 cases planned typed aliases/actual published selection+Slate+capture/profile collision migration; no new real networkmatch. Pre-build/source codegen probe starts next, preserve all failures and sourcecohorts; final UI acceptance notyetclaimed.
