# 原生 workflow 監督實作

正式role啟動（非prepare-only）、headless選角host與remote選角join共用 `process.supervise_workflow`。外層固定Lua等待native helper；helper持有outer Lua原始handle與private Windows Job，inner Lua／所有新descendants在job內。help／prepare不建立Job，既有spawn API保持原樣。

native以暫停建立→assign→owner存活確認→resume順序阻止「先跑才assign」窗口；所有failure僅停止new child原handle。出生順序与精確固定Lua／cmd路徑驗證防止PID reuse，內層marker沿native會員檢查防繞過。正常root退場也終止job內殘留並等待active0；outer強制终止／helper crash由Job handle生命週期兜底。無remote-host、existing Editor或name-kill。

Windows-only，無非Windowsfallback。Job assignment受OS政策拒絕時fail closed，不使用breakaway。Windows canonical身分與Lua可消費argv／cwd分開，不放寬比對。此監督保護本機新流程樹，不是跨主機服務監督或全程序daemon。

## 主agent独立確認

- `cargo test --manifest-path tools/lua-host/Cargo.toml -- --test-threads=1`：9 passed、0 failed／ignored。
- 固定Lua `scripts/tests/workflow_supervisor_test.lua target/workflow-supervisor-20261006-v5`：5cases PASS，只有無遊戲固定Lua fixture。normal／kill outer兩種原始子孫全退出、fake env拒絕、non-Lua祖先拒絕、exit7及特殊args精確保留。
- 三正式入口help exit0，無啟動遊戲。source freshness遍歷Rust src，包含新native module。
- Grok601秒cancelled無補丁，已確認原process退出；本native patch由primary實作，無虛報Grok驗證／費用。細節見E340。

目前OpenSpec27/31；4.1與4.3已依原契約封關。4.4／6.2／6.4／6.5各自驗收，native fixture不替代它們。
