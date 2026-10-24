# 2026-10-22 全遊戲盤點：可見單位控制名牌

正式全遊戲盤點沿2026-10-21-whole-game-completeness-audit.md。地圖左下／右上、三路、塔／兵／護主與野怪返回／分裂等既有規則保留。本批補戰場控制效果可讀性，避免只有owner施法預覽才看得到阻擋原因。

## 缺口與實作

BuffStore既有stun／root／silence分別限制移動、攻擊或技能。公開active_buffs已包含合法可見entity的控制ID與剩餘時間；本批不改後端、內容catalog、CABI34或protobuf，新增Unreal型別模型與可見單位名牌。

三種控制各自保存公開時間，同類多來源取最長，永久優先；主效果優先暈眩、定身、沉默，其餘並存效果另標示。只顯示主要效果的時間，不能把長沉默誤當暈眩持續時間。永久無負數倒數；極大有限時間呈現9999+秒，避免CeilToInt溢位。使用typed catalog的canonical ID，不用顯示名稱、動畫或UI64項截斷清單推測。

Bridge有界4096 Buff／100000 entity rows，拒絕重複控制instance、非法duration與不完整指標；驗living kind與完整generation，重複entity rows拒絕。沒有公開entity就不保留Buff單獨揭露的控制資訊。缺席狀態不代表保證可操作。

接線為公開frame→Bridge摘要→typed frame payload→Actor→名牌。Health偏好與文字縮放共用；只在主要效果顯示秒數、組合或偏好改變時更新Slate，次要效果純倒數不引起重畫。死亡、reset/rebind清空；hide清cache、reveal不能重播，fresh authority才恢復；control-only保留而完整replacement缺席清理。沒有wall-clock預測或每entity每frame Slate寫入。

## 驗收與後續

新增原生語意與真World整條接線測試，含實際Slate截圖。集中回歸Rust／Lua／UE、既有野怪返回名牌與七張既有消費端PNG、兩張map影片manifest。本機synthetic呈現不取代網路ACK。

持續引導／特殊幾何、完整combat replica、退休來源callback上下文、召喚整組commit原子回滾、更多英雄／道具／召喚師、美術音效仍待擴充。MVP33/44，11項外部ACK／LAN／實際DPI／固定效能接受維持pending。最終數字見2026-10-22-crowd-control-final-acceptance.md與封存索引。

無新增真網路對局、quota reset、commit／push，不維護omfx。
