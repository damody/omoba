# 多人 roster、玩家 HUD 與真實助攻（2026-10-04）

## 本段計畫與決定

1. 保留預設 1v1 與初始兩個玩家欄位，新增 explicit `additional_players`；每隊最多五人，不擅自切換一般啟動模式。英雄 roster 改 Vec，每 slot 保存 side，玩家 index 不能再索引基地或隊伍。
2. 在權威 world 改動前驗完所有 player ID／team／hero；server secure V2 authenticated bindings 依 BTreeMap 穩定順序選各隊首名，再加入其他玩家。全隊目前仍用 training_luminary，選角與三英雄原型不是此段成果。
3. 三個 private HUD namespace（respawn、Recall active、Recall remaining）以低 32 bits 保存完整 player ID。team audience 不變，filtered 清命令必須匹配 `(team, player)`；runtime HUD 只查 configured player。Recall protocol 升 2，拒絕旧 version 1，C ABI 保持 7。
4. spawn、death、respawn、shop、Recall 基地、助攻 roster、owner economy 全部明確區分 side 與 player slot。額外同隊出生 Y 按 roster rank 間隔 60，保持原始兩英雄位置與 entity 建立順序。fixture Push／Guard Bot 依 side policy 走每個 slot，不將此稱作完整五位置視野 Bot。
5. 實際 ECS 測試兩名助手（重複命中去重，其中一人死亡）與 hero killer；100 Gold／300 Gold、唯一死亡、dead persistent gold、重生後 Gold／assists、兩次 60Hz replay digest 全部核對。
6. 三英雄 640 ticks 雙隊安全投影驗收：player 3 Recall 不阻擋 player 1 Move；回自己基地，死亡後取得助攻，兩人重生。每步完整 bootstrap hash 一致，無 ComponentRepair；敵隊看不到 owner metrics／助攻金錢。

## 已完成驗證

- base_content 全部 84 passed；末次额外英雄 script preflight 驗證單測另通過。
- core 329 unit passed；完整測試另含 3 integration passed（包含兩輪 TD 1–100 autoplay），1 doctest passed／1 ignored。最後 core --lib 再跑 329 passed。
- server 154 passed／1 ignored；包含 60Hz 5v5 authenticated roster 組態及超量／零 ID／缺隊／unknown team 拒絕。
- runtime 55 lib passed／4 opt-in ignored，3 main passed；同隊 Recall／respawn、完整 u32 player ID／unknown owner 不互相覆寫。
- bridge 50 lib passed／1 opt-in ignored，2 legacy driver integration passed／1外部server opt-in ignored；Fyrox check --tests 通過。
- 真實 KCP `moba-runtime-1791076242`：60Hz，server＋三個外部 runtimes（1/team1、2/team2、3/team1）。每人由既有 renderer intent 測試入口送正式 Move，全部位移；兩隊 9 checkpoints 至1080，player3 獨立 timeline 與 authority expected hash 精確比對 8 checkpoints 至1080，pre=post 無修補。safe tick1091。四個 owned PID90976／25208／86636／88120 均已清理並另用固定 Lua inspect 核對。
  - 該次腳本開始時 report.kind 仍為舊 two-runtime label，實際 processes／roster／獨立 manifest 是三人；已將 opt-in 報告標籤改 three-runtime，不回寫原始證據。
- Unreal build-only28589 exit0，生成11 files／15 Lua，class surface hash de9c7fcfc98d6479 不變；cbindgen header／ABI7 不變，codegen --check 通過。
- built／staged bridge SHA-256 `39f10b82752d0578fde317e637d1f5dd1c1a407c5c207a3bb1c4e8a8bea2c476` 一致。此段沒有開 Editor／PIE，不借前次 Editor 證據宣稱新多人 UE 已驗收。
- scoped diff --check 與 OpenSpec strict validate 通過。錯誤與防重犯見 E110。
- protocol2 真實回城回歸 `moba-runtime-1791076358`：60Hz兩隊8秒 channel皆完成回自己基地，safe1320，兩隊各10 checkpoints至1200，runtime-renderer-intent測試注入而非UE按鍵；三PID86300／16764／39752清理及獨立Lua inspect通過。此段不冒充新增UE B鍵驗收。
- 最終 Lua metadata 修正後重跑 `moba-runtime-1791076498`：kind明確three-runtime，success／cleanup_verified true，三玩家實際Move、雙隊9／第三玩家獨立8 checkpoints至1080、safe1092再通過；四PID86380／89648／8092／64972另行inspect不存在。使用此最後版報告交付，不修改前次證據。

## 邊界及後續

此段已有真實多人 ECS 助攻與三個 runtime 正式網路移動／parity，不等於真實 KCP 玩家施法造成助攻，亦不等於 5v5 Unreal 對局。經驗完整規則、擊殺／助攻計分 UI、三路、野區、完整五位置 Bot／100場、兩機 LAN 及 frame-time gate 尚未完成。5.3 保持未勾選，總進度19/30。

下一步讓玩家 roster 可選英雄，經正式 KCP 攻擊／技能產生一次可核對的助攻，將 authoritative kills／assists 投影至通用計分板；同時保留所有玩家 owner HUD 與逐步 hash 防回退，不新增角色 C++ 或 Blueprint graph。
