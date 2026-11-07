# Unreal 美術替換指南（只在 Editor 操作，不改程式）

這份指南給美術使用。所有步驟都在 Unreal Editor 裡完成：建立 Blueprint、在 Details 面板填入資產、存檔。不需要改 C++、Lua，也不需要重新產生內容。驗收方式見最後一節的自動化測試。

## 0. 新英雄：fallback 外觀 → 正式模型（最短步驟）

新產生的英雄如果還沒有自己的模型，遊戲中會顯示**共用 fallback 人形**：從訓練英雄的 Quaternius rig 中依英雄 id 固定挑一套，帶有 idle、move、attack、cast、hit、death 動畫。這條規則寫在 `scripts/lua_data/templates/moba_shared_hero_fallback.lua`，套用到所有沒有 mesh 的英雄。生成 class 會把這個 mesh 記在 `NativeSharedFallbackMesh`；只要你在 Blueprint 指定不同的 `NativeMeshAsset`，就算是正式美術。依下列步驟換成正式模型：

1. 確認內容 id：例如 `omatsu`。生成 class 是 `OmHero<PascalCase>`（`OmHeroOmatsu`），預設 Blueprint 路徑是 `/Game/RustBP/Heroes/BP_<PascalCase>`（`BP_Omatsu`）。
2. Content Browser 進入 `/Game/RustBP/Heroes/`，右鍵 → **Blueprint Class** → **All Classes** → 搜尋 `OmHeroOmatsu` 作為 Parent，命名為 `BP_Omatsu`。路徑和名稱一致，遊戲就會自動使用它（入口 A）；也可以任意命名，再到入口 B 指定。
3. 打開 **Class Defaults** → **Om|Visual**：
   - `NativeMeshAsset`：選 Skeletal Mesh。
   - `NativeAnimations`：至少填 `idle`、`move`、`attack`、`death`，其他槽位見第 3 節。缺少的槽位依 `NativeAnimationFallbackPolicy` 退回。
   - 材質不對時，再填 `NativeMaterialSlots`。
   - 不要新增元件。高度沿用英雄預設的 120，除非要調整 `PresentationHeightWorldUnits`。
4. 按 **Compile**、**Save**。
5. 驗收：
   - 執行 `Om.Runtime.PublicHeroCombatRoster`。它解析 class 的方式和實機相同（入口 B → 入口 A → 生成 class）。這位英雄的 `Content gap: hero <id> shows the shared fallback humanoid` warning 消失，就表示替換成功（fallback 人形同樣會跑完整的呈現生命週期，只是仍列為 content gap）。
   - 再跑 `Om.Generated.EveryUnitClassFacesActorForward`，確認正面方向（見第 5 節）。

## 1. 兩種替換入口

| 入口 | 用途 | 位置 |
|---|---|---|
| A. 內容的預設 Blueprint 路徑 | 一般英雄、技能、buff、投射物 | `/Game/RustBP/<種類>/BP_<名稱>`。種類為 `Heroes`、`Towers`、`Abilities`、`Buffs`、`Summons`、`Creeps`、`Projectiles`，名稱是內容 id 的 PascalCase，例如 `saika_magoichi` → `BP_SaikaMagoichi` |
| B. Project Settings 指定 | 任何單位都可以用，包括 `native_only` 單位（分路小兵、野怪、訓練英雄沒有入口 A） | Edit → Project Settings → Plugins → Om Runtime → **Art → Unit Class Overrides**：Key 填內容 id（例如 `lane_melee`、`saika_magoichi`），Value 選你的 Blueprint |

入口 B 優先於入口 A。兩者都沒有時，使用生成的原生 class。Blueprint 載入失敗會在 log 中留下警告，並自動退回原生 class，遊戲不會中斷。

## 2. 建立替換 Blueprint（步驟）

1. Content Browser 右鍵 → **Blueprint Class** → **All Classes**，搜尋單位的生成 class。
   - 英雄：`OmHero<名稱>`，例如 `OmHeroSaikaMagoichi`。
   - 小兵與野怪：`OmCreep<名稱>`，例如 `OmCreepLaneMelee`。
   - 召喚物：`OmSummon<名稱>`。
   - 選這個 class 作為 Parent。
2. 依入口 A 命名並放到對應路徑，或任意命名後在入口 B 指定。
3. 打開 Blueprint，點 **Class Defaults**，在 Details 面板修改第 3、4 節的欄位。**不要新增 Skeletal Mesh 元件**：原生呈現會改用你加的元件，第 3 節的欄位就不再生效。
4. 按 **Compile**、**Save**。

## 3. 外觀與動畫槽位（Details → Om|Visual）

| 欄位 | 填法 |
|---|---|
| `NativeMeshAsset` | Skeletal Mesh。任何骨架都可以（Quaternius、Warcraft-style rig 等）。 |
| `NativeMeshTransform` | Location／Rotation／Scale。Rotation 的 yaw 規則見第 5 節。英雄、塔、小兵、召喚物的整體大小由 `PresentationHeightWorldUnits` 決定（Scale 會等比例放大或縮小到目標高度），因此 Scale 只用來調整比例，例如 X／Y 的寬瘦；其他種類直接使用 Scale。 |
| `bAutoAlignMeshForward` | 預設勾選：自動把骨架正面轉到單位前方。見第 5 節。 |
| `NativeMaterialAsset` | 整個模型套用同一個材質（覆蓋所有 section）。 |
| `NativeMaterialSlots` | 依材質槽名稱（Skeletal Mesh 編輯器中的 Material Slot Name）指定材質。 |
| `NativeMaterialSections` | 依 section 索引指定材質；只在槽位名稱不可靠時使用。 |
| `NativeMeshAttachments` | 掛在骨頭上的靜態模型（武器、帽子）。每項有 `Id`、`Bone`、`Mesh`、`Transform`、`MaterialSlots`、`bNormalizeReferenceBoneScale`（骨頭本身帶縮放時勾選）。 |
| `NativeAnimations` | 動畫槽位 → `FOmNativeAnimationBinding`：`Sequence`（Anim Sequence，必須屬於同一個骨架）、`StartSeconds`／`EndSeconds`（截取片段，0 表示整段）、`ImpactSeconds`（攻擊命中的時間點，用來對齊攻擊節奏；-1 表示沒有）、`bLoop`。 |
| `NativeAnimationStateSlots` | 把遊戲狀態名稱指向另一個槽位，例如 `sniper_mode → sniper`。 |
| `NativeAnimationFallbackPolicy` | 缺少槽位時的退回策略：`UseGenericState` 會退回 idle／attack 等通用槽位。 |
| `NativeDefaultPlayRate` | 所有動畫的預設播放速度。 |
| `PresentationHeightWorldUnits` | 單位在遊戲單位下的顯示高度。模型會依此縮放，名牌和血條會放在模型頂端上方。0 表示使用種類預設值：英雄 120、塔 240、小兵與召喚物 70，其他種類不做高度正規化。 |
| `NativeStaticVisualMesh`／`Transform`／`Materials` | 只用靜態模型的單位（不需要動畫的物件）。 |

### 動畫槽位名稱

| 槽位 | 何時播放 |
|---|---|
| `idle`、`idle_2`、`idle_3` | 待機（後兩個是隨機變化） |
| `move` | 移動 |
| `attack` | 普攻（依 `ImpactSeconds` 對齊命中） |
| `critical` | 暴擊普攻（沒有時退回 `attack`） |
| `cast`、`cast_secondary` | 施法 |
| `channel` | 引導中 |
| `hit` | 受擊 |
| `death` | 死亡（播放一次） |
| `respawn` | 重生 |
| `recall` | 回城引導 |
| 其他（例如 `sniper`、`sniper_move`） | 由 `NativeAnimationStateSlots` 或英雄的特殊狀態指定 |

## 4. 音效與特效槽位（Details → Om|Presentation Slots → `PresentationCueSlots`）

每一項的 Key 是 cue 名稱，Value 有以下欄位：

- `Sound`：Sound Wave 或 Sound Cue。取代這個 cue 的原始音效；為了不洩漏位置，仍然是非空間化播放。
- `Effect`：Niagara System 或 Cascade Particle System。
- `AttachPoint`：特效掛載的 socket 或骨頭名稱，空白表示單位原點。
- `Offset`：相對位移、旋轉與縮放。位移會乘上單位的顯示縮放。
- `bReplaceOriginalVfx`：預設勾選，有 `Effect` 時不再顯示內建的佔位特效。

| Cue 名稱 | 觸發時機 |
|---|---|
| `attack_windup` | 普攻前搖開始（音效、特效） |
| `attack_impact` | 普攻命中 |
| `critical_hit` | 暴擊 |
| `attack_avoided` | 攻擊被閃避 |
| `cast` | 任何技能施放 |
| `ability.<技能 id>` | 指定技能施放，優先於 `cast`，例如 `ability.lumen_bolt` |
| `hit` | 受到傷害 |
| `projectile_impact` | 投射物命中這個單位 |
| `death` | 死亡 |
| `respawn` | 重生 |
| `healing` | 生命回復（音效） |
| `move_start` | 開始移動（音效） |

同一個 cue 只會播放一次：快照重複送達時不會重播。單位被迷霧隱藏時什麼都不播放。

如果你在 Blueprint 裡覆寫了 `OnAttackPhase`、`OnAbilityCue`、`OnDamageCue` 等事件，**要呼叫 Parent 節點**，否則槽位不會觸發。`bUseOriginalPresentationAudio` 取消勾選時，這個單位的所有音效（包括槽位音效）都會停用。

技能、buff 與投射物本身的特效，請照第 1 節入口 A 建立 `UOmAbilityVisual`／`UOmBuffVisual`／`UOmProjectileVisual` 的 Blueprint 子類別，並在事件中放置特效。

## 5. 面向規則（bAutoAlignMeshForward 與明確 yaw）

- 遊戲中單位的正面固定朝向 actor 的 +X，也就是移動或攻擊的方向。英雄、小兵、野怪、召喚物都套用同一條規則。
- `bAutoAlignMeshForward` 預設勾選：載入模型時，從骨架的 reference pose 自動找出正面，優先用腳到腳趾（toe／ball），沒有的話用頭到臉（jaw／nose／chin／mouth／eye），再把模型轉到 +X。這時 `NativeMeshTransform` 的 yaw 會被忽略，pitch 與 roll 照常套用。
- 需要手動指定朝向時（例如骨架沒有上述骨頭，或模型是特殊造型），**取消勾選** `bAutoAlignMeshForward`，再在 `NativeMeshTransform` 的 Rotation 填入 yaw。這時以你填的 yaw 為準。
- 對應到 Lua 內容：`ue.native_visual.yaw_deg` 有明確寫值時，生成的 class 會預設關閉自動對齊，並使用那個 yaw。

## 6. 驗收（自動化測試）

Editor 中執行 Session Frontend → Automation，或使用命令列 `-ExecCmds="Automation RunTests Om."`：

| 測試 | 證明 |
|---|---|
| `Om.Editor.ArtSwapTwoRigs` | 只用 Blueprint 的 Details 欄位，就能把 Quaternius 農民小兵換成 saika_magoichi 的 Warcraft-style rig，也能反向替換。驗證生成、mesh、材質、Scale、骨架正面跟著 actor yaw、walk 播放替換的動畫、名牌高度等於設定值，以及 Project Settings 入口 B 能正確解析。 |
| `Om.Runtime.PresentationCueSlots` | cue 名稱、音效對應、ability 槽位優先、特效只生成一次、隱藏時不播放。 |
| `Om.Generated.EveryUnitClassFacesActorForward` | 所有生成的單位 class 正面都對準 actor 前方。 |
| `Om.Generated.NativeHeroPresentation` | 英雄原生呈現（mesh、材質、動畫 binding）。 |
| `Om.Runtime.PublicHeroCombatRoster` | 每位 catalog 英雄都以實機的 class 解析（入口 B → 入口 A → 生成 class）跑完呈現生命週期；沒有模型的英雄會以 content-gap warning 列出，換上模型後自動改為實測。 |
