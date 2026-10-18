# 原創地面紋理候選（未匯入）

使用內建 image_gen 產生兩張獨立原創候選，原圖均已逐張目視，完整 prompt、來源、SHA256、實際 PNG 尺寸與格式保存於 `.ai-collab/candidates/terrain-textures-v1/manifest.json`。不是CLI／外部下載，不使用現有遊戲貼圖。固定Lua `.ai-collab/stage-original-terrain-candidates.lua` 先驗全部輸入再複製，工作區副本與產生器原圖byte-identical；原圖保留。

- `.ai-collab/candidates/terrain-textures-v1/grass-ground-v1.png`：低對比手繪草地與泥土。
- `.ai-collab/candidates/terrain-textures-v1/stone-paving-v1.png`：低對比不規則石板與土縫。

兩張實際都是1254×1254，不能把prompt要求1024當成實際尺寸。僅候選，未UE匯入／材質接線／權威幾何映射，也未驗無縫邊緣、mip／LOD、地圖尺度、實際光照與戰鬥可讀性。來源藝術不可當作完整lane／river／jungle幾何或實景驗收。未修改遊戲source或正在進行的模式介面批次；後續world presentation必須依正式published geometry和實際材質資源接線，保留碰撞／導航／fog。完整驗收最後。
