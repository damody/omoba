# 權威迷霧 rebase 恢復

## 本批計畫與結果

- [x] 檢查既有恢復：coordinator通常使用epoch1，manifest只有玩法world，runtime恢復後清空fog；不能宣稱已會恢復探索。
- [x] TeamViewRebase新增optional fog_grid；有grid使用manifest v2、獨立hash domain並納入完整typed grid。無grid保留v1與既有hash bytes；v1夾帶grid、v2缺grid、未知版本均拒絕。
- [x] AuthorityFogGrid新增recovery epoch操作，保留同隊／同場探索；reset_view仍是明確清空的新view操作，不混淆兩者。
- [x] projector在rebase前取樣committed view；bundle成功後同步active view epoch與fog。後續frame／TeamGameStart使用同一epoch，舊epoch請求不得倒退。
- [x] runtime先驗manifest fog，再完成chunk hash及玩法baseline恢復；成功後換成已驗證fog retention，不等待下一個fog取樣。
- [x] verified rebase要求snapshot ID／tick／epoch與manifest一致；不接受拼接不同baseline／呈現資料。
- [x] 正常與catch-up主迴圈共用恢復發布：critical FIFO reset→完整恢復快照，同一快照另保留為latest供renderer重連，不啟動第二份玩法模擬。
- [x] 新增兩個直接相關rebase測試，既有兩個rebase tick／sequence測試亦成功；runtime cargo check成功。
- [x] runtime實際建置與server schema編譯確認。

## 決定與限制

恢復不同於新對局或改隊；同隊已探索資料不因恢復任意清空。manifest grid綁定team／epoch／sample tick／geometry及三態，由版本化hash驗完整內容；仍不包含canonical source或隱藏實體身分，不改gameplay hash。版本1不允許未納入hash的grid，舊驗證器也無法通過版本2新的hash domain。

authority coordinator目前預設epoch1，這次不強迫每次rebase更換身分。明確要求更高epoch時projector同步推進；production請求epoch下限以active epoch夾住，不倒退。探索保留仍受相同隊伍限制；真正的新view reset維持clear語意。

core tests經真正projector、chunk staging、SelectiveReplicaRuntime與NoopDisclosedWorldStepper，確認epoch5／explored恢復、後續6個frame可套用、新取樣與重新加入bootstrap一致、hash tamper拒絕、錯snapshot epoch不改原world、v1相容及v2 audience／shape驗證。不是雙UE／KCP完整恢復驗收，也不是Specs script全對局測試。

## 成功確認

`cargo test --manifest-path omoba-core/Cargo.toml --lib authority_fog_rebase`：2 passed，351 filtered out。

`cargo test --manifest-path omoba-core/Cargo.toml --lib filtered_rebase`：2 passed，351 filtered out。

`cargo check --manifest-path omoba-client-runtime/Cargo.toml`：成功（含兩條main恢復分支）。未修改C ABI11／Unreal程式，未重跑MCP／PIE／雙UE／全套回歸；最後整合驗收仍待完成。20/30維持，不勾選4.3／6.1全項。

## 錯誤紀錄

runtime cargo build與初次`cargo check --manifest-path omb/Cargo.toml -p omobab`皆exit0，但收尾確認受版控proto fallback缺少新fog_grid：core即時生成與server fallback不同，不能把第一次編譯成功視為schema已同步。使用既有OMOBA_UPDATE_PROTO_FALLBACK=1更新generated/game.rs後重新確認，見E144；未安裝工具或新增fallback。增加receiver epoch單調驗證，拒絕verified但較舊epoch的manifest，避免回復後再回退active view。bridge／DLL staging留最後整合，沒有為純Rust/proto變更重啟Editor。

初次PowerShell讀取誤將Select-Object -First寫成sixty，並重複了一次相同錯誤；已改數字65後取得實際區段，不把錯誤輸出當讀取成功。E143記錄此操作錯誤及恢復資料／發布順序決策。

最後schema收尾確認：既有生成選項exit0，受版控fallback含FogGridPresentation及snapshot／rebase兩個fog_grid欄位；server再次check exit0。新rebase測試加入verified舊epoch不得倒退，仍2/2 passed；runtime最終build exit0。所有本批修改檔diff check通過。
