# DreamFX、DreamShader 文字化紀錄

轉換基準：omfue `1cbd8a5` 的 `Content/`，在影子專案 `target/e187/ue-shadow` 內操作，live 資產沒有改動。

## 外掛

| 外掛 | 來源 commit | 授權 | 修改 |
|---|---|---|---|
| DreamFX | `f11dd7d322` | repo 無授權檔，使用者轉述作者同意使用 | 無 |
| DreamShader | `d804ba9be0` | MIT | 移除預覽 WebSocket 與 WebSocketNetworking 相依 |

- **網路：** DreamShader 原本的 socket 只綁 127.0.0.1:17864，已移除，因為引擎沒有建置那個外掛，建置它會改動引擎。兩個外掛沒有其他監聽或對外連線；啟動外部程序只用於使用者開 VS Code。
- **引擎限制：** D:/UE5.8 沒有匯出 Custom HLSL 寫入 API，所以 `.dfm` 自訂模組無法生成，只會解析與檢查。也沒有快速批次 API，全量建置較慢。這批資產用不到 `.dfm`。

## 打包

- `om.uproject` 把兩個外掛限定在 Editor 目標。`OmGame` 的 UBT JsonExport 沒有任何 Dream 模組。
- `DefaultBackend=Graph`，產出普通 `UMaterial` 並存檔，不走 ThinCustom 的 Ephemeral 模式，也不用 `UDreamShaderMaterialInstance`。
- 樣本 cook（`M_OmStealthOverlay`、`NS_fx_pop` 的鏡像）成功，輸出只參照 `/Script/CoreUObject`、`Engine`、`Niagara`、`NiagaraCore`，沒有 Dream 字樣。還沒有做完整打包。
- **注意：** DreamShader 在 cook 開始時會重新生成專案 `DShader/` 下的來源並寫入 Content。樣本 cook 曾重新生成鏡像材質。文字切換成權威來源後，cook 會寫資產。

## Niagara（12 個）

| 檢查 | 結果 |
|---|---|
| `fx coverage` | 12/12 可匯出 |
| `fx build -All` | 12/12 建成，0 error，21 warning（DFX7102、DFX7103：沒設上限或 Determinism，是原本就有的設定） |
| `fx mirror-diff` L1、L2 | 12/12 通過 |
| `fx asset-diff` | 12/12 有差異，見下 |

`asset-diff` 的差異分類：

| 類別 | 例子 | 判斷 | 依據 |
|---|---|---|---|
| 被隱藏的輸入 | `AddVelocity.Velocity Speed Scale` 8→1、120→1 | 預期沒有效果 | 在 `NS_fx_stream` 實測寫回被拒（DFX5021），其餘同類未逐一實測 |
| FromPoint 模式的 `Velocity`；`LifeCycleMode=System` 下的 `Loop Duration` 2→1 | | 預期沒有效果 | 由模式推定，未實測 |
| 負零 | `Gravity` -0 → 0 | 沒有效果 | 數值相等 |
| 只存在原資產的 RI 常數（206 筆） | `InitializeParticle.Color` 等綁定 User 參數的項目 | 預期沒有效果 | 推定，未實測 |
| 編輯器資料路徑、曲線 DI 名稱 | `EditorData`、`FloatFromCurve` 與 `Scale Alpha` | 名稱不同 | 沒有逐條比對曲線內容 |

**尚未證明外觀相同：** 上面多數是靜態推定，還沒有比對畫面或粒子模擬。特別是曲線 DI 改名（`Scale Alpha.FloatCurve`→`FloatFromCurve.FloatCurve`）應在切換前確認曲線值相同。

## 材質（257 個：146 material、111 instance）

| 檢查 | 結果 |
|---|---|
| 反編譯 | 257/257 |
| 從文字重建 | 257/257（instance 暫時改 `@name` 指向鏡像） |
| 儲存屬性比對（`OmMaterialFacts`） | 257/257 無實質差異 |
| 重建後再反編譯的文字 | 253 相同，4 不同，見下 |

屬性比對略過的項目：

- GUID 類：`ParameterStateId` 等。
- `bCanMaskedBeAssumedOpaque`（21 個）：引擎會由圖形重新計算。未確認這個流程下何時重算。
- 排序不同：`ReferencedDefaultTextures`（4 個），內容相同。
- 等於父材質預設值的覆寫（4 個）：現在效果相同，但切換後父材質預設值改動時，這些 instance 會跟著變。
- 沒有對應父參數的舊覆寫（`Spin`、`Soft`，11 個）：本來就沒有作用，文字沒有保留。

文字不同的 4 個：

- `M_SaikaMagoichi`、`M_SaikaMagoichi_Accent`：反編譯不輸出 `bUsedWithMeshDeformer`，已在 `.dss` 手動補上，重建後屬性正確。**再次反編譯會遺失這個旗標。**
- `M_OmFxSprite`、`M_OmFxGround`：`Tex` 從 TextureSampleParameter2D 變成 texture 參數加 sample（DSH9073），計算相同、名稱相同。**這批鏡像 instance 的父材質仍是原材質，所以還沒驗過 instance 掛在重建後父材質上的結果。**

反編譯時的其他診斷：DSH9077（節點位置不保留，144 次）、DSH9075（變數或自訂函式改名，9 次，自訂內容沒有互相呼叫）、DSH9061（5 個材質各有 1 個未接輸出的節點被丟棄）。

## 無法轉換

沒有完全轉換失敗的資產。不保真的項目：

| 項目 | 影響 |
|---|---|
| 節點位置、註解 | 只影響編輯器顯示 |
| 上一節列出的旗標和覆寫 | 見上 |
| `.dfm` 自訂模組 | 這個引擎無法生成；目前沒有用到 |

## 切換第一批（2026-11-04）

範圍：`Content/Omoba/VFX/Fx/**` 以外、沒有被雜湊鎖定的 81 個材質與 instance，加上 `Omoba/VFX/CueShapes` 的 6 個 Niagara 系統。文字來源在 `omfue/DShader/`、`omfue/DFX/`，這些資產從此以文字為準。

流程（`scripts/dream_switch.lua`，只在影子執行）：標記 `DreamShader.SourceFile`（`OmDreamAdopt`）、`compile -All -Force` 覆寫原資產、比對前後屬性；CueShapes 用 `fx build -All -Force`。原資產沒有刪除，放回前確認都在 git 且未修改，可還原。放回時持有 `live_match` 鎖，寫入前再比對一次，live 自快照後沒有變動。

| 檢查 | 結果 |
|---|---|
| 屬性比對（當時含 Fx 共 271 個） | 271 相同；實質差異 0；等於父預設值而省略的覆寫 4；僅貼圖順序不同 7 |
| `bUsedWithMeshDeformer` | 重建後 `M_SaikaMagoichi`、`M_SaikaMagoichi_Accent` 為 True |
| CueShapes 建置 | 6 個，0 錯誤，20 警告 |
| 錄影比對（同一份內容快照，切換前後） | 佐佐木（近戰）、大谷（法系）顏色與特效相同，只有取樣時間差；未做逐像素量測 |
| 實機局是否寫檔 | 錄影後影子 Content、DShader、DFX 無新檔 |
| 完整 cook（影子，`-SkipZenStore`） | exit 0，3881 個 package，0 錯誤，2 警告；271 行 `[Cook] Generated`，但 Content 無檔案變動；cook 產物 3317 個 .uasset 無 Dream 字串 |
| Game target | 沒有 Dream 模組 |
| 放回 | 先放回 320 個檔案（160 資產、154 + 6 原始碼），`land_to_live` 的 codegen、UBT 通過，寫入後逐檔相同；之後還原 73 個，見下節 |
| 還原後 | `build_ue_moba` exit 0；`shader check` 81 份 0 錯誤 |
| 放回後 live 錄影 | 佐佐木、大谷與影子切換版對照：角色、地面材質與特效顏色無異常，沒有缺材質；特效出現時間點有取樣差異。目視抽樣，未做逐像素量測 |

反編譯遺失旗標的檢查：`dream_switch.lua` 比對資產的 `bUsedWith*` 與文字，缺少就失敗，加 `--fix-flags` 才會補。

### Niagara 必須從零建

`dream_niagara_sim.lua` 以固定種子和步長執行原資產與重建版，比對粒子數、每幀屬性範圍和渲染器設定。對照組（同一資產跑兩次）差異 0，方法可重現。

| 比對 | 粒子數 | 屬性值 | 結論 |
|---|---|---|---|
| 第一批 CueShapes（在既有資產上重建） | 116 筆不同 | 1856 筆不同 | `NS_shape_slash`、`NS_shape_trail` 的 ribbon 粒子數是 2 倍 |
| 修復後，live，共 960 筆粒子數、22776 筆屬性值 | 0 | 0 | 相同 |

原因：原資產的 event handler 有非零 usage id，DreamFX 在既有資產上重建時會另外加一個 zero-id handler，同一事件被處理兩次。`77fb8874` 放回的這兩個系統有這個問題，已修復。

做法：Niagara 在影子裡刪除原資產後從零建，再放回。從零建之後再重建不會累積（已實測）。放回前必須跑 `dream_niagara_sim.lua`，`fx asset-diff` 看不出這個問題。

### asset-diff 的 54 個同名不同值

12 個系統，全部從文字從零重建後，與原資產做粒子模擬比對。

| 類別 | 數量 | 影響畫面 |
|---|---:|---|
| 編輯器資料路徑（`EditorData`、`EditorParameters`） | 34 | 否，執行時不使用 |
| `GravityForce.Gravity` 的負零（-0→0） | 10 | 否，數值相等 |
| 被隱藏的輸入 `AddVelocity.Velocity Speed Scale` | 5 | 否 |
| FromPoint 模式的 `AddVelocity.Velocity` | 3 | 否 |
| `LifeCycleMode=System` 下的 `Loop Duration` | 2 | 否 |

| 比對 | 渲染器設定 | 粒子數 | 屬性值 | 差異 |
|---|---:|---:|---:|---:|
| 6 個 `NS_shape_*`（live） | 902 | 960 | 22776 | 0 |
| 6 個 `NS_fx_*`（影子，fix135 之前的版本） | 406 | 420 | 6168 | 0 |

模擬比對涵蓋 60 幀（2 秒）、預設 User 參數、CPU 粒子。沒有涵蓋：比 2 秒更長的行為、非預設的 `Tint`、`Size`、`Life`，以及實際畫面。實際畫面由切換前後的錄影比對。

曲線 DI 改名（`Scale Alpha.FloatCurve`→`FloatFromCurve.FloatCurve`）的疑慮也由這項比對排除：取樣結果相同。

### 影子檢查

`dream_shadow.lua` 解析連結後判斷；`Content`、`DShader`、`DFX` 連到 live，或內含任何連結，都拒絕。`dream_switch`、`dream_roundtrip`、`dream_assets --project` 的寫入動詞都會檢查。以 `target/ue-shadow-c` 實測全部被拒絕，沒有寫入檔案。

### 放回後的失敗與修復

第一次放回了 154 個材質，`build_ue_moba` 在地圖材質步驟失敗（`managed biome material changed: jungle`）。這些資產的雜湊被匯入工具鎖定，覆寫後檢查不再相符。`land_to_live` 和 `check_live_consistent --ubt` 都通過，抓不到這類問題，放回資產後要跑真正的 `build_ue_moba`。

修復：73 個資產還原成原檔，對應的 `DShader` 原始碼撤掉。之後切換前先跑 `dream_assets.lua pinned [--filter 樣式]`，`dream_switch.lua` 也會拒絕被鎖定的資產。

### 不轉換

| 資產 | 數量 | 鎖定位置 | 原因 |
|---|---:|---|---|
| `Omoba/Characters/TripoV1/**/tripo_mat_*` | 26 | `TripoV1/source-registry.json` | 匯入時產生；`import_character_sources` 要求位元組不變，否則報 `registered package changed` |
| `Omoba/Structures/TripoStaticV1/**/tripo_mat_*` | 40 | `TripoStaticV1/source-registry.json` | 同上 |
這 66 個留在原有流程。`MapBiomesV1`、`WorldSurfacesV1` 的 7 個材質已切換成文字，`ue_map_biome_materials` 對有來源的材質不再檢查雜湊。`omfue/DreamSources` 的快照裡仍有它們的文字，只供查閱。

### 注意事項

- 其他產生工具（`character_materials`、`build_world_model_assets`、`build_presentation_materials`、`combat_visual_assets` 等）重跑時可能覆寫已切換的資產，還沒有逐一確認，也沒有檢查擋住。
- `OmFxFamilyBuilder` 的 `-shapes=` 模式會覆寫 CueShapes，與 `DFX/` 不一致。`-fx-family` 只寫 `VFX/Fx`，不受影響。
- 在互動式編輯器裡，DreamShader bridge 的啟動掃描與 `bAutoCompileOnSave` 可能重建來源雜湊過期的材質。commandlet、cook、遊戲不會經過 bridge。
- （第一批時）`M_OmFxSprite`、`M_OmFxGround` 在 `VFX/Fx`，不在這批；影子裡含 Fx 的試轉驗過 instance 沒有變色或缺材質，第二批要重新轉換。
- 沒有跑完整 `BuildCookRun` 打包，只跑 cook 與 target 模組檢查。

## 第二批：`VFX/Fx`

153 個資產改由文字建成（共 248 個）：

| 資產 | 數量 | 來源 |
|---|---:|---|
| `M_OmFxSprite`、`M_OmFxGround`、`M_OmFxMeshTranslucent` | 3 | `DShader/Omoba/VFX/Fx/*.dss`，反編譯 |
| `MI_fx_*`、`MIG_fx_*`、`MIG_mesh_*` | 144 | `DShader/Omoba/VFX/Fx/Instances/*.dsi`，由 `fx_family.lua` 的 `IMAGES`、`MESHES` 產生，`--check` 檢查是否過期 |
| `NS_fx_*` | 6 | `DFX/Omoba/VFX/Fx/Systems/*.dfs`，反編譯，直接編輯 |

`T_fx_*`、`T_mesh_*`（84 個）是貼圖，繼續由 PNG 匯入、留在版控。`-run=OmCueVfx -fx-family=` 只匯入圖片，像素相同就不動。

`fx_family.lua` 拿掉 `SYSTEMS` 內容，改讀 `DFX` 目錄的檔名，避免兩份來源。圖片產生器放到 `scripts/fx_images/`：`gen_wave2.lua` 重現 36 張、`gen_thrust.lua` 重現 2 張，都逐位元組相同；其餘 22 張是生成圖，PNG 就是來源。

驗證（影子）：147 個材質與 instance 相同；6 個 Niagara 從零重建，406 設定、420 粒子數、6168 屬性值 0 差異；248 個都由文字建成；153 個都沒有被鎖定。同一英雄（`sasaki_kojiro`）live 與影子各錄一段，目視相同，沒有逐像素量測。

配方表裡沒有、`DefaultGame.ini` 或資產仍有的 5 組（`axe_cleave`、`mist_bands`、`quake_ripple`、`earth_spikes`、`stone_shards`）一併收進表裡，沒有刪除。

放回必須持有 `live_match`：實錄局的執行副本會開著 live 的 `.uasset`，沒有鎖時寫入會 `Permission denied`（`land_to_live` 這時整批不寫入）。

## 純文字 clone 預演

`scripts/dream_clone_verify.lua`：clone HEAD 與子模組，刪掉 248 個文字擁有的 `.uasset`，`build_ue_moba` 成功，實錄 `sasaki_kojiro` 一局正常。沒有動版控設定。

停止追蹤前要解決：`Plugins/BpGeneratorUltimate_1.7.3/Binaries` 被 gitignore，乾淨 clone 沒有它就無法建置。

預演找到：`build_ue_moba` 的 ensure 會等自己持有的 build 鎖而卡死（已修）；錄影需要 `target/tools/ffmpeg`；`build_ue_moba` 不建 server，要另外跑 `run_moba_split_local --build-only`。

## 還沒做

Tripo 的 66 個材質、停止追蹤 `.uasset`、完整 cook 驗收。

另外發現：`fx_family.lua` 的 `AddVelocity.Velocity Speed Scale` 很可能沒有作用，已告知線 C。
