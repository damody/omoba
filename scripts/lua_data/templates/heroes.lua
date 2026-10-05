return function(ctx)
  local heroes = {
    {
      id = "saika_magoichi",
      rust_module = "B01_saika_magoichi",
      display_name = "雜賀孫市",
      title = "千里狙擊手",
      portrait = "data/hero_portraits/hero_saika_magoichi_portrait.png",
      background = "雜賀眾的領袖，以精準的遠程射擊聞名於戰國時代",
      abilities = {
        "sniper_mode",
        "saika_reinforcements",
        "rain_iron_cannon",
        "three_stage_technique",
      },
      strength = 18,
      agility = 28,
      intelligence = 16,
      primary_attribute = "agility",
      attack_range = 900.0,
      base_damage = 52,
      base_armor = 1.5,
      base_hp = 580,
      base_mana = 300,
      move_speed = 320.0,
      turn_speed = 720.0,
      render = {
        render_mode = "model_3d",
        model = "templates/heroes/saika_magoichi/saika_magoichi.fbx",
        texture = "templates/heroes/saika_magoichi/saika_magoichi_mat.png",
        scale = 0.012,
        pitch_offset_deg = -90.0,
        roll_offset_deg = 0.0,
        yaw_offset_deg = -90.0,
        z_offset = 0.0,
        muzzle_bone = "Weapon Ref",
        animation_sources = {
          idle = {
            model = "templates/heroes/saika_magoichi/b01_ani_stand.fbx",
            animation = "Take 001",
            duration_ticks = 80.0,
            ticks_per_second = 30.0,
            timeline_offset_ticks = 66.0,
          },
          idle_2 = {
            model = "templates/heroes/saika_magoichi/b01_ani_stand2.fbx",
            animation = "Take 001",
            duration_ticks = 125.0,
            ticks_per_second = 30.0,
            timeline_offset_ticks = 143.0,
          },
          idle_3 = {
            model = "templates/heroes/saika_magoichi/b01_ani_stand3.fbx",
            animation = "Take 001",
            duration_ticks = 53.0,
            ticks_per_second = 30.0,
            timeline_offset_ticks = 747.0,
          },
          move = {
            model = "templates/heroes/saika_magoichi/b01_ani_run.fbx",
            animation = "Take 001",
            duration_ticks = 23.0,
            ticks_per_second = 30.0,
            timeline_offset_ticks = 394.0,
          },
          attack = {
            model = "templates/heroes/saika_magoichi/b01_ani_attack.fbx",
            animation = "Take 001",
            duration_ticks = 100.0,
            ticks_per_second = 30.0,
            timeline_offset_ticks = 268.0,
          },
          critical = {
            model = "templates/heroes/saika_magoichi/b01_ani_attack.fbx",
            animation = "Take 001",
            duration_ticks = 100.0,
            ticks_per_second = 30.0,
            timeline_offset_ticks = 268.0,
          },
          sniper = {
            model = "templates/heroes/saika_magoichi/b01_ani_stand3.fbx",
            animation = "Take 001",
            duration_ticks = 53.0,
            ticks_per_second = 30.0,
            timeline_offset_ticks = 747.0,
          },
        },
        animations = {
          move = {
            source = "move",
            start_tick = 0.0,
            end_tick = 23.0,
            loop = true,
          },
          attack = {
            source = "attack",
            start_tick = 0.0,
            repeat_start_tick = 20.0,
            impact_tick = 22.0,
            end_tick = 100.0,
            loop = false,
          },
          critical = {
            source = "critical",
            start_tick = 0.0,
            repeat_start_tick = 20.0,
            impact_tick = 22.0,
            end_tick = 100.0,
            loop = false,
          },
          idle = {
            source = "idle",
            start_tick = 0.0,
            end_tick = 80.0,
            loop = true,
          },
          idle_2 = {
            source = "idle_2",
            start_tick = 0.0,
            end_tick = 125.0,
            loop = true,
          },
          idle_3 = {
            source = "idle_3",
            start_tick = 0.0,
            end_tick = 53.0,
            loop = true,
          },
          sniper = {
            source = "sniper",
            start_tick = 0.0,
            end_tick = 53.0,
            loop = true,
          },
        },
      },
      ue = {
        animation = {
          idle_variants = { "stand_1", "stand_2", "stand_3" },
          locomotion_variants = {
            walk = "walk",
            sniper_mode_walk = "sniper_walk",
          },
          anim_bp_variables = {
            locomotion = "Locomotion",
            locomotion_variant = "LocomotionVariant",
            animation_overlay = "AnimationOverlay",
            idle_variant = "IdleVariant",
            action_state = "ActionState",
            attack_phase = "AttackPhase",
            phase_progress = "PhaseProgress",
            action_instance_id = "ActionInstanceId",
            critical = "bCriticalAttack",
          },
          state_mapping = {
            stand_1 = "Stand1",
            stand_2 = "Stand2",
            stand_3 = "Stand3",
            walk = "Walk",
            sniper_walk = "SniperWalk",
            attack = "Attack",
            critical_attack = "CriticalAttack",
          },
          attack_phase_mapping = {
            windup = "Windup",
            impact = "Impact",
            recovery = "Recovery",
          },
          critical_attack = {
            action_state = "CriticalAttack",
            animation = "critical",
          },
          default_play_rate = 1.0,
          fallback_policy = "UseGenericState",
        },
      },
      level_growth = {
        strength_per_level = 1.8,
        agility_per_level = 3.2,
        intelligence_per_level = 1.6,
        damage_per_level = 2.8,
        hp_per_level = 58.0,
        mana_per_level = 26.0,
      },
    },
    {
      id = "date_masamune",
      rust_module = "B02_date_masamune",
      display_name = "伊達政宗",
      title = "獨眼龍",
      portrait = "data/hero_portraits/hero_date_masamune_portrait.png",
      background = "奧州的霸主，以炎之鬥技與火繩銃聞名",
      abilities = {
        "flame_blade",
        "fire_dash",
        "flame_assault",
        "matchlock_gun",
      },
      strength = 26,
      agility = 22,
      intelligence = 14,
      primary_attribute = "strength",
      attack_range = 250.0,
      base_damage = 60,
      base_armor = 2.5,
      base_hp = 680,
      base_mana = 240,
      move_speed = 310.0,
      turn_speed = 200.0,
      level_growth = {
        strength_per_level = 3.0,
        agility_per_level = 1.8,
        intelligence_per_level = 1.4,
        damage_per_level = 3.2,
        hp_per_level = 68.0,
        mana_per_level = 22.0,
      },
    },
    {
      id = "training_luminary",
      ue = { native_only = true },
      display_name = "晨光導師",
      title = "技能範本英雄",
      background = "測試宣告式技能與通用 Unreal 呈現的原創訓練英雄。",
      abilities = { "lumen_bolt", "lumen_touch", "lumen_lance", "lumen_mend" },
      strength = 18,
      agility = 18,
      intelligence = 25,
      primary_attribute = "intelligence",
      attack_range = 550.0,
      base_damage = 45,
      base_armor = 1.0,
      base_hp = 550,
      base_mana = 400,
      move_speed = 310.0,
      turn_speed = 540.0,
      level_growth = {
        strength_per_level = 2.0,
        agility_per_level = 1.8,
        intelligence_per_level = 3.0,
        damage_per_level = 2.4,
        hp_per_level = 52.0,
        mana_per_level = 36.0,
      },
    },
  }
  -- Opt-in teaching loadout; append-only ID, no character-specific native code.
  local source
  for _, hero in ipairs(heroes) do if hero.id == "training_luminary" then source = hero end end
  assert(source, "training apprentice requires the luminary template")
  local apprentice = {}
  for key, value in pairs(source) do apprentice[key] = value end
  apprentice.id = "training_apprentice"
  apprentice.display_name = "晨光學徒"
  apprentice.background = "驗證未學習技能、首次學習與等級解鎖的訓練英雄。"
  apprentice.abilities = { "apprentice_bolt", "apprentice_touch", "apprentice_lance", "apprentice_mend" }
  apprentice.moba_loadout = { ranks = { 0, 0, 0, 0 }, skill_points = 1 }
  heroes[#heroes + 1] = apprentice
  -- Append after existing heroes: numeric IDs and tombstones remain stable.
  for _,hero in ipairs(ctx.include('templates/moba_archetypes.lua').heroes) do
    heroes[#heroes+1]=hero
  end
  return heroes
end
