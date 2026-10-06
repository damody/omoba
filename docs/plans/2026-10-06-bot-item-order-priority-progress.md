# Bot 主動物品的命令交接優先規則

## 計畫與決定

1. 檢查物品與普通攻擊命令交接；合法ItemUse沿共用權威執行器會clear_hero_command_queue。
2. 原回魔優先於防禦可能延誤低血自保，也可能為例行補魔取消已有前搖。
3. 以host-local ActiveItemPriority閉集表示護盾、減傷、逃生、回復、攻擊準備，依效果而非物品ID決策；同類以槽位穩定排序。不新增角色分支或權威效果副本。
4. 回魔及下一擊準備在Windup延後，其他相位仍可用；護盾／減傷／衝刺只有既有low-HP與current披露威脅門檻成立才可打斷。保持控制、既有效果、冷卻與本人資源限制。
5. 不改玩家輸入與真正ItemUse取消規則，不預扣冷卻、不在planner變更command或效果。

## 當前功能確認

```text
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only role_bot_active_items -- --nocapture
cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only role_bot_active_item_priority_60hz -- --nocapture
```

- core兩項通過：五種生成效果原矩陣、回魔新增前搖skip、混合背包兩種相反排列仍護盾→減傷→衝刺；已有三效果後前搖不回魔、解除前搖正常回魔，無威脅不自造防禦，planner不修改背包。
- 正式60Hz一項／兩情境通過：基地正式買入回魔（槽0）與可選護盾（槽1），低血低魔與可見對手，既有AttackTarget／Windup下例行補魔不送出也不變command；Backswing後正式補魔生效及耗冷卻。混合持有時先使用槽1護盾，回魔槽仍零冷卻。

## 防錯 E272

首輪60Hzfixture直接get_mut.unwrap命令佇列失敗；出生英雄可無此optional component，改為明確建立測試用HeroCommandQueue，不修改正式spawn。修正後同指定測試通過。無新編譯錯誤，既有dead_code警告不在本輪擴大處理。

這輪修訂E267回魔優先的決策；原紀錄是歷史，最新策略以上述閉集排序為準。無Lua／生成資料／hash／ABI／wire／IPC／Unreal C++或Blueprint變更，不需重新生成或UE建置。完整100場、PIE、filtered replay及release部署留最後，整體21/31與5.5保持未完成。
