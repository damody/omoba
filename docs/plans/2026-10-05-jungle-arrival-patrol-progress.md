# 野區抵達式巡邏

## 本批計畫與決定

1. 盤點Jungle巡邏：既有固定tick切換營地，與本人是否抵達無關。
2. 新增純通用patrol_destination：從已提交的本人AttackMove辨識目前公開巡邏點；尚未抵達就保持，50單位內依公開宣告順序循環到下一個未到達點。相同／重疊點不阻塞，空與單點安全，沒有私有AI游標或額外World。
3. 沒有有效巡邏目的地時依committed本人位置選最近點，以宣告index穩定決勝；戰鬥中斷後重新定位。技能、學習、sustain、Recall與披露戰鬥優先不變，所有動作仍正式PlayerInput。
4. 只使用公開camp.definition.position，不讀營地entity／HP／alive／respawn／aggro。不是安全gank、路線不可達偵測或所有策略完成。

## 當前功能確認

- core純planner新1：首輪成功；途中／到達邊界／環回／中斷重選／空單點與重疊點。
- base_content正式60Hz新1：首輪成功；105次正常step跨越舊時計輪換仍不改道／不重送，實際Pos有位移，未提交本人arrival不改計畫，committed arrival送下一個AttackMove並由正式driver接納。
- 重疊點與精確Wave B tick補強後，兩項相關測試再次成功；沒有跑其他全套測試。

沒有做完整驗收／100場／UE／部署DLL；Lua資料／內容hash、ABI13與IPC4不變，OpenSpec20/30及完整5.5維持未完成。E205保存問題、工具路徑错误與決定。
