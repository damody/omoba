-- Live evidence of jungle camps and tree cutting (E280, E290; scripts/autopilot_plan.lua): at the
-- own side's weakest camp, cut its nearest trees with a tree-cutting slot (shots before and after,
-- from where it is aimed), clear the camp, wait on the spot until it respawns (camp respawn 120 s),
-- then come back to the cut trees while they are still down and once they have regrown (tree_cut
-- REGROW_SECONDS 300). Fits a 10-minute nobu run with the recipe that starts every hero at a
-- higher level (moba_nobu_jungle_check.lua, showcase start_level): a level-1 hero dies to any
-- nobu camp (2,700+ hp, 100+ dps).
--   run_moba_role_ue ... --recipe scripts/lua_data/moba_nobu_jungle_check.lua --hero 1=date_masamune
--     --autopilot lane_v0 --autopilot-plan scripts/autopilot_plans/jungle_trees.lua
-- The cut uses slot W of 伊達政宗 (b04_fire_blast 火焰強襲, source B04E, destroy_trees radius 500,
-- range 700), aimed from the camp's open ground; another hero needs its own file.
local camp = { side = 'own', order = 'weakest' }
return { steps = {
  { ['do'] = 'cast', label = 'tree_cut', slot = 'w', tree = { near = { camp = camp } }, from = { camp = camp },
    timeout = 240, shot_start = 'tree_before', shot = 'tree_cut', shot_delay = 1.5 },
  { ['do'] = 'clear', label = 'camp_clear', camp = camp, timeout = 120, shot_start = 'camp_fight', shot = 'camp_cleared' },
  { ['do'] = 'watch', label = 'camp_respawn', same = 'camp_clear', timeout = 180, shot = 'camp_respawned', shot_delay = 1.0 },
  { ['do'] = 'walk', label = 'tree_still_cut', same = 'tree_cut', radius = 450, delay_from = 'tree_cut', delay = 200,
    timeout = 90, shot = 'tree_still_cut', shot_delay = 0.5 },
  { ['do'] = 'walk', label = 'tree_regrow', same = 'tree_cut', radius = 450, delay_from = 'tree_cut', delay = 310,
    timeout = 120, shot = 'tree_regrown', shot_delay = 0.5 },
} }
