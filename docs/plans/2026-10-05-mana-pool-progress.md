# 共用法力池第一階段（2026-10-05）

## 問題與決策

目前 `ParallelWorldAdapter::current_mana` 仍以最大法力冒充目前值，`spend_mana` 永遠回傳成功，`restore_mana` 未實作。Hero 原本沒有目前法力欄位；snapshot 仍以 0／0 表示未支援。這些是假實作，不可當成 Mana 功能已完成。

決定先建立可保存的共用 Rust 數值核心，再接正式施法提交及安全投影。不直接在每個技能 handler 補扣费：event dispatch 共用 immutable cache、outcomes 延後結算，同 tick 多次施法可能讀相同舊餘額；parallel on_tick 也不能靠 Mutex 搶鎖順序決定遊戲結果。

## 已實作

- `ability_runtime::ManaPool` 私有 current／maximum／regeneration_remainder，合法序列化狀態經 checked deserialization；拒絕負數、餘額超上限、非法餘數或滿池仍有小數 credit。
- `spend` 餘額不足或負費用不改狀態，成功精確扣除，0 費用合法。連續呼叫使用更新後餘額。
- `restore` 回傳實際恢復量、截到剩餘容量；避免先相加造成 i64 overflow。
- `set_maximum` 保留絕對目前量，縮小時限制到新上限；增加上限不自動補魔。
- `regenerate` 用 i128 乘積與 Q10 小數餘數，低速恢復不因每 tick 量化消失。只接受非負 rate／active dt，零 dt 不變；滿池丟棄 credit，不儲存將來可兌現的恢復。
- Hero 新增 optional pool，預設 None、舊 JSON 保存缺欄位仍 None。`initialize_mana_pool` 明確且冪等，重複初始化不補滿已消耗法力；`refresh_mana_capacity` 重算成長容量但不補魔。

這是 host Rust 資料，不新增 script ABI dependency 或角色 C++／Blueprint。

## 計畫進度

1. [x] 共用数值核心、保存驗證、Hero 明確啟用與容量更新。
2. [ ] 正式模式的規則啟用／Lua 恢復率、出生與重生、升級／active dt 接線。
3. [ ] deterministic ordered 施法扣費與腳本 API：同 tick 餘額、失敗不扣、避免 host／handler 雙扣及不確定平行寫入。
   - metadata host 扣費與 serial ledger、失敗丟棄效果、managed read view 已完成，正式60Hz2＋adapter1通過。任意腳本資源變動仍未接線，因此整項不勾選；詳見 `2026-10-05-mana-cast-progress.md`。
4. [ ] committed Mana facts、baseline／fresh bootstrap／filtered gameplay hash 與 owner HUD／IPC／Unreal 通用呈現。
   - committed Mana26／v1 codec、visibility gate、baseline及短60Hz filtered hash接線完成（core4／base1）；HUD／IPC／UE與正式協商仍待，因此整項不勾選，見 `2026-10-05-mana-projection-progress.md`。
5. [ ] Bot 讀自己的正式法力，必要 current-function 60Hz 確認；完整網路／UE／100 場與效能驗收集中最後。

在第 2–4 項接好前不自動啟用正常對局扣費，也不改 HUD 未支援狀態。不把只在權威扣費而 replica 不知道的方案當通用解。

## 本輪確認

- `cargo test --manifest-path omb/Cargo.toml -p omoba-core --lib mana_pool -- --nocapture`：初次核心 5/5。
- 新增 Hero 接線後，`cargo test --manifest-path omb/Cargo.toml -p omoba-core --lib comp::hero::mana_pool_tests -- --nocapture`：2/2。
- 7 個直接相關測試分兩次確認；不是正式施法／60Hz 對局或完整驗收。
- 既有 td_rounds dead-code warnings 保留；無編譯或測試失敗。E173 記錄本輪發現及操作防錯。

第一階段當時 OpenSpec 整體仍 20/30，5.5／6.2 未勾選；當時法力池未接 script adapter、沒有自動再生系統、未發 wire／IPC 資料。後續 managed cast／read view 進度見 `2026-10-05-mana-cast-progress.md`，不覆寫當時測試結果。
