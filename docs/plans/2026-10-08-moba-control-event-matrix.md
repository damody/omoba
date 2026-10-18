# 對局操作事件驗收矩陣

最新階段（2026-10-09）：六格道具卡 `owned-item-cards-accepted-v1` 已接受當前274cases／119intents／94stages及原253／115／80前綴；ordinary no-cache build-v6、native-v4實跑10 Success／0warnings、Rust reader16＋client all-targets與另17未變Rust／Lua31，沿用條件詳見summary。當次三張display PNG逐張目視。父項8／44，renderer local admission不是authority ACK。設定physical-text正在修復實際native-v2兩項Fail（E586／E587）；11張原創道具圖目前僅已保存候選，不算HUD實作。完整驗收仍最後，quota10／10不重設。下方較早的6／44或待驗段落是歷史進度，不能取代最新summary。

此表為待執行契約，未執行的列不得視為通過。對應主計畫 `2026-10-07-moba-controls-presentation-plan.md` 與 OpenSpec 44 項父任務。快捷鍵為本專案預設，玩家已保存的改鍵優先。

| 組別 | 必驗事件 | 可觀察結果 |
|---|---|---|
| A 點地板 | 按 A、移動游標、世界左鍵、按住重複事件 | 顯示實際攻擊射程；確認只送一次 AttackMove；沒有確認不發移動 |
| A 取消 | A 後右鍵、Esc、失焦、死亡、換英雄、開設定 | 預覽與按住狀態清除；舊 release 不送原瞄準指令；Esc只取消，合法world右鍵依普通路徑最多送一次移動／指定攻擊 |
| A 介面 | A 後點技能、道具、小地圖、商店、透明 HUD 空白 | 互動元件不確認地面指令；透明世界區域可確認；小地圖獨立操作 |
| 立即攻擊移動 | Shift+A、重綁主／次鍵、額外 Ctrl／Alt、repeat | 單次立即 AttackMove；無第二次左鍵；Shift 不同時追加 queue |
| 精確移動 | MoveOnlyClick 點地、enemy、ally、HUD | 合法世界位置只送 Move；即使敵人在游標下也不送 AttackTarget |
| 精確攻擊 | AttackOnlyClick 點敵人、空地、己方、死亡／hidden／退役 entity | 僅合法公開敵人送 AttackTarget；空地不走；舊 entity generation 不重用 |
| 英雄限定 | hold／toggle、鬆開、失焦、重綁、直接攻擊、敵／友方指定單位技能 | 限制游標選取的單位為英雄，友方ward／minion亦不可選；不改AttackMove自動尋敵；自用／地點技能不憑游標捏造單位目標；陣營／形狀仍按內容規則 |
| 尋敵偏好 | 游標附近／英雄附近、同距離、多於十個鄰近單位、範圍邊界 | 兩個權威 selector 一致；固定點距離及穩定身份排序；明確指定目標不被鄰兵擠掉 |
| 自動普攻 | 開／關、idle、明確 AttackMove／AttackTarget、S、H、重生／重連 | 關閉只影響 idle 尋敵；明確攻擊仍有效；S 與 H 保持不同語意 |
| 右鍵與佇列 | 地板／敵人、Shift 追加、上限、按住、失焦／UI／死亡 | 單一路徑；有界更新頻率；原 command ID 與真實 ACK；非法目標不改權威狀態 |
| QWER | 各 slot 普通／快速／指示器、自身／地點／單位、取消與原物理鍵 release | 普通左鍵確認、快速按下、指示器原鍵鬆開；每個實際形狀各自驗證 |
| 修飾鍵 | Ctrl 升級、Alt 自用、Shift 模式覆寫、先鬆修飾鍵、後鬆原鍵 | 精確組合；合法 rank／target；原 token 一次提交；不意外觸發基礎動作 |
| D/F | 各 mode、自己／地點、未配裝、capability 缺失、冷卻、點 HUD 卡片 | 獨立兩槽；自身立即、地點按 mode；卡片開始瞄準不把 HUD 座標當世界；拒絕不分配 input ID |
| 道具／ward | 六物品改鍵、Alt 自用、地點／單位、ward4、charges、expiry | 獨立索引與冷卻；合法權威效果、視野與存活期；舊物品4設定 migration 可見 |
| 鏡頭 | wheel、Space、Y、F1–F5、方向／edge／中鍵、F9 | 實際 camera transform／projection 變動與邊界；隊友依當前合法公開資料；不移動英雄 |
| 小地圖 | 左鍵／拖曳、邊界外 release、右鍵、Shift 右鍵、鏡頭框 | 左鍵只查看；右鍵只下移動；capture 回收；框線隨實際 camera 更新 |
| 小地圖偏好 | 左／右位置、移動開關、施法開關、瞄準時點擊、ping 修飾鍵 | 保存且不默改舊設定；僅開啟偏好後將合法小地圖位置交給共用目標；不重複camera／cast／ping |
| 隊友頭像 | 一般focus、友方指定單位技能瞄準、死亡、換owner／entity generation | 一般點擊不送技能；合法瞄準提供公開友方目標；死亡／退役／hidden 不重用舊 target |
| UI 遮擋 | HUD、透明空白、shop/settings scroll 頂底、editable、results | 實際 painted geometry；panel wheel 不洩漏至 zoom；文字／modal 吞 gameplay；自用快捷鍵不因一般 hover 失效 |
| 商店 | P／Esc、搜尋、分類、配方、購買、出售、undo | 距離／金幣／欄位由權威決定；receipt 對應原 ID；拒絕有具體可見原因 |
| 資訊 | Tab hold／toggle、C、選取目標、死亡、Ctrl+F、F12 | 合法公開資料；真實死亡／屬性／統計；截圖包含目前 viewport／panel 並持久保存 |
| 溝通 | G／V、Ctrl/Alt 拖曳輪盤、右鍵取消、minimap、HUD Alt-click | 八種語意；一次合法 team 傳送；rate limit／mute；objective 投票；無 hidden 資料洩漏 |
| 聊天／表情 | Enter、Shift+Enter、Z、T、Ctrl+1–5、輸入焦點／斷線 | team/all 明確；歷史／mute；原創動畫與回饋；文字編輯不發 gameplay |
| 設定 | 主／次鍵、mouse1–5、衝突、restore、寫入失敗、舊 profile、每英雄覆寫 | 原子保存／回退；標籤即時更新；不默改既有鍵；無功能的 action 不假裝可用 |
| 替代控制 | WASD／點擊、45°、動態鏡頭／Scout Ahead、英雄 profile、重連 | 同權威命令；依 match rules 限制切換；不與 QWER／camera 鍵互相誤觸 |
| 無障礙與拆分輸入 | 可改鍵游標四向／速度、搖桿映射WASD、選取／互動／普攻／確認施法分開綁定、必要鍵未綁 | 經本地同一輸入仲裁；游標不產生隱藏目標；失焦歸零；必要鍵有可見錯誤並原子回退；允許共用鍵的動作依上下文只執行合法一次 |

每列均需覆蓋 native 與既有 Blueprint controller。最終矩陣包含 1080p／1440p、100%／150% DPI、視窗／全螢幕／PIE、失焦／死亡／重生／斷線／重連／PIE restart。響應式 HUD 的 400×300 等 fixture 僅驗小視窗與元件，不替代這份完整矩陣。

最新已執行證據：普通NoEngineChanges `controls-precise-owner-build-v3`通過，`controls-precise-owner-native-v2` exact53 Success、0 failure／warning／notRun，八張production／responsive PNG已核對；E464–E466右鍵observer、真UI事件與click-only回歸修正已實際通過。`combat-native-consumers-validation-v5`共63測試與core all-targets通過，共用策略已接實際hero command／attack消費端，包含cursor AttackMove與idle獨立性。偏好wire與持久化仍未接；disconnected owned fixtures不替代合法connected Move／AttackTarget、已armed瞄準取消或真權威ACK。父項仍6/44，正在補實際disclosed faction／attackability路徑；真權威對局既有10/10不重設，缺少實際ACK或跨Engine frame證據的列保持待驗，不用本地fixture假造結果。

精確血量增量：precise-health-core-v2 6項及precise-health-rust-abi-v2 bridge92／input17／presentation28＋3checks通過；native／filtered實際資料路徑一致保存正數小數HP。其餘E470 attackability／source retirement及UE ABI19 stage仍pending，父項6/44不變，完整驗收未提前執行。

Current-match shared attackability independently accepted in Rust scope: current-match-native-v4 67 tests and core all-targets PASS; current-match-health-regression-v1 3+2+1 PASS. E475/E476/E477 failed evidence retained. ABI19 Unreal formal stage and source retirement part3 pending; no authority games, positive network ACK or full controls acceptance claimed.

E470三項修復完成目前範圍驗證：current-match-native-v4 67項＋core check；relation-source-core-v1 7項；relation-source-rust-abi-v1 bridge93／input17／presentation28＋3checks；普通nocache controls-target-metadata-build-v2與ABI19 C++ header smoke通過；controls-target-metadata-native-v1 exact54 Success／0warning，八張新PNG已逐張重新目視，adaptive尺寸差異已記錄。正式stage artifact-copy consistency已驗證。仍不代表positive owned操作／authority ACK／全計畫完成，6/44與10/10不變。


Renderer publisher foundation scoped acceptance: renderer-fixture-rust-v2 98 library tests＋thin bin compile＋compiled-content-only all-targets/build PASS; native host18＋real captured-start/alive/readiness/stop＋owned wait PASS; missing opt-in invocation failure retained and exact live identity rerun PASS. renderer-fixture-cli-v2 actual CLI gates, wrong-run/sequence/stale rejection,12 stage controls including source death, graceful stop,1s natural lifetime＋bounded cleanup grace, and deliberate cleanup-write failure all matched expected results; original processes retired. Current compiled content c41fcc1e80e95ad7, map catalog b7a9dda88f881c49. Primary Cargo default-run preserved/metadata verified. E483–E486 failed evidence retained. UE owned viewport events and authority ACK remain pending;6/44／10/10 unchanged; zero authority games.


Owned viewport IPC v1：Grok4.7 NON FAST run-muzakccd-1sda42 真正終止57m19s，208calls，cost5.66027716USD（input973033／cached25007488／full25980521／output229998／reason167390／total26210519）。Primary 正式 ordinary nocache build v1通過19actions／UBT452.27s，兩組Lua契約8／44通過；fresh real viewport v1為0success／1fail／0notRun，已送出第一筆 genuine ground RMB MoveTo，但讀intents併發IO失敗，不能接受整批。兩個原始process token與listener清理通過。E487來源接線與E488實跑失敗已記MD；同thread nonfast repair run-muzd0ol6-hjiie3執行中。generated content d5553bbb31c459a4與catalog_data c41fcc1e80e95ad7是不同欄位，非stale證據。完整驗收留到最後，6/44與10/10不變。


Owned viewport IPC v2：nonfast repair run-muzd0ol6-hjiie3 真正終止21m48s／99calls／cost2.82134176USD（input234812／cached13281024／full13515836／output92012／reason58167／total13607848）。Primary Lua契約21／44＋bridge93／all-targets通過；ordinary nocache build10actions／29.49秒通過，stage bridge d87cc097052e10ad30649d15b33805a1ebbffe28b552ab0a11fd9589ae1f8f99／base1e7fd7bf4494db93c9277e9227fc0fbe49ae4f3ab53f363b727f12d18d2a663c。scoped D3D12 native55通過，8PNG比較完成、7changed重新目視、1identical；adaptive尺寸差異非OS DPI量測。fresh real viewport v2完成前11case、停在不可聚焦SBorder的second-window fixture，E489已記MD，所有owned lifetimes/listener清理通過。後段source death/replacement/epoch尚未驗收；同thread nonfast tiny repair run-muzdzwge-rpzxy6執行中。6/44與10/10不變，未啟動full acceptance。


Owned viewport IPC v3：formal build passed；真實前19 case通過，epoch-attack Q preview失敗（E490），owned lifetimes/profile均清理成功。完整專項尚未接受，後續cadence尚未啟動。


Owned viewport IPC scoped acceptance v4：Grok4.7 nonfast retirement修復 run-muzeaj9g-dtcdrq 真正終止4m40s／18calls／cost0.73559476USD（input48772／cached3230208／full3278980／output19982／reason14723／total3298962）。Primary ordinary nocache build-v4通過；D3D12 retirement-native-v2 exact56 Success／0warning／0notRun；八張PNG比較，五張changed重新目視、三張adaptive byte-identical。fresh實際viewport v4 exact22case／15真實intent／10controlstage全部通過，含來源退役後Q恢復、stale release不送cast、epoch與target replacement。原始editor/publisher retired、listener released、profile清理成功；content/template/classsurface身份核對通過。accepted摘要 .ai-collab/owned-viewport-ipc-accepted-v1/summary.json。範圍是production renderer IPC與local admission，不是authority ACK；完整操作／畫面計畫仍未完成，父項6/44與quota10/10不變，零額外authority games，完整驗收尚未執行。後續開始cadence與A地板確認。


Cadence current scoped progress：Grok nonfast base run-muzelzhm-uxxpql終止29m39s／121calls／cost3.07442756USD；test repair run-muzfw2ih-hphbg7終止3m57s／26calls／cost0.79075908USD。Lua21/44通過，ordinary build-v2通過、D3D12 native57通過及真spin事件驗保存；fresh viewport v1在第23 A LMB座標比較失敗（E496），前22舊case通過、清理成功。31case/21intent整批未接受，repeat後段未執行；framecap恢復20000（E493），未提前full acceptance。


Cadence scoped acceptance：ordinary nocache build-v4通過；D3D12 native-v2 exact57通過，E497只改專屬viewport test，未重跑未變native suite。fresh held-world-cadence-viewport-v3 exact31cases／21intents／10stages全部通過，含A地板嚴格座標、UI不確認、Esc跨真tick取消、相同點去重、改變目標、large-delta只一次、release／UI focus／profile中止。原程序退役、listener釋放、profile還原通過。E496與E497修復驗證完成；scope為renderer IPC local admission，不是authority ACK。八張widget PNG中五responsive重新目視、三identical沿用，repeat設定區仍未截到。accepted .ai-collab/held-world-cadence-accepted-v1/summary.json；完整驗收尚未跑，quota10/10不變，開始雙快捷鍵批次。

雙快捷鍵目前仍未接受：ordinary dual-bindings-build-v2 通過，Lua contracts 21／44通過；D3D12 dual-bindings-native-v1 實際59項，56Success／3Fail／13errors／0warnings／0notRun。E501修正來源已終止並審查，尚未重建；viewport inventory來源增加至47cases／30intents／10stages但未實跑。四張設定PNG已目視，含repeat／typed search empty／副鍵欄位，整套測試失敗不能宣稱通過。另審查發現profile install清掉component physical latch卻保留model await-release，正由Grok4.7非fast run-muzkpm04-62h4to修正；同時補真正副鍵UI commit／unbind與新施法／舊release交錯證據。不得以這段來源進度勾選父項或稱完整驗收成功。


雙快捷鍵scoped acceptance：ordinary dual-bindings-build-v5 nocache通過；D3D12 dual-bindings-native-v3 exact59 Success／0errors／0warnings／0notRun。fresh dual-bindings-viewport-v2 exact47cases／30intents／10stages全部通過，含主副鍵獨立release、單owner連發時鐘、profile施法退休／新啟動／stale release、設定介面真實副鍵commit／unbind。11張widget PNG中9張重新目視、2張baseline byte-identical；400x300可讀性仍待偏好批次。原程序退休、listener釋放、profile還原皆通過。accepted .ai-collab/dual-bindings-accepted-v1/summary.json；scope是renderer IPC local admission，不是authority ACK。完整驗收尚未跑，quota10/10與父項完成數不變。


Owned ability scoped acceptance：ordinary owned-ability-build-v13 nocache PASS；D3D12 owned-ability-native-v4 exact60 Success／0errors／0notRun；含warning的測試數1，完整warning entries留在accepted summary，不隱藏或改寫原報告。fresh owned-ability-viewport-v10 exact112cases／54intents／35stages全部通過，包含QWER normal／quick／indicator、原鍵release、取消／focus／modal／UI不確認、modifier-first release、升級／self／allegiance／隱藏／死亡／中立拒絕。新增body radius regression驗證實際frame publisher，避免nameplate／裝飾／插值位移放大選取。11張widget PNG的實際目視或byte-identical沿用詳見visual-review.json。原程序退休、listener釋放、profile還原皆過。accepted .ai-collab/owned-ability-accepted-v1/summary.json；renderer IPC local admission，不是authority ACK。完整驗收尚未跑，quota10/10不變，HUD可讀性與其餘計畫仍待完成。


Minimap preferences scoped acceptance：ordinary minimap-preferences-build-v6 nocache PASS；D3D12 minimap-preferences-native-v3 exact63 Success／0errors／0notRun，warning entries完整保留於accepted summary。fresh minimap-preferences-viewport-v1 exact121cases／58intents／36stages通過，包含原112案例與新增小地圖移動／地點施法偏好。左右位置、實際hit-test／drag／有效camera、modal／focus／profile與資料失效／玩家切換capture退休及其他user／pointer隔離由native證據支持。11張PNG目視或byte-identical沿用見visual-review.json。原程序退休、listener釋放、profile還原皆過。accepted .ai-collab/minimap-preferences-accepted-v1/summary.json；僅renderer IPC local admission，不是authority ACK。戰術ping仍未開放，跨OS DPI／PIE完整矩陣待最後驗收，task2.7及完整計畫不因此勾完。quota10/10不變。


D/F scoped acceptance: owned-summoner-build-v3 ordinary nocache PASS; owned-summoner-native-v2 exact63 Success/0errors/0notRun, actual warning entries preserved. Fresh owned-summoner-viewport-v2 exact158cases/69intents/51stages with cleanup/listener/profile retirement PASS and actual minimap121-prefix comparison. Real D/F normal/quick/indicator, self/point, cancel/focus/modal/profile, cooldown/unequipped/malformed/unsupported/dead/replaced/removed owner states checked. Catalog rows are disclosed fixture state and admission is not authority ACK/server outcome. Evidence .ai-collab/owned-summoner-accepted-v1/summary.json. Full acceptance remains last; quota10/10 unchanged.
