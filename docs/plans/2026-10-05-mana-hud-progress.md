# 通用 Mana HUD 資料鏈路

## 決定與實作

- 本人 HUD 從既有 disclosed Hero 的 `mana_pool` 讀取，不查權威全世界或別人的法力。`None` 表示未知／未支援；啟用且餘額零仍是合法法力池。
- Proto `HeroHudPresentation` 追加 `mana_raw`／`max_mana_raw`（11／12），保留 Q10 到本機呈現邊界。使用既有 build.rs 正式生成並同步 tracked fallback，未手改生成 Rust。
- HUD schema 升為 2；橋接可讀 schema 1 的未支援法力資料，但拒絕 schema 1 啟用法力、非零 disabled payload、負值、超容量、超過 1000000 容量與未知 schema。沿用 owner team／player／render identity／epoch 檢查。
- `OmMobaHud` 追加兩個 float 呈現值，C ABI 12 同步 Rust 與 cbindgen header；版本閘門保留，不能將新 DLL 配上舊 native module 假裝可用。
- Unreal 共用 `FOmHeroHudStatePayload` 接線至 controller／native command bar。顯示 MP 數字與藍色法力條，非法／未支援／死亡顯示 `MP --` 並隱藏條；零容量不除零。既有 legacy HUD 不會被當成新 Mana。
- 共用 `HasValidMana`／`BuildManaText`，新增 `Om.Runtime.NativeManaHud` 精準測試並加入既有 Lua smoke 的選取清單；不需要英雄 C++ 分支或 Blueprint graph。
- 一般 server／LAN 仍未啟用 Mana；規則協商、裝備／Buff 與 explicit script 資源操作仍待後續，完整框架仍 20/30，不勾選 6.2。

## 必要確認

- client-runtime 新 `mana_hud` 2/2 passed：本人與隊伍隔離、None 與零池區分、Q10 值、移除本人後清除、schema 與非法範圍。
- bridge `hud` 3 passed／1 opt-in capture ignored：含 protobuf 欄位轉 C ABI、空 frame 清除、live owned identity／schema／超上限拒絕。ignored 不算成功。
- 新增測試前既有 moba_hud 8/8 通過；不將此數字冒充新增 Mana 測試。
- Rust bridge／cbindgen／DLL stage 已成功，Unreal UHT 處理新欄位成功；**native C++ 編譯尚未成功、NativeManaHud 尚未執行**。
- root 與 omfue whitespace diff 檢查通過。沒有 PIE、完整對局、真人操作、LAN 或全套驗收。

## 真正建置阻礙

`tools/lua/lua.exe scripts/build_ue_moba.lua --build-only` 被共享引擎基線擋住。UBT 報 `FailedDueToEngineChange`：Engine unity makefile 已變更，要求覆寫 Engine DLL／lib 與各 plugin modules 等既有引擎產物。

只讀 Git 確認 `D:/UE5.8/Engine/Source/Runtime/Engine` 有既有修改：`Private/Components/SkeletalMeshComponent.cpp`、`Private/SkeletalRender.cpp`、`Private/SkeletalRenderGPUSkin.cpp`。本批沒有修改／還原它們，也沒有移除 `-NoEngineChanges` 安全閘門。需先取得可用的引擎建置基線，才可完整編譯／載入 ABI 12 的 native module；不能把 UHT 或 Rust 成功算成 C++ 成功。

建置前 Editor 71024 執行中；MCP 確認 dirty content／map packages 均空。restart stop 等待逾時後回報 `force_terminated=true`，不是正常退出。本批會恢復 Editor 開啟狀態，但舊 native module 與新 bridge 的 ABI 不符時必須保持拒絕啟動 runtime，不繞過檢查。所有操作錯誤另見 E177。

收尾已重新開啟同專案 Editor 91188，`wait-mcp` 確認 HTTP 30000 ready。這只證明 Editor／MCP 恢復，不代表 ABI12 native module 已編譯或 Mana UI 已載入。兩個 repo 最後 whitespace diff 檢查通過。
