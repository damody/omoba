# 唯一任務：thread pool reconfigure 必須退役舊 cached dispatcher

Grok新fresh上下文，bounded小修。cwd D:/code/omoba，root HEAD c5211e8311cdca867841fcc1494899634b9d17b1/master，全部dirty保留。讀AGENTS.md及omoba-core/src/runtime/native/system_dispatcher.rs；不用全庫探索，不讀其他.ai-collab歷史packet。

具體已證明問題：run_systems快取Dispatcher且綁定建立時Arc pool；reconfigure_thread_pool只換self.thread_pool，沒有清cached dispatcher。新pool建立成功後須清self.dispatcher=None；build失敗原狀保持。不改system依賴/執行順序/世界/hash/ABI/OM_PERF/workerbudget/priority。這是API正確性，不宣稱修掉實測351/475ms。

只改system_dispatcher.rs及docs/plans/2026-10-06-dispatcher-reconfigure-progress.md。新增#[cfg(test)]小module單一reconfigure_retire_cached_dispatcher測試：建1thread pool與空DispatcherBuilder綁該pool，初始化SystemDispatcher cachedSome；reconfigure2後cachedNone/current_num_threads2，second同threadcount仍清cached；不跑gameplay不載DLL。不為測試公開API、不改歷史build_phase3_dispatcher/omfx。用apply_patch，禁止commit/push/rebase/reset/clean/restore/branch/install/credentials/killprocess/externalservices/security/engine/UE/omfx。

只執行一次cargo test --manifest-path omoba-core/Cargo.toml reconfigure_retire_cached_dispatcher --lib（至少1真正test，非0）；不fullsuite、release、simulation、UE。新build若出既有錯誤回primary，不碰其他檔。錯誤寫你的progress MD，由primary中央E323整合。禁止subagent特權操作，需要時回exact command/targets/risk/reversiblealternative，不問user。完成即回actualdiff/testcount/exit/warnings/limits，留primary獨立審查驗證，不勾6.5。
