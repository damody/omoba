# Bot 主動物品共用決策

## 計畫與決定

1. 每個 `BotItemBuild` 可宣告 `active_use`。省略保持舊行為，序列化不新增空欄位；Lua 只在開局工具匯出配方，執行期決策是編譯 Rust。
2. 政策為 combat_radius 1–10000、defend_below_hp_per_mille 與 restore_below_mana_per_mille 0–1000。零停用對應資源門檻；未知欄位或越界設定拒絕。
3. 僅讀 owner 健康、魔力池、背包、BuffStore 與當隊目前 committed disclosure。防禦需要已披露威脅與低血量；增傷需要合法種類的已披露近距離單位。不查看隱藏敵方 ECS、aggro 或記憶位置。
4. 決策只產生六格 NoTarget `ItemUse`，先由正式 dispatcher 驗證才套用效果與冷卻；不直接修改 HP／魔力／裝備／命令。
5. 候選順序為回魔、護盾、減傷、衝刺、普攻增傷，再按槽位排序。非法或非零冷卻、被動／未知商品、既有盾／減傷／衝刺／待消耗增傷跳過；定身不浪費衝刺，普攻前搖不被增傷道具取消。
6. 購買仍先於主動使用；已在回城或暈眩的 Bot 保持原 gate。主動使用先於後續回城／技能／普通戰鬥，每次 think 至多一個正式輸入。
   未啟用 active_use 時不建立新增 owner 效果觀測；已有物品減傷按自己的標準來源辨識，不因其他易傷加值抵消淨數值而浪費第二次減傷冷卻。
7. 訓練 Lua 配方依五位置加入主動商品與可調門檻，每個最終背包不超過六格。既有非魔力配方保持魔力規則關閉，不為了道具偷偷啟用新 gameplay 模式。

## 當前功能確認

- core 純決策：五種正式 generated 效果、opt-in、無威脅／死亡威脅、冷卻與非有限值、效果已存在、定身、前搖及非法 policy，一項矩陣測試通過。
- base_content 正式 60Hz：五種商品各自正式購買、Bot 唯讀規劃、本人 NoTarget ItemUse、實際效果與冷卻、另一 owner 裝備不變、冷卻中不重送，一項／五情境通過。
- 固定 Lua 開局工具局部確認7/7通過，包含更新配方的真正 Rust configuration preflight與開局前拒絕非法配方；lifecycle使用mock，沒有啟動server／runtime／Unreal。不做整場或完整驗收。

重現指令：

```text
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only role_bot_active_items -- --nocapture
cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only role_bot_active_items -- --nocapture
D:\code\omoba\tools\lua\lua.exe scripts/test_moba_role_launch.lua
```

## 問題紀錄 E267

- 工具錯誤：字面 Windows `*bot*` 路徑 os error123；猜 comp/item.rs、comp/property.rs、scripts/lib 不存在。改用 rg --files 確認 inventory.rs、creep.rs 與實際 Lua 入口。後續未知路徑必須先列檔，不得重用猜測。
- 測試編譯錯誤：moba_mana_capacity 是無參數方法；Inventory 沒有 is_empty，改用 items().count()。先讀現有定義再呼叫，不用概念上的 API 名稱。
- 測試失敗：Sprint 的標準限時加值在 MoveSpeedBonusBuff，不是 MoveSpeedBonus。修正斷言，不改正式執行器。
- 檢視發現購買 fixture 應先在出生商店購買，再移到隔離戰鬥位置；已修正且明確檢查正式入背包，沒有繞過商店距離驗證。
- 大段輸出截斷不當作完整已讀；既有 td_rounds warning 保留。

## 尚未完成

這是5.5的策略增量，不是100場 headless、LAN、Unreal畫面或全框架驗收。21/31維持，未改協定或內容 catalog；版本仍為 selective wire6／IPC5／C ABI15，release統一建置部署留最後。
