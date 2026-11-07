# H05 稻姬：驗收

status: implemented

kit 狀態：`supported`。四招都有 handler 與測試，以完整 kit 建置。

| 技能 | 實作 |
|---|---|
| `a02_venom_arrow` | G-2 attack orb（content `attack_orb`，handler 只切換標記） |
| `a02_wind_prayer` | 立即回血＋10 秒攻速 |
| `a02_wind_drift` | 被動魔抗／總攻擊% |
| `a02_shadow_meld` | `invisibility_level` buff 隱形（攻擊不破隱）＋`no_unit_collision`＋攻速 ×2＋移速 +60 |

實機（quick 地圖、10 分鐘內）要看：
1. Q 開啟後打小兵：每擊扣 10 法力，命中時額外扣血並減速；法力見底時自動關閉。
2. W：立即回血，10 秒內攻擊明顯變快。
3. E：學會後 HUD 的魔抗與攻擊數值上升。
4. 6 級 R：敵方畫面看不到稻姬，她攻擊時也不會現形，可以穿過小兵。敵方偵測（有偵測的塔或守衛）仍然看得到。
