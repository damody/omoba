# 通用範圍效果 cue

## 計畫與決定

1. 將範圍效果與施法 identity／施法者位移分開。通用 executor 保存已解析的 center／radius，整個目標計畫與額外 Mana 成本通過且效果提交完成後才呼叫 area_cue；合法空範圍同樣成功，拒絕不發布。
2. 使用既有 GameWorld.emit_explosion，不新增 GameWorld 或腳本 ABI。generic executor 使用 1 秒的預設呈現時間，不代表傷害持續時間；其他 Rust handler 可用既有 API 指定呈現時間。
3. 正式 MOBA dispatcher 啟用本次 invocation 的 area capture。adapter 將 emit_explosion 收集成有界結果，成功 gate 後才產生 SkillArea／AbilityArea，明確綁定真正 caster、stable ability ID、等級、來源 team、中心／半徑／時間。
4. 正式施法不再同時發匿名 Outcome::Explosion，避免 legacy converter 把 ordinal 當 source，或重複播放。不在其他 hooks、舊 Story／TD 或 projectile impact 全域啟用 capture，既有未具施法上下文的 Explosion 行為保持。
5. 僅向來源同隊公開 area；safe projector 仍要求 caster 目前可見且有 Disclosed mapping。可見對手也不取得範圍中心／命中目標清單。跨隊可見效果與 projectile impact 另需位置視野契約，尚未完成。

## 資料與呈現

- public APC1 精確48 bytes：magic4／ability ID8／rank4／center x、y各8／radius8／duration8。
- IPC ARC1 精確72 bytes：magic4／tick、caster replica、epoch、ability ID各8／rank4／center x、y各8／radius8／duration8。
- 座標及距離／時間皆 raw Fixed64；共用既有座標上限，radius為正且最多10000單位、duration為正且最多60秒，非法格式／長度／身分／極值 fail closed。
- PresentationCue新增獨立 AbilityArea，維持同一 caster replica／epoch dependency、1024總容量、coalescing、sent後matching connection ACK與reconnect基線。正式 ReplicaHost 與 snapshot adapter 共用 from_public_event／encode，不建立另一個可靠事件佇列。
- bridge查 compiled catalog並填入既有C ABI13 AbilityCast容器，cue_id明確為area、AREA_EFFECT flag，point是center、radius與duration沿既有欄位傳送；target entity仍零。使用既有含歷史命名的欄位不代表新增Saika玩法分支。
- native只在AREA_EFFECT時用WorldUnitsToCm換算半徑，center沿同一world換算與明確presence；area fallback使用Sphere、實際cm半徑與呈現時間，不套用舊PayloadDistanceScale或技能shape。保留catalog色彩與其他事件相容，不新增Blueprint graph。
- 這是debug fallback框架資料與接線，不是Shipping VFX／像素效果完整驗收。腳本ABI、C ABI layout、IPC4不變；新資料需正常建置部署新元件，未知版本安全略過。

## 本輪局部確認

```text
cargo test --manifest-path scripts/Cargo.toml -p base_content --features compiled-content-only area_effects -- --nocapture
cargo test --manifest-path scripts/Cargo.toml -p base_content --features compiled-content-only area_cue -- --nocapture
cargo test --manifest-path omoba-core/Cargo.toml ability_cue -- --nocapture
cargo test --manifest-path omoba-client-runtime/Cargo.toml --features compiled-content-only cue -- --nocapture
cargo test --manifest-path omfue/bridge/Cargo.toml ability_cue -- --nocapture
```

- executor／相鄰正式Bot area2/2：排序、去重、排除非法目標、整體preflight、失敗不cue、合法空區域cue。
- 新60Hz權威來源1/1：真實generated ranger_volley／正式輸入，射程拒絕無Area，合法效果中心與提交結果一致；同隊可見有cue、同隊hidden與可見對手均無，空step不重播。這是headless來源加顯式共用projector，不是自動KCP全鏈。
- core8/8：新ARC1／APC1零中心、距離／時間／座標極值與精確格式；舊cast／relocation identity與政策確認通過。
- client7/7：現有真實localhost IPC保留／覆寫／ACK fixture擴成DMG1、ABY1、ABY2、ABY3、ARC1五種，保留payload完整內容。
- bridge5/5：範圍事件catalog查表、明確flag／center／world radius／duration、舊embedded不偽裝cast；既有admission／lease仍通過。
- cbindgen同步AREA_EFFECT公開flag，native限定三project modules12 actions／Succeeded／exit0，log為omfue/Saved/Logs/area-cue-modules-20261005.log。新增native範圍shape／明確cm／duration斷言已編譯但未執行；沒有重試E224已知engine／project BuildId基線問題。
- 首輪bridge E0004是ScriptVisualEventKind::SkillArea未處理；以Option明確標示舊embedded adapter不支援，不用wildcard或把area當cast，補測試後重新編譯成功。既有warnings保留。
- 首輪OpenSpec strict因插入新Requirement時保留了一個空的重複「完整對局介面」標題失敗；移除空標題，保留原介面本文與scenario後重新確認。
- 未stage DLL／EXE、未做Unreal實際播放／完整對局／LAN／效能驗收。OpenSpec仍21/31，6.1／4.3完整項不勾；本輪只確認當前功能。

錯誤與防錯決策見E225。
