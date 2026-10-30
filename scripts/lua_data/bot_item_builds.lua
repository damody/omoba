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

return M
