# 本批唯一有效任務：Windows 程序 lifetime 安全閘門

本批只執行本檔末尾「本批唯一任務：Windows 程序 lifetime 安全閘門」的完整packet。下方直到該標題前所有內容是已完成歷史，禁止執行；無界report移除已完成並獨立接受。禁止重做它或歷史對局。主agent先前誤把新packet append在尾部，現在此頂部指令明確指定唯一有效packet。

---
以下是歷史：
# 已完成：移除未使用的逐step報告歷史（2026-10-06）

這是新限定任務；下方舊任務全部完成，只保留歷史，不執行其指令。root HEAD c5211e8311cdca867841fcc1494899634b9d17b1/master，保留所有既有dirty/untracked。使用同一Grok thread，不重做replica_stage診斷。讀AGENTS.md、omoba-client-runtime/src/replica_host.rs與main.rs checkpoint/evidence callers；rg所有pre_repair_reports引用。當前private BTreeMap在每Applied插入report.clone但無讀取/清除/對外API，無界歷史不應保留。

只允許編輯omoba-client-runtime/src/replica_host.rs與docs/plans/2026-10-06-unused-replica-report-history-progress.md。若確認無消費者，刪未使用field/初始化/插入與多餘import（BTreeMap若其他使用則保留）。保留ReplicaApplyReport、返回report、encoded SHA/pre/post hashes、sequence、fog、stage profiling以及main checkpoint writer/evidence安全閘門。不要另設任意history cap或改成默默漏checkpoint，不改wire/ABI/內容/hash/原OM_PERF/十二門檻。此修正只消除可證明的無界重複retention，不宣稱50ms尖峰根因或效能通過。

用實際已有精準測試filter（report/profile relevant至少1真test，不跑fullsuite或DLL）確認接口行為，cargo test --manifest-path omoba-client-runtime/Cargo.toml replica_stage --lib 一次，cargo check --manifest-path omoba-client-runtime/Cargo.toml --bin omoba-client-runtime --features compiled-content-only 一次。若純報告生成缺可直接測路徑，不為測試創另一份玩法或多餘公開API，說明coverage limitation即可。提供所有引用搜尋結果與刪除證據。只需要此通用fix，勿無界探索別的performance原因。

禁止執行對局/simulation/UE/release/buildstage/install/credentials/externalservices/security/killprocess/engine/omfx或commit/push/rebase/reset/clean/restore/branch。特權需求回primary exact action/target/risk/safer alternative，勿問user。apply_patch編輯，preserve dirty。錯誤只写你的progress MD，primary整合E323，勿改中央error/tasks/scripts/.ai-collab（primary並行此處）。終結繁中回actual diff/tests/count/exit/warnings與coverage，不勾任務，不稱完成全遊戲。

---
以下全部是歷史，不是本批任務：

# 正式 client step 分段診斷（已完成）

只補通用診斷，不跑對局／完整驗收。root HEAD c5211e8311cdca867841fcc1494899634b9d17b1/master，所有既有dirty/untracked保留。下方舊第5批全部完成，禁止重做或执行其中指令。

讀 AGENTS.md、omoba-core/src/runtime/selective_replica.rs、omoba-client-runtime/src/replica_host.rs、omoba-client-runtime/src/main.rs(apply_ready_frame)、omoba-core/src/comp/perf_window.rs；scripts/moba_performance_report.lua只讀。允許僅以上4個Rust檔案、必要新純Rustdiagnostic小module及mod接線、docs/plans/2026-10-06-replica-stage-diagnostics-progress.md。primary並行scripts/UI，不改scripts/Unreal/tasks/.ai-collab/其他docs。

已固定max50ms，實測52.654/51.626ms仍失败，原因未知。新增可選measured/profiled apply，具名非重疊phase：decode、preflight/pre-step/injection/baseline staging、fixed_step、restore/allowlist/pre-repair hash、post-step repair、post-repair hash、ReplicaHost fog/SHA/report/bookkeeping(host_finalize)。不用量測的其他caller不必讀時鐘。保留原API／結果／hash／錯誤與操作順序，時間不進world resource、canonicalhash、wire/ABI/catalog。

main只成功Applied採樣，duplicate/stalled/error不採樣／不洩漏上次profile。每60 successful samples输出獨立OM_REPLICA_STAGE JSON版本、player/team、ns/wall-clock scope、samples、outer最慢sample tick/sequence與同一sample全部phase/outer/residual。不是各phase獨立max相加，tie保留首次。固定有界struct/array，不存每tick無界history。phase sum<=outer，剩餘時間保留而不臆測CPU原因。Formatting/logging在原outer timer停止後；既有OM_PERF/ENABLE及12門檻完全不改，不扣未知成本／擴大門檻／挑樣本。不得說效能已修復。

Pure tests：窗口有界、同一尖峰sample全phase、tie/reset、invalid sum拒絕；量測/未量測結果與hash一致，duplicate/error不產生stale profile（用Noop stepper，不載DLL）。找到精準filter後各跑1次 cargo test --manifest-path omoba-core/Cargo.toml <newfilter> 和 cargo test --manifest-path omoba-client-runtime/Cargo.toml <newfilter> --lib，確保各至少1個實際test，不count0；cargo check --manifest-path omoba-client-runtime/Cargo.toml --bin omoba-client-runtime --features compiled-content-only。只對真錯修，不跑fullsuite/release/DLLstage/UE/simulation。

apply_patch編輯。禁止commit/push/rebase/reset/clean/restore/switchbranch/install/credentials/externalservices/security/killprocess/engine/omfx變更。特權需求回primary原因/exact action/targets/risk/reversiblealternative，勿問user。錯誤寫你的progress MD，primary整合E322。不要無界探索，scope外真blocker留下partialdiff並回報。最後繁中回實際files/API/gates/testnames/count/exit/warnings/剩餘；留Codex獨立審查，不能勾6.5。

---
以下全部是完成後保留的歷史，不是本批任務：

# 第5批限定修復：兩個具體語意邊界，不重讀舊任務

最高優先：下方第5批已completed run-muw3v0yp-57zms7，seed1在62951ticks自然Finished+replay，原證據result.json不得改／刪／覆蓋。Codex讀完整相關diff與HEAD/whitespace，root c521／omb578／UE1e29保持。以下三個具體findings是本次全部任務，不再重新調查Bot或跑對局。

1. omb/src/bin/moba_headless.rs objective_slots_progressed 用entity_id!=0認定alive，但specs Entity的ID0合法，collect以dead tuple(0,0,0)編碼。改用generation非零作存在判斷，或明確alive，不放寬0ID；測合法id0/gen1的HP減少及退休、空slot/gen0無進展、相同id新generation層替換，保留有界與stall測試。
2. 同檔report winner_team直接寫Finished.winner，它是Option<u8> side索引，正式對局teams[1,2]卻報winner0。參照omoba-core/src/runtime/native/moba_match.rs:1667的game.end mapping：winner_side保持內部side，可新增該欄；winner_team必須對config.teams索引映射、draw為null、非法side fail closed。加helper test teams[7,11]、side0/1/None/2。scripts/moba_headless_batch.lua verify須拒team0或不在role_plan的winner_team；合法兩隊/真正draw允許，測試fixture補正式team_id，不取消其他gate。不改world胜負或舊result文件。
3. diagnostic_tests現用固定PID/temp leaf，測試開头remove_dir_all可能刪之前檔案。改唯一create_dir預留，禁止刪既有目錄；只清理本次已創建具名檔案並remove_dir空目錄，無recursive deletion。勿重做存檔架構。

允許僅headless.rs、scripts/moba_headless_batch.lua、scripts/tests/moba_headless_batch_test.lua與bot-deadlock-progress.md/error-register新增E311。主agent已在cfg(test)自行修core missingBuffStore、base projectile missingObservableFactBuffer、mana fixture authored+2regen、omb rolemerge fixture與bridge oldABI常數；這些既有dirty你不得還原/編輯，root/submoduleHEAD未移動。no omfx/Unreal/Bot/ABI/data/map/rules changes。

只跑 affected headless diagnostic_tests（compiled-content-only）與固定Lua batch9+新cases；release host build一次（讓主agent後續實際batch使用修正版本），不用另buildDLL（runtime/core非test實作沒變）。不跑seed1/100/UE/fullsuite。不要commit/push/rebase/reset/clean/restore/branch/credentials/destructive/externaloperations，apply_patch；HEAD移動停寫。繁中回實際tests/compile，MD記錯。修復完成即返回，不再無界探索。

---
以下是已完成第5批背景，禁止重做：

# 第5批：通用執行預算與真正無進展診斷（新的限定任務）

根目錄D:/code/omoba，讀AGENTS.md。本任務取代舊第4批。root HEAD c5211e8311cdca867841fcc1494899634b9d17b1，omb578f475f97520127d1cc7cc930e7d7786f1cea0f，omfue1e29a27c0f58ff798f82fa13f592a53a967f78ac。使用者要求通用解，自主修正計畫；主agent已停止舊job run-muw29l3a-g23f3i，tracked cancelled，未發現相關worker/test存活。既有dirty包括core bots.rs YieldBehind、base single_lane_match_tests.rs新fixture、omb moba_headless.rs診斷，全部保留，不修改Bot策略，不commit。

實際證據：omb/target/moba-headless-repair-seed1/attempt2.failure-samples.json、attempt3.failure-samples.json，顶層.samples，tick28800→32400仍退休塔，32400→36000基地HP下降；600秒逾時不等於已證明死局。不要重試退讓策略。主agent新決定：將操作預算明示，可診斷真正長時間目標無進展；不改遊戲規則或成功條件。

讀omb/src/bin/moba_headless.rs、scripts/run_moba_headless.lua、scripts/moba_headless_batch.lua、scripts/run_moba_headless_batch.lua、scripts/tests/moba_headless_batch_test.lua。允許僅這些檔及docs/plans/2026-10-06-bot-deadlock-progress.md、error-register新增E307，其他都不改，不讀改omfx、不碰Unreal。

實作：
1. headless新增 --max-game-seconds 明確有界正整數（預設600，範圍60..3600），--stall-game-seconds（預設300，60..3600且<=max），profile乘法checked；時間不是遊戲规则。不能在超時成功。報告帶實際預算及stall scope。自然Finished、恰好1end、combat>0、完整replay hash一致仍必要，plan-only不是對局。
2. 僅headless診斷用有界objective progress monitor；以公開塔／基地真正HP下降、退休／層替換認定進展，不以waves、英雄移動、擊殺或營地respawn充數。不注入world。不讀敵方私有命令供Bot使用。開始後連續stall秒無objective進展要failure，明確原因與最近progress tick。觀測集合/樣本有界，較長運行保留末筆與有界最近歷史；不每frame full snapshot。
3. 所有plan-only/success/failure report要保留原有證據，不可覆蓋任何現有report或paired diagnostic（現在只failure branch辨success是不完整）。用create_new語意，預先拒絕現有輸出；不刪任何歷史。單場Lua預設fresh unique directory（explicit --report原樣但拒重複），批次原fresh目錄保持。
4. Lua batch可傳上述預算，預設保持600/300，在batch summary明記。verify必須與請求預算一致、不因較長預算假成功，現有60Hz/10Bot/guard/replay閘門保持。更新純Lua測試。
5. 新增Rust精準tests：invalid/overflow/duplicate option、HP/退休進展、wave/hero changes不進展、stall boundary、sampling有界與final保存、既有失敗/成功/invalid-json都拒overwrite。不得新增系統依賴或PS/Pythonworkflow。

驗證一次affected Rust tests（compiled-content-only）、固定tools/lua/lua.exe scripts/tests/moba_headless_batch_test.lua；release build base_content compiled-only與omb moba-headless（只變headless可沿現有DLL但host+DLL同rust1.95）。然後僅一場真實seed1：omb/target/release/moba-headless.exe --role-plan omb/target/moba-headless-batches/1791252707-1/role-plan.json --profile 60 --seed 1 --scripts-dir scripts/target/release --max-game-seconds 1800 --stall-game-seconds 300 --report omb/target/moba-headless-budget-seed1/result.json（不存在才run，若存在選新目錄）。保留真實成功或stall/timeout證據。不要100場，不再修改Bot；實際run失敗也回報，主agent決定下一步。最後git diff --check。總工作請收斂，不再無界策略重試。

apply_patch編輯。不要commit/push/rebase/reset/clean/restore/switch branches/edit credentials，不destructive/credentialed/external/permission-sensitive操作；需要時回主agentexactaction/target/risk/alternative。HEAD移動停寫回報。保留所有既有diff、不更改ABI/hash/data/map/角色stats/防守/rules。繁中回報實作、真實commands/tests/seed1/replay/剩餘；MD記錯，不勾完整5.5。
# 本批唯一任務：Windows 程序 lifetime 安全閘門（前文其餘任務全部已完成）

讀 AGENTS.md、tools/lua-host/src/main.rs、tools/lua/lib/process.lua、scripts/run_2player_ue.lua、tools/lua/lib/host.lua。root HEAD c5211e8311cdca867841fcc1494899634b9d17b1/master，既有dirty全保留。primary並行MD/replica collector，不碰那些檔案。

具體缺陷：run_2player_ue clean_previous_session、spawn cleanup與reconnect active record只憑PID/executable；PID可被重用。host.stop目前query path再另開handle terminate有TOCTOU。新增嚴格Windows process identity：native inspect同一handle取得exe+完整FILETIME creation token（十進位字串避免Lua double精度），保持舊created_unix_seconds相容。新增Lua capture/assert/stop_owned identity APIs；native stop_owned必須在同一query+terminate handle核對canonical exe、精確creation token才TerminateProcess，RAII或所有分支關handle。舊stop相容，但本launcher不得再用exe-only stop。Windows以外新strict API明確unsupported，不加shellfallback。

launcher capture spawn identity，cleanup用strict identity；active session保存identity所有五程序；reconnect新PID更新完整identity。舊record缺精確token必須fail closed拒啟動，保留active檔，不誤停；全部preflight核對後才清理，但native仍原子再核對。不要自動刪malformed state。已退出PID可略，inspect異常不可默認已退出（平台API區分not-alive vs query failure）。新identity capture失敗不得註冊exe-only cleanup。純Lua抽小module可測policy，不能模擬成真process安全驗收。

允許只改上列三檔、必要新scripts/ue_session_identity.lua與tests/ue_session_identity_test.lua、docs/plans/2026-10-06-session-lifetime-progress.md。不要改其他launcher/UE/reconnect module/collector/tasks/error/.ai-collab。不要expand全repo migration；記未接線部分。禁止實際stop/kill/spawn遊戲或fixture child、UE、simulation、releasebuild、安裝、credentials、external、engine/omfx、commit/push/rebase/reset/clean/restore/branch。subagent不執行特權操作，必要回primary exact action/target/risk。

只跑native純函式process identity validation tests（你新增精確filter，各至少1），cargo check --manifest-path tools/lua-host/Cargo.toml；固定Lua tests/ue_session_identity_test.lua mock測missing/invalid token/PIDreuse sameexe/preflight failure zero stop/validreversecleanup/reconnectcapture；不跑fullsuite、不執行TerminateProcess。以apply_patch編輯，避免過長探索，完成即回實際diff/testcount/exit/limitations。主agent獨立審查/build真正helper/實際ownfixture後才接受，不宣稱performance修復。

---
以下全部為已完成歷史，不再執行：
本檔為歷史packet；換機後請先讀README.md及最新pause_checkpoint，不自行重啟上述工作。
