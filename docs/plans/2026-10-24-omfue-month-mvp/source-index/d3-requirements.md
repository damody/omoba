# D3 候選能力需求表

由 `tools\lua\lua.exe scripts\hero_source_index.lua requirements` 從 `candidate-requirements.lua` 生成，不要手改。
技能與英雄狀態由機制推導（最差者為準：supported < ambiguous < needs_framework < deferred）；每招釘住閱讀時的 `raw_sha256`。

## 總覽

| 來源 | 名稱 | 名單 | 計畫槽 | 定位 | 狀態 | 額外槽 |
|---|---|---|---|---|---|---|
| B01 | 雜賀孫市 | delivery | H01 | 射手（遠程 700） | `supported` | — |
| B04 | 伊達政宗 | delivery | H02 | 近戰／位移 | `supported` | — |
| A34 | 阿松 | delivery | H03 | 施法者／範圍 | `supported` | — |
| C17 | 阿市 | delivery | H04 | 支援／控制 | `supported` | 需要 |
| A02 | 稻姬 | delivery | H05 | 切換／隱形（遠程 700） | `supported` | — |
| A13 | 服部半藏 | delivery | H06 | 特殊（分身／鉤拉／隱形） | `deferred` | 需要 |
| C15 | 玉子 | blind | — | 遠程射手（盲測） | `supported` | — |
| B29 | 村上義清 | blind | — | 近戰／控制（盲測） | `supported` | — |
| B08 | 淺井長政 | backup | — | 近戰（候補 H02／H06） | `supported` | 需要 |
| B28 | 松永久秀 | backup | — | 施法者（候補 H03） | `ambiguous` | — |
| B09 | 真田昌幸 | backup | — | 支援／切換（候補 H04／H05） | `needs_framework` | — |
| A04 | 竹中重治 | backup | — | 召喚／施法者（候補 H05） | `ambiguous` | 需要 |

## 能力登錄（引擎現況）

| capability | 狀態 | G 包 | 證據 | 限制 |
|---|---|---|---|---|
| `unit_damage` | `supported` | — | world.rs deal_damage; EffectOp::Damage (generic_effects.rs); lumen_bolt | 瞬發，含施法距離檢查 |
| `heal` | `supported` | — | world.rs heal; EffectOp::HealSelf/HealAlly |  |
| `shield_absorb` | `supported` | — | EffectOp::ShieldAlly → damage_shield buff; BuffStore::grant_damage_shield/absorb_damage_with_shield | 不疊加：大值與長時間取勝 |
| `timed_stat_buff` | `supported` | — | world.rs add_stat_buff + StatKey（攻擊、攻速、射程、移速、護甲、減傷）；sniper_mode/matchlock_gun/three_stage | 對友軍或複合增益要寫 Rust handler |
| `permanent_stat_bonus` | `supported` | — | AbilityScript::on_learn（ability.rs）+ add_stat_buff | 英雄模板雖有力量／敏捷／智力欄位，但 MOBA 戰鬥使用 authored 數值，舊屬性公式（comp/hero.rs）與 StatsAgilityBonus 在 MOBA 路徑沒有消費者；來源的屬性加成需在 package assumptions 換算成具體 StatKey |
| `slow_debuff` | `supported` | — | EffectOp::SlowEnemy（負 move_speed_bonus）；負攻速 StatKey |  |
| `stun` | `supported` | — | BuffId::Stun；EffectOp::StunEnemy/ControlEnemy；buff_store is_stunned | 來源的「結凍／停頓」以 stun 實作 |
| `root` | `supported` | — | BuffId::Root；ControlEnemy；buff_store is_rooted |  |
| `silence` | `supported` | — | BuffId::Silence；dispatch.rs 施法前檢查 |  |
| `flat_damage_block` | `supported` | — | StatKey::PhysicalConstantBlock 在 unit_stats.rs 結算 |  |
| `evasion` | `supported` | — | UnitStats evasion_chance（unit_stats.rs）；world.rs get_evasion_chance |  |
| `cooldown_reduction` | `supported` | — | UnitStats cooldown_mult（CooldownPercentage[Stacking]）；dispatch.rs prepare_cast_cooldown | 只有百分比，無固定秒數減免或退還 |
| `mana_burn` | `supported` | — | EffectOp::DrainManaEnemy；world.rs spend_mana |  |
| `max_hp_percent_damage` | `supported` | — | world.rs get_max_hp + deal_damage（Rust handler） |  |
| `dispel_self` | `supported` | — | EffectOp::CleanseAlly（可指定自身） |  |
| `on_attack_hook` | `supported` | — | AbilityScript::on_attack_hit（ability.rs）由 dispatch.rs 對被動技能呼叫；rain_iron_cannon | 只對 passive 技能呼叫；在命中後執行（額外傷害為另一筆 damage） |
| `proc_chance` | `supported` | — | world.rs rand_unit（每 tick 決定性 seed）；host attack_stun_chance（matchlock_gun） |  |
| `target_class_multiplier` | `supported` | — | world.rs is_building、unit_id_of；on_attack_hit 內加額外傷害 | 英雄／部隊以 unit_id_of 判別；以額外傷害實作百分比加成 |
| `every_nth_attack` | `supported` | — | on_attack_hit + buff payload 計數（buff_store.rs add 會替換 payload） | 無內建計數器 API |
| `stacking_buff` | `supported` | — | BuffStore::add 單 id 單實例、替換 payload；不同 id 加總 | 無 stack-count API；層數放在 payload，持續時間以最後一次刷新為準 |
| `aoe_circle` | `supported` | — | EffectOp::AreaDamage；base_content/point_blast.rs；world.rs deal_damage_splash/query_enemies_in_range | 以自身為中心需 Rust handler |
| `aoe_line_instant` | `supported` | — | EffectOp::LineDamage（lumen_ray，capsule）；swept_dash.rs | 瞬發、不受地形阻擋、無飛行時間 |
| `dot_debuff` | `supported` | — | EffectOp::DamageOverTimeEnemy；periodic_damage.rs；buff_store drain_periodic_damage | 每次施放一筆 DoT |
| `ground_area_dot` | `supported` | — | native/ability_runtime/damage_zone.rs DamageZoneSpec（圓形、獨立或綁施法者）；point_blast::with_zone | 只有圓形傷害區域、上限 30 秒；不能減速或治療 |
| `toggle` | `supported` | — | AbilityType::Toggle；No1_sniper_mode.rs has_buff/remove_buff 模式 |  |
| `summon_timed` | `supported` | — | world.rs spawn_summoned_unit；native/tick/summon_tick.rs 到期與跟隨；summons.lua 模板 | 需在 summons.lua 新增召喚物模板；沒有每位主人的數量上限 |
| `summon_attack_move` | `supported` | — | native/summon_orders.rs；saika_gunner UnitScript AI | 玩家完整控制仍在 G-3 待完善 |
| `dash_to_point` | `supported` | — | EffectOp::DashToPoint；base_content swept_dash.rs（沿實際路徑傷害、受碰撞截斷）；連續位移：swept_dash::motion_packet → omoba-core forced_motion.rs Mode::Dash（施法者自身、host 依速度逐 tick 移動、撞牆停止、PreStepMovement 事實；不暈眩、移動中受 root），例：date_masamune b04_flame_trail_dash（dash_speed）、fire_dash（authored dash_duration）、murakami b29_quake_axe 跳躍（共用速度） | 路徑效果（傷害、鎖定、落地重擊）在施法時依核准路徑決定，不是在通過或落地當下；未宣告 dash_speed 時用 swept_dash::DASH_SPEED 2000 |
| `blink_to_target` | `supported` | — | omoba-core blink_landing.rs：施法者自身的即時封包，serial settlement 以單點可走性（point_walkable，英雄移動同一套地形檢查加自身碰撞半徑）決定落點：偏好點不可走時，在目標周圍同距離以 15° 步進、左右交替找最近的可走點，並面向目標；例：asai_nagamasa b08_flash_cut | 只在目標周圍同距離的環上找；都不可走時留在原地 |
| `leap` | `supported` | — | 同 dash_to_point（連續位移，forced_motion Mode::Dash；例：murakami b29_quake_axe） | 沒有拋物線／空中狀態；落地重擊在施法時依落點決定 |
| `swap_position` | `supported` | — | world.rs set_pos 對兩個 handle |  |
| `attack_range_switch` | `supported` | — | 英雄普攻一律走投射物（hero_tick.rs ProjectileLine2）；matchlock_gun 以 AttackRangeBonus 讓近戰英雄遠程攻擊 | 沒有真正的近戰／遠程型態切換或模型替換 |
| `random_target_det` | `supported` | — | world.rs rand_unit；查詢結果穩定排序 |  |
| `global_ally_target` | `supported` | — | EffectOp::HealAlly/ShieldAlly 以單位為目標 | range 必填且上限 10000；需確認地圖尺寸內等同全圖 |
| `global_point_target` | `supported` | — | TargetType::Point + range 上限 10000 | 同上 |
| `projectile_tracking` | `supported` | — | G-1（work-G1.md）：omoba-core/src/runtime/native/ability_projectile.rs Homing；Lua projectile 投遞區塊或 Rust 以 ability_projectile::Spec 發射；lumen_frost_bolt | 只命中原目標同一世代；目標失效 expire／last_position 皆不造成傷害；每來源同時最多 32 顆 |
| `projectile_linear` | `supported` | — | G-1：ability_projectile.rs Linear，命中英雄與小兵、max_hits 1～64 穿透、牆 pass／stop；lumen_frost_lance | 飛行距離＝施法距離；寬度固定（不支援起訖寬度不同的錐形） |
| `bounce_chain` | `supported` | G-1 | E163：ability_projectile 追蹤彈的 bounce 列（with_bounce／bounce_of）：每次命中後在命中目標 radius 內以決定性亂數挑另一個可見的敵方英雄或一般單位（只排除當下目標），同一顆投射物重新鎖定，直到總命中數；例：sanada_yukimura b06_flame_fangs | 只限追蹤彈；總命中 2～16、半徑 ≤2000；每跳傷害不變（沒有逐跳加成） |
| `damage_kind_immunity` | `supported` | — | E163：omoba-core damage_immunity.rs，handle_damage 依 absolute_no_damage_{physical,magical,pure} 把該種類傷害歸零；例：honda_tadakatsu a07_unscathed |  |
| `damage_to_heal_conversion` | `supported` | — | E163：同一結算點依 absorb_{physical,magical,pure}_as_heal（0～1）把該種類傷害的一部分轉為回復 |  |
| `status_resistance` | `supported` | — | E163：BuffStore::add 依目標的 status_resistance 縮短暈眩、定身、沉默、技能減速與技能 DoT 的持續時間；例：a07_strongest |  |
| `slow_resistance` | `supported` | — | E163：BuffStore::add 依目標的 slow_resistance 減弱技能減速的移速降低量 |  |
| `self_slow_cleanse` | `supported` | — | E163：ally_cleanse::SELF_SLOW_BUFF 封包，只移除自己的移動減速；例：a07_red_thunder 1 秒後解除自身減速 |  |
| `spell_guard` | `supported` | — | E163：omoba-core spell_guard.rs，敵方指定技能施放成立時依目標的 spell_guard 項目套魔抗並開始冷卻；例：b06_mind_eye | 在施放成立時觸發（原作在下達指令時） |
| `cleave_on_attack` | `supported` | — | 英雄 UnitScript on_attack_hit（E163 起英雄普攻命中也會呼叫）＋get_final_atk＋query_enemies_in_range；例：a07_strongest |  |
| `damage_reduction` | `supported` | — | StatKey::DamageTakenBonus，傷害封包結算時套用 |  |
| `projectile_vs_building` | `supported` | — | ability_projectile::Spec.buildings（Homing 限定；hittable(..., buildings)）＋ability_projectile_tests::homing_reaches_enemy_buildings_only_when_the_spec_allows_it |  |
| `projectile_on_hit_dot` | `supported` | — | ability_projectile::Spec.dot → generic_dot_buff_id＋PeriodicDamageSpec::payload（與宣告式 DoT 同一套堆疊規則）；ability_projectile_tests::on_hit_dot_applies_* |  |
| `retarget_on_death` | `supported` | G-1 | ability_projectile TargetLostPolicy::Retarget(radius)（payload target_lost=retarget + retarget_radius）；追蹤彈目標失效時改鎖投射物 radius 內最近的存活敵方非建築單位（id 決勝） | 只限追蹤彈；以最近（決定性）取代來源的隨機；召喚物改鎖需由召喚物 UnitScript 自行處理 |
| `channel` | `supported` | — | G-2 pulse channel：content channel={anchor=self\|point\|unit, interval_key, pulses_key, radius_key}，host 擁有生命週期（runtime/native/ability_channel.rs）；每波套用 effects（damage／slow／stun）；示範 lumen_tremor、lumen_tether；另有 recovery_channel_self | 每波只能套用宣告式 effects；受控、移動、死亡、回城會中斷 |
| `toggle_mana_per_attack` | `supported` | — | omoba-core runtime/native/attack_orb.rs（G-2）：toggle 標記 buff、每次普攻扣魔、沒藍自動關閉、命中加傷與減速；content AttackOrb；示範 lumen_venom_orb |  |
| `charges` | `supported` | — | G-2（work-G2.md §2）：AbilityDefinition.charges={max_key,recharge_key}；Hero.ability_charges；cast_settlement 扣 1 次 | max 1..=10 且隨 rank 不遞減；recharge 0.25..600 秒；僅 active／ultimate |
| `recast_sub_ability` | `needs_framework` | G-2 | 無 sub-ability／recast 視窗 | 第二個按鍵與窗口都不存在 |
| `delayed_effect` | `supported` | — | E170 omoba-core runtime/native/delayed_effect.rs：技能在施法者身上掛排程 buff（≤10 秒），自然到期時依固定順序呼叫 AbilityScript::on_delayed_effect，只能套效果；例：murakami b29_quake_axe 0.2 秒猛擊、b29_earth_tremor 每擊 0.45 秒落下 | 施法者死亡時取消尚未結算者；不能扣費、施放或生成單位 |
| `fow_viewer` | `supported` | — | E170 reveal_zone detect=false：在指定點為施法者隊伍開一般視野（不偵測隱形），對應原作 AddFOWViewer；例：tamako c15_burst_arrow、murakami b29_earth_tremor | 只能開給施法者自己的隊伍；每隊上限 64 |
| `summon_flying_chase` | `supported` | — | E170 base_content summons/chaser.rs：召喚物直線飛行追擊鎖定目標（忽略地形與碰撞），否則取範圍內最近的已揭露敵人；例：tamako c15_hundred_arrows 箭矢 | 以中心距離判定普攻射程 |
| `projectile_dodge` | `supported` | — | E170 ability_projectile dodgeable 旗標＋躲避封包（DODGE_BUFF）：追蹤施法者的可閃避技能投射物與普攻投射物作廢不結算，對應 ProjectileManager:ProjectileDodge；例：omatsu a34_frost_bolt、tamako c15_scatter_shot、date b04_flame_trail_dash、asai b08_flash_cut／b08_endless_fang | 只由腳本明確呼叫；不會因瞬移道具自動觸發 |
| `projectile_widening` | `supported` | — | E170 ability_projectile end_radius：直線投射物命中半徑隨已飛距離由起始半徑線性增到 end_radius；例：omatsu a34_frost_breath 150→500 | UE 呈現的投射物寬度仍是起始半徑 |
| `discrete_dot` | `supported` | — | E170 ability_projectile dot_interval：命中附帶的 DoT 每 interval 秒結算一次固定量；例：omatsu a34_frost_breath 每秒一跳 | 每次命中一筆 |
| `cooldown_cap_buff` | `supported` | — | E170 omoba-core cooldown_cap.rs：buff 存在期間把指定技能的剩餘冷卻壓到全冷卻×(1−x)，含已在跑的冷卻；例：omatsu a34_rapid_cooling | 只作用於英雄技能，不含道具 |
| `mana_on_cast` | `supported` | — | E170 英雄 UnitScript on_skill_cast（含 E170 物品使用觸發）回復法力；例：omatsu a34_rapid_cooling 主動期間每次施放回魔 |  |
| `on_hit_stacking_debuff` | `supported` | — | E170 omoba-core hit_stack.rs：投射物命中時依施法者身上 buff 的疊層規則（上限、每層 payload、持續、滿層是否刷新）對目標疊層；例：omatsu A34T 主動期間命中降魔抗 | 只在命中當下判定施法者 buff |
| `command_restricted` | `supported` | — | E170 omoba-core command_restriction.rs：期間拒絕新的移動、停止、施法與道具指令；例：asai b08_fang_slash／b08_flash_cut 施放後 0.1 秒 | 召喚物指令與方向手勢不受限 |
| `summon_owner_cap` | `supported` | — | E170 omoba-core summon_owner_cap.rs：每位主人同時存在上限（隨 rank），超出時移除最早生成者；例：saika b01_saika_corps | 移除不給賞金 |
| `attack_launch_hook` | `supported` | — | E170 AbilityScript::on_attack_launch：英雄普攻發射時呼叫已學技能（含延遲發射的投射物）；例：date b04_tanegashima 直線子彈 |  |
| `mana_burn_damage` | `supported` | — | E170 mana_drain damage 選項：燒魔後由 host 依實際扣除量造成指定類型傷害；例：asai b08_flash_cut |  |
| `item_use_cast_trigger` | `supported` | — | E170 ScriptEvent::ItemUsed 呼叫持有者 on_skill_cast("item:<id>")，base_content owner_cast 套用原作排除清單；例：asai b08_living_soul、oichi c17_devotion |  |
| `summon_immobile` | `supported` | — | G-3（work-G2.md §3）：summons.lua immobile、damage_per_owner_level、owner_death；Lua summon={unit,duration_key} | 固定召喚物使用通用砲台 AI；L1 完成，實機待驗 |
| `extra_action_slot` | `supported` | — | ward-canonical 2026-10-25 23:19：英雄 abilities 第 5 項為 D 欄（runtime slot 4），moba_loadout.ranks 可有 5 項；ability auto_learn 依 required_hero_level 自動學會、不耗技能點 | 只有一個額外欄（extra1） |
| `stealth` | `supported` | — | BuffStore::stealth_level（invisibility_level buff，整數層）與 StealthProfile 取高者，visibility read view 與目標可見判定共用；VisionSource.detection_level 決定偵測 |  |
| `stealth_attack_no_reveal` | `supported` | — | buff 隱形沒有破隱規則：攻擊、施法都不會解除，持續到 buff 到期 | 需要「攻擊破隱」的英雄要另加規則 |
| `hero_unit_hooks` | `supported` | — | 英雄 content unit_script=true → rust/hero_unit.rs 的 UnitScript：on_skill_cast（施法交易內、execute 前）、on_attack_hit（普攻命中）、on_damage_taken、on_tick（每 tick；自行以標記 buff 節流） | GameWorld 讀不到技能等級與 buff payload：以 rank 命名的標記 buff 表示學會的 rank |
| `on_cast_trigger` | `supported` | — | 英雄 content unit_script=true → gen_hero_registry units 模式產生 UnitDef（heroes::<module>::hero_unit_script()），host bind_hero_unit_scripts 以 hero id 綁定；on_skill_cast 在施法交易內、execute 之前呼叫 |  |
| `cdr_on_cast_hook` | `supported` | — | content AbilityDefinition.cast_cooldown_refund → omoba-core runtime/native/cooldown_refund.rs，於 serial settlement 的 ScriptStartCooldown／ScriptConsumeCharge 結算；對手 replica 經 CommittedCooldown＋Hero::commit_cooldown（保留 admitted total） |  |
| `mana_shield` | `needs_framework` | G-4 | UnitScript::on_damage_taken 可改傷害，但英雄無 UnitScript | 需要英雄 UnitScript 掛點 |
| `enemy_cast_extends_debuff` | `needs_framework` | G-4 | 需要目標方的 on_skill_cast | 需要英雄 UnitScript 掛點與跨單位事件 |
| `aura` | `supported` | — | 英雄 UnitScript on_tick（content unit_script=true）每隔一段時間刷新：敵方光環用 query_enemies_in_range＋短 buff；友方光環用 ally_area_buff 封包（例：murakami_yoshikiyo b29_quake_axe／b29_morale_boost） |  |
| `ally_area_buff_query` | `supported` | — | omoba-core runtime/native/ally_area_buff.rs：施法者自身的即時封包，serial settlement 依提交位置對同隊非建築單位套上（刷新、不疊加）stat buff | 只能套 stat payload，最多 8 項 |
| `ally_area_query` | `supported` | — | omoba-core runtime/native/ally_area_restore.rs：施法者自身的即時封包，serial settlement 依提交位置選取同隊存活單位，以 received-healing 政策治療最大生命比例，並回復最大法力比例（mana_fraction）；committed vitals 一併揭露。使用者：oichi c17_aurora_guard（只回生命）、c17_devotion（英雄 UnitScript on_skill_cast，回復 5% 生命與 4% 法力） | 只限即時的一次性回復；持續型友軍光環用 ally_area_buff |
| `tether_distance_scaled` | `supported` | — | omoba-core runtime/native/tether.rs：目標身上的 timed buff（__tether_v1）自然到期時由 buff_tick 依施法者距離結算傷害（遠→高）與定身（近→長）；提早移除或任一端死亡則不結算 |  |
| `summon_forced_target` | `supported` | G-3 | 召喚物身上的 buff_ids::SUMMON_TARGET_BUFF（payload {"summon_target":[id,gen]}，summon_target_payload 產生）讓該生命成為優先目標（host 仍檢查射程、視野、存活、可攻擊性）；目標死亡後回到模板優先序（最近） | 目標死亡後改鎖是最近而非隨機；召喚物要有能打到目標的射程或自行移動 |
| `summon_invulnerable` | `supported` | G-3 | summons.lua invulnerable = true：生成時登記到 MobaMatch.invulnerable_summons，current_match_target_attackable（索敵、投射物、區域、傷害結算、replica 揭露閘門共用）一律排除 | 只在 MOBA 對局生效；BuffId::Invulnerable 狀態 buff 仍沒有消費者 |
| `ground_area_line` | `supported` | G-4 | DamageZoneSpec.end（center→end 膠囊）＋grow（隨投射物速度逐段鋪出，僅獨立區域）＋linger（每次 pulse 刷新同一個 DoT，離開後最多再燒 linger 秒）；zone_pulse_settlement 用 point_in_swept_circle | 長度 ≤10000；同一施法者所有 linger 區域共用一個灼燒 |
| `no_unit_collision` | `supported` | — | BuffStore NO_UNIT_COLLISION 狀態 buff：creep_tick 單位碰撞忽略帶此狀態者；英雄移動本來就只受地形阻擋 |  |
| `pull_hook` | `supported` | G-4 | content effect {kind='forced_move_enemy', motion='pull', distance_key, duration_key} 放進 linear projectile（max_hits=1、wall='stop'）；Rust 端用 forced_motion::Spec payload 經 add_stat_buff | distance ≤3000、duration ≤5 秒；被位移期間暈眩 |
| `knockback` | `supported` | G-4 | 同 forced_move_enemy，motion='knockback'；單體技能或投射物皆可 | distance ≤3000、duration ≤5 秒，期間暈眩；AoE 擊退尚不支援（area 效果只展開傷害） |
| `true_sight` | `supported` | G-4 | content effect {kind='reveal_area', radius_key, duration_key}，point 目標；Rust 端用 reveal_zone::Spec payload 經 add_stat_buff | 顯形半徑 ≤2000、時間 ≤30 秒 |
| `zone_self_state` | `needs_framework` | G-4 | 無「在區域內才生效」的自身狀態 | 煙霧內閃避／魔免／不可攻擊、離開即解除 |
| `vision_debuff` | `needs_framework` | G-4 | visibility.rs 無「降低單位自身視野」入口 |  |
| `vision_radius_bonus` | `deferred` | backlog | BonusNightVision 等 StatKey 宣告但 native 無引用；遊戲沒有日夜循環 | 夜晚視野在本遊戲沒有對應語意 |
| `curving_hook` | `deferred` | backlog | 投射物無轉向控制 | 可轉彎投射物不在本月範圍 |
| `illusion` | `deferred` | backlog | unit_stats.rs 只有註解提到 illusion | README §5 分身列為僅需求選中才做；本月不新增大架構（schedule §4.4） |

## 各 G 包需要的能力（依名單）

| G 包 | capability | 狀態 | 需要的英雄 |
|---|---|---|---|
| G-2 | `recast_sub_ability` | `needs_framework` | A13（delivery） |
| G-4 | `enemy_cast_extends_debuff` | `needs_framework` | B09（backup） |
| G-4 | `mana_shield` | `needs_framework` | B09（backup） |
| G-4 | `vision_debuff` | `needs_framework` | A13（delivery） |
| G-4 | `zone_self_state` | `needs_framework` | A13（delivery） |
| backlog | `curving_hook` | `deferred` | A13（delivery） |
| backlog | `illusion` | `deferred` | A13（delivery） |

## B01 雜賀孫市（delivery，射手（遠程 700））— `supported`

本地 B01_saika_magoichi 是混合改編（見 candidates-draft.md）；本表以來源 current 版為準。T 只有 3 個 rank。

| 來源鍵 | 技能 | 本遊戲 action | 狀態 |
|---|---|---|---|
| W | `B01W` 狙擊模式 | Q | `supported` |
| E | `B01E` 雜賀眾 | W | `supported` |
| R | `B01R` 雨鐵炮 | E | `supported` |
| T | `B01T` 三段 | R | `supported` |

### `B01W` 狙擊模式 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 12 秒自身增益：攻擊力 +0/10/20/30、攻擊距離 +200/350/500/700 | `timed_stat_buff` | `supported` |  |
| 夜晚視野 +1000 | `vision_radius_bonus` | `supported` | （覆寫：本遊戲沒有日夜循環，夜晚視野加成不改變任何玩法；在 package assumptions 記錄為不適用） |

- KV／描述不一致：描述寫「攻擊距離提升至 900/1050/1200/1400」（AbilityCastRange），KV modifier 是 bonus_atk_rng 200/350/500/700；700 基礎射程＋bonus 與描述只在最後一級相符

- bot_usage（草案）：目標 `self`；時機：敵方英雄在 900～1400 內且自身未被近身時開啟；不要：被近身或血量低於 35% 時

### `B01E` 雜賀眾 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 每 15 秒儲存 1 位，最多 2/3/4/5 位，施放消耗 1 位 | `charges` | `supported` |  |
| 在指定點召喚 1 位雜賀眾，攻擊依英雄等級 +10 | `summon_timed` | `supported` |  |
| 召喚物無法移動（定點砲台） | `summon_immobile` | `supported` |  |

- 假設（assumed）：來源未給召喚物存活時間與數值（在缺失的 vscript B01E 中）；MVP 沿用本地 saika_gunner 單位數值並標 assumed

- bot_usage（草案）：目標 `point`；時機：有儲存次數且在推線或守塔時，放在自身與敵方之間的安全位置；不要：敵方塔射程內

### `B01R` 雨鐵炮 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 普攻命中時依目標類別（建築／英雄／部隊）加成物理傷害 | `target_class_multiplier` | `supported` |  |

### `B01T` 三段 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 7/11/13 秒基礎傷害 +200% | `timed_stat_buff` | `supported` |  |

- 假設（assumed）：「三段」只說三段技法；KV 只有 BASEDAMAGEOUTGOING_PERCENTAGE，沒有連發判定。不加多段射擊，只做增傷

- bot_usage（草案）：目標 `self`；時機：團戰或敵方英雄在攻擊範圍內、自身血量高於 40%

## B04 伊達政宗（delivery，近戰／位移）— `supported`

來源 B04 對應本地 date_masamune（資料夾 B02_date_masamune）；依名稱對照，不使用編號。

| 來源鍵 | 技能 | 本遊戲 action | 狀態 |
|---|---|---|---|
| W | `B04W` 踏火無痕 | Q | `supported` |
| E | `B04E` 火焰強襲 | W | `supported` |
| R | `B04R` 炎之迅捷 | E | `supported` |
| T | `B04T` 種子島 | R | `supported` |

### `B04W` 踏火無痕 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 衝刺到指定點，最遠 900～1200，速度 3000 | `dash_to_point` | `supported` |  |
| 衝刺期間隨機對 300 半徑內敵人放火焰（100～400 魔法傷害） | `random_target_det` | `supported` |  |
| 火焰以投射物命中（OnProjectileHitUnit） | `projectile_tracking` | `supported` |  |

- bot_usage（草案）：目標 `point`；時機：敵方英雄血量低於 50% 且在 900～1200 內，或作為脫離（朝友方塔）；不要：衝進敵方塔射程

### `B04E` 火焰強襲 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 指定點 500 半徑圓形魔法傷害 | `aoe_circle` | `supported` |  |
| 暈眩 0.5/0.8/1.1/1.4 秒 | `stun` | `supported` |  |

- bot_usage（草案）：目標 `point`；時機：500 半徑內至少 1 名敵方英雄或 3 個以上敵兵

### `B04R` 炎之迅捷 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 永久 +10/20/30/40 靈活 | `permanent_stat_bonus` | `supported` |  |

- 假設（assumed）：英雄模板雖有力量／敏捷／智力欄位，但 MOBA 戰鬥使用 authored 數值，舊屬性公式（comp/hero.rs）與 StatsAgilityBonus 在 MOBA 路徑沒有消費者；換算成實際生效的攻速／護甲，規則寫在 package

### `B04T` 種子島 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 28/32/36 秒攻擊距離 +672（128→800）與基礎攻擊 +50/70/90 | `timed_stat_buff` | `supported` |  |
| 持續期間普攻 38% 機率使目標停頓 0.2 秒 | `proc_chance` | `supported` |  |
| 停頓（短暫暈眩） | `stun` | `supported` |  |
| 近戰單位以 800 射程攻擊時需要彈道或改為遠程攻擊呈現 | `attack_range_switch` | `supported` |  |

- bot_usage（草案）：目標 `self`；時機：準備交戰且敵方英雄在 800 內

## A34 阿松（delivery，施法者／範圍）— `supported`

有 _20 變體（T 雪女的祝福、D 加賀百萬石）；本表只用 current，不混用。

| 來源鍵 | 技能 | 本遊戲 action | 狀態 |
|---|---|---|---|
| W | `A34W` 寒霜彈 | Q | `supported` |
| E | `A34E` 冰山之襲 | W | `supported` |
| R | `A34R` 冰霜之息 | E | `supported` |
| T | `A34T` 急速冷卻 | R | `supported` |

### `A34W` 寒霜彈 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 對單一敵人投出追蹤寒霜彈（KV MoveSpeed 2000） | `projectile_tracking` | `supported` |  |
| 目標可以是建築（AbilityUnitTargetType 含 BUILDING，描述「對建築物有效」） | `projectile_vs_building` | `supported` |  |
| 100～400 魔法傷害 | `unit_damage` | `supported` |  |
| 結凍 1.0～2.5 秒（STUNNED） | `stun` | `supported` |  |

- bot_usage（草案）：目標 `enemy_unit`；時機：可見的敵方英雄在 600 內，優先正在施法或追擊者

### `A34E` 冰山之襲 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 指定敵人主傷害 | `unit_damage` | `supported` |  |
| 目標周圍 335 半徑擴散傷害 | `aoe_circle` | `supported` |  |
| 移速 −30%、攻速 −20%，持續 5 秒 | `slow_debuff` | `supported` |  |

- KV／描述不一致：描述主傷害 200/300/400/500、擴散 100～400；KV AbilityDamage 100/200/300/400、aoe_damage 100

- bot_usage（草案）：目標 `enemy_unit`；時機：目標周圍 335 內有多名敵人，或清線

### `A34R` 冰霜之息 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 方向性冰霜：起點寬 150、終點寬 500、距離 1200、速度 1250 的線性投射物 | `projectile_linear` | `supported` |  |
| 命中後 5 秒每秒傷害（由投射物命中觸發） | `projectile_on_hit_dot` | `supported` |  |

- KV／描述不一致：描述每秒 30/40/50/60，KV debuff_damage 50/60/70/80

- bot_usage（草案）：目標 `direction`；時機：朝 1200 內敵方英雄或兵群方向

### `A34T` 急速冷卻 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 被動：施放基礎技能時，其他基礎技能冷卻 −2 秒 | `cdr_on_cast_hook` | `supported` |  |
| 被動：最大法力 +300/400/500 | `permanent_stat_bonus` | `supported` |  |
| 主動：22 秒內基礎技能冷卻減免 40/50/60% | `cooldown_reduction` | `supported` |  |

- bot_usage（草案）：目標 `self`；時機：W／E／R 中至少兩招在冷卻且即將交戰

## C17 阿市（delivery，支援／控制）— `supported`

| 來源鍵 | 技能 | 本遊戲 action | 狀態 |
|---|---|---|---|
| W | `C17W` 光鍊罰則 | Q | `supported` |
| E | `C17E` 祈勝光輝 | W | `supported` |
| R | `C17R` 神助一擊 | E | `supported` |
| T | `C17T` 極光守護 | R | `supported` |
| D | `C17D` 自我奉獻 | extra1 | `supported` |

### `C17W` 光鍊罰則 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 對敵我英雄皆可施放的 3 秒光鍊，斷開時依距離決定效果 | `tether_distance_scaled` | `supported` |  |
| 對敵：距離越遠傷害越高（最高 350～860） | `unit_damage` | `supported` |  |
| 對敵：距離越近定身越久（最高 1.5～4.5 秒） | `root` | `supported` |  |
| 對友：光鍊期間受到傷害 −30～60% | `timed_stat_buff` | `supported` |  |

- bot_usage（草案）：目標 `enemy_or_ally_hero`；時機：對敵：目標在 600 內且我方要追擊；對友：友方英雄正在承受集火

### `C17E` 祈勝光輝 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 全地圖單一友軍（含建築） | `global_ally_target` | `supported` |  |
| 16 秒護甲 +5、總攻擊 +7.5～30%、每秒回血 5～20 | `timed_stat_buff` | `supported` |  |

- bot_usage（草案）：目標 `ally_unit`；時機：友方英雄進入交戰或推塔時（全圖）

### `C17R` 神助一擊 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 指定點 200 半徑魔法傷害 200～500 | `aoe_circle` | `supported` |  |

- bot_usage（草案）：目標 `point`；時機：200 半徑內有敵方英雄或多名敵兵

### `C17D` 自我奉獻 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 額外槽 D：本遊戲 loadout 上限 4 個技能（含被動），這是第 5 個 | `extra_action_slot` | `supported` |  |
| 受到物理傷害 −20（固定格擋） | `flat_damage_block` | `supported` |  |
| 每次施法使自身與 750 內友軍回復 5% 生命與 4% 法力 | `on_cast_trigger` | `supported` |  |
| 範圍內友軍需要友軍查詢 | `ally_area_query` | `supported` |  |
| 百分比回復 | `heal` | `supported` |  |

- KV／描述不一致：描述範圍 750，KV AOERadius 800

### `C17T` 極光守護 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 施法前搖 1 秒後生效 | `delayed_effect` | `supported` | （覆寫：延遲 1 秒等於 AbilityCastPoint=1，以施法前搖 cast_time 表達） |
| 800 半徑圓形：敵人受 650～750 或最大生命 20～30%（取高） | `max_hp_percent_damage` | `supported` |  |
| 同範圍友軍回復最大生命 40～50% | `heal` | `supported` |  |
| 範圍內友軍需要友軍查詢 | `ally_area_query` | `supported` |  |
| 圓形範圍判定 | `aoe_circle` | `supported` |  |

- bot_usage（草案）：目標 `point`；時機：800 半徑內友方血量偏低且有敵方英雄；不要：延遲 1 秒內目標可能離開時不要只為了傷害施放

## A02 稻姬（delivery，切換／隱形（遠程 700））— `supported`

| 來源鍵 | 技能 | 本遊戲 action | 狀態 |
|---|---|---|---|
| W | `A02W` 蠍毒箭 | Q | `supported` |
| E | `A02E` 風之祈 | W | `supported` |
| R | `A02R` 遊風行身 | E | `supported` |
| T | `A02T` 融影術 | R | `supported` |

### `A02W` 蠍毒箭 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| toggle 法球：開啟期間每次普攻消耗 10 法力 | `toggle_mana_per_attack` | `supported` |  |
| 命中額外毒傷 30～90 | `on_attack_hook` | `supported` |  |
| 移速與攻速 −9～21%，持續 4 秒 | `slow_debuff` | `supported` |  |

- 假設（assumed）：法力不足時自動關閉 toggle（來源 vscript 缺失，依 Dota 法球慣例）

- bot_usage（草案）：目標 `self_toggle`；時機：交戰中、法力高於 25% 時開啟；沒有交戰時關閉；不要：法力不足時

### `A02E` 風之祈 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 自身立即回復 100～400 生命 | `heal` | `supported` |  |
| 10 秒攻速 +40～130% | `timed_stat_buff` | `supported` |  |

- bot_usage（草案）：目標 `self`；時機：血量低於 60% 或準備交戰

### `A02R` 遊風行身 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 被動：魔法抗性 +9～36%、總輸出傷害 +6～15% | `permanent_stat_bonus` | `supported` |  |

### `A02T` 融影術 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 20/22/24 秒隱形，攻擊不現形 | `stealth_attack_no_reveal` | `supported` |  |
| 基礎攻擊間隔 0.9、移速 +60 | `timed_stat_buff` | `supported` |  |
| 無單位碰撞 | `no_unit_collision` | `supported` |  |

- bot_usage（草案）：目標 `self`；時機：需要切入或撤退時

## A13 服部半藏（delivery，特殊（分身／鉤拉／隱形））— `deferred`

A13E 的 KV 是 ability_lua 型（頂層 ScriptFile、無 Function），行為幾乎全在缺失的 vscript 中。

需要的 G 包能力：G-2 `recast_sub_ability`（needs_framework）、G-4 `vision_debuff`（needs_framework）、G-4 `zone_self_state`（needs_framework）、backlog `curving_hook`（deferred）、backlog `illusion`（deferred）。

| 來源鍵 | 技能 | 本遊戲 action | 狀態 |
|---|---|---|---|
| W | `A13W` 忍法．影惑之術 | Q | `deferred` |
| E | `A13E` 忍法．勾鎖鐮 | W | `deferred` |
| R | `A13R` 伊賀忍術流 | E | `supported` |
| T | `A13T` 忍法密傳．隱斬 | R | `needs_framework` |
| D | `A13D` 忍法．迷蹤步 | extra1 | `supported` |

### `A13W` 忍法．影惑之術 — `deferred`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 產生 1～4 個短暫分身並向外彈出 900 距離 | `illusion` | `deferred` |  |
| 驅散自身所有法術效果 | `dispel_self` | `supported` |  |

- bot_usage（草案）：目標 `self`；時機：受到控制或集火時脫身

### `A13W2` 忍法．換位之術 — `needs_framework`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 第二個按鍵：與自己的分身換位 | `recast_sub_ability` | `needs_framework` |  |
| 換位位移 | `swap_position` | `supported` |  |

### `A13E` 忍法．勾鎖鐮 — `deferred`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 1200 距離、寬 150、速度 1000 的鉤鎖，可轉彎 | `curving_hook` | `deferred` |  |
| 命中第一個單位拉回身旁 | `pull_hook` | `supported` |  |
| 敵人額外 100～400 物理傷害 | `unit_damage` | `supported` |  |

- bot_usage（草案）：目標 `point`；時機：第一個命中會是敵方英雄的方向（無兵擋路）

### `A13R` 伊賀忍術流 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 11% 閃避 | `evasion` | `supported` |  |
| 每第三次攻擊引爆小範圍 75～150 魔法傷害 | `every_nth_attack` | `supported` |  |
| 目標受到傷害 +10%，最多疊 3 層 | `stacking_buff` | `supported` |  |

### `A13D` 忍法．迷蹤步 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 額外槽 D（不可學，HotKey D） | `extra_action_slot` | `supported` |  |
| 最多 20 秒隱形，移動加速；攻擊或施法現身 | `stealth` | `supported` |  |
| 現身第一擊額外 90～345 傷害 | `on_attack_hook` | `supported` |  |

- KV／描述不一致：MaxLevel 5 且 rank 0 為 0，與描述 4 段數值不同

- bot_usage（草案）：目標 `self`；時機：準備偷襲或脫離視野

### `A13T` 忍法密傳．隱斬 — `needs_framework`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 在目標區域放 6 秒煙霧；煙霧內自身 100% 閃避、魔免、不可攻擊、不可用物品 | `zone_self_state` | `needs_framework` |  |
| 每 0.3 秒對煙霧內敵人造成自身攻擊傷害並附加攻擊效果 | `ground_area_dot` | `supported` |  |
| 進入煙霧的敵人失去自身視野 | `vision_debuff` | `needs_framework` |  |
| 煙霧內隱形 | `stealth` | `supported` |  |

- bot_usage（草案）：目標 `point`；時機：300 內有多名敵人

## C15 玉子（blind，遠程射手（盲測））— `supported`

盲測：只做需求表，不寫專用 handler。

| 來源鍵 | 技能 | 本遊戲 action | 狀態 |
|---|---|---|---|
| W | `C15W` 亂射 | Q | `supported` |
| E | `C15E` 爆裂箭 | W | `supported` |
| R | `C15R` 精準之光 | E | `supported` |
| T | `C15T` 百箭齊發 | R | `supported` |

### `C15W` 亂射 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 指定點 275 半徑內每個目標各射一箭（追蹤） | `projectile_tracking` | `supported` |  |
| 範圍內目標選取 | `aoe_circle` | `supported` |  |
| 暈眩 0.6～2.4 秒 | `stun` | `supported` |  |

- KV／描述不一致：描述每箭 75～300 傷害，KV AbilitySpecial 沒有傷害欄

### `C15E` 爆裂箭 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 方向性箭矢（速度 1666、寬 170、距離 2000） | `projectile_linear` | `supported` |  |
| 沿路徑留下 12 秒火焰帶，碰到每秒傷害 | `ground_area_dot` | `supported` |  |
| 火焰帶是線形地面區域，不是圓形 | `ground_area_line` | `supported` |  |

- KV／描述不一致：描述每秒 100～160，KV dmg 25/30/35/40

### `C15R` 精準之光 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 攻擊 +10～34%、靈活 +5～20 | `permanent_stat_bonus` | `supported` |  |

### `C15T` 百箭齊發 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 被動：每第 8 次攻擊產生致命箭矢 | `every_nth_attack` | `supported` |  |
| 主動：暈眩目標 1～2 秒並放出 10 發箭矢持續 10 秒 | `stun` | `supported` |  |
| 箭矢是會持續攻擊的暫存單位／投射物 | `summon_timed` | `supported` |  |
| 目標死亡後箭矢改找附近敵人 | `retarget_on_death` | `supported` |  |

## B29 村上義清（blind，近戰／控制（盲測））— `supported`

盲測：只做需求表，不寫專用 handler。

| 來源鍵 | 技能 | 本遊戲 action | 狀態 |
|---|---|---|---|
| W | `B29W` 動地斧 | Q | `supported` |
| E | `B29E` 震地斧 | W | `supported` |
| R | `B29R` 士氣提升 | E | `supported` |
| T | `B29T` 大地震撼 | R | `supported` |

### `B29W` 動地斧 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 自身 500 半徑傷害 | `aoe_circle` | `supported` |  |
| 暈眩 1.0～1.9 秒 | `stun` | `supported` |  |

### `B29E` 震地斧 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 向指定方向跳躍最多 300 | `leap` | `supported` |  |
| 落地 500 半徑 200 傷害 | `aoe_circle` | `supported` |  |
| 移速 −18～30% | `slow_debuff` | `supported` |  |
| 8 秒光環：附近敵人護甲 −4～13 | `aura` | `supported` |  |

- KV／描述不一致：描述落地範圍 500，KV radius 800

### `B29R` 士氣提升 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 身旁友軍攻擊 +7～28% | `aura` | `supported` |  |
| 自身永久力量 +4～16 | `permanent_stat_bonus` | `supported` |  |

- KV／描述不一致：KV AOERadius 900；描述未給半徑

### `B29T` 大地震撼 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 引導：指定範圍內每 0.1 秒一波震波 | `channel` | `supported` |  |
| 每波 340 半徑傷害 | `aoe_circle` | `supported` |  |
| 每波暈眩 0.2～0.4 秒 | `stun` | `supported` |  |

- KV／描述不一致：描述每波 60 傷害、半徑 340、最多 50/60/70 波；KV B29T_Amount 50/60/70、B29T_Radius 700/800/900、持續 4.5/5.2/5.9 秒

## B08 淺井長政（backup，近戰（候補 H02／H06））— `supported`

| 來源鍵 | 技能 | 本遊戲 action | 狀態 |
|---|---|---|---|
| W | `B08W` 一刀．牙 | Q | `supported` |
| E | `B08E` 一刀．閃瞬 | W | `supported` |
| R | `B08R` 活心．魂 | E | `supported` |
| T | `B08T` 一刀．空牙 | R | `supported` |
| D | `B08D` 一刀．連牙斷空 | extra1 | `supported` |

### `B08W` 一刀．牙 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 方向性刀牙：寬 175、距離 1200、速度 1500 的線性投射物 | `projectile_linear` | `supported` |  |
| 路徑上敵人 200～500 傷害 | `unit_damage` | `supported` |  |

- KV／描述不一致：描述距離 1100，KV shock_distance 1200

- bot_usage（草案）：目標 `direction`；時機：朝 1200 內敵方英雄或兵群

### `B08E` 一刀．閃瞬 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 瞬移到指定目標身旁 | `blink_to_target` | `supported` |  |
| 100～400 魔法傷害 | `unit_damage` | `supported` |  |
| 暈眩 0.3 秒 | `stun` | `supported` |  |
| 目標損失 100～280 法力 | `mana_burn` | `supported` |  |

- bot_usage（草案）：目標 `enemy_unit`；時機：可見敵方英雄在 1100 內且血量低於 60%；不要：目標在敵方塔射程內

### `B08R` 活心．魂 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 永久攻速 +20～65% | `permanent_stat_bonus` | `supported` |  |
| 使用技能或物品後 8 秒 +15 攻擊、−4% 受到傷害 | `on_cast_trigger` | `supported` |  |
| 可疊 4～7 層 | `stacking_buff` | `supported` |  |

### `B08D` 一刀．連牙斷空 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 額外槽 D（16 級大招，不可學） | `extra_action_slot` | `supported` |  |
| 對指定敵人 18 刀，每刀 100 魔法傷害 | `unit_damage` | `supported` |  |
| 總共暈眩 2.2 秒 | `stun` | `supported` |  |

- bot_usage（草案）：目標 `enemy_unit`；時機：可見敵方英雄在 1100 內

### `B08T` 一刀．空牙 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 寬 500、距離 3000、速度 3000 的線性投射物 | `projectile_linear` | `supported` |  |
| 路徑上敵人 800～1600 傷害 | `unit_damage` | `supported` |  |

- bot_usage（草案）：目標 `direction`；時機：路徑上有低血量敵方英雄

## B28 松永久秀（backup，施法者（候補 H03））— `ambiguous`

| 來源鍵 | 技能 | 本遊戲 action | 狀態 |
|---|---|---|---|
| W | `B28W` 飛凰焰 | Q | `ambiguous` |
| E | `B28E` 焰光桎梏 | W | `ambiguous` |
| R | `B28R` 火流焰 | E | `ambiguous` |
| T | `B28T` 鬼謀．蛇火 | R | `ambiguous` |

### `B28W` 飛凰焰 — `ambiguous`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 直線鳳凰：寬 300、距離 1500、速度 1100 | `projectile_linear` | `supported` |  |
| 命中 175 傷害 | `unit_damage` | `supported` |  |
| 燃燒 5 秒每秒 50～155 | `dot_debuff` | `supported` |  |

- bot_usage（草案）：目標 `direction`；時機：朝 1500 內敵方英雄或兵群

### `B28E` 焰光桎梏 — `ambiguous`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 引導綁縛目標（無法動作） | `channel` | `supported` |  |
| 持續時間隨距離縮短而延長（1～5 秒） | `tether_distance_scaled` | `supported` |  |
| 目標無法動作 | `stun` | `supported` |  |
| 每秒 100 傷害 | `dot_debuff` | `supported` |  |

- bot_usage（草案）：目標 `enemy_unit`；時機：可見敵方英雄在 800 內且自身安全；不要：自身被集火時

### `B28R` 火流焰 — `ambiguous`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 10 秒移速 +0～24% | `timed_stat_buff` | `supported` |  |
| 每 0.3 秒隨機對 600 內敵人射流焰 | `random_target_det` | `supported` |  |
| 流焰是投射物 | `projectile_tracking` | `supported` |  |
| 燃燒 3 秒每秒 80 | `dot_debuff` | `supported` |  |

- bot_usage（草案）：目標 `self`；時機：600 內有多名敵人

### `B28T` 鬼謀．蛇火 — `ambiguous`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 800 範圍內敵人之間彈跳的火流，每跳 +50% 傷害 | `bounce_chain` | `supported` |  |
| 目標範圍選取 | `aoe_circle` | `supported` |  |

- bot_usage（草案）：目標 `point`；時機：800 內有多名敵方英雄

## B09 真田昌幸（backup，支援／切換（候補 H04／H05））— `needs_framework`

需要的 G 包能力：G-4 `enemy_cast_extends_debuff`（needs_framework）、G-4 `mana_shield`（needs_framework）。

| 來源鍵 | 技能 | 本遊戲 action | 狀態 |
|---|---|---|---|
| W | `B09W` 反噬裂隙 | Q | `needs_framework` |
| E | `B09E` 意念爆發 | W | `supported` |
| R | `B09R` 實念之盾 | E | `needs_framework` |
| T | `B09T` 心靈壓制 | R | `supported` |

### `B09W` 反噬裂隙 — `needs_framework`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 敵方英雄身上的可累加 DoT，每秒 7～16，持續 8 秒 | `dot_debuff` | `supported` |  |
| 可疊加，上限 20 層、最長 20 秒 | `stacking_buff` | `supported` |  |
| 目標每施放一次法術持續時間 +1 秒 | `enemy_cast_extends_debuff` | `needs_framework` |  |

- bot_usage（草案）：目標 `enemy_unit`；時機：敵方施法型英雄在 700 內

### `B09E` 意念爆發 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 引導最多 10 秒 | `channel` | `supported` |  |
| 每 0.5 秒在目標身上 400 半徑傷害：自身總法力 5% + 50～140 | `aoe_circle` | `supported` |  |

- bot_usage（草案）：目標 `enemy_unit`；時機：目標周圍 400 內有多名敵人且自身安全；不要：自身被集火時

### `B09R` 實念之盾 — `needs_framework`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| toggle 盾 | `toggle` | `supported` |  |
| 以法力吸收每次傷害的 30%，每 1 法力吸收 2～5 傷害 | `mana_shield` | `needs_framework` |  |

- bot_usage（草案）：目標 `self_toggle`；時機：受到傷害且法力高於 30% 時開啟；法力低時關閉

### `B09T` 心靈壓制 — `supported`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 全戰場任意點 | `global_point_target` | `supported` |  |
| 範圍友軍回復全部法力與 400～700 生命 | `heal` | `supported` |  |
| 1 秒後 700 範圍敵人 400～700 魔法傷害 | `delayed_effect` | `supported` |  |
| 無視魔免沉默 5～7 秒 | `silence` | `supported` |  |
| 敵人流失現有法力 60% | `mana_burn` | `supported` |  |
| 範圍判定 | `aoe_circle` | `supported` |  |

- bot_usage（草案）：目標 `point`；時機：團戰中友方缺血缺藍且敵方聚集

## A04 竹中重治（backup，召喚／施法者（候補 H05））— `ambiguous`

| 來源鍵 | 技能 | 本遊戲 action | 狀態 |
|---|---|---|---|
| W | `A04W` 式神．水 | Q | `ambiguous` |
| E | `A04E` 凍結術 | W | `ambiguous` |
| R | `A04R` 霜之庇護 | E | `ambiguous` |
| D | `A04D` 霜雀 | extra1 | `ambiguous` |
| T | `A04T` 吹雪 | R | `ambiguous` |

### `A04W` 式神．水 — `ambiguous`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 召喚 2 隻水式神，最久 26 秒，數值依等級 | `summon_timed` | `supported` |  |
| 召喚物朝目標點進攻 | `summon_attack_move` | `supported` |  |
| 召喚物攻擊時減速／凍結建築 | `on_attack_hook` | `supported` |  |

- bot_usage（草案）：目標 `point`；時機：推線或交戰時朝敵方

### `A04E` 凍結術 — `ambiguous`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 直線冰凍路徑（寬 150）瞬發 | `aoe_line_instant` | `supported` |  |
| 200～500 傷害 | `unit_damage` | `supported` |  |
| 凍結 1.1～2.3 秒 | `stun` | `supported` |  |

- bot_usage（草案）：目標 `direction`；時機：直線上有敵方英雄或多名敵兵

### `A04R` 霜之庇護 — `ambiguous`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 對受傷友軍英雄或自身：治療 150 | `heal` | `supported` |  |
| 可吸收 250～625 傷害的冰盾，10 秒 | `shield_absorb` | `supported` |  |
| 冰盾吸滿後周圍 435 敵人凍結 0.5～1.1 秒 | `aoe_circle` | `supported` |  |
| 凍結 | `stun` | `supported` |  |

- bot_usage（草案）：目標 `ally_unit`；時機：受傷的友方英雄或自身在 900 內

### `A04D` 霜雀 — `ambiguous`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 額外槽 D：本遊戲 loadout 上限 4 個技能（含被動），這是第 5 個 | `extra_action_slot` | `supported` |  |
| 普攻使敵人移速與攻速 −30%，1 秒 | `on_attack_hook` | `supported` |  |
| 減速 | `slow_debuff` | `supported` |  |

### `A04T` 吹雪 — `ambiguous`

| 機制 | capability | 狀態 | 說明 |
|---|---|---|---|
| 引導 8 秒的暴雪 | `channel` | `supported` |  |
| 835 範圍內隨機位置冰爆（半徑 300、每 0.24 秒） | `random_target_det` | `supported` |  |
| 冰爆範圍傷害 | `aoe_circle` | `supported` |  |
| 減速 −40% 移速／−30% 攻速 | `slow_debuff` | `supported` |  |

- KV／描述不一致：描述初始傷害 180/240/300、每秒 30/45/60；KV damage 30/45/60、damage_scepter 180/240/300

- bot_usage（草案）：目標 `point`；時機：835 內有多名敵人且自身安全；不要：自身被集火時

