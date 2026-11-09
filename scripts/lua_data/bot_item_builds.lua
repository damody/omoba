-- Bot item builds derived from the shop catalog by role (D19, E336): no hand-written item list.
-- Each shop item is classified by what it gives, read from its compiled entry
-- (templates/moba_items.lua): the v3 stat lines, the active's bot hint (v3.bot, derived from the
-- item document), an aura and a unique movement passive. A role weighs those classes; its build
-- is its best-scoring final items (shop items no other item uses as a component) plus one
-- single-use item, with a penalty for items other roles already took, so the five roles between
-- them cover more of the shop. Every build carries at least one active item, so the bot executor's
-- active-item rules (omoba-core moba_match/bots/items.rs) are exercised.
--
-- The purchase planner (items.rs choose_purchase) buys a build's goals in order and assembles each
-- recipe from its components, inside six inventory slots (validate_feasible rejects a build that
-- cannot). Goals are ordered by the slots their assembly needs, largest first, and a goal that
-- would not fit is dropped.
local source = debug.getinfo(1, 'S').source:sub(2)
local dir = assert(source:match('^(.*)[/\\]'))
local content = assert(loadfile(dir .. '/../content_builder.lua'))().new(dir)

local M = {}
M.SLOTS = 6
M.FINALS = 4

-- Stat line / hint -> class.
local STAT_CLASS = {
  attack_damage = 'physical', attack_speed = 'physical', agi = 'physical',
  int = 'magic', spell_amp_pct = 'magic', mana_regen = 'magic', max_mana = 'magic',
  armor = 'tank', max_hp = 'tank', hp_regen = 'tank', str = 'tank',
}
local HINT_CLASS = {
  enemy_unit = 'magic', enemy_point = 'magic', ally_heal = 'support', self_recovery = 'tank',
}
-- Role -> class weights.
M.ROLE_WEIGHTS = {
  carry = { physical = 1.0, mobility = 0.3, tank = 0.2, support = 0.1 },
  mid = { magic = 1.0, physical = 0.3, mobility = 0.3 },
  top = { tank = 1.0, physical = 0.6, support = 0.2 },
  support = { support = 1.0, tank = 0.5, magic = 0.4 },
  jungle = { physical = 0.7, tank = 0.7, mobility = 0.3 },
}
M.DIVERSITY = 0.5
-- Single-use items are cheap and bought again after each use: a role takes one no other role
-- took while one with a use for it is left (m6item1: two roles both took liquor, sake none).
M.SINGLE_DIVERSITY = 0.1

function M.classes(item)
  local set = {}
  for _, line in ipairs(item.v3 and item.v3.stat_lines or {}) do
    if STAT_CLASS[line.stat] then set[STAT_CLASS[line.stat]] = true end
  end
  local hint = item.v3 and item.v3.bot and item.v3.bot.kind
  if hint and HINT_CLASS[hint] then set[HINT_CLASS[hint]] = true end
  local active = item.active
  if active and active.kind == 'apply_modifiers' then
    for _, modifier in ipairs(active.modifiers or {}) do
      if modifier.area then set.support = true end
      for stat in pairs(modifier.stats or {}) do
        if stat:find('damage') or stat:find('attackspeed') then set.physical = true
        elseif stat:find('regen') then set.tank = true
        elseif stat:find('movespeed') then set.mobility = true end
      end
    end
    if (active.magic_immunity or 0) > 0 then set.tank = true end
    if (active.heal or 0) > 0 then set.tank = true end
  end
  if item.aura then set.support = true end
  if item.unique_passive and (item.unique_passive.ms or 0) > 0 then set.mobility = true end
  local list = {}
  for class in pairs(set) do list[#list + 1] = class end
  table.sort(list)
  return list
end

function M.score(role, item)
  local weights = assert(M.ROLE_WEIGHTS[role], 'no item weights for role ' .. tostring(role))
  local classes = M.classes(item)
  if #classes == 0 then return 0 end
  local total = 0
  for _, class in ipairs(classes) do total = total + (weights[class] or 0) end
  return total / #classes
end

-- Slots the planner needs to assemble ITEM from nothing: the components it holds while it buys
-- the next one, recursively; the finished item takes one slot.
function M.need(item, by_id)
  local recipe = item.recipe or {}
  local need = 1
  for index, component in ipairs(recipe) do
    local part = assert(by_id[component], item.id .. ' recipe names unknown ' .. component)
    need = math.max(need, (index - 1) + M.need(part, by_id))
  end
  return need
end

function M.catalog()
  local shop, by_id, component = {}, {}, {}
  for _, item in ipairs(content.include('templates/moba_items.lua')) do
    by_id[item.id] = item
    if not item.test_only then shop[#shop + 1] = item end
  end
  for _, item in ipairs(shop) do
    for _, part in ipairs(item.recipe or {}) do component[part] = true end
  end
  return shop, by_id, component
end

local function group_of(item)
  return item.purchase_group and item.purchase_group.id or item.unique_passive and item.unique_passive.group
end

-- ROLES: role names in seat order. Returns { {role=, items={...}}, ... }.
function M.derive(roles)
  local shop, by_id, component = M.catalog()
  local finals, singles = {}, {}
  for _, item in ipairs(shop) do
    if item.consumable then singles[#singles + 1] = item
    elseif not component[item.id] then finals[#finals + 1] = item end
  end
  local taken = {}
  local function pick(role, pool, count, chosen, groups, diversity)
    local ranked = {}
    for _, item in ipairs(pool) do
      local group = group_of(item)
      if not chosen[item.id] and not (group and groups[group]) then
        local value = M.score(role, item) * ((diversity or M.DIVERSITY) ^ (taken[item.id] or 0))
        if value > 0 then ranked[#ranked + 1] = { item = item, value = value } end
      end
    end
    table.sort(ranked, function(a, b)
      if a.value ~= b.value then return a.value > b.value end
      if a.item.cost ~= b.item.cost then return a.item.cost < b.item.cost end
      return a.item.id < b.item.id
    end)
    local out = {}
    for _, row in ipairs(ranked) do
      if #out >= count then break end
      local group = group_of(row.item)
      if not (group and groups[group]) then
        out[#out + 1] = row.item
        chosen[row.item.id] = true
        if group then groups[group] = true end
      end
    end
    return out
  end
  local builds, seen_roles = {}, {}
  for _, role in ipairs(roles) do
    if not seen_roles[role] then
      seen_roles[role] = true
      local chosen, groups = {}, {}
      local goals = pick(role, finals, M.FINALS, chosen, groups)
      local has_active = false
      for _, item in ipairs(goals) do if item.active then has_active = true end end
      if not has_active then
        local actives = {}
        for _, item in ipairs(finals) do if item.active then actives[#actives + 1] = item end end
        local extra = pick(role, actives, 1, chosen, groups)
        if extra[1] then goals[#goals] = extra[1] end
      end
      -- Largest assembly first; drop goals that would not fit beside the ones before them.
      table.sort(goals, function(a, b)
        local na, nb = M.need(a, by_id), M.need(b, by_id)
        if na ~= nb then return na > nb end
        return a.id < b.id
      end)
      local single = pick(role, singles, 1, chosen, groups, M.SINGLE_DIVERSITY)[1]
      local items, held = {}, single and 1 or 0
      if single then items[1] = single.id end
      for _, item in ipairs(goals) do
        if held + M.need(item, by_id) <= M.SLOTS then
          items[#items + 1] = item.id
          held = held + 1
        end
      end
      for _, id in ipairs(items) do taken[id] = (taken[id] or 0) + 1 end
      builds[#builds + 1] = { role = role, items = items }
    end
  end
  return builds
end

-- Item coverage (E358): the shop items no build reaches, neither as a goal nor as a component a
-- goal is assembled from, so no bot ever obtains them. Shop order.
function M.uncovered(builds)
  local shop, by_id = M.catalog()
  local reached = {}
  local function reach(id)
    if reached[id] then return end
    reached[id] = true
    for _, part in ipairs((by_id[id] or {}).recipe or {}) do reach(part) end
  end
  for _, build in ipairs(builds) do
    for _, id in ipairs(build.items) do reach(id) end
  end
  local out = {}
  for _, item in ipairs(shop) do if not reached[item.id] then out[#out + 1] = item.id end end
  return out
end

-- BUILDS with each of IDS put into one build, bought first after its single-use item. A goal
-- takes the place of one of the build's goals (the latest one another build still reaches, else
-- the last), so the build is no longer and is finished as early as before (m6cover2: added in
-- front of six, it pushed the support's tea_bowl past the match end);
-- a single-use item is added beside the build's own. An id that another of IDS is assembled from
-- (mana_boots, a component of mana_boots_1) is skipped: that one covers it. The build is one
-- without a coverage item yet (once every build has one, all are open again), preferring one with
-- no item in the item's purchase group (such a goal would leave the build) and none sharing its
-- cooldown (m6cover1: mana_boots_1 beside perceive_wine, drunk again and again, was never off
-- cooldown), then one whose replaced goal another build still reaches (none goes uncovered),
-- then the role that scores the item highest, then seat order. Goals that no longer fit the six
-- slots are dropped, as derive does. Other builds are returned as they were; the normal derivation
-- is unchanged.
function M.cover(builds, ids)
  local _, by_id = M.catalog()
  local out, given = {}, {}
  for i, build in ipairs(builds) do
    local copy = {}
    for key, value in pairs(build) do copy[key] = value end
    copy.items = {}
    for k, id in ipairs(build.items) do copy.items[k] = id end
    out[i] = copy
  end
  local part_of_another = {}
  local function mark(id)
    for _, part in ipairs((by_id[id] or {}).recipe or {}) do part_of_another[part] = true; mark(part) end
  end
  for _, id in ipairs(ids) do mark(id) end
  for _, id in ipairs(ids) do if not part_of_another[id] then
    local item = assert(by_id[id], 'coverage names unknown item ' .. tostring(id))
    local group = group_of(item)
    local shared = {}
    for _, other in ipairs(item.shares_cooldown_with or {}) do shared[other] = true end
    local function clashes(build)
      for _, other in ipairs(build.items) do
        local entry = assert(by_id[other], 'build names unknown item ' .. other)
        if other ~= id and ((group and group_of(entry) == group) or shared[other]) then return true end
      end
      return false
    end
    -- the goal a coverage goal replaces in build I: the latest goal another build still reaches,
    -- else the last goal, which then no build reaches (second result)
    local function replaced(i)
      if item.consumable then return nil, false end
      local reached = {}
      local function reach(x)
        if reached[x] then return end
        reached[x] = true
        for _, part in ipairs((by_id[x] or {}).recipe or {}) do reach(part) end
      end
      for j, build in ipairs(out) do
        if j ~= i then for _, other in ipairs(build.items) do reach(other) end end
      end
      local last
      for k = #out[i].items, 1, -1 do
        local other = out[i].items[k]
        if not by_id[other].consumable then
          if reached[other] then return other, false end
          last = last or other
        end
      end
      return last, last ~= nil
    end
    local open = {}
    for i in ipairs(out) do if not given[i] then open[#open + 1] = i end end
    if #open == 0 then given = {}; for i in ipairs(out) do open[#open + 1] = i end end
    local best, best_key
    for _, i in ipairs(open) do
      local _, lost = replaced(i)
      -- lower is better: clash, then a lost goal, then the negated score
      local key = { clashes(out[i]) and 1 or 0, lost and 1 or 0, -M.score(out[i].role, item) }
      local better = not best_key
      if not better then
        for k = 1, #key do
          if key[k] ~= best_key[k] then better = key[k] < best_key[k]; break end
        end
      end
      if better then best, best_key = i, key end
    end
    local drop = replaced(best)
    given[best] = true
    local build = out[best]
    local singles, goals = {}, {}
    for _, other in ipairs(build.items) do
      local entry = assert(by_id[other], 'build names unknown item ' .. other)
      if other ~= id and other ~= drop and not (group and group_of(entry) == group) then
        if entry.consumable then singles[#singles + 1] = other else goals[#goals + 1] = other end
      end
    end
    -- a coverage goal whose assembly needs every slot (chidori: sukemune and sukezane, six) is
    -- bought without the build's single-use items, which would leave it no room
    if not item.consumable and #singles + M.need(item, by_id) > M.SLOTS then singles = {} end
    local items, held = {}, #singles
    for k, single in ipairs(singles) do items[k] = single end
    if item.consumable then items[#items + 1] = id; held = held + 1 else table.insert(goals, 1, id) end
    for _, goal in ipairs(goals) do
      if held + M.need(by_id[goal], by_id) <= M.SLOTS then
        items[#items + 1] = goal
        held = held + 1
      end
    end
    build.items = items
  end end
  return out
end

-- Coverage by rounds: a match has room for a few coverage goals, not for the whole uncovered
-- shop (72 shop items no bot obtained over the 103 matches of hero_status run 20261106, every
-- build the same). ROUND_SIZE of IDS (uncovered, shop order) for round ROUND, consecutive rounds
-- taking consecutive slices, so the groups of a sweep (verify_m6_batch, a round each) reach them all.
M.ROUND_SIZE = 5
function M.round(ids, round)
  local out = {}
  if #ids == 0 then return out end
  local start = (round * M.ROUND_SIZE) % #ids
  for k = 0, math.min(M.ROUND_SIZE, #ids) - 1 do out[#out + 1] = ids[(start + k) % #ids + 1] end
  return out
end

return M
