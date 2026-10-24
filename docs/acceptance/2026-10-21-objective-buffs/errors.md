# 史詩目標隊伍增益錯誤紀錄

- O01：既有環境OpenSpec CLI不可用，手動維護規格與任務，不宣稱CLI validate。
- O02：初查猜測moba_jungle.lua不存在，實際營地內容位於templates/moba_maps.lua。後續先rg定位，不新增平行資料源。
- O03：多hunk patch若先修改檔案後段，再搜尋前段context會失敗；本次工具拒絕而未寫入。改依來源行序排列，成功後核對接線。

最終RustV2 1676通過／10既有ignored（template60、core832、base248），UE5.8／後端build與World33成功。V1保留為失敗／取代cohort，未混入最終數字。詳2026-10-21-objective-buffs-final-acceptance.md。

- O04：RustV1 template60成功、core828通過／4新測試失敗。新fixture只process_outcomes一次，Damage的Death仍在next_outcomes；隊伍獎勵設計須在正式Death結算後才授予。修正fixture排空下一輪Death，60Hz測試也前進下一step再斷言，不把發獎提前到傷害意圖。V1保留並由完整新cohort取代。
