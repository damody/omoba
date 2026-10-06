# Frame 公開幾何範圍共用准入（2026-10-06）

## 計畫與問題

接續OpenSpec安全呈現。route已有模型／renderer局部檢查，非法span被跳過後仍套用其餘frame並ACK；polygon先整批拒收，兩者不一致。

## 決定與實作

- 新OmFrameRangeFits以uint64加總start＋count後比capacity，避免u32溢位；route與polygon都使用同一入口。
- OmValidateFrameContract在任何呈現更新前查全部map route spans，非法frame拒收、不更新／ACK；不把跳過單一route當整批成功。
- 合法空range可在尾端；空range的start超capacity仍拒絕。不新增任意point cap，不改路徑最少點數／名稱／有限座標等模型語意，lease仍負責storage lifetime。
- 新FrameGeometryRanges native矩陣涵蓋空range／合法尾端／超尾／overflow，route与polygon共用拒收，以及真實actor在非法route frame下不被部分移除，後續合法frame可繼續。加入既有Lua scoped runner。

## 局部確認

- UE限定OmRuntime＋OmGenerated＋OmEditor：9actions、Result: Succeeded、exit0；Saved/Logs/frame-geometry-ranges.log。
- 固定Lua runner loadfile及兩repo whitespace通過；native矩陣僅編譯、尚未執行，不重試E285 Cmd。
- 無新編譯失敗，未PIE／stage／完整驗收。版本／hash保持，整體21/31、完整6.1／6.4不勾。見E292。
