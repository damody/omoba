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
| `Omoba/Characters/TripoV1/**/tripo_mat_*` | 26 | `TripoV1/source-registry.json` | 當時的判斷；後來已轉換，見「Tripo 材質」 |
| `Omoba/Structures/TripoStaticV1/**/tripo_mat_*` | 40 | `TripoStaticV1/source-registry.json` | 同上 |
這 66 個當時留在原有流程。`MapBiomesV1`、`WorldSurfacesV1` 的 7 個材質已切換成文字，`ue_map_biome_materials` 對有來源的材質不再檢查雜湊。`omfue/DreamSources` 的快照裡仍有它們的文字，只供查閱。

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

## 切換前後逐招比對

切換前的片是線 C 的基準（`target/lineC-perf/baseline-2026-11-05/`），切換後用 run 目錄的原始影格。以每招的 `spotlight_cast` 對齊，兩邊各取施放後 0.15、0.4、0.8、1.3、2.0 秒；可疑的 5 招再每 0.07～0.12 秒取 8 格。對照圖在 `target/e187/cmp-slots/`（不進版控）。

全部 52 次施放：相同 45、無法判定 1、跳過 6、有差異 0。

| 英雄（切換前 → 切換後） | 相同 | 無法判定 | 跳過（配方改過） |
|---|---|---|---|
| 山縣 020633 → 042541 | Q（密集）、W、R、普攻、死亡 | | E 兩次：`b20e_thunderclap` 改了 |
| 德川 021028 → 043057 | W、E、R、X、普攻、死亡 | | Q：`a01w_holy_bolt` 來回改過 |
| 小島 021446 → 032255 | Q、W、E、普攻、死亡 | | R：裂紋時間改了 |
| 濃姬 021835 → 040419 | Q、普攻、E 兩次、R、死亡 | | W：火藥桶改了 |
| 佐佐木 022207 → 放回後 live | Q、W、E 兩次、X、普攻、死亡 | | R：刀痕時間改了 |
| 大熊 023445 → 放回後 live | Q、X（密集）、W、E 三次、R（密集）、普攻、死亡 | | |
| 慶次 024320 → 放回後 live | Q、W、R（密集）、普攻、E 兩次、死亡 | E 第三次：見下 | |

判定依據是肉眼看特效的形狀、顏色和層次，沒有逐像素量測。「相同」不代表逐格相同：兩局的命中和普攻時間有隨機差異，施放對齊後特效仍有約 0.1～0.2 秒的相位差。

另外比對了 7 位切換前後 `ue-p1.log` 的 `OM_CUE`：兩局觸發的效果種類完全相同（11～16 種）。次數差異只有這些，都是爆擊、連鎖、命中或回復次數，每項差 1～3 次：`b04e_blast`、`b04e_impact`、`cast`（山縣），`a01d_restore`、`fx_layer_mana_motes`（德川），`b01r_hit`（小島、大熊），`a26r_crit_gesture`（濃姬），`c22r_crit_sound`（佐佐木），`c05r_chain_lightning`（慶次）。

看起來不同但判定相同的：
- 佐佐木 X：切換後多了橘色爆發和金色弧，那次傷害 469.7 對 163，是爆擊。
- 慶次 R、大熊 R 與 X、山縣 Q：單格看有濃淡差，密集比對後同一階段的顏色和形狀一致，只差相位。
- 死亡段幾乎沒有特效，比的是角色材質和殘留的地面環。
- 德川 E：切換後在施放後 1.8 秒多一次藍環和閃電，切換前同一段沒有。兩局都是 4 道閃電（`a01r_lightning`），間隔不同：切換前 0、0.67、2.41、4.15 秒，切換後 0、0.67、1.53、3.27 秒，切換前的第三道落在取樣範圍外。兩局外觀相同。

無法判定：
- 慶次 E 第三次：鏡頭換到另一處，兩邊取樣的格子裡，只有切換後一格看得到閃電，沒有同相位可比。

## 停止追蹤與重新 clone 驗證

omfue e22daa97 停止追蹤 250 個 `.uasset`（`scripts/dream_untrack.lua` 列的；正好 250 筆刪除），主 repo 53bbbdfb 指向它。deb2bd99 的訊息說已停止追蹤，實際只提交了 `.gitignore`：`git commit -- 路徑` 比的是工作目錄，檔案還在就沒有變更。共用索引也可能被其他 session 的提交改動，這次改用臨時索引建立 commit，再比對舊值更新分支。

`target/e187/clone2`：HEAD 53bbbdfb、`git ls-files` 不含這 250 個、磁碟上 0 個、沒有 BpGeneratorUltimate 的 `Binaries`，也沒有複製任何建置產物。

| 步驟 | 結果 |
|---|---|
| `build_ue_moba` | exit 0，2549 秒；UBT 把 BpGeneratorUltimate 也編出來 |
| ensure | 重建 240 個材質、12 個 Niagara，252 個由文字建成（多的 2 個是線 C 新增的 `MIG_mesh_*`） |
| `run_moba_split_local --build-only` | exit 0，823 秒（`build_ue_moba` 不建 server 和 client runtime） |
| 實錄 `sasaki_kojiro` | record 0，持 root 的 `live_match`；只手動補了 `target/tools/ffmpeg` |

clone 後 OmGenerated 和 `om_bridge.h` 共 24 個檔案顯示已修改：23 個只差行尾，`om_codegen_manifest.json` 的 137 行是絕對路徑。

待處理（不擋主線）：引擎路徑寫死 `D:/UE5.8`；`om_codegen_manifest.json` 含絕對路徑；文字檔 checkout 後是 CRLF；`hud-assets-v1.json` 重建時順序會變；孤兒圖片繼續留著。

## Tripo 材質

73 個 `tripo_mat_*`（線 C 新增後從 66 變 73）改由文字建成，共 325 個。

工具改動：registry 拿掉這 73 個的 `package_sha256`；`import_character_sources` 與 `OmCharacterSourceImportCommandlet` 看到有來源的材質就不檢查、不存檔；`OmCharacterRetargetCommandlet` 也不再把它們記進 Meta 與雜湊。

驗證：
- `dream_switch`：73 個切換前後相同。
- live `build_ue_moba` 通過；`tripo_v1`、`tripo_static_v1` 匯入通過，第二次沒有存任何檔；`date_masamune` 實跑 retarget 通過，資產沒變。
- 打包 package3：BuildCookRun 成功，3958 個套件，73 個材質都在 utoc 裡，沒有 Dream。打包版實錄濃姬，材質與道具正常。

與線 C 基準片的比對（10 位，每位 6 招、施放後 0.4 秒；對照圖在 `target/e187/cmp-tripo/`，不進版控）：

| 項目 | 結果 |
|---|---|
| 施放順序 | 10 位都相同 |
| 身體、道具、材質 | 都沒看出差異（肉眼，沒有逐像素量測） |
| 特效 | 只有約 0.1 秒的相位差；大熊 R、島津 E 加密取樣後同一階段一致 |
| 小島死亡段 | 倒地時間差 0.6 秒、面向不同，受戰鬥隨機影響；材質相同 |
| `OM_CUE`（有 log 的 6 位） | 種類相同；山縣次數全同，另外 5 位是爆擊、命中或回復次數差 1～4 次 |
| `OM_CUE`（大熊、慶次、利家、島津） | 無法判定：基準沒有 log |

停止追蹤：omfue 0878d7ae（`.gitignore`）、26e08a17（正好 73 筆刪除），主 repo 00ab9d78。重新 clone：

`target/e187/clone3`：HEAD 00ab9d78，`git ls-files` 不含這 73 個，磁碟上 0 個，也沒有 BpGeneratorUltimate 的 `Binaries`；只手動補了 `target/tools`。

| 步驟 | 結果 |
|---|---|
| `build_ue_moba` | exit 0，2304 秒 |
| ensure | 重建 313 個材質、12 個 Niagara，325 個由文字建成 |
| `run_moba_split_local --build-only` | exit 0，907 秒 |
| 實錄 `nohime` | record 0，持 root 的 `live_match`，材質與道具正常 |
| 重建後 | 磁碟上 73 個，`git status` 不列它們；另有 24 個已修改檔案，和上一次相同（行尾、manifest 絕對路徑） |

新匯入的 Tripo 模型，材質先照舊是二進位、留在版控，registry 也照舊鎖 SHA。要轉成文字時整批做：`dream_switch`、放回、拿掉 `package_sha256`、`dream_untrack`。

## 特效貼圖

89 個 `T_fx_*`、`T_mesh_*`（`VFX/Fx/Textures`）停止追蹤，由版控裡的 PNG 匯入。全部都在 `fx_family.lua` 的 `IMAGES`、`MESHES`，PNG 也都在版控。之前文件寫 84、88 是當時的數量。

驗證：在沒有 Content 的影子裡從 PNG 重新匯入，和版控裡的檔案比對。89 個大小都相同，每個只差 6 段（20、32、16、32、16、16 位元組，最多 132 位元組），都落在檔案前約 3.8 KB，是 hash 與 GUID 的長度。其餘逐位元相同，包含壓縮後的來源像素與匯入設定（sRGB、LODGroup、壓縮、Address、NeverStream）。

工具：`dream_assets.lua ensure` 先跑 `fx_family.ensure_images`，貼圖缺檔或 PNG 較新才匯入，之後才建材質，因為 instance 引用這些貼圖。`.gitignore` 整個資料夾忽略；已被追蹤的貼圖不在 `fx_family.lua` 時，`dream_untrack` 會報錯。沒有追蹤、也沒列在 `fx_family.lua` 的貼圖，現在由 `dream_assets owned` 擋下，見「clone 後的收尾」。

提交：工具 8098e96e；omfue `.gitignore` 與 9c0e7fba（正好 89 筆刪除），主 repo 958e8926。

重新 clone（`target/e187/clone4`，HEAD 958e8926）：`git ls-files` 不含貼圖與 Tripo 材質，磁碟上 0 個，也沒有 BpGeneratorUltimate 的 `Binaries`；只手動補了 `target/tools`。

| 步驟 | 結果 |
|---|---|
| `build_ue_moba` | exit 0，1639 秒 |
| ensure | 匯入 89 張貼圖，重建 315 個材質、12 個 Niagara，327 個由文字建成 |
| `run_moba_split_local --build-only` | exit 0，749 秒 |
| 實錄 `nohime`、`yamagata_masakage` | 都是 record 0 |
| 重建後 | 磁碟上 89 張，`git status` 的 Content 為 0；另有 24 個已修改檔案，都在 OmGenerated 與 OmBridge（和前兩次相同的類別，這次沒有逐檔比對內容） |

畫面比對（切換前 `spot-tripo` 對 clone4，每位 6 招、施放後 0.4 秒；對照圖 `target/e187/cmp-tripo/clone4_*.jpg`，肉眼，沒有逐像素量測）：

- 山縣：配方沒改過。施放順序相同，貼圖、材質沒看出差異，特效只有相位差。`OM_CUE` 種類相同，`b04e_impact`、`b04e_blast`、`cast` 次數差 2～4 次，這幾項上次比對時也有差。
- 濃姬：施放順序相同，W 沒看出差異。普攻與 E 不一樣，是線 C 的 09162295 改了配方（橘色槍口、E 火焰加大），這兩招比對不出貼圖差異。

## 仍是二進位的資產

讀 3163 個受追蹤 `.uasset`／`.umap` 的註冊資料取得實際類別（`target/e187/classes.tsv`，不進版控），這是貼圖停止追蹤前的數字。**材質、材質實例、Niagara、材質函式、材質參數集都是 0 個**，沒有可再文字化的材質或特效。Plugins 下沒有受追蹤的 `.uasset`／`.umap`。

| 目錄 | 數量 | 實際類別 | 判定 |
|---|---:|---|---|
| Characters | 2701 | AnimSequence 2363、IKRigDefinition 100、SkeletalMesh 72、IKRetargeter 50、Skeleton 40、StaticMesh 15、Texture2D 61 | 不是材質或特效，不動 |
| OmGenerated | 36 | AnimSequence 11、PhysicsAsset 10、SkeletalMesh 10、Skeleton 2、Texture2D 3 | 同上 |
| UI | 74 | Texture2D 74 | 同上 |
| Structures | 118 | StaticMesh 53、Texture2D 65 | 同上 |
| WorldModelsV1、WorldSurfacesV1、Props、AuthoredAnimationsV2 | 20 | StaticMesh 9、Texture2D 7、AnimSequence 4 | 同上 |
| 地圖 | 2 | World 1（`Map/Main.umap`）、ObjectRedirector 1（`Main.umap`） | 同上 |
| VFX/Fx/Textures | 89 | Texture2D 89 | 已停止追蹤，見「特效貼圖」 |
| RustBP | 107 | Blueprint 59、SkeletalMesh 13、AnimSequence 12、PhysicsAsset 10、WidgetBlueprint 9、Skeleton 2、AnimBlueprint 1、Texture2D 1 | 未處理（這次範圍不含） |
| Audio | 17 | SoundWave 17 | 未處理（這次範圍不含） |

"不動"只表示不屬於這次的材質與特效範圍，沒有判斷它們能否由其他來源重建。

## clone 後的收尾

提交：omfue 9f957102，工具 74975141 與 6cfc9efe，主 repo 7cabb9d6。

**未登記的 ignore 資產。** `dream_assets owned` 檢查 `.gitignore` 區塊的整個資料夾與明列檔案：磁碟上有、但不在 `fx_family.lua` 或 `DShader`／`DFX` 來源裡，就失敗並列出檔案。`build_ue_moba` 的 ensure 會跑它。`scripts/tests/dream_orphans_test.lua` 測判定，另用只有一張未登記貼圖的臨時專案實跑過，exit 1。

**clone 後固定出現的 24 個已修改檔案。** 用 sparse clone 重現並逐檔比對：

| 檔案 | 原因 | 修正 |
|---|---|---|
| OmGenerated 22 個 | 內容相同。checkout 寫 CRLF、產生器寫 LF，大小不同，git 不比內容就報已修改 | `.gitattributes` 固定 `text eol=lf` |
| `om_bridge.h` 1 個 | sparse 驗證時以 live 產生的檔案代替 cbindgen，這個重現沒有單獨證明機制；重現前後都只見行尾差異 | 同上 |
| `om_codegen_manifest.json` 1 個 | 137 行絕對路徑，`lua_input_hash` 也混入路徑 | 改成相對 `scripts/lua_data` 的路徑；hash 用相對路徑並把 CRLF 正規化。`content_hash` 不含它，marker 不變 |

驗證：`core.autocrlf` 為 true 與 false 各一個 sparse clone（含 `om_bridge.h`），重新產生後 `git status` 都是 0 筆。`om_codegen` 測試通過，新增 `manifest_does_not_depend_on_the_checkout`。這次沒有跑全量 clone；上一次全量 clone 的已修改檔案只有這 24 個。

**HUD 清單不需要改。** 看了 `sync_ue_hud_manifest_labels` 與 `ue_hud_portrait_coverage`：寫入內容只經 `ipairs`，JSON 鍵已排序，沒有變更時不寫入。上一次全量 clone 的 Content 也是 0 筆已修改。先前「順序會變」的記錄沒有重現。

**引擎路徑。** `platform.ue_root(given)`：`--ue-root`、`UE_5_8_ROOT`、`UE_ROOT`、`UE_5_7_ROOT`，預設 `D:/UE5.8`。工具都改用它，`moba_role_launch`、`run_2player_ue`、`run_ue` 原本就先讀環境變數再搜尋常見路徑，維持不動。實測 `UE_5_8_ROOT` 會改變 `dream_switch` 的編輯器路徑；沒有在另一個引擎路徑上實跑建置。

## 還沒做

材質與特效已全部處理。RustBP 藍圖與音效未處理，範圍不含。孤兒圖片繼續留著。

另外發現：`fx_family.lua` 的 `AddVelocity.Velocity Speed Scale` 很可能沒有作用，已告知線 C。
