# 2026-10-22 場上護盾名牌最終驗收

OpenSpec complete-moba-shield-nameplates **5/5完成**。沿整體17系統盤點，補護盾在場上單位的可讀性；正式範圍維持omb／共用Rust／scripts／omfue。

## 實作

原內部__damage_shield改為damage_shield，追加native-only公開Buff catalog。儲存仍是一個有界效果，既有護盾上限100萬、期限60秒、強者保留、部分吸收／耗盡／到期／死亡政策維持。既有可見entity Buff snapshot與catalog hash承載shield_remaining_raw，不用HP推算、不變更CABI33。owner economy護盾值仍保留且與公開資料一致。

Unreal共用decoder驗證payload4096、JSON數字整數、raw1..1024000000、明確除1024與有限正期限；Blueprint typed FOmShieldState二次檢查。Bridge在原始UTF8 marker篩選後才解碼，只接registered catalog與公開存活精確generation；字串越界、未知catalog、多來源衝突拒絕。完整baseline缺項清除、control-only保留；重置清map。

typed frame經Actor送原生名牌，獨立淺藍護盾條相對max HP、視覺封頂1；數字保留完整護盾量與權威ceil秒數，HP條不變。沿原有名牌偏好／可見性budget／退休。整秒顯示快取避免同秒Slate重寫；無護盾時連固定高度容器一併收合，不改普通名牌尺寸。

## 集中驗收

RustV2 **1776通過／10既有ignored／0失敗**：template73、core895、codegen57、base264、backend190／1、bridge132／1、client150／8、ABI15；含generation九工作exit0且退休。

新增core驗證公開catalog與payload100→60、較弱重給保留60、超額傷害回傳餘額、耗盡移除及到期；新增ABI測試確認部分吸收bytes／Updated、generation換代OwnerRemoved與到期Removed。既有正式60Hz generated道具購買／施放案例擴充public snapshot100→60→0、到期／死亡，與owner economy相符；既有護盾減傷順序、不治療、整場雙隊filtered lifecycle等一併回歸。

UE5.8 build-v2成功（UBT **3.64秒**），正式後端與Lua三契約成功。WorldV2 **39成功**（38 Success＋1 SuccessWithWarnings，0 Failed／NotRun），4既有測試警告保留。新增ShieldNameplatePresentation驗證解碼型別／界線／期限、公用ABI bounds／catalog／generation／衝突／死亡、實際Bridge→Actor→Widget、control-only／完整缺項／退休，原生數字／偏好／死亡／非法Blueprint／同秒快取；專用原生PNG已目視確認護盾條、300與剩3秒及獨立HP條。

兩map各74影格／6片段，另兩map／既有首殺／塔皮名牌四張PNG回歸；五張原生圖片均目視。專用護盾截圖為原生widget fixture，不宣稱新增真連線畫面接受。

**3363來源／資產**接受前後集合、SHA／size／mtime一致；七項staging及import library另驗。**79份證據**逐byte／SHA封存並獨立複核；三分鐘監測 **8次／最大181秒**。7個最終worker／watcher、6個V1 worker／watcher與全程monitor均退出。

## 剩餘與界線

V1橋接測試因OwnerRemoved列舉名稱錯誤而編譯失敗，世界export因此未執行；全批退休後修為BuffOwnerRemoved，完成V2完整重跑與原始失敗封存。S01..S05記錄公開身分接線、查詢quote修正、單位／期限及無護盾容器收合。錯誤MD為2026-10-22-shield-ui-errors.md，總登錄E1476。仍需經濟補償、更多目標公告、持續引導／特殊技能幾何與動畫、召喚複本生命期與建築／場景美術；原MVP33/44與11項外部ACK／LAN／OS DPI／效能缺證不變。沒有新增真網路對局、重設配額、commit／push或omfx維護。

證據索引：../acceptance/2026-10-22-shield-ui/index.json。
