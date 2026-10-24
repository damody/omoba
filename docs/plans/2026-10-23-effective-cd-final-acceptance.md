# 實際冷卻總長HUD：集中驗收

complete-effective-cooldown-hud完成4/4。前批已讓cooldown modifier正式生效，但owner HUD仍取作者base cooldown作分母；slow會remaining>total而被native拒絕，haste的進度也錯。本批共享Hero新增optional serde-default ability_cooldown_totals，start_cooldown保留實際duration、refresh覆寫、清除或到期移除；tick不重新取buff，zero/negative dt不推進，扣除用saturating_sub。合法running total上限沿用1million秒且不小於remaining。

owner HUD沿用既有total_cooldown_raw欄位，讀實際instance total；現有protobuf／C ABI layout未改，Hero JSON只增加向後可讀的optional欄位。舊存檔無map時使用max(base,remaining)保守界線，不宣稱還原未知的原始總長。已有但非法total拒絕owner hero HUD，避免製造ready技能；stale已退休entry不影響ready base並在正常tick清除。無英雄／營地名稱分支。

新增3core測試覆蓋tick／refresh／clear／expiry／nonpositive dt／極大dt、serde新舊資料、非法present total／stale移除。新增1base正式60Hz測試：20秒技能-50%／+100%後實際10／40秒，成功cast後total正確，modifier remove／pause／tick／serde都保留當次總長。新增2client測試覆蓋披露Hero JSON→owner HUD，actual half-progress／owner isolation／legacy fallback／非法total拒絕；既有native CastTimeModifiers加上10與40秒actual cooldown分母下半進度與精確tooltip總長斷言，避免只檢查相同比例而誤收stale狀態。RustV2所有程序退休後封存native CPP，再補強這些native斷言與40total/20remaining的既有完整viewport截圖，Rust不再變更；最終原生驗收與像素檢視包含這個20秒倒數。

1819passed/0failed/9existingignored；World71全執行；3433正式輸入一致；150份證據／34PNG；health10 samples／max171秒。

|套件|通過|既有ignored|
|---|---:|---:|
|abi|15|0|
|content|42|0|
|templates|116|0|
|codegen|69|0|
|core|967|0|
|base_content|321|0|
|bridge|136|1|
|client|153|8|

backend／UE5.8／stage DLL與import library SHA成功；World71全執行、0failed/notrun，保留既有1warningcase／4RuntimePresentationReset警告。34PNG中28與前批像素相同、6逐張實際view；既有施法提示及viewport／paintcrop仍可讀。本批不宣稱新增cooldown專用圖像或切換OS DPI。firstkillHUD右下裁切／部分map hero-focus名牌重疊仍known。

正式inputs exactset/SHA/bytes/mtime一致，原始logs／ready／owner/watch receipts與正式4sources封存，全部readers/watchers及health退休reap後獨立核對archive bytes/SHA及原始來源。V1在bridge編譯client時E0433：誤用僅dev-dependency可見的omoba_sim；core967/base321已成功但bridge/client未完成。所有V1程序退休後保存4精確來源與原始logs/ready/false receipt，再改用既有正式依賴omoba_template_ids的Fixed64 reexport，完整V2重驗；不新增依賴，不抹除失敗。11外部/MVP33/44、完整combat replica、持續引導及有效Mana cost前端提示仍待完成；不將本批視為整個遊戲達LOL完整度。無真networkgames／quota reset／commit/push／omfx維護。原無人值守期限2026-10-25 00:33:38 Taipei不變。

[證據](D:/omoba/docs/acceptance/2026-10-23-effective-cd/index.json)；[錯誤](D:/omoba/docs/plans/2026-10-23-effective-cd-errors.md)。
