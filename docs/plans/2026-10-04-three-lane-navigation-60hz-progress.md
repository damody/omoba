# Lua 三路地圖與 60Hz 導航增量

## 計畫與決策

- [x] 保留單路預設，新增 opt-in `three_lane`／`three_lane_training`；沿正式 Rust ECS、secure roster 與 NPC facts，不在 Unreal 重算 NPC AI。
- [x] Lua 配置三條折線、整數座標與塔距；Rust 編譯期驗證並生成 map constants／catalog hash，runtime 不允許熱更新與 compiled map 不符。
- [x] 三路各隊分別出兵、各有一座塔；NPC 不因鄰近另一條兵線而換線，但仍可攻擊靠近的英雄。
- [x] Lua 明確 `all_lane_towers`：第三座塔經正式 Death retire 後基地才可受傷，直接攻擊／技能也由傷害邊界防護。這是原型規則，不宣稱多層塔／兵營已完成。
- [x] 固定點數 waypoint follower 保留轉角剩餘步長、量化小步不永久卡住，完成路徑後被 aggro 拉開仍回終點。這不是地形 navmesh／英雄避障。
- [ ] 固定種子批次完整對局與雙隊 filtered replica fresh 回歸完成後追加數字。
- [ ] 真正 `base_content.dll` 載入的三路 60Hz headless／每 tick replay 報告。
- [ ] 最後變更後重新生成、stage、完整 Unreal build 與 SHA 驗證。

## 已完成驗證

- 首輪導航四項測試通過；最後導航修正後 sim 全套 65 unit＋8 integration 通過。
- template-ids runtime Lua 全套 37 unit＋23 generated＋8 hero＋2 catalog integration 通過。非法 ID／重複 ID、非三路、未知 unlock、i32 最大 tower offset、錯誤端點、越界座標、重複 waypoint、過長 route／非整數／未知欄位均拒絕；地圖變更改 hash 並拒絕 hot reload。
- core 334、server 156／1 benchmark ignored、runtime 61／8 外部 capture opt-in ignored 通過。三路 server 配置保留 secure_v2_required、兩队合法 roster 與 single-lane default；shop／Recall join capability 已包含三路。
- 原型每路／每隊一兵，在三種固定種子 180 ticks 的正式 ECS trace 重放相同 digest；top／mid／bottom 位置分別正 Y／零 Y／負 Y，不沿原先單一直線。
- 首輪完整三路 filtered match（seed539365379）終了4955 tick，總4969 ticks／9936雙隊 applied steps、deaths3／1、respawns3／1，逐 tick canonical hash 無修復且完整一致，唯一 game.end、Finished後15 ticks凍結。它是非競技完成 fixture，守方以正式 MoveTo 撤離，不注入 HP／rank／塔死亡／phase。
- 基地門檻首輪測試失敗是測試未 drain 下一批 Death，修正後先驗原 Damage 被擋，再拆每座塔，第三座才解除門檻；最終全套結果待追加。
- 第一輪 `build_ue_moba --build-only` 成功，11 generated files／16 Lua inputs（原15），UE增量 build succeeded、stage f2af52e6cb3c2251a3e10ad5f51163ce96e9902cc40306f939f9ca55fcaf9db8。之後還改過導航終點與map hash，不能用此舊 stage 冒充最後版本。

## 執行入口

```text
tools/lua/lua.exe scripts/run_moba_headless.lua --profile 60 --map three_lane_training --defender withdraw --seed 42 --report D:/code/omoba/omb/target/moba-headless/three-lane-60hz-seed42.json
```

正式 server 設定 `MATCH_GAMEPLAY_MODE = "three_lane"`；保留已驗證的 `"single_lane"`／Story 預設，不自動改使用者 game.toml。headless 預設改為60Hz，仍接受15／120，但本輪不推120Hz效能。

## 尚未完成的範圍

OpenSpec 5.4 仍不勾選：缺少野區怪與重生／仇恨、地形與英雄／NPC通用避障、更多建築解鎖層次，以及 public map layout contract／Unreal map與minimap實際驗收。UE codegen仍生成共用英雄骨架，16 Lua input可讀不等於Unreal已生成三路場景；新增地圖影響權威 shared content hash，renderer美術content_hash可以不變。不得把一次 headless／filtered test當LAN、三個實際程序或完整前端畫面。

錯誤與避免方式記在 `docs/plans/unreal-moba-error-register.md` E126。未清理既有 dirty tree，未 commit／push。
