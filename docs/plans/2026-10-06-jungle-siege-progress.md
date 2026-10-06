# Jungle 的共用局部攻城

## 計畫與實作決定

1. 檢查5.5五位置Bot；Jungle原只協防／野怪，無野怪即Advance，略過塔基地。
2. 保留role_combat_focus的assist→farm優先；decide也保留farm安全fallback，但無farm後接共用建築挑選。
3. Jungle一般combat候選只允許kind4／5，不因共用selector加入solo hero或lane creep，其他角色原優先序不變。
4. 同一塔條件：current存活同隊兵距塔650內才攻，沒有支援且塔在1100觀測範圍則Hold；這是觀測，不是讀塔aggro或安全保證。
5. kind6鎖定建築不選；當前hidden／死亡／己方／超距不選，基地kind5才合法。沒有局部攻城機會沿原公開camp巡邏，不讀營地respawn timers或私有建築state。

## 當前功能確認

```text
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only siege -- --nocapture
cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only jungle_siege_60hz -- --nocapture
```

- core3項通過：共用原攻城與wave矩陣、新Jungle矩陣。協防仍先於野怪，野怪先於攻城；合法base可選，locked／dead／own／超距不選；近塔無兵Hold，死兵／敵兵／英雄不能假裝己方兵支援，孤立敵英雄不新增單挑。
- 正式60Hz base1項通過：three_lane_training原位mid敵塔，Jungle站400內；無wave透過正常HoldPosition並確認持久command。
- 未committed的wave位置變更不解除Hold；發布current披露後，精確選tower並由正式AttackTarget建立command。移除current tower披露後，cached baseline不再提供新tower攻擊輸入。

## 防錯 E275 與界線

本輪沒有新編譯或查詢失敗。記錄的是Jungle early return繞過共用攻城的實作缺口；不能以修正為理由取消assist／farm优先，也不能用權威私有建築解鎖補hidden資料。

fixture不搬動靜態tower／碰撞／建築state，以免再次製造不同資料源位置不一致。當前確認是Bot決策與正式輸入准入，不宣稱owner傷害归因或完整終局勝利。

這是局部機會策略，不是全域lane輪轉、停止農野時間門檻或完整終局策略。無Lua作者內容／生成hash／ABI／wire／IPC／Unreal C++／Blueprint變更，不需UE重建；最後更新Rust二進位並統一部署。完整5.5、100場、filtered與PIE驗收仍留最後，總進度21/31保持。
