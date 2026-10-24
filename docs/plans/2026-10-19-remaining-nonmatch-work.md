最新結果：本輪不需新對局的六個父項已完整本機接受，目前33/44、11待驗。詳 `2026-10-19-nonmatch-final-acceptance.md` 與 `docs/acceptance/2026-10-19-nonmatch/index.json`。完整v5成功，v2／v3／v4失敗不覆寫；以下為實作過程紀錄，先前的「正在跑」不是目前狀態。

# 剩餘項目的非新增對局實作

接續 complete-moba-controls-and-presentation 的 17 項。權威對局額度仍為 10/10，不重設、不啟動新對局。先核對 controller／HUD／世界呈現／動畫的共用實作，針對性驗證後集中完整本機驗收。只有父項全數滿足才勾選；wire admission 不冒稱 ACK。

## 動畫快照時序

共用 AOmAnimatedUnitActor 原先只檢查 identity，較舊動畫快照可覆寫較新的 attack／recall／pause。改為拒絕負 tick 與小於目前 snapshot 的 tick；相同 tick 的狀態細化保留。來源 epoch 或 pool identity 重設沿既有生命週期清空 watermark，不跨 timeline 卡死。

新增七位正式英雄的實際骨架動畫回歸，涵蓋亂序 idle／recall／pause、相同 tick、hidden 邊界、來源 epoch 與 pool identity。

七位正式英雄的動畫時序已在 NonmatchFocusedV1 與 NonmatchWorldV7 真實執行成功；此筆實作不自行勾完 6.4。

## PIE、公開世界與 nameplate

新增四次真實 standalone PIE：原生 controller stop/restart、未修改的原 BP controller stop/restart；每次均檢查實際 class、唯一 action owner、RTS pawn、view target 及停止後退休。暫存 map 在啟動前放入原 BP_WorldBridge，port 0 的 PresentationIpc 不啟動權威對局。唯一未佔用 profile 目錄、runtime 設定與命令列皆回收；原資產不儲存。

NonmatchWorldV7 共九項零 warning 成功：PIE、七英雄時序、nameplate 無 tick 退休、NativeCharacterAssembly、PlayableMvpHud、PublicUnitSelection、PublicUnitTeamMarkers、ResponsiveHudLayout、WorldBridgePresentationScale。兩張 compiled 公開 map 有 spawn／lane／biomes／team relations／shop 真 pointer 事件／fog／restore／Full HD／QHD 的實際視窗 PNG。

迷霧原圖揭露舊 actor 的六處灰色 nameplate 背景。共用 AOmUnitActor 同步處理 component 與 widget root 的 visibility，hide/reset 在停止 tick 時也立即 collapse；合法 reveal／新 occupant 恢復同一 widget。恢復僅在 hidden→visible 邊界計算，不增加重複 visible 呼叫的格式化成本。v5 原失敗 fog 圖與 v7 修正圖都保留，不以降低正常亮度門檻掩蓋；fog 用同視點 RGB 遮暗比較，restore 仍須通過原正常門檻。

公開世界視窗新增可選 MJPEG AVI：每次實際 paint 取原視窗像素，尺寸變更分段、有界總 bytes／frames、逐幀單調時間與相機位置。來源明示為 synthetic compiled public map；既有 filtered-feed 來源仍嚴格區分。v7 兩 map 共 136 幀／12 段／13,834,204 bytes，全部 RIFF／JPEG envelope／索引／SHA／timeline 通過，來源誤標與未知來源負例拒絕。每 map 實際約四秒，8 fps 是封裝播放率，非整場實時錄影、效能或權威 ACK。

## 完整驗收執行中

ordinary nonmatch-build-v15 成功（UBT 6.21 秒）；read-only stage 兩個 SHA 與上一輪接受相同。凍結 nonmatch-source-guard-v2 共 3329 份來源／資產。與上一輪 final-source-guard-v5 比對，527 份 Rust／ABI 輸入與 93 份 Lua authoring 的 SHA、size、mtime 完全相同，明示沿用上一輪完整 Rust／ABI 接受，不冒稱重新跑完整 Rust；當次公開 map exporter 一項有實跑。

nonmatch-final-v1 正在跑正式 native／五 PCM／完整 owned viewport 與可選錄影／69 Blueprint。程序存活時不得修改以上來源或資產，只能編輯文件或 Saved scratch orchestration。

待此 consumer 全部退出後處理正式清單的命名：目前完整 inventory 只自動收 Om.Runtime／Om.Generated 等允許名稱，新 Om.Editor.PieActionOwnerMatrix 已在九項專項真跑成功，但被正式 inventory 略過。須改為對應 Runtime feature 名稱並在正式清單核對，不把目前 200 項誤稱含該 PIE。

後續核對 17 項父項的證據界線；新權威 ACK／LAN／OS DPI 與整場視覺不由上述 fixture 代替，10/10 不重設。錯誤詳 E1335–E1343，失敗報告與錄影不覆寫。
