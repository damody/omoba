# HUD 剩餘玩家文字與快捷鍵說明

延續已接受的共用可讀提示與玩家商店文字。直接實作，不新增狀態或輸入路徑。

- [x] 背包提示以「使用／對自己」兩組主副鍵完整顯示，移除重複 compact self 行，保留次要鍵與未綁定資訊。
- [x] 小地圖不將傳輸狀態／原始錯誤寫入連線列；拒絕沿共用玩家提示，成功不冒稱權威完成。
- [x] 英雄初始 placeholder 中文化。
- [x] ordinary build-v2 與103來源 SHA；native-v2 exact4 Success／0warning；四種首尾閱讀測試通過，四張尾頁原圖逐張檢視。

範圍接受：playable-mvp-hud-text-cleanup-accepted-v1。原v1中三項成功而tooltip游標capture失敗，E1177保留；fresh v2全通過才接受。無新對局／ACK，非完整MVP。下一項盤點發現 Stats(C) 有預設binding但尚無consumer，接角色詳細資訊的正式操作與可讀提示。
