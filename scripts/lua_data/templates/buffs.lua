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
  }
end
