# MOBA 營地、地形與施放提示擴充

2026-10-21，Asia/Taipei。`D:/omoba`，UE `D:/UE5.8`。本輪在不新增權威連線對局的範圍增加實際功能；不以重複核對舊11項缺證作為完成。

## 已實作

- 技能／道具／召喚師／ward共用施放提示：公開射程圓、點施放方向線／箭頭、落點、射程內外顏色與距離、缺少可見單位／落點、QWER未學習／冷卻狀態，以及實際施放鍵與確認／取消操作。深色底板在落點附近，依目前玩家的physical viewport轉換並限制在視口內；使用實際paint scale補足至少12physical px字級、尊重文字偏好、依量測換行／行距及底板高度。
- 提示共用既有preview與owner生命期，取消、死亡、失焦、modal、來源退休不保留舊標記；沒有改變原本三種施法模式與提交契約。單位型只表示指向可見候選，不把地面游標距離當成單位中心、也不猜測隊伍／命中或未公開的技能碰撞形狀。
- 共享Lua營地members：兩份地圖各16營地、32隻怪，狼／鳥／石怪營地可有多個成員。成員獨立HP、攻擊節流與擊殺獎勵，共享群組仇恨及重置；清空最後一隻才開始整營重生。
- 固定點patience→軟返回→硬返回。營地範圍內攻擊可重接軟返回，外部攻擊不造成傷害；硬返回免傷、較快回家並恢復。私有狀態與全體成員位置／HP納入重播雜湊，不向客戶端公開仇恨或計時器。
- 12塊鏡射牆群，加既有2塊共14個terrain，形成實際野區碰撞／繞路選擇。authoring驗出生、lane、主怪與附屬怪本體不被牆擋住。fog／溝通bounds納入成員出生點。
- 共享基地解鎖策略編成到Rust；正式地圖任一路三層塔全部退休即可突破，保留舊all-lane策略。傷害、索敵、公開結構狀態使用同一政策。
- 附屬怪使用40000起獨立保留ID，新增map不改既有普通或附屬ID；有界schema與catalog角色衝突拒絕，runtime不按怪名硬編分支。

## Rust／工具結果

| 範圍 | 通過 | 既有 ignored |
|---|---:|---:|
| base_content release | 246 | 0 |
| core | 787 | 0 |
| backend | 189 | 1 |
| Unreal bridge | 127 | 1 |
| client runtime | 147 | 8 |
| script ABI | 15 | 0 |
| template authoring | 57 | 0 |
| 合計 | 1568 | 10 |

最新證據為 `Saved/BuildTemp/completeness-base-full-v3.log`、`core-full-v2`、`backend-v2`、`bridge-v2`、`client-v2`、`abi-v1`、`template-v4`。Lua owned140／local69與跨map普通／附屬ID專項成功。

真模擬驗小怪受擊整營接戰、主怪先死不提前重生、分別獎勵且重複lethal不重複給獎、整營新entity重生；驗軟返回重新接戰、外部傷害拒絕、硬返回免傷及到家滿血。三種子正式5v5各4500tick、雙authority世界及雙team副本，每frame hash與零repair；三種子營地3000tick與完整三路結束／凍結生命週期亦通過。這些是記憶體模擬，並非新權威連線對局。

## Unreal 接受狀態

UE5.8最後建置 `completeness-ue-build-v3.log` 成功、exit0、UBT8.38秒；前兩版78.84及8.03秒成功。正式backend debug亦已建置與staging成功。

最新場景 `Saved/Automation/CompletenessWorldV3/summary.json`：27項成功，0failed／not-run，其中一項帶4個測試world退休警告。除既有26項場景，另驗施放提示及.5／.6667／1／1.5／2 paint scale的字級下限。每張map各74幀／6段視窗錄影；牆群、三路與群怪已檢視。完整scope明示synthetic visibility／代表位置、零simulation與authority ACK。這份證明呈現，NPC行為由上面的真模擬驗證。

local-v1完整206 native／兩段操作／69Blueprint與五PCM成功，但原始AVI顯示10字級經paint scale後過小；因此它是被後續可讀性修正取代的接受，不作本輪最終UI畫面。擷取工具曾文字模式寫JPEG，失敗副本排除，原始AVI未修改；後續擷取逐byte核對，見C08／C09。

最終 `completeness-20261021-v2` **完整本機驗收成功，exit0**：206 native（198 Success＋8 SuccessWithWarnings，共12警告，0 failed／not-run）、458 cases／222 intents／157 stages／13 cursor observations的phased完整coverage、69 Blueprint（0 compiler error／warning／failed load）及五段真submix PCM成功。音訊專項另有1警告，實體喇叭聽感不在本輪接受範圍。所有本輪editor／publisher已退休，無殘留該輪引擎或fixture程序。phased不宣稱單session全流程效能。

已檢視最終原始AVI的800×600影格：`CompletenessCastFramesV4/frame-560.jpg`（缺少可見單位、橘色）、`frame-600.jpg`（已指向單位、放開確認）、`CompletenessCastFramesV5/frame-680.jpg`（左鍵確認）。文字與底板已可讀；六張與另外五張影格均由原始MJPEG逐byte擷取、保存來源segment／frame／size／SHA。這是本機fixture實際畫面，不是新對局ACK，也不冒稱所有OS DPI／裝置均已視覺驗收。

來源／資產前後 **3338個檔案**的SHA-256、size、mtime與file-set完全一致，`completeness-source-verified-v2.json`成功。三分鐘phase監測共8個樣本（含開始／完成），控制序號、案例日誌與產物持續前進；各長測試／建置亦有獨立監測，短工作流的綁定競態明示在C06。

封存 `docs/acceptance/2026-10-21-moba-completeness/index.json`：10份JSON副本逐byte相同，含最終／被取代的v1接受、來源指紋、compiled map、兩map錄影摘要及兩份原始MJPEG影格索引；另有Rust與工作流log的真實SHA／size及健康摘要。封存工具exit0；本輪OpenSpec 6/6完成。

## 範圍界線

目標是增加通用可維護規則，不宣稱複製完整現版LOL。仍沿用既有模型，未包含完整inhibitor／nexus雙塔及其重生、史詩buff／野怪分裂、草叢視野、所有技能精確AoE命中形狀或實體控制器／OS DPI／LAN／單session效能證據。歷史MVP清單仍33/44，11項外部缺證不冒勾。

參考Riot歷史軟重置概念：[12.22官方說明](https://www.leagueoflegends.com/en-us/news/game-updates/patch-12-22-notes/)，本專案採可調訓練規則，沒有宣稱現版數值相同。失敗與修正見 `docs/plans/2026-10-21-completeness-errors.md`；未修改omfx、未新增權威連線對局或重設10/10、未commit／push。
