# 公開場景物件的通用呈現

接續世界模型匯入與塔／基地模板，使用既有公開 tree occluder 與 collision terrain。Codex直接實作，不使用子代理。

1. 樹使用可設定的 soft static mesh 與共用 ISM，依來源實際 bounds 對齊公開圓形 footprint，底部對齊公開地面。保留 no-collision／不影響導航；模型不改權威遮擋半徑。缺模型時保留既有診斷 primitive，不替換為其它角色。
2. 一般世界樹只依完整 presentation snapshot 更新；control frame 保留，完整空快照退休，重連 reset 清除。對 count、指標、finite position／radius／world scale 設限後才配置，壞批次原子清空，不從隱藏單位或後端非公開狀態推測擺放。
3. 比較已驗 transform／mesh 清單，未變不 ClearInstances 或 AddInstance，避免每frame重建固定地景。world scale／metadata 改變必須重算，不能只比較原 game coordinates。
4. terrain rectangles 保留準確公開邊界。岩石可作裝飾，但不能用單一不規則石頭冒充完整矩形碰撞；先做可辨識的邊界及材質，再考慮有界沿邊裝飾。沒有 source geometry 依據時不任意創造河道或叢林。
5. 原生驗證來源 bounds／半徑／底部、scale roundtrip、mesh替換／fallback、批次無效／空快照／control／reset及 unchanged reuse；同正式bridge生成場景截圖，直接目視來源貼圖／法線／比例。最後完整三路與全部操作驗收獨立進行。

目前實作進行中：TreeVisualMesh soft metadata 預設原始 pine，缺檔診斷 cylinder；依實際 bounds 做均勻 footprint／底部對齊、同元件 ISM、公開 snapshot 與 reset／control 生命期。變換／mesh 不變不重建，count 4096 在 frame shape 層先拒絕（保留整個既有合法視圖），單筆語意／幾何非法則原子清空樹批次。原生新增 PublicTreePresentation 與正式橋接兩公開樹截圖／像素斷言；尚待 build／執行／實際圖片，因此未宣稱通過。

公開樹木範圍已接受：`.ai-collab/public-world-trees-accepted-v1/summary.json`。ordinary no-cache build-v1成功60.04秒；fresh native-v1 exact13全部 Success，保留5既有fixture warning（FrameContract刻意拒絕1、RuntimePresentationReset缺world context4），新增樹與正式橋接render測試0 warning。三張當次原圖直接目視；正式橋接的兩株原始pine、塔／基地、英雄與道路同圖正常，原模型面序與回城尺度保留。半徑／底部／uniform比例、scale100→1→100、原模型替換／null fallback、NaN整批清空／恢復、count預拒絕、unchanged reuse、control保留／空快照退休／stop重置均驗。原12模型package SHA／mtime不變，stage再驗。沒有新增權威對局、ACK或全世界接受；terrain材質與矩形可讀性下一步實作。

Terrain source 已補 soft stone material、材質獨立metadata cache與native診斷fallback；有限但極端的矩形用double運算與共用厘米上下限拒絕，count32在 frame 層先拒絕。保留每個公開矩形的完整正確 bounds，不使用不規則岩石替代碰撞形狀。fixture新增scale往返／metadata變更不重建／NaN與極端finite、倒置末筆清空及恢復；同橋接場景加入兩公開矩形。當前等待 ordinary build-v2／fresh native／原圖驗證。

build-v2普通成功57.77秒、native-v2 exact13 Success／既有5 warning，但側面world XY材質在原圖成直條，E908保留並拒絕terrain接受。正在補原始石材共用貼圖的world三向投影獨立材質，原grass／stone四package不改。待build-v3、正式新增材質匯入／負例／repeat、fresh native與當次原圖；父項仍12/44，完整世界驗收留最後。

當前 terrain 範圍已接受：`.ai-collab/public-world-terrain-accepted-v1/summary.json`。ordinary no-cache build-v3成功13.15秒、v4只補detail capture成功9.56秒。正式terrain-triplanar-import-v1：10負例零寫入，新增1材質／共用1既有texture，repeat2verified／0imported／0saved，原4地面package與12模型package SHA／mtime全不變。24節點graph／compiled shader／原PNG像素／SRGB／mips／wrap均驗。native-v3真13 Success／既有5warning保留；v4當前受影響2項Success／0warning，其餘11項明示沿用，未冒稱fresh13。當次bridge全景與近距離三面PNG直接目視：頂面與兩側石材尺度正常、無直條；v3原模型及回城原圖明示沿用已目視。尺寸／位置界限、double中間值、32預拒絕、非法末筆原子清空與恢復、材質更新不重建ISM、scale往返及scope清理均驗。完整三路／creep／英雄衣裝與最後完整操作驗收仍pending，12/44保持；接通用非英雄骨架父類與lane creep。
