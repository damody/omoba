-- Bot cast policies derived for every seated hero's kit (E260): an ability the recipe gives no
-- explicit policy gets the `bot` hint docs/character_pipeline/tools/v3_bot.lua derived from its v3
-- document (authored_heroes.lua `v3.bot`), when the bot executor (omoba-core moba_match/bots/
-- abilities.rs BotAbilityIntent) has that intent and the compiled ability passes the executor's own
-- checks (active or ultimate, instant cast, matching target type, the named extras present).
-- Explicit policies keep their place and their priority; derived ones follow in seat and kit order.
local source = debug.getinfo(1, 'S').source:sub(2)
local dir = assert(source:match('^(.*)[/\\]'))
local authored = assert(loadfile(dir .. '/templates/authored_heroes.lua'))()({})

-- intent kind -> { target type, parameters copied from the hint }
local SUPPORTED = {
  enemy_unit = { target = 'unit', params = {} },
  enemy_hero = { target = 'unit', params = {} },
  ally_heal = { target = 'unit', params = { 'below_hp_per_mille' } },
  enemy_point = { target = 'point', params = { 'radius_key', 'min_targets' }, extras = { 'radius_key' } },
  -- A range-free leap area (E281): reaches its targeting reach extra.
  leap_enemy_point = { target = 'point', params = { 'radius_key', 'reach_key', 'min_targets' }, extras = { 'radius_key', 'reach_key' } },
  enemy_line = { target = 'point', params = { 'min_targets' }, shape = 'line' },
  enemy_cone = { target = 'point', params = { 'min_targets' }, shape = 'cone' },
  approach_enemy_point = { target = 'point', params = { 'min_distance' } },
  summon_near_enemy = { target = 'point', params = { 'threat_radius' } },
  toggle_engage = { target = 'none', params = { 'threat_radius', 'on_status' }, toggle = true },
  self_heal = { target = 'none', params = { 'below_hp_per_mille' } },
  self_area = { target = 'none', params = { 'radius_key', 'min_targets' }, extras = { 'radius_key' } },
  self_engage = { target = 'none', params = { 'threat_radius' } },
}

local rows = {}
for _, row in ipairs(authored.abilities) do rows[row.id] = row end
local kits = {}
for _, hero in ipairs(authored.heroes) do kits[hero.id] = hero.abilities end
-- Hand-ported heroes whose kit the packages replace (hero_updates) seat with that kit.
for id, update in pairs(authored.hero_updates or {}) do kits[id] = update.abilities_by_slot end
-- The v3 heroes of templates/v3_test_heroes.lua (merged into the content heroes by heroes.lua) seat
-- with their own rows: a v3 hero marked playable is one of them, not an authored hero (E371).
local v3_heroes = assert(loadfile(dir .. '/templates/v3_test_heroes.lua'))()({})
for _, row in ipairs(v3_heroes.abilities or {}) do rows[row.id] = rows[row.id] or row end
for _, hero in ipairs(v3_heroes.heroes or {}) do kits[hero.id] = kits[hero.id] or hero.abilities end

-- The executor's compile checks, so a derived policy never fails the plan.
local function admissible(row, hint)
  local kind = SUPPORTED[hint.kind]
  if not kind or hint.proposed then return false end
  if kind.toggle then
    if row.ability_type ~= 'toggle' or type(hint.on_status) ~= 'string' or hint.on_status == '' then return false end
  elseif row.ability_type ~= 'active' and row.ability_type ~= 'ultimate' then return false end
  -- A pulse channel (no recovery rule) casts like an instant; the bot holds it (E281).
  local channel = row.cast_type == 'channeled' and not kind.toggle and type(row.channel) == 'table' and row.channel.pulses_key ~= nil
  if (row.cast_type ~= 'instant' and not channel) or row.target_type ~= kind.target then return false end
  for _, key in ipairs(kind.extras or {}) do
    local name = hint[key]
    if type(name) ~= 'string' or type(row.extras) ~= 'table' or type(row.extras[name]) ~= 'table' then return false end
  end
  if kind.shape and not (row.targeting_shape and row.targeting_shape.kind == kind.shape) then return false end
  -- A line reads its width from the compiled line damage, else the `shape_width` extra.
  if hint.kind == 'enemy_line' and not (row.targeting_shape.width_key == 'shape_width'
    and type(row.extras) == 'table' and type(row.extras.shape_width) == 'table') then return false end
  -- A placed area reaches only its cast range: a range-free leap is leap_enemy_point (E281).
  if hint.kind == 'enemy_point' then
    for _, level in ipairs(row.levels or {}) do
      if (level.range or 0) <= 0 then return false end
    end
  end
  if hint.kind == 'approach_enemy_point' or hint.kind == 'summon_near_enemy' then
    for _, level in ipairs(row.levels or {}) do
      if (level.range or 0) <= (hint.min_distance or 0) then return false end
    end
  end
  return true
end

local function intent(hint)
  local out = { kind = hint.kind }
  for _, name in ipairs(SUPPORTED[hint.kind].params) do out[name] = hint[name] end
  return out
end

-- policies: the recipe's explicit list (kept first); players: the recipe's seats.
return function(policies, players)
  local listed = {}
  for _, policy in ipairs(policies) do listed[policy.ability] = true end
  local out = {}
  for _, policy in ipairs(policies) do out[#out + 1] = policy end
  local seen = {}
  for _, player in ipairs(players) do
    if not seen[player.hero] then
      seen[player.hero] = true
      for _, ability in ipairs(kits[player.hero] or {}) do
        local row = rows[ability]
        local hint = row and row.v3 and row.v3.bot
        if not listed[ability] and type(hint) == 'table' and admissible(row, hint) then
          listed[ability] = true
          out[#out + 1] = { ability = ability, intent = intent(hint) }
        end
      end
    end
  end
  return out
end
