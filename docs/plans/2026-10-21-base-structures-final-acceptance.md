# 基地攻防、超級兵與地圖間距驗收

狀態：本輪 `complete-moba-base-structure-progression` **6/6完成**。UE D:/UE5.8，固定Rust1.95.0／Lua5.4，不新增權威連線對局。完整MVP仍33/44，外部缺證11項不因本輪而勾完。

## 本輪功能

- 共享Lua的可選base_structures，含位置、最小間距、HP、重生時間、雙塔重生血量、砲車與超級兵身份；資料有界且納入內容hash。舊無此規則地圖保留原基地策略。
- 三路水晶各由同路全部線塔保護；任一水晶死亡退休後雙塔可攻擊；兩座雙塔均退休後主堡可攻擊。水晶復生立即關閉雙塔／主堡入口，雙塔復生也重新保護主堡。共用傷害、鎖定、公開admission投影及AI規則。
- 水晶300秒復生，雙塔180秒並以40%血量復生；皆依可暫停的遊戲時間，結束後不復生，重複死亡不延長期限。使用新entity生命，deadline／私有state納入replay digest。
- 敵方同路水晶死亡期間，每波一隻超級兵取代砲車；三路全破時每路兩隻。水晶恢復後後續波次回復普通組成，既有超級兵不消失。真ECS出生，不只有圖示。
- 正式兩張地圖共享幾何2倍、4800邊長，保留左下／右上基地、反向河流、14terrain與16營地32怪。射程、速度與模型保持原單位；線塔2200/1600/1000、水晶650、雙塔400，min spacing250。路線長度以有界整數弧長驗證，沒有水平距離假設。
- 兩隊六座水晶與四座基地塔接上實際Unreal世界模型；超級兵保留ID60000，水晶Tower32770、雙塔32771，不移動原角色／野怪ID。超級兵已納入原生骨架、115高度、裝備、動畫及生命週期測試。
- 一般三路的無角色計畫入口也採共用地圖兵線與波次；修正原本只有RoleBotMatchPlan啟用authored編成的漏接。跨全部map的人類roster回歸及舊單路設定保留均已驗證。

## 驗收界線

實際NPC行為由正式Rust ECS／Outcome／SimulationDriver與雙隊副本驗證。場景fixture使用真共享ABI地圖幾何與生成模型，但可見性是合成輸入，沒有權威ACK，不冒稱新連線對局。native警告、場景清理與PCM警告保留實際數字。

原MVP仍33/44、11項待外部權威／LAN／實際OS DPI／單session效能證據；本輪OpenSpec獨立計數。沿用現有模型，水晶仍是縮小城堡模型；草叢視野、史詩buff／分裂、全部技能精確AoE與正式美術仍未完成。不維護omfx，不commit或push。

## 已拒絕的證據

UE build-v1有C4458，不接受；WorldV1自動27項通過但圖像重疊，不作最終視覺接受。RustV2舊通訊邊界fixture失敗；V3整批有技能隔離／backend大小斷言失敗；BaseV4有fixture i32/i64編譯錯誤，BaseV5仍有自訂牆未跟隨隔離origin。local-v1原生206項通過，但小地圖真實螢幕取樣與理想投影的兩raw誤差使foundation失敗，不接受為全套成功。修正fixture取樣來源後，原一raw誤差上限保留，以新建置及完整local-v2驗證。全部失敗紀錄保留，詳錯誤B01–B13。

## 最終結果

| 範圍 | 最終結果 |
|---|---|
| Rust主要套件 | **1573通過／10既有ignored**：base_content246、core790、Lua authoring template58、backend190、bridge127、client147、ABI15。另dash4＋navigation1專項通過。 |
| 實際模擬 | 三種子5v5各4500tick，雙世界與雙隊副本無repair、wire有界；三種子三路完整生命週期成功結束、結束事件一次且凍結。水晶／雙塔退休、重生期限、40% HP、暫停／終局與digest由權威時鐘邊界測試驗證。 |
| Lua | owned契約140、local契約69及跨map／超級兵保留ID契約通過；正式依賴邊界4/4無runtime Lua。 |
| UE5.8 | 正式build-v5成功，UBT10.63秒；Rust／生成內容／bridge／C++沿正式建置入口完成。最後backend-v3正式authority與moba-config建置／staging成功。 |
| 完整local-v2 | **206 native**：198成功＋8帶警告成功，12 warnings，0 failed/not-run；**458cases／222intents／157stages／13cursor**嚴格phased聯集成功。 |
| Blueprint／音訊 | **69 Blueprint**，0 compiler errors／warnings／load failures；**五組真PCM**成功，音訊專項1 warning，未聲稱實體喇叭聽感已驗。 |
| 世界場景 | WorldV4 **27項成功**，26成功＋1帶4警告；兩map各74幀／6片段，真共享幾何、30建築／42小兵／32怪／6英雄呈現。已檢視修正後基地間距與左下／右上全景。 |
| 施放提示原影格 | local-v2 foundation錄影2098幀／3片段；未修改AVI，JPEG逐byte保留。`BaseStructuresCastFramesV1/frame-450.jpg`的E「請選擇可見單位」與放開／取消提示可讀；其餘擷取影格有完成施法或tooltip狀態，不都當作預覽證據。 |
| 程序與監測 | 長流程使用受擁有權驗證的180秒watcher，最終local-v2共6個開始／三分鐘／結束樣本，均持續有進度；完整workflow確認owned lifetimes退休，收尾沒有UnrealEditor或權威server程序殘留。 |

來源分兩個實際接受cohort：完整UE local-v2／WorldV4在source-v4的**3339檔完全前後一致**；完成該檢查後才修正單一`omb/src/config/server_config.rs`。精確transition證明其餘**3338個UI／共享等檔案完全未變**，唯一後端檔另跑完整190項與正式建置。最後source-v5的**3339檔亦前後一致**。沒有把最後後端改動冒稱發生在UE凍結cohort之前，也沒有用重跑無關UI取代後端驗證。

正式封存：[13份byte-identical JSON及SHA-256／size索引](../acceptance/2026-10-21-base-structures/index.json)。原始日誌／AVI／JPEG／PCM保留於`omfue/Saved/Automation`及`Saved/BuildTemp`，不提交建置暫存。錯誤詳本輪errors B01–B15、總索引E1448。

未新增權威連線對局、重設10/10、維護omfx、commit或push。OpenSpec CLI未安裝，本輪維護規格文件與勾選實作／驗收結果，未宣稱CLI validation。phased覆蓋不當成單session效能或權威ACK。
