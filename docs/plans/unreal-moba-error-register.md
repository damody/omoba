# Unreal MOBA 防錯紀錄

## E186：實際原型接線要看生成 handler 與 full data hash；raw 期望型別別混用（2026-10-05）

- 決定：先鋒整備＋temporary capacity、遊俠包紮＋自然regen、共享術士回春＋立即Mana，全部Lua effects／extras，不改技能ID／原cost／HP或角色程式。三原型正式PlayerInput→正常生成manifest→outcome／finish確認，不手換handler。
- 編譯錯誤：成本通知Vec<i64>與期望Vec<i32>比較E0277；明確i64::from(cost)*1024後新正式case成功。世界單位與Q10 raw需要同時核對scale與寬度。
- 規則影響：遊俠rank1回魔現在5＋2=7；既有Bot預算期望改讀UnitStats並assert7，不讓stale固定5被誤認runtime錯誤。仍只走正式input。
- 工具問題：rg單行launch-plan輸出截斷，改唯讀JSON欄位；猜tick_hz缺值不當成功證據，60Hz以prepare真實回報。content_hash不變而catalog_data_hash更新14ef1dea84790afb，不能混舊DLL冒充stage。
- 確認：新三原型case1／Bot相關1、codegen生成＋check、固定Lua單人九Bot60Hz prepare成功；沒有100場／DLL載入／UE驗收，20/30與E177保持。詳見mana-archetype-content進度。

## E185：Lua Buff 必須型別化與固定刷新身分；測試別猜 CD 欄位（2026-10-05）

- 決定：mana_buff_self用shared七stat白名單、signed value／正duration envelope、每技能unique stat；固定Lua生成typed EffectOp，generic preflight後deferred add_stat_buff，ROk才提交。Buff ID含ability／stat／entity／generation，同來源max duration刷新不增層，不同技能疊加。
- 編譯錯誤：fixture猜hero.cooldowns造成E0609；查實際ability_cooldowns後改公開start_cooldown目標技能ZERO。最終新model1／base2成功，不為測試擴大API。
- 證據界線：原fixture mana_enabled=false只能證明payload，改正式初始化明確true與隔離回魔後確認90－45=45、容量280→335不補滿、重施不再加55、失敗留舊Buff正常倒數。不用後置切flag冒充正常啟用。
- 確認：新Rust3／Lua34（其中新21）與codegen --check通過；既有Mana2相關案例曾成功。沒有完整新英雄DLL／UE驗收，20/30與E177維持。詳見lua-mana-buff-declarations進度。

## E184：容量 Buff 必須在施法前同步；只看結尾不越界不夠（2026-10-05）

- 決定：UnitStats原始Lua capacity＋checked ManaBonus／ExtraManaBonus，合法負值截0，非法／overflow／超百萬退base；ManaPool增容量不補滿、縮小截限並清滿池餘數。
- 邊界風險：只在finish同步會讓已到期Buff的超額魔力被當輪cast消費。共用dispatch建立cache前同步，效果outcome後finish再同步；同hook新Buff維持deferred可見性。只處理active mana match living slots，不創建None池／TD不變。
- 操作／編輯問題：猜native/mod.rs搜尋失敗，rg找到native.rs re-export；helper插入暫移接了finish的commit comment，diff檢查後修回。
- 確認：新core1／正式60Hz base1與既有lifecycle3共5 passed，沒有compile／test failure；Lua持續Buff與完整框架仍未完成。詳見mana-capacity-buffs進度，20/30及E177狀態不變。

## E183：有回魔 helper 不等於已接線；負倍率與固定數值溢位（2026-10-05）

- 發現：UnitStats.mana_regen已存在，但MOBA finish仍用固定Lua rate。已接自然Buff與独立基地加成，合併一次Q10 remainder，保留有效時間／死亡／暫停等既有gate。
- 數值風險：原式只截最後结果，兩個負倍率可變成正回魔；fixed加法／abs／乘法可溢位。新增checked_sum_add以i128與unsigned_abs保留family聚合，checked_mana_regen逐factor截0與checked產品；非法自然rate fail closed但不禁用合法基地恢復，沒有全面改其他stat。
- 操作／編輯問題：buff_tick輸出被小cap截斷，改以正式到期與pool確認而不猜；新增測試把舊移速comment暫留到Mana測試前，diff檢查後修正。
- 確認：新core2／正式60Hz base1與既有sustain2共5 passed，沒有編譯或測試失敗；非Lua持續Buff作者流程／上限Buff／完整UE驗收。詳見mana-buff-recovery進度，20/30維持，E177未解除。

## E182：宣告式資源效果依賴 host 交易；外部 build-dependency 不要混選測試（2026-10-05）

- 決定：共享schema／固定Lua生成器／generic handler支援restore_mana_self與spend_mana_self。全plan preflight後按實際池與作者順序准入資源，再emit其他效果；額外成本不足由host回滾，不能用猜容量或把helper當成自帶rollback。魔力不是HP，不偽造Heal preview。
- Cargo錯誤：scripts workspace同選外部僅build-dependency的omoba-content-model，Rust1.95 Cargo resolver在NormalOrDev features panic；拆成各自manifest後新model1與base2成功，不動工具鏈／lockfile。
- 編譯錯誤：漏補既有ResolvedEffect fixture exhaustive match造成E0004；補兩個明確variant分支後成功。新增enum必須檢查production與test matches，不用wildcard掩蓋。
- 操作錯誤：一次context-only patch沒有改動；改具名test插入並檢查實際執行。literal buff*／猜unit_stats路徑搜尋失敗，改rg --files定位，不改未讀懂的Buff架構。
- 確認：9個不同Rust tests與Lua13案例passed、UE codegen --check沒有diff；不是新Lua英雄端到端或完整UE驗收。20/30保持，詳見declarative-mana-effects進度，E177未解除。

## E181：腳本魔力要共用帳本；tick fixture 必須實際註冊（2026-10-05）

- 決定：host metadata 扣一次，腳本讀 post-cost 餘額並可額外 spend／restore；serial event＋managed tick 共用 ledger、成功才提交狀態與通知，非 managed TD 保留平行。移除舊 cast view；ABI 只補語義文件，不增加方法。
- 實際失敗：兩項正式60Hz測試得到55／20而非54／19，因 fixture 嘗試替換 manifest 不存在的英雄 unit；只補 tag 仍不執行。檢查 units() 後明確 append UnitDef＋tag，最後3/3通過。不修改預期或正式英雄生命週期迎合測試。
- 邊界：legacy caster 也可能修改 managed recipient，不能只用施法者 reservation 決定 rollback；失敗 cast 有 dirty pool 也丟棄交易，新跨角色案例證明 recipient／通知／CD皆不變。
- 編輯／操作錯誤：過寬 patch 將 mana view 暫插到 buff remaining，編譯前移正；再次猜檔名與 literal glob 路徑失敗、多檔輸出截斷。使用 rg 定位、獨特上下文與逐段讀取，shell最後exit0不代表前面的搜尋成功。
- 確認：core mana篩選19／base新交易3／既有cast2通過；沒有全套驗收、UE或100場，20/30維持，E177未解除。詳見script-mana-transactions進度檔。

## E180：補魔策略必須有正式恢復與明確規則；feature 測試零項不是成功（2026-10-05）

- 決策：sustain 可選mana門檻＋配方明確mana_enabled，server旗標必須一致；None／零容量不等候。基地速率來自Lua，權威有效時間、活本人基地、存活／非lethal才加成；合併自然與基地速率後一次Q10餘數結算，AI只正式Recall／MoveTo。
- 設定風險：若只補AI門檻而没有基地資源恢復，會長時間等待自然回魔；若server覆寫配方flag而不驗一致性，會產生已宣告策略卻沒有法力池。兩者已用共用正式恢復與啟用衝突拒絕修正，未修改一般啟動預設。
- 編輯錯誤：插入server測試時提交空update hunk，apply_patch整份拒絕；改用實際完整函式上下文插入，沒有部分落地。
- 操作錯誤：多來源／OpenSpec輸出再次超過外層輸出上限，改分段補讀。猜 moba_role_config／moba_role_match_config／filtered_match_tests／plan.json 路徑失敗；以rg --files與現有launcher定位，實際產物為launch-plan.json。不得因錯誤在非最後一個shell指令而exit0就忽略。
- 測試錯誤：template預設未啟用runtime-lua-content，初次指定測試回報0；直接給workspace外package features又被Cargo拒絕。最後透過workspace內omobab/runtime-lua-content啟用依賴，再確認template新測試實際1/1，不拿零項或server成功代替。
- 確認：本功能新core2／正式60Hz base2／template1／server1與舊HP功能1共7個不同測試通過；固定Lua新一真人九Bot60Hz prepare與codegen生成／check通過。沒有全套驗收、stage或Unreal執行，詳見bot-mana-sustain進度。

## E179：Bot 預算必須讀正確 metadata 資源；fixture 不得猜成本（2026-10-05）

- 實作決定：共用 checked_mana_cost 給 Bot／authority，原始 f32→Q10與倍率進位完全相同；AI只有本人讀取權，不預扣，跳過不夠／缺資料技能並保留後續候選。
- 實際接線錯誤：首次 Bot 讀 World 的 ScriptRegistry，純 planner 通過但正式60Hz沒有選到施法；SimulationDriver::from_world 已 remove 並持有 registry，診斷直接 fetch 證明不存在。改讀初始化直接 clone 同一 AbilityDef 的 AbilityRegistry，不複製執行器或放寬缺資料的 fail-closed。
- Fixture 錯誤：property(20,0) 同時設 hp／mhp=20，不符合自療門檻；改 mhp100／hp20。另誤認現有箭雨一級70，實際 Lua builder 全部一級45；正式案例使用合法 rank4成本60對比rank1成本45，不改遊戲資料迎合測試。
- 操作錯誤：多來源輸出超過 max_output_tokens 再次截斷；後續改具體片段。猜 parallel_adapter／state_initializer 檔名搜尋失敗，改 rg 目錄與 rg --files 定位 parallel_world_adapter／initialization。一次診斷 patch 命中相同舊assert，已立即還原；之後使用獨特 expected-pool 上下文。
- 防再犯：先追查資源生命週期與來源建立點，再選 ECS 查詢；讀實際 Lua rank 成本和 fixture helper 實作。診斷不能變成 production fallback，不以純函式通過代表正式接線成功。
- 確認：core2／正式60Hz base1／既有 managed施法base2 passed；完整框架、補魔策略、UE與100場未驗收，見 bot-mana-budget 進度。

## E178：Mana 協商必須先於註冊；停用對局不能洩漏新事件版本（2026-10-05）

- 發現：既有 finish hook 無論啟用與否都發 CommittedMana26／None，可能讓未宣告新能力的舊端收到未知事件。改為明確啟用或已存在 managed pool 才發；短60Hz兩種配置確認停用0／啟用2 pools每tick。
- 決定：獨立 version1＋full data hash；server opt-in 預設false、secure MOBA限定，註冊前驗證加入者支援與 selective-player條件，再於 initial／rejoin bootstrap回覆。新client宣告能力不代表server開啟；無共享引擎修改或直接啟用launcher。
- 操作錯誤：首次新增fact條件的多檔patch附带猜測不存在的測試函式，整份驗證拒絕；移除無關hunk後依實際來源重套。再次猜simulation_driver／observable／match_plan路徑；以rg --files定位native/simulation_driver與bots/plan，確認 result.facts後才寫回歸。
- 編譯界線：server仍使用checked-in proto fallback（protoc warning），本輪以core vendored protoc正式同步生成，server編譯成功。將來新增proto需先同步fallback，避免多workspace建置讀取舊fallback；不以zero tests或單獨core成功代表server成功。
- 確認：core1／server2／base1（正式60Hz）成功；root／omb／omfx whitespace通過，無測試或編譯失敗。沒有真實KCP、renderer重連、UE／LAN或全套驗收；E177引擎阻礙仍待。

## E177：Mana HUD 版本界線與共享引擎建置阻礙（2026-10-05）

- 實作：本人 disclosed Mana→HUD schema2→受驗 IPC→C ABI12→共用 UE payload／MP文字與法力條；legacy schema1 只允許未支援且數值零，None 不等於零池。新 Rust 測試2＋bridge3 passed（1capture ignored）。UHT 已成功，但 native C++ 未成功，不能冒充 UI 驗收。
- 編輯錯誤：多檔 patch 留空 update hunk，整份拒絕；移除空 hunk 後重套，檢查新測試確實存在。第一次runtime測試在新增test前執行，8tests只是原回歸；之後新Mana2tests單獨確認。
- 操作錯誤：再次猜 `build_ue.lua`／smoke／TargetDescriptor 路徑並傳 literal restart glob給rg；改 `rg --files` 查實際 `build_ue_moba.lua`／`ue_native_visual_smoke.lua`／Configuration/Descriptors。restart executable 相對路徑配錯 cwd、誤用不存在的 OmGame.uproject；實際 project 是 `omfue/om.uproject`，後續採絕對路徑。
- 程序錯誤：在 stop session 尚未完成時提前重跑build，被Editor gate再次拒絕；必須 await stop完成，再建置。MCP dirty content／maps皆空，但既有restart超時自動force terminate71024，回報force_terminated=true；不得宣稱graceful。未刪檔／修改未儲存資產。
- 真正阻礙：UBT `FailedDueToEngineChange`，引擎現有三個 Skeletal*來源修改造成Engine unity cache／link產物待重建。保留 `-NoEngineChanges`、不還原或建置使用者的共享引擎修改。bridge12已stage，舊native ABI11必須fail closed；完整native驗證需可用引擎基線。
- 防再犯：辨識 project scope與engine scope；不得藉舊DLL、移除安全旗標、只看UHT或禁用版本檢查，把功能宣稱成功。長UBT輸出要改讀具體log摘要，不能依截斷末端推論原因。
- 收尾：Editor91188重新啟動，HTTP MCP30000 ready；root／omfue whitespace檢查通過。不宣稱new native module／NativeManaHud／PIE成功，完整引擎基線問題仍待。

## E176：Mana 生命週期不能使用舊智力公式；測試不可跨私有邊界（2026-10-05）

- 決定：正式 opt-in 由 Lua base_mana／growth 取得上限，i128 與開局所有等級檢查；新生命滿池、升級不補滿、有效時間 post-combat 再生。共用 Lua rate 進 full data hash／compiled agreement／no hot reload，既有模式預設停用。
- 實際編輯問題：第一次多檔 patch 假設 Lua 欄位獨立一行，與實際 compact table 不符，整份 patch 驗證失敗；確認無部分落地後依實際行重套。編輯時另移除誤留的 duplicate doc comment 與多餘 dead branch，未帶入測試版本。
- 實際編譯問題：fixture 把 `Finished` 當 unit variant（缺 winner／tick），並直接讀 private `MobaHeroSlot.hero`。改正 struct variant，使用公開新生命身分／等級／滿池結果與 pause 不重生斷言，不為測試公開內部快照。
- 實際工具問題：對 workspace 外 template-id 直接 `--features runtime-lua-content` 被 Cargo 拒絕。依既有模式同選 base_content／template-id，使用 `--features base_content/runtime-lua-content`；不能把 package filter 執行零 tests 當成功。另誤讀不存在的根 Cargo.toml；先定位真實 workspace，不猜根 manifest。
- 輸出問題：多檔讀取再次超過 cap；需要精確內容時分段限行，不依截斷部分推論。
- 確認：base3／core1／template2 通過（含欄位型別與省略預設），雙隊短60Hz完整hash零repair；UE生成與--check、來源diff whitespace檢查通過。最後template篩選的伴隨base0 tests只算編譯；沒有一般網路啟用、buff再生、script restore、HUD或完整Unreal驗收。

## E175：Mana相同仍可能缺owner輸入；再生餘數不能漏投影（2026-10-05）

- 決策：append CommittedMana26／binary v1 disabled2或enabled20bytes；current／maximum／Q10再生餘數同傳，checked decode先驗再寫，沿既有Hero baseline與visibility gate，不傳hidden來源或canonical身份payload。零Mana不等於None。
- 實際失敗：新短60Hz filtered fixture team1 tick3 hash不符；加入逐component診斷再次重現，Mana三欄完全一致，只有owner ranger_patch cooldown {} 對權威20480。漏pending_accepted_inputs使owner未重演施法；project_tick本身不替SimulationDriver建立server accepted projection。
- 修正與防再犯：fixture沿正式 CanonicalAcceptedInput／normal PlayerInput接線，原ID、team1-only及其他隊零accepted數量明確断言，兩隊每tick完整hash通過。不得覆寫owner cooldown、只比Mana小雜湊或加入ComponentRepair來讓fixture通過。
- 操作問題：多檔合併輸出再度截斷；精確小段補讀producer、decoder及既有正常accepted-input fixture，不把截斷當全讀或猜hash原因。
- 確認：core4／base1（12ticks雙隊24steps）通過、無repair，保留mismatch診斷；不等於網路／UE或整場驗收。正式協商／規則啟用／再生／生命週期／完整script API與HUD仍待，不開一般對局Mana。

## E174：法力與冷卻都不能讀同 tick 舊 cache；0 tests 不是驗證（2026-10-05）

- 決定：managed SkillCast 以 serial queue 私有 ledger 預約候選 Hero、當級合法成本與既有倍率，成功才提交 Mana outcome與CD，失败丟棄該 invocation outcomes／overlay／visual。同tick不同招餘額、同招CD都讀更新後帳本，不用平行鎖搶先順序。既有未啟用模式保持相容。
- 防再犯：execute成功不代表unit pre-hook沒有panic；已捕捉的hook panic另記flag，managed cast不得提交。只保证延後adapter效果，不宣稱ABI abort／crash／外部I/O可回滾。host扣metadata一次，enabled explicit script spend明確拒絕，不沿旧stub虛報或雙扣；restore等仍待ordered接線。
- 操作誤判風險：初次 cargo test filter mana_cast 還沒有測試，只得到 running 0 tests；僅可記編譯成功。加入實際測試後base2與adapter1均執行通過，最後核心修改後base2再通過。
- 輸出問題：合併多檔限行仍超過內層token cap截斷；後續精確讀需要的段落，不把截斷當完整檢查。本輪沒有編譯或測試失敗，不虛構修復案例。
- 邊界：正常網路對局尚未啟用新池；Mana committed投影／hash／規則版本、再生／生命週期／HUD與完整script API仍待完成，不能因fixture60Hz成功開一般扣費或勾完整5.5。

## E173：最大法力不能冒充餘額；延後提交不可直接各自扣費（2026-10-05）

- 發現：adapter current_mana 回最大值、spend_mana 永遠成功、restore_mana 空實作；不宣稱新 pool 已讓既有腳本 API 生效。共用 event cache 與延後 outcomes 使同 tick 舊餘額有超支風險，平行 Mutex 的競速順序也不等於 deterministic gameplay。
- 決策：先實作具私有狀態／checked serde 的 Q10 ManaPool 與 Hero optional 保存欄位；成本不足／负值原子拒絕、恢復與容量邊界、i128 再生餘數。正常對局不先開扣費，下一階段必須完成 ordered commit／失敗不扣／雙扣排除與安全投影。
- 防再犯：不能逐英雄特殊扣費、只 authority 改卻漏 replica hash 或以 input ACK 當扣費成功；GameMode::Moba 也不能代表新規則 opt-in，舊 Story 同樣使用它。初始化必須冪等，不藉 repeated spawn/setup call 補滿既有池。
- 操作問題：長 OpenSpec 合併輸出仍截斷（含外層 tool aggregate token cap）；改為單次單檔限行補讀，不能以 nested command token cap 足夠就認定模型已收到全檔。
- 確認：核心 5/5、Hero 2/2 通過，沒有編譯或測試失敗；既有 td_rounds warnings 不相關。未做正常施法／60Hz對局／網路／Unreal驗收，完整Mana未完成。

## E172：減速必須走正式統計；測試須使用編譯數值與導航實際路點（2026-10-05）

- 決定：slow_enemy 共用 model／Lua FFI／Rust effects；全序列 preflight 後用既有 BuffStore 整數 Q10 move_speed_bonus、獨立來源 key 與 generic_ability_slow strongest-family，不使用 legacy float slow_factor，也不寫英雄特殊 UE 程式。家族只覆蓋新通用減速，不宣稱所有 legacy 慢速已統一。
- 實際測試錯誤：40% 編譯為 Q10 raw410，fixture 卻假定 floor409，導致速度預期不符。改讀正式 compiled rank extra，保留正式 round-nearest 行為，不為測試改 gameplay。
- 實際導航預期錯誤：先用單軸 speed×dt，再用最終目的地 normalized vector，都與實際位移不符；正式導航使用局部 MoveTarget，位移帶 y 分量。改讀同 tick 實際 waypoint、用同 Fixed64 normalized 計算精確向量；不加寬容差或跳過導航。
- 生成器檢視：以 extras key 作 Lua map 會在 reduction_key==duration_key 時覆蓋其中一個上界。改 ordered list 驗兩次，新增同 key 1.01 必須拒絕的回歸。
- 操作錯誤：猜不存在 native/ability_runtime.rs 與 docs/plans/implementation-errors.md；用 rg --files 確認實際模組與本紀錄檔。合併長輸出再度截斷，後續改限行小段；不把截斷輸出稱完整檢查。
- 確認：model1／base2（正式60Hz移動、strongest、TTL）／十二招1／Lua14 通過，UE正式生成及--check成功。未編譯／stage最新OmGame，未做網路／UE／100場驗收，完整5.5不勾選。

## E171：即時位移不可只寫終點；新增effect要補所有enum比對（2026-10-05）

- 發現：既有fire_dash直接set_pos到要求位置；本批不把該特殊handler當通用安全模板，也不改其既有語意。新增dash_to_point使用既有權威swept-terrain計算，要求整段可通且精確終點，生成／runtime皆拒絕混合效果；不宣稱沿路傷害、動態碰撞或持續衝刺。
- 實際編譯失敗E0004：新增EffectOp／ResolvedEffect::DashToPoint後，舊四個Luminary技能測試的兩個match未涵蓋新variant。補明確非該fixture效果的分支後base編譯與新功能測試成功；不得以萬用分支改掉正式runtime語意。
- 操作錯誤：先猜heroes/date/fire_dash.rs不存在，rg --files確認實際B02_date_masamune/No2_fire_dash.rs後才讀取。幾次合併source/status token cap截斷；正式修改依精確小段補讀，不把截斷稱完整檢查。先尋路徑、再讀已存在檔案。
- 決策與防再犯：advance_with_collision只計算位置、set_pos才提交；point超距／原地／受阻不部分移動或啟CD。Bot只看disclosed living target、實際rank range與明確策略，用正常CastAbility；舊vanguard_resolve的ID保留但target／語意變更必須重建與hash gate，不沿用舊SelfHeal策略。
- 確認：model1／base2含正式60Hz薄牆與Bot位移／core1／Lua7、新三原型十二招1通過；正式生成與--check、新一真人九Bot60Hz配方預檢成功。原dead-code warnings保留；最新OmGame／DLL stage、filtered／UE全套未驗收。

## E170：有限非負不等於可執行數值；Lua double 與 Rust f32 需同值驗證（2026-10-05）

- 問題：rank資料原只驗數量／解鎖門檻，負cooldown或非有限成本可進入生成；效果量缺上界，小於Q10的正值可能變零。共同model與Lua FFI補零或有限[1/1024,1000000]；range／radius仍維持原較小限制，非法內容拒絕而非clamp。
- 決定：這是內容安全範圍，不是完整Mana功能、平衡門檻或全部組合運算安全證明。特殊Rust handler也驗共用rank欄位，無關extras保留；deleted tombstone可不具執行資料。
- 生成器差異：直接比較Lua double會讓1000000.01／10000.0001等邊界只在FFI拒絕，Rust f32卻合法。Lua標準pack／unpack先转相同f32，並加兩邊邊界案例；不藉此修改正式Lua數值或hash。
- 操作錯誤：又猜不存在 native/component/hero.rs，實際是 native/comp/hero.rs；已用rg --files確認後讀取。合併長文件輸出也截斷，已用明確行數補讀OpenSpec design與tasks；後續先找實際路徑、控制單次輸出，不把截斷當完整閱讀。
- 確認：新model2、Lua numeric34、原include5、正式24項FFI IDs與base cast_preflight2（含60Hz）成功；UE生成--check內容hash不變。既有td_rounds dead-code warnings未改，不冒充Mana／盟友／網路／完整Unreal驗收。

## E169：Bot 篩選距離不能取代權威執行器；單體與範圍距離不可混用（2026-10-05）

- 發現：Generic Damage preflight 原只驗 alive／enemy，沒有依目前 rank 的 range 檢查 caster／victim 位置；HealSelf 原接受任意 Target，且純執行器沒有統一 caster 存活 gate。不能因正常 Bot 會篩選距離而宣稱非法玩家施法也被攔住。
- 決定：shared model＋固定 Lua 生成器要求有目標效果的合法可量化 per-rank range。單體 resolved Damage 帶 Some(range)，area 展開帶 None；共用全序列 preflight 先驗有效／正 HP caster，再驗單體兩端位置／精確距離，SelfHeal 強制 None。全部驗完才輸出效果，失敗沿原 dispatch 不啟 CD，不使用前端或 Bot 修補。
- 重要邊界：AoE 限制中心而非每個 victim；更新正式 fixture 的 x790 victim（cast700／center590／radius220）確認仍受傷，x1000不受傷。不能重用「所有傷害 victim 都須在 cast range內」造成範圍技能回退。
- 相依決策：GameWorld::faction_of 仍未實作，不能以「非敵人」推成「盟友」，本批未擅自新增友軍治療或 ABI 方法。输入 admission／queued 不是效果生效；未宣稱拒絕效果會還原 input handler 清掉的命令。
- 操作錯誤：再次猜 omb/src/input_buffer.rs，實際是 lockstep/input_buffer.rs；已先用 rg --files 列實際路徑。合併來源／git status 輸出也截斷，必須縮小每次輸出、以明確行數補讀，而不是將工具呼叫當作已讀完。
- 防再犯與確認：新 base cast_preflight2＋model1、更新正式AoE1，邊界／一 raw 超界／死 caster／錯 heal target／rank資料與 HP-CD 全數通過；同批原 generic3與固定Lua include5通過，UE生成--check通過。沒有 OmGame／stage／網路／全套驗收。

## E168：單體名稱不等於 AoE；群聚中心會改變測試範圍（2026-10-05）

- 發現與決定：三原型只用 Damage／HealSelf，即使名為 volley 仍只有單體。新增通用 area_damage point／per-rank amount-radius 與共用 model／Lua FFI 驗證、Rust host query／穩定去重／全序列 preflight；不加角色分支、不修改 GameWorld ABI。radius／range 必須至少 1/1024，不能接受量化後為 0 的「正數」。
- 實際測試失敗：正式五人 fixture 將 player5 放在 x900，誤認必在範圍外。Bot 正確選 x690，涵蓋三名敵人；900−690=210，小於 radius220，因此 player5 HP1000→890，測試預期1000錯誤。修 fixture 為 x1000，保留最大覆蓋與正式傷害；不得為通過測試降低 Bot 策略或忽略第三名受害者。
- 複雜度決定：不得對所有披露單位執行無界平方級群聚。先穩定排序最近512可用單位，最多32候選中心，最多16384次覆蓋檢查；超額 gameplay resolved effects128則全招拒絕，不任意丟棄傷害。這是工作量限制，不是穩定60FPS或全圖最優聲明。
- 操作錯誤：PowerShell 下把帶 *.rs 的路徑直接交 rg 導致 OS error123，應傳實際目錄搭配 -g '*.rs'；也猜不存在 native/systems/player_input.rs／native/systems 與 OmGenerated/manifest.json，再次造成 error2／3。已用 rg --files 確認 game_processor.rs、scripting/dispatch.rs 與 om_codegen_manifest.json。不得以檔名印象取代實際清單。
- 輸出截斷：合併 context 的大輸出與單檔 token cap 截斷。已以 bounded line ranges 補讀缺段；後續不要把一次 Get-Content 稱為已讀完，必须檢查工具是否截斷。
- 當前確認：model1、base area2、core area1、原三英雄十二招1，共五项指定測試成功；正式 codegen 與 Lua 一真人九Bot60Hz preflight成功。新 generated metadata 仍須最後 OmGame／DLL stage／網路及UE整合，沒有把預檢當畫面或100場通過。

## E167：購裝游標不等於終局背包；正式交易規則不可在 Bot 重寫（2026-10-05）

- 問題與決定：若固定依「劍、劍、大劍」游標或看是否還有材料，合成後容易重新買材料，死亡也容易丟失階段。改宣告終局物品 multiset，先保留所有完成目標、遞迴推導第一個缺項；只看 owner 背包／金錢與公開 catalog。沒有 Bot 金錢或 Inventory mutation。
- 問題與決定：只看空格會錯拒滿格合成，另寫補差額也可能忽略重複材料。read-only preview 直接複製資料呼叫正式 buy_item，authority 仍重新驗身份／存活／範圍。六格與 goal 順序造成的無法組裝，正式 plan compile 有界預檢拒絕，不留靜默卡住配方。
- 實際編譯錯誤 E0425：moba_match 父層未匯入 ItemRegistry，新增 Bot resource lookup 後編譯失敗。已明確匯入 native::item::ItemRegistry，並用 try_fetch 讓缺少 resource 保守不購買，不破壞省略策略的既有行為。
- 操作錯誤：多檔 patch 中重複的 think_interval_ticks 行有不同後綴，預期 context 不符，整個 patch 拒絕且未修改。先讀精確 constructor 行，再以完整唯一 context 重做；不可猜測部分 patch 已成功。多檔讀取輸出也再次截斷，之後應分檔限行，不把截斷當完整閱讀。
- 防再犯：核心測試涵蓋重複／二階材料、完成不重買、保留終局目標、滿格合成、餘額／容量／循環、非法配置及 preflight 死局；正式 60Hz 測試涵蓋只送 ItemBuy、非直接修改、精確扣款、對手不變與 phase／死亡／範圍 gate。補 return_to_shop 的金錢門檻、披露威脅／中立、共用 threat helper；正式 Recall→完整 channel→home→大劍購裝，完成後不因金錢回城。最終指定 core 4＋base 2 成功，Lua 一真人九 Bot 新配置預檢成功；不是完整網路／Unreal／100 場驗收。

## E166：回城傳送不等於續航；恢復不可復活死亡英雄（2026-10-05）

- 發現：既有 Recall 只傳送，不恢復HP。若僅新增低血回城，Bot會在基地反覆Recall。決定新增可選權威基地恢復，Lua compiled rate／radius、Playing active delta、己方存活基地／存活英雄／非lethal生命／距離檢查；在post-combat及recall完成後、final EquipmentStats投影前結算。Bot不改HP或位置。
- 決定：配方sustain嚴格門檻與安全距離，當隊committed披露威脅→正式MoveTo撤退，無附近威脅→正式Recall；讀條不打斷，基地未恢復至離開門檻不送推線。省略policy保持既有行為，Policy啟用match的恢復旗標，真人也共用相同恢復規則。
- 操作錯誤：本輪仍猜不存在的events/outcome*、runtime/simulation_driver.rs、runtime/driver.rs、native/mod.rs、runtime.rs及game_proto*；已改用rg列出實際native/simulation_driver.rs／game_processor.rs／generated/game.rs。不要以既有檔名印象代替實際清單。
- 測試錯誤：先在base_content用只有core才有的篩選得到0tests，僅屬編譯證據；之後分別執行真正core決策與base ECS測試。ECS先存取private routes造成E0616，改由公開bases與Pos核對實際傳送目標；CProperty不是Copy，E0507修為只複製HP／maxHP欄位，不新增公開介面或Clone熱路徑。
- 大型範圍組合讀取仍有截斷，後續補讀必要區段；不能把被截斷的歷史或程式碼当作完整證據。
- Cargo指定scripts workspace之外的依賴package再加該package的--features被拒絕；改以workspace內base_content/runtime-lua-content啟用相依feature並選兩package，只計真正執行的template-id測試，不計base篩選0tests。
- 最後功能確認：core2/2、base3/3（含雙隊20ticks／40steps完整hash零repair）、template-id恢復規則1/1、新混合Lua配方正式Rust預檢成功；root／omb／omfue空白檢查通過。恢復資料改變的是完整catalog_data_hash，不要求僅呈現model的generated content_hash也改變。未stage或做完整Unreal對局，詳見bot-sustain-base-recovery進度檔。

## E165：共用 Lua 模板 include 必須涵蓋 FFI 產生器與建置相依（2026-10-05）

- 首次原型 ID 測試編譯失敗：猜測 HeroId 有 as_u16，實際公開方法為 raw。讀取生成器方法後改用 raw，不新增多餘轉換 API。
- 正式 base_content 建置失敗：gen_hero_registry.lua 原本只呼叫 builder({})，新增共用 Lua 模板後 ctx.include=nil；Rust／Unreal 完整內容 loader 能 include，不代表 FFI 產生器也能。
- 決定：FFI 產生器提供一般化 content-relative include，拒絕絕對路徑／父目錄逃逸／循環，所有 builder 共用 context；base_content build.rs 監看完整 templates 目錄，避免只改新 include 檔卻留舊註冊。保留 append-only 英雄／技能顺序，不複製共用模板或新增角色專屬 handler。
- 本輪大型歷史 context／來源搜尋輸出再次截斷；改分段補讀缺少部分。後續搜尋需限定來源目錄與輸出範圍，不能讓資產清單或合併多份長檔耗盡預算。
- 原型目前使用真正已支援的敵人單體傷害與自身治療，遠程連射不是 AoE，前排恢復不是護盾；mana 為內容 metadata，未支援消耗前不得宣稱有 mana 規則。完整三原型平衡／美術／100場留最後，不以新增名稱封關5.5。
- 修正後：附加ID與loadout指定1/1、三英雄十二次正式60Hz施法指定1/1、include正負向5/5、新混合配方正式Rust prepare-only成功；codegen隔離生成／check與正式來源生成成功。未編譯OmGame／stage新DLL，不將舊產物視為新內容證據，詳見moba-archetypes進度檔。

## E164：完整 TOML 不可用簡易讀取器重寫；監聽就緒不等於真人入局（2026-10-05）

- 發現：既有 Lua scalar reader 不支援完整 TOML 結構，拿來合併會破壞 inline table／array／multiline。對真人 AUTH 做遞迴合併還會殘留原玩家；換輸出目錄後相對 content path 也會失效。
- 決定：固定 lua-host 新增完整 TOML section-field replacement；指定欄位整個取代，其他值保留，所有 content path 絕對化，不修改來源。設定预檢由正式 Rust Setting／role compiler 執行，不載 DLL／World／socket。
- 發現：runtime presentation listener 在 KCP admission／replica 就緒之前輸出；同隊玩家共用以 team 算的 IPC 埠會衝突。
- 決定：只接受精確 player_id／team_id ready 行；IPC 埠依真人順序分配，Bot 不啟 client。Unreal 明確 presentation-only；準備或程序啟動都不宣稱 renderer／60FPS／完整對局通過。
- 決定：每次 spawn 立即存 own PID manifest；cleanup 身分檢查、反向 stop＋wait，錯誤不能被 cleanup_stack 的 pcall 吞掉。失敗記在 run/errors.md 並傳出；mock stop failure 是預期測試，不是真實 PID 清理故障。
- 本輪兩次組合讀取輸出仍超過預算而截斷；後續以窄範圍重讀必要欄位，不能把截斷視為已閱讀全檔。舊 E163 的閱讀習慣提醒仍有效。
- omb Cargo.toml 的 index／worktree 原本皆 mixed 換行，新增bin段落CRLF導致預設diff --check報cr尾隨空白；保留原文件，不大範圍改換行或autocrlf，以core.whitespace=cr-at-eol檢查真正空白，root／omb均exit0。
- 當前功能：host TOML 2/2、固定 Lua 配方／正式 Rust 預檢／readiness／mock lifecycle 5/5 passed；一真人九Bot60Hz prepare-only 成功，沒有實際 Unreal 對局验收。

## E163：Bot 對局名冊不是 KCP 真人認證；正式 tick fixture 必須保留接收端（2026-10-05）

- Bot roster不能塞進AUTHENTICATED_TEAM_BINDINGS補canonical projection。新增server-owned控制名冊，真人認證必須等於配方bot=false玩家；外部輸入先排除Bot／未知ID，內部Bot再經既有MobaMatch gate／canonical accepted projection／PendingPlayerInputs／phase dispatcher。KCP仍只授權真人。
- 第一次正式State::tick測試失敗「secure team outbound queue disconnected」：fixture helper把出站receiver丟棄，正式可靠隊伍發送正確回報斷線。修正helper回傳receiver並由測試保留；不得關閉可靠send gate、忽略錯誤或只改成SimulationDriver測試假裝正式server成功。最後正式server相關3/3 passed，安全兩隊輸入frame與九Bot實際位移亦驗證。
- 第二次fixture預期11輸入但只有2真人輸入：正式server的Wave B用一tick披露延遲，不同於舊headless delay0；第一tick缺owner disclosure，Bot正確fail closed。改先正常跑兩tick建立正式披露，不把delay改0或取消owner gate。
- 第三次已正常披露但只有9而非11：兩個Support初始在Carry跟隨半徑內、無在途命令，正確hold不發輸入。改驗精確controller ID序列（2真人命令＋7Bot命令），而非要求每位Bot每tick必發命令；後續正式60tick仍要求九Bot皆有實際移動。沒有修改AI政策迎合測試。
- 一真人Lua配方測試首次失敗：toml::Value直接try_into對字串map key "1" 拒絕u32（十Bot空認證表沒有暴露）。改以正式read_setting同路徑的TOML文字序列化→toml::from_str核對，不以十Bot配置掩蓋真人入口。
- 最後指定Lua配方測試1/1 passed，十Bot與一真人九Bot皆由固定Lua匯出／正式TOML讀入並驗證，並未修改正式AUTH型別。主server binary check exit0；無DLL stage／實際socket或Unreal完整驗收，詳見role-bot-server-control進度檔。
- 嘗試以core.autocrlf=false抑制diff warnings，反使既有CRLF module檔的CR被diff --check當成全檔行尾空白。恢復正常換行轉換並只設core.safecrlf=false後check exit0；沒有為了checker重寫全檔行尾。
- 初次role_server篩選未寫測試時0 tests，只算編譯確認，不能宣稱功能通過。一個running編譯呼叫只印output漏session ID；之後保留完整工具回傳，避免失去poll handle。
- 一次合併OpenSpec讀取與多次來源合併輸出截斷，後拆小段補讀；又猜scene/import_map.rs、runtime/team_projection.rs而不存在，已用rg --files／實際識別字定位。不得從猜測路徑或截斷輸出推定完成。

## E162：技能學習不能繞過權威扣點；Bot roster不等於真人認證（2026-10-05）

- 新學習政策只提出正式UpgradeAbility，owner點數／rank／等級門檻由原handler重新驗證與扣點／SkillLearn。僅有rank0施法策略會永遠跳過未學技能；採獨立有序學習步驟，等級blocked不阻塞其他合法步驟。異常rank以checked_add防溢位，不能直接改Hero假裝學習成功。
- 查核正式server canonical accepted input team取自AUTHENTICATED_TEAM_BINDINGS，而main同表提供KCP authorization。未來不能把Bot加進真人auth表來補projection，須分開server-owned match/controller roster與外部認證身分；本批沒有改這些邊界或宣稱KCP Bot已完成。
- PowerShell命令帶`{tasks,design}.md`觸發ParserError/Missing argument，整個讀取沒有執行；修為兩個明確路徑再讀。不得在PowerShell假設bash brace expansion。
- 合併輸出多次被max tokens截斷，已用明確小段另查upgrade handler／server tick／canonical accepted input；來源仍應縮小讀取範圍，不把截斷讀取當作完整證據。
- 本批core學習2/2、plan2/2、正式60Hz ECS1/1與固定Lua配置入口均成功，沒有功能測試失敗；不是完整框架／Unreal／LAN／100場驗收，詳見role-bot-learning進度檔。

## E161：tooltip preview不是AI執行語義；HP總差額不是單一技能傷害（2026-10-05）

- 已查明effects_preview僅預覽，改以Lua配方explicit intent與compiled target/rank/range建立通用政策，不以preview／slot猜用途。
- core測試首次E0382：CProperty非Copy，low=health後再次借用health；改測試clone後2/2通過。
- ECS期望80卻得169.121，移除TAttack後仍失敗，證明非一般attack元件能隔離。正式MOBA塔攻擊有私有NPC state，原fixture仍在650塔距離內；還原元件、改公開位置fixture離塔／camp範圍，最後80傷害與70自療精確通過。不得調低assert或改實際技能平衡來掩蓋混入來源。
- 又猜ability_runtime.rs不存在；後以rg --files確認為ability_runtime/registry.rs。來源路徑先列實際清單。部分合併輸出截斷後另依明確技能cast／NPC spawn實作定位，不能以截斷內容確認完整實作。
- 本批只有core2／正式ECS1／Lua配置入口確認；Mana未實作完整current值，不宣稱資源驗證、任意handler語義或完整對局已通過。

## E160：MoveTo admission 不代表該 tick 即時停止（2026-10-05）

- 新Support停止追逐測試首次失敗：提交到committed自身位置後，位置仍多移動約5.1units。原因是共用Dispatcher先執行既有移動，Moves才接收替換命令，不能拿headless ACK或接入當作即時停住。
- 修正Support對近距離在途MoveTo的去重，等待正式命令完成，不每tick追新pose替換造成來回修正；不改global phase、不直接清ECS queue。新測試驗admitted target精確／下一tick回到目標／不再重送，最後base相關3/3通過。
- 初稿曾引用queue.pending，讀實際HeroCommandQueue後確認欄位是queued，編譯前已修正；未捏造編譯失敗或測試通過。大段讀取一度截斷，時序結論依明確phase表及個別command實作，不採截斷文字推斷。
- 感知使用同baseline的public HP与render；缺健康資料不回讀敵方CProperty。Support配對來自完整role roster，不能只在Bot名單找Carry，否則漏真人。core相關8/8與最後ECS3/3通過，不代表完整Bot／網路驗收。

## E159：已列出入口後仍猜名稱／工具路徑（2026-10-05）

- 本批讀取 omfue/rust、scripts/moba_headless.lua、scripts/lib/common.lua 時遇到不存在路徑；其中已列出 run_moba_headless.lua 卻仍猜舊名稱。改以 scripts/_bootstrap.lua 取得 tools/lua/lib 的既有 JSON／path／process，不新增平台 fallback。
- 合併大量規劃讀取造成截斷，後續分段補讀；大型歷史 MD 應單檔、小段，不能以 command exit0 當全部內容已讀。
- 設計檢查發現單把 Bot 指令推入網路 ECS 會漏 controller ownership／正式 acceptance。決定先接共用配方與 headless 正式 driver；KCP 接線保留待辦，不以 headless 通過宣稱網路九 Bot 完成。
- 本批新 Rust 測試2/2與九 Bot ECS1/1、headless check與Lua配置入口通過；沒有新編譯或功能測試失敗。protoc fallback／td_rounds warnings 是既有建置診斷，不當作已修復。

## E158：MOBA-local key 不能當正式視野 canonical ID（2026-10-05）

- 第一版 Bot 整合測試取得0個輸入而非10個；舊 entity_key 是 id<<32|generation，Wave B canonical 是 generation<<32|id，兩者 namespace 不同。沒有放寬視野白名單，改用正式排列匹配自己／拆 target ID，並驗完整 generation。
- 修正後五位置10人60Hz正式 driver 功能測試1/1、core視野與身分3/3通過。新增 identity layout regression，不能只以純 planner 測試代替 ECS adapter。
- 本批又出現猜 native/mod.rs、native/visibility.rs 等不存在路徑，以及 PowerShell rg wildcard 路徑；已改用實際檔案與根目錄 -g，這些是重複操作失誤，不宣稱已永久杜絕。
- 數次合併讀取仍截斷；功能結論只採獨立小輸出的實際測試數／exit。本批不以截斷的整份 tasks 或文件內容當新的完整閱讀證據。

## E157：無內容 patch hunk 與重複猜 staging 入口（2026-10-05）

- server 設定 patch 兩次留下只有 context、沒有新增／刪除的空 hunk，被 apply_patch 拒絕，沒有部分修改；改成有實際替換／插入內容的 hunk 後成功。送出前應刪掉空 hunk，不重送相同錯誤。
- 猜 scripts/stage_base_content.lua 不存在；實際 build_ue_moba 呼叫 dev_run_freshness.lua --action stage-dll。應從呼叫者或rg --files -g '*.lua' 找入口，不因DLL名稱猜檔名。
- 本批 root scripts／UE 以現行debug DLL staging；omb/scripts 是不同舊副本，未被本build入口使用，不能拿三者混比宣稱全部host已更新。正式launch需沿既有配置指向當前scripts/base_content.dll，所有 peers同catalog重建；尚未做正式網路啟動。

## E156：feature-gated 測試零案例不是成功證據（2026-10-05）

- 新 Lua 層次驗證測試首次未帶 runtime-lua-content，cargo exit0但 running0；不計入通過。開feature後出現缺少 canonical_template_hash 匯入的編譯錯誤，補正確 omoba_content_model import，最後實際1/1通過。
- 同批 PowerShell 直接傳 omoba*／runtime_peer*.lua 作rg路徑被視為非法名稱；應用明確根目錄＋-g選擇檔案，或rg --files找入口，不能重複猜 shell glob。
- 多段輸出仍可能截斷，測試結論以實際命中的case／完成exit及獨立小輸出確認，不以總exit0或truncated當完整證据。

## E155：IPC 模式不能由位址有無推測（2026-10-05）

- 發現 PRESENTATION_IPC flag 雖指定，bridge 仍以空位址選路；若 legacy DLL／Story 完整，缺 endpoint 可能回落舊 gameplay。修正為保存 explicit intent，create 與 driver 雙 gate 缺位址拒絕，不建立 runtime／legacy threads。
- Unreal 同時新增明確 PresentationIpc 配置與 launcher flag；缺位址不借用 ServerAddress。測試須用有效 legacy inputs 證明不能 fallback，也以真實 localhost IPC 確認兩種 local mode 都 sim_thread=None。
- 盤點時猜錯 OmWorldBridgeActor.cpp 所在 module 與 runtime_driver.rs 檔名，rg 回報缺檔；改先 rg --files 定位 OmGenerated/Private 與 driver.rs。缺檔不代表該機制不存在，不能據此新增重複實作。

## E154：restart 相對 project 定位與 readiness 的證據界線（2026-10-05）

- Editor102680 第一次 wait-mcp45 秒逾時；日誌顯示 Engine Initialization 32.94秒、HTTP30000 已綁定，但初次 health I/O timeout 原因未確認，不能直接照 E149 稱為初始化超時。
- 再次檢查時誤在 monorepo 根目錄呼叫 restart，因預設相對 om.uproject 而 exit2。正確入口 cwd=omfue，或明確 --project 絕對路徑；修正 cwd 後同樣45秒 readiness立即成功，未重啟Editor或調高期限。
- 保留兩個失敗，不把 port bound 當 health成功，也不把 cwd錯誤當 MCP服務故障。

## E153：跨專案共用 Editor executable 的 Live Coding 鎖（2026-10-05）

- 本專案 Editor 101752 已退出，但建置 90267 exit 1／UBT OtherCompilationError：Unable to build while Live Coding is active。另一專案 PID96936 使用相同 UE5.8 executable，未停止該程序。
- 讀引擎 HotReload.cs 確认鎖由 executable 路徑命名，不包含 project；BuildConfiguration.cs 提供 -NoHotReloadFromIDE。
- restart 所有 offline build 先驗本專案 Editor 不存在，再帶 -NoHotReloadFromIDE 與 -NoEngineChanges，避免跨專案鎖阻擋，同時禁止覆寫共用引擎產物；不是刪鎖、停止其他專案或修改引擎。
- 保留首次失敗；只在修正後重建本功能，不重跑完整驗收。若引擎產物真的需要改動，應由 guard 明確拒絕，不移除保護繼續建置。

## E152：相容宣告不能繼續隱含專屬派發（2026-10-04）

- 舊實作把 Saika 分支移出通用模板後，仍由三個 generic hook 自動呼叫 legacy adapter，包含四個技能 ID 判斷與 payload fallback；僅確認模板內沒有英雄名稱不足以證明正常路徑通用化。
- 先 MCP 查唯一子 Blueprint 已只用通用事件，再刪除自動派發與轉換器，保留 reflected API 的顯式相容呼叫；測試必須同時證明通用欄位保留與舊回呼不觸發，不以「舊回呼收到」當正常成功條件。
- 本批讀取累積過大的 OpenSpec 文件時仍遇到工具輸出截斷；應按小段讀取並分別限制輸出，不把 truncated 當完整讀取或證據。歷史文件目前包含大量逐批紀錄，不能依總任務數猜完成狀態。

## E151：Lua reserved key 與既有工具 API 必須先確認（2026-10-04）

- 新 planner 初次使用 node.detail.function，Lua 的 function 是保留字，載入即語法錯誤；改為 detail['function']，無資產修改。往後 JSON 的 reserved key 一律用 bracket access。
- 備份 hash 一度寫成不存在的 sha256_file，查現有 hash.lua 後在 apply 前改為 sha256；不要從其他語言的 helper 名稱猜 API。
- 查參考腳本時猜了不存在的 ue_art_replacement_smoke.lua；改以 rg --files 找實際 scripts/tests/ue_art_swap_editor_test.lua。不得把缺檔讀取當已取得依據。

## E150：MCP ok=true 的 graph preview 仍可能含拒絕項目（2026-10-04）

- 首次 preview 六個刪除項目，MCP ok=true 但 evtSaikaAction 被拒絕：cannot delete a structural/entry-point node。初版工具只檢查外層 ok，錯誤回報 success=true；該次只有 preview，沒有修改資產。保留原 report1791123876 與 raw 回覆，不能當成功驗收。
- 增加逐項 gate：preview 全部 would_apply=true、數量一致；apply 必須 applied=true 且無 rejected，之後完整 snapshot 核對。負向 CAS／未套用回歸測試保留。
- 普通五個節點用 CAS，入口則先核對 isolated exact event，再由 documented delete_nodes allow_destructive=true 精確刪一個；備份／raw 操作／部分失敗報告保留，不自動重送整批。
- 最後真實遷移1791124011成功，保留25nodes完整內容／接線一致、compile/save成功；重跑無改動。13個直接測試與動畫派發單輪通過，詳見 Blueprint generic event migration 進度檔。

## E149：Editor 初始化時間不能與 MCP 等待期限混為程式錯誤（2026-10-04）

- Editor 101752 首次 wait-mcp 45 秒 exit 6；專案 om.log 顯示 Engine Initialization 實際 46.06 秒後完成，process 身分仍是本專案，沒有 crash 或 modal 證據。
- 初始化完成後再做一次相同有界 readiness 檢查即 HTTP30000 成功；GenericAnimationOverlay 一次1/1通過。沒有反覆重啟 Editor、調高完整驗收 deadline 或停其他程序。
- 防重犯：先讀本專案啟動階段與 process 狀態，區分尚在初始化、服務故障與功能失敗；保存首次失敗，不把二次成功改寫成首次成功。

## E148：stage SHA 一致不代表最後 source 已建置（2026-10-04）

- 本批 build-only 進行時追加 unknown hero fallback 修正；初次 bridge DLL 22:14:06 早於 projection.rs 最後修改 22:14:40，雖然 --verify-staged-only SHA 一致仍不能當最新實作證據。
- 決定：停止本專案新開的 Editor 47420，再以既有 Lua build-only 補增量建置；此後不再同時修改程式。原 Editor 83404 已先核對完整 project command line 再由 project-scoped restart 停止，沒有停止其他專案。
- 防重犯：最後程式修改完成才建置；staging 一致與來源建置成功分別核對。不能讓讀取 hash 的成功掩蓋建置／編輯競態。最後 build-only 27886 exit0、DLL 22:16:58 晚於最後 source 22:14:40，stage c31089…一致；詳見 generic-animation-overlay 進度檔。
- 另有工具輸出合併過大被截斷；改按檔案／範圍拆讀。截斷不能作完整日誌證據，建置以該工作 session 的 exit 與本功能 Editor 報告確認，不讀其他專案可能覆寫的全域 UBT log。

## E147：非零 overlay 不等於 Saika 狙擊動畫（2026-10-04）

- bridge 寫死兩個 buff 名稱，Unreal 又將全部非零 overlay 指向 sniper_mode／sniper_walk；新 buff 即使有 metadata 仍會被誤解。
- 共用 Lua metadata parser → bridge priority／stable ID 選擇 → 生成 native OverlayName／Walk／Stand 欄位 → 共用 Unreal model。實際 buff 決定適用，缺映射回普通動畫，不使用角色旗標猜名稱。
- 同時移除未知 hero 冒充 Saika catalog 的回退；新增直接回歸。負向與新英雄任意名稱測試保留，仍不刪有資產引用的 Blueprint 相容介面。

## E146：新增直接測試仍須遵守 fixture 與 CLI 契約（2026-10-04）

- 新增 codegen 測試第一次從兄弟 test module 使用 private helpers，編譯 E0603；只將 cfg(test) helpers 可見度調整為 pub(super)，不擴大正式 API。
- 第二次 fixture 的 ability 少了 max_level／levels，先被既有內容驗證拒絕。補齊 max_level=1、levels={{}} 後兩個新測試通過。負向測試必須從合法基線改一個目標欄位，不把無關 schema 錯誤當目標驗證成功。
- codegen CLI 曾誤用 --out-dir，exit 1；讀實際 main.rs，改用 --out 後生成及 --check 成功。呼叫前先查既有入口參數，不能從 Rust CodegenOptions 欄位猜 CLI。
- 初次查讀輸出再度截斷；改以已知函式／明確行數拆讀並提高對應輸出額度。不得把截斷輸出當完整內容證據。延伸調查另猜測不存在的 OmNativeVisualActor.cpp；該次搜尋不是取得原生動畫處理證據，後續必須先 rg --files 確認實際檔名。

## E145：通用英雄事件 API 底下不能仍靠角色名稱生成事件（2026-10-04）

- bridge 只有 sniper_mode／three_stage 的 hardcoded buff→ability 轉換；任意新英雄無法沿用該事件來源。
- 改由 Lua buff_visual.ability_binding → 型別化生成 manifest → numeric buff 索引，名稱不參與投影分支。未知引用／模式／欄位／重複綁定與不一致 ID fail closed，未宣告綁定不猜測。
- 任意 custom ID 的八種 lifecycle 組合、非法 metadata／catalog 與既有快照回歸直接測試通過；沒有因此宣稱 Blueprint 相容介面與動畫分支全數遷移。細節見 generic-buff-ability-binding 進度檔。

## E144：即時proto與受版控fallback可能不同，即使server check通過（2026-10-04）

- 收尾查核發現core用vendored protoc即時生成，但omb無protoc時讀src/generated/game.rs；先前新增fog_grid後尚未同步該受版控fallback。初次server check成功不能證明fallback含新欄位。
- 使用既有core build.rs的OMOBA_UPDATE_PROTO_FALLBACK=1生成選項刷新，還原呼叫前環境值；不手寫generated structs、不安裝protoc或新增fallback。之後查明兩個fog_grid欄位，重新server check。
- 同輪曾猜測不存在的src/game_proto.rs；以實際build.rs與generated路徑定位。不把缺檔或rg無match當資料已取得。
- 收尾：既有生成選項及server重新check均exit0；fallback含FogGridPresentation、snapshot fog_grid及rebase fog_grid。後續修改proto必須同步受版控fallback，再檢查無protoc的server路徑；單查cargo check不是一致性證據。

## E143：rebase缺少已綁定迷霧與立即恢復快照（2026-10-04）

- 現況：恢復manifest只帶world，runtime清空fog；主／catch-up分支只送ResetView後等下一個gameplay frame，不能當完整迷霧恢復。
- 決定：manifest v2以新hash domain綁typed grid；v1無grid保留原hash，v1夾帶grid／v2缺grid／未知版本拒絕。恢復保留同隊同場探索，active epoch與後續frame／bootstrap一致；明確新view reset仍清探索。
- 主／catch-up改用同一helper，critical reset→恢復snapshot，另保留latest供renderer重連；不增加玩法world。core兩個新rebase測試、兩個原tick／sequence測試與runtime check通過，完整實戰驗收仍未執行。
- 操作錯誤：Select-Object -First誤填sixty，且同錯誤重複一次；數字參數必須直接填65，修正後讀取，不聲稱先前讀取成功。沒有新增PS／Python fallback或變動外部程序。

## E142：小地圖fog不能保留lease指標或把網格外當可見（2026-10-04）

- 通用模型綁定expected team、ABI11完整frame後驗全部geometry／三態／audience／tick，複製cells，不保留租約指標。control先返回保留狀態，full reset／Stop清空；同epoch幾何變動、同tick衝突、倒退拒絕。
- 三態row-run mask在公開路線／地形之上，memory／live在其上；網格外與schematic padding明確遮罩。不能用取樣fog替代entity target gate或在UE生成隱藏位置。
- 操作紀錄：建置輸出仍截斷；共用Engine Log.txt已被另一專案覆寫，不能當omfue證據。本次build task exit0／staged SHA及專屬Editor test report才是依據；未修改外部專案或停止其程序。
- E141外部lock已解除，本批正常build-only成功、owned Editor83404，本功能MinimapFogGrid一次1/1 passed；未跑全套MCP／PIE／雙UE60Hz。詳見unreal-minimap-fog-grid進度檔，完整rebase／整合仍待完成。

## E141：共用引擎 DLL 被其他專案 commandlet 鎖住（2026-10-04）

- ABI11 build-only 中 OmEditor／OmGenerated／OmRuntime compile/link 成功；完整 target 的 NetCore.dll link 失敗 UBA9001／LNK1104，build_ue_moba 依規則非零退出。不能把局部模組成功当完整建置。
- 讀取 actual UnrealEditor-Cmd.exe PID68548 command line：C:/portable/OpenKoikatsu/Saved/Tests/PaintSpray396/FrameFollowProject/OpenKoikatsu.uproject，執行 ok.PaintGalleryAudit。不是 omfue，不在停止範圍；保留其程序，不把停所有 Unreal 當一般修正。
- 決定：記錄外部 shared engine lock，停止重複整體 UBT；本專案 bridge 以既有 build_bridge.bat 收尾並驗 built/staged SHA，完整 Unreal build 等外部檔案鎖解除後才重跑。未新增 PS／Python fallback、修改引擎 link 設定或隱藏失敗。
- tool 結果含重複長 build logs 再次截斷；關鍵失敗來自可見明確 link 訊息，另以 actual process command line 檢查原因，不推測程序歸屬。

## E140：正式 fog 不可沿用無 geometry 的 legacy tiles／過期 ABI smoke（2026-10-04）

- 決定：增加 typed FogGridPresentation schema 1 與 ABI 11 OmFogGrid，保留 Q10 座標、三態與 audience；舊 tiles 不改意義，正式資料存在時非法也不退回 demo。
- lease 自有 metadata／cells；busy ring retry 保留結果，不覆寫仍有 reader 的舊槽。4 個本功能指定 Rust 測試成功，Unreal 建置結果見 authority-fog-ipc-bridge 進度檔。
- 發現 C++ header smoke 尚固定 ABI 7；更新到 11 並實際引用新 grid 成員，後續 ABI 改動須一起修改，不只改 Rust constant。原 terrain 測試改用 OM_ABI_VERSION。
- 操作錯誤：初次廣泛 rg 指定不存在的 scripts/build_ue.lua，且多段讀取再發生截斷；後續改讀實際 build_ue_moba.lua 與縮小區段，不以未取得全文當驗證完成。
- 建置前查核先前 owned Editor PID102392 的 exact project command line，僅停止該 project scope；restart 回報 force_terminated=true，另查 PID 已退出後才 staging。沒有停止其他應用、操作資產或跑完整 MCP 驗收。

## E139：fog 發布與快取必須同時約束 audience、封包及生命週期（2026-10-04）

- 決定：從 compiled Lua map 產生 bounded geometry，重用正式 Wave B read view；每隊 projector 在編碼／padding 前發布 FG01，私人 bootstrap 保留同一資料，不傳 source／hidden IDs、不加入 gameplay hash。
- runtime 先驗 bootstrap／event envelope、team／epoch／tick／geometry／衝突，再保留最新合法結果；失敗不猜全可見，production 清空 cache。缺更新保留，verified rebase 清空後等待合法新 epoch 資料；完整 rebase／IPC／UE 尚未串接完成。
- 追趕略過 intermediate 呈現：修正成每個成功 Applied 後立即借用 public events 留存，不能只在 extract_presentation_source 時更新；Duplicate／Rejected 不收。再查詢时曾猜測不存在的 team_wire_validation.rs，改用已知目錄搭配 --glob，不把缺檔當完成檢查。
- 操作錯誤：PowerShell 的 `{design,tasks}` 路徑展開造成 ParserError；直接交給 rg 的 `src/*.rs` 造成 error 123；猜測根 Cargo.toml 造成 error 2；多檔廣泛查詢再次截斷。後續明確列已知檔案，目錄搜尋使用 --glob，逐區段讀取。這些失敗不是成功讀取或實作證據。
- 確認：6 個指定核心測試、另 1 個真實 selective replica 追趕留存測試及 client runtime cargo check 通過；不重跑 MCP／Unreal／雙玩家／完整核心 suite。詳見 authority-fog-publication 進度檔，不勾選完整 6.1／6.2。

## E138：正式fog核心不可重算另一份玩法視野（2026-10-04）

- 權威已有WaveBReadView及共用line_of_sight，新增grid只從正式source／occluders取樣；不能從safe hero位置再猜700半徑，更不能拿碰撞地形當遮蔽。
- grid geometry先checked乘加／容量4096，wire FG01包含team／epoch／tick；decoder先驗全部size／states才配置，拒絕錯team／epoch／truncated／未知version／trailing bytes。
- sources radius<=0不平方成正半徑；raw差值用i128，平方距離用u128 saturating處理極端值，不沿用可能overflow的Fixed64運算。
- explored保存在每隊AuthorityFogGrid，reset必須epochadvance；同tickimmutable／倒退tick拒絕。不以呈現網格取代原entity disclosure／stealth／input gate。
- 多檔查詢輸出仍發生截斷；本批只以完整取到的signature／資料結構實作，縮小後續輸出。成功確認與未接網路／UE的限制見authority-fog-grid進度檔。
- 本功能3個指定測試通過，含shared LOS tree blocking與極端座標，沒有重新執行343個其他核心測試或UE全套；網路／Unreal尚未接上，不能宣稱正式fog呈現完成。

## E137：DemoFog不可直接當正式MOBA戰爭迷霧（2026-10-04）

- 實際fog_cache.derive對所有safe entities套DemoFogCache；其10-unit tiles／700半徑不是版本化權威視野或explored資料。ABI也缺grid geometry／provenance，不能憑固定尺寸畫正式小地圖，更不能從empty列表猜全可見。
- 改在production envelope辨識權威safe phase HUD metric後永久停用該session的demo fog／circles／trees／polygons，清cache；reset／缺HUD不回落demo。legacy/demo保留原路徑。這不更改真正authority visibility或safe披露，只禁止假呈現。
- 小地圖明確VISION N/A，公開geometry／live與memory仍保留；完整fog底色尚未實作，需後續正式契約，不以邊界修正勾選完整任務。
- 初次patch又使用不完整MarkerColor簽名導致整批拒絕；以完整函式行重新套用，新增runtime／UI確認，禁止把patch error當已寫入。
- 本批停止先前owned Editor106460並另查PID退出後建置，未操作資產或擴停止範圍。結果見formal-fog-boundary進度檔。
- Rust指定測試1 passed／runtime executable build exit0，OmGameEditor Succeeded，MinimapFogBoundary單輪success=true。ABI10未改但bridge dependency重建，stage改c66b56…，不能沿用前批554332…報告；沒有實作正式fog底色或全套驗收。

## E136：小地圖不能把owner當team（2026-10-04）

- OmFrameEntity.owner是player ID，owner>0不表示敵人；以前所有其他玩家被画紅。同隊映射改用權威公开scoreboard roster，先完整原子驗證再使用，缺值／重複／不合法時未知。
- roster只補已披露live marker的team，不用公開player清單新增位置；記憶仍不帶owner／team。NPC無owner保持灰色，不以spawn或位置猜隊伍。
- owner i32保留Rust u32 bit pattern，roster查詢轉回u32；high-bit fixture驗證。不解釋-1哨兵為u32::MAX合法owner，這個既有ABI歧義須另行schema遷移，不能偷偷猜。
- 查找再次猜OmScoreboardModel.cpp不存在；改rg --files確認目前只有MinimapModel。patch首次使用不完整Project函式行導致拒絕，確認實際完整簽名後重套，不將失敗視為寫入。
- 本批Editor由前批啟動、無資產修改；restart stop回報force_terminated，另驗原PID107948已退出才建置，不擴大停止其他專案。
- 確認結果記入minimap-teams進度檔，完整套件與雙UE長測不在本批執行。
- 本批build-only exit0、OmGameEditor Succeeded；MinimapTeams單輪1 passed／0 failed／error_count0，独立report已保存。未把十人fixture稱為十人網路實戰或像素驗收。

## E135：小地圖安全記憶不能混入live與目標資料（2026-10-04）

- entities pointer 為空時的早退會漏 ghost-only 完整view；改成兩個獨立迴圈，live與remembered型別分開。memory不含entity reference／owner、不猜team、不自行TTL或更新hidden位置。
- 以render_id排除已有live／重複ghost，epoch0／非有限／範圍外拒絕。公開route／terrain決定bounds，memory不能擴張地圖或clamp到邊缘形成假目標。
- restart從根目錄執行stop導致誤找根目錄om.uproject，工具正確拒絕且未停止任何程序；修正cwd到omfue，停止本批先前啟動的Editor並另核對PID91152已不存在。後續restart命令一律指定project cwd，不使用root預設。
- 此次專案stop回報force_terminated=true；僅本批已啟動的本專案Editor，沒有執行資產編輯或對局。不能把stop dispatch當退出，建置前另查原PID。
- 成功確認與限制記入minimap-memory進度檔；全套完整驗收留最後。
- build-only exit0、OmGameEditor Succeeded；MinimapMemory單輪1 passed／0 failed／error_count0，包含正式WorldBridge整合，独立report已保存。不跑完整套件或雙UE長測，不以此勾選完整6.1／6.2。

## E134：小地圖漏掉公开地形與功能確認範圍（2026-10-04）

- 原小地圖缺route pointer就提早返回，範圍也只從兵線取得，無法顯示獨立公開地形。改route＋terrain共同計算示意範圍，geometry自有複製，不拿live entities或vision polygons補資料。
- terrain批次最多32；空指標／非有限／min>=max拒絕並清空完整view，不保留前一個有效矩形造成部分地圖。control不清、full empty清空。
- 初次patch的MakeLines context漏同一行後半段，整批未套用；先rg確認實際檔案未變，再使用完整行套用，不能把工具失敗當作已實作。
- OpenSpec文件多次合併輸出超量；以分段補讀截斷範圍。後續單次輸出應控制總量，避免在多個命令間累積截斷。
- 全套兩輪驗收不再每批執行；工具支援單一已知功能單輪與獨立報告，不以功能確認覆寫完整驗收證據。結果記入minimap-terrain進度檔。
- 本批build-only／OmGameEditor編譯成功；MinimapTerrain單輪exit0／success=true，獨立報告已保存。未執行全套／PIE／雙UE／60Hz長測，不宣稱完整UI或框架已完成。

## E133：獨立碰撞地形呈現與ABI10（2026-10-04）

- 共用PresentationExtras保留validated compiled map，full/lifecycle同一路徑；新增frame-owned terrain_rects與獨立UE ISMC，絕不能塞進blocked_regions／polygon_occluders而意外改fog。Unreal物理／overlap／navigation全部關閉，Rust仍持唯一collision rules。
- 首次編譯E0063：原PresentationExtras完整literal漏public_map；E0596測試extras未mutable；E0277新增OmTerrainRect不必要derive Debug／Default而OmVec2不支援。補明確None／mut，僅保留所需Clone／Copy，不為方便fixture擴大基礎型別改動。
- frame結構增加欄位必須升ABI10並同步cbindgen staging與Unreal建置，不能沿用9。empty header／slot初始化與publish均需帶新Vec與null pointer，不能讓frame pointer引用臨時data。
- 查找再猜scripts/ue_bridge_stage.lua不存在；改用rg --files定位。合併讀取過量截斷後補讀所需片段，不以截斷尾部猜函式。
- build-only exit0：ABI10 header／bridge同步stage、OmGameEditor Succeeded；bridge55＋1ignore／integration2＋1ignore與Lua terrain13斷言已過。依使用者最新指示，MCP／雙UE／60Hz完整驗收集中到最後，不再每個增量重跑；不是地形美術／完整地圖或60FPS完成。
- 建置後查header又誤用OmRuntime/Public路徑；實際產物在Source/ThirdParty/OmBridge/include/om_bridge.h，建置輸出已明列。查找須先使用實際輸出或rg --files，不再猜目錄。

## E132：編譯地圖幾何單一來源與開局驗證（2026-10-04）

- 發現權威MobaMatch與初始KCP bootstrap各自組装相同矩形corner；抽compiled_blocked_regions為唯一轉換，後续Lua地圖不需新增角色／map專屬C++或BP。
- 只驗map id／hash不能確保client收到相同碰撞。validate_metadata要求compiled map同時有唯一schema1 blocked-regions及canonical bytes一致，runtime bootstrap與正式ready都拒絕缺失／重複／錯配，legacy無compiled identity保留。不用預設地圖或空碰撞fallback掩蓋錯配。
- decode_public_blocked_regions原先在檢查內容前依wire count配置Vec；加入剩餘bytes上限後再配置region與point capacity，u32::MAX攻擊輸入測試不可觸發巨大配置。
- 本輪大段合併讀檔再截斷，已分檔／分段補讀完整OpenSpec。patch fixture空白context不符導致整批未套用，讀實際行後重新套用；禁止猜格式或把失敗當已寫入。
- 查active-session再次猜不存在路徑；改從實際run目錄與報告processes讀取，不拿缺檔當程序退出證據。大量列檔輸出仍須縮小到具體suffix／子目錄。
- 最後core343／base108（227.93秒）／server156＋1ignore／runtime62＋8ignore＋3integration／bridge54＋1ignore＋2integration／1ignore全過；codegen11／16與OpenSpec strict／diff通過。format後重新build／stage，OmGameEditor Succeeded，bridge2d2e19…／script三份719264…獨立一致。
- 真實新版雙UE60Hz1791116233 success：双route4-2-4／minimap3／普通移動；保存三方team1 41 PASS rows／27 unique ticks、team2 43／27至3240，零FAIL且所有已驗parity=true。報告SHAf45237…，五owned PID另驗皆退出，post-run stage仍一致。沒有PNG／LAN／稳定60FPS／地形營地美術驗收，整項仍未完成，完整紀錄見compiled-map-contract進度檔。

## E131：公開三路地圖身分與 Unreal 路線（2026-10-04）

- 缺口：Unreal共用route actor已存在，但IPC只傳lane_length，bridge把三路畫成一條直線。決定用public bootstrap map/moba-layout schema1發compiled map id／catalog hash，runtime bootstrap與bridge握手都嚴格驗證，使用同一Lua生成常數恢復完整route。不傳aggro／camp timer，不把collision terrain冒充vision occluder，不新增角色C++／BP。
- 新增RuntimeReady protobuf7／8後，舊fixture literal漏新欄位E0063；補Default並將production ready封裝成ready_envelope_for_start，真實TCP晚連線／重連測試沿用此函式。同步更新proto與checked-in prost fallback，不只改其中一份。
- Windows字面presentation*、猜render_projection.rs／codegen/templates／ue_minimap_observation.lua／scripts/lib失敗。實際列檔後定位；沿E128規則先rg --files再查內容。本輪大段exec合併输出再截斷，已分開讀完缺漏context，後續不能靠提高單command上限忽略exec總上限。
- 暫只封關public identity／三路route流程；營地／地形美術仍待下一段，不放寬Lua導航安全限制，不把CPU actor同步log當PNG／60FPS證據。
- 最後core340／base108（214.43秒）／server156＋1ignore／runtime61＋8ignore＋3integration／bridge54＋1ignore＋2integration／1ignore、Lua觀測器通過。重新OmGameEditor Succeeded，stage90bfc4…與script三份65d6e1…獨立一致。真實雙UE60Hz1791115215 success／雙隊route4-2-4與native minimap3／正式snapshot移動；三方team1 62 PASS rows／46 unique ticks至5520、team2 70／45至5400，零FAIL／全部已驗parity=true。報告SHA6fe606…，五owned PID另查皆退出。不是PNG／LAN／60FPS／完整三路UI驗收；全項5.4／6.1／6.2保持未完成，完整數據見三路layout進度檔。
- 最後文件patch附帶不存在的design heading導致整批驗證失敗；移除無意義hunk再套用，沒有猜heading或忽略失敗。大段讀檔再次截斷後改用rg精確行定位。
- 第一輪雙UE1791114846失敗：team1單路，team2第三條route fatal Cannot generate unique name。初始KCP空bootstrap另有producer，public_metadata=空，第一名加入玩家永遠缺map；補相同compiled id／hash與terrain到外／內bootstrap，不只修改tick producer。初始碰撞也須在第一個MoveTo之前安裝。
- route actor MakeUniqueObjectName使用World當outer，但SpawnActor實際outer是PersistentLevel。多路名稱的數字suffix衝突，改同一PersistentLevel作unique outer與OverrideLevel；只修共用框架，不增加每map BP。原失敗report保存，五owned PID已另查皆退出。修正後必須重新build／stage／双UE，不調高timeout冒充修復。

## E130：共用 NPC 靜態避障（2026-10-04）

- 決定：抽出英雄既有 static_next_waypoint，三路 creep 追擊／回兵線與野怪追擊／回位使用同一公開地形 swept-circle；直線可通不做 BFS。單路 None／TD 保留既有行為；Lua 兵線與 camp leash 的生成限制暫不放寬。
- 首次 route 測試失敗：只看每 tick 的短 proposed step，繞障後下一 tick 又走回牆邊，最後卡在 x=55.78。修正為先檢查到完整當前 waypoint 的路徑，受阻就持續共用尋路；不能只證明每步沒有碰撞，還必須證明抵達與來回返回。修正後 shared NPC 60Hz 往返／route cursor／零負預算／blocked endpoint 測試通過。
- 第一次 patch context 太寬，把 regions resource 插入另一個同名 properties 區塊；編譯前以 rg 查實際插入位置，改用唯一註解定位到 NPC tick。後續 patch 必須帶足夠唯一上下文。
- 本輪 OpenSpec 大段讀取再次截斷；分段補讀全部缺漏，不把截斷輸出當作完整 context。查 export 時誤猜 runtime.rs（實際module分檔），沿E128規則只用rg --files定位，不再推測檔名。
- 最後驗證：core339／base108（204.58秒）／server156＋1ignore／runtime61＋8ignore＋3integration／bridge53＋1ignore＋2integration／1ignore通過，新NPC fixture5400雙隊steps每tickhash零repair、兵與camp越牆／回位Heal通過。真實三路KCP60Hz1791114001保存證據雙隊9／9 unique checkpoints至1080、零FAIL／parity全true，三PID另驗退出；不是KCP現場NPC跨牆。最後OmGameEditor Succeeded、獨立stage d6f3e…／script三份4305…一致、codegen與OpenSpec strict／diff check通過。無Editor／PIE／UE地圖或60FPS驗收；完整5.4仍不勾選，不沿用E129 SHA。完整矩陣見NPC terrain進度檔。

## E129：Lua 地形編譯／公開地圖接入（2026-10-04）

- 決定：Lua map 新增最多32個整數矩形 terrain（id／min／max），編譯產生 MobaTerrainConst、納入既有完整 map catalog hash／compiled agreement。選定 compiled map 才取代 BlockedRegions；None 保留原單路／TD 地形，不增加英雄 C++ 或 Blueprint graph。
- NPC 尚無通用避障；生成器先以相同 i128 swept-circle 驗證所有兵線100-unit corridor與野怪完整 leash＋100 margin 不被地形擋住。不得把這個安全限制冒充NPC已會繞障，也不允許為通過測試而關掉碰撞。
- 新測試初次編譯 E0308：Specs read_resource 的 `.clone()` 複製 Fetch 而非底層 BlockedRegions。改 `(*read_resource::<BlockedRegions>()).clone()`，避免保留 read borrow 時再 write_resource。
- 同一 scripts workspace 完整測試仍執行時重新編譯新增測試，Windows linker LNK1104 無法覆寫正在執行的 base_content test EXE。等待原 session 正常完成再串行重跑；不刪 target、不 kill 不明程序，不把 Cargo build lock 當成執行中 EXE 的保護。不同 target workspace 可以平行，但同一 test binary 必須串行。
- 啟用真實 Lua 地形後，舊完整 base106 回歸仍全部通過，但耗時301.43秒（之前無地形185.07秒）。先補 integer broad phase 與四點矩形 stack buffer，減少遠處牆的 narrow-phase／heap allocation；不用移除碰撞換測試速度，不把總測試耗時當成 server tick wall-clock 基線，最後版本需重新驗證。
- 查找測試入口又誤猜 verify_2player_ue.lua／lib/runtime_smoke.lua／run_3process* 路徑；改以 rg --files 得到實際 scripts/run_moba_runtime_smoke.lua。沿用E128防錯規則：先列已存在檔名，glob只用rg -g參數。
- 最後驗證：真實Lua地形7200雙隊steps逐tick零repair／hash／正式MoveTo detour抵達、base107（271.10秒）／core338／sim69＋8／template39＋23＋8＋2／server156＋1ignore／runtime61＋8ignore＋3integration／bridge53＋1ignore＋2integration／1ignore通過。三路KCP60Hz1791112988成功、9／8 unique checkpoint至1080、零FAIL、三owned PID另驗退出；KCP未現場跨island，不混稱。最後build-only Succeeded、獨立stage094de…與script三份c5c4…一致、codegen11／16與OpenSpec strict／diff check通過。完整5.4／UE地圖／60FPS仍未完成；不能沿用E128 SHA。

## E128：公開地形整段碰撞與 60Hz 尋路前置（2026-10-04）

- 問題：hero planner 僅檢查格點、同格／超過 96 格時直接回傳目標；移動只查終點。兩端合法不代表路段合法，薄牆／對角切角可能穿透。決定以 omoba-sim 共用 i128 固定點 swept-circle polygon 檢查每条 BFS edge、fallback 與移動／axis slide／near snap；零或負預算不移動。只查公開靜態地形，不把隱藏單位加入導航。
- legacy polygon f32 僅在資料邊界 round 成 raw Fixed64；幾何不轉回 render float。raw 座標／半徑限制 2^29（約 524288 world units），超界／非有限資料 fail closed，防止 i128 平方溢位。這不是完整 Lua 地形或 NPC navmesh。
- 初次正式雙隊地形 fixture 未發生繞路但已抵達目標；原因是 MOBA 英雄出生不是 (0,0)，牆被放在出生點後方。改從實際出生位置設置公開牆與目標，不 teleport 英雄、不修改 gameplay 繞過。必須同時驗證實際 detour、抵達、每 tick 雙隊 hash 與零 repair。
- 本輪讀取過大造成工具輸出截斷；改分段讀完 OpenSpec context。再次誤用 Windows 字面 fixed*／runtime/*.rs 與推測 team_projection_runtime.rs 路徑；改用已確認目錄配合 rg -g '*.rs'，禁止把 glob 拼入 Windows path 或猜檔名。
- 最後驗證：sim69＋8、core338、base106（185.07秒）、server156＋1ignore、runtime61＋8ignore＋3integration、bridge53＋1ignore＋2integration／1ignore全部通過；三seed地形3600雙隊steps每tickhash一致／無repair／實際detour抵達。build-only OmGameEditor Result:Succeeded、獨立stage79533…與script三份c6c6…一致。codegen11files／16inputs與OpenSpec strict／diff check通過；沒啟動Editor或宣稱UE地圖／60FPS完成，5.4仍不勾選。不沿用 E127 的 DLL／stage SHA。
- codegen --check 預設 content-root 是 scripts/lua_data、out 是 Plugins/OmRuntime/Source/OmGenerated，兩者不在同一個 cwd。先後只換 cwd 皆失敗；讀 main.rs CLI 後明確傳兩個絕對路徑：`cargo run --manifest-path omfue/codegen/Cargo.toml -- --content-root D:/code/omoba/scripts/lua_data --out D:/code/omoba/omfue/Plugins/OmRuntime/Source/OmGenerated --check`。不改生成檔、不把參數／cwd 錯誤判成 stale codegen，之後優先使用既有 Lua 建置入口。

## E127：Lua 野區營地與 60Hz 正式傷害整合（2026-10-04）

- 最後封關本輪單怪野區原型：base105／core335／server156／runtime61／bridge53及template38＋23＋8＋2全部通過；三seed野區18000與三路41400雙隊steps逐tick零repair一致。最後release DLL45ea50…透過真實DLL headless seed42勝利22745／四槽46-6-5-4／全tick replay／end1；stage7d934…再次獨立核對、兩headless PID退出。詳見野區進度檔；沒有LAN／UE野區畫面／60FPS證據，5.4仍不勾選。

- 初次編譯 `AiType::None` 命中 legacy enemy 的同名 enum，E0599。改成明確匯入 `runtime::native::comp::unit::AiType`，不用 glob 推測型別。
- 測試對 `Outcome::Damage` 使用 `..lethal(...)`，E0436；enum variant 不支援 struct update。改以 pattern 修改 fixture 的傷害值。
- 查找再次誤猜 native/visibility.rs、native/filtered_specs.rs、native/outcome.rs、native/mod.rs 與字面 `comp/*` 路徑失敗。已以 `rg --files` 定位；後續禁止自行拼猜路徑與將 Windows glob 當目錄。
- 設計：新增 HostileNeutral 而不修改既有 Neutral；kind=3 只披露可交戰分類，野怪沒有 VisionSource，AI 仇恨／回位／重生 timer 不傳客戶端。所有正規及 Lua direct damage 均檢查回位免傷；獎勵僅實際 first-lethal 的 roster 英雄領取，重複 Death 不得重複付款。
- 執行過程中的 compile／局部生成成功不算野區／5.4 完成；本輪最後驗證見上方摘要，僅單怪野區原型封關。
- 正式輸入／雙隊重播 fixture 首次不發生第二次擊殺：撤退到基地後營地距離 1063 超過英雄視野 1000，合法目標篩選一直拒絕 AttackTarget。不是 AI 卡住；改成先 MoveTo 回野區、可見後才攻擊，保留迷霧合法性，禁止測試用未知目標繞過。
- 第二次仍不擊殺，實際 trace 顯示 attack_seq 每 tick 增長但沒有命中：fixture 每 tick 重送非 queued AttackTarget，正式新命令會重啟 attack windup。改成在可見且接近營地後送一次，已有相同 active target 不重送；不改 production 命令取消契約，也不以直接傷害代替此正式輸入測試。
- 擊殺已成功但原 1800 tick 視窗不足以覆蓋完整撤退／回位／重返／擊殺＋15 秒重生，改為 3000 tick，擊殺後正式 MoveTo 撤離避免空閒普攻再次刷營地。三 seed 最後局部驗證均 6000 雙隊 steps、230 external effects、一次重生，hash 零 repair。
- template default features 僅跑 7 unit，不涵蓋 runtime Lua 驗證。補 `--features runtime-lua-content` 才發現新測試少匯入 content-model hash 函式（E0425）；已補明確 import。不可用 default 子集宣稱完整 authoring validation 通過。
- build-only 後又跑 bridge 測試，新建置目錄 DLL 與 Unreal stage SHA 不同，獨立 verify-staged-only 如預期拒絕。等全部 Cargo 測試結束，重新 build-only 並再獨立 verify，最後 stage `2223060b2ce8ab712ccb64dd45eedab9f208f5ba0cee2d34cc7a0df75f8b40c6`；不能混用先前 stage 當最後驗收。多個 PowerShell 診斷命令的最終 exit 0 不代表前面的 Lua 已成功，逐條讀取結果。
- 誤把 `omb/scripts/base_content.dll` 當 debug stage 目標；實際 Lua freshness 工具 stage 到根 `scripts/base_content.dll`，與 debug build／Unreal stage 三份皆 `26c588…54d8` 一致。legacy omb/scripts 檔未覆蓋，release headless 明確 `--scripts-dir scripts/target/release`，不拿錯 DLL 的 SHA 當 stale runtime。
- 三路完整測試加入營地後 seed42／539365380勝利tick改為7930／7776；記錄最後程式結果，不沿用上一版7870／7767。完整 base105 全過、野區三seed18000雙隊steps與三路41400雙隊steps零repair，並不等於LAN／UE野區畫面／60FPS驗收。
- 最後 review 發現 ScriptDirectDamage 沒沿 filtered authority-combat settlement 保護，可能在客戶端重扣 HP／回位免傷分歧。補與 regular Damage 相同的 DisclosedAuthorityCombatTargets gate、針對性 core 回歸；重跑受影響 suites 與最後 stage。此前222306…是修正前階段值，最終SHA與數字以野區進度檔最後更新為準。
- UE5.8 engine共用UBT Log.txt 顯示Failed但實際target是 `C:/portable/OpenKoikatsu/OpenKoikatsu.uproject`，已被其他工作覆蓋。不得把別的專案log當OmGame失敗；本輪restart明確檢查UBT exitstatus後exit0，不修改／終止外部專案。最後stage7d934…、script ae61…三份相符；補強後base105／core335／server156／runtime61／bridge53全部通過。

## E126：Lua 三路地圖與固定點數導航（2026-10-04）

- 最後版本base102全部通過；三seed1／42／539365380勝利4955／7870／7767，雙隊41262 steps逐tickhash無repair。實際release DLL三路60Hz headless seed42勝利12808、四招26／11／3／7、全12808tick重放。最後fullbuild stage127176…／MCP11BP／Editor67136兩輪19/19＋串行PIE通過，owned正常退出獨立確認；core334／server156＋1ignore／runtime61＋8ignore／bridge52＋1ignore／template37＋23＋8＋2／sim65＋8、codegen check／OpenSpec strict／diff check通過。5.4只完成三路原型，野區／地形通用避障／完整建築層次／Unreal map仍未勾選。
- Review發現cursor已complete後，若NPC被aggro拉離終點，原helper返回origin會永久停在追擊點；修改advance_route完成狀態仍朝最後點前進，新增回歸斷言。此變更讓seed42勝利tick由7920變7870；重跑全套及最後stage，不混用舊數字。
- 原生PIE煙霧fixture截圖仍是合成120Hz／3FPS且HUD空owner，僅能證明原生mesh／ghost與UI surface；不把它當本輪三路、完整HUD或60FPS的驗收。最終PNG已檢視、限制寫入進度檔。

- 新基地解鎖測試在第三座塔 HP 歸零後立刻期待 unlock，失敗。正式 outcome 是 Damage 排入 Death、下一次 drain 才 retire；修正測試按正式順序排空，不把 HP 歸零當實體已拆除，不修改 production 傷害門檻。
- 新測試 patch 猜測 `mod tests` 已有頂層 `use super::*`，anchor 不符而原子拒絕；讀實際 module 後使用既有模式的函式內 import。猜測 headless 檔名用連字號失敗，實際檔名 `moba_headless.rs` 已由 rg 定位。

- 新增 `moba_maps.lua` 初稿直接回傳 table，編譯失敗：`ctx.include` 的契約是 builder function，不是資料 table。已改為 `return function(ctx) ... end`；後續新增 Lua module 先讀相鄰模組及 loader 契約，不以一般 Lua require 的形式猜測。
- Windows `rg` 字面路徑 `tests*` 查找失敗；應搜尋既有目錄或先 `rg --files`，不得重複使用不存在的 glob 路徑。多檔輸出超過設定上限時只視為定位結果，必要段落縮小重讀。
- 本輪保留單路預設，三路 map opt-in；基地規則由 Lua 設定且只接受已實作的 `all_lane_towers`，傷害邊界必須再次檢查。固定點數路徑不是 navmesh，不能用路徑測試宣稱地形避障、野區或 Unreal 三路驗收完成。

## E125：Unreal 首次學習後施法驗收（2026-10-04）

- 最終1791105946 saved verifier success：兩隊學習2／cast3各一次，raw cast tick2524／2838 HP563200→706560，compiled Lua heal143360（140HP）exact；5,883快照，兩隊各25unique三方checkpoint至3000，post-cast4／2。五PID全部退出、來源SHA前後不變。最後fullbuild／MCP11BP／Editor76216兩輪19/19／串行PIE與stage0f471b…通過，ownedEditor退出另驗。回城／shop／upgrade共享PID身分verifier及8mock通過，舊kill-assist／lane XP原captures重驗通過；core334／base98／server155重新通過，依明確數值規則scope封關5.3為20/30。E121與全部其他任務仍未宣稱完成。

- 收斂5.3時舊回城run1791073998保存verifier誤判ue-p1還活著：PID72524已被Windows conhost.exe重用，原記錄是UnrealEditor.exe。新增共用只讀saved-process identity判定與8個mock正負向；只有不存在或明確不同exe才可證明舊owned程序已退出，同exe／缺identity／pid不符／重複role仍拒絕，絕不stop現在的無關程序。shop／Recall／upgrade保存verifier共用此判定。

- 修正後1791105946實跑success／cleanup true，兩隊原cast3 result與HP550→690／正CD。保存raw verifier因誤把稀疏CommittedVitals視為每tick存在而拒絕；已讀authority emitter確認hero每tick final CommittedEquipmentStats(kind22、40 bytes)前16 bytes是HP／maxHP，改核對此最終事實，不補造缺失資料、不放寬exact tick。原captures不重跑交易，重驗相同保存證據。

- 第一輪1791105739兩隊CtrlR學習與正式R input3送出、runtime AbilityCast ok，但沒有cast result log導致60秒gate逾時。新harness在讀applied_inputs前因control-only frame缺HUD提前return，漏掉合法ACK；決定與既有升級／Recall一致，先讀結果再檢查HUD。保留失敗run，不延長timeout、不以server接受冒充端到端成功；修正後重跑。

- 決定保留CtrlQ原模式，新opt-in學R後以唯一普通R binding正常送出；不注入治療／傷害／rank，等待正式玩法HP缺額（正常level growth也會擴maxHP，不必聲稱一定是戰鬥伤害）。驗收需原cast一次、status0、正CD與實際HP增加、raw final EquipmentStats與IPC逐筆一致、施法後unique三方hash及五PID退出。
- 本輪檢索猜錯bridge header目錄。先rg定位Source/ThirdParty header再確認共享enum，不新增硬編碼魔數；未造成建置或玩法變更。
- runtime61 passed／8 opt-in ignored、新舊Lua觀測器通過；完整UE與真實網路結果待追加，不把單元測試當端到端。

## E124：雙 Unreal rank0 首次學習（2026-10-04）

- 最末adaptive width版本重新fullbuild／MCP11BP compile／同Editor105396兩輪19/19全部通過，codegen --check／OpenSpec strict／diff --check通過。owned105396正常退出後inspect不存在；不是只派發shutdown，也未留下Editor或對局程序。
- 最終1791104682保存verifier success：5,932 live raw/IPC snapshots、兩隊原CtrlQ input2各一次、rank0→1／p1SP3→2／p2SP4→3、每隊25個unique三方checkpoint至3000（post-upgrade6／2），五PID另驗退出、證據SHA前後不變。長名layout門檻現在通過，四PNG仍有E121局部文字截短，不能宣稱GPU文字問題修好或整個UI完成；舊rank1→2保存verifier亦通過。
- 第三輪1791104474對局success／cleanup true：每隊input2一次、rank0→1、p1SP3→2／p2SP4→3、三方50／50至3000。保存verifier因新apprentice長ID的確定layout不足拒絕：touch desired120／lance117／mend119，但內寬116。不是E121「layout充足卻偶發GPU裁字」的同一證據；決定共用SBox寬依實際name desired＋padding12，min128，既有短名不改。不得直接改verifier門檻、縮ID或宣称旧GPU問題已修復；保留失敗報告與PNG後重跑。
- 修正cast／learning ID後full OmGameEditor build與stage f84a14…、BpGeneratorUltimate11BP gate通過；同Editor81376兩輪19/19含rank0文字與first-level6門檻回歸，owned81376正常退出另驗不存在。正式雙UE首次學習仍須重跑，不能把Editor unit測試冒充端到端。
- 第二輪1791104221 KEY_GATE顯示explicit0／pointer0仍queued0，否定前述pointer假設，已撤回那個production行為變更。實際程式阻擋：DispatchHud將rank0的OwnedAbilityIds清0（正确禁止未學施法），SubmitOwnedAbilityUpgrade卻也用同欄位gate，導致不可能首次學習。新增獨立OwnedAbilityUpgradeIds保留HUD catalog ID供升級入口，cast ID仍rank0清0，authority仍決定level／SP。兩個失敗run皆保留且owned cleanup verified，不把未知cursor根因或fallback當修復。
- 第一輪1791103763正式Ctrl+Q delegate各執行一次但queued=0，雙runtime只有Move input1，沒有upgrade提交；launcher失敗且五PID清理verified。檢查UpgradeAbilitySlot使用IsHudConsumingInput把滑鼠停在HUD也當keyboard輸入消耗。決定keyboard升級僅擋explicit bHudConsumesInput（文字／modal），不擋pointer hover；仍走正式反射／IPC／authority，沒有fallback至API。新增opt-in KEY_GATE診斷驗explicit／pointer，確切第一輪cursor狀態未記錄，不能把推論寫成已證實根因；需重跑。
- raw verifier patch 用 generic tick_rate anchor，誤放另一個 capture test，E0425 initial_rank／first_learning 不在正確scope。rg定位後移至明確 upgrade_smoke anchor；不放寬raw驗證，重跑編譯與兩種保存證據。
- Windows rg 不接受不存在的字面 `scripts/*tests*`／`Private/OmAbility*` 路徑；以 rg --files 定位或搜尋既有目錄。讀取猜測不存在的 cleanup 測試路徑失敗，後續不得猜檔名。多工具總輸出截斷時改單次／分段讀取，不能宣稱已讀完截斷內容。
- 決定FIRST_LEARN與舊UPGRADE明確互斥，使用相同正式Ctrl+Q綁定；前者server-owned apprentice四槽0，一出生點＋正常每級SP，後者仍rank1→2。不能從runtime FIRST_LEARN旗標繼承自動操作，仍強制關閉。此輪先驗Ctrl首次學習，不冒充學後四招／OS按鍵／GPU像素封關。

## E123：純 Lua rank0 英雄與正式網路首次學習（2026-10-04）

- 保存 verifier 不可要求 final checkpoint 計數等於 launcher 當時統計：launcher 取樣後至 owned server 退出前會追加合法紀錄。首次嚴格相等斷言失敗，改成獨立逐筆驗三方 PASS／zero-repair／frame hash、unique tick 與 post-learning 足量，且必須包含 launcher 原末次 tick 並至少涵蓋原統計；沒有改原 capture、縮短門檻或忽略 FAIL。
- 最終 base_content 全測試 98 passed；原 registry 硬數字與 server stream 問題都已修正後重驗。真實 KCP60Hz run1791102528，1,582 live snapshots exact raw rank／SP、每人升級 input3 只有一次、未學施法與 level6拒絕／学後CD／雙隊post-learning parity通過，三個owned PID獨立退出。UE生成與OmGameEditor完整build／bridge stage通過，不冒充本輪Unreal rank0按鍵／畫面驗收。

- base_content 全測試兩項只因硬編碼 registry=12 失敗，新增四招後實際16且生成清單／metadata已一致。改與generated IDs／實際manifest精確比對，另加全體ID唯一性與八個declarative skill的effect metadata斷言，不只是改成下一個硬數字。修正後完整98項已重跑通過。

- raw verifier 首次失敗：只讀 server.stderr，但 server 的 log4rs WARN 實際在 stdout。rg 查到每玩家未學施法／level6拒絕訊息，改讀兩個原始 stream 並精確核對 pid／slot，不以runtime replica的WARN替代authority證據。原capture不變，重跑只讀驗證。

- ECS fixture E0277：String 不是 Copy，不能 `["training_apprentice".into();2]`；改兩個獨立 String，修正後單項測試通過。Lua prototype 找來源時以穩定 id 查詢，不依賴 heroes 最後一筆；既有UE launcher明確關閉新 internal FIRST_LEARN flag，避免繼承環境把runtime injection誤當Unreal操作。

- 先前單路 server 寫死 training_luminary，新 Lua loadout 無法正常選用。決定新增 server-owned AUTHENTICATED_HERO_BINDINGS，必須屬於既有 authenticated roster 且引用 active compiled hero；缺值保持原英雄，沒有 wire 選角或信任 client 任意 hero ID。
- 新 training_apprentice 以 Lua append-only hero／獨立 apprentice_* abilities 宣告，四槽 rank0／一點、lance 第一級 level6。不能共用同 ID 再生成同一 handler，既有 registry 為逐英雄產生；用獨立追加 ID 避免重複 FFI registration，不改原 lumen_lance 的 level1 門檻。
- fixture 只用明確 FIRST_LEARN opt-in＋test-mode 的一般 renderer intent，不直接寫 rank/SP/傷害。先嘗試未學習施法／等級不足升級，再首次學習、同request重送、學後施法。必須核對 raw authority facts／原input接受次數／IPC rank-SP-CD／雙隊 post-learning hash，不能把 status0 transport acceptance 當 gameplay 成功。
- 編譯 E0271：request closure ordinal 預設 i32，request_id 要求 u64；顯式 ordinal:u64／常數u64，並把player_id複製到local避免closure持有self借用。修正後重跑，不放寬資料型別。

## E122：rank 0 初次學習與 Lua 出生配置（2026-10-04）

- HUD regression E0502：長持有 components mutable borrow 時不能 clone owner；改每輪 insert 已序列化 Hero，clone 前沒有可變借用。修正後必須重跑測試，舊工具 session 的失敗不能視為新 source 結果。

- 猜 omb/src/scene、omb/src/server 目錄不存在；改直接搜尋 omb，正式 handler 位於 src/state/resource_management.rs。後續不要假設其他 workspace 採相同目錄結構。

- json! test macro 不接受 Rust `[0;4]` array repeat 表達式直接作 JSON array，改四個明確元素；實作 library 編譯不受影響，補測前不可稱全測試通過。

- template-ids 全測試因註解中的 `Story/TD` 觸發既有「舊 Story 路徑」文字檢查；不是生成資料回退。改註解為 Story and TD，保留檢查，不放寬測試。

- test 編譯 E0594：HeroEntry 只有 Deref，不能透過它改寫共用欄位；改 `.common.moba_loadout`，不新增不必要 DerefMut。猜 `codegen/src/model.rs` 不存在，後续先 rg --files 定位而非猜檔名。

- 問題：single_lane 強制全部 rank 1，無法宣告未學習技能；ScriptCast dispatch 的 `max(1)` 可能使內部事件繞過正式 input 的未學習檢查。決定加入英雄 Lua 可選 `moba_loadout = { ranks = {0,0,0,0}, skill_points = 1 }`，預設維持舊四槽 rank 1／0 點；只在 MOBA match birth 使用，respawn 保存進度，不重發點數。
- 編譯 E0433：content-model 在 template-ids 的正常 dependencies 是 optional，只在 build-dependencies 永遠存在，不能讓預設生成常數直接引用該 crate。修為輕量 MobaLoadoutConst 邊界，由兩個 parser 轉換，不擴 script ABI 或強制 runtime serde dependency。
- 多檔 apply_patch 失敗：同一檔 hunk 先下後上不能定位；工具原子失敗沒有部分修改。先 rg 確認，再依檔案順序提交 hunks。
- rg 帶 Windows `src/*.rs` 路徑得到 os error 123；改目錄配 `-g '*.rs'`。大段合併讀取導致截斷，改分段完整讀 OpenSpec context；不可把截斷當已讀完。
- GameMode::Moba 是舊 Story 預設，不可拿它作新規則 opt-in。內部 rank-zero cast gate 改用 MobaMatch 存在，避免改掉 Story/TD 舊腳本 fallback；filtered 正式 input 原本即拒絕未學習 cast。
- 本輪測試與剩餘邊界見 rank-zero progress 檔。之前裁字沒有修好，不把 rank 功能測試當像素驗收。

## E121：名稱裁切改查實際繪製狀態（2026-10-04）

- run1791100483每隊前後所有matching font batch372 vertices／558 indices／max-relative371／invalid-relative0，排除batch-relative越界；真正backbuffer也仍截字。新增regular Editor opt-in拒絕／shutdown冪等回歸時，裸#endif patch誤命中最前面的宏fallback區；rg檢查立即發現並移出#ifndef，後續patch需完整上下文，未把未執行到的test當通過。

- 真正backbuffer run1791100233四張亦有截字（隊1after完整，其餘截短）；因此「僅Slate截圖重畫」假設不成立，不改正常UI或把capture工具稱為修正。保留真正呈現取證工具及原PNG，繼續檢查GPU input的batch-relative索引範圍，前次僅檢查global vertex range不足以排除錯batch引用。

- 本機SlateApplication::TakeScreenshot會PrivateDrawWindows再重畫，不是正常呈現backbuffer；新增opt-in真正backbuffer readback比較，尚須PNG驗證，不能先稱遊戲UI根因。UE5.8 OnBackBufferReadyToPresent參數是ISlateViewportProvider而非舊FRHITexture，已查本機介面。首編C2440：GetRenderer回傳raw pointer，不是SharedPtr；改非擁有指標與shutdown存活核對，禁止用shared ownership包raw引擎物件。猜SlateRenderTargetRHI.h未找到，实际介面為Engine/Public/Slate/SlateViewportProvider.h。

- D3D12 run1791099494仍截短，不改default或宣稱RHI修好。權威upgrade成功但cleanup_verified=false，事後五PID90268／88156／27924／75128／26092均退出；原失敗報告保留。process.graceful_stop在close-window timeout→stop後沒有等待實際退出，修成再次bounded wait＋assert；補無OS mock回歸，避免拿稍後退出冒充當次cleanup成功。另猜scripts/test_ue_two_team_observation.lua與tools/lua-host/tests未找到，實際測試在scripts/tests，以rg --files定位。

- run1791099179／1791099344：公開GetBatchData等待merge後，名稱rect內索引54／60／60／54，right332／432／525／618，兩隊前後相同；實際batch clip1280×720、stencil0，排除少送出字形／batch clip不足。仍不能以CPU提交冒充GPU像素。下一比較：雙UE launcher一直強制D3D11，而project預設DX12；加入白名單OMOBA_UE_RHI及report欄位，保留原default並明確測D3D12，不更改global engine或放寬視覺驗收。

- glyph診斷run1791098864每字Valid=1、合理XAdvance、同texture0、scale0.6660，排除缺字fallback；未因此宣稱GPU atlas已驗證。嘗試讀cached batch時GetCachedElementDataList是private，編譯失敗，撤掉該callback，不改引擎公開權限；不要只看rg找到宣告就假設public。FontServices.cpp猜名亦未找到，後續以rg內容定位。

- run1791098401：名稱獨立layer64／65／66／67仍截短，撤回無效layer隔離；不再重試同層merge假設。另SBoxPanel.cpp實際在SlateCore而非Slate，後續先rg定位。

- 診斷run1791098159：每隊前後各四個OnPaint的actual name完整、size116×18、cull／scissor均(0,0,1280,720)，名字都layer63；四PNG仍有截字，排除繼承clip不足。依本機ElementBatcher.cpp同layer merge及simple-text批次路徑，改四個名稱分別繪於Layer+1+Slot並回傳該層，與短rank／hint隔離；是小範圍render-batch假設，尚非已確認引擎根因，需真實PNG驗證。不改引擎或全局renderer設定。
- 查SlateBatchData.cpp不存在；用rg --files找到實際SlateRenderBatch.cpp，再由rg定位MergeRenderBatches在ElementBatcher.cpp，不猜檔名。

- 續做19/30的6.2增量，先保留E120全部反例；只在om-upgrade-smoke要求layout診斷時排一次OnPaint記錄，實際name／geometry／cull／GPU clip state／layer／frame一起核對。不再以cached geometry冒充實際paint，不先改width／wrap／volatile／clip override。
- 本輪合併讀長context再被截斷，已按單檔小段補齊，不把截斷當已讀完；後續診斷先build-only與真實雙UE，避免每次猜測都跑整套Editor gate，確定修正後再完整驗收。

## E120：技能升級綁定不能以API／queued代替權威與畫面（2026-10-04）

- 第八輪1791097735四PNG與尺寸／actual text再次核對：自身clip zone仍沒有消除偶發名稱裁切，撤掉這個無效override，不放開parent clipping。queued／rank／SP／raw／三方hash的60Hz增量已成立，但名稱像素驗收保持未完成；後續先取得實際OnPaint clip／draw資料再改，不重复wrap／volatile／clip zone猜測。保留失敗圖片及layout診斷，不把saved verifier資料成功當PNG成功。

- 第七輪1791097384actual name四槽都完整，before／after cached與desired全為116×18；saved verifier明確驗八筆每隊，原始authority仍正確。但PNG仍裁短，排除資料截短與名稱widget尺寸不足，ForceVolatile未解決，撤掉這個無效每frame重繪。下一步名称使用自身已驗116×18的ClipToBoundsWithoutIntersecting clip zone，隔離繼承的clip；不是放開整個HUD clipping或放寬驗收，需第八輪PNG，不宣稱根因。

- 第七輪前建置C2039：STextBlock FArguments沒有IsVolatile；查本機DeclarativeSyntaxSupport.h實際參數ForceVolatile後修正。後續C7732／C2679是此失效chain造成的連鎖，不改SharedPointer或引擎；建置失敗不得啟動舊binary冒充新修正。

- 第六輪1791096909單行模式仍有before與隊1after名稱裁切；不再猜wrap。加入opt-in前後每槽actual name／cached geometry／desired size診斷；僅四個名稱STextBlock設volatile避免重用cached paint，每frame固定四個短label，不是per-entity熱路徑。需第七輪用數據排除文字／layout錯誤，再核對PNG；不宣稱引擎根因或已解決。同檔多個Update段被apply_patch整包拒絕，合併並按檔案順序排列hunks後才重送，未把拒絕當已改。

- 第五輪1791096619四張PNG已人工檢查：after兩隊名稱／rank2／Ctrl提示完整，但before兩隊名稱仍裁短，三block不足以穩定所有畫面。依本機STextBlock.h的SimpleTextMode契約，ASCII content ID、rank／CD與Ctrl提示改單行simple layout，不用shaping／wrap cache；不是已證實引擎根因，需第六輪前後PNG再驗。不要再用Get-Content -First讀單行巨大JSON；本次又截斷，改Lua解析後僅印success／passed／failed。

- 第四輪1791096121操作／提示／rank／SP再次成功，但名字裁短仍出現，相同Text跳過與延後三frame並非完整解法；撤回根因已解決推論。改通用Slate名稱／rank-CD／升級提示三個獨立單行block，GetAbilitySlotText讀實際三個block重組；新增移除hint清除回歸，避免多行layout互相裁切。仍需第五輪PNG，不改玩法或增加deadline。

- 第三輪1791095707仍有隊1PNG名字裁短、隊2完整，固定wrap不是已確認根因／完整解法。檢查通用SetAbilitySlot每個persistent HUD sample無條件SetText；改EqualTo相同值不反覆invalidate，after截圖等三個rank2 presentation frames讓前面frame先繪製。這是呈現穩定化，不是GPU fence／同tick像素原子契約，不修改rank／SP／timeout；需要第四輪視覺QA。

- 第二輪1791095365操作／權威仍成功，提示已可見，但SetText後AutoWrapText的desired width縮小，PNG出現技能名稱裁短。對唯一AbilityTexts block設定與128格內padding一致的MinDesiredWidth／WrapTextAt116，關閉AutoWrap，避免動態文字用自己的舊desired width反覆縮小。仍需第三輪實際PNG，不能只靠GetText判定修好。

- 首輪UE啟動有handled ensure：ConsoleManager.cpp4916明確指出r.Mobile.VirtualTextures已deprecated；project DefaultEngine.ini仍有該False欄位。移除僅此失效欄位，保持既有desktop r.VirtualTextures=True，不修改引擎或擅自新增mobile平台設定；下一次啟動核對無此ensure，不把非致命ensure稱為崩潰。

- 首輪1791094787 launcher與獨立raw verifier成功，兩隊input2各一次、rank1→2／SP3→2；PNG視覺QA卻發現72×72 layout-unit技能格裁掉Ctrl提示及名字，不能以Slate GetText完整冒充像素可讀。改通用128×128格、名稱／rank-CD／Ctrl+實際鍵各一行，保留tooltip；需要重建與第二轮PNG核對，首輪只算資料／操作驗收。

- close_window逾時fallback stop後立即inspect仍可能短暫alive，首次assert非零；追加有界wait再inspect確認70396退出。stop返回不是退出證據，未擴大PID範圍。只讀查尚未啟動的ue-p1.log不存在，後續先看inventory；此輪當時實際在release scripts/core的LTO編譯，不是UE卡死。

- MCP QUIT派發成功但Editor70396在10秒後仍alive，不能當退出；get_editor_dialog明確無modal。以固定Lua host對已核對exe的owned70396發close_window、逾時才fallback stop，單獨inspect確認。合併PowerShell指令中第一個Lua assert失敗不會阻止後一命令，後續驗收需逐command核對；雙UE launcher已在release Cargo編譯中，沒有把首次assert當passed。

- ue_mcp.lua不支援--help（明確非零）；改讀實際CLI與MCP tools schema。--list的保存檔含完整tools，需Lua依實際name過濾後只印需要的schema，不印全部巨大JSON。full流程本次在核對本專案Editor79996後正常關閉失敗，既有10秒fallback僅force該PID子樹，不操作其他Editor。

- 現有Ctrl升級在普通QWER callback內查按鍵狀態；改明確Ctrl FInputChord與獨立共用回呼，不用測試旗標覆寫正式Ctrl判定。delegate驗收不冒充OS鍵盤注入。
- 本輪context混合讀取再次截斷，改小段讀回缺失內容；不得把截斷當已讀完或成功。驗收不得注入SP／rank，queued／ACK不是升級數值結算，需原始資料與UI核對。

## E119：升級门檻不能硬編槽位或只改UI（2026-10-04）

- 收尾：core333／base94最後防呆重跑、full UE／11BP compile1791094033／同Editor79996兩輪17/17／串行PIE與stop全部通過；最終stage e968182a…與codegen --check、OpenSpec strict核對。3902筆KCP60Hz證據是最後開局防呆前的有效內容回歸，不冒充新非法開局網路測試；實體Ctrl／升級HUD像素、rank0仍未驗收。

- 最後開局防呆變更後，build-only被本專案Editor95892阻止stage（exit4），不是編譯成功。改既有full流程，精確核對本專案Editor後關閉／重建／重啟；既有stop在10秒正常關閉失敗後僅force已核對PID及子程序（前次22308、本次95892），不是全域終止其他Editor。測試Editor程序不可用作未保存使用者資產的處理方式。大段合併讀檔再次截斷，後續改限量定位與單檔讀取。

- bridge完整測試51/52通過，單獨再跑仍失敗，排除單純parallel flaky；catalog fixture兩技能max_level4卻無levels，生成失敗導致catalog_id0。補完整fixture，不放寬production契約；完整重跑52通過、1 ignored。最後開局防呆後core333與base94再次通過；UE兩輪17/17與PIE通過，但不是實體Ctrl操作驗收。

- 首次UE build C4018：ABI State.level為u32，Required使用int32比較觸發warnings-as-errors；Required改uint32、Printf用%u，重新完整build。base94已過不代表Unreal已過。

- codegen首次21/24通過，三個舊fixture只有ability ID或max_level4卻只有零／一筆levels，被新一致驗證正確拒絕；補完整四筆fixture而非放寬正式契約。新的rank門檻與升級傷害／冷卻測試仍需跑過。

- shared per-rank required_hero_level預設1保留舊內容；lumen_lance明確[1,6,11,16]，rank1初始學會維持既有垂直流程。必須生成Rust／UE一致資料、驗單調與1..25、權威扣點前檢查，runtime compiled agreement防單邊Lua修改；不冒充整套LoL學習規則。
- 本輪合併context讀取超過exec總輸出上限再次截斷，已分段補讀；PowerShell rg Public/*字面glob錯誤，改直接搜尋目錄。首次patch使用過於泛用e.tombstone anchor把ability validator插在tower emitter，立即rg定位發現，在編譯前移至emit_ability_const；所有patch須核對實際落點。

## E118：技能升級不能只路由input，必須驗權威條件與披露rank（2026-10-04）

- 最終core333／base92／server154／runtime60／bridge52與Fyrox check通過。真實60Hz兩輪upgrade，最終1791091387雙送同request仍input3各一次、3950 wire／IPC snapshots rank／SP／XP／Gold核對與四PID另驗退出。full UE build／stage e08d2a…／11BP compile通過；之後Editor73092正常shutdown，MCP list curl7且inspect已退出，不能說崩潰，也不宣稱Ctrl操作或native automation已驗收。完整5.3未完成，詳見ability-upgrade60Hz進度檔。

- full build剛launch Editor時便呼叫MCP list，curl7端口尚未ready；必須先等om_restart wait-mcp完成再讀schema，啟動中不等於插件失效，不快速重複連線。

- 後续檢查誤猜Fyrox package omobaf，Cargo實際game package是omfx；誤用scripts/lib/process.lua，inventory已指出tools/lua/lib/process.lua。改讀實際位置／manifest再跑，不把exit1當成功。新增升級request有界1024筆去重，同ID重送與衝突不再配置input ID；此為同runtime保障，不宣稱跨runtime重連journal。

- IPC新測試首次失敗不是正式拒絕錯誤：assert_rejected helper固定要求request_id91，零ID案例應明確驗request_id0，已修正重跑。OmGenerated實際位於OmRuntime/Source，不在Plugins/OmGenerated，不能猜路徑。新增Ctrl+Q/W/E/R僅走共用native glue，沒有每英雄C++或Blueprint圖；仍需編譯與真實操作驗收。

- 既有UpgradeAbility handler有SP與Lua max_level，但無Moba phase／pause／roster gate；敵方不披露upgrade input，既有CommittedProgression只更新level／XP／SP，不更新四槽rank，會造成合法升級後hash分歧。新增權威條件與visible-only typed四槽rank事實，不能分享敵方input或用ComponentRepair遮錯。
- 保留四招初始rank1的vertical slice；本輪Lua沒有rank解鎖英雄等級門檻，所以不硬編LoL門檻、不以R前綴猜ultimate。正式網路／UE操作仍待實作驗收，不能以kernel替代。
- 新kernel首次E0425 GamePause不在game_processor import，改用crate::runtime::GamePause完整名稱；rg猜comp/ability.rs與script_registry.rs不存在，改rg --files定位。大段合併context讀取再次截斷，已縮小分段補讀，不能把截斷當完成。
- filtered升級fixture編譯E0283：collect後extend無法推斷集合型別，明確Vec<_>；又猜bridge/src/ipc_runtime.rs不存在，改列rg --files。core333已通過，但base未過前不得聲稱升級整體通過。

## E117：兵線XP不能沿用最近英雄Bounty／浮點單級結算（2026-10-04）

- 最終core332／base89（含15／60／120Hz完整filtered與零repair）／template65／server154／runtime59／bridge51／Fyrox check通過。真實KCP60Hz1791083346與3,883筆raw progression／IPC、戰績／Gold精確核對通過，實際觀測額外兵線XP，四owned PID另驗退出。build-only OmGameEditor、stage e2c3aa…與codegen --check通過；無本輪Unreal XP畫面／MCP驗收。詳見moba-lane-xp60Hz進度檔，整個5.3仍未完成。
- 最末60Hz完整filtered独立再跑2580tick／5158雙隊steps，Finished2566 winner0、雙death1／respawn1、15tick終局凍結與零repair hash全部通過；OpenSpec strict及本輪檔案whitespace检查通過，無待跑測試程序。

- 正式single_lane_creep沒有Bounty，不能假設legacy預設25XP已自動支付。新增Lua lane_creep_xp25／lane_xp_radius1200→compiled rules／full hash，權威tracked creep真實HP0死亡依存活敵隊roster範圍分享，整數floor餘數捨棄；移除tracked unit是一次性結算邊界。正式MobaMatch停用legacy Bounty，不引入兵線Gold；無此resource的舊MOBA／TD保留原行為。
- 新測試首次E0308／E0277：Faction.team_id實際i32，不是lane side u8；fixture helper改為i32，不修改正式schema。E0533：Finished為{winner,tick} struct variant，fixture提供完整值。這些失敗必須修後重跑，不能當passed。
- 大量混合讀檔再次被截斷；只用rg定位與小段讀取，不以截斷內容判定驗收。測試與實戰證據尚在進行，不預填成功。
- 分享測試第二次XP仍12而非37，初步懷疑同tick ordinal重用，但進下一tick仍失敗，排除此假設；不得據此改掉正式ledger。
- 上述ordinal假設不是本fixture根因：Damage只將Death送入next_outcomes，第二次process_outcomes才做正式死亡結算；首次測試直接record_moba_death意外替代Death queue，修為兩次正式process後才驗去重。完整base首次87/89：分享fixture與60Hz完整對局終局斷言失敗，其他兩個fps hash無錯；增加唯一終局診斷定位，不能把未結束或其他winner當完成。
- 60Hz診斷：18401 tick仍Playing、deaths[3,2]／respawns[3,2]，不是hash錯誤。加入XP後fixture Push的AttackMove會在兵線永久換目標；嘗試在可見objective進550距離後鎖AttackTarget並保留同目標active command，不改HP／XP／終局。新增bot回歸首次E0559：HeroCommand實際chase_origin而非網路input queued；改完整正式型別，需重跑。
- 首次鎖objective後60／120Hz完整filtered通過，但15Hz在4601tick仍Playing。Push在接近建築前仍AttackMove清兵；改Push使用正式MoveTo推進、Guard仍AttackMove守線，建築進距離才鎖AttackTarget。這是既有headless objective fixture，不宣稱五位置Bot完成。新Bot測試unwrap失敗因spawn無HeroCommandQueue，fixture須insert預設queue後才設定active，不假設每entity必有component。
- Push MoveTo候選仍造成60Hz逾時，沒有充分證據支持重寫AI；已撤回本輪所有production Bot改動及候選專用測試。生命周期回歸改明確正式input劇本：雙方死亡／重生後守方MoveTo離線、推方保留原Push，所有數值／XP／建築／winner仍正式結算。不得把此fixture當平衡Bot驗收，5.5仍未完成。

## E116：XP結算需權威去重與持久進度，不得以HUD有欄位當完成（2026-10-04）

- 最終core332／base86／template64／server154／runtime59／bridge51及Fyrox check通過；真實KCP60Hz1791081719、3,923筆wire／IPC progression與唯一kill／assist reward精確核對成功，四owned PID退出。最末Unreal build-only／stage d918c4…／codegen check及OpenSpec strict通過，沒有本輪Unreal XP實戰畫面／Editor測試，不引用上輪驗收。詳見moba-kill-assist-xp60Hz進度檔。

- Lua新增hero kill／assist XP，compiled rules與full canonical hash一致；runtime改值需重建所有peers，不接受dev hot reload。只接既有正傷害／唯一生命／合法歸屬結算，死亡助攻者存slot進度、重生恢復；沒有兵線XP或技能升級輸入驗收。
- MOBA獨立整數升級方法有多級迴圈、25級上限、飽和與零XP保護，舊TD add_experience保持不變。活體只加HP上限／攻擊成長delta、保留裝備、不補HP或復活；filtered只收既有CommittedProgression與EquipmentStats，不自行重付XP。
- 首次base測試编譯失敗E0616：跨crate試圖直接讀MobaHeroSlot.hero私有欄位。移除此直接斷言，不擴大正式可見性；在正式死亡／重生後讀ECS Hero經驗驗持久值。
- template首次不帶runtime-lua-content測試執行0項（該模組受feature gate），不能聲稱新回歸通過；改明確feature再跑並核對非零測試數。大型合併讀檔再次截斷，後續分段限量读取。
- XP成長改變戰鬥時序後，15Hz完整filtered在2399出現owner lumen_bolt冷卻差異（60／120Hz當輪通過）；診斷顯示當tick沒有新accepted input且NPC已被PreStep Forget，需檢查queued cast與實際HP。不得覆寫owner CD或ComponentRepair。暫時diagnostic首次patch又命中較早相同context，立即rg定位移到唯一lifecycle subject行；查找又用Windows path wildcard及猜不存在dispatch路徑，改rg --files。失敗不掩蓋，修正後需重新驗證。
- 根因為通用效果GameWorldDyn::is_alive只有ECS entity存活，HP0／pending-deletion仍可能queued cast成功起CD；改同時要求get_hp>0，新增真實adapter回歸。15／60／120Hz完整lifecycle與86個base tests重跑全部通過，診斷暫碼移除，owner CD與hash gate未放寬。
- 真實KCP XP verifier首次錯誤要求Warmup tick35也有CommittedProgression（正式inactive只保留bootstrap／persistent HUD，並不發active結算）；改只在Playing逐tick核對raw progression，所有live樣本仍依score與Lua reward核對數值、並要求killer／assistant非零reward實際觀測。不得因0 tests／只看report宣稱raw verifier成功。

## E115：非零死亡戰績畫面不得以開局零分或注入資料替代（2026-10-04）

- 正式60Hz雙UE run1791080545成功：兩隊各4技能、死亡／重生／Finished10028、Victory／Defeat與非零死亡計分板PNG已檢視，20,562筆IPC逐tick對authority及初始／死亡UI五欄完全一致。三方hash169／167 PASS rows至10200；五owned PID獨立確認退出。詳見unreal-scoreboard-death60Hz進度檔。
- 收尾full build／11BP gate／同Editor兩輪15/15／串行PIE全部通過；MCP保存後正常關閉owned Editor5380，六程序另驗退出。最後build-only／stage d7dd01…／codegen --check／OpenSpec strict驗證通過，無未完成測試程序。
- 本輪收尾查找又猜不存在的scripts/test_ue_moba.lua，rg明確exit1；改rg --files scripts定位ue_native_visual_smoke.lua／ue_pie_smoke.lua。不得把同一呼叫內其他成功輸出當成此查找成功；未修改遊戲來源。

- 本輪增加獨立death opt-in，依Playing、真實owner bAlive=false、supported Deaths>0及原生viewport layout就緒截圖；兩隊都要自己的死亡數>0，原始wire與IPC精確比對。NPC造成的死亡不冒充玩家kill／assist畫面。
- 初始與death使用不同PNG；首次截圖request後立即return，避免同一frame後續request覆蓋前者。這只保護測試request，不提供GPU fence或同tick像素原子契約。
- 沿用正式四技能／match lifecycle測試到死亡重生、基地勝負與結算，核對後120 ticks三方hash。不更改timeout掩蓋失敗、不加入角色C++或BP graph。

## E114：計分板畫面驗收不能以資料更新 log 代替實際 viewport（2026-10-04）

- 最終雙UE60Hz1791079800兩张1280×720PNG已檢視、6786筆IPC對authority逐tick一致與UI列精確對照；雙隊54 PASS rows至3240、六owned PID退出。Lua nested重現／module、9項觀測器、runtime59／bridge51+2、11BP recovery、兩輪15/15與串行PIE／最後build-only及stage一致通過。shader準備時FPS21／23，不宣稱穩定60FPS；初始0KDA不冒充擊殺更新或十人完整對局。詳見unreal-scoreboard-ui60Hz進度檔。

- 收尾full build的Blueprint gate compile-1791080124失敗，不是BP編譯錯誤：父子Lua同秒且random重複，host request檔名撞到仍使用中的檔案，path.write安全拒絕覆寫。改lfs.mkdir原子保留獨立owned request目錄、有界retry，不移除／覆寫其他程序檔案；強制父子相同time／random加入重現回歸。保留失敗report，不藉重跑掩蓋根因。
- 新增nested回歸第一次失敗是test既有root由source suffix推導，relative `tools/lua/tests/run.lua`沒有前置separator，child exe落到錯誤目錄；改用path.repo_root正式解析。修patch時首次錯用轉義context被apply_patch拒絕（未變更檔案），改唯一require行作context。不得以shell command最末exit0判定前面tests成功，需逐項明確核對。

- 新增非Shipping且明確flag的原生截圖 gate：Playing／tick至少120、IsInViewport、面板可見、實際cached geometry非零，再記錄UI資料並要求PNG存在。只有測試模式截圖，不加入角色graph或玩法入口。
- SetScoreboardState不能在值相同時直接return，否則首次資料先於layout，後續相同KDA永遠無法驗證；只節流文字更新，獨立重試layout就緒。
- 1v1 launcher觀測器只驗固定fixture的兩名玩家／兩隊列、配置身分、完整KDA與UI後120 ticks三方hash；不是十人或非零擊殺更新證據。後續需要原始IPC與畫面逐樣本核對。
- 本輪查找誤將截圖機制放在PlayerController搜尋（實際在共用Widget），以及大型OpenSpec合併輸出截斷；改rg整個Source與分段讀檔。不新增替代workflow。

## E113：公開計分板不得依可見英雄推算或放寬私人 HUD audience（2026-10-04）

- 最終真實三runtime1791079034通過一kill／assist／death，3955筆wire／IPC戰績板逐tick一致；core330／base84／server154／runtime59+3／bridge51+2／Fyrox與完整UE建置、11BP、兩輪15/15、串行PIE、最後build-only及codegen check通過。report cleanup與六個owned PID獨立核對退出，尚無多人計分板實戰像素驗收；詳見public-scoreboard60Hz進度檔。

- 本輪又猜錯OmGenerated plugin位置，改rg --files定位OmRuntime/Source/OmGenerated。新增clear保護首次patch只用常見entity loop context而命中前面的函式；立即以rg核對並移至PublishAbilityHudState之後，未建置錯誤版本。後續patch必須使用唯一鄰接context。
- 完整presentation snapshot缺少有效MOBA HUD時明確清除原生HUD；control-only frame不清除持久畫面。計分板放寬700px並允許換行，避免完整u32數值的十人欄位被固定420px截斷；尚須區分自動化widget驗收與實際多人畫面證據。

- 決定另設公開count與team／K／D／A namespace，AllPlayers僅包含持久玩家資料；原owner score／Gold／Recall仍維持隊伍隔離。不傳位置、entity reference、物品、助攻參與帳本；名單不依存活或視野推算。
- runtime atomic完整列驗證：2–10人、兩隊各最多5人、完整四metrics、count一致、唯一player、非零team及u32值、固定team/player排序；缺少／重複／錯誤資料顯示unsupported，不补零。bridge再核對owner team與私人KDA一致。
- ABI固定十列frame-owned array，升9並拒絕8。Unreal int64完整容納u32，唯讀原生panel只在有效快照显示，Stop／非MOBA清除；無角色C++或BP graph。OpenSpec大段合併輸出截斷已分段補讀。

## E112：網路擊殺／助攻驗收不可以 Move 或 NPC 死亡替代（2026-10-04）

- 最後TD1–100 integration三項通過（228.45秒，headless／observed完整hash與ledger相同）；Lua launcher最終語法與60Hz-only profile gate檢查通過。沒有留下待完成測試程序。
- 最終1791078192真實KCP60Hz成功：一kill／一assist／一death，三人各2post-kill checkpoints，3829筆wire／IPC score及收入＋300kill／100assist逐筆精確驗證。12個owned PID獨立核對退出；core330／base84／server154／runtime56+3／bridge51+2／Fyrox與build-only、stage72abf9…／codegen check通過。未做本輪Unreal多人畫面或Editor測試；詳見network-assist60Hz進度檔。
- run1791077851正式兩個AttackTarget均套用，但player1／victim重合，hero_tick把整個攻擊windup／impact包在距離平方>0.01的方向判斷內，合法近零距離攻擊永不發射。修正為零距離沿當前facing、正常啟動攻擊；非零方向維持atan2／轉向／範圍／敵我／HP gate。新增方向回歸並維持原重合網路fixture重驗，不靠拉開站位避開bug。保留失敗report及四程序cleanup。
- 查找時又猜不存在attack_tick.rs及run timestamp，改使用rg --files與列出實際evidence目錄；不得推算路徑再讀檔。
- 首次建置1791077835被rustc拒絕：MoveToIntent沒有queued欄位（只有AttackTargetIntent具有queued）。移除不屬於schema的欄位；未啟動任何遊戲程序，保留失敗report，不新增替代命令。
- 本輪加入獨立 opt-in 三玩家戰鬥 fixture，只用披露資料提交普通 MoveTo／AttackTarget，經既有 renderer intent queue、輸入驗證與 KCP；不注入傷害／Gold／計分，不稱為 Unreal 或 TCP renderer 操作。
- 驗收要求同隊兩人各提交可見敵方普攻、一擊殺加一助攻及 victim 一死亡，結算後至少240 ticks持續進行並核對三方hash。擊殺者不依執行緒時序寫死。
- 又使用 Windows rg path wildcard 而失敗：後續固定目錄加 --glob，禁止把 *.rs 作檔案路徑；查找錯誤不影響來源檔案。

## E111：K／D／A 必須讀傷害與死亡後的唯一權威樣本，不可取同 tick 的舊值（2026-10-04）

- 最終驗證：三runtime1791077095原始wire／IPC共3324筆KDA一致；runtime56+3／bridge51+2再次通過，UE full build／11BP gate／兩輪14/14與最終build-only、stage7a4d11…／codegen check通過。未把既有art PIE當作KDA實戰畫面；詳見owner-score60Hz進度檔。
- capture逐筆比較首次使用相同tick而失敗：SelectiveReplica apply後設定world.tick = frame.replica_tick + 1，render snapshot使用world.tick。因此明確以snapshot tick減1對照來源frame；不使用最近樣本／最大值／容錯範圍掩蓋差異。
- 三runtime capture首驗錯誤假定Move-only就沒有死亡：cleanup期間server仍進行，team2遭NPC合法擊殺，真實HUD為0/1/0。不能把合法死亡視為失敗或只檢查零值；verifier改逐tick原始TeamTickFrame player-scoped權威metric與IPC score精確比較，並核對所有score metrics隊伍隔離，不重跑／改寫原始交易或capture。

- 首次三英雄640tick計分回歸在player3死亡的step508失敗：PreStep／PostStep各發布同metric，renderer的first-match reader取到死亡前0。改active tick只PostStep結算後發布，warmup／pause／finished inactive tick保留單一PreStep；不改reader任意取最大值，不由Unreal補算死亡。
- K／D／A 以namespace＋完整player ID、team audience持久發布，不傳assist ledger／enemy Gold／隱藏entity。IPC optional score缺少／負值／overflow代表unsupported，絕不假造0/0/0；bridge核對score.player_id與HUD/configured owner。C ABI結構擴充必須升8並重建／stage，明確拒絕7。
- Unreal UPROPERTY以int64容納u32完整範圍，原生formatter拒絕負值及超量；死亡時仍顯示KDA。UI使用共用native欄位，無角色專屬程式／Blueprint graph。
- 再次以PowerShell路徑wildcard搜尋及猜disclosed_world.rs／disclosed_stepper.rs失敗：改rg目錄＋--glob並先rg --files。此為查找錯誤，不用shell fallback或額外workflow繞過固定Lua。

## E110：多人 roster 不可拿玩家索引當兵線側；測試必須尊重傷害與死亡的分階段結算（2026-10-04）

- 最後版三runtime1791076498亦通過：three-runtime明確metadata、雙隊9／第三玩家8 checkpoints至1080、safe1092；四PID86380／89648／8092／64972清理與獨立inspect通過。未把網路移動驗收當作網路助攻／多人UE驗收。

- 驗證：base84／core329＋TD1–100 integration／server154／runtime55+3／bridge50+2／Fyrox通過，build-only28589與ABI7／stage39f10b…一致。真實60Hz三runtime1791076242第三玩家獨立8checkpoints至1080；Recall2回歸1791076358兩隊各10 checkpoints至1200／基地完成，四＋三PID均核對退出。原始三runtime報告kind曾沿用two label，已修opt-in metadata，保留原證據不回寫。詳見roster-assist60Hz進度檔，完整5.3不勾選。

- 原本 Recall／respawn HUD 以 `(team, metric)` 作鍵，同隊多人會覆寫；filtered active Recall 更會清整隊命令。改為三個獨立 namespace 加完整 u32 player ID，filtered 核對 `(team, player)`，runtime 僅讀配置玩家；Recall 協商升 version 2，舊 version 1 明確拒絕。
- roster 採保留初始兩名玩家加 explicit additional_players，英雄 slot 保存 side；基地、shop、死亡、重生、助攻、收入與 fixture Bot 均分開 slot 與 side。每隊最多五人，所有玩家 ID／team／hero 必須在 spawn 前驗證，未知、重複、零值與超量不得留下半場景。
- 首次新測試失敗不是結算漏掉死亡：Damage 產生下一批 Death，單次 process_outcomes 尚未退休 victim。比照正式分階段流程再處理 queued outcomes，不放寬死亡斷言。
- 首次三英雄 Recall fixture 將敵人擺在普攻範圍內，合法正傷害取消回城，造成 active=0；改擺在視野內但普攻範圍外，保留全部傷害取消規則，不禁用戰鬥或偽造 active。修正後 640-tick／1280 雙隊步驟通過，零 ComponentRepair，完整 bootstrap hash 一致。
- 本輪再次遇到合併文檔輸出截斷與不存在的 runtime.rs 搜尋路徑；已分段補讀並使用實際 native.rs 匯出。後續先 rg --files，再讀檔，避免依記憶猜路徑。

## E109：雙人單路不等於多人助攻，匿名傷害也不能推導攻擊者（2026-10-04）

- 驗證：core329／base81／template63／server153及runtime54+3／bridge50／Fyrox check通過；KCP1791075055真實60Hz兩隊9 checkpoints至1080與3程序清理。build-only60465／ABI7／stage cfc40ab…成功，無助攻UI或多人對局驗收，5.3保持未完成。

- 先檢查實際MobaMatch：固定heroes[2]、config.players[2]且teams不同，目前不存在同隊第二名英雄。決定先交付authority assist ledger與damage hooks，純核心三／四人fixtures不冒充真實多人對局；後續必須一起擴充roster、private recall／HUD metrics與安全投影。
- 助攻採原創規則：Lua100 Gold、10 active seconds，實際正傷害才記錄最新時間；同隊英雄擊殺時每名有效參與者一次固定獎勵，排除killer／self／friendly／零傷害／過期，恰好窗口端點有效。只認目前roster英雄entity，不推導召喚物／退休entity归屬，匿名ScriptDirectDamage不發助攻。
- ScriptDirectDamage沒有source欄位，也未走既有kill hook；若匿名致死後heal再被hero致死，不能補搶獎勵。新增共同hook使匿名致死退休該生命參與記錄與lethal_pending，不擴ABI或虛構source。
- 搜尋再次猜native/mod.rs與omb/server/src不存在；應用rg --files與實際native.rs／omb crate來源，不延用路徑猜測。上下文過長的輸出已分段補讀，不當作完整內容。

## E108：UE B 綁定驗收必須連到原始 ACK，不得使用 runtime 內部注入（2026-10-04）

- 最終full build1195／Editor36528：11BP與兩輪13native＋PIE通過，SaveAll後正常關閉且inspect退出；raw capture test來源變動令bridge重新連結，最終stage057c643…，雙UE原始證據stage382dea…分開註明。再次scripts/ue*.lua失誤仍改用rg scripts --glob 'ue*.lua'，不交給PowerShell展開。

- 最終1791073998雙UE60Hz通過：Recall2／Move3取消／Recall4基地完成，原生HUD四PNG實看完整，兩隊各66 hash PASS至3960；protobuf4085／4063 snapshots均取消124ticks、完成479ticks。saved verifier重讀原始logs/config/PNG/capture與5PID無殘留通過，不把物理鍵盤或整項5.3／6.2標完成。

- interactive-ue-1791073690 gameplay成功但實際PNG顯示 Recall文字超出右側：原生match行改兩行、小字與wrap，必須再跑截圖。saved checker用文字模式讀PNG使CRLF被Windows轉換，signature誤判；改rb讀原始檔，不改PNG或放寬signature。

- 真實首跑 interactive-ue-1791073582 在 runtime 載入舊 release base_content.dll 時 abort：未知 recall_channel_seconds（跨 FFI panic）。雙玩家 launcher 原本只 build server/runtime，漏 scripts workspace；改為同 profile 先 build base_content，不使用 skip-build 掩蓋相依性。失敗報告 cleanup_verified=true，server58292/runtime98856 已退出。

- 本輪新增 opt-in 雙段回城 fixture：B 綁定 delegate →正式 Recall→移動取消→第二次 B→權威基地傳送；不改 HP、位置或時間。此入口不等於實體鍵盤／OS 事件。
- UE5.8 本地 InputComponent.h 證實 KeyDelegate.Execute 要傳 FKey；修正 Execute(EKeys::B)，不依賴猜測的無參數介面。
- 小地圖未就緒只能等待測試的初次移動，不能從 ProcessFrame return 而丟掉 HUD／entity 處理。移入前置條件。
- 路徑搜尋再次誤猜 OmGenerated 插件與 omoba_bridge.h；應先 rg --files，實際皆在 OmRuntime 插件，FFI header 是 ThirdParty/OmBridge/include/om_bridge.h。
- launcher 明確關閉 OMOBA_RECALL_SMOKE 的 runtime 內部注入，保留原始 input ID／owner／command kind／status、原生 HUD 文字及 PNG、完成後 120 ticks 三方 hash。未跑完雙 UE 不宣稱驗收成功。

## E107：回城 acceptance 不等於讀條成立；filtered 舊命令必須權威清除（2026-10-04）

- 最終增量：KCP1791072423雙隊正式Recall input2／8秒基地傳送，11／10 checkpoints至1320、cleanup通過；這是runtime內部renderer-intent注入，不是真實UE按鍵／socket。UE C ABI7、通用B鍵／HUD full63411／stage382dea…、11BP gate、同Editor兩輪13/13／PIE59543通過，SaveAll後48060正常WM_CLOSE退出。UE實際B鍵對局仍未封關；完整進度見moba-recall-network-60hz-progress。
- Lua helper搜尋已顯示實際tools/lua/lib/process.lua，卻又用了scripts/lib/process.lua而missing；已按實際路徑讀取並使用固定Lua／lua-host正常關閉Editor，不添加shell fallback。
- 最後補MD的合併patch錯放不存在的design「##」context，整批驗證失敗未落檔；移除猜測context重新套用兩個已讀文件，再讀design尾段才更新，不能把apply失敗當成功。

- 真實run1791072213失敗保留：team1完成tick993，team2 tick570 active後579取消；原smoke把正常取消回傳Ipc error導致重複rebase，是測試器錯誤。改記canceled／stage255，不污染正常frame apply；完成fixture正式移動往自己基地外側／離開兵線，而非穿越兵線後假設不會受傷。三程序cleanup_verified=true。
- Fyrox package猜成game導致package ID不符；讀Cargo.toml後改-p omfx並check --tests通過。codegen模板資料夾不存在的搜尋同樣先rg --files再定位，不硬猜。

- 發現：authority 清 HeroCommandQueue／MoveTarget，而 filtered 的 Recall branch 是 no-op；只測靜止英雄會漏掉傳送後繼續舊命令的問題。改以 owner-team PreStep 回城 active HUD fact 清除命令，不能對所有 accepted Recall 無條件清除，因暖場／同批移動可能拒絕回城。PostStep remaining 在傷害／死亡／終局結算後更新。
- 協商：獨立 Recall protocol1＋完整 Lua rules hash，legacy0/empty閉合；server 僅 authenticated selective player＋SingleLane 開放，不沿用 shop gate。IPC RecallIntent 使用 tag18：tag17 已被 AttackTargetIntent 使用，先查 schema 不沿用初步猜測。
- 搜尋失誤：PowerShell 傳給 rg 的路徑 wildcard（accepted_input*／team*）不是合法 resolved path；改用資料夾與 --glob。再次猜 hero_tick.rs 根目錄造成 missing path，應先 rg --files 找真正 tick 目錄。
- 驗證新增移動→正式 Recall acceptance→8秒回城的525tick雙隊filtered／owner-private metrics／零repair hash，以及 capability／owner／epoch／zero-ID gate。真實網路與UE的結果分開記錄，不以單元測試冒充 B 鍵成功。

## E106：回城必須在傷害結算後完成，且不能沿用未接妥的網路入口（2026-10-04）

- 此增量驗證：base79／template62／core323／server152＋1ignored／runtime52＋3 passed與3ignored、Fyrox check --tests通過；build85353／stage c82bbf…、codegen --check／OpenSpec strict通過。真實60Hz KCP1791071189双隊9 checkpoints及三程序清理通過，不是UE回城驗收。完整證據見moba-recall-60hz-progress。

- Fyrox相容check也發現E0004（game/src/native.rs的input分類漏新Recall）；保留舊前端，不把新共享enum只修Unreal端。分類顯式補齊後重驗，不做cleanup或commit。

- server編譯E0004：新增oneof後authority canonical acceptance的完整match漏Recall。補明確return None（未協商前拒絕），不以虛構action_kind繞過safe projection；必須同步檢查server與generated fallback，不能只跑core。

- 邊界補強：同batch任一移動／攻擊／施法／物品使用意圖優先取消Recall，不讓尚未dispatch的早先命令藏在讀條後執行。另有ScriptDirectDamage直接改HP而不經handle_damage，故補同一中斷hook與正向測試；零傷害不取消。停自動攻擊也以既有safe MovementPriority投影，filtered world不持有MobaMatch。

- template回歸發現10個舊TD-only fixtures沒有moba_economy：derive Default把新秒數設0，1..60驗證先攔掉舊內容。修正為整個section省略時的相容Default（不發經濟獎勵、8秒有效預設）；有宣告section仍要求完整型別欄位，正式Lua明列8秒。不能為新MOBA規則破壞舊TD載入。

- 決定：獨立Recall tag19，Lua整數1..60秒生成規則、compiled與hot-reload agreement；權威開始時清命令與MoveTarget、停自動攻擊，post-step在傷害／死亡後傳送，正傷害／移動／攻擊／施法／物品使用取消；pause不推進。secure入口暫拒絕Recall，不能把authority kernel當UE已可按B。
- 編譯錯誤：測試猜Vec2::from_i32而API是Vec2::new(Fixed64,Fixed64)，並錯用enum variant的struct update syntax（E0599／E0436）；修正為實際constructor與完整Outcome::Damage欄位。不再用不支援的便利API猜測。
- 搜尋錯誤：再次猜comp/pos.rs、tattack.rs、attack.rs造成missing path；實際先用rg目錄找型別，避免靠命名猜來源。
- Context輸出過長被截斷：改分段重新讀取OpenSpec tasks，不把被截斷內容視為已完整讀取。

## E105：擊殺不能由死亡通知／ACK 發錢；Cargo 與來源路徑不可猜測（2026-10-04）

- 最終生命去重版本封關：base77／core323再跑成功、NPC夾Heal分支通過、build89587／stage424449…一致。真實60Hz1791070433雙隊9 checkpoints、三程序退出且CIM無殘留；非UE擊殺画面或助攻驗收。

- 最後檢查新增同batch lethal→Heal→lethal邊界：HP正→零本身不足以代表新的生命，死亡還未刪除entity。增加hero slot lethal_pending（含replay digest），首次致死先標記、只有spawn新生命清除；即使首筆是NPC擊殺也不能後續hero搶已排定死亡的獎勵。新增治療夾傷害／重生後第二次擊殺測試。

- 驗證：base77／core323／template61 passed、60Hz完整filtered與125tick擊殺投影通過，build-only16479／stage、真實60Hz KCP1791070108雙隊9 checkpoints與清理通過；詳見2026-10-04-moba-hero-kill-60hz-progress.md。
- MD更新第一次apply_patch因猜測design context不符而整批失敗；確認新檔未建立／tasks未更新後，拆除無效context重套。讀到設計原文後才補design，不依片段猜行。

- 發現 runtime smoke 的非商店分支沿用 game.toml 的120Hz，只有商店分支改60。第一次啟動87440保留為120Hz相容性診斷，不當60Hz證據；修正為所有模式預設60且明確報告tick_rate_hz，原OMOBA_SHOP_SMOKE_FPS保留、新OMOBA_MOBA_SMOKE_FPS可覆寫。不改使用者game.toml。

- 新增投影測試第一次編譯 E0609：錯把事件放在 PostStep.events；實際 transport 是 Step.public_events，PostStep 只含 repair/hash 等結算欄位。已依既有商店投影測試修正，不能靠欄位名稱猜測 protocol。

- 設計檢查：同 tick 多筆 Damage 可在 Death 刪除實體前命中零 HP，若在 Death 或每次 lethal 判斷發錢會重付。決定只在權威正 HP → 零 HP 的正傷害首次轉換呼叫 credit；只接受當前雙英雄 entity 的敵方直接擊殺，排除自己／NPC／建築／非 Playing／pause，Gold 與 kill counter 飽和。
- 規則：Lua hero_kill_gold 生成 Rust 常數；compiled/runtime 與 dev reload compatibility 同步檢查。不得只更新 Lua 後使用舊 bridge／host 驗收。
- 操作錯誤：搜尋了不存在的 runtime/simulation.rs、implementation-error-register.md、根 Cargo.toml；實際是 native/simulation_driver.rs、此檔，根目錄沒有 Cargo workspace。後續先 rg --files 確认實際路徑。
- Cargo 錯誤：從 omb workspace 對外部 package 指定 features 被拒絕（cannot specify features for packages outside of workspace）。已改用 cargo test --manifest-path omoba-template-ids/Cargo.toml --features runtime-lua-content；不得把 dependency package 視作 workspace member。
- 邊界：召喚物／舊 entity 持續傷害歸屬、助攻、擊殺 UI 與多人規則尚未完成；此增量不勾選 5.3。

## E104：Blueprint 編譯失敗不能被 transport 成功掩蓋（2026-10-04）

- 封關：live fixture1791069017三種create／兩次3asset save／8次compile包含真實error_count1與原asset修復；獨立saved verifier重讀raw通過。default gate1791069249双英雄＋九UMG11個全部error_count0、無created；full18805／MCP gate、Editor10844最終兩輪13/13／PIE87395通過。17個正負向測試通過，3.2完成、19/30；沒有改角色graph或藉空Widget宣稱完整UI完成。

- Editor fixture49320返回running session後，未先取completion就提交九個UI compile；兩者最終都成功，但後續必須串行確認前一個mutable Editor workflow已exit0。此點不推成工具互斥保證；後續automation20057先確認exit0、再關Editor與重新full建置，不在PIE／重啟中交錯資產工作。

- 預設recipe gate首次拒絕native-only英雄的空blueprint_path：Lua空字串仍為truthy。改明確略過空路徑，實際有路徑者仍完整驗證；不是替native-only角色新增空Blueprint來繞過錯誤。

- 刻意負向案例：/Game/OmAutomation/BlueprintValidation_1791069000/BP_CompileBoundary 的 Actor self 呼叫 Character.Jump；graph build成功，但真正compile_blueprint為isError=true、ok=false、error_count1，診斷為self不是Character、Target必須接線。不能只看build ok或HTTP成功。
- 已只移除本輪owned fixture的invalid_jump節點並重編譯，error_count0、graph intent僅BeginPlay空body，沒有更改既有英雄graph。建立可重跑fixture與共用compile gate，必須保留asset／raw compiler／diagnostics／pending；transport／missing structured response／health issue／asset mismatch均拒絕。
- 決定沿用原生Slate，不為MOBA新增不必要UMG。必要舊Blueprint／UMG相容資產仍需compile驗證；WidgetBlueprint建立能力只在獨立fixture測試，不把空測試widget當完整遊戲UI。

## E103：真實 Unreal renderer 重連驗收（2026-10-04）

- 正式封關run1791068322（啟動前無Editor）exit0／cleanup_verified：renderer42712→43836正常關閉重啟，backend／對侧UE90208／84812／48392／75400不變；fresh Playing4859、actualrate60、Consumed870→4901、ownHUD／economy／movement恢復。新minimap renderer input1對應authority allocator input2／tick6511／target raw491520,294912，獨立保存verifierexit0。gate兩隊114／113 PASS至6840、各兩個post-input unique checkpoints，零FAIL。六PID與Editor92496查無；完整4.3／LAN／全cue／60FPS仍未宣稱完成，18/30不變。

- Editor92496 MCP QUIT dispatch 回覆成功，但 process.wait(30秒)失敗；錯誤地在 wait shell 還是 running session 時先啟動了 run1791068180。readback證實 Editor 仍在且無 modal，固定Lua針對該已保存Editor WM_CLOSE／wait10秒後正常退出。此run有短暫Editor重疊，僅保留作診斷；正式結果必須在確認wait exit0與Editor不存在後重跑，不能以啟動命令沒有輸出就當作完成。

- MCP get_tool_docs 傳 tool_name=execute_console_command 被工具視為 category 名並拒絕（尚未執行 console command）；改先 search_tools 取得真正名稱／schema，再按 discovery 呼叫。不能把 doc lookup 失敗當作 Editor QUIT 已完成。

- 新增 observation 負向測試首次失敗：Lua gsub 的第二個回傳值 count 被測試包裝函式當成 runtime_tail，導致 number:match 錯誤。測試呼叫以括號強制單一字串回傳；不是放寬 production 驗收或忽略失敗。後續此類字串變造 helper 明確截斷 multiple returns。

- 排查時猜測 scripts/lib/process.lua 與 OmRuntime/Private/OmWorldBridgeActor.cpp，實際分別為 tools/lua/lib/process.lua、OmGenerated/Private/OmWorldBridgeActor.cpp；PowerShell 呼叫 rg 時 literal *.cpp 路徑也失敗。後續先 rg --files 定位，使用目錄搭配 -g 過濾，不依據模組名猜檔案。
- 決定以 opt-in、有界 single_lane 60Hz 測試維持同一 server／雙 runtime／對側 Unreal，只正常關閉並重啟 team1 renderer。重啟使用獨立 logs／UserDir，原始證據不混入新 session；使用不同的公開小地圖 Point Move，避免同一舊目標冒充恢復輸入。只測 renderer 重連，不把 Rust runtime 重啟／兩台 LAN／全部 cue 當成已完成。
- 驗收必須讀實際 bridge diagnostics 的 tick rate，而不是 launcher 配置；HUD／economy／Consumed 必須在新 renderer 出現，正式輸入原 ID status0 與重連後三方 hash 才算通過。結果待實測，不勾選完整 4.3。

## E102：美術替換必須驗證真正匯入，並能還原（2026-10-04）

- 最後原版60Hz回歸run1791067086通過：原240秒／release，雙4/4、死亡重生10／9、原input8指定攻擊ACK、winner2、Finished12837／UI12838，actual Defeat／Victory與60Hz PNG已檢視；保存verifier獨立exit0，hash216／215 PASS rows／108 unique ticks各至12960、0 FAIL，UNVERIFIED25902／25900獨立不算PASS。五PID94372／3556／88660／78668／63348查無。來源b3ccc...已還原、Lua override已移除、最終stage cc76ef2d8328126a7ca255de84fa49daf72272af426ba9a0b40f8ceb8871d709；6.3封關，整體18/30，未宣稱全框架完成。

- art-swap-1791066725完成：256→160→256 Texture2D readback，actual PIE scale0.9且rendered，三張PNG已直接檢視；替換／還原各import2 jobs，所有repeat0 imports／0bindings／22packages SHA不變，Source C++與RustBP packages hash不變；來源binary已精確還原，保存verifier独立exit0。原版配置重建57914／UBT5.77秒、Editor85496兩輪13/13／PIE18379 actual scale1.2通過，content hash已恢復。6.3代表性流程通過，不代表任意Skeleton／全部動畫品質通過。
- 本次還原Save／QUIT後立即build57914，QUIT是async dispatch，restart一開始仍見49928並發taskkill，但隨後正常退出，force_terminated=false。防重犯修正：最後Save／QUIT後用固定Lua process.wait(PID,30000)確認85496真正退出，再啟動正式回歸；工具收到dispatch成功不等於程序已退出。

- 第二次build2110前未先MCP QUIT：restart對本輪只讀查詢的Editor62308正常taskkill／10秒等待失敗，後續精確project scope強制關閉才成功，未做資產改動但不應依賴force作正常收尾。後續variant／原版重建先確認PIE停、MCP Save／QUIT並查PID退出，避免強制關閉未保存Editor。build2110成功（UBT9.40秒），variant生成內容与bridge SHA已改，原版最後必須重建還原，不能留混版。

- 既有Editor測試與PIE腳本把Saika縮放固定1.2，會將合法Lua native_visual override誤判失敗。共用Editor fixture改比對生成CDO的NativeMeshTransform；PIE期望值讀同一Lua配置／meter→cm邊界，保留數值斷言，不刪驗收。角色runtime C++／Blueprint graph不修改。測試配置scale0.9／yaw-45，最後移除並重建原版。

- 排查時先猜templates/heroes與omfx/data/templates/heroes，兩路徑均不存在；生成配方已明確列scripts/lua_data/templates/heroes。後續以recipe source為準，不靠renderer舊路徑推測來源。
- 決定先驗6.3現有原生英雄貼圖替換：只操作已由ledger持有的RecipeV1資產與來源binary，保存原始來源備份／hash，替換後讀回Texture2D與PIE畫面，再還原來源／配方。不能只以imported count或save成功證明美術生效；不改角色C++／Blueprint graph，不刪原資產，未完成全部驗證前不勾選。

## E101：presentation diagnostics 不可用預設 120Hz 換算 60Hz 對局（2026-10-04）

- 最後capture逐筆9451／9429 snapshots owner HUD／公開live hero射程各通過，verify-staged-only保持03aa0c4d804ad706ecb2815366a817566776fdac2869df01c4d9ac9bea729098，scoped diff --check通過；既有warning未被關閉或冒充修正。

- 最後驗證run1791065966：release60Hz、原240秒、success=true／cleanup_verified=true，winner1、Finished9165、UI9166、雙技能4/4與死亡重生5／6；保存verifier獨立exit0。已直接檢視兩1280x720 PNG，頂端60Hz／tick時間152.8，中央權威Playing elapsed152.7，沒有?Hz或假120Hz。每隊156 PASS rows／78 unique ticks至9360、0 FAIL；UNVERIFIED18748／18739獨立，不算PASS。五PID28712／92908／87600／88404／84972查無。metadata bug完成封關，但不宣稱全框架或持續frame-time60FPS完成。

- 持久ready版本：runtime52+3／bridge50+2 passed，實際TCP晚連線、重連各先ready再snapshot，wrong-team無metadata；fullbuild14047 exit0／UBT2.25秒／stage03aa0c4d804ad706ecb2815366a817566776fdac2869df01c4d9ac9bea729098，Editor91056兩輪13/13、PIE95327含截圖exit0，正常Save／QUIT且PID查無。正式新版驗證中，僅這些測試不足以宣稱實際PNG rate已修正。

- 新版run1791065418完整60Hz對局成功，UI13504／Finished13503／winner2，雙队hash224 PASS rows至13680，PNG已檢視；但頂端仍?Hz／Time--，不能宣稱metadata修正完成。根因ready_envelope經publish_latest被snapshot watch覆蓋，UE晚連線收不到。改將RuntimeReady獨立持久保留，驗renderer bound identity後每次首次連線／重連先發ready，再正常snapshot；不靠一次critical FIFO解決重連、不拿snapshot cadence猜rate。補實際TCP晚連線／重連回歸後再build／Editor／PIE／正式60Hz。

- 新版build56090成功（UBT7.77秒、stage eef2d3aa3794cd9fbdb083732cbea9f27481f887547162ea361199051cb68dad），Editor86808兩輪13/13；PIE46123行為通過但take_pie_screenshot失敗，MCP明確回報editor minimized。保留失敗report；透過search_tools發現editor_ui_windows，list確認唯一om Editor ref1ab4c8d9確實minimized=true，再精確restore，不要求使用者介入／不刪截圖斷言；串行PIE重跑驗證。

- run1791064374已通過真實60Hz終局與三方hash，但PNG頂端沿用預設120Hz，tick9994顯示83.3秒；原生結算的權威elapsed166.5秒正確。這是diagnostics metadata缺口，不能將假rate或假lag=0當成同步／效能證據。
- 決定：RuntimeReadyPresentation新增additive tag6 tick_rate_hz，直接來自bootstrap；bridge先驗bound player/team，再接受60／90／120。缺欄位／不支援rate顯示未知0，不推測snapshot cadence。頂端改Observed、未知顯示?／--，刪除沒有量測依據的lag=0t。模擬速度／遊戲數值不改。
- 新增IPC roundtrip（presentation30Hz不覆蓋authority60Hz）、不同player/team拒絕、缺失／不支援rate回歸；重建、Editor／PIE與正式雙端驗證尚待完成，舊成功run不能代替新版。
- 統計raw JSONL初稿在Lua -e使用arg[1]得到nil；該呼叫filename位置為arg[0]，改明確已驗證run路徑。重跑實得每隊209 PASS rows／105 unique ticks、0 FAIL；UNVERIFIED25240／25228獨立列出，不能混算PASS。

## E100：原生終局畫面必須使用明確隊伍（2026-10-04）

- 第九run1791064374首次launcher success=true／cleanup=true，兩隊four slots、死亡重生、ACK、authority winner2，實際Defeat／Victory PNG已人工檢視；保存verifier初次錯用path.read文字mode，Windows把PNG signature CRLF轉LF而誤拒絕。真實圖片沒壞，改既有API path.read(file,true)二進位模式，不改rawPNG／不忽略signature。第一次複合診斷command末段exit0亦不能掩蓋前段verifier失敗，後續獨立單指令重跑確認exit0。

- 第八run1791063849再逾時，both四槽／ACK已成功；p1進基地射程，但p2 AttackMove中線持續小兵戰，只有p1有完整objective策略，NPC／英雄互耗拖延結局。第九版讓兩測試玩家共用可見塔／基地流程，方向以runtime明確team而非player ID，unknown team拒絕；補team2 tower700／base0回歸。fallback每次正常Move到公開endpoint、無MatchSmokeHero cache，防施法中斷後同hero不再移動。兩隊皆須原ID attack status0，仍雙死亡重生／真實基地勝者／240秒／60Hz／UI後hash，不固定哪隊獲勝，不退讓。

- 更新進度檔一次patch因只摘半行而找不到context，未套用；重新讀實際完整行後更新，不假設先前已改成功。第八版build69536 exit0／UBT7.76秒、Editor93504同session兩輪13/13／串行PIE exit0，stage8dc4c15c74c413f41dab8fe20902a9828b0e37633b02f35f2c01f280da0599f6；正式run待核對，不把前版建置當最新證據。

- 第七run1791063365仍240秒逾時，但首次有原ID attack ACK status0、敵塔摧毀、base真實扣血。重生後base被Hide，fallback停舊tower x1650，尚未再公開base就被前線敵人擊殺；不是再次IPC拒絕。改opt-in fallback Move到公開催線endpoint2400，已公開tower／base分支仍优先接管；不從Hide推導摧毀、不讀隱藏base、不改HP／勝者／期限。此fixture策略修正須重建兩輪13/13／PIE／正式60Hz重驗。

- 第七版fullbuild18958／Editor98396兩輪13/13／PIE1407通過，core323、runtime50、bridge49＋integration2通過，stage034cb8dcd1991d46f4f4f12c4f2a3064dfd80f8e4f27e92f12cc4e6383cc648c。fallback逐行與本機protoc輸出相同。六次failed report均success=false／cleanup=true。事後查舊PIDs時24920已被dotnet.exe重用，不是原UE process；保留不殺，不能只按舊PID清理或說它絕對不存在。最新驗收清理須核對該run自己的owned程序。

- 第六run1791062840 release重建仍240秒逾時，double kernel改動尚不足。最直接的正式端到端缺口是player_input_to_renderer_intent沒有AttackTarget，proto RendererInput也未定義，UE Submit成功只代表本地enqueue，driver最後INVALID_ACTION；之前objective log不是authority ACK。新增additive AttackTargetIntent tag17（render ID／queued）、runtime先owner／epoch／secure disclosed reference，沿原KCP secure target路由，不塞進TowerAction／不送裸authority ID；fallback同步。smoke新增原ID objective_result，正式result-UI gate與保存verifier強制至少一次p1 attack status0，不能再用enqueue作攻擊成功。

- 首次capture cargo test未限定--lib，連帶編譯integration runtime_driver_smoke出E0063：舊OmInputCommand三個initializer漏新item_catalog_id，先前--lib掩蓋編譯缺口。補非商店fixture的0欄位，不排除integration；重跑完整bridge tests／opt-in capture才可封關。

- 新路線15Hz hash缺失的精確症狀為tick1475 owner lance cooldown81920：加入fixture自己的可見target admission後2405ticks／4808steps通過；完整base73也已通過（52.91秒），不是放寬hash／死亡斷言。capture回歸初稿又把protobuf HeroHudPresentation誤當有alive（native payload才有），立即逐段查proto，以hp_raw>0判定，編譯前修正；不能只靠名稱猜資料層欄位。

- 正常explicit target修正後core323通過，但base suite初次71/73：兩個完整lifecycle只在終局「雙方必須死亡重生」斷言失敗，逐tick hash本來一致；不能刪斷言或稱全suite成功。fixture先以正常MoveTo經塔戰，再既有Push／Guard，不改HP／phase／winner。新路線又於15Hz tick1475暴露owner lance cooldown缺失hash mismatch：headless helper讀authority選到該隊不可見目標，並不是可接受的網路玩家輸入。fixture新增現有TeamVisibilityRuntime current gate，target command無視野不admit，不增加ComponentRepair／owner cooldown修正fact，不更動正式權威規則。重驗15Hz2405 ticks／4808steps、death／respawn5／3與15終局凍結通過；完整73項（含120Hz）仍須重跑後封關。

- 第五run1791062140仍240秒逾時，射程已550、雙隊4/4及死亡重生、塔HP確實下降。進一步查正常hero_tick：explicit AttackTarget只在nearest10候選找，較多小兵會排除較遠指定塔並fallback小兵；修正把live／current-position／射程內的明確目標加入有界候選，仍經原敵友／HP／射程檢查，不加hidden目標、不提高近鄰查詢n。新增超過10候選／重複／射程外／NaN／無position回歸，後續完整filtered lifecycle必須同hash。fixture selector另需公開max_hp以拒絕座標剛好重合的小兵。初稿誤記base1800，立即查MobaMatchConfig::default是1600，編譯前改正；固定fixture1200／1600不是一般Bot／資料驅動結構識別，改fixture數值時此驗收必須同步。

- 第四run1791061680仍240秒逾時：p1四槽4/4且死亡重生10；p2 slot2 status4被拒絕、僅3/4且死亡重生1。追到正式IPC使用driver.filtered_entity_to_render_data，未填attack_range而Default=0，先前只查projection/native extractor不足；不是塔未公開或策略不夠積極。補解碼已公開TAttack.range.v（缺失／壞資料0，不重建未公開Buff modifier）；新增真實render payload回歸。它是公開current range，不宣稱全modifier有效射程，權威仍判合法性。被拒絕的smoke技能原先永久保留ID，改保留拒絕log、等新可見目標再正常新ID提交，不將拒絕算PASS。
- 本輪又猜OmGame/Private WorldBridge與comp/attack.rs、ue_shop_acceptance.lua不存在，並把*.stdout.log當Windows rg literal產生123。已用rg --files／已定位plugin與tower.rs／-g查證；之後先定位不猜路徑。退出Editor59032後立即build遇到尚未自然退出，restart graceful taskkill/T未使用/F且最後force_terminated=false；後續QUIT後先確認PID消失才啟動。

- 決定：先補6.2終局UI而非重寫已存在的QWER binding。native結算面板只讀正式HUD的phase／winner／elapsed與已綁定runtime的明確team；新增共用payload明確LocalPlayerId／LocalTeamId，不用player ID推導team，player7/team2列入測試。勝／敗／平手、死亡也顯示；TD／非Finished／無效team／winner／時間／tick清空。未知team不能猜結果。
- 不新增再開局／離開／自算比分按鈕；沒有相應權威契約前不做假按鈕。面板HitTestInvisible純呈現，正式權威已凍結終局玩法；StopRuntime／EndPlay清空結果。快照斷線不擅自改寫已收到的終局。
- opt-in原生面板必須在viewport有實際非零cached geometry才記log，驗收核對明確team／winner／outcome／tick，與同玩家死亡重生後的權威Finished一致；終局UI出現後至少120 ticks的三方hash。不是OS／完整像素或真人操作驗收。
- OpenSpec合併讀取再次遭output budget截斷，已分段補讀design尾段與tasks；後續不得以截斷內容當完整讀取。沒有新Rust玩法／ABI變更，新增的是UE共用HUD payload欄位。
- 首次build55721 exit1／C2039：誤把OmRuntimeConfig.team_id當OmFrameHeader欄位；先前rg單行命中不足以確認struct歸屬。改逐段讀header，team ID從runtime成功Start時保存的Config取得，Stop清0，不重讀可變env、不推導player、不新增ABI。日後查欄位必須看所屬struct。此處沒有完成C++建置，必須修正後重跑。
- 修正後build26264 exit0；後續補正式終局畫面截圖再重建，不把前一版本建置當最終證據。依本機UE5.8 UnrealClient.h六參數RequestScreenshot API，opt-in在實際面板layout後捕捉含UI的game viewport，目標為每次run獨立result-ui/team-N.png；launcher要求檔案存在，完成後仍需看圖，不用visibility log冒充像素驗收。Lua result observer14 assertions及原有movement／ability／parity／lifecycle皆通過。
- build86320通過，但第一次12項Editor automation在NativeMatchResult使Editor21832 access violation，MCP curl56 reset只是崩潰後症狀。call stack UWidgetTree::ForEachWidget→UUserWidget::OnWidgetRebuilt→新fixture.TakeWidget；NewObject後漏Initialize，既有NativeShopInput早已使用Initialize→TakeWidget。補初始化並assert成功，不刪測試、不忽略crash、不只retry HTTP；崩潰檔保留Saved/Crashes/UECC-Windows-AC8B88034AAFE137294D1680BCEE3612_0001，必須重建後兩輪12/12再進PIE。
- 首個正式60Hz run1791059641於240秒逾時：四槽各4/4、死亡重生3／4，但無Finished／UI；success=false、cleanup=true。最後p1長停x1504，base x2400仍存活，不能當UI成功。檢查hero_command_tick與hero_tick：AttackMove遭持续creep skirmish，AttackTarget射程外時普攻windup／backswing可Hold且fallback到射程內creep，重發命令仍無法推進。只補opt-in可見基地策略：若公開催體且無仍存活的可見fixture塔，先正式Move到安全快照attack_range的75%距離，進90%射程後正式AttackTarget；不讀hidden身分／不改HP／不保證可見塔缺席等於不存在（合法性仍權威判定）、不改玩法數值／勝者／240秒門檻。重建與重跑後才可宣稱修正驗收。
- 策略修正第一次build94954 C4458：local Owner遮蔽AActor::Owner，UE工具链視為error；改明確ObjectiveHero，不關閉warning。failed run1791059641被保存verifier如預期拒絕，未產生success evidence；五PID已CIM查無。
- 第二次60Hz run1791060294亦240秒逾時，四槽與死亡重生成功但沒有objective log／Finished。檢查p1 x1448、base x2400未進視野，發現前半段AttackMove仍卡，僅補基地分支不足；舊末段無條件AttackTarget又可覆寫approach。修正opt-in p1以Move到既有fixture塔附近x1650施法，p2保留Guard AttackMove；移除舊基地迴圈，由已存在可見基地分支唯一接管。這不是正式五位置Bot／導航演算法，仍不改玩法／HP／勝者／240秒，不把再次失敗宣稱通過。
- 第三次run1791060700仍逾時，无objective log／Finished；只移動到塔附近再施法，欠缺持續tower AttackTarget，普攻仍會轉打小兵。停止零散分支修補，收斂成單一可見目標流程：tower優先base、射程外Move／射程內unit spell與AttackTarget，移除另個enemy/skill選擇分支。新增共用SelectSingleLaneSmokeObjective與Editor回歸（snapshot順序／死亡／hero／NaN／無效身分／無frame不造目標），由同一helper被真正smoke使用。必須13项兩輪與正式重跑後才封關，不將三次逾時混成成功。
- 第三run完整資料：p1死亡／重生14、p2 0，原Guard長期疊己方塔。第四版把opt-in p2 AttackMove目的地改對面塔區x1000，兩個測試玩家都經過戰鬥，不用退讓／修改HP強制死亡或指定勝者。這是結算fixture輸入策略，不是遊戲平衡修正，也不宣稱正式五位置Bot；四技能、各自新身分重生、任一一致權威勝者與UI後hash等門檻不變。

## E099：小地圖座標輸入與 HUD 消費邊界（2026-10-04）

- 決定：小地圖只接受原生右鍵座標Move／Shift排隊，不選取entity、不反查actor，不新增Rust玩法入口。Unproject使用同一aspect／Y inversion，留白、NaN、零size、無map拒絕；輸入由configured player／settings scale換算cm，runtime Connected才提交。權威仍處理合法性／地形，不把schematic bounds當導航資料。
- 避免混淆：Slate FReply::Handled防止HUD點擊穿透；FOmGameplayInputEvent.bConsumed必須保持false，否則既有bridge拒絕整個命令。左鍵、右鍵invalid與release均Handled，但只有有效right-down提交一次。舊HitTestInvisible已不適用，測試改驗Visible及無Entity target／left／release不提交，不略過斷言。
- opt-in非Shipping `om-minimap-move-smoke` 在真實cached geometry觸發既有right-down handler／UObject callback，exact一次callback及原input ID套用status0後再核對移動／post-input三方hash。未ready等待；已dispatch失敗停止該smoke，不能fallback直接WorldBridge API或重送新ID。這不是OS／真人滑鼠或完整hit-test grid驗收。
- Editor測試使用本機UE5.8 FPointerEvent／FModifierKeysState constructor；OmEditor新增直接InputCore相依，不靠transitive link。模型留白座標不可clamp到地圖邊緣；world scale非法時fail closed。
- 讀OpenSpec合併輸出遭截斷，改分段補讀design／tasks完整內容後才實作；不能用截斷內容當已完整讀取。Lua observer14項（含JSON round-trip）通過；建置／Editor／正式60Hz待驗證。
- 首次fullbuild17732 exit1：UBT在Main啟動SetCurrentDirectory時，使用者global Temp目錄遭其他程序占用、IOException（Build.bat code6）。尚未編譯本段C++，不能當code錯誤或建置成功。檢查UBT實際初始化來源，決定只為此次Lua建置子程序使用workspace專屬TEMP／TMP；不刪global Temp、不終止無關程序、不修改系統環境。
- 確認本機EpicGames.Core DirectoryReference.cs：大小寫等價的cwd切換會先Directory.SetCurrentDirectory(Path.GetTempPath())，不是遊戲C++編譯錯誤。隔離TEMP後UBT成功（本專案實際13.68秒；AppData舊Log的129.12秒來自別的建置，不可混用）；將防護收進既有build_ue_moba.lua，僅om_restart build子程序設定Saved/BuildTemp/ubt，Editor啟動與其他stage不改環境。查檔又誤用omfue/Source/OmRuntime（不存在），已用rg --files定位Plugins/OmRuntime，避免猜測plugin source位置。
- 本次SOmMinimap新編譯C4701警告：雖然Unproject成功才使用out參數，仍明確零初始化BackendPoint，不靠編譯器推斷跨函式初始化。驗收log改記實際UObject回呼的反轉座標而非smoke要求座標，並驗requested/actual誤差≤0.05 backend units。需重建與重跑後才當最終結果。
- 外層process.run包住會啟動Editor的full Lua workflow，繼承pipe使stdout晚到Editor退出才返回；本次已用MCP／automation確認Editor95744，正常Save／QUIT後92229 exit0。後續直接呼叫既有workflow，不再以外層capturing process.run包完整Editor啟動。
- 最終build60172 exit0（10.36秒）、同Editor91416兩輪11/11／串行PIE通過，新C4701已消失。正式60Hz run1791058752 success／cleanup=true、雙隊原ID1各exact一次callback，status0於2452／3050、抵達480,288／1920,-288；post-input hash54／54 PASS至3240、FAIL0，UNVERIFIED6543／6543不算PASS，五PIDCIM查無。詳細證據與未完成範圍見2026-10-04-unreal-minimap-input-60hz-progress.md；全框架與6.2未完成。
- 讀active-ue-session.json時檔案不存在：它是互動存活狀態，不是所有smoke期間必然存在的證據；改讀launcher回報的run目錄／owned PID，不把檔案缺席當cleanup成功或程序失敗。

## E098：小地圖安全資料邊界與查檔錯誤（2026-10-04）

- 操作錯誤：猜測 `omfue/bridge/include/om_bridge.h` 不存在；真正 header 位於 plugin ThirdParty。又把 `maps*`／`story*` 當 rg literal Windows 路徑，產生123錯誤。先 `rg --files` 定位，pattern 用 `-g` 而不是猜路徑。
- patch 操作兩次因多檔 hunk 不存在／順序反向而未套用；檢查實際內容後重新套用，不能假設失敗 patch 已改過前面的檔案。
- 初稿 EntityGeneration 欄位拼錯，實際是 EntityGen；在編譯前依 schema 修正。不得靠猜 ABI／反射欄位。
- 新測試插入 generic `#endif` 誤落在 OM_BRIDGE_BINARY_SUBDIR fallback guard，會造成已定義 macro 時不註冊測試；檢查位置後把 fallback guard 先閉合、測試移到 WITH_DEV_AUTOMATION_TESTS 主範圍，新增 limits include。不以測試未discover為理由跳過。
- 決定：小地圖只取 presentation_snapshot 的安全 entities 及公開 route points。以 routes 建 square schematic bounds、10% padding，不從單位位置自動伸縮；無有效 map 時 fail closed。控制 frame 保留、完整 empty/reset 清空、Stop 清空。ghost 非即時目標，第一版不呈現 frozen memory、不提供點擊移動／敵人命中。
- 最終code fullbuild96915 exit0／stage SHA e9085c7115541e12f7bf1a64cdd0d5ebf3a8d1a8f40f274c84f80a44b59e657f；Editor57864同session兩輪10/10、串行PIE35587 exit0並停止。core322／base_content73／runtime49+3／bridge47（opt-in capture未執行）通過。不得把本段宣稱完整小地圖／三路視野或完整UI完成。
- 第一版完整build4626 exit0、Editor28184同session兩輪10/10通過，但追查正式IPC發現沒有paths欄位，不可將synthetic通過當正式遊戲已顯示map。補公開lane-length HUD metric、additive protobuf欄位，由權威config產生，runtime只讀公開metric、bridge轉路線；未知／越界長度不得猜測。另一次猜ipc.rs不存在，實際處理位於driver.rs；使用rg --files後定位。
- 新bridge測試發現 owner converter 沒單獨過濾HUD player_id（socket入口已有身分檢查），wrong-owner fixture仍產生map。加入converter同樣的owner gate；player0僅existing test wrapper許可，不宣稱正式socket已越權。首次46pass／1fail／1ignored保留，修正後重跑。
- 第一個正式60Hz minimap run1791057171 success／cleanup=true，双native map／movement／HUD成功，三方56／57 PASS零FAIL至3480，但最後移動觀察3551晚於最後checkpoint。加上minimap的post-input hash gate（每隊至少6 PASS且tick不早於movement觀察）再跑，不將舊run宣稱移動後完整checkpoint通過。
- 補強run1791057377 exit0、release／60Hz，雙native map／HUD／movement／ACK、post-input parity60／59 PASS至3600成功，五PID82452／37072／73216／45844／85308皆CIM查無。report success／cleanup=true；完整證據見 minimap progress。PIE screenshot僅legacy fixture（含map marker），顯示3FPS，不當60FPS基線。
- protobuf fallback 原來沒tag8。比對本機vendored-protoc產物只有三行差異，狹義同步lane_length_raw，保留原有dirty生成內容；不能只改proto卻漏掉受版控fallback。
- fallback同步後release server8656 exit0，diff --check通過。最終hash原檔還有非checkpoint UNVERIFIED7257／7258（沒有expected），不計為PASS；有效週期性checkpoint60／59、零FAIL。沒有宣稱每tick或實際render60FPS全封關。

## E097：Slate按鈕交易驗收不得繞過回呼（2026-10-04）

- 最終驗證：build45033 exit0、同Editor86232兩輪9/9、串行PIE26384通過；正式release run1791055726 exit0／success／cleanup=true、雙隊exact三個Slate pointer回呼各對應request1／2／3、拒絕→買→賣與pending0。獨立verifier49688 exit0，三方hash178／179 PASS至10800、capture5413／5404逐snapshot通過，五PID查無。launcherJSON亦保存成功。詳細計數及界線見 `2026-10-04-unreal-shop-button-60hz-progress.md`。
- 決定：opt-in `om-shop-button-smoke` 使用已建立且enabled的原生SButton、真實cached geometry與捲動範圍，合成pointer按下／放開事件觸發既有OnClicked→SubmitShopAction。不呼叫受保護ExecuteOnClick、不強制enabled、不建測試專用Rust交易入口。使用PreciseTap符合scroll list觸控行為，正常滑鼠DownAndUp不改。
- 非Shipping且明確flag才可操作；layout未ready不送交易，已dispatch但失敗則停止該smoke，不換ID重送。驗收要求exact三次callback各一次、request與既有queued三階段一一對應，再核對真實receipt／金錢／六格／pending／三方hash。
- 範圍：此為合成Slate pointer handler／按鈕回呼整合，不是實體滑鼠或OS／完整hit-test grid驗收。同Editor automation與PIE必須串行。

## E096：原生商店實際按鈕路徑與舊 HUD 相依（2026-10-04）

- 最終：完整build42013 exit0、staged SHA一致；Editor80648同session兩輪9/9通過。串行PIE79747與補preflight／cleanup後61417均exit0且已停止。fixture失敗未略過斷言；沒有驗收真人滑鼠或hit-test grid，详見 `2026-10-04-unreal-native-shop-ui-progress.md`。
- 操作錯誤：新9項automation仍在執行時便啟動PIE smoke，共用Editor操作重疊。automation最終兩輪9/9通過，但PIE未觀察到player且cleanup回報No PIE session，不能當成PIE成功。後續同一Editor的automation／PIE必須串行，先確認automation complete与PIE狀態，保留此失敗；停止cleanup錯誤不能覆蓋原始startup失敗。
- 第一輪新Editor automation 8/9，NativeShopInput 26個離線button-enabled斷言失敗。獨立TakeWidget tree尚無Slate frame/prepass，IsEnabled讀cached預設true；依本機UE5.8 SWidget::UpdateAllAttributes實際API，在遍歷檢查前明確更新綁定屬性。提交gate與policy斷言均通過，不能把fixture尚未刷新直接當成實際離線可交易。修正後須重建／重跑同Editor兩輪，不略過停用斷言。
- 檢查發現：原生 SubmitShopAction 未指定 PlayerId，FOmGameplayInputEvent 預設1；進一步讀取 BuildInputCommandFromEvent 確認 converter 會以配置的local player覆寫，所以不是實際越權／錯隊提交缺陷。仍明確填入事件身分，新增player2與invalid player回歸以避免上游診斷誤導。先前雙UE smoke走WorldBridge API，不得將其冒稱Slate按鈕驗收，也不得僅憑event預設值就誤判已送錯玩家。
- CreateOmHud 在沒有legacy Blueprint class或已有root時提前返回，native command bar因此缺失。分別建立可選root與native bar，最後統一bind，保留重入冪等。
- HUD命中區以280固定像素判定，但Slate使用viewport DPI scale；共用bar高度及scaled rect，native與legacy命中區分開檢查，缺legacy不阻止native。
- 空catalog首次signature同為空，RefreshShopRows提前返回而不建六格售出列；加入首次建立旗標。SetAbilitySlot沒有範圍檢查亦補上。這些是檢查確認的程式缺陷，已經完整UE建置／Editor automation驗證，不宣稱物理點擊成功。

## E095：商店 Unreal 輸入接軌的編譯與工具錯誤（2026-10-04）

- 最終驗證：保存證據 verifier exit0，run1791053894 的獨立報告 success／cleanup_verified／reconstructed_from_original_evidence=true。兩隊真實原始三筆交易、出售pending0、三方hash170／162 PASS（最後10800）、capture5457／5443筆逐snapshot通過，五PID查無。原launcher報告保存exit1仍如實保留；不能以恢復報告掩蓋原失敗。完整證據及重現指令見 `2026-10-04-unreal-shop-input-60hz-progress.md`；debug效能與人手滑鼠點擊未驗收。

- 保存證據 verifier 第一次誤用 path.is_dir，實際工具 API 是 path.is_directory；只讀 assert 前失敗，沒有啟動／寫入交易。查 path.lua 後修正，不新增平台 fallback。

- 第二跑 release run1791053894：兩隊真實拒絕→買入→出售、pending=0與移動已被原生日誌觀察，owned五程序已清理；但最末 JSON 報告保存失敗（JSON object key must be string）。原因是新 observer queued 以稀疏 numeric stage0／2／4 作 keys。改字串 keys並新增 JSON encode/decode round-trip 回歸。使用原始保存的日誌／三方hash／protobuf capture 重新驗證及重建獨立報告，不重送交易、不忽略失敗、不竄改原始證據。

- 首次雙 UE run1791053032 失敗，不能宣稱端到端成功。team1 已完成拒絕／買／賣並移動；team2 在 tick11070 買入、11381 售出已由 runtime item_tick 結算，但 UE 最後只呈現11365，deadline 前未觀察出售。debug runtime2 inbound backlog 升至602；主因尚未單獨定位，不能說 pending 清理是所有效能問題的根因。五 owned PIDs 58208／88020／27460／81776／22380 已由 launcher 清理，CIM 查無。
- 決定：不提高210秒期限、不注入金錢。正式驗收改用既有 release server／runtime／script DLL profile，重建相同1.95.0 toolchain與最新Lua規則；Unreal 渲染限制60FPS，network仍60Hz／presentation30。商店完結 gate 額外要求 bridge pending=0，正向結果不能只見到 receipt 卻洩漏佇列。此為 production profile 驗收，debug效能仍是獨立未封關項。
- 又一次猜測 scripts/prepare_server.lua 不存在；建置入口實際是已讀取的 run_2player_ue.lua build_server／build_runtime。不新增假入口。

- 整合檢查發現舊 bridge 只接受 APPLIED_TO_PRESENTATION，會忽略 accepted SHOP_SETTLED 並留下 pending；商店成功必須以 SHOP_SETTLED 退休，普通 APPLIED／FORWARDED／query-only 不能當成交。SHOP_TRANSPORT_UNCERTAIN 即使 accepted=false 也保留原 request，供同 ID 查詢恢復。新增 busy-ring／原 tick／重複結果與不分配新 ID 回歸。第一次雙 UE run 使用修正前 DLL；修正後需再次建置與實跑，不能以單元測試冒稱修正版端到端驗收。

- Unreal C4458：UWidget 已有 Slot 成員，新增售出迴圈不可用 Slot 區域變數；改 InventoryIndex，不關閉 warning-as-error。full build 因此失敗，Editor 尚未重開，修正後重建。
- 再次猜測 bridge/build.rs 不存在；bridge header 是既有 build_bridge.bat 的 cbindgen 階段，不新增 fallback。只讀搜尋需遵守實際路徑清單。
- MCP save_all_dirty_assets 成功；exec_console_command QUIT_EDITOR 派發後 CIM 證實 Editor 28056 已退出，om.log 有完整 LogExit／closed，這輪沒有 taskkill force。

- 搜尋猜測 presentation.rs／OmInputTypes.h 不存在；改用已列出的 presentation_bridge.rs／OmGameplayInputTypes.h。再次違反 E018 的先列路徑原則，不能視為已消除操作問題。
- Rust E0609：compiled MobaItemConst 名稱欄位是 name，不是 display_name；依實際型別修正。
- Rust E0063：新增 protobuf capability 後測試 OwnerEconomyPresentation initializer 漏填；所有消費端一起搜尋／更新，欄位預設 false。
- apply_patch 同一批次兩次 Update 相同檔案被拒；合併 hunks 後重跑。紀錄檔標題亦需依實際內容定位，不猜標題。被拒批次没有寫入。
- 決定：C ABI 明確升至 6，舊 DLL／header 不可混用。買賣沿用共用輸入，只有外部 runtime IPC 路徑可提交，不走 legacy gameplay driver。驗證結果記於本輪進度檔。

每次遇到錯誤，先查本檔；追加「現象、原因、決定、驗證」，不得只反覆重跑同一失敗指令。歷史詳細過程見 `2026-10-03-unreal-moba-ipc-editor-progress.md`。

## E094：商店 transport 開放與真實交易驗收（2026-10-04）

- 決定：catalog agreement 不等於交易能力。新增 shop protocol 1／完整 rules hash，僅在已綁定玩家的 secure V2 single_lane session 啟用；舊／Story session 不開放。read-only query 最大 32 bytes、每 session 每牆鐘秒 8 次，客戶端時間無法擴充預算，暫停遊戲不阻擋恢復。
- runtime 最多 64 個待查 ID，每秒最多查 4 個，依上次查詢時間排序避免低 ID 餓死後面的交易；終端／expired／late 結果退休，不自動換 ID 再買。renderer shop request ID 同 ID／同 payload 只查原 wire ID；衝突拒絕，1024 筆歷史满額拒絕新交易而不逐出重放防護。此歷史不跨 runtime 程序重啟。
- 編譯錯誤 E0063：新 TeamGameStart 協商欄位漏了 generic projector initializer；補為無能力版本 0，再更新 legacy JoinRequest literals。E0599：測試猜測不存在的 render_snapshot API；查真實 extract_presentation_source 後將 smoke 搬入原本 snapshot extraction，不能額外 extraction 消耗 Hide／Forget directives。
- 操作錯誤：再次把 OmUi* literal glob 交給 rg 作路徑失敗；改以 rg --files 列實際來源，避免重複猜測。
- 真實 run 1791050713 失敗且三程序 cleanup_verified=true：210 秒內最後 safe tick 18311（120Hz nominal），尚未走到可合法累積 350 Gold 的 tick 21240；capture 只有原始不足金錢 receipt，沒有正向購買。capture checker 明確失敗，不提高 timeout、不注入 Gold、不假稱成功。
- 第二次 run 1791051137 啟動即失敗且唯一 server cleanup：誤將 headless Coarse15Hz 當成 network STEP_FPS 支援值；ServerSetting 明確只支援 120／90／60，正確 fail closed。不能為測試鬆綁正式 FPS 驗證；先查 server_config.rs，再改為已有正式 60Hz。診斷又把 state/core/*.rs literal glob 交給 rg 失敗，後續只依已確認實際來源。
- 第三次 run 1791051262 的 runtime 起動被 presentation-hz=10 拒絕，清理 server／runtime；config.rs 明確只允許 30／60／120，不將想用的低頻率當成已有能力。再次先讀真實 validator，改30Hz。
- 修正驗收方式：正式 60Hz network profile、presentation 30Hz，wait 仍是 210 秒、Gold 規則仍每 active 秒 2、warmup 2 秒；門檻按 negotiated fps 算，不把牆鐘等候當成模擬進度。120Hz 網路正向交易與效能仍需單獨驗收。
  - 最終 run1791051305 success／cleanup=true，60Hz雙隊各89三方hashPASS。原始capture逐snapshot驗證Gold／裝備／三筆immutable拒絕、買、賣receipts，兩隊5554／5518筆通過；每input額外5次完全相同重送、原buy terminal查詢成功，沒有重扣或多退。
  - 後續runtime write失敗視為不確定，不刪原ID；主／catch-up都回報原renderer request的recovered terminal結果，expired明確不確定。此補強有回歸／建置，沒有宣稱已做網路故障注入。使用者本輪指定先60Hz，120Hz驗收延後。

## E093：正式商店驗收不能依賴不存在的收入；Editor 開啟時不可 stage DLL（2026-10-04）

- 問題：正式 match Gold 0 且沒有收入，無法合法驗證正向購買；單元 fixture 注入 Gold 不代表正式遊戲可買。
- 決定：Lua moba_economy → build-time 整數常數 → Rust authority commit 被動收入，預設每 active 秒 2 Gold。累積 Fixed64 時間餘數，暖場跨界只計 playing 部分，暫停／Finished 不累積，死亡 slot 保留收入，重生不重領，i32 上限飽和。pending delta commit 後取走，重複 commit 不重付。
- 版本：全模板 catalog_data_hash 包含收入規則；identity content_hash 沒有角色 ID 改變時可保持不變，不能混為一談。Lua dev hot reload 不得單邊改收入，必須重建所有 peers。
- 操作錯誤：仍猜測 simulation_driver.rs 與 codegen/README.md 導致只讀搜尋 error 2；再次要求先 rg --files 確定真實路徑。之後改查 native/simulation_driver.rs 與 codegen/src/main.rs。
- 建置失敗：build-only 在本專案 Editor PID 78632 執行時拒絕 stage bridge（exit 4）；不重跑同一命令、不強制覆寫。確認唯一 Editor 的完整 project command line，MCP save_all_dirty_assets ok 後改 full 正常停止／重建／重開。
- 已驗證：base_content 72 tests 通過（雙 profile lifecycle 啟用正式收入）；template 28＋23＋8＋1 通過；core 321 通過；runtime 46＋3 通過。真實 run 1791049560 三方 10／8 checkpoints PASS、三程序 cleanup；原始 protobuf IPC capture 逐 snapshot核對 playing 時間與金錢，兩隊 571／224 筆，最後 18／16 Gold，六格空且無交易 receipt。
- 邊界：尚未以正式 KCP 成功買賣，shop secure gate 保持关闭。不能用此部分結果勾選完整 5.3／6.2。Unreal 重建結果見收入進度檔。
  - 重建結果：full 成功，stage SHA a6978590f720087819d2422190b86f28c0c08aa3cfe45c3310bd124e61bff63d，新 Editor 28056／MCP ready／native automation 兩輪各 8/8；restart 的正常 taskkill 逾時，工具自動 force 舊 PID 78632 及其子程序。資產已先保存，但不能描述成正常關閉成功；往後重啟前需優先查 MCP orderly quit 支援，並检查工具 force fallback。新增從真實收入 350 Gold 購買／出售的 headless 正式 PlayerInput 測試通過，不注入餘額、不冒稱 KCP。

## E092：交易結果重送不得再次交易或改寫原結果（2026-10-04）

- 現象：admission journal 只保存排入結果，DuplicateShop 不含實際 terminal receipt，不能用作成交成功回覆。
- 決定：以 authority filtered frame 的正式 receipt finalize journal，嚴格核對 player／ID／原命令 catalog 或 slot／effective tick；原結果不可改寫。新增獨立 read-only KCP query／replay，玩家由已加入的 V2 session 決定，不接收客戶端指定 player，不進 gameplay queue。
- 結果區分 unknown／pending／terminal／expired／admission-late-rejected；非 terminal 沒有 receipt，不假造成交。超過 1024 筆保留 watermark，expired 不重新執行，記錄不是永久帳本。
- runtime 只收自己、相同 input ID、schema 1 的 typed receipt，保留原 settled tick，更新 presentation history 而不套用 Gold／Inventory。history 改為同 input ID 即衝突檢查，避免同 ID 換 tick 另建結果。
- 查詢再次誤用 Windows literal glob omoba-content-model/src/moba*，失敗後改查實際 build.rs emitted catalog 欄位；不要把 glob 路徑交給 rg 作檔名。
- Fyrox check 出現 E0004：新增 LockstepInbound::ShopReceiptReplay 後既有 exhaustive match 漏更新。加入明確 unsupported 警告，不使用 wildcard／NoOp 把 receipt 送回 gameplay，不開放 legacy 商店恢復。
- hot path 決定：沒有 pending shop 就不 decode 額外 TeamTickFrame；history 保存 pending 計數，finalize／evict 精確減少，不每 tick 掃描 1024 筆結果。沒有新的效能基線前不宣稱 overhead 已量測。
- 真實 query smoke run 1791048551 未收到回覆而失敗，三程序已清理：Lua 檢查父程序旗標，但 spawn 明確 child env 漏傳 OMOBA_SHOP_QUERY_SMOKE，runtime 根本沒有送查詢。改為明確傳遞，不調整 timeout 或伺服器結果；必須以 runtime log／實際回覆核對，不只看啟動環境設定。
- 第二跑 1791048712 仍失敗，三程序清理成功：runtime 的兩個 inbound pump（主 select 與 catch-up try_recv）只新增第一處 ShopReceiptReplay 處理，catch-up 丟棄了新訊息。補齊第二處；往後每次新增 inbound variant 都查完整程式的所有 drain／match，不因 wildcard 能編譯就視為已整合。
- 最終 run 1791048834 success=true／四程序清理：雙隊及重連 runtime 都收到 unknown query，ID floor 仍為 1、新 ID=2；hash 14／12 PASS、重連後 4 個。core 321、server 151／1 ignored、runtime 46＋3、bridge 44／1 opt-in ignored、Fyrox check 通過。真正 terminal 查詢為 kernel／投影／journal 單元驗收，不冒充尚未開放的 KCP／UE 買賣；完整範圍見 `2026-10-04-shop-result-recovery-progress.md`。

## E091：runtime 重啟與輸入 ID 溢位不可回到 1（2026-10-04）

- 現象：InputBridge 每次程序啟動使用 default，且 wrapping_add 可在 u32::MAX 後重用 1，與既有商店 journal 衝突。
- 決定：InputBuffer 保留每玩家、match-lifetime 的 admission high-water mark，涵蓋已排入與晚到拒絕，不因 drain／eviction／玩家斷線消失；bootstrap 回傳 input_allocator_version=1 與 last_seen_input_id。runtime 握手拒絕缺少／未知版本，從 floor+1 分配；耗盡回報 INPUT_ID_EXHAUSTED，永不繞回。
- 範圍：不是交易 terminal receipt、帳本持久化或 exactly-once；未到達 server 的封包不算已見，並未補自動交易重試。既有 roster 拒絕同玩家重複活動 session，重連測試須等待 server session cleanup。
- 工具錯誤：再次猜了不存在的 omoba-core/src/runtime/input_bridge.rs、lockstep/server_state.rs；真實檔案是 omoba-client-runtime/src/input_bridge.rs、omb/src/lockstep/state.rs。往後先 rg --files 查到路徑再讀檔。
- apply_patch 兩次失敗：同檔 hunks 先修改 new() 後倒退至 impl，無法匹配；調整為原檔順序，不依賴失敗 patch 已寫入。
- 重連測試檢查發現 time.poll predicate 回傳 boolean 卻後續讀取 replica_tick；改成符合條件後回傳 safe object。程序載入中的 Lua 已經解析，修改檔案不會更新正在執行的測試；須保留結果並重新執行。
- 驗證：core 320、server 147 passed／1 ignored、runtime 46＋3、Fyrox cargo check --tests 通過；真實 KCP 重連與 bridge 驗證完成後另補進度紀錄。
- 真實首跑 moba-runtime-1791047611 失敗並完成三程序清理：hard stop runtime 後 15 秒未觀察 session cleanup；server 記錄 UDP 10054，但不等於 roster 已解除。測試使用既有 --shutdown-file → KCP SESSION_CLOSE，等待程序退出及 server cleanup，再重新加入；不提高 timeout、不直接修改 roster。server 日誌在 stdout，poll 同時讀 stdout／stderr。此驗收範圍是 graceful runtime 重啟，不能宣稱已驗證 crash／斷網復原。
- 第二跑 moba-runtime-1791047805 正常重連得到 floor=1，確實送出 ID=2，但 movement evidence 逾時；日誌確認目標與第一輪相同，英雄已站在目標，不會有位移。加入既有 scripted-move-interval-ticks=120，使第二筆往另一方向，仍走正式 RendererInput；不以送出成功冒充玩法已套用。四程序清理已核對，保留失敗報告。
- 最終 moba-runtime-1791047918 success=true：原 ID=1／floor=1／新 ID=2、正式位移、雙隊三方 hash 14／12 PASS、重連後 4 個檢查點；四程序清理核對。最終 server 148 passed／1 ignored，bridge 44 passed／1 opt-in ignored。完整限制與重現見 `2026-10-04-runtime-input-resume-progress.md`。

## E090：正式交易開放前必須先防重送與協商 catalog（2026-10-04）

- 再次猜錯 omb/src/lockstep.rs 與 Windows literal glob kcp/*，查詢失敗。以 rg --files 找到 lockstep/input_buffer.rs 與 kcp/client.rs；不要因熟悉概念就跳過真實路徑查詢。
- InputBuffer 原本每次 submit 都追加，相同 shop input ID 可跨 tick／drain 重送並再次交易。新增 match-lifetime、player-scoped 的 shop admission journal；同 ID／原始 target tick／命令只排一次，換命令／換 tick／零 ID 拒絕。已拒絕晚到的原始結果保留，不隨重送的 grace 改成成功。
- 每玩家最多 1024 筆、Buy item_id 最多 128 bytes；逐出最低 ID 後保留 expired-through watermark，不能在清理後再次執行舊交易。這是 transport 排入 at-most-once，不是永久帳本、exactly-once settlement 或 terminal receipt 重播。
- catalog 握手是版本／內容相容，不是開放交易或安全認證。Join 與 TeamGameStart 增加獨立 version/hash；只宣告 version 或只有 hash 均 fail closed。舊客戶端可不宣告，但新版 selective client 必須收到匹配回覆，不能默默降級；未知混版仍拒絕。
- protobuf fallback 由既有 vendored protoc 流程生成，同步舊 Fyrox web Join initializer；不手改 generated/game.rs、不改 C ABI 5。
- Fyrox cargo check --tests 發現 native.rs 的 InputActionKind 分類未包含先前新增 ItemBuy／ItemSell，編譯 E0004。新增對應診斷 enum／match，不以 wildcard 或 NoOp 假裝處理買賣，不開放其 secure shop intent；這是既有共用協議漏更新，後续每次 proto 變動要檢查全部消費端。
- secure shop gate 仍關閉，直到原始 terminal-result 重新送達／runtime 重連 ID 恢復完成。DuplicateShop 只表示不重排，絕不冒充購買成功；wire correlation 不進 gameplay 或 script ABI。
- 本輪驗證：core 320、server 146／1 ignored、runtime 45＋3、bridge 44／1 opt-in ignored、base_content 71、catalog 1、Fyrox check 通過；新版 full UE build／stage SHA／MCP、同 Editor 兩輪各 8/8、真實雙隊 run 1791046869 新握手／四槽／三方 hash 120／113 PASS，五測試程序退出核對。詳細證據與未完成契約見 `2026-10-04-shop-admission-catalog-progress.md`。

## E089：新增經濟 C ABI 時的路徑／結構相容錯誤（2026-10-04）

- 查詢猜測不存在的 bridge/build.rs 與 OmGenerated/Private/OmEditorAutomationTests.cpp；實際 header 由 build_bridge.bat 呼叫 cbindgen 生成，Editor fixtures 在 OmEditor/Private。先以 rg --files 查路徑，不依模組名称猜測。
- driver patch 用了錯誤的區域變數名稱 frame（實際 world），apply_patch 原子拒絕該檔；讀取實際呼叫後修正，不把拒絕當已套用。
- OmInventorySlot 增加 OmStringRef 後 derive Debug 編譯失敗；補上 OmStringRef 的 Debug（不改 ABI layout），bridge 43 passed、1 opt-in ignored。
- C ABI 升 5，owner economy／receipt／物品名稱字串均由 frame slot 擁有，control／reset 清空。不得以 ABI 4 DLL 配合新版 header。
- 物品名稱／價格取 compiled Lua catalog，不寫死 UE 角色／物品名稱；shop_available 僅代表權威場景條件，不能視為 transport capability。UI 明確唯讀，receipt 只呈現權威結算，不以 accepted input 假造成功。
- 重綁 WorldBridge 需同步清除新 EconomyHudStateChanged delegate，避免舊 actor 仍更新 HUD。
- 首次 UE full build 失敗 C4458：UOmCommandBarWidget 的區域 Slot 遮蔽 UWidget::Slot（warnings-as-errors）。改名 InventoryIndex，不降低編譯警告門檻；完成後重建，不載入部分更新的 ABI 5 插件。
- 新 smoke 納入 Om.Runtime.UiOverlaySurface，卻仍以 Om.Generated 過濾 discovery，造成 required test missing；改成 Om. 共同前綴，保留精確八項 required test／兩輪／零 skipped 門檻，不移除新測試。
- 讀取 cleanup PID 時 Get-Content 回傳多行 array，不能直接 cast int；首次輸出 pid=null／running=false 無效，不可拿來宣稱清理成功。改 Raw＋Trim＋純數字驗證，重新查實際 PID。此處只是唯讀診斷，不新增 PowerShell workflow fallback。
- 最終驗證：bridge 44 passed／1 opt-in ignored、最新版完整 UE build／SHA／MCP、同 Editor 兩輪各 8/8、PIE smoke、真實雙隊 owner economy＋四槽＋三方 hash 114／106 PASS 均通過。真實非空物品／receipt 畫面與 secure 交易仍未完成。
- full build 的既有 restart 對已驗證專案 PID 19428 正常關閉失敗，等待 10 秒後執行 scoped force termination 並確認退出；不擴大關閉其他 Unreal 專案。

## E088：交易結果不可等同輸入 ACK，也不可把 correlation 帶進玩法狀態（2026-10-04）

- 本輪再次猜錯 stable_fact／team_projector／filtered_replica／state_initializer 路徑。真實檔案由 `rg --files` 找到：runtime/stable_fact.rs、runtime/team_projector.rs、runtime/selective_replica.rs；初始化在 native/initialization。沿用 E085 規則，後續必須先查真實路徑，不能依模組概念推測檔案位置。
- PendingPlayerInputs 刻意沒有 input ID，不能為商店把 wire correlation 寫進 gameplay components 或 script ABI。交易 drain 只產生 tick-local ShopSettlement；投影層依 player 與原始命令順序精確核對 CanonicalAcceptedInput，才附上 correlation。缺結算不產生成功，命令不符直接失敗。
- 舊 APPLIED_TO_PRESENTATION 只代表輸入套用，不代表買／賣成功。商店由真實 ShopReceipt 決定 SHOP_SETTLED 或具體拒絕；死亡沒有 actor 時仍能回覆拒絕。Warmup／Pause／Finished 在跳過 dispatcher 前產生 MatchUnavailable，下一個 tick 清空舊結果。
- OwnerEconomy 是 owner-team 權威資料，與 live hero 身分分離；死亡期間使用已保存的 slot Gold／Inventory，Unreal/runtime 不自算、不假造滿背包或餘額。IPC 再限定 configured player；敵方不收此資料。
- 新完整背包斷言第一次編譯 E0369：Inventory 沒有 PartialEq。改比對既有 serde_json::Value 精確表示，不為測試變動 production component trait。
- server metadata_guard 初次失敗：新的 scripts fixture 直接查 input_id，即使只是測試也違反 scripts 不接觸 wire ID 的既有來源邊界。將 wire correlation 的精確比對保留於 core/shop_receipt 單元測試；scripts 的正式 World 測試只查 player／tick／結果。沒有放寬 guard 或把 ID 加進 gameplay。
- 驗證結果與剩餘限制記於 `2026-10-04-moba-shop-receipt-ipc-progress.md`。正式 secure shop 仍關閉；IPC 欄位已增加，但未更新 C ABI／UE 畫面，不能引用舊 UE 成功結果當本輪驗收。

## E087：經濟結算必須在正確階段套用，baseline 也必須按隊伍過濾

- 初版 patch 只以 `for event` 定位，誤插入 movement helper，編譯 E0425（找不到 world）；修改為獨立 typed settlement helper，在 component export 後呼叫，再同步 Specs。後續 patch 必須含函式名稱等唯一上下文。
- baseline、Reveal、server expected hash／rebase 都要使用同一 owner-team 過濾；只限制每 tick fact，仍會在出生或重連洩漏金錢與背包。
- 新 CommittedEconomy 使用 81-byte bounded typed payload，包含 Gold、六格 numeric catalog id／cooldown、十個 ItemEffects bits／dirty；float bits 僅為保留既有 component 精確表示，解碼檢查 finite／範圍／未知 id／空格 cooldown。先驗證完整記錄與既有私有 schema，才提交三個 component。
- CommittedEquipmentStats 只公開可見 actor 最終 speed／armor／attack；客戶端不取得敵方物品或餘額、不重放交易，所有既有技能 input／cooldown 邊界維持不變。
- Lua 物品新增明確 catalog_id，拒絕 0／重複；不依賴宣告順序當網路 ID。測試用 fixture 數值但使用真實 catalog 名稱，以便 typed inventory 可表示；不把測試價格當正式 Lua 價格。
- 新 filtered 商店回歸在 tick 3 失敗；增加 schema 級差異診斷，確認對手 CProperty.mhp 沒有裝甲 HP 加成，而 Gold／Inventory／ItemEffects 已吻合。敵方不持有物品，不會本地算出 max HP；CommittedEquipmentStats 擴為 40-byte 最終 HP／max HP／speed／armor／attack。保留完整 hash 門檻，不使用 ComponentRepair。
- 修正後新商店／fresh bootstrap／裝備重生完整 hash 回歸通過；core 314、base_content 70（含 15Hz／120Hz）、template-ids 27＋23＋8、server 140／1 ignored、runtime 41＋3、bridge 41／1 ignored。secure 買／賣仍拒絕；本輪沒有真實 UE／TCP shop 驗收，不把 headless snapshot 重建當作 renderer 重連。

## E085：正式商店輸入必須保持順序，不能提前開放 secure 客戶端

- 查詢曾猜錯 client-runtime/build.rs、comp/pending_item_use.rs、transport/secure_acceptance.rs、runtime.rs；實際為 core/build.rs、comp/lockstep_resources.rs、transport/kcp_transport.rs、runtime/mod.rs。後續先用目錄搜尋取得真實路徑。
- 第一個 patch 的結尾誤帶加號，驗證拒絕且沒有修改檔案；修正語法後才執行，不將失敗當成功。
- 買／賣與使用物品不能分成兩個 queue 後分批 drain，否則同 tick 的交易順序會改變。決定沿用 item phase，改為 ordered PendingItemAction enum。
- 新增 protobuf tag 17／18，不重用舊 tag、不傳 client 金額或 hero id；fallback 由 vendored protoc 正式生成。
- secure V2 目前沒有 owner 經濟結算／catalog 相容協議，因此保留網路入口拒絕新交易；正式權威 tick 的測試不是 Unreal 商店端到端驗收。

## E086：Lua 物品不能靜默切換成與 compiled catalog 不同的值

- 物品先前只在 server JSON，不是共用內容；MOBA 改用 templates/moba_items.lua，由既有 build.rs 生成 Fixed64 catalog，納入完整 template data hash。舊 TD JSON 不變。
- 僅開放確實實作的 passive atk／hp／ms／armor；deny_unknown_fields 拒絕未支援 mana／active 欄位，避免資料看似成功但沒有行為。配方價格必須沿依賴嚴格增加，拒絕 cycle／未知材料／材料總價超過成品價。
- 開發 hot reload 不會自動重建 world ItemRegistry，不能只更新內容 hash。MOBA 啟動核對 runtime Lua 與 compiled JSON；物品變更禁止 hot reload，必須重建 peers。
- 本輪另一 patch 對同一 runtime_content.rs 寫兩個 Update File，被工具原子拒絕；合併同檔 hunks 後通過。搜尋又誤用字面 Windows glob omoba-content*，已改已發現的明確目錄。
- catalog 改變後舊 Unreal staged binary 不再代表目前源碼；尚未重建／重新跑 UE，禁止挪用前輪成功紀錄。
- 驗證：template-ids 27 lib、core 311、base_content 69（含 15Hz／120Hz 完整 filtered lifecycle）、script-abi 13、omobab 140／1 ignored、runtime 41＋3、bridge 41／1 ignored 均通過；正 bonus 量化後為零也拒絕，startup compiled 不符／hot reload 變價／完整 hash 改變均有回歸。

## E084：商店的舊實作不是正式雙隊 deterministic input

- 舊 JSON ResourceManager 商店存在，但距離固定 (0,0)、配方扣完整 cost，不能直接當新 MOBA 共用交易。保留舊 TD 行為；新 kernel 定義完整價格減去全部材料價，六格交易先 clone 驗證、成功才提交，售價以整數 floor 50%，拒絕負價／溢位／非有限 bonus。
- 單路英雄原先沒有 ItemEffects，背包保留重生後也不會重套裝備數值；新增 dirty ItemEffects，使用既有 item_tick 重算，不另寫 C++ 或第二套 stats。
- 測試首次編譯 E0596：Vf32.val() 的簽名要求 &mut self（實作只是回傳 v，沒有 cache）；只讀斷言先 clone 再 val，不把查詢改成寫入 ECS。
- 正式 PlayerInput 尚無 Buy／Sell，owner Gold／inventory 結算投影及 Lua 共用物品 catalog 未完成；新 API 先限權威端，不能以 kernel 單元測試宣稱 Unreal 商店已可玩。
- 驗證：core 311、base_content 66、omobab 139／1 ignored、runtime 41＋3、bridge 41／1 ignored；兩隊基地、phase／pause／death、六格／合成／overflow、stats 撤銷、裝備金錢重生不加倍、含 ItemEffects 的完整逐 tick 重播均通過。

## E081：日誌格式與檔案路徑必須先核對真實宣告

- Match smoke 的 EntityGen 是 int32，不能用 %lld；改成 %d，Unreal build 通過。HUD signature 增加 hero generation 與倒數是否為正，才能留下死亡倒數與新身分重生，不逐 frame 印日誌。
- 本輪仍曾猜不存在的 OmGameplayInputComponent.h／OmPlayerHudWidget.cpp；實際 HUD 在 OmPlayerController.cpp。不得猜類名即路徑；先 rg --files，再搜尋既有 symbol。
- 更新本檔曾誤加日期前綴而 patch 失敗；改真實路徑後另一 patch 的 hunks 次序與檔案不符，又失敗且未改檔。先讀取目前檔案，按實際出現順序 patch，不重試同一錯誤。
- restart graceful close 的 child processes 未退出，工具等待後限定既有 project PID 53480 做 force close 並確認退出；未擴大到其他 Unreal project，重建後 Editor PID 89472。

## E082：完整 Unreal smoke 的進攻策略不能省略傷害技能

- run 1791035563 保留 success=false：兩隊四槽、正倒數死亡與新身分重生均發生，但 240 秒門檻內無 Finished；三方沒有 FAIL。只 heal／attack-move 的 Push 不等於既有 headless Push 的技能策略，不能以延長時間代替修正。
- 決定：p1 對當前安全 frame 中距離 <550 的可見 fixture 敵塔／基地或敵英雄施放 generated unit-target 槽 2／0，依 owner cooldown；p2 保持 Guard。所有操作只走正常提交 API，沒有 hidden lookup／HP 寫入／強制勝負／減弱敵人。
- 當前 frame 的 NPC 缺通用陣營／objective metadata，此 smoke 只適用既有長度 2400 的單路 fixture，不宣稱正式 Bot。正式 Bot 與三路 metadata 仍未完成。
- 終局 HUD 修正：死亡者不再顯示 Respawning，顯示 Match Finished；Draw 與 Winner Team 分開。沒有 local-team payload，暫不假造 Victory／Defeat。
- 修正驗證：run 1791036154 在相同 240 秒門檻完整通過，兩隊 winner 1，終局後三方 169／203 PASS、零 FAIL；不刪失敗 run。可見目標進攻＋正常規則完成，不延長門檻。

## E083：Lua assert 的多回傳不能直接傳入 optional base 路徑

- 保存完整對局證據的新工具首次失敗：`path.absolute(assert(arg[1], usage))` 在成功時仍回傳 path、usage 兩值，usage 被當作 base。沒有寫出假成功證據。
- 決定：先綁定 requested_run，再呼叫 path.absolute；確認 path.lua 沒有 basename API 後用受限路徑末段 pattern，不猜不存在的函式。
- 另一次診斷猜 comp/faction.rs 路徑不存在，已改對 comp 目錄搜尋真實宣告 unit.rs。後續路徑搜尋需先目錄，再實際檔案。
- 修正後保存工具成功從真實日誌／全部 checkpoints 重驗並產生精簡 evidence JSON；不只複製 success flag。
- 負向驗證：失敗 run 1791035563 被明確拒絕，不寫 success evidence；成功 run 清理後重驗 170／204 PASS、tick 18720。

## E080：完整 filtered 對局測試必須包含權威 accepted-input 邊界

- 新完整對局測試首次在 tick 73 發現 owner cooldown／attack phase hash 不同。原因是 headless SimulationDriver 只執行正式 PlayerInput，並不模擬 server 的 CanonicalAcceptedInput 網路投影；漏掉這層會讓 owner replica 沒收到施法。
- 初始決定：測試在實際 authority input 前記錄 actor／target 的 generation 身分，使用與 server 相同的 stripped protobuf payload 與 canonical acceptance 投影；不可用 ComponentRepair 掩蓋漏輸入。owner cooldown override 一直禁止；原先也禁止 owner attack settlement 的假設，後續被 tick 522 的 lifecycle 邊界推翻（原因與修正見下）。
- 這個 Bot 是既有 headless fixture policy，不宣稱已完成正式視野受限五位置 Bot；新驗收目的是逐 tick 核對死亡／重生到終局的兩隊 canonical world。
- 補 acceptance 後 tick 74 仍失敗，這次是正式缺口：authority hero XP=25，而 owner filtered XP=0。新增 CommittedProgression（16-byte level／XP／next／skill points），來源可見才投影，沒有殺手／目標／隱藏敵人資訊，不覆寫 cooldown。
- 查找曾使用 Windows literal outcome* 路徑，已改目錄 -g；test acceptance 起初猜錯 AttackTarget／AttackMove kind，讀取 server 真正映射後改成 3／11。首次 patch 猜成 slot loop 不匹配，核對实际 hero loop 後才套用；新增 action／補丁都必須先讀取正式來源。
- 新 fact 首次編譯 E0308：VisibilityPolicy 需要 String，不是 &str，改與既有 emitter 相同的 to_owned，不改 ABI 型別。
- tick 129 新差異：team 1 可見敵塔 replica 4，authority HP 360，replica 提前由本地 projectile 預測成 315。單路 NPC／建築沒有在 filtered world 持有 AI、基地保護或完整投射物規則，不能同時本地預測其傷害與吃權威結算。
- 決定：只在已有單路 delta metric 的 filtered world，把當前可見非英雄 kind=2 目標標為 authority combat；其 HP／死亡走既有 committed vitals／lifecycle，自己的正式 input、script/cooldown 與英雄 gameplay 仍重演。不安裝 MobaMatch、不傳 hidden target、不修改 TD／FOG 非單路規則，也不逐 tick repair 全世界。
- tick 522 暴露 owner 自動普攻 clock 邊界：NPC 的 commit lifecycle 可在 filtered pre-step 先退休，authority 本 tick 普攻仍看到 phase 內目標。修正設計：CommittedAttack 的 13-byte phase／clock／sequence 也提供 owner；不提供 target，owner cooldown 仍嚴格禁止 override。此前「owner attack 一律不可結算」假設不成立，更新測試與設計，不宣稱既有回歸證明不存在這個邊界。
- tick 1463 再發現可見敵方英雄的本地自動攻擊／projectile 重播也提前扣 owner HP（expected raw 340256、actual 291718）。只有 NPC 停止預測不夠：對手私有 target／input 本來就不鏡像，不能重建完整攻擊 flight。
- 最終邊界改為單路所有已披露 target 的 Damage 結算由 authority vitals／external effect 提供；filtered 仍執行 owner inputs／scripts／cooldowns、視野內姿態，不重播對手私有玩法。這不是忽略 hash 或停掉 owner技能：四槽 own cooldown 仍不得 override，完整 canonical hash／實際 HP 仍逐 tick 要求吻合。非單路不安裝 authority-combat targets。

## E079：重負載並行驗收讓到達位置的短暫 snapshot 未被呈現

- run 1791033230 失敗且保留：team 1 四槽 applied／cooldown 通過，team 2 未施放；沒有 three-way FAIL。runtime team 2 的正式移動證據到達 x=1400（tick 7315），但 UE owner frame 維持 x=2272，之後收到死亡／重生，未觸發 arrival latch。
- 觀測：runtime inbound_depth 曾達 438、每秒只前進約 40 ticks，而 authority 約 100 ticks；同時執行 Editor PIE 與 Fyrox cargo check。資源競爭是待隔離驗證的推論，不宣稱根因已確認，也不把此次逾時當成功。
- 決定：所有其他編譯與 PIE 已結束，先在獨立負載條件重新驗收；保留原來 gate、正式技能與視野規則，不透過 teleport／假 HUD／修補 hash 通過。
- 本輪查找又對 Windows 路徑使用 literal *.rs 而失敗，後續已使用目錄配合 -g '*.rs'；不能將空結果解讀為不存在。
- 獨立 run 1791033455 同樣失敗，排除「僅 PIE／cargo 並行」的推論。main.rs 每個 presentation interval 即使 catch-up 已設定 publish_latest_snapshot=false，仍會 snapshot_envelope／fog derive 再直接丟棄；移動時 exact-center cache 不命中，計算浪費讓 backlog 自我放大。
- 修正：在建置霧區與記錄 presentation envelope 之前決定 Latest／Critical／不準備。lifecycle 保持先處理，input-bearing snapshot 即使 catch-up 仍走 Critical FIFO；每 applied tick damage ledger 不改。新增四種 route 組合測試，沒有降低 authority tick 或關閉 hash gate。
- 檢查另外猜了不存在的 ue_stage_guard.lua／demo_projection.rs，已改實際 main.rs／presentation_bridge.rs 與 build_ue_moba.lua 來源，不將 failed rg 當驗證。
- 全 repo diff --check 發現 omb/Cargo.toml 既有 CRLF 行尾在其 Git 設定被視為 trailing whitespace；本輪沒有修改該 manifest，不擅自重寫。檢查本輪修改的實際來源另行執行。
- `build_ue_moba.lua --verify` 是不存在的參數；讀取實際 parser 後用 `--verify-staged-only` 通過，staged SHA-256 為 0a3de7bb2f17e75a891321d1e01d72ba050a9d98b2309a04b686136b5a3f41bf。不得再次猜 CLI 選項。
- 修正後 run 1791033665 exit 0／success=true：兩隊各 4 applied／4 positive cooldown、最後施法 tick 6509 後三方 parity gate 到 tick 6600（65／87 PASS），完整結束 checkpoints 67／89 PASS、零 FAIL。第一次真實驗收已確認效能修正能避免重現 stale arrival，仍需重跑確認，不宣稱整體效能基線已完成。
- 第二次 run 1791033806 exit 0／success=true：兩隊各四槽通過，三方 61／67 PASS、零 FAIL 到 tick 6360；五程序精確 PID 清理 verified。runtime lib 41／binary 3 與 Lua gate tests 再通過。兩次共同確認此修正，不是增加 timeout；仍不等於正式 6.5 效能門檻封關。

## E078：敵方英雄交戰普攻 phase／clock 無權威投影

- run 1791032353 不通過：team 1 四槽全成功、team 2 三槽成功後串流終止；最早 team 2 tick 6360 的 hash FAIL，不刪除失敗紀錄。
- 逐 component 比較 authority expected 與真正 runtime dump，唯一差異為可見 hero replica 25 的 TAttack schema 1179600646：authority asd_count=948／Backswing，replica -306／Windup，attack_seq=5 相同。不能用射程、延長逾時或 component repair 隱藏此差異。
- 決定：opt-in MobaMatch commit hook 發布 typed 13-byte attack clock／sequence／phase，只向可見敵方投影，不帶敵人 target／input／完整 JSON。own-team 不收 override，仍須自己的正式 deterministic gameplay 正確；新事件 fail closed 驗證长度與 phase。
- 回歸與新 staged artifacts／真實雙 UE gate 完成後追加結果，不預填成功。
- 首次新增 handler 誤匹配同名 for-loop，放到 movement 函式而非 apply_disclosed_events，導致 E0425 world 不存在。依實際函式邊界移至 post-gameplay disclosed apply；core 303 通過，新增 idempotent／數值保存／malformed phase 與 hidden／owner-clock 不覆寫檢查。
- 擴充四槽回歸第一次把「owner team 不收自己的 attack override」誤寫成「該隊完全不能收到 attack event」，連可見敵人都拒絕；修正測試逐 subject 檢查披露 team，不取消 owner guard。技能後延伸 120 tick 真正比較兩隊全部 canonical hash，不只比較四個瞬間。
- 最終 full build 54205 exit 0；修正 catch-up 浪費後兩次真實 UE 1791033665／1791033806 各四槽通過、三方 pre/post 零 FAIL。完整證據在 four-ability progress；未加入 proactive repair 或關閉 fail-closed。

## E077：安全目標的 generation-packed 身分被誤當 u32 索引

- 真實 run 1791031862：兩隊 self skills applied／cooldown 通過，但指定可見敵方的 slot 0／2 status=4，沒有 authoritative cast。不是射程調大即可解決，失敗 run 保留。
- 根因：TeamViewProjector 以 `(generation << 32) | ECS index` 建立 server validation canonical ID；`rewrite_secure_target` 對整個 u64 用 u32::try_from，所有正常 generation > 0 都失敗。
- 決定：既有 validate_and_resolve 完成 team／view epoch／disclosure epoch／visibility／owner 檢查後，僅在權威內部轉換取低 32-bit ECS index；canonical identity 不回傳 renderer。不取消安全驗證，不把 client opaque ID 當 ECS index。
- 回歸加入 generation 7 的 cast／attack index 42 轉換與不合法 action 拒絕；真正雙 UE 四槽與三方驗證後補記結果。
- 最終 server 139 passed（1 ignored），兩次真實雙 UE 1791033665／1791033806 的兩隊 unit-target 槽 0／2 均 status=0，並有 positive cooldown／post-cast 三方 hash，確認不是只有內部函式測試。

## E076：施法 API 不等於原生四槽操作與實際結算

- 檢查：原生 controller 尚無 Q/W/E/R 綁定；既有 SubmitCast API 與本機 queued 不能證明後端套用及冷卻。決定補通用 owner 槽位輸入、opt-in smoke 與 input-ID 關聯的 applied／cooldown gate，不寫角色專屬 C++。
- 模組邊界：OmGenerated 已依賴 OmRuntime，不能反向 include 造成循環；原生 controller 使用 reflected 通用函式。新增 InputCore 明確直接相依，不倚賴間接連結。
- 工具錯誤：本輪又猜測不存在的 OmGameplayInputComponent.cpp、OmRuntimeTypes.h 與少 `_test` 的測試檔名；後續先使用 rg --files／目錄內搜尋。不得將找不到檔案當成功檢查，也不要把重複猜路徑合理化。
- 編譯／實際端到端驗證結果完成後記到本輪 four-ability progress；不得提前勾選整項 4.1／6.2。
- 首次 build session 59396 因 C4458 失敗：smoke 的區域 `Owner` 遮蔽 AActor 成員，改為 `OwnerHeroFrame`；不降低 warnings-as-errors。發現 controller 綁定依賴 HudRootWidget，改為只依賴 World，避免沒有 HUD Blueprint 時技能也不可用。
- 真實 run 1791031584：smoke 在 approach move 的同個／接近 tick 施放自身治療，正式施法中斷移動，英雄無法進入射程。改成安全 owner frame 確認距交戰位置 30 內才施放，不修改 gameplay 射程或使用 teleport。保留原始 run，不把兩槽 self-heal 成功當四槽完成。
- 活躍寫入中的大型 checkpoint 讀取曾造成 PowerShell Substring 型別錯誤，未取得計數不當證據；完成後再讀取並明確限定 string／完整換行 records。
- 後續 1791032111：修正安全身分後 team 1 四槽全通過、team 2 指定目標 slot 0 通過，證實原拒絕原因；但 arrival guard 被錯誤用在每次技能，正常 cast／追擊移動離開測試起點後停止其餘槽位。改為一次性 arrival latch，後續仍以當前 disclosed target 與正式距離驗證，不鎖住英雄位置、不放寬 gate。
- 一次 `cargo test --lib anti_probing` filter 未匹配任何測試，不算安全驗證，隨後 core 完整 302 tests 通過。再次 Windows wildcard literal 與估計 run ID 搜尋失敗已改用 -g 與實際目錄清單。
- 最新 Editor 直接 run_automation_tests 回報尚未 discovery，不當成測試失敗或通過；先經既有 NativeVisual 流程完成 discovery 並等待 ready，再執行 GameplayInputSurface。每次 Editor restart 都必須重新確認 discovery。

## E075：驗收用自動移動沒有隔離出正式遊戲

- 檢查：WorldBridge 的 28 秒 approach move 原本無 smoke flag，會在一般互動遊戲自行發命令，干擾玩家。
- 決定：只有明確 `-om-presentation-smoke` 才可發驗收移動；固定 Lua 入口只在正值 OMOBA_UE_SMOKE_SECONDS 時加 flag，正常入口沒有這個 flag。不要用測試機器人的動作冒充玩家操作。
- 驗證：最終 rebuild session 8153 exit 0，run 1791030915 帶 flag 的真實雙 UE movement／owner HUD gate 通過；不帶 flag Editor NativeVisual 兩輪各 7/7、PIE success 並停止。程式明確以 flag guard 命令發送，Lua 正常互動入口不加 flag；完整普通互動長時間無自動命令的觀測仍非本次 fixture PIE 的證據。詳見 HUD progress；不改 root bat 或加入 fallback shell。

## E074：正式技能未向可見對手同步已結算狀態

- 初始 fixture 忘記 server 的 CanonicalAcceptedInput 邊界，因此 owner replica 沒有施法；補上 stripped protobuf input、projector 重寫可見 target 後，owner 通過，opponent 仍缺 cooldown 與傷害後 HP。不能把這兩種原因混為同一 bug。
- 修正：僅 opt-in MobaMatch 的 serial outcome reduction 產生 typed `CommittedVitals`／`CommittedCooldown` facts；只帶 fixed-point HP／maxHP 或 slot／remaining，不傳 JSON world、敵方輸入或 canonical identity payload。按真正 visible source 投影，隱藏 actor 不發布；owner cooldown 不發 committed override，仍須由自身 accepted input 正確重演。
- 同 tick 多次 outcome reduction 使用獨立的 serial ordinal，post gameplay 套用絕對結算值，不把正常同步塞進 ComponentRepair；非法長度／負 maxHP／槽位／冷卻 fail closed。
- 測試：四槽逐次比較兩隊 canonical hash、实际 HP 920→740／400→470→550，enemy input 不發布、owner cooldown 不由 committed override 掩蓋；新增 hidden actor／無 canonical payload／malformed 不部分修改／absolute replay 測試。
- 開發錯誤：filtered_specs 未匯入 Fixed64，改用明確 omoba_sim 路徑；隱私 fixture 首次誤把同隊 actor 當對手，修成 explicit team 1 projector／team 2 baseline。再次猜測兩個不存在的來源路徑搜尋失敗，後續均改目錄搜尋；必須持續遵守先列來源規則。
- 這段是 production filtered gameplay 測試，不是四技能由 Unreal 按鍵送出與完整終局驗收；artifact／UE 需重新建置後再記最終證據。

## E073：HUD 首次真實雙 UE 驗收逾時，不能用編譯與單元測試冒充成功

- 失敗 run：interactive-ue-1791028402，success=false。server/runtime 維持運作至 team 1 tick 9309；UE 已 LoadMap／StartRuntime，但沒有第一個呈現／consumed snapshot 日誌，smoke 60 秒 gate 逾時。
- 已排除：讀取原始 presentation.capture，bridge 真正 protobuf decoder／HUD 身分驗證／frame conversion 重播 4654 snapshots，4654 HUD 皆合法；不是後端停止或 HUD schema 全部被拒絕。
- 決定：保留失敗 capture，新增可重跑的 opt-in capture test；加入一次性的第一 Tick／首個 HUD frame 階段診斷，定位阻塞位置，不刪 HUD gate／hash gate 或無根據加長 timeout。
- capture test 首次缺 prost 直接相依，已加 dev-dependency；曾假設 Cargo 有 dev-dependencies section 使 patch 匹配失敗，讀取實際檔案後補 section。遵守依實際 manifest 操作。
- 另有 UE 5.8 r.Mobile.VirtualTextures deprecated handled ensure 與可選 profiler DLL 缺失；它們在啟動後繼續，不先把既有引擎警告當成 HUD 阻塞原因。
- 後續兩次 1791028917／1791029014 真實雙 UE gate 通過，未重現首輪零 frame；原因尚未確認，不宣稱根治。診斷 rg 搜尋忽略的 target/logs 目錄需要 `-uuu`，否則空結果不代表沒有日誌。

## E072：HUD 契約與 API 必須從實際來源確認

- 問題：本輪曾誤稱 snapshot 有 hud_view；實際 protobuf 沒有。猜測 AbilityId::from_str_id 導致 E0599；新增 HUD derive 時 OmEntityRef 缺 Debug/Default 導致 E0277。另一次猜測 OmGenerated plugin 路徑與 Windows wildcard 路徑搜尋失敗。
- 修正：讀取真正 protobuf、generated ability_by_name API 與 rg --files 清單；新增可選的 typed moba_hud 欄位、為 C ABI 明確升版 4，補齊零值 entity 的 derive。
- 決定：只顯示 configured player 的已披露英雄；HUD 是持續狀態，不依賴一次性特效。現有 Hero 沒有 current mana，不捏造滿魔數值；mana_supported=false，介面明確標示未支援。
- 預防：查證實際型別／路徑再編寫；產生 protobuf fallback 並驗證所有消費端；重新 staging header/DLL 後才驗證 Unreal。完成證據見本輪 HUD progress 文件，不把未完成介面列為驗收完成。
- 測試編譯另發現 Fixed64 不由 core runtime 重匯出，且 client 沒有直接 omoba-sim 相依；補上測試專用 dev-dependency 與明確 import，不依賴 transitive crate 名称。

## E064：單路 headless 成功不代表正式安全投影可用

- 程式檢查：單路只有 CircularVision，正式 Wave B 只讀 VisionSource；陣列 side 0/1 被當成 wire team，與正式支援 1/2 不一致。兵線移動沒有 Movement fact，兵塔可見來源傷害也不能由沒有該 AI 的 filtered client 重算。
- 決定：明確分離 side／authenticated team，加入 OwnerTeam scope 與真實視野來源，兵線發布可見 Movement；NPC 傷害用專用 external-direct-combat policy，只送可見目標結果，不暴露來源 ID。保留 TD 預設，不用全圖可見、停用 hash 或每 tick 全世界 repair 通過測試。
- 同步檢查：死亡實體已不在 view.entities，原 visibility loop 不會退休舊身分。以已公開的既有 remember policy 退休缺席身分，不藉此發布隱藏死亡資訊。
- 操作錯誤：本輪兩次猜測 system_dispatcher／visibility 路徑失敗；後續實際 dispatcher 在 shared native。首次新測試把 PaddedTeamFrame 當 TeamTickFrame、Death 猜成 entity 欄位；依實際 wrapper／proto／既有測試修正，不重複盲試。
- 驗證：core 299、base_content 62 passed；真實 KCP run 1791026421 success=true／cleanup_verified=true，雙隊 16／7 checkpoints 三方一致。完整技能／終局／UE 邊界未完成，見 `2026-10-03-single-lane-runtime-progress.md`。

## E065：replica gameplay 初始化與 Warmup 契約缺漏

- 首次雙隊測試在 tick 120 攻擊 timer hash 不同。逐 component 比對發現 filtered hero 缺 FacingBroadcast，未進入 HeroTick join；MasterSeed ECS resource 也未取 TeamGameStart.global_seed；idle jitter 使用 authority/local 不同的 Entity.id。
- 決定：补正 shared seed／必要 hero runtime components，owned hero jitter 用已公開 player ID；同隊仍友好，不同 Player team 明確敵對。
- 真實 KCP run `moba-runtime-1791024537` 再抓到 Warmup：server 冻結玩法，replica 卻繼續更新 timer，observer 多次恢復後 fail-closed，未降級協定。該失敗 run 已保存 report/logs，三個自行啟動的程序清理 verified。
- 決定：每 tick 發布無私有身分的 global HUD delta metric，Warmup／Finished／host Pause 的 delta=0；client 同樣跳過玩法而繼續處理 frame。測試改用真正 2 秒 Warmup，不以設 warmup=0 繞過正式問題。
- 測試工具初次錯把完成 snapshot 的 tick 當 next replica tick；改成正式 server 相同的 committed tick+1。保留每個 checkpoint hash 比對與診斷，不關閉 mismatch。
- SimulationDriver 的 inactive early return 原本回傳空 facts，會把新增的控制 facts 留到下一個 Playing tick；已改成每個 inactive tick 同樣 drain／驗證 facts。包含 2 秒 Warmup 與 host Pause 的雙隊 900 ticks 測試通過，保留全 checkpoint hash。

## E066：正式 server 只更新 local_tick，沒有更新 ECS Tick

- 真實 run `moba-runtime-1791025028` 使用 default Warmup，tick 120 components 一致，但 tick 240／360 的 TAttack.asd_count 不一致。透過 server expected-components 與 runtime components 逐欄比較，排除位置與版本。
- 原因：State::tick 只更新 transport local_tick；ECS Tick 一直是 0，HeroTick jitter／damage RNG／script facts 都用錯 tick。transport 投影卻標成 240，client 使用正確 240；之前 FOG demo 沒有戰鬥 timer，因此未暴露。
- 決定：在 tick 起點同步 ECS Tick=local_tick，再執行時間、輸入與玩法；不改 seed、不修補 hash、不降級 secure V2。兩次後續失敗 run 也已保存診斷並 verified 清理。

## E067：filtered runtime 戰鬥 registry 缺漏

- 真實 KCP run `moba-runtime-1791025237` 在約 tick 898 普攻進入結算後 panic：缺少 TowerTemplateRegistry；server UDP 10054 是 client 崩潰後的結果，不是根因。
- 決定：空 filtered world 初始化必要 registry；初次 bootstrap／rebootstrap 從既有 script registry 建立塔、升級、能力靜態內容，絕不初始化 Story 或隱藏實體。新增空世界 registry 回歸測試。
- 操作防錯：本次仍猜測 scripting/registry.rs、facts.rs、movement_tick.rs 不存在；以目錄 rg 找到實際 stable_fact.rs，後續不使用猜測檔名。
- `moba-runtime-1791025680` 無崩潰且雙隊前進，但舊 smoke 只檢查移動，不能當作同步通過；補上三方 checkpoint gate 後重新驗證。

## E068：可見敵英雄仍缺少敵隊移動優先狀態

- run `1791025237` tick 840：可見敵英雄 TAttack 在 authority 為 Idle，observer／外部 runtime 卻同樣進入 Backswing。兩個 client 相同不代表與 authority 相同。
- 原因：安全投影不傳敵隊 accepted input，filtered hero 沒有敵人的 MoveTarget，因此在敵方正在移動時錯誤啟動自動普攻。
- 決定：新增每 tick、僅可見 actor 的 MovementPriority 布林 fact。它不攜帶目的地、敵方輸入或 target ID；replica 只據此套用與 authority 相同的普攻移動優先門檻。目的地與真實移動仍由原有安全 Movement outcome 處理。
- 驗收門檻：雙隊至少前進 900 ticks，每隊至少 6 個三方 checkpoint，包含 tick 840；要求 authority／observer／external pre-repair 與 post-repair hash 一致，不能用 repair 或 client 相互一致冒充成功。結果另見本輪進度檔。

## E069：雙隊較晚加入與最終姿態不可只驗證早期 tick

- run `1791025876` 的同步全部通過但 gate 在第二隊只到 913 時要求六個 checkpoint；第二隊於 tick 240 後加入，960 的第六份報告尚未收到。改為至少 1080 ticks，保留六個 checkpoint 門檻，不減少 hash 檢查。
- run `1791026020` 因第二隊 catch-up 使第一隊走到 2105；tick 1800 敵英雄只有 Facing 不一致（authority 486、client 2532），其他 components 相同。Movement 只帶移動階段姿態，後續轉向／抵達完成不一定再次送出。
- 決定：單路 commit hook 發布 visible hero 最終 Pos/Facing 的 Movement outcome，phase=PostStep，仍受既有 visibility policy；不傳 MoveTarget、私有命令或隱藏 actor。新的敵隊移動回歸測試延長至 2400 ticks，原空輸入 900 ticks 測試保留。
- run `1791026167` 的 gate 讀取仍在 append 的 JSONL 尾行，報 unterminated JSON string。只讀以換行完成的 records，未完成尾行不當證據；完整行若 malformed 仍失敗，不吞掉同步錯誤。

## E070：NPC PreStep 運動被 client 延後到 PostStep

- 新敵隊移動回歸延長至 2400 ticks 後，tick 1800 的兵 HP 差 45（一發英雄普攻），authority 135、client 90。這不是前一個 Facing mismatch，不能重跑短測冒充修正。
- 原因：authority lane hook 在 dispatcher 前移動 NPC；舊 wire Movement 在 replica dispatcher 後套用，因此 replica 用上一 tick 的兵位置做攻擊判定。
- 決定：新增 PreStepMovement event kind（沿用僅可見 Pos 的 payload 與既有 policy），於 replica dispatcher 前套用。Hero 最終姿態仍是 PostStep Movement，不把所有位移一律提前，不暴露 NPC AI 或私有目的地。
- 驗證：原 900 ticks Warmup／Pause／NPC 傷害與新 2400 ticks 雙英雄移動測試均通過，每個 checkpoint 仍比對 authority hash。最終真實 KCP run 另記。

## E071：Unreal 單路 smoke deadline 早於測試移動

- run `interactive-ue-1791026808` 五程序單路已顯示英雄／兵／塔，兩個 runtime 持續前進；但 30 秒 smoke 尚未出現 issued approach move，沒有 scripted-move evidence，不能宣告通過。
- 已讀實際 C++：ApproachMoveAtSeconds 是首次 FocusCameraOnOwnedHero 的 wall-clock +28 秒；UE readiness 早於首個可消費 snapshot，冷啟動佔去額外時間。改以 60 秒重新跑，仍要求 UE 自己提交、rendered movement 和 consumed snapshot，不改成 runtime 代送。
- 本輪操作又猜測 codegen/templates／Windows `OmWorldBridgeActor.*`，rg 失敗；改直接對實際 OmGenerated 目錄搜索。不因搜尋不到就新增同名模板；不將 runtime evidence 缺檔當成 JSON parser 問題。
- 清理核對初次把含空行的 pid 檔直接 cast 成 int 失敗；改讀 Raw.Trim 後逐 PID 查詢。停程序後 JSONL 仍可能有未寫完尾行；同樣只解析換行完成的 records，不能用 parser error 推論 runtime failed。
- 最終 run `interactive-ue-1791026989` success=true：雙隊 safe tick 5321、雙 UE 自行移動且各 15 個 rendered frames／正確 consumed sequence；各 46 筆三方 PASS、零 FAIL，五個精確測試 PID 均退出。完整 HUD／技能／终局仍未完成，不勾选完整框架。

## E059：單路對局實作的路徑與 API 假設

- 現象：只讀查詢再次猜測 native/mod.rs、game_processor.rs、runtime/src、hero_command.rs；Windows rg 不展開 comp/*。首次編譯另發現 CircularVision::new 只有兩參數、AttackMove 欄位是 pos、Vec2I 是 i32；commands 變數插入到 digest 而非 Bot scope。
- 決定：改用 rg --files／對目錄搜尋，再讀實際定義；修正呼叫，限制 lane_length 避免輸入 raw 溢位，保留命令狀態在 digest 並在 Bot 自己取得 storage。後續驗證以實際 cargo 結果為準。
- 診斷輸出首次使用 Fixed64 的 Display，該型別只支援 Debug；改為 {:?}，不為方便輸出添加模擬浮點轉換。
- loader.rs 初次仍猜測位於 native/scripting；實際在 omoba-core/src/scripting。格式化後 patch context 未匹配時工具原子拒絕；重新讀實際區段後套用，不混用格式化前的片段。

## E062：兵塔傷害不能只繞過戰鬥計算直接扣血

- 程式檢查：首版 lane AI 發出 Outcome::Damage，雖經正式結算但未經 DamageInstance 的護甲／暴擊／DirectCombat fact 管線；Unit::new 預設數值與 CProperty／Lua 英雄數值不一致。
- 決定：兵塔提交 DamageInstance::new_attack，交由既有 damage_tick 生成結算及可觀察戰鬥事實；英雄 Unit／CProperty／TAttack 取同一 generated Lua stats，重生 HP／傷害包括既有等級成長。修改後重新執行對局與逐 tick 重播，舊報告不能當成新碼驗收。
- 同步修正 headless SimulationDriver 把 tick 誤傳為 script rng_seed（正式 server 使用 MasterSeed）；每 tick drain 並驗證 OrderedFact，不累積整場未讀 facts。新的 facts 是回傳資料，不在 renderer 重建權威事件。

E059–E062 最終驗證：base_content 59 passed、core 296 passed、backend 135 passed／1 既有 ignored、core no-default-features 編譯通過。固定 Lua + 真實 release DLL：15Hz seed 1，3155 ticks／27 波／[11,1] 死亡與重生／864 combat facts；120Hz seed 1，15480 ticks／16 波／[7,3] 死亡／[7,2] 重生／953 facts。兩場 team 0 勝、game.end 各一次，所有 tick replay 一致。詳細 digest／指令與邊界见 `2026-10-03-single-lane-moba-progress.md`。15Hz／120Hz profile 間不宣稱相同平衡結果；沒有 Unreal 完整對局驗收。

## E063：CRLF 被 submodule 的 diff whitespace 預設當成尾空白

- 現象：根 repo diff --check 通過，但 omb/Cargo.toml 新 bin 宣告的 CRLF 在 submodule diff --check 被報 trailing whitespace。
- 決定：保留既有 Windows 行尾，不改 shared git config；只讀檢查使用 `git -C omb -c core.whitespace=blank-at-eol,blank-at-eof,space-before-tab,cr-at-eol diff --check`，仍檢查真正尾空白／EOF 問題，該檢查通過。
- 預防：report 路徑可能仍有上次成功檔，必須等待本次 Lua／CLI exit 0，再讀其結果；本輪等待 120Hz 完成後才引用新的 15480 ticks 報告，不能把運行中的舊 16594 ticks 報告當成本次通過。

## E061：首場 Bot 對局達到時間上限卻沒有結束

- 現象：真實 ECS／正式输入測試 4500 ticks 後仍 Playing，38 波、9 死亡／9 重生；其餘四個對局規則測試通過。
- 決定：先印出失敗對局的實際位置、血量與命令找出死局，不以延長上限或直接宣告勝方通過驗收。原因／修正及最終結果追加於此。
- 診斷：敵塔已被摧毀，但兩座基地都滿血；Push 保留 AttackMove，反覆停下清兵／敵英雄，沒有完成建築目標。修正：技能在合法範圍內優先當前建築目標，敵塔倒下後正式提交 AttackTarget 基地；相同命令不重送，避免每 tick 打斷普攻前搖。

## E060：戰鬥結果依賴 wall-clock 負載與平行收集順序

- 程式檢查：hero_tick 以本次系統耗時 <50ms 決定是否攻擊；par_join outcomes 未按 entity 排序，同距離目標缺穩定 tie-break。
- 決定：只以 fixed delta 判定是否推進攻擊；按 entity reduce，搜尋同距離以 ID／generation 排序。對局以逐 tick digest 及正式輸入重播驗證，不把最終勝方相同當成完整一致。

## E058：直接刪除 Saika 分支會移除既有 Blueprint 的 reflected 介面

- 程式檢查：通用事件模板內嵌 Saika ID／技能判斷與 typed payload；現有 Editor BlueprintSurface／SaikaEventDispatch 使用 HandleSaika*／FSaika*。直接刪除不是無回退的模板遷移。
- 決定：先隔離 legacy_hero_compat adapter，通用模板不再直接判斷角色；保留舊名字、manifest shape、生成 bytes 及 ABI。新英雄不得經此 legacy 層擴充。真正資產／typed projection 遷移另驗，不將隔離冒充完整移除。
- 驗證：初次搬移後既有 codegen 22 tests 與 shipped --check 全過、輸出不變；新增防回退與 UE 驗證結果見 generic-event-adapter 進度檔。本輪搜尋不到可用 Blueprint inspection 字串（rg exit 1）只代表需查工具文件，不能憑空假設已有 API。

E058 最終驗證：codegen 24 passed、shipped --check 11 files／13 inputs unchanged；full build 31311 exit 0、Editor 68560、stage SHA `c5736f6bb22e92cef41c4edd53eba413305bcffbad8c7817db12f2c3ce9a6863`；native 10981 同 Editor 兩輪各 7/7、PIE 75028 native／ghost rendered=true 且 stop。MCP 查實際 Blueprint skeleton／graph intent：31 nodes，legacy HandleSaikaActionEvent 接到 ABP 動畫更新，不是可猜測為空的舊入口；未改資產圖。不將隔離相容模組誤算成 2.2b 全部完成。

## E053：runtime 呈現節流與 latest 通道仍可吞掉未消費傷害

- 程式檢查：last_injections 每 simulation step 取代，原本只有發布 snapshot 才取 external effects；即使 bridge 保留 busy／published slot，也接不到此前被節流或 watch overwrite 丟失的事件。
- 決定：每個已套用 step 先只 drain DirectCombat，其他持續 state effects 保留。runtime 有界 ledger 跨發布保留 DMG1，最多 1024 筆、4096 ticks；記錄第一個成功寫入 socket 的 snapshot／connection，只有該連線有效 ACK 才退休，不把 prepare／publish／release 當消費。reconnect baseline 不重播歷史；Hide／Forget 僅清理同 disclosure epoch，ResetView／新 view epoch 清理。送出前再次過濾已退休事件，涵蓋 ACK 前建立但 ACK 後送出的 envelope。
- 邊界：capture 只接受已套用且有序的 replica step，不是任意外部重送 API；過期／超量為明確丟棄，第一筆超量記警告。只涵蓋 DMG1 外部直接傷害，不宣稱所有音效／技能 reliable delivery。
- 驗證：core 296 passed、runtime 38＋2 passed；新增真實 TCP latest overwrite／ACK 退休／過時 prepared frame 過濾，以及容量／TTL／epoch／baseline 單元測試。完整 UE 驗證另記進度檔。

## E054：新增測試 patch 含空的 update hunk

- 現象：apply_patch 拒絕「Update hunk does not contain any lines」，整份修改未套用。
- 決定：重新提交含實際替換內容的原子 patch，確認 test helper 可見性與 socket 測試都存在後再測，不假設部分成功。
- 驗證：新增 retention／TCP 測試均已編譯且通過。首次測試另有 cfg(test) wrapper 的 unused_mut，已移除；保留正式 serve receiver 的 mut。

## E055：清理核對把含換行的 PID 檔當整數陣列

- 現象：只讀核對使用 `[int](Get-Content path)`，PID 檔含額外 CR／空行，PowerShell 得到 Object[]，整數轉換失敗，後續 Get-Process 的 Id 為空。最末 diff 指令成功讓整條 command exit 0，不能據 exit code 宣稱前面檢查通過。
- 決定：讀 `Get-Content -Raw` 後 Trim，先驗證 `^\d+$` 再轉數值；只查該 run 保存的五個 PID，不以程序名稱終止任何東西。沒有執行刪除／終止。
- 驗證：run 1791018681 的 server 71164、runtime 66260／73736、UE 30292／35540 全部 StillRunning=False；active-ue-session.json 不存在。雙隊 smoke 本身 exit 0，保存 observation 複驗成功。

## E056：空傷害 ledger 不應在每個 presentation frame 建 entity 集合

- 程式檢查：有界 retention 投影為避免 events×entities 掃描而建立 BTreeSet，但即使沒有待送傷害也掃描全部 entity，對 stress 熱路徑增加無效成本。
- 決定：entries 為空直接返回；有待送事件才建可見身分集合，仍精確比對 epoch。屬實際效能風險修正，不將未執行的 10000 entity benchmark 冒充已通過。
- 驗證：修改後重新執行 runtime／bridge 回歸，再重新 build／stage；不能沿用修改前的 DLL SHA 或 UE 報告。

## E057：長時間 renderer 離線時 saved latest tick 不是新的恢復時點

- 程式檢查：presentation_enabled 的 30 秒 grace 結束後 latest snapshot 停止更新，但 runtime 仍進行 replica steps。新 renderer 若只以 saved snapshot tick 清 ledger，離線期間事件可能被當成新 cue 重播。
- 決定：恢復基準為 max(saved view tick, ledger 已套用 high_tick)，所有初次／late-first snapshot 路徑使用此回傳 floor；不將舊 view 偽造為新的 state tick，只提高歷史事件排除界線。
- 驗證：baseline 測試以 saved tick 2／current tick 3 驗證兩個歷史 cue 全部退休，tick 4 新事件可保留；現有真實 socket overwrite／late-first／reconnect 測試與 bridge 回歸再跑。最後 full build／stage／UE 報告另記進度檔。

E053–E057 最終驗證：core 296、runtime 38＋2、bridge 39＋2 passed（1 外部 KCP ignored）。最終 full build 46020 exit 0，Editor 36276／MCP 30000；built／staged SHA `c7288cd34861753bab3656c0146eec0e4f8331d197908b34d59a7c1d23ca34f0`。native 14423 同 Editor 兩輪各 7/7、PIE 26431 native／ghost rendered=true 且 stop；dual 67749／run 1791019365 exit 0，兩隊 own-only／UE 與 replica 位移／ACK 1105、1533。五程序退出與 active session 移除已核對，saved observation PASS，最後 stage gate 一致。此證據涵蓋外部傷害管線分段與 demo 移動，沒有將完整戰鬥／所有 cue 或 4.3 誤算完成。

## E046：只讀搜尋再次猜測 runtime.rs／未存在的 projection 函式檔名

- 現象：rg 參數包含 omoba-core/src/runtime.rs，但實際為 runtime/mod.rs；另一次 projection 函式搜尋無符合項目（exit 1 不是建置失敗）。
- 決定：先 rg --files 確認路徑；不把搜尋不到當成可假設的 API，改讀現有 selective_replica／driver 實作。

## E047：patch automation 測試文字未對上

- 現象：多檔 patch 最末測試 anchor 文字不符，整份 patch 未套用。
- 決定與驗證：先 rg 查是否有新增符號（無），讀真實原文後重套，不假設前面的檔案已修改。

## E048：replica ID 與 native snapshot ID 寬度不同

- 現象：bridge 編譯拒絕 u32 entity_id 與 u64 cue.target_id 直接比較。
- 決定：比較時將現有 native ID 無損提升 u64；decoder 對 native 無法表示的 ID／epoch 使用 try_from 拒絕，不截斷。修正後 bridge 39 unit＋2 TD integration 通過。

## E049：擴充測試後重用已 move 的 fact

- 現象：core 測試編譯指出單元素陣列已 move fact，新增雙來源驗證無法 clone。
- 決定：測試第一輪使用 fact.clone()，正式投影不增加不必要 clone；重新執行 core 回歸。

## E050：producer-local ordinal 撞號與 coalescing 丟失一次性提示

- 程式檢查：外部效果 stable_sub_index 原為各來源 local_ordinal，不能充當隊伍內唯一 cue ID；bridge 原本合併 snapshot 只留下最新，舊 snapshot 的 one-shot 會被吞掉。
- 決定：先排序已清洗的效果，再配置隊伍內索引（不加入隱藏來源 ID）；ID=(replica_tick << 32)|(index+1)，overflow 拒絕、不截斷。bridge 保留 batch／busy pending 提示，去除相同 ID，限 1024，僅允許最新可見相同 epoch 目標；reset 清除。Buff／持續狀態不冒充 one-shot。
- 驗證：雙 hidden producer、同 tick／target／local ordinal 的兩件事仍有不同索引，反序輸入產生同結果；typed codec／runtime mapping／bridge decoder、busy retry／coalescing／epoch change／reset 測試陸續加入，不提前宣稱所有 cue 完成。

## E051：重連基準之後的 snapshot 仍可帶入舊傷害

- 程式檢查：僅清除首個 snapshot 的 effects，無法阻止之後的 retained damage 在新 renderer 播放。
- 決定：每條連線保存首個完整 snapshot 的 replica_tick，DMG1 damage 只有 tick 大於這個基準才可出現在後續 live frame；不修改共享 source。沒有舊 cue 時 Cow 借用，避免每 frame 複製整個視野。未知格式維持舊相容邊界，bridge 不會 dispatch 成 DMG1。
- 驗證：late-first snapshot 的 latest／critical 兩種真實 socket 測試，後續 frame 的 tick 3 damage 被剔除，tick 4 保留；不能把這項專用契約宣稱為全部音效／技能完成。

## E052：已發布到 ring 但尚未處理的提示也可能被下一個 view 覆蓋

- 程式檢查：僅保留 busy pending build，不能涵蓋已成功發布、UE 還沒呼叫 mark 的 slot。
- 決定：slot 保留 typed cues 與 consumed 狀態；合併新 view 時攜帶未處理的最新 slot 提示，mark 後才清除。不把 release／publish 當消費。pending／published 均遵循 visibility／epoch／reset 與 1024 上限；UE 同一 ID 重播由獨立 history 阻擋。
- 驗證：bridge regression 新增 published-but-unprocessed → 新 view 仍保留；明確 consumed 後下一 view 不再攜帶，39 unit＋2 integration 通過。

E046–E052 最終驗證：core 295、runtime 34 library＋2 binary、bridge 39 unit＋2 TD integration passed（1 外部 KCP ignored）。full build 74762 exit 0、Editor 98896／MCP 30000；stage SHA-256 2f7c43f332003d04ab712850070ce49607c3ea7ec06707db22a86b4b6d8046e9。native 16540 同 Editor 兩輪各 7/7（含新增 damage dispatch assertions），PIE 86336 native/ghost rendered=true、memory=[1,0]、已 stop。dual 57125／run 1791017466 exit 0：兩隊 own-only／UE 與 replica 位移／ACK 1154、1595；五程序清理、active session 移除。DirectCombat 管線分段驗證，不當作完整戰鬥／所有音效／4.3 完成。

## E042：bridge frame 序號不等於 IPC snapshot 序號

- 程式檢查：Unreal lease.sequence 是本機 ring 發佈序號；driver 原先丟棄 envelope.sequence，控制 frame 也會增加本機序號，不能直接送 RendererConsumed。
- 決定：原始 connection ID／snapshot sequence 只存在 Rust 的 extras → pending build → frame slot；新增 om_mark_frame_consumed，UE 在 ProcessFrame 返回後、ReleaseFrame 前回報。單純 release、控制 frame／lifecycle 重建不 ACK；同 slot 重複 mark 冪等。ABI 結構與 v3 不變，UE 要求新 export 存在，舊 DLL 不靜默降級。
- 驗證：真實 TCP＋C ABI 測試刻意使用 IPC 97 與不同本機序號，release 無封包、mark 正確送 97、duplicate/control 無額外 ACK；busy ring retry／coalescing metadata 保留通過。

## E043：延遲的舊 lease 可能污染新連線，release 原先未驗序號

- 程式檢查：只保存 snapshot sequence 無法區分斷線前後；release 原先只驗 slot pointer，沒有驗 lease.sequence／該 slot 是否仍有 reader。
- 決定：程序內 AtomicU64 配發不重用 connection ID（包含 driver 重啟）；writer 只接受自己代次的 ACK，採最高序號並限制每次最多取 256。mark/release 都核對 slot pointer、sequence 與有效 reader；已釋放或錯誤序號不得修改 reader 計數。
- 驗證：兩次真實 socket 重連，刻意把第一連線的 999 ACK 放入第二連線，第二 socket 只收到自己 snapshot 1 的 ACK。C ABI wrong/released lease 拒絕、仍可正確釋放；37 bridge unit＋2 TD integration 通過，1 外部 KCP 測試維持 ignored。

## E044：只讀檢查猜測不存在的 bridge/build.rs

- 現象：本輪附帶檢查 bridge/build.rs 不存在，雖未修改任何檔案，命令 exit 1。
- 決定：遵循 E018，先以 rg --files 確認路徑；現有 header 產生流程在 omfue/build_bridge.bat，不再猜測 Cargo build script。

## E045：只讀 PowerShell 行數參數誤植

- 現象：檢查 subsystem 時誤寫 Select-Object -Skip sixty，整數參數解析失敗；未影響建置或修改檔案。
- 決定與驗證：改為明確 -Skip 60 後取得完整 export 必要性檢查；之後行數參數僅使用數字，不把文字描述放入指令。

E042–E045 最終驗證：full build 49694 exit 0，Editor 89572／MCP 30000；bridge built/staged SHA-256 5ec882ff08ada09fd2f05987e59b3279ebfc6d33d96ff06511d3b2d177d52ee4。Editor 同一 instance 兩輪各 7/7、PIE native/ghost rendered=true 並停止。dual run 1791016176 exit 0，兩隊 filtered view／UE 與 replica 位移、runtime 收到 snapshot 1090／1772 ACK 全通過；owned 5-process cleanup、active session 檔移除。4.3 仍未封關，正式 cue ID／所有 effects/audio 投影尚待完成。

## E038：renderer 初始安全 snapshot 可能重播歷史一次性效果

- 程式檢查：serve_renderer 原先直接寫出共享 latest snapshot。effects/audio_cues 一旦帶入 retained 資料，新的 renderer process 可重播舊效果；先收到 RuntimeReady、後收到第一個 snapshot 的分支也有相同問題。
- 決定：每條 renderer 連線的第一個完整 snapshot 是恢復基準，只複製一次並清除 effects/audio_cues；可見單位、ghost、fog、view epoch、tick 及 input result 保持原值。latest 與 critical 首個 snapshot 路徑都使用相同規則；之後 live one-shots 仍保留。不修改共享 watch source，不重新啟動 replica。
- 驗證：Rust 真實 TCP 連續兩個 renderer 在沒有新 simulation frame 時恢復同一 ghost，歷史 VFX/audio 不重播；延後首個 snapshot 的兩條路徑亦通過，新 live cue 與 terminal input result 保留。runtime 33 lib + 2 binary passed。
- 尚未解決：目前正式 snapshot builder 仍未映射所有 MOBA effects，RendererConsumed 仍缺少 UE lease sequence → IPC sequence 對應；持續 live frame 的完整 cue 身分／去重契約尚未完成，不能將「初始基準不播放效果」冒充 4.3 全部完成。buff/script 狀態更新不可直接用 projectile key 去重，以免合法重建／更新被吞掉。

## E039：RendererConsumed 可回報尚未送出的未來序號

- 程式檢查：runtime 原先忽略全部 RendererConsumed，沒有區分真實消費與未來序號；不能基於這些回報安全清理 retained effects。
- 決定：每條連線追蹤實際送出的完整 snapshot 最大序號，只接受不超過此界線的回報；duplicate／older lease 回報以 fetch_max 冪等處理，未來序號明確 IPC error 並斷線。input result／RuntimeReady 不能抬高 snapshot 界線。
- 驗證：兩條首個 snapshot 路徑皆以真實 socket 發送有效、重複、較舊回報後仍接收新 live cue；未來序號 999 被拒絕並關閉 socket。runtime 33 lib + 2 binary 全數通過。尚未用此 cursor 刪除 effects，需等正式 cue ID／UE 消費對應完成。

## E040：stage 後執行 bridge Cargo test 重新產出不同 DLL

- 現象：本輪 full build 31119 與双 client run 1791015128 通過後，SHA-256 比較發現 bridge/target/debug/om_bridge.dll 與 Unreal 已 stage 的 DLL 不相同；build 後再跑 bridge tests 會重建 cdylib。即使只有測試 fixture 更新，也不能用 ABI 相同宣稱二進位一致。
- 決定：正式驗收固定為所有 Rust tests 先完成，再 build/stage，最後 Editor／PIE／dual client；stage 後不再執行會重建 bridge DLL 的 Cargo 指令。驗收前比較兩個 DLL SHA-256，若不同先重新 stage，不忽略差異、不在載入中覆寫 DLL。
- 驗證：重新 full build/stage 後核對 SHA-256，再執行本轮最終 Unreal 回歸；先前 smoke 保留，不取代新二進位驗收。

## E041：Lua 測試 fixture 的 and/or 把 nil 缺檔值換成另一檔案

- 現象：stage gate 的缺少 DLL fixture 測試先失敗，因測試 stub 用 `condition and built or staged`；built=nil 會自動落到 staged，沒有模擬真正缺檔。正式 sha256 gate 未使用此寫法。
- 決定：stub 明確 if/else 保留 nil，重新測 identical/mismatch/missing 三種情境；verify-only 不得啟動任何 process，不透過改寫真實 DLL 測失敗分支。

E038–E041 最終驗證：runtime 33 lib + 2 binary、bridge 36 unit + 2 TD integration passed（1 外部 KCP ignored）；full build 31227 exit 0、Editor 10556/MCP ready，build/stage SHA-256 一致。session 7289 同一 Editor 兩輪各 7/7，PIE native/ghost rendered=true、memory counts=[1,0]、PIE stopped。最終 dual run 1791015412 exit 0，兩隊各自 filtered view、UE 與 replica 位移皆通過。stage gate 相同／不同／缺檔三案例、Lua module tests、codegen --check、diff whitespace 通過；4.3 仍留未完成，不冒充完整 cue 消費契約。

## E028：player_id 被當成 team_id

- 現象：bridge 的 start_driver_if_configured 將玩家 ID 直接填入 DriverConfig.team_id，Unreal 也沒有傳遞獨立隊伍欄位。玩家 7／隊伍 2 會送出錯誤握手。
- 決定：C ABI v3 增加 explicit team_id；UE 解析 -om-team／-om-team-id 或 OM_TEAM_ID／OMB_TEAM_ID，不猜測隊伍；IPC 拒絕零玩家／零隊伍。舊 TD 非 IPC 保持相容，舊 ABI 明確拒絕。双 Unreal 啟動明確提供玩家與隊伍，缺少設定不降級直連玩法。
- 驗證：新增 player=7/team=2、缺失 ID、舊 ABI 拒絕測試；Rust 34 unit + 2 TD integration 通過，1 外部 KCP ignored。完整 UE 与双队測試結果另記進度。

## E029：雙 Unreal launcher 只用檔案存在判定建置新鮮度

- 現象：run_2player_ue.lua 即使未要求 skip-build，仍只在 EXE 不存在時建置，可能執行舊 runtime／server；ready 只檢查 LoadMap，不能證明 IPC 與移動成功。
- 決定：非 skip-build 每次執行增量 Cargo build；smoke 模式新增安全 replica 與原生 UE 呈現／移動報告，未達斷言就失敗並保存錯誤。保留互動模式，不用等待時間或 LoadMap 冒充驗收。

## E030：Unreal -log／-abslog 未產出預期檔案

- 現象：真實双 client run 1791012372 失敗於讀取 ue-p1.editor.log；實際全部日志在 host 擷取的 ue-p1.stdout.log。
- 決定：驗收使用啟動時必定擷取的 stdout，不再假設 -abslog 和 -log 同時生效；保留首輪 failure report。

## E031：積壓的完整 snapshots 被逐一投影

- 現象：双 client 第二輪 1791012553 正確拒絕驗收，25 秒內沒觀察到自身移動；日誌只有最初 frame。code review 發現 drain_driver_updates 無界收集，再逐一生成大視野 frame，最後只保留最後 frame。
- 決定：單次最多取 256 updates；只投影最新完整 snapshot，所有 removal identities 與 terminal input results 保留。reset 淘汰先前 snapshot，較新的安全 snapshot 才恢復 view。新增 coalescing 單元測試，再以實際双 client 復驗；尚不以程式檢查認定這是停頓的唯一原因。

- 後續證據：退出後 stdout 刷新顯示 frame 其實持續增加；demo 的 ApproachMoveAtSeconds 固定在相機定位後 +28 秒，25 秒 smoke 不足。改用 50 秒觀察，不降低移動斷言，也不將日誌緩衝等同主執行緒永久停頓。coalescing 是實際可測的熱路徑改善，但不宣稱它解釋所有延遲。

## E032：舊 creep Blueprint 與原生 BodyMesh 欄位撞名

- 現象：dual UE 的 BP_PracticeDummy 編譯回報 BodyMesh 在 SKEL_BP_PracticeDummy_C 與 OmUnitActor 重複；既有 native 自動化沒有測這支舊 Blueprint。
- 決定：新增明確 -om-native-content 選項，生成前端路徑優先走 native registry 類別，避免不必要的舊 Blueprint；保留原資產與未帶選項時的相容路徑，不刪除使用者 graph。

## E033：-game 第一個 frame 等待額外載入的 fog Plane

- 現象：run 1791012933 的 50 秒驗收仍失敗，p1 stdout 最末停在 Waiting on static mesh /Engine/BasicShapes/Plane.Plane being ready before playing；runtime replica 無落後持續前進。原生程式在 EnsureFogOverlay 第一次 frame 額外 LoadObject Plane。
- 決定：fog overlay 重用 constructor 已載入的 Cube primitive，Z 壓薄至 0.001，保留尺寸／高度與 fog mask 材質；不改共享引擎、不編造收到 frame 就已玩得動。以雙 -game client 復驗判斷是否解決卡點。

- 復驗：full build 52197 exit 0；run 1791013302 不再停在 Plane，兩隊都收到自己的英雄，但 50 秒仍未送出移動。frame 日誌間隔偏長，+28 秒的 demo 移動只在 ProcessFrame 觸發，故不能把原網格修正宣稱為完整解法。

## E034：固定睡眠把場景載入時間混入移動驗收

- 現象：LoadMap 不代表相機已找到英雄；從 LoadMap 起睡 50 秒，可能不足以涵蓋首次安全投影、相機定位後 28 秒，以及下一個 presentation frame。
- 決定：smoke_seconds 改為條件等待的上限；兩隊 UE 移動日志與安全 replica 座標變更均存在才完成，逾時明確失敗。保留 filtered-world 與 own-only 斷言，不提高效能評價、不注入 runtime scripted move。
- 操作防錯：本輪再次使用不存在的 runtime.rs／runtime_driver.rs，以及把 Windows glob 直接傳給 rg，造成唯讀查詢失敗；先 rg --files 確認路徑，glob 必須使用 rg -g。未產出文件亦先檢查存在再讀。

## E035：安全 replica 接收佇列落後讓移動過期

- 現象：120 秒條件等待 run 1791013521 中兩邊 UE 都發送移動，但 server 明確拒絕 player 1：original_tick=3138、current_tick=5769、late_by_ticks=2632。runtime inbound_depth=1024，lag=0 僅代表已收到 tick 與已套用 tick 相同，不能代表沒有 server backlog。
- 程式證據：snapshot_envelope 對靜止英雄每次重新執行 demo fog raycast（約 15,000 tiles × 64 trees）；追趕丟棄的普通 snapshot 也做這份重工。
- 決定：每個 PresentationHub 保存一份 exact-center/team demo fog cache；相同完整座標及隊伍才重用，位移 1 raw unit、隊伍變更、空視野/reset 都失效。資料僅來自安全 snapshot，不放寬後端 late-input 規則，不跳過 replica simulation。
- 驗證：新增 exact-key 命中／失效及與未快取結果逐欄相等測試，再跑真實双 UE；尚不以 lag=0 宣稱效能合格。Windows 對仍開啟 File 的 Length 可能尚未更新，看到 capture Length=0 不代表讀不到內容，須實際讀取。

- 首輪 cache 復驗：run 1791013831 exit 0，兩隊 replica 都位移，server 正常套用輸入。加強下一輪驗收要求 UE frame 日誌自身位置亦變更。

## E036：Lua pattern 誤混入反斜線 escape

- 現象：新增 UE 位置解析 pattern 時寫成反斜線加百分號，loadfile 明確拒絕 invalid escape sequence；尚未執行新雙 client 測試。
- 決定：Lua pattern 使用百分號、不用反斜線，修正為 `([-%d%.]+)`；先 loadfile syntax gate，再用已保存的兩隊實際 frame 日誌檢查位移解析，才跑 launcher。

## E037：最終重建再次碰到外部程序鎖住共享 Renderer DLL

- 現象：full build 80262 的 Rust bridge／DLL stage 成功，但共享引擎有其他變更，UBT 重建 Renderer；UnrealEditor-Renderer.dll 連結被 `D:\UE5.8\CrowdRuntimeStable\Engine\Binaries\Win64\UnrealEditor-Cmd.exe` 鎖住，UBA 等待 20 秒後重試仍 LNK1104，exit 1。
- 決定：遵守 E007，僅唯讀核對程序，不能停止其他專案、覆蓋引擎或用舊成功輸出當本次成功。保存 stage 成功與 full build 失敗的區別，完成不依賴 Editor 的 Rust／Lua 驗證；外部鎖解除後才復驗最終 UE build。

- 最終復驗：外部 PID 20476 自行結束；full build 3355 exit 0、Editor 52212/MCP ready。run 1791014362 的加強 live smoke exit 0，兩隊 UE 日誌及 replica 都位移。session 90909 的同一 Editor 兩輪各 7/7，PIE native/memory rendered=true、memory counts=[1,0]、PIE 已停止。bridge 36 unit + 2 TD integration（1 外部 KCP ignored）、runtime 31 lib + 2 binary、Lua module與 6 parser cases、codegen --check 均通過；4.2 封關，整體 15/30，未宣稱整個框架完成。

## E024：完整視野、控制 frame 與局部生命周期不能混用

- 現象：接 UE memory 時發現輸入結果發布空 FrameBuild；若每個 frame 都覆蓋 ghost，輸入結果會清掉記憶。ViewRemoved 用完整空 frame 處理局部 Hide，亦可能清掉無關角色。
- 決定：C ABI v2 增加獨立 remembered ghost 陣列與 presentation_snapshot 標記；控制 frame 不修改 ghost；生命周期只變更已取得的 safe snapshot，完整空視野才清除所有角色／記憶。ghost-only 斷線仍必須發布 reset，不能因 live removals 為空而跳過。
- 忙碌處理：保留最新完整安全 frame 直到有空閒 lease slot；失敗時取回原 build，不複製每個正常 frame，也不重複附加 pending control records。
- 驗證：新增 Rust lifecycle、decode、lease 與 UE marker 自動化測試；最終結果見本輪進度文件。

## E025：UE 5.8 TestNotNull 無法推導 TObjectPtr

- 現象：完整 build session 19431 exit 1，測試 GetStaticMesh() 傳給 TestNotNull 出現模板推導錯誤；其回傳型別為 TObjectPtr<UStaticMesh>。
- 決定：以 .Get() 明確取得原始指標，不修改正式呈現流程。依本地實際回傳型別修正，完整重建後再跑測試。
- 預防：automation assertion 需核對引擎 API 型別，不假設 TObjectPtr 的隱式轉型能參與模板推導。

## E026：PIE 文字序列化預設值與背景渲染驗收時序

- 現象：新的 PIE memory smoke 首次失敗於 missing memory diagnostics。GetBridgeStats 的 ReturnValue 已正確記錄 1→0，但 Unreal ExportText 省略與預設值相等的零欄位；ghost 的首次 WasRecentlyRendered 也在 screenshot 實際擷取前回傳 False。
- 決定：先核對 success 與 FOmWorldBridgeStats 回傳型別，只有通過時才依該結構預設值解讀省略的 count=0；不把任意缺失回應當成功。marker screenshot 在 WasRecentlyRendered 之前擷取，以背景 Editor 的實際渲染結果驗收；不取消渲染斷言。
- 驗證：修正後 PIE smoke exit 0，native_mesh_rendered=true、remembered_ghost_rendered=true、remembered_ghost_counts=[1,0]，自行啟動的 PIE 已停止。

## E027：停止 runtime 不可只丟棄仍顯示的 actor 參照

- 現象：程式檢查發現 StopRuntime 使用 ReleaseAllActors(false)，既不隱藏也不 Destroy live actor，隨即清空 EntityActors／ActorPools，可能留下無人管理的顯示物。
- 決定：明確停止 runtime 時使用 ReleaseAllActors(true)，銷毀 bridge 所管理的 live／pooled actors 並清空 memory markers。Editor-only memory fixture 不啟動 runtime，也覆寫 EndPlay，避免 fixture 清理停止另一個 bridge 的對局。
- 驗證：完整 build、Editor repeat 與 PIE cleanup 為驗收，不以單純指標清空宣稱 actor 消失。

本輪操作錯誤補記：E018 再次猜測 WorldBridgeActor 在 Source/omGame，實際位於 Plugins/OmRuntime/Source/OmGenerated，已用 rg --files 解析正確路徑。另一次 apply_patch 尾段使用不存在的「#」上下文使整包驗證失敗；確認沒有部分寫入後改用真實全文段落重套，禁止用假錨點追加 MD。

E018 同輪也出現猜測 Public/om_bridge.h 與 bridge/README.md 路徑的唯讀搜尋失敗。實際 header 在 Source/ThirdParty/OmBridge/include；不能只對程式來源遵守「先列路徑」，文件與生成產物也必須遵守。

E024–E027 最終驗證：Rust 33 unit + 2 TD integration passed（1 外部 KCP ignored）；full build session 84528 exit 0、Editor PID 91580；session 28134 同一 Editor 兩輪各 7/7，自動化零 failed／skipped／not_run；PIE 17 steps PASS、native 與 memory 皆 rendered、memory counts=[1,0]，自行啟動的 PIE 已停止。詳細決策及剩餘範圍見 `2026-10-03-unreal-remembered-ghost-progress.md`。

## 已確認的錯誤

| 編號 | 現象與原因 | 修正與預防 | 驗證 |
| --- | --- | --- | --- |
| E001 | TCP 分段讀取被 `select!` outgoing 分支取消，已消耗的封包前綴遺失 | 接收與送出各自持續執行，只在整條連線結束時取消 | 分段 socket／重連測試通過 |
| E002 | renderer 排入輸入就回報 applied，且 secure request ID 與 renderer request ID 混用 | 等 runtime 終局結果，保留原 request ID；重要結果保留至 frame 被取得 | runtime／bridge 測試通過 |
| E003 | 舊 source patch 要求不存在的插件 ImportTools.cpp | 識別完整預編譯 2.0.6 套件；未知版本停止；另驗證真實動畫 | 完整建置與動畫匯入通過 |
| E004 | `import_animation` 把 SkeletalMesh 當主要 AnimSequence 回傳 | 找到額外 `_Anim`，以真實型別、時長、frame、Skeleton 依賴驗證，保存所有額外產物 | 5 支動畫、重跑套件 hash 通過 |
| E005 | 材質工具無法從 `_mat` 檔名猜貼圖角色 | 明確指定 BaseColor，不靠檔名推測 | 材質 valid、零 issue、Texture2D 依賴通過 |
| E006 | 重試材質建立回報 already exists；材質摘要也不是 class 欄位 | owned pending 材質讀回與驗證後保存，不刪除重建；用 validate_material 驗證 | 配方重跑零 import／零 binding 變更 |
| E007 | 另一個專案鎖住共用 Engine DLL，LNK1104，啟動 0xc0000135 | 不停止別的專案；獨立驗證 Rust，待 DLL 可連結後重建本專案 | 其他程序自行結束後完整建置通過 |
| E008 | `openspec status` 的 isComplete 只代表計畫 artifacts 完成 | 用 instructions apply 的 progress 與 tasks 勾選判斷實作完成度 | 目前實作 15/30，不宣稱框架完成 |

## 本輪計畫與決定

- 新增通用 native skeletal 呈現，生成器從現有 Lua render 資料產生 soft asset 引用與動畫區間。
- 不在 native CDO constructor 載入資產；於 BeginPlay 初始化，以免 Editor 載入類別時同步匯入或載入美術。
- 若既有 Blueprint 已提供 SkeletalMeshComponent，保留它的呈現路徑，避免同時出現兩套 mesh。
- 缺少模型保留方塊；動畫僅改變呈現，不計算傷害或驅動權威 tick。
- 本輪新錯誤追加於下方，並以實際建置／Editor 結果更新驗證狀態。

## E009：新增呈現驗證使兩個既有 codegen fixture 失敗

- 現象：首輪 codegen 19/21 通過；未填 scale 的 art fixture 被拒絕，Saika 摘要 fixture 的 attack binding 引用未宣告來源。
- 原因：共用 schema 對缺少 scale 使用 0；舊摘要測試只測資料輸出，沒有完整可播放來源。
- 決定：native 邊界將未指定的 0 scale 視為 UE identity scale，負值／非有限值仍拒絕；補齊摘要 fixture 的 animation source，保留正式生成器的引用驗證。
- 驗證：修正 fixture 與 default 邊界後 codegen 22/22 通過，包含新 native 路徑／clip／非法引用測試。

## E010：apply_patch 區塊順序錯誤

- 現象：同一檔案先修改較後方 fixture，再尋找較前方測試，兩次 patch verification failed，沒有套用任何變更。
- 原因：此工具依序搜尋 patch 區塊，不能向前回找。
- 決定：同檔案區塊按來源行序排列；失敗後先檢查實際檔案，不重複送同一 patch。
- 驗證：重排區塊後成功套用，22 項 codegen 測試通過。

## E011：UE Automation 的數值比較多載衝突

- 現象：完整建置的 OmEditorAutomationTests.cpp 出現 C2665，`FVector.X` 是 double，預期值用了 float；native runtime／生成 C++ 已編譯，整體建置仍失敗。
- 原因：UE5 的 Large World Coordinates 將 FVector 分量改為 double，Automation 的 float／double overload 無法唯一選擇。
- 決定：FVector 比較使用 double 預期值，動畫 player 的 float 時間比較仍使用 float；不修改引擎或關閉測試。
- 驗證：修正後完整 Unreal 建置通過，Editor PID 60860，codegen 22/22 通過。

## E012：MCP 連線成功不代表 Editor 可操作

- 現象：Editor 重啟後 MCP health 通過，但 enable_extension 被 Restore Packages modal 阻擋。
- 原因：本專案 Editor 子程序不能優雅關閉，restart 在逾時後 force stop，舊 autosave 在啟動時觸發還原提示。
- 決定：先用 get_editor_dialog 讀取完整清單。本次只包含本輪 RecipeV1 的兩個 imported materials 與已保存、通過 SHA-256 驗收的生成材質，選 Skip Restore，保留 autosave，不覆寫保存資產。若出現其他來源，不套用此決定。網路 health 與實際 Editor 驗收分開記錄。
- 預防：重啟前保存 owned 新資產；重啟後第一步檢查 modal，再執行編輯命令。
- 驗證：本輪完整建置、Editor 測試與 22 套件配方重跑通過；未覆寫 autosave。

## E013：Editor package path 不能直接當 native soft object path

- 現象：Editor automation 4 項中既有 3 項通過，native 呈現失敗；模型存在但 LoadSynchronous 回報 missing，連帶動畫／visibility 斷言失敗。
- 原因：MCP 支援 `/Game/Folder/Asset` 簡寫，FSoftObjectPath 必須使用 `/Game/Folder/Asset.Asset` 才能解析 UObject。
- 決定：生成器對所有 native mesh／材質／動畫輸出完整 object path，MCP 配方仍保留 package path；增加完整 suffix 的 Rust 斷言，避免兩種引用混用。
- 驗證：完整建置通過；native 與既有 Blueprint／Saika 的 4 項 Editor automation 全部通過、零 error／warning；PIE 生成 transform 斷言通過。

## E014：材質 graph valid 不代表支援 SkeletalMesh

- 現象：Editor automation 通過，但檢查 PIE 日誌發現 `missing usage flag SkeletalMesh`，遊戲使用 Default Material。
- 原因：create_material_from_textures 未啟用 bUsedWithSkeletalMesh；原先只檢查 graph 與引用，漏掉 runtime usage。
- 決定：將 skeletal usage 納入配方操作 signature，owned 材質更新旗標後保存；每次驗證都讀回旗標，再驗 shader／引用。不能只以 asset 存在或 graph valid 宣稱畫面正確。
- 驗證：更新 1 個 owned 材質、略過 9 項；Editor 配方復驗 10 jobs、22 套件 hash 不變；PIE 無 missing SkeletalMesh usage 警告。

## E015：沿用 Fyrox 軸向修正，未驗證 FBX importer 已轉軸

- 現象：native 屬性／播放測試通過，PIE 截圖仍看不到正常站立的角色；最初測試位置也離相機中心太遠，不能用該圖作畫面完成證據。
- 證據：透過 GetImportedBounds 讀回模型 Z extent 約 65.85 cm，來源已由 UE importer 轉為 Z-up；原 render.pitch=-90 是 Fyrox 的原始來源修正，不應再次套用。
- 決定：native pitch／roll 預設 0，保留 scale／yaw／offset 的資料來源；新增 `ue.native_visual` 的 scale／pitch_deg／yaw_deg／roll_deg／z_offset_cm 供 Lua 美術配置明確覆寫，不改後端或既有 Fyrox 軸向。PIE 截圖在相機附近生成角色，等待實際 render frame 後再取圖。
- 驗證：軸向／rest offset 修正後完整建置與 4 項 Editor 測試通過；PIE 真實 render 與截圖通過。模型先前不可見也受 E017 的相機視點影響，不能把軸向當作唯一根因。

## E016：重建執行中讀取了舊啟動／共用 UBT 日誌

- 現象：讀到舊的 start JSON 誤以為新 Editor 已啟動，MCP 連線兩次失敗；共用 UBT Log.txt 同時包含另一個專案。
- 原因：完整 Lua 建置仍在停止／staging 階段，start 檔案尚未更新；多專案共用引擎的 UBT 日誌不能當成本專案的唯一證據。
- 決定：以當次建置 process session exit code 與輸出為準，完成後才呼叫 MCP；舊 start 檔與共用 UBT log 只作輔助，不用來判定當次成功。
- 驗證：以 session 7541 exit 0 確認完整建置完成，再執行 4 項 Editor 與 PIE 驗收。

## 呈現插值補強

Code review 發現原插值固定回到 relative zero，會抹掉 `ue.native_visual.z_offset_cm`。已改為記住首個權威 frame 前的美術 rest offset，插值與重置回到該 offset；缺省 offset 0 的舊 fallback 保持原行為。此項屬預防性修正，並非已觀察的使用者資料錯誤。

## E017：Actor 在原點不代表位於玩家視野

- 現象：角色 scale／引用通過，但截圖沒有模型，WasRecentlyRendered=False。
- 原因：舊場景初始鏡頭距離 90000 cm；游標邊緣移動後實際 GetPlayerViewPoint 在約 (-25985,-25985)。相機 pawn 舊快照不是即時玩家視點。
- 決定：只對工具自行啟動的 PIE 設定 UI pan guard 與 700 cm zoom，等待後讀取真實視點／旋轉，將視線與測試高度平面相交定位角色。使用 WasRecentlyRendered 作為獨立必要斷言；已有使用者 PIE 不改鏡頭，也不宣稱畫面驗收。
- 驗證：新 fresh PIE 全步驟 PASS、native_mesh_rendered=true，截圖可見帶材質模型；自行啟動的 PIE 已停止。SetFocusLocation 的兩種參數形式未達到定位目的，不依賴其回傳 success 當效果證據。

## E018：診斷入口與 Windows 搜尋路徑不可猜測

- 現象：誤呼叫不存在的 get_tool_info；rg 對 client*／ipc* 路徑收到 Windows error 123；讀取不存在的 presentation_ipc.rs／om_restart.lua。
- 決定：MCP 用已存在的 get_tool_docs(action=...)；先 rg --files 取得實際檔案，再以 rg pattern 目錄 或 -g glob 搜尋；restart 入口是 Rust executable，不虛構 Lua wrapper。
- 驗證：正確工具文件成功取得；實際 IPC adapter 位於 bridge/src/driver.rs，WorldBridgeActor 位於 OmGenerated 而非 OmRuntime。後续搜尋一律先取得實際路徑。

## E019：BlueprintSurface 測試第二次執行撞名導致 Editor 崩潰

- 現象：首輪 4/4 通過，重跑 BlueprintSurface 在 Kismet2.cpp:441 assertion，Editor shutdown，MCP 測試報告正確記為失敗。
- 原因：CreateBlueprint 固定使用 transient 名稱；同一 Editor session 上次暫存 Blueprint 尚存在，不能建立同名 UObject。
- 決定：英雄與 buff 測試均用 MakeUniqueObjectName，不刪除已有 UObject／使用者資產；不得僅重啟掩蓋不可重跑問題。
- 驗證：完整建置 session 13474 exit 0；同一 Editor PID 57700 連跑兩次均 4/4 通過。驗收工具固定兩輪，避免只以 fresh session 掩蓋殘留狀態。

## 建置診斷基線（非阻斷）

當次 full build 成功，但預編譯 BpGeneratorUltimate 的 extension modules 有 plugin dependency 宣告警告；Rust td_rounds 有既有 dead-code 警告，cbindgen 略過非 pub 常數。這些不是本轮 native 呈現失敗，也不能宣稱「整體建置零警告」。不修改共享引擎或憑猜測調整預編譯插件相依。

## E020：Hide 後 Forget 仍要求即時 entity 存在

- 現象：程式檢查發現 Hide 先移除 world entity，後續 Forget 呼叫同一 remove_with_epoch，會回傳 UnknownEntity。
- 原因：忘記安全記憶與移除即時 entity 被混為同一操作。
- 決定：Forget 若 live entity 存在仍嚴格檢查 epoch；否則只接受相同 (replica_id, disclosure_epoch) 的記憶。舊 epoch 不得刪除新 entity／新記憶；未知 ID 仍拒絕。
- 驗證：core 294/294，包含 Hide→兩次 extraction→Forget、同 frame Hide→Forget、新世代防舊 Forget、Reveal／verified rebase 清除記憶。
- 首次測試仍失敗：apply 路徑修正後，preflight_entity_references 仍只認 live entity，在套用前先拒絕。已同步修正 preflight 的暫存記憶集合，包含同 frame Hide→Forget；預檢与執行必須使用一致的身分規則，不能只修正最末端操作。

## E021：重連 snapshot 遺漏已保存的安全記憶

- 現象：runtime 保存 remembered_presentations，但 snapshot_envelope 固定輸出 remembered_ghosts=[]；最新狀態不足以恢復記憶。
- 決定：FilteredRenderSnapshot 攜帶凍結的 server-sanitized 記憶，以獨立 ghost 欄位投影至 IPC；不放入 live entities，不從隱藏 world 推算新位置。空 payload 不建立 ghost；生命周期 edges 仍走既有獨立通道。
- 驗證：client runtime 30+2 通過，真實 TCP 斷線後兩次取得同一最新凍結 snapshot；bridge 28+2 通過（1 外部 KCP ignored）。UE ghost 呈現尚未驗收。

## E022：共用 RendererReady 更新後漏了舊 Fyrox 呼叫端

- 現象：相容性 cargo check 發現 omfx 的 RendererReady 缺 player_id／team_id，且仍使用 protocol 2。
- 原因：先前 IPC v3 只修正 Unreal renderer；共用 proto 的所有消費端需要一起驗證。
- 決定：保留 Fyrox 相容路徑，將既有明確設定的 local_player_id／local_team_id 傳入握手，版本升 3；不猜測身分，不取消 runtime 身分檢查。新增共用 snapshot 欄位也同步更新其 adapter 與 fixtures。
- 驗證：omfx cargo check --tests 通過，沒有取消 IPC 身分檢查。

E018 本輪再次出現一次猜測來源名稱（team_replica_specs.rs）造成搜尋 error 2；後續以目錄搜尋確認實際為 filtered_specs.rs。這項操作習慣仍需持續遵守「先列實際路徑」，不可把曾取得正確檔案等同永不重犯。

## E023：新 frame sequence 不代表新的投射物事件

- 現象：程式檢查發現 DispatchFxCues 對每個 frame 的 retained tower_fire_fx 都呼叫 OnProjectileCue；最外層只檢查 frame sequence，無法阻擋同一 cue 重播。
- 原因：保留事件供較低頻 renderer 取得，卻把 frame 的新 sequence 當事件身分。
- 決定：在真正的 UE dispatch 路徑使用 (kind, source ID, source generation, instance ID, cue ID, event tick) 去重，不使用 renderer frame sequence。找不到 source actor 不消費；只有 projectile branch 消費其事件，未知 FX kind 保持原行為。
- 預防：加入新 frame／相同 cue、同 tick／不同 instance、新世代、零 instance、容量上限、過期重播及未來事件測試；synthetic frame 驗證實際 dispatch 次數。
- 邊界：4096 tick 歷史、最多 16384 keys；超量時 fail closed 並提供診斷，不能逐出當前 key 後重播。只在明確 StopRuntime（結束該 runtime）重設，Hide／actor 回收不重設。新的 WorldBridge instance 重連不重播的跨 instance 契約仍需 runtime cue 消費機制；本輪不宣稱已完成。
- 驗證：完整 Unreal build session 22346 exit 0、Editor PID 72232；同一 Editor 兩輪皆 6/6 passed（包含真實 synthetic dispatch），零 failed／skipped／not_run；PIE 模型與 fallback smoke 通過並停止自行啟動的 PIE。
