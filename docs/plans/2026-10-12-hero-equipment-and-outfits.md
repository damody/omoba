# 英雄衣裝與原始武器接續計畫

## 當前範圍驗收成功（2026-10-12）

Codex 自行完成，無子代理。`.ai-collab/hero-equipment-accepted-v1/summary.json` 綁定 ordinary NoEngineChanges／NoUBA／nocache build-v10 與完整本機 hero-equipment-local-v7：169 原生測試全部成功、17 既有 warning 保留；實際音訊三段 PCM、397 viewport cases／177 wire intents／138 stages、69 Blueprint 零 compiler error／warning／load failure，程序／profile／listener 清理成功。69 Blueprint SHA／mtime 與原模型／頭部輸入輸出再核對未變。8 個角色／世界 scoped native 與55 codegen／722 core／40 bridge IPC、4 all-target checks 明示沿用未變來源，1既有capture-dependent ignored保留；22角色＋2世界原圖已直接檢視，其餘生成圖不宣稱逐張目視。衣裝、原始頸部幾何、44.8cm短劍掌心握點接受；E919–E939與失敗報告保留。父項仍12/44，完整三路／authority／DPI矩陣沒有因此勾完。立即續做中立守衛與公開戰鬥呈現。

由Codex自行實作，不使用子代理，不另要求確認。接通用骨架小兵後，依6.2補兩個基底角色衣裝與缺少的近戰武器；完整驗收最後執行。

1. 先盤點已驗證Quaternius Peasant／Ranger服裝與各自獨立骨架，不把不同rest pose直接當作可共用的leader pose。呈現profile可共用來源模型，但英雄ID／權威技能／角色名與各自動作保持獨立；不得讓正式英雄維持未完成基底內衣造型。
2. 近戰裝備優先使用Kenney官方免費CC0原始Mini Dungeon武器，來源頁 https://kenney.nl/assets/mini-dungeon ，2026-10-12實際查官方頁確認版本2.0／含weapons與CC0。下載、archive SHA、included license、每個OBJ／MTL／texture原bytes與bounds先驗；無合適來源則換正式可驗來源，不用佔位cube冒充完成裝備。
3. 復用通用原始模型匯入與native bone attachment metadata，保留既有世界模型／材質packages。握點與方向以來源座標與實際骨架reference推導，reference bone scale normalization沿用通用機制；不能以放大Actor掩蓋單位錯誤。
4. 材質槽、姿勢、手部附掛在idle／move／attack／cast／death／respawn事件中驗證，完整原圖要能看到裝備跟隨手部並維持比例。隊伍識別只使用當前公開owner/team，不推測隱藏敵方資訊。
5. 來源檔／packages不改寫、負例零寫入、重複匯入零保存、正式普通NoEngineChanges建置與受影響native／Blueprint相容驗證後才記錄範圍成功。控制／權威／完整三路／DPI矩陣保持原驗收範圍，不因素材匯入勾完整父項。

## 握點與比例修正

頭部死亡姿勢的可見頸部缺口：確認六個基底／衣裝骨架的 neck_01 與 Head reference pose 同性別完全一致，禁止任意平移頭部。保留原兩個 Head subtree 產物，新增 male_head_neck／female_head_neck，使用既有通用 weighted subtree 抽取 neck_01 並以 Head 為 pivot，涵蓋實際原始頸部幾何。負例全批零寫入、舊資產 SHA／mtime、重複匯入零保存及所有角色動作重新驗證。

Prototype Kit 原始預覽同樣粗握柄，未匯入。Quaternius 官方公開來源 Sword 與 Sword_2 都回傳額度已滿 HTML，來源檢查拒絕，不繞過。回查 Mini Dungeon 實際米制來源後，先前額外 scale=2 將 10.8cm 握柄放大到 21.6cm 是呈現比例錯誤，並非必須換來源。保留原始 44.8cm 短劍 scale=1，四指根部距手腕平均 10.7cm，取掌心約 (-0.4,-5.4,0)cm；原握柄中心 -5cm 由 metadata +5cm 補償。再驗各動作實際手掌握點與原圖，不以縮小遮掩缺件或改寫原始幾何。
