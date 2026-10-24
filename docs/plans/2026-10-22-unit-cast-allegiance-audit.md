# 2026-10-22 全遊戲盤點：指定單位技能陣營提示

本批接續正式全遊戲盤點，先補單台電腦能驗證且會影響日常施法可讀性的缺口。地圖已具左下／右上基地、兩張三路圖、兵線護主、塔壓力與野怪回頭／重生／分裂；本輪保留這些既有規則與回歸，不以重複複核冒稱新增玩法。

## 本批變更

QWER的entity技能已有逐rank ally／enemy目標metadata，正式命令提交也有存活友軍／合法敵軍admission；原先預覽只顯示靜態需求與距離。新增Bridge只讀查詢，以目前公開source／target完整generation與玩家歸屬守門，重用正式提交的IsPublishedLivingAlly／IsPublishedLegalHostile。敵友不符、敵方目前不可施放與未知條件各自顯示提示。目標關係變動不需移動游標，隱藏／死亡／source退休或世代替換即失效。

OmRuntime透過嚴格反射契約呼叫，不依賴OmGenerated；不掃Actor、不保存全場關係cache、不改網路命令或權威執行。道具／召喚師未提供此authoring需求，維持既有range提示，不聲稱新增陣營檢查。提示僅表示目前公開條件，不能當作未來施法成功或ACK。

真實World frame→Bridge公開表→owner反射→feedback測試覆蓋敵友、中立、可攻擊條件與生命期，集中驗收沿用既有完整Rust／Lua／UE／地圖與HUD矩陣。原生測試World限定FEditorScriptExecutionGuard，沒有放寬正式Actor生命周期。

## 仍待擴充

持續引導／特殊技能幾何、多段／導引與對應提示；完整combat複本及退休來源callback上下文；召喚競爭落點commit整組原子回滾與更多控制；更多英雄／道具／召喚師與目標專屬音效美術。這些是後續獨立變更，不因本次預覽而宣稱全遊戲已與LOL一致。MVP33/44，11項真ACK／LAN／實際DPI／固定效能外部條件維持pending。

本批最終數字以2026-10-22-unit-cast-allegiance-final-acceptance.md與封存索引為準。零新增真網路對局、quota reset、commit／push，omfx未維護。
