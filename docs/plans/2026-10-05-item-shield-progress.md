# 通用真正護盾進度（2026-10-05）

## 計畫與決定

1. Shield從瞬回HP假實作改真正限時吸收；零秒不作相容回血fallback。既有fixture改1秒並驗證HP不變。
2. BuffStore保存單一私有shield_remaining_raw，授予取目前餘量／新量最大值、時間沿add刷新；不疊加多道具盾量，不新增entity timer。
3. 正常非TD-layer Damage在共用modifier結算後吸收；ScriptDirectDamage也吸收，不將pure或direct偷偷繞過盾。
4. 全吸收有效hit中斷回城，HP attribution只在真正扣血時記錄。TD-layer branch保持既有atomic pop規則，不擴充盾。

## 實作與局部確認

- grant_damage_shield／shield_remaining／absorb_damage_with_shield通用API；有限正amount至1000000、duration1/1024–60秒，stats／store與cooldown預檢。
- `item_shield_60hz`1/1：正式買用100盾不回血，120raw先50%減傷，盾餘40；50direct耗盾40再扣HP10；合法重新授予、較弱refresh不疊量，20步到期後正常扣血；全吸收hit中斷正式Recall。
- `single_lane_buy_use_sell_keeps_mixed_item_input_order`1/1：HP100保持、盾50存在、gold與slot順序維持；已移除舊回血150錯誤期待。
- `item_active_rejection`1/1：原7情境＋Shield非法amount／負量／0時間／無限時間4情境，不改CD／命令，無待用增傷或盾量。

使用`cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only <filter>`。本輪無編譯／test失敗，已有td_rounds警告保留；工具glob錯誤記E260。

## 待辦與邊界

- 調查發現omb/src/state/resource_management.rs::use_item仍重複MVP，須核對模式准入後收斂到共用executor；此輪未修改，不冒稱全入口已完成。
- generated active作者模型／Lua道具資料與UE盾量呈現、完整死亡重生、TD-layer盾尚未處理；未把私有盾payload作公共visual事件。
- 無runtime Lua／角色C++／Blueprint graph／內容hash或ABI14／wire5／IPC4變更，不部署或全驗收。整體仍21/31。
