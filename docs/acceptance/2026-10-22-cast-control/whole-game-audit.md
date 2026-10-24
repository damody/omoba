# 2026-10-22 全遊戲盤點：控制效果施法提示

正式全遊戲盤點沿2026-10-21-whole-game-completeness-audit.md。地圖雙方左下／右上、三路、塔／兵／護主、野怪回頭／分裂、成長與目標經濟等已有接受；本批保留這些正式規則，補日常操作的施法阻擋原因。

## 實際缺口與通用解

權威BuffStore已規定stun同時禁移動／攻擊／技能，silence禁技能，root僅禁移動。既有QWER預覽提供rank、cooldown、mana、目標陣營與射程，但缺少目前控制狀態原因。Bridge新增source-relative摘要，只讀當前公開frame；使用typed catalog識別正式控制ID，不依賴英雄名稱或顯示名稱。全表有界4096項，避免UI清單64項截斷漏判；僅保存一個目前source摘要，查詢O(1)，無全場cache／Actor掃描／每entity每幀UI操作。

完整來源與玩家／generation守門沿既有公開中心查詢。來源清空／revoke／新快照重建，非完整presentation、未知metadata、malformed控制資料保持未知。暈眩優先沉默，同類取最長公開剩餘，永久不顯示負數倒數。不用牆鐘推測，pause與延遲更新仍沿公開值。實際繪製透過owner嚴格反射，QWER point／unit預覽顯示原因與剩餘時間；instant self未開預覽者不宣稱新增浮層。

控制提示只影響可讀性；未學習／冷卻／魔力保留優先。不改命令提交或權威admission。root不當沉默；道具／召喚師未共用此技能控制規則，不硬套預覽。CABI34、protobuf與後端規則未變動。

## 驗收界線與後續

新增真World公開frame→Bridge→owner反射→feedback語意測試；集中Rust／Lua／UE與既有地圖、技能、HUD及資源生命周期回歸。七張既有PNG僅是消費端回歸，不是本功能專用截圖或真網路ACK。

持續引導／特殊幾何、完整combat replica、退休來源callback上下文、召喚競爭落點commit整組原子回滾，以及原創英雄／道具／召喚師、美術與音效仍待擴充。MVP33/44、11項外部ACK／LAN／實際DPI／固定效能接受維持pending，不以本批提示替代。

最終驗收以2026-10-22-cast-control-final-acceptance.md及封存索引為準。無新增真網路對局、quota reset、commit／push；不維護omfx。
