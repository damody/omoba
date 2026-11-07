-- Summon templates owned by I1 tamako (appended to templates/summons.lua by build).
-- 百箭齊發 lethal arrows: one template per R rank (attack 50/100/150).
-- 原作 C15T_dummy（npc_units_custom_load.txt:5187-5259）：DOTA_UNIT_CAP_MOVE_FLY 移速 900、
-- 近戰 AttackRange 10（本遊戲以中心距離量射程：10＋雙方 DOTA_HULL_SIZE_HERO 24＝58）、AttackRate 1、AttackAcquisitionRange 900、視野 400；
-- majia_vison／modifier_C15T：無敵、不可選取、無碰撞。箭矢由 base_content
-- summons::chaser（飛行追擊 AI）驅動：直線飛向鎖定目標（summon_target），貼身才攻擊。
local function arrow(rank, damage)
  return {
    id = "c15_lethal_arrow_r" .. rank,
    display_name = "致命箭矢",
    native_visual_profile = "training_ranger",
    hp = 10, damage = damage, duration = 10,
    move_speed = 900, attack_range = 58, attack_interval = 1,
    armor = 0, magic_resistance = 0, collision_radius = 0, vision_radius = 400,
    follow_start_distance = 0, follow_stop_distance = 0,
    target_priority = "nearest",
    invulnerable = true,
    damage_per_owner_level = 0,
    owner_death = "persist",
  }
end

return function(ctx)
  return {
    schema_version = 2,
    summons = { arrow(1, 50), arrow(2, 100), arrow(3, 150) },
  }
end
