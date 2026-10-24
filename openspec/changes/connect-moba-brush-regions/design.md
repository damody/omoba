# 設計

草叢載入時計算連通元件，使用 doubled fixed-point capsule center segment distance，包含切線邊界；以雙 limb 平方避免極端 i64 raw 座標溢位。Immutable Arc topology 可廉價分享至 Wave B，逐 entity／fog cell 不重建圖、不配置 heap。來源與目標屬同一元件即可通過 brush 規則，range／wall／stealth 與既有揭露規則不變。相交 AABB 本身不構成連通。保持 32 區塊上限、合法 bounds 與唯一 ID，允許 distinct ID 重疊。

操作提示讀取同一份 PlayerControlProfile，不另存 mode，避免設定變更後過期。沒有 owner 時使用現行預設。未知／退休技能不顯示舊操作資料。完整驗收使用既有 owned phased 本機流程。
