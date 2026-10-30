-- E358: item coverage puts the shop items no derived build reaches into the builds, leaves the
-- derived builds as they were, keeps each build inside six slots and one item per purchase group.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
local builds = assert(loadfile(dir .. '/../lua_data/bot_item_builds.lua'))()

local checks = 0
local function check(value, message) assert(value, message); checks = checks + 1 end
local _, by_id = builds.catalog()
local roles = { 'top', 'mid', 'carry', 'support', 'jungle' }
local derived = builds.derive(roles)
local before = {}
for i, build in ipairs(derived) do before[i] = table.concat(build.items, ',') end

local uncovered = builds.uncovered(derived)
local covered = builds.cover(derived, uncovered)
check(#builds.uncovered(covered) == 0, 'after coverage every shop item is reached: '
  .. table.concat(builds.uncovered(covered), ', '))
for i, build in ipairs(derived) do
  check(table.concat(build.items, ',') == before[i], 'the derived build of ' .. build.role .. ' is unchanged')
end
for _, build in ipairs(covered) do
  local held, groups = 0, {}
  for _, id in ipairs(build.items) do
    local item = assert(by_id[id])
    held = held + 1
    local group = item.purchase_group and item.purchase_group.id or item.unique_passive and item.unique_passive.group
    if group then
      check(not groups[group], build.role .. ' holds one item of group ' .. group)
      groups[group] = true
    end
  end
  check(held <= builds.SLOTS, build.role .. ' fits six slots')
end

-- a coverage item avoids a build holding an item that shares its cooldown (a free build exists)
for i, build in ipairs(covered) do
  for _, id in ipairs(build.items) do
    local added = true
    for _, kept in ipairs(derived[i].items) do if kept == id then added = false end end
    for _, other in ipairs(added and by_id[id].shares_cooldown_with or {}) do
      for _, held in ipairs(build.items) do
        check(held ~= other, build.role .. ' holds ' .. id .. ' beside ' .. other .. ', which shares its cooldown')
      end
    end
  end
end
-- a coverage goal replaces a goal: no build has more goals than derived (m6cover2: a longer
-- support build never reached its last goal, tea_bowl)
for i, build in ipairs(covered) do
  local function goals(items)
    local n = 0
    for _, id in ipairs(items) do if not by_id[id].consumable then n = n + 1 end end
    return n
  end
  check(goals(build.items) <= goals(derived[i].items), build.role .. ' has no more goals than derived')
end
check(#builds.cover(derived, {}) == #derived, 'no ids, same builds')
check(not pcall(builds.cover, derived, { 'no_such_item' }), 'an unknown item is refused')
print(('bot_item_builds tests passed (%d checks)'):format(checks))
