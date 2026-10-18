# 本機原生與資產驗證入口

## 計畫與範圍

接續機器 handoff，以 Grok Build 處理舊 native BodyMesh 覆寫相容性；primary 獨立審查、正式重編並驗證。另把本輪已證實可執行的官方原生測試與 CompileAllBlueprints 整理成 Lua 5.4 入口，避免換機或新增測試後再靠手動計數。這項結果不取代 vendor protected MCP、真人原場 replay、same-PIE 完整對局、十二效能門檻或真雙機 LAN 驗收。

## 實作

`scripts/ue_local_acceptance.lua --ue-root PATH [--run-id UNIQUE_NAME]`：先驗選定引擎 manifest／stage SHA 與 bootstrap checkpoint，使用既有 owned workflow supervision。預期原生名稱由正式 OmEditor／MCP／PIE planner C++ macros 讀取；只納入原先 game-native 套件範圍，排除 Linked TD simulation。預期 Blueprint 清單由現有 Content/RustBP 的 BP_／WBP_／ABP_ uasset 建立。兩個官方 commandlet 串行執行，不 save 既有資產、不啟動 match；證據使用全新 Saved/Automation/LocalAcceptance 目錄，拒絕覆寫。

`ue_local_acceptance_contract.lua` 核對逐項完整名稱、重複／缺項／額外項目、狀態、錯誤、warning 與 summary counters；程序 exit0 或空報告不足以通過。UE native warnings 保留計數，不隱藏。Blueprint compiler errors／warnings／failed loads 必須全零。執行失敗保存 errors.md 與 success=false summary，原始 engine logs／JSON 不改。

## 實際局部驗證與錯誤

- 26 個契約案例 PASS，包括零測試、缺項、重複、額外、失敗 state、counter 不符、compiler errors／warnings／failed loads 與 BOM。help PASS。
- 同一新入口／契約測試從含空白、非 clone root 的實際 CWD 以絕對 Lua／script 路徑啟動：26案例與help再次PASS，正確取得原clone；沒有UE或match啟動。
- 對上一輪未改寫的真報告重新核對：native-v4 70／70，63 succeeded＋7 succeededWithWarnings，13 warnings，failed0／notRun0；官方 Blueprint-v2 69／69，compiler errors0／warnings0／failed loads0。這是新 auditor 對既有證據的驗證，尚不是本轮新增 commandlet 執行。
- 對原始失敗證據做真正負驗證：native-v3 的 NativeAndAuthoredMeshCompatibility Fail 與 Blueprint-v1 的46 compiler errors 均被新 auditor 拒絕，原始報告不修改；判定證據存於 ignored `.ai-collab/.bridge-data/local-acceptance-audit-machine-v1/negative.json`。
- 首次 auditor 讀 UE 原始 index.json 失敗：JSON 在 byte1 非法。原因為官方 exporter 的 UTF-8 BOM；只在報告讀取邊界去掉 UTF-8 BOM 後解碼，不修改原始檔或全域 JSON parser。真報告與帶／不帶 BOM、新 UTF-16 拒絕案例均已驗。
- Primary 曾再次猜不存在的 validate_ue_pie.lua；後續只用 rg --files 已發現的 ue_pie_smoke.lua 等真入口。另一次過大 rg 全文截斷，改只印四個 unchecked tasks 與必要來源段落。禁止以猜路徑與大量截斷輸出代替發現／閱讀。

Grok job run-muxm1bdq-7oq4qy／thread dbb38f2c-f583-4c6c-bfdc-d8bd7d4c27b3 completed33m2s，USD3.6198814；input539926／cached17011968／full17551894／output122510／reasoning104440／total17674404／calls126／turns126／API duration未知。只改 OmUnitActor.cpp 與 OmEditorAutomationTests.cpp；其實際 final 明說未 build／test，bridge summary只是最初句，因此讀公開 storedJob.result.rawOutput 確認。不可把 completed當驗收完成。

Primary 第一輪正式建置44.52秒FAIL：新fixture把 TRefCountPtr 當 TSharedPtr 呼叫不存在的 Get()（C2039／C2672），另 GetObjectsWithPackage bool overload C4996。讀UE5.8真宣告後改 GetReference() 與 EGetObjectsFlags::IncludeNestedObjects；原始 log 保存在 native-serialization-machine-v1，新v2重試。未改assert或production以迎合失敗。

Grok production會改export ObjectName及hash。Primary 查 `LinkerLoad.cpp:4394` 的 VerifyImport：import依原名字hash及 SourceExport.ObjectName 相等查找。因此尚不接受這個方案為通用相容性修復；單一map的index references成功仍不能证明跨package舊import可用，后续須修復並做必要驗證。

API修正後正式 NoEngineChanges14.58秒＋兩個 stage SHA PASS。新Lua入口真實跑71項：70Success（63無warning＋7有warning），新增legacy fixture FAIL1，notRun0；Cmd exit0但Lua正確exit1並寫errors.md／success=false，Blueprint階段未執行。原因為EditorPreview World未有 RF_Public／RF_Standalone，SavePackage明確拒絕資料損失；primary只在隔離新World設定正確save flags，不開save.FixupStandaloneFlags／不弱化assert。另擴充同一fixture，以隔離第二package的UObjectRedirector實存跨package舊native component import，準備檢驗Export改名缺陷；既有project content不save。

Cross-package fixture第一輪兩項中既有meshPASS、新legacy FAIL：Source map PersistentLevel是private，因此SavePackage正確拒絕外部參照。這還不能當production的import斷參照重現；已只把隔離新World的Level設定RF_Public，使整個外部owner chain可import，再正式重編。原始拒絕log與report留在 native-serialization-cases-v1，不用忽略save error或Engine開關通過。

合法public owner chain後，native-serialization-cases-v2真實重現production缺陷：單mapoverride與既有mesh通過，但第二package的StaticMeshComponent舊BodyMesh import失敗，DestinationObject不等於migrated native。Primary改以legacy Export.Object綁constructor新component，保留ObjectName／hash；用正常SetLinker、RF_NeedLoad／PostLoad flags與既有serialize context AddLoadedObject接UE deferred load。只在loading actor／CDO與live loading context介入，不在普通gameplayspawn掃export表；owner用實體outer比對並支援import outer／external GetPackage。新舊名稱分離不變，SCS非RF_DefaultSubObject不介入。新compile與相同負fixture轉正尚待實跑，不預先宣稱成功。

Primary 另核對實際 UE5.8 `CoreUObject/Private/UObject/Class.cpp:1342`：tagged property 先沿 owner struct super chain 查 FindRedirectedPropertyName，後於1357查 FindPropertyByName。不能假設 BP 已有同名 authored 欄位會先命中並免受父類 redirect 影響。`UObjectGlobals.cpp:4243`／4247先初始化 actor 與 subobject defaults，4320才呼叫 PostInitProperties；來源證據只是審查約束，未當作 migration 已完成。

## 2026-10-07 匯入身分與 load admission 審查

只改 `omfue/Plugins/OmRuntime/Source/OmRuntime/Private/OmUnitActor.cpp` 的 `ExportOuterIsThisActor` 匯入 outer。`OmEditorAutomationTests.cpp` 的 `NativeAndAuthoredMeshCompatibility` 與 `NativeBodyMeshLegacyOverride` 未改：sentinel、跨 package redirector、SCS／`RF_DefaultSubObject` 區分、插值與 package cleanup 都留在原 fixture。本輪沒有編譯、沒有跑測試，不把這次原始碼修正當成驗收通過。

編譯錯誤是具體的存取錯誤。`FLinkerLoad::IndexToObject` 在 `LinkerLoad.h:993`，位於約 844 行開始的 private 區。實作 `LinkerLoad.cpp:6697` 對 import 會呼叫同為 private 的 `CreateImport`，再回傳 `Import.XObject`；index 越界在該函式內是 fatal。遊戲模組不能呼叫它，也不能改走 `FLinker::Imp`／`ImpPtr`（`Linker.h:140`、`157` 對越界 `check`）。

公開替代是 `FLinker::ImportMap`（`Linker.h:68`）與 `FObjectImport::XObject`（`ObjectResource.h:479`）。註解寫明 XObject 是該 import 第一次 `CreateImport` 後的 UObject。`FPackageIndex::ToImport` 只在 `IsImport()` 後合法，不檢查陣列長度，所以先 `ImportMap.IsValidIndex`。越界或 `XObject == nullptr` 都拒絕綁定。不另外追 `UObjectRedirector::DestinationObject`：actor 的實際 outer 就是當時已解析的那個指標，再追一次會把別的物件當成 outer。

這個呼叫點讀已解析身分是成立的。`CreateExport` 在 `LinkerLoad.cpp:6066` 先對 actor export 的 `OuterIndex` 做 `IndexToObject`，`6395` 才 `StaticConstructObject`。constructor 內看到的 import outer，其 `XObject` 已經是 `Actor->GetOuter()`。讀取它不會重入 `CreateImport`。

Load／PostLoad 准入對過公開契約後維持原綁定，沒有改名、改 hash 或改 flag 組合：

- 預先寫入 `FObjectExport::Object`（`ObjectResource.h:296`）後，`CreateExport` 開頭 `LinkerLoad.cpp:5922` 的 `if (!Export.Object && !FilterExport)` 整段略過，直接回傳已綁物件。constructor 必須自己完成准入，否則舊 `BodyMesh` export 不會序列化也不會 PostLoad。
- `UObject::SetLinker`（`Object.h:1450`）寫入 linker index。`Preload`（`LinkerLoad.cpp:5408`）要求 `GetLinker()==this`、index 有效，且 `Export.Object==Object`（`5483`）；只有 `RF_NeedLoad` 才序列化。
- `FUObjectSerializeContext::AddLoadedObject`（`UObjectThreadContext.h:197`）只是追加到 `EndLoad` 清單（`UObjectGlobals.cpp:1992`）。`EndLoad` 對 `RF_NeedLoad` 呼叫 `Preload`，之後 `ConditionalPostLoad`（`Obj.cpp:1412`）才在 `RF_NeedPostLoad` 下呼叫 `PostLoad`。
- 元件不是 CDO。已存在物件的非 CDO 旗標是 `RF_NeedLoad | RF_NeedPostLoad | RF_NeedPostLoadSubobjects | RF_WasLoaded`（`LinkerLoad.cpp:6256`）。CDO 才省略 `RF_NeedPostLoadSubobjects`（`6250`），而且條件是物件自己帶 `RF_ClassDefaultObject`。目前 `SetFlags` 對這個 mesh 使用非 CDO 組合。
- `FObjectExport::ObjectFlags` 公開註解（`ObjectResource.h:260`）與 `RF_Load`（`ObjectMacros.h:643`）規定只有該遮罩會套到物件。磁碟讀取已做 `Save & RF_Load`（`ObjectResource.cpp:143`）。現有 `SetFlags(Export.ObjectFlags | load flags)` 套用的是已遮罩的序列化旗標，不另改。
- `SetExternalPackage(LinkerRoot)` 只用於該 export 自己的 outer 是 import（`LinkerLoad.cpp:6240`）。舊 mesh 的 outer 是同一 package 的 actor export，不設 external package。
- `ObjectName` 與 `ExportHash`／`HashNext` 都不改。`VerifyImport`（`LinkerLoad.cpp:4394`）用 `GetHashBucket(Import.ObjectName)` 走 hash，並要求 `SourceExport.ObjectName == Import.ObjectName`（`4408`）。先前改 export 名或 hash 會讓第二個 package 的舊 `BodyMesh` import 找不到來源。外部參照要的是保留序列化名稱。

仍保留的拒絕條件：actor 須 `RF_NeedLoad` 或 CDO、serialize context 已 `HasStartedLoading`、package 非 transient、export 為 `RF_DefaultSubObject` 且非 `RF_InheritableComponentTemplate`、類名 `StaticMeshComponent`、outer chain 是這個 actor。已有 `OmNativeBodyMesh` export 則整段中止。SCS `BodyMesh` 沒有 `RF_DefaultSubObject`，不會被這條路徑綁走。

Fast loading repair run-muxoets3-95v8hx was cancelled after about15min of unchanged reading-stage progress, no source edits observed after cancellation; cost unavailable, do not report zero. Primary narrowed next fresh Fast job run-muxoy50h-yn8mj5 to temporary bounded runtime diagnostics and mandatory linker assertion, avoiding another broad speculative serialization review. Two current tests remain Success1/Fail1 (4 errors), no full acceptance repeat.

## 2026-10-07 async 守衛與錯誤的 LoadContext 成員

上一則指令把 `LinkerLoad.h:107` 的 `LoadContext` 說成 `FLinkerLoad` 的公開成員。該名稱屬於 `FScopedCreateImportCounter`，不是 linker。承接該指令的 worker 已取消，沒有依那個成員改來源。本輪不讀引擎、不發明 `FLinkerLoad::LoadContext`。序列化 context 仍只使用 `FUObjectThreadContext::Get().GetSerializeContext()`。

證據只有 `native-serialization-cases-v5` 這兩個測試：`Om.Runtime.NativeAndAuthoredMeshCompatibility` Success；`Om.Runtime.NativeBodyMeshLegacyOverride` Fail，5 errors。`FlushAsyncLoading` 之後 `OmFallbackUnitActor_0` flags `0x00083609`（含 `RF_NeedLoad`），同一執行緒 `FUObjectSerializeContext::HasStartedLoading` 為 0。非同步載入不要求該執行緒 `ObjBeginLoadCount > 0`。把 `HasStartedLoading` 當必要守衛會拒絕正在載入的 actor，舊 `BodyMesh` 沒有綁上 constructor 的 native mesh。上文「仍保留的拒絕條件」裡的 `HasStartedLoading` 條款因此撤回。`OmEditorAutomationTests.cpp` 的診斷輸出與 `Migrated component linker` 斷言保留，v5 中該斷言仍失敗。

本輪只改 `AdoptLegacyNativeBodyMeshExport`：拿掉 `HasStartedLoading` 必要拒絕與全部暫時 helper `UE_LOG`。保留 actor `RF_NeedLoad`／CDO、非 transient package、實際 package linker（`GetLinker` 或 `FindExistingLinkerForPackage`）。相符的 legacy export 在建立前綁到既有 native object；`ObjectName`／hash 不改，`SetLinker` 與 need-load／PostLoad flags 保留。`AddLoadedObject` 只在 serialize context 存在且 `HasStartedLoading()` 時呼叫，供同步 `EndLoad` 收錄；async 路徑自己消費 export table。沒有編譯、沒有跑測試，不把這次原始碼修正當成驗收通過。

## 2026-10-07 Fast 修復與當前功能獨立驗證成功

診斷 job run-muxoy50h-yn8mj5／thread01aed398-e064-4ef9-ad5e-bdb2f09d7752 completed212s／USD0.2664648；8calls／310992total tokens。實際v7建置21.42s通過，cases-v5仍FAIL：loading actor RF_NeedLoad已設，但thread HasStartedLoading=0而提前return；舊BodyMesh仍獨立載入且持有完整sentinel，external reference指向舊物件。新增mandatory linker assertion真實FAIL，避免optional檢查漏掉未綁定。

Primary誤把LinkerLoad.h107的FScopedCreateImportCounter.LoadContext看成FLinkerLoad成員，run-muxp49wb-by6xmb因此約1m取消，成本未知；錯誤交接不得直接採用。FLinkerLoad沒有該member，不新增private API或Engine修改。更正packet後run-muxp6h7l-i8k0dr／thread17080a93-78b8-4bb8-8530-4e88587de4bc completed143s／USD0.37393472，14calls／583436total tokens。Grok移除錯誤HasStartedLoading硬guard，保留RF_NeedLoad／CDO、真package linker、exact owner/class/native default-subobject檢查；sync時有BeginLoad才AddLoadedObject，async由既有exporttable載入。舊ObjectName/hash保留，暫時production log全移除。測試diagnostic與mandatorylinker保持，斷言未放寬。

Primary完整檢查相關修改後v8正式NoEngineChanges12.71秒exit0、兩個stage SHA一致。全新cases-v6實際兩項exact names均Success、succeeded2／warnings0／failed0／notRun0／errors0：跨package BodyMesh舊import指向同一OmNativeBodyMesh，舊scale/location與native flags/linker、authored SCS區分/visibility、frame→tick interpolation皆通過。這是同一真失敗fixture轉正，當前功能接受。既有Content git status乾淨，root/submodule HEAD未移動。最後完整本機入口machine-native-final-v1正在跑，不預先宣稱通過；尚無live/auth/真人capture/真雙機LAN新證據，新對局仍0、batch8/10。

最後完整本機驗收 machine-native-final-v1 實際Lua exit0／summarysuccess=true：native71exactnames全Success（64＋7withWarnings、13warnings、failed0／notRun0），Blueprint69exactnamescompiler errors0／warnings0／failedloads0、兩Cmdexit0。當前feature接受；外部整合未冒稱完成。詳見交接最新段與state/review。

最後metadata核對錯誤：Primary為壓掉CRLF提醒而臨時用core.autocrlf=false做diff --check，導致Windows原有CRLF被當trailing whitespace，巨大輸出截斷。這是檢查方式錯誤，不能因此改所有檔案行尾。已恢復仓庫正常設定、將兩repo diff-check輸出保存ignored log，只回報exit code；strict Lua JSON驗證state成功、active_job_id=null、final local success=true。後續不為減少warning改變Git行尾語義，也不直接印無界diff診斷。
