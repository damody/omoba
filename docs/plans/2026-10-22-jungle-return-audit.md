# 2026-10-22 全遊戲盤點：野怪返回狀態

接續正式全遊戲盤點，地圖左下／右上、三路、兵線護主、塔壓力與獎勵、野怪patience／軟硬回頭／回血／分裂／重生已有權威規則。本批補可見野怪在返回期間的操作可讀性，不聲稱修改了既有戰鬥規則。

## 通用實作

新增內容catalog jungle_return（ID22，既有stun1／silence9／cast_preparation21保留）。commit hook於outcomes與終局裁決後同步存活camp bodies的非戰鬥狀態，根怪／members／死亡分裂成員共用根camp invulnerable判定。mode1可重新交戰，mode2目前不可受傷；只含mode，不含target、patience、reset或respawn timer。不改HP／傷害／回頭導航與獎勵。

使用正式PERMANENT_BUFF_DURATION維持狀態，僅在狀態改變時替換；重接仇恨、到家、死亡／lethal pending與終局移除。pause保留已提交狀態，不自行推測時間。既有entity公開快照與可見性投影承載，未變更CABI34／protobuf，沒有新增私有全圖查詢。

Unreal以typed catalog及bounded JSON解碼模式，驗完整entity generation與living creep；重複／無效／未知資料拒絕。Bridge→frame payload→Actor→名牌，顯示「正在返回 · 可重新交戰」／「返回重置 · 無法受傷」。尊重health偏好、色盲與文字縮放，沿既有SameDisplay節流，不新增每entity每frame Slate寫入。顯式hide清空返回cache，reveal不能重播，fresh authority才恢復；完整快照缺席與reset/rebind清除。

## 驗收與範圍

兩項Rust規則測試涵蓋根怪／members、永久duration、無戰鬥CC副作用、pause、實際reaggro、death／finished清理；實際leash耗盡→soft→hard→home完成。原生測試涵蓋嚴格解碼、typed ABI、不可見／錯generation／死亡、名牌文字、偏好、節流、control-only保留、完整replacement、hide/reveal與退休。新增本功能的實際Slate PNG；另保留既有七張消費端回歸圖與兩map影片manifest。

持續引導／特殊幾何、完整combat replica、退休來源callback上下文、召喚整組commit原子回滾與更多原創英雄／道具／召喚師、美術音效仍待擴充。MVP33/44、11項外部ACK／LAN／實際DPI／固定效能接受維持pending。本批為單機規則與呈現接受，非真網路ACK。

最終數字見2026-10-22-jungle-return-final-acceptance.md及封存索引。零新增真網路對局、quota reset、commit／push；omfx未維護。
