# 剩餘非新增對局工作的完整本機驗收

2026-10-19：本輪完成1.2／1.3／4.1／6.2／6.4／6.6，目前 **33/44完成、11項待驗**。使用 `D:/UE5.8`（實際5.8.4），零新增權威對局，10/10額度不重設。可攜原圖、摘要、輸入報告與指紋索引：`docs/acceptance/2026-10-19-nonmatch/index.json`。沒有維護omfx，沒有commit／push。

## 共用實作

- 骨架動畫拒絕負tick與較舊快照，相同tick的合法細化保留；來源epoch及pool identity沿既有生命週期重設。
- nameplate同步回收component與widget root，停止tick也不留灰色背景；合法reveal恢復同一widget，不重複格式化可見actor。
- 傷害／治療文字依實際血條cached geometry避讓。ground cue使用公開表面的渲染高度登記，body cue依鏡頭移至模型前緣；保留深度測試、原公開XY與生命週期。高度只在geometry／lifecycle改變時更新，不逐entity掃描地圖。
- 原生及未修改原Blueprint controller共四次真standalone PIE，驗唯一action owner、RTS pawn／view target、focus／hit-test、windowed／borderless與stop／restart清理。七英雄的真骨架／pooled cue覆蓋動畫時序、pause、fog及generation回收。
- 視窗PNG／錄影／frozen對照共用實際render-buffer範圍與path offset校正；卡片命中共用actual paint／scroll geometry及hit-grid座標，仍要求同一button或原disabled shield。沒有直接呼叫click delegate或硬改單一卡片座標。
- 兩producer共用持續、可共享讀取的有界UTF-8 timeline，每列一次完整write、結束時flush；不逐幀重開或強制同步磁碟。live-reader／UTF-8／界線／close回歸納入正式inventory。Lua先檢查native成功才讀dependent影片，原始錯誤不被次級驗證遮蔽。

## 當次完整接受

| 範圍 | 結果 |
|---|---|
| 普通UE建置 | nonmatch-build-v29成功，UBT14.31秒；local executor、NoEngineChanges、未使用action cache。 |
| 專項 | NonmatchWorldV17：13成功、0失敗、0warning，含實際AVI decoder與共享timeline writer。 |
| 正式native | nonmatch-final-v5：203成功；8項帶12筆既有warning，沒有新增ignore。 |
| 完整實際輸入 | 458 cases／222 wire／157 control stages／13 cursor observations；原順序、身份、scope與全部清理通過。 |
| 既有Blueprint | 69個編譯／載入零error、零warning，package SHA及mtime不變。 |
| 聲音 | 五份真submix PCM：routed mix、master mute、ambient restore、voice及voice mute通過；不宣稱實體喇叭聽感或AVI音軌。 |
| Lua契約 | 當次127 cases成功；純artifact contract，不啟動對局。 |
| 來源／資產 | source-v6共3331份，驗收前後完整SHA、size、mtime一致，非只比count。 |
| DLL stage | bridge及base_content的read-only SHA仍與新機接受相同。 |

527份Rust／ABI與93份Lua authoring輸入的SHA、size、mtime與上一輪完全相同，明示沿用 `docs/acceptance/2026-10-19-new-machine/index.json` 的完整Rust／ABI／TD1–100及all-target接受，不宣稱fresh全套Rust。本輪公開map exporter有實跑一項。

## 原圖與錄影

兩張compiled公開map的出生點、lane、河道／野區、多人cast、fog／restore、HUD與shop有當次固定視點原圖；實際pointer開關shop。三個cast clip與公開HP delta走正式frame-consumption路徑。凍結相同world time，只隱藏文字或效果的實際視窗對照，兩map分別驗到371／534個文字像素及3320／3460個VFX像素；控制圖不混入事件錄影。實際原圖與AVI抽出的原始JPEG已直接檢視。

沒有專屬hit／respawn clip的角色使用已配置generic pose加共享動畫cue，不宣稱新增專用素材。XP可從hero details讀取，公開objective在既有溝通目標面板，未知／未揭露資料照實保留；沒有偽造常駐XP進度比例、購買成功或隱藏敵人資訊。

公開世界錄影：144原始影格／12片段／14643932 bytes，RIFF、JPEG envelope、索引、SHA、逐幀時間及總數全部通過。來源明示synthetic compiled public map；8fps是封裝播放率，不是整場實時或效能數據。

完整操作錄影另有3556影格／13片段／112463546 bytes，實際經過590.189秒；來源為synthetic filtered-feed的真Slate視窗，與公開世界來源嚴格區分。兩種影片都不冒稱新權威對局／ACK。

## 勾選與剩餘條件

| 已完成父項 | 接受依據 |
|---|---|
| 1.2／1.3 | 當次固定世界畫面及實際controller／pawn／view target；原生／原BP真PIE、focus／hit-test與唯一owner回收，正式owner／重連回歸。 |
| 4.1 | 合法portrait／HP／資源／level／XP、QWER／DF、items／ward／gold、team／time／objective等可讀HUD與面板，含未知狀態與實際輸入。 |
| 6.2 | 七英雄、creep、tower／base的正式mesh／material與team／selection，合法native fallback診斷保留。 |
| 6.4 | 七英雄配置動畫及共享cue的move／attack／cast／hit／death／respawn／recall、pause／fog／pool regression。 |
| 6.6 | 兩公開map固定before／after PNG、多人cast與HP文字、fog／HUD／shop及真視窗事件取樣。 |

剩餘3.1／3.3／3.4／3.5／3.9／4.3／4.4／7.1／7.4包含權威ACK、真對局／LAN或固定效能證據。4.7／7.3仍缺實際100／150 OS DPI配對；目前只有一台150%螢幕，原生／原BP兩種window mode及HD／QHD真surface已驗，不把text scale或強制尺寸當100% OS DPI，不改系統設定。後續僅在具可用執行條件時驗剩餘範圍，10/10不重設。

## 拒絕證據與監控

E1335–E1357在 `unreal-moba-error-register.md`。v2的卡片命中／裁切、v3的timeline寫入與次級驗證、v4的共用期限失敗皆保留；沒有改寫失敗報告、補timeline缺列、截斷AVI或增加timeout來冒稱成功。最後v5才整體接受。

長步驟使用exact owned process identity，每3分鐘檢查程序及log／JSON／JSONL metadata變化，另核對CPU與公開快照是否前進。活躍錄影不額外讀逐幀內容；結束後嚴格全檔驗證。完整raw監控留在Saved，可攜目錄只保留衍生檢查摘要與完成證據。
