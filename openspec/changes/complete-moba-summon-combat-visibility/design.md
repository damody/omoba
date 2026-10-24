# 設計

沿用 DisclosureContext 的 authority fail-closed／filtered admitted／legacy open 規則，不自行推算草叢是否可見。召喚查詢共用目標存活、敵對、距離、LOS、match attackability 與 ward auto policy；near/range 查詢保持相同合法性。

內容增加 vision_radius，由正式生成寫入 VisionSource。Stun 阻擋召喚自主 hook，root 阻擋追擊，silence 不應封鎖普通攻擊；既有碰撞移動繼續使用 BuffStore。冷卻以正式共享屬性讀取，不增加每幀UI工作。
