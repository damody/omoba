# Unreal 小地圖權威三態迷霧

## 本批計畫與結果

- [x] 共用 FOmMinimapModel 接收 ABI11 OmFogGrid；只在已綁定的 expected team 1／2 與完整 frame 處理，不接受未綁定隊伍。
- [x] 檢查 schema1、epoch非零、sample tick不超過frame tick、精確count、最多4096格、合法三態、checked Q10 spans與可呈現座標精度；驗證後才複製 lease cells。
- [x] 同 epoch 拒絕幾何變動／倒退 sample tick／同 tick資料衝突；非法 grid清空fog但保留合法公開路線與地形。
- [x] state持有自有 cells／Q10 metadata，不保留 C ABI pointer；control frame保留，完整 reset／既有StopRuntime清空全部payload。
- [x] 產生 row-run遮罩矩形：0 unseen較深、1 explored較淡、2 visible不遮罩；網格外與schematic padding也遮罩。每次最多4096+4矩形，不在OnPaint重建逐格陣列。
- [x] terrain／routes之上畫fog，frozen memory與已披露live marks之後畫，最後畫status；不以grid替換entity披露或輸入權限。
- [x] 沒有路線但有合法grid時可建立小地圖；缺grid仍VISION N/A，不猜全可見或demo視野。
- [x] Unreal build-only成功；本功能Editor測試一次1/1成功。

## 決定

FOmMinimapFogGrid與FOmMinimapFogRect是共用原生資料模型，不增加任何英雄專屬C++／Blueprint graph。網格bounds来自權威公開compiled geometry，不從live／hidden markers擴展。右鍵Point Move仍沿用公開map投影，fog僅影響背景呈現，不提供entity reference或新的target能力。

控制frame不能先MoveTemp既有grid或清空state；只有presentation_snapshot完整frame才更新。新的完整frame缺grid即unavailable，不能跨reset保留舊探索。可見標記與memory之生命週期仍由權威安全資料決定；不能依取樣格子的視野來自行隱藏always-disclosed單位或生成敵人位置。

## 成功確認

`tools/lua/lua.exe scripts/build_ue_moba.lua --build-only`：session55423 exit0，正常生成／bridge／OmGameEditor建置成功；先前E141外部檔案鎖已解除，沒有停止外部專案。built/staged bridge SHA-256仍是 `68a32d8ec314ee1b199d9bae3b78e4417788a1ee08a3d09e57d21b309485a497`，ABI11不變。

owned Editor PID83404，wait-mcp成功。只執行 `ue_native_visual_smoke.lua --test Om.Runtime.MinimapFogGrid --runs 1 --out-dir omfue/Saved/McpAutomation/MinimapFogGrid`：success=true，1 passed／0 failed／0 skipped／0 not_run。

直接確認負Q10座標、三態opacity、四個外圍mask加三個merged runs、矩形可投影、原lease bytes改變不影響UI、同tick衝突、錯team／schema／epoch／future tick／count／nullptr／容量／overflow／未知state、未綁定隊伍、倒退tick、grid-only map、control保留及full reset清空。修改檔diff check通過。

report位於 `omfue/Saved/McpAutomation/MinimapFogGrid/report.json`，不覆寫完整验收報告。沒有PIE／畫面像素、雙UE60Hz對局、全套回歸或完整效能驗收；仍需正式rebase grid恢复與最後整合驗收。OpenSpec維持20/30，6.1／6.2整項不勾選。

## 操作紀錄

廣泛build tool輸出再次截斷，使用其exit0與stage SHA確認，不把不可見log當成功依據。共用引擎Log.txt已被OpenKoikatsu另一輪建置覆寫，不能把其中內容引用為omfue紀錄；以本次task的build結果與專屬Editor功能report為準。詳見E142。
