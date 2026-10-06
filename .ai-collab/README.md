# 協作交接資料

此目錄只保存已審查的人工任務packet、review與state，不含Grok session／憑證／worker runtime。新機器先讀 `docs/plans/2026-10-07-machine-handoff.md`、`state.json` 的 `pause_checkpoint` 與 `review.md`；其餘packet和state內舊increment是歷史，不可當成仍有活躍job或目前完成證據。

遊戲開發目前依使用者要求暫停。本次只交接Git；恢復需使用者說繼續。不得在新機器直接resume原PID／本機worker，或採用舊Engine BuildId／DLL。新Grok工作用新的bounded packet；primary仍負責審查、整合及验证。
