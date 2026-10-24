# 2026-10-22 內容驅動施法受傷中斷盤點

接續全遊戲17系統盤點，選擇單機可完整接受的「準備期間受傷中斷政策」。之前已有前搖與可見角色動作；這輪讓技能作者指定每rank是否因HP損失取消，而不是所有技能套同一個受傷規則。

## 已實作

- AbilityLevel嚴格cast_interrupt_on_damage bool，缺值false；true要求cast_time在Q10可表示的1/1024..60秒。固定Lua generator與Rust loader一致驗證。
- 靜態模板、runtime catalog、script AbilityDef JSON與Unreal generated metadata逐rank傳遞；canonical content hash包含政策，runtime與compiled比較及reload shape要求所有peer重建。
- admission將政策複製進Windup，既有replay digest涵蓋。Damage／ProjectileDamage共用的傷害結算及腳本直接傷害在shield之後，僅正HP debit取消仍pending的opt-in windup。
- 完全護盾吸收、零傷害、治療與非opt-in不觸發該政策；已有release不回滾。取消沿同一結果／owner進度／公開準備Buff／角色動作生命周期，未新增一套本機猜測。
- 雜賀援軍四rank啟用，其餘缺值沿預設；正式generated tooltip公開HP傷害與護盾例外，rank切換／缺值／零前搖不繼承舊提示。

## 接受

新增Rust／固定Lua parity、內容hash與rebuild守門、核心default／opt-in／release及digest、generic codegen rank順序；正式Production60Hz比較一般／腳本直接傷害、完全與部分shield、零傷害、治療及成功release一次。原生World檢查正式四rank metadata與真Widget tooltip，並回歸43項既有動畫／技能／地圖／HUD接受。最終結果以final-acceptance與封存索引為準，不以盤點代替測試。

## 尚需完善

持續引導／脈衝、特殊線／扇形／多段／導引技能幾何與提示；召喚完整可見複本生命期、更多控制與競爭落點原子回滾；目標專屬名稱／音效與更多道具／召喚師內容、兵種平衡與英雄／場景美術仍有工作。既有地圖左下對右上、護主、塔focus與野怪leash持續回歸。

MVP33/44與11項外部ACK／LAN／實際OS DPI／固定效能證據維持待驗。零新增真網路對局／quota reset／commit／push，omfx不維護。
