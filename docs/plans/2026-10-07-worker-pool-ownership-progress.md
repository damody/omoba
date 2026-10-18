# Worker pool ownership holes

## 這次修什麼

先前唯讀檢查指出的是既有 generic budget 的 pool 歸屬缺口，不是 133/132ms 尖峰的證明原因。這輪只補三個呼叫點，沒有改 Rayon process-global 設定，也沒有再建立一個 pool。

1. `SystemDispatcher::run_post_commit_visibility` 在 `Dispatcher::dispatch` 的 `install` 結束後才跑 Wave B。`run_team_wave_b_parallel` 的 `par_iter_mut` 因此會落到 lazily created 的 global pool。現在整段 visibility 包在這個 dispatcher 已持有的 `Arc<rayon::ThreadPool>::install` 裡。
2. `SCRIPT_ON_TICK_PARALLEL` 的非 managed `par_iter` 同樣在 dispatcher install 之外。`StateInitializer::initialize_resources` 原本收到 pool 卻丟掉（`_thread_pool`），所以 world 上沒有可安裝的 owner。現在把同一份 `Arc` 留在 `OwnedEcsPool`。有這個 resource 時，非 managed on_tick 進入該 pool。沒有 resource 的裸 harness 維持原本的 ambient `par_iter`。Managed / mana 路徑仍是排序後的 `iter`，沒有放進 install。`normalize_authoritative_no_target` 沒有改。
3. `SpecsDisclosedWorldStepper::rebootstrap` 呼叫 `FilteredReplicaWorldBuilder::empty`，會 `create_thread_pool` 再在函式結束時銷毀。`from_start` 已經改走 `empty_with_pool`。rebootstrap 改為 clone dispatcher 上的既有 `Arc` 再 `empty_with_pool`。成員 rebase、缺席 entity drop、allowlist 與 script registry 保留邏輯沒有改。公開 `empty()` 仍自己建立 pool，不借用別的 dispatcher；world 會留住那份 `Arc`，所以 standalone world 不再在 `empty` 返回前把 workers join 掉。

`build_phase3_dispatcher` 仍另外建立滿 `num_cpus` 的 pool，SimulationDriver / UE / omfx 的 system dispatch 還是用它。那些路徑的 script on_tick 用的是 world setup 留下的 budgeted `OwnedEcsPool`（`create_thread_pool` / `OM_ECS_WORKER_THREADS`），不是 phase3 pool，也不是 global pool。`reconfigure_thread_pool` 只換 dispatcher 上的 pool，不更新 world resource；正式 tick 不呼叫它。

沒有改 tick、gameplay hash、ABI、wire、content、schema、順序、OM_PERF、sample math 或 fixed12 門檻。這不構成 historical 133/132ms spike 已修復。

## 測試

沒有在這輪編譯、測試、bench 或開遊戲。終端上有進行中的 PIE startup/integration，依指示留給後續驗證。沒有讀 Engine/Editor log，沒有改 UE/native fixture。

建議 exact filter（裸名稱加 `--exact` 會跑 0 tests）：

```bat
cargo test --manifest-path omoba-core/Cargo.toml --lib --exact runtime::native::system_dispatcher::worker_pool_ownership::post_commit_visibility_uses_dispatcher_pool
cargo test --manifest-path omoba-core/Cargo.toml --lib --exact runtime::native::scripting::dispatch::worker_pool_ownership::parallel_on_tick_uses_world_pool
cargo test --manifest-path omoba-core/Cargo.toml --lib --exact runtime::native::scripting::dispatch::worker_pool_ownership::managed_on_tick_stays_off_world_pool
cargo test --manifest-path omoba-core/Cargo.toml --lib --exact runtime::filtered_specs::worker_pool_ownership::rebootstrap_reuses_dispatcher_pool
```

不帶 `--exact` 時，`worker_pool_ownership` 會選到上面四個。Post-commit 與 script 測試用 `num_threads(2)` 的 scoped pool，不寫 process env。Post-commit 比對 Wave B 實際 worker 的 `current_num_threads` 與 `ThreadId` 是否落在該 pool 的 `broadcast` 集合。Parallel on_tick 由 script hook 回報 `ThreadPool::current_thread_index` 與 worker 數。Managed 測試只確認 hook 沒有進入該 pool，不呼叫會初始化 global registry 的 `rayon::current_num_threads()`。Rebootstrap 比對 dispatcher 與新 world `OwnedEcsPool` 的 `Arc` 同一性，並確認替換後舊 entity 不留在新 world。`TeamVisibilityState` 的 witness 欄位只在 `cfg(test)` 存在。

## 檔案

- `omoba-core/src/runtime/native/initialization.rs`
- `omoba-core/src/runtime/native.rs`
- `omoba-core/src/runtime/native/system_dispatcher.rs`
- `omoba-core/src/runtime/native/scripting/dispatch.rs`
- `omoba-core/src/runtime/visibility.rs`
- `omoba-core/src/runtime/filtered_specs.rs`
- 本檔

## 編譯失敗（feature-tests.log）

`cargo test --manifest-path omoba-core/Cargo.toml --lib worker_pool_ownership` 沒有跑到測試。`omoba-core` lib test 只有一個編譯錯誤，日誌在 `.ai-collab/.bridge-data/worker-pool-machine-v1/feature-tests.log`：

```
error[E0597]: `hits` does not live long enough
   --> src\runtime\native\scripting\dispatch.rs:1525:9
    hits.lock().expect("probe lock").drain(..).collect()
error: could not compile `omoba-core` (lib test) due to 1 previous error
```

`dispatch_probes` 把 `drain(..).collect()` 放在尾端運算式，`MutexGuard` temporary 活過 `hits` drop。只改這個測試：guard scope 內收集到具名 `Vec<Hit>`，scope 結束丟掉 guard 後再回傳該 vector。沒有改 production。這筆修正還沒重編譯；四個 feature test 留給後續重跑。

## Primary 功能接受（2026-10-07）

Grok writer run-muxpldmz-9jh1zt／thread4451370d-7504-4aa8-8f69-89a31bd841fe completed816s／USD4.53590736，86calls，input365720／cached10994944／output73590／reasoning59645／total11434254。Primary審查六個Rust來源diff，既有None normalization保留、owned resource不進canonicalhash／ABI／wire，cfg(test) witness只觀測真WaveB worker。

首次cargo test編譯E0597，程序101、沒有執行test，不算PASS。Fast bounded repair run-muxqc8cm-ngkerr／thread713b1792-413c-4212-92f2-918268d99745 completed50s／USD0.10886664，只修測試guard scope。Primary重跑 `cargo test --manifest-path omoba-core/Cargo.toml --lib -j 4 worker_pool_ownership -- --nocapture`，exit0、4 passed／0failed／0ignored／504filtered；四項完整名稱與原始log在ignored worker-pool-machine-v1/feature-tests-v2.log。current feature接受，不宣稱歷史尖峰已修復。

原建議exact命令將--exact放cargo參數端不正確；正確應放在`--`後，且fully-qualified testname必須一致。本輪用共同filter worker_pool_ownership，真實4項執行，不用零測試判PASS。

使用者改要求先看整場模擬。same-PIE-continuation-v1原ownedEditor仍cold D3D11shader初始化，尚無authorityspawn，新場次尚0／batch8/10。此場使用前次一致staged artifacts；目前不重建或覆蓋被Editor載入的DLL。待其退出／清理後補正式core消費產物建置／stage，不把目前測試PASS當production已部署。whole fixed12與整體4項未勾。

## Production build and related filtered-world validation completed

Primary正式release server/moba-config、client runtime、base_content均建置exit0；debug bridge/content已由正式Lua入口stage且SHA一致（6cb1145683ccc10d4a21e8dc73e416c164bbbfa9ac041ebb03ce5aa9d0b8bd0d／1fc65608a4fb46d3aa9db5f531289256f55e35ac3a606abe3e9b3d6c951b249c）。正式NoEngineChanges C++local建置315.40秒成功，新Editor actual PIE選角與exact-widget capture通過、cleanup=true、0新對局。上述production pending為歷史。

追加受影響filtered_specs回歸真實19passed／0failed／489filtered（filtered-specs-regression.log），含新rebootstrap pool同一性與既有committed actor state／canonical/filtered projection。固定十二性能仍未重驗，先前133/132ms尖峰不能因pool測試通過就宣稱修復。真authority batch10/10，不能reset。
