# 權威動畫暫停狀態接軌

## 計畫與決定

1. 不從兩個相同進度快照猜暈眩；權威 hero_tick 明確標記 animation timing 是否 paused，stun／零 dt 凍結，正常更新解除。
2. AVS2 固定 26 bytes，公開 sequence／phase／paused／elapsed／duration；strict 0／1 與 canonical idle，不傳 Buff 身分、來源、目標或暴擊。
3. 沿既有安全 baseline／absolute update／filtered／IPC 路徑讀取，bridge animation record flags bit 2 表示 playback paused，snapshot global pause 可同樣傳遞。
4. 原生 animation 與 attack-phase payload 都帶 pause；播放端先定位權威游標再 SetPlaying(false)／rate 0，下一個正常狀態恢復，不改 gameplay counter 或時間。
5. selective wire5 拒絕4，C ABI14拒絕13，C header同步；IPC framing4與frame record layout不變。最後部署須同步重建 server／client／bridge／native，不接受舊元件靜默忽略新語意。

## 局部結果

- core `cargo test --manifest-path omoba-core/Cargo.toml --features compiled-content-only attack_visual_state`：4 passed，AVS2 codec／paused roundtrip／非法 bool／AVS1拒絕、absolute update、可見投影、wire拒絕4。
- bridge `cargo test --manifest-path omfue/bridge/Cargo.toml attack_visual_state`：2 passed，safe component與paused／解除／global pause flags投影，權威進度不變。
- bridge `attack_visual_state_requires_pause_aware_abi`：1 passed，ABI14正常／13拒絕；首次 E0133 已修測試 unsafe 區塊，不改 validator。
- scripts `cargo test --manifest-path scripts/Cargo.toml -p base_content --features compiled-content-only attack_visual_state`：2 passed，真實60Hz解析攻速，以及真實攻擊前搖→stun三步的進度／序號保持及公開paused event→解除後前進。
- 共9項不同局部測試。既有 dead_code warning 保持，沒有新的 production 編譯錯誤。
- native scoped OmRuntime＋OmGenerated＋OmEditor：22 actions／11.77秒，Succeeded；日誌 `omfue/Saved/Logs/native-animation-pause-state-modules-20261005.log`。未執行 Editor 或原生畫面斷言。
- 正式 codegen `--check`：15 files／17 Lua inputs，content_hash b348872bbe367985，generator_version5與data／identity hash保持；Lua作者來源未變、遊戲不執行 Lua。
- 沒有部署、stage binary、改 BuildId、完整對局／LAN／Editor驗收或 omfx 維護；21/31、6.1整項保持未完成。
