# 舊物品入口收斂進度（2026-10-05）

## 計畫與決定

1. 保留正式 MOBA 只能走 authenticated lockstep PlayerInput；舊 InboundMsg.name 不是驗證過的身分，不把名稱查找冒稱授權。
2. 非 MOBA 相容入口只解析六格無號整數槽位、精確唯一英雄名稱、非零且唯一 owner、存活英雄；不使用第一英雄 fallback，不將大整數截斷。
3. 效果、冷卻與核心准入一律委派 handle_item_use_from_input，移除 Shield 回血、Sprint 永久移速及其他三種 log-only 副本。失敗不發布 completed。

## 實作與局部確認

- `omb/src/state/resource_management.rs::use_item` 已收斂；公開與私有入口都拒絕正式 MOBA 舊指令。
- 五種主動效果成功、第二次冷卻拒絕；HP／基礎移速不被假護盾或永久衝刺修改。
- 空／未知／重名、重複 owner、負／超界／超大／小數／字串槽位拒絕且不改護盾、冷卻或發布成功。
- 真正 setup_single_lane_match 建立的 MobaMatch 資源配合法定可用英雄，驗證兩個入口均先拒絕正式模式。
- `cargo test --manifest-path omb/Cargo.toml -p omobab --lib --no-default-features --features kcp,compiled-content-only legacy_item_adapter -- --nocapture`：3/3 通過，無編譯／測試失敗。既有 td_rounds 與 protoc fallback 警告未改動。

## 邊界與後續

- 其他舊技能／買賣入口的 find_hero_entity fallback 未在本輪變更，不宣稱全後端名稱 fallback 已移除。
- 接續主動物品 build-time 作者契約與生成資料；UE 呈現、LAN／完整對局與部署驗收留最後。無 runtime Lua、hash／ABI／wire／IPC 或 Unreal 改動，整體仍 21/31。
