## TD UI 圖片生圖提示詞：甜點戰爭

這份文件給企劃逐張產生 `scripts/base_content/assets/td_ui/` 的預設圖或正式圖。請把產出的 PNG 以對應檔名覆蓋到 scripts asset 目錄，不要放到 `omfx/data/td_ui/`。

主題是「甜點戰爭」：糖果、餅乾、蛋糕、巧克力、奶油、冰淇淋、果醬、糖霜與玩具感武器組成的可愛塔防戰場。風格要原創，不要複製任何既有遊戲素材。

## 通用規則

所有圖片共同要求：

| 項目 | 要求 |
|---|---|
| 風格 | 甜點戰爭、Q 版、亮色、厚描邊、糖果質感、玩具感武器、乾淨可讀 |
| 邊框 | 每張圖都必須有清楚可見的外框。panel/card 是完整 UI frame；icon 類也要有糖果徽章、圓形底座或厚描邊輪廓，讓人一眼看出圖片範圍 |
| 文字 | 圖片內不要放文字、數字、商標、浮水印，UI 文字會由遊戲程式疊上去 |
| 背景 | icon 類圖片用透明背景；panel/card 類圖片外框外也要透明 |
| 輸出 | PNG，保留 alpha channel |
| 構圖 | 主體置中，四周留 8-12% 安全邊界，不要裁切 |
| 負面提示 | no text, no letters, no numbers, no watermark, no logo, no copyrighted character, no photorealism, no messy background, no existing game asset copy |

建議先把「通用風格前綴」貼在每張提示詞前面：

```text
Original cute dessert-war tower-defense UI asset, candy battlefield theme, frosting, cookies, chocolate, jelly, cream, sprinkles, toy-like weapons, chunky rounded shapes, clear visible border/frame on the whole asset, thick dark outline, glossy highlights, saturated pastel colors, clean readable silhouette, mobile game icon quality, transparent PNG with alpha, centered composition, no text, no numbers, no watermark, no logo, do not copy any existing game assets.
```

## 面板與卡片背景

| 檔名 | 尺寸 | 用途 | 提示詞 |
|---|---:|---|---|
| `panel_left.png` | 320x760 | 左側選中塔/升級/出售面板背景 | A tall vertical dessert-war upgrade side panel frame made of layered wafer cookies and caramel wood, frosting trim, candy rivets, rounded corners, empty slots for upgrade cards, transparent outside the panel, no text. |
| `panel_right.png` | 300x920 | 右側買塔/開始暫停面板背景 | A tall vertical dessert-war shop side panel frame made of wafer boards, chocolate edges, frosting seams and candy buttons, space for tower cards and bottom control buttons, transparent outside the panel, no text. |
| `shop_card.png` | 128x138 | 一般買塔格背景 | A small glossy dessert shop card frame, blue sugar-glass center, cookie border, frosting highlights, darker syrup strip area at bottom for price overlay, transparent outside the card, no text. |
| `shop_card_selected.png` | 128x138 | 已選買塔格背景 | A selected dessert shop card frame, golden caramel glow border, frosting sparkle, blue sugar-glass center, cookie rim, transparent outside the card, no text. |
| `shop_card_locked.png` | 128x138 | 錢不夠/鎖定買塔格背景 | A locked dessert shop card frame, muted gray-blue sugar glass, stale cookie border, subtle candy lock decoration without text, transparent outside the card, no text. |

## 通用操作圖示

| 檔名 | 尺寸 | 用途 | 提示詞 |
|---|---:|---|---|
| `tower_fallback.png` | 128x128 | 未知塔種 fallback | A generic dessert-war mystery tower placeholder, cupcake-shaped tower silhouette with candy question-like swirl shape but no actual text, frosting top, neutral cookie and cream colors, transparent background. |
| `sell.png` | 96x96 | 左側出售按鈕圖 | A dessert-war sell icon, orange-red candy price tag with caramel coin and frosting shine, friendly action button feeling, transparent background, no text. |
| `start_round.png` | 96x96 | 右側開始回合圖 | A bright green candy-gloss play button for starting a wave, white frosting triangular play symbol as simple shape only, sugar sparkle, transparent background, no text. |
| `pause.png` | 96x96 | 右側暫停 placeholder 圖 | A cyan-blue candy-gloss pause button, two vertical white frosting pause bars as simple shapes, slightly disabled placeholder feeling, transparent background, no text. |

## 塔圖示

| 檔名 | 尺寸 | 用途 | 提示詞 |
|---|---:|---|---|
| `tower_dart.png` | 128x128 | 飛鏢猴買塔格與選中塔大圖 | A cute dessert-war dart tower character, cookie soldier mascot throwing a candy cane boomerang, frosting helmet, jelly eyes, heroic pose, thick outline, transparent background, no text. |
| `tower_bomb.png` | 128x128 | 炸彈射手買塔格與選中塔大圖 | A dessert-war bomb cannon tower, chunky chocolate cannon on biscuit wheels, firing round truffle bombs with cream fuse, playful toy weapon silhouette, thick outline, transparent background, no text. |
| `tower_tack.png` | 128x128 | 鐵釘射手買塔格與選中塔大圖 | A dessert-war tack shooter tower, round tart turret that shoots sugar spikes and candy sprinkles, metallic candy wrapper accents, compact radial weapon silhouette, thick outline, transparent background, no text. |
| `tower_ice.png` | 128x128 | 冰凍猴買塔格與選中塔大圖 | A dessert-war ice tower, cute popsicle mage or ice cream turret, blue shaved-ice body, snow sugar crystals, frosty cream swirl, thick outline, transparent background, no text. |

## 共用升級路線 fallback 圖

| 檔名 | 尺寸 | 用途 | 提示詞 |
|---|---:|---|---|
| `upgrade_p1.png` | 96x96 | Path 1 fallback | A dessert-war upgrade path icon for range or precision, green candy badge with frosting arrow and sugar target reticle, thick outline, transparent background, no text. |
| `upgrade_p2.png` | 96x96 | Path 2 fallback | A dessert-war upgrade path icon for speed, green candy badge with fast jelly streaks and tiny candy projectile, thick outline, transparent background, no text. |
| `upgrade_p3.png` | 96x96 | Path 3 fallback | A dessert-war upgrade path icon for utility or special effect, green candy badge with branching licorice arrow, sparkle sprinkles and frosting burst, thick outline, transparent background, no text. |

## Dart Tower 專屬升級圖

| 檔名 | 尺寸 | 用途 | 提示詞 |
|---|---:|---|---|
| `tower_dart_p1.png` | 96x96 | Dart Path 1 | A dessert-war dart upgrade icon focused on long range, candy cane boomerang flying far with syrup trail over a green candy badge, transparent background, no text. |
| `tower_dart_p2.png` | 96x96 | Dart Path 2 | A dessert-war dart upgrade icon focused on fast throwing, multiple candy darts launched rapidly with jelly motion streaks over a green candy badge, transparent background, no text. |
| `tower_dart_p3.png` | 96x96 | Dart Path 3 | A dessert-war dart upgrade icon focused on sharp vision or precision, cookie soldier eye with sugar target sparkle and candy dart over a green candy badge, transparent background, no text. |

## Bomb Tower 專屬升級圖

| 檔名 | 尺寸 | 用途 | 提示詞 |
|---|---:|---|---|
| `tower_bomb_p1.png` | 96x96 | Bomb Path 1 | A dessert-war bomb upgrade icon focused on bigger blast radius, chocolate truffle bomb exploding into whipped cream cloud and sprinkles over a green candy badge, transparent background, no text. |
| `tower_bomb_p2.png` | 96x96 | Bomb Path 2 | A dessert-war bomb upgrade icon focused on faster missiles, wafer rocket with caramel flame trail over a green candy badge, transparent background, no text. |
| `tower_bomb_p3.png` | 96x96 | Bomb Path 3 | A dessert-war bomb upgrade icon focused on fragments, chocolate cannonball splitting into many hard candy shards over a green candy badge, transparent background, no text. |

## Tack Tower 專屬升級圖

| 檔名 | 尺寸 | 用途 | 提示詞 |
|---|---:|---|---|
| `tower_tack_p1.png` | 96x96 | Tack Path 1 | A dessert-war tack upgrade icon focused on more spikes, radial burst of sugar spikes and candy sprinkles from a tart turret over a green candy badge, transparent background, no text. |
| `tower_tack_p2.png` | 96x96 | Tack Path 2 | A dessert-war tack upgrade icon focused on fire ring, circular caramel flame ring around a small tart turret over a green candy badge, transparent background, no text. |
| `tower_tack_p3.png` | 96x96 | Tack Path 3 | A dessert-war tack upgrade icon focused on spinning blades, peppermint candy saw blade whirl with syrup motion blur over a green candy badge, transparent background, no text. |

## Ice Tower 專屬升級圖

| 檔名 | 尺寸 | 用途 | 提示詞 |
|---|---:|---|---|
| `tower_ice_p1.png` | 96x96 | Ice Path 1 | A dessert-war ice upgrade icon focused on deep freeze, blue popsicle crystal trapping a small gummy target silhouette over a green candy badge, transparent background, no text. |
| `tower_ice_p2.png` | 96x96 | Ice Path 2 | A dessert-war ice upgrade icon focused on icy aura, circular powdered-sugar snowflake aura expanding from ice cream turret over a green candy badge, transparent background, no text. |
| `tower_ice_p3.png` | 96x96 | Ice Path 3 | A dessert-war ice upgrade icon focused on icicle projectile, long sharp rock-candy icicle spear flying forward with frost sugar trail over a green candy badge, transparent background, no text. |

## 產圖後檢查清單

| 檢查 | 通過條件 |
|---|---|
| 檔名 | 必須完全符合表格檔名 |
| 格式 | PNG with alpha |
| 邊框 | 每張圖都有明確外框、糖果徽章底座或厚描邊輪廓，可直接看出圖片範圍 |
| 透明度 | icon 外圍透明，panel/card 外框外透明 |
| 文字 | 圖內不可有文字，避免和遊戲 UI 疊字衝突 |
| 主題 | 必須符合甜點戰爭，不要變成一般軍事、一般猴子、寫實槍械或科幻 UI |
| 構圖 | 主體沒有被裁切，小尺寸下仍能辨識 |
| 版權 | 不含既有遊戲角色、商標或可識別受保護素材 |
