# 召喚結算守門：最終驗收

完成complete-summon-settlement-admission。正式serial outcome settlement重新檢查召喚者存活、世代、陣營種類／隊伍、optional PlayerOwner與原有0..3600秒生命期。零秒仍為永久召喚。重複結果不覆寫、回滿血或刪除已落地單位；其他已有位置／生命元件的實體也受保護。拒絕時釋放落點預約，空白保留handle依specs正常frame boundary退休，不觸發Spawn hook或成功fact。成功spawn fact延後至實際元件落地後才送出，其他outcome排序維持原契約。

## 集中驗收

- Rust **930成功／零失敗／零ignored**：全omoba-core926，以及base_content召喚陣形4（另295 filtered）。包含5個新增結算測試、正式60Hz全rank召喚、扣費一次及pause生命周期。沒有冒稱重跑所有workspace Rust。
- 新測試透過真正process_outcomes及ECS資源：死亡／缺生命元件、隊伍／陣營種類改變、玩家歸屬變更／移除／新增、owner世代替換、永久／3600秒界限、負值／超界、未知template、封閉落點、重複結果、既有owner實體保護；成功hook與fact各一次，拒絕均為零。
- 正式backend及UE5.8建置成功；**64項Unreal回歸成功**，零Failed／NotRun，4個既有RuntimePresentationReset warning保留。7 staged DLL與import library一致。
- **3416來源／資產**集合、SHA、size、mtime前後一致；**108份證據／24張已審閱PNG**獨立逐byte／SHA校驗，圖片沿前批已審閱像素精確比對或差異目視。此批是authority結算變更，圖片只代表原生呈現回歸，不代替新玩法權威證明。
- 健康採樣4次，最大間隔170秒；6最終worker、3歷史worker及其watcher均退休。全局170秒timer保留；其clone沿用部分歷史來源路徑，當前進度另由各owned worker監看及模型讀取實際日誌／CPU確認，未用不變的歷史檔案推斷當前程序有進度。
- RustV1因test未匯入BlockedRegion而E0422；V2因誤把specs延遲刪除當成立即is_alive=false而3項新測試失敗（923通過）。全部reader退休後保留來源／日誌，再補明確import及frame boundary maintain，V3全926成功。buildV1及V2均成功，正式backend沿V1；兩者間僅cfg(test)fixture修正，正式結算來源未再改動。

## 邊界與接續

這是每個召喚實體的結算接受，不是整個formation的資源原子回滾。若一組中後段因其他outcome改變場景而拒絕，整組rollback仍pending，不能因本批勾完。未修改ABI、英雄名稱特判、平衡值或現有召喚生命周期政策。一般技能channels／zones、完整combat replica、更多原創內容／美術／音效及11項外部接受仍pending，MVP33/44不變。

錯誤詳見2026-10-23-summon-settlement-errors.md與E1506。48h omoba-48沿原2026-10-25 00:33:38 Taipei期限持續；未新增網路對局、quota reset、commit/push或omfx工作。證據：`../acceptance/2026-10-23-summon-settlement/index.json`。
