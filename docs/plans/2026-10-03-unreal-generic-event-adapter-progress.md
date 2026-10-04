# 英雄事件通用模板／相容 adapter 隔離

## 計畫與決定

1. 將 Saika typed payload、metadata、事件轉換與舊介面宣告移到 `omfue/codegen/src/legacy_hero_compat.rs`。通用模板只呼叫 adapter hook，不直接判斷英雄 ID 或技能 ID。
2. 保留現有 reflected 名稱、manifest `saika` 欄位與生成 bytes，避免破壞已保存的 Blueprint graph 與 bridge metadata。這是相容隔離，不冒充已刪除 legacy adapter。
3. 新英雄繼續使用共用 `FOm*` payload／OnAbilityCue、OnAnimationState、OnAttackPhase；不得在相容模組新增新英雄行為。Rust／Lua 與美術仍是新英雄製作入口。
4. 加入角色判斷不得回到通用模板的防回退測試，以及 exact ID／kind gating、一般英雄共用 Super hook、舊四技能入口保留的測試。
5. 先 codegen tests／只讀 --check，再完整建置、native 兩輪與 PIE。生成輸出保持不變，故本輪不更動 C ABI／protocol、Unreal 資產、橋接行為或重跑無關 gameplay 實作。

## 驗證

- 第一段搬移後既有 codegen 22 tests passed；--check 驗證 11 files／13 inputs、content_hash=de9c7fcfc98d6479，沒有改寫生成檔。
- 新增後 codegen 24 tests passed：新英雄／Date／名稱近似 Saika 的英雄均沒有 legacy declarations／dispatch，但保留通用 Super hooks；legacy 四技能 handler 保持，錯 kind 不派發；通用模板 source 防回退測試通過。
- formatting 後 --check 仍通過，11 份輸出 bytes 未改；full build session 31311 exit 0，Editor PID 68560／MCP HTTP 30000，OmGameEditor incremental 成功。
- bridge 因依賴 codegen 重新建置，built／staged SHA-256 一致：`c5736f6bb22e92cef41c4edd53eba413305bcffbad8c7817db12f2c3ce9a6863`。生成內容 unchanged 不代表 DLL bytes unchanged；本輪驗證的是新的 DLL。
- native session 10981 同一 Editor 兩輪各 7/7，failed／skipped／not_run=0；PIE session 75028 exit 0、native_mesh_rendered=true、remembered_ghost_rendered=true、counts=[1,0]，已 stop。既有 transient world DestroyActor warning、預編譯 plugin dependency warning 仍存在，不宣稱零警告。
- Git whitespace 檢查通過。本輪未改 gameplay／IPC／ABI，未將前輪 dual run 冒充本輪重跑結果。

## MCP 的實際資產證據

- 先 `--list` 讀 MCP discovery tools，再 `search_tools` 取得 `get_blueprint_skeleton`／`get_graph_intent` 的正確 action 與參數，不猜 API。
- `/Game/RustBP/Heroes/BP_SaikaMagoichi` 的 parent 是 OmHeroSaikaMagoichi，EventGraph 有 31 nodes。legacy `HandleSaikaActionEvent` 的 Payload 型別是 `struct:SaikaActionEvent`；另有通用 OnAnimationState／OnAttackPhase。
- graph intent：legacy action event→取得 SaikaMesh AnimInstance→cast ABP_SaikaMagoichi→ApplySaikaAnimSnapshot；通用動畫／攻擊 handler 也有獨立的動畫更新與 debug tracer。不能只因存在通用 handler 就假定 legacy graph 已無人使用，也不能任意刪掉既有接線。
- 原始只讀報告：`omfue/Saved/McpAutomation/legacy-saika-blueprint-skeleton.json`、`legacy-saika-graph-intent.json`；含實際 node aliases／IDs，未 compile/save/patch 使用者 Blueprint。本輪 legacy 根節點為 evtSaikaAction，castAnim、applySnapshot；保留現狀，待有備份與全引用盤點的精確遷移。

建置重遇 E012：旧 Editor 36276 graceful child shutdown 不成功，既有 restart 等待 10 秒後僅 force 終止已驗明該專案 PID／children，確認退出才 stage。沒有停止其它專案、刪除資產或修改共享引擎來源。

## 剩餘範圍

OpenSpec 仍 15/30，2.2b 不勾選。真正移除 legacy 介面需先確認已保存 Blueprint 使用情況，再將測試與資產 graph 轉為通用 payload，並遷移 bridge 的舊 typed projection；本輪不刪除任何資產或改 ABI layout。相容模組是過渡層，不是新增英雄的擴充點。
