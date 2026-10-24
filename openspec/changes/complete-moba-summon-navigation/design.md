# 設計

沿用公開 polygon 的 fixed-point swept circle 與 static_next_waypoint，不查隱藏動態單位。生成時在要求點附近固定順序有界搜尋，保留與主人之間無穿牆路徑；沒有合法落點則退休 reservation。每批已接納 summons 的圓形體積也參與落點去重，不讀敵方隱藏資料。

SummonStats／Lua 增加 follow_start_distance 與 follow_stop_distance，0/0 表示固定站位；驗證 finite、範圍及 stop < start。SummonedUnit 保存 opt-in 跟隨與導航狀態，納入既有 digest。明確 MoveTarget 優先，否則距主人超出 start 後返回 stop 範圍；主人死亡／完整 generation 失效時取消自主返回，不重綁新生命。使用共用路徑規劃並快取 waypoint，250ms 重新規劃，不瞬移、不每幀重建 UI。

原生生成、命令移動與腳本追擊共用公開地形路徑規則。Root／stun、pause、生命期保留；預設無 follow 設定的舊召喚 fixture 行為維持。OpenSpec CLI 已於前輪確認不可用，本輪依既有結構維護 artifacts，不新增 shell 工作流。
