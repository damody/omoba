# Unreal renderer 重連：60Hz 增量

## 本輪計畫與決策

1. 沿用 OpenSpec build-unreal-rust-moba-framework；18/30 已完成，先補 4.3 實際 renderer 重連，完整 4.3 仍未勾選。
2. opt-in OMOBA_UE_RECONNECT_SMOKE=1，只允許有界 single_lane 60Hz，不混用 ability／match／shop／minimap smoke。初始階段沿用雙隊 movement／owner HUD／economy／Consumed gate。
3. WM_CLOSE 正常退出 team1 Unreal，10 秒內不退出即失敗，不以強制終止冒充 graceful；離線 2 秒後新 UserDir／stdout 重啟。server、兩 runtime、team2 Unreal PID 必須保持存活且 exe 身分一致。cleanup 仍負責失败收尾。
4. 新 renderer 加公開 minimap Point Move smoke，目標不同於初始 approach；要求真實 callback 原 ID status0、新 session movement、Playing tick 推進、owner economy、fresh Consumed sequence 大於舊 session。
5. HUD 診斷附加 bridge 真實 LockstepStepFps，不猜測 rate；metadata 缺失／0／120 均不通過 60Hz 重連測試。
6. 重啟後 gate 90 秒內完成，原輸入後 120 ticks 以後每隊至少兩個不同 PASS checkpoint；任何 FAIL／repair／三方 hash 不同都拒絕。不是只查一個最後 tick。
7. 原／新 UE logs 不合併，新的 PID 立即寫 active session，保存 raw report 與獨立只讀 acceptance。僅新增共用診斷／Lua workflow，沒有角色 C++ 或 Blueprint graph 變更。

## 已完成的程式

- scripts/run_2player_ue.lua 新 opt-in gate 與 launch descriptor。
- scripts/ue_renderer_reconnect.lua 有界重連／continuity／fresh session 觀察／post-input parity。
- scripts/tests/ue_renderer_reconnect_test.lua：17 個正負向場景，含 rate／owner／ACK ID／tick／Consumed／duplicate checkpoint。
- scripts/tests/ue_renderer_reconnect_acceptance.lua：重新读取保存原始 logs／checkpoint／六個啟動記錄，獨立驗收。
- 共用 OmWorldBridgeActor HUD log 增加 rate，來源是實際 Rust bridge diagnostics，不改玩法。

## 實測狀態

- 完整 build49314 exit0，UBT9.38秒；bridge stage cc76ef2d8328126a7ca255de84fa49daf72272af426ba9a0b40f8ceb8871d709 一致。Editor92496 同 session 兩輪各13/13，串行 PIE78233 exit0。既有警告未隱藏，沒有宣稱警告全修好。
- 診斷 run1791068180 功能通過，但啟動時 Editor 的 QUIT 尚未退出而短暫重疊，不採為正式隔離結果；其 raw logs／compact evidence 保留，原因與正常 WM_CLOSE 修正見 E103。
- 正式 run1791068322 啟動前確認沒有 Unreal Editor；release server／runtime 建置 exit0，120秒初始階段＋獨立90秒重連gate均成功，總流程6019 exit0。原UE42712正常退出，新UE43836；server90208／runtime84812與48392／對側UE75400保持同一PID與exe身分、始終存活。
- 新 session Playing tick4859 大於原可觀察Playing tick866；實際HUD rate60、owner economy與自己的hero位置皆恢復。新 first Consumed4901 大於舊870，runtime保存恰有兩個 session ACK。
- 新小地圖真實 callback renderer input1／status0／tick6511／target(480,288)；runtime allocator從原authority input1進至input2，正式MoveTo raw(491520,294912)精確吻合該公開Point目標。新session呈現位置由(-400,-400)移至(480,288)，不是先前尚未完成的舊移動冒充恢復。
- gate時雙隊hash114／113 PASS rows，最後6840；各有兩個不同tick在原ACK6511+120之後通過，不計UNVERIFIED、不以duplicate row充數。保存acceptance獨立exit0，再次讀取raw新／舊stdout、runtime forwarding／MoveTo、全部checkpoints及六個啟動記錄。
- 六個本輪PID與Editor92496均確認退出；final stage gate再次exit0。完整4.3仍未勾選，整體18/30，這一增量已完成。

正式 compact evidence：openspec/changes/build-unreal-rust-moba-framework/evidence/unreal-renderer-reconnect/interactive-ue-1791068322.json。初始／重啟後stdout分開保存在target/interactive-runs/interactive-ue-1791068322/logs。

## 重跑

以固定 tools/lua/lua.exe 呼叫 scripts/build_ue_moba.lua --full，完成 Editor automation 與 PIE、正常 QUIT 並確認 PID 退出，再設定 OMOBA_UE_RECONNECT_SMOKE=1、OMOBA_UE_SMOKE_SECONDS=120、OMOBA_UE_STEP_FPS=60、OMOBA_RELEASE=1、OMOBA_SKIP_UE_BUILD=1、OMOBA_SKIP_BUILD=0，執行 scripts/run_2player_ue.lua --single-lane。

獨立驗收：tools/lua/lua.exe scripts/tests/ue_renderer_reconnect_acceptance.lua RUN_DIRECTORY。

## 限制

這是同機真實 renderer 正常退出／重啟，不是 Rust runtime crash／斷網恢復、兩台 LAN、全技能 cue 重播或持續 frame-time 60FPS；完整 4.3／6.4／6.5 不勾選。原全對局 60Hz 驗收已在前輪完成，本輪不把省略技能終局的重連測試稱為完整對局。
