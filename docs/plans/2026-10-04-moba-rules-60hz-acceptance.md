# 第5.3項：MOBA數值／經濟規則60Hz驗收

## 範圍與完成判準

第5.3項要求經驗、擊殺／助攻、金錢、商店、六格裝備與回城，驗證合法／非法輸入及數值結算。逐項由相同Rust權威核心實作、Lua編譯規則、正式PlayerInput與安全投影完成；技能學習／升級也追加驗證。不要求此項涵蓋三路導航、所有動畫cue、選角至終局完整UI、兩台LAN、cross-runtime crash recovery或效能基線，這些仍由4.x／5.4／5.5／6.x獨立驗收。

不把舊進度文件中的「當時尚未完成」視為永久缺項，也不把不同日期的已保存對局當作本輪新對局。本輪重新執行目前程式的測試及已保存protobuf驗證，不重交易／修改原始captures。

| 要求 | 合法與非法核心驗證 | 正式60Hz證據與本輪重驗 |
| --- | --- | --- |
| XP與升級 | creep死亡去重、range／alive／phase、整數分享、多級／上限、成長不補血／裝備不重加 | 兵線`moba-runtime-1791083346`，本輪3,883 snapshots逐筆raw progression通過 |
| 擊殺／助攻 | 唯一生命結算、匿名死亡／NPC／friendly／零傷害拒絕、窗口／roster、死亡助手重生保存 | `moba-runtime-1791081719`，本輪3,923 snapshots逐筆KDA／Gold／XP、非零kill／assist通過 |
| 金錢／收入 | active固定時間、Warmup／pause／Finished凍結、死亡保存、飽和 | 正式商店對局收入支付350而非注入Gold；raw IPC核對被動收入−買入＋售出 |
| 商店 | phase／pause／owner／alive／基地距離／金錢／catalog／slot拒絕原子、不重付 | `moba-runtime-1791051305`保存success，wire同input五次重送只一次買賣；Unreal商店另重驗 |
| 六格與裝備 | 第七件拒絕、滿格合成、重複材料、差價／整數退款、overflow／negative拒絕、買用賣順序、死亡重生不double bonus | `interactive-ue-1791055726`原生按鈕→正式買／賣與receipt／inventory／Gold保存verifier本輪重跑 |
| 回城 | 8秒exact active時間、pause、不同行動／傷害／死亡取消、無免費heal、owner／team基地 | `interactive-ue-1791073998`正式B→Move取消→B完成，保存verifier本輪重跑；PID重用不停止無關程序 |
| 學習／技能升級 | rank0不能cast、level6門檻／SP／max-rank／非法原子拒絕、重生保存、rank傷害／CD | 本輪新雙UE`1791105946` CtrlR學習→R施法，5,883 snapshots與原input一次、exact Lua140HP治療通過；舊CtrlQ模式保存回歸通過 |

核心實作測試位於`scripts/base_content/src/single_lane_match_tests.rs`（正式ECS／SimulationDriver／script DLL、filtered replay）、`omoba-core/src/runtime/native/shop.rs`（原子交易）、`omoba-core/src/runtime/native/moba_assist.rs`（生命／窗口／roster）。本輪完整core334／base_content98／server155 passed（另1 benchmark ignored），runtime61／bridge52 passed（capture測試需明確opt-in，未冒充一般suite已執行）。base98包含完整60Hz filtered lifecycle，以及15Hz／120Hz相容回歸；使用者當前成功門檻仍60Hz。

本輪Unreal full編譯、11個必要BP的MCP compile、同Editor兩輪19/19、串行PIE、generated11檔／15Lua來源check通過，owned Editor與新對局五程序另驗退出。最後stage SHA `0f471bc1984353f0c6b088a37101d4c236bcf2edc1a2609b19b2b680633fedae`。

## 明確不算完成的內容

- E121局部畫面文字截短仍未修復，截圖不是GPU同tick fence或穩定60FPS證明。
- 部分網路測試是runtime intent injection或native正式delegate回呼，不冒充OS實體鍵盤／真人點擊。
- 三路／野區／五位置視野Bot、100場batch、完整選角UI、正式mana資源、兩機LAN与renderer所有cue恢复／crash recovery仍待後續，不以第5.3項覆蓋。

詳細本輪R學習與施法結果見`2026-10-04-unreal-first-learning-cast-60hz-progress.md`；所有失敗／修正／預防見error register E125。第5.3項完成後優先5.4的三路／野區／導航資料契約與合法可走路徑，停止無界重跑同一單路UI小增量。
