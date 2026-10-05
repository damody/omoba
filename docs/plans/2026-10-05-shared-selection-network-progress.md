# 共用選角 host／gateway 與原生房間更新

## 本輪計畫與決定

1. 在既有 moba-config 增加獨立選角host模式，使用共用 SelectionRoom，不建立CONFIG singleton／gameplay World、不載入script DLL或runtime Lua。
2. 由host產生每個真人的獨立隨機邀請token；handshake先核對schema1／scope／full catalog hash／player-token，才建立fixed-player service。不能自報其他player ID就控制席位，Bot不會取得邀請。
3. 原生widget仍使用相同 --selection-session FILE PLAYER_ID：普通plan走單真人服務，selection-room-link invitation走Rust TCP gateway。Unreal只處理原有pipe／JSON，不需要新增角色C++或socket Blueprint。
4. Gateway回覆加入shared_room=true；Unreal僅對共享模式每0.5秒發read，仍一次一個pending request，revision相同不重建列表。其他玩家完成後的stale回覆仍有權威finalized snapshot，追隨者退出但不寫假配方；最後成功finalizer維持原有結果檔，host另保存canonical finalized-plan.json。
5. Host worker上限10，handshake最多16 KiB／5秒、write10秒；每席位一次active，disconnect退役lease。沒有在room lock內讀socket；玩家思考不設read idle期限。完成配方發布且所有連線退出後host正常退出；取消／未ready不自動開局。
6. Artifact使用完整pending檔＋sync_all＋同目錄hard link原子發布，已存在目標不覆寫，不以讀取重試遮蔽部分JSON。這是完整檔案可見性，不宣稱host process crash後的端到端durability。

## 原生入口契約

- Host：`moba-config --selection-host PLAN.json NEW_OUTPUT_DIRECTORY`，預設localhost的動態port。
- 明確endpoint：`moba-config --selection-host PLAN.json BIND_IP:PORT NEW_OUTPUT_DIRECTORY`，拒絕unspecified／multicast；本輪僅實跑127.0.0.1，不開放外部監聽、不改防火牆。
- Host output：ready.json（不含token）、player-ID.json（敏感邀請）、finalized-plan.json（唯一host完成配方），及最多64筆error kind的errors.md。
- Gateway：`moba-config --selection-session player-ID.json ADMITTED_PLAYER_ID`。Unreal啟動時 -om-selection-plan 可指向該邀請檔；原單機plan方式維持。
- 上述為native API契約；一般完整啟動仍須由固定Lua工具負責build／stage／owned-process lifecycle，沒有新增根目錄bat或shell fallback。
- 邀請檔含bearer token，僅交給相應玩家／可信host，不應提交或公開上傳。TCP沒有加密、不能防LAN竊聽；對外正式服務仍需受保護transport／admission設計，不把本機capability当成完整帳號系統。

## 局部確認結果

- 真實child-process host＋兩個CLI gateway＋TCP，驗證初始flush、wrong-token不洩catalog、duplicate active seat拒絕、invitation不能改player、兩席位共用選角／鎖定、stale回覆帶最新狀態、明確finalize、追隨者final snapshot、EOF正常退出、host配方與成功回覆完全一致及host退役。
- 首兩次失敗與socket blocking修正記E244；修正後通過，最後原子發布／bounded MD版本再次1/1成功，測試0.28秒。這是當前功能修正確認，不是全套重複驗收。
- Framing／token與原子publication／不覆寫／bounded MD兩個unit tests成功；原單真人真正CLI process測試1/1成功，0.01秒。最終共4項當前相關測試成功。
- Rust正式compiled-content-only首次cargo check成功；最終test已重新編譯native CLI。沒有新增Rust依賴。
- Scoped OmRuntime／OmGenerated／OmEditor首輪5 actions／13.18秒成功，最後stale-finalized修正3 actions／4.98秒成功，日誌native-shared-selection-poll-modules-20261005.log、native-shared-selection-finalized-modules-20261005.log。
- E224仍限制整體Editor執行；原生poll／follower退出只已編譯，不宣稱雙Unreal畫面或完整LAN成功。未stage binary／部署／commit／push，不維護omfx。
- OpenSpec strict validation及主repo／omb／omfue whitespace check均成功。TCP admission的負向案例預期印出拒絕stderr並保存errors.md，不應誤讀為最終測試失敗；沒有孤兒owned child留在測試後。

## 尚未完成

- 本輪結束時Lua多人launcher尚待；後續已完成本機共用host與每席位renderer協調、收據／配方保護及owned清理，局部結果見 `2026-10-05-shared-selection-launcher-progress.md`。雙Unreal實際執行仍待，不回寫為本轮TCP驗收成果。
- 兩台LAN invitation交付／玩家選角到正式server與runtime接線、完整對局／效能验收仍待。完整6.2／6.4不勾選，21/31保持。
- protocol1／compiled Lua catalog hash／C ABI14／IPC4／selective wire5不變；shared_room為gateway可選標記，單真人回覆不加此欄位。
