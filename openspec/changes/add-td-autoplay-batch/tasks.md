## 1. Lua 入口實作

- [x] 1.1 新增 `scripts/test_td_1_to_100.lua`，透過共用 bootstrap 從自身位置解析 repository 根目錄
- [x] 1.2 加入 release `base_content` 建置閘門，建置失敗時立即傳回非零狀態
- [x] 1.3 加入單次 `layered_td_coarse_autoplay_completes_rounds_1_to_100` 測試命令並傳遞 exit code

## 2. 驗證

- [x] 2.1 以 `D:\code\omoba\tools\lua\lua.exe scripts\test_td_1_to_100.lua` 完成第 1 至 100 關測試
- [x] 2.2 確認 Git 狀態未包含 DLL、`target/`、log 或 failure report 等暫存產物
