# H04 阿市：假設與阻塞

來源：`damody/nobugg_official@ca6ee86`，C17 current 版。結構化版本在 `source.lua`。

## 阻塞

無（2026-10-25 解除）：

- 光鍊：host tether（`runtime/native/tether.rs`）。
- 極光守護、自我奉獻的範圍友軍：host `ally_area_restore` 封包。
- 自我奉獻：第 5 欄（ward-canonical）＋`auto_learn`，施法掛點用英雄 UnitScript。

## 假設

- 基礎數值：本地沒有阿市。生命、法力、攻擊、轉身沿用遠程施法者 `training_luminary`；護甲、射程、移速、普攻彈速與攻擊前搖取來源。
- 只使用 current 版，不混入 `_old` 版本。
- `C17.lua` 不在來源 repo，效果依 KV 與描述推定。
- 祈勝光輝：
  - 「全區域」以施法距離 10000 表達。
  - 總攻擊 +7.5/15/22.5/30% 以 `base_damage_outgoing_pct` 表達。Fixed64 精度 1/1024，7.5% 存成 77/1024。
- 極光守護：KV 有兩個 `"05"`，依描述解讀：`regen` 是友軍回復最大生命 %，`hp` 是敵人最大生命 % 傷害。6/12/18 級解鎖。
- 自我奉獻：描述範圍 750、KV 800，依 D4 政策採 KV 800。
