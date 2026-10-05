# Bot 通用法力預算

## 計畫與決定

1. 抽出 `checked_mana_cost`，讓 Bot 與正式施法共用原始 f32 腳本成本→Q10、倍率向上進位及 i128／i64 範圍驗證；不使用第二份 compiled Fixed64 成本重新量化。
2. Bot 從 World 的 AbilityRegistry 讀取當級 metadata。此 registry 在初始化時直接 clone ScriptRegistry 的 AbilityDef；SimulationDriver 已接管 ScriptRegistry，不把執行器 registry 再放回 World。
3. 只讀受授權本人 Hero 法力及 BuffStore 倍率，對手仍只取 committed team disclosure。按作者政策優先序跳過付不起、缺資料或非法成本，再選後續技能。None 法力池保持 legacy 行為；Some 零池不是未支援。
4. Bot 只產生正式 CastAbility，不預扣、不保留、不寫冷卻。後端 serial ledger／handler 成功提交仍是唯一扣費路徑。

## 實作

- 新增 `ability_runtime/mana_cost.rs`，正式 dispatch 改用共用 checked 函式。
- planner 成本 provider 與候選選擇分離，不寫死英雄名稱、技能槽或成本；由實際 learned rank 查 metadata。managed 模式缺 AbilityRegistry／BuffStore 或 metadata 會跳過技能，不猜免費。
- 原有目標、距離、CD、學習、治療門檻與未啟用模式保留。施法都仍走正常 authority input dispatcher。

## 當前功能確認

- core `mana_budget` 2/2：成本合法／非有限／極小／負值／溢位、正數進位、免耗／半價／加價、剛好夠／少一 raw、缺資料、legacy None、不預扣與跳過候選。
- base_content `mana_budget` 1/1：正式 Production60Hz 啟用 Mana 對局，rank4 箭雨成本60被跳過，rank1 自療成本45成功；思考不扣魔，正式扣45後依有效 dt 再生精確一致；低餘額不施法，本人免耗 Buff 後選原優先技能並由正式流程成功／CD／零扣費。
- base_content `mana_cast_` 2/2：抽共用函式後保留正式拒絕不扣費、成功扣費、同批次 ordered 超支與CD限制。
- 沒有 DLL stage、Unreal restart、長對局或完整驗收。既有 td_rounds warnings 保留。

## 未完成

本批完成 Bot 避免負擔不起技能，不是低魔回城／基地補魔策略。Buff／裝備再生、explicit script 資源操作、socket／UE Mana 接線及100場完整 Bot 對局仍待。E177 的 native engine 編譯阻礙未變，OpenSpec 維持20/30，不勾選5.5或完整UI。
