-- Package schema v3 vocabulary: the single list of every name the v3 validator
-- accepts (fields, units, actions, events, functions, unit properties, stats,
-- states, tags). docs/character_pipeline/schema/v3.md explains the meaning;
-- `lua package_v3.lua --vocab-md` renders this file for the authoring guide.
-- Extending the schema means adding here (and a changelog line in v3.md).
local V = {}

V.SCHEMA_VERSION = 3
V.SCHEMA_REVISION = "3.3"

local function set(list) local s = {} for _, k in ipairs(list) do s[k] = true end return s end
V.set = set

---------------------------------------------------------------------------
-- values
---------------------------------------------------------------------------
-- unit -> zh description. `pct` values read as fractions in expressions (25 -> 0.25).
V.UNITS = {
  s = "秒", pct = "百分比（運算式中為比例）", dmg = "傷害", heal = "治療量", mana = "法力",
  hp = "生命值", dist = "原作距離", speed = "投射物或位移速度（距離／秒）",
  move_speed = "移動速度", atk_speed = "攻擊速度", armor = "護甲", stat = "三圍屬性點數",
  count = "次數或層數（整數）", deg = "角度", gold = "金錢", xp = "經驗", level = "等級",
  per_s = "每秒量", ratio = "倍率（不轉換）", number = "其他無單位數值",
}
V.VALUE_FIELDS = set({ "expr", "unit", "source", "source_differs", "kv", "hidden", "damage_type", "note", "per", "integer", "rounding", "v2_key", "v2_only" })
-- Decimal -> Fixed64 conversion of a value (v3.md §2): nearest (default) or truncate
-- (reproduces a handler that truncated, e.g. B01R percent -> fraction).
V.ROUNDING = set({ "nearest", "truncate" })
-- source prefixes: where the number came from.
V.SOURCE_PREFIXES = {
  kv = "原作 KV 鍵（kv:<Key>，可加 @<path>:<line>）",
  script = "原作腳本（script:<path>:<lines> 說明）",
  text = "只出現在原作說明文字（text:<file>:<line>）",
  rules = "全域規則層（rules:<key>）",
  design = "本專案的設計決定或改編（design:<理由>）",
  assumed = "原作無法判定，暫定值（assumed:<理由>）",
}
V.DAMAGE_TYPES = set({ "physical", "magical", "pure" })

---------------------------------------------------------------------------
-- document / entries
---------------------------------------------------------------------------
V.DOC_KINDS = set({ "hero", "items" })
V.DOC_FIELDS = set({ "schema_version", "kind", "id", "source", "hero", "unit_state", "routines", "unit_templates",
  "abilities", "items", "statuses", "values", "hooks", "notes", "deferred" })
-- entries deliberately not ported (v3.1): { id, source_id, reason, note }
V.DEFERRED_FIELDS = set({ "id", "source_id", "reason", "note" })
V.DEFERRED_REASONS = {
  not_in_source = "原作 KV 沒有定義（或整段被註解），遊戲裡買不到、用不到；還原原作不包括原作不存在的東西",
  old_version = "_old／_20 等非 current 版本（D2）",
  unused = "原作有定義但沒有任何英雄、商店或腳本使用",
}
V.SOURCE_FIELDS = set({ "repository", "commit", "root", "variant", "language" })
V.HERO_FIELDS = set({ "source_id", "text", "attack", "auto_learn", "notes" })
-- rust_handler (M1): the behavior still runs in the v2 Rust handler named by `handler`;
-- the entry carries values and text only. M3 replaces it with data.
-- deferred (v3.1): not ported, with deferred_reason (V.DEFERRED_REASONS)
V.COVERAGE = set({ "expressible", "needs_hook", "pending", "rust_handler", "deferred" })
V.HANDLER_FIELDS = set({ "module", "reads", "notes" })
-- host (v3.3): values a host component reads by name from the catalog while the behavior is data
-- (coverage expressible / needs_hook / pending), e.g. attack_orb extras, ability_projectile caps.
V.HOST_FIELDS = set({ "component", "reads", "notes" })
V.HOST_COMPONENTS = set({ "attack_orb", "ability_projectile", "flame_trail", "delayed_effect", "tether", "damage_zone",
  "reveal_zone", "ally_area_restore", "periodic_damage", "summon" })
V.CONFIDENCE = set({ "high", "medium", "low" })
-- D13 (v1 §11) policy for each recorded original-game quirk.
V.QUIRK_POLICIES = {
  as_source = "照原作實際行為還原（預設）",
  fix_candidate = "明顯違反設計意圖且影響可玩性，列入修正候選清單由使用者決定；目前仍照原作",
  -- D13（使用者 2026-10-29：fix-candidates.md 的 134 項全部修）
  fixed = "原作 bug，依 D13 照設計意圖修正；資料是修正後的行為，text 註明 fix-candidates.md 的編號",
  design_intent = "跟 tick 頻率等引擎副作用有關，照設計意圖（KV 數值）",
  no_effect = "原作執行期報錯或死碼，照中斷後的結果（通常沒有效果）",
  note = "只是紀錄（命名錯誤、呈現細節），不影響結算",
  port_adaptation = "現行移植（v2 Rust handler）與原作不同的改編；資料照現行，M1 決定保留或改回原作",
}
V.QUIRK_FIELDS = set({ "text", "policy", "ref" })
V.PENDING_FIELDS = set({ "what", "why", "gap" })

V.ABILITY_FIELDS = set({ "id", "source_id", "slot", "kind", "tags", "max_level", "learnable", "level_link", "target",
  "cast", "cost", "requires", "flags", "values", "text", "statuses", "events", "passive", "unit_state", "coverage",
  "pending", "provenance", "quirks", "confidence", "notes", "handler", "host", "deferred_reason", "bot", "hook" })
V.SLOTS = set({ "Q", "W", "E", "R", "D", "F", "sub", "hidden" })
V.ABILITY_KINDS = set({ "active", "passive", "toggle", "autocast", "channel" })
V.ABILITY_TAGS = set({ "ultimate", "basic", "toggle", "channel", "item", "sub", "attack_modifier", "orb", "aura",
  "summon", "movement", "no_refresh", "sub_ability", "no_cast_trigger" })
V.TARGET_FIELDS = set({ "type", "range", "aoe_radius", "team", "types", "flags", "min_range", "vector_length" })
V.TARGET_TYPES = set({ "none", "unit", "point", "direction", "vector", "unit_or_point", "tree", "item" })
V.CAST_FIELDS = set({ "cast_point", "backswing", "channel_time", "immediate", "interruptible", "move_while_casting",
  "turn_while_casting", "ignore_backswing" })
V.COST_FIELDS = set({ "mana", "health", "cooldown", "cooldown_start", "charges", "cooldown_groups", "refund", "gold" })
V.COOLDOWN_STARTS = set({ "on_cast", "on_end", "on_toggle_off", "on_channel_end", "manual" })
V.CHARGES_FIELDS = set({ "max", "restore", "initial", "on_rank_up", "scope" })
-- who shares the charge pool: this ability/item instance, the hero, or every item of the same name on the hero
V.CHARGES_SCOPES = set({ "instance", "hero", "item_name" })
V.STOCK_FIELDS = set({ "max", "initial", "restock", "first_restock" })
-- unique_group combination: highest value wins, or the most recently applied instance wins
V.UNIQUE_RULES = set({ "max", "latest" })
-- what a rank up does to stored charges (B01E keeps the count and restarts the timer)
V.CHARGES_RANK_UP = set({ "keep_restart", "keep", "refill" })
V.COOLDOWN_GROUP_FIELDS = set({ "group", "length", "role" })
-- member: starts and receives the group's cooldown; trigger_only: starts it but never receives it;
-- receive_only: receives it but casting does not start it (原作清單打錯名稱的單向共用冷卻)
V.COOLDOWN_GROUP_ROLES = set({ "member", "trigger_only", "receive_only" })
V.REQUIRES_FIELDS = set({ "hero_level", "has_status", "not_status", "condition" })
V.ABILITY_FLAGS = set({ "pierce_magic_immune", "ignore_invulnerable", "can_target_buildings", "can_target_trees",
  "not_on_illusions", "hidden_cooldown", "ignore_cast_range_bonus", "self_cast_allowed", "no_target_self",
  "break_invisibility", "does_not_break_invisibility", "dispellable_cast" })
V.PASSIVE_FIELDS = set({ "stats", "states", "unique_group", "triggers", "aura", "reveal", "visibility", "orb_slot" })
-- Hero-wide orb slot (A21W, nobuorb1): one orb effect at a time across abilities and items.
V.ORB_SLOT_FIELDS = set({ "release", "priority", "claim", "overridable" })
V.ORB_RELEASE = set({ "unequip", "never", "status_end" })

V.ITEM_FIELDS = set({ "id", "source_id", "values", "text", "shop", "passive", "active", "statuses", "unit_state", "hook",
  "coverage", "pending", "provenance", "quirks", "confidence", "notes", "max_level", "handler", "host", "deferred_reason", "tags" })
V.SHOP_FIELDS = set({ "cost", "recipe", "category", "sellable", "droppable", "shareable", "consumable",
  "purchasable", "stock", "unique" })
V.RECIPE_FIELDS = set({ "components", "recipe_cost" })
V.CONSUMABLE_FIELDS = set({ "charges", "remove_when_empty" })
V.ITEM_ACTIVE_FIELDS = set({ "kind", "tags", "target", "cast", "cost", "requires", "flags", "events", "learnable", "bot" })

V.UNIT_TEMPLATE_FIELDS = set({ "id", "source_id", "values", "text", "stats", "statuses", "abilities", "ai",
  "limit_per_owner", "over_limit", "bounty", "attack", "tags", "collision_radius", "blocks_pathing", "flying",
  "vision", "passive", "notes", "provenance", "quirks", "max_level", "acquisition_range", "turn_rate", "owner_death",
  "inventory_locked", "item_cast_blacklist", "chase" })
V.OWNER_DEATH = set({ "persist", "die", "remove" })
V.UNIT_STATS = set({ "hp", "mana", "armor", "magic_resist", "move_speed", "hp_regen", "mana_regen", "str", "agi", "int" })
V.UNIT_ATTACK_FIELDS = set({ "kind", "range", "interval", "damage", "damage_max", "projectile_speed", "damage_type", "attack_point" })
-- create_unit.overrides keys
V.UNIT_OVERRIDES = set({ "hp", "mana", "armor", "magic_resist", "move_speed", "hp_regen", "mana_regen", "str", "agi",
  "int", "attack_damage", "attack_damage_max", "attack_range", "attack_interval", "level", "ability_level",
  "bounty_gold", "bounty_xp", "vision_day", "vision_night", "model_scale" })
-- create_illusion damage tables (outgoing / incoming), percent of the original's damage
V.ILLUSION_DAMAGE_KEYS = set({ "physical", "magical", "pure", "all" })
V.ILLUSION_COPY_FIELDS = set({ "level", "abilities", "except_abilities", "items", "statuses", "hp", "mana", "unit_state",
  "status_duration", "status_caster", "status_tags", "status_except", "team" })
V.ILLUSION_TEAM = set({ "caster", "original" })
V.ILLUSION_STATUS_DURATION = set({ "remaining", "original" })
V.ILLUSION_STATUS_CASTER = set({ "original", "illusion" })
V.ILLUSION_ITEMS = set({ "same", "fresh", "none" })
-- chase_target (v3.2): runs at the unit given by create_unit.ai_target (原作召喚物追擊 AI：A29T、B10W、C16R)
V.UNIT_AI = set({ "none", "follow_owner", "guard", "attack_move", "owner_controlled", "idle", "wander", "chase_target" })
-- teleport_back_distance: farther than this from its current target point, the unit teleports to the anchor
V.CHASE_FIELDS = set({ "speed", "leash", "on_reach", "reach_radius", "retarget", "give_up_after", "max_distance", "anchor",
  "teleport_back_distance" })
-- max_distance is measured from the anchor: the owner (default) or where the unit was created
V.CHASE_ANCHORS = set({ "owner", "spawn" })
V.CHASE_RETARGET = set({ "none", "nearest_enemy", "owner_target", "attacker" })
-- orphan_oldest: the oldest loses its lifetime and its slot but stays (原作下忍超額)
V.OVER_LIMIT = set({ "remove_oldest", "reject", "none", "orphan_oldest" })
V.BOUNTY_FIELDS = set({ "gold", "xp" })
V.UNIT_VISION_FIELDS = set({ "day", "night" })

V.STATE_TYPES = set({ "int", "fixed", "bool", "unit", "point", "vec", "tick", "unit_list", "ability", "item", "item_name" })
V.STATE_FIELDS = set({ "type", "init", "reveal", "scope", "note" })
-- match (v3.2): one variable for the whole match (原作全域變數，例如 B30T 的 tsum)
V.STATE_SCOPES = set({ "hero", "ability", "unit", "item_instance", "item_name", "permanent", "match" })
-- §3.2 visibility policy of each state, var and status.
V.REVEAL = {
  public = "隨單位揭露，雙方 replica 都知道",
  team = "只有己方 replica 知道；不可影響對方 replica 的判定",
  derived = "只揭露結算結果（例如屬性總值）",
}
-- owner (v3.1): the ability whose values ($x) and unit_state the routine reads
V.ROUTINE_FIELDS = set({ "params", "actions", "notes", "owner" })
V.HOOK_FIELDS = set({ "why", "test", "reads", "kind", "params", "notes", "behavior", "cases" })
-- hooks.<name>.cases[]: conformance scenarios of a hook (v3_scenarios.lua), in scenario vocabulary
V.HOOK_CASE_FIELDS = set({ "id", "given", "when", "expect", "notes" })
-- `ability`: the whole ability (or item) is a hand-written trait (m3-rust-codegen-design.md §2.4)
V.HOOK_KINDS = set({ "action", "condition", "value", "trigger", "ability" })

---------------------------------------------------------------------------
-- text
---------------------------------------------------------------------------
V.LANGS = { "zh_TW", "en" }
V.TEXT_FIELDS = set({ "name", "body", "short", "lore", "status_name", "status_body" })
-- {value:fmt}
V.TEXT_FORMATS = {
  all = "每個 rank 用 / 分隔，目前 rank 加粗（預設）",
  current = "只顯示目前 rank",
  signed = "帶正負號",
  abs = "取絕對值",
  int = "四捨五入成整數",
  max = "只顯示最後一個 rank",
}

---------------------------------------------------------------------------
-- status
---------------------------------------------------------------------------
V.STATUS_FIELDS = set({ "id", "duration", "stacking", "dispel", "visibility", "status_resistance", "scope",
  "reveal", "stats", "states", "vars", "absorb", "charges", "immune_to", "block_reapply", "unique_group",
  "aura", "triggers", "cue", "text", "tags", "forced_move", "notes", "values", "linger", "remove_on", "per_source",
  "reapply_if_removed", "unique_rule", "runtime_id" })
V.STACKING_MODES = set({ "refresh", "extend", "independent", "keep", "replace" })
V.STACK_FIELDS = set({ "max", "mode", "value_stacks", "per_stack_duration", "decay", "initial" })
V.STACK_MODES = set({ "refresh_on_add", "independent_timers", "no_refresh" })
V.DISPEL = set({ "none", "basic", "strong", "death_only" })
V.VISIBILITY = set({ "buff", "debuff", "hidden" })
V.STATUS_SCOPES = set({ "unit", "hero", "item_instance", "permanent" })
V.STAT_OPS = set({ "add", "pct", "set_absolute", "min", "max", "mul" })
V.STAT_ENTRY_FIELDS = set({ "op", "value", "when", "per_stack" })
V.ABSORB_FIELDS = set({ "amount", "types", "on_absorb", "on_broken", "lethal_only", "fraction", "phase" })
V.AURA_FIELDS = set({ "radius", "team", "types", "flags", "status", "linger", "include_self", "where", "unique_group" })
V.FORCED_MOVE_FIELDS = set({ "toward", "locks", "speed" })
V.ORDER_LOCKS = set({ "move", "attack", "cast", "item", "stop", "hold", "all" })
V.LANDMARKS = set({ "base", "fort", "fountain", "barracks", "tower", "shop", "spawn" })
V.REMOVE_ON = set({ "death", "owner_death", "unequip", "channel_end", "attack", "cast", "move" })

-- StatKey names (status / passive `stats`). Extends package_v2's list with the
-- nobu census needs; M1/M3 maps each to a runtime stat or rules-layer channel.
V.STATS = set({
  "str", "agi", "int", "all_stats", "primary_stat",
  "attack_damage", "attack_damage_pct", "base_attack_damage", "attack_speed", "base_attack_time", "attack_range",
  "move_speed", "move_speed_pct", "move_speed_min", "slow_pct", "slow_resistance_pct",
  "armor", "magic_resist", "evasion_pct", "hp_regen", "hp_regen_pct", "mana_regen", "mana_regen_pct",
  "max_hp", "max_mana", "lifesteal_pct", "spell_lifesteal_pct", "spell_amp_pct",
  "incoming_damage_pct", "incoming_physical_pct", "incoming_magical_pct", "outgoing_damage_pct",
  "cooldown_reduction_pct", "mana_cost_reduction_pct", "status_resistance_pct",
  "heal_amp_pct", "heal_reduction_pct", "vision_day", "vision_night", "cast_range",
  "physical_block", "physical_block_chance", "turn_rate", "bonus_gold_pct", "bonus_xp_pct", "base_attack_damage_pct", "cast_point_reduction_pct", "magic_resist_pool", "base_move_speed", "miss_chance_pct",
  "hp_regen_amp_pct", "mana_regen_amp_pct",
  "true_strike_chance", "bash_chance", "invisibility_level", "detection_level", "model_scale",
})
V.STATES = set({
  "stun", "root", "silence", "disarm", "mute", "hex", "frozen", "command_restricted", "sleep", "taunt", "fear",
  "invulnerable", "magic_immune", "attack_immune", "out_of_game", "untargetable_by_spells",
  "invisible", "unselectable", "no_healthbar", "not_on_minimap", "no_unit_collision", "flying",
  "cannot_miss", "evade_disabled", "passives_disabled", "blind", "low_attack_priority", "provides_vision",
  "true_sight", "revealed", "cannot_be_magic_immune", "physical_immune", "melee_attack", "ranged_attack",
  "flying_for_pathing", "block_disabled", "attack_slow_immune", "hidden", "break", "ignore_collision_trees", "cannot_be_healed", "no_attack", "no_cast",
})
V.UNIT_TYPES = set({ "hero", "creep", "building", "summon", "ward", "neutral", "boss", "illusion", "tree",
  "item_on_ground", "courier", "tower", "base", "fountain", "dummy", "all", "general" })
V.TEAMS = set({ "enemy", "ally", "both", "self", "any", "ally_not_self" })
-- Default (Dota FindUnitsInRadius without flags): invisible and fogged units are included;
-- magic-immune, invulnerable, out-of-game, untargetable and dead units are excluded.
V.SELECT_FLAGS = set({ "visible_only", "exclude_invisible", "include_magic_immune", "include_invulnerable", "exclude_self",
  "include_dead", "include_out_of_game", "include_untargetable", "include_illusions", "exclude_illusions",
  "include_hidden", "heroes_first", "ignore_fog" })

---------------------------------------------------------------------------
-- events (ability, item, status trigger) and their `event.*` parameters
---------------------------------------------------------------------------
-- param kind: unit / point / num / bool / attack / damage / ability / item / order / enum
local attack_params = { attacker = "unit", target = "unit", attack = "attack" }
local damage_params = { attacker = "unit", victim = "unit", damage = "damage", ability = "ability", item = "item" }
V.ABILITY_EVENTS = {
  on_phase_start = {}, on_cast = {}, on_phase_cancel = {},
  on_channel_tick = { tick = "num" }, on_channel_end = { interrupted = "bool" },
  on_toggle_on = {}, on_toggle_off = {}, on_learn = {}, on_upgrade = {},
  on_order_targeted = { order = "order", issuer = "unit" },
  on_equip = {}, on_unequip = {}, on_charge_restored = {}, on_owner_death = {}, on_owner_respawn = {},
}
V.STATUS_EVENTS = {
  on_created = {}, on_refreshed = {}, on_expired = {}, on_removed = { reason = "enum" }, periodic = {},
  on_stack_changed = { delta = "num" },
  on_attack_windup = attack_params, on_attack_launch = attack_params, on_attack_hit = attack_params,
  on_attack_miss = attack_params, on_attack_cancel = attack_params, on_attacked = attack_params,
  on_attacked_hit = attack_params,
  on_deal_damage_pre = damage_params, on_deal_damage = damage_params,
  on_take_damage_pre = damage_params, on_take_damage = damage_params,
  on_kill = { victim = "unit", ability = "ability" }, on_death = { killer = "unit" },
  on_owner_death = { killer = "unit" }, on_respawn = {},
  -- abilities and items alike (Dota OnAbilityExecuted); fires before the caster's own on_cast
  on_ability_executed = { unit = "unit", ability = "ability", target = "unit", point = "point" },
  on_item_used = { unit = "unit", item = "item" },
  on_order = { order = "order" }, on_order_targeted = { order = "order", issuer = "unit" },
  on_moved = { distance = "num" }, on_heal_received = { amount = "num", healer = "unit" },
  on_mana_spent = { amount = "num", ability = "ability" },
  on_aura_enter = { unit = "unit" }, on_aura_leave = { unit = "unit" },
  on_level_up = { level = "num" }, on_gold_changed = { delta = "num" },
}
V.TRIGGER_FIELDS = set({ "actions", "filter", "interval", "first", "who", "notes", "priority", "phase" })
V.TRIGGER_WHO = set({ "self", "any_enemy", "any_ally", "any", "owner" })
-- event.<kind>.<field>
V.EVENT_RECORDS = {
  attack = { crit = "num", damage = "num", is_orb = "bool", triggers_effects = "bool", effect_multiplier = "num",
    is_ranged = "bool", from_ability = "bool", missed = "bool", target = "unit", attacker = "unit" },
  damage = { raw = "num", after_armor = "num", final = "num", type = "enum", is_attack = "bool",
    is_physical = "bool", is_magical = "bool", is_pure = "bool",
    is_reflection = "bool", is_ability = "bool", source_kind = "enum", lethal = "bool", source = "unit", credit = "unit" },
  order = { kind = "enum", target = "unit", point = "point", ability = "ability", by_player = "bool" },
}

---------------------------------------------------------------------------
-- expressions
---------------------------------------------------------------------------
-- Unit properties readable as <unit>.<prop>. pct-like ones are fractions.
V.UNIT_PROPS = {
  str = "num", agi = "num", int = "num", primary = "num", base_str = "num", base_agi = "num", base_int = "num",
  attack = "num", attack_base = "num", attack_bonus = "num", attack_speed = "num", attack_interval = "num",
  attack_range = "num", max_hp = "num", hp = "num", missing_hp = "num", hp_pct = "num", mana = "num",
  max_mana = "num", missing_mana = "num", mana_pct = "num", armor = "num", base_armor = "num",
  magic_resist = "num", base_magic_resist = "num", level = "num", move_speed = "num", base_move_speed = "num",
  gold = "num", position = "point", base_attack_range = "num", bonus_armor = "num",
  projectile_speed = "num", attack_speed_mult = "num",
  -- rules-layer illusion damage multiplier of this unit (1 for a real unit)
  illusion_damage = "num", forward = "vec", team = "team", alive = "bool", is_hero = "bool",
  is_building = "bool", is_illusion = "bool", is_summon = "bool", is_creep = "bool", is_channeling = "bool",
  is_magic_immune = "bool", is_invulnerable = "bool", is_invisible = "bool", is_ranged = "bool",
  is_stunned = "bool", is_silenced = "bool", owner = "unit", hero = "unit", collision_radius = "num",
  kills = "num", deaths = "num", attack_target = "unit", is_moving = "bool", spell_amp = "num",
  status_resistance = "num", evasion = "num", is_attacking = "bool", is_casting = "bool",
}
V.ABILITY_PROPS = { level = "num", max_level = "num", cooldown_remaining = "num", charges = "num",
  cast_point = "num", mana_cost = "num", cooldown = "num", is_item = "bool", is_toggle = "bool", is_ultimate = "bool",
  is_passive = "bool", slot_index = "num",
  is_toggled = "bool", is_channeling = "bool", cast_range = "num", is_ready = "bool", cast_count = "num",
  is_learned = "bool", owner = "unit" }
V.ITEM_PROPS = { charges = "num", owner = "unit", purchaser = "unit", cooldown_remaining = "num", level = "num",
  slot = "num", cost = "num", sellable = "bool", alive = "bool", name = "item_name" }
V.STATUS_PROPS = { stacks = "num", display_stacks = "num", remaining = "num", elapsed = "num", duration = "num",
  source = "unit", holder = "unit" }
V.POINT_PROPS = { x = "num", y = "num" }
V.GAME_PROPS = { time = "num", is_night = "bool", score_radiant = "num", score_dire = "num", tick = "num",
  dt = "num", creep_wave = "num", creep_armor_bonus = "num" }
V.PROJECTILE_PROPS = { position = "point", traveled = "num", path_length = "num", source = "unit", alive = "bool" }

-- Function signatures: args (kinds) -> result. Arg kinds: num bool unit point vec any
-- status (declared status id), ability (declared ability id or ability expr), template
-- (unit template id), tag (V.UNIT_TYPES/ABILITY_TAGS), state (state var name),
-- team (V.TEAMS or team expr). `...` = variadic of the previous kind.
V.FUNCTIONS = {
  min = { args = { "num", "num", "..." }, ret = "num" },
  max = { args = { "num", "num", "..." }, ret = "num" },
  clamp = { args = { "num", "num", "num" }, ret = "num" },
  abs = { args = { "num" }, ret = "num" },
  floor = { args = { "num" }, ret = "num" },
  ceil = { args = { "num" }, ret = "num" },
  round = { args = { "num" }, ret = "num" },
  sqrt = { args = { "num" }, ret = "num" },
  sin = { args = { "num" }, ret = "num" }, cos = { args = { "num" }, ret = "num" },
  atan2 = { args = { "num", "num" }, ret = "num" },
  select = { args = { "bool", "any", "any" }, ret = "same" },
  random = { args = { "num", "num" }, ret = "num" },
  random_int = { args = { "num", "num" }, ret = "num" },
  -- geometry
  distance = { args = { "any", "any" }, ret = "num" },
  edge_distance = { args = { "unit", "unit" }, ret = "num" },
  direction = { args = { "any", "any" }, ret = "vec" },
  angle_between = { args = { "vec", "vec" }, ret = "num" },
  vec = { args = { "num", "num" }, ret = "vec" },
  rotate = { args = { "vec", "num" }, ret = "vec" },
  normalize = { args = { "vec" }, ret = "vec" },
  offset = { args = { "any", "vec", "num" }, ret = "point" },
  polar = { args = { "any", "num", "num" }, ret = "point" },
  random_point = { args = { "any", "num", "num" }, ret = "point" },
  circle_point = { args = { "any", "num", "num", "num" }, ret = "point" },
  lerp_point = { args = { "any", "any", "num" }, ret = "point" },
  position = { args = { "any" }, ret = "point" },
  in_circle = { args = { "any", "any", "num" }, ret = "bool" },
  in_cone = { args = { "any", "any", "vec", "num", "num" }, ret = "bool" },
  -- units and statuses
  alive = { args = { "any" }, ret = "bool" },
  exists = { args = { "any" }, ret = "bool" },
  has_status = { args = { "unit", "status" }, ret = "bool" },
  has_status_tag = { args = { "unit", "status_tag" }, ret = "bool" },
  -- the ability in a hero's slot (Q W E R D F), for ability theft (C08T)
  ability_in_slot = { args = { "unit", "slot" }, ret = "ability" },
  order_is = { args = { "any", "order_kind" }, ret = "bool" },
  -- uniform over the ring's area (random_point is uniform in radius, like Dota RandomVector(RandomInt))
  random_point_area = { args = { "any", "num", "num" }, ret = "point" },
  is_ability = { args = { "any", "name" }, ret = "bool" },
  is_item = { args = { "any", "item_name" }, ret = "bool" },
  is_illusion_of = { args = { "unit", "unit" }, ret = "bool" },
  is_hero_id = { args = { "unit", "name" }, ret = "bool" },
  primary_is = { args = { "unit", "name" }, ret = "bool" },
  stacks = { args = { "unit", "status" }, ret = "num" },
  status_remaining = { args = { "unit", "status" }, ret = "num" },
  status_var = { args = { "unit", "status", "name" }, ret = "any" },
  status_source = { args = { "unit", "status" }, ret = "unit" },
  has_tag = { args = { "any", "tag" }, ret = "bool" },
  has_state = { args = { "unit", "state_flag" }, ret = "bool" },
  is_enemy = { args = { "unit", "unit" }, ret = "bool" },
  is_ally = { args = { "unit", "unit" }, ret = "bool" },
  is_template = { args = { "unit", "template" }, ret = "bool" },
  is_visible_to = { args = { "any", "unit" }, ret = "bool" },
  count_in_radius = { args = { "any", "num", "team", "tag" }, ret = "num" },
  ability_of = { args = { "unit", "ability" }, ret = "ability" },
  ability_level = { args = { "unit", "ability" }, ret = "num" },
  ability_ready = { args = { "unit", "ability" }, ret = "bool" },
  has_item = { args = { "unit", "item_name" }, ret = "bool" },
  item_count = { args = { "unit", "item_name" }, ret = "num" },
  owner_of = { args = { "any" }, ret = "unit" },
  landmark = { args = { "team", "landmark" }, ret = "unit" },
  team_of = { args = { "unit" }, ret = "team" },
  enemy_team = { args = { "unit" }, ret = "team" },
  avg_attack_against = { args = { "unit", "unit" }, ret = "num" },
  armor_multiplier = { args = { "num" }, ret = "num" },
  units_owned = { args = { "unit", "template" }, ret = "num" },
  hook_value = { args = { "name", "any", "..." }, ret = "any" },
  -- trees
  tree_alive = { args = { "any" }, ret = "bool" },
  -- orbs: true when `ability` may use the hero-wide orb slot (free, or held by it)
  orb_slot_free = { args = { "unit", "ability" }, ret = "bool" },
}
V.FUNCTION_ENUM_ARGS = { tag = true, name = true, team = true, item_name = true, state_flag = true, landmark = true,
  status_tag = true, slot = true, order_kind = true }
V.ORDER_KINDS = set({ "move", "attack", "attack_move", "stop", "hold", "cast", "cast_target", "cast_point", "cast_no_target",
  "use_item", "pick_up", "drop_item", "follow", "patrol" })
-- Roots readable in every expression; context adds event/self/target/... (v3.md §5.4).
-- `none` (v3.1): the empty reference, for unit / item / ability variables
V.ROOTS_ALWAYS = { caster = "unit", caster_hero = "unit", ability = "ability", game = "game", rules = "rules",
  state = "state", none = "any" }
-- Numeric literals behavior/expressions may use without naming (structural constants).
V.STRUCTURAL_NUMBERS = { [0] = true, [1] = true, [-1] = true }

---------------------------------------------------------------------------
-- selectors
---------------------------------------------------------------------------
V.SELECTOR_FIELDS = set({ "from", "shape", "radius", "inner", "outer", "angle", "length", "width", "to",
  "team", "types", "exclude_types", "flags", "where", "order", "prefer", "limit", "post_filter", "visible_to",
  "exclude", "relative_to", "notes", "take_while" })
V.SHAPES = {
  circle = { "radius" }, ring = { "inner", "outer" }, cone = { "angle", "length" }, line = { "length", "width" },
  rect = { "length", "width" }, global = {}, unit = {}, segment = { "to", "width" },
}
V.ORDERS = set({ "id", "nearest", "farthest", "random", "lowest_hp", "highest_hp", "lowest_hp_pct",
  "highest_hp_pct", "highest_attack", "most_recent" })

---------------------------------------------------------------------------
-- actions
---------------------------------------------------------------------------
-- Param kinds: expr (number expression), cond (bool expression), unit / point / vec / any
-- (expressions of that kind), actions (nested action list), selector, status, ability,
-- template, routine, hook, item_name (string), cue (free presentation id), name (binding),
-- bool (Lua boolean), enum:<a|b|c>, list:<kind>, table (free-form named expressions), text.
-- `!` suffix = required. `as` binds the action's result for later actions.
local DMG = "enum:physical|magical|pure|event"
V.ACTIONS = {
  -- numeric
  damage = { to = "unit!", amount = "expr!", type = DMG .. "!", source = "unit", credit = "unit",
    require_source_alive = "bool", flags = "list:enum:no_amplify|no_reflect|hp_removal|non_lethal|reflection|no_spell_lifesteal|no_lifesteal|is_attack|ignore_armor|bypass_block|no_crit|no_source_bonuses",
    ability = "ability", as = "name" },
  heal = { to = "unit!", amount = "expr!", source = "unit", flags = "list:enum:ignore_heal_amp|ignore_heal_reduction|no_overheal|silent", as = "name" },
  restore_mana = { to = "unit!", amount = "expr!", source = "unit", as = "name" },
  burn_mana = { to = "unit!", amount = "expr!", source = "unit", as = "name" },
  -- the unit pays mana itself (fires on_mana_spent), unlike burn_mana
  spend_mana = { from = "unit!", amount = "expr!", ability = "ability" },
  set_health = { to = "unit!", value = "expr!" },
  set_mana = { to = "unit!", value = "expr!" },
  -- statuses
  apply_status = { status = "status!", to = "unit!", duration = "expr", stacks = "expr", source = "unit",
    vars = "table", as = "name" },
  remove_status = { from = "unit!", status = "status", dispel = "enum:basic|strong|all_debuffs|all_buffs|all",
    team_of_source = "enum:enemy|ally|any", tags = "list:name", min_remaining = "expr", limit = "expr",
    except = "list:status", exclude_tags = "list:name", min_duration = "expr", states = "list:state_flag",
    only = "enum:debuffs|buffs", exclude_source_tags = "list:name", source_ability = "name",
    -- every listed condition must hold for a status to be removed (v3.2)
    source_is = "list:enum:has_ability|no_ability|alive|gone" },
  add_stacks = { to = "unit!", status = "status!", count = "expr!", max = "expr", refresh = "bool" },
  set_stacks = { to = "unit!", status = "status!", count = "expr!" },
  set_display_stacks = { to = "unit!", status = "status!", count = "expr!" },
  set_var = { var = "name!", value = "any!", on = "unit", status = "status" },
  set_state = { var = "name!", value = "any!", on = "unit", item = "any" },
  clear_state = { var = "name!", on = "unit", item = "any" },
  temporary_night = { duration = "expr!" },
  -- delivery
  projectile = { kind = "enum:linear|tracking|path!", from = "any!", to = "any", direction = "vec", speed = "expr!",
    radius = "expr", end_radius = "expr", distance = "expr", pierce = "bool", dodgeable = "bool",
    hit_buildings = "bool", vision = "expr", team = "enum:enemy|ally|both|any", types = "list:enum:" ..
    "hero|creep|building|summon|ward|neutral|boss|illusion|all", flags = "list:flag", where = "cond",
    path = "list:any", max_hits = "expr", on_hit = "actions", on_end = "actions", on_tick = "actions",
    tick_interval = "expr", cue = "cue", as = "name", bind = "name",
    hit_priority = "list:enum:enemy|ally", lifetime = "expr", arm_distance = "expr", remove_on = "list:enum:owner_death|status_end" },
  zone = { at = "any!", shape = "enum:circle|ring|cone|line|rect!", radius = "expr", inner = "expr", outer = "expr",
    length = "expr", width = "expr", angle = "expr", direction = "vec", duration = "expr!", interval = "expr",
    first = "expr", follow = "unit", team = "enum:enemy|ally|both|any",
    types = "list:enum:hero|creep|building|summon|ward|neutral|boss|illusion|all", flags = "list:flag",
    on_enter = "actions", on_leave = "actions", on_tick = "actions", on_end = "actions", cue = "cue", as = "name",
    blocks_pathing = "bool", vision = "expr" },
  tether = { from = "unit!", to = "unit!", max_distance = "expr", duration = "expr!", on_break = "actions",
    on_expire = "actions", on_tick = "actions", interval = "expr", cue = "cue", as = "name" },
  dodge_projectiles = { of = "unit!", from_team = "enum:enemy|ally|any", kind = "enum:tracking|all" },
  destroy_projectile = { projectile = "any!" },
  -- units
  create_unit = { template = "template!", at = "any!", count = "expr", owner = "unit", team = "unit", lifetime = "expr",
    ai_target = "unit",
    control = "enum:none|owner|ai", overrides = "unit_overrides", copy_of = "unit", facing = "vec", as = "name",
    limit_per_owner = "expr", over_limit = "enum:remove_oldest|reject|none|orphan_oldest" },
  create_illusion = { of = "unit!", at = "any", count = "expr", duration = "expr!", outgoing = "illusion_damage",
    incoming = "illusion_damage", copy = "illusion_copy", control = "enum:none|owner|ai", as = "name",
    triggers_effects = "bool" },
  modify_base_stat = { to = "unit!", stat = "enum:str|agi|int!", amount = "expr!" },
  kill = { to = "unit!", killer = "unit", silent = "bool" },
  remove_unit = { to = "unit!" },
  set_owner = { unit = "unit!", owner = "unit!" },
  -- motion
  teleport = { who = "unit!", to = "any!", keep_orders = "bool", find_clear_space = "bool", max_distance = "expr" },
  dash = { who = "unit!", to = "any", direction = "vec", speed = "expr!", distance = "expr", height = "expr",
    stop_at_target = "bool", on_end = "actions", on_tick = "actions", hit_radius = "expr", on_hit = "actions",
    interruptible = "bool", as = "name" },
  leap = { who = "unit!", to = "any!", duration = "expr!", height = "expr", on_end = "actions" },
  -- B24W: change an ongoing knockback / dash / pull of `who`
  redirect_motion = { who = "unit!", toward = "any", direction = "vec", speed = "expr", distance = "expr" },
  -- B25T: a stackable velocity impulse that decays
  impulse = { who = "unit!", direction = "vec!", speed = "expr!", decay = "expr!", duration = "expr", as = "name" },
  -- B28T: a bouncing chain; each chain has its own hit set
  chain = { from = "any!", first = "unit!", jumps = "expr!", radius = "expr!", interval = "expr", team = "enum:enemy|ally|both|any!",
    types = "list:enum:hero|creep|building|summon|ward|neutral|boss|illusion|all", flags = "list:flag", where = "cond",
    take_while = "cond", allow_repeat = "bool", order = "enum:nearest|random|id", actions = "actions!", projectile_speed = "expr",
    cue = "cue", as = "name" },
  set_base_max_hp = { to = "unit!", value = "expr!" },
  set_time_of_day = { mode = "enum:day|night!", duration = "expr" },
  clear_slow_pool = { of = "unit!" },
  knockback = { who = "unit!", from = "any!", distance = "expr!", duration = "expr!", height = "expr",
    on_end = "actions", stun = "bool", stop_on_contact = "bool", contact_radius = "expr" },
  pull = { who = "unit!", toward = "any", along = "any", speed = "expr!", stop_distance = "expr", max_duration = "expr",
    on_end = "actions" },
  swap_positions = { a = "unit!", b = "unit!" },
  face = { who = "unit!", toward = "any!" },
  -- vision
  reveal = { at = "any!", radius = "expr!", duration = "expr!", teams = "enum:own|enemy|all!", follow = "unit",
    true_sight = "bool", ignore_terrain = "bool", as = "name" },
  -- abilities
  modify_cooldown = { ability = "ability", of = "unit", mode = "enum:set|reduce|reset|end|start!", amount = "expr",
    item_tag = "name", item = "any" },
  modify_charges = { ability = "ability!", of = "unit", mode = "enum:add|set|refill!", amount = "expr" },
  add_ability = { to = "unit!", ability = "ability!", level = "expr", slot = "enum:Q|W|E|R|D|F|sub|hidden",
    cast_point = "expr", as = "name" },
  remove_ability = { from = "unit!", ability = "ability!" },
  swap_abilities = { on = "unit!", from = "ability!", to = "ability!", duration = "expr",
    cooldown_policy = "enum:freeze|continue|reset!" },
  set_ability_level = { of = "unit!", ability = "ability!", level = "expr!" },
  set_ability_active = { of = "unit!", ability = "ability!", active = "bool!" },
  toggle_ability = { of = "unit!", ability = "ability!", on = "bool!" },
  refund = { mana = "bool", cooldown = "bool", health = "bool" },
  cancel_cast = { reason = "text" },
  end_channel = { interrupted = "bool" },
  -- attacks
  attack = { attacker = "unit!", target = "unit!", use_orb = "bool", trigger_effects = "bool", fake = "bool",
    effect_multiplier = "expr", ignore_range = "bool", instant = "bool", cannot_miss = "bool", as = "name" },
  modify_attack = { crit_multiplier = "expr", bonus_damage = "expr", bonus_damage_type = DMG,
    cannot_miss = "bool", no_effects = "bool", damage_multiplier = "expr", is_orb = "bool", orb_priority = "expr" },
  -- damage-event modification (on_*_damage_pre)
  modify_damage = { multiplier = "expr", add = "expr", set = "expr", block = "expr", cancel = "bool" },
  -- control flow
  ["if"] = { cond = "cond!", actions = "actions!", otherwise = "actions" },
  for_each = { select = "selector!", as = "name", actions = "actions!" },
  ["repeat"] = { count = "expr", interval = "expr", first = "expr", ["while"] = "cond", stop_when = "cond",
    as = "name", actions = "actions!", id = "name" },
  delay = { time = "expr!", id = "name", capture = "list:name", actions = "actions!" },
  cancel_delay = { id = "name!" },
  chance = { p = "expr!", pity_after = "expr", counter = "enum:ability|hero|item_name|status", counter_var = "name", mode = "enum:independent|prd",
    actions = "actions!",
    otherwise = "actions" },
  random_pick = { options = "options!", as = "name" },
  let = { name = "name!", value = "any!" },
  call = { routine = "routine!", args = "table" },
  hook = { name = "hook!", args = "table", as = "name", callbacks = "callbacks" },
  stop = {},
  -- economy / items / trees
  give_gold = { to = "unit!", amount = "expr!", reliable = "bool", show = "bool" },
  take_gold = { from = "unit!", amount = "expr!", as = "name" },
  give_xp = { to = "unit!", amount = "expr!" },
  give_item = { to = "unit!", item = "item_ref!", charges = "expr", vars = "table", as = "name" },
  pick_item = { from = "unit!", where = "cond", order = "enum:random|first|last", slots = "enum:inventory|backpack|all", as = "name!" },
  remove_item = { from = "unit!", item = "any!" },
  swap_item = { a = "any!", b = "any!" },
  drop_item = { from = "unit!", item = "any!", at = "any" },
  set_item_charges = { item = "any!", count = "expr!" },
  create_pickup = { at = "any!", lifetime = "expr", on_pickup = "actions!", who = "enum:any|hero|ally|enemy", cue = "cue", as = "name",
    killable = "bool", on_destroyed = "actions" },
  destroy_trees = { at = "any!", radius = "expr!" },
  destroy_tree = { tree = "any!" },
  regrow_trees = { at = "any", radius = "expr" },
  create_tree = { at = "any!", lifetime = "expr", as = "name" },
  -- orders
  issue_order = { who = "unit!", order = "enum:move|attack|attack_move|stop|hold|cast|follow!", target = "unit",
    at = "any", ability = "ability", force = "bool", queue = "bool" },
  -- presentation (never affects gameplay)
  cue = { id = "cue!", at = "any", attach = "unit", to = "any", duration = "expr", radius = "expr", follow = "bool",
    team = "enum:own|enemy|all", stop = "bool" },
}
V.CONTROL_FLOW = set({ "if", "for_each", "repeat", "delay", "chance", "random_pick" })
V.OPTION_FIELDS = set({ "weight", "actions", "value" })

-- Rules-layer keys (v1 §1) readable as rules.<a>.<b> that game_rules.lua does not
-- carry yet; M2 adds them there (with the original-game source) and removes them here.
V.RULES_PENDING = {
  damage = { require_source_alive = 1 },
}

return V
