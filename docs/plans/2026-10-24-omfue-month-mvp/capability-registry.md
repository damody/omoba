# 能力登錄（D3 registry 匯出）

由 `scripts/hero_source_index.lua capabilities` 從 `candidate-requirements.lua` 的 capabilities 產生；不含任何英雄的需求分析。

| 能力 id | 狀態 | G 包 | 依據 | 限制 |
|---|---|---|---|---|
| `ally_area_buff_query` | supported |  | omoba-core runtime/native/ally_area_buff.rs：施法者自身的即時封包，serial settlement 依提交位置對同隊非建築單位套上（刷新、不疊加）stat buff | 只能套 stat payload，最多 8 項 |
| `ally_area_query` | supported |  | omoba-core runtime/native/ally_area_restore.rs：施法者自身的即時封包，serial settlement 依提交位置選取同隊存活單位，以 received-healing 政策治療最大生命比例，並回復最大法力比例（mana_fraction）；committed vitals 一併揭露。使用者：oichi c17_aurora_guard（只回生命）、c17_devotion（英雄 UnitScript on_skill_cast，回復 5% 生命與 4% 法力） | 只限即時的一次性回復；持續型友軍光環用 ally_area_buff |
| `aoe_circle` | supported |  | EffectOp::AreaDamage；base_content/point_blast.rs；world.rs deal_damage_splash/query_enemies_in_range | 以自身為中心需 Rust handler |
| `aoe_line_instant` | supported |  | EffectOp::LineDamage（lumen_ray，capsule）；swept_dash.rs | 瞬發、不受地形阻擋、無飛行時間 |
| `attack_launch_hook` | supported |  | E170 AbilityScript::on_attack_launch：英雄普攻發射時呼叫已學技能（含延遲發射的投射物）；例：date b04_tanegashima 直線子彈 |  |
| `attack_range_switch` | supported |  | 英雄普攻一律走投射物（hero_tick.rs ProjectileLine2）；matchlock_gun 以 AttackRangeBonus 讓近戰英雄遠程攻擊 | 沒有真正的近戰／遠程型態切換或模型替換 |
| `aura` | supported |  | 英雄 UnitScript on_tick（content unit_script=true）每隔一段時間刷新：敵方光環用 query_enemies_in_range＋短 buff；友方光環用 ally_area_buff 封包（例：murakami_yoshikiyo b29_quake_axe／b29_morale_boost） |  |
| `blink_to_target` | supported |  | omoba-core blink_landing.rs：施法者自身的即時封包，serial settlement 以單點可走性（point_walkable，英雄移動同一套地形檢查加自身碰撞半徑）決定落點：偏好點不可走時，在目標周圍同距離以 15° 步進、左右交替找最近的可走點，並面向目標；例：asai_nagamasa b08_flash_cut | 只在目標周圍同距離的環上找；都不可走時留在原地 |
| `bounce_chain` | supported | G-1 | E163：ability_projectile 追蹤彈的 bounce 列（with_bounce／bounce_of）：每次命中後在命中目標 radius 內以決定性亂數挑另一個可見的敵方英雄或一般單位（只排除當下目標），同一顆投射物重新鎖定，直到總命中數；例：sanada_yukimura b06_flame_fangs | 只限追蹤彈；總命中 2～16、半徑 ≤2000；每跳傷害不變（沒有逐跳加成） |
| `cdr_on_cast_hook` | supported |  | content AbilityDefinition.cast_cooldown_refund → omoba-core runtime/native/cooldown_refund.rs，於 serial settlement 的 ScriptStartCooldown／ScriptConsumeCharge 結算；對手 replica 經 CommittedCooldown＋Hero::commit_cooldown（保留 admitted total） |  |
| `channel` | supported |  | G-2 pulse channel：content channel={anchor=self／point／unit, interval_key, pulses_key, radius_key}，host 擁有生命週期（runtime/native/ability_channel.rs）；每波套用 effects（damage／slow／stun）；示範 lumen_tremor、lumen_tether；另有 recovery_channel_self | 每波只能套用宣告式 effects；受控、移動、死亡、回城會中斷 |
| `charges` | supported |  | G-2（work-G2.md §2）：AbilityDefinition.charges={max_key,recharge_key}；Hero.ability_charges；cast_settlement 扣 1 次 | max 1..=10 且隨 rank 不遞減；recharge 0.25..600 秒；僅 active／ultimate |
| `cleave_on_attack` | supported |  | 英雄 UnitScript on_attack_hit（E163 起英雄普攻命中也會呼叫）＋get_final_atk＋query_enemies_in_range；例：a07_strongest |  |
| `command_restricted` | supported |  | E170 omoba-core command_restriction.rs：期間拒絕新的移動、停止、施法與道具指令；例：asai b08_fang_slash／b08_flash_cut 施放後 0.1 秒 | 召喚物指令與方向手勢不受限 |
| `cooldown_cap_buff` | supported |  | E170 omoba-core cooldown_cap.rs：buff 存在期間把指定技能的剩餘冷卻壓到全冷卻×(1−x)，含已在跑的冷卻；例：omatsu a34_rapid_cooling | 只作用於英雄技能，不含道具 |
| `cooldown_reduction` | supported |  | UnitStats cooldown_mult（CooldownPercentage[Stacking]）；dispatch.rs prepare_cast_cooldown | 只有百分比，無固定秒數減免或退還 |
| `damage_kind_immunity` | supported |  | E163：omoba-core damage_immunity.rs，handle_damage 依 absolute_no_damage_{physical,magical,pure} 把該種類傷害歸零；例：honda_tadakatsu a07_unscathed |  |
| `damage_reduction` | supported |  | StatKey::DamageTakenBonus，傷害封包結算時套用 |  |
| `damage_to_heal_conversion` | supported |  | E163：同一結算點依 absorb_{physical,magical,pure}_as_heal（0～1）把該種類傷害的一部分轉為回復 |  |
| `dash_to_point` | supported |  | EffectOp::DashToPoint；base_content swept_dash.rs（沿實際路徑傷害、受碰撞截斷）；連續位移：swept_dash::motion_packet → omoba-core forced_motion.rs Mode::Dash（施法者自身、host 依速度逐 tick 移動、撞牆停止、PreStepMovement 事實；不暈眩、移動中受 root），例：date_masamune b04_flame_trail_dash（dash_speed）、fire_dash（authored dash_duration）、murakami b29_quake_axe 跳躍（共用速度） | 路徑效果（傷害、鎖定、落地重擊）在施法時依核准路徑決定，不是在通過或落地當下；未宣告 dash_speed 時用 swept_dash::DASH_SPEED 2000 |
| `delayed_effect` | supported |  | E170 omoba-core runtime/native/delayed_effect.rs：技能在施法者身上掛排程 buff（≤10 秒），自然到期時依固定順序呼叫 AbilityScript::on_delayed_effect，只能套效果；例：murakami b29_quake_axe 0.2 秒猛擊、b29_earth_tremor 每擊 0.45 秒落下 | 施法者死亡時取消尚未結算者；不能扣費、施放或生成單位 |
| `discrete_dot` | supported |  | E170 ability_projectile dot_interval：命中附帶的 DoT 每 interval 秒結算一次固定量；例：omatsu a34_frost_breath 每秒一跳 | 每次命中一筆 |
| `dispel_self` | supported |  | EffectOp::CleanseAlly（可指定自身） |  |
| `dot_debuff` | supported |  | EffectOp::DamageOverTimeEnemy；periodic_damage.rs；buff_store drain_periodic_damage | 每次施放一筆 DoT |
| `evasion` | supported |  | UnitStats evasion_chance（unit_stats.rs）；world.rs get_evasion_chance |  |
| `every_nth_attack` | supported |  | on_attack_hit + buff payload 計數（buff_store.rs add 會替換 payload） | 無內建計數器 API |
| `extra_action_slot` | supported |  | ward-canonical 2026-10-25 23:19：英雄 abilities 第 5 項為 D 欄（runtime slot 4），moba_loadout.ranks 可有 5 項；ability auto_learn 依 required_hero_level 自動學會、不耗技能點 | 只有一個額外欄（extra1） |
| `flat_damage_block` | supported |  | StatKey::PhysicalConstantBlock 在 unit_stats.rs 結算 |  |
| `fow_viewer` | supported |  | E170 reveal_zone detect=false：在指定點為施法者隊伍開一般視野（不偵測隱形），對應原作 AddFOWViewer；例：tamako c15_burst_arrow、murakami b29_earth_tremor | 只能開給施法者自己的隊伍；每隊上限 64 |
| `global_ally_target` | supported |  | EffectOp::HealAlly/ShieldAlly 以單位為目標 | range 必填且上限 10000；需確認地圖尺寸內等同全圖 |
| `global_point_target` | supported |  | TargetType::Point + range 上限 10000 | 同上 |
| `ground_area_dot` | supported |  | native/ability_runtime/damage_zone.rs DamageZoneSpec（圓形、獨立或綁施法者）；point_blast::with_zone | 只有圓形傷害區域、上限 30 秒；不能減速或治療 |
| `ground_area_line` | supported | G-4 | DamageZoneSpec.end（center→end 膠囊）＋grow（隨投射物速度逐段鋪出，僅獨立區域）＋linger（每次 pulse 刷新同一個 DoT，離開後最多再燒 linger 秒）；zone_pulse_settlement 用 point_in_swept_circle | 長度 ≤10000；同一施法者所有 linger 區域共用一個灼燒 |
| `heal` | supported |  | world.rs heal; EffectOp::HealSelf/HealAlly |  |
| `hero_unit_hooks` | supported |  | 英雄 content unit_script=true → rust/hero_unit.rs 的 UnitScript：on_skill_cast（施法交易內、execute 前）、on_attack_hit（普攻命中）、on_damage_taken、on_tick（每 tick；自行以標記 buff 節流） | GameWorld 讀不到技能等級與 buff payload：以 rank 命名的標記 buff 表示學會的 rank |
| `item_use_cast_trigger` | supported |  | E170 ScriptEvent::ItemUsed 呼叫持有者 on_skill_cast("item:<id>")，base_content owner_cast 套用原作排除清單；例：asai b08_living_soul、oichi c17_devotion |  |
| `knockback` | supported | G-4 | 同 forced_move_enemy，motion='knockback'；單體技能或投射物皆可 | distance ≤3000、duration ≤5 秒，期間暈眩；AoE 擊退尚不支援（area 效果只展開傷害） |
| `leap` | supported |  | 同 dash_to_point（連續位移，forced_motion Mode::Dash；例：murakami b29_quake_axe） | 沒有拋物線／空中狀態；落地重擊在施法時依落點決定 |
| `mana_burn` | supported |  | EffectOp::DrainManaEnemy；world.rs spend_mana |  |
| `mana_burn_damage` | supported |  | E170 mana_drain damage 選項：燒魔後由 host 依實際扣除量造成指定類型傷害；例：asai b08_flash_cut |  |
| `mana_on_cast` | supported |  | E170 英雄 UnitScript on_skill_cast（含 E170 物品使用觸發）回復法力；例：omatsu a34_rapid_cooling 主動期間每次施放回魔 |  |
| `max_hp_percent_damage` | supported |  | world.rs get_max_hp + deal_damage（Rust handler） |  |
| `no_unit_collision` | supported |  | BuffStore NO_UNIT_COLLISION 狀態 buff：creep_tick 單位碰撞忽略帶此狀態者；英雄移動本來就只受地形阻擋 |  |
| `on_attack_hook` | supported |  | AbilityScript::on_attack_hit（ability.rs）由 dispatch.rs 對被動技能呼叫；rain_iron_cannon | 只對 passive 技能呼叫；在命中後執行（額外傷害為另一筆 damage） |
| `on_cast_trigger` | supported |  | 英雄 content unit_script=true → gen_hero_registry units 模式產生 UnitDef（heroes::<module>::hero_unit_script()），host bind_hero_unit_scripts 以 hero id 綁定；on_skill_cast 在施法交易內、execute 之前呼叫 |  |
| `on_hit_stacking_debuff` | supported |  | E170 omoba-core hit_stack.rs：投射物命中時依施法者身上 buff 的疊層規則（上限、每層 payload、持續、滿層是否刷新）對目標疊層；例：omatsu A34T 主動期間命中降魔抗 | 只在命中當下判定施法者 buff |
| `permanent_stat_bonus` | supported |  | AbilityScript::on_learn（ability.rs）+ add_stat_buff | 英雄模板雖有力量／敏捷／智力欄位，但 MOBA 戰鬥使用 authored 數值，舊屬性公式（comp/hero.rs）與 StatsAgilityBonus 在 MOBA 路徑沒有消費者；來源的屬性加成需在 package assumptions 換算成具體 StatKey |
| `proc_chance` | supported |  | world.rs rand_unit（每 tick 決定性 seed）；host attack_stun_chance（matchlock_gun） |  |
| `projectile_dodge` | supported |  | E170 ability_projectile dodgeable 旗標＋躲避封包（DODGE_BUFF）：追蹤施法者的可閃避技能投射物與普攻投射物作廢不結算，對應 ProjectileManager:ProjectileDodge；例：omatsu a34_frost_bolt、tamako c15_scatter_shot、date b04_flame_trail_dash、asai b08_flash_cut／b08_endless_fang | 只由腳本明確呼叫；不會因瞬移道具自動觸發 |
| `projectile_linear` | supported |  | G-1：ability_projectile.rs Linear，命中英雄與小兵、max_hits 1～64 穿透、牆 pass／stop；lumen_frost_lance | 飛行距離＝施法距離；寬度固定（不支援起訖寬度不同的錐形） |
| `projectile_on_hit_dot` | supported |  | ability_projectile::Spec.dot → generic_dot_buff_id＋PeriodicDamageSpec::payload（與宣告式 DoT 同一套堆疊規則）；ability_projectile_tests::on_hit_dot_applies_* |  |
| `projectile_tracking` | supported |  | G-1（work-G1.md）：omoba-core/src/runtime/native/ability_projectile.rs Homing；Lua projectile 投遞區塊或 Rust 以 ability_projectile::Spec 發射；lumen_frost_bolt | 只命中原目標同一世代；目標失效 expire／last_position 皆不造成傷害；每來源同時最多 32 顆 |
| `projectile_vs_building` | supported |  | ability_projectile::Spec.buildings（Homing 限定；hittable(..., buildings)）＋ability_projectile_tests::homing_reaches_enemy_buildings_only_when_the_spec_allows_it |  |
| `projectile_widening` | supported |  | E170 ability_projectile end_radius：直線投射物命中半徑隨已飛距離由起始半徑線性增到 end_radius；例：omatsu a34_frost_breath 150→500 | UE 呈現的投射物寬度仍是起始半徑 |
| `pull_hook` | supported | G-4 | content effect {kind='forced_move_enemy', motion='pull', distance_key, duration_key} 放進 linear projectile（max_hits=1、wall='stop'）；Rust 端用 forced_motion::Spec payload 經 add_stat_buff | distance ≤3000、duration ≤5 秒；被位移期間暈眩 |
| `random_target_det` | supported |  | world.rs rand_unit；查詢結果穩定排序 |  |
| `retarget_on_death` | supported | G-1 | ability_projectile TargetLostPolicy::Retarget(radius)（payload target_lost=retarget + retarget_radius）；追蹤彈目標失效時改鎖投射物 radius 內最近的存活敵方非建築單位（id 決勝） | 只限追蹤彈；以最近（決定性）取代來源的隨機；召喚物改鎖需由召喚物 UnitScript 自行處理 |
| `root` | supported |  | BuffId::Root；ControlEnemy；buff_store is_rooted |  |
| `self_slow_cleanse` | supported |  | E163：ally_cleanse::SELF_SLOW_BUFF 封包，只移除自己的移動減速；例：a07_red_thunder 1 秒後解除自身減速 |  |
| `shield_absorb` | supported |  | EffectOp::ShieldAlly → damage_shield buff; BuffStore::grant_damage_shield/absorb_damage_with_shield | 不疊加：大值與長時間取勝 |
| `silence` | supported |  | BuffId::Silence；dispatch.rs 施法前檢查 |  |
| `slow_debuff` | supported |  | EffectOp::SlowEnemy（負 move_speed_bonus）；負攻速 StatKey |  |
| `slow_resistance` | supported |  | E163：BuffStore::add 依目標的 slow_resistance 減弱技能減速的移速降低量 |  |
| `spell_amplify` | supported | G-4 | E188 omoba-core runtime/native/spell_amplify.rs：傷害封包帶 DamageProfile::SPELL（原作有 inflictor 的傷害：KV Damage、ApplyDamage{ability=}）時，依來源 BuffStore 的 spell_amplify_percentage（各來源相加）放大，在 handle_damage 結算、之後剝掉旗標；AMHC:Damage、普攻、PerformAttack 不帶旗標。道具被動（仕手能靴、各書）與英雄 buff 都用同一個 StatKey；英雄技能逐招依原作標記（A02W、A07E、A34W/E/R、B04W/E、B06E、B08W/T/D、B29W、C15W、C15T 5 點、C17W 定身傷害） | 攻擊者是 dummy 的原作傷害（C15E、大炎上之書的燃燒）不吃增幅；HUD 屬性面板尚未顯示法術增幅 |
| `spell_guard` | supported |  | E163：omoba-core spell_guard.rs，敵方指定技能施放成立時依目標的 spell_guard 項目套魔抗並開始冷卻；例：b06_mind_eye | 在施放成立時觸發（原作在下達指令時） |
| `stacking_buff` | supported |  | BuffStore::add 單 id 單實例、替換 payload；不同 id 加總 | 無 stack-count API；層數放在 payload，持續時間以最後一次刷新為準 |
| `status_resistance` | supported |  | E163：BuffStore::add 依目標的 status_resistance 縮短暈眩、定身、沉默、技能減速與技能 DoT 的持續時間；例：a07_strongest |  |
| `stealth` | supported |  | BuffStore::stealth_level（invisibility_level buff，整數層）與 StealthProfile 取高者，visibility read view 與目標可見判定共用；VisionSource.detection_level 決定偵測 |  |
| `stealth_attack_no_reveal` | supported |  | buff 隱形沒有破隱規則：攻擊、施法都不會解除，持續到 buff 到期 | 需要「攻擊破隱」的英雄要另加規則 |
| `stun` | supported |  | BuffId::Stun；EffectOp::StunEnemy/ControlEnemy；buff_store is_stunned | 來源的「結凍／停頓」以 stun 實作 |
| `summon_attack_move` | supported |  | native/summon_orders.rs；saika_gunner UnitScript AI | 玩家完整控制仍在 G-3 待完善 |
| `summon_flying_chase` | supported |  | E170 base_content summons/chaser.rs：召喚物直線飛行追擊鎖定目標（忽略地形與碰撞），否則取範圍內最近的已揭露敵人；例：tamako c15_hundred_arrows 箭矢 | 以中心距離判定普攻射程 |
| `summon_forced_target` | supported | G-3 | 召喚物身上的 buff_ids::SUMMON_TARGET_BUFF（payload {"summon_target":[id,gen]}，summon_target_payload 產生）讓該生命成為優先目標（host 仍檢查射程、視野、存活、可攻擊性）；目標死亡後回到模板優先序（最近） | 目標死亡後改鎖是最近而非隨機；召喚物要有能打到目標的射程或自行移動 |
| `summon_immobile` | supported |  | G-3（work-G2.md §3）：summons.lua immobile、damage_per_owner_level、owner_death；Lua summon={unit,duration_key} | 固定召喚物使用通用砲台 AI；L1 完成，實機待驗 |
| `summon_invulnerable` | supported | G-3 | summons.lua invulnerable = true：生成時登記到 MobaMatch.invulnerable_summons，current_match_target_attackable（索敵、投射物、區域、傷害結算、replica 揭露閘門共用）一律排除 | 只在 MOBA 對局生效；BuffId::Invulnerable 狀態 buff 仍沒有消費者 |
| `summon_owner_cap` | supported |  | E170 omoba-core summon_owner_cap.rs：每位主人同時存在上限（隨 rank），超出時移除最早生成者；例：saika b01_saika_corps | 移除不給賞金 |
| `summon_timed` | supported |  | world.rs spawn_summoned_unit；native/tick/summon_tick.rs 到期與跟隨；summons.lua 模板 | 需在 summons.lua 新增召喚物模板；沒有每位主人的數量上限 |
| `swap_position` | supported |  | world.rs set_pos 對兩個 handle |  |
| `target_class_multiplier` | supported |  | world.rs is_building、unit_id_of；on_attack_hit 內加額外傷害 | 英雄／部隊以 unit_id_of 判別；以額外傷害實作百分比加成 |
| `tether_distance_scaled` | supported |  | omoba-core runtime/native/tether.rs：目標身上的 timed buff（__tether_v1）自然到期時由 buff_tick 依施法者距離結算傷害（遠→高）與定身（近→長）；提早移除或任一端死亡則不結算 |  |
| `timed_stat_buff` | supported |  | world.rs add_stat_buff + StatKey（攻擊、攻速、射程、移速、護甲、減傷）；sniper_mode/matchlock_gun/three_stage | 對友軍或複合增益要寫 Rust handler |
| `toggle` | supported |  | AbilityType::Toggle；No1_sniper_mode.rs has_buff/remove_buff 模式 |  |
| `toggle_mana_per_attack` | supported |  | omoba-core runtime/native/attack_orb.rs（G-2）：toggle 標記 buff、每次普攻扣魔、沒藍自動關閉、命中加傷與減速；content AttackOrb；示範 lumen_venom_orb |  |
| `true_sight` | supported | G-4 | content effect {kind='reveal_area', radius_key, duration_key}，point 目標；Rust 端用 reveal_zone::Spec payload 經 add_stat_buff | 顯形半徑 ≤2000、時間 ≤30 秒 |
| `unit_damage` | supported |  | world.rs deal_damage; EffectOp::Damage (generic_effects.rs); lumen_bolt | 瞬發，含施法距離檢查 |
| `enemy_cast_extends_debuff` | needs_framework | G-4 | 需要目標方的 on_skill_cast | 需要英雄 UnitScript 掛點與跨單位事件 |
| `mana_shield` | needs_framework | G-4 | UnitScript::on_damage_taken 可改傷害，但英雄無 UnitScript | 需要英雄 UnitScript 掛點 |
| `recast_sub_ability` | needs_framework | G-2 | 無 sub-ability／recast 視窗 | 第二個按鍵與窗口都不存在 |
| `vision_debuff` | needs_framework | G-4 | visibility.rs 無「降低單位自身視野」入口 |  |
| `zone_self_state` | needs_framework | G-4 | 無「在區域內才生效」的自身狀態 | 煙霧內閃避／魔免／不可攻擊、離開即解除 |
| `curving_hook` | deferred | backlog | 投射物無轉向控制 | 可轉彎投射物不在本月範圍 |
| `illusion` | deferred | backlog | unit_stats.rs 只有註解提到 illusion | README §5 分身列為僅需求選中才做；本月不新增大架構（schedule §4.4） |
| `vision_radius_bonus` | deferred | backlog | BonusNightVision 等 StatKey 宣告但 native 無引用；遊戲沒有日夜循環 | 夜晚視野在本遊戲沒有對應語意 |
