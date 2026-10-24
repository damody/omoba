# 設計

僅正式 MOBA 將非零 cast_time 排入每位施法者一筆的原生前搖表。排程前檢查 rank、資源、冷卻、生命與控制；無效果、無扣款、無成功 cue。釋放時仍經既有交易執行，成功才扣魔力／冷卻並發布原成功 cue。只以固定 dt 推進，每 tick 至多一次；pause 凍結，生命／generation、silence／stun、移動或接受新命令取消。明確保留 channeled／持續脈衝等後續範圍，不改舊非 MOBA 腳本 fixture 的即時語意。

前搖保存完整實體與目標 generation，釋放不查裸 id。新同批輸入優先於舊到期釋放。資料放原生 optional resource，digest 包含未釋放前搖；不把私有位置／目標加入 public schema。UE tooltip 由既有 generated metadata 呈現前搖秒數與取消規則，實際成功 cue 仍只在釋放成功後出現。
