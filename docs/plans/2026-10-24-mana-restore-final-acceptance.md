# 召喚師回魔：最終單機驗收

OpenSpec `complete-summoner-mana-restore` 3/3完成。

新增可配置 `self_restore_mana {amount}`；內容 catalog7「澄息」回復200魔力／180秒冷卻，使用原有D/F NoTarget路徑、公開隊伍投影與HUD名稱／說明。可選 `scripts/lua_data/moba_mana_restore_match.lua` 啟用魔力與單人人類D閃躍／F澄息；離線moba-config lock-plan驗證，沒有建立真網路對局。

共用模型將目前／最大魔力納入plan identity，按缺少量夾限；缺少或零容量拒絕、滿魔仍消耗冷卻。ECS在同一獨占交易重驗owner／phase／pause／life，使用原有ManaPool恢復並提交冷卻，拒絕時不修改資源或冷卻。回魔不套禁療、不初始化缺失資源、不恢復HP、不解除控制、不取消移動／引導。沒有英雄名稱特判、額外世界掃描或新增wire欄位；ABI36／presentation12／selective14維持。

聚焦7項通過：內容有限正值／封閉shape、generated名稱、模型cap／stale／原子失敗／重複提交、真ECS滿魔／缺池／控制／移動／HP、正式60Hz輸入＋owner-team catalog7/CD projection、pause、同tick有施放／未施放差額200與paired replay。正式測試排除自然回魔，且不讀私有roster欄位。

FreshRust9套 **2100通過／0失敗／9既有ignored**；後端與moba-config、UE5.8、正式DLL／import staging、generated catalog data hash與離線選角成功；原生 `Om.Generated.SummonerManaIdentity`、`Om.Runtime.SummonerPayload` **2成功／0測試warnings**。新測試只驗生成文字／現有payload；沿用召喚師fallback圖示，沒有新增畫面截圖、真對局或美術宣稱。

**3488來源／資產一致**：相對前批3487中3474未變／13變更／1新增；hash、set、size、mtime在集中驗收前後一致。封存390份檔案並獨立重核；index `docs/acceptance/2026-10-24-mana-restore/index.json`，ledger `omfue/Saved/BuildTemp/mana-restore-final-ledger-v1.json`。

原始ProbeV1 API直接actual1、V2 API68694 actual1、V3 API35509 actual0、accept API93410 actual0全數原始reap、owner／cleanup退出；固定Lua supervisor health gap≤180秒。M01–M03根因與修正保留於 `docs/plans/2026-10-24-mana-restore-errors.md`。未commit／push／quota reset／處理omfx／終止無關程序。

後續仍可補更多目標型召喚師技能、英雄技能與動畫／美術；本批不宣稱完整LOL玩法一致。MVP33/44與11外部驗收維持。
