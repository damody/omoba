# Lua 多層兵線建築與權威解鎖

## 本批計畫與決策

1. 接續 5.4，讓每路建築不再固定一座；先補權威規則與資料契約，不反覆執行整場／雙 UE 驗收。
2. Lua map 可選 tower_layers，依己方出生點沿該路的距離以外→內遞減宣告。缺值沿舊 tower_offset 生成一層；1..8 層、每層距基地至少300、小於 lane_length/2，相鄰至少200，非法型別／順序／容量拒絕。
3. layers 進入 map catalog JSON／hash 與既有 compiled agreement。未宣告欄位不新增序列化 null，舊 map 的規則語意保留；新增 map 仍會改整份 catalog hash，所有 peers 必須同內容重建。
4. MobaMatch 以 lane_tower_layers 保存每路每隊外→內 retirement 狀態。lane_towers／towers 是第一存活塔的相容摘要，不能取代基地解鎖權威資料；實際死亡退役後才推進，不以 HP0／ACK 解鎖。
5. 共用 structure_unlocked 同時供 NPC／Bot 目標與 moba_damage_allowed 的兩個正式 Damage 邊界使用。內塔只在前層退役後可受傷；基地仍須全路全層退役。內塔仍可防禦，不新增前端 gameplay 判斷。
6. 保留 three_lane_training 的單層行為，新增 opt-in three_lane_layered_training，三路每隊各三座（1000／700／400）；共用 route／terrain／jungle 配置，無地圖或英雄名稱分支、無專屬 C++／Blueprint graph。

## 當前功能確認

- 正式 ECS layered retirement 測試 1/1：內塔非法 lethal 不改 HP、HP0 尚未退役仍鎖、每次退役更新當路摘要、九塔退役才解鎖敵基地、己方仍鎖。
- 舊 three_lane_base_damage_requires_all_lane_towers_and_finishes_once 1/1：單層相容、唯一終局與15tick frozen digest 通過。
- Lua layer validation／hash 測試在 runtime-lua-content feature 下 1/1：容量、邊界、排序、型別與修改規則 hash。
- 三 seed 各180 ticks、60Hz 相同輸入每 tick replay digest 一致，共540 ticks／1080 steps；不是完整終局或filtered雙隊驗收。
- 發現 server 固定選舊三路圖，新增 `server.MATCH_MAP_ID`，只在 three_lane 合法，精確查 compiled catalog，並取該圖 lane_length；未設定保留舊預設。layered／未知／空／空白尾碼／非法模式的指定測試在lib與main各1/1。
- 正常build-only14378 exit0，腳本DLL／bridge重建並stage，Unreal target up to date／Succeeded；未改角色C++，0 action正常。stage bridge SHA `9425b13b3bf90193ab959317ba9ffa2e9b5daa97fde1c87d41c63d9873033ebe`。先核對Editor14716本專案command line，project scope stop一個並另確認PID退出。
- scripts/target/debug／scripts/base_content.dll／UE base_content.dll 三份目前SHA一致 `c419f1de93f0a7d6b3cacf9c1d551b9fbc02b2ab00c40ef80bb88c456aad58f9`；不包含舊omb/scripts副本。新Editor71024已啟動並保留，不另做MCP／PIE驗收。

## 選用方式

在既有合法 secure roster 的 `game.toml` 的 `[server]` 設定：

```toml
MATCH_GAMEPLAY_MODE = "three_lane"
MATCH_MAP_ID = "three_lane_layered_training"
```

此設定只改資料，仍須重建相同 catalog 的server／runtime／bridge／script peers；不是把client任意map ID送入權威。

## 剩餘範圍

- 多層地圖可由 SingleLaneConfig.map_id／server.MATCH_MAP_ID 選用；尚未將互動預設改成此圖，也未做多層真實 KCP／Unreal 對局。
- 每層沿用共用 tower HP／傷害／呈現，不含抑制器重生或特殊基地雙塔規則。這些須另有資料契約，不宣稱完整商業 MOBA 建築已完成。
- 5.4 固定種子完整批次及最後整合驗收留最後，仍20/30。
