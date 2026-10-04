# 60Hz 原生商店按鈕到權威交易

## 計畫與本輪決定

沿用OpenSpec `build-unreal-rust-moba-framework` 4.1／6.2部分，先完成60Hz，不擴到120Hz。

1. 既有雙UE商店smoke走WorldBridge API，不代表Slate按鈕回呼已驗收。加入明確opt-in按鈕route，但不新增Rust交易入口。
2. 使用原本SButton的合成pointer按下／放開事件，觸發原本OnClicked→SubmitShopAction→generic HUD event→bridge→正式runtime／server。不得直接呼叫SubmitItemBuy／Sell充當按鈕驗收。
3. 只有非Shipping且`-om-shop-button-smoke`才允許合成事件。必須是viewport內已建立且enabled的按鈕、真實cached geometry；中心離開scroll範圍時先捲動並等新layout，不捏造幾何／強制enabled。layout未ready不提交；真正dispatch後若失敗則停止smoke，不分配新ID重試交易。
4. Scroll list使用PreciseTap處理觸控，不修改既有滑鼠DownAndUp行為。測試是合成touch pointer handler／回呼整合，不是OS／實體滑鼠／完整hit-test grid驗收。
5. 回呼次數與原request一一對應：exact三次buy1／buy1／sell0，每次pressed=1／released=1／callbacks=1／accepted=1，ID須等於既有queued階段0／2／4。只看到queued不算成交，仍須真實拒絕6／成交0／售出0、六格／Gold與pending0。
6. Lua launcher與保存證據verifier核對route及兩隊callback，不把歷史API run改稱按鈕run。原始protobuf checker維持逐snapshot owner／收入／裝備／immutable receipts核對。

## 已驗證：按鈕回呼到60Hz權威交易通過

- Lua交易observer12情境＋JSON round-trip；按鈕observer12個positive／negative情境＋JSON round-trip通過，涵蓋漏按下／放開／回呼重複／request錯配／錯隊／缺權威結果。
- 既有movement6／ability7／parity3／match8 observer regression通過。
- 完整build session45033 exit0：生成11檔、content identity hash `de9c7fcfc98d6479`不變；staged bridge SHA-256 `3924164b43a99b463eb2cdb06537a698605385adaf60c6240a0535bd4fdde720`一致。
- Editor86232同session兩輪各9/9，串行PIE26384 exit0且已停止。Editor保存後QUIT_EDITOR正常退出，確認沒有Editor才啟動五程序release驗收。
- 正式release run `interactive-ue-1791055726` exit0：network60Hz／presentation30Hz／UE60FPS，route=`slate_pointer`。兩隊各exact三次buy1／buy1／sell0，每次pressed／released／callbacks／accepted均1，request1／2／3與queued一一匹配，未fallback至WorldBridge API。
- 兩隊tick1864收到不足金錢拒絕；自然收入到350後tick10623買入，Gold0／slot0=1。team1 tick10627、team2 tick10628售出，Gold175／空六格／pending0，之後各以正式輸入完成移動並觀察own-only replica與Unreal呈現。
- launcher報告success／cleanup_verified=true，出售後三方hash176／177 PASS、最後10680。獨立保存證據verifier重新核對含清理前最後寫入的完整日誌，hash178／179 PASS、最後10800，沒有FAIL／repair；兩隊safe tick10821／10811，Consumed sequence716／937。兩組計數差異是停止前新增checkpoint，不改寫原始報告。
- 原始protobuf capture每筆owner／Gold／六格／三筆immutable receipts均通過：team1 5413、team2 5404 snapshots；最後Gold181為售出後繼續累積的正式收入，不是多退。
- 獨立verifier session49688 exit0，`target/interactive-runs/interactive-ue-1791055726/unreal-shop-verification-report.json` success／cleanup_verified=true、route=slate_pointer；保存原始來源SHA-256。原launcher報告`unreal-ipc-smoke-report.json`亦正常保存，沒有沿用先前JSON保存失敗的run作此次成功證據。
- Owned五PID65640／62640／11252／30232／42376均停止，CIM查無；stage-only再次核對SHA一致。Editor86232已正常退出，沒有殘留本輪遊戲程序。

## 重現

沿用Lua入口，環境設定如下（一般遊玩不設定smoke）：

| 變數 | 值 |
|---|---|
| OMOBA_RELEASE | 1 |
| OMOBA_UE_STEP_FPS | 60 |
| OMOBA_UE_SMOKE_SECONDS | 210 |
| OMOBA_UE_SHOP_SMOKE | 1 |
| OMOBA_UE_SHOP_BUTTON_SMOKE | 1 |

```bat
tools\lua\lua.exe scripts\run_2player_ue.lua --single-lane
tools\lua\lua.exe scripts\verify_ue_shop_run.lua interactive-ue-1791055726
tools\lua\lua.exe scripts\tests\ue_shop_observation_test.lua
```

跳過建置僅限已完成相同版本完整建置與stage核對；本輪沒有変更Rust交易規則，使用已建置的release server／runtime／base_content，不注入金錢。

完整框架與真人滑鼠、完整選角／小地圖／計分板、跨runtime pending持久化、硬即時效能基線仍待驗收；OpenSpec17/30不因商店局部通過而勾選完整4.1／6.2。錯誤／防誤判規則見E097。
