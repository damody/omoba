# 通用技能投影 ABI 收斂（2026-10-06）

## 計畫與決定

接續2.2b：盤點角色相容資料，保留有資產風險的reflected名稱，不猜測刪除Blueprint引用。

盤點發現共用Buff lifecycle及正式範圍技能已使用同一技能投影，但C ABI仍叫OmSaikaAbilityProjection／event.saika。這不是缺少另一個英雄執行器，不新增重複管線。

## 實作

- 共用C ABI名稱改OmAbilityProjection／OmAbilityEvent.projection。
- Rust buff binding／正式area writer、FrameBuild與租約測試同步；Unreal DispatchAbilityEvents全部從projection讀取通用payload。
- cbindgen export更新並真正重新生成om_bridge.h，不手改生成header。
- C ABI升級16，以既有strict config／input／載入gate拒絕舊版本。型別欄位及順序不變，但不靠舊DLL的layout巧合放寬版本。
- C++ header smoke改通用型別／欄位及ABI16，修正原ABI11過期斷言。
- 不改caster-only／area flags、等級／事件身分、來源披露、world-unit換算、Buff任意payload規則或Blueprint事件派發。

## 當前確認

- `cargo test --manifest-path omfue/bridge/Cargo.toml --lib projection -- --nocapture`：11/11。
- 新header／ABI gate測試：1/1，檢查通用export、缺舊型別與舊欄位，config拒絕0..15。
- 既有正式area→frame精確world-unit投影測試：1/1，確認半徑200／時間1與caster-only／area flags。
- cbindgen成功生成header；既有非pub常數略過警告保持。
- OmGameEditor限定OmRuntime／OmGenerated／OmEditor、保留NoEngineChanges：10actions、Result Succeeded、exit0。
- 日誌：`D:/code/omoba/omfue/Saved/Logs/generic-ability-projection-abi.log`。
- strict OpenSpec與兩個repo whitespace檢查通過。

## 版本與剩餘

C ABI16；selective wire6／IPC5保持。identity `ff3ef5e2957aa89f`、data `2df5b6b1e02d95d7`、presentation `3634f807282b0515`未改。

未執行header smoke獨立編譯、Editor／PIE／native automation或release統一建置部署；不能稱舊stage可開局，最後必須一致重建，不繞過版本gate。僅維護omfue，不修改omfx。

SaikaSummary仍是歷史manifest輸出；其他typed payload與saved資產引用仍待遷移。本輪通用C ABI收斂不代表完整2.2b無回退驗收已完成，維持21/31。

防錯紀錄：E282。
