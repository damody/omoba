# 正式編譯內容與 headless 批次工具

## 決定與實作

- `run_moba_runtime_smoke.lua` 原先明確啟用 runtime Lua，不能作為 Lua 生成／Rust 執行的正式證據。改沿共用 `moba_compiled_content.lua`：三個 build 使用 `compiled-content-only`，生成 TOML 禁用 Lua／熱重載，程序環境清空 Lua 載入來源。商店呈現頻率不再偷偷降到30Hz，使用選定的 tick profile。此 smoke 仍是測試注入入口，不冒充真人操作或 Unreal 對局。
- `run_2player_ue.lua` 已使用編譯feature與禁用Lua環境；同樣沿共用契約把生成TOML禁用Lua／熱重載，商店呈現頻率跟tick profile一致。Grok第2批稽核指出的30Hz限制已在這次工具修正移除；以前的30Hz證據不會因此變成60Hz成功。
- 新 `run_moba_headless_batch.lua` 凍結一次 Lua 作者配方為 JSON、release 建置一次、正式60Hz逐種子跑獨立程序。每場保留獨立 JSON與log；彙總記錄成功場數、種子、結束 tick、重播範圍、地圖及digest。失敗停止、保留已有證據並產生errors.md，禁止低頻／withdraw／plan-only fallback。
- 新 `moba_headless_batch.lua` 可注入執行邊界作局部測試。彙總 success 只表示 headless 與重播契約，不宣稱 Unreal、視野、LAN 或完整驗收成功。沒有獨立越權／非法目標計數，這個證據限制明確記在報告。

## 指令（尚未執行正式100場）

```bat
tools\lua\lua.exe scripts\run_moba_headless_batch.lua --matches 100 --seed 1 --recipe scripts/lua_data/moba_archetype_match.lua
```

預設建立 `omb/target/moba-headless-batches` 下新的獨占目錄；`--output` 必須是未存在的新目錄，拒絕覆蓋歷史證據。此輪只確認工具功能，不執行正式對局。

## 局部錯誤與預防

- E302（工具格式斷言）：TOML host 重寫會調整等號空白，原先測試比對 `LUA_CONTENT=false` 字串失敗。改驗證解析後 boolean與保留欄位值；不修改TOML serializer以迎合測試。
- 同輪新接線測試使用 `tests/../?.lua` 載入_bootstrap，其root辨識依賴實際source字尾，未正規化的`..`導致root錯誤。改直接定位scripts父目錄再載入；固定bootstrap不因測試去改。PowerShell接續第二指令會掩蓋前一指令exit code，因此測試以獨立tool結果核對。
- 查找路徑錯誤：PowerShell下直接把 `scripts/*.lua` 或猜的 `moba_headless*.lua` 交給rg會造成os error123，猜`stories.lua`不存在會os error2。今後先用 `rg --files` 定位，再用 `rg -g '*.lua' scripts`，不用猜路徑或把glob當檔名。

驗證使用固定Lua的 `scripts/tests/moba_headless_batch_test.lua`；覆蓋選項界限／重複種子溢位、60Hz正式證據限制、build-once、獨立報告名稱、失敗保留與停止、編譯環境／TOML及smoke接線。未跑Cargo、server/runtime、UE、100場或全面驗收。正式smoke執行結果仍待最後驗收。

最終局部結果：6/6通過（exit0），CLI `--help`成功。新增CLI与兩個smoke經loadfile語法確認；未啟動遊戲。
