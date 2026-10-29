-- HTML pages of the generated site, from a site catalog (site/census_adapter.lua or
-- --catalog). Every number on a page is written by one of three helpers so that the
-- tests can check it against the catalog:
--   value(owner, name, fmt, rank) -> <span class="n" data-v="owner/name" data-f data-r>
--   stat(key, x)                  -> <span class="n" data-stat="key">   (hero unit stats)
--   level(k)                      -> <span class="lv">                   (rank tabs)
-- plus the content hash in <code class="hash">. No other digit may appear in page text.
local Q = require("site.rational")
local R = require("site.text_render")
local I = require("site.i18n")
local M = {}

local esc = R.escape

-------------------------------------------------------------------------------
-- context
-------------------------------------------------------------------------------
function M.context(catalog)
  local C = { catalog = catalog, values = {}, by_id = { ability = {}, item = {}, status = {}, template = {}, hero = {} } }
  for owner, list in pairs(catalog.values) do
    local t = {}
    for _, v in ipairs(list) do t[v.name] = v end
    C.values[owner] = t
  end
  for _, a in ipairs(catalog.abilities) do C.by_id.ability[a.id] = a end
  for _, it in ipairs(catalog.items) do C.by_id.item[it.id] = it end
  for _, s in ipairs(catalog.statuses) do C.by_id.status[s.id] = s end
  for _, u in ipairs(catalog.unit_templates) do C.by_id.template[u.id] = u end
  for _, h in ipairs(catalog.heroes) do C.by_id.hero[h.id] = h end
  C.builds_into = {}
  for _, it in ipairs(catalog.items) do
    for _, c in ipairs(it.shop.recipe and it.shop.recipe.components or {}) do
      C.builds_into[c] = C.builds_into[c] or {}
      table.insert(C.builds_into[c], it.id)
    end
  end
  return C
end

local function find_entry(C, id)
  for _, kind in ipairs({ "ability", "item", "status", "template", "hero" }) do
    local e = C.by_id[kind][id]
    if e then return e, kind end
  end
end

-- plain name of any catalog entry in `lang` (names are literal-only templates)
function M.name(C, id, lang)
  local e = find_entry(C, id)
  local t = e and e.text and (e.text[lang] or e.text.zh_TW)
  if not t or not t.name then return nil end
  local out = {}
  for _, p in ipairs(t.name) do out[#out + 1] = p.lit or "" end
  return table.concat(out)
end

-------------------------------------------------------------------------------
-- number helpers
-------------------------------------------------------------------------------
function M.value(C, owner, name, fmt, rank, attrs)
  local v = C.values[owner] and C.values[owner][name]
  if not (v and v.exact) then
    -- the catalog lacks a value its own text refers to: show the gap (never a guessed number) and
    -- report it; build_site lists every one as a warning
    C.missing = C.missing or {}
    C.missing[owner .. "/" .. name] = true
    return string.format('<span class="missing-value" title="%s/%s">?</span>', esc(owner), esc(name))
  end
  fmt = fmt or "all"
  local runs = R.value_runs(v, fmt, { rank = rank })
  local inner = {}
  for _, r in ipairs(runs) do inner[#inner + 1] = r.b and ("<b>" .. esc(r.t) .. "</b>") or esc(r.t) end
  local class = "n"
  if v.damage_type then class = class .. " dt dt-" .. v.damage_type end
  return string.format('<span class="%s" data-v="%s/%s" data-f="%s" data-r="%s"%s>%s</span>',
    class, esc(owner), esc(name), fmt, rank and tostring(rank) or "", attrs or "", table.concat(inner))
end

function M.stat(key, x)
  if x == nil then return "—" end  -- the roster does not record it for this hero
  return string.format('<span class="n" data-stat="%s">%s</span>', key, Q.decimal(Q.parse(x), 2))
end

function M.level(k) return string.format('<span class="lv">%d</span>', k) end

-- a parsed template as HTML
function M.pieces(C, pieces, lang, rank)
  local out = {}
  for _, p in ipairs(pieces or {}) do
    if p.lit then out[#out + 1] = (esc(p.lit):gsub("\n", "<br>"))
    elseif p.name_of then out[#out + 1] = esc(assert(M.name(C, p.name_of, lang), "{@" .. p.name_of .. "} has no name"))
    elseif p.rules then
      local cur = C.catalog.rules
      for part in p.rules:gmatch("[^.]+") do cur = type(cur) == "table" and cur[part] or nil end
      assert(cur ~= nil and type(cur) ~= "table", "{rules." .. p.rules .. "} missing")
      out[#out + 1] = string.format('<span class="n" data-rule="%s">%s</span>', p.rules, R.number(Q.parse(cur), nil, p.fmt))
    else
      out[#out + 1] = M.value(C, p.owner, p.value, p.fmt, rank)
    end
  end
  return table.concat(out)
end

local function text_field(C, e, lang, field, rank)
  local t = e.text and e.text[lang]
  return t and t[field] and M.pieces(C, t[field], lang, rank) or nil
end

-------------------------------------------------------------------------------
-- layout
-------------------------------------------------------------------------------
local function lang_info(code)
  for _, l in ipairs(I.LANGS) do if l.code == code then return l end end
end

-- page: { lang, path (site-relative, e.g. "zh-TW/heroes/C17.html"), title, body }
function M.layout(C, page)
  local L = I.strings(page.lang)
  local depth = select(2, page.path:gsub("/", ""))
  local up = string.rep("../", depth)
  local li = lang_info(page.lang)
  local base = up .. li.dir .. "/"
  local switch = {}
  for _, other in ipairs(I.LANGS) do
    local target = up .. other.dir .. page.path:sub(#li.dir + 1)
    if other.code == page.lang then switch[#switch + 1] = '<span class="cur">' .. other.label .. "</span>"
    else switch[#switch + 1] = '<a href="' .. target .. '" hreflang="' .. other.html .. '">' .. other.label .. "</a>" end
  end
  local cat = C.catalog
  return table.concat({
    "<!doctype html>",
    '<html lang="' .. li.html .. '">',
    "<head>",
    '<meta charset="utf-8">',
    '<meta name="viewport" content="width=device-width, initial-scale=1">',
    "<title>" .. esc(page.title) .. " | " .. esc(L.site_title) .. "</title>",
    '<link rel="stylesheet" href="' .. up .. 'assets/style.css">',
    "</head>",
    "<body>",
    '<header class="top"><a class="brand" href="' .. base .. 'index.html">' .. esc(L.site_title) .. "</a>",
    '<nav><a href="' .. base .. 'index.html">' .. esc(L.heroes) .. '</a><a href="' .. base .. 'items/index.html">' .. esc(L.shop) ..
      '</a><a href="' .. base .. 'changes.html">' .. esc(L.changes) .. "</a></nav>",
    '<div class="langs" aria-label="' .. esc(L.language) .. '">' .. table.concat(switch, " ") .. "</div></header>",
    '<main>' .. page.body .. "</main>",
    '<footer><p>' .. esc(L.footer_note) .. "</p>",
    "<p>" .. esc(L.footer_hash) .. ' <code class="hash">' .. esc(cat.content_hash or "") .. "</code></p>",
    cat.source_catalog_hash and ("<p>" .. esc(L.source_catalog) .. ' <code class="hash">' .. esc(cat.source_catalog_hash) .. "</code></p>") or "",
    "<p>" .. esc(L.footer_source) .. " " .. esc(cat.roster and cat.roster.repository or "") .. ' <code class="hash">' ..
      esc(cat.roster and cat.roster.commit or "") .. "</code></p></footer>",
    "</body>",
    "</html>",
    "",
  }, "\n")
end

local function icon(C, rel, up, alt, class)
  local src = rel and (up .. "img/" .. rel:gsub("^.-/images/", "")) or (up .. "img/placeholder.svg")
  return string.format('<img class="%s" src="%s" alt="%s" loading="lazy">', class or "icon", esc(src), esc(alt or ""))
end

-------------------------------------------------------------------------------
-- info rows
-------------------------------------------------------------------------------
local ROW_ORDER = { "cooldown", "mana", "health", "gold", "charges", "charge_restore", "range", "radius",
  "vector_length", "cast_point", "channel_time", "required_level" }

local function unit_suffix(C, ref, L)
  local v = C.values[ref.owner][ref.name]
  if v.unit == "s" then return " " .. esc(L.seconds) end
  return ""
end

local function dl(rows)
  if #rows == 0 then return "" end
  local out = { '<dl class="row">' }
  for _, r in ipairs(rows) do out[#out + 1] = "<div><dt>" .. esc(r[1]) .. "</dt><dd>" .. r[2] .. "</dd></div>" end
  out[#out + 1] = "</dl>"
  return table.concat(out)
end

-- damage types of the visible values of an entry and its statuses
local function damage_types(C, e)
  local seen, list = {}, {}
  local owners = { e.id }
  for _, s in ipairs(e.statuses or {}) do owners[#owners + 1] = s end
  for _, o in ipairs(owners) do
    for _, v in pairs(C.values[o] or {}) do
      if v.damage_type and not v.hidden and not seen[v.damage_type] then seen[v.damage_type] = true; list[#list + 1] = v.damage_type end
    end
  end
  table.sort(list)
  return list
end

local function dispels(C, e)
  local seen, list = {}, {}
  for _, sid in ipairs(e.statuses or {}) do
    local s = C.by_id.status[sid]
    if s and s.visibility ~= "hidden" and s.dispel and not seen[s.dispel] then seen[s.dispel] = true; list[#list + 1] = s.dispel end
  end
  table.sort(list)
  return list
end

function M.info_rows(C, e, header, lang, rank, with_effects)
  local L = I.strings(lang)
  local rows = {}
  if header then
    if header.target_type and header.target_type ~= "none" then
      local t = L.target[header.target_type] or header.target_type
      if header.target_team then t = t .. "（" .. (L.team[header.target_team] or header.target_team) .. "）" end
      if lang == "en" then t = t:gsub("（", " ("):gsub("）", ")") end
      rows[#rows + 1] = { L.row.target, esc(t) }
    end
    for _, key in ipairs(ROW_ORDER) do
      local ref = header[key]
      if ref then rows[#rows + 1] = { L.row[key], M.value(C, ref.owner, ref.name, "all", rank) .. unit_suffix(C, ref, L) } end
    end
  end
  if with_effects then
    local dts = damage_types(C, e)
    if #dts > 0 then
      local names = {}
      for i, d in ipairs(dts) do names[i] = '<span class="dt dt-' .. d .. '">' .. esc(L.damage_type[d] or d) .. "</span>" end
      rows[#rows + 1] = { L.row.damage_type, table.concat(names, lang == "en" and ", " or "、") }
    end
    local flags = {}
    for _, f in ipairs(e.flags or (e.active and e.active.flags) or {}) do flags[f] = true end
    if #dts > 0 or #(e.statuses or {}) > 0 then
      rows[#rows + 1] = { L.row.pierce, esc(flags.pierce_magic_immune and L.yes or L.no) }
    end
    local ds = dispels(C, e)
    if #ds > 0 then
      local names = {}
      for i, d in ipairs(ds) do names[i] = esc(L.dispel[d] or d) end
      rows[#rows + 1] = { L.row.dispel, table.concat(names, lang == "en" and ", " or "、") }
    end
  end
  return dl(rows)
end

local function stat_lines(C, entries, lang, rank)
  if not entries or #entries == 0 then return "" end
  local L = I.strings(lang)
  local out = { '<ul class="stats">' }
  for _, s in ipairs(entries) do
    local label = L.stat[s.stat] or s.stat
    out[#out + 1] = "<li>" .. M.value(C, s.ref.owner, s.ref.name, "signed", rank) .. " " .. esc(label) .. "</li>"
  end
  out[#out + 1] = "</ul>"
  return table.concat(out)
end

local function statuses_block(C, e, lang, rank)
  local L = I.strings(lang)
  local items = {}
  for _, sid in ipairs(e.statuses or {}) do
    local s = C.by_id.status[sid]
    if s and s.visibility ~= "hidden" and s.text and s.text[lang] then
      local bits = {}
      if s.duration then bits[#bits + 1] = esc(L.duration) .. " " .. M.value(C, s.duration.owner, s.duration.name, "all", rank) .. unit_suffix(C, s.duration, L) end
      if s.infinite then bits[#bits + 1] = esc(L.infinite) end
      if s.dispel then bits[#bits + 1] = esc(L.dispel[s.dispel] or s.dispel) end
      local body = text_field(C, s, lang, "body", rank) or text_field(C, s, lang, "status_body", rank)
      items[#items + 1] = "<li><strong>" .. esc(M.name(C, sid, lang)) .. "</strong>" ..
        (#bits > 0 and ('<span class="meta">' .. table.concat(bits, " · ") .. "</span>") or "") ..
        (body and ("<p>" .. body .. "</p>") or "") .. "</li>"
    end
  end
  if #items == 0 then return "" end
  return '<div class="statuses"><h4>' .. esc(L.statuses) .. "</h4><ul>" .. table.concat(items) .. "</ul></div>"
end

-- rank tabs (CSS only): "all levels" plus one tab per rank; each panel is rendered by fn(rank)
local function rank_tabs(id, max_level, lang, fn)
  local L = I.strings(lang)
  if max_level <= 1 then return '<div class="panel">' .. fn(nil) .. "</div>" end
  local out = { '<div class="rs">' }
  for k = 0, max_level do
    local rid = "r-" .. id .. "-" .. k
    out[#out + 1] = '<input type="radio" name="r-' .. id .. '" id="' .. rid .. '"' .. (k == 0 and " checked" or "") .. ">"
    out[#out + 1] = '<label for="' .. rid .. '">' .. (k == 0 and esc(L.rank_all) or (esc(L.level) .. " " .. M.level(k))) .. "</label>"
  end
  for k = 0, max_level do out[#out + 1] = '<div class="panel">' .. fn(k > 0 and k or nil) .. "</div>" end
  out[#out + 1] = "</div>"
  return table.concat(out)
end

-------------------------------------------------------------------------------
-- hero pages
-------------------------------------------------------------------------------
local function ability_card(C, a, lang, up)
  local L = I.strings(lang)
  local name = M.name(C, a.id, lang) or a.id
  local badges = {}
  badges[#badges + 1] = '<span class="slot">' .. esc((a.slot == "sub" or not a.slot) and "·" or a.slot) .. "</span>"
  if a.kind and L.kind[a.kind] then badges[#badges + 1] = '<span class="kind">' .. esc(L.kind[a.kind]) .. "</span>" end
  if a.coverage == "pending" then badges[#badges + 1] = '<span class="badge warn">' .. esc(L.pending) .. "</span>" end
  local head = '<div class="ab-head">' .. icon(C, a.icon, up, name, "icon ab-icon") ..
    "<div><h3>" .. esc(name) .. "</h3>" .. table.concat(badges, " ") .. "</div></div>"
  local body = rank_tabs(a.id, a.max_level or 1, lang, function(rank)
    return M.info_rows(C, a, a.header, lang, rank, true) ..
      stat_lines(C, a.passive_stats, lang, rank) ..
      '<p class="desc">' .. (text_field(C, a, lang, "body", rank) or "") .. "</p>" ..
      statuses_block(C, a, lang, rank)
  end)
  return '<section class="ability" id="' .. esc(a.id) .. '">' .. head .. body .. "</section>"
end

local function hero_stats(h, lang)
  local L = I.strings(lang)
  local s = h.stats
  if not s then return "" end
  local prim = ({ strength = "str", agility = "agi", intelligence = "int" })[s.primary]
  local attr = { '<table class="attrs"><thead><tr><th></th><th>' .. esc(L.base) .. "</th><th>" .. esc(L.per_level) .. "</th></tr></thead><tbody>" }
  for _, a in ipairs({ { "str", "strength" }, { "agi", "agility" }, { "int", "intelligence" } }) do
    local cls = a[1] == prim and ' class="primary"' or ""
    attr[#attr + 1] = "<tr" .. cls .. "><th>" .. esc(L[a[1]]) .. "</th><td>" .. M.stat("base_" .. a[2], s["base_" .. a[2]]) ..
      "</td><td>+" .. M.stat(a[2] .. "_gain", s[a[2] .. "_gain"]) .. "</td></tr>"
  end
  attr[#attr + 1] = "</tbody></table>"
  local dmg = M.stat("attack_damage_min", s.attack_damage_min)
  if s.attack_damage_max ~= s.attack_damage_min then dmg = dmg .. "–" .. M.stat("attack_damage_max", s.attack_damage_max) end
  local rows = {
    { L.primary, esc(L[prim] or s.primary) },
    { L.hp, M.stat("health", s.health) },
    { L.hp_regen, M.stat("health_regen", s.health_regen) },
    { L.mana_pool, M.stat("mana", s.mana) },
    { L.mana_regen, M.stat("mana_regen", s.mana_regen) },
    { L.armor, M.stat("armor", s.armor) },
    { L.attack_damage, dmg },
    { L.attack_rate, M.stat("attack_rate", s.attack_rate) .. " " .. esc(L.seconds) },
    { L.attack_range, M.stat("attack_range", s.attack_range) },
    { L.move_speed, M.stat("movement_speed", s.movement_speed) },
  }
  return '<div class="hero-stats"><h2>' .. esc(L.attributes) .. "</h2>" .. table.concat(attr) .. dl(rows) .. "</div>"
end

function M.hero_page(C, h, lang)
  local L = I.strings(lang)
  local dir = lang_info(lang).dir
  local path = dir .. "/heroes/" .. h.id .. ".html"
  local up = "../../"
  local name = M.name(C, h.id, lang) or h.id
  local other = lang == "en" and M.name(C, h.id, "zh_TW") or M.name(C, h.id, "en")
  local main, rest = {}, {}
  for _, aid in ipairs(h.abilities) do
    local a = C.by_id.ability[aid]
    if a.slot == "hidden" then -- internal
    elseif a.slot == "sub" then rest[#rest + 1] = ability_card(C, a, lang, up)
    else main[#main + 1] = ability_card(C, a, lang, up) end
  end
  local body = '<div class="hero-head">' .. icon(C, h.icon, up, name, "portrait") ..
    "<div><h1>" .. esc(name) .. (h.unlisted and (' <span class="badge warn">' .. esc(L.unlisted) .. "</span>") or "") .. "</h1>" ..
    (other and ('<p class="sub">' .. esc(other) .. "</p>") or "") ..
    '<p class="sub"><code>' .. esc(h.id) .. "</code></p>" ..
    ((h.game_values and ('<p class="sub">' .. esc(L.game_values) .. "</p>")) or (h.unlisted and ('<p class="sub">' .. esc(L.source_values) .. "</p>")) or "") ..
    "</div></div>" ..
    hero_stats(h, lang) ..
    "<h2>" .. esc(L.abilities) .. "</h2>" .. table.concat(main) ..
    (#rest > 0 and ("<h2>" .. esc(L.sub_abilities) .. "</h2>" .. table.concat(rest)) or "")
  return { lang = lang, path = path, title = name, body = body }
end

function M.home_page(C, lang)
  local L = I.strings(lang)
  local dir = lang_info(lang).dir
  local listed, unlisted = {}, {}
  for _, h in ipairs(C.catalog.heroes) do
    local name = M.name(C, h.id, lang) or h.id
    local list = h.unlisted and unlisted or listed
    list[#list + 1] = '<a class="tile' .. (h.unlisted and " unlisted" or "") .. '" href="heroes/' .. esc(h.id) .. '.html">' ..
      icon(C, h.icon, "../", name, "portrait") .. "<span>" .. esc(name) .. "</span></a>"
  end
  local grids = '<div class="grid">' .. table.concat(listed) .. "</div>"
  if #unlisted > 0 then
    grids = (#listed > 0 and ("<h2>" .. esc(L.listed) .. "</h2>" .. grids) or "") ..
      "<h2>" .. esc(L.unlisted) .. '</h2><div class="grid">' .. table.concat(unlisted) .. "</div>"
  end
  local body = "<h1>" .. esc(L.hero_list) .. "</h1>" .. grids ..
    '<p class="more"><a href="items/index.html">' .. esc(L.shop) .. '</a> · <a href="changes.html">' .. esc(L.changes) .. "</a></p>"
  return { lang = lang, path = dir .. "/index.html", title = L.home, body = body }
end

-------------------------------------------------------------------------------
-- items
-------------------------------------------------------------------------------
local function item_link(C, id, lang, up)
  local it = C.by_id.item[id]
  local L = I.strings(lang)
  if not it then
    local open, close = "（", "）"
    if lang == "en" then open, close = " (", ")" end
    return '<span class="missing">' .. icon(C, nil, up, id, "icon sm") .. "<code>" .. esc(id) .. "</code>" .. open .. esc(L.not_in_catalog) .. close .. "</span>"
  end
  local name = M.name(C, id, lang) or id
  local cost = it.shop.cost and (' <span class="gold">' .. M.value(C, it.shop.cost.owner, it.shop.cost.name, "all") .. "</span>") or ""
  return '<a href="' .. esc(id) .. '.html">' .. icon(C, it.icon, up, name, "icon sm") .. esc(name) .. "</a>" .. cost
end

local function recipe_tree(C, id, lang, up, seen)
  seen = seen or {}
  local it = C.by_id.item[id]
  local L = I.strings(lang)
  local out = { "<li>" .. item_link(C, id, lang, up) }
  if it and it.shop.recipe and not seen[id] then
    seen[id] = true
    out[#out + 1] = "<ul>"
    for _, c in ipairs(it.shop.recipe.components) do out[#out + 1] = recipe_tree(C, c, lang, up, seen) end
    if it.shop.recipe.recipe_cost then
      local rc = it.shop.recipe.recipe_cost
      out[#out + 1] = '<li class="rc">' .. esc(L.recipe_cost) .. ' <span class="gold">' .. M.value(C, rc.owner, rc.name, "all") .. "</span></li>"
    end
    out[#out + 1] = "</ul>"
    seen[id] = nil
  end
  out[#out + 1] = "</li>"
  return table.concat(out)
end

function M.item_page(C, it, lang)
  local L = I.strings(lang)
  local dir = lang_info(lang).dir
  local up = "../../"
  local name = M.name(C, it.id, lang) or it.id
  local other = lang == "en" and M.name(C, it.id, "zh_TW") or M.name(C, it.id, "en")
  local rows = {}
  if it.shop.cost then rows[#rows + 1] = { L.cost, '<span class="gold">' .. M.value(C, it.shop.cost.owner, it.shop.cost.name, "all") .. "</span>" } end
  if it.shop.category then rows[#rows + 1] = { L.category, esc(L.shop_category[it.shop.category] or it.shop.category) } end
  if it.shop.charges then rows[#rows + 1] = { L.consumable_charges, M.value(C, it.shop.charges.owner, it.shop.charges.name, "all") } end
  if it.shop.sellable == false then rows[#rows + 1] = { L.shop, esc(L.sellable_no) } end
  local badges = it.coverage == "pending" and (' <span class="badge warn">' .. esc(L.pending) .. "</span>") or ""
  if it.unlisted then badges = badges .. ' <span class="badge warn">' .. esc(L.unlisted) .. "</span>" end
  local body = {
    '<div class="hero-head">' .. icon(C, it.icon, up, name, "icon big") .. "<div><h1>" .. esc(name) .. badges .. "</h1>" ..
      (other and ('<p class="sub">' .. esc(other) .. "</p>") or "") .. '<p class="sub"><code>' .. esc(it.id) .. "</code></p></div></div>",
    dl(rows),
    stat_lines(C, it.stats, lang, nil),
  }
  local desc = text_field(C, it, lang, "body", nil)
  if desc then body[#body + 1] = '<p class="desc">' .. desc .. "</p>" end
  if it.active then
    body[#body + 1] = "<h2>" .. esc(L.active) .. "</h2>" .. M.info_rows(C, it, it.active.header, lang, nil, true)
  elseif #(it.statuses or {}) > 0 then
    body[#body + 1] = M.info_rows(C, it, nil, lang, nil, true)
  end
  body[#body + 1] = statuses_block(C, it, lang, nil)
  if it.shop.recipe then
    body[#body + 1] = "<h2>" .. esc(L.recipe) .. '</h2><ul class="tree">' .. recipe_tree(C, it.id, lang, up) .. "</ul>"
  end
  local into = C.builds_into[it.id]
  if into then
    local links = {}
    for _, id in ipairs(into) do links[#links + 1] = "<li>" .. item_link(C, id, lang, up) .. "</li>" end
    body[#body + 1] = "<h2>" .. esc(L.builds_into) .. "</h2><ul>" .. table.concat(links) .. "</ul>"
  end
  return { lang = lang, path = dir .. "/items/" .. it.id .. ".html", title = name, body = table.concat(body) }
end

function M.shop_page(C, lang)
  local L = I.strings(lang)
  local dir = lang_info(lang).dir
  local groups, order = {}, {}
  for _, c in ipairs(C.catalog.shop_order or {}) do order[#order + 1] = c end
  for _, it in ipairs(C.catalog.items) do
    local c = it.shop.category or "_other"
    if not groups[c] then
      groups[c] = {}
      local known = false
      for _, k in ipairs(order) do if k == c then known = true end end
      if not known then order[#order + 1] = c end
    end
    table.insert(groups[c], it)
  end
  local out = { "<h1>" .. esc(L.shop) .. "</h1>" }
  for _, c in ipairs(order) do
    local list = groups[c]
    if list then
      table.sort(list, function(a, b)
        local ca, cb = C.values[a.shop.cost.owner][a.shop.cost.name], C.values[b.shop.cost.owner][b.shop.cost.name]
        local cmp = Q.cmp(Q.parse(ca.exact[1]), Q.parse(cb.exact[1]))
        if cmp ~= 0 then return cmp < 0 end
        return a.id < b.id
      end)
      local title = c == "_other" and L.other_category or (L.shop_category[c] or c)
      out[#out + 1] = "<h2>" .. esc(title) .. '</h2><div class="grid items">'
      for _, it in ipairs(list) do
        local name = M.name(C, it.id, lang) or it.id
        out[#out + 1] = '<a class="tile" href="' .. esc(it.id) .. '.html">' .. icon(C, it.icon, "../../", name, "icon big") ..
          "<span>" .. esc(name) .. '</span><span class="gold">' .. M.value(C, it.shop.cost.owner, it.shop.cost.name, "all") .. "</span></a>"
      end
      out[#out + 1] = "</div>"
    end
  end
  local deferred = C.catalog.deferred_items or {}
  if #deferred > 0 then
    out[#out + 1] = "<h2>" .. esc(L.deferred_items) .. '</h2><ul class="deferred">'
    for _, d in ipairs(deferred) do
      local name = d.name and (d.name[lang] or d.name.zh_TW)
      out[#out + 1] = "<li>" .. (name and esc(name) .. " " or "") .. "<code>" .. esc(d.id or d.source_id or "") .. "</code>" ..
        (d.reason and (' <code class="reason">' .. esc(d.reason) .. "</code>") or "") .. "</li>"
    end
    out[#out + 1] = "</ul>"
  end
  return { lang = lang, path = dir .. "/items/index.html", title = L.shop, body = table.concat(out) }
end

return M
