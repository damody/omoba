# 通用限時物品減傷進度（2026-10-05）

## 本輪計畫與決定

1. DamageReduce接既有BuffStore与DamageTakenBonus，不另建英雄專屬傷害流程。
2. 負比例用共用fixed payload；穩定item來源同道具刷新，跨道具採item_damage_reduce最強family，避免疊滿免傷。
3. 合法比例1/1024–1、時間1/1024–60秒；連同原cooldown與必要stats／store先預檢，拒絕不改CD／命令。
4. 使用共用packet sum-before-multiply契約，包含physical／magical／pure，不把全傷害減免冒稱護甲。最大100%合法，但本輪沒有另外測該邊界。

## 實作與局部確認

- `omoba-core/src/runtime/native/game_processor.rs`：來源`item_damage_reduce:<item-id>`、負DamageTakenBonus raw與最強family、標準duration。移除DamageReduce unsupported分支；HeadshotNext仍明確拒絕。
- `item.rs`更新支援契約註解；相鄰拒絕fixture改為非法NaN比例，避免把新增合法效果當成應拒絕。
- `item_damage_reduce`新2/2：正式60Hz買兩道具再用，25%與50%採50%，160混合packet實際扣80；cooldown拒絕不刷新、fixture合法刷新不疊加；較強效果到期後扣120，全部到期後扣160。另一英雄不獲減傷。
- 另一案例11情境：非法比例／時間被拒絕，無Buff、CD零、既有Hold命令保持。
- 相鄰`item_active_rejection`1/1通過；compiled-content-only。沒有新編譯或測試失敗，既有td_rounds警告保留。

指令：`cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only <filter>`。

## 邊界與待辦

正式Lua生成物品仍active=None；不新增作者schema／道具資料、runtime Lua、角色C++、Blueprint graph或協定。標準BuffStore死亡清除沿既有流程，本輪未另測死亡或Unreal視覺。HeadshotNext、真正護盾與generated active作者模型仍待實作；無部署／全驗收，21/31保持。E258保存缺口與決定，不抹除E256／E257當時事實。
