# 當前版本正式本機完整驗收

Grok 4.7 Fast執行既有 `ue_local_acceptance.lua --ue-root D:/UE5.8 --run-id machine-current-final-v2`，新的證據目錄、Lua程序exit0。Job `run-muxvnpnl-j4gd2z`，thread `a00f2792-45bb-4591-969f-55d23e250828`，completed176s／13calls／USD0.26761128。沒有source修改、既有RustBP資產diff、commit或push；沒有新authority對局。

Primary獨立重新讀取expected清單、原native JSON與Blueprint log，使用已通過26個negative/positive cases的contract auditor核對summary，不僅接受Grok或程序exit0的結論。

| 正式驗收 | 結果 |
|---|---|
| Native exact test names | 71全Success；64無warning、7有warning；共13warnings；failed0／notRun0／inProcess0／test errors0 |
| 既有RustBP exact asset names | 69／69；compiler errors0／warnings0／failed loads0 |
| Native commandlet | exit0 |
| CompileAllBlueprints commandlet | exit0 |
| Primary獨立report auditor | exit0、success=true，原report與summary一致 |
| 本輪commandlet程序 | exact run-id查詢remaining0 |

Native warnings的test為CompiledContentReloadPolicy1、FrameContract1、FrameGeometryRanges1、RememberedGhostPresentation1、RuntimePresentationReset4、MinimapMemory2、NativeMinimap3。保留真warning，不改assert／門檻。13warnings不等於Blueprint compiler warnings，後者確實0。

Blueprint engine log另有vendor sound_gen_config遠端HTTP讀取15秒逾時warning，commandlet仍完成、exit0。這不是Blueprint compiler warning，也不是整個engine log零warning；錯誤register E359記錄，未重試／改cache／授權或遠端設定。

證據：`omfue/Saved/Automation/LocalAcceptance/machine-current-final-v2/summary.json`、`expected.json`、`native-report/index.json`、`native-engine.log`、`blueprint-engine.log`及原stdout/stderr；沒有errors.md。Root HEAD c7c653b3dcf9664935577f65ff89cd259d92d71c、omfue HEAD1f1270d3f4de3f6910a0168b024666547057432c保持。

當前本機功能與既有資產接受。先前selection-only真實150453ms／native callbacks3／cleanup true保持有效。本次commandlet驗收不是自然整場、真LAN、固定十二效能或vendor protected telemetry的替代；整體27/31、true-authority batch10/10不重設。相關剩餘契約見 `2026-10-07-remaining-contract-audit.md`，錯誤與工具教訓維持error register。
