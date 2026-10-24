# 野怪來源歸因錯誤紀錄

- J01：OpenSpec CLI既有環境不可用，手動維護規格與任務，不宣稱CLI validate。
- J02：初查以shell glob／猜測script_outcome_processor.rs、scripting.rs查詢，實際檔案是scripting/parallel_world_adapter.rs。後續用rg定位。
- J03：實查發現ParallelWorldAdapter::spawn_summoned_unit仍回EntityHandle::INVALID，屬未實作stub。不能把SummonedUnit元件存在、或新增歸因resolver，當成正式召喚技能生成已完成；本輪只接受正式Damage與隊伍／owner結算，召喚生成列優先缺口。

最終RustV2 1687通過／10既有ignored（core842、base249），UE5.8／後端build與World33成功。V1 core839／3失敗保留，未混入最終cohort。工作跨2026-10-21／22，檔名沿起始日；完整結果見2026-10-21-jungle-credit-final-acceptance.md。

- J04：新增測試一度將300 XP獎勵誤斷言為experience欄位300；該欄是升級扣除門檻後餘額，正式300 XP從level1會成為level3／80。修正測試，不改既有進度規則；驗收以修正後來源cohort為準。
- J05：RustV1 core839通過／3失敗，其中兩個舊camp fixture的hero沒有Faction，也未安裝Vec<Outcome>。新的非英雄目標驗證拒絕無Faction目標，進入回頭後暴露缺少Outcome資源。補齊fixture的正式Faction與Outcome，不放寬正式目標驗證，保留原攻擊cue／LOS／追擊斷言。V1保留，新cohort等待舊工作與watcher全退休再重跑。
- J06：Saved封存prepare先全域改prefix後，scope值也已改名，後續精確replace仍找舊scope而assert。尚未寫出新seal或封存資料；修正查找已變換後scope，重跑並語法核對。prefix替換後不可假設原文字仍存在。
- J07：World33成功，31 Success＋2 SuccessWithWarnings／5 warnings；4筆既有preview-world DestroyActor context及1筆UE背景generate_204 HTTP 3秒timeout。保留警告，不放寬測試或關閉檢測；最終報告讀本輪實際數字。
