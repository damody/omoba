# 目標公告橫幅：最終驗收

完成 `complete-objective-announcement-banner`。正式 Unreal HUD 現在將公開目標事件顯示成獨立的粗體公告橫幅，不只附在小字對局資訊列。兩隊的防禦塔、兵營、基地防禦塔及史詩目標共用同一型別驗證／文字選擇函式，沒有英雄或營地名稱特判。

HUD owner 只從已接受來源的權威快照更新。公告需要有效隊伍、角色、sequence、Playing phase及0..4秒內的正有限remaining；缺失、到期、終局、非MOBA、runtime重置與退休立即清除。退休後舊快照不能重建公告。沒有本地倒數或成功推測。使用者隱藏HUD立即收合，恢復顯示沿目前有效狀態；容器HitTestInvisible，不攔截遊戲輸入。

沿既有HeaderBudget給定寬度／內距及版面失效流程，文字18..36px隨使用者文字縮放；與原本對局摘要共享資料驗證，保留舊摘要相容性。未修改Rust玩法、ABI、平衡或網路協定。

- **UE5.8 buildV2成功**，當前DLL部署及import library SHA一致；buildV1亦成功。
- **66項原生回歸成功、零Failed／NotRun**；保留4個既有RuntimePresentationReset warning。另1項真正Rust compiled-map export fixture成功；未重算前批1419項Rust。
- 新原生測試含兩隊四種目標、11種無效／缺失／終局樣本、即時expiry/reset、HUD hide/show、退休與重送，以及1280／640／320 fixture視窗、1x／2x文字幾何與像素。
- **29張PNG**：21張與前批已審閱像素雜湊相同，8張差異／新增已目視（回城、分層地圖、首殺、召喚姿勢及4公告）。新320寬2x文字分為四行，內容沒有裁切；不等於實體DPI測試或完整320px對局接受。既有首殺圖下方／右側HUD裁切及部分英雄／focus名牌重疊仍保留限制。
- **3421來源／資產** exactset、SHA、size、mtime前後一致；**129份證據**逐byte／SHA獨立校驗成功。4最終worker、4歷史worker及watcher均退休/reap，兩段health timer亦結束；7次健康採樣，最大間隔171秒。最後CIM沒有本輪Lua／Cargo／rustc／cl／link／Unreal程序。

V1有65項原生成功／1項新測試失敗：文字已清空，但Slate visibility lambda在狀態邊界仍保留先前求值。保留原始widget/header/owner/test、完整report/log及4張失敗cohort公告圖；全部reader退休後改用authority setter、chrome與retirement明確push visibility，新增立即hide/show/retirement斷言，以V2全66及新source baseline接受。V1 source因失敗前置被拒，不冒稱通過。詳見[錯誤與預防](2026-10-23-objective-banner-errors.md)及E1509。

最初的「缺少換行」推論已在實作前更正：既有HeaderBudget本來就明確設定WrapTextAt。這批是新可讀公告功能，不冒稱修復原有換行或首殺下方HUD。

證據：[index.json](../acceptance/2026-10-23-objective-banner/index.json)。4/4 tasks完成。一般channels／zones、整組召喚資源交易、完整combat replica與11外部父項仍pending，MVP33/44不變。下一批優先具體玩法，避免重複名牌、目錄或公告複核。48h原期限2026-10-25 00:33:38 Taipei不變。沒有新增真網路對局、quota reset、commit/push或omfx工作。
