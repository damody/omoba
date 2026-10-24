# 投射物來源與單機完整度盤點

本輪接續全遊戲 17 系統盤點，先補戰鬥／經濟的跨生命期缺口：召喚物或英雄發射後消失，投射物仍應有獨立且穩定的隊伍／玩家歸屬。範圍是正式 MOBA 原生傷害與獎勵；不新增網路對局。

## 本輪實作

- 普攻、方向彈與腳本彈三個正式工廠，在每發建立時記錄完整來源 Entity generation、發射位置、已驗證的隊伍與玩家。每發獨立保留，來源後續改歸屬不會改寫舊彈。
- 真實 projectile tick 將來源快照隨該次 damage 傳入結算。結算 scope 在成功或回傳錯誤時移除；公開 fact 仍走原有傷害／生命值管線，不揭露私有來源資源。
- 英雄擊殺／助攻、小兵尾刀、野怪與建築隊伍獎勵使用同一歸屬解析；玩家死亡時只接受已建立的重生錢包，不製造虛構玩家或復活。英雄重生後由現在的英雄實體領取。
- 回頭野怪仍保留 hard reset 無敵；來源已退休時，soft return 距離判斷可使用發射位置。仍存活的來源使用現在位置。
- 每發來源納入 authority replay digest；投射物退休後清理來源表。無 MOBA 的舊路徑不新增此來源表。

## 實際驗證入口

核心 4 個新增案例使用正式 ScriptProjectile 工廠與 projectile system，涵蓋各發歸屬、完整 generation 重用、來源退休、英雄死亡／重生錢包、兵線／野怪／建築獎勵、重複死亡與 digest／清理。正式 base_content 新增 60Hz 雜賀技能案例，透過 generated handler 召喚、真實 Wave B 提交與自動普攻，讓槍手過期後才命中英雄並檢查玩家獎勵。

驗收結果以同日 projectile-credit-final-acceptance.md 和封存 index.json 為準；本文件不以已接線代替測試通過。

## 後續仍缺

通用施法排程／中斷、召喚落點避障／跟隨、可見性複本完整生命期、已退休來源的腳本 on-hit callback 重建、永久 objective stack／傳承／小兵光環、特殊營地分裂、特殊技能幾何、塔皮／公告與完整美術。TD 分層賞金不是本輪退休來源改動範圍。

原 MVP 仍是 33/44，11 項外部 ACK／LAN／DPI／效能證據維持 pending。詳細盤點仍參考 2026-10-21-whole-game-completeness-audit.md，不宣稱完整複製 LOL。
