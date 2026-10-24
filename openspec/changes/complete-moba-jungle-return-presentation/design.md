# Design
commit hook於outcomes後同步jungle_return永久非戰鬥Buff，payload mode=1可重接返回／2不可受傷重置。只對存活camp bodies發布，根怪／members／死亡分裂共用根camp invulnerable規則；reaggro／抵家／死亡／終局移除。使用既有可見性投影、typed catalog與active_buffs，不改CABI。Unreal嚴格有界解碼、完整generation與living creep，加入既有名牌節流與退休鏈；無效／重複／未知皆清除，避免last-known狀態殘留。
