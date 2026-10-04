# Blueprint 編譯與 MCP 建立驗證

## 計畫與決策

1. 使用 OpenSpec build-unreal-rust-moba-framework 的 3.2；起始18/30。
2. 原生Slate維持MOBA UI預設，不新增角色graph或不必要UMG。保留既有Blueprint／UMG相容資產，必要新英雄Blueprint由生成recipe提供parent／package，透過MCP建立。
3. compile結果必須是該asset、exit0、isError非true、ok=true、明確error_count0、無errors／health_issues／action_required；失敗保存raw compiler與pending／diagnostics，禁止把HTTP或graph build成功當作compile成功。
4. 只對新隔離的OmAutomation測試namespace注入錯誤；不破壞既有英雄或使用者graph。同一fixture移除錯誤節點並重新編譯／保存，保留乾淨測試資產與證據，不做刪除。
5. `ue_validate_blueprints.lua --create-missing`只對generated recipe允許建立；existing parent不符拒絕、不改parent或graph，不接管手寫graph。native-only空blueprint_path略過，絕不為了測試新增空英雄Blueprint。
6. full建置在MCP ready後執行此gate；無Editor的build-only不冒充Editor驗證。current相容UMG另以明確package驗證，不把測試空widget稱作完整遊戲UI。

## 程式與測試

- scripts/ue_blueprint_validation.lua：package/object/class path正規化、fail-closed compiler verdict、獨立raw response拒絕stale檔。
- scripts/ue_validate_blueprints.lua：預設recipe雙英雄、必要資產建立／parent核對／compile／只保存新建乾淨asset；所有失敗保留當前asset的pending診斷。
- scripts/build_ue_moba.lua：full路徑加Editor compile gate。
- scripts/tests/ue_blueprint_validation_test.lua：17個正負向場景。
- scripts/tests/ue_blueprint_editor_test.lua：真實MCP建Actor BP／UserWidget WBP／generated native hero parent BP；故意錯誤、CLI非零／pending、修復、save与repeat compile。
- scripts/tests/ue_blueprint_editor_acceptance.lua：獨立重讀raw MCP結果，驗證三個create／兩次三asset save／一次owned節點修復與8次compile狀態。

## 已測結果

- 第一輪完整建置27337 exit0、Editor57960。Blueprint fixture1791069017通過，隔離資產namespace BlueprintValidation_1791069017，三種asset皆建立／保存；invalid Actor→Character.Jump明確error_count1、self不是Character，正式CLI失敗與pending保留。修復後error_count0；重複compile Actor／Widget／generated parent均成功。保存verifier獨立exit0，compact evidence在change/evidence/blueprint-validation/blueprint-1791069017.json。
- 既有雙英雄recipe驗證compile-1791068856／1791068977通過；九個RustBP/UI相容UMG compile-1791069027通過，無角色graph替換。
- 同Editor兩輪13/13通過；asset save後固定Lua WM_CLOSE／wait確認57960正常退出，沒有重犯QUIT未退出就啟動下一階段。
- 最後整合full建置18805 exit0，UBT1.83秒、Editor10844，MCP ready後真實呼叫新的--create-missing gate（當時雙英雄）。再擴充預設gate包含現有九個WBP，direct相同入口compile-1791069249真實11個資產全部error_count0、created=false；完整保存verifier重新核對recipe／UI檔案集合與每個raw compile，無漏項。
- 最終Editor10844同session兩輪各13/13（79371），串行PIE87395 exit0／正常停止；這是既有Editor／PIE相容回歸，不是新的60Hz完整網路／GPU效能驗收。runtime／C++玩法未變，bridge stage保持cc76ef2d8328126a7ca255de84fa49daf72272af426ba9a0b40f8ceb8871d709。
- 3.2已完整勾選，整體19/30、剩餘11项。測試asset與原始失敗證據保留供排查，沒有留下故意無法編譯的graph。

## 重跑

固定Lua入口：tools/lua/lua.exe scripts/ue_validate_blueprints.lua --create-missing（只允許generated recipe建立）。明確package只驗證，不能搭--create-missing接管外部資產。

Editor負向／恢復：tools/lua/lua.exe scripts/tests/ue_blueprint_editor_test.lua。

保存驗收：tools/lua/lua.exe scripts/tests/ue_blueprint_editor_acceptance.lua target/blueprint-validation-runs/blueprint-1791069017 target/blueprint-validation-runs/compile-1791069249/report.json。

## 錯誤與限制

E104記錄真正負向編譯錯誤、native-only空字串的Lua truthy誤判與修正。現有生成資料沒有MOBA UMG配方，因此不強行導入新UMG graph。這一項只驗必要資產工具能力与診斷，不代表選角／計分板／三路／LAN完成；不影響先前60Hz驗收，也不宣稱新的效能結果。
