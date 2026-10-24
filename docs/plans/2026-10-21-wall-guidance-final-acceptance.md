# 牆體視野與技能目標提示：最終驗收

日期2026-10-21，D:/omoba，UE D:/UE5.8（5.8.4），Rust1.95.0。未新增權威／網路對局，未重設配額；本輪使用記憶體World模擬及synthetic renderer／IPC。未commit／push。

## 本輪實作

- compiled_vision_occluders由正式terrain整數bounds直接產生Fixed64四角polygon，與碰撞／公開牆體同源，依名稱排序取得map-local ID，沒有map名稱或水平座標假設。
- compiled map啟動覆寫VisionOccluderSet，兩map各14堵牆進入既有opaque LOS；Wave B公開單位、fog格網、即時單位道具及採用committed可見目標的攻擊取得沿同一視野規則。不同友方來源可各自揭露；Public／Owner／ForceShow等原優先序保留。舊campaign／單路保留descriptor資源，不混入另一張map的牆。
- 由生成的UnitAllegiance與公開rank發布ally／enemy需求，owner slot快取後加入共用施放提示。顯示「需己方／敵方可見單位」，滑鼠所指仍只稱候選，不當權威距離／陣營／可見性確認。未知、非unit、未學習、替換、runtime退休清除舊值。未提供明確metadata的特殊手寫技能保留一般提示。
- 正式即時音訊驗收runner以process-local command-line config固定Engine與Editor背景音訊政策，避免是否有焦點決定PCM結果；不修改持久使用者設定、OS音量或使用者焦點，不放寬聲音／靜音斷言。

## 完整驗證

Rust **1640通過／10既有ignored**；UE WorldV2 **29項成功**；完整local-v3 **208 native、458cases／222intents／157stages／13cursor、69Blueprint、五PCM**成功。native為200 Success＋8 SuccessWithWarnings／12 warnings；World為28 Success＋1 SuccessWithWarnings／4 warnings；音訊1 warning已包含於native統計，Blueprint0 error／0 warning。

Rust套件為template59、core799、codegen57、base246、backend190、bridge127、client147、ABI15。新測試覆蓋碰撞／LOS同源、排序／負座標／切線／牆前後視線、兩map實際World boot、legacy保留、跨牆公開／fog及另一友方視野來源。既有三種子5v5、完整三路生命週期、野怪回頭、群怪、護主、塔優先序及超級兵回歸通過。不是新增服務端對局。

Lua local契約69、owned IPC契約140及多map ID契約成功。UE首次build-v1 UBT65.08秒，最新build-v2 UBT1.72秒，backend-v2正式建置／staging成功。完整接受run-id為wall-guidance-20261021-v3，phased coverage不是單session效能接受。

獨立audio-probe-v3及完整接受內音訊皆完成五段實際48kHz立體聲PCM：routed mix／ambient restored／voice routed非零，master muted／voice muted為零。保留Steam裝置RAW_MODE警告，不宣稱實體喇叭聽感已驗。

## 畫面與證據

實際檢視兩map overview／biomes、fog及camp四張UE圖，基地維持左下／右上，三路、反向河道、牆與野區正常呈現。兩map各74影格／6片段。由完成且已關閉的owned AVI無改寫抽出8張原始JPEG；**frame850**已看見「Q · 已指向單位 · 需敵方可見單位」與「左鍵確認 · 右鍵／Esc取消」，800×600可讀。己方需求通過Om.Runtime.UnitTargetPreview原生測試，本批抽樣未取得其字樣截圖，不冒稱兩者都已視覺接受。原生metadata／owner快取測試同時覆蓋ally及enemy。

正式來源／資產 **3339檔**接受前後一致；2項derived outputs依current build、接受前staging SHA與import library另驗。v1→v3只有音訊runner Lua改變，Rust／Unreal來源不變，明確保留RustV2／WorldV2 cohort。19份原始JSON逐byte封存及SHA獨立複核；完整流程8個健康樣本，UTC最大間隔180秒，另有持續讀取最新測試log檢查；owned workflow與watcher全部退休。

[封存索引](../acceptance/2026-10-21-wall-guidance/index.json)包含原始摘要、source before／verified／transition、staged-before、影格及目視紀錄、owned退休收據；原始大型log／AVI／PNG／PCM留在Saved或.ai-collab指定路徑，不提交建置暫存。

## 失敗保留與接受界線

RustV1新fixture因static lifetime編譯失敗，修const後完整RustV2通過。local-v1及v2均實際PCM為零，已保留失敗WAV／native report，主動退休superseded流程，不當成功。第一層Engine override不足，讀EditorEngine每幀邏輯後補EditorPerProjectUserSettings兩個獨立參數，再獨立驗證與完整v3重跑通過。第一輪監測缺口亦保留，不補造證據。詳見[錯誤W01–W06](2026-10-21-wall-guidance-errors.md)。

本輪OpenSpec add-moba-wall-vision-and-target-guidance **5/5完成**。完整MVP仍33/44，11項外部接受不以本機證據代替；未新增草叢攻擊暫時揭露、連接多邊形草叢、特殊手寫技能精確扇形／路徑、史詩怪增益／分裂或專用美術。opaque LOS只影響視野，不改point AoE可穿牆命中的既有傷害語意。
