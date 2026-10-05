# 通用物品回魔進度（2026-10-05）

## 本輪計畫與決定

1. RestoreMana接共用權威ManaPool，不新增英雄專屬handler或前端數值。
2. 回復量須有限且為正1/1024–1000000；使用既有checked fixed換算。在正式mana模式，從目前英雄base與BuffStore解析合法容量。
3. clone pool完成容量／restore與必要事件佇列预檢後，才提交pool、實際ManaGained、冷卻與命令變更。
4. 滿魔合法使用仍耗冷卻但不產生零gain；缺pool不初始化或猜balance。

## 實作與局部確認

- `omoba-core/src/runtime/native/game_processor.rs`：RestoreMana不再屬於unsupported分支；所有拒絕路徑在pool／CD／命令變更前，回復使用ManaPool自己的clamp，不改HP或別人的pool。
- `item.rs`更新schema註解；正式生成catalog仍active=None，不擅自新增Lua內容或主動作者模型。
- `item_restore_mana`新2/2：正式60Hz買用、當前容量Buff＋60、超大回復量clamp、exact正gain與owner隔離、冷卻拒絕、滿魔合法使用無事件；另有8情境非法量／缺pool／非法容量不改pool／CD／Hold命令。
- 相鄰`item_active_rejection`與`item_sprint_60hz`各1/1，compiled-content-only建置。
- 首輪事件fixture讀错phase佇列已修正：正式step後查ScriptVisualEventQueue，直接API查ScriptEventQueue；記E257，不改正式dispatch時序。

確認指令：`cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only <filter>`。

## 仍未完成

DamageReduce、HeadshotNext、真正護盾與Lua生成active作者契約／UI仍待實作；此輪只完成共用回魔核心，不冒稱全部主動物品完成。無runtime Lua、英雄C++、Blueprint graph、Lua資料／hash／ABI14／wire5／IPC4變更，不部署或完整驗收。整體21/31維持。
