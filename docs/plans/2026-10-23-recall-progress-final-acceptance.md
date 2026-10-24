# 回城引導進度條：最終驗收

完成complete-recall-channel-progress。共用內容模型統一既有省略default8與1..60整數秒驗證；generated registry從同一作者資料提供總秒數，bridge透過UI-only total送入owned HUD。既有compiled/runtime duration與catalog guards保留，不擴C ABI、不從首次倒數猜總長。Header使用實際共用Slate strip顯示權威剩餘時間與1-remaining/total進度，並提示移動／攻擊／施法可中斷。沒有本機計時外插，零倒數只隱藏，不冒稱回城成功。

## 驗收

- 本批Rust **190項通過／零失敗／零ignored**：content39、runtime template79、codegen67與既有core回城5項（其餘916項filtered）。實際產生、四Lua契約、正式backend與UE5.8建置成功。沒有冒稱重跑全部Rust。
- **World64成功／零Failed／NotRun**，含新增RecallChannelProgress與既有技能升級／道具提示回歸；4既有RuntimePresentationReset warning保留。
- 原生驗證半途首次快照即50%、重送pause不前進、舊tick／sequence不倒退、新快照75%、零與重啟、12種非法／未知／死亡／終局／缺protocol情況、生命替換、runtime reset、retirement與晚到資料；實際共用Strip為HitTestInvisible並能收合。
- **3415來源／資產**集合、SHA、size、mtime一致；7 staged DLL及import library一致。
- **203份證據**獨立逐byte／SHA驗證，**24張PNG**沿已目視像素精確比對及差異目視，review_source可追溯。新圖由實際HUD共用strip呈現三個published進度；不是新網路對局或權威回城完成回執。
- 健康採樣10次、最大間隔171秒；7最終worker、9歷史worker及各watcher與health均退休。RustV1因serialized Root漏欄位而2項codegen測試失敗；補Root／initializer後RustV2接受。WorldV1因fixture沒有Initialize WidgetTree就TakeWidget而崩潰。WorldV2雖64項通過，目視發現地圖時間／KDA標籤回歸；根因是新進度條排序防護擋住共用入口的舊標籤更新。所有reader退休後，改為獨立channel快照並新增標籤回歸斷言，以build／World／source V3接受。全部失敗source／log／summary、engine堆疊、V2圖片與舊來源指紋保留。

## 範圍與接續

錯誤根因／預防見errors.md及E1505。保持原回城快捷鍵、命令／傷害／控制中斷及完成移動的authority規則。既有局部HUD裁切、objective窄strip及英雄／焦點名牌交疊仍有限制。

48h omoba-48沿用2026-10-25 00:33:38 Taipei期限。一般技能channels／zones、整組召喚settlement、完整combat replica生命期、更多原創內容／美術／音效及11外部接受仍pending，MVP33/44不變。未新增真網路對局、quota reset、commit/push或omfx工作。

證據：`../acceptance/2026-10-23-recall-progress/index.json`。
