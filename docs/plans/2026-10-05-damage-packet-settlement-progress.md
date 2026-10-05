# 正式傷害 packet 共用結算進度

## 計畫與決定

1. 確認 Carry 補刀所需的普攻結算來源，不另造攻擊公式。
2. 抽出正式 Outcome::Damage packet 結算，供權威 HP／TD 共用；未取得合法披露倍率以前，不接入假精準 Bot 預測。
3. 只確認當前功能，更新防錯與待辦；完整 UE／多人／效能驗收留到最後。

## 本次實作

- `unit_stats::settle_damage_packet`：Fixed64 三種 packet 傷害先合計，再乘 `max(1 + DamageTakenBonus, 0)`；純函式不接觸 ECS，保留正式計算順序。
- `UnitStats::incoming_damage_packet`：權威 adapter 取得 victim 的聚合倍率；game_processor 只計算一次，普通 HP 與 TD layer resolver 使用同一結果。
- 來源 final_atk／accuracy 仍在既有 projectile 建立路徑，避免重複套用。沒有修改 gameplay、內容 hash、ABI、生成 C++ 或 Blueprint。
- 沒有把尚未接上 Outcome::Damage 的獨立 armor/block helper 當成正式公式。DamageInstance 是另一條既有路徑，本次不改變它。

## 當前功能確認

- `cargo test --manifest-path scripts/Cargo.toml -p base_content --lib damage_packet_settlement`：1 passed。使用正式 Production60Hz driver 與 generated lumen_bolt；80 傷害在 1.5／0.5／負倍率下分別結算 120／40／0，三例均有正常 cooldown。
- `cargo test --manifest-path scripts/Cargo.toml -p omoba-core --lib damage_packet_settlement`：1 passed。零／正／負倍率、零 packet、Fixed64 合計後 rounding 與 BuffStore adapter。
- 首次 linker 因 C 槽滿失敗；當次 TEMP/TMP 改到 D 槽既有建置目錄後成功，退出還原。錯誤詳見 E194。

## 尚未完成

後續更新：E195 已接上安全聚合觀測、變更事件與 Carry 目前可擊殺普攻優先；原先缺觀測的前置已解除。詳見 `2026-10-05-incoming-damage-observation-progress.md`。未來 impact／完整 farming 仍不保證。

- Carry 精準補刀必須取得合法、目前已披露的目標入傷倍率，並處理 accuracy 與投射物抵達前 HP 變化；不承諾命中／尾刀，不從私有敵方 BuffStore 讀取。
- 若增加公開 combat observation，需涵蓋版本、可見性撤銷與後續變動更新，不可只有 baseline 欄位。
- 完整 5.5／100 場與 Unreal 驗收未完成，OpenSpec 維持 20/30。E177 的共享引擎修改阻擋仍未解決；不覆蓋使用者引擎變更。
