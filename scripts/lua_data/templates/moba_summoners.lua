-- Training summoner spells. Names, descriptions, and balance live only here.
-- catalog_id is append-only numeric identity. Declaration order does not assign it.
-- An empty or omitted list is rejected by the content model. No legacy default.
return function(ctx)
  return {
    {
      catalog_id = 1,
      id = "training_point_blink_v1",
      display_name = "閃躍",
      description = "指定地面一點並瞬間移動。超出距離時沿射線夾到範圍內。可跨越牆面，落點必須可站。",
      cooldown = 300,
      target = "point",
      operation = { kind = "point_blink", range = 400, out_of_range = "clamp_to_range" },
    },
    {
      catalog_id = 2,
      id = "training_self_heal_v1",
      display_name = "復甦",
      description = "立即回復自身生命，不選取任何單位。",
      cooldown = 240,
      target = "none",
      operation = { kind = "self_heal", amount = 200 },
    },
    {
      catalog_id = 3,
      id = "training_self_shield_v1",
      display_name = "護障",
      description = "立即獲得 160 點護盾，持續 2 秒。護盾不累加：保留較高護盾量及較長時限。可在控制狀態使用，不中斷移動；不受禁療影響。",
      cooldown = 180,
      target = "none",
      operation = { kind = "self_shield", amount = 160, duration = 2 },
    },
    {
      catalog_id = 4,
      id = "training_control_cleanse_v1",
      display_name = "淨念",
      description = "立即解除自身目前的暈眩、定身、沉默與正式技能／彈體緩速，可在這些控制狀態使用。保留自用姿態減速、持續傷害、抑療與增益，不提供控制免疫；沒有控制時仍消耗冷卻，不中斷移動。",
      cooldown = 210,
      target = "none",
      operation = { kind = "self_control_cleanse" },
    },
    {
      catalog_id = 5,
      id = "training_self_sprint_v1",
      display_name = "疾行",
      description = "自身移速增加 80，持續 6 秒；與道具疾行只採最高加成，各自倒數。可在控制狀態使用，但不解除控制、不提供控制免疫或穿越單位；不取消原有移動命令。",
      cooldown = 180,
      target = "none",
      operation = { kind = "self_sprint", ms_bonus = 80, duration = 6 },
    },
    {
      catalog_id = 6,
      id = "training_self_guard_v1",
      display_name = "堅守",
      description = "自身受到的傷害降低 25%，持續 3 秒；與道具減傷只採最高減免，各自倒數。可在控制狀態使用，但不解除控制、不提供控制免疫；不取消原有移動命令，也不回復生命或產生護盾。",
      cooldown = 180,
      target = "none",
      operation = { kind = "self_damage_reduction", reduction = 0.25, duration = 3 },
    },
    {
      catalog_id = 7,
      id = "training_self_mana_v1",
      display_name = "澄息",
      description = "立即回復自身 200 點魔力，最多回到魔力上限，不受禁療影響。需要已啟用且有容量的魔力池；魔力已滿仍消耗冷卻。可在控制狀態使用，不解除控制、不取消移動或引導。",
      cooldown = 180,
      target = "none",
      operation = { kind = "self_restore_mana", amount = 200 },
    },
  }
end
