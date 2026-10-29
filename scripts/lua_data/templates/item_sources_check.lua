-- Build-time check that MOBA items follow the source catalog (E155). Called by
-- templates/moba_items.lua with its item list and templates/source_items.lua; any
-- violation is a build error, so a shop item can never drift from history_nobu.
--
-- A shop item (not test_only) must cite source_ref and equal its source row: name, cost
-- (components + combine cost), recipe (components by their source_ref) and every supported
-- stat; it may carry no stat the source does not have, and the source row must have no
-- capability gap (needs_framework rows never reach the shop). Items with test_only are
-- engine fixtures: never listed, may not cite a source.
--
-- Like every template, the builder returns a table (scripts/content_builder.lua requires it):
-- `ctx.include("templates/item_sources_check.lua").check(items, source)`.
return function(ctx)
  local STATS = { "atk", "hp", "armor", "ms", "lifesteal", "critical_chance" }
  local M = {}
  function M.check(items, source)
    local rows, by_id, errors = {}, {}, {}
    for _, row in ipairs(source.items) do rows[row.source_ref] = row end
    for _, item in ipairs(items) do by_id[item.id] = item end
    local function fail(item, text) errors[#errors + 1] = ("item '%s': %s"):format(item.id, text) end
    for _, item in ipairs(items) do
      if item.test_only then
        if item.source_ref then fail(item, "an engine fixture (test_only) cannot cite a source") end
      else
        local row = item.source_ref and rows[item.source_ref]
        if not item.source_ref then fail(item, "a shop item needs source_ref (items outside history_nobu cannot be sold)")
        elseif not row then fail(item, "source_ref " .. item.source_ref .. " is not a shop item of the source")
        else
          if #row.gaps > 0 then fail(item, "source needs_framework: " .. table.concat(row.gaps, ", ")) end
          if item.name ~= row.name then fail(item, ("name '%s' is not the source name '%s'"):format(tostring(item.name), tostring(row.name))) end
          if item.cost ~= row.cost then fail(item, ("cost %s is not the source cost %s"):format(tostring(item.cost), tostring(row.cost))) end
          if item.description ~= row.description then fail(item, "description differs from the source") end
          for _, stat in ipairs(STATS) do
            local mine, theirs = item[stat] or 0, row.stats[stat] or 0
            if mine ~= theirs then fail(item, ("%s %s is not the source %s"):format(stat, tostring(mine), tostring(theirs))) end
          end
          -- Passive stat modifiers (E155 C): exactly the source's, no more and no less.
          local keys = {}
          for key in pairs(item.modifiers or {}) do keys[key] = true end
          for key in pairs(row.modifiers or {}) do keys[key] = true end
          for key in pairs(keys) do
            local mine, theirs = (item.modifiers or {})[key] or 0, (row.modifiers or {})[key] or 0
            if math.abs(mine - theirs) > 1e-9 then
              fail(item, ("modifier %s %s is not the source %s"):format(key, tostring(mine), tostring(theirs)))
            end
          end
          -- The active (E160): exactly the source's translated one, with its cooldown and mana cost.
          local function same(a, b)
            if type(a) ~= type(b) then return false end
            if type(a) == "number" then return math.abs(a - b) <= 1e-9 end
            if type(a) ~= "table" then return a == b end
            for k, v in pairs(a) do if not same(v, b[k]) then return false end end
            for k in pairs(b) do if a[k] == nil then return false end end
            return true
          end
          if not same(item.active, row.active) then fail(item, "active differs from the source's translated active") end
          if (item.cooldown or 0) ~= (row.cooldown or 0) then
            fail(item, ("cooldown %s is not the source %s"):format(tostring(item.cooldown), tostring(row.cooldown)))
          end
          if not same(item.aura, row.aura) then fail(item, "aura differs from the source's passive aura") end
          if not same(item.unique_passive, row.unique_passive) then fail(item, "unique passive differs from the source's unique movespeed") end
          if not same(item.shares_cooldown_with, row.shares_cooldown_with) then
            fail(item, "shared cooldown items differ from the source script and AbilitySharedCooldown group")
          end
          if not same(item.target, row.target) then fail(item, "target differs from the source's cast range and team") end
          if (item.consumable or false) ~= (row.consumable or false) then
            fail(item, ("consumable %s is not the source %s"):format(tostring(item.consumable), tostring(row.consumable)))
          end
          if (item.mana_cost or 0) ~= (row.mana_cost or 0) then
            fail(item, ("mana cost %s is not the source %s"):format(tostring(item.mana_cost), tostring(row.mana_cost)))
          end
          local want = row.recipe and row.recipe.components or {}
          local have = {}
          for _, part in ipairs(item.recipe or {}) do
            local component = by_id[part]
            have[#have + 1] = component and component.source_ref or ("<" .. part .. ">")
          end
          table.sort(have)
          local sorted = { table.unpack(want) }
          table.sort(sorted)
          if table.concat(have, ",") ~= table.concat(sorted, ",") then
            fail(item, ("recipe [%s] is not the source recipe [%s]"):format(table.concat(have, ","), table.concat(sorted, ",")))
          end
        end
      end
    end
    if #errors > 0 then error("MOBA items do not follow the source (E155):\n  " .. table.concat(errors, "\n  "), 0) end
    return true
  end
  return M
end
