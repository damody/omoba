# Bot 即時魔力恢復意圖（2026-10-05）

## 計畫與決策

1. 使用共用 `BotAbilityIntent::SelfRecovery`，不是英雄 ID 分支。作者提供 `below_hp_per_mille`、`below_mana_per_mille`、`restore_key`；HP 門檻零表示只看魔力，HP 上限1000、Mana必須1..1000，恢復 extras 必須完整涵蓋各級且在0..1,000,000世界單位。
2. 同一技能只保留一筆政策，HP OR Mana，不另加重複 policy。本人存活、已學習、四槽內、無冷卻且付得起權威成本才產生正式 CastAbility；Bot 不修改 pool、CD 或效果。
3. 低魔分支使用本人非零容量與嚴格低於門檻，且作者指定的即時恢復量必須大於 host 的 metadata 成本，包含本人 UnitStats 成本 Buff；不讀敵方資源、不從 tooltip 推測、不把持續回魔或容量 Buff 當即時回魔。
4. `restore_key` 是作者的即時恢復規劃提示，並非完整腳本效果預演。這個意圖適用沒有其他額外資源支出的即時恢復技能；額外支出、多段有条件效果與持續 Buff 策略仍需各自的完整規劃契約，不能用單一恢復數字冒充淨收益證明。實際效果仍由原本 script transaction 與權威准入結算。
5. plan 與執行中的 RoleBotConfig 都要求 explicit mana_enabled。舊 SelfHeal／省略魔力旗標的配方不改；動態沒有pool時只可能走原HP分支，不建立魔力需求。
6. 只把 opt-in `moba_mana_single_player.lua` 的 lumen_touch policy 換成 self_recovery：HP600／Mana500／mana_restore。它原成本55、恢復20，正常滿血低魔不亂施法；有足夠成本折扣才走Mana分支。原低血量療傷照常。
7. 不修改平衡數字來製造成功；既有回城／安全撤退優先序不變。配方Mana低於200仍先回城／撤退，200到500的資源區間才可能使用技能恢復；在基地持續補魔仍由原策略處理。

## 當前功能確認

- `cargo test --manifest-path scripts/Cargo.toml -p omoba-core --lib mana_recovery`：2 passed。正收益／零收益／負收益、嚴格門檻、原HP分支、不預扣、不能超預算、None／零容量／死亡、作者欄位驗證及明確啟用旗標。
- `cargo test --manifest-path scripts/Cargo.toml -p base_content --lib mana_recovery`：1 passed。正常 generated manifest，正式Production60Hz；滿血低魔不施放55成本／20恢復的虧損技能，本人成本倍率0.25後正式slot1成功，精確60－13.75＋20＋5×有效dt，HP1000不變且冷卻生效，下一次思考不重施。
- 固定Lua `scripts/run_moba_role_ue.lua --prepare-only --recipe scripts/lua_data/moba_mana_single_player.lua --output omfue/Saved/MobaManaRecovery-20261005-01` 成功：1 human／9 Bot／60Hz。只生成配置，不啟動server、runtime或Unreal。
- 本次 Rust 編譯與功能案例沒有失敗；只有原有未使用項目警告。工具錯誤與設計陷阱記錄於 E187。

## 未完成範圍

持續 Buff 的資源策略、三原型完整對局、100場、完整UE／LAN／重連／cue與效能最後驗收仍待。共享Unreal引擎建置限制E177未繞過、不混舊ABI DLL。OpenSpec仍20/30，不把本功能當完整5.5完成。
