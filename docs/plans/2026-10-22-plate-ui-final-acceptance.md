# 2026-10-22 塔皮分段名牌最終驗收

OpenSpec complete-moba-tower-plating-presentation **5/5完成**。接續17系統盤點，補塔皮的玩家可見層數與期限。

## 實作

獨立tower_plating_status native Buff公開health_segments={total,remaining}，本身沒有戰鬥加成。有效外塔第一次active tick即提供5/5，未受損也可見；剩餘由權威不可倒退ledger決定，治療不恢復已破層數。有限期限、退休／到期／結束清除，既有護甲與金額政策維持。

沿ABI33既有公開Buff payload與catalog，Unreal共用有界decoder驗證total1..16、remaining0..total、有限正期限、payload4096與字串邊界。只接精確公開存活generation，相同target多來源衝突拒絕。每frame只建一次map，原始UTF8 marker先篩選，沒有候選就不做JSON或entity集合，活體集合只保留候選key。完整baseline清除缺項，control-only保留。

typed FOmHealthSegmentState沿FrameState到實際Actor與原生名牌。分段列搭配數字「塔皮3/5」與權威剩餘秒數，與HP條獨立，HP回復不會誤導成塔皮恢復。普通名牌不建立cell，首次有效資料才建立最多16格並重用；Actor／Widget以顯示ceil秒數快取，避免同秒每frame寫Slate。血條偏好、可見性預算與退休清除延用，沒有renderer自行推算deadline。

## 集中驗收

V1 Rust全數通過；UE V1於測試列舉Hostile不存在而編譯失敗。前批Rust／UE／後端／Lua consumer與watcher均退休後才改Enemy，並完成延遲配置／整秒快取修正；V1 Rust與UE失敗原始紀錄封存。

RustV2 **1770通過／10既有ignored／0失敗**：template72、core892、codegen57、base263、backend190／1、bridge131／1、client150／8、ABI15。含generation九工作exit0且退休。新增core驗證未受損5層、無額外加成、破層後治療仍3層、到期／退休；新增ABI驗證初始metadata bytes、有限期限、剩餘變更事件與移除。正式60Hz既有塔皮技能／錢包／減傷案例擴充初始status、公開remaining與到期清除，其他玩法與完整map／雙隊digest／終局凍結一併回歸。

UE5.8 build-v2成功（UBT **81.22秒**）、正式後端V1與Lua三契約成功。WorldV1 **38成功**（37 Success＋1 SuccessWithWarnings，0 Failed／NotRun），4則測試警告記錄保留。

新增HealthSegmentsPresentation原生測試：非法JSON／型別／整數／超界／期限拒絕，公用ABI字串／catalog／generation／衝突守門，實際Bridge→Actor→Widget，control-only保留／完整缺項清除／退休，普通名牌0 cell／首次16格／重用／同顯示秒不重寫、偏好與死亡清除。專用原生Slate PNG已目視確認分段格、3/5與剩120秒；另兩map與既有首殺資訊列三張PNG回歸，兩map各74影格／6片段。專用截圖為實際原生widget fixture，不宣稱新增真對局畫面接受。

**3360來源／資產**接受前後集合、SHA／size／mtime一致，七項staging與import library另驗。**67份證據**逐byte／SHA封存獨立複核；三分鐘監測 **9次／最大181秒**。7個最終worker／watcher、2個V1 worker／watcher與全程monitor均退出。

## 剩餘與界線

錯誤U01..U06與修正收於2026-10-22-plate-ui-errors.md，總登錄E1474。MVP33/44與11項外部ACK／LAN／OS DPI／效能待驗維持；沒有新增網路對局、重設配額、commit／push或omfx維護，沒有引用前輪native／IPC／Blueprint／PCM數字冒作本輪接受。

仍需更完整建築／場景美術、野怪分裂、經濟補償、更多目標公告、技能引導／特殊幾何與動畫、召喚複本生命期。本輪完成不代表整個遊戲已達LOL全部規則與內容。

證據索引：../acceptance/2026-10-22-plate-ui/index.json。
