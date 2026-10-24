# 2026-10-22 通用Buff層數最終驗收

OpenSpec complete-moba-buff-stack-presentation **5/5完成**。延續全遊戲17系統盤點，補上單台電腦可驗證的玩家狀態資訊。

## 實作

權威隊伍Buff新增通用JSON欄位stacks，既有objective_stacks維持。限時／永久規則、數值與封頂由原有lease控制，沒有UI自行推算或依營地／英雄ID特判。沿原有ABI33 payload字串與hash／更新事件傳遞。

Unreal共用有界decoder：空payload／缺欄為單層，stacks只接受1..65535整數；非法JSON、非數字、非整數、超界、超過4096 bytes與越界string ref拒絕。完整owner列表先篩選hero／generation／duration才解析，typed row StackCount在widget再次驗證。複數顯示「名稱 ×N（剩餘時間／持續生效）」，單層文字與diagnostic getter維持相容；只改層數也刷新player快取，空baseline／換代仍清除。

## 集中驗收

RustV1 **1756通過／10既有ignored／0失敗**：template70、core883、codegen57、base261、backend190／1、bridge130／1、client150／8、ABI15；generation等九工作exit0且退休。

新增權威測試經五次實際南史詩死亡／重生，限時／永久共用stacks與舊欄位一致、最多四層。新增ABI回歸驗證payload bytes、permanent flag、hash、同key更新、hero換代舊owner退休與空snapshot清除。

原生OwnedBuffListSnapshot測試新增timed／permanent的×3／×4文字、count-only快取刷新、非法Blueprint輸入、decoder缺欄／最大值／錯誤型別／非整數／超界、ABI string ref越界等案例。這是實際初始化／TakeWidget後的原生widget驗證；本輪沒有新增專用層數截圖。

UE5.8 build-v1成功（UBT **58.24秒**）、正式後端與Lua三契約成功。WorldV1 **37成功**（36 Success＋1 SuccessWithWarnings，0 Failed／NotRun），4既有preview-world清理警告保留。兩張map各74影格／6片段，兩張map及既有首殺資訊列三張PNG已目視，屬場景與既有HUD回歸。

**3355來源／資產**接受前後集合、SHA、size、mtime一致，七項staging與import library另驗。**44份證據**逐byte／SHA封存後獨立複核；三分鐘監測 **5次／最大181秒**，7 worker＋7 watcher與全程monitor退出。

## 界線與剩餘

本輪無建置／測試失敗，兩次路徑查詢錯誤與熱路徑設計注意記錄於2026-10-22-buff-stacks-errors.md。完整MVP仍33/44，11項外部ACK／LAN／OS DPI／效能待驗未改；沒有新增網路對局、配額重設、commit／push或omfx維護。未把前輪native／IPC／Blueprint／PCM數字算成本輪接受。

仍需補小兵光環／分裂、經濟補償、更多擊殺／目標公告、技能引導／幾何與施法動畫、召喚複本生命期、塔皮與美術。盤點是功能入口清單，這批完成不代表整個遊戲已達LOL全部規則與內容。

證據索引：../acceptance/2026-10-22-buff-stacks/index.json。
