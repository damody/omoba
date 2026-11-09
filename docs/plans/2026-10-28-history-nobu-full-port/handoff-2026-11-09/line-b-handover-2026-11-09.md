# B 線交接（2026-11-09 停工）

## 已完成的 commit（都在 live，沒有 push）
- 50423af0：使用道具扣充能時發出 CommittedCharge（E cover14 第一型）。
- 01913839：server 端 replica 也通過道具目標可見性檢查（C 回報的雙重處理）。
- 07fa0d1e：道具初始充能同步，用 announce_primed_item_charges，附 soak 與反向驗證；新增 bot intent ally_anchor_unit（千里大劍）。
- c38a574a：英雄技能 target_type 支援 unit_or_point（TargetTypeC::UnitOrPoint=3）。

## 未完成

### or21：信使規格 §2 倉庫第一部分
狀態：影子已完成，全套測試通過；land 排隊中被停工取消，live 沒有被改到。
- 影子：`target/snapshots/or21/tree`
- base：`target/m3p/or21base/`（共 14 個檔案）
- 測試腳本：`target/m3p/or21suite.sh [quick]`

測試結果：
- core 1625 個
- bc 498 個
- client runtime 168 個
- cm、tids、kit、omobab check、om_codegen 都通過
- 協定版本改成 19 後，另外跑過 proto、stash、ward、omb secure_join 測試，都通過。

內容：
- `MobaHeroSlot.stash`
- `shop::transact_with_stash`、`buy_with_stash` 與 StashPlacement
- 合成可用倉庫裡的零件
- `pick_up_stash`：在商店範圍內自動取出
- 範圍外不能賣
- `ShopError::StashFull`，回執 17
- `ShopCommand::SellStash`，回執 slot 是 6+i
- `OwnerStash` 13 bytes，加在 OwnerEconomyState 後面
- proto OwnerEconomyPresentation 加 `stash_enabled=9`、`stash_slots=10`，schema 4
- `SELECTIVE_LOCKSTEP_PROTOCOL_VERSION` 從 18 改成 19。共三處 assert，其中 omb kcp_transport.rs 那處需要 submodule commit。
- soak：`the_stash_rule_keeps_every_team_replica_with_the_authority`

下一步：
1. 先通知 C（C 已同意 land）。
2. 執行 `land_to_live`，對 14 個檔案各傳一個 `--change 影子=live@base`。指令在 or21land.log 對應的那次呼叫裡，可以參考 transcript。
3. moba_match.rs 的 live 已經被 E 的信使改動改過，會走 3-way merge。
4. 先在 omb commit kcp_transport.rs，再在主 repo commit 其餘檔案與 gitlink。

### or22：content-model 允許 unit_or_point 搭配 unit_target（A 要求）
狀態：影子已改，還沒測試。
- 影子：`target/snapshots/or22/tree`，檔案 `omoba-content-model/src/lib.rs` 的 `ability_unit_allegiance`
- base：`target/m3p/or22base/cm_lib.rs`

下一步：
1. `cargo test --manifest-path omoba-content-model/Cargo.toml`
2. land
3. commit
4. 通知 A

### S2：從倉庫賣出的 wire
狀態：還沒開始。
- 要放寬的檢查：
  - omb `kcp_transport.rs:196` 的 `item_slot < 6`
  - input_buffer 的回執
  - client `input_bridge.rs` 第 534、546 行的驗證
- 改成 `item_slot` 6..11 對應 `SellStash`。
- C 另外要一個 `in_shop_range` 旗標放進經濟狀態，用在合成價格提示要不要算倉庫。

### §3 信使送貨與回家
狀態：還沒開始。
- E 正在把 Summon 改成 Summon|Courier，範圍包括：
  - summon_orders.rs
  - filtered_specs.rs
  - snapshot.rs
  - visibility.rs
- E land 前不要碰這幾處。之後送貨、回家走 SummonOrder operation 3。
- B 要做的：
  - 生成時授予 courier_deliver 和 courier_return。
  - 信使庫存依玩家保存在 MobaHeroSlot。
  - 送貨：在商店範圍內取倉庫物品，走到距主人 150 內放進空格；主人死亡時等待。
  - 拒絕 operation 2（釋放）。
  - 離自家泉水 1200 內免疫物理傷害。
  - replica 同步與 soak。
  - §4 bots。
  - 實機確認千里大劍指向信使會中止。

## 已知問題
- 道具前搖的反向測試抓不到：replica 只有預測不同的部分會被修正，觀察者比對的是修正後的 hash（之前已回報）。
- sc3 cargo target 還沒跑 retention。復工後先執行：`tools\lua\lua.exe scripts\cargo_target_retention.lua target/m3p/sc3/cargo --drop-pdb`
