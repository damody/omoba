# 權威成功施法事實發布（2026-10-05）

## 問題與決策

- E217 已阻擋失敗的 SkillCast visual，E218～E220 已提供安全格式、IPC 保留及 Unreal 消費；但真正 dispatcher 的成功事件仍未進入 ObservableFactBuffer。只完成下游消費不代表遊戲施法能送到畫面。
- 在共用 dispatcher 的事件 batch 成功 gate／回滾完成、immutable adapter cache 釋放後，僅把仍保留的成功 SkillCast visual 透過既有 `script_visual_event_to_observable_fact` 發布為 Ability。其他 hook 不改、不把 queued／accepted input 當成功，不從 legacy snapshot drain 補播。
- 使用已存在的 HERO_ABILITY policy 與 ABI-friendly ProjectionPolicyId；註冊來源路徑及 visual 的 explicit policy，不新增放寬 audience。後续安全 projector 仍要求 caster 目前可見且 Disclosed，不揭露 hidden target 或猜座標。
- 新 `ScriptCastFactOrder` producer-local resource 在每 tick 重設 ordinal，跨同 tick 多次 dispatcher drain 持續遞增；team projector 再配置最終安全 wire ordinal，不暴露 canonical producer ID。
- StateInitializer 的基本 ECS 初始化原本有 fact buffer，但 policy registry 只在完整 content-world 建立流程加入。把 secure defaults registry 與 fact buffer 放同一基本層，亦初始化 cast ordering resource；不在 dispatcher 臨時建立或容忍缺少 policy。沒有 ObservableFactBuffer 的 bare legacy hook harness 維持原 visual-only 行為，不用這個相容邊界作正式 MOBA fallback。

## 局部實作確認

指令：`cargo test --manifest-path scripts/Cargo.toml -p base_content --features compiled-content-only successful_cast_facts -- --nocapture`。

最後版本 3 passed／0 failed／170 filtered out／exit0：

- Production60Hz headless driver＋真實 generated handler＋正常 PlayerInput：同隊不合法目標不產生 Ability，成功施法產生唯一 Ability 與 HERO_ABILITY audience，冷卻拒絕及空 step 不補播。
- 明確將真正 handler 產生的 fact 交給共用 TeamViewProjector，已揭露 caster 可取得精確 skill ID 並轉 ABY1；未揭露 caster 的另一隊不取得 Ability event。這是顯式安全投影 fixture，不是自動 authority server／KCP／client 全鏈。
- 同 tick 分兩次 dispatcher drain 的兩個成功技能各有不同 producer-local ordinal，visual 亦保留 explicit policy。
- Mana 不足不產生 Ability 或 cooldown。

相鄰變更指定確認：`cast_visual` 2 passed／0 failed／171 filtered out／exit0，包含有／無 Mana 的成功與拒絕、同批失敗不刪既有成功、冷卻重送及缺 handler；缺 handler 另確認 fact buffer 無 Ability。

沒有重跑全套、Unreal、DLL 部署、LAN 或效能驗收。Lua／catalog hash／IPC4／C ABI13 沒改；仍只維護 omfue 方向。

## 防錯 E221

- 首轮指定測試2/2失敗：缺少 ProjectionPolicyRegistry。修正共同初始化資源契約，不在測試或dispatcher塞臨時fallback。
- 初版測試錯把 SimulationDriver 的 headless step 當會自動寫 CommittedProjectionBatch／latest team frames。查清真實程式碼後改讀 SimulationTickResult.facts，再顯式呼叫同一 TeamViewProjector。測試名稱與報告必須標示兩段 fixture，不能冒認正式 network server 自動全鏈。
- PowerShell Skip 誤填 fifty、rg 把 system* 當路徑造成os error123；改整數与 directory＋-g。歷史輸出截斷改小段，不根據截斷內容判斷通過。
- 最後補強 Mana 不足及缺 handler 的 fact 斷言後重新編譯指定 filters，沒有拿補強前結果代替最後版本。

## 後續邊界

- 成功施法的權威來源已補上，與先前下游接線形成可整合程式路徑；尚未重新部署 script／bridge DLL 或完成真實同局全鏈。
- ABY1 仍只含 caster／stable skill ID／epoch／tick，rank／point target／toggle state／陣形參數仍待安全內容契約。共用來源不代表全部視覺參數已支援。
- 完整 Unreal build 的 engine baseline 仍待整合，E220 的限定模組成功不當完整 build 證據。本批未動 engine 或 omfx。
- OpenSpec 仍21/31，4.3／6.1與最後完整驗收保持未完成。
