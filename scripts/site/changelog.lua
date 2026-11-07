-- Patch notes from two site catalogs (value-single-source §4): every value whose ranks,
-- unit or damage type changed, plus owners that were added or removed. Nothing is
-- written by hand; the numbers are the two catalogs' own values.
local P = require("site.pages")
local I = require("site.i18n")
local R = require("site.text_render")
local M = {}

local esc = R.escape

local function same_value(a, b)
  if a.unit ~= b.unit or a.damage_type ~= b.damage_type or (a.exact == nil) ~= (b.exact == nil) then return false end
  if not a.exact then return a.expr == b.expr end
  if #a.exact ~= #b.exact then return false end
  for i = 1, #a.exact do if a.exact[i] ~= b.exact[i] then return false end end
  return true
end

-- -> sorted list of { owner, added = true | removed = true | changes = { { name, old, new } } }
function M.diff(old, new)
  local owners = {}
  for o in pairs(old.values) do owners[o] = true end
  for o in pairs(new.values) do owners[o] = true end
  local ids = {}
  for o in pairs(owners) do ids[#ids + 1] = o end
  table.sort(ids)
  local out = {}
  for _, o in ipairs(ids) do
    local a, b = old.values[o], new.values[o]
    if not a then out[#out + 1] = { owner = o, added = true }
    elseif not b then out[#out + 1] = { owner = o, removed = true }
    else
      local ai, bi, names = {}, {}, {}
      for _, v in ipairs(a) do ai[v.name] = v; names[v.name] = true end
      for _, v in ipairs(b) do bi[v.name] = v; names[v.name] = true end
      local list = {}
      for n in pairs(names) do list[#list + 1] = n end
      table.sort(list)
      local changes = {}
      for _, n in ipairs(list) do
        if not n:match("^__") then
          local x, y = ai[n], bi[n]
          if not x or not y or not same_value(x, y) then changes[#changes + 1] = { name = n, old = x, new = y } end
        end
      end
      if #changes > 0 then out[#out + 1] = { owner = o, changes = changes } end
    end
  end
  return out
end

-- HTML: display names, or the id in <code> when an owner has none (ids carry digits)
local function label(C, owner, lang)
  local function named(id) local n = P.name(C, id, lang); return n and esc(n) or ("<code>" .. esc(id) .. "</code>") end
  local a = C.by_id.ability[owner]
  if a and a.hero then return named(a.hero) .. " · " .. named(owner) end
  local s = C.by_id.status[owner]
  if s and s.owner then return label(C, s.owner, lang) .. " · " .. named(owner) end
  return named(owner)
end

-- Cold/Cnew: page contexts of the previous and current catalogs (nil Cold: no comparison)
function M.page(Cold, Cnew, lang, dir)
  local L = I.strings(lang)
  local out = { "<h1>" .. esc(L.changes) .. "</h1><p>" .. esc(L.changes_intro) .. "</p>" }
  if not Cold then
    out[#out + 1] = "<p>" .. esc(L.changes_no_previous) .. "</p>"
  else
    out[#out + 1] = '<p class="meta">' .. esc(L.changes_from) .. ' <code class="hash">' .. esc(Cold.catalog.content_hash or "") ..
      "</code> → " .. esc(L.changes_to) .. ' <code class="hash">' .. esc(Cnew.catalog.content_hash or "") .. "</code></p>"
    local entries = M.diff(Cold.catalog, Cnew.catalog)
    if #entries == 0 then out[#out + 1] = "<p>" .. esc(L.changes_none) .. "</p>" end
    for _, e in ipairs(entries) do
      if e.added then
        out[#out + 1] = '<h3>' .. label(Cnew, e.owner, lang) .. ' <span class="badge">' .. esc(L.added) .. "</span></h3>"
      elseif e.removed then
        out[#out + 1] = '<h3>' .. label(Cold, e.owner, lang) .. ' <span class="badge warn">' .. esc(L.removed) .. "</span></h3>"
      else
        out[#out + 1] = "<h3>" .. label(Cnew, e.owner, lang) .. "</h3><ul>"
        for _, c in ipairs(e.changes) do
          local before = c.old and c.old.exact and P.value(Cold, e.owner, c.name, "all", nil, ' data-c="old"') or "—"
          local after = c.new and c.new.exact and P.value(Cnew, e.owner, c.name, "all", nil, ' data-c="new"') or "—"
          local hidden = (c.new or c.old).hidden and ' <span class="badge">hidden</span>' or ""
          out[#out + 1] = "<li><code>" .. esc(c.name) .. "</code>" .. hidden .. "：" .. before .. " → " .. after .. "</li>"
        end
        out[#out + 1] = "</ul>"
      end
    end
  end
  return { lang = lang, path = dir .. "/changes.html", title = L.changes, body = table.concat(out) }
end

return M
