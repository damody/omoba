-- Bot cast policies from package schema v3 (M6 prerequisite): when and at what a server bot or the
-- UE autopilot casts an ability, so no hero needs bot code of its own. An ability (or item active)
-- may write `bot = { kind = ..., <params> }` or `bot = false` (never cast); without it the policy is
-- derived from the document's target, cast, cost and behavior.
--
-- Kinds and parameters mirror omoba-core moba_match/bots/abilities.rs BotAbilityIntent (serde
-- snake_case, `kind` tag): a `*_key` parameter is written `$value` in the document and exported as
-- the value name (an extras key of the compiled ability). Kinds marked `proposed` are not in the
-- executor yet (docs/plans/2026-10-28-history-nobu-full-port/m6-bot-intents.md); a bot skips them.
local M = {}

-- kind -> { params = { name = "per_mille" | "radius" | "count" | "value" }, proposed? }
M.KINDS = {
  enemy_unit = { params = {} },
  enemy_hero = { params = {} },
  enemy_point = { params = { radius_key = "value", min_targets = "count" } },
  enemy_cone = { params = { min_targets = "count" } },
  enemy_line = { params = { min_targets = "count" } },
  approach_enemy_point = { params = { min_distance = "radius" } },
  self_heal = { params = { below_hp_per_mille = "per_mille" } },
  self_area = { params = { radius_key = "value", min_targets = "count" } },
  self_engage = { params = { threat_radius = "radius" } },
  self_channel_recovery = { params = { below_hp_per_mille = "per_mille", threat_radius = "radius" } },
  ally_heal = { params = { below_hp_per_mille = "per_mille" } },
  ally_heal_or_cleanse = { params = { below_hp_per_mille = "per_mille" } },
  self_recovery = { params = { below_hp_per_mille = "per_mille", below_mana_per_mille = "per_mille", restore_key = "value" } },
  self_mana_regeneration = { params = { below_hp_per_mille = "per_mille", below_mana_per_mille = "per_mille",
    rate_key = "value", duration_key = "value" } },
  -- proposed (m6-bot-intents.md): not cast until the executor has them
  toggle_engage = { params = { threat_radius = "radius" }, proposed = true },
  ally_engage = { params = { threat_radius = "radius" }, proposed = true },
  summon_near_enemy = { params = { threat_radius = "radius" }, proposed = true },
  escape = { params = { below_hp_per_mille = "per_mille", threat_radius = "radius" }, proposed = true },
  channel_enemy_area = { params = { radius_key = "value", min_targets = "count" }, proposed = true },
}
M.DEFAULTS = { heal_below = 600, threat = 900, engage = 900, min_targets = 1, mana_below = 400, escape_below = 300, approach = 300 }

local function sorted_keys(t)
  local ks = {}
  for k in pairs(t) do ks[#ks + 1] = k end
  table.sort(ks, function(a, b) return tostring(a) < tostring(b) end)
  return ks
end

local function value_name(expr)
  if type(expr) ~= "string" then return nil end
  return expr:match("^%s*%$([%a_][%w_]*)%s*$")
end

-- every action and selector of a program tree
local function walk(t, on_action, on_selector)
  if type(t) ~= "table" then return end
  if type(t.op) == "string" then on_action(t) end
  if type(t.shape) == "string" then on_selector(t) end
  for _, k in ipairs(sorted_keys(t)) do walk(t[k], on_action, on_selector) end
end

-- facts about one castable's behavior
function M.facts(def, castable)
  local f = { heal_self = false, heal_target = false, cleanse = false, mana = nil, damage = false,
    summon = false, move = false, selectors = {}, enemy_status = false }
  local function action(a)
    if a.op == "heal" then
      if a.to == "caster" or a.to == "self" then f.heal_self = true else f.heal_target = true end
    elseif a.op == "restore_mana" and (a.to == "caster" or a.to == "self") then f.mana = value_name(a.amount)
    elseif a.op == "remove_status" then f.cleanse = true
    elseif a.op == "damage" then f.damage = true
    elseif a.op == "create_unit" or a.op == "create_illusion" then f.summon = true
    elseif a.op == "dash" or a.op == "leap" or a.op == "blink" or a.op == "teleport" then f.move = true
    elseif a.op == "apply_status" and a.to ~= "caster" and a.to ~= "self" then f.enemy_status = true end
  end
  -- the cast's own selectors decide the aim; status triggers (on-hit cleaves, auras) only add facts
  local function selector(s) f.selectors[#f.selectors + 1] = s end
  walk(castable.events, action, selector)
  for _, s in ipairs(def.statuses or {}) do walk(s.triggers, action, function() end) end
  return f
end

-- the derived policy (a table with `kind`, or { kind = "none", reason })
function M.derive(def, kind_of_entry)
  local castable = kind_of_entry == "item" and def.active or def
  if not castable or (kind_of_entry == "ability" and def.kind == "passive") then return { kind = "none", reason = "passive" } end
  if kind_of_entry == "ability" and (def.slot == "hidden" or def.slot == "sub") then
    return { kind = "none", reason = "sub-ability (cast through its parent)" }
  end
  local D = M.DEFAULTS
  local target = castable.target or {}
  local cast = castable.cast or {}
  local f = M.facts(def, castable)
  local toggle = kind_of_entry == "ability" and def.kind == "toggle"
  if toggle then return { kind = "toggle_engage", threat_radius = D.threat } end
  if cast.channel_time then
    if f.heal_self then return { kind = "self_channel_recovery", below_hp_per_mille = D.heal_below, threat_radius = D.threat } end
    for _, s in ipairs(f.selectors) do
      local r = value_name(s.radius)
      if s.shape == "circle" and r then return { kind = "channel_enemy_area", radius_key = r, min_targets = D.min_targets } end
    end
    return { kind = "self_engage", threat_radius = D.engage, note = "channel without a recovery or area rule" }
  end
  local t = target.type or "none"
  local team = target.team or "enemy"
  local hero_only = type(target.types) == "table" and #target.types == 1 and target.types[1] == "hero"
  if t == "unit" or t == "unit_or_point" then
    if team == "enemy" or team == "both" then
      if team == "both" and (f.heal_target or f.cleanse) and not f.damage then
        return { kind = f.cleanse and "ally_heal_or_cleanse" or "ally_heal", below_hp_per_mille = D.heal_below }
      end
      return { kind = hero_only and "enemy_hero" or "enemy_unit" }
    end
    if f.heal_target or f.heal_self then
      return { kind = f.cleanse and "ally_heal_or_cleanse" or "ally_heal", below_hp_per_mille = D.heal_below }
    end
    return { kind = "ally_engage", threat_radius = D.engage }
  end
  if t == "point" or t == "direction" or t == "vector" then
    if f.move and not f.damage then return { kind = "approach_enemy_point", min_distance = D.approach } end
    local area = value_name(target.aoe_radius)
    if area then return { kind = "enemy_point", radius_key = area, min_targets = D.min_targets } end
    for _, s in ipairs(f.selectors) do
      if s.shape == "cone" then return { kind = "enemy_cone", min_targets = D.min_targets } end
      if s.shape == "line" or s.shape == "segment" or s.shape == "rect" then return { kind = "enemy_line", min_targets = D.min_targets } end
    end
    for _, s in ipairs(f.selectors) do
      local r = value_name(s.radius)
      if s.shape == "circle" and r then return { kind = "enemy_point", radius_key = r, min_targets = D.min_targets } end
    end
    if f.summon then return { kind = "summon_near_enemy", threat_radius = D.threat } end
    if f.move then return { kind = "approach_enemy_point", min_distance = D.approach } end
    return { kind = "enemy_line", min_targets = D.min_targets, note = "point skill without an area: aimed at the nearest enemy" }
  end
  -- no target
  if f.mana and not f.damage then
    return { kind = "self_recovery", below_hp_per_mille = 0, below_mana_per_mille = D.mana_below, restore_key = f.mana }
  end
  if f.heal_self and not f.damage then return { kind = "self_heal", below_hp_per_mille = D.heal_below } end
  if f.summon then return { kind = "summon_near_enemy", threat_radius = D.threat } end
  for _, s in ipairs(f.selectors) do
    local r = value_name(s.radius)
    local around_caster = s.from == nil or s.from == "caster" or s.from == "self"
    if s.shape == "circle" and r and around_caster and (s.team == nil or s.team == "enemy") and (f.damage or f.enemy_status) then
      return { kind = "self_area", radius_key = r, min_targets = D.min_targets }
    end
  end
  return { kind = "self_engage", threat_radius = D.engage }
end

-- validation of an authored `bot` field: problems as strings; find_value(name) -> def|nil
function M.check(bot, find_value)
  local problems = {}
  if bot == false then return problems end
  if type(bot) ~= "table" then return { "bot 必須是 table 或 false" } end
  local spec = M.KINDS[bot.kind]
  if not spec then return { "bot.kind `" .. tostring(bot.kind) .. "` 不在詞彙表（v3_bot.lua KINDS）" } end
  for k, v in pairs(bot) do
    if k ~= "kind" and k ~= "notes" then
      local p = spec.params[k]
      if not p then problems[#problems + 1] = "bot." .. tostring(k) .. " 不是 " .. bot.kind .. " 的參數"
      elseif p == "value" then
        local n = value_name(v)
        if not n then problems[#problems + 1] = "bot." .. k .. " 必須寫成 `$值名稱`"
        elseif not find_value(n) then problems[#problems + 1] = "bot." .. k .. "：值 `" .. n .. "` 不存在" end
      elseif math.type(v) ~= "integer" then problems[#problems + 1] = "bot." .. k .. " 必須是整數"
      elseif p == "per_mille" and (v < 0 or v > 1000) then problems[#problems + 1] = "bot." .. k .. " 必須在 0～1000"
      elseif p == "radius" and (v < 1 or v > 10000) then problems[#problems + 1] = "bot." .. k .. " 必須在 1～10000"
      elseif p == "count" and (v < 1 or v > 128) then problems[#problems + 1] = "bot." .. k .. " 必須在 1～128" end
    end
  end
  for k in pairs(spec.params) do
    if bot[k] == nil and not (bot.kind == "self_recovery" and k == "below_hp_per_mille") then
      problems[#problems + 1] = "bot 缺少 " .. bot.kind .. " 的參數 " .. k
    end
  end
  return problems
end

-- the catalog row: Rust BotAbilityIntent shape plus `source` (authored | derived) and `proposed`
function M.export(def, kind_of_entry)
  local castable = kind_of_entry == "item" and def.active or def
  local authored = castable and castable.bot
  if kind_of_entry == "item" and authored == nil then authored = def.bot end
  if authored == false then return { kind = "none", reason = "authored: never cast", source = "authored" } end
  local policy, source
  if type(authored) == "table" then
    policy, source = {}, "authored"
    for k, v in pairs(authored) do policy[k] = v end
  else
    policy, source = M.derive(def, kind_of_entry), "derived"
  end
  local spec = M.KINDS[policy.kind]
  for k, v in pairs(policy) do
    if spec and spec.params[k] == "value" then policy[k] = value_name(v) or v end
  end
  policy.notes = nil
  policy.source = source
  if spec and spec.proposed then policy.proposed = true end
  return policy
end

return M
