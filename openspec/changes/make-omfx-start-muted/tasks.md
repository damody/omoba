## 1. 啟動靜音實作

- [x] 1.1 將 `DEFAULT_MUSIC_VOLUME` 改為 `0.0`，並更新 fallback log 的預設音量說明
- [x] 1.2 在 `AudioSettings::load_or_create` 邊界將新建、既有與 fallback 設定的 runtime 初始音量正規化為 `0.0`

## 2. 測試與驗證

- [x] 2.1 更新新建設定測試，確認設定檔寫入 `music_volume = 0.0`
- [x] 2.2 新增既有非零設定在 runtime 啟動時靜音且不覆寫磁碟內容的單元測試
- [x] 2.3 執行格式化、`audio_settings` 針對性測試與 OpenSpec strict validation
