# WASD 權威輸入與英雄個別設定

動態鏡頭／Scout 與精簡設定範圍接受（2026-10-14）：`control-lookahead-build-v3` ordinary no-cache 成功9.98秒；`worker-pool-machine-v1/control-lookahead-native-v3` D3D12 exact28 Success／0warning／0fail／0notRun／真exit0；`control-lookahead-sources-v3` 35來源build/native前後一致。鍵盤模式首次 dynamic follow、真 cursor／viewport projection 偏移、M3 Scout 非 drag、Space 優先及恢復、最後放開恢復精確 free view、modal／小地圖／hero不可用退休均由新增 CameraProjectedDrag 斷言驗證。相機 offset 依四角 focus-plane footprint 的內切半徑有界計算，dynamic平滑不延遲英雄基點，兩種模式互不套用。schema9 additive optional camera fields 保持舊檔相容，roundtrip／錯型別原子拒絕／範圍驗證成功；模式／編輯範圍按鈕改 wrap 並停用不可提交選項，當次設定圖已實際目視。E1064 的 fixture paint 與 E1065 的正式首事件 observer 缺口已修復，原兩輪失敗保留；v1冷建置454.77秒／v2 12.83秒為歷史。此項未涵蓋 keyboard owner 的真 Slate M3／WASD→wire、HUD 遮擋保護／accessibility／public route、完整DPI與權威結果；不勾父項、不冒稱全套完成。接 owned IPC 正向方向事件與生命期，無新 authority game／ACK。

共用 keyboard mouse／camera 與繼承修正範圍接受（2026-10-14）：`control-pointer-camera-build-v4` ordinary no-cache成功19.65秒，`worker-pool-machine-v1/control-pointer-camera-native-v3` D3D12 exact28 Success／0warning／真exit0，`control-pointer-camera-sources-v4` 34來源build/native前後一致。真正Slate RMB進CastQ／不走Move，release清同一physical fact，contextual left走AttackOnly分類且離線零admission；RightShift佇列不擋Space，修改修飾鍵後release由共同held facts退休；既有camera／drag／minimap／world route／rebind回歸通過。Global parent變更以同一SeedKeyboard rebase真mode自訂，default reserved roles維持、明確mode衝突整份原子拒絕；unknown reset無patch也拒絕。E1057–E1062與兩輪失敗保留；LayoutPlayers修正共用fixture，production pointer guard未放寬。v1 cold build695.10秒／v2 10.93／v3 37.93明示歷史；active PCH唯讀預讀26.5秒非SHA完整性驗證。此範圍仍不等於真正owner WASD→wire／正向hostile contextual attack／Scout／dynamic／accessibility／public route或完整驗收，無新authority game／ACK。下一步精簡設定範圍UI與動態camera／Scout，然後補owned IPC方向事件。

設定編輯增量接受：`control-profile-editor-build-v4` ordinary no-cache 成功18.98秒；`worker-pool-machine-v1/control-profile-editor-native-v3` D3D12 exact19 Success／0warning／真exit0，`control-profile-editor-sources-v4` 28來源在build/native前後一致。全域／模式／當前英雄編輯範圍、獨立模式要求、45度偏好、Shift primary實際Slate capture、逐組按鍵reset與全域scalar/audio reset已驗；inactive edit不重裝active profile。E1049–E1052失敗與修正保留；DPI量測改讀真正BindingColumns、BindingRow仍統一filter與geometry，原搜尋／hit／文字縮放斷言全過。尚無真正owner WASD→wire、keyboard MB2／情境左鍵／camera role與Scout／dynamic／accessibility全範圍證據，不勾父項、不宣稱authority ACK。下一步共用physical入口與mouse/camera仲裁。

keyboard修飾鍵／實體held狀態增量已接受：`control-chord-owner-build-v2` ordinary成功97.87秒；`worker-pool-machine-v1/control-chord-owner-native-v1` D3D12 exact19全Success／0warning／真exit0，`control-chord-owner-sources-v2` 28來源build/native前後不變。新resolver明確組合鍵優先，再按keyboard primary ability／queue角色去除隱含modifier；預設Ctrl升級／Alt自施依真正技能鍵映射，point模式原Shift override保留。唯一preprocessor呼叫既有已安裝delegate，執行前copy以防callback重建array，不另寫技能router。keyboard held／world-modifier使用同一physical model與同事件scope，真正Slate Tab按下→owner tick維持→放開關閉成功；Recall legacyqueue退休及修飾鍵純模型已驗。v1 build199.27秒成功但E1048審查後未跑native，不作接受。此範圍尚未涵蓋真正owner WASD→wire、MB2新技能切換／情境左鍵、camera role dispatch、mode/profile selector、Scout／dynamic／accessibility與public offscreen route；不能因此勾完3.10。無新authority game／ACK。

唯一 owner／分層儲存接線增量已接受：`control-profile-owner-build-v1` ordinary成功213.69秒，`worker-pool-machine-v1/control-profile-owner-native-v1` D3D12 exact18 Success／0warning／真exit0，`control-profile-owner-sources-v1` 25來源build/native前後不變。真正 live owner 設定儲存、inactive mode edit不變active revision、英雄ID大小寫隔離與切換、wrapper檔案readback、舊point-mode設定與實際Slate UI回歸成功。現有設定頁現在讀取editable profile，運作中的輸入／相機仍讀active profile；只有scope／mode／hero或active內容變更才安裝，frame熱路徑不重讀JSON／磁碟。local-admission delegate統一讓Stop／Hold／即時move／attack-move／模式／Recall退休舊方向，AttackTarget保留。此增量沒有actual owned WASD按鍵→wire證據，也沒有主修飾鍵／情境左鍵／mode selector／Scout等完整UI，因此玩家keyboard mode仍不列為完成；無新authority game／ACK，父項不勾完。

方向按鍵與 profile-set 純模型增量已接受：`control-profile-stage-build-v4` ordinary NoEngineChanges／NoUBA／nocache 成功13.86秒，`worker-pool-machine-v1/control-profile-stage-native-v3` D3D12 exact14全部Success、0warning、真exit0；`control-profile-stage-sources-v4` 23來源在build與native前後不變。涵蓋 bounded heartbeat／真正新按壓／漏掉OS放鍵後重入／45度／counter上限、schema9與舊版遷移、三層稀疏繼承、完整binding identity／held press-release reset、case-sensitive英雄ID隔離、嚴格patch欄位／authority與UIaudio拒絕，以及實際檔案read-only原子替換失敗保留bytes。E1043–E1046修復與失敗log全部保留。此範圍尚未把profile-set接入owner/UI，也未做actual玩家WASD輸入或新authority game／ACK；父1.4／3.10仍pending，下一步接唯一owner與設定介面。

UE typed owner-control 增量已接受：ordinary `owner-control-native-build-v1` 真退出成功，64 actions、1325.19 秒，NoEngineChanges／NoUBA／nocache，ABI28 header 與 staged bridge 由正式流程生成。`worker-pool-machine-v1/owner-control-native-v1` D3D12 原生 exact8 全部 Success、0 warning、exit0；`owner-control-native-sources-v1` 證明 native 執行前後16來源不變（此 fingerprint 不涵蓋之前 build 時段）。驗證 command strict aliases／完整u64／frame owner與source／128字元英雄ID／JSONL及原設定持久化回歸。沒有新 authority game／ACK；玩家 WASD 與 profile set 仍在實作，1.4／3.10不勾完。

ABI28增量已接受：`.ai-collab/input-scheme-owner-control-abi-v5/summary.json` 真退出成功，core779／0ignored、client143／8既有ignored、bridge124／1既有ignored、backend wire2及五項all-target checks全過，40來源SHA前後相同。真正cbindgen header-v3與staged byte identity成功、MSVC maintained layout-v1成功；E1032–E1037修正由此輪涵蓋。現在接Unreal inline roster consumer與typed command builder，新增原生測試尚未編譯執行；玩家WASD與profile set仍pending，1.4／3.10不勾完。

Codex直接實作，不使用子代理。沿用 complete-moba-controls-and-presentation 3.10／1.4。點擊模式保持預設；現有 bWasdCameraProfileEnabled 只控制鏡頭，不充當英雄移動。

1. 先加入有型別的 input scheme／match rules：預設對局開始後600秒鎖定模式，使用權威 Fixed64 時鐘；模式存在 owner roster，死亡／重生／重連不重設。只有 authenticated unique owner 可更新，整批拒絕非法模式／時間／phase，不拿本機時鐘決定鎖定。
2. 連續方向獨立保存於有期限的 source-life lease。signed axis 嚴格範圍、整數歸一化，對角線不加速；neutral／失焦／死亡／模式切換退出，未收到後續更新則權威300ms到期。owner偏好持續，held方向不跨重生。不得靠client遠端MoveTo連發或teleport。
3. 方向進正式 fixed-tick movement／相同 public terrain collision／root與channel／attack仲裁；先驗production consumer，再接 authenticated wire、filtered projection、hash、bootstrap／rebase。完整合法投影包含穩定英雄內容ID，不能因死亡遺失角色設定選擇；未知／隱藏資料不補查世界。
4. profile使用全域基底＋英雄內容ID／模式的稀疏override，保存與衝突驗證原子化。全域UI／音量偏好不因換英雄回退；角色未覆寫欄位繼承全域後續更新。設定頁明確區分全域／當前英雄，改模式使舊held／aim退休。
5. WASD mode採獨立active bindings與HUD labels，補dynamic camera、Scout Ahead、45度方向、無障礙游標與joystick映射；必要移動鍵缺失拒絕。現有點擊模式S／D／C／Shift+A不直接混入WASD，兼容同鍵依明確context仲裁。
6. 每個增量先Rust與實際consumer，再普通NoEngineChanges／ABI／真Slate與owned events；原10/10對局quota不重設，fixture不冒稱權威ACK。完整414-case與全專案／DPI／效能驗收最後，尚缺證據的父項保持pending。

目前開始步驟1–2的名冊與決定性核心；這不是WASD已可由玩家使用的成功宣稱。

基礎驗證：input-scheme-foundation-v1 core740／0ignored與core、client、bridge、backend、base_content五個all-target checks全成功，四來源前後SHA相同。六項新測試涵蓋真名冊constructor／unique owner／非法更新／自訂鎖定邊界／clone持續／死亡與真replacement／digest，以及tiny/full-scale axes、不加速、lease邊界與溢位原子拒絕。

連續方向的正式戰鬥決策：明確指定攻擊保留合法target，前搖與impact期間停步，冷卻／後搖允許按住方向移動；點擊模式維持既有行為。方向本身只生成權威每tick的近端movement waypoint並走原collision，不呼叫A*或client MoveTo。新方向press會在正式router退休舊point／Hold指令，同向heartbeat只續lease、不反覆取消攻擊。Recall／Stop／Hold及focus neutralization一併接router，不能只完成本機按鍵。

正式 movement consumer 增量驗證：input-scheme-consumer-v2 core745／0ignored＋五正式 all-target checks 全成功，六來源 SHA 前後相同。新增五項 production consumer 測試實際涵蓋無 point queue 的真出生英雄、八方向與滿幅輸入速度上限、neutral／精確 lease 到期退休舊 waypoint、薄牆 swept collision 與 Y 滑動、Root／前搖／impact 停步與冷卻恢復，以及合法可見指定攻擊／隱藏目標退休／超出攻擊距離／不提前推進 attack clock。v1 借用錯誤與修復見 E1009，原失敗不覆寫。這是 Rust 權威 movement consumer 的成功，尚未接 authenticated wire／filtered replay／Unreal 玩家入口，不能勾完3.10。

按壓生命期與正式既有命令增量：input-scheme-gestures-v1 完整 core750／0ignored＋五組正式 all-target checks 成功，七來源 SHA 前後不變。Begin 使用當前英雄生命期遞增 press id；Update 只能延長未退休的同次按壓，普通 lease timeout 可恢復，Stop／Hold／MoveTo／AttackMove／Recall／模式切換／死亡的退休不可由舊 heartbeat 撤銷；End 必須匹配最新 press，不會清掉後續新按壓。新 World admission 先驗 owner／Hero／HP／generation／phase／pause，再一次取消舊 queue／recall／攻擊動畫；Update 不重複取消。新增四項完整測試，含真 command drain、真 recall pre-dispatch 與暫停時仍可 matching End。上述尚未包含新 wire／UE 入口。

另修復 E1011 CombatPreferences 漏接 MOBA pre-dispatch allowlist：combat-match-input-v2 真入口→dispatcher→roster 測試與五 checks 成功，完整 core750 再涵蓋該測試。先前 combat-settings-accepted-v1 的 UI／transport scoped 證據保留，但不單獨證明真對局套用；仍無新 authority game／ACK。

名冊網路投影採獨立 typed InputSchemeOwnerState，附在既有完整 owner preference row，而非建立第二份生命期不同的名冊。缺該 nested capability 仍可有 combat preferences，但不得推定 mode／hero profile 可用。schema／非法 mode／內容 ID／權威時間與規則／press high-water 整批驗證，沿用外層 own-team／committed opponents 範圍與 epoch／team hash／manifest。selective11、renderer9 拒絕會丟失新雜湊欄位的舊 peer；新欄位不含 held 方向，尚未代表 directional wire 可用。

owner projection v1 已實際驗證：core753／0ignored、client140／8既有ignored、bridge120／1既有ignored及五組 all-target checks 成功，21來源 SHA 前後不變。未跑新 Unreal／ABI／玩家入口；後續新 mode RPC／active press 欄位需fresh重驗。先前 protocol10／renderer8 接受紀錄是歷史範圍，不改寫舊證據。

mode-wire-v1 已實際驗證：core756／0ignored、client141／8既有ignored、bridge120／1既有ignored、backend authenticated mode test1與五組all-target checks成功，25來源SHA前後相同。typed mode RPC已走正式pre-dispatch，包含warmup／pause／死亡、權威鎖定邊界與idempotent重送；actorless ACK整批驗payload與owner capability。這仍未接C ABI／UE設定頁，亦不是方向輸入已接網路。

E1016修正中的正式設計：新方向Begin只退休導航／recall，保留攻擊clock；keyboard windup記住host-local精確Entity生命期，impact只允許原目標，不從附近敵人重新取得。該欄位serde skip，authority replay digest另外綁定；隱藏／死亡／換生命期／超射程會空發並正常進後搖，不重設ready clock。當前正在fresh commitment-v1 gate，結果尚不可宣稱完成。

commitment-v1 已全部退出並成功：core757／0ignored、client141／8既有ignored、bridge120／1既有ignored、backend mode1與五組all-target checks通過，29來源SHA前後相同。五種真HeroTick目標情境、方向Begin前搖clock／sequence不變、impact前零移動、後搖恢復移動、serde拒絕注入私有target均已驗。E1016的取消行為已修正；上述gesture-v1「取消攻擊動畫」是被此增量取代的歷史行為，不再作為正確規則。仍不是UE WASD入口完成。

step-snapshot-v1 已全部退出成功：core760／0ignored、client141／8既有ignored、bridge120／1既有ignored、backend mode1與五組all-target checks通過，29來源SHA前後相同。真 pre-dispatch→movement→Stop→次tick與pause、唯一live owner／body、recall neutral，以及私有attack commitment／step digest binding已驗。接下來 step wire 使用 selective12，renderer仍9／ABI仍27；新 owner nested step 與 filtered movement consumer 正在實作，還沒有方向按鍵入口。

step-wire-v1 原763成功／1完整fixture失敗保留；client141、bridge120、backend mode1與五checks全成功，29來源凍結不變。E1020補齊完整body，E1021另加attack_reset權威旗標、模式切換重播使用正式interrupt政策；Direction Begin只有navigation_reset，禁止attack_reset而沒有navigation_reset。兩種reset各一次，pause時延至第一個active step，source死亡／換生命期不把舊快照附給新body。protoc-step-v3已成功，正在fresh step-wire-v2重驗；此處尚非WASD玩家入口接受。

step-wire-v2 已真退出成功：core767／0ignored、client141／8既有ignored、bridge120／1既有ignored、backend mode1、五checks成功，29來源SHA相同。開始 source-only direction typed契約：分開client reference／authority seal／redacted ACK validator，session resolver驗當tick公開且owned的精確生命期。此增量尚不廣告新direction capability，避免先開入口再補安全邊界。

direction-edge增量已接正式PlayerInput tag28與owner direction_protocol tag10，selective升13；renderer仍9、ABI仍27。authenticated InputSubmit edge驗session/player/team/view/disclosure/input tick，再移除opaque source並封存exact life；legacy或client seal拒絕。authority prehook按mode/direction原順序處理，暖場首次active tick先進Playing，只有成功Begin阻止同batch Recall，Update/invalid Begin不阻止，paused/zero delta只准matching End。ACK擦除source/seal，filtered以snapshot移動而不重放ACK。新core與client gates已通過，整輪尚待bridge/backend退出；renderer與UE未開方向入口，不能宣稱玩家可用。

direction-edge-v3全程真退出成功：core777／0ignored、client141／8既有ignored、bridge120／1既有ignored、backend wire2與五checks成功，31來源SHA一致。E1026補owner+Hero驗證、sealed leak與kind alias在preflight拒絕，full frame→Specs方向ACK仍只走一次並與權威hash相同。E1027/E1028原失敗保留。接renderer10 typed Direction intent（tag30），只驗唯一current source與capability，不建立第二份press狀態；ABI仍27、UE入口未實作。

direction-renderer-v1已全程真退出成功：core778／0ignored、client143／8既有ignored、bridge120／1既有ignored、backend wire2及五checks成功，35來源SHA一致。Lua owned observer115契約成功；C++ JSONL新增原生測試尚未UE實跑，不能當作native接受。已驗client唯一own source、公開別人source拒絕、重複owner／缺cap／point mode／inactive／非法寬整數與request不配置ID；高延遲下active_press仍0，Update／End仍傳至authority；paused只放End。renderer10／selective13／當時ABI27。source/press觀察欄位用canonical u64 decimal string避免double精度損失。

現在實作ABI28：獨立OmOwnerControlCommand帶current connection/view/source/press/phase/signed axes；inline OmOwnerInputScheme帶名冊模式／鎖定clock／rules／heroContentID[129]／唯一公開生命期，不保存lease pointer。driver把combat-only queued scope泛化為owner scope，跨連線／rebase拒絕舊排隊控制。尚未更新native consumer／profile set，不能宣稱玩家WASD可用。E1032舊End修復已在abi-v1 core779成功，整輪仍執行中；header與舊layout smoke需在真退出後一併同步再驗。

接下來方向wire的階段契約：client只能提供當前own disclosed source reference與press生命期，不接受client canonical ID；server按session／view／disclosure／tick驗證後封存精確authority生命期。filtered replay使用獨立的當tick pre-dispatch control snapshot，加上post-tick名冊狀態，避免同tick Begin後Stop／點擊在Moves階段退休時，replica漏掉已發生的移動。新Reveal仍使用既有post-tick baseline staging，不能多走一步。先完成並驗證此契約，才開UE WASD入口。

方向wire待實作的具體順序：新增 typed schema／press Begin-Update-End／signed axes 與 source-only SecureReplicaTarget；authenticated edge 驗 view／disclosure／input tick，再封存精確 source life，拒絕 client 提供封存欄位。不要借敵方 target handler，也不要在重播時用目前 owner 反查來替換舊生命期。mode 與 direction 在 pre-dispatch 保持同 lane 的輸入順序，死亡／warmup／pause 分別驗，matching End 可在暫停退休。

快照網路投影只帶 tick／scheme／normalized direction／press／navigation reset，private source 只用來核對當前 roster life，不進 protobuf／JSON。outer complete owner vector、epoch／hash／bootstrap／rebase 共用既有契約；filtered consumer 按 exact tick 與唯一公開 owner body 安裝，Reveal／replacement 仍依既有 baseline staging 排除當步模擬。direction ACK 擦除封存 identity，使用 projected actor；direction 本身不在 filtered input tick 再套用一次。capability 不完整、step tick 不符或欄位非法整批拒絕，不能默認方向可用。

E1021補充：step另帶attack_reset，僅authority模式切換設定，必須連同navigation_reset；consumer調用同一interrupt_attack_for_accepted_command，方向Begin禁止取消攻擊。新direction wire要另有明確direction protocol capability，不能因為有step資料就推定支援新命令；paused／dead可沒有當步step，matching End仍依當前公開source與press生命期退休。client bridge要識別本機已接受但尚未出現在權威投影的Begin，讓同press heartbeat／End不被延遲投影誤拒絕；此本機狀態不能冒充authority applied，Stop／mode／owner／source／connection退休一律清除，不可跨生命期或恢復已退休press。

實作時進一步收斂：bridge只驗語法、明確protocol capability、模式與當前owned source reference，不拿延遲的active_press當作Update／End admission門檻。唯一authority gesture state machine決定是否退休；單一local action owner只保存實際held keys／當前source token／本機press counter，負責有界heartbeat與失焦／Stop退休。避免再複製第二份「權威press」到bridge而造成高延遲誤拒絕；local accepted仍不是authority applied。

英雄設定採版本化 profile set：全域基底、point／keyboard mode patch、heroContentID＋mode sparse patch。binding identity 為 action／slot／trigger；未覆寫的欄位永遠繼承後續全域修改，提供逐欄 reset。UI／audio 不放 hero patch。當前 authority scheme 決定 active bindings，設定中的下場希望模式不能繞過鎖定。keyboard 的 Shift 技能與舊 Shift queue／cast override 要獨立驗證，必要移動鍵與 contextual primary-button 仲裁不能靠 catalog 順序碰巧工作。所有 schema／conflict／migration 驗證成功後才 atomic publish。

官方操作基準於2026-10-12重新核對：[Keyboard (WASD) FAQ](https://support.riotgames.com/en-us/league-of-legends/gameplay/league-of-legends-keyboard-wasd-input-faq)。鍵盤模式四技能預設右鍵／Shift／E／R，召喚師Q／F，C取消瞄準，左鍵確認／攻擊，中鍵Scout Ahead；首次動態鏡頭與Scout啟用，兩種模式獨立改鍵、可按英雄保存，10分鐘鎖定，可選45度方向。

[官方 Ranked Release 設計說明](https://www.leagueoflegends.com/en-us/news/dev/dev-wasds-ranked-release/)補充：WASD開始普攻後不由方向取消；無障礙游標輸入、joystick映射、必填移動鍵、明確分開選取／互動／攻擊／確認施法且允許情境共鍵。新方向 Begin 的舊點擊 attack-cancel 呼叫需依E1016修正，保留已承諾 target至合法impact，不能靠歸一化／heartbeat測試代替戰鬥行為驗收。官方離鏡頭地形繞行也需另建基於公開地形的確定性策略，不能讀隱藏單位或由 client 送遠端 MoveTo；當前 swept slide 不冒稱已完成這一項。
