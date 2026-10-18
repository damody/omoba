# 商店交易的玩家文字

接在被動道具加成之後執行，沿用同一份已做owner／epoch／request關聯的交易追蹤，不新增購物命令或本機結算。

目前短狀態已是中文，但錢包／商店的tooltip仍接診斷用英文交易歷史。不得靠字串替換翻譯未知錯誤，或因已排入／已受理就顯示購買完成。

- [x] 玩家交易tooltip直接使用既有typed phase／result／PlayerShopTransactionPhrase；保留診斷getter，兩種文字共用同一歷史篩選。
- [x] 分清處理中／受理仍等待／未確認／真settled／拒絕及並行歷史；legacy只接受合法owner／epoch／action17或18／RequestId0／非負result。
- [x] 錢包共用可讀tooltip顯示金幣／商店／護盾／當前道具公開說明，缺資料明示未提供；前景可真hit-test閱讀，不被HitTestInvisible排除。
- [x] 真wallet／foreground內容與既有亂序／重複／斷線／容量64／退休矩陣；ordinary build、103來源SHA、exact5 native與4原圖。

範圍接受：playable-mvp-player-shop-text-accepted-v1。ordinary build-v2 NoEngineChanges／NoUBA／no-cache 20.17秒成功；native-v1 exact5 Success／0warning，四張當次400／800商店原圖已逐張目視。103來源驗證後不變，E1174–E1175保留。這是synthetic公開狀態與正式widget／交易追蹤驗證，不新增權威對局／ACK；完整MVP仍最後。
