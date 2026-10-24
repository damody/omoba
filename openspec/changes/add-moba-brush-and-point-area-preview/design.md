# 設計

- optional brushes 為最多 32 個不重疊／不接觸的外接 bounds；實際隱蔽為相同膠囊形，有界、唯一 ID，舊地圖預設空集合。Lua 同源產生公開 jungle floor/fringe；保留通行，不混入 terrain LOS。
- Wave B immutable view 複製權威草叢。單位揭露及 fog cell 共用同一個來源／目標草叢判定；仍必須通過距離、偵測與牆壁 LOS。既有 public、owner、force overrides 優先順序保留。
- 各級 point area radius 來自 effects 的 radius_key；同心圓聯集取最大半徑，其他技能零值。HUD 只傳已揭露等級；清除／更換／未學習時失效。Slate 落點圈共用玩家 viewport／DPI 投影與分段裁切。
- 本輪不聲稱完成所有 LOL 機制；攻擊暫時揭露草叢、連接多邊形草叢、專用草叢美術屬後續範圍。既有塔、野怪、護主及波次須回歸。
- 不新增權威／網路對局、不修改既有對局配額；Rust World 模擬和 synthetic renderer/IPC 為本機證據。
