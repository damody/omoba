# 通用原生播放倍率接軌

## 計畫與決定

1. 從 Lua ue.animation.default_play_rate 生成 NativeDefaultPlayRate，缺省 1；不用角色 ID 分支。
2. 普通及缺少合法權威 phase metadata 的 legacy 動畫共用 ResolveNativePlayRate，預設／當前倍率各自檢查有限正值，再使用 double 相乘、限制 0.01–10；非法參數各自退回 1。
3. 權威攻擊相位的 duration 決定游標插值速度，不讓美術或 state 倍率再次修改；Impact 明確 rate 0 暫停。
4. 只確認目前功能，完整素材／對局／Editor 驗收留最後。

## 局部結果

- `cargo test --manifest-path omfue/codegen/Cargo.toml native_animation_default_rate`：1 passed，任意英雄 default／0.25／1.25／2.0 生成與零／負數／溢位／下溢拒絕。
- 正式生成及 `--check`：15 files／17 Lua inputs 通過；generator_version 5、content_hash b348872bbe367985。
- data hash 2027e0ada2f76866／identity hash ff3ef5e2957aa89f 維持，作者 Lua 內容未變。
- 原生 automation 新增倍率合併、上下限、極端值／非法值斷言；僅模組編譯確認，不宣稱執行過這些斷言。
- scoped OmRuntime＋OmGenerated＋OmEditor：16 actions／10.86 秒，Succeeded；日誌 `omfue/Saved/Logs/native-animation-default-rate-modules-20261005.log`。
- 沒有 gameplay／IPC／C ABI 改動、部署／stage binary／Editor 啟動或 omfx 維護；21/31 與 6.1 整項未完成保持。
