# M6 仍缺的美術（第 1～9 批）

由 `tools\lua\lua.exe scripts\m6_presentation_coverage.lua --missing-art` 產生，不要手改。逐批細節在 `m6-batch<N>-presentation.md`。

> **2026-10-29 使用者決定：缺的美術用 Grok 生成**（README D17）。沿用 Grok Build CLI 的 `image_gen`，逐張看過後經 `scripts/generated_art.lua --import` 匯入（`generated-art-v1.json`，HUD sync 在沒有原作圖時使用），每次生成都記到 `docs/plans/2026-10-24-omfue-month-mvp/asset-ledger.md`。已生成的不再列在下面。

還沒生成的替代方案都已經在 UE 生效：

- **頭像**：共用佔位圖 `hero-placeholder-v1.png`，由 HUD sync 在英雄上架時自動補上。
- **技能圖示**：共用佔位圖 `ability-placeholder-v1.png`，由 HUD sync 補上，建置時會印出警告。
- **cue**：依 cue 目錄的 kind 播通用佔位特效（環、圓盤、連線；sound 不畫）。之後可以在 Editor 的 Om Presentation Cues 指定美術。

## 頭像（替代：共用佔位圖）（0）

沒有。

## 技能圖示（替代：共用佔位圖）（0）

沒有。

## 沒有原作資產的特效 cue（替代：依 kind 的通用佔位特效）（0）

沒有。

## 沒有原作資產的聲音與動作 cue（不需要參考圖；sound 不畫，gesture 用預設動作）（56）

| 批 | 英雄 | cue | kind |
|---|---|---|---|
| 1 | C22 佐佐木小次郎 | `c22d_cast_sound` | sound |
| 1 | C22 佐佐木小次郎 | `c22r_crit_sound` | sound |
| 1 | A26 濃姬 | `a26r_crit_gesture` | gesture |
| 1 | A26 濃姬 | `a26w_flak_sound` | sound |
| 2 | B30 柿崎景家 | `b30r_windwalk_sound` | sound |
| 2 | C01 明智光秀 | `c01r_crit_sound` | sound |
| 2 | C01 明智光秀 | `c01r_slam_gesture` | gesture |
| 2 | C01 明智光秀 | `c01w_slam_gesture` | gesture |
| 2 | A35 島左近 | `a35t_cast_sound` | sound |
| 3 | C14 齋藤義龍 | `c14t_cast_sound` | sound |
| 3 | C24 柳生宗嚴 | `c24w_cast_sound` | sound |
| 3 | A22 仙石秀久 | `a22w_spin_gesture` | gesture |
| 3 | A36 前野長康 | `a36r_vo` | sound |
| 4 | A33 本多正信 | `a33e_ghost_walk_sound` | sound |
| 4 | B11 菊姬 | `b11t_echo_slam_gesture` | gesture |
| 4 | B09 真田昌幸 | `b09r_toggle_sound` | sound |
| 4 | B09 真田昌幸 | `b09t_cast_sound` | sound |
| 4 | B09 真田昌幸 | `b09w_cast_sound` | sound |
| 4 | B18 湖衣姬 | `b18t_sound` | sound |
| 4 | B18 湖衣姬 | `b18t_title` | sound |
| 4 | B18 湖衣姬 | `b18t_vo` | sound |
| 5 | A03 石川數正 | `a03w_tick_sound` | sound |
| 5 | B35 上泉信綱 | `b35r_counter_sound` | sound |
| 5 | C20 長宗我部元親 | `c20t_title` | sound |
| 5 | A17 佐佐成政 | `a17e_cast_sound` | sound |
| 5 | B10 三好長慶 | `b10t_ghost_ship_sound` | sound |
| 5 | C11 立花宗茂 | `c11t_title` | sound |
| 5 | C16 果心居士 | `c16r_heal_sound` | sound |
| 6 | A27 柴田勝家 | `a27r_crit_sound` | sound |
| 6 | A27 柴田勝家 | `a27r_echo_slam_gesture` | gesture |
| 6 | A27 柴田勝家 | `relocate_sound` | sound |
| 6 | A25 織田信長 | `a25r_crit_sound` | sound |
| 6 | A25 織田信長 | `a25r_slam_gesture` | gesture |
| 6 | A31 森蘭丸 | `a31w_impact_sound` | sound |
| 6 | C04 阿國 | `c04r_sound` | sound |
| 6 | C04 阿國 | `c04w_sound` | sound |
| 6 | A32 瀧川一益 | `a32e_time_lapse_sound` | sound |
| 7 | C13 龍造寺隆信 | `c13t_title` | sound |
| 7 | C21 宮本武藏 | `c21r_crit_sound` | sound |
| 7 | C21 宮本武藏 | `c21r_slam_gesture` | gesture |
| 7 | A15 寧寧 | `a15w_cast_sound` | sound |
| 7 | B12 山本勘助 | `b12e_heal_sound` | sound |
| 8 | B14 片倉景綱 | `b14d_night_sound` | sound |
| 8 | B14 片倉景綱 | `b14r_slam_gesture` | gesture |
| 8 | B23 伊勢姬 | `b23r_heal_sound` | sound |
| 8 | B23 伊勢姬 | `b23t_title` | sound |
| 8 | A21 瑞龍院日秀 | `a21d_mana_break_sound` | sound |
| 8 | B13 百地三太夫 | `b13t_tornado_sound` | sound |
| 9 | A13 服部半藏 | `a13e_hook_throw_sound` | sound |
| 9 | A13 服部半藏 | `a13t_mirror_image_sound` | sound |
| 9 | A13 服部半藏 | `a13t_title` | sound |
| 9 | B07 足利義昭 | `b07e_teleport_sound` | sound |
| 9 | B07 足利義昭 | `b07t_voice` | sound |
| 9 | C08 加藤段藏 | `c08r_cast_gesture` | gesture |
| 9 | C08 加藤段藏 | `c08t_title` | sound |
| 9 | C18 小早川隆景 | `c18t_cast_sound` | sound |

## 已有生成參考圖的 cue（遊戲內仍播通用佔位特效，Editor 做特效時參考）（73）

| 批 | 英雄 | cue | kind |
|---|---|---|---|
| 1 | A01 德川家康 | `a01w_holy_bolt` | effect |
| 1 | B20 山縣昌景 | `b20t_epicenter` | effect |
| 1 | B26 大熊朝秀 | `b26e_model_scale_up` | effect |
| 1 | A26 濃姬 | `a26t_big_explosion` | effect |
| 1 | A26 濃姬 | `a26t_small_explosion` | effect |
| 1 | B27 小島貞興 | `b27r_model_grow` | effect |
| 1 | C03 島津義弘 | `c03t_hammer` | effect |
| 1 | C03 島津義弘 | `c03t_hammer_hit` | effect |
| 1 | C03 島津義弘 | `c03w_grow` | effect |
| 2 | C05 前田慶次 | `c05e_impact` | impact |
| 2 | A24 佐久間盛政 | `a24_blood_spray` | effect |
| 2 | C01 明智光秀 | `c01e_light_strike` | effect |
| 2 | A23 宇喜多秀家 | `a23t_cast` | effect |
| 3 | C14 齋藤義龍 | `c14e_breath` | projectile |
| 3 | C14 齋藤義龍 | `c14e_hit_sound` | impact |
| 3 | A22 仙石秀久 | `a22e_bolt` | projectile |
| 3 | A30 丹羽長秀 | `a30e_toss` | impact |
| 3 | A30 丹羽長秀 | `a30w_wolf_howl` | effect |
| 3 | A36 前野長康 | `a36e_explosion` | effect |
| 3 | B02 風魔小太郎 | `b02w_spirit_lance` | effect |
| 4 | C02 明智秀滿 | `c02t_riptide` | effect |
| 4 | B03 齋藤朝信 | `b03t_hit_sound` | impact |
| 4 | B03 齋藤朝信 | `b03w_split_earth` | effect |
| 4 | B11 菊姬 | `b11e_troll_priest_heal` | effect |
| 4 | B11 菊姬 | `b11f_blade_glow` | effect |
| 4 | B09 真田昌幸 | `b09e_pulse` | effect |
| 4 | A20 九鬼嘉隆 | `a20r_wave` | projectile |
| 4 | A20 九鬼嘉隆 | `a20t_tidehunter_model` | aura |
| 5 | B36 直江兼續 | `b36e_cast` | effect |
| 5 | C12 大友宗麟 | `c12r_mask` | aura |
| 5 | C12 大友宗麟 | `c12t_beam` | projectile |
| 5 | C12 大友宗麟 | `c12w_impact` | effect |
| 5 | B10 三好長慶 | `b10w_spirit_hit_fx` | effect |
| 5 | B19 宇佐美定滿 | `b19t_sound` | channel |
| 5 | C11 立花宗茂 | `c11w_arc_lightning` | effect |
| 5 | C11 立花宗茂 | `c11w_bolt` | projectile |
| 5 | C16 果心居士 | `c16r_spirit_hit_fx` | summon |
| 5 | C16 果心居士 | `c16t_split_earth` | effect |
| 6 | A10 平手政秀 | `a10e_hit_burst` | impact |
| 6 | A10 平手政秀 | `a10e_sword_wave` | projectile |
| 6 | A25 織田信長 | `a25d_message` | effect |
| 6 | A25 織田信長 | `a25d_press_cast` | effect |
| 6 | A25 織田信長 | `a25t_small_tornado` | effect |
| 6 | A25 織田信長 | `a25t_tornado` | effect |
| 6 | A29 石田三成 | `a29e_overgrowth_fx` | aura |
| 6 | A29 石田三成 | `a29w_poison_number` | effect |
| 6 | A31 森蘭丸 | `a31t_split_hit` | effect |
| 6 | A32 瀧川一益 | `a32w_thunderclap` | effect |
| 7 | C13 龍造寺隆信 | `c13r_wind_fx` | effect |
| 7 | A05 酒井忠次 | `a05d_die_gesture` | aura |
| 7 | C21 宮本武藏 | `c21t_flash` | effect |
| 7 | A15 寧寧 | `a15e_moonfall` | effect |
| 7 | A15 寧寧 | `a15r_glaive` | projectile |
| 7 | A15 寧寧 | `a15t_split_hit_sound` | impact |
| 8 | B23 伊勢姬 | `b23d_ghost_death` | effect |
| 8 | C10 香宗我部親泰 | `c10e_manavoid_cast` | effect |
| 8 | C10 香宗我部親泰 | `c10r_cleave` | effect |
| 8 | A28 羽柴秀吉 | `a28tw_fire_projectile` | projectile |
| 8 | A28 羽柴秀吉 | `a28w_frog_model` | aura |
| 8 | A21 瑞龍院日秀 | `a21w_attack_projectile` | aura |
| 8 | B13 百地三太夫 | `b13d_burrow_animation` | aura |
| 8 | B13 百地三太夫 | `b13d_crater_model` | aura |
| 8 | B13 百地三太夫 | `b13e_bell_toll` | effect |
| 8 | A09 蜂須賀政勝 | `a09e_hex` | effect |
| 9 | B15 今川義元 | `b15d_split_hit` | impact |
| 9 | C07 立花道雪 | `c07d_attack_pose` | effect |
| 9 | C07 立花道雪 | `c07d_pose_freeze` | aura |
| 9 | C07 立花道雪 | `c07t_bolt` | effect |
| 9 | B07 足利義昭 | `b07w_impact` | effect |
| 9 | C08 加藤段藏 | `c08t_cheese` | effect |
| 9 | B24 秋山信友 | `b24r_avalanche` | projectile |
| 9 | B22 太原雪齋 | `hero_lina_dragon_slave` | effect |
| 9 | C06 石川五右衛門 | `item_money_bag` | effect |

