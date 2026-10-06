# MOBA 完成執行清單（2026-10-06）

## 最新執行狀態

使用者要求暫停，已停止v5與Grok lane-review。原workflow cleanup_verified=true，selection→同世界gameplay與真人商店／移動／Q升級已確認，但自然結果尚未出現，不勾6.2。本輪實際8場／10場；恢復先定位真人R execute RErr（已加詳細log，headless局部1通過），再續尚缺項。錯誤／暫停原因詳E348，OpenSpec仍27/31，不改效能FAIL與真LAN缺證據。

後續第7場取得具體差異且安全清理完成：Bot零關聯購物回執誤觸MalformedDisclosedState，整隊漏frame／生命週期後留下對手幽靈。已修為先consume＋verify FIFO，再省略無UI關聯回執，真人codec／交易／accepted inputs不變；7項局部回歸通過，server41.68s、runtime28.75s release成功。第8場同PIE自然結算啟動，最多10場，不先勾6.2。詳E347與原v4證據。

同世界 PIE 原生 handoff 六檔已編譯（37.01s），真實選角接入、10人計分板、技能／商店／背包／小地圖顯示已有截圖。自然結算仍未完成：v2／v3皆為 Team 2 checkpoint不一致後安全終止，v3首錯 tick15240，Team 1正常；兩場均原owned cleanup確認。當前6場／最多10，不重跑無改碼效能場次。正式模式 evidence-dir 修正由Grok提案、primary整合，4項局部測試通過；迷霧全域ordinal修正11項通過，未冒称 observer 根因。接著使用新server mismatch component對照定位，再修通用狀態同步問題。OpenSpec仍27/31，固定12效能峰值未過，真雙機LAN尚無第二台機器。

本文件是當前執行索引；歷史增量文件保留，不依歷史「當時尚缺」重做已有功能。OpenSpec `build-unreal-rust-moba-framework` 目前27/31。正式60Hz、omfue唯一前端、Lua只在工具／建置生成使用、不新增角色C++或Blueprint graph、最多10場模擬、不拆批規避。

## 計畫

1. Grok有界實作原生workflow supervision。現有合作式cancel只處理正常Lua執行，不能替代強制終止／crash時的子程序監督。使用private Windows Job Object、suspended-before-assignment、原始owner handle，不用name kill／PID-only清理；主agent整合正式CLI並做無遊戲的故障fixture。
2. 核對已有4.1／4.3／4.4與6.2接線，不新增不必要Bot政策、cue類型、導航或防禦性支線。真正缺程式才補；feature check與完整驗收分開。
3. 功能整合後只做一輪正式release／compiled-only建置與最後驗收；已有headless／native呈現／美術替換成功證據不無條件重跑。選角必須有真人明確操作或具名opt-in測試，不能造consent；自然終局不能強制winner。
4. 保存60Hz選角→遊戲→自然結果、IPC／視野／重連／單一模擬與固定十二項效能門檻的精確證據。缺證據不勾整項，效能不因失敗放寬門檻或反覆挑樣本。
5. 兩台LAN需要真正第二台主機／網路執行環境。本任務目前工具只提供Local，兩本機程序不能代替；先完成其餘可行工作，再明確記錄該環境限制。

## 當前狀態

| 項目 | 狀態／所需證據 |
|---|---|
| 原生workflow監督 | Grok601秒無補丁取消且原process確認退出；primary通用native實作，host9／真實無遊戲fixture5通過，正式入口整合；見E340 |
| 4.1 IPC／輸入／視野 | 已封關；當前27/27合併與既有實際UE輸入／shop／視野證據，見ipc-reconnect-contract-closure |
| 4.3 lifecycle／六類cue／重連 | 已封關；真TCP六類歷史去重、已有native42兩輪與真實renderer替換／新input／hash／cleanup互補證據 |
| 4.4 單一模擬 | presentation-only mode與拒絕fallback已有；單機／LAN完整證據仍分開 |
| 6.2 完整UI | 個別UI與選角已有，尚缺同一正式流程到自然結算完整證據 |
| 6.4 真LAN | 第二台實機環境尚不可用，不以loopback冒充 |
| 6.5 效能 | 四段量測與十二固定門檻已有；最新局部client窗口低於門檻不是整項通過 |

完整建置v1已成功（frontend13.79秒Unreal動作、bridge/base stage SHA一致；release各元件成功），實跑因正常close的原handle/query競態中止，cleanup_verified=true、success=false保留。E341通用修正及3/3局部回歸後，v2沿原十二門檻、明確reuse-built驗證中，不重建／不放寬／不挑樣本；本任務至此場次2。6.5仍未勾。

工具失誤與實際功能錯誤集中寫 `unreal-moba-error-register.md`，不以「Grok未回覆」冒稱已實作；委派不轉移整合責任。未授權commit／push、刪除或終止無關程序。
