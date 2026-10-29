-- Playable heroes take their unit stats from the source roster (templates/hero_sources.lua, read
-- from history_nobu X_2.txt). Applied at build time to the hero templates, so the Rust codegen,
-- the runtime Lua content and the Unreal codegen all see the same numbers. What the attributes
-- grant on top (HP per strength, ...) is templates/attribute_rules.lua, applied by the runtime.
--
-- Mapping (Dota unit KV -> hero template):
--   AttributePrimary, AttributeBase*            -> primary_attribute, strength/agility/intelligence
--   Attribute*Gain                              -> level_growth.*_per_level (no flat HP/damage/mana growth)
--   StatusHealth, StatusMana, ArmorPhysical     -> base_hp, base_mana, base_armor
--   AttackDamageMin/Max                         -> base_damage (the mean, rounded down)
--   AttackRate, AttackRange, MovementSpeed      -> base_attack_time, attack_range, move_speed
--   StatusHealthRegen, StatusManaRegen          -> health_regen, mana_regen
-- Not mapped: MovementTurnRate (the roster's turn_rate is not in the runtime's degrees per second)
-- and MagicalResistance (one source hero sets 30; the rest 0) stay as authored.
-- Like every template it returns a table (content_builder, E159): `.apply(heroes, roster)`.
return function(ctx)
  local M = {}
  function M.apply(heroes, roster)
    local by_id = {}
    for _, entry in ipairs(roster.heroes or {}) do by_id[entry.id] = entry end
    local function whole(value, what, hero)
      local n = math.tointeger(value)
      assert(n, ("hero '%s': source %s %s is not a whole number"):format(hero.id, what, tostring(value)))
      return n
    end
    for _, hero in ipairs(heroes) do
      -- a test_only hero names its source unit with `source_stats` (scripts/v3_test_heroes.lua)
      if not hero.tombstone and ((hero.source_ref and not hero.test_only) or hero.source_stats) then
        local id = hero.source_stats or hero.source_ref:match(":([%w_]+)$")
        local entry = assert(id and by_id[id], ("hero '%s': source %s is not in the roster"):format(hero.id, tostring(hero.source_stats or hero.source_ref)))
        local s = assert(entry.stats, ("hero '%s': source hero %s has no unit stats"):format(hero.id, id))
        assert(s.primary == "strength" or s.primary == "agility" or s.primary == "intelligence",
          ("hero '%s': source primary attribute %s is not supported"):format(hero.id, tostring(s.primary)))
        hero.primary_attribute = s.primary
        hero.strength = whole(s.base_strength, "base strength", hero)
        hero.agility = whole(s.base_agility, "base agility", hero)
        hero.intelligence = whole(s.base_intelligence, "base intelligence", hero)
        hero.base_hp = whole(s.health, "health", hero)
        hero.base_mana = whole(s.mana, "mana", hero)
        hero.base_armor = s.armor
        hero.base_damage = (whole(s.attack_damage_min, "attack damage min", hero) + whole(s.attack_damage_max, "attack damage max", hero)) // 2
        hero.attack_range = s.attack_range
        hero.move_speed = s.movement_speed
        hero.base_attack_time = s.attack_rate
        hero.health_regen = s.health_regen
        hero.mana_regen = s.mana_regen
        -- AttackAcquisitionRange when the unit sets one (else game_rules.lua's hero default).
        hero.acquisition_range = s.acquisition_range
        hero.level_growth = {
          strength_per_level = s.strength_gain,
          agility_per_level = s.agility_gain,
          intelligence_per_level = s.intelligence_gain,
          damage_per_level = 0,
          hp_per_level = 0,
          mana_per_level = 0,
        }
      end
    end
    return heroes
  end
  return M
end
