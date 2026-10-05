# 通用物品主動效果准入進度（2026-10-05）

## 計畫與決定

1. 先修復共用物品執行器的永久數值污染與log-only假成功，不先加無實際內容的前端快捷鍵。
2. 衝刺使用既有BuffStore；同item來源刷新、不同item來源共用最強family，沿既有到期／死亡清除，不造獨立計時器或英雄分支。
3. 未實作的回魔／減傷／下一擊增傷明確拒絕；保留明示為瞬回HP的legacy Shield，不把它冒稱真正護盾。

## 實作

- `omoba-core/src/runtime/native/game_processor.rs`：SprintBuff不改CProperty.msd，來源`item_sprint:<item-id>`；MoveSpeedBonusBuff payload為fixed raw，family為item_sprint。
- 衝刺bonus有限1/1024–10000，duration有限1/1024–60秒；cooldown有限0–3600秒，且stats／BuffStore須可用。拒絕發生於效果、冷卻與命令變更前。
- `item.rs`明示保留但未支援的schema分支；不刪作者型別、不以no-op成功掩蓋未完成效果。
- 正式Lua生成物品目前都是被動；本輪不加入主動物品作者schema或內容，不宣稱完整active道具框架已完成。

## 局部功能確認

使用`cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only <filter>`：

- `item_sprint_60hz`：正式60Hz買→用，移速生效且基礎不變、CD拒絕不刷新、合法刷新不疊加、20步後到期回復，1/1。
- `item_active_rejection`：非有限bonus／duration／cooldown、零duration與三種未實作效果，拒絕後CD零／基礎不變／原Hold命令保留，7情境1/1。
- `single_lane_buy_use_sell_keeps_mixed_item_input_order`：既有買用賣顺序與瞬回HP相容，1/1。
- 現有td_rounds dead-code警告未擴大處理。編譯型別錯誤與fixture購買拒絕已修復，詳記E256。

## 範圍與待辦

沒有Lua資料／內容hash／ABI14／wire5／IPC4／Unreal變更，不部署或完整驗收。BuffStore死亡清除與跨來源family沿既有共用契約，本輪未另測這兩條。真正護盾、回魔、減傷、下一擊增傷與生成active作者模型仍需後續實作；整體維持21/31，不將此補強列作完整對局完成。
