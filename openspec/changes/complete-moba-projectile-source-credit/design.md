# 設計

原生資源以完整投射物 Entity key 保存來源快照。命中時複製到獨立 outcome，處理damage期間以有界scope提供歸因，結束／錯誤均移除context。每發保存，不能只以來源id保存一份；generation重用不能改寫舊發來源。

結算共用已驗證的player／team解析，野怪、兵線、英雄、建築讀同一快照；仍遵守first lethal、phase／pause與死亡錢包規則。快照不加入公開disclosure，加入原生replay digest；已退休投射物記錄依實際存活集合清理。
