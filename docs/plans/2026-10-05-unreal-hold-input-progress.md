# Unreal 通用 Hold 輸入接線（2026-10-05）

## 決定與實作

- 使用既有 OmRuntime 共用 FOmGameplayInputEvent／SubmitGameplayInputEvent，不新增英雄專屬 C++、BP graph，亦不修改 OmGenerated 輸出。WorldBridge 原有通用事件入口也可使用新增 action。
- 共用 EOmGameplayInputAction 附加 HoldPosition；C ABI enum 附加 InputHoldPosition=15，舊值不重新編號。Runtime 將事件轉成無目標命令並保留 queued。
- 原生 PlayerController H 為立即 Hold，Shift+H 為排隊 Hold。兩種 chord 明確綁定，使用目前 configured local player ID，HUD consumption guard／runtime 未啟動拒絕沿既有路徑。排隊 Hold 持續至後續非排隊命令取代，並非短暫 Stop。
- Rust C command→PlayerInput::HoldPosition→RendererInput::HoldPosition 保留 queued；後續沿既有 client owner／epoch admission 與後端正式命令。沒有自身位置 MoveTo workaround。
- 舊 embedded runtime 不支援此 MOBA 行為，與 Recall／shop 一樣在分配 input ID 前拒絕；錯誤訊息改成 MOBA commands，避免誤報 shop。
- ABI 12→13：Rust enum 新 discriminant 不能交給舊 DLL 解讀，因此 header 與 Rust 同步升級、舊版本拒絕。未 stage DLL，不能混用舊 Unreal native 與新 bridge；需解決 E177 後同批編譯部署。

## 當前確認

- `cargo test --manifest-path omfue/bridge/Cargo.toml --lib hold_position`：2 passed。涵蓋 queued false／true、C header 與 Rust ABI 一致、無目標驗證、ABI12拒絕、embedded 拒絕且 input ID／pending 不變、IPC 轉換保留 queue。
- 現有 GameplayInputSurface Editor automation 加入 enum reflection assertion；尚未執行，不能當成 UE 編譯或按鍵成功證據。
- 原生 H／Shift+H、HUD guard、權威 ACK／實際 Hold 的 UE 端到端仍待 E177 引擎基線阻擋解除。沒有重跑完整驗收、100 場或部署。
- C 槽空間問題沿 E194，只在本次 cargo 命令將 TEMP／TMP 指向既有 D 槽 debug 目錄，finally 還原，無刪除或系統設定變更。

## 待續

Unreal 安全編譯基線可用後，同批 ABI13 編譯／stage，確認 H、Shift+H 與 HUD consumption；完整框架仍20/30，維持未完成項，不以兩個 bridge 測試替代完整對局驗收。
