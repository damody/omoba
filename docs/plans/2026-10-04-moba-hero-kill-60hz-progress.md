# 英雄直接擊殺獎勵／60Hz 增量驗收

## 計畫與決定

1. 獎勵接在權威 Damage 首次正 HP → 零 HP 的正傷害結算，不依 input ACK、Death 通知或 UE 自算。lethal_pending標記同一生命首次致死、重生才清除，防同批overkill或lethal→Heal→lethal重付；NPC先擊殺後同批hero命中也不能搶獎勵，重生後新擊殺仍正常支付。
2. Lua `moba_economy.hero_kill_gold = 300` 生成 Rust 常數；整數／非負 schema、上限1,000,000、完整canonical hash、compiled/runtime agreement與禁止單端hot reload一起驗證。
3. 僅接受當前敵方英雄 source／target 身分；排除自己、NPC、建築、暖場、pause、終局。Gold／kills飽和，死亡重生保存。沿用owner-team committed economy，filtered client不發錢。
4. 增加 deterministic `Production60Hz`，時間總和与15／120Hz的週期餘數相容；不是render frame-time效能保證。
5. runtime smoke所有模式預設60Hz，不改使用者game.toml；報告明列tick_rate_hz。

## 驗證結果

- base_content全部77 passed；含60Hz真正致死、同批overkill、killer死亡重生、固定seed digest重播，以及self／NPC／建築／inactive不付、Gold飽和。
- 直接擊殺後125ticks／250雙隊apply：owner-only金錢、victim死亡重生、canonical hash與fresh authority view一致，零ComponentRepair。
- 60Hz完整filtered對局5254ticks／10506steps，Finished5240／winner side0，deaths與respawns[4,2]；唯一game.end與15終局ticks凍結、逐tick雙隊hash一致。既有15／120Hz回歸通過，不宣稱120Hz效能。
- template-ids的runtime-lua-content：29unit＋23generated＋8hero＋1catalog，共61 passed；新kill-gold full hash／compiled mismatch／dev reload拒絕與非整數拒絕通過。
- core --lib：323 passed；加入60Hz clock後再跑simulation_driver::tests，3 passed。
- build_ue_moba.lua --build-only，session16479 exit0：11生成檔、content_hash de9c7fcfc98d6479、bridge與OmGame編譯成功。stage SHA256 `426722b006d14bcddd2f87a673bda34c301d3be661e3bb5c3a4a736bcb8d0684`校對通過。
- 真實60Hz KCP：run1791070108／session11578 exit0，report success與cleanup_verified=true；實際config STEP_FPS60、report tick_rate_hz60，兩隊safe_tick1095、各9 checkpoints到1080、實際位移。server24408／runtime84996、69776均退出，CIM核對無殘留。raw在target/interactive-runs/moba-runtime-1791070108。
- 首次run1791070018是120Hz相容性診斷，不列60Hz證據。

最終生命去重補強後：base全部77與core全部323再次通過；額外NPC→Heal→hero分支回歸1 passed。build-only89587 exit0、codegen內容hash不變，最終stage SHA256 `42444994e243490628748b83abb33963bc3a2905c6503978f4a86bc55223d363`通過，取代上面第一版stage。未用Editor或UE操作驗收冒充此build-only結果。

最終新版KCP重跑：run1791070433／session67055 exit0，success／cleanup_verified=true、config與report皆60Hz、兩隊safe_tick1103，各9 checkpoints至1080。server99140／runtime82396、24728退出，CIM核對三者及Editor均無殘留；原始report與logs位於target/interactive-runs/moba-runtime-1791070433。Lua syntax、生成器11檔--check、OpenSpec strict、scoped git diff --check均通過（只有既有CRLF提示）。

## 重現指令

```text
cargo test --manifest-path scripts/Cargo.toml -p base_content
cargo test --manifest-path omoba-template-ids/Cargo.toml --features runtime-lua-content
cargo test --manifest-path omb/Cargo.toml -p omoba-core --lib
tools/lua/lua.exe scripts/build_ue_moba.lua --build-only
tools/lua/lua.exe scripts/run_moba_runtime_smoke.lua
```

## 邊界與下一步

未新增角色C++／Blueprint graph／protocol欄位。助攻、召喚物／已退休hero DOT歸屬、公開擊殺事件與計分板、回城、多人團隊仍未完成；真實網路測試是新內容的移動／hash回歸，不是UE擊殺畫面或frame-time验收。5.3保持未勾選，仍19／30。下一個玩法增量先補可取消的權威回城，再處理多人歸屬與助攻。錯誤與防重犯見E105。
