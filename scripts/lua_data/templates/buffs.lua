return function(ctx)
  return {
    {
      id = "stun",
      display_name = "暈眩",
    },
    {
      id = "slow",
      display_name = "減速",
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
          lifecycle_events = { "added", "removed", "refreshed", "updated" },
        },
        animation_overlay = {
          overlay = "sniper_mode",
          priority = 100,
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
  }
end
