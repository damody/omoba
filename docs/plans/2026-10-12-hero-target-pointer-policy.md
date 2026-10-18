2026-10-12 只選英雄本機指標策略範圍接受：`.ai-collab/hero-target-accepted-v1/summary.json`。Codex自行實作無子代理。沿既有TargetChampionsOnly主／副鍵（預設Tilde）與profile hold／toggle，新增精確Slate按下／放開與OS repeat／雙鍵狀態，設定使用原子保存；modal／profile／死亡／來源退休／focus及connection tick清除。公開Kind先過濾再比較距離，接選取／hover、右鍵及AttackOnly、指定單位施法取樣；地點、自己施法與A攻擊移動不受影響。HUD啟用提示沿共用HeaderBottom量測，不做widget-path查詢。

ordinary NoEngineChanges／NoUBA／nocache build-v5成功19.35秒（v4成功12.78秒），fresh native-v2真9Success零warning，含真Slate press／release／repeat／toggle／modal／death／focus／保存及實際HUD文字；公開重疊候選與hostile/source驗證通過。fresh69Blueprint零compiler error／warning／load failure且原packages SHA／mtime不變，12來源凍結全程不變，stage再驗；Rust exporter9來源未改明示沿用。不冒稱新full409／owned TCP／真施法authority ACK或原圖檢視。E989–E992原失敗及stack-overflow crash保留；修正為Slate attribute只讀狀態，事件／tick負責焦點退役。autoattack／cursor偏好、英雄個別profile、WASD仍待接通，3.5不勾完；父項12/44與quota10/10保持，完整驗收最後。

# 只選英雄：本機指標策略

Codex 自行實作，不使用子代理。沿用現有 TargetChampionsOnly action、主要／次要綁定及 profile 的 hold／toggle 欄位，不新增 wire command 或假造 authority ACK。

1. 以單一按鍵狀態模型處理真 press／release、OS repeat、兩個綁定同時按住與 toggle；失焦、文字／modal、profile 或玩家來源退休後清除，不在恢復焦點時重新啟用舊按鍵。
2. 明確設定開關使用既有原子 profile 保存路徑；指標選取、hover、直接攻擊及指定單位施法先依公開 Kind 過濾英雄，再挑最近合法候選，避免較近小兵遮住英雄。不影響地點技能、移動與 A 攻擊移動。
3. 實際 native 檢查狀態模型、公開來源／隱藏／世代及重疊候選；再跑普通 Unreal build、相關 native 與既有 Blueprint。真實視窗事件留在完整驗收中驗證；未驗部分不標完成。
4. 錯誤連同原始失敗證據記入 unreal-moba-error-register.md；本範圍成功後繼續其餘操作，整套驗收最後。
