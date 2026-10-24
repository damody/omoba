# 權威施法進度與結果提示盤點

接續全遊戲17系統盤點，補前輪已有執行但缺動態回饋的前搖。採既有正式MOBA、client-runtime與唯一Unreal前端，不新增網路對局。

## 本輪實作

- 權威前搖表保留每英雄一筆Preparing／Canceled／Released、固定點remaining／total與遞增instance。成功／失敗由真正handler交易結果記錄，不從cooldown推測；明確取消、生命期失效與釋放再驗證失敗皆可清理進度。
- PostStep輸出5個既有Team HUD metrics：slot、state、remaining、total、instance。非playing／死亡／沒有當前英雄時輸出零；沒有施法目標或位置。client-runtime只取指定team／player，拒絕缺欄、重複、非法狀態／時間與instance。
- proto新增typed optional AbilityCastPresentation，同步checked-in generated fallback並做codec驗證；保留舊snapshot缺欄時無進度的相容性。正式後端／共用Rust／Unreal消費端接線，歷史omfx不補線。
- OmAbilityHud擴充，Rust／C header／smoke同步ABI31；bridge驗證型別值並轉為秒，空frame清理。Unreal owner ability payload帶權威值，技能欄進度條與準備／短暫取消／釋放文字沿既有更新，UI不自行完成前搖。
- 同instance不容許時間增加、已取消重新準備或較舊終態覆寫；新instance可獨立開始，退休／死亡輸入清零。終態文字1.25秒的本機顯示時限只屬表現，不影響遊戲時序；pause時進度採權威凍結值。

## 驗證

核心明確結果／instance與退休清理；正式60Hz技能檢查真實PostStep準備、遞減、Stop取消、再次施法、成功釋放、死亡零值。client-runtime測team／player／slot隔離、非法／缺少／重複tuple及protobuf codec。bridge測typed HUD跨到ABI31／40-byte ability row與空frame；UE測進度比例、文字、舊封包、終態與清理。兩map／既有UI場景作回歸，不把閒置場景畫面當真實連線施法或動態bar的視覺接受。

結果以同日cast-progress-final-acceptance.md及封存index為準；不以接線文件代替實際通過。

## 後續缺口

真實連線owner ACK下的動態施法畫面、公開角色施法動畫／可見敵方提示、更多中斷／扣款政策、持續引導／脈衝、非法施法意圖與舊命令規則、特殊技能幾何；完整召喚複本生命期、競爭落點整組原子回滾、退休callback、objective永久stack／傳承／小兵光環、營地分裂、塔皮／公告與美術。

原MVP維持33/44／11外部ACK／LAN／DPI／效能待驗，不重設配額、commit／push或維護omfx。
