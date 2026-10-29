-- 技能行為普查：B11 菊姬（current 版；_old 不列入）
-- 來源：C:\nobu\history_nobu（commit f966720），路徑相對於 game/dota_addons/nobu/scripts/
-- 英雄技能欄（npc/B_Unified/B11_2.txt:13-21）：W=B11W、E=B11E、R=B11R、magical_resistance、F=B11F、T=B11T、D=B11D、B11_precache（純資產）
-- 出生時自動學習：magical_resistance 與 B11D（library/events/eventforpichero.lua:53-61；heromap_autoskill B11 "16" = "D"，addon_game_mode.lua:393）
-- 值以 KV＋腳本為準；說明文字（resource/addon_tchinese.txt 第 2535-2546 行）只作名稱參考。
return {
  hero = "B11", name = "菊姬",
  abilities = {
    ---------------------------------------------------------------------------
    { id = "B11W", slot = "Q", name = "忍法．櫻雨",
      summary = "以自身為中心、半徑 1000（取技能施法距離）對所有可見、非魔免的敵方英雄與一般單位，先造成 200/250/300/350 魔法傷害，再造成 敏捷×1.0/1.2/1.4/1.6 純粹傷害。",
      ability = {
        kind = "active", target = "none",
        filter = { team = "enemy", types = { "hero", "creep" }, flags = { "visible_only" } },
        cast = { cast_point = 0.2, backswing = "engine_default", animation_rate = 2.0 },
        cost = { mana = { 100, 120, 140, 160 }, cooldown = 10, cooldown_start = "on_cast" },
        ranks = {
          range = 1000,                       -- AbilityCastRange，同時當作搜尋半徑
          magic_damage = { 200, 250, 300, 350 },
          agi_mul = { 1.0, 1.2, 1.4, 1.6 },
        },
        flags = {},                           -- SPELL_IMMUNITY_ENEMIES_NO：不打魔免
      },
      statuses = {},
      flow = [=[
on_cast:
  cue { id="b11w_spray", at=caster }            -- 原作連續建 5 個同樣的粒子
  for_each(selector{ origin=caster, shape=circle, radius=rank.range,
                     team=enemy, types={hero,creep}, flags={visible_only} }):
    damage { type=magical, amount=rank.magic_damage }
    if(unit_valid(event_unit)):                  -- 原作 IsValidEntity；剛死的單位仍 valid
      damage { type=pure, amount=caster.agi * rank.agi_mul }
    if(unit_valid(event_unit)):
      cue { id="b11w_hit", at=event_unit, facing=dir(event_unit -> caster) }
]=],
      units = {},
      gaps = {
        { concept = "value：技能本身的屬性（cast_range）",
          why = "搜尋半徑寫成 ability:GetCastRange()（vscripts/heroes/B_Unified/B11.lua:32），而不是獨立的 radius 特殊值；若有施法距離加成道具，原作半徑是否跟著變要確認。",
          proposal = "v0 §5 增加 `ability.cast_range`、`ability.level`、`ability.cast_point` 等技能自身屬性值；同時明確規定施法距離加成是否套用。" },
        { concept = "selector／condition：unit_valid",
          why = "第二段純粹傷害前檢查 IsValidEntity（B11.lua:50,62），被第一段打死的單位仍會被判為有效。",
          proposal = "condition 詞彙加入 `alive(unit)`，並在普查規則中寫明：原作 IsValidEntity 對剛死亡的單位為 true，移植時以 alive 取代、差異僅限無效傷害。" },
      },
      hooks = {},
      quirks = {
        "搜尋 flags 只有 FOW_VISIBLE，FindUnitsInRadius 預設排除魔免；與 SpellImmunityType=ENEMIES_NO 一致（npc/B_Unified/B11.txt:73-78）。",
        "KV 的 RunScript 參數名稱沿用 A11W_agiMul（npc/B_Unified/B11.txt:95,110），只是命名錯誤，不影響行為。",
        "for i=1,5 重複建同一個粒子 5 次（vscripts/heroes/B_Unified/B11.lua:23-26），僅呈現。",
        "魔法傷害會經過全域 DamageFilter：攻擊者有 spell_leech 時會吸血、magic_damage 會增幅（vscripts/library/events/eventfordamage.lua:30-40,66-70）。",
      },
      provenance = {
        "npc/B_Unified/B11.txt:66-114",
        "vscripts/heroes/B_Unified/B11.lua:19-70",
        "vscripts/library/events/eventfordamage.lua:30-70",
      },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "B11E", slot = "W", name = "忍法．調息",
      summary = "引導 10 秒：引導期間每秒回復 35/70/105/140 生命（以生命回復屬性實作）；另外立即獲得 +10/20/30/40 敏捷 10 秒，此加成不因引導中斷而消失。",
      ability = {
        kind = "channel", target = "none",
        cast = { cast_point = 0, channel_time = 10, interruptible = true, move_while_casting = false },
        cost = { mana = { 55, 65, 75, 85 }, cooldown = 16, cooldown_start = "on_cast" },
        ranks = {
          bonus_agi = { 10, 20, 30, 40 },
          hp_regen = { 35, 70, 105, 140 },
        },
      },
      statuses = {
        { id = "b11e_heal", duration = "infinite（由引導結束移除）", stacking = "refresh",
          dispel = "none", visibility = "buff",
          stats = { hp_regen_flat = "rank.hp_regen" },
          triggers = {
            on_created = "cue { id='b11e_shell', at=owner, follow=true }",
            on_removed = "if(caster.is_channeling(ability)): end_channel { interrupted=true }；cue_stop b11e_shell",
            ["periodic(1)"] = "（原作呼叫不存在的函式 modifier_B11E_OnIntervalThink，無效果）",
          } },
        { id = "b11e_agi", duration = 10, stacking = "refresh", dispel = "none", visibility = "buff",
          stats = { agi_bonus = "rank.bonus_agi" } },
      },
      flow = [=[
on_cast:
  apply_status { id=b11e_heal, target=caster }
  apply_status { id=b11e_agi,  target=caster, duration=10 }
  cue { id="snd_troll_priest_heal", at=caster }
on_channel_end {interrupted=any}:
  remove_status { by=id, id=b11e_heal, target=caster }
-- b11e_heal 若被外力移除（例如驅散），on_removed 會反過來中斷引導
]=],
      units = {},
      gaps = {
        { concept = "action：end_channel",
          why = "modifier_B11E_heal 被移除時若仍在引導，呼叫 ability:EndChannel(true)（B11.lua:95），狀態與引導雙向綁定。",
          proposal = "v0 §3「技能」類加 `end_channel {ability, interrupted}`；或在 Status 加 `bound_to_channel=ability`，由執行器自動雙向解除。" },
        { concept = "status：dispel 預設值",
          why = "datadriven modifier 未寫 IsPurgable，預設可否被驅散影響 b11e_heal 能否被打斷；需要一條全域規則。",
          proposal = "在 v0 §2 寫明 KV 未指定 IsPurgable 時的對應預設（建議 dispel=none），由 KV 編譯器套用。" },
      },
      hooks = {},
      quirks = {
        "OnIntervalThink 呼叫的 modifier_B11E_OnIntervalThink 在 B11.lua 不存在，每秒產生腳本錯誤但沒有玩法效果（npc/B_Unified/B11.txt:188-195；vscripts/heroes/B_Unified/B11.lua 全檔無此函式）。",
        "回血是 MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT（生命回復），不是 Heal 事件，所以不經過全域 HealFilter 的 decrease_health（vscripts/events.lua:279-378）。需確認 omoba 的回血屬性是否也不受治療減免。",
        "敏捷加成固定 10 秒，提前中斷引導仍保留（npc/B_Unified/B11.txt:217-226）。",
        "b11e_heal 沒有 Duration，完全依賴 OnChannelFinish 移除（B11.lua:72-77）；OnChannelFinish 在 Dota 中無論中斷或完成都會觸發。",
        "B11E_OnChannelFinish 使用全域變數 caster（B11.lua:73），未加 local，只是全域污染。",
      },
      provenance = {
        "npc/B_Unified/B11.txt:117-227",
        "vscripts/heroes/B_Unified/B11.lua:72-98",
      },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "B11R", slot = "E", name = "忍法．截殺術",
      summary = "被動：移動速度 +0/2/4/6%。普攻命中非建築目標時約 20.8% 機率（且每 7 次必中）使目標移速被設為 1 共 0.75 秒，並在目標背後產生一個持續 5 秒、可操控的菊姬分身（物理輸出 30%～60%、承受 +500/450/400/350% 傷害）；觸發後 0.3 秒內不再判定。",
      ability = {
        kind = "passive", target = "none",
        cost = {},
        ranks = {
          move_speed_pct = { 0, 2, 4, 6 },
          illusion_incoming_pct = { 500, 450, 400, 350 },
          root_duration = 0.75,
          proc_lockout = 0.3,                 -- 腳本常數 B11.lua:184
          pity_count = 7,                      -- 腳本常數 B11.lua:121
          proc_chance = "21/101",              -- RandomInt(0,100) <= 20
          illusion_duration = 5,               -- 腳本常數 B11.lua:177
          illusion_damage_rate = "hero_state.b11r_rate（未學 B11T 時 0.3；B11T 1/2/3 級為 0.4/0.5/0.6）",
        },
      },
      statuses = {
        { id = "b11r_passive", duration = "infinite", dispel = "none", visibility = "hidden",
          stats = { move_speed_pct = "rank.move_speed_pct" },
          triggers = { on_attack_landed = "見 flow：routine b11r_proc" } },
        { id = "nobu_rooted（共用）", duration = 0.75, stacking = "refresh", dispel = "none", visibility = "hidden",
          stats = { move_speed_absolute = 1 },
          states = {},
          note = "不是真正的 root：只把移速設成 1，仍可轉身、施法、攻擊、閃現；屬性 PERMANENT＋IGNORE_INVULNERABLE，死亡不移除；會受全域狀態抗性縮短（vscripts/events.lua:381-411）",
          cue = "viper_poison_debuff" },
        { id = "phased（引擎）", duration = 0.1, visibility = "hidden", states = { "no_unit_collision" } },
      },
      flow = [=[
hero_state: b11r_counter:int=0, b11r_locked:bool=false, b11r_rate:float=0.3

b11r_passive.on_attack_landed (event_unit = 被攻擊者):
  call routine b11r_proc { target=event_unit, values=this_ability }

routine b11r_proc { target, values }:          -- B11F 也會呼叫，values 換成 B11F 的 rank 表
  if(hero_state.b11r_locked): stop
  if(not is_building(target)): set_state(b11r_counter += 1)
  roll = random_int(0,100)
  if(hero_state.b11r_counter >= 7 or (roll <= 20 and not is_building(target))):
    set_state(b11r_counter = 0)
    apply_status { id=nobu_rooted, target=target, duration=values.root_duration, source_ability=none }
    create_illusion {
      of=caster, at=target.position - target.forward * 100, find_clear_space=true,
      control=owner, duration=5,
      outgoing_dmg=(hero_state.b11r_rate - 1) * 100,     -- 物理攻擊
      nonphysical_dmg_mul=hero_state.b11r_rate,          -- 全域 DamageFilter：illusion_damage
      incoming_dmg=values.illusion_incoming_pct,
      copy_level=true, copy_abilities={except={"B11R"}}, copy_items=true,
      set_health=caster.hp, copy_unit_state={"magical_resistance"},
      copy_statuses_if_present={ {id=b11t_enable, duration=status_default} },
      source_ability=values.ability }
    apply_status { id=phased, target=caster, duration=0.1 }
    apply_status { id=phased, target=target, duration=0.1 }
    set_state(b11r_locked = true)
    delay(0.3): set_state(b11r_locked = false)
]=],
      units = {
        { template = "B11 菊姬分身（create_illusion of caster）",
          lifetime = 5, control = "owner",
          notes = "以 CreateUnitByName 建出完整英雄後 MakeIllusion；逐級 HeroLevelUp 到施法者等級；移除預設技能後依施法者技能欄重學（排除 B11R，所以分身不會連鎖觸發）；以 CreateItem 複製 6 格道具；生命設為施法者目前生命；分身也帶 B11D、B11T 被動爆擊等。" },
      },
      gaps = {
        { concept = "create_illusion 的完整選項",
          why = "B11R 需要：生成位置相對於目標面向（背後 100）、複製等級／技能（含排除清單）／道具、設定生命、複製單位腳本變數（magical_resistance）、條件性帶入施法者身上的狀態、非物理傷害倍率（B11.lua:128-180；eventfordamage.lua:63-65）。v0 只有 incoming/outgoing/duration。",
          proposal = "create_illusion 加 `at`（point 表達式）、`copy {level, abilities{except}, items, statuses{ids}, unit_state{vars}}`、`set_health`、`damage_mul {physical, nonphysical}`、`control`、`source_ability`。" },
        { concept = "status stats：絕對值覆寫（move_speed_absolute）",
          why = "原作的「定身」nobu_modifier_rooted 其實是 MODIFIER_PROPERTY_MOVESPEED_ABSOLUTE=1（vscripts/nobu_modifiers/nobu_modifier_rooted.lua:16-25），不是 root state。",
          proposal = "v0 §2 stats 支援運算子：`add`、`pct`、`set_absolute`（含多個 absolute 來源的取值規則，例如取最小）。" },
        { concept = "chance 帶保底計數（pity）",
          why = "B11R 第 7 次必中、B11T 第 8 次必中，計數器與鎖定期間不計數的細節都在腳本裡（B11.lua:111-121,226-234）。可用 set_state＋if 拼出，但全作很常見。",
          proposal = "`chance {p, pseudo_random=false, pity={counter_var, threshold, count_if}}`，計數器存在 §6 unit state 以便 hash。" },
        { concept = "可共用的 action 例程（routine）",
          why = "B11F 直接呼叫 B11R_OnAttackLanded，並以 B11F 的 keys.ability 取數值（B11.lua:269-270）。",
          proposal = "v0 增加具名 `routine`（英雄或全域範圍），以 `call routine {args}` 呼叫，明確指定 rank 表取自呼叫端或定義端。" },
        { concept = "全域狀態抗性與豁免清單",
          why = "ModifierGainedFilter 以 target.states_resistance 縮短敵方施加在英雄身上的所有 modifier，ultimate_modifiers 清單除外（vscripts/events.lua:246-270,381-411）。nobu_rooted 會被縮短。",
          proposal = "v0 §2 加 `ignores_status_resistance` 旗標與全域規則章節；狀態抗性來源列為 StatKey。" },
        { concept = "condition 詞彙",
          why = "需要 is_building、is_hero、is_illusion、has_status、is_silenced、caster_alive 等判斷（B11.lua:4-5,118-121）。v0 §3 只寫 if(condition)，沒定義條件。",
          proposal = "定義 condition 子語言：單位類型、狀態、是否有 status、數值比較，以及 and/or/not。" },
      },
      hooks = {},
      quirks = {
        "機率是 RandomInt(0,100) <= 20，即 21/101 ≈ 20.8%，不是 20%（B11.lua:110,121）。",
        "條件 `caster.B11R >= 7 or ran <= 20 and not target:IsBuilding()` 中 and 優先；計數只在非建築時增加，所以實務上只有 B11F 把計數設為 10 時才可能對其他目標保底（B11.lua:118-121,269）。",
        "0.3 秒鎖定期間整段跳過，連計數都不增加（B11.lua:111,183-186）。",
        "OnAttackLanded 對任何目標都觸發，包括反補友軍時，也會定身友軍並在其背後放分身（Dota 引擎行為，KV 無 team 過濾，npc/B_Unified/B11.txt:274-287）。",
        "nobu_modifier_rooted 以 ability=nil 套用（B11.lua:123），所以傷害／來源歸屬不是 B11R。",
        "分身帶入 modifier_B11T_Enable 時 duration 用了未定義的全域變數 rate（=nil），結果使用 KV Duration %B11T_duration 全長，而不是施法者剩餘時間（B11.lua:163-164）。",
        "分身的 illusion_damage 只縮放非物理傷害（eventfordamage.lua:63-65），物理攻擊由 modifier_illusion outgoing_damage 縮放（B11.lua:177）。",
        "CreateUnitByName 會建出完整英雄並逐級 HeroLevelUp（B11.lua:131-140），可能觸發英雄出生與升級的全域事件；未逐一確認，列為 low confidence 子項。",
        "全域變數 B11R_trigger_counter、B11R_trigger_in_CD 宣告但未使用（B11.lua:101-102）。",
        "KV 中 B11R_old_attack、B11R_old_duration 是舊版殘留，未被 current 腳本使用（npc/B_Unified/B11.txt:252-262）。",
      },
      provenance = {
        "npc/B_Unified/B11.txt:229-300",
        "vscripts/heroes/B_Unified/B11.lua:101-190",
        "vscripts/nobu_modifiers/nobu_modifier_rooted.lua:1-43",
        "vscripts/library/events/eventfordamage.lua:63-65",
        "vscripts/events.lua:246-270,381-411",
      },
      confidence = "medium",
    },
    ---------------------------------------------------------------------------
    { id = "B11F", slot = "F", name = "鴉襲",
      summary = "B11T 啟動期間才可用的子技能（3/4/5 次）：躲開來襲投射物、瞬移到 800 內敵方目標身上，強制觸發一次截殺術（定身＋分身），並對目標下攻擊指令；本身沒有直接傷害。",
      ability = {
        kind = "active", target = "unit",
        filter = { team = "enemy", types = { "hero", "creep" }, flags = { "include_magic_immune" } },
        cast = { cast_point = 0.1, animation = "attack" },
        cost = { mana = 0, cooldown = 2, cooldown_start = "on_cast",
                 charges = "非真正 charges：以 hero_state.b11f_uses 計次，達上限即 set_ability_active(false)" },
        requires = { "由 B11T 設定等級並啟用；本身 NOT_LEARNABLE（npc/B_Unified/B11.txt:417）" },
        ranks = {
          range = 800,
          uses = { 3, 4, 5 },
          root_duration = 0.75,                       -- 給 routine b11r_proc 用
          illusion_incoming_pct = { 500, 450, 400 },  -- KV 寫 4 個值，但最高 3 級
          unused = { stun_time = 0.5, damage_mul = 1.25, crit_scale = 250, crit_pct = 15 },
        },
        flags = { "pierce_magic_immune" },
      },
      statuses = {
        { id = "b11f_phase_fx", duration = 0.5, visibility = "hidden", cue = "b08w2（attach_attack1）" },
      },
      flow = [=[
on_phase_start:
  apply_status { id=b11f_phase_fx, target=caster }
on_cast:
  cue { id="snd_b08e_vo1+storm_bolt+phoenix_bird_kill", at=caster }
  dodge_projectiles { unit=caster }                 -- ProjectileManager:ProjectileDodge
  apply_status { id=phased, target=caster, duration=0.1 }
  apply_status { id=phased, target=target, duration=0.1 }
  teleport { unit=caster, to=target.position, find_clear_space=false }
  set_state(b11r_counter = 10)                      -- 強制保底
  call routine b11r_proc { target=target, values=this_ability }   -- 數值取 B11F 的 rank 表
  issue_order { unit=caster, order=attack_target, target=target }
  set_state(b11f_uses += 1)
  if(hero_state.b11f_uses >= rank.uses):
    set_ability_active { ability=B11F, active=false }
    set_state(b11f_uses = 0)
]=],
      units = {},
      gaps = {
        { concept = "action：dodge_projectiles",
          why = "ProjectileManager:ProjectileDodge(caster) 讓已鎖定施法者的可閃避追蹤投射物失效（B11.lua:257）。v0 投射物有 dodgeable 欄位，但沒有觸發閃避的 action。",
          proposal = "v0 §3「投遞」加 `dodge_projectiles {unit}`，只影響 dodgeable=true 的追蹤投射物。" },
        { concept = "issue_order 的對象範圍",
          why = "對施法者本人下攻擊指令（B11.lua:271-275）；v0 寫 issue_order 只限召喚物或受控單位。",
          proposal = "issue_order 允許任何單位（含施法者與敵方，見 A11E），並定義指令類型列舉與是否可被玩家覆寫。" },
        { concept = "value：rank 表依呼叫端技能",
          why = "routine b11r_proc 用 keys.ability:GetSpecialValueFor，從 B11F 呼叫時取到 B11F 的 illusion_incoming_damage（B11.lua:108-109；npc/B_Unified/B11.txt:521-530）。",
          proposal = "routine 的 values 參數明確綁定；KV 編譯器檢查呼叫端技能是否具備所需特殊值。" },
      },
      hooks = {},
      quirks = {
        "若 0.3 秒鎖定中施放，b11r_proc 直接跳過（不定身、不出分身），但計數器仍保持 10，下一次可判定的普攻必定觸發（B11.lua:111,269）。",
        "SetAbsOrigin 直接疊到目標原點，不找空位，只靠 0.1 秒 phased 避免卡位；phased 結束後由引擎碰撞推開（B11.lua:264-266）。",
        "B11F_Stun_Time、B11F_Damage、B11F_buff_CritScale、B11F_buff_CritPercentage 在腳本中完全未使用（npc/B_Unified/B11.txt:497-520）。",
        "illusion_incoming_damage 給 4 個值但 MaxLevel=3（npc/B_Unified/B11.txt:426,527-530）。",
        "分身經 b11r_proc 建立時 modifier_illusion 的 ability 是 B11F（B11.lua:177）。",
        "分身也會複製 B11F（已設等級）；Dota 分身能否施放取決於引擎規則，未確認。",
      },
      provenance = {
        "npc/B_Unified/B11.txt:411-532",
        "vscripts/heroes/B_Unified/B11.lua:256-284",
        "vscripts/heroes/B_Unified/B11.lua:104-190",
      },
      confidence = "medium",
    },
    ---------------------------------------------------------------------------
    { id = "B11T", slot = "R", name = "極限體術．鴉襲",
      summary = "被動：每次攻擊開始 20% 機率（第 8 次必中）爆擊 150/175/200%；啟動（無目標、立即）後 13/20/27 秒內爆擊改為 200/250/300%，並解鎖 B11F 可用 3/4/5 次；學習等級同時把截殺術分身傷害倍率設為 0.4/0.5/0.6。",
      ability = {
        kind = "active", target = "none",
        cast = { cast_point = 0, immediate = true },
        cost = { mana = 150, cooldown = { 120, 110, 100 }, cooldown_start = "on_cast" },
        ranks = {
          duration = { 13, 20, 27 },
          passive_crit = { 150, 175, 200 },
          active_crit = { 200, 250, 300 },
          illusion_rate = { 0.4, 0.5, 0.6 },
          crit_chance = 20,       -- 腳本常數 B11.lua:230（RandomInt(1,100) <= 20，恰好 20%）
          crit_pity = 8,          -- count > 7
        },
      },
      statuses = {
        { id = "b11t_crit_passive", duration = "infinite", visibility = "hidden", dispel = "none",
          triggers = { on_attack_start = "見 flow" } },
        { id = "b11t_enable", duration = "rank.duration", visibility = "buff", dispel = "none" },
        { id = "b11t_crit2", duration = "1 / caster.attacks_per_second", visibility = "hidden",
          stats = { crit_pct_this_attack = "rank.passive_crit" } },
        { id = "b11t_crit3", duration = "1 / caster.attacks_per_second", visibility = "hidden",
          stats = { crit_pct_this_attack = "rank.active_crit" } },
      },
      flow = [=[
hero_state: b11t_count:int=0, b11f_uses:int=0, max_crit_record:float=0（計分用）

on_upgrade:
  set_state(b11r_rate = rank.illusion_rate)

on_cast:
  set_state(b11f_uses = 0)
  set_ability_level { ability=B11F, level=this.level }
  set_ability_active { ability=B11F, active=true }
  apply_status { id=b11t_enable, target=caster, duration=rank.duration }
  delay(rank.duration):
    set_ability_active { ability=B11F, active=false }

b11t_crit_passive.on_attack_start (event_unit = 攻擊目標):
  remove_status { by=id, id=b11t_crit2, target=caster }      -- 注意：不移除 crit3
  set_state(b11t_count += 1)
  if(random_int(1,100) <= 20 or hero_state.b11t_count > 7):
    set_state(b11t_count = 0)
    if(has_status(caster, b11t_enable)):
      set_state(max_crit_record = max(max_crit_record, caster.avg_attack_vs(event_unit) * rank.active_crit/100))
      apply_status { id=b11t_crit3, target=caster, duration=1/caster.attacks_per_second }
    else:
      set_state(max_crit_record = max(..., rank.passive_crit/100 ...))
      apply_status { id=b11t_crit2, target=caster, duration=1/caster.attacks_per_second }
    cue { id="anim_echo_slam", at=caster, rate=max(1, 1/caster.attacks_per_second) }
]=],
      units = {},
      gaps = {
        { concept = "爆擊／單次攻擊修飾",
          why = "MODIFIER_PROPERTY_PREATTACK_CRITICALSTRIKE 在 on_attack_start 中掛上、套用到本次攻擊（B11.lua:220-253；npc/B_Unified/B11.txt:363-382）。v0 沒有爆擊概念，也沒有「本次攻擊紀錄」。",
          proposal = "v0 加入 attack record：on_attack_start 可呼叫 `modify_attack {crit_mult, damage_bonus, cannot_miss}`，只作用於該次攻擊；多個爆擊來源取最大值寫成全域規則。" },
        { concept = "value：攻擊相關衍生值",
          why = "持續時間用 1/GetAttacksPerSecond，記錄用 GetAverageTrueAttackDamage(target)（B11.lua:235-243）。",
          proposal = "v0 §5 加 `attacks_per_second`、`attack_interval`、`avg_attack_damage(vs target)`。" },
        { concept = "技能限次使用窗口",
          why = "B11T 解鎖 B11F N 次並在時限後關閉（B11.lua:202-217,277-283）。可用 set_ability_active＋unit state＋delay 拼出，但「子技能限時、限次」在全作反覆出現。",
          proposal = "`grant_ability_window {ability, duration, uses, level_from}`，或讓 cost.charges 支援「由其他技能補滿、到時清空」。" },
        { concept = "可取消的 delay／排程",
          why = "SetContextThink 計時器沒有被取消的機制（B11.lua:211-216）；若以後有冷卻重置，第二次施放會被前一次的計時器提早關閉 B11F。",
          proposal = "delay 產生具名排程，可 `cancel_delay {name}`，或與 status 生命週期綁定（建議改用 b11t_enable.on_removed 關閉 B11F）。" },
      },
      hooks = {},
      quirks = {
        "on_attack_start 只移除 Crit2、不移除 Crit3（B11.lua:225），Crit3 若在下一次攻擊開始時尚未到期（持續時間=攻擊間隔），理論上會讓下一擊也爆擊。",
        "被動爆擊在啟動技能未學（等級 0）時不存在；學到 1 級才掛上 Passive modifier。",
        "計時器關閉 B11F 時寫入的是全域 B11F_counter 而非 caster.B11F_counter（B11.lua:214），由下次施放的 caster.B11F_counter=0 掩蓋（B11.lua:206）。",
        "OnUpgrade 才更新 caster.B11R_rate；未學 B11T 時 B11R 以 0.3 為預設（B11.lua:115-117,195-200）。",
        "maximum_critical_damage 只是計分板統計值（B11.lua:231-244），不影響玩法，但仍要進 unit state 才能 hash。",
        "爆擊在 OnAttackStart 中掛上即作用於當次攻擊，依 Dota datadriven 慣例（SpellLibrary Coup de Grace 同模式）推定；屬於引擎行為，待實測。",
        "分身也複製 B11T 被動爆擊與 b11t_enable（見 B11R），所以分身也會打出強化爆擊。",
      },
      provenance = {
        "npc/B_Unified/B11.txt:302-410",
        "vscripts/heroes/B_Unified/B11.lua:192-253",
      },
      confidence = "medium",
    },
    ---------------------------------------------------------------------------
    { id = "B11D", slot = "D", name = "忍法．繼血",
      summary = "天生被動（出生自動 1 級）：菊姬（或她的分身）擊殺英雄單位時，若菊姬本人存活且未被沉默，永久 +2 敏捷。",
      ability = {
        kind = "passive", target = "none",
        cost = {},
        requires = { "NOT_LEARNABLE，MaxLevel 1，出生時 SetLevel(1)" },
        ranks = { bonus_agi = 2 },
      },
      statuses = {
        { id = "b11d_passive", duration = "infinite", visibility = "buff", dispel = "none",
          triggers = { on_kill = "見 flow" } },
      },
      flow = [=[
b11d_passive.on_kill (owner = 擁有此狀態的單位，event_unit = 被殺者):
  if(alive(owner) and is_hero(event_unit) and not has_state(owner, silence)):
    if(is_illusion(owner)):
      modify_base_stat { unit=owner_of(owner), stat=agi, add=rank.bonus_agi }
    else:
      modify_base_stat { unit=owner, stat=agi, add=rank.bonus_agi }
]=],
      units = {},
      gaps = {
        { concept = "action：永久修改基礎屬性（modify_base_stat）",
          why = "ModifyAgility 永久加基礎敏捷（B11.lua:8,12）；v0 只能透過 status 加 stats。",
          proposal = "v0 §3「數值」加 `modify_base_stat {unit, stat, add}`，寫入英雄永久屬性並進 hash。" },
        { concept = "selector 起點：owner_of(unit)",
          why = "分身擊殺時把獎勵給 GetOwner()（B11.lua:5-8）。",
          proposal = "v0 §4 起點加 `owner`／`owner_of(x)`，以及 `is_illusion` 條件。" },
      },
      hooks = {},
      quirks = {
        "IsHero() 對英雄分身也回傳 true，所以擊殺敵方英雄分身同樣 +2 敏捷（B11.lua:4；Dota 引擎行為）。",
        "沉默時不觸發（B11.lua:4），說明文字也如此寫；這是腳本自訂條件，不是引擎的被動失效。",
        "檢查的是 keys.caster（分身時是分身本身）的存活與沉默，不是本體（B11.lua:4）。",
        "CalculateStatBonus 被註解（B11.lua:9,13），屬性衍生值要等引擎下一次自動重算。",
      },
      provenance = {
        "npc/B_Unified/B11.txt:26-65",
        "vscripts/heroes/B_Unified/B11.lua:3-16",
        "vscripts/library/events/eventforpichero.lua:53-61",
      },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "magical_resistance", slot = "hidden", name = "（共用）魔法抗性維持",
      summary = "全英雄共用的隱藏被動：每 0.2 秒把基礎魔抗設為 unit_state.magical_resistance（預設 30，上限 70）。應做成全域規則，不屬於 B11 專有。",
      ability = { kind = "passive", target = "none", cost = {}, ranks = { default = 30, cap = 70, interval = 0.2 } },
      statuses = {
        { id = "magical_resistance", duration = "infinite", visibility = "hidden",
          triggers = { ["periodic(0.2)"] = "set base_magic_resist = min(unit_state.magical_resistance or 30, 70)" } },
      },
      flow = [=[
periodic(0.2): set_stat_base { stat=magic_resist, value=min(unit_state.magical_resistance ?? 30, 70) }
]=],
      units = {},
      gaps = {
        { concept = "全域規則層",
          why = "魔抗由道具 OnEquip 改 unit 變數、再由此被動輪詢寫回（vscripts/library/common/dummy.lua:690-700；vscripts/items/Addon_Items/item_magical_resistance.lua:1-33）。",
          proposal = "改成 StatKey＋全域上限規則，移除輪詢；普查另立「全域規則」檔記錄。" },
      },
      hooks = {},
      quirks = { "B11R 分身會複製本體的 magical_resistance 變數（B11.lua:135）。" },
      provenance = {
        "npc/npc_abilities_custom_load.txt:3926-3952",
        "vscripts/library/common/dummy.lua:690-700",
      },
      confidence = "medium",
    },
  },
}
