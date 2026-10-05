# 權威施法等級增量

## 計畫與決策

1. 技能 cue 使用成功施法當下的技能等級，不從 renderer HUD、目前 snapshot 或輸入 ACK 推測。
2. 共用 dispatcher 在 handler 執行前從 gameplay 使用的 serial Hero ledger／cache 捕捉等級，通過既有成功 gate 後才寫入 visual 與 Ability fact；拒絕施法仍不發布。沒有可用等級的 legacy hook 明確保留 0（未知）。
3. 沿用目前可見且具 Disclosed replica mapping 的施法者 gate。成功施法等級作為這一次公開技能效果的 metadata；不額外公開隱藏施法者、目標、座標或 toggle 狀態。
4. 不改 C ABI 13 layout：bridge 填入既有 AbilityCast.level，Unreal 既有通用 consumer 使用 Event.level。rank 不超過 i32::MAX，避免 native int32 截斷。

## 實作格式

- public Ability payload：舊 8 bytes stable ability ID 表示未知等級；新 ABS2 為 magic 4＋stable ID 8＋rank u32 4，共 16 bytes。
- presentation cue：ABY1 36 bytes 保留讀寫；已知等級用 ABY2 40 bytes，前四個 u64 欄位不變，尾端增加 rank u32。
- 每個版本嚴格比對 magic／長度。新版本拒絕 rank=0、超過 i32::MAX、非法身分；舊版本不自行補成 1。
- rank 不改 effect identity、公開序號、live epoch dependency、1024 共用容量、ACK／重連基線與一次性 cue 行為。
- 新舊格式是可辨識的選用呈現 payload；未知版本 fail closed，不當作 gameplay 支援。IPC 4／selective KCP 2／command 1／C ABI 13 不變；部署仍必須更新對應 server／client／bridge binaries，舊 consumer 不能呈現新 rank payload。

## 本批局部確認

指定指令（不是完整 suites）：

```text
cargo test --manifest-path scripts/Cargo.toml -p base_content --features compiled-content-only successful_cast_facts -- --nocapture
cargo test --manifest-path omoba-core/Cargo.toml ability_cue -- --nocapture
cargo test --manifest-path omoba-client-runtime/Cargo.toml --features compiled-content-only cue -- --nocapture
cargo test --manifest-path omfue/bridge/Cargo.toml ability_cue -- --nocapture
```

- base_content compiled-content-only successful_cast_facts：3/3。使用正常 generated handler／正式輸入／60Hz driver 與顯式共用 projector；把實際技能設成 3，施法後改 Hero 為 4，原成功事件與 cue 仍是 3。不是自動 server team frame／KCP 整條鏈。
- core ability_cue：6/6。新增 ABS2／ABY2 嚴格格式、舊版未知語義、rank 範圍與 shared PresentationCue decode；既有安全 gate／序號確認仍通過。
- client compiled-content-only cue：7/7。安全投影 fixture 保留 rank=3；實際 localhost IPC 保留／覆寫／ACK fixture 依序測 DMG1、ABY1、ABY2 三種格式。
- bridge ability_cue：3/3。compiled catalog 查表、busy lease／coalescing 的同一事件保留 level=3，沒有發明 target 或 payload；既有 admission 及非法 catalog 測試通過。
- 未跑完整 suites、未 stage DLL／EXE、未重新啟動 Unreal、未做完整對局驗收。本批為 Rust 來源與橋接資料修改，沒有變更 native C++ layout 或 source；不得把前一批 native synthetic 結果當作新版本部署驗收。
- 既有 build script dead_code 與 bridge dev helper warnings 未改；指定測試 exit code 均為 0。

## 未完成範圍

完整 OpenSpec 仍 21/31。6.1、4.3 等完整項目不勾選；實際 effect 目標座標、toggle 結果及通用動畫／地圖呈現仍需獨立權威契約，不能用輸入點或 renderer 現況代替。最後再統一部署與完整驗收。

工具與防錯記錄見 E222。
