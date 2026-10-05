# 通用減速效果進度（2026-10-05）

## 本輪計畫與決策

1. 以 Lua `slow_enemy` 宣告減速比例與時間的 per-rank extras；shared model 與 FFI 生成器同步拒絕非法內容。
2. Rust 共用效果執行器先完成全部目標驗證，再提交傷害與減速。共用既有 BuffStore、UnitStats、移動系統及 TTL，不寫英雄特殊 handler 或 Unreal C++／Blueprint。
3. 用正式 60Hz 輸入確認生效、實際移動、來源聚合與到期；只測當前功能，完整框架驗收留最後。
4. 保存設計、結果與 E172 錯誤教訓；OpenSpec 5.5 維持未完成。

## 內容契約

`{kind="slow_enemy",reduction_key="slow_reduction",duration_key="slow_duration"}` 用於 instant active／ultimate、unit enemy target。兩個 extras 都須完整涵蓋所有 rank；減速比例有限且介於 1/1024 與 1，時間介於 1/1024 與 60 秒，cast range 介於 1/1024 與 10000。Lua 驗證先轉 shared schema 的 f32。

Ranger shot 保留既有 ID、傷害與冷卻，新增四級比例 25／30／35／40%，時間 2／2.25／2.5／2.75 秒。Bot 沿用 EnemyUnit 正式技能輸入，不直接加 buff。

## 執行與聚合

- 傷害與減速共用 caster／victim 存活、敵對、位置及當級距離 preflight；非法目標不輸出任何效果，也不啟動技能冷卻。這不是輸入 admission 的完整命令 rollback。
- Buff key 包含 ability、caster id 與 generation，避免不同來源互相覆寫。整數 Q10 `move_speed_bonus` 負值與 `__aggregation_family="generic_ability_slow"` 走既有 strongest-family；不使用 legacy 浮點 `slow_factor`。
- 不同通用來源只取最強減速；同來源刷新沿用 BuffStore 的新 payload／最長剩餘時間。此 family 不代表所有既有特殊 handler 的減速都已統一。
- 移動系統讀正式 UnitStats；TTL 到期由既有 buff tick 移除。沒有角色專用移動程式或新 ABI。

## 當前功能確認

- shared model `slow_effect`：1/1。
- base_content `slow_effect`：2/2，含傷害＋減速全序列拒絕、正式 60Hz 不同來源 strongest、實際導航位移及到期恢復。
- 三原型十二招既有測試：1/1。
- 固定 Lua `test_slow_effect_contract.lua`：14/14，含全部 rank、上下界、錯 target、缺 extra、零 range、同 key 不可漏驗。
- Unreal 正式生成與 `--check`：11 files／17 Lua inputs。

生成 identity hash `58136a29dddae4af` 保持穩定；catalog data hash 更新為 `f965f50e0f2772ac`。presentation content hash `f4f3b7871388ed77` 未變不代表 gameplay 資料未變，兩者不可混用。

## 未完成範圍

本輪沒有 OmGame 編譯／DLL stage、Editor／PIE、網路、完整 filtered 或 100 場驗收。減速 cue／HUD 顯示未新增；Mana 權威狀態與其他框架待辦仍未完成。這是通用效果功能完成，不是整個框架完成。錯誤與修正見 `unreal-moba-error-register.md` 的 E172。
