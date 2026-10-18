## E1299：忽略AI暫存目錄後仍對其追蹤檔使用一般git add（2026-10-18）

交接已以`/.ai-collab/`排除大量本機cache／草稿，最後一次git add同時指定其已追蹤review／state時回報ignored path。先前`git add -u`已暫存這兩個追蹤檔，沒有內容遺失；改為只對這兩個明確追蹤檔使用`git add -u --`，其他正式檔案分開add，不以`-f .ai-collab`把整批cache帶入提交。

## E1298：換機暫存檢查把原始CRLF當作尾端空白（2026-10-18）

為保護匯入來源／授權／receipt與producer的原始SHA，新增Git `-text`屬性；第一次staged diff-check因原始CRLF把OmOriginalAudioImportCommandlet.cpp及授權檔整行報成trailing whitespace。已保留原始byte，對producer明示`whitespace=cr-at-eol`並將原始素材與一般程式格式檢查分開；不改第三方授權、不改驗收來源來消除顯示噪音。另有OwnedViewportIpcTests與OmCommandBarWidget各一個既有行尾空格，原樣保留並在handoff說明，沒有冒稱全量diff-check零診斷。接續以暫存區重新取出檔案核對素材／producer SHA，避免只驗工作目錄而漏掉checkout轉換。

## E1297：文件同步helper對既有文件誤傳overwrite=false（2026-10-18）

錄影所有功能驗收成功後，Lua文件同步helper第一個write被path.write的既有檔案保護拒絕；尚未寫入任何文件。已讀固定工具API確認第三參數為overwrite，僅對先讀取的明確MD目標改為true，保留歷史段落；新證據仍維持禁止覆寫。不是產品或錄影失敗，也不修改工具的保護機制。

## E1296：讀取已知Widget時誤用OmEditor前綴（2026-10-18）

錄影完成後盤點下一範圍，已知`OmCommandBarWidget.cpp`位於OmRuntime/Private，後續行號讀取卻誤寫OmEditor/Private，檔案不存在而失敗；沒有來源變更。後續延續搜尋結果的完整絕對路徑，不手動重組module前綴；讀取失敗不作任何功能判斷。

## E1295：專項報告旁的log檔名未經列檔確認（2026-10-18）

讀取兩項專項summary時順帶猜測`native.log`，該檔不存在；沒有檔案變更，正式summary已確認兩Success／零warnings。後續只讀summary列出的路徑，或先對精確證據目錄列檔，不用猜測檔名再以SilentlyContinue掩蓋失敗。

## E1294：Slate TakeScreenshot的輸出尺寸是FIntVector（2026-10-18）

修復接受：ordinary build-v2入口20秒、focused兩Success／零warning、完整197 native／458事件／69 Blueprint／五PCM及4811張錄影核對成功；1197來源與資產mtime一致。v1 C2665原失敗保留。

owned-video-build-v1 ordinary真編譯C2665失敗，因新錄影直接傳入FIntPoint；公開API要求FIntVector。程序全部退出、1197輸入與資產mtime核對後改用實際輸出型別，再明確取XY作AVI尺寸。保留v1建置證據；不修改Engine或停用型別檢查。

## E1293：對單行大型JSON使用Get-Content -TotalCount（2026-10-18）

盤點既有accepted receipt時以行數截斷讀取，JSON為單行而輸出47k tokens遭截斷。未改任何檔案。後續必須ConvertFrom-Json或Lua decode後只輸出所需欄位；不能以行數控制minified JSON大小。

## E1292：複雜rg正則的雙引號被PowerShell/native引數解析拆開（2026-10-18）

查找WriteReport時把含`TEXT("success")`的多分支正則放入shell字串，部分內容被當作路徑，回傳os error3；讀取失敗，沒有檔案變更。改用單純symbol搜尋與已確認行號讀取。跨shell的查詢優先簡單字面symbol；需要多模式時用獨立`-e`且避免內嵌引號。

## E1291：再次把PowerShell萬用字元當作rg檔案參數（2026-10-18）

錄影盤點時執行`rg ... scripts/*owned*`，Windows回傳os error123，未改任何來源或資產。已改用`rg --files scripts -g "*owned*"`取得真實路徑再讀取。E1287已有相同教訓；後續檔案篩選必須放在rg的`-g`，不能在PowerShell引數內假設會展開。

## E1290：新WorldBridge音訊整合fixture缺少WorldContext，DestroyActor發出warning（2026-10-18）

修復接受：正確註冊／回收WorldContext後public-voice-native-v2六Success／零warning，最後full197 native／458操作／69 Blueprint／五PCM與清理成功；1193輸入及mtime一致。原v1一warning保留。

public-voice-native-v1真六項成功，但AudioResourceScope新增真WorldBridge消費路徑後，最後Relay->Destroy在未註冊context的獨立World發出LogSpawn warning。不是產品播報失敗；實際start／result／結果重複／epoch斷言全成功，五PCM亦成功。仍保留一warning與原圖，待audio consumer退出、1193輸入核對後註冊／回收正確WorldContext，再fresh專項與完整接受；不以關閉log或預期warning掩蓋fixture生命期缺口。

## E1289：source snapshot尚在執行時就啟動下一次普通建置（2026-10-18）

public-voice-source-v6 capture第一次wait仍回傳session，卻接著啟動build-v4；兩者為唯讀來源擷取與正式建置，期間未修改production，最後1193輸入與所有保護mtime核對一致。保留此流程錯誤：不得把wait回傳當terminal，後續依賴必須檢查exit_code／session並真正等完。此次不冒稱snapshot在建置前已全部退出；source最終完整核對及後續fresh測試作為接受依據。

## E1288：UE SoundFileIO/libsndfile對實際Ogg的讀取發生Array越界（2026-10-18）

修復接受：refresh-v2使用正式Vorbis解碼、四SoundWave真匯入；獨立repeat兩次零save，原Ogg／derived WAV／package SHA與mtime一致。ordinary build-v5與fresh五PCM／197 native／完整操作已成功，bootstrap marker由正式流程退休；原SoundFileIO crash保留。

唯讀檢查1137行確認該callback以GetAllocatedSize而非TArray邏輯Num決定讀取上限，隨後用operator[]索引；實際assert發生於此。未擷取當下Index值，故不宣稱已定位確切越界位移或所有觸發條件；初次EOF推斷以本段更正。仍採正式Vorbis解碼API與canonical WAV來源，不改引擎或添加資料padding。

public-voice-refresh-v1的正式refresh與普通C++皆編譯成功，但第一個語音source前置解碼在SoundFileIOManagerImpl.cpp:1137 ReadBytes經libsndfile回呼發生Array assert；callstack指向ReadVoicePcm→ConvertAudioToWav。尚未建立任何語音package。完整failed engine log保留於voice-audio-import/ensure-1792316230-table0000000000d926c0；consumer退出、1184來源／資產mtime一致後修正。改用UE公開VorbisAudioDecoder有界解碼並生成canonical PCM16 WAV，再沿用既有安全WAV SoundFactory匯入；原始Ogg／derived WAV／SoundWave分別SHA，重複解碼必須byte-identical，既有WAV與package異動均拒絕。這是正式Vorbis來源匯入路徑，不加padding／EOF特例、不改Engine、不使用shell轉檔fallback。保留bootstrap checkpoint，修復後由正式refresh完成並退休。

## E1287：Voice後端之外，設定頁仍有舊的強制disabled與過時標籤（2026-10-18）

修復接受：真Slate Voice滑桿啟用、鍵盤調整為99%、獨立保存與重開；native-v2六Success零warning，當次音訊設定原圖已直接檢視。來源授權留provenance，玩家頁只顯示英文播報與可用性。

來源盤點發現ControlSettingsWidget除舊標籤「尚無原創素材」外，IsEnabled明確排除Voice；既有ControlRuntimePersistence更固定斷言disabled。於第一批refresh consumer退出並核對來源後，改依實際分類可用性、如實標示CC0英文播報，加入真Slate鍵盤調整／獨立保存與重開值驗證。先前查詢也誤把OmControl*放進rg檔案參數而非-g glob，唯讀失敗後改目錄加-g；無產品mutation。不可只讓音訊後端測試通過便宣稱UI滑桿已可用。

## E1286：語音header變更後普通NoEngineChanges被正式NetCore圖擋下（2026-10-18）

public-voice-build-v2在UHT後FailedDueToEngineChange，要求更新NetCore及平台manifest，尚未編譯本批C++；UBT130.89秒，helper總137秒。原log與失敗保留；程序退出、1184輸入與保護mtime一致後才做必要來源審查修正，再走正式refresh及普通gate，不改Engine／manifest或手刪checkpoint。同批唯讀盤點又猜不存在的OmMobaHudModel.cpp，改以已確認bridge driver的phase<=2／winner<=2驗證契約，不再猜其他模型檔名。

## E1285：建置helper在supervisor重入前建立fresh輸出目錄（2026-10-18）

public-voice-build-v1在任何建置consumer啟動前拒絕：helper先mkdir，再supervise_workflow重啟同腳本，子程序看到目錄已存在。保留原空輸出目錄及錯誤，將supervisor放到所有目錄mutation之前，以fresh build-v2重跑。未動production來源或繞過fresh檢查。

## E1284：語音provenance把來源物件寫進URL欄位（2026-10-18）

準備素材時將候選report的source整個物件當archive_url；盤點發現為{page,url}，未啟動consumer。保留原provenance，欄位正規化為實際source.url，SHA與四原始Ogg不變。匯入metadata需要固定純字串，不能依名稱猜report欄位型別。

## E1283：語音準備helper多餘替換與文字模式造成兩次前置拒絕（2026-10-18）

voice-integration-v1在最後多餘original-audio替換斷言失敗，四原始Ogg與provenance已建立但未產生正式匯入腳本。v2續跑以text模式讀含CRLF的license再和原bytes比較，Lua文字模式略過CR導致拒絕。原兩目錄保留，沒有聲稱匯入或播放成功；改binary license比對、既有素材只驗SHA不覆寫，fresh v3完成。此錯誤僅在準備helper，不放寬正式匯入驗證。

## E1282：唯讀盤點又猜測ABI與建置helper路徑（2026-10-18）

查詢音訊後續流程時使用不存在的rust/omfue-bridge/include/om_bridge.h及map-biome-build.lua，兩次唯讀查詢失敗；無來源變動。改從已定位的bridge/src與rg --files .ai-collab清單查實際入口；不再猜生成header或helper名稱。大範圍omfue --no-ignore查詢也過慢，後續限正式source子目錄。

## E1281：唯讀音訊盤點再次猜測WorldBridge檔案所在module（2026-10-18）

查詢audio接線時誤用OmRuntime/Private/OmWorldBridge.cpp，rg明確報不存在，未改檔案。再次執行rg --files後確認實際為OmGenerated/Private/OmWorldBridgeActor.cpp。後續先由檔案清單定位，避免沿用錯誤module／檔名；此失敗不是產品或編譯失敗。

## E1280：沿用ZIP盤點器的Windows保留字檢查過度比對prefix（2026-10-18）

準備語音候選盤點時來源審查發現舊模型download helper以`^CON`等prefix判斷Windows device名，會誤拒合法congratulations.ogg。新獨立盤點器只比對第一個點前的完整device basename，加入CON.txt／COM1.ogg拒絕、congratulations.ogg／COM10.wav接受的自檢；不改歷史模型腳本或其證據。此時尚未解壓候選，不能記素材已驗。

## E1279：生成獨立盤點腳本時漏掉片段間的換行（2026-10-18）

inspect-voice-sources.lua第一次執行在parse階段報endlocal，尚未下載或建立候選輸出。Lua long string會略過開頭換行，直接拼接擷取的helper函式與footer造成相鄰token。保留原syntax-rejected腳本，producer改明確加入`'\n'`並另產fresh v2再跑。另web reader無法讀application/zip，已從官方HTML取得真下載URL；這不代表資產下載失敗，binary須走原有Lua／NativeHost managed native tool流程，不把web錯誤當素材驗證結果。

## E1278：standalone portrait截圖沒有正式GameLayer，175% DPI下HUD右側裁切（2026-10-18）

修復已接受：hero-portrait-native-v2八項成功、零warning，改用正式viewport／LocalPlayer後七英雄與兩map共九張1280×720原圖皆完整fit；ordinary build-v2及final-v1完整197 native／458操作／69 Blueprint／三PCM成功，1148輸入與資產mtime一致。原v1裁切圖與成功測試結果皆保留，不追改為已接受。

hero-portrait-native-v1真8項成功，新兩圖確實載入；直接檢視apprentice／support／luminary的actual portrait原圖卻發現新fixture以裸SWindow承載完整UUserWidget，800×600 logical在目前OS 175%變成1400×1050，缺少正式player／game viewport層的排版脈絡，右側HUD被裁切。這些圖只證明可見portrait資源，不接受為完整HUD版面；原綠燈與原圖保留。兩map的既有實體viewport整合圖1280×720已見隊友2由問號變為新頭像，未出現相同裁切。等Rust consumer退出並核對1148來源／資產後，將新portrait fixture改用既有FOmEditorGameViewportFixture、真LocalPlayer／owned widget與明確實體client尺寸，再驗完整fit與PNG尺寸；不修改OS DPI、Engine或正式layout演算法。

## E1277：新測試曾直接引用未匯出的generated array，且舊registry斷言仍為43（2026-10-18）

接全英雄覆蓋時，來源審查在編譯前發現OmContentIds.h的ContentEntries／Count並未宣告DLL匯出，不能讓OmEditor直接import；同時HudAssetRegistryTests有兩個固定43斷言，新增兩圖後會失效。尚未嘗試失敗編譯，不冒稱有LNK log。改Rust generator加入OMGENERATED_API的immutable ArrayView accessor，正式generate後供兩測試共用；registry由generated heroes驗完整性，負例保存接受時的count，不以45取代另一個易過時常數。此輪generator Rust輸入確實變動，須fresh Rust驗證；不手改生成header／cpp，也不暴露可變array。

## E1276：portrait測試硬列五個ID，漏掉正式目錄的兩位英雄（2026-10-18）

實際map-biomes-native-v4原圖隊友2的training_apprentice仍顯示問號。對照正式OmContentIds七位非tombstone英雄與hud-assets-v1.json五個portrait，另缺training_support；HudAbilityIconAtlas只列五個已有圖ID，所以測試成功未覆蓋完整目錄。這是呈現缺口，不能靠未知fallback通過判定完整。接原創兩圖、authoring目錄／PNG完整性唯讀guard，以及從generated catalog逐一驗資源的native覆蓋；未知／來源退休仍須清除舊圖。計畫及獨立素材已備妥，前批完整驗收退出前不改正式輸入。

## E1275：native全通過但actual jungle cap出現黑塊／破面，拒絕畫面接受（2026-10-18）

map-biomes-native-v3真8 Success／0warning，直接檢視兩map當次game-viewport-biomes原圖仍發現四區野地黑塊與三角破面；不以測試綠燈接受畫面。source-v7於該程序退出後核對1024來源一致。圓柱end cap先前把600m直徑壓成2mm高，長短軸比例極端；僅修改幾何：深度維持直徑1%，向地底延伸，公開XY與可見top 0.6cm保持不變，新增實際instance bounds斷言。source-v8／ordinary build-v5成功15.49秒；fresh native-v4真8 Success／0warning，17張當次原圖直接檢視，黑塊與破面在兩map overview及combat均消失，來源1024一致，接full final-v1。此證據只支持幾何修正，不把底層原因冒稱已定位到Nanite或distance-field；全域光照及DefaultLit材質均未改，v3綠燈報告與拒絕原圖保留。

## E1274：鏡頭公開bounds區域變數觸發C4701（2026-10-18）

refresh-v3編譯OmWorldBridgeActor時對PublicMinimum／PublicMaximum各報C4701。核對TryGetPublicCameraBounds在true時必定寫入兩值，且使用點受bPublicBounds分支保護，未發現可到達的未初始化讀取；但為避免依賴跨函式輸出推導，等整批退出及source-v6核對1024來源後，將兩個區域明確初始化為ZeroVector，保留原有成功／失敗條件。source-v7凍結後普通map-biomes-build-v4成功10.84秒，該C4701已消失，1024來源一致；原refresh警告保留，不把整個toolchain說成零warning。

## E1273：fixture team入口標頭更新後普通建置再次EngineChange（2026-10-18）

source-v6的map-biomes-build-v3 NoEngineChanges／NoUBA／nocache實際失敗FailedDueToEngineChange，112.30秒；UHT已完成，但尚未進入C++編譯，不能宣稱修正已編譯成功。保留原false與UBT log，等程序退出並驗1024來源一致後，使用正式refresh-v3，再回普通建置。不改引擎來源、cache或manifest，不把先前refresh成功當作本次header後的普通建置成功。

## E1272：查詢bridge subsystem前再次猜測檔名（2026-10-18）

定位整合fixture時，先猜OmBridgeSubsystem.h而讀取失敗；實際名稱是OmRuntimeBridgeSubsystem.h。重新依rg --files盤點才讀取真檔。不得把型別縮寫當檔名；即使同批已有一個已知檔，也要先查出每個尚未看過的路徑，遵守E1259。

## E1271：合成世界沒有active team，整合地表被正式fog守衛拒絕（2026-10-18）

map-biomes-native-v2真8項執行：7 Success、WorldBridgePresentationScale 1 Fail，預期5個公開surface卻0。獨立MapSurfaceModel與MapSurfacePresentation成功；fixture並未啟動runtime，GetActiveLocalTeamId為0，故MinimapState.Fog不可用，新的surface入口正確拒絕。不能從frame的team欄位猜隊伍或放寬正式admission；以test-only入口明示expected team，呼叫同一個ProcessFrameForTeam流程，正式入口仍只取started runtime的固定team。fixture新增未知／外隊拒絕，並改用Bridge真正建立的ground，不另生手動地表。MapSurfacePresentation另有24個缺world context的child-actor回收warning，已補context並保留至destroy後；仍待fresh native重驗。另兩warning是既有HTTP generate_204 timeout；原報告與false結果完整保留。

## E1270：新增唯讀素材gate後漏補既有stage fixture依賴（2026-10-18）

實跑ue_bridge_stage_test於自訂require的第32行失敗：正式verify_stage_consistency已新增ue_map_biome_materials.require_ready，但隔離測試仍只提供舊兩個模組。frontend preflight五組與local acceptance contract66項均通過。這不是放寬產品檢查的理由；等當前消費者退出並核對來源後，補注入唯讀material readiness fixture及缺失時禁止任何process的負例，實際8案例全部成功，亦驗bridge已失敗時不可繼續material check。正式素材工具另外11項自足SHA／路徑／來源檢查已通過。

## E1269：scoped native runner仍拒絕正式schema2 fixture（2026-10-18）

map-biomes-native-v1在啟動Unreal之前於helper第97行失敗：exporter與native consumer已更新schema2，scoped runner卻仍限定schema1。原success=false紀錄保留，未執行原生測試，不算產品功能失敗或成功。將該helper明確限定目前schema2，不放寬為任意版本；使用fresh v2重跑。版本演進須盤點exporter、consumer及所有啟動前guard，不能只改讀取端。

## E1268：通用PCH盤點深入外掛私有SDK／Web而未開始讀PCH（2026-10-18）

唯讀warm-v1持續數分鐘卻只讀約484KB，仍在插件目錄盤點；D磁碟當時讀取約2MB/s且queue壓力高。以creation_token核對後只停止自己的reader21724，保留success=false、cancelled=true紀錄；未停止compiler或其他程序。改用當前三模組140個response精準盤點，唯一2,574,581,760-byte shared PCH在2141ms預讀，編譯隨後推進到48／71以上。正式ue_shared_pch_preflight改依.uplugin邊界只進Intermediate/Build與巢狀Plugins，跳過私有SDK／Web／node_modules；新增真實filesystem fixture驗證避免再次遍歷無關樹，仍維持8GB budget及唯讀size／mtime守衛。修後通用warm-v3實際1609ms讀完同一PCH，17項測試成功；預讀仍不宣稱完整性檢查。

## E1267：UE5.8 GetStaticMesh回傳TObjectPtr與C字串型別未明確轉換（2026-10-18）

map-biomes-refresh-v1實際C++失敗（879.40秒，含PCH冷讀等待）：OmMapSurfaceActor以auto*接GetStaticMesh的TObjectPtr造成C3535／C2440及後續型別連鎖錯誤；MapSurfaceModelTests把uint8*直接指定給ABI const char*造成C2440。等全批退出並驗1022來源一致後，改成明確UStaticMesh*與reinterpret_cast<const char*>；不更改API或放寬驗證，原false與log保留。因refresh失敗仍有checkpoint，接著須完成refresh-v2及普通NoEngineChanges gate，再匯入與實測。

## E1266：refresh編譯器在大型PCH冷讀階段長時間低CPU（2026-10-18）

map-biomes-refresh-v1三個cl.exe於14:51:33啟動，14:57仍無編譯完成輸出，CPU約0.7–1秒、working set由約370MB緩增至610–668MB，各commit約2.5GB；機器尚有約14GB可用physical memory。這是冷讀／mapped PCH等待的特徵觀察，不宣稱已證明唯一原因。使用專案既有ue_shared_pch_preflight唯讀預取入口，最多8GB，先後核對size／mtime且不修改PCH／Engine／manifest／cache，不終止來源未明程序；需觀察正式編譯實際完成才接受，預取不等於完整性檢查或功能測試。

## E1265：素材workflow審查發現不存在的path.basename及成功檔寫入順序（2026-10-18）

執行材質匯入前盤點工具API，發現path.lua沒有basename。已改由lfs原始枚舉保留檔名，並將summary=true移到provenance成功寫入之後，避免子程序成功但外層失敗仍留整體成功標記。provenance改存相對package path／SHA，不把此機absolute path或mtime當跨機器輸入。此修正於匯入前完成，尚未產生材質或成功證據。

## E1264：河道／野區新UCLASS後普通建置再次遭EngineChange（2026-10-18）

map-biomes-build-v1普通NoEngineChanges／local／NoUBA／nocache真實在UHT後回傳FailedDueToEngineChange（172.82秒），列出Engine modules及NetCore將修改，尚無本批C++成功證據。1022維護來源在退出後完整驗證未變。保留原log／false，只依此實際結果走既有refresh入口，之後回普通gate；不手改Engine或manifest。首次材質／actual viewport結果尚待建置及匯入，不能以Rust197／export成功代替UE驗收。

## E1263：呈現接線審查發現整份frame拒絕路徑不應只清新layer（2026-10-18）

新map-surface接線最初在OmValidateFrameContract失敗分支直接清新裝飾，但其他scene仍維持最後合法frame，造成拒絕操作的部分狀態變動。編譯／執行前審查已移除這項副作用：整份frame被拒保留最後合法scene；合法frame內非法可選surface由共用decoder原子清空世界與小地圖layer，合法fog保留。同步改用明確UMaterialExpression*遍歷TObjectPtr expression集合，避免auto*型別推導依賴隱式轉換。此為來源審查修正，尚未宣稱UE執行已驗。

## E1262：ABI30變更後未先重生header就執行header一致性測試（2026-10-18）

map-biomes-rust-v1的template IDs實際56Success；bridge實際123Success／4Fail／1既有ignored，四失敗皆為生成header仍#define OM_ABI_VERSION 29，而Rust已30。新surface projection／lease／retirement測試本身成功，不能把整批當成功。以正式cbindgen.toml與cbindgen命令產生新header後依byte SHA同步到既有staged include，不手改生成header，再fresh重跑；原失敗log保留。

## E1261：Rust彙整進度檔誤用禁止覆寫模式（2026-10-18）

run-map-biome-rust.lua每步都以json.write(...,false)寫summary.json；第二步bridge完成後遭refusing to overwrite，尚未報告bridge的真實四失敗，也未跑core兩組。原目錄summary=false與兩份log保留。改成每步唯一step-N.json，全部完成後才寫一次summary.json；必須fresh目錄重跑，不能把原v1補寫為成功。

## E1260：同一apply_patch重複宣告更新同一檔案（2026-10-18）

ABI30斷言更新的patch誤放兩個Omfue bridge lib.rs Update File區塊，工具在驗證時拒絕multiple operations target，未執行該patch。移除多餘的無效區塊，維持同一檔案一次宣告、hunk按行序遞增後重送。後續組合patch先去重檔案標頭，不重複加入no-op hunk。

## E1259：從include名稱推測FrameContract實作檔再次失敗（2026-10-18）

同批亦曾猜測scripts/build_ue_bridge.lua而不存在。實際生成命令由已存在的omfue/build_bridge.bat查得；後續只接受rg --files或先前工具已返回的具體路徑，不以「應該有同名檔案」作為讀取依據。

河道／野區唯讀盤點曾猜測OmFrameContractTests.cpp與OmRuntime/Private/OmFrameContract.cpp，實際兩者都不存在。rg --files定位後確認契約為OmGenerated/Public/OmFrameContract.h，相關測試在OmEditorAutomationTests.cpp等已知檔案。前面提醒尚未落實；後續不得從include推測cpp或Tests檔名，必須先以rg --files取得存在路徑才讀取。未修改生成header，也沒有因此略過ABI或frame契約驗收。

## E1258：剩餘地圖項目唯讀盤點再次把星號當作rg檔案路徑（2026-10-18）

完整驗收等待期間，唯讀盤點傳入omoba-core/src/presentation*而回報os error123；立即改成rg --files -g定位實際runtime/presentation_cue.rs。這與E1252同類，先前約束未落實。後續工具呼叫不得把含星號的檔案路徑交給rg；通配只放在-g參數。未修改任何內容來源或正在驗收的程式，不影響本輪凍結。

## E1257：失敗refresh留下的初始化checkpoint尚未完成（2026-10-18）

已經正式refresh-v2完成，checkpoint由原入口正常退休；接著ordinary lighting-build-v5成功2.38秒、153來源一致，full入口已成功開始195項native。未手動刪除checkpoint或改驗收guard。

lighting-build-v4普通成功、native-v2六Success／零warning後，完整入口在建立驗收目錄前拒絕：finish pending source-engine initialization。先前refresh-v1編譯失敗留下正式bootstrap checkpoint；普通建置成功不會自行退休此checkpoint。依build_ue_moba／ue_engine_bootstrap的既有入口重新完成refresh及其普通gate，不手刪marker、不繞過驗收；之後再跑普通建置與完整驗收。唯讀追查曾猜測不存在的ue_source_build.lua，改用rg --files定位真實ue_engine_bootstrap.lua，仍須遵守先定位再讀。

## E1256：光照測試把來源重載的合法geometry重建當成光照寫入（2026-10-18）

修復後lighting-native-v2實際六Success／零warning，geometry零寫入與重載精確+1、light／exposure／shadow斷言均通過；153來源一致，15張當次原圖已直接檢視。完整回歸另由lighting-final-v1執行。

lighting-native-v1實際六項中五Success，WorldSurfacePresentation唯一失敗為geometry count預期1、實際2；所有新增光照切換斷言及實際renderer完成。原production對ResetPublicView後首次有效來源明確要求重新套用地表，測試卻把重載後計數與重載前比較。保留false／原圖與153來源驗證；將零重建斷言放在光照生命週期結束、來源重載之前，重載後另驗精確+1，不改production幾何規則。同步補真light component visibility／registration、曝光與key-only動態陰影斷言，需fresh重建與測試。

## E1255：Actor成員函式區域Owner名稱觸發C4458（2026-10-18）

lighting-refresh-v1實際C++編譯回報OmWorldPresentationActor.cpp的區域Owner遮蔽AActor::Owner，依UE規則為error，原建置false與log保留。等所有編譯程序退出並驗153來源後，改名LightOwner；同步將黑光判斷限制於RGB，不讓無照明意義的alpha影響決策。仍需fresh普通建置與測試，不把refresh失敗當成功。其他既有plugin dependency與bridge C4701警告照實保留，未宣稱建置零warning。

## E1254：光照header更新後普通建置遭EngineChange gate拒絕（2026-10-18）

lighting-build-v1普通local／NoUBA／nocache／NoEngineChanges在UHT處理一份生成檔後真實回傳FailedDueToEngineChange；UBT列出既有Engine modules將被修改，尚未C++編譯，不是功能通過。153來源於失敗退出後完全一致，build.log／UBT-Log.txt保留。只因這次實際gate結果執行既有--refresh-engine入口，之後仍須再跑普通NoEngineChanges建置；不手改Engine／manifest／cache。

## E1253：補光判斷永久快取第一次場景狀態（2026-10-18）

修復已接受：`playable-mvp-lighting-accepted-v1`，普通build-v5成功2.38秒，focused6項零warning；fresh195 native（12 warning保留）、full458／222 wire／157 stage、13筆游標觀察、69 Blueprint零compiler／load診斷與三PCM成功。153來源一致、69資產SHA／mtime不變、全部清理及15當次原圖直接檢視完成。E1254–E1258與所有失敗結果保留，未修改舊false，沒有新權威對局或ACK。

盤點發現InitializeLighting以bLightingInitialized一次性略過後續判斷，ResetPublicView也未清除此旗標；動態燈光／重新載入會留下過期補光與曝光決策，且只檢查IsVisible與Intensity，未排除hidden owner／未註冊／不影響世界光源。改為有界頻率的同世界光源仲裁，保留原風格與曝光優先序；需原生生命週期、實際renderer及完整回歸成功才接受。

## E1252：PowerShell傳遞路徑萬用字元給rg導致唯讀搜尋失敗（2026-10-18）

搜尋Engine的LightComponent*.h時，rg未替檔案路徑展開星號而回報os error123。已先rg --files -g定位真實header後再讀；不以shell萬用字元推測檔名。另首次規格／程式合併讀取仍因長輸出截斷，後續必須分檔或限制具體範圍。

## E1251：文件差異檢查未處理既有CRLF與巨大dirty diff（2026-10-18）

收尾git diff --check直接掃既有大量未提交文件，CRLF被當成行尾空白，產生過量輸出；同一PowerShell呼叫後接其他成功指令，外層exit0也不能代表diff檢查成功。改用明確cr-at-eol設定、先保存LASTEXITCODE並限量顯示。重驗回傳exit2，30行診斷位於原有歷史段落／檔尾，不宣稱整份dirty diff乾淨，也不為本輪功能改寫既有歷史或換全檔行尾。後續須以本輪修改與既有工作樹分開審查，所有非零結果先處理再進行相依步驟。

## E1250：驗收彙整以文字讀取器檢查PNG簽章（2026-10-18）

彙整修復後已通過，最終接受為`playable-mvp-portrait-accepted-v1`；PNG原始尺寸與SHA檢查保留，沒有跳過斷言。

portrait-final-v1實際完整驗收成功後，accept-portrait-projection.lua在PNG簽章斷言失敗。彙整工具誤用會處理文字換行的path.read讀取二進位PNG；改用io.open的rb模式，維持原始八位元組簽章、尺寸及SHA檢查，不修改測試結果或圖片。後續二進位驗證一律使用二進位讀取，不用文字工具。此失敗不代表195 native或458操作失敗，原失敗輸出保留。

## E1249：RTS相機繼承MaintainX環境設定時極窄視窗會越過地平線（2026-10-18）

修復已接受：`.ai-collab/playable-mvp-portrait-accepted-v1/summary.json`，普通建置13.38秒、focused6項零warning／24組實際投影、fresh195 native（12 warning保留）／458操作／69 Blueprint／三PCM全成功；153來源一致、69資產SHA／mtime不變，兩map十二當次原圖已直接檢視。下面「須fresh驗證」為修復前狀態，現在已有上述證據；原失敗不改寫。

portrait-repro-v1預設400×1000實際成功，未把推測當錯誤。擴充至LocalPlayer三種正式axis後，repro-v2在MaintainX（axis=1）、400×1000、zoom7000真失敗；視點Z6062.178、pitch-60、玩家已註冊且local，地表合法射線守衛拒絕朝上角落，153來源一致。修原生RTS camera明確override為MaintainYFOV，沿用Camera的16:9參考aspect與既有FOV／縮放，不改Engine或ULocalPlayer全域設定、不關覆蓋守衛。測試加入三axis×四比例×兩有效縮放，直接驗實際projection矩陣垂直尺度不依環境；兩map portrait原圖改真400×1000。原default成功、repro-v2 false與log全保留，須fresh驗證才接受。

## E1248：整合角落原圖雖通過，但遠近縮放的有效arm仍相同（2026-10-18）

camera-coverage-native-v2真六項Success／0warning、三比例四角與退休通過；兩map角落原圖已產生。但讀原log發現far／near的camera高度皆77942.286cm，fixture改DesiredZoom後只做Tick(0)，不能把這批圖當遠近已驗。保留原圖並拒絕該縮放證據。補明確正常Pawn／spring-arm收斂與實際arm長、相機到focus距離、far大於normal斷言，不改production／不關平滑；新原生也核對有效arm。重新建置與拍攝後才能接受。

## E1247：新viewport測試未先推進world就讀camera projection（2026-10-18）

camera-coverage-native-v1六項中CameraGroundCoverage失敗，其餘原測項完成；首次UpdateCameraCoverage拒絕投影，原log保留，153來源一致。新fixture只手動tick pawn／spring arm與UpdateCamera(0)，沒有推進World，可能仍讀到camera manager同時間戳的初始cache；先補真World tick，再讀實際projection，另增加失敗時視點／角度／尺寸／註冊診斷，不放寬四角覆蓋或owner檢查。這是待重跑驗證的原因判斷，不將首次false改成功。

## E1246：新增視野測試誤讀protected相機欄位（2026-10-18）

camera-coverage-build-v1 C2248：新整合fixture讀MaxZoomDistanceCm，新原生測試寫ZoomSmoothing，兩者為protected。等完整建置退出並驗153 SHA後，改走SetZoomDistance有限最大值由正式setter夾住、Pawn Tick讓平滑正常收斂；不擴大欄位權限、不繞過相機。原log／false保留。同步審查補正無entity但公開grid仍有效的分支：只清英雄focus，仍維持公開地圖bounds與裝飾coverage；整個來源為null才撤銷coverage controller，不把單位缺席當地圖退休。

## E1245：唯讀盤點再次猜測bridge header路徑（2026-10-18）

盤點相機地圖邊界時曾對OmRuntime/Public/om_bridge.h執行rg而報不存在；已用rg --files找到ThirdParty/OmBridge/include/om_bridge.h，沒有修改生成header。後續必須先定位檔案再讀，不能以include名稱推測模組位置。長段落盤點也曾截斷，後續改為行號定位後小段讀取，不把未讀內容當作尚未實作。

## E1244：完整配方購買的pending不得被後段無receipt投影當作不存在（2026-10-18）

final-v5真194 native全成功（13 warning保留）、三PCM成功、69 Blueprint零errors／warnings／load failures；full IPC完成305case後shop-txn-ready期待空字串，實際前段shop-recipe-buy保留Buy waiting。同owner／view／connection、沒有receipt時正式保留pending正確，tail無前購買故先前通過。修測試為stage前擷取既有狀態與action sequence，檢查合法「未提交空字串／已有購買Buy waiting」，stage後必須逐字與sequence不變；不清pending、不捏造結算、不放寬後續新buy／sell／receipt／scope清除與零額外wire。editor／publisher／profile退出、153 SHA相同後才改唯一owned測試來源，fresh full必須重跑；194與69僅可在排除owned測試且其餘來源、PCM與資產SHA／mtime皆一致的明示證明後沿用，原final-v5 overall=false不改。

## E1243：音效驗收程序在測試開始前遭D3D12裝置移除（2026-10-18）

final-v4 native-008 audio-output另有獨立失敗，不能只看一般native結果。實際native-engine.log在frame2報GPU crash／D3D Device Removed，exit3；Local Budget5198MB／Used1847MB，未產生PCM或通過結果。保留log與false，不將此當音效邏輯已驗。後續用同一維護入口、相同D3D12與AudioMixer真輸出重新跑fresh ID；不切NullRHI、不代用來源WAV、不修改Engine或降低功能。先前工具輸出截斷，必須逐項讀native_batches的success／error，不能由外層仍在繼續就推定該批成功。

## E1242：商店交易舊回歸仍要求已退休的背景錢包提示可見（2026-10-18）

final-v4完整native僅ShopTransactionStatus失敗；193個常規測試185 clean／7 warning-success／1 fail，獨立audio另列。兩個assert均源於E1238新scope：shop開啟時背景錢包提示須失效；真幾何font／painted／window／band／shop全1，僅phrase=0。修測試：shop開啟嚴格驗tooltip存在但IsEmpty且讀取空字串；關閉仍比正式完整tooltip；未知商店／完整背包文字從正式data accessor核對，不捏造空欄位。gold短字、真交易狀態、前景提示、排版邊界全保留。full IPC尚在前段時主動正常關閉publisher，listener釋放、editor／profile清理成功、153來源一致後才修改；final-v4 false與原log保留，fresh完整驗收重新跑。

## E1241：英雄層設定初始化仍須等待公開的英雄輸入身分（2026-10-18）

tail-v4在accessible-ready正式拒絕Hero editor context：baseline沒有InputSchemeView.HeroContentId，不能將整個AccessibleConfigure提前。publisher／editor／profile已退休，153 SHA相同。改為投影前僅重設Global（不需要英雄），投影就緒後才執行既有Hero／Mode與游標設定；不修改正式身分檢查。保留v4 false與E1240原wire失敗，重新凍結來源後建置與驗收。

## E1240：游標情境在投影生效後才重設上一情境的戰鬥偏好（2026-10-18）

tail-v3已通過前150個case；accessible-ready切到input_point時，正式SyncCombatPreferences依前段保留的游標攻擊／關閉自動攻擊，合法送出一筆CombatPreferences，違反此情境零額外wire契約。baseline無combat投影，input_point會公開預設偏好；將AccessibleConfigure移到WriteControl前，在無投影時先建立本情境完整profile，不改正式同步邏輯、不吞wire、不放寬oracle。原失敗與153 SHA驗證保留，待fresh tail/full。另本輪首次唯讀查找誤猜Source路徑，已以rg --files定位Plugins下真檔；再次提醒先定位後讀取。

## E1239：新提示scope回歸誤以為 FindTag 是檔級helper（2026-10-18）

final-build-v7 OmActionOwnerTests.cpp:5709 C3861 FindTag未宣告，其他Tip／TestTrue錯誤為連鎖。既有FindTag其實為各RunTest內的區域lambda，不能跨測試借用。等建置完全結束、153 SHA驗證後補新測試自己的widget tree查找，不修改production或移除回歸。之後引用測試helper前必須確認宣告scope，不能從同檔呼叫推斷全檔可用。

## E1238：開啟商店後舊戰鬥長提示仍可擋住購買按鈕（2026-10-18）

tail-v2通過ward後，shop-txn-buy真hit leaf落在SOmHudToolTipText，不是OmShopBuy.4；舊ward長提示仍具有效權限。正式MakePlayerToolTip增加明確shop scope：預設戰鬥提示只在shop關閉時可呈現，商店交易提示明確屬shop scope；既有Stats guard／退休權限保留。舊提示即失效、host不再佔hit-test，不用測試強關提示或把錯誤點擊放寬。新增ShopTooltipScope native驗open／close／retire。這是production改動，前193／69的沿用證明現在僅保留歷史，必須重新native與Blueprint驗收。tail-v2 editor／publisher已退出，153來源一致才修。

## E1237：新tail分段缺少由前段配方流程初始化的商品ID（2026-10-18）

tail-v1真10項ward與12stage完成（含正確短badge／完整tooltip／結果），進ShopTxn前因ShopBuyCatalog預設0而失敗。原full由ShopRecipe設定，但獨立tail跳過前段。改初次為0時從正式生成目錄FindItemMetadata(4)取得並驗moba_greatsword身分，仍保留後續catalog==4 fixture契約，不鬆動價格／wire／Receipt檢查。流程已退出、153來源一致後修；tail-v1 false保留，不冒稱164全通。

## E1236：單檔多hunk patch依行號逆序導致找不到已存在context（2026-10-18）

新增tail與tooltip讀回時，同一patch先改10387行再回9061行，apply_patch順序驗證找不到真存在的WardCardMatches，整份未落地。已依來源順序重新套用並讀回，不把失敗patch當已改；大型檔的hunk按原始行序排列，或拆成獨立patch。

## E1235：來源審查發現owned商店仍讀舊tooltip型別與金幣列長狀態（2026-10-18）

final-owned-v4執行中唯讀審查確認：正式shop tooltip已為SOmHudToolTip，fixture卻static_cast成SDeferredToolTipText；還比診斷英文tooltip、HitTestInvisible與舊gold列的Buy/Sell長句。為避免已知錯誤跑至不安全cast，使用正式renderer-fixture stop，listener_released=true，editor／publisher正常退休，153來源一致後修改；v4為主動中止的未完成測試，不宣稱通過。改IToolTip.GetContentWidget遍歷實際STextBlock，通用支援自訂與deferred提示；比玩家中文版tooltip及可hover文字，raw交易狀態另比既有診斷API，gold列嚴格保持真金幣。新增tail phase由ward-ready-status邊界開始，164case／102wire／54stage且不冒稱full，先集中驗後半流程，再fresh full458。沒有更動production或重設權威對局額度。

## E1234：full owned 眼位測試把短圖示讀數當作完整電荷／結果文字（2026-10-18）

final-v3的193 native全成功與69 Blueprint零compiler／load診斷、69 SHA／mtime不變；full owned已過模式／商店原294prefix，ward-ready-status卻要求OmWardStatus短badge等於完整「電荷1/2」與結果文字，實際短badge為1。正式GetWardStatusText與tooltip仍保留電荷容量／不可放置／結果，本來便有分層。修fixture：短badge精確比當前公開Charges（本組各stage皆保留電荷），完整欄位驗Charges/MaxCharges與Second結果字、tooltip存在、button／hotkey／name／可見性繼續嚴格驗。僅改owned測試，不改production縮回長字。193 native與69 Blueprint可依所有綁定來源一致且只變owned fixture的SHA證明明示沿用；full owned仍須fresh全458。publisher／editor／profile已清理、153來源一致才修改，原final-v3 false保留。

## E1233：空連線通知會覆蓋未取得模式時的玩家狀態（2026-10-18）

盤點final-v2 mode失敗後發現真正UI缺口：SetAuthoritativePresentation寫入中文等待／TD標籤，HudOwner稍後SetStatusText空ConnectionNotice又清掉。SetStatusText統一合成：非空真通知優先，空通知回到當前已公開模式的中文說明，正常MOBA仍留白，未設定權威模式的preview不造資料。新增ModeStatusComposition回歸驗未知模式、TD、錯誤優先／清除恢復、正常MOBA與非空技能通知。不是把Mode unavailable英文硬塞回玩家畫面；尚待build／native與full重驗。

## E1232：唯讀盤點再次猜測不存在的 ModeChromeTests 檔名（2026-10-18）

rg對OmModeChromeTests.cpp報不存在，實際相關測試分散在既有HUD／owner測試。未改來源，已改rg --files定位。不得由測項名稱猜檔名；後續引用以已知真檔路徑為準。

## E1231：完整模式chrome測試把collapsed祖先下的子幾何算成可見（2026-10-18）

final-v2真192 native全成功（12 warning保留）、69 Blueprint compiler errors／warnings／load failures均0，69 SHA／mtime不變；full owned到mode-td-controls失敗item=1。正式ItemWrap已collapsed，但CollectArrangedTags用EVisibility::All跨過不可見祖先，讀到保留子幾何，造成假可見。改為祖先不可見立即停止，ArrangeChildren只收可見項，仍嚴格檢查geometry／數量，不能放寬Spec。相同來源盤點將未知模式狀態斷言改驗中文，並另修真正空通知覆蓋缺口E1233。publisher／editor已退出、153來源一致後才改。原final-v2 success=false保留，不將69 Blueprint成功冒充整輪成功。

## E1230：full owned 的舊物品狀態與配方英文搜尋未跟隨正式玩家文字（2026-10-18）

final-v1 full native真192項186成功（8項warning-success）／6失敗，0notRun；獨立音效3PCM成功。full owned在第185項item-ready失敗，原圖／欄位顯示守護符可用、鐵劍被動等正確中文，舊斷言仍比ready/passive。另來源盤點同一fixture的Empty與sword搜尋也過期。publisher／editor／profile清理成功，150來源一致後才修改：中文可用／被動與空槽破折號，配方部分搜尋直接來自正式Meta.Name；價格／配方／命令／欄位／source／scope仍嚴格驗。沒有改production來滿足舊英文；新增三個漏列的舊native測試來源到freeze closure。原full失敗保留，Blueprint依正式gate因native失敗未啟動，不冒稱已驗。

## E1229：對單行大型 JSON 使用 TotalCount 仍導致輸出截斷（2026-10-18）

唯讀盤點final-v1時Get-Content native-combined.json -TotalCount 1印出整個單行55k-token JSON，工具截斷。行數限制不等於JSON大小限制；後續先ConvertFrom-Json／Lua解析，只輸出指定計數與失敗名稱，不再直接輸出完整combined report。原artifact未改。

## E1228：獨立配方測試用舊英文 sword 搜尋正式中文物品（2026-10-18）

playable-mvp-final-v1 native-006 的 ShopRecipePresentation 在真ClickTag失敗；測試先用「 sword 」過濾，正式名稱已中文化，故目標列不在畫面。前批ResponsiveHud的中文流程有驗而此獨立測試漏改。待所有消費者結束，改從GreatMeta正式名稱取部分字串，保留外圍空白trim、分類／返回／resize／epoch狀態保存與真點擊檢查，不為測試另造英文玩家名稱。

## E1227：兩項 card 回歸仍期待舊英文狀態與長冷卻比值（2026-10-18）

final-v1 native-005 OwnedItemCards 期待ready／passive，現可見狀態為可用／被動；native-006 SummonerOwnerCards的短icon冷卻期待4/10而新HUD顯示剩餘4。須驗目前玩家短文字並保留詳細欄位的總時間，不改回舊英文／冗長主介面。這是完整回歸捕捉的測試落後，原fail保留，待full流程結束才修。

## E1226：小地圖獨立 routed fixture 沒有 platform user 身分（2026-10-18）

final-v1 native-004 MinimapCameraRoutedInput 真camera／capture四斷言失敗；其裸NewObject<ULocalPlayer>未初始化PlatformUserId，正式新confirm query因無法qualify而拒絕。相同完整批次的真game viewport MinimapPreferences／focused owned皆通過。依SDK正式SetPlatformUserId初始化fixture並以實際Slate user建pointer events，不能放寬production的foreign-user guard。待整輪結束與SHA驗證後才修改與實驗；此時尚不宣稱修好。

## E1225：完整原生音效回歸仍寫死舊 profile version 8（2026-10-18）

playable-mvp-final-v1 native-002 真 AudioProfile 失敗，Expected migrates to version eight=8，實際正式9；native-001的32項成功（8筆既有warning保留）。本輪繼續其餘native／owned／Blueprint以集中盤點；不在消費者存活時修改。待完整結束與來源驗證後，migration目的版本應比較維護常數CurrentVersion，legacy輸入版本仍保留真7作回歸，不更動原失敗log。

## E1224：800×600 原圖發現短標頭按鈕不必要折行（2026-10-18）

accessible-owned-v2 真14case／7wire／5stage及13游標觀察通過後直接看原圖，關閉兩字與清除搜尋最後字折到次行。共用按鈕AutoWrap在AutoWidth標頭造成desired width回授；僅這兩個短標頭標籤停用wrap，讓AutoWidth依字型量測自然取寬，保留其他長內容及小視窗一般換行。需重新ordinary與設定介面／owned原圖驗證，不把邏輯成功當畫面完成。

## E1223：ScrollDescendantIntoView 預設只讓標題剛進底部（2026-10-18）

owned-v2的accessible-controls.png僅在底部顯示「游標與搖桿」，控制內容未進畫面。SDK預設IntoView採最少捲動；改明確TopOrLeft，再看新原圖，不修改舊成功artifact或宣稱舊圖已有控制內容。

## E1222：兩項唯讀查詢再次未遵守已知路徑／rg參數順序（2026-10-18）

先猜control.history.jsonl（真為control-history.jsonl），另將-g放在rg的--之後被當路徑，均未改來源。已rg --files定位與讀SDK真header。後續必須獨立先列檔，再讀取；rg所有選項在--前，不把選項跟在搜尋目錄後。

## E1221：新 owned 測試猜測 F10 為設定入口（2026-10-18）

accessible-owned-v1 前五case及十筆真游標觀察通過，第六在settings search unavailable失敗。RefreshShortcut真綁EscapePriority（預設Esc），正式HUD按鈕也能開啟，沒有F10設定綁定。改沿既有TryClickTag對 OmControlSettingsButton 的真Slate點擊，不新增假快捷鍵或直接呼叫開啟來掩蓋。publisher／editor清理成功，150來源前後一致後才改；原失敗與已觀察游標資料保留。

## E1220：同名 if context 導致差異生成程式誤貼到 Overlay（2026-10-18）

修 scoped input 時 patch 使用第一個 if(bRoot && IsRows(Key))，錯放 Difference 的 Changed／Previous 程式到 Overlay。立即讀回完整函式捕捉，編譯前已移到有 Previous／Patch 的真 Difference；後續patch以唯一鄰行錨定，不能只憑重複if片段。

## E1219：兩項舊原生斷言未隨四軸與物品目標說明更新（2026-10-18）

accessible-input-native-v1 25項實跑22成功3失敗，其餘兩失敗為 ControlProfileBindingSlots 把全部axis數當唯一camera軸（實際5），與 ControlRuntimePersistence 堅持所有item不需點地（正式已有point／unit物品）。改成驗camera軸唯一且四gamepad軸存在，tooltip驗依裝備與指定地點／單位；保留真正行為斷言，不把原失敗報成功。

## E1218：游標偏好放在 Interface 後漏處理英雄／模式分層（2026-10-18）

accessible-input-native-v1 的 AccessibleInput.Profile 真的失敗：Hero速度1200洩漏Global800。既有ProfileSet把所有Interface包含新input偏好全域化；不把測試改成接受洩漏。新增只允許五個維護輸入欄位的 sparse interface patch，畫面／音量／未知欄位仍不可由hero patch改寫。Edit只全域化presentation部分；scoped reset檢查輸入白名單且清掉空子物件。新增跨hero／跨mode／field reset及拒絕音量／畫面scope回歸。v1結束後150來源一致才修改。

## E1217：來源清單重複列入已存在的 Lua contract（2026-10-18）

v3 capture 印151列，verify真實hash map為150個唯一來源；新增fragment時重複加了原有contract測試。原150檔前後一致未失效；刪除重複列並讓capture遇duplicate直接拒絕，避免以列數誇大唯一來源數。

## E1216：新增 owned 案例漏改編譯期 frame budget（2026-10-18）

accessible-input-build-v3 refresh 在 OmOwnedViewportIpcTests.cpp:149 static_assert C2338 失敗，132.04秒為該UE階段時間，正式runtime連結與五新測試來源編譯成功。等待程序結束後驗150個唯一來源不變，依原每case 90 absence＋5 transition及32 hit slack公式增加14項，總53124；每case timeout不變，不以取消assert或任意延長掩蓋掛住。

## E1215：新增 synthetic history 把 disclosure epoch 誤算為額外 IPC envelope（2026-10-18）

contract 第二次執行在 check_history published_sequence 失敗；額外 envelope 只來自獨立 CommunicationPresentation，disclosure_epoch 自己只產生既有 snapshot。新增155至157 stage 應維持 preceding communication frames=15。修正測試資料並保留正式 envelope 計數驗證。

## E1214：案例靜態盤點未展開新增的 C++ include fragment（2026-10-18）

owned_viewport_ipc_contract_test 首次執行在 accessible-ready inventory assertion 失敗；14案例在正式 include 的 OmOwnedAccessibleInput.inl，舊測試只讀主 cpp。改為先斷言正式 include 存在，再拼接真 fragment 盤點，不移除 inventory 驗證或偽造案例。

## E1213：再次猜測 ProfileSet header 名稱（2026-10-18）

唯讀查詢用了不存在的 OmPlayerControlProfileSet.h；真檔為 OmControlProfileSet.h。rg 報找不到檔案，未影響來源或消費者。後續先以 rg --files 定位，再讀取真路徑，不從相鄰類別名稱推測檔名。

## E1212：確認改鍵初版只覆蓋 world，漏掉小地圖 widget 的實體左鍵判斷（2026-10-18）

來源盤點發現 SOmMinimap 自己決定合法地點施法，原型只改 owner 的 world／keyboard 路徑，非左鍵滑鼠 confirm 在小地圖仍不能操作。沿原 widget 唯一路徑加入 owner-qualified optional confirm query：foreign user 回 unset 且不動別人的 capture，已改掉的左鍵不再確認；camera／right cancel與unit不從地點推斷等原語意保留。profile明確保留右鍵取消，新增policy回歸。此為來源檢查捕捉的未驗整合缺口，不冒稱真點擊失敗／成功；後續需 native與owned實際事件證明。

## E1211：新增軸映射按鈕只傳了 MakeSettingsButton 的三個參數（2026-10-18）

accessible-input-build-v2 refresh 編譯中，OmControlSettingsWidget.cpp:1207 C2660；真共用 MakeSettingsButton 定義需 tag／label／delegate／font／color／tooltip 六參數，新增呼叫只給前三項，後續 SWrapBox C2088 為連鎖錯誤。盤點時截斷舊按鈕呼叫後半而誤以為有三參數 wrapper；下一次修改必須依完整簽名提供 BodyFontAttr 與樣式／提示，不增加忽略字級的 overload。此時其餘 queued compile 尚在跑，等實際結束、核對149來源並保留log後才修，不冒稱本輪成功。

## E1210：初版搖桿配對要求兩軸新事件，未考慮靜止零值省略（2026-10-18）

在首輪 UE 建置消費者結束後，依真 SDK XInputInterface.cpp:306 確認：僅軸值改變或超過平台死區才送 OnControllerAnalog，未動的縱軸可能永不送零事件。初版 ReadPair 必須兩軸都有 cache，會把純水平輸入卡住。改為共用逐軸 FOmAnalogAxisSampler：每個實際使用軸獨立觀察中立後才允許貢獻，未知／舊按住軸封鎖而不阻塞另一軸；不假造實體零值、不要求玩家額外搖動另一軸。cursor 與 WASD mapping 採一致逐軸死區，斜向仍正規化不加速。加入 sparse-zero／未中立軸不能借用別軸中立等回歸。此為 SDK 盤點發現的未驗初版問題，非已宣稱硬體成功或虛構 native 失敗。

## E1209：新 public header 首輪普通建置需要引擎衍生檔 refresh（2026-10-18）

accessible-input-build-v1 真 FailedDueToEngineChange，140.54秒，命令含 NoEngineChanges／NoUBA／nocache；149來源前後一致。UBT列出 Android／IOS／LinuxArm64／Linux 等既有 UnrealEditor.modules 與其他引擎衍生檔需重產生，尚未進入本批 C++ 的成功編譯。保留原log，沿正式 refresh no-cache 流程再回 ordinary gate；不手改 Engine、BuildId、cache 或時間戳。

## E1208：Windows 下把目錄 glob 直接傳給 rg（2026-10-18）

確認改鍵來源盤點時，對 rg 傳入 OmEditor/Private/* 導致 os error123；實際需要的 fixture 已在已知 OmActionOwnerTests.cpp 內。沒有修改來源或建立替代 shell；改查已知檔案。後續目錄搜尋用 rg -g 的檔案篩選，不把 shell glob 當已展開路徑。

## E1207：不變性 helper 重跑使用舊輸出檔名遭保護拒絕（2026-10-18）

native-v4 十成功後，item-label-invariants 再比對成功到寫報告階段，因 summary.json 已存在且 overwrite=false 被正確拒絕。未覆寫舊證據；擴充具名 checkpoint，另存 final-v1.json 再執行檢查。第一份 scoped acceptance v1 引用先前成功的 invariant 與當前源碼 SHA，原檔保留；最終 v2 改引用這次 fresh final checkpoint，不把拒絕寫入當作成功重跑。依賴工具應檢查 exit code 後才啟動接受步驟。

## E1206：新按鈕測試未處理既有 HUD overflow 捲動模式（2026-10-18）

item-label-native-v3 真九成功、一失敗，ResponsiveHudLayout 新 800×600／scale2 完整通過，但 400×300／scale2 要求商店 label 立即在窗口內。該案例已進入現有 OmCombatScroll 的有界 overflow fallback；設定按鈕在外層，商店按鈕在可捲內容，不能以全內容未捲動的幾何聲稱不可達。補真 Slate wheel 到該 scroll、捲動 offset 增加、完整 label 位於真 viewport、鏡頭 zoom 不變、當次捲後 PNG，再恢復頂端供後續欄位檢查；保留設定按鈕始終位於窗口、完整字級／繪製尺寸斷言。最終可達性待 fresh 結果，不先宣稱成功。144 來源於失敗回歸前後一致，原報告與 800／scale2 PNG 保留。

## E1205：新增兩倍字級案例時舊測試公式未涵蓋既有字級上限（2026-10-18）

item-label-build-v4 成功 25.87 秒後，送下一輪 native 前完整讀取文字分支，發現 MinPhysicalEm 仍寫死 10×scale；既有 FOmHudReadableText 明定 BodyPxMax=18，PhysicalBodyFloorEm 同樣有 18 上限。此前案例最大 1.5，沒有暴露差異；新 scale2 會讓 body 18 的合法呈現被舊公式要求至少20。修為共用已存在的 PhysicalBodyFloorEm，不改 production 字型／上限，也不刪 actual paint／ancestor／尺寸斷言。這是來源檢查發現的契約不一致，尚未實跑該新分支，不虛構 native 失敗結果。144 來源於 v4 建置前後一致，完整紀錄保留。

## E1204：修搜尋輸入卻漏改稍後的同值保留斷言（2026-10-18）

item-label-native-v2 真九成功、一失敗；中文部分名稱搜尋、配方／材料點擊與返回均成功，但最後保留搜尋斷言仍硬編碼 Great。上一輪只盤點 SetText，不足以涵蓋同一測試的讀回契約。改用同一 RecipeSearchTerm 同時供輸入與返回後比對，仍獨立檢查正式 catalog 名稱、類別、recipe identity 和零購買序號；不刪除保留搜尋斷言。144 來源於該回歸一致，原結果保留。修改測試輸入時必須盤點整個來源中所有相同 token 與後續狀態斷言，不能逐個等下一輪才暴露。

## E1203：名稱改中文後漏盤點測試中的英文部分搜尋（2026-10-18）

item-label-native-v1 真八成功、兩失敗，ItemIconPresentation 搜尋 Guard 找不到守護符；ResponsiveHudLayout 搜尋 Great 找不到精鋼重劍而無法點配方。正式十一項 manifest／catalog 名稱同步成功，runtime 搜尋正確不匹配舊英文。先前只搜尋完整英文名稱，漏掉部分前綴；修為中文部分名稱「守護」與「精鋼」，保留搜尋排除其他列、配方實際點擊／返回／保留搜尋等斷言。獨立人工英文 fixture（例如 ShopPanel 的 Boots）不是正式目錄，不盲目取代。144 來源於失敗回歸前後一致，原報告／圖片保留，不能把八項成功冒稱整輪通過。

## E1202：字級測試把 UE float Size 與 int32 預期直接傳 TestEqual（2026-10-18）

item-label-build-v1 普通 NoEngineChanges／NoUBA／nocache 真失敗，448.87 秒；OmResponsiveHudTests.cpp:1583 C2666，TestEqual 的數值多載對 float 與 int32 無法決定。Runtime widget、正式裝備目錄與圖示測試已編譯，不代表整輪通過。SDK SlateFontInfo.h 的 Size 為 float，應將預期明確轉 float，保留完整字級與 paint 幾何斷言。144 來源於該失敗建置前後一致，原 log 保存；等待並行 Rust 消費者完成才修改來源，不能在編譯中搶改。

## E1201：按鈕首次修改仍用了建構時固定 BodyFont（2026-10-18）

送編譯前盤點發現 RebuildWidget 的 BodyFont 是固定 11px，其他現有文字依 RefreshReadableFonts 更新；新按鈕若僅 Font(BodyFont) 不會隨偏好改變。已在同批編譯前改為 Font_Lambda 讀 OmShopBodyFont(AppliedTextScale)，並加入 800／400 與字級 2 的實際文字與幾何檢查。不可由變數名稱推測 attribute 或可變性。

## E1200：再次猜測 Lua 共用函式庫位置（2026-10-18）

唯讀盤點誤讀不存在的 scripts/lib/json.lua，實際檔案是 tools/lua/lib/json.lua。未改動來源或執行 fallback；先 rg --files 發現路徑後才讀取。後續不得由 bootstrap 的 lib 名稱推測實體目錄。

## E1199：新短提示hit測試未區分空路徑與攔截且未完成實際tooltip窗口paint（2026-10-18）

hud-hint-input-native-v1 真12成功1失敗：小地圖兩項／responsive與新ABI29兩map world皆成功，只有新短提示的合併hit斷言失敗；IsInteractive=false與實際呈現斷言均成功。原圖顯示已完整呈現的短內容，不能僅由合併失敗推斷仍攔截。新增before／after路徑、實際window與parent visibility診斷，依既有長提示流程FindWidgetWindow完成該真窗口的最後paint，再分別驗路徑有效與不含tooltip；不刪除穿透斷言，也不改production讓測試跳過。確切原因待當次fresh結果，未宣稱只是座標問題。140來源於失敗回歸不變，原12項成功不冒稱整輪通過。

## E1198：新passive提示測試再次混淆TSharedRef與TSharedPtr的Get（2026-10-18）

hud-hint-input-build-v1普通建置23.21秒後C2664：FWidgetPath::ContainsWidget要求const SWidget*，測試卻傳Tip->AsWidget().Get()回傳的SWidget&。Runtime tooltip／host程式已通過該輪編譯，不代表整輪成功。修為明確取址&...Get()，不改SDK或把參照硬轉；140來源於失敗建置全程一致，保留原log。這是已知TSharedRef規則再次漏用，應在送編譯前逐一核對AsWidget回傳型別，不以Get字樣推測指標。

## E1197：短提示不需捲動仍攔截小地圖輸入（2026-10-18）

playable-mvp-current-world-native-v1 中ResponsiveHudLayout短視窗400×234的minimap中心命中SBorder，失敗原圖B918A0BD42A28E6017CB6698A14D5386已直接檢視：五行技能E短提示覆蓋小地圖。既有SOmHudToolTip一律interactive，host一律讓子widget參與hit-test；即使文字全部可見仍搶滑鼠。通用修正應以真ScrollBox overflow區分passive與reading，不以特定座標／關掉測試所有提示處理；溢出仍保留可捲動閱讀。此4項回歸2成功2失敗，另一失敗是E1196；139來源完全一致，不接受為整合world通過。

## E1196：ABI升29後沿用舊ABI28的正式地圖fixture（2026-10-18）

擴大整合world回歸時，Strict compiled fixture identity正確拒絕playable-mvp-world-export-v1：其abi_version=28，當前reader要求29。map catalog hash未變，不代表ABI身分仍有效；不能直接改JSON版號。沿run-compiled-map-export.lua正式Rust producer另存world-export-v2，一個精確export測試成功，ABI29、兩張map、b7a9dda88f881c49、零simulation／ACK；原fixture與失敗證據保留，139來源經匯出後驗證仍相同。

## E1195：responsive 字級分支漏改舊契約且 tooltip decoder 有錯誤向下轉型（2026-10-18）

hero-attributes-native-v2 真9成功1失敗，C面板的actual wheel與buff名稱／更新／生命期都成功；唯一失敗是 ResponsiveHudLayout 到文字1.25才碰到另一個舊診斷字串斷言。全檔盤點後讓兩個分支共用完整玩家屬性預期，不再逐個修到哪裡才找到下一個。此外舊 ToolTipString 與 summoner 分支把任何 IToolTip 裸轉 SDeferredToolTipText；SDK確認它是獨立 IToolTip wrapper，不能用來讀 SOmHudToolTip。改經 AsWidget 取得真 body，確認 SOmHudToolTip／SToolTip 型別後讀 SToolTip 公開 GetTextTooltip，unknown body 回空讓測試明確失敗；去掉所有舊 SDeferred 裸轉型。139來源於v2全程不變，原失敗log／圖片保留，未當作10項通過。

## E1194：responsive fixture 仍把診斷字串當玩家 tooltip 契約（2026-10-18）

hero-attributes-native-v1 為7成功3失敗；ResponsiveHudLayout 要求玩家 tooltip 包含隱藏 OmHeroStats 的英文診斷字串，但新玩家提示只讀正式 typed hero payload，該 fixture 只有 SetHeroStats，未提供有效 local player／body／詳細數值。修為明確供應同一組720/1000、等級3、經驗40與六項型別化屬性，仍保留診斷文字與 collapsed widget 的相容檢查，玩家提示則逐項驗中文與正確數值，不以刪除 tooltip 斷言解決。v1完整失敗artifact保留，不能當成功驗收。

## E1193：小視窗的技能提示遮住 C 閱讀面板並攔截滾輪（2026-10-18）

native-v1 PlayerStatsPanel 的400×300／文字2原圖直接檢視，D技能 interactive tooltip 真正蓋在 C 面板上，實際 wheel hit path 未包含 Stats scroll；這不是只需調整座標的問題。玩家提示 permission 新增同 owner 的閱讀面板優先序：滑鼠進入 Stats 的實際 paint geometry，或同玩家 keyboard focus 在 Stats 內時，該 owner 的 HUD hints 暫不呈現；在面板外仍可正常閱讀提示，不全域關閉其他玩家／視窗的 tooltip。測試在真 mouse move 後跑 Slate paint，檢查更新後 hit grid 並保留實際 wheel／捲動斷言。小地圖文字 scale binding 同時改用既有弱 UObject 模式，避免新增屬性將被外部保留的子 widget 綁到裸 owner。尚待重建與 fresh native，未聲稱成功。

## E1192：buff 玩家資訊測試未保留 Slate root 而先觸發退休（2026-10-18）

native-v1 OwnedBuffListSnapshot 的正式 catalog 名稱／純 formatter 皆成功，但玩家讀值與名稱單獨更新兩項失敗。來源確認 fixture 用 Widget->TakeWidget() 的臨時回傳值，SObjectWidget 隨即銷毀，NativeDestruct 呼叫 RetireGameplayInput；玩家 getter 因已退休正確回傳空。修為在整個 fixture 保留 Slate root，新增未退休前置斷言，保留所有顯示／名稱更新／世代清除斷言，不能移除正式 retirement guard。139份來源於失敗回歸全程未變；v1四張原圖有產生，目前只檢視800文字1與400文字2兩張做診斷，尚未接受。

## E1191：唯讀焦點檢查誤將 viewport client 宣告為 const（2026-10-18）

hero-attributes-build-v2 refresh 的實際 C++ 編譯 C2662：HasOwnedReadFocus lambda 使用 const auto* Client，新增呼叫 UGameViewportClient::IgnoreInput() 並非 const 方法。應保留 GetViewportClient 原回傳的可變指標型別，不能 const_cast 或移除輸入檢查。錯誤觀察時 consumer 尚在跑，先只記錄文件，等實際退出並驗 SHA 後才修改來源；本輪尚未成功，後續原生測試不可提前執行。

## E1190：普通屬性建置被 NoEngineChanges 擋下（2026-10-18）

hero-attributes-build-v1 ordinary local／NoUBA／nocache 真失敗：UBT FailedDueToEngineChange，168.74秒，要求更新現有引擎 module 產物；不是已證實的 C++ 功能編譯失敗。134份來源於正式 codegen／stage 後仍與 source-v2 完全一致。依既有流程執行明確 refresh no-cache，之後再跑普通 gate；不改引擎來源、BuildId、manifest 或快取來遮蔽阻擋。原 summary／build.log／UBT-Log.txt 保留。前次進度訊息稱前置 bridge release，實際 process inventory 曾顯示 base_content compiled-content-only debug linking；以正式 build.log 為準，不從無輸出的等待推測階段。

## E1189：內容產生器目錄再次未經 inventory 確認（2026-10-18）

盤點後續 buff 名稱時誤猜 omfue/bridge/codegen 為產生器目錄，rg 回報不存在。未修改來源、未影響正在執行的 UE 建置。後續探索先以 clone 根目錄 rg --files 找實際 manifest／產生器檔名，不以功能名稱推測實體路徑。

## E1188：唯讀探索附帶未確認的工具庫路徑（2026-10-18）

在檢視屬性顯示來源時，額外 rg 猜測 scripts/lib/process.lua，檔案不存在。沒有來源變更，也沒有阻擋既有 Rust consumer；該查詢與當前功能無關，不應附帶執行。後續僅對 inventory 確認的來源與必要符號查詢。

## E1187：ABI 升版仍漏掉既有 mode 測試的舊常數（2026-10-18）

hero-attributes-rust-v1 Core782／Client147成功，Bridge125成功、1失敗、1既有忽略；失敗為 authoritative_mode 測試仍要求 ABI28，而正式 ABI 已29。新增屬性跨 ABI 測試本身通過。改為比較正式匯出函式與維護常數，保留專門 ABI29 layout 斷言；不可將本批當完整通過。另來源審查發現唯讀 HUD focus 可能仍讓 SViewport 收到遊戲鍵，加入共用 Slate metadata 輸入範圍與 ancestor guard，Stats 消耗非閱讀鍵，並補 Q／S／H repeat 的真 Slate 路徑測試，待 Unreal 驗證。

## E1186：在 cfg 前插入函式時須保留原函式的 feature guard（2026-10-18）

新增 map_hero_combat_attributes 時插在原 map_summoner_hud_row 的 cfg(runtime-driver) 後，來源審查發現原 guard 會改套新函式，讓舊函式在 no-default-features 引用 optional core。正式 compile 前補回兩個函式各自的 guard，並將 no-default-features --lib 納入此批必要檢查。未把尚未執行的 feature check 當成功；同時維護 ABI29 明確 layout、舊方向輸入測試改用當前常數，避免再次只改主 lib 而漏 consumer。

## E1185：filtered ECS 公開世代更新沿用舊永久 modifier（2026-10-18）

hero-attributes-core-compile-v1 正式 protoc／Core test compilation 成功，14來源凍結；三個新 producer tests 實跑 core-tests-v1 為2成功1失敗。真 Specs stepper 的 disclosure epoch 1→2 後攻擊仍50，預期新來源基底40；來源確認 synchronize_specs_membership 只更新 identity 欄位，沒有退休舊 entity，原 stale刪除亦未清 BuffStore。改為缺 body 或世代不同均刪舊本地 life，先 remove_all_for 清永久 modifier／索引再 delete_entity，然後正常 restore。新測試同驗新世代、缺 component、重複 owner及刪除後不留 buff。不可在 presentation 上覆寫40來遮蔽底層生命期。另曾在 compile consumer 結束前讀彙整log，檔尚未建立；必須先 poll completion，再讀該產物。v1已有既有 authoritative_mode unused import／template dead-code warnings，與新功能結果分開保留。

## E1184：patch context 把 FName(TEXT(...)) 簡寫而未匹配真來源（2026-10-18）

Esc／focus 修復首次整批 apply_patch 因 Built->SetTag 真正使用 FName(TEXT(...)) 而驗證失敗；查詢確認各修改均未落地。重新讀精確行再套用，沒有啟動失敗 patch 後的 build。不得把 summary 的語義簡寫當成完整 source context。

## E1183：新角色面板漏列中央 Esc 動態啟用條件（2026-10-18）

player-stats-panel-native-v3 的首例 C／字級／真 wheel／面板內邊界已走到 Esc，但 viewport Esc 仍未關閉。查明 SetEscapeBinding 已集中 OR shop／scoreboard／history／selection，漏新 Stats；不能只加 HandleEscapePressed 分支就視為接好。將 Stats 加入同一中央條件並於 C handler 即時重算，其他取消流程亦共享此條件。native 分別驗 viewport Esc 與真可focus的 ScrollBox／UMG preview Esc，及關閉後焦點恢復；原 v3 失敗保留。

## E1182：角色資訊固定高度造成大片空白（2026-10-18）

直接檢視 player-stats-panel-native-v2 原圖，800×600 一般字級僅七行資訊卻固定520高，遮蓋遊戲區。此圖不接受為玩家完成品質。改用內容量測與 MaxDesiredHeight，而非固定 HeightOverride；真 viewport 上限及明確 wrap width 保留，超長文字可逐字換行。窄窗／放大字透過同一 ScrollBox 取用，不縮字或猜固定內容行數。待 fresh native／原圖重驗。

## E1181：Stats 讀取焦點無法關閉，且 live profile 不接受 preview 字級（2026-10-18）

後續來源審查補正：toggle 資訊面板的 focus guard 必須承認同 local player、同 active window 的 readonly HUD descendant focus，不能把 ScrollBox 焦點當 application focus loss；文字輸入／modal 仍獨立攔截。native 新增真 wheel 後執行 owner tick，驗下一 frame 仍可閱讀。只沿焦點父鏈查 owned root，不每 frame 搜尋 widget tree。

player-stats-panel-native-v2 真 C 已可開啟，但實際 wheel 讓 focus 留在 ScrollBox，按鍵不再送到 sibling SViewport，Esc／再按 C 無法關閉。沿既有 CommandBar NativeOnPreviewKeyDown 補 readonly stats descendant 的當前主副鍵與精確 modifier 關閉；close 只在焦點仍屬此面板時恢復 local viewport，不搶其他文字輸入。測試同時誤用 preview：production 明確在 live profile 存在時忽略 preview，改用已隔離隨機 player 的正式 TrySetInterfaceSettings 保存字級。原 v2 及其他四成功保留，不假稱 Stats 全通過。

## E1180：C 的 Slate 輸入測試漏跑 point-click 原生 PlayerInput frame（2026-10-18）

player-stats-panel-build-v1 成功，native-v1 其餘四項成功；新 Stats 在第一次 C 失敗。來源確認 RouteDirectionalKey 的直接 delegate 路徑僅限 WASD，預設 point-click 會排入 UPlayerInput，不能只呼叫 Slate.Tick 就當已處理。測試補真 Controller.PlayerTick 於 down/up 事件之後，保留原完整輸入鏈，不直接呼叫 handler。另記錄 details／profile／active window／focus，若仍失敗再依真證據判斷；原 v1 保留，未接受 C 功能。

## E1179：唯讀盤點再度使用 Windows wildcard path 與猜測 model.rs（2026-10-18）

屬性公開鏈盤點時 rg 收到 native/*.rs 作路徑，Windows 不展開而失敗；另猜測 codegen/src/model.rs 不存在。來源未因此修改，建置 consumer 不受影響。後續只允許 rg -g '*.rs' 搜尋已證實存在的目錄；讀新檔前用 rg --files 得到完整真路徑，必須分開工具回合，禁止同批列檔後仍預猜讀取。這是已知規則重犯，保留紀錄以便後續檢查。

## E1178：Stats 預設 binding 有註冊但沒有正式 consumer（2026-10-18）

盤點發現 C／Stats 只有 profile row，implemented list、按鍵 delegate 與實際面板全部缺失；不能把可改鍵設定當功能完成。接單一 owner 的主副鍵／精確 modifier／repeat與profile變更退休，bounded readable panel、Esc／失焦／來源更換清除，僅顯示目前正式 payload 已公開的角色資料。原4.5完整攻防屬性仍需正式 producer／transport／ABI，不能填零冒稱完整。唯讀盤點再度誤猜 OmPlayerControlInterface.cpp、omfue/rust、Content/OmGenerated/HudAssets.json；實際為 OmPlayerActionOwnerComponent.cpp、omfue/bridge、Content/Omoba/UI/hud-assets-v1.json。必須先看列檔結果，再決定下一個讀取，不在同一 call 預猜檔案。

## E1177：提示測試與前序 HUD 測試合跑時游標停在舊 capture 座標（2026-10-18）

hud-text-cleanup-native-v1 中 ItemIcon／MinimapInput／PlayableMvpHud 成功，但 ReadableHudToolTip 首例 cursor=(910,0)、hit path 空白而失敗。保留原失敗，不假稱新來源 tooltip 通過。SDK確認 SceneViewport hidden capture 的 move 會復原舊座標，controller BeginPlay 也可覆寫 fixture 原本 NoCapture。讀取測試在 setup 明確恢復該 fixture UI policy 並釋放本地 capture，另記錄 requested／actual cursor 以驗明，而不移除真 hit-test。盤點曾誤猜 OmContentAssets.cpp，該檔不存在；後續只沿實際 CommandBar／HudAssetRegistry 查詢。

## E1176：提示重複 self 快捷鍵且小地圖仍寫入診斷狀態（2026-10-18）

tooltip 原圖審查發現同時列 compact self 行與完整主副鍵 self 區塊，重複又拉長內容；保留完整兩組主副鍵，移除 compact 重複。文字盤點另發現 SubmitMinimapMove 仍將 Move queued／原始拒絕寫入連線狀態列；改為只在拒絕時沿既有 ShowPlayerActionFailure，成功不覆寫連線狀態或冒稱已抵達。英雄 placeholder 中文化；不新增權威狀態與輸入派送路徑，待本批驗證。

## E1175：UWidget 方法內 Slot 區域變數遮蔽繼承成員（2026-10-18）

player-shop-text-build-v1 報 C4458：錢包 inventory loop 的 Slot 遮蔽 UWidget::Slot，UE 將警告視為錯誤。改用明確 ItemState。順帶檢查驗收殘留斷言，64筆容量的真 foreground tooltip 應比對中文，而診斷 getter 保留英文；原失敗 log 保留，未跑 native。不在建置途中修改來源。前一次查 build.log 尚未建立，後續追即時進度只讀 UBT 原 log，runner 結束後才讀彙整。

## E1174：玩家商店提示沿用診斷文字且歷史把未知 action 當購買（2026-10-18）

來源盤點確認 wallet／shop foreground 仍顯示英文診斷，legacy receipt 只檢查 RequestId>0，並把非 Sell 的任意 action 當 Buy。改為兩種文字共用 typed history formatter，只接受 RequestId=0、合法 owner／epoch／17或18 action、非負公開 result 的 legacy history。玩家文字直接沿既有 typed phase／reason，不翻譯診斷字串。錢包顯示已公開商店／完整背包資訊，缺資料不猜空背包或店外；更換共用提示後，測試 helper 先判別真型別再讀取，避免把 SOmHudToolTip 錯轉 SDeferredToolTipText。待 fresh ordinary／native 驗證。

## E1173：接受證據 helper 誤用不存在的 path.dirname（2026-10-18）

八項 native 成功後，證據彙整 helper 呼叫不存在的 path.dirname 而失敗，未產生接受 summary。實際工具是 tools/lua/lib/path.lua 的 parent；唯讀查詢另誤猜 scripts/lib/path.lua，已列檔修正。改用真 API，以新 evidence id 重跑，不覆蓋失敗目錄，不需重跑未變更的 C++。

## E1172：GC fixture 不能直接建立 abstract UObject（2026-10-18）

readable-tooltips-native-v4 四組實際 hover、scroll、字級及最後一行均通過；結尾 NewObject<UObject>() 在 UE5.8 觸發 abstract class ensure。改以既有 concrete UOmRuntimeSettings 作弱生命週期 fixture，不隱藏 ensure，也不把此輪算成功。補上實際離開來源與提示範圍後的關閉與焦點恢復斷言，與 Esc、退休路徑分別驗證。

## E1171：窄版放大文字的卡片未先捲入可見區（2026-10-18）

readable-tooltips-native-v3 前三例真 hover／wheel 成功；400×300 text2 的目標卡片未被 paint，讀到前輪 geometry，實際 hit chain 是召喚師卡。測試改用正式 hotbar scroller 的 ScrollDescendantIntoView、完成 paint 後再移動滑鼠；不以手動 ShowTooltip 或跳過 hit-test 掩蓋。另曾在 runner 結束前讀 native-report 而檔案尚未建立，後續必須先等待實際結束。原失敗保留，尚未接受完整 tooltip 範圍。

## E1170：FArrangedChildren 不支援 range-for（2026-10-18）

readable-tooltips-build-v7 在新增 hit path 診斷時報 C3312：FArrangedChildren 沒有 begin/end。改用既有 Num() 與索引存取，保留完整失敗紀錄；診斷程式也須遵守實際 SDK 容器 API，不能由一般容器慣例推定。此輪未執行 native，不視為功能驗證通過。

## E1169：tooltip驗收在初始disabled-card hitpath失敗（2026-10-18）

readable-tooltips-native-v2中Responsive成功，tooltip在初始卡片命中斷言失敗，尚未取得本版wheel成功證據。加入每例size／scale／cursor／card geometry／host狀態及實際hit chain；setup暫停tooltip生成，排版及移到來源後先驗真hit，再開啟正常hover，避免既有cursor位置在setup自動打開提示而污染階段。這是待驗假設，不把未執行的wheel／focus當成功；原v2保留。

## E1168：TSharedRef::Get回傳參考，不能傳給raw pointer介面（2026-10-18）

readable-tooltips-build-v5 runtime host已編譯，native測試四處把Tip->AsWidget().Get()傳給IsPresenting(const SWidget*)而C2664。UE的TSharedRef::Get是SWidget&，不同於TSharedPtr::Get；改host介面收const SWidget&比較地址，不做危險cast。保留v5。另按SDK visualizer流程，在pointer進提示時暫留interactive內容，明確close用weak host＋exact hint identity回呼，避免stale關閉新提示或source GC後洩漏。

## E1167：SOverlay slot沒有Padding_Lambda便捷API（2026-10-18）

readable-tooltips-build-v4在SOmHudToolTipHost.cpp報C2039：SOverlay::FSlotArguments沒有Padding_Lambda，後面的bracket錯誤是連帶結果。consumer結束後，改為專案既有的Padding(TAttribute<FMargin>::CreateLambda(...))。保留v4失敗與新source指紋，fresh ordinary gate重驗；不可因其他widget有同名便捷method就推定slot也有。

## E1166：沿SDK型別名稱猜.cpp位置再度讀取失敗（2026-10-18）

OmHudResolutionNormalizer只有inline header，沒有對應cpp；STooltipPresenter實際位於Widgets/LayerManager而非Widgets/Layout；TransformCalculus2D也在Core/Public/Math而非猜測的SlateCore/Rendering。唯讀讀取失敗後，已用rg --files取得真路徑再讀。禁止根據include或類名推定source存在／位置；後續一律先列實際檔案。paint scale最後直接沿既有成功測試的absolute/local ratio量測，未引入猜測的matrix API。

## E1165：遊戲tooltip visualizer不可互動且繞過HUD normalization（2026-10-18）

readable-tooltips-native-v1的Responsive成功，真tooltip hover及stored font通過，但實際wheel無法捲動而失敗。SDK查證SGameLayerManager::OnVisualizeTooltip將內容交給HitTestInvisible的STooltipPresenter；先前FindWidgetWindow拿到的是主視窗，不能當成獨立tooltip窗或量測提示大小。原圖亦顯示受viewport curve縮小。改為專案SOmHudToolTipHost在既有resolution normalizer內接管本專案提示，保持實際paint／fit／hit-test同一空間與可互動內容，不改engine。native改查真提示矩形／paint scale／host及wheel hit path，保留原失敗。

## E1164：tooltip ordinary gate實際EngineChange拒絕（2026-10-18）

readable-tooltips-build-v1沿正式NoEngineChanges／local／nocache流程，UBT4.41秒以FailedDueToEngineChange拒絕，沒有新tooltip C++成功證據；101來源指紋在拒絕前後一致。保留log，owner consumer完全退出後才修正與正式refresh，不手改engine／BuildId／cache。補充source review：商店同一次rebuild共用一份immutable revision字串，避免每個按鈕各複製整個catalog signature形成平方記憶體成本。

## E1163：玩家工具提示仍使用引擎固定8pt與1000寬預設（2026-10-18）

被動加成盤點讀SDK確認CoreStyle.cpp的ToolTip.Font=8，SToolTip.cpp全域預設wrap=1000。現有ToolTipText不跟玩家text scale走，長道具說明在小視窗也沒有owner範圍限制。後續以專案共用有界tooltip元件處理font／DPI／長內容與退休，不改engine style／全域CVar，也不把直接讀GetToolTip內容當成已驗過真hover可讀性。

## E1162：猜測可讀字體helper所在檔名而讀取失敗（2026-10-18）

嘗試讀不存在的OmHudReadableText.h，實際FOmHudReadableText定義在OmHudViewportSpace.h。已先以rg --files與符號搜尋定位實際宣告；有export macro時不要只用struct＋名稱相鄰的regex。唯讀失敗未修改來源或影響建置。

## E1161：rg結束選項後又放-g導致唯讀搜尋錯誤（2026-10-18）

尋找codegen正式CLI時，將-g放在--之後，被rg當成檔名，回報os error 2／123。正確順序是rg -n -g '*.lua' -- 'pattern' 既存目錄；所有flags必須在--之前。已修正搜尋並讀實際main.rs，確認輸出參數是--out，不猜--out-dir。未執行錯誤生成命令或修改生成檔。

## E1160：歷史compact測試要求隱藏MVP必要道具倒數（2026-10-18）

player-item-text-native-v3 targeted item流程已通過正常／窄版icon與狀態，續至cooldown時舊測試要求compact不顯示倒數chip而失敗。現行MVP把剩餘秒數放圖示內，所有density都必須可讀；既有PlayableMvpHud已有真圖示內數字契約。將此歷史斷言改成實際4秒數字、真paint字級、位於同icon與client內；不再套用舊「倒數不可與icon交疊」佈局。保留v3，fresh gate繼續完整item流程，不把尚未跑到的scoreboard圖當作完成。

## E1159：歷史item圖測試還把未提供的ward長狀態當常駐文字（2026-10-18）

player-item-text-native-v2真9項已有8項Success／0warning；ItemIconPresentation在無ward protocol的fixture仍要求常駐完整ward長字串而失敗。MVP的正式ward readout只顯示已公開charges／recharge，無protocol時應空白，完整狀態已在tooltip；ResponsiveHudLayout同一契約本輪成功。修正歷史item fixture為無資料不顯示數字、tooltip保留邏輯狀態、hotkey真paint及bounding驗證不變。不是讓測試接受消失的必要資訊，更不能偽造charge數值；保留v2並fresh重驗完整item測試。

## E1158：本批回歸抓到舊可見文字與商店測試範圍假設（2026-10-18）

player-item-text-native-v1實際9項有3項失敗，不能接受workflow=false為成功。NativeItem／Scoreboard／PlayerItemText等6項成功；ItemIconPresentation仍要求已由MVP移除的重複status常駐（Responsive測試早已明確要求Collapsed），另有ready／Empty英文字面未更新；NativeShopInput遍歷整個HUD，把非商店的結算按鈕也當商店focus錯誤。修正為圖示／hotkey／倒數真paint與當前tooltip保留狀態／效果／改鍵，邏輯status仍Collapsed；商店focus只檢查OmShop按鈕，交易按鈕計數與禁用檢查保留。另每個計分板尺寸／字級先ScrollToStart再取start圖，避免上一案例的scroll offset污染標籤。原失敗及圖保留，以fresh build/native再驗。

## E1157：背包道具tooltip漏接既有公開效果說明（2026-10-18）

盤點P0.3發現BuildItemHotbar已填入正式catalog生成的Tooltip，但WriteItemSlotLabels只拼名稱／狀態／改鍵，導致玩家在背包看不到效果。將既有公開Tooltip接進同一張卡，不另存字典或反查內部ID；只有當前非空ItemId可顯示，空欄位不帶出殘留內容。以真widget tooltip驗明所有名稱／效果／改鍵仍可取用，不以商店顯示成功替代背包。

## E1156：呈現helper與FFI型別OmScoreboardRow撞名（2026-10-18）

player-item-text-build-v1在新增OmScoreboardRow helper報C2872，實際FFI header已有同名型別；後面Slate語法錯誤都是連帶結果。consumer退出後改名OmPlayerScoreboardRowText，保留原失敗與fresh建置。新增helper應以用途＋Text／Format等完整命名，先盤點同名符號，不能只根據匿名namespace推定不會衝突。

## E1155：Markdown反引號截斷JS raw template（2026-10-18）

更新操作手冊時，把含Markdown inline-code反引號的patch直接放進JS raw template，functions.exec解析階段報SyntaxError，沒有執行工具或修改檔案。改用普通JS字串傳patch並保留實際換行。raw template適用含反斜線程式碼，但含反引號的Markdown必須另行正確編碼，不能直接拼入。

## E1154：UE Windows wrapper不提供FALSE巨集（2026-10-18）

managed-next-build-v2正式refresh的C++編譯在OmManagedMatchControl.cpp兩處FALSE回報C2065；UE WindowsHWrapper隱藏了此巨集。改用Win32 BOOL的0值與OpenJobObject繼承參數0，不引入額外Windows巨集或修改engine。保留v2 log，consumer結束後修正，再以fresh ordinary gate驗證。

## E1153：本批ordinary建置實際觸發EngineChange拒絕（2026-10-18）

managed-next-build-v1使用正式local／NoEngineChanges／停用action cache，UBT在33.50秒以FailedDueToEngineChange拒絕，尚未取得新C++編譯成功證據。保留build與UBT log，確認來源指紋及consumer退出後，才以正式--refresh-engine重試；禁止手改engine、BuildId或cache繞過。

## E1152：混合patch依賴不相鄰context而整批拒絕（2026-10-18）

新增subsystem宣告的patch把public方法與private直接相鄰當成context，中間其實有既存automation條件區塊。apply_patch拒絕整批，確認新檔仍未修改後改成小型唯一context再套用。來源修改後必須讀回；同次調查再犯Windows wildcard path搜尋，已按E1150改為既存目錄與-g，避免拿查無內容推斷API不存在。

## E1151：coordinator測試手算參數數量錯誤（2026-10-18）

managed-coordinator-v1在兩代快照檢查後，測試把12個原始參數誤寫成13而失敗。改成執行前後完整JSON比對原始arguments，直接檢查未被修改，不依賴手算長度。保留v1失敗summary，以新目錄重驗；不將此失敗當成coordinator成功證據。

## E1150：Windows rg再度使用檔名萬用字元（2026-10-18）

調查作者配方時，將scripts/lua_data/moba_*.lua直接交給rg，Windows回傳os error 123。已改成rg scripts/lua_data -g 'moba_*.lua'。後續一律使用既存目錄配合-g，不在path參數放萬用字元；本次是唯讀搜尋錯誤，未影響建置或驗證結果。

## E1149：all-ready＋exit0仍可能是手動關窗，不能當成協調清場完成（2026-10-18）

下一局模型審查發現：所有玩家ready後，原模型只收exit0便可通過retire；若玩家直接關窗而非處理coordinator通知，也可能正常exit0。新增獨立retired通知，須同scope／player／原ready epoch與input／HUD／runtime三項真正退休旗標；模型先confirm_retirement再接受正常退出，缺通知的exit0取消下一局。GUI只有協調離場才可寫通知，手動離場不寫；parent確認child退出後再讀一次通知以關閉發布競態。新增精確反例／wrong epoch／未停runtime與任意字段拒絕，不把本地清理通知稱為權威ACK。

## E1148：退出收據發布與waiter退出之間有觀察競態（2026-10-18）

owned-run-platform-v2 原生25tests與production build成功、九來源一致；實際整合在32斷言後失敗，cleanup成功。三個原生exit.json均已存在，值為0／3／3221225477；第三輪父測試先觀察file不存在，接著查waiter時它已完成並退出，錯誤立刻斷言失敗，漏了最後一次讀取。新增共用process.poll_owned_exit：在確認原始writer已退出後重讀、嚴格比對同birth收據，缺失／格式錯仍拒絕，不能直接當exit0。以實際退休程序＋一次stale file observation重現該競態；不是放寬原始PID／creation token檢查。另將新run能力明確要求既有private workflow實際membership，避免獨立waiter崩潰留下未受supervisor保護的後代；原spawn語義不變。v2保留false summary／logs，fresh v3重驗。

## E1147：capture失敗後仍呼叫依賴它的rust-gates（2026-10-18）

owned-run-platform-v1 capture返回exit1，但同一工具編排未檢查結果就接著呼叫rust-gates，造成缺少source-before.json的第二錯誤。沒有實際Rust build／test，不能稱平台驗證開始。之後capture、依賴gate分開檢視成功再繼續，不因await順序正確就忽略exit code；原v1目錄與兩筆失敗保留，fresh v2才重做。

## E1146：evidence helper多餘lfs.chdir使Windows cwd成verbatim路徑（2026-10-18）

owned-run-platform-v1 capture的helper先lfs.chdir(b.root)，cmd隨後報UNC cwd不支援，relative input經path.absolute折成無效的單斜線?前綴；原生SHA回os error123。這是新evidence helper自己改cwd造成，未啟動Rust gate或任何子對局。移除不必要chdir，SHA input一律對已知clone root解析，並以工具明確workdir執行；不硬編D作產品fallback、不改既有spawn語義。v1保存request／原生error response與errors.md，改fresh ID重驗。

## E1145：協調模型初版將程序退出碼限制成 int32，漏掉 Windows crash NTSTATUS（2026-10-18）

初版managed-session純Lua242斷言通過，但接平台程序結果前核對發現player_exited只接受0..INT32_MAX，Windows access violation的0xC0000005是合法u32退出碼，應取消下一局而不是留在closing。調整為完整u32範圍，成功仍只認0；新增高位NTSTATUS導致cancelled與無法commit斷言。接下來的native wait adapter必須保留原始u32位元，不把負i32轉成0、不把PID消失當作exit0。

## E1144：UE RunTests 的前綴選取多跑 NativeMinimapInput，expected清單漏列（2026-10-18）

vision-text-build-v1 ordinary31.15秒成功；native-v1真exit0，但嚴格workflow拒絕unexpected Om.Runtime.NativeMinimapInput。UE對NativeMinimap的前綴選取同時包含Input，原expected只列3個；不把workflow=false稱成功，也不放寬unknown／duplicate gate。補列真實四個完整名稱後另用fresh native ID驗明；當前owned phase獨立執行，不同時開第二個互動UE consumer搶focus。原report與錯誤summary不覆寫。

## E1143：查共用 process library 時再次猜錯 scripts/lib（2026-10-18）

唯讀調查session協調器時對scripts/lib/process.lua執行rg，路徑不存在；改先rg --files scripts tools，實際為tools/lua/lib/process.lua，_bootstrap.lib也明示載入tools.lua.lib。未新增fallback或修改檔案；這是已知「先查清單再讀檔」規則再次違反，後續調查直接沿bootstrap所指定的library根目錄。

## E1142：兩倍字體原圖的名稱＋數值同列仍截字，模型寬界成功不等於文字完整（2026-10-18）

health-budget-build-v3 ordinary8.62秒成功，native-v3 exact7 Success／0warning、73來源一致；training一般與文字2原圖直接檢視後拒絕大字版本，550/550被截成550/55，名稱首字亦被center截去。移除production世界名牌的重複HP字串，只以正式公開DisplayName作名稱列，生命用血條、精確數值保留既有自身HUD與選取目標資訊；原完整Label payload不改以維持檢查與legacy相容。名稱左對齊並保留ellipsis，不縮小28pt字級或無限加寬；待fresh完整當前範圍及兩map原圖檢查。這不是7項測試失敗，明確區分原圖品質拒絕。

## E1141：新 Slate 名牌測試用未 Initialize 的 NewObject，觸發 WidgetTree null crash（2026-10-18）

health-budget-build-v2 ordinary23.13秒成功，native-v2 NameplateUpdateOwnership成功後，新HealthNameplateBudget的NewObject<UOmUnitNameplateWidget>直接TakeWidget觸發UUserWidget::OnWidgetRebuilt→UWidgetTree::ForEachWidget，read0x38／真exit3，沒有完整report，不稱7項完成。保持production介面，測試改建立有world context的EditorPreview world，以正式CreateWidget初始化WidgetTree，並保留Slate root到最後；不套用其他有自訂初始化的CommandBar測試模式。73來源於崩潰前後一致，原engine callstack完整保留。

## E1140：小兵血條功能已通過，原圖名牌基準文字仍過小（2026-10-18）

health-budget-build-v1 ordinary92.18秒成功，native-v1 exact7 Success／0warning，73來源一致；兩map一般原圖直接檢視，兩隊滿血小兵顯示與source撤銷斷言均通過，但名牌9pt經720p DPI後中文約6px，不能把有字當成可讀。基準改14pt、兩倍28pt；名稱列Fill寬＋ellipsis保留完整payload，名牌／compact血條有界隨文字放大，避免固定小字cap14的舊問題。補真Slate font／長名稱200字寬界／無名稱關係／低血獨立!與兩map文字2原圖；不改權威資料或縮字迎合測試。未接受為完整畫面，待fresh驗證。

## E1139：把未讀取的 legacy wrapper 誤寫成不存在（2026-10-18）

E1138初稿以「只保留四個root bat」規則推斷run_2player_ue.bat已移除，但同次rg實際清單列出它仍存在。這是錯誤推斷，立即更正記錄；規則不能替代檔案清單。README改用現存Lua正式入口仍合理，不刪除或改動這個既有legacy wrapper，也不把它當成本輪新增。之後先完成工具回傳檢查再撰寫存在性結論。

## E1138：啟動說明仍有舊 clone 絕對路徑與 legacy root 入口（2026-10-18）

補root MOBA入口後交叉核對README，發現仍聲稱bat把Lua寫死在D:/code/omoba，實際四個指定bat均已用%~dp0；表格也仍列legacy run_2player_ue.bat，與現在由固定Lua直接提供其他工具的政策不一致。改成已實際讀取的clone-relative說明、現存scripts/run_2player_ue.lua入口與正式MOBA範例；不新增root bat或刪改既有legacy wrapper。真root help兩路與非法參數六項程序檢查已成功，沒有為了文件校驗啟動對局。

## E1137：關係原圖仍是藍血條、模型未等插值，色盲快照誤換 governor（2026-10-18）

world-relation-native-v2 的 WorldBridgePresentationScale通過，但training兩張新增原圖直接檢視未接受：血條都是預設藍色，移動後的名牌和舊模型未對齊，色盲後標記全變?。先用widget自有白色FillImage與深色BackgroundImage排除預設style brush色盤；同步automation task不推進global frame counter，新增64次實際actor Tick讓正式插值收斂，不停用production插值。色盲renderer快照先解析真正GoverningOwner（可能null），不能把尚未govern的Action塞進Bind而觸發合法的關係退休。不是完整畫面／MVP通過，原圖保留；待fresh build／native／原圖重驗。

## E1136：StopRuntime 是 destroy 路徑，E1133 的 pool-only 修正不足（2026-10-18）

ordinary build-v2 17.38秒成功、71來源一致；native-v2四項中3成功，NameplateUpdateOwnership最後退休斷言仍失敗。仔細追蹤確認 ResetRuntimePresentation→ReleaseAllActors(true) 直接Destroy，當前EndPlay僅清buff／audio與listener，沒有走ResetPresentationState；E1133的「回池」定因不精確。改共用EndPlay及ReleaseAllActors兩條入口呼叫正式ResetPresentationState（包含未BeginPlay actor），保留widget退休斷言，不只改測試或只設Unknown。reset既有差異快取避免第二次清理重寫Slate，原失敗完整保存。

## E1135：renderer 截圖不得借用玩家 1 的持久設定（2026-10-18）

新四色盲截圖初稿使用 Action.TrySetInterfaceSettings，來源核對發現此正式API會寫玩家profile；compiled renderer使用固定synthetic player1，不能借用／覆寫使用者設定。尚未建置或執行初稿即改走名牌既有BindGoverningNameplate快照與地面圈NotifyInterfaceChanged，只切renderer，隨後還原；此圖不宣稱再次測試profile保存，正式保存契約由既有獨立隨機玩家測試負責。

## E1134：舊 compiled 截圖 fixture 缺 source marker，不能提供敵我色證據（2026-10-18）

來源審查發現 OmCompiledMapSceneCapture 的合成 entity 有 source player，但沒有 OM_RELATION_SOURCE_ROW／SOURCE_ALIVE，敵方也未標明 HOSTILE；正式分類按規則只能 Unknown。這不是權威資料錯誤，不在production依owner／team猜測補認證。補齊明示合成frame的合法source旗標與敵對旗標，再額外佈置己／友／敵／未知四者，逐個斷言真actor widget分類，以正式profile切色盲，保存同視窗world＋HUD兩種原圖。既有synthetic標示保留，不冒稱正常真對局。

## E1133：池化清理只隱藏 component，保留 widget 仍有上次關係與名稱（2026-10-18）

world-relation-build-v1 ordinary116.14秒成功，71來源核對一致；native-v1兩項中 PublicUnitTeamRelations成功，NameplateUpdateOwnership新增最後退休斷言失敗：source StopRuntime先將actors回池並清空索引，後續ClearPublishedDirectTargets已無法觸及保留widget，所以舊「己」仍留在隱藏widget。改在actor共用ResetPresentationState立即清空名牌文字、HP與顯示，不靠後面的索引掃描；補reset當下label空／badge隱藏斷言。原失敗logs完整保留，不放寬退休測試。

## E1132：新增測試觀測曾假定 SProgressBar 有顏色 getter，建置前查 SDK 移除（2026-10-18）

世界關係血條實作審查時，D:/UE5.8 實際 SProgressBar.h 只有 SetFillColorAndOpacity，沒有先前寫入的 GetFillColorAndOpacity。尚未啟動建置即移除該不存在的呼叫，不捏造編譯失敗。原生測試觀測真 Slate 關係文字／可見性、公開分類與更新次數，色盤契約檢查各色盲模式不隨血量混成敵我，實際顏色另由整合世界原圖審查。之後先核對當前 SDK 再加 renderer getter。

## E1131：結果實際點擊未送達 callback，來源資格與清理路徑已分離診斷（2026-10-18）

post-match-native-v3 exact5中4成功；PostMatchLeaveOwnership所有資格／1280、400及文字1、2隱藏HUD的panel／exit範圍通過，physical click仍quit0。真資格result1／viewPlayer1／configured1／selected1／captured1／hudEpoch1／currentBar1／local1；後續原negative直接Request返回true並執行真退休清理，不能當作GUI成功。四原圖保存，其中1280文字1與400文字2已直接檢視。先在截圖後重畫真視窗hit grid、送實際MouseMove，驗全interactive windows的hit path並記down／capture／up／hover；若第一click失敗立即return，禁止後面的直接呼叫把失敗狀態清掉。未放寬production gate／以直接方法替代點擊，66來源一致，原完整logs保留。

## E1130：離場 fixture 提前建立 HUD，未由 publisher 取得來源 epoch（2026-10-18）

post-match-native-v2 NativeMatchResult成功，PostMatchLeaveOwnership失敗。真診斷result0／viewPlayer0／configured1／selected1／captured0／hudEpoch1／currentBar1／local1；fixture先Ensure建立Live HUD，TryPrepareHudPublish按既有「Live但沒有capture不能補認證」規則拒絕發布，不能在production放寬。改由第一筆公開狀態建立owner與capture，再取得current bar；wrong-player UI negative只改UI view，避免把惡意測試資料送進gameplay owner。66來源一致，原logs保留；NativeMatchResult另補有效late Finished無法復活退休UI的明確斷言。

## E1129：結果測試未保留 Slate root，舊測試實際在退休後更新（2026-10-18）

post-match-native-v1 exact5中3成功，NativeMatchResult與PostMatchLeaveOwnership失敗。前者TakeWidget回傳temporary立即釋放，SObjectWidget destructor呼叫NativeDestruct／RetireGameplayInput；新增正確退休保護後，原測試的後續Defeat／Draw更新被拒絕。改保留const Slate root整個test生命期；foreign widget negative也保留root，確保拒絕原因是非owned bar而非已退休。另一離場資格失敗尚未定因，先補result／retired／player／source epoch／current bar／controller實際診斷，不放寬production gate。66來源一致、完整native／engine logs保留，待fresh驗證。

## E1128：來源審查發現有界結果空容器未跟隨結果收起（2026-10-18）

post-match初版新增外層SBox高度上限，但只收起內層ResultPanel；結果未顯示時外層仍會保留有高度的可命中空容器。此為編譯期間來源審查發現，尚未宣稱真點擊驗證；與E1112同類的empty-host原則。待所有編譯結束後，外層也依結果／模式／退休狀態collapsed，保留真世界transparent-gap既有測試，不用HitTest guard豁免掩蓋。

## E1127：QuitGame 呼叫漏傳 WorldContext（2026-10-18）

post-match-build-v2正式refresh真OtherCompilationError／80.10秒，唯一C2660為QuitGame傳3參數。已讀本機UE5.8 KismetSystemLibrary.h649，真簽名是WorldContextObject、SpecificPlayer、QuitPreference、ignorePlatform四參數；用當前controller同時提供前兩項。66來源在失敗建置期間一致，所有編譯結束後才修正，原完整logs保留。SDK呼叫須先讀真宣告，不靠印象。

## E1126：session interface／subsystem 新增後普通 gate 要求更新引擎產物（2026-10-18）

post-match-build-v1 真FailedDueToEngineChange／32.09秒，尚未完成C++編譯；原build與UBT完整log保留。按既有正式--refresh-engine流程更新UBT列出的必要產物，再跑普通NoEngineChanges／NoUBA／nocache gate。不手改引擎來源、manifest或BuildId，不清cache繞過，未宣稱本批成功。

## E1125：明確換行後名稱與快捷鍵合併仍超出最小可讀區（2026-10-18）

public-name-native-v3仍4成功／1失敗。真rect：400×300 text2名稱高度68，host高度66；明確換行已由三行降兩行，但原資訊分區把英雄名與F2當同一文字段，仍無法完整進clip。400壞圖已依E1124新增流程保存並直接檢視；62來源一致。改單行名稱（長名ellipsis，完整名保留tooltip）、頭像旁獨立真focus chord，保留原字級與host比例；新增hotkey本身clip／字型／F2值與小視窗原圖驗證。不以改66門檻或放寬ContainsRect過關。

## E1124：隊友名稱置中自動換行造成大字級極小視窗三行裁切（2026-10-18）

public-name-native-v2 exact5中4成功，ResponsiveHudLayout真失敗於400×300 text2完整名稱無法進clip。800名稱端原圖直接檢視為「晨光／導師／F2」三行，舊只驗生命值端漏掉名稱端。改名稱slot填滿卡片，停用依前次allotment的AutoWrap，WrapTextAt與卡片WidthOverride共用同一寬度lambda；不縮字、不增加特定resolution豁免。保留原logs／800壞圖／62來源一致；新增名稱rect記錄，將名稱原圖寫出移到assert前，之後失敗亦保留400原圖。完整名稱與生命值兩端clip／font斷言不放寬，待fresh重驗。

## E1123：唯讀名稱測試盤點再次猜測測試 header 位於 Public（2026-10-18）

OmEditor/Public/OmAutomationTestActors.h不存在；rg跨檔查詢該目標失敗，另一已知WorldBridge header仍有回傳。之後先rg --files定位測試header實際Private位置，不按UE目錄慣例猜測；無來源變更。

## E1122：fixture baseline 漏清新增隊伍名單旗標（2026-10-18）

ally-card-owned-v2 真正常／quick／indicator鍵盤事件各單發lumen_aid／catalog25／unit21，dead拒絕零wire；最後retire stage失敗。baseline手動重設各欄位，未重建FixtureWorld，先前誤認其會自然清team_cards_disclosed。修baseline明確清旗標；Rust測試新增ready直接baseline及公開COUNT3消失，避免先經disclose_owner清旗標掩蓋缺陷。三cleanup成功、62來源一致、原workflow=false及38筆wire保留。此為fixture退休缺陷，不放寬production或退休斷言。

## E1121：隊友卡 fixture 把 R 鍵送成滑鼠事件（2026-10-17）

ally-card-owned-v1 真失敗於indicator未送wire。normal／quick雖各看到lumen_aid單位21，但新案例誤用Down／Up（ProcessMouseButton事件）傳R，並非真鍵盤證據，不能接受前兩筆為鍵盤成功。publisher／editor／profile三cleanup成功，62來源一致，原native／workflow／wire／log完整保留。改用既有KeyDown／KeyUp真FKeyEvent；indicator另跨一幀持有並斷言preview再release。hover工具log也改標hover，不假稱click。修fixture不放寬wire、owner或UI guard；fresh重跑三種模式。

## E1120：小視窗隊友測試漏讀 FText setter 簽名（2026-10-17）

ally-wire-build-v2正式refresh在OmResponsiveHudTests.cpp1866真C2664：SetPresentationMatchLine接受FText，新增測試傳TEXT常值。先等待所有編譯子程序結束、驗62來源未變，再用FText::FromString修正；原完整build／UBT logs保留。新增呼叫須讀真header簽名，不以其他相似setter或傳入BuildPlayerMatchSummary推測FString。

## E1119：隊友卡 hit-path header 變更後普通 gate 要求 NetCore 更新（2026-10-17）

ally-wire-build-v1 的Rust bridge成功，UE普通NoEngineChanges gate真失敗FailedDueToEngineChange／55.53秒，UBT列出NetCore及相關modules/version產物，尚未通過C++編譯。build.log與UBT-Log.txt完整保留。沿正式build_ue_moba.lua既有--refresh-engine更新必要產物，再跑普通gate；不改engine來源、manifest、BuildId或清cache繞過。

## E1118：Cargo exact 名稱未帶模組導致零測試（2026-10-17）

補存新增Rust測試明細時，--exact只給函式名而非renderer_fixture::tests完整名稱，真結果0 passed／153 filtered，不能算驗證。affected-test.log保留；改完整名稱並同時斷言exit0與明細1 passed／0 failed／0 ignored，另存affected-test-v2.log。正式fixture三個gate原先已exit0，不能用其不轉印輸出的空log宣稱測試筆數。

## E1117：新公開隊伍 stage 未列入歷史身份分類（2026-10-17）

第二次 Lua 契約測試在check_history真拒絕 owner_support_team_cards：stage allowlist與inventory已追加，但未列入維持source／target身份的分類。兩新stage明確加入keep_identity，仍逐筆驗證source11／target21及相同generation／view floor；不放寬未知stage拒絕。

## E1116：追加隊友卡案例時漏改第二個歷史筆數斷言（2026-10-17）

Lua 契約測試在550行真失敗：新增三個stage後，頂部inventory已改152，但合成history結尾仍硬編149。保留完整prefix與每筆順序驗證，將該精確數字同步152後重跑；不得移除數量與published sequence斷言來過關。

## E1115：唯讀查詢猜測不存在的 Rust fixture runner（2026-10-14）

run-owned-fixture-rust-gate.lua 不存在。之後先以 rg --files 列出實際 Lua入口，或直接讀正式 moba_renderer_fixture.lua 的producer；不從功能名稱拼接工具路徑。唯讀失敗未修改工作流。

## E1114：名牌 header 變更後普通 NoEngineChanges gate 要求更新 NetCore（2026-10-14）

names-build-v1 trueexit1，UBT FailedDueToEngineChange 7.56秒；列出既有 UnrealEditor-NetCore.dll／pdb／obj／dep／sarif／lib 及相關 modules/version 產物。無C++編譯成功，原完整log保留。依已有 build_ue_moba.lua 的 --refresh-engine 正式流程更新必要引擎產物，再跑普通 NoEngineChanges／NoUBA／nocache gate；不手改引擎來源、manifest、BuildId、config或清cache繞過。

## E1113：唯讀診斷猜測不存在的 engine workflow module（2026-10-14）

ue_engine_build.lua 不存在；應直接讀已定位的 build_ue_moba.lua 與其實際require，不從用途推測 module名稱。此唯讀失敗未改任何工作流；後續使用現有明確 --refresh-engine producer。

## E1112：新增空隊友捲動區破壞 transparent HUD gap 契約（2026-10-14）

integrated-native-v5 中 WorldBridgePresentationScale／HudOwnerEpochAndIsolation／PlayableMvpHud成功，ResponsiveHudLayout真失敗：800×600 blank gap 被當interactive。空 roster 的 ScrollBox仍有desired geometry，host又stretch整個header。改無隊友時整個host collapsed，非空時只量測內容寬、以visible host clipping約束四張卡的hit判斷。保留原blank gap斷言，不新增豁免；52來源SHA及原log保留。清理warning已消失，legacy實際widget資料定位HeroOverlay父Border未collapse。

## E1111：整合原生通過但目視仍有中央 Hero 占位卡（2026-10-14）

integrated-native-v4 exact2成功、52來源SHA一致，真 Slate隊友卡已把同一camera移到公開隊友位置；three_lane_training三張game原圖已直接檢視。中央仍有不隨鏡頭的Hero紅條，不能把原生成功當畫面完成。增加實際HUD widget tree診斷定位來源，維持原production角色分類與單一路徑，不直接把整個root隱藏遮掉可能的TD操作。

## E1110：唯讀查詢再次猜測 Blueprint UI generator 檔名（2026-10-14）

OmBlueprintUiGenerator.cpp 不存在；應先列 OmEditor 真檔案或讀實際活 widget tree。未修改來源，後續以 GetHudRoot 的真 widget 資料定位占位卡。

## E1109：唯讀查詢猜測 EntityOverlayWidget 檔名（2026-10-14）

OmEntityOverlayWidget.cpp 不存在；實際呈現包含 OmUnitActor 的 Nameplate component 與既有HUD root，改查已定位的 owner／unit來源。避免從畫面功能推測類別檔案。

## E1108：隊友點擊測試缺少公開 roster 導致小地圖合法拒絕 focus（2026-10-14）

integrated-native-v3 trueexit1：WorldSurfacePresentation成功，真game viewport已正常俯看、中央115200像素且target=OmRtsCameraPawn，但點卡focus失敗。讀正式 FOmMinimapModel：marker隊伍只來自 Frame.moba_hud公開scoreboard；初fixture沒提供，手補LocalTeamId不會替marker造team。改從同一OmMobaHud公開六人roster產生正式Minimap模型，保留拒絕未知隊伍規則，不讓測試鬆綁production guard。原圖／native log與52來源SHA保留；燈光畫面警告已消失。

## E1107：整合原圖揭露兩個原生 directional lights 同優先級（2026-10-14）

integrated-native-v2 原圖包含 ForwardShadingPriority 衝突長警告；native key 與 fill 原本均預設0，雖 deferred光照存在，forward／translucency只能選一個主光。正式 native fallback key 設1、fill設0，保留既有 authored light 讓位邏輯；新增唯一主光斷言，不關閉畫面警告遮掩問題。

## E1106：推進 World time 後仍是 Controller view target，E1105 假說不足（2026-10-14）

integrated-native-v2 trueexit1，原始 game-viewport-combat.png 已保存並目視：貼地平視、大片黑天，native記錄 target=OmAutomationInputController_0；不是可接受的MOBA畫面。BeginPlay 後必須像既有 owned viewport fixture 一樣明確 SetViewTarget(Camera)，之前僅在 BeginPlay 前設定。改生命期順序並驗真正 target／owner，維持原中央像素門檻，不縮小門檻換通過。52來源SHA一致，兩次原失敗保留。

## E1105：整合 game viewport 首次原生測試沒有可見戰場（2026-10-14）

integrated-native-v1 trueexit1，普通 build-v2 9.84秒成功、52來源SHA一致。既有 SceneCapture 圖成功，但真 game viewport 中央像素不足，不能接受整合完成。初版未推進 World time，PlayerCameraManager 的 frame cache 可能仍保留初始相機；改走真 World Tick 後更新鏡頭，保留實際 scene visibility 斷言。另外原初版在驗像素前未先保存失敗PNG，修正為先存原圖再拒絕，後續每次失敗都可直接目視；不把SceneCapture圖片貼到HUD後方冒充實際viewport。

## E1104：整合視窗首次編譯誤用不存在的 LocalPlayer API（2026-10-14）

integrated-build-v1 ordinary真失敗24秒，ULocalPlayer 沒有 GetSlateUserIndex。已讀實際 LocalPlayer header 與現有 production ResolveSlateUserIndex，改 Slate.GetUserIndexForPlatformUser(Player->GetPlatformUserId())。保留原log，使用已維護路徑，不增加 compatibility fallback。

## E1103：唯讀 rg 再次把 Windows glob 放進檔案路徑（2026-10-14）

OmAutomationTestActors.* 被 Windows 當作非法 literal path。修正為對存在的 Private 目錄查詢並用 -g 'OmAutomationTestActors.*' 過濾，未變更來源。所有 glob 只放 -g，禁止插入路徑參數。

## E1102：無必要的 path library 檔案查詢再次猜測位置（2026-10-14）

唯讀查詢 scripts/lib/path.lua 不存在；既有 bootstrap 的 b.lib('path') 已足夠，刪除這類無需求的檔案查詢。後續直接沿既有 Lua producer 使用，不推測 library path。

## E1101：隊友卡首次 ordinary build 真失敗（2026-10-14）

team-build-v1 trueexit1，92.78秒，MSVC C4458：UWidget 已有 Slot 成員，新方法的同名參數／local 需改 AllyIndex；CompactChordLabel 定義在下方需先宣告；SWrapBox 建構參數是 PreferredSize，setter 才是 SetWrapSize。完整 UBT log、52來源 capture保留。按實際 header 修正，不關閉 warning、不替換 compiler、不抹失敗。

## E1100：唯讀查詢猜測兩個不存在的 runtime 檔名（2026-10-14）

render_snapshot.rs／team_replica.rs 不存在；以 rg --files 定位實際 fog_grid.rs，並從既有 presentation_bridge.rs 的 snapshot_envelope_cached 看公開 event 接線。不得從型別名稱推測檔名。

## E1099：沿用摘要的 renderer fixture 路徑而未查實際清單（2026-10-14）

omfue/bridge/src/renderer_fixture.rs 不存在；根目錄 rg --files 定位到 omoba-client-runtime/src/renderer_fixture.rs。唯讀失敗，未改來源；摘要只提供線索，後續仍先定位檔案。

## E1098：隊友測試 patch 夾帶未修改且括號誤寫的 context（2026-10-14）

整包 patch 因末尾多餘 context 比實際來源多一個右括號而被拒絕，沒有部分落地。刪除無變更 hunk，只使用已讀取的精確修改段；之後加入幾何斷言另讀取實際相鄰來源。

## E1097：同一 patch 重複列出同一目標檔案（2026-10-14）

隊友卡 patch 把 OmCommandBarWidget.h 的 forward declaration 與 member 分成兩個 Update File，工具拒絕整包、未落地修改。已合併成單一依來源順序的操作；同一檔案只列一次。

## E1096：唯讀查詢誤把 editor 測試放在 runtime Tests 目錄（2026-10-14）

查詢 OmResponsiveHudTests.cpp 時使用不存在的 OmRuntime/Private/Tests 路徑；已以 rg --files 定位到 OmEditor/Private，沒有修改來源或測試結果。再次要求按實際檔案清單定位，不從一般專案慣例推測目錄。

## E1095：小地圖整合測試末尾仍保留舊 reset-left 斷言（2026-10-14）

responsive-native-v5 trueexit1，ResponsiveHudLayout／PlayableMvpHud／ShopTransactionStatus三項Success；MinimapPreferences僅末尾Reset side is left一失敗，前面真左右切換、hit／camera／保存及新profile-set reader已通過。同步明確恢復預設為Right，不改既有檔案缺欄位仍Left的migration契約。原四項報告與49來源指紋保留，fresh重驗不抹掉失敗。

## E1094：header 排版 fixture 仍手造已移除的整段除錯狀態（2026-10-14）

responsive-native-v4中settings字級／paint檢查已越過，但Header400x300 text2 selected1 fitted失敗；fixture仍手造Team/Playing/Respawn/Recall unavailable長段落，新產品已使用typed短摘要。改以真正BuildPlayerMatchSummary及明確public100/100/100、9999.9秒狀態驅動；保留長對局、大小字級、左右小地圖、選取卡、actual bounds/no-overlap/fitted斷言。原完整長字串不是新MVP常駐契約，不以縮字／藏選取卡解除失敗。

## E1093：舊小地圖整合測試暗依賴左側預設與單層 profile reader（2026-10-14）

responsive-native-v4真4項，PlayableMvpHud與ShopTransactionStatus成功，MinimapPreferences五失敗。新profile預設Right，fixture把初始Right命名LeftX後再切Right；另仍用FOmPlayerControlProfile讀正式owner已保存的FOmControlProfileSet envelope，導致roundtrip失敗及假預設值。改明確保存Left作比較起點，保存後用正式ProfileSet Load+Resolve驗同一有效profile，保留真Slate右側hit、camera、移動偏好及持久化檢查。49來源前後一致；不是修改產品成左預設或改單層保存格式。

## E1092：唯讀查詢再次猜測 ModalFit 獨立 header（2026-10-14）

rg誤傳不存在 OmHudModalFit.h；FOmHudModalFit實際定義於已include的OmHudViewportSpace.h。改在真Public目錄按型別名稱搜尋，未修改來源。防重複：找符號先搜目錄，不從型別名推測檔名。

## E1091：只改 scroll target 未修好小視窗設定頁（2026-10-14）

responsive-native-v3 trueexit1，PlayableMvpHud與ShopTransactionStatus成功；ResponsiveHudLayout仍在400x300 text2 label paint0，證明E1089只改target不足。正式modal有固定標題、搜尋、長狀態及恢復／關閉footer，字級放大後擠壓可捲動清單；重整為標題＋固定關閉、搜尋、scroll內容及狀態，恢復預設移至同一scroll，減少固定垂直spacing。保留全部字級／actual paint／close／原操作檢查，不用縮字或0paint通過。另發現MinimapPingDeferred仍宣稱已實作訊號不可用，改成當前共用手勢說明並同步真UI測試。48來源前後一致、原三項報告保留，待fresh建置／native。

## E1090：單次契約更新 helper 的 suffix 取代順序不應依賴 pairs（2026-10-14）

Lua helper把Line/CapacityLine/TerminalLine逐一取代，但pairs順序未保證，短Line可能命中長名稱尾端。實際執行本次39項結果逐行確認正確，無CapacityWidget／TerminalWidget，也未編譯；仍改為固定最長名稱優先ipairs，並把39項變更數固定斷言。此為source review捕獲的潛在helper錯誤，不冒稱production或build失敗。

## E1089：字級測試捲動整個技能設定 row，不保證首個 label 已繪出（2026-10-14）

responsive-native-v2 trueexit1，PlayableMvpHud成功；普通尺寸與400x300卡片／資源／冷卻／hit／shop操作檢查已走過，最後settings400x300 text2的Label.CastQ paint0失敗。測試把整個多列BindingRow.CastQ捲入小scroll viewport，預設IntoView保住列尾而首label可能在clip外。改明確捲到實際需要驗font的Label.CastQ，仍要求真正painted font、window/panel/row bounds及所有controls存在；不把0paint當成功、不縮字體。原失敗與47來源驗證保留。

## E1088：新版 tooltip 測試把 concise 冷卻字串當作完整 tooltip 格式（2026-10-14）

responsive-native-v1 trueexit1，PlayableMvpHud成功，ResponsiveHudLayout在800x600 item1 tooltip斷言失敗；舊concise文字2.0s、真正完整tooltip保留2.0/5.0s。拆開兩個明確fixture期待值，繼續驗可見卡片的完整tooltip與hotkey真font；不改產品冷卻資訊或以hidden文字代替可見tooltip。原失敗log及47來源驗證保留。

## E1087：新版 tooltip 測試草稿假定英雄名稱已有 tag（2026-10-14）

調整舊ResponsiveHudLayout草稿使用OmHeroName，但建置前讀來源發現原HeroNameText未設tag；已替正式名稱widget加上穩定tag，保留測試直接驗真正tooltip，不讀其他hidden text代替。此錯誤在source review捕獲，未宣稱native失敗。新增可觀察名稱須先盤點真tag，禁止猜測。

## E1086：把來源名 leafsDark 誤當作暗綠色材質（2026-10-14）

playable-mvp-world-native-v1 exact2成功／0warning，四張出生與combat原圖目視仍拒絕產品接受：外側平面呈亮青色。讀原始tree_pineDefaultA.mtl確認leafsDark Kd為0.1686/0.6510/0.6667，名稱不代表暗綠。改用既有原創grass來源新增world_boundary_v1材質：同一WorldXY貼圖乘固定.18、opaque/default-lit/roughness1/specular0、精確graph與managed signature；保留所有既有packages不覆寫。正式Lua工具驗首次建立、重開0save與非法計畫零寫入，原圖與原成功測試不冒稱美術接受。

## E1085：Ctrl 點技能與既有 HUD 訊號手勢衝突（2026-10-14）

playable-mvp-card-owned-v2 trueexit1，地面R圖示開啟瞄準／世界確認已成功送33rd wire，但升級Ctrl點擊無wire。實讀 HandleCommunicationHudPointer：Ctrl點擊是既有訊號手勢，辨識技能卡後即使訊號未受理也必須consume，不能穿透施法。保留此通用防穿透規則，改技能升級為公開bCanUpgrade控制的明確＋按鈕，走同一RequestAbilityCard／CombatAim升級路徑；既有Ctrl+QWER快捷鍵保留。新案例實際點＋按鈕，不繞過Slate。原失敗native/workflow保存，三cleanup成功。

## E1084：新 HUD 圖示測試只驗 resource pointer，尚未等待實際貼圖編譯完成（2026-10-14）

native-v4 exact2 Success／0warning，但逐圖發現正式道具圖示仍空白，因此不接受有道具畫面。沿用既有 ItemIconPresentation 的合法 TextureCompilingManager FinishCompilation，驗真正paint brush引用、Image drawtype與1254平台尺寸後再paint/capture。只修改明示fixture等待，不在production每frame阻塞編譯；資源指標存在不是GPU圖示可見的充分證據。

## E1083：唯讀搜尋再次猜測 HUD payload header 名稱（2026-10-14）

查短比分欄位誤用不存在 OmHudStatePayloads.h，rg exit1；實際 CommandBarWidget.h 明確 include OmUiEventTypes.h。未修改來源。後續直接循已讀 include，不推測檔名。

## E1082：新技能圖示案例錯認 Vanguard Q 為地面技能（2026-10-14）

playable-mvp-card-owned-v1 trueexit1，原35案例已走完，新 ability-card-ready 等待不存在的 Q Point metadata 超時；正式 Vanguard 地面技能是 R vanguard_resolve，既有案例已明確列出。改用真正 R card、R Quick profile 與 slot3 wire 契約，升級 Q 與 Luminary R self 個別保留。不修改正式技能 shape、不放寬 metadata 等待。原workflow/native/log保留，publisher/editor/profile 三cleanup成功，沒有 authority game或ACK。

## E1081：空背包首次 refresh 的快取命中保留預設灰色 Brush（2026-10-14）

native-v3 六原圖目視發現空槽灰色實心。ItemId None／resource nullptr 與初始快取相等而提前 return，預設 FSlateBrush 尚未改成 NoDrawType。修正快取命中需 texture存在或 brush已正確NoDraw；空槽顯示既有暗底邊框，不造假貼圖。需fresh原圖重驗。

## E1080：未讀 JSON schema 便假定 HUD manifest 使用 assets 欄位（2026-10-14）

唯讀 Select-Object -ExpandProperty assets 報欄位不存在；實際 schema 使用 entries。沒有資料寫入，改先讀原始manifest頂層，再使用正式 entries，不以空結果當無資產。

## E1079：新有道具 HUD fixture 沿用不存在的 training 道具 ID（2026-10-14）

playable-mvp-hud-native-v3 trueexit1，CameraProjectedDrag成功，PlayableMvpHud三個資源斷言失敗；六版面均150logical px且冷卻位置通過，但training_cloth／training_salve不是正式manifest條目。改fixture用moba_armor／moba_guard_charm，保留必須有真資源的斷言，不添加虛構正式道具或放寬測試。六原圖與失敗報告保留。

## E1078：續作唯讀搜尋省略 Plugins 路徑（2026-10-14）

rg 誤傳 omfue/Source/OmRuntime，實際位於 omfue/Plugins/OmRuntime/Source/OmRuntime，exit1。先用rg --files確認三檔實體再讀取，未修改來源。續作仍必須驗真路徑，不把摘要的短檔名當絕對結構。

## E1077：新技能卡 wrapper 混用 wire kind 與 Unreal action enum 名稱（2026-10-14）

實作 RequestAbilityCard 草稿把 wire 的 AbilityCast／AbilityUpgrade 當作 EOmGameplayInputAction 成員。建置前讀實際 header 發現 Unreal 成員是 CastAbility／UpgradeAbility，已修正；尚未執行編譯，不冒稱 build失敗。後續新增 adapter須先讀唯一型別定義，不按 JSON kind 推測 C++ enum。此 wrapper仍消費既有CombatAim.Press／ApplyCastResult，不新增第二套技能邏輯。

## E1076：接受紀錄 patch 附帶逆序空 hunk，整批驗證失敗（2026-10-14）

來源checkpoint helper與接受紀錄同批patch，最後在MVP計畫的P0.1 checkbox後又附上較前面的「進度」空hunk，違反同檔source-order而驗證失敗；三檔皆未寫入，已由實際讀檔確認。移除無內容hunk，再用單一精確checkbox／標題修改。重犯hunk順序操作錯誤，不能假定前面helper已套用或跳過核對。

## E1075：第二次來源核對未使用新 checkpoint，嘗試覆寫既有 summary 遭拒（2026-10-14）

MVP native-v2與owned-viewport-v2都結束後，重跑 sources-v2 verify；40個SHA核對完成，但 helper 隨後嘗試寫已有summary.json，path.write正確拒絕覆寫，exit1。沒有覆寫歷史證據或修改來源。修 helper接受明確的新checkpoint名稱，後續延長驗證用獨立 `summary-after-owned-v2.json`；不得把同一 evidence id 的summary反覆改成最新結果。原第一次來源summary仍保留。

## E1074：移除背包長句後沿用舊 compact 隱藏規則，造成空背包消失（2026-10-14）

`playable-mvp-hud-build-v1` 普通建置成功216.56秒；native-v1 CameraProjectedDrag成功，PlayableMvpHud真失敗18個背包可見範圍斷言。三張原圖完整保存，1280圖已目視：英雄／HP數值／技能列確實重排，但六個空格消失，所以92px高度不可當完整HUD接受。舊 compact 把熱鍵／倒數藏起來，空格 icon box 也依有無texture折疊，先前靠長句維持格子；移除長句後漏處理這個依賴。修為每格永遠保留固定圖示槽與熱鍵，倒數依真狀態顯示，空格不要求假圖示或假道具。新增 native 正向檢查已抓到實際視覺缺口，不刪除斷言；待 fresh build/native 原圖驗證。40來源在原建置／native期間一致，原fail保留。

## E1073：模式 fixture 重設無關戰鬥偏好，觸發真正同步提交（2026-10-14）

`input-scheme-owned-build-v1` 普通建置成功378.03秒；fresh owned-viewport-v1 在完成原 selection／combat17案例後，input-point-ready 真失敗，wire ordinal5 是 acquisition1／idle=false 的 CombatPreferences。input_point 草稿先 baseline 將先前已確認的戰鬥偏好清除，projection fallback變成0／true；正式 owner 因 desired 與 projected 不同而合法重送。修 fixture 三個 mode stage 保存原 combat_preferences，只切換明確 mode／clock，新增 Rust 斷言保持1／false。禁止忽略多出的 wire、關閉正式同步或改預期掩蓋缺口；原失敗log／workflow與36來源一致證據保留，待 fresh gate／owned 重驗。

## E1072：比較鏡頭 cm 時忽略角色也按世界單位縮放（2026-10-14）

E1071 的90000cm來源數值正確，但「2400cm才是正常遊玩距離」判讀不完整。進一步讀正式 AOmUnitActor 註解、ConfigureWorldPresentationScale 與既有 WorldBridgePresentationScale 真斷言，角色高度120 world units在100cm換算下是12000cm，不能用常見180cm角色直接比較。錯誤判讀已向使用者更正；尚未修改相機，避免把鏡頭移到角色體內。P0.4 應以實際角色視覺 bounds／camera projection／螢幕佔比與公開地圖邊界驗證，而不是直接換成2400。E1071保留為原審查紀錄，但不可把其推論當已確認產品缺陷。

## E1071：正式進場鏡頭沿用全景展示尺度，覆蓋正常遊玩 zoom（2026-10-14）

後續更正：數值已確認，產品缺陷推論尚未成立，必須連同 E1072 讀取；不得直接改為2400cm。

使用者再次指出整體介面與可玩性不足後，重新盤點正式 `FocusCameraOnOwnedHero`，確認初次 hero frame 呼叫 `FitPresentationView(Focus, Clamp(900*WorldUnitsToCm,12000,180000))`。預設100cm換算時是90000cm，而相機原本正常 DesiredZoomDistance 為2400cm；這不是玩家的全景操作，卻覆蓋一般進場視野並擴大正常 zoom range。此為來源已確認的產品缺口，舊 compiled-map 全景截圖不能代替正常遊玩視角接受。P0 收斂須將一般 hero startup 與展示 fit 分開，保留現有合法 zoom／profile，按實際 owned camera 與公開地圖 bounds 聚焦，不藉隱藏敵人或寫死全圖尺寸。待目前 owned consumer 結束再修改及 fresh 驗證。

## E1070：工具環境刷新後未先檢查暫存 patch 是否仍存在（2026-10-14）

呼叫 apply_patch(load(...)) 時，工具暫存值不存在，輸入不是字串而遭拒；整個 exec 在任何檔案修改／新 gate 啟動前停止。改用已讀來源的明確字串 patch，後續不可把跨工具環境刷新後的 store 當作持久來源。沒有修改歷史證據或重複啟動流程。

## E1069：新 fixture 僅發布 mode capability，漏掉方向輸入必需的當前 step capability（2026-10-14）

owned-fixture-v1 Rust gate 執行期間唯讀審查發現新 input_keyboard／input_locked 草稿沿用 `direction_step:None`；正式 bridge 將 `step_active` 設為 direction_step 是否存在，故按鍵 Begin 必然被拒絕。這是來源審查缺口，尚未新跑 UE build／owned case，不能冒稱執行失敗或已接通。等 gate 結束後，fixture snapshot 應明確發布 exact replica tick 的中性 step（零方向／press0，非任何要求的確認），並測試 step tick／scheme／零 press 與 baseline 清除。只在 keyboard synthetic stage 啟用，不放寬正式 bridge guard。另 source review 發現新增18案例後需同步公開 pumped-frame budget 算式與 static_assert；一併在建置前修正。

## E1068：Windows rg 搜尋再次把 wildcard 放進路徑（2026-10-14）

盤點 renderer fixture Lua 契約時使用 `scripts/tests/renderer_fixture*` 作路徑，Windows 回傳123，唯讀搜尋未執行；重犯既有路徑模式操作錯誤。改對已存在 `scripts/tests` 使用 `-g '*renderer*fixture*.lua'` 並由 `rg --files` 盤點。沒有改動測試、來源或證據，後續必須以目錄加 glob 過濾，不把 wildcard 放路徑參數。

## E1067：新增 direction 負例預期錯誤訊息忽略既有更早的 queued guard（2026-10-14）

新增 Lua wire 負例時預期 `owner control aliased queued`，實際共用 check_intent 在專用 shape validator 前已拒絕 queued，測試因此失敗；正式拒絕行為本身正確。順便修正該共用訊息硬寫 Communication 的問題，依實際 kind 回報 `Direction carried queued`，負例核對真正先發生的拒絕。其餘原115契約已先成功，待含新增負例 fresh 重跑；沒有接受或隱藏這次失敗。

## E1066：接受紀錄 patch 再次把長行前綴當完整 context（2026-10-14）

記錄 look-ahead 接受範圍時，apply_patch 使用上一筆長段落的前綴作完整行，驗證失敗，文件未變；重犯 E1038／E1053 同類問題。改讀真正獨立標題後插入。這是文件工具操作錯誤，未改 production 或任何接受證據；不得再用截斷長行作 patch context。

## E1065：相機重綁輸入後延遲到 Tick 才註冊 pointer observer，遺失第一個事件（2026-10-14）

`control-lookahead-build-v2` 成功12.83秒；native-v2 真實 hit leaf 已確認為 viewport，仍有四個新斷言失敗，故 E1064 的舊 paint 不是全部原因。進一步讀取正式接線發現 RebindOwnedControlInput 立即 AttachOwnedInput，卻只有後續 RefreshInputOwnership／Tick 才註冊 observer。重綁完成後第一個真 cursor event 因此遺失；同一 fixture 後續再次 cursor event 成功且 free-view Scout 可啟動，吻合此生命期缺口。修正為 AttachOwnedInput 在 PushInputComponent／InputOwnerPC 設定後立即註冊同一 observer；舊 observer 仍先退休，沒有重用舊 sample、沒有合成事件或放寬 guard。待 fresh build/native 驗證，兩輪失敗與35來源SHA均保留。

## E1064：Scout／dynamic 新增 fixture 在移除阻擋 UI 後沒有重新 paint（2026-10-14）

`control-lookahead-build-v1` 普通建置成功454.77秒。`control-lookahead-native-v1` 的 CameraProjectedDrag 出現五個新增 positive 斷言失敗：dynamic 無偏移、Scout 不啟用等；舊 drag 與其他 camera 斷言未報錯。來源審查發現 fixture 移除覆蓋全視窗的 BlockingButton 後，未更新 Slate paint／hit-test grid 就送 cursor；production 的 world-leaf guard 因此仍可能看見舊 UI。先補公開 SlatePrepass／ForceRedrawWindow 並檢查真正 hit leaf，再重跑；此階段為待驗證原因，不放寬正式 guard、不把本輪當接受。原始失敗證據保留。

## E1063：唯讀盤點再次猜測不存在的 Rust 模組檔名（2026-10-14）

動態鏡頭建置期間盤點下一批 fixture，rg 指定不存在的 `omfue/bridge/src/owner_input_scheme.rs`，唯讀搜尋 exit1；沒有改動來源或影響建置。改用目錄與 glob 盤點；bridge 並無此獨立模組，owner scheme 型別／解碼位於 core 的 `input_scheme_transport.rs`。後續直接依已讀來源與 `rg --files` 結果定位，禁止以功能名稱推測檔名。此錯誤與先前 E1056 同類，明確保留重犯紀錄。

## E1062：共用獨立 viewport fixture 綁定玩家後未執行標準 LayoutPlayers（2026-10-14）

修正後build-v4真成功19.65秒、native-v3真28 Success／0warning／exit0，34來源前後一致。新增local Size>0、原Slate world hit、RMB CastQ／非Move、contextual left單次分類、physical release全部成功；支持fixture viewport rect的根因，原production sampler維持嚴格拒絕。

pointer-camera-build-v3普通成功37.93秒；native-v2真process exit0、28中27成功／RuntimePersistence一項三斷言失敗，32來源前後一致。真正Slate probe已確認leaf SViewport、contains own viewport、實際local 457.1×342.9，但mouse仍未分類。審查定位到共用FOmEditorGameViewportFixture::BindPlayer只設ViewportClient，未呼叫Engine LayoutPlayers，獨立fixture沒有遊戲world tick補上local player's有效pixel rect；production sampler正確拒絕無viewport pixel的world hit。修正共用fixture呼叫標準Client.LayoutPlayers，新增正尺寸斷言；不放寬production sampler／pointer guards。待fresh native證明；Rebase／strict reset／camera role本輪成功明示沿用，失敗原報告保留。

## E1061：新設定工具列未完整考慮窄視窗與放大文字（2026-10-14）

來源審查發現長 context 按鈕未換行、兩個還原按鈕使用固定水平 auto-width，400×300／大字體可能超寬。修為一般設定按鈕 label 自動換行、還原工具列使用現有 Slate SWrapBox 的實際allotted size排版；API先讀本機SWrapBox.h確認。待實際capture檢查，不以標準寬度native成功冒充小視窗視覺接受。

## E1060：無 override 時 reset 提早成功而略過 schema 驗證（2026-10-14）

來源審查發現 mode／hero 沒有patch就直接Success，unknown field／row identity未驗；scalar reset也忽略多餘row identity。改為先驗完整schema路徑／known row，再允許合法no-op；scalar附row一律拒絕。owner的interface/audio reset統一導向global，與既有編輯規則一致。新增三類negative regression，待fresh native。

## E1059：全域改鍵未重新區分 keyboard factory seed 與使用者覆寫（2026-10-14）

pointer-camera-build-v2普通成功10.93秒；native-v1真process exit0但28中26成功／2失敗，32來源前後不變，不接受整輪。PreciseWorldCommand.Rebind 的global MoveOnly接已釋放RMB失敗：舊mode sparse seed未隨global變更重建，新繼承RMB與keyboard技能Q衝突。改為從舊factory與舊effective mode取真正使用者diff，再依新global重建同一SeedKeyboard並套回diff；Global Edit／whole Reset／field Reset共用此規則，明確mode自訂衝突仍原子拒絕，不能靜默刪除。另RuntimePersistence三個mouse斷言沒走到命令，先新增真正Slate hit leaf／viewport geometry證據定位，尚未把原因當作已修正；camera role／held release等其他測試成功明示範圍。

## E1058：測試仍執行中就讀取結束才產生的 errors.md（2026-10-14）

read-only rg 指向 native-v1/errors.md 時該檔尚未產生，命令報不存在，與測試失敗無關。後續進度只讀已存在的 native-engine.log；summary／report／errors 必須在真程序退出並經 inventory 或 Test-Path確認後再讀。

## E1057：原生 keyboard mouse 入口未重用完整 pointer 仲裁（2026-10-14）

v1 ordinary 真成功695.10秒／32actions、32來源在build前後一致；UHT153.38秒，冷PCH compiler working set持續增加。依actual response經maintained preflight唯讀2,574,581,760 bytes／26,500ms，大小／mtime不變（非SHA完整性驗證），之後actions完成。完整真退出後才套用上述仲裁／context down去重修正；v1無native接受，接fresh-v2。

pointer-camera-build-v1 執行中審查發現：新的 native mouse adapter 直接呼叫 keyboard chord helper，會略過 communication／既有cast左鍵確認／hero target policy；native keyboard 也漏同一 raw communication／target policy 前置。contextual left 又尚未記錄 physical down，重複 down／原生 mouse IE_Repeat 可能多次分類。已準備改走同一 HandleAimPointerDown／HandleWorldPointerUp 與同一 policy／raw facts，contextual left 記錄同一 physical model、native repeat不重發；待本輪真退出才套用，v1不得當native接受。來源正凍結，未在建置中修改。

## E1056：Rust bridge 根目錄猜錯（2026-10-14）

read-only rg 附帶不存在的 om_bridge/src，實際維護 workspace 是 omfue/bridge/src，已由正式 gate 的 source inventory讀回確認。沒有修改不存在檔案。後續 Rust crate 路徑必須來自manifest／已驗 source list，不以 package 名推資料夾。

## E1055：同檔 header hunks 逆序導致 patch 拒絕（2026-10-14）

camera header 的 held 方法在305行、snapshot在253行，patch先改305再改253，工具向前尋找而拒絕。讀回確認沒有部分修改，再按253→305來源順序成功套用。多hunk必須按來源行序排序。

## E1054：未讀 include 先猜 automation header（2026-10-14）

rg 附帶不存在的 OmActionOwnerTestTypes.h 路徑報錯；既有測試 include 實際為 OmAutomationTestActors.h。讀回include並rg --files確認後使用正確檔案；沒有修改錯誤路徑。禁止憑類名推測檔名。

## E1053：紀錄更新誤用整段文字的局部行當 patch context（2026-10-14）

嘗試加入接受紀錄時以不存在的局部行作錨點被 apply_patch 拒絕，未改任何檔案。已讀回實際標題，以完整既有標題作唯一錨點。避免無效 no-op patch，紀錄更新必須以工具讀回的完整行為準。

## E1052：新增還原列改變原 BindingRow layout tag 語意（2026-10-12）

後續v3 ordinary成功、native-v2仍拒絕：移動tag至內層後DPI斷言通過，但search測試發現內層自身visibility與外層collapsed分離，七項失敗。改為明確分工：BindingRow保持整列filter／geometry語意；新增BindingColumns標記真正水平／堆疊欄，DPI測試直接量該實際layout type，原搜尋與hit assertions不放寬。28SHA本輪前後一致，整輪仍不接受，待fresh rerun。

editor-build-v2普通成功13.88秒，native-v1真 process exit0但19中18成功、RuntimePersistence一項失敗：既有DPI column量測依 BindingRow 的真正layout child判斷水平／堆疊，新增按鍵還原 wrapper 使該tag永遠讀到vertical。新增context／Shift capture／mode row reset斷言均未失敗。修正production tree：BindingRow仍標記真正按鍵layout，還原工具列由外層共同容器包住，search filter控制外層，維持語意與共同顯隱。不放寬斷言。28SHA build/native前後一致，但原native失敗不接受整輪，原logs保留。

## E1051：重複量測常數當 patch 錨點，UI 被插入 const layout 函式（2026-10-12）

control-profile-editor-build-v1 真 exit1／UBT exit6，99.05秒：新增 context UI 錯插入 const binding width measure，而非 RefreshRows，造成 FontAttr 未宣告／const mutation 等連鎖錯誤；另 ResetButtons loop 的 Slot 隱藏 UWidget 成員，C4458 視為錯誤。已將完整區塊移至 RefreshRows 的 SectionFontAttr＋ContentWidth 多行唯一錨點，loop 改 ResetSlot。後續 source patch 必須同時驗證函式邊界與唯一上下文，不只依重複的單行常數。失敗 evidence 保留，不作 native接受。

## E1050：keyboard mode 逐列還原會直接繼承衝突的 point 按鍵（2026-10-12）

接線實際設定 UI 時，來源審查發現 mode1 的 ResetField 只刪 sparse override；還原 CastW 便繼承全域 W，與必要的 MoveForward W 衝突，完整 validator 會拒絕。修正為 mode1 依同一 SeedKeyboard 與目前 global 重建 factory mapping，逐列原子還原 press/release；hero 層仍繼承使用者的 mode。scalar 同樣使用 mode factory override，沒有預設覆寫則繼承 global。新增實際 Slate 還原成功與 Shift primary 斷言，待 fresh build/native，不以 click handled 當保存成功。

## E1049：已知測試檔仍誤用 Runtime 目錄（2026-10-12）

兩次讀取 ControlProfileSetTests.cpp 誤指 OmRuntime/Private；已盤點的正確位置是 OmEditor/Private。未修改或執行錯誤檔案。後續直接使用已驗證完整路徑，禁止沿用上一個 production 檔案的目錄拼接測試檔名。

## E1048：Slate 接手 keyboard chord 後，held consumer 仍查 UPlayerInput 狀態（2026-10-12）

control-chord-owner-build-v1 進行中的來源審查發現：keyboard 手動呼叫既有 owner bindings 並消耗 Slate event 後，Scoreboard held 與 keyboard world-repeat 若仍讀 Controller.IsInputKeyDown，就會錯認已放開。這版尚未 native接受。修正方向：同一 physical model 提供唯讀 key-down facts，keyboard held consumer依它退休，point-mode仍使用原UPlayerInput；world modifier仲裁使用同一實際事件的side modifiers與已解析chord，不從已被消耗的modifier事件反查舊狀態。等build真退出後才套用並fresh build/native，不建立第二份權威／按鍵狀態。

## E1047：local direction retirement 把 Recall 的 legacy queue flag 誤當延後命令（2026-10-12）

owner增量18原生成功後來源審查發現：bridge InputRecall 直接轉成沒有 queued 欄位的 Recall protobuf，但 local delegate 先對所有 bQueued=true return，可能持續送已退休press的舊heartbeat。不是新權威對局失敗。收斂成受測的 RetiresForCommand：Recall一律退休，真正queued移動仍保留方向，AttackTarget仍保留攻擊／方向；owner身分檢查維持。加三類命令與Recall legacy旗標原生回歸，待 fresh build/native接受。

## E1046：逐列 reset 未保留 held action 的 press/release 原子性（2026-10-12）

control-profile-stage-native-v2 真 process exit0，但 exact14 中 ControlProfileSet 兩斷言失敗，workflow正確拒絕。CastQ 的 TrySetChord 同時覆寫 press/release；ResetField 只刪 pressed override，留下 F10 release，因此正式 flat validator 拒絕不一致配對。修為由 catalog bHeldPair 決定，對同 action/scope/slot 一起移除兩觸發列，其他槽位與英雄不動；加 release繼承斷言。原report/log及23來源不變證據保留，不放寬 validator，也不把exit0當成功。

## E1045：負例 fixture 把同一 TArray 的元素直接 Add 回自身（2026-10-12）

control-profile-stage-build-v2 真成功11.11秒，但 native-v1 在 ControlProfileSet duplicate-row fixture `Rows.Add(Rows[0])` 觸發 UE Array.h2196 self-alias assert，真 exit3；原 log／crash stack／失敗報告保留，整批不接受。前六項既有 profile 測試成功不當整批成功。先 copy shared pointer 到獨立 local 再 Add，另重用 JSON output 前 Reset，避免錯誤 JSON 讓負例以錯誤理由通過。23來源在 build/native 期間 SHA 不變已獨立驗證；修正後使用 fresh普通build/native。

## E1044：UE5.8 JSON object key 是 SharedString，不能隱式轉 FString（2026-10-12）

control-profile-stage-build-v1 真失敗 C2664／C2088，223.27秒；完整 build.log／UBT-Log／summary 保留。JSON Values 的 key 是 UE shared-string storage，不能直接傳到 const FString helper。按維護中 profile parser 的正式寫法使用 `FString(*Pair.Key)` 建立具名字串；不啟用 UE_JSONOBJECT_LEGACY_STRING_KEYS、不改 Engine。先修正，再 fresh普通建置。v1未執行 native，不作接受證據。

## E1043：profile-set 的 FString equality 會摺疊英雄 ID 大小寫（2026-10-12）

增量 build-v1 進行中唯讀審查，實際 UE `UnrealString.h.inl` 證實 `UEOpEquals` 使用 IgnoreCase。新 profile-set 的英雄查找、字串比較與 Seen FString set 因而不能當作 case-sensitive identity。此版尚未 native 接受；必須等 build 真退出後，改用明確 CaseSensitive equality、嚴格欄位名比對與 case-sensitive duplicate 檢查，加入同名大小寫英雄隔離／schema alias負例，再 fresh build/native。不要以 FString 取代 FName 就推定精確內容 ID 安全。

## E1042：唯讀查詢猜測 Engine Containers/String.h（2026-10-12）

猜測標頭名稱回 file-not-found；使用 `rg --files` 對已存在 Containers 目錄盤點，找到真正 UnrealString.h.inl，讀到實際 equality 實作。禁止猜測檔名，含第三方／Engine 來源也須先盤點。

## E1041：Windows rg 路徑參數誤用 shell glob（2026-10-12）

唯讀搜尋把 `OmControlProfile*` 當檔案路徑傳給 rg，Windows 回 os error123。改為已存在目錄加 `-g '*ControlProfile*Tests.cpp'`，搜尋成功；沒有修改來源或測試證據。Windows 搜尋一律使用 rg 自身 glob filter。

## E1040：文件 patch 再次使用未讀取的標題 context（2026-10-12）

方向模型 production patch 已成功，但後續獨立文件 patch 用單獨 `#` 猜測標題，被驗證拒絕，文件未改。重新讀取實際完整標題後才修改。禁止以猜測或截斷標題作 patch context；production 與證據文件分開提交工具操作。

## E988：出生點近照用固定半徑，基地屋頂被畫面裁切（2026-10-12）

compiled-map-native-v5 真1Success零warning，Blueprint-v2真69成功；首四張training圖已直接檢視，建築與idle已正確出現，但spawn.png基地屋頂碰到上界並被裁切。此輪不接受近照視覺，其他十張不冒稱已看。原報告與圖片保留。近照不應只取固定zoom：將同一個actual-projection八角有界fit套用所有shot；每個近照bounds由公開座標半徑地面區域與區域內正式actor的真body bounds合併。保持正式相機有效component、公開座標／比例與原材質，不移基地／不後製。區域外其他物件可自然出界，區域內主體必須完整投影。

## E987：native pass 但新圖有未編譯建築與 T-pose，圖片拒絕（2026-10-12）

compiled-map-native-v4 真1Success零warning，七張three_lane_training原圖已直接檢視（其餘七張不冒稱看過），兩張全景完整入鏡但建築只有環圈，骨架角色T-pose。v4不接受視覺範圍；原圖、summary與Blueprint-v1成功報告均保留。原因：獨立新world capture沒有沿既有WorldBridge capture的 FAssetCompilingManager::FinishAllCompilation／SendAllEndOfFrameUpdates／render fence；此外合成Frame漏了動畫持續狀態，非正式真對局資料。補完整資產編譯與actual scene proxy斷言、合成stand/play_rate1動畫records及真native idle asset斷言，再以序列更新與0.25秒動畫取樣渲染；不改正式模型、比例、材質或authority狀態。新fresh capture與Blueprint最後重驗，不能沿用v4圖當成功。

## E986：全景取第一個包圍球距離，雖 fit 卻讓地圖僅占14.4%畫面（2026-10-12）

compiled-map-native-v3 仍真 Fail，現在原始overview.png已直接檢視：並非黑材質；zoom386914.6、113370/786432可見像素，地圖過小且大量黑邊。保留原圖和report/audit。修有界排版求解：16次膨脹找通過上界後，再12次二分，以實際正式camera component／FOV／1024:768投影檢查所有scene bounds八角，求最接近且保留90%內邊界的完整視野；不換假座標、不改單位比例、不後製圖片、不放寬Visible>25%門檻。近距離固定視點沿原公開座標。

## E985：場景可見像素檢查失敗前沒有保存拒絕原圖（2026-10-12）

compiled-map-native-v2 真 Fail：相機ownership／effective arm與bounds fit已通過，但第一張overview未達Visible>25%門檻。此時尚不能判定材質黑圖或過大視野；原log/report/audit保留。原碼在門檻前return，沒有保存GPU原圖／可見pixel數，妨礙正確診斷。改先保存原圖和實際相機／數量資訊再判門檻，保持門檻未放寬；新run使用fresh目錄，失敗也保留圖片，不算接受。

## E984：renderer fixture 未初始化 PlayerCameraManager，SetViewTarget 沒有接上實際 camera（2026-10-12）

compiled-map-native-v1 真 Fail（process exit0不當成功），actual scene bounds fit 在16有界候選後仍失敗，原log/report/audit保留，尚無接受新圖。比對既有有效相機 fixture，缺少 PlayerCameraManager spawn/InitializeFor，APlayerController::SetViewTarget 不能設定有效view target，正式camera ownership因此拒絕Tick。補齊真controller相機管理器、斷言實際view target與每次effective arm length；不直接覆寫SpringArm或繞過正式ownership。另補每個合成公開單位必須有正式可見actor，避免影像只驗草地而漏角色。

## E982：C++ local Id 遮蔽 FMap::Id，被嚴格編譯設定拒絕（2026-10-12）

compiled-map-build-v1 實際失敗 C4458，來源 OmCompiledMapSceneCapture.cpp 第100行；完整失敗 build.log/UBT-Log/summary 保留。將 terrain 區域名改 TerrainId，避免與 map identity 同名。普通建置重跑，不改 Engine 或警告設定。

## E983：靜態檢視發現 renderer fixture 以 camp 陣列奇偶推測模型（2026-10-12）

尚未執行新場景時發現 fixture 使用 I%2 選 north/south，是依現有陣列順序工作的暫時解。改讀正式匯出 camp id，經 generated OmRegistry::FindContentEntryById(Creep,id) 解析 NumericId 並拒絕未知／tombstone。座標與 identity 都由 compiled map 提供，不抄模型 ID、順序或地圖座標。

## E981：測試 actor header 目錄猜錯（2026-10-12）

唯讀查 OmAutomationTestActors.h 時誤猜 OmEditor/Public，實際在 Private。此檔是測試內部 actor，應先 rg --files 定位而非套用 runtime 公開 header 規則；未改來源。已定位並讀取 Private 檔。

## E980：rg 搜尋又將 Windows 路徑 wildcard 當檔名（2026-10-12）

唯讀查閱 Lua env 用法時傳 `.ai-collab/*map*.lua`，rg 在 Windows 不展開此路徑而失敗。未修改來源。應傳實際目錄與 `-g '*.lua'` 過濾；本輪已改正。避免因只讀操作而漏記重複錯誤。

## E978：接續摘要中的模組短路徑不可當實際檔案路徑（2026-10-12）

讀取 OmCommandBarWidget.cpp 時誤用 omfue/Source/OmRuntime，實際檔案在 omfue/Plugins/OmRuntime/Source/OmRuntime。唯讀失敗，未改來源。先 rg --files 定位再讀；摘要省略路徑只是模組描述。

## E979：Lua find 多回傳值被 sub 意外當作結尾位置（2026-10-12）

一次性排版編輯器以 sub(assert(find(...))) 擷取 MatchStrip，assert 保留 find 的 start/end 兩個結果，造成只截取搜尋字串本身。後續檢查即失敗，來源尚未寫入。改以額外括號強制單一結果；其他字串切片同樣檢視。保留失敗執行，不當作編譯失敗。

## E977：scope recorder沿用selection單筆wire假設，未讀full的twice規則（2026-10-12）

full-v4真409/179/139與cleanup及public checker成功、Blueprint-v3真69成功後，record-unit-selection-accepted-v2.lua第30行拒絕；尚未建立accepted-v2目錄。此為摘要錄製器錯誤，原workflow/native/JSON不改。原因從12-case recorder轉full時只改總數，忘記expected_cases第三欄twice（完整public checker原本已嚴格處理）。改依原spec[3]計算2或1，保留逐項ordered名稱與179總數驗證。重跑只讀相同真成功證據，不冒稱新的UE執行。

## E976：唯讀搜尋混用Lua pattern與rg regex跳脫（2026-10-12）

盤點驗收record器時將expected%[混入rg regex，造成unclosed character class；該次搜尋未取得結果，沒有寫入或執行驗收。已改rg -F -e逐個精確字串搜尋，避免跨語言跳脫混用。full-v4原workflow仍為success，不能把搜尋命令錯誤混成產品測試失敗。

## E975：400寬選取原圖的頂部對局文字與英雄區重疊（2026-10-12，待修）

直接檢視 selection-owned-v12/selection-enemy-400.png 發現英雄區上緣約y32，頂部Team/KDA/Playing文字也在該高度，部分字被英雄區覆蓋。目標名稱／HP本身可見且viewport fit為true，但fit並未證明各固定overlay彼此不重疊，不能當作完整HUD畫質接受。原圖保留。此時full-v4正驗共用世界取點，20來源維持凍結；待其terminal後接一般版面量測修復，應將實際可見頂部狀態／按鈕區納入預算與交疊斷言，不隱藏資訊或以整組縮小文字掩蓋。父項4.7及完整HUD驗收保持pending；單位選取功能範圍也不宣稱解決此問題。

## E974：第二條指定單位取點仍假定中央可點（2026-10-12）

full-v3真通過原失敗item2-normal-point並發布合法point ItemUse，但在item3-normal-unit失敗；workflow=false與三項成功cleanup保留。原因是AimPaintedUnit仍直接Project中央，E972第一次抽共用時只覆蓋Aim與selection400，漏盤點所有Project呼叫。已逐項列出全部Project入口：內部兩個primitive、minimap投影獨立保留、self選取及指定單位均改用共用ProjectVisibleGround；正式SampleOwnedCastPointer繼續檢查公開unit並加精確generation，無放寬准入。補指定單位的project/sample/identity診斷。待普通build及fresh full409，不能將native-v7十四成功當full通過。以後抽共用必須先完整盤點原primitive所有呼叫，避免只修第一個失敗案例。

## E973：把 PowerShell ErrorAction 參數傳給外部 rg（2026-10-12）

唯讀檢查命令在 rg 後加 -ErrorAction SilentlyContinue，被 rg 當作 -E 編碼參數而拒絕，未取得該次診斷；不能據此判斷建置成功或失敗。外部程式不使用 PowerShell cmdlet 參數；可選路徑先 Test-Path，再用實際存在的檔案或已有summary讀取。建置session保持執行，待真terminal。

## E972：fresh full409 的道具地點施法把目標投到被 HUD 蓋住的中央（2026-10-12）

unit-selection-owned-full-v2 真正失敗於 item2-normal-point，原 log/workflow 及三項成功cleanup保留。既有Aim helper固定把地板candidate放畫面中心，Project的正式pointer qualifier拒絕該HUD覆蓋像素，八候選全 project failure；不能以移走HUD或放寬命中守衛解決。抽出 selection-v12已驗過的有界實際world-pixel／deprojection／camera平移為 ProjectVisibleGround，共用於Aim與400選取；仍重新project、正式pointer gate與Sample、精確generation，施法距離未改。待普通build／native／fresh full409驗證。v2已真通過E968先前失敗的sequence29，但E968成因尚未確定，僅記fresh未重現，不能稱已修復。

## E971：公開目標生命值經 RoundToInt 可能超出32位顯示範圍（2026-10-12）

來源審查發現選取模型只要求finite正值，正式Fixed64生命值範圍可大於int32，UI卻窄化整數。改直接以FText浮點數格式化零小數及正值half-up，不改公開數值或任意限制HP。ResponsiveHudLayout透過實際Slate文字驗4294967296/8589934592及67.5→68、清除Collapsed；沒有只測複製公式。尚待build/native成功才能接受。

## E970：來源盤點再次假設 scripts/ue_codegen 目錄存在（2026-10-12）

唯讀 rg 指定不存在的目錄而回報錯誤，無檔案修改。已改先 rg --files scripts 定位，再只搜尋實際存在的 OmRuntime/Source 與 bridge/src；地圖編譯來源實際為 omoba-template-ids/build.rs。E969已規定先列路徑，本次未遵守，後續不把熟悉的工具名稱當作實際目錄。

## E969：診斷時猜測 UE CachedReadPlatformFile.cpp 路徑（2026-10-12）

唯讀搜尋輸入不存在的 Engine 檔名，rg 回報找不到檔案；沒有寫入 Engine。已先以 rg --files 定位實際 FileManagerGeneric.cpp 與 FileHelper.cpp 再讀取。後續一律先列實際來源路徑，不依舊版本記憶猜測。

## E968：fresh full409 在 q-epoch-cancels 的控制回覆等待失敗（2026-10-12）

unit-selection-owned-full-v1 真正失敗，workflow.success=false；publisher/editor/profile 三項清理成功。最後 control.json 為 sequence29/disclosure_epoch，磁碟最終 control.ack.json 也是 accepted29，published31；history 只收到28。UE 在300個既定等待frame後回報未到29，原錯誤未記觀察到的sequence/stage，因此目前不能判斷為file cache、producer遲延或fixture狀態問題。原始log/JSON/時間戳保留，沒有接受或放寬時間/身分檢查。補充逾時診斷的observed sequence/stage/run/raw，再普通建置與fresh重跑定位。native-v6真14 Success零warning、selection-v12真12/2/1不受影響，但不能拿它們替代full409成功。

## E967：新增目標tooltip前檢查到文字仍HitTestInvisible（2026-10-12）

新版目標標題／kind/team的12-case owned-v11與14 native-v5皆成功，兩HUD原圖已直接檢視；但來源審查指出文字沿用舊 HitTestInvisible，新增 tooltip 將無法由使用者hover觸發，不能以非空字串當可用。將有效目標文字改Visible，範圍仍位於既有HUD panel內，不改空白HUD世界穿透與正式pointer guard；失效仍Collapsed清資訊。800／400真owned案例另要求文字可hit-test、tooltip非空且含公開kind/team，防止同錯復發。原v6凍結及已通過證據保留，另開v7驗修正。

## E966：新增full case後 pumped-frame總帳static_assert未同步（2026-10-12）

unit-selection-build-v17 ordinary nocache真失敗170.25秒，OmOwnedViewportIpcTests.cpp:144 C2338。已新增12個case，但舊總hang guard仍硬驗45377。按原公式增加12×(90 absence＋5 transition)及與其他UIcase相同的12×32 hit／resize slack，得到46901；600／300／180／90的個別ready／ACK／intent／quiet失敗上限不改，也不掩蓋missing wire或重試失敗。原validation-v5與build17 logs保留，另開v6凍結並重新普通建置，沒有EngineChange因此不refresh。

## E965：full inventory 擴充的 patch 與合成 published-sequence 帳本（2026-10-12）

接入12選取case時，一個native patch以半行Names為anchor失敗、全組未套用，已讀實際整行重新窄patch。Lua契約先抓到舊138 stage數未全部更新，再抓到新baseline的 published_sequence少算最後communication_retire之後獨立發布的CommunicationPresentation。新full是409cases／179intents／139stages，原397-prefix不變；最後baseline要計入完整15個先前communication envelope（之前終態只有14個preceding，因最後第15個在該ack後發出）。只修合成帳本／inventory，正式checker仍逐筆嚴格檢查兩種clock，communication-only83／49／15不變。未跑fresh UE full，不能當通過。

## E964：fixture 的 Slate 自繪標題列侵占獨立 viewport，400 fit 不能代表真 client 幾何（2026-10-12）

unit-selection-owned-v9 前十case成功；400失敗診斷 project=9／其餘0、target=(240000,0)、投影=(279.433,81.209)、arranged viewport=389.5×233.5、HUD fit=1。Primary直接看原 selection-400-unavailable.png：自繪標題列與邊框佔400×300畫面內部，HUD及下方經濟列被截。讀 UE5.8 SWindow／FSceneViewport／GameEngine 原始碼證實 SetViewportSize 設獨立window paint尺寸，fixture預設Slate chrome又扣掉title／border；正式GameEngine用UseOSWindowBorder及AdjustInitialSizeAndPositionForDPIScale(false)。fixture改用同一正式OS client政策，不改Engine／DPI config／HUD fit門檻。選點不再猜九個固定focus，改有限掃實際arranged viewport未被HUD覆蓋的像素，正式deproject後平移同一camera使該點對準同一公開敵人，再嚴格重驗world／hostile／inspect。validation-v3原凍結及失敗診斷保留，另開v4。這次屬共用owned fixture修正，原397 prefix需在最後完整驗收重新跑，不能沿用舊圖當新幾何通過。

## E963：400 選點仍失敗，先保留實際幾何／GPU診斷再決定（2026-10-12）

unit-selection-owned-v8 前十case再次成功，400九方向helper仍失敗。前輪只有 unavailable，無法區分投影、HUD遮擋、sample與公開inspect。補每類候選失敗計數、目標／camera／投影／viewport實測與當時HUD fit，終態保存原始 selection-400-unavailable.png；沿用既有Aim的 SetFollowOwnedHero(false) 以明確控制fixture鏡頭。未放寬hit-test／resize／半徑／quiet或重試上限。validation-v2原來源凍結保留，後續另開v3；不能把這輪或當前400操作宣稱通過。

## E962：400 viewport 中心是 HUD，不能用中心 AimEnemy 尋找世界目標（2026-10-12）

unit-selection-owned-v7 前十case成功，真 AttackTarget／AttackMove各一筆且quiet window通過；selection-400 等待中心點目標失敗。原 AimEnemy 將待測地點放鏡頭中心，在400×300時中心可被正式compact HUD覆蓋。不能放寬hit-test或移除HUD。新helper有限嘗試九個鏡頭focus方向，投影的是同一個敵人實際公開位置，必須同時通過真arranged viewport／WorldLeaf／正式hostile sample／公開inspect精確世代才接受；400再送真正左鍵，不只拍既有狀態。原v7失敗與cleanup、validation-v1來源凍結保留。新的驗證另存validation-v2，不覆寫。

## E961：fixture 在 UE 分派 A 的 IE_Pressed 前就送出左鍵（2026-10-12）

unit-selection-owned-v6 前九case通過、真 AttackTarget21 僅一筆且quiet window成功；selection-armed-a 無指令。新fixture把KeyDown／Up A與滑鼠左鍵放同一步，A的InputComponent callback尚未在game tick分派，指標先走普通地板清選。修正為真事件A之後有限等待正式 IsAttackMoveAimActive，再送真左鍵；不直接呼叫arming helper，不改生產順序或延長quiet window。原失敗及cleanup保留。另新增原始GPU focus capture與gold像素／差異檢查，驗選取外圈及hover輪廓確實渲染。

## E960：selection fixture 收到正確單次攻擊後未推進 quiet-window baseline（2026-10-12）

unit-selection-owned-v5 前八case成功，selection-attack 誤報 extra wire。原 intents.jsonl 真正只有一筆 AttackTarget21，PollExpect(Attack) 已驗唯一線與身分，但新fixture進 quiet window 前未更新 WatchLines，將同一筆當新增。修正為已嚴格驗證的 CaseStartLines+1，不吞額外事件；quiet window 仍拒絕任何第二筆。另補兩個focus在10002 row輸入下優先保留與4098總instance硬上限／static零transform更新斷言；不是萬人效能宣稱。原失敗與清理保留。

## E959：鍵盤選取繞過正式 pointer tracker 並重驗已放開的鍵（2026-10-12）

unit-selection-owned-v4 前五case成功（含地板清選），selection-keyboard 真失敗。新 handler 用 Slate 全域 GetCursorPos，而既有玩家命令使用 OwnedPointerTracker 的最新同user事件位置；又在已排入 IE_Pressed callback 後要求 IsInputKeyDown，快速按放同frame可被錯誤拒絕。修正鍵盤選取使用現有 BindModifiedPress 捕捉 chord／MatchesExactModifiers，callback 不依賴此後的 held state；keyboard／hover 都復用 ReadOwnerPointerPosition＋正式 SampleOwnerPointer。保留錯誤Slate user／UI／focus guard，不全域讀游標或放寬modifier。原v4 failed與cleanup保留；重新普通建置後再驗真K／舊左鍵／A／右鍵。

## E958：沿用固定 AimGroundAway 候選仍無法證明可選 footprint 外地板（2026-10-12）

unit-selection-owned-v3 在 selection-ground 有限重試後真失敗；前四case成功，原報告與清理保留。E957新增 inspect 檢查是正確的，但仍沿用固定2400／3600候選，未隨正式模型公開半徑調整。改為依來源及目標實際 SelectionBodyRadius 尺度產生四方向候選，逐一以正式 sample 的世界點再做公開 inspect query；同時補具體point／兩半徑診斷，不能再只寫 unavailable。其他維持正式 admission／半徑。若仍失敗，以實測值判斷，不任意擴大重試或縮小半徑。

## E957：既有非敵對 AimGround 不保證沒有友方／自身 footprint（2026-10-12）

unit-selection-owned-v2 已通過 ready／enemy／Esc／self，selection-ground 仍選自身11。這是 fixture 選點錯誤：原 AimGround 僅要求攻擊查詢無合法敵人，900 cm 等候選可仍在自己的公開半徑內。通用選取支援自身／友軍，不能為測試排除它們。新地板選點離開來源實際位置，並由同一公開 inspect query 確認沒有任何可選單位；未滿足就有限重試，不改正式取點半徑。400 capture 另等待 resize 後重新把鏡頭對準選取敵人，不拿離鏡頭的地板當選取視覺證據。原 v2 failed／完整清理與800診斷圖保留。

## E956：Esc 清選已實作但未加入動態 Escape binding 生命週期（2026-10-12）

unit-selection-owned-v1 真視窗／TCP fixture 在 selection-escape 失敗，entity21仍被選取；ready／enemy已完成。原 workflow success=false、publisher／editor／profile cleanup 都成功，800原圖保留而非接受。根因正式 Escape binding 原本只在施法／A／shop／scoreboard／history 活躍時安裝，新選取僅在 handler 末尾清除，普通情境無綁定。通用修正把選取狀態加入同一動態仲裁，選取成功立即重算，資料退休時每輪 reconcile 回收；維持原既有取消優先序，不另做全域Esc攔截。owned測試另加入實際viewport fit斷言，不以截圖存在當排版通過。

## E955：selection artifact 負例測試猜錯錯誤文字與過廣診斷搜尋（2026-10-12）

新增自足 selection bundle 負例時，checker 已正確拒絕偽造 authority_ack，但測試預期字串 authority_ack，實際錯誤是 intent claimed authority。只修正負例 needle，不改 checker 或放寬 authority。診斷亦曾過廣搜尋全部 .ai-collab，已中止該唯讀 rg，改直接讀 Lua workflow 宣告的 OwnedViewportIpc 證據位置；不存在的報告不當失敗或成功，先等 producer 建置終態。後續依 rg --files 定位來源，避免猜目錄。

## E954：空白 catalog 名稱覆蓋資訊 fallback／模型 HUD fixture 被正式 owner 生命週期退休（2026-10-12）

unit-selection-native-v2 完整跑完14項，13 Success、PublicUnitSelection 兩assert失敗。native creep catalog 允許 DisplayName 空字串，選取程式把既有「單位」fallback覆蓋成空白；改為只接受非空 authored display name。另 EditorPreview 的 Widget 沒有合法 local owner，TakeWidget 觸發正式 HUD 生命週期後被退休，不能假定它是可操作的玩家 HUD。模型測試保留 Initialize 與資料設定／退休斷言，不觸發 Slate owner；真建立、繪製與點擊由 selection owned viewport phase 負責，不放寬正式退休規則。原失敗保留，需 fresh 普通建置／原14項／真owned phase 成功。

## E953：舊設定測試把互斥指標情境與可選綁定判斷錯誤（2026-10-12）

unit-selection-native-v1 崩潰前已有兩筆真失敗：ControlProfileDefaults 把 SelectUnit／PointerConfirm 相同左鍵視為同時衝突；ControlRuntimePersistence 假設全部 optional row 預設必須未綁定。前者只豁免這一組明確互斥的指標情境，其餘碰撞仍失敗；後者對實際未綁定列驗證未綁定文字，有預設鍵的可選動作照常驗證鍵名。保留原失敗，不宣稱整輪成功；unit-selection-build-v5 已成功但尚未含這兩筆測試修正，重新普通建置後再跑原14項。

## E952：選取 native fixture 未初始化 UUserWidget（2026-10-12）

unit-selection-native-v1 Editor exit 3，OmUnitTeamMarkerTests.cpp:266 直接 NewObject 後 TakeWidget，UUserWidget::OnWidgetRebuilt 的 WidgetTree 空指標造成 access violation 0x38，整輪不算通過。修正 fixture 使用有效 World outer 並先 Initialize 成功再 TakeWidget；實際 Slate 輸入另由 owned viewport phase 驗證，不以模型測試代替。新增 viewport 測試的前置審查亦修正不存在的 Interface scope，正式 scope 是 WorldCombat；category interface 是顯示分類，不能當 enum。

## E951：唯讀選取查詢呼叫帶清理副作用的連線檢查（2026-10-12）

unit-selection-build-v3 普通 nocache 建置失敗，UBT 85.77 秒，OmWorldBridgeActor.cpp:2096 C2662。const getter 誤用非 const 的 IsOwnedGameplayConnectionReady；該函式失敗時會 InvalidateOwnedHud，不能直接改成 const 或強制轉型。改為讀取 RuntimeStarted、連線診斷與玩家身分，失效回傳空資訊，不在 getter 清理 HUD。保留原失敗證據；重新普通建置成功後才執行受影響 native 測試。

# Unreal MOBA 防錯紀錄## E950：新增選取UI反射資料後普通建置要求引擎更新（2026-10-12）unit-selection-build-v1真FailedDueToEngineChange、UBT4.71秒；95行起列出NetCore與Engine modules等現存檔，原summary／logs保留，尚無C++成功。依正式Lua流程refresh nocache，再補普通成功才跑native。新選取來源凍結至每輪建置終態，禁止手改manifest／BuildId或清cache繞過。## E949：新選取來源的前置API／patch順序審查（2026-10-12）建置前審查抓到新碼猜測 HasRuntimeProfile／Sample.Pixel，實際既有API是 HasRuntimeControlProfile／ViewportPixel；已讀header及同一取點結構後修正，沒有失敗編譯或測試可宣稱。一次多檔patch又將同檔較後面的Sync段排在較前Publish段之前，驗證失敗且整組未套用，已讀檔確認後拆成依來源順序修改，去除無作用hunk。後續先定位API，再依檔案順序產生窄patch。

## E948：交接文件長行使用不完整patch上下文（2026-10-12）

apply_patch以首行片段當作整行匹配而失敗；整組patch未套用，已讀檔確認。改用一次性Lua對已接受summary檢查後前置新紀錄，保留完整歷史、不盲目重跑。沒有改寫既有驗收產物。


## E947：專用ring capture未完成EOF／資源更新，線性色比不適用tone-mapped PNG（2026-10-12）

v3仍拒絕：default原圖直接看仍灰地／無ring，deuteranopia的實際control已有3331個blue及299個warm像素；六項中五成功、marker只default失敗。等待發生在首度CaptureScene之前，首度render才可能排入material shader／PSO，故應依既有世界fixture順序在首次render＋texture residency之後再FinishAllCompilation／EOF／Flush，才保存第二次圖。保留首次觸發render，不以多次盲拍或再降像素門檻當解法；兩個預期顏色斷言及原100pixel下限不變。原v3輸出不覆寫。

team-markers-build-v4普通成功13.13秒；native-v2六項中五成功，PublicUnitTeamMarkers兩個GPU顏色斷言失敗，零warning。原report／兩PNG保留且直接檢視：default只有灰地與fallback模型，deuteranopia已有真正標記與草地，證明首張在pending render／asset更新前拍攝，不能接受。capture須沿既有世界fixture先FinishAllCompilation、SendAllEndOfFrameUpdates／Flush、StreamAllResources，再取圖，內部visibility control每次也必須EOF。線性RGB1.35比值不能套到tone-mapped sRGB；改以輸出通道差（主色至少12、次通道至少5）且仍至少100個被兩個真render control隔離的ring像素。沒有角色／草地顏色假陽性、沒有後製PNG、沒有放棄GPU檢查。此前native-v1六項純生命期／公開關係成功保留，尚未接受專用彩色渲染。

## E946：WorldBridge 區域 Owner 遮蔽 AActor 成員（2026-10-12）

team-markers-build-v2 正式refresh編譯 OmWorldBridgeActor.cpp2061 真 C4458，const auto* Owner 遮蔽 AActor::Owner。共用component與action owner已編譯，不能據此宣稱整輪成功。等本輪所有編譯程序終態後改成 GoverningOwner，保留原log再ordinary nocache重建；不降低警告級別。AActor子類區域變數避免使用Owner等既有成員名稱。另E944的唯讀診斷再次用了literal OmCombatVisual*路徑；已改為目錄配合 -g，沒有產生檔案變更。

## E945：新增隊伍標記元件的普通建置要求更新引擎（2026-10-12）

team-markers-build-v1 真實終態 FailedDueToEngineChange，UBT55.57秒。UHT9份generated輸出成功，尚無C++成功證據；原summary／build.log／UBT-Log保留。UBT120行起明列NetCore DLL／PDB／object與Engine modules／version等現存檔，故依正式Lua流程refresh nocache，再補普通NoEngineChanges／NoUBA／nocache成功才跑native。不手改BuildId或manifest，不刪cache，也不把refresh當普通驗收。

## E944：接續時猜錯插件模組路徑與輸出過量（2026-10-12）

唯讀診斷誤用 omfue/Source/OmEditor 與 omfue/Plugins/OmGenerated，實際兩者都在 omfue/Plugins/OmRuntime/Source。先用 rg --files 加檔名 filter 定位後已修正；沒有改動錯誤位置。初次全量 git status 混合來源讀取造成輸出截斷，後續僅讀指定來源與有限上下文。避免再從模組名稱猜目錄，也不以截斷輸出當作完整審查。

## E943：正交大範圍總覽只有地面裁切，不接受非均勻像素假陽性（2026-10-12）

neutral-unit-native-v1 三項原生成功、四張守衛idle／attack近照直接目視正常，但 guardian-camp-overview.png 黑底佔大半，只剩底部地面三角形，看不到兩個守衛。不得因Colors.Num()>64就接受；v1完整圖片／index保留，這張總覽拒絕。已先啟動69Blueprint相容檢查，其終態前不修改C++。接著核對UE SceneCaptureComponent2D的正交裁切／view origin推導，依實際OverviewAim及OrthoWidth設有效視錐，並加入兩個實際guardian bounds在畫面與非背景可見的檢查；fresh ordinary build／native-v2重拍後才接受。沒有用影像後製或放大替身遮掩問題，父項仍12/44。

## E942：UE5.8 GetStaticMesh 的 TObjectPtr 不參與 TestNotNull 模板推導（2026-10-12）

v2 終態 OtherCompilationError，1293.08秒；唯一 error C2672。OmWorldUnitVisualScaleTests.cpp 與 NetCore／generated／game DLL 已完成編譯／連結，原產物與失敗報告不改。終態後才把 getter 改 `.Get()`；fresh ordinary neutral-unit-build-v3 執行中，未宣稱 native／渲染成功。

neutral-unit-build-v2 refresh 的 OmNativeAnimatedUnitTests.cpp43 真 C2672：TestNotNull(TEXT("Verified original attachment mesh"), Part->GetStaticMesh()) 找不到 overload；UE5.8 getter 回 TObjectPtr<UStaticMesh>，FAutomationTestBase 模板需要 const ValueType*，不能靠隱式轉換推導。應明確 `.Get()`，保留原附掛 mesh 非空斷言；不是移除測試或改素材。來源仍凍結直到當輪終態；原 UBT log／build 輸出保留。修後再 ordinary NoEngineChanges／NoUBA／nocache；若仍有真 EngineChange 才走正式refresh，不盲目刷新。

## E941：新增守衛反射類別後普通建置真 EngineChange（2026-10-12）

neutral-unit-build-v1 普通 NoEngineChanges／NoUBA／nocache 真失敗，UBT215.02秒、FailedDueToEngineChange，UBT-Log.txt第96行起列出需要更新的 Engine／plugin UnrealEditor.modules manifests。UHT已完成新 generated class header處理，未有本次C++編譯成功證據；不是因 NoUBA banner 便猜開關無效。原build.log／UBT-Log／summary exit1保留。依既有正式Lua流程，只在這筆真EngineChange後允許 neutral-unit-build-v2 refresh nocache；完成後必須再 ordinary build-v3 NoEngineChanges／NoUBA／nocache，才跑native。禁止手改BuildId／manifest、刪Engine cache或換掉既有資料。accept recorder要求ordinary成功，不能拿refresh當普通驗收。

## E940：中立守衛真 producer 測試缺 ScriptUnitTag 明確匯入（2026-10-12）

首輪 cargo test real_camp_attack_publishes_resolved_recovery_without_hero_or_script 在編譯期 E0425，test module 的 super::* 不包含 ScriptUnitTag。依正式 runtime re-export 明確匯入；不是移除「不冒充 scripted／Hero」斷言，也不改 ABI。原工具輸出保留，fresh rerun 才判結果。英雄衣裝先前 hero-equipment-local-v7／accepted-v1 已完整通過，本錯誤屬後續中立守衛範圍。

## E939：Windows 檔案搜尋再次誤把 glob 當實際路徑（2026-10-12）

接續盤點時 rg 的 OmWorld*Tests.cpp 與 observable* 路徑再次回 os123；另猜 outcome.rs 的舊位置不存在。沒有修改或遺失來源。統一以已存在目錄作搜尋範圍、`-g '*Tests.cpp'` 作 filter；未知檔名先 rg --files。已定位 OmWorldUnitVisualScaleTests.cpp 與 runtime/native/comp/outcome.rs。這是操作錯誤，不能用空搜尋結果推論功能不存在；後續不再傳帶 glob 的檔案路徑給 rg。

## E938：修飾鍵精確守衛與 UE 跨元件整鍵消耗不同（2026-10-12）

hero-equipment-fixture-native-v1 三項實跑兩成功、一失敗：Admission／BodyRadius 成功；OwnedRecallItemHotkeys 的 Ctrl+Alt+1 legacy delegate expected1／actual0。核對 UE5.8 PlayerInput.cpp GetChordForKey：修飾鍵採 required subset 匹配，較高元件 Ctrl+1 的 bConsumeInput 會消耗整個 One，晚於此步的 MatchesExactModifiers 拒絕 Ctrl+Alt+1 無法回復下層派送。這不是模擬事件遺漏 Alt。保留 Ctrl+Alt+1 binding 存在與零誤觸斷言，新增 default profile 沒占用 K 的 Ctrl+Alt+K 實際 legacy delegate 派送，明確區分保留綁定與整鍵保留。不改正式輸入消耗政策，不假裝同一鍵任意修飾鍵保證跨元件透傳。原失敗報告不改，fresh build-v10／fixture-native-v2 才判修復。接續一次讀取把 Tests 子目錄猜錯；改先 rg --files 定位 OmEditor/Private，避免重複猜路徑。

## E937：完整原生清單三個 fixture 尚未適配正式模型／互動鍵（2026-10-12）

hero-equipment-local-v6 已真執行一般168＋實際音訊1，音訊三段PCM驗證成功；第五批32項三Fail，共12 errors，其餘各批成功。Admission 十筆錯誤：相鄰點僅2 world units，正式native英雄視覺選取半徑約65 units，非敵方位置仍被鄰近合法敵方覆蓋；應依當前公開body footprint隔離fixture座標，保留同位置tie／ID0／source切換／退休判定，不縮正式選取半徑。BodyRadius 一筆：原fixture縮放OmNativeBodyMesh，但正式Hero已使用骨架mesh，舊body為隱藏fallback；靜態裝飾／插值／半徑案例應明確用靜態fallback Creep，非等比縮放期望依原始XY bounds與比例推導；現有WorldUnitVisualScale另驗真骨架bounds與縮放，不能改回正式Hero使用cube。OwnedRecallItemHotkeys 一筆：Ctrl+1 已是正式Interaction1，不能再要求舊delegate執行；未被profile使用的Ctrl+Alt+1保留legacy delegate案例，正式Ctrl+1由communication真事件驗收。第五批已能正常啟動，無新的os5；全部原始index／error MD保留。目前IPC仍在執行，terminal前不改C++或templates。第一次MD數字誤寫13，已依原始index 10＋1＋1更正12；一次只比對段落前半的patch被拒，重新以完整原行修正。

E937 source repair：Admission 先從真發布actor取得最大GetSelectionBodyRadiusCm，以原2-unit最小中心間隔推導IsolationFactor，實際發布座標與20個查詢中心同步隔離；同原X的tie仍同位置，header100換算／source切換／ID0／退休斷言保留。BodyRadius 明確用無catalog靜態Creep，再透過NativeStaticVisualMesh／NativeStaticVisualTransform宣告4×／2×／1×形狀，來源scale先除去現有PresentationVisualScale才交正式世界正規化；不能直接改會在下一frame被恢復的呈現scale。期望半徑用原XY bounds推導，不沿用舊1.4常數。Ctrl+Alt+1 exact legacy chord獨立於正式Ctrl+1 Interaction。v6 IPC另真397／177／138與cleanup成功，local整體仍false；改動後先普通build-v9＋三項scoped native，未通過前不計修復成功。

## E936：CmdLineFile 內不能放比展開更早讀取的 TestExit（2026-10-12）

hero-equipment-local-v5 第一批確實載入明文 manifest、32項完成並輸出原始 index，但 Queue Empty 後不退出。核對 UE LaunchEngineLoop：TestExit logger 註冊約1920行，CmdLineFile 展開約2088行，故 TestExit 放進檔案太晚。必須將短的 TestExit 留作原始啟動參數；測試清單／report／log仍在正式 .txt。受控停止本輪：已讀取 Win32_Process 核對 lua119508 為此 v5內層workflow、editor138672 為10:08啟動的該批次，再由固定 Lua process.inspect_owned／stop_owned 以精確 creation token 清理，artifact獨立記錄 controlled abort，不把人工關閉當正常成功。未改 Engine／防護設定。fresh v6 重驗，66契約案例保留並修退出條件斷言。

## E935：Windows 啟動拒絕已有 Defender CmdLine 偵測證據（2026-10-12）

v4 終態正式確認第五批 run UnrealEditor-Cmd.exe os5；IPC 本身完整成功397 cases／177 intents／138 stages、catalog相容、ACK false、editor／publisher／profile清理及 listener released。聊天 immediate offset0→44／end44，chat-received-large.png 已直接目視。整個 local-v4 因音訊與第五批失敗仍為 false，不能改寫原報告。後續 source audit 統一剩餘 item-card／foreign-user release 分支到 ReleasedPointer，三個放開路徑都以空 pressed set 表示事後狀態；普通 build-v8 後 fresh 重驗。新 runner 一般168與實際音訊1共169，66 Lua契約案例通過；每批原始錯誤獨立寫 MD，增量 native-progress 與明文 CmdLineFile 保留。

讀取 Microsoft-Windows-Windows Defender/Operational：2026-10-12 09:57:58 event1116 與09:58:20 event1117，Trojan:Win32/ClickFix.DAD!MTB，資源是 CmdLine:_D:\UE5.8\Engine\Binaries\Win64\UnrealEditor-Cmd.exe 加第五批本機 Automation RunTests 長清單；不是已確認某個來源檔含惡意程式。時間與 v4 第五批無 index 的啟動失敗吻合，不能再把此輪原因只歸於猜測命令長度。未停用防護、加排除或還原隔離物。核對 UE LaunchEngineLoop 的 LaunchCheckForCmdLineFile 正式支援 .txt 明文展開並完整記錄於 log；改採當次可檢視 command manifest，使用官方精確 ^name$ filters，仍驗169精確集合，避免把長清單當 OS 啟動參數。這是一般測試 manifest 管理，不改測試或隱藏參數。舊 E926 的第一次拒絕不溯推必然同因。

## E934：完整驗收必須依測試所需裝置分派執行 profile（2026-10-12）

hero-equipment-local-v4 前三批96 Success，ControlRuntimePersistence 已通過，第三批 warning 由8降為7，舊 stale descendant 警告消失。第四批31 Success／OriginalAudioOutput Fail：fresh bounded -om-original-audio-output=RUN required。原生 discovery 納入音訊測試，但通用啟動一律 NoSound，漏接既有 scripts/verify_original_audio_output.lua 的 AudioMixer／唯一輸出目錄／三段實際 PCM 檢查。不能排除音訊以縮成168項；應一般168加音訊專用1項，再驗原始169精確聯集。第五批尚無 index，失敗原因待整 workflow 終態確認，不把未啟動當作成功；第六批真9 Success。原失敗全保留，完整驗收未通過。

## E933：滑鼠 release fixture 不能把已放開按鈕仍列為 pressed（2026-10-12）

hero-equipment-local-v3 IPC 原始終態明確失敗：chat wheel still captured before dispatch，captor leaf=SOmOwnedFixtureViewport；全域命中歷史已通過。對照 UE SceneViewport::OnMouseButtonUp，只有事件表示左右中鍵均已放開才會 ReleaseMouseCapture。共用 Up 原本使用 Pointer(Button)，誤把 released Button 放進 PressedButtons。修為空 pressed set、保留 EffectingButton 與實際 ProcessMouseButtonUpEvent；不手動 ReleaseAllPointerCapture、不改正式聊天／camera／guard。原始 wheel failure 與新 capture 診斷全部保留。普通 build-v6 成功122.67秒不代表整驗收通過；native 95／96 Success，其餘73未跑，Blueprint未跑，須完整重驗。

## E932：完整原生第三批設定 slider 命中失敗（2026-10-12）

hero-equipment-local-v3 第一／二批各 32 Success；第三批 31 Success／ControlRuntimePersistence Fail，actual painted hit path contains master slider 為 false，另保留 scroll descendant warning。準備 fixture 在 layout 真 Tick 前取得 slider，後續 NativeTick 的 SyncPresentation／ApplyMeasuredLayout 可 RefreshRows 清除重建行，故不可跨 layout settle 持有舊 child。必須先完成 hosted layout，再重新取當前 slider、捲到當前 descendant 並驗實際 global hit path；不能直接呼叫 profile setter 或跳過 pointer 驗收。仍待當前 IPC 終態後修改／重跑，不能把未跑的後三批計為通過。

## E931：診斷指令與 patch 必須先核對實際格式（2026-10-12）

再次把 Windows glob 放進 rg literal path，造成 os123，改用目錄與 -g 篩選；另一個 demo_render* 搜尋又犯同錯，禁止以 literal glob 重試。多餘讀取未確認存在的 tools/lua-host/src/lib.rs 失敗，沒有必要時不讀猜測路徑。一次 combined patch 使用不存在的簡寫 heading，整批拒絕，沒有部分寫入；重新依實際 heading 套用。fixture 診斷先核對 Test 為 reference，使用 Test.AddInfo，避免編譯後才發現型別錯誤。

## E930：完整 IPC 的聊天真滾輪偏移不變（2026-10-12）

hero-equipment-local-v2 在聊天放大歷史失敗，before0／after0／end44，viewport358×77、row346×56。已直接檢視 chat-scroll-failure.png，文字超出可視區，不能宣稱完整驗收成功。UE 5.8 ProcessMouseWheelOrGestureEvent 以全域 interactive windows 命中，再優先使用 pointer capture；原 fixture 只核對單一 window path 不足以確認實際 dispatch。先加入全域 path 與 capture 診斷及 immediate offset，保留真 wheel／實際相機不動與 row 可見斷言，不直接改 offset、不盲目釋放 capture，根因尚待新一輪證據確認。

## E929：商店斷線 fixture 必須先通過本機可用狀態（2026-10-12）

hero-equipment-local-v2 第一批原生真 32 項：31 Success／OwnedInputUnavailableHud Fail，一筆能力通知累計數 expected25／actual21。這是通知次數，不是 disabled mask；前一次 commentary 已更正。既有 fixture 把 Bar 商店狀態保留 unavailable，SubmitShopAction 因正式本機 availability／protocol／shop-region guard 正確先拒絕，未走到連線檢查，也不應清全域 Actor HUD。修 fixture 要先構造合法公開商店狀態，才驗斷線清除；另保留本機 unavailable 不廣播與清除後重複操作不 flood 的斷言，不刪 guard、不降低既有清除通知數。第一批成功啟動，但不能宣稱 batching 已證實 Windows os5 根因。其他批次尚未執行，不計成功。下一個實際錯誤 E930。

## E928：UBA local 名稱不能直接當作 NoUBA 未生效（2026-10-12）

hero-equipment-build-v5 編譯輸入 fixture 期間 CL 長時間低 CPU／等待，最終普通 NoEngineChanges／NoUBA／nocache 成功，UBT 440.75 秒，其中約 403 秒 unknown 等待。核對 UE 5.8 正式 BuildConfiguration 與 UBAExecutor 後，NoUBA 有正式開關，舊 bAllowUBALocalExecutor 已 deprecated alias；日誌仍可顯示 local executor，不代表 detours 開啟，既有 Lua 註解已說明此差異。未改 Engine／config／cache，也未加入推測性的不存在開關。一次查詢舊 CL PID 時程序已退出，是正常生命週期競態，不是編譯失敗。下一個實際錯誤 E929。

## E927：仍未完成的驗收不能讀取不存在原生報告（2026-10-12）

hero-equipment-local-v1 切至 IPC 階段時，原生子程序其實已在建立時遭 Windows os error 5 拒絕，尚無 native-report／native-engine.log。盤點工具提早讀取這些不存在路徑失敗，又猜錯 IPC log 名稱；完整終態報告才確認原因。先列出實際檔案，live log 只用回傳的確實路徑，完成摘要只在整個 workflow terminal 後讀取；一行 JSON 必須用 Lua 小摘要，禁止直接輸出全文。下一個實際錯誤 E928。

## E926：新模型回歸須用公開選取圓尋找地板，完整測試啟動失敗不得算通過（2026-10-12）

hero-equipment-local-v1 整體失敗：原生 169 項未啟動，Windows 建立 UnrealEditor-Cmd.exe 回傳存取被拒 os error 5，尚不推斷是權限／防毒／命令長度原因；owned viewport 在 item2-far-refuses 失敗，舊 fixture 固定 800～1200cm 點都落入新模型公開 6546cm 選取半徑。8 項 scoped native 本身成功，不代表完整回歸。正式 selection 使用實際 visual body bounds 是既有契約，不能縮半徑讓測試過；fixture 應依當前公開選取圓往外找點，有界候選、保留原始相機投影／hit-test／事件與 wire assertion。完整原生 runner 改採有界 exact-name 批次，逐批／全體都驗精確集合及 counters，不縮驗收範圍或把沒啟動算成功。原失敗報告及清理證據保留。下一個實際錯誤 E928。

## E925：World bridge 來源位於 OmGenerated，不依型別簡稱猜路徑（2026-10-12）

唯讀盤點再次猜測 OmRuntime/Private/OmWorldBridge.cpp，不存在且未修改檔案。已由 rg --files omfue -g '*WorldBridge*' 找到真實 OmGenerated/Private/OmWorldBridgeActor.cpp；SyncPublicTrees／地形批次也在同檔，OmRuntime/Private/OmWorldPresentationActor.cpp 僅共用地面與燈光。往後先列出實際檔案，再把回傳完整路徑交給搜尋，禁止自行縮寫型別名推導檔名。下一個實際錯誤 E926。

## E924：只抽 Head subtree 會漏掉頸部原始幾何（2026-10-12）

hero-equipment-native-v2 原圖審查在 Luminary 死亡姿勢發現頭與衣領間缺口。六個來源骨架的同性別 neck_01／Head reference pose 完全一致，排除 rest pose 差異，不能任意移動頭部掩蓋。原 Head subtree 抽取未包含 neck_01 頂點；新增頭頸 subtree 資產，保留既有來源與產物，再做零寫入負例與原圖驗證。另唯讀 rg 指令誤帶 PowerShell 的 -ErrorAction 旗標，安全失敗無修改；rg 僅用其自身參數與已列出的確實檔名。下一個實際錯誤 E925。

## E923：改裝備比例要同步檢查生命週期斷言（2026-10-12）

hero-equipment-build-v3 建置成功後的來源審查發現 lifecycle 還留有 scale=2 的舊斷言；尚未啟動 native，沒有把此建置當測試通過。同步修正為原始 scale=1，再正式建置後才跑全部受影響測試。另唯讀搜尋曾猜測不存在 OmLocalPlayerControlSubsystem.cpp；真入口由 rg --files 找到 OmPlayerActionOwnerComponent.cpp，不再猜 class 對應路徑。下一個實際錯誤 E924。

## E922：短劍原始米制比例不能任意放大兩倍（2026-10-12）

依實際來源 bounds 與骨架指根位置檢查，先前為追求約 90cm 劍長使用 scale=2，連同原 10.8cm 握柄放大至 21.6cm，使手部造型不合理；90cm 不是既有需求。保留原始 44.8cm 短劍與 scale=1，依手腕至四指根平均距離取掌心握點，使用通用 attachment metadata 修正。另一公開 Sword_2 來源同樣額度已滿，檢查拒絕且保留回應，停止該來源下載。修正仍須新建置與原圖檢查才能接受。下一個實際錯誤 E923。

## E921：公開來源下載也必須驗證內容，不把 HTML 當模型（2026-10-12）

Quaternius 官方連結的公開 Drive 中 Sword.obj 下載回傳 HTTP 成功但內容為 Google Drive Quota exceeded HTML。來源工具在任何匯入前拒絕並保留回應，沒有寫入正式模型。只能選其他正常公開來源或其他原始模型，不繞過額度／登入；來源不可得不能當作功能成功。另兩次唯讀查詢猜測 scripts/lib 與 omfue/Source 路徑失敗，實際為 scripts/world_model_assets.lua 與 omfue/Plugins/OmRuntime/Source；之後同批使用 rg --files 取得真路徑。下一個實際錯誤 E922。

## E920：數值驗證成功仍要審查武器原始造型與握點（2026-10-12）

hero-equipment-native-v2 真 8 Success／零 warning／零 error，既有 69 Blueprint 也編譯成功，但逐張看 Date 的七種動作後，Mini Dungeon 劍的橘色粗握柄與完整人物手部搭配不合適，因此武器畫面仍未接受。原始 OBJ 底部獨立島為 24 vertices，寬約 10.8cm，不能單憑外觀宣稱作者做的是手，也不能刪改原始幾何掩蓋問題。Prototype Kit 原始預覽同樣不適合，未匯入正式專案。改查官方 Quaternius Modular Weapons 公開來源，保留既有裝備接線、舊來源與失敗／成功證據。另一次唯讀查詢猜測 scripts/lua_data/maps 不存在，實際地圖在 templates/moba_maps.lua；往後先列檔再讀取。下一個實際錯誤 E921。

## E919：骨骼裝備ID不能使用UObject唯一名稱，尺寸驗證不能套頭部上限（2026-10-12）

hero-equipment-native-v1實際7 Success／NativeCharacterAssembly Fail，96錯誤保留。主要原因是fixture以固定NativeAttachment_head／sword比較MakeUniqueObjectName產生的帶唯一suffix名稱，導致head誤分成weapon；另三筆把90公分劍的旋轉AABB套頭部100公分門檻，旋轉後AABB可變大。新增穩定OmNativeAttachment.<binding-id>元件語意tag，不改唯一UObject名字；fixture驗tag、實際骨架parent／socket、來源模型、握點、metric scale與旋轉不變的source sphere radius，頭部仍保留原嚴格尺寸／材質檢查。四張當次idle／attack原圖已目視衣裝與劍跟隨動作，但此v1整組仍失敗不接受；後續全受影響測試重新跑。下一個實際錯誤 E920。

## E918：正式建置進行中不得讀取完成後才建立的 summary（2026-10-12）

hero-equipment-build-v1 尚在執行時，以rg查詢其summary.json導致不存在檔案錯誤；這不是建置失敗。完成狀態只能由已保存session的terminal exit確認，中途可讀真UBT log／列目錄，禁止再次查尚未寫出的summary。下一個實際錯誤 E919。

## E917：匯入摘要工具名稱仍須由檔案清單確認（2026-10-12）

唯讀查詢猜測 inspect-world-model-import-v1.lua 存在，實際不存在且未修改檔案。建立明確的 inspect-equipment-import-v1.lua 後才執行，已驗新3模型／9packages、8負例零寫入、repeat7verified／零保存、原12packages SHA／mtime不變。不得再憑名稱猜測工具路徑。E916當次修復已實驗：animated-unit-native-v4兩Success／零warning，直接看move與attack原圖明顯不同且無遮擋，正式記錄animated-unit-accepted-v1；同frame GPU pose問題只修fixture發布時序，未改正常runtime或Engine。下一個實際錯誤 E918。

## E916：同 Engine frame 連續骨架截圖要發布新 GPU pose（2026-10-12）

animated-unit-native-v3真兩Success，北側近照已無遮擋，但move／attack原圖身體仍近乎一致，只有獨立頭部附掛改變，不據此宣稱兩動作已接受。查UE來源後，preview fixture同一Engine frame內反覆改clip／pose，需要確保renderer skin object接收當次骨架資料；既有CharacterAssembly fixture用GetCPUSkinnedVertices切換skin模式會重建render state。本fixture改用元件正式RecreateRenderState_Concurrent在每張近照前發布當次pose，保留GPU skin、原素材與正常runtime路徑。新增attack AnimInstance資產相等及前後超過5骨頭姿勢差異斷言。另一次工具搜尋加入不存在scripts/tools目錄已安全失敗無修改，後續只使用存在的scripts目錄。下一個實際錯誤 E917。

## E915：角色近照要避免既有場景遮擋，兩個成功測試不能代替原圖審查（2026-10-12）

animated-unit-native-v2 真兩Success，bridge全景小兵已非T-pose，但move／attack近照的相機從南側取景，射線穿過塔牆面，主體大部分被遮擋。Primary已看四張原圖，拒絕把兩張遮擋圖當角色動作接受；不能關閉塔可見性／裁圖／後製。調整同一公開場景相機至小兵北側，再正式普通建置與fresh截圖；原v2證據保留。下一個實際錯誤 E916。

## E914：公開小兵動畫整合 fixture 必須帶已公開生命值（2026-10-12）

animated-unit-rust-v1 core真722成功，bridge真39成功／1失敗／1既有ignored；新整合測試第二frame仍Stand而非Walk。fixture只帶render／identity，未帶公開property，依既有安全契約decoder將HP保留0，motion observer對死亡／未知生命值不推測移動，正確拒絕。補既有raw_property helper構造公開180／180 Fixed64生命值；不修改正式decoder／observer，不放寬死亡動畫規則。原v1失敗報告保留，後續重新跑受影響bridge與all-target checks，core來源／測試不變可明示沿用722。下一個實際錯誤 E915。

## E913：工具回傳先保存變數，未落地的執行 log 不當成檔案遺失（2026-10-12）

啟動 animated-unit-rust-v1 後誤對 text(...) 的 undefined 回傳讀 session_id，JS報TypeError，但子程序90683已成功啟動，沒有重跑；改以工具已回傳的session追蹤。隨後提前讀只有process結束才寫出的core-tests.log，檔案尚未建立，並非Rust失敗。後續先保存 const result，再 text(result)，只對 result.session_id 判斷；中途只檢查目錄或poll，不猜測尚未落地log。下一個實際錯誤 E914。

## E912：橋接小兵截圖不得僅驗骨架存在，必須送入狀態並刷新姿勢（2026-10-12）

animated-unit-native-v1 真14 Success，但當次 bridge PNG 的新小兵為 T-pose；原生生命期測試成功不等於截圖已呈現動畫。fixture只送entity且只刷新英雄骨架，未送小兵 animation record／刷新其骨架。正式橋接 DispatchAnimationStates 對所有 AOmUnitActor 分派，沒有英雄限定。補真 OmAnimationStateRecord、實際move binding／AnimInstance相等斷言、預覽骨架姿勢刷新與完整四單位pool回收重用；不得把v1原圖當成動畫接受。69既有Blueprint commandlet編譯成功、errors／warnings／failed-loads全0，package SHA／mtime全不變。接續新Rust整合測試驗公開identity→真compiled catalog→creep frame與walk動畫，未新增script callback／權威對局。下一個實際錯誤 E913。

## E911：接續實作的唯讀探索仍重複路徑猜測，必須限制查詢方式（2026-10-12）

本輪唯讀探索先後把 scripts/ue_local_acceptance*.lua、docs/plans/2026-10-11-character* 作為 rg literal path，及猜測 OmDefaultGameMode.cpp 路徑，均安全失敗無檔案修改；同時一次 .ai-collab 遞迴掃描延長為背景 session，已讀取至正常結束。這些是已知工具使用錯誤的重犯，後續一律先 rg --files 查出存在路徑，副檔名只放 -g，使用確切檔案／來源目錄，避免廣掃 evidence。下一個實際錯誤 E912。

## E910：新增共用原生父類後的正式建置依實際 EngineChange 才 refresh（2026-10-12）

animated-unit-build-v1 普通 NoEngineChanges／nocache 建置實際回傳 FailedDueToEngineChange，UBT-Log.txt 396／442 行保留證據；沒有先修改 Engine 或 BuildId。依正式流程執行 animated-unit-build-v2 refresh no-cache，完成前不改 C++／模板／Rust 來源。另一次唯讀查詢仍誤將 Windows literal glob `.ai-collab/*.lua` 當路徑，已改固定目錄搭配 `-g`；不得再重複此查詢方式。下一個實際錯誤 E911。

## E909：進度文件寫入必須使用完整 patch context 與明確覆寫參數（2026-10-12）

更新 machine handoff 時用一行的前綴當成完整 patch 行，apply_patch安全拒絕且未改檔；改用固定Lua路徑工具prepend，又誤猜path.write覆寫參數位置而兩次安全拒絕未改檔。實際讀 tools/lua/lib/path.lua 後確認簽名 write(path,data,overwrite,binary)，覆寫是第三參數，第四是binary。修為完整舊文、重複marker守衛後write(file,data,true,true)，保留全部歷史內容與行尾。不得再猜測API參數。這是工具／文件錯誤，並非UE功能失敗。下一個錯誤E910。

## E908：世界 XY 投影不適用垂直地形面，必須檢查所有可見面（2026-10-12）

public-world-props-build-v2普通nocache成功57.77秒，native-v2真13 Success／既有5 warning，資料邊界與生命期成功；Primary 查看三張當次原圖後拒絕terrain視覺接受：兩矩形頂面正常但垂直面成直條。原因是沿用路面 world XY UV，側面 Z 不參與座標。不是用調亮／放寬像素測試補救。

正式修復新增 bounded world_triplanar_v1 匯入選項與獨立 M_stone_terrain：三個正交世界座標投影，依 world normal 絕對值正規化混合，24個明確材質節點逐一驗證；共用既有T_stone，保留原grass／stone ground graphs和資產。texture像素也與原PNG嚴格比對。新投影有獨立managed signature，全部現有asset預檢後才建立新材質，禁止覆蓋未知資產；工作流驗負例零寫入、舊package不變、repeat零保存。尚待build-v3／正式triplanar匯入及fresh native／原圖，不能把native-v2 PASS冒稱材質正確。

E908修復證據：terrain-triplanar-import-v1真10負例零寫入，新增1材質／共用原T_stone、第二次2verified／零保存，所有原4地面與12模型package SHA／mtime不變。普通build-v3 13.15秒、detail-only v4 9.56秒；native-v3真13 Success／既有5警告保留，v4兩當前測試Success／0warning，其餘11明示沿用。Primary直接看當次近距離三面圖與bridge全景，原石材在頂面和兩側正常、無垂直拉條。public-world-terrain-accepted-v1綁定來源／stage／四原圖。沒有新對局／ACK或完整世界接受。下一個實際錯誤編號E909。

## E907：公開地景驗證須保留既有 fixture 警告並有界處理極端矩形（2026-10-12）

public-world-props-native-v1 真13 Success，新增 PublicTreePresentation／修改的 bridge render 均零 warning，但整組有5 warning：FrameContract刻意 unsupported-mode 拒絕1筆，RuntimePresentationReset原 fixture 未建立world context而 DestroyActor 4筆。原報告保留，accepted 明示5，不能稱整組零警告。三張原圖直接目視成功，正式橋接兩株原始樹已顯示；無新權威對局／ACK。

下一步 terrain source review 發現原矩形 `(max-min)*WorldScale` 先用 float 做差，finite 原始值仍可溢位，亦未限制世界幾何尺寸／位置。正式改 double 中間值、共用世界厘米界限與 frame 層32矩形預拒絕，不裁剪／猜測權威形狀；原 bounds 校正及矩形完整輪廓保留。加入 soft stone material、材質metadata變更不重建instance及診斷fallback、scale往返、NaN／極端finite／倒置末筆整批清空與恢復。尚待 ordinary build-v2／fresh native 與當次圖片，不能先宣稱修復通過。

另一次 rg 搜尋過大範圍進入 build產物，已中止；以明確 src 目錄及副檔名限制查詢，勿掃 target／大cache。

## E906：EditorPreview 截圖必須完成資產編譯並驗證 scene proxy（2026-10-12）

world-model-native-v2 真25 Success／0 warning，模型黑面修復已目視，但 bridge PNG 仍缺塔／主堡，繼續拒絕接受。位置／visible／bounds 均正確仍不足以代表已渲染。查 Engine StaticMeshComponentHelper：mesh IsCompiling／render data 未初始化／PSO 未完成都可暫不建立 scene proxy；正常引擎 frame 會完成編譯後重建，孤立 EditorPreview 測試必須等待正式資產編譯完成再送出 end-of-frame。fixture 加入 FAssetCompilingManager::FinishAllCompilation、scene proxy 非空斷言與實際藍屋頂像素門檻，未改 runtime 生命期、未關 PSO 或繞過渲染。ordinary nocache build-v9 真成功12.97秒；fresh native-v3 三個受影響測試全 Success／0 warning，renderer log 兩 proxy=1／state=1／compiling=0，實際三張 PNG 已目視：塔／主堡與英雄道路同圖正常，四原始模型正面正常，回城尺度保留。不是新對局或完整世界驗收。

工具錯誤另記：store(undefined session_id) 會拋錯，但對應同步 contracts-v3 已真成功；往後只在 session_id 存在時 store。Windows rg 檔案路徑不可用 literal glob，且 Engine／generated 私有檔名必須先 rg --files 查得，接手仍數次誤猜，禁止沿用猜測。

## E905：原生測試 PASS 仍必須檢查畫面；OBJ 面序遵循 Unreal 左手座標規則（2026-10-12）

world-model-native-v1 真 25 Success／0 warning，但 Primary 逐張查看三張 PNG 後拒絕視覺接受：original-models.png 多數面為黑面，bridge-lane-scale.png 未見兩座結構。原圖與原報告保留，不建立 accepted 紀錄。已查 Engine StaticMeshOperations：UE 法線使用 Cross(P2-P0,P1-P0)。Y-up 到 Z-up 的軸交換已反射，原 importer 又把 OBJ corner 改成 1,3,2，造成面序與原作者法線相反。正式修復保留 1,2,3，逐面正向 dot 驗證；四原始來源共 1166 可渲染面、零法線方向衝突。新增 signature v2 與精確 legacy 幾何／UV／法線／材質／貼圖預檢，全部預檢後才重建管理中資產，禁止手動刪除／覆蓋未知資產。upgraded 是 imported 的子集，代表重建而非新建。原生新增逐面法線檢查，橋接增加 published position／body bounds／visibility 斷言並在 EditorPreview 捕捉前送出 end-of-frame 更新。橋接根因仍待 fresh 原生畫面，不能先宣稱修好。

接手讀檔另兩次誤猜 Plugins/Omoba 與 OmSingleLaneTower.cpp 路徑；正確路徑以 rg --files／符號查詢為準。不要再次輸出整個 dirty status（數千行）或把路徑猜測當有效檔案。尚待本輪普通 nocache build、fresh import、native 與三圖複驗。

## E904：補明確module依賴後以 fresh ordinary 的真 EngineChange 重新授權refresh（2026-10-12）

world-model-import-build-v6 ordinary UBT21.89秒真FailedDueToEngineChange，原summary/UBT完整保留。E902／E903的OtherCompilationError沒有直接觸發refresh；先依序ordinary v5、修明確ImageCore依賴後ordinary v6，只有這次新EngineChange才接正式refresh v7。原未完成bootstrap checkpoint維持正式workflow管理，不手刪或改Engine modules/version。當前52 codegen＋17 Lua contracts在world-model-contracts-v2重新完整保存成功，指紋包含當前C++ fixture／ImageCore Build.cs／capture runner。

## E903：使用 FImage 格式轉換必須明列 ImageCore module dependency（2026-10-12）

world-model-import-build-v5 ordinary NoEngineChanges 真OtherCompilationError，UBT16.43秒；兩個新C++檔已編譯成功，但TextureValid新增 FImage::CopyTo／ChangeFormat在link出現LNK2019。依API的 IMAGECORE_API 所屬module補 OmEditor.Build.cs ImageCore 明確dependency，不靠傳遞連結或省略像素驗證。沒有再次refresh；接ordinary nocache fresh build。原log與失敗DLL連結證據保留。初次patch猜不存在ImageWrapper行導致context mismatch，整個patch零修改；依實際RHI行重新套用後才啟動build。

## E902：UE5.8 GetStaticMesh 回傳 TObjectPtr，automation 泛型比較須明確取 pointer（2026-10-12）

world-model-import-build-v4 正式refresh UBT473.89秒真OtherCompilationError，兩個新fixture的 TestEqual 把 TObjectPtr<UStaticMesh> 與 UStaticMesh* 混用，無法推導同一泛型型別。改 GetStaticMesh().Get()，不改生產模型邏輯或移除斷言。原輸出保留；此次不是EngineChange，先ordinary fresh build，不因編譯錯誤直接再次refresh。另補四來源模型獨立D3D12預覽測試與texture source像素／sampler核對，完整世界地景放置仍獨立。

refresh期間兩個cl幾乎無CPU且讀取PCH慢，依實際responsefile定位，Lua順序唯讀UnrealEd 2574581760與CoreUObject537001984 bytes，兩mtime／size完全不變。沒有更改Engine／cache／BuildId／modules。fresh Rust-v3已722 core與39 IPC過，另1項真HUDcapture依賴ignored明示保留；四checks最後已完成成功。

## E901：新增呈現欄位後 ordinary build 真 FailedDueToEngineChange 才進正式 refresh（2026-10-12）

world-model-import-build-v3 原始 summary/build/UBT log 保留：UBT137.99秒，UHT129.45秒／12個generated檔，真 FailedDueToEngineChange，列出既有 NetCore DLL/PDB 及 Engine module/version 等預期更新。這次才符合既有正式refresh條件，後續使用相同 Lua build workflow 的 --refresh-engine／NoUBA／nocache，不手改 Engine source、BuildId、modules、cache 或伪造成功。先前 v1 OtherCompilationError 不授權 refresh，v2曾 ordinary成功仍保留。終止確認後補 material expression 的 Material owner，依既有已驗原生 importer 一致設定，再開始 fresh refresh build；此時不得宣稱 C++或模型已驗成功。

## E900：原作者 OBJ 包含退化面；新測試必須與當前生成規則一致（2026-10-12）

world-model-import-v1 八個負例都拒絕且零寫入，first-result 正式失敗於 geometry preflight，saved/imported=0。唯讀 Lua 幾何盤點發現 tower_square 原352面含4個零面積面，base_keep_roof 原516面含8個，pine/rock無退化。原始 OBJ/MTL/PNG 與授權 SHA 保留，不修改原檔；通用 parser 僅略過零面積面，在建立 vertex instance 前檢查，保留全部原 vertices，新增 source_triangles / degenerate_triangles 對帳與 signature policy。其他壞索引、法線、不支援語法及全退化仍拒絕，補 OBJ id/行號診斷，不放寬有限值與容量上限。正式新匯入仍待重建驗證。

fresh codegen 真52測試首輪51過1失敗：測試誤期待 AOmTowerSingleLaneTower，既有 Tower 命名規則是 AOm+PascalId，應為 AOmSingleLaneTower。修測試預期，保留生產規則；加入正式 world_units Lua 後 fresh52全過。舊49測試是在新增三個測試前啟動，不代表新52已驗。一次 apply_patch context mismatch 零修改後依實際內容重新套用。查詢又猜 class_events.rs 不存在，後續限定 rg --files inventory；未改資料。

## E899：模型匯入commandlet需避免Engine同名FMaterial並以實際API建texture（2026-10-12）

world-model-import-build-v1 ordinary真OtherCompilationError，UBT648.45秒，原log／summary保留；不是EngineChange，不准refresh。自訂FMaterial在Main using namespace後與Engine FMaterial衝突，改FSourceMaterial；CreateTexture2DFromImage誤用4參數，依實際UE ImageUtils.h及既有已驗world surface importer改CreateTexture(ETextureClass::TwoD,Image,Package,Name,Flags,false)，不猜overload。另補原生material compiled shader與完整graph guard後重建。當時cl兩程序讀2.57GB PCH很慢；只用Lua順序唯讀原檔，mtime／size不變，未修改Engine、BuildId、modules或cache。曾誤將active_cpp_build清空，但在任何C++修改前立刻恢復，實際terminal／summary確認完成後才開始改source。唯讀又猜ue_build.lua、OmRuntimeBridgeActor.cpp及template-gen/schema目錄、Windows literal glob，記錄並要求後續查詢只使用實際inventory存在路徑；未修改資料。

## E898：移動fixture須完整註冊新共享command system所需storage（2026-10-12）

schema碰撞修復後完整core重跑真717 passed／5 failed，原terminal session53822保留；兩個MalformedBaseline已消失。剩餘同五項movement fixture補ReplicaIdentity後又缺DisclosedWardBody，均由共同HeroCommandTick SystemData要求；依實際SystemData盤點一次補齊，不逐次放寬生產fetch或跳過測試。後續正式Lua runner改接受fresh run-id保存完整log，避免大段terminal輸出截斷。

## E897：完整Rust檢查揭露回城／戰鬥關係schema碰撞與移動fixture缺註冊（2026-10-12）

world-identity-rust-v1/core-tests.log真715 passed／7 failed，原始結果保留。兩個filtered combat／ward測試MalformedBaseline原因是既有channel_visual_state與DISCLOSED_COMBAT_RELATION同用0x464f4713；單獨回城fixture未包含combat bytes，先前範圍驗證未捕捉。combat保留4713，channel改未使用4716，新presentation identity4715，selective protocol9拒絕舊peer；secure allowlist加入去重前後長度相等斷言，不能讓BTreeSet靜默吞重複。其餘五個移動測試缺ReplicaIdentity storage，補共同movement_world註冊，不移除生產讀取／guard。接續fresh測試，不把舊失敗改成功。

本輪唯讀另對含空白OBJ路徑使用反斜線escape，PowerShell不接受；改-LiteralPath單引號。曾Get-Content讀一行巨型source manifest造成輸出截斷；後續一律固定Lua decode後只印小型summary。中間新driver草稿使用不存在的transpose_option，在正式編譯前source review即改成match與公開共享structure decoder，未冒稱編譯失敗。

## E896：查詢必須先以實際inventory定位；場景識別需追到正式投影（2026-10-12）

接續場景來源盤點時，又先猜render_frame.rs、comp/tower.rs、templates/init.lua、unit.rs、native/comp.rs及visibility目錄，唯讀工具報缺檔；另一次omoba-render*當Windows literal路徑失敗。這些查詢沒有修改資料，但重複先前已記錄的錯誤。後續只使用rg --files實際回傳的完整路徑或已知存在目錄，glob放-g。source review另發現single_lane_tower／base／creep未在正式模板catalog登錄；native snapshot分類仰賴Tower component，名稱仰賴ScriptUnitTag，不能只補美術或把未知catalog冒充其它單位。先追正式filtered read-view如何公開identity，測試現有缺口後再決定通用接線，不能把source推測當已跑失敗測試。

## E895：HTTP200可能是下載流量限制頁，不能當授權檔（2026-10-12）

environment-source-research-v2保存的 medieval-license.txt 實際2040bytes HTML「Google Drive - Quota exceeded」，不是license。原下載helper只驗HTTP exit0／長度，summary.success=true僅代表下載流程結束，不能當license／模型接受；原HTML及summary保留，另做嚴格review=false。兩folder原HTML仍只作metadata盤點。後續下載須依預期內容驗magic／格式／license文字並拒絕HTML錯誤頁，不能以HTTP200假成功，也不繞過該file quota；改採其它可正常取得且授權明確的作者來源。

## E894：複雜Lua pattern不可直接內嵌PowerShell雙引號（2026-10-12）

public source research已用固定Lua取得作者兩頁與公開folder原HTML及SHA。後續唯讀-e查詢將含雙引號與方括號pattern嵌入PowerShell雙引號，shell parser先拒絕，未執行Lua或修改檔案。改獨立Lua檔保存pattern後執行，不把JSON.stringify或反斜線當shell quoting。world-route-scale-native-v2真24 Success／零warning，兩當輪原圖直接目視，接受紀錄另存。

## E893：道路世界單位設定新增反射欄位需正式refresh（2026-10-12）

world-route-scale-build-v1 ordinary真 FailedDueToEngineChange，12.13秒，原log保留。僅依該實際結果使用維護Lua refresh，再以未變C++跑fresh ordinary；新增設定／bridge欄位不授權手改Engine、BuildId、modules或cache。父項12/44，整合新測試尚未接受。

## E892：正式道路與單位快取必須包含世界尺度（2026-10-12）

world-unit-scale-native-v3已真23 Success／零warning，圖與來源接受紀錄獨立保存。接續source review發現正式route仍用32cm診斷寬，RouteHash只含原公開points；單位binding亦只含catalog／kind／owner，世界尺度變更時會重用舊幾何與舊顯示高度。新增明確WorldUnits道路寬、0保留cm模式，route快取納入scale並比較width／offset／material，unit binding納入scale；新增真正bridge publication／control／retire／pool／尺度切換測試。這是複核發現，未冒稱native failure。唯讀又猜錯FrameContract.cpp，inventory實為Generated/Public/OmFrameContract.h；不能在未知檔名查詢加假定路徑。

## E891：新比例fixture只同步VFX而未建立原生動畫生命期（2026-10-12）

world-unit-scale-build-v3 ordinary成功20.42秒；native-v2實際22 Success／1 Fail／零warning，Editor exit0不代表通過。WorldUnitVisualScale新增area／cone兩case被原生hero既有bNativeLifeEpochSet保護拒絕，三斷言失敗；長projectile及其它範圍通過。原因是新fixture只呼叫SynchronizeCombatFrame，漏正式bridge緊接的OnFrameState，未建立native生命期，並非放寬生產guard的理由。補完整frame／animation兩階段入口，再fresh build／native，不改舊失敗報告或圖。之前追加測試apply_patch上下文不符，工具拒絕零修改，已依實際行重新套用。

## E890：診斷尺寸上限會截短正式世界技能（2026-10-12）

world-unit-scale-refresh-v1真成功，未變C++ ordinary build-v2再成功2.55秒；native-v1真23 Success／零warning，正式camera90000cm原圖直接目視，角色及回城圈存在。後續複核發現Distance原10000cm clamp及VFX20000cm size／ribbon gate會截短／拒絕scale100的合法area／cone／trajectory；此為source review發現，不宣稱曾跑失敗native。改共用有界cm renderer契約，合法公開距離原值保留、越界明確fallback／拒絕，固定512pool不變。增加大半徑及長軌跡／越界測試。唯讀兩次將*.rs／OmPresentation*.h作Windows literal路徑導致缺檔，後續以rg目錄加-g glob查詢。

## E889：新增世界比例測試觸發 EngineChange gate（2026-10-12）

world-unit-scale-build-v1 ordinary NoEngineChanges／NoUBA／nocache 實際 FailedDueToEngineChange，9.67秒；保留原UBT log，依維護中Lua refresh後再跑未變C++ ordinary，不改Engine source／BuildId／cache。49 codegen tests真正成功。等待建置時查詢尚未落盤的UBT log及誤猜 OmAbilityCueRegistry.cpp 造成唯讀缺檔；實際inventory為OmVisualRegistry.cpp。後續依流程terminal／已存在artifact及inventory查詢。

## E888：原圖沿用必須逐張驗 SHA，不能預先假定全相同（2026-10-12）

world-v3 真23 Success／零warning，三world原圖已直接目視。接受helper嚴格比較發現 cast_secondary 與既有原圖SHA不同而拒絕，沒有輸出成功紀錄；其餘八authored與11combat相同。直接重新檢視當次 cast_secondary，角色／長槍／紅cloth完整、pose正確後才明確登記該張新目視，其他19張沿用舊目視證據。唯讀盤點又誤猜 OmCameraPawn.cpp，實際inventory為 OmRtsCameraPawn.cpp；後續檔名只從inventory取用。

## E887：重複座標必須共用同一份非退化道路路徑（2026-10-12）

build-v4 ordinary 成功且 exit0。後續程式複核發現，只有 segment 跳過重複點時，三個完全相同座標仍會產生孤立 join；這是複核發現，尚未宣稱原生測試失敗。先建立局部 Polyline 合併連續重複點，再讓 spline、segment、join 共用它，保留公開 RoutePoints 原始資料。新增完全相同三點零 segment／零 join、來源資料不變及有效路徑恢復斷言，接 fresh build-v5 與 native-v3，不修改舊證據。

## E886：直線道路segment在轉角留下楔形缺口（2026-10-12）

world-surfaces-build-v3 ordinary成功29.59秒；native-v2真正23 Success／零test warnings，三張當次world原圖全部直接檢視，曝光已較穩定，但route-corner可見兩直線立方體在轉角留缺口。這是通用polyline接合缺項，不接受為完整道路。補每條route一個共享圓形join ISM，依真mesh bounds正規化寬／厚，沿公開內部轉角放置，重複點不增重複instance，與直線段使用同world XY材質；reset／非法資料清除全部。無新actor／component per joint，無碰撞／導航，source幾何與PNG不改。測試新增join數與完整清除斷言，接fresh build-v4／native-v3與原圖。先前未知omoba-core/src/render唯讀路徑查詢無檔，後續只從實際rg inventory定位，不對缺檔做fallback修改。

## E885：Unreal RunTests 前綴擴展與地面曝光未校準（2026-10-12）

world-surfaces-import-v1 真成功：first4 saved、second4 verified／0 saved、九種原生拒絕零寫入，來源與四package不變。native-v1 Editor exit0、實際23項零Fail（22無warning、1 Success有3warning）；workflow因請求NativeMinimap同時選到NativeMinimapInput、清單僅22而拒絕。原summary仍false不改；下一fresh清單明列兩者。三warning來源是既有NativeMinimap fixture的DestroyActor缺world context，補成正式GEngine註冊／退休上下文。唯讀直接json.read native-report遇UTF-8 BOM也保留為查詢錯誤，後續按既有inspect helper先移除BOM。三張當輪world原圖已逐張目視：ground／route存在且shader正確，但草地曝光偏亮，不能作畫質接受。補缺燈場景的低priority native manual exposure與較低key／fill，已有authored方向光時同時不啟用native燈或曝光；新增真prelit場景測試。不改原PNG或來源貼圖，接fresh build-v3／native-v2。

## E884：世界表面使用舊版／推測API導致編譯失敗（2026-10-12）

world-surfaces-refresh-v1 原生C++實際失敗：GetStaticMesh 回傳 TObjectPtr 不能以 auto* 推導；SpawnActor 的 Transform overload 需要明確Class；TextureSource.GetSizeX 與 int literal 使 TestEqual overload 歧義。原log全部保留，未在建置活動中改source。等程序結束後分別改明確 UStaticMesh*、SpawnActor(StaticClass,Transform,Params)、直接布林尺寸比較。亦限制地面端點cm絕對值，避免有限但超出可呈現範圍的極端origin。繼續 maintained refresh-v2，再跑不改C++的fresh ordinary，尚無資產匯入或畫面接受。

## E883：世界表面新增來源觸發正式 EngineChange gate（2026-10-12）

world-surfaces-build-v1 ordinary NoEngineChanges／NoUBA／nocache 真失敗 FailedDueToEngineChange，7.63秒；UBT列出需更新引擎依賴（含 NetCore），完整原log保存。只依此實際結果執行既有 Lua refresh，再以未變C++跑fresh ordinary，禁止改Engine source／BuildId／modules／cache。建置後唯讀API複核也發現新測試的 TNumericLimits::QuietNaN 不存在，尚未進C++編譯即改用本repo已採用的 std::numeric_limits::quiet_NaN，不宣稱曾是compiler failure。PNG來源1254平方，使用Engine DecompressImage＋CreateTexture保留原Source並明確StretchToPowerOfTwo產生mips，避開TextureFactory全域NPOT設定；不改使用者ini。唯讀查詢再次出現未知AudioSource／Saika／MinimapTypes／TextureFactory路徑假設，實際inventory／header include已解析；未知路徑不得直接讀取。

## E882：材質 JSON 區域變數初始化警告（2026-10-12）

saika-cloth-build-v1 ordinary nocache 真成功128.53秒（refresh false），但新 commandlet 的 TryGetNumberField 短路分支引發 MSVC C4701 Roughness 可能未初始化。所有輸入仍經成功讀值才使用；為讓控制流程與編譯器檢查一致，Roughness／Specular／Number 明確零初始化，保留有限數值與範圍拒絕，不抑制warning。原log保留，接fresh普通build-v2，成功後才實跑資產流程。

## E881：接續時再次假設檔名造成唯讀查詢失敗（2026-10-12）

唯讀查詢曾假設 Generated/Private/OmHeroSaikaMagoichi.cpp、scripts/lib/json.lua、scripts/character_material_import.lua；實際分別為 OmContentClasses.cpp、tools/lua/lib/json.lua、scripts/import_character_materials.lua。未改錯檔，後續以 rg --files／rg -l 找到正式位置，確認產生結果含 NativeMaterialSections.Add(2)。接續盤點時也不應讀取未使用的 Grok skill；最新使用者要求自行完成，本輪無派發。所有後續檔案查詢必須從實際 inventory 或已確認路徑取用，不沿用猜測。

## E880：原生測試通過仍可能套錯來源材質（2026-10-12）

build-v12 ordinary nocache 成功17.29秒，native-v4 17 Success／0 test warnings，但直接檢視 idle、move、sniper_move、attack、critical、cast 六張原圖，外衣出現臉部 atlas。不可接受。唯讀 binary FBX 材質盤點 saika-source-material-structure-v1.json 證實 Material 03 無 texture connection，Diffuse=(1,0.01176470797509,0.01176470797509)；前兩材質使用 atlas。E879 的 section 覆蓋能力正確，但英雄內容需保留各材質原始語意。補通用、有界 indexed material_sections（支援 recipe inferred mesh）；新增自有純色材質產生器，完整 preflight、簽章／graph／shader驗證、第二次零儲存。不改原 mesh／原材料／原圖，不隱藏衣物。原生測試改逐 section 比對來源指定材質，不再錯誤要求三區共用同一貼圖。接 fresh build／兩次 material import／九張實圖，未完成前仍不接受。

## E879：整體材質覆蓋只迭代 palette，漏掉真正 render section（2026-10-12）

build-v11 ordinary nocache 真成功（9.92秒）；native-import-v8 首次4 imported／4 saved、第二次4 verified／0 saved，全原package與新片段SHA／mtime不變，逐幀root scale儲存前檢查通過。native-v3 仍15 Success／1 Fail：九片段root scale ratio均1.0、原500cm幾何檢查已通過，但每個片段 section2 material index2、asset palette僅2格，舊 GetNumMaterials 迴圈漏接該section，造成衣物預設材質。原圖／失敗保留。修正 full-mesh override 依全部LOD真實section＋LODMaterialMap收集索引，完整驗證16LOD／256sections／64slots邊界後才套用component override；不猜英雄名字、不改mesh資產或原palette。UE正式 GetMaterial 支援超過asset palette的component override，GetNumMaterials仍維持原asset定義，因此測試改驗實際每個section取得指定同一材質，不再把asset palette長度當override界線。需fresh build-v12與native-v4原圖確認。

## E878：Interchange animation-only 命名不自動加 _Anim（2026-10-12）

build-v10 ordinary nocache 成功（11.08秒，refresh false）；import-v7 原設定讀回通過並實際匯入第一個片段，但目的路徑檢查拒絕，0 saved，所有原package不變。log 顯示目的物件為 b01_ani_cast.b01_ani_cast。引擎 generic pipeline 在 animation-only 模式使用 DestinationName 原樣，只有 mesh+animation 模式才加 _Anim。改傳完整 Job.Name，不改 slot path 或移動舊資產；把骨架變更與 unexpected objects 錯誤分開，列實際返回路徑，避免混淆。FBX invalid bind pose warning 保留，不宣稱零warning。接 fresh build-v11／import-v8。

## E877：反射匯出省略 false，私人設定讀回因此拒絕（2026-10-12）

build-v9 ordinary nocache 真成功（10.46秒，refresh false，無新增C++ warning）；import-v6 零寫入拒絕 prepared translator differs: bConvertSceneUnit。原生 log 同時記錄 false 欄位空值無法 ImportText。FProperty::ExportText_Direct 在 Data 與 nullptr defaults 相同時可省略零值，未檢查回傳值造成空字串。改與引擎 SaveSettings 相同傳入 Data==Delta 強制完整值，並檢查匯出回傳值；仍以新 translator 全欄位讀回確認。沒有修改／儲存使用者或 Engine ini，原失敗保留。接 fresh build-v10／import-v7。

## E876：原始 translator 設定與此電腦 defaults 不同（2026-10-12）

build-v8 ordinary nocache 成功（11.59秒，refresh false），保留一筆 GetObjectsWithOuter 舊 bool overload C4996；改用 EGetObjectsFlags 正式 API。import-v5 成功讀出唯一舊 embedded settings，隨後因與目前 defaults 不同而零寫入拒絕。不能要求使用者改全域 defaults。依 Unreal 實際 LoadSettings 實作補每次匯入的獨立設定上下文：複製原 Editor config cache，載入目標原設定，只使用新私有 cache key，NoSave=true；同步匯入完成或失敗以 RAII 恢復原 GEditorPerProjectIni 並移除私有 cache。不呼叫 SaveSettings，不寫 Engine／使用者 ini，不改原 mesh。新 translator 再讀回全部設定逐項比對，不能生效仍在 preflight 拒絕。此是資產導向的設定接線，非固定 2.54 補償；接 fresh build-v9／import-v6。

## E875：舊 Interchange package 缺少新版 translator settings cache（2026-10-12）

build-v7 ordinary nocache 真成功（11.58秒，refresh false）；import-v4 仍零寫入前置拒絕。詳細診斷證實 translator 與 Actual settings 都存在，Expected=null，不能把 E874 的缺項推測當成已證實註冊問題。原 mesh binary 確實含 InterchangeFbxTranslatorSettings export 名稱。補唯讀舊資產相容路徑：新版 accessor 缺失時完全載入目標 package，只接受同 class 且唯一的原始 embedded settings；不存在或多筆仍拒絕。不以目前 defaults 冒充原匯入設定，不修改原 package。原失敗保留；接 fresh build-v8／import-v5 實證。

## E874：commandlet 的 Interchange translator／設定前置不可用（2026-10-12）

refresh-v2 實際成功（source-engine 242.30秒，內部ordinary21.56秒），fresh build-v6 ordinary nocache 真成功（2.55秒，refresh false）。import-v3 前置拒絕 translator settings unavailable，0 imported／verified／saved，既有packages SHA／mtime已驗不變。引擎 InterchangeImport 在 GEngine 尚未初始化時延後至 PostEngineInit 註冊；為 commandlet 明確使用標準 RegisterTranslator／RegisterFactory API（集合式可重入），不改全域設定。新增錯誤中的 translator／actual／expected class 明細以驗明實際缺項，尚未宣稱根因已解決。同輪再把 PowerShell -ErrorAction 誤傳給 rg 的唯讀錯誤記錄；後續缺檔可選讀取改用 Get-Content／Select-String 的正式參數，rg 只接受自身選項。原始失敗保留，接 fresh build-v7 與 import-v4。

## E873：Interchange editor 依賴變更觸發正式 EngineChange 門檻（2026-10-12）

saika-authored-build-v5 ordinary nocache 實際拒絕，UBT FailedDueToEngineChange／12.14秒；尚無新 Interchange C++ 成功。原結果保留。依維護 Lua refresh 流程建置後，再在 C++ 不變情況執行 fresh ordinary；不手改 Engine／BuildId／modules／cache。

## E872：既有 mesh 使用 Interchange，不能只支援 legacy FBX metadata（2026-10-12）

saika-authored-build-v4 ordinary nocache 真成功（38.14秒，refresh false）；native-import-v2 前置拒絕 target mesh FBX import coordinates unavailable or invalid，0 imported／verified／saved。資產實際含 InterchangeGenericAssetsPipeline，先前假設 UFbxSkeletalMeshImportData 不適用。修正為依實際 provenance 選擇匯入器：legacy 沿用 FBX 設定；Interchange 複製目標 generic pipeline，保持座標設定並只開動畫、指定原骨架，關閉 mesh／physics／material／texture 與骨架 curve metadata 寫入。mesh package SHA 納入簽章，translator 與既有設定不相同則在前置拒絕，避免默默使用不同全域設定。所有 root track frame 的縮放在 save 前嚴格對照原 ref pose，防止再次保存錯誤動畫。須 fresh build／import／native 後才可接受。另有唯讀查詢誤猜 OmWorldBridge.cpp、idle檔名及對 rg 傳入 PowerShell 專用參數；均未修改來源，後續先 rg --files 再取實際路徑，避免擴大到整個 Engine 的慢速掃描。

## E871：獨立 FBX 動畫強制單位轉換，與既有 mesh 不一致（2026-10-12）

saika-authored-native-v2 真實失敗；shader warmup 修正後 idle 已有來源色彩，但四個新增片段的 Bone_Root scale ratio 全為 2.540000，既有五個片段則為 1.000000。sniper_move 的 CPU 幾何仍超過 500cm。不能把 raw import success 當成顯示成功，也不放寬幾何上限。原因是匯入器強制 bConvertSceneUnit，未沿用目標 mesh 的 FBX 座標設定。V2 改讀目標 UFbxSkeletalMeshImportData，沿用轉換政策、單位、前軸、平移、旋轉及縮放；完整精度設定與 mesh 路徑納入簽章。新資產使用 AuthoredAnimationsV2，原錯誤 V1 資產及截圖保留。需 fresh ordinary build、兩輪原生匯入、拒絕案例及實際畫面驗證後才可接受。唯讀 git status 輸出過多已改採指定檔案查詢。

## E870：authored 原生畫面測試抓到幾何範圍與首張材質問題（2026-10-12）

saika-authored-build-v2 ordinary nocache 真成功（11.34 秒、refresh false）；native-import-v1 首次4 imported／4 saved，第二次4 verified／0 saved，全既有 RecipeV1 與新動畫 SHA／mtime 不變。八 native contract 實際拒絕、所有 package 不變。saika-authored-native-v1 Editor exit0，但 workflow false：15 Success、1 Fail、零測試 warnings；sniper_move CPU world X 範圍約529cm，超過幾何檢查上限500cm。九張當輪原圖全部已直接檢視，首張 idle 灰白，其他圖有來源紋理但部分衣服仍 checker。不能接受。原截圖與 log 保留；不放寬幾何上限。先補逐骨 scale／root 與材質 slot 診斷，並在 warmup render 後等待新排入 shader 編譯（既有角色 fixture 也有此步），加每張原圖來源彩色像素檢查；仍需確定根因及 fresh build／native。

## E869：TObjectPtr 不能以 auto* 推導為裸指標（2026-10-12）

saika-authored-refresh-v1 真實編譯失敗，UBT OtherCompilationError、24.72 秒；C3535／C2440 指向 `auto* UI=Factory->ImportUI`，UE5.8 此成員為 TObjectPtr。修正成明確 UFbxImportUI* 與 `.Get()`。原 refresh 失敗證據保留，不能當成 refresh 或 C++ 成功。source 修正後先執行 fresh ordinary；若仍有實際 EngineChange 拒絕，才再用維護 refresh，再跑 C++ 不變的 fresh ordinary。對同類成員使用已確認的實際型別，避免重複 auto* 誤用。

## E868：新增 authored commandlet ordinary build 真正遭 EngineChange 門檻拒絕（2026-10-12）

saika-authored-build-v1 ordinary nocache exit 1，UBT Result FailedDueToEngineChange、47.17 秒；原 summary／UBT log 保留，尚無新 C++ compile PASS。依既有維護流程，只有這個實際拒絕才啟用 maintained Lua refresh，完成後必須在 C++ 不變情況跑全新 ordinary nocache，不能手改 Engine、BuildId、modules 或 cache。E867 原 MCP 失敗保持，不以此次 build 遮蓋。

## E867：舊 recipe MCP 匯入依賴此插件版本不支援的工具（2026-10-12）

saika-authored-import-v1 實際失敗、cleanup_verified true、無對局。002-check_asset_exists.json 明確 isError=true／Unknown command type: check_asset_exists；get_pie_status 則成功。原 report／import log／errors.md 保留，未匯入任何新動畫。此前 commentary 先把缺失 ledger 推斷為本次阻擋原因，與原回應不符，已更正；未執行碰撞檢查，不能宣稱已證實目的路徑碰撞。改為原生 authored animation commandlet，所有來源與目標完整 preflight、缺少的四個動畫首次新增、簽章／骨架／真 duration 驗證、第二次零儲存且檔案雜湊／mtime 不變。既有 RecipeV1 不覆寫／不接管，骨架不儲存並檢查 ref pose／bone parent 全量不變。新 commandlet 尚未建置驗證。

## E866：唯讀 FBX 時序檢查的 magic 比較長度錯誤（2026-10-12）

inspect-authored-fbx-timing.lua 首次 exit 1，固定 sub(1,19) 與含兩個尾端空白的 20-byte magic 比較，錯拒合法 binary FBX。已依實際十六進位 header 修正為 #magic 長度；原始動畫、資產與設定未寫入。使用 binary node 邊界／深度／數量／檔案大小限制，陣列略過而不解壓，僅讀公開 AnimationStack timing。失敗不當成缺檔或動畫不可用。

## E865：native 測試成功但 cone 截圖被固定鏡頭裁切（2026-10-12）

runtime-build-v2 ordinary nocache 真成功（137.50 秒、refresh false），runtime-native-v1 15 exact tests Success／零測試警告，11 PNG 存在。但逐張實際檢視 cone 與 cone-fx-only 發現固定 OrthoWidth=700 裁掉部分輪廓，因此本輪不接受完整視覺證據。修正截圖 fixture 依三組 ISM 有效 instance 的真世界四角，加上完整角色高度取包圍範圍，自動設定 focus／orthographic width，並以四像素邊緣沒有彩色特效檢查防止再裁切。不是縮小實際技能範圍。原 PNG／native log 保留。Editor 啟動仍有外部設定 HTTP401，零測試警告不等於整份 Editor log 零警告。下一輪須重新正式建置與原 15 native／11 PNG。

## E864：動畫 payload 缺少事件時序，戰鬥呈現編譯失敗（2026-10-12）

combat-vfx-runtime-build-v1 ordinary nocache 真實失敗，UBT 203.30 秒，C2039 指向新增程式誤用尚不存在的 FOmAnimationStatePayload.Tick。原 summary、UBT log 保留，無 refresh、無 native PASS。修正是在 Blueprint payload 正式加入 Tick／Sequence，並從現有 ABI OmAnimationStateRecord 的公開欄位傳入；不改 ABI、不使用牆鐘或推測時間。另查詢曾再度猜測不存在的 UECPCore.cpp、UECPUI 目錄、ensure_bp_generator_animation_fix.lua 與 PowerShell literal glob，全部只是唯讀失敗；正確來源已由 rg 找到，後續只使用實際檔案路徑。修正後必須 fresh ordinary build 與 native 驗證才可接受。

## E863：戰鬥呈現測試的輔助 tick 未同步，以及重複猜測檔案路徑（2026-10-12）

原始新測試在推進 pool 至 tick 1000 後，Health helper 仍讀取舊 Tick，會先造成不必要的倒退。建置前審查已改為先設定共享 Tick=1000，再呼叫 Health；倒退拒絕斷言仍保留。另一次唯讀查詢直接猜測 OmPresentationCueHistory.cpp 而失敗；實作其實是已存在的 .h 內聯，應先搜尋檔案清單再讀取。上述修正尚未取得新的編譯或 native PASS。既有 AbilityCastCue fixture 改用受支援的 Editor world，以驗證真 bridge 到 pooled Recall 的啟停，不能以不建立呈現子系統的 EditorPreview 代替。

## E862：材質 commandlet 啟動仍有外部設定 HTTP timeout（2026-10-12）

- `combat-vfx-material-import-v1/first-engine.log` 實際有四筆HTTP15秒timeout，分別是BpGeneratorUltimate外部free-tier／provider／agent／sound設定請求；三份材質graph／shader與兩次零重寫驗證仍成功，不能宣稱整個Editor log零warning。
- 這些不是shader編譯失敗，也沒有據此修改材質或假造成功。保留原log，後續檢查commandlet是否不必要地初始化雲端設定，應在未使用相關功能時避免啟動連線；不得為消掉warning修改Engine或填造遠端設定。

## E861：新增 VFX commandlet 後 ordinary build 被 EngineChange gate 拒絕（2026-10-12）

- `combat-vfx-material-build-v1` 實際workflow exit1、UBT exit5，Result為 `FailedDueToEngineChange`，17.14秒；不是commandlet編譯或shader成功。完整build／UBT log與summary保留。
- 已有實際EngineChange證據，接下來僅用維護中的Lua `--refresh-engine` 流程；不手改Engine／BuildId／modules／cache。refresh後保持C++／Build.cs不變，再跑fresh ordinary NoEngineChanges／NoUBA／nocache，兩者都成功才驗資產。
- E859常數已改明確float，但本次在gate前拒絕，尚不能宣稱該warning已經過編譯重驗。VFX shader／資產／pool仍pending；已接受的Recall範圍與紀錄獨立保留。

## E860：接受 helper 的來源清單誤把 transport 放進 runtime（2026-10-12）

- `accept-native-recall.lua` 第一次執行實際 exit1，hash輸入不存在：`omoba-core/src/runtime/transport.rs`。檔案實際在 `omoba-core/src/transport.rs`；原生十Success／Rust／46圖結果沒有失效，但接受 summary 尚未產生，不宣稱已接受。
- 改正為先以實際符號定位的檔案，重跑前核對整份來源清單檔案存在。失敗記於 `.ai-collab/native-recall-accepted-v1-failure.md`，改用fresh v2接受ID；不能靠省略該來源繞過hash綁定。
- 該核對的初次inline Lua命令在PowerShell引號解析階段被拒絕，Lua未執行。改存獨立Lua檔再呼叫，避免將複雜字串／pattern混入shell引號。

## E859：全量受影響編譯揭露既有標記色彩常數的 float 警告（2026-10-12）

- `native-recall-build-v1` 編譯 `OmCommunicationMarkerPresentation.cpp:65` 實際出現三筆 C4305；`FLinearColor` 的0.025／0.035／0.045以double常數傳入float。這是建置 warning，並非 native runtime warning 或 Recall 編譯失敗。
- 保留當次 log，不在 active C++ build 中修改來源。下一個來源批次將常數與alpha乘數明確寫float，再以fresh普通建置驗證；不把此次建置宣稱零warnings。

## E858：檢查腳本時仍直接猜檔名，造成無效查詢（2026-10-12）

- read-only 查詢使用不存在的 `scripts/ue_bridge_build.lua`／`scripts/build_ue_bridge.lua`，並把 evidence 路徑多寫 `run-` 前綴；未修改檔案，沒有作為驗收證據。
- 改用 `rg --files` 先定位，確認 bridge stage 實際由 `omfue/build_bridge.bat` 執行。搜尋限實際目錄與檔名條件，避免整個 `.ai-collab` 輸出超量。與既有 E851／E853 相同防錯規則，後續不得重複猜路徑。
- 同輪也曾誤查不存在的 `compile-current-abi-header.lua`，實際定位到 `verify-summoner-c-header.lua` 並成功執行 ABI26 smoke。另一筆 `rg` 查詢含未跳脫左大括號而被拒絕，改成多個 `-e` 精確符號查詢；以上僅 read-only 查詢錯誤，沒有寫入或當作成功證據。
- PowerShell傳給`rg`的檔案路徑glob也曾被當成literal而拒絕；正確寫法是實際目錄加`-g`篩選。此規則已存在，後續命令須在執行前直接核對，不再依賴shell展開。
- UE材質header曾沿用舊 `Engine/Classes/Materials` 路徑；實際5.8已在 `Engine/Public/Materials`。後續以 `rg --files` 精確檔名先定位，再讀header，不沿用舊版本目錄慣例。

## E857：權威回城 producer 測試誤用未匯入的型別名稱（2026-10-12）

- `channel-visual-rust-v4` 實際 exit1，首個 core test build 發生 E0425：`ScriptUnitTag` 不在 moba_match 模組作用域。原始 log／失敗 summary 保留，未宣稱通過。
- 改為實際 public 路徑 `crate::scripting::ScriptUnitTag`。新增測試應確認每個型別的定義／匯入，不能從其他模組的 glob 推斷。先前 source 檢查也修正 codec 常數名稱為實際 `SCHEMA_ID`，該筆在建置前發現。
- 此測試以真 Recall／Stop 指令及正式 `build_wave_b_read_view` 驗證當前 disclosed baseline，不增加權威對局數；須 fresh 重跑後才接受。

## E856：同一 patch 重複宣告修改同一檔案，被工具原子拒絕（2026-10-12）

- 回城 bridge 測試 patch 的 `driver.rs` 出現兩個 Update File 區塊，apply_patch 拒絕；該批沒有寫入，後續 ABI 修改腳本也尚未執行。
- 已將同一檔案的多處 hunks 合併到單一 Update File，並移除沒有實際變化的 hunk。先確認實際狀態再重試，不把拒絕當成部分完成。Driver 使用其實際匯入的 `EntityKind`，不沿用 projection 私有的 `SimEntityKind` alias。

## E855：原生測試全綠仍有巨大診斷線框遮住施法／受擊畫面（2026-10-12）

- `native-lifecycle-native-v2` 實際十 Success／零warnings／Editor與workflow皆0，但逐張原圖審查 Date 的 cast／hit／death／respawn 發現基底 callback 繪出的青色與紅色 DrawDebugSphere；同步 fixture 沒有經過正常秒數清除，線框會累積，正式遊戲也會短暫顯示這些診斷圖形。
- 新增明確 `bShowAnimationCueDebugShapes` 預設false，只控制attack／ability／damage的診斷繪圖；保留事件、原生動畫與Blueprint自訂呈現入口，開發者可選擇開啟。沒有靠capture隱藏全部debug showflag或後製修圖，沒有把診斷球當完成的VFX。
- v2數值測試通過仍只保留為診斷證據，未接受視覺；fresh普通建置、同十測試與40原圖重驗後才接受當前動畫範圍。完整正式combat VFX仍列6.4 pending。

## E854：擴大生命期驗證揭露 fixture 的素材假設已過時（2026-10-12）

- `native-lifecycle-native-v1` 真 Editor exit0，但十測試只有八 Success／兩 Fail／零 warning；workflow 正確拒絕。六角色組裝與新生命期測試已過，不能以程序0把整組當成功。
- AbilityCastCue 原 caster catalog1 是 Saika RecipeV1，沒有 cast／death／respawn clip，新增的端到端動畫斷言因此得到 None／idle。改用有完整生命期綁定的正式 generated Vanguard catalog 常數；不替 Saika 偽造素材或改正式 hero ID。
- NativeHeroPresentation 原本假設 Luminary 永遠沒有美術，與已加入的真素材衝突。保留「缺少 optional mesh 使用 fallback」契約，fixture 明確清空該 optional 設定再驗，不刪斷言、不讓正式 Luminary 退回方塊。
- 失敗原報告／40張當次 captures 保留，待 fresh build 與全部十項原生重驗後再做視覺接受。Saika 原 authored／RecipeV1 的擴充生命期不由此六角色範圍宣稱完成。

## E853：AttackPhase 身份欄位誤寫 Entity，實際為 Attacker（2026-10-12）

- `native-lifecycle-refresh-v2` 實際 exit1／OtherCompilationError，OmHeroActor.cpp 三筆 C2039；新增身份守衛誤將 animation snapshot 的 `Entity` 用在 attack phase。
- 已讀取正式 `FOmAttackPhasePayload` 宣告，使用 `Attacker` 比對並傳入保存的 animation snapshot。不得用刪掉身份守衛或接受所有 generation 消除編譯錯誤；fresh build／native 還未通過。原失敗 logs 保留。

## E852：生命期新增檔案後 ordinary 建置觸發 FailedDueToEngineChange（2026-10-12）

- `native-lifecycle-build-v1` 實際程序 exit 1，UBT exit 5／FailedDueToEngineChange；原始 build.log 與 UBT-Log.txt 保留，未啟動 Editor 或宣稱通過。
- 依正式流程使用 maintained Lua refresh，完成後再跑獨立 ordinary NoEngineChanges／NoUBA／nocache。不得手改 Engine source、BuildId、modules 或 cache 以避開檢查。
- 首次 refresh 實際成功 exit0／268.65秒；其後加入 hidden 回收 override／生命期 tick 下限，`native-lifecycle-build-v2` 再次實際被 Engine modules／NetCore 產物檢查拒絕。兩次不是 C++ compile error，也不以 refresh 成功替代 ordinary 成功；完成所有當前 C++ 審查修正後，再 refresh 並在來源不變下跑 ordinary。

## E851：生命期修改在審查時使用了不存在的欄位與路徑（2026-10-12）

- 未編譯前審查發現錯用 `ActionInstance`／`Locomotion`；正式 payload 為 `ActionInstanceId`／`LocomotionState`，已依實際宣告修正。沒有將未建置版本記為成功。
- 讀檔也曾猜測 `omfue/Source` 與 `OmBaseUnitActor.h`，實際 runtime 位於 `Plugins/OmRuntime/Source`，基底為 `OmUnitActor.h`。後續先用 `rg --files`／實際宣告定位，再讀取或修改；避免大量 git status 與單行 JSON 輸出掩蓋診斷。

## E850：首張截圖早於貼圖 residency，shader map完成仍出現灰白（2026-10-12）

- `character-assembly-native-v4` 六角色idle／move／attack比例及頭部全過，材質resource／compiled／shader_map均1；只有Date與Luminary首idle chromatic pixels為0／6，下一張attack分別27500／27888，來源貼圖未改。首次載入的Ranger服裝也在idle灰白、attack上色。
- 同步automation fixture沒有正常game frame的streaming回饋；capture先對角色實際mesh要求PrestreamTextures，warmup一次真scene capture，再有界等待StreamAllResources與shader／render commands完成，才保存PNG。此為驗收取景同步，不在正式每entity每frame加入阻塞／全域等待、不放寬色彩門檻。
- 原v4失敗圖與native結果保留；fresh ordinary build／native／原圖重驗後才能接受材質畫面。

## E849：原始匯入動畫已遺失 reference root 的比例與旋轉（2026-10-12）

- `character-source-pose-native-v2` 真正原始來源TPose高度1.8287公分、idle1.7747、jog1.5167、SwordIdle1.6046；來源reference root為scale100／yaw180，但原始animation root是scale1／identity。問題在IK之前已存在；V3單純normalize原raw動畫仍餵入錯誤比例／方向。
- V4 plan明確指定reference clip，從其root與來源mesh reference root計算校正transform，對所有source clip root套用同一座標轉換，再建立公分計算副本；不硬寫100或180、不改來源FBX／uasset，不把全英雄套同一骨架。signature包含校正clip身份。
- 原始來源對照capture使用實際posed bounds調整鏡頭取景（actor不縮放）；其1.8公分原始量測仍明示，不因放大取景宣稱來源動畫已正確。V4仍須fresh轉換／reload／實際尺寸與原圖驗證。

## E848：來源對照 fixture 直接生成 abstract hero，未檢查 null outer（2026-10-12）

- `character-source-pose-native-v1` exit3：SpawnActor拒絕abstract OmHeroActor，後續NewObject SkeletalMeshComponent的outer為null而fatal。未取得來源播放證據。
- 來源診斷改用具體AActor與明確root skeletal component，先斷言actor存在再建立component；capture helper接受AActor。不變更正式hero的abstract限制，原crash／log保留。

## E847：V3實際播放仍有明顯姿勢塌縮，停止推測並補來源對照（2026-10-12）

- `character-assembly-native-v3` reference比例門檻正確拒絕，多數idle／move／attack只有85–130公分，原PNG仍灰白且四肢異常，不能接受；V3轉換資料success亦不代表變形正確。
- 新增原始 animation_library mesh＋四個原始clip的真正CPU skin、root/ref transform與scene capture，直接比較原始匯入與重定向，定位錯誤從哪一階段開始。保留原失敗與圖片，不再以靜態bounds或猜測比例推導成功。

## E846：計算副本的 outer 使用抽象 UObject，result 成功但程序拒絕（2026-10-12）

- `character-centimeter-evaluation-six-v1` result寫success／276saved，但實際程序exit1，UObjectGlobals abstract-class ensure。正式runner正確拒絕，不能用result success覆蓋程序失敗。
- 將計算副本 outer 改為每target獨立RF_Transient的 /Temp package，保留強引用且不保存計算package。既有V3產物用fresh獨立Editor重新verify／zeroSave，原exit1完整保留。
- V2原圖只有灰白材質，source貼圖實際有膚色；scene capture新增等待GShaderCompilingManager真正完成（FAssetCompilingManager不包含全部shader），避免在fallback shader尚在使用時接受原圖。

## E845：寬泛尺寸門檻讓塌縮 pose 通過，原圖審查仍拒絕（2026-10-12）

- `character-assembly-native-v2` 兩native成功，但原PNG角色擠成矮小形狀，CPU skin高度52–71公分，未達原始156–187公分比例；不接受組裝或視覺範圍。改以每個mesh的實際reference height比例檢查，而非50–300公分寬泛範圍。
- UE FResolvedRetargetPoseSet去除local/global scale後，仍以未換算的local位移重算global；僅在batch之後補scale不夠。RetargetV3使用transient公分reference mesh／source skeleton與source clip計算副本，保留世界reference位置、rotation、動畫global位置，scale歸1後重新求local；batch輸出再轉回真正target reference units。
- 計算副本強引用覆蓋batch生命期，不保存、不改寫原source skeleton或mesh；V1/V2原產物與失敗PNG保留。仍須真正轉換、reload、比例與原圖審查。先前maps.lua猜檔名與未完成build log讀取也違反E839規則，後續只讀已發現的路徑／完成receipt。

## E844：IK batch 移除骨骼 scale，靜態 bounds 驗證漏掉播放後百倍縮小（2026-10-12）

- `character-assembly-native-v1` math test 成功，但實際六角色 idle／move／attack CPU skin 高度只有1.327–1.841公分，root scale1；頭部與capture也失敗。原PNG與log保留，不接受該視覺範圍。
- UE5.8 IKRetargetBatchOperation.cpp 明確 strip source component scale。新 RetargetV2 由輸出公分 component pose 與目標 reference hierarchy scale 重建 local animation tracks，包含 child translation 的 parent-scale 換算；非uniform／異常scale明確拒絕，不對角色actor硬加100倍。
- V1歷史資產不覆寫。V2需重新真正轉換、zeroSave與實際CPU skin／PNG驗證。測試world另補正式Engine context，修正DestroyActor no-context warnings。
- 本次仍有兩次讀檔規則違反（猜IAnimationDataController.h的Public位置、rg Windows wildcard路徑）；後續以發現的Classes實際路徑與rg -g讀取。

## E843：增加 RenderCore 依賴後普通 gate 回報 EngineChange（2026-10-12）

- `character-assembly-build-v3` 實際 FailedDueToEngineChange／exit1；不是 C++ 失敗。依既有維護入口執行一次 refresh，再跑 fresh ordinary NoEngineChanges／NoUBA／nocache；不手改 Engine source、BuildId 或 modules。

## E842：原生截圖使用 RenderCore API 卻缺少直接模組依賴（2026-10-12）

- `character-assembly-build-v2` C++ 編譯成功但 link 失敗，FlushRenderingCommands 為 LNK2019；RHI 並不提供此 RenderCore 符號。
- OmEditor.Build.cs 加入直接 RenderCore 依賴，維持真正 scene capture／render flush 驗證，不刪掉檢查規避問題。保留 v2 原失敗，fresh ordinary v3 重建。

## E841：驗收場景使用不存在的 directional light accessor（2026-10-12）

- `character-assembly-build-v1` 實際 exit1，UBT 報兩個 C2039：ADirectionalLight 沒有 GetDirectionalLightComponent；不能當作資產或動畫失敗。
- 讀取 UE5.8 DirectionalLight.h 確認公開 accessor 是 GetComponent，修正兩處後重新普通建置。編譯完成前不啟動 Editor、不修改 C++。
- 續接時又猜測不存在的 error helper 檔名；之後先完成檔案搜尋才讀取，沿用 E839 的規則。

## E840：未確認本次建置完成就修改 native 驗證來源（2026-10-12）

- `character-assembly-build-v1` tool session仍未完成／summary尚未寫入時，新增了assembly test的全12動畫slot解析檢查。即使僅測試檔，仍違反建置期間來源固定要求；本次不能直接綁定為最新測試來源已驗。
- 等本次build終止後，再跑fresh ordinary build-v2；期間不改C++／Build.cs。工具協作狀態明確保存active_cpp_build，之後CPP patch先檢查該狀態，僅在已取得實際exit與summary後清除。

## E839：重複猜測檔名與 UE5.8 舊路徑，造成無效讀取（2026-10-12）

- 本次曾讀不存在的OmUiTextureImportCommandlet.cpp、OmNativeAnimationTests.cpp、OmEventPayloads.h、bridge/presentation_bridge.rs及舊Classes/Materials/Material.h；實際位置分別由符號搜尋找到，沒有修改或建置不存在檔案。
- 檔案發現與依結果讀取是相依操作：先單獨rg --files／rg符號，看到真實路徑後才讀取；不得在同一批獨立命令裡接著猜檔名。bracket／空格路徑使用LiteralPath或Lua path.read。


## E838：新增 automation test 名稱未遵守維護 runner 的命名範圍（2026-10-11）

- `run-native-subset.lua character-attachment-native-v1 Omoba.Visual.NativeAttachmentReferenceScale` 在第21行參數檢查失敗，尚未建立evidence目錄或啟動Editor，不能當成native test失敗或成功。
- 維護runner只接受Om.Runtime／Om.Editor／Om.Generated；新增測試改為Om.Runtime.NativeAttachmentReferenceScale，沿用正式runner，不擴大命名白名單或避開驗收契約。重新建置後用fresh run執行。


## E837：把有100倍reference scale的骨骼局部座標誤當公分（2026-10-11）

- `character-rigid-parts-v2` 實測male_head有2519 vertices／4232 triangles，但pivot reference scale約100，直接完整inverse得到local size0.181×0.231×0.223；來源世界mesh仍正確約182公分。這不是缺頭或需要放寬幾何門檻。
- 改以移除scale的rigid pivot產生真正公分幾何，結果標示coordinate policy與requires_reference_bone_scale_normalization；consumer從各目標mesh的實際ref hierarchy計算reference scale，補償附掛scale及translation，不硬寫0.01、不修改source skeleton／inverse binds。
- 相同規則處理不同unit roots；不支援FTransform無法表示的nonuniform scale加旋轉shear時明確拒絕，不靜默近似。重新native輸出／zeroSave／實際附掛驗證後才能接受。


## E836：頭部權重擷取被幾何／尺寸門檻拒絕，首版診斷不足（2026-10-11）

- `character-rigid-parts-v1` native preflight Extract拒絕：empty or unreasonable centimeter bounds；imported／verified／saved均0，所有來源及輸出未寫入。首版錯誤無角色ID／統計，不能推斷為來源缺頭或放寬門檻。
- 補id、vertices、triangles、bounds validity／實際size及pivot transform，fresh build後用實際診斷判斷原因；未降低最小幾何或尺寸要求。


## E835：誤讀上次 UBT 成功 log，在當前建置未完成時啟動 Editor（2026-10-11）

- `character-material-import-v3` 尚未進入commandlet，即OmRuntime DLL讀取GetLastError=32（sharing violation），ModuleManager assertion／exit3。當時 `character-presentation-build-v1` session仍執行，先前讀到的是舊UBT log尾端；原兩份log保留。
- 必須確認本次build tool session真的exit0，且本次summary.json已寫成功，才啟動任何Editor／commandlet；既有binary metadata preflight是readiness檢查，無法保證沒有進行中的link。不可用舊Log.txt尾端替代本次完成證據。
- build-v1完成後另修UE5.8 deprecated material usage field，改GetUsageByFlag／SetUsageByFlag，重新普通gate完成才重跑；不在build進行中修改C++。


## E834：匯入後直接更改 SRGB 與非同步貼圖 source read 競態（2026-10-11）

- `character-material-import-v2` 實際處理male roughness G8時，Background Worker assertion：MipView.GammaSpace != LayerData.SourceGammaSpace（TextureDerivedDataTask.cpp:423），exit3；crash／完整log保留，尚未進入全批保存。
- factory階段先指定color／linear與compression／LOD／mip政策；取得texture後先用FTextureCompilingManager完成該貼圖的既有工作，才調整政策並重建，再等待該貼圖完成。避免非同步source read期間改SRGB，不改Engine、不刪cache、不把所有shader工作混進逐texture等待。
- 需fresh普通建置與真正匯入／獨立重載成功才接受；v1無shader resource與v2 crash均保留失敗狀態。


## E833：材質 commandlet 未允許 rendering，無可驗證的 shader resource（2026-10-11）

- `character-material-import-v1` 實際28 textures驗過但第一個eyes材質無shader resource，success=false、saved=0；原result與log保留，所有資產延後到完整驗證才保存。
- 依本機官方App.h／Material.cpp，commandlet預設CanEverRender=false，CacheResourceShadersForRendering跳過resource建立；runner補正式 `-AllowCommandletRendering`，後續以實際resource／compile errors驗證，不改Engine、不跳過shader檢查。
- 尚未以此旗標通過前，不把這個原因推論當實測成功；新run需獨立重載及零寫入。


## E832：材質 shader 驗證直接使用 RHI 全域但未宣告模組相依（2026-10-11）

- `character-material-refresh-v1` C++編譯通過但link實際LNK2019：GMaxRHIShaderPlatform缺RHI模組；原log保留。
- OmEditor.Build.cs補明確RHI相依，維持實際材質resource與shader compile error檢查，不以metadata取代。等待建置完成後才改signature，重新fresh ordinary gate。


## E831：新增材質 commandlet 後 ordinary gate 實際要求 refresh（2026-10-11）

- `character-material-build-v1` 實際 FailedDueToEngineChange，UBT-Log／build.log保留。依既有維護Lua refresh後必須fresh ordinary NoEngineChanges／NoUBA／nocache通過。
- 期間不改C++／Build.cs，不手改Engine或modules metadata，不把refresh成功當ordinary gate通過。


## E830：素材 staging 呼叫不存在的 Lua path.dirname（2026-10-11）

- `stage-character-material-inputs-v1.lua` 第一份 license 尚未寫入即因 path.dirname 不存在失敗；既有庫提供的是 path.parent。先讀 API 再用，不新增 shell fallback。
- 僅對本次建立的兩個已確認空目錄使用 lfs.rmdir；若有任何檔案即拒絕，沒有 recursive delete。原資產及 archive 不變。


## E829：原生動畫重定向有實際 warning，不可宣稱零警告（2026-10-11）

- `character-retarget-male-v1` 原生轉換43 clips／46 packages成功，獨立重載零寫入；首次136 warning行（含引擎結尾摘要重列），第二次0。六角色批次首次484行，第二次8行，均保留完整log。
- UE5.8 官方 batch exporter 對沒有 float curves 的來源寫 warning；骨骼動作仍須獨立檢查。另有動畫 import-data ConditionalPostLoad 相依警告，以及 BpGeneratorUltimate 雲端設定 HTTP timeout（第二次8行為4筆加摘要），不能混成動畫骨架錯誤或略去不報。
- `PrecomputedVisibilityWarning:0` 是 CVar 名稱，不是 warning；新的 inspector 只計實際 `: Warning:` log。不刪警告／不改 Engine，不以資料驗證代替實際角色變形、材質和渲染驗收。


## E828：UE5.8 JSON object key 已改為 SharedString（2026-10-11）

- `character-retarget-refresh-v1` 實際 C2039／C2664：FJsonObject::Values key 不再是 FString，直接 StartsWith／Contains 或拿 FString 查 map 均不成立。原 UBT 與 build log 保留。
- 依本機官方 header 將遍歷 key 顯式轉 FString，查欄位用 HasField(FStringView)，不修改 Engine 或降低來源 SHA／ownership 檢查。


## E827：新增 IKRig 模組後 ordinary gate 要求 metadata refresh（2026-10-11）

- `character-retarget-build-v1` 實際 FailedDueToEngineChange；原 build.log 與 UBT-Log.txt 保留。只依實際 gate 使用維護中的 Lua refresh，之後重新跑 ordinary NoEngineChanges／NoUBA／nocache。
- refresh 執行期間不改 C++／Build.cs，不手改 Engine source、BuildId、modules 或 cache。


## E826：FBX 匯入未啟用 scene unit conversion，角色變成1.8公分（2026-10-11）

- `character-source-import-v1` 實際7個65-bone來源、43動畫、57package與第二次零寫入全過，但檢查 native bounds 發現身體 height約1.8（UE公分），不是約180公分；原始FBX UnitScaleFactor=100，factory預設未換算。
- 通用修正為 skeletal 與 animation import data 均啟用 `bConvertScene`／`bConvertSceneUnit`；不靠每個角色臨時放大100倍。以新QuaterniusV2資產namespace與metadata版本重匯入，V1原結果保留。
- plan／result固定coordinate policy，驗收增加50–500公分humanoid height界限與錯誤單位回歸；原始FBX bytes不改。正式映射仍須實景及rest-pose／retarget驗證。

## E825：官方來源 FBX 的 bind pose／smoothing 警告不可當成乾淨素材接受（2026-10-11）

- `character-source-import-v1/first-engine.log` 實際238筆 warning，包含 invalid bind pose、FBX SDK重建bind pose及缺少smoothing groups；second驗證0warning只表示載入已保存資產，不代表原始來源沒有問題。
- 原檔與所有警告保留；source import通過不勾世界畫面或角色驗收。後續需看實際變形、法線、材質和retarget結果，不能只靠相同65個bone名稱斷言可用。

## E823：角色匯入新增編譯輸入後 ordinary gate 實際要求 engine metadata refresh（2026-10-11）

- `character-source-build-v1` ordinary NoEngineChanges／NoUBA／nocache 實際失敗 `FailedDueToEngineChange`；原 UBT-Log 的 engine modules 清單與結果保留。
- 僅依這次實際失敗使用維護中的 Lua `--refresh-engine`，完成後必須 fresh ordinary gate；不手改 Engine source／BuildId／modules／cache。refresh 中不修改任何 C++ 或 signature。
- 新 importer 在第一次建置後、refresh 前補上顯式 Package include 與磁碟既有 package／registry 一致性檢查；未以 import metadata 覆蓋 foreign 或未讀到的資產。

## E824：新素材 durable manifest 曾寫入本機絕對路徑（2026-10-11）

- preflight 審查發現新 manifest 的 FBX／license 路徑固定在 D 槽，換 clone 根目錄會失效；尚未匯入或接受。
- durable manifest 改存 workspace-relative 路徑，正式 Lua validate 依實際 clone root 解析，只在當次 commandlet plan 使用絕對路徑。來源 FBX 與授權 bytes／SHA 保持不變。

## E822：以段落前綴當作整行 patch context 造成文件更新失敗（2026-10-11）

- 更新 handoff 時只提供第一行的前綴，與實際完整長段落不符，apply_patch 拒絕，未修改檔案。
- 長段落追加狀態採固定 Lua binary 寫入並保留 CRLF；patch context 必須使用實際完整行或獨立標題，不猜測行內容。

## E821：新增通知模型誤解正式名冊排除自己的契約（2026-10-11）

- `communication-audio-owned-v4` 有界診斷顯示 ready roster=1；查閱 `ValidateRoster` 確認正式 API 已驗兩隊完整 scoreboard，回傳其他玩家供 mute UI 使用，刻意排除本機玩家。
- 新模型錯誤要求兩筆以上且包含本機玩家；原 native fixture 也自行建立 `{1,2}`，與正式 API 不符，因此單元通過卻實際 receiver 永遠 reset。E819 只是 readiness 假設，不是此次根因。
- 修正模型直接接受正式 peer-only 1–9 人名冊，在驗證唯一、正數、無自己後內部加入本機玩家，保留既有 UI API。測試改用正式 `{2}` 並新增本機公開事件、錯誤 self-inclusive／duplicate／empty 名冊拒絕。
- 後續新增消費端先讀 producer 契約與呼叫端，不能用自造 fixture 的型態假定 API 語意。
- fresh ordinary build-v8成功55.94秒；native-v2 exact14 Success／0warning；owned-v5實際83cases／49TCP intents／15stages與production字幕斷言、原圖、cleanup全過，根因修正已有實際驗證。

## E820：只憑 source 假設修正，實際通知字幕仍失敗（2026-10-11）

- `communication-audio-owned-v3` 在接收通知字幕斷言再次失敗；E819 readiness 修正尚不能證明是根因，保留原失敗，不放寬文字、時間或成功條件。
- 接續以 revision 變更與實際 notification 的有界診斷確認名冊、內容、時鐘、audio scope 與 caption，逐段定位，而非繼續猜測。
- 讀取來源先用 `rg --files` 核對路徑；本次再次猜錯 test 與 generated actor 目錄，已改用實際清單定位。

## E819：receiver 名冊尚未 ready 時被 UI revision 快取永久略過（2026-10-11）

- `communication-audio-owned-v2` 新增真正收到表情→production caption widget 斷言後，在第64個案例（signals-received-800）失敗；未把先前純 model14 PASS 冒稱實際 receiver 成功，原失敗與清理證據保留。
- source 審查：audio roster acquisition 位於 UI `CommunicationLastRevision` early return 後。首個 ready view 可能早於可讀完整 scoreboard frame；當次取得名冊失敗後，revision 不變便不再重試，下一批訊號才初始化水位而全部當成重連歷史略過。
- 通用修正：接收通知獨立於 UI rows rebuild gate，使用同 scope 的已驗證小型名冊快取與0.2秒有界 readiness retry；暫態 lease 失敗不退休已驗證 receiver，真正 bridge／scope 變更才清除。mute receipts 仍先於通知處理，不直接呼叫提示偽造成功、不延長字幕來遮掩缺接線。

## E818：補強錄音診斷時誤用不存在的 Device 名稱與 Fail 參數型別（2026-10-11）

- `communication-audio-build-v3` ordinary 實際 C2065／C7595／C2664 失敗；原 UBT-Log 保留。fixture 成員為 `Mixer`，Fail 接收 `const TCHAR*`，不是 FString。
- 修正為已存在的 `Mixer->GetPrimaryVolume()` 與 `*FString::Printf(...)`，不改 amplitude／時間門檻；fresh ordinary build-v4 與 serial audio output 驗證。
- 擴增診斷也先核對本檔成員與既有函式 signature，不能從其他 fixture 猜名稱。

## E817：實際錄音與會搶焦點的設定 GUI 驗收並行，錄到靜音（2026-10-11）

- `communication-audio-output-v1` 在 native14／實際視窗 BringToFront 同時執行時，routed-mix amplitude 為靜音而失敗；原 native report／engine log 保留。當時診斷只報 nonzero mismatch，未寫出失敗 WAV，因此不能斷言是 focus、裝置或資源競爭中的哪一個原因。
- 同一份產品來源、相同參數，在 native14 terminal 後 serial `communication-audio-output-v2` 真正通過三段 PCM／通知音效／字幕／回城／pool，沒有提高時間或 amplitude 容差，也沒有重設系統音量。
- 錄音 gate 與會改視窗焦點的 Unreal GUI gate 必須順序執行。補強 fixture：amplitude 失敗前也保存原始實錄 WAV，記錄 stage／peak／sample count／FApp volume／device primary，讓之後失敗可分辨外部靜音與路由缺陷；原失敗不能被 serial PASS 蓋掉。

## E816：refresh 執行中補改 receiver signature 會造成混合編譯輸入（2026-10-11）

- Primary 在 `communication-audio-engine-refresh-v1` 尚未 terminal 時補上 emote 原創 subtitle 目錄驗證並更改 model signature。即使結果成功也不能當作穩定來源編譯證據；若 link 失敗不回退正式修正去配舊物件。
- 此後等待 terminal 才再改來源；refresh 僅作前置，必須以 fresh ordinary no-cache 完整重新編譯與 native 驗證最新同一份來源，再做前後指紋比對。
- 保留原 log；不得藉清除 cache、修改 Engine metadata 或隱藏原失敗消除此錯誤。
- 實際結果：refresh-v1 terminal `OtherCompilationError`／184.68秒，OmEditor LNK2019／LNK1120 正是新 Consume signature 與先前編譯 runtime library 不一致；未把這次 refresh 算通過。接續 fresh ordinary build-v2 使用最終一致來源。

## E815：新增 receiver 音效範圍 ordinary gate 實際要求 Engine metadata 更新（2026-10-11）

- `communication-audio-build-v1` ordinary no-cache 實際 exit1，UBT `FailedDueToEngineChange`／5.48秒；原 build.log、UBT-Log 與 summary 完整保留。這不是 C++ 編譯成功。
- 僅在此真實結果後允許 maintained `build_ue_moba.lua --refresh-engine`，維持 local executor／no cache；refresh 後必須 fresh ordinary `NoEngineChanges` 成功才可驗收。
- 不手改 Engine Source、Config、BuildId、modules 或清除 cache；refresh 結果不冒稱普通 gate。

## E814：主音量錯綁單一 effects 資源可用性（2026-10-11）

- receiver 音效實作前獨立審查發現：主音量 slider 用 effects 完整性決定 enabled，僅剩 music 可播放時也會被停用。
- 主音量依任一可播放類別可用性開放；各分類保留個別缺檔診斷。音效字幕獨立於主音量；切換字幕清除舊 caption，重新開啟不重播舊提示。
- read-only 查 log 再次猜 `.editor.log` 失敗，實際 maintained runner 為 `.ue.log`；依 E807 先列實際產物再讀。失敗原輸出保留，非產品故障。

## E813：重開設定測試未掛入視窗就讀取 Slate managed attribute（2026-10-11）

- `original-audio-runtime-native-v3` 實際12 Success／1 Fail／0warning；滑鼠 midpoint、鍵盤 Right、磁碟 roundtrip 全過，只有 reopened slider value 失敗，原報告保留。
- `TakeWidget()` 只建立 widget；SSlider 的 managed Value attribute 須由 hosted prepass 更新。重開頁面必須掛到實際視窗並完成 prepass／draw 再讀取，沿用相同磁碟與數值斷言，不改成直接設定 slider。
- 修復後 fresh ordinary build 與 targeted test；先前12項只在來源未變時明示沿用。實際音訊 output-v4 已獨立通過，不因測試視窗修正重跑。
- 本次第一次 combined patch 因猜錯 MD 標題而原子拒絕；先讀實際標題後重做，沒有半套寫入。

## E812：一次性音效節流錯套到可恢復的回城 loop（2026-10-11）

original-audio-output-v3 的 routed／mute／restored 三份真錄音均已保存，但整輪原生測試因「effects unmute 後回城 loop 應立即恢復」只有一個 playing component而失敗，不能把 output.json 存在當整輪成功。根因是 StartVoice 的0.08秒同source／cue一次性burst節流，也攔截剛靜音停止的持續loop。修為僅對 one-shot 套節流；回城loop已有 SyncRecall 的當前公開channel／source identity／hidden／scope／volume與already-playing仲裁，應立即恢復，不加等待或重試規避。保留 v3 失敗與真PCM，fresh普通建置及完整輸出驗證再跑。

## E811：SetSubmixSend 僅影響目前 ActiveSound，元件重新 Play 不保留測試 send（2026-10-11）

original-audio-output-v2 已正確註冊 submix，實際 routed-mix 非零及 master-muted 靜音 WAV 都已保存；第三段 ambient-restored 因測試 aux send 遺失而取得零資料，整輪仍失敗。UE5.8 AudioComponent 實作顯示 SetSubmixSend 對 playing component 只送到當前 ActiveSound；未播放時 PendingSubmixSends 也在下一次 Play 後清空。因此原始聲道重啟後不能假設測試錄音send持久存在。修為在 unmute 實際重啟 ambient 後，對當前 playing components 重新掛測試專用 send，然後錄音；不改產品音量邏輯，不靠 Play 計數推斷輸出。v2兩份 partial WAV與失敗 logs保留，只有完整三段真PCM通過才可接受。

## E810：暫態 submix 未完成註冊就開始錄音，Start／Stop 指到不同 mixer 節點（2026-10-11）

original-audio-output-v1 實際建立獨立 WASAPI realtime 裝置（MSI G321CU／48000Hz／2ch），但原生輸出測試因空 recording shape 失敗，未產生接受的錄音或 output manifest。讀 UE5.8 實作確認 StartRecording 會排入 audio thread；找不到 submix 時會 fallback 該裝置 master，後續 SetSubmixSend 才註冊暫態 submix，StopRecording 找到新節點便取得空 buffer。修為明確 RegisterSoundSubmix，使用有十秒上限的 latent AudioCommandFence 確認實際 instance 存在，再 fence 確認錄音啟動後播放。計時起點改為初始化／fence完成後，避免把裝置初始化時間計入錄音等待；失敗加入 samples／channels／rate／stage，先驗 finite 再窄化。cleanup 明確 unregister 測試專用 submix，強參照持有到獨立裝置退出。保留 v1 原失敗，不使用 master fallback 或空資料冒充成功。

## E809：RetreatPing 已有 V 預設，catalog 卻仍宣告預設未綁定（2026-10-11）

original-audio-runtime-native-v2 已不再 fatal，13項中12 Success，ControlRuntimePersistence 因 RetreatPing optional-unbound 斷言失敗。原 wheel 批次把 RetreatPing 預設設成 V，卻遺留 `bOptionalUnbound=true` metadata；舊 settings/persistence 實際回歸揭露此不一致。修正正式 catalog metadata，保留 V 的既有操作；沒有把測試改成略過該列，也不把12成功稱為全部通過。fresh audit 與原 logs 保留，再建置重驗。新增 action 預設時必須同步 default-unbound metadata、說明和真正設定畫面回歸。

## E808：新音效 fixture 重複初始化 UWorld 造成原生程序 fatal（2026-10-11）

original-audio-runtime-build-v1 普通建置成功114.35秒，但 native-v1 在 AudioResourceScope fatal：CreateWorld 預設已呼叫 InitializeNewWorld，測試又手動呼叫一次，第二次 WorldSettings 唯一名稱建立失敗。原完整 log／crash／非零程序結果保留；已通過的三項（DisplayPreferences／MinimapPreferences／AudioProfile）不替代未完成的整批。修復為透過 CreateWorld 的 InIVS 傳入音訊關閉／無 physics 等測試初始化設定，只初始化一次；不改 Engine 或降低錯誤。之後 fresh 普通建置與全部13項重新跑。建立 fixture 前必須讀公開 API 的初始化語意，不能把 CreateWorld 與 InitializeNewWorld 無條件併用。

## E807：PowerShell 傳給 rg 的檔案參數包含未展開 wildcard／猜測路徑（2026-10-11）

另一次將 PowerShell cmdlet 的 `-ErrorAction SilentlyContinue` 誤傳給外部 rg，導致 rg 解析 `-E` 為 encoding 而中止；外部 executable 不接受 PowerShell common parameters。改用確定存在的引擎當前 log 精確讀取，正式證據仍等 workflow 結束後保存，不忽略外部工具的退出碼。

直接把 `Private/OmCommandBar*` 或 `OmWorldBridgeActor*` 當 path 傳入 rg，在 Windows 被視為字面檔名而失敗；另曾猜測不存在的 OmMobaHudTypes.h／omfue/templates。這些為唯讀搜尋失敗，沒有改檔，但重複發生會浪費時間。已改用 `rg --files` 先確認目錄／檔名，然後用已存在目錄加 `-g 'OmCommandBar*.cpp'` 或精確完整路徑。不得再用檔案 path wildcard 當 rg 的輸入。廣泛搜尋 codegen 時漏排除 target 造成長查詢，已中止該唯讀程序；接續只搜確切 src 檔。

## E806：unity 編譯的匿名 namespace 常數與引擎模板參數撞名（2026-10-11）

修復 helper 首次預期六個 token，實際五個，assert 在寫入前中止；核對所有引用後修正預期五個並重跑。數量 assertion 必須由實際搜尋核對，不能憑記憶猜測。

original-audio-import-build-v1 普通 NoEngineChanges／NoUBA／nocache 建置失敗25.78秒，C4459 指向 TaskDelegate.h 的 Destination 參數。新 commandlet 的匿名 namespace 常數 Destination 在 unity translation unit 中仍可見，導致後續引擎 header 模板被視為隱藏全域名稱。改為具體 OmOriginalAudioDestination，不改 Engine／warning policy／unity 設定；保留原完整 UBT log，再跑 fresh 普通建置。新增檔案的 file-scope 常數與 helper 一律使用明確專案語意名稱，避免短泛用名稱污染 unity TU。

## E805：目標控制加入 sender viewport 後靜音按鈕被裁切（2026-10-11）

communication-shortcuts-owned-phase-v4 已通過先前聊天scroll點，卻在 chat-mute 的真實 Slate hit 路徑失敗；兩個目標按鈕直接插進64px sender viewport，使原名冊按鈕中心落在clip外。保留原失敗與cleanup，不以cached rectangle強行點擊。改為明確「目標投票／返回聊天與隊友」切換：目標使用同一主scroll viewport，正常聊天保持名冊原本用途；切換只改顯示不發wire，關閉歷史／scope失效／開啟草稿會重置。目標入口與文字使用目前字級，面板高度量測納入header。兩個新增投票案例實際點toggle後再點目標；第二案例也重新開歷史，不依賴前一case的狀態。

## E804：聊天滾輪測試假設起始 offset 一定為零（2026-10-11）

v3診斷實測 before33.7／after0／end119.7，History viewport高僅1.3、AllRow paint0，失敗原圖已檢查；真正問題為新增目標控制壓縮歷史區，前述方向修改只處理一個測試假設。正式修復改依目前字高、通知換行、64px bounded roster、草稿及至少兩行歷史計算互動面板高度，仍受實際client高度限制。scroll檢查要求至少24px readable viewport與至少一列真實可見文字（向上後不要求最下面的全體列仍可見），保持真方向位移與camera隔離，不把1.3px viewport算成功。

後續 v2 重跑仍在同一測試失敗，證明單改捲動方向不足以解釋問題；不把第一個推測當結論。保留v2失敗，新增before／after／end／實際viewport與文字geometry及失敗原圖，先取得實測值再決定版面修復。

communication-shortcuts-owned-phase-v1 真實停在 chat-received-large，尚未執行新增 HUD／objective cases；原native／workflow／log／PNG及完整cleanup保留。新的公開目標按鈕改變 bounded roster 區所占高度，歷史已經 ScrollToEnd，測試仍往下捲並要求 offset 增加。修為依實際起始 offset 選擇可移動方向：正 offset 向上、零 offset 向下，嚴格驗證該方向確實改變 scroll offset、真實文字 geometry及camera不變；不是直接設 offset 或略過測試。不宣稱首輪83項成功，fresh ordinary build／native／actual phase重新驗。

## E803：無 runtime-driver 組態仍編譯 ward 的 optional core helpers（2026-10-11）

communication-shortcuts-rust-v1 的六項 communication、25項 fixture與 default all-target check成功，額外 bridge-no-driver check 真正失敗；三個既有 ward helpers 未掛 runtime-driver cfg，引用該組態不存在的 optional omoba_core，衍生9個錯誤。原log與失敗summary不改。確認所有正式消費端與對應測試本來都限 runtime-driver 後，同樣限定這三個 helpers；不新增重依賴、不引入假的常數或 fallback。新fresh validation 同時重跑通訊／fixture／ward reader以及default／no-driver checks。

## E802：新的 owned 點擊案例錯用不存在的 helper（2026-10-11）

communication-shortcuts-build-v3 ordinary 真正失敗10.91秒：新案例誤用不存在的 EExpect::Any／JsonObject，且 Lines 不在區域作用域；native 測試把 AddDefaulted 的索引當成 row。原logs保留。修正為既有 LoadIntents／ContinueIntentRead、單一新增 wire、FJsonSerializer 及明確區域 Lines，新增 row 使用 AddDefaulted_GetRef。之後先完整讀取相鄰成功案例的宣告與作用域，再移植驗證；不從截斷工具輸出推測 helper。

## E801：optional objective query 沒有轉換 opaque runtime handle（2026-10-11）

communication-shortcuts-build-v2 ordinary 真正 C2664 失敗113.29秒：RuntimeHandle 為 void*，新的函式指標要求 OmRuntime*。保留原 logs／summary，terminal 後依既有 bridge 呼叫慣例明確 static_cast；不修改 C ABI 型別或取消型別檢查。Rust 六項 communication 測試實際成功，不能取代 Unreal 編譯及實際點擊驗證。

## E800：HUD shortcut 的成員名稱遮蔽與 Slate 容器迭代錯誤（2026-10-11）

communication-shortcuts-build-v1 ordinary 真正失敗（76.85秒）：區域 Slot 遮蔽 UWidget::Slot，FArrangedChildren 沒有 begin/end。原 logs 與 summary 保留。等待 terminal 後改用 HudSlot，依既有 Slate 路徑使用 Num／索引迭代；不降低編譯檢查。發布 helper 另因 Windows text-mode 重複 CR，在編譯前已以 binary 模式修正七個來源並記錄於 direct errors MD。之後 explicit CRLF 寫入一律傳入第四參數 binary=true。

## E799：共用HUD guard區域變數遮蔽UWidget.Visibility（2026-10-11）

communication-markers-build-v4 ordinary17.42秒真正C4458失敗，新增區域Visibility遮蔽UWidget既有成員。原build.log／UBT-Log／summary保留。等待terminal後改用ClusterVisibility；不關閉warning-as-error或改Engine。guard行為維持E798正式修復，fresh ordinary build後才跑native／actual圖片驗證。

## E798：共同HUD邊界忽略SelfHitTestInvisible容器的可互動子元件（2026-10-11）

communication-markers-owned-phase-v1真69/39/13與cleanup成功，八native及25Rust/兩checks通過，847來源不變；七張新增original目視仍發現小視窗world marker畫在combat HUD上，故不接受完整marker呈現。來源追查確認SBox預設SelfHitTestInvisible，IsOwnedClusterUnderPoint卻只看IsHitTestVisible，導致新的CombatWidthBox可視viewport永遠被排除。改為可見且self或children可命中；combat另檢查真正CombatCluster仍可見，維持HUD關閉時世界可操作與透明空白不攔截。這個共同修復同時維護世界標記／pointer／edge-scroll的HUD區域判定；不把world標記改成另一套硬編rectangle。保留v1所有成功操作與不合格原圖，fresh native包含可見／隱藏／退休guard，再跑actual phase及原圖。

## E797：最後案例新增純呈現檢查後，missing-wire負例沿用錯誤最後名稱（2026-10-11）

新增received／expired／retired與combat overflow scroll六項無wire案例後，Lua自足契約真失敗：checker正確指出缺少最後wire案例ping-repeat，測試卻把inventory最後的combat-overflow-scroll當作最後wire。修正測試從末尾尋找明確spec[2]存在的最後wire-bearing案例，而非硬編名稱或假定每個案例送指令；一般checker未放寬，missing/extra wire拒絕保持。新宣告69/39/13 phase與383/167/136 full仍須實際執行，不能據宣告／fixture測試算完成。

## E796：marker新增來源觸發UHT後一般建置要求更新Engine manifests（2026-10-11）

communication-markers-build-v1 ordinary NoEngineChanges／NoUBA／nocache真失敗，UBT14.86秒 FailedDueToEngineChange，UHT成功且寫1個generated file。此次清單為Engine及插件UnrealEditor.modules等產物；沒有本批C++ compiler diagnostic，也不能沿用輪盤build成功宣稱marker已編譯。完整build.log／UBT-Log／summary保留。依已維護scripts/build_ue_moba.lua的--refresh-engine產物初始化流程，之後仍必須fresh普通NoEngineChanges gate；不手改Engine source/config/BuildId/manifest、不刪cache或跳過檢查。這是實際gate要求，不宣稱已找出歷次UHT package變化根因。

## E795：400×300 最大文字輪盤標籤重疊（2026-10-11）

fresh communication-wheel-owned-phase-v5 實際63cases／39 TCP intents／10stages全部成功、cleanup成功，843來源在驗證期間不變；十張original輪盤圖逐張檢視後，最大文字的左右語意與中心標籤重疊，故不接受完整畫面呈現。原圖與結果保持不變。根因是標籤沿固定圓周排列，未依實際字體量測留出橫向寬度。改用八個實際標籤的最大量測寬高配置三欄三列、視窗內clamp與橢圓輪廓，保留原始pointer角度選擇及world point；不縮小字體或刪除語意。後續必須fresh build／實際操作與original大字圖再驗。

## E794：native forced-window paint 座標誤作 desktop hit 座標（2026-10-11）

native-v6 ResponsiveHudLayout 仍失敗，保留診斷與fresh原圖。實際資料顯示mini desktop rectangle `(187,325)-(259,397)`、Window `(175,175)-(575,409)`，卻以paint-space center `(48,186)` 呼叫要求desktop位置的 LocateWindowUnderMouse，因此hit none。原圖 `ResponsiveHudOverflow/166E2310481D2A0E371FD6B6E944D11F.png` 逐張檢查，確認scroll HUD在小地圖旁且未遮住。這個fixture的forced window paint座標是window-relative，改用cached/tick desktop geometry，與正式TrySampleMinimapPoint相同；保留完整interactive windows、own user與同一widget嚴格命中，沒有更換小地圖中央位置或放寬斷言。不同取圖路徑不可一概假設PaintSpace含desktop位移。

## E793：短 client viewport 的新增 native 中心命中測試未通過（2026-10-11）

build-v10成功55.92秒，native-v5真10 Success／1 ResponsiveHudLayout Fail：新400×234中心 `LocateWindowUnderMouse` 未包含小地圖。原邊界不重疊斷言已過，不能僅據 rectangle 宣稱可點。測試補明確actual SlateUser，保留完整interactive window列表與嚴格widget身分斷言，追加paint/tick/window/body/hit/fit診斷及fresh GUID原圖保存；若仍失敗依真正命中與原圖修正。沒有放寬到任意widget、限制window列表偽造命中或跳過中心點。

## E792：raw Slate user index 的 unsigned 型別與 token aggregate 不符（2026-10-11）

ordinary `communication-wheel-build-v9` 實際回報 C2397：FKeyEvent::GetUserIndex 是 uint32，但新release token User寫成int32，aggregate初始化拒絕縮窄。token改用來源相同的uint32，保留前置own-user檢查，無截斷／猜測轉換。新增事件歸屬欄位先核對UE公開型別；修後fresh普通build，原錯誤log保留，不把之前成功build當成新來源已編譯。

## E791：來源審查發現同 frame paired release 重建本機 user，原外來 release 測試太早取消（2026-10-11）

這是審查發現，沒有宣稱既有run已實際洩漏。為支援G/V同frame tap新增的IE_Released callback自行使用owner user重建FKeyEvent，丟失原始Slate user；既有foreign-user case在同frame注入後立刻Escape，未等待PlayerInput flush，不能證明延後release隔離。新增有界raw-release token：只記錄真正own user／owned focus／非文字modal的wheel按鍵release，保留原user、profile revision與frame，只讓兩frame內同來源paired release消費一次；新press／owner退休清除，foreign release不產生token。測試改為等待三個實際frame、要求gesture仍active且零新增TCP，再取消。這是通用事件歸屬修正，不消耗其他local user事件或fabricate本機身分；fresh build/actual proof pending。

## E790：捲動 viewport 回歸仍以未裁切 content rectangle 判定極小視窗重疊（2026-10-11）

`communication-wheel-native-v4` 真正6 Success／1 Fail（ResponsiveHudLayout），ordinary build-v8成功15.28秒。既有一般解析度／文字排版斷言已通過，80×60 no-fit 的重疊斷言仍取完整 scroll content，而應取可見 clipped viewport。補公開widget tag OmCombatViewport，只在該 tiny bounds 斷言改用可見邊界；原 tiny no-fit 斷言保留。另新增400×234實際client、combat/minimap可見邊界分離、真正Slate中心hit與獨立訊號取點測試，不放寬命中或文字下限。新native gate pending，失敗報告保留。

## E789：SScrollBox 參數 ScrollBarThickness 大小寫錯誤（2026-10-11）

ordinary `communication-wheel-build-v7` 實際編譯回報 C2039，錯寫 `ScrollbarThickness`，並引出後續 Slate bracket 語法錯誤。UE5.8 本機公開 `Widgets/Layout/SScrollBox.h` 的確切參數是 `ScrollBarThickness`。修正大小寫，不改Engine、不降低警告、不將連帶語法錯誤當多個獨立產品缺陷。新增陌生Slate參數應先查本機header；需fresh普通建置驗證。

## E788：不可容納的自動高度 HUD 溢出，遮住實際小地圖 hit grid（2026-10-11）

`communication-wheel-owned-phase-v4` 調整一次視窗後仍失敗44／20／10，清理完整；E787並未解決產品問題。原始 `wheel-minimap-unreachable.png` 證明道具列覆蓋小地圖中心。actual viewport content僅約394×234，所有 readable composition嘗試均combatH不合，舊fallback直接安排完整auto-height內容，子項向下溢出覆蓋小地圖。改為共用實測內容高度不足時的有界垂直scroll viewport，優先保留小地圖旁的欄位；正常可容納尺寸保留原排版。scrollbar溝槽納入所有內容寬度預算，UI guard採可見viewport邊界。未縮小文字下限、未隱藏道具、未換測試點、未直接呼叫delegate；必須fresh responsive/native及同中央實際點驗證。原失敗PNG保留，修復尚待驗證。

## E787：小視窗輪盤案例在每次取點重試重設視窗與 profile（2026-10-11）

實際 `communication-wheel-owned-phase-v3` 失敗於 ping-wheel-400：44 cases／20 TCP intents／10stages，cleanup全部成功。800輸入前綴完整，但新增小視窗取點不到；不得當成63-case通過。審查指出 Substep3 重試會再次 ResizeOwnedClient 與 SetWardTextScale，反覆安裝設定／重排版。改為一次調整、獨立等待實際paint後再取點，並在仍無命中時保存 `wheel-minimap-unreachable.png`。這項修正先排除測試本身不穩定；若fresh實跑仍失敗，須依原圖與實際hit path修產品版面，不能假設問題已修或換成直接 delegate。

## E786：八方向表情案例的 Lua ID 生成差一（2026-10-11）

新增實際八方向案例後，自包含 Lua contract 正向驗證立即失敗 `Communication emote mismatch`。C++ 與 synthetic wire 使用 canonical102–108，但案例 policy 生成算式將第二方向算成101。修正 index+89 為 index+90，並同步正式七筆 policy；不放寬 canonical ID／方向／逐 case raw wire 檢查。每次擴充 producer inventory 必須先跑同一公開 validator 的完整自足正向 fixture，成功後才開 UE。

## E785：小地圖訊號取點誤用施法偏好 gate，且輪盤未套共用 DPI 閱讀空間（2026-10-11）

真實 `communication-wheel-owned-phase-v1` 失敗於 ping-minimap-alt：42 cases／18 TCP intents／10 stages，所有程序、profile、listener 完整清理，原始圖片及 log 保留。先前 G/V 同 frame tap、G/T north drag、取消／外部焦點／profile退休與世界 Alt/Ctrl-left 已產生正確一次 wire。小地圖 callback 誤用 `TrySampleMinimapGround`，該 API 正確受 allow-ground-casts 偏好限制，卻不應限制訊號。抽出不含施法權限的 `TrySampleMinimapPoint` 共用幾何取樣；原 ground API 保留原 gate，訊號改用共用 point API，仍檢查實際 owner/scope/可見地圖/finite座標。鍵盤訊號同樣拆開權限，不開啟施法設定來掩蓋問題。原 PNG 另顯示輪盤文字被 viewport resolution curve 縮小；使用既有 FOmHudViewportSpace 正規化繪製，保持原世界取點與 gesture dead zone 不變。需 fresh build/native/actual gate，不能以失敗 run 宣稱完整接受。

## E784：新增負向測試比對錯誤文字而非既有契約文字（2026-10-11）

輪盤契約第一次自包含驗證正確拒絕 null 座標，但新增測試預期 `must be an integer`，既有共用驗證實際回報 `is not an integer`，因此測試失敗。修正測試比對原有訊息，未放寬整數檢查；重新執行 `owned_viewport_ipc_contract_test.lua`，86 cases 全數通過。新增負向測試先核對實際共用 validator 訊息，不能把測試斷言錯誤誤認為產品輸入被接受。

## E783：聊天鍵盤事件冒泡至 viewport，同 frame 送出後觸發技能（2026-10-11）

真實 `communication-chat-owned-phase-v3` 完成 mute/unmute/reconnect/all-chat-disable/re-enable，最後 chat-text-controls 失敗；已看到12個預期 Communication wire，另多出2個 AbilityCast（lumen_touch/lumen_mend）。原始 TCP、log與9張圖保留。普通 Slate 文字元件以 OnKeyChar 收文字，未處理的 key-down/up 可冒泡至 SViewport，被 PlayerInput 排隊；同 frame Enter 關閉後舊有當下 focus guard 已看見 viewport，因此不足以隔離。聊天框採一般 SEditableTextBox 子類，先執行原本 OnKeyDown/Up，保留已處理的編輯導航／剪貼簿／Enter／IME；只有未處理回覆改成 Handled，停止向遊戲冒泡。這是文字輸入邊界的通用隔離，不依賴 QWER 或指定 action 黑名單，不延遲傳送、不修改原測試次序或允許額外技能。需 fresh actual typed keys → 唯一 chat wire 驗證。

## E782：歷史捲動與名冊操作共享內容使小視窗控制不可達（2026-10-11）

`communication-chat-owned-phase-v2` 再次實際失敗17/7/3，清理完整。E781的明確換行/prepass並未使按鈕可達，不能宣稱修復；這輪按鈕從未 paint，cached geometry為零。將聊天歷史和名冊操作拆成兩個一般 Slate scroll regions：名冊區固定保留可見的有界高度64，只在歷史開啟時顯示；訊息長度/字級不得擠走唯一的 reconnect/unmute 操作。排除自己名冊列，保留最多9個其他玩家、有界垂直捲動、明確靜音/解除靜音與原 TCP receipt policy。仍以同一實際 hit-grid 身分及鍵鼠事件驗證，失敗原圖/log不變；fresh gate pending。

## E781：聊天 AutoWrapText 延後量測，捲動內容壓縮靜音按鈕至零高度（2026-10-11）

真實 `communication-chat-owned-phase-v1` 在第17項 chat-mute 失敗：7個 TCP intents、3個控制階段；清理 editor/publisher/profile/listener 成功。large PNG 的訊息換行可見，但靜音按鈕 paint/tick 高度為0；僅重試或使用 paint 座標無法修復。來源將 received row 的換行改成以實際 panel 寬扣除 scrollbar 的明確 WrapTextAt，寬度變化納入有界 cache invalidation，建立 rows 後先 SlatePrepass 再 ScrollToEnd。保留一般 Slate button、完整 hit identity 檢查與文字下限。不得放寬命中、直接呼叫按鈕 delegate 或將失敗原圖當成可用畫面。fresh build/owned 與原圖必須重驗。

## E780：新增測試階段成員與既有參數撞名（2026-10-11）

實際 ordinary build `communication-chat-owned-build-v10` 在 26.38 秒失敗，原 UBT log 保留。新增 `Phase` 成員遮蔽既有 `AssertFixtureFocus(const TCHAR* Phase)` 的參數，MSVC C4458 視為錯誤。將階段成員統一改名 `TestPhase`，保留既有焦點診斷參數與所有斷言。不得降低 warning 等級或改引擎。後續新增 session 成員前，先查整個類別同名參數；修後需 fresh build 與實際測試，不能將 Lua 契約通過當成 C++ 編譯成功。

## E779：捲動容器的 ArrangeChildren 座標未含 paint offset，實際點擊落在技能列（2026-10-11）

communication-chat-owned-viewport-v5 原native／workflow失敗，331 cases／135 wires／126 stages。原314操作、team/all中文、取消／失焦／歷史及Ctrl+1–5真wire全過，三種received圖片已產生，停於chat-mute；publisher/editor/profile/listener全部清理完成。沒有authority ACK，原報告及圖不改。

真hit-grid診斷：400x300大字狀態下mute按鈕實際PaintGeometry Y123，遞迴ArrangeChildren卻為Y199（差76px scroll offset）；錯點擊被嚴格identity guard拒絕，leaf是OmAbilityIcon.2。通用測試修復先使用同一個已arrange widget的實際paint rectangle，再以own window hit grid確認同一identity後才送正常Down/Up；保留原arranged候選與禁止delegate直呼。不做任意像素搜索或忽略clip。

三張original received圖已直接檢視：800／400兩頻道與sender controls可讀，大字圖在字體重排後尚未重新ScrollToEnd。UI現在只在真AppliedTextScale變動時使有界history cache失效、重建並重新捲到末尾；capture等待八個真Slate frames，不拿同一paint呼叫當排版完成。繼續fresh build/native/owned與原圖審查，尚未接受此批。

## E777：未加原生按鍵追蹤的既有控制器把同批真 press＋repeat 當成純重複（2026-10-11）

communication-chat-owned-viewport-v4 原生／workflow失敗，321 cases／130 wires／124 stages。Team channel1與All channel2實際中文wire皆正確，Escape／focus cancel／Z歷史及Escape已通過；interaction-1未送wire。清理publisher/editor/profile/listener成功，沒有authority ACK，原logs／兩張draft PNG保留。

真正owned fixture使用AOmUninstrumentedInputController，代表既有Blueprint入口。HandleCommunicationPressed沿用IsHeldOsRepeat，但該判斷依賴native InputKey caller instrumentation；同批真press＋repeat在未instrument入口被誤拒。修在共同Slate前處理器：只攔截當前已實作communication action的精確主／次鍵OS repeat，且須owned viewport、同Slate user、無text/modal；真press照常走原InputComponent，不替控制器偽造ObserveCallerPressed。原Stop/Hold政策與native fallback保留。fresh gates待驗。

## E778：來源審查發現通信 fixture 未發布公開名冊（2026-10-11）

v4在E777停止，尚未執行後段roster-reconnected案例。檢視真正FixtureWorld::to_snapshot只有phase／elapsed／lane與可選owner facts，沒有scoreboard公共metric，因此先前「fixture本來有1／2名冊」的假設錯誤。不是新owned失敗結果，也不是product roster helper應放寬。通信ready／received現在明確發布兩名玩家／兩隊的合成PUBLIC scoreboard metrics，由正式scoreboard projector及leased C ABI取得；baseline／retire維持沒有名冊，無authority receipt或ACK。新增真projector測試驗scope退休與無live owner依然可公開。原314操作前綴不增metric；fresh Rust/native/owned待驗。

## E776：追加 owned 案例未同步 inventory 靜態斷言；同檔 draft patch 被舊內容覆蓋（2026-10-11）

communication-chat-owned-build-v6 失敗9.62秒；真正編譯抓到 kOwnedCaseCount／pumped-frame budget 靜態斷言仍為330／36868。四個追加案例的實際推導值為334／37376，同步明示數字，保留靜態斷言與各階段timeout。原build證據保留。

Lua repair helper同一檔案兩次從磁碟讀取，pending最後一次寫入覆蓋先前missing-wire訊息修正，故再次失敗。正式檔改為逐檔單次原子patch，尾筆負例從公開inventory取得；point負例配合已拆開的typed mute檢查改用新的精確point錯誤訊息。不刪案例、不弱化驗證。一次帶空Markdown hunk的apply_patch另被atomic拒絕，無部分修改；後續只使用實際上下文。helper往後必須累積同檔內容或拒絕重複檔名。

## E775：公開名冊迴圈名稱遮蔽 local-player 變數；append-only 測試仍固定尾筆名稱（2026-10-11）

communication-chat-owned-build-v5 ordinary NoEngineChanges／NoUBA／nocache 真實編譯失敗，C4456指出 RefreshCommunicationUi 新名冊迴圈 Player 遮蔽同函式 ULocalPlayer* Player。改為 RosterPlayer；既有warning-as-error不放寬。原build.log／UBT-Log保留。修改前需等編譯程序terminal，避免變動正在驗證的source。

同輪自足Lua契約positive整份334／137／128 inventory已通過讀取，negative「移除最後wire」仍硬編 interaction-5，實際最後是chat-unmute，錯誤訊息不符。改為由公開expected_cases最後案例取得名稱，避免未來append重複犯錯；不刪負例、不放寬missing-wire gate。新增mute boolean／desired state／sender／free-text負例與native相同typed contract。fresh結果另記。

## E774：owned shortcut 只傳 modifier 旗標，未實際按下修飾鍵（2026-10-11）

communication-chat-owned-viewport-v3 真正317 cases／130 wires／124 stages，原314前綴完成，但 all-chat 實際送出 channel1，native/workflow皆失敗。Team中文正確、兩筆不同request token，沒有authority ACK；publisher/editor/profile及listener清理完成。原log、wire及PNG保留。畫面原圖證明E773修復後輸入框在實際800x600視窗可見。

原因是 fixture 的 Shift+Enter 只有 FKeyEvent modifier旗標，沒有 LeftShift down；production 精確修飾鍵讀取 UPlayerInput 真實按鍵狀態。Ctrl+1–5同樣缺modifier down，source review一併修正。現在先以正常Slate按下LeftShift／LeftControl，維持到下一次實際input flush／wire，再正常release。不修改production binding來迎合錯誤fixture，不偽造channel。接fresh ordinary build/native/owned gate，尚未宣稱修後通過。

## E773：聊天 overlay 用 DPI normalizer 外層座標排版，焦點可輸入但畫面被 HUD 遮住（2026-10-11）

original 檢視 `communication-chat-owned-viewport-v2/chat-team-draft.png` 時，真正中文已送出，但圖中沒有可見聊天輸入框。來源確認 RefreshCommunicationUi 用外層 UUserWidget geometry 量測，實際 overlay 由既有 SOmHudResolutionNormalizer 以 ChildScale／Allotted 排版；低resolution DPI曲線下，外層local高度與 normalized child坐標混用，把panel推到combat區域後方。且原chat overlay先於combat繪製，窄版明確開啟時可能被後繪的HUD完全遮住。

直接復用 FOmHudViewportSpace 的量測transform建立相同child geometry，再將實際combat／status絕對幾何轉入同一空間；不硬寫DPI曲線或特定resolution像素。chat overlay改排在HUD之後、shop之前，保留商店上層仲裁。新增真focused editor的actual client bounds／非零可讀尺寸斷言、最多8frame布局等待與失敗原圖；後續fresh圖片需逐張檢視，不能用成功wire宣稱可見GUI已成功。

## E772：聊天 wire observer 把 objective_id 空字串誤寫成數字零（2026-10-11）

`communication-chat-owned-viewport-v2` 實際 native=false／workflow=false，316 records／129 wires／124 stages；原314 prefix完成，真正Enter開聊天及中文「集合」TCP封包已出現（ordinal129，kind Communication／chat_channel1／source1／presentation1／view11，無authority ACK），原 chat-team-draft.png 保留。失敗根因在新觀察器：把 protobuf string `objective_id` 列入整數0欄位，實際producer正確輸出 `""`。自包含C++／Lua正例也鏡射此錯誤，之前三native與65Lua通過不等於實際wire符合。

依真正producer／proto欄位型別修正兩個嚴格observer及正例，`objective_id` 必須是空字串；數字0與非空objective各自新增負例。正式提交／producer沒有改成迎合錯誤測試。原實跑完整logs／失敗報告保留，程序與profile清理、listener released=true；fresh建置／native／owned再驗才可接受。防重犯：跨語言正例需依實際序列化欄位契約，而非抄寫被測驗證器的分類。

## E771：owned chat 在同一呼叫內檢查延後處理的 viewport 按鍵（2026-10-11）

`communication-chat-owned-viewport-v1` 實際 native=false／workflow=false，316個record（第316個 chat-team-chinese失敗）、128條舊wire與124 publisher stages；原314案例已完成，publisher／editor／profile清理及listener released=true，authority=false。失敗文字是 actual Enter did not open chat。新測試透過真正 Slate viewport 送鍵後，立刻在同一 latent Advance 檢查面板；viewport輸入先排入 UPlayerInput，須下一個正式frame flush才會執行綁定。改成明確事件派送／等待實際open／editor字符與commit／等待實際close的階段，保留180frame上限、逾時真畫面與focus／notice／route診斷，不直接呼叫handler、不假造成功。Z與來源退休前開草稿同樣使用分階段事件。

來源審查另修正 history Escape 的真綁定缺口（owner安裝／visibility notify納入history），以及Lua artifact checker將control ordinal誤當IPC sequence：communication stage真正另送一個Presentation，後續Snapshot序號須計入；保留精確control順序與舊prefix，獨立self-contained ACK fixture／兩個序號負例一起擴充。這兩項是本輪來源確認，尚未冒稱首次實跑曾走到它們或修復後已成功。

## E770：owned chat 測試區域變數遮蔽既有 Instance 成員（2026-10-11）

`communication-chat-owned-build-v1` 正式普通 NoEngineChanges／NoUBA／nocache 建置於15.93秒失敗，唯一 compiler diagnostic 是 OmOwnedViewportIpcTests.cpp 的 C4458：新區域 `Instance` 遮蔽同類別已建立的 UGameInstance 成員。原 build.log／UBT-Log.txt／summary.json 保留。直接沿用測試真正建立的既有 Instance，移除重複宣告；不關閉 warning-as-error、不改 Engine，fresh v2 才能作成功證據。

## E769：追加案例後舊 Lua 負例仍假設最後一筆是商店命令（2026-10-11）

owned viewport 自包含契約第一次擴充實跑失敗：負例刪除最後封包，卻仍期待 shop-txn-stale 缺失。實際新尾端是 interaction-5。保留原314／128／123順序，將這個通用尾筆缺失／多筆負例命名與預期改成真實新尾端；fresh Lua 契約56項通過。不得為維持舊測試字串而截斷新功能封包，後續加入明確溝通封包負例。

## E768：聊天負例用 AppendChar(0)，未真正保留 NUL（2026-10-11）

`communication-chat-ui-native-v1` 實跑五項：ActionOwnerLifetime／ControlProfileDefaults／ControlRuntimePersistence／KeyboardFocusGuard 四項Success，CommunicationDraftContract 一項Fail、一個錯誤。Unreal process exit0但workflow=false，不能把exit0當測試全過；原native-report與summary保留。ControlRuntimePersistence 另有一筆 Slate scroll-descendant warning，照實保留。

負例原先以 FString::AppendChar(Scalar) 組裝 `x+控制字元+y`。已唯讀檢查當前 Engine `Core/Private/Containers/String.cpp.inl:399`，AppendChar 明確只在 `InChar != 0` 才附加，因此 NUL 案例實際變成 `xy`。改為先建 `x?y`、直接設定fixture的UTF16陣列位置1，並逐例驗證 Len=3、錯誤名稱帶實際U+碼；保留NUL／C0／C1／line-separator／不成對surrogate的拒絕斷言，未放寬正式Unicode驗證。只重建受影響測試模組並 fresh rerun 草稿契約；前四項通過和scroll警告明示沿用。E767已修復，ordinary build-v2成功16.54秒；補設定頁中文標籤與真正已接功能的測試期待後，build-v3成功20.59秒。

## E767：聊天 UI 首次正式建置的 Slate slot 屬性與 checked format 使用錯誤（2026-10-11）

`communication-chat-ui-build-v1` 的真實 UBT 編譯已回報兩個根因：OmCommandBarWidget.cpp 的 `SOverlay::FOverlaySlot::FSlotArguments` 沒有 `Padding_Lambda`（C2039）；OmCommandBarCommunication.cpp 把條件運算式選出的 `const TCHAR*` 傳入 `FString::Printf`，不符合 UE5.8 的編譯期 checked format（C2664），其餘 C7732／C2679 是同一表達式的後續解析錯誤。尚未執行新UI原生測試，不把新來源當成功。原建置先跑到 terminal，保留同名 build.log／UBT-Log.txt／summary.json。

已直接讀取目前 Engine 的 BasicLayoutWidgetSlot.h，正式 slot 支援 `.Padding(TAttribute<FMargin>)`；改用 `TAttribute<FMargin>::CreateLambda` 綁定量測後 margin，維持動態幾何布局。靜音標籤改成各自以 literal 呼叫 Printf，再選擇已建立的 FString。不得關閉格式檢查、改 Engine header 或改成固定像素 workaround。修復後另跑 fresh v2 ordinary NoEngineChanges／NoUBA／nocache，實際成功後才跑 native／真路由UI驗證。

E766後續已解決：正式 refresh-v4 初始化成功1014.94秒，最後普通 gate 成功4.63秒；native-v1 真載入 staged bridge，CommunicationBridgeContract 一項Success／測試0warning，來源835與stage／header／MSVC／legacy隔離接受。原v3 EngineChange失敗與v4成功分開保留；完整紀錄 `.ai-collab/communication-public-ffi-accepted-v1/summary.json`。全域startup／vendor警告沒有被冒稱消失。

## E766：新增 reflected header 後正式建置遇既有引擎產物更新（2026-10-11）

`communication-public-ffi-ue-build-v3` 的 UHT已成功（156.0086404秒／6 generated files），普通UBT仍於159.70秒以 FailedDueToEngineChange exit5拒絕；不是新的C++diagnostic，也不能寫成已編譯成功。原完整build.log／UBT-Log／summary保留。待更新清單包含 NetCore DLL/PDB/obj及Engine .modules/.version，不能只稱manifest不同或手改BuildId。依已維護且多次記錄的一般 `--refresh-engine --build-executor local --disable-action-cache` 流程正式初始化source-engine產物，再走最後普通NoEngineChanges/NoUBA/nocache與stage SHA；不手改Engine source/config/manifest/BuildId、不刪cache、不使用module-only shortcut當完整成功。先凍結目前source，待fresh refresh與普通gate真正terminal後再驗native API。

## E765：Unreal reflected enum 缺少0值，正式 UHT WarningsAsErrors 拒絕（2026-10-11）

`communication-public-ffi-ue-build-v2` 的正式 base_content DLL、19/20 codegen、bridge與header stage完成，但 UHT 指出新 `EOmCommunicationKind` 缺0值並使正式建置 exit6；未宣稱 UE API已可用。build.log／UBT-Log／summary 保留於同名目錄。直接加入明確 `Unknown=0` 作為 reflected storage 的無效值，各 intent 仍預設 Ping，Rust實際提交仍只接受1..6；不關閉 WarningsAsErrors，不把0當合法通訊。重建前另外完成來源審查發現的 match-clock expiry 接線：從實際已驗證 snapshot clock 在真正期限邊界更新public revision，不以wall time刷新TTL，也不每tick複製history；新增pause／regression／foreign binding／exact boundary回歸。等待fresh focused tests及ordinary build-v3，不冒稱目前已成功。

## E764：舊「未支援版本」fixture 把新版7當成拒絕案例（2026-10-11）

`communication-feature-boundary-tests-v2` 的無網路 feature check、28 communication transport 與5 framing tests通過；client presentation實跑32項中31通過、1失敗：`authoritative_mode_projects_through_snapshot_and_rejects_bad_protocol` 原硬編 `[5,7,99]`，新版合法7被錯誤要求拒絕。原 logs 保留於同名 worker evidence 目錄，後續 bridge步驟未跑。修正拒絕矩陣為舊版3／5／6及 `PRESENTATION_PROTOCOL_VERSION + 1`，保留不得安裝不同模式的斷言；不放寬 parser、模式驗證或實際協議。Fresh v3重新驗證，不改原失敗。

## E763：Lua 二進位讀取後用文字模式寫入造成 CRCRLF（2026-10-11）

`communication-feature-boundary-tests-v1` 無 network feature check 已通過；接著 core transport 編譯在 framing.rs 的 doc-comments 實際出現32個 bare CR 錯誤，後續測試未跑。原 stdout/stderr／summary 保留於同名 worker evidence 目錄。直接 tag-move helper 誤把 `path.write(path,data,overwrite,binary)` 第三參數當 binary，讀取既有 CRLF 後以 Windows text mode 寫入，形成 CRCRLF。查閱實際 library 簽名後，helper 改四參數二進位寫入；僅修正這兩個已知受影響 Rust 檔案的行尾，保留38個 tag 宣告值及功能。新的 repair helper 驗證無殘餘 CRCR。之後 fresh v2 重跑；不能改舊失敗 log 或宣稱第一輪通過。

## E762：正式 script DLL 無 KCP feature 時共用通訊模組引用錯誤（2026-10-11）

`communication-public-ffi-ue-build-v1` 在正式 `base_content --features compiled-content-only` 建置失敗 E0433（及衍生 E0408），尚未進入 UE 編譯。原始 build.log／summary 保留於 `.ai-collab/communication-public-ffi-ue-build-v1/`。先前 core default／bridge／backend checks 啟用 KCP，script tests 的 dev-dependency feature unification 也會遮蔽此問題。不能為 script DLL 強加 KCP、複製 magic number 或跳過正式 DLL。將完全相同的 wire tag 常數移至無網路依賴的共享模組，KCP 原 public path re-export，通訊 policy 從共享模組讀取。補 no-default-features compiled-content-only check，再重跑正式建置。來源盤點另發現一個未執行的舊 client fixture 還斷言 presentation6，已依實際 version7 更新並納入整個 presentation 測試群；不冒稱它已實跑失敗。

2026-10-11 E761 修正後 fresh `communication-public-input-tests-v2` 實跑53個不同測試及四項 maintained all-target checks，零 ignored／重複；警告保留。831 source fingerprints 相對前一接受點新增1／修改10／無刪除；legacy 排除稽核通過。接受範圍僅 Rust renderer producer，schema3／presentation7／ABI25；尚未公開 C ABI／UE 啟用、無新 authority game。正式父項仍9/44，繼續由 Codex 直接實作。

## E761：新增 communication intent 後遺漏兩個 exhaustive match（2026-10-11）

`communication-public-input-tests-v1` 的 core transport 測試通過，但 client 編譯 E0004：`input_bridge::validate_intent_shape` 與 `renderer_fixture::intent_value` 未處理新 Communication 分支。原始 stderr 保留於 `.ai-collab/.bridge-data/worker-pool-machine-v1/communication-public-input-tests-v1/client-communication-stderr.log`；後續步驟未執行。直接修正為 gameplay validation 明確拒絕通訊、fixture 記錄其真實 typed intent。新增 oneof 時必須搜尋全部消費者並通過 maintained all-target checks，不能只編 core 或把通訊轉成遊戲 ACK。修復後另跑 fresh v2，尚未宣稱通過。

## E760：公開 Unreal target producer 不能提供 canonical generation（2026-10-11）

此為直接接 API 時發現的來源缺口，不冒稱已跑出失敗測試：ReplicaHost::secure_reference 只提供 filtered replica／view／disclosure epoch，FilteredRenderEntity 和 DemoRenderState 也沒有 authority canonical generation；網路 ClientVisibleTarget 原先卻強制 generation 非零並和 canonical generation 比較。不能把 UE OmEntityRef 的 disclosure generation 當 canonical generation，也不能猜1。

通訊 protocol/schema 明確升3：generation=0 代表由 authority 目前已揭露 mapping 取得內部 generation；非零保留相容性 assertion，仍須與 mapping 相符。現有 replica／disclosure／view／team／retired guard 全保留，TargetClaim 內部 generation 仍從 canonical mapping 產生，不從 renderer 猜測。新 renderer intent 不含 canonical identity，presentation protocol升7並分開本地 token／network request／negotiated 狀態。此批尚未生成或驗證，不沿用已接受 schema2 的49測試冒稱 schema3/API通過。

## E759：升級協定後 unsupported fixture 固定值變成有效版本（2026-10-11）

實跑 communication-content-status-codegen-v1 三階段通過（19 outputs／20 Lua inputs）。tests-v1 的 model2、compiled catalog1、runtime compatibility1、ledger15 通過；transport26通過／1失敗，後續 client/bridge/checks未跑。失敗在 communication_transport.rs:2829：原 fixture 固定 `protocol_version=2` 期待拒絕，但新協定正式版已是2。改成同時測舊版1及 `COMMUNICATION_PROTOCOL_VERSION+1` 都拒絕，保留真實協定檢查。原 log/summary 不覆寫，fresh tests-v2 待實跑。另新增五種互動必須有真正 owned Hero 與 Playing 的成功／拒絕測試，媒體 reference 不允許不同語意共用同一資源；這是來源審查強化，不冒稱測試失敗。

## E758：後端通訊接線複製 Fetch 而非 MobaMatch（2026-10-11）

實跑 team-transport-direct-tests-v1：42 項測試、core/client all-target checks 通過；backend-all-targets 因 src/state/core.rs:2146 的 E0308 失敗，bridge check 尚未執行。`.map(|state| state.clone())` 複製 specs Fetch 包裝，得到 Option<Fetch<MobaMatch>>，不能傳給 Option<&MobaMatch>。改為 `(*state).clone()`，在有待處理請求時取得 owned match snapshot；保留空佇列早退與既有 live World facts。只重跑受影響 backend/bridge checks，不覆寫原失敗紀錄。

另一次搜尋誤用 omoba-client-runtime/src/renderer_protocol.rs（檔案不存在）；已由 rg --files 定位真正 omoba-core/src/renderer_protocol.rs。後續先定位再讀取。

## E757：通訊回歸 fixture 順序與型別不符正式路徑（2026-10-11）

實跑 team-transport-repaired-tests-v4：ledger13通過，transport23通過／3失敗；保留原stdout／stderr／summary。失敗分別是：被拒的duplicate未推進准入時鐘，fixture卻立即退回同一時間期待RegressedClock；換run已清理舊stream，fixture卻期待舊run floor保留；hello fixture未像正式maybe_send_communication_hello先note_transition，後續第一次view觀察被當成新transition。另remainder-tests-v1的framing1通過，client-envelope在編譯前因RuntimeReady傳入RendererReady而非RuntimeReadyPresentation引發E0308，client測試0執行，其餘步驟未跑。

依使用者最新指示，改由Codex直接實作、不再派發子代理。保留原拒絕／重播／換局斷言，新增成功nonce6後才用nonce7測真正回退；舊run floor明確為0並檢查新run floor仍保留；hello先觀察實際view；改用正式RuntimeReadyPresentation型別。原失敗不覆寫，fresh team-transport-direct-tests-v1正在執行，尚未接受。後續必須檢查實際結果，不能把修fixture冒稱修了四個正式產品缺陷。

## E756：tests-v3 在第一個測試編譯失敗；scope cursor 與 delivery 鎖是來源發現（2026-10-11）

已執行的失敗是 team-transport-repaired-tests-v3：exit 101，零個測試跑到。`E0618` 在 `omoba-core/src/runtime/communication_transport.rs:3393`，同函式 3349 的區域變數 `authority` 遮住測試工廠 `authority()`，`votes_authority` 因此無法呼叫工廠。官方 codegen-v3 三階段已通過，不改寫 codegen-v2 的失敗紀錄。沒有 check 或其他測試通過。tests-v2 是 Primary 計畫 metadata 的 scope-cap 錯誤，與這次產品編譯失敗不同，已另行修好。同一次 stderr 還有 3511 未使用的 `presented`，以及既有 authoritative_mode prost import／template warning；那些是警告，不是這次 exit 101 的原因。

以下兩項是審查時的來源發現，不是已執行的失敗 repro：

1. `admission_cursor` 只有 `next_sequence`。換 scope 後第一個成功事件把序號重設，第一個事件結束時游標又是 2。舊 scope 若剛好只有一個事件，新 scope 的第一次成功准入不會 `commit_nonce`。
2. `spawn_communication_delivery` 先讀 registry 的 generation／clock／origin，放下鎖，再 `await sessions.lock()`。等待期間 scope 可以退休、binding 可以換掉。之後仍用等待前的快取，而且 receipt／replay 只比 `communication_generation`。同一 generation 的新 binding 仍可能送出舊的 receipt、replay 或 event。

本筆來源修正（未編譯、未測試）：測試工廠改名 `authority_fixture`，呼叫點跟著改，斷言保留。`presented` 沒有斷言，只拿掉綁定，hello 斷言保留。`AdmissionCursor` 含 room／run／next_sequence，游標不同才 commit。`authorize_queued_delivery` 在拿到 session 鎖之後才讀目前 registry；std `Mutex` 不跨 `await`，持有 session 時只用 `try_lock`，送出用有界 `try_send`。129 筆測試、E751–E755、ABI 25、schema、tags、proto、canonical 隱私與 legacy framing 保留。未接受 transport 或 full 44。原 E751–E755 條目不改寫。

## E755：codegen-v2 在 lib 編譯失敗；nonce、registry tick、field 17 是來源發現（2026-10-11）

已執行的失敗只有官方 codegen-v2：lib 編譯 exit 101，`E0423` 在 `omoba-core/src/runtime/communication_transport.rs:2321`，`ReplicaEntityId` 元組欄位私有，外部 `ReplicaEntityId(target.replica_id)` 不能呼叫。generate／check 沒有跑到。沒有測試執行。summary／proto stderr／errors.md 保留。build script 可能已在 lib 失敗前重生 fallback；該 fallback 不手改。

以下三項是審查時的來源發現，不是已執行的失敗 repro：

1. `admit_live` 只在 `history().len()` 變長時 `commit_nonce`。`MAX_HISTORY` 是 32，其後成功准入不再提交網路 nonce。
2. `State::drain_authenticated_communication` 在 `rx.is_empty()` 或 `tx.is_full()` 時先返回，還沒呼叫 registry 的 `publish_scope`／`publish_clock`。那兩個是 omb 裡唯一的 registry 發佈點。
3. target 測試把 `[0x88, 0x01]` 當成 field 17，但那只是 key、沒有 value；`[0x88, 0x00]` 也不是完整的 field 17 零值。`reserved_canonical_guess` 在第一個 field 17 就返回。

本筆來源修正（未編譯、未測試）：`TeamIdentityState::canonical_for_replica` 在模組內做私有建構與既有 guard；`admission_cursor` 作為新准入身分，游標變了才 commit；`begin_communication_tick` 由 `State` 每個拿到 registry 鎖的 tick 呼叫，空佇列與滿 delivery 也發佈 scope／clock；field 17 掃完整個有界訊息，測試位元組改成完整 varint。Ledger 政策、E752 生命期、ABI 25、tags、omfx、generated fallback 與 proto 未改。未接受 transport、task 5.1 或 full 44。原 E751、E752、E753、E754 條目不改寫。

## E754：確認 target producer 仍吃 client canonical，E753 五行仍在並已從來源移除（2026-10-11）

E752 修復報告自己寫明：`decode_body` 把 `CommunicationSubmit.canonical_id`（proto field 17）抄進內部 `TargetClaim`，filtered client 無法形成該欄位。本次核對修復後來源，該複製仍在，ledger 的 `resolve_target` 仍用這顆 client 提供的 canonical。這是已確認的 producer 缺陷，不是未檢查的推測。

E753 在 E752 terminal 之後複核：`git -C omfx diff` 仍只有 `game/src/lockstep_client.rs` 新增的 `Ok(Some(_))` 五行，與 `moba-communication-omfx-scope-repair-task.md` 記載相同，沒有其他 omfx 差異。

本筆來源修正（未編譯、未測試）：准入在呼叫 ledger 前只拿 client 可見的 replica、generation、disclosure epoch、view，經現有 `TeamIdentityState::canonical_for` 得到內部 canonical。field 17 改為 reserved；非零或非 varint 的 field 17 在 `decode_submit` 拒絕，不進入 ledger。內部 `TargetClaim` 與輸出省略 canonical 維持。omfx 只刪除那五行，周圍分支與 gitlink 未改。未跑 Cargo／omfx build。未接受 transport、task 5.1 或 full 44。原 E752、E753 條目不改寫。

## E753：首版通訊writer越界修改停止維護的omfx相容分支（2026-10-11）

初始transport report提到embedded Fyrox忽略新LockstepInbound；Primary追查正式來源scope以外的Git狀態。team-foundation-review-v1及team-foundation-repair-review-v1保留的root-status均沒有omfx變更，team-transport-review-v1/root-status.log才新增「m omfx」。實際git -C omfx diff僅game/src/lockstep_client.rs新增5行Ok(Some(_))忽略分支及註解，無其他差異。這證實是首版run-mv3awt40-kemrni引入的越界相容維護，不是既有使用者修改，也不能因只有5行就例外保留。

AGENTS明定omfx不新增功能／修相容／編譯或納入驗收。正式828指紋刻意不含legacy目錄，不能只憑正式scope delta聲稱所有其他目錄未變；Git submodule dirty旗標必須追查真實內部diff。當前E752同thread修復run-mv3cd2sc-k2el8a仍是唯一source writer，沒有並行修改／測試omfx，也未中止健康writer。terminal後先核對是否已自行撤回；若仍存在，讓Grok精確移除這5行首版新增內容，保留其餘檔案／gitlink，不執行git restore/reset或legacy build。prepared moba-communication-omfx-scope-repair-task.md。未修復／未驗證前不能接受transport。

## E752：通訊首版測試無法編譯，正式排隊／逐收件者／重播生命周期未完整接線（2026-10-11）

Grok4.7非fast run-mv3awt40-kemrni真正terminal34m7s／150calls／$3.81212352；新增communication_transport.rs、修改13個Rust/proto來源，共828來源，無UE來源變更。初始30m是follower正常timeout，原worker仍在執行，重新追蹤同job直到真正terminal，沒有重啟或第二writer。

Primary實跑team-transport-codegen-v1成功：正式vendored protoc fallback、19檔generate／check，content_hash12804a1a812b658a；fallback來源確實已更新。team-transport-review-tests-v1實際exit101／0passed，測試尚未執行：communication_transport.rs:1640 local authority遮蔽authority()引發E0618，:1668 objective_id的&str未轉String引發E0308。原summary／stdout／stderr／errors.md保留；不得說13測試已通過或只有warning。

正式來源審查另發現：State drain只使用排隊時複製的BoundCommunicationSession，沒有在准入前核對目前連線生命期；delivery向origin_session送receipt／replay也只查字串，沒有核對原binding。所有live收件者共用同一event bytes，receipt_request_id未依非sender清零、view_epoch取sender值，不能支援不同team epoch的all-chat；delivery未核對當前published scope與當前接收者揭露。claimed_view_epoch在submit／replay decode被丟棄，refresh server epoch不能取代驗client intent所屬epoch。

新tags仍走兩份read_framed先依任意u32配置Vec、再任意LZ4 size配置的路徑，decode cap不等於pre-allocation bound。Ledger bounded64 dedup明示允許淘汰後舊nonce重新准入，新transport尚無強化guard；hello可反覆重設replay-once。Ping無原始發生時鐘／到期metadata，replay_for只處理objective expiry，會重新送很久以前的ping；objective history也未去掉同sender已被後續投票取代的舊票。

Client wire shape／receipt驗證不完整：未知outcome／refusal、channel與audience矛盾、控制字元及不適用欄位可穿過部分檢查；apply_hello收到拒絕不撤銷既有agreed binding，source_changed不含team；begin_submit僅檢查pending，新增測試卻期待settled id不可再用。Renderer retain只驗team/player/nonzero，沒綁current safe snapshot epoch或防止較舊generation／epoch覆蓋。兩次hello固定上限與scope retry綁任意下一TeamTickFrame，沒有一般scope／view退休重新協商路徑。這些是獨立來源發現，不冒稱已執行的失敗repro。

本批未接受。準備同thread bounded修復，補正式共享session token／當前scope核對、逐recipient event、allocation guard、stream replay guard與時鐘生命周期、完整codec與client/renderer退休；新增production seam回歸，不以設定列、mock roster或本地echo完成。無authority game／ACK／UE或full44驗收；原失敗與首版來源指紋不覆寫。

已啟動同thread Grok4.7非fast修復run-mv3cd2sc-k2el8a，thread e9eb8f62-c5b1-43aa-b9d9-98d4addc74a3，當前follower25033。baseline team-transport-before-repair-source-v1.json828；與首版review比較唯一新增來源變化是正式生成的game.rs，無其他漂移。修復尚未terminal／未驗證／未接受，不能用active狀態蓋過上述失敗。

補充首版已讀來源的合法target producer缺口：decode_body直接把client field17 canonical_id填入內部TargetClaim，再原樣交ledger canonical mapping驗證；filtered client／UE只持公開replica/render identity，不應持有authority-private canonical ID。首版test從自建TeamIdentityState取canonical放進請求，不能證明真實client能送合法target。修復terminal後須核對是否已改成真正server-side current replica／generation／disclosure／view→canonical解析，且不要求／暴露canonical給client；尚未檢查變動中的修正版，不先斷言它仍失敗。詳team-transport-post-repair-review-gates-v1.md。

## E751：通訊HUD讀取名冊舊副本，mute以match槽位跨scope保存且可耗盡他人容量（2026-10-11）

E751已修復並範圍接受：Grok同thread repair run-mv3a109d-fjup0k真正terminal17m19s／55calls／$1.43452732；delta僅communication.rs與moba_match.rs。fresh team-foundation-rust-v2真正13不同測試／四all-target checks／零ignored或重複，827來源驗證後完全不變；實際warning保留在audit與warning-review。NativeCommunicationFacts讀真World current entity／PlayerOwner／Hero／Inventory，poison cache與live值不同、live變更／刪除／wrong owner／generation及dead-wallet均實跑。mute以room/run scope、合法sender、每recipient16／recipient數32上限處理，首個event前／換scope／拒絕／獨立容量均實跑。ABI25／metadata／art／trinket／staged DLL明示沿用再驗，未新跑UE／authority game／ACK；task5.1未完成。真接受記錄.ai-collab/team-communication-foundation-accepted-v1/summary.json，已立即接fresh非fast Grok傳輸批次，原首版12成功與審查缺口證據不覆寫。

Grok4.7非fast foundation run-mv3971ed-xuio55真正terminal16m59s／55calls／$1.06153576。來源delta為新增communication.rs、修改runtime/mod.rs／native/moba_match.rs／selective.rs，共827來源；其自寫12項communication_測試實跑全部成功，不能因此接受正式producer。

獨立來源審查：communication_owned_ability/item取MobaHeroSlot.hero/inventory快取；record_moba_death才從實際World複製，正式snapshot則讀ECS Hero／Inventory。活著英雄的技能與道具會改ECS，新增test-only setters只改名冊副本，expected又讀同getter，遮住producer錯誤。這是來源已證實缺口，尚無新失敗執行repro；不可記為12項測試失敗。需接真正read-only native World／current entity generation／owner adapter，故意讓快取與live component不同來驗證，缺元件／退役／wrong owner原子拒絕，不加每frame影子快取或退回舊值。

mute目前只有(recipient player_id,sender player_id)，卻跨room/run保留；TrustedSession未提供全球穩定authenticated principal，不能把新room同slot ID當同一人。另set_mute忽略auth.roster，只驗sender非0，合法玩家可填128個非member ID佔滿global容量，阻止其他收件人mute真正sender。需明確scope／替換回收、合法sender與獨立recipient容量，保留同scope重連及拒絕不改state；不能藉假全域身分解決。已準備bounded Grok修復packet moba-team-communication-foundation-repair-task-v1.md，等當前驗證／來源指紋真正完成再啟動。原terminal、source與實跑12成功logs不覆寫；foundation／task5.1未接受，無authority game／ACK。

## E750：較新買單結算後，狀態列回到較舊仍等待交易（2026-10-11）

fresh shop-transaction-viewport-v2 在shop-txn-settled失敗：實際309records／126wires／119stages，catalog相容、authority_ack=false，全部publisher/editor/profile/listener清理成功。前面buy126與admitted／uncertain已通過；結算control119 accepted且published121，但可見行是0 Shop in range · Buy waiting gold=0。原失敗證據保留，不把字樣直接當收據丟失。

來源審查發現ActiveShopTransactionIndex先逆序找任何pending，再找terminal；前綴recipe buy request120未結算，同player/view在source12替換後按既有wallet契約仍保留。較新126結算後即可能被舊120 waiting遮住。原始任務要求實際提交交易的可見狀態、舊結果不得覆蓋新請求，故不能為測試清掉120、強行重設source/view或丟棄舊pending。仍須核對實際correlation／receipt路徑，交Grok修穩定提交順序的狀態選擇與有意義回歸，保留所有old in-flight及有界history／tooltip。禁止按收據到達順序改成最後到達獲勝、偽造結算或弱化owned settled斷言。

E750已由穩定提交順序選擇修復並實際驗證：shop-transaction-build-v9普通nocache成功，fresh native-v5八項全Success，四張真正800x600／400x300原圖目視；fresh owned-v3真正314cases／128intents／123stages、304／125／115前綴與完整cleanup通過，九張當次原圖目視。原126收據映射確實存在，不能誤記為漏收據。舊pending保留，較舊遲到／重播不蓋新狀態；canonical terminal與unsent decline保持可見，既有identified provisional decline在仍有in-flight時保留原讓位語意。826來源驗證後不變，範圍接受見.ai-collab/shop-transaction-accepted-v1/summary.json；非新權威ACK／整體畫面／full44驗收。

## E749：ShopTxn 確認一筆合法買單後未推進 no-extra-wire 基準（2026-10-11）

fresh shop-transaction-viewport-v1 失敗於 shop-txn-buy；native306cases、126intents、116stages，catalog相容、authority_ack=false。publisher/editor/profile退休與listener釋放成功，原證據不修改。錯誤訊息 emitted an extra wire intent 並不證明重複買單：實際125筆舊指令後只有request126一筆 ItemBuy catalog4，source12／actor generation74／target21 generation10、presentation disclosure10獨立保留。repeat=1是先前鍵路由診斷，不是買單重複證據。Primary起初據錯誤字樣稱多送，已向使用者更正。

來源：BeginCase設WatchLines=CaseStartLines；ShopTxn Substep5的PollExpect(ItemBuy/ItemSell)嚴格確認一筆，之後Substep10 PollExpect(None)仍用舊WatchLines，因而拒絕那筆已確認指令。現有其他操作在確認後用CaseStartLines+1移動基準。交Grok4.7非fast核對並精確修復，只能在真正通過one-wire驗證後推進確切一筆；不能設為當前任意Lines.Num、放寬PollExpect或刪no-extra窗口，以免吞掉真正多送。買／賣／stale買三筆仍須128總量，full disabled真點擊仍零新增，314／128／123不變。

E749修復已由fresh owned-v3嚴格驗證：正向one-wire成功後只前移一筆watch基準，完整314／128／123與零額外wire斷言通過；未放寬全域PollExpect或full disabled點擊檢查，原v1失敗紀錄保留。

## E748：續接後工具讀取猜測目錄，與 Windows rg glob 用法錯誤（2026-10-11）

後續disjoint UI唯讀準備又誤猜OmMinimapWidget.h，inventory已確認實際Public/SOmMinimap.h與Private/SOmMinimap.cpp。無來源／測試影響。即使檔名看似自然，也必須先rg --files，不能從類別概念補造檔名。

續接盤點 `.ai-collab` helper 時，`rg --files` 與僅加 `--hidden` 都未列出被 ignore 的檔案；改為限定 `.ai-collab`、`-g '*.lua'` 並加 `--hidden --no-ignore` 後取得實際 inventory。空搜尋結果不代表 helper 不存在；限定目錄解除 ignore，不掃描整個 repo 的建置產物。

後續新增audio API準備說明時先猜了packet標題，apply_patch核對失敗且沒有修改檔案；讀取真實首行後以正確context重做。所有patch context必須來自當前已讀內容，不能由概念或摘要重建標題。此為文件工具錯誤，不冒稱遊戲／音效測試失敗。

另一次Primary工具誤用：verify-current-helper-syntax.lua是固定六檔清單，不讀命令列額外檔名；其成功輸出不能當新acceptance helper語法通過。已讀實際helper，改用固定Lua -e明確assert(loadfile('.ai-collab/record-team-foundation-accepted-v1.lua'))，真正語法檢查exit0。此為工具驗證範圍修正，無產品測試影響，未執行未具備證據的acceptance recorder。

本次唯讀準備又誤猜native/mod.rs及renderer_ipc.rs；實際rg inventory是omoba-core/src/runtime/native.rs及renderer_protocol.rs。無產品修改／測試影響。禁止從一般Rust慣例猜目錄與檔名；先inventory，再讀取。多個rg -g條件是包含規則而非自動AND，廣泛列出.ai-collab會輸出過多歷史檔；改對rg --files結果用單一明確名稱模式過濾，不能再次打印大型minified artifact。

續接又猜錯OmWorldBridge.cpp；正式實際consumer由rg inventory確認為OmGenerated/Private/OmWorldBridgeActor.cpp，RuntimeBridgeSubsystem不是同一檔。準備驗收helper時亦遇JS字串語法與長行patch context不符，均在寫入前失敗；改成讀取完整當前helper、從實際行產生patch，再以固定Lua loadfile檢查語法。不能手抄帶Markdown反引號的長行當成新context，不能以helper語法成功當產品驗收成功。

續接實際又誤猜 inspect-owned-viewport-failure.lua（正確inventory為inspect-owned-failure.lua）及shop-transaction-rust-plan-v3.json；都只是讀取失敗，沒有影響產品或驗證。不得把 summary 裡的 run id 自行轉成未確認檔名。後續需要的 validation plan 應從實際summary.plan或rg --files結果取得。

Primary 曾以不存在的 OmRuntimeEditor/Private/Tests 路徑讀檔，實際為 OmEditor/Private；renderer fixture helper 實際為 scripts/moba_renderer_fixture.lua，非 scripts/lib/renderer_fixture.lua。另 PowerShell 下把 .ai-collab/*ward*.lua 當 rg 檔名會得到 os error123，應用 rg 的 -g 選項；列 ignored .ai-collab 要明確 --hidden 並排除 .bridge-data。這些是工具查詢失敗，不是產品測試失敗。之後先 rg --files 確認路徑，對未產生的 owned startup 檔先 Test-Path，不能把尚未出現的 artifact 當失敗。避免直接輸出 minified fingerprint JSON／過長 log；使用既有比較 helper 或限制單行長度。

## E747：重連「無關聯」fixture 保留前一個 uncertain sell 投影（2026-10-11）

shop-transaction-build-v6 ordinary no-cache 真正成功49.87秒，refresh=false。fresh native-v3 真正7Success／1Fail／0incomplete，process0不等於workflow成功，保留FrameContract刻意負例warning。ShopTransactionStatus兩錯：Reconnect without a correlation 不為true，Empty tracks adopt the current admitted buy 預期Buy admitted實際Sell uncertain。四張新PNG均真正產生且client／root／PNG尺寸符合800x600或400x300；本輪不是完整接受。

來源指向測試State.ShopTransactions在613行仍保留SellUncertain／ForeignAdmit／OldAdmit；652..660 unavailable→available重連只清ShopReceipts，卻稱為「without a correlation」。正式SetEconomyState會重新讀incoming關聯，不能為通過此fixture而丟棄合法重連投影。交Grok4.7非fast核對並精確修正fixture；全部狀態、容量、實際點擊、前景／economy／embedded幾何斷言應保留。原失敗audit/logs不修改。

## E746：CreateWidget 後重複要求 Initialize 成功（2026-10-11）

E745 Grok 來源獨立審查發現三個 CreateWidget 後再次 Initialize 的斷言。UE5.8 UserWidget.cpp 的 CreateInstanceInternal 已呼叫 Initialize；第二次會回傳 false，不能用來證明首次初始化。Primary 改為驗證建立成功且 GetOwningPlayer 是指定 controller，保留 AddToViewport／IsInViewport 與全部實際布局、點擊斷言。未執行失敗測試，不得記為重現。修正期間核對發現 UE5.8 沒有公開 IsInitialized accessor，未將該暫存寫法留入建置。Grok report 的 cleanup 敘述次序也不準確：後宣告的 PlayerRemoved scope guard 先執行，再由前一 guard 移除 widgets、關 viewport 與銷毀 world。之後以 fresh 指紋、build 與 native 檢查此來源。

## E745：真正400x300放大文字下economy超窗；standalone root與game viewport DPI需分清（2026-10-11）

fresh shop-transaction-native-v2 process exit0但workflow失敗：真正7Success／1Fail／0incomplete，保留FrameContract一筆刻意負例warning。ShopTransactionStatus失敗於「Narrow large text keeps the settled purchase inside the economy band」；font／phrase／painted／band／left-of-shop均true，window=false。實際text=(234.5,402.6 242x39)、parent=(234.5,402.5 351x39.2)、toggleX483.5。原logs/index/summary/audit保留，不能以process0當八項成功。

兩張真正400x300 waiting／uncertain PNG已逐張original目視，前景文字可讀、shop仍開啟，pixel header／client／root均400x300；未達settled與rejected capture。這是部分診斷成果，不是四圖或shop接受。此次引擎初始化16.11秒，與先前371.92秒不同；不能因此宣稱E678根因或所有啟動延遲已解決，也無需臆測CEF／授權故障。

來源審查：此test將TakeWidget直接放入SWindow；既有ResponsiveHudLayout則使用正式FOmEditorGameViewportFixture／UGameViewportClient::CreateViewport及AddToViewport，涵蓋實際viewport DPI曲線。不能拿裸window的OS scale代替game viewport的合成尺度，也不能以這一差異直接斷言產品或fixture哪個有錯。需在相同正式viewport路徑下保留economy／foreground／真尺寸／長文字／embedded regression及disabled真點擊斷言，若仍超窗才修正式共用布局。不得刪window斷言、縮字、加容差、硬編DPI取消或只關shop拍照。

## E744：新增前景測試診斷把EVisibility當enum、字型float當整數列印（2026-10-11）

shop-transaction-build-v4正式refresh no-cache真正exit6，UBT OtherCompilationError／95.33秒。OmRuntime生產元件已編譯，但新兩test檔診斷輸出有兩個C2440及三個連帶／格式C7595：EVisibility是struct沒有int32 cast，OmShopPanelTests新增的Text->GetFont().Size是float，不能對應%d。原失敗build.log／UBT-Log與summary保留，ordinary gate尚未執行，不能當建置成功。

Primary僅改三處diagnostic：visibility使用正式EVisibility::ToString()與%s；原生測試實際字型大小使用%.1f，預期BodyPx整數仍%d。沒有round掉實際font斷言、沒有改containment／狀態／disabled／pointer／清單期望，沒有弱化assert或Engine變更。唯讀Visibility.h確認ToString API。新指紋後續跑正式refresh與ordinary gate，再fresh native／owned；同類診斷須依目前UE型別核對，不假設名稱EVisibility就是enum。

## E743：新UI標頭觸發UHT後再次更新NetCore.init.gen.cpp，ordinary gate正確拒絕（2026-10-11）

shop-transaction-build-v3 ordinary no-cache／refresh=false真正exit5 FailedDueToEngineChange，沒有本批C++ compiler error。此前v2 refresh及其ordinary gate均真正成功，不能拿舊成功替代v3。新OmCommandBarWidget.h時間觸發UHT，實際顯示1 generated file written。唯讀檔案檢查確認NetCore/UHT/NetCore.init.gen.cpp時間09:31:40，其他NetCore生成檔仍10/7，NetCore obj則08:54:32；module unity cpp確實include該init.gen.cpp，故新生成輸入晚於舊obj要求重編。dep.json未找到omoba路徑。引擎manifest／NetCore產物列表及完整失敗log保留。

這證明此次阻擋的直接生成輸入，不足以判斷package hash為何又變；不可宣稱UBA故障、generator race、已修根因或一般gate通過。沒有手改Engine／生成檔／BuildId，沒有刪cache或繞過NoEngineChanges。接正式refresh no-cache準備必要引擎產物及本批C++，成功後同一維護流程必須再實跑ordinary gate。這是產物準備，非產品語意權宜修正；重複UHT package變化的根因仍明示待查。Native／owned仍pending，Rust259沿用有四UE檔來源差異證明。

## E742：新商店前景換行取整個host window，未優先用玩家viewport（2026-10-11）

E741 Grok真正terminal19m56s／86calls／$2.35831276，四個正式Unreal來源檔變更。Primary獨立source review發現MeasureShopForegroundWrap優先FindWidgetWindow的整個window local size，只有沒有window時才取ViewportSpace.Allotted；但SOmShopFrame實際按自己的AllottedGeometry排版。嵌入PIE或分割玩家區域可小於OS視窗，故該優先序不符合既有通用viewport契約。這是來源審查發現，尚未執行到失敗，不冒稱native測試已失敗。

Primary改為優先同一HUD normalized ViewportSpace.Allotted，其次由widget cached geometry透過相同FOmHudViewportSpace換算；只有未配置時才以host window作初始化fallback。加上同一正式root在800x600 host內佔400x300實體區域的長拒絕原因回歸，先斷言真正root尺寸再用既有ProveForeground檢查可讀／containment，沒有裁圖或改字級。原四capture與所有交易／disabled事件斷言保留。待新來源指紋、ordinary build及fresh native／owned執行驗證，沒有藉此接受full PIE矩陣或full44。

## E741：交易文字幾何在視窗內仍被商店遮住，截圖名也不代表實際尺寸（2026-10-11）

Primary逐張original檢視失敗native-v1已產生的三張原圖：settled／waiting／uncertain的economy交易狀態都位於前景shop不透明底板後，無法清楚閱讀。ProveEconomyContained只證明bounds不超窗，不證明可見／未遮擋，不能接受「可見交易狀態」。實際PNG header與SHA另存 `.ai-collab/shop-transaction-native-v1-partial-images.json`：nominal800x600為800x603，兩張nominal400x300都是700x528；不能以filename冒充compact實體視窗驗證。第四張rejected未產生。

原圖／SHA／逐張review保留。通用修正需在開啟shop的前景可讀區呈現同一正式交易狀態／reason，復用原owner/epoch/receipt狀態，不製造第二套結算。量測compact／放大文字且保留close與scroll可達；建立真正所需physical-size fixture並檢查PNG尺寸，不只是>0或改檔名。不可只關shop拍照、提高被遮擋背景亮度、縮小字體或減少斷言。E740 pointer修復與此視覺修復串行source writer，全部完成後再重編／fresh native／owned，現仍未接受shop／4.4／full44。

## E740：滿容量點擊測試把無virtual pointer資料的arranged path直接送進Slate route而崩潰（2026-10-11）

fresh shop-transaction-native-v1 真正process exit3，workflow失敗。FrameContract、ItemIconPresentation、JsonlObservation、ResponsiveHudLayout、ShopPanelSearchAndRetirement、ShopRecipePresentation六項先完成Success；ShopTransactionStatus未完成，ShopUndoProjectionAndPin未執行，不得列為八項通過。Engine initialization實際371.92秒，E678啟動延遲仍存在。

真實callstack：OmShopPanelTests.cpp:521 → FSlateApplication::RoutePointerDownEvent → FEventRouter::Route → FWidgetPath::GetVirtualPointerPosition → Array index0/size0。滿容量fixture已經送public ProcessMouseButtonDownEvent／Up，之後又把GeneratePathToWidgetUnchecked得到的純arranged CapacityPath直接route；不能假設這個path帶有pointer route需要的virtual pointer資料。此次沒有產品商店handler崩潰證據，屬實際automation路由錯誤。前三張PNG存在但尚未目視，第四張未產生，不能接受截圖。

保留 `.ai-collab/.bridge-data/worker-pool-machine-v1/shop-transaction-native-v1/` 原log／summary／errors及三張原圖。修復應使用正式公開mouse down/up從真實painted座標hit-test的單次路徑，保留disabled／sequence／pending／owned zero-wire斷言，不force-enable或直接click delegate，不刪除實際點擊測試、不修改Engine或吞assert。查同類新fixture，避免混用arranged path與pointer route。Grok4.7非fast focused source repair後獨立重編與受影響native／owned驗證；父項8/44與完整驗收不變。

## E739：監看混合程序先截前15筆，錯誤推論沒有C++編譯器（2026-10-11）

refresh build-v2在All processes queued後較久沒有完成行，Primary查cl／Uba／dotnet時先依ProcessName降序取前15筆，眾多dotnet節點擠掉cl；另一regex使用cl-也不會匹配plain cl。據此推論當時沒有compiler，啟動唯讀Grok run-mv33qm4g-tlu2z8診斷，這個負面觀測不足以成立。後續未取消建置、未更改引擎設定／cache，日誌自然進展25再36／53；直接Get-Process cl確認三個實際編譯器在運行。先前的完整等待原因仍未知，不宣稱找到UBA deadlock或修復。

正式bridge stop取消不再需要的唯讀診斷，claimed=true／killDelivered=true，原本建置不受影響。memorygroups唯讀decoder證明v2格式正常、15rows、最大average約3367MiB且SHA未變；沒有刪cache、改安全閾值或Engine原碼。預防：判斷指定程序不存在時先完整精確篩選，不對混合清單先排序截斷；沒有完成log不等於沒有在編譯。以實際建置terminal判斷結果，保留觀測與推論差別，不把延遲當作產品錯誤。

## E738：普通NoEngineChanges建置確實需要刷新NetCore引擎產物（2026-10-11）

shop-transaction-build-v1 ordinary local／no-cache／refresh=false實際失敗，正式UE入口exit5。Rust bridge／base_content／19generate／header已建置與stage，UHT processed9generated files後，UBT回報Building would modify existing engine files，包括UnrealEditor-NetCore.dll／pdb、NetCore obj／dep與UnrealEditor.modules／version及平台module manifests。尚未證明本批C++成功，不能拿先前ward build39.38秒或既有日誌當此次結果。

完整build.log與UBT-Log.txt、summary保留，插件相依宣告warnings與git status timeout也保留。這是實際引擎重建需求，不跳過NoEngineChanges、不手改BuildId／manifest／Engine原碼。接既有Lua --refresh-engine／local／no-cache／12parallel流程刷新必要產物，成功後必須再跑ordinary no-cache gate。原始失敗不能覆寫成成功；native／截圖／owned仍未執行，Rust259與六checks／9ignored結果維持。

## E737：商店不確定結果分派攔截了Undo的延遲拒絕（2026-10-11）

shop-transaction-rust-v2實際client131passed／8ignored，bridge103passed／1failed／1ignored。失敗undo_uncertainty_keeps_the_pending_request_and_publishes_busy：收到SHOP_ADMISSION_REJECTED_LATE後pending仍非空，預期既有Undo終結InvalidArgument。新publish_presentation_input_result在查原pending命令類型之前，只依result_code就轉publish_shop_uncertainty；helper發現不是Buy/Sell直接true返回，使Undo合法拒絕被吞掉。

Primary把商店不確定分派移到查到同request與正式command_kind之後，只有Buy/Sell才走原有保留pending／Busy處理；Undo的共用late-admission拒絕恢復既有terminal拒絕，UNDO_RESULT_UNCERTAIN仍保留pending／Busy。原失敗Undo測試不改，既有Buy測試加入expired與late-admission兩碼，確認同碼對兩種真命令的不同政策，無新ACK／refund／resend。以request identity與命令類型共同分派，不只靠相似字串前綴。須跑新正式回歸，原失敗logs與ignored完整保留。

## E736：catch-up幀數測試受真實4ms時限與排程搶占影響（2026-10-11）

shop-transaction-rust-v1實際client library130passed／1failed／8ignored。商店六個shop_correlation_*均成功；失敗是既有draining_available_inbound_applies_a_full_catch_up_batch，applied實際1、預期32。測試從建立tracker到開始drain／plan之間採真實Instant elapsed，4ms時限可能先到，因此不能固定推論一定由32幀邊界結束。未當作商店產品失敗，也不隱藏或重跑碰運氣。

Primary把正式同一policy提取成plan_next_frame_with_elapsed，公開plan_next_frame仍傳真實batch_started.elapsed，正式4ms／32frames／publish／yield行為不變。幀數與buffer測試明確傳Duration::ZERO；時間測試驗deadline前1ns與恰deadline兩側，直接呼叫相同正式policy，不複製判斷。移除測試用force_batch_age與真實牆鐘依賴，不增加sleep、不放寬32幀或略過assert。須重跑正式client suite與其餘未執行步驟，原failure logs保留；8ignored需如實審查並獨立列出。

## E735：新轉換測試同一Vec被移走後又使用（2026-10-11）

Grok E732–E734交付後Primary來源審查：validated_shop_transactions_projects_one_owner_and_drops_a_bad_batch 的reject closure先snapshot(rows)，下一行再snapshot(rows)；snapshot取得Vec所有權，第一次就移走rows。尚未執行編譯，不冒稱實跑失敗。Primary把第一次呼叫改為snapshot(rows.clone())，兩次仍使用內容相同而獨立擁有的合法／非法資料，保留直接validator與正式frame path的兩項斷言。僅測試資料所有權修正，正式validator／ABI／行為不變。預防：同一非Copy測試資料供兩個by-value API時，先clone或分別建立，不能只審library公開API而漏測試本身編譯。

## E734：滿載商店案例只檢查停用／mouse move，未送出真實點擊（2026-10-11）

E730修復後來源審查：native ShopTransactionStatus 呼叫正式 SubmitShopAction 的容量拒絕分支，但後續只有 ProcessMouseMoveEvent／LocateWindowUnderMouse，沒有 mouse down/up；owned shop-txn-full 只確認 IsEnabled=false／CanSubmitShop=false後PollExpect(None)，完全未點擊。這不是實跑native失敗，而是原修復包要求「真正pointer click後零新增wire」的驗收缺口。應沿用現有公開Slate事件與實際繪製座標，送出按下／放開，確認callback／pending不變與零新增wire；不得強制啟用按鈕、直接執行delegate或偽造ACK。

## E733：晚綁定／新交易未一起檢查保留收據與pending的request唯一性（2026-10-11）

Primary新增兩個production history回歸，實跑 shop-repair-boundary-repro-v1：3passed／2failed／0ignored。先pending802/request41、再收未綁定canonical801，note801/request41竟回Completed並偷用request41；先801/request41已成交，再note802/request41竟回Pending。原因note_request先找到authority對應entries即修改並返回，沒有先檢查pending的request占用；一般新pending只掃pending，不掃已完成entries。修復須在修改任一容器之前，以同owner/view完整request↔authority↔action檢查兩容器，衝突原子拒絕；保留同tuple重複與立即晚綁定。兩失敗測試保留，不得放寬。原E730兩測試本次真正成功，不改寫先前失敗證據。

## E732：library回傳型別與binary crate路徑／可見性不相容（2026-10-11）

實跑 shop-repair-binary-check-v1 的正式 client all-targets check 失敗：main.rs八個crate::shop_presentation::ShopRequestNote路徑均E0433，因main binary crate不是library crate；library的pub PresentationHub::note_shop_request卻回pub(crate) enum另有private_interfaces警告。lib測試能通過不代表binary可編譯。應使用明確正式library公開API／正確外部crate路徑並保持型別可見性一致，不能複製enum到main或忽略binary驗證。完整stdout／stderr／summary／errors.md保留。此批來源尚未接受；Lua56自足契約實跑成功，尚未Unreal build／native／截圖。

## E731：Grok 橋接狀態仍為 running，但追蹤程序已不存在（2026-10-11）

Grok 修復工作 `run-mv2zrnrp-xi1jxv` 的 public follower／runs 持續回報 running，沒有 terminal 或修復報告。Primary 先依此等待，未及早核對存活，造成不必要等待。約39分鐘時核對 tracked agent PID82552與bridge PID82688，兩者均不存在；實際原因尚未確認，不推斷模型卡住或主機重啟。透過正式 bridge stop 執行，實際結果 cancelled／claimed=true／killDelivered=false；沒有偽造完成事件、修改 bridge 私有狀態或接受部分來源。

預防：長時間只有相同 progress heartbeat、沒有新公開進度時，同時核對橋接所記錄的程序存活。running 只是儲存狀態，不能單獨當作活躍 worker 的證據。兩程序皆不存在時，保留原工作與失敗紀錄，以正式 stop 清除失效寫入鎖，擷取部分來源指紋，再由同一 Grok4.7非fast thread 恢復限定修復。來源／測試必須重新獨立審查，沒有完成報告不能當已交付。

## E730：商店已成交重查重開pending、晚綁定不處理保留收據，且UI過早終結／派送後才檢查容量（2026-10-11）

Grok run-mv2xmn5z-1tzuiy真正terminal後Primary獨立審查，追加兩個production ShopReceiptHistory測試。實跑`shop-transaction-review-repro-v1`：0passed／2failed／0ignored，完整stdout／stderr／summary／errors.md保留。`shop_correlation_repeated_settlement_does_not_reopen_pending`在已成交後再次note_request同pair時發現新pending；`shop_correlation_late_request_binds_existing_receipt_without_replay`先capture未綁定收據再note_request，仍為request0/view0，預期41/4。note_request只看pending、不看已保留canonical結果；duplicate capture的same分支也未清除重開pending。修正需保持真正冪等、晚綁定立即處理已留結果、衝突pair不污染關聯與容量，不能靠再送購買／等待額外網路replay。

另有尚未跑native的來源審查發現：SubmitShopAction在Bridge送出後Commit才TryKeep容量，64pending時可能指令已送卻無UI追蹤；應在同一提交路徑先保留容量並在返回時重驗發起owner/view/HUD退休狀態。ApplyShopTransactionProjection在檢查action是否匹配前先寫AuthorityInput，錯誤action會污染後續合法收據。NoteShopTransactionCritical把InvalidArgument先設phase5/ResultCode-1，真canonical拒絕的critical通知若早於詳細收據，後續Receipt又因phase>=4被略過，會永遠顯示could not send而不是實際拒絕原因；須區分本地／暫時通知與已確認canonical terminal，兩種到達順序都以匹配收據定案，重複／衝突canonical保持冪等。這些是來源審查發現，不能說native已失敗或已修復。

新native窄版斷言只看單行文字與DesiredSize.Y，沒有驗實際可用寬度；OmEconomyStatus仍AutoWrapText(false)，長拒絕原因加gold/range時尚未證明不溢出。須補400×300／放大文字下實際painted bounds及原圖，按一般量測排版處理，不以縮小字體／藏掉可見狀態替代。此為驗收缺口，尚非實跑畫面失敗。全部交回同一Grok thread限定修正，保留ABI25、40-byte權威收據、313/128/122來源inventory與304/125/115已接受前綴；無新對局／ACK，不接受本批或勾4.4。

## E729：商店關聯測試把外國玩家當成這份歷史的擁有者（2026-10-11）

`buy_sell_correlation_is_bounded_and_only_a_matching_receipt_is_terminal` 首次實際失敗：`capture(4, 10, 8, event(player 8))` 後歷史長度是 2，斷言要 1。正式 `PresentationHub` 只用本地 `player_id` 呼叫 `capture`，外國收據會被 `receipt.player_id != player` 丟掉；這個新測試卻把 player 8 當成歷史擁有者，所以 801 被存進同一份歷史，接著本地 999 變成第二筆。產品過濾沒有壞。測試改為 `capture(4, 10, 7, event(player 8))`，並断言外國收據不留在這份歷史。修正後該測試、fixture `shop_transaction_correlation_follows_the_current_source_and_keeps_gold`、以及 bridge `shop_terminal_ack_requires_settlement_and_preserves_uncertain_request` 都通過。預防：歷史物件的 player 參數是本地擁有者，不是收據裡想被拒絕的那個玩家。

## E728：一般Lua contract test不可強制依賴本機v5暫存紀錄（2026-10-11）

Primary審查E727交付發現，maintained unit test末尾無條件讀取ignored的ward-ui-viewport-v5 artifact，要求原workflow=false／gen74，並每次覆寫strict-reverify.json。這會使新clone、清理Saved後或一般CI無法跑unit test，且把一次事故現場硬綁在正式測試。尚未執行到此分支，屬來源審查發現。移除該末尾本機區塊並存為ignored診斷檔，所有自足fixture與原／新增負例不變；真v5紀錄仍必須由Primary獨立helper呼叫同一public verify_artifacts，凍結15原artifact SHA並寫fresh獨立重驗JSON，不能省略真資料驗證。一般unit test不讀寫歷史Saved報告；產品／nativeproducer／原report未改。預防正式測試不可把個人機器上ignored執行產物當必要輸入，也不可覆寫原報告。

## E727：控制ack沒有獨立view epoch，同view替換時source epoch是actor generation（2026-10-11）

`ControlAck` 只有 source／target render id 與 disclosure epoch，沒有 view_epoch 欄位。`project_ward` 保留目前來源；`controlled_owner_ward_same_epoch_source` 把 actor epoch 設成 view floor+64，floor 本身不進。保留的 ward-ui-viewport-v5 history 第110筆是 source 12／74、target 21／10，其後 projection 仍是 12／74；五筆 WardPlacement 的 disclosure_epoch 與 entity_epoch_floor 仍是 10，最後一筆 source 才是 12／74。不能把 74 當成 view，也不能在 ack 上發明欄位。Lua 契約必須分開追蹤 view floor 與 actor generation。此條是來源與既有 artifact 欄位閱讀，不是實跑，產品與 native 不需改。

## E726：Unreal已通過304操作，但Lua仍只登錄294案例／103控制階段（2026-10-11）

ward-ui-viewport-v5 native.success=true／304cases／125intents／115stages，最後ward-status-rejected成功；workflow.success=false於moba_owned_viewport_ipc.lua:484 control history stage count mismatch。實際Lua CASES結束於shop-recipe-buy，沒有十個ward案例；STAGE_BEFORE/AFTER、history分類與wire validator也未接WardPlacement與同viewepoch actor generation變更。不能只放寬數量或跳過history。須把新增案例、每案例ordered stage lists（status一案例有多個after stages）、ward slot0單點wire與viewepoch/actor-generation分離契約完整接入，維持原294/120/103逐序前綴與所有舊斷言，新增偽造ACK／stage缺失重排／epoch混淆／額外wire負例。原workflow=false與native=true分開保存，不改舊report；若產品與native來源不變，可用修好的嚴格checker重新驗真實v5原始artifact並寫fresh獨立reverification報告，不能把原workflow冒稱成功。所有程序／profile／listener清理成功。委派Grok source-only修workflow契約，後由Primary實跑Lua與真artifact重驗。

## E725：Ward source-retire通過後，後續status案例仍誤要求hero11（2026-10-11）

ordinary ward-ui-build-v11真正成功133.88s，fresh native-v5真正23Success／0Fail／四既有warnings，包含WardAdmissionRetirement。fresh owned-v4在303rows／124intents／111stages失敗於ward-status-uncertain-then-settled的起始baseline gate；前一個ward-source-retire已完整通過並記錄input124，證明E722/E723修復在實際路徑生效。實際ack controlled_owner_ward_projection為source12/gen74、target21/gen10，view保持10，authority_ack=false。project_ward的明示契約是保留目前來源，不會恢復hero11；owned後續兩個status案例卻硬用LivingSource。修僅owned測試：cursor>=8起始及forwarded/uncertain/settled ack使用專用SameViewEpochReplacementSource，增加phase90等實際ProbeHero12/gen74、ownerward view epoch10及preview清除，再進狀態操作。不改fixture、production、cases304／已有prefix、期限、authority語意，不把actor generation冒充view epoch。原失敗與所有正常程序／profile／listener清理保留。Native23與19圖明示沿用因正式來源未變；待fresh普通build／owned-v5。

## E724：事前Ward token檢查還需拒絕同步退役的owner（2026-10-11）

Primary獨立審查E723來源：RetireInput清admission並設bInputRetired，但保留hero與ward快取，OwnerEpoch不在此遞增；若ProcessEvent同步退役後仍返回accepted，WardAdmissionIssueStillBound原條件可能全部為真，使Note重新寫回舊請求。這是具體來源分支審查發現，尚非實跑失敗。加入!bInputRetired至共用事前token驗證；既有automation入口的retired guard保留，native末尾補退休後positive id拒絕與admission仍0斷言。該新增斷言驗公開automation入口，不冒稱實際重入ProcessEvent已測。無新state鏡像／路由／ACK。待普通build／affected native／fresh owned。

## E723：Ward admission 綁在送出當下的 actor，英雄更新才清掉舊 request（2026-10-11）

ward-ui-viewport-v3 的 ward-source-retire 在 Substep105 已看到 Probe hero 12／generation 74、owner ward view epoch 10，隨後 180 frame 內 GetWardRendererAdmissionId 仍非零。根因是同一份 presentation snapshot 先 ApplyOwnedWardFromFrame／PublishOwnedWardView，後才 PublishHeroHudState。UpdateOwnerWardState 用當時的 CachedHero 比對；那仍是 input 124 送出時的 hero 11／generation 10，view epoch 與 player 都沒變，所以三個退休條件都是假。接著 UpdateOwnerHeroState 換成 hero 12／generation 74 並清掉瞄準，但沒清 WardRendererAdmissionId。該幀之後沒有下一筆 ProcessFrame（OM_PRESENTATION 停在 seq 111／tick 110；tick 111 的 OM_MATCH_STATE 在同一幀且落在 2 秒 log 節流內；約 181 frame 後 teardown）。這次 OM_WARD input=124 在 frame 494，hero 12 在 frame 502，事後抓取並沒有把 generation 74 蓋到舊 request 上；但 ProcessEvent 若同步發布替換，事後抓取會把新 actor 寫進舊 request。

修復在 UOmPlayerActionOwnerComponent。SubmitWardIntent 在 ProcessEvent 前記下完整 actor id／generation、view epoch、owner player 與 OwnerEpoch；接受後只綁這份事前身分，同步替換、disclosure 消失或 epoch 已變則立即退休、不讓新 actor 繼承。UpdateOwnerHeroState 在 id 或 generation 改變時清 admission；同一 id／generation（含 hp 0）保留。ClearHeroSnapshot、view epoch、owner player、OwnerEpoch 重綁與 RetireInput 也清。Correlation／outcome 不寫回 admission。只在 pending domain 仍是 Ward 時取消該瞄準；英雄替換原本就會取消全部瞄準。沒有自動重試、假 ACK、本地放置或改 ABI。Native 新增 Om.Runtime.WardAdmissionRetirement。尚未編譯、尚未重跑，尚未接受。

## E722：同 view epoch 的真英雄替換未清除舊 Ward renderer admission（2026-10-11）

ward-ui-viewport-v3 真正失敗：302cases／124intents／110stages，ward-source-retire kept the renderer admission。E720/E721專用gate已實際看到hero12／generation74與owner ward view epoch10，故這次不是canonical hero或不存在ACK欄位錯誤；新frame／來源替換已確認，但GetWardRendererAdmissionId仍非零直到原期限。authority_ack=false，editor／publisher／profile清理及listener_released全通過，失敗logs保留。正式UpdateOwnerWardState目前以captured entity完整id/gen判斷退休，需追查真正capture時序、hero更新與ward更新順序，不能以延長期限、fixture手動清除、取消斷言或偽造結果通過。委派Grok做一般owner/source/view epoch/disconnect生命週期修復，補實際owner regression，待fresh普通build與受影響native／owned，尚未接受。

## E721：owned 修復誤把 ready metadata 欄位當成 control ack 欄位（2026-10-11）

Primary對E720的首版修復要求ControlAck.entity_epoch_floor，ward-ui-viewport-v2再次在302case失敗。獨立解碼實際ack：source12/generation74、target21/generation10、authority_ack=false，沒有entity_epoch_floor；Rust ControlAck結構也沒有該欄位，之前看到的欄位屬另一份ready metadata。這是Primary的schema判讀錯誤，不是正式狀態失效。移除不存在的ack欄位要求；ack仍嚴格驗來源替換／目標不變，隨後以實際ActionOwner.GetOwnerWardView().ViewEpoch==10及Probe hero12/generation74新frame確認view epoch未變，再驗舊admission退休。未放寬原baseline，未改fixture或production，未編造協定欄位。原失敗／124intents／110stages與正常cleanup完整保留。預防每個欄位必須確認所屬struct與實際payload，而非同檔字串搜尋就推定schema。

## E720：同 view epoch 替換來源案例仍用 canonical hero 11 gate（2026-10-11）

ward-ui-viewport-v1 原生報告列302cases／124intents／110stages時，在ward-source-retire報baseline ack did not refresh source11/target21，未接受。所有程序／profile正常回收，listener_released=true、authority_ack=false。實際fixture controlled_owner_ward_same_epoch_source刻意把來源換12、actor generation升至epoch_floor+64，但保留view epoch與target；新增owned案例卻以LivingSource要求hero11及generation等於viewfloor，與欲驗契約相反。新增專用SameViewEpochReplacementSource expectation，從真ack驗entity_epoch_floor仍等於ViewEpoch、來源12且generation更新、target身分未變，保留原LivingSource嚴格條件與原source replacement新view世代分支。再等實際Probe看到hero12／ack同generation的新frame、preview清除及舊ward admission退休；不改正式owner／fixture／協定，不把控制ack冒充權威ACK。原失敗證據完整保留，待fresh普通build／owned rerun。這是測試身分預期錯誤，不能透過放寬所有baseline檢查解決。

## E719：新增資源條測試假設 SProgressBar 有 GetPercent（2026-10-11）

ward-ui-build-refresh-v4 實際C2039四處GetPercent不存在，C2737與format診斷為連帶錯誤，UBT75.85秒，原log保留。Primary獨立讀取UE5.8 Slate/Public/Widgets/Notifications/SProgressBar.h確認只有SetPercent，Percent為private，不能編造getter或改Engine。修新增的HeroResourcesStayPainted為實際paint geometry／可見／bar extents／不重疊／roomy parent回復斷言，移除尚未能編譯的四處getter呼叫；不改任何原有fit／字級／stats／輸入斷言，也不增加只為測試的正式值副本。既有SetHealthPercent與SetManaState的clamp／比值及percent更新來源未改；本新增geometry測試不宣稱直接讀取fill比例。預防寫UE API呼叫前先讀實際版本header，來源review不能代替正式編譯。待fresh build及targeted native。

## E718：v5 明示 Ward G 與預設 PingWheel G 重複，整份設定被拒（2026-10-11）

ward-ui-native-v2 的 ControlProfileMigration 在補上 slot 後，OccupiedFour 已通過，其餘三個明示 Ward 斷言仍失敗。這不是 E714。v5 JSON 已能解析；ImportJson 從 MakeDefault 疊上 Ward 主鍵 G，但目錄預設 PingWheel 主鍵也是 worldCombat 的 G。ReleaseOccupiedDefault 只放開未提及的動作，PingWheel 仍占用 G。Validate 的 ConflictMessage 為「Ward primary (pressed) and PingWheel primary (pressed) both use G in worldCombat.」。ImportJson 在寫回 this 前直接拒絕，記憶體維持建構預設，所以載入、保留 G、Stop 仍為 Four 三個斷言一起失敗。native log 只有這三條 TestTrue，沒有另印診斷字串；訊息來自正式 ConflictMessage 與目錄順序。正式驗證、parser、profile 5 與預設綁鍵不放寬。修復是同一份 v5 明示文件以空 key 解除 PingWheel，並另用未解除的文件斷言 DuplicateChord 與上述訊息。尚未接受。

## E717：文字縮放 1.25 的 400×300 戰鬥欄高度仍超出（2026-10-11）

ward-ui-build-v6 普通閘門已通過。ward-ui-fit-native-v1 實跑 ItemIconPresentation Success、ResponsiveHudLayout Fail，唯一錯誤仍是「Text scale 1.25 at 400x300 fits」。最接近的 outer 80、stack 0、combatW 296、inset 8、limit 284、combat 291×293、hero 57、ability 104、economy 22、items 88、stats 151×44。寬度 291 在 296+2 內，高度 293 比 284+1 高出 8。joint 24 組都已寫進同一條 fit；最短是 a0s1w2 的 ability 104（前一次 102）。寬欄 384 的 ability 71 其高度上限只有 196，不可行。技能／召喚師／眼位的聯合排列已經實作並在這次 native 獨立跑過，仍然失敗，不是剩餘根因。英雄欄在 column 242、name cap 78 時把名字與 160 換行的數值並排，ResourceHost 仍是其後的獨立直帶，數值 44 加血魔列 13 等於 hero 57。本輪改為量測一般排列：保留原獨立資源帶，並比較把名字與兩條資源條放進數值旁同一欄的直排與橫排。只有量到的內容寬不超過欄寬、英雄帶確實更短、血條與魔條都達到 28px 且高度保留 8／6 時才採用；等高則留較寬的條。直排讓兩條都用 name cap。Roomy 與不並排的窄欄把 ResourceHost 掛回 OmHeroColumn。字級、圖示、160 換行、169 寬斷言、300 高度、容差與顯示資料未放寬。24 組 joint 保留。此來源尚未重跑，尚未接受。

## E716：新增眼位卡後 400×300 已有道具圖示排版失敗（2026-10-11）

ward-ui-native-v1實跑ItemIconPresentation在400×300 fitted=0。原完整OM_HUD_FIT log保留：80px minimap並列candidate combat292×301、可用高度284，stack candidate343×275但可用196；不能靠增加timeout、忽略fit、藏卡片或縮字至不可讀通過。需將新增眼位卡的實際paint需求納入一般responsive composition/budget，保留六格圖示、文字、4快捷鍵、狀態與既有minimap/tooltip/hit契約，重跑原實際窗口案例及owned。此為實際回歸，尚未接受。

## E715：HUD registry 測試把未匯入狀態當成永遠不變的出貨前提（2026-10-11）

ward-ui-import-v1真正import1/save1、second verify1/save0並發布resource後，ward-ui-native-v1的HudAssetRegistry有三個IsEmpty斷言失敗。正式資產已正確匯入並獨立SHA驗證；測試不該要求它永遠不存在。source-only允許／已匯入resource及拒絕原子性須分開以明確fixture驗證；真實manifest驗正確種類路徑／已匯入resource，不能移除正式resource來迎合測試。原15Success／3Fail及全部warnings保留。

## E714：v5 快捷鍵遷移新增 fixture 漏 mandatory slot（2026-10-11）

ward-ui-native-v1實跑ControlProfileMigration七個新增斷言失敗。兩份v5 JSON的Stop／Ward row漏slot；ParseChordObject明確要求dual-slot版本帶primary或secondary，於migration前即InvalidData返回。修fixture使用真實v5 schema或正式export產物，保留個人Four／WardG／unbound／MigrationConflict原斷言，不能放寬正式parser或刪除失敗檢查。待修復與fresh native重跑。

## E713：飾品生成型別在 namespace 內，正式 consumers 漏限定（2026-10-11）

ward-ui-build-refresh-v1實報C2065/C2146：OmTrinketCatalog.h宣告OmContent::FOmTrinketIdentity，但WorldBridge、OmItemIconImportCommandlet兩處及OmUiTextureImportTests使用未限定名稱，後續Identity/TestTrue錯誤為連帶診斷。同步修正四個使用處；生成器本身的namespace內實作合法、不改已正確生成產物。OmWorldBridgeActor為既有手寫owner，codegen未生成該檔。Rust143／六checks不受此C++來源修改影響，仍明示保留；UE需要fresh重建。原完整UBT log保留，防止只查Rust就宣稱UE可編譯。

## E712：HUD registry 資源路徑變數遮蔽 JSON Root（2026-10-11）

ward-ui-build-refresh-v1實報OmHudAssetRegistry.cpp:80 C4456，新const TCHAR* Root遮蔽同函式的TSharedPtr<FJsonObject> Root，正式UE把此警告視為error。改名ResourceRoot，無行為改變。原失敗編譯1001.42秒及全部diagnostics保留；需fresh重建，不能關warning或改compiler設定避開。

## E711：普通眼位 UE 建置實際要求 Engine metadata refresh（2026-10-11）

續記：新增HeroColumnStack member的 ward-ui-build-v7 實際仍以 FailedDueToEngineChange 結束（UBT5.96秒），不是 C++ 成功；全log保留。沿用同一正式 refresh→普通 gate 程序，不手改引擎metadata、不把refresh本身冒充普通建置。

續記：ward-ui-build-v4 在新的 C++ header 修改後再次實報同一 FailedDueToEngineChange（UBT75.04秒），原 build.log／UBT-Log.txt 保留。這不是已通過的 v3 結果，也不能沿用 v3 宣稱新來源建置成功。重新以正式 refresh 流程更新產物，再重跑普通 NoEngineChanges／nocache。未修改 Engine source/config/BuildId。

ward-ui-build-v1正式NoEngineChanges／NoUBA／nocache在UHT後報FailedDueToEngineChange，UBT327.94秒，列出需更新的Engine modules產物。Rust143／六checks已過，但不能當UE成功；原build.log與UBT-Log.txt完整保存。依既有正式build入口做維護refresh，再必須重新通過普通NoEngineChanges／nocache；不手改Engine source/config/BuildId、不以refresh取代普通驗收。此為實際建置gate，尚未成功。

## E710：ABI24 的正式 C++ smoke 仍斷言 ABI23 結構大小（2026-10-11）

實際cbindgen ABI24生成成功後，ward-ui-header-smoke-v1的MSVC編譯報三個C2338：FrameHeader仍假設Undo pointer結尾、InputCommand仍96bytes、FrameEntity仍72bytes。Rust正式layout test已改為owner_ward結尾／ward_slot追加／entity88，C++ smoke只升版本常數而漏接結構斷言。同步保留原offset並新增ward_slot96／command104、owner ward416／correlation32及其offset斷言；原compile.log保留。預防ABI變更必須實跑雙語layout，不可只比較版本字串或手寫header。待fresh smoke與Rust實跑。

## E709：貼圖匯入器用 minified 字串與括號掃描修改 JSON（2026-10-11）

Primary 實跑 ui-registry-json-probe-v1：相同合法registry加入冒號空白後，publish_registry_resource報「registry entry is missing」。原函式尋找固定的無空白字串，再以不懂JSON字串的括號掃描判斷物件邊界，並非通用JSON處理。改為既有json.decode按entries／kind／id定位唯一entry、驗resource後encode；同resource原bytes直接返回，重複identity與不同resource拒絕。回歸涵蓋合法空白、字串內括號／假id、重複entry、匯入前後皆可執行的測試fixture。原失敗probe保留；待實際Lua及後續匯入驗證。

## E708：不相容眼位私人 codec 沿用相同協定版本（2026-10-11）

Primary 在 run-mv2l2z9e-cvv6r4 terminal 後確認：WARD_PROTOCOL_VERSION 仍為2，但 OWNER_WARD_SCHEMA 已從1／47bytes改為2／147bytes，catalog data hash不變。舊、新peer以相同版本與hash協商成功後，才被私人bootstrap decoder拒絕，違反不支援眼位只停用該能力的契約。修復需把不相容codec的能力版本升至3，保留selective7／presentation6／envelope1各自邊界，實測舊2關閉眼位但其他加入／操作保留，以及新3的已驗bootstrap／recovery。不能只改schema而不改協商版本。本輪已在來源把 WARD_PROTOCOL_VERSION 升至3；舊2不再協商成功，空投影與攜帶47bytes payload 都只清眼位、不失敗 selective bootstrap；協議3的既有 sealed bootstrap／recovery 測試仍跟隨常數。schema 2／147 與 envelope 1 未改。cargo check 已編譯這些測試，尚未執行。尚未接受。

## E707：插眼卡片把 renderer request ID 當作權威 input ID（2026-10-11）

獨立檢查 run-mv2hxi7a-8r4xfy：OmCommandBarWidget::BuildWardStatusText 直接把 WardRendererAdmissionId cast成u32比對私人 outcome.Correlation。正式 omoba-client-runtime InputBridge 從 last_seen_input_id 獨立分配權威ID，main 保留 input_id→request_id；renderer ID不保證相等，重連後尤其如此。UI未使用這份關聯，合法結算可能永遠等待；ActionOwner只按epoch清除追蹤，沒有owner／actor life範圍。不得靠相等ID的synthetic fixture當完成。修復需有界保留真實兩端ID及owner/epoch/actor關聯，區分admission、terminal、uncertain，處理foreign/late/source retirement，並以renderer1／authority901等不同ID實際驗證。本輪已把關聯寫進來源（snapshot tag 25、OmOwnerWard append-only、卡片比對權威ID、owner／epoch／actor退休），仍未接受：Primary尚未實跑受影響檢查、native／owned。

## E706：手寫飾品ID常數繞過生成catalog（2026-10-11）

本批新增 OmGameplayInputTypes.h 的 OmGeneratedTrinketCatalogId=1／固定content ID，但它不是生成產物；ActionOwner、WorldBridge及CommandBar均用這個ID gate，名稱與tooltip也固定偵察燈。與Lua catalog唯一來源及通用方案要求不符，新增合法飾品或更動catalog ID會被前端拒絕。修復需由實際content/codegen產生catalog查找與顯示／icon identity，正式consumer按收到的已驗catalog查找，不用第一行／字串派送／手寫1。沒有要求新增實際遊戲飾品；測試使用獨立多行／非1生成fixture驗泛用查找。本輪已移除手寫gate，改由 trinket_catalog 生成查找與 WorldBridge 寫入view；手寫的已檢入產物只含出貨列，待Primary重新生成。尚未接受。

## E705：新增插眼point動作被舊catch-all target gate拒絕（2026-10-11）

Grok公開進度與交付source指出validate_input_command未列入InputWardPlacement時，舊catch-all要求InputTargetNone，合法point會在提交前被拒。本批已加專屬point分支，尚待Primary實跑reader／wire／owned事件。預防新增action時同時盤點型別、轉換、通用validator、正式submit及事件測試；在交付前修正仍須留防錯紀錄，不能因不是剩餘問題就略過。

## E704：renderer fixture用了權威WardPlacement而非WardPlacementIntent（2026-10-11）

實際worker首輪cargo check在omoba-client-runtime/src/renderer_fixture.rs:3674報E0308：Intent::WardPlacement要求WardPlacementIntent，卻傳入WardPlacement；core成功，其餘未到達。原始%TEMP%/ward-ue-preflight.log有實際compiler診斷。worker修正後第二輪client/bridge/backend/scripts checks成功，但未執行測試；本批尚未接受。預防先確認RendererInput與PlayerInput各自oneof型別，保留失敗與修復檢查紀錄，不能把型別檢查當事件驗證。交付報告原稱No E704，Primary補登已發生錯誤。

## E703：舊 visibility fixture 未註冊新增的 PlacedWard storage（2026-10-10）

ward-private-final-rust-v3 前16步通過，visibility實際6 passed／1 failed；disclosed_combat_relation_codec_rejects_malformed_bytes_and_emits_only_real_factions 手建 World 缺 MaskedStorage<PlacedWard>，build_wave_b_read_view 取用時 panic。正式 constructor 已註冊；補上該舊 fixture 的 storage，保留無Faction／敵軍／死亡友軍原斷言，未修改正式可見性政策。同類E699只補combat fixture，未涵蓋此手建 visibility World；預防新增正式 storage 時一併檢查所有受影響的手建 World，並實跑回歸。失敗logs保留ward-private-final-rust-v3，後續未跑步驟不列PASS。

## E702：重送淘汰測試的新增 ID 區間撞到既有請求（2026-10-10）

ward-private-final-rust-v2 前六步通過，ward-authenticated-input-buffer 實際失敗：fixture 先使用 ID81，再把 2..257 當作全新請求，其中81的 tick／座標與原請求不同，正式去重流程正確拒絕。將初始有效請求及其重送／衝突斷言統一為 ID1，讓後續256筆新請求確實唯一，並驗證 ID1 被淘汰、epoch退休及新ID接入。保留原去重檢查；後續未執行步驟不列為通過。預防：有界 journal 測試先確認填滿區間與既有ID不相交，不能只編譯不執行。原失敗 logs 留存 ward-private-final-rust-v2。

## E701：第二位玩家的私人資料 fixture 沿用第一位玩家身分（2026-10-10）

獨立 ward-private-final-rust-v1 的 ward-codec 實際 7 passed／1 failed，後續步驟未執行。verified_owner_ward_receiver_rejects_unverified_order_and_restores_by_recovery 在 bootstrap 第二位玩家時報 MalformedBaseline：外層 owner=8，內層 projection.player_id=7。這是正確的身分檢查拒絕不一致 fixture。修正為使用 owner=8 的獨立 canonical projection，保留兩位玩家 canonical team hash 相同的隱私斷言，未放寬正式驗證。預防：多玩家 fixture 必須同步 owner、session、epoch 與內層身分；cargo check 不能取代實際執行測試。失敗原始紀錄保留於 .ai-collab/.bridge-data/worker-pool-machine-v1/ward-private-final-rust-v1。

## E700：private verification preflight 再次發現 fixture scope 與跨 await guard（2026-10-10）

ward-private-verification-check-v1 core編譯失敗：新combat helper未import TAttack／HeroCommandQueue／Outcome／run_now／hero_tick，observer runtime缺mut；其餘checks未開始。v2五checks成功後修改recovery tick；v3前三checks成功、backend join的if-let scrutinee std::MutexGuard生命週期跨sessions.lock().await造成非Send，scripts未開始。v4最後五checks真正exit0。原failed logs/summaries與repair report完整保留。預防：fixture helper用同module實際imports；需要&mut的receiver顯式mut；std guard在獨立block clone後釋放再await，if-let scrutinee也會延長guard生命週期，這是E696同類錯誤的重犯，不能只看block內沒有await。另外bytes clone及舊version1斷言／epoch fixture檢查修正亦保留於report。尚未執行新的功能測試，不能當scoped接受。下一新錯誤 E701。

## E699：眼位修復後世界8通過，索敵回歸23通過4失敗（2026-10-10）

ward-transport-codegen-v1 真正 vendored protoc／17generate／17check 成功，817來源指紋真正結束後執行 ward-transport-repaired-rust-v1：codec6／route1／world8全過；combat27項23Success／4Fail／0ignored，後端三步未跑。三個既有 current_match_attackability 測試缺 MaskedStorage<PlacedWard> 註冊；新 filtered ward 測試僅移除 component，仍保留 entity與 unrestricted legacy disclosure，卻期待真正 hidden target 拒絕。需初始化完整 fixture storage、使用實際 membership／disclosure hide/reveal，保留普通 kind0 合法語義與所有原斷言。原 logs／failed summary 保留，完整功能仍未接受。

## E698：私人 bootstrap 未驗證與 continuing 同一份 sealed hash／absence 直接中斷（2026-10-10）

restore_owner_ward_bootstrap 只 decode TeamGameStart.owner_ward_projection，沒有 expected private hash/envelope；私人 bytes 正確不進 team hash，因此仍缺 bootstrap 的獨立完整性驗證。另 negotiated+empty 直接 MalformedBaseline，未提供已認證 owner 的 canonical unavailable／後續恢復契約。需重用 owner sealer/verifier、明確追加bootstrap欄位及版本、驗 echo與owner/session/epoch/tick，缺狀態只關放眼能力並保留正確綁定；不能本地重算 hash 就宣稱驗證，也不能洩漏team/observer。尚未修復。

## E697：拒絕的私人 envelope 可用未驗證 tick 汙染恢復 frontier（2026-10-10）

apply_verified_owner_ward_envelope 在 hash/schema/owner/session 拒絕時仍將 envelope.authoritative_tick 傳入 invalidate，後者可推進 tick並設 revision=u64::MAX。壞 hash配u64::MAX tick 會讓之後正常合法訊息永久 stale。三個正式 receive paths僅log，只有rebase才query；server change-only時暫停中可能無法恢復。需只保留最後已驗證順序、清capability並以有界/coalesced唯讀query恢復，測 malformed future/MAX後正常tick／same-order恢復及跨owner/epoch；未驗證header不能決定可信版本。尚未修復；目前沒有直接執行 verified receiver狀態機的測試。

## E696：worker preflight 的已修正 compiler errors 也必須保留防錯紀錄（2026-10-10）

ward-transport-review-repair-check-v1 core E0382 moved health；v2前三checks成功，後端失敗：MutexGuard跨await導致非Send、WardAdmissionEpoch E0614、late path input移動兩次、WardPlacement Option<i64> fixture用裸i64；scripts未開始。v3五checks真正exit0，但沒有執行測試。原failed summaries/logs與修復report保留。預防：複製owner rows後釋放guard才await；明確fetch的解參考層級；move前保留必要clone；合法raw座標用Some並保留missing負例。即使已修正也不能因非lasting product error省略MD錯誤紀錄。下一新錯誤 E700。

## E695：filtered world 的眼位政策旗標沒有自動索敵候選來源（2026-10-10）

獨立來源檢查：gather_local 只在有 MobaMatch 時從 retained roster 加入無 Unit/Creep 的眼位。filtered reconstruction 沒有 MobaMatch，也不插 Unit/Creep，只有 direct/automatic gate，故 AutoEnabled 的已披露眼位仍無法進入候選集合。需有界、隨 disclosure/lifecycle 回收的 filtered 候選來源，保留直接／自動獨立政策及一般單位；不能加假 Unit/Creep 或全世界每幀掃描。需真實 filtered acquisition 正負例，不以 token codec 當索敵驗證。尚未接受；交 Grok 同 thread 修復。

## E694：正式眼位路由丟失輸入識別且延後綁定英雄來源（2026-10-10）

route_input 正式路徑固定 input_id=0、source_generation=None；drain 的成功 ID BTreeSet 僅處理 nonzero 且無界，正式路徑完全繞過它，None 會在結算重綁目前英雄。現有測試手動注入非零ID與generation，無法證明正式路由的來源退休／收據關聯。既有 authenticated lockstep buffer 已做輸入ID去重，不能宣稱所有網路重送目前會重複扣款；修復需沿用真實 admission identity／receipt、可信 actor life 與有界退休，並測 submit→route→drain／replacement／recharge後舊重送。尚未接受。

## E693：私人眼位狀態只有 bootstrap，持續更新與驗證 hash 未接入正式流程（2026-10-10）

實際來源搜尋：apply_continuing_owner_ward_projection 僅測試呼叫，hash_owner_ward_projection 沒有消費者；正式流程只於 join 複製 owner bytes，放眼／回充後沒有 continuing unicast，rebase 只清除而不恢復。bootstrap admission 僅檢查 ward version 非零，沒有完整 negotiation；malformed continuing decode 先返回，可能保留舊可用狀態。需正式 owner-addressed envelope／hash／順序與epoch admission、重連／rebase recovery，保持 team/observer 無私人資料。不能以未維護 omfx 的 exhaustive match 阻止正式 transport enum 擴充；依 AGENTS.md 不修改、不編譯 omfx，也不把其修復延期列入工作。尚未接受。

## E692：眼位傳輸初次獨立執行為 core6通過、世界4通過3失敗（2026-10-10）

ward-transport-initial-rust-v1 真正結束：codec5／route1 Success；base_content ward_transport_tests 7項執行、4Success／3Fail／0ignored，後兩 backend steps 未跑。原 stdout/stderr 與 failed summary 保留。失敗分別：schema1 BE 的 stale[0]=0 沒改任何位元卻期待拒絕；固定 damage10 未證明 hero 真正死亡卻期待 capabilityfalse；刪除眼位後直接 WaveB、漏掉該函式明確要求的 World::maintain，卻期待已退休baseline移除。需依真實生命週期修正 fixture、保留 schema rejection／death charge persistence／committed removal 斷言，不放寬正式規則。來源另有 E693–E695 實際接線缺口；交同 Grok thread 修復後再獨立驗證，不能將編譯預檢當功能成功。下一新錯誤 E696。

## E691：眼位傳輸測試在同一呼叫同時可變與不可變借用 world（2026-10-10）

ward-transport-worker-consumers-check-v1 前三項真正結束：client-runtime 11.70s、bridge 11.92s、omobab 16.19s，皆 Finished dev exit 0。scripts-all-targets 編譯失敗 E0502，七處 queue_ward(&mut world, ..., in_range(&world)) 同時借用 world。原 stderr 保留於 ward-transport-worker-consumers-check-v1。Worker 改為先算目標再入列，未改正式 drain／投影。fresh scripts check ward-transport-worker-scripts-check-v1 Finished dev 2.43s exit 0。測試未執行。下一錯誤 E692。

## E690：重複索引負例同時可變與不可變借用同一 Vec（2026-10-10）

ward-world-rust-v9 原失敗的 real filtered shop／reconnect／respawn integration 真正1Success，E688 等值檢查保留且已到終點。接著 core undo-wire lib test 編譯 E0502：duplicate_index[record..record+4].copy_from_slice(&duplicate_index[0..4]) 同時借用同一 Vec。Primary 改為 copy_within(0..4,record)，產生完全相同的重複 sub-index 負例，保留拒絕斷言與正式 codec；未改 production。原 log 保留，後續 checks 尚未跑；fresh rerun 待驗。下一錯誤 E691。

## E689：新增 reconnect 檢查與捕捉 replicas 的可變 closure 借用衝突（2026-10-10）

bootstrap hash repair 真正 terminal 後，ward-world-bootstrap-codegen-v1 實際 vendored protoc／17 generate／17 check 全成功；資料hash9fa與呈現hashc9dd不變。ward-world-rust-v8 第一項編譯失敗 E0502：原 advance closure 捕捉 &mut replicas，新增 reconnect loop 在 closure 後續還會用到時讀取 &replicas。Primary 將 replicas 改為每次呼叫明確傳入的可變 slice，保留原交易／逐tick hash 等值、reconnect 與死亡重生斷言；不刪 reconnect 檢查也不改正式 metadata/hash。原 v8 log 保留，fresh rerun 待驗。下一錯誤 E690。

## E688：受影響回歸發現 shop filtered snapshot 與 continuing replica 的 canonical hash 不同（2026-10-10）

ward-world-rust-v7 依序真正成功：眼位10／索敵26／summoner world18／visibility7／recall pause replay1／formal shop atomic replay1，共63項。第七步 single_lane_shop_filtered_settlement_is_private_atomic_and_survives_respawn 在 team1 tick2 canonical_team_hash 比較失敗，原 per-component 差異診斷無輸出；五 all-target checks 未執行。來源追蹤發現 hash 混入 owner_undo_hash_material，continuing frame 由 public_events 建立，fresh bootstrap constructor 預設空。尚需精確比對 metadata／bootstrap／projection 決定根因，不能僅據推測移除 hash、降低等值斷言或把既有回歸排除。無新對局；原63Success與1Fail log保留。下一錯誤 E689。

## E687：core 索敵測試使用另一模組的向量 alias（2026-10-10）

ward-world-rust-v6 第一項10眼位測試真正全過；第二項 core combat-acquisition 在 lib test 編譯失敗 E0433，新增 indexed ward policy fixture 兩處使用 SimVec2，但實際 tests 模組只 import Vec2（SimVec2 在別的 test_support 模組）。Primary 兩處改用該模組既有 Vec2，不改正式索敵或斷言；原 log 與10Pass保留。後續回歸／五checks尚未執行，不能把v6整體當PASS。fresh驗證待跑；下一錯誤 E688。

## E686：眼位測試沿用前次攻擊動畫鎖，移動 creep 後沒有提交新視野（2026-10-10）

ward-world-rust-v5 真正編譯並執行10項／8Success／2Fail／0ignored。owner-limit／跨截止暫停與死亡重生／directDisabled+autoEnabled／hidden friendly stale champion-only 已成功。剩餘舊 direct+autoDisabled 測試在 ordinary creep 攻擊後僅換 search index，hero_command_tick 正確保留 Windup/Backswing animation lock，不能當成又鎖定 ward；另一 realward 測試將普通 creep 搬到 detector 內只更新 nearby index，沒有 run_committed_visibility_wave_b，故 shared acquisition 仍視為未披露而回 None。Primary 在第二次新索敵前結束前次動畫 fixture；搬動 creep 後呼叫既有 project 並斷言該隊現在真正 sees creep，保留全部政策／普通 creep／no Unit/index 斷言與正式 animation/disclosure gates。原2Fail與8Success log 保留；fresh rerun 待驗，下一錯誤 E687。

## E685：真實眼位測試跑到4通過6失敗，fixture未準備命令佇列且使用負 generation handle（2026-10-10）

ward-world-rust-v4 第一項真正編譯成功，執行10測試／4Success／6Fail／0ignored。allied sight gain/loss、occlusion/detection、reject atomicity與replay digest通過。四項 direct/auto 政策測試於 ward_world_tests.rs724 的 HeroCommandQueue.get_mut().unwrap() 失敗，實際 match setup 沒有該 queue；owner-limit 與 recharge/respawn 兩項於 specs WorldExt::is_alive assert「Generation is dead」失敗，需區分原始正 generation Entity 與 entities().entity(dead_id) 產生的負 generation handle。原 stdout/stderr 保留；後續 consumer checks 未執行，不能接受世界範圍。需以實際 maintained 命令佇列建立方式與 generation-safe 查核修正，保留真實 shared tick／攻擊政策／死亡回收斷言；下一錯誤 E686。

## E684：眼位測試 fixture 的 closure 回傳、索引與 borrow 不符合實際 Rust API（2026-10-10）

ward-world-rust-v3 已通過前次 production hp 編譯位置，但 base_content lib test 真正出現18編譯錯誤：拒絕 helper FnMut 未指定 WardReject 回傳造成連帶型別錯誤、owner-limit loop 丟棄 index 卻引用它、BlockedRegions.clone 複製 Fetch wrapper、free function 同時可變借用 world 與取 hero、兩個 TAttack helper 持有臨時 storage 參照。Primary 僅補正 fixture 回傳型別／loop index、clone 真實 resource、先取得 identity／保留 storage；死亡負例精確保存與還原原 hp。保留全部拒絕原子性、owner-limit、政策與暫停斷言。新 note_ward_policy 只供 core unit test，加入 cfg(test) 消除該正式未使用入口；既有 template warnings 保留。原18錯誤 log 保留，尚無通過測試，fresh 驗證待執行。下一錯誤 E685。

## E683：實編譯揭露眼位驗證持有臨時 storage 的參照（2026-10-10）

ward-world-rust-v2 第一項仍在 omoba-core 編譯階段失敗：ward_world.rs347 的 let health = world.read_storage::<CProperty>().get(...) 產生 E0716，storage 在該敘述結束已釋放，下一行仍讀 health.hp。原 log 保留，沒有已執行測試。Primary 改為當下複製 Fixed64 hp 值再比較，保留 Dead 與零血量拒絕，不延長不必要的 storage borrow。尚需重新編譯／測試；下一錯誤 E684。

## E682：測試政策替換函式沒有編譯隔離（2026-10-10）

Grok review repair 真正 terminal 後獨立檢查：override_retained_ward_combat_policy 雖註解 Test authority only，仍無 cfg 且公開 re-export，正式 omoba-core 也會編入可覆寫目錄政策的入口。Primary 在 repair-source-v1 指紋真正退出後，加入 cfg(any(test, feature="test-support")) 函式與 export gate；test-support 預設關閉，只於 base_content dev-dependency 啟用，正式 dependencies 不變。保留真實放置眼位的政策組合測試，未以註解當隔離、未加假 production catalog。需 fresh 指紋與獨立測試／五 consumer checks 證實；尚未通過。下一錯誤 E683。

## E681：眼位使用會在 paused 正 delta 推進的 match elapsed，恢復後可能追補暫停時間（2026-10-10）

獨立來源審查：begin_moba_match_tick 在檢查 GamePause 前增加 state.elapsed；tick_match_wards 當次雖因 paused 返回，下一次使用已增加的絕對 elapsed 判定充能與到期，可能在恢復時追補暫停時間。原測試只比較 paused 當次且用一秒，沒有跨截止點再恢復，因此無法證明時鐘真正凍結。正式 SimulationDriver 會把 paused delta 設零，但公開 maintained tick seam 不應靠呼叫者偶然做到。需維持既有正式行為並以實際放置後跨到期／充能時間的暫停、恢復測試驗證；不得以忽略 paused 或調整 balance 避開。尚未修復。

## E680：允許自動索敵的眼位未進候選集合（2026-10-10）

獨立來源審查：spawn_ward 刻意不加入 Unit，避免 creep index 與獎勵路徑；gather_local 卻只讀 hero／creep／tower index，沒有眼位候選來源。因此 automatic_acquisition=Enabled 的真實眼位也無法被選中。現有測試把假眼位塞進 creep index，只證明已入集合後的 policy filter，未證明真實 spawn 的通用政策。需為實際眼位提供受限候選來源並保留 generation／距離／disclosure／友敵／champion-only 與直接攻擊政策獨立性，不得把眼位標成 Creep 或增加全實體每 frame 掃描。尚未修復。

## E679：眼位世界第一輪實編譯揭露 Option 與私有欄位錯用（2026-10-10）

ward-world-rust-v1 的第一項 base_content ward_world_tests 在編譯 omoba-core 時真正失敗，尚無任何通過測試。ward_world.rs:208 對 try_fetch 回傳 Option 呼叫 .ok() 造成 E0599；:303 跨模組存取 MobaHeroSlot 私有 lethal_pending 造成 E0616。原 stdout／stderr 與失敗 summary 保留。需使用實際 Option API 及既有或最小只讀存活／待死亡 accessor，不任意公開可寫內部狀態。後續步驟未執行，不能宣稱任何受影響 consumer 通過。下一錯誤 E682。

## E678：原生目錄驗證啟動仍耗時345秒，不能以移除新聞宣稱所有啟動延遲已解決（2026-10-10）

ward-catalog-build-v1普通nocache成功／UBT115.44秒／refreshfalse；native-v1三項真正Success，1筆FrameContract刻意不支援mode負例warning。native-engine.log實際LogLoad Engine Initialization Total time345.00秒；09:45:53載入CEF3、09:45:58啟用CEF GPU，09:48:57才完成Engine初始化。這是實際啟動延遲紀錄，不能僅因最後一筆log是CEF就判定根因，也沒有此次顯示新聞視窗的證據。保留先前已驗移除新聞啟動ticker的來源，不宣稱這個延遲已修復；後續獨立啟動診斷需分清WebBrowser模組／plugin授權網路／其他初始化，不能以停用授權、修改Engine或虛構新聞彈窗處理。當前三catalog測試與其範圍驗收不受此觀測冒充失敗；完整GUI啟動／效能驗收仍pending。下一錯誤E679。

## E677：runtime-lua-content 全測試揭露舊 fixture 缺少必填 catalog（2026-10-10）

ward-catalog-rust-v1 真實content-model29Success；template-ids啟用runtime-lua-content後43Success／10Fail／0ignored。七項根因是舊Lua fixture缺少既有必填moba_summoners，阻擋include順序／story／spawn selector／creep引用／TD layer／runtime hash目標驗證；三項reload因前一runtime測試panic污染共用Mutex，PoisonError連帶失敗。不能以default空catalog、略過feature或忽略舊測試解決。新trinket必填也必須在有效fixture補齊，讓各測試到達原本要驗的行為；不以吞poison掩蓋根因。原v1全部log保留。Grok source-only focused repair待獨立review／rerun，尚未通過。下一錯誤E678。

## E676：可達性測試錯要求正常文字必須超高，且縮放前的 close 幾何被當作捲動基準（2026-10-10）

普通 build-v6 真正成功18.34秒；targeted native-v4兩Fail、各1error、0warning。斷線Buy已實測enabled0，實際pointer拒絕與零action斷言通過。large首尾兩fixture均start/end/span/close為1。Responsive normal font/body11，文字112高、viewport207高，本來容納得下，start/end/span/close皆1，但required0使測試失敗。Standalone normal切換字級後未先Paint就取CloseBefore，之後rest Paint才完成新layout，start/end/span皆1但close0。修正只限fixture：先完成rest Paint才記錄close基準；可達性不要求一定超出視窗，保留兩端實測、超高時必須露出相反端溢出、scroll span、左右bounds、完整文字與實測字級，兩端都須成功。required診斷保留而不當產品前提；沒有縮字／截文／放寬bounds／改production。原失敗保留。下一錯誤E677。

## E675：來源指紋完成前修改測試檔，初次 manifest 不可作穩定基線（2026-10-10）

Primary 在 native-repair-source-v1 指紋程序尚未真正退出時補上兩份 fixture 的左右裁切斷言。沒有 build/native 同時執行，但該 manifest 可能混合修改前後內容，不能用於驗證後來源一致性。保留 v1 作失誤紀錄；重新產生 v2，真正退出後才開始 build，期間禁止編輯被掃描的 source。先前 E674 修正不因本次流程錯誤而宣稱通過。另一次 review/error MD 合併 patch 使用未觀察的 `#` 整行 context，工具原子拒絕，無任何檔案寫入；已讀實際 heading 後重送精確 patch，避免猜測 context。下一錯誤 E676。

## E674：窄視窗長配方行高於捲動區，購買按鈕讀到預繪製啟用快取（2026-10-10）

shop-recipe-build-v5普通成功23.77秒。shop-recipe-native-v3只跑Responsive與ShopRecipePresentation，兩項Fail／0warning。Responsive真點擊OmShopRecipe.90已down1／up1／selection90。IntoView把normal材料行底對齊scroll底：文字Y880.75高150、scroll Y888.5高142.25、同一widget、offset594.64，整塊contained不可能。large同樣用整塊contained，兩份測試都失敗。Buy在SetEconomyState重建後、Paint前讀IsEnabled：Slate綁定屬性預設快取true（INITIALIZATION_ON_BIND為0），CanSubmit已是0；該斷言之後的點擊沒有推進sequence。測試改為TopOrLeft／BottomOrRight實測行首與行尾、全文、實測字級與關閉鈕不動，並在既有prepass之後檢查停用與pointer route。正式UI不變。E672／E673保留。尚未重跑。下一錯誤E675。

## E673：配方 fixture 未更新 Slate 屬性且點擊落在 scroll 裁切區外（2026-10-10）

ordinary build-v4成功44.55秒；targeted native-v2兩項Fail／0warning，原斷言未改。真診斷Responsive按鈕Y438..473、scroll252..449，點擊中心455.5不在裁切內，route有效但無目標。獨立fixture長文字same1排除舊widget，normal高度150大於scroll142.25，Buy存在卻enabled1／CanSubmit0；其Paint只prepass＋ForceRedraw，漏掉既有Responsive／Undo fixture使用的Slate.Tick，bound font/enabled仍未更新。Primary只將獨立Paint補為正常兩次Slate tick／prepass繪製循環，兩個點擊helper先繪製取得當前layout再以Center捲入後照常真hit-route/down/up；不sleep、不直接呼叫handler、不縮字體、不刪全文contained／禁止購買斷言，正式UI不變。原兩輪失敗／八項成功及六張800x600／400x300當次original已目視保留。fresh build/native後段尚待通過，下一錯誤E674。

## E672：配方 native 真實點擊與窄版幾何檢查未通過（2026-10-10）

shop-recipe-build-v3普通nocache真正成功17.13秒；native-v1 process0但workflow失敗，10項中8Success／2Fail／0incomplete，FrameContract保留1筆刻意不支援mode負例警告。Responsive在Narrow production shop opens the long recipe失敗；ShopRecipePresentation在normal長材料全文contained與Disconnected buy stays disabled兩項失敗，其他早期滑鼠ingredient/back等檢查已執行。尚無證據判定正式UI或fixture根因；Primary只加真button/scroll存在、rect／scrolloffset／hit-path／down-up／selection與current長文字identity／buy狀態診斷，保留所有原斷言。八項成功與原失敗／六張當次原圖保留，fresh targeted兩項需定位再修，不宣稱recipe完成。下一錯誤E673。

## E671：新增 owned 商店測試混用 shared pointer 與焦點 API（2026-10-10）

shop-recipe-build-v2普通nocache真正OtherCompilationError40.14秒，先前E669/E670相關decoder／reader／ShopRecipeTests編譯已過。OmOwnedViewportIpcTests.cpp8278將TSharedPtr傳StaticCastSharedRef而C2672；8295在SWidget呼叫不存在IsFocusable而C2039。Primary僅改StaticCastSharedPtr與實際SWidget.h929的SupportsKeyboardFocus()，保留型別／有效性guard、真按鈕不可焦點斷言與真正TryClickTag事件，不改正式控制或移除檢查。原v2全部log保留，fresh v3需重建/native/owned。下一錯誤E672。

## E670：配方測試將 FString 傳給 const TCHAR*（2026-10-10）

shop-recipe-build-v1真正OtherCompilationError，OmShopRecipeTests.cpp182／199的Projected lambda第二參數為const TCHAR*，兩處卻傳LongName FString，MSVC C2664。原UBT log保留；修正呼叫端字串指標或一致型別，不修改原長名稱／窄視窗檢查。尚未重建通過。下一錯誤E671。

## E669：錯誤前置宣告與 cbindgen 匿名 typedef 衝突（2026-10-10）

shop-recipe-build-v1普通NoEngineChanges／NoUBA／nocache真正exit1，UBT OtherCompilationError／1075.09秒，未進native。新OmShopCatalogDecode.h與OmShopCatalogRead.h使用struct OmFrameHeader／OmShopCatalogItem前置宣告，但維護om_bridge.h是匿名struct typedef；C2371引發兩個reader、actor與測試大量C2027。ABI23實際cbindgen與獨立C-layout已通過，不能改產生標頭或以Engine refresh掩蓋consumer錯誤。修正新增header直接引用正式om_bridge.h並移除錯誤宣告，保留原ABI／所有資料驗證。原build/UBT/summary完整保留，fresh普通build與native仍待執行。

## E668：共用合成費驗證晚於材料消耗檢查而改變錯誤語意（2026-10-10）

shop-recipe-rust-v2真正36項、35成功／1失敗：large_prices_use_exact_integer_refunds_and_recipe_overflow_is_atomic預期InvalidCatalog卻收到MissingComponent。新迴圈先找第二份材料，再驗總合成費，讓壞價格被缺材料遮蔽。Primary將buy_item分成完整catalog／checked_combine_price驗證，再於clone消耗材料，最後原子提交；未放寬測試或修改預期。原v2日誌保留。fresh v3重跑所有affected Rust與checks，不能沿用失敗batch為通過。下一錯誤E669。

## E667：新增配方測試誤移除既有滿背包回歸的 test 標記（2026-10-10）

Primary實際source review發現full_inventory_can_upgrade_but_cannot_buy_seventh_item仍有完整函式卻失去#[test]，會靜默漏跑。只恢復標記與原有斷言，未刪除新配方測試；fresh core-shop必須包含該具名成功列才可接受。原worker來源809指紋與差異保留，不能以少跑數量宣稱回歸成功。

## E666：配方圖遞迴借用未明確連結 stack 元素生命週期（2026-10-10）

shop-recipe-rust-v1真正core-shop exit101，未執行測試；編譯器於shop.rs1016 stack.push(id)拒絕獨立&str lifetime經可變Vec不變性寫入。Primary只為walk_recipe的catalog keys、RecipeSource、id與stack元素加入共同命名生命週期，不使用unsafe、clone全字串或繞過圖驗證。原stdout/stderr與失敗summary保留；後續fresh v2重跑才算證據。既有template-ids dead_code與authoritative_mode unused import警告仍保留。下一錯誤E668。

## E665：純 HUD fixture 要求停用的設定按鈕進入命中路徑（2026-10-10）

ordinary responsive-shop-build-v4成功，targeted native-v2仍Fail／0warning。真診斷window／closeInside／searchInside／三標籤／fonts11／closeHit全通過，只有settingsHit0。原圖shop-roomy-normal.png逐張original檢視：設定按鈕灰色停用。fixture只建立standalone command bar，沒有UOmControlSettingsComponent；正式按鈕IsEnabled委派因此false，Slate LocateWindowUnderMouse(false)依法排除停用項。Primary只修測試後置條件：按鈕enabled要求真path含該widget，disabled要求真path排除；明確輸出enabled狀態，不啟用假按鈕、不新增假owner、不放寬shop-close優先序。真正可用settings在fresh owned suite另驗。正式UI無變動；原兩輪Responsive失敗與五項成功保留。下一錯誤E666。

## E664：Primary 診斷日誌將 font Size 誤用整數格式（2026-10-10）

responsive-shop-build-v3真正OtherCompilationError／exit1，新增診斷FString::Printf的三個Font.Size實際為float，Primary用了%d，UE編譯期format checker以C7595拒絕。只改三處格式為%.1f，保留原值與斷言，未改正式UI或轉型掩蓋資料。原build log保留；後續診斷依本機型別而非記憶撰寫。下一錯誤E665。

## E663：實際商店 viewport 寬版綜合幾何斷言失敗（2026-10-10）

responsive-shop-build-v2普通no-cache真正成功；native-v1 process0但workflow1，六項中五Success／ResponsiveHudLayout一Fail（兩個綜合error）／0incomplete／保留E653一警告。Undo全後段、ShopPanel、NativeShopInput、ItemIcon、ControlRuntimePersistence均成功。失敗log roomy text1 allotted800x600／painted620x480／preferred620x480，viewport curve0.555／layout1.75，尚無證據定位綜合條件的哪一項。Primary只加各實際rect／標籤尺寸／font／hit-path條件診斷，並讓第一張original圖於斷言前保存，不刪或放寬檢查。當次兩張Undo原圖已original檢視，關閉與分類改善但完整responsive尚未接受。原失敗保留，待fresh targeted定位。下一錯誤E664。

## E662：商店幾何測試誤用 ContainsWidget 參數（2026-10-10）

responsive-shop-build-v1普通no-cache真正exit1／OtherCompilationError。UBT列出OmResponsiveHudTests.cpp889、890、899及OmShopUndoUiTests.cpp278、279共五個C2664：本機FWidgetPath::ContainsWidget要求const SWidget*，新增測試傳TSharedRef。Primary僅將五處改為既有TSharedPtr的Get()，保留真hit-path斷言、不直接呼叫按鈕或刪檢查。正式shop來源已編譯，尚未runtime驗證。原build／UBT／summary保留；下一錯誤E663。

## E661：焦點測試將未變更誤判為恢復失敗（2026-10-10）

stop-outside-recovery-build-v6普通no-cache實際成功。native-v2兩項中ActionOwnerPressRepeatCoalesce成功；UninstrumentedStopHoldRepeat通過primary/secondary長按、flush、rearm、Pause與foreign-window路由後，停在Owned viewport focus returns，0警告。UE SlateApplication.cpp的SetUserFocus在目標已是OldFocusedWidget時明確回傳false；關閉外窗會自動恢復原焦點，因此回傳值不是焦點後置條件。Primary只修三處恢復斷言：要求實際GetUserFocusedWidget等於原owned viewport，保留真正路由與後續foreign-user/text/modal檢查，無sleep或降低預期。失敗保留，尚未證明後段。下一錯誤E662。

## E660：新增輸入測試使用非同步日誌 observer 卻立即斷言（2026-10-10）

普通no-cache build-v5實際成功；native-v1真process0／workflow1、8項中6Success／2Fail／0incomplete，保留ControlRuntimePersistence一筆E653警告。兩項新增測試皆handlers=0早退，但原始engine log明確有各自同幀OM_HERO_INPUT_REJECT label=Stop；uninstrumented案例另有reason=raw_os_repeat，真handler與原始repeat保護已發生。根因：FPressTally沿用FOutputDevice預設buffered分類，尚未收到專用log thread派送就斷言。依UE OutputDeviceRedirector.cpp429／440的實際分類規則，Primary僅改兩個測試observer為unbuffered（CanBeUsedOnMultipleThreads／AnyThread），先過不可變建立執行緒與原始LogOmPlayerActionOwner category guard後才接觸可變tally，避免其他執行緒與Automation echo誤計。沒有sleep、放寬數量、直接呼叫handler或修改正式來源。原native兩個失敗與其他六個成功保留；新兩項需fresh重跑且後段尚未證實。下一錯誤E661。

## E659：實際輸入裝置 mapper 缺少直接 ApplicationCore 連結依賴（2026-10-10）

refresh build-v3實際exit1／OtherCompilationError，UBT19.21秒：C++編譯已過，OmEditor DLL連結LNK2019 IPlatformInputDeviceMapper::Get、LNK1120。本機GenericPlatformInputDeviceMapper.h明確以APPLICATIONCORE_API匯出；OmEditor原本只有傳遞include、未宣告直接ApplicationCore私有依賴。Primary在OmEditor.Build.cs加入該直接私有依賴，保留實際裝置／user映射與foreign-user測試，不硬編device或刪斷言。只修必要模組邊界，原失敗保留；先fresh來源指紋，再完成refresh及普通no-cache建置。下一錯誤E660。

## E658：新增輸入測試的 TObjectPtr 與 Slate user API 使用錯誤（2026-10-10）

refresh build-v2實際exit1／OtherCompilationError，UBT108.83秒：OmActionOwnerTests.cpp263、515的TestNotNull模板不接受TObjectPtr<UPlayerInput>；757呼叫protected GetOrCreateUser。Primary依本機UE5.8 API僅修測試：兩處傳PlayerInput.Get()；以公開FindOrCreateVirtualUser取得RAII handle與實際GetUserIndex，確認與owner不同，handle管理user生命週期。保留真Slate foreign-user焦點／傳遞斷言，不刪案例或調整預期，不碰正式輸入來源。原編譯失敗完整保留。接續fresh來源指紋、完成refresh，然後fresh普通no-cache與真native。下一錯誤E659。

## E657：新增控制器類別後普通建置要求 Engine metadata 更新（2026-10-10）

`stop-outside-recovery-build-v1` 真正exit1，UBT20.28秒，Result FailedDueToEngineChange；普通no-cache／NoEngineChanges拒絕修改列出的Engine .modules等輸出，非已證明的C++編譯錯誤。原build.log／UBT-Log／summary保留。依既有流程只在此真實結果後使用正式--refresh-engine建置，成功後再跑fresh普通no-cache，不能手動改Engine source／BuildId或把refresh當普通建置通過。尚未執行新的native／owned，不記成功。下一錯誤E658。

## E656：原始按鍵追蹤只接兩種控制器，未覆蓋通用 owner（2026-10-10）

Primary獨立審查v2交付：ObserveCallerPressed與batch入口僅由AOmPlayerController及新增特殊override的AOmAutomationInputController呼叫。EnsureForController接受一般APlayerController，但未改寫控制器沒有origin紀錄，合法同批Pressed＋Repeat仍可能拒絕；修改既有「plain controller」測試類別不能證明通用相容。此為來源審查缺口，尚未建置／native實跑，不記成實跑失敗。補修要求在現有owner Slate processor對精確Stop／Hold重複事件使用local user／owned viewport／非文字模態／active chord guards，讓任意控制器實際OS repeat在引擎合成按下前停止；genuine press與其他輸入沿原路。新增真正未instrument的控制器與實際Slate／owned回歸，不減少290／119／101既有標準。下一錯誤E657。

原因：`ObserveCallerPressed`、`BeginCallerPressBatch`、`FinishCallerPressBatch` 只由 `AOmPlayerController` 與 `AOmAutomationInputController` 呼叫。`EnsureForController` 仍接受任意 `APlayerController`。未改寫的控制器沒有呼叫端按下紀錄，同一批合法 `IE_Pressed`＋`IE_Repeat` 會被 `IsHeldOsRepeat` 拒絕。把既有測試控制器改成有這些 override 會把這個缺口藏起來。

修正：上述兩個控制器的呼叫端紀錄保留，給直接或程式呼叫的 `InputKey` 使用。`FOmAttackMoveAimProcessor::HandleKeyDownEvent` 改呼叫 `ConsumeOwnedStopHoldOsRepeat`。只有同時成立才吃掉事件：OS repeat、鍵與修飾鍵等於目前 profile（沒有 runtime profile 時用 `MakeDefault`）的主要或次要 Stop／Hold、`Event.GetUserIndex()` 是這個控制器的本機 Slate user、`HasOwnedViewportFocus`、且 `IsKeyboardBlocked(false)` 為假。文字、模態、`IgnoreInput`、沒有 user 都不吃。真正的非 repeat 按下仍進同一個 `InputComponent` 綁定。其他鍵、修飾鍵、user、視窗與 UI 不改，也不從 processor 直接送指令。吃掉的 repeat 不會進 `UPlayerInput`，因此不會再合成按下。該拒絕記 `OM_HERO_CONTROL_IGNORED reason=raw_os_repeat`，與事後 fallback `reason=os_repeat_after_flush` 分開。`KeyRepeat` 不 `SetUserFocus` 與 route 診斷保留。

預防：新增 `AOmUninstrumentedInputController`，直接繼承 `APlayerController`，只有 `ProcessOwnedInput` 與 `CountOwnedInputComponents`，沒有 caller-press override。owned IPC 只把控制器型別換成它，290／119／101 案例順序與 wire 契約不變。`Om.Runtime.ActionOwnerPressRepeatCoalesce` 仍用有紀錄的 `AOmAutomationInputController`。新測試 `Om.Runtime.UninstrumentedStopHoldRepeat` 走真實 `SViewport`／`UGameViewportClient`／Slate user 與 `InputComponent`：按下加 repeat 一次；`FlushPressedKeys` 後的 raw repeat 零次且 Pressed／Repeat 計數皆空；放開再按與同批 release＋press＋repeat 重新武裝；Ctrl+J／Alt+K 的精確修飾鍵成立，錯誤修飾鍵、外窗、外 user、可編輯文字與 modal 的 repeat 不被這條 guard 吃掉。此來源尚未建置或重跑。原 v1／v2 report 與 owned 失敗 log 不覆蓋。下一錯誤仍為 E657。

## E655：以同幀 Pressed＋Repeat 推斷合成按下會丟失合法按鍵（2026-10-10）

Primary獨立審查E654來源修正發現：IsHeldOsRepeat只看兩個EventCounts非空，實際正常IE_Pressed後接IE_Repeat、同一ProcessInputStack前累積也符合，會拒絕合法第一筆S／H。UE PlayerInput.cpp421的flush後合成與普通switch皆可形成此形狀，不能只靠counts推斷來源。此為來源審查問題，尚未執行新native／build，不宣稱實跑失敗。要求依真原始事件／owner生命週期區分，補真input binding批次press／repeat及flush／re-arm回歸，不以bDownPrevious單一條件掩蓋release／repress同幀問題。原v1修正來源、report及原owned失敗完整保留。下一錯誤E656。

原因：同一處理幀只要 `EventCounts[IE_Pressed]` 與 `EventCounts[IE_Repeat]` 都非空就當成 flush 合成按下。呼叫端先 `IE_Pressed` 再 `IE_Repeat`、尚未 `ProcessInputStack` 時也是這個形狀，合法的第一筆 Stop／Hold 被丟掉。`bDownPrevious` 不能補這個洞：按住時同一批 `IE_Released`＋`IE_Pressed`＋`IE_Repeat` 在 delegate 當下仍是 true，和按住時 `FlushPressedKeys` 之後只送 `IE_Repeat` 相同。合成按下只發生在 `UPlayerInput::InputKey` 內部（`bWasJustFlushed` 且 Pressed 累加器為空），不會再進一次 Slate，也不會再呼叫 `APlayerController::InputKey`。`FOmAttackMoveAimProcessor` 看不到這次插入，也看不到直接的 `InputKey`；在那裡吃掉按鍵會影響所有視窗。

修正：`AOmPlayerController` 與 `AOmAutomationInputController` 的 `InputKey` 只在呼叫端事件是非類比 `IE_Pressed` 時呼叫 `ObserveCallerPressed`。`PreProcessInput` 打開本批，`PostProcessInput` 在 delegate 之後清掉本批，下一筆真正按下才能再武裝。`IsHeldOsRepeat` 僅在 Pressed 與 Repeat 都非空、且本批沒有呼叫端按下時為真。`BindImmediateHero` 仍只綁 `IE_Pressed`，拒絕時不佔鍵。不改 Engine、ini、cvar，也不做 F11 特例。`KeyRepeat` 不 `SetUserFocus` 與 route 診斷保留。

預防：新 native 測試 `Om.Runtime.ActionOwnerPressRepeatCoalesce`。正式 `APlayerController::InputKey` 進入真實 `UPlayerInput`，再 `ProcessOwnedInput` 跑正式 `InputComponent` 綁定。S、H、次要 Stop（J）、次要 Hold（K）各驗證：單獨按下一次；Pressed＋Repeat 一次；按住時 Release＋Pressed＋Repeat 一次；`FlushPressedKeys` 後只送 Repeat 零次且 Pause sentinel 仍跑；其後 Release＋Pressed＋Repeat 重新武裝。缺連線時 `bAccepted` 為 false、`InputId` 為 0，不記 authority ACK。此來源尚未建置或重跑。原 v1 report 與 owned 失敗 log 不覆蓋。下一錯誤 E656。

## E654：owned 操作回歸在其他視窗焦點恢復時多送 Stop（2026-10-10）

`shop-undo-ui-viewport-v1` 實際 terminal exit1：`stop-outside-recover emitted an extra wire intent`。先前 stop-text-recover／hold-modal-recover 各兩筆成功；control history 停在 sequence76，不能記為完整290案例成功。原 native／workflow／wire logs 保留。cleanup 實際 shutdown_file、listener_released=true、authority=false。先定位額外 wire 的時序與真正 Slate 焦點／按鍵生命週期，再修正通用原因；不減少預期案例、不放寬指令數、不用 sleep 隱藏。完整回歸與商店 responsive 尚未驗收。下一錯誤 E655。

來源修正（尚未建置或重跑，原失敗證據不覆蓋）：`shop-undo-ui-viewport-v1.ue.log` 在 frame 564 執行 `Cmd: LevelEditor.ToggleImmersive`，frame 565 `OM_HERO_CONTROL action=Stop input=108`，frame 568 再一筆 `input=109`，frame 570 才報 extra wire。中間沒有 aux 視窗銷毀。對照已通過的 stop-text-recover：input 104（frame 186）之後先 `Window '' being destroyed`（frame 278），第二筆 input 105 在 frame 279。hold-modal-recover 的 Hold input 106／107 相隔約一輪 quiet。更早兩次 F11 也各觸發一次 ToggleImmersive（frame 973→Stop 75、frame 449→Stop 102），但那兩次之後沒有「進入 immersive 仍按住再送 repeat」。

原因：`Engine/Config/BaseInput.ini` 的 `DebugExecBindings` 把 F11 綁到 `LevelEditor.ToggleImmersive`。`UPlayerInput::InputKey` 在非 shipping 執行該命令。進入 immersive 會讓 viewport `LostFocus`，`bShouldFlushPressedKeysOnViewportFocusLost` 呼叫 `FlushPressedKeys`。引擎接著把 flush 之後的第一個 `IE_Repeat` 合成一筆 `IE_Pressed`（`PlayerInput.cpp` 的 `bAutoReconcilePressedEventsOnFirstRepeat`）。Stop／Hold 只綁 `IE_Pressed`，所以這筆被當成新的一次按下。fixture 的 `KeyRepeat` 還在送出 repeat 之前 `SetUserFocus` 到 owned viewport，`FSlateApplication::SetUserFocus` 會把 `ActiveTopLevelWindow` 設回遊戲視窗，等於把外窗的 repeat 拉回遊戲。這是生產路徑會在 viewport 失焦後把 OS repeat 再送一筆 Stop，同時 fixture 用錯誤的焦點力學重現它。不是少算案例。

修正：`BindImmediateHero` 在同一處理幀同時有 `IE_Pressed` 與 `IE_Repeat` 時拒絕 Stop／Hold，記 `OM_HERO_CONTROL_IGNORED reason=os_repeat_after_flush`，不佔住按鍵；真正的下一筆只有 `IE_Pressed` 的按下仍送出。`KeyRepeat` 不再呼叫 `SetUserFocus`，repeat 留在當下擁有按鍵的視窗。`KeyDown` 仍先把新按下送到 owned viewport。失敗字串附上 repeat、鍵盤焦點視窗與 active window（game／aux／foreign）前後角色，鍵盤焦點與視窗激活分開記錄。S／H 與次要鍵仍是按下一次、OS repeat 忽略、放開後跨 modal／文字／其他視窗重新武裝，此案例仍應正好兩筆。

預防：新增 press-once 英雄指令不得只靠 `IE_Pressed` 綁定；flush 後的 repeat 會再合成 pressed。自動化 repeat 不得為了把事件打進遊戲而激活 owned viewport。不要再用會觸發 editor debug exec 的按鍵假裝成單純外窗，除非生產路徑明確忽略該合成按下。E653 scroll descendant 與 responsive shop 仍未處理。此修正尚未由 Codex 建置或重跑 owned IPC，不能記 290 通過，也不能記 authority ACK。

## E653：設定持久化 native 成功但保留 scroll descendant 警告（2026-10-10）

fresh ordinary build-v7成功；native-v3真正5項Success／0error／0incomplete，UndoProjectionAndPin完整成功且0warning，E652狀態文字已過。ControlRuntimePersistence 出現1筆 LogSlate: Unable to scroll to descendant as it's not a child of the scrollbox。此為實際warning，尚未定位root cause，不能標成預期或零警告，也不能憑warning宣稱操作失敗；設定持久化斷言本輪全部成功。完整settings／responsive驗收保留此診斷待查。其他未變正式來源的Rust168／4checks明示沿用，native-v1其餘未受影響6項成功亦明示沿用；新owned regression執行中。下一錯誤E654。

## E652：改鍵退休已接受撤銷請求後狀態文字未即時更新（2026-10-10）

fresh native-v2 真process0／workflow1，Undo單項Fail／1error／0warning；E650焦點及E651實際重新點擊已過，後段收據、真正owned鍵盤與control-only結果分支均執行完畢。唯一失敗是已接受id41的profile retirement：pending=false／watch=true／id41／pin9斷言已過，但狀態文字仍為「撤銷已送出，等待結算，尚未確認前不會視為已退款」，預期「連線狀態未確認，不會視為已退款」。正式 RefreshClusterScales 的revision變動只RetireShopUndoCapture，未更新chrome，需等下一economy snapshot才同步。

Primary在同一正式revision變動分支，退休後立即ApplyShopUndoChrome，僅改鍵事件時更新，不增加每frame UI操作；保留原id／pin／watch與收據規則，不改測試期待。原native-v2失敗保留；接續fresh fingerprint、普通no-cache build與受影響native驗證。下一錯誤E653。

## E650–E651：Undo 原生實跑兩個 fixture 斷言失敗（2026-10-10）

修復來源交付：Grok4.7非fast run-mv1tyoaa-9fm8o8 真terminal14m32s／42calls／$1.13828736，source-v3完整798指紋僅 OmShopUndoUiTests.cpp 改變。以真keyboard user、內層focus路徑、輸入q及Ctrl+Z恢復驗證焦點；點擊仍經pointer route，補真視窗／tick／hover／capture／release與分階段失敗資訊，不直接設pending或以提交函式替代。Primary獨立核對本機Engine API、普通nocache build-v6真正成功；native-v2單獨重跑Undo中，尚未宣稱通過。未變正式來源的Rust168／4checks與其他9個native成功明示沿用。

shop-undo-ui-native-v1 真正 process exit0，但 workflow exit1；實際10 tests，9Success（FrameContract刻意拒絕非法mode一筆warning）、ShopUndoProjectionAndPin Fail／2errors／0notRun。E650：266 比較 GetKeyboardFocusedWidget()==外層 SEditableTextBox，實際焦點可能是內層 editable child；必須驗證真焦點路徑與文字輸入，不把外層指標相等當焦點。E651：344 Rearm before accepted transport id 的真正 ClickUndo 失敗，早退使後段接受ID／收據與真正 owned keyboard fixture 尚未執行。需診斷 eligibility、paint geometry／hit path／焦點及capture，走正式生命週期重新準備，不能直接設 pending／跳過click或削弱斷言。

兩張當次PNG已逐張 original 檢視，屬失敗診斷，未接受。800是邏輯尺寸，首張實際受1.75DPI影響，不可報為800實體pixel；400畫面中Undo按鈕與reason可見，但商店close與設定鈕重疊、categories省略且下半商店裁切，不能當完整商店／HUD可讀性通過。完整responsive shop問題保留至4.7／最終畫面驗收，不用這兩張圖提前勾完父項。來源未變的9個成功項明示可沿用，新的Undo fixture須真正重跑。下一錯誤 E652。

## E649：Slate key event 的 bool 結果誤用 FReply API（2026-10-10）

refresh build-v3 真正 exit1，UBT9.21秒，唯一 root compile error 是 OmShopUndoUiTests.cpp687 C2228：ProcessKeyDownEvent(TextUndo) 回傳 bool，不能再呼叫 IsEventHandled；C2661 為連帶錯誤。Primary 改為直接 TestTrue(bool)，保留 Ctrl+Z 真文字焦點事件斷言。其他 RoutePointerDown／Up 的 FReply.IsEventHandled 不改。原 v3 log 保留；先 fresh refresh-v4 完成尚未成功的引擎初始化，再 fresh ordinary-v5。下一錯誤 E650。

## E646–E648：Undo 原生 fixture 真正 C++ 編譯錯誤（2026-10-10）

後續真正 terminal：refresh-v2 exit1，UBT OtherCompilationError／831.10秒。上述三類窄 fixture 修正已套用；refresh 流程未成功完成，故先用 fresh refresh-v3完成其餘連結／metadata，再以 fresh ordinary-v4驗證，不把失敗 refresh 當已接受。Rust／協定／標頭來源不變，168 tests／4checks 明示沿用，native 必須真跑。

refresh build-v2 48/55 的 OmShopUndoUiTests.cpp 已真正編譯 exit2；此紀錄時整體建置仍在等待其他工作結束，不能宣稱整體 terminal。E646：275／276 呼叫 private IsImplementedChordAction／HandleShopUndoPressed（C2248）；這是首輪遺留的未 owned helper fixture，後段已實作真 viewport／PlayerInput 路徑。移除遺留直接 private 呼叫及其無效斷言，不開放 private、不新增 friend，保留真 owned 正向／拒絕測試。

E647：599 的 FInputChord UndoChord 與97既有 pointer 同一 function scope 重名（C2373；600的 C2661 是連帶錯誤）。將新 chord 變數具體改名 OwnedUndoChord。E648：687 用六參數 FCharacterEvent（C2440），實際本機 Engine SlateCore Input/Events.h619 只有 character／modifiers／user／repeat 四參數；既有 production settings fixture 也用四參數。改為真 API，保持真正文字輸入焦點測試。待整體 consumer 真正結束後才修改來源與 fresh 普通 build；原 refresh log 保留。下一錯誤 E649。

## E645：普通 no-cache 建置真正回報 EngineChange（2026-10-10）

shop-undo-ui-build-v1 真正 exit1；base_content／Rust bridge／cbindgen stage 已完成，UHT 成功寫13檔、201.2697116秒；UBT326.74秒後 FailedDueToEngineChange，指出 UnrealEditor-NetCore 及模組/version 產物需重建。原 build.log／UBT-Log.txt／summary 保留；BpGeneratorUltimate 缺 plugin dependency 等既有警告亦保留，不宣稱零警告。尚未 C++ 或 UI 通過。

依既有恢復流程啟動 fresh build-v2 refresh no-cache（正式 --refresh-engine，max12），成功後仍須 fresh ordinary no-cache 驗證；禁止手改 BuildId、modules、version、Engine source 或開關繞過檢查。下一錯誤 E646。

## E644：all-targets 發現三個舊整合 fixture 缺新增命令欄位（2026-10-10）

fresh shop-undo-ui-rust-v3：核心11、client57、bridge100 真正通過；core／client all-targets 已成功，bridge all-targets 實際 E0063，runtime_driver_smoke.rs222／315／485 的完整 OmInputCommand literal 漏填 ABI22 undo_sequence。backend check 尚未執行。Primary 僅為這三個既有非撤銷命令補 undo_sequence:0，不修改正式命令驗證；原 compile stderr 保留。此為編譯整合 fixture，未額外執行 smoke 對局，不新增權威遊戲或 ACK。下一錯誤 E645。

## E642–E643：橋接層實跑發現兩項 fixture 契約錯誤（2026-10-10）

fresh shop-undo-ui-rust-v2：core11、client57 真正通過；bridge 實際98pass／2fail／1ignored，後四項 all-target checks 未執行。E642：summoner_abi_layout_matches_the_handwritten_header 預期 undo_sequence=summoner_slot+8，但真正 repr(C) offset 為88，summoner_slot84，加 sizeof(u32) 即已對齊，不再加 padding。Primary 改為真欄位大小關係，C++ smoke 另固定 offset88／sizeof96，沒有改 ABI 或生成標頭。

E643：mapped_undo_receipts_keep_authority_id_and_renderer_request 第一段不同 ID 映射已通過，stale epoch 段的 Buy/Sell receipt 卻填 action18；正式40byte authority codec 正確拒絕非法 action，導致 unrelated receipt 數量0。將該 fixture 改為已有合法 action17，仍要求 stale Undo 整批拒絕且合法 Buy/Sell 保留；不放寬正式 codec。原兩失敗 stdout／stderr 完整保留。下一錯誤 E644，尚未 UE 驗證。

## E641：Undo UI Rust 測試 closure 缺少可變 binding（2026-10-10）

fresh shop-undo-ui-rust-v1 真正 exit1：core renderer 7、shop transport 3、maintained join 1 已通過；client-input 在測試編譯 E0596，input_bridge.rs 的 reject_other 與 closed closure 捕捉可變 bridge，binding 卻未宣告 mut。此組實際未執行測試，後續組也未跑。Primary 僅補兩個 mut，保留斷言及正式行為；原 stdout、stderr、summary、errors.md 保留，另開 fresh v2 驗證，不覆蓋失敗紀錄。

## E640：只有控制結果的 frame 無法派送 Undo 等待／拒絕（2026-10-10）

來源審查確認 WorldBridge 把 ShopUndo critical result 派送放在 presentation_snapshot 分支內；正式 Busy 會產生沒有完整 snapshot 的控制 frame，因此 HUD 收不到等待結果。Primary 將派送移到可選 economy snapshot 之後的獨立路徑，並檢查真正 local player、command kind、有效 request id。原生 owned fixture 補透過 ProcessSyntheticFrameForAutomation 的 control-only Busy、外人拒絕忽略及本人明確拒絕釋放測試；控制接受與 Busy 不視為權威退款或 ACK。尚未原生實跑，不能宣稱通過。

工作流修正：先以 rg --files 發現實際檔名，避免猜測不存在的測試檔；對大型 dirty workspace 使用限定路徑與有界摘要，避免完整 status／單行 JSON 淹沒結果。新增測試不得呼叫未宣告的 GetEconomyState；改用既有公開狀態驗證退款文字不成立。下一錯誤 E642。

## E639：真正C++標頭編譯仍斷言ABI21的舊frame尾端（2026-10-10）

fresh header-v2真正cbindgen／stage byte identity已通過ABI22。`verify-summoner-c-header.lua shop-undo-ui-c-layout-v1`實際C++compile失敗，compile.log保留C2338在smoke.cpp12：presentation view epoch closes the frame header。新ABI22明確在epoch後新增undo receipt count與pointer，Rust來源相應tail斷言已改，但C++smoke只改版本號，仍驗舊尾端。Primary把該斷言換為嚴格的新count／padding／pointer／struct尾端關係，另加InputShopUndo18、Buy/Sell40bytes維持、UndoIdentity56bytes及request40／epoch48固定offset，沒有放寬對齊檢查或手改生成標頭。fresh c-layout-v2重跑；原v1summary／compile.log保留。尚未UE或native驗收。下一E640。

## E638：真正cbindgen成功後標頭stage helper拒絕覆寫（2026-10-10）

實際 shop-undo-ui-codegen-v1已成功（vendored protobuf、17檔generate/check、content c9dda913c42a5118）。接著 `regenerate-current-bridge-header.lua shop-undo-ui-header-v1` 真exit1，path.lua56拒絕覆寫既有om_bridge.h。原cbindgen已成功且新標頭/stdout/stderr均保留；helper第三參數false代表禁止overwrite，第四參數true僅代表binary。尚未stage新標頭，不能宣稱ABI22已驗證。Primary只將這個經明確授權的生成標頭stage操作改為write(target,bytes,true,true)，不放寬path.write全域預設，fresh header-v2再跑。原失敗留在header-v1/errors.md；C++／Rust／UE驗證尚未執行。下一E639。

## E637：新增uint64欄位使用BlueprintVisible反射（2026-10-10）

E636 source交付run-mv1r4xea-en0tz7真正terminal18m56s／51calls／$1.45106288；獨立798來源fingerprint shop-undo-identity-source-v2已真正完成。Primary來源檢查新GameplayInput.UndoSequence與UI SettledTick／Sequence／ViewEpoch／UndoSequence／PresentationViewEpoch共六個uint64標記BlueprintReadWrite／ReadOnly。已直接核對D:/UE5.8/Engine/Source/Programs/Shared/EpicGames.UHT/Types/Properties/UhtUInt64Property.cs與UhtNumericProperty.cs不設定IsMemberSupportedByBlueprint，而UhtScriptStruct.cs1152–1154對BlueprintVisible的不支援型別LogError；Int64Property則明確設定該capability。這是本機引擎來源確認，尚未執行UHT或宣稱編譯失敗。

Primary窄修正只把這六個欄位改為UPROPERTY()，維持完整uint64權威序號／epoch／tick、UHT反射與原生C++資料傳遞，不做可能溢位的signed截斷或轉float。既有HUD minimap uint64欄位也用此反射形式。待真正生成／普通UE build驗證；尚未接受。下一個錯誤編號E638。

## E636：Undo把renderer request id與權威input id當成同一個身分（2026-10-10）

首輪修復真terminal run-mv1pvfc4-6os7ey，33m13s／170calls／$5.12578356；獨立798來源fingerprint shop-undo-ui-repair-source-v1完成。E632–E635已有來源修正，尚未測試接受。新的具體來源缺陷：`OmBridgeInner::publish_applied_input` 用自己的AtomicU64產生renderer request id並回傳OmInputSubmitResult.input_id；widget存此值。client InputBridge獨立分配後端u32 input id，CriticalInputResult同時帶兩值。`driver::validated_shop_undo_receipts`直接把authority receipt input_id放入C收據，WorldBridge與widget再跟UI request id比較，從未映射。bridge單元fixture本來就有requestid=submitted.input_id、authority inputid901的例子，原生新fixture卻把兩邊都設41掩蓋差異。兩計數不同時正確退款不能解除等待；若authority id剛好撞另一筆UI id，還可能錯誤宣告另一筆退款。屬來源發現，尚未執行新的失敗測試。

需明確區分兩身分並以原correlation＋owner／epoch／sequence傳遞真正收據，完整有界生命週期；不得把任意applied Ok、gold變化或數值偶然相等當退款證明。加不同起始counter／matching＋unrelated receipt batch／collision／重試及replay regression，經實跑才接受。下一個錯誤編號E637。

## E632–E635：Undo UI 第一輪來源審查未通過（2026-10-10）

真實 Grok terminal `run-mv1oeqy0-tpsegl`，38m32s／217calls／$5.42880788。獨立798來源指紋及差異 `.ai-collab/shop-undo-ui-source-v1.json`、`shop-undo-ui-source-delta-v1.json`。本輪尚未生成、編譯或執行測試；以下是來源發現，不是假稱runtime失敗或通過。

- E632：`omoba-core/src/kcp/client.rs` 正式 `join_selective_lockstep` 仍用 `SHOP_PROTOCOL_VERSION=1`，main依協商把Undo關閉。UI合成fixture設true不代表正常連線可用。需正式client請求已支援的capability2，保留v1 absence與unknown fail-closed。
- E633：`InputBridge::validate_view` 在分類指令前一律檢查 `replica.owns_hero`；其實作只掃場上disclosed kind1英雄。合法retained dead owner沒有英雄entity，Buy/Sell/Undo都遭OWN_HERO_NOT_DISCLOSED。需依真實owner-scoped economy/Undo權威投影允許限定經濟指令，不能為任意缺席/敵方/退役owner放寬移動、施法或其他操作。
- E634：`RetireShopUndoCapture(true)` 保留inputid並設receipt watch，但CanSubmitShopUndo只擋pending、不擋watch；Submit又覆寫id與pin。profile retirement後仍可撤銷的投影可使上一筆未確認請求被新請求取代。另WorldBridge只發布整批收據的latest，而非當前相關id，可能丟掉同批較早的待結算收據。需保留相關身分直到正確終態，測試accepted-id分支而非僅無bridge的id0。
- E635：新native測試只在未綁快捷鍵時送Z/4，再直接呼叫沒有owned viewport的Owner handler；未驗證自訂ShopUndo綁鍵的真正owned事件成功路徑。需沿正式owned fixture補實際綁鍵、精確修飾鍵、repeat、文字焦點、失焦與退休路徑，保留原290cases／119intents／101stages完整前綴，報告實際新事件數。

修復前不接受UI。原權威協定驗收仍是之前範圍，沒有新對局、ACK或完整驗收。

商店撤銷協定範圍接受（2026-10-10）：`.ai-collab/shop-undo-protocol-accepted-v1/summary.json`。Grok4.7非fast協定修復14m3s／60calls／$1.61808176，runtime修復run-mv1nxxbt-p9xmdk真正terminal3m19s／22calls／$0.4547704；Primary獨立審查、窄編譯修正與真正負例regression後，fresh rust-v3實際187distinct（34undo）／三項all-targets＋Unreal Rust橋接all-targets共四項成功，maintained codegen-v3生成／check17檔成功，797來源驗證後未變。selective協定明確遷移7、舊6拒絕，team v2 hash與權威ledger契約；48-byte scalar權威can-undo／reason／sequence／commitment、不傳完整歷史；92-byte新收據嚴格驗錢包，Buy/Sell v1仍40bytes；死亡錢包／真正driver與小數HP不補血回歸均通過。E622–E631原來源／編譯／fixture／runtime失敗保留。UE Undo按鈕／改鍵與外部reason／ABI／HUD尚未接，4.4與父項8／44維持未完，無新對局／ACK或完整驗收。下一批`.ai-collab/moba-owned-shop-undo-ui-task.md`。

## E629：共享Undo判定重構讓死亡錢包發生資源借用衝突（2026-10-10）

fresh shop-undo-protocol-rust-v2先有wire5pass；shop實際47pass4fail。dead_roster_shop_mutates_retained_wallet_and_keeps_live_radius_rules與shop_undo_protocol_dead_wallet_undo_leaves_corpse_hp均atomic_refcell already immutably borrowed。來源undo_moba_shop使用state.clone()；state是Fetch<MobaMatch>，可能保留Fetch借用直到稍後write_resource，而非預期釋放借用。應在限定唯讀借用區塊產生owned UndoPlan，退出區塊才執行寫入，不複製整場比賽來掩蓋資源生命週期。實際原失敗保留，尚未修復。

## E630：兩項新Undo測試混淆tick結算退休與受益失效規則（2026-10-10）

同輪shop另外兩失敗：drain_invokes_reviewed_undo_once在2023預期buy settlements空，卻重複呼叫drain而未經正式begin_moba_match_tick清除上一tick settlements；scalar_admission在2091預期移動使moba_sword失效，但note_movement僅作廢BENEFIT_MOVE，劍是攻擊加成。不可清改正式drain消滅Buy/Sell結果，也不可把所有移動一律作廢所有道具。修正fixture走真tick退休（或明確現有queue cleanup生命週期）及符合道具受益的真combat事件／移速道具。保留四個原失敗斷言記錄。HP fractional真實itemtick與zero／negative／sub-one直接邊界本輪確實pass；不把整個shop組寫成通過。

## E631：撤銷投影未檢查執行所需ItemEffects，與權威結果不一致（2026-10-10）

Primary新增shop_undo_protocol_scalar_missing_effects_agrees_with_execution，真正fresh shop-undo-missing-effects-rust-v1執行1項0pass1fail：買入後移除ItemEffects，真正undo回MissingComponents且錢包未變，但inspect_owner_undo仍can_undo=true，負斷言失敗。應在共享唯讀admission檢查正式執行必需元件，投影給reason19，執行仍原子拒絕，不新增本地退款或假元件。新regression保留，尚未修復。下一個實際錯誤E632。

## E628：首次Undo Rust測試編譯仍有四項fixture接線錯誤（2026-10-10）

fresh codegen-v3核心lib、17檔generate／check成功，內容hash c9dda913c42a5118不變；fresh shop-undo-protocol-rust-v1第一組core-undo-wire在lib test編譯exit101，實際測試0項。shop.rs1939 E0618局部property遮蔽同名factory；team_projector.rs1857／1881／1888三處E0061未補build_step新增undo settlements第六參數。Primary僅將局部property改clamped、三處明確傳空undo settlements，保留斷言及正式API；尚未驗證修正。unused prost::Message等原warning保留。正式server_config::single_lane_config要求MOBA secure_v2_required，main依該設定拒絕legacy join，因此不把允許Story legacy協定誤報為正式MOBA版本漏洞。下一個實際錯誤E629。

## E627：Undo修復後核心編譯發現兩項所有權錯誤（2026-10-10）

Grok run-mv1n7cpr-q5lb1k真正terminal14m3s／60calls／$1.61808176。fresh shop-undo-protocol-codegen-v2 cargo build --lib實際exit101：shop.rs307 E0382把非Copy的ShopError移入tuple後又傳入reason函式；shop.rs372 E0716從臨時try_fetch借出owner，Fetch已在statement尾端釋放。後續codegen/check與測試未執行。Primary做窄範圍編譯修正：先從error.clone取得reason再回傳原error；以let ledger延長資源借用期間，未改權威判定或放寬型別。原失敗完整保留，修正尚未編成，接續fresh v3。報告所稱resource_management複製MobaMatch實際是在legacy adapter測試fixture，不是正式State啟動；正式core.rs呼叫完整setup，不能將該測試描述成正式遷移漏洞。下一個實際錯誤E628。

## E622：MOBA雜湊改變但未遷移實際協定（2026-10-10）

Primary對真正terminal run-mv1l8x8m-0j97ho的來源審查：state_hash_producer僅憑MobaMatch存在混入新economy/ledger域，undo_hash_contract_active也僅測MobaMatch；selective version仍6、canonical-team-view仍v1卻無條件發布新fact32。這不是協定協商，舊版6消費端可能遇到不同雜湊契約。須在正式邊界明確遷移或協商並拒絕不相容版本，維持Buy/Sell v1 receipt不變。不為歷史omfx補接線。尚未修復／未接受。

## E623：Undo投影傳完整歷史，卻未提供權威撤銷可用結果（2026-10-10）

來源審查：owner_shop_undo_projection逐tick複製完整records與consumed components，但shop_available/reason只是購物狀態；client owner_undo捨棄records並固定undo_enabled=true。戰鬥使最新紀錄失效時仍可能顯示可以撤銷。應由共享唯讀撤銷准入給出sequence、available與精確reason，發布有界scalar；完整ledger留權威端／hash commitment，不交給UI猜。未執行UI測試，尚未修復。

## E624：Undo receipt解碼未驗證錢包內容（2026-10-10）

來源審查：ShopUndoReceipt::valid只檢查player、input id與result上限，未拒絕負gold、未知catalog、NaN／負／過量cooldown、空格非零cooldown或成功sequence=0。須沿用正式economy元件驗證及結果／sequence契約，維持v1獨立。這是來源缺陷，尚未執行畸形receipt測試。

## E625：普通裝備重算把正的小數生命值提高到1（2026-10-10）

來源審查：apply_item_stats ExistingBody在hp>0時將min(hp,mhp)小於ONE的結果上調ONE，故0.5生命會被補成1。Primary新增shop_undo_fractional_damage_is_preserved_by_real_item_rebuild，經真正item_tick驗證買入、撤銷與賣出；尚未執行，不能聲稱測試已失敗。修正應保留小數損血、死亡不復活，NewLife仍一次補最終滿血。

## E626：Undo wire首次正式codegen驗證在核心編譯失敗（2026-10-10）

fresh shop-undo-protocol-codegen-v1實際cargo build --lib exit101，shop_undo_wire.rs353的collect／unwrap_or_default缺少records型別，E0282。build.rs已產生game.rs但lib未編成，後續om_codegen generate/check與Rust測試均未執行。原proto stdout/stderr及summary保留，不將生成檔存在當作生成驗證通過。須隨有界來源修復補明確型別或移除不合需求的完整records投影，再用全新證據目錄重跑。下一個實際錯誤E627。

E618–E621範圍結案（2026-10-10）：dead-shop-authority-accepted-v1記錄fresh172distinct測試與core／client／omb共3項all-targets成功，796來源驗證後未變。E619實際缺資源在extract_snapshot的無條件TowerUpgradeRegistry讀取，空upgrade queue的普通driver並不需要它；共同fixture已補正式populate。E620原為來源缺陷，前兩輪未走到滿血斷言，本輪真正driver滿血／一般買賣撤銷不補血與不重複加成都已通過；不倒填不存在的失敗斷言。E621為world原點剛好是隊一基地，已改actual base＋radius外座標。原失敗logs／summary完整保留，HUDreason／wireundo與完整驗收仍pending。下一個實際錯誤E622。

## E621：越界購物driver測試把隊伍一基地原點當作商店外（2026-10-10）

Grok生命周期修復run-mv1kk2tr-6oa316 terminal11m22s／49calls／$1.17332164，Primary審查exact5Rust檔；fresh dead-shop-authority-rust-v2 core六組與all-targets通過，但真正driver1項0pass1fail：single_lane_match_tests.rs5171預期OutsideShop，實際Ok。新fixture將player1移到Vec2::ZERO，而該基地就在原點，所以權威正確允許賣出。不是正式距離檢查失效。Primary保留拒絕斷言，改新driver兩隊的外部座標為實際該玩家基地位置＋MOBA_SHOP_RADIUS+1的Y偏移，不寫死世界原點或地圖座標。未改權威接線／半徑／期望結果；原失敗log完整保留，fresh計畫重跑。下一個實際錯誤E622。

## E620：增加生命上限裝備在重生時只補基礎生命，測試反而鎖定缺陷（2026-10-10）

Primary實際來源審查：spawn_hero以base＋level HP設滿血，複製retained inventory後只dirty；item_tick重算增加mhp但保持現有hp，所以帶＋100HP裝備的新生命少100HP。新driver測試原assert born.hp<born.mhp，反而鎖定此問題。Primary改為正式重生應hp==mhp的回歸斷言；目前regression-v2先停在E619缺resource，尚未走到滿血斷言，不冒稱該斷言已runtime失敗。要求一般化共享裝備屬性初始化，重生在行動前以最終loadout滿血；普通購買／出售／撤銷／dirty tick不得補血或復活舊屍體，不得手工重複加成、double cooldown tick或改弱斷言。原生命週期與無二次wallet政策保留。

## E619：真正死亡購物driver測試缺TowerUpgradeRegistry資源（2026-10-10）

Primary修正E618兩個clone並增加滿血回歸後，fresh dead-shop-respawn-regression-v2真正編譯成功／執行1項／0pass1fail／exit101。shred panic找不到omoba_core::runtime::native::comp::tower_upgrade_registry::TowerUpgradeRegistry。原stdout／stderr／summary保留；要求依真正driver／process_outcomes SystemData補齊共同fixture依賴，不得刪真正driver或手工取代death／respawn。其餘scripts／client消費端未執行。修復後重跑fresh完整受影響計畫。

## E618：死亡購物driver測試把CProperty當Copy，兩筆E0507（2026-10-10）

Grok4.7非fast run-mv1j6pl7-sv5ndx真正terminal33m12s／151calls／$4.0446094，Primary審查exact8Rust檔。fresh dead-shop-authority-rust-v1先通過core shop38／hero_move8／economy2／item7／moba_match69／snapshot11與core all-targets，script driver編譯exit101／未執行測試。single_lane_match_tests.rs5270與5278對CProperty shared reference解參考搬移，型別沒有Copy；Primary依實際Clone取得快照修正測試，不更動正式元件型別或刪斷言。所有原失敗log保留；不把core通過當整個dead shop已接受。下一個實際錯誤E621。

E614–E617修復範圍結案（2026-10-10）：shop-undo-authority-accepted-v1記錄fresh rust-v3真正19項undo全過、相關126distinct測試與core all-targets成功，來源796檔驗證後未變。原rust-v1編譯失敗與rust-v2真正17pass／2fail證據均保留，沒有改成成功；僅權威基礎接受，wire／dead shop／UI與完整驗收仍pending。下一個實際錯誤E618。

## E617：撤銷後屬性重建測試 fixture 缺少 SysMetrics（2026-10-10）

Primary fresh shop-undo-authority-rust-v2 真正執行19測試，17通過2失敗，exit101。shop_undo_rebuilds_item_stats_without_rolling_back_damage_or_xp 在真正ECS系統讀取 SysMetrics 時 panic，fixture 未完整setup。要求依實際SystemData與既有fixture補齊，不得刪除HP／XP／效果斷言或改為手工賦值。原stdout／stderr／summary保留；後續hero_move／shop／item／moba_match／snapshot／alltargets未執行。

## E616：合成之後追加無關購買，舊材料撤銷歷史仍被永久作廢（2026-10-10）

Primary新增 shop_undo_recipe_chain_survives_later_unrelated_purchase，真正執行確認 sword+sword+greatsword+armor 後依序undo4／3成功，但undo2得到UndoUnavailable，預期Ok(2)。上一版只反推最新一筆購買的 covered inventory，未遍歷所有後續已記錄購買；無關購買會再次把仍被合成遮蔽的材料紀錄eligible設false。要求通用有界歷史還原遍歷，保留真正combat／use／sell／move／death失效，禁止盲目復活eligible或回滾現時金錢／HP／XP。原失敗證據保留；同thread grok-4.7非fast限定修復，獨立驗證後才能接受。真正移動、護盾、負餘額案例在本次通過；不是整套撤銷成功。下一個錯誤編號E618。

## E615：配方購買把較舊材料紀錄永久失效，且移動／護盾缺真正消費端驗證（2026-10-10）

Primary獨立來源審查發現：commit_purchase 對所有 older records 執行 can_restore，合成購買暫時消耗原材料就設eligible=false。撤銷合成雖還原原材料，原本合法的材料購買仍永遠不可撤銷；不能形成正常多步購買／撤銷鏈。這是實際來源缺陷，尚未執行新的鏈測試，不冒稱測試失敗。修復必須區分後續可逆購買造成的暫時遮蔽與真正不可逆使用／出售／收益／來源失效，保留 newest-only 與每次原子還原的 shape 檢查，不能盲目恢復 false eligibility。

另新移動測試僅直接呼叫 note_shop_undo_movement，未走 hero_move::Sys 的 optional SystemData／真位移接線；新增護盾全吸收／HP不變失效分支亦缺真正 BuffStore／傷害消費端案例。要求補真ECS位移、root／blocked／facing與no-ledger不插入，以及真護盾傷害／攻擊收益與credit不誤改的檢查。Undo gold<0 應延續原 buy／sell InvalidBalance 原子拒絕。實際SimWorldSnapshot僅render投影且無authority save schema，未假造持久化；backend鎖步hash在 omb/src/lockstep/state_hash_producer.rs，目前不含inventory或ledger，必須在後續協定啟用前處理／明確協商，不得宣稱整套完成。

## E614：shop undo 新測試漏匯入 specs::Builder，cargo 編譯三筆 E0599（2026-10-10）

Grok4.7非fast run-mv1h36m7-5o8fw1 真正terminal31m7s／140calls／$3.49688092，來源exact4 Rust檔已審查，未動wire／ABI／Unreal。Primary fresh shop-undo-authority-rust-v1 第一項 cargo test --manifest-path omoba-core/Cargo.toml --lib shop_undo exit101，passed0，整個plan successfalse；後續shop／item／moba_match／snapshot／alltargets均未跑，不能稱14測試成功。stderr在shop.rs865／878／879為EntityBuilder.with找不到方法，編譯器指出需匯入specs::Builder；另hero_move_tick.rs新WorldExt為unused_import warning，原warning保留。原summary／stdout／stderr／errors.md完整保留，不覆蓋。

同thread bounded修復涵蓋 E614編譯與E615實際來源／測試缺口；完成後Primary重新審查並跑fresh受影響Rust計畫。既有圖示範圍接受不改寫為undo成功。父項8／44、ABI21／IPC6與完整驗收最後不變；無新對局或authority ACK。下一個實際錯誤編號E616。

E603–E613範圍驗證結案補記（2026-10-10）：原失敗不可改寫。item-icons-build-v4普通nocache／refreshfalse PASS115.31s；fresh native-v3真實8Success／0error／0incomplete，僅刻意missing_item負例warning保留。四張當次原圖已逐張original檢視，400完整六卡與1.25文字回歸通過。fresh owned-v1真正290／119／101、完整mode-chrome-v2前綴／cleanup／六圖與來源資產不變通過。正式匯入11與fresh重開0save已有獨立證據；記錄item-icons-accepted-v1，非全套驗收。Primary唯讀路徑失誤及防重複措施另存 `.ai-collab/item-icons-primary-verification-notes-v1.md`。下一個實際功能錯誤編號E614。

## E613：緊湊標籤與熱鍵列的預設 profile 不一致，tooltip 測試依賴具體型別 cast（2026-10-10）

E612 Grok真正terminal29m1s後獨立來源審查：GetItemHotkeyText缺owner/profile時使用MakeDefault，但新WriteItemSlotLabels仍傳nullptr給CompactChordLabel／tooltip binding。compact隱藏重複熱鍵列、又移除ordinal後，preview／尚無profile入口會失去實際顯示熱鍵，tooltip也誤寫未綁定。這是來源發現，尚未跑新build／native，不冒稱實跑失敗。Primary以相同cached default選擇統一兩條呈現路徑；有真實profile且使用者unbind時仍保留該profile，不以預設覆蓋。加入compact可見熱鍵與實際GetItemHotkeyText一致的檢查。

同批新tooltip檢查直接static_cast IToolTip為SDeferredToolTipText，依賴具體實作。Primary改用公開IToolTip::GetContentWidget＋prepass後讀真正文字內容，並保留FullIdentity／主要／次要／self資訊斷言，避免型別不符或只讀私有cache。兩項Primary修正與Grok量測排版一起待普通建置／fresh native重驗，不放寬fit／font／bounds。

## E612：固定六欄文字份額使 compact 道具列撐高，並破壞既有 1.25 倍排版（2026-10-10）

ordinary item-icons-build-v3真正PASS23.57s／nocache／refresh=false。native-v2實際process0/helper1，8名稱6Success／2Fail／0incomplete，一筆unknownTexture warning保留。E610資源已成功驗十一張平台1254；E611真實viewport 800圖已完整六卡，Primary逐張ORIGINAL看過，舊裁切修正有效。但Presentation在400x300 presentation fits失敗，原ResponsiveHudLayout另退化為Text scale1.25 at400x300 fits失敗，不能稱E611或整批成功。

實測400：combat384寬stack/inset96/limit196卻高336，items194；side296寬limit284卻高354，items212。1.25原responsive combat高296>side284，items124。固定把全部狀態字串塞進六份窄寬，原Concise仍含序號／完整名字／狀態／binding，造成多行與可能重排，並非文字floor太大或允許縮小的理由。需一般量測卡片／文字密度，避免盲目固定六欄；保留圖示、熱鍵、cooldown／truthful狀態、完整identity／self／primary-secondary tooltip及真實命中。400文字1.25既有要求不刪，80x60 no-fit與1.5未保證界線不改。原v2圖／報告保留，同Grok4.7非fast有界修復後再驗。

## E610／E611：貼圖資源未等待編譯，獨立圖示視窗未重現正式 HUD 幾何（2026-10-10）

item-icons-native-v1真正terminal：process0但helper1，8名稱6Success／2Fail／0incomplete，一筆unknown Texture2D預期缺席warning保留；audit完整保存。E610 ItemIconResources 22errors，十一張source正確但立即GetSizeX/Y均32（非1254），需按真正texture async compilation lifecycle完成後驗實際平台資料；前面commandlet先FinishCompilation再驗已真實1254，不能改成接受32或用Source代替。

E611 ItemIconPresentation在400x300 slot0 icon inside-window失敗。Primary逐張ORIGINAL看當次唯一800六卡圖，實際貼圖可辨識，但右側Greatsword／文字與底部Guard／Stride文字明顯裁切；800斷言通過仍不能算畫面成功。Standalone SWindow+TakeWidget只Resize／prepass／redraw未走真實production viewport與normalization／widget lifecycle。需真正viewport/localplayer物理client幾何與正常settle，讓完整卡片／文字／icons都在實際capture範圍，不放寬字級／scale／bounds、不後製縮圖／裁圖掩蓋。修fixture若產品正式路徑原本正確；若填滿六卡正式fitter有實際缺陷才修一般layout。原失敗與PNG不改，後續Grok4.7非fast bounded修復後獨立重驗。

已實際通過：Ownership／HudAssetRegistry／Fallback／OwnedItemCards／ResponsiveHudLayout／ShopPanelSearchAndRetirement六項。另durable-repeat-v1兩新程序都11verify／0save；11package雜湊／size／mtime／ctime交叉檢查全不變。以上不代表圖示整批或完整驗收通過。

## E609：NullRHI 匯入的貼圖平台資料為零，嚴格 policy 正確拒絕（2026-10-10）

實際重驗：同Grok4.7非fast修復terminal6m36s，Primary讀安裝Engine CanEverRender／CachePlatformData／AllowGlobalShaderLoad／GetSizeX確認根因；唯一source delta為scripts/import_item_icons.lua。保留未變C++ ordinary build-v2成功19.10s；改正式命令列AllowCommandletRendering＋d3d12、移除NullRHI／NoShaderCompile，尺寸與policy未放寬。Lua測試與正式Content預設validate-only11均過。fresh bootstrap-v2真正首次11import／11save／0failed，第二個新程序11verify／0save／0failed、平台1254×1254與mtime檢查通過。獨立再跑durable-repeat-v1與package hash／timestamp交叉檢查仍進行中，native／畫面／owned尚未驗，不能稱整批圖示接受。原失敗不改。

E608修復後 ordinary item-icons-build-v2真正PASS19.10s，refresh=false/nocache。實際首次item-icons-bootstrap-v1已持久stage11原PNG＋manifest，但run-1 commandlet真正exit1：imported0／verified0／failed11／saved0，每張source1254×1254、platform0×0，錯誤platform dimensions are not 1254。沒有run2或成功summary；完整JSON／engine log保留。不可用來源尺寸假充平台尺寸，也不可放寬斷言或縮圖。需按安裝Engine的commandlet rendering／texture platform-data條件修正正式流程，再fresh驗證真實貼圖與重開資源；修正前不算匯入成功。

## E608：道具卡標頭與 SImage 資源讀取未符合 UE5.8 公開 API（2026-10-10）

實際 item-icons-build-refresh-v1 terminal失敗：OmCommandBarWidget.cpp27 C1083找不到Widgets/Layout/SHorizontalBox.h；OmItemIconPresentationTests.cpp53／318／326 C2248，SImage::GetImageAttribute是protected。已讀安裝Engine的SImage.h確認，不能透過改Engine權限、unsafe cast或刪除嚴格貼圖斷言解決。需正確SBoxPanel標頭與合法、唯讀且觀察實際paint brush的production seam。保留完整build／UBT log；這不是下一次Engine refresh的理由。交同Grok4.7非fast有界來源修復後獨立審查，再fresh ordinary nocache，尚未圖示匯入／native／接受。

E607實際重驗：修正bootstrap後原Lua測試exit0、item icon import validation tests passed；首次候選validate-only也實際驗完11張。之後測試優先正式Content來源，避免把cache當唯一正式輸入。C++錯誤不抹去Lua通過，但Lua也不能代替編譯或資源／畫面驗證。

## E607：道具 Lua 測試 bootstrap 指向 tests 子目錄（2026-10-10）

實際 tools/lua/lua.exe scripts/tests/item_icon_import_test.lua exit1，尚未執行斷言：require('_bootstrap') 搜尋 scripts/tests/_bootstrap.lua 而正式 bootstrap 位於 scripts。Primary修正從 tests 的父目錄載入，同時測試優先使用正式 Content manifest；尚未 bootstrap 時才使用候選輸入。不得為測試新增另一份 bootstrap 或依賴執行者的 LUA_PATH。重跑前不宣稱成功。

同批普通 item-icons-build-v1 nocache 真正 FailedDueToEngineChange，UBT39.29s，列出 NetCore 與 Engine receipts 需重建；不是遊戲 compiler 成功或產品 C++ 錯誤。保留原失敗，依既有明確 gate 先 fresh refresh-engine/nocache，再 fresh 普通 nocache，不能只以 refresh 成功當一般建置通過。

## E603–E606：道具圖示交付的獨立來源審查（2026-10-10）

Grok4.7非fast run-mv1bnqa3-oy5l36 真正terminal41m1s／114calls／$2.95331004；generic Git／HEAD／diff-check通過，793檔fingerprint為7新增＋6修改，另HUD manifest新增11項。未建置、未匯入、未執行新測試、未接受圖示。以下是實際來源發現，不冒稱已發生compiler／runtime失敗。

- E603：scripts/import_item_icons.lua與OmItemIconResourceTests.cpp固定讀忽略版控的.ai-collab/candidates/item-icons-v1/manifest.json；只stage PNG而沒有持久manifest，清除協作cache／新clone即無法重跑正式工具／資源測試。item_icon_import.validate也未限制candidate_dir在project clone內（CLI目前固定安全位置，但可呼叫module無此guard）。需持久project-owned manifest＋PNG、明確首次候選bootstrap入口與後續可攜預設；外部source只作provenance。
- E604：DiscardCreated僅由.uasset當下不存在判定bCreated，失敗時無條件刪同名uexp／ubulk；若原本有sidecar而缺.uasset，會刪除非本次建立的檔案。不得以一個主檔缺席推斷整組檔案都歸本次所有。先全面collision／ownership預檢，清理僅處理確認本次建立的檔案；原使用者內容不可刪除／覆寫。
- E605：native ParseJobs只驗字串／numeric範圍與destination，未核對真正OmContent catalog ID→item ID；Catalog／Bytes在有限範圍檢查前先cast，Bytes也沒有Lua同等16MiB上界。PolicyMatches漏掉宣稱設定的MaxTextureSize0／FilterBilinear。另include Engine/TextureCompiler.h與安裝Engine唯一Public/TextureCompiler.h實際路徑不符。需authored mapping／finite-before-cast／完整policy與真實標頭；尚未跑compiler，不虛報編譯錯誤碼。
- E606：ItemIconPresentation以bIcons條件跳過所有重要圖示／square／shop resource斷言，可能只靠fallback文字就Success；唯一capture在先清空slot0／換unknown slot1並打開shop之後，不能證明六張實際不同道具卡的畫面。資源整合驗收須嚴格要求真實貼圖，另保留明確fallback負例；保存未被商店遮擋的六張卡、400/800及populated shop當次圖，維持真實pointer／bounds／text floor檢查。這不是授權增加其他玩法。

正面來源核對：runtime實際LoadObject一次後cache（含miss），UPROPERTY TMap保留Texture；不是只FindObject未載入。ShopRows先ClearChildren再一次預配置brush，避免同一build內TMap擴容破壞SImage指標；六卡brush為固定array。原flat PNG安全guard、item typed resource、命令與ABI21／290前綴未改。不要把最初report「already loaded」措辭誤當無LoadObject缺線。以上有界修復後才執行普通nocache build／實際匯入與受影響驗證，完整驗收最後。

## E602：統一點包含 helper 時不可遺漏被測 widget 尺寸檢查（2026-10-10）

E601 Grok source terminal 後，Primary 審查發現 bSettingsInside 改呼叫 ContainsPoint(SettingsCenter,WindowPos,WindowSize)，helper驗的是window尺寸，原本SettingsSize.X/Y>1卻遺漏。這是來源審查發現，沒有冒稱實跑失敗。Primary直接補回原settings兩軸有效尺寸條件；不改閾值、不再開額外writer。新paint/window座標修正保留，後續普通build與fresh owned回歸才是執行證據。

防再犯：共用幾何helper只能取代它實際驗證的predicate；外層被測widget自己的有效尺寸／identity／visibility前提不得因抽取helper而刪除。

## E601：owned TD 邊界檢查混用 window tick 與 paint 座標（2026-10-10）

實際重驗：Primary保留settings有效尺寸條件（E602）後，普通mode-chrome-build-v4 nocache／refresh=false成功36.31s；fresh mode-chrome-viewport-v2真正290／119／101到mode-retire、原274／119／94前綴與editor／publisher／profile／listener清理全過，六張當次原圖逐張目視。E601原失敗點與後續mode／generation／epoch／quiet／retire檢查均通過；接受摘要mode-chrome-accepted-v1。原v1失敗與下方診斷保留，沒有新增authority ACK／整場模擬。

mode-chrome-viewport-v1 真正 terminal 失敗：276cases／119intents／95stages，last mode-td-controls；native／workflow均false，editor／publisher／profile清理與listener釋放均成功。原native.json／workflow.json／ue.log與四張已產生PNG保留。StartRound center400.0,90.1 inside0，Dart center669.9,141.5 inside1，兩者settingsInside0，combat／minimap遮擋均0；不能宣稱290全過或模式UI接受。

Primary讀到 MeasuredCombatCoversTag 從 Window->GetWindowGeometryInWindow() fresh arrange得到tag center，combat／minimap／settings使用GetPaintSpaceGeometry，卻以Window->GetTickSpaceGeometry()判斷inside。這與E598相同的具體座標混用仍殘留在owned測試，先前修正只涵蓋ResponsiveHudLayout，審查未搜尋另一個window邊界helper。需用一致且真正已呈現的paint-space window/tag/cluster幾何，再以公開pointer hit確認；不能移動正式按鈕、刪除bounds／settings檢查、加常數offset或放寬誤差掩蓋。

後續同Grok4.7非fast bounded修復，保留ABI21／290宣告與274原前綴，先獨立來源審查，再普通nocache build與fresh owned回歸。native-v2實際8Success／兩筆保留warning為先前真實證據，不能把尚未重跑的修復稱通過。完整驗收最後。

## E600：設定 native 成功但保留一筆 scroll 警告（2026-10-10）

mode-chrome-native-v2 真正8Success／0errors／0incomplete，固定Lua gate exit0。audit完整保留兩筆warning：FrameContract預期非法mode3拒絕，另ControlRuntimePersistence「LogSlate: Unable to scroll to descendant as it's not a child of the scrollbox」。不可宣稱native零警告。E598/E599原斷言已過；此次僅兩個測試來源改動，產品source／ABI21未變。

scroll警告意指請求時descendant不屬於指定scrollbox，但目前尚未證明產品缺陷或具體呼叫點，不能逕稱已修／忽略原log。當次設定公開hover→down→capture→up→release、同寬spin／stack／取消／關閉／還原測試仍全過；以非阻塞fixture警告保留追蹤，不放寬檢查／不關閉日誌。後續若修正，需核對當前widget生命週期與pending scroll target，不能使用舊row identity。後續owned viewport已terminal，失敗另記E601；source writer與consumer不得重疊。

## E598／E599：ABI21 後真正 native 排版與設定回歸失敗（2026-10-10）

實際重驗：mode-chrome-build-v3普通nocache／refresh=false PASS（5actions／22.14s）；mode-chrome-native-v2八項真正Success／0errors／0incomplete，未加capture-settings也通過。原reservation bounds／stable fit與同寬spin／stack／公開改鍵／取消／關閉／還原檢查保留並通過。兩筆warning如E600保留；後續owned的新失敗另記E601，模式UI未接受。以下診斷前時態為歷史，不覆蓋原失敗資料。

後續來源診斷（尚待重驗）：Grok run-mv1a7v4l-ajtosg 已terminal；Primary核對Engine SWidget實碼，E598是reservation新測試將GetCachedGeometry／tick含desktop位移的window與GetPaintSpaceGeometry的combat比較，panel也重複ChildScale。修為同一真實paint space，bounds／overlap／stable-fit等斷言原樣保留。E599是Slate.AddWindow(SettingsWindow,bCaptureSettings)使無capture的視窗不顯示；shared PresentSettings 在需要呈現時ShowWindow，PNG輸出仍可選。只改兩個測試檔，產品fitter未改；mode-chrome-build-v3普通nocache正在跑，尚不能稱重驗通過。

普通 mode-chrome-build-v2 nocache／refresh=false 成功（24actions／UBT129.67s）。mode-chrome-native-v1 實際8項完成：6Success、2Fail、0incomplete、1筆FrameContract預期拒絕mode3警告。native process exit0不代表成功，固定Lua gate正確exit1；原 native-report/index.json、native-engine.log 與 mode-chrome-native-v1-audit.json 保留。ABI21 FrameContract與五項HUD owner成功。Rust bridge6／all-targets、renderer fixture18／client check、cbindgen byte identity／ABI21 C++layout、Lua31均有實跑證據，不當作UI成功。

- E598：ResponsiveHudLayout 在第一組800×600 text1 minimap-left measured reservation失敗「combat stays inside the window」。實際fit log宣稱 fitted1／combatW480／render -176，但真正paint bounds不合格。必須診斷實際paint/window/normalized/box座標與layout時序，保留所有bounds／overlap／font floor／no group scale檢查，不能用DesiredSize或fit flag代替畫面。
- E599：ControlRuntimePersistence 未指定capture旗標時，open request2066與narrow360兩次OmSettingsPanel量測均0；窄視窗仍SHorizontalBox、stacked0。失敗於OmActionOwnerTests.cpp913。先檢查真正window呈現／paint／capture分支／settings生命週期；此次未證明產品回歸或環境原因，不可猜測。測試需在無可選截圖參數下也實際呈現再量測，不能以換成capture-only執行掩蓋失敗或放寬stack條件。

後續bounded Grok4.7非fast修正只處理上述真實失敗。原ABI／ownership測試保留；目前模式UI尚未接受，owned viewport未跑，完整驗收最後。

## E597：模式不可用被選填 fog epoch 阻擋（2026-10-10）

E595/E596 repair terminal run-mv18y0zu-l4golx 後，Primary 來源審查確認：ConsumePresentationEpoch 對沒有fog-grid的fixture一直不設 bPresentationEpochSeen；PollAuthoritativePresentation 卻在 !bPresentationEpochSeen && Mode==0 時直接 return，故先前MOBA／TD切到不可用的正式mode0會被忽略。這是具體來源條件，不虛報尚未執行的native失敗。另一方面fog-less view_epoch變更也無法清除selected caches；worker報告誠實標明限制，但不符合使用者通用解要求。

決策：新增真正的frame presentation view_epoch，維護ABI21 producer／C header／consumer／layout checks；mode0的有效性不再以可選fog-grid為前提。選填fog-grid不是完整view lifecycle來源，不能以特定fixture加fog資料掩蓋缺線，也不能以sequence／tick冒充epoch。保留E595/E596編譯失敗，擴充驗證包含無fog-grid的mode0／epoch切換、控制frame保持與reset清空。

## E596：誤認原生 frame 具有 view_epoch（2026-10-10）

mode-chrome-build-v1 實際 C2039：OmWorldBridgeActor.cpp2928 使用 Frame->view_epoch，但真實 ABI20 OmFrameHeader 沒有此欄位。原有 OmFogGrid 才有 view_epoch；Rust snapshot 的同名欄位不能當作 C header 已存在。Primary 前次 source review 也未對照實際標頭，已由真正 compiler 揭露，不宣稱 epoch 功能完成。

必須從真實既有權威 epoch／生命週期來源接線，明確處理缺席；禁止以 sequence、tick、round、hero generation 或固定值冒充 view epoch。先讀實際 frame/header/producer 與 reset seam 決定通用處理；如需改 ABI，須完整維護產生器、消費端和 layout 驗證，不能手塞 generated header 或僅令編譯通過。保留原 build log，修正後重新普通 build 與 native／owned 事件檢查。

## E595：Slate double 座標窄化至 float span（2026-10-10）

普通 mode-chrome-build-v1（nocache，非refresh）實際 cl 編譯 OmCommandBarWidget.cpp 報 C2397，行1998／2016：MiniSpans.Add({Margin, Allotted.X - Margin}) 與 CombatSpans 同式。UE FVector2D.X 是 double，FHudSpan 欄位 float，aggregate list initialization 禁止隱式 narrowing。原 UBT log／build evidence 保留；此次為真正 compiler 失敗，不是 FailedDueToEngineChange，不得以 engine refresh 處理。

待所有當前 consumer terminal 後，僅對兩個已有限且已驗證的橫向邊界作明確 scalar 型別轉換（或一致型別的邊界 local），保留量測值與完整 fit predicate。重跑 ordinary nocache build 和受影響 native／owned 檢查。後續使用 UE double 幾何填 float aggregate 時須在邊界明確處理型別。

## E593–E594：量測 fitter 修復來源審查（2026-10-10）

run-mv17dbod-a0yi23 已真正 terminal（32m10s／93 calls／USD2.401556），Primary 逐讀 repair report 與實際 fitter。尚未建置，不能稱為 compiler／runtime 失敗。

- E593：RefreshClusterScales 的 Fits lambda 使用 bReserveFailed、LargestReserveSpan、ChosenTravel、ChosenRender、bPlacementReady，但這五個 local 在 lambda 後才宣告，C++ 無法捕捉尚未宣告的名字。將原有初始化移至 lambda 前，不能重複宣告；以普通 UHT／C++ build 驗證。
- E594：最終 fallback 縮窄後僅依 TryPlaceMeasuredClusters 就標 bFitted=true，漏掉既有 Fits 的內容寬高與可用高度檢查。任何標 fit 成功的分支都須通過同一完整 Fits predicate，不能以 placement 成功取代尺寸驗證；以實際 ResponsiveHudLayout 與 owned viewport 驗證。

兩項均為已讀來源的具體缺陷，已交同 Grok4.7 非fast thread 作 bounded repair。原 E590–E592 及失敗資料保留；尚未接受模式 UI。

## E590–E592：模式HUD來源審查拒絕暫時排版與身份alias（2026-10-10）

Grok模式介面run-mv165eqh-l9ra18已terminal，Primary獨立讀10檔變更與報告、786檔fingerprint／Git及HEAD檢查後，尚未建置／測試就發現下列實際source缺陷；不虛報runtime失敗或壓測。

E590：ApplyEntityOverlay每entity呼叫SyncAuthoritativePresentation；bridge在entity_count迴圈發布，Sync無條件反射讀mode並遞迴ForEachWidget／SetVisibility／tag，TickComponent也每tick走同一條。形成O(entity數×HUD tree)熱路徑，違反既有節流。必須cache結構role、只在模式／選取完整identity／可用性改變時更新，無關overlay及穩定tick不能重走／重寫；來源／tree／epoch／退休時正確回收。

E591：RefreshClusterScales在既有fitter之後，TD無條件render shift -400px，未reserve／refit／clamp。800px內近528寬戰鬥列可被推到負座標，400更差；尚無當次paint量測，不冒稱截圖證據。使用量測過的mode-aware composition／占用區域／fitter通用解，保留TD真實click與小視窗／minimap side／文字設定，不用另一個魔術offset或放寬邊界。

E592：新選取只存NewTower.EntityId，丟掉EntityGen；OverlayEntityKinds也只按id。選取(id88001,gen1)會誤認可見新塔(id88001,gen2)，舊gen removal還可清掉新life。新stalecase僅另一id88002，未覆蓋reuse。須完整life／kind／visibility比對與同id換gen／stale removal實際案例，生命週期清cache。

原source／報告與`.ai-collab/mode-chrome-primary-review-v1.md`保留，bounded修復同thread後才驗證。模式資料基礎mode-projection-accepted-v1仍成立；父項8／44、完整驗收最後、無新權威對局或ACK。下一個實際新錯誤E593。

## E589：Rust fixture 尾端 resource borrow 晚於 World 釋放（2026-10-09）

後續：E589修復已獨立審查，mode-projection-rust-v2實際8測試與3all-target checks全部通過。mode-projection-build-v1普通nocache／refresh=false實際失敗為 FailedDueToEngineChange（build.log339／UBT-Log397、450），不是新C++編譯缺陷。原紀錄保留，僅依實際條件啟用既有maintained refresh，之後須另跑fresh普通nocache gate才能接受；尚未建置成功。不手改Engine／config／BuildId。

實際 mode-projection-codegen-v1 通過，mode-projection-rust-v1 的 core2／client2 通過。橋接測試編譯失敗 Rust E0597，位置 omfue/bridge/src/driver.rs:3215；新增 production_mode_decodes_into_native_extras_without_commands 內 project 函式把 world.read_resource::<TeamProjectionRuntime>().latest_frames.get(&1).unwrap().clone() 作為尾端表達式。edition2021 的 Fetch 暫存解構晚於區域 World，必須先將 clone 存入 owned local，結束 borrow 再回傳。不改正式投影路徑或斷言。原始 stderr、stdout、summary 保留於 .ai-collab/.bridge-data/worker-pool-machine-v1/mode-projection-rust-v1/。尚未通過 bridge／Unreal／當次 viewport；修復後須重新驗證。下一個實際新錯誤 E590。

2026-10-09 最新受影響重驗：settings-physical-build-v5 ordinary no-cache／refresh=false PASS（UBT18.92秒），native-v4兩項Success／0warnings，settings-physical-viewport-v1實際274／119／94與完整原前綴、清理全過，七張當次圖逐張original目視。E580–E588相關失敗保留，當前設定範圍已由Primary接受於 `.ai-collab/settings-physical-accepted-v1/summary.json`；不代表全計畫或OS DPI／PIE完整驗收。下一個實際新錯誤E589。

## E588：公開按鈕路徑down／up已handled，卻沒有啟動改鍵（2026-10-09）

ordinary settings-physical-build-v4 nocache／refresh=false PASS（UBT19.96秒）。native-v3實際ResponsiveHudLayout Success／0warnings（含11設定倍率案例）；ControlRuntimePersistence在E585／E587交叉、同寬spin、堆疊邊界、font18後，失敗 `Stacked capture is active`，0warnings。新HitClick已驗公開LocateWindowUnderMouse精確SButton leaf、RoutePointerDown／Up handled，卻不能把handled當OnClicked成功。Primary查實際UE5.8 SButton.cpp403–458：一般mouse-up以IsHovered判斷是否執行OnClicked，否則仍回Handled。須透過公開Slate pointer move建立實際hover，維持同user／pointer與公開path，並驗capture副作用；不能直接呼叫OnMouseEnter、OnClicked或生成強制path。原native-v3與audit完整保留。下一個實際新錯誤E589。

修正（source only，測試、建置都還沒跑，不能寫成通過）：`SButton` 預設 `DownAndUp`。`OnMouseButtonUp` 在 hovered 且 `HasMouseCapture` 時才 `ExecuteOnClick`；條件不成立仍回 `FReply::Handled()`。native-v3 的 `HitClick` 沒有 pointer move，所以 `IsHovered` 仍是 false，down/up handled 不能啟動 `BeginChordCapture`。helper 在同一條 `LocateWindowUnderMouse` leaf path 上呼叫公開 `RoutePointerMoveEvent`（同一 user、`CursorPointerIndex`、`bIsSynthetic` false），再讀 `IsHovered`。down 之後要求 `HasMouseCaptureByUser` 是這個 user 與 pointer；up handled 只代表事件被收下，helper 還要求 capture 已釋放。缺 hover、capture、leaf 或幾何時，失敗字串寫明缺哪一項。沒有改 `ClickMethod`、沒有直接呼叫 `OnMouseEnter`／`OnClicked`、沒有 `GeneratePathToWidgetUnchecked`。改鍵是否生效仍由後面的 `IsCapturingChord` 斷言。沒有 PASS。這次沒有觀察到 E589。

E586／E587這次已越過／通過原失敗斷言；完整設定功能仍因E588未接受，尚未跑settings owned viewport。

## E587：實際文字放大提交後堆疊列超出面板（2026-10-09）

ordinary no-cache settings-physical-build-v3 PASS（refresh=false／UBT41.85秒）。settings-physical-native-v2 process0但兩項Fail／一項warning，ControlRuntimePersistence 已越過 E585 的實測並排／堆疊交叉與同寬 spin 提交，失敗於 `Stacked row stays inside the same panel`。尚未定位是 production 寬度配置或重建後未繪幾何；不可移除 row>1 與 row<=panel+1 的要求、放寬字級或假設固定 DPI。保留完整 native-v2 與 `.ai-collab/settings-physical-native-v2-audit.json`。下一個實際新錯誤 E588。

修正（source only，測試、建置都還沒跑，不能寫成通過）：失敗訊息沒有印出 row／panel 寬度。同一次檢查裡面板絕對寬仍在 side snap 的 1px 內，而且 `ReadStacked()==1`，所以目前列的直接子件已是 `SVerticalBox`。垂直 `SScrollPanel` 會把子件的橫向排成捲動區寬度，排得到的列不應寬過面板；沒畫到的列則是預設幾何，絕對寬是 0，`RowAfter > 1` 失敗。scale 2 的堆疊列比 scale 1 高，CastQ 前面還有數個戰鬥列，窄並排視窗裡它可以落在捲動區外。`SetValue` 只讓 cache 失效，重建發生在隨後的 paint。測試在那次 present 之後重新尋找目前的 `BindingRow.CastQ`，用公開 `OmSettingsScroll.ScrollDescendantIntoView` 捲兩次並各 present 一次，再讀目前列。斷言仍是 `RowAfter > 1 && RowAfter <= PanelAfter + 1`，訊息帶實際寬度。沒有改欄寬、沒有放寬字級、沒有假設新的 DPI。沒有 PASS。這次沒有觀察到 E588。

E586 native-v2 再現，診斷已確定 status font10 正確、label font24 錯誤；widgets=1，labelPaint=0，label geometry0。labelExists 欄位實際命名的是「label已繪」而非指標存在，因此不能寫成 label 不存在。floorMath/statusPaint 均通過。須查重建／public scroll 後持有的 widget 是否已退休，重新尋找目前樹與真正 paint witness，不能用未繪文字當通過。當次一項 warning 也須讀實際內容並處理。尚未 PASS。

修正（source only，測試、建置都還沒跑，不能寫成通過）：native-v2 的唯一 warning 是 `LogSlate: Unable to scroll to descendant as it's not a child of the scrollbox`（SScrollBox.cpp:650），緊接在 800×600 scale 0.50 的診斷之前。`ApplyBindingFilter` 只用 live owner profile；responsive fixture 沒有 live owner，`ActiveProfile` 仍用 automation preview 建列，過濾卻把每一列設成 Collapsed。指標還在樹上，所以 widgets=1。Collapsed 不會被 arrange，`FindChildGeometries` 因此發出那則 warning，也不會畫出幾何。`SWidget::Prepass_ChildLoop` 對 Collapsed 只更新 visibility attribute；`UE_SLATE_WITH_ATTRIBUTE_INITIALIZATION_ON_BIND` 預設是 0，未初始化的 `FSlateFontInfo` 是 Size 24。status 在捲動區外且可見，所以字級是 10、幾何是 744×15。生產字體不是 24。過濾改為 `ActiveProfile()`，有 live profile 時行為不變。測試在每次 settle tick 之後重新取得 `BindingRow.CastQ`，對 `OmSettingsScroll` 呼叫公開 `ScrollDescendantIntoView`，再畫一次才讀目前的 label／row。`labelExists` 改記成 `labelPainted`。沒有 `!painted || PASS`、沒有跳過 body label、沒有放寬 floor 或 0.05、11 個 case 都在。沒有 PASS。這次沒有觀察到 E588。

## E586：設定0.5倍率繪製複合檢查失敗，根縮放已正規化（2026-10-09）

實際 settings-physical-native-v1 的 ResponsiveHudLayout 失敗於 `Settings body 800x600 text 0.50 is painted at the physical floor`（OmResponsiveHudTests.cpp1015）。當次log body10／floor10／applied0.50，root layout/render0.555，normalized layout/render1.000、800x600。因此不能僅憑訊息斷言縮放補償失敗；bBody有多個條件，須記錄status／label字體及paint幾何以定位。保留完整failed report，不放寬10／18端點、跳過倍率或用未繪文字宣稱通過。下一個實際錯誤E587。

修正（source only，測試、建置都還沒跑，不能寫成通過）：用當次數字靜態對過條件後，floor<=body、低端點 `round(11*0.5)=6<10`、以及 scale<=1 的成長條件都成立。正規化容器 render 1.000 時，字級 10 乘上同一個累積 render 也會過下限。剩下的是字體不相等、列不存在，或 status／label 的繪製幾何。`Paint` 裡的 `NativeTick` 會在該次繪製中途才套上 normalizer，捲動區外的列若沒再畫到，會留下前一次較小的幾何，而 `bPainted` 只看尺寸，會把舊幾何當成已繪製。測試在讀幾何前用公開 `ScrollDescendantIntoView` 把 CastQ 列捲進視窗，再以視窗 DPI 做一次 `SlatePrepass`／`Tick`／`ForceRedrawWindow`。斷言訊息現在帶 widgets／fonts／floorMath／statusPaint／labelPaint 與 status、label 的 paint 幾何。沒有放寬 0.05、沒有跳過 0.5 或 2、沒有把未繪文字當成通過。沒有改 `clamp(10*scale,10,18)`。沒有 PASS。這次沒有觀察到 E587。native-v2 已證明這段捲動不足；實際原因與後續修正在 E587 一節。

## E585：同視窗文字回歸找不到預期並排欄位寬度（2026-10-09）

普通 no-cache settings-physical-build-v2 PASS（refresh=false／UBT68.27秒）後，actual D3D12 native-v1 process0但workflowFAIL，兩項測試均Fail／零警告。ControlRuntimePersistence 實際失敗 `Same settings window has a side-by-side width at text scale 1 (last 0)`（OmActionOwnerTests.cpp827），在production文字spin提交之前。320..1000的requested window width搜尋沒有找到並排；須區分實際paint／layout／OS scale與本地可用寬度，並查row類型／量測，不硬改寬度或跳過斷言。證據 `.ai-collab/settings-physical-native-v1-audit.json` 與 worker-pool native-v1完整目錄保留。尚未開始這批owned viewport。

修正（source only，測試、建置都還沒跑，不能寫成通過）：`SWindow::Resize` 的參數是螢幕客戶區像素。同一輪 responsive log 的 windowDpi 是 1.750，800 螢幕像素的 local 是 457.1。因此 1000 螢幕像素大約只有 571 local。獨立設定視窗的 layout scale 是 OS DPI（>=1），`FOmHudViewportSpace` 的 Allotted 用 local，不是要求的 Resize 像素。原本 `ClientSize(1180, 900)` 會先乘 DPI，螢幕客戶區約 2065×1575、local 約 1180；1180 螢幕像素並不是那個視窗。測試改為先量開放視窗與 360 螢幕像素，再在兩者之間用實際 `ReadStacked` 找最窄的並排寬，並記錄 screen／local／abs／panel／child type。並排寬上仍走 production spin `SetValue(ScaleUpper, true)`、同一面板寬、改鍵、Escape、關閉，再由 owner 把倍率設回 1。結束時還原量到的原始客戶區，並要求欄位方向與倍率 1 時的那次量測相同，不再斷言 1180 螢幕像素必須並排。沒有寫入私人 cache，沒有跳過找不到的交叉。沒有 PASS。這次沒有觀察到 E587。

## E584：Primary 新增 leaf 指標比較混用 SharedRef／SharedPtr Get（2026-10-09）

實際 ordinary settings-physical-build-v1 出現 `OmActionOwnerTests.cpp:808` C2678。Primary 為 E582 加的精確 leaf 比較，將 `TSharedRef::Get()` 的物件參考直接與 `TSharedPtr::Get()` 的指標比較，這是 Primary 自己引入的錯誤。已改為取參考位址 `&Path.Widgets.Last().Widget.Get()`；公開查找、同一SButton、指標 ContainsWidget 與真down/up檢查保留。原build失敗證據保留，尚未取得新build結果。防重犯：先辨識智慧指標種類，不能假設 Get() 都回傳指標。下一個實際錯誤E585。

## E583：UE FVector2D 建構子不能初始化 constexpr 靜態欄位（2026-10-09）

實際 ordinary settings-physical-build-v1（UBT44.80秒、OtherCompilationError、wrapper exit1）在 `OmHudViewportSpace.h:118/119` C2131。新 SettingsMax／SettingsFloor 的 FVector2D 建構子不是 constexpr。Primary 改為 `inline static const FVector2D`，尺寸與純量constexpr保持不變。不是 FailedDueToEngineChange，因此不refresh Engine。原build／UBT log完整保留；修復後用新run-id跑ordinary no-cache gate。詳細見 `.ai-collab/settings-physical-build-v1-errors.md`。

## E582：新增設定回歸再次把 SharedRef 傳給 ContainsWidget（2026-10-09）

Primary 在 E580/E581 repair terminal 後發現新 `OmActionOwnerTests.cpp` HitClick 呼叫 `ContainsWidget(Widget.ToSharedRef())`；已直接核對安裝引擎 `SlateCore/Public/Layout/WidgetPath.h:73`，參數是 `const SWidget*`。這重複 E572 的 API 錯誤。Primary 在建置前改為 `Widget.Get()` 並要求公開 hit path 的 leaf 為同一個實際 SButton。沒有宣稱曾跑出編譯失敗。防重犯：新增 helper 也必須核對既有錯誤紀錄與安裝版簽名，不能從 SharedRef 對稱推測 API。下一個實際錯誤E583。

E581 Primary policy補正：worker repair 的 one-pixel 分支使0.5倍率下限為9，且倍率超過1.8時由18降17。已在建置前改為單調的 `clamp(10*TextScale,10,18)`，保持既有字體大小與上限不變；端點繪製測試要求10／18，未放寬容差。`.ai-collab/settings-cache-repair-primary-review-v1.md` 為最新修正依據，worker report 的17下限是歷史內容。

## E581：設定字體宣告下限與支援倍率上限矛盾（2026-10-09）

Primary 在 settings source review 實際發現：新增 `FOmHudReadableText::PhysicalBodyFloorEm` 回傳 `10 * TextScale`，`BodyPxMax` 卻是18；正式 profile `ScaleUpper` 是2。在正規化 render scale 約1時，倍率2的實際18無法滿足宣告20。新增測試只含1／1.25，未覆蓋正式上限。尚未編譯／跑測試，這是來源契約發現，不能寫成 runtime failure。修復須明確統一可達字體尺寸與下限策略，保留既有HUD行為或提出必要變動依據，實際驗證支援倍率端點與繪製尺寸；不能跳過上限或只驗常數。完整 review 見 `.ai-collab/settings-physical-primary-review-v1.md`。下一個實際錯誤E582。

修正（source only，測試、建置都還沒跑，不能寫成通過）：HUD 的 body／title／slot 上限沒有改，倍率 2 的 slate body 仍是 18，不是 `round(11 * 2)` 的 22。`PhysicalBodyFloorEm` 只在 `10 * TextScale` 落在 slate 尺寸下方 1px 以內時回傳那個比例值；差距更大，或比例值高過 capped slate 尺寸時，下限改成 slate 尺寸減 1。倍率 2 的下限是 17，不再宣告 20。`Om.Runtime.ResponsiveHudLayout` 在 800×600 與 400×300 繪製 0.5、1、1.25、2，並在 800×600 加上 0.954／0.955 的捨入邊界與 1.59 的 cap 前一階。狀態文字必須真的畫出來並通過這個可達下限。80×60 仍沒有設定文字可讀性承諾。沒有 PASS。下一個實際錯誤編號仍是 E582，要等真實發現。這次沒有製造 E582。

## E580：頁面內改文字尺寸提前更新快取，阻止同寬欄位重排（2026-10-09）

Primary 在 settings source review 發現 `ApplyInterface` callback 將 `BuiltBodyPx` 設成新的字體值後直接 `RefreshRows`，卻未重算 `bStackBindings`。後續同寬 `SyncPresentation` 因字體與寬度快取相同而返回，舊欄位配置可能一直保留至視窗寬度改變。新 automation preview 與外部 profile change 有不同失效路徑，不能證明頁面內 callback 正確。尚未編譯／跑測試；要求統一版面失效／量測更新，新增真正同視窗 production callback 回歸，保留穩定 frame 不掃 catalog 與原操作。source 尚未接受，失敗發現與修復報告保留。

修正（source only，測試、建置都還沒跑，不能寫成通過）：快取鍵只有 body 像素與內容寬。兩者沒變就不掃 catalog、不重建列。`PersistInterface` 在 spin box 的 `CommitValue` 裡只把這兩個鍵清成未量測，不寫入新的 body、也不在 callback 裡 `RefreshRows`。下一次 `ApplyMeasuredLayout`（`NativeTick` 的 `SyncPresentation`，以及已有幾何的 preview）才一起重算堆疊、寫入兩個鍵並重建。外部 `TrySetInterfaceSettings` 不碰快取，下一個 tick 因 body 鍵不同而走同一條量測。`Om.Runtime.ControlRuntimePersistence` 在既有 live owner 的設定視窗裡，先找到倍率 1 並排的客戶寬，再用文字縮放 spin 的 `SetValue(..., true)` 收到支援上限，tick 後同一面板寬必須改成直向欄，接著仍能改鍵、取消與點到關閉 `SButton`；再由 owner 把倍率設回 1，同一寬度回到並排，最後把視窗還原 1180×900。沒有改 profile 版本，沒有 PASS。下一個實際錯誤編號仍是 E582，要等真實發現。這次沒有製造 E582。

## E579：item-card-dead 沿用零冷卻死亡預期，但前一階冷卻仍在（2026-10-09）

`owned-item-cards-viewport-v2` 實際停在 `item-card-dead`。`owned-item-cards-viewport-v2.ue.log` 的 `OmOwnedViewportIpcTests.cpp(5834)` 寫 `item HUD did not match the owner rows: kind=8 texts=1 Guard Charm 4.0s [1] | 2 Training Flare [2] | 3 Training Hex [3] | 4 Sword passive [5] | 5 Stride Charm [6] | 6 Mana Charm [7]`。`cleanup.json` reason 是 `shutdown_file`，`listener_released` true，`authority` false。workflow 的 publisher、editor、profile cleanup 都成功。native report `catalog_compatible` true，`authority_ack` false，`success` false。沒有 `error.json`。publisher 的 OS 5 沒有再出現。點擊還沒走到，因為 HUD 比對先失敗。native-v4 的 10 Success／0 warnings、Rust reader 16 與 client all-targets、build-v5 普通 nocache／refresh false 7.88s 是這次任務保留的既有結果，這輪沒有重跑。下一個實際錯誤 E580。

`MatchItemHud` 種類 8 要求就緒 catalog，而且每一列冷卻都是 0、沒有 ready。新表是 `item-card-cooldown`（`owner_item_cooldown`，槽 0 剩餘 4）接著 `item-card-dead`（`source_death`）。`source_death` 只把來源設成不可攻擊，不改 item contract。實際 HUD 因此留下 Guard Charm 4.0s，其餘主動列沒有 ready，被動列仍是 passive。這是預期用錯，不是把冷卻清成 0 的 production 缺陷。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`item-card-dead` 的 Hud 從 8 改成 10。種類 8 與 `item-dead` 的零冷卻死亡檢查沒改。種類 10 仍用就緒 catalog 5／10／11／1／6／7，槽 0 的 `CooldownRemainingSeconds` 用既有 0.05 容差對 4，文字含 `4.0` 且不含 ready；其餘五槽冷卻對 0；被動槽是 `ActiveKind::None`、停用、文字含 passive；六列 `bEnabled` 為假、`IsItemCardEnabled` 為假、`OmItemCard` 的 `SButton` 停用。失敗字串在原 texts 後面加上六個冷卻與六個 row enabled。style 67 的停用殼、公開點擊、指令不變、零 wire 留在 substep 670／671，670 另外要求槽 0 文字是 4.0 且不是 ready。沒有改 production、Rust、native 測試、清單 274／119／94 或 frame budget。沒有 PASS。下一個實際錯誤編號仍是 E580，要等真實發現。這次沒有製造 E580。

## E578：control reader 註解混用 poll 毫秒與 UE frame 預算（2026-10-09）

Primary 在 Rust 驗證前發現 E577 source/report 把 300×50ms（名義15秒）說成落在 UE 的300-frame ACK預算內，單位不同，不能這樣推論；select處理input也可能延後poll。已更正來源註解：限制是連續300次poll，不保證15秒內耗盡，UE獨立frame timeout可能先到，既有process／lifetime watchdog仍有效。移除一項只重述常數、沒有驗證時序的新增測試；實際persistent denial耗盡與狀態不變測試保留。沒有改runtime上限、沒有放寬任何timeout或結果。E577下方原writer說明為歷史宣告，以此更正為準。這是實際來源／說明錯誤，不是新runtime失敗；尚未跑Rust，下一個實際錯誤E579。

## E577：renderer fixture 讀 control.json 時把 Windows 存取被拒當成 publisher 失敗（2026-10-09）

`owned-item-cards-viewport-v1` 實際終止。證據在 `omfue/Saved/Automation/OwnedViewportIpc/owned-item-cards-viewport-v1/`。`error.json` 的 message 是 `ipc: 存取被拒。 (os error 5)`。`cleanup.json` 的 reason 是 `control_poll_failed`，`listener_released` 為 true，`authority` 為 false。`owned-item-cards-viewport-v1.ue.log` 在 `OmOwnedViewportIpcTests.cpp(2319)` 寫 `ability-cast-ready-5 control ack did not reach sequence 28`。workflow 的 publisher、editor、profile cleanup 都成功，ue-report `failed` 是 1。catalog 相容，沒有 authority ACK。這不是卡片斷言，也不是編譯失敗。native-v3 的 10 Success／0 warnings 與 build-v4 普通 nocache 57.08s 仍是既有結果。metadata 與 read 的精確交錯沒有被擷取。下一個實際錯誤 E578。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`poll_control_file` 不再先 `metadata` 再 `read`。`read_control_file` 只開一個 handle，最多讀 `MAX_CONTROL_BYTES + 1`（4097）個位元組；超過就記 `oversized:4097`，不把整個檔案讀進來。Windows raw 5／32／33 是連續暫態，第 `MAX_CONTROL_READ_TRANSIENTS`（300）次才失敗。300 次既有 50ms poll 是 15s，落在 UE `kAckFrames` 300、Lua `OPERATION_MS` 860000、fixture lifetime 900s 裡面。這三個 watchdog 沒有加大。NotFound（raw 2，或沒有 raw code 的 NotFound）不計入，啟動缺檔與替換空窗仍等待，而且會把暫態計數清掉。其他 OS code，包含 raw 3 與沒有 raw code 的 PermissionDenied，立刻失敗。Unix 的 raw 5 不是這組暫態。同一次 open／read 裡的 `Interrupted` 先重試 8 次、不睡眠；仍然中斷才算一次 poll。失敗字串帶 operation、path、`raw_os_error`，以及 `transient_streak`（耗盡是 300，永久錯誤的第一次是 0）。暫態、缺檔、永久失敗都不改 token、control sequence、world、ACK。完整位元組仍走原來的 token、`apply_control_bytes` 與拒絕 ACK。`write_atomic` 與 C++ `WriteControl` 沒改。沒有 PASS。274／119／94 與既有 253／115／80 前綴沒動。下一個實際錯誤編號仍是 E578，要等真實發現。這次沒有製造 E578。

## E576：卡片 preview 與焦點追蹤呼叫了 private IsKeyboardBlocked（2026-10-09）

E575 寫完後、建置前的 source review。`OmPlayerActionOwnerComponent.h` 公開區從 63 行開始，private 從 221 行開始，friend 只有 `FOmAttackMoveAimProcessor` 與 `SOmAttackMoveAimOverlay`。`IsKeyboardBlocked` 在 320 行，是 private。新的 `OmCommandBarWidget.cpp` 召喚師與道具 preview，以及 `OmActionOwnerTests.cpp` 的焦點 `AddInfo`，都從外面呼叫它。沒有 friend，也沒有公開包裝。這是源碼 API 問題，還沒有對這份源碼跑編譯或測試，不能寫成編譯失敗。下一個實際錯誤 E577。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：公開 `IsInterfaceInputBlocked() const`，只轉呼既有 private `IsKeyboardBlocked()` 的預設 playing 政策。兩個 preview 與焦點追蹤改用這個查詢。private 實作、friend、其他內部呼叫、拒絕字串與 preview 時機都沒改。沒有把內部全公開，沒有測試 friend 或 private macro，沒有複製守衛，沒有新的可變旗標。沒有 PASS。274／119／94 與既有 253／115／80 前綴沒動。下一個實際錯誤編號仍是 E577，要等真實發現。這次沒有製造 E577。

## E575：文字焦點的道具卡在滑鼠放開時才看鍵盤封鎖（2026-10-09）

`owned-item-cards-native-v2` process exit 0，實際 9 Success／1 Fail／0 warnings。`Om.Editor.ResponsiveHudLayout` 已 Success。普通 build-v3 nocache PASS，UBT 22.93s。`Om.Runtime.OwnedItemCards` 通過啟用 self／point 與停用 passive／empty／cooling／dead，在當時的 `OmActionOwnerTests.cpp:4236` 失敗：`Text focus blocks the card` 期望 `Keyboard input is blocked`，實際 `Cast request was rejected`。點擊本身已被消費。沒有 viewport。下一個實際錯誤 E576。

根因不在 `SetUserFocus` 的回傳值。已安裝 `SlateApplication.cpp` 的 `RoutePointerDownEvent` 先把 `OnMouseButtonDown` 送給葉子，返回之後若處理者沒有自己改焦點，就 `SetUserFocus(..., EFocusCause::Mouse)` 到路徑上最深的 `SupportsKeyboardFocus` widget。道具卡 `IsFocusable(false)`，遊戲 `SViewport::SupportsKeyboardFocus` 為 true，所以按下之後焦點離開 `SEditableText`。`SButton` 預設 `DownAndUp`，`OnClicked` 在放開時才跑 `RequestItemCard`。那時 `IsKeyboardBlocked` 看的是 viewport，不是文字框，於是同一條 `ActivateItem` 寫下 `Cast request was rejected`。`PrepareWindow` 的 `Tick`／`ForceRedrawWindow` 本身沒有 `SetUserFocus`。`IsKeyboardBlocked` 與測試用的是同一個 `GetUserIndexForPlatformUser`。這不是放寬預期錯誤能修的，也不是停用殼或 400×300 的回歸。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`SOmSummonerCardHit::OnPreviewMouseButtonDown` 在左鍵或觸控、且公開路由尚未改焦點時呼叫卡上的 preview。只有 `IsKeyboardBlocked` 為真才 `RequestItemCard`／`RequestSummonerCard` 並回 Handled，讓後面的 `OnClicked` 不再送出第二筆指令。沒封鎖就回 Unhandled，原本的放開點擊、停用殼、右鍵都還在。道具卡沿用原來的 HUD source guard；召喚師卡沿用原來的 `RequestSummonerCard`。沒有 `SetEnabled`、沒有直接 `OnClicked`、沒有 `GeneratePathToWidgetUnchecked`、沒有把焦點設回文字框、沒有用 modal flag 代替文字焦點。`OwnedItemCards` 在 `SetUserFocus` 之後要求同一個 `SEditableText` 與同一個文字視窗，並在 prepare 前、prepare 後、move、down、up 用 `AddInfo` 記下該 user 的焦點型別、是不是這顆文字、是不是這個視窗、鍵盤焦點型別，以及當下 `IsKeyboardBlocked`。預期錯誤仍是 `Keyboard input is blocked`。能力卡不是按鈕，沒有這條點擊。`Om.Runtime.SummonerOwnerCards` 與 viewport 的啟用卡點擊共用這層 preview，但只有真正封鎖時才改道；這次沒跑它們。沒有 PASS。274／119／94 與既有 253／115／80 前綴沒動。下一個實際錯誤編號仍是 E576，要等真實發現。這次沒有製造 E576。

## E573：停用道具卡的命中證明仍要求最後一個 widget 是 SButton（2026-10-09）

`owned-item-cards-native-v1` process exit 0，實際報告 8 Success／2 Fail／0 warnings，workflow 正確判失敗。普通 build-v2 nocache 已過（refresh=false，UBT 14.83s）。`Om.Runtime.OwnedItemCards` 在 `OmActionOwnerTests.cpp:4115`：`Expected 'Disabled passive card consumes the click' to be true.` 同一輪啟用的 self／point 與 detached guard 已走到斷言。已安裝 `Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp` 在 `!bIgnoreEnabledStatus` 時找到第一個 disabled widget，`Path.RemoveAt` 連同該 widget 與其子孫一起拿掉。`LocateWindowUnderMouse` 因此不含停用的 `SButton`，葉子是父層。`SOmSummonerCardHit::OnMouseButtonDown`／`OnMouseButtonUp` 已回 `FReply::Handled()`。失敗發生在公開事件送出之前，不能當成世界指令外洩。Viewport `LocateOwnedItemCard` 對 style 600 的被動格與 style 640／650／660／670 用了同一個 SButton 葉子假設。沒有 viewport 執行。下一個實際錯誤 E574。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：production 命中殼沒改。啟用卡仍走原來的 `Click`／`LocateOwnedItemCard`：同一顆 `SButton`、`ContainsWidget`、最後一個 widget 是 `SButton`、公開 down／up 兩個 `bool` 都為 true。停用卡改走 `ClickDisabled` 與 `LocateOwnedDisabledItemCard`。證據是按鈕本身 `IsEnabled` 為 false、`Advanced_GetPaintParentWidget` 的型別是 `SNew` 寫下的 `SOmSummonerCardHit`、公開 path 的最後一個 widget 指標就是這個殼、path 不含那顆停用按鈕，然後才送公開 down／up，兩個 `bool` 都要為 true。身分不合時 `AddInfo` 寫下 leaf、shield、containsButton；viewport 逾時的失敗字串同樣寫這三項。沒有 `SetEnabled(true)`、沒有直接 `OnClicked`、沒有 `GeneratePathToWidgetUnchecked`、沒有改路線。Owner 測試的 passive／empty／cooling／dead 四個消費斷言改用停用證明；text／modal／retired 仍是啟用卡。Viewport 只有 `item-card-map` 的被動格與 `item-card-passive`／`item-card-empty`／`item-card-cooldown`／`item-card-dead` 改走停用定位，按下仍用原來的 `PressLocatedItemCard`。啟用的 self／point／unit／right／held 定位沒動。沒有 PASS。274／119／94 與既有 253／115／80 前綴沒動。下一個實際錯誤編號仍是 E574，要等這一輪的版面失敗處理完。這次沒有製造新的編號。

## E574：緊湊道具卡把已寫在狀態列裡的熱鍵與冷卻又畫成一列（2026-10-09）

同一輪 `Om.Editor.ResponsiveHudLayout` 在 `OmResponsiveHudTests.cpp:631` 失敗：`logical 400x300 for 800x600 at DPI 2, not process scale layout fits`。最後一次 stacked 嘗試 outer=80 stack=1 combatW=384 inset=96 limit=196 combat=(375,246) mini=(80,80) clause=combatH。bands hero=49 ability=65 economy=22 items=88 stats=154x36。compose itemBeside=6。戰鬥欄高 246 超過上限 196。字型地板、minimap、stats 與 `layout fits` 斷言都要留。下一個實際錯誤 E575。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：沒有改 `IsHudLayoutFitted`、支援尺寸、字型地板、minimap、stats、群組縮放或 DPI。`RefreshItemSlotPresentation` 的狀態列已經包含槽號、名稱、passive／ready／不可用、冷卻秒數與 `[UseKey]`。緊湊時 `OmItemHotkey` 列與 `OmItemCooldown` 列的 visibility lambda 讀 `bCompactHud` 並收成 Collapsed，所以這兩列不再佔測量高度；`GetText` 仍回熱鍵與冷卻。名稱列本來就在緊湊時收起。寬度至少 800 且 roomy 放得下時，這兩列維持分開的 chip。組合仍只在 layout signature 變更時跑，沒有每幀重建。`LastCompactCompositionNote` 多了 `itemLines=1`（緊湊只留狀態列）或 `itemLines=2`（roomy 分開讀數），既有 fit log 會帶出這段，測試沒有改讀它。沒有 PASS，也沒有重測 400×300 是否已經落到 196 以內。下一個實際錯誤編號仍是 E575，要等真實發現。這次沒有製造 E575。

## E572：新道具卡測試用錯已安裝的 Slate 公開簽章（2026-10-09）

`owned-item-cards-build-v1` 普通 nocache 實際 UBT 失敗，不是 EngineChange。`OmActionOwnerTests.cpp:3976` C2664：`FWidgetPath::ContainsWidget` 要 `const SWidget*`，新測試傳了 `TSharedRef`。同函式 3982–3983 C2248：公開 `ProcessMouseButtonDownEvent`／`ProcessMouseButtonUpEvent` 回傳 `bool`，新測試把回傳值存成 `FReply`。已安裝 `Engine/Source/Runtime/Slate/Public/Framework/Application/SlateApplication.h:1302/1310` 與 `SlateCore/Public/Layout/WidgetPath.h:73` 就是這兩個簽章。`OmOwnedViewportIpcTests.cpp` 的 `LocateOwnedItemCard`／`PressLocatedItemCard` 有同樣兩個假設。沒有 native 或 viewport 通過。下一個實際錯誤 E573。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：兩處 `ContainsWidget` 改傳同一個 `TSharedPtr` 的 `Get()`，也就是 `const SWidget*`。按下與放開改收 `bool`，兩個都為 true 才算處理。仍用 `LocateWindowUnderMouse`、同一顆 `SButton`、最後一個 widget 型別，以及公開滑鼠路由。沒有改成直接 `OnClicked`、`GeneratePathToWidgetUnchecked` 或選定路線。Production、274／119／94、既有 253／115／80 前綴、lifecycle／modifier 與 frame budget 沒動。沒有 PASS。下一個實際錯誤編號仍是 E573，要等真實發現。這次沒有製造 E573。

## E571：死亡／替換／移除案例沿用來源變更前 HP／pose baseline（2026-10-09）

`owned-items-recall-viewport-v8` TERMINAL FAIL236cases／103wires／69stages，last item2-held-dead；`moved the owned hero locally: distance=0.000 hp=0.000 was=550.000`。catalogcompatible、editor／publisher／profilecleanup與listenerreleased全ok，無authorityACK。E570 budget已越過，Stop／Hold修飾鍵已完成，253仍未全過。

Source新held retirement Style40在401 CaptureSummonerPose存活HP550，403寫source_death，404ACK、405WaitSettled驗死亡、406HUD死亡，finish卻對原baseline無條件SummonerPoseUnchanged（同時比較HP），因此把刻意注入HP0當本地輸入效果。Style38／39 replacement/removal走相同baseline流程亦須同輪審查。不可直接刪pose/HP斷言或任意重新baseline掩蓋輸入效果；應以已驗證的來源generation／生命階段及fixture轉換政策分別確認舊來源退休、預期HP／位置／identity變更、release後零wire／無preview／新狀態不再額外改動。下一實際錯誤E572。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：沒有清 `bSummonerPoseHeld`，也沒有在階段後重抓當下 pose 蓋掉按下時的位置與 HP。`SummonerPoseUnchanged` 本體沒改。Style 38／39／40 在 `WriteControl` 前仍呼叫它，證明按下到注入階段前沒有移動、沒有掉血。Style 49 的 Stop wire 仍由 `PollPlainWire` 走同一個檢查；該檢查會清 flag，所以寫 `source_death` 前改比對按下時記住的 id 11、generation、位置與 HP，避免把已經清掉的 flag 當成「沒有姿態可證」。三條來源轉換都在 `WaitSettled` 成功後、KeyUp 前，用當下 Probe 與 actor 封存：死亡是同一個 hero 11、同一個 generation、HP≤0、位置與按下時相差不超過 0.01；替換是舊 actor 已不在、hero 12、generation 等於 ACK 寫入的 `ViewEpoch` 且大於按下時，新位置只作放開後不得再動的觀測，不要求等於舊位置；移除是 HUD 沒有存活來源且舊 actor 不在，不讀孤兒位置、不對它做 pose。放開後原 `PollExpect(None)` 仍證明零 wire，再要求封存狀態沒變、沒有 cast preview、沒有 attack aim。存活 HP 落到 0 是 fixture `source_death`，不是輸入副作用，也不是 authority 結果。沒有故意改狀態的案例仍走原來的 `SummonerPoseUnchanged`。同輪查過的其他新案例沒有這條「先抓活著的 pose、再注入 source_death／replacement／removal」：`item2-stale` 是 living `owner_item_replaced`，召喚師死亡／替換／移除與 ability cancel 沒有把 pose 留過階段，`recall-dead-label` 沒有抓 pose。沒有改計數、28145、production、Rust、Lua、native。沒有 PASS。253／115／80 仍是宣告。下一個實際錯誤編號仍是 E572，要等真實發現。這次沒有製造 E572。

## E570：整批 budget 將 latent Update 次數誤作引擎幀並不足以涵蓋新增觀察（2026-10-09）

`owned-items-recall-viewport-v7` TERMINAL FAIL：231cases／101wires／68stages，last hold-modified，`owned viewport IPC exceeded the engine frame budget`。stop-modified已完成所有修飾鍵held／quiet／actualall-up，未多送Stop；hold-modified尚在執行。cleanup/editor/publisher/profile全ok、listenerreleased/catalogcompatible/noauthorityACK。253全批仍未通過。

Source FOmOwnedViewportIpcCommand::Update 在 `PumpFrame()`之前執行 `++Frames > 20000`，而PumpFrame會在GFrameCounter未移動時returnfalse。因此counter實際算latent Update呼叫，不是宣稱的新引擎幀。新的分別三modifier quiet觀察還增加必要幀；應只計實際成功PumpFrame，並從實際案例／觀察契約推導有限且可解釋的整批budget，保留個別ready／ACK／intent／quiet逾時與外部watchdog。不能刪除守門或任意倍增魔法數來通過。Primary一條Get-Content -Tail讀到單行native.json整份造成輸出截斷；後續只用結構化摘要，不作完整讀取證據。下一個實際錯誤E571。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`Update` 先 `PumpFrame`。GFrameCounter 沒動就 return false，不加計數。只有成功泵出的新引擎幀才 `++PumpedFrames`，超過 `kOwnedViewportFrameBudget`（28145）才失敗。失敗字串保留 engine frame budget，並寫上 pumped、budget、case、phase、substep、caseBudget。預算是 `2*kReadyFrames + 253*(kAbsentFrames+5) + 27*kAbsentFrames + kAckFrames + kIntentFrames`。253 來自同一組 phase count（前綴 47、ability 65、minimap 9、summoner 37、display 7、items 88），各表 static_assert 對這組常數。27 是關閉用 PollExpect 之外、會跑滿 kAbsentFrames 的證明，含 stop-modified 與 hold-modified 各三個 modifier quiet。個別 600／300／180／90 與 Lua 860000ms watchdog 沒改。沒有刪守門，沒有把 20000 改成無來源的大常數。widget 邊界與 focus 守衛沒改。沒有 PASS。253／115／80 仍是宣告。下一個實際錯誤編號仍是 E571，要等真實發現。這次沒有製造 E571。

## E569：測試修正以最優先 preprocessor 強制轉送，繞過真實焦點路由（2026-10-09）

E568修正的獨立source review發現 `FOmOwnedChordRouteProcessor` 以Overlay priority0攔截所有keydown/up，直接呼叫選定FSceneViewport::OnKeyDown／OnKeyUp後returntrue，不核對Slateuser／實際focusedwidget／window／key，並在遊戲production preprocessors前消費事件。因此即使真正focus錯誤、其他preprocessor應攔截，也能送達選定viewport；不能當作真實publicSlatefocus／操作入口證據。此為實際審查發現，未跑新build／viewport，不宣稱runtime失敗。

E568編輯器Alt+S Simulate冒泡根因有source與實際PIE world證據；保留這項發現，但應在實際focused fixture viewport的widget鍵盤事件邊界，先走正常SViewport→SceneViewport→PlayerInput及原production preprocessors，再將該fixture scope中未處理的reply收住，阻止editor unhandled fallback。不得用priority0選定route或直接owner/input注入。保留真實focus/user/window、按下/放開、零wire與all-up斷言。下一個實際錯誤E570。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`FOmOwnedChordRouteProcessor` 整段移除。繪製中的 viewport 改為 `SOmOwnedFixtureViewport`，繼承 `SViewport`。`OnKeyDown`／`OnKeyUp` 先呼叫一次父類，父類才經正常 widget dispatch 進 `FSceneViewport`。Style 42 範圍內若父類 reply 未處理，才改成 Handled，擋下編輯器 unhandled fallback。閘門關掉時原樣交回父類 reply。沒有 input preprocessor，沒有直接 `OnKeyDown`／`OnKeyUp`，production preprocessor 順序沒變。按下前、修飾鍵按下後、指令鍵按下後、quiet 每一拍、放開前、指令鍵放開後、修飾鍵放開後、以及下一 tick，都要求同一個 fixture viewport、同一個 Slate user、同一個 window。焦點不對就失敗，不重新 SetUserFocus，也不再送 key。`KeyDown`／`KeyUp` 本體沒改。all-up、零 wire、沒有 preview、pose 仍在。沒有 PASS。253／115／80 仍是宣告。下一個實際錯誤編號仍是 E570，要等真實發現。這次沒有製造 E570。

## E568：Alt modified Stop 放開後 PlayerInput 仍呈按住（2026-10-09）

`owned-items-recall-viewport-v6` TERMINAL FAIL：230cases／101wires／68stages，last stop-modified；`left a modifier down after Alt (shift=0 ctrl=0 alt=1 cmd=0 command=1)`。editor／publisher／profile cleanup全ok，listener released，catalog compatible，無authority ACK。E567額外Stop已消除，Shift／Ctrl的實際held／quiet／released步驟已經走過，Alt held quiet亦無新wire；但ReleaseModifiedChord的公開keyup後下一步PlayerInput仍down。不可刪掉all-up檢查或偽造FlushPressedKeys來通過。先核對Slate真實focus／keyup handled與preprocessor／viewport路徑，分辨路由消費、同frame尚未process或production事件問題，修共用實際原因。下一個實際錯誤E569。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：根因在 fixture 的 Slate 路由，不在 production，也不是下一 tick 沒處理。`FSceneViewport::OnKeyDown` 在 `UPlayerInput::InputKey` 對 `IE_Pressed` 回傳 false（不是 action mapping）時把事件當 Unhandled。Unhandled 會進編輯器 `FMainFrameActionCallbacks::OnUnhandledKeyDownEvent`，其中 `FPlayWorldCommands` 的 Alt+S 是 Simulate。v6 log 在失敗同一個時間點出現 `Match State Changed from EnteringMap to WaitingToStart`、`OM_BRIDGE_FIRST_TICK`，結束時 `BeginTearingDown for /Game/Map/UEDPIE_0_Main`。PIE 拿走焦點後，`KeyUp` 沒有 `SetUserFocus`，S 與 LeftAlt 的 key-up 沒進這個 fixture 的 PlayerInput，所以 alt=1 且 command=1。Shift／Ctrl 沒有這條 Simulate 和弦，同一拍的 key-up 有清掉，證明 World.Tick 有處理已送達的放開。`FOmAttackMoveAimProcessor::HandleKeyUpEvent` 只在 cast hold 開啟時吃 key-up；quiet 已要求沒有 preview。all-up 檢查保留，沒有加等待。Style 42 的 Stop／Hold 與 Shift／Ctrl／Alt 改走公開 `KeyDown`／`KeyUp`，但先掛 Overlay preprocessor：把事件送到 fixture `FSceneViewport::OnKeyDown`／`OnKeyUp` 並回傳 handled，編輯器指令表看不到。Style 5／6／7 是物品鍵，v5 的 Alt 拒絕已通過；Style 47 不是 Alt+S。沒有改 `KeyDown`／`KeyUp` 本體、production、計數或 native。沒有 PASS。253／115／80 仍是宣告。下一個實際錯誤編號仍是 E569，要等真實發現。這次沒有製造 E569。

## E567：修飾鍵零指令案例實際多送三筆 Stop（2026-10-09）

`owned-items-recall-viewport-v5` TERMINAL FAIL：230cases／104wires／68stages，last `stop-modified`，`emitted an extra wire intent`。實際末尾ordinal102／103／104皆Stop，editor／publisher／profile cleanup全ok，listener released，catalog compatible，無authority ACK。E566兩個Alt拒絕已通過，全253仍未通過。

新Style42在同一latent tick內依次完成Shift／Ctrl／Alt的modifier down、key down/up、modifier up三組tap。production BindSingleHeroCommand在PlayerInput執行delegate時以MatchesExactModifiers讀取實際PlayerInput狀態。需先核對Engine事件處理時序，區分同frame測試清空modifier後才處理與真正production缺陷；不能直接忽略三筆wire、改成接受或偽造狀態。測試需讓每個修飾鍵在實際PlayerInput處理key期間維持，分幀驗三種modifier皆無wire且放開後仍可恢復正常Stop／Hold。若確認生產入口確有契約問題，修共用事件路徑並增受影響native驗證。下一個實際錯誤E568。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：根因在 fixture，不在 production。`UPlayerInput::InputKey` 只累積事件；`ProcessInputStack` 在下一次 `World.Tick` 才 `EvaluateKeyMapState`。`ProcessNonAxesKeys` 用按下減放開：同一批裡兩者都有時 `bDown` 維持先前的 false，但 `IE_Pressed` 仍會派發。`MatchesExactModifiers` 此時讀到修飾鍵沒按下，無修飾的 Stop 和弦成立，所以三筆 tap 變成 ordinal 102–104。Style 42 改為每個 Shift／Ctrl／Alt 分開：公開 `KeyDown` 按下修飾鍵與指令鍵後，跨 tick 用 `UPlayerInput` 斷言只有該修飾鍵與指令鍵為 down，安靜 `kAbsentFrames` 內零 wire、無 preview、pose 不變，再 `KeyUp` 指令鍵然後修飾鍵，下一 tick 確認全部放開且仍零 wire，才進下一個修飾鍵。`stop-modified` 與 `hold-modified` 都走這條路。Style 5／6／7 本來就跨觀察幀按住修飾鍵；Style 47 的 Shift 包住的是已經在前一 tick 處理過的放開，不是同一缺陷，沒有改。沒有改 `KeyDown`、`BindImmediateHero`、`MatchesExactModifiers`、計數或 native。沒有 PASS。253／115／80 仍是宣告。下一個實際錯誤編號仍是 E568，要等真實發現。這次沒有製造 E568。

## E566：Alt item 拒絕案例漏送 Alt 修飾鍵（2026-10-09）

實際 `owned-items-recall-viewport-v4` TERMINAL FAIL：196cases／88wires／58stages，last `alt-item2-refuses`；錯誤 `refusal was not Cast request was rejected (accepted=1 id=88 error=)`。editor／publisher／profile cleanup全ok，listener released，catalog compatible，無authority ACK。E565兩個文字焦點案例已通過，253／115／80全批仍未通過。

Source：Style6（alt-item2-refuses、alt-item3-refuses）映射Substep150，但150／160／170／280／300共用分支只在160送Shift，其他一律KeyDown(Key, ModNone())，151也只區別Shift放開。故測試並未送出它宣稱的Alt self-cast事件。應為Style6使用實際Alt down／modified key down／modified key up／Alt up，保留真正拒絕與零wire檢查；一次核對所有新items-phase修飾鍵分支，不能修改production迎合錯誤測試。下一個實際錯誤E567。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：Style 6 的 `alt-item2-refuses` 與 `alt-item3-refuses` 從共用的無修飾分支拆出。按下是 `KeyDown(LeftAlt, ModLeftAlt)` 再 `KeyDown(Key, ModLeftAlt)`，看到真正的 `Cast request was rejected` 之後才 `KeyUp(Key, ModLeftAlt)` 與 `KeyUp(LeftAlt, ModNone)`。完成仍要求零 wire、pose 不變、沒有 preview，且 `SeeRefusal` 必須讀到該拒絕；沒有偽造狀態。Style 5 的正向 Alt self、Style 7 的 Shift、Style 42 的 Shift／Ctrl／Alt、Style 47 的持鍵後 Shift 放開，以及 170／280／300 的無修飾按下都已核對，事件與案例相符，沒有改。`KeyDown`／`KeyUp`、生產 `HandleItemChord`／`ActivateItem`、計數、E563／E564／E565 都沒改。沒有 PASS。253／115／80 仍是宣告。下一個實際錯誤編號仍是 E567，要等真實發現。這次沒有製造 E567。

## E565：文字焦點案例的 KeyDown helper 搶回 viewport，並錯誤要求 owner 拒絕（2026-10-09）

`owned-items-recall-viewport-v3` TERMINAL FAIL：173cases／77wires／51stages，last stop-text，failure `stop-text refusal was not Keyboard input is blocked (accepted=1 id=77 error=)`；真正wire77為Stop。editor／publisher／profile cleanup全ok、listenerreleasedtrue／authorityfalse、catalogcompatibletrue。先前新的Stop／Hold primary/secondary七筆正向wire70–76已經通過讀取器；這不是intake種類缺漏或已證明的producerfocusguard問題。

Source根因：新items-phase Style17（stop-text與b-focus）Substep260先OpenTextWindow，接著KeyDown(Key)，但KeyDown1384–1391無條件Slate->SetUserFocus(UserIndex, ViewportWidget)再ProcessKeyDownEvent，故它自己離開文字框，送出合法Stop。Substep261再SeeRefusal要求owner寫Keyboardblocked亦不符合真實Slate消費：文字元件攔截時owner可能根本未呼叫，LastSubmit可保持原值。應新增/使用明確保留現有focus的公開按鍵入口，驗證焦點確實留在同一editable/window、按下/放開與quiet-log零wire、不開preview/無來源pose改變。既有165案例的KeyDown焦點政策不變。這兩個Style17不得假設owner一定收到key；若owner沒收到，不可偽造拒絕旗標或清LastSubmit當證據。保留modal／其他真正producer拒絕斷言，完成時要求實際focusguard觀察而非跳過檢查。

Actual普通build-v7 PASS15.58s及owned-intent-kind-native-v1 exactJsonlObservation1 PASS0warning已完成，組合44/4圖/Rust26/Lua31未改。253/115/80仍非PASS。源碼只有兩個新案例共用Style17；同輪修正stop-text與b-focus，不逐個盲試。下一個實際錯誤E566。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：Style 17 的 stop-text 與 b-focus 不再走會 `SetUserFocus` 回 viewport 的 `KeyDown`。新的公開路徑只呼叫 `ProcessKeyDownEvent`／`ProcessKeyUpEvent`。按下前、按下後、放開前、放開後必須是同一個 `AuxText` 與同一個 `AuxWindow`，並記下實際 focus path、viewport path 與 handled。既有 `KeyDown`／`KeyRepeat`／`KeyUp` 給接受前綴 165 的焦點政策不變。安靜視窗在 `CloseAux`／`RestoreGameFocus` 之前跑滿 `kAbsentFrames`，要求零新 wire、沒有 cast／attack-move preview、來源 pose 不變。`LastSubmit` 沒變就保持，不偽造 Keyboard blocked、不清 `LastSubmit`。狀態若真的變了，只有未接受、input id 0、且錯誤含 `Keyboard input is blocked` 才算真實拒絕。完成契約改為要求該 focus-guard 觀察與零 wire，不再無條件要求 `Spec.Refusal`。hold-modal、死亡、來源與其他拒絕，以及後段恢復正向案例仍用原檢查。items 階段裡 item2-held-focus、stop-text-recover、stop-refused-recover 與更早階段的 `OpenTextWindow` 不是先開文字再 `KeyDown`，沒有改。沒有改 case／wire／stage 數量、E563／E564、生產控制。沒有 PASS。253/115/80 仍是宣告。下一個實際錯誤編號仍是 E566，要等真實發現。這次沒有製造 E566。

## E564：新案例使用的四種指令未接上視窗全域讀取器（2026-10-09）

Primary 後續實際驗證：source修正後 ordinary `owned-items-recall-build-v7` PASS／refreshfalse／nocache，UBT15.58s；`owned-intent-kind-native-v1` exact1 `Om.Runtime.JsonlObservation` Success／0error／0warning／0notrun，process0，實際涵蓋共用new-kind shape正反與原E563held-reader／永久寫入失敗。組合44與4張設定視覺來源未改、明示保留。fresh `owned-items-recall-viewport-v3` 已啟動；尚未宣稱253/115/80 PASS，v2失敗保留。下一個實際錯誤仍E565。

`owned-items-recall-viewport-v2` TERMINAL FAIL：166 cases／70wires／51stages，last s-stop；`unexpected wire intent Stop` 在 OmOwnedViewportIpcTests.cpp975。前69條舊wire已存在，第70條是真正合法 `{kind:"Stop"}`，source11/epoch7、targetrecord21/epoch7、player1、authority_ackfalse。全域 LoadIntents 970–975 仍只接受 MoveTo／AttackMove／AttackTarget／AbilityCast／AbilityUpgrade／SummonerCast，未列 Stop／HoldPosition／Recall／ItemUse，故專用 PollPlainWire／PollItemWire尚未能驗該指令。這不是已證明的S鍵repeat問題；不修改生產Stop行為迎合讀取器。

v2 catalogcompatibletrue、native/workflowfalse；editor／publisher／profilecleanup全部ok，listenerreleasedtrue／authorityfalse。E563歷史修正已走到51stage而未重現appendfailure，但全批仍非253/115/80 PASS；沒有跑接受前綴記錄。源碼可見其它新3種類同樣缺口，應一次完整接上四類typed shape，不能先allowStop再逐個盲試。保留舊queued／catalog／target-kind／ability-registry等檢查，依實際Rust wire政策驗Stop無queue／Hold有falsequeue／Recall無queue、三者無point/target/ability/summoner/item別名；ItemUse slot0..5、實際catalog/item、null/explicit點存在性政策及沒有queue/ability/summoner別名。加入actual入口共用shape validator的native正反fixture，保留嚴格逐case預期與原165前綴。組合44＋JsonlObservation1、Rust26／Lua31保留；若共用fixtureheader改動，再重跑受影響JsonlObservation，完整驗收仍最後。下一個實際錯誤E565。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`ValidateOwnedWireIntent` 是 `ParseIntentLine`／`LoadIntents` 與 `Om.Runtime.JsonlObservation` 共用的 kind／shape。Stop、Recall 只接受沒有 queue、point、unit、ability、summoner、item 欄位的 body；HoldPosition 必須是 `queued:false` 且沒有其他 payload；ItemUse 接受 slot 0..5、正 catalog、非空 item id、整數 target 與 raw，explicit 兩側同為 null 或同為與 raw 相等的整數。null explicit 加原點 target 0 是 self，explicit 0／0 是 point，正 target 加 null explicit 是 unit。錯誤型別不算缺欄。未知 kind 失敗。舊 Move／Attack／Ability／Summoner 規則留在同一個函式，視覺 registry 仍在 LoadIntents。沒有改生產指令。沒有 PASS。下一個實際錯誤編號仍是 E565，要等真實發現。這次沒有製造 E565。

## E563：控制歷史 append 使用排他寫入，與即時觀察讀者不相容（2026-10-09）

Primary 後續實際驗證：`owned-items-recall-build-v6` ordinary nocache PASS／refreshfalse，UBT38.68s；`owned-history-sharing-native-v1` exact1 `Om.Runtime.JsonlObservation` Success／0error／0warning／0notrun，process0。包含真實 held-reader 追加與永久目的地失敗，並保留舊 JSONL 契約。未改的組合44項與4張設定視覺證據明示保留。fresh `owned-items-recall-viewport-v2` 已啟動，尚未宣稱253/115/80 PASS；v1失敗完整保留。下一個實際錯誤仍E564。

實際 `owned-items-recall-viewport-v1` TERMINAL FAIL：native122 cases／58wire／36stages，last summoner-ready；native與workflow failure `control history was not appended`，UE OmOwnedViewportIpcTests.cpp1774。sequence37 owner_summoner_ready 的合法 publisher ack已在 control.ack.json，但歷史只到36 owner_point_ready，沒有補造缺失紀錄。catalog compatibletrue／authority_ackfalse；editor、publisher、profile cleanups全ok，listener_releasedtrue、authorityfalse。未達宣告253/115/80或接受165前綴，不可接受這批視窗。

原生44 scope仍是實際42保留＋2重測，普通build-v5及4張設定視覺證據保留。來源觀察：AppendHistory 的 SaveArrayToFile 只有 FILEWRITE_Append，沒有 FILEWRITE_AllowRead；FileManagerGeneric 把 AllowRead轉為OpenWrite的bAllowRead，WindowsPlatformFile.cpp1641在false時ShareFlags=0（排他）。Primary在進行中用 Get-Content 讀過歷史檔，這可能是共享衝突的觸發者；本次沒有記下失敗當下OS error，故不宣稱量到error32。通用修正應讓持有允許寫入的正常讀者與單一歷史writer共存，補實際 held-reader file fixture；保留validated ACK原bytes、每request一次、newline、嚴格序號與真正永久寫入失敗，禁止吞掉失敗／跳過歷史／憑空ACK／盲目sleep重跑。Primary之後不再在Append期間直接開歷史／intent檔觀察，改用引擎log或終端完成產物。下一個實際錯誤E564。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`AppendOwnedHistoryRecord` 是 `AppendHistory` 與 `Om.Runtime.JsonlObservation` 共用的唯一 append。它把已驗證 bytes 原樣寫入，旗標是 `FILEWRITE_Append | FILEWRITE_AllowRead`，對應 Windows `OpenWrite` 的 `FILE_SHARE_READ`，因此 `FILEREAD_AllowWrite` 的既有讀者可以留著，第二個 writer 不共享。空紀錄在開檔前拒絕。`SaveArrayToFile` 回失敗就停，記下當下 platform last error，不重試、不把該數字說成 viewport-v1 的原因。`bAckCopied` 仍只在成功後設 true，每個 request 仍先清掉。Jsonl 測試用獨立暫存路徑：先寫第一筆，持有 AllowWrite 讀者再寫第二筆，關閉後核對剛好兩筆 newline bytes；父路徑是檔案的目的地必須失敗且不改已寫 bytes。沒有改控制、UI、authority。沒有 PASS。下一個實際錯誤編號仍是 E564，要等真實發現。這次沒有製造 E564。

## E562：原生測試假設 JSON 空白格式及舊道具拒絕順序（2026-10-09）

後續 Primary 獨立驗證：source-only修正後普通 `owned-items-recall-build-v5` PASS、refreshfalse／nocachetrue、UBT16.40s；`owned-items-recall-native-v2` 實際 exact2 Success／0warning／0failed／0notrun，process0。只重跑上述兩項，未改的42項成功明示保留自失敗 native-v1。`owned-items-recall-native-composed-v1` 為42保留＋2重測的組合證據，不宣稱一場新44項執行；原失敗workflow未改。4張實際設定 PNG 全變更且Primary逐張看過，comparison／visual-review保存。`owned-items-recall-viewport-v1` 正在獨立執行，未有完整253/115/80 PASS。下個實際錯誤仍 E563。

`owned-items-recall-native-v1` D3D12 process exit0，但 suite FAIL：實際 index exact44，42 Success、2 Fail、0 warning、0 notRun／inProcess。`ControlProfileSummonerCastModes` 2 errors：612要求 itemCastModes 只允許緊湊或單空白 header，實際 ExportJson 合法 pretty JSON 是冒號後 CRLF/tab；615印出該 header。需要結構化 JSON mutation／serialization，保留 unknown key9、queued mode、回滾與既有版本 migration，不更改 production export 格式迎合字串拼接。

`OwnedRecallItemHotkeys` 8 errors：4筆 item拒絕（ItemSlot1／4／6／1Self）各錯2個預期。fixture只 UpdateOwnerHeroState，沒有裝備inventory，正式 ActivateItem 先讀 configured player7 並拒絕 empty slot；舊 helper1621期待player0、1624期待 Runtime connection is unavailable。實際item player7／Owned item slot is empty 是現在共用目標施放入口的合法早期拒絕。Recall仍player0／runtime unavailable。測試應分開真實item metadata admission與Recall transport refusal，保留action、slot、label、source、無queue、零inputid、不接受、不開preview與legacy modifier隔離，不可倒退production gate或偽造連線。

普通 build-v4 refreshfalse／nocache PASS2.23s 已完成；ControlRuntimePersistence 的真實 item settings hit 是 Success，不能被此2項失敗抹掉。不可變 index／log／summary 與實際設定 PNG 保存在 `.ai-collab/.bridge-data/worker-pool-machine-v1/owned-items-recall-native-v1/`；本批沒有 viewport，253/115/80仍未驗證。僅派 source-only test repair，完成後重新 ordinary build 與兩項受影響測試，42項成功只在來源不變且明示重用時保留。下個實際錯誤 E563。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`ControlProfileSummonerCastModes` 不再用 header 字串替換。它反序列化實際 `ExportJson`，只改 `itemCastModes`，再用 pretty 與 condensed serializer 寫出未知 key 9 與 queued。兩份都必須和原 export 不同，且解析後真的帶有該變更；invalid data 與整份 rollback 保留，schema、version、Q、召喚師 D、bindings 數量在 mutation 中不變。`OwnedRecallItemHotkeys` 的四筆物品拒絕改為 player 7、`Owned item slot is empty`；Recall 仍是 player 0、`Runtime connection is unavailable`。動作、槽、label、Hotkey、NoTarget、不排隊、不接受、input id 0、不開 preview、六鍵與 Alt-self、legacy modifier 隔離都留著。沒有改 production serializer 或 `ActivateItem`。沒有 PASS。下一個實際錯誤編號仍是 E563，要等真實發現。這次沒有製造 E563。

## E561：普通建置確實觸發 FailedDueToEngineChange（2026-10-09）

`owned-items-recall-build-v2` 普通 nocache／NoEngineChanges 真實失敗，summary exit1、refresh=false；不可變 `UBT-Log.txt` 明確記錄 `CompilationResultException: FailedDueToEngineChange` 及最後 `Result: Failed (FailedDueToEngineChange)`。這次與 E560 的 FString 編譯錯誤不同，符合正式 engine refresh 條件。使用既有 Lua 建置入口執行 `owned-items-recall-refresh-v3 refresh no-cache`，不手改 Engine 來源、設定或 BuildId。refresh 成功後必須再跑新的普通 nocache gate，才可做 native／viewport；目前沒有新的建置 PASS，也沒有 native／viewport PASS。完整 v2 產物保留 `.ai-collab/owned-items-recall-build-v2/`。下一個實際錯誤編號 E562。

後續實際建置：`owned-items-recall-refresh-v3` exit0／refreshtrue／nocachetrue；build.log 記錄正式 refresh 31 actions、175.26s，內建普通 gate7.34s。再獨立 `owned-items-recall-build-v4` exit0／refreshfalse／nocachetrue，UBT Succeeded2.23s。兩者未跑 simulation。E560 修正已編成。Primary 才開始 `owned-items-recall-native-v1` 的44項受影響 D3D12 測試及4張設定 capture；此句不代表 native／viewport 通過。引擎來源、設定與 BuildId 均未手動修改。

## E560：回城案例把 FString 傳入 FCString::Strcmp（2026-10-09）

owned-items-recall-build-v1 普通 NoEngineChanges／NoUBA／nocache 真實失敗：workflow exit1，UBT OtherCompilationError，226.69秒；OmOwnedViewportIpcTests.cpp5303 C2664，FCString::Strcmp(ActiveCase, TEXT("b-channel-admits")) 的 ActiveCase 為 FString，無法轉成 const TCHAR*。修為 FString 比較或明確 *ActiveCase，保留該 channel admission 斷言並檢查同類呼叫。codegen17files/18inputs內容hash仍c9dda913c42a5118，Rust bridge staging已執行；不得因編譯錯誤觸發 Engine refresh 或手改 BuildId。完整build.log／UBT-Log／summary保存於 `.ai-collab/owned-items-recall-build-v1/`。未跑 native／viewport。下個實際錯誤 E561。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`PollPlainWire` 改為 `ActiveCase == TEXT("b-channel-admits")`。`Kind` 與 `TargetKind` 的 `const TCHAR*` `FCString::Strcmp` 呼叫點保留。`b-channel-admits` 仍要求協議開啟且 remaining 大於 1，並註明這是本地 transport admission，不是伺服器重開或完成。這不是 FailedDueToEngineChange，沒有 Engine refresh，也沒有手改 BuildId。完整 log 仍留在 `.ai-collab/owned-items-recall-build-v1/`。沒有 PASS。下一個實際錯誤編號仍是 E561，要等真實發現。這次沒有製造 E561。

## E559：新 Stop／Hold 鎖存漏失焦／修飾鍵 release 回收，設定測試仍繞過 hit test（2026-10-09）

E556修正後 Primary 實際 Lua contract 重驗31項通過，保存 owned-items-recall-lua-v1.json；這不是243／103／76視窗PASS。獨立 source review 發現新 AdmitLatchedHeroCommand 在正式可用性檢查前設 Down=true，Release 只有帶原 chord 的 UE IE_Released；全域 Slate HandleKeyUpEvent 只處理 Cast。ClearLatchedHeroCommands 僅出現於 profile install1597／RetireInput2154，Tick、focus/modal/source 退休均不清理或觀察實際放開。失焦／文字面板吞掉 release、或按住期間修飾鍵改變令原 chord release 不路由後，下一次正常 S/H 會被 Down&&sameKey 提前拒絕直到重裝profile。需沿用完整 physical release／owner lifecycle，或移除不必要額外鎖存並以實際IE_Repeat契約處理；必須證明恢復後能再操作，不能只驗無多餘wire。

同批 E557 的新模式按鈕測試仍沿用 ClickTag 的 GeneratePathToWidgetUnchecked→顯式 RoutePointerDown/Up 路徑，沒有實際 LocateWindowUnderMouse／hit.ContainsWidget 驗同一 SButton；ScrollDescendantIntoView 後僅 Prepass，不能證明已繪製且可點。需重用 E546／E547 已證的公開實際命中方法與 bounded paint/scroll settle，不接受指定路徑強制路由為玩家 hit 證據。

同批 Recall HUD 仍在 OmPlayerHudOwnerComponent.cpp516 硬編碼 Recall[B] 且僅依 protocol／remaining 判 ready，不看alive／phase；使用者改鍵後標示仍B，死亡保留protocol時仍ready。需實際profile鍵標籤及同一合法owner／alive／phase可用性，保留channeling與既有authority政策、不新增本地計時。普通 owned-items-recall-build-v1 正在執行，尚未有建置／native／viewport PASS。下個實際錯誤 E560。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：Stop／Hold 的 Down 鎖存、`AdmitLatchedHeroCommand`、`IE_Released` 與 `ClearLatchedHeroCommands` 已從 action owner 移除。`BindImmediateHero` 只綁 `IE_Pressed`，精確修飾鍵仍在。`IE_Repeat` 不進入該 delegate，拒絕也不占用下一記合法按下。沒有第二套鍵盤狀態機，測試也不清私人旗標。新視窗案例用公開按鍵事件：文字焦點、modal、視窗外、修飾鍵改變後恢復，再按一次恰好一條 wire；repeat 在下一記之前先等零 wire；來源死亡後放開沒有新 wire；按住 S 時把主要 Stop 改成 F7，S 的 repeat 不新增 wire，F7 才是下一條，結束前恢復 S。物品模式只把 `CastMode.Item2.Quick` 與 `CastMode.Item6.QuickWithIndicator` 改成 `LocateWindowUnderMouse`、目前 arranged geometry、hit-grid origin、同一 `SButton` 的 `ContainsWidget`，以及最多 12 次 Tick／Prepass／scroll。舊的 Q、召喚師、Shift、解除綁定仍用 `ClickTag`。`AddToViewport(100)`、`SOmFitModal` 與設定關閉命中沒改。回城狀態改為 `Recall[主要/次要]`，存活且 phase 1 且有限 hp 大於 0 才區分 channeling／ready，否則 unavailable。標籤走 `RefreshItemSlotPresentation` 與含 profile revision 的 `RefreshClusterScales`。`recall-label-rebind` 在 hp／mana／level 不變下記 F7／F8 再恢復 B。`recall-dead-label` 先對 channeling，死亡後協議仍開仍顯示 unavailable。`b-channel-admits` 仍是本地 admission。宣告清單改為 253 cases／115 wires／80 stages；已接受 165 前綴與原 53、其後 25 個名稱保留。243／103／76 不是這份清單的 PASS。Lua 31 與 Cargo 26 沒有重跑。沒有 PASS。下一個實際錯誤編號仍是 E561，要等真實發現。這次沒有製造 E561。

## E557：道具施法模式只有 API／JSON，操作設定沒有入口（2026-10-09）

Primary source review：新增六格 `TrySetItemCastMode`／`itemCastModes` 持久化，但 OmControlSettingsWidget.cpp415–498 僅建立 QWER 與 D/F 模式列，沒有呼叫 TrySetItemCastMode；owned cases 直接呼叫 setter，未證明玩家能改變模式。需加入六格資料驅動模式控制，顯示實際主要／次要鍵與選取狀態，沿用正式 save／rollback／retirement；用真實 arranged button hit 驗設定、關閉、重新載入，不可直接 OnClicked 或 setter 冒充 UI。尚未修正或重驗。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`OmControlSettingsWidget.cpp` 用同一個 `AddCastModeDomain` 建立 QWER、召喚師與六格物品列。物品列標籤是 `CastMode.Item1` 到 `Item6`，按鈕是普通／快速／指示器，標題顯示該格目前主要鍵、次要鍵與「目前」模式。按下呼叫 `UOmPlayerActionOwnerComponent::TrySetItemCastMode`，退休後回 `Handled`。`Om.Runtime.ControlRuntimePersistence` 以既有 `ClickTag`（先 `ScrollDescendantIntoView` 再公開指標命中）點 `CastMode.Item2.Quick` 與 `CastMode.Item6.QuickWithIndicator`，再 `LoadFromFile`。斷言版本仍是 5、只有被點的槽改變、Q 與召喚師模式保留、眼位列仍不存在。`AddToViewport(100)` 與設定關閉命中沒改。按鍵 4 綁定沒改。沒有 PASS。下一個實際錯誤編號仍是 E559。這次沒有製造 E559。

## E558：死亡道具與回城中 HUD 狀態不正確、六格／repeat 證據不足（2026-10-09）

Primary source review：OmWorldBridgeActor.cpp4489 的 bCanUse 只檢查 has_hero／phase，source_death 保留 hero 與物品並使 hp_raw=0，因此零冷卻主動道具仍 BuildItemHotbar 為 ready；實際提交另因死亡拒絕，HUD 與可用狀態不一致。回城剩餘3秒的 MatchRecallHud 刻意期待 `Recall[B] ready`，應呈現 channeling，不能當成 ready 或已完成。需使用同一個合法 owner／alive／phase／protocol 投影產生狀態，不新增本地預測計時器。

53個新增案例只對 slot0–2 施放及 slot3 被動拒絕，未實際按 key6／7 驗 slot4／5；名為 s-stop-repeat／h-hold-repeat 的案例仍呼叫固定 bIsRepeat=false 的 KeyDown，測的是再次按下而非 OS repeat。需補六格真正操作、Alt合法／拒絕、真實 repeat／精確 modifiers、道具 held focus／modal 退休與來源身分變更；保留165已接受前綴，更新一致的實際清單。現階段無 218／91／67 PASS，父項仍未完成。下個實際錯誤 E559。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`bCanUse` 改為與 `bOwnedAbilityHudReady` 相同的 has_hero、phase==1、有限且大於 0 的 hp，再加上既有玩家相符。物品列仍複製 catalog、item id 與冷卻；死亡且冷卻為 0 時 `bEnabled` 為假，簡潔文字不含 ready。`item-dead` 的 HUD 種類改為 8，拒絕字改為正式的 `Owned item is unavailable`。回城字在協議開啟且剩餘時間有限且大於 0 時是 channeling，否則 ready 或 unavailable。沒有本地計時器，也沒有把重複 Recall 改成拒絕。`MatchRecallHud` 種類 6、`OmPlayerHudOwnerTests` 的 1.5 秒，以及 recall smoke 的非零剩餘文字一起改成 channeling；剩餘 0.0 仍是 ready。Stop／Hold 改為精確修飾鍵，並用兩格 latch：已按下的同一鍵 repeat 不再送出，放開只清該鍵。`KeyDown` 仍是 bIsRepeat false。新增 `KeyRepeat` 送出 bIsRepeat true。原 165 名稱與原 53 名稱保留。清單改為 243 cases／103 wires／76 stages，新增 slot4／5 三種模式與 Alt-self、Stop／Hold 真 repeat、Shift／Ctrl／Alt、主鍵與 F11／J 次鍵、held focus／modal、source_death／source_replacement／source_removal 後原鍵放開零 wire。`s-stop-repeat` 與 `h-hold-repeat` 仍是第二次實體按下。沒有 PASS。下一個實際錯誤編號仍是 E559。這次沒有製造 E559。

## E556：ItemUse 驗證把 Lua pattern 當作正規表示式（2026-10-09）

Grok run-mv0kjmdk-u5ddcr 完成後，Primary 實際執行 `tools/lua/lua.exe scripts/tests/owned_viewport_ipc_contract_test.lua` exit1：`scripts/moba_owned_viewport_ipc.lua:534: ItemUse case policy is missing`，由 contract_test.lua321 的合法 bundle 觸發。解析 pattern `^(%d+):(self|point|unit):(%d+):([%w_]+)$` 使用 Lua 不支援的 alternation，因此 self／point／unit 合法案例均無法匹配。應先解析一般 token，再明確檢查三個合法 policy；保留嚴格 slot／catalog／item identity、explicit point 與 negative case 斷言。尚未修正或重驗，218／91／67 是宣告清單而非通過結果。下個實際錯誤 E557。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`M.parse_item_policy` 先用 `^(%d+):(%a+):(%d+):([%w_]+)$` 取有界 token，再只接受 self、point、unit。未知 policy（含 `ally` 與 `Self`）回 `ItemUse case policy is unknown`；`self|point` 與截斷字串回 `ItemUse case policy is missing`。slot、catalog、item id、explicit 相等、unit／self 的 null 與禁止別名都留在 `check_intent`。契約測試直接呼叫 parser，不靠改 bundle 假裝政策。清單數字改為 243／103／76，這只是宣告，Lua 沒有重跑。沒有 PASS。下一個實際錯誤編號仍是 E559。這次沒有製造 E559。

### E548–E555 道具基礎當前範圍重驗結果（2026-10-09）

item-target-foundation-accepted-v1：21道具＋27content-model＋3catalog＋1renderer＋18client＋1bridge＋3codegen＋2secure transport＝76實際測試成功，core all-targets check成功。E553造成部分workflow中止但已成功case明確沿用，未宣稱失敗workflow整體成功；原v1/v2編譯／fixture失敗、v3/v4中止證據保留。vendored protoc正式fallback與codegen17files/18inputs／check通過（c9dda913c42a5118）；E555依實際FailedDueToEngineChange作refresh後，freshordinary build-v3 NoEngineChanges／nocache成功2.70s。native-v1 exact4 D3D12成功／0warning，artifact consistency通過。僅接受道具基礎；六格鍵鼠／回城continuation、所有畫面品質與最終完整驗收仍pending。下個實際錯誤E556。

## E555：普通道具建置遇到實際 Engine artifact change（2026-10-09）

item-foundation-build-v1 普通NoEngineChanges／NoUBA／nocache失敗（workflow exit1／UBT exit5）；UHT處理完成後，UBT列出需重建NetCore與Engine modules/version等既有產物，實際Result FailedDueToEngineChange，UBT227.77秒。Rust DLL／bridge／cbindgen staging已執行，非新道具C++ compiler error。遵守既定規則：只因這個實際結果，執行正式refresh no-cache建置，再以新的ordinary no-cache證據通過 gate。不得手改Engine source/config或BuildId。原失敗log保留，尚未refresh或PASS。下個實際錯誤E556。

## E554：client explicit-point helper 呼叫漏借用（2026-10-09）

Rust v4 catalog3與renderer1通過，core all-targets check成功；client-input-admission 編譯E0308，input_bridge.rs198把ItemUseIntent傳值給接受&ItemUseIntent的item_point_from_intent。需修正呼叫借用，保留shape與ownership，非schema問題。完整stderr保留，client測試未執行。後續只跑未完成步驟；下個實際錯誤E555。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`input_bridge.rs` 第 198 行改為 `item_point_from_intent(&intent)`。helper 仍只讀 explicit 成對座標或舊 scalar，之後仍用同一個 intent 寫 `item_slot`，target／owner life 仍是 None。沒有改 schema，也沒有改測試。原 warning 保留。沒有 PASS。下一個實際錯誤編號仍是 E555。這次沒有製造 E555。

## E553：Primary 驗證 helper 重寫 progress.json 被正確拒絕（2026-10-09）

Rust v3 道具21測試全部成功，content-model 已完成，但 helper 第二步再次 json.write(progress.json,false) 被固定 Lua path 防覆寫契約拒絕。這是 Primary 新增驗證 helper 的錯誤，不是遊戲失敗。改為每步唯一 step-name-progress.json，原 v3 summary／stdout 全保留；後續只執行尚未完成步驟，不重跑已通過且未變更的21道具。下個實際錯誤E554。

## E552：傷害歸屬 fixture 在離開商店後才購買（2026-10-09）

item-foundation-rust-v2 compilation17.70s成功，21 item tests 執行20成功／1失敗；credit fixture gold實際5000而非4550。該測試先把 owner／ally 移到(0,7000)，再送 ItemBuy；正式 shop 要求距離己方base300內，因而正確拒絕。修測試順序：在真實己方商店透過正式 input 購買後，再搬到隔離戰鬥位置；保留商店規則與 gold／kill／assist／lane credit 全部斷言。原失败日志保留，尚未重驗；不能宣稱全部道具通過。下個實際錯誤E553。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：只改 `item_target_credit_60hz_records_kill_assist_and_lane_last_hit`。購買前把 player 1 與 player 3 放到 `MobaMatch.bases[slot.side]` 的實際 `Pos`，再用 `driver.step` 買 `moba_training_hex`。買成後才檢查背包與 4550，然後才 disarm／hold 並移到 (0,7000)／敵人 (400,7000)。商店半徑、生產 shop、其他 20 個已通過測試都沒改。kill 40、assist 15、lane 20 與 anonymous script 斷言保留。沒有 PASS。下一個實際錯誤編號仍是 E553。這次沒有製造 E553。

## E551：新道具 fixture 編譯失敗（2026-10-09）

item-foundation-rust-v1 首步 base-content-items cargo exit101／零測試執行，19 diagnostics。根因：specs::Builder 未 import，read_resource::<ItemRegistry>().clone() 複製 Fetch guard 而非 ItemRegistry 導致 borrow／'static cascade，MobaMatch temporary fetch 的 slot 引用超過 guard lifetime。完整 stderr／summary 保留在該 evidence 目錄。修正 trait 與 guard／資料 lifetime，不移除或削弱斷言；與 E550 source finding 同批 bounded repair 後 fresh 重跑。沒有 PASS。下個實際錯誤 E552。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`single_lane_match_tests.rs` 補上 `specs::Builder`。ItemRegistry 先解參考再 `clone`，複製的是資源本體，guard 在 `insert` 前結束。`counts` 先綁住 `MobaMatch` guard，再讀 kills／assists。原斷言、mismatch registry 與 credit 案例都留著。沒有 PASS。下一個實際錯誤編號仍是 E552。這次沒有製造 E552。

## E550：地點道具把失效 entity 解析結果誤當欄位未提供（2026-10-09）

source review：direct helper 與 queued bind 將原始 target_entity 解析成 target_life；PointNova 只檢查 target_life.is_some。失效 index 或 generation 解析失敗會變 None，使 explicit point＋非法 entity 可能通過錯誤 shape。必須分開保留欄位 presence 與生命週期解析結果，嚴格拒絕 point 的附加 entity／orphan generation，保留合法 origin 與舊 self 相容。当前 Rust v1 尚在執行，未宣稱此案已測試或修復；等驗證 terminal 才派 sole writer。下個實際錯誤 E551。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：point 預檢同時看原始欄位 presence 與已解析 `target_life`。direct 的 presence 是 `target_entity.is_some()`，lookup 失敗不再當成沒有目標。queued drain 的 presence 是 index 或 generation 任一存在，bind 仍只寫 life、不清原始欄位。有名稱的 point，含失效 index、generation 0、`u32::MAX`、已刪除 index、orphan generation，在效果與冷卻前拒絕，合法 explicit origin 仍打在原點。unit 仍要有名稱且是當下存活的同一 life，不改綁現在佔用該 index 的實體。self 仍忽略附加座標、entity、generation 與 explicit。authority projection 在目標有名稱但沒有 canonical id 時整筆丟棄，不再把欄位清掉後當成無目標 point；`(None, None)` 仍是合法無目標。filtered injection 若 payload 仍帶名稱而 accepted target 是空的，拒絕注入，不把該 shape 清成缺欄。新 fixture `item_point_shape_rejects_unresolved_entity_presence` 已寫入但未執行。沒有 PASS。下一個實際錯誤編號仍是 E552。這次沒有製造 E552。

## E549：owner life 沒有從 accepted actor 保留到道具執行（2026-10-09）

E548 修正已 terminal，但 owner_entity 仍在第一次 drain 才從 player_id 找當前英雄。secure gateway 丟棄已驗證 actor_id；filtered injection 雖驗 actor 所有權卻沒有把 actor life 帶進 ItemUse。既有 owner replacement fixture 先手動 bind，未涵蓋 accepted 後、首次 bind 前的替換。要保留受信任 owner life 並驗玩家所有權，補真實前段替換與其他同隊玩家 actor 拒絕案例；renderer 不得提供 canonical life。source-aware damage 與新 fixed 數值保留。尚未驗證，不宣稱 PASS。下個實際錯誤 E550。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：`ItemUse` 增加 `owner_entity`／`owner_generation`（field 6、7，是 index 與 life，不是 canonical id）。secure rewrite 保留已驗證 actor 並覆蓋 client 欄位；actor generation 0 整筆拒絕。reference-free `TAG_INPUT_SUBMIT` 先清掉偽造 owner 與 `target_generation`，再由 authority tick 的 `seal_item_use_owner` 綁當前英雄。已成對的 owner life 若不再屬於該玩家或不再是當前英雄，seal 與第一次 bind 都不會改綁新人。filtered injection 寫本地 actor generation，不抄 authority generation。`ItemUseIntent` 沒有這兩個欄位。`handle_item_use_from_input` 仍是當下 index，不是 queued acceptance 的證據。新 fixture `item_accepted_owner_life_rejects_replacement_before_first_bind` 已寫入但未執行。沒有 PASS。下一個實際錯誤編號仍是 E550。這次沒有製造 E550。

## E548：道具 source review 的傷害歸屬、生命週期與精度缺口（2026-10-09）

foundation 已 terminal 但尚未接受。實際程式把新道具傷害傳給 anonymous script helper，三種傷害 ledger 均收到 None；secure rewrite 與 filtered injection 丟棄已驗證 generation，queue 沒綁定 owner／target life，既有 replacement fixture 沒有真正重用 index；新 Fixed64 damage／radius 先轉 render f32 再轉回 Fixed64。三者違反原始 contract，不能以 UI／admission 通過代替正確 authority 效果。bounded repair 要保留真實施放者與受驗證生命週期、維持新數值 fixed 精度，並加入真實 credit／index 重用／精度 fixtures。舊 self 道具與 anonymous script 行為保留。尚未測試，沒有 PASS。下個實際錯誤 E549。

修正（source only，測試、建置、codegen 都還沒跑，不能寫成通過）：道具改經 `apply_checked_direct_damage`，並傳入實際 owner `Entity`；`handle_script_direct_damage` 仍傳 `None`。`ItemUse` 增加 optional `target_generation`（field 5）。secure rewrite 與 filtered injection 寫入的是該世界自己的本地 life，不是 canonical generation，renderer intent 不帶這個欄位。`bind_queued_item_lives` 在 drain 前綁定 owner Entity 與 target life；generation 0，或只有 generation 沒有 index，都不改綁目前佔用該 index 的實體。PointNova／UnitHex 以 Fixed64 執行，`checked_item_fixed` 拒絕非正與超上限，不回退 catalog。三個新 fixture 已寫入但未執行：kill／assist／lane last-hit、同 index 重生與 owner 替換、fixed 精度邊界。沒有 PASS。下一個實際錯誤編號仍是 E549。這次沒有製造 E549。

### E544–E547 當前範圍重跑結果（2026-10-09）

hud-readable-accepted-v1：ordinary build-v17 nocache成功（refresh=false，UBT13.28秒）；native-v11 exact65成功與16張已目視widget圖保留，設定層級修正另跑settings-layer-native-v1 exact1 ControlRuntimePersistence成功／0警告。fresh viewport-v4 165cases／69intents／51stages與全部cleanup成功，三張owned-window PNG逐張目視，D/F158案例實際前綴比對通過。關閉鈕仍使用原座標888.3,661.9，現在實際hit path為同一OmSettingsClose SButton，真實Down／Up後modal解除；證實原player layer位於HUD viewport overlay之下。五張放大文字圖均恢復小地圖。沒有抹除先前失敗，也沒有全套驗收／authority ACK／新增權威對局。完整世界品質與實際DPI下設定字體仍列後續，不能把此次功能通過當成所有畫面完成。

## E547：LocateWidgetInWindow 是 protected（2026-10-09）

Primary ordinary hud-readable-build-v16 nocache 失敗（refresh=false，UBT 14.60 秒）。唯一錯誤是 C2248：`OmOwnedViewportIpcTests.cpp` 第 5229 行呼叫 `FSlateApplication::LocateWidgetInWindow`，該函式是 protected。沒有跑 native，也沒有跑 viewport。編譯紀錄在 `.ai-collab/hud-readable-build-v16/UBT-Log.txt`。

`SlateApplication.h` 第 1180 行起是 protected，第 1275 行宣告 `LocateWidgetInWindow`。第 1664 行起是 public，第 1674 行是 `LocateWindowUnderMouse(coordinate, const TArray<TSharedRef<SWindow>>&, bool, int32)`。

修正（build-v17、ControlRuntimePersistence、viewport-v4 都還沒跑，不能寫成通過）：同一處改呼叫公開的 `LocateWindowUnderMouse`，陣列裡只有目前這扇 owned `Window`。Desktop 仍是 arranged 中心加 `GetGridOrigin()-GetGridWindowOrigin()`。命中路徑仍必須包含同一顆 tagged widget，才做 Down／Up。失敗診斷、`AddToViewport(100)`、modal 解除檢查都保留。沒有子類別去碰 protected，也沒有改用全部互動視窗。

清單維持 65 native／165 cases／69 intents／51 stages／16 native PNG／3 viewport PNG。沒有 PASS。下一個實際錯誤編號是 E548。這次沒有製造 E548。Primary 先 ordinary hud-readable-build-v17，接著 D3D12 的 `Om.Runtime.ControlRuntimePersistence`，然後 fresh viewport-v4。完整驗收留最後。

## E546：設定關閉鈕排在 HUD overlay 之下（2026-10-09）

Primary 實測 ordinary hud-readable-build-v15 nocache 成功（refresh=false，UBT 38.01 秒）。D3D12 hud-readable-native-v11 exact 65（64 無警告＋1 警告 Success）。16 張圖各看一次，五張放大文字圖都有小地圖地面與 VISION N/A。E544 的 paint 矩形與像素檢查通過。那些修正保留。

fresh hud-readable-viewport-v3 已終止失敗。session 73201 closed。161 cases、catalog true、69 intents、51 stages。最後案例 display-modal-suppresses。Publisher 34140、token 134359889512935630。editor 與 profile cleanup 都 retired。listener released true、authority false。沒有新的權威對局。後面 4 個 display case 沒跑完。display-settings-visible.png 沒產生。沒有 HUD 接受。

OmSettingsClose 是 Visible。tick Abs 872.390,655.915、paint Abs 662.390,515.915、scale 0.56、local 57.225×21.414。視窗 tick Abs 210,140、paint Abs 0,0、scale 1.75、local 457.143×342.857。screen 210,140、client 1400.5×1050。tick 中心 888.3,661.9 命中 SBorder#OmCombatCluster，路徑穿過 HUD normalizer。tick-on-paint 678.3,521.9 命中 STextBlock#Chord.UpgradeW，路徑在 SOmFitModal／OmSettingsPanel 的 BindingRow.UpgradeW。paint-on-tick 與 tick 中心相同，已略過。兩點都不是關閉鈕。

假設是 tick 與 paint 只差一個平移，把中心換到另一個原點就會命中。這次實測否證。兩個中心正好差 screen (210,140)。Engine `SWindow::PaintWindow`（SWindow.cpp）把 hit grid 設成 `SetHittestArea(GetPositionInScreen(), GetViewportSize())`，所以 GridOrigin 是螢幕原點、GridWindowOrigin 是 0。`GetBubblePath` 用 desktop 減 GridOrigin，再拿目前 paint geometry 做精確測試。tick 中心減掉 GridOrigin 就是按鈕自己的 paint 中心，命中的卻是戰鬥列。不再加一次 scale：0.56 已是累積 layout scale（視窗 1.75 乘上遊戲 DPI 曲線再除回視窗 scale，800×600 約 0.555，ToString 印成 0.56）。`HittestGrid.h` 另外有 `GetWidgetRenderBoundingRect`，它是 `GetHitTestBoundingRect()` 再減 GridWindowOrigin，不是另一套按鈕矩形。沒有第二顆 retired `OmSettingsClose`；元件只留一個 `SettingsWidget`，關閉時 `RetireSettings` 加 `RemoveFromParent`。

原因是圖層，不是平移。`ToggleSettings` 用 `AddToPlayerScreen(100)`。`UGameViewportClient::AddViewportWidgetForPlayer` 把它放進 `SGameLayerManager` 的 player slot。HUD 的 `AddToViewport` 放進後面的 viewport overlay slot。`SOverlay` 後面的 slot 畫在上面，也蓋住 hit test。關閉鈕排在視窗下緣，和 `OmCombatCluster` 重疊，所以正確的 paint 中心打到戰鬥列。清單裡的 UpgradeW 沒被戰鬥列蓋住，所以少減一次原點的那個點打進設定頁。`SOmFitModal` 的排版沒有把關閉鈕移到清單中央，這次不改它，也不改小地圖。

修正（build-v16、native、viewport-v4 都還沒跑，不能寫成通過）：設定改 `AddToViewport(100)`，跟 HUD 同一個 overlay，Z 高於 HUD root 0 與 command bar 20（viewport client 會再加 10）。`TryClickTag` 從目前遊戲視窗的 `GetWindowGeometryInWindow` 做 ArrangeChildren，不用 fast-path 的 cached geometry。取這個 tag 的可見arranged按鈕，中心用accumulated scale 進 window space，再加上 `GetGridOrigin()-GetGridWindowOrigin()`。只對這個視窗呼叫 `LocateWidgetInWindow`。路徑裡必須是同一顆 widget，才做真正的 Down／Up。成功時記下 point-space `arranged-window+grid-origin`、desktop、window center、兩個 grid 原點、scale、arranged path 與 hit path。失敗時同時印 cached tick／paint、grid、screen、client 與每個候選的 hit path。不掃格、不寫死 1.75 或 210,140、不呼叫 CloseSettings、不用 Esc、不把 SBorder 當成成功。後面仍檢查 modal 有沒有放開。

清單維持 65 native／165 cases／69 intents／51 stages／16 native PNG／3 viewport PNG。沒有新的 automation 名稱。沒有 PASS。下一個實際錯誤編號是 E547。這次沒有製造 E547。Primary 先跑 ordinary hud-readable-build-v16。生產檔有改，viewport-v4 之前要跑會呼叫 `ToggleSettings` 的 `Om.Runtime.ControlRuntimePersistence`。16 張已檢視的 HUD 圖沒有改像素，不為它們重做視覺比對。然後 fresh viewport-v4。成功才看 3 張實際 display 圖、對 DF-v2 做 prefix、範圍內接受。完整驗收留最後。

## E545：設定關閉點擊落在 SBorder（2026-10-09）

實際 owned `hud-readable-viewport-v2` 已終止失敗。session 32996 closed。161 cases、catalog true、69 intents、51 stages。最後案例 `display-modal-suppresses`。失敗是 `display-modal-suppresses click on OmSettingsClose landed on SBorder`。native/workflow false、authority_ack false。Publisher、editor、profile cleanup 都已 retired。cleanup listener released true、authority false。publisher 45112、token 134359869383456406 retired。沒有新遊戲。後面 4 個 display case 沒有跑完。3 張 PNG 沒有全部產生。沒有 HUD prefix，也沒有 acceptance。`display-hud-visible.png` 與 `display-hud-hidden.png` 已產生，Primary 以 ORIGINAL 檢視：遊戲世界、HUD 與 native frame 可見；隱藏後戰鬥列與小地圖拿掉，設定按鈕仍在。E539 的空截圖沒有再次發生。這兩張證明目前範圍的視窗讀回。`display-settings-visible.png` 還沒走到。

`OmSettingsClose` 是 `SButton`，按鈕文字是 HitTestInvisible。點在按鈕上時，leaf 必須是 `SButton`。同一案例稍早的 `OmControlSettingsButton` 已用同一個 `TryClickTag` 打開設定，所以這個視窗的 tick 中心可以命中真正的 `SButton`。失敗字串代表 tagged widget 已找到、可見、tick 絕對尺寸 min 至少 4，然後 `LocateWindowUnderMouse` 的 leaf 在整個 retry 都是 `SBorder`。全螢幕暗色 `SBorder`，以及沒有覆寫 `GetType` 的面板 `SOmConsumeWheelBorder`，都符合這個字串。假設是關閉鈕的 tick 中心不在它這一幀的 paint／hit 矩形裡，不是 `ButtonLeaf` 該把 `SBorder` 當成按鈕。`SOmFitModal` 建構時是 1100×760，原本要等 `Tick` 才收到 `Local-16`。Hit grid 用的是當下畫出來的 geometry。IPC 的 `Paint` 只有 `SlatePrepass` 與 `ForceRedrawWindow`，沒有 responsive fixture 的 `Slate.Tick()`。Responsive 測試只檢查關閉鈕的 cached 矩形在面板與視窗內，沒有真的點下去。

E545 source repair（尚未重跑 build、native、capture、viewport 或 prefix，不能寫成通過）：`SOmFitModal` 改在 `OnArrangeChildren` 用這一幀的 allotted 尺寸把面板排進視窗，關閉鈕和 hit grid 同一幀。`TryClickTag` 仍先用 tick 中心做真正的 mouse Down／Up。沒有命中這個 tagged `SButton` 時，再用視窗 tick 原點與 paint 原點把中心換到另一個空間。沒有寫死 1.75，也沒有固定像素位移。兩個空間都不中才失敗，並印出 paint 矩形、tick 矩形、嘗試過的點、視窗 tick／paint、screen、client、路徑上的型別與 tag、visibility。不呼叫 `CloseSettings`、不用 Esc、不刪案例、不把 `SBorder` 當成成功。後面仍檢查關閉有沒有放開 modal。若真正原因還不是這個幾何差，案例仍會失敗，但不再只剩 `landed on SBorder`。

清單維持 65 native／165 cases／69 intents／51 stages／16 native PNG／3 viewport PNG。沒有新的 automation 名稱。沒有 PASS。下一個實際錯誤編號是 E546。這次沒有製造 E546。

## E544：原生全過後目視發現放大文字截圖缺少小地圖（2026-10-09）

ordinary nocache hud-readable-build-v14 實際成功，refresh false，UBT16.09秒。D3D12 hud-readable-native-v10 exact65名稱全Success（64無警告、1警告成功），failed/notRun/inProcess均0。Primary對owned-summoner-native-v2只執行一次比對，16張皆為變更／新圖，並逐張以ORIGINAL要求檢視。紀錄 `.ai-collab/hud-native-v10-visual-findings.md`。沒有寫接受用visual-review JSON。

實際5張預設responsive截圖都有左下小地圖地面矩形與VISION N/A；5張放大文字截圖（800-text125、1280/1920/2560-text150、400-text125）皆沒有這個矩形與狀態文字，只見黑底，戰鬥列與設定按鈕仍在。HP／LV／XP與技能熱鍵／冷卻重疊問題已改善，但不能因此接受完整HUD。原因尚未確認成 TextScale 單獨的生產回歸。`SOmMinimap::OnPaint` 只要被畫到就會畫背景；有地圖時最後畫 `FogStatusLabel`（沒有 fog 資料就是 VISION N/A）。戰鬥列與小地圖 cluster 共用 `bShowHud`。Viewport-v2 的 `display-hud-visible.png` 有畫出小地圖，所以不能把生產繪製整段改掉。文字迴圈在 80×60 explicit no-fit、商店 wheel、settings `AddToViewport`／`RetireSettings`／`RemoveFromParent` 之後，只改 `TextScale`。native-v10 的 fit log 在這些文字尺寸上 mini desired 是 120 或 80，但那個迴圈當時沒有檢查 paint 矩形、左右側或截圖像素。

E544 source repair（尚未重跑，不能寫成通過）：fit 已經會讓小地圖 cluster layout／prepass 失效；現在同一處再加上 Paint，套用左右 `RenderTransform` 之後再失效一次 Paint。這是讓 80×60 之後的 fast path 不能留著空的繪製層。沒有改 `OnPaint`、沒有藏起小地圖、沒有拿掉截圖、沒有重排 80×60、沒有加另一圈 paint。文字迴圈在既有截圖寫出之後，檢查小地圖 paint 矩形在視窗內、不壓到戰鬥列、paint 局部邊長至少 32、中心在 preview 的左或右側；有 capture 時，再用 HudRoot 截圖裡該矩形的像素確認有地形或 VISION N/A 那種亮度。沒有假的 golden PNG。若像素仍是黑的，這條既有測試會失敗並印出矩形，原因仍要以那次輸出為準。Viewport-v2 已終止，見 E545。完整驗收留最後。沒有 PASS。

## E543：縮放後的文字換行快取讓 compact 仍過高（2026-10-09）

實際 ordinary nocache `hud-readable-build-v13` 通過，refresh false，UBT 98.08 秒，session 46304 closed。D3D12 `hud-readable-native-v9` 已終止，程序 exit 0、suite false、session 64005 closed：65 名稱、64 Success、1 Fail、0 notRun、0 inProcess。唯一失敗 ResponsiveHudLayout，1 error、1 warning。錯誤仍是 `Text scale 1.25 at 400x300 fits`，`OmResponsiveHudTests.cpp:895`。預設解析度、就緒列，以及其他放大解析度都已到達並通過。沒有 16 張 capture 或視覺接受。沒有 viewport-v2。E539 的視窗截圖仍未執行。

原因在 Engine `Slate/Private/Widgets/Text/SlateTextBlockLayout.cpp`，這次沒有改 Engine。`CachedSize` 初始為 0。`OnPaint` 第 240 行把當下 `AllottedGeometry.GetLocalSize()` 寫入 `CachedSize`。`CalculateWrappingWidth` 第 397–407 行先取明確的 `CachedWrapTextAt`；若 auto-wrap 開啟且 `CachedSize.X >= 1`，寬度改成 `min(明確寬, CachedSize.X)`。隔離的 `SlatePrepass` 不會 paint，所以 `WrapTextAt(160)` 加上 auto-wrap 仍沿用上次繪製的較窄寬度。測試在文字縮放迴圈之前先 paint 80×60 的 explicit no-fit。英雄狀態建構時 auto-wrap 開啟且 `WrapTextAt` 為 160；compact 的經濟與六個物品又 `SetAutoWrapText(true)` 並另設欄寬。`ApplyHudDensity` 也把經濟 auto-wrap 設成 compact。同一段文字有兩個寬度主人。

`hud-readable-native-v9/native-engine.log` 第 3118–3121 行是同一幀較早的四次嘗試，物品高都是 102、狀態都是 95×88：mini 120 並排 combat 247×351、堆疊 360×340；mini 80 並排 279×340、堆疊 360×340。最後一次 400×300 文字 1.25 的四條都是 `clause=combatH`（第 3130–3133 行，自動化複本第 3228–3231 行，錯誤在第 3238 行）。mini 120 並排 limit 284、combat 247×337、hero 101、ability 82、economy 44、items 88、stats 95×88、column 202、summonerBudget 47.5，英雄與資源並排，技能 4 並排、召喚 0。mini 120 堆疊 limit 156、combat 368×274、hero 101、ability 71、economy 22、items 58、stats 95×88、column 330、summonerBudget 111.5，技能 4 與召喚 2 都並排。mini 80 並排 limit 284、combat 290×304、hero 101、ability 71、economy 22、items 88、stats 95×88、column 242、summonerBudget 67.5，技能 4 與召喚 2 都並排。mini 80 堆疊 limit 196、combat 368×274、其餘與上一條堆疊相同。E542 已把最近的 mini 80 並排從 313 降到 304，技能 71、經濟 22，但狀態在 column 330 與明確 160 之下仍是 95×88。95 小於 160，第二個主人是上次繪製寬度。最近一次仍高約 20。Fit 條款與門檻仍然正確，不能放寬。

E543 source repair（尚未重跑 build、native、capture、viewport 或 prefix，不能寫成通過）：每次 fit 在 cluster prepass 之前關掉英雄狀態、經濟、六個物品的 auto-wrap，只留一個明確 `WrapTextAt`。英雄是 `min(160, 實際欄寬)`，完整 HP／LV／XP 仍畫出，不省略、不縮字。compact 經濟維持商店鈕剩下的寬，`max(48, InnerWrap - 商店鈕 - 8)`；roomy 經濟為 0。物品是內欄扣掉既有槽邊 6 的文字預算。`ApplyHudDensity` 不再打開經濟 auto-wrap。E542 的並排、資源、讀數與診斷保留。熱鍵與冷卻仍是明確 wrap。狀態列、商店空結果、記分板、結算與設定頁的 auto-wrap 不進入戰鬥 cluster 的期望尺寸，沒有改。沒有改測試順序，沒有拿掉 80×60，沒有改 Engine。400 預設與文字 1.25 仍要 fit。400 文字 1.5 不保證。80×60 仍是 explicit no-fit。沒有新的 automation 名稱或 PNG。清單維持 65 native／165 cases／69 intents／51 stages／16 native PNG／3 viewport PNG。

Primary 審查後跑新的 ordinary `hud-readable-build-v14`；只有 build 成功才跑 D3D12 `hud-readable-native-v10`。成功才對 DF-native-v2 做一次 capture 比對並看全部變更圖，然後 fresh viewport-v2，預期 165／69／51 加 3 張實際畫面與 prefix。完整驗收留最後。沒有 PASS。下一個實際錯誤編號是 E544。這次沒有製造 E544。

## E542：放大後的 compact 戰鬥列高過 400×300 文字 1.25（2026-10-09）

實際 ordinary nocache `hud-readable-build-v12` 通過，refresh false，UBT 13.81 秒。D3D12 `hud-readable-native-v8` 已終止，程序 exit 0、suite false、session 20880 closed：65 名稱、64 Success、1 Fail、0 notRun、0 inProcess。唯一失敗 ResponsiveHudLayout，1 error、1 warning。錯誤是 `Text scale 1.25 at 400x300 fits`，`OmResponsiveHudTests.cpp:895`。E540 的就緒列已經通過。預設 400／800／1280／1920／2560，以及放大的 800@1.25、1280／1920／2560@1.5，都在最後的 400@1.25 之前到達並成功。沒有 16 張 capture 或視覺接受。沒有 viewport-v2。E539 的視窗截圖仍未執行。

400×300 文字 1.25 的四條 `OM_HUD_FIT` 都是 `clause=combatH`。mini 120 並排 combatW 256、inset 8、limit 284、combat 247×421。mini 120 堆疊 combatW 384、inset 136、limit 156、combat 368×283。mini 80 並排 combatW 296、inset 8、limit 284、combat 290×313。mini 80 堆疊 combatW 384、inset 96、limit 196、combat 368×283。預設 400 的並排 mini 80 是 combat 272×279，低於 284，已通過。寬度沒有超過 CombatW+2。最近的一次只高約 29。E538 的獨立讀數加上完整 HP／LV／XP 換行，把 compact 高度抬過上限。Fit 條款、字級底限與數值門檻仍然正確，不能放寬。

E542 source repair（尚未重跑 build、native、capture、viewport 或 prefix，不能寫成通過）：compact 仍用測量後的欄寬決定排列。英雄名字在欄寬放得下 160 的狀態換行再加名字餘量時，跟 HP／LV／XP 同一列；狀態仍自動換行、不省略、不縮小字級。熱鍵與冷卻在未換行的一對讀數放得進卡片預算時並排，否則維持上下兩列並把過寬的字換回圖示寬。技能卡寬仍要小於等於圖示加 20。冷卻可見性仍只在那條 `SBorder`。有魔力時，compact 的生命與魔力同一列；roomy 維持上下排列，沒有魔力時不再留空高。每次 fit 另外記下 hero、ability、economy、items 的期望高度與狀態字尺寸，條款字串與比較式不變。400 預設與文字 1.25 仍要 fit。400 文字 1.5 不保證。80×60 仍是 explicit no-fit。沒有新的 automation 名稱或 PNG。清單維持 65 native／165 cases／69 intents／51 stages／16 native PNG／3 viewport PNG。

Primary 審查後跑新的 ordinary `hud-readable-build-v13`；只有 build 成功才跑 D3D12 `hud-readable-native-v9`。成功才對 DF-native-v2 做一次 capture 比對並看全部變更圖，然後 fresh viewport-v2，預期 165／69／51 加 3 張實際畫面與 prefix。完整驗收留最後。沒有 PASS。下一個實際錯誤編號是 E543。這次沒有製造 E543。

## E541：就緒冷卻診斷把 EVisibility 轉成 int32（2026-10-09）

實際 ordinary nocache `hud-readable-build-v11` 終止失敗，refresh false，exit 1，UBT 17.35 秒。`OmResponsiveHudTests.cpp` 143 的 `static_cast<int32>(Cooldown->GetVisibility())` 造成 C2440，接著格式檢查 C7595。沒有 native-v8，也沒有 viewport-v2。Primary 審查時沒有先核對這個型別。

原因：`Engine/Source/Runtime/SlateCore/Public/Layout/Visibility.h` 的 `EVisibility` 是 struct，不是 enum。公開診斷是 `FString ToString() const`。私有 `Value` 不能拿來印。`%d` 與這個運算式不合，所以格式檢查跟著失敗。列是否 `Collapsed`、`ArrangeChildren` 是否含這一列，以及其餘錯誤文字都不是這次的編譯錯誤。

E541 source repair（尚未重跑 build、native、capture、viewport 或 prefix，不能寫成通過）：同一句診斷改成 `%s`。有冷卻元件時用 `ToString()`；元件不在或 `ToString()` 是空字串時用 `missing`。不轉整數、不讀私有欄位、不加抑制。檢查與 `ArrangeChildren` 不變。清單維持 65 native／165 cases／69 intents／51 stages／16 native PNG／3 viewport PNG。

Primary 審查後跑新的 ordinary `hud-readable-build-v12`；只有 build 成功才跑 D3D12 `hud-readable-native-v8`。成功才對 DF-native-v2 做一次 capture 比對並看全部變更圖，然後 fresh viewport-v2 與 prefix。E539 的視窗截圖仍未執行。完整驗收留最後。沒有 PASS。下一個實際錯誤編號是 E542。這次沒有製造 E542。

## E540：就緒技能冷卻子元件的可見性檢查失敗（2026-10-09）

ordinary nocache hud-readable-build-v10 實際成功，refresh false，UBT 127.32 秒。D3D12 hud-readable-native-v7 已終止，程序 exit 0 但 suite false：65 個預期名稱／64 Success／1 Fail／0 notRun／0 inProcess。唯一失敗 ResponsiveHudLayout，1 error／1 warning，實際訊息為 `fixture 800x600 ability 1 hotkey and cooldown are separate painted regions (ready cooldown is not collapsed)`，OmResponsiveHudTests.cpp608。後續 compact／enlarged 與完整 16 張 capture 尚未驗證，沒有視覺接受或 viewport-v2。E539 的視窗截圖修正還沒有被 viewport 跑過。

原因：技能 1 的冷卻 fraction 是 0。可見性綁在冷卻列的 `SBorder`。`SWidget::Prepass_ChildLoop` 先 `UpdateOnlyVisibilityAttributes`，父層因此變成 `Collapsed`，接著不呼叫該列的 `Prepass_Internal`。子 `STextBlock` 的 visibility lambda 不會被計算。`SWidget` 建構時 `VisibilityAttribute` 的預設是 `EVisibility::Visible`，所以子元件的 `GetVisibility()` 仍是 Visible。這不是列還在畫。`SBoxPanel::ComputeDesiredSize` 與 `ArrangeChildrenInStack` 都跳過 `Collapsed` 子層，沒被 arrange 的列也不會進入 paint。native-v7 失敗的是這個子屬性，不是冷卻資料。召喚師列有同一組雙重綁定；fixture 的召喚師冷卻一直可見，所以這次沒有走到那條失敗。

E540 source repair（尚未重跑 build、native、capture、viewport 或 prefix，不能寫成通過）：可見性只留在冷卻列 `SBorder`。技能列仍是 fraction > 0 才 `HitTestInvisible`，否則 `Collapsed`。召喚師列仍是 remaining／total 都有限才顯示，包含 `0/12`。同一顆 `STextBlock`、真實秒數、已知／未知／就緒／冷卻中的語意都留著。子元件不再各自綁 visibility。就緒時測試擁有列是 `Collapsed`，而且父 panel 的 `ArrangeChildren` 不含這一列，所以沒有版面空間、這一幀也不畫。冷卻中仍要求擁有列可見，文字畫進卡片、不與熱鍵或圖示重疊，並維持字級底限。不拿空字串當通過、不刪冷卻、不造假秒數、不放寬 fit／字級／重疊。沒有新的 automation 名稱。清單維持 65 native／165 cases／69 intents／51 stages／16 native PNG／3 viewport PNG。

Primary 審查後跑新的 ordinary `hud-readable-build-v11`；只有 build 成功才跑 D3D12 `hud-readable-native-v8`。成功才對 DF-native-v2 做一次 capture 比對並看全部變更圖，然後 fresh viewport-v2，預期 165／69／51 加 3 張實際畫面與 prefix。完整驗收留最後。沒有 PASS。下一個實際錯誤編號是 E541。這次沒有製造 E541。

## E538：原生全過後目視發現 compact XP 截斷與技能文字重疊（2026-10-09）

實際 ordinary hud-readable-build-v9 nocache PASS70.86秒、refreshfalse；D3D12 hud-readable-native-v6 exact65個名稱全為Success狀態（64無警告、1帶既有警告），failed/notRun/inProcess均0。Primary對owned-summoner-native-v2執行一次capture比對，16張皆為變更或新圖，並逐張以ORIGINAL要求檢視；大型圖在工具回傳後有顯示縮圖，沒有修改圖檔。詳細清單與觀察保留於.ai-collab/hud-native-v6-visual-findings.md。實際responsive-hud-400x300-text125.png的HP／LV／XP行尾只見HP720/1000 LV3，XP40沒有畫出；放大文字的技能／召喚師卡熱鍵與置中的冷卻數字在圖示上重疊。原生fontfloor／fit通過不等於完整文字可見或區域不互相遮擋；尚未接受HUD，沒有寫visual-review接受JSON。

原因：`HeroStatsText` 在肖像欄 `SBox` 的 `MaxDesiredWidth(168)` 裡，沒有 `WrapTextAt`。fixture 字串是 `HP 720/1000  LV 3  XP 40`。文字縮放 1、body 11 時整行進得了 168，所以預設 400 看得到 XP。文字 1.25 的 body 是 `round(11*1.25)=14`，同一行比 168 寬，欄位把尾端裁掉，XP 40 沒有畫出來。當時的測試只查 `Text.Contains` 與字級，沒有查排版後的期望尺寸是否真的畫進欄裡。技能與召喚師的熱鍵、冷卻原本都疊在同一個圖示 `SOverlay` 上：熱鍵左下、冷卻置中。圖示邊長是 `clamp(36*HudScale, 28, 40)`。文字 1.5 的 title 是 21、slot 是 18，`F` 與 `0/12` 放不進同一格。冷卻數字仍是真實剩餘秒數，不是假的 0。

E538 source repair（尚未重跑 build、native、capture、viewport 或 prefix，不能寫成通過）：英雄狀態在原 168 欄內 `AutoWrapText` 且 `WrapTextAt(160)`，不縮小字級、不加省略號、不發明 XP 下一級或上限。熱鍵與冷卻仍是原來的 `STextBlock`，移到圖示下方各自的深色讀數列；冷卻罩留在圖示上。技能冷卻在 fraction ≤ 0 時 Collapsed，元件還在。召喚師在 remaining／total 都是有限值時仍顯示，包含 `0/12`。tooltip、按鈕、啟用狀態、主副綁定沒改。測試改查 paint 空間的期望尺寸、欄寬、字高，以及熱鍵／可見冷卻彼此與圖示不重疊且落在卡片內；compact 召喚師不再用 `Icon+24` 高度把新列判失敗。400 預設與文字 1.25 仍要 fit。400 文字 1.5 不保證。80×60 仍是 explicit no-fit。E536 字級底限與 E537 wrap／fit 門檻保留。沒有新的 automation 名稱或 PNG。清單維持 65 native／165 cases／69 intents／51 stages／16 native PNG／3 viewport PNG。

## E539：display-hud-hide 的遊戲視窗截圖是空的（2026-10-09）

實際 `hud-readable-viewport-v1` TERMINAL FAIL，workflow 92447 CLOSED。cases 159、catalog compatible true、wire intents 69、stages 51。最後案例是 `display-hud-hide`，失敗文字是 `display-hud-hide game window screenshot was empty`。native+workflow false，authority_ack false。Publisher、editor、profile cleanup 都 ok 且已退役；publisher cleanup listener released true、authority false。沒有新對局。後面 6 個 display case 沒跑完，3 張 display PNG 沒有全部產生。沒有 HUD prefix 審查或接受。native、workflow、UE log 與證據保留。

`OmOwnedViewportIpcTests.cpp` 的 `CaptureGameWindow` 先 `Paint()`，再 `Slate->TakeScreenshot(Window)`。`display-hud-hide` 的 `bHidden` 已經通過：combat 與 minimap 外層 Collapsed，內層 minimap 在 Collapsed 祖先下仍是 Visible，設定按鈕可見，profile `bShowHud` false。不能因此當成全部案例完成或正向 ACK。可見畫面那張截圖在這次檢查之前已經返回，所以失敗的是隱藏後那次；兩次都用同一個空圖訊息，因為 `ActiveCase` 仍是 `display-hud-hide`。

引擎讀取條件：`TakeScreenshot` 的預設矩形是 `GetDrawSize()`（layout scale 乘 local size），再加 `AbsolutePosition - GetPositionInScreen()`。`FSlateRHIRenderer::PrepareToTakeScreenshot` 在繪製前記住 `WindowToViewportInfo`。`DrawWindows_Private` 略過 viewport size ≤ 0 的視窗，而且只畫已經在那張表裡的視窗。readback 只在 `ScreenshotState.ViewportToCapture == &ViewportInfo` 時發生，並把矩形夾進 `OutputExtent`。夾完是空的就沒有 `AddReadbackTexturePass`。`TakeScreenshot` 在像素數小於未夾取的 `OutSize.X * OutSize.Y` 時返回 false。viewport texture 的範圍是 `Ceil(GetViewportSize())`，不是 `GetDrawSize()`。設定面板是 `AddToPlayerScreen`，跟世界與 HUD 在同一個遊戲視窗裡。

E539 source repair（尚未重跑 build、native 或 viewport，不能寫成通過）：仍截同一個 owned `SWindow`，不改成只截 HUD widget，不寫空白 PNG，不拿掉三張必要圖，也不放寬空圖檢查。最多兩次。每次先 `Paint()`，再要求一個內矩形：引擎加上同一組 delta 之後對齊 `[0, Ceil(GetViewportSize()))`。成功仍要 `TakeScreenshot` 為 true、`Size` 兩邊都大於 0、而且像素數至少是 `Size.X * Size.Y`。用完兩次仍失敗就保留 `game window screenshot was empty`，並附上 extent、delta、回傳 size 與像素數。沒有 sleep，也沒有魔術偏移。O／N／U、secondary、modal、text 與預設還原都留著。沒有新 case、wire intent 或 stage。

Primary 審查來源後跑新的 ordinary nocache `hud-readable-build-v10`；只有 build 成功才跑 D3D12 `hud-readable-native-v7`，再只對成功輪的新圖做一次比對並逐張看變更圖，然後 fresh viewport-v2 與 prefix。完整驗收留最後。沒有 PASS。下一個實際錯誤編號是 E540。這次沒有製造 E540。

## E531：HUD 重建遭 NoEngineChanges 拒絕（2026-10-09）

實際 `hud-readable-build-v2` ordinary nocache 失敗，exit1／refreshfalse；UBT `FailedDueToEngineChange`，4.37秒。完整原始證據 `.ai-collab/hud-readable-build-v2/{build.log,UBT-Log.txt,summary.json}` 保留。UHT 產生1個檔案後，UBT 判定既有引擎 NetCore DLL/PDB/object/library 與多個 `.modules`／version 需更新，因 `-NoEngineChanges` 拒絕；這不是新的 C++ 編譯診斷，也不能據此宣稱 E530 已通過。使用既有明確 `--refresh-engine --disable-action-cache` 流程更新建置產物，之後另以新的 ordinary NoEngineChanges run 驗證；不手改引擎來源、設定或 BuildId。刷新與後續驗證尚未完成，無 PASS。E530失敗紀錄不覆寫。Primary 首次紀錄時猜錯 Markdown 標題，apply_patch 在寫入前失敗；讀取實際標題後才重試，避免以猜測內容當 patch context。

## E351：續作擴大新機原生驗證（2026-10-07）

- 最後查active_job_id時primary誤把Lua JSON null sentinel當nil，assert失敗但state JSON本身合法且active_job_id為null；讀正式json.lua後用j.null核對PASS，不修改合法state去迎合錯誤assert。

- 最終native-v4真70名字完全一致／70Success（63無warning＋7有warning）／failed0／notRun0；mesh生成與插值、BlueprintAssetLoad均PASS。正式官方commandlet同一69 RustBP資產清單69/69完全一致／0compilerErrors／0compilerWarnings／0failedLoads／exit0，Content Git狀態無變更，未save資產或新開對局。正常gate12.27秒、source-refresh SCW4.14／OmGame15.55／正常2.70秒全PASS，SHA一致且checkpoint已退役。此不能當vendor authenticated MCP／same-PIE／任意舊native override迁移或LAN全驗完成。

- Grok只讀review run-muxkr8we-a4gd8u／thread dbb38f2c-f583-4c6c-bfdc-d8bd7d4c27b3 completed586秒／USD0.51144432（input254003／cache1545856／output37219／reasoning32797／total1837078／calls28／apiDuration未知）。Bridge摘要只回初始句無verdict，因此只讀公開storedJob.result.rawOutput取真final：兩項fixture缺陷與native component rename序列化限制。Include已由primary修正；時序缺陷native-v3真70項中69Success／1FAIL亦重現。先前frame之後無presentation Tick不能假設已有visual sample，已補真Tick再capture，不改production插值行為或弱化assert。v3失敗留存，正常v4重編12.27秒＋SHA PASS，新70項執行中。Native舊component override迁移未驗證，不以compile/load代稱任意舊override完全保留，不增加未有實際資產證據的全域redirect。Primary在serialization替代方案分析花過多時間且未及時update；後續先做已知必要測試、限定source證據，不持續猜想或擴大schema修復。

- Primary新增spawn回歸漏直接include OmFallbackUnitActor.h，source refresh143.95秒在OmEditorAutomationTests.cpp C2065失敗；前三actor／NetCore已正式編譯，不能冒稱整體成功。已補直接include，不依unity/transitive include。checkpoint保留，正式refresh續建，失敗log原樣留存。

- Native mesh header變更觸發正式 UHT200.71秒後 NoEngineChanges 拒絕（總227.82秒，尚未C++編譯）；UBT真列表含NetCore DLL/object與266個modules metadata。不能當C++通過，也不能改manifest/BuildId。既有bootstrap只在Editor缺失或pending時初始化；新增明確 --refresh-engine 強制正式source-only初始化，原bootstrap/default行為保留，installed／缺source／verify-only拒絕與1..64限制保留。純planner五個新refresh情境連同既有tests PASS，官方refresh最多4actions進行中，完成後必須正常gate+SHA才退役checkpoint。第一次PS replace又猜錯flag宣告，guard拒絕且未寫檔；讀真local宣告後用精確patch。

- 第二個限定writer run-muxkd4hp-2pr8ke接近8分鐘仍無更新，bridge取消，PID33852／43676退出，OmRuntime仍零diff，成本未知。Primary接手實作三actor來源與實際spawn/interpolation fixture；NativeBodyMesh／OmNativeBodyMesh分離SCS名稱，legacy authored mesh優先且nativefallback確定解析，construction隱藏重複fallback，不改uasset或redirect、不修改gate。正式重編進行中，另交Grok只讀具體diff審查run-muxkr8we-a4gd8u。

- BodyMesh首次Grok run-muxk10kp-x0exso超過8分鐘無新進度，primary透過bridge取消；原PID22840／36392已退出，OmRuntime無任何diff，成本未知。改fresh限定三個actor來源，run-muxkd4hp-2pr8ke，要求native property／subobject避開SCS名字並保留legacy authored interpolation；不並行writer。官方CompileAllBlueprints負驗證69/69清單完全一致，46errors／0compilerWarnings／0failedLoads，main46／程序exit1；完整log留存，後續stdout重導避免大量輸出。

- 補跑真兩項Playable：ProjectDefaults PASS，RustBlueprintAssetLoad FAIL46 native Blueprint compiler errors（23個creep／summon，SKEL與generated class BodyMesh跟native反射property撞名）。不把asset-load exit0當成功；report才是事實。交Grok run-muxk10kp-x0exso以共用native類別修正，禁止逐asset繞過或弱化測試。前次一次印全46error與source又截斷；後續只取錯誤總數與代表例。

- 正式 NoEngineChanges build exit0，83.75秒／13 actions，OmEditor／OmGenerated／UECPShell重新編譯連結，Rust stage兩個SHA一致。新native-v2真執行67項Success，60無warning／7有warning／0fail；warning包括拒絕壞frame的預期診斷與無WorldContext fixture cleanup，不得宣稱warnings0。
- Primary 原69項預期與67實跑不符；讀正式UE AutomationCommandline.cpp才證實 StartsWith 自動加句點，完整leaf名稱不匹配。兩項Playable改substring完整名獨立補跑，排除Linked TD；原報告保留。Report 的 succeededWithWarnings必須加總，warnings不存在不可當0。一次又猜Engine AutomationTest.cpp路徑不存在，已只用實際已知AutomationCommandline.cpp來源。

- 廣範圍 Grok run-muxix87x-ws3x9s 約19分鐘仍未修改來源，已透過 bridge 取消，原 PID 428／43552 均退出；成本未知。改派單一 selection fixture，run-muxjo6as-63gdpk 完成82秒／USD0.07898812，只改 TArray Add 的獨立 shared-pointer 引數，原 duplicate-ID 拒絕仍保留。
- Primary 分別修正 missing catalog metadata 不得創造作者 ID、HUD fixture 保留 Slate lifetime、抽象 UObject fixture 改 concrete SceneComponent、float midpoint 用 ULP 比較；compiled DLL 的 reload 測試依既有 Rust 契約驗證 ReloadDisabled、catalog/hash 不變與 malformed authoring 不影響 runtime，未恢復 gameplay Lua。正式重編與新測試尚待完成。
- 本次一次把 state 的巨大單行與其他結果合併輸出又截斷；後續用解析後的指定欄位，不整行 rg state。

- Grok只讀telemetry diagnosis409秒／USD0.3517096完成。Primary讀真source核對初始seq<=6與slot0/1/2：七次通過不能當authenticated readiness；200 callback之前仍無真cache。Learning401無法單獨推定過期／missing token。正式SEditorSyncPanel在slot2已建構但Startup直接slot0，支援啟用入口不可达；限定只修UI routing，不改licence／telemetry／grace／protected validation。
- 本輪primary再次預猜OmRuntime/Private/OmWorldBridgeActor.cpp不存在；rg --files找到真OmGenerated/Private位置。模組名OmGenerated也不代表檔案由codegen生成：正式manifest17 outputs沒有WorldBridge，文件不能宣稱它一定會被codegen覆蓋。一次過寬rg codegen生成關鍵字又截斷；後續只取具體function與正式manifest。

- 新機57項Om.Generated／Om.Runtime與兩項Playable只讀fixture驗證，排除會啟動舊TD模擬的LinkedSinglePlayerLockstepSmoke。實際exit1，SelectionReplyContract以Heroes.Add(Heroes[0])alias儲存導致Array assertion崩潰；另ActorContentRebinding含抽象UObject allocation與fallback invented content失敗、NativeAttackPhaseCursor midpoint、OwnedInputUnavailableHud通知計數及舊RuntimeLuaReloadSmoke期待失敗。不得套用先前MCP12PASS宣稱遊戲native全通過；失敗log保持game-native-tests-machine-v1，交Grok依現行契約修正，不恢復gameplay Lua或鬆動parser。
- 修復packet初稿誤稱Actor allocation，讀實際3407行後改為NewObject<UObject>與獨立內容身分失敗；HUD數值也是notification count，不是mask。必須先區分fixture問題与production契約問題，不能統一改期望掩盖缺陷。
- 完整tasks.md讀取再次截斷（歷史32093tokens）；改rg只列四個未完成checkbox與指定區塊。禁止把截斷輸出當完整審查。Bridge即使現行job只讀也拒絕同workspace新writer；等待診斷job完成後才dispatch修復，不另建bridge資料目錄繞過guard。更新本節第一次又猜錯MD標題而patch拒絕；已讀實際首行後才重做。

## E350：新機原生介面、無目標技能與 UE5.8 header 驗證（2026-10-07）

- 最終state用PowerShell ConvertFrom-Json可讀不代表固定Lua嚴格JSON也可讀；保留machine_continuation行尾comma後根object出現trailing comma，實際固定Lua讀取失敗expected JSON object key。已只移除根物件最後comma、保留所有歷史欄位，再用固定Lua真解析PASS；後續更新state必須以正式consumer驗證。

- 最新正常NoEngineChanges gate成功135.48秒，五個晚修改native來源正式編譯且UECPTools／UECPMCPBridge DLL完整連結；SHA stage一致。全新native-v3 report succeeded12／failed0／notRun0／warnings0，log十二項Success（含ToolsListUnion／runtime gate diagnostic／planner），不是把舊十項套到新程式。
- 自有Editor8988／token134358151719249468實際health與owned tools/list PASS，capture有1064排序去重工具與完整native provider。Primary核對時又猜play_in_editor／stop_play而失敗；真名稱是preflight.required_tools的begin_play_in_editor／stop_play_in_editor。finally已退役原owner，之後只核對原live capture並保留first-validation FAIL，不重開Editor取grace、不把offline解析當新runtime dispatch成功。源碼已明確名字時不可手寫猜測清單。
- 一次UBT進度查詢讀了LocalAppData舊Log.txt，实际属于Creative3D旧建置；没有继续读取／操作该项目。正式当前stdout明确给出D:/UE5.8/Engine/Programs/UnrealBuildTool/Log.txt，已依真实路径保存本轮135.48秒报告。不能仅凭常见日志路径或cl.exe存在推定当前项目进度。

- Bootstrap Grok1897秒／USD1.38409444完成；原103 checks含16個source文字比對，不能當實際API／runner驗收。Primary刪除這些實作鏡像測試，加入15個真callback/response拒絕情境，102項PASS；既有compile50項也PASS。新helper原unwrap只回第一段JSON，可能遮蔽後面的負wrapper，已改全content負值優先、isError／malformed envelope拒絕；idle同時支援explicit native success／legacy ok，矛盾或unknown拒絕。正式NoEngineChanges重連結已開始，尚無live資產成功聲稱。
- Grok follower30分鐘timeout只結束watcher，worker仍running，不可當取消／完成。重新runs --follow --stream接同job即可；primary誤將--stream與--json同時指定遭CLI拒絕，已更正互斥flags，沒有啟動第二writer。worker後續真正completed1897秒，以completed metrics記成本。

- Blueprint bootstrap packet 曾錯寫 native create_blueprint 回傳 asset_path；實際 registered HandleCreateBlueprintFromArgs 寫的是 success=true／blueprint_path=GetPathName()。Grok讀實際source後修正helper，要求blueprint_path，存在衝突asset_path也拒絕；legacy create_asset才保留asset_path契約。測試fixture必須採真handler欄位，不能以同一份錯誤packet自我證明。

- Fresh catalog Grok922秒／USD0.62874976，四檔完整審查後三個實際UBT-rsp /Zs PASS；新增ToolsListUnion保留原九項，native執行待重建。共享State按值捕獲，atomic release/acquire保護結果，不在worker讀無鎖registry、不LoadModule／未載入extension。Primary補超時／停止後共同檢查：finished跳出while也不可接受晚到成功，deadline覆蓋排程前階段；仍不把前次10PASS套到新增測試。
- Asset bootstrap來源稽核：原生registered list_assets FromArgs回應folder_path／assets／count，沒有success／recursive，且原版本未ScanPathsSynchronous；不能混用另一個direct函數的schema，也不能把尚未scan的空目錄當不存在。find_blueprint_by_parent是substring＋NativeParentClass fallback且上限50，不能證明direct parent。交Grok限定API實際metadata／同步scan與Lua native/legacy能力適配，保留嚴格身份與父類／只save新資產／真正compilergate，不解析人讀summary或猜父類。
- Grok resumed catalog／diagnostics worker約723秒仍只有讀源碼、未改檔；primary經bridge取消並核對兩PID已退出，成本未知。改fresh native-only短packet，Lua診斷由primary獨立修復，不併發改同檔。五十個情境PASS，真正第八項raw回應重播仍FAIL且顯示mcp_runtime_gate_failed；多content正值不能遮蔽負wrapper。Packet第一次截取非空行索引與預期真行號不一致，已改保留空行的索引；worker仍必須讀實際檔案，任何片段不能替代diff審查。
- Live Blueprint報告compile-1791336995：前七項真編譯／gate成功；WBP_OmHudRoot compiler成功0error，但原生top-level mcp_runtime_gate_failed／telemetry_cache_ok=false／isError=true，因此整體FAIL。TelemetryManager.cpp的initial seq<=6寬限與authenticated cache保持原契約，不得unwrap original_response的success、不重啟取寬限、不偽造validation。原Lua只說missing structured result遮蔽真正gate錯誤，交Grok保留負回應diagnostics；未開新場，8/10。
- 先前「原生catalog沒有工具」必須區分advertised與實際registered：TryDispatcherFallback可以派送dispatcher已註冊create_blueprint／list_assets，但HTTP工具清單只掃static Dispatch cases漏列這些真工具。交Grok以GameThread真registry snapshot＋static deduplicated/sorted union補正能力宣告，不能宣告未載入extension或改權限。舊check_asset_exists／get_blueprint_skeleton仍未找到對應註冊，不推定可直接調用。
- Grok冷啟動期限修正393秒／USD0.6041324完成；primary審查parser／consumer及完整新增測試。最後fixture以ipairs({nil,...})會零迭代，8PASS中該組原先未測到；primary拆nil獨立拒絕再遍歷實際其他值，八組重跑PASS。default1800、CLI1..3600秒只控制原owned Editor readiness，真timeout／晚到true／PID重用拒絕，選角與結算期限不變。一次source搜尋又預猜不存在MCPDispatch.cpp，已改目錄＋--glob查真入口，不把截斷Grokprogress當完整diff。
- 首次圖形Editor冷啟動十個owned ShaderCompileWorker仍持續運算，MCP registry尚未初始化；600秒 readiness檢查真實FAIL且沒有對局。原same-PIE啟動固定120秒更會拒絕有效新機冷啟動，交同thread Grok限定通用有界可設定Editor期限（不改選角／結算期限、不跳shader或偽造ready）。保留原Editor pid32780／creation token與快取；另外新檔重試readiness，不能覆寫首次FAIL或把程序alive當ready。
- 原生630工具catalog無舊check_asset_exists／get_blueprint_skeleton／create_asset；create-missing工作流需先capability適配，不能將unsupported當missing並重建。現有163個uasset與真正recipe兩個Blueprint仍須實際編譯；training英雄沒有Blueprint path，不造空包裝或假skeleton。
- 文件稽核再次出現 primary 預猜 MCPDispatchCatalog.inl 不存在及長歷史 rg 輸出截斷；已以 rg --files 定位真 MCPStaticTools.inl。後續按具體欄位／短區塊查詢，截斷輸出不算完整審查。不得因既有錯誤已寫入 MD 就宣稱操作上已避免重犯。
- 相容宣告修正後 native-tests-machine-v2 實際成功十項（HTTP九項／PIE planner一項），report index.json succeeded=10、failed=0、notRun=0、succeededWithWarnings=0；引擎 log 十條 Result={Success}，程序exit0，不能混用首次零測試結果。另 primary 提早啟動 runtime 與尚未結束的 UBT 重疊，AutoSDK probe遭 mutex conflict；正式 gate 隨後9.32秒成功、Editor繼續載入並完成十項，後續依賴 runtime 啟動必須等待建置 terminal 再開始。
- 原生 automation 首次程序 exit0，但 log 明確為 NoTestsMatched／0 tests performed，不能算十項通過。外掛宣告5.8.4遭 unattended 相容提示拒絕載入；EngineVersion.cpp207 明確非 licensee 的 CompatibleWith 使用 patch0，實際 runtime Current=5.8.4／Compatible=5.8.0。更正此前「應宣告5.8.4而非5.8.0」的錯誤推論：外掛宣告5.8.0相容基線，實際引擎仍5.8.4；不改引擎版本、BuildId或略過相容檢查。首次零測試 log 保存在 ignored native-tests-machine-v1，重試使用新目录並核對實際十項結果。
- 正式 source bootstrap、OmGameEditor link、正常 NoEngineChanges gate 均已成功（session71699，正常 gate6.49秒），SHA stage一致、初始化 checkpoint 已移除；之後單獨無 bootstrap gate session92230也成功9.27秒。這是完整建置成功，仍不等於 runtime／Blueprint／對局通過。
- 完整2089-action retry 結束4985.07秒，唯一新增錯誤為 Gemini history field key；Grok 約116秒／USD0.37517572 的單檔 +2/-1 完整 diff 已審查，Owned FString 只用於 AllowedFields.Contains，SetField保留原 shared key、四欄白名單與角色排除不變。實際 UBT-rsp MSVC syntax exit0。Primary task 初稿把另一檔 KnownToolNames 套到本欄位 allowlist，讀實際區塊後更正 task/state；審查以真正 source 為準。保存完整失敗 log，同 checkpoint 正式 entry session71699 重試；不清理快取、不假稱 full build成功。
- 正式2089-action retry 在約2005時發現 UECPArchitect/FUECPGeminiAgentLoop.cpp196 新 C2664：JSON shared key 傳入 TSet<FString>::Contains。交 Grok 限定該檔以 Len-preserving owned FString 遷移；保留 allowlist／hidden filtering／object requirements，不設 legacy define。其餘 build繼續完成，之後同 checkpoint 正式 retry，不能把 MCP 模組已連結當成整個外掛已成功。
- Blueprint provider Grok 完成約289秒／USD0.31477404；完整兩檔 diff 審查，primary 固定 Lua 四十五情境PASS。positive success／ok 相容，任一 false／nonboolean 拒絕，既有 compiler／health／identity／transport gates保持。另 git HEAD 舊 native stop 的明確成功回應沒有 request_accepted；primary 保留 absence 的 legacy positive 相容，explicit negative／malformed仍拒絕，必須輪詢同 Editor 真 idle，新增兩情境後 lifecycle十五PASS，非用 transition_observed 代替觀察。
- Source-proven Blueprint provider mismatch：AssetManagementTools::HandleCompileBlueprint 的原生 result 明確 success=bSuccess、error_count／blueprint_path 等，沒有 ok；HTTP wrapper 不合成 ok，但 Lua M.result 強制 ok==true，將拒絕真成功。交 Grok 限定 consumer／pure tests 相容 positive success 或 ok；任一明確負值／矛盾／malformed flags 一律拒絕，保留 compiler／graph-health／identity／transport gates，不合成工具成功、不改 Blueprint 或 engine。另 Windows literal glob 再次造成 rg 123；固定以目錄＋--glob 查找，定位與讀檔分開。
- Extra-world repair Grok 完成約473秒／USD0.46335404，僅 transient override 與既有 planner test；primary 核對 public bLaunchSeparateServer=false 與 applied check、scratch=true 回歸、global flag snapshot 不變，兩檔實際 UBT-rsp syntax 再次PASS。SDK ClampPIEClientValues 上限 MAX_PIE_INSTANCES-1，實際 LazyObjectPtr.h 定義11，故十個 client 的 fixture 合法。Primary 在同一命令列先列檔又預猜 SettingsClasses.cpp 路徑失敗，實際多一層 Private/Settings；相依定位／读取必須分次，不能假定工具前段結果已可供後段固定命令使用。
- SDK PlayLevel.cpp2840-2909 實際 bNeedsServer = PIE_Client OR bLaunchSeparateServer；standalone 也會繼承個人 extra-server 偏好並新增 world。只設定 mode／clients／one-process 不足以保證 root 一個 PIE world。交既有 Grok thread 限定修 transient request override／原生 planner fixture，明確 overrides 排除 inherited optional extra server，client mode 的必需 server 保留；無參數與 global CDO 不變。不重啟引擎建置圖、不新增 module/file。
- Primary 補查 deadline 邊界：idle response 在期限之後才抵達時，原 helper 先接受 false 再查時間，會誤報及時停止。移動有效 clock／deadline 檢查到接受 idle 前，加入實際延遲回應 mock，十三情境PASS。HTTP 單次 request 仍使用既有20秒 timeout；此10秒是 stop 接受後的觀察期限，不宣稱包含初始化／HTTP 的硬總時限。另一次 rg 猜 ue_mcp_http.lua 不存在；實際 HTTP 工作流在 ue_mcp.lua，後續用 rg --files 定位。
- PIE lifecycle Grok run-muxacrfr-zpq5ln 完成1570秒／USD2.10597972；primary 已讀完整設定 planner、transient duplicate、dispatch 與 cleanup helper。獨立 Lua 驗證發現 test 缺 end)（line136），修正後12情境PASS，capability19情境仍PASS。原生新增測試尚未執行。Primary 又猜錯 ToolsLevelActorTools.cpp；應先 rg --files，實際為 Tools/LevelActorTools.cpp。多個歷史長行合併讀取再截斷，不能當完整審查；後續分段讀實際差異。
- 新增 AutomationController 模組相依與 PIEPlaySessionPlanTests.cpp 後需要重載正式 UBT graph；只核對並退役本次 PID11764／parent8144／birth2026-10-07T04:22:57.651764+08:00 的 OmGameEditor build，保留 objects、checkpoint 與舊 log，再同入口重試。停止原因為建置圖變更與已修復失敗單元，不是 HDD 低CPU或速度；不清理引擎、不改 BuildId。
- 原生外掛 descriptor EngineVersion 仍5.7.0，Installed=true。已核對實際 UE5.8 PluginManager.cpp2687-2720：會比較 CompatibleWith 並彈出 attempt-load-anyway，unattended可能不載入；不能把新編譯binaries當成descriptor已符合。待真正 full build/link成功後，將此已移植fork的 EngineVersion 宣告更新到實際驗證SDK5.8.4（不是猜5.8.0、不是改 UnrealEditor.modules/BuildId 或設置 ignore-version），再跑runtime驗證。其他patch／SDK仍需正式移植，不自動偽造相容版本。
- PIE capability Grok 完成1086秒／USD1.4063182，primary 審查完整 root/helper/native diff，固定 Lua18情境實際通過；static catalog生成630項。Primary 審查補強 count-vs-busy-bit 相容：有效正整數 count一律busy，負／小數／非有限值拒絕；optional framework/controller busy／unknown 不得被 running=0 覆蓋。新增矛盾來源／未知狀態回歸後十九情境PASS；native AutomationController include／link尚待新UBT graph，非runtime驗收。Follower終局progress輸出曾被1400 budget截斷；實際檔案diff與公開metrics已另完整核對，沒有從lossy progress提取程式。
- Grok SDK 稽核更正 primary task 的另一個未驗證 API 假設：UE5.8 FAutomationTestFramework 沒有 IsAutomationTesting()，需用實際公開 GIsAutomationTesting／GetCurrentTest 與已載入 controller GetTestState。不得將熟悉的舊 API 名稱当成已核對的現行 API；helper 的 source metadata 只列實際讀過的狀態，不能宣稱呼叫不存在的方法。
- Primary 再次猜錯 SDK Public/PlayLevel.h；實際檔只在 UnrealEd/Private，公開 request struct 應先由 rg --files 找到 Editor/EditorEngine.h 再查符號。只有 include 名稱不能推出磁碟 Public 路徑；未由工具回傳的完整路徑一律先列檔確認，避免繼續把既有錯誤紀錄當作防錯已完成。
- Same-PIE source 契約稽核另有兩個確切問題，需現行 Grok 完成後限定接續修復：LevelActorTools::HandleBeginPlayInEditorFromArgs 忽略所有 Args，使 root num_clients=1／net_mode=standalone 依賴 Editor 個人設定；HandleStopPlayInEditor 呼叫非同步 RequestEndPlayMap 卻用同 tick PlayWorld!=nullptr 回傳 success=false，root mcp helper會立即 throw 且將已自然結算的run判失败。應以 request-specific settings override 保留無參數舊契約、正確分開接受請求與觀察到停止，root cleanup有界輪詢原 Editor 的真狀態；不能強行把 observed=true、變更永久偏好或把 stop_requested 當完成。來源範圍與 regression 細節待下一 task，現有 job 不併發改同檔。
- 等待建置時核對正式 same-PIE 入口，發現 source-proven 契約缺口：第一個 enable_extension 是 write，違反本機原生 read-first unlock；此外 native source 內建全部 PIE tools，但沒有 enable_extension／get_automation_test_results 這兩個舊 provider API。交 Grok run-mux9m9wy-15dji9／threade4253909-d028-4ba5-b493-f208ca8643fb 做 capabilities-aware owned preflight，保留舊 extension-backed provider 並新增真正 framework/controller busy-state 只讀工具，不能固定回傳 running=0、不能省略 busy gate、不能移除 license／permission／read-first。CLI 實際旗標是 --list，非 primary 初步想像的 --list-tools。新增 AutomationController module 依賴後需正式重建 UBT graph；review 完成後只退役目前自有 build，保留已完成 objects，再同 checkpoint 入口重試，非因 HDD／低CPU重啟。
- Architect 同類修復完成：Grok 184秒／USD0.2169234，單檔 +2/-2，以 Len-preserving FString 一次轉換且 allowlist／hidden filters／synthetic tool wrapper 保持；primary 完整 diff/check 與實際 UBT-rsp MSVC /Zs exit0。合計二十個直接修改的 native translation units syntax PASS，仍非 full link／九項 automation runtime PASS；UBT 先前七檔十六個錯誤待正式重試。
- Architect 驗證命令將 workdir 設 Engine/Source 卻給 git -C 相對 repo 路徑，兩個 diff 讀取失敗，但後續 cl 照樣啟動；primary 隨即改實際 absolute repo 路徑讀完整 diff/check。PowerShell 的 native exit 非零不會自動阻止後續語句；相依步驟須顯式檢查 LASTEXITCODE，跨 workdir 的 repo 操作一律 absolute。
- 完整 UBT 約918/4046 揭露 Shell Architect 的 C2664/C2440：single-key JSON object 的 shared key 直接用於 FString set／copy-init。交 Grok run-mux8u842-5g9wef／thread6cc81001-13ef-4697-822a-1fb123c5bf70 限定此檔同類遷移；KnownToolNames allowlist、object requirement、hidden filter 與合成 tool wrapper 不得放寬。十九個已改檔 syntax PASS 不等於其餘未檢查來源相容；目前累計16個舊編譯錯誤，需正式 retry。
- 等待建置時讀 OpenSpec tasks 的前140行，因歷史長行造成8774-token output截斷，這是 primary 可避免的過量讀取；不能聲稱已讀完整。四個未完成 checkbox 的 rg 行號輸出完整；之後短行段仍達4642 tokens再截斷，證明限制行數不等於限制字數。後續只取需求 checkbox；必須讀歷史時，除定位與行數限制外也限制每行字數，不再完整印出長歷史行。
- 六檔實際 UBT 錯誤修復完成：Grok 750秒／USD0.95166204，六檔 +43/-18，完整 diff 經 primary 審查；shared-key 使用長度保留轉換，WidgetTree 先快照 keys 再修改且使用 TryGetField，StringTable 保留原 DevNotes，enum 五參數依 SDK 舊 overload 的 int64／None／add-MAX Yes 契約。六檔各自實際 UBT-rsp MSVC /Zs 皆exit0，合計十九個直接變更的 native translation units 前置驗證通過。正式 UBT 已失敗的舊單元仍需完整 build 後重試；目前 full link／九項 native automation 仍 pending，不將 /Zs 當成執行驗收。
- Primary 又猜了 json-key-scan-v2/ProjectScanTools.rsp 與 omfue/AGENTS.md，兩者不存在；實際 rg --files 確认 rsp 在 v1、AGENTS 僅 root。不能將前一輪「已記錄」當成已避免；後續必須在同一次使用前先確認路徑。同步根 AGENTS／Lua migration manifest 的舊 D:\code 絕對路徑為 clone-relative 固定 runtime，版本與禁止 fallback 規則保持。
- 完整 UBT 在約500/4046 時揭露另外六檔十四個實際錯誤：DataTableTools 的硬編碼 FString JSON map、Niagara／PCG 的 shared-key、WidgetTree 的 key snapshot／lookup、StringTable editor-only 三參數 API、UserDefinedEnum 五參數 override。交 Grok run-mux7r9nz-oic4d1／thread0fbb3ad4-28c1-426b-b91a-a44f742af8e5 限定六檔通用修復，原引擎建置持續保留成果，之後需正式重試失敗單元。Primary 初始 task 將 StringTable 舊 overload 誤稱 legacy define；立即按實際 SDK 更正為 WITH_EDITORONLY_DATA 分支，要求保留既有 DevNotes，不臆測 enum underlying／MAX defaults。
- 三個工具同類修復完成：Grok 344秒／USD0.41818912，三檔 +25/-19（JSON key 轉換保留完整長度、搜尋／reserved filtering／slot normalization／connection 原case與pin split）。Primary 審查完整實際 diff，ProjectScanTools、WidgetTools、BlueprintGraphTools 重新實際 /Zs 皆exit0；連同 ImportTools 與先前九個變更來源，共十三個直接語意變更的 native translation units 前置 syntax 通過。完整 UBT link 與九項 automation 仍未執行。
- 同類交叉檢查：先區分普通 FString map 與 FJsonObject key，未盲目替換所有 .Key。三個實際 JSON 工具來源 ProjectScanTools／WidgetTools／BlueprintGraphTools 的 /Zs 皆重現 shared-key 型別錯誤，交 Grok run-mux6wbhg-5ci3sq／thread4d3cb255-aac1-4b21-add4-7b86c789d6ef 做同類遷移；普通 Sections map 保留。既有 animation import 修補來源 ImportTools 實際 syntax exit0。先前九個 syntax PASS 為已檢查的九個變更來源，不含當時未檢查的 ImportTools，亦非全外掛已編譯。
- JSON shared-key repair 完成：Grok 361 秒／USD0.3159552，兩檔十處實際 API 錯誤以 ToView 搜尋及 Len-preserving owned FString 修复，保留 case／排序／完整 key 與 ReservedTopLevelKeys 規則。Primary 審查 diff、核對官方 StringView 的 StartsWith／Contains 契約，兩檔實際 syntax exit0。連同先前七項來源，本輪直接變更九個 native translation units 均通過 /Zs；這是 source/type/API 驗證，仍非 DLL link／九項 automation runtime PASS。
- 再次檢查結果：MCP subsystem／helper／九項測試來源 syntax 通過；六個 include 修正檔中四個通過，Scanner 與 ToolExecution 真實 C2039/C2440/C2665/C2664。來源核對為 FJsonObject::Values 的 key 改成 UE::FSharedString，非 AssetRegistry tag（primary 初步口頭分類不精確，已更正）。官方 JsonObject.h 預設新 key API；Grok run-mux6eqy5-yw3zbk／thread89c4f088-7304-4d37-866f-bb312bc4676a 限定依正式 public string API 修復兩檔，不開 UE_JSONOBJECT_LEGACY_STRING_KEYS 暫時開關。搜尋／reserved key filtering／排序／case 規則與完整長度須保持。
- include 修復後實際 MSVC 又揭露 C7595：舊 MCP write-lock rejection 的 FString::Printf 是純字串、沒有格式符卻多傳 CommandType，觸發 UE5.8 format sanitizer。Primary 改直接常數字串賦值，拒絕訊息與權限維持原樣；再次 syntax 檢查中，沒有關閉引擎格式檢查。
- 原生 socket 細節按 UE5.8 本機 SocketsBSD.cpp 核對：STREAM Recv true/0 表示非阻塞 WouldBlock，false/0 表示 EOF／錯誤。不能用可能殘留的 GetLastErrorCode 把 EOF 判為 WouldBlock；receive 與 mutative disconnect guard 已按回傳契約修正，仍待原生實測。Grok 完整公開 result.rawOutput 可用於審查，progress log 有去重／斷行不能作程式來源。九項 native tests 尚未執行，不能把加入測試當成通過。
- 正式 Lua --bootstrap-engine 本輪 ShaderCompileWorker 284 actions／429.02 秒實際 Succeeded；OmGameEditor 自動接續。初始化失敗 checkpoint 由同引擎重試使用；已有 exe 不等於首次建置完整，只有正常 NoEngineChanges 與 SHA stage 成功才清 checkpoint。

- 真人 R 候選原因已從來源定位：renderer input bridge 的 AbilityCast 會附帶 target_pos，game_processor 不看技能 metadata 直接轉 Point；HealSelf 要求 None。原 headless 直接 None 未覆蓋輸入邊界。交 Grok job run-mux4tujj-gz6di9／thread550392df-6253-4c5e-a241-38d33fd43d85 實作 authoritative metadata 通用正規化與真正輸入→效果回歸；原 tick83725 尚未實測重現，不能提前宣稱修復。再次搜尋猜 heroes／script_api／renderer_decode 不存在及 Windows literal glob 造成失敗；這是 primary 重犯，後續只能使用已確認目錄或 rg --files。

- 無目標 cast 通用修正已獨立審查：依 authoritative ScriptRegistry TargetType::None，於 rank/mana/cue/hooks/ABI 前正規化；不改 HealSelf 契約，不以 zero coords 判斷，不加英雄特例。新真輸入→settlement 回歸與原四技能 headless 2/2 通過（209 filtered）；獨立停用該呼叫，新增回歸實際 FAIL，自療 HP 保持200而不是270；finally 還原來源 SHA 與審查版本一致，重新2/2 PASS。這是有效來源缺陷證據，仍不可宣稱真人 tick83725 已 replay。受影響四種Rust產物重建中，舊 stage SHA 只代表修正前版本。

- 修正後正式重建結果：release backend/config 5m07s、release client runtime 2m51s、release script DLL 2m07s、debug script DLL 39.86s 均成功。固定 Lua 正式 --prepare-only/--no-build 配置預檢 exit0，1 local human seat／9 Bot／60Hz，僅配置，不是選角確認或對局。bridge 正式重建／SHA stage 進行中；模板 dead-code 與 compiled-only 下既有 unused warnings 不當作此次新增錯誤。

- 修正後 bridge 正式129秒建置成功，cbindgen header unchanged；debug兩份DLL及bridge正式stage一致。verify-staged-only實際exit0，新bridge SHA099fe318ff63030274dcf3f267507fd51da394e1a6fe75462c9bd68961b29ab0、script SHAe6610cbd847a4e0286c6156c1b965fed7321cf5e150223c57310c807bc083621。此前SHA為修正前版本，已被此結果取代；Editor build與九項原生驗證仍待，不提前清checkpoint。

- 使用實際 UBT response include/define 做 /Zs native syntax 前置檢查；副本移除 Fo/Fp/Yu/sourceDependencies/SARIF 輸出，存ignored目錄，未改UBT正式rsp或objects。WITH_DEV_AUTOMATION_TESTS=1已核對：HTTP helper與九項測試來源實際syntax通過；MCP_EditorSubsystem遭fatal C1083（Engine/UserDefinedStruct.h已不存在）。引擎正式檔案實際位於Runtime/CoreUObject/Public/StructUtils/UserDefinedStruct.h，外掛已有正確引用；Grok run-mux62dxf-n7kgft／threadb0065ccd-6801-4336-8caa-84a3cf2b4ab6 限定修正七個來源檔九處舊include，不造stub或改include搜索路径。模組link／automation仍未驗證，不把/Zs當完整build。

- 紀錄操作：Add-Content 曾把本輪新機項目接在歷史 E331 尾端，已按唯一內容 anchor 移至本節；新錯誤應使用明確編號標題，不混入別次工作。搜尋也再次猜不存在的根 Cargo.toml／scripts/run_session，應先列實際檔案。moba-config --help 回傳 usage exit1 是目前 CLI 契約，正式 --prepare-only 預檢 exit0 才是成功證據。

## E349：新機接續先核對實際路徑與建置產物（2026-10-07）

- 目前 clone 是 `D:\omoba`，不是原機 `D:\code\omoba`。`scripts/_bootstrap.lua`、`tools/lua/lib/path.lua` 與 `platform.lua` 寫死原機路徑，會阻擋 bundled lfs 與子程序啟動。通用修正交給 Grok Build，必須由目前來源位置推導固定 Lua runtime；不搬倉庫、不用 junction、不加其他語言 fallback。
- 首次 fetch 的遞迴子模組歷史 ref 缺失，不等於正式 gitlink 缺失；主庫 fast-forward 後依新 `.gitmodules` sync/update，全部正式子模組取得成功。原兩筆數值修改先保存，使用者明確要求放棄後 restore 並移除該次 stash，未刪除既有 `graphify-out/`。
- 使用者指定引擎 `D:\UE5.8` 正確，但來源 checkout 的 `Engine/Binaries/Win64/UnrealEditor.exe` 尚不存在（只見四個 DLL）。不能把引擎目錄存在當成可啟動 Editor；已使用引擎正式 Build.bat 建置 UnrealEditor，不偽造 BuildId、不移植舊 binaries。
- 操作錯誤：讀取前猜測 `omfue/restart/src/build.rs`，實際建置邏輯在 `lib.rs`；再次提醒先 `rg --files`。數次一次輸出過長造成截斷，後續以行段和數量摘要檢查，不宣稱已審閱截斷內容。
- 恢復沿用原批次 8/10、OpenSpec 27/31；未開新場次、未宣稱真人 R 已修復、LAN／固定十二效能驗收仍未通過。後续將實際建置與回歸結果補在此節。
- Grok 首個 write worker `run-mux177lo-cc6trw` 讀取約 361 秒沒有補丁，primary 經 bridge 取消，terminal cancelled、原 agent/bridge PID 32560/25524 已不存在；成本未知。改完整來源片段的 no-tool 提案 `run-mux1gm3i-dcbx43`／thread `26a5090c-c966-4198-b987-fab4e08c8215`，407 秒完成、USD 0.07486052、API duration 未提供。提案字串出現非預期斷行，primary 審查後整理程式並整合，不直接把輸出当可編譯補丁；Grok 沒有執行測試。
- 通用工具修正：platform 從自身檔案定位 bundled Lua/lfs，bootstrap 共用同一來源根；repo_root 預設來源根而不是 caller cwd，接受 worktree `.git` 檔案。四個根目錄 `.bat` 使用 `%~dp0tools\lua\lua.exe`，保持 CRLF/無 BOM/參數與 exit code。primary 四種實際搬移 fixture、Lua 模組全套與四 launcher PATH/身分測試通過；從 `C:\Windows` 呼叫真正 PIE `--help` 通過。
- 首版 portability fixture 用 `os.execute` 遇到空白路徑引號與 CMD 對 extended cwd 的限制；改用既有 native process.run 傳結構化 argv，不改正式 shell quoting 或加 workaround。真正入口也揭露 host 的 repo_root 依賴 cwd，補修後才接受提案。再次誤用 Windows 路徑內 `*` 與 `rg -ErrorAction`、猜不存在的 role build/targeting/config 檔；這些是 primary 操作錯誤，之後必須先從實際檔案清單取得路徑、rg 選项不可混用 PowerShell 參數。
- Rust 元件安裝完成且 `rustc +1.95.0 --version` 確認；rustup 最後 self-update 因 updater 檔案缺失 exit1，與已安裝編譯器分開記錄。檢視 toolchain list 又觸發第二個 lazy install，已按原程序身份停止重複下載，改單一顯式 pinned install，避免多個下載共用 partial。Lua host 3m43s 與 restart 21.85s 建置成功；四技能 compiled-only headless 1/1（209 filtered）通過，首次編譯 7m09s，不是原場 RErr 已重現或修復。
- bridge 所需 `cbindgen` 本機缺失，從 Cargo registry 核對並安裝明確 0.29.4／locked，2m33s 成功；新機建置必須包含該工具。
- 引擎 UBA 只執行一個 compiler，原始設定 MemStartWaitPercent=85，當時 commit 39577/46352 MiB 觸及門檻。primary 只終止原本自有 UBT/compiler lifetimes，原 UBA 場至 662/9023 中止，改官方 `-NoUBA -MaxParallelActions=4` 接續（UE5.8 仍用 UBA scheduler，NoUBA 只關閉 detour，不是舊 executor）；不改系統 pagefile、引擎來源或記憶體門檻，舊回傳非零保留，不能當引擎建置成功。

- 新增精確檢查：Git 的 BpGeneratorUltimate 1.4.0 只有 raw TCP MCP 與掃描端口的 Python proxy，缺原機 2.0.6 HTTP／專案 registry 契約；StartServer 還包含按端口殺其他 PID 的舊流程。禁止為就緒繞過專案／lifetime 判定或開 Editor 執行這段流程，須先移除殺端口行為並補原生專案綁定相容介面。Grok 新任務被 bridge overlap guard 拒絕，先等目前 proposal 結束，不另造追蹤目錄繞過。
- 原獨立 all-modules target 第二次至 42/8360 仍慢；只停止自有 UBT 9404 與其 cl children，改官方 OmGameEditor 專案 target，4204 actions。正式 Lua 已補 --bootstrap-engine，source／installed engine admission、並行參數上限與唯讀 verify 測試通過；實際首次 UBT 還在進行，不能把 planner 測試當作引擎已完成。
- Grok bootstrap proposal run-mux2cls4-71juov / thread29f9c164-634a-4e55-be3c-de496f648111 約444秒產出提案；follower 收到 Grok finished 後 exit1 / No run found，jobs JSON 仍停 finalizing、原兩個 PID 已消失、registry index 丟失舊 jobs。只採用可審核公開提案，terminal/cost 未提供，不手改 bridge runtime 或冒稱 terminal success。新的 MCP implementation job run-mux2n04e-631s5b / thread51032637-5102-4df6-a0da-48915e47da7e 已由正常 bridge 啟動。

- 新增回歸揭露 bootstrap 過度限定來源字串為 scripts/_bootstrap.lua，會拒絕 package.searchpath 保留的 scripts/tests/../_bootstrap.lua。修正從檔案所在目錄解析 parent、交由 platform 正規化；搬移 fixture 加第5個 tests/../ 真子程序，5/5 通過；MCP endpoint identity12＋owned4、binary preflight15與 Lua module suite 通過。新失敗與修正保留，不重犯以字串形狀代替實際檔案路徑的錯誤。

- 建置診斷修正：D: 實際是 TOSHIBA DT01ACA200 SATA HDD；兩個 cl 的 page faults／working set 持續上升、D: 約4MB/s讀取，低CPU／無新action不等於死鎖。primary 在未觀察 I/O 前過早把 detour 場判成停滯並停止（自有UBT9144），新UBT20408使用官方 -UBANoDetour -UBALog／2 actions；此後保留本輪，不能無新證據重啟，也不把關detour宣稱通用修復。沒有改引擎／pagefile／記憶體門檻或移動使用者指定引擎。

- MCP write worker 約15分17秒沒有完整補丁／新公開進度，經 bridge 正式取消 run-mux2n04e-631s5b，killDelivered=true，原 agent/bridge34056/32668後續確認不再存在；取消前最後約15秒已改 header4行宣告，primary稍後審查才發現，未刪除，待整合。改完整公共來源的 no-tool 補丁提案 run-mux36sp5-h8gbne / thread700ef34b-5fdd-4b98-ad76-ad877b34f356；不把等待或取消當作完成，成本未知。
- 讀取 UE5.8 VCProject.cs2257 確認 source Editor 專案還會建置 ShaderCompileWorker；新機該exe亦缺。通用初始化入口因此補正式 worker prerequisite，完成後檢查兩個exe，再走正常 gate；不只建Editor就冒稱可啟動。planner與六項stage gate回歸通過，實際worker建置仍待目前自有Editor build完成。

- MCP 提案包原先漏讀 wait_ue_mcp.lua 的 GET /mcp/health（server=uecp）契約，不能只提供 POST /mcp 就宣稱 readiness。proposal 回來後 primary 必須補此唯讀 route 的實作／測試，仍由原生Editor process擁有port。搜尋再次把 Windows literal wildcard 當路徑失敗，後續此輪搜尋固定整個 scripts 目錄＋-g pattern，不再指定glob作path。

- Primary先獨立移除原 StartServer 的 shell/netstat/lsof 按端口殺程序區塊，改 FIPv4Address::InternalLoopback 且不用AsReusable。UE官方TcpSocketBuilder預設Reusable=false已核對；只做source審查，尚未編譯／啟動驗證。取消worker後仍須立即檢查diff，不可把上一次status當取消瞬間的狀態；本次漏掉最後header修改，已更正紀錄。

- Grok native proposal run-mux36sp5-h8gbne terminal completed，約1127秒／USD0.21661672，input32951/cached1152/output95105/reasoning78598/total129208，1model call，API duration未提供。進度log會失去重複括號／斷行；必須用stored job.result.rawOutput的公開完整結果，仍修正其中不合法literal／小數斷行，不從progress拼可執行原始碼。提案diff有placeholder，所以primary手動整合並保留原permission/dispatch區塊。
- Native transport source整合9項automation測試（尚未執行），primary補GET /mcp/health、JSON media type／Origin拒絕、收送10s期限與完整非阻塞送出、WouldBlock不作斷線、模組Private include路徑；按實際UE Sockets.h驗證HasPendingData參數應uint32（提案寫int32），primary改採Wait/Recv流程。編譯／真連線未通過前不能接受MCP就緒。一次apply_patch因原空白行不匹配原子失敗，先讀真尾段後分段patch，未套半個patch。
- 新增native .cpp必須讓UBT重新收集action graph；primary只退役自有UBT20408及compiler children（本輪至137+/4186，完整末端以保留log為準），改真Lua build_ue_moba --bootstrap-engine --max-parallel-actions2正式入口，Rust工具／codegen／bridge stage已實際通過，正在先補ShaderCompileWorker。這次退役是新source graph所需，不是因HDD低CPU；objects保留，無新對局。

## E348：使用者暫停、同世界真人操作與施法失敗不可冒稱成功（2026-10-06）

- 使用者要求「先停止，等下再繼續」。Primary以原exe／creation token核對後只停止本場authority，原workflow自動停止PIE並清理三個original lifetimes；v5 report cleanup_verified=true、success=false。此false是人工暫停而非新observer failure；自然結果未完成，不勾6.2。本輪實際8場，上限10，恢復時不可重置計數。
- 原owned Editor最小化造成截圖拒絕。透過本project MCP search_tools查到editor_ui_windows，list取得原窗口ref再restore，未要求使用者操作、未新增OS fallback、未採用外部Editor。實際shop圖已檢視：buy InputId1、sell InputId2，HUD Transaction #2 settled(code0)；attack-move InputId3後出商店範圍，Q upgrade InputId4後L2，真人死亡重生可見。
- R no-target InputId5被接受不等於效果成功：server tick83725記錄AbilityCast ok後緊接execute returned error。僅本次真人R看到兩條（authority／observer）；尚未定位根因。不可宣稱heal/CD成功，也不能由靜態推測修補。通用diagnostic改為保留ABI RErr文字，未改交易或施法規則；真正四技能headless cast/settlement scoped1項通過，並不重現本場失敗。新log改動未部署至已停止v5。
- 圖片tick78782／84342顯示simulation60Hz但renderer FPS3；與先前FPS60不同，不混稱畫面效能過關。既有固定十二效能FAIL保留。
- Grok純幾何lane-review run-muwoiwru-cg0ms8/threadc3e0d4b6-def5-441c-986a-37ec99efceb7在197秒仍starting，因使用者暫停取消，follower terminal cancelled、無提案或修改、成本未知。stop回報原bridge PID36852已不存在（taskkill128）；不把清空追蹤欄位冒稱所有未知原agent身份已驗證退出。
- 操作錯誤：又猜OmAbilityVisualCatalog.h／expected-checkpoints.jsonl／scripts/lib/process.lua不存在，已改rg --files／目錄listing／bootstrap實際模組；Select-Object -First誤寫英文seventy已改70。禁止重複猜檔名與型別錯誤。

## E347：Bot 零關聯購物回執使整隊 frame 遺失，重同步不能補救遺失的生命週期（2026-10-06）

- Grok上述no-tool job最終277s完成，logic和primary已實作通用規則一致；其測試把四參數回傳Vec API猜成五參數output-parameter，不能編譯，不採用。Primary實際7tests已證明行為，並非Grok跑過測試。成本USD0.05481208，API duration unavailable。第8場v5已建立唯一authority/runtime/原PIE，仍最多10，不更改既有門檻。

- 第7場 v4 原owned cleanup=true，保留原結果false。新增disclosed元件對照191對191，仅enemy replica1903 的Hero冷卻／TAttack差異。原wire capture：14980 Reveal，15120仍有Movement／CommittedAttack／Progression，15238起僅cached AttackVisual；原server tick15132記錄projection MalformedDisclosedState，接著兩observer MissingReplicaTick。不是移除schema、資源hash、迷霧或GPU問題。
- 實際source唯一MalformedDisclosedState生成在shop_receipt。ServerRoleBotControllers正式Bot input correlation=0，投影回執valid要求input_id非零，造成成功／拒絕的Bot交易均可能令frame生成失败；生命週期transition因此錯過，使對手幽靈繼續進行client tick。
- 通用決定：零關聯authority-local輸入仍先消耗並驗證同player FIFO settlement，再省略UIreceipt；不能在pop前過濾零ID，不能製造假ID，不改交易、accepted input、真人receipt codec或安全驗證。Primary已實作，scoped shop_receipt 7/7通過，含零ID→非零ID不同商品FIFO、command不匹配仍拒絕、UnknownItem零ID不造成功、兩連續projector tick不遺失；非完整驗收。
- Grok新no-tool packet run-muwnjopx-ctgbc7/threaddc393f43-da9f-4ef3-af8b-1828edfe300d仍starting；primary不空等已知修正，由primary產碼並待提案作比對，尚不冒稱Grok完成此修正。固定Lua離線capture檢查工具與component比較工具不開新遊戲。禁止印server私密sentinel；比較輸出只打印schema／replica／差異數，payload留本機證據。
- 又誤把demo*.rs當Windows具體路徑，已改目錄 -g 及rg --files；禁止用猜檔名失敗反覆代替source定位。

## E346：正式模式證據選項被 test-mode 門檻忽略與 Team 2 確定性中斷（2026-10-06）

- Grok read-only調查 run-muwn1qpx-68qy5m/threadb301bf02-265c-4030-b2d3-559461e5c291 在 source/evidence讀取處296s無結論，primary取消，follower terminal及bridge三個追蹤PID清空確認；成本未知。「兩hash包含resources不同」是靜態差異，不等於本場根因，沒有採納成修正。後續改交已取得的具體元件差異／原始碼片段，避免工具讀取停滯。

- PIE v3 同世界選角、60Hz、1真人9Bot與原生HUD成功，但仍在 Team 2 sequence15586 因 observer-mismatch-after-rebase fail closed。首個不一致檢查點 tick15240／sequence15227，Team 1 持續一致；不能宣稱迷霧索引修正已修好此問題。原workflow已自動完成owned cleanup，report success=false、cleanup_verified=true。至此實際6場，最高10場。
- runtime 原本只在 test_mode=true 建立 EvidenceRecorder，顯式 --evidence-dir 在正式模式被忽略；且先前把 runtime CLI 選項誤當server環境變數。入口改傳真正 CLI；採納 Grok run-muwmuk5g-2mfryn／thread6852d8a5-cc3c-4c3f-884d-039bf88a9c89 的純 capture_policy，primary正規化stream空白、審查並整合，4項測試通過。正常無directory仍不記錄、test-mode缺directory仍原錯誤，不啟用腳本／fault injection。
- Server／runtime scoped release build39.90s／38.30s成功。observer mismatch 新增既有 opt-in evidence 目錄的 disclosed component值／digest對照，僅含該隊已公開資料，不印出私密sentinel檔；不放寬hash、安全、rebase重試或強制winner。下一場只為取得具體差異，不是無改碼重跑驗收。
- 操作錯誤：猜測不存在的replica_world.rs和src/bin資料夾，後續先rg --files；MD patch把完整標題誤簡寫為E345導致拒絕，原檔未改，改用實際完整context。

## E345：混合呈現事件的迷霧索引與長局 observer 錯誤不可掩蓋（2026-10-06）

- `final-same-world-20261006-v2` 真實選角三回呼／同一 PIE world 接續／compiled release60Hz／十人 scoreboard 成功，約 replica tick15586 後 authority `observer-mismatch-after-rebase` fail closed。不能把 stale Connected renderer 或自然結果等待時間當成成功；原原因目前未查明，沒有關閉安全檢查、放寬 hash、改成全世界 replication 或強制 winner。
- primary 核對該場 owned-processes.json 第2筆原 server 的 executable／creation token，透過固定 Lua stop_owned／wait_owned 只退役原 authority，使原workflow走失敗清理；report 同世界 handoff=true、success=false、cleanup_verified=true。至此實際5場（含先前 PIE v1 已開 server），未超10。原Grok review run-muwmfy30-juchdm/thread9828da38-9b8f-4fdc-abd5-245596b88414 290s starting 無輸出取消，terminal與原75212／91084不存在确认；成本未知，未冒稱完成審查。
- 當前功能修正：FogGridRetention 不再要求 fog stable_sub_index=0；真正 producer 在公共事件／external effects 排序後賦全域 ordinal，HUD／攻擊／buff 等令霧不再排第0。保留唯一霧、無subject、reserved u32::MAX、payload／team／epoch／tick／geometry／conflict全部檢查。新增真正 TeamViewProjector 的 HUD+fog 混合 frame 正向及 subject／duplicate／sentinel 反向回歸；compiled-only fog_grid 11/11 通過。這不是 observer mismatch 根因修復聲明。
- PIE observer 新增 bounded authority log 失敗偵測；遇 secure termination 即保留原診斷並走原owned cleanup，不再空等勝負。每次 formal PIE 診斷下一步沿既有 OMOBA_FOG_EVIDENCE_DIR 保存實際 filtered frames／checkpoint，不編造證據；下次真場次必須先部署新 runtime。
- 操作錯誤又把 moba*／observer* 當具體Windows路徑及猜 game.rs／observer.rs，失敗已保留；後續先 rg --files 找檔再搜目錄 -g，不能將讀取失敗当作不存在該功能。

## E344：選角流程不能用關閉 renderer 代替同一 PIE 世界的接續（2026-10-06）

- 新同世界入口 v1 真實 PIE 選角三個 pointer callback／Rust terminal 全部成功，尚未接入 gameplay 就因 owned-processes.json 二次寫入未傳 overwrite=true 拒絕。當前文件只允許更新本次自有 ledger，新增 true；不得放寬全域 path/json overwrite。server 已 spawn，所以保守计入第4場，舊 report 的 simulations_executed=0 是記錄時序錯誤，保留原錯誤檔，改成在成功記錄 server 身分時立即計數。cleanup_verified=true，沒有6.2完成宣稱。

- 新版本正式雙 UE 場次 `final-renderer-budget-baseline-20261006-v2` 重連／60Hz／cleanup 通過，三個 UE log 均實際回報核心提示 8。但十二效能門檻仍 FAIL：server max328.7441ms、p1／p2 replica max197.9532／227.7925ms、UE interval max180.390596／183.081299ms。平均 server4.844195ms、兩隊 replica <0.9ms、UE interval約16.75ms 通過，不把平均當全部峰值通過，也不歸因於已證明的外部建置；保留全樣本，不再無改碼重跑。
- `final-selection-natural-result-20261006-v1` 在選角 UI/service handshake 前逾時，未 spawn authority／replica、未開局。末日誌停於 TargetPlatform／Turnkey 的 SDK 初始化；僅證明停留位置，不足以證明是哪個子程序鎖住。讀原引擎原始碼後不使用影響其他系統的 `-Multiprocess` 來跳過檢查，也不只拉長原握手逾時。至此實際遊戲3場，另2次開局前拒絕／選角失敗。
- 原預設選角 terminal receipt 後 RequestExit，不能稱同一 PIE 選角到結算。新增明確 opt-in `-om-selection-in-place`／native saved receipt gate／controller handoff／strict presentation-only subsystem gate；預設舊流程保持。Lua 新入口先等待自己建立的 Editor MCP，再啟動 PIE 選角，validate Rust plan 後才啟動唯一 server／replica，MCP 接續原世界；不修改全域 commandline、不啟動第二份模擬、不使用 gameplay Lua，不強制 winner。
- Grok write job `run-muwm40c0-vi35dl`／thread `deddf896-09fa-4c3c-b1ca-c74d04ab576b` 在讀取阶段267秒無補丁，primary取消；follower terminal cancelled 且原83056／105444均不存在後 primary接手。stop taskkill128不是退役證據，成本／根因未知，不算 Grok 產碼。
- 新 MCP 可選 `--owned-editor` metadata gate 每請求確認原始 executable／creation-token 所指 lifetime 與 registry endpoint，相同 project 不等於本工作流所有權；局部原12＋新4 fixture通過，尚非實際 HTTP／PIE proof。
- 操作錯誤：再次猜錯 test_ue_binary_preflight.lua，實際位於 scripts/tests/ue_binary_preflight_test.lua；正確15項通過。新 Lua 初稿誤猜 path.copy，不執行缺少的 API，改保留原始截圖路徑，不製造多餘副本；編排尚在當前功能確認，6.2不先勾。


## E343：共用引擎的外部建置可使已建好的專案失去相容（2026-10-06）

- `final-renderer-budget-baseline-20261006-v1 --reuse-built` 在遊戲 spawn 前被正式 binary gate 拒絕；engine ID 為 `99e98d23-40b5-4b0a-b234-587b1458be0a`，project／BpGeneratorUltimate／OmRuntime 仍為 `cfa68684-c474-4502-9a0a-18f8585e7a6a`。該次未啟動場次，不能算成第三場或核心提示成功證據；前兩場實際遊戲的失敗與結果均保留。
- 唯讀查詢發現另一專案 `C:/portable/OpenKoikatsu/OpenKoikatsu.uproject` 的 UBT 正使用 UE5.8。保留其程序、不殺未知建置、不手改 BuildId、不移除 `-NoEngineChanges`。只執行既有固定 Lua build-only，以正式 UBT 鎖與專案建置同步自身模組；不對共用引擎做修復。
- staged-only 原契約只驗證 DLL SHA，不代表 Editor binary 相容。最終工具應在正式 frontend build／verify 之後加既有只讀 binary readiness；不能在 build 之前阻止合法修復，也不能取消真正 spawn 前的再次檢查。Grok 有界提案待 primary 審查，尚未宣稱已修復環境或效能。
- 本次再次猜 `moba_selection_flow.lua` 不存在；仍使用已存在的檔案清單找入口，不新造同名工作流。大型 engine manifest 不應整份輸出，解析後只列 BuildId／mtime；避免截斷掩蓋診斷。

## E342：renderers預留容量不能只留在JSON、不套用真正啟動參數（2026-10-06）

- v2功能重連通過／cleanup_verified=true，但十二固定效能門檻失敗：server mean5.33744／max106.9183ms；p1 UE interval max87.215498／work max21.523401ms；p2 replica max54.5498／UE interval max88.6259ms。原完整兩隊report與全部樣本保留，無提高／切片／重試挑樣本。
- 共用Lua已預留一半logical CPU給renderers，卻只傳Rust ECS／Tokio hint，Unreal仍回報physical cores16。原預算沒有約束Unreal數量提示；這是確實接線缺口，不宣稱已證明所有峰值根因。
- primary讀本機UE5.8 GenericPlatformMisc::GetConfiguredCoreLimits与WindowsPlatformMisc實作，確認-corelimit同時限制reported physical／logical count；只用既有-corelimit，不使用experimental physicalcorelimit、affinity或修改引擎。Grok有界no-tool diff加unreal_cores_per_renderer／args、獨立hint總量與oversubscribed；原Rust配置、schema1、oversubscribed語意不改。最低1造成小機器超預留時明確承認，不把hint當quota。
- Grok run-muwlkq5l-sk7uzt／threadc9cfc2f1-0ed6-42cb-bf96-47dbf40ce2bd completed133s、1call／1turn，input20386／cache1152／full21538／output13156／reasoning11144／total34694，USD0.04089656，API時間未提供。沒有工具／測試／檔案寫入；其diff串流片段於字串和數字內被空白切開，primary不直接套壞hunk，按原文件context保留語意整理再apply_patch。不能把該傳輸文字當已驗證patch。
- primary独立968配置＋6拒絕、role8/8通過。兩正式launcher共用hint；reconnect複用原argv因此保持。最終工具加入三個原生log之commandline／Cores回報與configured cap檢查，不只檢JSON。新版本驗證於 spawn 前遭 BuildId gate 拒絕，未宣稱效能通過。本任務目前實際2場、另1次無場次預檢失敗，最多10不拆批。

## E341：正常退出與身分查詢之間仍有原始handle競態（2026-10-06）

- 一次完整 frontend/bridge/stage、release content/server/runtime建置成功後，final-supervised-baseline-20261006-v1進入真60Hz雙UE；舊UE正常WM_CLOSE後inspect_owned在QueryFullProcessImageNameW回access denied5，尚未重開新UE就中止。原五程序cleanup_verified=true；success=false原始報告與全部日誌保留，不把它叫效能通過或真LAN。
- 原因：alive的零時wait與接下來image/time query並非原子操作。通用修正settle_owned_query：查詢失敗只在**同一個已保留handle**的新wait確認signalled時返回已退休；仍alive、wait本身失败維持原錯誤。不拿PID重查作退出證據、不用延遲掩蓋、不將access denied一律忽略。inspect／stop／close-window沿同源處理，TerminateProcess失敗後亦只接受原handle退出。
- 當前3個局部回歸全部通過：query失败+dead／alive區分、wait失敗拒絕、成功query不走錯誤恢復。新版本v2明確reuse-built+嚴格stage/readiness，不重建Unreal、不改十二門檻；這是修正新重現功能錯誤後的一次驗證，不是為挑效能樣本重跑。這兩場合計2，不拆批规避最多10場。
- 工具接線修正：實際server OM_PERF在stdout，stderr為空；最終收集明確合併該場stdout+stderr，runtime仍stderr，player1保留old/new兩份UE窗口。不能沿名稱猜日誌流，不能切掉失敗樣本。首次失敗的reconnect不是完整固定workload，另留原success=false不冒稱完整效能報告。
- 本批另發現runner content/server build label都叫content（執行前修正），改explicit label避免覆蓋不同建置日誌。reuse-built是明確選項，缺release元件拒絕、profile仍必須release，無debug fallback成功聲明。

## E340：強制終止監督需原生生命週期，不可只靠 Lua cleanup（2026-10-06）

- Grok job run-muwkcv1b-3wks70／thread333f680f-ae22-435b-926a-df4e179c7431 在Job API讀取階段無補丁，primary於601秒取消。stop回taskkill128不是退出證據；follower terminal cancelled且原bridge36112／agent21776均不存在後才接手。成本、API時間、根因未知；不改全域設定或憑證，不把primary程式算成Grok交付。
- 通用修正：明確 opt-in 原生 workflow_supervise／workflow_member。固定Lua、helper→cmd→Lua有界原始handle與出生序驗證、private Job KILL_ON_JOB_CLOSE、暫停建立後assign才resume，無breakaway／name-kill／PID-only。標記必須驗證實際Lua加入該named Job，偽造env不能跳過；入口只有正式role（非prepare-only）／headless selection／remote join，沒有全域改寫spawn。正常完成也確認job active0後才交回exit code。
- 初次compile誤用常數名稱／module：windows-sys0.61為PROCESS_SYNCHRONIZE、JOB_OBJECT_QUERY在SystemServices；依本機鎖定API來源修正，不使用未知數值。
- fixture v1把Lua global arg的0／-1一起序列化，JSON拒絕非string object key。改只複製連續正索引，保留空字串／雙引號／尾反斜線；不改全域JSON。
- v2：canonical verbatim path的argv被固定Lua loader破壞。v3／v4：verbatim cwd使cmd拒絕UNC並轉Windows目錄，native response空、exit1。直接固定Lua傳verbatim source亦重現拒絕；只在啟動argv／cwd移除canonical前綴（UNC正規轉換），handle身分比較仍canonical。host無效response診斷加入真exit code，不把空response算成功。
- normal fixture的ready是退出前durable receipt，不是live-service gate，改只該fixture使用bounded檔案receipt；正式ready保留original lifetime gate。
- v5：5cases PASS，normal/crash的original子／孫程序皆wait確認退出，fake-marker無spawn、unsafe-helper ancestry拒絕、exit7與空／引號／尾斜線精確保留。證據target/workflow-supervisor-20261006-v5/report.json。host9/9通過；固定入口help均exit0。source freshness改遍歷src所有Rust檔，不再漏新module；不跟隨symlink。
- 正在做一次完整建置／固定本機效能驗收，未先宣稱通過；不涉及真LAN或UI全流程。本批沒有commit／push／omfx／引擎修改／無關程序清理。

## E339：收尾時不可猜來源路徑或把截斷輸出當作審閱（2026-10-06）

- 本批再次誤猜 scripts/lib、OmController.cpp、moba_runtime_config.lua、moba_server_stage.lua 與獨立 OmGenerated plugin 路徑；實際 Lua library 在 tools/lua/lib，OmGenerated 是 OmRuntime plugin 內的 module。Windows rg 不接受字面 wildcard 路徑。後續先列實際檔案，使用實際目錄配 -g，不假定 module 就是 plugin。
- 單行 JSON 的 Get-Content -TotalCount 不限制輸出大小，native 報告與歷史 tasks 多次截斷；應解析 JSON 後先確認欄位，僅輸出所需統計。截斷不是完整審閱。本批重新解析 complete result 確認 native42／failed0／skipped0／not_run0；不把歷史報告當成本批執行。
- 收尾範圍：舊記錄的「當時尚缺」不能變成重做指令，也不能加入原契約外的新封關條件。IPC／cue 合併當前確認使用 compiled-content-only presentation_bridge tests，27 passed、0 failed、0 ignored；沒有遊戲模擬或 LAN。
- 原生 workflow supervision 仍由 Grok 有界工作處理中；未交付補丁前不宣稱強制終止清理成功。真正兩台 LAN 需要第二台環境，localhost 不替代。

## E338：選角取消必須共用，且不能冒稱強制終止監督（2026-10-06）

- 決定：明確 `--selection-cancel-file` 進入共用 budget，無期限／deadline 均可取消；單機、共享、headless 與 remote join 使用同一判斷，保留原始 owned cleanup、真人 consent 與 final 配方驗證。訊號只查存在、不讀／不刪，已存在則不 spawn；交付前晚到訊號仍拒絕。remote 不停止遠端主機。
- 真實邊界：native Child 未持續監督父程序，Lua 強制終止不執行 xpcall cleanup。合作式取消不是 crash supervision 修復；沒有實際強制終止測試、也不宣稱 orphan 已重現或已解決。有界 native wait 不是即時可中斷。
- 重複操作錯誤：Windows `rg` 的具體路徑再次含 `*.rs`／測試檔 glob 而出現 os123；改先 `rg --files` 找檔，再用實際路徑或 `-g`。既有 register 已警告，這次仍犯錯，後續不可把 wildcard 當 LiteralPath。remote fixture patch 猜 `execute(mode)`，實際為 `lifecycle(mode)`，首次 patch 未修改；讀取真實函式再重套，避免依猜名修補。
- Grok 自足無工具 patch-only job `run-muwk0gcn-9bt5vq`／thread `594ec77e-0668-4f6e-aff1-4de345e01cf6` 201 秒 starting 無結果，由 primary 取消。follower cancelled／三 tracked PID null／OS 原92460及85644不存在確認後接手；stop128非唯一證據，generic「Stopped by user」不是人類要求取消。成本／根因未知，不改全域設定，不接受或冒稱 Grok 產碼。
- 當前 budget assertions、headless12／join19／single13／shared25 共69組通過；join早先17組不重複計數。config 部分真 compiled binary，程序與renderer注入。沒有Rust／UE建置、game／sim／LAN／完整驗收，25/31保持。詳見 selection-cancellation-progress。

## E337：專用部署仍需可無畫面提供遠端選角（2026-10-06）

- 缺口：server-only 不需要 Unreal，但原共享選角一定取得至少一個本機真人席位，因此無法在沒有 Editor 的專用主機提供選角房間。
- 通用決定：新增獨立 run_moba_selection_host.lua／moba_selection_host，明確 selection-bind 與新 output；不放寬原 server-only／interactive-selection 互斥，不偷偷將空 local-player 改成另一種意思。共用 run_room 以空本機 renderer 清單支援全遠端，同一 Rust 房間與私人邀請、ready hash／interface、baseline roster／規則、original-owned cleanup／deadline／final recipe 保持。
- 原單人／共享選角與新入口共用 moba_selection_prepare。--lock-plan 只驗證本機配方副本，不當作遠端玩家 consent；作者 Lua 只在開局工具求值，正式遊戲仍 Rust／C++ compiled-only。選角完成不自動啟動任何 gameplay。
- Grok job run-muwjqv3n-r3f1lo／thread 6d137eeb-495c-41f3-b13c-ef8debade675 有讀取階段文字，但246秒無修改；primary取消，follower terminal cancelled／三 tracked PID null／OS 原85836及100776不存在確認後接手。stop exit1/taskkill128不是單獨退役證據，不宣稱Grok產碼、原因或成本，也不改全域設定。
- 新入口9組（真 release native config，host lifecycle注入）、既有單人13及共享25組通過，共47組；help與diff check成功。沒有Rust／UE建置、遊戲、場次模擬或實機LAN；完整25/31保持，未將功能存在或fixture當最終驗收。詳見headless-selection-host-progress。


## E336：選角配方必須確認正式 TCP 邊界與實際主機建置（2026-10-06）

- 證據缺口：E335 的服務測試／注入 renderer 不能證明正式 TCP admission worker；主機 executable 也尚未重新編譯。
- 決定：正式 compiled-content-only release moba-config 建置成功；新增測試直接呼叫 production connection，以真實 ephemeral loopback socket、私人 Invitation、JSON-line command 驗證兩真人各自 lock／finalize、一次 host take 後 follower read 與重新 admission 得到相同不可變配方。拒絕錯 token 無回覆、無席位占用、無 revision 變更；退出釋放席位。不新增正式協定、測試專用 production fallback 或 gameplay Lua。
- 測試生命週期：client connect／read／write 有界，RAII 先關閉 socket 再 join worker，包括 assertion unwind；production 的真人思考無期限保持，不為測試改短。
- 操作錯誤：首次讀取猜測的 moba-framework-error-register.md 不存在；rg --files docs 找到真正 unreal-moba-error-register.md。不要依猜名新增第二份 register。大型 OpenSpec 歷史讀取輸出截斷不能當完整內容，以精確區段核對當前任務。
- Grok write job run-muwjfkw0-w21bq7／thread 5a874ee0-7ee6-431f-b063-f63944a18868 在讀取階段 203 秒無修改，primary 取消；follower terminal cancelled、tracked 三 PID null、OS 原 101476／43324 均不存在後才接手。stop exit1／taskkill128 不當作唯一退役證據；原因與成本未知，沒有改全域 MCP／auth／Grok 設定，沒有接受 Grok 補丁。
- 當前選角網路 3 項通過（新增真 TCP 1 項＋既有 framing／publication 2 項），修改後格式化再確認不重複計數。release build 成功不是一致部署全部遊戲元件；worker 測試不是 host accept loop／proxy executable／Unreal renderer、實機 LAN 或完整遊戲驗收。未跑場次模擬，完整 25/31 保持。


## E335：遠端選角缺少主機最終配方分發（2026-10-06）

- 實際缺口：只有finalize成功提交者取得plan，其餘read/stale回覆雖finalized卻無配方，遠端完成選角後仍需再次人工傳檔。
- 通用決定：Rust session的finalized plan只讀副本供所有已綁定玩家取得；host take仍一次領取、read／EOF／rebind／競爭失敗不重建authority handoff。Lua核對全部locked seats／自己真人／catalog及原生compiled validator的完全相同plan後才保存，不從seats補造、不同結果不normalize接受、不啟動gameplay。
- 已拒絕的方向：不以所有DLL副本一致冒稱source provenance，不用mtime／作者Lua存在性作runtime啟動要求，以免破壞compiled部署。
- 指令錯誤：core不是omb workspace member，cargo test --manifest-path omb/Cargo.toml -p omoba-core --features...失敗；改core自己manifest並保留compiled-content-only/kcp成功。rg的Windows具體路徑含glob與猜不存在Cargo.toml／moba_role_result.lua失敗，後續先rg --files找實際檔名，不新造入口。
- Grok只讀job run-muwich4j-mnzbqf／thread d652d9ec-5663-4b93-a258-cb4c29ae3d11 329秒無完整稽核結果取消；follower cancelled／三tracked PIDnull／原47028與68152不存在另驗，stop exit1/taskkill128不是單獨成功證據。成本與根因未知，primary實作，沒有冒稱Grok產碼或調全域設定。
- 當前Rust選角11／Lua join17（含真實debug compiled config canonical roundtrip）／shared25共53通過；renderer與網路仍注入，沒有game／sim／LAN／UE／UBT或全验收。主機moba-config需正式指定profile重建才能分發新plan，旧版缺plan明確拒絕，不fallback。完整25/31維持，見selection-final-plan-distribution-progress。

## E334：遠端選角漏接正式前端部署檢查（2026-10-06）

- 問題：私人邀請入口只找到Editor就spawn，漏用正式對局的BuildId／模組binary與staged DLL副本SHA檢查。
- 通用修正：共用moba_frontend_preflight，兩個入口沿同一唯讀gate。遠端在mkdir／spawn之前檢查；固定Lua的verify-only指令exit0才通過，失敗不build／stage／改BuildId或切換profile。這不是完整ABI／runtime內容相容性驗收，dedicated不引入UE。
- 操作失誤：讀取猜測的test_moba_launch_workflow.lua不存在；以rg找到實際test_moba_launch_contract.lua，往後先搜尋檔名。接線改具名local function variable，避免Lua換行括號黏接前一呼叫。
- Grok run-muwi643t-m7crhy／thread84673de8-6bbb-45f6-95ee-ca6cd221d050有讀取階段文字但128秒未產碼取消。stop exit1／taskkill128不是清理證明；follower cancelled／三PIDnull與OS原42964／45864不存在確認後primary實作。原因與成本未知，不宣稱Grok修碼或變更全域設定。
- 局部53組通過：gate5／join10／workflow8／binary15／role8／dedicated7；設定工具部分為真Rust，renderer／build均注入。沒有game／UE／LAN／sim／Cargo／UBT或完整驗收，25/31保持。詳見frontend-preflight-progress。

## E333：共享選角必須區分本機席位與遠端玩家（2026-10-06）

- 真缺口：moba_shared_selection為全部真人開本機Unreal，未使用local-player；raw Rust選角服務已支援具體bind與private invitation，但正式Lua入口只允許loopback且缺remote join入口。不是缺新的Rust選角規則，也不重做已存在的遊戲connect。
- 決定：共用placement驗證／排序本機席位，不刪遠端roster；selection-bind明確opt-in且獨立於game bind，Rust分配實際port，ready地址精確核對requested interface／canonical port。原預設仍loopback，不改防火牆。
- 遠端入口僅啟動選角renderer，compiled-profile工具沿原生Rust proxy驗席位／token／catalog；讀取邀請時size cap及generic parser錯誤，token不回傳、不進arguments／logs／result。只清理original local renderer，不停止remote host、不自動lock／開局。
- 一個fixture首次失敗：錯把「valid terminal receipt之後PID重用」預期為失敗。對齊既有original-lifetime契約：合法回條可完成，replacement不能被stop或wait；另加沒有回條的reused-cancel必須拒絕，沒有放寬receipt／shared handshake。
- Grok run-muwhr9w2-0c8cqj／threaddbcb543e-12bb-442a-b32f-8b78f4eb5d8b 使用本次child env限制tools空、disable-web/no-subagents，4分9秒仍無文字／補丁，primary取消；terminal cancelled、tracked3PIDnull与original84364／13104不存在確認。這個結果不能支持「file tool為根因」；成本、API時間及根因仍未知，沒有全域MCP／憑證／插件設定變更。
- primary實作placement7／join9／shared25／single13共54組局部通過，都是table／injected renderer，不啟動network／Unreal／game或simulation。完整兩台LAN与UI自然終局仍未驗收；25/31保持。

## E332：啟動預檢不可用 cargo run 偷換建置 profile（2026-10-06）

- 問題：正常工作流已建置所選profile，prepare卻呼叫cargo run（隱含debug），造成release／no-build仍需要Rust工具鏈並可能重新編譯，不是可部署的原生工具工作流。
- 決定：正式host／remote／dedicated與prepare-only、interactive selection共享指定profile的moba-config.exe入口；缺檔在輸出mkdir之前拒絕並給明確建置指令，不自動切profile或cargo fallback。真正建置仍由明確build流程負責。
- 非零或缺失exit_code必須先拒絕，再解析JSON；不讓失敗程序附帶的舊stdout通過。舊selection fixture補真實exit_code=0，不弱化正式檢查。
- 操作錯誤：本批rg猜不存在的moba_hero_selection.lua；實際選角在moba_role_launch／moba_shared_selection。沿rg --files清單定位，不把歷史猜名當已存在。
- Grok run-muwhfuzt-dx1moz／thread84646d8f-5f9f-4e80-8c8f-95633f5c75b4 1分59秒仍停讀取，未產生修改；primary取消。stop exit128不當退出證據；follower cancelled、三tracked PID null與original agent20828／bridge60604不存在確認後接手，成本/API時間／卡住原因未知。
- 測試操作再犯E327類型：nested tests以未折疊../載入_bootstrap，使其lexical root含_bootstrap.lua；真實integration mkdir失敗，不是主機執行失敗。測試入口明確取tests的scripts父目錄，避免傳入../；兩個新測試一致修正後通過，不修改遊戲路徑fallback。
- 局部helper5／integration3／selection13／shared22／role8／network7／dedicated7，共65組通過；integration含真實debug、release設定／lock-plan與缺工具不建立輸出。無Cargo建置／simulation／game／Unreal／LAN完整驗收，25/31保持。

## E330：退場輸入邊界必須包含 Widget 的獨立提交入口（2026-10-06）

- Controller EndPlay 阻擋 controller input，不能取代商店／小地圖自己入口的 lifetime gate。retained widget／Slate callback 可能持有舊baseline但查到仍存活的shared runtime；加入widget-local terminal退役，不以全域Stop修此缺口。
- 退役 gate 必須早於 bridge query／NotifyOwnedGameplayUnavailable：舊widget不能因late callback清掉新HUD。NativeDestruct和controller顯式退役共用入口，late setter／Slate rebuild不可復活instance輸入；新HUD用新widget。
- 技能目錄初次猜在專案 `.agents/skills/grok-build` 而不存在；依技能 catalog 改讀 `C:/Users/Damody/.agents/skills/grok-build/SKILL.md`。不把技能缺檔當Grok CLI故障。
- 搜尋初次猜 `OmWorldBridge.h` 不存在，改 `rg --files ... -g '*WorldBridge*'` 找 `OmWorldBridgeActor.h`／Subsystem 真實檔案。大型設計／task歷史輸出被截斷時，改讀所需實際區段，不把截斷當完整讀取。
- Grok本次3m35s只回啟動文字、無code delta；取消後follower terminal／三PIDnull／原agent與bridge不存在確認才primary接手，不將stop128單獨當退出證據。成本與停滯根因未知。
- 第一版late-setter gate用值型temporary會額外複製整份minimap／economy陣列；review改空baseline＋const reference選擇，保留既有單次cache copy，避免防護增加每frame配置。
- 局部Editor已ready但native runner的out-dir用了path.join(root, absolute)，組成D:/code/omoba/D:而未執行斷言；原Editor32884原lifetime已退出。正式入口改path.absolute並對正規化後default路徑檢查，支援相對／絕對輸出，不為fixture退回只支援相對路徑。嘗試讀預期editor.log不存在並非Editor未启动證據，真正ready與original退休由owned token／MCP核對。
- 第二次Editor78528在新fixture TakeWidget時crash，MCP回Recv failure不可當暫時網路問題重試。CrashContext BBD5FA0B... 經已安装llvm-symbolizer／真正PDB對到FOmWidgetInputLifetimeTest:2287與UUserWidget::OnWidgetRebuilt／WidgetTree::ForEachWidget；NewObject後漏Initialize。沿既有fixture先Initialize再TakeWidget，修測試不改正式UMG或放寬退役gate。兩測試started不等於passed；該原Editor已退出。取證啟動用abslog而非把log旗標當絕對檔名。
- 最後原Editor78512單輪兩native實際2/2 passed、零error／skipped／not_run，沒有真對局／完整PIE。cleanup在退出期間exact identity query access denied，不能宣稱其精確Windows根因或直接當dead。保持第一report失敗與原錯誤，再獨立成功original-token liveness回false／OS原PID不存在，另存retirement-confirmation成功。不是重跑或改寫舊失敗report，不能為這次錯誤放寬正式process inspector。

## E329：Controller 退役必須清自有 UI，不能停止整個對局（2026-10-06）

- AOmPlayerController 沒有 EndPlay override，只在 bridge rebind 時解除 listener。Controller 退役必須阻擋 late input／HUD重建、解除自己 listener、明確 StopSelection 並移除自己的 widget；不使用 RemoveAllWidgets，不停其他玩家、全域runtime或後端。
- 初次 restart status 從 monorepo cwd 執行，工具找 D:/code/omoba/om.uproject 而失敗；改用明確 omfue cwd 後 success／matching_editors=0。所有此工具呼叫須綁定實際專案，不把找錯路徑當引擎問題。
- 又一次 PowerShell literal `tools/lua/lib/*.lua` 造成 rg 路徑錯誤，改 rg 對真實目錄加 `-g '*.lua'`。不得用 Bash-style filename expansion；本次不新增 fallback。
- HEAD 已由使用者 damody 改為02757d51、正式上一批修改已提交；只保留和接續，不 reset／覆寫使用者commit。既有omfue未追蹤assets／slnx不屬此次清理範圍。
- Grok run-muwg4oay-dtia3c 3m59s 無 code delta，primary 取消後確認 follower cancelled／三tracked PID null／原agent和bridge不存在才接手。stop128不能當唯一退出證據；成本與確切停滯原因未知，不擅改全域MCP或認證設定。Primary限定三Unreal檔實作，11-action NoEngineChanges compile 成功（13.88s）。
- Native fixture 用 explicit EndPlay／EditorPreview parent attachment，不能宣稱測到真 Actor destruction 或 viewport；只編譯不能宣稱斷言通過。新測試加入既有 Lua runner 清單，完整驗收最後執行。
- 本次後續搜尋又對 literal scripts/moba*／docs/plans/2026-10-06* 出錯；正式搜尋改真實目錄搭配 -g，避免猜測路徑／shell expansion。此類錯誤不改遊戲實作也不新增 shell fallback。

## E328：結算讀取失敗不得沿用舊結果；共用 reader 必須保留消費端語意（2026-10-06）

- `moba_role_finish_observer` 曾把 `file:read(...)` 的 nil／IO error 轉為 `''`，使讀取錯誤與 EOF 混淆並可能沿用舊 complete result。決定與選角共用 bounded IO primitive，read／seek／close／short read 必須 propagate，opened handle 在 callback 前關閉；不提交失敗 IO offset、不用額外輪詢掩蓋。
- 共用的是 bounded IO，不是 mode policy：選角握手 latch 後免重讀；結算持續觀察以抓矛盾歷史，各玩家只保留一筆 canonical 結果。Result marker 完整行／canonical numeric 欄位，不能讓額外尾欄位或前綴部分匹配作為已結算證據。
- 新搜尋曾對 PowerShell literal `scripts/moba*` 及猜測 `scripts/test_moba_role_finish.lua` 失敗；用 `rg --files scripts` 定位實際 `scripts/tests/moba_role_finish_observer_test.lua`。不可用 Bash-style 路徑 glob 或猜檔名重複讀取。
- 找到勝負 launcher fixture 仍是 PID-only fake（沒有 spawn_owned 等），與已修改 production 不相容；接既有 original-child fixture 並從 fixture 取實際 waited records，不放寬 production、不把舊 fixture 失配當成正式 gameplay 回退。
- 結算 success 分支曾先寫 observation／break 才檢查 deadline，late screenshot predicate 可繞過限時。決定同原 deadline 在 poll 前／後與 publication 前檢查，補 screenshot IO 將 clock 推過期限的回歸；合法 UI 結果不授權逾時成功，不強制指定勝方。
- 初版 byte-boundary fixture 用 Windows text-mode write，手動CRLF被再次轉換，chunk byte位置亦改變；改既有 `path.write(...,true,true)`／fixture-only binary append。不要為錯 fixture 更改reader的CRLF語意。Prefix反例需把X放在marker前，不是合法UE log prefix前；Lua table最後的gsub需括號避免第二返回count成額外case，修測試而非放寬正式parser。
- Tool-free Grok run-muwe88iz-sv5due 6m43s沒有回覆或patch，primary取消；follower cancelled／metadata三PIDnull／原agent與bridge不存在確認後接手。stop128不當唯一退休證據；未證明確切停滯根因、成本未知，不因本次無工具就歸咎MCP或把取消算成功。

## E327：握手不能用版本前綴；退役後仍需有界讀完日誌（2026-10-06）

- 選角 launcher 曾用 substring 判斷 `protocol=1`／`shared_room=1`，會接受 `10` 前綴；single 更未驗證實際 producer 固定輸出的 `shared_room=0`。決定共用嚴格完整行 parser，綁定 player、protocol、room，拒绝額外／缺失欄位；不放寬 production 契約去配合過時 fixture。
- 重複 `path.read` 完整日誌讓成本隨選角時間增長；shared final artifact 快速路徑也會重讀全檔。改為增量、每 poll 至多 128KiB、單行至多 64KiB；有效握手 latch 後不再掃描。原始 producer 退役後可用既有 handshake/completion deadline 分段 drain，避免只讀第一塊而錯拒合法 backlog。
- 活著的 producer 最後半行不能作為握手；只有 verified original-child retirement 後，EOF 才能接受完整但沒有換行的 marker。Final artifact 本身不取代 renderer 握手、terminal receipt、內容 hash 或原 child 退出證據。
- 單人歷史 mock 缺 `shared_room`，此次依實際 `OmHeroSelectionWidget.cpp` producer 修正。新增版本前綴、錯誤 room，以及已發布 final artifact 的 backlog／EOF 快速路徑回歸；驗證結果以進度文件實際輸出為準，不當成真 Unreal／LAN 驗收。
- 新 test 初版以 `scripts/tests/../?.lua` 載入 bootstrap，但 bootstrap 的 lexical root regex 不消去 `..`，使 root 變成 `_bootstrap.lua` 檔案並在 mkdir 失敗；沒有建立錯誤目錄。改從 test source 擷取實際 scripts 父目錄，不引入 shell fallback、不更動 production root 契約。
- Grok `run-muwdxk9k-vxwgie` 初始讀取後 3m22s 無 code delta，primary 取消並獨立確認 follower cancelled、metadata 三 PID null、原 bridge/agent PID 已不存在才接手。stop exit128 原 bridge PID 已消失，不能單憑該錯誤就宣稱 worker 終止；成本與停滯根因未知，本批實作不得歸功 Grok。

## E326：選角與 role launcher 仍需原始程序生命週期（2026-10-06）

- 發現兩個正式 launcher 還使用 spawn/inspect/stop/wait/poll_ready 的 PID-only API；同 executable PID 重用會誤判，不能因 paired launcher 已修就宣稱全部入口安全。已統一 original Child token、持久 canonical executable，補 bounded wait/readiness（predicate前後與耗時期限），原 child 退役不操作新程序，partial startup只清已知originalchildren。局部9／17／8／10全通過，詳selection-lifetime-progress；不是UI／LAN驗收。
- 第二Grok run-muwd53b1-fuex10 7m15s 未產碼，primary取消、follower cancelled／metadata三PID null後接手。stop exit128為PID已消失；成本未知。partial metadata記ueCP 30000連線拒絕，未啟動無關Editor／改全域MCP，不把warning直接判為stall根因。純程式委派不應為初始化未使用工具無限等待；無實際delta時按bounded策略接手。
- primary初版 owned-wait mock回傳非canonical path，已用正式validate_owned修fixture；production不放寬identity。角色launcher測試含既有Cargo配置preflight，不能稱全部純fixture；printed Unreal started來自mock流程，不當真正開過UE。
- apply_patch一次上下文匹配失敗，重新讀實際尾段並用唯一test名稱錨點成功；沒有假稱失敗patch已套用或同時讓第二writer接手。曾猜不存在OmGamePlayerController.cpp／local_worker_budget.lua，rg確認實際功能來源，budget正式名稱moba_host_budget且role與paired已接線，不重造模組。避免再憑慣例猜路徑。
- 加入late-predicate期限案例時第一patch多出空@@導致invalid hunk，沒有套用；修合法hunk後實際10/10。Predicate耗時必須計入原deadline，true不能略過時間檢查，否則 bounded readiness 是暫時解。

## E325：dispatcher 本體、排程邊界與局部 fixture（2026-10-06）

- 本次 outer30.7792/24.8499ms 通過單項窗口門檻不能宣稱 Nearby 配置造成舊93/102ms或永久修好；同一步Job本體只有.1316/.0279ms，不相減冒稱CPU或wait。保留歷史失敗，停止追加效能採樣，轉到具體未接齊的選角/對局原始child ownership。
- 新 owned wait fixture 的 mock 返回 `/` 路徑，但正式 inspect_owned／validate_owned 都返回 canonical Windows路徑，導致 liveness 被 fixture 誤判退出；改fixture用正式 validate_owned，六項通過，不放寬 production identity 比較。新wait詢問失敗要propagate，不能當原child已退出；原child退出後PID被重用則成功退休，不等／停新程序。

- 新 fixture 原用 tick 0，正式語意會跳過 gameplay dispatcher，所以期待 Some 是錯誤測試；改 tick 1 確认執行路徑，保留 tick 0 規則，1 個 concrete Specs equality test 實際通過。
- PowerShell 不支援 Bash brace path 列表，ParserError 後改為明確檔案；error register 亦一度猜錯名稱，rg --files 後定位既有 unreal-moba-error-register.md，不新增重複文件。
- 大段歷史 context 輸出截斷不能宣稱全新完整閱讀；使用既有已讀背景、compact OpenSpec 狀態及本批具體來源。不得把截斷當零缺項。
- Grok Nearby 交付實際是 3 tests，不是進度文字猜的 4；主 agent 實際 diff 與獨立 3 passed 後接受。配置改善不代表 dispatcher 93/102ms 已修。Job sum 可高於 wall，不做相減偽 wait；ProcessCPU 合法 0 不等於根因。
- 同時編譯 artifact lock 只代表 Cargo 等候，不當 hang；本批限受影響功能測試及一次 bounded 真採樣，不重跑完整驗收或修改效能門檻。

## E324：固定步進 wall time 不等於 CPU 根因；新增診斷不得破壞既有契約（2026-10-06）

- 最終git核對HEAD從c5211e83變ca34c786；log顯示damody於15:03:24另行提交，末筆只更新omb/omfue gitlinks。主 agent／本批CPU代理沒有commit/push；保持外部提交並實際核對本批剩餘diff，不以舊baseline宣稱HEAD未變或reset他人工作。獨立scanner改動已被外部提交收入，不能再冒稱未提交全是本批未追蹤。

- 真 paired run兩隊各50完整窗口/3000samples，peak dispatcher93.0694/101.7626ms，process CPU delta合法0，不當null、忙碌CPU或精確wait證明；仍client超50ms。讀log時rg目錄被ignore漏查，改指定實際stderr檔後找到新記錄並collector成功，不能因第一次零結果誤判接線未用。
- 查實際shred可知dispatch_seq不執行thread-local，且gameplay仍有nested par_join；直接替换可能逃到global Rayon pool／破壞thread-local工作，不採未證明的「inline就好」暫時解。rootcause工作限dispatcher/inner jobs，不回猜hash或Lua runtime。
- CPU原生query fixture不應要求第一total>0：成功API可合法0；修fixture只查Some、100ns encoding與non-regression，不把counter encoding當100ns精確測量保證。

- 實機採樣前 Lua -e guard 誤猜 path.is_dir，不存在而在任何 spawn 前失敗；rg 確認正式 API 為 is_directory／exists，改 exists 拒絕任何既有同名 evidence 才啟動唯一新 run。不是多次遊戲重試，未覆寫證據或操作既有程序。

- 主 agent 新 Rust fixture 首次缺 DisclosedWorldStepper／DisclosedReplicaWorld／StepInjections imports，第二輪缺 specs::WorldExt；依實際 compiler 補 import，第三輪9 stage／2 concrete Specs+reconciliation／7 CPU通過，runtime compiled-only check成功。不是零測試exit0，不忽略編譯錯誤；既有三個 td_rounds dead_code warnings保持標示。
- 小範圍 CPU Grok run-muwc1hp7-rqf7ez terminal completed，primary實際 module／native FFI diff審查及獨立7 tests接受。全接線由primary完成，不把該代理未跑的tests冒稱已跑，也不把診斷功能當50ms修復。

- 第一輪新 collector fixture 用 `i==1 and nil or 999` 表示 unavailable，Lua 會落入 999；實際第一項通過後 tie 測試失敗，修成明確 json.null 後 7/7。不是正式計時功能失敗，不跳過檢查或用 0 代替 unknown。
- 六檔 Rust 委派 6m13s 沒有本批程式码，主 agent 取消 run-muwbt1eq-sk1khc，follower cancelled、metadata pid/agentPid/bridgePid null。stop exit128 是追蹤 PID 已消失，未照錯碼猜仍在運作。metadata partialResult 顯示 read_file tool_output_error 與未運作 ueCP 30000 連線拒絕；本批純 Rust 不需 Editor，不去啟動其他專案或掃埠。成本未知。改獨立 CPU 模組委派，主 agent 接手其餘接線。

- 351/475 ms 峰值集中 fixed_step，只能排除外段為主要 wall bucket，不能判定 CPU 或 scheduler。補同 invocation preparation／18 production phases／finalize 與 process CPU，後者跨執行緒、可大於 wall，不做 wall−CPU 偽等待，不改 50 ms／不排除冷啟動。
- 舊 OM_REPLICA_STAGE v1 reader 拒絕未知欄位；新資訊需另一 linked 記錄並保留同一 outer peak，不能独立挑 phase 最大值。錯誤／stall／duplicate／rebase 必須清 stale，unavailable CPU 不補 0。
- compact packet 初稿再度錯猜 runtime/native/filtered_specs.rs；rg --files 確認為 runtime/filtered_specs.rs 並修 packet，不因路徑不存在增加重複模組。大段歷史設計讀取截斷，改精確小段來源與 compact apply 狀態，不宣稱截斷輸出已完整新讀。
- 委派 run-muwbt1eq-sk1khc 的初始狀態只代表已啟動，不代表完工或性能通過；由主 agent 在 terminal 後實際 diff／局部功能確認才決定接受。具體結果見 fixed-step-diagnostic-decisions／fixed-step-detail-progress。

## E323：未使用的無界報告、嚴格診斷與程序生命週期（2026-10-06）

- dispatcher快取綁舊pool，reconfigure_thread_pool不能只換Arc；fresh Grok run-muwbcwhp-fntlwj完成成功換pool後清dispatcher，失敗build?先於欄位寫入原狀保持。primary實際diff/獨立1test／481filtered接受。同count重配置也退役。正式game當前不呼叫此API，不能宣稱這是已確定的351/475ms原因；no fullsuite或第二輪真遊戲。

- fixed_step靜態追蹤先列出實際檔案後仍誤猜specs/src/dispatch/dispatcher.rs、runtime/deterministic_gameplay.rs及native/collision_index.rs；實際是native/gameplay_phases.rs與native/comp/collision_index.rs，dispatcher由shred來源提供，不在猜的specs路徑。找不到時用rg --files/符號定位，未新增fallback或依錯誤路徑修改。

- 真短程stage採樣exit0不能算性能通過：replica-stage-20261006-lifetime-v1雙隊tick227 outer351.1038/475.328ms，99.98%以上是fixed_step wall time。五own lifetime退出／遊戲movement/HUD/consumed成功，功能與性能分開；不推論是CPU或OS scheduler、不調50ms、不排除cold-start尖峰。真log在stderr而非空stdout，以實際目錄確認再collector；59窗口/3540samples不包含最後未滿60筆，限制明示。下一次只定位filtered dispatcher/step內原因，避免又猜JSON/hash/未讀map成本。

- lifetime委派run-muwamhpi-sxwx3k讀來源9m35s仍無本批diff，primary取消／tracked handles null後接手，不無界等待也不冒稱worker完成；cost未知。primary實作原始Child身份、same-handle native stop／close、strict session schema及reconnect完整身份，Rust1／Lua7／本次own fixture實際邊界確認通過，詳session-lifetime-progress。沒有遷移全庫legacy APIs或宣稱純fixture就是完整renderer驗收。

- `pre_repair_reports` 每 Applied clone/insert，但全庫只有 field/init/insert，無讀者或退役；Grok限定刪除後主agent獨立審查及runtime1/check通過。保留returned report/checkpoint，不用任意cap遮問題，也不冒稱50ms已修。
- 診斷收集器固定60 sample，同一outer peak整筆phases；bounded streaming、未知residual、缺窗口／倒退timeline限制明示。11/11局部確認。初稿read(nil,error)可能被當EOF，提交前修成fail closed並加injected IO regression，不能稱缺資料成功。
- 清理舊active UE session僅PID/executable，PID重用會誤停同名程序；native舊stop又query與terminate分開handle。新實機採樣暫不執行，先修精確creation FILETIME identity與same-handle操作，保留舊record缺token時拒絕而非猜／刪。
- 操作曾猜不存在的verify_ue_smoke_report.lua、launch-plan.json、implementation-errors.md，與PowerShell literal glob `docs/plans/2026-10-06-*`；改用rg --files／-g與實際error-register，不新增重複入口。大段讀取截斷改小段。
- 第二個Grok packet無錨點patch被append到尾，啟動事件仍顯示舊任務；主agent15秒內取消run-muwakryy-5uqrp2，follower confirmed cancelled、bridge pid/agentPid null。stop工具報PID已不存在exit128，不當仍在運作；metrics未知。新packet頂部加唯一有效任務與歷史邊界後才重新派送；Stopped by user為bridge制式文字，實際取消由primary決定。

## E322：選角 ready 不等於 finalized；效能超標須量測而非盲目調參（2026-10-06）

- 原單人／shared 選角的45秒只限制handshake，ready後可能無限等；game finish timeout 尚未生效。共用session completion deadline覆蓋啟動／所有本機席位／terminal receipt與退役／交接；人工預設不限時、自動預設120秒、顯式1..7200。逾時只取消並清owned程序，不合成lock／winner或偷偷開局。純budget、單人7/7、shared14/14通過，非真UE／完整對局。
- client max仍超固定50ms，不能把worker hint當已證明修復。使用者明確要求Grok子代理，主agent派 `run-muw9jh35-ydrn8m` 做固定窗口／同一尖峰sample分段唯讀wall-time，維持原OM_PERF、hash、ABI、玩法與門檻；終結後實際diff審查及獨立core6/runtime1/compiled-only check均通過，接受診斷，不接受效能已修。未知residual不得直接推論CPU根因；真對局採樣尚未執行。
- 最新進度原以無錨點patch追加至歷史文件尾且多留空行，git diff --check揭露new blank line at EOF；改置於文件頂部明確區分當前25/31與歷史，清掉本批多餘空行，不還原其他dirty。
- 為縮短read-only diff-check警告曾錯用一次性core.autocrlf=false，導致原CRLF被當新增trailing whitespace、大量假陽性；不是程式缺陷，不應批次改行尾。改沿repository原設定逐repo執行並分開保留exit code，只在顯示時摘要既有LF→CRLF warnings；未改git設定或檔案行尾。
- 操作錯誤再次猜 `src/replica.rs`、`scripts/tests/moba_launch_workflow_test.lua`／`moba_role_launch_test.lua`，實際是 `replica_host.rs` 與根scripts的 `test_moba_hero_selection.lua`／`test_moba_shared_selection.lua`；先列目錄再讀，不把rg零結果當檔案不存在或猜下一個名稱。
- 首次Grok packet patch在同一patch delete/add同一路徑被工具拒絕，未生效、未啟動worker；改Update既有packet加入明確本批唯一任務並把舊指令標歷史。不用shell覆寫或取消既有dirty。
- 大段OpenSpec/測試讀取曾截斷，不宣稱完整新讀；改讀精確來源與小段。詳見 selection-completion-budget-progress；Grok結果另記診斷progress。
- 已存PIE結果檔也曾猜為report.json，實際是pie-smoke-report.json；列實際目錄後只抽success／render／counts／pie-after字段，避免再次列完整nested results截斷。原生42x2含warnings，不把零failed當零warning。核對功能證據完成6.1（25/31），不把PIE120fps當正式60Hz效能證據。
- 共用time.monotonic_ms舊實作是os.time UTC且只有秒粒度，校時可能拉長/縮短deadline；改lua-host Windows machine uptime、跨程序同domain，Lua拒非法/倒退。不影響simulation/hash，不稱CPU/perf修復；非Windows明確unsupported而不fallback。Rust1及真跨helper Lua clock、相鄰selection期限通過。另誤猜lib/log.lua但不存在，未因失敗新增重複logging模块。
- restart IPv4 listener驗明後send不能再以localhost解析到未驗明IPv6程序；改固定127.0.0.1，移除私有send的多餘config參數。首次patch猜import版型未匹配，工具拒絕／沒有部分套用；依實際獨立use行修正。第一次check發現新增unused config警告即修，不取消檢查、不稱零警告。
- bounded結算觀測舊每500ms重讀整場日誌，長對局讀取量隨檔案成長；改有界append-reader、跨poll精簡合法歷史且保留矛盾檢查。新增65537-byte無newline測試揭露初版Lua greedy .*回溯平方成本，雖functional11/11最後完成但耗時，不把它當效率成功；改reverse+find線性搜索，不增加timeout。保留file truncate/disappear/IO/overflow fail closed；不是降低正式client50ms門檻或完整對局證據。

## E321：restart 舊 readiness／續作掃埠會選錯 Editor；測試入口與 TCP 關閉也必須嚴格（2026-10-06）

- 通用修正：wait-mcp 使用固定 Lua project/PID/listener gate，health 後再核對 executable 與 process birth，防同 PID 重用；auto-resume 的設定 TCP 埠也必須同一 Editor lifetime 持有，移除所有埠範圍 fallback。readiness 19 案例、endpoint 12、restart Rust 9 案例與真正無本專案 Editor 的拒絕確認通過。沒有命令其他專案、修改引擎或手改 BuildId。
- 操作錯誤：在 omfue cwd 用根目錄相對 Lua 路徑，並猜測 `--json`，造成找不到入口／CLI 拒絕；改絕對固定 Lua 與實際 `--output json`。另誤讀 `.ai-collab/state`，實際為 `state.json`；先用 rg --files 確認再讀。
- 新六類 cue TCP 重連測試首次失敗：寫 RendererShutdown 後立即 drop client／發布下一段 snapshot，舊 server 尚未處理關閉而寫入已 reset socket（10053/10054）。用 fixture 完成通道等待 serve 返回，再換 session；不把任意 I/O error 當成功、不修改正式 renderer gate。修後 1/1 通過。
- 精準 patch 曾匹配到另一個相同 `for _ in 0..2`，導致 session 不在 scope／E0425；回復該非目標 loop，改用相鄰唯一 context。不能只看 patch applied，必須編譯當前功能。
- 不把這次兩段 TCP 局部測試算兩場遊戲或完整 LAN／UE 視覺驗收。詳見 `2026-10-06-restart-readiness-and-cue-reconnect-progress.md`。

## E320：重連啟動與新輸入後證據不能共用一個耗盡的 deadline（2026-10-06）

- `final-perf-budget-20261006-v1`在新renderer完整HUD/Consumed/真實Point input status0 tick8916後仍逾時；required post-input門檻9036之後須兩個不同checkpoint，90秒包含Editor冷啟動及輸入，最後只剩不足下一個checkpoint的時間。報告success=false、cleanup保留，不把observed.complete=true等同全項成功。
- 共用wait_stages拆90秒renderer/input readiness與獨立30秒post-input parity，仍需雙隊/至少兩個不同tick/原hash及零repair，不放寬准入。啟動超時或後段超時都明確failure_phase、不能reset deadline無限等。17原觀測＋3時鐘fixture檢查；沒有覆寫舊timeout、沒有追加無限重試場次。
- 資源budget後server max76.135ms進原100ms門檻；client最大52.654/51.626ms仍超原50ms，全部其他門檻通過。不能說budget已解決所有尖峰，也不能在遊戲run失敗時勾6.5。須後續定量診斷，不挑通過樣本。
- Grok專案MCP委派run-muw8gwpc-qzt5lj等待13m36s仍無code diff，主agent收回。bridge stop報已不存在PID79120/exit128，但follower確認cancelled、tracked agent105696也不存在，無殘留writer。metadata `Stopped by user`是bridge制式訊息，本次是主agent決定，不冒稱人類要求。cost未知，不估數字；未接受實作，直接restart readiness仍須主agent接續。

## E319：Cargo exit0可能是零個測試（2026-10-06）

- filtered_specs::tests篩選並不存在，Cargo exit0但running0；沒有當成功測試。先定位實際team_replica_contract_tests中的filtered bootstrap/accepted move/rebase fixture，再精準執行。完整剩餘驗收不以filter錯誤縮小範圍。
- CPU預算功能不是已定因的profiler修復，受控hint不代表OS quota；正式來源已補workers log與launch provenance，維持v1門檻、舊尖峰失敗與單獨啟動原預設，避免調參挑樣本。

## E318：固定門檻後的獨立效能執行必須允許失敗（2026-10-06）

- `final-perf-validation-20261006-v1`正式60Hz重連/清理exit0，但雙玩家比較均exit1：server max133893500ns >100ms；client p1 max58001000ns/p2 max132987300ns >50ms。全部mean、IPC與UE門檻通過。原report/raw保存，不調大v1門檻、不排除這批樣本、不勾6.5。
- 三個Rust尖峰window同為05:25:15Z（server本地13:25:15），與首次renderer ACK同秒，不是約05:26:14Z的重連ACK；不能誤判成重連專屬問題。32 logical CPU，現shared create_thread_pool每個pool32threads；多程序競爭是一個待確認假說，尚沒有CPU profiler可定因，不以wall time等同pureCPU耗時。
- 本輪唯讀搜尋仍猜錯presentation_ipc.rs、tests/moba_role_launch_test.lua、OmWorldBridgeActor.cpp；真正IPC為presentation_bridge.rs、測試為scripts/test_moba_role_launch.lua。後續先rg --files盤點，再選實檔；空rg exit1不能算驗證成功。

## E317：正式 logger 的來源尾碼不能當成 JSON（2026-10-06）

- 真實60Hz renderer重連exit0且cleanup_verified，server/雙runtime/對方renderer均保持；第一次彙總讀server.stdout.log失敗，log4rs在JSON後加`(omobab::state::core 1529)`，不只是ANSI。原失敗report/raw logs保留。
- 通用parser僅剝尾端SGR與明確`(module::path line)`來源格式，再嚴格decode；未知尾碼、連續兩個JSON仍拒收，不截取到最後一個`}`硬吞錯誤。新增四斷言，88checks通過。修正後雙玩家六項完整、errors/missing0；thresholds尚未評估，不能以capture success計基線通過。
- 固定十二個同負載開發回歸門檻於versioned baseline JSON，再另開新同配置run，不用初次採樣冒充獨立驗證。IPC單位byte，不稱latency；UE DeltaSeconds不稱GPU；已觀測>16.67ms尖峰，不稱穩定60FPS。

## E316：驗證工具仍使用舊的單檔 Lua builder 載入方式（2026-10-06）

- PIE smoke在UI/Editor操作前因heroes.lua新增ctx.include而失敗，沒有啟動PIE；舊程式用空context執行builder。抽出與FFI生成同源的content_builder.lua，共用相對路徑/大小寫cycle/64層/parent escape防護，PIE工具和gen_hero_registry都沿同一loader；base_content build.rs補watch此module，避免loader修改後Rust產物過期。這只屬建置/工具Lua，沒有遊戲runtime VM。
- include測試8/8通過；新PIE輸出支援--out-dir NEW_DIRECTORY，拒絕覆寫，保留歷史證據。真實PIE新路徑FinalPie-20261006-includes-fixed成功啟動/生成英雄/mesh實際render/記憶marker實際render與1→0清除/截圖/停止。原生全套FinalNative-20261006-fixtures-fixed同Editor兩輪各42/42、failed0/skipped0通過，第一個崩潰report不覆蓋。
- 工具使用rg時誤把PowerShell檔名glob当實際path、猜不存在的registry/manifest/fixture header；之後先rg --files/--no-ignore取得已存在檔名，再用-g選擇，所有原失敗輸出保留。

## E313：基線變更須重新判斷；大型輸出不能視為完整讀取（2026-10-06）

- engine BuildId 已由上輪 b248… 改為 1323cea4-7408-4662-8321-6abdaf191604，不能沿用「同基線重試無效」結論。確認沒有本專案 Editor 載入後，以既有 Lua build-only / -NoEngineChanges 合法重建成功13 actions，UBT正常更新project與兩本地plugin manifest，沒有手改BuildId或重建共享engine。
- 新增只讀 ue_binary_preflight / check_ue_binaries，涵蓋missing/invalid manifest、BuildId、模組與DLL存在、unsafe filename、default/disabled plugin；完整build啟動前及role launcher選角前/開局前使用，不阻擋合法重新編譯。15個virtual-filesystem案例通過、真實重建前拒收/後ready。
- 初次讀plugin descriptor漏了合法UTF-8 BOM，造成誤報；修正僅移除解碼輸入的BOM，不改vendor檔，新增案例。長context/程序清單輸出遭截斷，分小區段補讀，之後只取需用欄位，不宣稱截斷內容已完整讀取。

## E314：固定 MCP 30000 可能是別的 Unreal 專案（2026-10-06）

- 本專案新Editor PID3800實際綁30001，30000為其他專案PID80700。原restart wait-mcp只確認HTTP可用，誤報ready；Blueprint首個唯讀get_pie_status逾時，full workflow exit1，保留target/blueprint-validation-runs/compile-1791262798。尚未執行compile/create等資產操作，不以此失敗歸因內容。
- 修正：ue_mcp依UECP/instances選精確project path；Lua-host使用Windows GetExtendedTcpTable確認實際IPv4 loopback owner PID及UnrealEditor.exe，GetProcessTimes與registry時間排除PID重用。每則請求再驗pid/port；missing/ambiguous/foreign/invalid owner fail closed，不回退30000。完整build的MCP ready改用同一project-bound get_pie_status。沒有停止其他專案。
- endpoint12案例、Lua-host6（含真實TcpListener建立/關閉/owner與invalid port）、launch contract8通過；初次stale-time fixture 102-1=101仍在180秒範圍卻預期拒絕，改為-100後確實測到超界。真實omfue get_pie_status成功，Blueprint11/11成功（compile-1791262947）。

## E315：NewObject UUserWidget 測試必須先 Initialize 再 TakeWidget（2026-10-06）

- 第一次project-bound native全套在OwnedInputUnavailableHud崩潰：UWidgetTree::ForEachWidget讀0x38，callstack精確指OmEditorAutomationTests.cpp:293；不是MCP網路失效。FinalNative-20261006-project-bound失敗report及UECC-Windows-E12ECCD5428DB46A8996FCA597DA27A4崩潰保留。
- 同檔另有Buff HUD fixture缺相同初始化。兩者加Initialize成功斷言再TakeWidget，遵循同檔原NativeMatchResult/Scoreboard做法；不改production UMG、不改共享engine、不略過測試。修復後編譯/必要確認結果待更新。
- 修復後-NoEngineChanges project build4 actions成功。手動om_restart start首次誤用repo根cwd，預設解析到不存在的D:/code/omoba/om.uproject，exit1未啟動任何Editor；改明確workdir omfue，之後保持既有Lua入口/精確cwd，不猜restart預設project。
- 重新啟動後尚未註冊就立即跑scoped測試，正確fail closed found0，沒有回退其他Editor；完整流程加入--wait-ready有界30秒等project-bound endpoint，不拿start返回當MCP完成。第一次stop仍running時提前build被工具正確拒絕exit4；等stop完成並確認PID退出才重建，嚴格串行。
- 動態HUD delegate在EditorPreview未初始化actor world時會被AActor::ProcessEvent拒執行，首個scoped斷言abilities=0不是production漏通知；兩HUD測試使用FEditorScriptExecutionGuard RAII。CompiledContentReloadPolicy把GameInstance subsystem建在transient package，改明確UGameInstance Outer。所有修正只在fixture，不變正式HUD/安全gate。找generated/header須rg --files --no-ignore，不猜Public路徑。
- endpoint lifetime僅要求registry寫入不早於owner程序出生；移除180秒上限，避免合法延遲載入插件被誤拒。舊101秒fixture與上限修正歷史保留，但現版本以晚PID出生拒絕重用、早出生可合法註冊；不是逾時放寬身份檢查。
- 最終修復編譯4 actions成功，project-bound --wait-ready成功（Editor PID22632）；三項OwnedInputUnavailableHud/DisconnectedHud/CompiledContentReloadPolicy單輪必要確認3/3、error0通過，report=omfue/Saved/McpAutomation/HudFixtures-20261006-final/report.json。沒有移除斷言，完整native結果另記。

## E312：分層三路需用正式角色配方驗證，舊雙英雄模式不等於十席對局（2026-10-06）

- 主agent額外分層map seed101 legacy Push/Guard在108000ticks/1800秒Playing逾時；last_progress_tick103699，不能稱deadlock或成功。原始失敗report及paired samples保留final-20261006-114908-layered-seed101。
- 末筆兩英雄105次死亡/0次死亡，仍有外路塔與雙基地；這是舊兩英雄模式觀測，不是五位置十Bot結果。正式五位置100場持續，不將該legacy run計入。
- 新增可重用moba_layered_archetype_match.lua，繼承原十席mixed archetype所有policies，只選既有compiled layered map；未修改英雄/塔/map stats、解鎖、勝負、防守或runtime source。直接沿用已凍結release binary/DLL匯出配方再跑，避開在DLL載入期間重新Cargo建置。舊失敗不覆蓋；新結果待實跑。
- 正式十Bot layered-role seed101也在1800預算逾時，last_progress107924距停止108000僅76ticks，不是objective stall。主agent依實際持續進展改用工具既有max3600上限做最後一次分層驗證，stall仍300、自然勝負/replay不變；新fresh report root帶budget3600，舊1800失敗保留，不將操作預算誤寫為遊戲參數或先計成功。
- 最終分層結果：同一正式十席配方、同一已凍結host/DLL、明示max3600/stall300，自然Finished152648ticks，winner_side1／winner_team2，replay152648ticks、final_digest=1f96829cad6398c943bd023234afe263354a8bc8c1d7ab95f89ce1bd4ee5456c，exit0。主agent另以固定Lua核對60Hz／10Bot／guard／預算、map_id、完整tick_digests數量及末digest通過。證據final-20261006-114908-layered-role-seed101-budget3600/result.json；不計入另行seed1..100批次、不冒稱1800成功或Unreal/filtered/LAN驗收。

## E311：目標存在判斷與 winner_team 把 side 索引誤當隊伍（2026-10-06）

- 問題：`objective_slots_progressed` 用 `entity_id != 0` 認定存活，但 specs Entity 的 ID 0 合法；缺席槽由 collect 編成 dead tuple `(0,0,0)`。成功報告把 `Finished.winner`（`Option<u8>` side 索引）直接寫進 `winner_team`。正式對局 `config.teams` 是 `[1, 2]`，seed1 自然終局卻報 winner team 0。`diagnostic_tests` 用固定 PID／temp 目錄，測試開頭 `remove_dir_all` 可能刪掉先前檔案。
- 決定：存在改看 generation 非零，不把 ID 0 放寬成缺席。`winner_side` 保留內部 side；`winner_team` 依 `moba_match.rs` game.end 對 `config.teams` 索引，draw 為 null，非法 side fail closed、不寫成功或失敗報告。Lua verify 拒絕 team 0，以及不在 role plan 的 `winner_team`；合法兩隊與真正 draw（`winner_team` 與 `winner_side` 都缺席，含 JSON null）允許。測試目錄改為唯一 `create_dir`，只刪本次已創建的具名檔，再 `remove_dir` 空目錄，不做遞迴刪除。不改 world 勝負，不改、不刪、不覆蓋 `omb/target/moba-headless-budget-seed1/result.json`（該檔仍是 winner_team 0）。
- 驗證：`cargo test --manifest-path omb/Cargo.toml -p omobab --bin moba-headless --features compiled-content-only -- diagnostic_tests` 7 passed、0 failed，測試 0.02 秒，編譯 5.05 秒。`D:\code\omoba\tools\lua\lua.exe scripts\tests\moba_headless_batch_test.lua` 10 passed，沒有啟動對局。`cargo build --manifest-path omb/Cargo.toml -p omobab --bin moba-headless --features compiled-content-only --release` exit 0，11.01 秒，`rustc 1.95.0`，沒有另建 DLL。沒有重跑 seed1、100 場或 UE。HEAD 未移動。完整 5.5 不勾。

## E307：600秒逾時不是死局，執行預算與目標無進展必須分開（2026-10-06）

- 問題：seed1 在 36000 ticks 仍 Playing 被當成可能死局。attempt2／attempt3 的 failure samples 頂層 `samples` 顯示 tick 28800→32400 仍有塔退休、32400→36000 基地 HP 仍下降。600 秒執行逾時沒有證明目標停止。舊失敗路徑只在既有 JSON `success=true` 時拒絕覆蓋，失敗 JSON 與非法 JSON 不算，而且成功與 plan-only 寫入仍會覆蓋。
- 決定：`--max-game-seconds` 預設 600、`--stall-game-seconds` 預設 300，兩者都是 60..=3600 的明確整數，stall 不得大於 max，profile 乘法用 checked。這是 headless 診斷預算，不是遊戲規則或勝利條件；逾時與 stall 都不能 success。進展只認公開塔／基地真實 HP 下降、退休或層替換，waves、英雄移動、擊殺、營地 respawn 與私有命令都不算。連續 stall 秒沒有這類進展才是 `objective_stall`，並寫下 `last_progress_tick`。報告與配對診斷用 create_new，既有成功、失敗或非法 JSON 都在開跑前拒絕，不刪歷史。
- 本批實作錯誤：Lua 測試先用 seed 1 的 fixture 去對請求 seed 4，verify 先因 seed 失敗，預算斷言沒被執行；改成同一 seed、預算仍是 600 後才測到 mismatch。`write_failure` 把借用的 error 字串丟進要求 `'static` 的 `err_msg`，編譯 E0521；改成擁有的 String 後重跑。
- 真實 seed1（只此一場，預算 1800／300，不是預設 600，也不是 100 場）：自然 Finished，winner team 0，finish_tick=62951（game_seconds 1049.1826171875），end_events=1，committed_combat_facts=10210，replay_verified_ticks=62951，final_digest=`80eede71fccbfd9ebf27d54e91775d11238f68b0a02c08a5ff18fe7a1de6bd94`，last_progress_tick=62951。exit 0，wall 136.10 秒。成功路徑沒有寫 paired diagnostic。這不能拿去充當 600 秒批次成功，完整 5.5 不勾。
- 預防：批次 verify 必須同時滿足請求的 max／stall、tick 預算與 finish_tick 上限；較長預算的自然終局不能冒充較短預算。不重試退讓，不改 Bot、ABI、地圖或角色數值。

## E309：fog lease fixture殘留舊ABI常數（2026-10-06）

- Bridge compiled-only library初次82通過／1失敗／1略過；authority_fog_grid_lease_owns_cells_busy_ring_and_reset期待11，正式ABI為16。
- 只改該fixture以OM_ABI_VERSION檢查當前header；不改正式ABI／config拒絕／lease ownership或reset斷言。主agent精準case1通過；不能把初次完整結果寫為通過。

## E310：輸入合併fixture不可凍結Support策略（2026-10-06）

- server compiled-only library初次167通過／1失敗／1略過。九Bot merge fixture寫死Support4/9不輸入；目前共用策略合法輸入包含4/9。
- 只改cfg(test)：用獨立formal role_bot_inputs輸出作合併預期，核對完整(player,action,request_id)與team投影，而非只放寬ID清單。human77/78順序／Bot request_id0／非法99與外部Bot控制排除／62ticks及真實移動斷言保留。主agent精準case1成功。不改productionmerge或授權。

## E308：統一Rust library驗收揭露fixture未同步（2026-10-06）

- 主agentcore compiled-content-only --lib實跑474：473通過，disclosed_authority_targets_do_not_reapply_lua_direct_damage失敗。fixture僅CProperty；正式direct damage加入護盾吸收，未插BuffStore而panic。保留disclosed authority不重做傷害與一般90HP兩個斷言，需補正式必要resource，不放寬production。
- base_content --lib210完成206通過／4失敗：三個塔projectile fixture缺ObservableFactBuffer，不是GamePause根因（dispatcher暫停只停on_tick，仍drain事件）。另一mana_lifecycle_60hz_filtered_regeneration_matches_authority_without_repairs不一致，left raw47593／right47185，max286720與tick17相同；需核對fixture與正式Buff/mana投影，不改期待數字掩蓋差異。保存真實錯誤，後續精準修復，不重跑全套直到受影響案例解決。
- 搜尋誤猜projectile_test_support.rs、scripting/dispatch.rs不存在；實際模組內嵌towers/mod.rs:248，dispatcher位於runtime/native/scripting/dispatch.rs。改目錄rg具體符號定位，不由模組名假定獨立檔案。
- 查略過測試時又猜omoba-client-runtime/src/ipc.rs不存在；此次不再擴大搜尋或把未列出的ignored算通過。只記實際library結果78passed／8ignored，後續先inventory。
- 第5批審查又猜moba_match/bots/config.rs、moba_match/mod.rs；rg --files確認主檔runtime/native/moba_match.rs、plan在moba_match/bots/plan.rs。winner side／team不能因名稱猜一致，已定位正式game.end映射再列限定修復。
- 修正只限cfg(test)：core fixture補BuffStore；塔共用pipeline fixture補ObservableFactBuffer；ranger_patch正式rank1 authored +2 mana/sec/6秒，fixture保留base bootstrap dt與cast後dt分開計算，沒有硬改actual期待raw值。逐tick hash／零repair等原斷言完整保留。主agent精準core1、mana1、dart5、ice1全部成功，不再重跑206秒全套；初次全套失敗與局部修復結果分開記錄。

## E306：驗收fixture須接真實共用契約，不能凍結舊英雄數（2026-10-06）

- stage fixture只允許require _bootstrap，正式入口已使用moba_stage_contract，初次exit1。改在同isolated environment載入真實stage模組，按路徑對應五artifact digests；bridge錯配／缺檔與base兩copy缺失／錯配均fail closed，六case成功。
- asset recipe fixture期待3heroes，正式catalog7heroes，初次exit1。改一對一完整目錄與unique ID，明確驗五training fallback；10jobs與Saika動畫共用斷言維持，成功。不改production容錯或資產。
- 工具預防：不要再以同patch Delete+Add同檔（被拒且沒有寫入），改Update。Windows rg用真實目錄配-g，不literal glob或猜文件名。
- 續作仍誤猜 scripts/build_bridge.lua，唯讀失敗；實際盤點為 scripts/build_ue_moba.lua 與 omfue/build_bridge.bat。後續 stage 決策先讀已盤點入口，不新增或執行猜測的 workflow。過量歷史搜尋再次截斷，改讀檔首具體 E306 段落。
- 停止Grok時bridge報PID94148不存在exit128；follower及runs確認cancelled，scoped查詢無相關worker。取消是主agent決定，bridge預設errorMessage為Stopped by user不可當人類指示；未終止其他程序。一次MD patch用未存在的#錨點被原子拒絕，改讀實際檔首後使用確切heading，無部分寫入。
- 診斷讀取：PowerShell foreach陳述式不能直接接pipe，先指派結果陣列再Format-Table；新配對診斷是object.samples陣列，不可直接把頂層當array。先讀JSON形狀再取末筆，不以空欄位推斷玩法狀態。
- 真實UE選角exit1另由E305 engine/plugin BuildId mismatch解釋，保留target/unreal-selection-tests/1791253029-1/error與原始log，不以mock通過替代。詳細全部結果見2026-10-06-final-acceptance-progress.md。

## E305：完整UE build與project-only成功不可混稱（2026-10-06）

- build_ue_moba --full已建置並stage compiled-only DLL及bridge，但完整UBT FailedDueToEngineChange exit4，列出共用引擎.modules待修改。沒有啟動Editor；與scoped project模組11actions成功分開記錄。
- 決定：保留-NoEngineChanges與使用者共享引擎修改；不手改BuildId、不將stage視為整體成功。唯讀核對實際engine/project module manifest與Editor啟動結果，不重複完整build撞相同閘門。

## E304：正式十Bot第一場逾時，100場尚未通過（2026-10-06）

- seed1、正式60Hz、moba_archetype_match配方、compiled-only DLL，36000 ticks仍Playing而exit1。batch completed_matches=0，逐場log及errors.md保留於omb/target/moba-headless-batches/1791252707-1。
- 決定：派Grok查真實失敗狀態與通用Bot決策，不延長600秒、不用withdraw／plan-only替代、不弱化replay。先精準確認seed1修正成功後再100場。完整5.5不勾。

## E302：正式 smoke 不可啟用 runtime Lua，批次報告不可覆蓋（2026-10-06）

- 問題：舊runtime smoke仍啟用Lua runtime與hot reload，雙UE商店30Hz未沿正式profile；逐次headless共用report會覆蓋證據。
- 決定：共用compiled-content-only建置／environment／TOML helper，兩個launchers關閉Lua runtime，shop presentation跟隨tick profile。新增build-once逐seed批次，每場獨立JSON/log，exact60Hz、十Bot、正式終局與replay一致才計成功；不允許重用既有output目錄，失敗保留已完成證據並寫errors.md。
- 局部結果：batch/helper6/6通過。TOML literal空白期待改為解碼值；bootstrap測試先用正規scripts path；PS不可用最後一個command exit掩蓋前者失敗。Windows rg不傳literal glob，不猜scripts/lib/ue_mcp.lua不存在路徑，先rg --files。一次含錯誤MD標題的多檔patch整批拒絕，核對原標題後重新apply，沒有部分寫入。
- 完整100場已啟動，未完成前不勾5.5。詳見2026-10-06-compiled-smoke-batch-progress.md。

## E303：跨玩家 capture、空 baseline 與整數邊界不能算成功（2026-10-06）

- 問題：同一份 capture 的 runtime 是 player 7/team 2、UE 是 player 4/team 1，收集器仍回 success。`muldiv_floor` 的中間 room 會拒絕合法速率 `floor(20000000 * 1000000000 / 9007199254740991) = 2`。`comparisons` 為空時 `threshold_result` 仍是 pass。`limit` 接受負數與小數。視窗沒有拒絕 `sum > samples * max`。`OmPresentationPerfSecondsToNs` 用 `Scaled > double(MAX_uint64)`；`double` 會把 `MAX_uint64` 進位到 `2^64`，等於 `2^64` 的值會通過後再越界轉成 `uint64`。本機 UE 5.8 的 `TestTrue`／`TestFalse` 只有 `(What, Value)`，三個單參數呼叫無法編譯。
- 決定：client、IPC send/receive 與兩個 UE 指標的 player/team 必須一致；各 component 的啟用契約必須與該段摘要一致；每條有樣本的 IPC connection 必須有相同身分的 enable。同一玩家重連多條 connection 合法，server 不參與。速率改為有界整數二元除法，至多約 53 步，有效結果仍 exact；不改 rate 定義、不用浮點、不提高 JSON 精度宣稱。空或非 array 的 `comparisons` 是 error，`threshold_result` 為 fail。`limit` 必須是非負精確整數。視窗用 `ceil(sum/samples)` 拒絕超過上界，避免 `samples * max` 溢位。秒轉奈秒在 cast 前以 `>= 2^64` 拒絕。三個斷言補上 label，數值用 `uint64` literal。不寫正式門檻數字。
- 預防：Lua `scripts/tests/moba_performance_report_test.lua`。C++ 只改轉換邊界與雙參數斷言，本輪不跑 UBT。
- 結果：Lua collector 84 checks passed。C++ 未編譯。沒有門檻數字。完整 6.5 不勾。
- 主agent後續獨立確認：Lua84 checks passed；scoped UE11 actions編譯成功，新增native斷言仍待Editor實際執行，完整6.5不勾。

## E301：正式量測不能把局部階段、證據 IO、KCP 或幀間隔冒充完整指標（2026-10-06）

- 問題：info 路徑沒有可彙總的正式 MOBA 量測。TickProfile 的 run/dispatch/outcomes 只在 debug 印出，而且那是局部階段和，不是 State::tick。註解裡的 30 TPS 不能當門檻。STEP_US_ 只在 test-mode evidence 寫入，且包含 evidence IO。localhost presentation 的 write_envelope 先前不計長度前綴與 payload。KCP inbound wire_bytes 被丟掉，不能拿來冒充這段 IPC。UE Tick 沒有把 game-thread DeltaSeconds 和 actor 呈現工作分開。缺段、空窗、NaN 或錯 scope 也沒有收集器擋下。
- 決定：60 筆視窗。server 只在已有 MobaMatch 且 gameplay active 的 State::tick 成功結束前記錄，減去每一次成功的 reliable_send_with_watchdog；scope 是 state_tick_excluding_transport_send，hash_input false，不含 scheduler sleep。client 在 apply_encoded_frame 返回後立刻停表，STEP_US_ 語意不變。IPC 以單一連線 sender/reader 成功邊界計數，失敗與截斷不計，不新增 KCP feature，測試端 write_envelope 不寫進 server meter。UE 只用 FPlatformTime；runtime 未啟動、catalog 失敗、未連線或暫停不入窗；gap 後第一個 DeltaSeconds 不記；Reset/Stop/新 handle 清窗。約 60 筆才 log 一筆 OM_PERF JSON。收集器缺段或零時長速率為 unverified，非法值、錯 scope、p95，以及把 mean 當百分位為 error。門檻只接受外部 baseline 的 mean 或 max。
- 預防：Rust filter formal_perf、Lua scripts/tests/moba_performance_report_test.lua、C++ Om.Generated.PresentationPerfWindow。本批不跑 UBT／PIE／一場正式對局，不寫門檻數字。完整 6.5 不勾。
- 結果：omoba-core formal_perf 4 passed；client-runtime formal_perf 2 passed；Lua collector 60 checks passed；omobab compiled-content-only check exit 0。C++ automation 只新增、未執行。

## E299：協作期間HEAD可被其他工作移動，不能以舊diff當本批結果（2026-10-06）

- 問題：Grok本批期間，另一提交工作收錄既有dirty變更，root／omfue HEAD移動，diff從大量歷史增量變為5個C++檔；另有其他Codex啟動Editor。未核對就reset、kill或再dispatch會破壞使用者工作。
- 決定：暫停下一寫入dispatch，讀git log/stat與聊天「分批提交並整合所有改動」狀態／任務，確認是限定提交工作且已idle；保留新HEAD cb0a5600／2098776，審查当前Grok增量。未發消息、未commit／push／reset，不關非本輪Editor。獨立scoped UE13 actions成功，native只編譯。
- 交接準備錯誤：同patch同target Delete＋Add被apply_patch拒，改單一Update；猜OmGenerated插件根導致rg os error2，改rg --files定位真實OmRuntime/Source目錄。Node bridge DEP0190警告記錄為工具警告，不宣稱遭利用。
- Grok job run-muvyl79j-i29ns7完成，13m18s；下一批以核對後baseline派發，不繞bridge conflict guard。本輪不啟動全驗收／stage，見grok-completion-plan與grok-owned-ui-progress。

## E298：斷線清HUD漏掉Buff列表，物品與商店／小地圖失敗未通知畫面（2026-10-06）

- 問題：InvalidateOwnedHud已清Hero／四技能／六物品／Economy，BuffListSnapshot仍留在bridge與command bar。SubmitOwnedItemUse斷線只清物品baseline。施法在游標失敗時還沒做連線檢查。商店按鈕、買賣與小地圖斷線只拒絕輸入，畫面要等下一Tick。
- 決定：同一InvalidateOwnedHud抑制owned HUD並發布空BuffListSnapshot；只有非空列表拉起dirty latch，publishing guard避免清除重入或每tick洪泛。物品、游標施法、升級、回城與bridge買賣在未Connected時走這個入口。Command bar商店／小地圖只在runtime未連線時反射NotifyOwnedGameplayUnavailable；仍連線但超出商店或投影非法不清除。control-only與被抑制的retained frame不能恢復已清ready。不重啟後端，不改ABI。
- 預防：DisconnectedHud補Buff清除、去重與Stop只通知一次。新OwnedInputUnavailableHud涵蓋八個入口各通知一次、第二次不洪泛，並把空snapshot套回command bar。native斷言未執行。
- 不跑UBT／Cargo／PIE／stage。版本hash與21/31保持，完整6.2／6.4不勾。見grok-owned-ui-progress。

## E297：拒絕追擊後須改選可達候選，不可移除警戒披露（2026-10-06）

- 問題：正式命令會拒絕不可達AttackTarget追擊，但Bot下一think可能再選同一目標。
- 決定：共用combat決策依原角色優先及focus，最多8個候選；失敗ID只在本次決策排除，保留完整披露供兵線／塔前警戒，改選其他合法候選，全失敗正式Hold去重。射程內普通攻擊不查路或新增LOS；技能選擇不套追擊導航。不建立永久黑名單、不讀敵方私有ECS、不以partial path放行。
- 編譯錯誤：新fixture地形閉包參數未標型別，推斷f64運算不符合Vec2<f32>，E0271；明確標x:f32，不改正式BlockedRegion型別或取消測試。
- 局部結果：core2與base4通過，新正式60Hz1內含四兵線位置、相鄰Carry前搖／Jungle farm focus／Support guard各1。完整100場／PIE／LAN／stage留最後，21/31與完整5.5未勾；見bot-combat-navigation-progress。

## E296：護送也須導航准入，已驗攻城不能在提交時重查（2026-10-06）

- 問題：Escort不查路徑；它與ApproachStructure共用提交分支，若一起加導航會使攻城候選最多8次之外再查第9次。
- 決定：只對未驗證的Escort查本人半徑／公開完整路徑，失敗正式Hold去重，不猜未披露Carry位置；地形恢復普通MoveTo。拆分提交分支，ApproachStructure保留已查結果、不重查，不加budget、不改協防／自保優先。
- 預防：新正式60Hz受阻／去重／未披露新位置／恢復矩陣及相鄰護送／協防。3 tests通過，無新編譯或測試失敗，既有template三個dead_code warnings保持。
- 不跑100場／PIE／全驗收或stage，版本hash與21/31保持，完整5.5未勾。見bot-escort-navigation-progress。

## E295：兵線不能保留不可達waypoint，巡邏與推進排序不可混用（2026-10-06）

- 問題：Jungle已查公開完整路徑，四兵線位置Advance仍直接提交／保留不可達AttackMove。
- 決定：共用有界候選迭代與8次查詢預算、本人半徑及公開導航；兵線preferred後只向前，野區維持ring，不環狀回头。全失敗正式Hold且去重、地形恢復重選、不建立永久黑名單；既有戰鬥／塔前等待／護送優先不變。
- 預防：純函式排序／budget／抵達／空或非法route矩陣與四位置正式60Hz命令／Hold／恢復案例；fixture用公開compiled waypoints及正式輸入，未重複E294私有欄位或缺command假設。
- 局部結果：core2與base3（新60Hz1含四位置、相鄰原bend1／Jungle patrol1）通過，無新編譯或執行失敗；不跑100場／PIE／全驗收或stage，版本hash與21/31保持，完整5.5未勾。見bot-lane-navigation-progress。

## E294：撤退也須共用完整路徑判斷，不可阻斷自保（2026-10-06）

- 問題：sustain Retreat未檢查公開路徑，無效MoveTo分支仍continue，地形封住基地時自保技能被餓死，舊攻擊可能繼續。
- 決定：五位置共用本人半徑／公開地形的有界導航；不可達時封進攻、保留合法自保優先，無自保才正式Hold且去重，地形恢復重新普通MoveTo，不建立永久黑名單。定身原等待保持。
- 工具錯誤：再次對Windows literal glob使用rg造成os error123，改rg --files定位；fixture誤猜last_known_pos，編譯前讀真實HeroCommand改chase_origin。先定位真實檔與讀宣告，不放寬型別或移除斷言。
- 編譯錯誤：fixture讀MobaMatch.routes私有欄位，E0616；改從正常role_bot_inputs產生的MoveTo取得正式目的地，不擴大production內部可見性。
- 執行錯誤：fixture假設spawn已有HeroCommandQueue，get_mut unwrap失敗；明確insert測試初態（舊AttackTarget及queued），再由正式Hold清除，不改production spawn。
- 局部結果：新60Hz案例1通過（五位置×有／無自保10組），相鄰定身／基地恢復／回城受控3通過，合計4 tests；既有template dead_code warnings保持。不跑100場／UE全验收或stage，未部署release DLL，21/31與完整5.5未勾。見bot-sustain-navigation-progress。

## E293：文字局部fallback不能取代frame准入，長度不得縮窄（2026-10-06）

- 問題：非法string ref只回空字串後frame仍ACK，unsigned ref／FX view長度直接cast int32。
- 決定：OmFrameStringRefFits以寬整數驗offset＋len及converter MAX_int32容量，len0保持absent offset-unused；共用frame gate查所有frame refs（含nested projection／inventory）及FX view storage／length，更新前拒收。不要求NUL、不任意截短、不宣稱UTF-8內容驗證或pointer配置長度證明；讀取器也補縮窄防禦。
- 預防：新FrameTextContract逐欄位注入／恢復、邊界／overflow／容量／FX檢查、helper不apply／ACK矩陣；已編譯未執行。
- 結果：UE限定9actions exit0、Lua syntax／whitespace通過，無新編譯失敗；未PIE／stage／全驗收、不重試E285 Cmd。版本hash與21/31保持，完整6.4不勾。
- 詳見 `2026-10-06-frame-text-contract-progress.md`。

## E292：route局部跳過不等於frame原子准入（2026-10-06）

- 問題：map route非法range僅由局部模型跳過，frame其餘呈現仍更新且ACK，與polygon整批拒收不一致。
- 決定：OmFrameRangeFits共用uint64 start＋count邊界，OmValidateFrameContract在更新前查全部route／polygon spans；越界整批拒收，不新增任意point cap或更改模型語意。合法尾端空range允許，超capacity起點拒絕。
- 預防：新FrameGeometryRanges native矩陣含空值／尾端／overflow／雙類別拒收、真實actor不得部分移除及合法後續恢復；僅編譯未執行。
- 局部結果：UE限定9actions exit0，Lua syntax與whitespace通過，無新編譯失敗；未PIE／stage／完整驗收，不重試E285 Cmd。版本hash／21/31保持，完整6.1／6.4不勾。
- 詳見 `2026-10-06-frame-geometry-ranges-progress.md`。

## E291：霧快取不能只用origin與數量，失敗不可提前commit（2026-10-06）

- 問題：格內移動、第二viewer、半徑及同數量遮擋變形未更新；無viewer隱藏後同origin可能不重建。polygon range未經共用准入，segment可能越界。
- 決定：完整公開霧幾何的精確length-delimited陣列鍵、含world scale，不依sequence／padding／可碰撞hash；finite與int32 origin裕量檢查。共用frame gate以uint64驗polygon spans；無viewer／非法／重設清鍵，重建前隱藏舊plane，成功顯示後才commit。
- 預防：native鍵矩陣含格內／多viewer／半徑／tree／polygon／scale、overflow／NaN／極端origin及恢復；只編譯未執行。
- 局部結果：UE限定9actions exit0，Lua syntax／whitespace通過，無新編譯失敗；未PIE／stage／效能或全驗收，不重試E285 Cmd。版本hash與21/31保持，完整6.1不勾。
- 詳見 `2026-10-06-fog-geometry-key-progress.md`。

## E290：斷線只清內部ready仍會留下舊HUD，測試須使用實際事件契約（2026-10-06）

- 問題：E289只清owned baseline，沒有通知技能／物品UI；retained frame可再發布HUD。
- 決定：InvalidateOwnedHud同源清輸入並發布Hero／四技能／六物品／Economy空HUD，dirty latch與publishing guard防每tick洪泛／重入；Stop強制同源清。Tick斷線抑制owned HUD但保持world／frame消費，不重啟後端，恢復須新baseline。
- 錯誤：新fixture把BlueprintImplementableEvent誤當BlueprintNativeEvent覆寫_Implementation，C3668。讀實際UFUNCTION後改UFUNCTION dynamic delegate listener，不改production API或移除斷言；以後測試先核對事件宣告種類。
- 結果：修正後限定5actions成功，Lua syntax／whitespace通過；新DisconnectedHud案例僅編譯未執行，未PIE／stage、不重試E285 Cmd。版本hash／21/31保持，完整6.2／6.4不勾。
- 詳見 `2026-10-06-disconnected-hud-progress.md`。

## E289：NaN／負冷卻不能當ready，斷線owned baseline必須失效（2026-10-06）

- 問題：僅remaining > 0的檢查放過NaN／負值，施法／升級／回城缺共同即時連線准入。
- 決定：共用finite、非負、remaining<=total且remaining==0檢查；三owned入口要求started＋Connected，失敗清四槽／ready；Tick斷線前後與Stop共用清除。完整owner HUD且Playing／finite正HP才建ready，control-only不重建；清舊InputId，回城明確configured owner，不預扣技能點或冷卻。
- 編譯錯誤：新增SubmitOwnedRecall本地Bridge後，舊日誌if initializer遮蔽同名變數，C4456。改沿用已驗證變數；以後擴充入口先檢查既有區域宣告，不放寬編譯警告。
- 當前Lua syntax／whitespace通過，native冷卻矩陣已編譯但未執行；修正後限定UE4actions、Result: Succeeded、exit0。不重試E285 Cmd、不PIE／stage；版本／hash與21/31不變。
- 詳見 `2026-10-06-ability-input-policy-progress.md`。

## E288：被禁止的 runtime Lua 請求不可先清快取或繼續掃作者檔（2026-10-06）

- 問題：Rust compiled-only拒絕reload，但Unreal仍設定ENABLE_LUA_RELOAD／遞迴掃Lua檔，WorldBridge請求先清ClassCache／LastAttackStates才呼叫bridge。
- 決定：移除watcher／scanner／debounce與狀態、config flag設定；保留舊公開API與config欄位作資產相容，欄位deprecated且不再可編輯，API一律Disabled＋生成編譯指引、false，不呼叫DLL、不改呈現cache。Subsystem僅更新診斷。
- 預防：新CompiledContentReloadPolicy案例涵蓋OutStatus重設、兩入口一致拒絕、不啟動runtime、保留baseline；正式Lua只用於build／authoring，不恢復VM。
- 局部結果：UE限定14actions exit0、Lua syntax與whitespace通過；scanner／DLL reload呼叫／flag設定搜尋零結果。native斷言僅編譯未執行，不重試E285 Cmd、未PIE／stage。無新編譯失敗，版本／hash与21/31保持。
- 詳見 `2026-10-06-unreal-compiled-reload-policy-progress.md`。

## E287：呈現生命週期清理不可分散，重複Start不等於新runtime（2026-10-06）

- 問題：Stop未清技能／回城ready及四槽資料、tree／polygon／fog與部分HUD快取；EndPlay清理副本更短。StartRuntimeFromSettings同handle存在時回true，任意重設會重播或丟失未ACK狀態。
- 決定：ResetRuntimePresentation作唯一入口，Stop與EndPlay共用、新handle建立前先清；同handle重複Start保留消費與物品baseline。清四技能HUD／ready／IDs／cooldown、選塔、runtime與HUD快取、樹／polygon／fog與幾何快取，沿既有routes／entity／ghost／terrain清理；不重啟後端、不刪資產。
- 預防：RuntimePresentationReset案例編譯涵蓋停止／重複停止／同baseline恢复；正式Stop總會重設frame消費與cue歷史。
- 局部結果：UE限定8actions成功、Lua syntax與whitespace通過；新native斷言未執行，不宣稱PIE或功能測試通過、不重試E285 Cmd。無新編譯失敗，既有plugin warnings保持。版本／hash及21/31不變，完整6.4不勾。
- 詳見 `2026-10-06-runtime-presentation-reset-progress.md`。

## E286：ACK失敗要重試，但不能重複套用呈現（2026-10-06）

- 問題：Tick在MarkFrameConsumed失敗後仍推進唯一游標，後續同frame完全跳過，無法補ACK；單純延後游標則會重播呈現。
- 決定：共用FOmFrameConsumption分開apply與ACK commit，同frame失敗只重試ACK；成功duplicate無工作，新frame正常處理，Stop重設。所有取得的lease在回傳前release，不延後持有或建立無界queue。
- 安全：duplicate亦先shape gate，frame／lease sequence必須相同，拒收／apply失敗不得ACK；sequence0以具名初始化旗標處理。stats只回已套用游標，不冒充ACK完成。
- 當前結果：UE限定8actions exit0，Lua syntax與whitespace成功；native矩陣僅編譯未執行。未重試E285 Cmd入口、未PIE／stage／全验收；無新編譯錯誤。版本／hash與21/31保持。
- 詳見 `2026-10-06-unreal-frame-consumption-progress.md`。

## E285：frame必須先完整shape准入，拒收不可仍ACK（2026-10-06）

- 問題：ProcessFrame只檢查null便更新呈現，Tick無條件ACK／推進游標，晚期空陣列可造成部分更新或解參照。
- 決定：共用OmFrameContract先size／ABI、snapshot0／1、全部21個count／pointer與nested fog storage；不設任意entity cap、不宣稱pointer足以證明allocation。ProcessFrame回傳成功，Tick成功才ACK／推進，總會release；錯header不先讀sequence。拒收保留合法呈現、診斷節流2秒，下個合法frame恢復，不重啟後端。
- 預防：synthetic與正式共用gate，fixtures填正式header；新native矩陣含真實actor不得部分移除。UE限定8actions編譯成功，固定Lua runner語法與兩repo whitespace通過。
- 執行錯誤：獨立UnrealEditor-Cmd.exe scoped FrameContract立即exit1，只有bundled DotNet輸出且指定log不存在；後續Get-Content亦不存在錯誤。原因未證實，不反覆重試、不稱native斷言已通過；啟動失敗時先確認實際產物存在，不猜測讀log。沒有PIE／stage／完整验收，原生斷言待最後執行。
- 版本與hash不變，21/31維持，完整6.4不勾選。詳見 `2026-10-06-unreal-frame-contract-progress.md`。

## E284：catalog不相容不能只記統計／警告，空簽章不可放行（2026-10-06）

- 問題：RefreshCatalogGeneration把空surface當相容、內容hash不同只warning；bCatalogCompatible=false僅回報stats，Tick仍處理與ACK frame。
- 決定：共用OmValidateGeneratedCatalog嚴格檢查struct_size／ABI16／非零且匹配lease generation、兩個16字元小寫hex且有界string refs、精確compiled content／surface。沒有空值fallback或警告後繼續；不讀任意長UTF-8，不把offset＋len用u32相加。
- 真正接線：StartRuntime建立本地bridge後立即查catalog，失敗StopRuntime並回false；Tick先刷新／驗證，錯配先release catalog再停止本地bridge、清呈現，不取得／消費frame。已有相容catalog而本次Acquire失敗僅跳過，不當成已證實錯配。每次取得均驗證，不僅generation變更時檢查。
- 生命週期：相容旗標預設false，Start／Stop清catalog游標、Stop清frame游標，防止新runtime的generation1／sequence重用被舊值跳過。只停止本地bridge，不重啟或终止後端對局。
- 局部結果：UE限定8actions成功；新增原生CompiledCatalogContract矩陣已編譯、尚未執行Editor斷言，加入既有Lua scoped runner清單且syntax確認通過。未PIE／stage／全驗收，不宣稱native測試執行通過。無新編譯失敗。
- 本輪strict compiled registry亦不接受舊開發模式以content mismatch維持相容；換內容需生成並重建。hash／ABI／wire／IPC保持E283版本，完整6.4與21/31維持。詳見 `2026-10-06-unreal-compiled-catalog-gate-progress.md`。

## E283：生成介面簽章必須涵蓋真正公開宣告，而非只看類別名稱（2026-10-06）

- 調查更正：最初懷疑數值會誤觸介面變更；現有測試證明並非如此。真正缺口是class_surface_signature只有kind／ID／class與固定generator版本，metadata新增欄位未必改簽章。
- 決定：以generated-public-surface-v2 domain、穩定內容類別清單與全部8個實際生成header計算；header依檔名排序、名稱與宣告長度framing，不簽數值initializer、資產配方或角色summary。style宣告抽同一helper供輸出與簽章，避免副本漂移。
- 局部測試：數值／名稱／美術變更保持surface、每個header新增宣告均改surface、legacy reflected宣告存在與否改surface；實際8個輸出header反算與report一致。未知未來新增header若漏簽會被數量／結果對照測試抓出。
- 操作錯誤：新bridge fixture誤猜RuntimeInner.content_hash，E0609；讀實際型別後改generated_content_hash，不增正式欄位掩蓋錯誤。UBT遇既有Build.bat，等自然結束、不終止未知程序。
- 結果：generator2／bridge原子拒絕1、正式生成check17檔與UE限定3actions通過。bridge在錯配拒絕後維持舊hash／surface／generation；直接測core gate不啟用runtime Lua。生成presentation hash改d5553bbb31c459a4、surface1e2dbe976a9ee6d7；identity／data、ABI16／wire6／IPC5保持。
- 未stage、PIE、完整重建部署或驗收，不能把簽章更正當完整2.2b；詳見 `2026-10-06-generated-public-surface-signature-progress.md`。

## E282：通用技能投影不能繼續以角色專屬 ABI 命名；smoke 版本不可過期（2026-10-06）

- 盤點：任意Buff binding／正式AbilityArea已用同一投影，並非尚未實作的Saika限定效果；不得憑名字再建第二個adapter或變更事件語意。
- 決定：OmSaikaAbilityProjection／OmAbilityEvent.saika改OmAbilityProjection／projection，同步Rust writer、lease測試、cbindgen export、真正Unreal dispatch與C++ header smoke。資料欄位及順序、cue身分／flags、視野gate與單位換算保持，不移除saved Blueprint reflected名稱。
- C ABI15→16，現有strict gate拒絕所有舊版本；不以欄位大小暫時相同為由接受舊DLL。wire6／IPC5／內容hash不變，最後統一重建部署。
- 發現header smoke仍static_assert ABI11，已同步16及通用投影使用；本輪沒有單獨執行該smoke，不把更新原始碼當執行成功。
- 局部結果：投影11、header／ABI gate1、正式area投影1，共13項通過；cbindgen生成成功、UE限定10actions成功。沒有新增編譯／測試失敗。搜尋無匹配的exit1是盤點結果，不當成建置失敗；輸出截斷不宣稱完整審閱。
- 未PIE／native automation／stage／release全建置；整項2.2b及總21/31維持，其他typed payload／歷史summary與saved assets引用仍待。詳見 `2026-10-06-generic-ability-projection-abi-progress.md`。

## E281：英雄作者 metadata 與渲染 fallback 不可混為一談（2026-10-06）

- 問題：舊英雄 metadata 仍從 SaikaSummary 另生數值／美術清單，通用 metadata 不完整。
- 決定：所有英雄由 validated HeroEntry 生成 Strength／Agility／Intelligence／射程／轉速及作者 render 欄位；舊 GetSaikaMagoichiMetadata 僅轉接通用欄位，BaseHp→BaseHealth、AbilitySlots→AbilityIds。不改 native_visual 的資產路徑／cm換算／fallback，不刪 reflected 名稱或恢復舊事件派發。
- 首輪編譯 E0599：新 fixture 誤用 Manifest::default，該型別僅有 serde 欄位預設。改用既有 Deserialize 建立空 manifest，不為測試修改正式型別。
- 第二輪14/15：測試誤認省略 render.scale 為1；實際共用 HeroRender 與舊summary皆為0。修期待0，不放寬驗證或把實際渲染 fallback 注入作者 metadata。來源鍵以 BTreeMap 順序保持，旋轉保持 Pitch／Yaw／Roll。
- 最終15/15、正式生成／check各17檔、限定Unreal模組9actions成功。identity／data／presentation hash不變，未PIE／native automation／stage；歷史summary與其他typed payload／saved assets引用仍待，2.2b未勾、21/31保持。
- 詳見 `2026-10-06-generic-hero-metadata-adapter-progress.md`。

## E280：相容技能資料不可保留第二份角色數值生成（2026-10-06）

- 問題：通用技能 metadata 已存在，但 GetSaikaAbilityMetadata 仍從歷史 summary 生成技能 ID／數值分支，兩份輸出可能漂移。
- 決定：通用 FOmGeneratedAbilityMetadata 加入建置期序列化的 ExtrasJson；舊 reflected API 僅呼叫通用查詢並轉接欄位、FName 與逐級資料。不移除可能被 saved Blueprint 引用的名稱，不恢復 legacy 事件自動派發；未知技能沿通用空結果。
- 首輪 metadata 局部群 13/14：既有 animation_registry fixture 的空片段沒有 source，遭正式 validator 拒絕。補上明確來源、30Hz 與合法區間後 14/14；不放寬來源／時脈驗證，不改正式動畫內容。
- 操作錯誤：猜測 OmWorldBridge.cpp、codegen/src/Cargo.toml 路徑查詢失敗，後者曾用 SilentlyContinue。後續須先列實際檔名，不隱藏讀取錯誤；大段歷史 artifacts 輸出截斷不得宣稱完整重讀。
- 結果：正式生成／check 各17檔、限定 Unreal 模組9 actions成功；identity／data／presentation hash保持。未PIE、native automation、stage或完整驗收。英雄完整 typed metadata、歷史 summary、其他 typed payload／資產引用仍待，不勾完整2.2b，21/31保持。
- 詳見 `2026-10-06-generic-ability-metadata-adapter-progress.md`。

## E279：營地巡邏不可反覆送不可達點；命令接受與位移階段要分開驗（2026-10-06）

- 問題：巡邏保留最近／active點，但導航失敗後沒有候選交接，可能下次再選同一點。
- 決定：只在最終Jungle Advance分支從preferred起沿公開camp ring查共用完整路徑，最多8次；已抵達跳過，無結果沿持續Hold，地形恢復重新判斷，局部combat／siege仍優先。保留兵線waypoint順序，不新增黑名單／計時器／傳送。
- 首輪正式60Hz測試assert_ne失敗：新AttackMove接受時同tick movement phase已結束，Pos仍是held位置。修fixture為先核對active命令、再正常下一tick核對位移，不改正式phase或把接受當完成。
- core3及正式60Hz1最終通過；原camp座標不搬動，只加入公開地形fixture。預算是每selector8次，siege與patrol不可冒稱共用全think8次。完整5.4／5.5留最後，見 `2026-10-06-jungle-patrol-navigation-progress.md`。

## E278：權威拒絕受阻位移不等於Bot能跳過注定失敗的政策（2026-10-06）

- 問題：ApproachEnemyPoint只依距離選敵，公開薄牆受阻仍反覆佔用政策優先序；正式拒絕雖安全，不能代替Bot候選准入。
- 決定：只對位移意圖注入共用swept public terrain predicate，排除受阻候選後重用原focus／距離排序，全受阻continue下一作者政策；普通AoE不加旅行限制。
- 防錯：script collision adapter缺半徑預設30，普通命令預設20；本輪明確使用前者，不把E277繞行導航套到直線位移。不要讀hidden dynamic state、不預扣CD／魔力、不改Pos或把planning視為成功施法。
- core2／base4（含正式60Hz3）通過，無新編譯或測試失敗。新fixture明確薄牆→恢復、地形恢復／hidden gate→正常位移，完整5.5留最後。詳見 `2026-10-06-bot-relocation-terrain-progress.md`。

## E277：旅行選擇必須接共用導航的完整路徑契約（2026-10-06）

- 問題：E276只依距離選攻城目的地，公開地形使最近目標導航失敗時，後續思考仍可能重送同一目標。
- 決定：當前合法候選穩定排序後依序用共用static_next_waypoint查本人半徑／公開地形，最多8次、找到即停；失敗跳候選，無結果回原巡邏。不造另一份導航、私人cache、黑名單或傳送規則。
- 防錯：有界搜尋成功才授權新旅行；失敗不宣稱全域不可達。地形恢復會重新考慮，同樣不能從hidden cache選替代目標。純focus不執行導航，定身不查路。
- 查詢又猜測native/tick.rs而失敗；rg --files找到tick/mod.rs。後續不可猜mod.rs／同名rs的布局，先列檔再查。
- core3／正式60Hz base3通過，沒有新編譯／測試失敗。公開地形fixture保持塔原位置，只確認當前功能；完整5.4／5.5留最後。詳見 `2026-10-06-jungle-siege-navigation-progress.md`。

## E276：局部攻城不能取代接近目標的旅行政策（2026-10-06）

- 問題：E275僅讓Jungle在550內攻建築，遠处已披露推塔機會仍被營地巡邏略過；不能將近距離測試通過宣稱能主動接上兵線。
- 決定：保留協防／野怪及局部Hold優先，無局部行動後選current合法建築，塔仍需current活同隊兵，最近距離／canonical ID稳定排序。新旅行意圖沿共用MoveTo，抵達再交接AttackTarget，不以AttackMove意外取得兵線目標。
- 防錯：latest baseline不等於目前披露；正式60Hz fixture分別移除tower與wave的current身分且保留cache，確認不能建立新攻城旅行。使用原位建築，不搬塔或改unlock state。
- core2及正式60Hz1通過，無新編譯／測試失敗。查閱大型歷史計畫時合併工具輸出被截斷；不得把被截斷輸出宣稱已完整重讀，後續應分段獨立讀取。完整5.5及最終部署仍待，見 `2026-10-06-jungle-siege-travel-progress.md`。
- OpenSpec首輪strict失敗：Scenario插在下一Requirement與其SHALL本文之間，解析為missing requirement text。已移回Bot Requirement下，保留單機Requirement本文緊接標題；之後新增Scenario不可拆開標題與需求本文。

## E275：Jungle 不應在協防／野怪之外永久略過共用攻城（2026-10-06）

- 問題：Jungle在沒有野怪時直接Advance巡邏，完全跳過已披露塔／基地選擇，無法參與局部建築推進。
- 決定：保留協防→野怪的focus優先序；沒有focus時使用同一decide攻城分支。Jungle一般候選只允許已披露可攻擊建築，不因此新增無協防條件的單挑英雄／搶兵政策。
- 安全：塔仍需要current存活同隊兵的650觀測支援，無支援1100內Hold；base只接受kind5，鎖定kind6／死亡／己方／超距／hidden均不當攻城目標，不讀私有營地計時或建築state。
- 當前core3與正式60Hz base1通過，無新編譯或查詢失敗。正式fixture使用原位mid塔，不搬動建築碰撞／state；只證明正常Hold／AttackTarget准入，不冒稱完整終局。見 `2026-10-06-jungle-siege-progress.md`。

## E274：搜尋耗盡不可用最近格偽裝完整可達路徑（2026-10-06）

- 問題：static_next_waypoint未抵達goal仍從best-distance格重建路徑。封閉終點外的x64可接近，但不能到終點，原流程仍回傳Some使角色走向死路。
- 決定：僅找到goal或合法精確terminal edge時設定reached，再重建第一步；耗盡返回None，occupied target共用前置拒絕。移除最近格score副本，搜尋margin／span／neighbor排序及碰撞不變。
- 邊界：None只代表既有有界規劃無完整路徑，不宣稱全域不可達。英雄沿原queue.advance處理，NPC原static_step停留；不新增傳送、計時器、永遠擴大搜尋或假成功。
- 當前四項指定core測試通過，含60Hz不可達MoveTo／AttackMove不走死路、後续合法佇列抵達、同格繞行及NPC正常繞行回位。無新編譯或查詢失敗；完整5.4留最後。見 `2026-10-06-navigation-complete-path-progress.md`。

## E273：同導航格不代表實際端點直線可達（2026-10-06）

- 問題：static_next_waypoint在start==goal時只檢查直線，薄牆分隔同格兩端即拒絕；舊測試也把同格拒絕當預期，會掩蓋可繞行路徑。
- 決定：同格clear direct維持快路徑；受阻且終點未被占據時沿既有margin／span有界BFS搜尋，實際target作為獨立terminal edge，避免goal與已visited start混為同一節點。所有連線仍做swept-circle檢查，不傳送、不讀隱藏動態單位。
- 防錯：牆內target保持拒絕，超跨度受阻fallback保持拒絕，不用放寬碰撞或無限擴大搜尋來通過測試。
- 查詢再度猜錯hero_tick.rs；實際檔案由rg --files找到tick/hero_move_tick.rs與hero_command_tick.rs。讀檔先列目錄，查詢失敗不是編譯錯誤。
- 當前三項指定core測試通過，無新編譯失敗；完整三路／filtered／效能驗收仍待。詳見 `2026-10-06-same-cell-navigation-progress.md`。

## E272：Bot 回魔不能搶過緊急防禦或取消普通前搖（2026-10-06）

- 問題：合法ItemUse會清除既有command；上輪回魔priority0即使低HP與可用防禦共存仍搶先，也可能取消正在準備的普攻。
- 決定：按效果種類閉集排序護盾→減傷→逃生衝刺→回魔→下一擊準備，槽位僅同類tie-break。回魔與下一擊準備在Windup延後，紧急防禦仍可打斷；只使用本人狀態與current披露威脅，不改人類或權威ItemUse規則。
- 首輪正式60Hz測試失敗：出生fixture未必帶HeroCommandQueue，get_mut.unwrap在1520行失敗。改為明確insert測試command；不要為了fixture改正式spawn或假定optional component必然存在。
- 當前確認：core兩項、正式60Hz一項通過。局部實作與限制見 `2026-10-06-bot-item-order-priority-progress.md`；無UE／全套／100場驗收。

## E271：Carry 尾刀與正常普攻必須共用已準備增傷公式（2026-10-06）

- 問題：投射物會在 outgoing modifier 後加上 next-attack bonus，Carry 仍只讀 final_atk，造成自己的尾刀能力被低估。
- 決定：共用唯讀 UnitStats::normal_attack_physical，正常 launch 與 Carry 使用同一公式。只讀本人 BuffStore，不消耗或預先寫入效果，不把目前可擊殺當成未來命中保證；維持精確命中率 gate 與敵方 current disclosure／incoming observation。
- 首次局部測試失敗：fixture 使用不存在的 total_damage_outgoing_percentage，得到90而非105。正式鍵名為 totaldamageoutgoing_percentage；改由 StatKey::as_str 產生鍵名，最後通過。不改 production 數值規則迎合測試。
- 查詢防錯：再次把 Windows 的 systems/*.rs literal path 交給 rg，得到路徑錯誤；應對實際目錄搜尋或先列檔案，不再猜目錄與 glob。
- 最後當前確認：共用公式、正式60Hz Carry及既有真實launch消耗／visual-zero三項指定測試通過，完整5.5與最後驗收仍待。詳見 `2026-10-06-carry-armed-damage-progress.md`。

## E270：driver 模式在啟動前集中解析，內部入口也必須檢查 IPC 身分（2026-10-06）

- 問題：C ABI 已拒絕 IPC 的零玩家／隊伍，但直接呼叫 driver 缺少同一限制；分支與初始速率分別讀取布林設定，容易在後續擴充時分歧。
- 決定：啟動前唯一解析 `DriverMode::{Presentation, LocalTd, NetworkTd}`，檢查端點、身分及 TD 的 story／DLL；IPC 不需要本機 campaign 或 DLL，不得退回 gameplay。SinglePlayer 是啟動拓樸，不與 IPC 互斥。
- 防錯：不得用 story 名稱猜正式 MOBA 模式。此次查詢再次猜錯 import_campaign 與 OmRuntimeSubsystem 路徑；應先 `rg --files`，實際 campaign 位於 runtime/native/scene。查詢失敗不是程式建置失敗。
- 驗證範圍：本輪 driver 模式矩陣與既有 IPC 收送／停止測試；不宣稱完整單機／LAN 或發布 binary 已驗收。詳見 `2026-10-06-driver-mode-boundary-progress.md`。

## E269：Bot 不可依賴私人物品 Buff 名稱前綴（2026-10-06）

- 問題：planner上輪直接starts_with辨識衝刺／減傷，與writer私有命名耦合。淨stat亦可能被其他效果抵消，不能代替效果存在狀態。
- 決定：ItemTimedModifier閉集與BuffStore共用grant／has；對應family／stat／合法方向及量統一定義，writer／Bot同源。查詢使用原聚合numeric reader，不建第二份快取或計時器。
- 邊界：仍是本人host-local狀態，不公開私人payload；strongest-family、刷新與tick移除語意保持，pending-zero不提早假清空。沒有新編譯失敗。
- 當前core矩陣與正式60Hz五商品通過；局部結果與限制見typed-item-modifier-progress，沒有UE／整場或完整驗收。

## E268：原生 H 的佇列語意不能依賴實體 Shift 取樣（2026-10-06）

- 發現：H已有初版，普通／Shift chord共用回呼後讀 IsInputKeyDown；精確delegate dispatch與實際鍵盤狀態可能不同，不應重複建造已存在功能。
- 決定：兩個綁定明確false／true，共用owner-only Hold建構器與提交入口；HUD／選角、runtime啟動與Connected gate，非法owner清零，直接configured ID，不走UI helper的歷史fallback。
- 驗證界線：限定UE10actions編譯成功，新binding／builder assertions只已編譯，不宣稱Editor／PIE／OS按鍵已通過。Rust局部結果見native-hold-chord-progress。
- 預防：未知路徑先列檔；既有plugin warning不當新失敗，不修改engine或放寬NoEngineChanges。

## E267：Bot 主動物品必須使用正式輸入，測試不得猜 API（2026-10-06）

- 決定：BotItemBuild 可選 active_use、owner 資源與目前披露威脅，五種 compiled 效果共用唯讀規劃→正常 NoTarget ItemUse；防禦／回魔門檻與半徑由 Lua 開局配方配置。無 hero／商品 ID 專用執行分支。
- 注意：ItemUse 清除既有命令，普攻前搖不可被準備增傷取消；既有盾／減傷／衝刺／待消耗增傷與冷卻跳過，定身不花衝刺。正式 dispatcher 才修改效果／冷卻。
- 錯誤：moba_mana_capacity 誤傳兩參數 E0061、Inventory.is_empty 不存在 E0599；改實際簽章及 items().count()。Sprint 斷言錯讀 MoveSpeedBonus，改真正 MoveSpeedBonusBuff。購買 fixture 先在商店買再移動，沒有放寬 production 距離。
- 工具：Windows 字面 *bot* os error123、猜 comp/item.rs／property.rs／scripts/lib 失敗；先 rg --files 再讀實際來源。大段輸出截斷不當完整審閱。
- 當前 core 1項與正式60Hz 1項／5種商品通過；Lua確認與完整限制見 `2026-10-06-bot-active-item-progress.md`，不冒充整場驗收。

## E265：正式主動商品不能再用fixture代替，回魔測試要區分自然回復（2026-10-06）

- 缺口：前幾輪有共用執行器／metadata／六鍵入口，但正式商品只四個被動。Lua追加5–9五件訓練主動商品，保留原ID／配方與價格，生成Rust與C++；正式60Hz測試使用match setup生成registry，不覆寫效果或直接塞item instance。
- 實際test失敗：回魔池raw112810不等於112640，期待110漏算正式tick自然回復；修正為精確物品ManaGained100、當前池不低於物品回復且不超容量，不修改production regen來迎合期待。這是新測試期待問題，無Rust編譯失敗。
- 修正後1測試／五商品情境通過：扣款、入格、使用、五種實際效果與owner隔離、CD拒絕不改CD。九商品C++生成／17檔check、UE相關模組4 actions成功；未跑PIE或真實快捷鍵。
- 多段工具輸出截斷不當完整審閱；沒有猜新檔案路徑或終止其他建置。保留既有td_rounds／plugin警告。
- 新data=2df5b6b1e02d95d7、presentation=3634f807282b0515，identity不變。ABI14／wire5／IPC4不變，不舞弊改hash或接受舊stage；最後須統一重建部署。護盾安全投影與全對局驗收待辦，21/31保持，見active-shop-content-progress。

## E264：owned 物品輸入失敗不能殘留 player one，斷線無 frame 也要清 cache（2026-10-06）

- 缺口：只有通用 ItemUse API，沒有原生六格快捷鍵與 owner-local 準入。新增1–6唯一binding、反射 SubmitOwnedItemUse、完整六格 baseline 與 owner／metadata／CD／ready核對，五種已支援自身效果用 NoTarget，不綁物品／英雄專屬程式，不要求商店距離。
- 原事件 PlayerId 預設1，初版 BuildItemUseInput 以預設 constructor 重設仍殘留1；檢查來源發現後明確設0再准入，合法時才填 configured owner。未執行 C++斷言，不記成 test failure；未改全專案／舊TD預設。
- 斷線且 AcquireLatestFrame 失敗會跳過 Tick 尾清除；故在 acquire 前也清 cache，禁止重連前舊ready存活。Start／Stop與完整缺owner清空，control-only仍保留。
- 外部 Build.bat 鎖等待，不猜PID或終止未知建置，自然解除後14 actions成功；PlayerId修正後3 actions成功。既有plugin依賴警告未變，無本輪編譯／test執行失敗。多段調查輸出截斷不當完整審閱。
- 新 NativeItemInput／六鍵binding斷言只編譯，未跑Editor automation／PIE／真實按鍵；正式仍四被動商品，不假造主動可玩。盾量安全投影／新商品與最後部署全驗收待辦，hash／協定不變，21/31保持。

## E263：跨生成器共用物品模型，unity 匿名名稱會跨 cpp 撞名（2026-10-06）

- 缺口：UE 安全快照有 owner 冷卻卻沒有靜態主動效果 metadata。作者型別／完整驗證移共用 content-model，兩生成器使用同模型；生成 C++ lookup、原生 tooltip／六格 cooldown-total／主被動狀態，不改 ABI，不讀 private shield payload。護盾安全投影與新正式商品／原生使用操作仍待辦。
- Rust 編譯失敗：新 parse 未註記 Vec，slice validator 令推導成 unsized [MobaItemEntry]，7個 E0277／E0308。明確 Vec<MobaItemEntry>，不用轉換空資料掩蓋錯誤；作者 enum test-only import 改 cfg(test)，沒有新增 unused import 警告。
- UE 編譯失敗：兩個既有 cue style cpp 的匿名 CompiledStyles 在 unity 合併 C2371。修正內部名稱為 CompiledAbilityStyles／CompiledProjectileStyles；不依 adaptive exclusion 或關閉 unity 掩蓋名稱問題。修正後相關模組7 actions成功；未強制全 unity／release。
- 工具錯誤：猜 omfue/bridge/include/om_bridge.h 不存在，應用 rg --files 查實際 ThirdParty include；多段輸出截斷須縮小，不當完整閱讀。
- 生成器2/2、原作者2/2、17檔 check、相關 UE modules 編譯通過；C++ 新斷言只編譯未執行。presentation hash=e8bdc0929625fdd2，identity／data 保持，未 stage 舊 bridge 或聲稱 PIE 可玩；ABI14／wire5／IPC4與21/31保持。

## E262：主動效果必須從作者資料生成，外部套件 feature 要用自身 manifest（2026-10-05）

- 缺口：generated_moba 硬寫 active=None／cooldown=0，核心五效果雖實作但 Lua 作者無法生成。新增可選閉集 active／cooldown、嚴格生成前範圍與未知欄位驗證，生成 Fixed64 variant 並轉接 native 核心；被動商品省略預設欄位保持序列化，不新增正式商品或聲稱 UE metadata 已完成。
- 指令錯誤：cargo test --manifest-path omb/Cargo.toml -p omoba-template-ids --features runtime-lua-content 不可替 workspace 外部套件指定 feature；改自身 omoba-template-ids/Cargo.toml。新測試漏匯入 canonical_template_hash 導致2個 E0425，局部 import 修正，禁止改 production hash 迎合測試。
- 作者解析器2/2、正式 compiled-content-only 後端入口3/3通過；opt-in runtime-lua-content 僅作者單元測試。五種生成 enum 轉接經實際效果與冷卻驗證，不把測試 feature 帶回正式設定。Unreal C++ active metadata／HUD、實際新商品與最後全驗收待辦，21/31保持。

## E261：舊入口不可重複實作效果或把名字當授權（2026-10-05）

- 缺口：resource_management::use_item 仍有 Shield 回血、Sprint 永久改基礎移速、三種 log-only，未知名稱沿 find_hero_entity 退回第一英雄。已移除效果副本，僅嚴格解析六格槽位與唯一名稱／非零唯一 owner／存活，再委派共用執行器；失敗不回 completed。
- 正式 MOBA 本來在公開 handle_player_request 拒絕舊訊息，並非已證實的正式繞路；本輪在私有 use_item 再守一次，三個局部測試確認兩入口拒絕且不修改有效英雄。名稱不是 authenticated 身分，不放寬正式邊界。
- 調查錯誤：再次猜 omb/src/item.rs（實際 item/mod.rs）及 omoba-core/src/runtime/moba.rs 路徑；讀取前應先 rg --files 查目錄，不根据模組名猜 flat file。截斷輸出不當完整審閱。無 Rust 編譯或 test 失敗。
- 五種效果／冷卻、名稱與槽位拒絕、正式模式雙入口 3/3 通過；既有 protoc fallback／td_rounds 警告未修改。其他舊技能／買賣 fallback 不在本輪範圍，generated active 作者契約與 UE／完整驗收仍待後續，21/31 保持。

## E260：護盾不能回血，吸收傷害與HP attribution要分開（2026-10-05）

- 缺口：Shield的舊瞬回HP假實作與零秒fixture。本輪改真正標準Buff餘量／duration，明確拒絕非法amount與零／非有限duration。一般非TD-layer packet先modifier後shield，direct也shield；耗盡移除／到期標準清除，最強餘量刷新不累加。
- 有效hit即使被全吸收仍中斷Recall；HP傷害與擊殺紀錄僅真正扣血時適用，不把全吸收當失血。TD-layer branch不擴充shield，避免改既有pop immunity／atomic plan。
- 調查錯誤：rg再把docs/plans/2026-10-05-item*當Windows實體路徑導致os error123；已多次記錄，glob必須放-g，對存在目錄搜尋。這不是production故障；沒有Rust編譯或test失敗。
- 發現另一尚未收斂入口：omb/src/state/resource_management.rs::use_item仍有永久Sprint／log-only／Shield回血MVP；正式生成仍被動，本輪不宣稱該舊訊息入口已修。必須後續核對正式模式拒絕邊界並用共用executor取代重複邏輯，不能遺忘。
- 60Hz護盾／modifier順序／direct／expiry／全吸收Recall中斷1、mixed買用賣遷移1與拒絕11情境1通過。无Lua／hash／ABI／UE變更，不部署／全驗收，21/31保持。

## E259：一次性增傷須在真實攻擊建立消耗，核心test需要kcp（2026-10-05）

- 缺口：HeadshotNext原log-only，E256先拒絕。本輪BuffStore通用arm／peek／consume，pending私有key與有限最強值，不改base atk；正常projectile完成launch資料預檢才consume，命中率之前加傷害、miss仍consume且零damage，visual copies零傷害。未成功launch／windup／input／script技能projectile不消耗。
- 調查錯誤：再次猜comp/attack.rs不存在；不應在已知TAttack構造可從同檔fixture讀到時猜檔名。廣泛調查輸出截斷，不當完整審閱。
- 實際core test建置失敗：--no-default-features只開compiled-content-only，漏kcp，既有SimulationTickProfile／PendingPlayerInputs等14錯誤。修正為--features kcp,compiled-content-only，沒有修改production types或玩法繞過設定。套件test編譯所有cfg(test)，name filter不能補缺feature。
- 最終正式60Hz買用1、core實際normal projectile launch1與相鄰拒絕1通過。標準remove_all清除測通，完整死亡重生／前搖取消／UE按鍵未另測；無新Lua／hash／協定或部署，21/31保持。真正護盾與generated active authoring仍未完成。

## E258：減傷須進共用結算，重複道具不能無限制疊加（2026-10-05）

- 缺口：DamageReduce原只log，E256／E257先拒絕，本輪完成標準限時Buff，負DamageTakenBonus由既有incoming_damage_packet結算，不另造英雄護甲／HP或傷害分支。所有packet包含pure皆依既有共用契約減免，不宣稱僅物理／魔法。
- 有限比例1/1024–1、時間1/1024–60與原cooldown／stats／BuffStore預檢；raw ratio round後負值，item_damage_reduce:<item-id>刷新，跨item以最強family聚合，避免兩道具比例直接相加。強到期後弱仍有效，死亡沿既有BuffStore清除。
- 本輪沒有新的工具／Rust編譯／測試錯誤；保留已存在td_rounds警告。新60Hz買用／mixed packet／refresh／分段到期1與非法11情境1、相鄰拒絕1通過。零失敗不等同完整MOBA驗收。
- 正式generated物品仍被動，本輪未新增active作者契約／UE快捷鍵／Buff視覺映射或測死亡清除。HeadshotNext／真正護盾待後續；不變hash／協定、不部署／全驗收，21/31保持。歷史未支援狀態不抹除，由此紀錄取代。

## E257：回魔使用既有容量，腳本階段前後不能讀錯事件佇列（2026-10-05）

- 實作缺口：RestoreMana原只log，E256先拒絕，本輪改managed ManaPool真實restore；同源checked fixed轉換與MOBA容量解析、copy預檢後提交，實際gain正值才入標準ScriptEvent。滿魔可用與耗CD保持一般物品規則，但不造零gain；缺pool不隱式初始化。DamageReduce／HeadshotNext仍未完成。
- 實際test失敗：60Hz item事件在本tick腳本dispatch前產生，ScriptEventQueue已被正式消費；測試step後drain原queue得到空。改讀同dispatch產出的ScriptVisualEventQueue檢查ManaGained／owner／exact amount，不把production延後派發來迎合fixture。直接API滿魔仍查尚未dispatch的原queue。
- 調查錯誤：猜ability_runtime.rs不存在，inventory確認ability_runtime/mod.rs；不要再根據模組名稱猜flat檔案。rg某檔無match的exit1不是檔案缺失；截斷輸出須縮小，不當完整審閱。
- 首次沒有Rust編譯錯誤，拒絕8情境1已通過，事件fixture修正後新2／相鄰2全通過；既有td_rounds警告保持。本輪無Lua／hash／協定／UE變更，不部署／全驗收，21/31保持。E256回魔未支援記錄是當時事實，由本輪取代，不抹除歷史。

## E256：限時物品不能永久改基礎數值，log-only不能回報成功（2026-10-05）

- 缺口：legacy SprintBuff直接永久增加msd；RestoreMana／DamageReduce／HeadshotNext只log卻耗冷卻／清命令。衝刺改標準BuffStore限時來源、raw fixed stat與最強family；三個未完成效果明確拒絕。Shield舊契約明示瞬回HP，本輪保留，不能稱真正護盾或完整active物品完成。
- 完整衝刺預檢：bonus有限1/1024–10000、duration有限1/1024–60秒、cooldown有限0–3600秒，stats／BuffStore可用，再提交Buff與冷卻。原生成catalog仍active=None，不擅自新增Lua資料／schema、runtime Lua或角色UI。
- 工具調查錯誤：猜buff.rs、core／omb的single_lane_match_tests.rs與hero_command.rs失敗。inventory確認實際ability_runtime/buff_store.rs與scripts/base_content/src/single_lane_match_tests.rs；未知路徑先rg --files，不猜。廣泛Saika與多檔输出截斷，不當完整審閱。
- 實際編譯失敗兩次：HoldPosition未匯入，後續猜omoba_core::proto再次失敗；既有fixture明確使用game_proto，已改為該完整名稱。修復前先讀現有同型別用法，不猜module。
- 拒絕test首次unwrap失敗：NaN cooldown在shop購買准入就已拒絕，不能期待有slot。改合法購買後只替換測試registry，再測use獨立preflight；正式shop驗證不放寬。
- 最終新60Hz衝刺1／拒絕7情境1、舊混合買用賣1通過。死亡清除復用現有BuffStore lifecycle，本輪未另跑死亡／跨item family案例；無UE改動／部署／全驗收，21/31維持。

## E255：輸入 API 存在不等於玩家已有原生操作入口（2026-10-05）

- 缺口：WorldBridge已有AttackMove與queued API，controller卻沒有A按鍵。補通用A直接游標攻擊移動／Shift+A佇列，沿既有權威管線；不增加模態左鍵狀態、英雄分支或第二份玩法模擬。
- 准入：HUD／選角guard，configured owner取代任意玩家ID；游標必須有限且在有效viewport內，地面投影須有限。直接呼叫owned API也執行guard；實際生命／階段／控制／命令是否合法仍由Rust決定。queued表示命令語意，不表示權威已接受。
- 工具問題：本輪多檔合併讀取與全dirty diff統計再次截斷。後續改小區段，截斷不作完整審閱；原有大量修改不歸成本輪。没有C++／UHT編譯錯誤。
- 限定OmRuntime＋OmGenerated＋OmEditor13 actions建置成功，新增按鍵／拒絕斷言只编譯未执行。不得把guard返回false斷言或編譯當成有效游標提交／真實鍵盤成功。維持NoEngineChanges，不修改BuildId／stage／全驗收；完整6.2與21/31保持。

## E254：每份快照不能覆寫自由鏡頭，缺 owner 不可借另一英雄（2026-10-05）

- 缺口：FocusCameraOnOwnedHero每snapshot重新定位，邊緣捲動無法保留；自己缺席會借第一個可見英雄。改首次fit、預設自由，原生Space暫時追蹤／Y鎖定，共用camera owner state，不新增角色graph或gameplay命令。
- 目標僅configured owner＋存活Hero安全frame座標，缺席／非法位置清target不跳原點，精確key退休與Stop／ReleaseAll清除，鎖定偏好供新披露恢復。追蹤時不與edge scroll競爭，不猜hidden actor或另開World。
- 工具錯誤：猜不存在moba_interactive_selection.lua／OmGenerated插件根目錄，並把OmPlayerController*當Windows實體rg路徑；inventory找到moba_shared_selection.lua及OmRuntime插件下OmGenerated module。E253以前已有此規則仍重犯；未知路徑先rg --files，glob只放-g。長輸出截斷縮小，不當完整審閱。
- 限定模組10＋最終9actions建置成功，無C++／UHT失败；新增camera斷言只編譯、未執行Editor／PIE或實際按鍵。維持NoEngineChanges，不重試E224／改BuildId、不stage／全驗收，hash／協定與21/31保持。詳見owned-hero-camera-progress。

## E253：受阻續航不能排擠恢復技能，新生英雄不保證已有命令（2026-10-05）

- 缺口：定身時Bot仍選新旅行；續航Recall／Retreat受阻後直接continue使合法自療不被考慮。共用owner rooted限制新Retreat／Advance／非原地Escort，受阻續航只允許恢復policy（學習原規則保留），無可用恢復則不轉進攻／旅行，不清既有命令、不造跨tick等待計時。
- Root不是stun：正常沒有續航等待時仍保留普攻／一般施法，權威移動控制與技能資源准入不變；解除依當前committed disclosure恢復，不讀敵方私人Buff或位置。
- 首輪旅行test unwrap HeroCommandQueue失敗，新生hero未必有queue。改經正式HoldPosition建立既有命令再測保留，不直接注入component。續航首輪成功、最後新2及相鄰3成功，沒有編譯失敗；舊td_rounds警告保留。
- 長讀取截斷改小區段，不作完整審閱證據。不改Lua／hash／協定／UE，不部署／全驗收，21/31與完整5.5保持。詳見rooted-bot-recovery-progress。

## E252：cue 退役不等於忘記身分，窗口下界不可測錯（2026-10-05）

- 缺口：CueRetention ACK／Hide移除pending後，相同合法ID重收可再次准入。共用有界admitted歷史覆蓋所有typed cue，最大1024、最小ID淘汰推進拒絕floor；pending保留獨立，既有成功送出／同connection ACK契約不變。
- 重連baseline保存最高tick，晚到且不晚於基準的未見ID也拒絕；view epoch／ResetView開新domain。4096tick原窗口保留，sorted過期前綴避免新增每step全歷史掃描；過舊cue可丟棄，不宣稱永久exactly-once。
- 實際首輪capacity測試失敗：tick21新cue仍在inclusive下界21，fixture誤期待空。改high_tick為21＋WINDOW_TICKS＋1，不改production窗口。最終retention6/6與真實loopback TCP1/1（六種payload）成功，沒有編譯失敗，舊td_rounds警告保留。
- 操作長輸出截斷須分段讀；git diff對未追蹤檔無輸出不是修改不存在，核對實際檔與status。不改Lua／hash／協定／UE，不部署／全驗收，21/31與完整4.3保持。詳見retired-cue-admission-progress。

## E251：定身必須覆蓋通用主動位移，不能封鎖所有位置提交（2026-10-05）

- 缺口：普通移動遵守定身，但碰撞位移adapter未查控制，dash_to_point可能繞過。共用advance_with_collision在標準rooted（含stun）回傳目前overlay／cached位置，由既有完整效果預檢拒絕，無扣費／冷卻／成功施法事實。
- 決定：直接set_pos維持明確權威／強制定位用途，不能盲目丟棄所有ScriptSetPos。Bot只跳ApproachEnemyPoint，保留一般傷害與治療，僅讀owner控制；不造英雄分支或讓AI取代權威准入。
- 工具錯誤：從scripts workspace對非成員omoba-core指定features被Cargo拒絕（cannot specify features for packages outside of workspace）；改用core自己manifest後成功。path dependency不等於workspace member，不修改workspace解決錯誤指令。長dirty diff／合併讀取超限改符號小區段，不聲稱完整已讀。
- 正式60Hz新1／adapter新1／舊牆面射程位移1均成功。沒有實際編譯／斷言失敗，舊td_rounds警告保留；不改內容hash／協定／UE，不部署或全驗收，21/31保持。詳見rooted-voluntary-relocation-progress。

## E250：回城必須在 Outcome 後檢查控制，不能只靠傷害取消（2026-10-05）

- 缺口：原回城起始／完成檢查未納入BuffStore標準控制；純控制沒有正傷害或位移，可能最後一幀仍傳送。
- 決定：MobaMatch共用recall_control_blocked（rooted或silenced，均包含stun），begin禁止起始、post-outcome在deadline前取消；不偽造傷害、不把root改成禁技能／silence改成禁普攻。解除不自動補開channel，需新普通Recall。
- Bot低血／低魔與經濟回城同規則，避免控制期間提交無效Recall；權威仍獨立檢查，AI閘門不是安全准入。既有ACK為接受輸入不是讀條完成，不修改transport結果契約。
- 工具錯誤：猜不存在native/outcome.rs得到os error2；inventory確認native/comp/outcome.rs，後續只用已知路徑。大段合併输出截斷後縮短，不把截斷當完整已讀。
- 正式60Hz新2項（三種控制各自子案例）與相鄰回城／經濟回城2項均首輪成功，最後純AddBuff取消時HP／位置未變且不傳送，到期新讀條完成。沒有編譯／斷言失敗，舊td_rounds警告保留；不部署／UE／全驗收，21/31保持。詳見recall-control-interruption-progress。

## E249：控制效果必須閉集型別化，新 Buff 不可默認要求 Blueprint（2026-10-05）

- 決定：新增control_enemy的ControlEffectKind閉集合stun／root／silence，舊stun_enemy降低至同一resolved計畫／sink。共用完整每級時間、enemy unit／射程與敵我衝突驗證；先全plan preflight再提交，禁止任意Buff字串。
- 標準BuffId保留真正控制語意：root禁移動、silence禁施法，不禁普攻時鐘；刷新沿既有max TTL。先鋒重擊／遊俠終箭僅Lua接線，不新造角色handler或C++／BP graph。
- 首次生成發現root／silence未宣告native_only，manifest帶預設BP_Root／BP_Silence路徑；這會增加不必要資產相依。修正Lua ue.native_only=true，最終manifest兩處registry／records的Blueprint路徑均空，補生成與限定模組增量編譯。不要只聲稱「沒有手寫BP」卻留下新BP必要路徑。
- 操作問題：合併大量skill／status／code輸出超限；改縮小選取，不能將截斷當完整已讀。無實際編譯／測試失败，td_rounds既有警告保留。
- Lua新50／舊15／include8通過，Rust直接相關7項通過，native12actions＋最終5actions成功；生成check15檔成功不是畫面／完整對局驗收。data26f34a124129cc46、presentation ca42ac58715f2f69需共同部署，21/31／完整5.5保持。詳見declarative-control-effects-progress。

## E248：Bot 控制狀態須阻止無效決策，不能把沉默當全面停止（2026-10-05）

- 缺口：role_bot_inputs 原先未檢查自己標準控制狀態，暈眩時仍決策、沉默時仍優先選必定拒絕的技能，可能排擠正常普攻。
- 決定：僅讀 authorized owner BuffStore；暈眩本次零輸入且不清原命令，沉默只跳施法、保留普攻／技能學習。解除後讀當前 committed perception，不造等待游標／英雄分支，也不弱化權威 handler 驗證。
- 調查錯誤：多檔 context 合併輸出超限，改分段讀；猜不存在 comp/hero_command.rs 得 os error2，改在存在 runtime 目錄搜尋符號，正確檔為 comp/phys.rs。E247 已要求 inventory，仍重犯；未知符號路徑一律先在存在目錄 rg，不再猜檔名。
- 正式60Hz新控制案例與既有Bot傷害／治療案例各1/1首輪成功，沒有編譯／測試失敗。五位置等待、實際TTL、沉默普攻與解除後80傷害／CD均有確認；不代表100場／UE／LAN驗收。內容與協定不變、21/31保持，詳見role-bot-control-state-progress。

## E247：宣告式暈眩須使用標準控制狀態，預檢不可部分提交（2026-10-05）

- 決定：Lua stun_enemy 經共用模型與 registry 生成編譯 Rust EffectOp；每級時間 1/1024–60 秒、完整 ranks、敵方 unit 及有界射程必須有效。整份 effect plan 與資源准入成功才提交傷害／暈眩，不以 stat modifier 偽造控制。
- 沿標準 BuffId::Stun：既有移動／普攻／施法阻擋與公開 Buff 視覺同源，刷新取較長剩餘時間。不是獨立 caster source／驅散／韌性或免疫的新契約；先鋒僅改 Lua 資料，不新增英雄 C++／Blueprint。
- 工具錯誤：rg 路徑含 buff* 得 os error123；猜 om_content_manifest.json 得 os error2，清單確認實際為 om_codegen_manifest.json。即使 E246 已記錄，仍再次發生；後續必須先 inventory，再用存在目錄與 -g filter。大量合併讀取被截斷，不能據此聲稱完整已讀。
- 測試在編譯前修正 BuffVisualState import；沒有實際編譯失敗。正式 60Hz 長控制案例追加命中傷害及冷卻斷言，防止沒有施法卻誤當刷新成功。新局部與相鄰確認皆通過，既有 td_rounds dead_code 警告保留。
- 完整資料 hash 更新為49f5b104fdd4c75c，呈現 hash b348872bbe367985 不變；部署仍須同步內容，不能只看呈現 hash。生成／check成功不代表 Unreal 畫面或完整對局已驗收，21/31與完整5.5保持。詳見 declarative-stun-progress。

## E246：LAN launcher 必須分離遠端 server 與本機 owned 程序（2026-10-05）

- 缺口：正式配方 launcher 固定 SERVER_IP／runtime --server 為127.0.0.1，且每次都啟動server及所有真人；第二台不能沿同一入口加入，也不能把遠端server當成自己的PID清理。
- 決定：--server-bind 明確unicast IPv4指定host監聽；--connect 明確host IPv4切換remote-client，--local-player可重複但必須是配方中真人。remote要求host最終JSON，不允許hero override／互動重新選角／非預設server bind；生成plan完全沒有server規格，只啟動本機選定席位的runtime＋Unreal。原localhost與全真人預設不變。
- 安全邊界：IPC永遠loopback，精確player/team ready仍是必要條件；遠端server健康由既有runtime連線／准入處理，不查或停止遠端PID。JSON／席位選擇不是帳號認證，沿既有authority admission與native內容協商，沒有TLS／防竊聽的新承諾；不改防火牆、不自動對外啟動。
- 首輪錯誤：測試以SERVER_IP="..."字串核對，正式TOML工具保留SERVER_IP = "..."，第52行失敗；讀回已產生的設定確認值正確後，改容許空白並精確核對值，不改production格式或重寫來源。
- 工具失誤：兩次rg把docs/plans/2026-10-05*與omb/src/config*當Windows實體路徑，得到os error123。後續對存在目錄使用-g檔名filter或rg --files inventory，不把glob放在路徑參數；大段dirty diff截斷不當作完整審閱證據。
- 當前新7/7（正式Rust設定預檢＋mock程序生命周期）、相鄰本機7/7與help入口成功；沒有新的Rust／C++內容或協定版本改動。未執行雙機LAN／Unreal畫面／完整對局，不stage、commit或維護omfx。詳見network-role-launch-progress；完整6.4與21/31保持。

## E245：多人 launcher 的程序紀錄不可覆寫，退出必須有權威終局收據（2026-10-05）

- 首輪問題：第二個程序 spawn 後寫同一 owned-processes.json，觸發 path.write 預設禁止覆寫，成功案例中斷。改每個 spawn 保存獨立 owned-process-N.json；先加入記憶體 ownership 再存檔，保存失敗也會清理剛啟動的程序，不放寬工具的覆寫保護。
- 測試 fixture 的 delayed-publication／publication-timeout 可能重寫同一收據；改為只發布一次，後續僅維持程序退役狀態，不取消 production 的不可覆寫契約。負向案例逐一核對預期錯誤，防止無關失敗冒充成功。
- 決定：2..10 真人共用唯一 Rust host、每真人獨立 Unreal／invitation／log／UserDir。Lua 不讀 invitation token；shared_room handshake 與每席位 finalized reply 為必要條件。追隨者 stale error 可帶合法終局快照，但不生成自己的假配方；唯一開局配方來自 host。
- 取消與發布區分：無收據退出立即取消；有合法終局收據才給 host 5秒原子發布期限，不以 sleep 或 EOF 推測同意。讀取期間 host 可能已發布並退出，因此檢查最終 artifact 或活程序，不誤判正常退休。
- 完整配方僅允許真人英雄變動；Bot、隊伍、角色、玩法規則及 locked roster 全部一致才交接。任一失敗反向逐一清理本輪 owned PID／expected exe，每項獨立捕捉，清理錯誤彙整 errors.md，不因單项失敗停止其他清理。
- 局部13/13模擬程序案例成功（含發布延遲／逾時、取消、host退出、舊握手、hash／配方／收據不符、缺收據與cleanup注入），原單真人6/6成功；native收據3 actions／5.31秒編譯成功。未執行Editor／雙UE／LAN／完整對局；E224仍在，21/31保持。詳見 shared-selection-launcher-progress。

## E244：共用選角 TCP 必須固定准入、明確 socket 模式與原子發布（2026-10-05）

- 實作：moba-config --selection-host 建立唯一 SelectionRoom；每真人 32 bytes 隨機 token 與獨立 invitation，先驗 schema／scope／full catalog hash／player-token 配對才 bind。每席位只允許一條 active connection；總 worker 上限10、handshake16 KiB／5秒、寫入10秒。預設127.0.0.1動態port，具體LAN位址須明確指定；不改防火牆，不將明文 bearer TCP 宣稱為 TLS 或防竊聽認證。
- Gateway：原 --selection-session 對普通 plan 維持舊服務，對 invitation 則連線既有 host，固定身分與hash驗證後把回覆送至原生 pipe；token只在檔案與handshake，不進程序參數／log。stdin EOF 關閉自有socket，不鎖定或完成房間。
- 首輪問題：真實雙gateway測試兩次在 duplicate-seat assertion 失敗；既有有效連線提早結束，測試尚未收到新的輸入。listener採nonblocking，而worker未明確設定blocking。顯式 socket.set_nonblocking(false)（worker／gateway）後，原案例首輪成功，正確拒絕duplicate且保留有效席位。不要只把這個失敗當 flaky／加sleep掩蓋。
- 追加發現：ready／邀請／最終配方不能 create_new 後直接寫，會讓等待方看到半份檔案；改同目錄唯一pending檔完整寫入／sync_all後hard_link發布，目的地已存在或不支援hard link即失敗，不覆寫、不fallback。成功移除精確自有pending名稱；失敗保留診斷檔，不刪廣泛路徑。測試移除讀JSON重試以免遮蔽production race。
- 原生：gateway的shared_room=true控制0.5秒read，僅revision變更重建列表；local模式不poll、不改原request編號。合法stale reply亦可能提供finalized快照，追隨者必須依權威完成狀態退出，不能因error文字永遠卡住；不偽造自己的結果檔，host持有canonical配方。
- 診斷：host的owned output/errors.md最多64筆，只記error kind／拒絕此連線的決定，不記token或command。故意錯token／duplicate／wrong-bound測試的PermissionDenied及gateway stderr是負向確認，不是最終測試失敗。
- 操作失誤：讀取不存在的scripts/lib；應依tools/lua/lib inventory定位。一次patch沿用rustfmt前的多行錨點而遭拒絕，確認未套用後讀短區段再修；不重試同一失效patch。既有td_rounds／protoc fallback警告保留。
- 最終新增framing／原子發布2項、真實host＋雙gateway1項、原local CLI1項，共4項局部確認成功；native首次5 actions／13.18秒，最後finalized修正3 actions／4.98秒成功。原生poll／畫面未執行（E224），不當完整UI／LAN驗收。
- 尚未將Lua多人launcher改為啟動host與多個renderer、尚未做兩台LAN／選角至結算；完整6.2／6.4及21/31保持。不stage／部署、不維護omfx。

## E243：多真人選角必須共用房間，開局配方不能依賴最後一條連線（2026-10-05）

- 缺口：現有 SelectionService 每次建構都持有獨立 HeroSelectionSession；不能把各玩家獨立程序當作同一 LAN 大廳。finalize 配方只存在於該次回覆，若多人 host 僅依最後一位玩家轉交，會缺少獨立、一次性的開局交接。
- 決定：SelectionRoom 以 Arc／Mutex 保持唯一 session；host 以已准入身分 bind 不可改身分的 SelectionService。clone 只共用 handle，不複製狀態／World；舊單真人 new 亦委派同一房間。command 不加入 player ID，不將 bind 當成認證。
- 原子性：protocol／hash／revision 檢查、session mutation 與回覆 snapshot 在同一 lock；同 revision 的並發命令只有一方能提交。lock poison 拒絕後續 bind／handle／serve／host 交接，不接受可能部分修改的狀態。
- 交接：成功 finalize 的配方保存在 room；host take_finalized_plan 只能領取一次，與 finalizing connection 存活無關。失敗／read／reconnect／EOF 不建立或補發 host 交接、不重新啟動 gameplay。
- API：handle 現回傳 Result<SelectionReply, String>，區分正常規則拒絕回覆與不可用 authority；serve 將後者轉 io error 停止，JSON-line protocol1、16 KiB framing 與 CLI參數不變。
- 調查失誤：猜測 moba-config.rs／main/moba_config.rs 路徑造成兩次 rg 找不到檔案；inventory 已確認真正路徑 omb/src/bin/moba_config.rs，後續只用 inventory 結果。多檔大輸出也再次截斷；應按檔案／段落讀取，不能把截斷當完整已讀。
- 局部新3項與相鄰舊4項首輪成功，新增 host 交接後的最終結果见 shared-selection-room-progress。poison 測試的 injected panic 為刻意故障注入，不是 production 編譯／執行錯誤；既有 td_rounds dead_code／protoc fallback 警告未擴大修復。
- 這是共用多人核心，不是已完成 LAN transport／認證或 Unreal 多人大廳；不啟動完整驗收、不維護 omfx、不部署／stage。完整6.2／6.4、21/31保持。
- 最終加入 host 交接後，選角局部11項全部通過（新3／service舊4／kernel舊4），正式 moba-config check／OpenSpec strict／whitespace 均成功，沒有編譯或斷言失敗；不重建無變更的 Unreal、不把本輪結果當完整LAN驗收。

## E242：選角回覆必須原子驗證，介面重建不可丟失已接受資料（2026-10-05）

- 問題：原生選角只驗證 JSON 欄位型別；其他玩家重複／分數或超出 u32 的 ID、非法隊伍、目錄重複英雄、座位引用不存在英雄與 ready／locked 矛盾仍可能進入畫面。非 object 的 array entry 直接 AsObject 也可能觸發診斷，而不是正常拒絕。
- 決定：共用 FOmSelectionReplyModel 在 UI mutation 前一次驗證；所有玩家 ID 必須為唯一正 u32，team 為既有協定 1／2，選擇須存在於本次 Rust catalog，真人 seat 不得為 Bot。ready 必須與所有 lock 一致，finalized 不能未 ready；plan／error 與既有協定一致。這是傳輸一致性檢查，不複製角色配裝、lane、技能或選角遊戲規則。
- 版本：decimal revision 使用 canonical ASCII u64 token，拒絕前導零／溢位／回退；既有 request correlation 與 catalog 不變檢查收斂至同一解析器，合法錯誤／read 同 revision 仍可接受。失敗不修改輸出，沒有把精確 revision 轉成 double。
- 生命週期：成功解析才保存回覆；RebuildWidget 從該快照重建列表，不再次提交命令／寫結果。StopSelection 清快照／目錄／revision／buttons／旗標，新 Start 重設等待訊息及 opt-in smoke stage，避免沿用前次工作階段。
- 調查失誤：多檔合併輸出再次截斷；後續改用短區段／小額符號搜尋。JSON tests 新增 OmEditor 的明確 Json dependency，不依賴其他 module 的間接 include。
- 原生指定模組編譯結果與限制見 selection-reply-contract-progress；Editor assertions 僅編譯，不當作已執行。不繞過 E224，不跑完整驗收，不部署／stage、不維護 omfx。
- 最終 scoped native 首輪9 actions／11.57秒成功，沒有編譯失敗；目前功能編譯與接線完成，整體21/31保持，不以小項完成代替完整6.2。

## E241：權威進度凍結必須明確傳送，不能靠前端猜暈眩（2026-10-05）

- 問題：hero_tick 暈眩時不增加 attack counter，但 AVS1 無暫停狀態；native 每個快照仍按 duration 插值，可能往前播放後又被下個相同進度拉回。
- 決定：host-local AttackAnimationTiming 改具名型別，權威在 stun／零 dt 標記 paused，正常更新清除；AVS2 固定 26 bytes 加入嚴格 0／1 暫停旗標，不包含 Buff ID、來源或目標。global snapshot pause 也可標記播放暫停，不从重複快照猜測。
- 接軌：既有 baseline／absolute event／filtered codec／IPC converter 同源；animation record flags bit 2 經原生 animation／attack-phase payload 傳遞，游標先依权威 progress 定位再凍結，解除後沿目前狀態恢复。Idle canonical paused=false。
- 升版：selective wire 5 拒絕 4，C ABI 14 與 C header 同步、拒絕 13；IPC framing 4 不變，flags 使用既有欄位。避免舊 frontend 靜默忽略暫停語意；最後部署需同步重建各元件。
- 測試失誤：新增 ABI 拒絕測試首次 E0133，validate_config 是 unsafe API；補測試呼叫的明確 unsafe 區塊，未更動 production validator。修正後通過。一次短輸出截斷警告／錯誤，後改足夠預算保存結果。
- 最終 core4／bridge3／真實60Hz source2 共9項局部測試通過；native scoped22 actions／11.77秒成功，正式codegen --check15檔通過。無 Editor／完整對局／畫面驗收、不部署／stage binary、不維護 omfx。generator_version5、生成hash b348872bbe367985及作者 catalog hash不變。

## E240：Reset 必須清除播放資產，重複快照不能重新啟動已完成片段（2026-10-05）

- 問題：Reset 只 Stop，未清 animation asset，仍可能留下前一個身分的姿勢；相同 slot／action 的每個快照無條件 SetPlaying(true)，會反覆恢復已完成的非循環片段。載入路徑亦每個快照呼叫 LoadSynchronous。
- 決定：Reset／fallback 共用 ClearNativeAnimationPlayback，清 asset／slot／action／游標／loop；循環或尚未完成且區間合法的片段才恢復播放。先用 soft pointer Get，未載入才 LoadSynchronous，既有失敗快取保留。
- 追加發現：較短素材可能使區間被裁成空區間；首次播放與重複狀態共用區間判定，非法長度／起迄或空區間標記失敗、走既有 fallback，不在第一次播放接受而第二次才停止。
- 調查失誤：第一次合併讀檔輸出截斷，FOmAnimationStatePayload 實際在 OmVisualTypes.h 而非 OmUiEventTypes.h；改以符號定位及短區段。沒有編譯／斷言失敗。
- scoped native 首輪 13 actions／9.48 秒成功，追加 clip guard 最終 3 actions／4.46 秒成功。6 個新 resume 原生斷言只已編譯、未執行，不繞過 E224、不啟動整體驗收、不 stage／部署、不維護 omfx。
- 沒有 Lua／生成內容／IPC／C ABI 變更，generator_version5／content_hash b348872bbe367985 保持；不因局部原生實作就勾選完整 4.3／6.1。

## E239：預設播放倍率不能只停在 metadata，也不能重算權威攻擊時間（2026-10-05）

- 問題：default_play_rate 已驗證並寫入 registry，但原生英雄播放忽略它；攻擊 phase cursor 又可被 state multiplier 再次改速，與權威解析 duration 不一致。
- 決定：通用 constructor 生成 NativeDefaultPlayRate；普通／legacy 動畫透過同一 helper 合併預設及當前倍率，非法非正／非有限參數各自視為 1，以 double 相乘後一次限制 0.01–10，避免 f32 中間溢位。合法權威 phase cursor 直接使用 clip span／權威 duration 的 rate，不再乘美術或 state 倍率，Impact 的 rate 0 維持暫停。
- 邊界：沒有改 gameplay、攻速、Lua runtime、IPC 或 C ABI；缺少合法 phase metadata 的舊播放仍採普通倍率。新 helper 的 native assertions 僅编譯，未視為已執行。
- 本輪沒有編譯／斷言失敗；指定 codegen 測試及正式生成／check 通過。generator_version 5，生成 content_hash b348872bbe367985；data 2027e0ada2f76866、identity ff3ef5e2957aa89f 保持，不混用 domain。
- 模組編譯結果見 native-animation-default-rate-progress；不啟動完整驗收、不部署／stage binary、不重試 E224、不維護 omfx。
- 最終 scoped native 16 actions／10.86 秒成功；原生斷言仍僅編譯，不當成已執行。

## E238：動畫 fallback 宣告必須接到播放端，缺片段不能留下舊攻擊（2026-10-05）

- 問題：原生英雄未使用已驗證的 fallback_policy；selector 找不到可用片段直接退出，可能留下前一段 attack／末幀姿勢。初始化也直接嘗試 idle，繞過共用狀態選擇。
- 決定：生成器將三種 policy 產生為原生 enum；UseGenericState 可退到通用 attack／idle，其他 policy 不以別的 action 代替缺失基本片段。無可播放片段時清除 asset、slot、action 與 loop 時間，UseReferencePose／通用耗盡保持模型參考姿勢，HideVisual 隱藏原生模型；有效片段恢復顯示，不影響 actor 視野隔離。
- 防錯：fallback 只首次套用，避免每個快照重複清 asset；失敗素材仍從既有可用集合移除且不重試，初始化與後續狀態共用播放路徑。沒有改 gameplay、Lua runtime、IPC 或 C ABI。
- 操作失誤：一次 patch 用不完整 if 條件當整行錨點，工具拒絕；確認未套用後改以 Rate／完整函式段定位。累積搜尋輸出再次截斷，後改短區段。沒有編譯／斷言失敗。
- 局部 codegen 1 項通過、正式生成與 --check 15 檔通過，scoped native 16 actions／23.87s 成功。新增原生 policy 斷言只已編譯，E224 未解除，不當成已執行；不啟動完整 Editor／对局驗收、不 stage／部署、不維護 omfx。
- generator_version 升至 4，生成 content_hash 79bf52bd0ba256e2；catalog_data_hash 2027e0ada2f76866、identity ff3ef5e2957aa89f 保持，避免混用雜湊 domain。

## E237：通用動畫生成不可隱含 Saika 狀態，metadata 型別不能靜默降級（2026-10-05）

- 問題：inferred_locomotion_variants 看到任意英雄的 sniper 片段就加入 sniper_mode_walk／sniper_walk；fallback_policy 非字串會被忽略並改用預設，數值超過 f32 範圍與多個映射欄位也缺少檢查。
- 決定：移除角色專屬推斷，特殊狀態只由 Lua 顯式宣告。共用 animation_metadata validator 同時接入 Rust Lua 建置與 Unreal codegen，檢查六種字串映射、idle list、正值且可表示的播放倍率與合法 fallback policy；錯誤帶內容 ID／欄位，生成前拒絕，不留下部分輸出。
- 邊界：保留通用 walk 預設與現有顯式 Saika 配置。驗證 fallback 宣告不等於新增其原生播放行為，不宣稱完整角色／畫面回歸已完成。
- 本輪沒有編譯／斷言失敗。移出 validator 後產生未使用 validate_string_array 警告，已移除失去用途的重複 helper，而非抑制新警告。
- 共用模型 2、codegen 2 項局部測試通過；正式來源 --check 15 檔／17 Lua inputs 通過，content_hash 6460930eee2ba926 不變。無生成檔變更，不重複編譯 Unreal／部署／啟動 Editor，也不維護 omfx。

## E236：正式攻擊動畫必須傳權威解析時間，不可由基礎 ASD 重算（2026-10-05）

- 原因：正式 IPC 不攜帶 legacy attack_phase_fx；CommittedAttack 只有 counter／sequence／phase，無法重現 Buff 修正後的攻擊時間。
- 決定：hero_tick 保存已解析的前搖／後搖，僅作 host-local presentation metadata，serde 跳過、不改 gameplay／script ABI。AVS1 公開狀態只有 sequence、phase、elapsed、duration，不包含 target、critical、Buff 來源；不能把 recovery 冒充一次性命中 cue。
- 接軌：披露 baseline 與每步 absolute event 使用同一份 typed 狀態；只更新已有公開 entity／component，非法資料拒絕。明確 idle 清除舊攻擊，Unreal bridge 直接用 Q10 時間，不猜 Hz。selective wire 升至 4，舊 3 拒絕。
- 操作失誤：搜尋猜錯 snapshot.rs 位置，實際位於 runtime/native；應先 rg --files。累積文件輸出再次截斷，後續使用短區段，不以截斷輸出當完整內容。
- 局部驗證結果與尚未涵蓋範圍記於 authoritative-attack-visual-state-progress；完整 6.1 不因本功能局部完成就勾選，不啟動重複整體驗收，也不維護 omfx。
- 60Hz 來源測試首次 E0609：latest_frames 值是 PaddedTeamFrame，不是直接 public_events；依既有 API 修為 frame.step 的事件，不修改 production schema 來迎合測試。另 buff_store 搜尋曾猜到錯誤 comp 目錄，實際在 ability_runtime，已使用檔案列舉定位。
- 最終 core 4／bridge 2／真實 60Hz source 1 共 7 項不同局部測試通過，OpenSpec strict 與兩 repo whitespace 通過；沒有新 native C++ 變更，不重複編譯／啟動 Editor，不宣稱部署或完整畫面驗收。

## E235：正式 IPC 位置更新不等於已有移動／攻擊動畫來源（2026-10-05）

- 問題：formal filtered converter把hero_render.is_moving固定false，又不提供private HeroCommand；原播放端只查這兩種來源，正式可見單位可能永遠stand。改從同ID／disclosure generation的兩個當前公開位置觀測移動，只供presentation-only動畫，不读hidden state／命令／目的地或建立gameplay模擬。
- 邊界：首次／不同世代／隱藏後再披露不推測旅程，完整缺席prune；相同tick重複保留合法觀測，倒退tick重新建基準；非法座標／死亡不保存。顯式retirement、ResetView、stop／restart清歷史，control-only不產生新觀測。embedded／TD原來源保留。
- 調查更正：CommittedAttack的13bytes只有counter／sequence／phase；hero_tick實際interval使用Buff聚合攻速倍率，所以不能把TAttack.asd.v直接當完整attack動畫時間。前兩批native相位consumer已實作，不等於formal IPC有時間來源。後續須由權威解析時間後明確披露，不猜target／critical／timeline，完整6.1保持未完成。
- 操作失誤：初查rg把attack*.rs當literal檔案造成Windows123，之後以rg --files確認tower.rs／fx_queues.rs；一次patch hunks順序逆向定位失敗、未部分寫入，拆正序後成功；累積文件輸出仍有截斷，改精確區段。
- 局部前三項首次通過；新增IPC converter測試首次編譯E0433，driver測試scope未import crate::OmLocomotionState，已補明確import，不改production API。修復後結果補於disclosed-motion-animation-progress。
- 最終4項不同局部測試、OpenSpec strict與兩repo whitespace通過；無native變更，不重複模組編譯／Editor驗收。本輪未部署，不宣稱真實對局畫面已驗收。

## E234：攻擊動畫命中點不能丟棄，重建不能從前搖開始（2026-10-05）

- 問題：生成器沒保留impact_tick，native未用phase progress，且OnAttackPhase固定PlayRate1可覆寫快照。共用clip_timing验证兩生成器，C++保存optional ImpactSeconds，兩回呼共用原生selector／cursor同步；Windup／Impact／Recovery依宣告片段定位，未知不猜，impact暫停且不觸發notify或gameplay。
- 草稿更正：一度把完整render驗證放在ContentCatalog的HeroId身分目錄，該型別沒有render；編譯前檢查後移至HeroDefinition與兩邊真正生成入口。不擴張identity型別或改identity hash來掩蓋型別誤用。
- 操作失誤：猜測codegen/templates目錄不存在，rg --files確認生成函式實在src/lib.rs；一次patch對同檔重複Update被工具整筆拒絕，未部分套用，合併各檔hunks後成功。大型累積design輸出仍截斷，後續只讀與本功能相關符號／段落，不重複掃描過量歷史。
- 首輪model1／codegen5與native22 actions全部成功，沒有編譯或斷言失敗。Native斷言只完成編譯，E224未解除不能當已執行；不重試相同啟動基線、不改BuildId、不stage或維護omfx。

## E233：動畫覆蓋不能只停在 metadata，動畫时间不能猜120Hz（2026-10-05）

- 問題：Rust／generated bridge已有 overlay選擇與名稱，但native hero播放端忽略它。用Lua native_visual.state_slots生成通用映射，驗證有界合法名稱／FName大小寫重複／目標片段；攻擊優先、移動不退站姿、Buff移除恢復普通idle，不新增角色特殊C++或重播toggle。
- 防錯：可用slot只初始化一次，錯asset／skeleton首次失敗剔除，當frame重新選擇，每次至少刪一slot确保有界；不逐entity／逐frame配置集合。已知缺獨立sniper walk時配置move，不以站姿冒充移動，後續僅美術與Lua可替換。
- 動畫原tick/120與固定16.7ms使60Hz相位錯誤；改從driver已協商rate取得LockstepTiming，未知不猜時脈。保留的FX過期也不能一直維持attack；過期與未到都保持無action。60／90／120Hz局部斷言通過。
- 操作失誤：多次把大型dirty status／累積tasks與讀檔輸出限制太低而截斷，應拆精確段並給足輸出上限。第一次測試僅輸出output丟失session／exit資訊，後重取結果造成一次多餘局部執行；長命令須保留完整session／exit metadata。未有編譯／斷言失敗。
- native13 actions成功後檢查素材語意修正Lua walk→move，最終3 actions成功。新native斷言仍只編譯（E224），不能報已執行。最终data hash2027e0ada2f76866，identity與codegen domain不變；不stage、不修omfx、不啟動全驗收。

## E232：宣告式效果的私有 key 不能直接當公開 Buff ID（2026-10-05）

- 問題：generic_slow／generic_mana key 含來源技能、stat、caster ID／generation，不能直接公開；之前只允許 catalog exact name，導致合法宣告式效果完全没有 persistent 視覺狀態。
- 決定：Lua buff.ue.buff_visual.sources 宣告來源，Rust／Unreal 生成器共用嚴格 validator。只 live ability、實際 SlowEnemy／匹配 ManaBuffSelf stat 可綁定；未知欄位、未知／tombstone ability、錯 stat 與同來源衝突拒絕。生成 Rust lookup，不在遊戲執行期求值 Lua。
- 私有 key 使用 script-abi 共用 writer／嚴格 parser：精確分段、非空名稱、canonical u32 ID／generation，拒絕負數、溢位、前導零及尾欄位。只在 host-local 解析，公開資料只有 Buff ID／時間；不從 arbitrary JSON payload 接受視覺 ID。
- 多來源合併同公開 ID：有任一無限期來源就 -1，否則最大剩餘時間；移除來源後重算，不傳來源數量，不改原 strongest-family／Mana stat／TTL gameplay。
- 操作失誤：再次猜測 Generated/om_manifest.json 不存在；rg --files 確認實際是 OmGenerated/om_codegen_manifest.json。過量搜尋輸出截斷，後改精確檔案與區段。
- 雜湊防錯：codegen 印出的 content_hash 6460930eee2ba926 是生成內容雜湊；完整 catalog_data_hash 為 29940eea6845d75b，catalog_identity_hash ff3ef5e2957aa89f 不變。不能混用不同 domain 或把新增 Buff 誤算英雄／技能身分變動。
- 當前 model1／ABI1／core7／真實 generated 60Hz 技能2 首次通過；生成／check15檔、native scoped12 actions成功。bridge 最後結果見進度檔；不 stage、不將編譯當 PIE／畫面執行，不重試 E224 已知 BuildId 錯配。


## E231：HUD Buff 必須是完整清單，不是最後一個單筆事件（2026-10-05）

- 問題：原 HandleBuffListStateChanged 每筆覆寫文字，没有 owner 過濾、空集合通知與停止清除；同隊其他英雄的最後一筆可能覆蓋自己的 Buff，效果到期也可能留下舊文字。
- 決定：新增 FOmBuffListSnapshotPayload／BuffListSnapshotChanged，完整 own-hero baseline 原子取代。以 configured player 對目前 frame 綁定唯一英雄，精確匹配 entity ID／generation；只已知 catalog、非零 key、有限正時間或 -1，最多 64 筆，忽略重複 key，不顯示內部 payload。owner 模糊／缺失或完整空清單皆清空。
- Control-only 不發 replacement；ReleaseAllActors（含停止、EndPlay、reset）清 snapshot。舊單筆 API 保留供 Blueprint 明確相容呼叫，原生控制器改綁完整事件，不能再讓舊事件覆寫新清單。
- 通用 native command bar 保存文字，內容不變不更新 Slate；文字列提供完整 tooltip，不新增角色專屬 C++／Blueprint graph／Lua runtime 或 ABI 變更。
- 防錯：讀寫新 helper 前檢查 C ABI 實際 entity_id 欄位，修掉草稿的 id 名称（尚未進入編譯）。native scoped 14 actions 首次成功；新 OwnedBuffListSnapshot 原生斷言僅已編譯，不當作已執行或全鏈畫面證據。
- 操作失誤：猜測 Plugins/OmGenerated 與 OmUiStateSubsystem 路徑不存在，後改 rg --files／精確符號。一次過量讀取設計與 tasks 多次截斷，改區段讀取；不再一次輸出兩份大型累積紀錄。E224 project BuildId 未改，沒有重試相同 Editor 啟動失敗、不手改 modules manifest。
- 當前只讀 BuildId 檢查：project 673c237e-5b5e-41ea-9643-75ad8a45bcac／engine 16d31f18-1d85-49ea-9104-55cdd3e143ad。engine 與 E224 歷史值不同，但仍失配；不能宣稱外部狀態完全不變，亦不能藉改 manifest 偽造相容。


## E230：持續 Buff 的安全來源與線路版本必須一起接齊（2026-10-05）

- 問題：正式 filtered render data 沒有 Buff 身分來源；從任意 payload 推導或重播 Added 會洩漏私有資料、重播 toggle。決定只披露同隊、有玩家 owner 的 Disclosed 英雄之已註冊 Buff ID 與 Fixed64 剩餘時間，每 tick 發布 committed 狀態，完整空集合代表移除。
- 首輪 codec 測試未知 ID 65535 觸發舊 generated lookup 的 debug_assert。共用 namespace generator 新增 try_*_id_str 回傳 Option；不 catch panic、不手寫 Buff whitelist。
- 靜態整合發現新 component 尚未加入 secure replica allowlist，以及 baseline 驗證被誤放在 structure 分支內；已補白名單、獨立驗證與 projector 斷言。
- 線路版本升至 3 後發現 framing、negotiation、projector、replica 與 server gate 散落硬編碼 2；改由 transport 的唯一常數共用，舊 V2 名稱保留作模式名稱。舊 wire 2 不得以新格式運行。
- 操作失誤：猜測 buff_store.rs 與 unreal-native-frontend spec 路徑不存在；改用 rg --files 確認。過大搜尋输出截斷後縮小範圍。兩次 patch 定位／前後順序失敗均未部分寫入；改用實際 sort_by_key 與遞增 hunk 順序。
- 防錯：持續快照只同步 active_buffs，presentation-only bridge 清除由持續資料推導的舊 Buff／Ability 事件；合法 typed caster cues 仍由後續独立管線添加。不宣稱敵方 Buff、所有 UI 或 Blueprint 恢復已完成。
- 後續整合：client config 與 Lua launcher 仍固定 2；改共用 default 並移除 launcher 的硬編碼參數。server 未強制 secure 時原 else 可能將未知版本當 legacy；新增實際入口 validate_join_wire_protocol 與舊 2 拒絕測試，不允許静默降級。
- 又一次將 -g 選項放在 rg 的 -- 後，被解讀為檔名（os123）；修正為全部選項放在 -- 前。猜測 client.rs 不存在後改檢查已知 config.rs。後續必須先枚舉檔案，窄查精確符號，不因既有紀錄而重犯。
- 移除 Lua 參數後留下三處 trailing whitespace；diff --check 發現後清除，再確認主 repo／omfue 皆通過。


## E229：持續 buff baseline 不等於重播 BuffAdded（2026-10-05）

- 問題：active_buffs 原本只發布 UI，actor 重建後缺失原已存在的 persistent effects；用 BuffAdded 補發會重播一次性特效，重複建立 component 也會覆蓋 map 而遺留舊 component。
- 決定：獨立 OnBuffState 同步既有 instance，完整 baseline 退役缺失項，control-only 不清除；不新增正式 MOBA 的 buff 身分披露。未知 catalog／隱藏或錯 generation 不能建立效果。
- 編譯錯誤：native test 直接讀 protected ActiveBuffEffects，C2248。改 automation-only 唯讀 accessor，正式欄位仍 protected。失敗日誌 buff-state-modules-20261005.log；修復編譯另存 buff-state-modules-repair-20261005.log。
- 修復後15 actions編譯成功。規格插入曾留下空白重複header，檢視後移除、strict通過；後續新增Requirement必須替換定位header，不保留空header。
- 邊界：正式 MOBA 還缺 typed visual-only buff baseline，此批不能因此宣稱整條 MOBA buff 視覺或完整 UI 完成；不把 native 斷言編譯成功說成已執行通過。

## E228：Actor 綁定不能永遠沿用首次類別，物件池不能保留舊呈現（2026-10-05）

- 問題：GetOrCreateUnitActor 原本對既存 replica 直接 return，未知→已知內容仍停留在 fallback；回收只清 buff，插值 tick／位置與原生動畫 instance 可能跨身分沿用。
- 決定：內容綁定變更才解析並交易式替換，先取得再回收，保留同 replica 去重。通用 virtual presentation reset 清除插值／nameplate／tracked effects，Hero 清動畫 slot／instance。
- 靜態檢查修正：不能只依 bHasAuthoritativeFrame 還原 visual rest，因為沒有插值也會收到 frame；新增實際捕捉 rest 的 presence flag，避免無效零偏移覆蓋美術設定。
- 操作錯誤：再次猜測 OmContentRegistry.h，rg 發生 IO error。改用 rg --files 查到 OmRegistry.h／OmVisualRegistry.h；不得因熟悉命名就猜檔案。
- 15 actions scoped native 編譯成功；新 automation 僅編譯、尚未執行。E224 啟動阻礙仍在，不重跑同條件、不手改 BuildId。持續 buff 視覺 baseline 及無通知美術 hot reload 不算本批完成。

## E227：未知內容不能冒用預設英雄或 dummy（2026-10-05）

- 問題：safe render 的未知英雄曾補 saika_magoichi、未知小兵補 practice_dummy；catalog lookup 又把所有未註冊小兵映射到 dummy，造成身分與美術錯配。unit actor 還含 Saika 專屬插值 log。
- 決定：缺少內容就保留未知，catalog_id=0 選通用 native fallback；明確內容 ID 與已披露 Hero.id 維持原路徑。診斷改通用 VeryVerbose，不刪 serialized legacy reflected API。
- 編譯錯誤：新矩陣將 DemoRenderState.kind 的 u8 直接寫入 FilteredRenderEntity.entity_kind 的 u32，出現 E0308。改用 u32::from(kind)，重跑新編譯測試 3/3 通過，不能把不同層同名 kind 視為相同型別。
- 操作錯誤：第一次廣泛搜尋 generated Saika 資料包含巨大的單行 JSON，造成截斷。改查 generator／driver／projection 精確區段；產物資料的角色名稱不能直接認定是程式邏輯分支。
- 詳見 `2026-10-05-unknown-content-fallback-progress.md`；局部 native 編譯不是 automation 執行或完整原英雄無回退驗收。

## E226：投射物消失不是命中，無來源 impact 也不能全域公開（2026-10-05）

- 問題：Projectile active=false 同時涵蓋命中與逾時；傷害事件不能代表零傷害命中。source=None 的普通 public gate 又可能讓事件對隱藏受擊者公開。
- 決定：只轉換真正 ProjectileHit；獨立 target-only fact／HIT1／IMP1，投影明確要求 visible＋Disclosed，native impact2 與 damage1 分開，共用保留／ACK／去重。不把既有 projectile fallback 的終點球當作實際命中證據。
- 邊界：受擊者退休後不補 hidden 位置，沒有世界點就不推測；C ABI 不增 layout，formal runtime 不執行 Lua。
- 規格編輯再次出現 E225 的空白重複 Requirement header，檢視後立即移除；插入新規格時要替換定位行，不保留一個空 header 再重複原 header。
- 操作錯誤：讀取過大的 rg／MD 輸出多次截斷；又猜測 OmEditor/Private/Tests 目錄而 IO error。改用已枚舉的 OmEditorAutomationTests.cpp 與精確區段。另一次多檔 patch 使用未讀取的 design 行作匹配而失敗，已確認原子性未寫入任何檔案，改用實際 Context header。Outcome::Death 沒有 enum Default，靜態檢查後改為明確 pos 欄位，錯誤版本未進入編譯。
- 確認與限制見 `2026-10-05-projectile-impact-cue-progress.md`；native 編譯不是 automation 執行，不重試未改變的 E224 BuildId 啟動條件。

## E225：範圍 cue 不能匿名、不能與 cast／relocation 混用（2026-10-05）

- 通用executor在完整preflight／資源及效果提交後呼叫既有emit_explosion，formal invocation收集有界area結果，成功後綁真正caster／team／skill／rank；合法空區域也有cue，拒絕沒有。
- formal capture取代匿名Explosion outcome，避免ordinal被當source及重播；其他hooks／legacy Story-TD不改。APC1／ARC1獨立kind，同隊且live disclosed caster才能公開，不揭露enemy ID或命中清單。
- 首輪bridge E0004：新增SkillArea漏了embedded visual enum mapping。改Option明確unsupported，不能把area映射成ScriptSkillCast；補單項測試，最後bridge5/5成功。
- 初始skill＋多段搜尋輸出截斷，後續用窄區段；輸出截斷不視為已讀完整或診斷完整。
- 首輪OpenSpec strict因新Requirement插入保留空的重複「完整對局介面」標題失敗；移除空標題、保留原本文與scenario，重新確認。規格插入應以新Requirement取代anchor或定位段落，不重複既有標題。
- 本輪executor2/2、權威來源1/1、core8/8、client7/7、bridge5/5；native12 actions編譯成功，未重試E224的BuildId啟動失敗。native斷言只編譯未跑，不宣稱像素或全鏈驗收。詳細指令／限制見ability-area-cue-progress。

## E224：位移結果與請求點不同；位置不能靠零向量判斷缺值（2026-10-05）

- 成功caster自己的ScriptSetPos overlay提供終點，不猜輸入target／enemy／impact。fact標記來源team，projector只向同隊披露位置，其他可見隊保留cast identity／rank；避免歷史位置洩漏。
- ABS3／ABY3精確長度與raw Fixed64座標，bridge明確換world units，C ABI既有point＋新flags；native新增presence與共用fallback helper，世界原點是合法位置。
- 首輪E0308：Faction.team_id i32不能直接放u32 metadata；查型別後checked conversion，拒零／負值。重新編譯成功。猜team_identity.rs／OmGeneratedTests.cpp／OmPresentationTypes.h失敗，改inventory定位，後續應先搜尋不猜路徑。
- 指定Rust來源1/1、core7/7、client7/7、bridge3/3成功；20 actions native project modules編譯成功。native automation卻exit1：engine與project BuildId不同，OmGame被略過。不能把「DLL存在」或「模組編譯成功」當可啟動。
- 補OmGame與NoUBTMakefiles都是0 actions，ID仍未對齊；停止同狀態重試，不手改BuildId／放寬NoEngineChanges。新增native斷言已編譯但未跑，沒有完整驗收或Rust artifact部署。證據詳見caster-relocation-cue-progress。

## E223：handler 的 u8 等級與公開 rank 必須同源（2026-10-05）

- 無 Mana 路徑的 `lv.max(1) as u8` 可能讓 257 執行成 1，cue 卻記錄 257。共用 checked rank resolver 在 handler 前解析一次，Mana／execute／成功 fact 共用結果，正等級超 u8／declared max 拒絕，正式／managed 未習得及缺 level data 拒絕。
- 舊無 Mana fallback 明確 handler=1／公開 rank=0 未知；不能為了呈現把未習得偽裝成已習得。新矩陣36組、指定新1/1＋相鄰3/3＋2/2成功。
- 初輪測試 E0609 是 FactOrderingKey 欄位應為 fact_kind 而非 kind；讀定義修正、重新編譯後成功。新增測試應先查型別，不依直覺命名。
- 調查又把 generated* 放 rg 位置參數造成 os123，改 directory＋-g；猜 OmHudWidget.cpp 不存在，改 rg --files 定位；多段輸出再次截斷，須分段／調整 token budget。這些是工具錯誤，不宣稱遊戲錯誤。
- AbilityScript.execute 沒有 actual effect point 回覆，不能把請求點當效果結果。位置契約尚待效果端與安全投影，不加 renderer 猜測 workaround；詳見 checked-cast-rank-progress。

## E222：技能等級要捕捉成功 invocation，不能用稍後的 HUD 推測（2026-10-05）

- 共用 dispatcher 在 handler 前捕捉實際技能 rank，成功 gate 後才發布。舊 cue 的 0 是未知，不得補 1；public ABS2／presentation ABY2 使用明確版本與精確長度，拒絕零 rank／超 int32 範圍，仍要求 live caster／epoch。
- C ABI 使用既有 level，不改 layout、視野或 ACK。舊 consumer 不支援新版本就安全略過，不把不 crash 當作部署完成。
- 本批工具曾猜 omb/client_runtime/src、錯的 OmGenerated plugin 根路徑與 core comp/hero.rs，讀取失敗後用 rg --files 定位正確 omoba-client-runtime 與 OmRuntime/Source/OmGenerated。再次出現多段輸出截斷；後續應按單一檔案／窄區段調整 output budget，不推測被截內容。
- 擴充 struct 後全域 inventory 發現 client 測試兩處 literal 尚未補 rank，編譯前補齊；不得只確認 bridge library 編譯便認定所有 test targets 已同步。
- 最後權威3/3、core6/6、client7/7、bridge3/3成功；具體指令與版本限制見 authoritative-cast-rank-progress。未 stage 或完整驗收，不勾整項。

## E221：成功施法 visual 不是已發布的權威事實（2026-10-05）

- 下游ABY1／IPC／UE已有接線，但dispatcher成功SkillCast尚未進fact buffer。共用成功gate後經既有converter發布Ability，explicit HERO_ABILITY policy與來源路徑；不從input ACK／snapshot補播，不放寬hidden caster gate。
- 新producer-local ordering跨同tick dispatcher drains持續遞增，team投影再配安全wire ordinal。基本ECS初始化把secure policy registry與fact buffer放同層並加ordering resource，不做臨時policy fallback。
- 首輪2/2測試因registry缺失失敗，修共同初始化。初版又誤認headless SimulationDriver會寫committed team frame，改讀真實result.facts並顯式通過共用projector，報告不宣稱自動KCP／client全鏈。
- Skip誤填fifty、rg位置字串system*觸發os123，改整數／directory＋-g；大輸出縮小。補強負向fact斷言後重新編譯，最後successful_cast_facts3/3、cast_visual2/2成功，不跑全套。
- 詳見authoritative-cast-fact進度；未stage DLL／Unreal／真實同局驗收，不能因此勾完整6.1或宣稱全部cue參數已揭露。

## E220：施法 cue 消費必須共用保留、查表與原生去重（2026-10-05）

- bridge DMG1 專用 FrameSlot／pending／coalescing 改 PresentationCue 共用 ABY1。stable fact hash不等於UE catalog index，抽共用既有FNV-1a演算法，registry拒絕空／零／重複stable／numeric ID，未知不派發；string自有frame lease，不猜目標／rank／toggle state。
- 原生 caster-only AbilityCast 用既有cue history去重，Stop清空；cast不被fallback誤認disabled toggle。不增加角色分支或runtime Lua；C ABI13維持layout，flags bit由cbindgen重產header。
- 工具錯誤：restart在根cwd找錯uproject；改omfue。猜Subsystem／TargetDescriptor舊路徑、Skip誤用sixty、空omoba/none patch被拒絕，改inventory／整數／有效hunk並讀回。大log截斷改精確尾段，不猜證據路徑。
- 完整restart build仍FailedDueToEngineChange，沒有去掉NoEngineChanges；本機UBT確認-Module支援後限定三個project modules，14 actions成功，不當fullbuild完成。synthetic原生初輪2項Success但world無context警告，補上context生命週期，未壓警告。詳細結果與最後證據見ability-cue-unreal-consumer進度。
- 局部bridge cue4/4、damage2/2、core hash1/1、no-default-driver check；numeric duplicate補強後ability_cue3/3。未部署Rust DLL或完整同局鏈，不以局部接線宣稱整案完成。
- 最後原生feature report AbilityCast-1791186352：2/2 Success、errors0／warnings0、exit0；preview context警告已修，不壓警告。root／omfue whitespace與OpenSpec strict通過。全build engine baseline仍待整合，限定模組成功不代表fullbuild完成。

## E219：一次性施法必須共用可靠保留與不碰撞事件 ID（2026-10-05）

- 原 damage retention／baseline／ACK 後 prepared frame 過濾僅認 DMG1，ABY1 不能單純加入最新 snapshot。改共用 PresentationCue／CueRetention，正式 main 每個成功套用 frame 在畫面降頻前擷取傷害與施法；相同 live dependency／Hide／Forget／Reset／connection ACK／重連基線與總容量。
- 發現 external 序號與 public producer-local ordinal 會共用 tick＋ordinal ID，施法可能和傷害或另一 producer 撞 ID。權威排序後跨列表連續配置，保持安全 replica 身分，不暴露 canonical producer。
- 工具錯誤：兩次 patch hunk 次序由後段 test 回到前段 converter，verification failed 且未套用。依檔內位置由前到後重排後成功；先確認實際檔名再判斷 rename，不盲目重送。大輸出截斷須縮小，diff stat 不含 untracked 新檔且混有既有修改，另讀回新檔／編譯，不冒認全部差異。
- 局部 client cue 7/7（含 DMG1／ABY1 實際 localhost IPC）、core ability_cue 4/4、正式 client binary check 成功。bridge／Unreal 尚未消費 ABY1，不能稱完整施法或重連驗收；詳見 ability-cue-ipc-retention 進度檔。

## E218：可見目標不能代替隱藏施法者發布施法 cue（2026-10-05）

- 實際缺口：Ability 共用投影可能因 target 可見發布，subject fallback 指向 target，且帶出隱藏來源技能 ID。共用 gate 要求 source 可見且有 Disclosed mapping，否則不發布；不放寬傷害／AOE sanitizer。
- 現有 public Ability 沒有目標／位置；新 ABY1 固定 36 bytes 只含 tick、caster replica、epoch、stable skill ID。解碼嚴格版本／長度／非零，admission 嚴格 event ID 與 live epoch。後續仍需 catalog lookup、可靠保留／去重與 UE 接線，不用 ACK 或 remembered ghost 冒充成功施法。
- 工具錯誤：猜不存在根 Cargo.toml／replica.rs／team_identity.rs，改 inventory／型別搜尋；Select-Object Skip 誤用 sixty 改整數。大輸出截斷須縮小精確搜尋。測試編譯中補強 mapping case，重新編譯確認最後版本。
- 局部範圍與後續限制見 `2026-10-05-safe-ability-cue-contract-progress.md`，不得將格式／投影測試稱 IPC→Unreal 完整成功。
- 最後版本指定 ability_cue 3/3、OpenSpec strict 與 whitespace check 通過；428 項過濾，沒有重跑全套。

## E217：queued SkillCast 不是成功施法 cue（2026-10-05）

- 實際缺陷：dispatcher先加SkillCast visual再執行gate，legacy無Mana拒絕不一定回滾。共用外層以adapter.cast_succeeded判斷所有cast，失敗只回復本invocation的visual checkpoint，保留前面成功事件；不在Unreal按技能ID遮掩，也不把queued／input accepted當handler成功。
- 最後局部2/2確認有／無Mana的同隊拒絕、同批成功後失敗不刪前者、成功身分欄位、冷卻重送及缺handler；正式generated handler與真實dispatcher，不是mock。未放寬投影policy／audience或修改既有legacy Outcome rollback語義。
- 工具錯誤：PowerShell下把*.rs當rg位置字串會得到os error123；改rg DIRECTORY -g '*.rs'。又猜不存在native/script_system.rs，改先rg --files定位scripting/dispatch.rs。大輸出截斷時改縮小區塊，不據截斷內容作完成判斷。
- 修改測試時第一輪編譯已開始，舊binary結果不能證明新case；補強同批case後重新cargo指定filter，最後2/2成功。只針對變更做確認，不因此重跑全套。
- 詳見successful-cast-cue進度；完整IPC→Unreal、cue重連及6.1／4.3尚未驗收，不能把此來源gate稱作全鏈完成。

## E216：技能 fallback 不得依名稱派發或猜距離單位（2026-10-05）

- 實際缺口：OnAbilityCue依sniper_mode／saika_reinforcements／rain_iron_cannon／three_stage_technique比較派發。改Lua ue.ability_cue五種通用形狀生成原生registry，Actor只轉送；Blueprint／payload／C ABI保留，不把美術參數當傷害規則。
- 單位缺陷：原AoeRadius<50時乘100、大值當cm，值域不代表單位；改明確payload_distance_scale，legacy宣告100、未宣告預設1。原cone轉成degree後直接交DrawDebugCone；新模板在draw邊界用radians，不按ID修補。
- 安全規則：生成期嚴格RGB／shape／未知欄位與有界有限正數、count限制；事件距離非有限／非正值fallback、有限值截限，count截限。12個非法fixture均拒絕且不覆寫生成檔。
- 實際編輯失誤：新增header時兩個#include被串成同一行，讀回立即發現並在編譯前修正。patch後應讀回關鍵header／宣告，不等編譯器才找純文字錯誤。
- 生成器指定1/1、原生12項project build、NullRHI單項run1791183357的1/1 Success／exit0通過。此測試不是特效像素／所有cue可靠發布／完整對局驗收；詳細邊界見declarative-ability-style進度檔。
- 新full catalog hash23c4614509fdbf2d；未重新部署Rust DLL，不能拿unchanged presentation hash91320001f1ba2429聲稱完整版本相同。下次正式開局保留full catalog握手與一致部署。

## E215：投射物 fallback 不能靠兩份名稱分支維護（2026-10-05）

- 實際缺口：OmProjectileVisual 與 OmUnitActor 重複投射物 ID 顏色／爆炸判斷，半徑與線寬不一致。改 Lua ue.projectile_cue→codegen 原生 registry→共用 game-thread lookup；未知 ID、未宣告或卸載後使用通用預設，不新增角色分支或執行期 Lua。
- 數值與拼字防錯：RGB 必須三個 u8；有限正數線寬≤100 cm、半徑≤10000 cm，未知欄位拒絕。七種負向 fixture 確認拒絕時不覆寫既有生成檔；tombstone 不註冊。爆炸120 cm只代表呈現，不可拿來改玩法作用範圍。
- 版本邊界：完整 catalog data hash 因 Lua 呈現宣告更新為40641b573c2890e9，既有 presentation content_hash91320001f1ba2429不包含此新設定；不能把後者當完整內容身分。保留正式 full catalog handshake，未部署舊 DLL 冒稱完整可用。
- 工具查找仍出現猜不存在 types.rs／restart/src/build.rs 的錯誤；先 rg --files，再讀已存在 lib.rs，不依猜測檔名搜尋。大型歷史 artifacts／搜尋輸出曾截斷，應逐檔／小區塊讀取並排除長 generated JSON 行。
- 生成器單項1/1與保留-NoEngineChanges的15項project增量編譯成功；既有插件 dependency warnings 未隱藏，沒有新的編譯失敗。原生NullRHI單項run1791182856為1/1 Success、exit0，沒有特效像素或完整對局驗收，不以局部結果封關完整6.1。紀錄build log曾先寫錯時間戳，已依目錄清單改1791182806；證據路徑必須查實再引用，不猜檔名。

## E214：bridge 拷貝相同不代表 script DLL 部署完整（2026-10-05）

- 查找工具模組時曾錯猜 scripts/lib/process.lua，不存在；應先讀 scripts/_bootstrap.lua 的 lib mapping，再以 rg --files 找到 tools/lua/lib/process.lua。已確認 process.run 預設對非零 exit code 直接失敗，不會繼續選角或開局。
- 實際缺口：verify-staged-only 只比對 bridge，而 server 與 Unreal 分別從 scripts 與 plugin 載入 base_content。改用共用 artifact-copy-consistency 契約檢查三份部署 copy；缺失或不一致拒絕，不自动換 profile、重新拷貝或忽略版本。
- 開局順序缺口：server／client runtime 原在選角後才建置。新增共用 workflow，所有建置與初次部署檢查在選角前；選角後再 guard 一次，--no-build 也不能跳過，prepare-only 不啟動相關程序。
- 安全決定：輸出／選角目錄重複在建置前就拒絕；既有目錄不寫入 errors.md。原有建置／選角失敗記錄路徑保留，不啟動 fallback 遊戲。
- 當前 8/8 功能確認使用實際 fixture SHA 與 mock 流程，舊 script DLL 的拒絕是測試中的預期失敗。本批沒有虛構編譯或遊戲執行失敗，也沒有完整驗收。
- 界線：此 report 僅證明拷貝一致，不能推論 compiled-only feature、ABI、內容 hash、來源新鮮度或 gameplay 成功。既有編譯約束與 runtime handshake 不能移除。

## E213：重新確認引擎基線，不把歷史阻礙當永久狀態（2026-10-05）

- 本批只讀確認：目標專案 Editor 數量為 0，共享引擎仍有四個 Animation／Skeletal 來源修改；全部保留，不還原、不移除 -NoEngineChanges。舊 ue58_build.log 是 9/9 的紀錄，不能當現在成功或失敗證據。
- 新 build 保留 -NoEngineChanges 已通過引擎閘門，實際失敗是 OmPlayerController.cpp C4458：區域變數 Player 遮蔽 APlayerController::Player。改名 SelectionPlayerId，不降警告或更改父類別；後續 project-only native compile／link 成功，E177 不再是目前建置阻礙。
- 診斷錯誤：rg 命中單行巨大的 TargetMetadata.json，造成輸出截斷；改讀明確 build log 摘要，不以截斷內容推論成功，也不繼續直接印整份 metadata。
- 選角獨立性：Initialize 原本會 LoadBridge，雖未建立 World，仍使選角依賴 gameplay DLL。改選角模式不載入 bridge，StartRuntime 最終 gate 保留；不借舊 DLL 或版本放寬確認選角。
- 原生選角單項確認入口加入明確 opt-in 的非 Shipping pointer 操作，只使用真正 Slate 按鈕與已布局 geometry，沒有虛構座標或直接改 kernel。此入口只選角、不啟動對局；最終結果另記進度檔。
- 真實功能失敗：1791181448-1 的 initial handshake 成功，但合成 touch down/up 都 handled 而 request 仍 0。檢查 SButton 實作，預設 DownAndUp 需 Slate capture；改用既有商店同一 PreciseTap 觸控策略，以 geometry 判定釋放，滑鼠模式不變。未強制 enable 或直接呼叫 Send 冒充按鈕；失敗自動化新增明確退出，避免已報錯畫面等到測試總期限。原 run 已 timeout 並只清理所擁有的 renderer，errors.md 與 log 保留。
- 使用者另明確停用 omfx 後續維護，已同步 AGENTS／README／OpenSpec；不回修或建置 Fyrox，不刪除舊檔案。
- 修正後真實 run1791181633-1 成功：指定真人 ready、原生 pointer select／lock／finalize request1／2／3各一次、最終 training_ranger 配方正確、沒有 gameplay bridge／對局。最後 native build1791181625成功；完整遊戲驗收未跑。詳見 unreal-selection-native-confirmation 進度檔。

## E212：選角不得順便啟動玩法；原生 pipe 與啟動交接要分離（2026-10-05）

- 實際缺口：遊戲 controller 與 world subsystem 原本自動建立戰鬥介面／bridge。新增 om-hero-selection 模式及 StartRuntimeFromSettings 最終 gate，選角階段沒有 gameplay World；關閉選角不自動開局。
- pipe API 調查：Windows 的 FString WritePipe 自帶換行，直接使用可能多送空行；改用 byte overload 明確寫一次換行並檢查完整寫入。stdout 以 bytes 留存至完整換行再轉 UTF-8，stderr 另行 drain，不混入 JSON。只保留自己的 process handle，不能按名稱殺其他程序。
- 整數防錯：Unreal JSON numeric 經 double 可能截斷 u64 revision；Rust 新增 decimal tokens，C++ 原樣回送驗證後的 revision 整數，不縮減 Rust kernel 的版本範圍。
- 實際程式 API 缺陷：moba_role_launch 的 lifecycle loop 使用 time.sleep，但固定 time library 只提供 sleep_ms。修正呼叫與 mock；未聲稱本批重現過真人對局崩潰。
- 交接：退出選角 renderer 後才讀結果，NoReplaceExisting 不覆寫；檢查 hash／身分／完成狀態及完整 host-owned 配方，僅允許真人英雄變更。UI 取消或非法結果記 MD 並拒絕開局，舊 --hero override 不再覆蓋玩家明確選擇。
- 舊 binary 防錯：新增 presentation-only 保護旗標；45 秒內必須讀到指定真人／protocol1 的 OM_SELECTION_READY，否則停止自己的 renderer 並拒絕交接。握手完成後不把玩家思考時間當啟動逾時。
- 確認範圍：Rust native 子程序 1/1、Lua mock 最終 6/6；新 C++ 尚未編譯或畫面確認，E177 既有共享引擎限制不擅自繞過。不以 source 寫入或 mock 測試冒充 UE 功能已驗證。

## E211：一次性自動鎖定不能當互動選角入口（2026-10-05）

- 缺口：既有 launcher 在 Unreal 開啟前自動鎖定配方，沒有持續處理真人選擇的服務。新增共用 Rust SelectionService／moba-config --selection-session，逐則 read／select／lock／finalize；沒有新增 Lua 執行期。
- 防錯決定：身分只由可信 host 啟動時綁定，請求不含 player ID；逐則版本、hash、revision 檢查，拒絕不更動。EOF 不自動 finalize，不將其他真人未同意的席位自動鎖定。
- framing：16 KiB 上限、只執行完整換行 frame、flush 回覆；超長輸入直接終止，不能把尾端誤認下一個合法請求。一般非法 JSON 可回覆後繼續。
- 工具限制：現有 Lua process.run／spawn 沒有 child stdin 管道，不能臆造 stdin option。當前以 Rust IO 功能測試確認，不新增 PowerShell/Python workflow fallback，也不把單人 stdin service 宣稱為 LAN 共享大廳或 Unreal UI。
- 本批首輪沒有新增編譯失敗；記錄的是實際架構缺口與預防規則。結果見 hero-selection-service 進度檔。

## E210：選角鎖定要有共用權威狀態，不可只保存 UI 選擇（2026-10-05）

- 缺口：之前 --hero 只準備已選配方，沒有真人鎖定／全部 ready／finalize 狀態。新增 Rust HeroSelectionSession，不建立 World／Lua VM；席位、版本、鎖定與最終配方由主機持有。
- 通用決定：請求只有 expected_revision／action，玩家 ID 由呼叫端的 admitted identity 傳入；未知玩家、Bot、已鎖定、過期版本、非法 compiled hero 與版本溢位皆不改狀態。全部真人鎖定才 finalize，finalize 後不能修改或再次開始。這個 kernel 不是連線認證，尚未實作網路大廳，不可宣稱自動防止未認證呼叫。
- UI 來源：建置生成 HERO_CATALOG_IDS 排除 tombstone，Rust 共用 compiled catalog 提供 ID／名稱／稱號／portrait／技能；不在 Lua／Unreal 重寫英雄白名單。
- 工具錯誤：cargo test --manifest-path scripts/Cargo.toml -p omoba-core --features compiled-content-only 被拒絕，因 core 是 workspace 外的 path dependency，不能從這個 manifest 指定其 feature。改以相同 manifest 的 core 正常測試（無 runtime Lua），正式 moba-config 則由 omobab/compiled-content-only 傳遞約束；不切换 Rust 工具鏈或清 lock。
- 真實局部失敗：先寫 candidate 到 match-plan.json，finalize 後再次寫同一路徑，被 path.write 防覆寫拒絕。改分開 selection-candidate.json 與最終 match-plan.json，各寫一次，保留準備資料與來源，不放寬安全預設。
- 本機 --lock-plan 明確屬 trusted host preparation，不冒充多人玩家已按鎖定；互動 UI／連線控制權／選角到結算仍待。當前結果見 hero-selection-lock 進度檔。

## E209：關閉熱更新不等於沒有執行期 Lua（2026-10-05）

- 架構偏離：正式 launcher／DLL build 啟用 runtime-lua-content；bridge 還直接依賴帶 Lua VM 的 C++ 生成器，並在 RuntimeInner::new 重新求值作者檔來取得 catalog。只改 LUA_HOT_RELOAD=false 無法符合使用者指定的生成／原生執行架構。
- 修正：正式 compiled-content-only 與 runtime-lua-content 互斥，四個執行元件正常依賴沒有 Lua VM；生成器 authoring optional，bridge build.rs 在建置端生成嵌入 catalog，遊戲只反序列化內嵌資料。DEV reload 保留明確 opt-in，正式要求 reload flag 也拒絕；非零 Lua generation/hash batch 不接受。
- 編譯錯誤：把 om_codegen/compiled-content-only 從 bridge feature 轉送時，也影響 build dependency 的 authoring unit，導致本來允許的建置生成器被拒絕。保留 resolver2，改 bridge const assertion 檢查其實際 target om_codegen::AUTHORING_ENABLED；host authoring 不受限制，target VM 仍被阻擋。
- 測試錯誤：TOML formatter 輸出 LUA_CONTENT = false，測試用固定無空白字串誤判。改接受合法欄位空白，不改生成內容。
- 操作錯誤：rg 對明確含 *.rs／run*ue.lua 路徑不會由 PowerShell 展開，另猜了不存在的 omfue/build.rs、state_initializer.rs、根 Cargo.toml。改對實際目錄搭配 -g 或 rg --files；本專案沒有根 Cargo workspace，不可再猜。
- patch 錯誤：同一檔的 hunks 先後次序倒置導致定位失敗，拆成按原始碼順序的 patch 後成功；不重複盲試。
- 當前確認與限制見 compiled-content-only 進度檔；不把 cargo check 或作者檔不存在的 fixture 當成已部署 Unreal／無來源完整對局验收。共享引擎 E177 不處理。

## E208：開局選角必須進入權威配方，不是只改 renderer 名稱（2026-10-05）

- 缺口：互動啟動只有固定 Lua 配方，真人選英雄需手改資料。新增共用 select_heroes 複製配方後套用真人選擇，再沿既有 Rust moba-config 檢查；不修改原檔、不由 renderer 決定出生英雄。
- 邊界：拒絕重複 CLI 玩家（含 1／01 別名）、零／超 u32、未知玩家、Bot 與不合法名稱；未知 catalog 由 Rust 拒絕，沒有新增 Lua 英雄白名單。隊伍／位置／Bot 策略不變，尚非 UE 選角畫面或連線大廳。
- 操作錯誤：再次猜成 omb/src/bin/moba-config.rs，實際為 moba_config.rs；已用 rg --files 確認後讀取。必須先盤點路徑，不把 bin 名稱當來源檔名。
- 當前確認結果見 pre-match-hero-selection 進度檔；未做完整對局驗收。

## E207：不要把已一致的單體選敵誤認成缺口；範圍清野需共享焦點（2026-10-05）

- 調查更正：最初說技能低HP優先，完整讀priority後證實EnemyUnit已距離優先且中立技能已支援。真正缺口是EnemyPoint按cluster count可能離開普攻的近處營地；沒有為錯誤假設重寫單體語義。
- 通用解：jungle_farm_target只取current披露、kind3／team0／正HP、550內，distance／canonical ID穩定排序；普攻與role_combat_focus共用。可見隊友支援仍先於清野，範圍技能沿既有focus排序且min_targets／range／Mana／CD與作者heal優先不變；不新增hero分支或hidden營地查詢。
- 真實編譯錯誤：fixture假設CProperty有Default導致E0277，讀實際型別後填全部五欄，不新增Default來掩蓋假設。
- 真實測試錯誤：min_targets2的遠端cluster fixture放900超出compiled range700而unwrap失敗，改600／610（近camp100），不放寬正式射程。
- 當前確認：core新1與正常generated正式60Hz新1首輪成功；最後shared helper／既有assist結果見jungle-farm-focus進度。第三隻中立fixture只提供正常Unit／Faction／Pos／Property與Vision scope，不冒充完整camp reward／respawn驗收。完整5.5及100場／UE仍待。

## E206：最近頂點不是折線所在段，不能直接跳到其下一點（2026-10-05）

- 確定缺口：route_destination用nearest vertex＋1，在第一段走過半時可提前跳過轉角，與前批Jungle時計改道不同。
- 通用解：有本人正式AttackMove路線目的地就完成該段；否則以Fixed64最近線段投影定位，穩定index決勝，50單位內到達才前進。重疊／單點／空線安全、反向route自然支援；保留combat／sustain／技能與正式地形移動，不讀hidden敵人或新增私有cursor。
- 真實錯誤：base fixture讀MobaMatch.routes造成E0616 private field。改讀公開compiled moba_map_by_name(...).lanes[0].waypoints，不把routes改pub、不增測試专用API。
- 當前確認：core新1與正常generated manifest正式60Hz新1通過；40steps實際位移、不重送、未commit arrival不前進、commit後正常接納下一個AttackMove。不是全部三路終局／不可達導航／100場／UE驗收，詳見lane-segment-progression進度。

## E205：按固定時間輪換目的地不能代表完成野區巡邏（2026-10-05）

- 原始碼缺口：Jungle以tick／(think_interval×100)選營地，不確認抵達，途中可能被時鐘改道。這是策略缺口，不宣稱已重現完整對局死局。
- 通用決定：pure patrol_destination使用公開點、本人的committed位置與本人正式AttackMove目的地；途中保留，50單位內抵達才循環轉點，重疊／已到達點跳過，空／單點安全。combat／學習／恢復優先不變，中斷後依位置重選，不讀camp alive／respawn／aggro。
- 工具錯誤：先搜尋不存在的runtime/simulation_driver.rs，實際檔在runtime/native/simulation_driver.rs。以rg --files確認後讀取，未新增fallback；仍必須先盤點實際路徑。
- Fixture改進：driver.tick即本次已完成tick，不需減1；首輪通過後改成精確当前tick發布Wave B。未提交arrival mutation不影響選點，正常正式input才改命令／移動。
- 當前確認：pure planner1與正常generated manifest／60Hz正式driver1首輪成功，重疊點與精確tick補強後結果見jungle-arrival-patrol進度。無Rust編譯／測試失敗，未全套／100場／UE／stage，完整5.5仍待。

## E204：filter後take只限制有效輸出，不限制cue解碼工作（2026-10-05）

- 確定缺口：bridge原effects.iter().filter_map(...).take(1024)，無效前綴可讓解碼掃過整批；每個cue再線性搜尋實體。這是原始碼與局部fixture確認，不冒稱量測到60Hz掉幀。
- 通用解：core共用DMG1 effect ID／tick驗證與1024常數；runtime ledger及bridge使用相同契約。bridge在decode前take，建立一次(render_id, disclosure_epoch)索引，同批effect ID去重；不查hidden、memory或actor，不改既有跨frame消費／重連／ResetView機制。
- 邊界：超出輸入budget的尾端不掃描且明確warn；無效與未知records也佔budget，不默默掃尾端補滿。這個限制針對既有damage輸入，不宣稱所有音效／技能cue已完成。
- 工具錯誤：本批又嘗試讀不存在的omfue/Cargo.toml。真正入口已確認為omfue/bridge/Cargo.toml；不得把Unreal專案根推測成Cargo workspace，後續只使用已讀過的manifest。
- 局部確認：core事件契約1與runtime retention3成功；bridge本功能結果見damage-cue-admission進度。未跑全套、未部署、未繞E177共享引擎閘門。

## E203：有Hold轉換與模擬，不代表server網路准入／accepted投影已接通（2026-10-05）

- 原始碼缺口：secure coordinate白名單未包含HoldPosition；不能靠renderer codec與ECS測試推論玩家網路命令已可用。
- 通用決定：新增command protocol1＋完整content rules hash，與shop／Recall／Mana獨立協商。只有bound selective MOBA player啟用；legacy 0／空值保持關閉，舊移動相容。client在分配input ID前拒絕未協商Hold；server仍檢查owner與nonzero input ID，cached／fresh bootstrap同樣回報。
- 真正編譯錯誤：client binary新增接線用了不存在的ClientRuntimeError，改用既有anyhow結果；server accepted-input的exhaustive match漏Hold，補append-only action_kind20／無target，保留既有owner解析與安全投影。
- 預防：新增command能力必須檢查Join／bootstrap、client admission、server admission、accepted input投影及generated fallback，不能只補兩端encoder。protobuf fallback經既有build腳本機械更新，不手改generated。
- 當前確認：core契約1、client input bridge13、server能力契約1通過；client binary check成功。server binary check結果記於command-capability進度。沒有全套／UE／完整網路對局驗收，未stage、未改ABI13／IPC4／KCP2。

## E202：等待不能同時用空命令、MoveTo自己與Hold三種語意（2026-10-05）

- 缺口：Recovery::Hold與近距離Escort仍MoveTo自己／idle，無法持續抑制autoattack；推塔已用正式Hold。共用hold_input，active Hold＋空queue去重，其餘送普通立即Hold以正常清stale queue。
- 通用決定：HP／Mana恢復、推塔與Escort等待共用；recovery_move只處理真Move，不保留自己的pose／在途Stop workaround。恢復門檻與committed隊友遠離後沿正式命令release，不直接改ECS／HP／Pos。
- 工具失敗：初次patch的Escort尾段context因先前輸出截斷而不完整，verification failed且未套用；重讀完整短區段後精確patch成功。防止以截斷結果猜尾段。
- 正常建置鎖：同scripts target並行cargo導致artifact lock等待，未kill／刪lock；後續同workspace串行建置以免浪費。非runtime failure。
- 當前確認：core新1、base既有sustain3／support1更新後成功；沒有Rust編譯／測試失敗。等待不重送、回城恢復／暫停／離開門檻、commit後重新follow皆通過。完整UE／網路／100場未做，詳見role-bot-persistent-wait進度，20/30保持。


## E201：前端編譯成功不代表IPC版本一致；轉換與常數應共用（2026-10-05）

- 盤點發現：Fyrox presentation_client仍const VERSION3，上一批queued修復／lib check成功不能保證其連上v4 runtime。這是原始碼缺口，未冒稱實際連線failure。
- 通用解：omoba-core純renderer_protocol持唯一magic／IPC4／framing bound與exhaustive PlayerInput encoder；兩個前端只forward，client re-export constants。缺point／未知item／原本unsupported action明確拒絕，新oneof分支必須編譯處理。
- 邊界：encoder不授權／不查hidden world，保留client owner／epoch／secure target與server admission。Fyrox支援集合與Unreal一致但無自動送指令；仍以omfue為主要前端，不新增runtime-heavy dependency。
- 當前確認：新core3／client1、既有bridge point_queue1 passed，omfx lib check成功；本批無編譯或測試失敗，既有template warnings保留，不虛構error。沒有UE／LAN／完整驗收／stage，詳見shared-renderer-input-codec進度，20/30與E177保持。
- 預防：新增輸入協定不能只搜尋struct literal；也要查版本常數與每個consumer。共用常數與exhaustive match優先於複製修正／wildcard。


## E200：Shift旗標不能只留在UE事件；正式位移有階段時序（2026-10-05）

- 發現：MoveToIntent／AttackMoveIntent只有座標，Unreal／Fyrox IPC adapter遺失queued，InputBridge固定false。先前小地圖事件有Shift資料不是權威排隊驗收成功。
- 通用解：兩intent append bool queued3，adapter／InputBridge保留原值；IPC4拒絕3，避免protobuf unknown-field靜默忽略。C ABI13／selective2／HUD schema2／Lua內容版本不混改；固定build.rs機械刷新game.rs。
- 真實失敗：60Hz新fixture已驗Hold＋兩queued append／立即Move replace，卻在提交當tick要求位置必變而失敗；保留隊列斷言，storage借用結束後推進20ticks確認實際位移，第二次成功，未更改正式phase。
- 預防：跨層輸入新增欄位時，必須檢查所有converter與反向admission、proto round-trip及舊版本拒絕；ACK不是queue行為證據，command接收不是同tick物理完成。
- 當前確認：本批新client2／bridge1／正常60Hz base1成功，另既有checkpoint1成功，client binary check成功。UE／完整驗收與部署未做，詳見point-command-queue進度；E177與20/30維持。
- 相容編譯真實失敗：Fyrox native InputActionKind分類少前批Hold20，cargo check E0004。補明確HoldPosition enum／match及presentation converter，不映射NoOp或吞掉未知值；再次omfx lib check成功。跨workspace新增oneof必須查所有exhaustive match，不只主要UE workspace。


## E199：新 Hold 不得交給舊 enum DLL；鍵盤 modifier 要明確綁定（2026-10-05）

- 決定：共用 Runtime action→C command15→PlayerInput／RendererInput Hold，queued保留，原生H／Shift+H；不用角色專屬生成程式或BP、不用自身MoveTo。舊embedded路徑分配交易前拒絕。
- ABI風險：舊Rust repr(C) enum不認識15，不能只改header後仍載入舊DLL。Rust/header同步ABI13，focused測試驗舊12拒絕及header一致；未stage／UE，E177持續阻擋。
- 事前修正：只綁無modifier H不能保證Shift+H會匹配，補明確Shift chord；不是已發生真人按鍵失敗的宣告。
- 工具失誤：本批初始又用Get-Content猜omfue/AGENTS.md且忽略不存在，後以rg --files -g AGENTS.md確認沒有。避免靜默忽略讀取錯誤，後續只讀已盤點路徑。
- 當前確認：bridge hold_position兩測成功；新增Editor reflection assertion未執行，原生按鍵未編譯驗證。沒有新compile/test failure；既有template dead_code warnings保留，不虛構失敗。详見unreal-hold-input進度。


## E198：MoveTo 自己不是持續 Hold；攻城援軍 fixture 要活著且支援多路（2026-10-05）

- 缺口：MoveTo 抵達後 queue.advance，autoattack 可重新開始；AttackMove fallback 也可能繼續進塔。不能靠重送自身位置或假 NoOp 取代持續等待。
- 通用解：PlayerInput／RendererInput 新增 HoldPosition20（queued），正式 PendingHeroCommand→HeroCommand::HoldPosition；既有 owner admission／replace／queue 上限／attack interruption 生效，command tick 保留且無 MoveTarget，hero tick 抑制 autoattack。原安全 MovementPriority 只公開抑制旗標供 filtered replay，不公開敵方命令內容。MOBA allowlist 與同批 Recall 中斷接線。
- 推塔：目前已披露、存活、同隊 kind2 兵距塔650內才視為推塔時機；沒有援軍則1100內觀測區使用正式 Hold，不退回 AttackMove。既有戰鬥與生存優先；不是塔已鎖定兵的保證，Hold 不免傷、不自動撤退。
- 真實 fixture 失敗1：舊 lane_creeps helper 限 path==__single_lane__，layered map 回空 vec 而 [0] 越界；改依 Creep／Faction 群組取得同隊兵，沒有地圖名特例。
- 真實 fixture 失敗2：加入多層塔附近的180HP兵在首次commit被塔擊殺，wave_pos／wave_hp=None，Bot正確選Hold。這個fixture只隔離unlock／Hide／Reveal，給其援軍10000HP維持短窗口存活，不改正式數值或允許死兵。修正後舊攻城測試成功。
- 工具失誤：猜 comp/hero_commands.rs、native/hero_command.rs、根 Cargo.toml 不存在；實際phys.rs與lockstep_resources.rs、各自workspace manifest。應只對已確認目錄 rg／rg --files 盤點，不猜名稱。
- package cache短暫鎖：唯讀確認另有既存cargo build，未終止他人程序／刪cache鎖；原測試等鎖釋放後完成。C槽滿沿E194僅當次TEMP/TMP轉D槽、退出還原。
- 當前確認：新core wave1、新client hold1、新base wave1＋filtered1成功，另既有siege core1／base1成功。Hold持續65 ticks不移動／autoattack、不重送，uncommitted兵不供援軍，披露後正式AttackTarget取代；Hold／Move release共12雙隊filtered steps hash一致零repair。未完整驗收／UE／100場／DLLstage，20/30與E177保持。


## E197：建築不能當兵；Hide fixture 不得偷移動靜態塔（2026-10-05）

- 缺口：MOBA structure 也有 Unit，原 render.kind=2，role Bot 視為兵；既有 safe payload 亦未提供解鎖狀態。不能用名稱／私有 MobaMatch 推算攻城，也不能把塔當 Carry 尾刀。
- 通用解：獨立 0x464f4710 v1（role1塔／2基地、damage admission0／1），由同一 structure_unlocked／phase 規則擷取，CommittedStructure28 change-only absolute fact，既有 visibility policy 管 audience。保留 renderer.kind 與 property 格式，不增加角色 C++／BP。
- 結算邊界：檢查時發現若在 finish 入口擷取，當 tick 勝負後 baseline=false 而事件仍 true；在測試前改為勝負結算、釋放 state borrow 之後擷取。warmup／Finished early-return 也同步，避免過期 flags。不是已发生 runtime failure 的根因宣告。
- Bot：只在 current 披露辨識 tower／base／locked；兵／hero 優先於建築，locked／dead／friendly／neutral／outside 不選。Carry farming／offensive skill 目標不混建築；Jungle 保留原 camp／局部支援策略。
- 真實測試失敗：新 filtered fixture team2 step1 hash 不符；fixture 直接移動靜態 inner tower 6000 單位，但既有建築位置不走玩家／NPC 移動事件。改成移動英雄離開／返回視野，測真正 Hide／Reveal；不為錯誤 fixture 增加動態塔或 ComponentRepair。
- 工具失誤：猜 omoba-client-runtime/src/render_projection.rs 不存在。改對已確認目錄 rg 查詢（presentation_bridge.rs 等）；沿 E194/195 規則，先盤點路徑，不能猜模組名。
- 當前確認：core disclosed_s 篩選3 passed（新2＋既有1）、base disclosed_structure_state 新1 passed；正式60Hz解鎖／Hide／Reveal／Finished共8雙隊filtered steps每步hash一致、零repair。沒有全套／100場／UE／stage，E177與20/30保持。詳見disclosed-structure-siege進度。


## E196：尾刀優先須跨普攻與技能決策；不要把後搖當前搖（2026-10-05）

- 缺口：E195 的可擊殺排序在 choose_cast 之後執行，available 進攻技能會先佔本次輸入，普攻優先可能無法使用。不是證明每種技能都會取消普攻，不能據此修改 gameplay 取消規則。
- 通用決定：每 Carry 一次計算 safe current 尾刀候選；當正式 Idle 且 asd_count ≥有效 interval，或 Windup／負 asd_count／active AttackTarget 正是可擊殺兵時，defer offensive intentions。符合條件的既有 Windup 目標優先保留，避免換到其他低 HP 兵重送命令。
- 保留：治療／自身資源恢復仍按作者順序選擇，sustain／回城與學技能的既有順序不變。其他角色不套 gate；cooldown／Backswing 不因尾刀候選停用進攻技能，unknown／hidden／miss 不建立尾刀保留。
- 工具／建置：本批沒有編譯或測試失敗，不虛構 failure。新測試轉接 wrapper 在非 test 編譯出現 dead_code warning；確認全部 caller 只在測試後加 cfg(test)，保留原測試介面而不新增 production 死碼。
- 確認：core carry_attack_priority 1 passed；base 正式60Hz同篩選 1 passed，涵蓋就緒 AttackTarget、前搖不重送、低血生成療傷真的增加HP、後搖可 cast。未全套／100場／UE／DLL stage；E177與20/30保持。詳見carry-attack-priority進度。


## E195：入傷倍率須同步後續變更；未提交過的 fixture 不是 change-only 事件（2026-10-05）

- 通用修正：新增獨立披露 schema 0x464f470f（v1，8-byte signed Q10 bonus），保留 property 的既有 40 bytes。MOBA 從權威聚合生成 baseline；CommittedIncomingDamage=27 在正式 finish hook、fact barrier 前只發變更，使用既有 HERO_ABILITY 視野 policy。只公開總倍率，不公開 buff ID／來源／持續時間；不是 Bot 特權 cache 或每 tick ComponentRepair。
- Replica：allowlist、restore 驗證、事件序列化、absolute event apply 同步接線，保持 metadata 輸出／canonical hash。非法長度與加 1 溢位先拒絕、不寫入；出生／Reveal baseline 帶最新值，Hide 的即時變更不外洩。
- Carry：只用當隊 current 披露敵兵 HP／位置／倍率，以及本人正式 final_atk／range／accuracy。必中且範圍內的目前可擊殺候選優先；unknown 或低命中保留既有策略。不預言投射物到達時 HP，不改技能政策，不讀敵方 BuffStore。
- 編譯錯誤：新 emitter 缺 specs::Join 匯入（E0599）；TAttack.range 是 Vf32，需傳 range.v 給 Fixed64 helper（E0308）。查既有型別後修正，未新增轉型猜測。
- Fixture 失敗：加 buff 後只手動生成 visibility，隨即移除，authority change cache 從未提交負倍率，因此正確不發 0→0 事件。改為先走正式 driver commit 再測移除，不修改產品的 change-only 行為以迎合測試。
- 路徑／命令失誤：PowerShell 傳 literal native/*.rs 給 rg 得 os error123；猜 runtime.rs／comp/attack.rs 不存在。改對已確認目錄 rg 或 rg --files 盤點，不能再以命名慣例猜實際路徑。
- 當前確認：core 新 2 passed、base 新 2 passed；正式 60Hz Carry 輸入、未提交 buff 不影響策略、hidden cache 排除；倍率更新／Hide／Reveal 共 12 個雙隊 filtered steps 每步 hash 一致且零 ComponentRepair。沒有全套驗收、100 場、DLL stage 或 UE；E177 保持。細節見 incoming-damage-observation 進度。


## E194：補刀不能套用未接線護甲公式；C 槽滿造成 linker 暫存失敗（2026-10-05）

- 檢查：目前 hero 普攻經 ProjectileLine2／Projectile／Outcome::Damage；來源 final_atk 與 accuracy 在產生 packet 時處理，入傷結算是三類總和乘 DamageTakenBonus。獨立 apply_incoming_damage 護甲 helper 不是這條正式路徑，不得拿它做另一套「精準補刀」公式。
- 決定：抽出純函式 settle_damage_packet 與 UnitStats.incoming_damage_packet，一般 HP 與 TD 分層共用。保持 fixed-point 先合計再乘與原本負倍率歸零，不在這次重構改護甲、暴擊、命中或傷害規則。
- 視野邊界：Bot 現有已披露 property 沒有 DamageTakenBonus；缺值不代表零。不讀敵方私有 BuffStore，也不把新 wire 欄位只塞 baseline 而漏掉後續更新。Carry 精準尾刀策略尚未接入。
- 操作失誤：再次猜 native/visibility.rs、native/mod.rs、native/ability_runtime.rs；實際 visibility.rs 在 runtime，ability_runtime 是目錄。應先 rg --files，再讀已確認路徑；SilentlyContinue 不得掩蓋不存在的路徑。
- 環境失敗：首次新功能測試 LNK1108，C 槽 Free=0，link.exe 無法寫 AppData/Local/Temp/lnk*.tmp；Cargo cache 也報 disk full。不是 Rust 編譯錯誤。僅當次命令把 TEMP/TMP 指向已存在 D:/code/omoba/scripts/target/debug，finally 還原，未刪檔、未改系統設定、未新增工作流 fallback。
- 修正後確認：base_content 指定新測試 1 passed（正式 60Hz generated lumen_bolt，倍率 1.5／0.5／負倍率歸零）；core 指定新測試 1 passed（含合計後 rounding）。沒有完整驗收、DLL stage 或 UE 確認。E177 保持。


## E193：兵線金錢要首次致死歸屬；exit0且0 tests不算功能確認（2026-10-05）

- 缺口：範圍XP不能取代尾刀Gold。Lua lane_creep_gold20與compiled constant／config／規則hash接線；權威正傷害兩路共用首個致死退休，匿名／NPC也退休，Heal後再致死不得補領。只付目前roster敵hero，正常Gold保存／重生／安全投影，不用前端或legacy Bounty計算。
- 防重複：LaneUnit.last_hit_retired與規則納入digest，generation生命track重置／Death移除；forced Death不領、pause／Finished不付，金錢飽和不增加hero KDA。
- 測試操作失誤：template測試漏runtime-lua-content feature，exit0但running0tests。改正確feature後實際1 passed；不能記成原先成功。base指定新功能2 passed、codegen生成與check成功。
- 重犯路徑失誤：猜native/outcome.rs與OmGenerated/om_content_manifest.json各回os error2；實際game_processor.rs與om_codegen_manifest.json，改已知目錄rg與rg --files盤點。不得猜路徑、不得加shell fallback。
- 新catalog data502a59ee5aa2a677；未stage／UE／full suite，E177與20/30保持。詳見lane-last-hit-gold進度。


## E192：Support護衛不得讀隱藏仇恨或讓普攻與技能各選目標（2026-10-05）

- 功能缺口：Support既有近身英雄優先不等於escort保護。新增當隊current披露Carry與敵hero近距觀察，以距Carry／本人／canonical ID排序；不使用敵方命令或私有aggro，不稱為實際攻擊者辨識。
- 通用決定：role_combat_focus共用Support護衛與Jungle支援，每Bot一次、進攻技能與AttackTarget共用；rank／range／Mana／CD／min_targets與作者治療優先仍生效。正式輸入結算，不新增角色C++／BP。
- 工具錯誤：猜runtime/native/single_lane/bots回os error3，再由rg --files確認moba_match/bots；應先盤點實際檔案，不重犯E187／190／191的猜路徑問題。
- 當前core兩項確認通過，正式60Hz結果詳見support-guard-focus進度。不做full suite／完整UE驗收，E177保持。
- 真實首次60Hz失敗：fixture把上限寫200，authority依正式Lua容量刷新為400，assert pool.maximum不符；current158805 raw完全相同。改fixture沿出生實際maximum只設current200，不修改容量生命週期或產品規則以迎合測試。
- 修正後正式60Hz單項成功：護衛敵hero受到正常generated lumen_bolt80傷害／cost45與再生、CD；未提交pose與hidden escort cache不供focus，普通AttackTarget同目標。core2＋base1，沒有full suite或新DLL部署。


## E191：友軍輸入不應另造敵我篩選；互斥 unit 效果應在生成前拒絕（2026-10-05）

- 檢查發現通用游標施法已有 unit 目標選取，client InputBridge 沒有敵隊限定；不新增英雄 ID 特例、第二份隊伍判定或 ABI 欄位。後端 heal_ally 仍權威判定同隊、存活、距離與成本。
- 內容缺口：同一 unit 目標可混 heal_ally 與 damage／slow_enemy，生成成功卻必然施法失敗。shared Rust validator 與固定 Lua registry 同步拒絕 conflicting unit target allegiance；兩種順序均拒絕，保留正常單隊效果。
- 工具失誤：再次猜 omfue/bridge/src/input.rs 與根 Cargo.toml，造成 os error 2。已改對 bridge/src 已知目錄搜尋及使用明確 crate manifest；不得把這些搜尋失敗當成產品缺陷或成功測試。
- 原 E177 引擎基線未解決；此輪不嘗試引擎編譯、不部署舊 native／DLL、不宣稱真人游標或 UE 操作通過。
- 修正確認：shared model單項1、client兩hero友軍secure input單項1、fixed Lua工具8/8成功；fixture泛化產生unused BTreeSet warning已移除並重跑同一client單項成功，root whitespace通過。沒有full suite／完整對局。


## E190：隊友不是「非敵人」；效果階段與新增 perception 欄位必須完整遷移（2026-10-05）

- 契約缺口：faction_of原stub永遠None，不能用!is_enemy辨識隊友。實作原ABI方法的opaque combat-team identity，unique team一次字串；heal_ally同隊／活體／range／scalar全preflight，最後才emit治療，無ABI layout變更。
- 真實編譯失敗：泛match錨點放錯resolve／commit與資源准入迴圈，E0425／E0308／E0004；改完整階段錨點、治療移到資源全部准入後、補enum arms。SeenUnit maxHP漏sustain／items fixtures再E0063，搜尋全bots目錄補齊。修正後core1／base2與舊generic8成功。
- 重犯工具錯誤：猜dispatch／components／test檔路徑、Windows *.rs路徑、生成CPP直接層，回os error2／123。E187已記錄，仍應先rg --files確認，再對已知目錄rg，不在路徑參數寫glob或加fallback。
- 通用修正：Lua追加Support配裝variant與heal skill不改舊ID；多hero共享skill只註冊一次，implementation衝突拒絕。固定Lua include工具7成功、codegen生成／check與九Bot60Hzprepare成功。
- 內容界線：新catalog identity ff3ef5e2957aa89f／data9032cb2708e25e66，不混舊DLL；未build／stage／啟動UE，20/30与E177保持。詳見support-ally-heal進度。

## E189：Jungle 支援要共用目標且不能用自己充隊友（2026-10-05）

- 缺口：原Jungle普攻與技能只選野怪；若只改普攻，施法仍會優先打野。改共用局部assist focus，unit／point／area與普攻都限定原野怪或已披露且存活的focus，其他role不受影響。
- 決策：附近至少另一位已披露活隊友，owner_player_id排除本人；只按可見人數不劣勢支援，不把「看不見」當「沒有敵人」或安全保證。stable HP／distance／canonical ID決勝，不讀隱藏state。
- 熱路徑：一次perception掃描取局部hero子集，後續不重掃大量creep；正式最多10英雄。隱藏隊友cache不能提供援軍；未提交authority Pos不影響策略。
- 防錯：新fixture先取得Bot input再mutable step，預防同一參數列借用重疊；沒有改Dispatcher／admission時序。新功能没有編譯或測試失敗，不虛構錯誤紀錄。
- 確認：core新2／正常generated manifest正式60Hz base新1、既有role策略6成功，未跑full suite／100場／UE。完整5.5与20/30保持，詳見jungle-local-assist進度。

## E188：持續回魔要算邊際收益與容量，Buff 身分只能有一份契約（2026-10-05）

- 問題：只算新rate×duration會把原本自然回魔重複當技能收益；來源已存在時再算一次會把refresh當疊加；容量補滿也可能讓花魔沒有收益。
- 決策：self_mana_regeneration只處理新的additive無family flat regen；以共用UnitStats取得before／after rate，比較capped期末balance、扣真實cost。HP支援原治療，Mana分支同來源仍在store則不放；remaining零但未移除仍按現有聚合視為存在。
- 通用解：純generic_mana_buff_id放script-abi、脚本與Bot共用原格式與生命generation；不增加FFI欄位／runtime-heavy依賴、不新增英雄分支。Lua-only配方接線，現有平衡數字不改。
- 限制：預估採目前modifier／容量，未來death／dispel／modifier到期可改變收益，不能預發資源或把未來增益當現在成本。作者rate／duration提示不代表任意多段脚本完整預演。
- 編譯警告：正式Bot改用resources planner後budget wrapper只有測試呼叫，新增dead_code warning；改cfg(test)，不全域allow或新增fallback。功能編譯未失敗。
- 確認：core新2／base正式60Hz新1與九Bot prepare成功；ABI身分1／擴充plan gate1也通過，共5個不同的直接相關功能案例。最後production編譯沒有新增wrapper警告，三repo whitespace成功；完整UE／LAN／100場仍待，20/30保持。詳見bot-mana-regeneration進度。

## E187：低魔不等於該放回魔技能；Windows rg 不要猜檔名 glob（2026-10-05）

- 問題：lumen_touch成本55／恢復20，遊俠Buff也不是即時回魔；「低魔就放」會越放越缺魔，容量Buff更不能冒充恢復。沒有修改技能平衡來掩蓋問題。
- 決策：通用self_recovery保留HP OR本人Mana條件，Mana分支需作者即時restore extras大於權威折扣後成本，非零容量／存活／可付費／無CD；沒有完整額外支出效果預演時，不宣稱此提示能推導任意腳本淨收益。原回城安全優先不變，僅opt-in配方使用。
- 工具錯誤：兩次把ability_runtime*／scripts/moba*直接當rg路徑，Windows回os error123。應查已知目錄，必要時用rg --files或--glob，不重試猜出的路徑。沒有因此修改任何工作流或加shell fallback。
- 工作樹：觀察到其他操作已將本輪部分實作纳入新的HEAD；不reset、不覆蓋或撤銷，保留目前完整內容與新增案例，記錄以實際功能結果為準。
- 確認：core2／base正式60Hz1成功，固定Lua單人九Botprepare成功；不跑full suite／UE／100場，20/30保持。详見bot-mana-recovery進度。

## E186：實際原型接線要看生成 handler 與 full data hash；raw 期望型別別混用（2026-10-05）

- 決定：先鋒整備＋temporary capacity、遊俠包紮＋自然regen、共享術士回春＋立即Mana，全部Lua effects／extras，不改技能ID／原cost／HP或角色程式。三原型正式PlayerInput→正常生成manifest→outcome／finish確認，不手換handler。
- 編譯錯誤：成本通知Vec<i64>與期望Vec<i32>比較E0277；明確i64::from(cost)*1024後新正式case成功。世界單位與Q10 raw需要同時核對scale與寬度。
- 規則影響：遊俠rank1回魔現在5＋2=7；既有Bot預算期望改讀UnitStats並assert7，不讓stale固定5被誤認runtime錯誤。仍只走正式input。
- 工具問題：rg單行launch-plan輸出截斷，改唯讀JSON欄位；猜tick_hz缺值不當成功證據，60Hz以prepare真實回報。content_hash不變而catalog_data_hash更新14ef1dea84790afb，不能混舊DLL冒充stage。
- 確認：新三原型case1／Bot相關1、codegen生成＋check、固定Lua單人九Bot60Hz prepare成功；沒有100場／DLL載入／UE驗收，20/30與E177保持。詳見mana-archetype-content進度。

## E185：Lua Buff 必須型別化與固定刷新身分；測試別猜 CD 欄位（2026-10-05）

- 決定：mana_buff_self用shared七stat白名單、signed value／正duration envelope、每技能unique stat；固定Lua生成typed EffectOp，generic preflight後deferred add_stat_buff，ROk才提交。Buff ID含ability／stat／entity／generation，同來源max duration刷新不增層，不同技能疊加。
- 編譯錯誤：fixture猜hero.cooldowns造成E0609；查實際ability_cooldowns後改公開start_cooldown目標技能ZERO。最終新model1／base2成功，不為測試擴大API。
- 證據界線：原fixture mana_enabled=false只能證明payload，改正式初始化明確true與隔離回魔後確認90－45=45、容量280→335不補滿、重施不再加55、失敗留舊Buff正常倒數。不用後置切flag冒充正常啟用。
- 確認：新Rust3／Lua34（其中新21）與codegen --check通過；既有Mana2相關案例曾成功。沒有完整新英雄DLL／UE驗收，20/30與E177維持。詳見lua-mana-buff-declarations進度。

## E184：容量 Buff 必須在施法前同步；只看結尾不越界不夠（2026-10-05）

- 決定：UnitStats原始Lua capacity＋checked ManaBonus／ExtraManaBonus，合法負值截0，非法／overflow／超百萬退base；ManaPool增容量不補滿、縮小截限並清滿池餘數。
- 邊界風險：只在finish同步會讓已到期Buff的超額魔力被當輪cast消費。共用dispatch建立cache前同步，效果outcome後finish再同步；同hook新Buff維持deferred可見性。只處理active mana match living slots，不創建None池／TD不變。
- 操作／編輯問題：猜native/mod.rs搜尋失敗，rg找到native.rs re-export；helper插入暫移接了finish的commit comment，diff檢查後修回。
- 確認：新core1／正式60Hz base1與既有lifecycle3共5 passed，沒有compile／test failure；Lua持續Buff與完整框架仍未完成。詳見mana-capacity-buffs進度，20/30及E177狀態不變。

## E183：有回魔 helper 不等於已接線；負倍率與固定數值溢位（2026-10-05）

- 發現：UnitStats.mana_regen已存在，但MOBA finish仍用固定Lua rate。已接自然Buff與独立基地加成，合併一次Q10 remainder，保留有效時間／死亡／暫停等既有gate。
- 數值風險：原式只截最後结果，兩個負倍率可變成正回魔；fixed加法／abs／乘法可溢位。新增checked_sum_add以i128與unsigned_abs保留family聚合，checked_mana_regen逐factor截0與checked產品；非法自然rate fail closed但不禁用合法基地恢復，沒有全面改其他stat。
- 操作／編輯問題：buff_tick輸出被小cap截斷，改以正式到期與pool確認而不猜；新增測試把舊移速comment暫留到Mana測試前，diff檢查後修正。
- 確認：新core2／正式60Hz base1與既有sustain2共5 passed，沒有編譯或測試失敗；非Lua持續Buff作者流程／上限Buff／完整UE驗收。詳見mana-buff-recovery進度，20/30維持，E177未解除。

## E182：宣告式資源效果依賴 host 交易；外部 build-dependency 不要混選測試（2026-10-05）

- 決定：共享schema／固定Lua生成器／generic handler支援restore_mana_self與spend_mana_self。全plan preflight後按實際池與作者順序准入資源，再emit其他效果；額外成本不足由host回滾，不能用猜容量或把helper當成自帶rollback。魔力不是HP，不偽造Heal preview。
- Cargo錯誤：scripts workspace同選外部僅build-dependency的omoba-content-model，Rust1.95 Cargo resolver在NormalOrDev features panic；拆成各自manifest後新model1與base2成功，不動工具鏈／lockfile。
- 編譯錯誤：漏補既有ResolvedEffect fixture exhaustive match造成E0004；補兩個明確variant分支後成功。新增enum必須檢查production與test matches，不用wildcard掩蓋。
- 操作錯誤：一次context-only patch沒有改動；改具名test插入並檢查實際執行。literal buff*／猜unit_stats路徑搜尋失敗，改rg --files定位，不改未讀懂的Buff架構。
- 確認：9個不同Rust tests與Lua13案例passed、UE codegen --check沒有diff；不是新Lua英雄端到端或完整UE驗收。20/30保持，詳見declarative-mana-effects進度，E177未解除。

## E181：腳本魔力要共用帳本；tick fixture 必須實際註冊（2026-10-05）

- 決定：host metadata 扣一次，腳本讀 post-cost 餘額並可額外 spend／restore；serial event＋managed tick 共用 ledger、成功才提交狀態與通知，非 managed TD 保留平行。移除舊 cast view；ABI 只補語義文件，不增加方法。
- 實際失敗：兩項正式60Hz測試得到55／20而非54／19，因 fixture 嘗試替換 manifest 不存在的英雄 unit；只補 tag 仍不執行。檢查 units() 後明確 append UnitDef＋tag，最後3/3通過。不修改預期或正式英雄生命週期迎合測試。
- 邊界：legacy caster 也可能修改 managed recipient，不能只用施法者 reservation 決定 rollback；失敗 cast 有 dirty pool 也丟棄交易，新跨角色案例證明 recipient／通知／CD皆不變。
- 編輯／操作錯誤：過寬 patch 將 mana view 暫插到 buff remaining，編譯前移正；再次猜檔名與 literal glob 路徑失敗、多檔輸出截斷。使用 rg 定位、獨特上下文與逐段讀取，shell最後exit0不代表前面的搜尋成功。
- 確認：core mana篩選19／base新交易3／既有cast2通過；沒有全套驗收、UE或100場，20/30維持，E177未解除。詳見script-mana-transactions進度檔。

## E180：補魔策略必須有正式恢復與明確規則；feature 測試零項不是成功（2026-10-05）

- 決策：sustain 可選mana門檻＋配方明確mana_enabled，server旗標必須一致；None／零容量不等候。基地速率來自Lua，權威有效時間、活本人基地、存活／非lethal才加成；合併自然與基地速率後一次Q10餘數結算，AI只正式Recall／MoveTo。
- 設定風險：若只補AI門檻而没有基地資源恢復，會長時間等待自然回魔；若server覆寫配方flag而不驗一致性，會產生已宣告策略卻沒有法力池。兩者已用共用正式恢復與啟用衝突拒絕修正，未修改一般啟動預設。
- 編輯錯誤：插入server測試時提交空update hunk，apply_patch整份拒絕；改用實際完整函式上下文插入，沒有部分落地。
- 操作錯誤：多來源／OpenSpec輸出再次超過外層輸出上限，改分段補讀。猜 moba_role_config／moba_role_match_config／filtered_match_tests／plan.json 路徑失敗；以rg --files與現有launcher定位，實際產物為launch-plan.json。不得因錯誤在非最後一個shell指令而exit0就忽略。
- 測試錯誤：template預設未啟用runtime-lua-content，初次指定測試回報0；直接給workspace外package features又被Cargo拒絕。最後透過workspace內omobab/runtime-lua-content啟用依賴，再確認template新測試實際1/1，不拿零項或server成功代替。
- 確認：本功能新core2／正式60Hz base2／template1／server1與舊HP功能1共7個不同測試通過；固定Lua新一真人九Bot60Hz prepare與codegen生成／check通過。沒有全套驗收、stage或Unreal執行，詳見bot-mana-sustain進度。

## E179：Bot 預算必須讀正確 metadata 資源；fixture 不得猜成本（2026-10-05）

- 實作決定：共用 checked_mana_cost 給 Bot／authority，原始 f32→Q10與倍率進位完全相同；AI只有本人讀取權，不預扣，跳過不夠／缺資料技能並保留後續候選。
- 實際接線錯誤：首次 Bot 讀 World 的 ScriptRegistry，純 planner 通過但正式60Hz沒有選到施法；SimulationDriver::from_world 已 remove 並持有 registry，診斷直接 fetch 證明不存在。改讀初始化直接 clone 同一 AbilityDef 的 AbilityRegistry，不複製執行器或放寬缺資料的 fail-closed。
- Fixture 錯誤：property(20,0) 同時設 hp／mhp=20，不符合自療門檻；改 mhp100／hp20。另誤認現有箭雨一級70，實際 Lua builder 全部一級45；正式案例使用合法 rank4成本60對比rank1成本45，不改遊戲資料迎合測試。
- 操作錯誤：多來源輸出超過 max_output_tokens 再次截斷；後續改具體片段。猜 parallel_adapter／state_initializer 檔名搜尋失敗，改 rg 目錄與 rg --files 定位 parallel_world_adapter／initialization。一次診斷 patch 命中相同舊assert，已立即還原；之後使用獨特 expected-pool 上下文。
- 防再犯：先追查資源生命週期與來源建立點，再選 ECS 查詢；讀實際 Lua rank 成本和 fixture helper 實作。診斷不能變成 production fallback，不以純函式通過代表正式接線成功。
- 確認：core2／正式60Hz base1／既有 managed施法base2 passed；完整框架、補魔策略、UE與100場未驗收，見 bot-mana-budget 進度。

## E178：Mana 協商必須先於註冊；停用對局不能洩漏新事件版本（2026-10-05）

- 發現：既有 finish hook 無論啟用與否都發 CommittedMana26／None，可能讓未宣告新能力的舊端收到未知事件。改為明確啟用或已存在 managed pool 才發；短60Hz兩種配置確認停用0／啟用2 pools每tick。
- 決定：獨立 version1＋full data hash；server opt-in 預設false、secure MOBA限定，註冊前驗證加入者支援與 selective-player條件，再於 initial／rejoin bootstrap回覆。新client宣告能力不代表server開啟；無共享引擎修改或直接啟用launcher。
- 操作錯誤：首次新增fact條件的多檔patch附带猜測不存在的測試函式，整份驗證拒絕；移除無關hunk後依實際來源重套。再次猜simulation_driver／observable／match_plan路徑；以rg --files定位native/simulation_driver與bots/plan，確認 result.facts後才寫回歸。
- 編譯界線：server仍使用checked-in proto fallback（protoc warning），本輪以core vendored protoc正式同步生成，server編譯成功。將來新增proto需先同步fallback，避免多workspace建置讀取舊fallback；不以zero tests或單獨core成功代表server成功。
- 確認：core1／server2／base1（正式60Hz）成功；root／omb／omfx whitespace通過，無測試或編譯失敗。沒有真實KCP、renderer重連、UE／LAN或全套驗收；E177引擎阻礙仍待。

## E177：Mana HUD 版本界線與共享引擎建置阻礙（2026-10-05）

- 實作：本人 disclosed Mana→HUD schema2→受驗 IPC→C ABI12→共用 UE payload／MP文字與法力條；legacy schema1 只允許未支援且數值零，None 不等於零池。新 Rust 測試2＋bridge3 passed（1capture ignored）。UHT 已成功，但 native C++ 未成功，不能冒充 UI 驗收。
- 編輯錯誤：多檔 patch 留空 update hunk，整份拒絕；移除空 hunk 後重套，檢查新測試確實存在。第一次runtime測試在新增test前執行，8tests只是原回歸；之後新Mana2tests單獨確認。
- 操作錯誤：再次猜 `build_ue.lua`／smoke／TargetDescriptor 路徑並傳 literal restart glob給rg；改 `rg --files` 查實際 `build_ue_moba.lua`／`ue_native_visual_smoke.lua`／Configuration/Descriptors。restart executable 相對路徑配錯 cwd、誤用不存在的 OmGame.uproject；實際 project 是 `omfue/om.uproject`，後續採絕對路徑。
- 程序錯誤：在 stop session 尚未完成時提前重跑build，被Editor gate再次拒絕；必須 await stop完成，再建置。MCP dirty content／maps皆空，但既有restart超時自動force terminate71024，回報force_terminated=true；不得宣稱graceful。未刪檔／修改未儲存資產。
- 真正阻礙：UBT `FailedDueToEngineChange`，引擎現有三個 Skeletal*來源修改造成Engine unity cache／link產物待重建。保留 `-NoEngineChanges`、不還原或建置使用者的共享引擎修改。bridge12已stage，舊native ABI11必須fail closed；完整native驗證需可用引擎基線。
- 防再犯：辨識 project scope與engine scope；不得藉舊DLL、移除安全旗標、只看UHT或禁用版本檢查，把功能宣稱成功。長UBT輸出要改讀具體log摘要，不能依截斷末端推論原因。
- 收尾：Editor91188重新啟動，HTTP MCP30000 ready；root／omfue whitespace檢查通過。不宣稱new native module／NativeManaHud／PIE成功，完整引擎基線問題仍待。

## E176：Mana 生命週期不能使用舊智力公式；測試不可跨私有邊界（2026-10-05）

- 決定：正式 opt-in 由 Lua base_mana／growth 取得上限，i128 與開局所有等級檢查；新生命滿池、升級不補滿、有效時間 post-combat 再生。共用 Lua rate 進 full data hash／compiled agreement／no hot reload，既有模式預設停用。
- 實際編輯問題：第一次多檔 patch 假設 Lua 欄位獨立一行，與實際 compact table 不符，整份 patch 驗證失敗；確認無部分落地後依實際行重套。編輯時另移除誤留的 duplicate doc comment 與多餘 dead branch，未帶入測試版本。
- 實際編譯問題：fixture 把 `Finished` 當 unit variant（缺 winner／tick），並直接讀 private `MobaHeroSlot.hero`。改正 struct variant，使用公開新生命身分／等級／滿池結果與 pause 不重生斷言，不為測試公開內部快照。
- 實際工具問題：對 workspace 外 template-id 直接 `--features runtime-lua-content` 被 Cargo 拒絕。依既有模式同選 base_content／template-id，使用 `--features base_content/runtime-lua-content`；不能把 package filter 執行零 tests 當成功。另誤讀不存在的根 Cargo.toml；先定位真實 workspace，不猜根 manifest。
- 輸出問題：多檔讀取再次超過 cap；需要精確內容時分段限行，不依截斷部分推論。
- 確認：base3／core1／template2 通過（含欄位型別與省略預設），雙隊短60Hz完整hash零repair；UE生成與--check、來源diff whitespace檢查通過。最後template篩選的伴隨base0 tests只算編譯；沒有一般網路啟用、buff再生、script restore、HUD或完整Unreal驗收。

## E175：Mana相同仍可能缺owner輸入；再生餘數不能漏投影（2026-10-05）

- 決策：append CommittedMana26／binary v1 disabled2或enabled20bytes；current／maximum／Q10再生餘數同傳，checked decode先驗再寫，沿既有Hero baseline與visibility gate，不傳hidden來源或canonical身份payload。零Mana不等於None。
- 實際失敗：新短60Hz filtered fixture team1 tick3 hash不符；加入逐component診斷再次重現，Mana三欄完全一致，只有owner ranger_patch cooldown {} 對權威20480。漏pending_accepted_inputs使owner未重演施法；project_tick本身不替SimulationDriver建立server accepted projection。
- 修正與防再犯：fixture沿正式 CanonicalAcceptedInput／normal PlayerInput接線，原ID、team1-only及其他隊零accepted數量明確断言，兩隊每tick完整hash通過。不得覆寫owner cooldown、只比Mana小雜湊或加入ComponentRepair來讓fixture通過。
- 操作問題：多檔合併輸出再度截斷；精確小段補讀producer、decoder及既有正常accepted-input fixture，不把截斷當全讀或猜hash原因。
- 確認：core4／base1（12ticks雙隊24steps）通過、無repair，保留mismatch診斷；不等於網路／UE或整場驗收。正式協商／規則啟用／再生／生命週期／完整script API與HUD仍待，不開一般對局Mana。

## E174：法力與冷卻都不能讀同 tick 舊 cache；0 tests 不是驗證（2026-10-05）

- 決定：managed SkillCast 以 serial queue 私有 ledger 預約候選 Hero、當級合法成本與既有倍率，成功才提交 Mana outcome與CD，失败丟棄該 invocation outcomes／overlay／visual。同tick不同招餘額、同招CD都讀更新後帳本，不用平行鎖搶先順序。既有未啟用模式保持相容。
- 防再犯：execute成功不代表unit pre-hook沒有panic；已捕捉的hook panic另記flag，managed cast不得提交。只保证延後adapter效果，不宣稱ABI abort／crash／外部I/O可回滾。host扣metadata一次，enabled explicit script spend明確拒絕，不沿旧stub虛報或雙扣；restore等仍待ordered接線。
- 操作誤判風險：初次 cargo test filter mana_cast 還沒有測試，只得到 running 0 tests；僅可記編譯成功。加入實際測試後base2與adapter1均執行通過，最後核心修改後base2再通過。
- 輸出問題：合併多檔限行仍超過內層token cap截斷；後續精確讀需要的段落，不把截斷當完整檢查。本輪沒有編譯或測試失敗，不虛構修復案例。
- 邊界：正常網路對局尚未啟用新池；Mana committed投影／hash／規則版本、再生／生命週期／HUD與完整script API仍待完成，不能因fixture60Hz成功開一般扣費或勾完整5.5。

## E173：最大法力不能冒充餘額；延後提交不可直接各自扣費（2026-10-05）

- 發現：adapter current_mana 回最大值、spend_mana 永遠成功、restore_mana 空實作；不宣稱新 pool 已讓既有腳本 API 生效。共用 event cache 與延後 outcomes 使同 tick 舊餘額有超支風險，平行 Mutex 的競速順序也不等於 deterministic gameplay。
- 決策：先實作具私有狀態／checked serde 的 Q10 ManaPool 與 Hero optional 保存欄位；成本不足／负值原子拒絕、恢復與容量邊界、i128 再生餘數。正常對局不先開扣費，下一階段必須完成 ordered commit／失敗不扣／雙扣排除與安全投影。
- 防再犯：不能逐英雄特殊扣費、只 authority 改卻漏 replica hash 或以 input ACK 當扣費成功；GameMode::Moba 也不能代表新規則 opt-in，舊 Story 同樣使用它。初始化必須冪等，不藉 repeated spawn/setup call 補滿既有池。
- 操作問題：長 OpenSpec 合併輸出仍截斷（含外層 tool aggregate token cap）；改為單次單檔限行補讀，不能以 nested command token cap 足夠就認定模型已收到全檔。
- 確認：核心 5/5、Hero 2/2 通過，沒有編譯或測試失敗；既有 td_rounds warnings 不相關。未做正常施法／60Hz對局／網路／Unreal驗收，完整Mana未完成。

## E172：減速必須走正式統計；測試須使用編譯數值與導航實際路點（2026-10-05）

- 決定：slow_enemy 共用 model／Lua FFI／Rust effects；全序列 preflight 後用既有 BuffStore 整數 Q10 move_speed_bonus、獨立來源 key 與 generic_ability_slow strongest-family，不使用 legacy float slow_factor，也不寫英雄特殊 UE 程式。家族只覆蓋新通用減速，不宣稱所有 legacy 慢速已統一。
- 實際測試錯誤：40% 編譯為 Q10 raw410，fixture 卻假定 floor409，導致速度預期不符。改讀正式 compiled rank extra，保留正式 round-nearest 行為，不為測試改 gameplay。
- 實際導航預期錯誤：先用單軸 speed×dt，再用最終目的地 normalized vector，都與實際位移不符；正式導航使用局部 MoveTarget，位移帶 y 分量。改讀同 tick 實際 waypoint、用同 Fixed64 normalized 計算精確向量；不加寬容差或跳過導航。
- 生成器檢視：以 extras key 作 Lua map 會在 reduction_key==duration_key 時覆蓋其中一個上界。改 ordered list 驗兩次，新增同 key 1.01 必須拒絕的回歸。
- 操作錯誤：猜不存在 native/ability_runtime.rs 與 docs/plans/implementation-errors.md；用 rg --files 確認實際模組與本紀錄檔。合併長輸出再度截斷，後續改限行小段；不把截斷輸出稱完整檢查。
- 確認：model1／base2（正式60Hz移動、strongest、TTL）／十二招1／Lua14 通過，UE正式生成及--check成功。未編譯／stage最新OmGame，未做網路／UE／100場驗收，完整5.5不勾選。

## E171：即時位移不可只寫終點；新增effect要補所有enum比對（2026-10-05）

- 發現：既有fire_dash直接set_pos到要求位置；本批不把該特殊handler當通用安全模板，也不改其既有語意。新增dash_to_point使用既有權威swept-terrain計算，要求整段可通且精確終點，生成／runtime皆拒絕混合效果；不宣稱沿路傷害、動態碰撞或持續衝刺。
- 實際編譯失敗E0004：新增EffectOp／ResolvedEffect::DashToPoint後，舊四個Luminary技能測試的兩個match未涵蓋新variant。補明確非該fixture效果的分支後base編譯與新功能測試成功；不得以萬用分支改掉正式runtime語意。
- 操作錯誤：先猜heroes/date/fire_dash.rs不存在，rg --files確認實際B02_date_masamune/No2_fire_dash.rs後才讀取。幾次合併source/status token cap截斷；正式修改依精確小段補讀，不把截斷稱完整檢查。先尋路徑、再讀已存在檔案。
- 決策與防再犯：advance_with_collision只計算位置、set_pos才提交；point超距／原地／受阻不部分移動或啟CD。Bot只看disclosed living target、實際rank range與明確策略，用正常CastAbility；舊vanguard_resolve的ID保留但target／語意變更必須重建與hash gate，不沿用舊SelfHeal策略。
- 確認：model1／base2含正式60Hz薄牆與Bot位移／core1／Lua7、新三原型十二招1通過；正式生成與--check、新一真人九Bot60Hz配方預檢成功。原dead-code warnings保留；最新OmGame／DLL stage、filtered／UE全套未驗收。

## E170：有限非負不等於可執行數值；Lua double 與 Rust f32 需同值驗證（2026-10-05）

- 問題：rank資料原只驗數量／解鎖門檻，負cooldown或非有限成本可進入生成；效果量缺上界，小於Q10的正值可能變零。共同model與Lua FFI補零或有限[1/1024,1000000]；range／radius仍維持原較小限制，非法內容拒絕而非clamp。
- 決定：這是內容安全範圍，不是完整Mana功能、平衡門檻或全部組合運算安全證明。特殊Rust handler也驗共用rank欄位，無關extras保留；deleted tombstone可不具執行資料。
- 生成器差異：直接比較Lua double會讓1000000.01／10000.0001等邊界只在FFI拒絕，Rust f32卻合法。Lua標準pack／unpack先转相同f32，並加兩邊邊界案例；不藉此修改正式Lua數值或hash。
- 操作錯誤：又猜不存在 native/component/hero.rs，實際是 native/comp/hero.rs；已用rg --files確認後讀取。合併長文件輸出也截斷，已用明確行數補讀OpenSpec design與tasks；後續先找實際路徑、控制單次輸出，不把截斷當完整閱讀。
- 確認：新model2、Lua numeric34、原include5、正式24項FFI IDs與base cast_preflight2（含60Hz）成功；UE生成--check內容hash不變。既有td_rounds dead-code warnings未改，不冒充Mana／盟友／網路／完整Unreal驗收。

## E169：Bot 篩選距離不能取代權威執行器；單體與範圍距離不可混用（2026-10-05）

- 發現：Generic Damage preflight 原只驗 alive／enemy，沒有依目前 rank 的 range 檢查 caster／victim 位置；HealSelf 原接受任意 Target，且純執行器沒有統一 caster 存活 gate。不能因正常 Bot 會篩選距離而宣稱非法玩家施法也被攔住。
- 決定：shared model＋固定 Lua 生成器要求有目標效果的合法可量化 per-rank range。單體 resolved Damage 帶 Some(range)，area 展開帶 None；共用全序列 preflight 先驗有效／正 HP caster，再驗單體兩端位置／精確距離，SelfHeal 強制 None。全部驗完才輸出效果，失敗沿原 dispatch 不啟 CD，不使用前端或 Bot 修補。
- 重要邊界：AoE 限制中心而非每個 victim；更新正式 fixture 的 x790 victim（cast700／center590／radius220）確認仍受傷，x1000不受傷。不能重用「所有傷害 victim 都須在 cast range內」造成範圍技能回退。
- 相依決策：GameWorld::faction_of 仍未實作，不能以「非敵人」推成「盟友」，本批未擅自新增友軍治療或 ABI 方法。输入 admission／queued 不是效果生效；未宣稱拒絕效果會還原 input handler 清掉的命令。
- 操作錯誤：再次猜 omb/src/input_buffer.rs，實際是 lockstep/input_buffer.rs；已先用 rg --files 列實際路徑。合併來源／git status 輸出也截斷，必須縮小每次輸出、以明確行數補讀，而不是將工具呼叫當作已讀完。
- 防再犯與確認：新 base cast_preflight2＋model1、更新正式AoE1，邊界／一 raw 超界／死 caster／錯 heal target／rank資料與 HP-CD 全數通過；同批原 generic3與固定Lua include5通過，UE生成--check通過。沒有 OmGame／stage／網路／全套驗收。

## E168：單體名稱不等於 AoE；群聚中心會改變測試範圍（2026-10-05）

- 發現與決定：三原型只用 Damage／HealSelf，即使名為 volley 仍只有單體。新增通用 area_damage point／per-rank amount-radius 與共用 model／Lua FFI 驗證、Rust host query／穩定去重／全序列 preflight；不加角色分支、不修改 GameWorld ABI。radius／range 必須至少 1/1024，不能接受量化後為 0 的「正數」。
- 實際測試失敗：正式五人 fixture 將 player5 放在 x900，誤認必在範圍外。Bot 正確選 x690，涵蓋三名敵人；900−690=210，小於 radius220，因此 player5 HP1000→890，測試預期1000錯誤。修 fixture 為 x1000，保留最大覆蓋與正式傷害；不得為通過測試降低 Bot 策略或忽略第三名受害者。
- 複雜度決定：不得對所有披露單位執行無界平方級群聚。先穩定排序最近512可用單位，最多32候選中心，最多16384次覆蓋檢查；超額 gameplay resolved effects128則全招拒絕，不任意丟棄傷害。這是工作量限制，不是穩定60FPS或全圖最優聲明。
- 操作錯誤：PowerShell 下把帶 *.rs 的路徑直接交 rg 導致 OS error123，應傳實際目錄搭配 -g '*.rs'；也猜不存在 native/systems/player_input.rs／native/systems 與 OmGenerated/manifest.json，再次造成 error2／3。已用 rg --files 確認 game_processor.rs、scripting/dispatch.rs 與 om_codegen_manifest.json。不得以檔名印象取代實際清單。
- 輸出截斷：合併 context 的大輸出與單檔 token cap 截斷。已以 bounded line ranges 補讀缺段；後續不要把一次 Get-Content 稱為已讀完，必须檢查工具是否截斷。
- 當前確認：model1、base area2、core area1、原三英雄十二招1，共五项指定測試成功；正式 codegen 與 Lua 一真人九Bot60Hz preflight成功。新 generated metadata 仍須最後 OmGame／DLL stage／網路及UE整合，沒有把預檢當畫面或100場通過。

## E167：購裝游標不等於終局背包；正式交易規則不可在 Bot 重寫（2026-10-05）

- 問題與決定：若固定依「劍、劍、大劍」游標或看是否還有材料，合成後容易重新買材料，死亡也容易丟失階段。改宣告終局物品 multiset，先保留所有完成目標、遞迴推導第一個缺項；只看 owner 背包／金錢與公開 catalog。沒有 Bot 金錢或 Inventory mutation。
- 問題與決定：只看空格會錯拒滿格合成，另寫補差額也可能忽略重複材料。read-only preview 直接複製資料呼叫正式 buy_item，authority 仍重新驗身份／存活／範圍。六格與 goal 順序造成的無法組裝，正式 plan compile 有界預檢拒絕，不留靜默卡住配方。
- 實際編譯錯誤 E0425：moba_match 父層未匯入 ItemRegistry，新增 Bot resource lookup 後編譯失敗。已明確匯入 native::item::ItemRegistry，並用 try_fetch 讓缺少 resource 保守不購買，不破壞省略策略的既有行為。
- 操作錯誤：多檔 patch 中重複的 think_interval_ticks 行有不同後綴，預期 context 不符，整個 patch 拒絕且未修改。先讀精確 constructor 行，再以完整唯一 context 重做；不可猜測部分 patch 已成功。多檔讀取輸出也再次截斷，之後應分檔限行，不把截斷當完整閱讀。
- 防再犯：核心測試涵蓋重複／二階材料、完成不重買、保留終局目標、滿格合成、餘額／容量／循環、非法配置及 preflight 死局；正式 60Hz 測試涵蓋只送 ItemBuy、非直接修改、精確扣款、對手不變與 phase／死亡／範圍 gate。補 return_to_shop 的金錢門檻、披露威脅／中立、共用 threat helper；正式 Recall→完整 channel→home→大劍購裝，完成後不因金錢回城。最終指定 core 4＋base 2 成功，Lua 一真人九 Bot 新配置預檢成功；不是完整網路／Unreal／100 場驗收。

## E166：回城傳送不等於續航；恢復不可復活死亡英雄（2026-10-05）

- 發現：既有 Recall 只傳送，不恢復HP。若僅新增低血回城，Bot會在基地反覆Recall。決定新增可選權威基地恢復，Lua compiled rate／radius、Playing active delta、己方存活基地／存活英雄／非lethal生命／距離檢查；在post-combat及recall完成後、final EquipmentStats投影前結算。Bot不改HP或位置。
- 決定：配方sustain嚴格門檻與安全距離，當隊committed披露威脅→正式MoveTo撤退，無附近威脅→正式Recall；讀條不打斷，基地未恢復至離開門檻不送推線。省略policy保持既有行為，Policy啟用match的恢復旗標，真人也共用相同恢復規則。
- 操作錯誤：本輪仍猜不存在的events/outcome*、runtime/simulation_driver.rs、runtime/driver.rs、native/mod.rs、runtime.rs及game_proto*；已改用rg列出實際native/simulation_driver.rs／game_processor.rs／generated/game.rs。不要以既有檔名印象代替實際清單。
- 測試錯誤：先在base_content用只有core才有的篩選得到0tests，僅屬編譯證據；之後分別執行真正core決策與base ECS測試。ECS先存取private routes造成E0616，改由公開bases與Pos核對實際傳送目標；CProperty不是Copy，E0507修為只複製HP／maxHP欄位，不新增公開介面或Clone熱路徑。
- 大型範圍組合讀取仍有截斷，後續補讀必要區段；不能把被截斷的歷史或程式碼当作完整證據。
- Cargo指定scripts workspace之外的依賴package再加該package的--features被拒絕；改以workspace內base_content/runtime-lua-content啟用相依feature並選兩package，只計真正執行的template-id測試，不計base篩選0tests。
- 最後功能確認：core2/2、base3/3（含雙隊20ticks／40steps完整hash零repair）、template-id恢復規則1/1、新混合Lua配方正式Rust預檢成功；root／omb／omfue空白檢查通過。恢復資料改變的是完整catalog_data_hash，不要求僅呈現model的generated content_hash也改變。未stage或做完整Unreal對局，詳見bot-sustain-base-recovery進度檔。

## E165：共用 Lua 模板 include 必須涵蓋 FFI 產生器與建置相依（2026-10-05）

- 首次原型 ID 測試編譯失敗：猜測 HeroId 有 as_u16，實際公開方法為 raw。讀取生成器方法後改用 raw，不新增多餘轉換 API。
- 正式 base_content 建置失敗：gen_hero_registry.lua 原本只呼叫 builder({})，新增共用 Lua 模板後 ctx.include=nil；Rust／Unreal 完整內容 loader 能 include，不代表 FFI 產生器也能。
- 決定：FFI 產生器提供一般化 content-relative include，拒絕絕對路徑／父目錄逃逸／循環，所有 builder 共用 context；base_content build.rs 監看完整 templates 目錄，避免只改新 include 檔卻留舊註冊。保留 append-only 英雄／技能顺序，不複製共用模板或新增角色專屬 handler。
- 本輪大型歷史 context／來源搜尋輸出再次截斷；改分段補讀缺少部分。後續搜尋需限定來源目錄與輸出範圍，不能讓資產清單或合併多份長檔耗盡預算。
- 原型目前使用真正已支援的敵人單體傷害與自身治療，遠程連射不是 AoE，前排恢復不是護盾；mana 為內容 metadata，未支援消耗前不得宣稱有 mana 規則。完整三原型平衡／美術／100場留最後，不以新增名稱封關5.5。
- 修正後：附加ID與loadout指定1/1、三英雄十二次正式60Hz施法指定1/1、include正負向5/5、新混合配方正式Rust prepare-only成功；codegen隔離生成／check與正式來源生成成功。未編譯OmGame／stage新DLL，不將舊產物視為新內容證據，詳見moba-archetypes進度檔。

## E164：完整 TOML 不可用簡易讀取器重寫；監聽就緒不等於真人入局（2026-10-05）

- 發現：既有 Lua scalar reader 不支援完整 TOML 結構，拿來合併會破壞 inline table／array／multiline。對真人 AUTH 做遞迴合併還會殘留原玩家；換輸出目錄後相對 content path 也會失效。
- 決定：固定 lua-host 新增完整 TOML section-field replacement；指定欄位整個取代，其他值保留，所有 content path 絕對化，不修改來源。設定预檢由正式 Rust Setting／role compiler 執行，不載 DLL／World／socket。
- 發現：runtime presentation listener 在 KCP admission／replica 就緒之前輸出；同隊玩家共用以 team 算的 IPC 埠會衝突。
- 決定：只接受精確 player_id／team_id ready 行；IPC 埠依真人順序分配，Bot 不啟 client。Unreal 明確 presentation-only；準備或程序啟動都不宣稱 renderer／60FPS／完整對局通過。
- 決定：每次 spawn 立即存 own PID manifest；cleanup 身分檢查、反向 stop＋wait，錯誤不能被 cleanup_stack 的 pcall 吞掉。失敗記在 run/errors.md 並傳出；mock stop failure 是預期測試，不是真實 PID 清理故障。
- 本輪兩次組合讀取輸出仍超過預算而截斷；後續以窄範圍重讀必要欄位，不能把截斷視為已閱讀全檔。舊 E163 的閱讀習慣提醒仍有效。
- omb Cargo.toml 的 index／worktree 原本皆 mixed 換行，新增bin段落CRLF導致預設diff --check報cr尾隨空白；保留原文件，不大範圍改換行或autocrlf，以core.whitespace=cr-at-eol檢查真正空白，root／omb均exit0。
- 當前功能：host TOML 2/2、固定 Lua 配方／正式 Rust 預檢／readiness／mock lifecycle 5/5 passed；一真人九Bot60Hz prepare-only 成功，沒有實際 Unreal 對局验收。

## E163：Bot 對局名冊不是 KCP 真人認證；正式 tick fixture 必須保留接收端（2026-10-05）

- Bot roster不能塞進AUTHENTICATED_TEAM_BINDINGS補canonical projection。新增server-owned控制名冊，真人認證必須等於配方bot=false玩家；外部輸入先排除Bot／未知ID，內部Bot再經既有MobaMatch gate／canonical accepted projection／PendingPlayerInputs／phase dispatcher。KCP仍只授權真人。
- 第一次正式State::tick測試失敗「secure team outbound queue disconnected」：fixture helper把出站receiver丟棄，正式可靠隊伍發送正確回報斷線。修正helper回傳receiver並由測試保留；不得關閉可靠send gate、忽略錯誤或只改成SimulationDriver測試假裝正式server成功。最後正式server相關3/3 passed，安全兩隊輸入frame與九Bot實際位移亦驗證。
- 第二次fixture預期11輸入但只有2真人輸入：正式server的Wave B用一tick披露延遲，不同於舊headless delay0；第一tick缺owner disclosure，Bot正確fail closed。改先正常跑兩tick建立正式披露，不把delay改0或取消owner gate。
- 第三次已正常披露但只有9而非11：兩個Support初始在Carry跟隨半徑內、無在途命令，正確hold不發輸入。改驗精確controller ID序列（2真人命令＋7Bot命令），而非要求每位Bot每tick必發命令；後續正式60tick仍要求九Bot皆有實際移動。沒有修改AI政策迎合測試。
- 一真人Lua配方測試首次失敗：toml::Value直接try_into對字串map key "1" 拒絕u32（十Bot空認證表沒有暴露）。改以正式read_setting同路徑的TOML文字序列化→toml::from_str核對，不以十Bot配置掩蓋真人入口。
- 最後指定Lua配方測試1/1 passed，十Bot與一真人九Bot皆由固定Lua匯出／正式TOML讀入並驗證，並未修改正式AUTH型別。主server binary check exit0；無DLL stage／實際socket或Unreal完整驗收，詳見role-bot-server-control進度檔。
- 嘗試以core.autocrlf=false抑制diff warnings，反使既有CRLF module檔的CR被diff --check當成全檔行尾空白。恢復正常換行轉換並只設core.safecrlf=false後check exit0；沒有為了checker重寫全檔行尾。
- 初次role_server篩選未寫測試時0 tests，只算編譯確認，不能宣稱功能通過。一個running編譯呼叫只印output漏session ID；之後保留完整工具回傳，避免失去poll handle。
- 一次合併OpenSpec讀取與多次來源合併輸出截斷，後拆小段補讀；又猜scene/import_map.rs、runtime/team_projection.rs而不存在，已用rg --files／實際識別字定位。不得從猜測路徑或截斷輸出推定完成。

## E162：技能學習不能繞過權威扣點；Bot roster不等於真人認證（2026-10-05）

- 新學習政策只提出正式UpgradeAbility，owner點數／rank／等級門檻由原handler重新驗證與扣點／SkillLearn。僅有rank0施法策略會永遠跳過未學技能；採獨立有序學習步驟，等級blocked不阻塞其他合法步驟。異常rank以checked_add防溢位，不能直接改Hero假裝學習成功。
- 查核正式server canonical accepted input team取自AUTHENTICATED_TEAM_BINDINGS，而main同表提供KCP authorization。未來不能把Bot加進真人auth表來補projection，須分開server-owned match/controller roster與外部認證身分；本批沒有改這些邊界或宣稱KCP Bot已完成。
- PowerShell命令帶`{tasks,design}.md`觸發ParserError/Missing argument，整個讀取沒有執行；修為兩個明確路徑再讀。不得在PowerShell假設bash brace expansion。
- 合併輸出多次被max tokens截斷，已用明確小段另查upgrade handler／server tick／canonical accepted input；來源仍應縮小讀取範圍，不把截斷讀取當作完整證據。
- 本批core學習2/2、plan2/2、正式60Hz ECS1/1與固定Lua配置入口均成功，沒有功能測試失敗；不是完整框架／Unreal／LAN／100場驗收，詳見role-bot-learning進度檔。

## E161：tooltip preview不是AI執行語義；HP總差額不是單一技能傷害（2026-10-05）

- 已查明effects_preview僅預覽，改以Lua配方explicit intent與compiled target/rank/range建立通用政策，不以preview／slot猜用途。
- core測試首次E0382：CProperty非Copy，low=health後再次借用health；改測試clone後2/2通過。
- ECS期望80卻得169.121，移除TAttack後仍失敗，證明非一般attack元件能隔離。正式MOBA塔攻擊有私有NPC state，原fixture仍在650塔距離內；還原元件、改公開位置fixture離塔／camp範圍，最後80傷害與70自療精確通過。不得調低assert或改實際技能平衡來掩蓋混入來源。
- 又猜ability_runtime.rs不存在；後以rg --files確認為ability_runtime/registry.rs。來源路徑先列實際清單。部分合併輸出截斷後另依明確技能cast／NPC spawn實作定位，不能以截斷內容確認完整實作。
- 本批只有core2／正式ECS1／Lua配置入口確認；Mana未實作完整current值，不宣稱資源驗證、任意handler語義或完整對局已通過。

## E160：MoveTo admission 不代表該 tick 即時停止（2026-10-05）

- 新Support停止追逐測試首次失敗：提交到committed自身位置後，位置仍多移動約5.1units。原因是共用Dispatcher先執行既有移動，Moves才接收替換命令，不能拿headless ACK或接入當作即時停住。
- 修正Support對近距離在途MoveTo的去重，等待正式命令完成，不每tick追新pose替換造成來回修正；不改global phase、不直接清ECS queue。新測試驗admitted target精確／下一tick回到目標／不再重送，最後base相關3/3通過。
- 初稿曾引用queue.pending，讀實際HeroCommandQueue後確認欄位是queued，編譯前已修正；未捏造編譯失敗或測試通過。大段讀取一度截斷，時序結論依明確phase表及個別command實作，不採截斷文字推斷。
- 感知使用同baseline的public HP与render；缺健康資料不回讀敵方CProperty。Support配對來自完整role roster，不能只在Bot名單找Carry，否則漏真人。core相關8/8與最後ECS3/3通過，不代表完整Bot／網路驗收。

## E159：已列出入口後仍猜名稱／工具路徑（2026-10-05）

- 本批讀取 omfue/rust、scripts/moba_headless.lua、scripts/lib/common.lua 時遇到不存在路徑；其中已列出 run_moba_headless.lua 卻仍猜舊名稱。改以 scripts/_bootstrap.lua 取得 tools/lua/lib 的既有 JSON／path／process，不新增平台 fallback。
- 合併大量規劃讀取造成截斷，後續分段補讀；大型歷史 MD 應單檔、小段，不能以 command exit0 當全部內容已讀。
- 設計檢查發現單把 Bot 指令推入網路 ECS 會漏 controller ownership／正式 acceptance。決定先接共用配方與 headless 正式 driver；KCP 接線保留待辦，不以 headless 通過宣稱網路九 Bot 完成。
- 本批新 Rust 測試2/2與九 Bot ECS1/1、headless check與Lua配置入口通過；沒有新編譯或功能測試失敗。protoc fallback／td_rounds warnings 是既有建置診斷，不當作已修復。

## E158：MOBA-local key 不能當正式視野 canonical ID（2026-10-05）

- 第一版 Bot 整合測試取得0個輸入而非10個；舊 entity_key 是 id<<32|generation，Wave B canonical 是 generation<<32|id，兩者 namespace 不同。沒有放寬視野白名單，改用正式排列匹配自己／拆 target ID，並驗完整 generation。
- 修正後五位置10人60Hz正式 driver 功能測試1/1、core視野與身分3/3通過。新增 identity layout regression，不能只以純 planner 測試代替 ECS adapter。
- 本批又出現猜 native/mod.rs、native/visibility.rs 等不存在路徑，以及 PowerShell rg wildcard 路徑；已改用實際檔案與根目錄 -g，這些是重複操作失誤，不宣稱已永久杜絕。
- 數次合併讀取仍截斷；功能結論只採獨立小輸出的實際測試數／exit。本批不以截斷的整份 tasks 或文件內容當新的完整閱讀證據。

## E157：無內容 patch hunk 與重複猜 staging 入口（2026-10-05）

- server 設定 patch 兩次留下只有 context、沒有新增／刪除的空 hunk，被 apply_patch 拒絕，沒有部分修改；改成有實際替換／插入內容的 hunk 後成功。送出前應刪掉空 hunk，不重送相同錯誤。
- 猜 scripts/stage_base_content.lua 不存在；實際 build_ue_moba 呼叫 dev_run_freshness.lua --action stage-dll。應從呼叫者或rg --files -g '*.lua' 找入口，不因DLL名稱猜檔名。
- 本批 root scripts／UE 以現行debug DLL staging；omb/scripts 是不同舊副本，未被本build入口使用，不能拿三者混比宣稱全部host已更新。正式launch需沿既有配置指向當前scripts/base_content.dll，所有 peers同catalog重建；尚未做正式網路啟動。

## E156：feature-gated 測試零案例不是成功證據（2026-10-05）

- 新 Lua 層次驗證測試首次未帶 runtime-lua-content，cargo exit0但 running0；不計入通過。開feature後出現缺少 canonical_template_hash 匯入的編譯錯誤，補正確 omoba_content_model import，最後實際1/1通過。
- 同批 PowerShell 直接傳 omoba*／runtime_peer*.lua 作rg路徑被視為非法名稱；應用明確根目錄＋-g選擇檔案，或rg --files找入口，不能重複猜 shell glob。
- 多段輸出仍可能截斷，測試結論以實際命中的case／完成exit及獨立小輸出確認，不以總exit0或truncated當完整證据。

## E155：IPC 模式不能由位址有無推測（2026-10-05）

- 發現 PRESENTATION_IPC flag 雖指定，bridge 仍以空位址選路；若 legacy DLL／Story 完整，缺 endpoint 可能回落舊 gameplay。修正為保存 explicit intent，create 與 driver 雙 gate 缺位址拒絕，不建立 runtime／legacy threads。
- Unreal 同時新增明確 PresentationIpc 配置與 launcher flag；缺位址不借用 ServerAddress。測試須用有效 legacy inputs 證明不能 fallback，也以真實 localhost IPC 確認兩種 local mode 都 sim_thread=None。
- 盤點時猜錯 OmWorldBridgeActor.cpp 所在 module 與 runtime_driver.rs 檔名，rg 回報缺檔；改先 rg --files 定位 OmGenerated/Private 與 driver.rs。缺檔不代表該機制不存在，不能據此新增重複實作。

## E154：restart 相對 project 定位與 readiness 的證據界線（2026-10-05）

- Editor102680 第一次 wait-mcp45 秒逾時；日誌顯示 Engine Initialization 32.94秒、HTTP30000 已綁定，但初次 health I/O timeout 原因未確認，不能直接照 E149 稱為初始化超時。
- 再次檢查時誤在 monorepo 根目錄呼叫 restart，因預設相對 om.uproject 而 exit2。正確入口 cwd=omfue，或明確 --project 絕對路徑；修正 cwd 後同樣45秒 readiness立即成功，未重啟Editor或調高期限。
- 保留兩個失敗，不把 port bound 當 health成功，也不把 cwd錯誤當 MCP服務故障。

## E153：跨專案共用 Editor executable 的 Live Coding 鎖（2026-10-05）

- 本專案 Editor 101752 已退出，但建置 90267 exit 1／UBT OtherCompilationError：Unable to build while Live Coding is active。另一專案 PID96936 使用相同 UE5.8 executable，未停止該程序。
- 讀引擎 HotReload.cs 確认鎖由 executable 路徑命名，不包含 project；BuildConfiguration.cs 提供 -NoHotReloadFromIDE。
- restart 所有 offline build 先驗本專案 Editor 不存在，再帶 -NoHotReloadFromIDE 與 -NoEngineChanges，避免跨專案鎖阻擋，同時禁止覆寫共用引擎產物；不是刪鎖、停止其他專案或修改引擎。
- 保留首次失敗；只在修正後重建本功能，不重跑完整驗收。若引擎產物真的需要改動，應由 guard 明確拒絕，不移除保護繼續建置。

## E152：相容宣告不能繼續隱含專屬派發（2026-10-04）

- 舊實作把 Saika 分支移出通用模板後，仍由三個 generic hook 自動呼叫 legacy adapter，包含四個技能 ID 判斷與 payload fallback；僅確認模板內沒有英雄名稱不足以證明正常路徑通用化。
- 先 MCP 查唯一子 Blueprint 已只用通用事件，再刪除自動派發與轉換器，保留 reflected API 的顯式相容呼叫；測試必須同時證明通用欄位保留與舊回呼不觸發，不以「舊回呼收到」當正常成功條件。
- 本批讀取累積過大的 OpenSpec 文件時仍遇到工具輸出截斷；應按小段讀取並分別限制輸出，不把 truncated 當完整讀取或證據。歷史文件目前包含大量逐批紀錄，不能依總任務數猜完成狀態。

## E151：Lua reserved key 與既有工具 API 必須先確認（2026-10-04）

- 新 planner 初次使用 node.detail.function，Lua 的 function 是保留字，載入即語法錯誤；改為 detail['function']，無資產修改。往後 JSON 的 reserved key 一律用 bracket access。
- 備份 hash 一度寫成不存在的 sha256_file，查現有 hash.lua 後在 apply 前改為 sha256；不要從其他語言的 helper 名稱猜 API。
- 查參考腳本時猜了不存在的 ue_art_replacement_smoke.lua；改以 rg --files 找實際 scripts/tests/ue_art_swap_editor_test.lua。不得把缺檔讀取當已取得依據。

## E150：MCP ok=true 的 graph preview 仍可能含拒絕項目（2026-10-04）

- 首次 preview 六個刪除項目，MCP ok=true 但 evtSaikaAction 被拒絕：cannot delete a structural/entry-point node。初版工具只檢查外層 ok，錯誤回報 success=true；該次只有 preview，沒有修改資產。保留原 report1791123876 與 raw 回覆，不能當成功驗收。
- 增加逐項 gate：preview 全部 would_apply=true、數量一致；apply 必須 applied=true 且無 rejected，之後完整 snapshot 核對。負向 CAS／未套用回歸測試保留。
- 普通五個節點用 CAS，入口則先核對 isolated exact event，再由 documented delete_nodes allow_destructive=true 精確刪一個；備份／raw 操作／部分失敗報告保留，不自動重送整批。
- 最後真實遷移1791124011成功，保留25nodes完整內容／接線一致、compile/save成功；重跑無改動。13個直接測試與動畫派發單輪通過，詳見 Blueprint generic event migration 進度檔。

## E149：Editor 初始化時間不能與 MCP 等待期限混為程式錯誤（2026-10-04）

- Editor 101752 首次 wait-mcp 45 秒 exit 6；專案 om.log 顯示 Engine Initialization 實際 46.06 秒後完成，process 身分仍是本專案，沒有 crash 或 modal 證據。
- 初始化完成後再做一次相同有界 readiness 檢查即 HTTP30000 成功；GenericAnimationOverlay 一次1/1通過。沒有反覆重啟 Editor、調高完整驗收 deadline 或停其他程序。
- 防重犯：先讀本專案啟動階段與 process 狀態，區分尚在初始化、服務故障與功能失敗；保存首次失敗，不把二次成功改寫成首次成功。

## E148：stage SHA 一致不代表最後 source 已建置（2026-10-04）

- 本批 build-only 進行時追加 unknown hero fallback 修正；初次 bridge DLL 22:14:06 早於 projection.rs 最後修改 22:14:40，雖然 --verify-staged-only SHA 一致仍不能當最新實作證據。
- 決定：停止本專案新開的 Editor 47420，再以既有 Lua build-only 補增量建置；此後不再同時修改程式。原 Editor 83404 已先核對完整 project command line 再由 project-scoped restart 停止，沒有停止其他專案。
- 防重犯：最後程式修改完成才建置；staging 一致與來源建置成功分別核對。不能讓讀取 hash 的成功掩蓋建置／編輯競態。最後 build-only 27886 exit0、DLL 22:16:58 晚於最後 source 22:14:40，stage c31089…一致；詳見 generic-animation-overlay 進度檔。
- 另有工具輸出合併過大被截斷；改按檔案／範圍拆讀。截斷不能作完整日誌證據，建置以該工作 session 的 exit 與本功能 Editor 報告確認，不讀其他專案可能覆寫的全域 UBT log。

## E147：非零 overlay 不等於 Saika 狙擊動畫（2026-10-04）

- bridge 寫死兩個 buff 名稱，Unreal 又將全部非零 overlay 指向 sniper_mode／sniper_walk；新 buff 即使有 metadata 仍會被誤解。
- 共用 Lua metadata parser → bridge priority／stable ID 選擇 → 生成 native OverlayName／Walk／Stand 欄位 → 共用 Unreal model。實際 buff 決定適用，缺映射回普通動畫，不使用角色旗標猜名稱。
- 同時移除未知 hero 冒充 Saika catalog 的回退；新增直接回歸。負向與新英雄任意名稱測試保留，仍不刪有資產引用的 Blueprint 相容介面。

## E146：新增直接測試仍須遵守 fixture 與 CLI 契約（2026-10-04）

- 新增 codegen 測試第一次從兄弟 test module 使用 private helpers，編譯 E0603；只將 cfg(test) helpers 可見度調整為 pub(super)，不擴大正式 API。
- 第二次 fixture 的 ability 少了 max_level／levels，先被既有內容驗證拒絕。補齊 max_level=1、levels={{}} 後兩個新測試通過。負向測試必須從合法基線改一個目標欄位，不把無關 schema 錯誤當目標驗證成功。
- codegen CLI 曾誤用 --out-dir，exit 1；讀實際 main.rs，改用 --out 後生成及 --check 成功。呼叫前先查既有入口參數，不能從 Rust CodegenOptions 欄位猜 CLI。
- 初次查讀輸出再度截斷；改以已知函式／明確行數拆讀並提高對應輸出額度。不得把截斷輸出當完整內容證據。延伸調查另猜測不存在的 OmNativeVisualActor.cpp；該次搜尋不是取得原生動畫處理證據，後續必須先 rg --files 確認實際檔名。

## E145：通用英雄事件 API 底下不能仍靠角色名稱生成事件（2026-10-04）

- bridge 只有 sniper_mode／three_stage 的 hardcoded buff→ability 轉換；任意新英雄無法沿用該事件來源。
- 改由 Lua buff_visual.ability_binding → 型別化生成 manifest → numeric buff 索引，名稱不參與投影分支。未知引用／模式／欄位／重複綁定與不一致 ID fail closed，未宣告綁定不猜測。
- 任意 custom ID 的八種 lifecycle 組合、非法 metadata／catalog 與既有快照回歸直接測試通過；沒有因此宣稱 Blueprint 相容介面與動畫分支全數遷移。細節見 generic-buff-ability-binding 進度檔。

## E144：即時proto與受版控fallback可能不同，即使server check通過（2026-10-04）

- 收尾查核發現core用vendored protoc即時生成，但omb無protoc時讀src/generated/game.rs；先前新增fog_grid後尚未同步該受版控fallback。初次server check成功不能證明fallback含新欄位。
- 使用既有core build.rs的OMOBA_UPDATE_PROTO_FALLBACK=1生成選項刷新，還原呼叫前環境值；不手寫generated structs、不安裝protoc或新增fallback。之後查明兩個fog_grid欄位，重新server check。
- 同輪曾猜測不存在的src/game_proto.rs；以實際build.rs與generated路徑定位。不把缺檔或rg無match當資料已取得。
- 收尾：既有生成選項及server重新check均exit0；fallback含FogGridPresentation、snapshot fog_grid及rebase fog_grid。後續修改proto必須同步受版控fallback，再檢查無protoc的server路徑；單查cargo check不是一致性證據。

## E143：rebase缺少已綁定迷霧與立即恢復快照（2026-10-04）

- 現況：恢復manifest只帶world，runtime清空fog；主／catch-up分支只送ResetView後等下一個gameplay frame，不能當完整迷霧恢復。
- 決定：manifest v2以新hash domain綁typed grid；v1無grid保留原hash，v1夾帶grid／v2缺grid／未知版本拒絕。恢復保留同隊同場探索，active epoch與後續frame／bootstrap一致；明確新view reset仍清探索。
- 主／catch-up改用同一helper，critical reset→恢復snapshot，另保留latest供renderer重連；不增加玩法world。core兩個新rebase測試、兩個原tick／sequence測試與runtime check通過，完整實戰驗收仍未執行。
- 操作錯誤：Select-Object -First誤填sixty，且同錯誤重複一次；數字參數必須直接填65，修正後讀取，不聲稱先前讀取成功。沒有新增PS／Python fallback或變動外部程序。

## E142：小地圖fog不能保留lease指標或把網格外當可見（2026-10-04）

- 通用模型綁定expected team、ABI11完整frame後驗全部geometry／三態／audience／tick，複製cells，不保留租約指標。control先返回保留狀態，full reset／Stop清空；同epoch幾何變動、同tick衝突、倒退拒絕。
- 三態row-run mask在公開路線／地形之上，memory／live在其上；網格外與schematic padding明確遮罩。不能用取樣fog替代entity target gate或在UE生成隱藏位置。
- 操作紀錄：建置輸出仍截斷；共用Engine Log.txt已被另一專案覆寫，不能當omfue證據。本次build task exit0／staged SHA及專屬Editor test report才是依據；未修改外部專案或停止其程序。
- E141外部lock已解除，本批正常build-only成功、owned Editor83404，本功能MinimapFogGrid一次1/1 passed；未跑全套MCP／PIE／雙UE60Hz。詳見unreal-minimap-fog-grid進度檔，完整rebase／整合仍待完成。

## E141：共用引擎 DLL 被其他專案 commandlet 鎖住（2026-10-04）

- ABI11 build-only 中 OmEditor／OmGenerated／OmRuntime compile/link 成功；完整 target 的 NetCore.dll link 失敗 UBA9001／LNK1104，build_ue_moba 依規則非零退出。不能把局部模組成功当完整建置。
- 讀取 actual UnrealEditor-Cmd.exe PID68548 command line：C:/portable/OpenKoikatsu/Saved/Tests/PaintSpray396/FrameFollowProject/OpenKoikatsu.uproject，執行 ok.PaintGalleryAudit。不是 omfue，不在停止範圍；保留其程序，不把停所有 Unreal 當一般修正。
- 決定：記錄外部 shared engine lock，停止重複整體 UBT；本專案 bridge 以既有 build_bridge.bat 收尾並驗 built/staged SHA，完整 Unreal build 等外部檔案鎖解除後才重跑。未新增 PS／Python fallback、修改引擎 link 設定或隱藏失敗。
- tool 結果含重複長 build logs 再次截斷；關鍵失敗來自可見明確 link 訊息，另以 actual process command line 檢查原因，不推測程序歸屬。

## E140：正式 fog 不可沿用無 geometry 的 legacy tiles／過期 ABI smoke（2026-10-04）

- 決定：增加 typed FogGridPresentation schema 1 與 ABI 11 OmFogGrid，保留 Q10 座標、三態與 audience；舊 tiles 不改意義，正式資料存在時非法也不退回 demo。
- lease 自有 metadata／cells；busy ring retry 保留結果，不覆寫仍有 reader 的舊槽。4 個本功能指定 Rust 測試成功，Unreal 建置結果見 authority-fog-ipc-bridge 進度檔。
- 發現 C++ header smoke 尚固定 ABI 7；更新到 11 並實際引用新 grid 成員，後續 ABI 改動須一起修改，不只改 Rust constant。原 terrain 測試改用 OM_ABI_VERSION。
- 操作錯誤：初次廣泛 rg 指定不存在的 scripts/build_ue.lua，且多段讀取再發生截斷；後續改讀實際 build_ue_moba.lua 與縮小區段，不以未取得全文當驗證完成。
- 建置前查核先前 owned Editor PID102392 的 exact project command line，僅停止該 project scope；restart 回報 force_terminated=true，另查 PID 已退出後才 staging。沒有停止其他應用、操作資產或跑完整 MCP 驗收。

## E139：fog 發布與快取必須同時約束 audience、封包及生命週期（2026-10-04）

- 決定：從 compiled Lua map 產生 bounded geometry，重用正式 Wave B read view；每隊 projector 在編碼／padding 前發布 FG01，私人 bootstrap 保留同一資料，不傳 source／hidden IDs、不加入 gameplay hash。
- runtime 先驗 bootstrap／event envelope、team／epoch／tick／geometry／衝突，再保留最新合法結果；失敗不猜全可見，production 清空 cache。缺更新保留，verified rebase 清空後等待合法新 epoch 資料；完整 rebase／IPC／UE 尚未串接完成。
- 追趕略過 intermediate 呈現：修正成每個成功 Applied 後立即借用 public events 留存，不能只在 extract_presentation_source 時更新；Duplicate／Rejected 不收。再查詢时曾猜測不存在的 team_wire_validation.rs，改用已知目錄搭配 --glob，不把缺檔當完成檢查。
- 操作錯誤：PowerShell 的 `{design,tasks}` 路徑展開造成 ParserError；直接交給 rg 的 `src/*.rs` 造成 error 123；猜測根 Cargo.toml 造成 error 2；多檔廣泛查詢再次截斷。後續明確列已知檔案，目錄搜尋使用 --glob，逐區段讀取。這些失敗不是成功讀取或實作證據。
- 確認：6 個指定核心測試、另 1 個真實 selective replica 追趕留存測試及 client runtime cargo check 通過；不重跑 MCP／Unreal／雙玩家／完整核心 suite。詳見 authority-fog-publication 進度檔，不勾選完整 6.1／6.2。

## E138：正式fog核心不可重算另一份玩法視野（2026-10-04）

- 權威已有WaveBReadView及共用line_of_sight，新增grid只從正式source／occluders取樣；不能從safe hero位置再猜700半徑，更不能拿碰撞地形當遮蔽。
- grid geometry先checked乘加／容量4096，wire FG01包含team／epoch／tick；decoder先驗全部size／states才配置，拒絕錯team／epoch／truncated／未知version／trailing bytes。
- sources radius<=0不平方成正半徑；raw差值用i128，平方距離用u128 saturating處理極端值，不沿用可能overflow的Fixed64運算。
- explored保存在每隊AuthorityFogGrid，reset必須epochadvance；同tickimmutable／倒退tick拒絕。不以呈現網格取代原entity disclosure／stealth／input gate。
- 多檔查詢輸出仍發生截斷；本批只以完整取到的signature／資料結構實作，縮小後續輸出。成功確認與未接網路／UE的限制見authority-fog-grid進度檔。
- 本功能3個指定測試通過，含shared LOS tree blocking與極端座標，沒有重新執行343個其他核心測試或UE全套；網路／Unreal尚未接上，不能宣稱正式fog呈現完成。

## E137：DemoFog不可直接當正式MOBA戰爭迷霧（2026-10-04）

- 實際fog_cache.derive對所有safe entities套DemoFogCache；其10-unit tiles／700半徑不是版本化權威視野或explored資料。ABI也缺grid geometry／provenance，不能憑固定尺寸畫正式小地圖，更不能從empty列表猜全可見。
- 改在production envelope辨識權威safe phase HUD metric後永久停用該session的demo fog／circles／trees／polygons，清cache；reset／缺HUD不回落demo。legacy/demo保留原路徑。這不更改真正authority visibility或safe披露，只禁止假呈現。
- 小地圖明確VISION N/A，公開geometry／live與memory仍保留；完整fog底色尚未實作，需後續正式契約，不以邊界修正勾選完整任務。
- 初次patch又使用不完整MarkerColor簽名導致整批拒絕；以完整函式行重新套用，新增runtime／UI確認，禁止把patch error當已寫入。
- 本批停止先前owned Editor106460並另查PID退出後建置，未操作資產或擴停止範圍。結果見formal-fog-boundary進度檔。
- Rust指定測試1 passed／runtime executable build exit0，OmGameEditor Succeeded，MinimapFogBoundary單輪success=true。ABI10未改但bridge dependency重建，stage改c66b56…，不能沿用前批554332…報告；沒有實作正式fog底色或全套驗收。

## E136：小地圖不能把owner當team（2026-10-04）

- OmFrameEntity.owner是player ID，owner>0不表示敵人；以前所有其他玩家被画紅。同隊映射改用權威公开scoreboard roster，先完整原子驗證再使用，缺值／重複／不合法時未知。
- roster只補已披露live marker的team，不用公開player清單新增位置；記憶仍不帶owner／team。NPC無owner保持灰色，不以spawn或位置猜隊伍。
- owner i32保留Rust u32 bit pattern，roster查詢轉回u32；high-bit fixture驗證。不解釋-1哨兵為u32::MAX合法owner，這個既有ABI歧義須另行schema遷移，不能偷偷猜。
- 查找再次猜OmScoreboardModel.cpp不存在；改rg --files確認目前只有MinimapModel。patch首次使用不完整Project函式行導致拒絕，確認實際完整簽名後重套，不將失敗視為寫入。
- 本批Editor由前批啟動、無資產修改；restart stop回報force_terminated，另驗原PID107948已退出才建置，不擴大停止其他專案。
- 確認結果記入minimap-teams進度檔，完整套件與雙UE長測不在本批執行。
- 本批build-only exit0、OmGameEditor Succeeded；MinimapTeams單輪1 passed／0 failed／error_count0，独立report已保存。未把十人fixture稱為十人網路實戰或像素驗收。

## E135：小地圖安全記憶不能混入live與目標資料（2026-10-04）

- entities pointer 為空時的早退會漏 ghost-only 完整view；改成兩個獨立迴圈，live與remembered型別分開。memory不含entity reference／owner、不猜team、不自行TTL或更新hidden位置。
- 以render_id排除已有live／重複ghost，epoch0／非有限／範圍外拒絕。公開route／terrain決定bounds，memory不能擴張地圖或clamp到邊缘形成假目標。
- restart從根目錄執行stop導致誤找根目錄om.uproject，工具正確拒絕且未停止任何程序；修正cwd到omfue，停止本批先前啟動的Editor並另核對PID91152已不存在。後續restart命令一律指定project cwd，不使用root預設。
- 此次專案stop回報force_terminated=true；僅本批已啟動的本專案Editor，沒有執行資產編輯或對局。不能把stop dispatch當退出，建置前另查原PID。
- 成功確認與限制記入minimap-memory進度檔；全套完整驗收留最後。
- build-only exit0、OmGameEditor Succeeded；MinimapMemory單輪1 passed／0 failed／error_count0，包含正式WorldBridge整合，独立report已保存。不跑完整套件或雙UE長測，不以此勾選完整6.1／6.2。

## E134：小地圖漏掉公开地形與功能確認範圍（2026-10-04）

- 原小地圖缺route pointer就提早返回，範圍也只從兵線取得，無法顯示獨立公開地形。改route＋terrain共同計算示意範圍，geometry自有複製，不拿live entities或vision polygons補資料。
- terrain批次最多32；空指標／非有限／min>=max拒絕並清空完整view，不保留前一個有效矩形造成部分地圖。control不清、full empty清空。
- 初次patch的MakeLines context漏同一行後半段，整批未套用；先rg確認實際檔案未變，再使用完整行套用，不能把工具失敗當作已實作。
- OpenSpec文件多次合併輸出超量；以分段補讀截斷範圍。後續單次輸出應控制總量，避免在多個命令間累積截斷。
- 全套兩輪驗收不再每批執行；工具支援單一已知功能單輪與獨立報告，不以功能確認覆寫完整驗收證據。結果記入minimap-terrain進度檔。
- 本批build-only／OmGameEditor編譯成功；MinimapTerrain單輪exit0／success=true，獨立報告已保存。未執行全套／PIE／雙UE／60Hz長測，不宣稱完整UI或框架已完成。

## E133：獨立碰撞地形呈現與ABI10（2026-10-04）

- 共用PresentationExtras保留validated compiled map，full/lifecycle同一路徑；新增frame-owned terrain_rects與獨立UE ISMC，絕不能塞進blocked_regions／polygon_occluders而意外改fog。Unreal物理／overlap／navigation全部關閉，Rust仍持唯一collision rules。
- 首次編譯E0063：原PresentationExtras完整literal漏public_map；E0596測試extras未mutable；E0277新增OmTerrainRect不必要derive Debug／Default而OmVec2不支援。補明確None／mut，僅保留所需Clone／Copy，不為方便fixture擴大基礎型別改動。
- frame結構增加欄位必須升ABI10並同步cbindgen staging與Unreal建置，不能沿用9。empty header／slot初始化與publish均需帶新Vec與null pointer，不能讓frame pointer引用臨時data。
- 查找再猜scripts/ue_bridge_stage.lua不存在；改用rg --files定位。合併讀取過量截斷後補讀所需片段，不以截斷尾部猜函式。
- build-only exit0：ABI10 header／bridge同步stage、OmGameEditor Succeeded；bridge55＋1ignore／integration2＋1ignore與Lua terrain13斷言已過。依使用者最新指示，MCP／雙UE／60Hz完整驗收集中到最後，不再每個增量重跑；不是地形美術／完整地圖或60FPS完成。
- 建置後查header又誤用OmRuntime/Public路徑；實際產物在Source/ThirdParty/OmBridge/include/om_bridge.h，建置輸出已明列。查找須先使用實際輸出或rg --files，不再猜目錄。

## E132：編譯地圖幾何單一來源與開局驗證（2026-10-04）

- 發現權威MobaMatch與初始KCP bootstrap各自組装相同矩形corner；抽compiled_blocked_regions為唯一轉換，後续Lua地圖不需新增角色／map專屬C++或BP。
- 只驗map id／hash不能確保client收到相同碰撞。validate_metadata要求compiled map同時有唯一schema1 blocked-regions及canonical bytes一致，runtime bootstrap與正式ready都拒絕缺失／重複／錯配，legacy無compiled identity保留。不用預設地圖或空碰撞fallback掩蓋錯配。
- decode_public_blocked_regions原先在檢查內容前依wire count配置Vec；加入剩餘bytes上限後再配置region與point capacity，u32::MAX攻擊輸入測試不可觸發巨大配置。
- 本輪大段合併讀檔再截斷，已分檔／分段補讀完整OpenSpec。patch fixture空白context不符導致整批未套用，讀實際行後重新套用；禁止猜格式或把失敗當已寫入。
- 查active-session再次猜不存在路徑；改從實際run目錄與報告processes讀取，不拿缺檔當程序退出證據。大量列檔輸出仍須縮小到具體suffix／子目錄。
- 最後core343／base108（227.93秒）／server156＋1ignore／runtime62＋8ignore＋3integration／bridge54＋1ignore＋2integration／1ignore全過；codegen11／16與OpenSpec strict／diff通過。format後重新build／stage，OmGameEditor Succeeded，bridge2d2e19…／script三份719264…獨立一致。
- 真實新版雙UE60Hz1791116233 success：双route4-2-4／minimap3／普通移動；保存三方team1 41 PASS rows／27 unique ticks、team2 43／27至3240，零FAIL且所有已驗parity=true。報告SHAf45237…，五owned PID另驗皆退出，post-run stage仍一致。沒有PNG／LAN／稳定60FPS／地形營地美術驗收，整項仍未完成，完整紀錄見compiled-map-contract進度檔。

## E131：公開三路地圖身分與 Unreal 路線（2026-10-04）

- 缺口：Unreal共用route actor已存在，但IPC只傳lane_length，bridge把三路畫成一條直線。決定用public bootstrap map/moba-layout schema1發compiled map id／catalog hash，runtime bootstrap與bridge握手都嚴格驗證，使用同一Lua生成常數恢復完整route。不傳aggro／camp timer，不把collision terrain冒充vision occluder，不新增角色C++／BP。
- 新增RuntimeReady protobuf7／8後，舊fixture literal漏新欄位E0063；補Default並將production ready封裝成ready_envelope_for_start，真實TCP晚連線／重連測試沿用此函式。同步更新proto與checked-in prost fallback，不只改其中一份。
- Windows字面presentation*、猜render_projection.rs／codegen/templates／ue_minimap_observation.lua／scripts/lib失敗。實際列檔後定位；沿E128規則先rg --files再查內容。本輪大段exec合併输出再截斷，已分開讀完缺漏context，後續不能靠提高單command上限忽略exec總上限。
- 暫只封關public identity／三路route流程；營地／地形美術仍待下一段，不放寬Lua導航安全限制，不把CPU actor同步log當PNG／60FPS證據。
- 最後core340／base108（214.43秒）／server156＋1ignore／runtime61＋8ignore＋3integration／bridge54＋1ignore＋2integration／1ignore、Lua觀測器通過。重新OmGameEditor Succeeded，stage90bfc4…與script三份65d6e1…獨立一致。真實雙UE60Hz1791115215 success／雙隊route4-2-4與native minimap3／正式snapshot移動；三方team1 62 PASS rows／46 unique ticks至5520、team2 70／45至5400，零FAIL／全部已驗parity=true。報告SHA6fe606…，五owned PID另查皆退出。不是PNG／LAN／60FPS／完整三路UI驗收；全項5.4／6.1／6.2保持未完成，完整數據見三路layout進度檔。
- 最後文件patch附帶不存在的design heading導致整批驗證失敗；移除無意義hunk再套用，沒有猜heading或忽略失敗。大段讀檔再次截斷後改用rg精確行定位。
- 第一輪雙UE1791114846失敗：team1單路，team2第三條route fatal Cannot generate unique name。初始KCP空bootstrap另有producer，public_metadata=空，第一名加入玩家永遠缺map；補相同compiled id／hash與terrain到外／內bootstrap，不只修改tick producer。初始碰撞也須在第一個MoveTo之前安裝。
- route actor MakeUniqueObjectName使用World當outer，但SpawnActor實際outer是PersistentLevel。多路名稱的數字suffix衝突，改同一PersistentLevel作unique outer與OverrideLevel；只修共用框架，不增加每map BP。原失敗report保存，五owned PID已另查皆退出。修正後必須重新build／stage／双UE，不調高timeout冒充修復。

## E130：共用 NPC 靜態避障（2026-10-04）

- 決定：抽出英雄既有 static_next_waypoint，三路 creep 追擊／回兵線與野怪追擊／回位使用同一公開地形 swept-circle；直線可通不做 BFS。單路 None／TD 保留既有行為；Lua 兵線與 camp leash 的生成限制暫不放寬。
- 首次 route 測試失敗：只看每 tick 的短 proposed step，繞障後下一 tick 又走回牆邊，最後卡在 x=55.78。修正為先檢查到完整當前 waypoint 的路徑，受阻就持續共用尋路；不能只證明每步沒有碰撞，還必須證明抵達與來回返回。修正後 shared NPC 60Hz 往返／route cursor／零負預算／blocked endpoint 測試通過。
- 第一次 patch context 太寬，把 regions resource 插入另一個同名 properties 區塊；編譯前以 rg 查實際插入位置，改用唯一註解定位到 NPC tick。後續 patch 必須帶足夠唯一上下文。
- 本輪 OpenSpec 大段讀取再次截斷；分段補讀全部缺漏，不把截斷輸出當作完整 context。查 export 時誤猜 runtime.rs（實際module分檔），沿E128規則只用rg --files定位，不再推測檔名。
- 最後驗證：core339／base108（204.58秒）／server156＋1ignore／runtime61＋8ignore＋3integration／bridge53＋1ignore＋2integration／1ignore通過，新NPC fixture5400雙隊steps每tickhash零repair、兵與camp越牆／回位Heal通過。真實三路KCP60Hz1791114001保存證據雙隊9／9 unique checkpoints至1080、零FAIL／parity全true，三PID另驗退出；不是KCP現場NPC跨牆。最後OmGameEditor Succeeded、獨立stage d6f3e…／script三份4305…一致、codegen與OpenSpec strict／diff check通過。無Editor／PIE／UE地圖或60FPS驗收；完整5.4仍不勾選，不沿用E129 SHA。完整矩陣見NPC terrain進度檔。

## E129：Lua 地形編譯／公開地圖接入（2026-10-04）

- 決定：Lua map 新增最多32個整數矩形 terrain（id／min／max），編譯產生 MobaTerrainConst、納入既有完整 map catalog hash／compiled agreement。選定 compiled map 才取代 BlockedRegions；None 保留原單路／TD 地形，不增加英雄 C++ 或 Blueprint graph。
- NPC 尚無通用避障；生成器先以相同 i128 swept-circle 驗證所有兵線100-unit corridor與野怪完整 leash＋100 margin 不被地形擋住。不得把這個安全限制冒充NPC已會繞障，也不允許為通過測試而關掉碰撞。
- 新測試初次編譯 E0308：Specs read_resource 的 `.clone()` 複製 Fetch 而非底層 BlockedRegions。改 `(*read_resource::<BlockedRegions>()).clone()`，避免保留 read borrow 時再 write_resource。
- 同一 scripts workspace 完整測試仍執行時重新編譯新增測試，Windows linker LNK1104 無法覆寫正在執行的 base_content test EXE。等待原 session 正常完成再串行重跑；不刪 target、不 kill 不明程序，不把 Cargo build lock 當成執行中 EXE 的保護。不同 target workspace 可以平行，但同一 test binary 必須串行。
- 啟用真實 Lua 地形後，舊完整 base106 回歸仍全部通過，但耗時301.43秒（之前無地形185.07秒）。先補 integer broad phase 與四點矩形 stack buffer，減少遠處牆的 narrow-phase／heap allocation；不用移除碰撞換測試速度，不把總測試耗時當成 server tick wall-clock 基線，最後版本需重新驗證。
- 查找測試入口又誤猜 verify_2player_ue.lua／lib/runtime_smoke.lua／run_3process* 路徑；改以 rg --files 得到實際 scripts/run_moba_runtime_smoke.lua。沿用E128防錯規則：先列已存在檔名，glob只用rg -g參數。
- 最後驗證：真實Lua地形7200雙隊steps逐tick零repair／hash／正式MoveTo detour抵達、base107（271.10秒）／core338／sim69＋8／template39＋23＋8＋2／server156＋1ignore／runtime61＋8ignore＋3integration／bridge53＋1ignore＋2integration／1ignore通過。三路KCP60Hz1791112988成功、9／8 unique checkpoint至1080、零FAIL、三owned PID另驗退出；KCP未現場跨island，不混稱。最後build-only Succeeded、獨立stage094de…與script三份c5c4…一致、codegen11／16與OpenSpec strict／diff check通過。完整5.4／UE地圖／60FPS仍未完成；不能沿用E128 SHA。

## E128：公開地形整段碰撞與 60Hz 尋路前置（2026-10-04）

- 問題：hero planner 僅檢查格點、同格／超過 96 格時直接回傳目標；移動只查終點。兩端合法不代表路段合法，薄牆／對角切角可能穿透。決定以 omoba-sim 共用 i128 固定點 swept-circle polygon 檢查每条 BFS edge、fallback 與移動／axis slide／near snap；零或負預算不移動。只查公開靜態地形，不把隱藏單位加入導航。
- legacy polygon f32 僅在資料邊界 round 成 raw Fixed64；幾何不轉回 render float。raw 座標／半徑限制 2^29（約 524288 world units），超界／非有限資料 fail closed，防止 i128 平方溢位。這不是完整 Lua 地形或 NPC navmesh。
- 初次正式雙隊地形 fixture 未發生繞路但已抵達目標；原因是 MOBA 英雄出生不是 (0,0)，牆被放在出生點後方。改從實際出生位置設置公開牆與目標，不 teleport 英雄、不修改 gameplay 繞過。必須同時驗證實際 detour、抵達、每 tick 雙隊 hash 與零 repair。
- 本輪讀取過大造成工具輸出截斷；改分段讀完 OpenSpec context。再次誤用 Windows 字面 fixed*／runtime/*.rs 與推測 team_projection_runtime.rs 路徑；改用已確認目錄配合 rg -g '*.rs'，禁止把 glob 拼入 Windows path 或猜檔名。
- 最後驗證：sim69＋8、core338、base106（185.07秒）、server156＋1ignore、runtime61＋8ignore＋3integration、bridge53＋1ignore＋2integration／1ignore全部通過；三seed地形3600雙隊steps每tickhash一致／無repair／實際detour抵達。build-only OmGameEditor Result:Succeeded、獨立stage79533…與script三份c6c6…一致。codegen11files／16inputs與OpenSpec strict／diff check通過；沒啟動Editor或宣稱UE地圖／60FPS完成，5.4仍不勾選。不沿用 E127 的 DLL／stage SHA。
- codegen --check 預設 content-root 是 scripts/lua_data、out 是 Plugins/OmRuntime/Source/OmGenerated，兩者不在同一個 cwd。先後只換 cwd 皆失敗；讀 main.rs CLI 後明確傳兩個絕對路徑：`cargo run --manifest-path omfue/codegen/Cargo.toml -- --content-root D:/code/omoba/scripts/lua_data --out D:/code/omoba/omfue/Plugins/OmRuntime/Source/OmGenerated --check`。不改生成檔、不把參數／cwd 錯誤判成 stale codegen，之後優先使用既有 Lua 建置入口。

## E127：Lua 野區營地與 60Hz 正式傷害整合（2026-10-04）

- 最後封關本輪單怪野區原型：base105／core335／server156／runtime61／bridge53及template38＋23＋8＋2全部通過；三seed野區18000與三路41400雙隊steps逐tick零repair一致。最後release DLL45ea50…透過真實DLL headless seed42勝利22745／四槽46-6-5-4／全tick replay／end1；stage7d934…再次獨立核對、兩headless PID退出。詳見野區進度檔；沒有LAN／UE野區畫面／60FPS證據，5.4仍不勾選。

- 初次編譯 `AiType::None` 命中 legacy enemy 的同名 enum，E0599。改成明確匯入 `runtime::native::comp::unit::AiType`，不用 glob 推測型別。
- 測試對 `Outcome::Damage` 使用 `..lethal(...)`，E0436；enum variant 不支援 struct update。改以 pattern 修改 fixture 的傷害值。
- 查找再次誤猜 native/visibility.rs、native/filtered_specs.rs、native/outcome.rs、native/mod.rs 與字面 `comp/*` 路徑失敗。已以 `rg --files` 定位；後續禁止自行拼猜路徑與將 Windows glob 當目錄。
- 設計：新增 HostileNeutral 而不修改既有 Neutral；kind=3 只披露可交戰分類，野怪沒有 VisionSource，AI 仇恨／回位／重生 timer 不傳客戶端。所有正規及 Lua direct damage 均檢查回位免傷；獎勵僅實際 first-lethal 的 roster 英雄領取，重複 Death 不得重複付款。
- 執行過程中的 compile／局部生成成功不算野區／5.4 完成；本輪最後驗證見上方摘要，僅單怪野區原型封關。
- 正式輸入／雙隊重播 fixture 首次不發生第二次擊殺：撤退到基地後營地距離 1063 超過英雄視野 1000，合法目標篩選一直拒絕 AttackTarget。不是 AI 卡住；改成先 MoveTo 回野區、可見後才攻擊，保留迷霧合法性，禁止測試用未知目標繞過。
- 第二次仍不擊殺，實際 trace 顯示 attack_seq 每 tick 增長但沒有命中：fixture 每 tick 重送非 queued AttackTarget，正式新命令會重啟 attack windup。改成在可見且接近營地後送一次，已有相同 active target 不重送；不改 production 命令取消契約，也不以直接傷害代替此正式輸入測試。
- 擊殺已成功但原 1800 tick 視窗不足以覆蓋完整撤退／回位／重返／擊殺＋15 秒重生，改為 3000 tick，擊殺後正式 MoveTo 撤離避免空閒普攻再次刷營地。三 seed 最後局部驗證均 6000 雙隊 steps、230 external effects、一次重生，hash 零 repair。
- template default features 僅跑 7 unit，不涵蓋 runtime Lua 驗證。補 `--features runtime-lua-content` 才發現新測試少匯入 content-model hash 函式（E0425）；已補明確 import。不可用 default 子集宣稱完整 authoring validation 通過。
- build-only 後又跑 bridge 測試，新建置目錄 DLL 與 Unreal stage SHA 不同，獨立 verify-staged-only 如預期拒絕。等全部 Cargo 測試結束，重新 build-only 並再獨立 verify，最後 stage `2223060b2ce8ab712ccb64dd45eedab9f208f5ba0cee2d34cc7a0df75f8b40c6`；不能混用先前 stage 當最後驗收。多個 PowerShell 診斷命令的最終 exit 0 不代表前面的 Lua 已成功，逐條讀取結果。
- 誤把 `omb/scripts/base_content.dll` 當 debug stage 目標；實際 Lua freshness 工具 stage 到根 `scripts/base_content.dll`，與 debug build／Unreal stage 三份皆 `26c588…54d8` 一致。legacy omb/scripts 檔未覆蓋，release headless 明確 `--scripts-dir scripts/target/release`，不拿錯 DLL 的 SHA 當 stale runtime。
- 三路完整測試加入營地後 seed42／539365380勝利tick改為7930／7776；記錄最後程式結果，不沿用上一版7870／7767。完整 base105 全過、野區三seed18000雙隊steps與三路41400雙隊steps零repair，並不等於LAN／UE野區畫面／60FPS驗收。
- 最後 review 發現 ScriptDirectDamage 沒沿 filtered authority-combat settlement 保護，可能在客戶端重扣 HP／回位免傷分歧。補與 regular Damage 相同的 DisclosedAuthorityCombatTargets gate、針對性 core 回歸；重跑受影響 suites 與最後 stage。此前222306…是修正前階段值，最終SHA與數字以野區進度檔最後更新為準。
- UE5.8 engine共用UBT Log.txt 顯示Failed但實際target是 `C:/portable/OpenKoikatsu/OpenKoikatsu.uproject`，已被其他工作覆蓋。不得把別的專案log當OmGame失敗；本輪restart明確檢查UBT exitstatus後exit0，不修改／終止外部專案。最後stage7d934…、script ae61…三份相符；補強後base105／core335／server156／runtime61／bridge53全部通過。

## E126：Lua 三路地圖與固定點數導航（2026-10-04）

- 最後版本base102全部通過；三seed1／42／539365380勝利4955／7870／7767，雙隊41262 steps逐tickhash無repair。實際release DLL三路60Hz headless seed42勝利12808、四招26／11／3／7、全12808tick重放。最後fullbuild stage127176…／MCP11BP／Editor67136兩輪19/19＋串行PIE通過，owned正常退出獨立確認；core334／server156＋1ignore／runtime61＋8ignore／bridge52＋1ignore／template37＋23＋8＋2／sim65＋8、codegen check／OpenSpec strict／diff check通過。5.4只完成三路原型，野區／地形通用避障／完整建築層次／Unreal map仍未勾選。
- Review發現cursor已complete後，若NPC被aggro拉離終點，原helper返回origin會永久停在追擊點；修改advance_route完成狀態仍朝最後點前進，新增回歸斷言。此變更讓seed42勝利tick由7920變7870；重跑全套及最後stage，不混用舊數字。
- 原生PIE煙霧fixture截圖仍是合成120Hz／3FPS且HUD空owner，僅能證明原生mesh／ghost與UI surface；不把它當本輪三路、完整HUD或60FPS的驗收。最終PNG已檢視、限制寫入進度檔。

- 新基地解鎖測試在第三座塔 HP 歸零後立刻期待 unlock，失敗。正式 outcome 是 Damage 排入 Death、下一次 drain 才 retire；修正測試按正式順序排空，不把 HP 歸零當實體已拆除，不修改 production 傷害門檻。
- 新測試 patch 猜測 `mod tests` 已有頂層 `use super::*`，anchor 不符而原子拒絕；讀實際 module 後使用既有模式的函式內 import。猜測 headless 檔名用連字號失敗，實際檔名 `moba_headless.rs` 已由 rg 定位。

- 新增 `moba_maps.lua` 初稿直接回傳 table，編譯失敗：`ctx.include` 的契約是 builder function，不是資料 table。已改為 `return function(ctx) ... end`；後續新增 Lua module 先讀相鄰模組及 loader 契約，不以一般 Lua require 的形式猜測。
- Windows `rg` 字面路徑 `tests*` 查找失敗；應搜尋既有目錄或先 `rg --files`，不得重複使用不存在的 glob 路徑。多檔輸出超過設定上限時只視為定位結果，必要段落縮小重讀。
- 本輪保留單路預設，三路 map opt-in；基地規則由 Lua 設定且只接受已實作的 `all_lane_towers`，傷害邊界必須再次檢查。固定點數路徑不是 navmesh，不能用路徑測試宣稱地形避障、野區或 Unreal 三路驗收完成。

## E125：Unreal 首次學習後施法驗收（2026-10-04）

- 最終1791105946 saved verifier success：兩隊學習2／cast3各一次，raw cast tick2524／2838 HP563200→706560，compiled Lua heal143360（140HP）exact；5,883快照，兩隊各25unique三方checkpoint至3000，post-cast4／2。五PID全部退出、來源SHA前後不變。最後fullbuild／MCP11BP／Editor76216兩輪19/19／串行PIE與stage0f471b…通過，ownedEditor退出另驗。回城／shop／upgrade共享PID身分verifier及8mock通過，舊kill-assist／lane XP原captures重驗通過；core334／base98／server155重新通過，依明確數值規則scope封關5.3為20/30。E121與全部其他任務仍未宣稱完成。

- 收斂5.3時舊回城run1791073998保存verifier誤判ue-p1還活著：PID72524已被Windows conhost.exe重用，原記錄是UnrealEditor.exe。新增共用只讀saved-process identity判定與8個mock正負向；只有不存在或明確不同exe才可證明舊owned程序已退出，同exe／缺identity／pid不符／重複role仍拒絕，絕不stop現在的無關程序。shop／Recall／upgrade保存verifier共用此判定。

- 修正後1791105946實跑success／cleanup true，兩隊原cast3 result與HP550→690／正CD。保存raw verifier因誤把稀疏CommittedVitals視為每tick存在而拒絕；已讀authority emitter確認hero每tick final CommittedEquipmentStats(kind22、40 bytes)前16 bytes是HP／maxHP，改核對此最終事實，不補造缺失資料、不放寬exact tick。原captures不重跑交易，重驗相同保存證據。

- 第一輪1791105739兩隊CtrlR學習與正式R input3送出、runtime AbilityCast ok，但沒有cast result log導致60秒gate逾時。新harness在讀applied_inputs前因control-only frame缺HUD提前return，漏掉合法ACK；決定與既有升級／Recall一致，先讀結果再檢查HUD。保留失敗run，不延長timeout、不以server接受冒充端到端成功；修正後重跑。

- 決定保留CtrlQ原模式，新opt-in學R後以唯一普通R binding正常送出；不注入治療／傷害／rank，等待正式玩法HP缺額（正常level growth也會擴maxHP，不必聲稱一定是戰鬥伤害）。驗收需原cast一次、status0、正CD與實際HP增加、raw final EquipmentStats與IPC逐筆一致、施法後unique三方hash及五PID退出。
- 本輪檢索猜錯bridge header目錄。先rg定位Source/ThirdParty header再確認共享enum，不新增硬編碼魔數；未造成建置或玩法變更。
- runtime61 passed／8 opt-in ignored、新舊Lua觀測器通過；完整UE與真實網路結果待追加，不把單元測試當端到端。

## E124：雙 Unreal rank0 首次學習（2026-10-04）

- 最末adaptive width版本重新fullbuild／MCP11BP compile／同Editor105396兩輪19/19全部通過，codegen --check／OpenSpec strict／diff --check通過。owned105396正常退出後inspect不存在；不是只派發shutdown，也未留下Editor或對局程序。
- 最終1791104682保存verifier success：5,932 live raw/IPC snapshots、兩隊原CtrlQ input2各一次、rank0→1／p1SP3→2／p2SP4→3、每隊25個unique三方checkpoint至3000（post-upgrade6／2），五PID另驗退出、證據SHA前後不變。長名layout門檻現在通過，四PNG仍有E121局部文字截短，不能宣稱GPU文字問題修好或整個UI完成；舊rank1→2保存verifier亦通過。
- 第三輪1791104474對局success／cleanup true：每隊input2一次、rank0→1、p1SP3→2／p2SP4→3、三方50／50至3000。保存verifier因新apprentice長ID的確定layout不足拒絕：touch desired120／lance117／mend119，但內寬116。不是E121「layout充足卻偶發GPU裁字」的同一證據；決定共用SBox寬依實際name desired＋padding12，min128，既有短名不改。不得直接改verifier門檻、縮ID或宣称旧GPU問題已修復；保留失敗報告與PNG後重跑。
- 修正cast／learning ID後full OmGameEditor build與stage f84a14…、BpGeneratorUltimate11BP gate通過；同Editor81376兩輪19/19含rank0文字與first-level6門檻回歸，owned81376正常退出另驗不存在。正式雙UE首次學習仍須重跑，不能把Editor unit測試冒充端到端。
- 第二輪1791104221 KEY_GATE顯示explicit0／pointer0仍queued0，否定前述pointer假設，已撤回那個production行為變更。實際程式阻擋：DispatchHud將rank0的OwnedAbilityIds清0（正确禁止未學施法），SubmitOwnedAbilityUpgrade卻也用同欄位gate，導致不可能首次學習。新增獨立OwnedAbilityUpgradeIds保留HUD catalog ID供升級入口，cast ID仍rank0清0，authority仍決定level／SP。兩個失敗run皆保留且owned cleanup verified，不把未知cursor根因或fallback當修復。
- 第一輪1791103763正式Ctrl+Q delegate各執行一次但queued=0，雙runtime只有Move input1，沒有upgrade提交；launcher失敗且五PID清理verified。檢查UpgradeAbilitySlot使用IsHudConsumingInput把滑鼠停在HUD也當keyboard輸入消耗。決定keyboard升級僅擋explicit bHudConsumesInput（文字／modal），不擋pointer hover；仍走正式反射／IPC／authority，沒有fallback至API。新增opt-in KEY_GATE診斷驗explicit／pointer，確切第一輪cursor狀態未記錄，不能把推論寫成已證實根因；需重跑。
- raw verifier patch 用 generic tick_rate anchor，誤放另一個 capture test，E0425 initial_rank／first_learning 不在正確scope。rg定位後移至明確 upgrade_smoke anchor；不放寬raw驗證，重跑編譯與兩種保存證據。
- Windows rg 不接受不存在的字面 `scripts/*tests*`／`Private/OmAbility*` 路徑；以 rg --files 定位或搜尋既有目錄。讀取猜測不存在的 cleanup 測試路徑失敗，後續不得猜檔名。多工具總輸出截斷時改單次／分段讀取，不能宣稱已讀完截斷內容。
- 決定FIRST_LEARN與舊UPGRADE明確互斥，使用相同正式Ctrl+Q綁定；前者server-owned apprentice四槽0，一出生點＋正常每級SP，後者仍rank1→2。不能從runtime FIRST_LEARN旗標繼承自動操作，仍強制關閉。此輪先驗Ctrl首次學習，不冒充學後四招／OS按鍵／GPU像素封關。

## E123：純 Lua rank0 英雄與正式網路首次學習（2026-10-04）

- 保存 verifier 不可要求 final checkpoint 計數等於 launcher 當時統計：launcher 取樣後至 owned server 退出前會追加合法紀錄。首次嚴格相等斷言失敗，改成獨立逐筆驗三方 PASS／zero-repair／frame hash、unique tick 與 post-learning 足量，且必須包含 launcher 原末次 tick 並至少涵蓋原統計；沒有改原 capture、縮短門檻或忽略 FAIL。
- 最終 base_content 全測試 98 passed；原 registry 硬數字與 server stream 問題都已修正後重驗。真實 KCP60Hz run1791102528，1,582 live snapshots exact raw rank／SP、每人升級 input3 只有一次、未學施法與 level6拒絕／学後CD／雙隊post-learning parity通過，三個owned PID獨立退出。UE生成與OmGameEditor完整build／bridge stage通過，不冒充本輪Unreal rank0按鍵／畫面驗收。

- base_content 全測試兩項只因硬編碼 registry=12 失敗，新增四招後實際16且生成清單／metadata已一致。改與generated IDs／實際manifest精確比對，另加全體ID唯一性與八個declarative skill的effect metadata斷言，不只是改成下一個硬數字。修正後完整98項已重跑通過。

- raw verifier 首次失敗：只讀 server.stderr，但 server 的 log4rs WARN 實際在 stdout。rg 查到每玩家未學施法／level6拒絕訊息，改讀兩個原始 stream 並精確核對 pid／slot，不以runtime replica的WARN替代authority證據。原capture不變，重跑只讀驗證。

- ECS fixture E0277：String 不是 Copy，不能 `["training_apprentice".into();2]`；改兩個獨立 String，修正後單項測試通過。Lua prototype 找來源時以穩定 id 查詢，不依賴 heroes 最後一筆；既有UE launcher明確關閉新 internal FIRST_LEARN flag，避免繼承環境把runtime injection誤當Unreal操作。

- 先前單路 server 寫死 training_luminary，新 Lua loadout 無法正常選用。決定新增 server-owned AUTHENTICATED_HERO_BINDINGS，必須屬於既有 authenticated roster 且引用 active compiled hero；缺值保持原英雄，沒有 wire 選角或信任 client 任意 hero ID。
- 新 training_apprentice 以 Lua append-only hero／獨立 apprentice_* abilities 宣告，四槽 rank0／一點、lance 第一級 level6。不能共用同 ID 再生成同一 handler，既有 registry 為逐英雄產生；用獨立追加 ID 避免重複 FFI registration，不改原 lumen_lance 的 level1 門檻。
- fixture 只用明確 FIRST_LEARN opt-in＋test-mode 的一般 renderer intent，不直接寫 rank/SP/傷害。先嘗試未學習施法／等級不足升級，再首次學習、同request重送、學後施法。必須核對 raw authority facts／原input接受次數／IPC rank-SP-CD／雙隊 post-learning hash，不能把 status0 transport acceptance 當 gameplay 成功。
- 編譯 E0271：request closure ordinal 預設 i32，request_id 要求 u64；顯式 ordinal:u64／常數u64，並把player_id複製到local避免closure持有self借用。修正後重跑，不放寬資料型別。

## E122：rank 0 初次學習與 Lua 出生配置（2026-10-04）

- HUD regression E0502：長持有 components mutable borrow 時不能 clone owner；改每輪 insert 已序列化 Hero，clone 前沒有可變借用。修正後必須重跑測試，舊工具 session 的失敗不能視為新 source 結果。

- 猜 omb/src/scene、omb/src/server 目錄不存在；改直接搜尋 omb，正式 handler 位於 src/state/resource_management.rs。後續不要假設其他 workspace 採相同目錄結構。

- json! test macro 不接受 Rust `[0;4]` array repeat 表達式直接作 JSON array，改四個明確元素；實作 library 編譯不受影響，補測前不可稱全測試通過。

- template-ids 全測試因註解中的 `Story/TD` 觸發既有「舊 Story 路徑」文字檢查；不是生成資料回退。改註解為 Story and TD，保留檢查，不放寬測試。

- test 編譯 E0594：HeroEntry 只有 Deref，不能透過它改寫共用欄位；改 `.common.moba_loadout`，不新增不必要 DerefMut。猜 `codegen/src/model.rs` 不存在，後续先 rg --files 定位而非猜檔名。

- 問題：single_lane 強制全部 rank 1，無法宣告未學習技能；ScriptCast dispatch 的 `max(1)` 可能使內部事件繞過正式 input 的未學習檢查。決定加入英雄 Lua 可選 `moba_loadout = { ranks = {0,0,0,0}, skill_points = 1 }`，預設維持舊四槽 rank 1／0 點；只在 MOBA match birth 使用，respawn 保存進度，不重發點數。
- 編譯 E0433：content-model 在 template-ids 的正常 dependencies 是 optional，只在 build-dependencies 永遠存在，不能讓預設生成常數直接引用該 crate。修為輕量 MobaLoadoutConst 邊界，由兩個 parser 轉換，不擴 script ABI 或強制 runtime serde dependency。
- 多檔 apply_patch 失敗：同一檔 hunk 先下後上不能定位；工具原子失敗沒有部分修改。先 rg 確認，再依檔案順序提交 hunks。
- rg 帶 Windows `src/*.rs` 路徑得到 os error 123；改目錄配 `-g '*.rs'`。大段合併讀取導致截斷，改分段完整讀 OpenSpec context；不可把截斷當已讀完。
- GameMode::Moba 是舊 Story 預設，不可拿它作新規則 opt-in。內部 rank-zero cast gate 改用 MobaMatch 存在，避免改掉 Story/TD 舊腳本 fallback；filtered 正式 input 原本即拒絕未學習 cast。
- 本輪測試與剩餘邊界見 rank-zero progress 檔。之前裁字沒有修好，不把 rank 功能測試當像素驗收。

## E121：名稱裁切改查實際繪製狀態（2026-10-04）

- run1791100483每隊前後所有matching font batch372 vertices／558 indices／max-relative371／invalid-relative0，排除batch-relative越界；真正backbuffer也仍截字。新增regular Editor opt-in拒絕／shutdown冪等回歸時，裸#endif patch誤命中最前面的宏fallback區；rg檢查立即發現並移出#ifndef，後續patch需完整上下文，未把未執行到的test當通過。

- 真正backbuffer run1791100233四張亦有截字（隊1after完整，其餘截短）；因此「僅Slate截圖重畫」假設不成立，不改正常UI或把capture工具稱為修正。保留真正呈現取證工具及原PNG，繼續檢查GPU input的batch-relative索引範圍，前次僅檢查global vertex range不足以排除錯batch引用。

- 本機SlateApplication::TakeScreenshot會PrivateDrawWindows再重畫，不是正常呈現backbuffer；新增opt-in真正backbuffer readback比較，尚須PNG驗證，不能先稱遊戲UI根因。UE5.8 OnBackBufferReadyToPresent參數是ISlateViewportProvider而非舊FRHITexture，已查本機介面。首編C2440：GetRenderer回傳raw pointer，不是SharedPtr；改非擁有指標與shutdown存活核對，禁止用shared ownership包raw引擎物件。猜SlateRenderTargetRHI.h未找到，实际介面為Engine/Public/Slate/SlateViewportProvider.h。

- D3D12 run1791099494仍截短，不改default或宣稱RHI修好。權威upgrade成功但cleanup_verified=false，事後五PID90268／88156／27924／75128／26092均退出；原失敗報告保留。process.graceful_stop在close-window timeout→stop後沒有等待實際退出，修成再次bounded wait＋assert；補無OS mock回歸，避免拿稍後退出冒充當次cleanup成功。另猜scripts/test_ue_two_team_observation.lua與tools/lua-host/tests未找到，實際測試在scripts/tests，以rg --files定位。

- run1791099179／1791099344：公開GetBatchData等待merge後，名稱rect內索引54／60／60／54，right332／432／525／618，兩隊前後相同；實際batch clip1280×720、stencil0，排除少送出字形／batch clip不足。仍不能以CPU提交冒充GPU像素。下一比較：雙UE launcher一直強制D3D11，而project預設DX12；加入白名單OMOBA_UE_RHI及report欄位，保留原default並明確測D3D12，不更改global engine或放寬視覺驗收。

- glyph診斷run1791098864每字Valid=1、合理XAdvance、同texture0、scale0.6660，排除缺字fallback；未因此宣稱GPU atlas已驗證。嘗試讀cached batch時GetCachedElementDataList是private，編譯失敗，撤掉該callback，不改引擎公開權限；不要只看rg找到宣告就假設public。FontServices.cpp猜名亦未找到，後續以rg內容定位。

- run1791098401：名稱獨立layer64／65／66／67仍截短，撤回無效layer隔離；不再重試同層merge假設。另SBoxPanel.cpp實際在SlateCore而非Slate，後續先rg定位。

- 診斷run1791098159：每隊前後各四個OnPaint的actual name完整、size116×18、cull／scissor均(0,0,1280,720)，名字都layer63；四PNG仍有截字，排除繼承clip不足。依本機ElementBatcher.cpp同layer merge及simple-text批次路徑，改四個名稱分別繪於Layer+1+Slot並回傳該層，與短rank／hint隔離；是小範圍render-batch假設，尚非已確認引擎根因，需真實PNG驗證。不改引擎或全局renderer設定。
- 查SlateBatchData.cpp不存在；用rg --files找到實際SlateRenderBatch.cpp，再由rg定位MergeRenderBatches在ElementBatcher.cpp，不猜檔名。

- 續做19/30的6.2增量，先保留E120全部反例；只在om-upgrade-smoke要求layout診斷時排一次OnPaint記錄，實際name／geometry／cull／GPU clip state／layer／frame一起核對。不再以cached geometry冒充實際paint，不先改width／wrap／volatile／clip override。
- 本輪合併讀長context再被截斷，已按單檔小段補齊，不把截斷當已讀完；後續診斷先build-only與真實雙UE，避免每次猜測都跑整套Editor gate，確定修正後再完整驗收。

## E120：技能升級綁定不能以API／queued代替權威與畫面（2026-10-04）

- 第八輪1791097735四PNG與尺寸／actual text再次核對：自身clip zone仍沒有消除偶發名稱裁切，撤掉這個無效override，不放開parent clipping。queued／rank／SP／raw／三方hash的60Hz增量已成立，但名稱像素驗收保持未完成；後續先取得實際OnPaint clip／draw資料再改，不重复wrap／volatile／clip zone猜測。保留失敗圖片及layout診斷，不把saved verifier資料成功當PNG成功。

- 第七輪1791097384actual name四槽都完整，before／after cached與desired全為116×18；saved verifier明確驗八筆每隊，原始authority仍正確。但PNG仍裁短，排除資料截短與名稱widget尺寸不足，ForceVolatile未解決，撤掉這個無效每frame重繪。下一步名称使用自身已驗116×18的ClipToBoundsWithoutIntersecting clip zone，隔離繼承的clip；不是放開整個HUD clipping或放寬驗收，需第八輪PNG，不宣稱根因。

- 第七輪前建置C2039：STextBlock FArguments沒有IsVolatile；查本機DeclarativeSyntaxSupport.h實際參數ForceVolatile後修正。後續C7732／C2679是此失效chain造成的連鎖，不改SharedPointer或引擎；建置失敗不得啟動舊binary冒充新修正。

- 第六輪1791096909單行模式仍有before與隊1after名稱裁切；不再猜wrap。加入opt-in前後每槽actual name／cached geometry／desired size診斷；僅四個名稱STextBlock設volatile避免重用cached paint，每frame固定四個短label，不是per-entity熱路徑。需第七輪用數據排除文字／layout錯誤，再核對PNG；不宣稱引擎根因或已解決。同檔多個Update段被apply_patch整包拒絕，合併並按檔案順序排列hunks後才重送，未把拒絕當已改。

- 第五輪1791096619四張PNG已人工檢查：after兩隊名稱／rank2／Ctrl提示完整，但before兩隊名稱仍裁短，三block不足以穩定所有畫面。依本機STextBlock.h的SimpleTextMode契約，ASCII content ID、rank／CD與Ctrl提示改單行simple layout，不用shaping／wrap cache；不是已證實引擎根因，需第六輪前後PNG再驗。不要再用Get-Content -First讀單行巨大JSON；本次又截斷，改Lua解析後僅印success／passed／failed。

- 第四輪1791096121操作／提示／rank／SP再次成功，但名字裁短仍出現，相同Text跳過與延後三frame並非完整解法；撤回根因已解決推論。改通用Slate名稱／rank-CD／升級提示三個獨立單行block，GetAbilitySlotText讀實際三個block重組；新增移除hint清除回歸，避免多行layout互相裁切。仍需第五輪PNG，不改玩法或增加deadline。

- 第三輪1791095707仍有隊1PNG名字裁短、隊2完整，固定wrap不是已確認根因／完整解法。檢查通用SetAbilitySlot每個persistent HUD sample無條件SetText；改EqualTo相同值不反覆invalidate，after截圖等三個rank2 presentation frames讓前面frame先繪製。這是呈現穩定化，不是GPU fence／同tick像素原子契約，不修改rank／SP／timeout；需要第四輪視覺QA。

- 第二輪1791095365操作／權威仍成功，提示已可見，但SetText後AutoWrapText的desired width縮小，PNG出現技能名稱裁短。對唯一AbilityTexts block設定與128格內padding一致的MinDesiredWidth／WrapTextAt116，關閉AutoWrap，避免動態文字用自己的舊desired width反覆縮小。仍需第三輪實際PNG，不能只靠GetText判定修好。

- 首輪UE啟動有handled ensure：ConsoleManager.cpp4916明確指出r.Mobile.VirtualTextures已deprecated；project DefaultEngine.ini仍有該False欄位。移除僅此失效欄位，保持既有desktop r.VirtualTextures=True，不修改引擎或擅自新增mobile平台設定；下一次啟動核對無此ensure，不把非致命ensure稱為崩潰。

- 首輪1791094787 launcher與獨立raw verifier成功，兩隊input2各一次、rank1→2／SP3→2；PNG視覺QA卻發現72×72 layout-unit技能格裁掉Ctrl提示及名字，不能以Slate GetText完整冒充像素可讀。改通用128×128格、名稱／rank-CD／Ctrl+實際鍵各一行，保留tooltip；需要重建與第二轮PNG核對，首輪只算資料／操作驗收。

- close_window逾時fallback stop後立即inspect仍可能短暫alive，首次assert非零；追加有界wait再inspect確認70396退出。stop返回不是退出證據，未擴大PID範圍。只讀查尚未啟動的ue-p1.log不存在，後續先看inventory；此輪當時實際在release scripts/core的LTO編譯，不是UE卡死。

- MCP QUIT派發成功但Editor70396在10秒後仍alive，不能當退出；get_editor_dialog明確無modal。以固定Lua host對已核對exe的owned70396發close_window、逾時才fallback stop，單獨inspect確認。合併PowerShell指令中第一個Lua assert失敗不會阻止後一命令，後續驗收需逐command核對；雙UE launcher已在release Cargo編譯中，沒有把首次assert當passed。

- ue_mcp.lua不支援--help（明確非零）；改讀實際CLI與MCP tools schema。--list的保存檔含完整tools，需Lua依實際name過濾後只印需要的schema，不印全部巨大JSON。full流程本次在核對本專案Editor79996後正常關閉失敗，既有10秒fallback僅force該PID子樹，不操作其他Editor。

- 現有Ctrl升級在普通QWER callback內查按鍵狀態；改明確Ctrl FInputChord與獨立共用回呼，不用測試旗標覆寫正式Ctrl判定。delegate驗收不冒充OS鍵盤注入。
- 本輪context混合讀取再次截斷，改小段讀回缺失內容；不得把截斷當已讀完或成功。驗收不得注入SP／rank，queued／ACK不是升級數值結算，需原始資料與UI核對。

## E119：升級门檻不能硬編槽位或只改UI（2026-10-04）

- 收尾：core333／base94最後防呆重跑、full UE／11BP compile1791094033／同Editor79996兩輪17/17／串行PIE與stop全部通過；最終stage e968182a…與codegen --check、OpenSpec strict核對。3902筆KCP60Hz證據是最後開局防呆前的有效內容回歸，不冒充新非法開局網路測試；實體Ctrl／升級HUD像素、rank0仍未驗收。

- 最後開局防呆變更後，build-only被本專案Editor95892阻止stage（exit4），不是編譯成功。改既有full流程，精確核對本專案Editor後關閉／重建／重啟；既有stop在10秒正常關閉失敗後僅force已核對PID及子程序（前次22308、本次95892），不是全域終止其他Editor。測試Editor程序不可用作未保存使用者資產的處理方式。大段合併讀檔再次截斷，後續改限量定位與單檔讀取。

- bridge完整測試51/52通過，單獨再跑仍失敗，排除單純parallel flaky；catalog fixture兩技能max_level4卻無levels，生成失敗導致catalog_id0。補完整fixture，不放寬production契約；完整重跑52通過、1 ignored。最後開局防呆後core333與base94再次通過；UE兩輪17/17與PIE通過，但不是實體Ctrl操作驗收。

- 首次UE build C4018：ABI State.level為u32，Required使用int32比較觸發warnings-as-errors；Required改uint32、Printf用%u，重新完整build。base94已過不代表Unreal已過。

- codegen首次21/24通過，三個舊fixture只有ability ID或max_level4卻只有零／一筆levels，被新一致驗證正確拒絕；補完整四筆fixture而非放寬正式契約。新的rank門檻與升級傷害／冷卻測試仍需跑過。

- shared per-rank required_hero_level預設1保留舊內容；lumen_lance明確[1,6,11,16]，rank1初始學會維持既有垂直流程。必須生成Rust／UE一致資料、驗單調與1..25、權威扣點前檢查，runtime compiled agreement防單邊Lua修改；不冒充整套LoL學習規則。
- 本輪合併context讀取超過exec總輸出上限再次截斷，已分段補讀；PowerShell rg Public/*字面glob錯誤，改直接搜尋目錄。首次patch使用過於泛用e.tombstone anchor把ability validator插在tower emitter，立即rg定位發現，在編譯前移至emit_ability_const；所有patch須核對實際落點。

## E118：技能升級不能只路由input，必須驗權威條件與披露rank（2026-10-04）

- 最終core333／base92／server154／runtime60／bridge52與Fyrox check通過。真實60Hz兩輪upgrade，最終1791091387雙送同request仍input3各一次、3950 wire／IPC snapshots rank／SP／XP／Gold核對與四PID另驗退出。full UE build／stage e08d2a…／11BP compile通過；之後Editor73092正常shutdown，MCP list curl7且inspect已退出，不能說崩潰，也不宣稱Ctrl操作或native automation已驗收。完整5.3未完成，詳見ability-upgrade60Hz進度檔。

- full build剛launch Editor時便呼叫MCP list，curl7端口尚未ready；必須先等om_restart wait-mcp完成再讀schema，啟動中不等於插件失效，不快速重複連線。

- 後续檢查誤猜Fyrox package omobaf，Cargo實際game package是omfx；誤用scripts/lib/process.lua，inventory已指出tools/lua/lib/process.lua。改讀實際位置／manifest再跑，不把exit1當成功。新增升級request有界1024筆去重，同ID重送與衝突不再配置input ID；此為同runtime保障，不宣稱跨runtime重連journal。

- IPC新測試首次失敗不是正式拒絕錯誤：assert_rejected helper固定要求request_id91，零ID案例應明確驗request_id0，已修正重跑。OmGenerated實際位於OmRuntime/Source，不在Plugins/OmGenerated，不能猜路徑。新增Ctrl+Q/W/E/R僅走共用native glue，沒有每英雄C++或Blueprint圖；仍需編譯與真實操作驗收。

- 既有UpgradeAbility handler有SP與Lua max_level，但無Moba phase／pause／roster gate；敵方不披露upgrade input，既有CommittedProgression只更新level／XP／SP，不更新四槽rank，會造成合法升級後hash分歧。新增權威條件與visible-only typed四槽rank事實，不能分享敵方input或用ComponentRepair遮錯。
- 保留四招初始rank1的vertical slice；本輪Lua沒有rank解鎖英雄等級門檻，所以不硬編LoL門檻、不以R前綴猜ultimate。正式網路／UE操作仍待實作驗收，不能以kernel替代。
- 新kernel首次E0425 GamePause不在game_processor import，改用crate::runtime::GamePause完整名稱；rg猜comp/ability.rs與script_registry.rs不存在，改rg --files定位。大段合併context讀取再次截斷，已縮小分段補讀，不能把截斷當完成。
- filtered升級fixture編譯E0283：collect後extend無法推斷集合型別，明確Vec<_>；又猜bridge/src/ipc_runtime.rs不存在，改列rg --files。core333已通過，但base未過前不得聲稱升級整體通過。

## E117：兵線XP不能沿用最近英雄Bounty／浮點單級結算（2026-10-04）

- 最終core332／base89（含15／60／120Hz完整filtered與零repair）／template65／server154／runtime59／bridge51／Fyrox check通過。真實KCP60Hz1791083346與3,883筆raw progression／IPC、戰績／Gold精確核對通過，實際觀測額外兵線XP，四owned PID另驗退出。build-only OmGameEditor、stage e2c3aa…與codegen --check通過；無本輪Unreal XP畫面／MCP驗收。詳見moba-lane-xp60Hz進度檔，整個5.3仍未完成。
- 最末60Hz完整filtered独立再跑2580tick／5158雙隊steps，Finished2566 winner0、雙death1／respawn1、15tick終局凍結與零repair hash全部通過；OpenSpec strict及本輪檔案whitespace检查通過，無待跑測試程序。

- 正式single_lane_creep沒有Bounty，不能假設legacy預設25XP已自動支付。新增Lua lane_creep_xp25／lane_xp_radius1200→compiled rules／full hash，權威tracked creep真實HP0死亡依存活敵隊roster範圍分享，整數floor餘數捨棄；移除tracked unit是一次性結算邊界。正式MobaMatch停用legacy Bounty，不引入兵線Gold；無此resource的舊MOBA／TD保留原行為。
- 新測試首次E0308／E0277：Faction.team_id實際i32，不是lane side u8；fixture helper改為i32，不修改正式schema。E0533：Finished為{winner,tick} struct variant，fixture提供完整值。這些失敗必須修後重跑，不能當passed。
- 大量混合讀檔再次被截斷；只用rg定位與小段讀取，不以截斷內容判定驗收。測試與實戰證據尚在進行，不預填成功。
- 分享測試第二次XP仍12而非37，初步懷疑同tick ordinal重用，但進下一tick仍失敗，排除此假設；不得據此改掉正式ledger。
- 上述ordinal假設不是本fixture根因：Damage只將Death送入next_outcomes，第二次process_outcomes才做正式死亡結算；首次測試直接record_moba_death意外替代Death queue，修為兩次正式process後才驗去重。完整base首次87/89：分享fixture與60Hz完整對局終局斷言失敗，其他兩個fps hash無錯；增加唯一終局診斷定位，不能把未結束或其他winner當完成。
- 60Hz診斷：18401 tick仍Playing、deaths[3,2]／respawns[3,2]，不是hash錯誤。加入XP後fixture Push的AttackMove會在兵線永久換目標；嘗試在可見objective進550距離後鎖AttackTarget並保留同目標active command，不改HP／XP／終局。新增bot回歸首次E0559：HeroCommand實際chase_origin而非網路input queued；改完整正式型別，需重跑。
- 首次鎖objective後60／120Hz完整filtered通過，但15Hz在4601tick仍Playing。Push在接近建築前仍AttackMove清兵；改Push使用正式MoveTo推進、Guard仍AttackMove守線，建築進距離才鎖AttackTarget。這是既有headless objective fixture，不宣稱五位置Bot完成。新Bot測試unwrap失敗因spawn無HeroCommandQueue，fixture須insert預設queue後才設定active，不假設每entity必有component。
- Push MoveTo候選仍造成60Hz逾時，沒有充分證據支持重寫AI；已撤回本輪所有production Bot改動及候選專用測試。生命周期回歸改明確正式input劇本：雙方死亡／重生後守方MoveTo離線、推方保留原Push，所有數值／XP／建築／winner仍正式結算。不得把此fixture當平衡Bot驗收，5.5仍未完成。

## E116：XP結算需權威去重與持久進度，不得以HUD有欄位當完成（2026-10-04）

- 最終core332／base86／template64／server154／runtime59／bridge51及Fyrox check通過；真實KCP60Hz1791081719、3,923筆wire／IPC progression與唯一kill／assist reward精確核對成功，四owned PID退出。最末Unreal build-only／stage d918c4…／codegen check及OpenSpec strict通過，沒有本輪Unreal XP實戰畫面／Editor測試，不引用上輪驗收。詳見moba-kill-assist-xp60Hz進度檔。

- Lua新增hero kill／assist XP，compiled rules與full canonical hash一致；runtime改值需重建所有peers，不接受dev hot reload。只接既有正傷害／唯一生命／合法歸屬結算，死亡助攻者存slot進度、重生恢復；沒有兵線XP或技能升級輸入驗收。
- MOBA獨立整數升級方法有多級迴圈、25級上限、飽和與零XP保護，舊TD add_experience保持不變。活體只加HP上限／攻擊成長delta、保留裝備、不補HP或復活；filtered只收既有CommittedProgression與EquipmentStats，不自行重付XP。
- 首次base測試编譯失敗E0616：跨crate試圖直接讀MobaHeroSlot.hero私有欄位。移除此直接斷言，不擴大正式可見性；在正式死亡／重生後讀ECS Hero經驗驗持久值。
- template首次不帶runtime-lua-content測試執行0項（該模組受feature gate），不能聲稱新回歸通過；改明確feature再跑並核對非零測試數。大型合併讀檔再次截斷，後續分段限量读取。
- XP成長改變戰鬥時序後，15Hz完整filtered在2399出現owner lumen_bolt冷卻差異（60／120Hz當輪通過）；診斷顯示當tick沒有新accepted input且NPC已被PreStep Forget，需檢查queued cast與實際HP。不得覆寫owner CD或ComponentRepair。暫時diagnostic首次patch又命中較早相同context，立即rg定位移到唯一lifecycle subject行；查找又用Windows path wildcard及猜不存在dispatch路徑，改rg --files。失敗不掩蓋，修正後需重新驗證。
- 根因為通用效果GameWorldDyn::is_alive只有ECS entity存活，HP0／pending-deletion仍可能queued cast成功起CD；改同時要求get_hp>0，新增真實adapter回歸。15／60／120Hz完整lifecycle與86個base tests重跑全部通過，診斷暫碼移除，owner CD與hash gate未放寬。
- 真實KCP XP verifier首次錯誤要求Warmup tick35也有CommittedProgression（正式inactive只保留bootstrap／persistent HUD，並不發active結算）；改只在Playing逐tick核對raw progression，所有live樣本仍依score與Lua reward核對數值、並要求killer／assistant非零reward實際觀測。不得因0 tests／只看report宣稱raw verifier成功。

## E115：非零死亡戰績畫面不得以開局零分或注入資料替代（2026-10-04）

- 正式60Hz雙UE run1791080545成功：兩隊各4技能、死亡／重生／Finished10028、Victory／Defeat與非零死亡計分板PNG已檢視，20,562筆IPC逐tick對authority及初始／死亡UI五欄完全一致。三方hash169／167 PASS rows至10200；五owned PID獨立確認退出。詳見unreal-scoreboard-death60Hz進度檔。
- 收尾full build／11BP gate／同Editor兩輪15/15／串行PIE全部通過；MCP保存後正常關閉owned Editor5380，六程序另驗退出。最後build-only／stage d7dd01…／codegen --check／OpenSpec strict驗證通過，無未完成測試程序。
- 本輪收尾查找又猜不存在的scripts/test_ue_moba.lua，rg明確exit1；改rg --files scripts定位ue_native_visual_smoke.lua／ue_pie_smoke.lua。不得把同一呼叫內其他成功輸出當成此查找成功；未修改遊戲來源。

- 本輪增加獨立death opt-in，依Playing、真實owner bAlive=false、supported Deaths>0及原生viewport layout就緒截圖；兩隊都要自己的死亡數>0，原始wire與IPC精確比對。NPC造成的死亡不冒充玩家kill／assist畫面。
- 初始與death使用不同PNG；首次截圖request後立即return，避免同一frame後續request覆蓋前者。這只保護測試request，不提供GPU fence或同tick像素原子契約。
- 沿用正式四技能／match lifecycle測試到死亡重生、基地勝負與結算，核對後120 ticks三方hash。不更改timeout掩蓋失敗、不加入角色C++或BP graph。

## E114：計分板畫面驗收不能以資料更新 log 代替實際 viewport（2026-10-04）

- 最終雙UE60Hz1791079800兩张1280×720PNG已檢視、6786筆IPC對authority逐tick一致與UI列精確對照；雙隊54 PASS rows至3240、六owned PID退出。Lua nested重現／module、9項觀測器、runtime59／bridge51+2、11BP recovery、兩輪15/15與串行PIE／最後build-only及stage一致通過。shader準備時FPS21／23，不宣稱穩定60FPS；初始0KDA不冒充擊殺更新或十人完整對局。詳見unreal-scoreboard-ui60Hz進度檔。

- 收尾full build的Blueprint gate compile-1791080124失敗，不是BP編譯錯誤：父子Lua同秒且random重複，host request檔名撞到仍使用中的檔案，path.write安全拒絕覆寫。改lfs.mkdir原子保留獨立owned request目錄、有界retry，不移除／覆寫其他程序檔案；強制父子相同time／random加入重現回歸。保留失敗report，不藉重跑掩蓋根因。
- 新增nested回歸第一次失敗是test既有root由source suffix推導，relative `tools/lua/tests/run.lua`沒有前置separator，child exe落到錯誤目錄；改用path.repo_root正式解析。修patch時首次錯用轉義context被apply_patch拒絕（未變更檔案），改唯一require行作context。不得以shell command最末exit0判定前面tests成功，需逐項明確核對。

- 新增非Shipping且明確flag的原生截圖 gate：Playing／tick至少120、IsInViewport、面板可見、實際cached geometry非零，再記錄UI資料並要求PNG存在。只有測試模式截圖，不加入角色graph或玩法入口。
- SetScoreboardState不能在值相同時直接return，否則首次資料先於layout，後續相同KDA永遠無法驗證；只節流文字更新，獨立重試layout就緒。
- 1v1 launcher觀測器只驗固定fixture的兩名玩家／兩隊列、配置身分、完整KDA與UI後120 ticks三方hash；不是十人或非零擊殺更新證據。後續需要原始IPC與畫面逐樣本核對。
- 本輪查找誤將截圖機制放在PlayerController搜尋（實際在共用Widget），以及大型OpenSpec合併輸出截斷；改rg整個Source與分段讀檔。不新增替代workflow。

## E113：公開計分板不得依可見英雄推算或放寬私人 HUD audience（2026-10-04）

- 最終真實三runtime1791079034通過一kill／assist／death，3955筆wire／IPC戰績板逐tick一致；core330／base84／server154／runtime59+3／bridge51+2／Fyrox與完整UE建置、11BP、兩輪15/15、串行PIE、最後build-only及codegen check通過。report cleanup與六個owned PID獨立核對退出，尚無多人計分板實戰像素驗收；詳見public-scoreboard60Hz進度檔。

- 本輪又猜錯OmGenerated plugin位置，改rg --files定位OmRuntime/Source/OmGenerated。新增clear保護首次patch只用常見entity loop context而命中前面的函式；立即以rg核對並移至PublishAbilityHudState之後，未建置錯誤版本。後續patch必須使用唯一鄰接context。
- 完整presentation snapshot缺少有效MOBA HUD時明確清除原生HUD；control-only frame不清除持久畫面。計分板放寬700px並允許換行，避免完整u32數值的十人欄位被固定420px截斷；尚須區分自動化widget驗收與實際多人畫面證據。

- 決定另設公開count與team／K／D／A namespace，AllPlayers僅包含持久玩家資料；原owner score／Gold／Recall仍維持隊伍隔離。不傳位置、entity reference、物品、助攻參與帳本；名單不依存活或視野推算。
- runtime atomic完整列驗證：2–10人、兩隊各最多5人、完整四metrics、count一致、唯一player、非零team及u32值、固定team/player排序；缺少／重複／錯誤資料顯示unsupported，不补零。bridge再核對owner team與私人KDA一致。
- ABI固定十列frame-owned array，升9並拒絕8。Unreal int64完整容納u32，唯讀原生panel只在有效快照显示，Stop／非MOBA清除；無角色C++或BP graph。OpenSpec大段合併輸出截斷已分段補讀。

## E112：網路擊殺／助攻驗收不可以 Move 或 NPC 死亡替代（2026-10-04）

- 最後TD1–100 integration三項通過（228.45秒，headless／observed完整hash與ledger相同）；Lua launcher最終語法與60Hz-only profile gate檢查通過。沒有留下待完成測試程序。
- 最終1791078192真實KCP60Hz成功：一kill／一assist／一death，三人各2post-kill checkpoints，3829筆wire／IPC score及收入＋300kill／100assist逐筆精確驗證。12個owned PID獨立核對退出；core330／base84／server154／runtime56+3／bridge51+2／Fyrox與build-only、stage72abf9…／codegen check通過。未做本輪Unreal多人畫面或Editor測試；詳見network-assist60Hz進度檔。
- run1791077851正式兩個AttackTarget均套用，但player1／victim重合，hero_tick把整個攻擊windup／impact包在距離平方>0.01的方向判斷內，合法近零距離攻擊永不發射。修正為零距離沿當前facing、正常啟動攻擊；非零方向維持atan2／轉向／範圍／敵我／HP gate。新增方向回歸並維持原重合網路fixture重驗，不靠拉開站位避開bug。保留失敗report及四程序cleanup。
- 查找時又猜不存在attack_tick.rs及run timestamp，改使用rg --files與列出實際evidence目錄；不得推算路徑再讀檔。
- 首次建置1791077835被rustc拒絕：MoveToIntent沒有queued欄位（只有AttackTargetIntent具有queued）。移除不屬於schema的欄位；未啟動任何遊戲程序，保留失敗report，不新增替代命令。
- 本輪加入獨立 opt-in 三玩家戰鬥 fixture，只用披露資料提交普通 MoveTo／AttackTarget，經既有 renderer intent queue、輸入驗證與 KCP；不注入傷害／Gold／計分，不稱為 Unreal 或 TCP renderer 操作。
- 驗收要求同隊兩人各提交可見敵方普攻、一擊殺加一助攻及 victim 一死亡，結算後至少240 ticks持續進行並核對三方hash。擊殺者不依執行緒時序寫死。
- 又使用 Windows rg path wildcard 而失敗：後續固定目錄加 --glob，禁止把 *.rs 作檔案路徑；查找錯誤不影響來源檔案。

## E111：K／D／A 必須讀傷害與死亡後的唯一權威樣本，不可取同 tick 的舊值（2026-10-04）

- 最終驗證：三runtime1791077095原始wire／IPC共3324筆KDA一致；runtime56+3／bridge51+2再次通過，UE full build／11BP gate／兩輪14/14與最終build-only、stage7a4d11…／codegen check通過。未把既有art PIE當作KDA實戰畫面；詳見owner-score60Hz進度檔。
- capture逐筆比較首次使用相同tick而失敗：SelectiveReplica apply後設定world.tick = frame.replica_tick + 1，render snapshot使用world.tick。因此明確以snapshot tick減1對照來源frame；不使用最近樣本／最大值／容錯範圍掩蓋差異。
- 三runtime capture首驗錯誤假定Move-only就沒有死亡：cleanup期間server仍進行，team2遭NPC合法擊殺，真實HUD為0/1/0。不能把合法死亡視為失敗或只檢查零值；verifier改逐tick原始TeamTickFrame player-scoped權威metric與IPC score精確比較，並核對所有score metrics隊伍隔離，不重跑／改寫原始交易或capture。

- 首次三英雄640tick計分回歸在player3死亡的step508失敗：PreStep／PostStep各發布同metric，renderer的first-match reader取到死亡前0。改active tick只PostStep結算後發布，warmup／pause／finished inactive tick保留單一PreStep；不改reader任意取最大值，不由Unreal補算死亡。
- K／D／A 以namespace＋完整player ID、team audience持久發布，不傳assist ledger／enemy Gold／隱藏entity。IPC optional score缺少／負值／overflow代表unsupported，絕不假造0/0/0；bridge核對score.player_id與HUD/configured owner。C ABI結構擴充必須升8並重建／stage，明確拒絕7。
- Unreal UPROPERTY以int64容納u32完整範圍，原生formatter拒絕負值及超量；死亡時仍顯示KDA。UI使用共用native欄位，無角色專屬程式／Blueprint graph。
- 再次以PowerShell路徑wildcard搜尋及猜disclosed_world.rs／disclosed_stepper.rs失敗：改rg目錄＋--glob並先rg --files。此為查找錯誤，不用shell fallback或額外workflow繞過固定Lua。

## E110：多人 roster 不可拿玩家索引當兵線側；測試必須尊重傷害與死亡的分階段結算（2026-10-04）

- 最後版三runtime1791076498亦通過：three-runtime明確metadata、雙隊9／第三玩家8 checkpoints至1080、safe1092；四PID86380／89648／8092／64972清理與獨立inspect通過。未把網路移動驗收當作網路助攻／多人UE驗收。

- 驗證：base84／core329＋TD1–100 integration／server154／runtime55+3／bridge50+2／Fyrox通過，build-only28589與ABI7／stage39f10b…一致。真實60Hz三runtime1791076242第三玩家獨立8checkpoints至1080；Recall2回歸1791076358兩隊各10 checkpoints至1200／基地完成，四＋三PID均核對退出。原始三runtime報告kind曾沿用two label，已修opt-in metadata，保留原證據不回寫。詳見roster-assist60Hz進度檔，完整5.3不勾選。

- 原本 Recall／respawn HUD 以 `(team, metric)` 作鍵，同隊多人會覆寫；filtered active Recall 更會清整隊命令。改為三個獨立 namespace 加完整 u32 player ID，filtered 核對 `(team, player)`，runtime 僅讀配置玩家；Recall 協商升 version 2，舊 version 1 明確拒絕。
- roster 採保留初始兩名玩家加 explicit additional_players，英雄 slot 保存 side；基地、shop、死亡、重生、助攻、收入與 fixture Bot 均分開 slot 與 side。每隊最多五人，所有玩家 ID／team／hero 必須在 spawn 前驗證，未知、重複、零值與超量不得留下半場景。
- 首次新測試失敗不是結算漏掉死亡：Damage 產生下一批 Death，單次 process_outcomes 尚未退休 victim。比照正式分階段流程再處理 queued outcomes，不放寬死亡斷言。
- 首次三英雄 Recall fixture 將敵人擺在普攻範圍內，合法正傷害取消回城，造成 active=0；改擺在視野內但普攻範圍外，保留全部傷害取消規則，不禁用戰鬥或偽造 active。修正後 640-tick／1280 雙隊步驟通過，零 ComponentRepair，完整 bootstrap hash 一致。
- 本輪再次遇到合併文檔輸出截斷與不存在的 runtime.rs 搜尋路徑；已分段補讀並使用實際 native.rs 匯出。後續先 rg --files，再讀檔，避免依記憶猜路徑。

## E109：雙人單路不等於多人助攻，匿名傷害也不能推導攻擊者（2026-10-04）

- 驗證：core329／base81／template63／server153及runtime54+3／bridge50／Fyrox check通過；KCP1791075055真實60Hz兩隊9 checkpoints至1080與3程序清理。build-only60465／ABI7／stage cfc40ab…成功，無助攻UI或多人對局驗收，5.3保持未完成。

- 先檢查實際MobaMatch：固定heroes[2]、config.players[2]且teams不同，目前不存在同隊第二名英雄。決定先交付authority assist ledger與damage hooks，純核心三／四人fixtures不冒充真實多人對局；後續必須一起擴充roster、private recall／HUD metrics與安全投影。
- 助攻採原創規則：Lua100 Gold、10 active seconds，實際正傷害才記錄最新時間；同隊英雄擊殺時每名有效參與者一次固定獎勵，排除killer／self／friendly／零傷害／過期，恰好窗口端點有效。只認目前roster英雄entity，不推導召喚物／退休entity归屬，匿名ScriptDirectDamage不發助攻。
- ScriptDirectDamage沒有source欄位，也未走既有kill hook；若匿名致死後heal再被hero致死，不能補搶獎勵。新增共同hook使匿名致死退休該生命參與記錄與lethal_pending，不擴ABI或虛構source。
- 搜尋再次猜native/mod.rs與omb/server/src不存在；應用rg --files與實際native.rs／omb crate來源，不延用路徑猜測。上下文過長的輸出已分段補讀，不當作完整內容。

## E108：UE B 綁定驗收必須連到原始 ACK，不得使用 runtime 內部注入（2026-10-04）

- 最終full build1195／Editor36528：11BP與兩輪13native＋PIE通過，SaveAll後正常關閉且inspect退出；raw capture test來源變動令bridge重新連結，最終stage057c643…，雙UE原始證據stage382dea…分開註明。再次scripts/ue*.lua失誤仍改用rg scripts --glob 'ue*.lua'，不交給PowerShell展開。

- 最終1791073998雙UE60Hz通過：Recall2／Move3取消／Recall4基地完成，原生HUD四PNG實看完整，兩隊各66 hash PASS至3960；protobuf4085／4063 snapshots均取消124ticks、完成479ticks。saved verifier重讀原始logs/config/PNG/capture與5PID無殘留通過，不把物理鍵盤或整項5.3／6.2標完成。

- interactive-ue-1791073690 gameplay成功但實際PNG顯示 Recall文字超出右側：原生match行改兩行、小字與wrap，必須再跑截圖。saved checker用文字模式讀PNG使CRLF被Windows轉換，signature誤判；改rb讀原始檔，不改PNG或放寬signature。

- 真實首跑 interactive-ue-1791073582 在 runtime 載入舊 release base_content.dll 時 abort：未知 recall_channel_seconds（跨 FFI panic）。雙玩家 launcher 原本只 build server/runtime，漏 scripts workspace；改為同 profile 先 build base_content，不使用 skip-build 掩蓋相依性。失敗報告 cleanup_verified=true，server58292/runtime98856 已退出。

- 本輪新增 opt-in 雙段回城 fixture：B 綁定 delegate →正式 Recall→移動取消→第二次 B→權威基地傳送；不改 HP、位置或時間。此入口不等於實體鍵盤／OS 事件。
- UE5.8 本地 InputComponent.h 證實 KeyDelegate.Execute 要傳 FKey；修正 Execute(EKeys::B)，不依賴猜測的無參數介面。
- 小地圖未就緒只能等待測試的初次移動，不能從 ProcessFrame return 而丟掉 HUD／entity 處理。移入前置條件。
- 路徑搜尋再次誤猜 OmGenerated 插件與 omoba_bridge.h；應先 rg --files，實際皆在 OmRuntime 插件，FFI header 是 ThirdParty/OmBridge/include/om_bridge.h。
- launcher 明確關閉 OMOBA_RECALL_SMOKE 的 runtime 內部注入，保留原始 input ID／owner／command kind／status、原生 HUD 文字及 PNG、完成後 120 ticks 三方 hash。未跑完雙 UE 不宣稱驗收成功。

## E107：回城 acceptance 不等於讀條成立；filtered 舊命令必須權威清除（2026-10-04）

- 最終增量：KCP1791072423雙隊正式Recall input2／8秒基地傳送，11／10 checkpoints至1320、cleanup通過；這是runtime內部renderer-intent注入，不是真實UE按鍵／socket。UE C ABI7、通用B鍵／HUD full63411／stage382dea…、11BP gate、同Editor兩輪13/13／PIE59543通過，SaveAll後48060正常WM_CLOSE退出。UE實際B鍵對局仍未封關；完整進度見moba-recall-network-60hz-progress。
- Lua helper搜尋已顯示實際tools/lua/lib/process.lua，卻又用了scripts/lib/process.lua而missing；已按實際路徑讀取並使用固定Lua／lua-host正常關閉Editor，不添加shell fallback。
- 最後補MD的合併patch錯放不存在的design「##」context，整批驗證失敗未落檔；移除猜測context重新套用兩個已讀文件，再讀design尾段才更新，不能把apply失敗當成功。

- 真實run1791072213失敗保留：team1完成tick993，team2 tick570 active後579取消；原smoke把正常取消回傳Ipc error導致重複rebase，是測試器錯誤。改記canceled／stage255，不污染正常frame apply；完成fixture正式移動往自己基地外側／離開兵線，而非穿越兵線後假設不會受傷。三程序cleanup_verified=true。
- Fyrox package猜成game導致package ID不符；讀Cargo.toml後改-p omfx並check --tests通過。codegen模板資料夾不存在的搜尋同樣先rg --files再定位，不硬猜。

- 發現：authority 清 HeroCommandQueue／MoveTarget，而 filtered 的 Recall branch 是 no-op；只測靜止英雄會漏掉傳送後繼續舊命令的問題。改以 owner-team PreStep 回城 active HUD fact 清除命令，不能對所有 accepted Recall 無條件清除，因暖場／同批移動可能拒絕回城。PostStep remaining 在傷害／死亡／終局結算後更新。
- 協商：獨立 Recall protocol1＋完整 Lua rules hash，legacy0/empty閉合；server 僅 authenticated selective player＋SingleLane 開放，不沿用 shop gate。IPC RecallIntent 使用 tag18：tag17 已被 AttackTargetIntent 使用，先查 schema 不沿用初步猜測。
- 搜尋失誤：PowerShell 傳給 rg 的路徑 wildcard（accepted_input*／team*）不是合法 resolved path；改用資料夾與 --glob。再次猜 hero_tick.rs 根目錄造成 missing path，應先 rg --files 找真正 tick 目錄。
- 驗證新增移動→正式 Recall acceptance→8秒回城的525tick雙隊filtered／owner-private metrics／零repair hash，以及 capability／owner／epoch／zero-ID gate。真實網路與UE的結果分開記錄，不以單元測試冒充 B 鍵成功。

## E106：回城必須在傷害結算後完成，且不能沿用未接妥的網路入口（2026-10-04）

- 此增量驗證：base79／template62／core323／server152＋1ignored／runtime52＋3 passed與3ignored、Fyrox check --tests通過；build85353／stage c82bbf…、codegen --check／OpenSpec strict通過。真實60Hz KCP1791071189双隊9 checkpoints及三程序清理通過，不是UE回城驗收。完整證據見moba-recall-60hz-progress。

- Fyrox相容check也發現E0004（game/src/native.rs的input分類漏新Recall）；保留舊前端，不把新共享enum只修Unreal端。分類顯式補齊後重驗，不做cleanup或commit。

- server編譯E0004：新增oneof後authority canonical acceptance的完整match漏Recall。補明確return None（未協商前拒絕），不以虛構action_kind繞過safe projection；必須同步檢查server與generated fallback，不能只跑core。

- 邊界補強：同batch任一移動／攻擊／施法／物品使用意圖優先取消Recall，不讓尚未dispatch的早先命令藏在讀條後執行。另有ScriptDirectDamage直接改HP而不經handle_damage，故補同一中斷hook與正向測試；零傷害不取消。停自動攻擊也以既有safe MovementPriority投影，filtered world不持有MobaMatch。

- template回歸發現10個舊TD-only fixtures沒有moba_economy：derive Default把新秒數設0，1..60驗證先攔掉舊內容。修正為整個section省略時的相容Default（不發經濟獎勵、8秒有效預設）；有宣告section仍要求完整型別欄位，正式Lua明列8秒。不能為新MOBA規則破壞舊TD載入。

- 決定：獨立Recall tag19，Lua整數1..60秒生成規則、compiled與hot-reload agreement；權威開始時清命令與MoveTarget、停自動攻擊，post-step在傷害／死亡後傳送，正傷害／移動／攻擊／施法／物品使用取消；pause不推進。secure入口暫拒絕Recall，不能把authority kernel當UE已可按B。
- 編譯錯誤：測試猜Vec2::from_i32而API是Vec2::new(Fixed64,Fixed64)，並錯用enum variant的struct update syntax（E0599／E0436）；修正為實際constructor與完整Outcome::Damage欄位。不再用不支援的便利API猜測。
- 搜尋錯誤：再次猜comp/pos.rs、tattack.rs、attack.rs造成missing path；實際先用rg目錄找型別，避免靠命名猜來源。
- Context輸出過長被截斷：改分段重新讀取OpenSpec tasks，不把被截斷內容視為已完整讀取。

## E105：擊殺不能由死亡通知／ACK 發錢；Cargo 與來源路徑不可猜測（2026-10-04）

- 最終生命去重版本封關：base77／core323再跑成功、NPC夾Heal分支通過、build89587／stage424449…一致。真實60Hz1791070433雙隊9 checkpoints、三程序退出且CIM無殘留；非UE擊殺画面或助攻驗收。

- 最後檢查新增同batch lethal→Heal→lethal邊界：HP正→零本身不足以代表新的生命，死亡還未刪除entity。增加hero slot lethal_pending（含replay digest），首次致死先標記、只有spawn新生命清除；即使首筆是NPC擊殺也不能後續hero搶已排定死亡的獎勵。新增治療夾傷害／重生後第二次擊殺測試。

- 驗證：base77／core323／template61 passed、60Hz完整filtered與125tick擊殺投影通過，build-only16479／stage、真實60Hz KCP1791070108雙隊9 checkpoints與清理通過；詳見2026-10-04-moba-hero-kill-60hz-progress.md。
- MD更新第一次apply_patch因猜測design context不符而整批失敗；確認新檔未建立／tasks未更新後，拆除無效context重套。讀到設計原文後才補design，不依片段猜行。

- 發現 runtime smoke 的非商店分支沿用 game.toml 的120Hz，只有商店分支改60。第一次啟動87440保留為120Hz相容性診斷，不當60Hz證據；修正為所有模式預設60且明確報告tick_rate_hz，原OMOBA_SHOP_SMOKE_FPS保留、新OMOBA_MOBA_SMOKE_FPS可覆寫。不改使用者game.toml。

- 新增投影測試第一次編譯 E0609：錯把事件放在 PostStep.events；實際 transport 是 Step.public_events，PostStep 只含 repair/hash 等結算欄位。已依既有商店投影測試修正，不能靠欄位名稱猜測 protocol。

- 設計檢查：同 tick 多筆 Damage 可在 Death 刪除實體前命中零 HP，若在 Death 或每次 lethal 判斷發錢會重付。決定只在權威正 HP → 零 HP 的正傷害首次轉換呼叫 credit；只接受當前雙英雄 entity 的敵方直接擊殺，排除自己／NPC／建築／非 Playing／pause，Gold 與 kill counter 飽和。
- 規則：Lua hero_kill_gold 生成 Rust 常數；compiled/runtime 與 dev reload compatibility 同步檢查。不得只更新 Lua 後使用舊 bridge／host 驗收。
- 操作錯誤：搜尋了不存在的 runtime/simulation.rs、implementation-error-register.md、根 Cargo.toml；實際是 native/simulation_driver.rs、此檔，根目錄沒有 Cargo workspace。後續先 rg --files 確认實際路徑。
- Cargo 錯誤：從 omb workspace 對外部 package 指定 features 被拒絕（cannot specify features for packages outside of workspace）。已改用 cargo test --manifest-path omoba-template-ids/Cargo.toml --features runtime-lua-content；不得把 dependency package 視作 workspace member。
- 邊界：召喚物／舊 entity 持續傷害歸屬、助攻、擊殺 UI 與多人規則尚未完成；此增量不勾選 5.3。

## E104：Blueprint 編譯失敗不能被 transport 成功掩蓋（2026-10-04）

- 封關：live fixture1791069017三種create／兩次3asset save／8次compile包含真實error_count1與原asset修復；獨立saved verifier重讀raw通過。default gate1791069249双英雄＋九UMG11個全部error_count0、無created；full18805／MCP gate、Editor10844最終兩輪13/13／PIE87395通過。17個正負向測試通過，3.2完成、19/30；沒有改角色graph或藉空Widget宣稱完整UI完成。

- Editor fixture49320返回running session後，未先取completion就提交九個UI compile；兩者最終都成功，但後續必須串行確認前一個mutable Editor workflow已exit0。此點不推成工具互斥保證；後續automation20057先確認exit0、再關Editor與重新full建置，不在PIE／重啟中交錯資產工作。

- 預設recipe gate首次拒絕native-only英雄的空blueprint_path：Lua空字串仍為truthy。改明確略過空路徑，實際有路徑者仍完整驗證；不是替native-only角色新增空Blueprint來繞過錯誤。

- 刻意負向案例：/Game/OmAutomation/BlueprintValidation_1791069000/BP_CompileBoundary 的 Actor self 呼叫 Character.Jump；graph build成功，但真正compile_blueprint為isError=true、ok=false、error_count1，診斷為self不是Character、Target必須接線。不能只看build ok或HTTP成功。
- 已只移除本輪owned fixture的invalid_jump節點並重編譯，error_count0、graph intent僅BeginPlay空body，沒有更改既有英雄graph。建立可重跑fixture與共用compile gate，必須保留asset／raw compiler／diagnostics／pending；transport／missing structured response／health issue／asset mismatch均拒絕。
- 決定沿用原生Slate，不為MOBA新增不必要UMG。必要舊Blueprint／UMG相容資產仍需compile驗證；WidgetBlueprint建立能力只在獨立fixture測試，不把空測試widget當完整遊戲UI。

## E103：真實 Unreal renderer 重連驗收（2026-10-04）

- 正式封關run1791068322（啟動前無Editor）exit0／cleanup_verified：renderer42712→43836正常關閉重啟，backend／對侧UE90208／84812／48392／75400不變；fresh Playing4859、actualrate60、Consumed870→4901、ownHUD／economy／movement恢復。新minimap renderer input1對應authority allocator input2／tick6511／target raw491520,294912，獨立保存verifierexit0。gate兩隊114／113 PASS至6840、各兩個post-input unique checkpoints，零FAIL。六PID與Editor92496查無；完整4.3／LAN／全cue／60FPS仍未宣稱完成，18/30不變。

- Editor92496 MCP QUIT dispatch 回覆成功，但 process.wait(30秒)失敗；錯誤地在 wait shell 還是 running session 時先啟動了 run1791068180。readback證實 Editor 仍在且無 modal，固定Lua針對該已保存Editor WM_CLOSE／wait10秒後正常退出。此run有短暫Editor重疊，僅保留作診斷；正式結果必須在確認wait exit0與Editor不存在後重跑，不能以啟動命令沒有輸出就當作完成。

- MCP get_tool_docs 傳 tool_name=execute_console_command 被工具視為 category 名並拒絕（尚未執行 console command）；改先 search_tools 取得真正名稱／schema，再按 discovery 呼叫。不能把 doc lookup 失敗當作 Editor QUIT 已完成。

- 新增 observation 負向測試首次失敗：Lua gsub 的第二個回傳值 count 被測試包裝函式當成 runtime_tail，導致 number:match 錯誤。測試呼叫以括號強制單一字串回傳；不是放寬 production 驗收或忽略失敗。後續此類字串變造 helper 明確截斷 multiple returns。

- 排查時猜測 scripts/lib/process.lua 與 OmRuntime/Private/OmWorldBridgeActor.cpp，實際分別為 tools/lua/lib/process.lua、OmGenerated/Private/OmWorldBridgeActor.cpp；PowerShell 呼叫 rg 時 literal *.cpp 路徑也失敗。後續先 rg --files 定位，使用目錄搭配 -g 過濾，不依據模組名猜檔案。
- 決定以 opt-in、有界 single_lane 60Hz 測試維持同一 server／雙 runtime／對側 Unreal，只正常關閉並重啟 team1 renderer。重啟使用獨立 logs／UserDir，原始證據不混入新 session；使用不同的公開小地圖 Point Move，避免同一舊目標冒充恢復輸入。只測 renderer 重連，不把 Rust runtime 重啟／兩台 LAN／全部 cue 當成已完成。
- 驗收必須讀實際 bridge diagnostics 的 tick rate，而不是 launcher 配置；HUD／economy／Consumed 必須在新 renderer 出現，正式輸入原 ID status0 與重連後三方 hash 才算通過。結果待實測，不勾選完整 4.3。

## E102：美術替換必須驗證真正匯入，並能還原（2026-10-04）

- 最後原版60Hz回歸run1791067086通過：原240秒／release，雙4/4、死亡重生10／9、原input8指定攻擊ACK、winner2、Finished12837／UI12838，actual Defeat／Victory與60Hz PNG已檢視；保存verifier獨立exit0，hash216／215 PASS rows／108 unique ticks各至12960、0 FAIL，UNVERIFIED25902／25900獨立不算PASS。五PID94372／3556／88660／78668／63348查無。來源b3ccc...已還原、Lua override已移除、最終stage cc76ef2d8328126a7ca255de84fa49daf72272af426ba9a0b40f8ceb8871d709；6.3封關，整體18/30，未宣稱全框架完成。

- art-swap-1791066725完成：256→160→256 Texture2D readback，actual PIE scale0.9且rendered，三張PNG已直接檢視；替換／還原各import2 jobs，所有repeat0 imports／0bindings／22packages SHA不變，Source C++與RustBP packages hash不變；來源binary已精確還原，保存verifier独立exit0。原版配置重建57914／UBT5.77秒、Editor85496兩輪13/13／PIE18379 actual scale1.2通過，content hash已恢復。6.3代表性流程通過，不代表任意Skeleton／全部動畫品質通過。
- 本次還原Save／QUIT後立即build57914，QUIT是async dispatch，restart一開始仍見49928並發taskkill，但隨後正常退出，force_terminated=false。防重犯修正：最後Save／QUIT後用固定Lua process.wait(PID,30000)確認85496真正退出，再啟動正式回歸；工具收到dispatch成功不等於程序已退出。

- 第二次build2110前未先MCP QUIT：restart對本輪只讀查詢的Editor62308正常taskkill／10秒等待失敗，後續精確project scope強制關閉才成功，未做資產改動但不應依賴force作正常收尾。後續variant／原版重建先確認PIE停、MCP Save／QUIT並查PID退出，避免強制關閉未保存Editor。build2110成功（UBT9.40秒），variant生成內容与bridge SHA已改，原版最後必須重建還原，不能留混版。

- 既有Editor測試與PIE腳本把Saika縮放固定1.2，會將合法Lua native_visual override誤判失敗。共用Editor fixture改比對生成CDO的NativeMeshTransform；PIE期望值讀同一Lua配置／meter→cm邊界，保留數值斷言，不刪驗收。角色runtime C++／Blueprint graph不修改。測試配置scale0.9／yaw-45，最後移除並重建原版。

- 排查時先猜templates/heroes與omfx/data/templates/heroes，兩路徑均不存在；生成配方已明確列scripts/lua_data/templates/heroes。後續以recipe source為準，不靠renderer舊路徑推測來源。
- 決定先驗6.3現有原生英雄貼圖替換：只操作已由ledger持有的RecipeV1資產與來源binary，保存原始來源備份／hash，替換後讀回Texture2D與PIE畫面，再還原來源／配方。不能只以imported count或save成功證明美術生效；不改角色C++／Blueprint graph，不刪原資產，未完成全部驗證前不勾選。

## E101：presentation diagnostics 不可用預設 120Hz 換算 60Hz 對局（2026-10-04）

- 最後capture逐筆9451／9429 snapshots owner HUD／公開live hero射程各通過，verify-staged-only保持03aa0c4d804ad706ecb2815366a817566776fdac2869df01c4d9ac9bea729098，scoped diff --check通過；既有warning未被關閉或冒充修正。

- 最後驗證run1791065966：release60Hz、原240秒、success=true／cleanup_verified=true，winner1、Finished9165、UI9166、雙技能4/4與死亡重生5／6；保存verifier獨立exit0。已直接檢視兩1280x720 PNG，頂端60Hz／tick時間152.8，中央權威Playing elapsed152.7，沒有?Hz或假120Hz。每隊156 PASS rows／78 unique ticks至9360、0 FAIL；UNVERIFIED18748／18739獨立，不算PASS。五PID28712／92908／87600／88404／84972查無。metadata bug完成封關，但不宣稱全框架或持續frame-time60FPS完成。

- 持久ready版本：runtime52+3／bridge50+2 passed，實際TCP晚連線、重連各先ready再snapshot，wrong-team無metadata；fullbuild14047 exit0／UBT2.25秒／stage03aa0c4d804ad706ecb2815366a817566776fdac2869df01c4d9ac9bea729098，Editor91056兩輪13/13、PIE95327含截圖exit0，正常Save／QUIT且PID查無。正式新版驗證中，僅這些測試不足以宣稱實際PNG rate已修正。

- 新版run1791065418完整60Hz對局成功，UI13504／Finished13503／winner2，雙队hash224 PASS rows至13680，PNG已檢視；但頂端仍?Hz／Time--，不能宣稱metadata修正完成。根因ready_envelope經publish_latest被snapshot watch覆蓋，UE晚連線收不到。改將RuntimeReady獨立持久保留，驗renderer bound identity後每次首次連線／重連先發ready，再正常snapshot；不靠一次critical FIFO解決重連、不拿snapshot cadence猜rate。補實際TCP晚連線／重連回歸後再build／Editor／PIE／正式60Hz。

- 新版build56090成功（UBT7.77秒、stage eef2d3aa3794cd9fbdb083732cbea9f27481f887547162ea361199051cb68dad），Editor86808兩輪13/13；PIE46123行為通過但take_pie_screenshot失敗，MCP明確回報editor minimized。保留失敗report；透過search_tools發現editor_ui_windows，list確認唯一om Editor ref1ab4c8d9確實minimized=true，再精確restore，不要求使用者介入／不刪截圖斷言；串行PIE重跑驗證。

- run1791064374已通過真實60Hz終局與三方hash，但PNG頂端沿用預設120Hz，tick9994顯示83.3秒；原生結算的權威elapsed166.5秒正確。這是diagnostics metadata缺口，不能將假rate或假lag=0當成同步／效能證據。
- 決定：RuntimeReadyPresentation新增additive tag6 tick_rate_hz，直接來自bootstrap；bridge先驗bound player/team，再接受60／90／120。缺欄位／不支援rate顯示未知0，不推測snapshot cadence。頂端改Observed、未知顯示?／--，刪除沒有量測依據的lag=0t。模擬速度／遊戲數值不改。
- 新增IPC roundtrip（presentation30Hz不覆蓋authority60Hz）、不同player/team拒絕、缺失／不支援rate回歸；重建、Editor／PIE與正式雙端驗證尚待完成，舊成功run不能代替新版。
- 統計raw JSONL初稿在Lua -e使用arg[1]得到nil；該呼叫filename位置為arg[0]，改明確已驗證run路徑。重跑實得每隊209 PASS rows／105 unique ticks、0 FAIL；UNVERIFIED25240／25228獨立列出，不能混算PASS。

## E100：原生終局畫面必須使用明確隊伍（2026-10-04）

- 第九run1791064374首次launcher success=true／cleanup=true，兩隊four slots、死亡重生、ACK、authority winner2，實際Defeat／Victory PNG已人工檢視；保存verifier初次錯用path.read文字mode，Windows把PNG signature CRLF轉LF而誤拒絕。真實圖片沒壞，改既有API path.read(file,true)二進位模式，不改rawPNG／不忽略signature。第一次複合診斷command末段exit0亦不能掩蓋前段verifier失敗，後續獨立單指令重跑確認exit0。

- 第八run1791063849再逾時，both四槽／ACK已成功；p1進基地射程，但p2 AttackMove中線持續小兵戰，只有p1有完整objective策略，NPC／英雄互耗拖延結局。第九版讓兩測試玩家共用可見塔／基地流程，方向以runtime明確team而非player ID，unknown team拒絕；補team2 tower700／base0回歸。fallback每次正常Move到公開endpoint、無MatchSmokeHero cache，防施法中斷後同hero不再移動。兩隊皆須原ID attack status0，仍雙死亡重生／真實基地勝者／240秒／60Hz／UI後hash，不固定哪隊獲勝，不退讓。

- 更新進度檔一次patch因只摘半行而找不到context，未套用；重新讀實際完整行後更新，不假設先前已改成功。第八版build69536 exit0／UBT7.76秒、Editor93504同session兩輪13/13／串行PIE exit0，stage8dc4c15c74c413f41dab8fe20902a9828b0e37633b02f35f2c01f280da0599f6；正式run待核對，不把前版建置當最新證據。

- 第七run1791063365仍240秒逾時，但首次有原ID attack ACK status0、敵塔摧毀、base真實扣血。重生後base被Hide，fallback停舊tower x1650，尚未再公開base就被前線敵人擊殺；不是再次IPC拒絕。改opt-in fallback Move到公開催線endpoint2400，已公開tower／base分支仍优先接管；不從Hide推導摧毀、不讀隱藏base、不改HP／勝者／期限。此fixture策略修正須重建兩輪13/13／PIE／正式60Hz重驗。

- 第七版fullbuild18958／Editor98396兩輪13/13／PIE1407通過，core323、runtime50、bridge49＋integration2通過，stage034cb8dcd1991d46f4f4f12c4f2a3064dfd80f8e4f27e92f12cc4e6383cc648c。fallback逐行與本機protoc輸出相同。六次failed report均success=false／cleanup=true。事後查舊PIDs時24920已被dotnet.exe重用，不是原UE process；保留不殺，不能只按舊PID清理或說它絕對不存在。最新驗收清理須核對該run自己的owned程序。

- 第六run1791062840 release重建仍240秒逾時，double kernel改動尚不足。最直接的正式端到端缺口是player_input_to_renderer_intent沒有AttackTarget，proto RendererInput也未定義，UE Submit成功只代表本地enqueue，driver最後INVALID_ACTION；之前objective log不是authority ACK。新增additive AttackTargetIntent tag17（render ID／queued）、runtime先owner／epoch／secure disclosed reference，沿原KCP secure target路由，不塞進TowerAction／不送裸authority ID；fallback同步。smoke新增原ID objective_result，正式result-UI gate與保存verifier強制至少一次p1 attack status0，不能再用enqueue作攻擊成功。

- 首次capture cargo test未限定--lib，連帶編譯integration runtime_driver_smoke出E0063：舊OmInputCommand三個initializer漏新item_catalog_id，先前--lib掩蓋編譯缺口。補非商店fixture的0欄位，不排除integration；重跑完整bridge tests／opt-in capture才可封關。

- 新路線15Hz hash缺失的精確症狀為tick1475 owner lance cooldown81920：加入fixture自己的可見target admission後2405ticks／4808steps通過；完整base73也已通過（52.91秒），不是放寬hash／死亡斷言。capture回歸初稿又把protobuf HeroHudPresentation誤當有alive（native payload才有），立即逐段查proto，以hp_raw>0判定，編譯前修正；不能只靠名稱猜資料層欄位。

- 正常explicit target修正後core323通過，但base suite初次71/73：兩個完整lifecycle只在終局「雙方必須死亡重生」斷言失敗，逐tick hash本來一致；不能刪斷言或稱全suite成功。fixture先以正常MoveTo經塔戰，再既有Push／Guard，不改HP／phase／winner。新路線又於15Hz tick1475暴露owner lance cooldown缺失hash mismatch：headless helper讀authority選到該隊不可見目標，並不是可接受的網路玩家輸入。fixture新增現有TeamVisibilityRuntime current gate，target command無視野不admit，不增加ComponentRepair／owner cooldown修正fact，不更動正式權威規則。重驗15Hz2405 ticks／4808steps、death／respawn5／3與15終局凍結通過；完整73項（含120Hz）仍須重跑後封關。

- 第五run1791062140仍240秒逾時，射程已550、雙隊4/4及死亡重生、塔HP確實下降。進一步查正常hero_tick：explicit AttackTarget只在nearest10候選找，較多小兵會排除較遠指定塔並fallback小兵；修正把live／current-position／射程內的明確目標加入有界候選，仍經原敵友／HP／射程檢查，不加hidden目標、不提高近鄰查詢n。新增超過10候選／重複／射程外／NaN／無position回歸，後續完整filtered lifecycle必須同hash。fixture selector另需公開max_hp以拒絕座標剛好重合的小兵。初稿誤記base1800，立即查MobaMatchConfig::default是1600，編譯前改正；固定fixture1200／1600不是一般Bot／資料驅動結構識別，改fixture數值時此驗收必須同步。

- 第四run1791061680仍240秒逾時：p1四槽4/4且死亡重生10；p2 slot2 status4被拒絕、僅3/4且死亡重生1。追到正式IPC使用driver.filtered_entity_to_render_data，未填attack_range而Default=0，先前只查projection/native extractor不足；不是塔未公開或策略不夠積極。補解碼已公開TAttack.range.v（缺失／壞資料0，不重建未公開Buff modifier）；新增真實render payload回歸。它是公開current range，不宣稱全modifier有效射程，權威仍判合法性。被拒絕的smoke技能原先永久保留ID，改保留拒絕log、等新可見目標再正常新ID提交，不將拒絕算PASS。
- 本輪又猜OmGame/Private WorldBridge與comp/attack.rs、ue_shop_acceptance.lua不存在，並把*.stdout.log當Windows rg literal產生123。已用rg --files／已定位plugin與tower.rs／-g查證；之後先定位不猜路徑。退出Editor59032後立即build遇到尚未自然退出，restart graceful taskkill/T未使用/F且最後force_terminated=false；後續QUIT後先確認PID消失才啟動。

- 決定：先補6.2終局UI而非重寫已存在的QWER binding。native結算面板只讀正式HUD的phase／winner／elapsed與已綁定runtime的明確team；新增共用payload明確LocalPlayerId／LocalTeamId，不用player ID推導team，player7/team2列入測試。勝／敗／平手、死亡也顯示；TD／非Finished／無效team／winner／時間／tick清空。未知team不能猜結果。
- 不新增再開局／離開／自算比分按鈕；沒有相應權威契約前不做假按鈕。面板HitTestInvisible純呈現，正式權威已凍結終局玩法；StopRuntime／EndPlay清空結果。快照斷線不擅自改寫已收到的終局。
- opt-in原生面板必須在viewport有實際非零cached geometry才記log，驗收核對明確team／winner／outcome／tick，與同玩家死亡重生後的權威Finished一致；終局UI出現後至少120 ticks的三方hash。不是OS／完整像素或真人操作驗收。
- OpenSpec合併讀取再次遭output budget截斷，已分段補讀design尾段與tasks；後續不得以截斷內容當完整讀取。沒有新Rust玩法／ABI變更，新增的是UE共用HUD payload欄位。
- 首次build55721 exit1／C2039：誤把OmRuntimeConfig.team_id當OmFrameHeader欄位；先前rg單行命中不足以確認struct歸屬。改逐段讀header，team ID從runtime成功Start時保存的Config取得，Stop清0，不重讀可變env、不推導player、不新增ABI。日後查欄位必須看所屬struct。此處沒有完成C++建置，必須修正後重跑。
- 修正後build26264 exit0；後續補正式終局畫面截圖再重建，不把前一版本建置當最終證據。依本機UE5.8 UnrealClient.h六參數RequestScreenshot API，opt-in在實際面板layout後捕捉含UI的game viewport，目標為每次run獨立result-ui/team-N.png；launcher要求檔案存在，完成後仍需看圖，不用visibility log冒充像素驗收。Lua result observer14 assertions及原有movement／ability／parity／lifecycle皆通過。
- build86320通過，但第一次12項Editor automation在NativeMatchResult使Editor21832 access violation，MCP curl56 reset只是崩潰後症狀。call stack UWidgetTree::ForEachWidget→UUserWidget::OnWidgetRebuilt→新fixture.TakeWidget；NewObject後漏Initialize，既有NativeShopInput早已使用Initialize→TakeWidget。補初始化並assert成功，不刪測試、不忽略crash、不只retry HTTP；崩潰檔保留Saved/Crashes/UECC-Windows-AC8B88034AAFE137294D1680BCEE3612_0001，必須重建後兩輪12/12再進PIE。
- 首個正式60Hz run1791059641於240秒逾時：四槽各4/4、死亡重生3／4，但無Finished／UI；success=false、cleanup=true。最後p1長停x1504，base x2400仍存活，不能當UI成功。檢查hero_command_tick與hero_tick：AttackMove遭持续creep skirmish，AttackTarget射程外時普攻windup／backswing可Hold且fallback到射程內creep，重發命令仍無法推進。只補opt-in可見基地策略：若公開催體且無仍存活的可見fixture塔，先正式Move到安全快照attack_range的75%距離，進90%射程後正式AttackTarget；不讀hidden身分／不改HP／不保證可見塔缺席等於不存在（合法性仍權威判定）、不改玩法數值／勝者／240秒門檻。重建與重跑後才可宣稱修正驗收。
- 策略修正第一次build94954 C4458：local Owner遮蔽AActor::Owner，UE工具链視為error；改明確ObjectiveHero，不關閉warning。failed run1791059641被保存verifier如預期拒絕，未產生success evidence；五PID已CIM查無。
- 第二次60Hz run1791060294亦240秒逾時，四槽與死亡重生成功但沒有objective log／Finished。檢查p1 x1448、base x2400未進視野，發現前半段AttackMove仍卡，僅補基地分支不足；舊末段無條件AttackTarget又可覆寫approach。修正opt-in p1以Move到既有fixture塔附近x1650施法，p2保留Guard AttackMove；移除舊基地迴圈，由已存在可見基地分支唯一接管。這不是正式五位置Bot／導航演算法，仍不改玩法／HP／勝者／240秒，不把再次失敗宣稱通過。
- 第三次run1791060700仍逾時，无objective log／Finished；只移動到塔附近再施法，欠缺持續tower AttackTarget，普攻仍會轉打小兵。停止零散分支修補，收斂成單一可見目標流程：tower優先base、射程外Move／射程內unit spell與AttackTarget，移除另個enemy/skill選擇分支。新增共用SelectSingleLaneSmokeObjective與Editor回歸（snapshot順序／死亡／hero／NaN／無效身分／無frame不造目標），由同一helper被真正smoke使用。必須13项兩輪與正式重跑後才封關，不將三次逾時混成成功。
- 第三run完整資料：p1死亡／重生14、p2 0，原Guard長期疊己方塔。第四版把opt-in p2 AttackMove目的地改對面塔區x1000，兩個測試玩家都經過戰鬥，不用退讓／修改HP強制死亡或指定勝者。這是結算fixture輸入策略，不是遊戲平衡修正，也不宣稱正式五位置Bot；四技能、各自新身分重生、任一一致權威勝者與UI後hash等門檻不變。

## E099：小地圖座標輸入與 HUD 消費邊界（2026-10-04）

- 決定：小地圖只接受原生右鍵座標Move／Shift排隊，不選取entity、不反查actor，不新增Rust玩法入口。Unproject使用同一aspect／Y inversion，留白、NaN、零size、無map拒絕；輸入由configured player／settings scale換算cm，runtime Connected才提交。權威仍處理合法性／地形，不把schematic bounds當導航資料。
- 避免混淆：Slate FReply::Handled防止HUD點擊穿透；FOmGameplayInputEvent.bConsumed必須保持false，否則既有bridge拒絕整個命令。左鍵、右鍵invalid與release均Handled，但只有有效right-down提交一次。舊HitTestInvisible已不適用，測試改驗Visible及無Entity target／left／release不提交，不略過斷言。
- opt-in非Shipping `om-minimap-move-smoke` 在真實cached geometry觸發既有right-down handler／UObject callback，exact一次callback及原input ID套用status0後再核對移動／post-input三方hash。未ready等待；已dispatch失敗停止該smoke，不能fallback直接WorldBridge API或重送新ID。這不是OS／真人滑鼠或完整hit-test grid驗收。
- Editor測試使用本機UE5.8 FPointerEvent／FModifierKeysState constructor；OmEditor新增直接InputCore相依，不靠transitive link。模型留白座標不可clamp到地圖邊緣；world scale非法時fail closed。
- 讀OpenSpec合併輸出遭截斷，改分段補讀design／tasks完整內容後才實作；不能用截斷內容當已完整讀取。Lua observer14項（含JSON round-trip）通過；建置／Editor／正式60Hz待驗證。
- 首次fullbuild17732 exit1：UBT在Main啟動SetCurrentDirectory時，使用者global Temp目錄遭其他程序占用、IOException（Build.bat code6）。尚未編譯本段C++，不能當code錯誤或建置成功。檢查UBT實際初始化來源，決定只為此次Lua建置子程序使用workspace專屬TEMP／TMP；不刪global Temp、不終止無關程序、不修改系統環境。
- 確認本機EpicGames.Core DirectoryReference.cs：大小寫等價的cwd切換會先Directory.SetCurrentDirectory(Path.GetTempPath())，不是遊戲C++編譯錯誤。隔離TEMP後UBT成功（本專案實際13.68秒；AppData舊Log的129.12秒來自別的建置，不可混用）；將防護收進既有build_ue_moba.lua，僅om_restart build子程序設定Saved/BuildTemp/ubt，Editor啟動與其他stage不改環境。查檔又誤用omfue/Source/OmRuntime（不存在），已用rg --files定位Plugins/OmRuntime，避免猜測plugin source位置。
- 本次SOmMinimap新編譯C4701警告：雖然Unproject成功才使用out參數，仍明確零初始化BackendPoint，不靠編譯器推斷跨函式初始化。驗收log改記實際UObject回呼的反轉座標而非smoke要求座標，並驗requested/actual誤差≤0.05 backend units。需重建與重跑後才當最終結果。
- 外層process.run包住會啟動Editor的full Lua workflow，繼承pipe使stdout晚到Editor退出才返回；本次已用MCP／automation確認Editor95744，正常Save／QUIT後92229 exit0。後續直接呼叫既有workflow，不再以外層capturing process.run包完整Editor啟動。
- 最終build60172 exit0（10.36秒）、同Editor91416兩輪11/11／串行PIE通過，新C4701已消失。正式60Hz run1791058752 success／cleanup=true、雙隊原ID1各exact一次callback，status0於2452／3050、抵達480,288／1920,-288；post-input hash54／54 PASS至3240、FAIL0，UNVERIFIED6543／6543不算PASS，五PIDCIM查無。詳細證據與未完成範圍見2026-10-04-unreal-minimap-input-60hz-progress.md；全框架與6.2未完成。
- 讀active-ue-session.json時檔案不存在：它是互動存活狀態，不是所有smoke期間必然存在的證據；改讀launcher回報的run目錄／owned PID，不把檔案缺席當cleanup成功或程序失敗。

## E098：小地圖安全資料邊界與查檔錯誤（2026-10-04）

- 操作錯誤：猜測 `omfue/bridge/include/om_bridge.h` 不存在；真正 header 位於 plugin ThirdParty。又把 `maps*`／`story*` 當 rg literal Windows 路徑，產生123錯誤。先 `rg --files` 定位，pattern 用 `-g` 而不是猜路徑。
- patch 操作兩次因多檔 hunk 不存在／順序反向而未套用；檢查實際內容後重新套用，不能假設失敗 patch 已改過前面的檔案。
- 初稿 EntityGeneration 欄位拼錯，實際是 EntityGen；在編譯前依 schema 修正。不得靠猜 ABI／反射欄位。
- 新測試插入 generic `#endif` 誤落在 OM_BRIDGE_BINARY_SUBDIR fallback guard，會造成已定義 macro 時不註冊測試；檢查位置後把 fallback guard 先閉合、測試移到 WITH_DEV_AUTOMATION_TESTS 主範圍，新增 limits include。不以測試未discover為理由跳過。
- 決定：小地圖只取 presentation_snapshot 的安全 entities 及公開 route points。以 routes 建 square schematic bounds、10% padding，不從單位位置自動伸縮；無有效 map 時 fail closed。控制 frame 保留、完整 empty/reset 清空、Stop 清空。ghost 非即時目標，第一版不呈現 frozen memory、不提供點擊移動／敵人命中。
- 最終code fullbuild96915 exit0／stage SHA e9085c7115541e12f7bf1a64cdd0d5ebf3a8d1a8f40f274c84f80a44b59e657f；Editor57864同session兩輪10/10、串行PIE35587 exit0並停止。core322／base_content73／runtime49+3／bridge47（opt-in capture未執行）通過。不得把本段宣稱完整小地圖／三路視野或完整UI完成。
- 第一版完整build4626 exit0、Editor28184同session兩輪10/10通過，但追查正式IPC發現沒有paths欄位，不可將synthetic通過當正式遊戲已顯示map。補公開lane-length HUD metric、additive protobuf欄位，由權威config產生，runtime只讀公開metric、bridge轉路線；未知／越界長度不得猜測。另一次猜ipc.rs不存在，實際處理位於driver.rs；使用rg --files後定位。
- 新bridge測試發現 owner converter 沒單獨過濾HUD player_id（socket入口已有身分檢查），wrong-owner fixture仍產生map。加入converter同樣的owner gate；player0僅existing test wrapper許可，不宣稱正式socket已越權。首次46pass／1fail／1ignored保留，修正後重跑。
- 第一個正式60Hz minimap run1791057171 success／cleanup=true，双native map／movement／HUD成功，三方56／57 PASS零FAIL至3480，但最後移動觀察3551晚於最後checkpoint。加上minimap的post-input hash gate（每隊至少6 PASS且tick不早於movement觀察）再跑，不將舊run宣稱移動後完整checkpoint通過。
- 補強run1791057377 exit0、release／60Hz，雙native map／HUD／movement／ACK、post-input parity60／59 PASS至3600成功，五PID82452／37072／73216／45844／85308皆CIM查無。report success／cleanup=true；完整證據見 minimap progress。PIE screenshot僅legacy fixture（含map marker），顯示3FPS，不當60FPS基線。
- protobuf fallback 原來沒tag8。比對本機vendored-protoc產物只有三行差異，狹義同步lane_length_raw，保留原有dirty生成內容；不能只改proto卻漏掉受版控fallback。
- fallback同步後release server8656 exit0，diff --check通過。最終hash原檔還有非checkpoint UNVERIFIED7257／7258（沒有expected），不計為PASS；有效週期性checkpoint60／59、零FAIL。沒有宣稱每tick或實際render60FPS全封關。

## E097：Slate按鈕交易驗收不得繞過回呼（2026-10-04）

- 最終驗證：build45033 exit0、同Editor86232兩輪9/9、串行PIE26384通過；正式release run1791055726 exit0／success／cleanup=true、雙隊exact三個Slate pointer回呼各對應request1／2／3、拒絕→買→賣與pending0。獨立verifier49688 exit0，三方hash178／179 PASS至10800、capture5413／5404逐snapshot通過，五PID查無。launcherJSON亦保存成功。詳細計數及界線見 `2026-10-04-unreal-shop-button-60hz-progress.md`。
- 決定：opt-in `om-shop-button-smoke` 使用已建立且enabled的原生SButton、真實cached geometry與捲動範圍，合成pointer按下／放開事件觸發既有OnClicked→SubmitShopAction。不呼叫受保護ExecuteOnClick、不強制enabled、不建測試專用Rust交易入口。使用PreciseTap符合scroll list觸控行為，正常滑鼠DownAndUp不改。
- 非Shipping且明確flag才可操作；layout未ready不送交易，已dispatch但失敗則停止該smoke，不換ID重送。驗收要求exact三次callback各一次、request與既有queued三階段一一對應，再核對真實receipt／金錢／六格／pending／三方hash。
- 範圍：此為合成Slate pointer handler／按鈕回呼整合，不是實體滑鼠或OS／完整hit-test grid驗收。同Editor automation與PIE必須串行。

## E096：原生商店實際按鈕路徑與舊 HUD 相依（2026-10-04）

- 最終：完整build42013 exit0、staged SHA一致；Editor80648同session兩輪9/9通過。串行PIE79747與補preflight／cleanup後61417均exit0且已停止。fixture失敗未略過斷言；沒有驗收真人滑鼠或hit-test grid，详見 `2026-10-04-unreal-native-shop-ui-progress.md`。
- 操作錯誤：新9項automation仍在執行時便啟動PIE smoke，共用Editor操作重疊。automation最終兩輪9/9通過，但PIE未觀察到player且cleanup回報No PIE session，不能當成PIE成功。後續同一Editor的automation／PIE必須串行，先確認automation complete与PIE狀態，保留此失敗；停止cleanup錯誤不能覆蓋原始startup失敗。
- 第一輪新Editor automation 8/9，NativeShopInput 26個離線button-enabled斷言失敗。獨立TakeWidget tree尚無Slate frame/prepass，IsEnabled讀cached預設true；依本機UE5.8 SWidget::UpdateAllAttributes實際API，在遍歷檢查前明確更新綁定屬性。提交gate與policy斷言均通過，不能把fixture尚未刷新直接當成實際離線可交易。修正後須重建／重跑同Editor兩輪，不略過停用斷言。
- 檢查發現：原生 SubmitShopAction 未指定 PlayerId，FOmGameplayInputEvent 預設1；進一步讀取 BuildInputCommandFromEvent 確認 converter 會以配置的local player覆寫，所以不是實際越權／錯隊提交缺陷。仍明確填入事件身分，新增player2與invalid player回歸以避免上游診斷誤導。先前雙UE smoke走WorldBridge API，不得將其冒稱Slate按鈕驗收，也不得僅憑event預設值就誤判已送錯玩家。
- CreateOmHud 在沒有legacy Blueprint class或已有root時提前返回，native command bar因此缺失。分別建立可選root與native bar，最後統一bind，保留重入冪等。
- HUD命中區以280固定像素判定，但Slate使用viewport DPI scale；共用bar高度及scaled rect，native與legacy命中區分開檢查，缺legacy不阻止native。
- 空catalog首次signature同為空，RefreshShopRows提前返回而不建六格售出列；加入首次建立旗標。SetAbilitySlot沒有範圍檢查亦補上。這些是檢查確認的程式缺陷，已經完整UE建置／Editor automation驗證，不宣稱物理點擊成功。

## E095：商店 Unreal 輸入接軌的編譯與工具錯誤（2026-10-04）

- 最終驗證：保存證據 verifier exit0，run1791053894 的獨立報告 success／cleanup_verified／reconstructed_from_original_evidence=true。兩隊真實原始三筆交易、出售pending0、三方hash170／162 PASS（最後10800）、capture5457／5443筆逐snapshot通過，五PID查無。原launcher報告保存exit1仍如實保留；不能以恢復報告掩蓋原失敗。完整證據及重現指令見 `2026-10-04-unreal-shop-input-60hz-progress.md`；debug效能與人手滑鼠點擊未驗收。

- 保存證據 verifier 第一次誤用 path.is_dir，實際工具 API 是 path.is_directory；只讀 assert 前失敗，沒有啟動／寫入交易。查 path.lua 後修正，不新增平台 fallback。

- 第二跑 release run1791053894：兩隊真實拒絕→買入→出售、pending=0與移動已被原生日誌觀察，owned五程序已清理；但最末 JSON 報告保存失敗（JSON object key must be string）。原因是新 observer queued 以稀疏 numeric stage0／2／4 作 keys。改字串 keys並新增 JSON encode/decode round-trip 回歸。使用原始保存的日誌／三方hash／protobuf capture 重新驗證及重建獨立報告，不重送交易、不忽略失敗、不竄改原始證據。

- 首次雙 UE run1791053032 失敗，不能宣稱端到端成功。team1 已完成拒絕／買／賣並移動；team2 在 tick11070 買入、11381 售出已由 runtime item_tick 結算，但 UE 最後只呈現11365，deadline 前未觀察出售。debug runtime2 inbound backlog 升至602；主因尚未單獨定位，不能說 pending 清理是所有效能問題的根因。五 owned PIDs 58208／88020／27460／81776／22380 已由 launcher 清理，CIM 查無。
- 決定：不提高210秒期限、不注入金錢。正式驗收改用既有 release server／runtime／script DLL profile，重建相同1.95.0 toolchain與最新Lua規則；Unreal 渲染限制60FPS，network仍60Hz／presentation30。商店完結 gate 額外要求 bridge pending=0，正向結果不能只見到 receipt 卻洩漏佇列。此為 production profile 驗收，debug效能仍是獨立未封關項。
- 又一次猜測 scripts/prepare_server.lua 不存在；建置入口實際是已讀取的 run_2player_ue.lua build_server／build_runtime。不新增假入口。

- 整合檢查發現舊 bridge 只接受 APPLIED_TO_PRESENTATION，會忽略 accepted SHOP_SETTLED 並留下 pending；商店成功必須以 SHOP_SETTLED 退休，普通 APPLIED／FORWARDED／query-only 不能當成交。SHOP_TRANSPORT_UNCERTAIN 即使 accepted=false 也保留原 request，供同 ID 查詢恢復。新增 busy-ring／原 tick／重複結果與不分配新 ID 回歸。第一次雙 UE run 使用修正前 DLL；修正後需再次建置與實跑，不能以單元測試冒稱修正版端到端驗收。

- Unreal C4458：UWidget 已有 Slot 成員，新增售出迴圈不可用 Slot 區域變數；改 InventoryIndex，不關閉 warning-as-error。full build 因此失敗，Editor 尚未重開，修正後重建。
- 再次猜測 bridge/build.rs 不存在；bridge header 是既有 build_bridge.bat 的 cbindgen 階段，不新增 fallback。只讀搜尋需遵守實際路徑清單。
- MCP save_all_dirty_assets 成功；exec_console_command QUIT_EDITOR 派發後 CIM 證實 Editor 28056 已退出，om.log 有完整 LogExit／closed，這輪沒有 taskkill force。

- 搜尋猜測 presentation.rs／OmInputTypes.h 不存在；改用已列出的 presentation_bridge.rs／OmGameplayInputTypes.h。再次違反 E018 的先列路徑原則，不能視為已消除操作問題。
- Rust E0609：compiled MobaItemConst 名稱欄位是 name，不是 display_name；依實際型別修正。
- Rust E0063：新增 protobuf capability 後測試 OwnerEconomyPresentation initializer 漏填；所有消費端一起搜尋／更新，欄位預設 false。
- apply_patch 同一批次兩次 Update 相同檔案被拒；合併 hunks 後重跑。紀錄檔標題亦需依實際內容定位，不猜標題。被拒批次没有寫入。
- 決定：C ABI 明確升至 6，舊 DLL／header 不可混用。買賣沿用共用輸入，只有外部 runtime IPC 路徑可提交，不走 legacy gameplay driver。驗證結果記於本輪進度檔。

每次遇到錯誤，先查本檔；追加「現象、原因、決定、驗證」，不得只反覆重跑同一失敗指令。歷史詳細過程見 `2026-10-03-unreal-moba-ipc-editor-progress.md`。

## E094：商店 transport 開放與真實交易驗收（2026-10-04）

- 決定：catalog agreement 不等於交易能力。新增 shop protocol 1／完整 rules hash，僅在已綁定玩家的 secure V2 single_lane session 啟用；舊／Story session 不開放。read-only query 最大 32 bytes、每 session 每牆鐘秒 8 次，客戶端時間無法擴充預算，暫停遊戲不阻擋恢復。
- runtime 最多 64 個待查 ID，每秒最多查 4 個，依上次查詢時間排序避免低 ID 餓死後面的交易；終端／expired／late 結果退休，不自動換 ID 再買。renderer shop request ID 同 ID／同 payload 只查原 wire ID；衝突拒絕，1024 筆歷史满額拒絕新交易而不逐出重放防護。此歷史不跨 runtime 程序重啟。
- 編譯錯誤 E0063：新 TeamGameStart 協商欄位漏了 generic projector initializer；補為無能力版本 0，再更新 legacy JoinRequest literals。E0599：測試猜測不存在的 render_snapshot API；查真實 extract_presentation_source 後將 smoke 搬入原本 snapshot extraction，不能額外 extraction 消耗 Hide／Forget directives。
- 操作錯誤：再次把 OmUi* literal glob 交給 rg 作路徑失敗；改以 rg --files 列實際來源，避免重複猜測。
- 真實 run 1791050713 失敗且三程序 cleanup_verified=true：210 秒內最後 safe tick 18311（120Hz nominal），尚未走到可合法累積 350 Gold 的 tick 21240；capture 只有原始不足金錢 receipt，沒有正向購買。capture checker 明確失敗，不提高 timeout、不注入 Gold、不假稱成功。
- 第二次 run 1791051137 啟動即失敗且唯一 server cleanup：誤將 headless Coarse15Hz 當成 network STEP_FPS 支援值；ServerSetting 明確只支援 120／90／60，正確 fail closed。不能為測試鬆綁正式 FPS 驗證；先查 server_config.rs，再改為已有正式 60Hz。診斷又把 state/core/*.rs literal glob 交給 rg 失敗，後續只依已確認實際來源。
- 第三次 run 1791051262 的 runtime 起動被 presentation-hz=10 拒絕，清理 server／runtime；config.rs 明確只允許 30／60／120，不將想用的低頻率當成已有能力。再次先讀真實 validator，改30Hz。
- 修正驗收方式：正式 60Hz network profile、presentation 30Hz，wait 仍是 210 秒、Gold 規則仍每 active 秒 2、warmup 2 秒；門檻按 negotiated fps 算，不把牆鐘等候當成模擬進度。120Hz 網路正向交易與效能仍需單獨驗收。
  - 最終 run1791051305 success／cleanup=true，60Hz雙隊各89三方hashPASS。原始capture逐snapshot驗證Gold／裝備／三筆immutable拒絕、買、賣receipts，兩隊5554／5518筆通過；每input額外5次完全相同重送、原buy terminal查詢成功，沒有重扣或多退。
  - 後續runtime write失敗視為不確定，不刪原ID；主／catch-up都回報原renderer request的recovered terminal結果，expired明確不確定。此補強有回歸／建置，沒有宣稱已做網路故障注入。使用者本輪指定先60Hz，120Hz驗收延後。

## E093：正式商店驗收不能依賴不存在的收入；Editor 開啟時不可 stage DLL（2026-10-04）

- 問題：正式 match Gold 0 且沒有收入，無法合法驗證正向購買；單元 fixture 注入 Gold 不代表正式遊戲可買。
- 決定：Lua moba_economy → build-time 整數常數 → Rust authority commit 被動收入，預設每 active 秒 2 Gold。累積 Fixed64 時間餘數，暖場跨界只計 playing 部分，暫停／Finished 不累積，死亡 slot 保留收入，重生不重領，i32 上限飽和。pending delta commit 後取走，重複 commit 不重付。
- 版本：全模板 catalog_data_hash 包含收入規則；identity content_hash 沒有角色 ID 改變時可保持不變，不能混為一談。Lua dev hot reload 不得單邊改收入，必須重建所有 peers。
- 操作錯誤：仍猜測 simulation_driver.rs 與 codegen/README.md 導致只讀搜尋 error 2；再次要求先 rg --files 確定真實路徑。之後改查 native/simulation_driver.rs 與 codegen/src/main.rs。
- 建置失敗：build-only 在本專案 Editor PID 78632 執行時拒絕 stage bridge（exit 4）；不重跑同一命令、不強制覆寫。確認唯一 Editor 的完整 project command line，MCP save_all_dirty_assets ok 後改 full 正常停止／重建／重開。
- 已驗證：base_content 72 tests 通過（雙 profile lifecycle 啟用正式收入）；template 28＋23＋8＋1 通過；core 321 通過；runtime 46＋3 通過。真實 run 1791049560 三方 10／8 checkpoints PASS、三程序 cleanup；原始 protobuf IPC capture 逐 snapshot核對 playing 時間與金錢，兩隊 571／224 筆，最後 18／16 Gold，六格空且無交易 receipt。
- 邊界：尚未以正式 KCP 成功買賣，shop secure gate 保持关闭。不能用此部分結果勾選完整 5.3／6.2。Unreal 重建結果見收入進度檔。
  - 重建結果：full 成功，stage SHA a6978590f720087819d2422190b86f28c0c08aa3cfe45c3310bd124e61bff63d，新 Editor 28056／MCP ready／native automation 兩輪各 8/8；restart 的正常 taskkill 逾時，工具自動 force 舊 PID 78632 及其子程序。資產已先保存，但不能描述成正常關閉成功；往後重啟前需優先查 MCP orderly quit 支援，並检查工具 force fallback。新增從真實收入 350 Gold 購買／出售的 headless 正式 PlayerInput 測試通過，不注入餘額、不冒稱 KCP。

## E092：交易結果重送不得再次交易或改寫原結果（2026-10-04）

- 現象：admission journal 只保存排入結果，DuplicateShop 不含實際 terminal receipt，不能用作成交成功回覆。
- 決定：以 authority filtered frame 的正式 receipt finalize journal，嚴格核對 player／ID／原命令 catalog 或 slot／effective tick；原結果不可改寫。新增獨立 read-only KCP query／replay，玩家由已加入的 V2 session 決定，不接收客戶端指定 player，不進 gameplay queue。
- 結果區分 unknown／pending／terminal／expired／admission-late-rejected；非 terminal 沒有 receipt，不假造成交。超過 1024 筆保留 watermark，expired 不重新執行，記錄不是永久帳本。
- runtime 只收自己、相同 input ID、schema 1 的 typed receipt，保留原 settled tick，更新 presentation history 而不套用 Gold／Inventory。history 改為同 input ID 即衝突檢查，避免同 ID 換 tick 另建結果。
- 查詢再次誤用 Windows literal glob omoba-content-model/src/moba*，失敗後改查實際 build.rs emitted catalog 欄位；不要把 glob 路徑交給 rg 作檔名。
- Fyrox check 出現 E0004：新增 LockstepInbound::ShopReceiptReplay 後既有 exhaustive match 漏更新。加入明確 unsupported 警告，不使用 wildcard／NoOp 把 receipt 送回 gameplay，不開放 legacy 商店恢復。
- hot path 決定：沒有 pending shop 就不 decode 額外 TeamTickFrame；history 保存 pending 計數，finalize／evict 精確減少，不每 tick 掃描 1024 筆結果。沒有新的效能基線前不宣稱 overhead 已量測。
- 真實 query smoke run 1791048551 未收到回覆而失敗，三程序已清理：Lua 檢查父程序旗標，但 spawn 明確 child env 漏傳 OMOBA_SHOP_QUERY_SMOKE，runtime 根本沒有送查詢。改為明確傳遞，不調整 timeout 或伺服器結果；必須以 runtime log／實際回覆核對，不只看啟動環境設定。
- 第二跑 1791048712 仍失敗，三程序清理成功：runtime 的兩個 inbound pump（主 select 與 catch-up try_recv）只新增第一處 ShopReceiptReplay 處理，catch-up 丟棄了新訊息。補齊第二處；往後每次新增 inbound variant 都查完整程式的所有 drain／match，不因 wildcard 能編譯就視為已整合。
- 最終 run 1791048834 success=true／四程序清理：雙隊及重連 runtime 都收到 unknown query，ID floor 仍為 1、新 ID=2；hash 14／12 PASS、重連後 4 個。core 321、server 151／1 ignored、runtime 46＋3、bridge 44／1 opt-in ignored、Fyrox check 通過。真正 terminal 查詢為 kernel／投影／journal 單元驗收，不冒充尚未開放的 KCP／UE 買賣；完整範圍見 `2026-10-04-shop-result-recovery-progress.md`。

## E091：runtime 重啟與輸入 ID 溢位不可回到 1（2026-10-04）

- 現象：InputBridge 每次程序啟動使用 default，且 wrapping_add 可在 u32::MAX 後重用 1，與既有商店 journal 衝突。
- 決定：InputBuffer 保留每玩家、match-lifetime 的 admission high-water mark，涵蓋已排入與晚到拒絕，不因 drain／eviction／玩家斷線消失；bootstrap 回傳 input_allocator_version=1 與 last_seen_input_id。runtime 握手拒絕缺少／未知版本，從 floor+1 分配；耗盡回報 INPUT_ID_EXHAUSTED，永不繞回。
- 範圍：不是交易 terminal receipt、帳本持久化或 exactly-once；未到達 server 的封包不算已見，並未補自動交易重試。既有 roster 拒絕同玩家重複活動 session，重連測試須等待 server session cleanup。
- 工具錯誤：再次猜了不存在的 omoba-core/src/runtime/input_bridge.rs、lockstep/server_state.rs；真實檔案是 omoba-client-runtime/src/input_bridge.rs、omb/src/lockstep/state.rs。往後先 rg --files 查到路徑再讀檔。
- apply_patch 兩次失敗：同檔 hunks 先修改 new() 後倒退至 impl，無法匹配；調整為原檔順序，不依賴失敗 patch 已寫入。
- 重連測試檢查發現 time.poll predicate 回傳 boolean 卻後續讀取 replica_tick；改成符合條件後回傳 safe object。程序載入中的 Lua 已經解析，修改檔案不會更新正在執行的測試；須保留結果並重新執行。
- 驗證：core 320、server 147 passed／1 ignored、runtime 46＋3、Fyrox cargo check --tests 通過；真實 KCP 重連與 bridge 驗證完成後另補進度紀錄。
- 真實首跑 moba-runtime-1791047611 失敗並完成三程序清理：hard stop runtime 後 15 秒未觀察 session cleanup；server 記錄 UDP 10054，但不等於 roster 已解除。測試使用既有 --shutdown-file → KCP SESSION_CLOSE，等待程序退出及 server cleanup，再重新加入；不提高 timeout、不直接修改 roster。server 日誌在 stdout，poll 同時讀 stdout／stderr。此驗收範圍是 graceful runtime 重啟，不能宣稱已驗證 crash／斷網復原。
- 第二跑 moba-runtime-1791047805 正常重連得到 floor=1，確實送出 ID=2，但 movement evidence 逾時；日誌確認目標與第一輪相同，英雄已站在目標，不會有位移。加入既有 scripted-move-interval-ticks=120，使第二筆往另一方向，仍走正式 RendererInput；不以送出成功冒充玩法已套用。四程序清理已核對，保留失敗報告。
- 最終 moba-runtime-1791047918 success=true：原 ID=1／floor=1／新 ID=2、正式位移、雙隊三方 hash 14／12 PASS、重連後 4 個檢查點；四程序清理核對。最終 server 148 passed／1 ignored，bridge 44 passed／1 opt-in ignored。完整限制與重現見 `2026-10-04-runtime-input-resume-progress.md`。

## E090：正式交易開放前必須先防重送與協商 catalog（2026-10-04）

- 再次猜錯 omb/src/lockstep.rs 與 Windows literal glob kcp/*，查詢失敗。以 rg --files 找到 lockstep/input_buffer.rs 與 kcp/client.rs；不要因熟悉概念就跳過真實路徑查詢。
- InputBuffer 原本每次 submit 都追加，相同 shop input ID 可跨 tick／drain 重送並再次交易。新增 match-lifetime、player-scoped 的 shop admission journal；同 ID／原始 target tick／命令只排一次，換命令／換 tick／零 ID 拒絕。已拒絕晚到的原始結果保留，不隨重送的 grace 改成成功。
- 每玩家最多 1024 筆、Buy item_id 最多 128 bytes；逐出最低 ID 後保留 expired-through watermark，不能在清理後再次執行舊交易。這是 transport 排入 at-most-once，不是永久帳本、exactly-once settlement 或 terminal receipt 重播。
- catalog 握手是版本／內容相容，不是開放交易或安全認證。Join 與 TeamGameStart 增加獨立 version/hash；只宣告 version 或只有 hash 均 fail closed。舊客戶端可不宣告，但新版 selective client 必須收到匹配回覆，不能默默降級；未知混版仍拒絕。
- protobuf fallback 由既有 vendored protoc 流程生成，同步舊 Fyrox web Join initializer；不手改 generated/game.rs、不改 C ABI 5。
- Fyrox cargo check --tests 發現 native.rs 的 InputActionKind 分類未包含先前新增 ItemBuy／ItemSell，編譯 E0004。新增對應診斷 enum／match，不以 wildcard 或 NoOp 假裝處理買賣，不開放其 secure shop intent；這是既有共用協議漏更新，後续每次 proto 變動要檢查全部消費端。
- secure shop gate 仍關閉，直到原始 terminal-result 重新送達／runtime 重連 ID 恢復完成。DuplicateShop 只表示不重排，絕不冒充購買成功；wire correlation 不進 gameplay 或 script ABI。
- 本輪驗證：core 320、server 146／1 ignored、runtime 45＋3、bridge 44／1 opt-in ignored、base_content 71、catalog 1、Fyrox check 通過；新版 full UE build／stage SHA／MCP、同 Editor 兩輪各 8/8、真實雙隊 run 1791046869 新握手／四槽／三方 hash 120／113 PASS，五測試程序退出核對。詳細證據與未完成契約見 `2026-10-04-shop-admission-catalog-progress.md`。

## E089：新增經濟 C ABI 時的路徑／結構相容錯誤（2026-10-04）

- 查詢猜測不存在的 bridge/build.rs 與 OmGenerated/Private/OmEditorAutomationTests.cpp；實際 header 由 build_bridge.bat 呼叫 cbindgen 生成，Editor fixtures 在 OmEditor/Private。先以 rg --files 查路徑，不依模組名称猜測。
- driver patch 用了錯誤的區域變數名稱 frame（實際 world），apply_patch 原子拒絕該檔；讀取實際呼叫後修正，不把拒絕當已套用。
- OmInventorySlot 增加 OmStringRef 後 derive Debug 編譯失敗；補上 OmStringRef 的 Debug（不改 ABI layout），bridge 43 passed、1 opt-in ignored。
- C ABI 升 5，owner economy／receipt／物品名稱字串均由 frame slot 擁有，control／reset 清空。不得以 ABI 4 DLL 配合新版 header。
- 物品名稱／價格取 compiled Lua catalog，不寫死 UE 角色／物品名稱；shop_available 僅代表權威場景條件，不能視為 transport capability。UI 明確唯讀，receipt 只呈現權威結算，不以 accepted input 假造成功。
- 重綁 WorldBridge 需同步清除新 EconomyHudStateChanged delegate，避免舊 actor 仍更新 HUD。
- 首次 UE full build 失敗 C4458：UOmCommandBarWidget 的區域 Slot 遮蔽 UWidget::Slot（warnings-as-errors）。改名 InventoryIndex，不降低編譯警告門檻；完成後重建，不載入部分更新的 ABI 5 插件。
- 新 smoke 納入 Om.Runtime.UiOverlaySurface，卻仍以 Om.Generated 過濾 discovery，造成 required test missing；改成 Om. 共同前綴，保留精確八項 required test／兩輪／零 skipped 門檻，不移除新測試。
- 讀取 cleanup PID 時 Get-Content 回傳多行 array，不能直接 cast int；首次輸出 pid=null／running=false 無效，不可拿來宣稱清理成功。改 Raw＋Trim＋純數字驗證，重新查實際 PID。此處只是唯讀診斷，不新增 PowerShell workflow fallback。
- 最終驗證：bridge 44 passed／1 opt-in ignored、最新版完整 UE build／SHA／MCP、同 Editor 兩輪各 8/8、PIE smoke、真實雙隊 owner economy＋四槽＋三方 hash 114／106 PASS 均通過。真實非空物品／receipt 畫面與 secure 交易仍未完成。
- full build 的既有 restart 對已驗證專案 PID 19428 正常關閉失敗，等待 10 秒後執行 scoped force termination 並確認退出；不擴大關閉其他 Unreal 專案。

## E088：交易結果不可等同輸入 ACK，也不可把 correlation 帶進玩法狀態（2026-10-04）

- 本輪再次猜錯 stable_fact／team_projector／filtered_replica／state_initializer 路徑。真實檔案由 `rg --files` 找到：runtime/stable_fact.rs、runtime/team_projector.rs、runtime/selective_replica.rs；初始化在 native/initialization。沿用 E085 規則，後續必須先查真實路徑，不能依模組概念推測檔案位置。
- PendingPlayerInputs 刻意沒有 input ID，不能為商店把 wire correlation 寫進 gameplay components 或 script ABI。交易 drain 只產生 tick-local ShopSettlement；投影層依 player 與原始命令順序精確核對 CanonicalAcceptedInput，才附上 correlation。缺結算不產生成功，命令不符直接失敗。
- 舊 APPLIED_TO_PRESENTATION 只代表輸入套用，不代表買／賣成功。商店由真實 ShopReceipt 決定 SHOP_SETTLED 或具體拒絕；死亡沒有 actor 時仍能回覆拒絕。Warmup／Pause／Finished 在跳過 dispatcher 前產生 MatchUnavailable，下一個 tick 清空舊結果。
- OwnerEconomy 是 owner-team 權威資料，與 live hero 身分分離；死亡期間使用已保存的 slot Gold／Inventory，Unreal/runtime 不自算、不假造滿背包或餘額。IPC 再限定 configured player；敵方不收此資料。
- 新完整背包斷言第一次編譯 E0369：Inventory 沒有 PartialEq。改比對既有 serde_json::Value 精確表示，不為測試變動 production component trait。
- server metadata_guard 初次失敗：新的 scripts fixture 直接查 input_id，即使只是測試也違反 scripts 不接觸 wire ID 的既有來源邊界。將 wire correlation 的精確比對保留於 core/shop_receipt 單元測試；scripts 的正式 World 測試只查 player／tick／結果。沒有放寬 guard 或把 ID 加進 gameplay。
- 驗證結果與剩餘限制記於 `2026-10-04-moba-shop-receipt-ipc-progress.md`。正式 secure shop 仍關閉；IPC 欄位已增加，但未更新 C ABI／UE 畫面，不能引用舊 UE 成功結果當本輪驗收。

## E087：經濟結算必須在正確階段套用，baseline 也必須按隊伍過濾

- 初版 patch 只以 `for event` 定位，誤插入 movement helper，編譯 E0425（找不到 world）；修改為獨立 typed settlement helper，在 component export 後呼叫，再同步 Specs。後續 patch 必須含函式名稱等唯一上下文。
- baseline、Reveal、server expected hash／rebase 都要使用同一 owner-team 過濾；只限制每 tick fact，仍會在出生或重連洩漏金錢與背包。
- 新 CommittedEconomy 使用 81-byte bounded typed payload，包含 Gold、六格 numeric catalog id／cooldown、十個 ItemEffects bits／dirty；float bits 僅為保留既有 component 精確表示，解碼檢查 finite／範圍／未知 id／空格 cooldown。先驗證完整記錄與既有私有 schema，才提交三個 component。
- CommittedEquipmentStats 只公開可見 actor 最終 speed／armor／attack；客戶端不取得敵方物品或餘額、不重放交易，所有既有技能 input／cooldown 邊界維持不變。
- Lua 物品新增明確 catalog_id，拒絕 0／重複；不依賴宣告順序當網路 ID。測試用 fixture 數值但使用真實 catalog 名稱，以便 typed inventory 可表示；不把測試價格當正式 Lua 價格。
- 新 filtered 商店回歸在 tick 3 失敗；增加 schema 級差異診斷，確認對手 CProperty.mhp 沒有裝甲 HP 加成，而 Gold／Inventory／ItemEffects 已吻合。敵方不持有物品，不會本地算出 max HP；CommittedEquipmentStats 擴為 40-byte 最終 HP／max HP／speed／armor／attack。保留完整 hash 門檻，不使用 ComponentRepair。
- 修正後新商店／fresh bootstrap／裝備重生完整 hash 回歸通過；core 314、base_content 70（含 15Hz／120Hz）、template-ids 27＋23＋8、server 140／1 ignored、runtime 41＋3、bridge 41／1 ignored。secure 買／賣仍拒絕；本輪沒有真實 UE／TCP shop 驗收，不把 headless snapshot 重建當作 renderer 重連。

## E085：正式商店輸入必須保持順序，不能提前開放 secure 客戶端

- 查詢曾猜錯 client-runtime/build.rs、comp/pending_item_use.rs、transport/secure_acceptance.rs、runtime.rs；實際為 core/build.rs、comp/lockstep_resources.rs、transport/kcp_transport.rs、runtime/mod.rs。後續先用目錄搜尋取得真實路徑。
- 第一個 patch 的結尾誤帶加號，驗證拒絕且沒有修改檔案；修正語法後才執行，不將失敗當成功。
- 買／賣與使用物品不能分成兩個 queue 後分批 drain，否則同 tick 的交易順序會改變。決定沿用 item phase，改為 ordered PendingItemAction enum。
- 新增 protobuf tag 17／18，不重用舊 tag、不傳 client 金額或 hero id；fallback 由 vendored protoc 正式生成。
- secure V2 目前沒有 owner 經濟結算／catalog 相容協議，因此保留網路入口拒絕新交易；正式權威 tick 的測試不是 Unreal 商店端到端驗收。

## E086：Lua 物品不能靜默切換成與 compiled catalog 不同的值

- 物品先前只在 server JSON，不是共用內容；MOBA 改用 templates/moba_items.lua，由既有 build.rs 生成 Fixed64 catalog，納入完整 template data hash。舊 TD JSON 不變。
- 僅開放確實實作的 passive atk／hp／ms／armor；deny_unknown_fields 拒絕未支援 mana／active 欄位，避免資料看似成功但沒有行為。配方價格必須沿依賴嚴格增加，拒絕 cycle／未知材料／材料總價超過成品價。
- 開發 hot reload 不會自動重建 world ItemRegistry，不能只更新內容 hash。MOBA 啟動核對 runtime Lua 與 compiled JSON；物品變更禁止 hot reload，必須重建 peers。
- 本輪另一 patch 對同一 runtime_content.rs 寫兩個 Update File，被工具原子拒絕；合併同檔 hunks 後通過。搜尋又誤用字面 Windows glob omoba-content*，已改已發現的明確目錄。
- catalog 改變後舊 Unreal staged binary 不再代表目前源碼；尚未重建／重新跑 UE，禁止挪用前輪成功紀錄。
- 驗證：template-ids 27 lib、core 311、base_content 69（含 15Hz／120Hz 完整 filtered lifecycle）、script-abi 13、omobab 140／1 ignored、runtime 41＋3、bridge 41／1 ignored 均通過；正 bonus 量化後為零也拒絕，startup compiled 不符／hot reload 變價／完整 hash 改變均有回歸。

## E084：商店的舊實作不是正式雙隊 deterministic input

- 舊 JSON ResourceManager 商店存在，但距離固定 (0,0)、配方扣完整 cost，不能直接當新 MOBA 共用交易。保留舊 TD 行為；新 kernel 定義完整價格減去全部材料價，六格交易先 clone 驗證、成功才提交，售價以整數 floor 50%，拒絕負價／溢位／非有限 bonus。
- 單路英雄原先沒有 ItemEffects，背包保留重生後也不會重套裝備數值；新增 dirty ItemEffects，使用既有 item_tick 重算，不另寫 C++ 或第二套 stats。
- 測試首次編譯 E0596：Vf32.val() 的簽名要求 &mut self（實作只是回傳 v，沒有 cache）；只讀斷言先 clone 再 val，不把查詢改成寫入 ECS。
- 正式 PlayerInput 尚無 Buy／Sell，owner Gold／inventory 結算投影及 Lua 共用物品 catalog 未完成；新 API 先限權威端，不能以 kernel 單元測試宣稱 Unreal 商店已可玩。
- 驗證：core 311、base_content 66、omobab 139／1 ignored、runtime 41＋3、bridge 41／1 ignored；兩隊基地、phase／pause／death、六格／合成／overflow、stats 撤銷、裝備金錢重生不加倍、含 ItemEffects 的完整逐 tick 重播均通過。

## E081：日誌格式與檔案路徑必須先核對真實宣告

- Match smoke 的 EntityGen 是 int32，不能用 %lld；改成 %d，Unreal build 通過。HUD signature 增加 hero generation 與倒數是否為正，才能留下死亡倒數與新身分重生，不逐 frame 印日誌。
- 本輪仍曾猜不存在的 OmGameplayInputComponent.h／OmPlayerHudWidget.cpp；實際 HUD 在 OmPlayerController.cpp。不得猜類名即路徑；先 rg --files，再搜尋既有 symbol。
- 更新本檔曾誤加日期前綴而 patch 失敗；改真實路徑後另一 patch 的 hunks 次序與檔案不符，又失敗且未改檔。先讀取目前檔案，按實際出現順序 patch，不重試同一錯誤。
- restart graceful close 的 child processes 未退出，工具等待後限定既有 project PID 53480 做 force close 並確認退出；未擴大到其他 Unreal project，重建後 Editor PID 89472。

## E082：完整 Unreal smoke 的進攻策略不能省略傷害技能

- run 1791035563 保留 success=false：兩隊四槽、正倒數死亡與新身分重生均發生，但 240 秒門檻內無 Finished；三方沒有 FAIL。只 heal／attack-move 的 Push 不等於既有 headless Push 的技能策略，不能以延長時間代替修正。
- 決定：p1 對當前安全 frame 中距離 <550 的可見 fixture 敵塔／基地或敵英雄施放 generated unit-target 槽 2／0，依 owner cooldown；p2 保持 Guard。所有操作只走正常提交 API，沒有 hidden lookup／HP 寫入／強制勝負／減弱敵人。
- 當前 frame 的 NPC 缺通用陣營／objective metadata，此 smoke 只適用既有長度 2400 的單路 fixture，不宣稱正式 Bot。正式 Bot 與三路 metadata 仍未完成。
- 終局 HUD 修正：死亡者不再顯示 Respawning，顯示 Match Finished；Draw 與 Winner Team 分開。沒有 local-team payload，暫不假造 Victory／Defeat。
- 修正驗證：run 1791036154 在相同 240 秒門檻完整通過，兩隊 winner 1，終局後三方 169／203 PASS、零 FAIL；不刪失敗 run。可見目標進攻＋正常規則完成，不延長門檻。

## E083：Lua assert 的多回傳不能直接傳入 optional base 路徑

- 保存完整對局證據的新工具首次失敗：`path.absolute(assert(arg[1], usage))` 在成功時仍回傳 path、usage 兩值，usage 被當作 base。沒有寫出假成功證據。
- 決定：先綁定 requested_run，再呼叫 path.absolute；確認 path.lua 沒有 basename API 後用受限路徑末段 pattern，不猜不存在的函式。
- 另一次診斷猜 comp/faction.rs 路徑不存在，已改對 comp 目錄搜尋真實宣告 unit.rs。後續路徑搜尋需先目錄，再實際檔案。
- 修正後保存工具成功從真實日誌／全部 checkpoints 重驗並產生精簡 evidence JSON；不只複製 success flag。
- 負向驗證：失敗 run 1791035563 被明確拒絕，不寫 success evidence；成功 run 清理後重驗 170／204 PASS、tick 18720。

## E080：完整 filtered 對局測試必須包含權威 accepted-input 邊界

- 新完整對局測試首次在 tick 73 發現 owner cooldown／attack phase hash 不同。原因是 headless SimulationDriver 只執行正式 PlayerInput，並不模擬 server 的 CanonicalAcceptedInput 網路投影；漏掉這層會讓 owner replica 沒收到施法。
- 初始決定：測試在實際 authority input 前記錄 actor／target 的 generation 身分，使用與 server 相同的 stripped protobuf payload 與 canonical acceptance 投影；不可用 ComponentRepair 掩蓋漏輸入。owner cooldown override 一直禁止；原先也禁止 owner attack settlement 的假設，後續被 tick 522 的 lifecycle 邊界推翻（原因與修正見下）。
- 這個 Bot 是既有 headless fixture policy，不宣稱已完成正式視野受限五位置 Bot；新驗收目的是逐 tick 核對死亡／重生到終局的兩隊 canonical world。
- 補 acceptance 後 tick 74 仍失敗，這次是正式缺口：authority hero XP=25，而 owner filtered XP=0。新增 CommittedProgression（16-byte level／XP／next／skill points），來源可見才投影，沒有殺手／目標／隱藏敵人資訊，不覆寫 cooldown。
- 查找曾使用 Windows literal outcome* 路徑，已改目錄 -g；test acceptance 起初猜錯 AttackTarget／AttackMove kind，讀取 server 真正映射後改成 3／11。首次 patch 猜成 slot loop 不匹配，核對实际 hero loop 後才套用；新增 action／補丁都必須先讀取正式來源。
- 新 fact 首次編譯 E0308：VisibilityPolicy 需要 String，不是 &str，改與既有 emitter 相同的 to_owned，不改 ABI 型別。
- tick 129 新差異：team 1 可見敵塔 replica 4，authority HP 360，replica 提前由本地 projectile 預測成 315。單路 NPC／建築沒有在 filtered world 持有 AI、基地保護或完整投射物規則，不能同時本地預測其傷害與吃權威結算。
- 決定：只在已有單路 delta metric 的 filtered world，把當前可見非英雄 kind=2 目標標為 authority combat；其 HP／死亡走既有 committed vitals／lifecycle，自己的正式 input、script/cooldown 與英雄 gameplay 仍重演。不安裝 MobaMatch、不傳 hidden target、不修改 TD／FOG 非單路規則，也不逐 tick repair 全世界。
- tick 522 暴露 owner 自動普攻 clock 邊界：NPC 的 commit lifecycle 可在 filtered pre-step 先退休，authority 本 tick 普攻仍看到 phase 內目標。修正設計：CommittedAttack 的 13-byte phase／clock／sequence 也提供 owner；不提供 target，owner cooldown 仍嚴格禁止 override。此前「owner attack 一律不可結算」假設不成立，更新測試與設計，不宣稱既有回歸證明不存在這個邊界。
- tick 1463 再發現可見敵方英雄的本地自動攻擊／projectile 重播也提前扣 owner HP（expected raw 340256、actual 291718）。只有 NPC 停止預測不夠：對手私有 target／input 本來就不鏡像，不能重建完整攻擊 flight。
- 最終邊界改為單路所有已披露 target 的 Damage 結算由 authority vitals／external effect 提供；filtered 仍執行 owner inputs／scripts／cooldowns、視野內姿態，不重播對手私有玩法。這不是忽略 hash 或停掉 owner技能：四槽 own cooldown 仍不得 override，完整 canonical hash／實際 HP 仍逐 tick 要求吻合。非單路不安裝 authority-combat targets。

## E079：重負載並行驗收讓到達位置的短暫 snapshot 未被呈現

- run 1791033230 失敗且保留：team 1 四槽 applied／cooldown 通過，team 2 未施放；沒有 three-way FAIL。runtime team 2 的正式移動證據到達 x=1400（tick 7315），但 UE owner frame 維持 x=2272，之後收到死亡／重生，未觸發 arrival latch。
- 觀測：runtime inbound_depth 曾達 438、每秒只前進約 40 ticks，而 authority 約 100 ticks；同時執行 Editor PIE 與 Fyrox cargo check。資源競爭是待隔離驗證的推論，不宣稱根因已確認，也不把此次逾時當成功。
- 決定：所有其他編譯與 PIE 已結束，先在獨立負載條件重新驗收；保留原來 gate、正式技能與視野規則，不透過 teleport／假 HUD／修補 hash 通過。
- 本輪查找又對 Windows 路徑使用 literal *.rs 而失敗，後續已使用目錄配合 -g '*.rs'；不能將空結果解讀為不存在。
- 獨立 run 1791033455 同樣失敗，排除「僅 PIE／cargo 並行」的推論。main.rs 每個 presentation interval 即使 catch-up 已設定 publish_latest_snapshot=false，仍會 snapshot_envelope／fog derive 再直接丟棄；移動時 exact-center cache 不命中，計算浪費讓 backlog 自我放大。
- 修正：在建置霧區與記錄 presentation envelope 之前決定 Latest／Critical／不準備。lifecycle 保持先處理，input-bearing snapshot 即使 catch-up 仍走 Critical FIFO；每 applied tick damage ledger 不改。新增四種 route 組合測試，沒有降低 authority tick 或關閉 hash gate。
- 檢查另外猜了不存在的 ue_stage_guard.lua／demo_projection.rs，已改實際 main.rs／presentation_bridge.rs 與 build_ue_moba.lua 來源，不將 failed rg 當驗證。
- 全 repo diff --check 發現 omb/Cargo.toml 既有 CRLF 行尾在其 Git 設定被視為 trailing whitespace；本輪沒有修改該 manifest，不擅自重寫。檢查本輪修改的實際來源另行執行。
- `build_ue_moba.lua --verify` 是不存在的參數；讀取實際 parser 後用 `--verify-staged-only` 通過，staged SHA-256 為 0a3de7bb2f17e75a891321d1e01d72ba050a9d98b2309a04b686136b5a3f41bf。不得再次猜 CLI 選項。
- 修正後 run 1791033665 exit 0／success=true：兩隊各 4 applied／4 positive cooldown、最後施法 tick 6509 後三方 parity gate 到 tick 6600（65／87 PASS），完整結束 checkpoints 67／89 PASS、零 FAIL。第一次真實驗收已確認效能修正能避免重現 stale arrival，仍需重跑確認，不宣稱整體效能基線已完成。
- 第二次 run 1791033806 exit 0／success=true：兩隊各四槽通過，三方 61／67 PASS、零 FAIL 到 tick 6360；五程序精確 PID 清理 verified。runtime lib 41／binary 3 與 Lua gate tests 再通過。兩次共同確認此修正，不是增加 timeout；仍不等於正式 6.5 效能門檻封關。

## E078：敵方英雄交戰普攻 phase／clock 無權威投影

- run 1791032353 不通過：team 1 四槽全成功、team 2 三槽成功後串流終止；最早 team 2 tick 6360 的 hash FAIL，不刪除失敗紀錄。
- 逐 component 比較 authority expected 與真正 runtime dump，唯一差異為可見 hero replica 25 的 TAttack schema 1179600646：authority asd_count=948／Backswing，replica -306／Windup，attack_seq=5 相同。不能用射程、延長逾時或 component repair 隱藏此差異。
- 決定：opt-in MobaMatch commit hook 發布 typed 13-byte attack clock／sequence／phase，只向可見敵方投影，不帶敵人 target／input／完整 JSON。own-team 不收 override，仍須自己的正式 deterministic gameplay 正確；新事件 fail closed 驗證长度與 phase。
- 回歸與新 staged artifacts／真實雙 UE gate 完成後追加結果，不預填成功。
- 首次新增 handler 誤匹配同名 for-loop，放到 movement 函式而非 apply_disclosed_events，導致 E0425 world 不存在。依實際函式邊界移至 post-gameplay disclosed apply；core 303 通過，新增 idempotent／數值保存／malformed phase 與 hidden／owner-clock 不覆寫檢查。
- 擴充四槽回歸第一次把「owner team 不收自己的 attack override」誤寫成「該隊完全不能收到 attack event」，連可見敵人都拒絕；修正測試逐 subject 檢查披露 team，不取消 owner guard。技能後延伸 120 tick 真正比較兩隊全部 canonical hash，不只比較四個瞬間。
- 最終 full build 54205 exit 0；修正 catch-up 浪費後兩次真實 UE 1791033665／1791033806 各四槽通過、三方 pre/post 零 FAIL。完整證據在 four-ability progress；未加入 proactive repair 或關閉 fail-closed。

## E077：安全目標的 generation-packed 身分被誤當 u32 索引

- 真實 run 1791031862：兩隊 self skills applied／cooldown 通過，但指定可見敵方的 slot 0／2 status=4，沒有 authoritative cast。不是射程調大即可解決，失敗 run 保留。
- 根因：TeamViewProjector 以 `(generation << 32) | ECS index` 建立 server validation canonical ID；`rewrite_secure_target` 對整個 u64 用 u32::try_from，所有正常 generation > 0 都失敗。
- 決定：既有 validate_and_resolve 完成 team／view epoch／disclosure epoch／visibility／owner 檢查後，僅在權威內部轉換取低 32-bit ECS index；canonical identity 不回傳 renderer。不取消安全驗證，不把 client opaque ID 當 ECS index。
- 回歸加入 generation 7 的 cast／attack index 42 轉換與不合法 action 拒絕；真正雙 UE 四槽與三方驗證後補記結果。
- 最終 server 139 passed（1 ignored），兩次真實雙 UE 1791033665／1791033806 的兩隊 unit-target 槽 0／2 均 status=0，並有 positive cooldown／post-cast 三方 hash，確認不是只有內部函式測試。

## E076：施法 API 不等於原生四槽操作與實際結算

- 檢查：原生 controller 尚無 Q/W/E/R 綁定；既有 SubmitCast API 與本機 queued 不能證明後端套用及冷卻。決定補通用 owner 槽位輸入、opt-in smoke 與 input-ID 關聯的 applied／cooldown gate，不寫角色專屬 C++。
- 模組邊界：OmGenerated 已依賴 OmRuntime，不能反向 include 造成循環；原生 controller 使用 reflected 通用函式。新增 InputCore 明確直接相依，不倚賴間接連結。
- 工具錯誤：本輪又猜測不存在的 OmGameplayInputComponent.cpp、OmRuntimeTypes.h 與少 `_test` 的測試檔名；後續先使用 rg --files／目錄內搜尋。不得將找不到檔案當成功檢查，也不要把重複猜路徑合理化。
- 編譯／實際端到端驗證結果完成後記到本輪 four-ability progress；不得提前勾選整項 4.1／6.2。
- 首次 build session 59396 因 C4458 失敗：smoke 的區域 `Owner` 遮蔽 AActor 成員，改為 `OwnerHeroFrame`；不降低 warnings-as-errors。發現 controller 綁定依賴 HudRootWidget，改為只依賴 World，避免沒有 HUD Blueprint 時技能也不可用。
- 真實 run 1791031584：smoke 在 approach move 的同個／接近 tick 施放自身治療，正式施法中斷移動，英雄無法進入射程。改成安全 owner frame 確認距交戰位置 30 內才施放，不修改 gameplay 射程或使用 teleport。保留原始 run，不把兩槽 self-heal 成功當四槽完成。
- 活躍寫入中的大型 checkpoint 讀取曾造成 PowerShell Substring 型別錯誤，未取得計數不當證據；完成後再讀取並明確限定 string／完整換行 records。
- 後續 1791032111：修正安全身分後 team 1 四槽全通過、team 2 指定目標 slot 0 通過，證實原拒絕原因；但 arrival guard 被錯誤用在每次技能，正常 cast／追擊移動離開測試起點後停止其餘槽位。改為一次性 arrival latch，後續仍以當前 disclosed target 與正式距離驗證，不鎖住英雄位置、不放寬 gate。
- 一次 `cargo test --lib anti_probing` filter 未匹配任何測試，不算安全驗證，隨後 core 完整 302 tests 通過。再次 Windows wildcard literal 與估計 run ID 搜尋失敗已改用 -g 與實際目錄清單。
- 最新 Editor 直接 run_automation_tests 回報尚未 discovery，不當成測試失敗或通過；先經既有 NativeVisual 流程完成 discovery 並等待 ready，再執行 GameplayInputSurface。每次 Editor restart 都必須重新確認 discovery。

## E075：驗收用自動移動沒有隔離出正式遊戲

- 檢查：WorldBridge 的 28 秒 approach move 原本無 smoke flag，會在一般互動遊戲自行發命令，干擾玩家。
- 決定：只有明確 `-om-presentation-smoke` 才可發驗收移動；固定 Lua 入口只在正值 OMOBA_UE_SMOKE_SECONDS 時加 flag，正常入口沒有這個 flag。不要用測試機器人的動作冒充玩家操作。
- 驗證：最終 rebuild session 8153 exit 0，run 1791030915 帶 flag 的真實雙 UE movement／owner HUD gate 通過；不帶 flag Editor NativeVisual 兩輪各 7/7、PIE success 並停止。程式明確以 flag guard 命令發送，Lua 正常互動入口不加 flag；完整普通互動長時間無自動命令的觀測仍非本次 fixture PIE 的證據。詳見 HUD progress；不改 root bat 或加入 fallback shell。

## E074：正式技能未向可見對手同步已結算狀態

- 初始 fixture 忘記 server 的 CanonicalAcceptedInput 邊界，因此 owner replica 沒有施法；補上 stripped protobuf input、projector 重寫可見 target 後，owner 通過，opponent 仍缺 cooldown 與傷害後 HP。不能把這兩種原因混為同一 bug。
- 修正：僅 opt-in MobaMatch 的 serial outcome reduction 產生 typed `CommittedVitals`／`CommittedCooldown` facts；只帶 fixed-point HP／maxHP 或 slot／remaining，不傳 JSON world、敵方輸入或 canonical identity payload。按真正 visible source 投影，隱藏 actor 不發布；owner cooldown 不發 committed override，仍須由自身 accepted input 正確重演。
- 同 tick 多次 outcome reduction 使用獨立的 serial ordinal，post gameplay 套用絕對結算值，不把正常同步塞進 ComponentRepair；非法長度／負 maxHP／槽位／冷卻 fail closed。
- 測試：四槽逐次比較兩隊 canonical hash、实际 HP 920→740／400→470→550，enemy input 不發布、owner cooldown 不由 committed override 掩蓋；新增 hidden actor／無 canonical payload／malformed 不部分修改／absolute replay 測試。
- 開發錯誤：filtered_specs 未匯入 Fixed64，改用明確 omoba_sim 路徑；隱私 fixture 首次誤把同隊 actor 當對手，修成 explicit team 1 projector／team 2 baseline。再次猜測兩個不存在的來源路徑搜尋失敗，後續均改目錄搜尋；必須持續遵守先列來源規則。
- 這段是 production filtered gameplay 測試，不是四技能由 Unreal 按鍵送出與完整終局驗收；artifact／UE 需重新建置後再記最終證據。

## E073：HUD 首次真實雙 UE 驗收逾時，不能用編譯與單元測試冒充成功

- 失敗 run：interactive-ue-1791028402，success=false。server/runtime 維持運作至 team 1 tick 9309；UE 已 LoadMap／StartRuntime，但沒有第一個呈現／consumed snapshot 日誌，smoke 60 秒 gate 逾時。
- 已排除：讀取原始 presentation.capture，bridge 真正 protobuf decoder／HUD 身分驗證／frame conversion 重播 4654 snapshots，4654 HUD 皆合法；不是後端停止或 HUD schema 全部被拒絕。
- 決定：保留失敗 capture，新增可重跑的 opt-in capture test；加入一次性的第一 Tick／首個 HUD frame 階段診斷，定位阻塞位置，不刪 HUD gate／hash gate 或無根據加長 timeout。
- capture test 首次缺 prost 直接相依，已加 dev-dependency；曾假設 Cargo 有 dev-dependencies section 使 patch 匹配失敗，讀取實際檔案後補 section。遵守依實際 manifest 操作。
- 另有 UE 5.8 r.Mobile.VirtualTextures deprecated handled ensure 與可選 profiler DLL 缺失；它們在啟動後繼續，不先把既有引擎警告當成 HUD 阻塞原因。
- 後續兩次 1791028917／1791029014 真實雙 UE gate 通過，未重現首輪零 frame；原因尚未確認，不宣稱根治。診斷 rg 搜尋忽略的 target/logs 目錄需要 `-uuu`，否則空結果不代表沒有日誌。

## E072：HUD 契約與 API 必須從實際來源確認

- 問題：本輪曾誤稱 snapshot 有 hud_view；實際 protobuf 沒有。猜測 AbilityId::from_str_id 導致 E0599；新增 HUD derive 時 OmEntityRef 缺 Debug/Default 導致 E0277。另一次猜測 OmGenerated plugin 路徑與 Windows wildcard 路徑搜尋失敗。
- 修正：讀取真正 protobuf、generated ability_by_name API 與 rg --files 清單；新增可選的 typed moba_hud 欄位、為 C ABI 明確升版 4，補齊零值 entity 的 derive。
- 決定：只顯示 configured player 的已披露英雄；HUD 是持續狀態，不依賴一次性特效。現有 Hero 沒有 current mana，不捏造滿魔數值；mana_supported=false，介面明確標示未支援。
- 預防：查證實際型別／路徑再編寫；產生 protobuf fallback 並驗證所有消費端；重新 staging header/DLL 後才驗證 Unreal。完成證據見本輪 HUD progress 文件，不把未完成介面列為驗收完成。
- 測試編譯另發現 Fixed64 不由 core runtime 重匯出，且 client 沒有直接 omoba-sim 相依；補上測試專用 dev-dependency 與明確 import，不依賴 transitive crate 名称。

## E064：單路 headless 成功不代表正式安全投影可用

- 程式檢查：單路只有 CircularVision，正式 Wave B 只讀 VisionSource；陣列 side 0/1 被當成 wire team，與正式支援 1/2 不一致。兵線移動沒有 Movement fact，兵塔可見來源傷害也不能由沒有該 AI 的 filtered client 重算。
- 決定：明確分離 side／authenticated team，加入 OwnerTeam scope 與真實視野來源，兵線發布可見 Movement；NPC 傷害用專用 external-direct-combat policy，只送可見目標結果，不暴露來源 ID。保留 TD 預設，不用全圖可見、停用 hash 或每 tick 全世界 repair 通過測試。
- 同步檢查：死亡實體已不在 view.entities，原 visibility loop 不會退休舊身分。以已公開的既有 remember policy 退休缺席身分，不藉此發布隱藏死亡資訊。
- 操作錯誤：本輪兩次猜測 system_dispatcher／visibility 路徑失敗；後續實際 dispatcher 在 shared native。首次新測試把 PaddedTeamFrame 當 TeamTickFrame、Death 猜成 entity 欄位；依實際 wrapper／proto／既有測試修正，不重複盲試。
- 驗證：core 299、base_content 62 passed；真實 KCP run 1791026421 success=true／cleanup_verified=true，雙隊 16／7 checkpoints 三方一致。完整技能／終局／UE 邊界未完成，見 `2026-10-03-single-lane-runtime-progress.md`。

## E065：replica gameplay 初始化與 Warmup 契約缺漏

- 首次雙隊測試在 tick 120 攻擊 timer hash 不同。逐 component 比對發現 filtered hero 缺 FacingBroadcast，未進入 HeroTick join；MasterSeed ECS resource 也未取 TeamGameStart.global_seed；idle jitter 使用 authority/local 不同的 Entity.id。
- 決定：补正 shared seed／必要 hero runtime components，owned hero jitter 用已公開 player ID；同隊仍友好，不同 Player team 明確敵對。
- 真實 KCP run `moba-runtime-1791024537` 再抓到 Warmup：server 冻結玩法，replica 卻繼續更新 timer，observer 多次恢復後 fail-closed，未降級協定。該失敗 run 已保存 report/logs，三個自行啟動的程序清理 verified。
- 決定：每 tick 發布無私有身分的 global HUD delta metric，Warmup／Finished／host Pause 的 delta=0；client 同樣跳過玩法而繼續處理 frame。測試改用真正 2 秒 Warmup，不以設 warmup=0 繞過正式問題。
- 測試工具初次錯把完成 snapshot 的 tick 當 next replica tick；改成正式 server 相同的 committed tick+1。保留每個 checkpoint hash 比對與診斷，不關閉 mismatch。
- SimulationDriver 的 inactive early return 原本回傳空 facts，會把新增的控制 facts 留到下一個 Playing tick；已改成每個 inactive tick 同樣 drain／驗證 facts。包含 2 秒 Warmup 與 host Pause 的雙隊 900 ticks 測試通過，保留全 checkpoint hash。

## E066：正式 server 只更新 local_tick，沒有更新 ECS Tick

- 真實 run `moba-runtime-1791025028` 使用 default Warmup，tick 120 components 一致，但 tick 240／360 的 TAttack.asd_count 不一致。透過 server expected-components 與 runtime components 逐欄比較，排除位置與版本。
- 原因：State::tick 只更新 transport local_tick；ECS Tick 一直是 0，HeroTick jitter／damage RNG／script facts 都用錯 tick。transport 投影卻標成 240，client 使用正確 240；之前 FOG demo 沒有戰鬥 timer，因此未暴露。
- 決定：在 tick 起點同步 ECS Tick=local_tick，再執行時間、輸入與玩法；不改 seed、不修補 hash、不降級 secure V2。兩次後續失敗 run 也已保存診斷並 verified 清理。

## E067：filtered runtime 戰鬥 registry 缺漏

- 真實 KCP run `moba-runtime-1791025237` 在約 tick 898 普攻進入結算後 panic：缺少 TowerTemplateRegistry；server UDP 10054 是 client 崩潰後的結果，不是根因。
- 決定：空 filtered world 初始化必要 registry；初次 bootstrap／rebootstrap 從既有 script registry 建立塔、升級、能力靜態內容，絕不初始化 Story 或隱藏實體。新增空世界 registry 回歸測試。
- 操作防錯：本次仍猜測 scripting/registry.rs、facts.rs、movement_tick.rs 不存在；以目錄 rg 找到實際 stable_fact.rs，後續不使用猜測檔名。
- `moba-runtime-1791025680` 無崩潰且雙隊前進，但舊 smoke 只檢查移動，不能當作同步通過；補上三方 checkpoint gate 後重新驗證。

## E068：可見敵英雄仍缺少敵隊移動優先狀態

- run `1791025237` tick 840：可見敵英雄 TAttack 在 authority 為 Idle，observer／外部 runtime 卻同樣進入 Backswing。兩個 client 相同不代表與 authority 相同。
- 原因：安全投影不傳敵隊 accepted input，filtered hero 沒有敵人的 MoveTarget，因此在敵方正在移動時錯誤啟動自動普攻。
- 決定：新增每 tick、僅可見 actor 的 MovementPriority 布林 fact。它不攜帶目的地、敵方輸入或 target ID；replica 只據此套用與 authority 相同的普攻移動優先門檻。目的地與真實移動仍由原有安全 Movement outcome 處理。
- 驗收門檻：雙隊至少前進 900 ticks，每隊至少 6 個三方 checkpoint，包含 tick 840；要求 authority／observer／external pre-repair 與 post-repair hash 一致，不能用 repair 或 client 相互一致冒充成功。結果另見本輪進度檔。

## E069：雙隊較晚加入與最終姿態不可只驗證早期 tick

- run `1791025876` 的同步全部通過但 gate 在第二隊只到 913 時要求六個 checkpoint；第二隊於 tick 240 後加入，960 的第六份報告尚未收到。改為至少 1080 ticks，保留六個 checkpoint 門檻，不減少 hash 檢查。
- run `1791026020` 因第二隊 catch-up 使第一隊走到 2105；tick 1800 敵英雄只有 Facing 不一致（authority 486、client 2532），其他 components 相同。Movement 只帶移動階段姿態，後續轉向／抵達完成不一定再次送出。
- 決定：單路 commit hook 發布 visible hero 最終 Pos/Facing 的 Movement outcome，phase=PostStep，仍受既有 visibility policy；不傳 MoveTarget、私有命令或隱藏 actor。新的敵隊移動回歸測試延長至 2400 ticks，原空輸入 900 ticks 測試保留。
- run `1791026167` 的 gate 讀取仍在 append 的 JSONL 尾行，報 unterminated JSON string。只讀以換行完成的 records，未完成尾行不當證據；完整行若 malformed 仍失敗，不吞掉同步錯誤。

## E070：NPC PreStep 運動被 client 延後到 PostStep

- 新敵隊移動回歸延長至 2400 ticks 後，tick 1800 的兵 HP 差 45（一發英雄普攻），authority 135、client 90。這不是前一個 Facing mismatch，不能重跑短測冒充修正。
- 原因：authority lane hook 在 dispatcher 前移動 NPC；舊 wire Movement 在 replica dispatcher 後套用，因此 replica 用上一 tick 的兵位置做攻擊判定。
- 決定：新增 PreStepMovement event kind（沿用僅可見 Pos 的 payload 與既有 policy），於 replica dispatcher 前套用。Hero 最終姿態仍是 PostStep Movement，不把所有位移一律提前，不暴露 NPC AI 或私有目的地。
- 驗證：原 900 ticks Warmup／Pause／NPC 傷害與新 2400 ticks 雙英雄移動測試均通過，每個 checkpoint 仍比對 authority hash。最終真實 KCP run 另記。

## E071：Unreal 單路 smoke deadline 早於測試移動

- run `interactive-ue-1791026808` 五程序單路已顯示英雄／兵／塔，兩個 runtime 持續前進；但 30 秒 smoke 尚未出現 issued approach move，沒有 scripted-move evidence，不能宣告通過。
- 已讀實際 C++：ApproachMoveAtSeconds 是首次 FocusCameraOnOwnedHero 的 wall-clock +28 秒；UE readiness 早於首個可消費 snapshot，冷啟動佔去額外時間。改以 60 秒重新跑，仍要求 UE 自己提交、rendered movement 和 consumed snapshot，不改成 runtime 代送。
- 本輪操作又猜測 codegen/templates／Windows `OmWorldBridgeActor.*`，rg 失敗；改直接對實際 OmGenerated 目錄搜索。不因搜尋不到就新增同名模板；不將 runtime evidence 缺檔當成 JSON parser 問題。
- 清理核對初次把含空行的 pid 檔直接 cast 成 int 失敗；改讀 Raw.Trim 後逐 PID 查詢。停程序後 JSONL 仍可能有未寫完尾行；同樣只解析換行完成的 records，不能用 parser error 推論 runtime failed。
- 最終 run `interactive-ue-1791026989` success=true：雙隊 safe tick 5321、雙 UE 自行移動且各 15 個 rendered frames／正確 consumed sequence；各 46 筆三方 PASS、零 FAIL，五個精確測試 PID 均退出。完整 HUD／技能／终局仍未完成，不勾选完整框架。

## E059：單路對局實作的路徑與 API 假設

- 現象：只讀查詢再次猜測 native/mod.rs、game_processor.rs、runtime/src、hero_command.rs；Windows rg 不展開 comp/*。首次編譯另發現 CircularVision::new 只有兩參數、AttackMove 欄位是 pos、Vec2I 是 i32；commands 變數插入到 digest 而非 Bot scope。
- 決定：改用 rg --files／對目錄搜尋，再讀實際定義；修正呼叫，限制 lane_length 避免輸入 raw 溢位，保留命令狀態在 digest 並在 Bot 自己取得 storage。後續驗證以實際 cargo 結果為準。
- 診斷輸出首次使用 Fixed64 的 Display，該型別只支援 Debug；改為 {:?}，不為方便輸出添加模擬浮點轉換。
- loader.rs 初次仍猜測位於 native/scripting；實際在 omoba-core/src/scripting。格式化後 patch context 未匹配時工具原子拒絕；重新讀實際區段後套用，不混用格式化前的片段。

## E062：兵塔傷害不能只繞過戰鬥計算直接扣血

- 程式檢查：首版 lane AI 發出 Outcome::Damage，雖經正式結算但未經 DamageInstance 的護甲／暴擊／DirectCombat fact 管線；Unit::new 預設數值與 CProperty／Lua 英雄數值不一致。
- 決定：兵塔提交 DamageInstance::new_attack，交由既有 damage_tick 生成結算及可觀察戰鬥事實；英雄 Unit／CProperty／TAttack 取同一 generated Lua stats，重生 HP／傷害包括既有等級成長。修改後重新執行對局與逐 tick 重播，舊報告不能當成新碼驗收。
- 同步修正 headless SimulationDriver 把 tick 誤傳為 script rng_seed（正式 server 使用 MasterSeed）；每 tick drain 並驗證 OrderedFact，不累積整場未讀 facts。新的 facts 是回傳資料，不在 renderer 重建權威事件。

E059–E062 最終驗證：base_content 59 passed、core 296 passed、backend 135 passed／1 既有 ignored、core no-default-features 編譯通過。固定 Lua + 真實 release DLL：15Hz seed 1，3155 ticks／27 波／[11,1] 死亡與重生／864 combat facts；120Hz seed 1，15480 ticks／16 波／[7,3] 死亡／[7,2] 重生／953 facts。兩場 team 0 勝、game.end 各一次，所有 tick replay 一致。詳細 digest／指令與邊界见 `2026-10-03-single-lane-moba-progress.md`。15Hz／120Hz profile 間不宣稱相同平衡結果；沒有 Unreal 完整對局驗收。

## E063：CRLF 被 submodule 的 diff whitespace 預設當成尾空白

- 現象：根 repo diff --check 通過，但 omb/Cargo.toml 新 bin 宣告的 CRLF 在 submodule diff --check 被報 trailing whitespace。
- 決定：保留既有 Windows 行尾，不改 shared git config；只讀檢查使用 `git -C omb -c core.whitespace=blank-at-eol,blank-at-eof,space-before-tab,cr-at-eol diff --check`，仍檢查真正尾空白／EOF 問題，該檢查通過。
- 預防：report 路徑可能仍有上次成功檔，必須等待本次 Lua／CLI exit 0，再讀其結果；本輪等待 120Hz 完成後才引用新的 15480 ticks 報告，不能把運行中的舊 16594 ticks 報告當成本次通過。

## E061：首場 Bot 對局達到時間上限卻沒有結束

- 現象：真實 ECS／正式输入測試 4500 ticks 後仍 Playing，38 波、9 死亡／9 重生；其餘四個對局規則測試通過。
- 決定：先印出失敗對局的實際位置、血量與命令找出死局，不以延長上限或直接宣告勝方通過驗收。原因／修正及最終結果追加於此。
- 診斷：敵塔已被摧毀，但兩座基地都滿血；Push 保留 AttackMove，反覆停下清兵／敵英雄，沒有完成建築目標。修正：技能在合法範圍內優先當前建築目標，敵塔倒下後正式提交 AttackTarget 基地；相同命令不重送，避免每 tick 打斷普攻前搖。

## E060：戰鬥結果依賴 wall-clock 負載與平行收集順序

- 程式檢查：hero_tick 以本次系統耗時 <50ms 決定是否攻擊；par_join outcomes 未按 entity 排序，同距離目標缺穩定 tie-break。
- 決定：只以 fixed delta 判定是否推進攻擊；按 entity reduce，搜尋同距離以 ID／generation 排序。對局以逐 tick digest 及正式輸入重播驗證，不把最終勝方相同當成完整一致。

## E058：直接刪除 Saika 分支會移除既有 Blueprint 的 reflected 介面

- 程式檢查：通用事件模板內嵌 Saika ID／技能判斷與 typed payload；現有 Editor BlueprintSurface／SaikaEventDispatch 使用 HandleSaika*／FSaika*。直接刪除不是無回退的模板遷移。
- 決定：先隔離 legacy_hero_compat adapter，通用模板不再直接判斷角色；保留舊名字、manifest shape、生成 bytes 及 ABI。新英雄不得經此 legacy 層擴充。真正資產／typed projection 遷移另驗，不將隔離冒充完整移除。
- 驗證：初次搬移後既有 codegen 22 tests 與 shipped --check 全過、輸出不變；新增防回退與 UE 驗證結果見 generic-event-adapter 進度檔。本輪搜尋不到可用 Blueprint inspection 字串（rg exit 1）只代表需查工具文件，不能憑空假設已有 API。

E058 最終驗證：codegen 24 passed、shipped --check 11 files／13 inputs unchanged；full build 31311 exit 0、Editor 68560、stage SHA `c5736f6bb22e92cef41c4edd53eba413305bcffbad8c7817db12f2c3ce9a6863`；native 10981 同 Editor 兩輪各 7/7、PIE 75028 native／ghost rendered=true 且 stop。MCP 查實際 Blueprint skeleton／graph intent：31 nodes，legacy HandleSaikaActionEvent 接到 ABP 動畫更新，不是可猜測為空的舊入口；未改資產圖。不將隔離相容模組誤算成 2.2b 全部完成。

## E053：runtime 呈現節流與 latest 通道仍可吞掉未消費傷害

- 程式檢查：last_injections 每 simulation step 取代，原本只有發布 snapshot 才取 external effects；即使 bridge 保留 busy／published slot，也接不到此前被節流或 watch overwrite 丟失的事件。
- 決定：每個已套用 step 先只 drain DirectCombat，其他持續 state effects 保留。runtime 有界 ledger 跨發布保留 DMG1，最多 1024 筆、4096 ticks；記錄第一個成功寫入 socket 的 snapshot／connection，只有該連線有效 ACK 才退休，不把 prepare／publish／release 當消費。reconnect baseline 不重播歷史；Hide／Forget 僅清理同 disclosure epoch，ResetView／新 view epoch 清理。送出前再次過濾已退休事件，涵蓋 ACK 前建立但 ACK 後送出的 envelope。
- 邊界：capture 只接受已套用且有序的 replica step，不是任意外部重送 API；過期／超量為明確丟棄，第一筆超量記警告。只涵蓋 DMG1 外部直接傷害，不宣稱所有音效／技能 reliable delivery。
- 驗證：core 296 passed、runtime 38＋2 passed；新增真實 TCP latest overwrite／ACK 退休／過時 prepared frame 過濾，以及容量／TTL／epoch／baseline 單元測試。完整 UE 驗證另記進度檔。

## E054：新增測試 patch 含空的 update hunk

- 現象：apply_patch 拒絕「Update hunk does not contain any lines」，整份修改未套用。
- 決定：重新提交含實際替換內容的原子 patch，確認 test helper 可見性與 socket 測試都存在後再測，不假設部分成功。
- 驗證：新增 retention／TCP 測試均已編譯且通過。首次測試另有 cfg(test) wrapper 的 unused_mut，已移除；保留正式 serve receiver 的 mut。

## E055：清理核對把含換行的 PID 檔當整數陣列

- 現象：只讀核對使用 `[int](Get-Content path)`，PID 檔含額外 CR／空行，PowerShell 得到 Object[]，整數轉換失敗，後續 Get-Process 的 Id 為空。最末 diff 指令成功讓整條 command exit 0，不能據 exit code 宣稱前面檢查通過。
- 決定：讀 `Get-Content -Raw` 後 Trim，先驗證 `^\d+$` 再轉數值；只查該 run 保存的五個 PID，不以程序名稱終止任何東西。沒有執行刪除／終止。
- 驗證：run 1791018681 的 server 71164、runtime 66260／73736、UE 30292／35540 全部 StillRunning=False；active-ue-session.json 不存在。雙隊 smoke 本身 exit 0，保存 observation 複驗成功。

## E056：空傷害 ledger 不應在每個 presentation frame 建 entity 集合

- 程式檢查：有界 retention 投影為避免 events×entities 掃描而建立 BTreeSet，但即使沒有待送傷害也掃描全部 entity，對 stress 熱路徑增加無效成本。
- 決定：entries 為空直接返回；有待送事件才建可見身分集合，仍精確比對 epoch。屬實際效能風險修正，不將未執行的 10000 entity benchmark 冒充已通過。
- 驗證：修改後重新執行 runtime／bridge 回歸，再重新 build／stage；不能沿用修改前的 DLL SHA 或 UE 報告。

## E057：長時間 renderer 離線時 saved latest tick 不是新的恢復時點

- 程式檢查：presentation_enabled 的 30 秒 grace 結束後 latest snapshot 停止更新，但 runtime 仍進行 replica steps。新 renderer 若只以 saved snapshot tick 清 ledger，離線期間事件可能被當成新 cue 重播。
- 決定：恢復基準為 max(saved view tick, ledger 已套用 high_tick)，所有初次／late-first snapshot 路徑使用此回傳 floor；不將舊 view 偽造為新的 state tick，只提高歷史事件排除界線。
- 驗證：baseline 測試以 saved tick 2／current tick 3 驗證兩個歷史 cue 全部退休，tick 4 新事件可保留；現有真實 socket overwrite／late-first／reconnect 測試與 bridge 回歸再跑。最後 full build／stage／UE 報告另記進度檔。

E053–E057 最終驗證：core 296、runtime 38＋2、bridge 39＋2 passed（1 外部 KCP ignored）。最終 full build 46020 exit 0，Editor 36276／MCP 30000；built／staged SHA `c7288cd34861753bab3656c0146eec0e4f8331d197908b34d59a7c1d23ca34f0`。native 14423 同 Editor 兩輪各 7/7、PIE 26431 native／ghost rendered=true 且 stop；dual 67749／run 1791019365 exit 0，兩隊 own-only／UE 與 replica 位移／ACK 1105、1533。五程序退出與 active session 移除已核對，saved observation PASS，最後 stage gate 一致。此證據涵蓋外部傷害管線分段與 demo 移動，沒有將完整戰鬥／所有 cue 或 4.3 誤算完成。

## E046：只讀搜尋再次猜測 runtime.rs／未存在的 projection 函式檔名

- 現象：rg 參數包含 omoba-core/src/runtime.rs，但實際為 runtime/mod.rs；另一次 projection 函式搜尋無符合項目（exit 1 不是建置失敗）。
- 決定：先 rg --files 確認路徑；不把搜尋不到當成可假設的 API，改讀現有 selective_replica／driver 實作。

## E047：patch automation 測試文字未對上

- 現象：多檔 patch 最末測試 anchor 文字不符，整份 patch 未套用。
- 決定與驗證：先 rg 查是否有新增符號（無），讀真實原文後重套，不假設前面的檔案已修改。

## E048：replica ID 與 native snapshot ID 寬度不同

- 現象：bridge 編譯拒絕 u32 entity_id 與 u64 cue.target_id 直接比較。
- 決定：比較時將現有 native ID 無損提升 u64；decoder 對 native 無法表示的 ID／epoch 使用 try_from 拒絕，不截斷。修正後 bridge 39 unit＋2 TD integration 通過。

## E049：擴充測試後重用已 move 的 fact

- 現象：core 測試編譯指出單元素陣列已 move fact，新增雙來源驗證無法 clone。
- 決定：測試第一輪使用 fact.clone()，正式投影不增加不必要 clone；重新執行 core 回歸。

## E050：producer-local ordinal 撞號與 coalescing 丟失一次性提示

- 程式檢查：外部效果 stable_sub_index 原為各來源 local_ordinal，不能充當隊伍內唯一 cue ID；bridge 原本合併 snapshot 只留下最新，舊 snapshot 的 one-shot 會被吞掉。
- 決定：先排序已清洗的效果，再配置隊伍內索引（不加入隱藏來源 ID）；ID=(replica_tick << 32)|(index+1)，overflow 拒絕、不截斷。bridge 保留 batch／busy pending 提示，去除相同 ID，限 1024，僅允許最新可見相同 epoch 目標；reset 清除。Buff／持續狀態不冒充 one-shot。
- 驗證：雙 hidden producer、同 tick／target／local ordinal 的兩件事仍有不同索引，反序輸入產生同結果；typed codec／runtime mapping／bridge decoder、busy retry／coalescing／epoch change／reset 測試陸續加入，不提前宣稱所有 cue 完成。

## E051：重連基準之後的 snapshot 仍可帶入舊傷害

- 程式檢查：僅清除首個 snapshot 的 effects，無法阻止之後的 retained damage 在新 renderer 播放。
- 決定：每條連線保存首個完整 snapshot 的 replica_tick，DMG1 damage 只有 tick 大於這個基準才可出現在後續 live frame；不修改共享 source。沒有舊 cue 時 Cow 借用，避免每 frame 複製整個視野。未知格式維持舊相容邊界，bridge 不會 dispatch 成 DMG1。
- 驗證：late-first snapshot 的 latest／critical 兩種真實 socket 測試，後續 frame 的 tick 3 damage 被剔除，tick 4 保留；不能把這項專用契約宣稱為全部音效／技能完成。

## E052：已發布到 ring 但尚未處理的提示也可能被下一個 view 覆蓋

- 程式檢查：僅保留 busy pending build，不能涵蓋已成功發布、UE 還沒呼叫 mark 的 slot。
- 決定：slot 保留 typed cues 與 consumed 狀態；合併新 view 時攜帶未處理的最新 slot 提示，mark 後才清除。不把 release／publish 當消費。pending／published 均遵循 visibility／epoch／reset 與 1024 上限；UE 同一 ID 重播由獨立 history 阻擋。
- 驗證：bridge regression 新增 published-but-unprocessed → 新 view 仍保留；明確 consumed 後下一 view 不再攜帶，39 unit＋2 integration 通過。

E046–E052 最終驗證：core 295、runtime 34 library＋2 binary、bridge 39 unit＋2 TD integration passed（1 外部 KCP ignored）。full build 74762 exit 0、Editor 98896／MCP 30000；stage SHA-256 2f7c43f332003d04ab712850070ce49607c3ea7ec06707db22a86b4b6d8046e9。native 16540 同 Editor 兩輪各 7/7（含新增 damage dispatch assertions），PIE 86336 native/ghost rendered=true、memory=[1,0]、已 stop。dual 57125／run 1791017466 exit 0：兩隊 own-only／UE 與 replica 位移／ACK 1154、1595；五程序清理、active session 移除。DirectCombat 管線分段驗證，不當作完整戰鬥／所有音效／4.3 完成。

## E042：bridge frame 序號不等於 IPC snapshot 序號

- 程式檢查：Unreal lease.sequence 是本機 ring 發佈序號；driver 原先丟棄 envelope.sequence，控制 frame 也會增加本機序號，不能直接送 RendererConsumed。
- 決定：原始 connection ID／snapshot sequence 只存在 Rust 的 extras → pending build → frame slot；新增 om_mark_frame_consumed，UE 在 ProcessFrame 返回後、ReleaseFrame 前回報。單純 release、控制 frame／lifecycle 重建不 ACK；同 slot 重複 mark 冪等。ABI 結構與 v3 不變，UE 要求新 export 存在，舊 DLL 不靜默降級。
- 驗證：真實 TCP＋C ABI 測試刻意使用 IPC 97 與不同本機序號，release 無封包、mark 正確送 97、duplicate/control 無額外 ACK；busy ring retry／coalescing metadata 保留通過。

## E043：延遲的舊 lease 可能污染新連線，release 原先未驗序號

- 程式檢查：只保存 snapshot sequence 無法區分斷線前後；release 原先只驗 slot pointer，沒有驗 lease.sequence／該 slot 是否仍有 reader。
- 決定：程序內 AtomicU64 配發不重用 connection ID（包含 driver 重啟）；writer 只接受自己代次的 ACK，採最高序號並限制每次最多取 256。mark/release 都核對 slot pointer、sequence 與有效 reader；已釋放或錯誤序號不得修改 reader 計數。
- 驗證：兩次真實 socket 重連，刻意把第一連線的 999 ACK 放入第二連線，第二 socket 只收到自己 snapshot 1 的 ACK。C ABI wrong/released lease 拒絕、仍可正確釋放；37 bridge unit＋2 TD integration 通過，1 外部 KCP 測試維持 ignored。

## E044：只讀檢查猜測不存在的 bridge/build.rs

- 現象：本輪附帶檢查 bridge/build.rs 不存在，雖未修改任何檔案，命令 exit 1。
- 決定：遵循 E018，先以 rg --files 確認路徑；現有 header 產生流程在 omfue/build_bridge.bat，不再猜測 Cargo build script。

## E045：只讀 PowerShell 行數參數誤植

- 現象：檢查 subsystem 時誤寫 Select-Object -Skip sixty，整數參數解析失敗；未影響建置或修改檔案。
- 決定與驗證：改為明確 -Skip 60 後取得完整 export 必要性檢查；之後行數參數僅使用數字，不把文字描述放入指令。

E042–E045 最終驗證：full build 49694 exit 0，Editor 89572／MCP 30000；bridge built/staged SHA-256 5ec882ff08ada09fd2f05987e59b3279ebfc6d33d96ff06511d3b2d177d52ee4。Editor 同一 instance 兩輪各 7/7、PIE native/ghost rendered=true 並停止。dual run 1791016176 exit 0，兩隊 filtered view／UE 與 replica 位移、runtime 收到 snapshot 1090／1772 ACK 全通過；owned 5-process cleanup、active session 檔移除。4.3 仍未封關，正式 cue ID／所有 effects/audio 投影尚待完成。

## E038：renderer 初始安全 snapshot 可能重播歷史一次性效果

- 程式檢查：serve_renderer 原先直接寫出共享 latest snapshot。effects/audio_cues 一旦帶入 retained 資料，新的 renderer process 可重播舊效果；先收到 RuntimeReady、後收到第一個 snapshot 的分支也有相同問題。
- 決定：每條 renderer 連線的第一個完整 snapshot 是恢復基準，只複製一次並清除 effects/audio_cues；可見單位、ghost、fog、view epoch、tick 及 input result 保持原值。latest 與 critical 首個 snapshot 路徑都使用相同規則；之後 live one-shots 仍保留。不修改共享 watch source，不重新啟動 replica。
- 驗證：Rust 真實 TCP 連續兩個 renderer 在沒有新 simulation frame 時恢復同一 ghost，歷史 VFX/audio 不重播；延後首個 snapshot 的兩條路徑亦通過，新 live cue 與 terminal input result 保留。runtime 33 lib + 2 binary passed。
- 尚未解決：目前正式 snapshot builder 仍未映射所有 MOBA effects，RendererConsumed 仍缺少 UE lease sequence → IPC sequence 對應；持續 live frame 的完整 cue 身分／去重契約尚未完成，不能將「初始基準不播放效果」冒充 4.3 全部完成。buff/script 狀態更新不可直接用 projectile key 去重，以免合法重建／更新被吞掉。

## E039：RendererConsumed 可回報尚未送出的未來序號

- 程式檢查：runtime 原先忽略全部 RendererConsumed，沒有區分真實消費與未來序號；不能基於這些回報安全清理 retained effects。
- 決定：每條連線追蹤實際送出的完整 snapshot 最大序號，只接受不超過此界線的回報；duplicate／older lease 回報以 fetch_max 冪等處理，未來序號明確 IPC error 並斷線。input result／RuntimeReady 不能抬高 snapshot 界線。
- 驗證：兩條首個 snapshot 路徑皆以真實 socket 發送有效、重複、較舊回報後仍接收新 live cue；未來序號 999 被拒絕並關閉 socket。runtime 33 lib + 2 binary 全數通過。尚未用此 cursor 刪除 effects，需等正式 cue ID／UE 消費對應完成。

## E040：stage 後執行 bridge Cargo test 重新產出不同 DLL

- 現象：本輪 full build 31119 與双 client run 1791015128 通過後，SHA-256 比較發現 bridge/target/debug/om_bridge.dll 與 Unreal 已 stage 的 DLL 不相同；build 後再跑 bridge tests 會重建 cdylib。即使只有測試 fixture 更新，也不能用 ABI 相同宣稱二進位一致。
- 決定：正式驗收固定為所有 Rust tests 先完成，再 build/stage，最後 Editor／PIE／dual client；stage 後不再執行會重建 bridge DLL 的 Cargo 指令。驗收前比較兩個 DLL SHA-256，若不同先重新 stage，不忽略差異、不在載入中覆寫 DLL。
- 驗證：重新 full build/stage 後核對 SHA-256，再執行本轮最終 Unreal 回歸；先前 smoke 保留，不取代新二進位驗收。

## E041：Lua 測試 fixture 的 and/or 把 nil 缺檔值換成另一檔案

- 現象：stage gate 的缺少 DLL fixture 測試先失敗，因測試 stub 用 `condition and built or staged`；built=nil 會自動落到 staged，沒有模擬真正缺檔。正式 sha256 gate 未使用此寫法。
- 決定：stub 明確 if/else 保留 nil，重新測 identical/mismatch/missing 三種情境；verify-only 不得啟動任何 process，不透過改寫真實 DLL 測失敗分支。

E038–E041 最終驗證：runtime 33 lib + 2 binary、bridge 36 unit + 2 TD integration passed（1 外部 KCP ignored）；full build 31227 exit 0、Editor 10556/MCP ready，build/stage SHA-256 一致。session 7289 同一 Editor 兩輪各 7/7，PIE native/ghost rendered=true、memory counts=[1,0]、PIE stopped。最終 dual run 1791015412 exit 0，兩隊各自 filtered view、UE 與 replica 位移皆通過。stage gate 相同／不同／缺檔三案例、Lua module tests、codegen --check、diff whitespace 通過；4.3 仍留未完成，不冒充完整 cue 消費契約。

## E028：player_id 被當成 team_id

- 現象：bridge 的 start_driver_if_configured 將玩家 ID 直接填入 DriverConfig.team_id，Unreal 也沒有傳遞獨立隊伍欄位。玩家 7／隊伍 2 會送出錯誤握手。
- 決定：C ABI v3 增加 explicit team_id；UE 解析 -om-team／-om-team-id 或 OM_TEAM_ID／OMB_TEAM_ID，不猜測隊伍；IPC 拒絕零玩家／零隊伍。舊 TD 非 IPC 保持相容，舊 ABI 明確拒絕。双 Unreal 啟動明確提供玩家與隊伍，缺少設定不降級直連玩法。
- 驗證：新增 player=7/team=2、缺失 ID、舊 ABI 拒絕測試；Rust 34 unit + 2 TD integration 通過，1 外部 KCP ignored。完整 UE 与双队測試結果另記進度。

## E029：雙 Unreal launcher 只用檔案存在判定建置新鮮度

- 現象：run_2player_ue.lua 即使未要求 skip-build，仍只在 EXE 不存在時建置，可能執行舊 runtime／server；ready 只檢查 LoadMap，不能證明 IPC 與移動成功。
- 決定：非 skip-build 每次執行增量 Cargo build；smoke 模式新增安全 replica 與原生 UE 呈現／移動報告，未達斷言就失敗並保存錯誤。保留互動模式，不用等待時間或 LoadMap 冒充驗收。

## E030：Unreal -log／-abslog 未產出預期檔案

- 現象：真實双 client run 1791012372 失敗於讀取 ue-p1.editor.log；實際全部日志在 host 擷取的 ue-p1.stdout.log。
- 決定：驗收使用啟動時必定擷取的 stdout，不再假設 -abslog 和 -log 同時生效；保留首輪 failure report。

## E031：積壓的完整 snapshots 被逐一投影

- 現象：双 client 第二輪 1791012553 正確拒絕驗收，25 秒內沒觀察到自身移動；日誌只有最初 frame。code review 發現 drain_driver_updates 無界收集，再逐一生成大視野 frame，最後只保留最後 frame。
- 決定：單次最多取 256 updates；只投影最新完整 snapshot，所有 removal identities 與 terminal input results 保留。reset 淘汰先前 snapshot，較新的安全 snapshot 才恢復 view。新增 coalescing 單元測試，再以實際双 client 復驗；尚不以程式檢查認定這是停頓的唯一原因。

- 後續證據：退出後 stdout 刷新顯示 frame 其實持續增加；demo 的 ApproachMoveAtSeconds 固定在相機定位後 +28 秒，25 秒 smoke 不足。改用 50 秒觀察，不降低移動斷言，也不將日誌緩衝等同主執行緒永久停頓。coalescing 是實際可測的熱路徑改善，但不宣稱它解釋所有延遲。

## E032：舊 creep Blueprint 與原生 BodyMesh 欄位撞名

- 現象：dual UE 的 BP_PracticeDummy 編譯回報 BodyMesh 在 SKEL_BP_PracticeDummy_C 與 OmUnitActor 重複；既有 native 自動化沒有測這支舊 Blueprint。
- 決定：新增明確 -om-native-content 選項，生成前端路徑優先走 native registry 類別，避免不必要的舊 Blueprint；保留原資產與未帶選項時的相容路徑，不刪除使用者 graph。

## E033：-game 第一個 frame 等待額外載入的 fog Plane

- 現象：run 1791012933 的 50 秒驗收仍失敗，p1 stdout 最末停在 Waiting on static mesh /Engine/BasicShapes/Plane.Plane being ready before playing；runtime replica 無落後持續前進。原生程式在 EnsureFogOverlay 第一次 frame 額外 LoadObject Plane。
- 決定：fog overlay 重用 constructor 已載入的 Cube primitive，Z 壓薄至 0.001，保留尺寸／高度與 fog mask 材質；不改共享引擎、不編造收到 frame 就已玩得動。以雙 -game client 復驗判斷是否解決卡點。

- 復驗：full build 52197 exit 0；run 1791013302 不再停在 Plane，兩隊都收到自己的英雄，但 50 秒仍未送出移動。frame 日誌間隔偏長，+28 秒的 demo 移動只在 ProcessFrame 觸發，故不能把原網格修正宣稱為完整解法。

## E034：固定睡眠把場景載入時間混入移動驗收

- 現象：LoadMap 不代表相機已找到英雄；從 LoadMap 起睡 50 秒，可能不足以涵蓋首次安全投影、相機定位後 28 秒，以及下一個 presentation frame。
- 決定：smoke_seconds 改為條件等待的上限；兩隊 UE 移動日志與安全 replica 座標變更均存在才完成，逾時明確失敗。保留 filtered-world 與 own-only 斷言，不提高效能評價、不注入 runtime scripted move。
- 操作防錯：本輪再次使用不存在的 runtime.rs／runtime_driver.rs，以及把 Windows glob 直接傳給 rg，造成唯讀查詢失敗；先 rg --files 確認路徑，glob 必須使用 rg -g。未產出文件亦先檢查存在再讀。

## E035：安全 replica 接收佇列落後讓移動過期

- 現象：120 秒條件等待 run 1791013521 中兩邊 UE 都發送移動，但 server 明確拒絕 player 1：original_tick=3138、current_tick=5769、late_by_ticks=2632。runtime inbound_depth=1024，lag=0 僅代表已收到 tick 與已套用 tick 相同，不能代表沒有 server backlog。
- 程式證據：snapshot_envelope 對靜止英雄每次重新執行 demo fog raycast（約 15,000 tiles × 64 trees）；追趕丟棄的普通 snapshot 也做這份重工。
- 決定：每個 PresentationHub 保存一份 exact-center/team demo fog cache；相同完整座標及隊伍才重用，位移 1 raw unit、隊伍變更、空視野/reset 都失效。資料僅來自安全 snapshot，不放寬後端 late-input 規則，不跳過 replica simulation。
- 驗證：新增 exact-key 命中／失效及與未快取結果逐欄相等測試，再跑真實双 UE；尚不以 lag=0 宣稱效能合格。Windows 對仍開啟 File 的 Length 可能尚未更新，看到 capture Length=0 不代表讀不到內容，須實際讀取。

- 首輪 cache 復驗：run 1791013831 exit 0，兩隊 replica 都位移，server 正常套用輸入。加強下一輪驗收要求 UE frame 日誌自身位置亦變更。

## E036：Lua pattern 誤混入反斜線 escape

- 現象：新增 UE 位置解析 pattern 時寫成反斜線加百分號，loadfile 明確拒絕 invalid escape sequence；尚未執行新雙 client 測試。
- 決定：Lua pattern 使用百分號、不用反斜線，修正為 `([-%d%.]+)`；先 loadfile syntax gate，再用已保存的兩隊實際 frame 日誌檢查位移解析，才跑 launcher。

## E037：最終重建再次碰到外部程序鎖住共享 Renderer DLL

- 現象：full build 80262 的 Rust bridge／DLL stage 成功，但共享引擎有其他變更，UBT 重建 Renderer；UnrealEditor-Renderer.dll 連結被 `D:\UE5.8\CrowdRuntimeStable\Engine\Binaries\Win64\UnrealEditor-Cmd.exe` 鎖住，UBA 等待 20 秒後重試仍 LNK1104，exit 1。
- 決定：遵守 E007，僅唯讀核對程序，不能停止其他專案、覆蓋引擎或用舊成功輸出當本次成功。保存 stage 成功與 full build 失敗的區別，完成不依賴 Editor 的 Rust／Lua 驗證；外部鎖解除後才復驗最終 UE build。

- 最終復驗：外部 PID 20476 自行結束；full build 3355 exit 0、Editor 52212/MCP ready。run 1791014362 的加強 live smoke exit 0，兩隊 UE 日誌及 replica 都位移。session 90909 的同一 Editor 兩輪各 7/7，PIE native/memory rendered=true、memory counts=[1,0]、PIE 已停止。bridge 36 unit + 2 TD integration（1 外部 KCP ignored）、runtime 31 lib + 2 binary、Lua module與 6 parser cases、codegen --check 均通過；4.2 封關，整體 15/30，未宣稱整個框架完成。

## E024：完整視野、控制 frame 與局部生命周期不能混用

- 現象：接 UE memory 時發現輸入結果發布空 FrameBuild；若每個 frame 都覆蓋 ghost，輸入結果會清掉記憶。ViewRemoved 用完整空 frame 處理局部 Hide，亦可能清掉無關角色。
- 決定：C ABI v2 增加獨立 remembered ghost 陣列與 presentation_snapshot 標記；控制 frame 不修改 ghost；生命周期只變更已取得的 safe snapshot，完整空視野才清除所有角色／記憶。ghost-only 斷線仍必須發布 reset，不能因 live removals 為空而跳過。
- 忙碌處理：保留最新完整安全 frame 直到有空閒 lease slot；失敗時取回原 build，不複製每個正常 frame，也不重複附加 pending control records。
- 驗證：新增 Rust lifecycle、decode、lease 與 UE marker 自動化測試；最終結果見本輪進度文件。

## E025：UE 5.8 TestNotNull 無法推導 TObjectPtr

- 現象：完整 build session 19431 exit 1，測試 GetStaticMesh() 傳給 TestNotNull 出現模板推導錯誤；其回傳型別為 TObjectPtr<UStaticMesh>。
- 決定：以 .Get() 明確取得原始指標，不修改正式呈現流程。依本地實際回傳型別修正，完整重建後再跑測試。
- 預防：automation assertion 需核對引擎 API 型別，不假設 TObjectPtr 的隱式轉型能參與模板推導。

## E026：PIE 文字序列化預設值與背景渲染驗收時序

- 現象：新的 PIE memory smoke 首次失敗於 missing memory diagnostics。GetBridgeStats 的 ReturnValue 已正確記錄 1→0，但 Unreal ExportText 省略與預設值相等的零欄位；ghost 的首次 WasRecentlyRendered 也在 screenshot 實際擷取前回傳 False。
- 決定：先核對 success 與 FOmWorldBridgeStats 回傳型別，只有通過時才依該結構預設值解讀省略的 count=0；不把任意缺失回應當成功。marker screenshot 在 WasRecentlyRendered 之前擷取，以背景 Editor 的實際渲染結果驗收；不取消渲染斷言。
- 驗證：修正後 PIE smoke exit 0，native_mesh_rendered=true、remembered_ghost_rendered=true、remembered_ghost_counts=[1,0]，自行啟動的 PIE 已停止。

## E027：停止 runtime 不可只丟棄仍顯示的 actor 參照

- 現象：程式檢查發現 StopRuntime 使用 ReleaseAllActors(false)，既不隱藏也不 Destroy live actor，隨即清空 EntityActors／ActorPools，可能留下無人管理的顯示物。
- 決定：明確停止 runtime 時使用 ReleaseAllActors(true)，銷毀 bridge 所管理的 live／pooled actors 並清空 memory markers。Editor-only memory fixture 不啟動 runtime，也覆寫 EndPlay，避免 fixture 清理停止另一個 bridge 的對局。
- 驗證：完整 build、Editor repeat 與 PIE cleanup 為驗收，不以單純指標清空宣稱 actor 消失。

本輪操作錯誤補記：E018 再次猜測 WorldBridgeActor 在 Source/omGame，實際位於 Plugins/OmRuntime/Source/OmGenerated，已用 rg --files 解析正確路徑。另一次 apply_patch 尾段使用不存在的「#」上下文使整包驗證失敗；確認沒有部分寫入後改用真實全文段落重套，禁止用假錨點追加 MD。

E018 同輪也出現猜測 Public/om_bridge.h 與 bridge/README.md 路徑的唯讀搜尋失敗。實際 header 在 Source/ThirdParty/OmBridge/include；不能只對程式來源遵守「先列路徑」，文件與生成產物也必須遵守。

E024–E027 最終驗證：Rust 33 unit + 2 TD integration passed（1 外部 KCP ignored）；full build session 84528 exit 0、Editor PID 91580；session 28134 同一 Editor 兩輪各 7/7，自動化零 failed／skipped／not_run；PIE 17 steps PASS、native 與 memory 皆 rendered、memory counts=[1,0]，自行啟動的 PIE 已停止。詳細決策及剩餘範圍見 `2026-10-03-unreal-remembered-ghost-progress.md`。

## 已確認的錯誤

| 編號 | 現象與原因 | 修正與預防 | 驗證 |
| --- | --- | --- | --- |
| E001 | TCP 分段讀取被 `select!` outgoing 分支取消，已消耗的封包前綴遺失 | 接收與送出各自持續執行，只在整條連線結束時取消 | 分段 socket／重連測試通過 |
| E002 | renderer 排入輸入就回報 applied，且 secure request ID 與 renderer request ID 混用 | 等 runtime 終局結果，保留原 request ID；重要結果保留至 frame 被取得 | runtime／bridge 測試通過 |
| E003 | 舊 source patch 要求不存在的插件 ImportTools.cpp | 識別完整預編譯 2.0.6 套件；未知版本停止；另驗證真實動畫 | 完整建置與動畫匯入通過 |
| E004 | `import_animation` 把 SkeletalMesh 當主要 AnimSequence 回傳 | 找到額外 `_Anim`，以真實型別、時長、frame、Skeleton 依賴驗證，保存所有額外產物 | 5 支動畫、重跑套件 hash 通過 |
| E005 | 材質工具無法從 `_mat` 檔名猜貼圖角色 | 明確指定 BaseColor，不靠檔名推測 | 材質 valid、零 issue、Texture2D 依賴通過 |
| E006 | 重試材質建立回報 already exists；材質摘要也不是 class 欄位 | owned pending 材質讀回與驗證後保存，不刪除重建；用 validate_material 驗證 | 配方重跑零 import／零 binding 變更 |
| E007 | 另一個專案鎖住共用 Engine DLL，LNK1104，啟動 0xc0000135 | 不停止別的專案；獨立驗證 Rust，待 DLL 可連結後重建本專案 | 其他程序自行結束後完整建置通過 |
| E008 | `openspec status` 的 isComplete 只代表計畫 artifacts 完成 | 用 instructions apply 的 progress 與 tasks 勾選判斷實作完成度 | 目前實作 15/30，不宣稱框架完成 |

## 本輪計畫與決定

- 新增通用 native skeletal 呈現，生成器從現有 Lua render 資料產生 soft asset 引用與動畫區間。
- 不在 native CDO constructor 載入資產；於 BeginPlay 初始化，以免 Editor 載入類別時同步匯入或載入美術。
- 若既有 Blueprint 已提供 SkeletalMeshComponent，保留它的呈現路徑，避免同時出現兩套 mesh。
- 缺少模型保留方塊；動畫僅改變呈現，不計算傷害或驅動權威 tick。
- 本輪新錯誤追加於下方，並以實際建置／Editor 結果更新驗證狀態。

## E009：新增呈現驗證使兩個既有 codegen fixture 失敗

- 現象：首輪 codegen 19/21 通過；未填 scale 的 art fixture 被拒絕，Saika 摘要 fixture 的 attack binding 引用未宣告來源。
- 原因：共用 schema 對缺少 scale 使用 0；舊摘要測試只測資料輸出，沒有完整可播放來源。
- 決定：native 邊界將未指定的 0 scale 視為 UE identity scale，負值／非有限值仍拒絕；補齊摘要 fixture 的 animation source，保留正式生成器的引用驗證。
- 驗證：修正 fixture 與 default 邊界後 codegen 22/22 通過，包含新 native 路徑／clip／非法引用測試。

## E010：apply_patch 區塊順序錯誤

- 現象：同一檔案先修改較後方 fixture，再尋找較前方測試，兩次 patch verification failed，沒有套用任何變更。
- 原因：此工具依序搜尋 patch 區塊，不能向前回找。
- 決定：同檔案區塊按來源行序排列；失敗後先檢查實際檔案，不重複送同一 patch。
- 驗證：重排區塊後成功套用，22 項 codegen 測試通過。

## E011：UE Automation 的數值比較多載衝突

- 現象：完整建置的 OmEditorAutomationTests.cpp 出現 C2665，`FVector.X` 是 double，預期值用了 float；native runtime／生成 C++ 已編譯，整體建置仍失敗。
- 原因：UE5 的 Large World Coordinates 將 FVector 分量改為 double，Automation 的 float／double overload 無法唯一選擇。
- 決定：FVector 比較使用 double 預期值，動畫 player 的 float 時間比較仍使用 float；不修改引擎或關閉測試。
- 驗證：修正後完整 Unreal 建置通過，Editor PID 60860，codegen 22/22 通過。

## E012：MCP 連線成功不代表 Editor 可操作

- 現象：Editor 重啟後 MCP health 通過，但 enable_extension 被 Restore Packages modal 阻擋。
- 原因：本專案 Editor 子程序不能優雅關閉，restart 在逾時後 force stop，舊 autosave 在啟動時觸發還原提示。
- 決定：先用 get_editor_dialog 讀取完整清單。本次只包含本輪 RecipeV1 的兩個 imported materials 與已保存、通過 SHA-256 驗收的生成材質，選 Skip Restore，保留 autosave，不覆寫保存資產。若出現其他來源，不套用此決定。網路 health 與實際 Editor 驗收分開記錄。
- 預防：重啟前保存 owned 新資產；重啟後第一步檢查 modal，再執行編輯命令。
- 驗證：本輪完整建置、Editor 測試與 22 套件配方重跑通過；未覆寫 autosave。

## E013：Editor package path 不能直接當 native soft object path

- 現象：Editor automation 4 項中既有 3 項通過，native 呈現失敗；模型存在但 LoadSynchronous 回報 missing，連帶動畫／visibility 斷言失敗。
- 原因：MCP 支援 `/Game/Folder/Asset` 簡寫，FSoftObjectPath 必須使用 `/Game/Folder/Asset.Asset` 才能解析 UObject。
- 決定：生成器對所有 native mesh／材質／動畫輸出完整 object path，MCP 配方仍保留 package path；增加完整 suffix 的 Rust 斷言，避免兩種引用混用。
- 驗證：完整建置通過；native 與既有 Blueprint／Saika 的 4 項 Editor automation 全部通過、零 error／warning；PIE 生成 transform 斷言通過。

## E014：材質 graph valid 不代表支援 SkeletalMesh

- 現象：Editor automation 通過，但檢查 PIE 日誌發現 `missing usage flag SkeletalMesh`，遊戲使用 Default Material。
- 原因：create_material_from_textures 未啟用 bUsedWithSkeletalMesh；原先只檢查 graph 與引用，漏掉 runtime usage。
- 決定：將 skeletal usage 納入配方操作 signature，owned 材質更新旗標後保存；每次驗證都讀回旗標，再驗 shader／引用。不能只以 asset 存在或 graph valid 宣稱畫面正確。
- 驗證：更新 1 個 owned 材質、略過 9 項；Editor 配方復驗 10 jobs、22 套件 hash 不變；PIE 無 missing SkeletalMesh usage 警告。

## E015：沿用 Fyrox 軸向修正，未驗證 FBX importer 已轉軸

- 現象：native 屬性／播放測試通過，PIE 截圖仍看不到正常站立的角色；最初測試位置也離相機中心太遠，不能用該圖作畫面完成證據。
- 證據：透過 GetImportedBounds 讀回模型 Z extent 約 65.85 cm，來源已由 UE importer 轉為 Z-up；原 render.pitch=-90 是 Fyrox 的原始來源修正，不應再次套用。
- 決定：native pitch／roll 預設 0，保留 scale／yaw／offset 的資料來源；新增 `ue.native_visual` 的 scale／pitch_deg／yaw_deg／roll_deg／z_offset_cm 供 Lua 美術配置明確覆寫，不改後端或既有 Fyrox 軸向。PIE 截圖在相機附近生成角色，等待實際 render frame 後再取圖。
- 驗證：軸向／rest offset 修正後完整建置與 4 項 Editor 測試通過；PIE 真實 render 與截圖通過。模型先前不可見也受 E017 的相機視點影響，不能把軸向當作唯一根因。

## E016：重建執行中讀取了舊啟動／共用 UBT 日誌

- 現象：讀到舊的 start JSON 誤以為新 Editor 已啟動，MCP 連線兩次失敗；共用 UBT Log.txt 同時包含另一個專案。
- 原因：完整 Lua 建置仍在停止／staging 階段，start 檔案尚未更新；多專案共用引擎的 UBT 日誌不能當成本專案的唯一證據。
- 決定：以當次建置 process session exit code 與輸出為準，完成後才呼叫 MCP；舊 start 檔與共用 UBT log 只作輔助，不用來判定當次成功。
- 驗證：以 session 7541 exit 0 確認完整建置完成，再執行 4 項 Editor 與 PIE 驗收。

## 呈現插值補強

Code review 發現原插值固定回到 relative zero，會抹掉 `ue.native_visual.z_offset_cm`。已改為記住首個權威 frame 前的美術 rest offset，插值與重置回到該 offset；缺省 offset 0 的舊 fallback 保持原行為。此項屬預防性修正，並非已觀察的使用者資料錯誤。

## E017：Actor 在原點不代表位於玩家視野

- 現象：角色 scale／引用通過，但截圖沒有模型，WasRecentlyRendered=False。
- 原因：舊場景初始鏡頭距離 90000 cm；游標邊緣移動後實際 GetPlayerViewPoint 在約 (-25985,-25985)。相機 pawn 舊快照不是即時玩家視點。
- 決定：只對工具自行啟動的 PIE 設定 UI pan guard 與 700 cm zoom，等待後讀取真實視點／旋轉，將視線與測試高度平面相交定位角色。使用 WasRecentlyRendered 作為獨立必要斷言；已有使用者 PIE 不改鏡頭，也不宣稱畫面驗收。
- 驗證：新 fresh PIE 全步驟 PASS、native_mesh_rendered=true，截圖可見帶材質模型；自行啟動的 PIE 已停止。SetFocusLocation 的兩種參數形式未達到定位目的，不依賴其回傳 success 當效果證據。

## E018：診斷入口與 Windows 搜尋路徑不可猜測

- 現象：誤呼叫不存在的 get_tool_info；rg 對 client*／ipc* 路徑收到 Windows error 123；讀取不存在的 presentation_ipc.rs／om_restart.lua。
- 決定：MCP 用已存在的 get_tool_docs(action=...)；先 rg --files 取得實際檔案，再以 rg pattern 目錄 或 -g glob 搜尋；restart 入口是 Rust executable，不虛構 Lua wrapper。
- 驗證：正確工具文件成功取得；實際 IPC adapter 位於 bridge/src/driver.rs，WorldBridgeActor 位於 OmGenerated 而非 OmRuntime。後续搜尋一律先取得實際路徑。

## E019：BlueprintSurface 測試第二次執行撞名導致 Editor 崩潰

- 現象：首輪 4/4 通過，重跑 BlueprintSurface 在 Kismet2.cpp:441 assertion，Editor shutdown，MCP 測試報告正確記為失敗。
- 原因：CreateBlueprint 固定使用 transient 名稱；同一 Editor session 上次暫存 Blueprint 尚存在，不能建立同名 UObject。
- 決定：英雄與 buff 測試均用 MakeUniqueObjectName，不刪除已有 UObject／使用者資產；不得僅重啟掩蓋不可重跑問題。
- 驗證：完整建置 session 13474 exit 0；同一 Editor PID 57700 連跑兩次均 4/4 通過。驗收工具固定兩輪，避免只以 fresh session 掩蓋殘留狀態。

## 建置診斷基線（非阻斷）

當次 full build 成功，但預編譯 BpGeneratorUltimate 的 extension modules 有 plugin dependency 宣告警告；Rust td_rounds 有既有 dead-code 警告，cbindgen 略過非 pub 常數。這些不是本轮 native 呈現失敗，也不能宣稱「整體建置零警告」。不修改共享引擎或憑猜測調整預編譯插件相依。

## E020：Hide 後 Forget 仍要求即時 entity 存在

- 現象：程式檢查發現 Hide 先移除 world entity，後續 Forget 呼叫同一 remove_with_epoch，會回傳 UnknownEntity。
- 原因：忘記安全記憶與移除即時 entity 被混為同一操作。
- 決定：Forget 若 live entity 存在仍嚴格檢查 epoch；否則只接受相同 (replica_id, disclosure_epoch) 的記憶。舊 epoch 不得刪除新 entity／新記憶；未知 ID 仍拒絕。
- 驗證：core 294/294，包含 Hide→兩次 extraction→Forget、同 frame Hide→Forget、新世代防舊 Forget、Reveal／verified rebase 清除記憶。
- 首次測試仍失敗：apply 路徑修正後，preflight_entity_references 仍只認 live entity，在套用前先拒絕。已同步修正 preflight 的暫存記憶集合，包含同 frame Hide→Forget；預檢与執行必須使用一致的身分規則，不能只修正最末端操作。

## E021：重連 snapshot 遺漏已保存的安全記憶

- 現象：runtime 保存 remembered_presentations，但 snapshot_envelope 固定輸出 remembered_ghosts=[]；最新狀態不足以恢復記憶。
- 決定：FilteredRenderSnapshot 攜帶凍結的 server-sanitized 記憶，以獨立 ghost 欄位投影至 IPC；不放入 live entities，不從隱藏 world 推算新位置。空 payload 不建立 ghost；生命周期 edges 仍走既有獨立通道。
- 驗證：client runtime 30+2 通過，真實 TCP 斷線後兩次取得同一最新凍結 snapshot；bridge 28+2 通過（1 外部 KCP ignored）。UE ghost 呈現尚未驗收。

## E022：共用 RendererReady 更新後漏了舊 Fyrox 呼叫端

- 現象：相容性 cargo check 發現 omfx 的 RendererReady 缺 player_id／team_id，且仍使用 protocol 2。
- 原因：先前 IPC v3 只修正 Unreal renderer；共用 proto 的所有消費端需要一起驗證。
- 決定：保留 Fyrox 相容路徑，將既有明確設定的 local_player_id／local_team_id 傳入握手，版本升 3；不猜測身分，不取消 runtime 身分檢查。新增共用 snapshot 欄位也同步更新其 adapter 與 fixtures。
- 驗證：omfx cargo check --tests 通過，沒有取消 IPC 身分檢查。

E018 本輪再次出現一次猜測來源名稱（team_replica_specs.rs）造成搜尋 error 2；後續以目錄搜尋確認實際為 filtered_specs.rs。這項操作習慣仍需持續遵守「先列實際路徑」，不可把曾取得正確檔案等同永不重犯。

## E023：新 frame sequence 不代表新的投射物事件

- 現象：程式檢查發現 DispatchFxCues 對每個 frame 的 retained tower_fire_fx 都呼叫 OnProjectileCue；最外層只檢查 frame sequence，無法阻擋同一 cue 重播。
- 原因：保留事件供較低頻 renderer 取得，卻把 frame 的新 sequence 當事件身分。
- 決定：在真正的 UE dispatch 路徑使用 (kind, source ID, source generation, instance ID, cue ID, event tick) 去重，不使用 renderer frame sequence。找不到 source actor 不消費；只有 projectile branch 消費其事件，未知 FX kind 保持原行為。
- 預防：加入新 frame／相同 cue、同 tick／不同 instance、新世代、零 instance、容量上限、過期重播及未來事件測試；synthetic frame 驗證實際 dispatch 次數。
- 邊界：4096 tick 歷史、最多 16384 keys；超量時 fail closed 並提供診斷，不能逐出當前 key 後重播。只在明確 StopRuntime（結束該 runtime）重設，Hide／actor 回收不重設。新的 WorldBridge instance 重連不重播的跨 instance 契約仍需 runtime cue 消費機制；本輪不宣稱已完成。
- 驗證：完整 Unreal build session 22346 exit 0、Editor PID 72232；同一 Editor 兩輪皆 6/6 passed（包含真實 synthetic dispatch），零 failed／skipped／not_run；PIE 模型與 fallback smoke 通過並停止自行啟動的 PIE。
# E266 — 護盾餘量需要具名安全狀態，不可由 metadata／私人 payload 猜測

- 決定：OwnerEconomyState 追加有界 Q10 scalar、94-byte strict decode；權威存活英雄取真實餘量，死亡／缺英雄零值。既有 team audience 與 configured owner 過濾保持，不發 Buff key／source／payload。
- 契約：OwnerEconomyPresentation schema 2、selective wire 6、IPC 5、C ABI 15。最後統一重建部署；禁止混用舊元件或繞過版本檢查。
- 當前結果：正式 generated 商品 60Hz 100→60→0／到期／死亡與另一 owner 零值測試、bridge 邊界／owner／舊 schema 拒絕通過；限定 UE 模組 14 actions 編譯成功，不代表 PIE。
- 操作錯誤：Windows 字面路徑 transport* 觸發 rg os error 123，改已知目錄加檔案 glob；大段工具輸出截斷後不宣稱完整審閱。等待未知既有 Build.bat 自然結束，不終止外部程序。
- 詳見 `2026-10-06-owner-shield-projection-progress.md`；完整驗收留最後。
## E331：專用主機不可沿用「空選擇代表全部本機玩家」與 Unreal 建置前置

- 問題：既有遠端加入入口已存在，但本機 host 的空 local-player 集合代表全部 human，且工作流固定先解析／建置 Unreal。純 Rust host 無法部署到沒有 Unreal 的機器。
- 決定：新增明確 server-only 角色；沿同一正式 recipe／Rust 選角鎖定／compiled-content-only／60Hz 設定，零本機 client 不代表刪除 human roster。不另建 gameplay，也不擴大為防火牆／憑證設定。
- 建置：獨立 Lua 模組依選定 debug／release 同步建置 base_content、moba-config、omobab，再沿既有 stage-dll 工具部署；SHA-256 比對該 profile 的來源 DLL 與正式 scripts/base_content.dll，無 Unreal／bridge／client runtime 前置。
- 本批操作錯誤：猜測不存在的 build_backend.lua／workflow 測試／match-lifecycle spec 路徑；應先 rg --files 或使用 OpenSpec 回傳實際路徑。DLL mismatch fixture 只認斜線而 Windows path.join 回傳反斜線，致 expected rejection 失敗；正規化 fixture 路徑後四組局部測試通過，不變更正式 hash 判斷。
- Grok run-muwh76hh-6y910g／thread 5fef2d53-a4e0-4d7a-950e-f689b4ccf650 在讀取階段2分18秒無程式變更，主agent取消；stop command報taskkill128不當退出證據，follower terminal cancelled、三個tracked PID為null且原agent52244／bridge21276皆不存在後才接手。原因、成本及API時間未知，不改憑證或全域MCP設定。
- 正式整合完成：server-only零本機client／完整roster、互斥旗標、後端獨立build／stage、只等待original authority與既有身分清理；原remote PID-only fixture改用既有owned adapter，已退出renderer不要求stop、PID reuse不碰replacement。局部server build4／dedicated7／network7／role8全部通過，共26組；Rust設定預檢是真實，程序生命週期是注入fixture，未跑對局／Unreal／兩台LAN，完整6項仍保留。

## E352 — 舊 native BodyMesh 序列化與驗證產物一致性（2026-10-07）

舊 export 改名／改 hash 會破壞跨 package 的 BodyMesh import；合法隔離雙 package fixture 已真實重現，不能只凭單 map reload 通過判定遷移成功。採保留序列化名稱／hash、綁新 native component 的方案，authored SCS 不介入；新方案待正式編譯與回歸。

Primary 第五輪建置 native-serialization-machine-v5 失敗 C2248：誤呼叫 private FLinkerLoad::IndexToObject。公開 ImportMap 的 FObjectImport.XObject 已核對可用，交 Grok 4.7 Build Fast 修正。該建置失敗後又啟動 native-serialization-cases-v3，實際使用舊 DLL；程序 exit0 不算新來源驗證。之後必須先確認正式 build exit0／NoEngineChanges／stage SHA，再跑新目錄 feature cases，並核對 report 名稱／狀態／錯誤；不把 Cmd exit0 當 PASS。

讀取 state.json 尾端的巨大單行造成截斷；只選取需要的欄位，禁止印整行 machine_continuation。舊原始失敗證據保留，不覆寫。

E352 更新：新版兩feature cases-v6真正2/2 Success，zero errors/warnings/notRun。根因是async load不保證thread ObjBeginLoadCount>0，HasStartedLoading不能用作是否載入的硬guard；必須依Actor NeedLoad、package linker及exactnativeexport判定，sync loaded-object admission與async export table路徑分開。錯誤的LinkerLoad.h107成員歸属假設已更正，禁止把rg命中當API所在class證據。完整本機驗收仍待結果。

## E353 — 新機接續 remaining work（2026-10-07）

正式 Lua stage 預檢正確參數是 --verify-staged-only。Primary曾猜--verify-only，未知參數exit1；讀實際help/parser後更正，兩個stageSHA通過。也曾猜不存在的moba_native_result_capture.lua；實際模組為moba_role_finish_observer.lua，後續只依rg --files與已讀require發現。多個歷史長段讀取再次截斷；改讀exact來源與短JSON欄位，不印完整history／單行大JSON。

prepare-only配方1真人9Bot/60Hz成功，不開局。same-PIE-continuation-v1使用正式既有launcher/opt-in selection smoke/自然終局1200s，新Editor cold DDC中；authority尚未啟動，batch仍8/10。非嘗試重啟取得grace，不繞過license。只有server真spawn後加到9；不把native result-only當完整UI／LAN／12效能PASS。

Grok readonly run-muxpexmo-tppal8 / thread5a61bd00-d3ec-4893-ad7f-54f65fbd7ab2 completed230s USD1.1732196。讀public terminal rawOutput因bridge summary是初始句；實際source指出WaveB/非managedparallel script進globalRayon、不受ECS explicitpool budget；rebootstrap.empty多建暫存pool。兩者不是historical spike已定因。writer run-muxpldmz-9jh1zt交boundedownership修復，不建置／改staged artifacts，現有integration使用既有一致產物。後續primary獨立diff/tests/build。

Pool writer completed816s USD4.53590736；第一真feature build E0597 fail101/0tests，50s Fast test-lifetime repair USD0.10886664後actual四featuretests exit0（4passed504filtered）。原worker建議`cargo ... --exact`位置錯，cargo test選項分隔`-- --exact`才合法；primary用實際4項filter驗證。錯誤記入worker-pool-ownership-progress。既有71/69驗收仍是先前產物；新Rust變更還未正式部署，ownedEditor使用原一致產物。使用者要求先看整場simulation，priority持續samePIE原啟動，不啟第二場。

same-PIE-continuation-v1正式啟動等待1800s逾時，原Report success=false／simulations_executed=0／cleanup_verified=true，原Editor45076已不存在。不是license／game失敗，不計新場次。D3D11 global shader首次編譯仍持續且Editorresponding；保留完成DDC、原log與errors.md。依使用者「先跑整場讓我看」改用既有explicit --editor-ready-timeout-seconds 3600 的v2全新證據目錄；不跳shaders／fake ready／取grace，等待同一source-engine cache補齊。Rust source四回歸已PASS但production stage仍待本場退出後，避免覆寫loaded DLL。

展示流程決策：DX11/SM5 reflection global shaders單支實測約40..128秒；v1首次1800s逾時、v2仍cold shader工作。檢查DefaultEngine.ini194..200：專案本來預設DX12並targetSM6。依使用者先看整場，主動以原creation-token owned record stop_owned／wait_owned退役v2 Editor；driver退出1並cleanup_verified=true／simulations0，不是新對局或license workaround。改為v3既有--graphics d3d12、正常shader編譯、同樣自然結算。未改project config／Engine／fixed12門檻；DX12展示不等同原DX11固定效能驗收。禁止按name／PID殺他人、重寫冷編譯進度或假Ready。

E353 continuation — 2026-10-07 user-watch v3 DX12: native MCP readiness/tools/list/ping/idle status passed, begin_play_in_editor timed out at hardcoded curl20s (zero bytes), simulations_executed0, cleanup_verifiedtrue. Added bounded --request-timeout-seconds (finite integer1..3600, default20), cold PIE begin uses180; no blind retry, ownership/auth gates unchanged. Primary22budget cases +19preflight scenarios exit0. Worker run-muxr0znr-3n4azo stalled~8min before source edits and was stopped via bridge; primary implemented and delegated read-only review run-muxrbt7r-xqhmdd. Cost of cancelled job unknown; do not claim completed. Primary guessed nonexistent Saved/Logs/om.log; inspect actual launch-plan abslog and rg --files first. v4 starts fresh owned Editor, prior artifacts remain consistently staged, production worker-pool rebuild pending.
E353 continuation: v4 started real authority (batch9/10), selection_finalized and same_world_handoff true. take_pie_screenshot explicitly returns asynchronous note; immediate file assertion falsely failed before PNG landed0.2s later. All owned cleanup verified. Fix bounded owned file wait, exact path and positive size; never use immediate request acceptance as durable evidence. Primary also repeated nonexistent guessed MCPCommandRouter.cpp/glob literal reads; use rg --files --no-ignore in ignored plugin subtree before reads.
E353 continuation: v5 began async PIE request (transition_observed=false) but fixed45s UI/service readiness deadline expired while cold world loading actually took~59s. simulations0/cleanuptrue, batch remains9/10. Readiness must use existing configured selection remaining budget, capped180s, without resetting token/cancellation or assuming accepted means running. AppTime off-thread ensure stack observed in vendor MCP context-check path; preserve diagnostic, no protected service/context/auth edits. Screenshot production wait still awaits successful full live run.
E353 continuation: v6 real authority starts, batch10/10. Selection+sameworld+boundedasyncPNG passed, but initial and midmatch high-res screenshots show sky/grey patch with HUD excluded; visible match unaccepted. Camera read-only evidence -90pitch, viewtargetRtsPawn, runtimeentities31 andHUD firstframe real. execute_pie_console_command Shot SHOWUI reports success but no requested file within30s (GEngine Exec return ignored), not screenshot success. Primary first queried properties instead of actual property_names; corrected source-verified query and preserved both results. Large CameraCache/ViewTarget exports must redirect rawoutput to ignored file and print only parsed fields. No protected FEditorSync context edits, gameplay mutations, winner forcing, or extra authority.
E353 continuation final v6: actual1human9bot60Hz match ran1200s, last observedtick57182, selection_finalized=true/same_world_handoff=true; natural result timed out, no result marker or screenshot, report.success=false/cleanup_verifiedtrue. Full match NOT accepted. Existing batch exhausted10/10; no extra authority, reset, forced winner or invented result. Primary chose1200s finish bound and cold async phase exposed preexisting latency; a bounded run is not whole-match completion. Correct screenshot capture source work and production pool consumer rebuild remain pending after owned shutdown.
E353 continuation: formalbuild UBA stuck~15min on PlayTestTools.cpp,compilerCPU3.4s,no actioncomplete. After exact current command/exe/creation match to own Lua20436, public inspect_owned+stop_owned retired own original build lifetime and descendants; exec1(cancelled), not successful compilation. First comparison used slashliteral instead of path.absolute and asserted before kill; corrected normalized pathcomparison. Stage SHAalready6cb1145683ccc10d4a21e8dc73e416c164bbbfa9ac041ebb03ce5aa9d0b8bd0d /1fc65608a4fb46d3aa9db5f531289256f55e35ac3a606abe3e9b3d6c951b249c. New optionalofficial local -NoUBA executor andbuild-only owned supervisor being implemented; no Engine/config/cache deletion or protected context changes.

### E354 — BpGeneratorUltimate 啟動公告造成 GUI 阻塞（2026-10-07）

- 使用者回報 Unreal 啟動時新聞視窗卡住 GUI；定位 UECPShellModule::StartupModule 五秒 ticker → OnShowFirstTimeStartupDialog → welcome／announcement fetch → topmost SWebBrowser。
- 修正方向：移除自動啟動入口與専屬 ticker lifecycle，不偽造 dismissed version／announcement ID，不改 auth／telemetry／editor-sync gates。Grok 4.7 Fast run-muxtj7rx-dzlho3 實作中；尚未驗收。
- 原 executor Grok run-muxtd4mj-d7wwd3 因使用者優先事項取消；未產生該功能修改。Primary接手 bounded --build-executor default|local，官方-NoUBA，保留-NoEngineChanges；help通過、非法值與verify/local組合均在stage前拒絕。
- 獨立source複核：啟動ticker註冊、關閉移除、callback、header member均已刪除，正常面板/menu/service initialization保留；手動歡迎函式無startup caller。正式建置中。
- 防重犯：UE5.8 ExecutorFactory 永遠保留UBA排程名稱；-NoUBA只將bAllowDetour=false，不能只看「Using UBA」就認定失效或再次取消。實際SUECPMainWidget.cpp.obj在16:06:10更新，下一個Architect translation unit繼續，證明有進展。
- 正式驗證完成：build_ue_moba.lua --build-only --build-executor local --ue-root D:/UE5.8 實際exit0，C++ Result:Succeeded／315.40秒，bridge/content stage SHA一致。UECPShell.dll於16:08:45更新，新Editor實際載入該DLL。
- 新owned Editor fixture實際ping／PIE status／begin PIE成功，選角UI ready、source=pie_widget的1280x723 PNG落地並獨立檢視可見選角UI；stop PIE observed=true、cleanup_verified=true、report success=true、simulations_executed=0。證據target/role-ue-runs/machine-startup-news-capture-v1/report.json與selection.png。啟動ticker／callback全源碼引用掃描不存在；不以「沒有新聞log」單獨作為證明。
- 此為啟動新聞及當前PIE截圖功能驗證，不能外推到完整same-PIE自然結算／真雙機LAN／全十二效能已通過。

### E355 — 選角 renderer 的冷啟動預算沒有接齊（2026-10-07）

- Primary核對剩餘OpenSpec4.4／6.2／6.4／6.5，發現single interactive／shared renderer／remote join仍各自hardcode45000ms；既有editor-ready-timeout可設定1..3600秒但這三路未共用。實機冷啟動已>45秒，正常producer可能被過早清理，與已修PIE入口屬同類缺陷。
- 委派Grok4.7Fast run-muxu7gbg-aruu0j／threadd9d7afb1-dec4-4a5e-9a16-587524998421，統一renderer deadline helper、forward remote選項、三路晚ready與較早total/cancel回歸；不得改native host／runtime45秒、marker／receipt gate、auto-lock或ownership。尚在實作，不提前宣稱PASS。
- 前一readonly remaining audit run-muxu3emt-682baz因已定位可實作缺口收斂而取消；沒有source edit／完整audit結論，成本未知。Grok未提供的結論不歸屬給Grok。
- 獨立共用Rust回歸：cargo test --manifest-path omoba-core/Cargo.toml --lib -j4 filtered_specs -- --nocapture實際exit0，19passed／0failed／489filtered，含rebootstrap pool identity、membership與安全projection。不是新authority／LAN／十二效能驗收。
- 工具教訓：OpenSpec design/tasks混有長篇歷史，整檔合併輸出造成截斷；應先rg headings再讀當前需求區。查檔名先rg --files，不再猜不存在的join_moba_selection_ue.lua。
- 初次negative回歸未到cold-ready情境：三個既有測試要求debug moba-config.exe，但本機只已完成release工具；前置缺檔拒絕屬有效失敗，不是45秒缺陷的negative證據。正補正式debug compiled-only moba-config建置，不改測試為fallback、不偽造binary。
- 較大Grok writer8分鐘無source edits取消，改明確非重疊分工：Grok helper-only run-muxuid9b-3cfo15／thread77d1d838-1fc0-4334-b073-d5039043bd7c完成40秒、4calls、USD0.08853872；Primaryowns三路接線與測試。helper完成待獨立審查和驗證。
- Helper獨立純邏輯回歸14/14PASS：default1800s、configured90s、原overall cap、expiry、integer/range／NaN／infinity拒絕、cancel token優先。三consumer尚未接線，待debug配置工具建置後保存old45s實際negative，再接線正驗。
- Primary新增實機selection-only驗證入口，已依既有path API改用is_directory檢查後進一步原子預留fixture根目錄，避免失敗時覆寫別的output；尚未實跑。一次apply_patch猜錯print尾行被拒絕，未修改檔案，已依實際行修正；讀取結果應先檢視再形成patch。
- Negative v2曾因fixture改寫自己空log未給path.write overwrite=true而先失敗；已限定自己fixture log改寫，不改production policy。Negative v3真正單人／shared／join各exit1，exact reason為三路handshake timeout；後接線五個受影響test入口皆exit0（single14、shared26、join20、readiness14，加deadline/cancel guards）。
- 新實機validation首次在spawn前被正式options gate拒絕：gameplay selection automation必須finish observer；fixture並不執行gameplay。沿既有test_unreal_hero_selection方式，在fixture獨立驗證hero名稱後只傳一般interactive flags、再交bounded selection adapter；不放寬正式launcher gate、不假造finish observer，沒有前次output／程序殘留。v1b實跑中。

### E356 — 真選角完成後 Windows identity query／retirement 競態

- startup budget v1b真實renderer約3分鐘後OM_SELECTION_READY，select／lock／finalize exact1..3成功、world cleanup；report242016ms仍success=false，原因cannot query exact process identity access denied，cleanup同錯誤。不能把選角成功當完整驗證成功。
- 原PID31232現在CIM已無該程序，外層private Job supervisor已退役，0真對局。這不是要以PID或name殺process的授權。現有source／fresh native helper已有failed-query→samehandle0ms recheck；這次仍重現，不能重複宣稱舊修正已涵蓋。
- Generic fix委派Grok4.7Fast：僅query failure時在原retained handle上有界等待signal；signal才承認退休，timeout／waitfailure保持原identity錯誤，正常成功query不延遲。不得access-denied=false、reopen/adopt、放寬creationtoken／exe、或跨PID處理。待審查/tests和真selection-only重跑。

E356最終修正驗收：Grok run-muxuxdyh-15vd2j（352s／12calls／USD0.44945552），Primary native9／ownedwait10／真own-child身分與清理PASS；實機selection-startup-budget-v2 exit0、150453ms、success=true／cleanup_verified=true，0對局。實跑沒有宣稱必然走到failed-query wait分支；該分支由真child signal／timeout／invalid-handle回歸驗證。原失敗report保留。

### E357 — Primary協作state序列化指令錯誤

PowerShell將-Depth30誤當不存在參數，nonterminating error後仍執行WriteAllText，把machine_continuation摘要值寫空，造成JSON失效；game/source/evidence/review未受影響。已立即以apply_patch恢復合法object，保留root其他欄位，依verified MD重建current machine摘要，batch10/10／27/31／v6failed／新聞accepted與startup actualfailed明確保持。之前nested摘要細節不可恢復，歷史以完整review/MD為準，未假裝無損恢復或回退source。

防重犯：狀態更新使用解析後受檢JSON、寫入前驗證，PowerShell必須ErrorActionPreference=Stop且-Depth 30正確空格；不要以regex替換掉未成功序列化的object。此為primary工具錯誤，不能歸屬Grok。

### E358 — Windows rg wildcard路徑與讀取路徑防錯

Primary本次再次將scripts/moba*.lua等glob當Windows literal path傳给rg，報os error123，沒有搜尋結果或檔案修改；前次亦猜scripts/lib而實際為tools/lua/lib。已改成先rg --files盤點，再傳目录並用-g '*.lua'篩選。這些工具錯誤不歸屬Grok，也不作測試FAIL或功能缺陷證據。後續查詢必須使用實際盤點所得路徑。

### E359 — 正式本機驗收的vendor遠端sound設定逾時warning

machine-current-final-v2／blueprint-engine.log於09:02:43.438UTC记录sound_gen_config HTTP讀取15秒逾時。CompileAllBlueprints仍69／69、compiler errors0／warnings0／failed loads0、exit0；native71Success／13test warnings另外計數。不得把compiler零warning當engine全部log零warning，也不得以外部服務warning否定實際已完成的native／資產驗收。保留原log，不重啟拿grace、不偽造cache、不修改授權或外部服務、不盲目重複執行驗收。若後續sound generation實際功能依賴該設定而失敗，再以exact功能契約診斷；本次沒有聲音生成驗收證據。

### E360 — 自然結算診斷的證據形狀與範圍

Primary廣泛evidence inventory造成千筆輸出截斷，並將components array誤當object選phase欄位得到null；未當成結論。改exact檔／root shape／first record，再依正式TeamTickFrame及projector payload解碼公開capture，實際57422records，末1315筆phase1／winner0，沒有phase2。不能從JSON不存在欄位猜phase、不能在任意capture尾offset解析record。原v6是仍Playing時逾時，未找到可證明的UI掉終局缺陷，保留原失敗report，不製造source patch。Windows wildcard／猜檔名重複問題仍依E358防錯。

### E361 — 使用者操作回報揭露camera/minimap實際接線缺口

使用者回報小地圖點擊不改視野、wheel不能縮放，以及A點地板等完整對局操作缺漏。Source確認SOmMinimap只在right click送MoveRequested，left被Handled而無camera path；camera namedMouseWheelAxis缺AxisMappings且預設EnhancedInput。首hero fit約90000cm但zoomstep450cm，即使接線也很不明顯；follow亦會覆蓋單次focus。Native測試直接呼叫AddZoomInput且舊minimap測試只確認left不移hero，71全PASS不代表這些玩家操作完整。

防重犯：input owner兼容native與既有BP controller、驗證local viewtarget／possessed camera、UI focus／hit-test／wheel guard；輸入事件需導向actualrender camera或originalcommand ACK，helper／counter不作完整證據。新change complete-moba-controls-and-presentation與完整plan已strictvalidate，Grok4.7Fast source audit完成、P0 writer run-muy0acws-plucgx實作中，未先宣稱修復成功。A進入aim＋左鍵confirm與立即action分開；S/H、施法修飾鍵、DF／ward／ping／chat有獨立契約，避免假功能。原authority batch10/10不重設。Primary本輪曾猜OmWorldBridgeActor於OmRuntime/Private，盤點確認實際OmGenerated/Private，未改檔；E358路徑盤點規則同樣適用。

E361實作進度補充：上述廣泛writer由Primary於859秒後正式取消，當時未產生OmRuntime source修改，cost不可用，未把取消歸屬使用者。拆分的minimap pointer writer run-muy0u0pf-l853w3完成（312秒／12calls／USD0.36035784），Primary獨立review其2-file source；camera raw-input writer run-muy16o0t-brqg09仍執行。Primary已接HUD local view-target focus、owned gesture retirement、DPI panel wheel guard與實際投影的camera footprint邊界裁切，新增real controller stack／routed Slate fixtures；此刻尚未編譯或驗收，不援用前版71PASS替新source背書。Native routed fixture不是真人滑鼠／live PIE viewport證據，2.7仍待獨立驗收。另曾猜OmAutomationTestActors位於Public而實際Private，已用rg --files定位；E358防錯規則仍適用。

### E362 — 新source/header觸發source-engine metadata更新

controls-camera-build-v1正式Lua／NoEngineChanges建置於219.75秒exit5，錯誤FailedDueToEngineChange；列出的engine outputs均.modules，尚未C++編譯。Engine與project目前BuildId均ad19c95d-0383-4fa2-bde4-df0b0243dfc0，不能猜DLL不同版或手改BuildId。UBT原source在makefile invalidation及UHT後安排EngineMetadata write action，NoEngineChanges依existing ProducedItems攔下。既有handoff已明確設計與驗收--refresh-engine解此header更新情境（先正式source initialization，再正常gate／stageSHA），此次沿用該一般流程，不移除正常gate、不手改engine source/config/manifest/BuildId、不刪cache。

Primary審查亦發現新routed fixture不能省略HUD中間widget層而構造假window→minimap path，否則capture weakpath重建不成立；在首次native測試前改為GeneratePathToWidgetUnchecked與實際arranged geometry，仍標為native fixture而非真人／PIE。E358另有未先盤點的舊workflow名稱／UBT類別路徑猜測錯誤，改用rg --files所得exact paths；這些不是功能FAIL。錯誤log保持原樣，refresh與native驗收尚待結果。

### E363 — source-engine refresh未沿用明確local executor

controls-camera-refresh-v2正式SCW2.40秒exit0；OmGameEditor預設UBA建置21actions後超過十分鐘無completed action，compiler CPU在數分鐘內只增不到1秒，不能僅憑executor名稱宣稱掛住或殺程序。已驗證本次exec79834的Lua54960→cmd48852→private supervisor49384→Lua49528親子與來源，固定Lua native inspector保存exact creation_token/executable/PID後只stop supervisor；private Job KillOnClose回收本次descendants，exact Lua lifetimes退休驗證PASS，log與checkpoint保留。首次記錄要求compiler39324仍alive但它已退休，capture中止未寫receipt；改為optional compiler，不假裝原PID仍存活。

通用修正：build_ue_moba.lua允許--build-executor local與明確--bootstrap-engine／--refresh-engine共用；ue_engine_bootstrap.plan將官方-NoUBA送入SCW與OmGameEditor兩階段，最後正常NoEngineChanges／stageSHA照常。非法executor仍拒絕，local+verify仍在stage前拒絕。Source-engine admission／command回歸實際exit0，含default不改、local兩階段-NoUBA、maxparallel保留、正常既有engine不自動refresh與非法executor拒絕。controls-camera-refresh-local-v3以4並行執行中，已有[NoUba]編譯與連結完成動作；不手改Engine source/config/BuildId、關gate、刪cache或宣稱新功能PASS。Primary曾把MD更新與不存在的plan header匹配一併apply_patch，驗證失敗未寫任何檔案；移除錯誤第二個update後重套，不把tool失敗當source已修改。

E362／E363正式v3完成exit0：SCW2.31秒、source OmGameEditor302.28秒、正常NoEngineChanges3.82秒，兩stage SHA一致，checkpoint退役。local executor與明確refresh組合接受；camera行為尚需native／viewport驗收。

### E364 — 新camera native fixture建立ULocalPlayer的Outer錯誤

controls-camera-p0-v1本機驗收native exit3／success=false，執行MinimapCameraRoutedInput時NewObject<ULocalPlayer>()使用TransientPackage，但ULocalPlayer ClassWithin=Engine，UE ensure／fatal。MinimapCameraFootprintClip此前實際Success；其餘不能假設已完成，Blueprint階段未執行。Primary錯誤不歸屬Grok、不掩蓋crash或使用舊DLL驗收。

修正fixture兩处以GEngine為Outer，並建立LocalPlayer.PlayerController／Controller.Player相互關聯，讓real UUserWidget.GetOwningPlayer與native stack可以取得實際fixture controller；只測試EditorPreview、本scope不啟動authority。後續必須先正常NoEngineChanges編譯exit0，再新run-id執行74native／69資產；原v1失敗report與log保留。未把helper或原71PASS當本輪成功。

E364第二次fixture診斷：v4正常建置16.48秒exit0後，controls-camera-p0-v2仍native exit3；此次Outer錯誤已消失，RoutePointerDownEvent的GetVirtualPointerPosition讀到size0。UE5.8 GeneratePathToWidgetUnchecked給geometry-only FWidgetPath，該constructor沒有populate VirtualPointerPositions；pointer route需FWidgetPath(TArrayView<FWidgetAndPointer>)。已從同一完整arranged path建立pointer entries而非省略widget祖先，待正常編譯與新v3驗收。此為Primary測試API使用错误，不歸屬Grok或以skip新test避開。

E364第三次：v5正常建置exit0後，controls-camera-native-v3完成但兩項行為FAIL（clip成功），沒有crash。EditorPreview fixture未建立PlayerCameraManager；UE的SetViewTarget在manager不存在時不執行，GetViewTarget退回controller。修正兩個fixture以真正APlayerCameraManager.InitializeFor建立camera ownership，保留exact view target assertion，不改production放寬owner。v6正常建置exit0，新的scoped native-v4驗收中；之前失敗完整保留。防錯：fixture須確認Engine Outer、pointer path與真正camera manager，不能只呼叫SetViewTarget便假設成功。

E364後續native-v4/v5仍兩項FAIL；增加actual值後確認HUD.GetOwningPlayer=None、camera stack僅一個null controller entry。EditorPreview actors沒有正常註冊controller list；補World.AddController後native-v6小地圖routed與clip實際Success，但input stack增加owned component後UE5.8穩定排序解參考未初始化的controller InputComponent而crash。補真正controller UInputComponent並RegisterComponent，不移除BuildInputStack排序或測試assert。v9正常建置exit0，native-v7重跑中。防錯以完整fixture生命週期及actual owner／view target／非null input stack為準，不能為避開production接線降低assert。

E364結案範圍：native-v7 exact3 Success／0warnings／0games，full controls-camera-p0-v3 exact74完成（failed0／notRun0）＋69Blueprint compile/load零錯誤；不代表後續A source或actual真人PIE已驗收。

### E365 — A owner候選首次正式編譯錯誤

Grok4.7Fast候選與接入後，controls-a-owner-build-v1正式NoEngineChanges於95.09秒編譯FAIL（exit4；外層exit1），不是metadata／授權錯誤。Grok候選把TSharedRef.Get()回傳reference當pointer比較（330、342），以及WorldBridge local `Owner`遮蔽AActor.Owner而WarningsAsErrors拒絕。Primary修正取reference地址、改ActionOwner名稱。Primary fixture直接寫protected bAutoStartRuntime亦遭C2248，改Editor專用Bridge subclass constructor關閉auto-start，保留真BeginPlay／EndPlay及production guard。沒有跳過編譯或宣稱native acceptance。

防錯：TSharedRef.Get與TSharedPtr.Get的回傳型別不同；Actor方法內避免Owner等成員名稱遮蔽；測試配置protected property必須經合法fixture constructor。另查到使用者目錄2023年UBT log是舊專案資料，與本輪無關；以本次正式Lua捕捉的log判定，不援用舊global log。

Primary controls-a-owner-build-v2錯把只適用bootstrap／refresh的--max-parallel-actions與normal build合用，CLI在stage前拒絕，未編譯。移除不適用參數，v3仍明確使用合法local executor；不擴大既有CLI契約來掩蓋呼叫錯誤。

E365編譯修正確認：controls-a-owner-build-v3正式local／NoEngineChanges建置95.25秒exit0，14actions完成，stage SHA一致；scoped controls-a-camera-native-v1四項完成，full controls-a-camera-full-v1 native75（failed0／notRun0）與Blueprint69 compile/load零錯誤。新fixture有CleanupWorld缺EndPlay警告，補合法World.EndPlay再Destroy，不以mute／warnings whitelist避開。positive A確認的真人／連線command ACK仍pending，生命週期fixture沒有runtime或server，不宣稱四項fixture等於真實對局。

### E366 — 舊player ID解析會靜默借用玩家1

新A owner要求exact configured ID；Primary查ParsePlayerId發現舊code先抽出字串全部digits，再Max(Atoi,1)。`guest`／`player_0`變1、`a2b3`變23、負數變正數，不能當exact ownership。改單一ParseConfiguredPlayerId：只接受正整數或既有文件的player_<正整數>形式，ASCII digit完整驗證，UI int32上界，無效／零／負數／overflow返回0；既有正常default player_1保留，runtime配置及UE input owner使用同一解析，不以額外player1 fallback補洞。新增native回歸並正式build-v4驗證中。

E365／E366當前驗證：build-v4 normal local／NoEngineChanges326.57s exit0；scoped native-v2 exact5 Success／0warnings，完整controls-a-camera-full-v2 native76（69Success、7warning-success／13舊warnings、failed0／notRun0）與69Blueprint compile/load零錯誤，兩程序exit0。EndPlay新warning消失；原失敗／警告報告保留，positive A連線ACK及actual真人viewport仍未驗收。

### E367 — PNG 複製必須使用 binary mode

Primary首次以Lua path.read預設文字模式讀PNG，Windows Ctrl-Z造成資料截斷；驗證攔下，未寫專案檔。改read(source,true)、write(destination,bytes,false,true)、read(destination,true)逐byte驗證。第二次錯要求邊長能整除4，實際1254×1254；normalized quarter UV不需要integer tile pixels，改square尺寸及safe UV inset，原pixels保持。正式專案asset已byte-identical保存並記完整prompt/provenance至omfue/Content/Omoba/UI/README.md，尚待新HUD consumer編譯及顯示驗收。防錯：binary檔不得文字模式讀寫，依consumer的UV契約驗證，不另造不必要限制。

### E368 — 快捷鍵 profile 的 backup rename 不等於原子替換

Grok候選PublishAtomic把舊destination移至backup再搬temp，會產生正式檔不存在的期間，且LoadFromFile沒有backup recovery；固定temp名稱也可能競爭。Primary沒有接受atomic完成宣稱，已交同一Grok4.7Fast thread修成unique sibling temp、flush及真正OS atomic replace，失敗保留原檔，補實際替換／corrupt load／migration／conflict回歸。候選尚未正式編譯；catalog有action不代表已接runtime。另Primary本輪重新猜Source/OmRuntime及VisualRegistry module路徑而失敗，未修改檔；再次依rg --files定位Plugins/OmRuntime與OmGenerated，E358防錯要求仍適用。

E368 bounded repair run-muy53kmy-vzv7am completed599s／52calls／USD3.22969196。同thread使用Windows MoveFileExW REPLACE_EXISTING|WRITE_THROUGH／POSIX rename、unique sibling GUID temp、Flush(true)，沒有先delete／搬走destination。Primary review後production已promotion，新增Defaults／Migration／Persistence三項native回歸；尚未執行。公開profile header使用InputCore型別，Primary把模組InputCore依賴改為Public，不能依靠消費端碰巧另外引入。

### E369 — profile／HUD 新模組依賴令 metadata gate 拒絕

controls-profile-hud-build-v1 normal local／NoEngineChanges於UBT15.49s exit5 FailedDueToEngineChange，列出既有Engine .modules；UHT已完成但未C++編譯。沿用既有E362／E363正式 --refresh-engine --build-executor local --max-parallel-actions 4 流程，保留最後normal gate與stageSHA，不修改Engine source/config/BuildId，也不宣稱profile或HUD PASS。

E369正式refresh-v2 exit0：SCW2.29s、source130.12s（28NoUba actions含正常NetCore更新）、最終normal gate3.64s；原stageSHA一致，無手改metadata。

### E370 — profile 遷移／HUD 新 fixture 首次回歸失敗

controls-profile-hud-native-v1執行10項：8Success，ControlProfileMigration及HudAbilityIconAtlas Fail，process exit0不能取代exact test contract。遷移fixture先在memory改autoAttack／attackMoveOnCursor，但其v1 JSON未保存兩值，卻要求載入後保留；正式ImportJson採complete validated document取代profile，未寫欄位按defaults，不能改成沿用其他player舊memory。修fixture把兩設定明確寫入v1 JSON以驗證真正migration不遺失文件資料。HUD fixture忽略TakeWidget回傳shared reference，temporary析構觸發NativeDestruct及terminal retirement，清除狀態符合production；改持有實際Root完整測試生命期，不移除retirement guard或放寬assert。兩個原Fail與log保留，待正常編譯及新run-id重測。

E370正常build-v3 UBT17.25s exit0；native-v2 exact10 Success／0warnings，full-v1 exact81native及69Blueprint PASS。Real D3D12 HUD render-v2 exact1 Success，原native-v1與render-v1 FAIL留存。此scope無server／authority match，不能當positive combat ACK。

### E371 — 圖示 desired width 不保證實際保持正方形

Primary檢查render-v2 actual2400×1353 PNG（150%DPI）發現SBox.WidthOverride(56)仍被父VSlot預設Fill拉寬，方形atlas被壓成長方形。僅resource存在／cooldown ratio PASS不代表visual正確。改icon slot HAlign_Left，加入actual SImage cached geometry X==Y驗證；同步heroName auto-wrap避免窄欄裁切，能力名稱仍保持既有single-line測試契約。深色navy palette提升圖示對比。待正常編譯、實際PNG與complete acceptance；不以先前81PASS替新source背書。

### E372 — JSON version 必須在轉整數前拒絕超界

Primary後續review发现ImportJson先將finite double version cast int32才判支援版本。極大但finite的版本值可能觸發float-to-int超界，不應依賴平台轉型結果。改先double精確比較僅接受1／2，再安全narrow；新增1e100版本拒絕與memory保留回歸。此為corrupt／unknown schema契約內的修正，不新增格式或接受範圍。v4原HUD編譯84.42s exit0，v5將正式驗證這一增量，再跑actualPNG與complete acceptance。

E371／E372當前accepted source：build-v5 UBT40.21s exit0，render-v3 actual2400×1353 PNG exact1Success／0warnings且4slot geometry正方形，full-v2 exact81native failed0/notRun0及69Blueprint errors0/warnings0/loads0。native13個既有warnings保留。畫面是production Slate HUD fixture，沒有對局／server，未宣稱當前完整地圖及真人ACK。

### E373 — runtime profile 候選 cache／身份失效問題

Grok run-muy5l35h-bjx7qc completed783s／31calls／USD1.85741048，僅候選4檔。Primary完整diff review：ResolveConfiguredPlayerId失效／runtime stop返回時保留舊profile/path，write API僅看cached id能寫上一玩家設定；camera cache單憑revision，不同controller同revision可能沿用舊keys；clear只清keys而殘留zoom/edge/preferences；Sync先cache再忽略apply failure；改A鍵也重設Y lock。未promotion或宣稱PASS，交run-muy67wpz-d3gyta同thread bounded修正，要求exact owner+revision、current身份write gate、恢復完整defaults、success才cache、保留無關Y操作。

設定UI source亦在開發：用本local PC最高priority raw input barrier及独立modal guard，不能用UIOnly把整个共用viewport IgnoreInput而傷及其他local player。尚未接入／編譯，與舊81PASS分開紀錄。

E373後續：repair run-muy67wpz-d3gyta completed907s／20calls／USD1.24993792，Primary讀完整summary與四檔diff，guarded promotion後接設定component/widget與HUD按鈕。設定component在runtime identity暫時失效只close／清shortcut，保留同owner恢復能力；terminal retirement才不可復活。camera modal由action owner單一管理，避免兩個component互相清除同旗標。仍待當前source正式驗收。

### E374 — 新設定整合的 UE API／標頭及建置入口錯誤

Primary首次誤傳 build_ue_moba.lua 不支援的 --run-id，立即拒絕，未開始UE build；正式build依既有help呼叫，證據run-id用驗收helper。Normal local UBT5.58s FailedDueToEngineChange後走正式refresh。第一次refresh SCW2.27s，source UBT115.19s OtherCompilationError：Primary設定widget引用錯誤Styling/SlateColorBrush.h，正確Brushes/SlateColorBrush.h；Grok camera使用Actor不存在IsBeingDestroyed，正確IsActorBeingDestroyed；camera局部Controller遮蔽APawn member，改ProfileController。測試另避免直接存取protected ZoomStepCm，改以真input效果驗證。

第二次refresh source成功後，Primary在編譯中增改widget/header及新增test，final normal gate UBT7.93s再拒絕metadata。不可把source成功當正式build成功；完成HUD設定入口後凍結source，第三次正式refresh再跑normal gate。全部使用Lua既有流程，不改Engine source/config/BuildId或忽略NoEngineChanges。路徑搜尋曾再猜錯OmEditor與OmRuntimeTypes，依rg --files修正，E358仍適用。

第三次refresh final NoEngineChanges gate UBT7.31s exit0，stageSHA一致；scoped-native-v1 exact11 Success／0warnings／0games。

### E375 — 新 HUD 操作設定按鈕影響既有商店 fixture

controls-runtime-settings-full-v1執行82native，NativeShopInput Fail；新增操作設定按鈕預設可focus，而shop fixture將整棵HUD所有SButton當buy/sell rows，數量多1。修正production settings button IsFocusable(false)，避免點設定入口偷普通gameplay keyboard焦點；給明確OmControlSettingsButton tag。fixture保留每顆native button離線disabled／不偷focus檢查，只在buy/sell row count排除該明確tag，而非將預期6／7放寬為7／8。原failed report保留；重新正式build與完整驗收，不以process exit0或先前11Success當82通過。

E375修正正常build26.35s exit0，full-v2 exact82native（75Success／7warning-success／13既有warnings）+69Blueprint零errors/warnings/loads。再新增ControlRuntimePersistence：build15.06s exit0，scoped-native-v2 exact12Success／0warnings／0games；full-v3 exact83native+69Blueprint PASS。真presentation-only DLL127.0.0.1:0、初始化GI／實際local PC，證明玩家唯一profile寫入／讀回、Q conflict無revision／disk改變、identity0清path不能寫舊檔、identity恢復讀回J、actualRuntimeStop不能Reset，未建server／authority match／fakeStarted。

### E376 — 實際 Slate 設定按鈕 fixture 缺少 hover event

新增settings原生滑鼠／鍵盤操作後，scoped-native-v3 ControlRuntimePersistence Fail：只route down/up，未先route move讓SButton產生IsHovered，UE標準DownAndUp click不觸發OnClicked。Primary核對UE5.8 SButton.cpp403–431 public實作，保留production click語意，fixture補真RoutePointerMoveEvent＋hover／down／up handled assertions，不直接Execute delegate或手改hover來假裝真人操作。原Fail留存；等待build／real-event及render驗證。新的render helper僅擷取自己production settings window，使用公開Slate TakeScreenshot，非桌面GUI自動化。

### E377 — 設定視窗不能用別的 viewport 尺寸／focus 返回值判成失敗

controls-settings-render-v1 actualPNG產生但test Fail：click已讓settings取得焦點，再SetUserFocus同widget返回false表示「沒有變更」，非focus失敗。修fixture讀實際GetUserFocusedWidget相等，不放寬焦點要求。Primary看實際PNG發現panel只有160logical寬、rows幾乎不可見；PanelExtent讀UWidgetLayoutLibrary全局viewport，在獨立／local widget的實際allotted geometry不一致。移除全局pixel/DPI再換算，改Slate center slot Padding16＋desired720×640，由UE AlignChild clamp在實際local logical父範圍，字列auto-wrap與scroll維持。此修正適用多視窗／local-player／DPI，非只在fixture加硬編碼寬度。等待新build／render與回歸，保留v1 tiny PNG。

E376／E377修正正常build43.77s exit0，render-v2 exact1Success／0warnings／0games，actual1400×1578 production settingsPNG已獨立看圖：10row Chineselabels/key/rebind/unbind、bottomrestore/close完整；A改成J是本fixture真Slateclick/key/save結果，非預設鍵改動。full-v4 exact83native（76Success／7warning-success／13舊warnings）及69Blueprint compiler errors0/warnings0/failedloads0，兩程序exit0。

### E378 — middle／ally camera 候選沒有完整 owner／phase／world surface guard

Grokrun-muy7402y-z238jl completed1016s／66calls／USD3.67837772。Primary讀完整candidate h/cpp diff及tests：allycache在實際controller handoff後可能殘留；SetDisclosed不核Moba／Playing，允許minimap localID/team0與正hero身份混用，tests還要求phase2移camera；drag僅pixelrect＋cachedHUDflags，未核自己Slateviewport世界leaf/focus，projection失敗fallback全screen。未promotion，run-muy7uva0-i85w8j同thread三候選檔bounded修正，要求exact positive identities／phase1、nonzero currententity、handoff清stale、ownedSlatewindow/worldhit與projectionfailclosed；EditorPreview無實際viewport不能假稱drag位移PASS。

修正run-muy7uva0-i85w8j terminal546s／23calls／USD1.08074576，Primary完整讀source／tests後保存原production snapshot再promotion。整合中央PublishHeroHudState與minimap失效清空；新增真own viewport／projection／Slate pointer fixture，尚未編譯／執行，不標drag驗收成功。

### E379 — 讀全域滑鼠／user0焦點與modal布林會混用玩家

Primary核對UE5.8 public實作：FSceneViewport::HasFocus固定讀Slate user0，Grok drag及原AttackMoveClick使用FSlateApplication::GetCursorPos全域cursor、missing local user還fallback0。改使用exact local platform-user映射、own SlateUser cached pointer與focused widget，缺映射fail closed；native fixture用公開RoutePointerMove更新own pointer，不移動OS滑鼠。Modal新增source token，retired modal僅能釋放自己的阻擋，不能關掉replacement owner的guard；舊public modal API保持獨立累加。這批source須在Rust writer terminal後正式build驗收。

### 2026-10-07 camera preferences 已驗證增量

正式refresh後final normal NoEngineChanges gate2.72s exit0，bridge/content stageSHA一致；controls-camera-prefs-native-v1 exact12Success／0warnings／0authority games。camera設定負值拒絕不改disk/revision、合法設定原子存檔、wheel解除／恢復真input效果皆通過。controls-settings-render-v3 exact1Success／0warnings／0games，actual1400×1578看圖含camera lock／wheel開關與可捲动數值設定；這是中鍵／隊友新source整合前的DLL，不能拿來證明新camera source完成。

### E380 — Primary再度猜生成檔名／共用拒絕紀錄會覆蓋新動作

Primary查施法資料時誤猜不存在的OmAbilityVisuals.h，rg報missingfile；依rg --files確認實際OmAbilityStyles／OmVisualRegistry，未修改生成檔。未知路徑先inventory的E358規則仍適用。新增Stop時獨立核對FOmInputSubmitStatus實際成員為Error（不是LastError），且舊RecordRejection固定將action寫成AttackMove；在編譯前改typed action參數保存S/H原動作與NoTarget，錯誤紀錄不再假顯示A指令。原生負測試讀實際action、target kind、zeroinputid／notaccepted，仍待本批build。

### E381 — control frame 不是 minimap 建置錯誤／隊友死亡資料不能猜

Primary整合時曾把MinimapModel::Build false當invalid snapshot清空。獨立讀公開契約與實作後確認false僅代表null／control frame，正式snapshot失效本來就清state並returntrue；編譯前恢復control frame保留最近完整view，正式無效snapshot仍清空，避免每個control tick閃爍／丟隊友位置。另確認ScoreboardRow沒有Alive欄；不猜其死亡狀態，從已公開OmFrameEntity.hp有限且正值新增minimap hero bAlive，camera僅使用currentliving同隊marker，記憶ghost仍沒有alive／owner。補真正MinimapModel health投影與dead ally失效測試，本批仍待編譯驗收。

### E382 — Grok Stop projectile fixture借用temporary storage

run-muy8gdbu-jytqtn terminal1526s／166calls／USD8.62193624，Primary讀完整Stop差異，正式controls-stop-foundation-v1第一個core-stop Cargo exit101：E0716，game_processor.rs projectile storage臨時值在kept借用仍使用時被drop。Primary只將read_storage guard存成局部變数，再借get(projectile)，不改Stop／damage／projectile語意。失敗logs／errors.md保留；以新runid重跑，未宣稱未執行的其他case通過。

controls-stop-foundation-v2六case均exit0：core-stop10、protocol2、client-input15、client-stop2、bridge-stop2、server-stop2，合計33次測試執行（部分filter與lib/bin重複，不稱33個不同測試），零authority games。Server Cargo曾長時間沒有回傳輸出；僅read-only確認網路／程序狀態，未kill未知process或繞auth，最後原程序正常通過。仍需ABI17 cbindgen／DLL staging、UE按鍵與全部Blueprint驗收。

### E383 — 新 UE source 的 NoEngineChanges gate 需要正式 refresh

Stop／camera extra 批次 normal build 在 UHT 成功後以 FailedDueToEngineChange 結束，尚未編譯新 C++；不可當成 source 通過。改由正式 build_ue_moba.lua --refresh-engine --max-parallel-actions 4 初始化模組，再走 final normal gate，不手改引擎 modules／BuildId。編譯前另補設定介面 NativeOnPreviewMouseButtonDown，改鍵狀態先捕捉事件，避免 child button 消耗或換掉 CaptureAction。Primary 再次猜錯 WorldBridge 路徑；rg --files 查明是 OmGenerated/Public/OmWorldBridgeSubsystem.h，依 E358 不再猜檔名。

### E384 — 施法模型測試讀取取消後的舊 pending token

Grok cast candidate run-muy9hewv-m4otvq terminal 18m7s／42calls／USD2.6312464；候選尚未編譯或接入正式操作。OwnerHeroGeneration 測試在 Observe 取消 pending 後才用 GetPendingToken 比對舊 token，與測試意圖矛盾。交同 thread run-muyaa0lw-h2fw96 bounded repair：保存 Kept 當下狀態，取消後檢查清空，不改模型迎合錯誤斷言；不得據候選交付宣稱技能操作或 ACK 驗收完成。

Repair terminal 3m53s／14calls／USD0.64966656，Kept／Replaced 時點斷言修正已獨立讀取。Primary 續讀完整 target sequence 發現共用 Arm helper 的 !HasCommitted 會拒絕已有歷史提交的合法新 preview；只把 helper 改驗本次 Preview／pending token／Cast intent，保留原先提交歷史與所有負測試，未改 production 模型迎合測試。另記 Windows rg path wildcard 不由 PowerShell 展開，目錄搜尋使用 -g pattern。

### E385 — action seam 盤點建議混淆 attack-move-on-cursor 語意

Grok 盤點 run-muyaku9q-c9iyys completed 5m16s／46calls／USD2.11264032，僅寫 seam 文件。Primary 讀取發現建議把 bAttackMoveOnCursor 改成 A 立即送命令；這會混淆目標獲取偏好與既有 AttackMoveClick／A aim 操作，已改文件要求權威按點擊位置或英雄獲取目標，保留 A 確認流程。另修正「所有溝通都必須塞 PlayerInput」的過度推論：chat／emote 應按真正是否改 simulation state 選認證 team transport，不能用 catalog 或 RTT Ping 充當完成。盤點不算實作／測試驗收。

### E386 — Primary modal bridge guard 的局部 Owner 遮蔽 AActor 成員

正式 refresh 編譯 OmWorldBridgeActor.cpp:1625 出現 C4458：Primary 新 guard 的局部 Owner 遮蔽 AActor::Owner，警告視為錯誤。OmRuntime 已連結不代表 OmGenerated 完成；待建置 terminal 後統一改名 ActionOwner，保持 modal 語意，再用正式入口重編。Actor 派生類別避免 Owner 這種基類既有成員名稱；不得編譯途中改檔。

Refresh terminal exit6／1161.17s，只有這個 C4458；Primary terminal 後已改 ActionOwner。CameraProjectedDrag source 編譯過但沒有新 OmEditor DLL／執行證據；另依 UE5.8 真實 CreateViewport 自動 AddAssociation，去除多餘 deprecated SetViewport／SetViewportOverlayWidget，清理改公開 SetViewportClient(null) 觸發 RemoveAssociation，補真實 association 與焦點失去斷言。需下一批重編驗收，未改 Engine。

### E387 — cast settings Slot 遮蔽基類／fixture 呼叫 protected input API／null owner 比較

Grok cast-owner run-muyazqnr-n2vq4d terminal 1939s／140calls／USD7.810446。正式 normal NoEngineChanges 編譯 terminal 198.91s／exit6，兩個錯誤：OmControlSettingsWidget.cpp:170 的 Slot 遮蔽 UWidget::Slot（C4458）；OmActionOwnerTests.cpp:430 直接呼叫 protected APlayerController::BuildInputStack（C2248）。修復要求：使用 AbilitySlot 局部名，測試以真實 registered component 與公開 IsInputComponentInStack 找 owner，不修改 Engine access／假 input stack。Primary 獨立讀 source 另發現三個新 reflected seam 的 null==null 不能代表 exact local controller，必須明確拒絕 null 並驗 owner-specific error／zero id／清 sample。同 thread bounded repair 只授權上述三項；編譯中未改 source，不把 runtime 單一 object 成功當完整功能驗收。

同 thread repair run-muycey4r-72mrsm terminal 87s／7calls／USD0.68557328，三項逐一獨立讀取；正式 normal NoEngineChanges 22.83s exit0。之後 scoped native 仍因 E388 未完成，不稱整批成功。

### E388 — camera native fixture 建立 abstract UObject／GameViewportClient Outer 錯誤

controls-cast-camera-native-v1 actual native process exit3。13個 pure cast sequences 已 Success，但 camera fixture 觸發真實 UE ensure：NewObject<UObject> 建 abstract class；UGameViewportClient ClassWithin=Engine 卻用 GameInstance 當 Outer。Primary 改 nonabstract UInputComponent modal source，以 TStrongObjectPtr 保存；viewport client 改 GEngine Outer，保留 Init 真實 GameInstance／WorldContext 關聯。未修改 camera production gate／假 focus／pointer，保留失敗 logs，新的 HUD registry/portrait source同時加入後需重編及新 run-id 驗收。

### E389 — keyboard owner 已允許 HUD hover，bridge 卻仍全部攔截

Primary review 發現 CanSubmitInputEvent 的 bHudConsumesInput 仍只豁免 Source=HUD，Hotkey upgrade／no-target／recall 仍被攔。Self entity／point 的 reflected seam 又經舊 helper 改成 Source=Viewport，失去真正鍵盤來源。新 Grok recall/items packet 明確要求保留 consumed／modal／pointer guard，以 typed source 傳遞 verified self 操作，禁止暫時關 guard 或用 debug label 繞過；加真實 no-server negative gate regression，不能假 positive ACK。

### E390 — SlateUser 的 cursor pointer getter 仍讀 hardware cursor

controls-cast-camera-native-v2：31 Success／1 Fail／0 Warning，唯一 projected drag 未 arm。V3 D3D12 同樣31 Success／1 Fail／0 Warning；新增逐段診斷確認失敗在 own SlateUser pointer 不等於 routed event。讀 UE5.8 SlateUser.cpp:464–470 證明 CursorPointerIndex 分支回 Cursor->GetPosition，RoutePointerMoveEvent 不會移 hardware cursor。不能用 SetCursorPos／修改 desktop mouse 解測試。Primary新增通用 per-local-player FOmOwnedPointerTracker，僅保存實際 Slate input preprocessor 事件的 user／screen point／viewport widget/window，camera drag／edge 共用；失焦／owner退役／viewport替換清掉或拒絕 sample。Fixture 改公開 ProcessMouseMoveEvent，保留真實投影移動、release／focus／UI負斷言。新 source 尚待 Grok獨立writer terminal 後重編，不能把31項成功當全批通過。

Primary 續查 HUD NativeTick 仍取 global GetCursorPos／ControllerId fallback0，已共用 camera 的 exact-local event sample，焦點使用 platform user 對應；owner cast 的舊 hardware getter 留待正在執行的 Grok writer terminal 後統一遷移。實際 v3 HUD／settings PNG 已開圖檢視，頭像與 icons 正確載入，但 HUD 內部 ID 名稱／硬寫 Ctrl+Q 提示不適合作為完整介面；新增 manifest displayName 與實際 profile chord 投影，保留未知 content 的可讀 fallback、label 長度／control-character 檢查。這些新 source 尚未編譯，不用舊 DLL 的31項 Success 宣稱已驗收。接續讀檔再次猜錯 Source 路徑，立即按 rg --files 確认 Plugins/OmRuntime 路徑；此為 E358 既有規則，後續不再猜 module 位置。

### E391 — portrait 設了 desired size 但容器仍垂直填滿

Primary 開圖檢視 controls-cast-camera-native-v3/production-hud-fixture.png，方形原圖被拉成長條。源碼的 SBox Width/HeightOverride 只設定 desired size，內層 SHorizontalBox slot 預設 VAlign_Fill 仍把高度擴為整列。改該 portrait slot VAlign_Center，保留84×84容器與原圖 UV；render fixture 新增 actual cached image geometry 等寬高斷言，不能只靠 ImageSize 或 override 常數驗收。修正尚待下一次正式編譯及 D3D12 capture。

### E392 — 手寫呈現名稱重複既有內容來源

Primary 初版 displayName helper 手寫29名稱，續查實際 templates/moba_archetypes.lua 發現 vanguard_resolve 本來是「磐岩突進」，手寫「堅定意志」失去原內容語意。實際 OmContentIds／OmVisualRegistry 已由 templates display_name 生成；不得另維護一張翻譯表。改正式 build-time sync_ue_hud_manifest_labels.lua 透過既有 content_builder include heroes/abilities templates，自動投影 manifest 已登記ID名稱、拒絕不存在內容、不在遊戲執行Lua；接入 build_ue_moba 正式入口。首次實跑29entries／23updates，下一次應0updates。OmRuntime 保持不依賴 OmGenerated，未知未登記畫像仍不借用其他asset。

### E393 — 解除綁定隱藏動作，設定提示混入內部欄位

Grok Recall/item writer run-muyczzz2-sap8wj completed29m2s／180calls／USD9.7818068。Primary 獨立讀取發現設定頁跳過 !IsBound 的 action；解除 B 後 Rebind.Recall 一併消失，只能透過 API 或全體restore重新綁。已保留 implemented action 的未綁定列，顯示「未綁定」並保留改鍵按钮，新增 actual Slate unbind-row regression；D/F／ward等未實作動作仍不冒充可操作。另移除玩家tooltip中的 bAccepted／線路索引／語意slot等內部欄位，改成引導時間、中斷、作用對象、冷卻與飾品鍵說明，對應測試驗真正玩家資訊。Primary同批把action owner的快捷施法／AttackMoveClick pointer getter遷移到 E390 的共用實際事件tracker，input pop／profile變更／owner替換／退役時解除，active window失焦由tracker Tick清 sample。全部source已凍結，正式 normal NoEngineChanges建置正在執行；尚未宣稱新批 native通過。

### E394 — UE5.8 checked format／reflection array allocator／缺完整include

正式 normal build session48565 terminal exit6／161.42s。三項 source issue：OmControlSettingsWidget.cpp:31 將 ternary format pointer 傳给 FString::Printf，UE5.8 consteval checked format 要求literal；兩個新 Recall/item reflection resolver 用 TInlineAllocator<8>，但共用 CollectParms 固定接受16，allocator是不同的 C++ type；Primary OmPlayerController.cpp 使用 ActionOwner方法卻只得到forward declaration，缺完整include。Primary補 OmPlayerActionOwnerComponent.h，將另外兩項交同thread Grok bounded repair run-muyefhjr-c3s4vv，只改literal format與兩個local allocator，不擴大反射gate、不改Engine。原 UBT Log.txt 已由Lua保存至 controls-recall-pointer-build-v1/UBT-Log.txt，下一次建置不可抹掉失敗證據。

Repair completed23s／3calls／USD0.19479824；Primary逐處讀取兩literal format與兩16allocator，normal建置session72806 terminal32.83s exit0。

### E395 — 新 fixture 尚未初始化 subsystem／相機 component；JSON 型別被轉換

controls-recall-pointer-native-v4 actual D3D12 native terminal engine exit0，但strict report30 Success／4 Fail／0 Warning／0NotRun。CameraProjectedDrag 現已通過實際事件sample、viewport/window/focus、projection與SViewport hit path，但 real ray originZ0／directionZ0，表示未完成實際 camera component 啟用／更新，不能放寬投影gate或寫假ray。Primary fixture加入真World BeginPlay／Tick、驗camera componentregistered/active與actualviewtarget，再更新realcamera manager。兩個Grok新Recall/HUDguard tests只NewObject GameInstance而未Initialize，GetSubsystem為null；改真InitializeStandalone/world context/shutdown，runtime仍不start，不new一個假subsystem。HudAssetRegistry Test wrong-type displayName42被UE TryGetStringField轉成字串，意外取代acceptedregistry；production增加HasTypedField<EJson::String>，保留原atomic負測試。這四項皆保留v4 logs，修正後新run-id重編驗證。

Normal重編23.10s exit0，v5 native33 Success／1 Fail／0 Warning／0NotRun：Recall/itemHotkeys、HotkeyHudGuard、registryatomic負測試都通過，唯一camera active檢查仍失敗。Primary讀真正UGameInstance::InitializeStandalone，發現只建立dummy world並Init subsystem，並未InitializeActorsForPlay；NotifyBeginPlay與World.Tick不能替代actor/component初始化。補正式World.InitializeActorsForPlay(FURL())，拆開真component存在／registered／active斷言，再新run-id驗證，不直接改camera activeflag或POV。

v6 與 v7 都為33 Success／1 Fail／0 Warning／0NotRun；真camera registered/active、viewtarget、center ray、中鍵按下均通過。v7 的 processed end point、middle key held、own viewport focus、exact camera ownership也通過，但Tick後active=0且位置不變。續查自身tracker與input stack存活，不能用鬆綁focus/UI gate解決。

v8相同33/1且camera自身event sample前後皆存在。v9直接驗production ReadFocusPlaneHit，inside=false；v10實際EndPath leaf雖為SViewport但ContainsWidget(own viewport)=false，End=(2060.3,1037)命中另一視窗。Fixture原SWindow自動定位未保證拖曳終點位於自身實際hit-test區；改明確AutoCenter(None)/ScreenPosition(100,100)，不移OS滑鼠、不放寬跨視窗gate。新增header觸發真FailedDueToEngineChange，已經正式--refresh-engine後普通NoEngineChanges3.65s PASS；待新v11驗證。

v11固定視窗位置仍33/1，End=(1020.55,763)命中別的SBorder。真正座標問題為GeneratePathToWidgetUnchecked的配置路徑geometry與當前painted hit-test grid不一致；改ViewportWidget.GetTickSpaceGeometry取真實繪製的DesktopGeometry，保留原production hit-test與投影驗證。v12實際D3D12 strict34 Success／0Fail／0Warning／0NotRun，真正中鍵移動、鬆開不續移、失焦退役與UI遮擋負測試皆通過。Primary已開圖檢視本批HUD與settings PNG，頭像等寬高／原創icons／中文名稱及已保存改鍵顯示正確；空白世界是無server fixture，不代表正式地圖畫面。完整105 native＋既有Blueprint驗收已啟動，尚待terminal。

### E396 — D/F 隔離候選未經實編便有 Rust move／shadowing 錯誤

Grok候選 run-muyeyfb2-8qgd9u completed7m28s／9calls／USD0.54687912。Primary獨立跑 verify-summoner-candidate.lua，實際cargo test六個compile errors：Deserialize索引移出Option<String>陣列 E0508，以及測試local loadout遮蔽loadout() helper E0618。保留候選 validation.log，同thread bounded repair run-muyfad2h-353dns 要求owned array mapping/destructuring與helper一致改名，不刪斷言。此候選尚未接入正式遊戲、尚未驗收，不可當D/F已實作。

Repair terminal30s／5calls／USD0.25165304，Primary核對slots.map及equipped_loadout改名，獨立重跑9 unit tests全過；尚有4個測試warning，後續production promotion清除。新run-muyfk2e6-37757p開始正式runtime／Lua catalog foundation，FFI／world／transport／HUD尚未接通，不能把隔離候選通過視為遊戲D/F成功。

### E397 — 全量驗收仍固定NullRHI，無法驗painted viewport命中

完整controls-cast-recall-hud-native-v12-full terminal失敗：97 Success／7 Warning-Success／1 Fail；CameraProjectedDrag在painted own viewport命中斷言失敗，NullRHI無法提供該真實render契約，不可跳過測試或改假hit-test。正式ue_local_acceptance.lua增加明確--native-rhi d3d12|null，原生預設D3D12，Blueprint編譯維持NullRHI；report記錄兩種RHI，嚴格105test清單／Success／errors／counters規則不變。新的controls-cast-recall-hud-native-v13-full實際105native及69Blueprint完整PASS，兩程序exit0；warning具體數量以summary.json為準，不能把舊7個warning-tests說成0warnings。新backend foundation尚未編譯，這是目前UE鏡頭／cast／Recall／item／HUD增量完整回歸，非未接通D/F的驗收。

### E398 — 查檔路徑與 Lua helper API 未先讀取便猜測

續做 F9／召喚師核心驗證時再次猜成獨立 Plugins/OmEditor、templates/templates.lua 和 scripts/lib，rg 回報不存在；正式目錄是 Plugins/OmRuntime/Source/OmEditor，Lua library 由 _bootstrap 指向 tools/lua/lib。新驗證腳本又呼叫不存在的 path.mkdir，尚未執行 cargo 就 exit1。已讀實際入口，改成既有 path.mkdir_p 並拒絕重用證據目錄。後續未知檔案先 rg --files，API 先讀實際定義／現有使用例；不得將此 workflow 失敗算為 Rust 測試結果。

同批 collaboration state 更新第一次將 json.write overwrite 設 false，既有 state.json 正確拒絕覆寫，exit1；內容尚未變動。讀取 tools/lua/lib/json.lua/path.lua 與已成功的記錄入口，改既有狀態檔專用 true；新驗收證據仍禁止覆寫，兩用途不能混淆。

### E399 — F9偏好更新不應清掉其他按住的鏡頭模式

Primary source review 發現既有 ApplyValidatedControlProfile 在任何revision都ReleaseProfileHeldModes並pop/rebind camera input。新增F9透過atomic save發布profile後，会意外结束仍按住的Space／方向鍵／中鍵。改比較真正camera binding snapshot：owner／binding／lock-default改變仍清held state；只改偏好則保留原camera input和hold/release配對。加入真InputStack Space→F9→Space release斷言。首輪普通建置另實際FailedDueToEngineChange（UHT114.52s，UBT132.66s，整workflow exit1），將Log保存在controls-edge-build-v1後走正式refresh；不直接改Engine metadata。新camera/F9 source尚待重編與native驗證。

### E400 — UPROPERTY可供Blueprint讀取不表示C++欄位是public

正式refresh session14835 terminal exit1；UBT926.42s／exit6，八個C2248均來自新測試直接讀取protected bEnableEdgeScroll／EdgeScrollSpeedCmPerSecond。原欄位位於protected區，不能憑BlueprintReadWrite或摘要推定C++可直接存取。保留controls-edge-build-v2/UBT-Log.txt，新增public唯讀IsEdgeScrollEnabled/GetEdgeScrollSpeed，原封裝與實際InputStack斷言保留；不提升欄位可寫權限或刪除失敗測試。下一次正式refresh完成未完Engine graph再走normal gate。UE5.8 ExecutorFactory明確永遠使用UBA executor，-NoUBA實際關detouring，日誌[NoUba]也已確認；不能僅因UBA字樣推定參數失效，更不能加入不存在的NoUBALocal或修改Engine config。

修正後formal refresh session70183實際18 actions／121.29s exit0，接著normal NoEngineChanges為0 actions／3.44s；正式stage驗證bridge135c9a1ad4e8f35a0089e285b842664df9f84b43e884c26a630473ab7568d999、base8363e4ad550b6e1f76f2432494fa5c2058306a7a50a6bb7940bd1bdfc85f5739、content d5553bbb31c459a4。controls-edge-scroll-native-v1 D3D12實際34 Success／0Fail／0Warning／0NotRun，包含ControlRuntimePersistence真F9保存／F10改鍵／Shift負例／modal／Space保持與原release；Primary已開兩張實際PNG。此證據接受F9增量，不接受後續正在編寫的共用HUD或D/F ECS source。

E398補記：不可在process.run尚未返回時讀取其「完成後才保存」的validation.log；先等terminal或檢查path.exists。runtime/input實際位於native/input.rs，續接wire前仍應先rg --files再讀，不能依名稱猜路徑。

### E401 — 共用HUD首次source review發現來源回切、全量隱藏與不完整fixture

Grok初次共用HUD實作run-muyh39ym-f0vm78 terminal completed，但尚未建置驗收。Primary獨立source review確認：每次Publish呼叫Ensure並重新取得epoch，仍存活的舊bridge可把新來源回切；舊HUD隱藏函式遍歷collapse所有root child，會隱藏無關新增widget；Upgrade／Recall smoke仍依賴native controller cast，未涵蓋plain／BP入口；新fixture只有NewObject GameInstance和NotifyBeginPlay，缺真實初始化。已送同thread bounded repair run-muyi3b2z-72gb8x，要求生命週期選定來源與固定token、明確legacy名稱、四種真InputStack smoke、正式GameInstance／world初始化及實際publish回歸。不放寬owner／focus guard，不把source編寫完成算成驗收通過。

Primary同步檢查商店改鍵：輸入component重建後原EnsureActionInput只恢復A模式的Esc，開啟商店時改鍵會遺失關閉優先序。補目前商店／施法預覽的Esc重綁，新增「商店開啟→改鍵→真Esc關閉」測試。商店PNG fixture使用ForceRedrawWindow取得實際繪製，不能只以Slate.Tick視為完成paint。以上新source仍待正式建置與實際native驗證。

同thread repair terminal17m58s／74model calls／USD4.1279434；Primary再次獨立review發現worker已修HUD token，但Hero publish仍可把action owner與camera回切，且舊bridge EndPlay仍無條件StopRuntime，可能停掉B共用runtime。Primary補AcquireLocalHudSource同時handoff action owner、只允許selected來源發布owned hero/camera，舊來源EndPlay只清自身presentation。加入實際A→B→A stale publish／EndPlay的action owner斷言。正式普通build session96757已啟動，尚未宣稱通過；model calls不能當tool calls宣稱超出工具配額。

### E402 — Blueprint反射不能暴露uint32回傳值

共用HUD正式普通build session96757實際exit1，UBT49.25s／exit6。UHT唯一錯誤為OmWorldBridgeActor.h的GetCapturedHudSourceEpoch：uint32不支援Blueprint ReturnValue。此getter只供C++診斷／epoch測試，移除BlueprintPure標記並保留原uint32及完整斷言；不轉成可能溢位的int32，也不放寬來源驗證。新Lua建置證據入口先保存原UBT log，再重跑正式普通gate；本次Rust bridge／base已重編與stage但UE尚未成功，不能列為整批完成。

普通v2已通過該UHT型別錯誤，但實際UBT39.55s／exit5為FailedDueToEngineChange；證據controls-common-hud-build-v2/UBT-Log.txt。依既有正式流程啟動v3 --refresh-engine --max-parallel-actions12，不修改Engine config／metadata；refresh成功後仍必須再跑普通NoEngineChanges及真native驗收。

### E403 — Slate參數與controller輸入stack存取權限必須核對實際宣告

v3 formal refresh實際UBT315.72s／exit6，共三個compile errors：Primary商店焦點與CameraExtra測試兩处ContainsWidget傳TSharedRef，但UE5.8只接受const SWidget*；Grok通用smoke呼叫protected APlayerController::BuildInputStack，造成C2248。已核對實際Engine宣告與PlayerTick implementation：改用ShopPanel.Get()，smoke經public PlayerTick讓Engine執行其TickPlayerInput／BuildInputStack，保留plain／BP hooks與真InputKey press/release；不提高protected權限或改回native cast。另新owner兩处ForEachObjectWithOuter boolean overload改成實際EGetObjectsFlags::None以清除新deprecated warnings。保留v3完整log；source修正後重走正式refresh完成graph。

### E404 — 新跨module輸入符號需明確InputCore依賴；HUD token也須綁controller生命週期

v4 formal refresh的C++編譯錯誤已排除，但UBT38.50s／exit6在OmGenerated連結失敗，五個LNK2019均是FKey::IsValid及EKeys四種modifier。OmGenerated.Build.cs沒有InputCore私有依賴，新增正式module dependency，不依賴OmRuntime間接鏈結。保留v4/UBT-Log.txt。

續查來源生命週期又發現CapturedHudSourceEpoch只識別bridge：同一bridge在本地controller替換後仍保留非零舊token，會拒絕新controller首次HUD取得。補token對應的weak controller identity，僅在exact registered controller實際更換時清舊capture；同controller退役仍拒絕重建。新增真ULocalPlayer換綁／舊controller EndPlay／同bridge publish測試，保留舊widget終止與新widget唯一性断言。這是source review發現，尚待新native驗證，不稱為已發生對局故障。

### E405 — 真實小地圖輪廓paint觸發TArray自我元素追加崩潰

v5 formal refresh＋普通NoEngineChanges實際完成exit0，普通3.66s；bridge f24c4e42393c0db15d86c2eb592c08efeb4b11917b7fab0eaa09e476a392c834、base f39677befe880e769ecd858a5d94c203bb07bb0032d66751166d9e26c3d1018e正式stage一致。controls-common-hud-shop-native-v1期待41項D3D12實際測試，但程序exit3；在CameraProjectedDrag新production HUD真paint時崩潰，無完整報告，不能宣稱41項已執行或通過。

Actual callstack指SOmMinimap::OnPaint line185，ViewOutline.Add(ViewOutline[0])違反TArray CheckAddress；line162的remembered marker Outline.Add(Outline[0])也有同樣自我引用。先複製FVector2D起點到獨立值，再追加，保留真正封閉線段與camera outline；不以Reserve、關斷言、隱藏輪廓或刪測試避開。保留v1 native-engine.log與summary，重編普通gate後用新run-id重跑全部41項和三张PNG。

### E406 — 真viewport／widget ownership與selection fixture缺口不能當作無害warning

修正paint崩潰後ordinary buildv6 PASS；native-v2實際process0，但嚴格report為33Success／8Fail／0NotRun。CameraProjectedDrag的真pan／paint通過，zoom未更新outline、focused-shop close未還原focus。Primary補真正World.Tick讓spring-arm component更新，並把collapse移到focused descendant路徑檢查之後，不改desired值斷言替代真projection。

其他實際失敗：shared HUD fixtures缺usable game viewport，AddToViewport警告後Slate wrapper銷毀觸發正常NativeDestruct，導致其他controller／replacement／P與K的bar退役；widget計數和root／selection discovery假設CreateWidget以controller為Outer，必須核對真GetOwningPlayer；legacy panel test NewObject抽象UUserWidget產生EnsureFailed；selection fixture缺launch paths／identity並對nonfocusable widget設UIOnly。已把真index.json與bounded repair packet交回同Grok thread；要求real viewport／layer／window fixture、exact player/world ownership、concrete automation class、valid selection paths與focus，不禁NativeDestruct、不改預期值、不抑制錯誤或寫privateflags。本批仍未驗收。

### E407 — Automation模板參數不能直接傳TObjectPtr

修正真viewport後，普通v7因新增source要求EngineChange；formal refresh v8實際158.27s／UBT exit6，唯一編譯錯誤是HudOwnerLegacyPanelVisibility把Root->WidgetTree（TObjectPtr）直接傳給TestNotNull模板，無法推導原始指標型別。改為WidgetTree.Get()，保留null斷言及實際concrete widget tree，不改測試預期或移除測試。完整失敗證據保存在controls-common-hud-build-v8/UBT-Log.txt，接續新id重編；此時尚未有新native通過結果。

### E408 — Layout widget path不能直接用於滑鼠routing；平滑縮放需要非零delta

v9正式refresh＋最後普通NoEngineChanges成功，普通3.76s。controls-common-hud-shop-native-v3實際D3D12程序exit3，在ShopPanelSearchAndRetirement崩潰；沒有完整native report，不宣稱41全過。callstack為FWidgetPath::GetVirtualPointerPosition→RoutePointerMoveEvent→OmShopPanelTests.cpp88。GeneratePathToWidgetUnchecked建立layout路徑但未填每層VirtualPointerPositions；不能直接當pointer path。改為ProcessMouseMoveEvent＋LocateWindowUnderMouse的實際painted hit-test路徑，並要求path包含目標分類button，再route按下／鬆開，不自行補假的指標資料。

同批日誌明確記錄shared HUD五項、ControllerOwnedHudLifetime及ControlRuntimePersistence Success，focused-shop Esc已通過；CameraProjectedDrag只剩effective zoom outline失敗。測試SetZoomDistance後Tick(0)不能推進FInterpTo，改用真camera Tick非零delta，保留World.Tick更新components與projection，新增實際camera component位置改變斷言及before/after診斷。若新證據仍失敗需繼續修實際tick／projection，不能改成desired值通過。v3完整引擎log和crash保留，v10新建置／v4新native將重新檢驗整批。

v10普通16.29s PASS；native-v4完整41項為40Success／1Fail／0Warning／0NotRun，三張PNG存在，商店分類真pointer routing已通過且shop圖已開啟。唯一CameraProjectedDrag失敗新增診斷顯示desired3000但CameraComponent位置仍是原值，不能當作「outline比較太嚴」放寬。核對Engine TickTaskManager.cpp確認component的TickVisitedGFrameCounter去重：同步RunTest內多次World.Tick仍在同一Engine frame，並不重跑spring arm。fixture改用公開USpringArmComponent::TickComponent執行正式UpdateDesiredArmLocation，再讀actual projection；不手改TargetArmLength、socket／transform、Engine global frame或fake activeflags。此仍是同步native fixture，跨Engine frame／PIE整體入口矩陣保持未完成，後續須latent／實際viewport驗收。

v11普通24.49s PASS，native-v5 exact41Success／0Fail／0Warning／0NotRun，三張新PNG全部實際開圖；camera component位置及投影輪廓都已通過。此為本批scoped接受，完整整套功能仍未完成。

### E409 — 商店交易測試必須識別交易action，不能把本地UI按鈕混入權威限制

完整controls-common-hud-shop-native-v6-full native111為102Success＋8warning-success／1Fail／0NotRun，程序exit0但嚴格驗收拒絕，Blueprint階段未開始。唯一NativeShopInput仍把除settings外所有SButton算成交易列，新增開啟／關閉／三分類五個按鈕使6／7的交易列變成11／12，且本地UI按鈕正常enabled被錯認成允許斷線交易。補實際sell button的OmShopSell.<slot>標記，與既有OmShopBuy.<catalogId>一起精確辨識交易；保留6／7／7／6原期待數量、斷線交易禁用及所有button不可搶焦點的斷言，不修改CanSubmitShop或假造連線。v6-full失敗report保留，v12重編後用全新id重跑完整111＋69。

v12普通24.38s／exit0；full-v7 native111（103Success＋8warning-success／14既有warnings、failed0/notRun0）及69Blueprint（compiler errors/warnings/failedLoads0）正式通過。此只覆蓋當時source；新Tab source仍待建置，不借舊結果。

### E410 — Bridge CLI和patch格式須以實際入口契約為準

Primary查新wire thread的resume方式時，run --help實際被當作unknown option並exit1；不代表Grok驗證失敗，也沒有啟動job。核對CLI宣告後確認--resume是boolean、只resume最新tracked task，不能傳任意foundation UUID。新wire是新scope，使用--fresh與durable foundation integration，未手改metadata或繞過單writer。一次apply_patch缺terminal換行被工具拒絕，未修改檔案或執行後續state script；補完整End Patch＋換行後成功，實際Lua state讀回通過。兩類錯誤應先讀工具契約，不猜argument或以失敗腳本當成功。

另一次Primary假設summoner-world-validation-v2有summary.json而讀取失敗；實際verify-summoner-world-v2.lua只保存summoner.log／stop_command.log。重讀已存在的Lua入口確認後未改既有證據，應先inventory或核對產物契約，不能將一般UE驗證的summary格式套到另一個helper。

### E411 — Rust巢狀test模組路徑與持續projection不能靠原始碼宣告驗收

Grok wire/projection job實際terminal completed1535秒／186model calls／USD9.7884844，未執行測試。Primary第一輪summoner-wire-validation-v1的core summoner與Stop lib-test都實際exit101；summoner_world.rs新增tests八處super::comp指向summoner_world而不是native。完整第一輪已terminal：後端summoner3項及client summoner1項PASS、maintained backend／client all-targets check兩者exit0；helper依core兩項失敗正確exit1，不能借其他target通過宣稱整批通過。實際summary和所有日誌保留。修正時先核對模組層級，保留原有斷言，不跳過test target。

獨立source review另發現single_lane_replay_digest未包含persistent summoner loadout／cooldown，DisclosedSummonerProjection只有bootstrap安裝、沒有接收後續四種owner/team HUD facts的consumer，會保留過期clock。這些是既定contract缺口，須同thread bounded修正與實際zero-game驗證，不能以metadata存在或函式同名當作持續同步成功。ABI／UE D/F仍未接上，不宣告完整功能。

### E412 — 新增檢查仍須先定位實際檔案，不能猜通用Types／snapshot名稱

Primary後續readonly檢查誤猜OmRuntimeTypes.h與runtime/render_snapshot.rs，rg實際回報檔案不存在；沒有修改或驗收這些不存在的檔案。實際英雄HUD payload在OmRuntime/Public/OmUiEventTypes.h，已用rg --files／符號搜尋定位後重讀。新的跨ABI盤點一律先inventory與符號搜尋，不沿用其他專案慣例拼路徑。repair packet先前將digest函式寫作single_lane_state_digest，dispatch前核對實際declaration並修正為single_lane_replay_digest，避免子代理依錯誤名稱改碼。

readonly建置診斷同樣誤猜Executor檔案與test入口，後續用rg --files定位UnrealBuildAccelerator子目錄。退休helper第一次使用不存在的path.is_dir，未執行任何停止；改用已存在的path.exists後，第二次expected executable因Windows反斜線比對失败而安全拒絕，仍未停止。實際inspect_owned確認字串後統一斜線並鎖定原始creation token，只有完整驗證通過才執行。Lua API要先核對已讀library，不能猜名稱；Windows實際path格式要標準化，不能為通過安全檢查而放寬exe/lifetime限制。

新增Ctrl+F patch第一次在overlay之後附上位於前段的多餘Capture函式context，工具verification拒絕整個patch。實際rg確認零變更後重新用ascending hunks並刪去多餘context，才成功寫入；不能假設跨檔patch已部分成功，也不能在未確認結果時重複追加欄位。

### E413 — 編譯超時要保留實際取消證據；UE5.8 NoUBA不等於沒有UBA local executor

controls-scoreboard-screenshot-build-v1 Rust/base/bridge建置與staging完成，但UBT普通NoEngineChanges從05:43:18開始、UHT146.857s後停在24actions第一個Module.OmRuntime.cpp；MSVC單一thread長時間Executive等待，無completed compile action。超過十五分鐘後，Primary透過正式native Lua process API核對exact executable與creation token，先保存UBT-stall-before-retirement.txt與退休JSON，再只停止本次compiler70444原始lifetime。UBT明確記錄該action exit1，接著退休本次已失敗UBT68980原始lifetime，由既有supervised workflow回收其descendants；helper終局exit1，完整build.log／UBT-Log.txt／summary均保留，核對無cl殘留。這是明確timeout cancellation，不是假造C++diagnostic、編譯成功或native驗收。

讀Engine ExecutorFactory與UnrealBuildAcceleratorConfig確認UE5.8永遠使用UBA local executor；NoUBA只禁detour，不禁action cache，不能因log出現UBA就宣稱CLI被忽略，也不能猜不存在的Parallel／NoUBALocal旗標。專案Lua新增明確每次執行--disable-action-cache（要求local build）轉成已核對的-nocache；不改Engine／全域config、不reset或刪cache。下一輪用新evidence id進行無快取診斷，原因尚未證實、Tab/F12仍未接受；同thread Grok wire repair先接續，修正terminal前不跑新的formal build。

### E414 — 新增測試必須編譯實際 lib-test target

Grok bounded repair run-muynfdhg-2w0nrx terminal completed432秒／52model calls／USD3.14908408，未執行測試。Primary第二輪 summoner-wire-validation-v2 再次發現 core summoner／Stop lib-test exit101：filtered_specs test 缺 DisclosedWorldStepper trait 匯入；summoner_world 持有 resource Fetch 的 receipt 引用跨越後續可變 world 使用；FactAudience 從 shared fact 被移出。後端與 client summoner 測試仍通過，不能用非 test target 的編譯通過代替核心測試。修正保留原有斷言及真正 fixed-step 接線，限定 borrow scope、借用 match 和實際 trait import；失敗日誌保留，新 evidence id 重跑。

Primary本輪唯讀搜尋再次誤用 comp/moba_match.rs 及 client-runtime/filtered_specs.rs，實際位於 core/runtime/native 及 core/runtime；已重新 rg --files 定位。產物也僅在 process.run 終局才落盤，不能在運行中把尚不存在的 client-summoner.log 當作遺失證據。

### E415 — Wire record 寬度須從欄位推導，不能硬寫錯誤常數

第三輪 core summoner 實測33通過／7失敗，Stop10、backend3、client1及兩個all-targets check均通過。encoder每玩家u32 ID＋兩組u16 catalog／i64 clock實際24bytes，decoder卻寫22，造成roundtrip、owner/team、dead/reconnect與真正fixed-step bootstrap測試失敗。Primary改為欄位推導常數24，保留count／長度／身份／冷卻／catalog拒絕，新增28byte單人、空rows及未知catalog負例。拒絕交易fixture原先沒有wall卻要求合法blink不移動；補上既有wall fixture，保留全部狀態斷言。phase guard誤把imports及profile字串當call順序；搜尋實際帶括號的drain呼叫，保留19phase順序斷言，不刪除檢查。新一輪證據驗證後才接受。

v4實測全部通過：core40／Stop10／backend3／client1及兩個all-targets check，zero authority games。只接受wire／fixed-step／Rust，D/F ABI與Unreal仍pending。

### E416 — 普通建置真正拒絕 EngineChange 後才使用既有 refresh 流程

controls-scoreboard-screenshot-build-v2 實際UBT208.76s後FailedDueToEngineChange、workflow exit1：列出NetCore DLL／obj及Engine modules等待更新產物，並非C++diagnostic或新的compiler timeout。保留build.log／UBT-Log／summary。下一輪透過既有--refresh-engine建置而不手改Engine source／config／BuildId；之後仍須普通NoEngineChanges通過。新 per-run disable-action-cache 必須同時傳入refresh／ShaderCompileWorker與普通階段，避免同一正式workflow兩段使用不同action選項。Lua planner新增local-only限制與實際argument測試，helper可組合refresh和no-cache。失敗不能當作Source或ABI已接受。

v3 refresh實際28actions成功949.40s，最後普通NoEngineChanges成功2.85s，實際bridge SHA95f3865537b6cb82aa25c4b5bff0c09ec548776e09858e3b30fdbc5cdd1ddd50、base09b0862bb7789ff2b879860e642540b00013bcdcd997928620694de6aabb5e47驗證通過。只接受編譯／stage，功能另跑新native。

### E417 — 編譯成功仍須實測 profile rebuild、截圖 readback 與焦點生命週期

controls-scoreboard-screenshot-native-v1 D3D12實際41測試中40Success／ControlRuntimePersistence Fail，Editor process0但正式Lua契約正確exit1；全部日誌與報告保留。新斷言揭露InstallRuntimeProfile先PopActionInput，導致後續EnsureActionInput把元件當已失效、關閉未改鍵的held Tab。修正讓既有EnsureActionInput在舊bindings仍存活時依revision重建，保留相同chord／activation，仍關閉改鍵／失效情況。不能只在Ensure裡新增preserve分支卻忽略真正caller先拆元件。

F12實際保存失敗，當時只有generic notice，原因尚未充分定位；依UE5.8實際TakeScreenshot契約改為驗至少width×height像素並只編碼有效切片、重畫精確owned window，加入owner/window/readback/encode/directory/write的Display診斷；保持成功必須真正寫檔、禁止fake PNG或放寬ownership。焦點恢復後的CtrlF fixture補新Ctrl按下，不假設失焦清鍵後仍按住；設定頁新增正常Slate tick/layout後才送下一鍵。均須新build/native驗證，不預先宣稱根因完全修正。

Primary為修目前失敗而取消patch-only Grok run-muyokwum-hcf4m2，terminal cancelled16m57s，無patch/report產物、費用未知，不稱它完成D/F或已改source。已結束所有active UE／Grok source readers後才修目前source，下一批重新以實際新base接續，不套用過期patch。

### E418 — 截圖要驗真正 render rectangle，不能把 readback false 當成功

普通controls-scoreboard-screenshot-build-v4實際40.33s通過；native-v2仍40 Success／ControlRuntimePersistence Fail。Tab held跨profile revision與CtrlF開關已通過。實際TakeScreenshot回read=0，宣告1400×1053但只有428505 pixels，不能忽略回值或用不足pixels編碼。新增分段診斷成功區分為readback而非目錄／寫檔錯誤；接下來核對UE5.8實際widget/window/backbuffer座標契約，保持exact owned viewport與window身份驗證。另兩項是diagnostics bound visibility更新時序及actual settings焦點事件路由，必須實際Slate事件證明，不能改為直呼handler並稱GUI已通過。Grok4.7Fast run-muypi312-xo8ieu接bounded source repair，Primary終局後獨立驗證；所有失敗report保留。

本輪readonly再次誤猜OmRuntimeSettingsWidget.cpp，rg明確找不到；實際inventory為OmControlSettingsWidget.cpp。未修改不存在檔案。後續packet已使用實際名稱。

Grok run-muypi312-xo8ieu在548秒後由Primary取消：來源檔無變更、只有讀取前的commentary，沒有terminal修正成果；費用未知。bridge claim-before-kill成功，兩個exact PID皆確認不存在，才由Primary接手，不稱為human取消或Grok完成。

Primary核對Engine實作與真log：Screenshot rect [1405,1114]被截到[800,600]，起點[5,61]，所以實際795×539=428505；不是PNG編碼失敗。正式修正從同一owned viewport的真arranged geometry、精確window offset及SWindow.GetViewportSize計算可見交集，明確傳給TakeScreenshot，仍要求true與完整pixels，不捕捉其他window或製造圖片。設定焦點問題中SetInputMode將Slate操作排到LocalPlayer；同一attached focusable settings widget增加公開SetUserFocus(Controller)，讓第一鍵立即走真正preview，不移除modal raw-input barrier。diagnostics test在檢查bound visibility前增加正常Slate layout/tick；全部原斷言保留。v5普通建置與fresh native驗證後才可接受。

結果：build-v5普通NoEngineChanges33.46s／exit0，fresh native-v3 exact41 Success／0Fail／0Warning／0NotRun，Lua helper及Editor都exit0，zero authority games。五次真F12包含owning shop及settings成功寫不同794×538 PNG；Primary實際開圖檢視settings F12及三張production HUD／settings／shop圖。只接受當批Tab/F12/CtrlF/input子功能，C stats／整套畫面與D/F仍pending。截圖顯示測試world的legacy TD controls與共用HUD，不能當正式三路match的視覺完成證據。舊111／69不借用為本批final regression，等全部功能整合後執行最終完整驗收。

### E419 — 後續操作設計也須核對真正 enum 與公開 target 語意

Primary唯讀盤點誤猜disclosed_replica.rs及Generated/Public/OmUnitActor.h，實際前者struct在selective_replica.rs，後者在OmRuntime/Public；另Windows rg literal Om*Tests.cpp不展開glob，改用目錄加-g。沒有修改不存在檔案或執行測試。計畫中MoveTo是wire語意，真正UE enum為Move，不能照文字直接拼C++ enum。SampleOwnedCastPointer回傳visible entity並未做enemy filter，ABI owner也不等於team；下一批attack-only／只選英雄不能用可見性或owner猜隊伍代替真正合法target validation。具體設計另記moba-direct-click-actions-design.md，未宣告實作或通過。

### E420 — 文件 patch 必須用完整實際行匹配

原創D/F資產文件更新有兩次apply_patch verification拒絕：一次把段落末句當獨立行，另一次留下未對應檔案內容的placeholder hunk。整批未套用；改用實際標題完整行重新套用，PNG經固定Lua binary IO複製並逐byte驗證。後續每個hunk須來自真實讀取內容，不與無關placeholder混送；拒絕後先確認未寫入，避免重複新增檔案。資產圖示不當成功能通過證據。

### E421 — specs Fetch 的 Clone 不等於資源內容 Clone

Grok4.7Fast run-muyq8jrn-qjgwhl完成636秒／67calls／USD3.34591416，Primary獨立Lua presentation-v1測試揭露filtered_specs accessor複製Fetch而非DisclosedSummonerProjection，導致E0277 Default與E0308型別不合；未接受。修正須明確解參考clone。新manifest admission只使用codec，唯讀檢查又發現codec未驗catalog0非零clock／clock超過catalog總量，須在共享入口拒絕並保留舊rebase資料。明確repair packet已記錄，驗證後才接受。之前491秒取消批次無成果／費用未知，不能計為完成。

驗證helper的process.run要等程序terminal才寫log；本輪曾過早Get-Content尚不存在log，再次確認不應查未完成job的log當成失敗證據。後續v2加fail-fast及真正bridge summoner測試，未跑項明示not-run，不以check代替測試。

### E422 — Bootstrap/rebase 拒絕非法值也必須涵蓋 committed 更新

v2八項獨立Rust測試/check全部exit0。Primary再次檢查發現refresh_disclosed_summoner_projection只驗非負clock，仍可在committed facts寫入超過catalog總量／清空catalog卻保留clock。使用同一canonical_projected_clock在完整batch最後驗證，再一次替換resource；不逐筆驗證半成品，以保留equip與clock/reset facts的任意合法順序。既有真fixed-step負例加入overflow及unequipped非零clock，新測試驗兩種順序的clear/re-equip；v3新證據後才接受。

本輪唯讀又誤猜bridge/include/om_bridge.h、OmRuntimeTypes.h及scripts/ue_bridge_stage.lua，Windows literal *.rs也不展開。真正header已由inventory確認為Plugins/OmRuntime/Source/ThirdParty/OmBridge/include/om_bridge.h；後續只用rg --files已回傳路徑，或對已存在目錄加-g搜尋，不再照名稱記憶拼檔。

實際codegen outputs清單沒有OmWorldBridgeActor或om_bridge.h；它們屬維護來源，不能要求不存在的emitter來產生。下一批packet已澄清真實邊界，只有真正generated catalog/styles等才需要同步emitter。

### E423 — ABI／UE 大批次無交付時縮小到可驗證邊界

Grok4.7Fast run-muyr2wqo-ksep93在600秒由Primary取消；僅回報inventory，關鍵來源mtime未改，沒有可接受source/report，費用未知。bridge claim-before-kill；固定Lua確認exact PIDs61664／61312皆已不存在，再開始後續writer。沒有證據指認服務／認證錯誤，心跳也不能當實作進度。改同一verified4.7Fast、same thread低effort，僅Rust/C ABI+IPC5個檔案，C++消費端與owner/HUD分開接續，保留已接受85測試的資料基線。terminal文字Stopped by user是bridge泛用訊息，本次由Primary決定，非human停止要求。

### E424 — IPC 版本升級必須同步消費端版本斷言

ABI18／IPC6後，獨立驗證v1的bridge-all與client-input通過，client-presentation為27通過／1失敗：point_queue_rejects_version_three_before_admitting_input仍斷言版本5，實際6。Grok報告已指出此遺漏，Primary啟動測試前未先處理。修正版本斷言為6，保留真正read_envelope拒絕舊協定的負測試；完整增量改用新v2 evidence，保留v1失敗。往後先處理交付報告中已知的實際消費端不一致，再啟動驗證；不得因封包檔案範圍漏掉版本契約。

### E425 — 先查實際檔案清單再讀輸入介面

Primary誤查不存在的OmInputBridge.cpp／OmRuntimeSubsystem.cpp，rg回報不存在。實際消費端為OmRuntimeBridgeSubsystem.cpp與OmGenerated/OmWorldBridgeActor.cpp。已以rg --files取得現有路徑再讀；不得由Rust模組名猜Unreal檔名，也不把讀檔失敗當編譯錯誤。

### E426 — TOML slot decoder不能讓struct接受位置序列

Primary config adapter v1實際6測試中4通過／2失敗。serde untagged NamedSlots可把單元素陣列當位置struct並補預設，繞過legacy恰好2槽要求；改明確Visitor分map／sequence，sequence逐一檢查恰好兩項，map才交給deny_unknown_fields NamedSlots。另一失敗KeyNotString來自測試將既有u32-key team map直接轉TOML Value；改使用真正完整TOML文字輸入production ServerSetting decoder，不修改無關team格式。保留v1失敗，v2重新測試。

### E427 — 拷貝HUD時不能把異常冷卻當成ready

Primary審查Grok C++payload發現NaN／負數冷卻轉0後bEquipped只看identity，使malformed row看似ready；正式Rust已拒絕此資料，但UE消費端仍需一致失敗關閉。新增原始range／remaining／total有限與非負、remaining≤total、換算後range有限的equipped gate。保留安全顯示0但不可施放，測試新增NaN與大於total。Empty slots明確index0/1，constructor也初始化兩槽，避免第二空槽冒用0。編譯/native驗證未完成前不稱接受。

### E428 — 實際EngineChange後走維護refresh，不手改metadata

controls-summoner-payload-build-v1普通NoEngineChanges在249.58s實際FailedDueToEngineChange，包含NetCore等引擎產物；Rust／codegen／stage已成功，尚未通過C++編譯。保留UBT log與summary；trusted-handoff writer目前修改Rust，待其terminal再用固定Lua maintained --refresh-engine／no-cache，再普通NoEngineChanges驗證，不手改Engine/config/BuildId。正式workflow實際會cbindgen生成om_bridge.h，覆寫之前handwritten注解；已核對生成header ABI18／shape constants／two slots／summoner_slot均保留。來源權威是Rust repr(C)+維護cbindgen流程，不能以worker人工header註解誤稱此檔永遠handwritten。

### E429 — Struct move與空map正規化須在送驗前審查

Primary審查trusted-handoff交付即發現HeroSelectionSession::new在plan移入Self後才clone其loadouts；在測試前先clone到local再move，避免已知borrow-after-move。Lua原本將summoner_loadouts:null也正規化為空map，但真正Rust plan拒絕null map；改只容許缺省／空map，null仍可作兩槽陣列中的未裝備值。此為source review修正，尚無該錯誤compiler失敗紀錄，不假稱實測。新獨立驗證v1正在執行。

### E430 — 拒絕測試避免無關的Ok值Debug要求

trusted-handoff-validation-v1真compiler E0277：三個HeroSelectionSession::new(...).unwrap_err()要求Ok型別HeroSelectionSession實作Debug。改.err().expect以檢查真正error內容，不為測試改production session Debug contract。v1保留未選到任何tests的compiler失敗；v2重新跑全增量。另加Lua負例拒絕top-level null map，仍容許兩槽陣列中的null。

### E431 — 完整TOML測試必須序列化真正的外層Setting

trusted-handoff-validation-v2選角／plan／backend-summoner通過，backend-plan1test失敗：把[server]字串接到ServerSetting序列化結果前面，巢狀AUTHENTICATED_TEAM_BINDINGS表落在root而非server，真preflight正確拒絕缺少human teams。改序列化完整Setting，產生真正[server.AUTH...]層級。審查同時移除尚未跑到的整份text禁止null斷言；MATCH_ROLE_PLAN_JSON內合法空槽本來可含JSON null。改讀真正TOML table逐槽比對存在／省略，避免誤判內嵌JSON。僅修test fixture，production validation保持嚴格，保留v2失敗。

### E432 — 驗證launcher前先明確編譯當前設定工具

trusted-handoff-validation-v3的14selection＋7plan＋6backend-summoner＋1backend-plan測試及backend/content-tests compile checks均通過；Lua真nonempty recipe prepare正確拒絕舊release moba-config.exe不識summoner_loadouts。moba_config_command依專案規則只用prebuilt、不自動build，Primary helper前版遺漏此先決條件。v4將固定Lua cargo build moba-config compiled-content-only debug/release加入Lua測試之前；新增真兩次prepare及--config讀回每槽／空槽資料的測試，避免只用舊exe或pure exporter背書。保留v3實際錯誤，不更改工具的no-auto-build政策，也不啟動對局。

### E433 — 介面開啟取消原因與測試 activation token 必須一致

Primary 審查尚未合併的共用施法候選檔，發現既有 InterfacePanel enum 未列入 IsKnownCancel，造成商店開啟要求無法立即取消預覽。候選模型已補取消分支及原 token 延遲放開不提交的斷言。同時修正召喚師拒絕矩陣測試：每輪重新取當輪 token，避免反覆用第一輪過期 token 而漏測真正當輪 release。這是 source review 發現，尚未編譯／執行，不宣稱已驗收。

### E434 — Lua 紀錄工具字串串接必須使用 ..

Primary 狀態紀錄腳本錯用 + 串接字串，已寫入 state 與 E433 後於 review append 失敗。修正為 ..，並使 E433 寫入冪等，避免重跑重複紀錄。僅紀錄工具失敗，驗證證據未受影響。

### E435 — PCH response 路徑須按 compiler working directory 解讀

Primary 讀取 rsp 的 ../Intermediate 後誤猜為專案 Intermediate，Get-Item 回報不存在；實際以 rg --files 找到 UE Engine/Intermediate 的 shared PCH。這是唯讀診斷路徑錯誤，未修改 Engine 或產物。後續先列檔並核對實際 cwd，不直接從 rsp 相對路徑猜目的地。

### E436 — Responsive HUD 估寬不得漏區塊，事件測試不得手動代替路由

Primary 審查 run-muysuk99-mpaivs 候選，CombatNatural 漏英雄資訊寬度及未縮 padding；技能卡縮小但文字高度不縮，造成小 logical viewport 裁切風險。新測試只檢 slot0，並手動設 camera guard 後直接呼叫 zoom，不能證明實際 wheel wiring；optional capture directory 未使用且 widget parent ownership 不安全。候選尚未編譯／合併，已同 thread 委派 bounded repair，要求完整佈局、真事件與 production PNG。沒有虛報 native 失敗或成功。

### E437 — 低 CPU 與長時間無 action 完成不等於可斷言 compiler 掛死

payload refresh v2 第一個 compiler 經十五分鐘觀察後自然完成 Module.OmRuntime.cpp；原定有界退休腳本在 exact lifetime guard 發現原 compiler 已不存在而中止，未執行 stop，也未替換成下一個 PID。UBT 與下一個 compile 持續。保存 first-compile-completed log 及 natural-completion receipt。不能僅據 Executive wait／低 CPU 斷言 root cause 或假報取消；此輪繼續等待實際 build terminal。

### E438 — Slate 外框 padding 必須納入 fit 判定與佈局快取

Primary 審查第一輪 HUD repair：minimap inner size=MiniPx、Border padding4，outer desired=MiniPx+8，但 Measure 以 >MiniPx+2 拒絕，所有候選必敗；fallback 又與小地圖同底部重疊。另逐 frame 巢狀反覆 invalidate/prepass 與空白64高 seam 造成效能／裁切風險。未合併未實跑；同 thread bounded repair v2 要求完整 outer geometry、compact label/tooltip、零高未接線 seam、changed-input-only fitting 及小視窗真事件／截圖驗證。

### E439 — 狀態紀錄不得臆測不存在的 JSON key

Primary 紀錄工具誤猜 summoner_rust_abi key，於寫檔前 nil-index 中止；改成明確獨立 summoner_c_header 區塊。建置／測試與證據未受影響。後續修改既有 JSON 區塊先核對 key，勿依摘要命名推測。

### E440 — 小視窗空白區測試須避開實際 HUD

響應式 HUD 候選 fixture 把視窗35%位置固定當空白；400×300 的 compact combat cluster 可能合法覆蓋此處。Primary 改測上方左側5%世界區域，保留實際 hit-test assertion，未改生產遮擋判斷。候選尚待編譯與原生驗證。一次多檔 patch 因第二檔上下文不匹配被原子拒絕，確認未寫入後以精確上下文重做；不可假設部分已套用。

### E441 — Collapsed 元件的 tooltip 無法讓玩家查看

Primary 審查響應式候選發現 buff／GoldLives 文字收進 Collapsed 元件，但 tooltip 仍掛在不可命中的元件。保留 compact layout，將實際 stats／buff／gold-lives 的即時 tooltip 接到可見 portrait border。需隨新 source 建置與渲染驗證；不以 hidden widget tooltip 當可用資訊。

### E442 — Rust native module 是 native.rs 而非 native/mod.rs

Primary 在後續攻擊接線盤點時，已發現一部分目錄仍臆測 runtime_command.rs 與 native/mod.rs，rg 報不存在。先 rg --files 後確認實際入口 runtime/native.rs、runtime/command_transport.rs、native/comp/phys.rs、native/comp/collision_index.rs。未修改 source／未執行失敗 build；未知路徑必先列檔，不能從模組慣例推測。

### E443 — 藝術 provenance 與 Rust 元件路徑先列檔

Primary 把 summoner-artwork-provenance-v1.json 誤猜在 .ai-collab，實際已隨資產保存在 Content/Omoba/UI；另先前 property.rs／update_searcher_tick.rs 的猜測也報不存在。讀檔失敗未改 source；改用 rg --files／在已知目錄內 rg -n 定位。此類失誤再次發生，後續未經列檔確認的路徑不得傳入讀檔或精確搜尋。

### E444 — 新 Scenario 必須歸屬有正文的 Requirement

Primary 補附加快捷鍵／小地圖偏好／隊友頭像規格時，先把新 Scenario 插在原 Requirement 標題與正文之間，另立標題承接舊正文，造成兩個空 Requirement。實際 openspec validate complete-moba-controls-and-presentation --strict exit1：有效鏡頭滾輪縮放、hover／範圍與操作回饋 missing requirement text。已將新增契約分成具 SHALL 正文及 Scenario 的獨立 Requirement，保留原 Requirement 名稱與原 Scenario；相同嚴格驗證重跑 exit0 valid。規格驗證不當成功能驗收。

### E445 — 響應式 fit 不應無條件吃滿剩餘螢幕寬度

Primary 後續檢查候選 RefreshClusterScales 發現 CombatW 固定取 viewport 減 minimap／margins 的全部寬度，1920／2560 仍會形成過寬 painted combat border，內部空白也被 hit-test 視作互動區。應以實際 HUD scale 的 preferred column width 與 available budget 共同限制 side／stacked，保留小視窗 fit 和 explicit no-fit。Grok D/F writer 尚執行，未交錯修改其 source；待 terminal 後審查最終 source、補真正寬度 assertion 及實際 PNG。詳 responsive-hud-final-review-notes.md；目前只是待修缺陷，未宣稱驗收。

### E446 — HUD 地點施法不可依賴不存在的物理鍵鬆開

Grok owner 卡片路徑沒有游標 sample／release，但直接沿用保存的Quick／Indicator模式：Quick拒絕，Indicator無法完成。Primary新增共用model的interface來源：targeted action改成world左鍵確認，不讀UI座標、不建立physical hold；自用仍立即。補三模式純model序列（不宣稱權威ACK），owner只傳來源。另完成E445 preferred640×HudScale寬度上限與寬螢幕底部空白hit-test斷言。正式build／native驗證中，不能以舊42報告背書。

### E447 — 查找再次使用未盤點路徑及Windows shell glob

Primary唯讀調查誤猜runtime/stepper.rs，且將native/*.rs直接傳給rg在PowerShell不展開，出現os2／123。已用rg --files與rg目錄 --glob *.rs找到真正seam，無source寫入／程序變更。後續搜尋先發現檔名、只把glob交rg。額外釐清TeamVisibilityRuntime實際已在ECS，TeamVisibilityState是其內部資料；消費者應重用committed視野，不能因先前簡寫另建視野系統。詳combat-visibility-consumer-review.md。

### E448 — 候選HUD與D/F測試首次實際C++編譯失敗

controls-summoner-owner-build-v1實際UBT379.25s OtherCompilationError：OmCommandBarWidget的Slot遮蔽UWidget成員(C4458)，AbilityRow前多一個Slate結尾括號引發後續型別／語法連鎖；OmActionOwnerTests兩處調用private HasOwnedViewportFocus(C2248)。Primary改ItemState、修階層，並改公開Slate.GetUserFocusedWidget驗實際focus，不放寬private API或刪測試。已保留build.log／UBT-Log／summary／errors；v2一般NoEngineChanges建置中。未拿source review當編譯通過，亦無Engine refresh或權威對局。

### E449 — 首批49 native有三項真實失敗，不能只看Editor exit0

controls-summoner-owner-native-v1的Lua嚴格JSON拒絕：ControlProfileSummonerCastModes重綁G與catalog既有action衝突，改用實際空閒U，保留衝突規則；ResponsiveHudLayout在GameViewport window有title／border時TakeScreenshot offset達805×664但backbuffer800×600，改共用EditorGameViewportFixture為無title／零layoutborder，保持實際client geometry與RHI一致，不裁切PNG假裝通過；SummonerOwnerCards的錯owner先ClearHeroSnapshot，既有phase guard應Keyboard input is blocked，修預期守門順序並保留inputId0／無preview。原49結果全部保留；v3build後重新跑scope與八張PNG。Primary曾在修補草稿誤猜HasOwnedHeroSnapshot，寫完立即查header移除，未送入編譯；此類API必須先查宣告。

### E450 — 原生Automation JSON含UTF-8 BOM

Primary新triage helper直接json.read原生index.json在byte1拒絕；正式native gate原本已正確處理。改僅移除開頭UTF-8 BOM再json.decode，不鬆綁JSON或測試結果驗證；第一次失敗在任何寫檔前。

### E451 — 實際視窗排列與尺寸仍需驗證

controls-summoner-owner-native-v2仍有ResponsiveHudLayout／SummonerOwnerCards兩項Fail。400×300 capture得到406×300 request，renderer400×300因此拒絕；補EditorGameViewportFixture FixedSize／MinWidth1／MinHeight1並記Window/HUD/viewport真幾何，未縮裁圖片掩蓋問題。CardWindow AddWindow(false)並未可見，force redraw不能保證有效排列，改true並記card／ability／window大小。v4普通build24.90s PASS；native-v3正在跑，尺寸原因與結果依新log確認，未先宣稱完成。

### E452 — 世界滾輪可由SViewport正常處理，非所有Handled都等於HUD攔截

controls-summoner-owner-native-v3實際48/49通過；五尺寸Window/HUD/backbuffer完全一致，PNG已產出。剩ResponsiveHudLayout錯把world RouteMouseWheel返回Handled當HUD吃事件；SViewport合法處理也可Handled。改驗實際path.ContainsWidget自有viewport，保留HUD hit-test與zoom/component移動斷言。Primary開1920與400 PNG發現D/F名稱壓圖示，移到圖示下方並新增名稱／圖示不重疊、兩個卡片皆不高於QWER斷言，不改功能狀態。v5普通build／native待驗；舊PNG不當新視覺通過。

### E453 — 純策略候選的const轉換／測試遮蔽與空間模組路徑

Primary review修正const函式的非const u64::from為無損as轉換，測試區域preferences變數遮蔽helper改直接建立OwnerCombatPreferences。新19測試通過。讀取不存在runtime/spatial失敗，改先rg --files確認實際native/spatial；後續不得猜模組路徑。

### E454 — all-targets發現舊example漏新設定欄位

foundation-validation-v1純策略19測試通過，但phase6_differential建構TeamProjectorConfig漏fog_sample_interval_ticks導致E0063。example以..TeamProjectorConfig::default()保留既有顯式設定並繼承共用預設，v2兩項全過。保留v1失敗log，不將lib測試通過當all-targets通過。state為單行大JSON，避免Get-Content整行；只用Lua查必要欄位。

### E455 — 驗證helper名稱亦必須先查實際檔案

唯讀Get-Content猜verify-summoner-world.lua不存在；rg --files --glob *.lua確認實際verify-summoner-world-v1.lua/v2.lua。後續找helper先縮小副檔名並排除target，避免無關cache大量輸出；不把讀檔失敗當功能失敗。

E455補充：唯讀搜尋再次猜OmNativePlayerController.cpp不存在；rg --files確認OmPlayerController.cpp，後續以實際已發現完整路徑。不得把慣用類名當真實檔名。

### E456 — 共用選敵測試模組匯入與Option API編譯失敗

combat-native-consumers-validation-v1實際cargo編譯失敗、尚未執行新測試。BuffStore實際由runtime re-export而非native/comp；cfg(test)子模組需明確匯入OwnerCombatPreferences；Specs World::try_fetch回傳Option，應用is_none/is_some而非is_err/is_ok。Primary依實際compiler五處錯誤修正，v1原始log保留，v2驗證執行中。不將Grok靜態檢查當編譯通過。

### E457 — 直接run_now的ECS fixture需完整wrapper資源

consumer-validation-v2編譯通過但20個測試皆因SysMetrics缺失panic；v3補SysMetrics後仍20個因TickProfile缺失panic。實際comp/ecs.rs Job::SystemData同時ReadExpect兩者，應完整讀wrapper及既有fixture再一次建齊資源，避免逐漏重跑。Primary補齊兩項，v4新20／共用19／spatial2／command8／hero2／stop10共61測試及all-targets全過；v2/v3原始失敗log保留。未改production初始化來掩蓋fixture缺漏。

### E458 — 游標攻擊移動偏好不可隱性關閉閒置普攻

Primary source review確認原策略在Cursor+Idle+無command_point時回InvalidOrigin；原19測試甚至鎖定此行為，會使未來開啟游標攻擊移動時連帶停止autoAttack=true的閒置普攻，違反偏好獨立性。改只在AttackMove+Cursor要求command_point；Idle固定以英雄為排序原點，明確AttackTarget仍獨立。保留AttackMove缺原點拒絕測試，將錯誤Idle預期改為有效目標，新增純策略及實際hero_tick回歸。v5待驗，尚未啟用UI／wire。

### E459 — 跨crate讀檔仍須使用已確認路徑

唯讀搜尋誤寫omoba-core/src/runtime/input_bridge.rs，實際檔案為omoba-client-runtime/src/input_bridge.rs；rg --files重新確認後改正。此檔案路徑已存在設計紀錄，後續先查紀錄或檔案清單，不依名稱推測crate歸屬。未影響產品原始碼或既有驗證結果。

E459補充：同批讀取再次誤把comp/ecs.rs放在runtime/native下，實際為omoba-core/src/comp/ecs.rs；game_processor.rs則在runtime/native下，沒有comp/game_processor.rs。後續搜尋入口先限定已存在目錄的rg --files，再讀取實際回傳路徑。

E459補充：ward設計實際為.ai-collab/moba-ward-authority-design.md，不是moba-ward-design.md。必須分開完成檔案清單搜尋與下一次讀檔，不能在同一命令中先搜尋卻仍讀取預猜名稱。

E459補充：content唯讀搜尋誤帶scripts/lua_data/init.lua；該檔不存在。查content入口一律先rg --files，之後才將確切回傳路徑送給rg或Get-Content，不能為了批次讀取摻入尚未確認的名稱。

### E460 — 精準點擊不能以小地圖owner對隊伍代替完整戰鬥合法性

Primary審查Grok已交付source確認PublishedDirectTargets只從小地圖marker取得隊伍；marker源自public scoreboard的owner映射，無owner小兵／塔及HostileNeutral無法選取，並非完整Faction::is_hostile_to。重疊目標只保留TMap首項，缺固定identity平手排序。新fixture刻意將owner0 creep視為無敵方target，不能證明此契約完整。後續須補實際disclosed faction/team/attackability→bridge→UE並驗非1/2玩家／ownerless／neutral／locked structure，不把四個新測試存在當功能完成。另目前profile只有單chord，連續Thumb1改Thumb2不等於同時主／副鍵；按住更新借用Windows鍵盤repeat及假想未來UInputSettings欄位，需改明確跨平台輸入設定與有界scheduler。尚未編譯／執行新測試，沒有假ACK或新對局。

### E461 — 接續摘要中的相對native路徑不能直接拼成src/native

本輪唯讀rg誤查omoba-core/src/native/moba_match.rs及comp/unit.rs，實際位置為src/runtime/native。沒有修改產品或中斷建置。已分開執行rg --files確認實際位置，並修正下一批Grok任務的完整路徑。摘要中的簡寫也必須先做檔案探索，不能當已驗證絕對路徑使用；避免再以錯誤路徑反覆搜尋。

### E462 — Unreal forward declaration的class／struct必須與原型一致

controls-precise-owner-build-v1實際MSVC SARIF報C4099：OmPlayerActionOwnerComponent.h:23以struct forward-declare FModifierKeysState，但UE5.8 GenericApplication.h:74定義為class；warnings-as-errors導致Module.OmRuntime.1.cpp失敗。已保留原SARIF，等待建置程序結束後將該forward declaration改為class，保留原signature與行為，再執行新的正式build。不能忽略警告或改Engine定義。此輪尚無新native驗證，整體未接受。

### E463 — 反射參數收集器的TArray allocator也是型別契約

同輪正式build在OmPlayerActionOwnerComponent.cpp:673與698報C2664：既有CollectParms接受TArray<FProperty*,TInlineAllocator<16>>&，新增Move與AttackTarget resolver卻傳TInlineAllocator<8>。元素數量足夠不代表不同allocator型別可綁到同一reference。已保留原SARIF；待build terminal後將兩處容器統一為既有16，不改反射參數個數、順序或安全檢查，再正式重建。Primary source審查應同時檢查helper完整型別，不能只數六／五個反射參數。

E462／E463修正接續：第一輪已實際terminal exit1，Primary才改上述三行。controls-precise-owner-build-v2普通NoEngineChanges／NoUBA／nocache已啟動，無Engine refresh，尚未宣稱通過。原失敗log與SARIF保留。

### E464 — 真native驗證揭露fixture衝突／焦點與optional-unbound假設

controls-precise-owner-build-v2已正式通過；controls-precise-owner-native-v1實際53項中50Success、3Fail、0notRun、0warning-success，commandlet exit0但嚴格Lua gate正確exit1。失敗為PreciseWorldCommand.Rebind還原WorldCommand→RMB時MoveOnly仍占RMB；Route的disabled-repeat hold、UI button click、shop-open hold cancellation與未BeginPlay的Blueprintable native owner input四斷言；ControlRuntimePersistence假設所有可見Chord都非未綁定，與新增optional MoveOnly／AttackOnly預設unbound衝突。保留完整JSON/log/PNG及failure-audit.json。下一批Grok須逐一判明fixture與產品責任，修實際生命周期／事件路徑及衝突順序，optional-unbound只可對catalog明確optional項放寬；不能刪測試、略過失敗或造假Connected/ACK。E460後續陣營／repeat／主副鍵仍未完成。

### E465 — 新可選操作不能把英文catalog Purpose直接當繁中設定名稱

Primary已檢視native-v1八張原始production-widget PNG（測試仍50／3失敗，無功能接受）。設定畫面新增兩列直接顯示Optional attack-only click／Optional move-only click及Unbound by default，與其餘繁中操作名稱不一致。後續須在共用顯示名稱／tooltip映射補「只攻擊游標目標」「只移動到游標位置」與清楚的可選未綁定提示；不能因新增catalog列就把內部Purpose當最終產品文字。Grok目前修E464時Primary不改其來源，待terminal審查再補。400×300雖無幾何重疊，字體仍小，不能當完整DPI可讀性驗收。黑背景為斷線widget fixture，不是完整地圖呈現。

E464／E465修正進度：Grok repair已terminal，Primary審查後補繁中設定名稱／tooltip；普通build-v3通過。原先Game階段的pointer observer收不到PreGame消耗的右鍵，已改為前置PreGame純觀察器。native-v2尚未完成，不先宣告功能接受。

### E466 — 回歸測試不能讓後續MouseMove掩蓋被消耗的MouseDown

審查E464候選時發現測試在Tick前補MouseMove，可能即使observer錯過右鍵仍補到游標樣本。Primary已在任何補發Move之前加入click-only Tick(0)，要求hold保留且submit attempts仍恰好一次；後續真實Move／Tick驗證仍保留。這是來源審查發現的測試缺口，非新增執行失敗。build-v3通過，等待native-v2實際執行。

E462／E463驗證結案：普通controls-precise-owner-build-v2通過，controls-precise-owner-native-v2 exact53 Success／0warning／0failed／0notRun。反射class與allocator型別修正接受；E460功能缺口仍待後續實作，不以此次測試通過勾選整個右鍵或主／副鍵父項。

E464／E465／E466當前驗證：controls-precise-owner-build-v3正式普通建置通過；controls-precise-owner-native-v2 exact53 Success，零warning／failed／notRun，八張production captures已核對。click-only tick與真按鈕capture／click等斷言保留且實際通過。未把斷線拒絕驗證擴大為合法connected command或權威ACK接受。

### E467 — C標頭驗證報告不可永遠寫死ABI18

Primary在準備metadata後續驗證時發現verify-summoner-c-header.lua的summary與成功文字固定填18。已改成執行時嚴格讀取唯一維護中的Rust OM_ABI_VERSION宣告，產生fresh evidence wrapper，以static_assert要求實際C標頭與Rust版本一致，再保留原maintained smoke完整size／offset／type檢查。這是驗收工具來源審查缺口，不是新增產品執行失敗；目前metadata writer仍執行，未讀取其變動source或編譯，待terminal後真正執行工具再接受。若版本宣告不符合明確literal格式即拒絕，不猜版本。

### E468 — Windows binary-read／text-write不能直接混用於整份MD重寫

Primary交接摘要更新使用path.read(file,true)後以預設text-mode path.write重寫，Windows把既有CRLF再次轉成CRCRLF，git diff --check因此報交接MD trailing whitespace。已只針對該MD把CR+LF正規化為LF，使用path.write第四參數binary=true落地，保留全部文字與歷史內容。往後binary-read的原文重寫必須明確保留／正規化換行並binary-write，不把第三參數overwrite誤當binary。這是Primary文件工具錯誤，不是產品錯誤；接著重跑diff-check。

### E469 — metadata新測試缺少所在module的型別import

獨立disclosed-target-core-v1實際cargo exit101，七個診斷均由兩個缺失import產生：filtered_specs::canonical_json_tests的BTreeMap（六處）及combat_acquisition新gate測試的Searcher。原始log與失敗summary保留；第二filter未執行，不宣稱測試通過。Primary在所有worker／驗證均terminal後補兩個明確module import，不更動產品語意，接著fresh run-id重驗。
### E470 — 公開目標 metadata 的存活、對局階段與來源失效語意不一致

獨立原始碼審查確認三項合約缺口，尚未宣稱修復或執行新的回歸案例：(1) Fixed64 正數小數 HP 在兩條 native extractor 與 filtered converter 被截成 i32 的 0，導致 InputBridge raw-health admission 與 Unreal 目標判定不同；(2) 非建築的公開 attackability 未套用 MobaMatch 階段，authority acquisition 亦未共用 damage boundary 的 locked/returning 判定；(3) Unreal selection 未比對目前設定玩家與 relation source，且 control-only source removal 未清除其他 row 的來源相對關係。以同一來源政策、精確生命值及來源 identity retirement 修復，禁止假血量、強制 Connected 或偽造 ACK。修復交由同一 Grok 4.7 Fast thread，保留既有失敗證據。

修復前實際證據：disclosed-target-core-v2 的 3+2 測試通過；disclosed-target-rust-abi-v1 的 bridge91、input17、presentation28 測試及三項 all-targets check 通過。這些檢查未涵蓋上述三個缺口，因此不代表 metadata 功能驗收完成；未執行 authority game，正式 staged DLL 仍為上一輪 ABI18。

### E471 — 後續 visual audit 仍有未 discover 的 registry 路徑

Primary 對不存在的 OmGenerated/Private/OmContentRegistry.cpp 執行唯讀 rg，實際 exit1／os error2；沒有產品修改。這違反既有 E459／E461 的 discover-before-read 防錯規則。已立即改由 rg --files OmRuntime/Source 的 *Registry*／*Content* inventory 定位真實檔案，後續 packet 必須引用 inventory 原樣路徑，不能根據命名慣例猜測 registry。此錯誤不影響正在獨立執行的 Grok E470 修復，亦不當作產品缺陷或通過證據。
### E472 — 大型舊 thread 的三合一修復未交付，改小批次

Primary 自主取消 Grok repair run-muz4bm6p-zimh5p，bridge/follower 均確認 terminal cancelled，實際 duration14m39s，成本／tokens 未提供。兩個 tracked process64280／64328 已由唯讀 OS 查詢確認不存在。取消前所列十二個核心／bridge／UE修復路徑的 LastWriteTime 均早於本輪啟動13:50:56，且沒有交付報告；不宣稱已修任何 E470 缺口。不能由低CPU或沒有 progress 更新推論具體網路／模型故障。這是主代理的進度管理決策，不是使用者取消整體目標；bridge 通用 Stopped by user 字串不能改寫這個事實。

避免重複：原 same-thread repair 已嘗試；後續改 fresh、較小的精確血量批次，只讀必要檔案及 E470 摘要，不反覆讀完整歷史 error register／長 thread。其餘 attackability 與 source retirement 另外接續。保留原 candidate ABI19、先前失敗／通過證據與所有 dirty changes，未執行新權威場次。

### E473 — 新 Lua 音源入口缺少相對 bootstrap search path

Primary 實際執行 tools/lua/lua.exe scripts/author_moba_audio.lua exit1，require(_bootstrap) 在初始化前找不到模組，沒有產生音源。已採與 build_ue_moba／ue_apply_asset_recipe 相同的 debug.getinfo source directory 加 package.path，不硬編 D:/omoba 或依賴呼叫者 CWD；fresh 執行再驗真 PCM 檔案。錯誤與產品音效播放無關，尚未宣稱 Unreal audio 成功。

E472 接續：fresh 精確血量 run-muz4vz3d-omzope 於7m1s由 Primary 取消；follower true terminal、tracked74584／64448不存在，六條已列來源時間仍早於啟動、無報告／未修復宣告，成本未知。正常 bridge --help 已確認支援 --effort low；下一次用相同 grok-4.7-build-fast 與同一小 packet 明確設定 low，避免再依賴未明示 effort。這是排程策略修正，沒有足夠證據指認模型／網路故障；未執行 games。
### E474 — 精確血量驗證：新測試 trait import 與舊16-byte property fixture

實際 precise-health-core-v1 cargo exit101：snapshot::disclosed_relation_tests 漏 specs::Builder，EntityBuilder.with 不可用。實際 precise-health-rust-abi-v1 bridge91passed/1failed/1ignored：disclosed_motion_from_ipc_positions_reaches_animation_without_private_commands 仍寫16-byte property，現行正式40-byte codec拒絕後沒有合法血量，斷言得到 Stand 而預期 Walk。兩組原log／failed summary保留，後續 checks未執行不宣稱通過。

Primary 等兩組程序 terminal 後補 trait import；動畫fixture改用同 module 的 raw_property，該 helper 改走正式 encode_disclosed_property，避免自行拼接過時寬度。另在新 filtered-to-C-frame 回歸補短16-byte拒絕案例；不放寬產品codec／存活檢查。E469同類 import 缺失再次發生，後續 packet要明寫 actual module trait imports／fixtures使用正式encoder。報告 lowest living 的描述亦更正為最低 eligible identity 後獨立算alive，保留既有政策。

E474 重驗：precise-health-core-v2 的3+2+1全通過；precise-health-rust-abi-v2 bridge92／input17／presentation28與三項all-targets全通過。精確血量部分接受為 scoped Rust 路徑，ABI19尚未正式UE stage；其餘E470與connected操作仍待完成，不宣稱完整驗收。
### E475 — 叢林攻擊判定 fixture 越過子模組私有欄位

current-match-native-v1 實際 cargo exit101/E0451，父模組測試helper直接初始化 jungle::MobaJungleCamp 的 target／attack_remaining／lethal_pending／killer_player 私有欄位。首 filter 未編譯完成，後續未跑；原 failed summary/log/errors.md 保留。Primary 在 terminal 後於 jungle 模組提供窄 pub(super) new(definition)，正式 setup 與測試共用同一完整初始狀態；parent fixture只設定已公開entity／returning，不暴露私有AI狀態。初始值與既有setup逐欄一致，不改AI／balance／phase。後續 fixture 必須先看定義所在module與現有建構介面，不能從 pub struct 推論欄位公開；fresh重驗後才接受。
### E476 — 權威對局攻擊性測試未提交來源隊伍可見性

current-match-native-v2 實際 `combat_acquisition` 25 tests、23 passed、2 failed。`current_match_attackability_ordinary_unit_playing_and_warmup`（當時 combat_acquisition.rs:1277）與 `current_match_attackability_structure_and_jungle`（當時 :1321）在 Playing 預期 consume `(true, true, true, true)`，實際全是 `(false, false, false, false)`。log：`.ai-collab/current-match-native-v2/combat_acquisition.log`。gate precedence 通過，因為該例走 filtered disclosure。

原因是測試把真實 `MobaMatch` 插進 legacy fixture 後，`DisclosureContext::resolve` 變成 authority。authority 要求支援的來源隊伍，而且目標必須在該隊已提交的 `latest_read_view` 集合裡。兩例沒有 owner、沒有 `commit`，所以選擇在攻擊性政策之前就被可見性拒絕。傷害與 publication 斷言已通過，產品契約沒有缺陷。`authority_hidden_and_missing_commit_fail_closed` 的隱藏與缺提交拒絕保持不變。

修正只補測試：來源 `owner(4)`、隊伍 1，並只 `commit` 本例目標。不把全世界標成可見，不改成 filtered 來繞過 authority。後續權威對局的正向取得測試必須使用真實擁有者與實際提交的目標可見性；缺提交仍應失敗。本次未重跑，驗證仍 pending。
### E477 — 移動來源後的 AttackMove 測試仍使用原點目的地

current-match-native-v3 實際25 tests／24 passed／1 failed：structure_and_jungle 第1334行，consume 得到(true,true,true,false)。E476 正確視野修復後三個消費者均通過；第四個 AttackMove 使用 away_from(0)=80，而來源已移至90，落在正式到達容差內，所以指令先正常 advance，並非攻擊政策拒絕。Primary 將新 consume helper 的目的地改成依當前來源 Pos 原始固定點 x 加80；仍保留全部四個正向斷言，未改產品到達判定或攻擊規則。後續會移動來源的 fixture 必須從當前位置生成遠目的地，避免把到達結束誤判為取得失敗。原v3失敗log保留，fresh重驗 pending。

Current-match shared attackability independently accepted in Rust scope: current-match-native-v4 67 tests and core all-targets PASS; current-match-health-regression-v1 3+2+1 PASS. E475/E476/E477 failed evidence retained. ABI19 Unreal formal stage and source retirement part3 pending; no authority games, positive network ACK or full controls acceptance claimed.
### E478 — Primary 再次先猜檔名才完成搜尋

兩次唯讀盤點使用未驗證路徑：OmRuntime/Private/OmMinimapWidget.cpp、scripts/ue_build_policy.lua，rg 實際回報不存在。前者由完成的 rg --files 取得真正 Public/SOmMinimap.h／Private/SOmMinimap.cpp，後續正確讀取；後者的必要建置旗標已直接在已知 scripts/build_ue_moba.lua:195–199 取得，沒有需要新增或假造 policy file。無產品修改、無建置失敗，獨立詳情另保留 .ai-collab/minimap-discovery-error.md。

E459／E461／E471 同類問題仍復發。預防規則：不把內容查詢與尚未完成的 filename discovery 放同一次呼叫；已知入口先查其自身，新增檔案必須先有 rg --files 的實際輸出，下一次呼叫才讀。查詢無命中與檔案不存在要分開處理，不將 rg exit1 無命中自動當產品失敗。
### E479 — 待派工 packet 錯稱 client runtime 無 lib facade

Primary 在下一批啟動前重新盤點：實際 omoba-client-runtime/src/lib.rs 存在，公開 presentation_bridge/config/replica_host 等；main.rs 首行即 use omoba_client_runtime。先前 prepared owned-presentation-fixture packet 的「has no lib facade」不正確，已改為直接復用既有library的gated fixture binary；另更正 ready_envelope／ready_envelope_for_start 為module functions，非PresentationHub methods。尚未派工／實作該批，因此未產生重複facade或模組。預防：每批啟動前先用實際檔案清單與入口確認關鍵架構前提，handoff記憶與prepared packet不是現況證據。
### E480 — metadata 正式建置被程序中斷，未完成22動作

controls-target-metadata-build-v1 實際 summary exit1、refresh=false、disable_action_cache=true。build.log 記錄 Lua host invalid response／exit1073807364；UBT原log停在17/22 OmEditorAutomationTests.cpp，已完成 OmRuntime／OmGenerated DLL 及多個測試編譯，沒有 C++ error 或 UBT Succeeded。原 shell session35062已不可恢復；Primary 新session唯讀核對 lua／compiler／dotnet等程序與檔案，未將工具 Unknown process id 當成成功。未有證據指認模型、編譯器或原碼故障；中斷原因不推論為使用者取消整體任務。

原summary/build.log/UBT-Log.txt保留。以fresh v2普通NoEngineChanges／NoUBA／nocache正式流程續建，復用合法增量產物但不跳過完整成功gate；只有實際FailedDueToEngineChange才允許refresh。後續驗收必須先有新terminal與stage/header成功；程序中斷不能假稱source已驗證。
### E481 — screenshot comparer 對所有視窗假設相同像素尺寸

controls-target-metadata-native-v1 實際exact54 Success／0warning／0failed／0notRun；compare-controls-captures.lua 首次卻在line12因capture dimensions differ退出。新HUD/shop1600x902、settings800x902；舊baseline2800x1578／1400x1578。五張responsive固定尺寸皆維持宣告值。未量測OS DPI，不能僅依尺寸比例診斷原因，也不能宣稱SHA相同。

Primary 修正通用比較規則：同名／八張／成功report仍嚴格；fixed responsive檔名尺寸必須精確吻合，新舊adaptive尺寸皆記錄並標RESIZED REVIEW。任何變圖或變尺寸都需重新實際目視，不縮放圖偽造SHA。已重跑compare成功，八張新PNG全用view_image檢查並寫capture-review.json；400x300文字仍小，黑背景非完整地圖，完整視覺／DPI驗收pending。此為證據工具假設錯誤，未改產品或繞過native斷言。

E470三項修復完成目前範圍驗證：current-match-native-v4 67項＋core check；relation-source-core-v1 7項；relation-source-rust-abi-v1 bridge93／input17／presentation28＋3checks；普通nocache controls-target-metadata-build-v2與ABI19 C++ header smoke通過；controls-target-metadata-native-v1 exact54 Success／0warning，八張新PNG已逐張重新目視，adaptive尺寸差異已記錄。正式stage artifact-copy consistency已驗證。仍不代表positive owned操作／authority ACK／全計畫完成，6/44與10/10不變。

### E482 — Primary 搜尋起點仍猜測不存在的 scripts/lib

準備下一階段驗證時，唯讀 rg --files scripts/lib 回報目錄不存在。未讀取或修改產品、未啟動建置。改從已確認的 scripts 根目錄完成檔案清單；工具模組位置必須由實際 _bootstrap.lua 或全根目錄清單取得。E478 的規則也適用搜尋起點，不能只驗證末端檔名而猜中間目錄。

### E483 — renderer fixture 首次 lib test 在執行前就編譯失敗

`.ai-collab/renderer-fixture-rust-v1/summary.json` 記錄 library job exit 101，`simulations_executed` 0，後續 check/build 未跑。`library.log` 保留未改。`omoba-client-runtime` lib test 九個 error、零個測試執行：`renderer_fixture.rs` E0308（`parse_args` 回傳 `&str` 而非 `String`）、E0277（私有 `Parsed` 沒有 `Debug`，`unwrap_err` 無法使用）、兩次 E0382（`moba_hud.unwrap()` 移走後又把同一個 snapshot 傳給 `admitted`）、五次 E0502（`reject(&mut fixture, json!(fixture.run_id))` 同時借用）。同次還有新的 `unused_mut`（`finish_start` 的 `hub`）。`input_bridge.rs` 的既有 unused import 不是這次 fixture 引入的，未一併清理。

預防：fixture 測試在送驗前要先看實際 compiler 輸出，不能把未執行的測試名當成通過。`Result<Private, String>` 的錯誤字串要用 `String`；`unwrap_err` 的 Ok 型別要有 `Debug`，或不要把只給內部用的 enum 留在公開函式。同一個 statement 不要同時 `&mut` 值和在 `json!` 裡借用它的欄位，先 clone 再呼叫。`Option::unwrap` 會移走 HUD，後續斷言要改看 `as_ref`。清理檔寫入失敗必須是 nonzero，而且不能蓋掉原本的失敗原因。操作期限和最多 2 秒的 listener probe 要分開，不能在 probe 期間 `stop_owned` 一個正常到期的 publisher。

### E484 — nested process.run(start) 等到 publisher 壽命結束才返回

`.ai-collab/renderer-fixture-cli-v1/summary.json` 保留未改：`all_passed` false，`simulations_executed` 0，`failure_cleanup_retired` true。jobs：missing-gate／help／bad-lifetime 符合預期；start exit 0／預期 0；startup exit 1／預期 0。其後 control、natural lifetime、negative cleanup 都沒有跑。start.stdout 是 ownership：pid 2192、creation_token `134359216624378439`、lifetime_seconds 30。startup.stderr 是 `process.lua:96` `poll_owned_ready`：「renderer fixture original lifetime exited before ready」，呼叫點 `moba_renderer_fixture.lua:180`。同次 evidence 的 startup.json 已存在（`127.0.0.1:6339`，deadline 恰為 started 加 30 秒），cleanup.json reason 為 `lifetime` 且 `listener_released` true。不是 publisher 沒綁定，也不是把 lifetime 加長就能修的流程。

原因已由這次證據對上原始碼，不是推測後就改 lifetime。`process.run` → `host.call('run')` → `os.execute` → `run_command` 的 `Command::output` 持有可繼承的 pipe 寫端，並把它們當成 start Lua 的 stdout／stderr。start 再 `os.execute` 原生 helper 的 `spawn`。Rust 1.95 `Command::spawn` 以 `bInheritHandles=TRUE` 呼叫 `CreateProcessW`，子行程繼承所有可繼承 handle，不只是日誌檔與 null stdin。publisher 因此一直握著 capture pipe 寫端，`Command::output` 等到 lifetime 結束、pipe 關閉才返回。start 接著印出 exit 0 與 ownership，下一個 wait-startup 看到的已是同一 creation token 的結束。`CommandExt::inherit_handles(false)` 在 1.95 不是 stable API。

預防：長命 `spawn` 在 `CreateProcess` 前清除目前 process 三個標準 handle 的 `HANDLE_FLAG_INHERIT`。stdout／stderr 仍寫入指定檔，stdin 仍是 null，creation token 仍來自這個 Child handle，而且不加 `CREATE_BREAKAWAY_FROM_JOB`。不要用加 lifetime、sleep，或拆掉 job 監督來掩蓋同步 start。失敗證據目錄不要覆寫。


### E485 — Primary 未讀既有 live test 的 opt-in 參數

renderer-spawn-validation-v1 native18、nested live spawn 與 owned wait 均通過；最後 owned-identity-live 因少帶 --run-owned-fixture 在入口 assert 失敗，沒有執行該項 native fixture。原 failed summary/log 保留。Primary 讀實際測試入口後修正 validator；只補跑受影響項 renderer-spawn-owned-identity-v2，實際 original-handle／wrong-token 拒絕／exact stop 通過。後續執行已知測試也必須先核對 CLI gate，不能從檔名推論無參數。

### E486 — 新增 fixture binary 未保留 Cargo 預設入口

來源審查發現 client runtime 原本只有 main binary，新增 src/bin/omoba-renderer-fixture.rs 後 Cargo.toml 未設定 default-run，一般 cargo run 會面臨二進位選擇歧義。Primary 明確設 default-run = omoba-client-runtime；實際 cargo metadata 同時找到原本與 fixture target，確認 default_run 指向原程式。未啟動正常遊戲，也未把 main.rs 改成 fixture。新增工具 binary 時須同步維持既有預設啟動入口。

### E487 — owned viewport 初稿的真實事件與 artifact 預期不一致

Grok run-muzakccd-1sda42 終止後，Primary 來源審查發現：C++ RemovalHold 先透過 BeginHold 送出一筆 MoveTo，但 Lua CASES 將 source-removal-clears-hold 設成零筆，固定總數 13，兩者不可能同時通過。AttackUi 雖綁定 ThumbMouseButton2 作 AttackOnlyClick，卻送 RightMouseButton，無法驗證宣稱的 action。另外原任務要求的 source_death 清除 hold/cast 尚無 dedicated case。這是來源審查結果，尚不是實際 viewport 執行失敗；兩組 Lua 契約 8／44 通過只代表 synthetic verifier/source assertions，沒有發現上述接線錯誤。

預防：逐一核對實際 C++ physical event、case ordinal 與 publisher intent，不能只對照案例名稱或用手工 JSON 當真實輸入證據。來源移除前須真的持有有效命令，並將那筆命令計入 inventory；UI case 必須送它實際綁定的按鍵；死亡與退役分開驗證，先證明 preview／hold 有效，再改狀態、最後 physical release 且不產生 stale intent。修復後重跑受影響契約、正式建置與 fresh owned viewport workflow，保留原證據。

同次審查追加：Q indicator case 未呼叫 TrySetCastMode，profile 預設 Normal，無法代表 QuickWithIndicator release 的取消路徑；測試應保存原 profile、明確安裝已驗證預設與指定模式，完成後還原原 bytes。維護中的 Lua workflow 不得強制依賴 ignored `.ai-collab` 的本機舊驗收摘要，歷史 hash 比較只能是可選資訊，live identity 必須由當前 production runtime 驗證。workflow report 寫入失敗必須使結果失敗；editor 原始 creation token 尚活著時不得還原其 profile。這些列入同一 bounded repair，尚未宣稱修復或通過。

### E488 — 真實 viewport 第一筆 intent 已送達，併發讀檔卻中止測試

正式 owned-viewport-ipc-build-v1 建置通過：NoEngineChanges／NoUBA／nocache，19 actions，UBT 452.27 秒；stage bridge dc4e9ac54533f5671c4c35ec2197d593988cab49c0782934fc0231bcce7d1f76，base_content 1e7fd7bf4494db93c9277e9227fc0fbe49ae4f3ab53f363b727f12d18d2a663c。fresh owned-viewport-ipc-v1 專項實跑失敗，UE report 0 succeeded／1 failed／0 notRun，native failure 為 intents.jsonl could not be read，停在 ground-rmb。實際 publisher intents.jsonl 有 ordinal1 MoveTo、local_request_id1、player1、epoch1、x_raw333／y_raw8928、authority_ack=false；catalog_compatible=true、configured_player_id1 與 genuine HUD/frame 已到達。這只證明第一個真實輸入成功傳輸，不能接受整個測試。

Lua 在讀缺少的 control-history.jsonl 時又產生例外，掩蓋 native failure。workflow JSON 保留兩個原始 lifetime retired=true、所有 cleanup ok=true；publisher cleanup reason shutdown_file／listener_released=true。所有失敗證據保留於 omfue/Saved/Automation/OwnedViewportIpc/owned-viewport-ipc-v1*，零個 authority games。

來源顯示 Rust append_line 每次 append 並 sync_all，UE LoadFileToString 使用預設 ReadFlags0。Windows 併發 reader 必須正確允許 writer，且 JSONL 尾端尚未完成換行不能當壞 JSON；確切 OS 錯誤本輪未收集，不能把推測當成已證實根因。修復應使用公開 FILEREAD_AllowWrite／完整 newline record 與既有有限 frame budget；永久 IO／完整壞 JSON 仍要失敗，不得無限制重試或減少預期筆數。先驗 native result 再讀後續 artifact，保留最早原因並附帶缺檔／cleanup 錯誤。

### E489 — 第二視窗測試把焦點送給不可聚焦的 SBorder

owned-viewport-contracts-v2 的 Lua21／44、bridge93／all-targets 均通過；ordinary nocache owned-viewport-ipc-build-v2 通過10 actions／UBT29.49秒；owned-viewport-regression-native-v2 exact55成功（包含 JsonlObservation），八張PNG已做SHA／尺寸比較，七張有差異的圖已逐張重新目視，固定2560x1080 byte-identical。前三張 adaptive 尺寸回到2800x1578／1400x1578；這不是量測OS DPI的證據，也不是實際地圖畫面。固定400x300字體仍偏小，屬既有可讀性後續工作。

fresh owned-viewport-ipc-v2 停在 window-focus-clears-hold，native failure second window did not take focus。前11個case已完成，包含真正 QuickWithIndicator preview 取消、正確 ThumbMouseButton2 UI 不送指令、Shift+A unqueued、text focus 與 modal 退役；window case 已先建立真實 held MoveTo，後續失敗不算完成。OpenBorderWindow 建立純 SBorder 然後 SetUserFocus 到它；SBorder 沒有可聚焦契約，不能用來驗證另一視窗 focus。改用明確 IsFocusable 的 SButton 等 public Slate 控制項，保留真實 focus path 與 held input 檢查，不能強制成功回傳／換成 text case／跳過 focus case。

v2 workflow 已保留正確最早 native error，不再被缺少 control-history 掩蓋；兩個 original lifetime retired=true、cleanup ok=true、listener_released=true。所有 v1/v2 證據保留；這批未接受，死亡／replacement／epoch 等後段尚未執行。新增 fixture 前應核對目標 widget 的 SupportsKeyboardFocus，並確認真實 Slate focus path，不把容器當作可聚焦控制項。


### E490 — 來源退役後 Q 預覽無法重新啟動

ordinary nocache owned-viewport-ipc-build-v3 通過；fresh real owned-viewport-ipc-v3 通過前19 case，包含真正 second-window focus、兩個 source death、replacement/removal 與 baseline restore。第20 case epoch-attack 在 Q preview 啟動前失敗：epoch-attack cast preview did not activate，LastSubmitStatus.Error 空白。原始 editor/publisher lifetime retired=true，三個 cleanup 均成功。全批尚未接受，22case/15intent/10stage 契約不變，零個 authority games。

來源初查：CombatAim.Cancel 先 Accept，若 context 失效或清除 pending 即提前返回，可能跳過 OwnerRetired 的 ClearHoldLatch；Accept.CloseHold 留下 await-release，但 component OwnerRetired 已清掉 hold token，後續 Q release 無法傳到 model。這是待修復驗證的具體根因候選，不能以重設測試 profile、增加重試或減少 case 規避。修復須保證 owner retirement 在無效／死亡／changed context 仍清理自身 latch，保留一般 cancel 的防重複與 stale-release 安全，加入可重現 regression 後重跑真實 viewport。


### E491 — 幾何與 hit-test 專項錯用 NullRHI

Primary 在 ordinary owned-viewport-ipc-build-v4 通過後，誤以 NullRHI 啟動 owned-viewport-retirement-native-v1 整組專項。新增 Om.Runtime.CombatAim.OwnerRetirement 通過，但 PreciseWorldCommand.Rebind/Route、CameraProjectedDrag、ResponsiveHudLayout 與 SummonerOwnerCards 等實際 painted geometry 案例失敗（window/card 0x0、hit-test path 不屬於 viewport）。這份失敗不接受，也不代表 production regression。完整原始 report/log 保留於 .ai-collab/.bridge-data/worker-pool-machine-v1/owned-viewport-retirement-native-v1。

修正 scoped helper 必須明確傳 with-render，使用 D3D12，不再允許同組含真實 Slate 幾何案例時默認 NullRHI。以 fresh run 重跑全部受影響56 native並核對實際報告；純模型單測才可獨立使用 NullRHI。


E490修復驗證：Cancel的OwnerRetired處理提前於Accept gate返回，確保ClearHoldLatch執行；新增純model regression驗dead/invalid/epoch/generation、重複退役、stale release與一般Escape鎖存。formal build-v4、D3D12 exact56及fresh real viewport v4 exact22/15/10通過。E491改用正確D3D12後全56通過，失敗證據保留。


### E492 — PowerShell 下把 wildcard 當成 rg 路徑

Primary 兩次 read-only 搜尋把 OmCombatAim*／transport* 放在 rg 的目錄參數，Windows 回報 OS error123；其餘已存在路徑的搜尋仍成功，沒有修改來源或影響驗收。不要依賴 shell 展開 rg 路徑 wildcard：先用 rg --files 找到真實路徑，再傳存在目錄與 -g 檔名過濾；未確認目錄不得作為實際讀取路徑。


### E493 — 擴充 fixture 時未量測就放寬總 frame 上限

Grok cadence batch在31case/21intent擴充時把OwnedViewportIpc總frame上限20000改為26000，report承認未量測。Primary來源審查恢復20000；原有readiness/intent/ack界限與workflow860000ms均不變。新增case本身不是放寬失敗條件的依據，須先保留原界限實跑；若確有合理新增工作量超限，需以實際分段耗時與有限最壞界限證明，不用提高timeout掩蓋退役／輸入／IO缺陷。此項為source-review修正，尚未代表整批實跑通過。


E492追加：Primary還曾猜測build.stdout.log路徑，Get-Content回報不存在；實際helper完成後產物為build.log／UBT-Log.txt／summary.json，執行中目錄尚未有captured stdout。後續先用rg --files確認，需即時進度則讀已確認的UBT Log.txt，不能猜證據檔名。

### E494 — UE5.8 checked FString::Printf 不接受動態格式

普通NoEngineChanges/NoUBA/nocache held-world-cadence-build-v1失敗（UBT exit6）；OmControlProfileTests.cpp631 FString::Printf(Pattern, OmControlProfileLiteral::Schema) 產生C2664，Pattern是const TCHAR* runtime資料，不能轉成checked literal format。失敗build.log與UBT-Log.txt保留。應以literal格式或明確單一placeholder替換構造相同malformed JSON案例，不能關閉format檢查、改Engine/BuildId或用refresh繞過。Lua21/44已通過；native與31case viewport尚未執行。

### E495 — 數值設定測試繞過新增的UI producer

來源審查發現ControlRuntimePersistence新增WorldRepeat區塊直接呼叫owner TrySetWorldRepeatSettings，再觀察spin值／磁碟；這不驗SSpinBox.OnValueCommitted是否真正接線。既有真SettingsWindow/SlateUser已可用，應聚焦實際tagged spin並送Slate Right按鍵（0.10→0.11），確認live profile與磁碟一致、initial0.20不變、world route不增加，完成後恢復SettingsRoot焦點。不得用直接setter或SetValue冒充真UI事件。這是測試producer缺口，尚未宣稱production故障或修復通過。


E492追加：Primary後續又將未發現的OmViewportPointerGeometry.h當既有檔案搜尋，rg回報不存在；實際Public inventory只有OmOwnedPointerTracker.h。再讀取前必須完成rg --files，新增helper與既有檔案不能混淆。

### E496 — 真實 A 左鍵指令與預期世界點不符

E492追加（恢復上下文後）：Primary把摘要中的簡寫omfue/Source當成實際路徑，三個read-only rg讀取失敗。已以rg --files確認實際位於omfue/Plugins/OmRuntime/Source，後續讀取成功。摘要簡寫不能取代檔案inventory；未修改程式或影響建置。

held-world-cadence-build-v2普通nocache通過；held-world-cadence-native-v1 D3D12 exact57 Success／0warning／0notRun，真spin Right事件及保存驗證通過。八張PNG比較，六changed重新目視、二identical；settings截圖僅頂部list，未目視到新repeat section，不能擴大視覺證明。fresh held-world-cadence-viewport-v1通過前22舊case，第23 armed-a-world-lmb genuine AttackMove已送達，但x_raw178／y_raw9062與存下的SampleOwnedWorldCommand point不符。當前錯誤沒記expected point/raw/pixel，尚無法定量判定根因。原editor/publisher retired、profile cleanup與listener release均成功，所有失敗證據保留。

來源初查fixture使用GetTickSpaceGeometry投影absolute pointer，production使用實際arranged FWidgetPath geometry；camera也可能在virtual mouse move前後更新。這些是待驗候選，不能認定production座標錯或直接放寬1raw quantum。修復應把expected採樣接到相同production event pixel/frame，使用共用轉換／唯讀觀測，不新增熱路徑重複採樣、不假造指令或刪case。31/21/10與frame20000保持，後段repeat尚未跑，整批未接受。

### E497 — UI 點擊後 Escape 同步斷言失敗

E496修復普通build-v3及D3D12 native-v2 exact57通過；fresh held-world-cadence-viewport-v2 的第23 armed-a-world-lmb嚴格座標通過，第24 armed-a-ui-lmb在真正UI點擊未確認aim後，送Escape同一Advance立即assert disarm失敗。應先核對Slate→gameplay按鍵與World.Tick時序、焦點及guard，不能直接認定production故障；維持真事件與有界跨frame驗證，不用直接Disarm或移除case。repeat後段仍未跑，整批未接受。

E497來源核對（尚未重跑）：viewport-v2 native.json 的 armed-a-ui-lmb intents 為空，failure 字串即該 assert；workflow 1 failed、editor/publisher retired；cleanup listener_released 且 authority false。ue.log 只引用 OmOwnedViewportIpcTests.cpp(2315)，沒有焦點或 IsKeyboardBlocked 記錄。Escape 是 ActionInput 的 consumed IE_Pressed BindKey，由之後的 player ProcessInputStack 呼叫 HandleEscapePressed；fixture 的 PumpFrame 在 Advance 之前 Tick，同一步 KeyDown/KeyUp 後立即讀 aim 發生在下一次 World.Tick 之前。A 的 arm 已用 WaitArmed 跨 frame，ArmedAEsc 原本也是同一步立即 assert，尚未跑到。UI 按鈕不是 editable text，這次失敗不能證明鍵盤被擋。修正只改 fixture：Escape 保持按下跨過既有 kAbsentFrames 內的後續 tick，再 KeyUp，失敗文字記錄 focus before/after/now、viewport path、active window、modal、Slate handled。未改 production、未放寬座標或 frame 20000。此次修正尚未建置或實跑。


Cadence scoped acceptance：ordinary nocache build-v4通過；D3D12 native-v2 exact57通過，E497只改專屬viewport test，未重跑未變native suite。fresh held-world-cadence-viewport-v3 exact31cases／21intents／10stages全部通過，含A地板嚴格座標、UI不確認、Esc跨真tick取消、相同點去重、改變目標、large-delta只一次、release／UI focus／profile中止。原程序退役、listener釋放、profile還原通過。E496與E497修復驗證完成；scope為renderer IPC local admission，不是authority ACK。八張widget PNG中五responsive重新目視、三identical沿用，repeat設定區仍未截到。accepted .ai-collab/held-world-cadence-accepted-v1/summary.json；完整驗收尚未跑，quota10/10不變，開始雙快捷鍵批次。

### E498 — 雙鍵 token 迴圈突破單 owner 連發上限

Primary來源審查：雙鍵batch的RefreshHeldWorldPointer改為逐token到期就IssueWorldCommand，註解也改成one attempt per token per tick；兩個按鍵同時held時可在同一owner tick多次採樣／提交，突破先前accepted cadence每owner tick最多一次的契約。需保留各實體按鍵獨立release，同時定義單一確定性排程仲裁、丟棄stall backlog，不能把上限乘以slots數。這是source-review發現，尚未執行新dual runtime。

E498來源修正（尚未重跑）：先清掉已鬆開的實體鍵，再讓每個仍按住的token累積自己的時鐘。同一tick裡所有已到期token都把餘量歸零，只選最舊的一筆發行一次，並把owner冷卻設成該筆的間隔。冷卻未結束時其他token即使跨過門檻也不發行、不保留backlog。放開其中一個鍵只移掉該鍵的hold。尚未建置或實跑。

### E499 — UMG Slot 成員被新快捷鍵欄位名稱遮蔽

ordinary dual-bindings-build-v1 NoEngineChanges／NoUBA／nocache實際失敗，UBT exit6。OmControlSettingsWidget.cpp298參數Slot、313區域Slot、331與368迴圈Slot觸發C4458，遮蔽繼承的UWidget::Slot。必須用BindingSlot等明確名稱並審查widget其餘宣告，不關閉warnings-as-errors或修改Engine；原build.log／UBT-Log.txt保留。

E499來源修正（尚未重跑）：上述四個UOmControlSettingsWidget成員範圍已改名BindingSlot，含標頭宣告與列迴圈捕捉。匿名函式SlotName／SlotLetter與指令列的自由函式不是UWidget成員，不遮蔽UWidget::Slot，未改。未關閉警告，未改Engine。OmWorldBridgeActor.cpp不在該次codegen輸出，其手改dispatcher不是生成覆寫。尚未重建。

### E500 — 雙快捷鍵正向事件與新增設定視覺證據缺口

Grok雙鍵report承認只補disconnected PreciseWorldCommand.Rebind拒絕與release，沒有擴充實際renderer IPC的主滑鼠＋副鍵盤正向case；同時未補新的search／empty／repeat section實際capture，雙drag及cast原token未有對應事件驗證。初始task明確要求上述producer／consumer範圍，不能以codec／拒絕或尚未拍攝的widget source代替。應保留原31cases並增加有限真事件與一致Lua inventory，新增實際production設定區截圖及真typed search事件，不製造Connected或ACK；全批仍未接受。

E500來源修正（尚未重跑）：OwnedViewportIpc在原31案之後追加12案，Lua CASES與contract改為43案／26條wire／10個既有stage，frame仍20000。新增主RMB與次鍵K同時送達、重複K不加算、放開RMB留K、改點嚴格座標、兩token大delta只一次attempt、最後放開停止，以及Q指示器的次鍵F6、Esc、右鍵、文字焦點與profile取消。設定截圖只在既有om-controls-settings-capture旗標存在時，依該路徑寫出-repeat、-search-empty、-secondary；搜尋另有聚焦後的ProcessKeyChar zzzz。CameraProjectedDrag追加中鍵與F6重疊放開。這些都還沒執行，也不能把檔名寫成已檢視的圖。

E492追加：Primary盤點runtime::shop時，在同一個shell呼叫先列檔後立即對猜測的runtime/shop目錄搜尋；列檔沒有匹配，第二步仍執行而回報不存在。Rust模組命名不能推斷檔案目錄，發現路徑與依賴該結果的讀取必須分開、先檢查inventory結果；此次read-only失敗沒有改動source。

### E501 — 雙鍵重建成功後原生測試三項失敗

ordinary dual-bindings-build-v2與Lua21／44通過；D3D12 dual-bindings-native-v1實際exact59，56Success／3Fail／13errors／0warnings／0notRun。PreciseWorldCommand.Route三個既有consumed click／hold斷言失敗：新RefreshHeldWorldPointer用Controller::IsInputKeyDown檢查已由前置pointer processor消耗的滑鼠，清掉合法hold，需按實際事件owner來源追蹤，不偽造PlayerInput狀態。CameraProjectedDrag四個新增drag斷言失敗，profile rebind會換pointer tracker，fixture未重新送公開mouse sample是待核對原因，不可直接SetCameraDrag。ControlProfileBindingSlots六個斷言失敗：Recall副鍵F8已與之前focus副鍵F8衝突；version2正向fixture將Stop改H卻保留預設Hold H，合法collision拒絕不能當migration故障，需修正有效測試資料並保留負向碰撞測試。43case viewport尚未跑，失敗native原report／log保存，全批未接受。

E501來源修正（尚未重跑）：native-engine.log 13條Expected與上述三測一致，Route是2797／2799／2802，CameraProjectedDrag是664／666／668／672，BindingSlots是761／764／793／796／798（798同時含extension與pretty-export）。滑鼠hold不再用IsInputKeyDown刪除；鍵盤物理鍵仍用。最後一個token離開時才把HeldWorldOwnerCooldown歸零，還有token就保留。CameraProjectedDrag在rebind後呼叫公開RefreshInputOwnership，確認舊sample已退出，再送一次公開mouse move才Press。Recall副鍵改未占用的F10；version 2先解除Hold再把Stop放上H。ExportJson是pretty JSON，斷言同時接受有空白與無空白的slot字面。dual-secondary-key改比對發行當下的GetIssuedWorldSampleForAutomation，滑鼠同frame檢查未改。另加四個viewport case：hold-clock-armed／join／partial／fresh，inventory改為47案／30條wire／10 stage，frame仍20000。這次沒有建置、沒有跑Lua或native、沒有看PNG，不能寫成通過。

### E502 — profile 安裝留下無法送達的 cast latch

來源審查，不是已測得的 native 失敗，也不是設定 widget 已證實的 production 故障。InstallRuntimeProfile 先增加 ControlProfileRevision，再 RetireCastPreview(FocusLoss)，然後清掉 component 的實體 cast token。新 revision 讓 Accept 因 pending／held 不符而 CloseHold，bAwaitRelease 留下舊 chord；FocusLoss 看到 Gate 已 cleared 就返回，不像 OwnerRetired 那樣在 Accept 中止後仍 ClearHoldLatch。component 接著丟掉唯一能 Release 的路徑，所以同一個技能鍵的下一次按下是 latched NoAction，舊鍵放開也碰不到新的 activation。Escape、世界右鍵與 FocusLoss 的 repeat-latch，以及 E490 的 OwnerRetired，必須保留。

E502來源修正（尚未重跑）：新增 EOmCombatAimCancelReason::ProfileOwnership。Cancel 對 OwnerRetired 與 ProfileOwnership 都 DiscardPending 並 ClearHoldLatch，即使 Accept 已把 hold 收成 await-release。RetireCastPreview 對這兩個原因都清 preview 與實體 token，輸入已 retired 時仍執行。InstallRuntimeProfile 改呼叫 ProfileOwnership，之後仍清一次實體 token。FocusLoss 本身不改。Om.Runtime.CombatAim.OwnerRetirement 追加：revision 後的 FocusLoss 仍要等真 release；ProfileOwnership 後同一鍵可再 Preview，舊 token 的 Release 不提交也不清掉該 Preview，接著 W 的新 activation 不被前一個 Q token 提交或清掉，Escape 仍保留 W 的 repeat-latch。OwnedViewportIpc 的 cast-profile-cancels 在 Q 預覽中把次要 CastQ 改成 F10，確認預覽熄掉且主要仍是 Q，再按下 F10，舊 Q 放開在 kAbsentFrames 內不送 wire 也不清掉新預覽，最後 Escape 與 F10 放開，結尾 PollExpect None。沒有新 case，Lua 仍是 47／30／10，frame 20000、kIntentFrames 180、kAbsentFrames 90 未放寬。

ControlRuntimePersistence 先前只提交主要鍵，次要 AttackMoveAim 只進入擷取再被搜尋取消。這是測試 producer 缺口，讀過的設定 widget 路徑已有 SecondaryRebind、NativeOnKeyDown 與 SecondaryUnbind，沒有改 widget。測試改為公開 Slate 點次要改鍵、ProcessKeyDown／Up F10、在解除前截 production-settings-fixture-secondary.png，再點 SecondaryUnbind。live／磁碟的次要是 F10 然後未綁定，主要仍是 J，revision 各加一，world route 不變，world-repeat 0.11 保留。截圖檔數仍是既有 11 張，沒有新增或刪除檔名。這次沒有建置、沒有跑 Lua 或 native、沒有看 PNG，不能寫成通過。

### E503 — OwnerRetirement 聚合斷言讀到後續 token

ordinary dual-bindings-build-v3 nocache 與 Lua21／44 通過。D3D12 dual-bindings-native-v2 實際 exact59，58Success／1Fail／1error／0warning／0notRun。E501 的 Route、CameraProjectedDrag、ControlProfileBindingSlots 與此前 ControlRuntimePersistence 次要提交／解除均通過。唯一失敗是 Om.Runtime.CombatAim.OwnerRetirement。native-report/index.json 的錯誤是 Expected 'Profile ownership lets the next press arm and a stale release does not commit or clear it' to be true，檔案 OmCombatAimModelTests.cpp 行 1373。

原因在測試，不在 model。該聚合先做完 NextQ、StaleQ、NextW、StaleAgainstW、Escape、release、AfterW，才讀 Profile.GetPendingToken()。此時 pending 已是 AfterW 的新 token，不再等於 StaleQ 當下的 NextQToken，也不等於 StaleAgainstW 當下的 NextWToken。production Cancel／Release 行為不改。

E503來源修正（尚未重跑）：StaleQ、StaleAgainstW、W release 之後立刻記下 pending token 與 HasCommitted，聚合改比對這些快照。結果、token、普通 Escape latch 與 OwnerRetired 斷言保留。沒有改其他測試名、Lua inventory 或截圖。這次沒有建置、沒有跑 native、沒有看 PNG，不能寫成通過。

### E504 — viewport 鍵盤放開與同一 Advance 的 hold 斷言

ordinary dual-bindings-build-v4 nocache 通過。D3D12 dual-bindings-native-v3 實際 exact59，Success／0errors／0warnings／0notRun。11 張截圖已盤點，Primary 目視 9 張 changed／new，2 張與 baseline byte-identical 沿用。fresh dual-bindings-viewport-v1 在進入的第 38 案 dual-final-release 失敗，字串是 final K release left a world hold。前 37 案完成，含原 31 案與 dual simultaneous／move／change／largeDelta／primary release。native JSON／cases 與原 log 留在 omfue/Saved/Automation/OwnedViewportIpc。不能寫成 47 案通過。該次工具結束後沒有作用中的 build、editor、publisher 或 workflow。

原因在 fixture 時序，不是 production hold 卡住。第 37 案 dual-release-primary 的滑鼠 Up 在同一 Advance 同步清掉 RMB 並留下 K，該案已通過。DualFinalRelease 對 K 呼叫公開 Slate KeyUp 後，同一步立刻讀 IsWorldPointerHeld。PumpFrame 的 World.Tick(LEVELTICK_All, 1/60) 在 Advance 之前；HandleWorldCommandKey(false) 要到下一次 World.Tick 的 player input 才清 token。這是 E497 已核對的 consumed BindKey 時序，方向相反：放開之後要等下一個 tick 才觀察清除。滑鼠 owned callback 同步，鍵盤 player stack 不是。沒有 production lifetime 證據，不改 owner、model 或間隔。

E504來源修正（尚未重跑）：只改 OmOwnedViewportIpcTests.cpp。dual-final-release 仍用公開 KeyUp(K)，先讓出一幀，再在既有 kIntentFrames 內確認沒有多一條 wire，並在 kAbsentFrames 內等到 hold 與 K 都清除；RMB 不得復活。失敗字串保留。沒有直接清 hold 或寫私有狀態。hold-clock-fresh 把滑鼠 Up、KeyUp(K)、零 hold 觀察、新的 RMB Down 拆到不同 Advance。零 hold 仍受 kAbsentFrames 限制，wire 讀取仍受 kIntentFrames 限制。最後一個 token 離開才讓對照用的 owner 冷卻歸零；partial 在 K 還在時放開再按下 RMB，不重置冷卻。0.05 初延遲改為對照實際每幀 1/60 加上一次 TickComponent(0.05)，不假設 publisher 輪詢是零幀，也不關掉 production tick、不繞過真實事件。一 tick 仍最多一次 attempt，到期但冷卻中的 token 不留 backlog。間隔仍是 0.05／1.0，恢復仍是 0.20／0.10。47／30／10、frame 20000、kIntentFrames 180、kAbsentFrames 90 未放寬。沒有新 case 或新截圖。native 59 沒有因這次僅 fixture 的改動而失效。這次沒有建置、沒有跑 viewport，不能寫成通過。

### E505 — hold-clock 測試複製了 production scheduler

來源審查，不是已測得的 production 或 runtime 失敗。E504 的新 fixture 尚未建置，也尚未跑 viewport。FOwnerClockSim 用 token、cooldown、BeginFrame 與 Tick 複製 RefreshHeldWorldPointer，ExpectOwnerClock 再要求嘗試次數等於該複本。測試不應鏡射實作。

E505來源修正（尚未重跑）：刪除該 scheduler 複本與 production-tick-sim 相等斷言。公開事件與 E504 的跨 frame KeyUp 保留，production 不改。四個 hold-clock case 只記實際 1/60 pump、顯式 0.05、嘗試次數前後值，以及 press 與上次 attempt 的時間。每個觀察到的 tick 最多一次 attempt；初刷新不得早於 press 之後 0.05；仍有 token 時，下一次不得早於上次 attempt 之後 1.0；最後一個 token 放開再按新滑鼠，0.05 重算，不必等舊的剩餘 1.0。沒有 token 陣列、armed 狀態或 cooldown 模擬。這四案的每個實際 tick 都觀察。join、partial、fresh 若在證明冷卻窗口前已經過了該 1.0，明確失敗，不改成弱斷言。repeat-identical 仍負責既有 backlog。47／30／10、frame 20000、kIntentFrames 180、kAbsentFrames 90 不變。沒有改 production 或 disconnected tests，native-v3 exact59 仍可沿用。這次沒有建置、沒有跑 viewport，不能寫成通過。


雙快捷鍵scoped acceptance：ordinary dual-bindings-build-v5 nocache通過；D3D12 dual-bindings-native-v3 exact59 Success／0errors／0warnings／0notRun。fresh dual-bindings-viewport-v2 exact47cases／30intents／10stages全部通過，含主副鍵獨立release、單owner連發時鐘、profile施法退休／新啟動／stale release、設定介面真實副鍵commit／unbind。11張widget PNG中9張重新目視、2張baseline byte-identical；400x300可讀性仍待偏好批次。原程序退休、listener釋放、profile還原皆通過。accepted .ai-collab/dual-bindings-accepted-v1/summary.json；scope是renderer IPC local admission，不是authority ACK。完整驗收尚未跑，quota10/10與父項完成數不變。

E492追加（Primary工具操作）：本輪再次猜測omoba-core/src/comp/hero.rs及review/ue-stat.txt，實際檔案為已發現的runtime/native/comp/hero.rs與ue-stat.log。另再次用未讀取的裸`#`當apply_patch anchor而被原子拒絕，沒有修改source。不得將namespace或heading猜測當路徑／context；先盤點再依實際結果讀取，append紀錄使用固定Lua helper與fresh guard，避免重複錯誤。

### E506 — 指定單位施法錯用普攻敵方資格，升級fixture直接注入技能點

來源審查發現，owned ability batch新SubmitOwnedAbilityCommand對所有Entity target呼叫IsPublishedLegalHostile，會拒絕已存在training_support的lumen_aid（templates/abilities.lua明確heal_ally，允許存活隊友及自身）；generic_effects.rs及既有native測試確有友方／自身治療規則。這是具體source-contract缺陷，還未跑新UE build／viewport，不能稱實測runtime失敗。應從authoring effects／共用內容規則產生目標陣營metadata，保留身份、公開性、生命、range／mana與權威驗證；不可硬編碼某技能ID或對友軍套用普攻attackable gate。加入實際友方單位與Alt自身的正向事件，敵方誤用與hidden／dead的負向事件，保留101案前綴。

同批owner_upgrade_point將birth skill_points直接改1、hero level不變，report已承認超過birth accounting。改用既有Hero::add_moba_experience與實際experience_to_next取得合法升級技能點，保持既有可用QWER及獨立apprentice測試。已有驗證：owned-ability-rust-v1 library99／all-targets／fixture build通過，fixture-bin測試0明確記錄不冒充正向測試；Lua owned viewport contract22通過。新declared101cases／51wires／30stages未實跑，整批尚未接受。

E506 source修復（尚未跑 content、codegen、Rust、Lua、UE、native、viewport，不能寫成通過）：單位陣營來自 omoba-content-model::unit_target_allegiance。HealAlly 且沒有 Damage／SlowEnemy／StunEnemy／ControlEnemy 為 ally；那些敵方單位效果且沒有 HealAlly 為 enemy；兩邊同時存在仍是 validate_ability_effects 的 conflicting unit target allegiance。空效果、自體、範圍、衝刺與未知 custom 為 unspecified，不預設成敵方。codegen 的 FOmGeneratedAbilityMetadata.UnitAllegiance 由 generated_ability_metadata_source 寫出。沒有手改 OmContentClasses、OmVisualRegistry 或 template_ids。目前已生成的標頭還沒有這個欄位，bridge 在 Primary 跑 codegen 之前不會編譯。SubmitOwnedAbilityCommand 的 Entity 分支讀該欄位：enemy 仍用 IsPublishedLegalHostile，所以 lumen_bolt 的 Alt-self 仍拒絕「not a disclosed legal hostile」；ally 用 IsPublishedLivingAlly，保留目前身份／世代、KNOWN、ALIVE、來源已知且存活、同一來源玩家、有限位置與既有 range／mana，不要求 HOSTILE 或 ATTACKABLE，帶 HOSTILE 旗標則拒絕「not a disclosed living ally」；其餘走 IsPublishedLivingDisclosedUnit，錯誤是「not a disclosed living unit」。SampleOwnedCastPointer 仍不看陣營。owner_upgrade_point 改為出生 level 1、經驗 0、點數 0 時呼叫 add_moba_experience(experience_to_next)，要求剛好升一級、經驗回到 0、一點，QWER rank 仍是 1。viewport 在 q-target-dead 之後追加 9 案，101／51／30 順序不變。宣告盤點改為 110 cases、53 wires、34 stages。沒有新的實測失敗。Primary 先前的 library 99、all-targets、fixture build 與 Lua contract 22 是這次修復前的 101 前綴結果。E507 修正下面這一條：當時的 IsPublishedLivingAlly 只拒絕 HOSTILE，尚未要求與目前來源列的 disclosed team 相等。110／53／34 是 E507 之前的前綴，最終盤點改看 E507。

### E507 — 非敵對不等於盟友，治療必須同一 disclosed team

來源契約審查，不是已測得的 runtime 失敗。run-muzolxmb-s90bjt 在 28m21s 結束。E506 的內容 helper、codegen 欄位與經驗來源可保留，但該批尚未 codegen、尚未測試。IsPublishedLivingAlly 當時只在 OM_RELATION_HOSTILE 時拒絕，其餘交給 IsPublishedLivingDisclosedUnit。scripts/base_content/src/generic_effects.rs 的 is_ally 比較 faction_of 字串；omoba-core/src/runtime/native/scripting/parallel_world_adapter.rs 的 faction_of 取 team_keys 裡該 team_id 的鍵。權威治療要的是同一隊，不是非敵對。Faction::is_hostile_to 明文規定 Neutral 對所有陣營都不敵對，所以另一隊的 Neutral 可以存活、已公開、沒有 HOSTILE，仍不是盟友。沒有改 game.proto 或 HUD ABI。FOmPublishedDirectTarget.TeamId 與 PublishedRelationSourceKey 已有雙方目前的 disclosed team。不讀 PublishedLocalTeamId，也不要求 ATTACKABLE。

E507 source修復（尚未跑 content、codegen、Rust、Lua、UE、native、viewport，不能寫成通過）：IsPublishedLivingAlly 仍先拒絕 HOSTILE，仍呼叫 IsPublishedLivingDisclosedUnit，因此身份、世代、生命、來源已知且存活、同一來源玩家、有限位置都保留。接著用 PublishedRelationSourceKey 取出目前來源列，該列也必須通過同一個 living disclosed 檢查，而且來源列的 OM_RELATION_KNOWN 必須成立，再要求來源列 TeamId 等於目標 TeamId。IsPublishedLivingDisclosedUnit 本身不改，unspecified 單位政策仍是通用公開單位，不套同一隊閘門。同隊盟友與 Alt-self 仍走這條通過路徑。renderer fixture 的 Actor 增加 faction_kind，actor() 預設仍是 1，entity() 不再把每個 combat 寫死成 faction_kind 1；disclose_owner 把目標重置成 Player。新增 owner_support_neutral：training_support、目標 faction_kind 3（visibility.rs 的 Neutral）、team 3、存活、可攻擊、combat 仍在、來源玩家與來源 faction 1 不變。這個目標對來源 is_hostile_to 為 false，而且 team 不同，所以拒絕的是同一隊閘門。viewport 在既有 110 案之後追加 ability-support-neutral（style 0）與 r-aid-neutral-refuses（style 25 畫出的單位，拒絕字串仍是 not a disclosed living ally）。沒有新 wire。宣告盤點是 112 cases、53 wires、35 stages；C++ ability 表 static_assert Count == 65。frame 20000、kIntentFrames 180、kAbsentFrames 90、OPERATION_MS 860000、lifetime 900 未放寬。沒有新的實測失敗。報告在 .ai-collab/moba-owned-ability-same-team-repair-report.md。

### E508 — Primary 驗證 helper 重複寫入拒絕覆寫的 summary

owned-ability-content-v1 的內容模型 27 tests 已執行通過，codegen-tests 也已完成；接著 Primary 新 helper 在迴圈第二次呼叫 json.write(summary.json, report, false)，因為 false 明確禁止覆寫而失敗。這是驗證工具的錯誤，不是內容或產生器測試失敗。codegen emit/check 尚未執行，原始 logs 與第一步 summary 保留，整個 v1 不宣稱成功。已核對 tools/lua/lib/json.lua 的 write(path,value,overwrite) 定義；移除迴圈內的寫入，只在迴圈結束時對 fresh 目錄寫一次 summary。使用新 evidence id 重跑。之後寫 helper 時先核對 API 參數與 evidence 生命週期，避免以禁止覆寫的選項更新同一檔案。

### E509 — OmEditor 連結未匯出的能力視覺陣列

ordinary owned-ability-build-v1，NoEngineChanges／NoUBA／nocache，UBT exit 6，outer 1。原始編譯已完成，連結 OmEditor 失敗。LNK2001：OmOwnedViewportIpcTests.cpp.obj 未解析 AbilityVisualEntries 與 AbilityVisualEntryCount。LNK1120：兩個未解析符號。日誌在 .ai-collab/owned-ability-build-v1/{build.log,UBT-Log.txt,summary.json}。這是實測連結失敗。

根因：FindVisualAbility 直接走訪 OmContent::AbilityVisualEntries／AbilityVisualEntryCount。這兩個產生陣列沒有 DLL export。OmRegistry.h 已有 OMGENERATED_API 的 FindAbilityVisualEntryById(const TCHAR* Id)，同一個測試檔已 include OmRegistry.h。

E509 source修復（尚未重跑 build，不能寫成通過）：FindVisualAbility 改呼叫 OmContent::FindAbilityVisualEntryById(*AbilityId)，不再讀原始陣列。呼叫端的 ability id、catalog、shape 檢查不變。沒有改產生器、產生標頭、新 API 或 export。盤點仍是 112／53／35，時限與 frame 未放寬。owned-ability-content-v2 的 content-model 27、codegen 41、emit／check，owned-ability-rust-v2 的 renderer library 99、all-targets、fixture build，以及 Lua owned 24／acceptance 44 已通過，這次只改測試查找，那些結果仍有效。Primary 接著跑 ordinary build-v2 nocache，再 dynamic native-v1 與 fresh viewport-v1。

### E510 — 第一個正向單位 Q 有預覽但沒有 AbilityCast

owned-ability-build-v2 ordinary nocache 在 E509 之後通過。D3D12 owned-ability-native-v1 exact 59 Success／0 warnings／0 notRun，11 張 PNG。content-v2 27／41 加 emit／check、rust-v2 library 99／all-targets／fixture build、Lua owned 24／acceptance 44 通過。native 59 不是正向施法證明。

fresh owned-ability-viewport-v1 失敗。native.json 的 failure 是 q-normal-unit produced no AbilityCast intent。success=false，authority_ack=false，catalog_compatible=true。已進入 49 案：前 47 案完成，ability-cast-ready 完成且沒有 wire，q-normal-unit 的 intents 是空的。已發布 30 條舊前綴 intent、11 個 control stage。最後一條 control ack 是 owner_cast_ready，source 11／epoch 4、target 21／epoch 4，authority_ack=false。startup 的 source／target epoch 是 1。cleanup 是 shutdown_file，listener_released=true，authority=false。publisher 與 editor 已退休，profile 已還原。112／53／35 未接受。

ue.log 在測試幀 22 有一筆 OM_CAST_REJECT player=1 slot=0 label=CastConfirm accepted=0 error=Cast confirm was rejected。沒有 OM_CAST 提交行。約 180 幀後，幀 203 才報沒有 AbilityCast。HandleCastPointer 只在 aim model 的 Outcome 是 Rejected 時寫這句固定字串，並蓋掉 LastSubmitStatus。Pointer 對 Normal 世界左鍵的 Rejected 只來自 CommitPending：Entity 形狀在樣本沒有已公開單位時拒絕。這次執行沒有留下該點的 cast／world 樣本，所以不能證明是瞄準像素、cast 與 world 取樣不同，還是確認當下的身份。沒有改 production。

E510 source修復只在測試（尚未重跑，不能寫成通過）：PollAbilityWire 在 kIntentFrames 內沒有新 wire 時，失敗字串改帶公開狀態：IsCastPreviewActive、GetLastSubmitStatus、GetLastRequestedCommand 的 label／slot／player／target kind／target、profile revision、目前 hero 與 view／target epoch，以及同一個 PendingPixel 上 SampleOwnedCastPointer 與 SampleOwnedWorldCommand 的單位、點與 error。不寫私有狀態，不放寬 frame 或盤點。Primary 跑 ordinary build-v3，再 fresh viewport-v2。這是 viewport 測試專用修正，native 59 可沿用。完整驗收最後。

owned-ability-build-v3 ordinary nocache 通過，14.30s／4 actions，只編譯 OmOwnedViewportIpcTests 再連結 Editor，production 未改。native-v1 exact 59 沿用正確。fresh owned-ability-viewport-v2 仍在第 49 案 q-normal-unit 失敗，30 wires／11 stages，catalog_compatible=true。publisher、editor、profile 清理成功，listener released。新診斷是 preview=0 accepted=0 id=0 error=Cast confirm was rejected label=CastConfirm slot=0 cmdPlayer=1 cmdKind=0 cmdTarget=0/0 rev=15 hero=11/4 view=4 target=21/4 pixel=400,300 cast=1 unit=0:0/0 point=1:5512,19216 castError= world=1 hostile=1:21 point=1 worldError=。同一像素的 world 樣本打中 target 21，cast 樣本只有地面點、沒有單位。

根因：SampleOwnedCastPointer 只用 Actor->GetSimpleCollisionRadius()，非正數就跳過。同一檔的 CapturePublishedDirectRadius 在 simple radius 為 0 時改用該 actor 自己的 GetActorBounds(false) 視覺範圍。ReplacePublishedDirectTargets 已用這個 helper，所以 world query 成功而 cast query 漏掉。OmUnitActor 建構子的根是 USceneComponent，NativeBodyMesh 是 SetCollisionEnabled(NoCollision)，這是一般 native render unit，不是單一技能或 fixture。

E510 source修復：SampleOwnedCastPointer 改呼叫既有 CapturePublishedDirectRadius(Actor)。身份、世代、EntityActors、隱藏、有限點與位置、最近候選，以及下游陣營、距離、魔力規則都不改。沒有硬編碼半徑、新碰撞、場景 overlap、測試用強制目標或新 API。盤點仍是 112／53／35，frame 與時限未放寬。content 27、codegen 41、Rust 99、Lua 24／44 不需重跑。

E510 實測：ordinary owned-ability-build-v4 nocache 通過。fresh owned-ability-viewport-v3 的 native.json 標頭是 success=false、authority_ack=false、catalog_compatible=true、address 127.0.0.1:9361、startup source 11／epoch 1、target 21／epoch 1。前 12 個正向施法已寫入 intents 31–42，全部 authority_ack=false、source 11／epoch 4、envelope target 21／epoch 4、published_sequence 13：Q lumen_bolt index 0 catalog 9 unit（31–33），W lumen_touch index 1 catalog 10 none（34–36），E lumen_lance index 2 catalog 11 unit（37–39），R lumen_mend index 3 catalog 12 none（40–42）。q-esc-cancels 已過。已進入 62 案後在 q-right-cancels 失敗，見 E511。control-history 仍是 11 個 stage，最後 owner_cast_ready。cleanup listener_released=true、authority=false。publisher、editor、profile 清理通過。112／53／35 仍未接受。native 59-v2 還沒有因 viewport-v3 重跑。完整驗收最後。

### E511 — q-right-cancels 無法瞄準地面

owned-ability-build-v4 通過之後，fresh owned-ability-viewport-v3 在進入的第 62 案失敗。ue.log 幀 435、OmOwnedViewportIpcTests.cpp(4217)：q-right-cancels could not aim ground。42 intents、11 stages、catalog_compatible=true。沒有 q-right 的 Move。cleanup 是 shutdown_file，listener_released=true，authority=false，127.0.0.1:9361。沒有作用中的 Editor、build 或 publisher。authority_ack=false，games 0。112／53／35 未接受。47／30／10 前綴不變。kIntentFrames 180、kAbsentFrames 90、frame 20000、OPERATION_MS 860000、publisher 900s 不變。

同一次執行裡，前綴 cast-right-cancels 也是在 Q 預覽之後呼叫同一個 AimGround，而且已經通過，所以不能把預覽 overlay 單獨當成原因。前 12 個正向施法的 AimEnemy 與 E510 半徑修正已讓單位取樣成功。style 11 在子步驟 5 預覽仍開著，AimGround 對 owned hero 試八個偏移：(0,900)、(0,1600)、(1400,0)、(0,-900)、(2200,1400)、(-1400,900)、(0,3200)、(4000,0) cm。每一點 FitPresentationView(Point, 8000)、UpdateCamera、Paint、Project、WorldLeaf，再 SampleOwnedWorldCommand，要有地面點而且沒有 hostile。

E511 第一輪診斷只在測試。Aim 失敗字串帶 gate、unit／camera／controller、preview、hero、actor、focus、camera、zoom，以及 project／leaf／sample／nopoint／hostile／miss／avoid。接受條件不變。

E511 實測：ordinary owned-ability-build-v5 nocache 通過，13.93s。fresh owned-ability-viewport-v4 仍在進入的第 62 案失敗，42 wires、11 stages、catalog_compatible=true，publisher／editor／profile 清理通過，listener released。診斷是 gate=exhausted unit=1 cam=1 ctl=1 preview=1 hero=11/4 actor=0,0 focus=4000,0 camLoc=-0,0,6928 zoom=8000 project=0 leaf=0 sample=0 nopoint=0 hostile=8 miss=0 avoid=0 lastLeaf=SViewport lastHostile=21 pixel=400,300 point=4017,-15 sampleError=。八個偏移都投影到 SViewport，地面取樣成功，但 SampleOwnedWorldCommand 都把 target 21 判成 hostile。沒有改半徑，沒有放寬偏移。

E511 第二輪診斷只加既有 QueryPublishedOwnedTargetForAutomation 的兩個可選輸出，回傳選中列的 WorldCm 與 RadiusCm。選取規則當時不變。

E511 實測 v5：ordinary owned-ability-build-v6 nocache 通過，105.93s／17 actions。fresh owned-ability-viewport-v5 仍在第 62 案失敗，42 wires、11 stages、catalog_compatible=true，cleanup 通過，listener released。診斷是 gate=exhausted unit=1 cam=1 ctl=1 preview=1 hero=11/4 actor=0,0 focus=4000,0 camLoc=-0,0,6928 zoom=8000 project=0 leaf=0 sample=0 nopoint=0 hostile=8 miss=0 avoid=0 lastLeaf=SViewport lastHostile=21 pixel=400,300 point=4017,-15 sampleError= geom sel=21/4 hostile=1 team=2 admitted=2 hostileRows=1 actor=1:5494,19231 simple=0 agg=5494,19231:70,70 body=OmNativeBodyMesh:5494,19231:70,70 row=5494,19231:117288 dist=19302 parts=3 maxPart=OmNativeBodyMesh:70,70@5494,19231。列位置與當時的 actor、本體一致，simple collision 是 0，本體與 aggregate 都是 70，但已發布半徑仍是 117288。地面距列 19302，落在這個舊半徑裡。

原因：ProcessFrame 先對每個實體 DispatchFrameState。SetActorLocationAndRotation 把 actor 連同附著元件移到新權威位置，OnFrameState 在已有幀且序號沒有倒回時，把視覺本體 SetWorldLocation 回上一幀的位置。實體迴圈結束後才呼叫 ReplacePublishedDirectTargets，CapturePublishedDirectRadius 用 GetActorBounds(false)。這個聯集把留在舊位置的本體和跟 root 走的名稱板合在一起，extent 約是分離距離的一半，寫進列的 RadiusCm。SampleOwnedWorldCommand 只讀這列，不重算。本體之後才插值回來，所以失敗當下本體已是 70，列上仍留著移動那一幀的 117288。目標從車道終點 (240000, 0) 到近點 (5494, 19231) 的一半跨距就是這個量級。正的 simple collision 仍沿用根元件半徑。

E511 source修復：simple collision 為 0 時改呼叫 AOmUnitActor::GetSelectionBodyRadiusCm。它走既有 GetNameplateMesh，也就是 VisualInterpolationComponentName 的視覺元件，沒有則用 native 本體，取該元件自己的 CalcBounds 世界 XY extent。不聯集 actor 元件，不含名稱板、cue、VFX，也不把插值位移算進尺寸。authored scale 與 rotation 留在元件 transform 裡。正的 simple collision 路徑不變。身份、世代、公開、有限、最近與下游陣營、魔力、距離都不改。沒有每指令重算、沒有全域夾限、沒有改 AimGround。新 native 案 Om.Runtime.PreciseWorldCommand.BodyRadius 用合成幀走正式列捕捉：放大本體、遠距移動讓插值把本體留在舊處、名稱板留在 actor、再掛一個遠處裝飾元件；aggregate 變大時，發布半徑仍等於放大後的本體，本體外 20000 cm 的空地沒有目標，本體內仍選中該單位。舊測試保留。viewport 盤點仍是 112／53／35，47 前綴不變。動態 native 由既有 59 加上這一案。kIntentFrames 180、kAbsentFrames 90、frame 20000、OPERATION_MS 860000、publisher 900s 不變。content 27、codegen 41、Rust 99、Lua 24／44 不需重跑。之後避免在發布列上保存跨元件的暫時 bounds。

E511 實測：來源審查 owned-ability-latched-radius-review-v1 對照 writer run-muzt2yx2-hzqitk，terminal 10m18s／22 calls，判定與正式事件一致。ordinary owned-ability-build-v7 nocache 通過，90.84s／24 actions。fresh owned-ability-viewport-v6 進入 63 案、43 wires、11 stages，catalog_compatible=true，cleanup 全部通過，listener released。q-right-cancels 通過並寫出 Move，E511 在這次實際事件裡已解決。下一案 q-ui-cancels 失敗，見 E512。外層 workflow 在 process.wait_owned 失敗，見 E513。動態 native 60（含 BodyRadius）尚未跑，不能寫成通過。authority_ack=false，games 0。112／53／35 仍未接受。47／30／10 前綴不變。沒有作用中的 Editor、publisher 或 build。工具 session 8142 已關閉。完整驗收最後。

### E512 — q-ui-cancels 把 HUD 點擊當成取消

viewport-v6 在 q-right-cancels 通過之後，下一案 q-ui-cancels 失敗。預覽在公開 SButton 左鍵之後仍保持 90 幀。這是 native artifact 失敗。Editor 已寫 native report 並 RequestExit。production FOmCombatAimModel::Pointer 對 Ui 固定回 NoAction，只有 World 點擊會確認或取消。已接受的 Om.Runtime.CombatAim.UiMinimap 中，IgnoreUiLeft 與 IgnoreUiRight 都要求原預覽留下。使用者計畫是 HUD 點擊不確認，也不是取消。

E512 source修復：不改 aim model，不拿掉斷言。未接受的 style 12 改名 q-ui-does-not-confirm。實際 UI 左鍵之後，原預覽必須仍在，而且嚴格 kAbsentFrames 窗口內沒有 AbilityCast。然後真正的 Escape 按下並跨過下一次 World.Tick，WaitPreviewClear 才接受取消。原 Q 的 KeyUp 留在取消之後的下一步，PollExpect(None) 再看一個 kAbsentFrames，證明這次放開沒有 AbilityCast。滑鼠同步、鍵盤跨 tick 的 E497／E504 時序不變，沒有新的 token API，也沒有改 Q 的 release ownership。q-focus-cancels 與 q-modal-cancels 仍符合 FocusLoss 與 modal context clear，沒有改。後續 indicator 案沒有把 HUD 點擊當成取消。盤點仍是 112／53／35，47 前綴不變。kIntentFrames 180、kAbsentFrames 90、frame 20000、OPERATION_MS 860000、publisher 900s 不變。

### E513 — owned wait 在退出期間重查 image 碰到 Access Denied

同一次 viewport-v6 的外層 workflow 失敗。tools/lua/lib/host.lua:73 回報 cannot query exact process identity，Access is denied（os error 5）。呼叫點是 scripts/moba_owned_viewport_ipc.lua 的 process.wait_owned。Editor 已寫 native report 並 RequestExit，之後的 cleanup identity 核對通過。沒有作用中的 Editor、publisher 或 build。

原因：wait_owned 每 50ms 重開 PID 並重查 executable 與 creation token。lua-host 的 settle_owned_query 已用同一個 handle 等最多 1000ms。程序仍活著時，ERROR_ACCESS_DENIED 是失敗關閉，不是退休。退出若超過這 1000ms 仍未 signal，迴圈就把查詢錯誤當成整次等待失敗。

E513 source修復：不把 AccessDenied 當成退休，不拉長 settle，也不 pcall 吃掉查詢錯誤。新增有界 native operation wait_owned，參數是 pid、executable、creation token、timeout。程序還活著時只開啟、查詢、核對身份一次，之後只等那個已核對的 handle。已 signal 的同一個 handle，或 OpenProcess 的 ERROR_INVALID_PARAMETER（不存在的 PID），才是退休。token 或 executable 不符是錯誤，不是換一個程序繼續等。timeout 回 exited=false。查詢或 wait 失敗仍是錯誤。沒有依 raw PID 終止，也沒有放寬 PID 重用。Lua process.wait_owned 改呼叫這次 operation，回傳 boolean，預設 timeout 仍是 5000，呼叫端傳入的 shared deadline 原樣轉交，時鐘倒退仍拒絕。沒有 PowerShell、Python 或 shell fallback。workflow supervisor 的 job handle 沒有改。Windows child 測試覆蓋退出、timeout、token 不符、executable 不符，以及已 signal 或 PID 消失。既有 settle 測試與 Lua 契約仍要求活著的 access denial 不是退休。

E512／E513 實測：來源審查 owned-ability-ui-wait-review-v1 對照 writer run-muztn4hy-u8e5qd，terminal 11m11s／34 calls。HEAD 未改，diff check 0。owned-wait-repair-v1 的 host tests 23、cargo check all targets，以及 Lua owned wait、live identity、graceful stop、saved identity、viewport contract 通過。ordinary owned-ability-build-v8 nocache 通過，12.11s。fresh viewport-v7 進入 67 案、43 wires、11 stages，catalog_compatible=true，cleanup 全部通過，listener released。q-ui-does-not-confirm、q-focus-cancels、q-modal-cancels、q-stale-release 通過。外層 workflow 正確回報 native failure，沒有 process identity error。E511 的 q-right-cancels、E512 的 UI 契約、E513 的 retained wait 都在這次路徑通過。下一案 shift-q-plain 沒有 AbilityCast，見 E514。動態 native 60 尚未跑。authority_ack=false，games 0。112／53／35 仍未接受，不是全案通過。沒有作用中的 Editor、publisher 或 build。工具 session 75024 已關閉。

### E514 — quick press 讀到舊的 observed pointer

viewport-v7 在 UI、focus、modal、stale release 通過之後，shift-q-plain 失敗，沒有 AbilityCast。診斷是 preview=0 accepted=0 id=0 error=Cast request was rejected label=CastQ slot=0 cmdPlayer=1 cmdKind=0 cmdTarget=0/0 rev=52 hero=11/4 view=4 target=21/4 pixel=400,300 cast=1 unit=1:21/4 point=1:5512,19216 castError= world=1 hostile=1:21 point=1 worldError=。失敗後用測試保存的 PendingPixel 取樣，候選點合法。那不是按下當下的游標。

原因：style 2、8、16 在按下前只把 AimEnemy 或 AimGround 的結果放進 PendingAbsolute／PendingPixel，沒有送出移動。KeyDown 只做焦點與 ProcessKeyDownEvent。Down 才先 MovePointer。HandleCastChord 的 quick 路徑呼叫 MakeCastPointerSample(nullptr)，再 ReadOwnerPointerPosition，讀的是 FOmOwnedPointerTracker 上一次真正的滑鼠事件。HandleMouseMoveEvent 在 ProcessMouseMoveEvent 當下寫入樣本，player tick 才消化 BindKey。UI 點擊之後 tracker 留在按鈕上。較早的 quick 會過，是因為前一個 normal 點擊剛好留在同一個位置。Indicator 的 KeyUp 同樣用 MakeCastPointerSample(nullptr)，substep 7 原本也沒有 MovePointer。NoTarget 與 SelfCast 不讀世界游標。

E514 source修復：只改 viewport 測試順序。style 2、8、16 先安裝一次 cast mode，再用有界 substep 瞄準，成功那一幀先 MovePointer，再按原本的鍵。style 16 按的是未修飾 Q；E515 移除了誤加的 Shift。沒有每幀重寫 profile，也沒有多插一個空 tick，避免瞄準像素再被模擬走掉。style 3、9、17 在原本的 KeyUp 之前補上同一個 MovePointer。其他鍵不移動游標，避免蓋掉 UI、focus、cancel 與修飾鍵。沒有寫 tracker、沒有私有游標、沒有改 production。E496 的地面偏移、E504 的下一個 tick 才觀察鍵盤、E497 的 Escape 跨 tick 都保持。當時宣告盤點是 112／53／35，47 前綴不變。動態 native 仍是 60。kIntentFrames 180、kAbsentFrames 90、frame 20000、OPERATION_MS 860000、publisher 900s 不變。content 27、codegen 41、Rust 99、host 23 沒有因這次僅測試的改動而失效。

E514 實測：ordinary owned-ability-build-v9 nocache 通過，12.92s。fresh viewport-v8 進入 69 案、46 wires、11 stages，catalog_compatible=true，cleanup 全部通過，listener released。shift-q-plain 與 shift-q-indicator 通過。不是全案通過。下一案 shift-q-modifier-up 多寫一條 AbilityCast，見 E516。動態 native 60 尚未跑。authority_ack=false，games 0。47／30／10 前綴不變。沒有作用中的 Editor、publisher 或 build。工具 session 19234 已關閉。

### E515 — 審查攔下 plain Q 案例誤加 Shift

Grok E514 修正 run-muzu6a9x-3it8o4 已 terminal，尚未 build／run。Primary 獨立審查發現新 Substep32 對 Style16 加入 LeftShift down 與 Shift modifier，但該列是 shift-q-plain：設定 QuickWithIndicator override 後按未修飾 Q，必須驗原本 Quick 仍直接施法。Style17／18 才是實際 Shift gesture。誤加 Shift 會測到另一個操作且預期立即施法不一致。

Primary 在任何新建置／執行前移除 Style16 的 Shift down，三個 on-press cursor 案例一律 MovePointer 後 KeyDown(Key, ModNone())；Style17／18 保持原有 Shift。同步更正 worker report 的誤述。record-quick-pointer-modifier-review.lua 已執行一次。E514 真實游標事件修正保留。這是審查攔截的 source 錯誤。

E515 實測：ordinary owned-ability-build-v9 nocache 通過，12.92s。fresh viewport-v8 進入 69 案、46 wires、11 stages，catalog_compatible=true，cleanup 全部通過，listener released。shift-q-plain 與 shift-q-indicator 通過。下一案 shift-q-modifier-up 發出額外 AbilityCast。最後三條 wire 44／45／46 都是 AbilityCast index 0 lumen_bolt catalog 9 unit，target 21，source 11／4，envelope target 21／4，view 4，authority_ack=false。見 E516。動態 native 60 尚未跑。當時宣告盤點仍是 112／53／35，不是全案通過。47／30／10 前綴不變。沒有作用中的 Editor、publisher 或 build。工具 session 19234 已關閉。避免再犯：以案例列的輸入與預期語意判斷 modifier，不能由名稱包含 shift 就推論按鍵要帶 Shift。

### E516 — Shift 放開不是取消，原 Q token 仍要提交

viewport-v8 在 shift-q-plain 與 shift-q-indicator 通過之後，shift-q-modifier-up 失敗。當時宣告盤點沒有這條 wire，所以多出來的 AbilityCast 被當成額外 intent。最後三條 wire 44／45／46 都是 AbilityCast index 0 lumen_bolt catalog 9 shape unit，target 21，source 11／4，envelope target 21／4，view 4，authority_ack=false。進入 69 案、46 wires、11 stages，catalog_compatible=true。cleanup 通過，listener released。動態 native 60 尚未跑。games 0。沒有作用中的 Editor、publisher 或 build。工具 session 19234 已關閉。

原因：style 18 在 substep 5 放開 LeftShift，substep 53 立刻 KeyUp Q，然後 substep 9 用 PollExpect(None)。HandleCastKeyUp 只核對原鍵與 binding slot，取出已存的 CastHoldToken 與 CastHoldChord，再 CombatAim.Release(Context, Token, 目前公開游標)。當下的 modifier bits 不參與 release。resolved mode 在 Press latch。QuickWithIndicator 對同一 token 的 Release 走 CommitPending。使用者計畫是可設定的 Shift 模式獨立於 queue，不是 Shift release cancel。既有 native override 與 original token 測試已走這條路徑。production 在原 Q 放開時提交是對的；測試把這次提交當成失敗。

E516 source修復（尚未重跑，不能寫成通過）：不改 production，不加 cast mode，不在 Shift 放下時取消。style 18 仍先按 Shift，再按帶 LeftShift 的 Q。只放開 Shift 之後，嚴格 kAbsentFrames 內 wire 數必須停在 WatchLines，而且 IsCastPreviewActive 保持原 Q 預覽。這個窗口不放開 Q，也不多送一次 KeyUp。窗口結束後用既有 AimEnemy，在有界 kIntentFrames 內瞄準公開敵人，成功那一幀 MovePointer，再 KeyUp 原本的 Q（ModNone，Shift 已經起來）。下一次 World.Tick 由 PollAbilityWire 接受恰好一條 Q unit AbilityCast。接著既有 substep 9 的 PollExpect(None) 再看一個完整 kAbsentFrames，沒有第二條。style 18 不加進 bUnit、bIndicator、bSubmitOnPress。沒有全域移動游標。Q 的 Quick 與 override QuickWithIndicator 維持原列。AbilityCast 沒有 queue 欄位。沒有新的 token API。E515 的 style 16 仍是 ModNone 的 plain Q。

盤點因這條正向施法合法加一條預期 wire：112 案、54 intents、35 stages。47／30／10 前綴不變。C++ 案例名不變。Lua expected_cases 的 shift-q-modifier-up 帶 AbilityCast。合約測試 declared_wires 由 53 改為 54，並在 shift-q-indicator 與 alt-w-self 之間插入 lumen_bolt catalog 9 index 0 unit。history 仍是 35。kIntentFrames 180、kAbsentFrames 90、frame 20000、OPERATION_MS 860000、publisher 900s 不變。

Primary 接著跑受影響的 Lua owned viewport contract，再 ordinary owned-ability-build-v10 nocache，fresh viewport-v9。正向批次通過之後才整合 native 60，含 render。不要重跑 content 27、codegen 41、Rust 99、host 23。完整驗收最後。Minimap 在 owned ability 接受之後。已備的 minimap packet 要等這次真正接受後，才把預期前綴改成新的 54。

避免再犯：Shift 放開不是 cancel。原 Q 的 indicator token 在原鍵放開時提交，與當下 modifier 無關。不要為了讓負向窗口通過而刪案例、縮短窗口，或改 production 去取消。

### E517 — 新 ability 子案例漏保存後續 control history

ordinary owned-ability-build-v10 nocache PASS36.28s。fresh viewport-v9 的 native success=true，112 cases／54 intents 全部完成，最後 r-aid-neutral-refuses，catalog compatible。最終 control.ack.json 是 accepted sequence35／owner_support_neutral，source11/6、target21/6。可是 control-history.jsonl 只有11列，最後 owner_cast_ready。外層 strict Lua verify_artifacts 在 stage count mismatch 拒絕整批。publisher／editor／profile cleanup 全過、listener released，authority_ack=false、games0。尚未接受112／54／35，native60也尚未跑。

根因是 AppendHistory 的 bAckCopied 只由 Next(EStep) 重設；新65個 ability 子案例使用 FinishAbilityCase／AbilityCursor 而不是 Next，因此第一個 ability stage 抄錄後所有後續請求都沿用 true。PollAck 仍逐次核對 sequence／stage／run id 並 AcceptStage，故實際 publisher 狀態已到35，但證據歷程漏列。

Primary 在 WriteControl 每次遞增 ControlSequence 時重設 bAckCopied=false，讓去重旗標以請求為生命週期。讀取重試仍同一請求只抄錄一次，舊 Next reset 無害保留；沒有重建或補寫舊歷程，沒有略過 Lua 35階段核對。112／54／35與所有時限不變。避免再犯：證據去重與 retirement flag 必須跟實際 request token／sequence 綁定，不能依靠某一種外層狀態機的 Next 呼叫。

E517 實測：ordinary build-v11 通過。fresh owned-ability-viewport-v10 通過，112 案、54 intents、35 個真實 control stage，catalog compatible，cleanup 與 listener release 通過。接著 owned-ability-native-v2 跑 60，見 E518。尚未接受。

### E518 — BodyRadius 合成幀沒有先記下視覺樣本

owned-ability-native-v2 的 index.json 是 succeeded 58、succeededWithWarnings 1、failed 1、notRun 0。失敗案是 Om.Runtime.PreciseWorldCommand.BodyRadius。錯誤是 Expected 'Interpolation left the body behind the actor' to be true，OmActionOwnerTests.cpp:2749。該案 warnings 0、errors 1。縮放斷言在這行之前已經通過。尚未接受。

SuccessWithWarnings 是 Om.Runtime.SummonerPayload。唯一 warning 是 LogRHICore: Total reserved resource allocated virtual size (258 GB) exceeds the budget (256 GB)。native-engine.log 在該案 CleanupWorld 時寫出這行，automation 把它記到這個案上。該案 state 是 Success、errors 0。這是這次 D3D12 工作階段累積的虛擬位址保留超過 256 GB，不是召喚物斷言，也不是本體半徑。不要關掉 warning，不要改 engine 或 RHI 設定。這次修復不處理它。若虛擬保留之後持續變大，用 log 指名的 rhi.DumpMemory 與 rhi.DumpResourceMemory 另查。

原因：DispatchFrameState 先 SetActorLocationAndRotation，再 OnFrameState。OnFrameState 只有 bHasVisualProxySample 為真時才恢復 LastVisualProxyWorldLocation；否則讀取的是已經跟著 actor 搬走的元件位置。那個樣本只由公開 Tick 進入 TickVisualInterpolation 時寫上。BodyRadius 連續送 tick 1、2、3 的合成幀，中間沒有 Tick。長移動是 4000 world unit，WorldUnitsToCm 仍是 100，也就是 400000 cm，但視覺元件已經在新位置，所以距離不會大於 100000。既有 Om.Runtime.NativeAndAuthoredMeshCompatibility 是先 OnFrameState，再 Actor->Tick(1/60)，然後才移動。viewport 已證明一般本體半徑；不改 GetSelectionBodyRadiusCm 或 CapturePublishedDirectRadius 來遷就這個合成排列。

E518 source修復（尚未重跑，不能寫成通過）：裝飾網格註冊之後、Publish(4000, 3) 之前，呼叫 Hostile->Tick(1/60)。下一個合成幀仍走 ProcessSyntheticFrameForAutomation。不寫 private interpolation 欄位，不注入已發布列。縮放、長移動、裝飾與名稱板造成的聚合位移、本體半徑對聚合、本體內選中、本體外 20000 cm 不選，全部保留。動態 native 仍是 60。時限與盤點不變。production 行為不變，不要為這次重跑 viewport。

檢查失誤：Primary 讀了不存在的 .ai-collab/moba-owned-ability-modifier-release-task.md。實際報告是 .ai-collab/moba-owned-ability-modifier-release-repair-report.md。另一次把過大的 state 整段印出。之後先用目錄裡的實際檔名，再只讀 index.json 的 state、fullTestPath、warnings、錯誤訊息與行號，以及 engine log 對應的那幾行。

避免再犯：要證明插值把本體留在 actor 後面時，必須先走公開 Tick 記下移動前的樣本；合成幀本身會先搬走 actor。

E518 實測：ordinary owned-ability-build-v12 nocache 通過，14.65s。owned-ability-native-v3 是 succeeded 58、succeededWithWarnings 1、failed 1、notRun 0。BodyRadius 的插值、名稱板、聚合、縮放後本體半徑都已通過。下一句 Expected 'Published radius is the scaled body, not the aggregate' 失敗，OmActionOwnerTests.cpp:2767。後面的圈外選擇沒有再報錯誤。見 E519。viewport-v10 的 112／54／35 仍是通過。production 未改。尚未接受。SummonerPayload 的 LogRHICore 258 GB／256 GB 仍在，不關閉、不改 engine。

### E519 — 裝飾網格的預設碰撞蓋過本體半徑

native-v3 的 BodyRadius 在公開半徑斷言失敗。index.json 只有這一條 Error：Expected 'Published radius is the scaled body, not the aggregate' to be true，檔案 OmActionOwnerTests.cpp 行 2767。該案 warnings 0、errors 1。插值距離、名稱板、聚合、GetSelectionBodyRadiusCm 等於縮放本體都已通過。圈外 20000 cm 的選擇斷言沒有失敗。

原因：CapturePublishedDirectRadius 在 GetSimpleCollisionRadius 為有限且大於 0 時直接用它，否則才用 GetSelectionBodyRadiusCm。GetSimpleCollisionRadius 走 GetSimpleCollisionCylinder。SceneRoot 是 USceneComponent，GetCollisionEnabled 是 NoCollision，IsRootComponentCollisionRegistered 為假，所以改走 GetComponentsBoundingCylinder，而且 bNonColliding 是 false，只收已註冊且 IsCollisionEnabled 的 primitive。原生本體與名稱板都設了 NoCollision。新的 UStaticMeshComponent 建構子呼叫 SetCollisionProfileName(BlockAllDynamic)。BaseEngine.ini 的 BlockAllDynamic 是 CollisionEnabled=QueryAndPhysics。測試沒有關掉裝飾網格的碰撞，註冊後它成為唯一的正向簡單碰撞，發布半徑就不再是縮放後的本體。這是既有的正向簡單碰撞優先，不是 publication 寫錯。不改 production。

E519 source修復（尚未重跑，不能寫成通過）：裝飾網格在 RegisterComponent 之前 SetCollisionEnabled(NoCollision)，和視覺用的原生本體一致。註冊後以及 Publish 之後各断言一次 GetSimpleCollisionRadius 仍為 0。縮放、插值、聚合、本體半徑、圈內選中、圈外 20000 cm 不選都保留。不寫 private 列，不放寬斷言。60 案、112／54／35、時間預算不變。

檢查失誤：Primary 在 native 程序還在跑時就讀結果，summary 不存在。另一次對 native-v2 呼叫截圖比較 helper，正確的 guard 拒絕了。之後要等該次執行 terminal，再讀 summary 或比較截圖。

Primary 接著跑 ordinary build-v13，然後 native-v4 with-render。production 未改，不要重跑 viewport。完整驗收最後。Minimap 等 owned ability 接受之後。LogRHICore 警告保持原樣。避免再犯：視覺用的附加網格若要走本體半徑，必須明確 NoCollision；新的 UStaticMeshComponent 預設就是 BlockAllDynamic。


Owned ability scoped acceptance：ordinary owned-ability-build-v13 nocache PASS；D3D12 owned-ability-native-v4 exact60 Success／0errors／0notRun；含warning的測試數1，完整warning entries留在accepted summary，不隱藏或改寫原報告。fresh owned-ability-viewport-v10 exact112cases／54intents／35stages全部通過，包含QWER normal／quick／indicator、原鍵release、取消／focus／modal／UI不確認、modifier-first release、升級／self／allegiance／隱藏／死亡／中立拒絕。新增body radius regression驗證實際frame publisher，避免nameplate／裝飾／插值位移放大選取。11張widget PNG的實際目視或byte-identical沿用詳見visual-review.json。原程序退休、listener釋放、profile還原皆過。accepted .ai-collab/owned-ability-accepted-v1/summary.json；renderer IPC local admission，不是authority ACK。完整驗收尚未跑，quota10/10不變，HUD可讀性與其餘計畫仍待完成。

### E520 — 小地圖版面 fallback 呼叫了不存在的 OuterUsedFallback

這次是寫 `UOmCommandBarWidget::RefreshClusterScales` 右側平移時的來源錯誤。沒有跑 Lua、ordinary build、native 或 viewport，不能寫成測試失敗。

原因：fit 迴圈沒有成功時，`Apply` 的第一個參數寫成 `OuterUsedFallback()`。那個函式不存在。同一函式裡已經有區域變數 `SmallOuter`，前面的 fit 嘗試也用它當最小外框。

E520 source修復（尚未跑，不能寫成通過）：改成 `Apply(SmallOuter, FallbackCombatW, true, FallbackInset)`。左側預設、同一顆 `SOmMinimap`、時間預算，以及已接受的 112／54／35 前綴都沒有為了這個錯誤放寬。

避免再犯：延伸這個 fit 迴圈時，fallback 的外框必須用該函式已經存在的 `SmallOuter`，不要再造一個 helper。

### E521 — 小地圖偏好第一次編譯失敗，右側點擊尚未證明

minimap-preferences-build-v1 ordinary 編譯失敗，exit 1。日誌在 `.ai-collab/minimap-preferences-build-v1/{build.log,UBT-Log.txt,summary.json}`。兩份 cpp 都以 exit 2 結束，沒有 native、viewport 或 PASS。

`OmPlayerActionOwnerComponent.cpp` 4051：`const UOmRuntimeSettings* Settings = GetDefault<UOmRuntimeSettings>();`。C4430、C2143、C2065、`GetDefault` C2672，下一行 `Scale` C2737 是連鎖。原因是新的 `SubmitMinimapGroundPointer` 使用 `GetDefault<UOmRuntimeSettings>()` 與 `WorldUnitsToCm`，這個 cpp 沒有直接包含公開標頭 `OmRuntimeSettings.h`。`UOmRuntimeSettings` 就在該標頭。同一份日誌裡 `OmOwnedViewportIpcTests.cpp` 3966 的區域 `World` 遮蔽 session 成員，4016 的 `Cases` 遮蔽 `TArray<FCaseRecord> Cases`，兩者都是 C4458 並且讓該編譯 exit 2。`SOmMinimap.cpp` 38 的 `ReleaseMouseCapture` 只有 C4996，編譯沒有因此失敗；UE 5.8 註明改名 `ReleaseAllPointerCapture()`，那個 API 會放開全部 capture，這次不改。

E521 source修復（尚未重跑 build、native、viewport，不能寫成通過）：`OmPlayerActionOwnerComponent.cpp` 直接包含 `OmRuntimeSettings.h`。viewport 的區域變數改名 `OffsetWorld` 與 `MinimapCases`，121／58／36 與九個案名不變。`Om.Runtime.MinimapPreferences` 在還原左側之前，用 `LocateWindowUnderMouse` 打實際右側絕對座標，要求葉節點是同一顆 `SOmMinimap`，再沿這條路徑送左鍵按下、拖曳、放開；公開 pawn 與 spring arm tick 之後，焦點對未投影點，有效相機距離對 `TargetArmLength`。這段不送移動、不施法、不假裝 Connected。左側原檢查留在還原之後。

同一次 Primary 的商店勘察曾猜不存在的 `omoba-core/src/player_input.rs`，rg 回 os2。改用 rg --files 後的實際檔是 `runtime/native/tick/player_input_tick.rs`。細節在 `.ai-collab/moba-shop-source-baseline-2026-10-09.md`。這裡不改商店。

避免再犯：新函式用到的 UCLASS 要在該 cpp 直接包含既有公開標頭。區域名稱不要遮蔽 fixture 成員。先用實際檔名，再搜尋。

### E522 — 小地圖在地圖不可用或本地玩家更換時留下滑鼠 capture，退休又會放開全部使用者

這次是讀來源的發現，不是已執行的 native 或 viewport 失敗。minimap-preferences-build-v1 對 `ReleaseMouseCapture` 只有 C4996 警告。include／right-hit 修復沒有改這條呼叫，也還沒有重跑 build、native、viewport。

原因：`SOmMinimap::SetState` 在 `!InState.bAvailable` 或 `LocalPlayer != InLocalPlayer` 時只呼叫 `EndInspect`。`EndInspect` 在 `bInspectActive` 為 false 時直接返回；否則把旗標清掉並通知 `OnCameraInspectEnded`。它不放開 Slate capture。`UOmCommandBarWidget::RetireMinimapPointerIfStale` 在 `!Minimap->IsInspecting()` 時直接返回，所以 inspect 被清掉之後，tick 接不住留下的 capture。

`SOmMinimap::RetirePointerCapture` 當時呼叫 `FSlateApplication::Get().ReleaseMouseCapture()`。`D:/UE5.8/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp` 2838–2860：`ReleaseMouseCapture` 轉呼叫無參數 `ReleaseAllPointerCapture()`，`ForEachUser` 對每個使用者 `ReleaseAllCapture()`。公開的 `ReleaseAllPointerCapture(int32 UserIndex)` 仍是該使用者的 `ReleaseAllCapture()`，會清掉該使用者其他 pointer。公開且只放一個 pointer 的 API 是 `FSlateApplication::GetUser(int32)` 加上 `FSlateUser::DoesWidgetHaveCapture` 與 `FSlateUser::ReleaseCapture(uint32)`。`SWidget::HasMouseCaptureByUser(int32, TOptional<int32>)` 是公開查詢。`GetOrCreateUser(int32)` 在 `SlateApplication.h` 1180 之後是 protected。測試能用的公開註冊是 `FindOrCreateVirtualUser(int32)`。`FSlateVirtualUserHandle` 解構時只對自己的 index 呼叫 `UnregisterUser`。`SetPointerCaptor` 在寫入新 capture 之前會先 `ReleaseCapture` 同一個 pointer，並在移除 map 之後才呼叫 `OnMouseCaptureLost`。

E522 source修復（尚未編譯或執行，不能寫成通過）：真正 `CaptureMouse` 的手勢記下 `Event.GetUserIndex()` 與 `Event.GetPointerIndex()`。同一個 user／pointer 已經由本 widget 持有時不再次要求 capture，避免 Slate 先放開再抓住時把記錄清掉。放開回覆、相符的 `OnMouseCaptureLost`，以及退休之前先清這筆記錄。退休只在 `GetUser` 有效且 `DoesWidgetHaveCapture(this, pointer)` 時呼叫 `ReleaseCapture`。`SetState` 的原條件改走這個退休，不另做逾時輪詢。`EndInspect` 在 release 回呼裡仍只通知一次。不推斷 user 0，不寫 private capture 旗標，不掃 controller 或 world。`FReply::Handled().ReleaseMouseCapture()` 留在左鍵放開，而且只在 `HasMouseCaptureByUser` 對該事件為真時送出。

`Om.Runtime.MinimapPreferences` 仍是同一個測試。右側 `LocateWindowUnderMouse` 證明留在還原左側之前。還原左側、modal、文字、profile、移動檢查之後，按下實際小地圖，把狀態設成不可用，要求 capture 與 inspect 都清；還原後再按下，換成另一個本地玩家，兩者再清，然後把原玩家與可用狀態還原。隔離使用公開 `FindOrCreateVirtualUser(31)` 與另一個 `SLeafWidget`，另用同一個手勢使用者的下一個 pointer index。退休小地圖時這兩份無關 capture 必須還在。清理只放開這兩個 probe 自己的 pointer，然後丟掉 virtual handle。不呼叫 `UnregisterUser` 去清測試開始前就存在的使用者。若虛擬使用者與手勢使用者相同，或 probe 沒有經公開路由拿到 capture，斷言失敗，不做成假的隔離通過。設定頁 `MinimapPingDeferred` 的句子改成「戰術標記尚未開放。」標籤與未接上的 ping 不變。沒有新增測試，原生仍是 63，viewport 仍是 121／58／36。同一函式裡移動放開的區域事件原名 `RightUp`，與右側證明的 `RightUp` 衝突，改名 `MovementRightUp`，檢查不變。

避免再犯：不要呼叫無參數 `ReleaseMouseCapture` 或 `ReleaseAllPointerCapture`。不要用 `ReleaseAllPointerCapture(UserIndex)` 代替單一 pointer。手勢沒有記下的 user 不要猜 0。`SetState` 清 inspect 時要一起退休本 widget 的 capture，不能靠 `IsInspecting` 為 false 的 tick 補放。

### E523 — 右側小地圖命中把 SWidget 參考和 SOmMinimap 指標拿來比較

minimap-preferences-build-v2 ordinary、nocache，exit 1。日誌在 `.ai-collab/minimap-preferences-build-v2/{UBT-Log.txt,build.log,summary.json}`。`summary.json` 的 `exit_code` 是 1。Native 與 viewport 沒有跑。Production 的 `SOmMinimap`、action owner、command bar、settings 這次有編成。失敗的編譯單元是 `OmCameraInteractionTests.cpp`。

`UBT-Log.txt` 實際錯誤只有四筆，都在這個檔：506 欄 109 C2678，506 欄 7 C2661 `FAutomationTestBase::TestTrue`；507 欄 116 C2678，507 欄 7 同樣的 C2661。C2661 是 C2678 讓條件式沒有 bool 之後的連鎖。

原因：`FWidgetPath::GetLastWidget()` 在 `D:/UE5.8/Engine/Source/Runtime/SlateCore/Public/Layout/WidgetPath.h` 170 回傳 `TSharedRef<SWidget>`。`TSharedRef::Get()` 回傳 `SWidget&`，`Minimap.Get()` 回傳 `SOmMinimap*`。`RightHit.GetLastWidget().Get() == Minimap.Get()` 與下一行的 `RightDragHit` 是參考對指標，沒有可用的 `operator==`。`IsValid`、葉節點型別 `SOmMinimap`、以及同一實例的要求都還在。

E523 source修復（尚未重跑 build、native、viewport，不能寫成通過）：兩處改成 `&RightHit.GetLastWidget().Get() == Minimap.Get()` 與 `&RightDragHit.GetLastWidget().Get() == Minimap.Get()`。`Get()` 的位址是那個 `SWidget` 物件，和 `SOmMinimap*` 可以比。`IsValid()` 仍在 `GetLastWidget()` 之前，短路由過時才呼叫。沒有拿掉斷言，沒有改版面或 capture。63／121／58／36 與時間預算不變。

檢查失誤：Primary 先要了整份 build stdout，輸出在 39198 tokens 被截斷，然後才用 rg 抽錯誤。完整日誌仍在上述目錄，實際錯誤都已看過。更早一次 Primary 猜了 `omfue/Source/OmFrontend/Private/SOmMinimap.cpp`，rg 回 Windows path error 3，記在 `.ai-collab/primary-inspection-path-error-2026-10-09.md`。那次沒有改來源，也不是這次編譯失敗。

避免再犯：`GetLastWidget().Get()` 是物件參考，要和 `TSharedPtr::Get()` 比時取位址，或先證明 shared-pointer 相等可用。不要把 `TSharedRef::Get()` 當成指標。抽編譯錯誤用 rg，不要先把整份 UBT 日誌塞進上下文。先用實際檔名，再搜尋。

### E524 — native-v1 的小地圖許可、右側拖曳與 widget tick 退休

minimap-preferences-build-v3 ordinary、nocache 通過，實際 9.93s。E521 的缺標頭與 C4458、E523 的 widget 位址比較，這次實際編成。第一輪 D3D12 minimap-preferences-native-v1 終端是 63 案：60 Success、1 SuccessWithWarnings、2 Fail、0 notRun。證據在 `.ai-collab/.bridge-data/worker-pool-machine-v1/minimap-preferences-native-v1/{native-report/index.json,native-engine.log,summary.json,errors.md}`。Viewport 尚未跑。`LogRHICore` 258 GB 對 256 GB 的警告留在 `native-engine.log` 3132 與 3138，不是要壓掉的錯誤。E522 的地圖不可用、本地玩家更換、以及虛擬使用者隔離這次沒有自動化錯誤；這不表示整份 native 通過。`errors.md` 只記到契約層的 `Om.Runtime.CombatAim.MinimapGround` Fail。`native-engine.log` 還有同一輪的第二個 Fail。

這輪實際斷言錯誤有五筆，都是測試先返回或繼續留下的結果，不是生產版面公式壞掉：

- `native-engine.log` 2323，frame 185：`Om.Runtime.CombatAim.MinimapGround`，`OmCombatAimModelTests.cpp(872)`，`A minimap sample without the permit does not confirm or cancel` 預期 true。測試先武裝普通 Q，再呼叫 `Model.Pointer(Context, WorldPoint(AimPoint()))`，然後才送未許可的小地圖樣本。那個世界左鍵本來就會提交，後面的 pending token 與 `!HasCommitted()` 因此失敗。
- 2986，frame 293：`Om.Runtime.MinimapPreferences`，`OmCameraInteractionTests.cpp(525)`，`Right-side drag matches the unprojected point`。右側同一葉節點、焦點、彈簧臂已通過。左拖通過。只有右拖失敗。容差 0.01 與實際相機位置都保留。
- 2987／2988／2989，同一 frame：`Modal retires capture`、`Editable focus retires capture`、`Profile change retires capture` 預期 false，條件仍是 true。三處都只呼叫 `World->Tick`。

來源發現，不是這輪多出來的執行失敗：同一個 `MinimapGround` 裡，`CancelModel.GetPendingToken() == CancelToken` 寫在右鍵取消之後，`IndicatorModel.IsPending()` 寫在原 token 的 `Release` 提交之後。第一個斷言已經 return，這兩段沒有單獨執行到。這是 E503 的同一種順序錯誤。

右拖原因是事件路徑，不是 `MinimapCluster` 的 render transform，也不是 `OnMouseMove` 丟掉平移。按下走 `LocateWindowUnderMouse` 的命中幾何，`GetTickSpaceGeometry` 算出的絕對座標能對上未投影點，所以焦點通過。`RoutePointerMoveEvent` 在已 capture 時不使用呼叫端路徑的幾何。`FSlateUser::GetCaptorPath` 用 `FWeakWidgetPath::ToWidgetPath(..., ReturnInvalid)`。`GSlateFastWidgetPath` 預設 false，重解從 `SWindow::GetWindowGeometryInScreen()` 排列子節點。截斷會先 `ReleaseCapture`，`SOmMinimap::OnMouseMove` 在 `!HasMouseCapture()` 時回 `Unhandled`，焦點留在按下的點。左拖用 `GeneratePathToWidgetUnchecked` 的版面父鏈，重解活著，所以通過。命中幾何與 tick 幾何都含 `GetPositionInScreen()`。生產的 `OnMouseMove` 用 Slate 交來的幾何解真實游標，這次不改投影、容差、cluster 平移或 capture 退休。

Widget tick 的原因：`UOmCommandBarWidget::RetireMinimapPointerIfStale` 只從 `NativeTick` 呼叫。`SObjectWidget::Tick` 在 `SWidget::Paint` 看到 `NeedsTick` 時才呼叫 `NativeTick`。`ForceRedrawWindow` 進入 `PrivateDrawWindows`。`World->Tick` 不會畫這個 widget。Modal、可編輯焦點、profile revision 的退休條件留在生產函式。

E524 source修復（尚未重跑 build、native、viewport，不能寫成通過）：

- `OmCombatAimModelTests.cpp` 刪掉未許可樣本之前的世界左鍵。斷言只看該樣本的 `NoAction`、`!bConsumePointer`、pending token 與 `!HasCommitted()`。雙擊之後先記下 `IsPending()` 與 `GetPendingToken()`，再送右鍵。指示器點擊之後先記下 `IsPending()`，再 `Release`。取消後與提交後的狀態分開斷言。普通、快速、指示器、自身、實體、小地圖限制與舊清單都不改。
- `OmCameraInteractionTests.cpp` 的右側命中證明仍用 tick 空間的 `LocateWindowUnderMouse` 與位址比較。手勢改走 `GeneratePathToWidgetUnchecked`，指標位置用該路徑幾何的 `LocalToAbsolute`。路由後的點必須在 0.01 以內等於原來的繪製未投影點，拖曳仍比實際 actor 位置，不比 `DesiredZoomDistance`。Modal、文字焦點、profile 在條件設立後走既有的 `SlatePrepass` 加 `ForceRedrawWindow`。三處都再要求 `HasMouseCapture()` 為 false。沒有直接呼叫 `NativeTick`，沒有私有旗標，沒有只給測試用的計時器。
- 生產的 `SOmMinimap::OnMouseMove`、`HandleMinimapFocusRequested`、`RefreshClusterScales`、`RetirePointerCapture` 這次沒改。

檢查失誤：這次把多個來源段併成一次讀取，輸出在 4018 tokens 被截斷，之後才重讀 `OnMouseMove`、`NativeTick`、`GetCaptorPath`、`ToWidgetPath`、`Paint` 與命中路徑。更早的 build stdout 截斷在 39198 tokens，見 E523。避免再犯：用 rg 或只讀函式本體。不要先把整份 native log、整份 UBT log 或整份 widget cpp 塞進上下文。`World->Tick` 不能當成 `UUserWidget::NativeTick` 已執行。命中路徑的幾何不能當成 capture 重解之後 `OnMouseMove` 收到的幾何。

### E525 — 右側實際送出的螢幕座標沒有再做命中證明

這是來源覆蓋缺口，不是已執行的失敗。minimap-preferences-build-v4 ordinary、nocache 已通過。native-v2 尚未跑。沒有作用中的 Editor、publisher 或 build。E524 的來源修復仍待 native-v2 驗證。native-v1 的五筆斷言錯誤留在歷史紀錄，不因 build-v4 通過而改寫成 native 通過。

`Om.Runtime.MinimapPreferences` 的右側區塊先用 `RightAbsolute` 與 `RightDragAbsolute` 呼叫 `LocateWindowUnderMouse`，證明繪製點打到同一個 `SOmMinimap`。E524 之後，真正送進 `RoutePointerDownEvent`、`RoutePointerMoveEvent`、`RoutePointerUpEvent` 的是 `RoutedPressAbsolute` 與 `RoutedDragAbsolute`，來自 `GeneratePathToWidgetUnchecked` 的幾何。新增的世界點斷言只把 0.25／0.75 的本機分數各別未投影後再比較，沒有證明這兩個螢幕座標打到右側那顆已繪製的小地圖。

E525 source修復（尚未重跑 build、native、viewport，不能寫成通過）：在送出事件之前，對 `RoutedPressAbsolute` 與 `RoutedDragAbsolute` 各呼叫一次公開的 `LocateWindowUnderMouse`。兩條路徑都要求 `IsValid()`、`&GetLastWidget().Get() == Minimap.Get()`、型別 `SOmMinimap`，而且兩個 X 都必須大於 `LeftX + 40`。原來的繪製點命中、未投影、有效相機、0.01、capture 與版面路徑檢查都保留。沒有改生產程式。座標若在下一輪實際分叉，要修的是公開 fixture 的事件空間，不能拿掉命中，也不能只留世界點相等。

避免再犯：手勢改用另一組絕對座標時，命中證明要打在即將送出的那組座標上。世界點相等不能代替 `LocateWindowUnderMouse`。

E524 補充：當時把右拖失敗歸成「命中路徑被 `ToWidgetPath` 截斷所以 capture 被放掉」或「重解幾何丟掉 cluster render transform」，那是讀取引擎後的來源假設，不是 native 量到的座標。native-v1 只證明按下的相機通過、拖曳的相機不通過。不要把那個分類當成已測量的原因。下面 E526 的 native-v2 才第一次量到送出的絕對座標沒有打中繪製的小地圖。

### E526 — native-v2 右側實際送出的座標沒有打中繪製的小地圖

minimap-preferences-build-v5 ordinary、nocache 通過，10.88s。native-v2 終端是 63 案：61 Success、1 SuccessWithWarnings、1 Fail、0 notRun。唯一自動化錯誤是 `Om.Runtime.MinimapPreferences`，`native-engine.log` 的 `Routed press hits the same minimap`，來源 `OmCameraInteractionTests.cpp` 528。證據在 `.ai-collab/.bridge-data/worker-pool-machine-v1/minimap-preferences-native-v2/`。Viewport 尚未跑。`LogRHICore` 警告保留。`Om.Runtime.CombatAim.MinimapGround` 這次通過。native-v1 的右拖與 modal／文字／profile 退休沒有全部跑到，因為這次在命中斷言提前 return。不能把那些寫成已解決。

E525 的嚴格命中斷言有用。它抓到 E524 改寫後的事件座標和繪製座標不是同一個點。同一測試裡，`GetTickSpaceGeometry` 算出的 `RightAbsolute`／`RightDragAbsolute` 已通過 `LocateWindowUnderMouse`，葉節點是同一個 `SOmMinimap`。失敗的是 `GeneratePathToWidgetUnchecked` 幾何的 `LocalToAbsolute`。世界點把 0.25／0.75 各別未投影後比較，不能證明螢幕座標打在右側那顆已繪製的 widget 上。

已核對的程式事實，還不是像素差：`FWidgetPath::GeneratePathToWidget` 用 `ArrangeChildren`，`FGeometry::MakeChild` 會帶子 widget 的 render transform。`FWeakWidgetPath::ToWidgetPath` 在 `GSlateFastWidgetPath == false` 時從 `SWindow::GetWindowGeometryInScreen()` 重排，那個根用視窗 `Size`。繪製與命中用 `GetWindowGeometryInWindow()` 加上 `GetPositionInScreen()`，命中格的範圍是 `GetViewportSize()`。fixture 視窗關標題列、邊框為 0、`ScreenPosition` 不是原點、client 與 scene viewport 是 800×600，而 `AdjustInitialSizeAndPositionForDPIScale` 預設為 true。`SGameLayerManager` 會依 DPI 曲線改玩家層的 slot。這些能讓兩套絕對座標分開。native-v2 沒有把原點、DPI、視窗大小印出來，所以像素差仍是待測量的假設，不是已證明的某一種原因。沒有因此改 cluster 的 render transform，也沒有把右側改成特製偏移。

E526 source修復（尚未重跑 build、native、viewport，不能寫成通過）：真正送進按下、拖曳、放開的座標改回已經命中的 tick 空間點，並在送出前再對那兩個座標做 `LocateWindowUnderMouse`、同一物件位址、`SOmMinimap` 型別，以及 X 大於 `LeftX + 40`。版面路徑仍用來建立 capture。`SOmMinimap` 的按下與移動先看路由幾何是否包含該點；不包含時改用 `GetTickSpaceGeometry()` 未投影。這不是直接呼叫 `OnMouseMove`，也沒有開 `GSlateFastWidgetPath`。送出前的 `AddInfo` 印出繪製與版面兩套絕對座標、原點、本機大小、視窗位置、DPI、視窗大小、viewport，以及拖曳後的相機、預期點與 capture。0.01、彈簧臂、不移動、不施法、左拖、modal／文字／profile、隔離都保留。

避免再犯：事件座標的命中證明必須打在即將送出的那組螢幕座標。來源上的路徑分類要等 `AddInfo` 的數字，不能把未執行的機制寫成已測量的原因。


Minimap preferences scoped acceptance：ordinary minimap-preferences-build-v6 nocache PASS；D3D12 minimap-preferences-native-v3 exact63 Success／0errors／0notRun，warning entries完整保留於accepted summary。fresh minimap-preferences-viewport-v1 exact121cases／58intents／36stages通過，包含原112案例與新增小地圖移動／地點施法偏好。左右位置、實際hit-test／drag／有效camera、modal／focus／profile與資料失效／玩家切換capture退休及其他user／pointer隔離由native證據支持。11張PNG目視或byte-identical沿用見visual-review.json。原程序退休、listener釋放、profile還原皆過。accepted .ai-collab/minimap-preferences-accepted-v1/summary.json；僅renderer IPC local admission，不是authority ACK。戰術ping仍未開放，跨OS DPI／PIE完整矩陣待最後驗收，task2.7及完整計畫不因此勾完。quota10/10不變。

### E527 — ACK 替換空窗的 Windows error 2 被當成永久讀取失敗

Actual owned-summoner-viewport-v1 FAILED after150cases/68intents/44stages; last recorded case summoner-ready-3, requested control sequence45 owner_summoner_ready. Native failure: control.ack.json could not be read (os 2), OmOwnedViewportIpcTests.cpp1825. The workflow then reported that native failure, not an independent Lua missing-file failure. All publisher/editor/profile cleanup passed; listener released, authority_ack false. No new authority games. Ordinary owned-summoner-build-v1 nocache PASS313.67s and scoped owned-summoner-native-v1 exact63Success remain real prior evidence; they do not make the failed viewport pass.

Source evidence: renderer_fixture.rs write_atomic creates/syncs temporary bytes, removes old destination then renames; on Windows a destination-not-found interval exists. PollAck checks FileExists before LoadFileToString, and AppendHistory rereads the ACK. PublisherReadFailed retries only0/5/32/33, so an error2 between existence check and open fails immediately instead of consuming the existing bounded ACK wait. The observed failure proves error2, while its exact interleaving is inferred from this source; no timing trace is claimed.

Required repair: handle expected absent-during-publication reads within the existing finite ACK budget, keep permanent failures and malformed/run/sequence/stage identity rejection strict, and preserve exact accepted history. Avoid sleeps, larger budgets, fabricated ACKs, skipped cases or relaxed epochs. Record repair source and fresh results separately; current158/69/51 acceptance is still pending.

### E528 — 新增 ACK 回歸測試的區域變數 Empty 與既有宣告撞名

Actual owned-summoner-build-v2 nocache reports C2373 at OmOwnedViewportJsonlTests.cpp90: FIntake Empty conflicts with the existing FOmJsonlObservation Empty in the same RunTest scope. Cascading C2039 at91/92 reports Observe/bFailed/History on FOmJsonlObservation. Complete actual compiler diagnostics are preserved by build-v2. Primary read the whole test but missed this duplicate local name; source review was insufficient and the compiler correctly rejected it. No native-v2/viewport-v2 has run; no passing build is claimed.

Required minimal repair: rename the new FIntake local and its two references to an unambiguous ACK-intake name. Preserve all assertions/classifier/history handling and the old Empty JSONL observation. No production or budget changes. Before adding locals to a long existing test, check names across the entire enclosing scope; independent compilation remains mandatory. Wait for the active build to retire before dispatching the writer.

E528 source repair (not compiled and not rerun: no build, native, or viewport, so this is not a pass): In `OmOwnedViewportJsonlTests.cpp` `RunTest`, the new `FIntake` local is renamed from `Empty` to `EmptyAckIntake`. The `.Observe`, `.bFailed`, and `.History` uses on the next two lines use that name. The existing `const FOmJsonlObservation Empty` stays. Assertion text, `ClassifyOwnedPublisherRead`, `PrepareAcceptedPublisherRecord`, the history reader, production source, D/F cases, and budgets are unchanged. Inventory stays 63 native / 158 cases / 69 intents / 51 stages. This worker did not compile or run tests. Failed owned-summoner-build-v2 and owned-summoner-viewport-v1 remain preserved. Primary reviews and runs fresh ordinary build-v3, D3D12 native-v2, and viewport-v2. Full acceptance stays last.

E527 source repair (not rerun: no build, native, or viewport, so this is not a pass): `ClassifyOwnedPublisherRead` in `OmOwnedViewportJsonl.h` treats OS 2 as the existing bounded wait, together with the previous 0/5/32/33 set. Other OS codes stay permanent. `PollAck` opens `control.ack.json` once per attempt, with no `FileExists` gap. OS 2 still fails as "control ack was not published" only after `kAckFrames`. `AppendHistory` writes the bytes `PollAck` already validated and does not read the file again. `bAckCopied` still allows one history record per accepted control. Parse, run, sequence, stage, epoch, and `authority_ack` checks are unchanged. `write_atomic` was not redesigned. `Om.Runtime.JsonlObservation` now covers missing-then-one-record, a replacement that must not append again, permanent OS 3, and empty bytes. Automation count stays 63. Viewport inventory stays 158/69/51. Fresh build-v2, native-v2, and viewport-v2 have not been run.

2026-10-09 E527/E528 獨立重驗完成：ordinary owned-summoner-build-v3 nocache PASS20.81s，native-v2 exact63 Success／0errors／0notRun（SummonerOwnerCards RHICore258GB>256GB warning完整保存）；新viewport-v2 exact158cases／69intents／51stages、lastd-profile-cancels與全部cleanup/listener/profile還原PASS。Om.Runtime.JsonlObservation共享分類回歸已實跑；實際viewport跨過原seq45失敗點並跑完所有案例。這次沒有故意操控OS替換時序，不能聲稱每個race interleaving已量測。actual121／58／36前綴比對成功，accepted `.ai-collab/owned-summoner-accepted-v1/summary.json`。E528完整build-v2還有C2661 TestTrue cascade，所有compiler diagnostics保留。舊failedviewport-v1/build-v2與上方source-only時間點保留；新權威對局0、authority_ackfalse、完整驗收仍留最後。

### E529 — TickNameplate 每 tick 掃描本地控制器並重刷 Slate

來源審查，不是 FPS 或 stress 測量。Source job `run-mv032vbk-1xm1c0` 已終止。獨立審查 `hud-readable-review-v1` 記錄此項。當時還沒有 HUD build 或 native/viewport。已接受的基準仍是 D/F 158 cases／69 intents／51 stages、native 63、11 captures。可讀 HUD 提案的 165／69／51、native 64、15 captures 尚未驗證。

`AOmUnitActor::Tick` 每個 actor tick 都呼叫 `TickNameplate`。修復前 `TickNameplate` 在可見性提前返回之前無條件呼叫 `ApplyPresentationPreferences`。該函式透過 `FOmPresentationPalette::InterfaceFromWorld` 走訪本地 PlayerController、複製 `FOmControlInterfaceSettings`、用 `FString::Printf` 組名字與 HP，然後 `SetNameplate` 每次都呼叫 Slate 的字型、文字、可見性、百分比與顏色 setter。隱藏或數值沒變的單位也一樣。名字與 HP 原本應在權威 frame 更新，不是在 `TickNameplate`。`AGENTS.md` 要求保留 per-entity UI 熱路徑既有節流。只在 `SetNameplate` 加相等判斷不能拿掉上游的每 tick 掃描與字串組裝。

E529 source repair（尚未編譯、尚未跑 build、native 或 viewport，不能寫成通過）：`TickNameplate` 只在 nameplate 已經可見時重定位。名字、HP、種類與 ContentId 的變化走 `OnFrameState`，而且只有實際變化才格式化並呼叫 `SetNameplate`。`SetIdentity` 與 `ConfigureFallbackVisual` 在已綁定偏好時立刻套用。偏好只在 governing action owner 的 nameplate 欄位真的改變時推送：`InstallRuntimeProfile` 比較名字、血條、色盲與文字縮放；chord 或只改 HUD 縮放不會掃描，也不會動 Slate。推送對象是該 owner 註冊的單位，加上第一次成為 governing owner 時還沒綁定的單位。沒有全域第一玩家偏好快取，也沒有 sleep 或會丟更新的節流。

實際支援的行為：一個 actor 上只有一個共享 widget。它跟隨第一個仍有 runtime profile 的本地控制器。後面的本地控制器在前者還有 profile 時不能覆寫。前者 `DropRuntimeProfile`、`DiscardPriorPlayerProfile`、`EndPlay` 或 `BeginDestroy` 時，下一個仍有 profile 的本地控制器接過註冊的單位；世界拆除時只解除註冊，不再套用。這不是分裂畫面各自獨立的 nameplate。`ShouldShowNameplate` 仍是揭露規則。偏好只折疊 widget。隱藏中的標籤仍會收到偏好與權威 HP，所以重新打開時不用再等一次掃描。`SetNameplate` 的相等判斷只是避免重複 setter，不是這次修復本身。

受影響的新測試是 `Om.Runtime.ActionOwner.NameplateUpdateOwnership`，在已掃描的 `OmActionOwnerTests.cpp`。它走 `TrySetInterfaceSettings`、`TrySetChord`、`OnFrameState` 與 owner 銷毀，不走測試專用的偏好 setter。尚未執行。動態發現會比已接受的 63 多兩個尚未驗證的名稱：既有可讀 HUD 的 `Om.Runtime.ControlProfile.DisplayPreferences`，再加上這個測試。不能把 65 寫成 Success。Viewport 清單維持提案中的 165／69／51，召喚師 `Count` 維持 37。四張文字縮放 capture 已在 `.ai-collab/run-controls-cast-native.lua` 的必要清單。`scripts/ue_local_acceptance.lua` 沒有 capture 清單，這次沒有另造一份。Primary 仍須獨立審查、受影響 Lua contract、ordinary no-cache build、D3D12 native/capture 與目前 owned viewport。完整驗收留最後。

### E530 — hud-readable-build-v1 的六筆 C4458／C3892

實際 ordinary nocache `hud-readable-build-v1` 失敗。`summary.json` 是 `exit_code` 1、`refresh` false。`build.log` 146–147：`Result: Failed (OtherCompilationError)`，`Total execution time: 517.42 seconds`。完整編譯診斷在 `.ai-collab/hud-readable-build-v1/UBT-Log.txt`。Lua contracts-v1 先前已過 24+44。HUD native 與 viewport 沒有跑。E529 的來源修復被這次編譯拒絕，仍然不是 FPS 或 stress 測量，也不能寫成通過。提案清單維持 65 native／165 cases／69 intents／51 stages／15 native PNG／3 viewport PNG。

`UBT-Log.txt` 只有這六筆錯誤，警告等級被專案當成錯誤：

- 149，`OmCommandBarWidget.cpp(1954,23)` C4458。`const FSlateFontInfo Slot` 遮蔽 `UWidget::Slot`（`Widget.h` 264）。下一行 lambda 以 `[&Slot]` 擷取這個區域變數。
- 157，`OmControlSettingsWidget.cpp(687,81)` C3892。`Next.*Member = Value` 不能寫，因為區域型別 `const float FOmControlInterfaceSettings::* Member` 把指向的欄位當成 const。
- 163，`OmControlSettingsWidget.cpp(736,79)` C3892。同一種 `const bool FOmControlInterfaceSettings::* Member` 使 `Next.*Member = ...` 不能寫。
- 176，`OmUnitActor.cpp(311,37)` C4458。`EndPlay` 的 `UOmPlayerActionOwnerComponent* Owner` 遮蔽 `AActor::Owner`（`Actor.h` 839）。
- 182，`OmUnitActor.cpp(571,33)` C4458。`ResolveNameplateOwner` 的同名區域變數。
- 191，`OmUnitActor.cpp(580,74)` C4458。`BindGoverningNameplate` 的參數 `Owner`。

Primary 的獨立來源審查沒有先看到這三類問題。沒有放寬警告、沒有改編譯旗標、沒有加 suppression。

E530 source repair（尚未重跑 build、native 或 viewport，不能寫成通過）：`Slot` 改名 `SlotFont`，lambda 擷取與 `SetFont` 使用同一名稱。兩個設定 lambda 的區域指標改成 `float FOmControlInterfaceSettings::* const` 與 `bool FOmControlInterfaceSettings::* const`，所以指向的欄位可寫、指標本身不改；讀取仍走 const profile，寫入仍走 `TrySetInterfaceSettings`。三處 governing action owner 連同 `OmUnitActor.h` 的宣告參數都改名 `GoverningOwner`。行為與清單不變。E529 的熱路徑修復維持原判斷，不因這次編譯失敗改寫成已測量的 FPS 結果。Primary 審查後跑新的 `hud-readable-build-v2` ordinary nocache；只有 build 成功才接著 native 65、15 captures、目前 viewport 165 與 prefix。完整驗收留最後。

### E532 — hud-readable-native-v1 的 23 筆錯誤與 1 筆警告

實際 `hud-readable-native-v1` TERMINAL FAIL。`index.json` 是 expected 65、actual 65、58 Success、7 Fail、0 notRun、0 inProcess。23 筆 error，1 筆 warning。警告只在失敗的 `Om.Runtime.ResponsiveHudLayout`：`LogRHICore` 保留的虛擬位址 258 GB 超過 256 GB。不能寫成沒有警告。`errors.md` 只有 NameplateUpdateOwnership 的外層 Fail，23 筆錯誤在 `index.json`。產物在 `.ai-collab/.bridge-data/worker-pool-machine-v1/hud-readable-native-v1/`，未改。沒有 viewport，也沒有已接受的視覺紀錄。

先前證據維持：`hud-readable-build-v3` refresh PASS（193.70 秒加上 ordinary gate 4.10），`hud-readable-build-v4` ordinary nocache PASS、refresh false、2.47 秒。Lua contracts-v1 PASS 24+44。E530 的來源修復已編成。E531 的 refresh 必要性不改寫。E529 仍是來源熱路徑修復，不是已測量的 FPS。

失敗與推論：

- `Om.Runtime.ActionOwner.NameplateUpdateOwnership`，10 errors、0 warnings。`OmActionOwnerTests.cpp` 3590、3597、3616、3649、3663：Slate apply 期望 1 或大於先前值，實際 0。3694：owner replacement resolve 期望 5，實際 4。3696：下一個 owner 的 scale／mode 沒有套上。3698：隱藏標籤沒有套上。3699：replacement Slate 期望 1，實際 0。標籤與可見性斷言不在錯誤清單，所以 pending widget 有更新。Screen-space `UWidgetComponent` 只在可見、有 game viewport 與 local player 時，由 `UpdateWidgetOnScreen` → `TakeWidget` 建立 Slate。計數只在 setter widget 存在時增加。`FirstController->Destroy()` 沒有走到 `ReleaseNameplateListeners` 的下一個 owner 解析。`APlayerController::DestroyNetworkActorHandled` 在 `ULocalPlayer` 時回傳 false，不是這次短路。`UWorld::DestroyActor` 仍會 `RemoveActor` 然後 `UnregisterAllComponents`；`UnregisterComponent` 本身不呼叫 `EndPlay`。
- `Om.Runtime.ControlProfileSummonerCastModes` 2 errors：513 Personal D rebind、529 round-trip 的 rebound D。`Om.Runtime.ControlRuntimePersistence` 5 errors：984 Cast Q rebind、saved press／release 不是 U、1008 U release 期望 1 實際 0、1009 plain Q press 期望 0 實際 1。`Om.Runtime.HudAssetRegistry` 2 errors：59 Q rebind、60 HUD 仍是 Q。`Om.Runtime.SummonerOwnerCards` 1 error：2123 Summoner D rebind。新鮮 profile 的 `ToggleUnitNames` 預設佔用 U，衝突拒絕仍應嚴格。召喚師測試裡其他 reject 是 Info，不是 error。
- `Om.Runtime.ResponsiveHudLayout` 1 error、1 warning。`OmResponsiveHudTests.cpp` 641：`Hidden clusters are collapsed`。`ApplyHudChromeVisibility` 折疊外層 `MinimapCluster` `SBorder`。`OmMinimap` 標在內層 `SOmMinimap`，父層折疊時子層自己仍是 Visible。文字縮放 capture 的 Info 在這筆錯誤之前，那些寫檔有走到，但整項測試失敗，所以沒有 native PASS 的 capture 組。
- `Om.Runtime.MinimapPreferences` 2 errors、0 warnings。`OmCameraInteractionTests.cpp` 557 與 597。Info：`RightDrag camera=X=7500.000 Y=7500.011 Z=0.000 expected=X=7500.000 Y=7500.000 Z=0.000 capture=1 inspecting=1`。同一筆 Info 的 paint 本機大小是 112×112，DPI 1.750，視窗 1400×1050，viewport 800×600，`arrangedUnderPress=0`，`tickUnderPress=1`。0.011 cm 是 0.00011 世界單位。地圖 0–100、本機 112 時每世界單位 1.12 px，再乘 `WorldUnitsToCm` 100。這是絕對座標 float 的 `LocalToAbsolute`／`AbsoluteToLocal` 往返，不是整張地圖映錯。capture 與 inspecting 都是 1。597 的實際相機沒有被印出來。左／右按下在 0.01 cm 內的斷言不在錯誤清單。Viewport 的 one-quantum wire 斷言沒有改。

E532 source repair（尚未重跑 build、native、capture、viewport 或 prefix，不能寫成通過）：

- Nameplate 測試在計數基準之前，用公開 `OnFrameState`、`SetOwnerPlayer`、world tick 與 viewport window 的 prepass／redraw 讓真正的 screen widget `TakeWidget`。計數仍只算已建立的 Slate setter。不把 pending 值算進去，不刪斷言，不從測試呼叫 private `EndPlay` 或偏好 setter。銷毀改在 viewport 仍開著時進行，斷言之後才 `Close`。`OnUnregister` 在 actor 或 component 正在銷毀時呼叫既有的 `ReleaseNameplateListeners`；`EndPlay` 與 `BeginDestroy` 仍會呼叫它，第二次看到空清單就返回。Owner 已經 `RemoveActor` 而沒有 world 時，改從仍活著的 listener 取 world，再解析下一個 profile。E533 更正：世界拆除或最後仍沒有 world 時改走 `DetachGoverningNameplate`，只解除註冊，不 `Bind` 預設偏好，也不呼叫 `ApplyCachedNameplate`。E532 報告原本寫成不套用，但當時程式在跳過 next-owner 之後仍 `Bind(nullptr, defaults)`。
- 四個失敗 fixture 在把 Q 或 D 綁到 U 之前，用公開 `TryUnbind` 或 `TrySetChord` 空 chord 放開 `ToggleUnitNames`。預設 O／N／U、衝突拒絕與舊 profile 的 migration conflict 不放寬。Version 2 匯入仍把已保存的 U 留給 SummonerSlot1。Owned viewport 的準備只綁 K、F6、F10、F8 secondary 與滑鼠鍵，沒有佔用 O／N／U；display 案例與 wire prefix 不變。
- 外層小地圖 `SBorder` 加上 `OmMinimapCluster`。隱藏斷言改為折疊的 combat cluster 與 minimap cluster、內層 `OmMinimap` 仍 Visible、兩簇不接受指標、設定按鈕與打開的商店仍接受指標。`Display()` 的 hide／show 改看 cluster，不再把內層 Visible 當成 HUD 已隱藏。
- 兩筆失敗的拖曳相機比較改用從該次幾何導出的容差。`AbsoluteRef * 2^-23` 是 float32 相對 epsilon 的上界，不是該數量級的精確 ulp 間距。公分值是這個上界的兩倍，除以 layout scale 與地圖 pixels-per-world-unit，再乘 `WorldUnitsToCm`。容差必須小於一個本機像素的 0.05。預期點仍是 75,75 與右拖的未投影點。拖曳必須離開按下點。0.01 cm 的按下斷言與 viewport one-quantum wire 斷言維持原樣。E533 更正了原本「精確兩次 ulp」的說法。

清單維持 65 native／165 cases／69 intents／51 stages／15 native PNG／3 viewport PNG。沒有新增 `IMPLEMENT_SIMPLE_AUTOMATION_TEST` 或 viewport case。Primary 審查後跑新的 ordinary nocache build；成功後才跑 D3D12 native 新目錄與實際 captures，再跑 viewport 與 prefix。完整驗收留最後。沒有 PASS。

### E533 — 400×300 仍把整條戰鬥列縮小，而不是緊湊重排

實際證據是失敗的 `hud-readable-native-v1` 裡不可變的 `responsive-hud-captures/responsive-hud-400x300.png` 與 `responsive-hud-1920x1080.png`。Primary 的 ORIGINAL-detail 紀錄在 `.ai-collab/hud-native-v1-diagnostic-visual-review.md`。1920×1080 的底部置中列可讀。400×300 仍把完整戰鬥列整組塞進視窗，標題、冷卻與物品文字看起來太小。這不是已測量的 OS DPI。未轉換的 `DesiredSize` 或字級不能當成最終像素可讀性。沒有成功的 HUD 視覺或 viewport 紀錄。已接受的基準仍是 D/F 158／69／51、native 63、11 captures。E532 的來源修復仍未編譯、未重跑；native-v1 的失敗產物不改。Lua 24+44 與 build-v4 ordinary 2.47 秒都在 E532 之前。

E533 source repair（尚未重跑 build、native、capture、viewport 或 prefix，不能寫成通過）：視窗寬度至少 800 時先試 roomy 欄。放不下，以及更窄的視窗，改為 compact：能力與召喚師名字行收合，完整標籤留在卡片 tooltip，熱鍵、冷卻、物品狀態、商店與設定仍在畫面上。字級仍是 body `round(11*scale)` 10–18、title `round(14*scale)` 12–22、slot `round(12*scale)` 11–20，不另乘縮小。英雄在 compact 顯示 `HP`／`LV`／`XP`。80×60 仍是 explicit no-fit。400×300 的 text scale 1.5 不保證放下。新增 capture 只有 `responsive-hud-400x300-text125.png`。斷言改看排列後的絕對尺寸與 layout／render scale，不再只用收合前的 DesiredSize。沒有新的 automation 名稱，也沒有新的 viewport case。清單改為 65 native／165 cases／69 intents／51 stages／16 native PNG／3 viewport PNG。同一批也更正 E532 的 teardown 與 ulp 上界敘述。Primary 審查後跑 ordinary nocache、D3D12 native 新目錄與全部變更過的 capture，再跑 viewport 與 prefix。完整驗收留最後。沒有 PASS。下一個實際錯誤編號是 E534。

### E534 — hud-readable-native-v2 在 NameplateUpdateOwnership 讀到空的 ViewportClient

實際 `hud-readable-build-v5` ordinary nocache PASS、refresh false、111.38 秒。它編成了 E532 與 E533 的來源。接著 `hud-readable-native-v2` TERMINAL：native process exit 3、success false。預期 65 個名稱。沒有 `native-report/index.json`。不能從缺席的最終報告製造 65 筆成功或失敗次數。`native-engine.log` 最後的測試是 `Om.Runtime.ActionOwner.NameplateUpdateOwnership`，接著 `EXCEPTION_ACCESS_VIOLATION`，讀取位址 0x100。呼叫是 `SGameLayerManager::GetNormalizeRect`（`Engine/Source/Runtime/Engine/Private/Slate/SGameLayerManager.cpp` 703）← `AddOrUpdatePlayerLayers`／`UpdateLayout` ← `ForceRedrawWindow` ← 修復前的 `OmActionOwnerTests.cpp` 3593。產物 `.ai-collab/.bridge-data/worker-pool-machine-v1/hud-readable-native-v2/{summary.json,native-engine.log,errors.md}` 不改。沒有成功的 capture、viewport 或驗收。E532 與 E533 因此仍未接受。

`GetNormalizeRect` 在 `LocalPlayer` 非空時直接呼叫 `LocalPlayer->GetProjectionData(LocalPlayer->ViewportClient->Viewport, ...)`，下一行再用 `ViewportClient->Viewport->GetSizeXY()`。Nameplate fixture 把 `FirstPlayer` 與 `SecondPlayer` 都 `AddLocalPlayer` 進 `GameInstance`，但當時只做 `Viewport.Open(Instance) && Viewport.BindPlayer(FirstPlayer)`。`SecondPlayer->ViewportClient` 維持 null。E532 的實際 screen-widget paint 讓 game layer 走訪每個 local player，於是讀到空指標。這不是要在 Engine 補 guard。`FOmEditorGameViewportFixture::BindPlayer` 已把傳入的 player 設成同一個 `Client` 並記入 `BoundPlayers`。`Close` 會把仍指向該 `Client` 的項目清成 null。`OmEditorAutomationTests.cpp` 已用同一 helper 連續綁兩個 player，那個測試不改。其他只加一個 local player 的 fixture 不是這次當掉的順序，也不改。

E534 source repair（尚未重跑 build、native、capture、viewport 或 prefix，不能寫成通過）：這個 fixture 在 `NotifyBeginPlay`、`Settle` 的 world tick 與任何 `ForceRedrawWindow` 之前，改為 `Viewport.Open(Instance) && Viewport.BindPlayer(FirstPlayer) && Viewport.BindPlayer(SecondPlayer)`。兩個 controller 仍先加入 First 再加入 Second，governing owner 仍是第一個。銷毀仍在 viewport 開著時呼叫 `FirstController->Destroy()`，replacement 斷言之後才 `Viewport.Close()`。後註冊的 `PlayerRemoved` scope 仍先於 `Close`。Paint 迴圈前增加斷言：兩個 local player 的 `ViewportClient` 與 `Viewport` 都非空，而且是同一個 client。Slate 計數、真實 screen widget、生命週期與既有斷言不放寬。不刪 `SecondPlayer`，不改 Engine。清單維持 65 native／165 cases／69 intents／51 stages／16 native PNG／3 viewport PNG。沒有新的 automation 名稱，也沒有新的 viewport case。E529–E533 的來源維持。Primary 獨立讀來源後跑新的 ordinary nocache `hud-readable-build-v6`；只有 build 成功才跑 D3D12 `hud-readable-native-v3`（65 名稱、16 PNG）、實際畫面、目前 viewport 165／69／51 與 prefix DF-v2。完整驗收留最後。沒有 PASS。下一個實際錯誤編號是 E535。

### E535 — hud-readable-native-v3 的 800×600 combat cluster 被當成 group scale

實際 `hud-readable-build-v6` ordinary nocache exit 0、refresh false。接著 D3D12 `hud-readable-native-v3` process exit 0，但 suite success false。`native-report/index.json` 存在：exact 65 名稱、64 Success、1 Fail、0 notRun、0 inProcess。唯一失敗是 `Om.Runtime.ResponsiveHudLayout`，1 error、1 warning。警告是既有的 `LogRHICore` 保留虛擬位址 258 GB，預算 256 GB，保留。錯誤原文：`Expected 'fixture 800x600 combat cluster is not group-scaled' to be true.`，`OmResponsiveHudTests.cpp` 388。這是尺寸迴圈的第一個 800×600，失敗就返回，所以後面的 capture、compact 與放大文字都沒有被這次 native 驗證。`NameplateUpdateOwnership` 與其他 E532 修復項是 Success。沒有成功的 HUD 驗收、viewport 或視覺驗收。產物 `.ai-collab/.bridge-data/worker-pool-machine-v1/hud-readable-native-v3/` 的 summary、index、log 不改。

修復前的 `ClusterIsNotGroupScaled` 讀 `GetCachedGeometry()`，也就是 tick space。它要求本機高度大於 1、累積 layout scale 至少 0.99，而且 `AbsH / LocalH + 0.05` 至少等於 layout scale。同一條 `ContainsRect` 在這筆錯誤之前已通過，所以 combat cluster 的絕對矩形在 800 視窗裡面，而且高度大於 1。這不是沒畫到的零矩形。`RefreshClusterScales` 沒有對 cluster 乘縮放。`MinimapCluster` 與 `CombatWidthBox` 的 render transform 只有平移。

累積 layout scale 低於 0.99 來自 viewport，不是戰鬥列自己縮小。`UGameViewportClient` 把 overlay 放進 `SGameLayerManager`，外層是 `SDPIScaler`。`GetGameViewportDPIScale` 用 `UUserInterfaceSettings::GetDPIScaleBasedOnSize`，再除以 viewport geometry 的 scale，把平台 DPI 消掉。專案沒有覆寫這條曲線。`BaseEngine.ini` 的 `UIScaleRule=ShortestSide`，鍵是 480→0.444、720→0.666、1080→1.0。800×600 的短邊是 600，落在前兩個鍵之間，曲線值低於 0.99。1280×720 的短邊 720 是 0.666。400×300 的短邊 300 不高於 480 的 0.444。1920×1080 與 2560×1080 的短邊 1080 是 1.0，但 800 先失敗，那些案例沒跑到。這次 log 沒有印出實際 scale，所以上面是曲線與呼叫路徑，不是從 native 量到的小數。

E535 source repair（尚未重跑 build、native、capture、viewport 或 prefix，不能寫成通過）：不把 0.99 或 0.05 放寬成能通過的門檻，也不改戰鬥列去反除 DPI。`OmCombatCluster` 是會畫的 `SBorder`。斷言改拿它的 paint space 與 tick space，跟同一個 command bar root 比。widget 的 layout scale 不能比 root 少超過 0.05，render scale（絕對高度除本機高度）也不能比自己的 layout scale 少超過 0.05。文字仍要字級至少 10，而且本機高度加 1 至少等於字級，再通過同一個 ancestor 比較。每個尺寸與文字縮放案例用 `AddInfo` 印出 viewport 像素、曲線值、viewport layout scale、視窗 DPI，以及 window、root、combat cluster、`OmAbilityHotkey.0` 的 paint 本機大小、絕對大小、layout scale、render scale 與 tick scale。失敗訊息也帶 combat 與 root 的數字。真實 `ForceRedrawWindow`、五種解析度、文字縮放、80×60 no-fit、槽位、hover、狀態與輸入檢查都保留。沒有新的 automation 名稱，也沒有新的 viewport case 或 PNG。清單維持 65 native／165 cases／69 intents／51 stages／16 native PNG／3 viewport PNG。E529–E534 的來源維持。Primary 獨立審查後跑新的 ordinary nocache `hud-readable-build-v7`；只有 build 成功才跑 D3D12 `hud-readable-native-v4`，再對成功的結果看實際 capture、目前 viewport 與 prefix DF-v2。完整驗收留最後。沒有 PASS。下一個實際錯誤編號是 E536。

### E536 — viewport DPI 曲線把 400×300 留在放大的 roomy 座標，字級小於 10 物理像素

實際 `hud-readable-build-v7` ordinary nocache PASS、refresh false。E535 的來源已審查並編成。接著 D3D12 `hud-readable-native-v4` process exit 0，suite success false。exact 65 名稱、64 Success、1 Fail、0 notRun、0 inProcess。唯一失敗仍是 `Om.Runtime.ResponsiveHudLayout`，1 error、1 筆 `LogRHICore` 258 GB／256 GB 警告。錯誤在 400×300：compact density 兩邊不相等。800×600 已通過並寫出 `responsive-hud-800x600.png`。後面的 400 capture 與文字縮放案例沒跑到。

`native-engine.log` 3099–3103 的實測：800 的 viewport 是 800×600、curve 0.555、viewportLayout 1.750、windowDpi 1.750。root local 1441.4×1081.1、abs 800×600、layout／render 0.555。combat local 640×209.5、abs 355.2×116.3、scale 0.555。hotkey0 local 10.8×19.8、abs 6×11、scale 0.555。400 的 viewport 是 400×300、curve 0.444、同一組 1.750。root local 900.9×675.7、abs 400×300、scale 0.444。combat 仍是 local 寬 640、abs 284.2×97.6。hotkey0 abs 5×8。Primary 看過 800 的 PNG 原圖：戰鬥列與文字都太小。那張圖只是失敗輪的診斷，不是視覺驗收。

`RefreshClusterScales` 用 `NativeTick` 的 `MyGeometry.GetLocalSize()`。viewport `SDPIScaler` 在曲線小於 1 時把 local 空間放大，所以 400 物理像素變成 local 寬約 901，仍走過 `>= 800` 的 roomy。字級約 12×0.444，大約 5.3 物理像素。只跟 root 比 scale 不能證明物理可讀。E535 的原因現在有量測。沒有完整 OS DPI 矩陣。曲線鍵仍是 `BaseEngine.ini` 的 480→0.444、720→0.666、1080→1.0，omfue 沒有覆寫。沒有改 Engine 或 config。

E536 source repair（尚未重跑 build、native、capture、viewport 或 prefix，不能寫成通過）：`FOmHudViewportSpace` 從 HUD 幾何取出累積 layout scale。小於 1 時子層 layout scale 用倒數補回，配置空間用四捨五入後的絕對像素，所以 400 進 compact、800 仍 roomy。大於或等於 1 的解析度放大直接留下。這是 `SOmHudResolutionNormalizer` 的 arrange，不是只改 render transform，hit-test 與寬度覆寫走同一空間。字級上下限不改；body／slot 在文字縮放 1 時，`font.Size * render scale` 應至少 10 物理像素。放大的文字縮放仍把字級加大。E535 的診斷保留，另外記錄 normalized content。文字斷言加上有效字級，0.05 仍是原有的絕對誤差，沒有為了現在的小字放寬。曲線或 DPI 改變會進 layout signature，並在子層 scale 改變時 Invalidate layout／prepass。小地圖左右、相機、capture、跨 owner teardown 沒改。80×60 仍是 explicit no-fit。沒有新的 automation 名稱、viewport case 或 PNG。清單維持 65 native／165 cases／69 intents／51 stages／16 native PNG／3 viewport PNG。Primary 審查後跑 ordinary nocache `hud-readable-build-v8`；只有 build 成功才跑 D3D12 `hud-readable-native-v5`，再看成功的 capture、目前 viewport 與 prefix。完整驗收留最後。沒有 PASS。下一個實際錯誤編號是 E537。

### E537 — 正規化之後 400×300 的 compact 測量沒過，800 中心被 SBorder 接走

實際 `hud-readable-build-v8` ordinary nocache PASS、73.40s、refresh false。E536 的正規化已編成。接著 D3D12 `hud-readable-native-v5` process exit 0，suite success false。`native-report/index.json`：succeeded 62、succeededWithWarnings 1、failed 2、notRun 0、inProcess 0。exact 65 名稱。63 筆是成功狀態。兩筆失敗各 1 error、0 warning。`Om.Runtime.SummonerOwnerCards` 仍是 Success，並保留既有 `LogRHICore` 258 GB／256 GB 警告。沒有把警告丟掉。產物 `.ai-collab/.bridge-data/worker-pool-machine-v1/hud-readable-native-v5/` 的 summary、index、log 與 800 PNG 不改。沒有成功的 HUD 視覺驗收。

`Om.Runtime.PreciseWorldCommand.Route` 在 `OmActionOwnerTests.cpp` 3157 失敗。Info 是 `Restored world leaf=SBorder`。錯誤是 `Expected 'Production bar leaves the center as a world hit' to be true.` 同一測試在 v4 的葉子是 `SViewport`。生產命令列仍註冊，記分板開過再 Escape，HudRoot 移除，prepass／`ForceRedrawWindow`／焦點回到實際 viewport 之後，`LocateWindowUnderMouse(Fixture.Center())` 仍要求路徑包含 viewport，而且最後一個 widget 是 `SViewport` 或 `SGameLayerManager`。

`Om.Runtime.ResponsiveHudLayout` 在 `OmResponsiveHudTests.cpp` 459 失敗。800 已通過並寫出截圖。400 之後的 PNG 與文字縮放案例沒跑到。index 的實測：800 的 viewport 800×600、curve 0.555、viewportLayout 1.750、windowDpi 1.750。root paint 1441.4×1081.1、abs 800×600、layout／render 0.555。normalized paint 800×600、abs 800、layout／render 1.000。combat paint 640×219、scale 1。hotkey0 paint 11×19、scale 1。400 的 viewport 400×300、curve 0.444、同一組 1.750。root paint 900.9×675.7、abs 400×300、scale 0.444。normalized 400×300、scale 1。combat paint 384×196、scale 1。hotkey0 11×19、scale 1。`IsHudLayoutFitted()` 為 false。384×196 是 explicit no-fit fallback 畫出來的尺寸，不是放寬後的合格尺寸。Primary 看過這次 800 PNG 原圖：字已經回到完整大小。那只是診斷，不是視覺驗收。E536 的物理字級與座標政策在這次量測裡是有效的，保留。

兩筆失敗是同一件未收斂的 `SWrapBox`。預設 `PreferredSize` 是 100。`UseAllottedSize` 的 Tick 發生在 paint，晚於 fitter 的孤立 `SlatePrepass`，也晚於建立 hit grid 的那次 prepass。E536 之後 local 空間就是物理像素，所以 100px 寬把六張卡排成多列，底部戰鬥列高到蓋住 800 的中心。葉子型別是 `SBorder`。商店在關閉時子層是 Collapsed，`SBox` 期望尺寸是 0；記分板在 Escape 之後也是 Collapsed，外層會是 `SBox` 而不是這次的 `SBorder`。400 的每一次 Fits 都在這個未收斂高度上失敗，於是 fallback 留下 384×196。寬度 +2、高度 +1、小地圖 +2、重疊與字級下限都沒有放寬。沒有拿 HUD 葉子當世界命中，沒有移走生產列，沒有假的世界座標偏移，也沒有繞過輸入守衛。

E537 source repair（尚未重跑 build、native、capture、viewport 或 prefix，不能寫成通過）：技能列與道具列在孤立 prepass 之前就把 wrap 寬設成戰鬥列內寬（`CombatW - 12`）。建構時先設 628，也就是既有 roomy 640 減邊框 12，所以第一個 hit prepass 不會停在預設 100。`UseAllottedSize` 關掉，避免零幾何的 Tick 把這個寬度清掉。道具六格改成同一個 `SWrapBox`。compact 時經濟文字與道具文字用 `WrapTextAt`，經濟列與商店按鈕同一列，商店按鈕寬度先 prepass 再留給文字。widgets、圖示、熱鍵、tooltip、冷卻、HP／LV／XP、設定、商店與小地圖都還是原來的。沒有整組縮小。Fits 的數字門檻不變，每次嘗試把 outer、stack、combatW、inset、高度上限、combat／minimap 期望尺寸與失敗條款寫進 `GetHudFitAttemptLog()` 和 `OM_HUD_FIT`。responsive 測試在 fit、80×60 與文字縮放斷言前記下這段 log。精確世界測試仍用 `Fixture.Center()`，仍只接受 `SViewport` 或 `SGameLayerManager`，另外記下最後幾個 widget 的型別、tag、可見性與幾何。商店外層 `SBox` 改成 `SelfHitTestInvisible`；打開時命中仍落在 `OmShopPanel`。記分板與比賽結果的外層維持可命中，因為它們打開時是真正蓋住畫面的面板，不是這次中心點的葉子。400 預設與文字 1.25 仍是要 fit 的目標。400 文字 1.5 不保證。80×60 仍是 explicit no-fit。較高解析度、viewport rescale、使用者偏好與名牌熱路徑的公式沒改。沒有新的 automation 名稱、viewport case、wire intent、control stage 或 PNG。清單維持 65 native／165 cases／69 intents／51 stages／16 native PNG／3 viewport PNG。E529–E536 的來源維持。Primary 獨立審查後跑新的 ordinary nocache `hud-readable-build-v9`；只有 build 成功才跑 D3D12 `hud-readable-native-v6`，再只看成功輪的實際畫面、目前 viewport 與 prefix。完整驗收留最後。沒有 PASS。下一個實際錯誤編號是 E538。


2026-10-11 Codex直接接手（不再派發子代理）：authenticated Rust communication transport範圍接受。team-transport-direct-accepted-v1/summary.json綁定42不同測試、四個maintained all-target checks、零ignored/重複；E757 fixture與E758 backend Fetch型別失敗及原log保留，後端修復後fresh check通過。ABI25與codegen-v3為既有紀錄，未新啟用UE/未跑authority game/無ACK。父項8/44保持，下一步直接實作canonical原創互動與獨立HUD status，再接public Unreal input/received widgets。


2026-10-11 Codex直接實作接受：communication-content-status-accepted-v1/summary.json。fresh49不同Rust測試／四個maintained all-target checks／零ignored或重複；官方fallback／19generate-check／20Lua inputs通過。固定ID五種原創角色互動與八種emote、單一ledger／nonce／mute／clock生命期、真World能力與道具／public KDA獨立status已驗。protocol/schema2、ABI25，無optional media匯入、無UE新啟用／authority game／ACK／完整驗收。E759原失敗保留，修復後unsupported測試明確拒絕舊版1與未支援版。5.1文字所列ping語意／transport／合法target座標／限流靜音與隔離契約已由已接受Rust證據滿足，故僅勾5.1（9/44）；5.2–5.4真UE入口與畫面仍pending。立即直接接public Unreal communication API，無子代理。
### E989 — 多檔 patch 使用未核對的測試行文字

2026-10-12 Codex自行接只選英雄策略時，patch最後一個hunk猜測既有assert文字，apply_patch整批驗證拒絕，未寫入任何檔案。已以rg及實際範圍重新讀取確認前兩檔也未變，再用真實assert錨點套用。後續多檔patch先核對每個錨點，不依記憶拼測試描述；拒絕後查清實際寫入狀態，不假定部分成功。

### E990 — 只選英雄批次的探索路徑及區域變數名稱錯誤

2026-10-12 Codex自行實作。只讀探索曾猜測不存在的 native/systems、transport_client.rs、scripts/lib，rg拒絕；正式位置必須先由rg --files確認，不能把猜測傳給讀取工具。hero-target-build-v1普通NoEngineChanges／NoUBA／nocache實際失敗（UBT200.34秒、exit6），新AOmWorldBridgeActor helper內Owner區域變數遮蔽AActor::Owner，C4458視為錯誤。原build.log／UBT-Log.txt／summary保留。terminal後改名ActionOwner，未改Engine或壓低warning等級；追加真Slate hero-only press/release／repeat／toggle／modal／death／focus與保存檢查，再fresh build-v2。E982已有同類shadow經驗，此次未套用到Owner命名；後續Actor方法新增區域名稱先檢查繼承成員，不用Id、Owner、Role等通用名。

### E991 — 新增 Slate 斷言早於既有區域搜尋 helper 的宣告

hero-target-build-v2實際失敗，OmActionOwnerTests.cpp新hero-only label斷言呼叫FindTag，但同一測試的helper定義在後方，C3861及連帶型別錯誤。原v2 logs／summary保留；不是產品編譯錯。已將既有同一helper往前移到首次使用前，沒有新增重複搜尋實作或移除label斷言，fresh build-v3重新驗證。插入大測試函式時須核對局部變數的宣告位置，而不只確認檔案裡有相同名稱。

### E992 — HUD bound attribute 查焦點路徑造成 Slate 遞迴 stack overflow

hero-target-build-v3普通建置成功18.12秒，但fresh hero-target-native-v1於ControlRuntimePersistence真崩潰exit3，EXCEPTION_STACK_OVERFLOW。原native-engine.log／summary及Saved/Crashes/UECC-Windows-B87AEDFA4905BD0420CFFE9110E565DD_0000保留；前四PreciseWorldCommand success不代表完整9項成功。callstack明示WidgetPath::SearchForWidgetRecursively重複，再經OmRuntime的HeroTargetPolicy文字attribute回到路徑搜尋。IsHeroTargetOnlyActive getter原本呼叫HasOwnedViewportFocus→GeneratePathToWidgetUnchecked；文字／visibility屬性求值不能做這種會重新求值widget屬性的路徑發現。修為畫面getter僅讀已保存狀態與純owner／phase／alive資料，焦點／modal／connection的admission及退役留在原事件與TickComponent。保留active HUD label及真Slate測試，不用關閉提示、跳過測試或加遞迴深度遮蔽問題。fresh build-v4與native-v2重新驗證。

### E993 — 權威偏好探索時再度猜測不存在的 ScriptRegistry 路徑

2026-10-12 新權威戰鬥偏好基礎探索，rg對native/script_registry.rs回報不存在，未修改檔案。已改為在實際runtime目錄按符號搜尋，再發現StateInitializer實際在native/initialization.rs；正式constructor測試直接用setup_campaign_ecs_world與setup_single_lane_match，無需猜測ScriptRegistry檔案或載入DLL。E990的inventory優先規則再次未落實，後續必須先取得命中的真實檔名再傳給讀取命令。

### E994 — 權威名冊優先後，舊守衛測試仍假設 replica 偏好可覆蓋權威

combat-roster-rust-v1 的 targeted 5 通過，但 core-all 真實失敗（727 passed／1 failed）。indexed_auto_disabled_ward 測試最後注入 champion-only replica resource，預期權威拒絕守衛；與新名冊單一持久偏好來源衝突。修正為檢查 replica 不得覆蓋權威名冊，保留守衛自動／直接／友軍／英雄所有檢查；另有既有 champion_only_blocks_direct_creep_and_not_attack_move 與 filtered ward 測試繼續驗暫時策略。未改產品去迎合舊 cache，也未刪測試。移除新 fixture 兩個 unused_mut。v1 runner 後續未執行的 bridge job 寫了不存在的 presentation-ipc feature，事前檢查 Cargo.toml 發現後，新 v2 改用既有 ipc_tests filter／default all-target check；沒有宣稱該 feature 命令實際跑過。v1 原 helper、log、summary 保留，fresh v2 重新凍結來源驗證。

### E995 — runner 修正用了 Lua pattern，未命中後仍啟動

combat-roster-rust-v2 的 targeted 5 與 core-all 728 全通過，但 bridge-ipc 真實 exit101：presentation-ipc 不存在。prepare v2 的 gsub 將含減號文字當 pattern，替換沒有命中，且缺少產出檢查。v3 改為 plain find 的精確替換，每次必須命中，寫入前確認不存在錯誤 feature；保留 v2 全部證據。另一次只讀搜尋再猜 authoritative_mode 在 native 內而失敗，inventory 已確定實際為 runtime/authoritative_mode.rs；後續只使用 inventory 實際路徑。

### E996 — wire 接線探索路徑與 patch 及 fixture compile 診斷

只讀探索再次誤用 native/authoritative_mode.rs、native/filtered_specs.rs、native/dispatcher.rs、src/transport 目錄及 Windows literal wildcard；均未寫入。實際位置以 rg inventory／directory symbol search 得到 runtime/authoritative_mode.rs、runtime/filtered_specs.rs、native/system_dispatcher.rs、src/transport.rs。後續未知位置僅傳已知目錄做搜尋。一次 patch 留空 hunk，另一次 filtered matches 錨點不符，整批拒絕；確認無寫入後分批按實際文字重做。combat-wire-check-v1 core --no-run 實際 E0063 五個 TeamTickFrame fixture 缺新欄位，原 log／summary 保留；補維護範圍明確舊無偏好的 fixture 欄位 None，不改斷言或跳過案例。協定 selective 9→10、presentation 7→8 的既有版本 assert 同步升版，不冒稱已完成新 wire／UI 驗收。

### E997 — 新 wire 全消費端診斷與舊協定硬編碼

combat-wire-check-v2 core 實際729 passed／1 failed，ward receiver 斷言仍硬編碼 selective9；新 wire codec 兩項通過。client／bridge 因 renderer_fixture intent match 未處理 CombatPreferences、三個 Some(mode) FilteredRenderSnapshot literal 缺新欄位而編譯失敗；backend fallback TeamGameStart 也缺新欄位。原四份 log／summary 保留。補全部實際消費端與舊無偏好 fixture，版本 assert 更新 selective10／presentation8；另外發現尚未執行的 backend ward assert7、bridge summoner assert6，同步改正式新版本，不冒稱已執行這兩個舊測試。審查補偏好進 authority／replica canonical team hash，以及真 router、可見敵方過濾、filtered stepper 安裝／清除、bootstrap／rebase／tamper、renderer dead-owner／非法 shape／owner／epoch、secure transport regressions。設定屬於 owner 名冊，不能因死亡要求一個 actor life；renderer 必須有合法 owner projection 才可提交，replica 不重播設定為英雄指令，而由完整投影在戰鬥前安裝。v3 fresh 驗證，尚無 UE／UI 成功宣稱。

### E998 — 新 checkpoint 測試未指定檢查頻率及 example literal 漏補

combat-wire-check-v3 core 真733 passed／1 failed：combat_wire_frames_bootstrap_rebase 在 tick1 的 checkpoint unwrap(None)。實際 TeamProjectorConfig default 是120 tick，先前進度訊息誤說30；此處依原碼更正為120。fixture 明確設 hash_checkpoint_interval_ticks=1，保留權威／replica hash 相等、bootstrap／rebase、tamper 與非法資料不變全部斷言。core all-target check另真E0063 examples/phase6_security.rs舊literal缺新欄位，補明確None。client-wire1、backend-wire1及client／bridge／backend all-target真成功；全部原v3 logs／summary保留，v4重新全跑。只讀探索還誤猜 OmGenerated 為獨立Plugin；inventory 已確認實際為 Plugins/OmRuntime/Source/OmGenerated，後續只用實際路徑。尚無 Unreal新ABI／UI驗收成功宣稱。

### E999 — ABI patch 錯把實際 multiline 函式當成 single line

新增 combat-preferences ABI 接線時，patch 對 publish_applied_input 猜 single-line signature，被整批拒絕，未修改來源。rg確認實際三行參數後以已存在的 admit_owned_summoner 語句為錨點，分離 converter、admission 與 shape helper 正確套用；不假設單一函式名代表格式。新增 ABI27 專用兩 u32 設定值與 inline owner state，現有 fixture 明確補預設，先由真 cbindgen 重新產 header，再跑 Rust compile／布局與consumer檢查；此筆不是成功驗收。

### E1000 — 再次誤猜 Unreal automation test 目錄

唯讀查詢誤把 OmActionOwnerTests.cpp 放在 OmRuntime/Private/Tests，實際 rg --files 確認在 OmEditor/Private。沒有修改錯誤路徑，也沒有測試成功證據。後續未知檔案先用已知 Source 根目錄查 inventory，直接使用查得路徑，禁止依慣例猜 Tests 目錄。

### E1001 — ABI27 integration smoke fixture 漏補專用欄位

combat-abi-check-v1 全 bridge library 真120 passed／1既有 ignored；all-target 真E0063指出 runtime_driver_smoke.rs 三個 OmInputCommand literal 缺四個新 combat 欄位。依編譯器位置補明確舊命令預設，不降低檢查或忽略 integration target。原 log／summary 保留，fresh v2重跑，尚未 Unreal 成功宣稱。

### E1002 — owner 設定回執測試錯用無 input 的 projector 入口

combat-wire-check-v5 core test／all-target 真E0308：把 CanonicalAcceptedInput 傳入 build_frame 的 OrderedFact 參數。實際公開入口是 build_frame_with_inputs，依宣告改用最後 Vec 參數，不轉換／偽裝 fact。其他 client／backend targeted、bridge120與三項all-target真成功；原 logs／summary保留。回執不能靠 living actor，新增 actorless owner action26 與 payload／owner／projection 驗證，再 fresh v6 重跑。

### E1003 — 普通 Unreal 建置兩個 MSVC action 無進展，逾時取消

combat-preferences-build-v1 Rust／cbindgen／staging 與 UHT完成，但 UBT 從16:59:14超過15分鐘仍無完成action，兩個cl單執行緒Executive等待。重現E413現象，未證實根因，不因UBA executor字樣修改引擎。保存 observed-lifetimes、完整stall log與timeout-retirement，依exact executable／creation token僅退休本輪兩個compiler與UBT；正式helper真exit1，summary／build.log／UBT-Log.txt保留，無原PID殘留。這是取消，不是C++diagnostic或驗收成功。修正回執owner範圍並追加實際owned IPC設定案例後fresh普通nocache重試；不改Engine/config/cache/manifests。

### E1004 — 擴充 owned IPC 時漏補第二個 stage 數量與 identity 分類

combat-owned-contract-v1 真失敗：舊139-stage斷言漏改；v2真失敗：combat_defaults 尚未列入 keep_identity。同步更新完整414-case／181-intent／142-stage及 selection17／4／4清單、保留原409-case ordered prefix，三個stage使用既有 identity 不變斷言，v3真95 cases passed。原logs／summary保留；這是 artifact contract，並非 Unreal／TCP 或 authority ACK 成功。額外增加schema／wide alias／idle值／point混用負例後再跑fresh結果。

### E1005 — 低 CPU／無完成 action 不能單獨判定 MSVC 死鎖

E1003 的取消仍保留為失敗證據，不改寫結果。fresh build-v2 限單 action 後觀察到 compiler working set／page faults 持續增加，D 槽機械硬碟滿載；實際 active shared PCH 為 2,574,581,760 bytes，唯讀 SHA-256 掃描耗時99,610ms，其後編譯完成 action 持續增加，普通 NoEngineChanges／NoUBA／nocache 建置真成功742.82秒。證據在 combat-preferences-build-v2/active-pch-sha256.json、resource-contract.log、build.log 與 summary.json；這支持冷 PCH I/O 的判斷，不證明所有停滯皆為同一根因。新增 maintained --prefetch-shared-pch：只從 project/plugin build response inventories 解析既有 SharedPCH，限定 engine Intermediate、去重、8GiB budget、唯讀分塊預讀並檢查大小／mtime；不修改 engine/cache/config，不把預讀當雜湊驗證。15項注入式契約檢查通過，實際 workflow 尚待驗證。後續逾時判斷需同看 I/O／記憶體／log／精確 process lifetime，不能只靠15分鐘無完成 action。

此輪只讀探索又誤用 scripts/ue_* literal wildcard，並猜不存在 tests/ue_engine_bootstrap_test.lua；沒有寫入。實際測試是 scripts/test_ue_engine_bootstrap.lua，已執行通過。重申未知路徑先 inventory，Windows rg 使用 -g glob 而不傳 literal wildcard path。

### E1006 — 新 owned intent 漏接共用 C++ JSONL shape gate

combat-preferences-owned-v1 真失敗於 combat-cursor-submit。TCP 原始 intent 已正確送出 schema1／acquisition1／idle=true，但 LoadIntents 使用的 ValidateOwnedWireIntent 尚未認得 CombatPreferences，原 native／workflow 與三條 intents／兩個 stages完整保留；publisher／editor／profile cleanup 全成功。這不是多送命令，也不是權威拒絕，前一進度訊息的「清單外」指 checker 不認得 kind，此處依實際證據更正。修共用 validator 的專用嚴格四欄 shape，加入四合法tuple、版本／溢位／fraction／錯bool／queue／null target alias／缺欄負例；未知 kind 仍拒絕，不放寬舊動作。fresh ordinary build與JsonlObservation後再fresh owned重跑。

同輪只讀探索又猜不存在 runtime/moba_hud.rs 及 OmRuntime/Public/OmHudWidget.h。實際HUD投影在 omoba-client-runtime/src/moba_hud.rs，來源header先查 inventory。禁止再以類名猜路徑。

### E1007 — apply_patch 重複 target operation 被整批拒絕

新增 input scheme 名冊回歸時，同一 patch 兩段 Update File 指向 moba_match.rs，工具以 multiple operations target 拒絕，沒有寫入。將同一檔案的修改合併到單一 Update File 後重做；不將 patch 拒絕當成 compiler／測試結果。

### E1008 — 固定 Rust toolchain 沒有 rustfmt component

執行 rustfmt 時實際回報1.95.0未安裝rustfmt.exe，未修改來源。沒有改用其他rustc／formatter版本或升級toolchain；後續維持原合法Rust程式並執行真正編譯／測試。input-scheme-foundation-v1完整core740 passed／0ignored、五個正式all-target checks全成功，四份變更來源前後SHA相同。這只接受權威名冊／規則／lease基礎，尚無direction production movement consumer／wire／Unreal成功宣稱；接真consumer與碰撞／攻擊仲裁回歸。

此輪 consumer 探索又誤猜 runtime/ability_runtime.rs；實際模組位於 runtime/native/ability_runtime/，final_move_speed 在 unit_stats.rs。此為唯讀路徑錯誤，不是編譯失敗；之後從已知 runtime 目錄搜尋符號，禁止用 re-export 名稱推測檔案。

### E1009 — FetchMut 測試中的複合指派借用衝突

input-scheme-consumer-v1 core test／core all-target check 真失敗於 input_scheme_tests.rs 的 state.elapsed += state.config...，E0502；其他四個正式 consumer all-target checks 成功。六來源 SHA 全程不變，原 summary／logs 保留。先把 lease 複製到 local 再修改 elapsed，避免 FetchMut 的 DerefMut 複合指派同時建立 immutable borrow；待全部程序退出後才修正，fresh v2 再驗，不改寫 v1 結果。

### E1010 — 將 PowerShell 專用參數誤傳 rg 與 re-export 路徑誤猜

唯讀探索把 -ErrorAction 傳給 rg，實際被解讀為 -E encoding 並拒絕；另誤猜 runtime/input.rs，不存在。沒有來源寫入或測試結果受影響。rg 只使用自身參數；錯誤處理是 PowerShell cmdlet 的選項，不能混用。輸入模組依實際 inventory 讀取，不從 re-export 推測路徑。

### E1011 — 戰鬥設定漏接 MOBA pre-dispatch allowlist

WASD router 探索確認 single_lane_input_action_allowed 沒有 CombatPreferences，begin_moba_match_tick 會在 player_input_tick 的正式 handler 前濾掉它。先前 transport／roster admission 與 Unreal scoped 證據仍保留，但不足以宣稱真對局套用成功；當時沒有 authority ACK，限制未解除。將合法 typed variant 加入唯一正式 allowlist，新增 real world→begin_moba_match_tick→player_input_tick→roster 的完整路徑測試，驗唯一 owner 更新、另一 owner 保留及 queue drain。後續新增命令必須包含 pre-dispatch allowlist 回歸，不只驗直接 admission。第一次 patch 因 MD context 尾字不符整批拒絕、未寫入；讀取實際 context 後重新套用。

### E1012 — 真對局入口 fixture 漏設定正 fixed delta

combat-match-input-v1 新完整路徑測試失敗在 begin_moba_match_tick 返回 false；五組編譯檢查全成功，七來源 SHA 一致。StateInitializer 正常預設 DeltaTime=ZERO，fixture 只改 phase／elapsed，沒有提供正式時鐘步長，因此按既有規則凍結遊戲。測試顯式提供34raw fixed delta後fresh重跑，不更改正式 inactive 判斷、不把 fixture 漏項當作遊戲邏輯故障。v1 原失敗保留。

### E1013 — 新 owner projection 測試的 import／hash 型別錯誤

input-scheme-proto-v1 protoc 已成功產生含 InputSchemeOwnerState 的 fallback，但 core all-target check 真失敗：測試模組漏 import BTreeSet、[u8;32] 直接與 Vec<u8> 比較。補明確 import 並兩側 as_slice，原 log／summary 保留。rebase 測試草稿曾誤猜函式名，已在編譯前依真 API 改為 build_filtered_rebase／apply_verified_rebase；後續先讀簽名再寫 caller。唯讀 rg 又傳 scripts/ue* literal wildcard，沒有寫入；更正為 scripts -g 'ue*.lua'，此類錯誤不可再靠猜路徑處理。

### E1014 — owner ACK refactor patch context 漏掉現有 validation 行

擴充 actorless owner setting ACK 的 patch 省略 selective_replica.rs 中既有 validate_owner_ack 行，工具驗證 context 失敗、整批未寫入。重新讀實際 preflight／inject_step 區塊，保留 payload validation 並加入新 typed input-scheme ACK。此為 patch 拒絕，不是編譯／測試失敗；不可藉 refactor 刪掉舊驗證以配合草稿。

### E1015 — inventory 與舊檔名猜測放在同批，未使用真結果

唯讀 C++ 探索 inventory 回報 OmPlayerActionOwnerComponent.cpp，後續同批查詢卻仍使用猜測的 OmPlayerActionOwner.cpp，實際不存在。改成對已知 Private 目錄以 -g '*ActionOwnerComponent.cpp' 查符號。檔案 inventory 與依賴它的讀取必須分開決定，不在看到結果前預填舊猜測。

### E1016 — fresh WASD press 仍呼叫舊點擊模式的 attack cancellation

核對 Riot 官方2026-05-04 Keyboard (WASD) FAQ 與2026-04-14 /dev Ranked Release 後確認方向輸入不可取消已開始的 WASD 普攻。現有新 apply_direction_input 在 Begin 呼叫 interrupt_attack_for_accepted_command，雖然 gesture tests 證明只取消一次，仍不符合此行為；官方來源見 WASD plan。修正方向：Begin 退休舊 command／recall、保留攻擊 clock，正式 hero_tick 在開啟 keyboard windup 時保存精確生命期 target、impact 只檢查同一合法 target，不轉打新近敵人；該 host-local target 不得序列化成 hidden canonical identity。待當前 mode-wire gate 全退出後修改並fresh回歸，不將舊通過測試當作新行為證據。

### E1017 — 再次把 inline 測試模組猜成檔案

唯讀探索將 combat_acquisition.rs 的 inline test_support 猜成 combat_acquisition/test_support.rs，rg 回報不存在，未寫入。後續用既有檔案的 mod test_support／struct CombatFixture 符號定位實際區塊。這是 E1015 同類重犯；Rust 模組不能從 use 路徑推定實體檔案，先在已知來源查宣告。

同輪 secure_input.rs 亦曾誤猜；依實際符號搜尋，真正 session／view／disclosure 驗證器在 anti_probing.rs。尚未產生的 gate core.log 曾被提前讀取而回報不存在；只讀 producer 已完成的 artifact，先查存在／inventory。

### E1018 — patch hunk 順序與混入錯誤 MD context 導致整批拒絕

snapshot 草稿先列 moba_match.rs 尾端 constructor，再列前方 pre-dispatch hook，apply_patch 按來源順序尋找失敗，整批未寫入。改依實際行號遞增排列後成功。另一批 production tests 搭配未讀取的 MD 猜測句子，再次原子拒絕、測試尚未寫入；拆開正式來源與文件 patch，讀真 MD 尾端再更新。不得以猜測 context 混入大型來源 patch，也不得把工具拒絕當成來源編譯失敗。

### E1019 — 新 reset marker 的 BTreeSet 漏 import

input-scheme-proto-step-v1 vendored protoc 已產生新 step fallback，core all-target check 真失敗於 moba_match.rs 未 import BTreeSet，E0425／E0433；原 log／summary 保留。原模組只有 BTreeMap／HashSet，不可從子測試 use 或其他 fully qualified 使用推斷 import 存在。待程序退出後補明確 import，新增 consumer 回歸後 fresh 重驗。

### E1020 — full filtered step fixture 只列三種 baseline component

step-wire-v1 core 實跑763成功／1失敗，失敗是新完整 frame→Specs stepper 測試第二 tick 的 ComponentNotAllowlisted；第一 tick Reveal staging 與 hash 已成功。fixture 只給 render／property／attack 並用這三種 schema 當 allowlist，正式 full step 還會 export 真 Hero／Facing／TurnSpeed／CollisionRadius，不能以 minimal fixture 冒稱完整 body。保留原 log，等整輪 gates 退出後補實際 fixture 的完整 body baseline，使用正式 maintained secure allowlist；不放寬正式 validator、不改移動／hash 邏輯來掩蓋缺漏。其餘新 typed step／過期 tick／缺 body／duplicate owner 測試均成功。

### E1021 — navigation reset 無法表示模式切換的攻擊重設

等待 step-wire-v1 退出時 source review 發現：Direction Begin 只清導航、ModeChange 同時按既有政策中斷攻擊，兩者不能都只投影 navigation_reset。若上一 tick 在 windup，切到 keyboard 並按方向時，權威已重設攻擊而會移動，filtered consumer 若保留負 attack clock 會停步，post-step attack settlement 來不及修復當步位置。應另帶 authority-only producer 的 attack_reset flag（必須同時 navigation_reset），filtered pre-dispatch 執行同一 accepted-command interrupt；方向 Begin 永遠不設定它。加入真 mode-change producer 與 filtered windup-reset movement 回歸，不以 post-step repair 隱藏落差。此為靜態 review 發現，尚不是執行測試失敗；待當前 gates 全退出後實作。

修正已由 input-scheme-step-wire-v2 驗證：core767、client141、bridge120、backend mode1及五組all-target checks成功，29來源SHA一致。包含完整frame→Specs重播hash、windup兩種reset、pause→resume與舊生命期隔離。E1020同輪修復成功。這是Rust方向快照驗證，不是UE玩家入口完成。

### E1022 — resumed 探索再次使用不存在的 wildcard 路徑

第一輪唯讀rg加入 omoba-core/src/model/moba*，PowerShell沒有展開該參數，rg回報不存在。沒有寫入與建置失敗；已移除猜測路徑，後續只在已知runtime目錄／已確認檔案查符號。E1015同類問題再次發生，禁止把模組名或shell wildcard當實體檔名。

### E1023 — 將 wire reference 誤當內部 validation reference

proto-direction-v1真編譯失敗：新增direction validator誤用內部SecureTargetReference的replica_id／plain epochs到protobuf SecureReplicaTarget；正式wire是optional ReplicaEntityId／ViewEpoch／DisclosureEpoch wrapper。已讀真schema並補缺欄位與zero wrapper回歸，保留原check.log；不能從相似型別名稱推定欄位，也不能改既有wire迎合草稿。程序退出後才修正，fresh重驗。

v2 all-target check成功。下一組edge patch因實際validate_update(&value)而草稿缺&造成整批原子拒絕；讀真context後重套，未改既有validation。

### E1024 — Recall signature 擴充漏掉四個直接測試 caller

proto-direction-v3的正式lib已通過，all-target測試編譯失敗E0061：apply_recall_commands新增admittedBegin owner集合，但四個舊直接caller仍三參數。已搜尋全部呼叫，舊案例明確傳空集合；新同batch Begin優先另建真prehook回歸。保留check.log，不改新signature繞過政策。

v4 protoc/all-target check已成功；真prehook加入同lane mode/direction順序、只有admitted Begin打斷同batch Recall、paused/zero-delta matching End、warmup crossing與wrong-owner/dead/old-life回歸。direction-edge-v1正在跑完整Rust gates。

### E1025 — gate producer 尚未寫出 log 時又提前讀取

direction-edge-v1的core cargo仍執行，helper只在子程序真退出才寫core.log；唯讀Get-Content過早回報不存在。沒有更動來源或中止程序。後續先inventory確認artifact存在，再讀取；執行進度用session stdout，不把尚未產生的檔案當作即時log。

### E1026 — direction ACK 提前忽略而未檢查公開actor owner

edge-v1執行期間source review發現filtered inject最前方validate_projected_ack後continue，繞過既有actor owner檢查。應把direction skip移到既有owned驗證之後，另要求Hero，並拒絕把Direction payload套到其他action_kind。不能因ACK不直接移動就略過身分驗證。待edge-v1真退出後修正並新增wrong-owner／非Hero負面回歸。

### E1027 — KCP owned match arm 漏 borrow

edge-v1 core775／client141／bridge120成功，backend-mode編譯E0308：secure_coordinate_input_with_ward按owned action match，新增validate_client(value)需要&value；既有模式validator同樣顯式borrow，草稿未遵守。已真退出、31來源SHA一致後修正。提取seal_client以共用實際mutation/validation路徑進行原子失敗測試；原logs保留，不當作backend成功。

### E1028 — 新Reveal後直接查Specs尚未匯入的actor

edge-v2 core776成功／1失敗，新增ACK負面測試在第一tick Reveal完成後直接索引stepper.filtered.entities，panic no entry。第一tick的post-step baseline只merge回disclosed world，Specs刻意未模擬新actor；下一tick才同步。測試直接呼叫正式bootstrap_membership匯入當前公開baseline（不前進tick），再做owner/non-Hero admission檢查。保留staging政策，禁止為測試方便提前模擬Reveal或造假actor；原log與31來源hash保留。另把Direction payload的action-kind alias放進preflight，在world mutation前拒絕。

### E1029 — 探索尾端夾帶錯誤的transport路徑

唯讀探索最後一條Get-Content誤寫omoba-core/src/runtime/transport.rs，真正檔案已知在omoba-core/src/transport.rs，回報不存在。沒有來源變更；不要為湊探索批次加入無目的的讀取，後續每條讀取必須直接服務當前接線與使用已確認位置。

### E1030 — native observer 測試檔名再次猜錯

探索shared JSONL validator時誤寫OmOwnedViewportTests.cpp，實際符號位於OmOwnedViewportJsonlTests.cpp。rg唯讀失敗，未寫入；改在已確認OmEditor/Private目錄搜尋ValidateOwnedWireIntent才讀取真檔。禁止在尚未得到inventory時把預想測試名稱接到下一條命令。

E1026/E1027/E1028修正已由direction-edge-v3接受：core777、client141、bridge120、backend wire2及五checks全成功，31來源SHA相同；沒有UE入口或新權威對局。renderer增量把source/press診斷欄位改canonical decimal string，避免u64經JSON double被四捨五入，C++／Lua觀察器共用嚴格型別與別名拒絕契約；native回歸待普通UE建置後實跑。

### E1031 — ABI workflow 探索夾帶未確認Lua檔名

唯讀rg誤傳scripts/lib/ue_bridge_stage.lua，實體不存在；scripts目錄搜尋cbindgen也沒有結果，故使用已確認的bridge cbindgen.toml與PATH cbindgen.exe。沒有修改workflow或補shell fallback。禁止先填想像中的workflow路徑，直接在已知目錄查真符號。

### E1032 — 重複舊End可能清掉後來的點擊waypoint

renderer gate期間review發現OwnerInputScheme允許matching End冪等重試，但World helper每次Ended均remove MoveTarget。Stop／point command退休held後，延遲End不該修改新point導航。修為只有admission前實際仍held才清方向waypoint；高水位保留、不把重複End當新press。另統一public WorldAPI與prehook的zero-delta拒Begin/Update政策。新增retired release保留後來point waypoint與zero-delta原子回歸；尚待下一輪gate。

### E1033 — ABI增量先跑Rust gate而尚未同步生成header／舊smoke

abi-v1 review發現正式Rust summoner layout仍以ward_slot+32作結尾，maintained C++ smoke甚至停ABI26／command104bytes，header目前仍ABI27。新ABI28須先更新真正layout assertions並由cbindgen生成header再跑gate；不能只更新臨時測試。abi-v1尚未跑到這兩個assert，先因E1034編譯失敗退出，不能宣稱這是執行assert失敗。修同步OmInputCommand184／owner_control56／owner_scheme232與frame末端相對offset；沿用動態版號MSVC maintained smoke runner。

### E1034 — 新測試草稿錯認OmVec2實作Default

abi-v1 core779／client143成功，bridge測試編譯E0277：OmVec2沒有Default，新增clean owner-command helper誤用Default::default。已讀真型別，改明確OmVec2{x:0.0,y:0.0}，不為測試新增production trait。37來源SHA前後一致，原log保留；本輪backend/content因fail-fast未跑，不能冒稱成功。

### E1039 — 未讀runner實際輸出目錄就查詢native artifact

ordinary build成功後native-v1執行中，唯讀Get-ChildItem誤假設輸出位於.ai-collab/owner-control-native-v1，實際runner使用.ai-collab/.bridge-data/worker-pool-machine-v1/owner-control-native-v1。沒有修改測試或來源；重新讀取runner的local dir後才查真目錄。避免沿用Rust gate的輸出路徑慣例，必須先讀producer定義再追蹤artifact。

### E1038 — 文件patch再次使用不完整的行作context

原生接線時把完整長段落的開頭「現在實作ABI28：」當獨立行做patch，apply_patch驗證失敗；同批production退休清理也未落地，已重新讀取真來源並獨立套用。不得把文件與production修改混在含猜測context的patch；長行須使用已讀到的完整行，或改用穩定標題位置。

### E1037 — ABI欄位盤點漏掉integration smoke的三個constructor

abi-v4 core779／client143／bridge124與backend wire2成功，core／client all-target成功；bridge all-target在runtime_driver_smoke.rs的三個OmInputCommand constructor報E0063缺owner_control。原lib-only範圍沒有涵蓋這些constructor；已對整個bridge目錄盤點並補零值owner_control，將此integration來源加入gate指紋。後續backend/content尚未執行，不能宣稱整輪通過；原log保留。

### E1036 — 既有frame結尾斷言漏掉新增的inline操作模式

abi-v3 core779／client143成功；bridge123成功／1失敗／1既有ignored，唯一失敗為summoner layout沿用owner_combat_preferences作frame末端，實際720bytes、舊預期488bytes。已補上combat至owner_input_scheme的相鄰offset與新末端尺寸兩項斷言，保留前面欄位的相容位置檢查。避免只改command layout卻漏掉frame layout；每次append須逐一盤點所有size_of與offset_of相關斷言。原失敗log保留，待新gate驗證。

### E1035 — 既有feature測試仍硬編碼ABI27

abi-v2 core779／client143成功；bridge121成功／3失敗／1既有ignored，新mode/direction/inline snapshot測試全部成功，失敗是Stop／ShopRecipe／Summoner的ABI27 literal。先前只改新frame版號assert，漏掉其餘同類與兩個舊header字串，另ignored測試也有literal。已盤點全部27，保留真正拒絕ABI27的negativecase，其餘feature測試改比較om_abi_version()與維護常數，header字串依常數格式化。專門ABI28 layout測試仍明確檢查版號與尺寸。cbindgen ABI28生成／staged byte identity與MSVC maintained smoke已真成功，不能把本輪三失敗當作新ABI行為通過整輪。
