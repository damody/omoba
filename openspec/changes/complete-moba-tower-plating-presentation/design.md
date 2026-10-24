# 設計

獨立native_only tower_plating_status公開health_segments={total,remaining}，無戰鬥加成；在有效外塔第一次active tick就建立，剩餘層數由不可倒退的權威ledger決定，治療不恢復。有限Buff期限、到期／退休／結束清除，既有護甲Buff與經濟規則維持。

Unreal共用decoder接受total1..16、remaining0..total及有限正期限；payload<=4096，整數／字串邊界與公開存活generation驗證。橋接每snapshot只建一次map，先掃原始UTF8 marker，沒有候選就不建entity集合或JSON；相同target多來源衝突拒絕。完整baseline清除缺項，control-only保留。

FOmFrameStatePayload帶typed model，Actor與Widget快取以count／實際顯示的ceil權威秒數觸發；普通名牌不建立分段cell，首次有效分段才建立最多16格並重用。顯示有效數量、剩餘／總層數與權威剩餘秒數。HP條保持獨立；血條偏好關閉、死亡／退休／新generation清除，沿既有可見性／health nameplate admission與節流。新增原生widget截圖與端到端回歸。
