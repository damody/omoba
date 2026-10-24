# 召喚陣形嚴格預檢：最終驗收

完成 complete-summon-formation-validation。共用resolver拒絕錯誤JSON、型別、小數／超額數量、非法rank、生命期／間距與超界陣形；缺省值只取目前rank正式內容，先完成所有固定點朝向座標再發出召喚。保留四rank數量2/4/6/8與45/50/55/60秒、自身朝向兩排與既有合法落點調整，修正內容說明。缺位置／死亡owner拒絕。

## 驗收

- Rust **1970通過／10既有ignored／零失敗**；新增3個resolver／真adapter測試與1個Production60Hz四rank測試。第二排落點失敗時第一排已預約實體經managed discard完全清理，可重試；正式60Hz各rank扣魔一次、啟動CD、pause凍結生命期。
- 四Lua契約、backend及UE5.8建置成功；World59成功、零Failed／NotRun，4既有場景銷毀warning保留。
- **3409份來源／資產**集合、SHA、size、mtime一致；7 staged DLL與import library核對成功。
- **137份證據**獨立逐byte／SHA驗證；程序退休後檢查 **20張PNG**。保留既有first-kill局部HUD裁切、objective窄strip及英雄／焦點交疊的範圍限制。
- 健康採樣5次、最大間隔171秒；最終及失敗worker／watcher與health程序均退休。

## 失敗與範圍

V1編譯發現Fixed64沒有abs方法；改採上下界比較，保留原始失敗，等所有reader退出後才修正。詳見errors.md與總登錄E1500。此批驗證施法預檢及既有managed request回滾，不宣稱後續並行owner變更時整組outcome settlement已具原子性。

48h omoba-48沿用期限2026-10-25 00:33:38 Taipei。下一批優先補齊既有召喚的共用自身朝向陣形瞄準提示：generated ExtrasJson已有逐rank陣形值，而point aiming目前只有range／area／swept-path。應用明確authored preview語意，不能使用英雄名稱特判或宣稱預覽就是地形調整後最終出生點。持續引導／區域、整組settlement、combat replica生命期、內容／美術／音效與11外部接受維持pending，MVP33/44不變。沒有新增真對局、quota reset、commit/push或omfx維護。

證據：`../acceptance/2026-10-23-summon-formation/index.json`。
