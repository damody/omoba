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
          cast = {
            model = "templates/heroes/saika_magoichi/b01_ani_cast.fbx",
            animation = "Take 001", duration_ticks = 73.0, ticks_per_second = 30.0, timeline_offset_ticks = 627.0,
          },
          cast_secondary = {
            model = "templates/heroes/saika_magoichi/b01_ani_cast2.fbx",
            animation = "Take 001", duration_ticks = 50.0, ticks_per_second = 30.0, timeline_offset_ticks = 815.0,
          },
          death = {
            model = "templates/heroes/saika_magoichi/b01_ani_die.fbx",
            animation = "Take 001", duration_ticks = 110.0, ticks_per_second = 30.0, timeline_offset_ticks = 436.0,
          },
          sniper_move = {
            model = "templates/heroes/saika_magoichi/b01_ani_run2.fbx",
            animation = "Take 001", duration_ticks = 32.0, ticks_per_second = 30.0, timeline_offset_ticks = 385.0,
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
          cast = { source = "cast", start_tick = 0.0, end_tick = 73.0, loop = false },
          cast_secondary = { source = "cast_secondary", start_tick = 0.0, end_tick = 50.0, loop = false },
          death = { source = "death", start_tick = 0.0, end_tick = 110.0, loop = false },
          sniper_move = { source = "sniper_move", start_tick = 0.0, end_tick = 32.0, loop = true },
          recall = { source = "idle_3", start_tick = 0.0, end_tick = 53.0, loop = true },
        },
      },
      ue = {
        native_visual = {
          state_slots = { sniper_mode = "sniper", sniper_walk = "sniper_move" },
          -- Source FBX Material 03 is solid cloth, without an atlas texture.
          material_sections = { ["2"] = "/Game/Omoba/PresentationMaterialsV1/M_SaikaCloth.M_SaikaCloth" },
          -- The existing mesh's import coordinates own these animation imports.
          -- V1 imports with mismatched unit conversion remain as error evidence.
          animation_assets = {
            cast = { asset = "/Game/Omoba/AuthoredAnimationsV2/saika_magoichi/b01_ani_cast_Anim.b01_ani_cast_Anim", end_seconds = 73.0/30.0, loop_animation = false },
            cast_secondary = { asset = "/Game/Omoba/AuthoredAnimationsV2/saika_magoichi/b01_ani_cast2_Anim.b01_ani_cast2_Anim", end_seconds = 50.0/30.0, loop_animation = false },
            death = { asset = "/Game/Omoba/AuthoredAnimationsV2/saika_magoichi/b01_ani_die_Anim.b01_ani_die_Anim", end_seconds = 110.0/30.0, loop_animation = false },
            sniper_move = { asset = "/Game/Omoba/AuthoredAnimationsV2/saika_magoichi/b01_ani_run2_Anim.b01_ani_run2_Anim", end_seconds = 32.0/30.0, loop_animation = true },
          },
        },
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
      turn_speed = 200.0,
    },
    {
      id = "training_luminary",
      -- Not a source hero: automated tests and fixtures only, never selectable or in a match.
      test_only = true,
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
  local support={}
  for key,value in pairs(source) do support[key]=value end
  support.id='training_support';support.display_name='晨光援護者'
  support.background='晨光術士的隊友治療配裝變體，沿用共用技能與原生呈現。'
  support.abilities={'lumen_bolt','lumen_touch','lumen_lance','lumen_aid'}
  heroes[#heroes+1]=support
  local native_visuals=ctx.include('templates/moba_native_visuals.lua')
  local scatter=ctx.include('templates/moba_scatter.lua').hero
  scatter.ue.native_visual=native_visuals.training_ranger
  heroes[#heroes+1]=scatter
  local line={};for key,value in pairs(source)do line[key]=value end
  line.id='training_line_mage';line.display_name='晨光貫光者';line.title='直線光束配裝'
  line.background='使用共用直線技能與生成預覽的晨光術士配裝。'
  line.abilities={'lumen_ray','lumen_touch','lumen_lance','lumen_mend'}
  line.ue={native_only=true,native_visual=native_visuals.training_luminary}
  heroes[#heroes+1]=line
  -- Append-only G-1 loadout: homing and piercing linear ability projectiles.
  local frost={};for key,value in pairs(source)do frost[key]=value end
  frost.id='training_frost_mage';frost.display_name='晨光霜彈師';frost.title='飛行投射物配裝'
  frost.background='使用通用技能投射物（追蹤寒霜彈與直線貫穿飛槍）的晨光術士配裝。'
  frost.abilities={'lumen_frost_bolt','lumen_frost_lance','lumen_venom_orb','lumen_sentry'}
  frost.ue={native_only=true,native_visual=native_visuals.training_luminary}
  heroes[#heroes+1]=frost
  -- Append-only G-2 loadout: point and unit pulse channels.
  local channeler={};for key,value in pairs(source)do channeler[key]=value end
  channeler.id='training_channeler';channeler.display_name='晨光引導者';channeler.title='引導技配裝'
  channeler.background='使用通用引導技（地點震波與單體牽引）的晨光術士配裝。'
  channeler.abilities={'lumen_bolt','lumen_tether','lumen_lance','lumen_tremor'}
  channeler.ue={native_only=true,native_visual=native_visuals.training_luminary}
  heroes[#heroes+1]=channeler
  -- Append-only G-4 loadout: forced displacement (pull hook, knockback) and true sight.
  local grappler={};for key,value in pairs(source)do grappler[key]=value end
  grappler.id='training_grappler';grappler.display_name='晨光鉤索者';grappler.title='強制位移與偵測配裝'
  grappler.background='使用通用強制位移（直線鉤鎖拉近與單體震退）與顯形範圍的晨光術士配裝。'
  grappler.abilities={'lumen_hook','lumen_repulse','lumen_flare','lumen_mend'}
  grappler.ue={native_only=true,native_visual=native_visuals.training_luminary}
  heroes[#heroes+1]=grappler
  for _,hero in ipairs(heroes)do
    local profile=native_visuals[hero.id]
    if profile then
      -- Apprentice/support are shallow gameplay clones; give each a separate Unreal settings table.
      local ue={};for key,value in pairs(hero.ue or{})do ue[key]=value end
      ue.native_visual=profile;hero.ue=ue
    end
  end
  -- @hero_author:authored-include (scripts/hero_author.lua build): packages are
  -- appended after every hand-written hero; updates only replace slot entries.
  do
    local authored = ctx.include('templates/authored_heroes.lua')
    for _, hero in ipairs(authored.heroes or {}) do heroes[#heroes + 1] = hero end
    for _, hero in ipairs(heroes) do
      local update = (authored.hero_updates or {})[hero.id]
      if update then
        assert(update.rust_module == hero.rust_module, hero.id .. ': authored update rust_module mismatch')
        local list = {}
        for i, id in ipairs(hero.abilities or {}) do list[i] = id end
        -- Slot 5 is the optional extra action slot (D).
        for i = 1, 5 do if update.abilities_by_slot[i] then list[i] = update.abilities_by_slot[i] end end
        -- Replaced abilities stay registered (gen_hero_registry reads
        -- retained_abilities) so legacy content keeps its handlers and tests.
        local kept, in_list = {}, {}
        for _, id in ipairs(list) do in_list[id] = true end
        for _, id in ipairs(hero.retained_abilities or {}) do if not in_list[id] then kept[#kept + 1] = id; in_list[id] = true end end
        for _, id in ipairs(hero.abilities or {}) do if not in_list[id] then kept[#kept + 1] = id; in_list[id] = true end end
        hero.abilities = list
        hero.retained_abilities = kept
        -- Birth ranks follow the replaced slots; other slots keep the hero's own.
        local loadout = hero.moba_loadout or {}
        local base = loadout.ranks or { 1, 1, 1, 1 }
        local ranks = {}
        for i = 1, 5 do ranks[i] = (update.ranks_by_slot or {})[i] or base[i] end
        hero.moba_loadout = { ranks = ranks, skill_points = update.skill_points or loadout.skill_points or 0 }
        -- The package that updates a hand-written hero carries its source hero.
        if update.source_ref then hero.source_ref = update.source_ref end
      end
    end
  end
  -- v3 test heroes (scripts/v3_test_heroes.lua): test_only heroes whose package is only a v3
  -- document, appended after the authored ones; their unit stats come from `source_stats`.
  for _, hero in ipairs(ctx.include('templates/v3_test_heroes.lua').heroes or {}) do heroes[#heroes + 1] = hero end
  -- Generated Tripo bodies (templates/moba_tripo_hero_visuals.lua) replace any earlier native visual,
  -- including a borrowed CC0 rig or legacy render model, once the hero's rig and clips are imported.
  local tripo_visuals=ctx.include('templates/moba_tripo_hero_visuals.lua')
  for _,hero in ipairs(heroes)do
    local profile=tripo_visuals.profile(hero.id,hero.ue and hero.ue.native_visual)
    if profile then
      local ue={};for key,value in pairs(hero.ue or{})do ue[key]=value end
      ue.native_visual=profile;hero.ue=ue
    end
  end
  -- Every hero without its own character art (no native mesh, no legacy render model) gets the
  -- shared humanoid fallback instead of the diagnostic box; Editor art replaces it.
  local shared_fallback=ctx.include('templates/moba_shared_hero_fallback.lua')
  for _,hero in ipairs(heroes)do
    local visual=hero.ue and hero.ue.native_visual
    local rendered=hero.render and hero.render.model and hero.render.model~=''
    if not rendered and not (visual and (visual.mesh_asset or visual.static_mesh_asset)) then
      local ue={};for key,value in pairs(hero.ue or{})do ue[key]=value end
      ue.native_visual=shared_fallback.profile(hero.id,visual);hero.ue=ue
    end
  end
  -- Playable heroes take these from their source unit (templates/hero_source_stats.lua, applied in
  -- templates.lua). Authoring them here would be dead data that silently disagrees with the source.
  local SOURCE_OWNED = { 'primary_attribute', 'strength', 'agility', 'intelligence', 'base_hp', 'base_mana',
    'base_armor', 'base_damage', 'attack_range', 'move_speed', 'base_attack_time', 'health_regen', 'mana_regen', 'level_growth' }
  for _, hero in ipairs(heroes) do
    if hero.source_ref and not hero.tombstone then
      for _, field in ipairs(SOURCE_OWNED) do
        assert(hero[field] == nil, ("hero '%s' authors %s, which its source unit provides (%s)"):format(hero.id, field, hero.source_ref))
      end
    end
  end
  return heroes
end
