# 原生動畫生命期當前範圍畫面審查

2026-10-12，直接由 Codex 審查，未使用子代理。證據為 `native-lifecycle-build-v5` ordinary NoEngineChanges／NoUBA／nocache exit0，以及 `native-lifecycle-native-v3` 實際十項 Success／零warnings／Editor與workflow皆0。

當次40個768×768原始PNG位於 `.ai-collab/.bridge-data/worker-pool-machine-v1/native-lifecycle-native-v3/characters/`。六個正式generated hero各idle、attack、cast、hit、death、respawn六圖；另四張原始來源動畫診斷圖。已逐檔讀取審查，未裁切、補畫或修圖。

| Hero | 畫面結果 | 尚未完成的美術 |
| --- | --- | --- |
| Date Masamune | 站姿／劍攻動作、施法伸手、受擊後仰、死亡倒下及重生回姿可見；身體比例與材質正常，無青紅診斷球 | 目前仍為男性基底內褲，尚無正式服裝與實體劍 |
| Training Luminary | 六事件對應姿勢、女性身體比例與材質正常 | 目前仍為女性基底內衣，尚無正式服裝 |
| Training Apprentice | 棕色服裝、頭部附掛與施法／受擊／倒下／重生保持正常大小 | 尚無獨立武器／角色配件 |
| Training Vanguard | 棕色服裝，拳擊與施法姿勢明確不同；頭部與倒下姿勢正常 | 尚無正式英雄配件 |
| Training Ranger | 綠色兜帽與服裝，頭部跟隨動畫，四生命期姿勢正常 | 尚無弓或其他正式武器 |
| Training Support | 綠色服裝與頭部跟隨動畫，四生命期姿勢正常 | 尚無正式手持道具 |

Luminary、Apprentice、Ranger、Support 的 attack／cast 都設定同一個 `Spell_Simple_Shoot` clip，取相同40% cursor的PNG逐byte相同；各組SHA-256已實際比較相等。它們是各自真正事件選取同一段美術的結果，不能宣稱已有不同攻擊／施法特效。40個檔案共36個不同內容；相同內容以已讀取的attack原圖及逐byte等價確認。Date／Vanguard的attack與cast不同，圖像也不同。

死亡圖取clip40%作為倒下過程，並非最終躺地pose；新原生測試另驗實際clip終點凍結、初次dead snapshot不重播死亡、死亡途中不被idle／attack覆蓋、重生後恢復最新狀態。所有生命期clip另驗實際active asset、CPU skin finite positions、reference root scale及頭部附掛物理大小，未用站立高度門檻拒絕合法倒地動作。

原始來源四圖是灰色資料診斷mesh，實際尺寸仍只有約1.5–1.83公分；鏡頭依其posed bounds取景，actor未縮放。它們不代表正式角色的尺寸／材質，也不是正式動畫來源直接播放接受。正式六角色使用已校正的RetargetV4，來源限制與校正紀錄維持E849。

E855的v2原圖保留：數值全過仍被診斷線框遮住，未接受。v3在正式預設關閉attack／cast／hit診斷繪圖後，40個檔案沒有該青紅線框遮擋；沒有只在capture關閉debug showflag來掩蓋正式行為。

接受範圍是六角色原生動畫事件生命期與既有基本Blueprint／RecipeV1相容性。公開回城、完整combat VFX、Saika擴充生命期、服裝／武器、隊伍辨識、世界場景與最終跨入口驗收仍未完成，父項6.2／6.4不勾完。未新跑權威對局、沒有ACK。
