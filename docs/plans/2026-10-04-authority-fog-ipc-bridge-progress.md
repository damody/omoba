# 正式迷霧 typed IPC 與 bridge ABI

## 本批計畫

- [x] 新增 FogGridPresentation schema 1：team／view epoch／sample tick／Q10 origin／cell size／columns／rows／row-major cells。
- [x] runtime 從已驗證並保留的 FG01 public presentation event 輸出正式網格；不重算 entity 視野或把 demo tiles 當正式結果。
- [x] core 共用轉換器先驗 audience／schema／tick／checked geometry／最多 4096 格與合法三態，再複製。
- [x] bridge 接收 typed grid，拒絕錯隊／epoch／未來／非法狀態；正式欄位存在時禁用 legacy tiles／circles／occluders，即使 grid 本身非法也不回退。
- [x] C ABI 10→11，frame 附 optional OmFogGrid，Q10 座標保持原值，null 表示 unavailable。FrameSlot 自有 metadata／cells，指標僅在 lease 有效。
- [x] 更新 cbindgen export 與 C++ header smoke；修正原 smoke 固定 ABI 7 的過期斷言。
- [x] 本功能指定 Rust 成功確認。
- [x] 產生／staging ABI 11 header 與 DLL。
- [ ] 完整 Unreal 建置確認：外部專案的 shared engine DLL lock 阻擋。

## 決定與邊界

舊 FogTilePresentation 只有列／行／visible，沒有 geometry 或 explored；保留給 legacy，新增獨立欄位而非改解讀。IPC envelope version 3 保留，protobuf 欄位 19 是可選增量；C ABI layout 改變則必須升到 11，拒絕舊 layout。

網格不包含 canonical source／隱藏單位或可攻擊身分。render 租約不持有 core authority view。absence／非法資料不代表全可見，Unreal 小地圖仍顯示 VISION N/A，尚未加入正式三態繪製；本批不宣稱完整視野、rebase 或 UI 完成。

runtime 投影也以正式 fog event 的存在鎖定 formal session，避免 bootstrap 尚無 phase HUD 時套用 demo。錯 epoch 網格不輸出 typed grid，但仍不回退 demo。

## 當前確認

- core `authority_fog_typed_presentation_validation`：1 passed。
- runtime `authority_fog_ipc_is_typed_bound_and_never_demo`：1 passed，包含真正 protobuf envelope encode/decode。
- bridge `authority_fog_`：2 passed，包含 typed audience／legacy 隔離、busy ring 重試、frame metadata／cell pointer、舊租約保留與完整 reset null。
- 只有直接相关測試，沒有跑全套 core/runtime/bridge 或 MCP／PIE／雙玩家。

`build_ue_moba.lua --build-only` 正常完成 base_content、codegen、bridge 與 ABI11 header／DLL staging。Unreal 的 OmEditor／OmGenerated／OmRuntime 三個模組編譯及 link 均通過；整體 target 失敗在 shared engine UnrealEditor-NetCore.dll 的 LNK1104／UBA9001。

查核 PID68548 的 UnrealEditor-Cmd.exe command line，是 C:/portable/OpenKoikatsu 的 PaintSpray396 GalleryAuditR2，與 omfue 無關；不停止、刪檔或調整其專案。這是外部共同引擎檔案鎖，不能標為完整 Unreal build 成功；不重複等待／重建整個 target。後續使用既有 build_bridge.bat 收尾當前 bridge，另確認 built／staged SHA一致；不另加臨時 fallback 或改 engine link target。

錯誤與操作診斷記於 E140／E141。OpenSpec 20/30 不變；三態 UE 繪製與完整 rebase 仍待實作。

收尾 `build_bridge.bat` exit0；生成 header 明確 OM_ABI_VERSION=11／OmFogGrid／frame.fog_grid。`build_ue_moba.lua --verify-staged-only` exit0，built/staged bridge SHA-256 同為 `68a32d8ec314ee1b199d9bae3b78e4417788a1ee08a3d09e57d21b309485a497`。本批修改檔 diff check 通過；未啟動 Editor／PIE，不在不完整 target 建置後宣稱畫面驗收。
