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

## 切換前還要做

1. 把 `.dfs` 的 `Name=` 和 `.dsi`、`.dss` 的資產路徑改成原資產；原資產需要先刪除或標記成外掛產生，否則會被拒絕覆寫（DSH8242）。
2. 和線 C 協調 `Content/Omoba/VFX/Fx/**`，並決定 `fx_family.lua` 的去留。
3. 實機錄影比對畫面。

另外發現：`fx_family.lua` 的 `AddVelocity.Velocity Speed Scale` 很可能沒有作用，已告知線 C。
