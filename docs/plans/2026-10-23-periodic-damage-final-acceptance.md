# 持續傷害生命期與遊俠流血：最終驗收

本批 `complete-periodic-damage-lifetime` 完成4/4 tasks。先結算有效生命期再移除到期buff，按 `min(dt, remaining)` 積分並保留每個buff的次量子餘數；目標／來源key與到期通知順序固定。移除即停止，刷新保留同來源餘數，更換來源世代／傷害種類／profile不繼承前筆餘數。來源ID、generation、profile及DPS使用有界檢查，不做窄化截斷。既有塔魔法DoT payload維持相容。

新增共用Lua `damage_over_time_enemy` 效果，逐rank DPS／duration／range、敵我條件、重複宣告及完整副作用預檢均有校驗。原創遊俠穿風箭增加每秒10/15/20/25物理流血、持續4秒；同能力／施法者生命刷新，不同來源分別結算。資料驅動的技能說明列出逐級數值。共用generic_dot來源與已檢查的buff family綁定公開「流血」，與同箭減速分開；公開visual state只含catalog ID和剩餘時間，隱藏內部來源key／payload。沒有英雄名稱執行特判或C ABI layout變更。

## 集中驗收

| 最終V2套件 | 通過 | 既有ignored |
|---|---:|---:|
| script ABI | 15 | 0 |
| content model | 42 | 0 |
| templates（runtime Lua lib81＋integration34） | 115 | 0 |
| UE codegen | 68 | 0 |
| core | 941 | 0 |
| base_content release | 304 | 0 |
| bridge | 135 | 1 |
| client runtime | 151 | 8 |
| 合計 | **1771** | **9** |

0 failed；完整base suite182.71秒。新增四rank正式60Hz input測試驗證瞬間命中、公開流血與減速、pause freeze、護盾吸收、完整四秒結算及到期不再扣血；另一正式input驗證失敗不附加DoT、顯式移除立即停止。核心測試覆蓋15/60/120Hz分數DPS與部分到期、刷新／移除、來源世代隔離、非法資料和穩定順序。既有完整模擬及過濾複本回放全部成功。

正式backend/moba-config建置與部署成功；UE5.8標準build成功，部署DLL／import library SHA一致。World66：65 succeeded＋1 succeeded with warnings，0 Failed/NotRun；4個warning均為既有Om.Generated.RuntimePresentationReset。另有1項compiled-map公共ABI export成功，不重複算進上述1771。

29PNG：25張與前批接受像素完全一致；recall進度、layered map、firstkill、summon四張已明確view。地圖04:32/KDA 0/0/0保持；既有首殺下／右HUD裁切及部分hero/focus名牌重疊仍在。這些是呈現回歸證據，流血權威行為由ECS／正式input測試證明，沒有宣稱新流血專用VFX／圖示已製作。

## 原始失敗與範圍

V1 base302通過／2舊fixture失敗，各差170 raw HP，正是10 DPS在17/1024秒的一個tick。保留失敗日誌和10份原始來源，所有reader退休/reap後才修正：魔力／CD fixture納入持續傷害，四技能獨立fixture在重設HP時也重設目標buff。原有魔力／CD／直接傷害斷言保留；新增完整生命期fixture V1/V2均成功。V1未啟用runtime Lua feature，template只跑42，最終V2重新啟用並接受全部115。

本批只接受raw DoT生命期及既有傷害packet／shield語意。審查發現既有直接script／DoT Outcome::Damage未經基礎護甲／抗性，而DamageInstance已先算過；兩者predeclared都為false。下一批優先建立明確raw／settled共用結算，禁止全域重算造成雙重減傷。本批零防禦fixture不代表護甲／抗性已對齊LOL，raw跨tick頻率相等也不等於任意post-modifier HP rounding相等。持續引導／獨立地面區域、完整combat replica及11外部父項繼續pending，MVP33/44不變。

## 證據與程序

165份證據／29PNG、3425輸入一致、health 13 samples／max171秒。3425正式輸入exact set/SHA/size/mtime接受；165份封存bytes/SHA及原始檔、29圖片雜湊均獨立通過。6 final workers＋2 historical workers的owner／watcher creation token均核對退休。兩個health timer均退休；v1前綴錯誤樣本保留但不作本批進度證據，v2為有效170秒全階段timer。所有exec sessions 93562/84139/70199/39005/1439/11848/2172/40325/48576/46339/37849已reap。最終CIM無本批Lua/Cargo/rustc/cl/link/UE程序。

[證據索引](D:/omoba/docs/acceptance/2026-10-23-periodic-damage/index.json)；[錯誤／根因／預防](D:/omoba/docs/plans/2026-10-23-periodic-damage-errors.md)。沒有新增真網路對局、quota reset、commit/push或omfx維護；原48h deadline2026-10-25 00:33:38 Taipei不變。
