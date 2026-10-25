# 友方護盾：最終單機驗收

OpenSpec `complete-ally-shield-effects` 3/3完成。

共用封閉 ShieldAlly 效果支援存活友方與自身，四級數值與持續時間由資料宣告。既有晨光援護保留治療140/220/300/380與施法者回魔10/15/20/25，新增友方護盾70/100/130/160，持續2秒；600施法距離、原消耗／冷卻及穩定ID維持。沿用canonical BuffStore：與既有護盾取較大剩餘量、較長時間，不累加。護盾與回魔不受禁療影響。

Rust authoring、固定Lua生成器、generic resolver與typed rank hints共用語意，沒有英雄／營地名稱特判。amount正值上限1000000、duration正值上限60秒、range正值上限10000；缺值、錯誤形狀、敵我衝突與非法數字拒絕。封閉內部grant封包透過既有ABI傳送；host序列結算重驗來源生命期、施法者綁定、友方存活與距離，再呼叫canonical shield grant。封包不進入公開buff payload；不擴充ABI或wire。

整個效果計畫先preflight，原本付款與managed rollback機制維持；後續無效sibling不留下付款、冷卻或護盾。定向驗證8項Rust與27項固定Lua契約通過，含任意名稱與效果順序、四級、邊界600與raw1越界、敵人／死亡／舊generation／外來來源、非法封包及後續sibling原子拒絕。Production60Hz正式模擬驗證不足魔力不施放、先付款再回魔、完全禁療下授盾、選取隊友且只替caster回魔、最大量／最大時間、public buff、吸收溢出、到期、暫停、冷卻不刷新與成對重播。

技能目前／下一級提示由同一封閉效果生成「友方護盾70，持續2秒」→「100，持續2秒」，原有治療與回魔仍保留。實際native Slate圖片已目視驗收，範圍是共用tooltip元件；不宣稱完整遊戲viewport或實體OS DPI矩陣。

FreshRust九套 **2113通過／0失敗／9既有ignored**；正式backend／moba-config、UE5.8、DLL/import staging與離線catalog成功。原生4項AbilityRankEffects／AbilityUpgradePreview／SummonGuidance／DisclosedAllyCard，0測試warnings。

**3491來源／資產一致**：3472未變／17變更／2新增；集中驗收前後hash、set、size、mtime一致。封存352份證據並獨立重核；原始失敗ProbeV1、成功ProbeV2與完整驗收的owned API實際退出、退休及健康間隔≤180秒均保留。錯誤／根因／預防見同批errors MD。

Catalog data hash `74b316918934d3f9`，identity `b01c80b5babb0fa3`，class surface `7c09b85b21eea4de`。ABI36／presentation12／selective14維持。

未新增真網路對局／reset quota／commit／push／維護omfx／終止無關程序；MVP33/44與11外部驗收維持。

![實際原生護盾提示](D:/omoba/docs/acceptance/2026-10-25-ally-shield/native/capture/rank-effects.png)
