# v6 自然結算逾時的離線診斷

## 計畫與結果

用既有v6公開隊伍證據，區分尚未終局與終局未呈現；只有確切source defect才修改規則或UI。Grok4.7Fast read-only job `run-muxvuirc-1qpeq9`／thread `6214733c-1e25-4ee6-aafb-f2515c573fd9` completed498s／41calls／USD1.7097308。未新增對局、修改source、建置或重跑Unreal。Primary独立讀取正式schema／projector／終局分支與解碼capture。

**晚期權威公開phase仍是Playing，沒有證據顯示已Finished卻被UI漏掉。** 原v6report保持success=false、results空、cleanup_verified=true；不是完整6.2驗收成功。沒有以改大timeout、強制winner、假marker、修改bot／塔規則代替驗收。

## Primary獨立證據

資料 `target/role-ue-runs/machine-same-pie-continuation-v6-dx12/evidence/team-1-runtime/team-frame.capture`，548495672 bytes。只讀自己隊伍公開capture，不讀秘密sentinel或私密資料。

依 `proto/game.proto` 的TeamTickFrame field6 replica_tick、field11 Step；Step field2 TeamPublicEvent；event_kind11是Hud，payload為projector正式little-endian u32 team／u64 metric／i64 value。match phase metric `0x4d4f424150483031`、winner metric `0x4d4f4241574e3031`由moba_match.rs正式產生。

Primary離線Lua auditor從capture首筆沿4byte big-endian長度走完整record boundary，僅解碼最後12MiB內完整frame；每個長度限制16MiB、拒絕truncated header/frame/field、HUD payload必須20bytes、team必須1、match audience0。實際exit0：

- 全檔57422 records；最後1315筆，tick56108到57422。
- phase1共1315，phase2不存在；winner0共1315。
- filtered-timeline最後replica_tick57422；server末尾仍於tick57432提交bot動作。
- Editor只記phase1／winner0／tick647，沒有OM_MATCH_RESULT_UI；晚期economy tick57182後PIE teardown。

`moba_match.rs`對Finished走early return、不再推進gameplay，`BuildMatchResultView`只在合法phase2下顯示。這些與晚期Playing及持续bot輸入一致。捕捉的是已公開權威frame，不能把缺少的runtime三方hash擴充成完整三方hash驗收。

## Grok補充與限制

配置three_lane_training／60Hz、player1真人training_apprentice且無輸入、其餘9bots。Grok從公開snapshot診斷：team1基地仍滿血且鎖定、底路塔仍存在，其他塔持續被摧毀；human在出生點。這些沒有獨立重建成整場gameplay replay，不當作新完整驗收。

20分鐘wall-clock觀測並非保證20分鐘simulation：HUD末期tick57182約953秒模擬時間，進度仍持續。不能證明底路塔後期HP未再下降的原因、不可見敵基地HP、或延長觀測必然終局。未找到此場能證明的程式缺陷，因此沒有不必要source patch。

原計畫仍27/31、authority batch10/10保持；需要新自然終局實機證據才能勾6.2，真雙機LAN與固定十二效能也未完成。當前正式本機71native／69Blueprint及選角startup/retirement已驗收，見current-full-local-acceptance MD。Root與omfue HEAD保持，未commit／push。

## 工具錯誤防重犯

初次inventory過廣列出千筆evidence路徑而截斷；改成根部exact log及指定team公開檔。components JSON根是array，Primary起初當object選phase欄位得到null且無有效診斷；改先確認schema/root shape，只讀第一record，最終用正式frame schema解碼，不把null當phase或winner。重複Windows rg literal wildcard／猜不存在source檔名問題仍適用E358；後續沿已讀mod.rs及proto正式type名稱定位，不猜TeamEncodedFrame（正式名稱TeamTickFrame）。這些錯誤均未修改原始evidence，也未作為功能FAIL證据。
