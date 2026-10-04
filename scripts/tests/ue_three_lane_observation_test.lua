local script = debug.getinfo(1,'S').source:sub(2)
package.path = assert(script:match('^(.*)[/\\]')) .. '/../?.lua;' .. package.path
local observe = require('ue_two_team_observation').observe_three_lane_map
local function line(id,n) return 'Synced map route ' .. id .. ' with ' .. n .. ' point(s)' end
local valid = line('route_0',4)..'\n'..line('route_1',2)..'\n'..line('route_2',4)
assert(observe(valid).complete)
assert(observe(valid..'\n'..valid).complete)
assert(not observe('').complete)
assert(not observe(line('route_0',4)..'\n'..line('route_1',2)).complete)
assert(not observe(valid..'\n'..line('route_3',4)).complete)
assert(not observe(valid..'\n'..line('route_0',2)).complete)
assert(not observe(valid:gsub('route_2','route_1')).complete)
assert(not observe('OM_MINIMAP available=1 routes=3').complete)
print('three-lane route observer: 8 assertions passed')
