# 公開地圖的河道與野區呈現

Codex直接實作，不使用子代理。接續已接受的光照基準（25／44），補6.1的河道與野區外觀；完整世界美術與跨環境驗收維持獨立。

## 計畫

1. 在正式moba_maps Lua目錄新增可選、版本相容的presentation surfaces：具寬度的river polyline，以及jungle矩形地表區域。嚴格限制kind、ID、座標、寬度、點數與總容量；舊內容缺少此欄位維持空集合，不猜測地圖名稱或從敵方資料推導外觀。
2. 同一份資料經template IDs生成公開const與catalog hash。只有正式metadata解析成功的public_map才投影到bridge；保留所有既有lane routes、camp leashes、碰撞／導航／fog geometry與遊戲規則。這些地表表示步行可穿越的河道／野區材質，不新增假阻擋或可互動物件。
3. 以獨立OmMapSurfaceEntry／points投影至C ABI並附加Frame欄位，明確提升ABI版本；不把視覺河道偽裝成可供小兵尋路的lane，不借用reserved flags或把寬度塞進bit欄位。renderer IPC維持既有public metadata解析，不新增不必要的私有遊戲資料。
4. Unreal共用tickless surface actor：河道復用正式spline route幾何與連接方式，分離岸與水；野區以真正地圖區域貼合地表。材質為原創水／岸配色及既有原創草地貼圖的野區變體，保留provenance；僅建立新package，不重寫既有美術。無碰撞、導航、目標或每entity操作。

   `width`定義為含岸的完整河道外界，水面占其中80%，確保材質風格不把geometry擴到公開邊界外。野區以min／max區域bounds及圓角邊緣呈現；它是可穿越的地表色澤，不宣稱是阻擋樹牆。既有兵線仍在最高地表層跨過水面，不改尋路或建立假互動橋。
5. WorldBridge按來源、ID、kind、尺寸與內容hash更新；不變資料零幾何重建。source／map切換、非法批次、失效公開地表與teardown退休整組表面及子actor。材質缺失明確失敗，不拿debug material當正式結果。

   世界與小地圖共用一個有界decoder，於lease有效期內複製ID與點位；小地圖以同一transform畫河道、岸與圓角野區，再疊兵線、碰撞輪廓、fog及單位。裝飾不能擴大小地圖bounds，也不新增hit-test或遊戲指令。合法frame中的非法裝飾批次只清空該可選layer，保留合法fog；整份frame header被拒時不部分改動最後合法scene。
6. 先Rust目錄／hash／投影／ABI與原生geometry／life-cycle測試，重新匯出兩張正式compiled map的實際ABI資料，驗河道與野區當次viewport原圖。一般增量先驗當前功能；最後才做完整Rust／ABI／NoEngineChanges／native／458操作／69 Blueprint與三PCM，保留所有失敗，不新增權威對局或重設10／10額度。

## 狀態

**完整本機接受成功**：`.ai-collab/map-biomes-accepted-v1/summary.json`。普通build-v5 NoEngineChanges／local／NoUBA／nocache成功15.49秒，1024來源一致。Rust真1168測試與五maintained checks成功，9項既有ignored明示；ABI30正式cbindgen及MSVC layout成功。fresh final-v1真197 native（12筆既有warning保留）、full458案例／222 wire／157 stage／13筆游標觀察、69 Blueprint零compiler／load診斷、三段實際PCM及全部清理成功；69 Blueprint原SHA／mtime不變。focused native-v4八項零warning，17張當次原圖已直接檢視；三新材質獨立repeat驗證3／保存0，原地表與新package不變。6.1依本批與既有正式模型／地形接線完成，進度26／44；不冒稱完整美術、語音、錄影或跨PIE／OS DPI／LAN／權威ACK。無新對局、無子代理，E1260–E1275與所有拒絕證據保留。以下pending段落保留實作過程，下一步`2026-10-18-hero-portrait-coverage.md`。

source-v8的ordinary build-v5成功15.49秒，fresh native-v4真8項Success／零warning；兩張全圖與combat原圖確認幾何修正後黑塊／破面消失。兩map各七張actual game viewport，加三張world原圖，合計17張已直接檢視；1024來源於建置及native後一致。當前功能已進入最後map-biomes-final-v1完整本機驗收，69 Blueprint SHA／mtime在native階段先凍結；完整197 native／458操作／69 Blueprint／三PCM及全部清理尚待terminal結果，不能提前記接受。

native-v3功能8／8 Success、零warning，已證明未知／外隊拒絕、正確team的5surface、來源回收及實際Bridge ground。但直接檢視兩張game-viewport-biomes原圖發現野區end cap黑塊／破面，**畫面拒絕，未接受6.1**，原圖與綠燈報告均保留。source-v7在退出後核對1024來源一致；接source-v8僅調整圓柱cap體積比例（深度為直徑1%，埋入地下，top仍0.6cm），新增真instance bounds斷言，不改材質／光照／碰撞／公開XY。build-v5執行中，尚待同視點重截圖，詳E1275。

refresh-v3現已正式成功（初始化161.36秒、內建普通gate2.92秒），source-v6核對1024來源一致後，明確初始化鏡頭兩個bounds區域變數以消除C4701。最終source-v7仍1024個維護輸入，ordinary map-biomes-build-v4 NoEngineChanges／local／NoUBA／nocache成功10.84秒，建置後SHA一致。fresh native-v3已啟動，待當次actor／minimap／圖像及最後完整驗收；沒有新權威對局或ACK。

完整Rust gate map-biomes-rust-full-v1現已全部成功：1168測試、9既有ignored及五maintained checks（core／client／bridge／backend all-target與bridge no-default lib），1024來源每階段及最後一致。之後才修改四個Unreal入口／fixture檔與一個Lua test，Rust／ABI來源未變；source-v6仍1024輸入已另凍結。stage fixture8項已實跑成功，接普通build-v3及fresh native-v3，後續完整native／458／69 Blueprint／PCM仍最後進行。

refresh-v2正式完成並退休bootstrap checkpoint；ordinary map-biomes-build-v2 NoEngineChanges／local／NoUBA／nocache成功2.58秒。正式素材首次建立3／保存3，獨立map-biomes-material-repeat-v1重跑驗證3／保存0／建立0，原地表與新package SHA／mtime不變；check-only成功。ABI30 MSVC header smoke實跑成功。當前完整Rust gate已通過1168測試（template56／core782／client147／bridge127／codegen56，9項既有ignored保留），仍在執行其餘正式all-target checks。

native-v2真8項中7成功、1整合fixture失敗：沒有started runtime的world其active team為0，正式fog守衛因此不建立surface。將修test-only expected-team入口，正式仍從runtime取team；整合圖改使用Bridge真正建立的ground，驗未知／外隊拒絕及正確隊伍5actor，不再另手動建立地表。獨立surface geometry另需補world context消除24個回收warning；原false／26warning與E1269–E1272保留。此時尚未產生合格河道／野區整合畫面，6.1不勾完成。

正式素材工作流已加入`scripts/ue_map_biome_materials.lua`及一般Unreal建置入口：三個新package具有可攜provenance、來源／貼圖／package SHA檢查，已存在且符合時不啟動Editor；驗證入口只檢查，不修復或偷建置。新工具11項自足測試成功，含修改／缺檔／不合法路徑與來源變動；PCH預讀17項成功。source-v5實際1024個維護輸入已凍結，refresh-v2已編譯連結C++，正執行首次native材質建立；仍待當次素材、一般NoEngineChanges與畫面驗收，不能把編譯完成當作功能驗收。

盤點確認正式MapConst已有lanes、terrain與jungle_camps，但未提供河道／野區外觀契約。bridge目前只有可行走路線與實際阻擋terrain矩形；不能在UE按固定座標硬畫河流，也不能把河道加入lanes改變小兵路線。先補共用內容契約與const生成，再接投影及Unreal。

共用契約第一批已完成：map-biomes-catalog-v1實際24個Lua目錄測試通過、零ignored，包含新surface合法形狀／邊界／容量／ID與舊內容空集合相容測試；32個runtime-content測試尚未在該批執行，不宣稱全crate已驗。正式內容接續新增一條河道與四區野地，兩張地圖共用同一份定義。依既有canonical_template_hash設計，完整內容（包含呈現）參與catalog hash，外觀變更會要求新內容身分與全部正式元件重建／stage；不私自改現有hash規則，也不改lane、camp、碰撞或fog geometry。

map-biomes-rust-v2已實跑197項（template/runtime56、bridge127、core map3／fog11）全部成功，1項既有bridge ignored明示保留；header由正式cbindgen重生為ABI30。新surface entry為24bytes／align4，FrameHeader為752bytes且舊欄位offset保留；retirement／舊lease保有資料／獨立lane與collision已驗。map-biomes-export-v1再以實際ABI匯出兩map，各3條lane／5個surface／13個surface points，schema2；全程0 authority games／ACK。後續Rust只增加註解與fixture匯出斷言，完整當前來源仍須最終fresh回歸，不把前批結果當作未變source重用。

Unreal程式已接通tickless river／jungle actor、共用解碼器、世界與小地圖呈現、來源退休及材質producer，新增兩項原生測試（MapSurfaceModel／MapSurfacePresentation）。formal prepare-v1已成功產生／stage新內容與ABI，未覆寫保護的WorldBridge／FrameContract；source-v2真1022來源在EngineChange退出後一致。source-v4凍結相同1022個維護輸入，包含補充的actual viewport全圖fit及顯式world-time水波設定，接續refresh-v1後須回普通建置。材質與原圖仍未驗收，不能只用程式完成宣稱畫面已成功。
