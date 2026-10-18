# 原創原生戰鬥呈現

本範圍由 Codex 自行實作，不使用子代理。父計畫 complete-moba-controls-and-presentation 仍為 12/44，6.4 未完成；本文件不是全功能驗收。

## 實作與驗證順序

1. 三種原創 unlit translucent 軟邊材質：ring、ribbon、disc。明確 commandlet 建立、完整 preflight、重跑不改既有資產。
2. 每 world 一個呈現 host，三組 instanced mesh，最多 512 個有效效果；保留有限歷史避免隱藏／超載後重播。每個單位的無效果回收走 owner index，避免大量單位各自掃描 pool。
3. 以公開 frame、動畫、技能、彈道與 Buff 驅動效果；回城持續圈不重啟，死亡／重生、隱藏、identity generation、epoch、世界清除回收。暫停跟隨公開播放狀態。
4. 彈道以明確座標有效欄位判定，世界原點合法。技能輪廓使用資料樣式與公開參數；形成範圍的標記不冒充實際召喚物位置。未知技能保留小型來源提示。
5. 正式 ordinary nocache build，真 native 生命週期／bridge／Buff／style 回歸，11 張 D3D12 原始 PNG 與特效單獨渲染像素檢查，人工檢視後才接受此範圍。

## 已有證據

- combat-vfx-material-build-v1 實際 FailedDueToEngineChange，保留 E861；maintained refresh-v1 後，全新 ordinary build-v2 exit 0。
- combat-vfx-material-import-v1：三材質首次儲存三個，第二次驗證三個、儲存零，SHA／mtime 不變；真 shader compile 驗證。首次啟動外部 HTTP timeout 保留 E862。
- combat-vfx-material-contract-v1：六個獨立 native 負例拒絕外部路徑、未知版本、重複／錯誤形狀等，既有三個資產不變。
- combat-vfx-runtime-build-v1 編譯失敗，E864 已記錄；v2 真成功，但 native-v1 的 cone 截圖被固定鏡頭裁切（E865）。修正 fixture 取景及邊界檢查後 build-v3 ordinary nocache 真成功（10.85 秒、refresh false）。
- combat-vfx-runtime-native-v2：15 exact tests Success、零測試 warnings，Editor／workflow exit 0。11 張 768×768 原始 D3D12 PNG 全由 Codex 直接逐圖檢視，cone 完整入鏡，形成圈與三個數量刻度、三發扇形提示、與來源分離的原點彈道、命中 disc、回城／Buff 圈可見。三張特效單獨畫面確認材質本身有顏色。全部彩色特效離邊緣至少四像素，無巨型 debug 球。角色在黑色診斷場景偏暗；此輪僅接受特效功能，不接受整體照明品質。Editor 外部 HTTP401 警告保留。

## 剩餘

實際完成 runtime 編譯、native 渲染與逐圖檢視。角色服裝、Saika 全生命週期、地圖／照明與跨入口最終驗收仍依父計畫進行。既有 authority 對局額度 10/10 不重設；本範圍 synthetic native world 不是新對局或 ACK。


2026-10-12 原創 pooled combat VFX 當前範圍接受：`.ai-collab\native-combat-vfx-accepted-v1\summary.json`。fresh ordinary nocache／NoEngineChanges／NoUBA 成功，15 exact native Success／零測試 warnings，11 D3D12 原始 PNG 全逐圖檢視及邊界檢查，ring／ribbon／disc 特效獨立渲染通過；512 上限、pause／hidden／identity／epoch／death／respawn／Recall／Buff 與 bridge 接線已驗。不是完整世界照明或最終驗收；Editor 外部 HTTP 問題保留，無新權威對局／ACK，父項 12/44、6.4 未勾完。
