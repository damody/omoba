# Bot折線段推進

## 本批計畫與實作

1. 檢查非Jungle路線：最近頂點＋1會在抵達轉角前跳至下一段。
2. route_destination採Fixed64線段最近投影，clamp於線段0..1，距離／段index穩定排序；本人既有正式AttackMove且目標屬此路線時保留，50單位內才沿路前進。已到達／重疊點略過、末點不環回；Jungle仍用前批環狀patrol。
3. 接入Top／Mid／Carry與沒有有效escort的Support共用路徑，保留披露戰鬥與恢復／技能優先，不改正式地形導航、不直接改命令／Pos、不新增其他模擬World。

## 當前確認

- core純策略新1 passed：轉角、到達邊界、combat後線段重定位、反向路、重疊／空／單點／末點。
- 既有role_bots_targets_are_role_specific_and_order_independent指定1 passed，角色戰鬥選取不回退。
- 正式60Hz新1 passed：compiled Lua三路正常generated manifest／正式AttackMove，第一段60%位置仍選轉角；40steps實際位移且不重送，uncommitted本人arrival不推進，commit後選route[2]並由正常driver接納。
- 首次fixture編譯E0616已修正，記E206；使用公開compiled路線，不公開MobaMatch內部routes。

未全套、未100場、未UE／stage、未量測效能或宣稱任意不可達路線自動恢復。Lua／內容hash、ABI13／IPC4不變。OpenSpec20/30及完整5.4／5.5仍待最後驗收。
