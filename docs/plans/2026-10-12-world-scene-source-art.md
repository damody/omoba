# 場景模型來源與正式匯入

接續世界地面、比例與道路，仍需可辨識的塔／基地／自然阻擋物。由Codex自行處理，不使用子代理。

1. 使用原作者公開來源，保存原始下載、license、SHA及成員清單；不把商店縮圖或研究結果當成正式資產。
2. 先盤點可用塔／建物與樹／岩石模型，檢查來源材質、座標／單位／頂點／索引及實際bounds；選定原始模型後建立有界靜態資產匯入流程，二次零儲存與錯誤計畫零寫入。
3. 建立通用每模板native static／skeletal呈現接線，tower／base／creep使用各自明確metadata；保持既有Blueprint優先／required模式可用，顯示不改權威種類或碰撞。
4. 使用公開terrain／tree／route資料放置裝飾與阻擋輪廓，不新增隱藏單位位置，不任意重設地圖幾何。
5. 正式build、實際native／bridge／render與原圖目視後再接受對應範圍；完整圖與最後驗收維持獨立。

2026-10-12作者官網確認：[Medieval Village Pack](https://quaternius.com/packs/medievalvillage.html)列44模型、FBX／OBJ／Blend、CC0；[Ultimate Nature Pack](https://quaternius.com/packs/ultimatenature.html)列150模型、相同格式、CC0。這只是官方頁面盤點，尚未取得或檢查原始模型／included license。Medieval下載實際連結由作者GitHub原始HTML確認為公開Google Drive folder；web reader無法讀該folder，後續固定Lua透過native process host執行HTTPS下載，不建立PS／Python fallback。

研究結果：v1與v2原HTML及SHA已保存；folder可讀但included License.txt下載回HTTP200流量限制HTML，strict review不接受license或素材，原始summary不改。E895保留；不繞過quota，繼續評估其它作者直接來源。

改採Kenney作者直接下載：[Castle Kit](https://kenney.nl/assets/castle-kit)與[Nature Kit](https://kenney.nl/assets/nature-kit)。`.ai-collab/world-model-sources-v1/summary.json`真成功，Castle406成員／152模型格式檔、Nature3626成員／658模型格式檔；兩者均實際取得內附Kenney CC0 License.txt。固定Lua先檢ZIP32 central directory／成員路徑／單檔與總解壓上限／加密與symlink，再native tar解至fresh owned目錄，逐檔長度／CRC32／SHA256驗證。原ZIP、所有成員SHA與作者URL已保留。模型格式檔數包含同模型多格式，不是獨立模型數；尚未Unreal匯入／畫面接受。

正式投影修復計畫：新增獨立有界presentation identity schema 0x464f4715（版本1／長度／96-byte ASCII模板名稱），由公開Unit.id或既有腳本identity投影，不新增ScriptUnitTag或啟用callback。secure allowlist及filtered restore嚴格驗證，橋接依已公開structure role辨識塔／主堡，拒絕衝突名稱及壞資料。selective protocol升9拒絕舊peer，遊戲數值不變；先測試此資料鏈，再接正式模板模型。

投影當前驗證：world-identity-rust-v2真722個core lib測試、39個bridge IPC測試全過，core／client／server／bridge四個all-target checks全過；原E897／E898兩輪失敗完整保留。回城channel原先撞combat4713已改未使用4716，allowlist保留去重前後數量相等guard。這是Rust資料鏈驗證，尚未正式Unreal build／模型接線接受。

原始素材已選四項并保存於`scripts/art/world_models/v1`，所有OBJ／MTL／atlas原bytes與下載manifest SHA相同，兩作者license亦保存SHA。塔tower-square（88頂點／352三角）、主堡上部建物tower-square-top-roof-high-windows（134／516）、松樹tree_pineDefaultA（117／230）、岩石rock_largeA（42／80）；作者OBJ為Y-up，地面皆Y=0。匯入流程採讀取原OBJ的有界parser，交換Y/Z到Unreal並轉公分、反轉winding／UV V以保留法線及貼圖對應；不以改寫原始FBX／OBJ修比例。主堡素材是作者單一上部建物模型，並未宣稱完整城堡組裝。原始預覽與套件Sample已直接檢視，Unreal原圖尚未接受。

模板設計補充：對戰runtime-only單位不應混入TD可建造塔表或冒充其它模型。採獨立`world_units`呈現metadata，明確kind／catalog_id／ue.native_visual，保留高位32768–65535給有穩定明示ID的呈現模板；一般遊戲template維持原順序ID並禁止占用保留區。source loader既有完整Lua資料hash仍包含新表，Unreal codegen／bridge共用生成manifest；此表不產生TowerStats、不開放購買、不啟用腳本。先接塔／主堡，之後同介面支援小兵與召喚物。每模板模型、材質、方向與世界高度使用相同有界設定，既有Blueprint authored mesh保留優先。

當前實作：world_units.lua 固定 single_lane_tower=32768／single_lane_base=32769，分別240／260遊戲單位高。生成19檔／22 Lua inputs，content hash 69b32894669cb164；fresh codegen52測試全過。NativeStaticVisualMesh／transform／具名材質在 AOmUnitActor 共用處理，authored BodyMesh優先；壞材質槽／移除metadata退回診斷方塊，world scale與pool共用既有高度正規化。正式 native fixture 已加入兩生成類別與同一bridge三單位frame，但仍待本次build與D3D12執行。

world-model-import-v1 八個負例零寫入成功，首次正式匯入失敗且零package儲存；原作者塔／主堡分別含4／8個零面積面。E900記錄通用略過政策與source/render面數對帳，原來源SHA不變。修復後待fresh v2，不能用負例成功宣稱模型匯入成功。

2026-10-12進度：world-model-import-build-v7 正式refresh成功，ShaderCompileWorker2.16秒／Editor12.42秒後再通過普通NoEngineChanges／NoUBA／nocache7.89秒與stage SHA。原v3／v6真EngineChange、v4泛型指標比較與v5 ImageCore依賴失敗完整保留（E901–E904），未手改Engine或cache。

world-model-import-v2 真成功：八個負例零寫入；第一次4模型／12package保存，第二次4verified／0imported／0saved，全部package SHA與mtime不變。原始來源與授權SHA再驗相同；塔88頂點／348可渲染面＋4退化面，主堡134／508＋8，樹117／230＋0，岩石42／80＋0。材質graph／compiled shader與atlas source像素、SRGB／clamp sampler已驗。source_vertices／source_triangles與正式manifest對帳，原始頂點皆保留。

當前52 codegen＋17 Lua契約完整logs與來源指紋在world-model-contracts-v2；world-identity-rust-v3真722 core＋39 bridge IPC／四all-target checks全過，另1項依賴真對局HUD capture的既有ignored明示保留。

world-model-native-v1 真25 Success／0 warning，但三張原圖檢查拒絕接受：原模型多數黑面，正式橋接截圖沒有塔／主堡。E905 記錄 UE 左手座標反向 cross 與 OBJ 軸交換造成的雙重反轉；來源不改，改 importer 面序並對既有管理資產做嚴格預檢後版本升級。增加逐面法線一致性與橋接位置／可見性證據，送出 EditorPreview end-of-frame 更新後再捕捉。下一步 build-v8 ordinary nocache、import-v3 升級、contracts-v3 與 native-v2；舊 native PASS 不等於視覺通過。

當前模型範圍已接受：world-models-accepted-v1。build-v9 ordinary nocache12.97秒；import-v3八負例零寫入、4既有模型升級／12package保存、repeat4verified／零保存且SHA／mtime不變。contracts-v4真52 codegen／17 Lua與12來源指紋。native-v2保留25 Success／0warning，但其bridge畫面仍拒絕；E906修fixture等待正式資產編譯並斷言scene proxy／藍屋頂像素後，fresh native-v3受影響3項Success／0warning、三張當次原圖目視正常。只fixture一檔變動，另22項以來源指紋明示沿用，不偽稱fresh25。Rust722 core／39 IPC／4 checks與1 capture-dependent ignored明示沿用。完整世界／對局／44父項未完成，12/44保持；接公開樹ISM通用呈現。
