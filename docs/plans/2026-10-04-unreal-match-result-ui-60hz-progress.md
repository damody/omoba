# 原生終局結算面板：60Hz 增量驗證

## 計畫與決策

1. 檢查玩家端缺口：QWER已有共用binding，終局卻只有HUD小字，因此本段先補6.2原生結算面板，而非重寫既有控制。
2. 同一native command bar增加中央480 layout-unit面板，顯示Victory／Defeat／Draw、權威winner team與elapsed；不新增角色C++／Blueprint graph。Lua內容與C ABI不變；排障另補正常指定攻擊候選與additive IPC target intent，非調整玩法數值。
3. 明確隊伍來自runtime成功Start時保存的Config.team_id，沿既有IPC身分檢查。新增共用UE payload LocalTeamId／LocalPlayerId；不以player ID推導team，也不重新讀可變env。無runtime或team未知為0，不能猜結果；兩隊契約只接受team1／2、winner0／1／2。
4. 只在正式MobaHud／Finished／合法時間與tick顯示；死亡玩家同樣顯示。TD／非終局／錯資料清空，StopRuntime／EndPlay發空HUD清除。renderer斷線不擅自推翻已收到的終局。
5. 面板HitTestInvisible純呈現；玩法終局凍結仍由Rust權威負責。不提供尚無後端契約的重新開局／離開／自算計分按鈕。
6. opt-in正式驗收要求實際viewport與非零cached geometry，authority lifecycle死亡重生後相同winner／正確outcome，UI出現後至少120 ticks的三方hash；保存每隊含UI的game viewport PNG並檢視，不以visibility log冒充像素。

## 建置與回歸

- Lua result observer14 assertions（含JSON round-trip、player7/team2、拒絕wrong team／winner／outcome／過早tick／矛盾後再正確）；movement6／ability9／parity3／lifecycle8通過。ability新增reject→新ID重試，禁止引用舊ID cooldown；minimap14、shop12與shop button12及JSON亦於本輪先前通過。
- 首次build55721錯用OmFrameHeader.team_id而失敗；修正後26264通過。截圖版86320建置通過，但Editor21832因新fixture漏Initialize→TakeWidget崩潰；MCP curl56只是症狀。錯誤、call stack與防重犯記於E100，不刪斷言／不忽略崩潰。
- 最終完整build86655 exit0（UBT12.91秒），content identity de9c7fcfc98d6479，bridge stage SHA 8dc4c15c74c413f41dab8fe20902a9828b0e37633b02f35f2c01f280da0599f6。先前策略build94954因Owner撞AActor成員失敗，改ObjectiveHero；不關警告。不同版本先前成功的build不替代最終版驗證。
- 最終Editor57396同session兩輪各13/13、failed0／skipped0。NativeMatchResult驗實際panel visibility、state-before-rebuild、player7/team2勝敗平手、死亡、錯phase／team／winner／NaN／負值、reset／TD；MatchSmokeObjective驗雙隊反向tower／base、snapshot順序／死亡／hero／NaN／無效身分／無frame與路過小兵，真正smoke使用同一selector。既有Preview無context DestroyActor warnings仍在，不宣稱全專案零warning。
- 最終串行PIE34783 exit0，正常Save／QUIT Editor57396且查無。最新core323、base_content73、runtime50 passed／3 opt-in ignored、bridge49 passed／1 capture ignored，加integration2 passed／1外部KCP ignored；後端完整cargo test exit0，unit152 passed／1 ignored、其餘integration／doc條件測試如實保留ignored。本輪另執行舊failed run的capture（15212 snapshots）逐筆HUD／公開hero射程檢查通過；它不是舊run已終局成功。

## 正式60Hz驗收

### 最後版通過：run1791065966

- Rust release重新建置、原240秒、60Hz，launcher success=true／cleanup_verified=true；四技能各4/4、own-only位移／HUD／economy、死亡重生5／6次。原指定攻擊input7／8分別tick3250／3567得到status0，不以local enqueue冒充攻擊成功。
- 實際基地摧毀winner1，Finished9165，兩隊viewport UI9166分別Victory／Defeat；不是固定勝者（前两次成功run winner2）。中央權威elapsed152.7秒；頂端bootstrap60Hz與tick/60=152.8秒已從真實1280x720 PNG直接檢視，差異來自Playing時間與總tick基準，不用錯誤120Hz換算。晚連線握手保留修正已生效。
- 保存verifier獨立exit0，證據evidence/unreal-match-lifecycle/interactive-ue-1791065966.json。三方pre/post／frame hash每隊156 PASS rows／78 unique ticks至9360，0 FAIL，UI後194ticks；UNVERIFIED18748／18739不算PASS。不以snapshot接收Observed或一張FPS60證明持續GPU60FPS。
- PNG1 394259 bytes／SHA0d3350bc359aac6240979f593d19daecb329629a748a9673b9fb38891594d5d2；PNG2 588362 bytes／SHAb47ea6ef121d2511603786a63d7e21ddbd00a27b84b54bf1218c1d2b90c3972d。五owned PID28712／92908／87600／88404／84972均CIM查無；不殺其他程序。
- 最後stage03aa0c4d804ad706ecb2815366a817566776fdac2869df01c4d9ac9bea729098，runtime52+3／bridge50+2；完整build14047、同Editor91056兩輪13/13、串行PIE95327含截圖exit0。下列為保留的歷史排障，不代表最後版仍pending。
- 最後實際capture逐筆回歸：team1 9451／team2 9429 snapshots，owner HUD／公開live hero射程檢查各exit0；最終verify-staged-only相同SHA，scoped diff --check無錯。Rust既有dead_code與BpGeneratorUltimate dependency warnings仍如實保留，未宣稱全專案零warning。

### 歷史失敗與中間版

- 首次run1791059641在原240秒門檻失敗：雙隊四槽4/4、死亡／重生3／4、无Finished，因此沒有終局UI或截圖，success=false／cleanup=true，五PID查無。保存verifier如預期拒絕該run，沒有成功證據。
- 第二次1791060294亦240秒逾時（各死亡／重生4／3）；第三次1791060700仍逾時（14／0）。三次都四槽4/4，但沒有Finished／UI，不列成功證據。第二次基地未進視野，第三次進塔區欠缺持續tower AttackTarget；零散補基地approach不足。
- 最終修正：opt-in雙玩家共用單一可見目標流程，runtime明確team1取對方tower1700／base2400、team2取tower700／base0，unknown team拒絕。射程外正式Move進public attack_range的75%距離，90%內正式unit spell／AttackTarget；移除互相覆寫的舊分支。無可見objective每次正常Move到公開endpoint2400／0，不因施法中斷或Hide留在原地。selector僅讀安全frame，不補hidden目標，所有命令仍權威判定；兩隊都須原attack ID的status0。這是結算fixture輸入策略，不是正式五位置Bot／導航；仍使用原240秒、60Hz、真實基地摧毀與各自死亡重生，不改HP／數值／勝者。
- 第四次1791061680逾時：p1四槽4/4／死亡重生10，p2 slot2被status4拒絕僅3/4／死亡重生1。具體根因是正式IPC converter漏填attack_range（Default=0），先前只讀一般projection與native extractor沒有確認真正converter。补解碼已公開TAttack.range.v，缺失／壞資料0；它是公開current range，不是含未公開Buff modifiers的完整有效射程，攻擊合法性仍由權威決定。另一個fixture錯誤是被拒絕技能ID永久保留，改等待新可見目標後以新正常命令重試，保留原拒絕log，不把拒絕當PASS。
- 第五1791062140／第六1791062840同樣240秒逾時，不算成功。正常hero_tick指定目標可能被nearest10近鄰截斷，補live／current position／射程內explicit candidate，仍過原敵友／HP gates。headless新路線fixture先正常移動覆蓋死亡重生、helper target admission加入既有team visibility；不删斷言／不加repair。完整73項重新通過。selector限此fixture的公開塔1200／基地1600 max_hp，拒絕座標碰巧重合的小兵，不宣稱正式Bot結構識別。
- 第六輪追到真正IPC缺口：原proto／bridge無AttackTarget intent，Submit成功僅local enqueue，最後driver拒絕。新增tag17 AttackTargetIntent（render ID／queued），runtime只接受bound owner／view epoch／disclosed secure reference，沿既有KCP target驗證。未知舊intent fail closed、不overload TowerAction／不用authority裸ID。新增轉換與owner／未知目標／0／overflow／queued回歸；fallback與本機protoc輸出逐行一致。驗收加入原attack ID的status0 authority ACK，observer5 scenarios，enqueue本身不能當普攻成功。
- 第七1791063365亦240秒逾時，已取得status0、塔摧毀與基地扣血，但重生後base被Hide，fallback停1650导致前線反覆死亡（p1死亡15／重生14，p2 6／5）。改沿公開endpoint2400繼續正常移動，不把Hide當摧毀；可見objective仍唯一接管。
- 第八1791063849亦逾時；攻方接近基地但守方AttackMove長陷小兵中線，只有單方有完整objective流程。第九版採上述同一雙隊流程，不退讓／預設勝者；維持240秒。
- 第九run1791064374：release60Hz、原240秒期限內success=true／cleanup=true；四槽4/4、own-only位移與HUD／economy、雙隊死亡／新身分重生7／5、原objective attack ID7／9的status0（ticks3741／4446）。真實基地摧毀winner2，Finished9993（166.5秒），兩隊actual viewport UI9994分別Defeat／Victory。PNG各1280x720已直接檢視，含UI且標題可讀；保存verifier獨立exit0，證據evidence/unreal-match-lifecycle/interactive-ue-1791064374.json。
- 第九run逐筆raw hash每隊209 PASS rows／105 unique ticks至12600、0 FAIL；UNVERIFIED25240／25228不算PASS。UI後超過120 ticks gate通過；五owned PID78288／59860／80552／21840／82484查無。最初PNG verifier錯用文字read造成Windows CRLF轉換，改binary read後重驗，不改真實圖片或忽略signature。
- 截圖暴露另一diagnostics缺口：頂端假120Hz使tick9994換算83.3秒，結算權威elapsed166.5秒本来正確。新增RuntimeReady additive tag6 authority tick_rate_hz，bridge驗明確player/team，無效metadata未知0；頂端Observed取代Sim、刪假lag=0t。runtime51+3／bridge50+2通過；build56090、UBT7.77秒、stage eef2d3aa3794cd9fbdb083732cbea9f27481f887547162ea361199051cb68dad，Editor86808兩輪13/13。首次PIE截圖因Editor最小化失敗，MCP精確restore後完整串行PIE含截圖exit0；正常Save／QUIT且PID查無。新版正式run重建Rust release中，尚未使用舊版success冒充。

## 重現與剩餘

診斷修正中間版run1791065418：對局／UI本身通過（winner2、Finished13503、225.0秒、UI13504；死亡11／10、重生10／10；原attack ID8各status0 tick3919；hash各224 PASS rows至13680），cleanup_verified=true且五PID76840／73716／83052／62200／60368查無，保存verifierexit0。實際PNG卻仍?Hz／Time--，表示舊latest watch覆蓋ready；本run不算diagnostics完成。獨立保留握手metadata後實際TCP晚連線／重連／wrong-team回歸通過，runtime52+3、bridge50+2；最後build14047 exit0、UBT2.25秒、stage03aa0c4d804ad706ecb2815366a817566776fdac2869df01c4d9ac9bea729098，Editor91056同session兩輪13/13、PIE95327含截圖exit0、Save／QUIT且PID查無；最新正式對局驗證中。

固定Lua按順序build_ue_moba.lua --full → ue_native_visual_smoke.lua（兩輪完成）→ ue_pie_smoke.lua，再正常Save／QUIT Editor。設定OMOBA_UE_RESULT_UI_SMOKE=1、OMOBA_UE_ABILITY_SMOKE=1、OMOBA_UE_MATCH_SMOKE=1、OMOBA_UE_SMOKE_SECONDS=240、OMOBA_UE_STEP_FPS=60、OMOBA_RELEASE=1、OMOBA_SKIP_UE_BUILD=1，執行tools/lua/lua.exe scripts/run_2player_ue.lua --single-lane。此次SKIP_BUILD僅適用已建置同Rust版本；修改Rust需重建後端。

本段只補6.2終局UI，總數仍17/30，不勾選完整6.2。選角／計分板／真人QWER／完整hit-test／LAN、五位置Bot／三英雄／三路與正式地形、全frame-time效能仍未完成。API smoke不是實體按鍵，不把network60Hz等同每frame達60FPS。
