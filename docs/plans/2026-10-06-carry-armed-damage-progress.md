# Carry 已準備普攻增傷的共用觀測

## 計畫與實作決定

1. 延續5.5的目前命中尾刀策略，不新增預測敵方私人狀態。
2. 發現真正正常投射物有 final_atk + next_attack_bonus，但 Bot 只算前者。
3. 新增唯讀 UnitStats::normal_attack_physical，正常 launch 與 Carry 同用；一次性量仍在 outgoing modifier 之後加入，不二次放大、不修改基礎攻擊力。
4. 只讀自己已合法準備的量；敵方HP與incoming倍率仍取 committed current disclosure，unknown仍未知，accuracy gate及前搖保留策略保持。
5. 觀測不消耗、不建立投射物、不修改命令。合法 launch 的原消耗位置、miss consume／zero physical以及visual零傷害不變。

## 當前功能確認

三項指定測試各1 passed：

```text
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only normal_attack_physical_includes -- --nocapture
cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only carry_armed_attack_bonus_60hz -- --nocapture
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only next_attack_bonus_consumes_only_valid_launch_and_never_visual_copies -- --nocapture
```

- 純公式：base20＋flat10乘1.5後加60，結果105，不是135；重複查詢不消耗，其他owner20，真正消耗後45。
- 正式60Hz：immune兵與HP80兵，own30攻擊未准备不選後者，其他owner准备1000不影響；本人准备60後精確選後者。
- 重複Bot規劃不消耗；從current披露移除候選後，cached baseline不能繼續選它。恢复披露後交正式AttackTarget至launch，本人bonus歸零，對手保留1000。
- 舊launch測試仍確認合法launch、miss、非法launch与visual副本的原消耗語意。

## 防錯 E271 與界線

首輪fixture手寫錯total_damage_outgoing_percentage，得到90而非105；改用StatKey::as_str，不修改production公式。查詢systems/*.rs再次錯用Windows literal glob，已記error register；後續先列實際檔案。

無內容／hash／ABI／wire／IPC／Unreal C++／Blueprint變更，不需要重生成或UE建置。未進行100場、完整filtered replay、release統一部署或PIE驗收。完整5.5與總進度21/31仍待最後驗收。
