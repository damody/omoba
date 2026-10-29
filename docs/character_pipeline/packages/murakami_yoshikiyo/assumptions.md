# 村上義清（B29 current）— 阻塞與假設

原作腳本都在來源 repo（`history_nobu@f966720`）：`S/vscripts/heroes/B_Unified/B29.lua`、`S/vscripts/library/common/dummy.lua`。每招的原作行為與移植差異見 `parity.md`；結構化版本在 `source.lua` 的 assumptions。

## 阻塞（needs_framework）

- **魔法免疫狀態與目標旗標**：W 動地斧、E 震地斧原作不打魔免敵人（旗標 0），T 大地震撼打魔免（`MAGIC_IMMUNE_ENEMIES`）。引擎沒有魔免狀態，現在等同全部可打。
- **為敵方隊伍開視野**：B29T 在施法者位置對雙方各開半徑 100 視野（`B29.lua:147-148`、`164-165`）。已用 reveal_zone `teams: all` 實作（E185）。

## 已用通用能力補上

- **delayed_effect**（omoba-core `runtime/native/delayed_effect.rs`）：W 猛擊施放後 0.2 秒依施法者當下位置結算（`B29.lua:63-80`）；T 每擊產生後 0.45 秒才落下，引導結束後已出手的仍落下（`B29.lua:246`）。排程是施法者身上的 buff，施法者死亡時取消（原作 Timer 照常結算）。
- **原作緩速疊加**（omoba-core `BuffStore` `generic_ability_slow`）：不同來源 1−Π(1−sᵢ)、同名只算一筆；`slow_resistance` 依比例削弱（`dummy.lua:816-902`）。
- **一般視野**（reveal_zone `detect=false`）：T 施法點己方視野（`B29.lua:166`），不揭露隱形。

## 假設

- `distance_units`：來源 1 距離單位 = 本遊戲 1 世界單位。
- `variant_current_only`：只用 current 版 W／E／R／T。
- `non_kit_slots`：欄 4 `magical_resistance`、欄 5 `B29_precache` 不屬於本 kit。
- `base_stats_from_local_melee`：基礎數值由 hero_source_stats 以 `B29_2.txt` 為準（E157）。
- `ground_slam_targets`（Q）：半徑 550（腳本寫死；AOERadius 500 只是指示），敵方英雄與一般單位、不含建築。
- `quake_leap`（W）：跳躍最多 300、固定 0.15 秒落地；受地形碰撞截斷；`modifier_phased` 不另外實作。
- `quake_slow_duration`（W）：減速 8 秒（KV `modifier_B29E_debuff`）。
- `quake_slam_delay`（W）：0.2 秒後依施法者當下位置 500 結算。
- `slow_stacking`（W）：見上方緩速疊加；小兵也會被減速（原作只對掛 `slow_self` 的單位生效）。
- `quake_aura_kv`（W）：靈光 8 秒，施放瞬間以起跳點先套一次，之後每 0.5 秒以施法者位置刷新 1 秒減甲。
- `morale_radius`（E）：光環半徑 600、含自身；`DAMAGEOUTGOING_PERCENTAGE` 以 `base_damage_outgoing_pct` 表達。
- `morale_passive_cast`（E）：`B29R_OnSpellStart` 是死碼，不實作。
- `strength_bonus`（E）：力量 +4/8/12/16 送 `StatKey::StatsStrengthBonus`，由三圍 settle 換算。
- `morale_aura_linger`（E）：光環殘留 0.5 秒（Dota 預設），每 0.25 秒刷新。
- `tremor_waves`（R）：45/52/59 擊（引導時間／0.1），45 與 46 的判斷見 `parity.md`。
- `tremor_wave_position`（R）：打擊點距施法點 10～350/400/450；每擊 45 魔法＋0.2 暈眩，建築 30% 不暈眩。
- `tremor_delay`（R）：每擊 0.45 秒後落下。
- `tremor_vision`（R）：施法點己方視野、施法者位置兩隊視野都已實作（E185）；砍樹不適用。
- `tremor_levels`（R）：6／12／18 級。
- `spell_immunity_not_modeled`：見阻塞。
- 魔免（原作設定，E170；通用魔免已接上，E179／E184：B29T `pierces_magic_immunity`，打擊的暈眩穿透魔免，魔法傷害照全域規則歸零；其餘招式的魔法傷害歸零、減益被擋）：B29W 不打魔免（旗標 0、ENEMIES_NO）；B29E 猛擊與減甲靈光不打魔免（ENEMIES_NO）；B29R 友方光環含魔免友軍（MAGIC_IMMUNE_ENEMIES）；B29T 穿透魔免（SPELL_IMMUNITY_ENEMIES_YES、搜尋旗標 MAGIC_IMMUNE_ENEMIES）。
