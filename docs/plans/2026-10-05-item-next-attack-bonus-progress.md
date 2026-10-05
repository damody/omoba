# 通用下一次普攻增傷進度（2026-10-05）

## 計畫與決定

1. 不改基礎攻擊力；BuffStore提供通用arm_next_attack_bonus／next_attack_bonus／consume_next_attack_bonus。
2. 一個entity只有一份私有pending值，重複準備取最強，跨道具不累加。標準Buff清除一起清掉；沒有另外的英雄timer。
3. normal attack projectile完成source／target／TAttack預檢後消耗，增傷在accuracy之前加入physical；miss消耗且零damage，技能投射物不觸發，視覺副本零damage。
4. HeadshotNext物品bonus有限正量1/1024–1000000，使用既有checked fixed換算並檢查attack／store後才準備、耗CD／清命令。

## 局部功能確認

- `item_next_attack_bonus`：正式Production60Hz買→用，owner準備60／另一英雄零、base atk不變、CD拒絕不改pending，1/1。
- 核心`next_attack_bonus_consumes`：實際handle_projectile，非法target缺Pos保留；重arm60與20採60，基礎10發出唯一70傷害真彈＋兩顆零damage visual，下一次真彈回10；100%miss消耗且真彈零傷害；remove_all清pending，1/1。
- 相鄰`item_active_rejection`更新非法NaN bonus並通過1/1。

指令：

- `cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only item_next_attack_bonus`
- `cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only next_attack_bonus_consumes`

首輪core漏kcp导致既有test類型不可用，修正feature後通過，詳記E259。原有td_rounds警告保留。

## 邊界

尚未測完整前搖取消／投射物抵達／死亡重生／renderer預測或Unreal操作；本輪確認的是準備及權威正常launch封裝。私有pending Buff不是新公共visual ID，不發布任意payload。真正Shield、Lua生成active作者模型與前端操作仍待後續；無runtime Lua／英雄C++／Blueprint graph或hash／ABI14／wire5／IPC4變更，不部署／全驗收，整體21/31保持。
