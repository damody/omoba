# H01～H06 實機逐招驗收表

跑完一局 quick map split（每局 ≤ 10 分鐘，autopilot 操作一席）後，依下表逐招打勾。判定已自動化：

```bat
tools\lua\lua.exe scripts\check_hero_live_acceptance.lua --run target\mvp-a3\<split 目錄> [--reachable-level 12]
```

工具從 `server/match-plan.json` 找出非 bot 的英雄，從該英雄的 package 推導每招的期望，不維護手寫表。接著用 server log 與 `p<N>/autopilot-p<N>.coverage.json`（沒有時改讀 `.jsonl` 的 accepted ack）判定，每招輸出一行：

- `PASS`／`FAIL`：一般判定。
- `NOT_REACHABLE_IN_10MIN`：auto_learn 門檻高於 `--reachable-level`。
- `SKIP`：英雄沒有 package，例如 training 英雄。

最後兩項全局 gate 是無 observer mismatch、無 termination。全部通過時 exit 0。

## 證據來源

| 來源 | 內容 |
|---|---|
| **server log**（omb） | `[scripting] cast ok ability='<id>' caster=<entity> rank=<n>`：handler 接受施放，每次一行（2026-10-26 新增）。`[scripting] learn ability='<id>' caster=<entity> rank=<n>`：學會或升級，含 auto_learn。失敗時會出現 `[scripting] ability '<id>' execute returned error: <原因>` 或 `blocked — ...`。 |
| **autopilot `coverage.json`** | 以 intent 為 key：`cast_<slot>_target`、`cast_<slot>_instant`、`cast_<slot>_self`、`cast_<slot>_toggle`、`upgrade_<slot>`，slot 為 `q`／`w`／`e`／`r`／`extra`。`EffectApplied > 0` 表示權威端已開始冷卻；toggle 沒有冷卻，只看 `Submitted`／`AckAccepted`。 |
| **autopilot trace** | `no_effect` 事件：卡片顯示可施放，權威端卻沒有效果，該欄退避 15 秒。出現時要查 server log 的拒絕原因。 |
| **UE 畫面／錄影** | 投射物、範圍、buff 圖示、隱形、位移。 |

server log 的 `cast ok` 只代表 handler 已接受。serial settlement 仍可能因資源不足拒絕，這時 `EffectApplied` 不會增加。兩邊都有才算過。

## autopilot 會怎麼用這些技能

規則只看技能卡片（形狀、敵我、被動、toggle），沒有英雄專屬分支（OmAutoPilot `ChooseCast`）：

- 敵方／未指定的單位或點技能：對射程內最近的敵方英雄施放。法力 ≥ 70% 時也會打小兵。
- 無目標技能（自身 buff、toggle、自身範圍）：敵方英雄在 600 內時施放。
- 友方單位技能：有友方英雄（含自己）血量低於 60% 時，對其中最傷的施放；或敵方英雄在 900 內時對自己施放（戰前增益）。
- toggle：只在要打開時按；死亡或法力低於 15% 後才會再按。
- 被動：只升級，不施放。D 欄的 auto_learn 由權威端自動學會。

## 逐招

| 英雄 | 欄 | 技能 | autopilot intent | server log | 實機要看到 | 可在 10 分鐘內驗到 |
|---|---|---|---|---|---|---|
| H01 雜賀孫市 | Q | `b01_sniper_mode` | `cast_q_instant` | `cast ok ability='b01_sniper_mode'` | 射程變長、攻速變慢的狀態，限時後解除 | 是 |
| | W | `b01_saika_corps` | `cast_w_target` | `cast ok ability='b01_saika_corps'` | 在點上出現固定鐵砲兵並開火；充能數減少 | 是 |
| | E | `b01_rain_iron_cannon`（被動） | `upgrade_e` | `learn ability='b01_rain_iron_cannon'` | 普攻對不同類別目標的加傷 | 是（只看升級） |
| | R | `b01_three_stage` | `cast_r_instant` | `cast ok ability='b01_three_stage'` | 三段射擊狀態 | 6 級後 |
| H02 伊達政宗 | Q | `b04_flame_trail_dash` | `cast_q_target` | `cast ok ability='b04_flame_trail_dash'` | 衝刺；路徑旁敵人被追蹤火焰命中 | 是 |
| | W | `b04_fire_blast` | `cast_w_target` | `cast ok ability='b04_fire_blast'` | 點上的火焰爆炸 | 是 |
| | E | `b04_flame_swiftness`（被動） | `upgrade_e` | `learn ability='b04_flame_swiftness'` | 攻速與護甲提升 | 是（只看升級） |
| | R | `b04_tanegashima` | `cast_r_instant` | `cast ok ability='b04_tanegashima'` | 種子島狀態（普攻強化） | 6 級後 |
| H03 阿松 | Q | `a34_frost_bolt` | `cast_q_target` | `cast ok ability='a34_frost_bolt'` | 追蹤寒霜彈飛行，命中後暈眩 | 是 |
| | W | `a34_iceberg_strike` | `cast_w_target` | `cast ok ability='a34_iceberg_strike'` | 主目標與周圍敵人扣血、變慢 | 是 |
| | E | `a34_frost_breath` | `cast_e_target` | `cast ok ability='a34_frost_breath'` | 直線冰霜穿過兵線，命中後持續扣血 | 是 |
| | R | `a34_rapid_cooling` | `cast_r_instant` | `cast ok ability='a34_rapid_cooling'`；`learn ability='a34_rapid_cooling'` | 學會後法力上限上升；施放 Q 後 W／E 冷卻少 2 秒；主動期間冷卻縮短 | 6 級後 |
| H04 阿市 | Q | `c17_light_chain` | `cast_q_target` | `cast ok ability='c17_light_chain'` | 對敵 3 秒後斷開：遠距傷害高、近距定身久。autopilot 只會對敵使用 | 是（只驗對敵） |
| | W | `c17_victory_blessing` | `cast_w_target`（受傷的友方英雄）或 `cast_w_self` | `cast ok ability='c17_victory_blessing'` | 友軍 buff 圖示，持續 16 秒 | 是。需要內容重建：見下方「前提」 |
| | E | `c17_divine_strike` | `cast_e_target` | `cast ok ability='c17_divine_strike'` | 點上 200 範圍敵人扣血 | 是 |
| | R | `c17_aurora_guard` | `cast_r_target` | `cast ok ability='c17_aurora_guard'` | 1 秒後敵人大量扣血，範圍內友軍回血 | 6 級後 |
| | D | `c17_devotion`（被動、auto_learn） | 無 | `learn ability='c17_devotion' rank=1`（出生時） | 每次施法後自己與附近友軍回一點血與魔 | 是 |
| H05 稻姬 | Q | `a02_venom_arrow`（toggle） | `cast_q_toggle` | `cast ok ability='a02_venom_arrow'` | 開啟後普攻扣魔、命中減速；法力見底時自動關閉 | 是。需要 UE 重建：見下方「前提」 |
| | W | `a02_wind_prayer` | `cast_w_instant` | `cast ok ability='a02_wind_prayer'` | 立即回血，攻速 buff 10 秒 | 是 |
| | E | `a02_wind_drift`（被動） | `upgrade_e` | `learn ability='a02_wind_drift'` | 魔抗與攻擊數值提升 | 是（只看升級） |
| | R | `a02_shadow_meld` | `cast_r_instant` | `cast ok ability='a02_shadow_meld'` | 敵方畫面看不到稻姬，攻擊時也不現形；可穿過小兵 | 6 級後；要從敵隊 client 看 |
| H06 淺井長政 | Q | `b08_fang_slash` | `cast_q_target` | `cast ok ability='b08_fang_slash'` | 直線刀牙穿過兵線 | 是 |
| | W | `b08_flash_cut` | `cast_w_target` | `cast ok ability='b08_flash_cut'` | 瞬移到敵人身旁，暈眩並燒魔 | 是 |
| | E | `b08_living_soul`（被動） | `upgrade_e` | `learn ability='b08_living_soul'` | 連續施法時普攻傷害逐層變高 | 是 |
| | R | `b08_sky_fang` | `cast_r_target` | `cast ok ability='b08_sky_fang'` | 寬刀牙飛 3000 | 6 級後 |
| | D | `b08_endless_fang`（auto_learn 16 級） | `cast_extra_target` | `learn ability='b08_endless_fang'`；`cast ok ability='b08_endless_fang'` | 1800 傷害並暈眩 2.2 秒 | **否**：10 分鐘內通常到不了 16 級。由 ward 的 `authored_auto_learn` 測試證明 |

## 前提與已知限制

- **內容重建**：`c17_victory_blessing` 這類以 Rust 實作的友方單位技能，content 要帶 `unit_target`（hero_author 2026-10-26 起會輸出）。UE codegen 也要用 `ability_unit_allegiance` 產生 `UnitAllegiance = "ally"`。重建前，卡片顯示 `unspecified`，autopilot 會把它當成攻擊技能對敵施放，server 會拒絕（log 出現 `execute returned error: blessing target is not a live ally`）。
- **UE 重建**：toggle 識別需要新的 `FOmAbilityHudStatePayload::bPublishedToggle`，由 OmWorldBridgeActor 依 metadata 的 AbilityType 填入。沒有重建時，autopilot 會反覆按法球，在開與關之間切換。
- **光鍊對友、極光守護的治療**：autopilot 不會特意對友軍使用光鍊（卡片敵我為 `unspecified`，因為技能對敵我都可施放）。極光守護是點技能，會在敵方英雄位置施放，治療只有在友軍剛好在範圍內時才看得到。
- **隱形**：要從敵隊 client 的畫面或 trace 確認看不到。己方畫面一定看得到自己。
- **16 級 D 欄**：10 分鐘的局到不了 16 級，所以不在實機必驗範圍。
