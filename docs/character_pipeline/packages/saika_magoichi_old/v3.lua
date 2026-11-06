-- B01_old 雜賀孫市（舊版，11 版技能組）— package schema v3。
-- 版本：原作 heromap_version B01 ["11"] = true，heromap_skill B01 ["11"] = "WERT"（vscripts/addon_game_mode.lua:178,280），
-- heromap_autoskill B01 ["11"] = ""（vscripts/addon_game_mode.lua:383）。
-- 槽位依據：vscripts/items/Addon_Items/tofar_goback.lua:105-133 先移除全部技能，再依 "WERT" 順序
-- AddAbility B01W_old、B01E_old、B01R_old、attribute_bonusx、B01T_old，所以 W/E/R/T → Q/W/E/R；沒有 D（同本體）。
-- 舊版的狙擊模式是切換技（減移速、加射程與夜視），雜賀眾一次在身邊召喚數位、沒有充能與上限，雨鐵炮是機率觸發。
-- attribute_bonusx、B01_precache、halloween_attack、buff_* 屬規則層，都不列入。
-- 原作沒有舊版招式的繁中說明（resource 沒有 B01*_old），招式名取 KV 註解（同本體），說明依 KV 與腳本還原，見 assumptions.md。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"（同本體）。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "B01_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B01", text = { zh_TW = { name = "雜賀孫市（舊版）" }, en = { name = "Saika Magoichi (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b01_old_sniper_mode", source_id = "B01W_old", slot = "Q", kind = "toggle", max_level = 4, tags = { "basic", "toggle" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 50, unit = "mana", source = "kv:AbilityManaCost" },
          range_bonus = { 250, 450, 650, 900, unit = "dist", source = "kv:bonus_atk_rng" },
          night_vision = { 250, 450, 650, 900, unit = "dist", source = "kv:bonus_sight_rng" },
          move_penalty = { -70, unit = "pct", source = "kv:modifier_B01W MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE" },
        },
        text = {
          zh_TW = { name = "狙擊模式",
            body = "切換：開啟後進入狙擊狀態，攻擊距離 +{range_bonus}、夜晚視野 +{night_vision}，移動速度降低 {move_penalty:abs}，直到關閉為止。" },
          en = { name = "Sniper Mode",
            body = "Toggle: while on, Magoichi takes a sniper stance with +{range_bonus} attack range and +{night_vision} night vision, but {move_penalty:abs} less movement speed, until switched off." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b01w_old_sniper", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "狙擊模式", body = "攻擊距離 +{range_bonus}、夜晚視野 +{night_vision}，移動速度降低 {move_penalty:abs}。持續到關閉為止。" },
              en = { name = "Sniper Mode", body = "+{range_bonus} attack range and +{night_vision} night vision, {move_penalty:abs} less movement speed. Lasts until switched off." },
            },
            reveal = "public", cue = "b01w_sniper_stance",
            stats = { attack_range = "$range_bonus", vision_night = "$night_vision", move_speed_pct = "$move_penalty" },
            notes = "原作 modifier_B01W（B01W_old 自己的 Modifiers 區塊）：ATTACK_RANGE_BONUS %bonus_atk_rng、BONUS_NIGHT_VISION %bonus_sight_rng、MOVESPEED_BONUS_PERCENTAGE −70、動作替換 alternate；OnToggleOn 掛上、OnToggleOff 移除。沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_toggle_on = {
            { op = "apply_status", status = "b01w_old_sniper", to = "caster" },
            { op = "cue", id = "b01w_load_sound", at = "caster" },
          },
          on_toggle_off = {
            { op = "remove_status", from = "caster", status = "b01w_old_sniper" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B01.txt:184-206",
            text = "KV 有 bonus_atk 0/10/20/30，但 modifier_B01W 沒有任何屬性引用它（舊版不加攻擊力），不列入。" },
          { policy = "note", ref = "npc/B_Unified/B01.txt:105",
            text = "KV AbilityCastRange 950/1150/1350/1600 是給說明顯示的最終射程（基礎射程加 bonus_atk_rng），不影響施放，不列入。" },
          { policy = "no_effect", ref = "vscripts/heroes/B_Unified/B01.lua:16-17",
            text = "B01W 腳本讀不存在的 keys.modifier_one 再 ApplyDataDrivenModifier，沒有效果（推定之後的粒子也不會建立）；增益由 KV 的 ApplyModifier 掛上。" },
          { policy = "as_source", ref = "npc/B_Unified/B01.txt:163-181",
            text = "開啟時另掛 modifier_B01W_locker（Duration %lock 2/3/4/5 秒），期間 SetActivated(false) 無法關閉。開啟即消耗冷卻 10 秒，冷卻中本來就無法再切換，鎖定時間都短於冷卻、沒有額外效果，不列入（產生器也不支援切換技的 requires）。冷卻在開啟時起算是推定。" },
          { policy = "note", ref = "npc/B_Unified/B01.txt:176-180",
            text = "死亡時 datadriven modifier 隨死亡移除，但切換鈕仍亮著（推定，同 A03E 的情況），復活後要重新開關才有效果。這裡未另寫死亡處理。" },
        },
        provenance = { "npc/B_Unified/B01.txt:97-211", "vscripts/heroes/B_Unified/B01.lua:3-39", "vscripts/heroes/B_Unified/B01.lua:416-422" },
      },
      -------------------------------------------------------------------------
      { id = "b01_old_saika_corps", source_id = "B01E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic", "summon" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          count = { 2, 3, 4, 5, unit = "count", source = "kv:count" },
          lifetime = { 10000, unit = "s", hidden = true, source = "kv:SpawnUnit Duration" },
          attack_base = { 70, unit = "dmg",
            source = "script:vscripts/heroes/B_Unified/B01.lua:156-157 SetBaseDamageMin/Max(70 + caster:GetLevel() * 15)",
            note = "原作腳本報錯沒有覆寫（雜賀眾保留 KV 67～73），D22 依腳本原意修正" },
          attack_per_level = { 15, unit = "dmg", source = "script:vscripts/heroes/B_Unified/B01.lua:156-157" },
        },
        text = {
          zh_TW = { name = "雜賀眾",
            body = "在身邊召喚 {count} 位{@b01_old_saika_gunner}，幾乎永久存在，數量沒有上限。雜賀眾原地駐守，攻擊射程內的敵人，攻擊力為 {attack_base} 加孫市每等級 {attack_per_level}（召喚當下決定）。" },
          en = { name = "Saika Corps",
            body = "Summons {count} {@b01_old_saika_gunner} around Magoichi. They last almost indefinitely and there is no limit on how many can exist. They hold their position and shoot enemies in range, with {attack_base} attack damage plus {attack_per_level} per Magoichi level, fixed when summoned." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "b01e_summon", at = "caster" },
            { op = "create_unit", template = "b01_old_saika_gunner", at = "caster", count = "$count",
              lifetime = "$lifetime", control = "ai", as = "gunner",
              overrides = {
                attack_damage = "$attack_base + $attack_per_level * caster.level",
                attack_damage_max = "$attack_base + $attack_per_level * caster.level",
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B01.lua:151-158",
            text = "FC-179 D22 舊版，比照 D13 修：依腳本原意，雜賀眾攻擊力 = 70 + 15 × 孫市等級（最小最大相同），生成當下取樣一次。原作：OnSpawn 的 B01E_old_CHECK 第一行 table.insert(caster.B01E, target)，但 caster.B01E 從未初始化（唯一的初始化在 B01.lua:91-92 已註解），執行期報錯中斷：設定攻擊力與粒子都不會執行，雜賀眾保留 KV 攻擊力 67～73，不隨等級成長。" },
          { policy = "no_effect", ref = "vscripts/heroes/B_Unified/B01.lua:41-60",
            text = "OnSpellStart 先跑本體的 B01E（充能），舊版沒有 cooldown 特殊值、FindAbilityByName(\"B01E\") 是 nil：modifier_charges 綁不到技能，OnAbilityExecuted 永遠不扣層，不影響施放，視為沒有效果。" },
          { policy = "as_source", ref = "npc/B_Unified/B01.txt:420-440",
            text = "SpawnUnit Duration 10000 秒、沒有 UnitLimit，也沒有清除舊的雜賀眾：每次施放都疊加，數量沒有上限。生成時 modifier_phased 0.03 秒屬避免卡位的呈現，不列入。" },
          { policy = "port_adaptation", ref = "npc/B_Unified/B01.txt:425",
            text = "SpawnUnit 生成的單位由施法者控制、GrantsGold 5；玩家操控召喚物尚未完成（G-3），比照本體以 ai = guard 的定點砲台代替，賞金同本體為零。" },
        },
        provenance = { "npc/B_Unified/B01.txt:382-455", "vscripts/heroes/B_Unified/B01.lua:41-60", "vscripts/heroes/B_Unified/B01.lua:151-163" },
      },
      -------------------------------------------------------------------------
      { id = "b01_old_rain_iron_cannon", source_id = "B01R_old", slot = "E", kind = "passive", max_level = 4,
        tags = { "basic", "attack_modifier" },
        values = {
          hero_pct = { 40, 50, 60, 70, unit = "pct", source = "kv:atk_hero" },
          building_pct = { 20, 30, 40, 55, unit = "pct", source = "kv:atk_building" },
          creep_pct = { 100, 140, 180, 220, unit = "pct", source = "kv:atk_unit" },
          chance = { "4600/101", unit = "pct", source = "script:vscripts/heroes/B_Unified/B01.lua:371-376 RandomInt(0,100) <= 45" },
          pity_after = { 2, unit = "count", source = "script:vscripts/heroes/B_Unified/B01.lua:372-375 失敗計數 > 2 時必中" },
        },
        text = {
          zh_TW = { name = "雨鐵炮",
            body = "被動：普攻命中時有 {chance} 機率追加攻擊力一定比例的物理傷害：英雄 {hero_pct}、建築 {building_pct}、其他單位 {creep_pct}。連續 {pity_after} 次沒有觸發，下一次必定觸發。" },
          en = { name = "Rain of Iron",
            body = "Passive: attacks that land have a {chance} chance to deal extra physical damage equal to a share of Magoichi's attack damage: {hero_pct} against heroes, {building_pct} against buildings and {creep_pct} against other units. After {pity_after} attacks in a row without a trigger, the next one always triggers." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = { filter = "alive(event.target)", actions = {
              { op = "chance", p = "$chance", pity_after = "$pity_after", counter = "ability", actions = {
                { op = "damage", to = "event.target", type = "physical",
                  amount = "self.attack * select(event.target.is_hero, $hero_pct, select(event.target.is_building, $building_pct, $creep_pct))" },
                { op = "cue", id = "b01r_hit", at = "event.target" },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B01.lua:368-377",
            text = "RandomInt(0,100) > 45 時失敗計數加一；計數 > 2 或骰到 ≤ 45 時觸發並歸零：每次機率 46/101，連續兩次失敗後第三次必中。計數存在 caster.B01R_old，每位孫市一份。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B01.lua:368",
            text = "只有 caster.nobuorb1 == nil 才判定；舊版孫市沒有會寫入 nobuorb1 的技能，永遠成立，不列入。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B01.lua:380-389",
            text = "類別判斷順序 IsHero → IsBuilding → 其他；傷害走 AMHC:Damage 物理（再經護甲）。舊版沒有本體的 EXCLUDE_TARGET_NAME 檢查，也不分敵我（反補友軍時也會追加）。" },
          { policy = "note", ref = "npc/B_Unified/B01.txt:281",
            text = "原作以 %attack_damage（本次攻擊的傷害）計；同本體以命中當下的攻擊力（self.attack）表達，是否含暴擊未確認。" },
        },
        provenance = { "npc/B_Unified/B01.txt:266-307", "vscripts/heroes/B_Unified/B01.lua:363-408" },
      },
      -------------------------------------------------------------------------
      { id = "b01_old_three_stage", source_id = "B01T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 135, 123, 111, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 300, 350, 400, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades" },
          duration = { 7, 11, 13, unit = "s", source = "kv:duration" },
          damage_pct = { 200, unit = "pct", source = "kv:per_att" },
        },
        text = {
          zh_TW = { name = "三段",
            body = "施展三段技法 {duration} 秒：基礎攻擊傷害提高 {damage_pct}。每次普攻仍只判定一次傷害。" },
          en = { name = "Three Stages",
            body = "Uses the three-stage technique for {duration} seconds: base attack damage is increased by {damage_pct}. Each attack still deals damage only once." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point", immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "b01t_old_stage", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "三段", body = "基礎攻擊傷害提高 {damage_pct}。每次普攻仍只判定一次傷害。" },
              en = { name = "Three Stages", body = "Base attack damage increased by {damage_pct}. Each attack still deals damage only once." },
            },
            reveal = "public", cue = "b01t_flame_aura",
            stats = { base_attack_damage_pct = "$damage_pct" },
            notes = "原作 modifier_B26T（B01T_old 自己的 Modifiers 區塊，名稱與 B26 相同）：BASEDAMAGEOUTGOING_PERCENTAGE %per_att、Duration %duration；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b01t_old_stage", to = "caster" },
            { op = "cue", id = "b01t_title", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B01.txt:542-618",
            text = "和本體相同只有增傷，前搖 0.35 秒（本體 0.1）。modifier 名稱 modifier_B26T 與 B26 的同名，若同場有 B26 的同名 modifier 會互相覆蓋；本遊戲狀態 id 各自獨立，不重現。" },
        },
        provenance = { "npc/B_Unified/B01.txt:542-618" },
      },
    },
    unit_templates = {
      { id = "b01_old_saika_gunner", source_id = "B01W_UNIT",
        values = {
          hp = { 500, unit = "hp", source = "kv:StatusHealth@npc/npc_units_custom_load.txt:4764" },
          armor = { 1, unit = "armor", source = "kv:ArmorPhysical@npc/npc_units_custom_load.txt:4737" },
          magic_resist = { 0, unit = "pct", source = "kv:MagicalResistance@npc/npc_units_custom_load.txt:4738" },
          attack_range = { 725, unit = "dist", source = "kv:AttackRange@npc/npc_units_custom_load.txt:4748" },
          attack_interval = { 1.4, unit = "s", source = "kv:AttackRate@npc/npc_units_custom_load.txt:4745" },
          attack_damage = { 67, unit = "dmg", source = "kv:AttackDamageMin@npc/npc_units_custom_load.txt:4742",
            note = "只是基底，召喚時依孫市等級覆寫（雜賀眾的 attack_base / attack_per_level）" },
          attack_damage_max = { 73, unit = "dmg", source = "kv:AttackDamageMax@npc/npc_units_custom_load.txt:4743" },
          projectile_speed = { 900, unit = "speed", kv = { 3000 },
            source = "design:host 定點砲台 TAttack 固定 900（同本體）",
            source_differs = "原作 ProjectileSpeed 3000（npc_units_custom_load.txt:4751）；比照本體" },
          move_speed = { 0, unit = "move_speed", kv = { 280 },
            source = "design:G-3 玩家操控召喚物未完成，以定點代替（同本體）",
            source_differs = "原作 MovementSpeed 280（npc_units_custom_load.txt:4760）且可操控" },
          vision_day = { 1400, unit = "dist", source = "kv:VisionDaytimeRange@npc/npc_units_custom_load.txt:4770" },
          vision_night = { 1100, unit = "dist", source = "kv:VisionNighttimeRange@npc/npc_units_custom_load.txt:4771" },
          acquisition_range = { 725, unit = "dist", kv = { 800 },
            source = "design:host 定點砲台的索敵範圍等於射程（同本體）",
            source_differs = "原作 AttackAcquisitionRange 800（npc_units_custom_load.txt:4747）" },
          collision = { 0, unit = "dist", source = "script:npc/npc_units_custom_load.txt:4777 for_no_collision" },
          bounty_gold = { 0, unit = "gold", kv = { 20 },
            source = "design:正式 MOBA 擊殺結算沒有召喚物賞金分支（同本體）",
            source_differs = "原作 BountyGoldMin/Max 20（npc_units_custom_load.txt:4754-4755）" },
          bounty_xp = { 0, unit = "xp", kv = { 5 },
            source = "design:同上（同本體）",
            source_differs = "原作 BountyXP 5（npc_units_custom_load.txt:4756）" },
        },
        text = { zh_TW = { name = "雜賀眾" }, en = { name = "Saika Gunner" } },
        stats = { hp = "$hp", armor = "$armor", magic_resist = "$magic_resist", move_speed = "$move_speed" },
        attack = { kind = "ranged", range = "$attack_range", interval = "$attack_interval", damage_type = "physical",
          damage = "$attack_damage", damage_max = "$attack_damage_max", projectile_speed = "$projectile_speed" },
        bounty = { gold = "$bounty_gold", xp = "$bounty_xp" },
        vision = { day = "$vision_day", night = "$vision_night" },
        collision_radius = "$collision",
        acquisition_range = "$acquisition_range",
        owner_death = "persist",
        ai = "guard",
        tags = { "summon" },
        notes = "攻擊力依腳本原意為 70 + 15 × 主人等級，生成當下取樣一次（原作 B01E_old_CHECK 報錯沒有執行，D22 修正）。主人死亡時保留（SpawnUnit 不隨主人死亡移除）。",
        quirks = {
          { policy = "port_adaptation", ref = "vscripts/library/common/dummy.lua:722-736",
            text = "原作 passive_b01w_unit（npc_abilities_custom_load.txt:5216-5245）：每 0.1 秒檢查一千範圍內有沒有友方孫市（不限主人），沒有時套 0.15 秒輸出 −90%；同本體，需要只用揭露資料判定距離，暫未實作。" },
        },
        provenance = { "npc/npc_units_custom_load.txt:4720-4798", "npc/npc_abilities_custom_load.txt:5216-5245" } },
    },
  }
end
