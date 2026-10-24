# 牆體視野與單位目標提示：錯誤紀錄

## W01：先盤點再讀測試路徑

調查時誤猜base_structure_tests.rs，實際沒有此檔。改由rg實際列出的input_scheme_tests.rs等既有World fixture建立新測試；不重試不存在路徑。

## 已知驗收原則

沿前輪P08：正式來源盤點要分開source與建置產物。Source/ThirdParty的om_bridge.lib及scripts/base_content.dll是derived outputs，須核對current build、接受前staging SHA與時間，不能假稱來源未變或直接忽略差異。所有失敗cohort保留，新輸出必須唯一命名。

OpenSpec CLI目前未安裝，手動維護spec-driven artifacts，不聲稱CLI validate成功。

## W02：MobaMapConst fixture 要求static資料

WallGuidanceRustV1新增四個測試陣列使用區域let，terrain欄位是static slice，core test編譯正確拒絕E0597。改const陣列，保留正式schema，不用洩漏記憶體或放寬lifetime。失敗V1保留，完整重跑V2。

## W03：第一輪監測未及時掛入

V1沒有完整180秒watcher紀錄，不能事後補造三分鐘檢查證據。後續runner在啟動內部立即建立owned identity監測，最終接受僅採有監測的新cohort；保留本次缺口。

## W04：技能測試必須使用實際loadout的英雄

編譯前檢視generated metadata，確認lumen_aid屬training_support，不是training_luminary。新C++fixture改用TrainingSupport，保留對ally的精確斷言；不增加硬編碼product分支。

## W05：無焦點的即時音訊錄製被引擎靜音

LocalAcceptance wall-guidance-20261021-v1 的 OriginalAudioOutput 實際 routed-mix.wav 為零，app_volume=0、device_primary=0。UE WindowsPlatformApplicationMisc.cpp 使用 UnfocusedVolumeMultiplier（預設0）設定背景音量；不是資源或技能實作錯誤。正式音訊runner改用本次程序專屬 -ini:Engine:[Audio]:UnfocusedVolumeMultiplier=1.0，讓無人操作的背景錄音可重現。不修改持久設定、OS音量、使用者焦點，不放寬五段PCM振幅斷言。保留失敗cohort及WAV，使用新run-id重新完整驗收。


## W06：編輯器另有每幀背景靜音政策

V2仍錄到零PCM；只有Engine UnfocusedVolumeMultiplier不足。實際UE EditorEngine.cpp 1905起每幀依LevelEditorMiscSettings.bAllowBackgroundAudio／EditorVolumeLevel覆寫FApp音量。補同樣僅限此驗收程序的EditorPerProjectUserSettings命令列覆寫，保留PCM斷言，先獨立音訊驗證再開完整V3。不得把第一層推論當已證明的根因修正。調查另有猜錯檔名查找失敗，後續先依rg盤點結果讀取，不把不存在路徑重試當工作進展。
