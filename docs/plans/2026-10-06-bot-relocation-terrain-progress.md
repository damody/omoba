# Bot 位移意圖公開地形准入（2026-10-06）

## 問題與實作決定

接續OpenSpec 5.5。ApproachEnemyPoint原先只用披露敵人、距離與冷卻選最近目標；公開薄牆阻擋時，正式handler會拒絕，但Bot每次仍優先送同樣技能，不能交接下一項作者政策。

1. 決策增加唯讀relocation_clear provider，只套用ApproachEnemyPoint候選。排除公開地形阻擋者後，沿原focus／距離／canonical ID選下一個合法披露目標；全數失敗就continue下一項作者政策。
2. 正式provider沿共用hero_move_tick::path_hits_regions，使用本人CollisionRadius與公開BlockedRegions；缺半徑採script adapter相同的30，不能誤用普通導航的20預設。
3. 位移檢查是完整直線swept segment，不是可繞行導航；不讓技能繞牆、放寬碰撞或直接改Pos。定身／沉默／魔力／rank／CD等原gate保持。
4. 普通EnemyPoint範圍技能不调用位移檢查，不能將地形阻擋施法者旅行誤當所有技能都受遮擋。
5. 不查隱藏動態單位或建立失敗黑名單；每次用當前披露與公開地形判斷。最後仍由正式權威handler驗整份效果與資源，Bot不預扣魔力／CD。

## 當前功能確認

- `cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only dash_ -- --nocapture`：2項通過。新案例確認受阻近目標改選可用遠目標、全受阻接恢復、無其他政策返回None、範圍技能不調用relocation provider；既有披露／距離／rank／CD案例保持。
- `cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only dash_ -- --nocapture`：4項通過，包含1項handler preflight與3項正式60Hz案例。新薄牆fixture：先選恢復而非注定拒絕的位移，planning保持原Pos／魔力／CD，正式提交HP100→210；地形恢復後隱藏cache不選，current恢復才正式位移到披露350距離目標並啟CD。既有權威薄牆／射程拒絕及定身拒絕／解除保持。
- 沒有新編譯／測試失敗；既有td_rounds dead-code warnings未修改。大型歷史計畫只讀本輪相关區段，沒有宣稱已完整重新審核。
- OpenSpec strict與兩工作樹diff whitespace確認通過。

## 界線與剩餘工作

21/31保持，完整5.5與100場留最後。這是公開地形位移意圖，不保證落點戰術安全，不讀敵方私有Buff、命令或隱藏阻擋；不更改所有技能的遮擋規則。沒有Lua執行期、Unreal／Blueprint、ABI／wire／IPC／內容hash變更。最後仍須一致重建Rust權威／replica／bridge並做完整驗收；omfx不維護。

防錯紀錄：E278。
