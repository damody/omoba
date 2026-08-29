## ADDED Requirements

### Requirement: omfx 啟動時 BGM 靜音
omfx MUST 在每次程序啟動時將 runtime 的初始 BGM 音量設為 `0.0`，不受設定檔中既有 `music_volume` 值影響。

#### Scenario: 沒有設定檔
- **WHEN** omfx 啟動且音訊設定檔不存在
- **THEN** 系統建立 `music_volume = 0.0` 的設定並以靜音啟動 BGM

#### Scenario: 既有設定為非零音量
- **WHEN** omfx 啟動且設定檔包含有效的非零 `music_volume`
- **THEN** runtime 的初始 BGM 音量為 `0.0`
- **THEN** 系統不因啟動靜音而覆寫該既有設定值或其他設定欄位

#### Scenario: 設定讀取失敗
- **WHEN** omfx 無法取得或解析音訊設定
- **THEN** fallback 的初始 BGM 音量為 `0.0`

### Requirement: 當次執行可手動開啟 BGM
omfx MUST 保留既有音樂滑桿與保存流程，讓玩家能在啟動後於當次執行中設定非零音量。

#### Scenario: 玩家啟動後調整音量
- **WHEN** 玩家在設定介面將音樂滑桿由零調為非零
- **THEN** BGM 於當次執行中使用玩家選擇的音量
- **THEN** 下一次程序啟動仍以 `0.0` 作為初始 BGM 音量
