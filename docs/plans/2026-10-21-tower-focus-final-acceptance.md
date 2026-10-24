# 全遊戲盤點、塔持續攻擊與冷卻明細：最終驗收

OpenSpec add-moba-tower-focus-and-cast-timing **5/5完成**，未新增連線對局。

## 全遊戲盤點

已完成[17範圍盤點](2026-10-21-whole-game-completeness-audit.md)，逐項區分接線證據、單機缺口與外部接受，非逐行程式證明，不宣稱整個遊戲已與特定LOL版本等同。

## 本輪實作

一般塔與主堡塔各自focus，第一擊base，對同英雄後續每擊+25%、上限+100%，3秒未出手重置。SingleLaneConfig可配置，零cap停用，setup原子驗證。換英雄、失去合法目標、轉小兵、目標死亡／退休與新spawn清理，pause沿模擬clock凍結。Policy與state入digest、不入公開wire，沿原damage／護甲／歸因流程。這是已承諾instant攻擊，不是投射物命中ACK；純規則及實際ECS驗證兩種塔、小兵不承接、reset／pause／死亡與digest。

技能tooltip新增實際公開冷卻剩餘／總長：ready保留total、零冷卻明確、未學習不顯示，非法snapshot沿既有validation清理；保留魔力、目標、射程／落點與控制模式，不攔截提交、不預測ACK。

盤點發現cast_time主要為metadata，尚無全技能權威排程，因此撤回準備時間payload／顯示，沒有把宣告值冒稱功能。通用cast scheduling、移動／CC／source退休中斷列優先缺口，連同建築獎勵、史詩怪效果／分裂與特殊技能幾何接續。

## 完整驗收

Rust **1659通過／10既有ignored**（core818）；World **30項**成功；完整local-v1 **209 native、458案例／222 intents／157 stages／13 cursor、69 Blueprint、五段實際PCM**成功。native為201 Success＋8 SuccessWithWarnings／12 warnings；Blueprint 0 compiler errors／0 warnings／0 failed loads。

RustV4：template59、core818、codegen57、base246、backend190/1、bridge127/1、client147/8、ABI15；9工作含生成均exit0並退休。Lua local69／owned140及map ID契約成功。UE5.8 build-v2成功，UBT61.65秒，backend-v1成功。World為28 Success＋2 SuccessWithWarnings／5 warnings，0 Failed／NotRun；兩map各74影格／6片段，背景HTTP timeout警告保留。

**3341來源／資產**接受前後集合、SHA、size、mtime一致，derived DLL及import library另依current build／接受前staging核對。**20份原始JSON＋1份盤點MD**逐byte封存並獨立複核；完整流程8次健康取樣，180秒排程UTC最大181秒，另持續讀最新log；**13個owned工作與13個watcher全退役**。

## 畫面與接受界線

已目視overview、layered HUD、camp、fog與實際SOmHudToolTip／BuildAbilityTooltip元件圖。冷卻3.5／12秒、魔力37.5、距離600／半徑220與操作說明可讀；圖為owned測試視窗使用fixture公開值繪製，非真對局hover。塔行為以正式ECS驗證，不拿靜態地圖圖代替。

原MVP仍**33/44，11項外部ACK／LAN／實際OS DPI／固定效能待驗**。phased coverage不等於單session效能／真ACK。未新增連線對局、重設配額、commit／push或維護omfx。

RustV1／V2失敗、V3通過但覆蓋不足、舊UI build-v1／baseline-v1均保留。最終依RustV4、build-v2、WorldV1、local-v1、source-v2。詳[錯誤F01–F08](2026-10-21-tower-focus-errors.md)與[原始封存](../acceptance/2026-10-21-tower-focus/index.json)。OpenSpec CLI不可用，未宣稱CLI validate。
