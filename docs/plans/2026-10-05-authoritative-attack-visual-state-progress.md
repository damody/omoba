# 權威攻擊動畫狀態接軌

## 計畫與決定

1. 使用 hero_tick 實際 Buff 聚合後的 effective_interval 與 attack_phase_durations，保存解析時間，不變更攻擊判定。
2. AVS1 固定 25 bytes，限制合法階段、Q10 時間範圍及 canonical idle；不包含目標、私有修正來源或猜測暴擊。
3. MOBA 可見英雄 baseline 與 AttackVisual absolute event 共用狀態；舊 selective wire 3 明確拒絕，新 wire 4 使用共用版本常數。
4. filtered consumer 僅更新已有 entity／component；bridge 解碼後投影至既有 C ABI 13 animation record，native 相位播放端不需新增角色 C++／Blueprint。
5. 當前功能局部確認，完整對局／LAN／Unreal 畫面驗收留最後。

## 邊界

- 只提供前搖／後搖持續狀態。實際命中 cue 獨立，沒有產生假命中或暴擊。
- Lua 仍只在建置時生成。新 metadata 不序列化至 TAttack JSON，不修改 script ABI。
- 權威 idle 阻止 legacy FX 重播；未知 timing 不猜時長。state validation／decode 不額外配置暫存 buffer。
- 不部署 DLL、不啟動 Editor、不更改 BuildId，不處理 omfx。

## 局部結果

- core `cargo test --manifest-path omoba-core/Cargo.toml --features compiled-content-only attack_visual_state`：最終 4 passed，codec／resolved timing／clear、absolute replay／缺少 baseline／malformed／未知 entity、可見敵方／idle／隱藏停止更新、wire 4／拒絕 3。
- bridge `cargo test --manifest-path omfue/bridge/Cargo.toml attack_visual_state`：2 passed，safe component 解碼、未知 Hz 仍依權威 Q10 時間產生 Windup／Recovery／idle 與穩定 instance，沒有 target／critical。
- source `cargo test --manifest-path scripts/Cargo.toml -p base_content --features compiled-content-only attack_visual_state`：1 passed；真實 generated manifest＋Production60Hz 執行 120 ticks，使用正常 BuffStore stat 聚合加入 +100 attack-speed points，實際解析 interval 為 base／2，兩階段都由正式 team projection 發布。
- 共 7 項不同局部測試。首次來源測試 E0609 已修為 PaddedTeamFrame.frame.step，不改 production schema；修復後通過。既有 dead_code warning 未當成新增失敗。
- OpenSpec strict、主 repo 與 omfue whitespace 檢查通過。本輪沒有部署、Editor 或整體對局驗收；新規格 wire 4 需最後部署時同步更新 server／client，21/31、6.1 未完成狀態保持。
