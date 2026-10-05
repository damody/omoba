# Mana 生命週期與 Lua 再生規則

## 本批決定與實作

- `SingleLaneConfig::mana_enabled` 明確啟用，預設 `false`；不以 `GameMode::Moba` 判斷，保留既有 TD／舊 Story。尚未在一般 server／LAN 啟用，須先完成版本協商與 HUD 接線。
- 正式 MOBA 上限採 Lua `base_mana + mana_per_level × (level - 1)`，不沿用舊的 `75 + intelligence × 13`。共用 `Hero::moba_mana_capacity` 使用 i128，要求等級 1–25、容量 0–1000000；開局在建立實體前檢查每名英雄的所有等級。舊初始化／上限 API 不變。
- 出生與新身分復活建立滿池，清除再生餘數；復活保留等級，但不沿用死亡時的法力餘額。升級只更新容量、不額外補魔。
- Lua `moba_economy.mana_regen_per_second = 5` 經既有 build-time 流程生成常數；省略欄位預設零、拒絕負值／小數／字串，載入時限制不超過 10000。
- 再生在權威 post-combat 階段使用與收入相同的有效時間，只處理存活且非 lethal_pending 的英雄。暖機、暫停、死亡／等待復活、結束不再生。`ManaPool` 整數再生保存餘數並限制容量；take delta 防止重複結算。
- 規則進入完整 Lua canonical data hash；runtime compiled mismatch 與 hot reload 拒絕單邊改動。UE 生成器同步來源 manifest，不手改生成的 C++。
- 本批僅接共用基礎再生率。Buff／裝備的再生倍率、explicit script spend／restore、一般網路規則協商、Mana HUD 尚未完成，不冒充完整 Mana 系統。

## 當前功能確認

- base_content `mana_lifecycle`：3/3，正式 60Hz 出生／有效時間／暫停／升級／結束，以及暖機／預設停用／死亡與新身分復活。
- 上述第三項短 filtered fixture：12 ticks、雙隊 24 applied steps，每 tick 完整 canonical hash 對 fresh bootstrap 相同、零 ComponentRepair；正常 owner accepted input 重演、自我治療扣 45 再加正式再生，餘數保存。
- core 容量測試：1/1，Lua 基礎值、智力不影響、25 級成長、溢位／非法等級／未知英雄拒絕。
- template-id 規則檢查：2/2，hash 改變、compiled mismatch、禁止 hot reload、欄位型別與 legacy omission；伴隨 base_content 篩選 0 tests 只代表编譯成功，不列入通過數。
- codegen 生成與 `--check`：11 files、17 Lua inputs，presentation content hash `f4f3b7871388ed77`；完整 catalog data hash 在生成 manifest 同步。presentation subset 未改不代表整體 Lua data hash 未改。

## 錯誤與界線

錯誤記錄於 `unreal-moba-error-register.md` E176。第一次測試編譯錯誤已修正；沒有放寬 private API。此輪不執行 OmGame 建置、DLL stage、MCP／PIE、長對局、實際 KCP／LAN 或全套驗收。整體仍 20/30，未勾選 5.5 或 6.2。

本批來源 whitespace diff 檢查通過；既有 td_rounds dead-code warnings 保留，沒有把無關警告當成本批錯誤。
