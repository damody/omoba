## Context

omfx 透過 `AudioSettings::load_or_create` 載入 `config.toml`，並以回傳的 `music_volume` 建立循環播放的 BGM node。目前新設定預設為 `0.2`，既有非零設定也會直接套用，因此啟動時可能自動播放音樂。`omfx/game/src/native.rs` 另有進行中的修改，本變更應將邊界留在獨立的 `audio_settings.rs`。

## Goals / Non-Goals

**Goals:**

- 新建設定、讀取失敗與既有非零設定皆以 `0.0` 作為本次啟動音量。
- 保留玩家在當次執行中使用滑桿開啟 BGM 的能力。
- 保留設定檔格式、既有值與其他欄位。

**Non-Goals:**

- 移除 BGM node、音訊資產或音樂滑桿。
- 永久禁止 BGM。
- 改寫其他音效設定。

## Decisions

1. 將 `DEFAULT_MUSIC_VOLUME` 改為 `0.0`，使新建設定與錯誤 fallback 一致靜音。
2. 在 `AudioSettings::load_or_create` 的 runtime 邊界套用啟動靜音正規化，而不改變 `load_or_create_at` 的持久化讀取語意。這可保留既有設定及其單元測試能力，也不必修改已有其他變更的 `native.rs`。
3. 不在啟動時將既有設定檔覆寫成零。替代方案是永久清除偏好，但會造成不必要的資料損失；保留磁碟值仍可讓當次手動調整與保存流程維持相容。

## Risks / Trade-offs

- [設定檔顯示非零但啟動仍靜音，可能看似不直覺] → 以明確命名的啟動正規化函式與測試說明這是產品規則。
- [未來其他呼叫者誤用較低階 helper] → helper 維持 module-private，正式 runtime 入口仍是 `load_or_create`。

## Migration Plan

不需設定遷移。部署後每次啟動即套用靜音；回滾程式碼即可恢復依設定值啟動的舊行為。

## Open Questions

無。
