# Bot 通用持續等待（2026-10-05）

## 功能缺口與決定

推塔已用正式Hold，但基地Recovery::Hold／Support近距離Escort仍以MoveTo自己或空命令代表等待。Move抵達會advance，空命令不抑制autoattack；不得保留三種不一致的等待語意。

- 共用 `hold_input` 只讀本人HeroCommandQueue，回傳正式HoldPosition queued=false；active=Hold且queued空才不重送。若Hold後有舊排隊命令，送立即Hold正常取代、清queue，不直接改ECS。
- 基地HP／Mana恢復、無援軍推塔與近距離Escort共用同一adapter。不修改Sustain門檻／視野／角色／英雄數值／Lua配方。
- `recovery_move`只負責真正Move與同目的地去重；刪除自身位置假Stop、200-unit在途Stop workaround。Support仍用公開200跟隨半徑判斷是否等待，這是策略半徑而非假停止去重。
- 恢復到離開門檻後正常角色命令取代Hold；Escort只從新committed隊友位置恢復Move。既有技能／戰鬥／回城／shop優先順序不改，Recall channel不被Hold打断。
- Hold沿原正式命令與safe MovementPriority replay，不新增C++／BP／ABI／IPC或內容hash。抑制autoattack不等於免傷或基地安全保證。

## 當前功能確認

- core `--lib role_bot_persistent_hold_adapter`：新1 passed，None／空queue送Hold、既有Hold去重、stale queued命令要求replacement、撤退Move與同目的地去重。
- base_content `--lib sustain`：既有3項更新後passed，其中HP／Mana正式60Hz回城、基地首個Hold／不重送、暫停、精確回魔與離開門檻維持成功，另基地回魔opt-in／己方home／上限規則通過。
- base_content `--lib role_bots_support_escorts`：既有1項更新後passed，真人Carry跟隨、stale pursuit→正式Hold、後續tick位置保持與不重送、authority-only ally pose不改決策、新committed pose才恢復Move。
- 主repo whitespace check成功；沒有full suite／100場／Unreal／LAN／DLLstage。20/30與5.5維持未完成，完整驗收集中最後。

## 工具問題

首次apply_patch預期Escort尾段不完整而verification failed，沒有套用變更；重新讀短範圍完整區段後用精確context成功，不手動覆寫檔案。core／base命令與另一support測試共享scripts target，產生正常artifact lock等待；未刪lock／kill程序，待建置釋放後完成。後續同workspace編譯應串行，避免無效並行等待。這批沒有Rust編譯或測試失敗；既有template warnings保留。

測試位置界線：正式Hold提交前可能已有本tick物理移動，不要求回溯到上一個committed pose；先捕捉Hold接受後位置，再驗下一tick保持，不調換Dispatcher／Moves順序。
