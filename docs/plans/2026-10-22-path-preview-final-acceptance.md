# 2026-10-22 沿路施放提示最終驗收

OpenSpec complete-swept-path-preview 5/5完成。本批接續真實沿路衝刺，補上內容驅動的意圖路徑輪廓與技能說明。

## 實作

共用AbilityDefinition新增可選typed aim_geometry::swept_path／width_key契約；兩端內容生成器校驗instant active/ultimate point、完整rank、正值且有界range／width。不依英雄名、技能ID、Custom字串或特效半徑推測。fire_dash顯式宣告既有dash_width，真handler仍消費同一extras。任意ID與不同key的codegen案例確認可重用。

Generated metadata發布逐rank SweptPathWidths，bridge依目前公開rank與世界比例送入既有owner HUD payload。owner在合法point且已學習／已知射程時保留寬度；替換／未知型別／非法值／角色退休清除。無ABI／protobuf／CABI改動。

新共用C++ builder產生51點封閉capsule，寬度為直徑，沿地面依請求距離與射程取較小值，含起終端；零位移與非有限值清除輪廓。owner實際paint使用同一builder及既有投影／split viewport／DPI裁切與pending aim生命期，不掃描actor或做隱藏碰撞查詢。射程圈與point-area預覽共用輪廓投影，既有單位提示保留。

技能tooltip補寬度、超距離截短、遇地形停止與伺服器結果說明；瞄準時超距離衝刺不再要求靠近目標。此輪廓是意圖，不預告已裁切地形終點、不阻擋或發送命令、不代表ACK；真實固定點碰撞、root拒絕、mana／CD回滾維持既有正式規則。root專用預覽文字、定時移動／火焰殘留、更多特殊技能幾何仍待完善。

## 集中驗收

RustV1 **1938通過／10既有ignored／0失敗**，含content33、sim72、template76、core919、codegen61、base277、backend190／1、bridge135／1、client151／8、ABI15、restart9，含generation共12工作全部成功。新增兩個schema測試與一個arbitrary-ID codegen測試；base正式60Hz衝刺、薄牆、rank、root／零位移／封閉回滾亦全量回歸。四Lua契約成功。

正式backend-v1與UE5.8 build-v3成功；最終UBT **8.94秒**（首次build-v1為84.32秒）。七DLL與import library另做SHA驗證。WorldV3 **52成功／0 Failed／0 NotRun**；warnings逐項留在本批errors及engine log。新增SweptPathPreview真native案例涵蓋四rank、輪廓端帽／閉合／寬度、射程clamp、斜向短距離／地面高度、zero／nonfinite、owner／tooltip替換與退休。新PNG擷取實際SOmHudToolTip，供文字可讀性驗收；不將tooltip圖宣稱為真對局路徑錄影。

兩張既有地圖各74影格／6片段；共11張PNG在World worker／watcher退出後目視，包含新技能提示及10張地圖／HUD／角色既有功能回歸。**3393來源／資產**前後集合、SHA／size／mtime一致。**123份證據**封存並獨立逐byte／SHA比對。健康監測**9次**、最大間隔**171秒**；7組最終worker／watcher與8組V1／V2過渡工作全部退休，monitor退出。

## 錯誤與剩餘

WorldV2另發現ClearHeroSnapshot未清路徑寬度；已補共用清理入口並維持嚴格斷言。目視同時修正tooltip fixture的world-unit／rank誤用，正式bridge比例規則未變。WorldV1發現TArray閉合自引用斷言，exit3。已先複製首點再Add，保留失敗engine log、receipt與來源快照；取消仍等待的source-v1 checker後、capture退出後才修復，重建並以WorldV3完整接受，未把崩潰當環境問題或覆寫舊證據。其他查詢／literal編輯錯誤記於2026-10-22-path-preview-errors.md，總登錄E1492。

持續引導、定時移動／殘留、完整combat replica接受、召喚整組commit原子回滾、更多英雄／道具／召喚師、美術音效仍待完善；全遊戲盤點已更新。MVP33/44與11項外部ACK／LAN／實際OS DPI／固定效能接受維持pending。零新增真網路對局、quota reset、commit／push；omfx未維護。

證據索引：../acceptance/2026-10-22-path-preview/index.json。
