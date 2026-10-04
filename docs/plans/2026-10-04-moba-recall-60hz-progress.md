# 權威回城核心／60Hz 增量

## 計畫與決定

1. 新增正式PlayerInput Recall（protobuf tag19），不借用技能／物品ID。先完成權威核心與安全投影；未協商channel capability前server canonical acceptance及MobaMatch secure admission明確拒絕Recall，UE B鍵／IPC尚未開放，不把headless成功算網路回城。
2. Lua moba_economy.recall_channel_seconds=8生成Rust常數。1..60整數驗證、完整canonical rules hash、compiled agreement與禁止單端hot reload；TD-only省略整個section時保持合法相容預設，不新增TD玩法。
3. 權威開始清HeroCommandQueue／MoveTarget、停自動攻擊，讀條使用累積Fixed64遊戲時間。重送不重設deadline，pause不計時。移動／攻擊／施法／物品使用意圖、實際位置改變、正傷害／ScriptDirectDamage、死亡或基地消失取消；同batch行動命令優先，不依次序藏命令。
4. post-step在伤害／死亡後完成，確保存活且原地、自己的基地仍在才傳送。最後當tick受傷也不能傳送。回城不免費回血／發錢；不新增尚無規則的泉水系統。
5. 新Recall channel保存於hero slot並進replay digest，重生清除；停自動攻擊以既有安全MovementPriority投影，最後位置走既有committed pose，不讓filtered client持有完整MobaMatch。

## 驗證

- base_content全79 passed（session54258），含60Hz完整filtered對局與既有15／120Hz相容回歸，不是120Hz效能驗收。
- 新回城測試：60Hz準確8秒、pause120ticks、重送不延長、固定種子重播、無免費回血或Gold；非法player、secure gate、移動、位置變動、完成當tick正傷害／死亡、基地消失、攻擊命令與同batch競爭、ScriptDirectDamage中斷。
- 擊殺／重生後加入Recall，525ticks／1050雙隊apply，owner-only Gold、整段讀條與基地傳送canonical team hash一致、零ComponentRepair。
- template runtime-lua-content共62 passed（30unit＋23generated＋8hero＋1catalog）；新recall rules hash／compiled mismatch／dev reload拒絕，舊TD fixtures恢復通過。
- core --lib323 passed，最後傷害hook補強後session81187再跑323 passed；server --lib152 passed／1 opt-in ignored，runtime52＋3 passed／3 opt-in ignored。
- protobuf fallback由內建vendored protoc與OMOBA_UPDATE_PROTO_FALLBACK=1重新生成，包含目前工作樹既有proto變更，沒有手改generated bindings或還原使用者proto。

- 最後build-only85353 exit0，bridge／OmGame成功；stage SHA256 `c82bbf3ac5712209656778e6549928ec8d88020d28c9f6b73605567b996d8a5f`一致，codegen --check11檔、OpenSpec strict、scoped diff --check通過（既有CRLF提示）。本輪未開Editor／PIE，不能以build-only冒充畫面驗收。
- 真實新版內容60Hz KCP run1791071189／session95622 exit0，success與cleanup_verified=true，兩隊safe_tick1096、各9 checkpoints至1080、實際移動。server8256／runtime41704、89964已退出，CIM核對無殘留。raw在target/interactive-runs/moba-runtime-1791071189；這是新schema／規則的移動與hash回歸，不是網路回城（gate仍關閉）。
- Fyrox cargo check --tests最後通過；新增InputActionKind::Recall只為舊前端shared input分類完整，不新增回城按鍵或繞過secure gate。

## 問題／限制

錯誤集中E106：Vec2 constructor／enum struct-update編譯失敗、TD省略section default0造成10個fixture拒絕、server新增oneof漏完整match；皆修正，不問使用者重複確認。

codegen輸出的content_hash de9c7fcfc98d6479是生成class surface hash，回城不改角色surface所以不變；正式規則相容使用完整canonical catalog_data_hash，不能混為同一hash。未改角色C++／Blueprint graph。

5.3保持未勾選：助攻、回城網路capability／IPC／UE B鍵與讀條HUD仍待接上；本輪是權威核心，不宣稱真人操作／UE回城或60FPS frame-time完成。下一步沿正式安全輸入接回城，再做助攻與多英雄規則。
