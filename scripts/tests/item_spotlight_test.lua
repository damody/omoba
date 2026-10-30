ITEM_SPOTLIGHT_AS_MODULE = true
package.path = "scripts/?.lua;" .. package.path
local M = require("item_spotlight")
local checks = 0
local function ok(cond, name) assert(cond, name); checks = checks + 1 end

local catalog = { items = {
  { id = "item_bomb", v3 = { bot = { kind = "enemy_point" }, text = { zh_TW = { name = "炸彈", rendered = "使用：投擲炸彈。" } } } },
  { id = "item_drink", v3 = { bot = { kind = "self_engage" }, text = { zh_TW = { name = "酒", rendered = "使用：加速。" } } } },
  { id = "item_heal", v3 = { bot = { kind = "ally_heal" }, text = { en = { name = "Heal" } } } },
  { id = "item_ring", v3 = { text = { zh_TW = { name = "戒指" } } } },
} }
local slots = M.slots(catalog, { "item_bomb", "item_drink", "item_heal" })
ok(#slots == 3 and slots[1].item_slot == 0 and slots[3].item_slot == 2, "one inventory slot per item, in order")
ok(slots[1].kind == "item" and slots[1].team == "enemy" and slots[1].target == "point", "a thrown item aims at the enemy's point")
ok(slots[2].team == "self" and slots[2].target == "none", "a self item has no target")
ok(slots[3].team == "ally" and slots[3].target == "unit" and slots[3].name == "Heal", "an ally item targets the ally; English name fallback")
ok(not pcall(M.slots, catalog, { "item_ring" }), "a passive item has nothing to press")
ok(not pcall(M.slots, catalog, { "item_nothing" }), "an unknown item is refused")
ok(not pcall(M.slots, catalog, {}), "at least one item")
ok(M.registry_id("item_napalm_bomb") == "napalm_bomb", "the authority's id drops the catalog prefix")

-- Combined items start from their components (the shop refuses a recipe without them).
local crafted = { items = {
  { id = "item_cloth" }, { id = "item_ingot" }, { id = "item_orb" },
  { id = "item_boots", recipe = { components = { "item_cloth", "item_ingot" } }, v3 = { bot = { kind = "self_engage" }, text = { en = { name = "Boots" } } } },
  { id = "item_boots_1", recipe = { components = { "item_orb", "item_boots" } }, v3 = { bot = { kind = "self_engage" }, text = { en = { name = "Boots+" } } } },
  { id = "item_bomb", v3 = { bot = { kind = "enemy_point" }, text = { en = { name = "Bomb" } } } },
} }
local start, slot_of = M.placement(crafted, { "item_bomb", "item_boots_1" })
ok(table.concat(start, ",") == "item_bomb,item_orb,item_cloth,item_ingot,item_boots,item_boots_1", "components first, depth first")
ok(slot_of.item_bomb == 0 and slot_of.item_boots_1 == 1 and slot_of.item_boots == nil, "the combined item takes the first slot its components freed")
local crafted_slots = M.slots(crafted, { "item_bomb", "item_boots_1" })
ok(crafted_slots[2].item_slot == 1 and crafted_slots[2].name == "Boots+", "the script presses the slot the item ends in")
local both = M.slots(crafted, { "item_boots", "item_boots_1" })
ok(both[1].item_slot ~= both[2].item_slot, "a shown item that is also a component is built again for the recipe")

local stage = { players = { { player_id = 1, hero = "saika_magoichi" }, { player_id = 2, hero = "x" } }, showcase = { start_level = 16 } }
local text = M.recipe(stage, "oichi", { "item_bomb", "item_drink" })
local recipe = assert(load(text))()
ok(recipe.players[1].hero == "oichi" and recipe.players[2].hero == "x", "the demonstrator plays the chosen hero")
ok(recipe.showcase.start_items["1"][1] == "bomb" and recipe.showcase.start_items["1"][2] == "drink", "start items by authority id")
ok(recipe.showcase.start_level == 16, "the rest of the stage is kept")
print("item_spotlight_test: " .. checks .. " checks passed")
