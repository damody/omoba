# 設計

World資源持有Arc<Mutex<BTreeMap<Entity,(team,pos,radius)>>>；只在既有canonical serial allocation階段建立預約，speculative hook仍不分配。新cache共用World資源，舊bare harness保留cache內預設ledger。僅同team佔位，避免敵人隱藏資訊影響預測。4096全域與128 invocation上限。commit仍重查公開地形／目前owner，移除對應預約；discard/drop釋放原generation。此變更完成跨invocation落點一致性，整組commit原子回滾及退休來源callback仍後續。