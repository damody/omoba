-- Live evidence of jungle camps and tree cutting (E280; scripts/autopilot_plan.lua): at the own
-- side's nearest camp, cut its nearest trees with a tree-cutting slot (shots before and after, from
-- where it is aimed), pull the camp into a fight, come back while the trees are still down and again
-- once they have regrown (tree_cut REGROW_SECONDS 300). Fits a 10-minute nobu run.
--   run_moba_role_ue ... --hero 1=date_masamune --autopilot lane_v0 --autopilot-plan scripts/autopilot_plans/jungle_trees.lua
-- The cut uses slot W of 伊達政宗 (b04_fire_blast 火焰強襲, source B04E, destroy_trees radius 500,
-- range 700), aimed from the camp's open ground; another hero needs its own file.
-- No camp clear: every nobu camp (2,700+ hp, 100+ dps) kills a hero in its first minutes, so a
-- respawn is not reachable within 10 minutes; the camp step only pulls it into a fight (aggro,
-- leash). Camp death, reward and respawn are covered headless (E187 jungle tests).
local camp = { side = 'own', rank = 1 }
return { steps = {
  { ['do'] = 'cast', label = 'tree_cut', slot = 'w', tree = { near = { camp = camp } }, from = { camp = camp },
    timeout = 240, shot_start = 'tree_before', shot = 'tree_cut', shot_delay = 1.5 },
  { ['do'] = 'clear', label = 'camp_fight', camp = camp, timeout = 20, shot_start = 'camp_fight' },
  { ['do'] = 'walk', label = 'tree_still_cut', same = 'tree_cut', radius = 450, delay_from = 'tree_cut', delay = 200,
    timeout = 90, shot = 'tree_still_cut', shot_delay = 0.5 },
  { ['do'] = 'walk', label = 'tree_regrow', same = 'tree_cut', radius = 450, delay_from = 'tree_cut', delay = 310,
    timeout = 120, shot = 'tree_regrown', shot_delay = 0.5 },
} }
