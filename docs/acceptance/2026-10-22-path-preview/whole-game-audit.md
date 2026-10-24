# 全遊戲盤點：內容驅動沿路施放提示

接續沿路碰撞衝刺：實際玩法已沿路命中，但原提示僅射程圈。此批新增typed aim_geometry契約／逐rank寬度、Unreal current-rank payload、owner capsule及技能tooltip；不依英雄ID／特效半徑推測。fire_dash以既有dash_width宣告，同一資料供真handler與提示使用。

capsule畫意圖路徑，沿地面clamp至射程，含起終端；實際地形裁切仍由後端決定。文字說明超距離截短、遇地形停止與權威結果；不發命令、不阻擋命令。owner未學習／型別替換／非法值／退休清除舊資料，沿既有pending-aim取消生命期。

新增content schema、arbitrary-ID codegen及native幾何／owner／tooltip案例，最後集中建置回歸；結果以final-acceptance與封存index為準。健康監測170秒採樣。

全遊戲盤點仍沿用2026-10-21-whole-game-completeness-audit.md：持續引導、定時衝刺／殘留、更多特殊幾何、combat replica生命期、召喚整組commit原子性、更多英雄道具召喚師、資產音效仍待完成。地圖／野怪／塔／兵線既有功能納入回歸。MVP33/44與11項外部ACK／LAN／實際OS DPI／固定效能接受不變；不新增真對局、不維護omfx。