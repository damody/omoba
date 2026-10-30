-- E357: lane roles come from hero data; the best total fit wins, smaller groups fill fewer lanes,
-- and the shipped heroes resolve to traits from their unit stats and kits.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local lanes = require('hero_lane_roles')

local checks = 0
local function check(value, message) assert(value, message); checks = checks + 1 end
local ROLES = { 'top', 'mid', 'carry', 'support', 'jungle' }
local fake = {
  bruiser = { ranged = false, primary = 'strength', heal = 0, mobility = 0, control = 1 },
  caster = { ranged = true, primary = 'intelligence', heal = 0, mobility = 0, control = 2 },
  marksman = { ranged = true, primary = 'agility', heal = 0, mobility = 0, control = 0 },
  healer = { ranged = true, primary = 'intelligence', heal = 2, mobility = 0, control = 1 },
  diver = { ranged = false, primary = 'agility', heal = 0, mobility = 2, control = 0 },
}
local function of(id) return assert(fake[id], id) end

-- listed in no lane order: the assignment follows the traits, not the list
local seated = lanes.assign({ 'healer', 'diver', 'marksman', 'caster', 'bruiser' }, ROLES, of)
check(seated.top == 'bruiser' and seated.mid == 'caster' and seated.carry == 'marksman'
  and seated.support == 'healer' and seated.jungle == 'diver', 'each hero takes the lane it fits')
local two = lanes.assign({ 'marksman', 'healer' }, ROLES, of)
local count = 0
for _ in pairs(two) do count = count + 1 end
check(count == 2 and two.carry == 'marksman' and two.support == 'healer', 'a smaller group fills its best lanes only')
check(not pcall(lanes.assign, { 'a', 'b' }, { 'top' }, of), 'more heroes than lanes is refused')
check(lanes.fit('support', fake.healer) > lanes.fit('support', fake.marksman), 'healing raises the support fit')

-- real data: every shipped hero has unit stats and a role in a full assignment
local shipped = lanes.assign({ 'murakami_yoshikiyo', 'date_masamune', 'saika_magoichi', 'oichi', 'asai_nagamasa' }, ROLES)
check(shipped.carry == 'saika_magoichi', 'the ranged agility hero carries')
local t = lanes.traits('oichi')
check(t.ranged and t.primary == 'intelligence' and t.heal > 0, 'traits read the applied unit stats and the kit')
check(lanes.traits('hattori_hanzo').primary ~= nil, 'v3 test heroes get unit stats from source_stats')
print(('hero_lane_roles tests passed (%d checks)'):format(checks))
