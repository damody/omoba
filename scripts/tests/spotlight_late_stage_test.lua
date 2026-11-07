-- The late spotlight stage: a hero whose ability is learned only above the base stage's start level
-- (A04D_old, B19D_old: auto learned at hero level 18) is recorded on a stage started at 18.
package.path='scripts/?.lua;'..package.path
local b=require('_bootstrap')
local spotlight=require('record_hero_spotlight')

local base=assert(loadfile(spotlight.RECIPE))()
local late=assert(loadfile(spotlight.LATE_RECIPE))()
assert(base.showcase.start_level==spotlight.START_LEVEL,'the base stage starts at START_LEVEL')
assert(late.showcase.start_level==spotlight.LATE_LEVEL and late.map_id==base.map_id and #late.players==#base.players,
  'the late stage is the base stage started at LATE_LEVEL')
for _,hero in ipairs({'takenaka_shigeharu_old','usami_sadamitsu_old'}) do
  local package=spotlight.load_package(b.root,hero)
  assert(package.late and spotlight.recipe_for(package)==spotlight.LATE_RECIPE,hero..' is recorded on the late stage')
end
for _,hero in ipairs({'takenaka_shigeharu','usami_sadamitsu'}) do
  local package=spotlight.load_package(b.root,hero)
  assert(not package.late and spotlight.recipe_for(package)==spotlight.RECIPE,hero..' stays on the base stage')
end
assert(not pcall(spotlight.recipe_for,{hero='maeda_toshiie',late=true,slots={}}),'no stage is late and another one')
print('spotlight_late_stage_test: ok')
