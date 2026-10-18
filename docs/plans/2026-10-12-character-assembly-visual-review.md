# 角色組裝原圖審查

本次直接由 Codex 實作，未使用子代理。檢視來源：`.ai-collab/.bridge-data/worker-pool-machine-v1/character-assembly-native-v5/characters/`，16 張768×768原始 scene-capture PNG，未後製。

## 逐組觀察

- Date／Luminary 各 idle、attack：膚色、頭髮、眼睛及基底服裝已實際上色；肢體比例正常，兩動作確實不同。仍是第三方基底內衣造型，Date 劍動作尚無武器，不能稱為完成的英雄造型。
- Apprentice／Vanguard 各 idle、attack：棕色服裝、鞋、護腕、頭部材質可見，頭部隨動作正常附掛，沒有百倍縮小或放大。
- Ranger／Support 各 idle、attack：綠色兜帽、護甲、褲、鞋、手與頭部上色；兩者使用不同性別 mesh／skeleton，頭部在所取姿勢中維持合理大小。角色辨識、職業裝備與隊伍色仍需後續設計。
- 原始 animation_library 的 TPose／Idle／Jog／SwordIdle 四圖：灰色來源 mesh 姿勢可辨，僅用於診斷；相機依1.5–1.83公分 posed bounds取景，actor未放大。原始raw動畫的root比例／方向錯誤仍保留在來源資產，正式消費端使用明確reference clip校正後的 RetargetV4。

## 實際驗證與範圍

`character-assembly-native-v5` 真正兩native Success／0 errors／0 warnings；普通 `character-capture-residency-build-v1` NoEngineChanges／NoUBA／nocache exit0。六個實際生成類別各自載入獨立 skeleton，12個設定動畫資產皆解析成功，idle／move／attack事件選擇並播放正確clip；CPU skin尺寸對照各自reference height，骨骼附掛尺寸、所有材質槽、shader map、實際有色pixel、重複初始化不重建附掛與retirement清理皆有斷言。

截圖fixture以角色實際mesh預載貼圖、一次真capture warmup及有界residency等待後保存，沒有在正式每entity每frame加入阻塞。E841–E850的原編譯、程序與視覺失敗保留，沒有覆寫為成功。

這是角色載入、比例校正、材質與三種動畫的組裝基礎；不是最終遊戲世界、完整服裝、武器、隊伍色、選取輪廓、cast／hit／death／respawn／recall事件生命期或全套LOL操作驗收。父項6.2／6.4仍待完成，總進度12/44不變；無新權威對局或ACK。
