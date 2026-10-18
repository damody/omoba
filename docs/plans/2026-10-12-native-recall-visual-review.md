# 公開回城當前範圍原圖審查

實際來源：`.ai-collab/.bridge-data/worker-pool-machine-v1/native-recall-native-v1/summary.json`，十個測試全部Success、零測試warnings，Editor／workflow exit0。普通 `native-recall-build-v1` NoEngineChanges／NoUBA／nocache成功，UBT實際1154.11秒；建置仍有既有plugin依賴提示及E859三筆float常數warning，不宣稱整個建置零warning。

46份768×768原始PNG全部核對當次SHA，見 `.ai-collab/native-recall-native-v1-capture-review.json`。六角色的idle／attack／cast／hit／death／respawn共36份，與先前實際逐張審過的 `native-lifecycle-accepted-v1` 原圖byte相同，明示沿用先前審查；沒有重作截圖或假稱新看了36次。六份新增recall與四份變動original-source已直接開啟當次原檔逐張檢視。46份有42種不同內容，四組attack／cast相同內容的原因沿用前次紀錄，沒有要求相同動畫被假造為不同畫面。

| 當次新增回城原圖 | 審查結果 |
| --- | --- |
| date_masamune-recall.png | 站立展臂施法姿態，身體比例與材質正常，沒有巨大診斷線框遮擋。仍為原基底短褲造型，正式服裝／武器待補。 |
| training_luminary-recall.png | 女性基底展臂姿態可讀，四肢未扭曲。仍為基底內衣造型，不能算正式服裝完成。 |
| training_apprentice-recall.png | 棕／米色衣物、靴與頭部正常，手臂姿態可讀。 |
| training_vanguard-recall.png | 淺色上衣、腰帶、靴與頭部正常，展臂姿態可讀。 |
| training_ranger-recall.png | 綠色兜帽／服裝與肩甲正常，手部未飛離、頭部未放大。 |
| training_support-recall.png | 女性綠色兜帽／服裝與靴正常，體型與回城姿態可讀。 |

四份 `original-source-A_TPose／Idle_Loop／Jog_Fwd_Loop／Sword_Idle.png` 已另行檢視：灰色原始來源的reference、待機、跑步與持劍空手姿態可辨識，未見爆炸／非finite變形；它們是來源診斷，沒有正式角色材質／服裝／武器，不能代替六角色正式畫面。與舊圖SHA不同，沒有宣稱沿用這四份。

單張recall原圖只證明該slot可渲染，持續loop、重複不重啟、pause／resume、cast／hit不搶channel、權威clear恢復movement、死亡打斷、重生不恢復舊channel、hide／reveal、missing clip fallback由當次真actor測試驗證；ABI26 flag實際走WorldBridge到原生hero，由當次AbilityCastCue端到端測試覆蓋。真Recall／Stop經正式visibility producer、可見投影／清除、replica與bridge的8項Rust測試、三ABI gate、五正式all-target checks及MSVC header smoke另有當次證據。

這次只接受公開回城動畫與生命期仲裁。回城環／完整戰鬥VFX、新增材質commandlet、Saika擴充生命期、服裝武器、team可讀性、地圖與完整跨入口／DPI／LAN均未由這些原圖接受。新的VFX commandlet來源是在上述回城建置成功後新增，尚未建置；不把它混入回城二進位證據。無新增權威對局／ACK，父項6.4與12／44進度維持pending。
