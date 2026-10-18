# 原計畫剩餘需求核對

## 本次結論

Grok 4.7 Fast read-only job `run-muxvewpm-sk6bxh` / thread `054245e7-f449-4287-8d56-8ce8ea4b0db9`，completed271s、46calls、USD1.87319056。Primary獨立核對production source與執行回歸。本次檢查未找到能在目前環境重現的新增程式缺口；這是有限範圍檢查結果，不是保證整個遊戲沒有錯誤，也不表示四項完整驗收已通過。Grok沒有修改source或執行測試。

## 單份玩家端模擬

正式 `moba_role_launch.lua` renderer使用 `-om-presentation-only` 與 `OM_RUNTIME_MODE=presentation-ipc`；Unreal `ResolveRuntimeMode` 的明確presentation flag優先。`StartRuntimeFromSettings` 設定presentation IPC旗標並移除single-player旗標。Rust `validated_driver_mode` 在分配channel／啟動worker之前選擇Presentation；該分支只有 `om-presentation-ipc` worker、`sim_thread=None`。`omoba-client-runtime/src/main.rs` 在連線後bootstrap一個ReplicaHost，server `omb/src`未查到SelectiveReplicaRuntime。

舊LocalTd分支仍服務TD入口；不依story名稱猜測MOBA、不移除歷史TD能力。正式MOBA與TD入口的區分已存在。

## Primary實際驗證

| 驗證 | 結果 |
|---|---|
| driver_mode_is_explicit_and_validated_before_startup | 1 passed / 0 failed |
| presentation_intent_cannot_fall_back_to_legacy_gameplay | 1 passed / 0 failed |
| ipc_modes_receive_filtered_frame_send_move_and_stop_while_idle | 1 passed / 0 failed；兩種single_player拓樸、sim_thread=None、真loopback IPC fixture、idle stop |
| scripts/test_moba_role_launch.lua | 8/8；native配置、mocked子程序 |
| scripts/test_moba_network_launch.lua | 7/7；native配置、mocked子程序 |

三個Rust命令皆exit0、各執行一個確切測試；同次Cargo另外的integration target因filter為0tests，不計入PASS數。編譯有既有2warnings，不宣稱zero-warning。Rust raw logs在 `.ai-collab/.bridge-data/worker-pool-machine-v1/remaining-*-test-v2.log`。所有測試無authority match。Root／omfue HEAD仍為c7c653b3dcf9664935577f65ff89cd259d92d71c／1f1270d3f4de3f6910a0168b024666547057432c，未commit／push。

## 四項保持未完成

- 4.4：正式MOBA隔離程式與本機IPC測試通過；完整單機與真LAN單份replica尚欠實機證據。
- 6.2：result view只接受MatchPhase=2及合法team/winner；observer要求exact OM_MATCH_RESULT_UI。此前same-PIE v6未自然終局，不能強制winner或偽造marker補PASS。
- 6.4：目前沒有第二台真電腦LAN紀錄，本機loopback不可替代。
- 6.5：固定十二正式效能之前server/client尖峰超標；owned pool修正已功能驗證，但新完整採樣未執行，不調高門檻。

原true-authority批次10/10已耗盡；本次沒有重設、拆批、啟動authority或效能採樣。完整計畫仍27/31。當前已接受新聞停用、exact PIE capture、選角startup與程序退休修正；沒有把局部功能驗證擴大成全場／LAN／效能PASS。
