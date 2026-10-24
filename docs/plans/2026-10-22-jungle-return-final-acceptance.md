# 2026-10-22 野怪返回名牌最終驗收

OpenSpec complete-moba-jungle-return-presentation **5/5完成**。接續全遊戲盤點，補可見野怪返回／重置期間的可讀性。

## 實作

權威commit hook於本tick outcomes與勝負裁決後，為存活camp bodies同步jungle_return非戰鬥狀態。mode1取自returning且仍可傷害，mode2取自returning且invulnerable；根怪、members與死亡分裂成員共用根camp正式規則。重接仇恨、回家、死亡／lethal pending、finished清除，pause保持已提交狀態。使用內部PERMANENT_BUFF_DURATION，公開snapshot轉-1 sentinel，不會當作倒數過期；只在狀態改變時替換。

內容catalog追加ID22，既有stun1／silence9／cast_preparation21保留。payload僅含mode，不公開aggro target、patience、reset或respawn timer，不改既有傷害、回血、導航、獎勵或重生規則。既有entity可見性投影與active_buffs承載，CABI34／protobuf不變。

Unreal嚴格有界解碼，驗typed catalog、完整generation與living creep；重複、未知、越界或malformed資料清除。Bridge→frame payload→Actor→名牌顯示「正在返回 · 可重新交戰」／「返回重置 · 無法受傷」，尊重health偏好與文字縮放。沿原有SameDisplay節流，沒有新增每entity每frame Slate寫入。control-only frame維持已公開狀態；完整replacement缺席、死亡、reset/rebind清空；hide時清除返回cache，reveal不可重播，fresh frame才恢復。

## 集中驗收

RustV1 **1820通過／10既有ignored／0失敗**：template76、core914、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15、restart9；含generation十工作成功。新增兩項core測試驗根怪／members、永久duration、無silence/root副作用、pause、實際reaggro、death／finished；實際leash耗盡→soft→hard→home完成。四Lua契約成功，正式後端build-v1成功，UE5.8 build-v1成功（UBT **86.31秒**）。

WorldV1 **49成功**、0 Failed／NotRun，保留4個既有清理warnings。新增嚴格JSON／sentinel、typed公開ABI、重複、錯generation、非creep、死亡、字串bounds、不可見entity、實際Frame→Bridge→Actor→名牌、節流、偏好、control-only／replacement、hide/reveal與退休測試。新返回名牌實際Slate PNG已目視，另保留七張既有消費端PNG。兩張地圖各74影格／6片段；本機synthetic消費端不當作真網路ACK。

**3383來源／資產**接受前後集合、SHA／size／mtime一致；七DLL與import library另驗。**76份證據**逐byte／SHA封存並獨立複核，8張PNG已目視。健康監測 **7次**、最大間隔 **171秒**，7組worker／watcher及全程monitor皆退出。

## 剩餘與錯誤

持續引導／特殊幾何、完整combat replica、退休來源callback上下文、召喚整組commit原子回滾、更多原創英雄／道具／召喚師、美術音效仍待擴充。MVP33/44，11項外部ACK／LAN／實際OS DPI／固定效能接受維持pending，未宣稱全遊戲已與LOL一致。

錯誤與決策：2026-10-22-jungle-return-errors.md；總登錄E1487。P01修正永久Buff內部與公開sentinel的差異；P02誤讀非必要模組路徑改依實際import；P03補hide/reveal舊狀態清理，修改時無原生消費者且generation已退出，Rust輸入未變。所有問題保留MD，沒有臨時關閉生命周期或可見性守門。

零新增真網路對局、quota reset、commit／push，omfx未維護。證據索引：../acceptance/2026-10-22-jungle-return/index.json。
