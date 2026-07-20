# 塔技能列垂直位移實作計畫

> **給代理工作者：** 必須使用子技能 `superpowers:subagent-driven-development`（建議）或 `superpowers:executing-plans`，逐項執行本計畫。每個步驟以核取方塊（`- [ ]`）追蹤。

**目標：** 將畫面底部中央的塔技能列固定向上移動 70 個螢幕像素。

**架構：** 保留既有的底部錨定排版，將隱含的 18 px 底部間距改為具名的 88 px 底部間距。把 Y 座標計算抽成純函式，讓固定間距與極小視窗的邊界限制可以在不建立 Fyrox UI 的情況下進行回歸測試。

**技術棧：** Rust 1.95.0、Fyrox UI、Rust 單元測試

---

### 任務 1：移動並測試塔技能列

**檔案：**
- 修改：`omfx/game/src/native.rs:10329-10375`
- 測試：`omfx/game/src/native.rs` 內嵌的 `#[cfg(test)]` 模組

- [ ] **步驟 1：先撰寫會失敗的排版測試**

在內嵌測試模組加入以下斷言：

```rust
#[test]
fn tower_ability_bar_uses_eighty_eight_pixel_bottom_margin() {
    assert_eq!(tower_ability_bar_y(1080.0, 88.0), 904.0);
    assert_eq!(tower_ability_bar_y(720.0, 88.0), 544.0);
    assert_eq!(tower_ability_bar_y(120.0, 88.0), 0.0);
}
```

- [ ] **步驟 2：執行聚焦測試，確認 RED 階段**

執行：

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx tower_ability_bar_uses_eighty_eight_pixel_bottom_margin -- --nocapture
```

預期結果：因為 `tower_ability_bar_y` 尚未存在，編譯失敗。

- [ ] **步驟 3：加入最小幅度的底部間距實作**

在既有技能列排版輔助函式附近加入排版常數與純函式：

```rust
const TOWER_ABILITY_BAR_BOTTOM_MARGIN: f32 = 88.0;

fn tower_ability_bar_y(window_height: f32, slot_height: f32) -> f32 {
    (window_height - slot_height - TOWER_ABILITY_BAR_BOTTOM_MARGIN).max(0.0)
}
```

將 `update_tower_ability_bar_ui` 裡的行內 Y 座標計算替換為：

```rust
let y = tower_ability_bar_y(self.window_size.y, slot_h);
```

上一頁／下一頁控制仍以 `y - 40.0` 定位；點擊區域與 tooltip 繼續使用計算後的技能格矩形，因此會與技能列一起移動。

- [ ] **步驟 4：執行聚焦測試與完整前端測試**

執行：

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx tower_ability_bar_uses_eighty_eight_pixel_bottom_margin -- --nocapture
cargo test --manifest-path omfx/Cargo.toml -p omfx
```

預期結果：聚焦測試與全部 `omfx` 測試皆通過，失敗數為零。

- [ ] **步驟 5：檢查格式並編譯檢查 executor**

執行：

```powershell
rustfmt --edition 2021 --check omfx/game/src/native.rs
cargo check --manifest-path omfx/Cargo.toml -p executor --features runtime-lua-content
```

預期結果：兩個指令皆成功結束。

- [ ] **步驟 6：先提交 `omfx` submodule，再提交主 repo 指標與計畫**

```powershell
git -C omfx add -- game/src/native.rs
git -C omfx commit -m "fix(ui): raise tower ability bar"
git add -- omfx docs/superpowers/plans/2026-07-14-tower-ability-bar-offset.md
git commit -m "chore: bump omfx for raised tower ability bar"
```
