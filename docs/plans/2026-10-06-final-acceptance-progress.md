# 統一驗收進度（2026-10-06）

## 最新增量：25/31（以下舊快照保留為歷史）

另完成Grok fresh dispatcher reconfigure正確性修復與primary獨立1test：換pool後退役舊cached dispatcher；正式game未呼叫該API，不當效能根因。該increment无新的release/UE/場次，見dispatcher-reconfigure-progress。總體仍25/31。

本批最終功能結果：strict 原始Child/lifetime native stop／close、session publication/cleanup/reconnect身份接線已實作；Rust1/Lua7/真本次fixture确认。一次真60Hz短程雙UE `replica-stage-20261006-lifetime-v1` movement/HUD/consumed與五owned lifetime退出獨立核對通過；分段各59窗口／3540samples，tick227峰351.1038／475.328ms幾乎全在fixed_step。非完整自然終局或全驗收，50ms依然失敗，不勾6.5。Grok retention工作接受，lifetime後續9m35s無diff由primary收回／完成；cost未知部分不估。詳session-lifetime-progress／E323。

本批 Grok report 無界 retention 移除已獨立審查／runtime1/check接受，primary 分段唯讀收集器11/11。固定50ms門檻不變，未實機採樣、未重新完整驗收。發現launcher PID-only cleanup安全缺口，先委派strict lifetime修正，不帶風險重跑遊戲。詳replica-stage-collector-progress／E323；下方前批證據保留。

6.1 已以保存原生42x2／PIE真render及fallback證據核對封關，不重跑完整驗收；詳presentation-functional-closure。單人/shared選角新增共用completion budget與machine uptime clock，pure/Rust1/單人7/shared14功能確認；結算append-reader11/11確認，不是完整選角到結算。Grok run-muw9jh35-ydrn8m 已完成有界replica分段診斷，主agent審查並獨立core6/runtime1/compiled-only check通過後接受；只診斷，沒有證明效能已修復。剩4.1、4.3、4.4、6.2、6.4、6.5；LAN缺第二實機，client max原50ms門檻保持。本批未跑模擬、未commit/push/engine/omfx變更。

## 現況

最新實際完成24/31：2.2b已以codegen17/17、原生同Editor兩輪42/42及PIE原英雄render/記憶marker清除完成；Blueprint11/11與真實Unreal選角select/lock/finalize三native回呼→Rust final plan成功（target/unreal-selection-tests/1791263616-1），不等於選角到終局。正式60Hz release renderer重連與量測目前執行中；歷史engine阻塞已解除，不假勾其餘7項。

2026-10-06後續基線更新：engine現在1323cea4-7408-4662-8321-6abdaf191604，正式OmGame建置與debug DLL部署成功、正常UBT更新三manifest，Editor啟動已成功。MCP固定30000誤連其他專案已改project-bound registry＋OS port-owner/lifetime檢查，Blueprint11/11成功；原生全套首次暴露未Initialize的UMG fixture崩潰，兩处同因修復中。詳細見project-bound-unreal-progress／E313–E315。下文「UE建置無法進行」是舊基線，不再是當前結論；23/31仍不冒進。

**最終headless完成，23/31；不是全框架驗收完成。** 四組seed1/26/51/76各25場全部exit0，合計100/100。主agent固定Lua逐100份report核對M.verify、unique seed1..100、正式three_lane_training map、每tick digest長度/末digest、side0/1→team1/2映射及batch摘要全欄位/檔案路徑，exit0：6004657 replay ticks；finish_tick最小27202/最大84070；team1勝65、team2勝35。全部60Hz、10Bot、guard、自然Finished/end1、有combat、max1800/stall300，沒有budget/stall失敗。不將明示1800預算冒稱預設600成功，也不將headless安全矩陣冒稱LAN認證或增加不存在的拒絕counter。

四份role-plan SHA仍078298f0869930120080c7c2f6dc2867fb2d2a29b14fe39e8324017ed3ee0b5e，release DLL仍c68e46df1a4d5f59ab71464fd688f691a5e017a98cf21f0825af5746235c36c6，host仍273f5d35ac0f5feb97a584d6859eff23b0494575341568b228c6bf9e2a8d26cd；三repo HEAD仍c5211e83/omb578f475/omfue1e29a27。role配方、artifact與生產程式在對局期間沒有改動或重建。完整JSON/log/batch-report留在上列四個final-20261006-114908-seed目錄；不納入git。

額外layered十Bot seed101自然Finished152648ticks、team2勝、逐tick replay/末digest及明示max3600/stall300独立核對通過，與既有地形/導航/解鎖回歸共同完成5.4；100場正式Bot/原型與安全回歸完成5.5。此兩項勾選不代表最新完整filtered/Unreal呈現驗收。先前0/18/49場快照及各失敗記錄以下均屬歷史。

剩餘8項：2.2b、4.1、4.3、4.4、6.1、6.2、6.4、6.5。實作/限定編譯及Rust/Lua確認已做，不再重跑相同完整UBT失敗。真實UE因engine/project/precompiled plugin BuildId不相容與NoEngineChanges阻塞；原zip BuildId也不相容，不能手改ID/安全閘門或拿舊native結果補數。需相容且可啟動的engine/plugin基線才能native/PIE/完整UI/重連/四段正式效能實測與門檻；兩台LAN另缺第二台實機。新release Bot DLL尚未重新完成UE統一build/stage，先前stage結果只代表當時artifact。不宣稱可正常啟動最新Unreal。無commit/push、無omfx維護。

最新執行快照：正式seed1..100四組批次已確認49/100，全部仍運行，尚無失敗；完成數只作進度，待四組退出及逐份完整核對才勾任務。額外layered正式十Bot seed101於明示max3600/stall300自然終局152648ticks、winner_team2，完整152648 tick replay及末digest由主agent獨立核對通過，詳E312；舊兩份1800逾時保留，不混入100場。

最新增量：第5批seed1已在1800/300明示操作預算自然結束、replay62951ticks，沒有修改原600秒預設或遊戲規則。Codex發現winner報告side/team混淆及entity0存活監測錯誤，run-muw4qdb8-ppm5he限定修復中；舊result保留不改。目前完整批次仍0/100，局部fixture七個失敗均已精準修復，真實UE/跨機LAN仍受外部基線限制。下文失敗是歷史證據，不代表最新seed1仍Playing。

限定修復已completed並被主agent接受：實際source／tests／HEAD／whitespace核對與獨立Rust7/Lua10通過，winner正式team映射與id0/gen存活／保存歷史已修。release DLL刷新後進四組25場新版正式batch，未先計成功。cost/job metrics見.ai-collab/review.md。

新版正式batch已啟動：omb/target/moba-headless-batches/final-20261006-114908-seed{1,26,51,76}，每組25場，明示max1800/stall300。四份role-plan SHA均078298f0869930120080c7c2f6dc2867fb2d2a29b14fe39e8324017ed3ee0b5e；執行前release base DLL c68e46df1a4d5f59ab71464fd688f691a5e017a98cf21f0825af5746235c36c6、host273f5d35ac0f5feb97a584d6859eff23b0494575341568b228c6bf9e2a8d26cd。不編輯production Rust／authoring／Lua verify或stage artifact直到批次結束。CLI四組全部完成且逐report重新核對才算100場，起跑不勾任務。

初始18/100已完成，前4份report由主agent固定Lua獨立verify預算／60Hz／10Bot／guard／replay與tick_digests長度、末digest；seed1=62951ticks/winner team1，digest仍80eede71...，其他seed26/51/76也自然Finished。舊winner0 report保留不納入。

額外分層地圖legacy雙英雄seed101在108000ticks budget timeout，last_progress103699，證據留final-20261006-114908-layered-seed101（E312），不算正式五位置100場。另新增正式可重用Lua recipe moba_layered_archetype_match，僅map_id不同已獨立確認，沿同frozen binary/DLL匯出並運行final-20261006-114908-layered-role-seed101；不是改difficulty或勝負規則。新recipe不改100場既有輸入，也沒有在已載入DLL期間再Cargo重建。

功能補齊後開始一次統一驗收。不是全部完成；OpenSpec仍21/31。Grok批次job與tokens/成本記.ai-collab/review.md，沒有commit/push或omfx維護。

## 已有實際結果

- 統一Rust library初次core473/474、base206/210；五個失敗case由主agent明確cfg(test)補必要resource／按ranger_patch authored回魔效果修正fixture，原功能/hash/rejection斷言保留。精準core1、mana1、dart5、ice1成功。没有在新結果後重跑全套、沒有宣稱初次全綠；錯誤E308。
- client runtime compiled-only library78通過／8略過／0失敗。Bridge初次82通過／1失敗／1略過，舊ABI11 fixture改共用OM_ABI_VERSION且精準1通過（E309）。Server library167通過／1失敗／1略過，merge fixture改核對formal Bot完整輸入而非凍結Support策略，精準1通過（E310）。略過項不算成功，也不是LAN／實際UE驗收。

- 第3批量測限定修正，主agent獨立collector84 checks passed；Rust core formal_perf4、client formal_perf2與server compiled-only check已先通過。
- UE scoped OmRuntime+OmGenerated+OmEditor編譯11 actions succeeded，Saved/Logs/grok-performance-local-build.log；native tests沒有執行。
- codegen --check：17 files / 17 Lua inputs，presentation hash d5553bbb31c459a4。
- compiled-content normal dependency audit4/4通過：server、client runtime、script DLL、UE bridge沒有Lua VM runtime dependency；允許build-time Lua。
- Lua tooling fixture、role launch7、network launch7、launch contract8、shared selection13、event migration13、renderer reconnect observer17、Blueprint compile observer17通過。這些mock／parser／configuration結果不是實際UE或LAN。
- 兩隊observer：scoreboard9、dead scoreboard4、objective ACK5、visibility6、ability9、three-way3、match lifecycle8。Shop observation12、button12、sparse JSON roundtrip；minimap14 assertions通過。
- 作者契約：hero include8、control50、mana34、dash7、numeric34通過。asset recipe7英雄/10jobs與legacy共用動畫去重／五training native fallback測試通過。
- DLL SHA stage verify通過：bridge53cfbe6154e456f95bf70aa3ce57b4ccde2cf095ca923b70d848a3fa2f3a07f9，base ee8900310fa1bb8b95f21c80aa2c71e1ed732add34e28472afbc2c113f053378。只代表該時間點artifact copies一致，之後Bot修正需重新stage。

## 真實失敗，不充數

第4批續作run-muw29l3a-g23f3i取消（43m6s），由主agent決定，不是使用者中止。partial策略未接受。attempt2/3仍36000ticks timeout；attempt3在32400→36000基地總HP raw3276800→2902016，客觀仍有進展，不足以單由600秒判定死局。第5批run-muw3v0yp-57zms7新增明示預算／真實objective-stall診斷；預設600、自然結束/replay保持，不強制結果、不改玩法。不覆蓋任何report的缺口也納入本批。

第4批續作已真正重現seed1；omb/target/moba-headless-repair-seed1/report.json是success=false診斷，不是驗收結果。主agent只讀核對tick36000、Playing、75waves：兩隊中路當前塔均已退休，雙基地仍滿血；九英雄存活，若干Carry／Support為Hold、Top／Jungle回基地。這些是觀測，不先宣稱根因。Grok續查通用策略；歷史failure-samples.json保留，不作刪除清理。

第4批初次策略修正後仍timeout；新的report.failure-samples.json記錄36000ticks/Playing/75waves，上路與中路兩隊當前塔皆退休，下路兩隊塔仍在（HP raw1100802／308848）、兩基地仍滿血。主agent只讀核對新配對診斷，不誤拿10:31舊report當10:48新結果；局部改善不是終局成功，100場仍0/100。後續修正待結果。

1. 十Bot正式60Hz seed1，36000ticks/600game seconds仍Playing；100場在第1場停止，completed_matches=0。原始證據omb/target/moba-headless-batches/1791252707-1。Grok第4批run-muw1s4np-6ogt2x / thread8e69a26d-a4a2-4db2-a1f3-a143e6ae14a1處理通用死局根因，沒有withdraw或延長時間。
2. build_ue_moba --full：compiled-only DLL／bridge已build/stage，但完整UBT FailedDueToEngineChange exit4；未start Editor，保留-NoEngineChanges。實際engine BuildId b248c73e-dc09-4ec7-8562-f5170393b1f1，project/OmRuntime/BpGeneratorUltimate都是673c237e-5b5e-41ea-9643-75ad8a45bcac。共享engine現有UBT／Animation／Skeletal／Renderer來源修改保留，不手改ID、不還原或建置其他專案的修改。
3. 真實原生選角test退出，未產生bound service handshake；target/unreal-selection-tests/1791253029-1/match-selection/ue.log列出OmRuntime／OmGenerated／OmEditor／BpGeneratorUltimate模組不相容。已自行退出，未啟動對局。不能當選角程式成功／壞掉證據，先解相容engine/plugin基線。
4. 舊stage fixture只mock _bootstrap且期待bridge-only文字，與目前共用moba_stage_contract不符而失敗。測試改為載入真實模組，用獨立digest檔映射驗六種bridge/base缺失與錯配；6cases通過，沒放寬production。
5. 舊asset recipe fixture把英雄數寫死3而現有正式catalog7英雄。改按recipe一對一、unique ID並明確驗全部五training fallback；10asset jobs與舊動畫共用斷言保持，通過。

## 待完成／外部限制

唯讀核對已提交UECP-Windows-UE5.8-v2.0.6.zip的UnrealEditor.modules，原始BuildId為55116800；並非目前source engine的b248c73e或project的673c237e，因此不能把重新解壓zip當成相容性修正。未抽換插件、未手改modules。上一輪YieldBehind獨立core1及正常generated base60Hz1局部測試成功；不等於對局終局成功。

- 第4批在外部commit改HEAD時停止，只新增diagnostic／release build，沒有run或策略修正。主agent已核對new name-only／submodule stat為先前既有改動與zip收錄，core Bot未變；保留root c5211e83／omfue1e29a27／omb578f475。沿同thread續run-muw29l3a-g23f3i，不從零重做。診斷固定檔名與成功刪除舊證據已列具體修正，禁止覆蓋或刪除歷史。

- Grok死局修正→獨立審查／局部確認→100場正式replay證據。
- 相容engine/project/plugin建置基線解除後，實際native全套／PIE／選角到結算／雙UE重連／四段效能實測與門檻；目前不可跑舊native充數。
- 第二台LAN玩家同局仍需要第二台實際機器；本機雙程序不能代替。
