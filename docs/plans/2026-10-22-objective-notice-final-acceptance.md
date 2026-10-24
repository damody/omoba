# 2026-10-22 權威目標公告最終驗收

OpenSpec complete-moba-objective-notices **5/5完成**。接續17系統盤點，補上可單機驗證的建築與史詩目標公告。

## 實作

lane tower／inhibitor／nexus tower／具objective rewards的epic根怪沿committed lethal credit與既有每生命退休守門產生claim；不靠視野中的實體消失猜測。普通野怪、匿名、友軍、重複傷害與強制退休不製造敵方成功。新生命重生可再次公告。

tick結束依tick／完整entity生命身分排序，現播公告不被打斷，最多七個最近待播；每則四秒scaled遊戲時間，pause凍結、到期換下一則、空值與終局清除。狀態與序號納入replay digest，沒有英雄／營地名稱特判。

public team／kind／sequence／remaining_raw四元組跨filtered public events、protobuf optional ObjectiveNoticePresentation、CABI34 OmObjectiveNoticeHud與Unreal typed HUD。checked-in proto fallback由既有OMOBA_UPDATE_PROTO_FALLBACK建置入口更新。protobuf保留schema4與首殺欄位的相容性；CABI明確從33升34，Rust／C++ header smoke驗證24-byte／8-byte alignment與各offset，既有版本守門拒絕舊bridge。

client檢查完整／唯一／public team／種類／正序號與4秒內剩時，包含同id但截短payload的重複拒絕；非法objective notice只清該公告，不清整個HUD。Unreal再次檢查非法Blueprint metadata，實際native match strip顯示藍／紅隊摧毀防禦塔、兵營、基地防禦塔或奪取史詩目標；下一空樣本清除舊文字。使用當前權威狀態，沒有一次性音效重播。

## 集中驗收

RustV2 **1794通過／10既有ignored／0失敗**：template75、core905、codegen58、base267、backend190／1、bridge133／1、client151／8、ABI15，含generation九工作全部成功退休。

新core驗證兩隊合法敵方外塔逆序結算一致、重複致死只一次、現播不中斷、最多七待播、pause／到期換序／耗盡／終局與digest、各建築role、普通野怪／匿名／友軍／強制退休排除與實際epic致死。client涵蓋完整／缺項／重複／截短／錯scope／界線／雙隊與codec；bridge驗證typed資料跨實際frame slot到ABI34，非法／缺項／終局只清公告而保留HUD。

正式60Hz由兩名英雄在同tick用generated單體技能實際摧塔與擊殺epic，公告先藍隊外塔後紅隊史詩，pause保持四元組，四秒後換下一則，再到期與終局清空。整場雙map／雙隊filtered lifecycle、既有塔／兵線／野怪／技能一起回歸。

UE5.8 build-v2與正式後端build-v2成功，Lua三契約成功。WorldV2 **42成功**，0 Failed／NotRun，4既有警告保留；新ObjectiveNotice驗證所有類別、非法／終局拒絕、實際Widget／Slate顯示與下一空樣本清除，專用原生目標公告PNG已目視。兩map各74影格／6片段及五張既有回歸圖片也均目視；六張圖片屬本機原生fixture，非新增真連線接受。

V1 core904通過／1新fixture把外塔改NexusTower而被真實開門保護擋住；bridge新fixture猜測不存在的latest_snapshot欄位，world在map export前置編譯失敗，沒有宣稱V1 UE測試完成。全部consumer／watcher退休後改用兩隊合法外塔及既有frame_slots／published_slot，完成V2整批；失敗原始log／receipt另封存，不加總多批數字。

**3369來源／資產**接受前後集合、SHA／size／mtime一致；七項DLL staging與import library另驗。**87份證據**逐byte／SHA封存並獨立複核；全程監測 **12次**，合併11項工作監測的最大檢查間隔 **181秒**（全程單一監測因排程抖動最大183秒，原始保留）。7最終worker／watcher、4先前worker／watcher及全程monitor均退出。

## 剩餘

N01..N05見2026-10-22-objective-notice-errors.md，總登錄E1479。仍有角色施法動畫／持續引導／特殊技能幾何、召喚完整複本生命期、目標專屬名稱／音效與平衡、建築／場景美術，以及11外部ACK／LAN／OS DPI／效能缺證。原MVP33/44維持；沒有新增真網路對局、quota reset、commit／push或omfx維護。

證據索引：../acceptance/2026-10-22-objective-notice/index.json。
