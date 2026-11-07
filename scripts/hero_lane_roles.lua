-- Lane roles derived from hero data (E357, D19): no hand-written hero -> role table.
-- A hero's fit for top / mid / carry / support / jungle comes from its unit stats (attack range and
-- primary attribute, applied from the source roster as templates.lua does) and its kit in
-- templates/authored_heroes.lua and templates/v3_test_heroes.lua: healing (a v3 bot hint ally_heal or self_heal), mobility (a leap,
-- approach or line dash hint) and control (abilities applying a debuff status).
--
--   local lanes = require('hero_lane_roles')
--   lanes.traits(hero_id)                  -> {ranged=, primary=, heal=, mobility=, control=}
--   lanes.assign({hero_id, ...}, roles [, traits_of])
--                                          -> {role -> hero_id}: the assignment with the highest
--                                             total fit (ties: role order, then hero order);
--                                             traits_of defaults to lanes.traits
local source = debug.getinfo(1, 'S').source:sub(2)
local dir = assert(source:match('^(.*)[/\\]'))
package.path = dir .. '/?.lua;' .. package.path
local M = {}

M.RANGED = 400
M.HEAL_HINTS = { ally_heal = true, self_heal = true }
M.MOBILITY_HINTS = { leap_enemy_point = true, approach_enemy_point = true, enemy_line = true }

local cache
local function data()
  if cache then return cache end
  local content = require('content_builder').new(dir .. '/lua_data')
  local heroes = {}
  -- the same application templates.lua makes
  local applied = content.include('templates/hero_source_stats.lua').apply(
    content.include('templates/heroes.lua'), content.include('templates/hero_sources.lua'))
  for _, hero in ipairs(applied) do heroes[hero.id] = hero end
  local abilities = {}
  for _, file in ipairs({ 'templates/authored_heroes.lua', 'templates/v3_test_heroes.lua' }) do
    for _, row in ipairs(content.include(file).abilities or {}) do abilities[row.id] = row end
  end
  cache = { heroes = heroes, abilities = abilities }
  return cache
end

function M.traits(id)
  local d = data()
  local hero = assert(d.heroes[id], 'unknown hero ' .. tostring(id))
  local traits = { ranged = (hero.attack_range or 0) >= M.RANGED, primary = hero.primary_attribute,
    heal = 0, mobility = 0, control = 0 }
  for _, ability_id in ipairs(hero.abilities or {}) do
    local row = d.abilities[ability_id]
    local v3 = row and row.v3 or {}
    local hint = v3.bot and v3.bot.kind
    if M.HEAL_HINTS[hint] then traits.heal = traits.heal + 1 end
    if M.MOBILITY_HINTS[hint] then traits.mobility = traits.mobility + 1 end
    for _, status in ipairs(v3.statuses or {}) do
      if status.visibility == 'debuff' then traits.control = traits.control + 1; break end
    end
  end
  return traits
end

-- How well a hero with TRAITS fits ROLE.
function M.fit(role, t)
  local ranged, melee = t.ranged and 1 or 0, t.ranged and 0 or 1
  local str, agi, int = t.primary == 'strength' and 1 or 0, t.primary == 'agility' and 1 or 0, t.primary == 'intelligence' and 1 or 0
  if role == 'top' then return 2 * melee + 2 * str + 0.5 * t.control
  elseif role == 'mid' then return 1.5 * ranged + 2 * int + 0.5 * t.control
  elseif role == 'carry' then return 2 * ranged + 2 * agi
  elseif role == 'support' then return 3 * t.heal + 1 * t.control + 0.5 * int
  elseif role == 'jungle' then return 1 * melee + 1.5 * t.mobility + 1 * agi + 0.5 * str
  end
  error('unknown lane role ' .. tostring(role))
end

-- Best assignment of HEROES (at most #ROLES) to distinct ROLES.
function M.assign(heroes, roles, traits_of)
  assert(#heroes <= #roles, 'more heroes than roles')
  local traits = {}
  for i, id in ipairs(heroes) do traits[i] = (traits_of or M.traits)(id) end
  local best, best_score, used, current = nil, -math.huge, {}, {}
  local function search(i, score)
    if i > #heroes then
      if score > best_score + 1e-9 then
        best_score = score
        best = {}
        for k, role in pairs(current) do best[role] = heroes[k] end
      end
      return
    end
    for r, role in ipairs(roles) do
      if not used[r] then
        used[r], current[i] = true, role
        search(i + 1, score + M.fit(role, traits[i]))
        used[r], current[i] = nil, nil
      end
    end
  end
  search(1, 0)
  return best, best_score
end

return M
