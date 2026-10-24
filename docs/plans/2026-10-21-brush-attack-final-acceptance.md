# 草叢普攻揭露與技能tooltip：最終驗收

日期2026-10-21，D:/omoba，UE D:/UE5.8（5.8.4），Rust1.95.0。未新增權威／網路對局，未重設配額，未commit／push。歷史omfx不納入維護或驗收。

## 實作

- 共用有限期RevealBrush權限：草叢內已成立的普攻對受擊的非中立敵對隊伍暫時揭露攻擊者，期限兩秒模擬時間，刷新延長、暫停不倒數、死亡／退休／換map清理。英雄、script attack cue、兵線及中立野怪（含附屬怪）的實際出手入口都接線；同隊及中立受擊方不授權，非普攻技能不自動暴露。
- 只略過草叢遮蔽，仍要求觀察者距離、牆體LOS與stealth detection；ServerOnly／ForceHide優先。Wave B及即時unit item一致，committed可見性取得目標可沿原路使用。期限與身分納入重播digest，不發布私有grant清單或deadline。
- Fog只照亮已合法公開攻擊者的所在格，保持其他草叢 occupants 隱藏，不生成VisionSource、不揭露整片brush；過期仍保留既有explored地面語意。
- 技能tooltip共用已公開形狀／ally-enemy需求、施放距離及point落點半徑，依WorldUnitsToCm換算，保留原有說明與legacy label。未知／未學習／非法浮點／非point替換／退休不顯示過期數值。名稱、圖示與文字slot沿同一提示邏輯及既有可捲動tooltip host；非per-entity UI更新。

## 已通過

- RustV5：template59、core807、codegen57、base246、backend190、bridge127、client147、ABI15，**1648通過／10既有ignored**。
- 八個新core測試涵蓋真正native／script／lane／neutral camp admission、第三隊觀察者、刷新／精確到期、暫停、死亡、獨立World摘要、其他藏身單位及fog單格、range／wall／stealth／ForceHide／ServerOnly負向條件。
- Lua local契約69、owned IPC契約140及多map ID契約成功。
- UE首次build-v1 UBT48.48秒；野怪修正後最新build-v2 UBT3.34秒、backend-v2正式建置及staging成功。
- 最新WorldV2 **30項成功**（29 Success＋1 SuccessWithWarnings／4 warnings），含Om.Runtime.AbilityTargetTooltip。兩map有各74影格／6片段輸出；實際看過overview／layered biomes／fog／camp四張圖。靜態圖不當草叢行為證據。

最終local-v2完整成功：**209 native、458案例／222 intents／157 stages／13 cursor、69 Blueprint、五段實際PCM**。native為201 Success＋8 SuccessWithWarnings、12 warnings（含音訊警告）；Blueprint 0 error／0 warning。

接受前後**3340個來源／資產檔一致**，另核對current build／接受前staging及import library SHA，沒有把2個derived outputs當來源。19份原始JSON逐byte封存並獨立複核；完整工作流7次健康取樣，180秒排程的UTC觀測最大間隔181秒，期間另持續讀取最新log確認進度。所有owned工作流與watcher已退休。

最終Source-before-v3／verified-v3及staged-v2分開驗證；新舊cohort精確transition只有三個Rust檔案，Unreal來源未變。原始來源／staging／退役收據／影格與目視紀錄見[封存索引](../acceptance/2026-10-21-brush-attack/index.json)。Source-before-v2於fixture修正期間捕獲，不作最終凍結基準。

## 失敗保留與界線

V1 canonical ID fixture、V2整個grant map誤判、V4私有camp欄位編譯失敗，皆保留原始證據；V3 Rust／WorldV1雖成功，但覆核發現漏接野怪入口，已主動退休完整local-v1並以新cohort重新接受，不能採舊摘要冒稱全接線。詳見[錯誤R01–R06](2026-10-21-brush-attack-errors.md)。

兩秒是目前明確遊戲規則，不宣稱精確複製特定LOL版本。未新增連接多邊形草叢、特殊手寫技能精確扇形／路徑、史詩怪增益／分裂或專用美術。完整MVP仍33/44，11項外部ACK／LAN／實際OS DPI／效能接受保留。本輪為記憶體World及synthetic renderer／IPC，不冒稱新對局、真權威ACK或單session效能。

## 視覺與功能證據界線

四張最新UE overview／layered biomes／fog／camp圖已實際檢視，兩map維持左下／右上基地與三路。關閉owned AVI後才抽取原始JPEG，不改寫影格。技能tooltip的公開目標類型、己方／敵方需求、單位換算、未學習／非法數值／退休由實際UE原生測試證明；原始frame850／1000已實際看見新增「目標：指定地點」「施放距離：450」，800×600可讀；frame850同時保留確認／取消提示。己方／敵方及落點半徑字樣本批未抽到，採native測試證明，不冒稱全都有截圖。草叢普攻揭露採真實World admission／Wave B／immediate item／fog測試證據，不冒稱靜態地圖圖已驗證動態行為。

本輪OpenSpec add-moba-brush-attack-reveal **6/6完成**。音訊沿前輪process-local Engine與Editor背景設定，五段實際48kHz立體聲PCM成功，未用來源WAV替代，也不宣稱實體喇叭聽感已驗。
