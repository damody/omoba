## Context

現行 selective-lockstep 已具備 server authoritative world、Wave A commit、兩隊平行 visibility、team-scoped replica ID、filtered frame、Reveal/Hide/Forget、repair/rebase與KCP team routing。然而 server-local observer 以 `NoopDisclosedWorldStepper` 套用 frame，再依每 tick authoritative component repair 得到相同hash；它只能驗證protocol state，不能證明Specs gameplay、script、input或RNG deterministic parity。

本遊戲固定為仿英雄聯盟的兩隊模型。每場只存在 Team 1 與 Team 2；每隊必須有一條獨立thread和一份filtered Specs world。兩隊同時執行，模擬兩個真實client，完成順序不可成為gameplay或repair輸入。

既有約束包括Rust 1.95.0、Specs 0.20、host與script DLL必須使用相同rustc、secure match不得降級global protocol，以及完整測試統一在所有實作完成後執行。

## Goals / Non-Goals

**Goals:**

- Team 1與Team 2各自在獨立thread執行完整filtered Specs simulation。
- Authoritative server、server observer與omfx共用同一套deterministic gameplay phase runner。
- `TeamGameStart`把同一個`global_seed`交給兩隊replica；每tick以`global_seed + tick`重建RNG。
- Hidden entity不進入未授權team world；跨界結果使用sanitized external effect。
- 正常frame不再以每tickcomponent repair代替simulation。
- Hash在authority correction之前比較，server衝突時仍由server repair/rebase。
- Encoded team frame必須可靠進入outbound queue，不得靜默丟棄。
- Team 1與Team 2平行驗算；單隊失敗不停止另一隊worker。
- Tasks拆成5.6 Terra可獨立理解、修改與驗證的小項目。

**Non-Goals:**

- 支援第三隊或動態team數量。
- 把observer移到另一個process。
- 讓replica改寫authoritative world。
- 保密`global_seed`。
- 在active secure match降級legacy `TickBatch`、global snapshot或global hash。
- 讓remembered presentation參與simulation。

## Decisions

### 1. 固定兩個平行team worker

Match建立時同時啟動`team-replica-1`與`team-replica-2`。每條thread獨占一份Specs `World`、replica ID map、pending queues、RNG與script runtime state。兩條thread可以同時處理相同tick，不設跨隊先後順序；回報以`(team_id, replica_tick, team_sequence)`關聯。

替代方案是一條thread依序維護兩個world，CPU競爭較低，但不符合模擬兩個獨立client同時執行的要求，因此不採用。

### 2. 共用deterministic gameplay phase runner

從`State::tick()`與omfx `sim_runner`抽出`DeterministicGameplayPhases`，固定包含dispatcher、所有pending queue drain、兩次outcome boundary、tower ability、script dispatch與creep wave。Authoritative adapter負責完整world與projection facts；replica adapter只處理disclosed world與filtered injections。

禁止為observer另寫簡化phase順序。Production server observer不得建立`NoopDisclosedWorldStepper`。

### 3. Filtered world從空world bootstrap

Replica world builder只註冊安全component/resource與公開catalog，不先載入完整地圖entity。`FilteredTeamSnapshot`建立初始disclosed entities；`RevealEntity`在PreStep建立local Specs entity，`HideEntity`與`ForgetEntity`在該tick step前移出simulation。`LastKnown`與`Silhouette`只寫入獨立render memory。

### 4. 單一global seed與tick-local RNG

`TeamGameStart`包含相同`global_seed`。每tick計算`tick_seed = hash(global_seed, tick)`並建立新的deterministic RNG stream，不加入entity/system/action domain key。

因Specs system平行執行，system不得直接競爭共享RNG。需要隨機值的工作先產生帶stable ordering key的request，在barrier排序後由tick-local stream依序取值。相同team world內request順序必須固定。Hidden entity不在team replica產生request；hidden random結果影響disclosed state時由server送external effect。若server與replica仍因request集合不同而分歧，repair前hash必須抓到差異，之後以server correction收斂。

替代方案是以entity/system/action建立counter-based domain；使用者已選擇較簡單的`global_seed + tick`，因此不採用。

### 5. External effect是hidden dependency邊界

只有全部dependency都已disclosed的行為才能在replica local simulate。Hidden attacker、projectile、caster、collision或AI影響visible state時，authoritative server建立不含canonical ID與hidden position的external effect。Replica把effect注入既定phase，不建立hidden surrogate entity。

### 6. 移除steady-state主動repair

`enqueue_visible_demo_repairs()`不得在每tick替所有可見entity同步component。正常變化由replica simulation產生。Reveal baseline、明確mismatch repair、entity replacement與filtered rebase保留。

Checkpoint流程固定為：完成local step、計算`pre_repair_observed_hash`、比較server expected hash、記錄mismatch、最後才套用PostStep correction。Repair後另記`post_repair_hash`，不得拿它冒充deterministic parity。

### 7. Server永遠是最終權威

單component差異使用`ComponentRepair`；entity layout或多component結構差異使用`EntityReplace`；sequence gap、無法安全diff或連續mismatch使用filtered rebase。Rebase後仍持續失敗則安全終止secure match。Observer與client都不得修正server。

### 8. Observer lifecycle屬於match

兩份observer bootstrap在match建立時產生，不等待玩家連線或session queue成功。Bootstrap仍經過正式filtered encode/redaction path。玩家disconnect、reconnect不建立或刪除server observer；match結束時向兩條thread送shutdown並join。

### 9. Reliable outbound使用阻塞enqueue

Authoritative tick建立Team 1、Team 2 encoded frame後，必須把它們成功送入broadcaster-owned reliable bounded queue才能完成。禁止忽略`try_send`錯誤。Blocking只等待queue取得ownership，不等待socket、玩家ACK、observer step或hash結果。

Broadcaster對network sessions與對應team worker分送同一份`Arc<[u8]>`。Queue持續滿載時記錄deadline miss；超過watchdog上限（預設5秒）安全終止secure match，不跳sequence、不丟frame、不降級protocol。

### 10. 平行完成順序不影響結果

Team 1與Team 2 worker不設全域barrier，也不等待對方完成。Repair coordinator以完整key儲存report，不能依channel arrival order挑選repair。測試會交換、延遲兩隊完成順序，確認frame bytes、authoritative state與repair decision不變。

### 11. Script runtime隔離

兩條worker各有獨立`ScriptRegistry`與world-local resource。DLL module可以唯讀共用，但script callback不得依賴process-global mutable state。若檢查發現global mutable state，實作必須先隔離或移除該狀態，不能以Noop observer繞過。

### 12. 完整驗證集中在最後

實作階段可以新增focused test fixture與測試程式，但不在每個小任務重跑完整workspace。所有unit、differential、fault、security、performance與soak suite在功能全部接通後集中執行。

## Risks / Trade-offs

- [三份完整simulation提高CPU與記憶體] → 固定只支援兩隊、量測每隊step p99，並以10,000 entity雙worker壓力測試作為blocking gate。
- [`global_seed + tick`對request集合敏感] → Stable request ordering、hidden dependency改external effect，並以repair前hash確保差異不會被掩蓋。
- [阻塞outbound queue可能拖慢authoritative tick] → Queue只承接記憶體ownership、broadcaster獨立執行，並以deadline metric與5秒watchdog安全終止。
- [平行worker完成順序造成非確定repair] → Report使用完整key，coordinator只依key與revision決策，加入completion-order permutation suite。
- [Script DLL含mutable global] → 新增隔離檢查；任何finding是blocking issue。
- [移除每tickrepair後暴露既有simulation缺口] → 逐一補齊filtered component、input、RNG與external effect policy，不恢復proactive repair掩蓋問題。
- [Observer落後但network仍前進] → 每隊獨立coverage與lag metric；落後範圍不得標記verified，必要時filtered rebootstrap。

## Migration Plan

1. 建立新change並宣告舊Noop observer parity evidence失效。
2. 抽出共用phase runner，先讓authoritative與既有omfx adapter使用。
3. 建立filtered Specs world builder與真正stepper。
4. 加入global seed wire欄位與tick-local RNG。
5. 建立兩條match-owned observer thread。
6. 移除steady-state主動repair，啟用pre-repair hash。
7. 將outbound改為blocking reliable enqueue與watchdog。
8. 讓omfx與server observer使用同一runtime。
9. 完成後集中執行所有驗證gate。
10. 只在所有blocking gate通過後把新observer標記production-ready。

Rollback只允許在match開始前選擇明確non-secure legacy mode。已開始的secure match不得runtime downgrade；發生不可恢復錯誤時安全終止。

## Open Questions

沒有需要使用者決定的未解問題。實作時若實測queue watchdog或performance gate需要調整，只能在不允許frame丟失、不停用任一team worker、不降低資訊隔離的前提下調整，並在evidence記錄實際值。
