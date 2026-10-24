# 設計

- compiled_vision_occluders由terrain的整數bounds直接產生固定點polygon，角序與碰撞相同，依wall id排序產生map-local stable id，無map名稱分支。
- 正式compiled map啟動時覆寫VisionOccluderSet，避免混入其他campaign遮蔽物；舊campaign／單路保留descriptor policy。
- Wave B、fog及單位道具已共用VisionOccluderSet，沿既有range／brush／detection與scope override規則。牆不阻擋point AoE的傷害語意，本輪只變視野。
- Public rank metadata的UnitAllegiance經HUD轉入owner slot。只有ally／enemy可用，unknown／未學習／非unit／替換／退休清除。提示文字表達需求，仍不聲稱候選目標有效、不改送出或權威拒絕流程。
- 不新增權威網路對局；Rust World雙副本與synthetic renderer／IPC屬本機接受，不冒充ACK、LAN、OS DPI或單session效能。
