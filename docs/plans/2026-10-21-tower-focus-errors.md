# 塔持續攻擊與技能冷卻明細錯誤紀錄

- F01：OpenSpec CLI先前確認不可用；依既有規格目錄手動維護，不宣稱CLI validate。此輪未使用過期封存代替新驗收。
- F02：新測試先使用猜測名稱single_lane_digest／Vec2::zero；實際API為single_lane_replay_digest與Vec2::new。RustV1三個編譯錯誤已保留；修正後新版本驗收，後續先定位API再寫呼叫。
- F03：RustV2 core816通過／1失敗，pause測試誤認begin_moba_match_tick應回true；原契約paused回false表示未推進。改斷言false，仍保留elapsed、focus state不變及無damage斷言，不修改production pause行為。
- F04：Saved workflow複用時，全域改prefix會一併改掉既有om-cast-resource-capture參數名稱。執行World前已修正回實際支援的capture介面，避免30項測試過了但沒產生新tooltip像素；圖檔仍須獨立存在、目視及SHA驗證。
- F05：RustV3 core817通過後，補上一般塔／主堡塔與實際小兵目標reset覆蓋，主動退休其餘工作，最終改以RustV4 core818及全套為接受cohort，不沿用較早通過數。
- F06：跨模組盤點發現cast_time目前只在內容定義／metadata轉換，未形成全技能權威延遲排程。已撤回準備時間payload／顯示，改用正在執行的公開cooldown剩餘／總長；不得將宣告metadata冒稱實際生效。完整cast scheduling／interrupt列入優先缺口。build-v1／source-before-v1保留但不接受；改為最後build-v2／baseline-v2。
- F07：Saved finalizer的patch內JS換行escape與template backtick造成工具拒絕，未修改來源。改以String.raw且內容不含未escaped backtick，執行前loadfile檢查語法。
- F08：WorldV1成功，28 Success＋2 SuccessWithWarnings／5 warnings，包含4筆既有preview-world DestroyActor context及1筆UE背景generate_204連線3秒timeout。未放寬測試或停用檢測，報告讀取本輪實際warning數。
