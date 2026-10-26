# 下個月 backlog（2026-10-26 更新；標 ✓ 的項目本月已完成，證據見 acceptance-index.md）

依實際能力缺口排序：先排擋住本月 gate（G2、G3、G6）的項目，再排產品與進階能力。每一項都註明來源紀錄。狀態與證據見 [release-notes.md](release-notes.md)。

## P0：補完本月 gate

| # | 項目 | 擋住 | 內容 | 來源 |
|---|---|---|---|---|
| 1 | ✓ 斷線恢復實機驗收（split35，11/11） | G6 | 跑 split33（兩次 10 秒、一次 40 秒斷線），以 `scripts/check_netem_outage.lua` 判定；量測 KCP 退避造成的額外凍結 | errors.md E060 |
| 2 | ✓ 重新加入流程（split44，E077） | G6 | 超過 30 秒判定斷線後，client 目前直接結束、hero 留在場上。需要：同一玩家以原身分重新連線、server 依 replay 視窗或 filtered rebase 補齊、UE 從「連線中斷」回到對局 | E060 已知限制 |
| 3 | 真雙機 LAN（J2） | G6 | 第二台機器跑 client；確認防火牆、port、`--connect` 的 LAN 位址、延遲矩陣 | README §8 網路矩陣 |
| 4 | OS DPI 100%／150% 與視窗矩陣 | G6 | 由使用者切換系統 DPI 後驗收；查明 split32 截圖只有 1423x889 的原因 | execution-status J4 |
| 5 | 固定硬體效能與長時間穩定（server tick 已達標：split43 6.18 ms；剩 1080p 10 英雄 frame time 與長跑取捨） | G6 | 1080p、10 英雄、p95／p99 frame time；2 小時 soak 與「實機局 ≤10 分鐘」規則的取捨需使用者決定（例如以 headless／server-only 長跑代替 UE 長跑） | README §8；release-notes §8 |
| 6 | ✓ G2 補證（split55-single、rematch-1；L3h 仍需使用者試玩） | G2 | 1 人類席＋9 bot 的配方實機局、「重開一局」實機證據、使用者 L3h 試玩一局 | README §8 G2 |
| 7 | ✓ H02 修正後重跑實機（split50／51，4/4）；`retained_abilities` 沒有出現在技能欄（split51 p6 shot-004 目視：只有 Q／W／E／R、D／F 與飾品） | G3 | E037 修正後未重跑伊達政宗 | work-H.md H02 |

## P1：英雄量產的共用能力缺口（H03～H06）

完成這些能力後，H03～H06 才能從「部分」變成完整 kit，G3 才能從 2 套達到 4～6 套。

| # | 能力 | 需要的英雄 | 說明 | 來源 |
|---|---|---|---|---|
| 8 | 英雄 UnitScript 註冊入口 | H06 淺井長政（R 施法疊層被動） | host 綁定已有，但 `base_content::units()` 是手寫清單，package 沒有產生入口 | work-H.md H06 |
| 9 | host 冷卻調整 API | H03 阿松（T 施放後減少其他技能冷卻） | GameWorld 方法表已滿，需比照 ability_projectile 用 payload 攔截或新的窄 trait | work-H.md H03、errors.md E045 |
| 10 | 範圍內友軍查詢 `ally_area_query` | H04 阿市（T 治療、D 回復） | GameWorld 沒有範圍友軍查詢；D3 原本誤記為 supported | work-H.md H04 |
| 11 | ✓ 第 5 技能欄 `extra_action_slot`（CABI 38，X 鍵） | H04（D 被動）、H06（D 16 級大招） | 牽涉 loadout、HUD、輸入協定與 bridge ABI | work-H.md H03～H06 盤點 |
| 12 | 持續連結 `tether_distance_scaled` | H04（W 光鍊） | 連結期間持續判定，到期依距離結算 | work-H.md H04 |
| 13 | 英雄隱形與無單位碰撞 | H05 稻姬（R 融影術） | 沿用 StealthProfile 與偵測過濾；攻擊不破隱 | work-H.md H05 |
| 14 | 單點可走性查詢 | H06（W 閃瞬落點） | 目前落點緊貼目標，目標貼牆時需實機確認 | work-H.md H06 |
| 15a | ✓ 跳躍類技能公布可達距離（E090，split56） | murakami b29_quake_axe（range 0，跳躍最多 300） | E088 已公布 shape 半徑，但沒有公布跳躍距離作為 range，指示器可能畫在跳不到的位置；package 應宣告位移距離並作為 published range | errors.md E088 |
| 15 | 衝刺改為連續位移 | H02 | 目前沿用 `swept_dash` 的瞬間位移；火焰速度、壽命、最多 8 目標為假設值 | work-H.md H02 風險 |

### 需要 authored 內容來源的前端功能（E079）

英雄重製成 `b01_*`／`b04_*` 之後，前端以下 4 個功能已經沒有任何 authored 能力在使用。目前的決定是維持現狀、不製作假內容：
- 測試透過 `OmAuthoredContentQuery.h` 依功能掃描 catalog。
- 找不到對應能力時，以 warning 回報 content gap。
- widget 與 preview 改用固定的 published spec 驗證。

之後量產英雄時，優先選擇帶有這些機制的 kit，讓功能重新有 authored 來源。

| # | 前端功能 | 需要的 authored metadata | 對應測試 |
|---|---|---|---|
| 16 | 朝向陣形預覽 | `FacingFormationRows`／`Counts`／`Dimensions` | `Om.Runtime.FacingFormationPreview` |
| 17 | 受傷中斷的施法前搖提示 | `CastInterruptOnDamage` 為 true，且 `CastTimes` 大於 0 | `Om.Runtime.CastDamageInterruptionTooltip` |
| 18 | 移動型掃掠路徑（有寬度、定身不可施放） | `bRequiresMovement`，且每個 rank 的 `SweptPathWidths` 與 `Ranges` 都大於 0 | `Om.Runtime.SweptPathPreview`、`Om.Runtime.MobilityCastFeedback` |
| 19 | 被動命中扇形 | `AbilityType=passive`，且 `ConeHalfAngleTicks` 大於 0 | `Om.Runtime.PassiveConeMetadata` |


### 美術接入（Editor）

| # | 項目 | 內容 | 來源 |
|---|---|---|---|
| 20 | 6 位 authored 英雄指定正式模型 | omatsu、oichi、inahime、asai_nagamasa、murakami_yoshikiyo、tamako 的生成 class 沒有正式的 `NativeMeshAsset`，目前顯示共用人形 fallback（E093，依 id 分配 4 套訓練 rig 之一）。依 [unreal-art-guide.md](unreal-art-guide.md) 第 0 節，在 Editor 建立 `BP_<名稱>`（或使用 Project Settings 覆寫），指定模型與動畫。`Om.Runtime.PublicHeroCombatRoster` 解析 class 的方式和實機相同（art 覆寫 → content Blueprint → 生成 class），換上模型後這位英雄會自動從 content-gap warning 變成實測，不需要改測試。 | errors.md E079 |


## P2：G4／G5 與交付文件

| # | 項目 | 說明 | 來源 |
|---|---|---|---|
| 16 | G4 美術交付（工具與指南已完成；剩 #20 的 6 位英雄指定模型） | 兩種不同骨架／外觀設定在 Editor 中替換，regenerate、重開、cook 後保留，不修改玩法程式；交付 `unreal-art-guide.md` | README §8 G4、§10 |
| 17 | ✓ G5 盲測（split53，I1／I2 8/8） | 2 位未參與開發的來源英雄從原始描述產生，至少一位需要自訂 Rust handler；不支援的能力如實計入缺口 | README §8 G5 |
| 18 | ✓ `hero-authoring-guide.md` | 從描述到產出英雄的完整範例與失敗診斷（hero_author validate／build／回滾） | README §10 |
| 19 | `source-catalog.lua` | 全來源目錄、版本與能力分類；本月 kit 與盲測的對照（D3／D4 候選尚待使用者逐招判定） | execution-status D3–D4 |
| 20 | ✓ Shipping 打包驗證（package-shipping-4；profiler DLL 與 UnlitText 警告仍在） | 確認 autopilot 不被編入；處理 profiler DLL 與 UnlitText 的載入失敗 | execution-status J6、A5 |

## P3：產品化與進階能力（本月不承諾）

| # | 項目 | 說明 |
|---|---|---|
| 21 | ✓ autopilot 打法強化（E094 死亡 19→5；E098 補師存活 11→3）。剩下：autopilot 變謹慎後，quick map 比賽偶爾超過 10 分鐘，可以考慮調整地圖推進或兵線傷害 | split19 p1 死亡 44 次；撤退、推塔、技能使用的判斷仍弱，影響實機驗收效率 |
| 22 | 斷線期間輸入的處理策略 | 目前以 INPUT_EXPIRED 回報；可評估恢復後重送最後一筆移動意圖 |
| 23 | 多人產品服務 | 配對、房間、帳號、再開一局的 lobby 流程 |
| 24 | 進階技能 | 地形改變、複雜連鎖、可互動召喚物控制等，待來源索引分類後排序 |
| 25 | 更多地圖與模式 | TD 只做共用改動的回歸；新地圖不在本月範圍 |
