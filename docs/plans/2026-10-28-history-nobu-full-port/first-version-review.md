# 第一版演示影片總表

影片：`target/spotlight/<hero>/<hero>_spotlight.mp4`（不進 git）；取樣：`peak/all.jpg`、`peak/attack.jpg`。

- **第一版**：看得到效果，普攻命中那幀有特效，死亡看得到倒地，沒有跨段殘留。
- **獨特**：由 `fx-identity-table.md` 算出，該段每個 cue 的造型都只用在這一招才算「是」。
- **顏色、角度**：還沒逐招檢查的標「未查」。普攻命中一律水平大彎月，三欄都不合格。
- 下一版的驗收改看獨特、顏色、角度三欄，所以這裡大多不合格是預期內的。

| 英雄 | 判定 | 段 | 第一版 | 獨特 | 顏色 | 角度 | 已知待優化 |
|---|---|---|---|---|---|---|---|
| sasaki_kojiro | main 判定通過 | Q | 通過 | 是 | 未查 | 未查 |  |
| sasaki_kojiro | main 判定通過 | W | 通過 | 否（5 招共用） | 未查 | 未查 |  |
| sasaki_kojiro | main 判定通過 | E | 通過 | 無 cue | 未查 | 未查 | 被動，段內只有普攻和命中 |
| sasaki_kojiro | main 判定通過 | R | 通過 | 否（5 招共用） | 未查 | 未查 |  |
| sasaki_kojiro | main 判定通過 | X | 通過 | 否（12 招共用） | 未查 | 未查 | 金色弧光和爆點清楚 |
| sasaki_kojiro | main 判定通過 | 普攻 | 通過 | 否（近戰共用） | 否 | 否（固定水平） | 改依武器和揮擊方向 |
| sasaki_kojiro | main 判定通過 | 死亡 | 通過 | 不適用 | 不適用 | 不適用 |  |
| kojima_sadaoki | main 判定通過 | Q | 通過 | 否（12 招共用） | 未查 | 未查 |  |
| kojima_sadaoki | main 判定通過 | W | 通過 | 否（5 招共用） | 未查 | 未查 |  |
| kojima_sadaoki | main 判定通過 | E | 通過 | 否（12 招共用） | 未查 | 未查 |  |
| kojima_sadaoki | main 判定通過 | R | 通過 | 否（5 招共用） | 未查 | 未查 | 第一下強，第 3 幀起主體偏小 |
| kojima_sadaoki | main 判定通過 | 普攻 | 通過 | 否（近戰共用） | 否 | 否（固定水平） | 改依武器和揮擊方向 |
| kojima_sadaoki | main 判定通過 | 死亡 | 通過 | 不適用 | 不適用 | 不適用 |  |
| maeda_keiji | main 判定通過 | Q | 通過 | 否（12 招共用） | 未查 | 未查 |  |
| maeda_keiji | main 判定通過 | W | 通過 | 否（5 招共用） | 未查 | 未查 | 後退已關（NO_VIEW），main 看到的後退待確認來源 |
| maeda_keiji | main 判定通過 | E | 通過 | 否（3 招共用） | 未查 | 未查 |  |
| maeda_keiji | main 判定通過 | R | 通過 | 否（5 招共用） | 未查 | 未查 | 法陣模糊（貼圖 512） |
| maeda_keiji | main 判定通過 | 普攻 | 通過 | 否（近戰共用） | 否 | 否（固定水平） | 改依武器和揮擊方向 |
| maeda_keiji | main 判定通過 | 死亡 | 通過 | 不適用 | 不適用 | 不適用 |  |
| otani_yoshitsugu | main 判定通過 | Q | 通過 | 否（12 招共用） | 未查 | 未查 |  |
| otani_yoshitsugu | main 判定通過 | W | 通過 | 否（2 招共用） | 未查 | 未查 |  |
| otani_yoshitsugu | main 判定通過 | E | 通過 | 否（4 招共用） | 未查 | 未查 |  |
| otani_yoshitsugu | main 判定通過 | R | 通過 | 否（4 招共用） | 未查 | 未查 |  |
| otani_yoshitsugu | main 判定通過 | X | 通過 | 否（4 招共用） | 未查 | 未查 | 被動，只有星光和漩渦 |
| otani_yoshitsugu | main 判定通過 | C | 通過 | 否（12 招共用） | 未查 | 未查 | 被動，只有星光和漩渦 |
| otani_yoshitsugu | main 判定通過 | 普攻 | 通過 | 否（近戰共用） | 否 | 否（固定水平） | 改依武器和揮擊方向 |
| otani_yoshitsugu | main 判定通過 | 死亡 | 通過 | 不適用 | 不適用 | 不適用 |  |
| nohime | main 判定通過 | Q | 通過 | 否（5 招共用） | 未查 | 未查 | 第一下後變很小 |
| nohime | main 判定通過 | W | 通過 | 否（5 招共用） | 未查 | 未查 |  |
| nohime | main 判定通過 | E | 通過 | 是 | 未查 | 未查 |  |
| nohime | main 判定通過 | R | 通過 | 否（12 招共用） | 未查 | 未查 |  |
| nohime | main 判定通過 | 普攻 | 通過 | 否（近戰共用） | 否 | 否（固定水平） | 改依武器和揮擊方向 |
| nohime | main 判定通過 | 死亡 | 通過 | 不適用 | 不適用 | 不適用 |  |
| shimazu_yoshihiro | main 判定通過 | Q | 通過 | 否（12 招共用） | 未查 | 未查 |  |
| shimazu_yoshihiro | main 判定通過 | W | 通過 | 否（5 招共用） | 未查 | 未查 |  |
| shimazu_yoshihiro | main 判定通過 | E | 通過 | 否（4 招共用） | 未查 | 未查 |  |
| shimazu_yoshihiro | main 判定通過 | R | 通過 | 否（5 招共用） | 未查 | 未查 | 火球落點偏離目標右側 |
| shimazu_yoshihiro | main 判定通過 | 普攻 | 通過 | 否（近戰共用） | 否 | 否（固定水平） | 改依武器和揮擊方向 |
| shimazu_yoshihiro | main 判定通過 | 死亡 | 通過 | 不適用 | 不適用 | 不適用 | 武器豎立在屍體上方 |
| tokugawa_ieyasu | C 判定通過，待 main 確認 | Q | 通過 | 否（12 招共用） | 未查 | 未查 |  |
| tokugawa_ieyasu | C 判定通過，待 main 確認 | W | 通過 | 是 | 未查 | 未查 |  |
| tokugawa_ieyasu | C 判定通過，待 main 確認 | E | 通過 | 否（5 招共用） | 未查 | 未查 |  |
| tokugawa_ieyasu | C 判定通過，待 main 確認 | R | 通過 | 否（3 招共用） | 未查 | 未查 | 法陣模糊（貼圖 512） |
| tokugawa_ieyasu | C 判定通過，待 main 確認 | X | 通過 | 否（3 招共用） | 未查 | 未查 | 受傷才觸發的被動，除第一下外與普攻相同 |
| tokugawa_ieyasu | C 判定通過，待 main 確認 | 普攻 | 通過 | 否（近戰共用） | 否 | 否（固定水平） | 改依武器和揮擊方向 |
| tokugawa_ieyasu | C 判定通過，待 main 確認 | 死亡 | 通過 | 不適用 | 不適用 | 不適用 |  |
| yamagata_masakage | C 判定通過，待 main 確認 | Q | 通過 | 否（5 招共用） | 未查 | 未查 |  |
| yamagata_masakage | C 判定通過，待 main 確認 | W | 通過 | 否（5 招共用） | 未查 | 未查 |  |
| yamagata_masakage | C 判定通過，待 main 確認 | E | 通過 | 否（3 招共用） | 未查 | 未查 | 被動，段內只有普攻和命中 |
| yamagata_masakage | C 判定通過，待 main 確認 | R | 通過 | 否（3 招共用） | 未查 | 未查 | 普攻段不再混入脈衝 |
| yamagata_masakage | C 判定通過，待 main 確認 | 普攻 | 通過 | 否（近戰共用） | 否 | 否（固定水平） | 改依武器和揮擊方向 |
| yamagata_masakage | C 判定通過，待 main 確認 | 死亡 | 通過 | 不適用 | 不適用 | 不適用 |  |
| okuma_tomohide | C 判定通過，待 main 確認 | Q | 通過 | 否（5 招共用） | 未查 | 未查 | 蓄力中段火焰偏小；硬邊已消失 |
| okuma_tomohide | C 判定通過，待 main 確認 | W | 通過 | 否（12 招共用） | 未查 | 未查 |  |
| okuma_tomohide | C 判定通過，待 main 確認 | E | 通過 | 否（3 招共用） | 未查 | 未查 |  |
| okuma_tomohide | C 判定通過，待 main 確認 | R | 通過 | 否（3 招共用） | 未查 | 未查 | 火翼只在施放瞬間 |
| okuma_tomohide | C 判定通過，待 main 確認 | 普攻 | 通過 | 否（近戰共用） | 否 | 否（固定水平） | 改依武器和揮擊方向 |
| okuma_tomohide | C 判定通過，待 main 確認 | 死亡 | 通過 | 不適用 | 不適用 | 不適用 | 武器豎立在屍體上方 |
| maeda_toshiie | C 判定通過，待 main 確認 | Q | 通過 | 否（2 招共用） | 未查 | 未查 | 目標擊退到畫面邊緣，命中仍看得到 |
| maeda_toshiie | C 判定通過，待 main 確認 | W | 通過 | 是 | 未查 | 未查 | 瞬移後主角在畫面內，目標被甩出畫面，特效偏弱 |
| maeda_toshiie | C 判定通過，待 main 確認 | E | 通過 | 否（12 招共用） | 未查 | 未查 | 只有普攻和命中 |
| maeda_toshiie | C 判定通過，待 main 確認 | R | 通過 | 否（3 招共用） | 未查 | 未查 |  |
| maeda_toshiie | C 判定通過，待 main 確認 | 普攻 | 通過 | 否（近戰共用） | 否 | 否（固定水平） | 改依武器和揮擊方向 |
| maeda_toshiie | C 判定通過，待 main 確認 | 死亡 | 通過 | 不適用 | 不適用 | 不適用 | 武器豎立在屍體上方 |

## 共通待優化

- 大面積地面圖放大後模糊：來源圖都是 512（`generated_art.lua` 的 `CUE_EDGE`）。
- 被打者腳下的淡環不是命中特效，來源還沒查清。
- 幀的受擊 cue（`OnDamageCue`）在這些英雄的實機局沒有觸發，命中改跟血量下降；原因還沒查。
- 效能量測（run_10000、1+9）和覆蓋統計還沒做。
