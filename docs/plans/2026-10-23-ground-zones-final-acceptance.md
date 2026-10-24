# 持續地面傷害區域：最終驗收

`complete-ground-damage-zones` 完成4/4 tasks。新增共用DamageZoneSpec與BuffStore索引排程，透過既有stat-buff JSON傳輸，沒有擴充已滿的GameWorld ABI method table。固定世界落點、每次脈衝重新採樣精確圓形內存活敵人，排除友軍／自身／建築；來源、能力key與目標世代順序固定，進出圓形立即影響後續脈衝。

第一發在一個interval後發生，恰好落在到期邊界的最後一發先於移除結算。刷新替換落點／生命期並重啟clock；同施法者不同能力使用完整能力ID分開，施法者ID／generation亦包含在key。移除與施法者死亡／退休取消，不會附著到重用ID的新生命。暫停不推進game time。間隔至少0.25秒、最長30秒，每活動區間最多120次catch-up脈衝；catch-up使用本tick位置，不宣稱還原歷史子tick位移。

每個脈衝輸出RawDamage，沿用防禦、護盾、HP、歸因與TD profile單一結算入口。typed payload檢查完整整數、geometry、kind、非零有效profile與有限lifetime，禁止夾帶其他stat modifier。沒有區域時只查reverse index，避免新增全實體每frame掃描。無英雄／營地名稱執行特判。

炎襲保留200/300/400/500瞬間魔法傷害與原逐級暈眩；追加固定落點燃燒，每0.5秒25/35/45/55魔法傷害、持續3秒，共六次脈衝。圈形瞄準半徑不變，metadata增加第二段AreaEffect，技能說明由相同extras產生。脈衝沿用現有Explosion圈形cue，未宣稱獨立地面貼花／持續輪廓資產已製作。

## 集中結果

1790passed/0failed/9existingignored；3427正式輸入一致；148份證據／29PNG；health10 samples／max170秒。

|套件|通過|既有ignored|
|---|---:|---:|
|abi|15|0|
|content|42|0|
|templates|115|0|
|codegen|68|0|
|core|955|0|
|base_content|309|0|
|bridge|135|1|
|client|151|8|
六個新增core測試覆蓋15/60/120Hz、最後邊界、catch-up、刷新／移除、stable order、非法資料、live敵友／circle／building排除、抗性與shield一次、死亡／retirement／重用ID和入口不覆寫有效zone。三個新增base測試覆蓋非法rank metadata的零輸出，以及四rank正式60Hz輸入的爆發一次＋六脈衝、pause、entry/exit與source death；既有完整離線回放成功。

正式backend與UE5.8 build成功，部署DLL／import library SHA一致。World66全執行、0 Failed/NotRun，4個既有RuntimePresentationReset warning保留；另有compiled-map公共ABI export成功。29PNG中24張與上一批完全一致，5張逐張開啟檢視。這些是呈現回歸證據，區域權威行為由ECS與正式input證明；既有首殺HUD下／右裁切與部分hero/focus名牌重疊仍在。

## 原始錯誤與接受邊界

RustV1 ABI/content/templates/codegen成功，core因新測試缺specs::Builder出現四個E0599，未執行行為測試。保存10份原始來源與失敗logs、核對owner/watcher退休並reap後才修正匯入；最終scope全部重新執行。其他歷史版本的輸入與結果亦依證據索引保存，不混作最終revision證明。詳見errors MD。

本批是施法者生命綁定區域，死亡即取消；未提供獨立施法者存活的區域、持續引導、任意多邊形或歷史子tick移動重建。公開提示沿用現有圓形cue，尚無專用地面美術。11外部父項、MVP33/44及完整combat replica保持pending。

所有final與history worker／watcher核對creation token退休，單一170秒health timer停止reap。最終來源exact set／SHA／bytes／mtime與封存內容獨立驗證成功。[證據索引](D:/omoba/docs/acceptance/2026-10-23-ground-zones/index.json)；[錯誤／根因／預防](D:/omoba/docs/plans/2026-10-23-ground-zones-errors.md)。無真網路對局／quota reset／commit/push／omfx維護；原無人值守期限2026-10-25 00:33:38 Taipei不變。
