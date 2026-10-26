# H03 阿松：驗收

status: implemented

kit 狀態：`supported`。四招都有 handler 與測試，以完整 kit 建置（不需 `--slice`）。

| 技能 | 實作 |
|---|---|
| `a34_frost_bolt` | G-1 追蹤彈，`Spec.buildings`：可鎖定敵方建築 |
| `a34_iceberg_strike` | 主目標＋335 冰爆、移速 −30%／攻速 −20% |
| `a34_frost_breath` | G-1 直線穿透，`Spec.dot` 命中後 DoT |
| `a34_rapid_cooling` | on_learn 最大法力；主動 22 秒 cooldown_percentage；被動減冷卻由 content `cast_cooldown_refund` 交給 host |

實機（quick 地圖、10 分鐘內）要看：
1. Q 對敵方英雄與敵方塔各施放一次：彈道可見，命中時才扣血並暈眩。
2. W 對兵群施放：主目標與 335 內其他敵人扣血，被擊中者 5 秒內明顯變慢。
3. E 朝兵線施放：沿途每個敵人只受一次命中，之後 5 秒持續扣血。
4. 6 級學 R：最大法力 +300。施放 Q 後，W／E 的冷卻數字立刻少 2 秒。主動 R 期間，基礎技能冷卻明顯變短。
