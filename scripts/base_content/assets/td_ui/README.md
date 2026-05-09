# TD UI Assets - 甜點戰爭

此目錄是 TD UI 圖片的權威來源。企劃要替換右側買塔、左側升級/出售、開始/暫停等圖片時，請直接替換本目錄同名 PNG。

不要把正式圖放到 `omfx/data/td_ui/`。前端只負責讀取與顯示，正式 content 圖片由 `scripts/base_content` 擁有。

## 替換規則

- 保留相同檔名。
- 使用 PNG，保留 alpha channel。
- 每張圖都要有清楚外框、徽章底座或厚描邊輪廓，讓人看得出圖片範圍。
- icon 外圍要透明；panel/card 外框外也要透明。
- 圖片內不要放文字、數字、商標或浮水印，文字由遊戲 UI 疊上去。
- 主題維持「甜點戰爭」：糖果、餅乾、蛋糕、奶油、巧克力、冰淇淋、糖霜、果醬、玩具感武器。

## 檔名契約

| 檔名 | 建議尺寸 | 用途 |
|---|---:|---|
| `panel_left.png` | 320x760 | 左側選中塔、升級、出售面板背景 |
| `panel_right.png` | 300x920 | 右側買塔、Start/Pause 面板背景 |
| `shop_card.png` | 128x138 | 一般買塔格背景 |
| `shop_card_selected.png` | 128x138 | 已選買塔格背景 |
| `shop_card_locked.png` | 128x138 | 錢不夠或鎖定狀態買塔格背景 |
| `tower_fallback.png` | 128x128 | 未知 tower kind fallback 圖 |
| `tower_dart.png` | 128x128 | Dart tower 買塔格與選中塔圖 |
| `tower_bomb.png` | 128x128 | Bomb tower 買塔格與選中塔圖 |
| `tower_tack.png` | 128x128 | Tack tower 買塔格與選中塔圖 |
| `tower_ice.png` | 128x128 | Ice tower 買塔格與選中塔圖 |
| `upgrade_p1.png` | 96x96 | Path 1 共用升級 fallback 圖 |
| `upgrade_p2.png` | 96x96 | Path 2 共用升級 fallback 圖 |
| `upgrade_p3.png` | 96x96 | Path 3 共用升級 fallback 圖 |
| `tower_dart_p1.png` | 96x96 | Dart tower Path 1 專屬升級圖 |
| `tower_dart_p2.png` | 96x96 | Dart tower Path 2 專屬升級圖 |
| `tower_dart_p3.png` | 96x96 | Dart tower Path 3 專屬升級圖 |
| `tower_bomb_p1.png` | 96x96 | Bomb tower Path 1 專屬升級圖 |
| `tower_bomb_p2.png` | 96x96 | Bomb tower Path 2 專屬升級圖 |
| `tower_bomb_p3.png` | 96x96 | Bomb tower Path 3 專屬升級圖 |
| `tower_tack_p1.png` | 96x96 | Tack tower Path 1 專屬升級圖 |
| `tower_tack_p2.png` | 96x96 | Tack tower Path 2 專屬升級圖 |
| `tower_tack_p3.png` | 96x96 | Tack tower Path 3 專屬升級圖 |
| `tower_ice_p1.png` | 96x96 | Ice tower Path 1 專屬升級圖 |
| `tower_ice_p2.png` | 96x96 | Ice tower Path 2 專屬升級圖 |
| `tower_ice_p3.png` | 96x96 | Ice tower Path 3 專屬升級圖 |
| `sell.png` | 96x96 | 左側出售按鈕圖 |
| `start_round.png` | 96x96 | 右側開始回合按鈕圖 |
| `pause.png` | 96x96 | 右側暫停 placeholder 圖 |

## 生圖提示詞

逐張生圖提示詞位於：`openspec/changes/script-owned-td-ui-assets/asset-prompts.md`。
