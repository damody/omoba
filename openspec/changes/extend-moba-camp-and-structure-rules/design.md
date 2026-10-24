# 設計

營地保留主怪公開身分，附屬怪由 bounded Lua member definitions 生成；共享索敵與 reset 狀態，每隻怪保有獨立血量、攻擊節流與擊殺獎勵，全部死亡後才計算整營重生。資料產生器／公開呈現 catalog 一起處理，禁止只為特定營地名稱寫 runtime 分支。

附屬怪使用40000起獨立保留ID，追加map不會改既有普通或附屬身分。地圖以同一組牆形經轉置與中心鏡射產生12塊野區牆，加上既有2塊terrain共14；authoring檢查lane／出生／主怪與附屬怪本體不可被阻擋。兩份地圖各16營地、32隻怪，視野／溝通bounds亦由全部營地成員出生點導出。牆在本輪提供碰撞與繞行，不宣稱已實作LOL完整草叢視野。

重置採固定點 patience → soft return → hard return。軟返回可在營地範圍內重新接戰，硬返回維持免傷；恢復透過現有 Outcome::Heal。所有私有狀態與成員納入 deterministic hash，不公開仇恨／patience。參考 Riot 12.22 的軟重置概念，不宣稱現版數值完全相同：https://www.leagueoflegends.com/en-us/news/game-updates/patch-12-22-notes/。

基地解鎖策略編成到共享 const，支援既有 all_lane_towers 及任一路線突破 any_lane_towers；正式資料使用後者，避免要求拆光三路。這不冒稱已具備完整 inhibitor／nexus turret 機制。

施放提示使用現有 owner、公開射程與公開目標取樣；提供射程內／外、缺少目標或落點、距離、操作方式，不改輸入提交契約，不猜測後端尚未公開的技能碰撞／命中。提示沿既有 preview 生命期退休；不增加逐 actor 掃描。
