## Why

omfx 目前會依新設定的 20% 預設值或既有非零設定在啟動時播放 BGM，與遊戲應預設保持安靜的需求不符。啟動行為必須一致靜音，同時保留玩家在當次執行中自行開啟音樂的能力。

## What Changes

- 將新建與錯誤 fallback 的音樂音量預設值改為 `0.0`。
- 每次 omfx 啟動時，將載入後交給 runtime 的初始音樂音量正規化為 `0.0`，包含設定檔已存有非零音量的情況。
- 保留既有音樂滑桿、設定保存與其他設定欄位，不移除 BGM 資產或播放節點。
- 新增測試覆蓋新設定檔與既有非零設定檔的啟動靜音行為。

## Capabilities

### New Capabilities

- `omfx-start-muted`: 規範 omfx 啟動時的 BGM 靜音行為，以及玩家於當次執行中手動調整音量的相容性。

### Modified Capabilities

無。

## Impact

- 主要影響 `omfx/game/src/audio_settings.rs` 的預設值與 runtime 載入邊界。
- 不變更公開 API、設定檔格式、依賴、BGM 資產或 UI 控制方式。
- 既有設定檔中的非零音量仍可被保存，但不再造成下次啟動自動播放。
