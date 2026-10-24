# 全遊戲盤點：位移技能受控與原地施放提示

依最新交接，地圖斜向配置、兵線／護主、野怪返回、塔與正式沿路衝刺已有驗收。本輪新增可單機驗證的缺口：root下位移仍顯射程內，原地落點無清楚原因；通用位移與普通技能必須分開。

共用內容函式由typed SweptPath或executable DashToPoint導出位移需求；Unreal generator→current-rank HUD→owner policy接線，無英雄ID／名稱／cosmetic特判。bridge以完整公開source buff snapshot產生獨立root channel，保留精確generation／owner／披露／存活守門與退休清理。普通施法仍只受stun／silence影響。owner兩種查詢共用嚴格reflection簽章校驗。

預覽新增定身／永久定身／狀態未知／原地無位移原因，既有unlearned／cooldown／mana／stun／silence優先；原地檢查僅地面距離，不把Z差當位移。tooltip說明root與不同落點需求。僅本機唯讀提示，不取消／阻擋／發送命令、不預測權威ACK或自行倒數控制時間。後端既有碰撞／root拒絕與mana／CD回滾不變。

新增schema、arbitrary-ID generator與native真bridge／owner reflection／feedback／tooltip／retirement案例。集中結果以本批final-acceptance與封存index為準；170秒健康採樣。

整體仍缺持續引導、定時衝刺／殘留、更完整特殊幾何、完整combat replica接受、召喚整組commit原子回滾、英雄道具召喚師內容量、美術與音效。MVP33/44與11項外部ACK／LAN／實際OS DPI／固定效能接受維持pending；不新增真對局、不維護omfx。盤點總表仍見2026-10-21-whole-game-completeness-audit.md。