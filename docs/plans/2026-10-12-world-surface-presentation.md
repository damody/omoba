# 公開地圖地面與光照

由 Codex 自行實作，不派子代理。接續 authored-hero-accepted-v1；父計畫仍12/44，不提前勾完整地圖或角色品質。

1. 保留已產生、已目視的兩張原創草地／石板 PNG，搬入正式來源目錄並保存 generator prompt／原 SHA；不重新壓縮原圖。
2. 新增有界世界表面資產 commandlet：真 PNG 尺寸／來源SHA／受控路徑／材質graph／world XY UV／mipmap／sRGB／shader；第二個 Editor 零儲存，錯誤計畫零寫入拒絕。
3. 地面僅使用已通過公開 fog grid 驗證的 origin／cell size／columns／rows，不能把小地圖帶padding的示意bounds當權威邊界。不新增碰撞／導航／遊戲幾何；控制frame保留，完整不可用或stop清除。固定一個component，尺寸不變不重建。
4. 道路沿現有公開route points，寬與厚度分開，按實際mesh bounds正規化；有界／finite／零長段驗證，避免目前立方體等寬等高的厚條。
5. 缺少場景方向光時才啟用原生可調key／fill，避免重複照明；不新增逐entity每frame光照操作。
6. 正式ordinary nocache建置、原生控制frame／reset／static重用／invalid輸入／碰撞導航與實際D3D12地面道路角色圖；逐圖目視後只接受當前範圍。

比例盤點：現有 WorldUnitsToCm=100、原生骨架約180–240cm，但出生camera fit=900×WorldUnitsToCm，不能只依獨立角色fixture宣稱實景可讀。材質地面接線後要量測公開world視點內實際英雄像素與相機範圍；比例與完整地圖輪廓仍留在6.2／6.3，不能任意變更後端尺度或以錯誤scene放大補償代替來源設定。

## 當前進度

來源兩份 PNG 已正式保留於 scripts/art/world_surfaces/v1，1254平方原Source與原SHA不變。世界表面 commandlet 使用 Engine DecompressImage／CreateTexture，不受全域TextureFactory NPOT設定控制；Source保留1254，明確StretchToPowerOfTwo與FromTextureGroup產生mips，world XY UV 重複4000cm；grass／stone四packages尚待實際匯入。新單一WorldPresentationActor接在正式MinimapModel通過的fog view之後，不使用示意padding。full unavailable／stop隱藏，尺寸不變不重建；無碰撞／導航，缺少方向光才開key／fill。道路厚度獨立於寬，mesh bounds正規化，重複點不產生零長segment，兩端tangent為實際直線delta。

world-surfaces-build-v1 EngineChange 7.63秒失敗保留（E883）；refresh-v1 C++ API失敗保留（E884）；refresh-v2成功（source-engine19.00秒、內部ordinary20.07秒），未變C++的fresh ordinary build-v2再成功2.62秒（refresh false）。world-surfaces-import-v1 正在執行九種原生拒絕、首次建立／第二次零儲存；尚無原生渲染接受，父項12/44不變。

import-v1 已真成功4 saved→4 verified／0 saved、九原生錯誤計畫零寫入。native-v1實際23無Fail但workflow false（前綴意外帶出NativeMinimapInput），三Minimap context warnings保留；ground／route-corner／hero-ground-detail三原圖直接檢視，曝光偏亮所以沒有接受。build-v3 ordinary nocache成功29.59秒：缺燈場景新增低priority（-1000）manual EV、不啟用physical camera exposure，key2.5／fill0.75；已有authored light時不啟用這組燈或曝光。加入prelit真場景檢查與既有Minimap fixture正式world context。native-v2改明列23項，正在跑fresh原圖。E885完整保留。


2026-10-12 世界表面當前範圍接受：`.ai-collab/world-surfaces-accepted-v1/summary.json`。ordinary NoEngineChanges／NoUBA／nocache build-v5 exit0，fresh native-v3 exact23 Success／零warning。import-v1四packages首輪建立、次輪零儲存及九拒絕零寫入；當前SHA／mtime再驗不變。三張當輪original world PNG逐張目視，道路轉角共享join已封閉；八authored及11combat原圖與既有目視證據SHA完全相同，明示沿用；cast_secondary雜湊不同，當次重新直接目視成功。公開grid地面、非法／reset清理、重複座標退化、native／authored光照仲裁已驗。這是synthetic公開grid fixture，英雄世界比例、lane／river／jungle／塔base與完整世界驗收未完成；父項12/44、6.1／6.3仍pending，無新權威對局或ACK。E883–E887保留。
