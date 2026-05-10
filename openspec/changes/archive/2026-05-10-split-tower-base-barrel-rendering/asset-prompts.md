## Tower Combat 圖片生圖提示詞：甜點戰爭

這份文件給企劃逐張產生 `scripts/base_content/assets/towers/` 的戰鬥畫面 tower composite 圖。產出的 PNG 請以對應檔名放進 scripts asset 目錄，保留透明背景與 alpha channel。

主題是「甜點戰爭」：糖果、餅乾、蛋糕、巧克力、奶油、冰淇淋、果醬、糖霜與玩具感武器組成的可愛塔防戰場。風格要原創，不要複製任何既有遊戲素材。

## 通用規則

所有 combat tower 圖片共同要求：

| 項目 | 要求 |
|---|---|
| 風格 | 甜點戰爭、Q 版、亮色、厚描邊、糖果質感、玩具感武器、乾淨可讀 |
| 背景 | 透明背景，PNG with alpha |
| 文字 | 圖片內不要放文字、數字、商標、浮水印 |
| 構圖 | 主體置中，四周留 10% 安全邊界，不要裁切 |
| 視角 | 俯視或 3/4 top-down tower defense sprite，適合放在戰鬥地圖上 |
| 尺寸 | 建議 256x256，實作可再縮放；base 與 barrel 同尺寸較容易對齊 |
| Base 圖 | 只畫不旋轉的底座、輪子、支架、底盤、甜點平台，不要畫明顯朝向性的砲管尖端 |
| Barrel 圖 | 只畫可旋轉或可獨立表演的上層攻擊部件，透明背景；target-facing 圖預設朝上；fixed 針塔要做成接近放射對稱 |
| Animation frame | 連續圖要保持同尺寸、同視角、同中心點；frame 01 是 idle/蓄勢，後續逐步變成發光、蓄力、爆發或回復 |
| 負面提示 | no text, no letters, no numbers, no watermark, no logo, no copyrighted character, no photorealism, no messy background, no existing game asset copy |

## Dart Tower

### `tower_dart_base.png`

用途：飛鏢塔底座，不旋轉。

```text
Create an original cute dessert-war tower-defense combat sprite for tower_dart_base.png, transparent PNG with alpha, 256x256, centered composition, 3/4 top-down view. Draw only the non-rotating base of a dart tower: a round butter cookie platform with frosting rim, wafer support legs, jelly candy bolts, small sprinkle decorations, sturdy toy-like dessert battlefield base. Do not include the dart launcher, cannon barrel, arrow, projectile, character face, text, letters, numbers, logo, watermark, or background. Use saturated pastel candy colors, thick dark outline, glossy frosting highlights, clean readable silhouette, mobile game sprite quality, original design, do not copy any existing game asset.
```

### `tower_dart_barrel.png`

用途：飛鏢塔砲口/投擲器，可旋轉，預設朝上。

```text
Create an original cute dessert-war tower-defense combat sprite for tower_dart_barrel.png, transparent PNG with alpha, 256x256, centered composition, 3/4 top-down view. Draw only the rotating upper attack part of a dart tower, default facing upward: a candy-cane dart launcher arm mounted on a small frosting swivel, with a striped sugar dart or boomerang-shaped candy projectile held at the tip, toy-like and clearly directional upward. Leave empty transparent space around it for rotation. Do not include the base platform, text, letters, numbers, logo, watermark, background, or any copyrighted character. Use saturated pastel candy colors, thick dark outline, glossy highlights, clean silhouette, original dessert-war style.
```

### `tower_dart_barrel_frame_01.png` 到 `tower_dart_barrel_frame_03.png`

用途：飛鏢塔砲口攻擊動畫連續圖，攻擊前搖開始播放，impact 對齊最有力的發射 frame。

```text
Create a 3-frame animation sequence for tower_dart_barrel_frame_01.png, tower_dart_barrel_frame_02.png, and tower_dart_barrel_frame_03.png, transparent PNG with alpha, 256x256 each, same center point and same 3/4 top-down view. Draw only the rotating upper dart launcher, default facing upward. Frame 01: candy-cane dart launcher idle and slightly pulled back. Frame 02: launcher winds up with frosting glow and candy motion tension. Frame 03: launch moment with the dart just released, bright sugar sparkle and small motion streaks, no separate projectile far away. Keep the base platform absent. Dessert-war theme, thick dark outline, saturated pastel candy colors, no text, no numbers, no logo, no watermark, no background, original design.
```

## Tack Tower

### `tower_tack_base.png`

用途：鐵釘射手底座，不旋轉。

```text
Create an original cute dessert-war tower-defense combat sprite for tower_tack_base.png, transparent PNG with alpha, 256x256, centered composition, 3/4 top-down view. Draw only the non-rotating base of a tack shooter: a round tart shell platform with caramel crust, frosting trim, tiny candy rivets, biscuit feet, compact sturdy dessert machine base. It should support a radial spike top but not include the spike launcher itself. No text, no letters, no numbers, no logo, no watermark, no background, no photorealism, do not copy existing game assets. Use bright candy colors, thick dark outline, glossy dessert highlights, clean readable silhouette.
```

### `tower_tack_barrel_8.png`

用途：鐵釘射手上層針塔，8 根 radial 針孔，固定不跟目標旋轉，開火時做 `scale_pulse`。

```text
Create an original cute dessert-war tower-defense combat sprite for tower_tack_barrel_8.png, transparent PNG with alpha, 256x256, centered composition, 3/4 top-down view. Draw only the non-directional radial upper attack part of a tack shooter with exactly 8 evenly spaced sugar-spike barrels or peppermint needle holes around a circular cupcake turret top. The silhouette must be symmetrical and must look good without rotating toward a single target; avoid any long one-way barrel or arrow shape. No base platform, no text, no letters, no numbers, no logo, no watermark, no background, no copyrighted character. Use dessert-war theme, saturated pastel colors, thick outline, glossy frosting and candy highlights, original design.
```

### `tower_tack_barrel_8_frame_01.png` 到 `tower_tack_barrel_8_frame_03.png`

用途：8 針塔攻擊動畫連續圖，固定不朝單一目標。

```text
Create a 3-frame animation sequence for tower_tack_barrel_8_frame_01.png, tower_tack_barrel_8_frame_02.png, and tower_tack_barrel_8_frame_03.png, transparent PNG with alpha, 256x256 each, same center and same 3/4 top-down view. Draw only a radial tack shooter top with exactly 8 evenly spaced sugar-spike barrels or peppermint needle holes. Frame 01: calm idle radial cupcake turret. Frame 02: all 8 needle holes glow with frosting energy. Frame 03: simultaneous radial firing moment with tiny sugar sparkle bursts at all 8 holes, no long projectile trails. Keep it symmetrical and non-directional. No base platform, no text, no numbers, no logo, no watermark, no background, original dessert-war style.
```

### `tower_tack_barrel_12.png`

用途：鐵釘射手上層針塔，12 根 radial 針孔，升級後使用，固定不跟目標旋轉。

```text
Create an original cute dessert-war tower-defense combat sprite for tower_tack_barrel_12.png, transparent PNG with alpha, 256x256, centered composition, 3/4 top-down view. Draw only the non-directional radial upper attack part of an upgraded tack shooter with exactly 12 evenly spaced sugar-spike barrels or peppermint needle holes around a circular cupcake turret top. Make it visibly denser and stronger than the 8-barrel version while keeping the same dessert-war style and circular symmetry. It must not point toward one target. No base platform, no text, no letters, no numbers, no logo, no watermark, no background, no copyrighted character. Use saturated pastel candy colors, thick outline, glossy frosting highlights, clean readable silhouette, original mobile game sprite.
```

### `tower_tack_barrel_12_frame_01.png` 到 `tower_tack_barrel_12_frame_03.png`

用途：12 針塔攻擊動畫連續圖，升級後使用。

```text
Create a 3-frame animation sequence for tower_tack_barrel_12_frame_01.png, tower_tack_barrel_12_frame_02.png, and tower_tack_barrel_12_frame_03.png, transparent PNG with alpha, 256x256 each, same center and same 3/4 top-down view. Draw only an upgraded radial tack shooter top with exactly 12 evenly spaced sugar-spike barrels or peppermint needle holes. Frame 01: idle dense radial cupcake turret. Frame 02: all 12 holes charge with brighter candy glow. Frame 03: simultaneous radial firing moment with 12 small frosting spark bursts around the circle, no single forward direction. No base platform, no text, no numbers, no logo, no watermark, no background, original dessert-war style.
```

### `tower_tack_barrel_16.png`

用途：鐵釘射手上層針塔，16 根 radial 針孔，高階升級後使用，固定不跟目標旋轉。

```text
Create an original cute dessert-war tower-defense combat sprite for tower_tack_barrel_16.png, transparent PNG with alpha, 256x256, centered composition, 3/4 top-down view. Draw only the non-directional radial upper attack part of a high-upgrade tack shooter with exactly 16 evenly spaced sugar-spike barrels or peppermint needle holes around a circular cupcake turret top. Make it look like a powerful dense radial candy spike launcher, but still cute, readable, and symmetrical. It must not have a single forward-facing cannon direction. No base platform, no text, no letters, no numbers, no logo, no watermark, no background, no copyrighted character. Use dessert-war theme, saturated pastel colors, thick dark outline, glossy frosting and candy highlights, original design.
```

### `tower_tack_barrel_16_frame_01.png` 到 `tower_tack_barrel_16_frame_03.png`

用途：16 針塔攻擊動畫連續圖，高階升級後使用。

```text
Create a 3-frame animation sequence for tower_tack_barrel_16_frame_01.png, tower_tack_barrel_16_frame_02.png, and tower_tack_barrel_16_frame_03.png, transparent PNG with alpha, 256x256 each, same center and same 3/4 top-down view. Draw only a high-upgrade radial tack shooter top with exactly 16 evenly spaced sugar-spike barrels or peppermint needle holes. Frame 01: idle powerful dense radial candy turret. Frame 02: 16 holes glow with intense frosting energy. Frame 03: simultaneous radial firing moment with 16 tiny sugar sparkle bursts, still readable and symmetrical, no single forward cannon direction. No base platform, no text, no numbers, no logo, no watermark, no background, original dessert-war style.
```

## Bomb Tower

### `tower_bomb_base.png`

用途：炸彈塔底座，不旋轉。

```text
Create an original cute dessert-war tower-defense combat sprite for tower_bomb_base.png, transparent PNG with alpha, 256x256, centered composition, 3/4 top-down view. Draw only the non-rotating base of a bomb cannon tower: a sturdy chocolate cookie carriage, biscuit wheels, caramel brackets, frosting bolts, cupcake-metal toy support frame, heavy but cute dessert battlefield machinery. Do not include the cannon barrel or bomb muzzle. No text, letters, numbers, logo, watermark, background, photorealism, or copied game asset. Use thick dark outline, saturated chocolate/caramel/pastel colors, glossy candy highlights, clean readable silhouette.
```

### `tower_bomb_barrel.png`

用途：炸彈塔砲管，可旋轉，預設朝上。

```text
Create an original cute dessert-war tower-defense combat sprite for tower_bomb_barrel.png, transparent PNG with alpha, 256x256, centered composition, 3/4 top-down view. Draw only the rotating upper cannon barrel of a dessert bomb tower, default facing upward: a chunky chocolate cannon tube with frosting bands, caramel metal rim, a round truffle bomb visible near the muzzle, toy-like but powerful, clearly directional upward. Do not include the base carriage. No text, letters, numbers, logo, watermark, background, photorealism, or copied game asset. Use thick dark outline, glossy candy highlights, clean mobile game sprite silhouette, original dessert-war design.
```

### `tower_bomb_barrel_frame_01.png` 到 `tower_bomb_barrel_frame_03.png`

用途：炸彈塔砲管攻擊動畫連續圖。

```text
Create a 3-frame animation sequence for tower_bomb_barrel_frame_01.png, tower_bomb_barrel_frame_02.png, and tower_bomb_barrel_frame_03.png, transparent PNG with alpha, 256x256 each, same center point and same 3/4 top-down view. Draw only the rotating chocolate cannon barrel, default facing upward. Frame 01: cannon idle with truffle bomb loaded. Frame 02: cannon compresses and glows with caramel pressure during windup. Frame 03: firing moment with frosting flash at muzzle and the truffle bomb just leaving the barrel, no far projectile. No base carriage, no text, no numbers, no logo, no watermark, no background, original dessert-war style.
```

## Ice Tower

### `tower_ice_base.png`

用途：冰塔底座，不旋轉。

```text
Create an original cute dessert-war tower-defense combat sprite for tower_ice_base.png, transparent PNG with alpha, 256x256, centered composition, 3/4 top-down view. Draw only the non-rotating base of an ice tower: a frosted cookie platform with blue sugar-glass rim, shaved-ice crystals, waffle cone supports, powdered sugar snow accents, stable dessert magic base. Do not include the ice cannon, wand, nozzle, or beam emitter. No text, letters, numbers, logo, watermark, background, photorealism, or copied game asset. Use cool candy colors, thick dark outline, glossy ice cream and sugar crystal highlights, clean readable silhouette.
```

### `tower_ice_barrel.png`

用途：冰塔砲口/冰霜發射器，可旋轉，預設朝上。

```text
Create an original cute dessert-war tower-defense combat sprite for tower_ice_barrel.png, transparent PNG with alpha, 256x256, centered composition, 3/4 top-down view. Draw only the rotating upper attack part of an ice tower, default facing upward: a popsicle-shaped frost cannon or ice cream wand emitter, blue rock-candy nozzle, snow sugar crystals, frosty cream swirl, clearly directional upward but cute and toy-like. Do not include the base platform. No text, letters, numbers, logo, watermark, background, photorealism, or copied game asset. Use thick dark outline, cool pastel blue and white candy colors, glossy frozen dessert highlights, clean silhouette.
```

### `tower_ice_barrel_frame_01.png` 到 `tower_ice_barrel_frame_03.png`

用途：冰塔砲口攻擊動畫連續圖。

```text
Create a 3-frame animation sequence for tower_ice_barrel_frame_01.png, tower_ice_barrel_frame_02.png, and tower_ice_barrel_frame_03.png, transparent PNG with alpha, 256x256 each, same center point and same 3/4 top-down view. Draw only the rotating ice tower emitter, default facing upward. Frame 01: frosty popsicle cannon idle. Frame 02: blue sugar crystals charge and glow during windup. Frame 03: icy burst at the nozzle with snow sugar sparkle and a short frost wave beginning, no far projectile. No base platform, no text, no numbers, no logo, no watermark, no background, original dessert-war style.
```

## Cake Splash Tower

### `tower_cake_splash_frame_01.png` 到 `tower_cake_splash_frame_06.png`

用途：沒有砲管的範圍傷害塔，本體動畫連續圖。攻擊前搖開始播放，impact frame 表現奶油/果醬範圍爆發。

```text
Create a 6-frame animation sequence for tower_cake_splash_frame_01.png through tower_cake_splash_frame_06.png, transparent PNG with alpha, 256x256 each, same center point and same 3/4 top-down tower-defense view. Draw a no-barrel animated area damage tower in an original cute dessert-war theme: a layered cupcake or cake fountain turret that releases frosting and jam splash around itself. Frame 01: calm cake tower idle, no barrel, no direction. Frame 02: frosting starts to swell upward during windup. Frame 03: cream and jelly energy gathers in a circular ring. Frame 04: impact moment, big circular frosting splash burst expanding evenly in all directions. Frame 05: splash droplets and sprinkles settle. Frame 06: tower returns toward idle. Keep the tower centered, symmetrical, no single cannon or barrel, no text, no numbers, no logo, no watermark, no background, no copyrighted character, thick dark outline, saturated pastel candy colors, clean mobile game sprite style.
```

## Fallback Tower

### `tower_fallback_base.png`

用途：未知塔種 fallback 底座。

```text
Create an original cute dessert-war tower-defense combat sprite for tower_fallback_base.png, transparent PNG with alpha, 256x256, centered composition, 3/4 top-down view. Draw only a generic non-rotating fallback base: a simple round cookie pedestal with frosting edge, candy rivets, neutral cream and caramel colors, readable placeholder dessert platform that can support any tower top. No question mark, no text, no letters, no numbers, no logo, no watermark, no background, no photorealism, no copied game asset. Use thick dark outline, glossy highlights, clean mobile game sprite style.
```

### `tower_fallback_barrel.png`

用途：未知塔種 fallback 砲口。

```text
Create an original cute dessert-war tower-defense combat sprite for tower_fallback_barrel.png, transparent PNG with alpha, 256x256, centered composition, 3/4 top-down view. Draw only a generic rotating fallback attack top, default facing upward: a small frosting swivel with a neutral candy nozzle, sugar-glass cap, simple toy-like shape that reads as a generic tower barrel but has no text or symbol. Do not include the base platform. No text, letters, numbers, logo, watermark, background, photorealism, or copied game asset. Use thick dark outline, pastel dessert colors, glossy candy highlights, clean silhouette.
```

## 產圖後檢查清單

| 檢查 | 通過條件 |
|---|---|
| 檔名 | 必須完全符合本文件的檔名 |
| 格式 | PNG with alpha，透明背景 |
| 尺寸 | 建議 256x256；若不同尺寸，base/barrel 同塔最好一致 |
| Base/Barrel 分離 | base 不含可旋轉砲口；barrel 不含底座平台 |
| Target-facing barrel | `tower_dart_barrel.png`、`tower_bomb_barrel.png`、`tower_ice_barrel.png` 預設朝上 |
| Tack barrel | `tower_tack_barrel_8.png`、`tower_tack_barrel_12.png`、`tower_tack_barrel_16.png` 接近放射對稱，且能辨識 8/12/16 根針孔或砲管，不暗示單一目標方向 |
| Animation frames | 同一序列每張圖尺寸、中心點、視角一致；frame 順序能從前搖、攻擊瞬間到後搖/回復 |
| No-barrel tower | `tower_cake_splash_frame_*.png` 不得出現砲管，必須用本體或範圍效果動畫表現攻擊 |
| 文字 | 圖內不可有文字、數字、商標或浮水印 |
| 主題 | 必須符合甜點戰爭，不要變成一般軍事、寫實槍械、科幻砲台或既有遊戲素材 |
| 可替換性 | 替換同名 PNG 後不需要改程式碼或 Lua metadata path |
