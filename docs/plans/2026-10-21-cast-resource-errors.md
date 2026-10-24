# 技能魔力提示／野怪牆體出手錯誤紀錄

- M01：查找引用不存在的runtime/vision.rs。後續先用rg --files盤點LOS模組；不得重複猜路徑。
- M02：OpenSpec CLI仍不可用，手動維護規格，不宣稱CLI validate。
- M03：Windows native rg不可傳目錄尾端*作展開。已改以實際目錄搭配--glob '*.rs'，不重複此查找方式。
- M04：RustV1 core812通過／1失敗，新camp繞行測試觸發移動event，但fixture缺少ObservableFactBuffer。正式world本來有；僅補測試資源，保留原LOS／攻擊週期／導航斷言，V1失敗證據保留。
- M05：UE build-v1的截圖fixture使用Tip區域名稱，遮蔽既有FString Tip，被C4456拒絕。改為RenderedTip；build-v1失敗65.87秒保留，最終採重建build-v2與重新擷取source-before-v2，不採舊baseline。
