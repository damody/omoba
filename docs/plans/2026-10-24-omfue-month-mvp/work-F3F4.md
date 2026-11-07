# F3／F4：美術槽位統一與 Editor-only 雙 rig 替換

狀態：已實作並以 3-way merge 著陸到 `omfue/`（未 commit）。這台機器不能編譯 UE，所以 Editor 自動化測試要由主 session 建置後執行。

美術操作手冊與槽位規格見 [unreal-art-guide.md](unreal-art-guide.md)。

## F3 盤點結果與通用槽位

- **動畫**：`AOmAnimatedUnitActor` 原本就已用 `NativeAnimations`（槽位 → binding）、`NativeAnimationStateSlots` 和 FallbackPolicy 涵蓋 idle／move／attack／critical／cast／channel／hit／death／respawn／recall，codegen 也會把 Lua 的 `ue.native_visual.animations` 產生成同一組欄位。這部分不缺槽位，只需要補文件。
- **音效**：`UOmAudioPresentationComponent::PlayPublicCue` 原本只會播放 `/Game/Omoba/Audio/OriginalV1` 中寫死的 wave，美術無法按單位或技能替換。
- **特效**：攻擊、受擊、死亡等只有程式內建的佔位特效（`Visuals->Emit`），沒有可在 Editor 填資產的入口。

新增內容：

- `AOmUnitActor::PresentationCueSlots`（`TMap<FName, FOmPresentationCueSlot>`）。每個槽位可填 Sound、Effect（Niagara 或 Cascade）、AttachPoint、Offset、bReplaceOriginalVfx。
  - 通用 cue 共 11 個：`attack_windup`、`attack_impact`、`critical_hit`、`attack_avoided`、`cast`、`hit`、`projectile_impact`、`death`、`respawn`、`healing`、`move_start`。
  - 另有按技能的 `ability.<id>`，優先於 `cast`。
  - 名稱集中在 `OmPresentationCue` namespace。
- 觸發點：OnAttackPhase、OnAbilityCue、OnDamageCue、OnAttackAvoidedCue、OnCriticalHitCue、OnProjectileImpactCue，以及死亡與重生的戰鬥幀同步。
  - 以 (cue, instance, tick) 去重，快照重複送達時不會重播。
  - 單位隱藏時不生成特效。
  - 有 Effect 時取代原本的佔位特效。
- `PlayPublicCue` 會先查單位的槽位音效，找不到才用原始 wave。
  - 原始 cue 與槽位的對應由 `OmPresentationCue::FromOriginalAudioCue` 處理。
  - 為了不洩漏位置，仍然是非空間化播放；也仍受 `bUseOriginalPresentationAudio` 控制。
- OmRuntime 加上 Niagara 依賴（uplugin 與 Build.cs）。

## F4 Editor-only 替換入口

- 新增 `UOmRuntimeSettings::UnitClassOverrides`（Project Settings → Om Runtime → Art）：內容 id 對應到 Blueprint class。
  - `AOmWorldBridgeActor::ResolveUnitClass` 會先查這張表，再查內容的 BP 路徑。所以 `native_only` 的單位（分路小兵、訓練英雄）也能只在 Editor 換美術。
  - class 無效或是 abstract 時會發出警告，並退回原生 class。
- 面向：沿用 E040／E057 的 `bAutoAlignMeshForward`。勾選時忽略 yaw；取消勾選時使用 `NativeMeshTransform` 的明確 yaw。Lua 有 `yaw_deg` 時，codegen 會預設關閉自動對齊。

## 測試（`OmEditor/Private/OmArtSlotTests.cpp`）

- `Om.Runtime.PresentationCueSlots`
  - 槽位名稱與音效對應。
  - 沒有槽位時保留原始行為。
  - ability 槽位優先於通用槽位。
  - 特效只生成一次並掛在 mesh 上；instance 改變時重新生成。
  - 有 Effect 時取代原始特效；隱藏的單位不生成特效。
- `Om.Editor.ArtSwapTwoRigs`
  - 雙向替換：Quaternius `female_peasant`（lane caster）↔ `saika_magoichi` 的 Warcraft-style rig。
  - 用 `FKismetEditorUtilities::CreateBlueprint` 建立 Blueprint，只複製 Details 欄位，再設定 scale 1.3、高度 95。
  - 驗證 mesh、scale、材質、yaw 0 與 120 時的正面對齊、walk 播放替換的 move binding、正規化後高度 ≈ 95、名牌在 mesh 上方。
  - 驗證 Project Settings 入口能解析 override，移除後回到原生 class。

## 已知限制

- 槽位音效仍是非空間化播放，這是刻意設計（避免洩漏位置）。
- Blueprint 覆寫 cue 事件時必須呼叫 Parent，否則槽位不會觸發（指南已註明）。
- 對英雄、塔、小兵與召喚物來說，Scale 只影響比例，整體大小由 `PresentationHeightWorldUnits` 決定（指南已註明）。

## 著陸檔案

- `omfue/Plugins/OmRuntime/OmRuntime.uplugin`
- `Source/OmRuntime/OmRuntime.Build.cs`
- `Source/OmRuntime/Public/OmUnitActor.h`、`Private/OmUnitActor.cpp`
- `Source/OmRuntime/Public/OmAudioPresentationComponent.h`、`Private/OmAudioPresentationComponent.cpp`
- `Source/OmRuntime/Public/OmRuntimeSettings.h`
- `Source/OmGenerated/Public/OmWorldBridgeActor.h`、`Private/OmWorldBridgeActor.cpp`（手寫檔，不是 codegen 產物）
- 新增 `Source/OmEditor/Private/OmArtSlotTests.cpp`
- 文件：`unreal-art-guide.md`、本檔

## 補充：authored 英雄 auto_learn 端到端測試

已著陸到 `scripts/base_content/src/single_lane_match_tests.rs`，模組為 `authored_auto_learn`。在 shadow 執行 `cargo test -p base_content --lib -- authored_auto_learn`，3 個測試全部通過。

- `oichi_is_born_with_devotion_without_spending_a_skill_point`：出生時 D 為 `c17_devotion` rank 1。給 1 點技能點並手動升級 D 後，rank 與技能點都不變。
- `asai_nagamasa_learns_endless_fang_at_level_sixteen_for_free`：15 級時無法學 `b08_endless_fang`，手動升級被拒且不扣點。16 級時自動學到 rank 1，技能點不變；之後再手動升級仍被拒絕。
- `auto_learn_keeps_both_team_replicas_identical_every_tick`：120 tick 內 15→16 級自動學習，期間兩次手動升級都被拒絕。兩隊 replica 每個 tick 的 hash 都與權威一致。
- Mutation 檢查：把 `auto_learn_gated_ranks` 改成直接 return 時，後兩個測試失敗，oichi 測試仍通過。這是預期結果，因為 oichi 的 rank 1 來自出生 loadout `ranks[4]=1`。
