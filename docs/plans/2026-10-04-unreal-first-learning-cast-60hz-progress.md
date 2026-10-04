# Unreal 首次學習後施法：60Hz

## 計畫與決策

- [x] 保留原CtrlQ rank0與rank1升級模式，新增獨立`OMOBA_UE_FIRST_LEARN_CAST_SMOKE=1`。
- [x] 同一server-owned純Lua apprentice，CtrlR唯一正式delegate學rank0→1；普通R唯一正式delegate經正常IPC／KCP施法。其他runtime injection關閉，不改Lua英雄／技能數值。
- [x] 等正式玩法產生HP缺額再治療（戰鬥傷害或正常升級擴張maxHP皆合法，不注入）；成功ACK不是效果證明，另要求正冷卻／HP上升，原始rank／SP／final EquipmentStats逐snapshot與UI exact tick核對。
- [x] 增加缺段／重複／錯槽／錯ID／失敗status／不增HP／超maxHP／負CD觀測器測試；Rust runtime 61 passed、8 opt-in ignored。
- [x] Unreal full build、MCP既有11資產gate、同Editor76216原生19/19兩輪與串行PIE通過。
- [x] 真實雙Unreal60Hz release五程序、保存證據只讀verifier、學後施法+120tick unique三方hash、owned退出核對。

## 最終結果

`interactive-ue-1791105946` success／cleanup_verified true，saved verifier核對來源SHA前後不變。未重送或重演已完成交易。

| 證據 | Team 1 | Team 2 |
| --- | --- | --- |
| CtrlR原input | 2，一次 | 2，一次 |
| rank／SP | 0→1／3→2 | 0→1／4→3 |
| 正常R原input／authority tick | 3／2524 | 3／2838 |
| 權威HP原始Q10 | 563200→706560 | 563200→706560 |
| Lua rank1治療Q10 | 143360（140HP） | 143360（140HP） |
| UI施法前→後tick | 2522→2527 | 2836→2840 |
| 正CD seconds | 24.967 | 24.983 |
| exact wire→IPC快照 | 2946 | 2937 |
| unique三方checkpoint | 25至3000 | 25至3000 |
| post-cast unique ticks | 4 | 2 |

Raw verifier除了逐筆rank／SP／level／XP／HP／maxHP外，直接核對施法step前後權威HP：`after=min(before+compiled Lua heal, maxHP)`，此fixture沒有同tick併發傷害。不是只看callback或ACK。Team1正常XP在施法前擴HP上限654→706，不以舊maxHP推算治療截頂，也不注入缺血。

兩張after PNG已檢視，R的L1與未學QWE的L0可見；這些截圖是學習stage，並非學後治療畫面／GPU同tick fence。英雄與局部技能文字仍截短，E121不宣稱修復。

server99032／runtime5484與27908／Unreal62776與62088五PID另驗退出；測試Editor106104與76216也另驗退出。最後bridge與stage SHA一致：`0f471bc1984353f0c6b088a37101d4c236bcf2edc1a2609b19b2b680633fedae`，generated內容hash仍`3b296ff5bdbcd7d9`。原CtrlQ rank0與rank1保存verifier回歸通過，新舊Lua觀測器／runtime61與bridge52 tests通過，codegen11檔check通過。

## 問題與限制

第一輪`interactive-ue-1791105739`兩隊CtrlR input2成功、R input3正常轉送／runtime接受，但新harness在control-only結果frame缺HUD時提前返回，因此沒有result／complete並60秒逾時。已改先讀applied_inputs，再檢查HUD；server100412／runtime97092與96072／UE41032與58528皆獨立確認不存在，保留失敗captured資料，不改timeout。

本輪檢索誤猜`omfue/bridge/include/om_bridge.h`，實際header在OmRuntime Source/ThirdParty。已用rg從Source確認InputCastAbility=3／InputUpgradeAbility=9；不以猜測enum值代替共享header。

E121畫面截字仍開啟，不重試已排除的相同假說。本輪不是OS鍵盤注入、GPU同tick像素、穩定60FPS或完整框架完成。
