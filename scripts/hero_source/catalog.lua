-- Builds the full provenance catalog (plan D1/D2) from a read-only checkout of
-- the hero source repository. Source files are only read as text: no script
-- from the source (read_hero.lua, langtext.lua, vscripts) is ever executed.
local kv = require("hero_source.kv")
local sha256 = require("hero_source.sha256")
local classify = require("hero_source.classify")
local lfs = require("lfs")

local M = {}

M.SCHEMA = "omoba.hero_source_catalog/1"
M.FACTIONS = { { dir = "A_Oda", prefix = "A" }, { dir = "B_Unified", prefix = "B" }, { dir = "C_Neutral", prefix = "C" } }
-- history_nobu layout (E169): the Dota addon lives under ADDON; every catalog path is relative
-- to the checkout root, so verify re-reads the same files. Tooltips are UTF-16LE (hero_source.text).
M.ADDON = "game/dota_addons/nobu"
M.NPC_DIR = M.ADDON .. "/scripts/npc"
M.VSCRIPTS_DIR = M.ADDON .. "/scripts/vscripts"
M.TOOLTIP_FILE = M.ADDON .. "/resource/addon_tchinese.txt"
-- Hero names: the `_G.hero_name_zh = { A01 = "...", ... }` table, read as text, never executed.
M.NAME_FILE = M.VSCRIPTS_DIR .. "/addon_game_mode.lua"
-- Selectable heroes (by their override_hero class) and the hero unit defaults.
M.HEROLIST_FILE = M.NPC_DIR .. "/herolist.txt"
M.HERO_DEFAULTS_FILE = M.NPC_DIR .. "/npc_heroes_override.txt"
local text = require("hero_source.text")
local SLOTS = { W = true, E = true, R = true, T = true, D = true, F = true }

local function read_file(file)
  return text.read(file)
end

local function sorted_dir(dir)
  local out = {}
  for name in lfs.dir(dir) do
    if name ~= "." and name ~= ".." then out[#out + 1] = name end
  end
  table.sort(out)
  return out
end

-- Resolves the checked-out commit from .git without running git.
function M.checkout_commit(source)
  local head_file = source .. "/.git/HEAD"
  local f = io.open(head_file, "rb")
  if not f then return nil end
  local head = f:read("a"):gsub("%s+$", "")
  f:close()
  local ref = head:match("^ref: (.+)$")
  if not ref then return head end
  local rf = io.open(source .. "/.git/" .. ref, "rb")
  if rf then local v = rf:read("a"):gsub("%s+$", ""); rf:close(); return v end
  local packed = io.open(source .. "/.git/packed-refs", "rb")
  if packed then
    for line in packed:lines() do
      local sha, name = line:match("^(%x+) (.+)$")
      if name == ref then packed:close(); return sha end
    end
    packed:close()
  end
  return nil
end

-- Ability ID grammar: <hero><core>[_20][_old]; see summary.md for meaning.
function M.parse_ability_id(id)
  local hero, rest = id:match("^([ABC]%d%d)(.*)$")
  if not hero then return { kind = "foreign" } end
  local info = { hero = hero }
  local core = rest
  if core:match("_old$") then info.old = true; core = core:gsub("_old$", "") end
  if core:find("_20", 1, true) then info.v20 = true; core = core:gsub("_20", "", 1) end
  info.core = core
  if core == "_precache" then info.kind = "precache"
  elseif #core == 1 and SLOTS[core] then info.kind = "skill"; info.slot = core
  elseif core ~= "" and SLOTS[core:sub(1, 1)] then info.kind = "sub_ability"; info.slot = core:sub(1, 1)
  elseif core == "" then info.kind = "bare_hero_id"
  else info.kind = "auxiliary" end
  if rest:match("_old_") then info.kind = "auxiliary" end
  info.variant = info.old and "old" or info.v20 and "_20" or "current"
  if info.old and info.v20 then info.variant = "_20_old" end
  return info
end

local function scalar_fields(nodes)
  local out = {}
  for _, n in ipairs(nodes or {}) do
    if not n.children then
      out[#out + 1] = { key = n.key, value = n.value,
        line = n.line, missing_value = n.missing_value or nil, condition = n.condition,
        trailing_comment = n.trailing_comment }
    end
  end
  return out
end

local function behavior_flags(value)
  local flags = {}
  for flag in (value or ""):gmatch("DOTA_ABILITY_BEHAVIOR_([%w_]+)") do flags[#flags + 1] = flag end
  return flags
end

local function specials_of(block)
  local out = {}
  local special = kv.find(block.children, "AbilitySpecial")
  for _, entry in ipairs(special and special.children or {}) do
    local var_type = kv.value(entry.children, "var_type")
    for _, n in ipairs(entry.children or {}) do
      if n.key and n.key:lower() ~= "var_type" and not n.children then
        out[#out + 1] = { index = entry.key, var_type = var_type, name = n.key, value = n.value,
          ranks = kv.rank_values(n.value), line = n.line,
          missing_value = n.missing_value or nil }
      end
    end
  end
  local values = kv.find(block.children, "AbilityValues")
  for _, n in ipairs(values and values.children or {}) do
    local v = n.value or kv.value(n.children, "value")
    out[#out + 1] = { index = "AbilityValues", name = n.key, value = v,
      ranks = kv.rank_values(v), line = n.line }
  end
  return out
end

local function ability_facts(block)
  local keys, modifier_keys, script_refs, placeholders = {}, {}, {}, {}
  local has_radius_target, dot_modifier = false, false
  local seen_script = {}
  kv.walk(block.children, function(n, _, parent)
    if n.children and not (n.key or ""):match("^%d+$") then keys[n.key or "?"] = true end
    if n.key and n.key:match("^MODIFIER_") then modifier_keys[n.key] = true end
    if n.key == "Aura" then keys.Aura = true end
    if n.key == "Radius" and parent and parent.key == "Target" then has_radius_target = true end
    if n.key == "ThinkInterval" and parent and parent.children then
      local think = kv.find(parent.children, "OnIntervalThink")
      if think then
        kv.walk(think.children, function(x) if x.key == "Damage" then dot_modifier = true end end)
      end
    end
    if n.key == "ScriptFile" and n.value then
      local fn = parent and kv.value(parent.children, "Function")
      local ref = n.value .. "#" .. tostring(fn)
      if not seen_script[ref] then seen_script[ref] = true; script_refs[#script_refs + 1] = { file = n.value, ["function"] = fn } end
    end
    local name = kv.placeholder(n.value)
    if name then placeholders[name] = true end
  end)
  return keys, modifier_keys, script_refs, placeholders, has_radius_target, dot_modifier
end

local function sorted_keys(set)
  local out = {}
  for k in pairs(set) do out[#out + 1] = k end
  table.sort(out)
  return out
end

-- zh-TW names from langtext.lua, read as text. A line commented out with `--`
-- marks a hero the source website did not list.
local function parse_names(source_text)
  local names, inside = {}, false
  local line_no = 0
  for line in (source_text .. "\n"):gmatch("([^\n]*)\n") do
    line_no = line_no + 1
    if line:match("^_G%.hero_name_zh = {") then inside = true
    elseif inside and line:match("^}") then inside = false
    elseif inside then
      local id, name = line:match('^%s*([ABC]%d%d) = "(.-)",?')
      if id and not names[id] then names[id] = { name = name, line = line_no, raw = line:gsub("\r$", "") } end
    end
  end
  return names
end

-- herolist.txt: hero classes enabled for selection ("npc_dota_hero_x" "1", not commented out).
local function parse_herolist(source_text)
  local listed = {}
  for line in (source_text .. "\n"):gmatch("([^\n]*)\n") do
    local class, flag = line:match('^%s*"(npc_dota_hero_[%w_]+)"%s*"(%-?%d+)"')
    if class and flag == "1" then listed[class] = true end
  end
  return listed
end

local function tooltip_index(doc)
  local tokens = {}
  local lang = doc.nodes[1]
  local tok = lang and kv.find(lang.children, "Tokens")
  local dups = 0
  for _, n in ipairs(tok and tok.children or {}) do
    if n.key and n.value ~= nil then
      if tokens[n.key] then dups = dups + 1 else tokens[n.key] = n end
    end
  end
  return tokens, dups
end

local function tooltip_entry(tokens, key, text)
  local n = tokens[key]
  if not n then return nil end
  -- Tokens are stored only when they differ from the plain quoted form.
  local entry = { key = key, value = n.value, line = n.line, start = n.start, stop = n.stop,
    raw_sha256 = sha256(text:sub(n.start, n.stop)) }
  if n.key_token ~= "\"" .. key .. "\"" then entry.key_token = n.key_token end
  if n.value_token ~= "\"" .. n.value .. "\"" then entry.value_token = n.value_token end
  return entry
end

local function local_heroes(repo_root)
  local out = {}
  local dir = repo_root .. "/scripts/base_content/src/heroes"
  if lfs.attributes(dir, "mode") ~= "directory" then return out end
  local templates = repo_root .. "/scripts/lua_data/templates/heroes.lua"
  local template_text = lfs.attributes(templates, "mode") == "file" and read_file(templates) or ""
  for _, name in ipairs(sorted_dir(dir)) do
    local number, slug = name:match("^([ABC]%d%d)_(.+)$")
    if number and lfs.attributes(dir .. "/" .. name, "mode") == "directory" then
      local mod = read_file(dir .. "/" .. name .. "/mod.rs")
      -- First doc line looks like "//! 雜賀孫市（Saika Magoichi）— ..."; cut at
      -- the full-width paren with a plain find (byte classes break UTF-8).
      local first = mod:match("^//! ([^\r\n]+)")
      local zh = first
      local cut = first and (first:find("（", 1, true) or first:find("(", 1, true))
      if cut then zh = first:sub(1, cut - 1) end
      zh = zh and zh:gsub("%s+$", "")
      out[#out + 1] = { dir = "scripts/base_content/src/heroes/" .. name, number = number, game_hero_id = slug,
        name_zhtw = zh, in_templates = template_text:find('id = "' .. slug .. '"', 1, true) ~= nil }
    end
  end
  return out
end

-- options: { source, repository, commit, repo_root }
function M.build(options)
  local source = options.source
  local actual = M.checkout_commit(source)
  assert(actual, "cannot resolve checked-out commit under " .. source .. "/.git")
  assert(actual == options.commit, "source checkout is at " .. actual .. ", expected " .. options.commit)
  local catalog = {
    schema = M.SCHEMA,
    source = { repository = options.repository, commit = options.commit,
      executed_source_scripts = false,
      note = "Read-only text indexing. read_hero.lua / langtext.lua / vscripts are never executed." },
    files = {}, heroes = {}, abilities = {}, diagnostics = {},
  }
  local function diag(entry) catalog.diagnostics[#catalog.diagnostics + 1] = entry end

  -- Tooltips and names first so abilities can link to them.
  local tooltip_text = read_file(source .. "/" .. M.TOOLTIP_FILE)
  local tooltip_doc = kv.parse(tooltip_text)
  local tokens, dup_tokens = tooltip_index(tooltip_doc)
  local tooltip_raw = text.read_raw(source .. "/" .. M.TOOLTIP_FILE)
  catalog.files[#catalog.files + 1] = { path = M.TOOLTIP_FILE, role = "tooltips_zhtw", sha256 = sha256(tooltip_raw),
    bytes = #tooltip_raw, encoding = tooltip_raw:sub(1, 2) == "\255\254" and "utf-16le" or "utf-8", bom = tooltip_doc.bom, duplicate_tokens = dup_tokens, diagnostics = #tooltip_doc.diagnostics }
  for _, d in ipairs(tooltip_doc.diagnostics) do d.path = M.TOOLTIP_FILE; diag(d) end
  local name_text = read_file(source .. "/" .. M.NAME_FILE)
  local names = parse_names(name_text)
  catalog.files[#catalog.files + 1] = { path = M.NAME_FILE, role = "hero_names_text_only", sha256 = sha256(name_text),
    bytes = #name_text, executed = false }
  local herolist_text = read_file(source .. "/" .. M.HEROLIST_FILE)
  local herolist = parse_herolist(herolist_text)
  catalog.files[#catalog.files + 1] = { path = M.HEROLIST_FILE, role = "hero_selection_list", sha256 = sha256(herolist_text),
    bytes = #herolist_text }
  -- Unit fields a hero's X_2.txt omits fall back to npc_dota_hero_base (Dota inheritance).
  local defaults_text = read_file(source .. "/" .. M.HERO_DEFAULTS_FILE)
  local defaults_doc = kv.parse(defaults_text)
  local defaults_root = defaults_doc.nodes[1]
  local hero_base = defaults_root and kv.find(defaults_root.children, "npc_dota_hero_base")
  catalog.hero_unit_defaults = hero_base and scalar_fields(hero_base.children) or {}
  catalog.files[#catalog.files + 1] = { path = M.HERO_DEFAULTS_FILE, role = "hero_unit_defaults", sha256 = sha256(defaults_text),
    bytes = #defaults_text }

  local heroes_by_id = {}
  local ability_ids = {}
  for _, faction in ipairs(M.FACTIONS) do
    local rel_dir = M.NPC_DIR .. "/" .. faction.dir
    for _, name in ipairs(sorted_dir(source .. "/" .. rel_dir)) do
      local rel = rel_dir .. "/" .. name
      local hero_id, suffix = name:match("^([ABC]%d%d)(.-)%.txt$")
      local text = read_file(source .. "/" .. rel)
      local doc = kv.parse(text)
      local root = doc.nodes[1]
      local variant = (suffix == "" and "main") or (suffix == "_2" and "_2") or "unrecognized"
      local file = { path = rel, faction = faction.dir, source_hero_id = hero_id, variant = variant,
        sha256 = sha256(text), bytes = #text, bom = doc.bom, root_key = root and root.key,
        root_key_token = root and root.key_token, top_level_nodes = #doc.nodes,
        first_comment = root and root.comments and root.comments[1], diagnostics = #doc.diagnostics,
        main_pattern = name:match("^[ABC]%d%d%.txt$") ~= nil }
      catalog.files[#catalog.files + 1] = file
      for _, d in ipairs(doc.diagnostics) do d.path = rel; diag(d) end
      if hero_id and variant ~= "unrecognized" then
        local hero = heroes_by_id[hero_id]
        if not hero then
          hero = { source_hero_id = hero_id, faction = faction.dir, abilities = {} }
          heroes_by_id[hero_id] = hero
        end
        if variant == "main" then
          hero.main_file = rel
          hero.main_file_comment = file.first_comment
        else
          hero.unit_file = rel
        end
        for _, block in ipairs(root and root.children or {}) do
          local id = block.key
          local info = id and M.parse_ability_id(id) or { kind = "foreign" }
          local raw = text:sub(block.start, block.stop)
          local entry_common = { path = rel, line = block.line, col = block.col, start = block.start, stop = block.stop,
            key_token = block.key_token, raw = raw, raw_sha256 = sha256(raw), comments = block.comments }
          if not block.children then
            -- Top-level scalar such as "Version" "1".
            hero.file_scalars = hero.file_scalars or {}
            hero.file_scalars[#hero.file_scalars + 1] = { path = rel, key = id, value = block.value, line = block.line,
              key_token = block.key_token, value_token = block.value_token }
          elseif variant == "_2" and info.kind == "bare_hero_id" then
            entry_common.fields = scalar_fields(block.children)
            hero.unit = entry_common
            hero.ability_list = {}
            for _, f in ipairs(entry_common.fields) do
              local idx = f.key and f.key:match("^Ability(%d+)$")
              if idx and f.value and f.value ~= "" then
                hero.ability_list[#hero.ability_list + 1] = { slot_index = tonumber(idx), ability = f.value }
              end
              if f.key == "AttackCapabilities" then hero.attack_capabilities = f.value end
              if f.key == "AttackRange" then hero.attack_range = tonumber(f.value) end
              if f.key == "AttributePrimary" then hero.attribute_primary = f.value end
              if f.key == "override_hero" then hero.override_hero = f.value end
            end
            table.sort(hero.ability_list, function(a, b) return a.slot_index < b.slot_index end)
          else
            local entry = entry_common
            entry.id = id
            entry.source_hero_id = info.hero
            entry.owner_file_hero_id = hero_id
            entry.kind = info.kind
            entry.slot = info.slot
            entry.variant = info.variant
            entry.id_core = info.core
            entry.file_variant = variant
            if info.hero and info.hero ~= hero_id then
              diag({ kind = "ability_in_other_hero_file", path = rel, line = block.line, detail = id })
            end
            if ability_ids[id] then
              diag({ kind = "duplicate_ability_id", path = rel, line = block.line, detail = id .. " also in " .. ability_ids[id] })
              entry.duplicate_of = ability_ids[id]
            else
              ability_ids[id] = rel .. ":" .. block.line
            end
            entry.fields = scalar_fields(block.children)
            local flags = behavior_flags(kv.value(block.children, "AbilityBehavior"))
            entry.behavior_flags = flags
            entry.specials = specials_of(block)
            local keys, modifier_keys, script_refs, placeholders, radius_target, dot_mod = ability_facts(block)
            entry.block_keys = sorted_keys(keys)
            entry.modifier_keys = sorted_keys(modifier_keys)
            entry.script_refs = script_refs
            local missing_script = false
            for _, r in ipairs(script_refs) do
              local candidates = { source .. "/" .. M.VSCRIPTS_DIR .. "/" .. r.file, source .. "/" .. M.ADDON .. "/" .. r.file }
              r.present_in_source = false
              for _, c in ipairs(candidates) do
                if lfs.attributes(c, "mode") == "file" then r.present_in_source = true end
              end
              if not r.present_in_source then missing_script = true end
            end
            entry.depends_on_missing_vscript = missing_script
            local known = {}
            for _, s in ipairs(entry.specials) do known[s.name] = true end
            for _, f in ipairs(entry.fields) do if f.key then known[f.key] = true end end
            local unresolved = {}
            for _, p in ipairs(sorted_keys(placeholders)) do
              if not known[p] then unresolved[#unresolved + 1] = p end
            end
            entry.placeholders = sorted_keys(placeholders)
            if #unresolved > 0 then entry.unresolved_placeholders = unresolved end
            -- Tooltip linkage.
            local prefix = "DOTA_Tooltip_Ability_" .. id
            local tip = { name = tooltip_entry(tokens, prefix, tooltip_text),
              description = tooltip_entry(tokens, prefix .. "_Description", tooltip_text),
              lore = tooltip_entry(tokens, prefix .. "_Lore", tooltip_text) }
            for i = 0, 9 do
              local note = tooltip_entry(tokens, prefix .. "_Note" .. i, tooltip_text)
              if note then tip.notes = tip.notes or {}; tip.notes[#tip.notes + 1] = note end
            end
            for _, s in ipairs(entry.specials) do
              local label = tooltip_entry(tokens, prefix .. "_" .. s.name, tooltip_text)
              if label then tip.special_labels = tip.special_labels or {}; tip.special_labels[#tip.special_labels + 1] = label end
            end
            if tip.name or tip.description then entry.tooltip = tip end
            entry.display_name = tip.name and tip.name.value or nil
            entry.description_plain = tip.description and classify.plain_text(tip.description.value) or nil
            if info.kind == "precache" then
              entry.classification = { status = "not_applicable", basis = "none", families = {}, tags = {} }
            else
              entry.classification = classify.classify({ behavior_flags = flags, block_keys = keys,
                modifier_keys = modifier_keys, text = entry.description_plain,
                has_radius_target = radius_target, dot_modifier = dot_mod })
            end
            catalog.abilities[#catalog.abilities + 1] = entry
            hero.abilities[#hero.abilities + 1] = id
          end
        end
      end
    end
  end

  -- Hero rows with names and unit ability list cross-check.
  local hero_ids = sorted_keys(heroes_by_id)
  local by_ability = {}
  for _, a in ipairs(catalog.abilities) do by_ability[a.id] = a end
  for _, id in ipairs(hero_ids) do
    local hero = heroes_by_id[id]
    local nm = names[id]
    hero.name_zhtw = nm and nm.name or nil
    hero.listed_on_site = hero.override_hero ~= nil and herolist[hero.override_hero] == true
    hero.name_source = nm and { path = M.NAME_FILE, line = nm.line, raw = nm.raw } or nil
    hero.has_main_file = hero.main_file ~= nil
    for _, entry in ipairs(hero.ability_list or {}) do
      local a = by_ability[entry.ability]
      entry.indexed = a ~= nil
      if a then a.in_unit_ability_list = true; a.unit_slot_index = entry.slot_index end
    end
    catalog.heroes[#catalog.heroes + 1] = hero
  end

  catalog.id_mapping = M.map_local(catalog.heroes, local_heroes(options.repo_root))
  return catalog
end

-- Source hero ID -> local game hero ID. Matching is by zh-TW identity only;
-- an equal number with a different identity is a conflict, never a match.
function M.map_local(heroes, locals)
  local out = { rule = "match by zh-TW hero name; equal folder number alone never maps", locals = locals, entries = {} }
  local by_name, by_number = {}, {}
  for _, l in ipairs(locals) do
    if l.name_zhtw then by_name[l.name_zhtw] = l end
    by_number[l.number] = l
  end
  for _, h in ipairs(heroes) do
    local row = { source_hero_id = h.source_hero_id, source_name_zhtw = h.name_zhtw, status = "unmapped" }
    local named = h.name_zhtw and by_name[h.name_zhtw]
    local numbered = by_number[h.source_hero_id]
    if named and named.number == h.source_hero_id then
      row.status = "name_and_number_match"
      row.game_hero_id = named.game_hero_id
      row.local_dir = named.dir
      row.note = "identity match; kit version still has to be checked against the source variant (D3)"
    elseif named then
      row.status = "name_match_number_differs"
      row.game_hero_id = named.game_hero_id
      row.local_dir = named.dir
      row.note = "same hero name, local folder number " .. named.number .. " differs from source id"
    end
    if numbered and not (named and named == numbered) then
      row.number_conflict = { local_dir = numbered.dir, game_hero_id = numbered.game_hero_id,
        local_name_zhtw = numbered.name_zhtw,
        note = "CONFLICT: same number, different identity; do not pair by number" }
      if row.status == "unmapped" then row.status = "conflict_number_only" end
    end
    out.entries[#out.entries + 1] = row
  end
  return out
end

-- Re-reads every indexed span and checks raw text, key token and hashes.
function M.verify(catalog, source)
  local cache, problems, checked = {}, {}, 0
  local function text_of(rel)
    if not cache[rel] then cache[rel] = read_file(source .. "/" .. rel) end
    return cache[rel]
  end
  local function check(label, rel, span)
    local text = text_of(rel)
    local raw = text:sub(span.start, span.stop)
    checked = checked + 1
    if span.raw and raw ~= span.raw then problems[#problems + 1] = label .. ": raw text differs at " .. rel .. ":" .. tostring(span.line) end
    if sha256(raw) ~= span.raw_sha256 then problems[#problems + 1] = label .. ": raw hash differs" end
    -- Tooltip spans keep key/value + hash; the tokens default to the plain quoted form.
    local key_token = span.key_token or (span.key and ('"' .. span.key .. '"'))
    if key_token and raw:sub(1, #key_token) ~= key_token then
      problems[#problems + 1] = label .. ": key token not at recorded offset"
    end
    local value_token = span.value_token or (not span.raw and span.value and ('"' .. span.value .. '"'))
    if value_token and raw:sub(-#value_token) ~= value_token then
      problems[#problems + 1] = label .. ": value token not at recorded end offset"
    end
  end
  for _, f in ipairs(catalog.files) do
    if sha256(text.read_raw(source .. "/" .. f.path)) ~= f.sha256 then problems[#problems + 1] = "file hash differs: " .. f.path end
  end
  for _, a in ipairs(catalog.abilities) do
    check(a.id, a.path, a)
    if a.tooltip then
      for _, k in ipairs({ "name", "description", "lore" }) do
        if a.tooltip[k] then check(a.id .. "." .. k, M.TOOLTIP_FILE, a.tooltip[k]) end
      end
      for _, list in ipairs({ a.tooltip.notes or {}, a.tooltip.special_labels or {} }) do
        for _, t in ipairs(list) do check(a.id .. "." .. t.key, M.TOOLTIP_FILE, t) end
      end
    end
  end
  for _, h in ipairs(catalog.heroes) do
    if h.unit then check(h.source_hero_id .. ".unit", h.unit.path, h.unit) end
  end
  return problems, checked
end

return M
