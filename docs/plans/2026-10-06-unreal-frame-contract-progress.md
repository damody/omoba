# Unreal frame 共用准入（2026-10-06）

## 計畫與問題

沿用 build-unreal-rust-moba-framework 的安全呈現工作。ProcessFrame原先只查null便修改actor／HUD／地圖／效果，Tick不論處理結果都ACK並推進sequence。非法陣列可能在早期呈現已變更後才被解參照。

## 實作與決定

- 新增共用OmFrameContract.h：先檢查完整struct_size及ABI16，才讀其餘欄位；presentation_snapshot只接受0／1。
- 全部21個header count／pointer配對（含string table）非空時必須有storage；optional fog_grid的正cell_count亦必須有cells。零count允许保留非null storage，optional HUD／economy仍可缺席，不任意限制10000 entity場景。
- 這是shape准入，不宣稱非null能證明allocation長度。bridge lease仍負責有效storage與生命週期，記錄語意由原有呈現模型檢查。
- ProcessFrame回傳bool，任何更新前拒絕；Tick只在成功後ACK／推進LastFrameSequence，無論拒絕與否釋放lease。Tick先檢查size／ABI，再使用sequence判斷是否新frame，避免錯ABI解讀欄位。
- 拒絕診斷最多每2秒一次；不重啟後端、不清空既有合法呈現，下個合法frame可繼續。synthetic入口走相同gate，沒有測試專用繞過。
- 全部原生automation fixture改用OmMakeTestFrame建立正式header，而非放寬production gate接受零header。
- 新增FrameContract原生矩陣，包含header／flag／每個配對／nested fog／零陣列／10000 storage，以及真實actor的晚期FX指標缺失不得先移除actor，恢復合法frame才可移除。加入既有Lua scoped runner清單。

## 局部確認與失敗紀錄

- UE限定OmRuntime＋OmGenerated＋OmEditor：8 actions，Result: Succeeded，exit 0；Saved/Logs/frame-contract-gate.log。
- 固定Lua執行runner loadfile語法確認通過；主repo及omfue whitespace check通過。
- 嘗試UnrealEditor-Cmd.exe只執行Om.Generated.FrameContract（unattended／NullRHI，非PIE），程序立即exit 1，仅印bundled DotNet SDK，指定frame-contract-native-test.log未建立。其後Get-Content亦因不存在失敗。尚無測試斷言執行證據，不能把編譯當通過；沒有重試、終止未知程序或繞過版本gate，原因未證實。
- 未啟動對局、未stage、未全套驗收，ABI／wire／IPC與生成hash不變；完整6.4仍未完成，維持21/31。見E285。
