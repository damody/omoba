local script=debug.getinfo(1,'S').source:sub(2)
package.path=assert(script:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local observe=require('ue_two_team_observation').observe_collision_terrain
local function line(expected,n,collision,nav)
  return ('OM_TERRAIN expected=%d instances=%d rebuild=1 collision=%d navigation=%d'):format(expected,n,collision or 0,nav or 0)
end
for _,n in ipairs({1,2,7,32}) do assert(observe(line(n,n)).complete) end
assert(not observe('').complete)
assert(not observe(line(0,0)).complete)
assert(not observe(line(33,33)).complete)
assert(not observe(line(3,2)).complete)
assert(not observe(line(2,2,1)).complete)
assert(not observe(line(2,2,0,1)).complete)
assert(not observe(line(2,2)..'\n'..line(0,0)).complete)
assert(observe(line(0,0)..'\n'..line(7,7)).complete)
assert(not observe('Invalid collision terrain presentation\n'..line(2,2)).complete)
print('collision terrain observer: 13 assertions passed')
