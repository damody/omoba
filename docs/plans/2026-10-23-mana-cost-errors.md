# Mana cost modifier errors and prevention

Preflight found checked_mana_cost validates only its already-formed multiplier, while mana_cost_mult uses wrapping Fixed64 sum/add and treats malformed stat values as absent. Both formal authority and bot budgeting consume it. Use checked_sum_add with existing family policy, checked one-plus and explicit None; valid negative values retain zero-cost semantics. Do not map invalid data to a free spell or introduce a second stat spelling: existing key is manacost_percentage. No failed runtime test is claimed for this inspection finding.

Workflow preflight validates exact CamelCase worldroot plus native capture flag and removes the prior cooldown supervisor recovery dependency. Keep old acceptance scripts/reports immutable.

Read-only inspection guessed native/mod.rs; rg rejected it, actual module is native.rs (discovered with rg --files). No mutation. Continue exact-path discovery before reads. While V1 compiling, found newbotfixture ManaPool not reexported by runtime::*; keep current sources frozen, wait full retirement, preserve original inputs/results before fixing local test import.

V1 actual compile failure E0433: new bot fixture omitted local ManaPool import (runtime::* intentionally reexports only selected ability types). Core964 passed, base tests did not run. V1 owner/watchers retired/reaped; exact4sources+manifest and original logs/ready/false supervisor receipt preserved before fix. Add explicit test-local import; no production behavior changed for this correction. FullV2 cohort required. Prevention: inspect actual reexport boundary and explicit imports when copying a fixture; do not assume wildcard exports every nested type.

Final accepted: 1813passed/0failed/9existingignored；World71全執行；3433正式輸入一致；145份證據／34PNG；health8 samples／max172秒；來源／證據獨立驗證，所有reader/watchers/health退休reap。
