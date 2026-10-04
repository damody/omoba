# Unreal 技能升級綁定：60Hz 驗收

## 計畫與決定

- 續做OpenSpec build-unreal-rust-moba-framework的5.3／6.2增量，目前19/30，未完成全項不勾選。
- Q/W/E/R維持普通施法，Ctrl+Q/W/E/R改明確FInputChord綁定獨立升級回呼；通用反射跨module，不新增英雄C++或Blueprint graph。
- 非Shipping明確opt-in執行唯一Ctrl chord delegate，核對一次回呼與queued，再走正式IPC／KCP；這不是OS鍵盤注入或完整hit-test驗收。
- 等正式兵線XP取得SP才操作，不直接注入點數／rank；權威回傳rank與SP、原ID applied、實際native文字與截圖、操作後120tick三方hash共同驗收。
- 發現問題先保存證據、修正原因，不增加timeout或換ID重送掩蓋失敗；錯誤見E120。

## 首輪資料與操作驗收

interactive-ue-1791094787：release scripts／server／runtime同profile重建，不使用runtime內部upgrade／combat／roster injection。兩隊Ctrl+Q各恰好一次callback與原input2 forwarding／authority acceptance；rank1→2、SP3→2、hero level4保持，前後XP61→86來自正式兵線結算。兩隊原ID result tick2982，UI完成tick3019／3017。

- launcher及獨立saved verifier通過，原protobuf核對live snapshots3207／3199（共6406），rank／SP／level／XP逐筆對原wire精確一致，before／after兩個UI tick亦精確匹配。
- 三方各54 PASS rows至3240，超過完成tick120以上；五owned PID44048／73284／33980／49532／83644由verifier另外inspect退出。
- 但實際PNG視覺QA發現72×72技能格裁掉提示，不能算UI完成；已改128×128、名字與rank-CD分行、Ctrl+實際鍵提示，第二輪實跑待核對。
- 完整UE建置與11BP MCP compile1791094597／1791095203通過，最新Editor94760兩輪18/18原生測試與bridge52／runtime60（7 ignored）通過。忽略的raw capture測試另由saved verifier明確執行，不當作一般測試已跑。
- 最新Editor串行PIE也通過並stop；以固定Lua host close_window／有界wait／必要fallback stop與inspect確認94760退出，再啟動第二輪，不在遊戲client執行時全域停止Editor。

## 第二輪排版檢查

interactive-ue-1791095365：兩隊input2各一次，隊1rank1→2／SP2→1（level3），隊2rank1→2／SP3→2（level4）。原始live snapshots2944／2936（共5880）與UI精確核對通過，launcher及saved verifier cleanup亦通過；兩隊升級提示可見，但名稱在更新後仍有裁短，故不封關完整排版。

三方各50 PASS rows至3000，超過兩隊完成tick2422／2844至少120ticks；五PID92080／42584／50288／13104／95672由saved verifier另驗退出。

判斷AutoWrap的動態desired width可能不穩定，改AbilityTexts的MinDesiredWidth／WrapTextAt為格內116 layout units並關閉AutoWrap；第三輪待實際PNG核對。deprecated r.Mobile.VirtualTextures欄位已移除，第二輪雙UE日誌沒有該ensure；桌面r.VirtualTextures=True保持。

## 第三輪與後續修正

interactive-ue-1791095707：raw live snapshots2954／2948（共5902）、兩隊各50 PASS至3000、獨立saved verifier與cleanup通過。隊1PNG仍有部分名稱裁短，隊2完整，所以撤回「固定wrap足以解決」判斷，不宣稱排版完成。

SetAbilitySlot改相同FText不反覆SetText，保留tooltip更新；after截圖等待三個rank2 presentation frames再擷取。這不改authority或timeout，也不是GPU fence／像素同tick原子契約。第四輪待畫面核對；最新完整UE建置與11BP MCP compile1791095969通過。

## 第四、五輪：保留失敗反例

interactive-ue-1791096121：live snapshots3174／3167（共6341），三方隊1各53／隊2各54 PASS至3240，兩隊rank1→2／SP3→2、level4／XP86保持，before3008／after3039。獨立saved verifier／cleanup成功，但四PNG仍見部分名稱裁切，不能說相同Text跳過與延後frame已解決。

interactive-ue-1791096619：名稱／rank／提示已拆成三個實際Slate block，原生兩輪18/18（含三block更新與清除hint回歸）、PIE、11BP compile1791096415通過。live snapshots3318／3312（共6630），隊1各56／隊2各55 PASS至3360，saved verifier／cleanup成功。隊1level3／XP105，SP2→1，tick2397→2424；隊2level4／XP86，SP3→2，tick3114→3140，兩队皆input2一次／rank1→2。

四張PNG逐張人工查看：after兩隊完整，但before名字仍裁短。故再改ASCII content ID／rank-CD／Ctrl提示為SimpleTextMode單行layout，不用shaping與wrap cache；不把局部畫面成功當全局修好，不宣稱已確認引擎根因。最新full UE／11BP compile1791096781通過，仍待第六輪前後PNG驗收。

## 第六輪：改用可驗證的布局診斷

interactive-ue-1791096909：raw live snapshots3053／3047（共6100），三方兩隊各52 PASS至3120；獨立saved verifier／cleanup成功。隊1level3／XP105、SP2→1、tick2406→2434；隊2level4／XP36、SP3→2、tick2899→2925；rank1→2與原input2各一次。四張PNG逐張看過，before名稱與隊1after的Q名稱仍裁短，因此SimpleTextMode不算完整修正。

新增opt-in的before／after每槽actual name、cached geometry與desired size診斷；僅四個技能名稱設ForceVolatile以免復用cached paint，是常數四短字塊、不是per-entity熱路徑。這是待验证呈現修正，不修改authority／timeout；第七輪須核對數據與四PNG。最新runtime60／bridge52重跑通過，ignored raw test由saved verifier明確執行。

## 第七輪：文字／尺寸已排除

interactive-ue-1791097384：raw live snapshots3293／3287（共6580），三方兩隊各55 PASS至3360；saved verifier／cleanup成功。兩隊level4／XP86、rank1→2／SP3→2，before3142／after3176與3175，原input2一次。before／after每隊八筆layout actual name完整、cached geometry及desired皆116×18，saved verifier新增明確核對每槽尺寸足夠、名稱保持與數量去重。這不是像素證據；四PNG仍見裁短，因此移除無效ForceVolatile，改名稱自己的ClipToBoundsWithoutIntersecting clip zone，不全局關閉HUD clipping；第八輪待驗。

最新11BP compile1791097297、兩輪18/18原生測試、PIE通過，owned Editor38804有界停止／wait／inspect確認退出，未使用舊binary。

## 最後60Hz資料驗收與畫面未完成項

interactive-ue-1791097735：3086／3079 live snapshots（共6165）逐筆精確匹配原protobuf rank／SP／level／XP，兩隊各52 PASS checkpoint至3120，超過最後UI記錄2980至少120tick。隊1rank1→2／SP3→2／level4／XP11，before2720／after2767；隊2rank1→2／SP3→2／level4／XP61，before2947／after2980。兩隊原input2只有一次Ctrl callback／forwarding／acceptance，正式兵線XP取得SP，沒有runtime升級注入。

saved verifier success／layout_verified／cleanup_verified=true；五owned PID43712／23392／38268／89052／85684另由process.inspect驗退出。最新full UE建置／11BP compile1791097628、兩輪18/18原生測試、PIE通過，Editor98760亦退出。橋接stage仍6330b7204db53d856d3aff4c78ae7d3d1a71fb65d7856d2bfe10762001cd9ad2，content hash ac6ff592a15c8b5e。

四PNG逐張檢查：技能等級與Ctrl提示可讀，但名稱仍偶發裁短；自身clip zone不是解法，已撤掉並重建最終binary（此撤除沒有再跑第九輪PNG，不能算視覺封關）。保留通用128格／三文字block／實際text-layout診斷和證據工具。後續名稱問題先取得實際OnPaint clip／draw資料，不重試wrap／volatile／自身clip等已排除假設；60Hz操作與權威增量成功不等於完整UI／框架完成。

本輪OpenSpec skill維持完整任務未勾選，總19/30；它促使更新既有設計／任務與驗收邊界，而非另建不一致計畫。

撤除無效clip後最終`--build-only` exit0，OmGameEditor增量建置與stage SHA核對通過；不是新一輪PNG／MCP驗收。

## 重跑指令

完整建置：`tools/lua/lua.exe scripts/build_ue_moba.lua --full`。Editor原生與PIE串行驗收：`scripts/ue_native_visual_smoke.lua`／`scripts/ue_pie_smoke.lua`；由固定Lua runtime執行。關閉本輪owned Editor後，環境`OMOBA_UE_UPGRADE_SMOKE=1`／`OMOBA_UE_SMOKE_SECONDS=120`／`OMOBA_SKIP_UE_BUILD=1`執行`tools/lua/lua.exe scripts/run_2player_ue.lua --single-lane`，同profile Cargo建置仍執行。saved verifier：`tools/lua/lua.exe scripts/verify_ue_upgrade_run.lua RUN_ID`。

僅驗唯一正式Ctrl chord delegate，不冒充OS按鍵注入；PNG非同tickGPU fence，world截圖相機差異不作玩法證據。60Hz是authority simulation／replica契約，不保證所有截圖renderer始終60FPS。rank0首次學習、跨runtime升級journal、LAN／三路／選角、完整5.3／6.2等仍保留未完成；本輪無提交／push、未清理既有dirty工作。
