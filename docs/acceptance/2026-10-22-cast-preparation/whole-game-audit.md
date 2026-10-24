# 2026-10-22 施法準備動作與遊戲完整度盤點

本批接續全遊戲17系統盤點，選擇單機可完整驗證的權威施法前搖動作。原有owner進度條與成功release cue沒有提供其他可見角色在準備期間的動作；這輪補上該缺口。

## 通用實作

- 實際AbilityWindups在outcome與buff timer後同步native-only cast_preparation公開狀態；duration／remaining為有界Q10，instance用十進位字串保留int64精度。只迭代有效windup與已索引的該類狀態，未增加全體entity逐frame掃描。
- 正式BuffSnapshot與既有ABI34 active buff跨generation與payload bounds傳遞，Unreal安全解碼成typed FrameState。非英雄、死亡、未知catalog、重複／非法狀態閉合；完整snapshot缺項清除，control-only沿用既有語義。
- 原生角色沿已配置cast clip對齊權威進度與速度，準備優先於移動／hit，死亡／respawn／recall維持較高優先。成功釋放使用獨立release lifecycle，取消回到最新移動姿態，退休instance不能復播。
- 暫停停播，延遲snapshot到clip末端保持姿態；隱藏清除快取動作，重新可見須有新的有效權威frame才能恢復進度。epoch／generation沿既有Bridge與actor守門。
- 不改魔力、冷卻、傷害或召喚效果的交易時點，不從本機瞄準或cooldown猜施法。

## 接受範圍

集中驗收包括core精確時鐘與中斷、正式Production60Hz generated技能取消／暫停／釋放一次、實際ABI payload的identity與退休、Unreal有界解碼及真正skeletal clip位置／播放狀態。另回歸既有兩張地圖、兵線／塔／野怪與HUD測試；最終數字及證據以final-acceptance與索引為準。

## 尚需完善

持續引導／脈衝、線／扇形／多段／導引幾何與對應提示，更多召喚控制及完整可見複本生命期，目標專屬名稱／音效與平衡，英雄與場景美術內容仍有工作。地圖左下對右上、兵線護主、塔focus、野怪leash等既有正式本機接受保持回歸，這不代表與任何特定LOL版本完全相同。

原MVP33/44及11項外部ACK／LAN／實際OS DPI／固定效能待驗維持。本批零新真網路對局、零quota reset，不涉及omfx、commit或push。
