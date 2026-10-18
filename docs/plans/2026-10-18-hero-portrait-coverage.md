# 正式英雄頭像覆蓋

## 最終接受（2026-10-18）

已完成本批功能與完整本機驗收：`.ai-collab/hero-portrait-accepted-v1/summary.json`。七位正式英雄皆有可解碼頭像，新增兩張原創圖片；45筆manifest保留原43筆內容，其他既有圖片、69 Blueprint與九地表資產未改。1148來源／資產SHA與保護mtime最終一致；fresh1168 Rust、五checks、九既有ignored與ABI30 MSVC通過。ordinary build-v2 27.10秒，focused native-v2八項成功、零warning，七位英雄及兩map共九張1280×720實體viewport原圖直接檢視。完整final-v1真197 native成功（12既有warning保留）、458操作／222 wire／157 stage／13游標觀察、69 Blueprint零compiler／load診斷、三份實際PCM及全部程序清理成功。

E1278的第一版裁切圖拒絕紀錄保留；修復僅更換測試的正式GameLayer與LocalPlayer生命期，不改產品layout或OS DPI。完整Rust後唯一delta為此editor fixture，已明列於接受紀錄。當前仍26／44，頭像覆蓋不單獨滿足4.1／6.2整項；無新權威對局／ACK／LAN／實體控制器驗收。以下中間進度保留歷史，以本段為準。

正式圖為`omfue/Content/Omoba/UI/training-apprentice-v1.png`與`training-support-v1.png`。使用內建imagegen、無reference、無像素後製；完整prompt與原始輸出／SHA存於同目錄`hero-portrait-artwork-provenance-v1.json`。下一批先盤點Voice類別與既有生命期，再制定通用播放／字幕／音量與素材來源方案。

前批公開河道／野區已完整接受：`.ai-collab/map-biomes-accepted-v1/summary.json`，最終source-v8的1024來源一致，所有consumer退出。當前26／44。以下計畫現在直接接續實作；不得修改前批接受紀錄或把預備素材算成已接入。

Codex直接實作，不使用子代理。接河道／野區當前功能驗收；在前批consumer退出及證據核對前，僅準備本文件與獨立原創素材，不更動UI manifest、正式圖片或來源。

## 已確認缺口

正式heroes目錄有七個ID，其中目前三路fixture使用六個；既有HUD manifest僅有五筆portrait，缺training_apprentice（晨光學徒）與training_support（晨光援護者）。現行HudAbilityIconAtlas測試亦只列五個已知英雄，因此通過仍漏掉這兩筆。實際整合圖的隊友2使用training_apprentice，顯示問號。未知英雄的明確fallback與來源退休仍是必要行為，不能藉補圖放寬公開資料或保留前一位英雄圖。

## 實作計畫

1. 以內建imagegen產生兩張獨立原創、opaque方形頭像：年輕女性晨光學徒與成熟女性綠兜帽援護者，依正式native visual的female_peasant／female_ranger定位；沿用既有海軍藍背景、清楚面部與金／翠綠光源的畫風。記錄完整prompt、工具、原始輸出路徑、SHA、尺寸；保留原始圖片，byte-identical複製至專案，不覆寫五張舊圖。
2. 新增兩筆精確content ID的portrait manifest entry，正式中文名由既有Lua authoring投影。使用既有有界圖片載入與公開HUD路徑；不加入ID別名或角色特例。
3. 將既有build-time HUD同步補成完整hero coverage守衛，從實際Lua目錄取得英雄集合，缺圖註冊時給出明確ID診斷；同一唯讀守衛納入verify-staged-only前置，檢查已知portrait的固定namespace、實際PNG檔／尺寸與square tile，不要求atlas像素整除。同步先在記憶體更新名字，全部檢查通過才寫manifest。保留runtime未知fallback契約。自足Lua案例驗完整集合、遺漏、壞檔與錯誤註冊，更新隔離stage fixture的新依賴，不依賴歷史Saved檔。
4. Native圖示測試改由正式generated hero catalog逐一取portrait，驗全部七個ID；既有未匯出的全域array不能直接跨DLL存取，因此在Rust generator新增正式匯出的唯讀ArrayView accessor，走正式generate，不手改產物。registry測試亦從同一目錄檢查，移除43筆舊固定數字，負例以實際接受count檢查狀態未被破壞。再驗未知、清空、來源退休與late update不復活。以真正Slate卡片／主英雄UI截圖確認新增圖片實際載入、方形比例及小尺寸可辨識，不以PNG存在代替接線。
5. 前批河道／野區已完整收尾，之後才接入並建置；保護全部舊UI圖片、manifest原43筆與69 Blueprint SHA／mtime。因Rust generator新增C++目錄API，codegen／bridge與其餘正式Rust檢查須fresh執行，不宣稱Rust輸入未變；FFI ABI30、遊戲規則與權威資料契約維持。當前功能通過後最後完整本機驗收，來源／素材雜湊與所有失敗保留。不新增權威對局或ACK。

## 素材準備

目前進度：兩圖已byte-identical接入，原43筆manifest保留、現45筆；其他既有UI檔與69 Blueprint未改。Lua portrait guard28自足檢查、stage10案例與實際verify-only通過，正式generate／stage成功，ordinary hero-portrait-build-v1 NoEngineChanges／NoUBA／nocache成功28.25秒、ABI30 MSVC成功。source-v1凍結1148輸入（1026維護來源、44UI檔、69 Blueprint、9地表資產），建置後SHA與asset mtime一致。native-v1八項成功，但新standalone portrait完整HUD圖在175% DPI下裁切，明確拒絕版面接受；改用既有實體viewport fixture的修正待Rust退出後套用。兩張既有實體viewport整合圖已直接見到apprentice隊友頭像。完整驗收尚未啟動，不勾父項。

已使用imagegen內建工具產生兩張原創1254×1254方形圖，無輸入reference、無像素後製；模型識別未由工具提供。已直接檢視，byte-identical準備副本位於`.ai-collab/mvp-hero-portrait-art-v1/`，完整prompt、原始路徑與SHA記於同目錄`prompts.md`及`summary.json`。尚未接入正式UI；接入後另提供可攜provenance。
