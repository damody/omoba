# 能力登錄（D3 registry 匯出）

由 `scripts/hero_source_index.lua capabilities` 從 `candidate-requirements.lua` 的 capabilities 產生；不含任何英雄的需求分析。

| 能力 id | 狀態 | G 包 | 依據 | 限制 |
|---|---|---|---|---|
| `ally_area_buff_query` | supported |  | omoba-core runtime/native/ally_area_buff.rs：施法者自身的即時封包，serial settlement 依提交位置對同隊非建築單位套上（刷新、不疊加）stat buff | 只能套 stat payload，最多 8 項 |
| `ally_area_query` | supported |  | omoba-core runtime/native/ally_area_restore.rs：施法者自身的即時封包，serial settlement 依提交位置選取同隊存活單位並以 received-healing 政策治療最大生命比例；committed vitals 一併揭露 | 目前只有生命；法力回復需另加 committed mana |
| `aoe_circle` | supported |  | EffectOp::AreaDamage；base_content/point_blast.rs；world.rs deal_damage_splash/query_enemies_in_range | 以自身為中心需 Rust handler |
| `aoe_line_instant` | supported |  | EffectOp::LineDamage（lumen_ray，capsule）；swept_dash.rs | 瞬發、不受地形阻擋、無飛行時間 |
| `attack_range_switch` | supported |  | 英雄普攻一律走投射物（hero_tick.rs ProjectileLine2）；matchlock_gun 以 AttackRangeBonus 讓近戰英雄遠程攻擊 | 沒有真正的近戰／遠程型態切換或模型替換 |
| `aura` | supported |  | 英雄 UnitScript on_tick（content unit_script=true）每隔一段時間刷新：敵方光環用 query_enemies_in_range＋短 buff；友方光環用 ally_area_buff 封包（例：murakami_yoshikiyo b29_quake_axe／b29_morale_boost） |  |
| `blink_to_target` | supported |  | world.rs set_pos + get_pos（Rust handler）；summoner Flash 前例 | 需自寫落點合法性檢查 |
| `cdr_on_cast_hook` | supported |  | content AbilityDefinition.cast_cooldown_refund → omoba-core runtime/native/cooldown_refund.rs，於 serial settlement 的 ScriptStartCooldown／ScriptConsumeCharge 結算；對手 replica 經 CommittedCooldown＋Hero::commit_cooldown（保留 admitted total） |  |
| `channel` | supported |  | G-2 pulse channel：content channel={anchor=self／point／unit, interval_key, pulses_key, radius_key}，host 擁有生命週期（runtime/native/ability_channel.rs）；每波套用 effects（damage／slow／stun）；示範 lumen_tremor、lumen_tether；另有 recovery_channel_self | 每波只能套用宣告式 effects；受控、移動、死亡、回城會中斷 |
| `charges` | supported |  | G-2（work-G2.md §2）：AbilityDefinition.charges={max_key,recharge_key}；Hero.ability_charges；cast_settlement 扣 1 次 | max 1..=10 且隨 rank 不遞減；recharge 0.25..600 秒；僅 active／ultimate |
| `cooldown_reduction` | supported |  | UnitStats cooldown_mult（CooldownPercentage[Stacking]）；dispatch.rs prepare_cast_cooldown | 只有百分比，無固定秒數減免或退還 |
| `dash_to_point` | supported |  | EffectOp::DashToPoint；base_content swept_dash.rs（沿實際路徑傷害、受碰撞截斷） | 瞬間位移，沒有隨時間移動 |
| `dispel_self` | supported |  | EffectOp::CleanseAlly（可指定自身） |  |
| `dot_debuff` | supported |  | EffectOp::DamageOverTimeEnemy；periodic_damage.rs；buff_store drain_periodic_damage | 每次施放一筆 DoT |
| `evasion` | supported |  | UnitStats evasion_chance（unit_stats.rs）；world.rs get_evasion_chance |  |
| `every_nth_attack` | supported |  | on_attack_hit + buff payload 計數（buff_store.rs add 會替換 payload） | 無內建計數器 API |
| `extra_action_slot` | supported |  | ward-canonical 2026-10-25 23:19：英雄 abilities 第 5 項為 D 欄（runtime slot 4），moba_loadout.ranks 可有 5 項；ability auto_learn 依 required_hero_level 自動學會、不耗技能點 | 只有一個額外欄（extra1） |
| `flat_damage_block` | supported |  | StatKey::PhysicalConstantBlock 在 unit_stats.rs 結算 |  |
| `global_ally_target` | supported |  | EffectOp::HealAlly/ShieldAlly 以單位為目標 | range 必填且上限 10000；需確認地圖尺寸內等同全圖 |
| `global_point_target` | supported |  | TargetType::Point + range 上限 10000 | 同上 |
| `ground_area_dot` | supported |  | native/ability_runtime/damage_zone.rs DamageZoneSpec（圓形、獨立或綁施法者）；point_blast::with_zone | 只有圓形傷害區域、上限 30 秒；不能減速或治療 |
| `ground_area_line` | supported | G-4 | DamageZoneSpec.end（center→end 膠囊）＋grow（隨投射物速度逐段鋪出，僅獨立區域）＋linger（每次 pulse 刷新同一個 DoT，離開後最多再燒 linger 秒）；zone_pulse_settlement 用 point_in_swept_circle | 長度 ≤10000；同一施法者所有 linger 區域共用一個灼燒 |
| `heal` | supported |  | world.rs heal; EffectOp::HealSelf/HealAlly |  |
| `hero_unit_hooks` | supported |  | 英雄 content unit_script=true → rust/hero_unit.rs 的 UnitScript：on_skill_cast（施法交易內、execute 前）、on_attack_hit（普攻命中）、on_damage_taken、on_tick（每 tick；自行以標記 buff 節流） | GameWorld 讀不到技能等級與 buff payload：以 rank 命名的標記 buff 表示學會的 rank |
| `knockback` | supported | G-4 | 同 forced_move_enemy，motion='knockback'；單體技能或投射物皆可 | distance ≤3000、duration ≤5 秒，期間暈眩；AoE 擊退尚不支援（area 效果只展開傷害） |
| `leap` | supported |  | 同 dash_to_point（瞬間位移） | 沒有拋物線／空中狀態 |
| `mana_burn` | supported |  | EffectOp::DrainManaEnemy；world.rs spend_mana |  |
| `max_hp_percent_damage` | supported |  | world.rs get_max_hp + deal_damage（Rust handler） |  |
| `no_unit_collision` | supported |  | BuffStore NO_UNIT_COLLISION 狀態 buff：creep_tick 單位碰撞忽略帶此狀態者；英雄移動本來就只受地形阻擋 |  |
| `on_attack_hook` | supported |  | AbilityScript::on_attack_hit（ability.rs）由 dispatch.rs 對被動技能呼叫；rain_iron_cannon | 只對 passive 技能呼叫；在命中後執行（額外傷害為另一筆 damage） |
| `on_cast_trigger` | supported |  | 英雄 content unit_script=true → gen_hero_registry units 模式產生 UnitDef（heroes::<module>::hero_unit_script()），host bind_hero_unit_scripts 以 hero id 綁定；on_skill_cast 在施法交易內、execute 之前呼叫 |  |
| `permanent_stat_bonus` | supported |  | AbilityScript::on_learn（ability.rs）+ add_stat_buff | 英雄模板雖有力量／敏捷／智力欄位，但 MOBA 戰鬥使用 authored 數值，舊屬性公式（comp/hero.rs）與 StatsAgilityBonus 在 MOBA 路徑沒有消費者；來源的屬性加成需在 package assumptions 換算成具體 StatKey |
| `proc_chance` | supported |  | world.rs rand_unit（每 tick 決定性 seed）；host attack_stun_chance（matchlock_gun） |  |
| `projectile_linear` | supported |  | G-1：ability_projectile.rs Linear，命中英雄與小兵、max_hits 1～64 穿透、牆 pass／stop；lumen_frost_lance | 飛行距離＝施法距離；寬度固定（不支援起訖寬度不同的錐形） |
| `projectile_on_hit_dot` | supported |  | ability_projectile::Spec.dot → generic_dot_buff_id＋PeriodicDamageSpec::payload（與宣告式 DoT 同一套堆疊規則）；ability_projectile_tests::on_hit_dot_applies_* |  |
| `projectile_tracking` | supported |  | G-1（work-G1.md）：omoba-core/src/runtime/native/ability_projectile.rs Homing；Lua projectile 投遞區塊或 Rust 以 ability_projectile::Spec 發射；lumen_frost_bolt | 只命中原目標同一世代；目標失效 expire／last_position 皆不造成傷害；每來源同時最多 32 顆 |
| `projectile_vs_building` | supported |  | ability_projectile::Spec.buildings（Homing 限定；hittable(..., buildings)）＋ability_projectile_tests::homing_reaches_enemy_buildings_only_when_the_spec_allows_it |  |
| `pull_hook` | supported | G-4 | content effect {kind='forced_move_enemy', motion='pull', distance_key, duration_key} 放進 linear projectile（max_hits=1、wall='stop'）；Rust 端用 forced_motion::Spec payload 經 add_stat_buff | distance ≤3000、duration ≤5 秒；被位移期間暈眩 |
| `random_target_det` | supported |  | world.rs rand_unit；查詢結果穩定排序 |  |
| `retarget_on_death` | supported | G-1 | ability_projectile TargetLostPolicy::Retarget(radius)（payload target_lost=retarget + retarget_radius）；追蹤彈目標失效時改鎖投射物 radius 內最近的存活敵方非建築單位（id 決勝） | 只限追蹤彈；以最近（決定性）取代來源的隨機；召喚物改鎖需由召喚物 UnitScript 自行處理 |
| `root` | supported |  | BuffId::Root；ControlEnemy；buff_store is_rooted |  |
| `shield_absorb` | supported |  | EffectOp::ShieldAlly → damage_shield buff; BuffStore::grant_damage_shield/absorb_damage_with_shield | 不疊加：大值與長時間取勝 |
| `silence` | supported |  | BuffId::Silence；dispatch.rs 施法前檢查 |  |
| `slow_debuff` | supported |  | EffectOp::SlowEnemy（負 move_speed_bonus）；負攻速 StatKey |  |
| `stacking_buff` | supported |  | BuffStore::add 單 id 單實例、替換 payload；不同 id 加總 | 無 stack-count API；層數放在 payload，持續時間以最後一次刷新為準 |
| `stealth` | supported |  | BuffStore::stealth_level（invisibility_level buff，整數層）與 StealthProfile 取高者，visibility read view 與目標可見判定共用；VisionSource.detection_level 決定偵測 |  |
| `stealth_attack_no_reveal` | supported |  | buff 隱形沒有破隱規則：攻擊、施法都不會解除，持續到 buff 到期 | 需要「攻擊破隱」的英雄要另加規則 |
| `stun` | supported |  | BuffId::Stun；EffectOp::StunEnemy/ControlEnemy；buff_store is_stunned | 來源的「結凍／停頓」以 stun 實作 |
| `summon_attack_move` | supported |  | native/summon_orders.rs；saika_gunner UnitScript AI | 玩家完整控制仍在 G-3 待完善 |
| `summon_forced_target` | supported | G-3 | 召喚物身上的 buff_ids::SUMMON_TARGET_BUFF（payload {"summon_target":[id,gen]}，summon_target_payload 產生）讓該生命成為優先目標（host 仍檢查射程、視野、存活、可攻擊性）；目標死亡後回到模板優先序（最近） | 目標死亡後改鎖是最近而非隨機；召喚物要有能打到目標的射程或自行移動 |
| `summon_immobile` | supported |  | G-3（work-G2.md §3）：summons.lua immobile、damage_per_owner_level、owner_death；Lua summon={unit,duration_key} | 固定召喚物使用通用砲台 AI；L1 完成，實機待驗 |
| `summon_invulnerable` | supported | G-3 | summons.lua invulnerable = true：生成時登記到 MobaMatch.invulnerable_summons，current_match_target_attackable（索敵、投射物、區域、傷害結算、replica 揭露閘門共用）一律排除 | 只在 MOBA 對局生效；BuffId::Invulnerable 狀態 buff 仍沒有消費者 |
| `summon_timed` | supported |  | world.rs spawn_summoned_unit；native/tick/summon_tick.rs 到期與跟隨；summons.lua 模板 | 需在 summons.lua 新增召喚物模板；沒有每位主人的數量上限 |
| `swap_position` | supported |  | world.rs set_pos 對兩個 handle |  |
| `target_class_multiplier` | supported |  | world.rs is_building、unit_id_of；on_attack_hit 內加額外傷害 | 英雄／部隊以 unit_id_of 判別；以額外傷害實作百分比加成 |
| `tether_distance_scaled` | supported |  | omoba-core runtime/native/tether.rs：目標身上的 timed buff（__tether_v1）自然到期時由 buff_tick 依施法者距離結算傷害（遠→高）與定身（近→長）；提早移除或任一端死亡則不結算 |  |
| `timed_stat_buff` | supported |  | world.rs add_stat_buff + StatKey（攻擊、攻速、射程、移速、護甲、減傷）；sniper_mode/matchlock_gun/three_stage | 對友軍或複合增益要寫 Rust handler |
| `toggle` | supported |  | AbilityType::Toggle；No1_sniper_mode.rs has_buff/remove_buff 模式 |  |
| `toggle_mana_per_attack` | supported |  | omoba-core runtime/native/attack_orb.rs（G-2）：toggle 標記 buff、每次普攻扣魔、沒藍自動關閉、命中加傷與減速；content AttackOrb；示範 lumen_venom_orb |  |
| `true_sight` | supported | G-4 | content effect {kind='reveal_area', radius_key, duration_key}，point 目標；Rust 端用 reveal_zone::Spec payload 經 add_stat_buff | 顯形半徑 ≤2000、時間 ≤30 秒 |
| `unit_damage` | supported |  | world.rs deal_damage; EffectOp::Damage (generic_effects.rs); lumen_bolt | 瞬發，含施法距離檢查 |
| `bounce_chain` | needs_framework | G-1 | on_projectile_hit + ProjectileHitContext.generation（只在塔的 UnitScript 用過） | 英雄技能沒有彈跳入口 |
| `delayed_effect` | needs_framework | G-2 | 只有施法前搖 cast_time（native/cast_windup.rs） | 施放後延遲 N 秒才結算的階段不存在；若延遲等於前搖可用 cast_time 表達 |
| `enemy_cast_extends_debuff` | needs_framework | G-4 | 需要目標方的 on_skill_cast | 需要英雄 UnitScript 掛點與跨單位事件 |
| `mana_shield` | needs_framework | G-4 | UnitScript::on_damage_taken 可改傷害，但英雄無 UnitScript | 需要英雄 UnitScript 掛點 |
| `recast_sub_ability` | needs_framework | G-2 | 無 sub-ability／recast 視窗 | 第二個按鍵與窗口都不存在 |
| `vision_debuff` | needs_framework | G-4 | visibility.rs 無「降低單位自身視野」入口 |  |
| `zone_self_state` | needs_framework | G-4 | 無「在區域內才生效」的自身狀態 | 煙霧內閃避／魔免／不可攻擊、離開即解除 |
| `curving_hook` | deferred | backlog | 投射物無轉向控制 | 可轉彎投射物不在本月範圍 |
| `illusion` | deferred | backlog | unit_stats.rs 只有註解提到 illusion | README §5 分身列為僅需求選中才做；本月不新增大架構（schedule §4.4） |
| `vision_radius_bonus` | deferred | backlog | BonusNightVision 等 StatKey 宣告但 native 無引用；遊戲沒有日夜循環 | 夜晚視野在本遊戲沒有對應語意 |
