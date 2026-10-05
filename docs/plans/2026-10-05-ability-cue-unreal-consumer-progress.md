# 施法 cue Unreal 消費接線（2026-10-05）

## 問題與決策

- 前批 ABY1 已進 IPC，bridge 仍只解 DMG1。將 `PresentationExtras`／pending FrameBuild／published FrameSlot／busy-ring coalescing 泛化成共用 `PresentationCue`，不建立施法專用第二套保留器。總容量及 lifecycle 去除一次性事件維持共用。
- Admission 僅讀前 1024 筆輸入，非法筆數仍占輸入預算；辨識精確版本／長度／event ID／tick，caster 或 target 必須目前 live 且 disclosure epoch 相同。epoch／render ID 超出既有 renderer u32 表示範圍拒絕。
- 穩定技能 hash 不是 Unreal catalog index。抽出共用 `stable_content_fact_id`，保留既有 FNV-1a wire identity，來源 event 與 bridge catalog 共用；載入 catalog 建立 stable hash→文字 ID／numeric catalog ID 查表，拒絕空字串、零 ID、重複 stable ID 與 numeric ID。未知技能不產生 AbilityEvent。
- ABY1 只含 caster／skill，轉為通用 AbilityCast，level＝0（未知）、action instance＝0、target／point／payload／schema 空白；不補猜技能等級、目標座標、toggle 狀態、召喚數量或傷害。文字 ID 保存在 frame lease 自有 string table。
- C ABI 13 沒有欄位／layout 變更；新增 `OM_ABILITY_EVENT_FLAG_CASTER_ONLY` 宣告現有 flags 的一個 bit。已由真正 cbindgen 重產 header，不維護手寫不同版本。
- Unreal 對 caster-only cast 以既有 `FOmPresentationCueHistory` 按 kind／caster／epoch／instance／catalog／tick 去重，frame sequence 不進事件身分；Actor 不存在、future tick 或重複事件不回呼。Stop 清空 history／計數。
- 事件仍走通用 `OnAbilityCue` 與 Lua 生成的 fallback registry。caster-only `cast` 不把未知 toggle state 誤畫成 disabled；沒有新英雄專屬 C++ 分支、Blueprint graph 或執行期 Lua。

## 本批局部確認

- bridge cue 指定 4/4，包括安全 admission、任意技能 registry 與非法身分、施法的 busy lease／coalescing／未知技能／epoch／view reset，以及既有傷害保留。
- bridge damage_payload 指定 2/2，確認共用 decoder 未放寬傷害的輸入預算、去重、live epoch 與精確格式。
- core 共用 hash 相容性指定 1/1：空字串既有 offset basis 與 hello 已知 FNV-1a64 值。
- `cargo check --manifest-path omfue/bridge/Cargo.toml --no-default-features --features compiled-content-only` 成功；新 lookup 的 runtime-driver conditional 不破壞無 driver 編譯。build-time authoring mlua 不是 runtime Lua，未啟用 runtime-lua-content。
- 補強 numeric ID 重複案例後，最後 ability_cue 指定 3/3 成功（catalog／admission／cast frame），不以補強前的 binary 結果冒認最後版本。

## Unreal 建置與證據邊界

- 根目錄誤執行 restart build，尋找不存在的 `D:\code\omoba\om.uproject`，exit2，未進編譯。正確 cwd 為 `D:\code\omoba\omfue`。
- 正式 `restart build --ue-root D:\UE5.8` 保留 `-NoEngineChanges`，於 `ability-cast-build-1791185853.log` 回報 FailedDueToEngineChange／exit4，要求重建 NetCore 及 engine manifests。沒有移除保護、修改或還原使用者 engine sources；Net/Core 指定 source status 無 dirty 輸出，不能憑此推斷所有引擎檔都乾淨。
- 從本機 UnrealBuildTool 的 `Configuration/Descriptors/TargetDescriptor.cs` 確認 `-Module=` 支援 module list。限定 OmRuntime＋OmGenerated＋OmEditor、仍保留 NoEngineChanges，14 actions 成功：`omfue/Saved/Logs/ability-cast-modules-1791186038.log`。
- 此為本批專案模組編譯，不是完整 OmGameEditor build，沒有偽稱 full build 阻擋已消失。
- 首次 NullRHI feature report `AbilityCast-1791186102`：AbilityCastCue、AbilityCueStyle 均 Success／exit0，但前者 1 warning（preview world 無 context 的 actor destroy）。補上真實 world context 與 scope cleanup，不隱藏 warning；最後局部模組3 actions編譯成功，log `ability-cast-modules-1791186227.log`。
- 最後版本 NullRHI report：`omfue/Saved/McpAutomation/AbilityCast-1791186352/index.json`，`Om.Generated.AbilityCastCue` 與 `Om.Generated.AbilityCueStyle` 均 Success、各0 errors／0 warnings，succeeded2／failed0／exit0。確認通用 Actor 回呼、retained cue跨較新frame不重播、不同instance可派發、舊epoch／future tick不派發、Stop清空，以及未知toggle state的cast樣式。不是特效像素確認。
- OpenSpec strict validate、根與omfue whitespace checks通過。header對Git HEAD仍顯示既有ABI12→13與HoldPosition差異，這些為先前工作；本批新增caster-only flag，沒有再升ABI或修改layout，不把整份dirty diff當本批變更。

## 仍待完成

- 完整建置的 engine baseline、重新部署 bridge／script DLL，以及實際權威→KCP→client→IPC→C ABI→Unreal 同局 cue 流程仍待最後整合。NullRHI synthetic callback 是本批接線確認，不是特效像素或完整對局。
- ABY1 目前不揭露目標／等級／toggle 狀態／陣形。現有通用 fallback 可呈現 caster-local cast，必要的正式參數必須先擴充安全權威宣告，不用 renderer 猜測補齊。
- HUD／三路／五位置 Bot／LAN／效能及完整 4.3／6.1 未封關；OpenSpec 仍21/31。本批未維護 omfx，未重新 stage Rust DLL。

## 防錯 E220

- 查找時猜不存在的 RuntimeSubsystem.cpp／Configuration/TargetDescriptor.cs；實際為 OmRuntimeBridgeSubsystem.cpp、Configuration/Descriptors/TargetDescriptor.cs，應先 inventory。
- PowerShell Skip 又誤填 sixty，改整數；無效空 patch hunk `omoba/none` 被拒絕，確認未套用後移除。只有純機械欄位／型別名稱替換使用 bulk rewrite，其餘功能與文件使用 apply_patch。
- 大引擎錯誤清單／編譯 warnings 輸出截斷，改讀保存 log 尾段與精確欄位，不據截斷內容宣稱成功。cbindgen 的 skip warnings、既有 dependency／dead-code warnings 未隱藏。
- 原生 report 的 succeeded 不含 succeededWithWarnings，不能只讀 aggregate succeeded 判斷案例數；逐個 fullTestPath／state／errors／warnings 及 process exit確認。
