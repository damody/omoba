# Bot 持續回魔策略（2026-10-05）

## 計畫與決策

1. 新增共用 `SelfManaRegeneration` 意圖，作者提供 HP／Mana 千分比門檻、`rate_key`／`duration_key`。只適用新的 additive、無 aggregation family 的 `mana_regen_constant` Buff，不把容量、百分比、BaseManaRegen override 或其他資源支出套進同一公式。
2. rate 各級必須正且不超過1,000,000；duration 各級正且最多60秒，完整 compiled extras、none target／instant active或ultimate。plan與執行中的Bot設定沿既有 requires_mana gate，未明確啟用拒絕。
3. `UnitStats::checked_mana_regen_with_flat_bonus` 共用真正權威的先加flat／截零／percentage／total factor／Q10 rounding流程，不複製簡化倍率，也不修改BuffStore。
4. 規劃比較兩條路徑的預估期末餘額：不施放＝current＋原自然rate×duration；施放＝current－權威metadata成本＋新rate×duration；兩者都截目前容量，只有施放者較高才走低魔分支。這避免把原本自然回魔算成技能收益，以及容量已足夠時仍浪費技能。
5. 這是以目前本人Buff／容量維持不變的有限時間規劃，不是未來收益保證；中途死亡、驅散、modifier到期、容量改變或額外腳本成本都可能改變結果。未來收益不計入現在可付預算，不預扣、不預先回魔，實際每tick仍由正式權威結算。
6. `generic_mana_buff_id` 抽至輕量script-abi純函式，由腳本與Bot共用原格式，含ability／stat／entity／generation，未改FFI layout。本人同來源尚在BuffStore時，低魔分支不重施；包含remaining零但尚未移除的entry，避免聚合仍計入它時誤當新來源。其他英雄或不同生命的Buff不共享身分。
7. HP分支保留治療用途：低血量時仍可以正常治療並刷新附带Buff；不為了「絕不刷新」阻斷救命技能。原回城／撤退優先序與正式成本／CD gate不改。
8. 只有 opt-in mana 配方的 ranger_patch 改為此意圖（HP600／Mana500，原Lua mana_buff_value／mana_buff_duration），一般配方維持SelfHeal、不改平衡、不寫英雄專屬C++或Blueprint。

## 作者配置

```lua
policy.intent = {
    kind = 'self_mana_regeneration',
    below_hp_per_mille = 600, -- 0 可停用 HP 分支
    below_mana_per_mille = 500,
    rate_key = 'mana_buff_value',
    duration_key = 'mana_buff_duration',
}
```

這些key是作者對該技能 additive flat regen 的規劃契約，不能指向heal或容量extras冒充恢復。全效果預演、多段支出、百分比或持續跨目標策略需要各自的契約，不從tooltip猜測。

## 當前功能確認

- core `--lib mana_regeneration`：2 passed。相等／負收益不放、正收益才放、不預扣、自然回魔已補滿不放、來源未知／已有時略過、低HP仍治療、None不建立需求、extras驗證；共用UnitStats對負flat截零／兩級倍率／非法payload與唯讀性確認。
- base_content `--lib mana_regeneration`：正常generated manifest正式Production60Hz 1 passed。满HP、current100／capacity280，rank1原cost45／持續2×6預估增益12不放；倍率0.25後cost11.25正式slot1施放，立即pool精確100－11.25＋自然7×有效dt（不是先補12），HP1000不變、正確CD與source Buff落地。只清該CD後仍被同來源Buff gate擋住，明確移除該fixture來源後可重新考慮新Buff。
- 固定Lua prepare成功：`scripts/run_moba_role_ue.lua --prepare-only --recipe scripts/lua_data/moba_mana_single_player.lua --output omfue/Saved/MobaManaRegeneration-20261005-01`，1 human／9 Bot／60Hz。不啟動服務或Unreal。
- ABI `--lib generic_mana_buff_identity`：1 passed，格式與ability／stat／生命generation區隔。core `--lib mana_recovery_plan_requires_explicit_rules`：1 passed，新增regen政策也要求明確魔力規則。最後production重新編譯與同一base案例通過，wrapper新增警告消失；不把重跑同一案例算成額外成功案例。
- 共5個不同的直接相關功能案例確認，root／omb／omfue whitespace檢查成功；未跑完整suite、長對局、UE、LAN或100場。

## 防錯與下一步

新production路徑讓舊budget wrapper只剩測試使用，第一次編譯出現dead_code warning；將wrapper標成cfg(test)，不壓掉整crate警告。新編譯／功能案例沒有失敗。持續預測不等於完整未來效益證明，部分功能確認也不勾選完整5.5；OpenSpec仍20/30，E177引擎限制未繞過。
