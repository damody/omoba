-- Hero attribute rules: what one point of strength, agility and intelligence is worth. The build
-- carries this table into Rust (omoba-template-ids ATTRIBUTE_RULES); the runtime never hard-codes a
-- coefficient. To change the rules, replace only this table.
--
-- Source: the original game, damody/history_nobu @ f966720b0956d002099d72aabec7ddc94b16d4bf
-- (2021-06-01), read as text. game/dota_addons/nobu/scripts/vscripts/equilibrium_constant.lua,
-- attached to every unit at spawn (eventforspawned.lua), sets the targets:
--   lines 9-14: HP_PER_STR 25, HP_REGEN_PER_STR 0.2, MANA_PER_INT 18, MANA_REGEN_PER_INT 0.15,
--               ARMOR_PER_AGI 0.2, ATKSPD_PER_AGI 1.5
-- and adds "target - Dota default" on top of the engine (lines 20-25 assume the defaults 20, 0.1,
-- 12, 0.05, 0.14 and 1). Unchanged since 2020-05 (the first commit had 25/0.2/20/0.15/0.333/2).
-- The primary attribute keeps the engine's 1 damage per point (the script does not change it).
-- Intelligence gives no magic resistance: hero magic resistance is a flat base (game_rules.lua).
--
-- Uncertain: agility armor. The script assumes the engine gives 0.14 and adds 0.06, but the Dota 2
-- engine of 2020-21 gave about 0.16-0.167, so players got about 0.22. The table follows the stated
-- design value 0.2.
--
-- Not used: settings.lua and gamemode.lua in the same repository are unloaded templates.
--
-- Earlier estimate (before history_nobu was the source), Dota 2 6.87-7.05: strength 20 HP and 0.03
-- regen, agility 1/7 armor and 1 attack speed, intelligence 12 mana and 0.04 regen. Dota 2 7.41f
-- for comparison: strength 22 HP and 0.1 regen, agility 0.167 armor and 1 attack speed,
-- intelligence 12 mana, 0.05 regen and 0.1% magic resistance.
return function(ctx)
  return {
    version = "history_nobu f966720b",
    strength = {
      health = 25,          -- max HP per point
      health_regen = 0.2,   -- HP per second per point
    },
    agility = {
      armor = 0.2,          -- armor per point (design value; see above)
      attack_speed = 1.5,   -- attack speed per point
    },
    intelligence = {
      mana = 18,            -- max mana per point
      mana_regen = 0.15,    -- mana per second per point
      magic_resistance = 0, -- percent magic resistance per point
    },
    primary_damage = 1,     -- attack damage per point of the primary attribute
  }
end
