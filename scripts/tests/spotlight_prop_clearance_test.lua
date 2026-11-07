-- The spotlight prop stage (E630): a unit prop is clicked as a unit target, so no seat stands within
-- record_hero_spotlight.PROP_HERO_CLEARANCE of it (c06_old's creep 114 from seat 7 took a hero click).
package.path='scripts/?.lua;'..package.path
local spotlight=require('record_hero_spotlight')

local stage=assert(loadfile(spotlight.PROPS_RECIPE))()
local at=stage.showcase.placements
local function distance(p,q) return math.sqrt((p[1]-q[1])^2+(p[2]-q[2])^2) end
local units={}
for _,prop in ipairs(stage.showcase.props) do if prop.kind=='unit' then units[prop.team_of]=prop end end
assert(units[1],'an allied creep of the demonstrator (C19 奇襲)')
-- c06_old 殺人越貨's enemy creep: inside its 500 cast range from the demonstrator.
assert(units[6] and distance(units[6].at,at['1'])<=500,'an enemy creep in reach of the demonstrator')
local errors=spotlight.prop_clearance_errors(stage)
assert(#errors==0,table.concat(errors,'; '))
assert(spotlight.recipe_for({hero='x',slots={{team='enemy_anchor'}}})==spotlight.PROPS_RECIPE,'the prop stage passes its own clearance')

-- The old place: next to seat 7. A body there is not clicked and is not reported.
local crowded={showcase={placements={['7']={2783,2238},['1']={2300,2400}},
  props={{kind='unit',team_of=6,at={2700,2160}},{kind='corpse',team_of=6,at={2783,2238}}}}}
errors=spotlight.prop_clearance_errors(crowded)
assert(#errors==1 and errors[1]:find('seat 7',1,true),table.concat(errors,'; '))
print('spotlight_prop_clearance_test: ok')
