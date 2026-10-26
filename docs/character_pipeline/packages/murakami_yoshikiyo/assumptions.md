# 村上義清（B29 current）— 阻塞與假設

## 阻塞

### E `b29_morale_boost` 士氣提升 — `needs_framework`

來源要求一個**友軍光環**：自身周圍 600 內所有友方英雄與一般單位（含自身，`Aura_ApplyToCaster 1`）攻擊傷害 +7/14/21/28%。引擎缺：

- **友軍範圍查詢**（capability `ally_area_buff_query`，新缺口，建議併入 G-4 `aura`）。`GameWorld` 只有 `query_enemies_in_range`、`disclosed_enemies_in_range`、`query_friendly_towers_in_range`，沒有「半徑內同隊單位」查詢；`ally_area_restore` 只做生命／法力回復，不能套 stat buff。`GameWorld` 方法表已滿（E045），作者不能自己加。
- 可行的補法（給引擎方參考）：仿 `ally_area_restore` 加一個 host 封包，例如 `ally_area_buff::Spec { center, radius, buff_id, duration, payload }`，由 serial settlement 選同隊存活單位並 `add_stat_buff`。有了它，本招就能在 hero UnitScript `on_tick` 每 0.5 秒刷新一次（震地斧的敵方靈光已用同一套 tick 機制驗證）。

這招的永久力量部分（+80/160/240/320 最大生命、+0.4/0.8/1.2/1.6 回血、+4/8/12/16 攻擊）本身做得到（`on_learn` + `add_stat_buff`），但整招狀態以最差的能力為準，不拆招、不只做一半，所以 `rust/No3_morale_boost.rs` 還沒寫；`implementation.note` 記下設計。

另一個小缺口：package `stats` 詞彙（`package_v2.lua` `M.STATS`）沒有最大生命，只能以 effect 的說明數值欄位 `max_hp_bonus` 帶進 extras（`stat_bonus_permanent_max_hp_bonus`），handler 再對應 `StatKey::HealthBonus`。

### 已知的引擎限制（不阻塞，已改編）

- **R 大地震撼沒有前搖**：host pulse channel 規定 `cast_time == 0`（`ability_channel::ability_spec` 遇到前搖即判定 metadata 錯誤而拒絕施放），沒有「前搖 0.3 秒後再開始引導」。改為按下即開始引導（assumption `tremor_cast_point`）。如果要還原，需要 host 支援 windup 後接 channel。
- **package 生成器不輸出 content `channel`**：`scripts/hero_author/generate.lua` 只把 `timing.channel` 轉成 `cast_type = "channeled"`，不輸出 `channel = {anchor, interval_key, pulses_key, radius_key}`。若只靠生成內容，施放會被 `recovery_channel::ability_spec` 判為壞掉的 channel 而拒絕。本 kit 改由 Rust handler 以 `build_ability_ffi_with_execution(..., Some(HostAbilityExecution::PulseChannel {...}))` 自己宣告 host channel，鍵名取自 extras（`damage_wave_interval`、`damage_wave_count`、`shape_radius`）。
- **`aura` 能力狀態**：登錄寫 `needs_framework`（理由是沒有 tick 掛點與友軍查詢）。對**敵方**光環兩者都已存在：`unit_script = true` 的英雄每 tick 都會收到 `UnitScript::on_tick`（`dispatch.rs` `bind_hero_unit_scripts`），敵人可用 `query_enemies_in_range` 查。所以 `capabilities.lua` 以 `override_reason` 把 `aura` 標為 supported（只涵蓋敵方光環，已跑 parity soak），友軍光環另以 `ally_area_buff_query` 記為缺口。

## 假設

以下逐項對應 `source.lua` 的 assumptions。

- `distance_units`：來源 1 距離單位 = 本遊戲 1 世界單位。
- `variant_current_only`：只用 current 版的 W／E／R／T；`_old` 技能（武王丸、舊動地斧、舊震地斧、舊士氣提升、舊大地震撼）都不使用。
- `non_kit_slots`：欄 4 `magical_resistance` 是所有英雄共用的通用被動，欄 5 `B29_precache` 只預載資源，兩者都不是本 kit 的技能。
- `base_stats_from_local_melee`：來源生命／法力由屬性推得，攻擊 120 是自訂值；本遊戲不用屬性，生命 680、法力 240、攻擊 60 沿用本地近戰英雄基準。護甲 5、移速 350、攻擊前搖 0.3、攻擊間隔 1.8、近戰射程 128 照來源。
- `vscripts_missing`：所有 `RunScript` 指向的 B29.lua／dummy.lua 都不在來源 repo；行為由 KV 與描述推定，不假裝還原原實作。
- `spell_immunity_not_modeled`：本遊戲沒有魔免狀態，來源的 SpellImmunityType 不另外處理。
- `ground_slam_targets`（Q 動地斧）：以自身為中心半徑 500，打所有存活敵方英雄與小兵（不打建築），不看視野；前搖 0.3 秒後結算。
- `quake_leap`（W 震地斧）：沒有施法距離，朝指定點跳，最多 300；撞牆會截斷；被定身、貼牆或點在腳下時原地猛擊。
- `quake_slow_duration`（W）：描述沒寫減速秒數，取 KV `modifier_B29E_debuff` 的 8 秒；猛擊 200 傷害、半徑 500。
- `quake_aura_kv`（W 破軍靈光）：持續 8 秒，每 0.5 秒對施法者周圍 800 內的敵方英雄與小兵套 1 秒減甲 −4/7/10/13（同一個 buff id，刷新不疊加）；施放瞬間先套一次。施法者死亡就不再刷新。
- `morale_radius`（E）：AOERadius 寫 900，但實際光環半徑 `Aura_Radius` 600，含自身；「攻擊傷害 %」換成 `base_damage_outgoing_pct`。
- `morale_passive_cast`（E）：E 是被動，KV 裡的 OnSpellStart 永遠不會觸發，不實作。
- `strength_conversion`（E）：力量 +4/8/12/16 依 Dota 規則換成 +20 生命／點、+0.1 回血／點，主屬性再 +1 攻擊／點。
- `tremor_waves`（R）：引導時間 4.5/5.2/5.9 秒只夠 45/52/59 波，但描述與 B29T_Amount 都是 50/60/70 波；採 50/60/70 波，每 0.1 秒一波（引導 5/6/7 秒）。
- `tremor_wave_position`（R）：每波中心是施法點半徑 700/800/900 內的隨機點（方向、距離各自均勻），用（施法者、施法點、波序）的決定性雜湊選出，權威端與 replica 一定一致；每波 340 內敵人受 60 魔法傷害、暈眩 0.2/0.3/0.4 秒。
- `tremor_levels`（R）：6／12／18 級解鎖 3 個 rank。
- `tremor_cast_point`（R）：見上方「已知的引擎限制」，0.3 秒前搖改為 0。
