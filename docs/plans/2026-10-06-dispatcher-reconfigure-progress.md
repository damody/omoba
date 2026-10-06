# Dispatcher reconfigure：退役舊 cached dispatcher

## Primary 審查接受

Grok job `run-muwbcwhp-fntlwj` / fresh thread `4b620afc-38b6-42ab-aed2-d761afbaf7f5` completed4m36s／follower exit0。primary讀完整實際diff及此紀錄，獨立同一filter 1 passed／481 filtered、exit0；normal git diff --check通過，root／omb／omfue HEAD未動。接受這個限定API修復，不接受50ms性能修復；目前正式對局沒有呼叫reconfigure_thread_pool，不能把這筆缺陷當已證明的tick227原因。

metrics input146091/cached1209216/full1355307/output18464/reasoning13788/total1373771，32modelCalls／32turns，reported USD0.34257516，API duration unavailable。前一本批retention完成USD0.24877936，兩完成jobs合計USD0.59135452；兩cancelled工作cost未知，不把合計當全批費用。

狀態：有界實作完成，留給 primary 獨立審查與驗證。不勾 6.5。這是 API 正確性，不宣稱修掉實測 351.1038 / 475.328 ms，也不改 50 ms 門檻。

## 問題

`run_systems` 把 `Dispatcher` 快取在 `self.dispatcher`，且該 dispatcher 綁定建立當下的 `Arc<ThreadPool>`。`reconfigure_thread_pool` 先前只替換 `self.thread_pool`，cached dispatcher 仍指向舊 pool。

## 變更

只改 `omoba-core/src/runtime/native/system_dispatcher.rs`。

- 新 pool `ThreadPoolBuilder::build()` 成功並寫入 `self.thread_pool` 之後，設 `self.dispatcher = None`。
- 執行緒數相同也清快取，沒有因 count 相同而 early-return。
- build 失敗走既有 `?`，發生在任何欄位寫入之前，原 pool 與 cached dispatcher 保持。這條沒有另開測試（本批只允許一個測試）。
- 未改 system 依賴、執行順序、世界、hash、ABI、`OM_PERF`、worker budget、priority。
- 未改 `build_phase3_dispatcher`，未改 omfx，未為測試公開 API。
- `#[cfg(test)]` module `reconfigure_retire_cached_dispatcher_tests` 只有 `reconfigure_retire_cached_dispatcher`：1-thread pool、空 `DispatcherBuilder` 綁該 pool、`SystemDispatcher` cached `Some`；`reconfigure(2)` 後 cached `None` 且 `current_num_threads==2`；再綁一次空 dispatcher 後以相同 thread count `reconfigure(2)`，cached 仍 `None`。不跑 gameplay、不 `dispatch`、不載 DLL。

## 測試

只跑一次：

```bat
cargo test --manifest-path omoba-core/Cargo.toml reconfigure_retire_cached_dispatcher --lib
```

- exit code：0
- test count：1 passed；0 failed；0 ignored；0 measured；481 filtered out
- 名稱：`runtime::native::system_dispatcher::reconfigure_retire_cached_dispatcher_tests::reconfigure_retire_cached_dispatcher`
- 耗時：`Finished test profile in 10.28s`；該測試 `finished in 0.00s`
- 本檔無 warning。既有 warning（未改那些檔）：`omoba-template-ids` build script `td_rounds.rs` dead_code ×3（`validate_layer_catalog`、`REGROW`、`FORTIFIED`），`generated 3 warnings`。

未跑 full suite、release、simulation、UE。無 commit / push / rebase / reset / clean / restore / branch。其他 dirty 未動。

## 限制

- 6.5 未勾。client max 仍超固定 50 ms 的歷史實測不在本批範圍。
- build 失敗保持原狀只由控制流保證（`build()?` 先於寫欄位），沒有失敗注入測試。
- 單測不執行系統、不證明下一次 `run_systems` 會用新 pool 重建依賴鏈；那是既有 `dispatcher.is_none()` 分支，本批未改該分支。
- 由 primary 中央 E323 整合。本紀錄不代替審查。
