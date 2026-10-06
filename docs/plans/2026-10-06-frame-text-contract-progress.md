# Frame 文字參照整批准入（2026-10-06）

## 問題與決策

文字讀取器局部越界只回空字串，frame仍更新／ACK；Ref.len及FX view.len直接轉int32可能縮窄。接續OpenSpec安全呈現，整批檢查在更新前完成，不依賴各讀取器fallback。

- OmFrameStringRefFits共用offset＋len寬整數邊界與Unreal converter MAX_int32容量；len0保持ABI absent-string語意，offset不用，不新增任意內容長度上限或NUL要求。
- OmValidateFrameContract涵蓋active buff／buff event payload、script五個文字欄位、ability ID／cue／payload與nested projection buff ID、route／shop／terrain及owner economy六槽display name。
- FX OmStringView長度不得超converter容量，非空要求非null storage；allocation／lifetime仍由lease提供，不宣稱pointer能證明實際配置長度。
- 正式ProcessFrame與FrameConsumption同源gate，任何非法文字不apply／ACK。Frame／catalog／FX讀取器也防禦縮窄，catalog仍保留本輪以外的獨立准入契約。
- 新FrameTextContract native矩陣逐欄位注入非法／合法refs、空值／尾端／overflow／超int32／FX storage與length，並確認消費helper不呼叫apply／ACK；加入既有Lua runner。

## 局部確認與限制

- UE限定9actions、Result: Succeeded、exit0；Saved/Logs/frame-text-contract.log。
- 固定Lua loadfile與兩repo whitespace通過；native矩陣僅編譯、尚未執行，未PIE／stage／完整驗收，不重試E285 Cmd。
- 無新編譯失敗；一次rg無命中是搜尋結果，不是建置失敗，大段輸出截斷不當作完整審閱。此輪驗文字範圍／表示容量，不宣稱UTF-8字元合法性驗證。
- 版本／生成hash保持，整體21/31、完整6.4不勾。見E293。
