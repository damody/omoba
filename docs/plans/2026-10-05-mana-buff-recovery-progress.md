# 魔力 Buff 回魔接軌（2026-10-05）

## 計畫與決策

1. 重用既有 StatKey／BuffStore／UnitStats，不新增英雄專用欄位或 C++。
2. MOBA 的自然回魔由 UnitStats 聚合；基地加成沿既有活本人基地 eligibility 獨立提供。最後合併兩種速率，只呼叫一次 ManaPool.regenerate，保留既有有效時間與Q10餘數。
3. 自然速率為：正值 BaseManaRegen 覆寫自然基底，否則用Lua自然速率；加上 ManaRegenConstant／ManaRegenConstantUnique；依序乘上 max(0,1+ManaRegenPercentage)、max(0,1+ManaRegenTotalPercentage)。每次乘法後Q10量化；flat先截到0。
4. 基地屬保護性恢復，不受上述自然Buff倍率／override影響。全mana關閉、warmup／pause／finished、死亡／lethal或非active delta仍不回魔。
5. checked_sum_add 保留 explicit __aggregation_family 的最強值與相等時較正值規則。以i128加總後轉回i64，避免HashMap次序影響溢位；unsigned_abs支援i64::MIN。沒有family的來源仍加總，Unique stat名稱不代表自動跨來源去重。
6. checked_mana_regen 的非法payload／溢位回None，既有mana_regen入口將該次自然恢復視為0，不改其他stat舊聚合。自然＋基地合併若超i64，保留合法基地速率，不wrap、不巨量補魔、不每tick重複印log。
7. 魔力上限Buff與Lua宣告式持續Buff需要独立容量／刷新契約，留待後續；不與這次回魔改動混入。

## 內容介面

既有Rust script的 add_stat_buff 可使用固定整數raw payload，例如：

```json
{
  "mana_regen_constant": 3072,
  "mana_regen_percentage": 512,
  "mana_regen_total_percentage": 1024,
  "__aggregation_family": "example_mana_recovery"
}
```

代表固定＋3、該組＋50%、總倍率＋100%；自然基底5時結果24/秒。整數是Q10 raw，不是世界單位，保留舊floating payload相容但必須能安全轉成i64。若要停自然回魔，百分比-1024代表倍率0；base_mana_regen=0不覆寫既有基底。

## 當前功能確認

- core `--lib mana_buff`：2 passed。證明override／兩種flat／percentage／total組合45、雙負倍率不得變正回魔、invalid／極端乘積fail closed；MAX＋MAX－MAX不同插入次序同結果；family強弱／移除強者回弱者／MIN绝對值安全。
- base_content `--lib mana_buff`：正式Production60Hz新1 passed。活本人基地的自然24＋基地60共用一個pool／remainder；pause不回；小於tick的Buff由正式buff system到期移除後回自然5；非法溢位Buff仍只保留基地60。
- base_content `--lib mana_sustain`：既有2 passed，Bot正式Recall／等待／離開，以及基地opt-in／owner-home／容量截限維持。
- 共5個不同測試成功。沒有完整驗收、DLL stage、UE建置／MCP／PIE、100場或效能宣稱。正式測試直接安裝BuffStore payload以隔離結算，不冒充Lua持續Buff生成已完成。

## 問題與防錯

- 發現已有mana_regen helper但MOBA原本只使用固定自然rate；不能因函式存在就宣稱已接線，必須正式60Hz確認餘額。
- 原聚合fixed加法、abs與乘法對極端數值可能溢位；原公式只截最後結果，兩個負倍率會回到正值。使用checked resource聚合與每個factor截限，保留其他stat語義，避免全面替換造成無關變更。
- 讀buff_tick多段輸出超出小上限截斷，不能用截斷末端推論；到期行為以實際正式tick後Buff已不存在與精確pool結果確認。
- 新测试插入在舊移速回歸comment之後，導致comment暫時指錯測試；檢查diff後移回對應函式。這輪沒有編譯或測試失敗。

## 後續

OpenSpec保持20/30，5.5／6.2整項仍未完成。下一步魔力上限Buff，再接Lua持續Buff作者介面；renderer／LAN、三原型100場及完整呈現仍待。E177共享引擎baseline阻礙未解除，最後再整體驗收。
