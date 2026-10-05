# 成功施放才發布技能 cue

## 問題與決定

1. 6.1 的事件來源缺陷：dispatch_one 在沉默／冷卻／passive／handler gate 前，把 SkillCast 放進本批 provisional visual vector；run_script_dispatch 原只在 managed Mana 或 panic／資源失敗時回滾 visual。沒有 Mana 的拒絕可能被發布成成功施法提示。
2. 使用既有 adapter.cast_succeeded 作唯一成功判斷；它要求 handler execute 成功且 unit hook 未 panic。所有 SkillCast 若不成功，都回復本次 invocation 的 visual_checkpoint，與 Mana 是否啟用無關。
3. 不直接新增 GenericEffectHandler 的特殊 cue 或在 Unreal 隱藏失敗動畫；Lua 生成技能与特殊 Rust handler 共用 dispatcher gate。保留之前已成功的事件與其他非 cast 事件，不清整批 queue。
4. 沉默／冷卻／passive／缺 handler／execute RErr／panic 都不能靠事件已入列冒充成功。原 gameplay Outcome、Mana transaction、cooldown／權威規則仍走原流程；本批只改最終 script visual 發布，不擴大成 legacy handler 的 Outcome 回滾重寫。

## 局部確認

- 指令：cargo test --manifest-path scripts/Cargo.toml -p base_content --features compiled-content-only cast_visual -- --nocapture。
- 最後版本2/2成功：generated ranger handler在有／無managed Mana時，非法同隊目標沒有cue／CD；同批合法ranger_shot後非法ranger_finisher，只保留前者的caster／target／skill且後者不啟CD；正常Outcome後已有CD，內部重送cast不發布cue。缺handler也沒有cue或CD。
- fixture使用正式Production60Hz建立的World／generated manifest、正式handle_ability_cast_from_input，再直接呼叫真實script dispatcher以便在publication barrier前檢查queue。不是完整driver每tick對局、Unreal或IPC端到端驗收。
- 第一輪2/2通過後補強同批成功→失敗的情境，重新編譯這兩個指定測試確認最後版本；沒有重跑全套。

## 邊界與後續

- 未新增或放寬投影 policy／audience，沒有曝光隱藏來源／target；仍須既有安全投影准入。
- 此為dispatcher事件來源正確性，不是所有成功cast已透過IPC→C ABI→Unreal OnAbilityCue的完整契約，也不是一次性cue可靠傳送／重連／像素驗收。
- 本批不改C ABI／IPC／Lua catalog，不重新stage DLL，不建／驗omfx，不commit／push。
- 6.1與4.3完整項目尚未完成，21/31保持；對局／LAN／效能最後集中驗收。
- 防錯與工具問題記E217。
