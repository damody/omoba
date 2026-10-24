# 2026-10-22 目標公告錯誤紀錄

- N01：盤點時再次把 Windows wildcard 放在 rg positional path，導致123；後續只用已找到的具體檔案／目錄，glob僅放在 -g。也有猜測不存在的 scripting.rs／ability.rs，依 rg --files改用 scripting/mod.rs／dispatch.rs。所有操作僅查詢，沒有修改正式碼；不再重試猜測路徑。
- N02：寫fixture後先核對實際API，SimulationTickResult／FactKind::Ability才是既有型別；snapshot()預置兩個phase／elapsed事件，補償tuple破壞測試需從index+2定位。C++ header smoke也有舊ABI33尺寸斷言，與Rust／header一併更新34。全部在啟動編譯前修正，沒有為測試改正式塔／技能規則。
- N03：V1 core904通過／1新測試失敗。測試為區別佇列順序而把一座外塔role改為NexusTower，真實Damage路徑正確套用未開門保護，只有兩座死亡。應用兩隊各自合法敵方外塔，以team差異驗證順序，不繞過正式開門保護。等待所有現有consumer／watcher退休後修fixture；原始失敗封存。helper複製另曾把既有ObjectiveComebackStatus測試ID誤改為新prefix，啟動world前已修正；只改路徑，保留既有測試名稱。
- N04：V1 world export因新bridge測試猜測不存在的RuntimeInner.latest_snapshot而編譯失敗，UE測試尚未開始；正式bridge lib／UE／後端建置成功。依既有first-kill fixture改用frame_slots[published_slot]取得實際public ABI HUD；全批退休後才修復並重跑。不能因函式名稱相近就假設內部欄位，新增fixture前要讀最近已成功的同一路徑案例。
- N05：最終seal因全程監測08:25:14→08:28:17為183秒而拒絕（既有182秒容差），沒有建立封存目錄。相同時段source-v2的owned watcher於08:25:23與08:27:37已記錄在場／完成，因此實際檢查覆蓋沒有中斷。保留原始global183秒；seal改為合併global與11項owned監測的真實時間，逐一要求final、排序驗證最大覆蓋間隔，額外封存所有監測原文與all-checks，不竄改樣本時間或只放寬門檻。後續全程取樣應預留排程餘裕，例如170秒，避免以剛好180秒排程當成硬即時保證。
- N05補記：修改Saved報告helper時，Lua gsub replacement直接帶%d再次觸發literal percent錯誤；seal程式已寫入、finalizer尚未寫入，沒有產生封存。只修未完成後段，改用精確patch／replacement function；獨立verifier重新由每份原始監測逐筆核對時間與final，不接受新增或漏掉的樣本。
- 設計防錯：公告只來自 committed lethal credit，不從視野中的實體消失推測，避免戰爭迷霧洩漏與重複公告。匿名／友軍／強制退休不製造敵方目標成功。
- 設計防錯：公開四元組需完整、唯一、team=0、公用範圍、有效隊伍／種類／正序號及4秒內權威剩時；非法 tuple不顯示。CABI新增明確結構並升版，不能重用reserved padding或首殺欄位。
