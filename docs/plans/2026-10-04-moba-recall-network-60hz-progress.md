# 回城正式網路與 Unreal 入口進度（2026-10-04）

## 決定與範圍

- 使用既有 OpenSpec `build-unreal-rust-moba-framework`（19/30），繼續4.1／5.3／6.2的回城增量，不勾選完整項目。
- Recall獨立protocol1／完整CONTENT_CATALOG_DATA_HASH；legacy0/empty關閉，版本／hash不符拒絕。server只對authenticated selective player＋SingleLane開放，shop能力不能解鎖回城。
- JoinRequest新增12/13、TeamGameStart新增25/26；PlayerInput仍tag19，RendererInput RecallIntent使用空message／tag18（17為AttackTarget）。無entity target、輸入ID必須非零；runtime仍檢查owner／disclosure／自己hero。
- 權威開始回城清除舊移動／attack queue；filtered不用accepted Recall猜測成立，而依owner-team PreStep active HUD fact清除命令。PostStep remaining在傷害／死亡／基地傳送／終局之後發布；不把APPLIED ACK當成回城完成。
- Unreal通用B鍵→reflection SubmitOwnedRecall→generic gameplay event→C ABI InputRecall→runtime intent；不寫角色專屬C++／Blueprint graph。HUD顯示協商能力與權威倒數。新增C ABI欄位，版本由6升7，正常建置由cbindgen再生成header。

## 驗證

- base_content79 passed：含Production60Hz完整對局與525tick雙隊移動→Recall accepted-input→基地傳送，每tick與fresh bootstrap hash一致、零ComponentRepair、回城metric／accepted input只給自己的隊伍。
- core324 passed；server153 passed／1 ignored；runtime54＋3 passed／3 opt-in ignored；bridge50 passed／1 opt-in ignored。Fyrox `cargo check --manifest-path omfx/Cargo.toml -p omfx --tests`通過，保留舊前端。
- 真實KCP run `moba-runtime-1791072423` success=true／tick_rate_hz60／cleanup_verified=true；team1 eleven、team2 ten三方checkpoints，雙隊最後verified tick1320。server96024／runtime64900、73740由Lua launcher清理。
- 同run：team1 Recall authority input2 target486、active487 remaining8175、完成966，坐標(0,0)；team2 input2 target531、active532 remaining8175、完成1011，坐標(2457600,0)，等於公開單路geometry的自己基地。先有正常移動input1，避免只测靜止英雄。
- 此KCP smoke用runtime內部renderer-intent injection進入正式InputBridge／KCP，不是真實Unreal B鍵／renderer TCP送入；報告input_route已修正為runtime-renderer-intent-injection。保存原run raw報告不事後覆寫。
- Unreal build-only82551、full63411 exit0；stage SHA256 `382dea09e779aacf310f87238b1fd82921b6d6ba3c260b28948ccb99d17d903d`一致，Editor48060／MCP HTTP30000，11 Blueprint compile gate通過。codegen --check11檔／15Luainputs，class hash de9c7fcfc98d6479；不是fullruleshash。
- 同Editor native visual95880 exit0，兩輪各13/13，新增reflection Recall／HUD欄位assert也通過。串行PIE59543 exit0，正常stop確認；SaveAll MCP成功後對owned Editor48060送WM_CLOSE，wait確認正常退出。兩輪KCP共六個已知PID也再次inspect確認不存在。
- OpenSpec strict與scoped diff --check通過；現有CRLF／dependency警告未隱藏。

## 失敗與防重犯

- run1791072213保留failure：team1完成，team2受干擾後讀條取消。測試器曾把正常取消當Ipc error，導致ordered frame反覆rebase；已改記canceled stage255／由外部launcher判定失败，不影響正常simulation。完成fixture正式移到自己基地外側／離開兵線後重驗通過，不改遊戲傷害規則。
- 詳見 `unreal-moba-error-register.md` E107；不刪失敗證據，不把單元測試或compiler成功當作實際B鍵對局封關。

## 仍未完成

- 實際雙Unreal B-key callback／權威ACK／倒數畫面／取消與成功傳送驗收（目前B鍵與HUD已編譯、reflection及Editor回歸通過）。
- 所有recall取消種類的實際KCP／重連讀條完整證據，助攻／完整商店UI／完整框架其他未勾選工作，兩台LAN與frame-time基線。
- 回城目前只傳送，不提供未定義的免費治療；Lua為8秒，不提高至120Hz。
