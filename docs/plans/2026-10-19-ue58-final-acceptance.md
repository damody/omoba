# UE5.8 新機建置與本輪功能驗收

2026-10-19：使用 `D:/UE5.8`（實際5.8.4），正式 bootstrap、普通 NoEngineChanges 建置與最後 stage SHA 檢查成功。VS2022 17.14.37806.7／MSVC19.44.35229／SDK10.0.22621.0／Rust1.95.0／cbindgen0.29.4。過程每3分鐘以具程序身分的監控記錄檢查，長步驟另核對CPU、影格與事件進度。

本輪完成公開HP增減數字：共用世界級64文字slot、每owner最多4個、lazy配置與回收；首幀／scope／世代／揭露／重生／MaxHP變更只種baseline，負HP限於實際生命損失，pause凍結、fog立即退休。使用Engine UnlitText，明確正負號，沒有逐entity新增空文字元件或推測未揭露的施法者／傷害來源。另修正公開攻擊資格持續同步、商店文字縮放的row cache、選角正常TCP收尾與新機建置／驗收前置。

可攜摘要：[index.json](../acceptance/2026-10-19-new-machine/index.json)。原始失敗、執行log、PCM與AVI留在本機Saved；沒有提交建置產物。

## 實際驗證

| 範圍 | 結果 |
|---|---|
| 完整native | 198項全部Success，12個既有warning保留；不是零warning。 |
| 既有Blueprint | 69份，compiler error／warning／failed load皆0，資產SHA與mtime受守衛保護。 |
| Owned viewport | 458案例、222 wire rows、157 control rows、13游標觀察；publisher／editor／profile全部清理成功。 |
| 音訊 | 五份實際submix PCM成功；不代表實體喇叭聽感接受。 |
| 錄影 | 3900取樣影格／11個原尺寸AVI／117512618 bytes，影格、RIFF／索引、JPEG envelope、SHA與時間紀錄通過；直接檢視兩個未重編碼JPEG樣本。 |
| Rust shared／script／ABI | base_content241、core784、ABI15單元成功；另三個TD integration（包括headless／observed完整1–100波）298.41秒成功；doc test1成功、1既有ignored。 |
| 其他Rust consumers | client、bridge、codegen完整測試成功；backend最後整包成功（library189成功／1既有ignored，main重複的library案例不加算不同測試，其他harness見JSON）。 |
| 建置檢查 | 五個all-target命令成功，覆蓋backend、shared／base／ABI、client、bridge、codegen；Lua host26及建置／驗收Lua契約成功。 |
| 最後對接 | 重建選角工具後，再驗1項HUD專項，Success／零warning；不把它加算為199項完整native。 |
| 指紋 | 最終3325份選定來源／資產SHA與mtime再核對成功。完整UE驗收基線3316份中，3313份不變；最後3個backend檔案修正已重驗，另9份backend integration輸入補納守衛。前端、shared runtime與資產沒有漂移。 |

Rust原combined單元命令因缺release script DLL而exit1的log仍保留；1040單元已實際成功，補正式release DLL後另跑三個integration，並補完doc tests。初次backend批次失敗也保留，以最後完整backend與all-target結果取代該scope，沒有改寫失敗報表。既有ignored按各harness列示，沒有新增ignore或把重複main案例加總為不同測試。

完整UE來源／資產基線、最後backend差異及新工具HUD確認的關係，見 `reviewed-source-delta.json` 與本機 `PostToolHudV1/summary.json`。最後stage為：

- bridge：`216d59e4f7b7f1698a4ec6d605ca302850aa65e5f235e3d3fa69d73f0a17b753`
- base_content：`6cf7a4285e53324900c73a99f78b7d29197f8fa1efb31b266f633e8475dba856`

## 直接看過的當輪原圖

生命恢復字形：[public-restoration.png](../acceptance/2026-10-19-new-machine/public-restoration.png)。另直接檢視400px聊天、操作設定，以及AVI中800×600／400×300兩個原始JPEG；可攜副本皆保留原SHA，沒有重編碼影片樣本。

## 接受界線

錄影與458事件屬synthetic filtered-feed真Slate視窗，世界區域黑底；不是新權威對局／ACK、LAN、完整三路美術或固定效能接受。沒有啟動新權威對局，10/10額度未重設；`omfx`沒有維護、編譯或驗收。本輪未commit／push。

最後一個TOML行尾格式修正僅移除CR一byte，前後完整Cargo metadata一致；來源守衛v5重新核對全部3325份成功，可攜摘要含`manifest-ending-proof.json`。

OpenSpec仍27/44，17個父項保留待辦。生命數字屬4.3的已完成子範圍，不因本機綠燈勾完權威ACK等未驗項。錯誤與避免重犯規則詳 `unreal-moba-error-register.md` E1300–E1334。
