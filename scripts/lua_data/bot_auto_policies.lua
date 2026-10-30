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
  -- Any-distance skills (E387): every rank's cast range is 0.
  global_enemy_point = { target = 'point', params = { 'radius_key', 'min_targets' }, extras = { 'radius_key' }, anywhere = true },
  ally_engage = { target = 'unit', params = { 'threat_radius' } },
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
-- Abilities a kit ability swaps into a slot (a form's replacements, a recast) are rows of the same
-- package that are not in the kit; they get policies too, after the kit, in row order (E408).
-- A child cast only through its parent carries the hint kind `none` and stays without one.
local package_of = {}
for _, hero in ipairs(authored.heroes) do package_of[hero.id] = hero.authored_package end
for _, hero in ipairs(v3_heroes.heroes or {}) do package_of[hero.id] = package_of[hero.id] or hero.authored_package end
local package_rows = {}
for _, list in ipairs({ authored.abilities, v3_heroes.abilities or {} }) do
  for _, row in ipairs(list) do
    if row.authored_package then
      package_rows[row.authored_package] = package_rows[row.authored_package] or {}
      table.insert(package_rows[row.authored_package], row.id)
    end
  end
end
for id, kit in pairs(kits) do
  local in_kit, full = {}, {}
  for _, ability in ipairs(kit) do in_kit[ability] = true; full[#full + 1] = ability end
  for _, ability in ipairs(package_rows[package_of[id]] or {}) do
    if not in_kit[ability] then in_kit[ability] = true; full[#full + 1] = ability end
  end
  kits[id] = full
end

-- The executor's compile checks, so a derived policy never fails the plan.
local function admissible(row, hint)
  local kind = SUPPORTED[hint.kind]
  if not kind or hint.proposed then return false end
  if kind.toggle then
    if row.ability_type ~= 'toggle' or type(hint.on_status) ~= 'string' or hint.on_status == '' then return false end
  elseif row.ability_type ~= 'active' and row.ability_type ~= 'ultimate' then return false end
  -- A channel with no recovery rule casts like an instant; the bot holds it (E281, E387: with
  -- pulses or not, e.g. C05T 瞬雷閃 blinks when it ends).
  local recovery = false
  for _, effect in ipairs(row.effects or {}) do
    if effect.kind == 'recovery_channel_self' then recovery = true end
  end
  local channel = row.cast_type == 'channeled' and not kind.toggle and not recovery
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
  if kind.anywhere then
    for _, level in ipairs(row.levels or {}) do
      if (level.range or 0) ~= 0 then return false end
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
-- An explicit policy for an ability no seated hero has is left out (E376): it casts nothing, and
-- with the derived policies of a reseated roster it pushed the plan past the executor's 64.
return function(policies, players)
  local seated = {}
  for _, player in ipairs(players) do
    for _, ability in ipairs(kits[player.hero] or {}) do seated[ability] = true end
  end
  local listed = {}
  local out = {}
  for _, policy in ipairs(policies) do
    if seated[policy.ability] then
      listed[policy.ability] = true
      out[#out + 1] = policy
    end
  end
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
