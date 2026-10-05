# Lua 宣告式魔力效果增量（2026-10-05）

## 計畫與實作

1. 共用 AbilityEffect 加入 restore_mana_self／spend_mana_self，引用每級 extras；Rust template/runtime 與 UE codegen 沿既有 validate_ability_effects 契約。
2. 固定 Lua gen_hero_registry.lua 生成相同 EffectOp，保留宣告順序。不新增英雄專用 Rust handler、角色 C++ 或 Blueprint graph。
3. GenericEffectHandler 先 resolve／preflight 整份 plan，再按宣告順序處理資源操作；所有額外扣魔准入後才發出 heal／damage／buff outcomes。RErr 由正式 host 交易回滾 metadata、資源 overlay、CD 與全部 deferred outcomes。不能把此 helper 脫離 host 後當成自帶 rollback 的世界交易。
4. restore 使用實際 ManaPool 容量截限，spend 使用當前交易餘額，不用 current_mana＋猜上限自行模擬。額外消耗為作者的確切效果量，不再套 metadata 成本倍率或重扣 mana_cost。
5. 當前 self 宣告要求 instant active／ultimate、target_type=none；可與 heal_self 組合，不能混入 point dash／敵人 damage。友軍補魔、持續再生 Buff、跨目標資源與效果讀取先前治療結果尚未有契約，不偽裝支援。
6. 不虛構 tooltip Heal（那是HP）。目前 EffectSpec 沒有Mana preview型別，魔力效果保留 Lua description／extras，略過該 preview，既有heal仍照常顯示。舊對局沒有 pool 時沿 ABI legacy 行為（不建立 pool），要有實際魔力效果須明確啟用 mana_enabled。

## 內容作者用法

英雄不宣告 rust_module，技能使用既有 levels／extras 配方，例如：

```lua
-- 技能片段；levels、max_level、ID等欄位沿既有資料契約提供。
ability_type = "active",
cast_type = "instant",
target_type = "none",
extras = { heal = {55, 85, 115, 145}, mana = {20, 30, 40, 50}, extra_cost = {10, 10, 15, 15} },
effects = {
  { kind = "heal_self", amount_key = "heal" },
  { kind = "restore_mana_self", amount_key = "mana" },
  { kind = "spend_mana_self", amount_key = "extra_cost" },
},
```

amount_key 必須有每級數值，允許0或[1/1024,1000000]的有限非負數；非零值不可在Q10變成0。Metadata mana_cost 必須先付得起，不能用效果中的恢復借款支付進入技能的成本。額外成本不足時，包含先前恢復與heal都不提交。

## 當前功能確認

- 共用模型直接manifest：新 mana_effect schema測試1 passed，含兩種enum、zero／minimum／maximum、負值／非有限值／太小／太大、缺key／錯rankcount、錯target／channel／mixed dash拒絕。
- 固定 Lua `scripts/test_mana_effect_contract.lua`：13/13 passed，isolated target fixtures證明兩種Rust EffectOp輸出與順序、邊界／非法資料拒絕；没有修改正式內容。
- base_content `--lib mana_effect`：2 passed，1為runtime preflight，1為正式Production60Hz host路徑（包含success與failure兩輪）。Lua既有ranger_patch rank1提供metadata45與heal55，fixture改共用效果列表：
  - mana145/max150 → 預扣45 → restore55只加50 → 額外55＋55 → 40；HP100→155；通知成本45／55／55及gain50。
  - mana50 → 預扣45 → restore55 → 第一額外55成功、第二不足 → RErr；原mana50／HP100保留，CD與資源通知都不留下。
- 共用 generic_effects 測試模組7 passed（含上述新preflight1，不能重複計數），保留damage／heal／slow／area／dash基本行為。
- UE codegen `--check` passed：11 files／17 Lua inputs，content_hash=f4f3b7871388ed77，沒有生成內容差異。

本輪共有9個不同Rust測試通過（model1＋正式host1＋generic7）、13個Lua案例。生成器contract與正式host fixture是分別確認，不冒充新Lua英雄完整生成／DLL載入／UE對局驗收；沒有full suite、100場、stage、UE build或PIE。

## 問題與決定

- 用scripts workspace一次選base_content與只有build-dependency身分的外部omoba-content-model，Cargo1.95 resolver panic（未取得NormalOrDev features）。改分別以omoba-content-model/Cargo.toml與scripts/Cargo.toml測試，model1與base2實際執行成功；不升級rustc或改鎖檔避錯。
- ResolvedEffect新增分支後漏補既有fixture exhaustive match，第一次base test編譯E0004。補上明確unreachable fixture分支，沒有用 wildcard隱藏缺漏。
- 一次apply_patch只含相同context，沒有實際修改；隨後以具名新測試插入，檢查實際test名稱與執行數。不得用patch回傳成功冒充落地。
- 路徑搜尋誤傳literal buff*與猜unit_stats位置，rg失敗；rg --files定位native/ability_runtime。沒有據此對Buff做推測式修改。

## 未完成範圍

OpenSpec保持20/30，不勾選5.5或6.2整項。Mana Buff疊加／再生與authoring、UI preview型別、三原型／100場、LAN／重連及完整呈現仍待；E177共享引擎baseline阻礙不變。完整驗收依使用者指示留到最後。
