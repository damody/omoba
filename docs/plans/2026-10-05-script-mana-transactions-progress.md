# 腳本魔力交易增量（2026-10-05）

## 計畫與實作決定

1. 讓既有 GameWorld spend_mana／restore_mana 使用正式 ManaPool，不增加 ABI 方法或角色 C++。
2. 每個 serial hook 持有交易 overlay，成功後以絕對池狀態提交；事件和 tick 共用同一帳本。managed pool 存在時，tick 依 entity／generation／unit ID 固定順序執行；沒有 managed pool 的既有 TD 保留平行路徑。
3. metadata 成本由 host 只扣一次，handler current_mana 看預留後餘額；script spend 是額外成本。負數、失效 generation、死亡、餘額不足拒絕；restore 依容量截限，只通知實際增加量。
4. 成功交易才產生 SpentMana／ManaGained，經 Outcome 回填 ScriptEventQueue，下一次 dispatch 通知，避免同步遞迴。ROk 提交；RErr／已捕捉的 hook panic 丟棄 deferred mutations 與資源通知。legacy caster 修改 managed recipient 時也遵守失敗回滾。
5. 移除舊 cast_mana_view，保留唯一交易視圖；不在 Unreal 或 AI 重算魔力，不為測試改正式英雄生命週期。

## 本功能確認

- core `--lib mana_`：19 passed，包括新交易邊界1與更新的 discard1；不是全套驗收。
- base_content `--lib mana_script`：最終3 passed，使用真實 manifest metadata、SimulationDriver Production60Hz、正式 dispatch／outcome 路徑：
  - 90 → metadata45 → restore20 → extra10 → tick1 = 54，成本通知45／10／1，恢復通知20。
  - 同 batch 兩次不同技能與重複技能按序結算；第三次被 CD 擋下，最後19。handler RErr 回滾成本、恢復與 CD，僅成功 tick 留89。
  - legacy caster 對 managed recipient 的 restore／extra spend 後回傳 RErr，recipient 仍50、無資源通知、無 CD。
- base_content 既有 `--lib mana_cast`：2 passed，保留成功預扣／非法目標拒絕／同批餘額與冷卻行為。
- 上述正式案例刻意使用手動 managed pool、關閉對局再生，以隔離交易；不代替 mana-enabled 完整對局、回呼下一 tick 實戰或效能驗收。

## 問題與防再犯

- 首次 fixture 嘗試替換 training_ranger unit handler，但現有 manifest 只註冊英雄 ability，没有該 unit；第一次兩項失敗得到55／20（缺 tick1）。補 tag 仍失敗，追查 units() 後改明確 append UnitDef 並掛 ScriptUnitTag，三項最終成功。不能降低 expected 數值或替正式英雄偷偷加入 tick。
- patch 上下文不夠獨特，mana view 曾誤入 buff remaining 查詢；在編譯前發現並移回 current_mana。
- 猜測 comp/script.rs、script/parallel_world_adapter.rs、generic_ability.rs 等路徑失敗；使用 rg 定位 native/scripting 與 generic_effects。多來源長輸出截斷，應分檔／限段查閱，不依截斷內容作結論。
- 跨角色資源操作不可以只依 caster 是否有 pool 決定 rollback；以交易實際 dirty pool 和 cast 成功狀態判斷，並加入正式回歸。

## 尚未完成

OpenSpec 保持20/30；本增量不勾選整個5.5或6.2。Lua 宣告式 resource effect、Mana相關 Buff、十人／100場、真實 transport／重連、完整 HUD／效能仍需後續工作。E177 共享引擎 baseline 阻礙未處理，這輪沒有 UE 編譯、stage、MCP／PIE 或全套驗收，也未聲稱 FFI 邊界外的 panic 能恢復。
