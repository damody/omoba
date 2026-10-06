# 統一驗收進度（2026-10-06）

## 現況

功能補齊後開始一次統一驗收。不是全部完成；OpenSpec仍21/31。Grok批次job與tokens/成本記.ai-collab/review.md，沒有commit/push或omfx維護。

## 已有實際結果

- 第3批量測限定修正，主agent獨立collector84 checks passed；Rust core formal_perf4、client formal_perf2與server compiled-only check已先通過。
- UE scoped OmRuntime+OmGenerated+OmEditor編譯11 actions succeeded，Saved/Logs/grok-performance-local-build.log；native tests沒有執行。
- codegen --check：17 files / 17 Lua inputs，presentation hash d5553bbb31c459a4。
- compiled-content normal dependency audit4/4通過：server、client runtime、script DLL、UE bridge沒有Lua VM runtime dependency；允許build-time Lua。
- Lua tooling fixture、role launch7、network launch7、launch contract8、shared selection13、event migration13、renderer reconnect observer17、Blueprint compile observer17通過。這些mock／parser／configuration結果不是實際UE或LAN。
- 兩隊observer：scoreboard9、dead scoreboard4、objective ACK5、visibility6、ability9、three-way3、match lifecycle8。Shop observation12、button12、sparse JSON roundtrip；minimap14 assertions通過。
- 作者契約：hero include8、control50、mana34、dash7、numeric34通過。asset recipe7英雄/10jobs與legacy共用動畫去重／五training native fallback測試通過。
- DLL SHA stage verify通過：bridge53cfbe6154e456f95bf70aa3ce57b4ccde2cf095ca923b70d848a3fa2f3a07f9，base ee8900310fa1bb8b95f21c80aa2c71e1ed732add34e28472afbc2c113f053378。只代表該時間點artifact copies一致，之後Bot修正需重新stage。

## 真實失敗，不充數

1. 十Bot正式60Hz seed1，36000ticks/600game seconds仍Playing；100場在第1場停止，completed_matches=0。原始證據omb/target/moba-headless-batches/1791252707-1。Grok第4批run-muw1s4np-6ogt2x / thread8e69a26d-a4a2-4db2-a1f3-a143e6ae14a1處理通用死局根因，沒有withdraw或延長時間。
2. build_ue_moba --full：compiled-only DLL／bridge已build/stage，但完整UBT FailedDueToEngineChange exit4；未start Editor，保留-NoEngineChanges。實際engine BuildId b248c73e-dc09-4ec7-8562-f5170393b1f1，project/OmRuntime/BpGeneratorUltimate都是673c237e-5b5e-41ea-9643-75ad8a45bcac。共享engine現有UBT／Animation／Skeletal／Renderer來源修改保留，不手改ID、不還原或建置其他專案的修改。
3. 真實原生選角test退出，未產生bound service handshake；target/unreal-selection-tests/1791253029-1/match-selection/ue.log列出OmRuntime／OmGenerated／OmEditor／BpGeneratorUltimate模組不相容。已自行退出，未啟動對局。不能當選角程式成功／壞掉證據，先解相容engine/plugin基線。
4. 舊stage fixture只mock _bootstrap且期待bridge-only文字，與目前共用moba_stage_contract不符而失敗。測試改為載入真實模組，用獨立digest檔映射驗六種bridge/base缺失與錯配；6cases通過，沒放寬production。
5. 舊asset recipe fixture把英雄數寫死3而現有正式catalog7英雄。改按recipe一對一、unique ID並明確驗全部五training fallback；10asset jobs與舊動畫共用斷言保持，通過。

## 待完成／外部限制

- Grok死局修正→獨立審查／局部確認→100場正式replay證據。
- 相容engine/project/plugin建置基線解除後，實際native全套／PIE／選角到結算／雙UE重連／四段效能實測與門檻；目前不可跑舊native充數。
- 第二台LAN玩家同局仍需要第二台實際機器；本機雙程序不能代替。
