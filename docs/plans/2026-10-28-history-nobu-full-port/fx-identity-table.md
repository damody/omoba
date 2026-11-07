# 10 位英雄特效識別表

由 `fx_family.lua` 配方與各英雄 package 產生。標準見使用者要求「所有特效都要長的不一樣 能識別，且顏色角度都要正確」。

- **造型**：系統圖（空中）＋地:地面圖。
- **方向**：配方實際能給的方向；「面向鏡頭，無方向」就是沒有依招式方向轉。
- **重複**：同一造型還用在幾招（含自己，跨英雄）。1 才算獨特。
- **顏色判定**：只比對招式名與描述裡的雷、火、治療、冰字樣；沒有這些字樣的標「需人工」。

| 英雄 | 招 | 招式 | cue | 造型 | 顏色 | 方向 | 重複 | 顏色判定 |
|---|---|---|---|---|---|---|---|---|
| tokugawa_ieyasu | Q | 東照權現 | `a01w_healing_ring` | spark_star | 綠 | 面向鏡頭，無方向 | 9 | 需人工 |
| tokugawa_ieyasu | Q | 東照權現 | `a01w_holy_bolt` | heal_ring+地:heal_ring | 綠 | 面向鏡頭＋貼地 | 2 | 需人工 |
| tokugawa_ieyasu | W | 天羅地網 | `a01e_net` | net | 橙 | 面向鏡頭，無方向 | 1 | 需人工 |
| tokugawa_ieyasu | W | 天羅地網 | `a01e_net_land` | net+地:net | 橙 | 面向鏡頭＋貼地 | 1 | 需人工 |
| tokugawa_ieyasu | W | 天羅地網 | `a01e_vo` | net | 橙 | 面向鏡頭，無方向 | 1 | 需人工 |
| tokugawa_ieyasu | E | 雷霆刃 | `a01r_lightning` | blink_flash | 青藍 | 面向鏡頭，無方向 | 7 | 雷：符合 |
| tokugawa_ieyasu | E | 雷霆刃 | `a01r_thunder_sound` | bolt_fork+地:shock_ring | 青藍 | 面向鏡頭＋貼地 | 1 | 雷：符合 |
| tokugawa_ieyasu | E | 雷霆刃 | `a01r_vo` | bolt_fork+地:shock_ring | 青藍 | 面向鏡頭＋貼地 | 1 | 雷：符合 |
| tokugawa_ieyasu | R | 三河之勇 | `a01t_banner` | 地:banner_rally | 橙 | 貼地 | 1 | 需人工 |
| tokugawa_ieyasu | R | 三河之勇 | `a01t_ring` | 地:shock_ring | 橙 | 貼地 | 3 | 需人工 |
| tokugawa_ieyasu | R | 三河之勇 | `a01t_vo` | banner_rally | 橙 | 面向鏡頭，無方向 | 1 | 需人工 |
| tokugawa_ieyasu | R | 三河之勇 | `spell_hint_self` | 地:arcane_circle | 紫 | 貼地 | 2 | 需人工 |
| tokugawa_ieyasu | X | 天下之勢 | `a01d_restore` | heal_cross | 綠 | 面向鏡頭，無方向 | 1 | 治療：符合 |
| yamagata_masakage | Q | 暈眩 | `b20w_pick_sound` | rock+地:dust_crack | 橙 | 面向鏡頭＋貼地 | 2 | 需人工 |
| yamagata_masakage | Q | 暈眩 | `b20w_rock` | (sparks) | 橙 | 飛向目標，圖不轉向 | 1 | 需人工 |
| yamagata_masakage | Q | 暈眩 | `b20w_rock_land` | rock+地:dust_crack | 未指定（貼圖原色） | 面向鏡頭＋貼地 | 2 | 需人工 |
| yamagata_masakage | W | 暈眩 | `b20e_burrow` | smoke_sheet+地:dust_crack | 未指定（貼圖原色） | 面向鏡頭＋貼地 | 2 | 需人工 |
| yamagata_masakage | W | 暈眩 | `b20e_thunderclap` | shock_ring | 橙 | 面向鏡頭，無方向 | 1 | 需人工 |
| yamagata_masakage | E | 強韌 | `b20r_restore` | heal_ring | 綠 | 面向鏡頭，無方向 | 1 | 治療：符合 |
| yamagata_masakage | R | 赤備猛攻 | `b20t_epicenter` | blink_flash | 紅 | 面向鏡頭，無方向 | 7 | 需人工 |
| yamagata_masakage | R | 赤備猛攻 | `move_slow` | 地:slow_chains | 藍 | 貼地 | 3 | 需人工 |
| okuma_tomohide | Q | 蓄力波（蓄力） | `b26d_thunder_clap` | shock_ring+地:shock_ring | 橙 | 面向鏡頭＋貼地 | 2 | 需人工 |
| okuma_tomohide | Q | 蓄力波（蓄力） | `b26w_charge_fire` | flame_sheet | 紅 | 面向鏡頭，無方向 | 4 | 需人工 |
| okuma_tomohide | Q | 蓄力波（蓄力） | `b26w_charge_pulse` | flame_sheet+地:shock_ring | 橙 | 面向鏡頭＋貼地 | 3 | 需人工 |
| okuma_tomohide | Q | 蓄力波（蓄力） | `b26w_fire_explode` | flame_sheet | 橙 | 面向鏡頭，無方向 | 4 | 需人工 |
| okuma_tomohide | Q | 蓄力波（蓄力） | `b26w_shockwave` | flame_sheet | 橙 | 面向鏡頭，無方向 | 4 | 需人工 |
| okuma_tomohide | Q | 蓄力波（蓄力） | `b26w_slow_overhead` | slow_chains | 青藍 | 面向鏡頭，無方向 | 2 | 需人工 |
| okuma_tomohide | Q | 蓄力波（蓄力） | `b26w_voice` | vortex+地:vortex | 橙 | 面向鏡頭＋貼地 | 4 | 需人工 |
| okuma_tomohide | W | 血性狂怒 | `b26e_model_scale_up` | blink_flash | 紅 | 面向鏡頭，無方向 | 7 | 需人工 |
| okuma_tomohide | W | 血性狂怒 | `b26e_status_effect` | spark_star | 紅 | 面向鏡頭，無方向 | 9 | 需人工 |
| okuma_tomohide | W | 血性狂怒 | `b26e_voice` | flame_sheet+地:shock_ring | 紅 | 面向鏡頭＋貼地 | 3 | 需人工 |
| okuma_tomohide | E | 上野武勇 | `b26r_restore` | heal_rise | 綠 | 面向鏡頭，無方向 | 1 | 治療：符合 |
| okuma_tomohide | R | 當者披靡 | `b26t_burst` | flame_sheet | 未指定（貼圖原色） | 面向鏡頭，無方向 | 4 | 需人工 |
| okuma_tomohide | R | 當者披靡 | `b26t_voice_roar` | flame_sheet+地:shock_ring | 橙 | 面向鏡頭＋貼地 | 3 | 需人工 |
| okuma_tomohide | R | 當者披靡 | `b26t_wings` | flame_sheet | 紅 | 面向鏡頭，無方向 | 4 | 需人工 |
| okuma_tomohide | X | 釋放蓄力波 | `b26d_thunder_clap` | shock_ring+地:shock_ring | 橙 | 面向鏡頭＋貼地 | 2 | 需人工 |
| okuma_tomohide | X | 釋放蓄力波 | `b26d_voice` | blink_flash | 橙 | 面向鏡頭，無方向 | 7 | 需人工 |
| sasaki_kojiro | Q | 燕巢 | `c22w_hide_model` | 地:soft_glow | 青藍 | 貼地 | 1 | 需人工 |
| sasaki_kojiro | Q | 燕巢 | `c22w_swallow_ring` | swallow+地:vortex | 青藍 | 面向鏡頭＋貼地 | 1 | 需人工 |
| sasaki_kojiro | Q | 燕巢 | `c22w_voice` | swallow | 青藍 | 面向鏡頭，無方向 | 1 | 需人工 |
| sasaki_kojiro | W | 飛燕暈眩 | `c22e_impact` | crescent_slash+地:blink_flash | 青藍 | 面向鏡頭＋貼地 | 1 | 需人工 |
| sasaki_kojiro | W | 飛燕暈眩 | `c22e_swallow` | soft_glow | 青藍 | 沿飛行方向 | 2 | 需人工 |
| sasaki_kojiro | W | 飛燕暈眩 | `c22e_voice` | crescent_slash | 青藍 | 面向鏡頭，無方向 | 5 | 需人工 |
| sasaki_kojiro | E | 祕劍．燕閃 | 無 cue | — | — | — | — | 沒有特效 |
| sasaki_kojiro | R | 滅剎夢霞暈眩 | `c22t_thunder_clap` | cut_cross+地:soft_glow | 青藍 | 面向鏡頭＋貼地 | 1 | 需人工 |
| sasaki_kojiro | X | 嚴流 | `c22d_cast_sound` | blink_flash+地:shock_ring | 青藍 | 面向鏡頭＋貼地 | 3 | 需人工 |
| sasaki_kojiro | X | 嚴流 | `c22d_hand_glow` | spark_star | 青藍 | 面向鏡頭，無方向 | 9 | 需人工 |
| sasaki_kojiro | X | 嚴流 | `c22d_return_projectile` | 地:lightning | 青藍 | 地面直線朝目標 | 1 | 需人工 |
| sasaki_kojiro | X | 嚴流 | `c22r_crit_sound` | blink_flash+地:crescent_slash | 橙 | 面向鏡頭＋貼地 | 1 | 需人工 |
| otani_yoshitsugu | Q | 暈眩 | `a12w_burst` | blink_flash | 紫 | 面向鏡頭，無方向 | 7 | 需人工 |
| otani_yoshitsugu | Q | 暈眩 | `a12w_glow` | blink_flash+地:shock_ring | 紫 | 面向鏡頭＋貼地 | 3 | 需人工 |
| otani_yoshitsugu | Q | 暈眩 | `a12w_vo` | blink_flash | 紫 | 面向鏡頭，無方向 | 7 | 需人工 |
| otani_yoshitsugu | W | 玄能通貫 | `a12e_heal_number` | heal_ring+地:heal_ring | 綠 | 面向鏡頭＋貼地 | 2 | 治療：符合 |
| otani_yoshitsugu | W | 玄能通貫 | `a12e_mana_number` | spark_star+地:arcane_circle | 青藍 | 面向鏡頭＋貼地 | 3 | 治療：不符 |
| otani_yoshitsugu | W | 玄能通貫 | `a12e_vo` | lightning+地:arcane_circle | 紫 | 面向鏡頭＋貼地 | 2 | 治療：不符 |
| otani_yoshitsugu | E | 為智畏服 | `a12r_burst` | vortex+地:vortex | 紫 | 面向鏡頭＋貼地 | 4 | 需人工 |
| otani_yoshitsugu | E | 為智畏服 | `a12r_vo` | arcane_circle | 紫 | 面向鏡頭，無方向 | 3 | 需人工 |
| otani_yoshitsugu | E | 為智畏服 | `a12r_vortex` | vortex | 紫 | 面向鏡頭，無方向 | 2 | 需人工 |
| otani_yoshitsugu | R | 玄能附體 | `a12t_aura` | lightning | 紫 | 面向鏡頭，無方向 | 4 | 治療：不符 |
| otani_yoshitsugu | R | 玄能附體 | `a12t_cast` | soft_glow+地:shock_ring | 紫 | 面向鏡頭＋貼地 | 1 | 治療：不符 |
| otani_yoshitsugu | R | 玄能附體 | `spell_hint_self` | 地:arcane_circle | 紫 | 貼地 | 2 | 治療：不符 |
| otani_yoshitsugu | X | 奇計層數 | `a12d_ready` | spark_star+地:arcane_circle | 橙 | 面向鏡頭＋貼地 | 3 | 需人工 |
| otani_yoshitsugu | X | 奇計層數 | `a12d_stack` | arcane_circle | 紫 | 面向鏡頭，無方向 | 3 | 需人工 |
| otani_yoshitsugu | C | 玄滅箭 | `a12f_arc_sound` | soft_glow | 紫 | 面向鏡頭，無方向 | 2 | 需人工 |
| otani_yoshitsugu | C | 玄滅箭 | `a12f_arrow` | spark_star | 紫 | 沿飛行方向 | 9 | 需人工 |
| otani_yoshitsugu | C | 玄滅箭 | `a12f_hit` | blink_flash+地:blink_flash | 紫 | 地面直線朝目標 | 1 | 需人工 |
| otani_yoshitsugu | C | 玄滅箭 | `a12f_streak` | 地:chain_bolt | 紫 | 地面直線朝目標 | 1 | 需人工 |
| otani_yoshitsugu | 附屬 | 暈眩 | `a12w_hide_ring` | vortex+地:vortex | 紫 | 面向鏡頭＋貼地 | 4 | 需人工 |
| otani_yoshitsugu | 附屬 | 暈眩 | `a12w_vo` | blink_flash | 紫 | 面向鏡頭，無方向 | 7 | 需人工 |
| otani_yoshitsugu | 附屬 | 玄能力場 | `a12e_vo` | lightning+地:arcane_circle | 紫 | 面向鏡頭＋貼地 | 2 | 需人工 |
| otani_yoshitsugu | 附屬 | 為智畏服．遠 | `a12r_burst` | vortex+地:vortex | 紫 | 面向鏡頭＋貼地 | 4 | 需人工 |
| otani_yoshitsugu | 附屬 | 為智畏服．遠 | `a12r_vo` | arcane_circle | 紫 | 面向鏡頭，無方向 | 3 | 需人工 |
| otani_yoshitsugu | 附屬 | 為智畏服．遠 | `a12r_vortex` | vortex | 紫 | 面向鏡頭，無方向 | 2 | 需人工 |
| maeda_toshiie | Q | 暈眩 | `a14w_impact` | spark_star+地:dust_crack | 橙 | 面向鏡頭＋貼地 | 1 | 需人工 |
| maeda_toshiie | Q | 暈眩 | `a14w_move_slow` | slow_chains | 青藍 | 面向鏡頭，無方向 | 2 | 需人工 |
| maeda_toshiie | Q | 暈眩 | `a14w_spear_cast` | chain_bolt | 橙 | 面向鏡頭，無方向 | 1 | 需人工 |
| maeda_toshiie | W | 暈眩 | `a14e_arrive` | blink_flash+地:shock_ring | 橙 | 面向鏡頭＋貼地 | 3 | 需人工 |
| maeda_toshiie | W | 暈眩 | `a14e_blink_sparkles` | blink_flash+地:lightning | 橙 | 地面直線朝目標 | 1 | 需人工 |
| maeda_toshiie | W | 暈眩 | `a14e_hit` | 地:fire_wings | 橙 | 貼地 | 1 | 需人工 |
| maeda_toshiie | E | 疊擊槍術層數 | `a14r_stack_hit` | stack_spears+地:shock_ring | 橙 | 面向鏡頭＋貼地 | 1 | 需人工 |
| maeda_toshiie | R | 狂怒層數 | `a14t_lightning` | lightning | 青藍 | 面向鏡頭，無方向 | 4 | 需人工 |
| maeda_toshiie | R | 狂怒層數 | `a14t_title` | lightning+地:shock_ring | 青藍 | 面向鏡頭＋貼地 | 3 | 需人工 |
| maeda_toshiie | R | 狂怒層數 | `a14t_vo` | spark_star+地:shock_ring | 青藍 | 面向鏡頭＋貼地 | 2 | 需人工 |
| nohime | Q | 連銃 | `a26w_flak_sound` | blink_flash | 橙 | 面向鏡頭，無方向 | 7 | 需人工 |
| nohime | Q | 連銃 | `a26w_hit_explosion` | flame_sheet+地:dust_crack | 紅 | 面向鏡頭＋貼地 | 3 | 需人工 |
| nohime | W | 暈眩 | `a26e_rocket_sound` | flame_sheet | 紅 | 面向鏡頭，無方向 | 4 | 需人工 |
| nohime | W | 暈眩 | `a26e_smoke` | smoke_sheet+地:dust_crack | 未指定（貼圖原色） | 地面直線朝目標 | 2 | 需人工 |
| nohime | W | 暈眩 | `a26e_voice` | crescent_slash | 橙 | 面向鏡頭，無方向 | 5 | 需人工 |
| nohime | E | 二丁銃 | `a26r_crit_gesture` | 地:twin_muzzle | 未指定（貼圖原色） | 貼地 | 1 | 需人工 |
| nohime | R | 投擲架勢 | `a26t_big_explosion` | flame_sheet+地:dust_crack | 紅 | 面向鏡頭＋貼地 | 3 | 需人工 |
| nohime | R | 投擲架勢 | `a26t_bomb_arcana` | flame_sheet | 紅 | 沿飛行方向 | 4 | 需人工 |
| nohime | R | 投擲架勢 | `a26t_overhead_bomb` | spark_star | 橙 | 面向鏡頭，無方向 | 9 | 需人工 |
| nohime | R | 投擲架勢 | `a26t_small_bomb` | flame_sheet | 紅 | 沿飛行方向 | 4 | 需人工 |
| nohime | R | 投擲架勢 | `a26t_small_explosion` | rock+地:dust_crack | 橙 | 面向鏡頭＋貼地 | 2 | 需人工 |
| nohime | R | 投擲架勢 | `a26t_voice` | spark_star+地:arcane_circle | 橙 | 面向鏡頭＋貼地 | 3 | 需人工 |
| nohime | R | 投擲架勢 | `spell_hint` | 地:shock_ring | 橙 | 貼地 | 3 | 需人工 |
| kojima_sadaoki | Q | 騎術 | `b27w_armlet_glow` | spark_star | 紅 | 面向鏡頭，無方向 | 9 | 需人工 |
| kojima_sadaoki | Q | 騎術 | `b27w_vo` | smoke_sheet+地:shock_ring | 未指定（貼圖原色） | 面向鏡頭＋貼地 | 1 | 需人工 |
| kojima_sadaoki | W | 剛力擊 | `b27e_axe` |  | 未指定（貼圖原色） | 沿飛行方向 | 2 | 需人工 |
| kojima_sadaoki | W | 剛力擊 | `b27e_hit_sound` | crescent_slash+地:dust_crack | 橙 | 面向鏡頭＋貼地 | 1 | 需人工 |
| kojima_sadaoki | W | 剛力擊 | `b27e_vo` | crescent_slash | 橙 | 面向鏡頭，無方向 | 5 | 需人工 |
| kojima_sadaoki | E | 無畏衝鋒 | `b27r_b15w3_aura` | spark_star | 橙 | 面向鏡頭，無方向 | 9 | 需人工 |
| kojima_sadaoki | E | 無畏衝鋒 | `b27r_model_grow` | iron_shield+地:iron_shield | 橙 | 面向鏡頭＋貼地 | 1 | 需人工 |
| kojima_sadaoki | E | 無畏衝鋒 | `b27r_vo` | flame_sheet+地:banner_rally | 紅 | 面向鏡頭＋貼地 | 1 | 需人工 |
| kojima_sadaoki | R | 裂地刃 | `b27t_move_slow` | 地:slow_chains | 藍 | 貼地 | 3 | 需人工 |
| kojima_sadaoki | R | 裂地刃 | `b27t_screen_shake` | spark_star+地:shock_ring | 橙 | 面向鏡頭＋貼地 | 2 | 需人工 |
| kojima_sadaoki | R | 裂地刃 | `b27t_slam` | rock | 未指定（貼圖原色） | 面向鏡頭，無方向 | 1 | 需人工 |
| kojima_sadaoki | R | 裂地刃 | `b27t_vo` | crescent_slash | 橙 | 面向鏡頭，無方向 | 5 | 需人工 |
| shimazu_yoshihiro | Q | 鐵布衫 | `c03w_grow` | spark_star+地:iron_shield | 青藍 | 面向鏡頭＋貼地 | 1 | 需人工 |
| shimazu_yoshihiro | Q | 鐵布衫 | `c03w_iron_glow` | spark_star | 青藍 | 面向鏡頭，無方向 | 9 | 需人工 |
| shimazu_yoshihiro | Q | 鐵布衫 | `c03w_vo` | iron_shield | 青藍 | 面向鏡頭，無方向 | 1 | 需人工 |
| shimazu_yoshihiro | W | 震地斧緩速 | `c03e_move_slow` | 地:slow_chains | 藍 | 貼地 | 3 | 需人工 |
| shimazu_yoshihiro | W | 震地斧緩速 | `c03e_thunder_clap` | (sparks)+地:quake_burst | 未指定（貼圖原色） | 貼地 | 1 | 需人工 |
| shimazu_yoshihiro | W | 震地斧緩速 | `c03e_vo` | crescent_slash | 紅 | 面向鏡頭，無方向 | 5 | 需人工 |
| shimazu_yoshihiro | E | 暴雷暈眩 | `c03r_thunder` | lightning+地:blink_flash | 紫 | 面向鏡頭＋貼地 | 3 | 雷：符合 |
| shimazu_yoshihiro | R | 不動戰錘暈眩 | `c03t_hammer` |  | 未指定（貼圖原色） | 飛向目標，圖不轉向 | 2 | 需人工 |
| shimazu_yoshihiro | R | 不動戰錘暈眩 | `c03t_hammer_hit` | flame_sheet+地:dust_crack | 紅 | 面向鏡頭＋貼地 | 3 | 需人工 |
| maeda_keiji | Q | 松風 | `c05w_armlet` | spark_star | 青藍 | 面向鏡頭，無方向 | 9 | 需人工 |
| maeda_keiji | Q | 松風 | `c05w_vo` | vortex+地:vortex | 青藍 | 面向鏡頭＋貼地 | 4 | 需人工 |
| maeda_keiji | W | 雷舞 | `c05e_bolt` | lightning | 藍 | 沿飛行方向 | 4 | 雷：符合 |
| maeda_keiji | W | 雷舞 | `c05e_cast_sound` | lightning+地:shock_ring | 藍 | 面向鏡頭＋貼地 | 3 | 雷：符合 |
| maeda_keiji | W | 雷舞 | `c05e_impact` | lightning+地:blink_flash | 藍 | 面向鏡頭＋貼地 | 3 | 雷：符合 |
| maeda_keiji | E | 雷鳴豪槍緩速 | `c05r_chain_lightning` | lightning+地:chain_bolt | 藍 | 地面直線朝目標 | 1 | 雷：符合 |
| maeda_keiji | E | 雷鳴豪槍緩速 | `c05r_tempest` | lightning | 藍 | 面向鏡頭，無方向 | 4 | 雷：符合 |
| maeda_keiji | R | 瞬雷閃引導 | `c05t_aoe_hint` | 地:shock_ring | 藍 | 貼地 | 3 | 雷：符合 |
| maeda_keiji | R | 瞬雷閃引導 | `c05t_aoe_hint_enemy` | 地:shock_ring | 紅 | 貼地 | 3 | 雷：不符 |
| maeda_keiji | R | 瞬雷閃引導 | `c05t_blink_end` | lightning+地:shock_ring | 藍 | 面向鏡頭＋貼地 | 3 | 雷：符合 |
| maeda_keiji | R | 瞬雷閃引導 | `c05t_blink_start` | lightning+地:blink_flash | 藍 | 面向鏡頭＋貼地 | 3 | 雷：符合 |
| maeda_keiji | R | 瞬雷閃引導 | `c05t_eclipse_area` | lightning+地:shock_ring | 藍 | 面向鏡頭＋貼地 | 3 | 雷：符合 |
| maeda_keiji | R | 瞬雷閃引導 | `c05t_vo` | lightning+地:vortex | 藍 | 面向鏡頭＋貼地 | 1 | 雷：符合 |

共 129 個 cue，造型獨特 38 個；顏色字樣不符 6 個；沒有 cue 的招 1 個。

## 共用最多的造型

- `spark_star`（9 招）：tokugawa_ieyasu Q、okuma_tomohide W、sasaki_kojiro X、otani_yoshitsugu C、nohime R、kojima_sadaoki Q、kojima_sadaoki E、shimazu_yoshihiro Q、maeda_keiji Q
- `blink_flash`（7 招）：tokugawa_ieyasu E、yamagata_masakage R、okuma_tomohide W、okuma_tomohide X、otani_yoshitsugu Q、otani_yoshitsugu 附屬、nohime Q
- `crescent_slash`（5 招）：sasaki_kojiro W、nohime W、kojima_sadaoki W、kojima_sadaoki R、shimazu_yoshihiro W
- `flame_sheet`（4 招）：okuma_tomohide Q、okuma_tomohide R、nohime W、nohime R
- `lightning`（4 招）：otani_yoshitsugu R、maeda_toshiie R、maeda_keiji W、maeda_keiji E
- `vortex+地:vortex`（4 招）：okuma_tomohide Q、otani_yoshitsugu E、otani_yoshitsugu 附屬、maeda_keiji Q
- `arcane_circle`（3 招）：otani_yoshitsugu E、otani_yoshitsugu X、otani_yoshitsugu 附屬
- `blink_flash+地:shock_ring`（3 招）：sasaki_kojiro X、otani_yoshitsugu Q、maeda_toshiie W
- `flame_sheet+地:dust_crack`（3 招）：nohime Q、nohime R、shimazu_yoshihiro R
- `flame_sheet+地:shock_ring`（3 招）：okuma_tomohide Q、okuma_tomohide W、okuma_tomohide R
- `lightning+地:blink_flash`（3 招）：shimazu_yoshihiro E、maeda_keiji W、maeda_keiji R
- `lightning+地:shock_ring`（3 招）：maeda_toshiie R、maeda_keiji W、maeda_keiji R
- `spark_star+地:arcane_circle`（3 招）：otani_yoshitsugu W、otani_yoshitsugu X、nohime R
- `地:shock_ring`（3 招）：tokugawa_ieyasu R、nohime R、maeda_keiji R
- `地:slow_chains`（3 招）：yamagata_masakage R、kojima_sadaoki R、shimazu_yoshihiro W
