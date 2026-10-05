# 通用施法者位移結果 cue

## 計畫與決策

1. 先接入具有現成權威結果的位移技能：從成功 invocation 的 ParallelWorldAdapter.overlay_pos 取得施法者自己提交的 ScriptSetPos 終點。不是 SkillTarget.Point、renderer 目前位置或他人的位移。
2. 成功且位置確實改變才增加可選 metadata；拒絕／panic 仍受原成功 gate 約束。沒有位移結果的技能維持原格式，不新增角色 ID 分支、Lua runtime 或 GameWorld FFI 方法。
3. 事實攜帶 team 與 raw Fixed64 終點。安全 projector 保留可見／Disclosed caster gate，且僅向非零、相同 team 公開終點；對手即使看見 caster 也只有舊有 identity／rank，不取得歷史位移軌跡。
4. 點位是「施法者位移結果」，不是通用敵方 target 或 projectile impact。範圍中心／命中點需要另外的效果端契約與安全規則，尚未完成。

## 格式與接線

- public ABS3：magic4＋ability ID8＋rank4＋x raw8＋y raw8，精確32 bytes。
- presentation ABY3：magic4＋tick8＋caster replica8＋epoch8＋ability ID8＋rank4＋x raw8＋y raw8，精確56 bytes。
- ABY1／ABY2 仍可讀；ABY3 的 rank0 保持未知。座標採 raw Fixed64，絕對值最多 1,000,000×1024；超界的來源可選位置降回既有安全 identity／rank payload，非法 wire point fail closed。
- 共用 PresentationCue／CueRetention 管理唯一 caster replica／epoch dependency、1024 總容量、coalescing、matching connection ACK、Hide／Reset／reconnect 基線；不用第二個位置事件 ledger。
- bridge 明確 /1024 轉 world units，沿 C ABI13 的既有 target_point 搭配 CASTER_RELOCATION flags bit。target_entity 保持零，C ABI layout／script ABI／IPC4 不變。新 binary 必須正常建置部署，舊 consumer 不支援新格式則安全略過。
- native consumer 沿既有 WorldUnitsToCm 換算，FOmAbilityCuePayload.bHasTargetLocation 表達明確存在，通用 fallback 的 ResolveTargetLocation 不再把合法世界原點當缺值。保留舊 legacy 非零點位相容。

## 本輪局部確認

```text
cargo test --manifest-path scripts/Cargo.toml -p base_content --features compiled-content-only dash_effect_formal -- --nocapture
cargo test --manifest-path omoba-core/Cargo.toml ability_cue -- --nocapture
cargo test --manifest-path omoba-client-runtime/Cargo.toml --features compiled-content-only cue -- --nocapture
cargo test --manifest-path omfue/bridge/Cargo.toml ability_cue -- --nocapture
```

- base_content 指定1/1：正常 generated dash／正式60Hz／Bot輸入，阻擋牆與非法射程後合法位移，成功 fact 終點與實際 World 相同，顯式 projector 同隊有點、可見對手無點。
- core7/7：新精確 ABS3／ABY3、合法零點、格式長度／i64極值拒絕，既有 identity／rank／hidden caster／排序仍通過。
- client7/7：現有真實 localhost IPC保留／watch覆寫／ACK測試擴成 DMG1、ABY1、ABY2、ABY3 四種，payload完整保留。
- bridge3/3：ABY3 admission與live epoch、compiled catalog、busy lease／coalescing；target_point=(0,300)、明確flags、rank3，仍無target entity。
- cbindgen 同步公開 flag；既有 skip／dead_code warnings 保留，沒有 suppress。
- 限定 OmRuntime＋OmGenerated＋OmEditor native build：20 actions，exit0／Succeeded。log：omfue/Saved/Logs/caster-relocation-modules-20261005.log。這是 source compile 確認，不是 full OmGameEditor build或遊戲驗收。
- native feature啟動未成功：CasterRelocation-1791188843/editor.log 顯示 project BuildId673c237e-5b5e-41ea-9643-75ad8a45bcac 與 engine 8efe457f-c803-4fca-96e9-fb6466924b49 不同，OmGame 被略過；exit1，沒有 automation report，不宣稱 native test 通過。
- 帶 OmGame 的四模組與 -NoUBTMakefiles 重產計畫均0 actions／Succeeded，但BuildId仍未對齊，因此不再無效重試。保留 NoEngineChanges，不手改 manifest ID、不偽造相容；完整 engine／project baseline 對齊留部署整合。
- 本輪沒有 stage Rust DLL／EXE、完整對局／LAN／效能／像素驗收。native新增零點與指定終點斷言已編譯，尚未執行。OpenSpec 21/31 不變，6.1／4.3完整項仍待完成。

## 錯誤紀錄

首輪 E0308 因 Faction.team_id 是 i32、公開 team 是 u32；改 checked conversion 並拒絕零／負team，不使用 as cast。另有猜測不存在的 team_identity.rs／OmGeneratedTests.cpp／OmPresentationTypes.h，改 inventory／directory search 定位。詳細見 E224。
