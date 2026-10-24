# 第一塔與分層獎勵錯誤紀錄

- T01：OpenSpec CLI既有環境不可用，手動維護規格與任務，不宣稱CLI validate。
- T02：設計核對發現只按entity選pending candidate，會在「致死後暫停、稍後才提交」時被後一tick較小entity覆蓋。改以(tick,entity)字典序，保留最早tick並只在同tick定序，並加入測試。

最終RustV1 1681通過／10既有ignored（core837、base248）；UE5.8／後端build與World33成功，無失敗cohort。完整結果見2026-10-21-first-tower-final-acceptance.md。
