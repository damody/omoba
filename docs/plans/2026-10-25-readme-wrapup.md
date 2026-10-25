# README 收尾紀錄

依使用者要求停止新增功能、整理截至2026-10-25 Asia/Taipei的 README。先完成已在執行的 minion-cadence 驗收及307檔獨立封存核對，再更新總覽；未新增真對局、重設quota、commit/push或維護omfx。

README包含各系統實作、最新兵種節奏／公開射程／選取範圍與距離／NPC／支援策略增量、最新fresh九套Rust2163通過／0失敗／9既有ignored、四項native零測試warnings、UE5.8／backend／staging／離線plan、3496來源與307份證據、版本15/12/36、MVP33/44與11待驗父項、後續玩法與外部接受限制。修正舊D:\code路徑、過時協定版本與來源檔案、runtime-lua-content適用範圍，保留技術指南與歷史前端說明。

minion-cadence index SHA-256：6294d80f8e11295097933aa7c058e13b2a7bc8cb20bf4f1c1fcc11c84c25484d。各原始程序均已取得actual exit0並退休；第一次probe編譯失敗與來源副本保留於功能證據。

無人值守omoba-48已透過app automation工具更新為PAUSED，避免收尾後又新增功能；原期限／排程／prompt保留，可由後續明確要求恢復。本次不將原MVP或整個遊戲宣告完成。

錯誤與防止重犯：RW01，探索時猜測complete-playable-mvp/tasks.md與runtime/presentation_bridge.rs不存在；實際目前presentation常數在omoba-core/src/renderer_protocol.rs。應先列出實際檔案，rg遇忽略的目錄可限制範圍使用--hidden --no-ignore；不要推測檔案位置。RW02，README唯讀核對首次退出1（chunke037f0），因預期的attack_interval_ms欄位說明未寫入總覽；補上單位／範圍／省略預設／Q10說明後重跑，遊戲來源未改。文件驗證應核對實際欄位而不只功能名稱，失敗不得冒稱通過。

文件核對使用固定tools/lua/lua.exe與omfue/Saved/BuildTemp/readme-wrapup-check-v1.lua；補上專案概觀段落標題後由不可變v1檢查器派生的check-v2再核對，最終機器可讀結果在readme-wrapup-verified-v2.json，保留v1結果。文件編輯未重跑已通過的遊戲測試；只檢查相對連結、最新封存數字／來源、版本及目前排程狀態。
