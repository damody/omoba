# Jungle Bot 局部隊友支援（2026-10-05）

## 計畫與決策

1. 推進五位置 Bot 的實際作戰缺口，不繼續增加魔力策略細節。Jungle 原本普攻與技能只選neutral kind3，改加入通用局部支援，不按hero ID、技能slot或特定營地分支。
2. 共用 `jungle_assist_target` 只接收目前本人位置與已提交的隊伍 `SeenUnit`。候選為550距離內的已披露、存活、非neutral敵方英雄；候選附近550內至少有另一位存活同隊英雄，且同隊觀察人數＋本人不少於可見敵方英雄人數才支援。
3. 用owner_player_id排除本人，不能把自己當成支援隊友；同一位置的其他真人或Bot仍算隊友。敵人血量較低優先，再依距離與canonical生命ID穩定決勝，來源順序不影響結果。
4. 人數只是目前可見觀察，不保證隱藏敵人不存在或戰鬥一定安全。不讀敵方authority Pos／HP、私人仇恨／營地重生，缺隊友披露、不利人數或死亡時仍使用原公開巡野／野怪戰鬥流程。
5. 先掃一次可能很大的兵線perception，只保留1100範圍內hero子集做局部人數計算。正式MOBA roster最多10人，不在每個敵方hero上重掃所有creep；沒有新增每frame UI操作。
6. 普攻與技能共用支援focus；Jungle技能可選該已披露存活敵方英雄或原野怪，在技能合法range內優先focus。unit／point approach／area中心都沿原作者意圖、正式CD／成本／學習／四槽檢查；缺少focus披露時不能由ID重建目標，其他位置策略維持不變。
7. 仍只發出一般CastAbility／AttackTarget，原回城安全、療傷、購買、升級优先序不改。看見隊友交戰即可支援，不需要額外Lua技能、角色C++或Blueprint圖表。

## 當前功能確認

- core `--lib jungle_assist`：2 passed。本人不能冒充隊友、死亡／遠距／異隊／非hero隊友不准入，可見劣勢不支援、穩定決勝／順序不影響、保留野怪fallback；技能偏好focus且死亡／友方／超range／隐藏focus不合法。
- 同一技能案例追加point approach／area中心確認後單點通過：有focus選敵方hero座標，無focus回到原neutral座標。仍是同一案例，不另增加測試數。
- base_content `--lib jungle_assist`：1 passed，正常Lua generated manifest、Production60Hz正式輸入。三人fixture（前排Jungle／敵方術士／同隊真人遊俠），滿HP／零護甲隔離傷害；slot0破岩擊使目標10000→9935、正常CD生效，普攻也送相同enemy的AttackTarget。
- 同一正式案例：修改未提交的同隊authority Pos不改Bot輸入；把隊友從current disclosure移除後，即使baseline快取仍在也不能授權支援；恢復current後正式施法成功。沒有直接改玩法pool／CD／命令來製造施法成功。
- 直接相關既有core `--lib role_bots` 6 passed，沒有跑整crate suite或完整對局；本輪共3個新功能案例＋6個原策略相容案例，不把同一案例內assert數當額外測試。
- 沒有新編譯／功能失敗，既有td_rounds warnings保留。編輯時先產生唯讀Bot inputs再呼叫mutable driver step，避免在同一argument list交疊World借用；不為此修改runtime時序。
- root／omb／omfue whitespace檢查成功，保留既有工作樹、沒有提交或清理產物。

## 未完成範圍

這是局部可見隊友支援，不是跨路策略性gank／安全記憶、戰力或彈道預測、精準補刀或完整保護技能。五位置完整平衡、100場、UE／LAN／重連／最後效能驗收仍待；5.5不勾選，整體20/30不變。Unreal共享引擎E177限制沒有繞過。
