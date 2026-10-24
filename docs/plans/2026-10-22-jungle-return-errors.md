# 野怪返回呈現錯誤與決策

- 決策：不公開aggro target、patience或respawn timer。只把已提交的returning與invulnerable可觀察狀態投影為可見性保護下的非戰鬥Buff。
- 正式內容追加catalog，不改既有ID；同時重建host／DLL／Unreal消費端。沿用Rust1.95、Lua5.4、UE5.8與-NoUBTMakefiles。
- 防錯：先rg --files／實際宣告再讀檔，不用PowerShell路徑glob。任何修復先等來源消費者與watcher全部退休；錯誤保留原始證據。
- OpenSpec CLI仍未安裝，依既有spec-driven文件手動實作，不宣稱CLI驗證。

- P01：內部永久Buff不是-1秒，而是PERMANENT_BUFF_DURATION（i64::MAX raw）；-1僅為公開snapshot秒數sentinel。已於建置前改用正式常數，避免被buff tick立即過期。

- P02：一次非必要讀取誤指向runtime/ability_runtime.rs（實際為目錄模組）；已停止該讀取，依既有Buff模組實際import使用完整PERMANENT_BUFF_DURATION路徑。

- P03：既有Actor隱藏只關閉名牌render，顯式reveal可能沿用快取。新返回狀態於hide時清空，fresh frame才恢復；加入hide/reveal測試。修復時generation工作已完成，沒有原生build／test／sourceguard消費者，Rust來源未變。
