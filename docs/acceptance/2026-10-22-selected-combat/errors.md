# 選取目標戰鬥資訊錯誤與決策

- P01：猜測OmPublicUnitSelectionTests.cpp而讀取不存在檔案；已用rg --files及測試ID查出實際OmUnitTeamMarkerTests.cpp，後續依實際宣告定位。
- 只接已公開的護盾／控制／塔皮／野怪返回狀態，不新增敵方私人stats／技能冷卻查詢。沿既有selection owner／generation／visibility／scope守門。
- 抽出與名牌共用的bounded文字格式，不複製不同版本的倒數／狀態規則；僅在顯示內容改變時發布HUD，raw selected snapshot仍跟隨最新公開frame。
- 沿前輪防錯：完整World receipt成功且worker／watcher退休後才目視全部圖片。來源消費者存活時不改其來源；固定Lua5.4、Rust1.95、UE5.8與-NoUBTMakefiles。
- OpenSpec CLI既有未安裝，依手動spec-driven文件實作，不宣稱CLI驗證。
