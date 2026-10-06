# 同世界 PIE 選角到自然結果：實作與根因修正

使用 change：build-unreal-rust-moba-framework（spec-driven）。維持 omfue 唯一前端、Lua只作工具／build-time生成、60Hz、單一外部Rust玩家模擬、不硬編碼英雄、不新增Blueprint graph。完整驗收留最後；當前以局部功能證據推進。

## 已實作

- Native selection widget保存嚴格Rust terminal receipt後，可用明確 opt-in `-om-selection-in-place` 保留原PIE；預設退出流程不變。
- PlayerController驗證saved receipt／player identity，原Subsystem只以presentation IPC啟動，移除選角widget、建立原生HUD，保持同一GameInstance與World；不改全域commandline、不開第二份模擬。
- 固定Lua入口 `scripts/run_moba_pie_ue.lua` 建立並綁定自己的Editor，等待MCPready才開始PIE，選角最終plan驗證後啟動唯一authority/runtime，呼叫原Controller handoff，等待native自然結算與原PIE截圖，最後清理原owned lifetimes。遇secure termination即保留原診斷，不強制winner。
- MCP可選owned-editor gate每請求確認原executable／creation token與endpoint，而非只憑同project名稱採用他人Editor。
- 正常模式顯式evidence-dir可保存filtered runtime證據，不啟用test-mode／fault。Observer首次hash不一致保存已公開component值／digest，可與原authority expected檔比較；私密sentinel檔不得印出／交给Grok。

## 問題與決定

1. 初次PIE已啟動server卻因本次owned ledger二次寫入拒絕中止；只允許更新本次自有ledger，並在server取得original身份時立即計入場次。
2. 混合HUD/攻擊/霧事件的 fog stable_sub_index是producer排序後的全域ordinal，不保證0；接收端修正，保留duplicate／subject／reserved／payload各項驗證，真正producer回歸11項通過。
3. 長場次Team2 mismatch不是迷霧原因。第7場首次component對照191對191，仅敵方Hero／TAttack兩個schema不同；原tick15132 projection錯誤後漏frame，兩observer MissingReplicaTick，rebase留幽靈。ShopReceipt是MalformedDisclosedState唯一來源；Bot正式input correlation=0，卻被UIreceipt非零ID規則拒絕。
4. 通用修正：無UI correlation的authority-local shop輸入仍先consume／verify settlement FIFO，才省略receipt；不偽造ID、不在pop前過濾、不更改交易／accepted input／真人codec。Scoped7項回歸通過，包括兩個連續真正projector frame。

## 當前證據與限制

- 使用者暫停：v5已停止，原workflow report cleanup_verified=true，三個原PID讀取核對均不存在。自然結算未完成，success=false；不是observer failure。保留selection／shop／move／upgrade／cast／截圖與capture，實際8場／最多10。
- 真人buy/sell已沿正式input處理，原HUD顯示Transaction #2 settled(code0)；attack-move tick77225落地、Q upgrade tick78165由L1變L2，離店與死亡重生已實際檢視。截圖pie-human-shop.png、pie-human-move-upgrade.png、pie-human-cast.png均在原v5目錄。圖中simulation60Hz而renderer3FPS，不能宣稱渲染60fps。
- 真人R tick83725 execute returned error，尚未定位。已增加原RErr內容log以便恢復時查因，四技能headless效果／結算局部1項通過，不等同實場修復。恢復先診斷，不強制winner、不重開大量採樣。
- Grok lane-review run-muwoiwru-cg0ms8/threadc3e0d4b6-def5-441c-986a-37ec99efceb7因使用者暫停取消，197s無輸出／無編碼，成本未知；前述Grok已接受提案與primary實作歸屬不變。

- Native六檔UBT成功37.01s；原MCP12＋4fixture；真正同世界selection→native IPC→10人HUD已有 `target/pie-moba-runs/final-same-world-20261006-v3/pie-gameplay.png`。
- Capture policy4項測試通過；shop7項測試通過。修正server41.68s／runtime28.75s release建置成功。
- v2／v3／v4長局失敗均保留success=false、owned cleanup=true；不計作6.2完成。v5為本輪第8場，修正版已越過第一次舊projection錯誤tick5436，但自然結果尚在進行；最多10場，不拆批规避。
- v5已越過原中斷tick15600，Team2 tick=sequence=15600，authority expected與observer pre/post相同；Team1 external runtime／observer／authority三方PASS。Team2無真人runtime，其三方verdict仍UNVERIFIED，不能把observer一致冒稱雙runtime驗收。沒有projection error／observer mismatch，等待自然結果。
- Grok capture proposal67s接受；Grok shop proposal277s邏輯與primary先行實作相符，其虛構五參數測試被拒絕；實際測試是primary四參數API版本。Job/thread與usage見.ai-collab/state.json、review.md。
- OpenSpec仍27/31；真雙機LAN缺第二台實機，固定十二效能峰值先前FAIL保留，不以同世界單人場次冒充LAN／全workload驗收，不放寬數值也不挑樣本。
