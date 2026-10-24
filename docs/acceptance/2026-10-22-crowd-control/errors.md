# 控制效果名牌錯誤與決策

- 依正式BuffStore規則：stun禁移動／攻擊／技能，root僅禁移動，silence禁技能。呈現不改admission。
- 使用完整當前公開active_buffs與typed catalog，不從移動／攻擊動畫或UI64項截斷清單猜測；只顯示公開entity完整generation的控制。
- 優先序stun、root、silence，時間僅屬目前主要效果，其他並存效果另標示，不能把最長沉默當暈眩時間。永久不倒數，極大有限時間以9999+秒安全呈現。
- 沿既有名牌節流；hide/reset/rebind清理，新公開frame才恢復。保留control-only與完整replacement的差異。
- OpenSpec CLI既有未安裝，依手動spec-driven文件實作；使用固定Lua5.4、Rust1.95、UE5.8及-NoUBTMakefiles。先查實際路徑，不用shell glob；正式消費者存活時不改其來源。

- P01：原生流程尚未完成時過早讀取第二張地圖截圖，檔案尚未產生（os error 2）；不是建置或測試失敗。先等World receipt成功且worker／watcher退休，再逐張目視，避免把非同步產物尚未就緒當缺失。
