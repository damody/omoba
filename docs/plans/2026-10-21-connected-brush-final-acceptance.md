# 連通草叢與施法操作提示：最終驗收

OpenSpec `connect-moba-brush-regions` **5/5完成**，全程未新增權威對局。

## 實作與通用規則

草叢以與地表相同的膠囊形狀建立連通元件，支援直接相交、切線與多段轉彎，與資料順序無關。載入時計算並以 immutable Arc 分享；逐 entity／fog cell 不配置圖、不重算相交。doubled fixed-point center segments 與雙 limb 平方涵蓋 i64 raw 全域，避免溢位。角色公開視野、即時目標檢查與霧格共用圖，保留 range／wall／stealth、顯式隔離及既有兩秒普攻揭露規則。

兩張現有三路 map 各由四段增加為六段草叢，加入鏡像連通分支。保留 bounds、32段上限與唯一 ID，允許不同 ID 的膠囊相交；支援膠囊聯集，尚非任意多邊形 authoring。

技能 tooltip 讀取目前 PlayerControlProfile，顯示一般施法、快速施法或按住指示／放開施放，跟隨改鍵，未綁定明確提示。無指定目標技能按下即施放，圖示點擊的強制瞄準行為分開說明；未知、未學習與退休不留舊操作提示。原有目標類型、距離與半徑保留。

## 驗收結果

| 項目 | 結果 |
|---|---|
| RustV3 | **1653通過／10既有ignored**：template59、core812、codegen57、base246、backend190/1、bridge127/1、client147/8、ABI15；9個工作包含生成均exit0並退休 |
| Lua契約 | local69、owned140及map ID契約成功 |
| UE5.8 | 凍結後build-v2成功，UBT23.86秒；正式backend-v1成功 |
| WorldV1 | 30項：29 Success＋1 SuccessWithWarnings、4 warnings，0 Failed／NotRun；兩map各74影格／6片段 |
| 完整local-v1 | **209 native**：201 Success＋8 SuccessWithWarnings、12 warnings；0 Failed／NotRun |
| 實際owned輸入 | **458案例／222 intents／157 stages／13 cursor**，完整phased coverage成功 |
| Blueprint／音訊 | **69 Blueprint**、0 compiler error／failed load；**5段實際submix PCM**成功 |
| 來源與產物 | **3340來源／資產**接受前後SHA、size、mtime及集合一致；2個derived outputs依current build、接受前staging及import library SHA另驗 |
| 證據／監測 | **20份原始JSON**逐byte封存並獨立複核；完整流程8次健康取樣，180秒排程UTC最大181秒，期間另持續讀取最新log；12個owned工作與12個watcher全部退休 |

## 實際畫面複核

已目視 training overview、layered game-viewport-biomes、training camp-0、layered game-viewport-fog。地圖維持左下／右上基地、三路、河流、塔與野怪；連通草叢分支已顯示。原始MJPEG影格350／590／802已目視既有施法預覽的改鍵F10、放開確認、敵方需求與射程外文字可讀。新版tooltip沒有在取樣錄影中展開，其三施法模式、改鍵與未綁定內容由Om.Runtime.AbilityTargetTooltip及完整native測試證明；不冒稱新增tooltip已有截圖。

## 接受界線與保留紀錄

phased本機成功不等於單session效能或真對局ACK；原MVP仍 **33/44，11項外部ACK／LAN／OS DPI／效能接受待驗**。沒有新增對局、重設配額、commit／push或維護omfx。仍可續做任意多邊形編輯、特殊script技能形狀／metadata、史詩怪增益／分裂與專用美術。

失敗RustV1、補整合測試後主動退休RustV2、來源capture-v1的false receipt與過期build-v1均保留，未當作最終成功。最終依RustV3、build-v2、WorldV1與local-v1。錯誤見[同日紀錄](2026-10-21-connected-brush-errors.md) C01–C09；原始證據見[封存index](../acceptance/2026-10-21-connected-brush/index.json)。OpenSpec CLI未安裝，未宣稱CLI validate。
