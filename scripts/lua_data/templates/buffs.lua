return function(ctx)
  return {
    {
      id = "stun",
      display_name = "暈眩",
    },
    {
      id = "slow",
      display_name = "減速",
      ue = { buff_visual = { sources = {{ability_id = "ranger_shot", kind = "slow_enemy"}} } },
    },
    {
      id = "burn",
      display_name = "燃燒",
    },
    {
      id = "sniper_mode",
      display_name = "狙擊姿態",
      ue = {
        editor_category = "Hero Buffs",
        buff_visual = {
          attach_policy = "AttachToOwner",
          attach_socket = "spine_03",
          effect_path = "/Game/Effects/Buffs/VFX_SniperMode.VFX_SniperMode",
          ability_binding = { ability_id = "sniper_mode", mode = "toggle" },
          lifecycle_events = { "added", "removed", "refreshed", "updated" },
        },
        animation_overlay = {
          overlay = "sniper_mode",
          priority = 100,
          locomotion_variant_id = 2,
          locomotion = {
            walk = "sniper_walk",
          },
        },
      },
    },
    {
      id = "three_stage",
      display_name = "三段擊",
      ue = {
        editor_category = "Hero Buffs",
        buff_visual = {
          attach_policy = "AttachToOwner",
          attach_socket = "weapon_r",
          effect_path = "/Game/Effects/Buffs/VFX_ThreeStage.VFX_ThreeStage",
          ability_binding = { ability_id = "three_stage_technique", mode = "transform", multi_shot_count = 3 },
        },
        animation_overlay = {
          overlay = "three_stage",
          priority = 80,
          action = {
            attack = "multi_shot_attack",
          },
        },
      },
    },
    {
      id = "mana_regeneration", display_name = "回魔",
      ue = { buff_visual = { sources = {{ability_id = "ranger_patch", kind = "mana_buff_self", stat = "mana_regen_constant"}} } },
    },
    {
      id = "mana_capacity", display_name = "魔力容量",
      ue = { buff_visual = { sources = {{ability_id = "vanguard_recover", kind = "mana_buff_self", stat = "mana_bonus"}} } },
    },
    { id = "root", display_name = "定身", ue = { native_only = true } },
    { id = "silence", display_name = "沉默", ue = { native_only = true } },
    { id = "objective_might", display_name = "史詩之力", ue = { native_only = true } },
    { id = "objective_guard", display_name = "守護祝福", ue = { native_only = true } },
    { id = "jungle_might", display_name = "野區力量", ue = { native_only = true } },
    { id = "jungle_guard", display_name = "野區守衛", ue = { native_only = true } },
    { id = "objective_legacy", display_name = "史詩傳承", ue = { native_only = true } },
    { id = "siege_aura", display_name = "攻城祝福", ue = { native_only = true } },
    { id = "tower_plating", display_name = "塔皮加固", ue = { native_only = true } },
    { id = "tower_plating_status", display_name = "塔皮", ue = { native_only = true } },
    { id = "damage_shield", display_name = "護盾", ue = { native_only = true } },
    { id = "healing_reduction", display_name = "抑療", ue = { native_only = true } },
    { id = "objective_comeback", display_name = "目標補償啟用", ue = { native_only = true } },
    { id = "cast_preparation", display_name = "施法準備", ue = { native_only = true } },
    { id = "jungle_return", display_name = "野怪返回", ue = { native_only = true } },
    { id = "bleed", display_name = "流血", ue = { native_only = true,
      buff_visual = { sources = {{ability_id = "ranger_shot", kind = "damage_over_time_enemy"}} } } },
    { id = "ground_damage_zone", display_name = "持續區域", ue = { native_only = true } },
    { id = "recovery_channel", display_name = "治療引導", ue = { native_only = true } },
    { id = "movement_boost", display_name = "移速提升", ue = { native_only = true } },
    { id = "damage_reduction", display_name = "傷害減免", ue = { native_only = true } },
    -- Append-only: existing buff ids keep their positions.
    { id = "ability_channel", display_name = "引導", ue = { native_only = true } },
    -- G-4 forced displacement status: stunned while moved; plays the hit pose, not a walk.
    { id = "forced_motion", display_name = "強制位移", ue = { native_only = true,
      animation_overlay = { overlay = "forced_motion", priority = 1000, locomotion = { walk = "hit", stand = "hit" } } } },
  }
end
